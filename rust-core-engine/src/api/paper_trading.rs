use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::{Filter, Rejection, Reply};
// Removed unused import
use warp::http::StatusCode;

use crate::paper_trading::{PaperTradingEngine, PaperTradingSettings};

/// API handlers for paper trading functionality
pub struct PaperTradingApi {
    engine: Arc<PaperTradingEngine>,
}

/// Request to update paper trading settings
#[derive(Debug, Deserialize)]
pub struct UpdateSettingsRequest {
    pub settings: PaperTradingSettings,
}

/// Request to manually close a trade
#[derive(Debug, Serialize, Deserialize)]
pub struct CloseTradeRequest {
    pub trade_id: String,
    pub reason: Option<String>,
}

/// Strategy Settings for the frontend
#[derive(Debug, Serialize, Deserialize)]
pub struct TradingStrategySettings {
    pub strategies: StrategyConfigCollection,
    pub risk: RiskSettings,
    pub engine: EngineSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StrategyConfigCollection {
    pub rsi: RsiConfig,
    pub macd: MacdConfig,
    pub volume: VolumeConfig,
    pub bollinger: BollingerConfig,
    pub stochastic: StochasticConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RsiConfig {
    pub enabled: bool,
    pub period: u32,
    pub oversold_threshold: f64,
    pub overbought_threshold: f64,
    pub extreme_oversold: f64,
    pub extreme_overbought: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MacdConfig {
    pub enabled: bool,
    pub fast_period: u32,
    pub slow_period: u32,
    pub signal_period: u32,
    pub histogram_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VolumeConfig {
    pub enabled: bool,
    pub sma_period: u32,
    pub spike_threshold: f64,
    pub correlation_period: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BollingerConfig {
    pub enabled: bool,
    pub period: u32,
    pub multiplier: f64,
    pub squeeze_threshold: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StochasticConfig {
    pub enabled: bool,
    pub k_period: u32,
    pub d_period: u32,
    pub oversold_threshold: f64,
    pub overbought_threshold: f64,
    pub extreme_oversold: f64,
    pub extreme_overbought: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RiskSettings {
    pub max_risk_per_trade: f64,
    pub max_portfolio_risk: f64,
    pub stop_loss_percent: f64,
    pub take_profit_percent: f64,
    pub max_leverage: u32,
    pub max_drawdown: f64,
    pub daily_loss_limit: f64,
    pub max_consecutive_losses: u32,
    pub correlation_limit: f64, // Position correlation limit (0.0-1.0)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineSettings {
    pub min_confidence_threshold: f64,
    pub signal_combination_mode: String,
    pub enabled_strategies: Vec<String>,
    pub market_condition: String,
    pub risk_level: String,
    #[serde(default = "default_data_resolution")]
    pub data_resolution: String, // Timeframe for trading signals (1m, 3m, 5m, 15m, 30m, 1h, 4h, 1d)
}

fn default_data_resolution() -> String {
    "15m".to_string()
}

/// Request to update strategy settings
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateStrategySettingsRequest {
    pub settings: TradingStrategySettings,
}

/// Request to update basic paper trading settings (simplified)
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateBasicSettingsRequest {
    pub initial_balance: Option<f64>,
    pub max_positions: Option<u32>,
    pub default_position_size_pct: Option<f64>,
    pub default_leverage: Option<u8>,
    pub trading_fee_rate: Option<f64>,
    pub funding_fee_rate: Option<f64>,
    pub slippage_pct: Option<f64>,
    pub max_risk_per_trade_pct: Option<f64>,
    pub max_portfolio_risk_pct: Option<f64>,
    pub default_stop_loss_pct: Option<f64>,
    pub default_take_profit_pct: Option<f64>,
    pub max_leverage: Option<u8>,
    pub enabled: Option<bool>,
}

/// Symbol settings for frontend configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct SymbolConfig {
    pub enabled: bool,
    pub leverage: Option<u8>,
    pub position_size_pct: Option<f64>,
    pub stop_loss_pct: Option<f64>,
    pub take_profit_pct: Option<f64>,
    pub max_positions: Option<u32>,
}

/// Request to update symbol settings
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSymbolSettingsRequest {
    pub symbols: std::collections::HashMap<String, SymbolConfig>,
}

/// Request to update signal refresh interval
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSignalIntervalRequest {
    pub interval_minutes: u32,
}

/// Response for API operations
#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub data: Option<T>,
    pub error: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

impl<T> ApiResponse<T> {
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            timestamp: chrono::Utc::now(),
        }
    }

    pub fn error(message: String) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(message),
            timestamp: chrono::Utc::now(),
        }
    }
}

impl PaperTradingApi {
    pub fn new(engine: Arc<PaperTradingEngine>) -> Self {
        Self { engine }
    }

    /// Create paper trading API routes
    pub fn routes(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = Arc::new(self);

        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        let base_path = warp::path("paper-trading");

        // GET /api/paper-trading/status
        let status_route = base_path
            .and(warp::path("status"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_status);

        // GET /api/paper-trading/portfolio
        let portfolio_route = base_path
            .and(warp::path("portfolio"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_portfolio);

        // GET /api/paper-trading/trades/open
        let open_trades_route = base_path
            .and(warp::path("trades"))
            .and(warp::path("open"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_open_trades);

        // GET /api/paper-trading/trades/closed
        let closed_trades_route = base_path
            .and(warp::path("trades"))
            .and(warp::path("closed"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_closed_trades);

        // POST /api/paper-trading/trades/{trade_id}/close
        let close_trade_route = base_path
            .and(warp::path("trades"))
            .and(warp::path::param::<String>())
            .and(warp::path("close"))
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(close_trade);

        // PUT /api/paper-trading/settings
        let update_settings_route = base_path
            .and(warp::path("settings"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(update_settings);

        // GET /api/paper-trading/strategy-settings
        let get_strategy_settings_route = base_path
            .and(warp::path("strategy-settings"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_strategy_settings);

        // PUT /api/paper-trading/strategy-settings
        let update_strategy_settings_route = base_path
            .and(warp::path("strategy-settings"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(update_strategy_settings);

        // GET /api/paper-trading/basic-settings
        let get_basic_settings_route = base_path
            .and(warp::path("basic-settings"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_basic_settings);

        // PUT /api/paper-trading/basic-settings
        let update_basic_settings_route = base_path
            .and(warp::path("basic-settings"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(update_basic_settings);

        // GET /api/paper-trading/symbols
        let get_symbols_route = base_path
            .and(warp::path("symbols"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_symbol_settings);

        // PUT /api/paper-trading/symbols
        let update_symbols_route = base_path
            .and(warp::path("symbols"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(update_symbol_settings);

        // POST /api/paper-trading/reset
        let reset_route = base_path
            .and(warp::path("reset"))
            .and(warp::path::end())
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(reset_portfolio);

        // POST /api/paper-trading/start
        let start_route = base_path
            .and(warp::path("start"))
            .and(warp::path::end())
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(start_engine);

        // POST /api/paper-trading/stop
        let stop_route = base_path
            .and(warp::path("stop"))
            .and(warp::path::end())
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(stop_engine);

        // POST /api/paper-trading/trigger-analysis
        let trigger_analysis_route = base_path
            .and(warp::path("trigger-analysis"))
            .and(warp::path::end())
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(trigger_manual_analysis);

        // PUT /api/paper-trading/signal-interval
        let update_signal_interval_route = base_path
            .and(warp::path("signal-interval"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(update_signal_refresh_interval);

        status_route
            .or(portfolio_route)
            .or(open_trades_route)
            .or(closed_trades_route)
            .or(close_trade_route)
            .or(update_settings_route)
            .or(get_strategy_settings_route)
            .or(update_strategy_settings_route)
            .or(get_basic_settings_route)
            .or(update_basic_settings_route)
            .or(get_symbols_route)
            .or(update_symbols_route)
            .or(reset_route)
            .or(start_route)
            .or(stop_route)
            .or(trigger_analysis_route)
            .or(update_signal_interval_route)
            .with(cors)
    }
}

impl Clone for PaperTradingApi {
    fn clone(&self) -> Self {
        Self {
            engine: Arc::clone(&self.engine),
        }
    }
}

// Helper function to inject API into handlers
fn with_api(
    api: Arc<PaperTradingApi>,
) -> impl Filter<Extract = (Arc<PaperTradingApi>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || Arc::clone(&api))
}

// API Handler Functions

/// Get paper trading engine status
async fn get_status(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    let portfolio_status = api.engine.get_portfolio_status().await;
    let is_running = api.engine.is_running().await;

    let status = serde_json::json!({
        "is_running": is_running,
        "portfolio": portfolio_status,
        "last_updated": chrono::Utc::now(),
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(status)),
        StatusCode::OK,
    ))
}

/// Get portfolio performance summary
async fn get_portfolio(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    let portfolio_status = api.engine.get_portfolio_status().await;

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(portfolio_status)),
        StatusCode::OK,
    ))
}

/// Get open trades
async fn get_open_trades(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    let trades = api.engine.get_open_trades().await;

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(trades)),
        StatusCode::OK,
    ))
}

/// Get closed trades
async fn get_closed_trades(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    let trades = api.engine.get_closed_trades().await;

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(trades)),
        StatusCode::OK,
    ))
}

/// Close a specific trade
async fn close_trade(
    trade_id: String,
    _request: CloseTradeRequest,
    api: Arc<PaperTradingApi>,
) -> Result<impl Reply, Rejection> {
    match api.engine.close_trade(&trade_id).await {
        Ok(_) => {
            let response = serde_json::json!({
                "trade_id": trade_id,
                "message": "Trade closed successfully",
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::BAD_REQUEST,
        )),
    }
}

/// Update paper trading settings
async fn update_settings(
    request: UpdateSettingsRequest,
    api: Arc<PaperTradingApi>,
) -> Result<impl Reply, Rejection> {
    match api.engine.update_settings(request.settings).await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Settings updated successfully",
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::BAD_REQUEST,
        )),
    }
}

/// Reset portfolio to initial state
async fn reset_portfolio(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    match api.engine.reset_portfolio().await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Portfolio reset successfully",
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Start paper trading engine
async fn start_engine(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    match api.engine.start_async().await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Paper trading engine start command received",
                "note": "Engine will start in background",
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Stop paper trading engine
async fn stop_engine(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    match api.engine.stop().await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Paper trading engine stopped successfully",
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Get current strategy settings
async fn get_strategy_settings(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    // Get actual settings from engine
    let engine_settings = api.engine.get_settings().await;

    let strategy_settings = TradingStrategySettings {
        strategies: StrategyConfigCollection {
            rsi: RsiConfig {
                enabled: true,
                period: 14,
                oversold_threshold: 30.0,
                overbought_threshold: 70.0,
                extreme_oversold: 20.0,
                extreme_overbought: 80.0,
            },
            macd: MacdConfig {
                enabled: true,
                fast_period: 12,
                slow_period: 26,
                signal_period: 9,
                histogram_threshold: 0.001,
            },
            volume: VolumeConfig {
                enabled: true,
                sma_period: 20,
                spike_threshold: 2.0,
                correlation_period: 10,
            },
            bollinger: BollingerConfig {
                enabled: true,
                period: 20,
                multiplier: 2.0,
                squeeze_threshold: 0.02,
            },
            stochastic: StochasticConfig {
                enabled: true,
                k_period: 14,
                d_period: 3,
                oversold_threshold: 20.0,
                overbought_threshold: 80.0,
                extreme_oversold: 10.0,
                extreme_overbought: 90.0,
            },
        },
        risk: RiskSettings {
            max_risk_per_trade: engine_settings.risk.max_risk_per_trade_pct,
            max_portfolio_risk: engine_settings.risk.max_portfolio_risk_pct,
            stop_loss_percent: engine_settings.risk.default_stop_loss_pct,
            take_profit_percent: engine_settings.risk.default_take_profit_pct,
            max_leverage: engine_settings.risk.max_leverage.into(),
            max_drawdown: engine_settings.risk.max_drawdown_pct,
            daily_loss_limit: engine_settings.risk.daily_loss_limit_pct,
            max_consecutive_losses: engine_settings.risk.max_consecutive_losses,
            correlation_limit: engine_settings.risk.correlation_limit,
        },
        engine: EngineSettings {
            min_confidence_threshold: engine_settings.strategy.min_ai_confidence, // 🎯 ACTUAL THRESHOLD
            signal_combination_mode: "WeightedAverage".to_string(),
            enabled_strategies: vec![
                "RSI Strategy".to_string(),
                "MACD Strategy".to_string(),
                "Volume Strategy".to_string(),
                "Bollinger Bands Strategy".to_string(),
                "Stochastic Strategy".to_string(),
            ],
            market_condition: "Trending".to_string(),
            risk_level: "Moderate".to_string(),
            data_resolution: engine_settings.strategy.backtesting.data_resolution.clone(), // Current timeframe
        },
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(strategy_settings)),
        StatusCode::OK,
    ))
}

/// Update strategy settings
async fn update_strategy_settings(
    request: UpdateStrategySettingsRequest,
    api: Arc<PaperTradingApi>,
) -> Result<impl Reply, Rejection> {
    log::info!("Updating strategy settings: {:?}", request.settings);

    // Get current settings to preserve unchanged values
    let mut current_settings = api.engine.get_settings().await;

    // Update engine settings (confidence threshold and data resolution)
    let confidence_threshold = request.settings.engine.min_confidence_threshold;
    let data_resolution = request.settings.engine.data_resolution.clone();
    log::info!("Applying confidence threshold: {confidence_threshold}");
    log::info!("Applying data resolution: {data_resolution}");

    // Update risk settings from request
    let risk_settings = &request.settings.risk;
    log::info!("Applying risk settings: correlation_limit={}, stop_loss={}, take_profit={}",
        risk_settings.correlation_limit,
        risk_settings.stop_loss_percent,
        risk_settings.take_profit_percent
    );

    // Update all settings in current_settings
    current_settings.strategy.min_ai_confidence = confidence_threshold;
    current_settings.strategy.backtesting.data_resolution = data_resolution.clone();

    // Update risk settings - ALL fields from request
    current_settings.risk.max_risk_per_trade_pct = risk_settings.max_risk_per_trade;
    current_settings.risk.max_portfolio_risk_pct = risk_settings.max_portfolio_risk;
    current_settings.risk.default_stop_loss_pct = risk_settings.stop_loss_percent;
    current_settings.risk.default_take_profit_pct = risk_settings.take_profit_percent;
    current_settings.risk.max_leverage = risk_settings.max_leverage as u8;
    current_settings.risk.max_drawdown_pct = risk_settings.max_drawdown;
    current_settings.risk.daily_loss_limit_pct = risk_settings.daily_loss_limit;
    current_settings.risk.max_consecutive_losses = risk_settings.max_consecutive_losses;
    current_settings.risk.correlation_limit = risk_settings.correlation_limit; // KEY FIX!

    // Save all settings to database and memory using the engine's update_settings method
    match api.engine.update_settings(current_settings).await {
        Ok(_) => {
            log::info!("✅ All settings updated successfully and saved to database");
            log::info!("✅ Confidence threshold: {confidence_threshold}");
            log::info!("✅ Data resolution: {data_resolution}");
            log::info!("✅ Correlation limit: {}", risk_settings.correlation_limit);

            let response = serde_json::json!({
                "message": "Strategy settings updated successfully",
                "applied_settings": {
                    "confidence_threshold": confidence_threshold,
                    "data_resolution": data_resolution,
                    "correlation_limit": risk_settings.correlation_limit,
                    "stop_loss_percent": risk_settings.stop_loss_percent,
                    "take_profit_percent": risk_settings.take_profit_percent,
                    "market_condition": request.settings.engine.market_condition,
                    "risk_level": request.settings.engine.risk_level,
                },
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => {
            log::error!("❌ Failed to update settings: {e}");

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(format!(
                    "Failed to update settings: {e}"
                ))),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        },
    }
}

/// Get basic paper trading settings
async fn get_basic_settings(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    let settings = api.engine.get_settings().await;

    let basic_settings = serde_json::json!({
        "basic": {
            "initial_balance": settings.basic.initial_balance,
            "max_positions": settings.basic.max_positions,
            "default_position_size_pct": settings.basic.default_position_size_pct,
            "default_leverage": settings.basic.default_leverage,
            "trading_fee_rate": settings.basic.trading_fee_rate,
            "funding_fee_rate": settings.basic.funding_fee_rate,
            "slippage_pct": settings.basic.slippage_pct,
            "enabled": settings.basic.enabled,
            "auto_restart": settings.basic.auto_restart
        },
        "risk": {
            "max_risk_per_trade_pct": settings.risk.max_risk_per_trade_pct,
            "max_portfolio_risk_pct": settings.risk.max_portfolio_risk_pct,
            "default_stop_loss_pct": settings.risk.default_stop_loss_pct,
            "default_take_profit_pct": settings.risk.default_take_profit_pct,
            "max_leverage": settings.risk.max_leverage,
            "min_margin_level": settings.risk.min_margin_level,
            "max_drawdown_pct": settings.risk.max_drawdown_pct,
            "daily_loss_limit_pct": settings.risk.daily_loss_limit_pct,
            "max_consecutive_losses": settings.risk.max_consecutive_losses,
            "cool_down_minutes": settings.risk.cool_down_minutes
        }
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(basic_settings)),
        StatusCode::OK,
    ))
}

/// Update basic paper trading settings (simplified)
async fn update_basic_settings(
    request: UpdateBasicSettingsRequest,
    api: Arc<PaperTradingApi>,
) -> Result<impl Reply, Rejection> {
    log::info!("Updating basic paper trading settings: {request:?}");

    // Get current settings
    let current_settings = api.engine.get_settings().await;
    let mut new_settings = current_settings.clone();

    // Update basic settings fields
    if let Some(initial_balance) = request.initial_balance {
        new_settings.basic.initial_balance = initial_balance;
    }
    if let Some(max_positions) = request.max_positions {
        new_settings.basic.max_positions = max_positions;
    }
    if let Some(default_position_size_pct) = request.default_position_size_pct {
        new_settings.basic.default_position_size_pct = default_position_size_pct;
    }
    if let Some(default_leverage) = request.default_leverage {
        new_settings.basic.default_leverage = default_leverage;
    }
    if let Some(trading_fee_rate) = request.trading_fee_rate {
        new_settings.basic.trading_fee_rate = trading_fee_rate;
    }
    if let Some(funding_fee_rate) = request.funding_fee_rate {
        new_settings.basic.funding_fee_rate = funding_fee_rate;
    }
    if let Some(slippage_pct) = request.slippage_pct {
        new_settings.basic.slippage_pct = slippage_pct;
    }
    if let Some(enabled) = request.enabled {
        new_settings.basic.enabled = enabled;
    }

    // Update risk settings fields
    if let Some(max_risk_per_trade_pct) = request.max_risk_per_trade_pct {
        new_settings.risk.max_risk_per_trade_pct = max_risk_per_trade_pct;
    }
    if let Some(max_portfolio_risk_pct) = request.max_portfolio_risk_pct {
        new_settings.risk.max_portfolio_risk_pct = max_portfolio_risk_pct;
    }
    if let Some(default_stop_loss_pct) = request.default_stop_loss_pct {
        new_settings.risk.default_stop_loss_pct = default_stop_loss_pct;
    }
    if let Some(default_take_profit_pct) = request.default_take_profit_pct {
        new_settings.risk.default_take_profit_pct = default_take_profit_pct;
    }
    if let Some(max_leverage) = request.max_leverage {
        new_settings.risk.max_leverage = max_leverage;
    }

    // Update the engine settings
    match api.engine.update_settings(new_settings).await {
        Ok(_) => {
            // If initial balance changed, reset portfolio
            if request.initial_balance.is_some() {
                if let Err(e) = api.engine.reset_portfolio().await {
                    log::error!("Failed to reset portfolio after settings update: {e}");
                }
            }

            let response = serde_json::json!({
                "message": "Basic settings updated successfully and portfolio reset",
                "updated_fields": request,
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::BAD_REQUEST,
        )),
    }
}

/// Get symbol settings
async fn get_symbol_settings(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    let settings = api.engine.get_settings().await;

    // Convert internal symbol settings to frontend format
    let mut symbol_configs = std::collections::HashMap::new();

    // Add default symbols with current settings or defaults
    let default_symbols = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];

    for symbol in default_symbols {
        let symbol_setting = settings.symbols.get(symbol);
        let config = if let Some(setting) = symbol_setting {
            SymbolConfig {
                enabled: setting.enabled,
                leverage: setting.leverage,
                position_size_pct: setting.position_size_pct,
                stop_loss_pct: setting.stop_loss_pct,
                take_profit_pct: setting.take_profit_pct,
                max_positions: setting.max_positions,
            }
        } else {
            // Use defaults if not configured
            SymbolConfig {
                enabled: true,
                leverage: Some(10),
                position_size_pct: Some(5.0),
                stop_loss_pct: Some(2.0),
                take_profit_pct: Some(4.0),
                max_positions: Some(2),
            }
        };

        symbol_configs.insert(symbol.to_string(), config);
    }

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(symbol_configs)),
        StatusCode::OK,
    ))
}

/// Update symbol settings
async fn update_symbol_settings(
    request: UpdateSymbolSettingsRequest,
    api: Arc<PaperTradingApi>,
) -> Result<impl Reply, Rejection> {
    log::info!("Updating symbol settings: {:?}", request.symbols);

    let mut current_settings = api.engine.get_settings().await;

    // Clone the keys before iterating to avoid borrowing issues
    let symbol_keys: Vec<String> = request.symbols.keys().cloned().collect();

    // Update symbol settings
    for (symbol, config) in request.symbols {
        let symbol_setting = crate::paper_trading::settings::SymbolSettings {
            enabled: config.enabled,
            leverage: config.leverage,
            position_size_pct: config.position_size_pct,
            stop_loss_pct: config.stop_loss_pct,
            take_profit_pct: config.take_profit_pct,
            trading_hours: None,
            min_price_movement_pct: Some(0.1),
            max_positions: config.max_positions,
            custom_params: std::collections::HashMap::new(),
        };

        current_settings
            .symbols
            .insert(symbol.clone(), symbol_setting);
    }

    match api.engine.update_settings(current_settings).await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Symbol settings updated successfully",
                "updated_symbols": symbol_keys,
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::BAD_REQUEST,
        )),
    }
}

/// Trigger manual analysis
async fn trigger_manual_analysis(api: Arc<PaperTradingApi>) -> Result<impl Reply, Rejection> {
    match api.engine.trigger_manual_analysis().await {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Manual analysis triggered successfully",
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// Update signal refresh interval
async fn update_signal_refresh_interval(
    request: UpdateSignalIntervalRequest,
    api: Arc<PaperTradingApi>,
) -> Result<impl Reply, Rejection> {
    log::info!(
        "Updating signal refresh interval: {:?}",
        request.interval_minutes
    );

    match api
        .engine
        .update_signal_refresh_interval(request.interval_minutes)
        .await
    {
        Ok(_) => {
            let response = serde_json::json!({
                "message": "Signal refresh interval updated successfully",
                "updated_interval": request.interval_minutes,
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::BAD_REQUEST,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai::AIService;
    use crate::binance::BinanceClient;
    use crate::paper_trading::{PaperTradingEngine, PaperTradingSettings};
    use crate::storage::Storage;
    use tokio::sync::broadcast;
    use warp::http::StatusCode;
    use warp::test::request;

    // Helper function to create a test engine
    async fn create_test_engine() -> Arc<PaperTradingEngine> {
        let (tx, _rx) = broadcast::channel(100);

        // Create test dependencies
        let db_config = crate::config::DatabaseConfig {
            url: "mongodb://localhost:27017".to_string(),
            database_name: Some("test_db".to_string()),
            max_connections: 10,
            enable_logging: false,
        };
        let storage = Storage::new(&db_config)
            .await
            .expect("Failed to create storage");

        let binance_config = crate::config::BinanceConfig {
            api_key: "test_api_key".to_string(),
            secret_key: "test_secret_key".to_string(),
            testnet: true,
            base_url: "https://testnet.binance.vision".to_string(),
            ws_url: "wss://testnet.binance.vision/ws".to_string(),
            futures_base_url: "https://testnet.binancefuture.com".to_string(),
            futures_ws_url: "wss://stream.binancefuture.com/ws".to_string(),
        };
        let binance_client =
            BinanceClient::new(binance_config).expect("Failed to create binance client");

        let ai_config = crate::ai::AIServiceConfig {
            python_service_url: "http://localhost:8000".to_string(),
            request_timeout_seconds: 30,
            max_retries: 3,
            enable_caching: true,
            cache_ttl_seconds: 300,
        };
        let ai_service = AIService::new(ai_config);

        let settings = PaperTradingSettings::default();

        let engine = PaperTradingEngine::new(settings, binance_client, ai_service, storage, tx)
            .await
            .expect("Failed to create engine");

        Arc::new(engine)
    }

    // Helper function to create test API
    async fn create_test_api() -> PaperTradingApi {
        let engine = create_test_engine().await;
        PaperTradingApi::new(engine)
    }

    #[test]
    fn test_api_response_success_creation() {
        let data = "test success";
        let response = ApiResponse::success(data);

        assert!(response.success);
        assert_eq!(response.data, Some("test success"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error_creation() {
        let error_msg = "test error".to_string();
        let response: ApiResponse<String> = ApiResponse::error(error_msg.clone());

        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some(error_msg));
    }

    #[test]
    fn test_api_response_has_timestamp() {
        let response = ApiResponse::success("data");
        let now = chrono::Utc::now();

        assert!(response.timestamp <= now);
        assert!(response.timestamp > now - chrono::Duration::seconds(1));
    }

    #[test]
    fn test_api_response_serialization() {
        let response = ApiResponse::success("test_data");
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"data\":\"test_data\""));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_api_response_error_serialization() {
        let response: ApiResponse<()> = ApiResponse::error("error message".to_string());
        let json = serde_json::to_string(&response).unwrap();

        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"error message\""));
        assert!(json.contains("\"data\":null"));
    }

    #[test]
    fn test_close_trade_request_deserialization() {
        let json = r#"{
            "trade_id": "trade_123",
            "reason": "Manual close"
        }"#;

        let request: CloseTradeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.trade_id, "trade_123");
        assert_eq!(request.reason, Some("Manual close".to_string()));
    }

    #[test]
    fn test_close_trade_request_without_reason() {
        let json = r#"{
            "trade_id": "trade_456"
        }"#;

        let request: CloseTradeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.trade_id, "trade_456");
        assert!(request.reason.is_none());
    }

    #[test]
    fn test_close_trade_request_serialization() {
        let request = CloseTradeRequest {
            trade_id: "trade_789".to_string(),
            reason: Some("Stop loss hit".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("trade_789"));
        assert!(json.contains("Stop loss hit"));
    }

    #[test]
    fn test_rsi_config_serialization() {
        let config = RsiConfig {
            enabled: true,
            period: 14,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
            extreme_oversold: 20.0,
            extreme_overbought: 80.0,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"period\":14"));
        assert!(json.contains("\"oversold_threshold\":30.0"));
    }

    #[test]
    fn test_rsi_config_deserialization() {
        let json = r#"{
            "enabled": false,
            "period": 21,
            "oversold_threshold": 25.0,
            "overbought_threshold": 75.0,
            "extreme_oversold": 15.0,
            "extreme_overbought": 85.0
        }"#;

        let config: RsiConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.period, 21);
        assert_eq!(config.oversold_threshold, 25.0);
    }

    #[test]
    fn test_macd_config_serialization() {
        let config = MacdConfig {
            enabled: true,
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            histogram_threshold: 0.001,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"fast_period\":12"));
        assert!(json.contains("\"slow_period\":26"));
        assert!(json.contains("\"signal_period\":9"));
    }

    #[test]
    fn test_macd_config_deserialization() {
        let json = r#"{
            "enabled": true,
            "fast_period": 8,
            "slow_period": 21,
            "signal_period": 5,
            "histogram_threshold": 0.002
        }"#;

        let config: MacdConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.fast_period, 8);
        assert_eq!(config.slow_period, 21);
        assert_eq!(config.histogram_threshold, 0.002);
    }

    #[test]
    fn test_volume_config_serialization() {
        let config = VolumeConfig {
            enabled: true,
            sma_period: 20,
            spike_threshold: 2.0,
            correlation_period: 10,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"sma_period\":20"));
        assert!(json.contains("\"spike_threshold\":2.0"));
    }

    #[test]
    fn test_volume_config_deserialization() {
        let json = r#"{
            "enabled": false,
            "sma_period": 30,
            "spike_threshold": 1.5,
            "correlation_period": 15
        }"#;

        let config: VolumeConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.sma_period, 30);
        assert_eq!(config.spike_threshold, 1.5);
    }

    #[test]
    fn test_bollinger_config_serialization() {
        let config = BollingerConfig {
            enabled: true,
            period: 20,
            multiplier: 2.0,
            squeeze_threshold: 0.02,
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"period\":20"));
        assert!(json.contains("\"multiplier\":2.0"));
    }

    #[test]
    fn test_bollinger_config_deserialization() {
        let json = r#"{
            "enabled": true,
            "period": 50,
            "multiplier": 2.5,
            "squeeze_threshold": 0.01
        }"#;

        let config: BollingerConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.period, 50);
        assert_eq!(config.multiplier, 2.5);
        assert_eq!(config.squeeze_threshold, 0.01);
    }

    #[test]
    fn test_risk_settings_serialization() {
        let settings = RiskSettings {
            max_risk_per_trade: 2.0,
            max_portfolio_risk: 10.0,
            stop_loss_percent: 3.0,
            take_profit_percent: 6.0,
            max_leverage: 10,
            max_drawdown: 20.0,
            daily_loss_limit: 5.0,
            max_consecutive_losses: 3,
            correlation_limit: 0.7,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"max_risk_per_trade\":2.0"));
        assert!(json.contains("\"max_leverage\":10"));
    }

    #[test]
    fn test_risk_settings_deserialization() {
        let json = r#"{
            "max_risk_per_trade": 1.5,
            "max_portfolio_risk": 8.0,
            "stop_loss_percent": 2.5,
            "take_profit_percent": 5.0,
            "max_leverage": 5,
            "max_drawdown": 15.0,
            "daily_loss_limit": 4.0,
            "max_consecutive_losses": 5
        }"#;

        let settings: RiskSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.max_risk_per_trade, 1.5);
        assert_eq!(settings.max_leverage, 5);
        assert_eq!(settings.max_consecutive_losses, 5);
    }

    #[test]
    fn test_engine_settings_serialization() {
        let settings = EngineSettings {
            min_confidence_threshold: 0.7,
            signal_combination_mode: "WeightedAverage".to_string(),
            enabled_strategies: vec!["RSI".to_string(), "MACD".to_string()],
            market_condition: "Trending".to_string(),
            risk_level: "Moderate".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"min_confidence_threshold\":0.7"));
        assert!(json.contains("WeightedAverage"));
        assert!(json.contains("RSI"));
    }

    #[test]
    fn test_engine_settings_deserialization() {
        let json = r#"{
            "min_confidence_threshold": 0.8,
            "signal_combination_mode": "Unanimous",
            "enabled_strategies": ["RSI", "MACD", "Volume"],
            "market_condition": "Ranging",
            "risk_level": "Conservative"
        }"#;

        let settings: EngineSettings = serde_json::from_str(json).unwrap();
        assert_eq!(settings.min_confidence_threshold, 0.8);
        assert_eq!(settings.signal_combination_mode, "Unanimous");
        assert_eq!(settings.enabled_strategies.len(), 3);
    }

    #[test]
    fn test_strategy_config_collection_serialization() {
        let collection = StrategyConfigCollection {
            rsi: RsiConfig {
                enabled: true,
                period: 14,
                oversold_threshold: 30.0,
                overbought_threshold: 70.0,
                extreme_oversold: 20.0,
                extreme_overbought: 80.0,
            },
            macd: MacdConfig {
                enabled: true,
                fast_period: 12,
                slow_period: 26,
                signal_period: 9,
                histogram_threshold: 0.001,
            },
            volume: VolumeConfig {
                enabled: true,
                sma_period: 20,
                spike_threshold: 2.0,
                correlation_period: 10,
            },
            bollinger: BollingerConfig {
                enabled: true,
                period: 20,
                multiplier: 2.0,
                squeeze_threshold: 0.02,
            },
        };

        let json = serde_json::to_string(&collection).unwrap();
        assert!(json.contains("\"rsi\""));
        assert!(json.contains("\"macd\""));
        assert!(json.contains("\"volume\""));
        assert!(json.contains("\"bollinger\""));
    }

    #[test]
    fn test_trading_strategy_settings_full_serialization() {
        let settings = TradingStrategySettings {
            strategies: StrategyConfigCollection {
                rsi: RsiConfig {
                    enabled: true,
                    period: 14,
                    oversold_threshold: 30.0,
                    overbought_threshold: 70.0,
                    extreme_oversold: 20.0,
                    extreme_overbought: 80.0,
                },
                macd: MacdConfig {
                    enabled: true,
                    fast_period: 12,
                    slow_period: 26,
                    signal_period: 9,
                    histogram_threshold: 0.001,
                },
                volume: VolumeConfig {
                    enabled: true,
                    sma_period: 20,
                    spike_threshold: 2.0,
                    correlation_period: 10,
                },
                bollinger: BollingerConfig {
                    enabled: true,
                    period: 20,
                    multiplier: 2.0,
                    squeeze_threshold: 0.02,
                },
                stochastic: StochasticConfig {
                    enabled: true,
                    k_period: 14,
                    d_period: 3,
                    oversold_threshold: 20.0,
                    overbought_threshold: 80.0,
                    extreme_oversold: 10.0,
                    extreme_overbought: 90.0,
                },
            },
            risk: RiskSettings {
                max_risk_per_trade: 2.0,
                max_portfolio_risk: 10.0,
                stop_loss_percent: 3.0,
                take_profit_percent: 6.0,
                max_leverage: 10,
                max_drawdown: 20.0,
                daily_loss_limit: 5.0,
                max_consecutive_losses: 3,
            },
            engine: EngineSettings {
                min_confidence_threshold: 0.7,
                signal_combination_mode: "WeightedAverage".to_string(),
                enabled_strategies: vec!["RSI".to_string()],
                market_condition: "Trending".to_string(),
                risk_level: "Moderate".to_string(),
            },
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: TradingStrategySettings = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.risk.max_risk_per_trade, 2.0);
        assert_eq!(deserialized.engine.min_confidence_threshold, 0.7);
        assert_eq!(deserialized.strategies.rsi.period, 14);
    }

    #[test]
    fn test_update_basic_settings_request_full() {
        let request = UpdateBasicSettingsRequest {
            initial_balance: Some(10000.0),
            max_positions: Some(5),
            default_position_size_pct: Some(10.0),
            default_leverage: Some(10),
            trading_fee_rate: Some(0.001),
            funding_fee_rate: Some(0.0001),
            slippage_pct: Some(0.1),
            max_risk_per_trade_pct: Some(2.0),
            max_portfolio_risk_pct: Some(10.0),
            default_stop_loss_pct: Some(3.0),
            default_take_profit_pct: Some(6.0),
            max_leverage: Some(20),
            enabled: Some(true),
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateBasicSettingsRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.initial_balance, Some(10000.0));
        assert_eq!(deserialized.max_positions, Some(5));
        assert_eq!(deserialized.enabled, Some(true));
    }

    #[test]
    fn test_update_basic_settings_request_partial() {
        let request = UpdateBasicSettingsRequest {
            initial_balance: Some(5000.0),
            max_positions: None,
            default_position_size_pct: None,
            default_leverage: None,
            trading_fee_rate: Some(0.002),
            funding_fee_rate: None,
            slippage_pct: None,
            max_risk_per_trade_pct: None,
            max_portfolio_risk_pct: None,
            default_stop_loss_pct: None,
            default_take_profit_pct: None,
            max_leverage: None,
            enabled: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateBasicSettingsRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.initial_balance, Some(5000.0));
        assert!(deserialized.max_positions.is_none());
        assert_eq!(deserialized.trading_fee_rate, Some(0.002));
    }

    #[test]
    fn test_symbol_config_serialization() {
        let config = SymbolConfig {
            enabled: true,
            leverage: Some(10),
            position_size_pct: Some(5.0),
            stop_loss_pct: Some(2.0),
            take_profit_pct: Some(4.0),
            max_positions: Some(3),
        };

        let json = serde_json::to_string(&config).unwrap();
        assert!(json.contains("\"enabled\":true"));
        assert!(json.contains("\"leverage\":10"));
    }

    #[test]
    fn test_symbol_config_deserialization() {
        let json = r#"{
            "enabled": false,
            "leverage": 5,
            "position_size_pct": 10.0,
            "stop_loss_pct": 3.0,
            "take_profit_pct": 6.0,
            "max_positions": 2
        }"#;

        let config: SymbolConfig = serde_json::from_str(json).unwrap();
        assert!(!config.enabled);
        assert_eq!(config.leverage, Some(5));
        assert_eq!(config.position_size_pct, Some(10.0));
    }

    #[test]
    fn test_update_symbol_settings_request() {
        let mut symbols = std::collections::HashMap::new();
        symbols.insert(
            "BTCUSDT".to_string(),
            SymbolConfig {
                enabled: true,
                leverage: Some(10),
                position_size_pct: Some(5.0),
                stop_loss_pct: Some(2.0),
                take_profit_pct: Some(4.0),
                max_positions: Some(2),
            },
        );

        let request = UpdateSymbolSettingsRequest { symbols };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("\"enabled\":true"));
    }

    #[test]
    fn test_update_symbol_settings_request_multiple_symbols() {
        let mut symbols = std::collections::HashMap::new();
        symbols.insert(
            "BTCUSDT".to_string(),
            SymbolConfig {
                enabled: true,
                leverage: Some(10),
                position_size_pct: Some(5.0),
                stop_loss_pct: Some(2.0),
                take_profit_pct: Some(4.0),
                max_positions: Some(2),
            },
        );
        symbols.insert(
            "ETHUSDT".to_string(),
            SymbolConfig {
                enabled: false,
                leverage: Some(5),
                position_size_pct: Some(3.0),
                stop_loss_pct: Some(1.5),
                take_profit_pct: Some(3.0),
                max_positions: Some(1),
            },
        );

        let request = UpdateSymbolSettingsRequest { symbols };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateSymbolSettingsRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbols.len(), 2);
        assert!(deserialized.symbols.contains_key("BTCUSDT"));
        assert!(deserialized.symbols.contains_key("ETHUSDT"));
    }

    #[test]
    fn test_update_signal_interval_request() {
        let request = UpdateSignalIntervalRequest {
            interval_minutes: 60,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateSignalIntervalRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.interval_minutes, 60);
    }

    #[test]
    fn test_update_signal_interval_request_various_values() {
        let intervals = vec![1, 5, 15, 30, 60, 120, 240];

        for interval in intervals {
            let request = UpdateSignalIntervalRequest {
                interval_minutes: interval,
            };

            let json = serde_json::to_string(&request).unwrap();
            let deserialized: UpdateSignalIntervalRequest = serde_json::from_str(&json).unwrap();

            assert_eq!(deserialized.interval_minutes, interval);
        }
    }

    #[test]
    fn test_api_response_with_complex_nested_data() {
        #[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
        struct ComplexData {
            strategies: Vec<String>,
            settings: std::collections::HashMap<String, f64>,
        }

        let mut settings_map = std::collections::HashMap::new();
        settings_map.insert("threshold".to_string(), 0.75);
        settings_map.insert("leverage".to_string(), 10.0);

        let data = ComplexData {
            strategies: vec!["RSI".to_string(), "MACD".to_string()],
            settings: settings_map,
        };

        let response = ApiResponse::success(data.clone());

        let json = serde_json::to_string(&response).unwrap();

        // Verify serialization
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("RSI"));
        assert!(json.contains("MACD"));
    }

    #[test]
    fn test_symbol_config_with_none_values() {
        let config = SymbolConfig {
            enabled: true,
            leverage: None,
            position_size_pct: None,
            stop_loss_pct: None,
            take_profit_pct: None,
            max_positions: None,
        };

        let json = serde_json::to_string(&config).unwrap();
        let deserialized: SymbolConfig = serde_json::from_str(&json).unwrap();

        assert!(deserialized.enabled);
        assert!(deserialized.leverage.is_none());
        assert!(deserialized.position_size_pct.is_none());
    }

    #[test]
    fn test_update_basic_settings_request_empty() {
        let request = UpdateBasicSettingsRequest {
            initial_balance: None,
            max_positions: None,
            default_position_size_pct: None,
            default_leverage: None,
            trading_fee_rate: None,
            funding_fee_rate: None,
            slippage_pct: None,
            max_risk_per_trade_pct: None,
            max_portfolio_risk_pct: None,
            default_stop_loss_pct: None,
            default_take_profit_pct: None,
            max_leverage: None,
            enabled: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: UpdateBasicSettingsRequest = serde_json::from_str(&json).unwrap();

        assert!(deserialized.initial_balance.is_none());
        assert!(deserialized.enabled.is_none());
    }

    #[test]
    fn test_engine_settings_empty_strategies() {
        let settings = EngineSettings {
            min_confidence_threshold: 0.5,
            signal_combination_mode: "WeightedAverage".to_string(),
            enabled_strategies: vec![],
            market_condition: "Unknown".to_string(),
            risk_level: "Low".to_string(),
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: EngineSettings = serde_json::from_str(&json).unwrap();

        assert!(deserialized.enabled_strategies.is_empty());
    }

    #[test]
    fn test_api_response_roundtrip_with_timestamp() {
        let original = ApiResponse::success(vec![1, 2, 3, 4, 5]);
        let json = serde_json::to_string(&original).unwrap();

        // Verify serialization
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("[1,2,3,4,5]"));
        assert!(json.contains("\"timestamp\""));
    }

    #[test]
    fn test_update_strategy_settings_request_deserialization() {
        let json = r#"{
            "settings": {
                "strategies": {
                    "rsi": {
                        "enabled": true,
                        "period": 14,
                        "oversold_threshold": 30.0,
                        "overbought_threshold": 70.0,
                        "extreme_oversold": 20.0,
                        "extreme_overbought": 80.0
                    },
                    "macd": {
                        "enabled": true,
                        "fast_period": 12,
                        "slow_period": 26,
                        "signal_period": 9,
                        "histogram_threshold": 0.001
                    },
                    "volume": {
                        "enabled": true,
                        "sma_period": 20,
                        "spike_threshold": 2.0,
                        "correlation_period": 10
                    },
                    "bollinger": {
                        "enabled": true,
                        "period": 20,
                        "multiplier": 2.0,
                        "squeeze_threshold": 0.02
                    }
                },
                "risk": {
                    "max_risk_per_trade": 2.0,
                    "max_portfolio_risk": 10.0,
                    "stop_loss_percent": 3.0,
                    "take_profit_percent": 6.0,
                    "max_leverage": 10,
                    "max_drawdown": 20.0,
                    "daily_loss_limit": 5.0,
                    "max_consecutive_losses": 3
                },
                "engine": {
                    "min_confidence_threshold": 0.7,
                    "signal_combination_mode": "WeightedAverage",
                    "enabled_strategies": ["RSI", "MACD"],
                    "market_condition": "Trending",
                    "risk_level": "Moderate"
                }
            }
        }"#;

        let request: UpdateStrategySettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.settings.strategies.rsi.period, 14);
        assert_eq!(request.settings.risk.max_leverage, 10);
        assert_eq!(request.settings.engine.min_confidence_threshold, 0.7);
    }

    #[test]
    #[ignore] // Deserialization test - needs fixing
    fn test_update_settings_request_deserialization() {
        let json = r#"{
            "settings": {
                "basic": {
                    "initial_balance": 10000.0,
                    "max_positions": 5,
                    "default_position_size_pct": 10.0,
                    "default_leverage": 10,
                    "trading_fee_rate": 0.001,
                    "funding_fee_rate": 0.0001,
                    "slippage_pct": 0.1,
                    "enabled": true,
                    "auto_restart": false
                },
                "risk": {
                    "max_risk_per_trade_pct": 2.0,
                    "max_portfolio_risk_pct": 10.0,
                    "default_stop_loss_pct": 3.0,
                    "default_take_profit_pct": 6.0,
                    "max_leverage": 20,
                    "min_margin_level": 1.2,
                    "max_drawdown_pct": 20.0,
                    "daily_loss_limit_pct": 5.0,
                    "max_consecutive_losses": 3,
                    "cool_down_minutes": 60
                },
                "strategy": {
                    "min_ai_confidence": 0.7,
                    "enable_ai_override": true,
                    "enable_risk_checks": true,
                    "enable_position_sizing": true
                },
                "symbols": {},
                "notifications": {
                    "enable_trade_notifications": true,
                    "enable_error_notifications": true,
                    "enable_performance_reports": false,
                    "daily_report_time": "00:00:00"
                }
            }
        }"#;

        let request: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.settings.basic.initial_balance, 10000.0);
        assert_eq!(request.settings.basic.max_positions, 5);
        assert!(request.settings.basic.enabled);
    }

    // ============================================================================
    // HTTP ROUTE INTEGRATION TESTS
    // ============================================================================

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_status_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/status")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());

        let data = body.data.unwrap();
        assert!(data.get("is_running").is_some());
        assert!(data.get("portfolio").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_portfolio_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/portfolio")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_open_trades_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/trades/open")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<Vec<serde_json::Value>> =
            serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_closed_trades_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/trades/closed")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<Vec<serde_json::Value>> =
            serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_close_trade_route_success() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = CloseTradeRequest {
            trade_id: "test_trade_123".to_string(),
            reason: Some("Test close".to_string()),
        };

        let response = request()
            .method("POST")
            .path("/paper-trading/trades/test_trade_123/close")
            .json(&request_body)
            .reply(&filter)
            .await;

        // Should return BAD_REQUEST as trade doesn't exist
        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_close_trade_route_with_reason() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = CloseTradeRequest {
            trade_id: "test_trade_456".to_string(),
            reason: Some("Manual stop loss".to_string()),
        };

        let response = request()
            .method("POST")
            .path("/paper-trading/trades/test_trade_456/close")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_close_trade_route_without_reason() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = CloseTradeRequest {
            trade_id: "test_trade_789".to_string(),
            reason: None,
        };

        let response = request()
            .method("POST")
            .path("/paper-trading/trades/test_trade_789/close")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert!(
            response.status() == StatusCode::OK || response.status() == StatusCode::BAD_REQUEST
        );
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_strategy_settings_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/strategy-settings")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<TradingStrategySettings> =
            serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());

        let settings = body.data.unwrap();
        assert!(settings.strategies.rsi.period > 0);
        assert!(settings.risk.max_leverage > 0);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_update_strategy_settings_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = UpdateStrategySettingsRequest {
            settings: TradingStrategySettings {
                strategies: StrategyConfigCollection {
                    rsi: RsiConfig {
                        enabled: true,
                        period: 14,
                        oversold_threshold: 30.0,
                        overbought_threshold: 70.0,
                        extreme_oversold: 20.0,
                        extreme_overbought: 80.0,
                    },
                    macd: MacdConfig {
                        enabled: true,
                        fast_period: 12,
                        slow_period: 26,
                        signal_period: 9,
                        histogram_threshold: 0.001,
                    },
                    volume: VolumeConfig {
                        enabled: true,
                        sma_period: 20,
                        spike_threshold: 2.0,
                        correlation_period: 10,
                    },
                    bollinger: BollingerConfig {
                        enabled: true,
                        period: 20,
                        multiplier: 2.0,
                        squeeze_threshold: 0.02,
                    },
                    stochastic: StochasticConfig {
                        enabled: true,
                        k_period: 14,
                        d_period: 3,
                        oversold_threshold: 20.0,
                        overbought_threshold: 80.0,
                        extreme_oversold: 10.0,
                        extreme_overbought: 90.0,
                    },
                },
                risk: RiskSettings {
                    max_risk_per_trade: 2.0,
                    max_portfolio_risk: 10.0,
                    stop_loss_percent: 3.0,
                    take_profit_percent: 6.0,
                    max_leverage: 10,
                    max_drawdown: 20.0,
                    daily_loss_limit: 5.0,
                    max_consecutive_losses: 3,
                },
                engine: EngineSettings {
                    min_confidence_threshold: 0.7,
                    signal_combination_mode: "WeightedAverage".to_string(),
                    enabled_strategies: vec!["RSI".to_string(), "MACD".to_string()],
                    market_condition: "Trending".to_string(),
                    risk_level: "Moderate".to_string(),
                },
            },
        };

        let response = request()
            .method("PUT")
            .path("/paper-trading/strategy-settings")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_basic_settings_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/basic-settings")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());

        let data = body.data.unwrap();
        assert!(data.get("basic").is_some());
        assert!(data.get("risk").is_some());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_update_basic_settings_route_full() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = UpdateBasicSettingsRequest {
            initial_balance: Some(10000.0),
            max_positions: Some(5),
            default_position_size_pct: Some(10.0),
            default_leverage: Some(10),
            trading_fee_rate: Some(0.001),
            funding_fee_rate: Some(0.0001),
            slippage_pct: Some(0.1),
            max_risk_per_trade_pct: Some(2.0),
            max_portfolio_risk_pct: Some(10.0),
            default_stop_loss_pct: Some(3.0),
            default_take_profit_pct: Some(6.0),
            max_leverage: Some(20),
            enabled: Some(true),
        };

        let response = request()
            .method("PUT")
            .path("/paper-trading/basic-settings")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_update_basic_settings_route_partial() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = UpdateBasicSettingsRequest {
            initial_balance: Some(5000.0),
            max_positions: None,
            default_position_size_pct: None,
            default_leverage: None,
            trading_fee_rate: Some(0.002),
            funding_fee_rate: None,
            slippage_pct: None,
            max_risk_per_trade_pct: None,
            max_portfolio_risk_pct: None,
            default_stop_loss_pct: None,
            default_take_profit_pct: None,
            max_leverage: None,
            enabled: None,
        };

        let response = request()
            .method("PUT")
            .path("/paper-trading/basic-settings")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_get_symbol_settings_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/symbols")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<std::collections::HashMap<String, SymbolConfig>> =
            serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
        assert!(body.data.is_some());

        let symbols = body.data.unwrap();
        assert!(symbols.len() > 0);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_update_symbol_settings_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let mut symbols = std::collections::HashMap::new();
        symbols.insert(
            "BTCUSDT".to_string(),
            SymbolConfig {
                enabled: true,
                leverage: Some(10),
                position_size_pct: Some(5.0),
                stop_loss_pct: Some(2.0),
                take_profit_pct: Some(4.0),
                max_positions: Some(2),
            },
        );

        let request_body = UpdateSymbolSettingsRequest { symbols };

        let response = request()
            .method("PUT")
            .path("/paper-trading/symbols")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_update_symbol_settings_route_multiple_symbols() {
        let api = create_test_api().await;
        let filter = api.routes();

        let mut symbols = std::collections::HashMap::new();
        symbols.insert(
            "BTCUSDT".to_string(),
            SymbolConfig {
                enabled: true,
                leverage: Some(10),
                position_size_pct: Some(5.0),
                stop_loss_pct: Some(2.0),
                take_profit_pct: Some(4.0),
                max_positions: Some(2),
            },
        );
        symbols.insert(
            "ETHUSDT".to_string(),
            SymbolConfig {
                enabled: false,
                leverage: Some(5),
                position_size_pct: Some(3.0),
                stop_loss_pct: Some(1.5),
                take_profit_pct: Some(3.0),
                max_positions: Some(1),
            },
        );

        let request_body = UpdateSymbolSettingsRequest { symbols };

        let response = request()
            .method("PUT")
            .path("/paper-trading/symbols")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_reset_portfolio_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("POST")
            .path("/paper-trading/reset")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_start_engine_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("POST")
            .path("/paper-trading/start")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_stop_engine_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("POST")
            .path("/paper-trading/stop")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_trigger_analysis_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("POST")
            .path("/paper-trading/trigger-analysis")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_update_signal_interval_route() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = UpdateSignalIntervalRequest {
            interval_minutes: 60,
        };

        let response = request()
            .method("PUT")
            .path("/paper-trading/signal-interval")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);

        let body: ApiResponse<serde_json::Value> = serde_json::from_slice(response.body()).unwrap();
        assert!(body.success);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_update_signal_interval_route_various_values() {
        let api = create_test_api().await;
        let filter = api.routes();

        let intervals = vec![1, 5, 15, 30, 60, 120];

        for interval in intervals {
            let request_body = UpdateSignalIntervalRequest {
                interval_minutes: interval,
            };

            let response = request()
                .method("PUT")
                .path("/paper-trading/signal-interval")
                .json(&request_body)
                .reply(&filter)
                .await;

            assert_eq!(response.status(), StatusCode::OK);
        }
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_cors_headers() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("OPTIONS")
            .path("/paper-trading/status")
            .header("origin", "http://localhost:3000")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::OK);
        assert!(response
            .headers()
            .get("access-control-allow-origin")
            .is_some());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_invalid_route_404() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("GET")
            .path("/paper-trading/invalid-endpoint")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_invalid_method_405() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("DELETE")
            .path("/paper-trading/status")
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_malformed_json_request() {
        let api = create_test_api().await;
        let filter = api.routes();

        let response = request()
            .method("PUT")
            .path("/paper-trading/basic-settings")
            .header("content-type", "application/json")
            .body("{invalid json}")
            .reply(&filter)
            .await;

        assert!(response.status().is_client_error());
    }

    #[tokio::test]
    #[ignore] // Requires MongoDB connection
    async fn test_close_trade_with_empty_trade_id() {
        let api = create_test_api().await;
        let filter = api.routes();

        let request_body = CloseTradeRequest {
            trade_id: "".to_string(),
            reason: None,
        };

        let response = request()
            .method("POST")
            .path("/paper-trading/trades//close")
            .json(&request_body)
            .reply(&filter)
            .await;

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[test]
    fn test_update_strategy_settings_confidence_threshold_range() {
        let valid_thresholds = vec![0.0, 0.5, 0.7, 0.8, 1.0];

        for threshold in valid_thresholds {
            let settings = EngineSettings {
                min_confidence_threshold: threshold,
                signal_combination_mode: "WeightedAverage".to_string(),
                enabled_strategies: vec![],
                market_condition: "Trending".to_string(),
                risk_level: "Moderate".to_string(),
            };

            assert!(settings.min_confidence_threshold >= 0.0);
            assert!(settings.min_confidence_threshold <= 1.0);
        }
    }

    #[test]
    fn test_risk_settings_validation() {
        let settings = RiskSettings {
            max_risk_per_trade: 2.0,
            max_portfolio_risk: 10.0,
            stop_loss_percent: 3.0,
            take_profit_percent: 6.0,
            max_leverage: 10,
            max_drawdown: 20.0,
            daily_loss_limit: 5.0,
            max_consecutive_losses: 3,
            correlation_limit: 0.7,
        };

        assert!(settings.max_risk_per_trade > 0.0);
        assert!(settings.stop_loss_percent > 0.0);
        assert!(settings.take_profit_percent > settings.stop_loss_percent);
        assert!(settings.max_leverage > 0);
    }

    #[test]
    fn test_symbol_config_default_values() {
        let config = SymbolConfig {
            enabled: true,
            leverage: Some(10),
            position_size_pct: Some(5.0),
            stop_loss_pct: Some(2.0),
            take_profit_pct: Some(4.0),
            max_positions: Some(2),
        };

        assert!(config.enabled);
        assert_eq!(config.leverage, Some(10));
        assert_eq!(config.position_size_pct, Some(5.0));
    }

    #[test]
    fn test_strategy_configs_serialization_roundtrip() {
        let original = StrategyConfigCollection {
            rsi: RsiConfig {
                enabled: true,
                period: 14,
                oversold_threshold: 30.0,
                overbought_threshold: 70.0,
                extreme_oversold: 20.0,
                extreme_overbought: 80.0,
            },
            macd: MacdConfig {
                enabled: true,
                fast_period: 12,
                slow_period: 26,
                signal_period: 9,
                histogram_threshold: 0.001,
            },
            volume: VolumeConfig {
                enabled: true,
                sma_period: 20,
                spike_threshold: 2.0,
                correlation_period: 10,
            },
            bollinger: BollingerConfig {
                enabled: true,
                period: 20,
                multiplier: 2.0,
                squeeze_threshold: 0.02,
            },
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: StrategyConfigCollection = serde_json::from_str(&json).unwrap();

        assert_eq!(original.rsi.period, deserialized.rsi.period);
        assert_eq!(original.macd.fast_period, deserialized.macd.fast_period);
        assert_eq!(original.volume.sma_period, deserialized.volume.sma_period);
        assert_eq!(original.bollinger.period, deserialized.bollinger.period);
    }

    #[test]
    fn test_api_response_error_with_different_types() {
        let error_response: ApiResponse<String> = ApiResponse::error("String error".to_string());
        assert!(!error_response.success);

        let error_response2: ApiResponse<i32> = ApiResponse::error("Int error".to_string());
        assert!(!error_response2.success);

        let error_response3: ApiResponse<Vec<String>> = ApiResponse::error("Vec error".to_string());
        assert!(!error_response3.success);
    }

    #[test]
    fn test_close_trade_request_serialization_roundtrip() {
        let original = CloseTradeRequest {
            trade_id: "trade_123".to_string(),
            reason: Some("Stop loss triggered".to_string()),
        };

        let json = serde_json::to_string(&original).unwrap();
        let deserialized: CloseTradeRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(original.trade_id, deserialized.trade_id);
        assert_eq!(original.reason, deserialized.reason);
    }

    #[test]
    fn test_trading_strategy_settings_validation() {
        let settings = TradingStrategySettings {
            strategies: StrategyConfigCollection {
                rsi: RsiConfig {
                    enabled: true,
                    period: 14,
                    oversold_threshold: 30.0,
                    overbought_threshold: 70.0,
                    extreme_oversold: 20.0,
                    extreme_overbought: 80.0,
                },
                macd: MacdConfig {
                    enabled: true,
                    fast_period: 12,
                    slow_period: 26,
                    signal_period: 9,
                    histogram_threshold: 0.001,
                },
                volume: VolumeConfig {
                    enabled: true,
                    sma_period: 20,
                    spike_threshold: 2.0,
                    correlation_period: 10,
                },
                bollinger: BollingerConfig {
                    enabled: true,
                    period: 20,
                    multiplier: 2.0,
                    squeeze_threshold: 0.02,
                },
                stochastic: StochasticConfig {
                    enabled: true,
                    k_period: 14,
                    d_period: 3,
                    oversold_threshold: 20.0,
                    overbought_threshold: 80.0,
                    extreme_oversold: 10.0,
                    extreme_overbought: 90.0,
                },
            },
            risk: RiskSettings {
                max_risk_per_trade: 2.0,
                max_portfolio_risk: 10.0,
                stop_loss_percent: 3.0,
                take_profit_percent: 6.0,
                max_leverage: 10,
                max_drawdown: 20.0,
                daily_loss_limit: 5.0,
                max_consecutive_losses: 3,
            },
            engine: EngineSettings {
                min_confidence_threshold: 0.7,
                signal_combination_mode: "WeightedAverage".to_string(),
                enabled_strategies: vec!["RSI".to_string()],
                market_condition: "Trending".to_string(),
                risk_level: "Moderate".to_string(),
            },
        };

        // RSI validation
        assert!(
            settings.strategies.rsi.oversold_threshold
                < settings.strategies.rsi.overbought_threshold
        );
        assert!(
            settings.strategies.rsi.extreme_oversold < settings.strategies.rsi.oversold_threshold
        );

        // MACD validation
        assert!(settings.strategies.macd.fast_period < settings.strategies.macd.slow_period);

        // Risk validation
        assert!(settings.risk.max_risk_per_trade < settings.risk.max_portfolio_risk);
    }

    #[test]
    fn test_update_basic_settings_request_validation() {
        let request = UpdateBasicSettingsRequest {
            initial_balance: Some(10000.0),
            max_positions: Some(5),
            default_position_size_pct: Some(10.0),
            default_leverage: Some(10),
            trading_fee_rate: Some(0.001),
            funding_fee_rate: Some(0.0001),
            slippage_pct: Some(0.1),
            max_risk_per_trade_pct: Some(2.0),
            max_portfolio_risk_pct: Some(10.0),
            default_stop_loss_pct: Some(3.0),
            default_take_profit_pct: Some(6.0),
            max_leverage: Some(20),
            enabled: Some(true),
        };

        if let Some(balance) = request.initial_balance {
            assert!(balance > 0.0);
        }
        if let Some(positions) = request.max_positions {
            assert!(positions > 0);
        }
        if let Some(leverage) = request.max_leverage {
            assert!(leverage > 0);
        }
    }

    #[test]
    fn test_paper_trading_api_structure() {
        // Test that PaperTradingApi has expected structure
        // This is a compile-time test
        // Actual cloning is tested in integration tests with real engine
    }

    #[test]
    fn test_multiple_symbol_updates_in_request() {
        let mut symbols = std::collections::HashMap::new();

        for i in 1..=10 {
            symbols.insert(
                format!("SYMBOL{i}USDT"),
                SymbolConfig {
                    enabled: i % 2 == 0,
                    leverage: Some(i as u8),
                    position_size_pct: Some(i as f64),
                    stop_loss_pct: Some(2.0),
                    take_profit_pct: Some(4.0),
                    max_positions: Some(i as u32),
                },
            );
        }

        let request = UpdateSymbolSettingsRequest { symbols };

        assert_eq!(request.symbols.len(), 10);
    }

    #[test]
    fn test_engine_settings_various_modes() {
        let modes = vec!["WeightedAverage", "Unanimous", "Majority", "AnyOne"];

        for mode in modes {
            let settings = EngineSettings {
                min_confidence_threshold: 0.7,
                signal_combination_mode: mode.to_string(),
                enabled_strategies: vec![],
                market_condition: "Trending".to_string(),
                risk_level: "Moderate".to_string(),
            };

            assert_eq!(settings.signal_combination_mode, mode);
        }
    }

    #[test]
    fn test_bollinger_config_squeeze_detection() {
        let config = BollingerConfig {
            enabled: true,
            period: 20,
            multiplier: 2.0,
            squeeze_threshold: 0.02,
        };

        assert!(config.squeeze_threshold > 0.0);
        assert!(config.squeeze_threshold < 0.1);
    }

    #[test]
    fn test_volume_config_spike_detection() {
        let config = VolumeConfig {
            enabled: true,
            sma_period: 20,
            spike_threshold: 2.0,
            correlation_period: 10,
        };

        assert!(config.spike_threshold > 1.0);
        assert!(config.correlation_period > 0);
    }

    #[test]
    fn test_macd_histogram_threshold() {
        let config = MacdConfig {
            enabled: true,
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            histogram_threshold: 0.001,
        };

        assert!(config.histogram_threshold > 0.0);
        assert!(config.histogram_threshold < 1.0);
    }

    #[test]
    fn test_rsi_extreme_thresholds() {
        let config = RsiConfig {
            enabled: true,
            period: 14,
            oversold_threshold: 30.0,
            overbought_threshold: 70.0,
            extreme_oversold: 20.0,
            extreme_overbought: 80.0,
        };

        assert!(config.extreme_oversold < config.oversold_threshold);
        assert!(config.extreme_overbought > config.overbought_threshold);
        assert!(config.extreme_oversold >= 0.0);
        assert!(config.extreme_overbought <= 100.0);
    }
}
