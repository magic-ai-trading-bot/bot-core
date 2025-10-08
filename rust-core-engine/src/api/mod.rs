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
use crate::auth::{AuthService, UserRepository};
use crate::config::ApiConfig;
use crate::market_data::MarketDataProcessor;
use crate::monitoring::MonitoringService;
use crate::paper_trading::PaperTradingEngine;
use crate::storage::Storage;
use crate::trading::TradingEngine;

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

#[derive(Serialize, Deserialize, Clone)]
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
            let jwt_secret = std::env::var("JWT_SECRET")
                .unwrap_or_else(|_| "default_jwt_secret_change_in_production".to_string());
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
        info!(
            "Starting API server on {}:{}",
            self.config.host, self.config.port
        );

        let api = self.create_routes();

        warp::serve(api).run(([0, 0, 0, 0], self.config.port)).await;

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
            .map(|| warp::reply::json(&ApiResponse::success("Bot is running")));

        // WebSocket endpoint
        let ws_broadcaster = self.ws_broadcaster.clone();
        let websocket = warp::path("ws").and(warp::ws()).map(move |ws: Ws| {
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
        let paper_trading_api =
            paper_trading::PaperTradingApi::new(self.paper_trading_engine.clone());
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
                    Ok(overview) => {
                        Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(overview)))
                    }
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
        let add_symbol = warp::path("symbols")
            .and(warp::post())
            .and(warp::body::json())
            .and(warp::any().map(move || market_data_clone5.clone()))
            .and_then(
                |request: AddSymbolRequest, market_data: MarketDataProcessor| async move {
                    match market_data
                        .add_symbol(request.symbol, request.timeframes)
                        .await
                    {
                        Ok(_) => Ok::<_, warp::Rejection>(warp::reply::json(
                            &ApiResponse::success("Symbol added successfully"),
                        )),
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
                    Ok(account) => {
                        Ok::<_, warp::Rejection>(warp::reply::json(&ApiResponse::success(account)))
                    }
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
                    }
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
                                "signal": response.signal.as_str().to_lowercase(),
                                "confidence": response.confidence,
                                "timestamp": response.timestamp,
                                "model_type": "GPT-4",
                                "timeframe": "1h",
                                "reasoning": response.reasoning,
                                "strategy_scores": response.strategy_scores
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
                    }
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
            timeframes: vec!["1m".to_string(), "5m".to_string(), "1h".to_string()],
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
        assert_eq!(request.timeframes.len(), 3);
        assert_eq!(request.timeframes[0], "15m");
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
        let response = ApiResponse::success(3.14159);
        assert!(response.success);
        assert_eq!(response.data, Some(3.14159));
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
            timeframes: vec![],
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AddSymbolRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert!(deserialized.timeframes.is_empty());
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

        let data = Nested { inner: vec![1, 2, 3, 4, 5] };
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
            timeframes: vec!["1m".to_string(), "5m".to_string()],
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: AddSymbolRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, request.symbol);
        assert_eq!(deserialized.timeframes.len(), request.timeframes.len());
    }

    #[test]
    fn test_add_symbol_request_with_many_timeframes() {
        let timeframes = vec![
            "1m", "3m", "5m", "15m", "30m", "1h", "2h", "4h", "6h", "8h", "12h", "1d",
        ]
        .into_iter()
        .map(|s| s.to_string())
        .collect();

        let request = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes,
        };

        assert_eq!(request.timeframes.len(), 12);
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
        assert_eq!(deserialized.available_timeframes, original.available_timeframes);
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
            timeframes: vec![],
        };
        let request2 = AddSymbolRequest {
            symbol: "BTCUSDT".to_string(),
            timeframes: vec![],
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
                timeframes: vec!["1h".to_string()],
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
            timeframes: vec!["1m".to_string()],
        };

        let json = serde_json::to_value(&request).unwrap();
        assert!(json.get("symbol").is_some());
        assert!(json.get("timeframes").is_some());
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

    // ============================================================================
    // Boundary Value Tests
    // ============================================================================

    #[test]
    fn test_candle_query_boundary_values() {
        let queries = vec![
            CandelQuery { limit: Some(0) },
            CandelQuery { limit: Some(1) },
            CandelQuery { limit: Some(1000) },
            CandelQuery { limit: Some(usize::MAX) },
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
            timeframes: vec!["1h".to_string()],
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
        let json = r#"{"symbol": "BTCUSDT"}"#; // missing timeframes
        let result = serde_json::from_str::<AddSymbolRequest>(json);
        assert!(result.is_err());
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
}
