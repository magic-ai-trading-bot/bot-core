# STRATEGIES.md - Deep Trading System Knowledge
# DO NOT run sync-openclaw-knowledge.sh — it would overwrite with code defaults.
#
# ⚠️ THRESHOLDS & RISK VALUES ARE RUNTIME-TUNABLE — numbers below are REFERENCE ONLY.
# ALWAYS query `botcore get_paper_indicator_settings` for live strategy thresholds.
# ALWAYS query `botcore get_paper_basic_settings` for live risk values.
# ALWAYS query `botcore get_paper_symbols` for per-symbol SL/TP/leverage/position_size.

## Strategy Signal Generation

> ⚠️ **Threshold values in signal tables below are REFERENCE patterns showing signal LOGIC.**
> Actual thresholds (oversold/overbought/periods) change via self-tuning.
> Query `botcore get_paper_indicator_settings` for LIVE values before making decisions.

### 1. RSI Strategy (Period: 14, Multi-timeframe: 5M + 15M)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | RSI_primary ≤ 20.0 AND RSI_confirm ≤ 30.0 AND RSI recovering (prev < current) | 0.87 |
| **Strong SELL** | RSI_primary ≥ 80.0 AND RSI_confirm ≥ 70.0 AND RSI declining | 0.87 |
| **Moderate BUY** | RSI_primary ≤ 30.0 AND RSI_confirm < 50 AND RSI recovering | 0.73 |
| **Moderate SELL** | RSI_primary ≥ 70.0 AND RSI_confirm > 50 AND RSI declining | 0.73 |
| **Weak BUY** | RSI_primary 30.0-50 AND rising AND RSI_confirm < 50 | 0.51 |
| **Weak SELL** | RSI_primary 50-70.0 AND falling AND RSI_confirm > 50 | 0.51 |

**Win rate**: 65%
**Common failure**: RSI oversold ≠ immediate bounce trong bear trend. Cần confirm trend reversal trước.

### 2. MACD Strategy (Fast: 12, Slow: 26, Signal: 9)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | Bullish crossover + histogram_confirm > 0.001 + both increasing | 0.89 |
| **Strong SELL** | Bearish crossover + histogram_confirm < -0.001 + both decreasing | 0.89 |
| **Moderate BUY** | Crossover + confirmation histogram increasing, OR both histograms positive + increasing | 0.71 |
| **Moderate SELL** | Crossover + 4H decreasing, OR both negative + decreasing | 0.71 |
| **Weak BUY** | histogram_primary increasing AND MACD > Signal AND momentum growing >10% | 0.55 |

**Win rate**: 61%
**Key**: Crossover = prev_MACD ≤ prev_Signal AND current_MACD > current_Signal

### 3. Bollinger Bands Strategy (Period: 20, StdDev: 2.0)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Squeeze Breakout BUY** | Squeeze (BB width < 2.00%) + expanding + price > upper + confirm_position > 0.5 | 0.87 |
| **Mean Reversion BUY** | BB position ≤ 0.1 + confirm_position < 0.3 + NOT expanding | 0.73 |
| **Trend Continuation BUY** | BB position > 0.8 + confirm > 0.6 + expanding | 0.69 |
| **Moderate BUY** | BB position < 0.25 + price > confirm middle band | 0.58 |

**Win rate**: 63%
**BB Position** = (Price - Lower) / (Upper - Lower). 0 = at lower band, 1 = at upper band.

### 4. Volume Strategy (SMA Period: 20, Spike: 2.0x)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong Volume Surge BUY** | Volume spike (≥2.0x avg) + bullish ratio ≥ 0.7 + price > POC | 0.91 |
| **Accumulation BUY** | High volume (≥1.5x) + bullish ratio ≥ 0.6, OR near POC + ratio ≥ 0.65 | 0.71 |
| **Weak BUY** | Bullish ratio ≥ 0.55 + volume > 1.2x avg | 0.51 |

**Win rate**: 58%
**POC** = Point of Control (price level with highest trading volume in 20 periods)

### 5. Stochastic Oscillator Strategy (K Period: 14, D Period: 3, Multi-timeframe: 5M + 15M)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | Bullish crossover (%K crosses above %D) + K_primary ≤ 20.0 + K_confirm ≤ 20.0 | 0.89 |
| **Extreme BUY** | K_primary ≤ 10.0 (extreme oversold) + K_confirm ≤ 20.0 + K_primary > D_primary | 0.85 |
| **Strong SELL** | Bearish crossover (%K crosses below %D) + K_primary ≥ 80.0 + K_confirm ≥ 80.0 | 0.89 |
| **Extreme SELL** | K_primary ≥ 90.0 (extreme overbought) + K_confirm ≥ 80.0 + K_primary < D_primary | 0.85 |
| **Moderate BUY** | Bullish crossover + K_primary ≤ 20.0+10 + K_confirm < 50 | 0.72 |
| **Moderate SELL** | Bearish crossover + K_primary ≥ 80.0-10 + K_confirm > 50 | 0.72 |
| **Weak BUY** | K_primary > D_primary + K_primary < 50 + K_confirm < 50 + K rising | 0.52 |
| **Weak SELL** | K_primary < D_primary + K_primary > 50 + K_confirm > 50 + K falling | 0.52 |

**Win rate**: 64%
**Thresholds**: Oversold = 20.0, Overbought = 80.0, Extreme = 10.0/90.0
**Common failure**: Stochastic crossover trong sideways market tạo nhiều false signals. Cần confirm với volume hoặc trend.

---

## Strategy Orchestration

- **5 strategies** run in parallel: RSI, MACD, Bollinger, Volume, Stochastic
- **Minimum agreement**: 4/5 strategies must agree on direction
- **Combination modes**: WeightedAverage (default), Consensus, BestConfidence, Conservative
- **Multi-timeframe**: All strategies analyze 5M (primary) + 15M (confirmation) candles. 1H loaded for AI bias analysis only.
- **Minimum data**: 50 candles per timeframe required before trading starts
- **Hybrid filter**: Optional AI trend filter for additional validation

### Signal Pipeline (order of checks)

1. **Neutral filter**: Skip neutral signals
2. **Confidence filter**: Skip if confidence < min_confidence (0.60)
3. **Market direction filter**: `short_only_mode` → block Longs; `long_only_mode` → block Shorts (DYNAMIC — check via `get_paper_basic_settings`)
4. **Choppy market filter**: Skip if 4+ direction flips in 15 minutes for the symbol
5. **Signal confirmation**: Require 2 consecutive same-direction signals within 10 minutes (60s dedup)
6. **AI bias check**: Stricter for Longs (threshold -0.3) vs Shorts (threshold -0.5). Skip if `signal_dir × direction_bias < threshold`
7. **Trade execution**: Pass through risk management layers → execute trade

**Note on step 6**: Long signals blocked when bias even mildly bearish (> -0.3). Short signals only blocked when bias mildly bullish (< 0.5). This asymmetry protects against losing Long trades in bearish markets.

### Choppy Market Detection

Prevents trading in ranging/whipsaw markets:
- Tracks all non-neutral signals per symbol with timestamps
- Counts direction changes (Long→Short or Short→Long) in 15-min window
- If ≥4 flips → market is choppy → block ALL signals for that symbol
- Window auto-cleans entries >15 minutes old

---

## Risk Management - 7 Protection Layers

> ⚠️ **All values below are runtime-tunable via self-tuning and manual adjustments.**
> Query `botcore get_paper_basic_settings` for LIVE risk values.
> Query `botcore get_paper_symbols` for per-symbol SL/TP/leverage/position_size.

| Layer | Name | What it controls | How to query |
|-------|------|-----------------|--------------|
| 1 | **Position Size** | Max % equity per trade | `get_paper_symbols` → `position_size_pct` |
| 2 | **Stop Loss** | PnL-based auto close (NOT price%) | `get_paper_symbols` → `stop_loss_pct` |
| 3 | **Portfolio Risk** | Max total portfolio risk → blocks all new trades | `get_paper_basic_settings` → `max_portfolio_risk_pct` |
| 4 | **Daily Loss** | Daily loss limit → stops all trading for the day (reset UTC) | `get_paper_basic_settings` → `daily_loss_limit_pct` |
| 5 | **Consecutive Losses** | N consecutive losses → triggers cool-down. Reset on first win | `get_paper_basic_settings` → `max_consecutive_losses` |
| 6 | **Cool-Down** | Block all trading for N minutes after consecutive losses | `get_paper_basic_settings` → `cool_down_minutes` |
| 7 | **Correlation** | Max % exposure in one direction (skipped when < 3 positions) | `get_paper_basic_settings` → `correlation_limit` |

**Execution order**: Daily Loss → Cool-Down → Correlation → Portfolio Risk → Position Size + Stop Loss → Execute

---

## Trade Close Reasons (8 Types)

When analyzing closed trades via `get_paper_closed_trades`, the `close_reason` field tells you WHY the trade was closed:

| Close Reason | Meaning | Who Triggered |
|-------------|---------|---------------|
| **TakeProfit** | PnL reached TP threshold → auto-close with profit | Engine auto |
| **StopLoss** | PnL reached SL threshold → auto-close with loss | Engine auto |
| **TrailingStop** | Trailing stop activated, then price reversed past trail distance → auto-close | Engine auto |
| **Manual** | Closed by user or OpenClaw AI decision | You or user |
| **AISignal** | Signal reversal — new high-confidence signal in opposite direction | Engine auto (reversal) |
| **RiskManagement** | Risk layer triggered (daily loss, portfolio risk, etc.) | Engine auto |
| **MarginCall** | Price near liquidation level → emergency close | Engine auto |
| **TimeBasedExit** | Order expired (stale order timeout) | Engine auto |

### How to Interpret Trade History

- **TrailingStop** = trade WAS profitable (trailing activated at ≥1% PnL), then price reversed. Check if PnL is positive (locked profit) or negative (price dropped below entry after trail activated)
- **StopLoss** = trade hit the fixed SL level WITHOUT trailing ever activating (price never reached +1% PnL)
- **Manual** = likely YOU (OpenClaw) closed it proactively — check your own reasoning
- **AISignal** = signal reversal feature auto-closed to open opposite position

### Trailing Stop Mechanics (Current Behavior)

1. **Activation**: PnL-based — trailing activates when `unrealized_pnl% ≥ trailing_activation_pct`
2. **Trail distance**: Price-based — stop follows `trailing_stop_pct` below peak (Long) or above trough (Short)
3. **One-way ratchet**: Stop only moves in favorable direction, never moves back
4. **Close reason**: When trail triggers close → `close_reason = "TrailingStop"` (NOT "StopLoss")

Query `get_paper_basic_settings` for current `trailing_stop_pct`, `trailing_activation_pct`, `trailing_stop_enabled`.

---

## Execution Simulation

| Feature | Default | Detail |
|---------|---------|--------|
| **Slippage** | ON (0.05% max) | BUY: price × (1 + slippage%), SELL: price × (1 - slippage%) |
| **Execution Delay** | 100ms | Simulates network latency, re-fetches price after delay |
| **Market Impact** | OFF | impact = (order_value / typical_volume) × factor, capped 1% |
| **Partial Fills** | OFF | 10% chance, fills 30-90% of order |

---

## Common Loss Patterns & Solutions

| Pattern | Symptoms | Solution |
|---------|----------|----------|
| **False breakout** | Entry on Bollinger squeeze breakout, reverses | Wait for volume confirmation |
| **Counter-trend entry** | RSI oversold BUY in strong downtrend | Add EMA 50/200 trend filter |
| **Overtrading** | >20 trades/day, many small losses | Increase confidence threshold |
| **Late entry** | Enter after majority of move | Check MACD histogram declining = late |
| **Stop loss too tight** | Many SL-hit losses that recover | Widen based on ATR (1.5-2x) |
| **Correlated positions** | Same-direction trades lose together | Check correlation limit |
| **Cool-down panic** | Force trades after cool-down | Extend cool-down, reduce size |
| **Volume dry-up** | Low volume, high slippage | Skip signals volume < 0.8x avg |
| **Stochastic whipsaw** | Crossovers in sideways market | Combine with volume/trend filter |

---

## Key Configuration — How to Query

> ⚠️ **DO NOT rely on hardcoded values. ALWAYS query API for live settings.**

| Category | What to query | Tool |
|----------|---------------|------|
| **Risk (7 layers)** | SL, TP, position size, leverage, max positions | `botcore get_paper_symbols` |
| **Risk (global)** | Daily loss, consecutive losses, cool-down, correlation, portfolio risk | `botcore get_paper_basic_settings` |
| **Market direction** | short_only_mode, long_only_mode (DYNAMIC) | `botcore get_paper_basic_settings` |
| **Strategy thresholds** | RSI/MACD/BB/Stoch oversold/overbought/periods | `botcore get_paper_indicator_settings` |
| **Full overview** | All settings + performance + positions | `botcore get_tuning_dashboard` |

**Fixed architecture** (not tunable):
- 5 strategies run in parallel: RSI, MACD, Bollinger, Volume, Stochastic
- Minimum agreement: 4/5 strategies
- Multi-timeframe: 5M (primary) + 15M (confirmation) candles
- Execution: slippage ON, 100ms delay, market impact OFF, partial fills OFF
