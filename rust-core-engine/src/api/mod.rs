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

use crate::config::ApiConfig;
use crate::market_data::{MarketDataProcessor, ChartData, CandleData};
use crate::trading::TradingEngine;
use crate::monitoring::{MonitoringService, SystemMetrics, TradingMetrics, ConnectionStatus};

#[derive(Clone)]
pub struct ApiServer {
    config: ApiConfig,
    market_data: MarketDataProcessor,
    trading_engine: TradingEngine,
    monitoring: Arc<RwLock<MonitoringService>>,
    ws_broadcaster: broadcast::Sender<String>,
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
    pub fn new(
        config: ApiConfig,
        market_data: MarketDataProcessor,
        trading_engine: TradingEngine,
        ws_broadcaster: broadcast::Sender<String>,
    ) -> Self {
        Self {
            config,
            market_data,
            trading_engine,
            monitoring: Arc::new(RwLock::new(MonitoringService::new())),
            ws_broadcaster,
        }
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

        let api = warp::path("api")
            .and(
                health
                    .or(market_data)
                    .or(trading)
                    .or(monitoring)
            );

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
        let (mut ws_sender, mut ws_receiver) = ws.split();
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