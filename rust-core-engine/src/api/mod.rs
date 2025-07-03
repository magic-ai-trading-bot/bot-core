use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::convert::Infallible;
use std::sync::Arc;
use tokio::sync::RwLock;
use warp::{Filter, Reply};
use tracing::{error, info};

use crate::config::ApiConfig;
use crate::market_data::MarketDataProcessor;
use crate::trading::TradingEngine;
use crate::monitoring::{MonitoringService, SystemMetrics, TradingMetrics, ConnectionStatus};

#[derive(Clone)]
pub struct ApiServer {
    config: ApiConfig,
    market_data: MarketDataProcessor,
    trading_engine: TradingEngine,
    monitoring: Arc<RwLock<MonitoringService>>,
}

#[derive(Serialize, Deserialize)]
struct ApiResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<String>,
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
    ) -> Self {
        Self {
            config,
            market_data,
            trading_engine,
            monitoring: Arc::new(RwLock::new(MonitoringService::new())),
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
            .allow_headers(vec!["content-type"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE"]);

        // Health check
        let health = warp::path("health")
            .and(warp::get())
            .map(|| {
                warp::reply::json(&ApiResponse::success("Bot is running"))
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

        api.with(cors)
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

        warp::path("market")
            .and(prices.or(overview).or(candles))
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
}

#[derive(Deserialize)]
struct CandelQuery {
    limit: Option<usize>,
} 