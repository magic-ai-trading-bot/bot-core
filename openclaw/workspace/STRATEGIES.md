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

### 1. RSI Strategy (Period: 14, Multi-timeframe: 1H + 4H)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | RSI1h ≤ 25.0 AND RSI4h ≤ 30.0 AND RSI recovering (prev < current) | 0.87 |
| **Strong SELL** | RSI1h ≥ 75.0 AND RSI4h ≥ 70.0 AND RSI declining | 0.87 |
| **Moderate BUY** | RSI1h ≤ 30.0 AND RSI4h < 50 AND RSI recovering | 0.73 |
| **Moderate SELL** | RSI1h ≥ 70.0 AND RSI4h > 50 AND RSI declining | 0.73 |
| **Weak BUY** | RSI1h 30.0-50 AND rising AND RSI4h < 50 | 0.51 |
| **Weak SELL** | RSI1h 50-70.0 AND falling AND RSI4h > 50 | 0.51 |

**Win rate**: 65%
**Common failure**: RSI oversold ≠ immediate bounce trong bear trend. Cần confirm trend reversal trước.

### 2. MACD Strategy (Fast: 12, Slow: 26, Signal: 9)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | Bullish crossover + histogram4h > 0.001 + both increasing | 0.89 |
| **Strong SELL** | Bearish crossover + histogram4h < -0.001 + both decreasing | 0.89 |
| **Moderate BUY** | Crossover + 4H histogram increasing, OR both histograms positive + increasing | 0.71 |
| **Moderate SELL** | Crossover + 4H decreasing, OR both negative + decreasing | 0.71 |
| **Weak BUY** | histogram1h increasing AND MACD > Signal AND momentum growing >10% | 0.55 |

**Win rate**: 61%
**Key**: Crossover = prev_MACD ≤ prev_Signal AND current_MACD > current_Signal

### 3. Bollinger Bands Strategy (Period: 20, StdDev: 2.0)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Squeeze Breakout BUY** | Squeeze (BB width < 2.00%) + expanding + price > upper + 4H position > 0.5 | 0.87 |
| **Mean Reversion BUY** | BB position ≤ 0.1 + 4H position < 0.3 + NOT expanding | 0.73 |
| **Trend Continuation BUY** | BB position > 0.8 + 4H > 0.6 + expanding | 0.69 |
| **Moderate BUY** | BB position < 0.25 + price > 4H middle band | 0.58 |

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

### 5. Stochastic Oscillator Strategy (K Period: 14, D Period: 3, Multi-timeframe: 1H + 4H)

| Signal | Condition | Confidence |
|--------|-----------|------------|
| **Strong BUY** | Bullish crossover (%K crosses above %D) + K1h ≤ 20.0 + K4h ≤ 20.0 | 0.89 |
| **Extreme BUY** | K1h ≤ 15.0 (extreme oversold) + K4h ≤ 20.0 + K1h > D1h | 0.85 |
| **Strong SELL** | Bearish crossover (%K crosses below %D) + K1h ≥ 80.0 + K4h ≥ 80.0 | 0.89 |
| **Extreme SELL** | K1h ≥ 85.0 (extreme overbought) + K4h ≥ 80.0 + K1h < D1h | 0.85 |
| **Moderate BUY** | Bullish crossover + K1h ≤ 20.0+10 + K4h < 50 | 0.72 |
| **Moderate SELL** | Bearish crossover + K1h ≥ 80.0-10 + K4h > 50 | 0.72 |
| **Weak BUY** | K1h > D1h + K1h < 50 + K4h < 50 + K rising | 0.52 |
| **Weak SELL** | K1h < D1h + K1h > 50 + K4h > 50 + K falling | 0.52 |

**Win rate**: 64%
**Thresholds**: Oversold = 20.0, Overbought = 80.0, Extreme = 15.0/85.0
**Common failure**: Stochastic crossover trong sideways market tạo nhiều false signals. Cần confirm với volume hoặc trend.

---

## Strategy Orchestration

- **5 strategies** run in parallel: RSI, MACD, Bollinger, Volume, Stochastic
- **Minimum agreement**: 4/5 strategies must agree on direction
- **Combination modes**: WeightedAverage (default), Consensus, BestConfidence, Conservative
- **Multi-timeframe**: All strategies analyze both 1H and 4H candles
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
- Multi-timeframe: 1H + 4H candles
- Execution: slippage ON, 100ms delay, market impact OFF, partial fills OFF
