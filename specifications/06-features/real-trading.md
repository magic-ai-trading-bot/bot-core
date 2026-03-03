# Real Trading

**Spec**: FR-REAL | **Status**: Implemented | **Updated**: 2026-03-03

## Quick Reference

| Item | Value |
|------|-------|
| Mode default | Testnet (`use_testnet: true`) |
| Trading type | Futures (USDT-M) |
| API | `GET/POST /api/real-trading/*` |
| Code | `rust-core-engine/src/real_trading/` |
| Spec | `specifications/01-requirements/1.1-functional-requirements/FR-REAL-TRADING.md` |
| MCP tools | 14 tools in `mcp-server/src/tools/real-trading.ts` |

### Code Locations

| File | Purpose |
|------|---------|
| `src/real_trading/mod.rs` | Module root, re-exports |
| `src/real_trading/config.rs` | `RealTradingConfig` — all settings |
| `src/real_trading/engine.rs` | `RealTradingEngine` — execution core |
| `src/real_trading/order.rs` | `RealOrder`, `OrderState`, `OrderFill` |
| `src/real_trading/position.rs` | `RealPosition`, `PositionSide` |
| `src/real_trading/risk.rs` | `RealTradingRiskManager` — pre-trade checks |
| `mcp-server/src/tools/real-trading.ts` | MCP tool definitions (8 tools) |

---

## How It Works

### Architecture

```
Signal/Manual Request
        |
        v
RealTradingEngine
  ├── Pre-trade risk check (RealTradingRiskManager)
  ├── Execution lock (prevents race conditions)
  ├── BinanceClient.place_order()
  ├── Order tracking (DashMap<String, RealOrder>)
  └── Position management (DashMap<String, RealPosition>)
        |
        v
UserDataStreamManager (WebSocket)
  ├── ExecutionReport → order state updates
  ├── OutboundAccountPosition → balance updates
  └── BalanceUpdate → live balance tracking
        |
        v
Event Broadcast (tokio::broadcast)
  └── RealTradingEvent enum → WebSocket clients
```

### Order Lifecycle

```
Pending → New → PartiallyFilled → Filled
                               → Cancelled
                               → Rejected
                               → Expired
```

States in `OrderState` enum. Mapped from Binance status strings via `from_binance_status()`.

### Position Flow

1. Market/Limit order fills → `ExecutionReport` received via WebSocket
2. Engine calls `position.add_fill()` — updates VWAP entry price
3. Price update → `position.update_price()` → recalculates unrealized PnL
4. SL/TP trigger check → `should_trigger_stop_loss()` / `should_trigger_take_profit()`
5. Close order placed → `partial_close()` records realized PnL

### Trailing Stop Logic

PnL-based activation (not price-based):
1. Set `trailing_stop_activation` = PnL% threshold (e.g., 4.0)
2. When unrealized PnL% >= threshold → `trailing_stop_active = true`
3. Track `best_price_since_trailing` (high-water mark for longs, low-water for shorts)
4. Trailing stop = best_price * (1 ± trail_pct/100)
5. Stop only ratchets in favorable direction (never retraces)

### Circuit Breaker

- Tracks consecutive errors via `CircuitBreakerState`
- Opens after N errors (default: 3 prod: 2) → halts trading
- Auto-closes after cooldown (default: 300s)
- Optional: auto-close all positions on open (`circuit_breaker_close_positions`)
- Events: `CircuitBreakerOpened(reason)`, `CircuitBreakerClosed`

### Reconciliation

REST API reconciliation runs every 5min (prod: 1min):
- Compare engine-tracked orders vs exchange open orders
- Cancel stale orders (older than `stale_order_timeout_secs`)
- Stop if discrepancy > `max_reconciliation_discrepancy_usdt`

---

## Risk Management

### Pre-Trade Checks (9 sequential validations)

| Check | Default limit | Behavior on fail |
|-------|--------------|-----------------|
| Daily loss limit | $500 | BLOCK |
| Max positions | 5 | BLOCK (new symbols) |
| Position size (margin) | $1000 | BLOCK |
| Total exposure (margin) | $5000 | BLOCK |
| Available balance (BUY only) | — | BLOCK |
| Min balance | $100 | BLOCK |
| Risk per trade % | 2% of balance | WARN + suggest size |
| Min order value | $10 | BLOCK |
| Symbol allowlist | empty=all | BLOCK |

Note: checks 3-6 are margin-based (notional / leverage), not notional.

### Position Sizing

Formula: `size = risk_amount / stop_distance / entry_price`
- risk_amount = balance * risk_per_trade_percent
- stop_distance = |entry - stop| / entry (min 0.5%)
- Capped by max_position_size and max_total_exposure

### Daily Loss Tracking

- `RealTradingRiskManager` tracks daily_loss, daily_pnl, daily_trades
- Auto-resets at UTC midnight via `check_daily_reset()`
- Warning at 80% of limit; block at 100%

### ATR-Based Sizing (optional)

When `atr_stop_enabled = true`:
- SL = entry ± ATR(period) * atr_stop_multiplier
- TP = entry ± ATR(period) * atr_tp_multiplier
- Size based on base_risk_pct of equity

### Kelly Criterion (optional)

When `kelly_enabled = true` and closed trades >= `kelly_min_trades`:
- Uses last `kelly_lookback` trades
- Half-Kelly by default (kelly_fraction = 0.5)

### Regime Filters (optional, all disabled by default)

| Filter | Trigger | Effect |
|--------|---------|--------|
| `funding_spike_filter_enabled` | funding rate > threshold (0.0003) | Reduce size by funding_spike_reduction (50%) |
| `atr_spike_filter_enabled` | ATR > mean * multiplier (2x) | Reduce size by atr_spike_reduction (50%) |
| `consecutive_loss_reduction_enabled` | N+ consecutive losses | Reduce size by consecutive_loss_reduction_pct (30%) per loss |

### Signal Reversal (optional)

When `enable_signal_reversal = true`:
- Requires confidence >= `reversal_min_confidence` (0.75)
- Position unrealized PnL% must be <= `reversal_max_pnl_pct` (5%)
- Market regime must be in `reversal_allowed_regimes` (["trending"])

---

## Configuration

### Safety Defaults

```toml
use_testnet = true               # NEVER mainnet by default
auto_trading_enabled = false     # No auto-trade without explicit opt-in
atr_stop_enabled = false
kelly_enabled = false
funding_spike_filter_enabled = false
atr_spike_filter_enabled = false
consecutive_loss_reduction_enabled = false
enable_signal_reversal = false
```

### Key Limits (Testnet Defaults)

```toml
max_position_size_usdt = 100.0   # Testnet: 100, Default: 1000
max_total_exposure_usdt = 500.0  # Testnet: 500, Default: 5000
max_daily_loss_usdt = 50.0       # Testnet: 50, Default: 500
max_positions = 5
risk_per_trade_percent = 2.0     # % of balance per trade
min_balance_usdt = 100.0
max_leverage = 1                 # No leverage by default
min_signal_confidence = 0.65
```

### Production Config Differences

```toml
use_testnet = false
circuit_breaker_errors = 2       # More sensitive (default: 3)
circuit_breaker_close_positions = true  # Close all on circuit break
reconciliation_interval_secs = 60       # More frequent (default: 300)
```

### Environment Variables

```
BINANCE_API_KEY=
BINANCE_SECRET_KEY=
BINANCE_FUTURES_TESTNET_API_KEY=
BINANCE_FUTURES_TESTNET_SECRET_KEY=
BINANCE_TESTNET=true             # CRITICAL: keep true until ready for real
TRADING_ENABLED=false            # Must explicitly set true
```

---

## MCP Tools

| Tool | Access | Endpoint |
|------|--------|---------|
| `get_real_trading_status` | Read | `GET /api/real-trading/status` |
| `get_real_portfolio` | Read | `GET /api/real-trading/portfolio` |
| `get_real_open_trades` | Read | `GET /api/real-trading/trades/open` |
| `get_real_closed_trades` | Read | `GET /api/real-trading/trades/closed` |
| `get_real_trading_settings` | Read | `GET /api/real-trading/settings` |
| `get_real_orders` | Read | `GET /api/real-trading/orders` |
| `start_real_engine` | Write | `POST /api/real-trading/start` |
| `stop_real_engine` | Write | `POST /api/real-trading/stop` |
| `close_real_trade` | Write | `POST /api/real-trading/trades/{id}/close` |
| `update_real_trading_settings` | Write | `PUT /api/real-trading/settings` |
| `create_real_order` | Write | `POST /api/real-trading/orders` |
| `cancel_real_order` | Write | `DELETE /api/real-trading/orders/{id}` |
| `cancel_all_real_orders` | Write | `DELETE /api/real-trading/orders/all` |
| `update_real_position_sltp` | Write | `PUT /api/real-trading/positions/{symbol}/sltp` |

All write tools are marked with warnings in MCP descriptions (real money).

---

## Order Types

| Type | Binance constant | Use case |
|------|-----------------|---------|
| Market | `MARKET` | Immediate execution |
| Limit | `LIMIT` | Price-specific entry/exit |
| Stop Loss Limit | `STOP_LOSS_LIMIT` | Stop orders |
| Take Profit Limit | `TAKE_PROFIT_LIMIT` | Profit targets |

Prefer maker orders toggle: `prefer_maker_orders = false` by default.

---

## Key Events

`RealTradingEvent` enum broadcast via tokio channel:

- `OrderPlaced`, `OrderFilled`, `OrderPartiallyFilled`, `OrderCancelled`, `OrderRejected`
- `PositionOpened`, `PositionUpdated`, `PositionClosed { pnl }`
- `BalanceUpdated { asset, free, locked }`
- `CircuitBreakerOpened(reason)`, `CircuitBreakerClosed`
- `ReconciliationComplete { discrepancies }`
- `DailyLossLimitReached { loss, limit }`
- `SignalGenerated`, `SignalExecuted`, `SignalRejected`
- `CooldownActivated { consecutive_losses, cool_down_minutes }`
- `EngineStarted`, `EngineStopped`

---

## Troubleshooting

| Symptom | Likely cause | Fix |
|---------|-------------|-----|
| Orders rejected on start | Circuit breaker open | Check logs for error count, wait for cooldown |
| Daily loss limit hit | Too many losses | Reset manually or wait for UTC midnight |
| Position not tracking | WebSocket disconnected | Check UserDataStreamManager logs |
| Reconciliation discrepancy | Engine state out of sync | Check /api/real-trading/status |
| "Symbol not allowed" | allowed_symbols set | Add symbol or clear list |
| High funding rate blocking trades | funding_spike_filter | Disable or raise threshold |

**Logs**: `rust-core-engine/logs/` or `docker logs rust-core-engine`

**Health**: `curl localhost:8080/api/real-trading/status`
