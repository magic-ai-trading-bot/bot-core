# FR-PAPER-TRADING: Paper Trading Simulation Functional Requirements

**Document Version:** 1.0
**Last Updated:** 2025-10-10
**Status:** Draft
**Owner:** Trading Engine Team

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Scope](#2-scope)
3. [Functional Requirements](#3-functional-requirements)
   - [FR-PAPER-TRADING-001: Paper Trading Mode Initialization](#fr-paper-trading-001-paper-trading-mode-initialization)
   - [FR-PAPER-TRADING-002: Simulated Order Execution](#fr-paper-trading-002-simulated-order-execution)
   - [FR-PAPER-TRADING-003: Virtual Portfolio Management](#fr-paper-trading-003-virtual-portfolio-management)
   - [FR-PAPER-TRADING-004: Paper Trading History & Analytics](#fr-paper-trading-004-paper-trading-history--analytics)
   - [FR-PAPER-TRADING-005: Strategy Testing & Validation](#fr-paper-trading-005-strategy-testing--validation)
   - [FR-PAPER-TRADING-006: Paper vs Live Comparison](#fr-paper-trading-006-paper-vs-live-comparison)
   - [FR-PAPER-TRADING-007: Risk Management Simulation](#fr-paper-trading-007-risk-management-simulation)
   - [FR-PAPER-TRADING-008: Performance Metrics Calculation](#fr-paper-trading-008-performance-metrics-calculation)
4. [Data Models](#4-data-models)
5. [Business Rules](#5-business-rules)
6. [Acceptance Criteria](#6-acceptance-criteria)
7. [Traceability](#7-traceability)

---

## 1. Introduction

### 1.1 Purpose
This document specifies the functional requirements for the Paper Trading Simulation system within the cryptocurrency trading bot. Paper trading allows users to test trading strategies, algorithms, and risk management rules in a risk-free environment using simulated orders and virtual capital.

### 1.2 Background
The paper trading system simulates real market conditions without risking actual capital. It provides a safe environment for:
- Strategy development and testing
- Algorithm validation
- Risk-free learning and experimentation
- Performance benchmarking before live trading
- Strategy optimization and parameter tuning

### 1.3 System Context
The paper trading engine is implemented in Rust (`rust-core-engine/src/paper_trading/`) and integrates with:
- **Strategy Engine**: Receives trading signals from strategies
- **Market Data Service**: Consumes real-time and historical market data
- **Risk Management**: Simulates risk checks and position limits
- **MongoDB**: Persists paper trading portfolio state and trade history

---

## 2. Scope

### 2.1 In Scope
- ☑ Virtual account initialization with configurable starting balance
- ☑ Simulated order placement and execution
- ☑ Virtual portfolio tracking with real-time PnL calculation
- ☑ Comprehensive performance metrics and analytics
- ☑ Strategy backtesting on historical data
- ☑ Forward testing on live market data (paper mode)
- ☑ Slippage and fee simulation
- ☑ Margin and leverage simulation
- ☑ Stop-loss and take-profit simulation
- ☑ Liquidation simulation
- ☑ Funding fee simulation for perpetual futures
- ☑ Multi-symbol portfolio support
- ☑ Trade history persistence and retrieval

### 2.2 Out of Scope
- ☐ Live order execution (covered in separate specification)
- ☐ Real fund transfers or withdrawals
- ☐ Exchange API integration for live trading
- ☐ Social trading or copy trading features

---

## 3. Functional Requirements

### FR-PAPER-TRADING-001: Paper Trading Mode Initialization

**Priority:** CRITICAL
**Spec ID:** @spec:FR-PAPER-TRADING-001
**Related APIs:** POST `/api/v1/paper-trading/initialize`, GET `/api/v1/paper-trading/portfolio`

#### 3.1.1 Description
The system shall provide functionality to initialize a new paper trading environment with configurable parameters including starting balance, leverage limits, and risk parameters.

#### 3.1.2 Detailed Requirements

##### 3.1.2.1 Portfolio Initialization
☐ **REQ-001-001**: System shall create a new `PaperPortfolio` instance with user-specified initial balance (USDT)
- Minimum balance: $100 USDT
- Maximum balance: $1,000,000 USDT
- Default balance: $10,000 USDT

☐ **REQ-001-002**: System shall initialize portfolio with the following default state:
```rust
PaperPortfolio {
    initial_balance: user_specified,
    cash_balance: initial_balance,
    equity: initial_balance,
    margin_used: 0.0,
    free_margin: initial_balance,
    margin_level: 0.0,
    trades: HashMap::new(),
    open_trade_ids: Vec::new(),
    closed_trade_ids: Vec::new(),
    current_prices: HashMap::new(),
    funding_rates: HashMap::new(),
    created_at: Utc::now(),
    last_updated: Utc::now(),
    metrics: PortfolioMetrics::default(),
    daily_performance: Vec::new(),
}
```

☐ **REQ-001-003**: System shall validate initial balance against minimum and maximum limits before creation

☐ **REQ-001-004**: System shall persist portfolio state to MongoDB upon initialization

☐ **REQ-001-005**: System shall generate unique portfolio ID using UUID v4 format

##### 3.1.2.2 Configuration Settings
☐ **REQ-001-006**: System shall support the following configurable parameters:
- **Max Leverage**: 1x to 125x (default: 10x)
- **Max Position Size**: Percentage of portfolio (default: 50%)
- **Max Open Positions**: 1 to 100 (default: 10)
- **Default Order Type**: Market, Limit, Stop-Market, Stop-Limit
- **Slippage Model**: Fixed percentage or dynamic based on volatility
- **Fee Structure**: Maker/Taker fee percentages

☐ **REQ-001-007**: System shall allow users to enable/disable specific features:
- Funding fee simulation (default: enabled)
- Liquidation simulation (default: enabled)
- Slippage simulation (default: enabled)
- Real-time price updates (default: enabled)

##### 3.1.2.3 Mode Switching
☐ **REQ-001-008**: System shall support switching between paper and live trading modes

☐ **REQ-001-009**: System shall require explicit user confirmation before switching to live mode

☐ **REQ-001-010**: System shall prevent accidental live trading by:
- Requiring two-factor authentication for live mode
- Displaying clear mode indicators in UI
- Logging all mode switches with timestamp and user ID

☐ **REQ-001-011**: System shall maintain separate portfolio states for paper and live modes

#### 3.1.3 Business Rules
- **BR-001-001**: Only one active paper trading portfolio per user account
- **BR-001-002**: Portfolio reset requires explicit user confirmation
- **BR-001-003**: Initial balance cannot be modified after portfolio creation (must reset)
- **BR-001-004**: Portfolio creation timestamp must use UTC timezone

#### 3.1.4 Error Handling
☐ **REQ-001-012**: System shall return error if initial balance is below minimum threshold
☐ **REQ-001-013**: System shall return error if user already has an active paper portfolio
☐ **REQ-001-014**: System shall validate all configuration parameters before initialization
☐ **REQ-001-015**: System shall rollback portfolio creation if MongoDB persistence fails

#### 3.1.5 Acceptance Criteria
```gherkin
Given a user wants to start paper trading
When they initialize a portfolio with $10,000 USDT
Then a new portfolio is created with:
  ☐ initial_balance = 10000.0
  ☐ cash_balance = 10000.0
  ☐ equity = 10000.0
  ☐ margin_used = 0.0
  ☐ free_margin = 10000.0
  ☐ empty trades list
  ☐ unique portfolio ID
  ☐ persisted to MongoDB
```

---

### FR-PAPER-TRADING-002: Simulated Order Execution

**Priority:** CRITICAL
**Spec ID:** @spec:FR-PAPER-TRADING-002
**Related APIs:** POST `/api/v1/paper-trading/orders`, GET `/api/v1/paper-trading/orders/{id}`

#### 3.2.1 Description
The system shall simulate realistic order execution including order placement, fill logic, slippage, and fee calculation using real-time market data.

#### 3.2.2 Detailed Requirements

##### 3.2.2.1 Order Types Support
☐ **REQ-002-001**: System shall support the following order types:
- **Market Order**: Immediate execution at current market price
- **Limit Order**: Execution when price reaches specified limit
- **Stop-Market Order**: Market order triggered at stop price
- **Stop-Limit Order**: Limit order triggered at stop price

☐ **REQ-002-002**: System shall validate order parameters:
- Symbol must be valid and supported
- Quantity must be > 0
- Price must be > 0 (for limit orders)
- Stop price must be > 0 (for stop orders)
- Leverage must be within configured limits (1x to 125x)

##### 3.2.2.2 Market Order Execution
☐ **REQ-002-003**: For market orders, system shall:
1. Fetch current market price from market data cache
2. Apply slippage simulation
3. Calculate trading fees (taker fee)
4. Verify sufficient free margin
5. Create and execute PaperTrade
6. Update portfolio state

**Slippage Calculation Formula:**
```rust
// Fixed slippage model
let slippage_pct = 0.001; // 0.1% default
let executed_price = if side == Long {
    current_price * (1.0 + slippage_pct)  // Buy at slightly higher price
} else {
    current_price * (1.0 - slippage_pct)  // Sell at slightly lower price
};

// Dynamic slippage model (based on volatility)
let volatility = calculate_price_volatility(recent_candles, 14);
let dynamic_slippage = base_slippage * (1.0 + volatility);
let executed_price = current_price * (1.0 ± dynamic_slippage);
```

☐ **REQ-002-004**: System shall calculate taker fees for market orders:
```rust
let taker_fee_rate = 0.0004; // 0.04% Binance taker fee
let notional_value = quantity * executed_price;
let trading_fees = notional_value * taker_fee_rate;
```

##### 3.2.2.3 Limit Order Execution
☐ **REQ-002-005**: For limit orders, system shall:
1. Validate limit price is valid for order side:
   - Long: limit price ≤ current price
   - Short: limit price ≥ current price
2. Store order in pending orders queue
3. Monitor market price on each tick
4. Execute when market price reaches limit price
5. Apply maker fees (lower than taker)

☐ **REQ-002-006**: System shall calculate maker fees for limit orders:
```rust
let maker_fee_rate = 0.0002; // 0.02% Binance maker fee
let notional_value = quantity * limit_price;
let trading_fees = notional_value * maker_fee_rate;
```

☐ **REQ-002-007**: System shall support limit order expiration:
- Good-Till-Cancelled (GTC): default, no expiration
- Immediate-Or-Cancel (IOC): fill immediately or cancel
- Fill-Or-Kill (FOK): fill completely or cancel
- Good-Till-Time (GTT): expire at specified timestamp

##### 3.2.2.4 Stop Order Execution
☐ **REQ-002-008**: For stop orders, system shall:
1. Validate stop price trigger condition:
   - Long stop-loss: stop price < current price
   - Long take-profit: stop price > current price
   - Short stop-loss: stop price > current price
   - Short take-profit: stop price < current price
2. Monitor market price continuously
3. Trigger order when stop condition met
4. Execute as market or limit order based on type

☐ **REQ-002-009**: System shall implement stop price triggering logic:
```rust
fn should_trigger_stop(
    current_price: f64,
    stop_price: f64,
    side: TradeSide,
    stop_type: StopType,
) -> bool {
    match (side, stop_type) {
        (Long, StopLoss) => current_price <= stop_price,
        (Long, TakeProfit) => current_price >= stop_price,
        (Short, StopLoss) => current_price >= stop_price,
        (Short, TakeProfit) => current_price <= stop_price,
    }
}
```

##### 3.2.2.5 Position Entry Simulation
☐ **REQ-002-010**: When opening a position, system shall create PaperTrade with:
```rust
PaperTrade {
    id: uuid::Uuid::new_v4().to_string(),
    symbol: order.symbol.clone(),
    side: order.side,
    quantity: order.quantity,
    leverage: order.leverage,
    entry_price: executed_price,
    exit_price: None,
    unrealized_pnl: 0.0,
    realized_pnl: None,
    trading_fees: calculated_fees,
    funding_fees: 0.0,
    initial_margin: (order.quantity * executed_price) / order.leverage,
    maintenance_margin: initial_margin * 0.4, // 40% for liquidation
    stop_loss: order.stop_loss,
    take_profit: order.take_profit,
    status: TradeStatus::Open,
    open_time: Utc::now(),
    close_time: None,
    close_reason: None,
    metadata: HashMap::new(),
}
```

☐ **REQ-002-011**: System shall verify margin requirements:
```rust
let required_margin = (quantity * price) / leverage;
if required_margin > portfolio.free_margin {
    return Err("Insufficient margin");
}
```

☐ **REQ-002-012**: System shall update portfolio state after order execution:
```rust
portfolio.margin_used += trade.initial_margin;
portfolio.free_margin = portfolio.equity - portfolio.margin_used;
portfolio.open_trade_ids.push(trade.id.clone());
portfolio.trades.insert(trade.id.clone(), trade);
```

##### 3.2.2.6 Order Rejection Handling
☐ **REQ-002-013**: System shall reject orders if:
- Insufficient free margin for required position margin
- Position size exceeds maximum allowed percentage
- Number of open positions exceeds limit
- Symbol not supported or market closed
- Invalid order parameters (negative quantity, zero price, etc.)
- Leverage exceeds maximum allowed

☐ **REQ-002-014**: System shall return detailed rejection reason with error code

#### 3.2.3 Business Rules
- **BR-002-001**: All orders must be validated before execution
- **BR-002-002**: Slippage simulation must be enabled unless explicitly disabled
- **BR-002-003**: Market orders execute immediately at simulated price
- **BR-002-004**: Limit orders only execute when price condition met
- **BR-002-005**: Fee calculation must match target exchange (Binance default)
- **BR-002-006**: Order execution must update portfolio atomically

#### 3.2.4 Acceptance Criteria
```gherkin
Scenario: Execute market long order
  Given portfolio has $10,000 free margin
  And current BTC price is $50,000
  When user places market long order:
    - Symbol: BTCUSDT
    - Quantity: 0.1 BTC
    - Leverage: 10x
  Then order executes with:
    ☐ Entry price = $50,000 * 1.001 (slippage)
    ☐ Notional value = 0.1 * $50,050 = $5,005
    ☐ Required margin = $5,005 / 10 = $500.50
    ☐ Trading fees = $5,005 * 0.0004 = $2.00
    ☐ Margin used = $500.50
    ☐ Free margin = $10,000 - $500.50 = $9,499.50
    ☐ Trade status = Open
    ☐ Trade persisted to database
```

---

### FR-PAPER-TRADING-003: Virtual Portfolio Management

**Priority:** CRITICAL
**Spec ID:** @spec:FR-PAPER-TRADING-003
**Related APIs:** GET `/api/v1/paper-trading/portfolio`, PATCH `/api/v1/paper-trading/portfolio`

#### 3.3.1 Description
The system shall track and manage the virtual portfolio including positions, margin, equity, and real-time profit/loss calculations.

#### 3.3.2 Detailed Requirements

##### 3.3.2.1 Real-Time Position Tracking
☐ **REQ-003-001**: System shall maintain real-time list of:
- Open positions (trades with status = Open)
- Closed positions (trades with status = Closed)
- Pending orders (limit/stop orders not yet executed)

☐ **REQ-003-002**: For each open position, system shall track:
- Symbol, side (Long/Short), quantity, leverage
- Entry price and current market price
- Unrealized PnL
- Initial margin and maintenance margin
- Stop-loss and take-profit levels
- Accumulated funding fees
- Position age (duration since entry)

##### 3.3.2.2 Unrealized PnL Calculation
☐ **REQ-003-003**: System shall calculate unrealized PnL using formula:
```rust
fn calculate_unrealized_pnl(
    side: TradeSide,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
) -> f64 {
    match side {
        TradeSide::Long => {
            // Long: profit when price increases
            quantity * (current_price - entry_price)
        },
        TradeSide::Short => {
            // Short: profit when price decreases
            quantity * (entry_price - current_price)
        },
    }
}
```

☐ **REQ-003-004**: System shall update unrealized PnL on every price update (real-time)

☐ **REQ-003-005**: System shall account for accumulated fees in total PnL:
```rust
let total_unrealized_pnl = raw_pnl - trading_fees - accumulated_funding_fees;
```

##### 3.3.2.3 Equity and Margin Calculations
☐ **REQ-003-006**: System shall calculate portfolio equity:
```rust
// Equity = Cash Balance + Total Unrealized PnL
let total_unrealized_pnl: f64 = open_trades
    .iter()
    .map(|t| t.unrealized_pnl)
    .sum();

portfolio.equity = portfolio.cash_balance + total_unrealized_pnl;
```

☐ **REQ-003-007**: System shall track total margin used:
```rust
let margin_used: f64 = open_trades
    .iter()
    .map(|t| t.initial_margin)
    .sum();

portfolio.margin_used = margin_used;
```

☐ **REQ-003-008**: System shall calculate free margin:
```rust
portfolio.free_margin = portfolio.equity - portfolio.margin_used;
```

☐ **REQ-003-009**: System shall calculate margin level percentage:
```rust
portfolio.margin_level = if portfolio.margin_used > 0.0 {
    (portfolio.equity / portfolio.margin_used) * 100.0
} else {
    0.0 // No open positions
};
```

##### 3.3.2.4 Funding Fee Simulation
☐ **REQ-003-010**: System shall simulate funding fees for perpetual futures:
- Funding occurs every 8 hours (00:00, 08:00, 16:00 UTC)
- Funding rate retrieved from market data service
- Positive rate: longs pay shorts
- Negative rate: shorts pay longs

☐ **REQ-003-011**: System shall calculate funding fee:
```rust
fn calculate_funding_fee(
    side: TradeSide,
    position_value: f64,
    funding_rate: f64,
) -> f64 {
    // Funding fee = Position Value × Funding Rate
    let fee = position_value * funding_rate;

    match side {
        TradeSide::Long => -fee,  // Longs pay (negative)
        TradeSide::Short => fee,  // Shorts receive (positive)
    }
}
```

☐ **REQ-003-012**: System shall apply funding fees to open positions every 8 hours

☐ **REQ-003-013**: System shall accumulate total funding fees per position:
```rust
trade.funding_fees += calculated_funding_fee;
trade.unrealized_pnl -= calculated_funding_fee.abs();
```

##### 3.3.2.5 Price Update Mechanism
☐ **REQ-003-014**: System shall update all open positions when new price data arrives:
```rust
pub fn update_prices(
    &mut self,
    prices: HashMap<String, f64>,
    funding_rates: Option<HashMap<String, f64>>,
) {
    // Update price cache
    self.current_prices.extend(prices.clone());

    if let Some(rates) = funding_rates {
        self.funding_rates.extend(rates);
    }

    // Update each open trade
    for trade_id in &self.open_trade_ids.clone() {
        if let Some(trade) = self.trades.get_mut(trade_id) {
            if let Some(&current_price) = prices.get(&trade.symbol) {
                let funding_rate = self.funding_rates.get(&trade.symbol).copied();
                trade.update_with_price(current_price, funding_rate);
            }
        }
    }

    // Recalculate portfolio values
    self.update_portfolio_values();
    self.last_updated = Utc::now();
}
```

☐ **REQ-003-015**: System shall check for automatic trade closures after each price update:
- Stop-loss triggered
- Take-profit triggered
- Liquidation threshold reached

##### 3.3.2.6 Position Closing
☐ **REQ-003-016**: System shall close positions via:
- User manual close request
- Stop-loss trigger
- Take-profit trigger
- Liquidation event
- Opposite order (close by reversal)

☐ **REQ-003-017**: System shall execute position close with:
```rust
pub fn close_trade(
    &mut self,
    trade_id: &str,
    exit_price: f64,
    close_reason: CloseReason,
) -> Result<()> {
    let trade = self.trades.get_mut(trade_id)?;

    // Calculate exit fees
    let exit_notional = trade.quantity * exit_price;
    let exit_fees = exit_notional * fee_rate;

    // Close trade and calculate realized PnL
    trade.close(exit_price, close_reason, exit_fees)?;

    // Update portfolio
    if let Some(realized_pnl) = trade.realized_pnl {
        self.cash_balance += trade.initial_margin + realized_pnl;
    }

    self.margin_used -= trade.initial_margin;

    // Move from open to closed
    self.open_trade_ids.retain(|id| id != trade_id);
    self.closed_trade_ids.push(trade_id.to_string());

    self.update_portfolio_values();
    Ok(())
}
```

☐ **REQ-003-018**: System shall calculate realized PnL on position close:
```rust
let raw_pnl = match trade.side {
    Long => trade.quantity * (exit_price - entry_price),
    Short => trade.quantity * (entry_price - exit_price),
};

let realized_pnl = raw_pnl - trading_fees - exit_fees - funding_fees;
trade.realized_pnl = Some(realized_pnl);
```

##### 3.3.2.7 Multi-Symbol Support
☐ **REQ-003-019**: System shall support simultaneous positions across multiple symbols

☐ **REQ-003-020**: System shall track positions grouped by symbol:
```rust
pub fn get_positions_by_symbol(&self) -> HashMap<String, Vec<&PaperTrade>> {
    let mut positions_map: HashMap<String, Vec<&PaperTrade>> = HashMap::new();

    for trade_id in &self.open_trade_ids {
        if let Some(trade) = self.trades.get(trade_id) {
            positions_map
                .entry(trade.symbol.clone())
                .or_insert_with(Vec::new)
                .push(trade);
        }
    }

    positions_map
}
```

☐ **REQ-003-021**: System shall calculate aggregate metrics per symbol:
- Total position size
- Average entry price
- Total unrealized PnL
- Total margin used

#### 3.3.3 Business Rules
- **BR-003-001**: Equity must be recalculated after every price update
- **BR-003-002**: Margin level below 100% triggers liquidation warning
- **BR-003-003**: Funding fees applied only to open positions
- **BR-003-004**: Position close must be atomic operation
- **BR-003-005**: Realized PnL only calculated upon position close
- **BR-003-006**: Maximum open positions per symbol: configurable (default 5)

#### 3.3.4 Acceptance Criteria
```gherkin
Scenario: Track open position with price update
  Given open long position:
    - Symbol: BTCUSDT
    - Quantity: 0.1 BTC
    - Entry price: $50,000
    - Leverage: 10x
    - Initial margin: $500
  When market price updates to $51,000
  Then position shows:
    ☐ Unrealized PnL = 0.1 × ($51,000 - $50,000) = $100
    ☐ ROI = $100 / $500 = 20%
    ☐ Portfolio equity = cash + $100
    ☐ Margin level = (equity / $500) × 100%
    ☐ Free margin = equity - $500
```

---

### FR-PAPER-TRADING-004: Paper Trading History & Analytics

**Priority:** HIGH
**Spec ID:** @spec:FR-PAPER-TRADING-004
**Related APIs:** GET `/api/v1/paper-trading/history`, GET `/api/v1/paper-trading/trades/{id}`

#### 3.4.1 Description
The system shall persist all paper trading activities and provide comprehensive trade history retrieval with filtering, sorting, and analytics capabilities.

#### 3.4.2 Detailed Requirements

##### 3.4.2.1 Trade History Storage
☐ **REQ-004-001**: System shall persist every trade to MongoDB with complete details:
```rust
TradeDocument {
    _id: ObjectId,
    user_id: String,
    portfolio_id: String,
    trade: PaperTrade,
    created_at: DateTime,
    updated_at: DateTime,
}
```

☐ **REQ-004-002**: System shall update trade document on:
- Trade creation (status = Open)
- Price updates (unrealized PnL changes)
- Trade closure (status = Closed)
- Funding fee application

☐ **REQ-004-003**: System shall create indexes for efficient queries:
```javascript
db.paper_trades.createIndex({ "user_id": 1, "portfolio_id": 1 })
db.paper_trades.createIndex({ "trade.symbol": 1, "trade.status": 1 })
db.paper_trades.createIndex({ "trade.open_time": -1 })
db.paper_trades.createIndex({ "trade.close_time": -1 })
```

##### 3.4.2.2 Trade History Retrieval
☐ **REQ-004-004**: System shall support trade history queries with filters:
- **Status**: Open, Closed, All
- **Symbol**: Specific symbol or all symbols
- **Date Range**: Start date to end date
- **Side**: Long, Short, Both
- **PnL Range**: Min/max realized PnL
- **Close Reason**: Manual, StopLoss, TakeProfit, Liquidation

☐ **REQ-004-005**: System shall support pagination:
```rust
pub struct TradeHistoryQuery {
    pub page: u32,
    pub page_size: u32,  // Max 100
    pub filters: TradeFilters,
    pub sort_by: SortField,
    pub sort_order: SortOrder,
}
```

☐ **REQ-004-006**: System shall support sorting by:
- Open time (ascending/descending)
- Close time (ascending/descending)
- Realized PnL (highest/lowest first)
- Trade duration (longest/shortest first)
- Symbol (alphabetical)

##### 3.4.2.3 Trade Detail Retrieval
☐ **REQ-004-007**: System shall provide detailed trade information by ID:
```rust
pub struct TradeDetail {
    pub trade: PaperTrade,
    pub entry_metadata: EntryMetadata,
    pub exit_metadata: Option<ExitMetadata>,
    pub price_history: Vec<PricePoint>,
    pub pnl_history: Vec<PnLPoint>,
    pub funding_history: Vec<FundingEvent>,
}
```

☐ **REQ-004-008**: System shall track price history during trade lifetime:
```rust
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
}
```

☐ **REQ-004-009**: System shall track PnL evolution:
```rust
pub struct PnLPoint {
    pub timestamp: DateTime<Utc>,
    pub unrealized_pnl: f64,
    pub pnl_percentage: f64,
}
```

##### 3.4.2.4 Daily Performance Snapshots
☐ **REQ-004-010**: System shall create daily portfolio snapshot at 00:00 UTC:
```rust
pub struct DailyPerformance {
    pub date: DateTime<Utc>,
    pub balance: f64,
    pub equity: f64,
    pub daily_pnl: f64,
    pub daily_pnl_percentage: f64,
    pub trades_executed: u32,
    pub winning_trades: u32,
    pub losing_trades: u32,
    pub total_volume: f64,
    pub max_drawdown: f64,
}
```

☐ **REQ-004-011**: System shall calculate daily PnL:
```rust
let daily_pnl = today_equity - yesterday_equity;
let daily_pnl_percentage = (daily_pnl / yesterday_equity) * 100.0;
```

☐ **REQ-004-012**: System shall maintain last 365 daily snapshots

##### 3.4.2.5 Trade Statistics
☐ **REQ-004-013**: System shall provide aggregate statistics:
```rust
pub struct TradeStatistics {
    pub total_trades: u64,
    pub open_trades: u64,
    pub closed_trades: u64,

    pub total_volume: f64,
    pub total_fees_paid: f64,
    pub total_funding_fees: f64,

    pub by_symbol: HashMap<String, SymbolStats>,
    pub by_side: HashMap<TradeSide, SideStats>,
    pub by_close_reason: HashMap<CloseReason, u64>,

    pub average_trade_duration_minutes: f64,
    pub longest_trade_duration_minutes: f64,
    pub shortest_trade_duration_minutes: f64,
}
```

☐ **REQ-004-014**: System shall calculate per-symbol statistics:
```rust
pub struct SymbolStats {
    pub symbol: String,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub total_pnl: f64,
    pub average_pnl: f64,
    pub win_rate: f64,
}
```

##### 3.4.2.6 Export Functionality
☐ **REQ-004-015**: System shall support trade history export in formats:
- JSON
- CSV
- Excel (XLSX)

☐ **REQ-004-016**: CSV export shall include columns:
```
Trade ID, Symbol, Side, Quantity, Leverage, Entry Price, Exit Price,
Entry Time, Exit Time, Duration (minutes), Realized PnL, PnL %,
Trading Fees, Funding Fees, Total Fees, Close Reason, Strategy Name
```

☐ **REQ-004-017**: System shall support date range filtering for exports

☐ **REQ-004-018**: System shall limit export size to 10,000 trades per request

#### 3.4.3 Business Rules
- **BR-004-001**: All trades must be persisted before execution completes
- **BR-004-002**: Trade history queries limited to user's own trades
- **BR-004-003**: Daily snapshots created at 00:00 UTC regardless of timezone
- **BR-004-004**: Exports include only closed trades unless explicitly requested

#### 3.4.4 Acceptance Criteria
```gherkin
Scenario: Retrieve trade history with filters
  Given portfolio has 100 closed trades
  When user queries trade history with:
    - Status: Closed
    - Symbol: BTCUSDT
    - Date range: Last 30 days
    - PnL > 0 (winners only)
    - Page: 1, Page size: 20
    - Sort by: Realized PnL (descending)
  Then system returns:
    ☐ Matching trades (max 20)
    ☐ Total count of matching trades
    ☐ Current page number
    ☐ Total pages available
    ☐ Trades sorted by PnL descending
    ☐ Each trade with complete details
```

---

### FR-PAPER-TRADING-005: Strategy Testing & Validation

**Priority:** HIGH
**Spec ID:** @spec:FR-PAPER-TRADING-005
**Related APIs:** POST `/api/v1/paper-trading/backtest`, GET `/api/v1/paper-trading/backtest/{id}`

#### 3.5.1 Description
The system shall provide comprehensive strategy testing capabilities including historical backtesting and forward testing on live data in paper mode.

#### 3.5.2 Detailed Requirements

##### 3.5.2.1 Historical Backtesting
☐ **REQ-005-001**: System shall support backtesting strategies on historical candlestick data

☐ **REQ-005-002**: System shall accept backtest configuration:
```rust
pub struct BacktestConfig {
    pub strategy_name: String,
    pub symbol: String,
    pub timeframe: String,  // "1m", "5m", "15m", "1h", "4h", "1d"
    pub start_date: DateTime<Utc>,
    pub end_date: DateTime<Utc>,
    pub initial_balance: f64,
    pub leverage: f64,
    pub position_size_pct: f64,  // % of portfolio per trade
    pub slippage_pct: f64,
    pub maker_fee: f64,
    pub taker_fee: f64,
    pub strategy_params: HashMap<String, serde_json::Value>,
}
```

☐ **REQ-005-003**: System shall fetch historical candlestick data from market data service

☐ **REQ-005-004**: System shall replay historical data chronologically:
```rust
for candle in historical_candles {
    // Update strategy with latest candle
    let signal = strategy.analyze(&candle_data).await?;

    // Execute trades based on signal
    if signal.should_trade() {
        execute_backtest_trade(&signal, &candle.close_price);
    }

    // Update existing positions
    update_open_positions(&candle.close_price);

    // Check for stop-loss / take-profit triggers
    check_exit_conditions(&candle);
}
```

☐ **REQ-005-005**: System shall simulate realistic order execution:
- Apply slippage to entry and exit prices
- Calculate fees based on maker/taker rates
- Respect position size limits
- Check margin requirements
- Simulate funding fees (if perpetual futures)

##### 3.5.2.2 Backtest Performance Metrics
☐ **REQ-005-006**: System shall calculate comprehensive backtest results:
```rust
pub struct BacktestResult {
    pub backtest_id: String,
    pub config: BacktestConfig,
    pub execution_time_ms: u64,

    // Portfolio metrics
    pub initial_balance: f64,
    pub final_balance: f64,
    pub total_pnl: f64,
    pub total_pnl_percentage: f64,

    // Trade metrics
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub win_rate: f64,

    pub average_win: f64,
    pub average_loss: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub profit_factor: f64,

    // Risk metrics
    pub max_drawdown: f64,
    pub max_drawdown_percentage: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub calmar_ratio: f64,

    // Exposure metrics
    pub total_days: u64,
    pub days_in_market: u64,
    pub market_exposure_pct: f64,
    pub average_trade_duration_minutes: f64,

    // Fee analysis
    pub total_fees_paid: f64,
    pub total_funding_fees: f64,

    // Equity curve
    pub equity_curve: Vec<EquityPoint>,

    // All trades
    pub trades: Vec<BacktestTrade>,
}
```

☐ **REQ-005-007**: System shall calculate Sharpe Ratio:
```rust
fn calculate_sharpe_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;
    let std_dev = calculate_std_deviation(returns);

    if std_dev == 0.0 {
        return 0.0;
    }

    // Annualized Sharpe Ratio (assuming daily returns)
    let excess_return = mean_return - risk_free_rate;
    (excess_return / std_dev) * (365.0_f64).sqrt()
}
```

☐ **REQ-005-008**: System shall calculate Sortino Ratio (downside deviation):
```rust
fn calculate_sortino_ratio(returns: &[f64], risk_free_rate: f64) -> f64 {
    let mean_return = returns.iter().sum::<f64>() / returns.len() as f64;

    // Calculate downside deviation (only negative returns)
    let negative_returns: Vec<f64> = returns
        .iter()
        .filter(|&&r| r < 0.0)
        .copied()
        .collect();

    let downside_dev = calculate_std_deviation(&negative_returns);

    if downside_dev == 0.0 {
        return 0.0;
    }

    let excess_return = mean_return - risk_free_rate;
    (excess_return / downside_dev) * (365.0_f64).sqrt()
}
```

☐ **REQ-005-009**: System shall calculate Maximum Drawdown:
```rust
fn calculate_max_drawdown(equity_curve: &[f64]) -> (f64, f64) {
    let mut peak = equity_curve[0];
    let mut max_dd = 0.0;
    let mut max_dd_pct = 0.0;

    for &equity in equity_curve {
        if equity > peak {
            peak = equity;
        }

        let drawdown = peak - equity;
        let drawdown_pct = (drawdown / peak) * 100.0;

        if drawdown > max_dd {
            max_dd = drawdown;
            max_dd_pct = drawdown_pct;
        }
    }

    (max_dd, max_dd_pct)
}
```

☐ **REQ-005-010**: System shall generate equity curve:
```rust
pub struct EquityPoint {
    pub timestamp: DateTime<Utc>,
    pub equity: f64,
    pub drawdown: f64,
    pub drawdown_percentage: f64,
}
```

##### 3.5.2.3 Forward Testing (Live Data)
☐ **REQ-005-011**: System shall support forward testing with real-time market data

☐ **REQ-005-012**: System shall execute strategy on live data in paper mode:
```rust
pub async fn run_forward_test(
    strategy: Box<dyn Strategy>,
    config: ForwardTestConfig,
) -> Result<ForwardTestHandle> {
    // Subscribe to real-time market data
    let market_stream = market_data_service.subscribe(&config.symbol).await?;

    // Create paper portfolio
    let portfolio = PaperPortfolio::new(config.initial_balance);

    // Process market data as it arrives
    while let Some(candle) = market_stream.next().await {
        let signal = strategy.analyze(&candle_data).await?;

        if signal.should_trade() {
            portfolio.execute_order(create_order_from_signal(&signal)).await?;
        }

        portfolio.update_prices(get_current_prices()).await?;
    }

    Ok(handle)
}
```

☐ **REQ-005-013**: System shall track forward test performance in real-time

☐ **REQ-005-014**: System shall allow starting/stopping forward tests

☐ **REQ-005-015**: System shall persist forward test state for resumption

##### 3.5.2.4 Strategy Optimization
☐ **REQ-005-016**: System shall support strategy parameter optimization:
```rust
pub struct OptimizationConfig {
    pub strategy_name: String,
    pub parameter_ranges: HashMap<String, ParameterRange>,
    pub optimization_metric: OptimizationMetric,
    pub backtest_config: BacktestConfig,
}

pub enum ParameterRange {
    Integer { min: i64, max: i64, step: i64 },
    Float { min: f64, max: f64, step: f64 },
    Discrete { values: Vec<serde_json::Value> },
}

pub enum OptimizationMetric {
    TotalReturn,
    SharpeRatio,
    SortinoRatio,
    ProfitFactor,
    MaxDrawdown,
    WinRate,
}
```

☐ **REQ-005-017**: System shall use grid search for parameter optimization:
```rust
pub async fn optimize_strategy(
    config: OptimizationConfig,
) -> Result<OptimizationResult> {
    let parameter_combinations = generate_parameter_grid(&config.parameter_ranges);
    let mut results = Vec::new();

    for params in parameter_combinations {
        let mut backtest_config = config.backtest_config.clone();
        backtest_config.strategy_params = params.clone();

        let result = run_backtest(backtest_config).await?;
        results.push((params, result));
    }

    // Sort by optimization metric
    results.sort_by(|a, b| {
        compare_by_metric(&a.1, &b.1, &config.optimization_metric)
    });

    Ok(OptimizationResult {
        best_parameters: results[0].0.clone(),
        best_result: results[0].1.clone(),
        all_results: results,
    })
}
```

☐ **REQ-005-018**: System shall limit optimization to prevent overfitting:
- Maximum parameter combinations: 10,000
- Minimum trades required: 30
- Walk-forward validation support

##### 3.5.2.5 Strategy Comparison
☐ **REQ-005-019**: System shall support comparing multiple strategies side-by-side

☐ **REQ-005-020**: System shall provide comparison metrics:
```rust
pub struct StrategyComparison {
    pub strategies: Vec<StrategyResult>,
    pub comparison_metrics: ComparisonMetrics,
}

pub struct ComparisonMetrics {
    pub best_total_return: String,  // Strategy name
    pub best_sharpe_ratio: String,
    pub best_win_rate: String,
    pub lowest_drawdown: String,
    pub most_consistent: String,  // Lowest std dev
}
```

#### 3.5.3 Business Rules
- **BR-005-001**: Backtest must use historical data only (no lookahead bias)
- **BR-005-002**: Slippage and fees must be applied consistently
- **BR-005-003**: Forward tests run in paper mode only
- **BR-005-004**: Optimization requires minimum 30 trades for validity
- **BR-005-005**: Backtest results cached for 24 hours

#### 3.5.4 Acceptance Criteria
```gherkin
Scenario: Run strategy backtest
  Given RSI strategy with default parameters
  And historical data for BTCUSDT (1h) from 2024-01-01 to 2024-12-31
  And initial balance $10,000
  When user runs backtest
  Then system returns results with:
    ☐ Total trades executed
    ☐ Win rate percentage
    ☐ Total PnL and PnL %
    ☐ Maximum drawdown
    ☐ Sharpe ratio
    ☐ Sortino ratio
    ☐ Equity curve (daily)
    ☐ All trades with entry/exit details
    ☐ Execution time < 30 seconds
```

---

### FR-PAPER-TRADING-006: Paper vs Live Comparison

**Priority:** MEDIUM
**Spec ID:** @spec:FR-PAPER-TRADING-006
**Related APIs:** GET `/api/v1/paper-trading/comparison`

#### 3.6.1 Description
The system shall provide side-by-side comparison of paper trading performance versus live trading performance to validate strategy effectiveness before committing real capital.

#### 3.6.2 Detailed Requirements

##### 3.6.2.1 Parallel Execution Tracking
☐ **REQ-006-001**: System shall support running same strategy in both paper and live modes simultaneously

☐ **REQ-006-002**: System shall tag all trades with execution mode:
```rust
pub enum ExecutionMode {
    Paper,
    Live,
}

pub struct TradeModeTag {
    pub mode: ExecutionMode,
    pub linked_trade_id: Option<String>,  // Corresponding trade in other mode
}
```

☐ **REQ-006-003**: System shall link corresponding paper and live trades for comparison

##### 3.6.2.2 Performance Comparison Metrics
☐ **REQ-006-004**: System shall calculate comparison metrics:
```rust
pub struct PaperVsLiveComparison {
    pub time_period: DateRange,
    pub paper_metrics: PerformanceMetrics,
    pub live_metrics: PerformanceMetrics,
    pub variance_analysis: VarianceAnalysis,
}

pub struct VarianceAnalysis {
    pub pnl_difference: f64,
    pub pnl_difference_percentage: f64,
    pub win_rate_difference: f64,
    pub average_slippage_difference: f64,
    pub average_fee_difference: f64,
    pub execution_quality_score: f64,
}
```

☐ **REQ-006-005**: System shall identify reasons for discrepancies:
- Slippage differences (simulated vs actual)
- Fee differences (estimated vs actual)
- Execution timing differences
- Partial fills (live) vs full fills (paper)
- Price feed differences

##### 3.6.2.3 Execution Quality Analysis
☐ **REQ-006-006**: System shall calculate execution quality score:
```rust
fn calculate_execution_quality(
    paper_trades: &[PaperTrade],
    live_trades: &[LiveTrade],
) -> f64 {
    let price_difference_score = calculate_price_variance(paper_trades, live_trades);
    let timing_score = calculate_timing_variance(paper_trades, live_trades);
    let fill_quality_score = calculate_fill_quality(paper_trades, live_trades);

    // Weighted average
    (price_difference_score * 0.5) +
    (timing_score * 0.3) +
    (fill_quality_score * 0.2)
}
```

☐ **REQ-006-007**: System shall track and report:
- Average execution price difference
- Average execution time difference
- Partial fill percentage (live only)
- Order rejection rate (live only)

##### 3.6.2.4 Validation Reports
☐ **REQ-006-008**: System shall generate validation report:
```rust
pub struct ValidationReport {
    pub summary: ValidationSummary,
    pub readiness_score: f64,  // 0-100
    pub recommendations: Vec<String>,
    pub risk_warnings: Vec<String>,
}

pub struct ValidationSummary {
    pub paper_win_rate: f64,
    pub paper_profit_factor: f64,
    pub paper_sharpe_ratio: f64,
    pub minimum_trades_met: bool,  // >= 30 trades
    pub consistent_performance: bool,  // Low variance
    pub acceptable_drawdown: bool,  // < 20%
}
```

☐ **REQ-006-009**: System shall calculate readiness score for going live:
```rust
fn calculate_readiness_score(validation: &ValidationSummary) -> f64 {
    let mut score = 0.0;

    // Win rate component (0-25 points)
    if validation.paper_win_rate >= 0.6 {
        score += 25.0;
    } else if validation.paper_win_rate >= 0.5 {
        score += 15.0;
    }

    // Profit factor component (0-25 points)
    if validation.paper_profit_factor >= 2.0 {
        score += 25.0;
    } else if validation.paper_profit_factor >= 1.5 {
        score += 15.0;
    }

    // Sharpe ratio component (0-25 points)
    if validation.paper_sharpe_ratio >= 2.0 {
        score += 25.0;
    } else if validation.paper_sharpe_ratio >= 1.0 {
        score += 15.0;
    }

    // Risk checks (0-25 points)
    if validation.minimum_trades_met { score += 10.0; }
    if validation.consistent_performance { score += 10.0; }
    if validation.acceptable_drawdown { score += 5.0; }

    score
}
```

☐ **REQ-006-010**: System shall provide recommendations based on score:
- Score >= 80: "Ready for live trading"
- Score 60-79: "Consider additional testing"
- Score 40-59: "Requires optimization"
- Score < 40: "Not recommended for live trading"

#### 3.6.3 Business Rules
- **BR-006-001**: Minimum 30 paper trades required before live trading
- **BR-006-002**: Comparison only valid for same time period
- **BR-006-003**: Readiness score calculated using last 30 days data

#### 3.6.4 Acceptance Criteria
```gherkin
Scenario: Compare paper vs live performance
  Given strategy ran in paper mode for 30 days
  And same strategy ran in live mode for 30 days
  When user requests comparison report
  Then system provides:
    ☐ Side-by-side performance metrics
    ☐ PnL variance analysis
    ☐ Execution quality score
    ☐ Readiness score for live trading
    ☐ Specific recommendations
    ☐ Risk warnings if applicable
```

---

### FR-PAPER-TRADING-007: Risk Management Simulation

**Priority:** HIGH
**Spec ID:** @spec:FR-PAPER-TRADING-007
**Related APIs:** POST `/api/v1/paper-trading/risk-check`, PATCH `/api/v1/paper-trading/risk-settings`

#### 3.7.1 Description
The system shall simulate comprehensive risk management including position sizing, stop-loss/take-profit, liquidation, and portfolio-level risk controls.

#### 3.7.2 Detailed Requirements

##### 3.7.2.1 Position Sizing Rules
☐ **REQ-007-001**: System shall enforce maximum position size per trade:
```rust
pub struct RiskSettings {
    pub max_position_size_pct: f64,  // % of portfolio
    pub max_leverage: f64,
    pub max_open_positions: usize,
    pub max_positions_per_symbol: usize,
    pub max_portfolio_risk_pct: f64,
}
```

☐ **REQ-007-002**: System shall calculate position size:
```rust
fn calculate_position_size(
    portfolio_equity: f64,
    risk_pct: f64,
    entry_price: f64,
    stop_loss_price: f64,
) -> f64 {
    let risk_amount = portfolio_equity * (risk_pct / 100.0);
    let price_risk = (entry_price - stop_loss_price).abs();
    let position_size = risk_amount / price_risk;

    // Apply max position size limit
    let max_size = portfolio_equity * (max_position_size_pct / 100.0) / entry_price;
    position_size.min(max_size)
}
```

☐ **REQ-007-003**: System shall reject orders exceeding position limits

##### 3.7.2.2 Stop-Loss Simulation
☐ **REQ-007-004**: System shall support stop-loss types:
- **Fixed Price**: Trigger at specific price level
- **Percentage**: Trigger at X% loss from entry
- **ATR-based**: Dynamic based on volatility
- **Trailing Stop**: Moves with favorable price movement

☐ **REQ-007-005**: System shall check stop-loss on every price update:
```rust
pub fn check_stop_loss_trigger(
    &self,
    current_price: f64,
) -> bool {
    if let Some(stop_loss) = self.stop_loss {
        match self.side {
            TradeSide::Long => current_price <= stop_loss,
            TradeSide::Short => current_price >= stop_loss,
        }
    } else {
        false
    }
}
```

☐ **REQ-007-006**: System shall execute stop-loss as market order with slippage:
```rust
let stop_loss_executed_price = match trade.side {
    Long => stop_loss * (1.0 - slippage_pct),  // Worse price
    Short => stop_loss * (1.0 + slippage_pct),
};
```

☐ **REQ-007-007**: System shall implement trailing stop-loss:
```rust
fn update_trailing_stop(
    &mut self,
    current_price: f64,
    trail_pct: f64,
) {
    match self.side {
        TradeSide::Long => {
            // For long, stop moves up with price
            let new_stop = current_price * (1.0 - trail_pct / 100.0);
            if let Some(current_stop) = self.stop_loss {
                if new_stop > current_stop {
                    self.stop_loss = Some(new_stop);
                }
            } else {
                self.stop_loss = Some(new_stop);
            }
        },
        TradeSide::Short => {
            // For short, stop moves down with price
            let new_stop = current_price * (1.0 + trail_pct / 100.0);
            if let Some(current_stop) = self.stop_loss {
                if new_stop < current_stop {
                    self.stop_loss = Some(new_stop);
                }
            } else {
                self.stop_loss = Some(new_stop);
            }
        },
    }
}
```

##### 3.7.2.3 Take-Profit Simulation
☐ **REQ-007-008**: System shall support take-profit types:
- **Fixed Price**: Close at specific price level
- **Percentage**: Close at X% profit from entry
- **Risk-Reward Ratio**: Take profit at multiple of risk
- **Partial Take-Profit**: Close portion of position at levels

☐ **REQ-007-009**: System shall check take-profit on every price update:
```rust
pub fn check_take_profit_trigger(
    &self,
    current_price: f64,
) -> bool {
    if let Some(take_profit) = self.take_profit {
        match self.side {
            TradeSide::Long => current_price >= take_profit,
            TradeSide::Short => current_price <= take_profit,
        }
    } else {
        false
    }
}
```

☐ **REQ-007-010**: System shall support multiple take-profit levels:
```rust
pub struct PartialTakeProfitConfig {
    pub levels: Vec<TakeProfitLevel>,
}

pub struct TakeProfitLevel {
    pub price: f64,
    pub close_percentage: f64,  // % of position to close
}

// Example: Close 50% at 2% profit, remaining 50% at 5% profit
```

##### 3.7.2.4 Liquidation Simulation
☐ **REQ-007-011**: System shall calculate liquidation price:
```rust
fn calculate_liquidation_price(
    entry_price: f64,
    leverage: f64,
    side: TradeSide,
    maintenance_margin_rate: f64,  // Default 0.4% for Binance
) -> f64 {
    match side {
        TradeSide::Long => {
            // Liquidation when loss = initial margin - maintenance margin
            let liquidation_drop = (1.0 / leverage) - maintenance_margin_rate;
            entry_price * (1.0 - liquidation_drop)
        },
        TradeSide::Short => {
            // Liquidation when loss = initial margin - maintenance margin
            let liquidation_rise = (1.0 / leverage) - maintenance_margin_rate;
            entry_price * (1.0 + liquidation_rise)
        },
    }
}
```

☐ **REQ-007-012**: System shall check for liquidation on every price update:
```rust
pub fn check_liquidation(&self, current_price: f64) -> bool {
    let maintenance_margin = self.initial_margin * 0.4;  // 40% for Binance
    let current_equity = self.initial_margin + self.unrealized_pnl;

    current_equity < maintenance_margin
}
```

☐ **REQ-007-013**: System shall execute liquidation when margin level < 100%:
```rust
if portfolio.margin_level < 100.0 {
    // Liquidate positions starting with largest loss
    let mut positions = get_open_positions_sorted_by_loss();

    while portfolio.margin_level < 100.0 && !positions.is_empty() {
        let worst_position = positions.remove(0);
        liquidate_position(&worst_position, CloseReason::Liquidation);
    }
}
```

☐ **REQ-007-014**: System shall apply liquidation penalties:
- Close at worse price (additional slippage)
- Liquidation fee: 0.5% of position size
- Entire initial margin lost if position underwater

##### 3.7.2.5 Portfolio-Level Risk Controls
☐ **REQ-007-015**: System shall enforce portfolio-level limits:
- Maximum total margin used
- Maximum portfolio risk percentage
- Maximum correlated positions
- Daily loss limit

☐ **REQ-007-016**: System shall calculate portfolio risk:
```rust
fn calculate_portfolio_risk(&self) -> f64 {
    let mut total_risk = 0.0;

    for trade in &self.open_trades {
        if let Some(stop_loss) = trade.stop_loss {
            let risk = (trade.entry_price - stop_loss).abs() * trade.quantity;
            total_risk += risk;
        }
    }

    (total_risk / self.equity) * 100.0
}
```

☐ **REQ-007-017**: System shall implement daily loss limit:
```rust
pub struct DailyLossLimit {
    pub limit_amount: f64,
    pub limit_percentage: f64,
    pub reset_time_utc: String,  // "00:00"
}

fn check_daily_loss_limit(&self) -> Result<()> {
    let today_pnl = self.calculate_daily_pnl();

    if today_pnl < -self.daily_loss_limit.limit_amount {
        return Err("Daily loss limit reached");
    }

    let loss_pct = (today_pnl / self.equity) * 100.0;
    if loss_pct < -self.daily_loss_limit.limit_percentage {
        return Err("Daily loss percentage limit reached");
    }

    Ok(())
}
```

☐ **REQ-007-018**: System shall block new trades when daily loss limit hit

##### 3.7.2.6 Risk Warnings and Alerts
☐ **REQ-007-019**: System shall generate risk warnings:
- Margin level < 150% (warning)
- Margin level < 120% (critical warning)
- Daily loss > 5% (warning)
- Daily loss > 10% (critical warning)
- Portfolio risk > 50% (warning)

☐ **REQ-007-020**: System shall log all risk events to audit trail

#### 3.7.3 Business Rules
- **BR-007-001**: Stop-loss always executed as market order
- **BR-007-002**: Liquidation has priority over other order types
- **BR-007-003**: Daily loss limit resets at 00:00 UTC
- **BR-007-004**: Margin level calculated in real-time
- **BR-007-005**: Trailing stop only moves in favorable direction

#### 3.7.4 Acceptance Criteria
```gherkin
Scenario: Stop-loss execution
  Given open long position:
    - Entry price: $50,000
    - Stop-loss: $49,000
    - Quantity: 0.1 BTC
  When market price drops to $49,000
  Then system:
    ☐ Triggers stop-loss
    ☐ Executes as market order
    ☐ Applies slippage (execution at $48,950)
    ☐ Closes position
    ☐ Calculates realized loss
    ☐ Updates portfolio cash balance
    ☐ Logs close reason as "StopLoss"
```

---

### FR-PAPER-TRADING-008: Performance Metrics Calculation

**Priority:** HIGH
**Spec ID:** @spec:FR-PAPER-TRADING-008
**Related APIs:** GET `/api/v1/paper-trading/metrics`

#### 3.8.1 Description
The system shall calculate comprehensive performance metrics to evaluate trading strategy effectiveness, including profitability, risk-adjusted returns, and trading efficiency.

#### 3.8.2 Detailed Requirements

##### 3.8.2.1 Profitability Metrics
☐ **REQ-008-001**: System shall calculate total PnL:
```rust
let total_realized_pnl: f64 = closed_trades
    .iter()
    .filter_map(|t| t.realized_pnl)
    .sum();

let total_unrealized_pnl: f64 = open_trades
    .iter()
    .map(|t| t.unrealized_pnl)
    .sum();

let total_pnl = total_realized_pnl + total_unrealized_pnl;
let total_pnl_percentage = (total_pnl / initial_balance) * 100.0;
```

☐ **REQ-008-002**: System shall calculate win/loss metrics:
```rust
let winning_trades: Vec<_> = closed_trades
    .iter()
    .filter(|t| t.realized_pnl.unwrap_or(0.0) > 0.0)
    .collect();

let losing_trades: Vec<_> = closed_trades
    .iter()
    .filter(|t| t.realized_pnl.unwrap_or(0.0) < 0.0)
    .collect();

let win_rate = (winning_trades.len() as f64 / closed_trades.len() as f64) * 100.0;

let average_win = winning_trades
    .iter()
    .filter_map(|t| t.realized_pnl)
    .sum::<f64>() / winning_trades.len() as f64;

let average_loss = losing_trades
    .iter()
    .filter_map(|t| t.realized_pnl)
    .sum::<f64>() / losing_trades.len() as f64;
```

☐ **REQ-008-003**: System shall calculate profit factor:
```rust
let gross_profit: f64 = winning_trades
    .iter()
    .filter_map(|t| t.realized_pnl)
    .sum();

let gross_loss: f64 = losing_trades
    .iter()
    .filter_map(|t| t.realized_pnl)
    .map(|pnl| pnl.abs())
    .sum();

let profit_factor = if gross_loss > 0.0 {
    gross_profit / gross_loss
} else {
    0.0  // No losses yet
};
```

☐ **REQ-008-004**: System shall identify largest wins/losses:
```rust
let largest_win = winning_trades
    .iter()
    .filter_map(|t| t.realized_pnl)
    .max_by(|a, b| a.partial_cmp(b).unwrap())
    .unwrap_or(0.0);

let largest_loss = losing_trades
    .iter()
    .filter_map(|t| t.realized_pnl)
    .min_by(|a, b| a.partial_cmp(b).unwrap())
    .unwrap_or(0.0);
```

##### 3.8.2.2 Drawdown Metrics
☐ **REQ-008-005**: System shall calculate maximum drawdown:
```rust
fn calculate_max_drawdown(&self) -> (f64, f64) {
    let mut peak_equity = self.initial_balance;
    let mut max_dd = 0.0;
    let mut max_dd_pct = 0.0;

    for snapshot in &self.daily_performance {
        if snapshot.equity > peak_equity {
            peak_equity = snapshot.equity;
        }

        let drawdown = peak_equity - snapshot.equity;
        let drawdown_pct = (drawdown / peak_equity) * 100.0;

        if drawdown > max_dd {
            max_dd = drawdown;
            max_dd_pct = drawdown_pct;
        }
    }

    (max_dd, max_dd_pct)
}
```

☐ **REQ-008-006**: System shall calculate current drawdown:
```rust
let peak_equity = self.daily_performance
    .iter()
    .map(|s| s.equity)
    .max_by(|a, b| a.partial_cmp(b).unwrap())
    .unwrap_or(self.initial_balance);

let current_drawdown = peak_equity - self.equity;
let current_drawdown_pct = (current_drawdown / peak_equity) * 100.0;
```

##### 3.8.2.3 Risk-Adjusted Returns
☐ **REQ-008-007**: System shall calculate Sharpe Ratio (annualized):
```rust
fn calculate_sharpe_ratio(&self, risk_free_rate: f64) -> f64 {
    if self.daily_performance.len() < 2 {
        return 0.0;
    }

    // Calculate daily returns
    let daily_returns: Vec<f64> = self.daily_performance
        .windows(2)
        .map(|w| {
            (w[1].equity - w[0].equity) / w[0].equity
        })
        .collect();

    let mean_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;
    let std_dev = calculate_std_deviation(&daily_returns);

    if std_dev == 0.0 {
        return 0.0;
    }

    // Annualized Sharpe Ratio
    let daily_rf = risk_free_rate / 365.0;
    ((mean_return - daily_rf) / std_dev) * (365.0_f64).sqrt()
}
```

☐ **REQ-008-008**: System shall calculate Sortino Ratio:
```rust
fn calculate_sortino_ratio(&self, risk_free_rate: f64) -> f64 {
    if self.daily_performance.len() < 2 {
        return 0.0;
    }

    let daily_returns: Vec<f64> = self.daily_performance
        .windows(2)
        .map(|w| (w[1].equity - w[0].equity) / w[0].equity)
        .collect();

    let mean_return = daily_returns.iter().sum::<f64>() / daily_returns.len() as f64;

    // Downside deviation (only negative returns)
    let negative_returns: Vec<f64> = daily_returns
        .iter()
        .filter(|&&r| r < 0.0)
        .copied()
        .collect();

    let downside_dev = if !negative_returns.is_empty() {
        let variance = negative_returns
            .iter()
            .map(|r| r.powi(2))
            .sum::<f64>() / negative_returns.len() as f64;
        variance.sqrt()
    } else {
        return f64::INFINITY;  // No downside risk
    };

    let daily_rf = risk_free_rate / 365.0;
    ((mean_return - daily_rf) / downside_dev) * (365.0_f64).sqrt()
}
```

☐ **REQ-008-009**: System shall calculate Calmar Ratio:
```rust
fn calculate_calmar_ratio(&self) -> f64 {
    let (max_dd, _) = self.calculate_max_drawdown();

    if max_dd == 0.0 {
        return 0.0;
    }

    let total_days = (Utc::now() - self.created_at).num_days() as f64;
    let annual_return = (self.total_pnl / self.initial_balance) * (365.0 / total_days);

    annual_return / (max_dd / self.initial_balance)
}
```

☐ **REQ-008-010**: System shall calculate Recovery Factor:
```rust
fn calculate_recovery_factor(&self) -> f64 {
    let (max_dd, _) = self.calculate_max_drawdown();

    if max_dd == 0.0 {
        return 0.0;
    }

    self.total_pnl / max_dd
}
```

##### 3.8.2.4 Trading Efficiency Metrics
☐ **REQ-008-011**: System shall calculate average trade duration:
```rust
fn calculate_average_trade_duration(&self) -> f64 {
    let durations: Vec<f64> = self.closed_trades
        .iter()
        .filter_map(|t| {
            t.close_time.map(|close| {
                (close - t.open_time).num_minutes() as f64
            })
        })
        .collect();

    if durations.is_empty() {
        return 0.0;
    }

    durations.iter().sum::<f64>() / durations.len() as f64
}
```

☐ **REQ-008-012**: System shall track consecutive wins/losses:
```rust
fn calculate_streaks(&self) -> (u64, u64, i64) {
    let mut max_consecutive_wins = 0u64;
    let mut max_consecutive_losses = 0u64;
    let mut current_streak = 0i64;

    let mut temp_wins = 0u64;
    let mut temp_losses = 0u64;

    for trade in &self.closed_trades {
        if let Some(pnl) = trade.realized_pnl {
            if pnl > 0.0 {
                temp_wins += 1;
                temp_losses = 0;
                current_streak += 1;
            } else if pnl < 0.0 {
                temp_losses += 1;
                temp_wins = 0;
                current_streak -= 1;
            }

            max_consecutive_wins = max_consecutive_wins.max(temp_wins);
            max_consecutive_losses = max_consecutive_losses.max(temp_losses);
        }
    }

    (max_consecutive_wins, max_consecutive_losses, current_streak)
}
```

☐ **REQ-008-013**: System shall calculate fee impact:
```rust
let total_fees = total_trading_fees + total_funding_fees;
let fee_percentage = (total_fees / initial_balance) * 100.0;
let pnl_without_fees = total_pnl + total_fees;
let fee_impact = (total_fees / pnl_without_fees.abs()) * 100.0;
```

##### 3.8.2.5 Exposure Metrics
☐ **REQ-008-014**: System shall calculate market exposure:
```rust
fn calculate_market_exposure(&self) -> f64 {
    let total_days = (Utc::now() - self.created_at).num_days() as f64;

    if total_days == 0.0 {
        return 0.0;
    }

    // Calculate total time in market across all trades
    let total_minutes_in_market: f64 = self.closed_trades
        .iter()
        .filter_map(|t| {
            t.close_time.map(|close| {
                (close - t.open_time).num_minutes() as f64
            })
        })
        .sum();

    let total_minutes = total_days * 24.0 * 60.0;
    (total_minutes_in_market / total_minutes) * 100.0
}
```

☐ **REQ-008-015**: System shall calculate average leverage used:
```rust
let avg_leverage = if !self.closed_trades.is_empty() {
    self.closed_trades
        .iter()
        .map(|t| t.leverage)
        .sum::<f64>() / self.closed_trades.len() as f64
} else {
    0.0
};
```

##### 3.8.2.6 Metrics Caching and Updates
☐ **REQ-008-016**: System shall cache metrics and update on:
- New trade opened
- Trade closed
- Price update (for unrealized PnL)
- Daily snapshot creation

☐ **REQ-008-017**: System shall timestamp all metric calculations

☐ **REQ-008-018**: System shall expose metrics via API endpoint with:
- Current metrics
- Historical metrics (daily/weekly/monthly)
- Comparison to previous period
- Trend indicators (improving/declining)

#### 3.8.3 Business Rules
- **BR-008-001**: Metrics calculated only on closed trades (except unrealized PnL)
- **BR-008-002**: Sharpe/Sortino use 2% annual risk-free rate (configurable)
- **BR-008-003**: Minimum 10 trades required for statistical significance
- **BR-008-004**: Metrics cached for 5 minutes to reduce computation
- **BR-008-005**: Drawdown always measured from peak equity

#### 3.8.4 Acceptance Criteria
```gherkin
Scenario: Calculate comprehensive metrics
  Given portfolio with:
    - Initial balance: $10,000
    - Current equity: $12,500
    - 50 closed trades (30 wins, 20 losses)
    - Max equity peak: $13,000
  When user requests performance metrics
  Then system returns:
    ☐ Total PnL = $2,500
    ☐ Total PnL % = 25%
    ☐ Win rate = 60%
    ☐ Profit factor = (gross wins / gross losses)
    ☐ Max drawdown = $500 (from peak $13,000)
    ☐ Max drawdown % = 3.85%
    ☐ Current drawdown = $500
    ☐ Sharpe ratio (calculated)
    ☐ Sortino ratio (calculated)
    ☐ Calmar ratio (calculated)
    ☐ Average trade duration
    ☐ Max consecutive wins
    ☐ Max consecutive losses
    ☐ Total fees paid
    ☐ All metrics with timestamps
```

---

## 4. Data Models

### 4.1 PaperPortfolio
```rust
pub struct PaperPortfolio {
    pub initial_balance: f64,
    pub cash_balance: f64,
    pub equity: f64,
    pub margin_used: f64,
    pub free_margin: f64,
    pub margin_level: f64,
    pub trades: HashMap<String, PaperTrade>,
    pub open_trade_ids: Vec<String>,
    pub closed_trade_ids: Vec<String>,
    pub current_prices: HashMap<String, f64>,
    pub funding_rates: HashMap<String, f64>,
    pub created_at: DateTime<Utc>,
    pub last_updated: DateTime<Utc>,
    pub metrics: PortfolioMetrics,
    pub daily_performance: Vec<DailyPerformance>,
}
```

### 4.2 PaperTrade
```rust
pub struct PaperTrade {
    pub id: String,
    pub symbol: String,
    pub side: TradeSide,
    pub quantity: f64,
    pub leverage: f64,
    pub entry_price: f64,
    pub exit_price: Option<f64>,
    pub unrealized_pnl: f64,
    pub realized_pnl: Option<f64>,
    pub trading_fees: f64,
    pub funding_fees: f64,
    pub initial_margin: f64,
    pub maintenance_margin: f64,
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub status: TradeStatus,
    pub open_time: DateTime<Utc>,
    pub close_time: Option<DateTime<Utc>>,
    pub close_reason: Option<CloseReason>,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### 4.3 PortfolioMetrics
```rust
pub struct PortfolioMetrics {
    pub total_pnl: f64,
    pub total_pnl_percentage: f64,
    pub realized_pnl: f64,
    pub unrealized_pnl: f64,
    pub total_trades: u64,
    pub winning_trades: u64,
    pub losing_trades: u64,
    pub win_rate: f64,
    pub average_win: f64,
    pub average_loss: f64,
    pub profit_factor: f64,
    pub max_drawdown: f64,
    pub max_drawdown_percentage: f64,
    pub current_drawdown: f64,
    pub current_drawdown_percentage: f64,
    pub largest_win: f64,
    pub largest_loss: f64,
    pub average_trade_return: f64,
    pub return_std_deviation: f64,
    pub sharpe_ratio: f64,
    pub sortino_ratio: f64,
    pub max_consecutive_wins: u64,
    pub max_consecutive_losses: u64,
    pub current_streak: i64,
    pub average_trade_duration_minutes: f64,
    pub total_fees_paid: f64,
    pub total_funding_fees: f64,
    pub positions_by_symbol: HashMap<String, u32>,
    pub average_leverage: f64,
    pub risk_adjusted_return: f64,
    pub calmar_ratio: f64,
    pub recovery_factor: f64,
    pub calculated_at: DateTime<Utc>,
}
```

---

## 5. Business Rules

### 5.1 Portfolio Management
- **BR-PM-001**: Only one active paper portfolio per user
- **BR-PM-002**: Initial balance immutable after creation
- **BR-PM-003**: Portfolio equity recalculated on every price update
- **BR-PM-004**: All timestamps in UTC timezone

### 5.2 Order Execution
- **BR-OE-001**: Market orders execute immediately at simulated price
- **BR-OE-002**: Slippage applied unless explicitly disabled
- **BR-OE-003**: Fees calculated based on Binance fee structure (default)
- **BR-OE-004**: Orders rejected if insufficient margin

### 5.3 Risk Management
- **BR-RM-001**: Stop-loss always executes as market order
- **BR-RM-002**: Liquidation triggers when margin level < 100%
- **BR-RM-003**: Daily loss limit resets at 00:00 UTC
- **BR-RM-004**: Trailing stop moves only in favorable direction

### 5.4 Performance Metrics
- **BR-PM-001**: Metrics require minimum 10 trades for significance
- **BR-PM-002**: Sharpe/Sortino use 2% risk-free rate (configurable)
- **BR-PM-003**: Drawdown measured from equity peak
- **BR-PM-004**: Metrics cached for 5 minutes

---

## 6. Acceptance Criteria

### 6.1 Overall System
☐ **AC-001**: User can initialize paper portfolio with custom balance
☐ **AC-002**: User can place market, limit, and stop orders
☐ **AC-003**: Orders execute with realistic slippage and fees
☐ **AC-004**: Portfolio tracks real-time PnL and margin
☐ **AC-005**: Stop-loss and take-profit execute automatically
☐ **AC-006**: Liquidation simulated accurately
☐ **AC-007**: Funding fees applied every 8 hours
☐ **AC-008**: Trade history persisted to MongoDB
☐ **AC-009**: Comprehensive metrics calculated
☐ **AC-010**: User can backtest strategies on historical data
☐ **AC-011**: User can run forward tests with live data
☐ **AC-012**: User can compare paper vs live performance
☐ **AC-013**: All timestamps in UTC
☐ **AC-014**: API responses < 500ms (p95)
☐ **AC-015**: No data loss on service restart

---

## 7. Traceability

### 7.1 Related Specifications
- **API-SPEC-001**: Paper Trading API Endpoints
- **DATA-MODELS-001**: Portfolio and Trade Data Structures
- **BUSINESS-RULES-001**: Trading Business Logic
- **RISK-SPEC-001**: Risk Management Specification

### 7.2 Source Code References
- `rust-core-engine/src/paper_trading/engine.rs`: Main paper trading engine
- `rust-core-engine/src/paper_trading/portfolio.rs`: Portfolio management
- `rust-core-engine/src/paper_trading/trade.rs`: Trade execution
- `rust-core-engine/src/paper_trading/settings.rs`: Configuration
- `rust-core-engine/src/paper_trading/strategy_optimizer.rs`: Backtesting

### 7.3 API Endpoints
- `POST /api/v1/paper-trading/initialize`: Initialize portfolio
- `GET /api/v1/paper-trading/portfolio`: Get portfolio state
- `POST /api/v1/paper-trading/orders`: Place order
- `GET /api/v1/paper-trading/orders/{id}`: Get order details
- `GET /api/v1/paper-trading/history`: Get trade history
- `GET /api/v1/paper-trading/metrics`: Get performance metrics
- `POST /api/v1/paper-trading/backtest`: Run backtest
- `GET /api/v1/paper-trading/comparison`: Compare paper vs live

### 7.4 Test Coverage
- Unit tests: `rust-core-engine/tests/paper_trading_tests.rs`
- Integration tests: `rust-core-engine/tests/integration/paper_trading.rs`
- API tests: `tests/api/paper_trading_api_tests.rs`

---

**Document End**

**Revision History:**
- v1.0 (2025-10-10): Initial draft - Complete functional requirements for paper trading simulation
