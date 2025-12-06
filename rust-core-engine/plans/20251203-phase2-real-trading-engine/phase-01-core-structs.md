# Phase 1: Core Structs

## Context

Reference existing patterns:
- `src/paper_trading/engine.rs:33-71` - PaperTradingEngine struct
- `src/trading/position_manager.rs:9-26` - Position struct, DashMap usage
- `src/binance/types.rs:609-740` - ExecutionReport for order state

## Requirements

1. **RealTradingEngine** - Main engine struct with thread-safe state
2. **RealPosition** - Extended position with order tracking
3. **RealOrder** - Order state machine with lifecycle tracking
4. **RealTradingConfig** - Configuration for risk limits, reconciliation intervals

## Implementation Steps

### 1.1 Create Module Structure

```
src/real_trading/
  mod.rs           # Module exports
  engine.rs        # RealTradingEngine
  position.rs      # RealPosition
  order.rs         # RealOrder, OrderState
  config.rs        # RealTradingConfig
```

### 1.2 RealOrder Struct (`order.rs`)

```rust
// @spec:FR-REAL-010 - Real Order Tracking
use dashmap::DashMap;
use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderState {
    Pending,           // Order submitted, awaiting confirmation
    New,               // Confirmed by exchange
    PartiallyFilled,   // Some quantity filled
    Filled,            // Fully filled
    Cancelled,         // Cancelled by user
    Rejected,          // Rejected by exchange
    Expired,           // Time expired
}

#[derive(Debug, Clone)]
pub struct RealOrder {
    pub id: String,                    // Client order ID
    pub exchange_order_id: i64,        // Binance order ID
    pub symbol: String,
    pub side: String,                  // "BUY" or "SELL"
    pub order_type: String,            // "MARKET", "LIMIT", etc.
    pub original_quantity: f64,
    pub executed_quantity: f64,
    pub remaining_quantity: f64,
    pub price: Option<f64>,            // For limit orders
    pub stop_price: Option<f64>,       // For stop orders
    pub average_fill_price: f64,
    pub state: OrderState,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub fills: Vec<OrderFill>,         // Individual fill records
}

#[derive(Debug, Clone)]
pub struct OrderFill {
    pub trade_id: i64,
    pub price: f64,
    pub quantity: f64,
    pub commission: f64,
    pub commission_asset: String,
    pub timestamp: DateTime<Utc>,
}

impl RealOrder {
    pub fn new(/* params */) -> Self { /* ... */ }
    pub fn update_from_execution_report(&mut self, report: &ExecutionReport) { /* ... */ }
    pub fn is_active(&self) -> bool { /* ... */ }
    pub fn is_terminal(&self) -> bool { /* ... */ }
}
```

### 1.3 RealPosition Struct (`position.rs`)

```rust
// @spec:FR-REAL-011 - Real Position Tracking
use dashmap::DashMap;

#[derive(Debug, Clone)]
pub struct RealPosition {
    pub id: String,
    pub symbol: String,
    pub side: String,                  // "LONG" or "SHORT"
    pub quantity: f64,
    pub entry_price: f64,              // Average entry
    pub current_price: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,             // From partial closes
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub stop_loss_order_id: Option<String>,
    pub take_profit_order_id: Option<String>,
    pub entry_order_ids: Vec<String>,  // Orders that opened this position
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl RealPosition {
    pub fn new(/* params */) -> Self { /* ... */ }
    pub fn add_fill(&mut self, price: f64, quantity: f64) { /* ... */ }
    pub fn partial_close(&mut self, price: f64, quantity: f64) -> f64 { /* PnL */ }
    pub fn update_price(&mut self, price: f64) { /* ... */ }
    pub fn calculate_pnl(&self) -> f64 { /* ... */ }
}
```

### 1.4 RealTradingConfig (`config.rs`)

```rust
// @spec:FR-REAL-012 - Real Trading Configuration
#[derive(Debug, Clone)]
pub struct RealTradingConfig {
    // Risk limits
    pub max_position_size_usdt: f64,       // Max single position value
    pub max_total_exposure_usdt: f64,      // Max total exposure
    pub max_daily_loss_usdt: f64,          // Daily loss limit
    pub max_positions: u32,                // Max concurrent positions
    pub risk_per_trade_percent: f64,       // % of balance per trade

    // Circuit breaker
    pub circuit_breaker_errors: u32,       // Errors before pause (default: 3)
    pub circuit_breaker_cooldown_secs: u64,// Cooldown period (default: 300)

    // Reconciliation
    pub reconciliation_interval_secs: u64, // REST sync interval (default: 300)
    pub stale_order_timeout_secs: u64,     // Cancel stale orders after

    // Order defaults
    pub default_slippage_percent: f64,     // Price buffer for limit orders
    pub order_timeout_secs: u64,           // Cancel if not filled
}
```

### 1.5 RealTradingEngine (`engine.rs`)

```rust
// @spec:FR-REAL-013 - Real Trading Engine
use std::sync::Arc;
use tokio::sync::{broadcast, RwLock, Mutex};
use dashmap::DashMap;

#[derive(Clone)]
pub struct RealTradingEngine {
    // Thread-safe state
    positions: Arc<DashMap<String, RealPosition>>,      // symbol -> position
    orders: Arc<DashMap<String, RealOrder>>,            // client_order_id -> order
    balances: Arc<RwLock<HashMap<String, f64>>>,        // asset -> free balance

    // Configuration
    config: Arc<RwLock<RealTradingConfig>>,

    // External services
    binance_client: BinanceClient,
    risk_manager: RiskManager,
    user_data_stream: Arc<RwLock<UserDataStreamManager>>,

    // Event broadcasting
    event_tx: broadcast::Sender<RealTradingEvent>,

    // Engine state
    is_running: Arc<RwLock<bool>>,
    circuit_breaker: Arc<RwLock<CircuitBreakerState>>,

    // Execution lock (prevent race conditions)
    execution_lock: Arc<Mutex<()>>,

    // Metrics
    daily_pnl: Arc<RwLock<f64>>,
    daily_trades: Arc<RwLock<u32>>,
    consecutive_errors: Arc<RwLock<u32>>,
}

#[derive(Debug, Clone)]
pub struct CircuitBreakerState {
    pub is_open: bool,
    pub error_count: u32,
    pub opened_at: Option<DateTime<Utc>>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub enum RealTradingEvent {
    OrderPlaced(RealOrder),
    OrderFilled(RealOrder),
    OrderPartiallyFilled(RealOrder),
    OrderCancelled(RealOrder),
    OrderRejected { order: RealOrder, reason: String },
    PositionOpened(RealPosition),
    PositionUpdated(RealPosition),
    PositionClosed { position: RealPosition, pnl: f64 },
    BalanceUpdated { asset: String, free: f64, locked: f64 },
    CircuitBreakerOpened(String),
    CircuitBreakerClosed,
    ReconciliationComplete { discrepancies: u32 },
    Error(String),
}
```

## Success Criteria

- [ ] All structs compile without errors
- [ ] Thread-safety verified: DashMap for high-frequency access, RwLock for balance
- [ ] OrderState transitions are valid (no invalid state changes)
- [ ] Position PnL calculation matches expected values
- [ ] Unit tests pass for all struct methods
- [ ] Config defaults are sane for testnet testing

## Risk Considerations

- **Memory**: DashMap entries must be cleaned up (closed positions/orders)
- **Deadlocks**: Avoid nested locks, use try_lock where appropriate
- **Overflow**: Use checked arithmetic for financial calculations
- **Precision**: Consider Decimal type for monetary values (evaluate perf impact)

## Tests to Write

```rust
#[cfg(test)]
mod tests {
    // Order state transitions
    #[test] fn test_order_new_to_filled() { }
    #[test] fn test_order_new_to_cancelled() { }
    #[test] fn test_order_partial_fill_accumulation() { }

    // Position calculations
    #[test] fn test_position_average_entry_price() { }
    #[test] fn test_position_unrealized_pnl_long() { }
    #[test] fn test_position_unrealized_pnl_short() { }
    #[test] fn test_position_partial_close_realized_pnl() { }

    // Circuit breaker
    #[test] fn test_circuit_breaker_opens_on_errors() { }
    #[test] fn test_circuit_breaker_closes_after_cooldown() { }
}
```
