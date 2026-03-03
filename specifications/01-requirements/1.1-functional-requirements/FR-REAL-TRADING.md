# Real Trading - Functional Requirements

**Spec ID**: FR-REAL
**Version**: 1.0
**Status**: ✅ Implemented
**Owner**: Trading Engine Team
**Last Updated**: 2026-02-06

---

## Overview

This specification defines functional requirements for the Real Trading Engine, which provides production-ready trading capabilities using the Binance API. The engine executes actual orders on Binance (testnet or mainnet) with comprehensive risk management, position tracking, and error handling.

**Key Capabilities**:
- Real order execution (market, limit, stop-loss, take-profit)
- Position tracking with real-time fills from ExecutionReports
- Pre-trade risk validation and daily loss limits
- Circuit breaker for error prevention
- User Data Stream integration for real-time updates
- Balance and order reconciliation

**Safety**:
- Defaults to testnet mode
- Configuration validation before trading
- Circuit breaker prevents cascade failures
- Execution lock prevents race conditions

---

## Requirements

### FR-REAL-001: Real Trading Module Initialization

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall initialize the real trading engine with proper configuration, API credentials, and dependencies.

**Acceptance Criteria**:
- [ ] Engine initializes with testnet or mainnet configuration
- [ ] Validates API credentials before starting
- [ ] Initializes Binance client with correct base URL
- [ ] Creates risk manager with position limits
- [ ] Sets up User Data Stream manager
- [ ] Defaults to testnet mode for safety
- [ ] Handles initialization failures gracefully
- [ ] Logs configuration details (excluding secrets)

**Code Location**:
- `rust-core-engine/src/real_trading/mod.rs:1-50`
- `rust-core-engine/src/binance/types.rs:331-400`
- `rust-core-engine/src/binance/client.rs:393-440`

**Test Cases**: TC-REAL-001, TC-REAL-002, TC-REAL-003

**Related Design**: COMP-RUST-TRADING.md, API-RUST-CORE.md

---

### FR-REAL-002: Market Order Execution

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall execute market buy/sell orders via Binance API with immediate execution at current market price.

**Acceptance Criteria**:
- [ ] Place market buy orders with quantity
- [ ] Place market sell orders with quantity
- [ ] Pre-trade risk validation before order submission
- [ ] Handle immediate fills (no partial fills for market orders)
- [ ] Return order response with order ID, status, fills
- [ ] Log order submission and response
- [ ] Handle API errors (insufficient balance, invalid symbol, rate limit)
- [ ] Prevent duplicate orders with execution lock

**Code Location**:
- `rust-core-engine/src/binance/client.rs:442-465` (place_market_order)
- `rust-core-engine/src/binance/types.rs:331-400` (SpotOrderRequest)

**Test Cases**: TC-REAL-010, TC-REAL-011, TC-REAL-012

**Related Design**: API-RUST-CORE.md, COMP-RUST-TRADING.md

---

### FR-REAL-003: Limit Order Execution

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall execute limit buy/sell orders at specified price with GTC (Good-Til-Cancelled) time-in-force.

**Acceptance Criteria**:
- [ ] Place limit buy orders with price and quantity
- [ ] Place limit sell orders with price and quantity
- [ ] Support GTC (Good-Til-Cancelled) time-in-force
- [ ] Track order status (NEW, PARTIALLY_FILLED, FILLED, CANCELED)
- [ ] Handle partial fills via ExecutionReports
- [ ] Return order response with order ID, status
- [ ] Log order submission
- [ ] Handle API errors

**Code Location**:
- `rust-core-engine/src/binance/client.rs:467-493` (place_limit_order)

**Test Cases**: TC-REAL-013, TC-REAL-014, TC-REAL-015

**Related Design**: API-RUST-CORE.md

---

### FR-REAL-004: Stop-Loss Order Execution

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall execute stop-loss orders to limit losses by selling at market when price hits stop price.

**Acceptance Criteria**:
- [ ] Place stop-loss orders with stop price
- [ ] Trigger at stop price, execute at market
- [ ] Track order status (NEW, TRIGGERED, FILLED)
- [ ] Handle immediate execution if price already below stop
- [ ] Return order response
- [ ] Log order submission
- [ ] Handle API errors

**Code Location**:
- `rust-core-engine/src/binance/client.rs:481-505` (place_stop_loss_order)

**Test Cases**: TC-REAL-016, TC-REAL-017, TC-REAL-018

**Related Design**: API-RUST-CORE.md, FR-RISK.md

---

### FR-REAL-005: Take-Profit Order Execution

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall execute take-profit orders to lock in profits by selling at market when price hits target.

**Acceptance Criteria**:
- [ ] Place take-profit orders with target price
- [ ] Trigger at target price, execute at market
- [ ] Track order status (NEW, TRIGGERED, FILLED)
- [ ] Handle immediate execution if price already above target
- [ ] Return order response
- [ ] Log order submission
- [ ] Handle API errors

**Code Location**:
- `rust-core-engine/src/binance/client.rs:495-520` (place_take_profit_order)

**Test Cases**: TC-REAL-019, TC-REAL-020, TC-REAL-021

**Related Design**: API-RUST-CORE.md

---

### FR-REAL-006: Cancel Order

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall cancel open orders by order ID or client order ID.

**Acceptance Criteria**:
- [ ] Cancel order by Binance order ID
- [ ] Cancel order by client order ID
- [ ] Return cancellation confirmation
- [ ] Handle already-filled orders gracefully
- [ ] Handle already-cancelled orders gracefully
- [ ] Log cancellation
- [ ] Handle API errors

**Code Location**:
- `rust-core-engine/src/binance/client.rs:518-550` (cancel_order)

**Test Cases**: TC-REAL-022, TC-REAL-023, TC-REAL-024

**Related Design**: API-RUST-CORE.md

---

### FR-REAL-007: Query Order Status

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall query order status from Binance API by order ID.

**Acceptance Criteria**:
- [ ] Query order by Binance order ID
- [ ] Return order details (status, fills, fees)
- [ ] Handle order not found
- [ ] Cache recent queries to reduce API calls
- [ ] Log query
- [ ] Handle API errors

**Code Location**:
- `rust-core-engine/src/binance/client.rs:583-615` (query_order)
- `rust-core-engine/src/binance/types.rs:596-700` (SpotOrderResponse)

**Test Cases**: TC-REAL-025, TC-REAL-026

**Related Design**: API-RUST-CORE.md

---

### FR-REAL-008: Get Account Balance

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall retrieve account balance from Binance API including free and locked amounts.

**Acceptance Criteria**:
- [ ] Retrieve balances for all assets
- [ ] Return free balance (available for trading)
- [ ] Return locked balance (in open orders)
- [ ] Filter zero balances for efficiency
- [ ] Cache balance for short duration to reduce API calls
- [ ] Handle API errors

**Code Location**:
- `rust-core-engine/src/binance/client.rs:205-240` (get_account_balance)
- `rust-core-engine/src/binance/types.rs:596-700`

**Test Cases**: TC-REAL-027, TC-REAL-028

**Related Design**: API-RUST-CORE.md

---

### FR-REAL-010: Real Order Tracking

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall track all orders in memory with status, fills, and timestamps.

**Acceptance Criteria**:
- [ ] Store order details (ID, symbol, side, type, quantity, price)
- [ ] Track order status (NEW, PARTIALLY_FILLED, FILLED, CANCELED, REJECTED)
- [ ] Track fills (fill price, quantity, commission)
- [ ] Calculate average fill price for partial fills
- [ ] Track order lifecycle (created_at, updated_at, filled_at)
- [ ] Thread-safe concurrent access (DashMap)
- [ ] Persist to database for audit trail

**Code Location**:
- `rust-core-engine/src/real_trading/order.rs:1-200`

**Test Cases**: TC-REAL-030, TC-REAL-031, TC-REAL-032

**Related Design**: DB-SCHEMA.md, COMP-RUST-TRADING.md

---

### FR-REAL-011: Real Position Tracking

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall track open positions with entry price, quantity, unrealized P&L, and exposure.

**Acceptance Criteria**:
- [ ] Open position on first fill
- [ ] Update position on subsequent fills (average entry price)
- [ ] Track position size (quantity)
- [ ] Calculate unrealized P&L based on current market price
- [ ] Track exposure (position value in quote currency)
- [ ] Close position when fully exited
- [ ] Thread-safe concurrent access
- [ ] Persist to database

**Code Location**:
- `rust-core-engine/src/real_trading/position.rs:1-300`
- `rust-core-engine/src/binance/types.rs:856-950`

**Test Cases**: TC-REAL-033, TC-REAL-034, TC-REAL-035

**Related Design**: DB-SCHEMA.md, COMP-RUST-TRADING.md

---

### FR-REAL-012: Real Trading Configuration

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall manage configuration for trading mode (testnet/mainnet), risk limits, and circuit breaker.

**Acceptance Criteria**:
- [ ] Support testnet mode (default)
- [ ] Support mainnet mode (requires explicit enable)
- [ ] Configure max open positions
- [ ] Configure daily loss limit percentage
- [ ] Configure circuit breaker threshold (consecutive errors)
- [ ] Configure circuit breaker cooldown duration
- [ ] Configure reconciliation interval
- [ ] Validate configuration on load

**Code Location**:
- `rust-core-engine/src/real_trading/config.rs:1-150`

**Test Cases**: TC-REAL-040, TC-REAL-041

**Related Design**: COMP-RUST-TRADING.md

---

### FR-REAL-013: Real Trading Engine Core

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall provide a real trading engine that orchestrates order execution, position tracking, risk management, and User Data Stream.

**Acceptance Criteria**:
- [ ] Start engine (initialize User Data Stream)
- [ ] Stop engine (cancel all open orders, disconnect stream)
- [ ] Place orders with risk validation
- [ ] Track orders in memory
- [ ] Track positions in memory
- [ ] Handle ExecutionReports from User Data Stream
- [ ] Broadcast events (OrderFilled, PositionOpened, BalanceUpdated)
- [ ] Run periodic reconciliation
- [ ] Circuit breaker on consecutive errors
- [ ] Execution lock prevents race conditions

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1-1200`

**Test Cases**: TC-REAL-050, TC-REAL-051, TC-REAL-052

**Related Design**: COMP-RUST-TRADING.md, ARCH-DATA-FLOW.md

---

### FR-REAL-030: User Data Stream Integration

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall integrate with Binance User Data Stream to receive real-time ExecutionReports, BalanceUpdates, and AccountUpdates.

**Acceptance Criteria**:
- [ ] Initialize User Data Stream with listen key
- [ ] Keep-alive listen key every 30 minutes
- [ ] Handle ExecutionReports (order fills)
- [ ] Handle BalanceUpdates (account balance changes)
- [ ] Handle OutboundAccountPosition (position updates)
- [ ] Reconnect on disconnect with exponential backoff
- [ ] Update orders and positions from stream events
- [ ] Broadcast events to subscribers

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1-100` (initialization)
- `rust-core-engine/src/binance/user_data_stream.rs` (stream manager)

**Test Cases**: TC-REAL-060, TC-REAL-061, TC-REAL-062

**Related Design**: API-WEBSOCKET.md, ARCH-DATA-FLOW.md

---

### FR-REAL-033: Balance Tracking from WebSocket

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall update account balance in real-time from BalanceUpdate events in User Data Stream.

**Acceptance Criteria**:
- [ ] Parse BalanceUpdate events
- [ ] Update cached balance (free, locked)
- [ ] Calculate balance change (delta)
- [ ] Broadcast BalanceUpdated event
- [ ] Handle multiple assets
- [ ] Thread-safe balance access

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1092-1150` (handle_balance_update)

**Test Cases**: TC-REAL-070, TC-REAL-071

**Related Design**: API-WEBSOCKET.md

---

### FR-REAL-034: Initial State Sync

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall sync initial state (open orders, positions, balance) from Binance API on engine start.

**Acceptance Criteria**:
- [ ] Fetch all open orders
- [ ] Reconstruct positions from open orders
- [ ] Fetch account balance
- [ ] Store in memory
- [ ] Log sync summary
- [ ] Handle API errors gracefully

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1209-1290` (sync_initial_state)

**Test Cases**: TC-REAL-080, TC-REAL-081

**Related Design**: COMP-RUST-TRADING.md

---

### FR-REAL-040: Real Trading Risk Manager

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall enforce risk controls before executing trades to prevent excessive losses and over-leveraging.

**Acceptance Criteria**:
- [ ] Validate pre-trade risk (position limits, daily loss)
- [ ] Check max open positions limit
- [ ] Check daily loss limit (track P&L since start of day)
- [ ] Check position size limits
- [ ] Calculate risk score (0-100, <50 = safe, >80 = dangerous)
- [ ] Reject orders exceeding risk thresholds
- [ ] Log risk checks
- [ ] Thread-safe risk calculation

**Code Location**:
- `rust-core-engine/src/real_trading/risk.rs:1-200`

**Test Cases**: TC-REAL-090, TC-REAL-091, TC-REAL-092

**Related Design**: FR-RISK.md, COMP-RUST-TRADING.md

---

### FR-REAL-041: Pre-Trade Risk Validation

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall validate all orders against risk rules before submission to exchange.

**Acceptance Criteria**:
- [ ] Check sufficient balance
- [ ] Check position size within limits
- [ ] Check daily loss not exceeded
- [ ] Check max open positions not exceeded
- [ ] Return validation result (pass/fail + reason)
- [ ] Log validation failures
- [ ] Prevent submission if validation fails

**Code Location**:
- `rust-core-engine/src/real_trading/risk.rs:1-50` (validate_trade)

**Test Cases**: TC-REAL-093, TC-REAL-094, TC-REAL-095

**Related Design**: FR-RISK.md

---

### FR-REAL-042: Risk-Based Position Sizing

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall calculate maximum safe position size based on account balance, risk parameters, and volatility.

**Acceptance Criteria**:
- [ ] Calculate max position size based on account balance percentage
- [ ] Adjust for symbol volatility (higher volatility = smaller position)
- [ ] Enforce min/max position size limits
- [ ] Consider existing positions (reduce size if many open)
- [ ] Return safe position size for order
- [ ] Log position sizing calculations

**Code Location**:
- `rust-core-engine/src/real_trading/risk.rs:50-150` (calculate_position_size)

**Test Cases**: TC-REAL-096, TC-REAL-097

**Related Design**: FR-RISK.md

---

### FR-REAL-051: Periodic Reconciliation

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall periodically reconcile in-memory state with Binance API to detect discrepancies.

**Acceptance Criteria**:
- [ ] Run reconciliation every N minutes (configurable)
- [ ] Compare orders: in-memory vs API
- [ ] Compare balance: in-memory vs API
- [ ] Detect missing orders (API has, we don't)
- [ ] Detect stale orders (we have, API doesn't)
- [ ] Update state to match API (source of truth)
- [ ] Log discrepancies
- [ ] Broadcast reconciliation events

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1294-1350` (start_reconciliation_loop)

**Test Cases**: TC-REAL-100, TC-REAL-101, TC-REAL-102

**Related Design**: NFR-RELIABILITY.md

---

### FR-REAL-052: Run Reconciliation

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall execute reconciliation logic comparing in-memory state with Binance API.

**Acceptance Criteria**:
- [ ] Fetch open orders from API
- [ ] Fetch account balance from API
- [ ] Compare order states
- [ ] Compare balance
- [ ] Fix discrepancies (add missing, remove stale)
- [ ] Log reconciliation results
- [ ] Handle API errors gracefully

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1378-1410` (run_reconciliation)

**Test Cases**: TC-REAL-103, TC-REAL-104

**Related Design**: NFR-RELIABILITY.md

---

### FR-REAL-053: Balance Reconciliation

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall reconcile account balance between in-memory cache and Binance API.

**Acceptance Criteria**:
- [ ] Fetch balance from API
- [ ] Compare with cached balance
- [ ] Log differences (free, locked)
- [ ] Update cache to match API
- [ ] Broadcast BalanceUpdated event if changed
- [ ] Handle API errors

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1411-1470` (reconcile_balance)

**Test Cases**: TC-REAL-110, TC-REAL-111

**Related Design**: NFR-RELIABILITY.md

---

### FR-REAL-054: Order Reconciliation

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall reconcile orders between in-memory state and Binance API.

**Acceptance Criteria**:
- [ ] Fetch all open orders from API
- [ ] Identify missing orders (API has, we don't)
- [ ] Identify stale orders (we have, API doesn't)
- [ ] Add missing orders to in-memory state
- [ ] Update stale orders (mark as FILLED or CANCELED)
- [ ] Log reconciliation actions
- [ ] Broadcast events for new/updated orders

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1472-1620` (reconcile_orders)

**Test Cases**: TC-REAL-115, TC-REAL-116, TC-REAL-117

**Related Design**: NFR-RELIABILITY.md

---

### FR-REAL-055: Stale Order Cleanup

**Priority**: Medium
**Status**: ✅ Implemented

**Description**:
The system shall clean up orders that are in our state but not in Binance API (likely filled/cancelled).

**Acceptance Criteria**:
- [ ] Identify orders in memory but not in API
- [ ] Query order status from API
- [ ] Update order to FILLED or CANCELED based on API response
- [ ] Remove from open orders map
- [ ] Log cleanup action
- [ ] Broadcast OrderFilled or OrderCancelled event

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1622-1710` (cleanup_stale_orders)

**Test Cases**: TC-REAL-120, TC-REAL-121

**Related Design**: NFR-RELIABILITY.md

---

### FR-REAL-056: WebSocket Disconnect Handler

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall handle User Data Stream disconnections gracefully with automatic reconnection.

**Acceptance Criteria**:
- [ ] Detect WebSocket disconnect
- [ ] Log disconnect event
- [ ] Attempt reconnection with exponential backoff
- [ ] Reinitialize listen key
- [ ] Run reconciliation after reconnect
- [ ] Broadcast ConnectionLost and ConnectionRestored events
- [ ] Circuit breaker after multiple consecutive failures

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1713-1735` (handle_disconnect)

**Test Cases**: TC-REAL-130, TC-REAL-131, TC-REAL-132

**Related Design**: NFR-RELIABILITY.md, API-WEBSOCKET.md

---

### FR-REAL-057: Emergency Stop

**Priority**: Critical
**Status**: ✅ Implemented

**Description**:
The system shall provide an emergency stop mechanism to halt all trading immediately.

**Acceptance Criteria**:
- [ ] Cancel all open orders
- [ ] Close all open positions (market sell)
- [ ] Halt order execution (circuit breaker)
- [ ] Log emergency stop
- [ ] Broadcast EmergencyStop event
- [ ] Persist emergency stop state
- [ ] Require manual reset to resume trading

**Code Location**:
- `rust-core-engine/src/real_trading/engine.rs:1738-1800` (emergency_stop)

**Test Cases**: TC-REAL-140, TC-REAL-141

**Related Design**: FR-RISK.md, NFR-RELIABILITY.md

---

### FR-REAL-API-001: Real Trading API Endpoints

**Priority**: High
**Status**: ✅ Implemented

**Description**:
The system shall provide REST API endpoints for real trading operations.

**Acceptance Criteria**:
- [ ] POST /api/real-trading/start - Start engine
- [ ] POST /api/real-trading/stop - Stop engine
- [ ] POST /api/real-trading/order/market - Place market order
- [ ] POST /api/real-trading/order/limit - Place limit order
- [ ] POST /api/real-trading/order/stop-loss - Place stop-loss order
- [ ] POST /api/real-trading/order/take-profit - Place take-profit order
- [ ] DELETE /api/real-trading/order/:order_id - Cancel order
- [ ] GET /api/real-trading/orders - Get all orders
- [ ] GET /api/real-trading/positions - Get all positions
- [ ] GET /api/real-trading/balance - Get account balance
- [ ] POST /api/real-trading/emergency-stop - Emergency stop
- [ ] All endpoints require authentication
- [ ] Return JSON responses

**Code Location**:
- `rust-core-engine/src/api/real_trading.rs:1-500`

**Test Cases**: TC-REAL-150 to TC-REAL-165

**Related Design**: API-RUST-CORE.md

---

## Data Requirements

### Input Data

**Configuration**:
```rust
pub struct RealTradingConfig {
    pub testnet: bool,              // Default: true
    pub max_open_positions: usize,  // Default: 10
    pub daily_loss_limit_percent: f64, // Default: 5.0%
    pub circuit_breaker_threshold: u32, // Default: 5
    pub circuit_breaker_cooldown_secs: u64, // Default: 300 (5 min)
    pub reconciliation_interval_secs: u64, // Default: 300
}
```

**Order Request**:
```rust
pub struct OrderRequest {
    pub symbol: String,         // e.g., "BTCUSDT"
    pub side: OrderSide,        // Buy/Sell
    pub quantity: f64,          // Amount to trade
    pub price: Option<f64>,     // For limit orders
    pub stop_price: Option<f64>, // For stop-loss/take-profit
}
```

### Output Data

**Order Response**:
```rust
pub struct RealOrder {
    pub order_id: u64,
    pub client_order_id: String,
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub status: OrderState,     // NEW, FILLED, CANCELED, REJECTED
    pub quantity: f64,
    pub filled_quantity: f64,
    pub fills: Vec<Fill>,
    pub created_at: DateTime<Utc>,
}
```

**Position**:
```rust
pub struct RealPosition {
    pub symbol: String,
    pub side: PositionSide,     // Long/Short
    pub entry_price: f64,
    pub quantity: f64,
    pub unrealized_pnl: f64,
    pub realized_pnl: f64,
    pub opened_at: DateTime<Utc>,
}
```

---

## Interface Requirements

### REST API

**Base URL**: `http://localhost:8080/api/real-trading`

**Authentication**: JWT Bearer token required for all endpoints

**Endpoints**:
- `POST /start` - Start real trading engine
- `POST /stop` - Stop engine
- `POST /order/market` - Place market order
- `POST /order/limit` - Place limit order
- `DELETE /order/:id` - Cancel order
- `GET /orders` - List all orders
- `GET /positions` - List all positions
- `GET /balance` - Get account balance
- `POST /emergency-stop` - Emergency stop

### WebSocket Events

**Subscription**: `ws://localhost:8080/ws` (real-time events)

**Events**:
- `RealTradingEvent::OrderPlaced` - Order submitted
- `RealTradingEvent::OrderFilled` - Order fully filled
- `RealTradingEvent::OrderPartiallyFilled` - Order partially filled
- `RealTradingEvent::OrderCancelled` - Order cancelled
- `RealTradingEvent::PositionOpened` - New position
- `RealTradingEvent::PositionUpdated` - Position changed
- `RealTradingEvent::PositionClosed` - Position closed
- `RealTradingEvent::BalanceUpdated` - Balance changed
- `RealTradingEvent::CircuitBreakerOpened` - Trading halted
- `RealTradingEvent::EmergencyStop` - Emergency stop triggered

---

## Testing Strategy

### Unit Tests

**Target Coverage**: >90%

**Test Files**:
- `tests/test_real_trading.rs` - Core engine tests
- `tests/test_real_order.rs` - Order tracking tests
- `tests/test_real_position.rs` - Position tracking tests
- `tests/test_real_risk.rs` - Risk management tests
- `tests/test_real_config.rs` - Configuration tests

**Test Scenarios**:
- Order execution (market, limit, stop-loss, take-profit)
- Position tracking (open, update, close)
- Risk validation (position limits, daily loss)
- Circuit breaker (open, close, cooldown)
- Reconciliation (balance, orders)
- WebSocket disconnect/reconnect

### Integration Tests

**Test Environment**: Binance Testnet

**Test Scenarios**:
- Place market order → receive fill → update position
- Place limit order → partial fill → full fill
- Exceed daily loss limit → reject order
- Circuit breaker → halt trading → cooldown → resume
- WebSocket disconnect → reconnect → reconcile

### Manual Testing

**Pre-Production Checklist**:
- [ ] Test with Binance Testnet for 7 days
- [ ] Verify all orders execute correctly
- [ ] Verify positions track accurately
- [ ] Verify risk limits enforced
- [ ] Verify reconciliation works
- [ ] Verify circuit breaker works
- [ ] Verify emergency stop works
- [ ] External security audit completed

---

## Traceability

### Related Requirements

- **FR-TRADING-001 to FR-TRADING-020**: General trading requirements
- **FR-RISK-001 to FR-RISK-011**: Risk management requirements
- **FR-WEBSOCKET-001 to FR-WEBSOCKET-007**: WebSocket requirements
- **NFR-SECURITY-001 to NFR-SECURITY-010**: Security requirements
- **NFR-RELIABILITY-001 to NFR-RELIABILITY-012**: Reliability requirements

### Design Documents

- **COMP-RUST-TRADING.md**: Real trading engine component design
- **API-RUST-CORE.md**: REST API specification
- **API-WEBSOCKET.md**: WebSocket event specification
- **DB-SCHEMA.md**: Database schema for orders/positions
- **ARCH-DATA-FLOW.md**: Data flow architecture

### Test Cases

- **TC-REAL-001 to TC-REAL-165**: Real trading test cases
- **TC-INTEGRATION-001 to TC-INTEGRATION-045**: Integration tests
- **TC-SECURITY-001 to TC-SECURITY-035**: Security tests

### Code Locations

All code uses `@spec:FR-REAL-XXX` tags for traceability:
- `rust-core-engine/src/real_trading/` - 9 files
- `rust-core-engine/src/binance/` - 5 files
- `rust-core-engine/src/api/real_trading.rs` - API endpoints

---

## Changelog

| Date | Version | Changes | Author |
|------|---------|---------|--------|
| 2026-02-06 | 1.0 | Initial specification created from code analysis | Claude (Fullstack Dev) |

---

## Notes

**⚠️ CRITICAL SAFETY REQUIREMENTS**:

1. **DO NOT enable mainnet trading** without:
   - [ ] 7+ days of testnet validation
   - [ ] External security audit
   - [ ] Risk management review
   - [ ] Explicit user confirmation

2. **Default configuration** must be:
   - Testnet: `true`
   - Trading enabled: `false`
   - Max daily loss: 5%
   - Circuit breaker: enabled

3. **Production deployment** requires:
   - Monitoring (Prometheus, Grafana)
   - Alerting (Slack, email)
   - Audit logging
   - Backup/restore procedures

**Finance Risk**: Real trading involves real money. All changes to this module require thorough testing and review.

---

**Document Status**: ✅ Complete
**Specification Version**: 1.0
**Total Requirements**: 23 (FR-REAL-001 to FR-REAL-057, FR-REAL-API-001)
**Implementation Status**: ✅ All Implemented
**Last Updated**: 2026-02-06
