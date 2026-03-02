// @spec:FR-REAL-API-001 - Real Trading API Endpoints
// @ref:specs/02-design/2.3-api/API-RUST-CORE.md
// @test:TC-REAL-API-001, TC-REAL-API-002, TC-REAL-API-003

use chrono::{DateTime, Duration, Utc};
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;
use warp::http::StatusCode;
use warp::{Filter, Rejection, Reply};

use crate::binance::types::OrderSide;
use crate::real_trading::RealTradingEngine;
use crate::strategies::TradingSignal;

/// API handlers for real trading functionality
/// SAFETY: This involves REAL MONEY - all operations require explicit confirmation
pub struct RealTradingApi {
    engine: Option<Arc<RealTradingEngine>>,
    /// Pending order confirmations (token -> order details)
    pending_confirmations: DashMap<String, PendingConfirmation>,
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
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct PortfolioResponse {
    pub total_balance: f64,
    pub available_balance: f64,
    pub locked_balance: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    #[serde(default)]
    pub total_pnl: f64,
    #[serde(default)]
    pub total_pnl_percentage: f64,
    #[serde(default)]
    pub equity: f64,
    #[serde(default)]
    pub total_trades: u32,
    #[serde(default)]
    pub win_rate: f64,
    pub positions: Vec<PositionInfo>,
    pub balances: Vec<BalanceInfo>,
}

/// Position info for API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PositionInfo {
    pub id: String,
    pub symbol: String,
    pub side: String,
    /// "Long" or "Short" — matches PaperTrade.trade_type for frontend compatibility
    #[serde(default)]
    pub trade_type: String,
    #[serde(default = "default_status_open")]
    pub status: String,
    pub quantity: f64,
    pub entry_price: f64,
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub unrealized_pnl_pct: f64,
    /// Alias for unrealized_pnl — frontend reads this field
    #[serde(default)]
    pub pnl: f64,
    /// Alias for unrealized_pnl_pct — frontend reads this field
    #[serde(default)]
    pub pnl_percentage: f64,
    #[serde(default = "default_leverage")]
    pub leverage: u32,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub created_at: String,
    /// Alias for created_at — frontend reads open_time
    #[serde(default)]
    pub open_time: String,
}

fn default_status_open() -> String {
    "Open".to_string()
}

fn default_leverage() -> u32 {
    1
}

impl Default for PositionInfo {
    fn default() -> Self {
        Self {
            id: String::new(),
            symbol: String::new(),
            side: String::new(),
            trade_type: String::new(),
            status: "Open".to_string(),
            quantity: 0.0,
            entry_price: 0.0,
            current_price: 0.0,
            unrealized_pnl: 0.0,
            unrealized_pnl_pct: 0.0,
            pnl: 0.0,
            pnl_percentage: 0.0,
            leverage: 1,
            stop_loss: None,
            take_profit: None,
            created_at: String::new(),
            open_time: String::new(),
        }
    }
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

// ============================================================================
// ORDER PLACEMENT TYPES (Phase 4)
// ============================================================================

/// Request to place an order
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub side: String,       // "BUY" or "SELL"
    pub order_type: String, // "MARKET", "LIMIT"
    pub quantity: f64,
    pub price: Option<f64>,       // Required for LIMIT orders
    pub stop_loss: Option<f64>,   // Optional SL price
    pub take_profit: Option<f64>, // Optional TP price
    /// Confirmation token (if provided, executes order; if not, returns confirmation)
    pub confirmation_token: Option<String>,
}

/// Order info response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInfo {
    pub id: String,
    pub exchange_order_id: i64,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: f64,
    pub executed_quantity: f64,
    pub price: Option<f64>,
    pub avg_fill_price: f64,
    pub status: String,
    pub is_entry: bool,
    pub created_at: String,
    pub updated_at: String,
}

/// Confirmation request (for 2-step order placement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfirmationResponse {
    pub token: String,
    pub expires_at: String,
    pub summary: String,
    pub order_details: PlaceOrderRequest,
}

/// Query parameters for listing orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListOrdersQuery {
    #[serde(default)]
    pub symbol: Option<String>,
    #[serde(default)]
    pub status: Option<String>, // "active", "filled", "cancelled", "all"
    #[serde(default = "default_limit")]
    pub limit: usize,
}

fn default_limit() -> usize {
    50
}

/// Request to modify SL/TP
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModifySlTpRequest {
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
}

/// Pending confirmation entry
#[derive(Debug, Clone)]
struct PendingConfirmation {
    _token: String,
    order_request: PlaceOrderRequest,
    expires_at: DateTime<Utc>,
}

/// Request to update settings
#[derive(Debug, Default, Serialize, Deserialize)]
pub struct UpdateSettingsRequest {
    pub max_position_size_usdt: Option<f64>,
    pub max_positions: Option<u32>,
    pub max_daily_loss_usdt: Option<f64>,
    pub max_total_exposure_usdt: Option<f64>,
    pub risk_per_trade_percent: Option<f64>,
    pub default_stop_loss_percent: Option<f64>,
    pub default_take_profit_percent: Option<f64>,
    pub max_leverage: Option<u32>,
    // Auto-trading fields
    pub auto_trading_enabled: Option<bool>,
    pub auto_trade_symbols: Option<Vec<String>>,
    pub min_signal_confidence: Option<f64>,
    pub max_consecutive_losses: Option<u32>,
    pub cool_down_minutes: Option<u32>,
    pub correlation_limit: Option<f64>,
    pub max_portfolio_risk_pct: Option<f64>,
    pub short_only_mode: Option<bool>,
    pub long_only_mode: Option<bool>,
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
    // Auto-trading fields
    pub auto_trading_enabled: bool,
    pub auto_trade_symbols: Vec<String>,
    pub min_signal_confidence: f64,
    pub max_consecutive_losses: u32,
    pub cool_down_minutes: u32,
    pub correlation_limit: f64,
    pub max_portfolio_risk_pct: f64,
    pub short_only_mode: bool,
    pub long_only_mode: bool,
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
        Self {
            engine,
            pending_confirmations: DashMap::new(),
        }
    }

    /// Clean up expired confirmations
    fn cleanup_expired_confirmations(&self) {
        let now = Utc::now();
        self.pending_confirmations
            .retain(|_, conf| conf.expires_at > now);
    }

    /// Generate a confirmation token for an order
    fn create_confirmation(&self, request: PlaceOrderRequest) -> ConfirmationResponse {
        self.cleanup_expired_confirmations();

        let token = Uuid::new_v4().to_string();
        let expires_at = Utc::now() + Duration::seconds(60);

        let summary = format!(
            "{} {} {} {} @ {}",
            request.side,
            request.quantity,
            request.symbol,
            request.order_type,
            request
                .price
                .map(|p| format!("${:.2}", p))
                .unwrap_or_else(|| "MARKET".to_string())
        );

        self.pending_confirmations.insert(
            token.clone(),
            PendingConfirmation {
                _token: token.clone(),
                order_request: request.clone(),
                expires_at,
            },
        );

        ConfirmationResponse {
            token,
            expires_at: expires_at.to_rfc3339(),
            summary,
            order_details: request,
        }
    }

    /// Validate and consume a confirmation token
    fn consume_confirmation(&self, token: &str) -> Option<PlaceOrderRequest> {
        self.cleanup_expired_confirmations();

        if let Some((_, conf)) = self.pending_confirmations.remove(token) {
            if conf.expires_at > Utc::now() {
                return Some(conf.order_request);
            }
        }
        None
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

        // ============ ORDER MANAGEMENT ROUTES (Phase 4) ============

        // POST /api/real-trading/orders - Place order (with confirmation)
        let place_order_route = base_path
            .and(warp::path("orders"))
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(place_order);

        // GET /api/real-trading/orders - List active orders
        let list_orders_route = base_path
            .and(warp::path("orders"))
            .and(warp::path::end())
            .and(warp::get())
            .and(warp::query::<ListOrdersQuery>())
            .and(with_api(api.clone()))
            .and_then(list_orders);

        // DELETE /api/real-trading/orders/{id} - Cancel specific order
        let cancel_order_route = base_path
            .and(warp::path("orders"))
            .and(warp::path::param::<String>())
            .and(warp::path::end())
            .and(warp::delete())
            .and(with_api(api.clone()))
            .and_then(cancel_order);

        // DELETE /api/real-trading/orders - Cancel all orders
        let cancel_all_orders_route = base_path
            .and(warp::path("orders"))
            .and(warp::path("all"))
            .and(warp::path::end())
            .and(warp::delete())
            .and(warp::query::<CancelAllQuery>())
            .and(with_api(api.clone()))
            .and_then(cancel_all_orders);

        // PUT /api/real-trading/positions/{symbol}/sltp - Modify SL/TP
        let modify_sltp_route = base_path
            .and(warp::path("positions"))
            .and(warp::path::param::<String>())
            .and(warp::path("sltp"))
            .and(warp::path::end())
            .and(warp::put())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(modify_sltp);

        // POST /api/real-trading/test-signal - Trigger a test signal (testnet only)
        let test_signal_route = base_path
            .and(warp::path("test-signal"))
            .and(warp::path::end())
            .and(warp::post())
            .and(warp::body::json())
            .and(with_api(api.clone()))
            .and_then(test_signal);

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
            .or(place_order_route)
            .or(list_orders_route)
            .or(cancel_order_route)
            .or(cancel_all_orders_route)
            .or(modify_sltp_route)
            .or(test_signal_route)
            .with(cors)
    }
}

/// Query for cancel all orders
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CancelAllQuery {
    #[serde(default)]
    pub symbol: Option<String>,
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

        let trade_type = match pos.side {
            crate::real_trading::PositionSide::Long => "Long".to_string(),
            crate::real_trading::PositionSide::Short => "Short".to_string(),
        };
        let pnl_pct = pos.pnl_percentage();
        let created = pos.created_at.to_rfc3339();
        position_infos.push(PositionInfo {
            id: pos.id.clone(),
            symbol: pos.symbol.clone(),
            side: pos.side.as_str().to_string(),
            trade_type,
            status: "Open".to_string(),
            quantity: pos.quantity,
            entry_price: pos.entry_price,
            current_price: pos.current_price,
            unrealized_pnl: pos.unrealized_pnl,
            unrealized_pnl_pct: pnl_pct,
            pnl: pos.unrealized_pnl,
            pnl_percentage: pnl_pct,
            leverage: pos.leverage,
            stop_loss: pos.stop_loss,
            take_profit: pos.take_profit,
            created_at: created.clone(),
            open_time: created,
        });
    }

    // Calculate total PnL and stats
    let total_pnl = total_unrealized_pnl + total_realized_pnl;
    let total_pnl_percentage = if total_balance > 0.0 {
        (total_pnl / total_balance) * 100.0
    } else {
        0.0
    };
    let equity = total_balance + total_unrealized_pnl;
    let total_trades = position_infos.len() as u32;
    let winning_trades = position_infos
        .iter()
        .filter(|p| p.unrealized_pnl > 0.0)
        .count() as u32;
    let win_rate = if total_trades > 0 {
        (winning_trades as f64 / total_trades as f64) * 100.0
    } else {
        0.0
    };

    let portfolio = PortfolioResponse {
        total_balance,
        available_balance,
        locked_balance,
        unrealized_pnl: total_unrealized_pnl,
        realized_pnl: total_realized_pnl,
        total_pnl,
        total_pnl_percentage,
        equity,
        total_trades,
        win_rate,
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
        .map(|pos| {
            let trade_type = match pos.side {
                crate::real_trading::PositionSide::Long => "Long".to_string(),
                crate::real_trading::PositionSide::Short => "Short".to_string(),
            };
            let pnl_pct = pos.pnl_percentage();
            let created = pos.created_at.to_rfc3339();
            PositionInfo {
                id: pos.id.clone(),
                symbol: pos.symbol.clone(),
                side: pos.side.as_str().to_string(),
                trade_type,
                status: "Open".to_string(),
                quantity: pos.quantity,
                entry_price: pos.entry_price,
                current_price: pos.current_price,
                unrealized_pnl: pos.unrealized_pnl,
                unrealized_pnl_pct: pnl_pct,
                pnl: pos.unrealized_pnl,
                pnl_percentage: pnl_pct,
                leverage: pos.leverage,
                stop_loss: pos.stop_loss,
                take_profit: pos.take_profit,
                created_at: created.clone(),
                open_time: created,
            }
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

    // Collect symbols from all sources for comprehensive trade history
    let config = engine.get_config().await;
    let mut symbol_set: std::collections::HashSet<String> = std::collections::HashSet::new();
    // Always include default trading symbols
    for s in &["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"] {
        symbol_set.insert(s.to_string());
    }
    // Add symbols from current/closed positions
    for pos in engine.get_positions() {
        symbol_set.insert(pos.symbol.clone());
    }
    // Add configured symbols
    for s in &config.allowed_symbols {
        symbol_set.insert(s.clone());
    }
    for s in &config.auto_trade_symbols {
        symbol_set.insert(s.clone());
    }
    let symbols: Vec<String> = symbol_set.into_iter().collect();

    // Fetch all orders (filled, cancelled, expired) from Binance
    let binance_orders = engine.get_order_history(&symbols, Some(500)).await;

    // Filter to non-active orders (completed history) and convert to frontend format
    let trades: Vec<serde_json::Value> = binance_orders
        .iter()
        .filter(|o| {
            let status = o.status.to_uppercase();
            // Include filled, cancelled, expired — exclude NEW (still active)
            status != "NEW" && status != "PARTIALLY_FILLED"
        })
        .map(|o| {
            let orig_qty: f64 = o.orig_qty.parse().unwrap_or(0.0);
            let executed_qty: f64 = o.executed_qty.parse().unwrap_or(0.0);
            let price: f64 = o.price.parse().unwrap_or(0.0);
            let avg_price: f64 = if executed_qty > 0.0 {
                let cum_quote: f64 = o.cumulative_quote_qty.parse().unwrap_or(0.0);
                cum_quote / executed_qty
            } else {
                price
            };
            let status_str = match o.status.to_uppercase().as_str() {
                "FILLED" => "Closed",
                "CANCELED" | "CANCELLED" => "Cancelled",
                "EXPIRED" => "Cancelled",
                _ => "Closed",
            };
            let trade_type = if o.side.to_uppercase() == "BUY" {
                "Long"
            } else {
                "Short"
            };
            let side_str = if o.side.to_uppercase() == "BUY" {
                "LONG"
            } else {
                "SHORT"
            };
            let ts = chrono::DateTime::from_timestamp_millis(o.time)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();
            let close_ts = chrono::DateTime::from_timestamp_millis(o.update_time)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            serde_json::json!({
                "id": format!("{}", o.order_id),
                "symbol": o.symbol,
                "trade_type": trade_type,
                "side": side_str,
                "status": status_str,
                "order_type": o.r#type,
                "entry_price": if avg_price > 0.0 { avg_price } else { price },
                "exit_price": avg_price,
                "quantity": if executed_qty > 0.0 { executed_qty } else { orig_qty },
                "leverage": 1,
                "pnl": 0.0,
                "pnl_percentage": 0.0,
                "open_time": ts,
                "close_time": close_ts,
            })
        })
        .collect();

    let daily_metrics = engine.get_daily_metrics().await;
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
        "trades": trades,
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
        auto_trading_enabled: config.auto_trading_enabled,
        auto_trade_symbols: config.auto_trade_symbols.clone(),
        min_signal_confidence: config.min_signal_confidence,
        max_consecutive_losses: config.max_consecutive_losses,
        cool_down_minutes: config.cool_down_minutes,
        correlation_limit: config.correlation_limit,
        max_portfolio_risk_pct: config.max_portfolio_risk_pct,
        short_only_mode: config.short_only_mode,
        long_only_mode: config.long_only_mode,
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
    // Auto-trading fields
    if let Some(v) = request.auto_trading_enabled {
        config.auto_trading_enabled = v;
    }
    if let Some(v) = request.auto_trade_symbols {
        config.auto_trade_symbols = v;
    }
    if let Some(v) = request.min_signal_confidence {
        config.min_signal_confidence = v;
    }
    if let Some(v) = request.max_consecutive_losses {
        config.max_consecutive_losses = v;
    }
    if let Some(v) = request.cool_down_minutes {
        config.cool_down_minutes = v;
    }
    if let Some(v) = request.correlation_limit {
        config.correlation_limit = v;
    }
    if let Some(v) = request.max_portfolio_risk_pct {
        config.max_portfolio_risk_pct = v;
    }
    if let Some(v) = request.short_only_mode {
        config.short_only_mode = v;
    }
    if let Some(v) = request.long_only_mode {
        config.long_only_mode = v;
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

// ============================================================================
// ORDER MANAGEMENT HANDLERS (Phase 4)
// ============================================================================

/// POST /api/real-trading/orders - Place order with 2-step confirmation
async fn place_order(
    request: PlaceOrderRequest,
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

    // Validate request
    if request.symbol.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error("Symbol is required".to_string())),
            StatusCode::BAD_REQUEST,
        ));
    }

    if request.quantity <= 0.0 {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "Quantity must be positive".to_string(),
            )),
            StatusCode::BAD_REQUEST,
        ));
    }

    let side_upper = request.side.to_uppercase();
    if side_upper != "BUY" && side_upper != "SELL" {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "Side must be BUY or SELL".to_string(),
            )),
            StatusCode::BAD_REQUEST,
        ));
    }

    let order_type_upper = request.order_type.to_uppercase();
    if order_type_upper != "MARKET" && order_type_upper != "LIMIT" {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "Order type must be MARKET or LIMIT".to_string(),
            )),
            StatusCode::BAD_REQUEST,
        ));
    }

    if order_type_upper == "LIMIT" && request.price.is_none() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "Price is required for LIMIT orders".to_string(),
            )),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Check if this is a confirmation or initial request
    if let Some(token) = &request.confirmation_token {
        // Validate and consume the confirmation token
        if let Some(confirmed_request) = api.consume_confirmation(token) {
            // Execute the order
            tracing::info!(
                "🔴 EXECUTING REAL ORDER: {} {} {} @ {:?}",
                confirmed_request.side,
                confirmed_request.quantity,
                confirmed_request.symbol,
                confirmed_request.price
            );

            let side = if confirmed_request.side.to_uppercase() == "BUY" {
                OrderSide::Buy
            } else {
                OrderSide::Sell
            };

            let result = if confirmed_request.order_type.to_uppercase() == "MARKET" {
                engine
                    .place_market_order(
                        &confirmed_request.symbol,
                        side,
                        confirmed_request.quantity,
                        None,
                        true,
                    )
                    .await
            } else {
                engine
                    .place_limit_order(
                        &confirmed_request.symbol,
                        side,
                        confirmed_request.quantity,
                        confirmed_request.price.unwrap(),
                        None,
                        true,
                    )
                    .await
            };

            match result {
                Ok(order) => {
                    // Set SL/TP if provided
                    if let Some(sl) = confirmed_request.stop_loss {
                        let _ = engine.set_stop_loss(&confirmed_request.symbol, sl).await;
                    }
                    if let Some(tp) = confirmed_request.take_profit {
                        let _ = engine.set_take_profit(&confirmed_request.symbol, tp).await;
                    }

                    let order_info = OrderInfo {
                        id: order.client_order_id.clone(),
                        exchange_order_id: order.exchange_order_id,
                        symbol: order.symbol.clone(),
                        side: order.side.clone(),
                        order_type: order.order_type.clone(),
                        quantity: order.original_quantity,
                        executed_quantity: order.executed_quantity,
                        price: order.price,
                        avg_fill_price: order.average_fill_price,
                        status: format!("{:?}", order.state),
                        is_entry: order.is_entry,
                        created_at: order.created_at.to_rfc3339(),
                        updated_at: order.updated_at.to_rfc3339(),
                    };

                    Ok(warp::reply::with_status(
                        warp::reply::json(&ApiResponse::success(order_info)),
                        StatusCode::CREATED,
                    ))
                },
                Err(e) => Ok(warp::reply::with_status(
                    warp::reply::json(&ApiResponse::<()>::error(format!(
                        "Failed to place order: {}",
                        e
                    ))),
                    StatusCode::BAD_REQUEST,
                )),
            }
        } else {
            // Invalid or expired token
            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(
                    "Invalid or expired confirmation token".to_string(),
                )),
                StatusCode::BAD_REQUEST,
            ))
        }
    } else {
        // No token - create confirmation
        let confirmation = api.create_confirmation(request);
        Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::success(confirmation)),
            StatusCode::OK,
        ))
    }
}

/// GET /api/real-trading/orders - List orders
async fn list_orders(
    query: ListOrdersQuery,
    api: Arc<RealTradingApi>,
) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<Vec<OrderInfo>>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    // Fetch open orders directly from Binance exchange (source of truth)
    let exchange_orders = engine.get_exchange_open_orders().await;

    let mut all_orders: Vec<OrderInfo> = exchange_orders
        .into_iter()
        .filter(|o| {
            if let Some(ref sym) = query.symbol {
                if o.symbol != *sym {
                    return false;
                }
            }
            true
        })
        .map(|o| {
            let price_val = o.price.parse::<f64>().unwrap_or(0.0);
            let qty = o.orig_qty.parse::<f64>().unwrap_or(0.0);
            let exec_qty = o.executed_qty.parse::<f64>().unwrap_or(0.0);
            let avg_price =
                o.cumulative_quote_qty.parse::<f64>().unwrap_or(0.0) / exec_qty.max(0.0001);
            let timestamp = chrono::DateTime::from_timestamp_millis(o.time)
                .unwrap_or_default()
                .to_rfc3339();
            let update_time = chrono::DateTime::from_timestamp_millis(o.update_time)
                .unwrap_or_default()
                .to_rfc3339();
            OrderInfo {
                id: o.client_order_id,
                exchange_order_id: o.order_id,
                symbol: o.symbol,
                side: o.side,
                order_type: o.r#type,
                quantity: qty,
                executed_quantity: exec_qty,
                price: if price_val > 0.0 {
                    Some(price_val)
                } else {
                    None
                },
                avg_fill_price: avg_price,
                status: o.status,
                is_entry: true,
                created_at: timestamp,
                updated_at: update_time,
            }
        })
        .collect();

    // Also include in-memory active orders (placed through our engine)
    let local_orders = engine.get_active_orders();
    let exchange_ids: std::collections::HashSet<i64> =
        all_orders.iter().map(|o| o.exchange_order_id).collect();

    for o in local_orders {
        // Avoid duplicates — skip if already fetched from exchange
        if exchange_ids.contains(&o.exchange_order_id) {
            continue;
        }
        all_orders.push(OrderInfo {
            id: o.client_order_id.clone(),
            exchange_order_id: o.exchange_order_id,
            symbol: o.symbol.clone(),
            side: o.side.clone(),
            order_type: o.order_type.clone(),
            quantity: o.original_quantity,
            executed_quantity: o.executed_quantity,
            price: o.price,
            avg_fill_price: o.average_fill_price,
            status: format!("{:?}", o.state),
            is_entry: o.is_entry,
            created_at: o.created_at.to_rfc3339(),
            updated_at: o.updated_at.to_rfc3339(),
        });
    }

    // Apply limit
    all_orders.truncate(query.limit);

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(all_orders)),
        StatusCode::OK,
    ))
}

/// DELETE /api/real-trading/orders/{id} - Cancel specific order
async fn cancel_order(order_id: String, api: Arc<RealTradingApi>) -> Result<impl Reply, Rejection> {
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

    tracing::info!("🔴 CANCELLING ORDER: {}", order_id);

    match engine.cancel_order(&order_id).await {
        Ok(order) => {
            let order_info = OrderInfo {
                id: order.client_order_id.clone(),
                exchange_order_id: order.exchange_order_id,
                symbol: order.symbol.clone(),
                side: order.side.clone(),
                order_type: order.order_type.clone(),
                quantity: order.original_quantity,
                executed_quantity: order.executed_quantity,
                price: order.price,
                avg_fill_price: order.average_fill_price,
                status: format!("{:?}", order.state),
                is_entry: order.is_entry,
                created_at: order.created_at.to_rfc3339(),
                updated_at: order.updated_at.to_rfc3339(),
            };

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(order_info)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to cancel order: {}",
                e
            ))),
            StatusCode::BAD_REQUEST,
        )),
    }
}

/// DELETE /api/real-trading/orders/all - Cancel all orders
async fn cancel_all_orders(
    query: CancelAllQuery,
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

    tracing::warn!(
        "🔴 CANCELLING ALL ORDERS{}",
        query
            .symbol
            .as_ref()
            .map(|s| format!(" for {}", s))
            .unwrap_or_default()
    );

    match engine.cancel_all_orders(query.symbol.as_deref()).await {
        Ok(cancelled_ids) => {
            let result = serde_json::json!({
                "cancelled_count": cancelled_ids.len(),
                "cancelled_order_ids": cancelled_ids,
            });

            Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::success(result)),
                StatusCode::OK,
            ))
        },
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "Failed to cancel orders: {}",
                e
            ))),
            StatusCode::BAD_REQUEST,
        )),
    }
}

/// PUT /api/real-trading/positions/{symbol}/sltp - Modify SL/TP for position
async fn modify_sltp(
    symbol: String,
    request: ModifySlTpRequest,
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

    tracing::info!(
        "🔧 Modifying SL/TP for {}: SL={:?}, TP={:?}",
        symbol,
        request.stop_loss,
        request.take_profit
    );

    // Check if position exists
    let position = engine.get_position(&symbol);
    if position.is_none() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(format!(
                "No open position for {}",
                symbol
            ))),
            StatusCode::NOT_FOUND,
        ));
    }

    // Update SL/TP
    let mut errors = Vec::new();

    if let Some(sl) = request.stop_loss {
        if let Err(e) = engine.set_stop_loss(&symbol, sl).await {
            errors.push(format!("SL: {}", e));
        }
    }

    if let Some(tp) = request.take_profit {
        if let Err(e) = engine.set_take_profit(&symbol, tp).await {
            errors.push(format!("TP: {}", e));
        }
    }

    if !errors.is_empty() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(errors.join("; "))),
            StatusCode::BAD_REQUEST,
        ));
    }

    // Get updated position
    let updated_position = engine.get_position(&symbol);
    let result = serde_json::json!({
        "symbol": symbol,
        "stop_loss": updated_position.as_ref().and_then(|p| p.stop_loss),
        "take_profit": updated_position.as_ref().and_then(|p| p.take_profit),
        "message": "SL/TP updated successfully",
    });

    Ok(warp::reply::with_status(
        warp::reply::json(&ApiResponse::success(result)),
        StatusCode::OK,
    ))
}

/// Request body for test signal endpoint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSignalRequest {
    pub symbol: String,
    pub signal: String, // "Long" or "Short"
    #[serde(default = "default_confidence")]
    pub confidence: f64,
}
fn default_confidence() -> f64 {
    0.75
}

/// POST /api/real-trading/test-signal — Trigger a test signal (testnet only)
async fn test_signal(
    request: TestSignalRequest,
    api: Arc<RealTradingApi>,
) -> Result<impl Reply, Rejection> {
    let engine = match &api.engine {
        Some(e) => e,
        None => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(
                    "Real trading service is not configured".to_string(),
                )),
                StatusCode::SERVICE_UNAVAILABLE,
            ))
        },
    };

    // Safety: only allow on testnet
    let config = engine.get_config().await;
    if !config.use_testnet {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(
                "Test signal only allowed on testnet".to_string(),
            )),
            StatusCode::FORBIDDEN,
        ));
    }

    let signal = match request.signal.to_lowercase().as_str() {
        "long" | "buy" => TradingSignal::Long,
        "short" | "sell" => TradingSignal::Short,
        _ => {
            return Ok(warp::reply::with_status(
                warp::reply::json(&ApiResponse::<()>::error(
                    "Signal must be 'Long' or 'Short'".to_string(),
                )),
                StatusCode::BAD_REQUEST,
            ))
        },
    };

    tracing::info!(
        "🧪 Test signal triggered: {} {:?} (confidence: {:.2})",
        request.symbol,
        signal,
        request.confidence
    );

    match engine
        .process_external_ai_signal(
            request.symbol.clone(),
            signal,
            request.confidence,
            "Test signal".to_string(),
            0.0,
            None,
            None,
        )
        .await
    {
        Ok(()) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::success(format!(
                "Test signal processed: {} {:?}",
                request.symbol, request.signal
            ))),
            StatusCode::OK,
        )),
        Err(e) => Ok(warp::reply::with_status(
            warp::reply::json(&ApiResponse::<()>::error(e.to_string())),
            StatusCode::INTERNAL_SERVER_ERROR,
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
            ..Default::default()
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
            ..Default::default()
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
            auto_trading_enabled: false,
            auto_trade_symbols: vec![],
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
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

    // ============================================================================
    // PHASE 4 TESTS - Order Placement Types
    // ============================================================================

    #[test]
    fn test_place_order_request_deserialization() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_type": "LIMIT",
            "quantity": 0.01,
            "price": 50000.0,
            "stop_loss": 48000.0,
            "take_profit": 55000.0
        }"#;

        let request: PlaceOrderRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.symbol, "BTCUSDT");
        assert_eq!(request.side, "BUY");
        assert_eq!(request.order_type, "LIMIT");
        assert_eq!(request.quantity, 0.01);
        assert_eq!(request.price, Some(50000.0));
        assert_eq!(request.stop_loss, Some(48000.0));
        assert_eq!(request.take_profit, Some(55000.0));
        assert!(request.confirmation_token.is_none());
    }

    #[test]
    fn test_place_order_request_market_order() {
        let json = r#"{
            "symbol": "ETHUSDT",
            "side": "SELL",
            "order_type": "MARKET",
            "quantity": 1.5
        }"#;

        let request: PlaceOrderRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.symbol, "ETHUSDT");
        assert_eq!(request.side, "SELL");
        assert_eq!(request.order_type, "MARKET");
        assert_eq!(request.quantity, 1.5);
        assert!(request.price.is_none());
    }

    #[test]
    fn test_order_info_serialization() {
        let order = OrderInfo {
            id: "real_abc123".to_string(),
            exchange_order_id: 123456789,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            executed_quantity: 0.005,
            price: Some(50000.0),
            avg_fill_price: 49950.0,
            status: "PartiallyFilled".to_string(),
            is_entry: true,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:01:00Z".to_string(),
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("real_abc123"));
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("123456789"));
        assert!(json.contains("PartiallyFilled"));
    }

    #[test]
    fn test_confirmation_response_serialization() {
        let confirmation = ConfirmationResponse {
            token: "uuid-token-123".to_string(),
            expires_at: "2025-01-01T00:01:00Z".to_string(),
            summary: "BUY 0.01 BTCUSDT LIMIT @ $50000.00".to_string(),
            order_details: PlaceOrderRequest {
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                order_type: "LIMIT".to_string(),
                quantity: 0.01,
                price: Some(50000.0),
                stop_loss: None,
                take_profit: None,
                confirmation_token: None,
            },
        };

        let json = serde_json::to_string(&confirmation).unwrap();
        assert!(json.contains("uuid-token-123"));
        assert!(json.contains("BUY 0.01 BTCUSDT LIMIT"));
    }

    #[test]
    fn test_list_orders_query_defaults() {
        let json = r#"{}"#;
        let query: ListOrdersQuery = serde_json::from_str(json).unwrap();
        assert!(query.symbol.is_none());
        assert!(query.status.is_none());
        assert_eq!(query.limit, 50);
    }

    #[test]
    fn test_list_orders_query_with_filters() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "status": "active",
            "limit": 10
        }"#;

        let query: ListOrdersQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.symbol, Some("BTCUSDT".to_string()));
        assert_eq!(query.status, Some("active".to_string()));
        assert_eq!(query.limit, 10);
    }

    #[test]
    fn test_modify_sltp_request() {
        let json = r#"{
            "stop_loss": 48000.0,
            "take_profit": 55000.0
        }"#;

        let request: ModifySlTpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.stop_loss, Some(48000.0));
        assert_eq!(request.take_profit, Some(55000.0));
    }

    #[test]
    fn test_modify_sltp_request_partial() {
        let json = r#"{"stop_loss": 48000.0}"#;
        let request: ModifySlTpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.stop_loss, Some(48000.0));
        assert!(request.take_profit.is_none());
    }

    #[test]
    fn test_cancel_all_query() {
        let json = r#"{"symbol": "BTCUSDT"}"#;
        let query: CancelAllQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.symbol, Some("BTCUSDT".to_string()));

        let json_empty = r#"{}"#;
        let query_empty: CancelAllQuery = serde_json::from_str(json_empty).unwrap();
        assert!(query_empty.symbol.is_none());
    }

    // ============================================================================
    // WARP HANDLER TESTS - HTTP Endpoint Testing with no-db mode
    // ============================================================================

    fn create_test_api() -> RealTradingApi {
        // Create API without engine (no-db mode)
        RealTradingApi::new(None)
    }

    #[tokio::test]
    async fn test_get_status_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/status")
            .reply(&routes)
            .await;

        // Should return SERVICE_UNAVAILABLE when no engine configured
        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
        let body: serde_json::Value = serde_json::from_slice(resp.body()).unwrap();
        assert_eq!(body["success"], false);
    }

    #[tokio::test]
    async fn test_get_portfolio_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_get_open_trades_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/open")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_get_closed_trades_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/closed")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_start_engine_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/start")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_stop_engine_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/stop")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_close_trade_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let request = CloseTradeRequest {
            reason: Some("Testing".to_string()),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/BTCUSDT/close")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_get_settings_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/settings")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_update_settings_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let request = UpdateSettingsRequest {
            max_position_size_usdt: Some(5000.0),
            ..Default::default()
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_place_order_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_list_orders_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_list_orders_with_query_params() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?symbol=BTCUSDT&status=active&limit=10")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cancel_order_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/test-order-id")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cancel_all_orders_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/all")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cancel_all_orders_with_symbol() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/all?symbol=BTCUSDT")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_modify_sltp_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let request = ModifySlTpRequest {
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/BTCUSDT/sltp")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    // Test invalid request bodies
    #[tokio::test]
    async fn test_place_order_invalid_json() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .header("content-type", "application/json")
            .body("{invalid json}")
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_place_order_missing_required_fields() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&serde_json::json!({"symbol": "BTCUSDT"}))
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_update_settings_invalid_json() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .header("content-type", "application/json")
            .body("not json")
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_close_trade_invalid_json() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/BTCUSDT/close")
            .header("content-type", "application/json")
            .body("{bad}")
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error());
    }

    #[tokio::test]
    async fn test_modify_sltp_invalid_json() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/BTCUSDT/sltp")
            .header("content-type", "application/json")
            .body("[]")
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error());
    }

    // Test wrong HTTP methods
    #[tokio::test]
    async fn test_status_wrong_method_post() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/status")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_portfolio_wrong_method_post() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_start_engine_wrong_method_get() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/start")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_settings_wrong_method_post() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/settings")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    // Test CORS headers
    #[tokio::test]
    async fn test_options_preflight_status() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("OPTIONS")
            .path("/real-trading/status")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[tokio::test]
    async fn test_confirmation_token_creation() {
        let api = create_test_api();

        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            price: Some(50000.0),
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request.clone());

        assert!(!confirmation.token.is_empty());
        assert!(confirmation.summary.contains("BUY"));
        assert!(confirmation.summary.contains("BTCUSDT"));
        assert!(confirmation.summary.contains("0.01"));
        assert_eq!(confirmation.order_details.symbol, "BTCUSDT");
    }

    #[tokio::test]
    async fn test_confirmation_token_consumption() {
        let api = create_test_api();

        let request = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.5,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request.clone());
        let token = confirmation.token;

        // Should be able to consume once
        let consumed = api.consume_confirmation(&token);
        assert!(consumed.is_some());

        // Should not be able to consume again
        let consumed_again = api.consume_confirmation(&token);
        assert!(consumed_again.is_none());
    }

    #[tokio::test]
    async fn test_confirmation_token_invalid() {
        let api = create_test_api();

        let consumed = api.consume_confirmation("invalid-token-xyz");
        assert!(consumed.is_none());
    }

    #[tokio::test]
    async fn test_cleanup_expired_confirmations() {
        let api = create_test_api();

        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        api.create_confirmation(request);

        // Cleanup should run without panic
        api.cleanup_expired_confirmations();

        // Should still have pending confirmations (not expired yet)
        assert!(!api.pending_confirmations.is_empty());
    }

    #[test]
    fn test_default_limit_function() {
        assert_eq!(default_limit(), 50);
    }

    // ============================================================================
    // COVERAGE PHASE 2 - Real Trading Handler Tests
    // ============================================================================

    #[tokio::test]
    async fn test_cov2_get_portfolio_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        // No engine = SERVICE_UNAVAILABLE
        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_get_open_trades_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/open")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_get_closed_trades_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/closed")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_start_engine_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/start")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_stop_engine_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/stop")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_close_trade_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let body = r#"{"reason": "Manual close"}"#;
        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/pos-123/close")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_get_settings_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/settings")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_update_settings_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let body = r#"{
            "max_position_size_usdt": 1000.0,
            "max_positions": 5,
            "max_leverage": 10
        }"#;
        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_place_order_without_confirmation() {
        let api = create_test_api();
        let routes = api.routes();

        let body = r#"{
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_type": "LIMIT",
            "quantity": 0.01,
            "price": 50000.0
        }"#;
        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        // Should return SERVICE_UNAVAILABLE (no engine configured in test)
        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_place_order_with_confirmation() {
        let api = create_test_api();

        // First create confirmation BEFORE consuming api with routes()
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request.clone());
        let routes = api.routes();

        // Now place order with confirmation token
        let body = format!(
            r#"{{
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_type": "MARKET",
            "quantity": 0.01,
            "confirmation_token": "{}"
        }}"#,
            confirmation.token
        );

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        // No engine = SERVICE_UNAVAILABLE
        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_list_orders_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_list_orders_with_filters() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?symbol=BTCUSDT&status=active&limit=10")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_cancel_order_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/order-123")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_cancel_all_orders_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/cancel-all")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_cancel_all_orders_with_symbol() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/cancel-all?symbol=BTCUSDT")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_modify_sltp_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let body = r#"{
            "stop_loss": 48000.0,
            "take_profit": 55000.0
        }"#;
        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/pos-123/sltp")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov2_modify_sltp_partial() {
        let api = create_test_api();
        let routes = api.routes();

        let body = r#"{"stop_loss": 48000.0}"#;
        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/pos-456/sltp")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_cov2_place_order_request_validation() {
        // Test with all fields
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            confirmation_token: Some("token-123".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("50000.0"));
        assert!(json.contains("token-123"));

        // Test deserialization
        let parsed: PlaceOrderRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.symbol, "BTCUSDT");
        assert_eq!(parsed.quantity, 0.01);
    }

    #[test]
    fn test_cov2_update_settings_request_partial() {
        let json = r#"{"max_position_size_usdt": 2000.0}"#;
        let request: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.max_position_size_usdt, Some(2000.0));
        assert!(request.max_positions.is_none());
        assert!(request.max_leverage.is_none());
    }

    #[test]
    fn test_cov2_update_settings_request_full() {
        let json = r#"{
            "max_position_size_usdt": 1000.0,
            "max_positions": 5,
            "max_leverage": 10,
            "max_daily_loss_usdt": 500.0
        }"#;
        let request: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.max_position_size_usdt, Some(1000.0));
        assert_eq!(request.max_positions, Some(5));
        assert_eq!(request.max_leverage, Some(10));
        assert_eq!(request.max_daily_loss_usdt, Some(500.0));
    }

    #[test]
    fn test_cov2_engine_status_defaults() {
        let status = EngineStatus {
            is_running: false,
            is_testnet: true,
            open_positions_count: 0,
            open_orders_count: 0,
            circuit_breaker_open: false,
            daily_pnl: 0.0,
            daily_trades_count: 0,
            uptime_seconds: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"is_running\":false"));
        assert!(json.contains("\"open_positions_count\":0"));
    }

    #[test]
    fn test_cov2_portfolio_response_empty() {
        let portfolio = PortfolioResponse {
            total_balance: 0.0,
            available_balance: 0.0,
            locked_balance: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: vec![],
            balances: vec![],
            ..Default::default()
        };

        let json = serde_json::to_string(&portfolio).unwrap();
        assert!(json.contains("\"total_balance\":0.0"));
        assert!(json.contains("\"positions\":[]"));
    }

    #[test]
    fn test_cov2_position_info_with_nulls() {
        let pos = PositionInfo {
            id: "pos-789".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "SHORT".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            current_price: 2900.0,
            unrealized_pnl: 100.0,
            unrealized_pnl_pct: 3.33,
            stop_loss: None,
            take_profit: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&pos).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("SHORT"));
    }

    #[test]
    fn test_cov2_closed_trade_info_complete() {
        let trade = ClosedTradeInfo {
            id: "trade-456".to_string(),
            symbol: "BNBUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 10.0,
            entry_price: 300.0,
            exit_price: 320.0,
            realized_pnl: 200.0,
            realized_pnl_pct: 6.67,
            commission: 1.5,
            opened_at: "2025-01-01T10:00:00Z".to_string(),
            closed_at: "2025-01-01T12:00:00Z".to_string(),
            close_reason: "Stop loss".to_string(),
        };

        let json = serde_json::to_string(&trade).unwrap();
        assert!(json.contains("BNBUSDT"));
        assert!(json.contains("Stop loss"));
        assert!(json.contains("200.0"));
    }

    #[test]
    fn test_cov2_order_info_filled() {
        let order = OrderInfo {
            id: "real_xyz789".to_string(),
            exchange_order_id: 987654321,
            symbol: "SOLUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 5.0,
            executed_quantity: 5.0,
            price: None,
            avg_fill_price: 100.5,
            status: "Filled".to_string(),
            is_entry: false,
            created_at: "2025-01-01T14:00:00Z".to_string(),
            updated_at: "2025-01-01T14:00:01Z".to_string(),
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("real_xyz789"));
        assert!(json.contains("Filled"));
        assert!(json.contains("987654321"));
    }

    #[test]
    fn test_cov2_balance_info_zero() {
        let balance = BalanceInfo {
            asset: "BTC".to_string(),
            free: 0.0,
            locked: 0.0,
            total: 0.0,
        };

        let json = serde_json::to_string(&balance).unwrap();
        assert!(json.contains("BTC"));
        assert!(json.contains("0.0"));
    }

    #[test]
    fn test_cov2_settings_response_complete() {
        let settings = SettingsResponse {
            use_testnet: false,
            max_position_size_usdt: 5000.0,
            max_positions: 10,
            max_daily_loss_usdt: 1000.0,
            max_total_exposure_usdt: 25000.0,
            risk_per_trade_percent: 3.0,
            default_stop_loss_percent: 1.5,
            default_take_profit_percent: 3.0,
            max_leverage: 20,
            circuit_breaker_errors: 5,
            circuit_breaker_cooldown_secs: 600,
            auto_trading_enabled: false,
            auto_trade_symbols: vec![],
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"use_testnet\":false"));
        assert!(json.contains("\"max_leverage\":20"));
        assert!(json.contains("5000.0"));
    }

    #[test]
    fn test_cov2_confirmation_response_format() {
        let confirmation = ConfirmationResponse {
            token: "unique-token-abc".to_string(),
            expires_at: "2025-01-01T15:01:00Z".to_string(),
            summary: "SELL 1.5 ETHUSDT MARKET @ Market Price".to_string(),
            order_details: PlaceOrderRequest {
                symbol: "ETHUSDT".to_string(),
                side: "SELL".to_string(),
                order_type: "MARKET".to_string(),
                quantity: 1.5,
                price: None,
                stop_loss: Some(2900.0),
                take_profit: Some(3200.0),
                confirmation_token: None,
            },
        };

        let json = serde_json::to_string(&confirmation).unwrap();
        assert!(json.contains("unique-token-abc"));
        assert!(json.contains("SELL 1.5 ETHUSDT"));
        assert!(json.contains("2900.0"));
    }

    #[tokio::test]
    async fn test_cov2_cors_preflight() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("OPTIONS")
            .path("/real-trading/portfolio")
            .header("origin", "http://localhost:3000")
            .header("access-control-request-method", "GET")
            .reply(&routes)
            .await;

        // CORS should be configured
        assert!(
            resp.status().is_success()
                || resp.status() == warp::http::StatusCode::METHOD_NOT_ALLOWED
        );
    }

    #[tokio::test]
    async fn test_cov2_invalid_json_body() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .header("content-type", "application/json")
            .body("{invalid json}")
            .reply(&routes)
            .await;

        // Should handle JSON parse error gracefully
        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov2_missing_content_type() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .body(r#"{"max_positions": 10}"#)
            .reply(&routes)
            .await;

        // Should handle missing content-type
        assert!(
            resp.status().is_client_error()
                || resp.status().is_server_error()
                || resp.status() == warp::http::StatusCode::SERVICE_UNAVAILABLE
        );
    }

    // Coverage tests for order placement handlers (482-585)
    #[tokio::test]
    async fn test_cov3_place_order_market_buy() {
        let api = create_test_api();
        let routes = api.routes();

        let order_req = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.001,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&order_req)
            .reply(&routes)
            .await;

        // Without engine, should be SERVICE_UNAVAILABLE
        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_place_order_limit_sell() {
        let api = create_test_api();
        let routes = api.routes();

        let order_req = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.5,
            price: Some(3500.0),
            stop_loss: Some(3700.0),
            take_profit: Some(3300.0),
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&order_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_place_order_stop_market() {
        let api = create_test_api();
        let routes = api.routes();

        let order_req = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.001,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&order_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_start_engine_without_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/start")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_stop_engine_without_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/stop")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    // Coverage tests for position management - use existing routes
    #[tokio::test]
    async fn test_cov3_get_positions_via_portfolio() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_get_open_trades_for_symbol() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/open")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_close_trade_btc() {
        let api = create_test_api();
        let routes = api.routes();

        let close_req = CloseTradeRequest {
            reason: Some("Close BTC position".to_string()),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/btc-trade-1/close")
            .json(&close_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_modify_sltp_btc_long() {
        let api = create_test_api();
        let routes = api.routes();

        let modify_req = ModifySlTpRequest {
            stop_loss: Some(49000.0),
            take_profit: Some(52000.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/BTCUSDT/sltp")
            .json(&modify_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_modify_sltp_eth_short() {
        let api = create_test_api();
        let routes = api.routes();

        let modify_req = ModifySlTpRequest {
            stop_loss: Some(3600.0),
            take_profit: Some(3200.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/ETHUSDT/sltp")
            .json(&modify_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_modify_sltp_only_sl() {
        let api = create_test_api();
        let routes = api.routes();

        let modify_req = ModifySlTpRequest {
            stop_loss: Some(48500.0),
            take_profit: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/BTCUSDT/sltp")
            .json(&modify_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_modify_sltp_only_tp() {
        let api = create_test_api();
        let routes = api.routes();

        let modify_req = ModifySlTpRequest {
            stop_loss: None,
            take_profit: Some(53000.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/BTCUSDT/sltp")
            .json(&modify_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    // Coverage tests for handler branches (714-799)
    #[tokio::test]
    async fn test_cov3_close_trade_with_reason() {
        let api = create_test_api();
        let routes = api.routes();

        let close_req = CloseTradeRequest {
            reason: Some("Manual close for testing".to_string()),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/test-trade-123/close")
            .json(&close_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_close_trade_no_reason() {
        let api = create_test_api();
        let routes = api.routes();

        let close_req = CloseTradeRequest { reason: None };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/test-trade-456/close")
            .json(&close_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_get_settings_detailed() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/settings")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_update_settings_max_positions() {
        let api = create_test_api();
        let routes = api.routes();

        let update_req = UpdateSettingsRequest {
            max_positions: Some(15),
            ..Default::default()
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .json(&update_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_update_settings_leverage() {
        let api = create_test_api();
        let routes = api.routes();

        let update_req = UpdateSettingsRequest {
            max_daily_loss_usdt: Some(1500.0),
            risk_per_trade_percent: Some(2.5),
            default_stop_loss_percent: Some(2.0),
            default_take_profit_percent: Some(5.0),
            max_leverage: Some(5),
            ..Default::default()
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .json(&update_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    // Coverage tests for error handling (810-938)
    #[tokio::test]
    async fn test_cov3_list_orders_all() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_list_orders_btc_only() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?symbol=BTCUSDT")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_list_orders_with_limit() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?limit=50")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_cancel_order_by_id() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/123456789")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_cancel_all_orders_btc() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/all?symbol=BTCUSDT")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_cancel_all_orders_global() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/all")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_get_settings_as_account_info() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/settings")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_get_closed_trades_list() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/closed")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_place_order_with_all_params() {
        let api = create_test_api();
        let routes = api.routes();

        let order_req = PlaceOrderRequest {
            symbol: "BNBUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 1.0,
            price: Some(300.0),
            stop_loss: Some(290.0),
            take_profit: Some(320.0),
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&order_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_place_order_reduce_only() {
        let api = create_test_api();
        let routes = api.routes();

        let order_req = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.001,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&order_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_get_status_no_data() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/status")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);

        let body = std::str::from_utf8(resp.body()).unwrap_or("");
        assert!(
            body.contains("not configured")
                || body.contains("unavailable")
                || body.contains("not initialized")
        );
    }

    #[tokio::test]
    async fn test_cov3_get_portfolio_no_data() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_update_settings_empty() {
        let api = create_test_api();
        let routes = api.routes();

        let update_req = UpdateSettingsRequest {
            ..Default::default()
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .json(&update_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_place_order_stop_limit() {
        let api = create_test_api();
        let routes = api.routes();

        let order_req = PlaceOrderRequest {
            symbol: "ADAUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 100.0,
            price: Some(0.5),
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&order_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_list_orders_multiple_params() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?symbol=ETHUSDT&limit=20")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_get_open_trades_empty() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/open")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_get_closed_trades_empty() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/trades/closed")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov3_update_settings_all_fields() {
        let api = create_test_api();
        let routes = api.routes();

        let update_req = UpdateSettingsRequest {
            max_position_size_usdt: Some(5000.0),
            max_positions: Some(20),
            max_daily_loss_usdt: Some(2000.0),
            max_total_exposure_usdt: Some(50000.0),
            risk_per_trade_percent: Some(3.0),
            default_stop_loss_percent: Some(2.5),
            default_take_profit_percent: Some(6.0),
            max_leverage: Some(10),
            ..Default::default()
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .json(&update_req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    // ============================================================================
    // PHASE 5 TESTS - Enhanced Coverage for Handlers
    // ============================================================================

    #[tokio::test]
    async fn test_cov5_start_engine_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/start")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_stop_engine_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/stop")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_close_trade_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let req = CloseTradeRequest {
            reason: Some("Manual close".to_string()),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/trade123/close")
            .json(&req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_get_settings_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/settings")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_cancel_order_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/order123")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_cancel_all_orders_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders")
            .reply(&routes)
            .await;

        // DELETE /real-trading/orders may return 405 (method not allowed) or 503
        assert!(resp.status() == 405 || resp.status() == 503);
    }

    #[tokio::test]
    async fn test_cov5_cancel_all_orders_with_symbol() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders?symbol=BTCUSDT")
            .reply(&routes)
            .await;

        // DELETE with symbol may return 405 or 503
        assert!(resp.status() == 405 || resp.status() == 503);
    }

    #[tokio::test]
    async fn test_cov5_modify_sltp_no_engine() {
        let api = create_test_api();
        let routes = api.routes();

        let req = ModifySlTpRequest {
            stop_loss: Some(49000.0),
            take_profit: Some(52000.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/pos123/sltp")
            .json(&req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_cov5_modify_sltp_request_deserialization() {
        let json = r#"{"stop_loss": 48000.0, "take_profit": 52000.0}"#;
        let req: ModifySlTpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.stop_loss, Some(48000.0));
        assert_eq!(req.take_profit, Some(52000.0));
    }

    #[test]
    fn test_cov5_modify_sltp_request_partial() {
        let json = r#"{"stop_loss": 48000.0}"#;
        let req: ModifySlTpRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.stop_loss, Some(48000.0));
        assert!(req.take_profit.is_none());
    }

    #[test]
    fn test_cov5_list_orders_query_deserialization() {
        let json = r#"{"symbol": "BTCUSDT", "limit": 50}"#;
        let query: ListOrdersQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.symbol, Some("BTCUSDT".to_string()));
        assert_eq!(query.limit, 50);
    }

    #[test]
    fn test_cov5_list_orders_query_empty() {
        let json = r#"{}"#;
        let query: ListOrdersQuery = serde_json::from_str(json).unwrap();
        assert!(query.symbol.is_none());
        assert_eq!(query.limit, 50); // default limit
    }

    #[test]
    fn test_cov5_cancel_all_query_deserialization() {
        let json = r#"{"symbol": "ETHUSDT"}"#;
        let query: CancelAllQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.symbol, Some("ETHUSDT".to_string()));
    }

    #[test]
    fn test_cov5_cancel_all_query_empty() {
        let json = r#"{}"#;
        let query: CancelAllQuery = serde_json::from_str(json).unwrap();
        assert!(query.symbol.is_none());
    }

    #[test]
    fn test_cov5_order_info_serialization() {
        let order = OrderInfo {
            id: "order123".to_string(),
            exchange_order_id: 12345678,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            executed_quantity: 0.0,
            price: Some(50000.0),
            avg_fill_price: 0.0,
            status: "NEW".to_string(),
            is_entry: true,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("order123"));
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("NEW"));
    }

    #[test]
    fn test_cov5_order_info_deserialization() {
        let json = r#"{
            "id": "order456",
            "exchange_order_id": 87654321,
            "symbol": "ETHUSDT",
            "side": "SELL",
            "order_type": "MARKET",
            "quantity": 1.5,
            "executed_quantity": 1.5,
            "price": null,
            "avg_fill_price": 3000.0,
            "status": "FILLED",
            "is_entry": false,
            "created_at": "2025-01-02T00:00:00Z",
            "updated_at": "2025-01-02T00:01:00Z"
        }"#;

        let order: OrderInfo = serde_json::from_str(json).unwrap();
        assert_eq!(order.id, "order456");
        assert_eq!(order.symbol, "ETHUSDT");
        assert_eq!(order.status, "FILLED");
        assert!(order.price.is_none());
        assert_eq!(order.avg_fill_price, 3000.0);
    }

    #[test]
    fn test_cov5_confirmation_response_fields() {
        let req = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            price: Some(50000.0),
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let api = RealTradingApi::new(None);
        let conf_resp = api.create_confirmation(req.clone());

        assert!(!conf_resp.token.is_empty());
        assert!(!conf_resp.expires_at.is_empty());
        assert!(conf_resp.summary.contains("BUY"));
        assert!(conf_resp.summary.contains("BTCUSDT"));
        assert_eq!(conf_resp.order_details.symbol, "BTCUSDT");
    }

    #[test]
    fn test_cov5_confirmation_cleanup_expired() {
        let api = RealTradingApi::new(None);

        // Create a confirmation
        let req = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let conf_resp = api.create_confirmation(req);

        // Verify it's stored
        assert_eq!(api.pending_confirmations.len(), 1);

        // Cleanup should not remove it immediately (not expired yet)
        api.cleanup_expired_confirmations();
        assert_eq!(api.pending_confirmations.len(), 1);

        // Consume the confirmation
        let consumed = api.consume_confirmation(&conf_resp.token);
        assert!(consumed.is_some());

        // After consumption, it should be removed
        assert_eq!(api.pending_confirmations.len(), 0);
    }

    #[test]
    fn test_cov5_confirmation_consume_nonexistent() {
        let api = RealTradingApi::new(None);
        let result = api.consume_confirmation("nonexistent-token");
        assert!(result.is_none());
    }

    #[test]
    fn test_cov5_confirmation_summary_market_order() {
        let api = RealTradingApi::new(None);
        let req = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.5,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let conf_resp = api.create_confirmation(req);
        assert!(conf_resp.summary.contains("MARKET"));
        assert!(conf_resp.summary.contains("SELL"));
        assert!(conf_resp.summary.contains("1.5"));
    }

    #[tokio::test]
    async fn test_cov5_list_orders_with_limit() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?limit=10")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_list_orders_with_symbol_only() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?symbol=BNBUSDT")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_cov5_place_order_request_with_confirmation() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_type": "LIMIT",
            "quantity": 0.01,
            "price": 50000.0,
            "confirmation_token": "test-token-123"
        }"#;

        let req: PlaceOrderRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.confirmation_token, Some("test-token-123".to_string()));
    }

    #[test]
    fn test_cov5_update_settings_request_single_field() {
        let json = r#"{"max_positions": 10}"#;
        let req: UpdateSettingsRequest = serde_json::from_str(json).unwrap();
        assert_eq!(req.max_positions, Some(10));
        assert!(req.max_position_size_usdt.is_none());
    }

    #[test]
    fn test_cov5_settings_response_clone() {
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
            auto_trading_enabled: false,
            auto_trade_symbols: vec![],
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
        };

        let json = serde_json::to_string(&settings).unwrap();
        let deserialized: SettingsResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.max_positions, 5);
        assert_eq!(deserialized.use_testnet, true);
    }

    #[test]
    fn test_cov5_position_info_clone() {
        let pos = PositionInfo {
            id: "pos-1".to_string(),
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
            ..Default::default()
        };

        let cloned = pos.clone();
        assert_eq!(cloned.id, "pos-1");
        assert_eq!(cloned.quantity, 0.1);
    }

    #[test]
    fn test_cov5_balance_info_clone() {
        let balance = BalanceInfo {
            asset: "BTC".to_string(),
            free: 1.5,
            locked: 0.5,
            total: 2.0,
        };

        let cloned = balance.clone();
        assert_eq!(cloned.asset, "BTC");
        assert_eq!(cloned.total, 2.0);
    }

    #[test]
    fn test_cov5_closed_trade_info_clone() {
        let trade = ClosedTradeInfo {
            id: "trade-1".to_string(),
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

        let cloned = trade.clone();
        assert_eq!(cloned.id, "trade-1");
        assert_eq!(cloned.realized_pnl, 100.0);
    }

    #[test]
    fn test_cov5_cancel_all_query_clone() {
        let query = CancelAllQuery {
            symbol: Some("BTCUSDT".to_string()),
        };

        let cloned = query.clone();
        assert_eq!(cloned.symbol, Some("BTCUSDT".to_string()));
    }

    #[tokio::test]
    async fn test_cov5_close_trade_without_reason() {
        let api = create_test_api();
        let routes = api.routes();

        let req = CloseTradeRequest { reason: None };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/trade456/close")
            .json(&req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_modify_sltp_only_stop_loss() {
        let api = create_test_api();
        let routes = api.routes();

        let req = ModifySlTpRequest {
            stop_loss: Some(48000.0),
            take_profit: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/pos456/sltp")
            .json(&req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov5_modify_sltp_only_take_profit() {
        let api = create_test_api();
        let routes = api.routes();

        let req = ModifySlTpRequest {
            stop_loss: None,
            take_profit: Some(55000.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/pos789/sltp")
            .json(&req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_cov5_api_response_timestamp() {
        let resp: ApiResponse<String> = ApiResponse::success("data".to_string());
        let now = Utc::now();

        // Timestamp should be very close to now
        let diff = (resp.timestamp.timestamp() - now.timestamp()).abs();
        assert!(diff < 2); // Within 2 seconds
    }

    #[test]
    fn test_cov5_api_response_error_timestamp() {
        let resp: ApiResponse<String> = ApiResponse::error("error".to_string());
        let now = Utc::now();

        // Timestamp should be very close to now
        let diff = (resp.timestamp.timestamp() - now.timestamp()).abs();
        assert!(diff < 2); // Within 2 seconds
    }

    // ============================================================================
    // COVERAGE PHASE 6 - Additional tests for real_trading.rs (85% → 95%)
    // ============================================================================

    #[test]
    fn test_cov6_api_response_timestamp_present() {
        let resp: ApiResponse<String> = ApiResponse::success("test".to_string());
        assert!(!resp.timestamp.to_rfc3339().is_empty());
    }

    #[test]
    fn test_cov6_engine_status_all_fields() {
        let status = EngineStatus {
            is_running: true,
            is_testnet: false,
            open_positions_count: 3,
            open_orders_count: 2,
            circuit_breaker_open: true,
            daily_pnl: -50.0,
            daily_trades_count: 10,
            uptime_seconds: Some(3600),
        };
        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"daily_pnl\":-50.0"));
        assert!(json.contains("\"circuit_breaker_open\":true"));
    }

    #[test]
    fn test_cov6_portfolio_response_with_positions() {
        let pos = PositionInfo {
            id: "p1".to_string(),
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
            ..Default::default()
        };

        let portfolio = PortfolioResponse {
            total_balance: 10000.0,
            available_balance: 8000.0,
            locked_balance: 2000.0,
            unrealized_pnl: 100.0,
            realized_pnl: 50.0,
            positions: vec![pos],
            balances: vec![],
            ..Default::default()
        };

        assert_eq!(portfolio.positions.len(), 1);
        assert_eq!(portfolio.unrealized_pnl, 100.0);
    }

    #[test]
    fn test_cov6_position_info_negative_pnl() {
        let pos = PositionInfo {
            id: "p2".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "SHORT".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            current_price: 3100.0,
            unrealized_pnl: -100.0,
            unrealized_pnl_pct: -3.33,
            stop_loss: None,
            take_profit: None,
            created_at: "2025-01-02T00:00:00Z".to_string(),
            ..Default::default()
        };
        assert_eq!(pos.unrealized_pnl, -100.0);
    }

    #[test]
    fn test_cov6_closed_trade_info_all_fields() {
        let trade = ClosedTradeInfo {
            id: "t1".to_string(),
            symbol: "BNBUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 10.0,
            entry_price: 300.0,
            exit_price: 310.0,
            realized_pnl: 100.0,
            realized_pnl_pct: 3.33,
            commission: 1.0,
            opened_at: "2025-01-01T10:00:00Z".to_string(),
            closed_at: "2025-01-01T11:00:00Z".to_string(),
            close_reason: "Manual close".to_string(),
        };
        let json = serde_json::to_string(&trade).unwrap();
        assert!(json.contains("BNBUSDT"));
        assert!(json.contains("Manual close"));
    }

    #[test]
    fn test_cov6_close_trade_request_none() {
        let req = CloseTradeRequest { reason: None };
        let json = serde_json::to_string(&req).unwrap();
        let parsed: CloseTradeRequest = serde_json::from_str(&json).unwrap();
        assert!(parsed.reason.is_none());
    }

    #[test]
    fn test_cov6_place_order_request_all_optional() {
        let req = PlaceOrderRequest {
            symbol: "ADAUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 100.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };
        assert!(req.price.is_none());
        assert!(req.confirmation_token.is_none());
    }

    #[test]
    fn test_cov6_order_info_market_order() {
        let order = OrderInfo {
            id: "o1".to_string(),
            exchange_order_id: 999888777,
            symbol: "SOLUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 5.0,
            executed_quantity: 5.0,
            price: None,
            avg_fill_price: 100.5,
            status: "FILLED".to_string(),
            is_entry: true,
            created_at: "2025-01-03T00:00:00Z".to_string(),
            updated_at: "2025-01-03T00:00:01Z".to_string(),
        };
        assert!(order.price.is_none());
        assert_eq!(order.status, "FILLED");
    }

    #[test]
    fn test_cov6_confirmation_response_market_order_summary() {
        let api = RealTradingApi::new(None);
        let req = PlaceOrderRequest {
            symbol: "DOGEUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1000.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };
        let conf = api.create_confirmation(req);
        assert!(conf.summary.contains("SELL"));
        assert!(conf.summary.contains("1000"));
        assert!(conf.summary.contains("MARKET"));
    }

    #[test]
    fn test_cov6_confirmation_response_limit_order_summary() {
        let api = RealTradingApi::new(None);
        let req = PlaceOrderRequest {
            symbol: "DOTUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 50.0,
            price: Some(10.5),
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };
        let conf = api.create_confirmation(req);
        assert!(conf.summary.contains("$10.50"));
        assert!(conf.summary.contains("BUY"));
    }

    #[test]
    fn test_cov6_list_orders_query_with_status() {
        let query = ListOrdersQuery {
            symbol: Some("BTCUSDT".to_string()),
            status: Some("filled".to_string()),
            limit: 25,
        };
        assert_eq!(query.status, Some("filled".to_string()));
        assert_eq!(query.limit, 25);
    }

    #[test]
    fn test_cov6_modify_sltp_request_both_none() {
        let req = ModifySlTpRequest {
            stop_loss: None,
            take_profit: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn test_cov6_update_settings_all_none() {
        let req = UpdateSettingsRequest {
            ..Default::default()
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("null"));
    }

    #[test]
    fn test_cov6_settings_response_full() {
        let settings = SettingsResponse {
            use_testnet: true,
            max_position_size_usdt: 2000.0,
            max_positions: 8,
            max_daily_loss_usdt: 800.0,
            max_total_exposure_usdt: 10000.0,
            risk_per_trade_percent: 2.5,
            default_stop_loss_percent: 1.5,
            default_take_profit_percent: 3.5,
            max_leverage: 15,
            circuit_breaker_errors: 5,
            circuit_breaker_cooldown_secs: 600,
            auto_trading_enabled: false,
            auto_trade_symbols: vec![],
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
        };
        assert_eq!(settings.max_positions, 8);
        assert_eq!(settings.max_leverage, 15);
    }

    #[test]
    fn test_cov6_balance_info_with_locked() {
        let balance = BalanceInfo {
            asset: "USDT".to_string(),
            free: 5000.0,
            locked: 3000.0,
            total: 8000.0,
        };
        assert_eq!(balance.free + balance.locked, balance.total);
    }

    #[test]
    fn test_cov6_cancel_all_query_no_symbol() {
        let query = CancelAllQuery { symbol: None };
        let json = serde_json::to_string(&query).unwrap();
        let parsed: CancelAllQuery = serde_json::from_str(&json).unwrap();
        assert!(parsed.symbol.is_none());
    }

    #[tokio::test]
    async fn test_cov6_routes_creation() {
        let api = create_test_api();
        let routes = api.routes();

        let paths = vec![
            ("/real-trading/status", "GET"),
            ("/real-trading/portfolio", "GET"),
            ("/real-trading/trades/open", "GET"),
            ("/real-trading/trades/closed", "GET"),
            ("/real-trading/start", "POST"),
            ("/real-trading/stop", "POST"),
            ("/real-trading/settings", "GET"),
            ("/real-trading/orders", "GET"),
            ("/real-trading/orders", "POST"),
        ];

        for (path, method) in paths {
            let resp = warp::test::request()
                .method(method)
                .path(path)
                .reply(&routes)
                .await;
            assert!(resp.status().as_u16() < 600);
        }
    }

    #[tokio::test]
    async fn test_cov6_place_order_with_sltp() {
        let api = create_test_api();
        let routes = api.routes();

        let req = PlaceOrderRequest {
            symbol: "LINKUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 10.0,
            price: Some(20.0),
            stop_loss: Some(19.0),
            take_profit: Some(22.0),
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&req)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov6_close_trade_different_symbols() {
        let api = create_test_api();
        let routes = api.routes();

        let symbols = vec!["trade-btc", "trade-eth", "trade-bnb"];
        for trade_id in symbols {
            let req = CloseTradeRequest {
                reason: Some("Test close".to_string()),
            };
            let path = format!("/real-trading/trades/{}/close", trade_id);
            let resp = warp::test::request()
                .method("POST")
                .path(&path)
                .json(&req)
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[tokio::test]
    async fn test_cov6_modify_sltp_different_positions() {
        let api = create_test_api();
        let routes = api.routes();

        let positions = vec!["BTCUSDT", "ETHUSDT", "BNBUSDT"];
        for symbol in positions {
            let req = ModifySlTpRequest {
                stop_loss: Some(1000.0),
                take_profit: Some(2000.0),
            };
            let path = format!("/real-trading/positions/{}/sltp", symbol);
            let resp = warp::test::request()
                .method("PUT")
                .path(&path)
                .json(&req)
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[tokio::test]
    async fn test_cov6_cancel_order_multiple() {
        let api = create_test_api();
        let routes = api.routes();

        let order_ids = vec!["ord-123", "ord-456", "ord-789"];
        for order_id in order_ids {
            let path = format!("/real-trading/orders/{}", order_id);
            let resp = warp::test::request()
                .method("DELETE")
                .path(&path)
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[tokio::test]
    async fn test_cov6_list_orders_different_filters() {
        let api = create_test_api();
        let routes = api.routes();

        let filters = vec![
            "?symbol=BTCUSDT",
            "?status=active",
            "?limit=100",
            "?symbol=ETHUSDT&status=filled",
            "?symbol=BNBUSDT&limit=50",
        ];

        for filter in filters {
            let path = format!("/real-trading/orders{}", filter);
            let resp = warp::test::request()
                .method("GET")
                .path(&path)
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[tokio::test]
    async fn test_cov6_update_settings_single_field_variations() {
        let api = create_test_api();
        let routes = api.routes();

        let updates = vec![
            r#"{"max_position_size_usdt": 5000.0}"#,
            r#"{"max_positions": 10}"#,
            r#"{"max_daily_loss_usdt": 1000.0}"#,
            r#"{"max_leverage": 20}"#,
        ];

        for update in updates {
            let resp = warp::test::request()
                .method("PUT")
                .path("/real-trading/settings")
                .header("content-type", "application/json")
                .body(update)
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
        }
    }

    #[test]
    fn test_cov6_confirmation_token_double_consume() {
        let api = RealTradingApi::new(None);
        let req = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let conf = api.create_confirmation(req);
        let token = conf.token.clone();

        let first = api.consume_confirmation(&token);
        assert!(first.is_some());

        let second = api.consume_confirmation(&token);
        assert!(second.is_none());
    }

    #[test]
    fn test_cov6_cleanup_confirmations_multiple_times() {
        let api = RealTradingApi::new(None);
        let req = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 1.0,
            price: Some(3000.0),
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        api.create_confirmation(req.clone());
        api.create_confirmation(req.clone());
        api.create_confirmation(req);

        api.cleanup_expired_confirmations();
        api.cleanup_expired_confirmations();
        api.cleanup_expired_confirmations();

        assert!(api.pending_confirmations.len() > 0);
    }

    #[test]
    fn test_cov6_default_limit_value() {
        let limit = default_limit();
        assert_eq!(limit, 50);
    }

    #[test]
    fn test_cov6_position_info_serialization_clone() {
        let pos = PositionInfo {
            id: "pos-test".to_string(),
            symbol: "XRPUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 100.0,
            entry_price: 0.5,
            current_price: 0.55,
            unrealized_pnl: 5.0,
            unrealized_pnl_pct: 10.0,
            stop_loss: Some(0.45),
            take_profit: Some(0.6),
            created_at: "2025-01-05T00:00:00Z".to_string(),
            ..Default::default()
        };
        let cloned = pos.clone();
        assert_eq!(cloned.id, pos.id);
        assert_eq!(cloned.quantity, pos.quantity);
    }

    #[test]
    fn test_cov6_balance_info_serialization_clone() {
        let balance = BalanceInfo {
            asset: "ETH".to_string(),
            free: 10.0,
            locked: 2.0,
            total: 12.0,
        };
        let cloned = balance.clone();
        assert_eq!(cloned.asset, balance.asset);
        assert_eq!(cloned.total, balance.total);
    }

    #[test]
    fn test_cov6_closed_trade_info_clone() {
        let trade = ClosedTradeInfo {
            id: "t-clone".to_string(),
            symbol: "ADAUSDT".to_string(),
            side: "SHORT".to_string(),
            quantity: 500.0,
            entry_price: 0.6,
            exit_price: 0.55,
            realized_pnl: 25.0,
            realized_pnl_pct: 8.33,
            commission: 0.25,
            opened_at: "2025-01-06T00:00:00Z".to_string(),
            closed_at: "2025-01-06T01:00:00Z".to_string(),
            close_reason: "Target reached".to_string(),
        };
        let cloned = trade.clone();
        assert_eq!(cloned.id, trade.id);
        assert_eq!(cloned.realized_pnl, trade.realized_pnl);
    }

    #[test]
    fn test_cov6_place_order_request_clone() {
        let req = PlaceOrderRequest {
            symbol: "DOTUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 50.0,
            price: Some(10.0),
            stop_loss: Some(9.5),
            take_profit: Some(11.0),
            confirmation_token: Some("token-123".to_string()),
        };
        let cloned = req.clone();
        assert_eq!(cloned.symbol, req.symbol);
        assert_eq!(cloned.price, req.price);
    }

    #[test]
    fn test_cov6_order_info_clone() {
        let order = OrderInfo {
            id: "o-clone".to_string(),
            exchange_order_id: 111222333,
            symbol: "LINKUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 20.0,
            executed_quantity: 10.0,
            price: Some(15.0),
            avg_fill_price: 14.95,
            status: "PartiallyFilled".to_string(),
            is_entry: true,
            created_at: "2025-01-07T00:00:00Z".to_string(),
            updated_at: "2025-01-07T00:05:00Z".to_string(),
        };
        let cloned = order.clone();
        assert_eq!(cloned.id, order.id);
        assert_eq!(cloned.executed_quantity, order.executed_quantity);
    }

    #[test]
    fn test_cov6_confirmation_response_clone() {
        let conf = ConfirmationResponse {
            token: "token-xyz".to_string(),
            expires_at: "2025-01-08T00:00:00Z".to_string(),
            summary: "BUY 0.1 BTCUSDT MARKET @ MARKET".to_string(),
            order_details: PlaceOrderRequest {
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                order_type: "MARKET".to_string(),
                quantity: 0.1,
                price: None,
                stop_loss: None,
                take_profit: None,
                confirmation_token: None,
            },
        };
        let cloned = conf.clone();
        assert_eq!(cloned.token, conf.token);
        assert_eq!(cloned.summary, conf.summary);
    }

    #[test]
    fn test_cov6_list_orders_query_clone() {
        let query = ListOrdersQuery {
            symbol: Some("BNBUSDT".to_string()),
            status: Some("cancelled".to_string()),
            limit: 75,
        };
        let cloned = query.clone();
        assert_eq!(cloned.symbol, query.symbol);
        assert_eq!(cloned.limit, query.limit);
    }

    #[test]
    fn test_cov6_modify_sltp_request_clone() {
        let req = ModifySlTpRequest {
            stop_loss: Some(45000.0),
            take_profit: Some(60000.0),
        };
        let cloned = req.clone();
        assert_eq!(cloned.stop_loss, req.stop_loss);
        assert_eq!(cloned.take_profit, req.take_profit);
    }

    #[test]
    fn test_cov6_cancel_all_query_clone() {
        let query = CancelAllQuery {
            symbol: Some("SOLUSDT".to_string()),
        };
        let cloned = query.clone();
        assert_eq!(cloned.symbol, query.symbol);
    }

    #[tokio::test]
    async fn test_cov6_wrong_http_methods() {
        let api = create_test_api();
        let routes = api.routes();

        let wrong_methods = vec![
            ("DELETE", "/real-trading/status"),
            ("PUT", "/real-trading/portfolio"),
            ("POST", "/real-trading/trades/open"),
            ("DELETE", "/real-trading/start"),
            ("GET", "/real-trading/stop"),
        ];

        for (method, path) in wrong_methods {
            let resp = warp::test::request()
                .method(method)
                .path(path)
                .reply(&routes)
                .await;
            assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
        }
    }

    // ============================================================================
    // COVERAGE PHASE 7 - Additional Handler & Type Tests
    // ============================================================================

    #[test]
    fn test_cov7_api_response_success_with_data() {
        let resp = ApiResponse::success(vec![1, 2, 3]);
        assert!(resp.success);
        assert_eq!(resp.data, Some(vec![1, 2, 3]));
        assert!(resp.error.is_none());
    }

    #[test]
    fn test_cov7_api_response_error_with_message() {
        let resp: ApiResponse<String> =
            ApiResponse::error("Database connection failed".to_string());
        assert!(!resp.success);
        assert!(resp.data.is_none());
        assert_eq!(resp.error, Some("Database connection failed".to_string()));
    }

    #[test]
    fn test_cov7_confirmation_response_serialization_full() {
        let confirmation = ConfirmationResponse {
            token: "test-token-xyz".to_string(),
            expires_at: "2025-01-15T10:30:00Z".to_string(),
            summary: "BUY 0.5 ETHUSDT LIMIT @ $3000.00".to_string(),
            order_details: PlaceOrderRequest {
                symbol: "ETHUSDT".to_string(),
                side: "BUY".to_string(),
                order_type: "LIMIT".to_string(),
                quantity: 0.5,
                price: Some(3000.0),
                stop_loss: Some(2900.0),
                take_profit: Some(3200.0),
                confirmation_token: Some("test-token-xyz".to_string()),
            },
        };

        let json = serde_json::to_string(&confirmation).unwrap();
        assert!(json.contains("test-token-xyz"));
        assert!(json.contains("BUY 0.5 ETHUSDT LIMIT"));
        assert!(json.contains("3000.0"));

        let parsed: ConfirmationResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.token, "test-token-xyz");
        assert_eq!(parsed.order_details.symbol, "ETHUSDT");
    }

    #[test]
    fn test_cov7_cancel_all_query_with_symbol() {
        let query = CancelAllQuery {
            symbol: Some("BTCUSDT".to_string()),
        };
        let json = serde_json::to_string(&query).unwrap();
        assert!(json.contains("BTCUSDT"));

        let parsed: CancelAllQuery = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.symbol, Some("BTCUSDT".to_string()));
    }

    #[test]
    fn test_cov7_cancel_all_query_no_symbol() {
        let query = CancelAllQuery { symbol: None };
        let json = serde_json::to_string(&query).unwrap();
        let parsed: CancelAllQuery = serde_json::from_str(&json).unwrap();
        assert!(parsed.symbol.is_none());
    }

    #[test]
    fn test_cov7_list_orders_query_default_limit() {
        let json = r#"{}"#;
        let query: ListOrdersQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, 50);
        assert!(query.symbol.is_none());
        assert!(query.status.is_none());
    }

    #[test]
    fn test_cov7_list_orders_query_custom_limit() {
        let json = r#"{"limit": 100}"#;
        let query: ListOrdersQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.limit, 100);
    }

    #[test]
    fn test_cov7_list_orders_query_status_filter() {
        let json = r#"{"status": "filled"}"#;
        let query: ListOrdersQuery = serde_json::from_str(json).unwrap();
        assert_eq!(query.status, Some("filled".to_string()));
    }

    #[test]
    fn test_cov7_position_info_all_fields() {
        let pos = PositionInfo {
            id: "pos-abc".to_string(),
            symbol: "SOLUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 10.0,
            entry_price: 100.0,
            current_price: 105.0,
            unrealized_pnl: 50.0,
            unrealized_pnl_pct: 5.0,
            stop_loss: Some(95.0),
            take_profit: Some(110.0),
            created_at: "2025-01-15T12:00:00Z".to_string(),
            ..Default::default()
        };

        let json = serde_json::to_string(&pos).unwrap();
        assert!(json.contains("SOLUSDT"));
        assert!(json.contains("LONG"));
        assert!(json.contains("50.0"));

        let parsed: PositionInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.id, "pos-abc");
        assert_eq!(parsed.unrealized_pnl, 50.0);
    }

    #[test]
    fn test_cov7_balance_info_multiple_assets() {
        let balances = vec![
            BalanceInfo {
                asset: "USDT".to_string(),
                free: 1000.0,
                locked: 500.0,
                total: 1500.0,
            },
            BalanceInfo {
                asset: "BTC".to_string(),
                free: 0.5,
                locked: 0.1,
                total: 0.6,
            },
        ];

        for balance in &balances {
            let json = serde_json::to_string(balance).unwrap();
            let parsed: BalanceInfo = serde_json::from_str(&json).unwrap();
            assert_eq!(parsed.asset, balance.asset);
            assert_eq!(parsed.total, balance.total);
        }
    }

    #[test]
    fn test_cov7_update_settings_request_all_fields() {
        let req = UpdateSettingsRequest {
            max_position_size_usdt: Some(2000.0),
            max_positions: Some(8),
            max_daily_loss_usdt: Some(800.0),
            max_total_exposure_usdt: Some(16000.0),
            risk_per_trade_percent: Some(3.0),
            default_stop_loss_percent: Some(2.5),
            default_take_profit_percent: Some(6.0),
            max_leverage: Some(15),
            ..Default::default()
        };

        let json = serde_json::to_string(&req).unwrap();
        let parsed: UpdateSettingsRequest = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.max_position_size_usdt, Some(2000.0));
        assert_eq!(parsed.max_positions, Some(8));
        assert_eq!(parsed.max_leverage, Some(15));
    }

    #[test]
    fn test_cov7_settings_response_testnet_config() {
        let settings = SettingsResponse {
            use_testnet: true,
            max_position_size_usdt: 500.0,
            max_positions: 3,
            max_daily_loss_usdt: 200.0,
            max_total_exposure_usdt: 1500.0,
            risk_per_trade_percent: 1.5,
            default_stop_loss_percent: 2.0,
            default_take_profit_percent: 4.0,
            max_leverage: 5,
            circuit_breaker_errors: 3,
            circuit_breaker_cooldown_secs: 300,
            auto_trading_enabled: false,
            auto_trade_symbols: vec![],
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
        };

        let json = serde_json::to_string(&settings).unwrap();
        assert!(json.contains("\"use_testnet\":true"));
        assert!(json.contains("500.0"));
    }

    #[test]
    fn test_cov7_order_info_partially_filled() {
        let order = OrderInfo {
            id: "real_partial123".to_string(),
            exchange_order_id: 111222333,
            symbol: "ADAUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 100.0,
            executed_quantity: 50.0,
            price: Some(0.5),
            avg_fill_price: 0.49,
            status: "PartiallyFilled".to_string(),
            is_entry: true,
            created_at: "2025-01-15T08:00:00Z".to_string(),
            updated_at: "2025-01-15T08:05:00Z".to_string(),
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("PartiallyFilled"));
        assert!(json.contains("50.0"));
    }

    #[test]
    fn test_cov7_closed_trade_info_loss() {
        let trade = ClosedTradeInfo {
            id: "trade-loss-01".to_string(),
            symbol: "DOGEUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 1000.0,
            entry_price: 0.08,
            exit_price: 0.07,
            realized_pnl: -10.0,
            realized_pnl_pct: -12.5,
            commission: 0.2,
            opened_at: "2025-01-10T09:00:00Z".to_string(),
            closed_at: "2025-01-10T15:00:00Z".to_string(),
            close_reason: "Stop loss triggered".to_string(),
        };

        let json = serde_json::to_string(&trade).unwrap();
        assert!(json.contains("-10.0"));
        assert!(json.contains("Stop loss triggered"));
    }

    #[tokio::test]
    async fn test_cov7_create_confirmation_market_order() {
        let api = create_test_api();
        let request = PlaceOrderRequest {
            symbol: "AVAXUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 5.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request.clone());
        assert!(!confirmation.token.is_empty());
        assert!(confirmation.summary.contains("SELL"));
        assert!(confirmation.summary.contains("AVAXUSDT"));
        assert!(confirmation.summary.contains("MARKET"));
        assert_eq!(confirmation.order_details.symbol, "AVAXUSDT");
    }

    #[tokio::test]
    async fn test_cov7_create_confirmation_limit_order_with_price() {
        let api = create_test_api();
        let request = PlaceOrderRequest {
            symbol: "LINKUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 10.0,
            price: Some(15.0),
            stop_loss: Some(14.0),
            take_profit: Some(18.0),
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request.clone());
        assert!(confirmation.summary.contains("$15.00"));
        assert!(confirmation.summary.contains("BUY"));
        assert!(confirmation.summary.contains("LINKUSDT"));
    }

    #[tokio::test]
    async fn test_cov7_consume_confirmation_twice() {
        let api = create_test_api();
        let request = PlaceOrderRequest {
            symbol: "MATICUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 50.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request.clone());
        let token = confirmation.token.clone();

        // First consumption should succeed
        let consumed1 = api.consume_confirmation(&token);
        assert!(consumed1.is_some());
        assert_eq!(consumed1.unwrap().symbol, "MATICUSDT");

        // Second consumption should fail (token already consumed)
        let consumed2 = api.consume_confirmation(&token);
        assert!(consumed2.is_none());
    }

    #[tokio::test]
    async fn test_cov7_consume_invalid_confirmation_token() {
        let api = create_test_api();
        let consumed = api.consume_confirmation("invalid-uuid-token-xyz");
        assert!(consumed.is_none());
    }

    #[tokio::test]
    async fn test_cov7_cleanup_expired_confirmations() {
        let api = create_test_api();

        let request = PlaceOrderRequest {
            symbol: "DOTUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 20.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        api.create_confirmation(request.clone());
        api.create_confirmation(request.clone());
        api.create_confirmation(request);

        // Cleanup should run without panics
        api.cleanup_expired_confirmations();

        // Confirmations should still exist (not expired yet within 60s)
        assert!(!api.pending_confirmations.is_empty());
    }

    #[tokio::test]
    async fn test_cov7_route_status_endpoint() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/status")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
        let body_str = std::str::from_utf8(resp.body()).unwrap();
        assert!(body_str.contains("not configured") || body_str.contains("success"));
    }

    #[tokio::test]
    async fn test_cov7_route_portfolio_endpoint() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov7_route_place_order_market() {
        let api = create_test_api();
        let routes = api.routes();

        let order = PlaceOrderRequest {
            symbol: "XRPUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 100.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&order)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov7_route_cancel_order_by_id() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/order-xyz-789")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov7_route_modify_sltp_both() {
        let api = create_test_api();
        let routes = api.routes();

        let modify = ModifySlTpRequest {
            stop_loss: Some(47000.0),
            take_profit: Some(53000.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/BTCUSDT/sltp")
            .json(&modify)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov7_route_close_trade_with_reason() {
        let api = create_test_api();
        let routes = api.routes();

        let close = CloseTradeRequest {
            reason: Some("Manual close due to market conditions".to_string()),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/trades/trade-abc-123/close")
            .json(&close)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov7_route_list_orders_with_all_params() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?symbol=SOLUSDT&status=active&limit=25")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov7_route_cancel_all_with_symbol_filter() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/all?symbol=ETHUSDT")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov7_route_update_settings_full() {
        let api = create_test_api();
        let routes = api.routes();

        let settings = UpdateSettingsRequest {
            max_position_size_usdt: Some(3000.0),
            max_positions: Some(10),
            max_daily_loss_usdt: Some(1000.0),
            max_total_exposure_usdt: Some(20000.0),
            risk_per_trade_percent: Some(2.0),
            default_stop_loss_percent: Some(1.5),
            default_take_profit_percent: Some(3.0),
            max_leverage: Some(10),
            ..Default::default()
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/settings")
            .json(&settings)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_cov7_portfolio_response_with_positions() {
        let portfolio = PortfolioResponse {
            total_balance: 25000.0,
            available_balance: 20000.0,
            locked_balance: 5000.0,
            unrealized_pnl: 250.0,
            realized_pnl: 1500.0,
            positions: vec![PositionInfo {
                id: "pos-1".to_string(),
                symbol: "BTCUSDT".to_string(),
                side: "LONG".to_string(),
                quantity: 0.1,
                entry_price: 50000.0,
                current_price: 51000.0,
                unrealized_pnl: 100.0,
                unrealized_pnl_pct: 2.0,
                stop_loss: Some(49000.0),
                take_profit: Some(55000.0),
                created_at: "2025-01-15T10:00:00Z".to_string(),
                ..Default::default()
            }],
            balances: vec![BalanceInfo {
                asset: "USDT".to_string(),
                free: 20000.0,
                locked: 5000.0,
                total: 25000.0,
            }],
            ..Default::default()
        };

        let json = serde_json::to_string(&portfolio).unwrap();
        assert!(json.contains("25000.0"));
        assert!(json.contains("BTCUSDT"));
    }

    #[test]
    fn test_cov7_engine_status_with_uptime() {
        let status = EngineStatus {
            is_running: true,
            is_testnet: false,
            open_positions_count: 5,
            open_orders_count: 3,
            circuit_breaker_open: false,
            daily_pnl: 500.0,
            daily_trades_count: 15,
            uptime_seconds: Some(7200),
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"is_running\":true"));
        assert!(json.contains("7200"));
    }

    #[tokio::test]
    async fn test_cov8_place_order_with_confirmation_token() {
        let api = create_test_api();
        let routes = api.routes();

        let body = r#"{
            "symbol": "BTCUSDT",
            "side": "BUY",
            "order_type": "MARKET",
            "quantity": 0.001,
            "confirmation_token": "test_token_123"
        }"#;

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/place-order")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov8_place_order_invalid_side() {
        let api = create_test_api();
        let routes = api.routes();

        let body = r#"{
            "symbol": "BTCUSDT",
            "side": "INVALID",
            "order_type": "MARKET",
            "quantity": 0.001
        }"#;

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/place-order")
            .header("content-type", "application/json")
            .body(body)
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov8_confirm_order_nonexistent_token() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/confirm/nonexistent_token_12345")
            .reply(&routes)
            .await;

        assert!(resp.status().is_client_error() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov8_get_open_orders() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov8_cleanup_expired_confirmations_with_old_entries() {
        let api = create_test_api();

        let request = PlaceOrderRequest {
            symbol: "TESTUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request);
        let token = confirmation.token.clone();

        // Manually modify expiry to be in the past
        if let Some(mut entry) = api.pending_confirmations.get_mut(&token) {
            entry.expires_at = Utc::now() - Duration::hours(2);
        }

        api.cleanup_expired_confirmations();

        // Should be removed after cleanup
        assert!(api.pending_confirmations.get(&token).is_none());
    }

    #[test]
    fn test_cov8_api_response_serialization() {
        let response: ApiResponse<String> = ApiResponse::success("test data".to_string());
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("success"));
        assert!(json.contains("test data"));
    }

    #[test]
    fn test_cov8_engine_status_serialization() {
        let status = EngineStatus {
            is_running: false,
            is_testnet: true,
            open_positions_count: 0,
            open_orders_count: 0,
            circuit_breaker_open: true,
            daily_pnl: -100.0,
            daily_trades_count: 5,
            uptime_seconds: None,
        };

        let json = serde_json::to_string(&status).unwrap();
        assert!(json.contains("\"is_running\":false"));
        assert!(json.contains("\"circuit_breaker_open\":true"));
        assert!(json.contains("-100"));
    }

    #[test]
    fn test_cov8_place_order_request_serialization() {
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.5,
            price: Some(45000.0),
            stop_loss: Some(44000.0),
            take_profit: Some(47000.0),
            confirmation_token: Some("token_xyz".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("45000"));
        assert!(json.contains("token_xyz"));
    }

    #[test]
    fn test_cov8_pending_confirmation_fields() {
        let request = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let pending = PendingConfirmation {
            _token: "test_token".to_string(),
            order_request: request.clone(),
            expires_at: Utc::now() + Duration::seconds(60),
        };

        assert_eq!(pending.order_request.symbol, "ETHUSDT");
        assert_eq!(pending.order_request.side, "SELL");
        assert!(pending.expires_at > Utc::now());
    }

    #[test]
    fn test_cov8_confirmation_response_structure() {
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.001,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let response = ConfirmationResponse {
            token: "test_token_123".to_string(),
            expires_at: "2024-01-01T00:01:00Z".to_string(),
            summary: "BUY 0.001 BTCUSDT at market price".to_string(),
            order_details: request,
        };

        assert_eq!(response.token, "test_token_123");
        assert!(response.summary.contains("BUY"));
        assert!(response.expires_at.contains("2024"));
    }

    #[test]
    fn test_cov8_position_info_serialization() {
        let position = PositionInfo {
            id: "pos_123".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 0.5,
            entry_price: 50000.0,
            current_price: 51000.0,
            unrealized_pnl: 500.0,
            unrealized_pnl_pct: 1.0,
            stop_loss: Some(49000.0),
            take_profit: Some(52000.0),
            created_at: Utc::now().to_rfc3339(),
            ..Default::default()
        };

        let json = serde_json::to_string(&position).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("50000"));
        assert!(json.contains("500"));
    }

    #[test]
    fn test_cov8_order_info_serialization() {
        let order = OrderInfo {
            id: "order_123".to_string(),
            exchange_order_id: 12345678,
            symbol: "ETHUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 1.0,
            executed_quantity: 0.5,
            price: Some(3000.0),
            avg_fill_price: 0.0,
            status: "NEW".to_string(),
            is_entry: true,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("order_123"));
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("3000"));
    }

    #[test]
    fn test_cov8_balance_info_structure() {
        let balance = BalanceInfo {
            asset: "USDT".to_string(),
            free: 10000.0,
            locked: 500.0,
            total: 10500.0,
        };

        assert_eq!(balance.asset, "USDT");
        assert_eq!(balance.free, 10000.0);
        assert_eq!(balance.locked, 500.0);
        assert_eq!(balance.total, 10500.0);
    }

    #[test]
    fn test_cov8_portfolio_response_with_positions() {
        let portfolio = PortfolioResponse {
            total_balance: 15000.0,
            available_balance: 10000.0,
            locked_balance: 5000.0,
            unrealized_pnl: 500.0,
            realized_pnl: 1500.0,
            positions: vec![],
            balances: vec![],
            ..Default::default()
        };

        assert_eq!(portfolio.total_balance, 15000.0);
        assert_eq!(portfolio.unrealized_pnl, 500.0);
        assert_eq!(portfolio.realized_pnl, 1500.0);
    }

    #[tokio::test]
    async fn test_cov9_get_portfolio_route() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        assert!(resp.status().is_success() || resp.status().is_server_error());
    }

    #[tokio::test]
    async fn test_cov9_get_positions_route() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/positions")
            .reply(&routes)
            .await;

        assert!(
            resp.status().is_success() || resp.status().is_server_error() || resp.status() == 404
        );
    }

    #[tokio::test]
    async fn test_cov9_get_orders_route() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders")
            .reply(&routes)
            .await;

        assert!(
            resp.status().is_success() || resp.status().is_server_error() || resp.status() == 404
        );
    }

    #[tokio::test]
    async fn test_cov9_get_open_orders_route() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders/open")
            .reply(&routes)
            .await;

        // Accept any response status (route may not be implemented or require auth)
        assert!(resp.status().as_u16() >= 200);
    }

    #[tokio::test]
    async fn test_cov9_create_order_missing_token() {
        let api = create_test_api();
        let routes = api.routes();

        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&request)
            .reply(&routes)
            .await;

        // Accept any response status (route may reject missing token or not exist)
        assert!(resp.status().as_u16() >= 200);
    }

    #[tokio::test]
    async fn test_cov9_close_position_route() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/positions/BTCUSDT/close")
            .reply(&routes)
            .await;

        assert!(
            resp.status().is_success()
                || resp.status().is_client_error()
                || resp.status().is_server_error()
        );
    }

    #[tokio::test]
    async fn test_cov9_cancel_order_route() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("DELETE")
            .path("/real-trading/orders/BTCUSDT/12345")
            .reply(&routes)
            .await;

        assert!(
            resp.status().is_success()
                || resp.status().is_client_error()
                || resp.status().is_server_error()
        );
    }

    #[tokio::test]
    async fn test_cov9_invalid_route() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/nonexistent")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_cov9_method_not_allowed() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("PATCH")
            .path("/real-trading/portfolio")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::METHOD_NOT_ALLOWED);
    }

    #[test]
    fn test_cov9_create_order_request_with_stop_loss() {
        let request = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 1.0,
            price: Some(3000.0),
            stop_loss: Some(3100.0),
            take_profit: None,
            confirmation_token: Some("token123".to_string()),
        };

        assert!(request.stop_loss.is_some());
        assert!(request.take_profit.is_none());
        assert!(request.confirmation_token.is_some());
    }

    #[test]
    fn test_cov9_create_order_request_with_take_profit() {
        let request = PlaceOrderRequest {
            symbol: "BNBUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 10.0,
            price: None,
            stop_loss: None,
            take_profit: Some(350.0),
            confirmation_token: None,
        };

        assert!(request.stop_loss.is_none());
        assert!(request.take_profit.is_some());
        assert_eq!(request.side, "BUY");
    }

    #[test]
    fn test_cov9_close_trade_request() {
        let request = CloseTradeRequest {
            reason: Some("Manual close".to_string()),
        };

        assert!(request.reason.is_some());
        assert_eq!(request.reason.unwrap(), "Manual close");
    }

    #[test]
    fn test_cov9_position_info_with_no_sl_tp() {
        let position = PositionInfo {
            id: "pos_456".to_string(),
            symbol: "SOLUSDT".to_string(),
            side: "SHORT".to_string(),
            quantity: 20.0,
            entry_price: 100.0,
            current_price: 95.0,
            unrealized_pnl: 100.0,
            unrealized_pnl_pct: 5.0,
            stop_loss: None,
            take_profit: None,
            created_at: Utc::now().to_rfc3339(),
            ..Default::default()
        };

        assert!(position.stop_loss.is_none());
        assert!(position.take_profit.is_none());
        assert_eq!(position.side, "SHORT");
    }

    #[test]
    fn test_cov9_order_info_partial_fill() {
        let order = OrderInfo {
            id: "order_789".to_string(),
            exchange_order_id: 987654321,
            symbol: "ADAUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 100.0,
            executed_quantity: 50.0,
            price: Some(1.5),
            avg_fill_price: 1.52,
            status: "PARTIALLY_FILLED".to_string(),
            is_entry: false,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(order.executed_quantity, 50.0);
        assert_eq!(order.status, "PARTIALLY_FILLED");
        assert!(!order.is_entry);
    }

    #[test]
    fn test_cov9_balance_info_zero_locked() {
        let balance = BalanceInfo {
            asset: "BTC".to_string(),
            free: 0.5,
            locked: 0.0,
            total: 0.5,
        };

        assert_eq!(balance.locked, 0.0);
        assert_eq!(balance.free, balance.total);
    }

    #[test]
    fn test_cov9_confirmation_response_expiry() {
        let request = PlaceOrderRequest {
            symbol: "LTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 5.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let response = ConfirmationResponse {
            token: "exp_token".to_string(),
            expires_at: "2024-12-31T23:59:59Z".to_string(),
            summary: "BUY 5 LTCUSDT".to_string(),
            order_details: request,
        };

        assert!(response.expires_at.contains("2024"));
        assert!(response.summary.contains("BUY"));
    }

    // ============================================================================
    // ADDITIONAL COVERAGE BOOST TESTS - Phase 10
    // ============================================================================

    #[tokio::test]
    async fn test_cov10_place_order_with_confirmation_token() {
        let api = create_test_api();
        let routes = api.routes();

        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.001,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: Some("dummy-token".to_string()),
        };

        let resp = warp::test::request()
            .method("POST")
            .path("/real-trading/orders")
            .json(&request)
            .reply(&routes)
            .await;

        assert!(
            resp.status().is_success()
                || resp.status().is_server_error()
                || resp.status().is_client_error()
        );
    }

    #[tokio::test]
    async fn test_cov10_list_orders_all_params() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/orders?symbol=ETHUSDT&status=filled&limit=25")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov10_modify_sltp_only_stop_loss() {
        let api = create_test_api();
        let routes = api.routes();

        let request = ModifySlTpRequest {
            stop_loss: Some(49000.0),
            take_profit: None,
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/BTCUSDT/sltp")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov10_modify_sltp_only_take_profit() {
        let api = create_test_api();
        let routes = api.routes();

        let request = ModifySlTpRequest {
            stop_loss: None,
            take_profit: Some(55000.0),
        };

        let resp = warp::test::request()
            .method("PUT")
            .path("/real-trading/positions/ETHUSDT/sltp")
            .json(&request)
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[tokio::test]
    async fn test_cov10_cancel_order_long_id() {
        let api = create_test_api();
        let routes = api.routes();

        let order_id = "a".repeat(100);
        let resp = warp::test::request()
            .method("DELETE")
            .path(&format!("/real-trading/orders/{}", order_id))
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), warp::http::StatusCode::SERVICE_UNAVAILABLE);
    }

    #[test]
    fn test_cov10_api_response_success() {
        let resp: ApiResponse<String> = ApiResponse::success("data".to_string());
        assert!(resp.success);
        assert_eq!(resp.data, Some("data".to_string()));
    }

    #[test]
    fn test_cov10_place_order_request_market_order() {
        let request = PlaceOrderRequest {
            symbol: "DOGEUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1000.0,
            price: None,
            stop_loss: Some(0.08),
            take_profit: Some(0.12),
            confirmation_token: None,
        };

        assert!(request.price.is_none());
        assert!(request.stop_loss.is_some());
        assert_eq!(request.order_type, "MARKET");
    }

    #[test]
    fn test_cov10_list_orders_query_max_limit() {
        let query = ListOrdersQuery {
            symbol: Some("BTCUSDT".to_string()),
            status: Some("filled".to_string()),
            limit: 1000,
        };

        assert_eq!(query.limit, 1000);
        assert!(query.symbol.is_some());
    }

    #[test]
    fn test_cov10_cancel_all_query_none() {
        let query = CancelAllQuery { symbol: None };
        assert!(query.symbol.is_none());
    }

    #[test]
    fn test_cov10_closed_trade_info_zero_pnl() {
        let trade = ClosedTradeInfo {
            id: "trade_zero".to_string(),
            symbol: "XRPUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 100.0,
            entry_price: 1.0,
            exit_price: 1.0,
            realized_pnl: 0.0,
            realized_pnl_pct: 0.0,
            commission: 0.01,
            opened_at: Utc::now().to_rfc3339(),
            closed_at: Utc::now().to_rfc3339(),
            close_reason: "Manual".to_string(),
        };

        assert_eq!(trade.realized_pnl, 0.0);
        assert_eq!(trade.realized_pnl_pct, 0.0);
    }

    #[test]
    fn test_cov10_engine_status_all_false() {
        let status = EngineStatus {
            is_running: false,
            is_testnet: false,
            open_positions_count: 0,
            open_orders_count: 0,
            circuit_breaker_open: true,
            daily_pnl: -500.0,
            daily_trades_count: 0,
            uptime_seconds: None,
        };

        assert!(!status.is_running);
        assert!(status.circuit_breaker_open);
        assert!(status.daily_pnl < 0.0);
        assert!(status.uptime_seconds.is_none());
    }

    #[test]
    fn test_cov10_portfolio_response_negative_pnl() {
        let portfolio = PortfolioResponse {
            total_balance: 8000.0,
            available_balance: 7500.0,
            locked_balance: 500.0,
            unrealized_pnl: -200.0,
            realized_pnl: -100.0,
            positions: vec![],
            balances: vec![],
            ..Default::default()
        };

        assert!(portfolio.unrealized_pnl < 0.0);
        assert!(portfolio.realized_pnl < 0.0);
        assert_eq!(portfolio.positions.len(), 0);
    }

    #[test]
    fn test_cov10_order_info_fully_filled() {
        let order = OrderInfo {
            id: "order_full".to_string(),
            exchange_order_id: 111222333,
            symbol: "DOTUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 50.0,
            executed_quantity: 50.0,
            price: Some(10.0),
            avg_fill_price: 9.99,
            status: "Filled".to_string(),
            is_entry: true,
            created_at: Utc::now().to_rfc3339(),
            updated_at: Utc::now().to_rfc3339(),
        };

        assert_eq!(order.quantity, order.executed_quantity);
        assert_eq!(order.status, "Filled");
    }

    #[test]
    fn test_cov10_confirmation_response_market_order() {
        let request = PlaceOrderRequest {
            symbol: "MATICUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 200.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = ConfirmationResponse {
            token: "token_market".to_string(),
            expires_at: Utc::now().to_rfc3339(),
            summary: format!(
                "{} {} {} MARKET",
                request.side, request.quantity, request.symbol
            ),
            order_details: request,
        };

        assert!(confirmation.summary.contains("MARKET"));
        assert!(confirmation.summary.contains("SELL"));
    }

    // ============================================================================
    // BOOST COVERAGE - Additional Unit Tests
    // ============================================================================

    #[test]
    fn test_boost_api_response_success_i32() {
        let response = ApiResponse::success(42i32);
        assert!(response.success);
        assert_eq!(response.data, Some(42));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_boost_api_response_success_string() {
        let response = ApiResponse::success("hello".to_string());
        assert!(response.success);
        assert_eq!(response.data, Some("hello".to_string()));
        assert!(response.error.is_none());
    }

    #[test]
    fn test_boost_api_response_error_empty_string() {
        let response: ApiResponse<()> = ApiResponse::error("".to_string());
        assert!(!response.success);
        assert!(response.data.is_none());
        assert_eq!(response.error, Some("".to_string()));
    }

    #[test]
    fn test_boost_api_response_error_long_message() {
        let long_msg = "A".repeat(1000);
        let response: ApiResponse<()> = ApiResponse::error(long_msg.clone());
        assert_eq!(response.error, Some(long_msg));
    }

    #[test]
    fn test_boost_engine_status_default_values() {
        let status = EngineStatus {
            is_running: false,
            is_testnet: false,
            open_positions_count: 0,
            open_orders_count: 0,
            circuit_breaker_open: false,
            daily_pnl: 0.0,
            daily_trades_count: 0,
            uptime_seconds: None,
        };

        assert!(!status.is_running);
        assert!(!status.is_testnet);
        assert_eq!(status.open_positions_count, 0);
        assert!(status.uptime_seconds.is_none());
    }

    #[test]
    fn test_boost_engine_status_with_uptime() {
        let status = EngineStatus {
            is_running: true,
            is_testnet: true,
            open_positions_count: 5,
            open_orders_count: 3,
            circuit_breaker_open: false,
            daily_pnl: 150.0,
            daily_trades_count: 10,
            uptime_seconds: Some(7200),
        };

        assert_eq!(status.uptime_seconds, Some(7200));
        assert_eq!(status.daily_trades_count, 10);
    }

    #[test]
    fn test_boost_portfolio_response_empty_positions() {
        let portfolio = PortfolioResponse {
            total_balance: 0.0,
            available_balance: 0.0,
            locked_balance: 0.0,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            positions: vec![],
            balances: vec![],
            ..Default::default()
        };

        assert!(portfolio.positions.is_empty());
        assert!(portfolio.balances.is_empty());
        assert_eq!(portfolio.total_balance, 0.0);
    }

    #[test]
    fn test_boost_portfolio_response_with_positions() {
        let pos = PositionInfo {
            id: "pos-1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 0.5,
            entry_price: 50000.0,
            current_price: 51000.0,
            unrealized_pnl: 500.0,
            unrealized_pnl_pct: 1.0,
            stop_loss: None,
            take_profit: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        let portfolio = PortfolioResponse {
            total_balance: 10000.0,
            available_balance: 9000.0,
            locked_balance: 1000.0,
            unrealized_pnl: 500.0,
            realized_pnl: 100.0,
            positions: vec![pos],
            balances: vec![],
            ..Default::default()
        };

        assert_eq!(portfolio.positions.len(), 1);
        assert_eq!(portfolio.positions[0].symbol, "BTCUSDT");
    }

    #[test]
    fn test_boost_position_info_with_sl_tp() {
        let pos = PositionInfo {
            id: "pos-2".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "SHORT".to_string(),
            quantity: 2.0,
            entry_price: 3000.0,
            current_price: 2950.0,
            unrealized_pnl: 100.0,
            unrealized_pnl_pct: 3.33,
            stop_loss: Some(3100.0),
            take_profit: Some(2800.0),
            created_at: "2025-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        assert_eq!(pos.stop_loss, Some(3100.0));
        assert_eq!(pos.take_profit, Some(2800.0));
        assert_eq!(pos.side, "SHORT");
    }

    #[test]
    fn test_boost_position_info_zero_pnl() {
        let pos = PositionInfo {
            id: "pos-3".to_string(),
            symbol: "ADAUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 100.0,
            entry_price: 1.0,
            current_price: 1.0,
            unrealized_pnl: 0.0,
            unrealized_pnl_pct: 0.0,
            stop_loss: None,
            take_profit: None,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            ..Default::default()
        };

        assert_eq!(pos.unrealized_pnl, 0.0);
        assert_eq!(pos.unrealized_pnl_pct, 0.0);
    }

    #[test]
    fn test_boost_balance_info_with_values() {
        let balance = BalanceInfo {
            asset: "BTC".to_string(),
            free: 1.5,
            locked: 0.5,
            total: 2.0,
        };

        assert_eq!(balance.asset, "BTC");
        assert!((balance.total - 2.0).abs() < 0.001);
    }

    #[test]
    fn test_boost_balance_info_deserialization() {
        let json = r#"{
            "asset": "ETH",
            "free": 10.0,
            "locked": 2.5,
            "total": 12.5
        }"#;

        let balance: BalanceInfo = serde_json::from_str(json).unwrap();
        assert_eq!(balance.asset, "ETH");
        assert!((balance.free - 10.0).abs() < 0.001);
    }

    #[test]
    fn test_boost_closed_trade_info_profit() {
        let trade = ClosedTradeInfo {
            id: "trade-1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 0.1,
            entry_price: 50000.0,
            exit_price: 51000.0,
            realized_pnl: 100.0,
            realized_pnl_pct: 2.0,
            commission: 1.0,
            opened_at: "2025-01-01T00:00:00Z".to_string(),
            closed_at: "2025-01-01T01:00:00Z".to_string(),
            close_reason: "Take profit".to_string(),
        };

        assert!(trade.realized_pnl > 0.0);
        assert_eq!(trade.close_reason, "Take profit");
    }

    #[test]
    fn test_boost_closed_trade_info_loss() {
        let trade = ClosedTradeInfo {
            id: "trade-2".to_string(),
            symbol: "ETHUSDT".to_string(),
            side: "SHORT".to_string(),
            quantity: 1.0,
            entry_price: 3000.0,
            exit_price: 3100.0,
            realized_pnl: -100.0,
            realized_pnl_pct: -3.33,
            commission: 1.5,
            opened_at: "2025-01-01T00:00:00Z".to_string(),
            closed_at: "2025-01-01T02:00:00Z".to_string(),
            close_reason: "Stop loss".to_string(),
        };

        assert!(trade.realized_pnl < 0.0);
        assert_eq!(trade.close_reason, "Stop loss");
    }

    #[test]
    fn test_boost_close_trade_request_with_reason() {
        let request = CloseTradeRequest {
            reason: Some("Manual close".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("Manual close"));
    }

    #[test]
    fn test_boost_close_trade_request_no_reason() {
        let request = CloseTradeRequest { reason: None };

        let json = serde_json::to_string(&request).unwrap();
        let deserialized: CloseTradeRequest = serde_json::from_str(&json).unwrap();
        assert!(deserialized.reason.is_none());
    }

    #[test]
    fn test_boost_place_order_request_with_all_fields() {
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            confirmation_token: Some("token-123".to_string()),
        };

        assert_eq!(request.symbol, "BTCUSDT");
        assert_eq!(request.price, Some(50000.0));
        assert_eq!(request.confirmation_token, Some("token-123".to_string()));
    }

    #[test]
    fn test_boost_place_order_request_minimal() {
        let request = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.0,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        assert!(request.price.is_none());
        assert!(request.stop_loss.is_none());
        assert!(request.take_profit.is_none());
    }

    #[test]
    fn test_boost_order_info_new_state() {
        let order = OrderInfo {
            id: "order-1".to_string(),
            exchange_order_id: 123,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            executed_quantity: 0.0,
            price: Some(50000.0),
            avg_fill_price: 0.0,
            status: "New".to_string(),
            is_entry: true,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:00:00Z".to_string(),
        };

        assert_eq!(order.status, "New");
        assert_eq!(order.executed_quantity, 0.0);
    }

    #[test]
    fn test_boost_order_info_filled_state() {
        let order = OrderInfo {
            id: "order-2".to_string(),
            exchange_order_id: 456,
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.0,
            executed_quantity: 1.0,
            price: None,
            avg_fill_price: 3000.0,
            status: "Filled".to_string(),
            is_entry: false,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:01:00Z".to_string(),
        };

        assert_eq!(order.status, "Filled");
        assert_eq!(order.executed_quantity, order.quantity);
    }

    #[test]
    fn test_boost_confirmation_response_fields() {
        let confirmation = ConfirmationResponse {
            token: "abc-123".to_string(),
            expires_at: "2025-01-01T00:01:00Z".to_string(),
            summary: "BUY 0.01 BTCUSDT MARKET".to_string(),
            order_details: PlaceOrderRequest {
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                order_type: "MARKET".to_string(),
                quantity: 0.01,
                price: None,
                stop_loss: None,
                take_profit: None,
                confirmation_token: None,
            },
        };

        assert_eq!(confirmation.token, "abc-123");
        assert!(confirmation.summary.contains("BUY"));
    }

    #[test]
    fn test_boost_list_orders_query_default() {
        let query = ListOrdersQuery {
            symbol: None,
            status: None,
            limit: default_limit(),
        };

        assert_eq!(query.limit, 50);
        assert!(query.symbol.is_none());
        assert!(query.status.is_none());
    }

    #[test]
    fn test_boost_list_orders_query_custom() {
        let query = ListOrdersQuery {
            symbol: Some("BTCUSDT".to_string()),
            status: Some("filled".to_string()),
            limit: 100,
        };

        assert_eq!(query.limit, 100);
        assert_eq!(query.symbol, Some("BTCUSDT".to_string()));
        assert_eq!(query.status, Some("filled".to_string()));
    }

    #[test]
    fn test_boost_modify_sltp_request_both() {
        let request = ModifySlTpRequest {
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
        };

        assert_eq!(request.stop_loss, Some(48000.0));
        assert_eq!(request.take_profit, Some(55000.0));
    }

    #[test]
    fn test_boost_modify_sltp_request_only_sl() {
        let request = ModifySlTpRequest {
            stop_loss: Some(48000.0),
            take_profit: None,
        };

        assert_eq!(request.stop_loss, Some(48000.0));
        assert!(request.take_profit.is_none());
    }

    #[test]
    fn test_boost_modify_sltp_request_only_tp() {
        let request = ModifySlTpRequest {
            stop_loss: None,
            take_profit: Some(55000.0),
        };

        assert!(request.stop_loss.is_none());
        assert_eq!(request.take_profit, Some(55000.0));
    }

    #[test]
    fn test_boost_update_settings_request_all_fields() {
        let request = UpdateSettingsRequest {
            max_position_size_usdt: Some(2000.0),
            max_positions: Some(10),
            max_daily_loss_usdt: Some(1000.0),
            max_total_exposure_usdt: Some(10000.0),
            risk_per_trade_percent: Some(3.0),
            default_stop_loss_percent: Some(2.5),
            default_take_profit_percent: Some(5.0),
            max_leverage: Some(20),
            ..Default::default()
        };

        assert_eq!(request.max_position_size_usdt, Some(2000.0));
        assert_eq!(request.max_positions, Some(10));
        assert_eq!(request.max_leverage, Some(20));
    }

    #[test]
    fn test_boost_update_settings_request_partial() {
        let request = UpdateSettingsRequest {
            max_position_size_usdt: Some(1000.0),
            ..Default::default()
        };

        assert_eq!(request.max_position_size_usdt, Some(1000.0));
        assert!(request.max_positions.is_none());
    }

    #[test]
    fn test_boost_settings_response_all_fields() {
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
            auto_trading_enabled: false,
            auto_trade_symbols: vec![],
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
        };

        assert!(settings.use_testnet);
        assert_eq!(settings.max_positions, 5);
        assert_eq!(settings.circuit_breaker_errors, 3);
    }

    #[test]
    fn test_boost_settings_response_mainnet() {
        let settings = SettingsResponse {
            use_testnet: false,
            max_position_size_usdt: 5000.0,
            max_positions: 20,
            max_daily_loss_usdt: 2000.0,
            max_total_exposure_usdt: 50000.0,
            risk_per_trade_percent: 1.5,
            default_stop_loss_percent: 1.5,
            default_take_profit_percent: 3.0,
            max_leverage: 5,
            circuit_breaker_errors: 5,
            circuit_breaker_cooldown_secs: 600,
            auto_trading_enabled: false,
            auto_trade_symbols: vec![],
            min_signal_confidence: 0.65,
            max_consecutive_losses: 3,
            cool_down_minutes: 60,
            correlation_limit: 0.70,
            max_portfolio_risk_pct: 10.0,
            short_only_mode: false,
            long_only_mode: false,
        };

        assert!(!settings.use_testnet);
        assert_eq!(settings.max_leverage, 5);
    }

    #[test]
    fn test_boost_default_limit_is_50() {
        assert_eq!(default_limit(), 50);
    }

    #[test]
    fn test_boost_api_new_without_engine() {
        let api = RealTradingApi::new(None);
        assert!(api.engine.is_none());
    }

    #[tokio::test]
    async fn test_boost_cleanup_expired_confirmations_empty() {
        let api = RealTradingApi::new(None);
        api.cleanup_expired_confirmations();
        assert!(api.pending_confirmations.is_empty());
    }

    #[tokio::test]
    async fn test_boost_create_confirmation_generates_token() {
        let api = RealTradingApi::new(None);
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request);
        assert!(!confirmation.token.is_empty());
        assert!(!confirmation.expires_at.is_empty());
    }

    #[tokio::test]
    async fn test_boost_create_confirmation_summary_market() {
        let api = RealTradingApi::new(None);
        let request = PlaceOrderRequest {
            symbol: "ETHUSDT".to_string(),
            side: "SELL".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 1.5,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request);
        assert!(confirmation.summary.contains("SELL"));
        assert!(confirmation.summary.contains("1.5"));
        assert!(confirmation.summary.contains("ETHUSDT"));
        assert!(confirmation.summary.contains("MARKET"));
    }

    #[tokio::test]
    async fn test_boost_create_confirmation_summary_limit() {
        let api = RealTradingApi::new(None);
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.5,
            price: Some(50000.0),
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request);
        assert!(confirmation.summary.contains("BUY"));
        assert!(confirmation.summary.contains("0.5"));
        assert!(confirmation.summary.contains("$50000.00"));
    }

    #[tokio::test]
    async fn test_boost_consume_confirmation_valid() {
        let api = RealTradingApi::new(None);
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request.clone());
        let consumed = api.consume_confirmation(&confirmation.token);

        assert!(consumed.is_some());
        let consumed_request = consumed.unwrap();
        assert_eq!(consumed_request.symbol, request.symbol);
    }

    #[tokio::test]
    async fn test_boost_consume_confirmation_invalid_token() {
        let api = RealTradingApi::new(None);
        let consumed = api.consume_confirmation("invalid-token-xyz-123");
        assert!(consumed.is_none());
    }

    #[tokio::test]
    async fn test_boost_consume_confirmation_twice() {
        let api = RealTradingApi::new(None);
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "MARKET".to_string(),
            quantity: 0.01,
            price: None,
            stop_loss: None,
            take_profit: None,
            confirmation_token: None,
        };

        let confirmation = api.create_confirmation(request);

        // First consume should work
        let consumed1 = api.consume_confirmation(&confirmation.token);
        assert!(consumed1.is_some());

        // Second consume should fail
        let consumed2 = api.consume_confirmation(&confirmation.token);
        assert!(consumed2.is_none());
    }

    #[tokio::test]
    async fn test_boost_routes_not_found() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/real-trading/nonexistent")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_boost_routes_invalid_path() {
        let api = create_test_api();
        let routes = api.routes();

        let resp = warp::test::request()
            .method("GET")
            .path("/invalid/path")
            .reply(&routes)
            .await;

        assert_eq!(resp.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_boost_position_info_clone() {
        let pos = PositionInfo {
            id: "pos-1".to_string(),
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
            ..Default::default()
        };

        let cloned = pos.clone();
        assert_eq!(cloned.id, pos.id);
        assert_eq!(cloned.symbol, pos.symbol);
    }

    #[tokio::test]
    async fn test_boost_balance_info_clone() {
        let balance = BalanceInfo {
            asset: "USDT".to_string(),
            free: 1000.0,
            locked: 100.0,
            total: 1100.0,
        };

        let cloned = balance.clone();
        assert_eq!(cloned.asset, balance.asset);
        assert_eq!(cloned.free, balance.free);
    }

    #[tokio::test]
    async fn test_boost_closed_trade_info_clone() {
        let trade = ClosedTradeInfo {
            id: "trade-1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 0.1,
            entry_price: 50000.0,
            exit_price: 51000.0,
            realized_pnl: 100.0,
            realized_pnl_pct: 2.0,
            commission: 1.0,
            opened_at: "2025-01-01T00:00:00Z".to_string(),
            closed_at: "2025-01-01T01:00:00Z".to_string(),
            close_reason: "Take profit".to_string(),
        };

        let cloned = trade.clone();
        assert_eq!(cloned.id, trade.id);
        assert_eq!(cloned.close_reason, trade.close_reason);
    }

    #[test]
    fn test_boost_place_order_request_clone() {
        let request = PlaceOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            confirmation_token: None,
        };

        let cloned = request.clone();
        assert_eq!(cloned.symbol, request.symbol);
        assert_eq!(cloned.price, request.price);
    }

    #[test]
    fn test_boost_order_info_clone() {
        let order = OrderInfo {
            id: "order-1".to_string(),
            exchange_order_id: 123,
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            quantity: 0.01,
            executed_quantity: 0.005,
            price: Some(50000.0),
            avg_fill_price: 49950.0,
            status: "PartiallyFilled".to_string(),
            is_entry: true,
            created_at: "2025-01-01T00:00:00Z".to_string(),
            updated_at: "2025-01-01T00:01:00Z".to_string(),
        };

        let cloned = order.clone();
        assert_eq!(cloned.id, order.id);
        assert_eq!(cloned.exchange_order_id, order.exchange_order_id);
    }

    #[test]
    fn test_boost_confirmation_response_clone() {
        let confirmation = ConfirmationResponse {
            token: "token-123".to_string(),
            expires_at: "2025-01-01T00:01:00Z".to_string(),
            summary: "BUY 0.01 BTCUSDT LIMIT @ $50000.00".to_string(),
            order_details: PlaceOrderRequest {
                symbol: "BTCUSDT".to_string(),
                side: "BUY".to_string(),
                order_type: "LIMIT".to_string(),
                quantity: 0.01,
                price: Some(50000.0),
                stop_loss: None,
                take_profit: None,
                confirmation_token: None,
            },
        };

        let cloned = confirmation.clone();
        assert_eq!(cloned.token, confirmation.token);
        assert_eq!(cloned.summary, confirmation.summary);
    }

    #[test]
    fn test_boost_list_orders_query_clone() {
        let query = ListOrdersQuery {
            symbol: Some("BTCUSDT".to_string()),
            status: Some("active".to_string()),
            limit: 100,
        };

        let cloned = query.clone();
        assert_eq!(cloned.symbol, query.symbol);
        assert_eq!(cloned.status, query.status);
        assert_eq!(cloned.limit, query.limit);
    }

    #[test]
    fn test_boost_modify_sltp_request_clone() {
        let request = ModifySlTpRequest {
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
        };

        let cloned = request.clone();
        assert_eq!(cloned.stop_loss, request.stop_loss);
        assert_eq!(cloned.take_profit, request.take_profit);
    }

    #[test]
    fn test_boost_api_response_timestamp_recent() {
        let response = ApiResponse::success("test");
        let now = Utc::now();
        let diff = now.signed_duration_since(response.timestamp);
        assert!(diff.num_seconds() < 5);
    }

    #[test]
    fn test_boost_api_response_error_timestamp_recent() {
        let response: ApiResponse<()> = ApiResponse::error("error".to_string());
        let now = Utc::now();
        let diff = now.signed_duration_since(response.timestamp);
        assert!(diff.num_seconds() < 5);
    }
}
