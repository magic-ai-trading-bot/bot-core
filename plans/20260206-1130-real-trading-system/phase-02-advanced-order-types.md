# Phase 02: Advanced Order Types

**Status**: ✅ Done | **Completed**: 2026-02-06

## Context

After core Spot trading works, add advanced order types for risk management: Stop-Loss, Take-Profit, Trailing Stop, and OCO orders.

## Overview

Implement advanced order types using Binance Spot API order variants and OCO endpoint.

## Key Insights (From Research)

- `STOP_LOSS` / `STOP_LOSS_LIMIT`: Trigger when price falls below stopPrice
- `TAKE_PROFIT` / `TAKE_PROFIT_LIMIT`: Trigger when price rises above stopPrice
- `TRAILING_STOP_LOSS`: Uses `trailingDelta` instead of fixed stopPrice
- OCO: `POST /api/v3/orderList/oco` - One-Cancels-Other (stop + limit pair)

## Requirements

1. Stop-Loss orders (market + limit variants)
2. Take-Profit orders (market + limit variants)
3. Trailing Stop orders with configurable delta
4. OCO orders (stop-loss + take-profit pair)
5. Attach SL/TP to positions automatically

## Architecture

```
SpotOrderType (enum)
    ├── StopLoss { stop_price }
    ├── StopLossLimit { stop_price, limit_price }
    ├── TakeProfit { stop_price }
    ├── TakeProfitLimit { stop_price, limit_price }
    └── TrailingStop { trailing_delta_bps }

OCOOrderRequest
    ├── symbol
    ├── quantity
    ├── price (limit)
    ├── stop_price
    └── stop_limit_price
```

## Related Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/binance/types.rs` | Modify | Add SL/TP/Trailing types |
| `rust-core-engine/src/binance/client.rs` | Modify | Add OCO endpoint |
| `rust-core-engine/src/real_trading/engine.rs` | Modify | Add SL/TP order placement |
| `rust-core-engine/src/real_trading/order.rs` | Modify | Track attached SL/TP orders |

## Implementation Steps

### 1. Types - Order Variants
```rust
// Extend SpotOrderType enum in types.rs
pub enum SpotOrderType {
    Market,
    Limit,
    StopLoss,
    StopLossLimit,
    TakeProfit,
    TakeProfitLimit,
    TrailingStopLoss,  // Uses trailingDelta
    LimitMaker,
}
```

### 2. Binance Client - OCO
```rust
// Add to client.rs
pub async fn place_oco_order(
    &self,
    symbol: &str,
    side: OrderSide,
    quantity: f64,
    price: f64,         // Limit price
    stop_price: f64,    // Stop trigger
    stop_limit_price: f64,
) -> Result<OcoOrderResponse>

pub async fn cancel_oco_order(
    &self,
    symbol: &str,
    order_list_id: i64,
) -> Result<CancelOcoResponse>
```

### 3. Engine - Position Risk Management
```rust
// Add to engine.rs
pub async fn attach_stop_loss(
    &self,
    position_id: &str,
    stop_price: f64,
) -> Result<RealOrder>

pub async fn attach_take_profit(
    &self,
    position_id: &str,
    take_profit_price: f64,
) -> Result<RealOrder>

pub async fn attach_oco(
    &self,
    position_id: &str,
    stop_price: f64,
    take_profit_price: f64,
) -> Result<(RealOrder, RealOrder)>
```

### 4. Trailing Stop
```rust
pub async fn attach_trailing_stop(
    &self,
    position_id: &str,
    trailing_delta_bps: u32,  // Basis points (100 = 1%)
) -> Result<RealOrder>
```

## Todo

- [x] Add SpotOrderType variants to types.rs (STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT)
- [x] Add `place_oco_order()` to BinanceClient (new API format with aboveType/belowType)
- [x] Add `cancel_oco_order()` to BinanceClient
- [x] Add OcoOrderRequest with new API format (2024+)
- [x] Test SL/TP on testnet - PASSED
- [x] Test OCO on testnet - PASSED (Order List ID: 14639)
- [ ] Implement trailing stop order placement (deferred to engine integration)
- [ ] Add `attach_stop_loss()` to engine (Phase 4)
- [ ] Add `attach_take_profit()` to engine (Phase 4)
- [ ] Add `attach_oco()` to engine (Phase 4)
- [ ] Track SL/TP order IDs in RealOrder (Phase 4)
- [ ] Handle OCO fill events via WebSocket (Phase 4)

## Test Results (Testnet)

```
✅ Market Buy Order (Order ID: 399879) - FILLED
✅ OCO Order (Order List ID: 14639) - EXECUTING
   - Stop Loss: STOP_LOSS_LIMIT @ $62,930.95 (Order ID: 399880)
   - Take Profit: LIMIT_MAKER @ $65,835.45 (Order ID: 399881)
✅ OCO Cancel - ALL_DONE
✅ Stop-Loss Limit (Order ID: 399888) - NEW
✅ Take-Profit Limit (Order ID: 399889) - NEW
```

## Success Criteria

- [x] Place stop-loss order successfully
- [x] Place take-profit order successfully
- [x] Place OCO order with both SL and TP
- [ ] OCO: when one side fills, other cancels (requires WebSocket integration)
- [ ] Trailing stop adjusts as price moves favorably (not available on Spot testnet)

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Trailing stop not on testnet | Medium | High | Fall back to regular stop |
| OCO validation errors | Medium | Medium | Validate prices before submit |
| Race condition on OCO cancel | Low | Medium | Check order state before action |

## Security Considerations

- Validate stop price < entry for long SL (prevent immediate trigger)
- Validate take profit > entry for long TP
- Log all SL/TP placements for audit

## Next Steps

After completion: Proceed to Phase 03 (Futures Trading) for leverage and position sides.
