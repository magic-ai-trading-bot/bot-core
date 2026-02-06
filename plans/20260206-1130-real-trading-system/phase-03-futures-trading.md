# Phase 03: Futures Trading

**Status**: Pending | **Estimated Time**: 1.5 days

## Context

Extend trading system to support USDM Futures with leverage, position sides (LONG/SHORT), and futures-specific order types.

## Overview

Implement Futures trading: leverage management, margin modes, position tracking, and futures order placement on testnet.

## Key Insights (From Research)

- Futures testnet: `https://testnet.binancefuture.com`
- Endpoints: `/fapi/v1/order`, `/fapi/v1/positionRisk`, `/fapi/v1/account`
- Position side: LONG / SHORT (or BOTH for hedge mode)
- Margin modes: CROSSED / ISOLATED
- Max leverage: 125x (we limit to 10x for safety)

## Requirements

1. Futures client with testnet URL configuration
2. Leverage setting per symbol
3. Long and Short position opening
4. Position close with reduce-only orders
5. Futures-specific SL/TP (STOP_MARKET, TAKE_PROFIT_MARKET)
6. TRAILING_STOP_MARKET for futures
7. Position risk tracking (liquidation price)

## Architecture

```
BinanceClient (extended)
    ├── place_futures_order(request) -> FuturesOrderResponse
    ├── cancel_futures_order(symbol, order_id)
    ├── set_leverage(symbol, leverage)
    ├── set_margin_type(symbol, margin_type)
    ├── get_futures_account() -> FuturesAccountInfo
    └── get_position_risk(symbol) -> FuturesPosition

RealTradingEngine
    ├── open_long(symbol, qty, leverage) -> RealPosition
    ├── open_short(symbol, qty, leverage) -> RealPosition
    ├── close_position(position_id)
    └── update_leverage(symbol, leverage)
```

## Related Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/binance/client.rs` | Modify | Add futures methods |
| `rust-core-engine/src/binance/types.rs` | Modify | FuturesOrderRequest, FuturesPosition |
| `rust-core-engine/src/real_trading/engine.rs` | Modify | Futures execution methods |
| `rust-core-engine/src/real_trading/position.rs` | Modify | Add leverage, margin fields |
| `rust-core-engine/src/real_trading/config.rs` | Modify | Futures config options |

## Implementation Steps

### 1. Binance Client - Futures Methods
```rust
// Add to client.rs
pub async fn place_futures_order(&self, request: FuturesOrderRequest) -> Result<FuturesOrderResponse>
pub async fn cancel_futures_order(&self, symbol: &str, order_id: i64) -> Result<CancelOrderResponse>
pub async fn set_leverage(&self, symbol: &str, leverage: u32) -> Result<LeverageResponse>
pub async fn set_margin_type(&self, symbol: &str, margin_type: &str) -> Result<()>
pub async fn get_futures_account(&self) -> Result<FuturesAccountInfo>
pub async fn get_position_risk(&self, symbol: Option<&str>) -> Result<Vec<FuturesPosition>>
```

### 2. Types - Futures Structures
```rust
pub struct FuturesOrderRequest {
    pub symbol: String,
    pub side: OrderSide,
    pub order_type: FuturesOrderType,
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub reduce_only: Option<bool>,
    pub position_side: PositionSide,
    pub time_in_force: Option<TimeInForce>,
}

pub enum FuturesOrderType {
    Market,
    Limit,
    Stop,
    StopMarket,
    TakeProfit,
    TakeProfitMarket,
    TrailingStopMarket,
}

pub enum PositionSide {
    Long,
    Short,
    Both,  // For one-way mode
}
```

### 3. Engine - Futures Execution
```rust
// Add to engine.rs
pub async fn open_long(
    &self,
    symbol: &str,
    quantity: f64,
    leverage: u32,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
) -> Result<RealPosition>

pub async fn open_short(
    &self,
    symbol: &str,
    quantity: f64,
    leverage: u32,
    stop_loss: Option<f64>,
    take_profit: Option<f64>,
) -> Result<RealPosition>

pub async fn close_position(&self, position_id: &str) -> Result<f64> // Returns PnL
pub async fn update_position_leverage(&self, symbol: &str, leverage: u32) -> Result<()>
```

### 4. Position Updates
```rust
// Extend RealPosition in position.rs
pub struct RealPosition {
    // ... existing fields ...
    pub leverage: u32,
    pub margin_type: String,  // "CROSSED" or "ISOLATED"
    pub liquidation_price: f64,
    pub mark_price: f64,
    pub margin: f64,
}
```

## Todo

- [ ] Add futures base URL to BinanceConfig
- [ ] Implement `place_futures_order()`
- [ ] Implement `cancel_futures_order()`
- [ ] Implement `set_leverage()`
- [ ] Implement `set_margin_type()`
- [ ] Implement `get_futures_account()`
- [ ] Implement `get_position_risk()`
- [ ] Add FuturesOrderRequest and response types
- [ ] Add PositionSide enum
- [ ] Implement `open_long()` in engine
- [ ] Implement `open_short()` in engine
- [ ] Implement `close_position()` in engine
- [ ] Add leverage/margin to RealPosition
- [ ] Handle futures user data stream events
- [ ] Test long position on futures testnet
- [ ] Test short position on futures testnet
- [ ] Test leverage adjustment

## Success Criteria

- [ ] Open long position with 5x leverage
- [ ] Open short position with 5x leverage
- [ ] Close position and verify PnL calculation
- [ ] SL/TP orders attached to futures positions
- [ ] Liquidation price calculated and displayed
- [ ] Leverage change applied successfully

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Futures testnet data differs | Medium | Medium | Accept simulation limits |
| Leverage validation | Low | High | Cap at 10x, validate range |
| Position mode confusion | Medium | Medium | Default to one-way mode |
| Margin calculation errors | Medium | High | Use exchange-provided values |

## Security Considerations

- Default leverage cap at 10x (configurable)
- Warn user when leverage > 5x
- Track margin ratio, alert when < 150%
- Never exceed max_leverage from config

## Next Steps

After completion: Proceed to Phase 04 (Manual Trading API) for REST endpoints.
