# Paper Trading System

## Quick Reference

### Code Locations
```
rust-core-engine/src/paper_trading/
├── engine.rs (12,793 lines)
│   ├── Line 1242: process_trading_signal() - Risk checks entry point
│   ├── Line 1677: apply_slippage() - Random slippage 0-0.05%
│   ├── Line 1714: calculate_market_impact() - Size-based price impact
│   ├── Line 1758: simulate_partial_fill() - 10% probability, 30-90% fill
│   ├── Line 1977: check_daily_loss_limit() - 3% max daily loss
│   ├── Line 2022: is_in_cooldown() - 60 min after 3 losses
│   ├── Line 2041: update_consecutive_losses() - Auto-reset on profit
│   ├── Line 2089: check_position_correlation() - 70% max directional
│   ├── Line 2624: execute_trade() - Full execution simulation
│   ├── Line 3344: close_trade() - Consecutive loss tracking
│   ├── Line 967:  Trailing stop loss update (FR-RISK-007/008)
│   ├── Line 1008: Pending stop-limit order checks (FR-PAPER-003)
│   ├── Line 1021: ATR-based stop loss (FR-RISK-010)
│   ├── Line 1053: Half-Kelly position sizing (FR-RISK-011)
│   ├── Line 1132: Regime filters — funding/ATR spike, loss reduction
│   ├── Line 1190: Weekly drawdown limit (FR-RISK-012)
│   ├── Line 510-530: Choppy market detection - 4+ flips in 15 min
│   └── Line 562+: AI bias check - signal vs AI market bias alignment
├── portfolio.rs
│   ├── Lines 79-87: Cool-down state fields
│   │   ├── consecutive_losses: u32 (line 79)
│   │   ├── cool_down_until: Option<DateTime<Utc>> (line 82)
│   │   └── week_start_equity: Option<(DateTime<Utc>, f64)> (line 87)
│   └── Lines 229-231: Initialization
├── trade.rs
│   ├── Lines 147-158: Latency & trailing stop fields
│   │   ├── signal_timestamp: Option<DateTime<Utc>> (line 147)
│   │   ├── execution_timestamp: DateTime<Utc> (line 150)
│   │   ├── execution_latency_ms: Option<u64> (line 153)
│   │   └── highest_price_achieved: Option<f64> (line 158) - trailing stop tracking
│   └── Lines 233-236: Initialization
└── settings.rs
    └── PaperTradingSettings struct - all configuration (defaults at line 749)
```

### API Endpoints
- `POST /api/paper-trading/start` - Start paper trading session
- `POST /api/paper-trading/stop` - Stop paper trading session
- `GET /api/paper-trading/status` - Get current status
- `GET /api/paper-trading/portfolio` - Get portfolio state
- `GET /api/paper-trading/trades` - List all trades
- `POST /api/paper-trading/settings` - Update settings

### Database Collections
- `paper_portfolios` - Portfolio state and performance
- `paper_trades` - All executed trades
- `paper_signals` - AI trading signals

### Tests
- `rust-core-engine/tests/test_paper_trading.rs` - Integration tests
- Unit tests inline in engine.rs

---

## Features Overview

### Phase 1: Execution Realism (5 features)
1. **Random Slippage** - Simulates price variance during execution
2. **Execution Delay** - 100ms network latency + price re-fetch
3. **Market Impact** - Large orders affect execution price
4. **Partial Fills** - 10% chance of partial order fill
5. **Price Re-fetch** - Get new price after delay (price may move)

### Phase 2: Risk Management (core 4 features)
6. **Daily Loss Limit** - Stops trading at 3% daily loss
7. **Cool-Down** - 60-minute pause after 3 consecutive losses
8. **Correlation Limits** - Blocks trades if >70% same direction
9. **Consecutive Tracking** - Auto-reset counter on profitable trade

### Phase 3: Advanced Risk Features
10. **Signal Confirmation** - Requires 2 consecutive same-direction signals within 10 min (line 635)
11. **Choppy Market Detection** - 4+ direction flips in 15 min blocks trades (line 510)
12. **AI Bias Check** - Validates signal alignment with AI market bias (line 562)
13. **Trailing Stop Loss** - `highest_price_achieved` tracks peak price (line 967)
14. **Pending Stop-Limit Orders** - FR-PAPER-003, checked on every price update (line 1008)
15. **ATR-based Stop Loss** - Dynamic stop based on volatility (line 1021)
16. **Half-Kelly Position Sizing** - FR-RISK-011, risk-adjusted sizing (line 1053)
17. **Regime Filters** - Funding spike, ATR spike, consecutive loss reduction (line 1132)
18. **Weekly Drawdown Limit** - FR-RISK-012, limits weekly capital loss (line 1190)

### Phase 4: Performance Metrics
19. **Execution Latency** - Tracks signal-to-execution time

---

## Settings Configuration

### Execution Settings
```rust
execution: ExecutionSettings {
    execution_delay_ms: 100,           // Network latency simulation
    simulate_slippage: true,            // Enable slippage
    max_slippage_pct: 0.05,             // Max 0.05% slippage
    simulate_market_impact: false,      // Disabled by default
    market_impact_factor: 0.001,        // 0.1% per $10M order
    simulate_partial_fills: false,      // Disabled by default
    partial_fill_probability: 0.1,      // 10% chance
}
```

### Risk Settings (defaults at settings.rs:749)
```rust
risk: RiskSettings {
    daily_loss_limit_pct: 3.0,          // Stop at 3% daily loss (optimized from 5%)
    max_consecutive_losses: 3,          // Cool-down after 3 losses (optimized from 5)
    cool_down_minutes: 60,              // 60-minute cool-down
    correlation_limit: 0.7,             // Max 70% directional
}
```

### Update Settings via API
```bash
curl -X POST http://localhost:8080/api/paper-trading/settings \
  -H "Content-Type: application/json" \
  -d '{
    "execution": {
      "simulate_slippage": true,
      "simulate_market_impact": true,
      "simulate_partial_fills": true
    }
  }'
```

---

## Common Tasks

### Monitor Trade Execution
```bash
docker logs -f rust-core-engine-dev | grep -E "💸|⏳|📊|⚠️|🛑|🧊|⚡"

# Expected logs:
# ⏳ Simulating execution delay: 100ms
# 💸 Slippage applied: 50000.00 -> 50025.50 (0.0510%)
# 📊 Market impact for BTCUSDT: 0.0100%
# ⚠️ Partial fill: requested 0.100000, filled 0.085000 (85.0%)
# ⚡ Execution latency: 150ms
```

### Check Daily Loss Limit
**Code**: `engine.rs:1977`
```rust
async fn check_daily_loss_limit(&self) -> Result<bool>
```
**Log**: `🛑 DAILY LOSS LIMIT REACHED: 3.20% (limit: 3.00%)`

### Check Cool-Down Status
**Code**: `engine.rs:2022`
```rust
async fn is_in_cooldown(&self) -> bool
```
**Log**: `🧊 In cool-down period until 2026-03-03 11:52:00 UTC`

---

## How It Works

### Trade Execution Flow

```
1. AI Signal Received
   |
2. Signal Confirmation Check (engine.rs:635)
   |- Requires 2 consecutive same-direction signals within 10 min
   |
3. Choppy Market Check (engine.rs:510)
   |- Blocks if 4+ direction flips in 15 min
   |
4. AI Bias Alignment Check (engine.rs:562)
   |- Validates signal vs AI market bias
   |
5. Risk Checks in process_trading_signal() (engine.rs:1242)
   |- Daily loss limit (engine.rs:1977)
   |- Weekly drawdown limit (engine.rs:1190)
   |- Cool-down check (engine.rs:2022)
   |- Correlation check (engine.rs:2089)
   |- Regime filters (engine.rs:1132)
   |- ATR-based stop loss calculation (engine.rs:1021)
   |- Half-Kelly position sizing (engine.rs:1053)
   |
6. Execution Simulation in execute_trade() (engine.rs:2624)
   |- Step 1: Delay 100ms
   |- Step 2: Re-fetch current price
   |- Step 3: Calculate market impact
   |- Step 4: Apply slippage
   |- Step 5: Simulate partial fill
   |
7. Create Trade with realistic execution price
   |
8. Track latency, trailing stop, pending stop-limit orders
   |
9. On close: Update consecutive losses (engine.rs:3344)
```

### Risk Management Flow

```
Daily Loss Limit (3%):
  starting_equity: $10,000
  current_equity: $9,680
  daily_loss: $320 (3.2%)
  -> BLOCK new trades

Cool-Down (3 losses):
  Loss 1: consecutive_losses = 1
  Loss 2: consecutive_losses = 2
  Loss 3: consecutive_losses = 3
  -> COOL-DOWN 60 min

  Profit: +$100
  -> consecutive_losses = 0
  -> Cool-down cleared

Correlation Limit:
  Position 1: LONG BTCUSDT
  Position 2: LONG ETHUSDT
  Position 3: LONG BNBUSDT
  Total long exposure: 75%
  -> BLOCK new LONG trades
  -> ALLOW SHORT trades
```

---

## Related Documentation

- **Code Specs**: `specs/01-requirements/1.1-functional-requirements/FR-PAPER-TRADING.md`
- **Design**: `specs/02-design/2.5-components/COMP-RUST-TRADING.md`
- **Test Cases**: `specs/03-testing/3.2-test-cases/TC-TRADING.md`

---

**Last Updated**: 2026-03-03
**Status**: Production-ready
**Engine**: 12,793 lines, 19 features
