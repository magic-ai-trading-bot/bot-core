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
#[derive(Debug, Deserialize)]
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
pub struct RiskSettings {
    pub max_risk_per_trade: f64,
    pub max_portfolio_risk: f64,
    pub stop_loss_percent: f64,
    pub take_profit_percent: f64,
    pub max_leverage: u32,
    pub max_drawdown: f64,
    pub daily_loss_limit: f64,
    pub max_consecutive_losses: u32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EngineSettings {
    pub min_confidence_threshold: f64,
    pub signal_combination_mode: String,
    pub enabled_strategies: Vec<String>,
    pub market_condition: String,
    pub risk_level: String,
}

/// Request to update strategy settings
#[derive(Debug, Deserialize)]
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
#[derive(Debug, Serialize)]
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
    pub fn routes(&self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = Arc::new(self.clone());

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
        }
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
        }
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
        }
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
        }
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
        }
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
        },
        engine: EngineSettings {
            min_confidence_threshold: engine_settings.strategy.min_ai_confidence, // 🎯 ACTUAL THRESHOLD
            signal_combination_mode: "WeightedAverage".to_string(),
            enabled_strategies: vec![
                "RSI Strategy".to_string(),
                "MACD Strategy".to_string(),
                "Volume Strategy".to_string(),
                "Bollinger Bands Strategy".to_string(),
            ],
            market_condition: "Trending".to_string(),
            risk_level: "Moderate".to_string(),
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

    // Get current settings and update with new values
    let current_settings = api.engine.get_settings().await;
    let _new_settings = current_settings.clone();

    // Update confidence threshold - this is the key setting!
    let confidence_threshold = request.settings.engine.min_confidence_threshold;
    log::info!("Applying confidence threshold: {}", confidence_threshold);

    // Update engine confidence threshold (this affects trade creation)
    // We need to update the internal engine configuration
    match api
        .engine
        .update_confidence_threshold(confidence_threshold)
        .await
    {
        Ok(_) => {
            log::info!(
                "✅ Confidence threshold updated to: {}",
                confidence_threshold
            );

            let response = serde_json::json!({
                "message": "Strategy settings updated successfully",
                "applied_settings": {
                    "confidence_threshold": confidence_threshold,
                    "market_condition": request.settings.engine.market_condition,
                    "risk_level": request.settings.engine.risk_level,
                },
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(response)),
                StatusCode::OK,
            ))
        }
        Err(e) => {
            log::error!("❌ Failed to update confidence threshold: {}", e);

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(format!(
                    "Failed to update settings: {}",
                    e
                ))),
                StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
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
    log::info!("Updating basic paper trading settings: {:?}", request);

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
                    log::error!("Failed to reset portfolio after settings update: {}", e);
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
        }
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
        }
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
        }
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
        }
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::BAD_REQUEST,
        )),
    }
}
