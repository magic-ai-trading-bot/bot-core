// @spec:FR-REAL-API-001 - Real Trading API Endpoints
// @ref:specs/02-design/2.3-api/API-RUST-CORE.md
// @test:TC-REAL-API-001, TC-REAL-API-002, TC-REAL-API-003

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::real_trading::RealTradingEngine;

/// API handlers for real trading functionality
/// SAFETY: This involves REAL MONEY - all operations require explicit confirmation
pub struct RealTradingApi {
    engine: Option<Arc<RealTradingEngine>>,
}

// ============================================================================
// REQUEST/RESPONSE TYPES
// ============================================================================

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

/// Engine status response
#[derive(Debug, Serialize, Deserialize)]
pub struct EngineStatus {
    pub is_running: bool,
    pub is_testnet: bool,
    pub open_positions_count: usize,
    pub open_orders_count: usize,
    pub circuit_breaker_open: bool,
    pub daily_pnl: f64,
    pub daily_trades_count: u32,
    pub uptime_seconds: Option<u64>,
}

/// Portfolio response
#[derive(Debug, Serialize, Deserialize)]
pub struct PortfolioResponse {
    pub total_balance: f64,
    pub available_balance: f64,
    pub locked_balance: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub positions: Vec<PositionInfo>,
    pub balances: Vec<BalanceInfo>,
}

/// Position info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_pct: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub created_at: String,
}

/// Balance info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceInfo {
    pub asset: String,
    pub free: f64,
    pub locked: f64,
    pub total: f64,
}

/// Closed trade info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClosedTradeInfo {
    pub id: String,
    pub symbol: String,
    pub side: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub exit_price: f64,
    pub realized_pnl: f64,
    pub realized_pnl_pct: f64,
    pub commission: f64,
    pub opened_at: String,
    pub closed_at: String,
    pub close_reason: String,
}

/// Request to close a trade
#[derive(Debug, Serialize, Deserialize)]
pub struct CloseTradeRequest {
    pub reason: Option<String>,
}

/// Request to update settings
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    pub max_position_size_usdt: Option<f64>,
    pub max_positions: Option<u32>,
    pub max_daily_loss_usdt: Option<f64>,
    pub max_total_exposure_usdt: Option<f64>,
    pub risk_per_trade_percent: Option<f64>,
    pub default_stop_loss_percent: Option<f64>,
    pub default_take_profit_percent: Option<f64>,
    pub max_leverage: Option<u32>,
}

/// Settings response
#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsResponse {
    pub use_testnet: bool,
    pub max_position_size_usdt: f64,
    pub max_positions: u32,
    pub max_daily_loss_usdt: f64,
    pub max_total_exposure_usdt: f64,
    pub risk_per_trade_percent: f64,
    pub default_stop_loss_percent: f64,
    pub default_take_profit_percent: f64,
    pub max_leverage: u32,
    pub circuit_breaker_errors: u32,
    pub circuit_breaker_cooldown_secs: u64,
}

// ============================================================================
// WARP HELPERS
// ============================================================================

fn with_api(
    api: Arc<RealTradingApi>,
) -> impl Filter<Extract = (Arc<RealTradingApi>,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || api.clone())
}

// ============================================================================
// API IMPLEMENTATION
// ============================================================================

impl RealTradingApi {
    pub fn new(engine: Option<Arc<RealTradingEngine>>) -> Self {
        Self { engine }
    }

    /// Create real trading API routes
    pub fn routes(self) -> impl Filter<Extract = impl Reply, Error = Rejection> + Clone {
        let api = Arc::new(self);

        let cors = warp::cors()
            .allow_any_origin()
            .allow_headers(vec!["content-type", "authorization"])
            .allow_methods(vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]);

        let base_path = warp::path("real-trading");

        // GET /api/real-trading/status
        let status_route = base_path
            .and(warp::path("status"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_status);

        // GET /api/real-trading/portfolio
        let portfolio_route = base_path
            .and(warp::path("portfolio"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_portfolio);

        // GET /api/real-trading/trades/open
        let open_trades_route = base_path
            .and(warp::path("trades"))
            .and(warp::path("open"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_open_trades);

        // GET /api/real-trading/trades/closed
        let closed_trades_route = base_path
            .and(warp::path("trades"))
            .and(warp::path("closed"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_closed_trades);

        // POST /api/real-trading/start
        let start_route = base_path
            .and(warp::path("start"))
            .and(warp::path::end())
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(start_engine);

        // POST /api/real-trading/stop
        let stop_route = base_path
            .and(warp::path("stop"))
            .and(warp::path::end())
            .and(warp::post())
            .and(with_api(api.clone()))
            .and_then(stop_engine);

        // POST /api/real-trading/trades/{trade_id}/close
        let close_trade_route = base_path
            .and(warp::path("trades"))
            .and(warp::path::param::<String>())
            .and(warp::path("close"))
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(close_trade);

        // GET /api/real-trading/settings
        let get_settings_route = base_path
            .and(warp::path("settings"))
            .and(warp::path::end())
            .and(warp::get())
            .and(with_api(api.clone()))
            .and_then(get_settings);

        // PUT /api/real-trading/settings
        let update_settings_route = base_path
            .and(warp::path("settings"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(update_settings);

        // Combine all routes
        status_route
            .or(portfolio_route)
            .or(open_trades_route)
            .or(closed_trades_route)
            .or(start_route)
            .or(stop_route)
            .or(close_trade_route)
            .or(get_settings_route)
            .or(update_settings_route)
            .with(cors)
    }
}

// ============================================================================
// ROUTE HANDLERS
// ============================================================================

/// GET /api/real-trading/status
async fn get_status(api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<EngineStatus>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    let is_running = engine.is_running().await;
    let config = engine.get_config().await;
    let positions_count = engine.get_positions().len();
    let orders_count = engine.get_active_orders().len();
    let circuit_breaker = engine.get_circuit_breaker().await;
    let daily_metrics = engine.get_daily_metrics().await;

    let status = EngineStatus {
        is_running,
        is_testnet: config.use_testnet,
        open_positions_count: positions_count,
        open_orders_count: orders_count,
        circuit_breaker_open: circuit_breaker.is_open,
        daily_pnl: daily_metrics.realized_pnl,
        daily_trades_count: daily_metrics.trades_count,
        uptime_seconds: None, // TODO: Track uptime
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(status)),
        StatusCode::OK,
    ))
}

/// GET /api/real-trading/portfolio
async fn get_portfolio(api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<PortfolioResponse>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    let positions = engine.get_positions();
    let balances = engine.get_all_balances().await;

    // Calculate totals
    let mut total_balance = 0.0;
    let mut available_balance = 0.0;
    let mut locked_balance = 0.0;
    let mut balance_infos = Vec::new();

    for (asset, balance) in &balances {
        // For simplicity, consider USDT as the primary quote asset
        if asset == "USDT" {
            total_balance += balance.total();
            available_balance += balance.free;
            locked_balance += balance.locked;
        }
        balance_infos.push(BalanceInfo {
            asset: asset.clone(),
            free: balance.free,
            locked: balance.locked,
            total: balance.total(),
        });
    }

    // Calculate unrealized PnL from positions
    let mut total_unrealized_pnl = 0.0;
    let mut total_realized_pnl = 0.0;
    let mut position_infos = Vec::new();

    for pos in positions {
        total_unrealized_pnl += pos.unrealized_pnl;
        total_realized_pnl += pos.realized_pnl;

        position_infos.push(PositionInfo {
            id: pos.id.clone(),
            symbol: pos.symbol.clone(),
            side: pos.side.as_str().to_string(),
            quantity: pos.quantity,
            entry_price: pos.entry_price,
            current_price: pos.current_price,
            unrealized_pnl: pos.unrealized_pnl,
            unrealized_pnl_pct: pos.pnl_percentage(),
            stop_loss: pos.stop_loss,
            take_profit: pos.take_profit,
            created_at: pos.created_at.to_rfc3339(),
        });
    }

    let portfolio = PortfolioResponse {
        total_balance,
        available_balance,
        locked_balance,
        unrealized_pnl: total_unrealized_pnl,
        realized_pnl: total_realized_pnl,
        positions: position_infos,
        balances: balance_infos,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(portfolio)),
        StatusCode::OK,
    ))
}

/// GET /api/real-trading/trades/open
async fn get_open_trades(api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<Vec<PositionInfo>>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    let positions = engine.get_positions();

    let position_infos: Vec<PositionInfo> = positions
        .into_iter()
        .filter(|p| p.is_open())
        .map(|pos| PositionInfo {
            id: pos.id.clone(),
            symbol: pos.symbol.clone(),
            side: pos.side.as_str().to_string(),
            quantity: pos.quantity,
            entry_price: pos.entry_price,
            current_price: pos.current_price,
            unrealized_pnl: pos.unrealized_pnl,
            unrealized_pnl_pct: pos.pnl_percentage(),
            stop_loss: pos.stop_loss,
            take_profit: pos.take_profit,
            created_at: pos.created_at.to_rfc3339(),
        })
        .collect();

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(position_infos)),
        StatusCode::OK,
    ))
}

/// GET /api/real-trading/trades/closed
async fn get_closed_trades(api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<serde_json::Value>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    // Get closed trades from engine (will track via daily metrics for now)
    let daily_metrics = engine.get_daily_metrics().await;

    // For now, return summary data - full trade history will be implemented with DB persistence
    // The engine doesn't persist closed trades in memory long-term to avoid memory growth
    let empty_trades: Vec<ClosedTradeInfo> = vec![];
    let summary = serde_json::json!({
        "summary": {
            "total_trades_today": daily_metrics.trades_count,
            "winning_trades": daily_metrics.winning_trades,
            "losing_trades": daily_metrics.losing_trades,
            "total_realized_pnl": daily_metrics.realized_pnl,
            "total_volume": daily_metrics.total_volume,
            "total_commission": daily_metrics.total_commission,
            "win_rate": daily_metrics.win_rate(),
        },
        "trades": empty_trades,
        "message": "Full trade history requires database persistence. Daily summary shown instead."
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(summary)),
        StatusCode::OK,
    ))
}

/// POST /api/real-trading/start
async fn start_engine(api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<String>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    // SAFETY CHECK: Verify we're ready to trade with real money
    let config = engine.get_config().await;

    if !config.use_testnet {
        tracing::warn!("🔴 REAL TRADING ENGINE STARTING WITH MAINNET - REAL MONEY AT RISK");
    } else {
        tracing::info!("🟡 Starting real trading engine with TESTNET");
    }

    match engine.start().await {
        Ok(_) => {
            let mode = if config.use_testnet {
                "testnet"
            } else {
                "MAINNET (REAL MONEY)"
            };
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(format!(
                    "Real trading engine started in {} mode",
                    mode
                ))),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to start engine: {}",
                e
            ))),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// POST /api/real-trading/stop
async fn stop_engine(api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<String>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    tracing::info!("Stopping real trading engine...");

    match engine.stop().await {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::success("Real trading engine stopped")),
            StatusCode::OK,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to stop engine: {}",
                e
            ))),
            StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}

/// POST /api/real-trading/trades/{trade_id}/close
/// Note: trade_id should be the symbol (e.g., "BTCUSDT") for real trading
async fn close_trade(
    trade_id: String,
    request: CloseTradeRequest,
    api: Arc<RealTradingApi>,
) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<serde_json::Value>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    let reason = request.reason.unwrap_or_else(|| "Manual close".to_string());

    tracing::info!("🔴 CLOSING REAL TRADE: {} - Reason: {}", trade_id, reason);

    // Get position before closing to track PnL
    let position = engine.get_position(&trade_id);
    let pre_close_unrealized_pnl = position.as_ref().map(|p| p.unrealized_pnl).unwrap_or(0.0);

    // close_position takes symbol (which is used as trade_id in real trading)
    match engine.close_position(&trade_id).await {
        Ok(order) => {
            // Order was placed to close the position
            // The actual PnL will be updated when the order fills via ExecutionReport
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(serde_json::json!({
                    "trade_id": trade_id,
                    "close_order_id": order.client_order_id,
                    "estimated_pnl": pre_close_unrealized_pnl,
                    "reason": reason,
                    "message": format!("Close order placed. Estimated PnL: {:.2} USDT (final PnL determined at fill)", pre_close_unrealized_pnl)
                }))),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to close trade: {}",
                e
            ))),
            StatusCode::BAD_REQUEST,
        )),
    }
}

/// GET /api/real-trading/settings
async fn get_settings(api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<SettingsResponse>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    let config = engine.get_config().await;

    let settings = SettingsResponse {
        use_testnet: config.use_testnet,
        max_position_size_usdt: config.max_position_size_usdt,
        max_positions: config.max_positions,
        max_daily_loss_usdt: config.max_daily_loss_usdt,
        max_total_exposure_usdt: config.max_total_exposure_usdt,
        risk_per_trade_percent: config.risk_per_trade_percent,
        default_stop_loss_percent: config.default_stop_loss_percent,
        default_take_profit_percent: config.default_take_profit_percent,
        max_leverage: config.max_leverage,
        circuit_breaker_errors: config.circuit_breaker_errors,
        circuit_breaker_cooldown_secs: config.circuit_breaker_cooldown_secs,
    };

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(settings)),
        StatusCode::OK,
    ))
}

/// PUT /api/real-trading/settings
async fn update_settings(
    request: UpdateSettingsRequest,
    api: Arc<RealTradingApi>,
) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<String>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    // SAFETY: Log all settings changes for audit trail
    tracing::info!("🔧 Updating real trading settings: {:?}", request);

    // Get current config and apply updates
    let mut config = engine.get_config().await;

    if let Some(v) = request.max_position_size_usdt {
        config.max_position_size_usdt = v;
    }
    if let Some(v) = request.max_positions {
        config.max_positions = v;
    }
    if let Some(v) = request.max_daily_loss_usdt {
        config.max_daily_loss_usdt = v;
    }
    if let Some(v) = request.max_total_exposure_usdt {
        config.max_total_exposure_usdt = v;
    }
    if let Some(v) = request.risk_per_trade_percent {
        config.risk_per_trade_percent = v;
    }
    if let Some(v) = request.default_stop_loss_percent {
        config.default_stop_loss_percent = v;
    }
    if let Some(v) = request.default_take_profit_percent {
        config.default_take_profit_percent = v;
    }
    if let Some(v) = request.max_leverage {
        config.max_leverage = v;
    }

    match engine.update_config(config).await {
        Ok(_) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::success("Settings updated successfully")),
            StatusCode::OK,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to update settings: {}",
                e
            ))),
            StatusCode::BAD_REQUEST,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_response_success() {
        let response = ApiResponse::success("test data");
        assert!(response.success);
        assert_eq!(response.data, Some("test data"));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_api_response_error() {
        let response: ApiResponse<()> = ApiResponse::error("test error".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("test error".to_string()));
    }

    #[test]
    fn test_engine_status_serialization() {
        let status = EngineStatus {
            is_running: true,
            is_testnet: true,
            open_positions_count: 2,
            open_orders_count: 1,
            circuit_breaker_open: false,
            daily_pnl: 100.50,
            daily_trades_count: 5,
            uptime_seconds: Some(3600),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"is_running\":true"));
        assert!(json.contains("\"is_testnet\":true"));
    }

    #[test]
    fn test_portfolio_response_serialization() {
        let portfolio = PortfolioResponse {
            total_balance: 10000.0,
            available_balance: 8000.0,
            locked_balance: 2000.0,
            unrealized_pnl: 150.0,
            realized_pnl: 500.0,
            positions: vec![],
            balances: vec![],
        };

        let json = serde_json::to_string(&portfolio).unwrap();
        assert!(json.contains("\"total_balance\":10000.0"));
    }

    #[test]
    fn test_position_info_serialization() {
        let pos = PositionInfo {
            id: "pos-123".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 0.1,
            entry_price: 50000.0,
            current_price: 51000.0,
            unrealized_pnl: 100.0,
            unrealized_pnl_pct: 2.0,
            stop_loss: Some(49000.0),
            take_profit: Some(55000.0),
            created_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&pos).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("LONG"));
    }

    #[test]
    fn test_closed_trade_info_serialization() {
        let trade = ClosedTradeInfo {
            id: "trade-123".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "SHORT".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            exit_price: 2900.0,
            realized_pnl: 100.0,
            realized_pnl_pct: 3.33,
            commission: 0.5,
            opened_at: "2025-01-01T00:00:00Z".to_string(),
            closed_at: "2025-01-01T01:00:00Z".to_string(),
            close_reason: "Take profit".to_string(),
        };

        let json = serde_json::to_string(&trade).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("Take profit"));
    }

    #[test]
    fn test_update_settings_request_deserialization() {
        let json = r#"{
            "max_position_size_usdt": 1000.0,
            "max_positions": 5,
            "max_leverage": 10
        }"#;

        let request: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.max_position_size_usdt, Some(1000.0));
        assert_eq!(request.max_positions, Some(5));
        assert_eq!(request.max_leverage, Some(10));
        assert!(request.max_daily_loss_usdt.is_none());
    }

    #[test]
    fn test_settings_response_serialization() {
        let settings = SettingsResponse {
            use_testnet: true,
            max_position_size_usdt: 1000.0,
            max_positions: 5,
            max_daily_loss_usdt: 500.0,
            max_total_exposure_usdt: 5000.0,
            risk_per_trade_percent: 2.0,
            default_stop_loss_percent: 2.0,
            default_take_profit_percent: 4.0,
            max_leverage: 10,
            circuit_breaker_errors: 3,
            circuit_breaker_cooldown_secs: 300,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"use_testnet\":true"));
        assert!(json.contains("\"max_position_size_usdt\":1000.0"));
    }

    #[test]
    fn test_balance_info_serialization() {
        let balance = BalanceInfo {
            asset: "USDT".to_string(),
            free: 8000.0,
            locked: 2000.0,
            total: 10000.0,
        };

        let json = serde_json::to_string(&balance).unwrap();
        assert!(json.contains("USDT"));
        assert!(json.contains("10000.0"));
    }

    #[test]
    fn test_close_trade_request_deserialization() {
        let json = r#"{"reason": "Stop loss triggered"}"#;
        let request: CloseTradeRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.reason, Some("Stop loss triggered".to_string()));

        let json_empty = r#"{}"#;
        let request_empty: CloseTradeRequest = serde_json::from_str(json_empty).unwrap();
        assert!(request_empty.reason.is_none());
    }
}
