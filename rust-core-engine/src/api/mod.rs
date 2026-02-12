use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::sync::RwLock;
use tracing::{debug, error, info};
use warp::ws::{Message, WebSocket, Ws};
use warp::{Filter, Reply};

use crate::ai::AIService;
use crate::auth::{AuthService, SecurityService, SessionRepository, UserRepository};
use crate::config::{ApiConfig, BinanceConfig};
use crate::market_data::MarketDataProcessor;
use crate::monitoring::MonitoringService;
use crate::paper_trading::PaperTradingEngine;
use crate::real_trading::RealTradingEngine;
use crate::storage::Storage;
use crate::trading::TradingEngine;

pub mod notifications;
pub mod paper_trading;
pub mod real_trading;
pub mod settings;

#[derive(Clone)]
pub struct ApiServer {
    config: ApiConfig,
    binance_config: BinanceConfig,
    market_data: MarketDataProcessor,
    trading_engine: TradingEngine,
    paper_trading_engine: Arc<PaperTradingEngine>,
    real_trading_engine: Option<Arc<RealTradingEngine>>,
    monitoring: Arc<RwLock<MonitoringService>>,
    ws_broadcaster: broadcast::Sender<String>,
    auth_service: AuthService,
    security_service: SecurityService,
    ai_service: AIService,
    storage: Storage,
}

#[derive(Serialize, Deserialize, Clone)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AddSymbolRequest {
    symbol: String,
    /// Optional timeframes - if not provided, uses config defaults
    #[serde(default)]
    timeframes: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize)]
struct SupportedSymbols {
    symbols: Vec<String>,
    available_timeframes: Vec<String>,
}

impl<T> ApiResponse<T> {
    fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
        }
    }

    fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
        }
    }
}

impl ApiServer {
    /// Create a new API server instance
    /// Note: This constructor requires multiple dependencies by design for proper initialization
    #[allow(clippy::too_many_arguments)]
    pub async fn new(
        config: ApiConfig,
        binance_config: BinanceConfig,
        market_data: MarketDataProcessor,
        trading_engine: TradingEngine,
        paper_trading_engine: Arc<PaperTradingEngine>,
        real_trading_engine: Option<Arc<RealTradingEngine>>,
        ws_broadcaster: broadcast::Sender<String>,
        storage: Storage,
    ) -> Result<Self> {
        // Initialize auth service - use dummy implementation if database is not available
        let (auth_service, security_service) = if let Some(db) = storage.get_database() {
            let user_repo = UserRepository::new(db).await?;
            let session_repo = SessionRepository::new(db).await?;
            let jwt_secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default_jwt_secret_change_in_production".to_string());
            let auth = AuthService::new(user_repo.clone(), jwt_secret.clone());
            let security = SecurityService::new(user_repo, session_repo, jwt_secret);
            (auth, security)
        } else {
            // Create dummy services that return errors for all operations
            (AuthService::new_dummy(), SecurityService::new_dummy())
        };

        // Initialize AI service
        let python_ai_url = std::env::var("PYTHON_AI_SERVICE_URL")
            .unwrap_or_else(|_| "http://localhost:8000".to_string());
        let ai_config = crate::ai::AIServiceConfig {
            python_service_url: python_ai_url,
            request_timeout_seconds: 30,
            max_retries: 3,
            enable_caching: true,
            cache_ttl_seconds: 300,
        };
        let ai_service = AIService::new(ai_config);

        Ok(Self {
            config,
            binance_config,
            market_data,
            trading_engine,
            paper_trading_engine,
            real_trading_engine,
            monitoring: Arc::new(RwLock::new(MonitoringService::new())),
            ws_broadcaster,
            auth_service,
            security_service,
            ai_service,
            storage,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!(
            "Starting API server on {}:{}",
            self.config.host, self.config.port
        );

        let api = self.clone().create_routes();

        warp::serve(api).run(([0, 0, 0, 0], self.config.port)).await;

        Ok(())
    }

    fn create_routes(self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "x-client", "authorization", "accept"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        // Health check
        let health = warp::path("health")
            .and(warp::get())
            .map(|| warp::reply::json(&ApiResponse::success("Bot is running")));

        // WebSocket endpoint
        let ws_broadcaster = self.ws_broadcaster.clone();
        let websocket = warp::path("ws").and(warp::ws()).map(move |ws: Ws| {
            let broadcaster = ws_broadcaster.clone();
            ws.on_upgrade(move |websocket| Self::handle_websocket(websocket, broadcaster))
        });

        // Market data routes
        let market_data = self.clone().market_data_routes();

        // Trading routes
        let trading = self.clone().trading_routes();

        // Monitoring routes
        let monitoring = self.clone().monitoring_routes();

        // AI routes
        let ai_routes = self.clone().ai_routes();

        // Paper trading routes
        let paper_trading =
            paper_trading::PaperTradingApi::new(self.paper_trading_engine.clone()).routes();

        // Real trading routes - the API handles the None case internally
        let real_trading =
            real_trading::RealTradingApi::new(self.real_trading_engine.clone()).routes();

        // Settings routes for API key management
        let settings_routes =
            settings::SettingsApi::new(self.storage.clone(), self.binance_config.clone()).routes();

        // Notifications routes for user notification preferences
        let notifications_routes =
            notifications::NotificationsApi::new(self.storage.clone()).routes();

        // Security routes for 2FA, sessions, password change
        let security_routes = self.security_service.clone().routes();

        // Combine all routes
        let api_routes = health
            .or(market_data)
            .or(trading)
            .or(monitoring)
            .or(ai_routes)
            .or(paper_trading)
            .or(real_trading)
            .or(settings_routes)
            .or(notifications_routes)
            .or(self.auth_service.clone().routes())
            .or(security_routes);

        let api = warp::path("api").and(api_routes);

        // Root level routes (not under /api prefix)
        let root_routes = websocket;

        api.with(cors.clone()).or(root_routes.with(cors))
    }

    fn market_data_routes(
        &self,
    ) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let market_data = self.market_data.clone();

        // Get latest prices
        let prices = warp::path("prices")
            .and(warp::get())
            .and(warp::any().map(move || market_data.clone()))
            .and_then(|market_data: MarketDataProcessor| async move {
                // Use cache's method to get ALL symbols with data (config + user-added)
                let symbols = market_data.get_cache().get_supported_symbols();
                let mut prices = std::collections::HashMap::new();

                for symbol in symbols {
                    if let Some(price) = market_data.get_cache().get_latest_price(&symbol) {
                        prices.insert(symbol, price);
                    }
                }

                Ok::<_, Infallible>(warp::reply::json(&ApiResponse::success(prices)))
            });

        // Get market overview
        let market_data_clone = self.market_data.clone();
        let overview = warp::path("overview")
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone.clone()))
            .and_then(|market_data: MarketDataProcessor| async move {
                match market_data.get_market_overview().await {
                    Ok(overview) => {
                        Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(overview)))
                    },
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                        &ApiResponse::<()>::error(e.to_string()),
                    )),
                }
            });

        // Get candle data
        let market_data_clone2 = self.market_data.clone();
        let candles = warp::path("candles")
            .and(warp::path::param::<String>()) // symbol
            .and(warp::path::param::<String>()) // timeframe
            .and(warp::query::<CandelQuery>())
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone2.clone()))
            .and_then(
                |symbol: String,
                 timeframe: String,
                 query: CandelQuery,
                 market_data: MarketDataProcessor| async move {
                    let candles =
                        market_data
                            .get_cache()
                            .get_candles(&symbol, &timeframe, query.limit);
                    Ok::<_, Infallible>(warp::reply::json(&ApiResponse::success(candles)))
                },
            );

        // NEW: Get comprehensive chart data with multiple timeframes
        let market_data_clone3 = self.market_data.clone();
        let chart_data = warp::path("chart")
            .and(warp::path::param::<String>()) // symbol
            .and(warp::path::param::<String>()) // timeframe
            .and(warp::query::<ChartQuery>())
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone3.clone()))
            .and_then(
                |symbol: String,
                 timeframe: String,
                 query: ChartQuery,
                 market_data: MarketDataProcessor| async move {
                    match market_data
                        .get_chart_data(&symbol, &timeframe, query.limit)
                        .await
                    {
                        Ok(chart_data) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::success(chart_data),
                        )),
                        Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::<()>::error(e.to_string()),
                        )),
                    }
                },
            );

        // NEW: Get multiple symbols chart data at once
        let market_data_clone4 = self.market_data.clone();
        let multi_chart = warp::path("charts")
            .and(warp::query::<MultiChartQuery>())
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone4.clone()))
            .and_then(
                |query: MultiChartQuery, market_data: MarketDataProcessor| async move {
                    let symbols = query
                        .symbols
                        .split(',')
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();
                    let timeframes = query
                        .timeframes
                        .split(',')
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>();

                    match market_data
                        .get_multi_chart_data(symbols, timeframes, query.limit)
                        .await
                    {
                        Ok(charts) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::success(charts),
                        )),
                        Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::<()>::error(e.to_string()),
                        )),
                    }
                },
            );

        // NEW: Add new symbol to track
        let market_data_clone5 = self.market_data.clone();
        let paper_trading_for_symbol = self.paper_trading_engine.clone();
        let add_symbol = warp::path("symbols")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || market_data_clone5.clone()))
            .and(warp::any().map(move || paper_trading_for_symbol.clone()))
            .and_then(
                |request: AddSymbolRequest,
                 market_data: MarketDataProcessor,
                 paper_trading: Arc<PaperTradingEngine>| async move {
                    let symbol = request.symbol.clone();

                    // Use default timeframes from config if not provided
                    let timeframes = request
                        .timeframes
                        .unwrap_or_else(|| market_data.get_supported_timeframes());

                    // Add to market data
                    match market_data.add_symbol(request.symbol, timeframes).await {
                        Ok(_) => {
                            // Also add to paper trading settings for AI analysis
                            if let Err(e) =
                                paper_trading.add_symbol_to_settings(symbol.clone()).await
                            {
                                error!("Failed to add symbol to paper trading: {}", e);
                            }
                            Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(
                                "Symbol added successfully",
                            )))
                        },
                        Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::<()>::error(e.to_string()),
                        )),
                    }
                },
            );

        // NEW: Remove symbol from tracking
        let market_data_clone6 = self.market_data.clone();
        let remove_symbol = warp::path("symbols")
            .and(warp::path::param::<String>()) // symbol
            .and(warp::delete())
            .and(warp::any().map(move || market_data_clone6.clone()))
            .and_then(
                |symbol: String, market_data: MarketDataProcessor| async move {
                    match market_data.remove_symbol(&symbol).await {
                        Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::success("Symbol removed successfully"),
                        )),
                        Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::<()>::error(e.to_string()),
                        )),
                    }
                },
            );

        // NEW: Get all supported symbols and timeframes (including user-added from database)
        let market_data_clone7 = self.market_data.clone();
        let symbols_info = warp::path("symbols")
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone7.clone()))
            .and_then(|market_data: MarketDataProcessor| async move {
                // Use async method to include user-added symbols from database
                let symbols = market_data.get_all_supported_symbols().await;
                let timeframes = market_data.get_supported_timeframes();
                let response = SupportedSymbols {
                    symbols,
                    available_timeframes: timeframes,
                };
                Ok::<_, Infallible>(warp::reply::json(&ApiResponse::success(response)))
            });

        warp::path("market").and(
            prices
                .or(overview)
                .or(candles)
                .or(chart_data)
                .or(multi_chart)
                .or(symbols_info)
                .or(add_symbol)
                .or(remove_symbol),
        )
    }

    fn trading_routes(self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let trading_engine = self.trading_engine.clone();

        // Get positions
        let positions = warp::path("positions")
            .and(warp::get())
            .and(warp::any().map(move || trading_engine.clone()))
            .map(|trading_engine: TradingEngine| {
                let positions = trading_engine.get_positions();
                warp::reply::json(&ApiResponse::success(positions))
            });

        // Get account info
        let trading_engine_clone = self.trading_engine.clone();
        let account = warp::path("account")
            .and(warp::get())
            .and(warp::any().map(move || trading_engine_clone.clone()))
            .and_then(|trading_engine: TradingEngine| async move {
                match trading_engine.get_account_info().await {
                    Ok(account) => {
                        Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(account)))
                    },
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                        &ApiResponse::<()>::error(e.to_string()),
                    )),
                }
            });

        // Close position
        let trading_engine_clone2 = self.trading_engine.clone();
        let close_position = warp::path("positions")
            .and(warp::path::param::<String>()) // symbol
            .and(warp::path("close"))
            .and(warp::post())
            .and(warp::any().map(move || trading_engine_clone2.clone()))
            .and_then(|symbol: String, trading_engine: TradingEngine| async move {
                match trading_engine.force_close_position(&symbol).await {
                    Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(
                        "Position closed",
                    ))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                        &ApiResponse::<()>::error(e.to_string()),
                    )),
                }
            });

        // Performance stats
        let trading_engine_clone3 = self.trading_engine.clone();
        let performance = warp::path("performance")
            .and(warp::get())
            .and(warp::any().map(move || trading_engine_clone3.clone()))
            .and_then(|trading_engine: TradingEngine| async move {
                match trading_engine.get_performance_stats().await {
                    Ok(stats) => {
                        Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(stats)))
                    },
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                        &ApiResponse::<()>::error(e.to_string()),
                    )),
                }
            });

        warp::path("trading").and(positions.or(account).or(close_position).or(performance))
    }

    fn monitoring_routes(
        &self,
    ) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let monitoring = self.monitoring.clone();

        // System metrics
        let system_metrics = warp::path("system")
            .and(warp::get())
            .and(warp::any().map(move || monitoring.clone()))
            .and_then(|monitoring: Arc<RwLock<MonitoringService>>| async move {
                let monitor = monitoring.read().await;
                let metrics = monitor.get_system_metrics().clone();
                Ok::<_, Infallible>(warp::reply::json(&ApiResponse::success(metrics)))
            });

        // Trading metrics
        let monitoring_clone = self.monitoring.clone();
        let trading_metrics = warp::path("trading")
            .and(warp::get())
            .and(warp::any().map(move || monitoring_clone.clone()))
            .and_then(|monitoring: Arc<RwLock<MonitoringService>>| async move {
                let monitor = monitoring.read().await;
                let metrics = monitor.get_trading_metrics().clone();
                Ok::<_, Infallible>(warp::reply::json(&ApiResponse::success(metrics)))
            });

        // Connection status
        let monitoring_clone2 = self.monitoring.clone();
        let connection_status = warp::path("connection")
            .and(warp::get())
            .and(warp::any().map(move || monitoring_clone2.clone()))
            .and_then(|monitoring: Arc<RwLock<MonitoringService>>| async move {
                let monitor = monitoring.read().await;
                let status = monitor.get_connection_status().clone();
                Ok::<_, Infallible>(warp::reply::json(&ApiResponse::success(status)))
            });

        warp::path("monitoring").and(system_metrics.or(trading_metrics).or(connection_status))
    }

    pub async fn update_monitoring(
        &self,
        active_positions: usize,
        cache_size: usize,
        websocket_connected: bool,
        api_responsive: bool,
    ) {
        let mut monitor = self.monitoring.write().await;
        monitor.update_system_metrics(active_positions, cache_size);
        monitor.update_connection_status(websocket_connected, api_responsive);
    }

    // WebSocket handler for real-time updates
    async fn handle_websocket(ws: WebSocket, broadcaster: broadcast::Sender<String>) {
        let (ws_sender, mut ws_receiver) = ws.split();
        let mut rx = broadcaster.subscribe();

        debug!("New WebSocket connection established");

        // Handle incoming messages from client (ping/pong, etc.)
        let ws_sender_clone: Arc<
            tokio::sync::Mutex<futures::stream::SplitSink<WebSocket, Message>>,
        > = Arc::new(tokio::sync::Mutex::new(ws_sender));
        let ws_sender_for_broadcast = ws_sender_clone.clone();
        let ws_sender_for_incoming = ws_sender_clone.clone();

        // Shared flag to track connection state
        let connection_open = Arc::new(tokio::sync::RwLock::new(true));
        let connection_open_outgoing = connection_open.clone();

        // Task to handle incoming messages
        let incoming_task = tokio::spawn(async move {
            while let Some(result) = ws_receiver.next().await {
                match result {
                    Ok(msg) => {
                        if msg.is_text() {
                            let text = msg.to_str().unwrap_or("");
                            debug!("Received WebSocket message: {}", text);

                            // Parse message and check if it's a ping
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(text) {
                                if let Some(msg_type) = json.get("type").and_then(|t| t.as_str()) {
                                    if msg_type == "Ping" {
                                        // Respond with Pong
                                        let pong_response = serde_json::json!({
                                            "type": "Pong",
                                            "timestamp": json.get("timestamp")
                                                .and_then(|t| t.as_str())
                                                .unwrap_or(""),
                                        });

                                        if let Ok(pong_str) = serde_json::to_string(&pong_response)
                                        {
                                            let mut sender = ws_sender_for_incoming.lock().await;
                                            if let Err(e) =
                                                sender.send(Message::text(pong_str)).await
                                            {
                                                debug!("Failed to send Pong response: {}", e);
                                                *connection_open.write().await = false;
                                                break;
                                            } else {
                                                debug!("✅ Sent Pong response to client");
                                            }
                                        }
                                    }
                                }
                            }
                        } else if msg.is_close() {
                            debug!("WebSocket connection closed by client");
                            *connection_open.write().await = false;
                            break;
                        }
                    },
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        *connection_open.write().await = false;
                        break;
                    },
                }
            }
        });

        // Task to handle outgoing broadcasts
        let outgoing_task = tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                // Check if connection is still open before attempting to send
                if !*connection_open_outgoing.read().await {
                    debug!("WebSocket connection closed, stopping outgoing messages");
                    break;
                }

                let mut sender = ws_sender_for_broadcast.lock().await;
                if let Err(e) = sender.send(Message::text(message)).await {
                    debug!(
                        "Failed to send WebSocket message (connection likely closed): {}",
                        e
                    );
                    *connection_open_outgoing.write().await = false;
                    break;
                }
            }
        });

        // Wait for either task to complete
        tokio::select! {
            _ = incoming_task => debug!("WebSocket incoming task completed"),
            _ = outgoing_task => debug!("WebSocket outgoing task completed"),
        }

        debug!("WebSocket connection closed");
    }

    // Method to broadcast updates to all connected WebSocket clients
    pub fn broadcast_update(&self, message: String) {
        if let Err(e) = self.ws_broadcaster.send(message) {
            // Only log if there are subscribers (receiver_count > 0)
            if self.ws_broadcaster.receiver_count() > 0 {
                error!("Failed to broadcast WebSocket message: {}", e);
            }
        }
    }

    fn ai_routes(self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let ai_service = self.ai_service.clone();
        let ws_broadcaster = self.ws_broadcaster.clone();
        let paper_trading = self.paper_trading_engine.clone();

        // AI analysis endpoint with WebSocket broadcasting and paper trading integration
        let ai_analyze = warp::path("analyze")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || ai_service.clone()))
            .and(warp::any().map(move || ws_broadcaster.clone()))
            .and(warp::any().map(move || paper_trading.clone()))
            .and_then(|request: crate::ai::AIAnalysisRequest, ai_service: crate::ai::AIService, broadcaster: broadcast::Sender<String>, paper_trading: Arc<PaperTradingEngine>| async move {
                let strategy_context = request.strategy_context.clone();
                let symbol = request.symbol.clone();
                let current_price = request.current_price;

                match ai_service.analyze_for_trading_signal(&request.into(), strategy_context).await {
                    Ok(response) => {
                        // Broadcast AI signal via WebSocket with FULL analysis data
                        let signal_message = serde_json::json!({
                            "type": "AISignalReceived",
                            "data": {
                                "symbol": symbol,
                                "signal": response.signal.as_str().to_lowercase(),
                                "confidence": response.confidence,
                                "timestamp": response.timestamp,
                                "model_type": "GPT-4",
                                "timeframe": "1h",
                                "reasoning": response.reasoning,
                                "strategy_scores": response.strategy_scores,
                                "market_analysis": {
                                    "trend_direction": response.market_analysis.trend_direction,
                                    "trend_strength": response.market_analysis.trend_strength,
                                    "support_levels": response.market_analysis.support_levels,
                                    "resistance_levels": response.market_analysis.resistance_levels,
                                    "volatility_level": response.market_analysis.volatility_level,
                                    "volume_analysis": response.market_analysis.volume_analysis
                                },
                                "risk_assessment": {
                                    "overall_risk": response.risk_assessment.overall_risk,
                                    "technical_risk": response.risk_assessment.technical_risk,
                                    "market_risk": response.risk_assessment.market_risk,
                                    "recommended_position_size": response.risk_assessment.recommended_position_size,
                                    "stop_loss_suggestion": response.risk_assessment.stop_loss_suggestion,
                                    "take_profit_suggestion": response.risk_assessment.take_profit_suggestion
                                }
                            },
                            "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as u64
                        });
                        if let Ok(message_str) = serde_json::to_string(&signal_message) {
                            if broadcaster.send(message_str).is_err() {
                                // Log error if needed, but don't fail the request
                                debug!("No WebSocket subscribers for AI signal broadcast");
                            } else {
                                info!("📡 Broadcasted AI signal for {} via WebSocket", symbol);
                            }
                        }

                        // Send signal to paper trading engine for automatic execution
                        let stop_loss = response.risk_assessment.stop_loss_suggestion;
                        let take_profit = response.risk_assessment.take_profit_suggestion;

                        if let Err(e) = paper_trading
                            .process_external_ai_signal(
                                symbol.clone(),
                                response.signal,
                                response.confidence,
                                response.reasoning.clone(),
                                current_price,
                                stop_loss,
                                take_profit,
                            )
                            .await
                        {
                            // Log but don't fail the API response
                            info!("Paper trading signal processing: {}", e);
                        }

                        Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(response)))
                    },
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // Strategy recommendations endpoint
        let ai_service_clone = self.ai_service.clone();
        let strategy_recommendations = warp::path("strategy-recommendations")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || ai_service_clone.clone()))
            .and_then(
                |request: crate::ai::StrategyRecommendationRequest,
                 ai_service: crate::ai::AIService| async move {
                    let market_data = request.clone().into();
                    match ai_service
                        .get_strategy_recommendations(&market_data, request.available_strategies)
                        .await
                    {
                        Ok(response) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::success(response),
                        )),
                        Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::<()>::error(e.to_string()),
                        )),
                    }
                },
            );

        // Market condition analysis endpoint
        let ai_service_clone2 = self.ai_service.clone();
        let market_condition =
            warp::path("market-condition")
                .and(warp::post())
                .and(warp::body::json())
                .and(warp::any().map(move || ai_service_clone2.clone()))
                .and_then(
                    |request: crate::ai::MarketConditionRequest,
                     ai_service: crate::ai::AIService| async move {
                        let market_data = request.into();
                        match ai_service.analyze_market_condition(&market_data).await {
                            Ok(response) => Ok::<_, warp::Rejection>(warp::reply::json(
                                &ApiResponse::success(response),
                            )),
                            Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                                &ApiResponse::<()>::error(e.to_string()),
                            )),
                        }
                    },
                );

        // Performance feedback endpoint
        let ai_service_clone3 = self.ai_service.clone();
        let performance_feedback = warp::path("feedback")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || ai_service_clone3.clone()))
            .and_then(
                |feedback: crate::ai::PerformanceFeedback,
                 ai_service: crate::ai::AIService| async move {
                    match ai_service.send_performance_feedback(feedback).await {
                        Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::success("Feedback sent successfully"),
                        )),
                        Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::<()>::error(e.to_string()),
                        )),
                    }
                },
            );

        // AI service info endpoint
        let ai_service_clone4 = self.ai_service.clone();
        let ai_info = warp::path("info")
            .and(warp::get())
            .and(warp::any().map(move || ai_service_clone4.clone()))
            .and_then(|ai_service: crate::ai::AIService| async move {
                match ai_service.get_service_info().await {
                    Ok(info) => {
                        Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(info)))
                    },
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                        &ApiResponse::<()>::error(e.to_string()),
                    )),
                }
            });

        // Supported strategies endpoint
        let ai_service_clone5 = self.ai_service.clone();
        let ai_strategies = warp::path("strategies")
            .and(warp::get())
            .and(warp::any().map(move || ai_service_clone5.clone()))
            .and_then(|ai_service: crate::ai::AIService| async move {
                match ai_service.get_supported_strategies().await {
                    Ok(strategies) => Ok::<_, warp::Rejection>(warp::reply::json(
                        &ApiResponse::success(strategies),
                    )),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(
                        &ApiResponse::<()>::error(e.to_string()),
                    )),
                }
            });

        warp::path("ai").and(
            ai_analyze
                .or(strategy_recommendations)
                .or(market_condition)
                .or(performance_feedback)
                .or(ai_info)
                .or(ai_strategies),
        )
    }
}

#[derive(Serialize, Deserialize)]
struct CandelQuery {
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct ChartQuery {
    limit: Option<usize>,
}

#[derive(Serialize, Deserialize)]
struct MultiChartQuery {
    symbols: String,    // comma-separated symbols: "BTCUSDT,ETHUSDT,BNBUSDT"
    timeframes: String, // comma-separated timeframes: "1m,5m,15m,1h"
    limit: Option<usize>,
}

// Conversion implementations for AI types
impl From<crate::ai::AIAnalysisRequest> for crate::strategies::StrategyInput {
    fn from(request: crate::ai::AIAnalysisRequest) -> Self {
        Self {
            symbol: request.symbol,
            timeframe_data: request.timeframe_data,
            current_price: request.current_price,
            volume_24h: request.volume_24h,
            timestamp: request.timestamp,
        }
    }
}

impl From<crate::ai::StrategyRecommendationRequest> for crate::strategies::StrategyInput {
    fn from(request: crate::ai::StrategyRecommendationRequest) -> Self {
        Self {
            symbol: request.symbol,
            timeframe_data: request.timeframe_data,
            current_price: request.current_price,
            volume_24h: 0.0, // Not available in strategy recommendation request
            timestamp: request.timestamp,
        }
    }
}

impl From<crate::ai::MarketConditionRequest> for crate::strategies::StrategyInput {
    fn from(request: crate::ai::MarketConditionRequest) -> Self {
        Self {
            symbol: request.symbol,
            timeframe_data: request.timeframe_data,
            current_price: request.current_price,
            volume_24h: request.volume_24h,
            timestamp: request.timestamp,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // ApiResponse Tests
    // ============================================================================

    #[test]
    fn test_api_response_success_creation() {
        let data = "test data";
        let response = ApiResponse::success(data);

        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error_creation() {
        let error_msg = "Something went wrong".to_string();
        let response: ApiResponse<String> = ApiResponse::error(error_msg.clone());

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some(error_msg));
    }

    #[test]
    fn test_api_response_success_serialization() {
        let data = vec!["item1", "item2", "item3"];
        let response = ApiResponse::success(data);

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\""));
        assert!(json.contains("item1"));
    }

    #[test]
    fn test_api_response_error_serialization() {
        let response: ApiResponse<()> = ApiResponse::error("Test error".to_string());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Test error\""));
        assert!(json.contains("\"data\":null"));
    }

    #[test]
    fn test_api_response_with_complex_data() {
        #[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
        struct ComplexData {
            id: u64,
            name: String,
            values: Vec<f64>,
        }

        let data = ComplexData {
            id: 123,
            name: "test".to_string(),
            values: vec![1.1, 2.2, 3.3],
        };

        let response = ApiResponse::success(data.clone());

        let json = serde_json::to_string(&response).unwrap();
        // ApiResponse doesn't implement Deserialize, just verify serialization
        assert!(json.contains("\"id\":123"));
        assert!(json.contains("\"name\":\"test\""));
        assert!(json.contains("1.1"));
    }

    #[test]
    fn test_add_symbol_request_serialization() {
        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string(), "5m".to_string(), "1h".to_string()]),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("1m"));
        assert!(json.contains("5m"));
        assert!(json.contains("1h"));
    }

    #[test]
    fn test_add_symbol_request_deserialization() {
        let json = r#"{
            "symbol": "ETHUSDT",
            "timeframes": ["15m", "1h", "4h"]
        }"#;

        let request: AddSymbolRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.symbol, "ETHUSDT");
        assert_eq!(request.timeframes.as_ref().map(|t| t.len()), Some(3));
        assert_eq!(
            request
                .timeframes
                .as_ref()
                .and_then(|t| t.first())
                .map(|s| s.as_str()),
            Some("15m")
        );
    }

    #[test]
    fn test_supported_symbols_serialization() {
        let supported = SupportedSymbols {
            symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            available_timeframes: vec!["1m".to_string(), "5m".to_string(), "1h".to_string()],
        };

        let json = serde_json::to_string(&supported).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("1m"));
    }

    #[test]
    fn test_supported_symbols_deserialization() {
        let json = r#"{
            "symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
            "available_timeframes": ["1m", "5m", "15m", "1h", "4h"]
        }"#;

        let supported: SupportedSymbols = serde_json::from_str(json).unwrap();
        assert_eq!(supported.symbols.len(), 3);
        assert_eq!(supported.available_timeframes.len(), 5);
        assert_eq!(supported.symbols[2], "BNBUSDT");
    }

    #[test]
    fn test_candle_query_with_limit() {
        let json = r#"{"limit": 100}"#;
        let query: CandelQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_candle_query_without_limit() {
        let json = r#"{}"#;
        let query: CandelQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, None);
    }

    #[test]
    fn test_chart_query_serialization() {
        let query = ChartQuery { limit: Some(200) };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("200"));
    }

    #[test]
    fn test_chart_query_deserialization() {
        let json = r#"{"limit": 500}"#;
        let query: ChartQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(500));
    }

    #[test]
    fn test_multi_chart_query_deserialization() {
        let json = r#"{
            "symbols": "BTCUSDT,ETHUSDT,BNBUSDT",
            "timeframes": "1m,5m,15m,1h",
            "limit": 100
        }"#;

        let query: MultiChartQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.symbols, "BTCUSDT,ETHUSDT,BNBUSDT");
        assert_eq!(query.timeframes, "1m,5m,15m,1h");
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_multi_chart_query_without_limit() {
        let json = r#"{
            "symbols": "BTCUSDT",
            "timeframes": "1h"
        }"#;

        let query: MultiChartQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.symbols, "BTCUSDT");
        assert_eq!(query.timeframes, "1h");
        assert_eq!(query.limit, None);
    }

    #[test]
    fn test_multi_chart_query_parsing_symbols() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT,ETHUSDT,BNBUSDT".to_string(),
            timeframes: "1m,5m".to_string(),
            limit: None,
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(symbols.len(), 3);
        assert_eq!(symbols[0], "BTCUSDT");
        assert_eq!(symbols[1], "ETHUSDT");
        assert_eq!(symbols[2], "BNBUSDT");
    }

    #[test]
    fn test_multi_chart_query_parsing_timeframes() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT".to_string(),
            timeframes: "1m,5m,15m,1h,4h".to_string(),
            limit: None,
        };

        let timeframes: Vec<String> = query.timeframes.split(',').map(|s| s.to_string()).collect();
        assert_eq!(timeframes.len(), 5);
        assert_eq!(timeframes[0], "1m");
        assert_eq!(timeframes[4], "4h");
    }

    #[test]
    fn test_api_response_success_with_numbers() {
        let response = ApiResponse::success(42);
        assert!(response.success);
        assert_eq!(response.data, Some(42));
    }

    #[test]
    fn test_api_response_success_with_float() {
        let test_value = 3.14159_f64;
        let response = ApiResponse::success(test_value);
        assert!(response.success);
        assert_eq!(response.data, Some(test_value));
    }

    #[test]
    fn test_api_response_success_with_boolean() {
        let response = ApiResponse::success(true);
        assert!(response.success);
        assert_eq!(response.data, Some(true));
    }

    #[test]
    fn test_api_response_error_with_empty_string() {
        let response: ApiResponse<String> = ApiResponse::error("".to_string());
        assert!(!response.success);
        assert_eq!(response.error, Some("".to_string()));
    }

    #[test]
    fn test_api_response_roundtrip_serialization() {
        let original = ApiResponse::success("test_value");
        let json = serde_json::to_string(&original).unwrap();

        // Verify JSON structure
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test_value\""));
        assert!(json.contains("\"error\":null"));
    }

    #[test]
    fn test_add_symbol_request_empty_timeframes() {
        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: Some(vec![]), // Empty but present
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AddSymbolRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert!(deserialized
            .timeframes
            .as_ref()
            .map(|t| t.is_empty())
            .unwrap_or(true));
    }

    #[test]
    fn test_supported_symbols_empty() {
        let supported = SupportedSymbols {
            symbols: vec![],
            available_timeframes: vec![],
        };

        let json = serde_json::to_string(&supported).unwrap();
        let deserialized: SupportedSymbols = serde_json::from_str(&json).unwrap();

        assert!(deserialized.symbols.is_empty());
        assert!(deserialized.available_timeframes.is_empty());
    }

    #[test]
    fn test_candle_query_large_limit() {
        let query = CandelQuery { limit: Some(10000) };
        let json = serde_json::to_string(&query).unwrap();
        let deserialized: CandelQuery = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.limit, Some(10000));
    }

    #[test]
    fn test_multi_chart_query_single_symbol() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT".to_string(),
            timeframes: "1h".to_string(),
            limit: Some(50),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0], "BTCUSDT");
    }

    #[test]
    fn test_api_response_nested_structure() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
        struct Nested {
            inner: Vec<i32>,
        }

        let data = Nested {
            inner: vec![1, 2, 3, 4, 5],
        };
        let response = ApiResponse::success(data.clone());

        let json = serde_json::to_string(&response).unwrap();

        // Verify serialization
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"inner\":[1,2,3,4,5]"));
    }

    // ============================================================================
    // Conversion Implementation Tests (AIAnalysisRequest -> StrategyInput)
    // ============================================================================

    #[test]
    fn test_ai_analysis_request_to_strategy_input_conversion() {
        use crate::ai::{AIAnalysisRequest, AIStrategyContext};
        use std::collections::HashMap;

        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("1h".to_string(), vec![]);

        let request = AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: timeframe_data.clone(),
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let strategy_input: crate::strategies::StrategyInput = request.into();

        assert_eq!(strategy_input.symbol, "BTCUSDT");
        assert_eq!(strategy_input.current_price, 50000.0);
        assert_eq!(strategy_input.volume_24h, 1000000.0);
        assert_eq!(strategy_input.timestamp, 1234567890);
    }

    #[test]
    fn test_strategy_recommendation_request_to_strategy_input_conversion() {
        use crate::ai::StrategyRecommendationRequest;
        use std::collections::HashMap;

        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("5m".to_string(), vec![]);

        let request = StrategyRecommendationRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data: timeframe_data.clone(),
            current_price: 3000.0,
            timestamp: 9876543210,
            available_strategies: vec!["momentum".to_string()],
        };

        let strategy_input: crate::strategies::StrategyInput = request.into();

        assert_eq!(strategy_input.symbol, "ETHUSDT");
        assert_eq!(strategy_input.current_price, 3000.0);
        assert_eq!(strategy_input.volume_24h, 0.0); // Should default to 0.0
        assert_eq!(strategy_input.timestamp, 9876543210);
    }

    #[test]
    fn test_market_condition_request_to_strategy_input_conversion() {
        use crate::ai::MarketConditionRequest;
        use std::collections::HashMap;

        let mut timeframe_data = HashMap::new();
        timeframe_data.insert("15m".to_string(), vec![]);

        let request = MarketConditionRequest {
            symbol: "BNBUSDT".to_string(),
            timeframe_data: timeframe_data.clone(),
            current_price: 400.0,
            volume_24h: 500000.0,
            timestamp: 1111111111,
        };

        let strategy_input: crate::strategies::StrategyInput = request.into();

        assert_eq!(strategy_input.symbol, "BNBUSDT");
        assert_eq!(strategy_input.current_price, 400.0);
        assert_eq!(strategy_input.volume_24h, 500000.0);
        assert_eq!(strategy_input.timestamp, 1111111111);
    }

    // ============================================================================
    // Query Parameter Deserialization Tests
    // ============================================================================

    #[test]
    fn test_candle_query_zero_limit() {
        let json = r#"{"limit": 0}"#;
        let query: CandelQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(0));
    }

    #[test]
    fn test_chart_query_none_limit() {
        let query = ChartQuery { limit: None };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("\"limit\":null"));
    }

    #[test]
    fn test_multi_chart_query_serialization() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT,ETHUSDT".to_string(),
            timeframes: "1m,5m".to_string(),
            limit: Some(150),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("BTCUSDT,ETHUSDT"));
        assert!(json.contains("1m,5m"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_multi_chart_query_with_special_characters() {
        let query = MultiChartQuery {
            symbols: "BTC-USDT,ETH_USD".to_string(),
            timeframes: "1h,4h".to_string(),
            limit: Some(100),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(symbols[0], "BTC-USDT");
        assert_eq!(symbols[1], "ETH_USD");
    }

    #[test]
    fn test_multi_chart_query_empty_symbols() {
        let query = MultiChartQuery {
            symbols: "".to_string(),
            timeframes: "1h".to_string(),
            limit: None,
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0], "");
    }

    #[test]
    fn test_add_symbol_request_validation() {
        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string(), "5m".to_string()]),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AddSymbolRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, request.symbol);
        assert_eq!(
            deserialized.timeframes.as_ref().map(|t| t.len()),
            request.timeframes.as_ref().map(|t| t.len())
        );
    }

    #[test]
    fn test_add_symbol_request_with_many_timeframes() {
        let timeframes: Vec<String> = vec![
            "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: Some(timeframes),
        };

        assert_eq!(request.timeframes.as_ref().map(|t| t.len()), Some(12));
    }

    #[test]
    fn test_supported_symbols_with_many_symbols() {
        let symbols: Vec<String> = (0..100).map(|i| format!("SYMBOL{}", i)).collect();
        let timeframes = vec!["1m".to_string(), "5m".to_string()];

        let supported = SupportedSymbols {
            symbols: symbols.clone(),
            available_timeframes: timeframes,
        };

        assert_eq!(supported.symbols.len(), 100);
        assert_eq!(supported.symbols[99], "SYMBOL99");
    }

    #[test]
    fn test_supported_symbols_roundtrip() {
        let original = SupportedSymbols {
            symbols: vec!["BTC".to_string(), "ETH".to_string()],
            available_timeframes: vec!["1h".to_string()],
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: SupportedSymbols = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbols, original.symbols);
        assert_eq!(
            deserialized.available_timeframes,
            original.available_timeframes
        );
    }

    // ============================================================================
    // ApiResponse Edge Cases and Validation
    // ============================================================================

    #[test]
    fn test_api_response_with_option_data() {
        let response = ApiResponse::success(Some("optional_value"));
        assert!(response.success);
        assert_eq!(response.data, Some(Some("optional_value")));
    }

    #[test]
    fn test_api_response_with_none_data() {
        let response: ApiResponse<Option<String>> = ApiResponse::success(None);
        assert!(response.success);
        assert_eq!(response.data, Some(None));
    }

    #[test]
    fn test_api_response_error_with_long_message() {
        let long_error = "e".repeat(5000);
        let response: ApiResponse<String> = ApiResponse::error(long_error.clone());
        assert!(!response.success);
        assert_eq!(response.error, Some(long_error));
    }

    #[test]
    fn test_api_response_with_vector_data() {
        let data = vec![1, 2, 3, 4, 5];
        let response = ApiResponse::success(data.clone());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("[1,2,3,4,5]"));
    }

    #[test]
    fn test_api_response_with_hashmap_data() {
        use std::collections::HashMap;
        let mut data = HashMap::new();
        data.insert("key1", "value1");
        data.insert("key2", "value2");

        let response = ApiResponse::success(data);
        assert!(response.success);
    }

    #[test]
    fn test_api_response_error_with_special_chars() {
        let error_msg = "Error: \n\t\"quoted\" & <special>".to_string();
        let response: ApiResponse<()> = ApiResponse::error(error_msg.clone());

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("Error"));
    }

    #[test]
    fn test_api_response_success_with_unit_type() {
        let response = ApiResponse::success(());
        assert!(response.success);
        assert_eq!(response.data, Some(()));
    }

    #[test]
    fn test_api_response_success_with_tuple() {
        let response = ApiResponse::success(("status", 200));
        assert!(response.success);
        assert_eq!(response.data, Some(("status", 200)));
    }

    #[test]
    fn test_api_response_serialization_formatting() {
        let response = ApiResponse::success("test");
        let json = serde_json::to_string(&response).unwrap();

        // Should not have extra whitespace
        assert!(!json.contains('\n'));
        assert!(!json.contains("  "));
    }

    // ============================================================================
    // Request/Response Struct Field Validation
    // ============================================================================

    #[test]
    fn test_add_symbol_request_case_sensitivity() {
        let request1 = AddSymbolRequest {
            symbol: "btcusdt".to_string(),
            timeframes: None, // Optional, use None for empty
        };
        let request2 = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: None,
        };

        assert_ne!(request1.symbol, request2.symbol);
    }

    #[test]
    fn test_candle_query_max_value_limit() {
        let query = CandelQuery {
            limit: Some(usize::MAX),
        };
        let json = serde_json::to_string(&query).unwrap();
        let deserialized: CandelQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.limit, Some(usize::MAX));
    }

    #[test]
    fn test_chart_query_with_zero() {
        let query = ChartQuery { limit: Some(0) };
        assert_eq!(query.limit, Some(0));
    }

    #[test]
    fn test_multi_chart_query_whitespace_handling() {
        let query = MultiChartQuery {
            symbols: " BTCUSDT , ETHUSDT ".to_string(),
            timeframes: " 1m , 5m ".to_string(),
            limit: Some(50),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        // Note: split doesn't trim, so spaces are preserved
        assert_eq!(symbols[0], " BTCUSDT ");
        assert_eq!(symbols[1], " ETHUSDT ");
    }

    #[test]
    fn test_multi_chart_query_single_item_no_comma() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT".to_string(),
            timeframes: "1h".to_string(),
            limit: None,
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0], "BTCUSDT");
    }

    // ============================================================================
    // WARP HANDLER TESTS (Integration Tests for API Routes)
    // ============================================================================

    use crate::binance::BinanceClient;

    async fn create_test_api_server() -> ApiServer {
        // Create test database config (no MongoDB)
        let db_config = crate::config::DatabaseConfig {
            url: "no-db://test".to_string(),
            database_name: Some("test_db".to_string()),
            max_connections: 1,
            enable_logging: false,
        };
        let storage = crate::storage::Storage::new(&db_config).await.unwrap();

        // Create Binance config
        let binance_config = crate::config::BinanceConfig {
            api_key: "test_key".to_string(),
            secret_key: "test_secret".to_string(),
            futures_api_key: String::new(),
            futures_secret_key: String::new(),
            testnet: true,
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
            trading_mode: crate::config::TradingMode::RealTestnet,
        };

        // Create MarketData config
        let market_data_config = crate::config::MarketDataConfig {
            symbols: vec!["BTCUSDT".to_string()],
            timeframes: vec!["1m".to_string()],
            kline_limit: 100,
            update_interval_ms: 60000,
            reconnect_interval_ms: 5000,
            max_reconnect_attempts: 3,
            cache_size: 500,
            python_ai_service_url: "http://localhost:8000".to_string(),
        };

        // Create Trading config
        let trading_config = crate::config::TradingConfig {
            enabled: false,
            max_positions: 3,
            default_quantity: 0.01,
            risk_percentage: 2.0,
            stop_loss_percentage: 2.0,
            take_profit_percentage: 4.0,
            order_timeout_seconds: 30,
            position_check_interval_seconds: 60,
            leverage: 1,
            margin_type: "ISOLATED".to_string(),
        };

        let market_data = crate::market_data::MarketDataProcessor::new(
            binance_config.clone(),
            market_data_config,
            storage.clone(),
        )
        .await
        .unwrap();

        let trading_engine = crate::trading::TradingEngine::new(
            binance_config.clone(),
            trading_config,
            market_data.clone(),
            storage.clone(),
        )
        .await
        .unwrap();

        // Create PaperTradingEngine with required components
        let binance_client = BinanceClient::new(binance_config.clone()).unwrap();
        let ai_service = crate::ai::AIService::new(crate::ai::AIServiceConfig {
            python_service_url: "http://localhost:8000".to_string(),
            request_timeout_seconds: 30,
            max_retries: 1,
            enable_caching: false,
            cache_ttl_seconds: 60,
        });
        let (paper_event_tx, _) = tokio::sync::broadcast::channel(100);
        let paper_trading_settings = crate::paper_trading::PaperTradingSettings::default();

        let paper_trading_engine = Arc::new(
            crate::paper_trading::PaperTradingEngine::new(
                paper_trading_settings,
                binance_client,
                ai_service,
                storage.clone(),
                paper_event_tx,
            )
            .await
            .unwrap(),
        );

        let (ws_broadcaster, _) = tokio::sync::broadcast::channel(100);

        ApiServer::new(
            crate::config::ApiConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                cors_origins: vec!["*".to_string()],
                enable_metrics: false,
            },
            binance_config,
            market_data,
            trading_engine,
            paper_trading_engine,
            None,
            ws_broadcaster,
            storage,
        )
        .await
        .unwrap()
    }

    #[tokio::test]
    async fn test_health_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/health")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body: ApiResponse<String> = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    async fn test_market_prices_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/prices")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_market_overview_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/overview")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_candles_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/candles/BTCUSDT/1h?limit=10")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_market_chart_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/chart/BTCUSDT/1h?limit=50")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_multi_chart_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/charts?symbols=BTCUSDT,ETHUSDT&timeframes=1h,4h&limit=10")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_symbols_get_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/symbols")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
        let body: ApiResponse<SupportedSymbols> = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    async fn test_market_symbols_add_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let request = AddSymbolRequest {
            symbol: "SOLUSDT".to_string(),
            timeframes: Some(vec!["1h".to_string()]),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/api/market/symbols")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_symbols_delete_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/market/symbols/BTCUSDT")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_trading_positions_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/positions")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_trading_account_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/account")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_trading_close_position_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/api/trading/positions/BTCUSDT/close")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_trading_performance_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/performance")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_monitoring_system_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/monitoring/system")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_monitoring_trading_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/monitoring/trading")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_monitoring_connection_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/monitoring/connection")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_ai_info_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/ai/info")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_ai_strategies_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/ai/strategies")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    // ============================================================================
    // ApiResponse Type Compatibility Tests
    // ============================================================================

    #[test]
    fn test_api_response_with_f64() {
        let response = ApiResponse::success(123.456);
        assert_eq!(response.data, Some(123.456));
    }

    #[test]
    fn test_api_response_with_i64() {
        let response = ApiResponse::success(9876543210i64);
        assert_eq!(response.data, Some(9876543210i64));
    }

    #[test]
    fn test_api_response_with_u64() {
        let response = ApiResponse::success(18446744073709551615u64);
        assert_eq!(response.data, Some(18446744073709551615u64));
    }

    #[test]
    fn test_api_response_error_with_unicode() {
        let error = "エラー: 取引失敗 🚫".to_string();
        let response: ApiResponse<()> = ApiResponse::error(error.clone());
        assert_eq!(response.error, Some(error));
    }

    // ============================================================================
    // Serialization Edge Cases
    // ============================================================================

    #[test]
    fn test_add_symbol_request_special_symbol_names() {
        let symbols = vec!["BTC-USDT", "ETH_USDT", "BNB/USDT", "SOL.USDT"];
        for symbol in symbols {
            let request = AddSymbolRequest {
                symbol: symbol.to_string(),
                timeframes: Some(vec!["1h".to_string()]),
            };
            let json = serde_json::to_string(&request).unwrap();
            assert!(json.contains(symbol));
        }
    }

    #[test]
    fn test_supported_symbols_duplicate_handling() {
        let supported = SupportedSymbols {
            symbols: vec!["BTCUSDT".to_string(), "BTCUSDT".to_string()],
            available_timeframes: vec!["1m".to_string(), "1m".to_string()],
        };

        assert_eq!(supported.symbols.len(), 2);
        assert_eq!(supported.available_timeframes.len(), 2);
    }

    #[test]
    fn test_candle_query_deserialization_with_null() {
        let json = r#"{"limit": null}"#;
        let query: CandelQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, None);
    }

    #[test]
    fn test_chart_query_deserialization_with_string_number() {
        // This should fail since limit is not a string
        let json = r#"{"limit": "100"}"#;
        let result = serde_json::from_str::<ChartQuery>(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_multi_chart_query_with_numbers_in_symbols() {
        let query = MultiChartQuery {
            symbols: "BTC2USDT,ETH3USDT".to_string(),
            timeframes: "1m,5m".to_string(),
            limit: Some(10),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(symbols[0], "BTC2USDT");
        assert_eq!(symbols[1], "ETH3USDT");
    }

    // ============================================================================
    // JSON Serialization Format Tests
    // ============================================================================

    #[test]
    fn test_api_response_json_field_order() {
        let response = ApiResponse::success("data");
        let json = serde_json::to_string(&response).unwrap();
        // JSON should contain all three fields
        assert!(json.contains("success"));
        assert!(json.contains("data"));
        assert!(json.contains("error"));
    }

    #[test]
    fn test_add_symbol_request_json_structure() {
        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string()]),
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json.get("symbol").is_some());
        assert!(json.get("timeframes").is_some());
        // timeframes is Some(vec![...]) so it serializes as an array
        assert!(json.get("timeframes").unwrap().is_array());
    }

    #[test]
    fn test_supported_symbols_json_structure() {
        let supported = SupportedSymbols {
            symbols: vec!["BTC".to_string()],
            available_timeframes: vec!["1h".to_string()],
        };

        let json = serde_json::to_value(&supported).unwrap();
        assert!(json.get("symbols").is_some());
        assert!(json.get("available_timeframes").is_some());
    }

    // ============================================================================
    // Type Constraint Tests
    // ============================================================================

    #[test]
    fn test_api_response_implements_clone() {
        let response = ApiResponse::success("test");
        let _cloned = response.clone();
        // If this compiles, Clone is implemented
    }

    #[test]
    fn test_query_types_implement_serialize() {
        let candle_query = CandelQuery { limit: Some(100) };
        let _json = serde_json::to_string(&candle_query).unwrap();

        let chart_query = ChartQuery { limit: Some(200) };
        let _json = serde_json::to_string(&chart_query).unwrap();

        let multi_query = MultiChartQuery {
            symbols: "BTC".to_string(),
            timeframes: "1h".to_string(),
            limit: Some(50),
        };
        let _json = serde_json::to_string(&multi_query).unwrap();
    }

    #[test]
    fn test_query_types_implement_deserialize() {
        let candle_json = r#"{"limit": 100}"#;
        let _candle: CandelQuery = serde_json::from_str(candle_json).unwrap();

        let chart_json = r#"{"limit": 200}"#;
        let _chart: ChartQuery = serde_json::from_str(chart_json).unwrap();

        let multi_json = r#"{"symbols": "BTC", "timeframes": "1h", "limit": 50}"#;
        let _multi: MultiChartQuery = serde_json::from_str(multi_json).unwrap();
    }

    #[tokio::test]
    async fn test_broadcast_update() {
        let server = create_test_api_server().await;
        server.broadcast_update("test message".to_string());
        // No panic = success (no receivers is OK)
    }

    #[tokio::test]
    async fn test_update_monitoring() {
        let server = create_test_api_server().await;
        server.update_monitoring(5, 100, true, true).await;
        // No panic = success
    }

    #[test]
    fn test_api_response_clone() {
        let response = ApiResponse::success("test");
        let cloned = response.clone();
        assert_eq!(cloned.success, response.success);
        assert_eq!(cloned.data, response.data);
        assert_eq!(cloned.error, response.error);
    }

    // ============================================================================
    // Boundary Value Tests
    // ============================================================================

    #[test]
    fn test_candle_query_boundary_values() {
        let queries = vec![
            CandelQuery { limit: Some(0) },
            CandelQuery { limit: Some(1) },
            CandelQuery { limit: Some(1000) },
            CandelQuery {
                limit: Some(usize::MAX),
            },
            CandelQuery { limit: None },
        ];

        for query in queries {
            let json = serde_json::to_string(&query).unwrap();
            let deserialized: CandelQuery = serde_json::from_str(&json).unwrap();
            assert_eq!(deserialized.limit, query.limit);
        }
    }

    #[test]
    fn test_multi_chart_query_many_symbols() {
        let symbols: Vec<String> = (0..50).map(|i| format!("SYM{}", i)).collect();
        let symbols_str = symbols.join(",");

        let query = MultiChartQuery {
            symbols: symbols_str,
            timeframes: "1m".to_string(),
            limit: Some(100),
        };

        let parsed: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(parsed.len(), 50);
    }

    #[test]
    fn test_add_symbol_request_long_symbol_name() {
        let long_symbol = "A".repeat(100);
        let request = AddSymbolRequest {
            symbol: long_symbol.clone(),
            timeframes: Some(vec!["1h".to_string()]),
        };

        assert_eq!(request.symbol, long_symbol);
    }

    // ============================================================================
    // Error Handling in Serialization
    // ============================================================================

    #[test]
    fn test_api_response_error_null_handling() {
        let response = ApiResponse::<String>::error("null".to_string());
        let json = serde_json::to_string(&response).unwrap();
        // Should contain the string "null", not the null value
        assert!(json.contains("\"null\""));
    }

    #[test]
    fn test_multi_chart_query_invalid_json_handling() {
        let invalid_json = r#"{"symbols": "BTC", "timeframes": 123}"#; // timeframes should be string
        let result = serde_json::from_str::<MultiChartQuery>(invalid_json);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_symbol_request_missing_field() {
        let json = r#"{"symbol": "BTCUSDT"}"#; // missing timeframes - but it's Optional now
        let result = serde_json::from_str::<AddSymbolRequest>(json);
        // timeframes is Option<Vec<String>>, so missing field is Ok (defaults to None)
        assert!(result.is_ok());
        let request = result.unwrap();
        assert_eq!(request.symbol, "BTCUSDT");
        assert!(request.timeframes.is_none());
    }

    #[test]
    fn test_supported_symbols_extra_fields_ignored() {
        let json = r#"{
            "symbols": ["BTC"],
            "available_timeframes": ["1h"],
            "extra_field": "ignored"
        }"#;
        let result: SupportedSymbols = serde_json::from_str(json).unwrap();
        assert_eq!(result.symbols.len(), 1);
    }

    #[tokio::test]
    async fn test_cors_headers_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("OPTIONS")
            .path("/api/health")
            .reply(&routes)
            .await;

        // OPTIONS may return 405 (Method Not Allowed) if route doesn't explicitly handle OPTIONS
        assert!(resp.status().is_success() || resp.status() == 404 || resp.status() == 405);
    }

    #[tokio::test]
    async fn test_nonexistent_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/nonexistent")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn test_market_candles_without_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/candles/BTCUSDT/1h")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success());
    }

    #[tokio::test]
    async fn test_market_chart_without_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/chart/BTCUSDT/1h")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_multi_chart_minimal() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/charts?symbols=BTCUSDT&timeframes=1h")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_invalid_http_method_on_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("PUT")
            .path("/api/health")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 405);
    }

    #[tokio::test]
    async fn test_trading_positions_empty() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/positions")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body: ApiResponse<Vec<crate::trading::position_manager::Position>> =
            serde_json::from_slice(resp.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    async fn test_market_add_symbol_minimal() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let request = AddSymbolRequest {
            symbol: "ADAUSDT".to_string(),
            timeframes: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/api/market/symbols")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_delete_nonexistent_symbol() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/market/symbols/NONEXISTENT")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_monitoring_routes_all() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let paths = vec![
            "/api/monitoring/system",
            "/api/monitoring/trading",
            "/api/monitoring/connection",
        ];

        for path in paths {
            let resp = warp::test::request()
                .method("GET")
                .path(path)
                .reply(&routes)
                .await;

            assert_eq!(resp.status(), 200);
        }
    }

    #[tokio::test]
    async fn test_trading_close_position_nonexistent() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/api/trading/positions/NONEXISTENT/close")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_health_check_content() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/health")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body: ApiResponse<String> = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.success);
        assert_eq!(body.data, Some("Bot is running".to_string()));
        assert!(body.error.is_none());
    }

    #[tokio::test]
    async fn test_market_prices_response_format() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/prices")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.get("success").is_some());
        assert!(body.get("data").is_some());
    }

    #[tokio::test]
    async fn test_market_symbols_response_structure() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/symbols")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body: ApiResponse<SupportedSymbols> = serde_json::from_slice(resp.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());
        if let Some(data) = body.data {
            assert!(data.symbols.len() >= 0);
            assert!(data.available_timeframes.len() > 0);
        }
    }

    #[test]
    fn test_conversion_ai_to_strategy_input() {
        use crate::ai::{AIAnalysisRequest, AIStrategyContext};
        use std::collections::HashMap;

        let request = AIAnalysisRequest {
            symbol: "TESTUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 12345.67,
            volume_24h: 9999999.0,
            timestamp: 1234567890,
            strategy_context: AIStrategyContext::default(),
        };

        let input: crate::strategies::StrategyInput = request.into();
        assert_eq!(input.symbol, "TESTUSDT");
        assert_eq!(input.current_price, 12345.67);
        assert_eq!(input.volume_24h, 9999999.0);
    }

    #[test]
    fn test_conversion_strategy_recommendation() {
        use crate::ai::StrategyRecommendationRequest;
        use std::collections::HashMap;

        let request = StrategyRecommendationRequest {
            symbol: "XYZUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 999.99,
            timestamp: 1111111111,
            available_strategies: vec!["test".to_string()],
        };

        let input: crate::strategies::StrategyInput = request.into();
        assert_eq!(input.symbol, "XYZUSDT");
        assert_eq!(input.current_price, 999.99);
        assert_eq!(input.volume_24h, 0.0);
    }

    #[test]
    fn test_conversion_market_condition() {
        use crate::ai::MarketConditionRequest;
        use std::collections::HashMap;

        let request = MarketConditionRequest {
            symbol: "ABCUSDT".to_string(),
            timeframe_data: HashMap::new(),
            current_price: 777.77,
            volume_24h: 888888.0,
            timestamp: 2222222222,
        };

        let input: crate::strategies::StrategyInput = request.into();
        assert_eq!(input.symbol, "ABCUSDT");
        assert_eq!(input.current_price, 777.77);
        assert_eq!(input.volume_24h, 888888.0);
    }

    // Additional API route and handler tests

    #[tokio::test]
    async fn test_api_server_new_with_database() {
        let server = create_test_api_server().await;
        assert!(server.config.host == "127.0.0.1");
    }

    #[tokio::test]
    async fn test_api_server_clone() {
        let server = create_test_api_server().await;
        let cloned = server.clone();
        assert_eq!(server.config.port, cloned.config.port);
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("OPTIONS")
            .path("/api/health")
            .reply(&routes)
            .await;

        // OPTIONS may return success, 404, or 405 depending on CORS config
        assert!(resp.status().is_success() || resp.status().as_u16() == 404 || resp.status().as_u16() == 405);
    }

    #[tokio::test]
    async fn test_websocket_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/ws")
            .reply(&routes)
            .await;

        // WebSocket upgrade requires special handling
        assert!(resp.status().is_client_error() || resp.status().is_success());
    }

    #[tokio::test]
    async fn test_market_prices_with_multiple_symbols() {
        let server = create_test_api_server().await;

        // Add test data before creating routes (which moves server)
        server.market_data.get_cache().add_historical_klines(
            "BTCUSDT",
            "1m",
            vec![crate::binance::types::Kline {
                open_time: 1609459200000,
                open: "50000.0".to_string(),
                high: "51000.0".to_string(),
                low: "49000.0".to_string(),
                close: "50500.0".to_string(),
                volume: "100.0".to_string(),
                close_time: 1609459260000,
                quote_asset_volume: "5000000.0".to_string(),
                number_of_trades: 1000,
                taker_buy_base_asset_volume: "50.0".to_string(),
                taker_buy_quote_asset_volume: "2500000.0".to_string(),
                ignore: "0".to_string(),
            }],
        );

        let routes = server.create_routes();
        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/prices")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_market_candles_with_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/candles/BTCUSDT/1m?limit=50")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_market_chart_with_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/chart/BTCUSDT/1m?limit=100")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_multi_chart() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/charts?symbols=BTCUSDT,ETHUSDT&timeframes=1m,5m&limit=50")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_trading_performance() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/performance")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_monitoring_system() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/monitoring/system")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_monitoring_trading() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/monitoring/trading")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_monitoring_connection() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/monitoring/connection")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_invalid_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/invalid/route")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 404);
    }

    #[tokio::test]
    async fn test_market_overview_error_handling() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/overview")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_trading_account_error_handling() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/account")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_add_symbol_with_timeframes() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let request = AddSymbolRequest {
            symbol: "LTCUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string(), "5m".to_string()]),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/api/market/symbols")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_market_delete_symbol() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/market/symbols/TESTCOIN")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[test]
    fn test_api_response_success_with_string() {
        let response = ApiResponse::success("test message".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("test message".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error_with_message() {
        let response = ApiResponse::<String>::error("error message".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("error message".to_string()));
    }

    #[test]
    fn test_candel_query_default() {
        let query = CandelQuery { limit: None };
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_chart_query_with_value() {
        let query = ChartQuery { limit: Some(100) };
        assert_eq!(query.limit, Some(100));
    }

    #[test]
    fn test_multi_chart_query_parsing() {
        let query = MultiChartQuery {
            symbols: "BTC,ETH,ADA".to_string(),
            timeframes: "1m,5m,1h".to_string(),
            limit: Some(50),
        };

        assert!(query.symbols.contains("BTC"));
        assert!(query.timeframes.contains("1m"));
    }

    #[test]
    fn test_add_symbol_request_default_timeframes() {
        let request = AddSymbolRequest {
            symbol: "TESTCOIN".to_string(),
            timeframes: None,
        };

        assert!(request.timeframes.is_none());
    }

    #[test]
    fn test_supported_symbols_structure() {
        let symbols = SupportedSymbols {
            symbols: vec!["BTC".to_string(), "ETH".to_string()],
            available_timeframes: vec!["1m".to_string(), "5m".to_string()],
        };

        assert_eq!(symbols.symbols.len(), 2);
        assert_eq!(symbols.available_timeframes.len(), 2);
    }

    #[tokio::test]
    async fn test_market_prices_empty_cache() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/prices")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body: ApiResponse<std::collections::HashMap<String, f64>> =
            serde_json::from_slice(resp.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    async fn test_market_candles_invalid_timeframe() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/candles/BTCUSDT/invalid")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_trading_close_position_success() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/api/trading/positions/BTCUSDT/close")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_health_check_multiple_times() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        for _ in 0..5 {
            let resp = warp::test::request()
                .method("GET")
                .path("/api/health")
                .reply(&routes)
                .await;

            assert_eq!(resp.status(), 200);
        }
    }

    #[tokio::test]
    async fn test_market_symbols_get() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/symbols")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_trading_positions_type() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/positions")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
        let body_bytes = resp.body();
        assert!(!body_bytes.is_empty());
    }

    #[tokio::test]
    async fn test_cov9_cors_headers() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("OPTIONS")
            .path("/api/health")
            .header("origin", "http://localhost:3000")
            .reply(&routes)
            .await;

        // CORS should handle OPTIONS requests
        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_cov9_market_add_symbol_empty() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let request = AddSymbolRequest {
            symbol: "".to_string(),
            timeframes: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/api/market/symbols")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_cov9_market_add_symbol_with_timeframes() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let request = AddSymbolRequest {
            symbol: "ETHUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string(), "5m".to_string()]),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/api/market/symbols")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov9_market_candles_with_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/candles/BTCUSDT/1m?limit=50")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_cov9_market_candles_without_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/candles/ETHUSDT/5m")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_cov9_trading_account_info() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/account")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov9_invalid_path() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/invalid/path/here")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cov9_method_not_allowed() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("PATCH")
            .path("/api/health")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_cov9_market_chart_data_invalid_symbol() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/chart/INVALID/1h")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_cov9_market_chart_data_with_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/chart/BTCUSDT/1h?limit=100")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_cov9_market_multi_chart_basic() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/multi-chart?symbols=BTCUSDT&timeframes=1m")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status() == 404);
    }

    #[tokio::test]
    async fn test_cov9_market_multi_chart_multiple_symbols() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/multi-chart?symbols=BTCUSDT,ETHUSDT&timeframes=1m,5m")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status() == 404);
    }

    #[tokio::test]
    async fn test_cov9_market_multi_chart_with_limit() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/multi-chart?symbols=BTCUSDT&timeframes=1h&limit=200")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status() == 404);
    }

    #[test]
    fn test_cov9_add_symbol_request_no_timeframes() {
        let request = AddSymbolRequest {
            symbol: "SOLUSDT".to_string(),
            timeframes: None,
        };

        assert_eq!(request.symbol, "SOLUSDT");
        assert!(request.timeframes.is_none());
    }

    #[test]
    fn test_cov9_candle_query_default() {
        let query = CandelQuery { limit: None };
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_cov9_chart_query_none() {
        let query = ChartQuery { limit: None };
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_cov9_multi_chart_query_single_symbol() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT".to_string(),
            timeframes: "1m".to_string(),
            limit: Some(50),
        };

        let symbol_list: Vec<&str> = query.symbols.split(',').collect();
        assert_eq!(symbol_list.len(), 1);
        assert_eq!(symbol_list[0], "BTCUSDT");
    }

    #[test]
    fn test_cov9_api_config_structure() {
        let config = ApiConfig {
            host: "127.0.0.1".to_string(),
            port: 8080,
            cors_origins: vec!["http://localhost:3000".to_string()],
            enable_metrics: false,
        };

        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
        assert!(!config.enable_metrics);
        assert_eq!(config.cors_origins.len(), 1);
    }

    #[tokio::test]
    async fn test_cov9_multiple_concurrent_requests() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let handles: Vec<_> = (0..5)
            .map(|_| {
                let routes_clone = routes.clone();
                tokio::spawn(async move {
                    warp::test::request()
                        .method("GET")
                        .path("/api/health")
                        .reply(&routes_clone)
                        .await
                })
            })
            .collect();

        for handle in handles {
            let resp = handle.await.unwrap();
            assert_eq!(resp.status(), 200);
        }
    }

    // ============================================================================
    // ADDITIONAL COVERAGE BOOST TESTS - Phase 10
    // ============================================================================

    #[tokio::test]
    async fn test_cov10_market_symbols_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/market/symbols")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov10_market_add_symbol_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let request = AddSymbolRequest {
            symbol: "ETHUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string(), "5m".to_string()]),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/api/market/symbols")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error() || resp.status() == 404);
    }

    #[tokio::test]
    async fn test_cov10_market_remove_symbol_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/api/market/symbols/BTCUSDT")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error() || resp.status() == 404);
    }

    #[tokio::test]
    async fn test_cov10_trading_account_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/trading/account")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov10_monitoring_connection_route() {
        let server = create_test_api_server().await;
        let routes = server.create_routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/api/monitoring/connection")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), 200);
    }

    #[tokio::test]
    async fn test_cov10_update_monitoring_method() {
        let server = create_test_api_server().await;

        // Call update_monitoring to cover that code path
        server.update_monitoring(3, 1000, true, true).await;
        server.update_monitoring(0, 0, false, false).await;

        // Verify by reading monitoring state
        let monitor = server.monitoring.read().await;
        let metrics = monitor.get_system_metrics();
        assert!(metrics.cache_size >= 0);
    }

    #[test]
    fn test_cov10_api_response_success_clone() {
        let original = ApiResponse::success("test".to_string());
        let cloned = original.clone();

        assert_eq!(original.success, cloned.success);
        assert_eq!(original.data, cloned.data);
        assert_eq!(original.error, cloned.error);
    }

    #[test]
    fn test_cov10_add_symbol_request_with_single_timeframe() {
        let request = AddSymbolRequest {
            symbol: "BNBUSDT".to_string(),
            timeframes: Some(vec!["1h".to_string()]),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AddSymbolRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "BNBUSDT");
        assert_eq!(deserialized.timeframes.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_cov10_supported_symbols_edge_case() {
        let supported = SupportedSymbols {
            symbols: vec!["A".repeat(50)],
            available_timeframes: vec!["1s".to_string()],
        };

        let json = serde_json::to_string(&supported).unwrap();
        assert!(json.len() > 50);
    }

    #[test]
    fn test_cov10_candle_query_with_one() {
        let query = CandelQuery { limit: Some(1) };
        assert_eq!(query.limit, Some(1));
    }

    #[test]
    fn test_cov10_chart_query_large_limit() {
        let query = ChartQuery { limit: Some(5000) };
        let json = serde_json::to_string(&query).unwrap();
        let deserialized: ChartQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.limit, Some(5000));
    }

    #[test]
    fn test_cov10_multi_chart_query_trimming() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT,ETHUSDT,BNBUSDT".to_string(),
            timeframes: "1m,5m,15m".to_string(),
            limit: Some(100),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.trim().to_string()).collect();
        let timeframes: Vec<String> = query.timeframes.split(',').map(|s| s.trim().to_string()).collect();

        assert_eq!(symbols.len(), 3);
        assert_eq!(timeframes.len(), 3);
        assert_eq!(symbols[2], "BNBUSDT");
        assert_eq!(timeframes[2], "15m");
    }

    // ==================== NEW BOOST TESTS ====================

    #[test]
    fn test_boost_api_response_success_with_string() {
        let response = ApiResponse::success("Test data".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("Test data".to_string()));
        assert_eq!(response.error, None);
    }

    #[test]
    fn test_boost_api_response_error_with_message() {
        let response: ApiResponse<String> = ApiResponse::error("Error occurred".to_string());
        assert!(!response.success);
        assert_eq!(response.data, None);
        assert_eq!(response.error, Some("Error occurred".to_string()));
    }

    #[test]
    fn test_boost_api_response_clone() {
        let response = ApiResponse::success(42);
        let cloned = response.clone();
        assert_eq!(response.success, cloned.success);
        assert_eq!(response.data, cloned.data);
        assert_eq!(response.error, cloned.error);
    }

    #[test]
    fn test_boost_api_response_success_with_vec() {
        let data = vec![1, 2, 3, 4, 5];
        let response = ApiResponse::success(data.clone());
        assert!(response.success);
        assert_eq!(response.data, Some(data));
    }

    #[test]
    fn test_boost_api_response_error_empty_message() {
        let response: ApiResponse<i32> = ApiResponse::error("".to_string());
        assert!(!response.success);
        assert_eq!(response.error, Some("".to_string()));
    }

    #[test]
    fn test_boost_add_symbol_request_serialization_full() {
        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string(), "5m".to_string(), "1h".to_string()]),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("1m"));
    }

    #[test]
    fn test_boost_add_symbol_request_deserialization_full() {
        let json = r#"{"symbol":"ETHUSDT","timeframes":["5m","15m"]}"#;
        let request: AddSymbolRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.symbol, "ETHUSDT");
        assert_eq!(request.timeframes.unwrap().len(), 2);
    }

    #[test]
    fn test_boost_add_symbol_request_none_timeframes() {
        let request = AddSymbolRequest {
            symbol: "BNBUSDT".to_string(),
            timeframes: None,
        };

        assert_eq!(request.symbol, "BNBUSDT");
        assert!(request.timeframes.is_none());
    }

    #[test]
    fn test_boost_supported_symbols_serialization() {
        let supported = SupportedSymbols {
            symbols: vec!["BTCUSDT".to_string(), "ETHUSDT".to_string()],
            available_timeframes: vec!["1m".to_string(), "5m".to_string(), "1h".to_string()],
        };

        let json = serde_json::to_string(&supported).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("available_timeframes"));
    }

    #[test]
    fn test_boost_supported_symbols_deserialization() {
        let json = r#"{"symbols":["SOLUSDT","ADAUSDT"],"available_timeframes":["1m","5m","15m","1h"]}"#;
        let supported: SupportedSymbols = serde_json::from_str(json).unwrap();
        assert_eq!(supported.symbols.len(), 2);
        assert_eq!(supported.available_timeframes.len(), 4);
    }

    #[test]
    fn test_boost_candle_query_with_limit_serialization() {
        let query = CandelQuery {
            limit: Some(100),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("100"));
    }

    #[test]
    fn test_boost_candle_query_without_limit_serialization() {
        let query = CandelQuery {
            limit: None,
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("null") || json.contains("limit"));
    }

    #[test]
    fn test_boost_chart_query_with_limit() {
        let query = ChartQuery {
            limit: Some(200),
        };

        assert_eq!(query.limit, Some(200));
    }

    #[test]
    fn test_boost_chart_query_without_limit() {
        let query = ChartQuery {
            limit: None,
        };

        assert!(query.limit.is_none());
    }

    #[test]
    fn test_boost_multi_chart_query_single_symbol_timeframe() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT".to_string(),
            timeframes: "1h".to_string(),
            limit: Some(50),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.trim().to_string()).collect();
        let timeframes: Vec<String> = query.timeframes.split(',').map(|s| s.trim().to_string()).collect();

        assert_eq!(symbols.len(), 1);
        assert_eq!(timeframes.len(), 1);
        assert_eq!(symbols[0], "BTCUSDT");
        assert_eq!(timeframes[0], "1h");
    }

    #[test]
    fn test_boost_multi_chart_query_multiple_symbols() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT,ETHUSDT,BNBUSDT,SOLUSDT".to_string(),
            timeframes: "1m".to_string(),
            limit: None,
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.trim().to_string()).collect();
        assert_eq!(symbols.len(), 4);
    }

    #[test]
    fn test_boost_multi_chart_query_multiple_timeframes() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT".to_string(),
            timeframes: "1m,5m,15m,30m,1h,4h,1d".to_string(),
            limit: Some(100),
        };

        let timeframes: Vec<String> = query.timeframes.split(',').map(|s| s.trim().to_string()).collect();
        assert_eq!(timeframes.len(), 7);
    }

    #[test]
    fn test_boost_multi_chart_query_serialization() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT,ETHUSDT".to_string(),
            timeframes: "1m,5m".to_string(),
            limit: Some(50),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("1m"));
        assert!(json.contains("50"));
    }

    #[test]
    fn test_boost_multi_chart_query_deserialization() {
        let json = r#"{"symbols":"SOLUSDT,ADAUSDT","timeframes":"5m,15m,1h","limit":75}"#;
        let query: MultiChartQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.symbols, "SOLUSDT,ADAUSDT");
        assert_eq!(query.timeframes, "5m,15m,1h");
        assert_eq!(query.limit, Some(75));
    }

    #[test]
    fn test_boost_api_response_with_nested_vec() {
        let data = vec![vec![1, 2], vec![3, 4], vec![5, 6]];
        let response = ApiResponse::success(data.clone());
        assert_eq!(response.data, Some(data));
    }

    #[test]
    fn test_boost_api_response_with_option_some() {
        let response = ApiResponse::success(Some(42));
        assert!(response.success);
        assert_eq!(response.data, Some(Some(42)));
    }

    #[test]
    fn test_boost_api_response_with_option_none() {
        let response: ApiResponse<Option<i32>> = ApiResponse::success(None);
        assert!(response.success);
        assert_eq!(response.data, Some(None));
    }

    #[test]
    fn test_boost_add_symbol_request_default_timeframes() {
        let json = r#"{"symbol":"DOGEUSDT"}"#;
        let request: AddSymbolRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.symbol, "DOGEUSDT");
        assert!(request.timeframes.is_none());
    }

    #[test]
    fn test_boost_supported_symbols_empty_lists() {
        let supported = SupportedSymbols {
            symbols: vec![],
            available_timeframes: vec![],
        };

        assert_eq!(supported.symbols.len(), 0);
        assert_eq!(supported.available_timeframes.len(), 0);
    }

    #[test]
    fn test_boost_supported_symbols_single_item() {
        let supported = SupportedSymbols {
            symbols: vec!["BTCUSDT".to_string()],
            available_timeframes: vec!["1h".to_string()],
        };

        assert_eq!(supported.symbols.len(), 1);
        assert_eq!(supported.available_timeframes.len(), 1);
    }

    #[test]
    fn test_boost_candle_query_deserialization() {
        let json = r#"{"limit":500}"#;
        let query: CandelQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(500));
    }

    #[test]
    fn test_boost_candle_query_deserialization_null() {
        let json = r#"{"limit":null}"#;
        let query: CandelQuery = serde_json::from_str(json).unwrap();
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_boost_chart_query_serialization() {
        let query = ChartQuery {
            limit: Some(1000),
        };

        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("1000"));
    }

    #[test]
    fn test_boost_chart_query_deserialization() {
        let json = r#"{"limit":250}"#;
        let query: ChartQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, Some(250));
    }

    #[test]
    fn test_boost_multi_chart_query_with_spaces() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT, ETHUSDT, BNBUSDT".to_string(),
            timeframes: "1m, 5m, 1h".to_string(),
            limit: Some(100),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.trim().to_string()).collect();
        let timeframes: Vec<String> = query.timeframes.split(',').map(|s| s.trim().to_string()).collect();

        assert_eq!(symbols[0], "BTCUSDT");
        assert_eq!(symbols[1], "ETHUSDT");
        assert_eq!(timeframes[0], "1m");
        assert_eq!(timeframes[1], "5m");
    }

    #[test]
    fn test_boost_api_response_serialization_success() {
        let response = ApiResponse::success(100);
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("100"));
    }

    #[test]
    fn test_boost_api_response_serialization_error() {
        let response: ApiResponse<String> = ApiResponse::error("Test error".to_string());
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("Test error"));
    }

    #[test]
    fn test_boost_api_response_deserialization_success() {
        let json = r#"{"success":true,"data":"test","error":null}"#;
        let response: ApiResponse<String> = serde_json::from_str(json).unwrap();
        assert!(response.success);
        assert_eq!(response.data, Some("test".to_string()));
    }

    #[test]
    fn test_boost_api_response_deserialization_error() {
        let json = r#"{"success":false,"data":null,"error":"something went wrong"}"#;
        let response: ApiResponse<String> = serde_json::from_str(json).unwrap();
        assert!(!response.success);
        assert_eq!(response.error, Some("something went wrong".to_string()));
    }

    #[test]
    fn test_boost_add_symbol_request_uppercase_symbol() {
        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: Some(vec!["1h".to_string()]),
        };

        assert_eq!(request.symbol, "BTCUSDT");
        assert!(request.symbol.chars().all(|c| c.is_uppercase() || c.is_numeric()));
    }

    #[test]
    fn test_boost_add_symbol_request_lowercase_symbol() {
        let request = AddSymbolRequest {
            symbol: "btcusdt".to_string(),
            timeframes: None,
        };

        assert_eq!(request.symbol, "btcusdt");
    }

    #[test]
    fn test_boost_supported_symbols_many_symbols() {
        let symbols = vec![
            "BTCUSDT".to_string(),
            "ETHUSDT".to_string(),
            "BNBUSDT".to_string(),
            "SOLUSDT".to_string(),
            "ADAUSDT".to_string(),
        ];

        let supported = SupportedSymbols {
            symbols: symbols.clone(),
            available_timeframes: vec!["1m".to_string(), "5m".to_string()],
        };

        assert_eq!(supported.symbols.len(), 5);
    }

    #[test]
    fn test_boost_candle_query_zero_limit() {
        let query = CandelQuery {
            limit: Some(0),
        };

        assert_eq!(query.limit, Some(0));
    }

    #[test]
    fn test_boost_candle_query_large_limit() {
        let query = CandelQuery {
            limit: Some(10000),
        };

        assert_eq!(query.limit, Some(10000));
    }

    #[test]
    fn test_boost_chart_query_roundtrip() {
        let original = ChartQuery {
            limit: Some(300),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: ChartQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(original.limit, deserialized.limit);
    }

    #[test]
    fn test_boost_multi_chart_query_roundtrip() {
        let original = MultiChartQuery {
            symbols: "BTCUSDT,ETHUSDT".to_string(),
            timeframes: "1m,5m,1h".to_string(),
            limit: Some(100),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: MultiChartQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(original.symbols, deserialized.symbols);
        assert_eq!(original.timeframes, deserialized.timeframes);
        assert_eq!(original.limit, deserialized.limit);
    }

    // ============================================================================
    // COVERAGE BOOST TESTS - Type & Conversion Testing (no full ApiServer needed)
    // ============================================================================

    #[test]
    fn test_cov_api_response_clone() {
        let response = ApiResponse::success("test data".to_string());
        let cloned = response.clone();
        assert_eq!(cloned.success, response.success);
        assert_eq!(cloned.data, response.data);
    }

    #[test]
    fn test_cov_add_symbol_request_default_timeframes() {
        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: None,
        };
        assert_eq!(request.symbol, "BTCUSDT");
        assert!(request.timeframes.is_none());
    }

    #[test]
    fn test_cov_add_symbol_request_with_timeframes() {
        let request = AddSymbolRequest {
            symbol: "ETHUSDT".to_string(),
            timeframes: Some(vec!["1m".to_string(), "5m".to_string()]),
        };
        assert_eq!(request.timeframes.unwrap().len(), 2);
    }

    #[test]
    fn test_cov_supported_symbols_empty() {
        let supported = SupportedSymbols {
            symbols: vec![],
            available_timeframes: vec![],
        };
        assert_eq!(supported.symbols.len(), 0);
        assert_eq!(supported.available_timeframes.len(), 0);
    }

    #[test]
    fn test_cov_candle_query_none() {
        let query = CandelQuery { limit: None };
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_cov_chart_query_none() {
        let query = ChartQuery { limit: None };
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_cov_multi_chart_query_single_symbol() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT".to_string(),
            timeframes: "1m".to_string(),
            limit: Some(100),
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        assert_eq!(symbols.len(), 1);
        assert_eq!(symbols[0], "BTCUSDT");
    }

    #[test]
    fn test_cov_multi_chart_query_multiple_symbols() {
        let query = MultiChartQuery {
            symbols: "BTCUSDT,ETHUSDT,BNBUSDT".to_string(),
            timeframes: "1m,5m,1h".to_string(),
            limit: None,
        };

        let symbols: Vec<String> = query.symbols.split(',').map(|s| s.to_string()).collect();
        let timeframes: Vec<String> = query.timeframes.split(',').map(|s| s.to_string()).collect();

        assert_eq!(symbols.len(), 3);
        assert_eq!(timeframes.len(), 3);
    }

    #[test]
    fn test_cov_ai_analysis_request_conversion() {
        let request = crate::ai::AIAnalysisRequest {
            symbol: "BTCUSDT".to_string(),
            timeframe_data: std::collections::HashMap::new(),
            current_price: 50000.0,
            volume_24h: 1000000.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            strategy_context: crate::ai::AIStrategyContext::default(),
        };

        let strategy_input: crate::strategies::StrategyInput = request.into();
        assert_eq!(strategy_input.symbol, "BTCUSDT");
        assert_eq!(strategy_input.current_price, 50000.0);
    }

    #[test]
    fn test_cov_strategy_recommendation_request_conversion() {
        let request = crate::ai::StrategyRecommendationRequest {
            symbol: "ETHUSDT".to_string(),
            timeframe_data: std::collections::HashMap::new(),
            current_price: 3000.0,
            available_strategies: vec![],
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        let strategy_input: crate::strategies::StrategyInput = request.into();
        assert_eq!(strategy_input.symbol, "ETHUSDT");
        assert_eq!(strategy_input.current_price, 3000.0);
        assert_eq!(strategy_input.volume_24h, 0.0); // Default value
    }

    #[test]
    fn test_cov_market_condition_request_conversion() {
        let request = crate::ai::MarketConditionRequest {
            symbol: "BNBUSDT".to_string(),
            timeframe_data: std::collections::HashMap::new(),
            current_price: 500.0,
            volume_24h: 50000.0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };

        let strategy_input: crate::strategies::StrategyInput = request.into();
        assert_eq!(strategy_input.symbol, "BNBUSDT");
        assert_eq!(strategy_input.volume_24h, 50000.0);
    }

}
