# Phase 01: Core Spot Trading

**Status**: Pending | **Estimated Time**: 1 day

## Context

Existing `real_trading` module has basic structure but needs Spot order placement, execution tracking, and balance management to be production-ready.

## Overview

Complete Spot trading foundation: market/limit orders, execution tracking via WebSocket, balance sync.

## Key Insights (From Research)

- Binance Spot testnet: `https://testnet.binance.vision`
- Order endpoints: `POST /api/v3/order`, `DELETE /api/v3/order`
- Auth: HMAC-SHA256 signature required
- User data stream for order updates (already in `user_data_stream.rs`)

## Requirements

1. Implement `place_spot_order()` for MARKET and LIMIT orders
2. Implement `cancel_spot_order()` with order ID
3. Handle ExecutionReport events for order state updates
4. Sync account balances on startup and after trades
5. Add testnet API key configuration

## Architecture

```
BinanceClient
    ├── place_spot_order(symbol, side, type, qty, price?) -> OrderResponse
    ├── cancel_spot_order(symbol, order_id) -> CancelResponse
    └── get_account() -> AccountInfo

RealTradingEngine
    ├── execute_spot_buy(symbol, qty, price?) -> RealOrder
    ├── execute_spot_sell(symbol, qty, price?) -> RealOrder
    └── process_execution_report(report) [already exists]
```

## Related Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/binance/client.rs` | Modify | Add `place_spot_order`, `cancel_spot_order` |
| `rust-core-engine/src/real_trading/engine.rs` | Modify | Add `execute_spot_buy`, `execute_spot_sell` |
| `rust-core-engine/src/binance/types.rs` | Modify | Add request/response types (most exist) |
| `rust-core-engine/.env` | Modify | Add testnet API keys |

## Implementation Steps

### 1. Binance Client - Spot Order Placement
```rust
// Add to client.rs
pub async fn place_spot_order(&self, request: SpotOrderRequest) -> Result<OrderResponse>
pub async fn cancel_spot_order(&self, symbol: &str, order_id: i64) -> Result<CancelOrderResponse>
pub async fn get_account(&self) -> Result<AccountInfo>
```

### 2. Engine - Spot Execution
```rust
// Add to engine.rs
pub async fn execute_spot_buy(&self, symbol: &str, quantity: f64, price: Option<f64>) -> Result<RealOrder>
pub async fn execute_spot_sell(&self, symbol: &str, quantity: f64, price: Option<f64>) -> Result<RealOrder>
```

### 3. Integration
- Wire up execution to existing `process_execution_report`
- Add balance refresh after order execution
- Update position tracking on fills

## Todo

- [ ] Add `place_spot_order()` to BinanceClient
- [ ] Add `cancel_spot_order()` to BinanceClient
- [ ] Add `get_account()` to BinanceClient (may exist)
- [ ] Add `execute_spot_buy()` to RealTradingEngine
- [ ] Add `execute_spot_sell()` to RealTradingEngine
- [ ] Add testnet API key config to `.env.example`
- [ ] Test order placement on testnet
- [ ] Test order cancellation on testnet
- [ ] Verify ExecutionReport handling

## Success Criteria

- [ ] Place market buy order successfully on testnet
- [ ] Place limit sell order successfully on testnet
- [ ] Cancel pending order successfully
- [ ] Balances update after order fill
- [ ] ExecutionReport updates order state correctly

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Testnet rate limits | Low | Low | Already have rate limiter |
| Signature errors | Medium | Medium | Test with simple order first |
| Balance parse errors | Low | Low | Use existing Balance type |

## Security Considerations

- API keys stored in `.env` only (never commit)
- Testnet mode as default in config
- Log order IDs but not API keys

## Next Steps

After completion: Proceed to Phase 02 (Advanced Order Types) for SL/TP, OCO, trailing stop.
