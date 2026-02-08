use rust_decimal::Decimal;
use serde::{Deserialize, Deserializer, Serialize};

/// Helper function to deserialize bool from various formats (string, bool, number)
fn deserialize_bool_from_anything<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum BoolOrString {
        Bool(bool),
        String(String),
        Number(i64),
    }

    match BoolOrString::deserialize(deserializer)? {
        BoolOrString::Bool(b) => Ok(b),
        BoolOrString::String(s) => match s.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(true),
            "false" | "0" | "no" | "" => Ok(false),
            _ => Err(D::Error::custom(format!("Invalid boolean value: {}", s))),
        },
        BoolOrString::Number(n) => Ok(n != 0),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Kline {
    pub open_time: i64,
    pub open: String,
    pub high: String,
    pub low: String,
    pub close: String,
    pub volume: String,
    pub close_time: i64,
    pub quote_asset_volume: String,
    pub number_of_trades: i64,
    pub taker_buy_base_asset_volume: String,
    pub taker_buy_quote_asset_volume: String,
    pub ignore: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolPrice {
    pub symbol: String,
    pub price: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FundingRate {
    pub symbol: String,
    pub funding_rate: String,
    pub funding_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "k")]
    pub kline: KlineData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlineData {
    #[serde(rename = "t")]
    pub kline_start_time: i64,
    #[serde(rename = "T")]
    pub kline_close_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "i")]
    pub interval: String,
    #[serde(rename = "f")]
    pub first_trade_id: i64,
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    #[serde(rename = "o")]
    pub open_price: String,
    #[serde(rename = "c")]
    pub close_price: String,
    #[serde(rename = "h")]
    pub high_price: String,
    #[serde(rename = "l")]
    pub low_price: String,
    #[serde(rename = "v")]
    pub base_asset_volume: String,
    #[serde(rename = "n")]
    pub number_of_trades: i64,
    #[serde(rename = "x")]
    pub is_this_kline_closed: bool,
    #[serde(rename = "q")]
    pub quote_asset_volume: String,
    #[serde(rename = "V")]
    pub taker_buy_base_asset_volume: String,
    #[serde(rename = "Q")]
    pub taker_buy_quote_asset_volume: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "p")]
    pub price_change: String,
    #[serde(rename = "P")]
    pub price_change_percent: String,
    #[serde(rename = "w")]
    pub weighted_avg_price: String,
    #[serde(rename = "x")]
    pub prev_close_price: String,
    #[serde(rename = "c")]
    pub last_price: String,
    #[serde(rename = "Q")]
    pub last_quantity: String,
    #[serde(rename = "b")]
    pub best_bid_price: String,
    #[serde(rename = "B")]
    pub best_bid_quantity: String,
    #[serde(rename = "a")]
    pub best_ask_price: String,
    #[serde(rename = "A")]
    pub best_ask_quantity: String,
    #[serde(rename = "o")]
    pub open_price: String,
    #[serde(rename = "h")]
    pub high_price: String,
    #[serde(rename = "l")]
    pub low_price: String,
    #[serde(rename = "v")]
    pub total_traded_base_asset_volume: String,
    #[serde(rename = "q")]
    pub total_traded_quote_asset_volume: String,
    #[serde(rename = "O")]
    pub statistics_open_time: i64,
    #[serde(rename = "C")]
    pub statistics_close_time: i64,
    #[serde(rename = "F")]
    pub first_trade_id: i64,
    #[serde(rename = "L")]
    pub last_trade_id: i64,
    #[serde(rename = "n")]
    pub total_number_of_trades: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderBookEvent {
    #[serde(rename = "e")]
    pub event_type: String,
    #[serde(rename = "E")]
    pub event_time: i64,
    #[serde(rename = "s")]
    pub symbol: String,
    #[serde(rename = "U")]
    pub first_update_id: i64,
    #[serde(rename = "u")]
    pub final_update_id: i64,
    #[serde(rename = "b")]
    pub bids: Vec<(String, String)>,
    #[serde(rename = "a")]
    pub asks: Vec<(String, String)>,
}

/// Futures order structure (for open orders query)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesOrder {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub order_id: i64,
    #[serde(default)]
    pub order_list_id: i64,
    #[serde(default)]
    pub client_order_id: String,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub orig_qty: String,
    #[serde(default)]
    pub executed_qty: String,
    #[serde(default, rename = "cumQuoteQty")]
    pub cumulative_quote_qty: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub time_in_force: String,
    #[serde(default, rename = "type")]
    pub r#type: String,
    #[serde(default)]
    pub side: String,
    #[serde(default)]
    pub stop_price: String,
    #[serde(default)]
    pub iceberg_qty: String,
    #[serde(default)]
    pub time: i64,
    #[serde(default)]
    pub update_time: i64,
    #[serde(default)]
    pub is_working: bool,
    #[serde(default)]
    pub orig_quote_order_qty: String,
}

/// Response from placing a futures order
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesOrderResponse {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub order_id: i64,
    #[serde(default)]
    pub client_order_id: String,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub orig_qty: String,
    #[serde(default)]
    pub executed_qty: String,
    #[serde(default, rename = "cumQuoteQty")]
    pub cumulative_quote_qty: String,
    #[serde(default, rename = "cumQty")]
    pub cum_qty: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub time_in_force: String,
    #[serde(default, rename = "type")]
    pub order_type: String,
    #[serde(default)]
    pub side: String,
    #[serde(default)]
    pub stop_price: String,
    #[serde(default)]
    pub avg_price: String,
    #[serde(default)]
    pub position_side: String,
    #[serde(default)]
    pub reduce_only: bool,
    #[serde(default)]
    pub close_position: bool,
    #[serde(default)]
    pub update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FuturesPosition {
    #[serde(default)]
    pub symbol: String,
    #[serde(default)]
    pub position_amt: String,
    #[serde(default)]
    pub entry_price: String,
    #[serde(default)]
    pub mark_price: String,
    #[serde(default, rename = "unRealizedProfit")]
    pub unrealized_pnl: String,
    #[serde(default)]
    pub liquidation_price: String,
    #[serde(default)]
    pub leverage: String,
    #[serde(default)]
    pub max_notional_value: String,
    #[serde(default)]
    pub margin_type: String,
    #[serde(default)]
    pub isolated_margin: String,
    #[serde(default, deserialize_with = "deserialize_bool_from_anything")]
    pub is_auto_add_margin: bool,
    #[serde(default)]
    pub position_side: String,
    #[serde(default)]
    pub notional: String,
    #[serde(default)]
    pub isolated_wallet: String,
    #[serde(default)]
    pub update_time: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountInfo {
    #[serde(default)]
    pub maker_commission: i64,
    #[serde(default)]
    pub taker_commission: i64,
    #[serde(default)]
    pub buyer_commission: i64,
    #[serde(default)]
    pub seller_commission: i64,
    #[serde(default)]
    pub can_trade: bool,
    #[serde(default)]
    pub can_withdraw: bool,
    #[serde(default)]
    pub can_deposit: bool,
    #[serde(default)]
    pub update_time: i64,
    #[serde(default)]
    pub account_type: String,
    #[serde(default)]
    pub balances: Vec<Balance>,
    #[serde(default)]
    pub permissions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: String,
    pub locked: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewOrderRequest {
    pub symbol: String,
    pub side: String,
    pub r#type: String,
    pub quantity: Option<String>,
    pub quote_order_qty: Option<String>,
    pub price: Option<String>,
    pub new_client_order_id: Option<String>,
    pub stop_price: Option<String>,
    pub iceberg_qty: Option<String>,
    pub new_order_resp_type: Option<String>,
    pub time_in_force: Option<String>,
    pub reduce_only: Option<bool>,
    pub close_position: Option<bool>,
    pub position_side: Option<String>,
    pub working_type: Option<String>,
    pub price_protect: Option<bool>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderResponse {
    pub symbol: String,
    pub order_id: i64,
    pub order_list_id: i64,
    pub client_order_id: String,
    pub transact_time: i64,
    pub price: String,
    pub orig_qty: String,
    pub executed_qty: String,
    pub cumulative_quote_qty: String,
    pub status: String,
    pub time_in_force: String,
    pub r#type: String,
    pub side: String,
    pub fills: Vec<Fill>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Fill {
    pub price: String,
    pub qty: String,
    pub commission: String,
    pub commission_asset: String,
    pub trade_id: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketMessage {
    pub stream: String,
    pub data: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "e")]
pub enum StreamEvent {
    #[serde(rename = "kline")]
    Kline(KlineEvent),
    #[serde(rename = "24hrTicker")]
    Ticker(TickerEvent),
    #[serde(rename = "depthUpdate")]
    OrderBook(OrderBookEvent),
}

// NEW: WebSocket events for chart data updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartUpdateEvent {
    pub symbol: String,
    pub timeframe: String,
    pub candle: ChartCandle,
    pub latest_price: f64,
    pub price_change_24h: f64,
    pub price_change_percent_24h: f64,
    pub volume_24h: f64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartCandle {
    pub timestamp: i64,
    pub open: f64,
    pub high: f64,
    pub low: f64,
    pub close: f64,
    pub volume: f64,
    pub is_closed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketDataUpdate {
    pub symbol: String,
    pub price: f64,
    pub price_change_24h: f64,
    pub price_change_percent_24h: f64,
    pub volume_24h: f64,
    pub timestamp: i64,
}

// Extended StreamEvent for chart data
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WebSocketEvent {
    #[serde(rename = "kline")]
    Kline(KlineEvent),
    #[serde(rename = "ticker")]
    Ticker(TickerEvent),
    #[serde(rename = "orderbook")]
    OrderBook(OrderBookEvent),
    #[serde(rename = "chart_update")]
    ChartUpdate(ChartUpdateEvent),
    #[serde(rename = "market_data")]
    MarketData(MarketDataUpdate),
    #[serde(rename = "error")]
    Error { message: String },
}

// ============================================================================
// SPOT ORDER TYPES (Phase 1: Real Trading)
// @spec:FR-REAL-001, FR-REAL-002, FR-REAL-003, FR-REAL-004
// @ref:specs/02-design/2.3-api/API-RUST-CORE.md
// ============================================================================

/// Order side for spot trading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderSide::Buy => write!(f, "BUY"),
            OrderSide::Sell => write!(f, "SELL"),
        }
    }
}

/// Order type for spot trading
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SpotOrderType {
    Market,
    Limit,
    StopLossLimit,
    TakeProfitLimit,
    LimitMaker,
}

impl std::fmt::Display for SpotOrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpotOrderType::Market => write!(f, "MARKET"),
            SpotOrderType::Limit => write!(f, "LIMIT"),
            SpotOrderType::StopLossLimit => write!(f, "STOP_LOSS_LIMIT"),
            SpotOrderType::TakeProfitLimit => write!(f, "TAKE_PROFIT_LIMIT"),
            SpotOrderType::LimitMaker => write!(f, "LIMIT_MAKER"),
        }
    }
}

/// Time in force options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeInForce {
    /// Good Till Cancelled
    Gtc,
    /// Immediate or Cancel
    Ioc,
    /// Fill or Kill
    Fok,
}

impl std::fmt::Display for TimeInForce {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TimeInForce::Gtc => write!(f, "GTC"),
            TimeInForce::Ioc => write!(f, "IOC"),
            TimeInForce::Fok => write!(f, "FOK"),
        }
    }
}

/// Spot order request - for placing orders on spot market
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpotOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: SpotOrderType,
    /// Quantity in base asset (e.g., BTC for BTCUSDT)
    pub quantity: Option<String>,
    /// Quantity in quote asset (e.g., USDT for BTCUSDT) - for MARKET orders
    pub quote_order_qty: Option<String>,
    /// Price for LIMIT orders
    pub price: Option<String>,
    /// Stop/trigger price for STOP_LOSS_LIMIT and TAKE_PROFIT_LIMIT
    pub stop_price: Option<String>,
    /// Time in force
    pub time_in_force: Option<TimeInForce>,
    /// Custom order ID for tracking
    pub client_order_id: Option<String>,
    /// Iceberg quantity
    pub iceberg_qty: Option<String>,
    /// Response type: ACK, RESULT, or FULL
    pub new_order_resp_type: Option<String>,
}

impl SpotOrderRequest {
    /// Create a market order
    pub fn market(symbol: &str, side: OrderSide, quantity: &str) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            side,
            order_type: SpotOrderType::Market,
            quantity: Some(quantity.to_string()),
            quote_order_qty: None,
            price: None,
            stop_price: None,
            time_in_force: None,
            client_order_id: None,
            iceberg_qty: None,
            new_order_resp_type: Some("FULL".to_string()),
        }
    }

    /// Create a limit order
    pub fn limit(symbol: &str, side: OrderSide, quantity: &str, price: &str) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            side,
            order_type: SpotOrderType::Limit,
            quantity: Some(quantity.to_string()),
            quote_order_qty: None,
            price: Some(price.to_string()),
            stop_price: None,
            time_in_force: Some(TimeInForce::Gtc),
            client_order_id: None,
            iceberg_qty: None,
            new_order_resp_type: Some("FULL".to_string()),
        }
    }

    /// Create a stop loss limit order
    pub fn stop_loss_limit(
        symbol: &str,
        side: OrderSide,
        quantity: &str,
        price: &str,
        stop_price: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            side,
            order_type: SpotOrderType::StopLossLimit,
            quantity: Some(quantity.to_string()),
            quote_order_qty: None,
            price: Some(price.to_string()),
            stop_price: Some(stop_price.to_string()),
            time_in_force: Some(TimeInForce::Gtc),
            client_order_id: None,
            iceberg_qty: None,
            new_order_resp_type: Some("FULL".to_string()),
        }
    }

    /// Create a take profit limit order
    pub fn take_profit_limit(
        symbol: &str,
        side: OrderSide,
        quantity: &str,
        price: &str,
        stop_price: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            side,
            order_type: SpotOrderType::TakeProfitLimit,
            quantity: Some(quantity.to_string()),
            quote_order_qty: None,
            price: Some(price.to_string()),
            stop_price: Some(stop_price.to_string()),
            time_in_force: Some(TimeInForce::Gtc),
            client_order_id: None,
            iceberg_qty: None,
            new_order_resp_type: Some("FULL".to_string()),
        }
    }

    /// Set custom client order ID
    pub fn with_client_order_id(mut self, client_order_id: &str) -> Self {
        self.client_order_id = Some(client_order_id.to_string());
        self
    }

    /// Set time in force
    pub fn with_time_in_force(mut self, tif: TimeInForce) -> Self {
        self.time_in_force = Some(tif);
        self
    }

    /// Set quote order quantity (for market orders - buy X amount in quote currency)
    pub fn with_quote_qty(mut self, quote_qty: &str) -> Self {
        self.quote_order_qty = Some(quote_qty.to_string());
        self
    }
}

/// Spot order response from Binance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpotOrderResponse {
    pub symbol: String,
    pub order_id: i64,
    pub order_list_id: Option<i64>,
    pub client_order_id: String,
    pub transact_time: i64,
    #[serde(default)]
    pub price: String,
    #[serde(default)]
    pub orig_qty: String,
    #[serde(default)]
    pub executed_qty: String,
    #[serde(default, rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: String,
    pub time_in_force: Option<String>,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
    #[serde(default)]
    pub fills: Vec<Fill>,
}

/// Query order response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct QueryOrderResponse {
    pub symbol: String,
    pub order_id: i64,
    pub order_list_id: Option<i64>,
    pub client_order_id: String,
    pub price: String,
    pub orig_qty: String,
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
    pub stop_price: Option<String>,
    pub iceberg_qty: Option<String>,
    pub time: i64,
    pub update_time: i64,
    pub is_working: bool,
    pub orig_quote_order_qty: Option<String>,
}

/// Cancel order response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOrderResponse {
    pub symbol: String,
    pub orig_client_order_id: Option<String>,
    pub order_id: i64,
    pub order_list_id: Option<i64>,
    pub client_order_id: String,
    pub price: String,
    pub orig_qty: String,
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
}

// ============================================================================
// OCO ORDER TYPES (Phase 2: Advanced Order Types)
// @spec:FR-REAL-010, FR-REAL-011
// @ref:specs/02-design/2.3-api/API-RUST-CORE.md
// ============================================================================

/// OCO (One-Cancels-Other) Order Request
/// Creates a pair of orders: limit order + stop-loss order
/// When one fills, the other is automatically cancelled
#[derive(Debug, Clone, Serialize, Deserialize)]
/// OCO Order Request - New API format (2024+)
/// For LONG positions (Sell OCO): Take profit ABOVE current price, Stop loss BELOW current price
/// For SHORT positions (Buy OCO): Take profit BELOW current price, Stop loss ABOVE current price
pub struct OcoOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    /// Quantity for both orders
    pub quantity: String,
    /// Type for the order above current price (LIMIT_MAKER, STOP_LOSS, STOP_LOSS_LIMIT)
    pub above_type: String,
    /// Price for above order (take profit for LONG/sell)
    pub above_price: Option<String>,
    /// Type for the order below current price (LIMIT_MAKER, STOP_LOSS, STOP_LOSS_LIMIT)
    pub below_type: String,
    /// Stop trigger price for below order
    pub below_stop_price: Option<String>,
    /// Limit price for below order (for STOP_LOSS_LIMIT)
    pub below_price: Option<String>,
    /// Time in force for above order
    pub above_time_in_force: Option<TimeInForce>,
    /// Time in force for below order
    pub below_time_in_force: Option<TimeInForce>,
    /// Custom order ID for the entire OCO list
    pub list_client_order_id: Option<String>,
    /// Custom order ID for the above order
    pub above_client_order_id: Option<String>,
    /// Custom order ID for the below order
    pub below_client_order_id: Option<String>,
    /// Response type: ACK, RESULT, or FULL
    pub new_order_resp_type: Option<String>,
}

impl OcoOrderRequest {
    /// Create a new OCO order for LONG position protection (Sell OCO)
    /// - Take profit: LIMIT_MAKER above current price
    /// - Stop loss: STOP_LOSS_LIMIT below current price
    pub fn new(
        symbol: &str,
        side: OrderSide,
        quantity: &str,
        take_profit_price: &str,
        stop_loss_trigger: &str,
        stop_loss_limit: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            side,
            quantity: quantity.to_string(),
            // Above = Take Profit (LIMIT_MAKER - maker order that provides liquidity)
            above_type: "LIMIT_MAKER".to_string(),
            above_price: Some(take_profit_price.to_string()),
            // Below = Stop Loss (STOP_LOSS_LIMIT - triggered when price drops)
            below_type: "STOP_LOSS_LIMIT".to_string(),
            below_stop_price: Some(stop_loss_trigger.to_string()),
            below_price: Some(stop_loss_limit.to_string()),
            above_time_in_force: None, // LIMIT_MAKER doesn't need TIF
            below_time_in_force: Some(TimeInForce::Gtc),
            list_client_order_id: None,
            above_client_order_id: None,
            below_client_order_id: None,
            new_order_resp_type: Some("FULL".to_string()),
        }
    }

    /// Create OCO for SHORT position (Buy OCO)
    /// - Take profit: LIMIT_MAKER below current price
    /// - Stop loss: STOP_LOSS_LIMIT above current price
    pub fn new_short(
        symbol: &str,
        quantity: &str,
        take_profit_price: &str,
        stop_loss_trigger: &str,
        stop_loss_limit: &str,
    ) -> Self {
        Self {
            symbol: symbol.to_uppercase(),
            side: OrderSide::Buy,
            quantity: quantity.to_string(),
            // For SHORT: Above = Stop Loss
            above_type: "STOP_LOSS_LIMIT".to_string(),
            above_price: Some(stop_loss_limit.to_string()),
            // For SHORT: Below = Take Profit
            below_type: "LIMIT_MAKER".to_string(),
            below_stop_price: Some(stop_loss_trigger.to_string()),
            below_price: Some(take_profit_price.to_string()),
            above_time_in_force: Some(TimeInForce::Gtc),
            below_time_in_force: None,
            list_client_order_id: None,
            above_client_order_id: None,
            below_client_order_id: None,
            new_order_resp_type: Some("FULL".to_string()),
        }
    }

    /// Add custom order IDs for tracking
    pub fn with_client_order_ids(mut self, list_id: &str, above_id: &str, below_id: &str) -> Self {
        self.list_client_order_id = Some(list_id.to_string());
        self.above_client_order_id = Some(above_id.to_string());
        self.below_client_order_id = Some(below_id.to_string());
        self
    }
}

/// OCO Order Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcoOrderResponse {
    pub order_list_id: i64,
    pub contingency_type: String,
    pub list_status_type: String,
    pub list_order_status: String,
    pub list_client_order_id: String,
    pub transaction_time: i64,
    pub symbol: String,
    pub orders: Vec<OcoOrderInfo>,
    #[serde(default)]
    pub order_reports: Vec<OcoOrderReport>,
}

/// Order info within OCO response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcoOrderInfo {
    pub symbol: String,
    pub order_id: i64,
    pub client_order_id: String,
}

/// Detailed order report within OCO response (when newOrderRespType=FULL)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OcoOrderReport {
    pub symbol: String,
    pub order_id: i64,
    pub order_list_id: i64,
    pub client_order_id: String,
    pub transact_time: i64,
    pub price: String,
    pub orig_qty: String,
    pub executed_qty: String,
    #[serde(rename = "cummulativeQuoteQty")]
    pub cumulative_quote_qty: String,
    pub status: String,
    pub time_in_force: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub side: String,
    #[serde(default)]
    pub stop_price: String,
}

/// Cancel OCO Response
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelOcoResponse {
    pub order_list_id: i64,
    pub contingency_type: String,
    pub list_status_type: String,
    pub list_order_status: String,
    pub list_client_order_id: String,
    pub transaction_time: i64,
    pub symbol: String,
    pub orders: Vec<OcoOrderInfo>,
    pub order_reports: Vec<OcoOrderReport>,
}

// ============================================================================
// USER DATA STREAM TYPES (Phase 1: Real Trading)
// @spec:FR-REAL-007, FR-REAL-008
// @ref:plans/20251203-1353-binance-real-trading-system/research/researcher-02-binance-websocket-userdata.md
// ============================================================================

/// Listen key response from Binance
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListenKeyResponse {
    pub listen_key: String,
}

/// Execution report from WebSocket user data stream
/// This is sent when order status changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// Event type - always "executionReport"
    /// Note: When deserializing via UserDataEvent (internally tagged by "e"),
    /// this field won't be present as serde consumes it for variant selection.
    #[serde(rename = "e", default)]
    pub event_type: String,
    /// Event time
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Symbol
    #[serde(rename = "s")]
    pub symbol: String,
    /// Client order ID
    #[serde(rename = "c")]
    pub client_order_id: String,
    /// Side (BUY/SELL)
    #[serde(rename = "S")]
    pub side: String,
    /// Order type
    #[serde(rename = "o")]
    pub order_type: String,
    /// Time in force
    #[serde(rename = "f")]
    pub time_in_force: String,
    /// Order quantity
    #[serde(rename = "q")]
    pub order_quantity: String,
    /// Order price
    #[serde(rename = "p")]
    pub order_price: String,
    /// Stop price
    #[serde(rename = "P")]
    pub stop_price: String,
    /// Iceberg quantity
    #[serde(rename = "F")]
    pub iceberg_quantity: String,
    /// Original client order ID (for cancel)
    #[serde(rename = "C")]
    pub original_client_order_id: String,
    /// Current execution type: NEW, CANCELED, REPLACED, REJECTED, TRADE, EXPIRED
    #[serde(rename = "x")]
    pub execution_type: String,
    /// Current order status: NEW, PARTIALLY_FILLED, FILLED, CANCELED, REJECTED, EXPIRED
    #[serde(rename = "X")]
    pub order_status: String,
    /// Order reject reason
    #[serde(rename = "r")]
    pub order_reject_reason: String,
    /// Order ID
    #[serde(rename = "i")]
    pub order_id: i64,
    /// Last executed quantity
    #[serde(rename = "l")]
    pub last_executed_quantity: String,
    /// Cumulative filled quantity
    #[serde(rename = "z")]
    pub cumulative_filled_quantity: String,
    /// Last executed price
    #[serde(rename = "L")]
    pub last_executed_price: String,
    /// Commission amount
    #[serde(rename = "n")]
    pub commission_amount: String,
    /// Commission asset
    #[serde(rename = "N")]
    pub commission_asset: Option<String>,
    /// Transaction time
    #[serde(rename = "T")]
    pub transaction_time: i64,
    /// Trade ID
    #[serde(rename = "t")]
    pub trade_id: i64,
    /// Is the order on the book?
    #[serde(rename = "w")]
    pub is_on_book: bool,
    /// Is this trade the maker side?
    #[serde(rename = "m")]
    pub is_maker: bool,
    /// Order creation time
    #[serde(rename = "O")]
    pub order_creation_time: i64,
    /// Cumulative quote asset transacted quantity
    #[serde(rename = "Z")]
    pub cumulative_quote_qty: String,
    /// Last quote asset transacted quantity
    #[serde(rename = "Y")]
    pub last_quote_qty: String,
    /// Quote order quantity
    #[serde(rename = "Q")]
    pub quote_order_qty: String,
}

impl ExecutionReport {
    /// Check if order is fully filled
    pub fn is_filled(&self) -> bool {
        self.order_status == "FILLED"
    }

    /// Check if order is partially filled
    pub fn is_partially_filled(&self) -> bool {
        self.order_status == "PARTIALLY_FILLED"
    }

    /// Check if order is cancelled
    pub fn is_cancelled(&self) -> bool {
        self.order_status == "CANCELED"
    }

    /// Check if order is rejected
    pub fn is_rejected(&self) -> bool {
        self.order_status == "REJECTED"
    }

    /// Check if order is new (just placed)
    pub fn is_new(&self) -> bool {
        self.execution_type == "NEW"
    }

    /// Check if this is a trade execution
    pub fn is_trade(&self) -> bool {
        self.execution_type == "TRADE"
    }

    /// Get filled percentage
    pub fn fill_percentage(&self) -> f64 {
        let filled: f64 = self.cumulative_filled_quantity.parse().unwrap_or(0.0);
        let total: f64 = self.order_quantity.parse().unwrap_or(1.0);
        if total > 0.0 {
            (filled / total) * 100.0
        } else {
            0.0
        }
    }
}

/// Account position update from WebSocket user data stream
/// This is sent when account balance changes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboundAccountPosition {
    /// Event type - always "outboundAccountPosition"
    /// Note: When deserialized via UserDataEvent tagged enum, this field uses default
    /// because serde consumes "e" for enum discrimination
    #[serde(rename = "e", default)]
    pub event_type: String,
    /// Event time
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Last account update time
    #[serde(rename = "u")]
    pub last_update_time: i64,
    /// Balances array
    #[serde(rename = "B")]
    pub balances: Vec<AccountBalance>,
}

/// Balance update in account position
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountBalance {
    /// Asset
    #[serde(rename = "a")]
    pub asset: String,
    /// Free amount
    #[serde(rename = "f")]
    pub free: String,
    /// Locked amount
    #[serde(rename = "l")]
    pub locked: String,
}

/// Balance update event from WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BalanceUpdate {
    /// Event type - always "balanceUpdate"
    /// Note: When deserialized via UserDataEvent tagged enum, this field uses default
    /// because serde consumes "e" for enum discrimination
    #[serde(rename = "e", default)]
    pub event_type: String,
    /// Event time
    #[serde(rename = "E")]
    pub event_time: i64,
    /// Asset
    #[serde(rename = "a")]
    pub asset: String,
    /// Balance delta
    #[serde(rename = "d")]
    pub balance_delta: String,
    /// Clear time
    #[serde(rename = "T")]
    pub clear_time: i64,
}

/// Union type for all user data stream events
/// Note: ExecutionReport is boxed to reduce enum size difference between variants
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "e")]
pub enum UserDataEvent {
    #[serde(rename = "executionReport")]
    ExecutionReport(Box<ExecutionReport>),
    #[serde(rename = "outboundAccountPosition")]
    AccountPosition(OutboundAccountPosition),
    #[serde(rename = "balanceUpdate")]
    BalanceUpdate(BalanceUpdate),
}

/// User data stream handle for tracking connection
#[derive(Debug, Clone)]
pub struct UserDataStreamHandle {
    pub listen_key: String,
    pub ws_url: String,
    pub created_at: i64,
    pub last_keepalive: i64,
}

impl UserDataStreamHandle {
    pub fn new(listen_key: String, ws_url: String) -> Self {
        let now = chrono::Utc::now().timestamp_millis();
        Self {
            listen_key,
            ws_url,
            created_at: now,
            last_keepalive: now,
        }
    }

    /// Check if keepalive is needed (every 30 minutes)
    pub fn needs_keepalive(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        let thirty_minutes_ms = 30 * 60 * 1000;
        now - self.last_keepalive > thirty_minutes_ms
    }

    /// Check if listen key is expired (60 minutes)
    pub fn is_expired(&self) -> bool {
        let now = chrono::Utc::now().timestamp_millis();
        let sixty_minutes_ms = 60 * 60 * 1000;
        now - self.last_keepalive > sixty_minutes_ms
    }

    /// Update last keepalive time
    pub fn update_keepalive(&mut self) {
        self.last_keepalive = chrono::Utc::now().timestamp_millis();
    }
}

// ============================================================================
// TRADING MODE (Re-exported from config.rs)
// @spec:FR-REAL-011
// ============================================================================
// TradingMode is defined in config.rs to avoid circular dependencies
// Use: crate::config::TradingMode

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

// Utility functions for type conversions
impl Kline {
    pub fn to_decimal_values(
        &self,
    ) -> Result<(Decimal, Decimal, Decimal, Decimal, Decimal), rust_decimal::Error> {
        let open = self.open.parse::<Decimal>()?;
        let high = self.high.parse::<Decimal>()?;
        let low = self.low.parse::<Decimal>()?;
        let close = self.close.parse::<Decimal>()?;
        let volume = self.volume.parse::<Decimal>()?;
        Ok((open, high, low, close, volume))
    }
}

impl KlineData {
    pub fn to_decimal_values(
        &self,
    ) -> Result<(Decimal, Decimal, Decimal, Decimal, Decimal), rust_decimal::Error> {
        let open = self.open_price.parse::<Decimal>()?;
        let high = self.high_price.parse::<Decimal>()?;
        let low = self.low_price.parse::<Decimal>()?;
        let close = self.close_price.parse::<Decimal>()?;
        let volume = self.base_asset_volume.parse::<Decimal>()?;
        Ok((open, high, low, close, volume))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_decimal::prelude::*;

    #[test]
    fn test_kline_serialization() {
        let kline = Kline {
            open_time: 1625097600000,
            open: "34000.00".to_string(),
            high: "35000.00".to_string(),
            low: "33000.00".to_string(),
            close: "34500.00".to_string(),
            volume: "100.5".to_string(),
            close_time: 1625097659999,
            quote_asset_volume: "3450000.00".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
            ignore: "0".to_string(),
        };

        let json = serde_json::to_string(&kline).unwrap();
        assert!(json.contains("34000.00"));
        assert!(json.contains("35000.00"));
    }

    #[test]
    fn test_kline_deserialization() {
        let json = r#"{
            "open_time": 1625097600000,
            "open": "34000.00",
            "high": "35000.00",
            "low": "33000.00",
            "close": "34500.00",
            "volume": "100.5",
            "close_time": 1625097659999,
            "quote_asset_volume": "3450000.00",
            "number_of_trades": 1000,
            "taker_buy_base_asset_volume": "50.25",
            "taker_buy_quote_asset_volume": "1725000.00",
            "ignore": "0"
        }"#;

        let kline: Kline = serde_json::from_str(json).unwrap();
        assert_eq!(kline.open, "34000.00");
        assert_eq!(kline.high, "35000.00");
        assert_eq!(kline.volume, "100.5");
        assert_eq!(kline.number_of_trades, 1000);
    }

    #[test]
    fn test_kline_to_decimal_values() {
        let kline = Kline {
            open_time: 1625097600000,
            open: "34000.00".to_string(),
            high: "35000.00".to_string(),
            low: "33000.00".to_string(),
            close: "34500.00".to_string(),
            volume: "100.5".to_string(),
            close_time: 1625097659999,
            quote_asset_volume: "3450000.00".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
            ignore: "0".to_string(),
        };

        let result = kline.to_decimal_values().unwrap();
        assert_eq!(result.0, Decimal::from_str("34000.00").unwrap());
        assert_eq!(result.1, Decimal::from_str("35000.00").unwrap());
        assert_eq!(result.2, Decimal::from_str("33000.00").unwrap());
        assert_eq!(result.3, Decimal::from_str("34500.00").unwrap());
        assert_eq!(result.4, Decimal::from_str("100.5").unwrap());
    }

    #[test]
    fn test_kline_to_decimal_values_invalid() {
        let kline = Kline {
            open_time: 1625097600000,
            open: "invalid".to_string(),
            high: "35000.00".to_string(),
            low: "33000.00".to_string(),
            close: "34500.00".to_string(),
            volume: "100.5".to_string(),
            close_time: 1625097659999,
            quote_asset_volume: "3450000.00".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
            ignore: "0".to_string(),
        };

        assert!(kline.to_decimal_values().is_err());
    }

    #[test]
    fn test_symbol_price_serialization() {
        let price = SymbolPrice {
            symbol: "BTCUSDT".to_string(),
            price: "34000.50".to_string(),
        };

        let json = serde_json::to_string(&price).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("34000.50"));
    }

    #[test]
    fn test_funding_rate_deserialization() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "funding_rate": "0.0001",
            "funding_time": 1625097600000
        }"#;

        let funding: FundingRate = serde_json::from_str(json).unwrap();
        assert_eq!(funding.symbol, "BTCUSDT");
        assert_eq!(funding.funding_rate, "0.0001");
        assert_eq!(funding.funding_time, 1625097600000);
    }

    #[test]
    fn test_kline_event_deserialization() {
        let json = r#"{
            "e": "kline",
            "E": 1625097600000,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000,
                "T": 1625097659999,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "34000.00",
                "c": "34500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "100.5",
                "n": 1000,
                "x": true,
                "q": "3450000.00",
                "V": "50.25",
                "Q": "1725000.00"
            }
        }"#;

        let event: KlineEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "kline");
        assert_eq!(event.symbol, "BTCUSDT");
        assert_eq!(event.kline.interval, "1m");
        assert!(event.kline.is_this_kline_closed);
    }

    #[test]
    fn test_kline_data_to_decimal_values() {
        let kline_data = KlineData {
            kline_start_time: 1625097600000,
            kline_close_time: 1625097659999,
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            first_trade_id: 100,
            last_trade_id: 200,
            open_price: "34000.00".to_string(),
            close_price: "34500.00".to_string(),
            high_price: "35000.00".to_string(),
            low_price: "33000.00".to_string(),
            base_asset_volume: "100.5".to_string(),
            number_of_trades: 1000,
            is_this_kline_closed: true,
            quote_asset_volume: "3450000.00".to_string(),
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
        };

        let result = kline_data.to_decimal_values().unwrap();
        assert_eq!(result.0, Decimal::from_str("34000.00").unwrap());
        assert_eq!(result.1, Decimal::from_str("35000.00").unwrap());
        assert_eq!(result.2, Decimal::from_str("33000.00").unwrap());
        assert_eq!(result.3, Decimal::from_str("34500.00").unwrap());
        assert_eq!(result.4, Decimal::from_str("100.5").unwrap());
    }

    #[test]
    fn test_ticker_event_deserialization() {
        let json = r#"{
            "e": "24hrTicker",
            "E": 1625097600000,
            "s": "BTCUSDT",
            "p": "500.00",
            "P": "1.5",
            "w": "34250.00",
            "x": "33500.00",
            "c": "34000.00",
            "Q": "10.5",
            "b": "33990.00",
            "B": "5.0",
            "a": "34010.00",
            "A": "4.5",
            "o": "33500.00",
            "h": "35000.00",
            "l": "33000.00",
            "v": "1000.5",
            "q": "34000000.00",
            "O": 1625011200000,
            "C": 1625097600000,
            "F": 1000,
            "L": 5000,
            "n": 4000
        }"#;

        let event: TickerEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "24hrTicker");
        assert_eq!(event.symbol, "BTCUSDT");
        assert_eq!(event.last_price, "34000.00");
        assert_eq!(event.total_number_of_trades, 4000);
    }

    #[test]
    fn test_order_book_event_deserialization() {
        let json = r#"{
            "e": "depthUpdate",
            "E": 1625097600000,
            "s": "BTCUSDT",
            "U": 1000,
            "u": 1005,
            "b": [["34000.00", "1.5"], ["33999.00", "2.0"]],
            "a": [["34001.00", "1.0"], ["34002.00", "0.5"]]
        }"#;

        let event: OrderBookEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.event_type, "depthUpdate");
        assert_eq!(event.symbol, "BTCUSDT");
        assert_eq!(event.bids.len(), 2);
        assert_eq!(event.asks.len(), 2);
        assert_eq!(event.bids[0].0, "34000.00");
        assert_eq!(event.asks[0].1, "1.0");
    }

    #[test]
    fn test_futures_order_deserialization() {
        // Note: FuturesOrder uses camelCase (matches Binance API response)
        let json = r#"{
            "symbol": "BTCUSDT",
            "orderId": 12345,
            "orderListId": -1,
            "clientOrderId": "test123",
            "price": "34000.00",
            "origQty": "0.01",
            "executedQty": "0.005",
            "cumQuoteQty": "170.00",
            "status": "PARTIALLY_FILLED",
            "timeInForce": "GTC",
            "type": "LIMIT",
            "side": "BUY",
            "stopPrice": "0.00",
            "icebergQty": "0.00",
            "time": 1625097600000,
            "updateTime": 1625097610000,
            "isWorking": true,
            "origQuoteOrderQty": "340.00"
        }"#;

        let order: FuturesOrder = serde_json::from_str(json).unwrap();
        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.order_id, 12345);
        assert_eq!(order.status, "PARTIALLY_FILLED");
        assert_eq!(order.side, "BUY");
        assert!(order.is_working);
    }

    #[test]
    fn test_futures_position_deserialization() {
        // Note: FuturesPosition uses camelCase (matches Binance API response)
        let json = r#"{
            "symbol": "BTCUSDT",
            "positionAmt": "0.01",
            "entryPrice": "34000.00",
            "markPrice": "34100.00",
            "unRealizedProfit": "1.00",
            "liquidationPrice": "30000.00",
            "leverage": "10",
            "maxNotionalValue": "100000.00",
            "marginType": "isolated",
            "isolatedMargin": "340.00",
            "isAutoAddMargin": "false",
            "positionSide": "LONG",
            "notional": "341.00",
            "isolatedWallet": "340.00",
            "updateTime": 1625097600000
        }"#;

        let position: FuturesPosition = serde_json::from_str(json).unwrap();
        assert_eq!(position.symbol, "BTCUSDT");
        assert_eq!(position.position_amt, "0.01");
        assert_eq!(position.leverage, "10");
        assert_eq!(position.margin_type, "isolated");
        assert!(!position.is_auto_add_margin);
    }

    #[test]
    fn test_account_info_deserialization() {
        // Note: AccountInfo uses camelCase (matches Binance API response)
        let json = r#"{
            "makerCommission": 10,
            "takerCommission": 10,
            "buyerCommission": 0,
            "sellerCommission": 0,
            "canTrade": true,
            "canWithdraw": true,
            "canDeposit": true,
            "updateTime": 1625097600000,
            "accountType": "SPOT",
            "balances": [
                {"asset": "BTC", "free": "1.0", "locked": "0.0"},
                {"asset": "USDT", "free": "10000.0", "locked": "500.0"}
            ],
            "permissions": ["SPOT", "MARGIN"]
        }"#;

        let account: AccountInfo = serde_json::from_str(json).unwrap();
        assert!(account.can_trade);
        assert_eq!(account.balances.len(), 2);
        assert_eq!(account.balances[0].asset, "BTC");
        assert_eq!(account.permissions.len(), 2);
    }

    #[test]
    fn test_balance_clone() {
        let balance = Balance {
            asset: "BTC".to_string(),
            free: "1.0".to_string(),
            locked: "0.0".to_string(),
        };

        let cloned = balance.clone();
        assert_eq!(balance.asset, cloned.asset);
        assert_eq!(balance.free, cloned.free);
    }

    #[test]
    fn test_new_order_request_serialization() {
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            r#type: "LIMIT".to_string(),
            quantity: Some("0.01".to_string()),
            quote_order_qty: None,
            price: Some("34000.00".to_string()),
            new_client_order_id: Some("test123".to_string()),
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            time_in_force: Some("GTC".to_string()),
            reduce_only: None,
            close_position: None,
            position_side: Some("LONG".to_string()),
            working_type: None,
            price_protect: None,
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("BUY"));
        assert!(json.contains("LIMIT"));
    }

    #[test]
    fn test_order_response_with_fills() {
        // Note: OrderResponse uses snake_case, but nested Fill uses camelCase
        let json = r#"{
            "symbol": "BTCUSDT",
            "order_id": 12345,
            "order_list_id": -1,
            "client_order_id": "test123",
            "transact_time": 1625097600000,
            "price": "34000.00",
            "orig_qty": "0.01",
            "executed_qty": "0.01",
            "cumulative_quote_qty": "340.00",
            "status": "FILLED",
            "time_in_force": "GTC",
            "type": "LIMIT",
            "side": "BUY",
            "fills": [
                {
                    "price": "34000.00",
                    "qty": "0.01",
                    "commission": "0.00001",
                    "commissionAsset": "BTC",
                    "tradeId": 1001
                }
            ]
        }"#;

        let response: OrderResponse = serde_json::from_str(json).unwrap();
        assert_eq!(response.symbol, "BTCUSDT");
        assert_eq!(response.status, "FILLED");
        assert_eq!(response.fills.len(), 1);
        assert_eq!(response.fills[0].trade_id, 1001);
    }

    #[test]
    fn test_websocket_message_deserialization() {
        let json = r#"{
            "stream": "btcusdt@kline_1m",
            "data": {"test": "value"}
        }"#;

        let msg: WebSocketMessage = serde_json::from_str(json).unwrap();
        assert_eq!(msg.stream, "btcusdt@kline_1m");
        assert!(msg.data.is_object());
    }

    #[test]
    fn test_chart_update_event_serialization() {
        let event = ChartUpdateEvent {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            candle: ChartCandle {
                timestamp: 1625097600000,
                open: 34000.0,
                high: 35000.0,
                low: 33000.0,
                close: 34500.0,
                volume: 100.5,
                is_closed: true,
            },
            latest_price: 34500.0,
            price_change_24h: 500.0,
            price_change_percent_24h: 1.5,
            volume_24h: 1000.5,
            timestamp: 1625097600000,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("BTCUSDT"));
        assert!(json.contains("34500"));
    }

    #[test]
    fn test_market_data_update_deserialization() {
        let json = r#"{
            "symbol": "BTCUSDT",
            "price": 34500.0,
            "price_change_24h": 500.0,
            "price_change_percent_24h": 1.5,
            "volume_24h": 1000.5,
            "timestamp": 1625097600000
        }"#;

        let update: MarketDataUpdate = serde_json::from_str(json).unwrap();
        assert_eq!(update.symbol, "BTCUSDT");
        assert_eq!(update.price, 34500.0);
        assert_eq!(update.volume_24h, 1000.5);
    }

    #[test]
    fn test_websocket_event_chart_update() {
        let json = r#"{
            "type": "chart_update",
            "symbol": "BTCUSDT",
            "timeframe": "1m",
            "candle": {
                "timestamp": 1625097600000,
                "open": 34000.0,
                "high": 35000.0,
                "low": 33000.0,
                "close": 34500.0,
                "volume": 100.5,
                "is_closed": true
            },
            "latest_price": 34500.0,
            "price_change_24h": 500.0,
            "price_change_percent_24h": 1.5,
            "volume_24h": 1000.5,
            "timestamp": 1625097600000
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::ChartUpdate(update) => {
                assert_eq!(update.symbol, "BTCUSDT");
                assert!(update.candle.is_closed);
            },
            _ => panic!("Expected ChartUpdate variant"),
        }
    }

    #[test]
    fn test_websocket_event_error() {
        let json = r#"{
            "type": "error",
            "message": "Connection failed"
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::Error { message } => {
                assert_eq!(message, "Connection failed");
            },
            _ => panic!("Expected Error variant"),
        }
    }

    #[test]
    fn test_chart_candle_clone() {
        let candle = ChartCandle {
            timestamp: 1625097600000,
            open: 34000.0,
            high: 35000.0,
            low: 33000.0,
            close: 34500.0,
            volume: 100.5,
            is_closed: true,
        };

        let cloned = candle.clone();
        assert_eq!(candle.timestamp, cloned.timestamp);
        assert_eq!(candle.close, cloned.close);
        assert_eq!(candle.is_closed, cloned.is_closed);
    }

    // Additional comprehensive tests for edge cases and full coverage

    #[test]
    fn test_kline_with_empty_strings() {
        let kline = Kline {
            open_time: 0,
            open: "".to_string(),
            high: "".to_string(),
            low: "".to_string(),
            close: "".to_string(),
            volume: "".to_string(),
            close_time: 0,
            quote_asset_volume: "".to_string(),
            number_of_trades: 0,
            taker_buy_base_asset_volume: "".to_string(),
            taker_buy_quote_asset_volume: "".to_string(),
            ignore: "".to_string(),
        };

        let json = serde_json::to_string(&kline).unwrap();
        let deserialized: Kline = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.open, "");
        assert_eq!(deserialized.number_of_trades, 0);
    }

    #[test]
    fn test_kline_with_extreme_values() {
        let kline = Kline {
            open_time: i64::MAX,
            open: "999999999.99999999".to_string(),
            high: "999999999.99999999".to_string(),
            low: "0.00000001".to_string(),
            close: "500000000.00000000".to_string(),
            volume: "99999999999.99999999".to_string(),
            close_time: i64::MAX,
            quote_asset_volume: "99999999999999.99".to_string(),
            number_of_trades: i64::MAX,
            taker_buy_base_asset_volume: "50000000000.00000000".to_string(),
            taker_buy_quote_asset_volume: "50000000000000.00".to_string(),
            ignore: "999".to_string(),
        };

        let json = serde_json::to_string(&kline).unwrap();
        let deserialized: Kline = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.open_time, i64::MAX);
        assert_eq!(deserialized.number_of_trades, i64::MAX);
    }

    #[test]
    fn test_kline_roundtrip_serialization() {
        let kline = Kline {
            open_time: 1625097600000,
            open: "34000.00".to_string(),
            high: "35000.00".to_string(),
            low: "33000.00".to_string(),
            close: "34500.00".to_string(),
            volume: "100.5".to_string(),
            close_time: 1625097659999,
            quote_asset_volume: "3450000.00".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
            ignore: "0".to_string(),
        };

        let json = serde_json::to_string(&kline).unwrap();
        let deserialized: Kline = serde_json::from_str(&json).unwrap();

        assert_eq!(kline.open_time, deserialized.open_time);
        assert_eq!(kline.open, deserialized.open);
        assert_eq!(kline.high, deserialized.high);
        assert_eq!(kline.low, deserialized.low);
        assert_eq!(kline.close, deserialized.close);
        assert_eq!(kline.volume, deserialized.volume);
        assert_eq!(kline.close_time, deserialized.close_time);
    }

    #[test]
    fn test_kline_to_decimal_with_zero_values() {
        let kline = Kline {
            open_time: 0,
            open: "0".to_string(),
            high: "0".to_string(),
            low: "0".to_string(),
            close: "0".to_string(),
            volume: "0".to_string(),
            close_time: 0,
            quote_asset_volume: "0".to_string(),
            number_of_trades: 0,
            taker_buy_base_asset_volume: "0".to_string(),
            taker_buy_quote_asset_volume: "0".to_string(),
            ignore: "0".to_string(),
        };

        let result = kline.to_decimal_values().unwrap();
        assert_eq!(result.0, Decimal::ZERO);
        assert_eq!(result.1, Decimal::ZERO);
        assert_eq!(result.2, Decimal::ZERO);
        assert_eq!(result.3, Decimal::ZERO);
        assert_eq!(result.4, Decimal::ZERO);
    }

    #[test]
    fn test_kline_clone_independence() {
        let original = Kline {
            open_time: 1625097600000,
            open: "34000.00".to_string(),
            high: "35000.00".to_string(),
            low: "33000.00".to_string(),
            close: "34500.00".to_string(),
            volume: "100.5".to_string(),
            close_time: 1625097659999,
            quote_asset_volume: "3450000.00".to_string(),
            number_of_trades: 1000,
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
            ignore: "0".to_string(),
        };

        let cloned = original.clone();
        assert_eq!(original.open_time, cloned.open_time);
        assert_eq!(original.open, cloned.open);
    }

    #[test]
    fn test_symbol_price_roundtrip() {
        let price = SymbolPrice {
            symbol: "ETHUSDT".to_string(),
            price: "2500.75".to_string(),
        };

        let json = serde_json::to_string(&price).unwrap();
        let deserialized: SymbolPrice = serde_json::from_str(&json).unwrap();

        assert_eq!(price.symbol, deserialized.symbol);
        assert_eq!(price.price, deserialized.price);
    }

    #[test]
    fn test_symbol_price_with_zero_price() {
        let price = SymbolPrice {
            symbol: "BTCUSDT".to_string(),
            price: "0".to_string(),
        };

        let json = serde_json::to_string(&price).unwrap();
        assert!(json.contains("\"price\":\"0\""));
    }

    #[test]
    fn test_symbol_price_clone() {
        let price = SymbolPrice {
            symbol: "BTCUSDT".to_string(),
            price: "50000.00".to_string(),
        };

        let cloned = price.clone();
        assert_eq!(price.symbol, cloned.symbol);
        assert_eq!(price.price, cloned.price);
    }

    #[test]
    fn test_funding_rate_serialization() {
        let funding = FundingRate {
            symbol: "ETHUSDT".to_string(),
            funding_rate: "0.0005".to_string(),
            funding_time: 1625097600000,
        };

        let json = serde_json::to_string(&funding).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("0.0005"));
    }

    #[test]
    fn test_funding_rate_roundtrip() {
        let funding = FundingRate {
            symbol: "BTCUSDT".to_string(),
            funding_rate: "-0.0001".to_string(),
            funding_time: 1625097600000,
        };

        let json = serde_json::to_string(&funding).unwrap();
        let deserialized: FundingRate = serde_json::from_str(&json).unwrap();

        assert_eq!(funding.symbol, deserialized.symbol);
        assert_eq!(funding.funding_rate, deserialized.funding_rate);
        assert_eq!(funding.funding_time, deserialized.funding_time);
    }

    #[test]
    fn test_funding_rate_clone() {
        let funding = FundingRate {
            symbol: "BTCUSDT".to_string(),
            funding_rate: "0.0001".to_string(),
            funding_time: 1625097600000,
        };

        let cloned = funding.clone();
        assert_eq!(funding.symbol, cloned.symbol);
        assert_eq!(funding.funding_rate, cloned.funding_rate);
    }

    #[test]
    fn test_kline_event_serialization() {
        let event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            kline: KlineData {
                kline_start_time: 1625097600000,
                kline_close_time: 1625097659999,
                symbol: "BTCUSDT".to_string(),
                interval: "1m".to_string(),
                first_trade_id: 100,
                last_trade_id: 200,
                open_price: "34000.00".to_string(),
                close_price: "34500.00".to_string(),
                high_price: "35000.00".to_string(),
                low_price: "33000.00".to_string(),
                base_asset_volume: "100.5".to_string(),
                number_of_trades: 1000,
                is_this_kline_closed: false,
                quote_asset_volume: "3450000.00".to_string(),
                taker_buy_base_asset_volume: "50.25".to_string(),
                taker_buy_quote_asset_volume: "1725000.00".to_string(),
            },
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"e\":\"kline\""));
        assert!(json.contains("\"s\":\"BTCUSDT\""));
    }

    #[test]
    fn test_kline_event_clone() {
        let event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            kline: KlineData {
                kline_start_time: 1625097600000,
                kline_close_time: 1625097659999,
                symbol: "BTCUSDT".to_string(),
                interval: "1m".to_string(),
                first_trade_id: 100,
                last_trade_id: 200,
                open_price: "34000.00".to_string(),
                close_price: "34500.00".to_string(),
                high_price: "35000.00".to_string(),
                low_price: "33000.00".to_string(),
                base_asset_volume: "100.5".to_string(),
                number_of_trades: 1000,
                is_this_kline_closed: true,
                quote_asset_volume: "3450000.00".to_string(),
                taker_buy_base_asset_volume: "50.25".to_string(),
                taker_buy_quote_asset_volume: "1725000.00".to_string(),
            },
        };

        let cloned = event.clone();
        assert_eq!(event.event_type, cloned.event_type);
        assert_eq!(event.symbol, cloned.symbol);
    }

    #[test]
    fn test_kline_data_serialization() {
        let kline_data = KlineData {
            kline_start_time: 1625097600000,
            kline_close_time: 1625097659999,
            symbol: "BTCUSDT".to_string(),
            interval: "5m".to_string(),
            first_trade_id: 500,
            last_trade_id: 600,
            open_price: "40000.00".to_string(),
            close_price: "40500.00".to_string(),
            high_price: "41000.00".to_string(),
            low_price: "39500.00".to_string(),
            base_asset_volume: "200.5".to_string(),
            number_of_trades: 2000,
            is_this_kline_closed: true,
            quote_asset_volume: "8000000.00".to_string(),
            taker_buy_base_asset_volume: "100.25".to_string(),
            taker_buy_quote_asset_volume: "4000000.00".to_string(),
        };

        let json = serde_json::to_string(&kline_data).unwrap();
        assert!(json.contains("\"i\":\"5m\""));
        assert!(json.contains("\"x\":true"));
    }

    #[test]
    fn test_kline_data_clone() {
        let kline_data = KlineData {
            kline_start_time: 1625097600000,
            kline_close_time: 1625097659999,
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            first_trade_id: 100,
            last_trade_id: 200,
            open_price: "34000.00".to_string(),
            close_price: "34500.00".to_string(),
            high_price: "35000.00".to_string(),
            low_price: "33000.00".to_string(),
            base_asset_volume: "100.5".to_string(),
            number_of_trades: 1000,
            is_this_kline_closed: true,
            quote_asset_volume: "3450000.00".to_string(),
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
        };

        let cloned = kline_data.clone();
        assert_eq!(kline_data.symbol, cloned.symbol);
        assert_eq!(kline_data.interval, cloned.interval);
    }

    #[test]
    fn test_kline_data_to_decimal_invalid_high() {
        let kline_data = KlineData {
            kline_start_time: 1625097600000,
            kline_close_time: 1625097659999,
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            first_trade_id: 100,
            last_trade_id: 200,
            open_price: "34000.00".to_string(),
            close_price: "34500.00".to_string(),
            high_price: "invalid".to_string(),
            low_price: "33000.00".to_string(),
            base_asset_volume: "100.5".to_string(),
            number_of_trades: 1000,
            is_this_kline_closed: true,
            quote_asset_volume: "3450000.00".to_string(),
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
        };

        assert!(kline_data.to_decimal_values().is_err());
    }

    #[test]
    fn test_ticker_event_serialization() {
        let event = TickerEvent {
            event_type: "24hrTicker".to_string(),
            event_time: 1625097600000,
            symbol: "ETHUSDT".to_string(),
            price_change: "100.00".to_string(),
            price_change_percent: "4.0".to_string(),
            weighted_avg_price: "2550.00".to_string(),
            prev_close_price: "2500.00".to_string(),
            last_price: "2600.00".to_string(),
            last_quantity: "5.0".to_string(),
            best_bid_price: "2599.00".to_string(),
            best_bid_quantity: "10.0".to_string(),
            best_ask_price: "2601.00".to_string(),
            best_ask_quantity: "8.0".to_string(),
            open_price: "2500.00".to_string(),
            high_price: "2650.00".to_string(),
            low_price: "2480.00".to_string(),
            total_traded_base_asset_volume: "5000.0".to_string(),
            total_traded_quote_asset_volume: "12750000.00".to_string(),
            statistics_open_time: 1625011200000,
            statistics_close_time: 1625097600000,
            first_trade_id: 2000,
            last_trade_id: 10000,
            total_number_of_trades: 8000,
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"e\":\"24hrTicker\""));
        assert!(json.contains("\"s\":\"ETHUSDT\""));
    }

    #[test]
    fn test_ticker_event_clone() {
        let event = TickerEvent {
            event_type: "24hrTicker".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            price_change: "500.00".to_string(),
            price_change_percent: "1.5".to_string(),
            weighted_avg_price: "34250.00".to_string(),
            prev_close_price: "33500.00".to_string(),
            last_price: "34000.00".to_string(),
            last_quantity: "10.5".to_string(),
            best_bid_price: "33990.00".to_string(),
            best_bid_quantity: "5.0".to_string(),
            best_ask_price: "34010.00".to_string(),
            best_ask_quantity: "4.5".to_string(),
            open_price: "33500.00".to_string(),
            high_price: "35000.00".to_string(),
            low_price: "33000.00".to_string(),
            total_traded_base_asset_volume: "1000.5".to_string(),
            total_traded_quote_asset_volume: "34000000.00".to_string(),
            statistics_open_time: 1625011200000,
            statistics_close_time: 1625097600000,
            first_trade_id: 1000,
            last_trade_id: 5000,
            total_number_of_trades: 4000,
        };

        let cloned = event.clone();
        assert_eq!(event.event_type, cloned.event_type);
        assert_eq!(event.symbol, cloned.symbol);
    }

    #[test]
    fn test_order_book_event_serialization() {
        let event = OrderBookEvent {
            event_type: "depthUpdate".to_string(),
            event_time: 1625097600000,
            symbol: "ETHUSDT".to_string(),
            first_update_id: 2000,
            final_update_id: 2010,
            bids: vec![
                ("2500.00".to_string(), "10.0".to_string()),
                ("2499.00".to_string(), "15.0".to_string()),
            ],
            asks: vec![
                ("2501.00".to_string(), "5.0".to_string()),
                ("2502.00".to_string(), "8.0".to_string()),
            ],
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("\"e\":\"depthUpdate\""));
        assert!(json.contains("\"s\":\"ETHUSDT\""));
    }

    #[test]
    fn test_order_book_event_clone() {
        let event = OrderBookEvent {
            event_type: "depthUpdate".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            first_update_id: 1000,
            final_update_id: 1005,
            bids: vec![("34000.00".to_string(), "1.5".to_string())],
            asks: vec![("34001.00".to_string(), "1.0".to_string())],
        };

        let cloned = event.clone();
        assert_eq!(event.event_type, cloned.event_type);
        assert_eq!(event.bids.len(), cloned.bids.len());
    }

    #[test]
    fn test_order_book_event_empty_bids_asks() {
        let event = OrderBookEvent {
            event_type: "depthUpdate".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            first_update_id: 1000,
            final_update_id: 1000,
            bids: vec![],
            asks: vec![],
        };

        let json = serde_json::to_string(&event).unwrap();
        let deserialized: OrderBookEvent = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.bids.len(), 0);
        assert_eq!(deserialized.asks.len(), 0);
    }

    #[test]
    fn test_futures_order_serialization() {
        let order = FuturesOrder {
            symbol: "ETHUSDT".to_string(),
            order_id: 67890,
            order_list_id: -1,
            client_order_id: "test456".to_string(),
            price: "2500.00".to_string(),
            orig_qty: "1.0".to_string(),
            executed_qty: "0.5".to_string(),
            cumulative_quote_qty: "1250.00".to_string(),
            status: "PARTIALLY_FILLED".to_string(),
            time_in_force: "GTC".to_string(),
            r#type: "LIMIT".to_string(),
            side: "SELL".to_string(),
            stop_price: "0.00".to_string(),
            iceberg_qty: "0.00".to_string(),
            time: 1625097600000,
            update_time: 1625097620000,
            is_working: true,
            orig_quote_order_qty: "2500.00".to_string(),
        };

        let json = serde_json::to_string(&order).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("SELL"));
    }

    #[test]
    fn test_futures_order_clone() {
        let order = FuturesOrder {
            symbol: "BTCUSDT".to_string(),
            order_id: 12345,
            order_list_id: -1,
            client_order_id: "test123".to_string(),
            price: "34000.00".to_string(),
            orig_qty: "0.01".to_string(),
            executed_qty: "0.005".to_string(),
            cumulative_quote_qty: "170.00".to_string(),
            status: "PARTIALLY_FILLED".to_string(),
            time_in_force: "GTC".to_string(),
            r#type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            stop_price: "0.00".to_string(),
            iceberg_qty: "0.00".to_string(),
            time: 1625097600000,
            update_time: 1625097610000,
            is_working: true,
            orig_quote_order_qty: "340.00".to_string(),
        };

        let cloned = order.clone();
        assert_eq!(order.symbol, cloned.symbol);
        assert_eq!(order.order_id, cloned.order_id);
    }

    #[test]
    fn test_futures_position_serialization() {
        let position = FuturesPosition {
            symbol: "ETHUSDT".to_string(),
            position_amt: "2.0".to_string(),
            entry_price: "2500.00".to_string(),
            mark_price: "2600.00".to_string(),
            unrealized_pnl: "200.00".to_string(),
            liquidation_price: "2000.00".to_string(),
            leverage: "5".to_string(),
            max_notional_value: "50000.00".to_string(),
            margin_type: "cross".to_string(),
            isolated_margin: "0.00".to_string(),
            is_auto_add_margin: true,
            position_side: "SHORT".to_string(),
            notional: "5200.00".to_string(),
            isolated_wallet: "0.00".to_string(),
            update_time: 1625097600000,
        };

        let json = serde_json::to_string(&position).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("cross"));
    }

    #[test]
    fn test_futures_position_clone() {
        let position = FuturesPosition {
            symbol: "BTCUSDT".to_string(),
            position_amt: "0.01".to_string(),
            entry_price: "34000.00".to_string(),
            mark_price: "34100.00".to_string(),
            unrealized_pnl: "1.00".to_string(),
            liquidation_price: "30000.00".to_string(),
            leverage: "10".to_string(),
            max_notional_value: "100000.00".to_string(),
            margin_type: "isolated".to_string(),
            isolated_margin: "340.00".to_string(),
            is_auto_add_margin: false,
            position_side: "LONG".to_string(),
            notional: "341.00".to_string(),
            isolated_wallet: "340.00".to_string(),
            update_time: 1625097600000,
        };

        let cloned = position.clone();
        assert_eq!(position.symbol, cloned.symbol);
        assert_eq!(position.leverage, cloned.leverage);
    }

    #[test]
    fn test_account_info_serialization() {
        let account = AccountInfo {
            maker_commission: 10,
            taker_commission: 10,
            buyer_commission: 0,
            seller_commission: 0,
            can_trade: true,
            can_withdraw: false,
            can_deposit: true,
            update_time: 1625097600000,
            account_type: "MARGIN".to_string(),
            balances: vec![Balance {
                asset: "ETH".to_string(),
                free: "5.0".to_string(),
                locked: "1.0".to_string(),
            }],
            permissions: vec!["SPOT".to_string(), "MARGIN".to_string()],
        };

        let json = serde_json::to_string(&account).unwrap();
        assert!(json.contains("MARGIN"));
        assert!(json.contains("ETH"));
    }

    #[test]
    fn test_account_info_clone() {
        let account = AccountInfo {
            maker_commission: 10,
            taker_commission: 10,
            buyer_commission: 0,
            seller_commission: 0,
            can_trade: true,
            can_withdraw: true,
            can_deposit: true,
            update_time: 1625097600000,
            account_type: "SPOT".to_string(),
            balances: vec![],
            permissions: vec![],
        };

        let cloned = account.clone();
        assert_eq!(account.maker_commission, cloned.maker_commission);
        assert_eq!(account.can_trade, cloned.can_trade);
    }

    #[test]
    fn test_balance_serialization() {
        let balance = Balance {
            asset: "USDT".to_string(),
            free: "5000.00".to_string(),
            locked: "250.00".to_string(),
        };

        let json = serde_json::to_string(&balance).unwrap();
        assert!(json.contains("USDT"));
        assert!(json.contains("5000.00"));
    }

    #[test]
    fn test_balance_roundtrip() {
        let balance = Balance {
            asset: "BNB".to_string(),
            free: "100.5".to_string(),
            locked: "10.25".to_string(),
        };

        let json = serde_json::to_string(&balance).unwrap();
        let deserialized: Balance = serde_json::from_str(&json).unwrap();

        assert_eq!(balance.asset, deserialized.asset);
        assert_eq!(balance.free, deserialized.free);
        assert_eq!(balance.locked, deserialized.locked);
    }

    #[test]
    fn test_new_order_request_deserialization() {
        let json = r#"{
            "symbol": "ETHUSDT",
            "side": "SELL",
            "type": "MARKET",
            "quantity": "1.0",
            "quote_order_qty": null,
            "price": null,
            "new_client_order_id": "order789",
            "stop_price": null,
            "iceberg_qty": null,
            "new_order_resp_type": "FULL",
            "time_in_force": "IOC",
            "reduce_only": true,
            "close_position": false,
            "position_side": "SHORT",
            "working_type": "MARK_PRICE",
            "price_protect": true
        }"#;

        let order: NewOrderRequest = serde_json::from_str(json).unwrap();
        assert_eq!(order.symbol, "ETHUSDT");
        assert_eq!(order.side, "SELL");
        assert_eq!(order.r#type, "MARKET");
        assert_eq!(order.reduce_only, Some(true));
        assert_eq!(order.price_protect, Some(true));
    }

    #[test]
    fn test_new_order_request_clone() {
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            r#type: "LIMIT".to_string(),
            quantity: Some("0.01".to_string()),
            quote_order_qty: None,
            price: Some("34000.00".to_string()),
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            time_in_force: Some("GTC".to_string()),
            reduce_only: None,
            close_position: None,
            position_side: None,
            working_type: None,
            price_protect: None,
        };

        let cloned = order.clone();
        assert_eq!(order.symbol, cloned.symbol);
        assert_eq!(order.side, cloned.side);
    }

    #[test]
    fn test_order_response_serialization() {
        let response = OrderResponse {
            symbol: "ETHUSDT".to_string(),
            order_id: 54321,
            order_list_id: -1,
            client_order_id: "order999".to_string(),
            transact_time: 1625097600000,
            price: "2500.00".to_string(),
            orig_qty: "1.0".to_string(),
            executed_qty: "1.0".to_string(),
            cumulative_quote_qty: "2500.00".to_string(),
            status: "FILLED".to_string(),
            time_in_force: "GTC".to_string(),
            r#type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            fills: vec![],
        };

        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("FILLED"));
    }

    #[test]
    fn test_order_response_clone() {
        let response = OrderResponse {
            symbol: "BTCUSDT".to_string(),
            order_id: 12345,
            order_list_id: -1,
            client_order_id: "test123".to_string(),
            transact_time: 1625097600000,
            price: "34000.00".to_string(),
            orig_qty: "0.01".to_string(),
            executed_qty: "0.01".to_string(),
            cumulative_quote_qty: "340.00".to_string(),
            status: "FILLED".to_string(),
            time_in_force: "GTC".to_string(),
            r#type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            fills: vec![],
        };

        let cloned = response.clone();
        assert_eq!(response.symbol, cloned.symbol);
        assert_eq!(response.order_id, cloned.order_id);
    }

    #[test]
    fn test_fill_serialization() {
        let fill = Fill {
            price: "34000.00".to_string(),
            qty: "0.01".to_string(),
            commission: "0.00002".to_string(),
            commission_asset: "BNB".to_string(),
            trade_id: 9999,
        };

        let json = serde_json::to_string(&fill).unwrap();
        assert!(json.contains("34000.00"));
        assert!(json.contains("BNB"));
    }

    #[test]
    fn test_fill_deserialization() {
        // Note: Fill struct uses #[serde(rename_all = "camelCase")]
        let json = r#"{
            "price": "2500.00",
            "qty": "1.0",
            "commission": "0.001",
            "commissionAsset": "ETH",
            "tradeId": 8888
        }"#;

        let fill: Fill = serde_json::from_str(json).unwrap();
        assert_eq!(fill.price, "2500.00");
        assert_eq!(fill.qty, "1.0");
        assert_eq!(fill.trade_id, 8888);
    }

    #[test]
    fn test_fill_clone() {
        let fill = Fill {
            price: "34000.00".to_string(),
            qty: "0.01".to_string(),
            commission: "0.00001".to_string(),
            commission_asset: "BTC".to_string(),
            trade_id: 1001,
        };

        let cloned = fill.clone();
        assert_eq!(fill.price, cloned.price);
        assert_eq!(fill.trade_id, cloned.trade_id);
    }

    #[test]
    fn test_websocket_message_serialization() {
        let msg = WebSocketMessage {
            stream: "ethusdt@ticker".to_string(),
            data: serde_json::json!({"price": "2500.00"}),
        };

        let json = serde_json::to_string(&msg).unwrap();
        assert!(json.contains("ethusdt@ticker"));
    }

    #[test]
    fn test_websocket_message_clone() {
        let msg = WebSocketMessage {
            stream: "btcusdt@kline_1m".to_string(),
            data: serde_json::json!({"test": "value"}),
        };

        let cloned = msg.clone();
        assert_eq!(msg.stream, cloned.stream);
    }

    #[test]
    fn test_stream_event_kline_variant_clone() {
        let event = KlineEvent {
            event_type: "kline".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            kline: KlineData {
                kline_start_time: 1625097600000,
                kline_close_time: 1625097659999,
                symbol: "BTCUSDT".to_string(),
                interval: "1m".to_string(),
                first_trade_id: 100,
                last_trade_id: 200,
                open_price: "34000.00".to_string(),
                close_price: "34500.00".to_string(),
                high_price: "35000.00".to_string(),
                low_price: "33000.00".to_string(),
                base_asset_volume: "100.5".to_string(),
                number_of_trades: 1000,
                is_this_kline_closed: true,
                quote_asset_volume: "3450000.00".to_string(),
                taker_buy_base_asset_volume: "50.25".to_string(),
                taker_buy_quote_asset_volume: "1725000.00".to_string(),
            },
        };

        let cloned_event = event.clone();
        assert_eq!(event.event_type, cloned_event.event_type);
        assert_eq!(event.symbol, cloned_event.symbol);
        assert_eq!(event.kline.interval, cloned_event.kline.interval);
    }

    #[test]
    fn test_stream_event_ticker_variant_clone() {
        let event = TickerEvent {
            event_type: "24hrTicker".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            price_change: "500.00".to_string(),
            price_change_percent: "1.5".to_string(),
            weighted_avg_price: "34250.00".to_string(),
            prev_close_price: "33500.00".to_string(),
            last_price: "34000.00".to_string(),
            last_quantity: "10.5".to_string(),
            best_bid_price: "33990.00".to_string(),
            best_bid_quantity: "5.0".to_string(),
            best_ask_price: "34010.00".to_string(),
            best_ask_quantity: "4.5".to_string(),
            open_price: "33500.00".to_string(),
            high_price: "35000.00".to_string(),
            low_price: "33000.00".to_string(),
            total_traded_base_asset_volume: "1000.5".to_string(),
            total_traded_quote_asset_volume: "34000000.00".to_string(),
            statistics_open_time: 1625011200000,
            statistics_close_time: 1625097600000,
            first_trade_id: 1000,
            last_trade_id: 5000,
            total_number_of_trades: 4000,
        };

        let cloned_event = event.clone();
        assert_eq!(event.event_type, cloned_event.event_type);
        assert_eq!(event.symbol, cloned_event.symbol);
        assert_eq!(event.last_price, cloned_event.last_price);
    }

    #[test]
    fn test_stream_event_orderbook_variant_clone() {
        let event = OrderBookEvent {
            event_type: "depthUpdate".to_string(),
            event_time: 1625097600000,
            symbol: "BTCUSDT".to_string(),
            first_update_id: 1000,
            final_update_id: 1005,
            bids: vec![("34000.00".to_string(), "1.5".to_string())],
            asks: vec![("34001.00".to_string(), "1.0".to_string())],
        };

        let cloned_event = event.clone();
        assert_eq!(event.event_type, cloned_event.event_type);
        assert_eq!(event.symbol, cloned_event.symbol);
        assert_eq!(event.bids.len(), cloned_event.bids.len());
    }

    #[test]
    fn test_chart_update_event_deserialization() {
        let json = r#"{
            "symbol": "ETHUSDT",
            "timeframe": "5m",
            "candle": {
                "timestamp": 1625097600000,
                "open": 2500.0,
                "high": 2600.0,
                "low": 2480.0,
                "close": 2550.0,
                "volume": 500.5,
                "is_closed": false
            },
            "latest_price": 2550.0,
            "price_change_24h": 100.0,
            "price_change_percent_24h": 4.0,
            "volume_24h": 5000.0,
            "timestamp": 1625097600000
        }"#;

        let event: ChartUpdateEvent = serde_json::from_str(json).unwrap();
        assert_eq!(event.symbol, "ETHUSDT");
        assert_eq!(event.timeframe, "5m");
        assert!(!event.candle.is_closed);
    }

    #[test]
    fn test_chart_update_event_clone() {
        let event = ChartUpdateEvent {
            symbol: "BTCUSDT".to_string(),
            timeframe: "1m".to_string(),
            candle: ChartCandle {
                timestamp: 1625097600000,
                open: 34000.0,
                high: 35000.0,
                low: 33000.0,
                close: 34500.0,
                volume: 100.5,
                is_closed: true,
            },
            latest_price: 34500.0,
            price_change_24h: 500.0,
            price_change_percent_24h: 1.5,
            volume_24h: 1000.5,
            timestamp: 1625097600000,
        };

        let cloned = event.clone();
        assert_eq!(event.symbol, cloned.symbol);
        assert_eq!(event.timeframe, cloned.timeframe);
    }

    #[test]
    fn test_chart_candle_serialization() {
        let candle = ChartCandle {
            timestamp: 1625097600000,
            open: 50000.0,
            high: 51000.0,
            low: 49500.0,
            close: 50500.0,
            volume: 250.75,
            is_closed: false,
        };

        let json = serde_json::to_string(&candle).unwrap();
        assert!(json.contains("50000"));
        assert!(json.contains("false"));
    }

    #[test]
    fn test_chart_candle_deserialization() {
        let json = r#"{
            "timestamp": 1625097600000,
            "open": 40000.0,
            "high": 41000.0,
            "low": 39500.0,
            "close": 40500.0,
            "volume": 150.25,
            "is_closed": true
        }"#;

        let candle: ChartCandle = serde_json::from_str(json).unwrap();
        assert_eq!(candle.timestamp, 1625097600000);
        assert_eq!(candle.open, 40000.0);
        assert!(candle.is_closed);
    }

    #[test]
    fn test_chart_candle_with_zero_volume() {
        let candle = ChartCandle {
            timestamp: 1625097600000,
            open: 34000.0,
            high: 34000.0,
            low: 34000.0,
            close: 34000.0,
            volume: 0.0,
            is_closed: true,
        };

        let json = serde_json::to_string(&candle).unwrap();
        let deserialized: ChartCandle = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.volume, 0.0);
    }

    #[test]
    fn test_market_data_update_serialization() {
        let update = MarketDataUpdate {
            symbol: "ETHUSDT".to_string(),
            price: 2600.0,
            price_change_24h: 150.0,
            price_change_percent_24h: 6.12,
            volume_24h: 8000.25,
            timestamp: 1625097600000,
        };

        let json = serde_json::to_string(&update).unwrap();
        assert!(json.contains("ETHUSDT"));
        assert!(json.contains("2600"));
    }

    #[test]
    fn test_market_data_update_clone() {
        let update = MarketDataUpdate {
            symbol: "BTCUSDT".to_string(),
            price: 34500.0,
            price_change_24h: 500.0,
            price_change_percent_24h: 1.5,
            volume_24h: 1000.5,
            timestamp: 1625097600000,
        };

        let cloned = update.clone();
        assert_eq!(update.symbol, cloned.symbol);
        assert_eq!(update.price, cloned.price);
    }

    #[test]
    fn test_websocket_event_kline_variant() {
        let json = r#"{
            "type": "kline",
            "e": "kline",
            "E": 1625097600000,
            "s": "BTCUSDT",
            "k": {
                "t": 1625097600000,
                "T": 1625097659999,
                "s": "BTCUSDT",
                "i": "1m",
                "f": 100,
                "L": 200,
                "o": "34000.00",
                "c": "34500.00",
                "h": "35000.00",
                "l": "33000.00",
                "v": "100.5",
                "n": 1000,
                "x": true,
                "q": "3450000.00",
                "V": "50.25",
                "Q": "1725000.00"
            }
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::Kline(kline_event) => {
                assert_eq!(kline_event.symbol, "BTCUSDT");
            },
            _ => panic!("Expected Kline variant"),
        }
    }

    #[test]
    fn test_websocket_event_ticker_variant() {
        let json = r#"{
            "type": "ticker",
            "e": "24hrTicker",
            "E": 1625097600000,
            "s": "ETHUSDT",
            "p": "100.00",
            "P": "4.0",
            "w": "2550.00",
            "x": "2500.00",
            "c": "2600.00",
            "Q": "5.0",
            "b": "2599.00",
            "B": "10.0",
            "a": "2601.00",
            "A": "8.0",
            "o": "2500.00",
            "h": "2650.00",
            "l": "2480.00",
            "v": "5000.0",
            "q": "12750000.00",
            "O": 1625011200000,
            "C": 1625097600000,
            "F": 2000,
            "L": 10000,
            "n": 8000
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::Ticker(ticker_event) => {
                assert_eq!(ticker_event.symbol, "ETHUSDT");
            },
            _ => panic!("Expected Ticker variant"),
        }
    }

    #[test]
    fn test_websocket_event_orderbook_variant() {
        let json = r#"{
            "type": "orderbook",
            "e": "depthUpdate",
            "E": 1625097600000,
            "s": "BTCUSDT",
            "U": 1000,
            "u": 1005,
            "b": [["34000.00", "1.5"]],
            "a": [["34001.00", "1.0"]]
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::OrderBook(orderbook_event) => {
                assert_eq!(orderbook_event.symbol, "BTCUSDT");
            },
            _ => panic!("Expected OrderBook variant"),
        }
    }

    #[test]
    fn test_websocket_event_market_data_variant() {
        let json = r#"{
            "type": "market_data",
            "symbol": "BTCUSDT",
            "price": 34500.0,
            "price_change_24h": 500.0,
            "price_change_percent_24h": 1.5,
            "volume_24h": 1000.5,
            "timestamp": 1625097600000
        }"#;

        let event: WebSocketEvent = serde_json::from_str(json).unwrap();
        match event {
            WebSocketEvent::MarketData(market_data) => {
                assert_eq!(market_data.symbol, "BTCUSDT");
                assert_eq!(market_data.price, 34500.0);
            },
            _ => panic!("Expected MarketData variant"),
        }
    }

    #[test]
    fn test_websocket_event_serialization() {
        let event = WebSocketEvent::Error {
            message: "Test error".to_string(),
        };

        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("error"));
        assert!(json.contains("Test error"));
    }

    #[test]
    fn test_kline_with_negative_values() {
        let kline = Kline {
            open_time: -1,
            open: "-100.00".to_string(),
            high: "-50.00".to_string(),
            low: "-200.00".to_string(),
            close: "-150.00".to_string(),
            volume: "-10.5".to_string(),
            close_time: -1,
            quote_asset_volume: "-1000.00".to_string(),
            number_of_trades: -1,
            taker_buy_base_asset_volume: "-5.25".to_string(),
            taker_buy_quote_asset_volume: "-500.00".to_string(),
            ignore: "-1".to_string(),
        };

        let json = serde_json::to_string(&kline).unwrap();
        let deserialized: Kline = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.open, "-100.00");
        assert_eq!(deserialized.number_of_trades, -1);
    }

    #[test]
    fn test_symbol_price_deserialization() {
        let json = r#"{
            "symbol": "BNBUSDT",
            "price": "450.25"
        }"#;

        let price: SymbolPrice = serde_json::from_str(json).unwrap();
        assert_eq!(price.symbol, "BNBUSDT");
        assert_eq!(price.price, "450.25");
    }

    #[test]
    fn test_kline_to_decimal_with_scientific_notation() {
        let kline = Kline {
            open_time: 1625097600000,
            open: "0.0000123".to_string(),
            high: "0.0000234".to_string(),
            low: "0.0000056".to_string(),
            close: "0.0000150".to_string(),
            volume: "1000000".to_string(),
            close_time: 1625097659999,
            quote_asset_volume: "15.5".to_string(),
            number_of_trades: 100,
            taker_buy_base_asset_volume: "500000".to_string(),
            taker_buy_quote_asset_volume: "7.5".to_string(),
            ignore: "0".to_string(),
        };

        let result = kline.to_decimal_values();
        assert!(result.is_ok());
    }

    #[test]
    fn test_futures_order_roundtrip() {
        let order = FuturesOrder {
            symbol: "ADAUSDT".to_string(),
            order_id: 99999,
            order_list_id: 0,
            client_order_id: "custom_id_123".to_string(),
            price: "1.25".to_string(),
            orig_qty: "100.0".to_string(),
            executed_qty: "100.0".to_string(),
            cumulative_quote_qty: "125.00".to_string(),
            status: "FILLED".to_string(),
            time_in_force: "IOC".to_string(),
            r#type: "MARKET".to_string(),
            side: "BUY".to_string(),
            stop_price: "1.20".to_string(),
            iceberg_qty: "10.0".to_string(),
            time: 1625097600000,
            update_time: 1625097605000,
            is_working: false,
            orig_quote_order_qty: "125.00".to_string(),
        };

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: FuturesOrder = serde_json::from_str(&json).unwrap();

        assert_eq!(order.symbol, deserialized.symbol);
        assert_eq!(order.order_id, deserialized.order_id);
        assert_eq!(order.status, deserialized.status);
        assert_eq!(order.is_working, deserialized.is_working);
    }

    #[test]
    fn test_account_info_with_empty_balances() {
        let account = AccountInfo {
            maker_commission: 0,
            taker_commission: 0,
            buyer_commission: 0,
            seller_commission: 0,
            can_trade: false,
            can_withdraw: false,
            can_deposit: false,
            update_time: 0,
            account_type: "SPOT".to_string(),
            balances: vec![],
            permissions: vec![],
        };

        let json = serde_json::to_string(&account).unwrap();
        let deserialized: AccountInfo = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.balances.len(), 0);
        assert_eq!(deserialized.permissions.len(), 0);
        assert!(!deserialized.can_trade);
    }

    #[test]
    fn test_new_order_request_with_all_none_options() {
        let order = NewOrderRequest {
            symbol: "BTCUSDT".to_string(),
            side: "BUY".to_string(),
            r#type: "MARKET".to_string(),
            quantity: None,
            quote_order_qty: None,
            price: None,
            new_client_order_id: None,
            stop_price: None,
            iceberg_qty: None,
            new_order_resp_type: None,
            time_in_force: None,
            reduce_only: None,
            close_position: None,
            position_side: None,
            working_type: None,
            price_protect: None,
        };

        let json = serde_json::to_string(&order).unwrap();
        let deserialized: NewOrderRequest = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.symbol, "BTCUSDT");
        assert_eq!(deserialized.quantity, None);
        assert_eq!(deserialized.price, None);
    }

    #[test]
    fn test_order_response_with_multiple_fills() {
        let response = OrderResponse {
            symbol: "BTCUSDT".to_string(),
            order_id: 11111,
            order_list_id: -1,
            client_order_id: "multi_fill_test".to_string(),
            transact_time: 1625097600000,
            price: "34000.00".to_string(),
            orig_qty: "1.0".to_string(),
            executed_qty: "1.0".to_string(),
            cumulative_quote_qty: "34000.00".to_string(),
            status: "FILLED".to_string(),
            time_in_force: "GTC".to_string(),
            r#type: "LIMIT".to_string(),
            side: "BUY".to_string(),
            fills: vec![
                Fill {
                    price: "34000.00".to_string(),
                    qty: "0.5".to_string(),
                    commission: "0.0005".to_string(),
                    commission_asset: "BTC".to_string(),
                    trade_id: 1001,
                },
                Fill {
                    price: "34000.00".to_string(),
                    qty: "0.5".to_string(),
                    commission: "0.0005".to_string(),
                    commission_asset: "BTC".to_string(),
                    trade_id: 1002,
                },
            ],
        };

        let json = serde_json::to_string(&response).unwrap();
        let deserialized: OrderResponse = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.fills.len(), 2);
        assert_eq!(deserialized.fills[0].trade_id, 1001);
        assert_eq!(deserialized.fills[1].trade_id, 1002);
    }

    #[test]
    fn test_chart_candle_roundtrip() {
        let candle = ChartCandle {
            timestamp: 1625097600000,
            open: 34567.89,
            high: 35678.90,
            low: 33456.78,
            close: 34890.12,
            volume: 1234.5678,
            is_closed: true,
        };

        let json = serde_json::to_string(&candle).unwrap();
        let deserialized: ChartCandle = serde_json::from_str(&json).unwrap();

        assert_eq!(candle.timestamp, deserialized.timestamp);
        assert_eq!(candle.open, deserialized.open);
        assert_eq!(candle.is_closed, deserialized.is_closed);
    }

    #[test]
    fn test_kline_data_with_negative_trades() {
        let kline_data = KlineData {
            kline_start_time: 1625097600000,
            kline_close_time: 1625097659999,
            symbol: "BTCUSDT".to_string(),
            interval: "1m".to_string(),
            first_trade_id: -100,
            last_trade_id: -50,
            open_price: "34000.00".to_string(),
            close_price: "34500.00".to_string(),
            high_price: "35000.00".to_string(),
            low_price: "33000.00".to_string(),
            base_asset_volume: "100.5".to_string(),
            number_of_trades: -10,
            is_this_kline_closed: false,
            quote_asset_volume: "3450000.00".to_string(),
            taker_buy_base_asset_volume: "50.25".to_string(),
            taker_buy_quote_asset_volume: "1725000.00".to_string(),
        };

        let json = serde_json::to_string(&kline_data).unwrap();
        let deserialized: KlineData = serde_json::from_str(&json).unwrap();

        assert_eq!(deserialized.first_trade_id, -100);
        assert_eq!(deserialized.number_of_trades, -10);
    }

    #[test]
    fn test_futures_position_roundtrip() {
        let position = FuturesPosition {
            symbol: "SOLUSDT".to_string(),
            position_amt: "50.0".to_string(),
            entry_price: "100.50".to_string(),
            mark_price: "105.75".to_string(),
            unrealized_pnl: "262.50".to_string(),
            liquidation_price: "75.00".to_string(),
            leverage: "20".to_string(),
            max_notional_value: "200000.00".to_string(),
            margin_type: "isolated".to_string(),
            isolated_margin: "250.00".to_string(),
            is_auto_add_margin: true,
            position_side: "BOTH".to_string(),
            notional: "5287.50".to_string(),
            isolated_wallet: "250.00".to_string(),
            update_time: 1625097600000,
        };

        let json = serde_json::to_string(&position).unwrap();
        let deserialized: FuturesPosition = serde_json::from_str(&json).unwrap();

        assert_eq!(position.symbol, deserialized.symbol);
        assert_eq!(position.leverage, deserialized.leverage);
        assert_eq!(position.is_auto_add_margin, deserialized.is_auto_add_margin);
    }

    #[test]
    fn test_order_side_display() {
        assert_eq!(OrderSide::Buy.to_string(), "BUY");
        assert_eq!(OrderSide::Sell.to_string(), "SELL");
    }

    #[test]
    fn test_order_side_serialization() {
        let buy = OrderSide::Buy;
        let sell = OrderSide::Sell;

        let buy_json = serde_json::to_string(&buy).unwrap();
        let sell_json = serde_json::to_string(&sell).unwrap();

        assert_eq!(buy_json, "\"BUY\"");
        assert_eq!(sell_json, "\"SELL\"");
    }

    #[test]
    fn test_spot_order_type_display() {
        assert_eq!(SpotOrderType::Market.to_string(), "MARKET");
        assert_eq!(SpotOrderType::Limit.to_string(), "LIMIT");
        assert_eq!(SpotOrderType::StopLossLimit.to_string(), "STOP_LOSS_LIMIT");
        assert_eq!(
            SpotOrderType::TakeProfitLimit.to_string(),
            "TAKE_PROFIT_LIMIT"
        );
        assert_eq!(SpotOrderType::LimitMaker.to_string(), "LIMIT_MAKER");
    }

    #[test]
    fn test_time_in_force_display() {
        assert_eq!(TimeInForce::Gtc.to_string(), "GTC");
        assert_eq!(TimeInForce::Ioc.to_string(), "IOC");
        assert_eq!(TimeInForce::Fok.to_string(), "FOK");
    }

    #[test]
    fn test_spot_order_request_market() {
        let order = SpotOrderRequest::market("BTCUSDT", OrderSide::Buy, "0.01");

        assert_eq!(order.symbol, "BTCUSDT");
        assert_eq!(order.side, OrderSide::Buy);
        assert_eq!(order.order_type, SpotOrderType::Market);
        assert_eq!(order.quantity, Some("0.01".to_string()));
        assert_eq!(order.new_order_resp_type, Some("FULL".to_string()));
    }

    #[test]
    fn test_spot_order_request_limit() {
        let order = SpotOrderRequest::limit("ETHUSDT", OrderSide::Sell, "1.5", "2000.00");

        assert_eq!(order.symbol, "ETHUSDT");
        assert_eq!(order.side, OrderSide::Sell);
        assert_eq!(order.order_type, SpotOrderType::Limit);
        assert_eq!(order.price, Some("2000.00".to_string()));
        assert_eq!(order.time_in_force, Some(TimeInForce::Gtc));
    }

    #[test]
    fn test_spot_order_request_stop_loss_limit() {
        let order = SpotOrderRequest::stop_loss_limit(
            "BNBUSDT",
            OrderSide::Sell,
            "10.0",
            "300.00",
            "305.00",
        );

        assert_eq!(order.symbol, "BNBUSDT");
        assert_eq!(order.order_type, SpotOrderType::StopLossLimit);
        assert_eq!(order.price, Some("300.00".to_string()));
        assert_eq!(order.stop_price, Some("305.00".to_string()));
    }

    #[test]
    fn test_spot_order_request_take_profit_limit() {
        let order = SpotOrderRequest::take_profit_limit(
            "ADAUSDT",
            OrderSide::Sell,
            "100.0",
            "0.50",
            "0.48",
        );

        assert_eq!(order.symbol, "ADAUSDT");
        assert_eq!(order.order_type, SpotOrderType::TakeProfitLimit);
        assert_eq!(order.price, Some("0.50".to_string()));
        assert_eq!(order.stop_price, Some("0.48".to_string()));
    }

    #[test]
    fn test_spot_order_request_with_client_order_id() {
        let order = SpotOrderRequest::market("BTCUSDT", OrderSide::Buy, "0.01")
            .with_client_order_id("my_order_123");

        assert_eq!(order.client_order_id, Some("my_order_123".to_string()));
    }

    #[test]
    fn test_spot_order_request_with_time_in_force() {
        let order = SpotOrderRequest::limit("ETHUSDT", OrderSide::Sell, "1.0", "2000.00")
            .with_time_in_force(TimeInForce::Ioc);

        assert_eq!(order.time_in_force, Some(TimeInForce::Ioc));
    }

    #[test]
    fn test_spot_order_request_with_quote_qty() {
        let order =
            SpotOrderRequest::market("BTCUSDT", OrderSide::Buy, "0.01").with_quote_qty("1000.00");

        assert_eq!(order.quote_order_qty, Some("1000.00".to_string()));
    }

    #[test]
    fn test_oco_order_request_new() {
        let oco = OcoOrderRequest::new(
            "BTCUSDT",
            OrderSide::Sell,
            "0.01",
            "36000.00",
            "33000.00",
            "32900.00",
        );

        assert_eq!(oco.symbol, "BTCUSDT");
        assert_eq!(oco.side, OrderSide::Sell);
        assert_eq!(oco.above_type, "LIMIT_MAKER");
        assert_eq!(oco.below_type, "STOP_LOSS_LIMIT");
        assert_eq!(oco.above_price, Some("36000.00".to_string()));
        assert_eq!(oco.below_stop_price, Some("33000.00".to_string()));
    }

    #[test]
    fn test_oco_order_request_new_short() {
        let oco = OcoOrderRequest::new_short("BTCUSDT", "0.01", "33000.00", "36000.00", "36100.00");

        assert_eq!(oco.side, OrderSide::Buy);
        assert_eq!(oco.below_type, "LIMIT_MAKER");
        assert_eq!(oco.above_type, "STOP_LOSS_LIMIT");
    }

    #[test]
    fn test_oco_order_request_with_client_order_ids() {
        let oco = OcoOrderRequest::new(
            "ETHUSDT",
            OrderSide::Sell,
            "1.0",
            "2100.00",
            "1900.00",
            "1890.00",
        )
        .with_client_order_ids("list_001", "above_001", "below_001");

        assert_eq!(oco.list_client_order_id, Some("list_001".to_string()));
        assert_eq!(oco.above_client_order_id, Some("above_001".to_string()));
        assert_eq!(oco.below_client_order_id, Some("below_001".to_string()));
    }

    #[test]
    fn test_execution_report_is_filled() {
        let mut report = ExecutionReport {
            event_type: "executionReport".to_string(),
            event_time: 123456,
            symbol: "BTCUSDT".to_string(),
            client_order_id: "test".to_string(),
            side: "BUY".to_string(),
            order_type: "LIMIT".to_string(),
            time_in_force: "GTC".to_string(),
            order_quantity: "0.01".to_string(),
            order_price: "34000.00".to_string(),
            stop_price: "0.00".to_string(),
            iceberg_quantity: "0.00".to_string(),
            original_client_order_id: "".to_string(),
            execution_type: "TRADE".to_string(),
            order_status: "FILLED".to_string(),
            order_reject_reason: "".to_string(),
            order_id: 12345,
            last_executed_quantity: "0.01".to_string(),
            cumulative_filled_quantity: "0.01".to_string(),
            last_executed_price: "34000.00".to_string(),
            commission_amount: "0.00001".to_string(),
            commission_asset: Some("BTC".to_string()),
            transaction_time: 123456,
            trade_id: 1001,
            is_on_book: false,
            is_maker: false,
            order_creation_time: 123455,
            cumulative_quote_qty: "340.00".to_string(),
            last_quote_qty: "340.00".to_string(),
            quote_order_qty: "0.00".to_string(),
        };

        assert!(report.is_filled());

        report.order_status = "PARTIALLY_FILLED".to_string();
        assert!(!report.is_filled());

        report.order_status = "NEW".to_string();
        assert!(!report.is_filled());
    }

    #[test]
    fn test_deserialize_bool_from_string_true() {
        let json = r#"{"test": "true"}"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "super::deserialize_bool_from_anything")]
            test: bool,
        }

        let result: Test = serde_json::from_str(json).unwrap();
        assert!(result.test);
    }

    #[test]
    fn test_deserialize_bool_from_string_false() {
        let json = r#"{"test": "false"}"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "super::deserialize_bool_from_anything")]
            test: bool,
        }

        let result: Test = serde_json::from_str(json).unwrap();
        assert!(!result.test);
    }

    #[test]
    fn test_deserialize_bool_from_number() {
        let json = r#"{"test": 1}"#;
        #[derive(Deserialize)]
        struct Test {
            #[serde(deserialize_with = "super::deserialize_bool_from_anything")]
            test: bool,
        }

        let result: Test = serde_json::from_str(json).unwrap();
        assert!(result.test);

        let json_false = r#"{"test": 0}"#;
        let result_false: Test = serde_json::from_str(json_false).unwrap();
        assert!(!result_false.test);
    }
}
