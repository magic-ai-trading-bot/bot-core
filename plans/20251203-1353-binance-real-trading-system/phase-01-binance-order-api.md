# Phase 1: Binance Order API Extension

## Context Links
- [Main Plan](./plan.md)
- [Binance REST API Research](./research/researcher-01-binance-rest-api.md)
- [WebSocket User Data Research](./research/researcher-02-binance-websocket-userdata.md)

## Overview

| Field | Value |
|-------|-------|
| Priority | HIGH |
| Status | PENDING |
| Effort | 3 days |
| Dependencies | Research complete |

Extend existing `BinanceClient` with order placement endpoints and WebSocket user data stream for real-time order updates. Foundation for real trading engine.

## Key Insights from Research

1. **Order Types Available**: MARKET, LIMIT, LIMIT_MAKER, STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT, TRAILING_STOP_MARKET
2. **Authentication**: HMAC-SHA256 signature (already implemented in client)
3. **Rate Limits**: 6,000 weight/min, 50 orders/10s, 160,000 orders/day
4. **User Data Stream**: Listen key expires 60 min, keepalive every 30 min
5. **Testnet URLs differ**: `testnet.binance.vision` vs `api.binance.com`

## Requirements

### Functional
- FR-REAL-001: Place MARKET orders (spot, futures)
- FR-REAL-002: Place LIMIT orders with timeInForce (GTC, IOC, FOK)
- FR-REAL-003: Place STOP_LOSS_LIMIT orders (trigger + execution price)
- FR-REAL-004: Place TAKE_PROFIT_LIMIT orders
- FR-REAL-005: Cancel orders by orderId or clientOrderId
- FR-REAL-006: Query order status
- FR-REAL-007: Create and manage user data stream listen key
- FR-REAL-008: Parse executionReport events from WebSocket

### Non-Functional
- NFR-REAL-001: Respect rate limits with exponential backoff
- NFR-REAL-002: Testnet by default via config flag
- NFR-REAL-003: <100ms latency for order operations

## Architecture

```
BinanceClient (Extended)
├── Order Methods (NEW)
│   ├── place_spot_order()
│   ├── place_market_order()
│   ├── place_limit_order()
│   ├── place_stop_loss_limit_order()
│   ├── place_take_profit_limit_order()
│   ├── cancel_order_by_id()
│   ├── get_order_status()
│   └── get_all_orders()
│
├── User Data Stream (NEW)
│   ├── create_listen_key()
│   ├── keepalive_listen_key()
│   ├── close_listen_key()
│   └── get_user_data_stream_url()
│
└── Existing Methods
    ├── get_klines()
    ├── get_account_info()
    ├── place_futures_order()  ← Already exists
    └── ...
```

### New Types

```rust
// Order request for spot trading
pub struct SpotOrderRequest {
    pub symbol: String,
    pub side: OrderSide,           // BUY, SELL
    pub order_type: SpotOrderType, // MARKET, LIMIT, STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT
    pub quantity: Option<String>,
    pub quote_order_qty: Option<String>, // For MARKET orders by quote amount
    pub price: Option<String>,           // For LIMIT orders
    pub stop_price: Option<String>,      // For STOP_LOSS/TAKE_PROFIT
    pub time_in_force: Option<String>,   // GTC, IOC, FOK
    pub client_order_id: Option<String>, // Custom ID for tracking
}

pub enum OrderSide {
    Buy,
    Sell,
}

pub enum SpotOrderType {
    Market,
    Limit,
    StopLossLimit,
    TakeProfitLimit,
}

// Execution report from WebSocket
pub struct ExecutionReport {
    pub event_type: String,         // "executionReport"
    pub event_time: i64,
    pub symbol: String,
    pub client_order_id: String,
    pub side: String,
    pub order_type: String,
    pub time_in_force: String,
    pub quantity: String,
    pub price: String,
    pub stop_price: String,
    pub execution_type: String,     // NEW, TRADE, CANCELED, REJECTED, EXPIRED
    pub order_status: String,       // NEW, PARTIALLY_FILLED, FILLED, CANCELED, REJECTED, EXPIRED
    pub order_id: i64,
    pub last_executed_qty: String,
    pub cumulative_filled_qty: String,
    pub last_executed_price: String,
    pub commission_amount: String,
    pub commission_asset: String,
    pub trade_id: i64,
}

pub struct UserDataStreamHandle {
    pub listen_key: String,
    pub ws_url: String,
    pub created_at: DateTime<Utc>,
    pub last_keepalive: DateTime<Utc>,
}
```

## Related Code Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/binance/client.rs` | MODIFY | Add order methods |
| `rust-core-engine/src/binance/types.rs` | MODIFY | Add new types |
| `rust-core-engine/src/binance/user_data_stream.rs` | CREATE | WebSocket handler |
| `rust-core-engine/src/binance/mod.rs` | MODIFY | Export new modules |
| `rust-core-engine/src/config.rs` | MODIFY | Add mainnet URLs |

## Implementation Steps

### Step 1: Add Spot Order Types (Day 1)

1. Add `SpotOrderRequest`, `OrderSide`, `SpotOrderType` to `types.rs`
2. Add `ExecutionReport` struct for WebSocket parsing
3. Add `UserDataStreamHandle` for stream management

### Step 2: Extend BinanceClient with Order Methods (Day 1-2)

1. Implement `place_spot_order()` - generic order placement
2. Implement convenience methods:
   - `place_market_order(symbol, side, quantity)`
   - `place_limit_order(symbol, side, quantity, price)`
   - `place_stop_loss_limit_order(symbol, side, quantity, price, stop_price)`
   - `place_take_profit_limit_order(symbol, side, quantity, price, stop_price)`
3. Implement `cancel_order_by_id(symbol, order_id)`
4. Implement `get_order_status(symbol, order_id)`
5. Implement `get_all_orders(symbol, limit)`

### Step 3: Add User Data Stream Management (Day 2)

1. Implement `create_listen_key()` - POST /api/v3/userDataStream
2. Implement `keepalive_listen_key(listen_key)` - PUT /api/v3/userDataStream
3. Implement `close_listen_key(listen_key)` - DELETE /api/v3/userDataStream
4. Add `get_user_data_stream_url(listen_key)` - construct WebSocket URL

### Step 4: Implement User Data Stream WebSocket (Day 2-3)

1. Create `user_data_stream.rs` module
2. Implement `UserDataStreamManager`:
   - Connect to WebSocket with listen key
   - Auto-keepalive every 30 minutes
   - Parse executionReport events
   - Parse outboundAccountPosition events
   - Reconnect with exponential backoff
   - Broadcast events via channel

### Step 5: Add Config for Mainnet URLs (Day 3)

1. Add mainnet URLs to `BinanceConfig`:
   - `mainnet_base_url: "https://api.binance.com"`
   - `mainnet_ws_url: "wss://stream.binance.com:9443"`
2. Add `get_base_url(&self) -> &str` method that returns URL based on `testnet` flag
3. Update `make_request()` to use dynamic URL

### Step 6: Write Tests (Day 3)

1. Unit tests for order serialization
2. Integration tests with testnet (mock for CI)
3. Test rate limit backoff behavior

## Todo List

- [ ] Add SpotOrderRequest and related types to types.rs
- [ ] Add ExecutionReport struct for WebSocket parsing
- [ ] Implement place_spot_order() in client.rs
- [ ] Implement place_market_order() convenience method
- [ ] Implement place_limit_order() convenience method
- [ ] Implement place_stop_loss_limit_order()
- [ ] Implement place_take_profit_limit_order()
- [ ] Implement cancel_order_by_id()
- [ ] Implement get_order_status()
- [ ] Implement get_all_orders()
- [ ] Implement create_listen_key()
- [ ] Implement keepalive_listen_key()
- [ ] Implement close_listen_key()
- [ ] Create UserDataStreamManager in user_data_stream.rs
- [ ] Add mainnet URLs to BinanceConfig
- [ ] Add get_base_url() method for dynamic URL selection
- [ ] Write unit tests for new types
- [ ] Write integration tests with testnet

## Success Criteria

1. All order types place successfully on testnet
2. Orders cancel and query correctly
3. User data stream connects and receives executionReport events
4. Listen key auto-refreshes before 60 min expiry
5. Rate limiting handled with backoff (no 429/418 errors)
6. All tests pass

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Testnet API differences | Medium | Low | Verify each endpoint manually |
| Rate limit exceeded | Medium | Medium | Track weight in headers |
| WebSocket disconnect | Medium | Medium | Auto-reconnect with backoff |
| Time sync issues | Low | Low | Use server time from API |

## Security Considerations

1. **API keys in env vars only** - Never hardcode or log
2. **Testnet by default** - Explicit flag required for mainnet
3. **Signature verification** - Already implemented in sign_request()
4. **TLS only** - HTTPS/WSS enforced

## Next Steps

After Phase 1 complete:
- Proceed to [Phase 2: Real Trading Engine](./phase-02-real-trading-engine.md)
- Use order API to build engine mirroring paper trading
