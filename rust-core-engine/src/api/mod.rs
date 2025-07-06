use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};
use warp::ws::{Message, WebSocket, Ws};
use futures_util::{SinkExt, StreamExt};
use tokio::sync::broadcast;
use tracing::{error, info, debug};
use serde_json;

use crate::config::ApiConfig;
use crate::market_data::MarketDataProcessor;
use crate::trading::TradingEngine;
use crate::monitoring::MonitoringService;
use crate::auth::{AuthService, UserRepository};
use crate::storage::Storage;
use crate::ai::AIService;
use crate::paper_trading::PaperTradingEngine;

pub mod paper_trading;

#[derive(Clone)]
pub struct ApiServer {
    config: ApiConfig,
    market_data: MarketDataProcessor,
    trading_engine: TradingEngine,
    paper_trading_engine: Arc<PaperTradingEngine>,
    monitoring: Arc<RwLock<MonitoringService>>,
    ws_broadcaster: broadcast::Sender<String>,
    auth_service: AuthService,
    ai_service: AIService,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Serialize, Deserialize)]
struct AddSymbolRequest {
    symbol: String,
    timeframes: Vec<String>,
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
    pub async fn new(
        config: ApiConfig,
        market_data: MarketDataProcessor,
        trading_engine: TradingEngine,
        paper_trading_engine: Arc<PaperTradingEngine>,
        ws_broadcaster: broadcast::Sender<String>,
        storage: Storage,
    ) -> Result<Self> {
        // Initialize auth service - use dummy implementation if database is not available
        let auth_service = if let Some(db) = storage.get_database() {
            let user_repo = UserRepository::new(db).await?;
            let jwt_secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret_change_in_production".to_string());
            AuthService::new(user_repo, jwt_secret)
        } else {
            // Create a dummy auth service that returns errors for all operations
            AuthService::new_dummy()
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
            market_data,
            trading_engine,
            paper_trading_engine,
            monitoring: Arc::new(RwLock::new(MonitoringService::new())),
            ws_broadcaster,
            auth_service,
            ai_service,
        })
    }

    pub async fn start(&self) -> Result<()> {
        info!("Starting API server on {}:{}", self.config.host, self.config.port);

        let api = self.create_routes();

        warp::serve(api)
            .run(([0, 0, 0, 0], self.config.port))
            .await;

        Ok(())
    }

    fn create_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "x-client", "authorization", "accept"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        // Health check
        let health = warp::path("health")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&ApiResponse::success("Bot is running"))
            });

        // WebSocket endpoint
        let ws_broadcaster = self.ws_broadcaster.clone();
        let websocket = warp::path("ws")
            .and(warp::ws())
            .map(move |ws: Ws| {
                let broadcaster = ws_broadcaster.clone();
                ws.on_upgrade(move |websocket| Self::handle_websocket(websocket, broadcaster))
            });

        // Market data routes
        let market_data = self.market_data_routes();
        
        // Trading routes
        let trading = self.trading_routes();
        
        // Monitoring routes
        let monitoring = self.monitoring_routes();

        // AI routes
        let ai_routes = self.ai_routes();

        // Paper trading routes
        let paper_trading_api = paper_trading::PaperTradingApi::new(self.paper_trading_engine.clone());
        let paper_trading = paper_trading_api.routes();

        // Combine all routes
        let api_routes = health
            .or(market_data)
            .or(trading)
            .or(monitoring)
            .or(ai_routes)
            .or(paper_trading)
            .or(self.auth_service.routes());

        let api = warp::path("api").and(api_routes);

        // Root level routes (not under /api prefix)
        let root_routes = websocket;

        api.with(cors.clone())
            .or(root_routes.with(cors))
    }

    fn market_data_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let market_data = self.market_data.clone();

        // Get latest prices
        let prices = warp::path("prices")
            .and(warp::get())
            .and(warp::any().map(move || market_data.clone()))
            .and_then(|market_data: MarketDataProcessor| async move {
                let symbols = market_data.get_supported_symbols();
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
                    Ok(overview) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(overview))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
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
            .and_then(|symbol: String, timeframe: String, query: CandelQuery, market_data: MarketDataProcessor| async move {
                let candles = market_data.get_cache().get_candles(&symbol, &timeframe, query.limit);
                Ok::<_, Infallible>(warp::reply::json(&ApiResponse::success(candles)))
            });

        // NEW: Get comprehensive chart data with multiple timeframes
        let market_data_clone3 = self.market_data.clone();
        let chart_data = warp::path("chart")
            .and(warp::path::param::<String>()) // symbol
            .and(warp::path::param::<String>()) // timeframe
            .and(warp::query::<ChartQuery>())
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone3.clone()))
            .and_then(|symbol: String, timeframe: String, query: ChartQuery, market_data: MarketDataProcessor| async move {
                match market_data.get_chart_data(&symbol, &timeframe, query.limit).await {
                    Ok(chart_data) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(chart_data))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // NEW: Get multiple symbols chart data at once
        let market_data_clone4 = self.market_data.clone();
        let multi_chart = warp::path("charts")
            .and(warp::query::<MultiChartQuery>())
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone4.clone()))
            .and_then(|query: MultiChartQuery, market_data: MarketDataProcessor| async move {
                let symbols = query.symbols.split(',').map(|s| s.to_string()).collect::<Vec<_>>();
                let timeframes = query.timeframes.split(',').map(|s| s.to_string()).collect::<Vec<_>>();
                
                match market_data.get_multi_chart_data(symbols, timeframes, query.limit).await {
                    Ok(charts) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(charts))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // NEW: Add new symbol to track
        let market_data_clone5 = self.market_data.clone();
        let add_symbol = warp::path("symbols")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || market_data_clone5.clone()))
            .and_then(|request: AddSymbolRequest, market_data: MarketDataProcessor| async move {
                match market_data.add_symbol(request.symbol, request.timeframes).await {
                    Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success("Symbol added successfully"))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // NEW: Remove symbol from tracking
        let market_data_clone6 = self.market_data.clone();
        let remove_symbol = warp::path("symbols")
            .and(warp::path::param::<String>()) // symbol
            .and(warp::delete())
            .and(warp::any().map(move || market_data_clone6.clone()))
            .and_then(|symbol: String, market_data: MarketDataProcessor| async move {
                match market_data.remove_symbol(&symbol).await {
                    Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success("Symbol removed successfully"))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // NEW: Get all supported symbols and timeframes
        let market_data_clone7 = self.market_data.clone();
        let symbols_info = warp::path("symbols")
            .and(warp::get())
            .and(warp::any().map(move || market_data_clone7.clone()))
            .map(|market_data: MarketDataProcessor| {
                let symbols = market_data.get_supported_symbols();
                let timeframes = market_data.get_supported_timeframes();
                let response = SupportedSymbols {
                    symbols,
                    available_timeframes: timeframes,
                };
                warp::reply::json(&ApiResponse::success(response))
            });

        warp::path("market")
            .and(
                prices
                    .or(overview)
                    .or(candles)
                    .or(chart_data)
                    .or(multi_chart)
                    .or(symbols_info)
                    .or(add_symbol)
                    .or(remove_symbol)
            )
    }

    fn trading_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
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
                    Ok(account) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(account))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
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
                    Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success("Position closed"))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // Performance stats
        let trading_engine_clone3 = self.trading_engine.clone();
        let performance = warp::path("performance")
            .and(warp::get())
            .and(warp::any().map(move || trading_engine_clone3.clone()))
            .and_then(|trading_engine: TradingEngine| async move {
                match trading_engine.get_performance_stats().await {
                    Ok(stats) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(stats))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        warp::path("trading")
            .and(positions.or(account).or(close_position).or(performance))
    }

    fn monitoring_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
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

        warp::path("monitoring")
            .and(system_metrics.or(trading_metrics).or(connection_status))
    }

    pub async fn update_monitoring(&self, 
        active_positions: usize, 
        cache_size: usize,
        websocket_connected: bool,
        api_responsive: bool
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
        let ws_sender_clone = Arc::new(tokio::sync::Mutex::new(ws_sender));
        let ws_sender_for_broadcast = ws_sender_clone.clone();
        
        // Task to handle incoming messages
        let incoming_task = tokio::spawn(async move {
            while let Some(result) = ws_receiver.next().await {
                match result {
                    Ok(msg) => {
                        if msg.is_text() {
                            debug!("Received WebSocket message: {:?}", msg);
                        } else if msg.is_close() {
                            debug!("WebSocket connection closed by client");
                            break;
                        }
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        break;
                    }
                }
            }
        });
        
        // Task to handle outgoing broadcasts
        let outgoing_task = tokio::spawn(async move {
            while let Ok(message) = rx.recv().await {
                let mut sender = ws_sender_for_broadcast.lock().await;
                if let Err(e) = sender.send(Message::text(message)).await {
                    error!("Failed to send WebSocket message: {}", e);
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

    fn ai_routes(&self) -> impl Filter<Extract = impl Reply, Error = warp::Rejection> + Clone {
        let ai_service = self.ai_service.clone();
        let ws_broadcaster = self.ws_broadcaster.clone();
        
        // AI analysis endpoint with WebSocket broadcasting
        let ai_analyze = warp::path("analyze")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || ai_service.clone()))
            .and(warp::any().map(move || ws_broadcaster.clone()))
            .and_then(|request: crate::ai::AIAnalysisRequest, ai_service: crate::ai::AIService, broadcaster: broadcast::Sender<String>| async move {
                let strategy_context = request.strategy_context.clone();
                let symbol = request.symbol.clone();
                
                match ai_service.analyze_for_trading_signal(&request.into(), strategy_context).await {
                    Ok(response) => {
                        // Broadcast AI signal via WebSocket
                        let signal_message = serde_json::json!({
                            "type": "AISignalReceived",
                            "data": {
                                "symbol": symbol,
                                "signal": response.signal.to_string().to_lowercase(),
                                "confidence": response.confidence,
                                "timestamp": response.timestamp,
                                "model_type": "GPT-4",
                                "timeframe": "1h",
                                "reasoning": response.reasoning,
                                "strategy_scores": response.strategy_scores
                            },
                                                         "timestamp": std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as u64
                        });
                        
                        if let Ok(message_str) = serde_json::to_string(&signal_message) {
                            if let Err(_) = broadcaster.send(message_str) {
                                // Log error if needed, but don't fail the request
                                debug!("No WebSocket subscribers for AI signal broadcast");
                            } else {
                                info!("📡 Broadcasted AI signal for {} via WebSocket", symbol);
                            }
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
            .and_then(|request: crate::ai::StrategyRecommendationRequest, ai_service: crate::ai::AIService| async move {
                let market_data = request.clone().into();
                match ai_service.get_strategy_recommendations(&market_data, request.available_strategies).await {
                    Ok(response) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(response))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // Market condition analysis endpoint
        let ai_service_clone2 = self.ai_service.clone();
        let market_condition = warp::path("market-condition")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || ai_service_clone2.clone()))
            .and_then(|request: crate::ai::MarketConditionRequest, ai_service: crate::ai::AIService| async move {
                let market_data = request.into();
                match ai_service.analyze_market_condition(&market_data).await {
                    Ok(response) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(response))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // Performance feedback endpoint
        let ai_service_clone3 = self.ai_service.clone();
        let performance_feedback = warp::path("feedback")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || ai_service_clone3.clone()))
            .and_then(|feedback: crate::ai::PerformanceFeedback, ai_service: crate::ai::AIService| async move {
                match ai_service.send_performance_feedback(feedback).await {
                    Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success("Feedback sent successfully"))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // AI service info endpoint
        let ai_service_clone4 = self.ai_service.clone();
        let ai_info = warp::path("info")
            .and(warp::get())
            .and(warp::any().map(move || ai_service_clone4.clone()))
            .and_then(|ai_service: crate::ai::AIService| async move {
                match ai_service.get_service_info().await {
                    Ok(info) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(info))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        // Supported strategies endpoint
        let ai_service_clone5 = self.ai_service.clone();
        let ai_strategies = warp::path("strategies")
            .and(warp::get())
            .and(warp::any().map(move || ai_service_clone5.clone()))
            .and_then(|ai_service: crate::ai::AIService| async move {
                match ai_service.get_supported_strategies().await {
                    Ok(strategies) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(strategies))),
                    Err(e) => Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::<()>::error(e.to_string()))),
                }
            });

        warp::path("ai")
            .and(ai_analyze.or(strategy_recommendations).or(market_condition).or(performance_feedback).or(ai_info).or(ai_strategies))
    }
}

#[derive(Deserialize)]
struct CandelQuery {
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct ChartQuery {
    limit: Option<usize>,
}

#[derive(Deserialize)]
struct MultiChartQuery {
    symbols: String,      // comma-separated symbols: "BTCUSDT,ETHUSDT,BNBUSDT"
    timeframes: String,   // comma-separated timeframes: "1m,5m,15m,1h"
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