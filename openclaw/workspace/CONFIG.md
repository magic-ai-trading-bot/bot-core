# CONFIG.md - All Tunable Parameters
#
# ⚠️ CRITICAL: ALL values below are INITIAL DEFAULTS from Rust code, NOT current live values!
# Live values change via self-tuning and manual adjustments. NEVER quote these as "current".
#
# ALWAYS QUERY BEFORE DECIDING:
#   botcore get_paper_symbols          → per-symbol SL, TP, leverage, position_size (ENGINE USES THESE)
#   botcore get_paper_basic_settings   → global risk settings
#   botcore get_paper_indicator_settings → strategy thresholds (RSI/MACD/BB/Stoch)
#   botcore get_tuning_dashboard       → full overview

## Quick Reference — Most Important Settings

> ⚠️ **Defaults below ≠ live values.** Query `get_paper_symbols` + `get_paper_basic_settings` for LIVE values.

| Parameter | Default | Range | API Endpoint |
|-----------|---------|-------|-------------|
| `initial_balance` | 10,000 USDT | > 0 | PUT /basic-settings |
| `default_leverage` | 10x | 1-125 | PUT /basic-settings |
| `default_position_size_pct` | 5.0% | - | PUT /basic-settings |
| `max_positions` | 1 | - | PUT /basic-settings |
| `default_stop_loss_pct` | 10.0% | - | PUT /basic-settings |
| `default_take_profit_pct` | 20.0% | - | PUT /basic-settings |
| `daily_loss_limit_pct` | 3.0% | - | PUT /basic-settings |
| `max_consecutive_losses` | 3 | - | PUT /basic-settings |
| `cool_down_minutes` | 60 | - | PUT /basic-settings |
| `correlation_limit` | 0.7 (70%) | - | PUT /basic-settings |
| `min_ai_confidence` | 0.6 | 0-1 | PUT /strategy-settings |
| `signal_refresh_interval` | 15 min | > 0 | PUT /signal-interval |

## Risk Settings (Full)

> ⚠️ Query `botcore get_paper_basic_settings` for LIVE values. Per-symbol overrides via `get_paper_symbols`.

| Parameter | Default | Description |
|-----------|---------|-------------|
| `max_risk_per_trade_pct` | 0.5% | Max risk per single trade |
| `max_portfolio_risk_pct` | 10.0% | Max total portfolio risk |
| `default_stop_loss_pct` | 10.0% | Auto stop loss |
| `default_take_profit_pct` | 20.0% | Auto take profit |
| `max_leverage` | 10x | Max allowed leverage |
| `min_margin_level` | 300% | Liquidation warning level |
| `max_drawdown_pct` | 10.0% | Auto-stop if drawdown exceeds |
| `daily_loss_limit_pct` | 3.0% | Daily loss limit |
| `max_consecutive_losses` | 3 | Triggers cool-down |
| `cool_down_minutes` | 60 | Block duration after streak |
| `correlation_limit` | 0.7 | Max 70% same direction (only enforced with 3+ open positions; with 1-2 positions check is skipped) |
| `min_risk_reward_ratio` | 3.0 | Min RR ratio required |
| `dynamic_sizing` | true | Volatility-adjusted sizing |
| `trailing_stop_enabled` | true | Trailing stop loss |
| `trailing_stop_pct` | 0.8% | Trailing distance |
| `trailing_activation_pct` | 1.0% | Min profit to activate trailing |
| `enable_signal_reversal` | true | Auto reverse positions |
| `ai_auto_enable_reversal` | true | AI decides when to reverse |
| `reversal_min_confidence` | 0.65 | Min confidence for reversal |
| `reversal_max_pnl_pct` | 10.0% | Max PnL before disabling reversal |

## ATR-Based Position Sizing

> ⚠️ Query `botcore get_paper_basic_settings` for LIVE values. Disabled by default.

When `atr_stop_enabled = true`, position size and SL/TP distances are computed from the ATR indicator instead of fixed percentages. This makes sizing volatility-adaptive: tighter stops in calm markets, wider stops in volatile markets.

| Parameter | Type | Default | Description | Command to change |
|-----------|------|---------|-------------|-------------------|
| `atr_stop_enabled` | bool | false | Master toggle for ATR-based sizing | `botcore update_paper_basic_settings '{"settings":{"risk":{"atr_stop_enabled":true}}}'` |
| `atr_period` | number | 14 | ATR lookback period (candles) | `botcore update_paper_basic_settings '{"settings":{"risk":{"atr_period":14}}}'` |
| `atr_stop_multiplier` | number | 1.2 | Stop loss = ATR × this value | `botcore update_paper_basic_settings '{"settings":{"risk":{"atr_stop_multiplier":1.2}}}'` |
| `atr_tp_multiplier` | number | 2.4 | Take profit = ATR × this value | `botcore update_paper_basic_settings '{"settings":{"risk":{"atr_tp_multiplier":2.4}}}'` |
| `base_risk_pct` | number | 2.0% | Base risk per trade (% of equity) when ATR sizing is active | `botcore update_paper_basic_settings '{"settings":{"risk":{"base_risk_pct":2.0}}}'` |

**How it works**: Position size = `(equity × base_risk_pct) / (ATR × atr_stop_multiplier × leverage)`. Ensures consistent dollar risk per trade regardless of volatility.

**Diagnostic tool**: `botcore get_atr_diagnostics` — shows current ATR values, multipliers, and computed position sizes per active symbol.

## Half-Kelly Criterion Position Sizing

> ⚠️ Query `botcore get_paper_basic_settings` for LIVE values. Disabled by default. Requires minimum trade history.

When `kelly_enabled = true`, position sizes are dynamically scaled using the Kelly Criterion formula based on historical win rate and average win/loss ratio. Uses half-Kelly for safety.

| Parameter | Type | Default | Description | Command to change |
|-----------|------|---------|-------------|-------------------|
| `kelly_enabled` | bool | false | Master toggle for Kelly Criterion sizing | `botcore update_paper_basic_settings '{"settings":{"risk":{"kelly_enabled":true}}}'` |
| `kelly_min_trades` | number | 200 | Minimum closed trades before Kelly activates | `botcore update_paper_basic_settings '{"settings":{"risk":{"kelly_min_trades":200}}}'` |
| `kelly_fraction` | number | 0.5 | Fraction of full Kelly to use (0.5 = half-Kelly) | `botcore update_paper_basic_settings '{"settings":{"risk":{"kelly_fraction":0.5}}}'` |
| `kelly_lookback` | number | 100 | Number of recent trades used to compute win rate / avg R | `botcore update_paper_basic_settings '{"settings":{"risk":{"kelly_lookback":100}}}'` |

**Formula**: `Kelly% = (win_rate × avg_win − (1 − win_rate) × avg_loss) / avg_win`, then multiply by `kelly_fraction`. Engine falls back to `default_position_size_pct` when trade count < `kelly_min_trades`.

## Regime Filters

> ⚠️ Query `botcore get_paper_basic_settings` for LIVE values. All filters disabled by default.

Regime filters reduce or block position sizing when adverse market conditions are detected. Each filter applies a multiplier to the computed position size (not a hard stop).

### Funding Rate Spike Filter

Reduces position size when Binance perpetual funding rate exceeds threshold (extreme funding = crowded trade, elevated reversal risk).

| Parameter | Type | Default | Description | Command to change |
|-----------|------|---------|-------------|-------------------|
| `funding_spike_filter_enabled` | bool | false | Enable funding spike filter | `botcore update_paper_basic_settings '{"settings":{"risk":{"funding_spike_filter_enabled":true}}}'` |
| `funding_spike_threshold` | number | 0.0003 | Funding rate threshold (0.0003 = 0.03% per 8h) | `botcore update_paper_basic_settings '{"settings":{"risk":{"funding_spike_threshold":0.0003}}}'` |
| `funding_spike_reduction` | number | 0.5 | Position size multiplier when spike detected (0.5 = 50% of normal) | `botcore update_paper_basic_settings '{"settings":{"risk":{"funding_spike_reduction":0.5}}}'` |

### ATR Spike Filter

Reduces position size when current ATR is unusually high relative to recent average (extreme volatility regime).

| Parameter | Type | Default | Description | Command to change |
|-----------|------|---------|-------------|-------------------|
| `atr_spike_filter_enabled` | bool | false | Enable ATR spike filter | `botcore update_paper_basic_settings '{"settings":{"risk":{"atr_spike_filter_enabled":true}}}'` |
| `atr_spike_multiplier` | number | 2.0 | ATR is "spiked" when current ATR > avg ATR × this value | `botcore update_paper_basic_settings '{"settings":{"risk":{"atr_spike_multiplier":2.0}}}'` |
| `atr_spike_reduction` | number | 0.5 | Position size multiplier when ATR spike detected (0.5 = 50% of normal) | `botcore update_paper_basic_settings '{"settings":{"risk":{"atr_spike_reduction":0.5}}}'` |

### Consecutive Loss Reduction

Progressively reduces position size after consecutive losses (before the cool-down threshold is reached).

| Parameter | Type | Default | Description | Command to change |
|-----------|------|---------|-------------|-------------------|
| `consecutive_loss_reduction_enabled` | bool | false | Enable consecutive loss size reduction | `botcore update_paper_basic_settings '{"settings":{"risk":{"consecutive_loss_reduction_enabled":true}}}'` |
| `consecutive_loss_reduction_pct` | number | 0.3 | Reduce position size by this fraction per extra loss beyond threshold (0.3 = 30% reduction) | `botcore update_paper_basic_settings '{"settings":{"risk":{"consecutive_loss_reduction_pct":0.3}}}'` |
| `consecutive_loss_reduction_threshold` | number | 3 | Number of consecutive losses before reduction kicks in | `botcore update_paper_basic_settings '{"settings":{"risk":{"consecutive_loss_reduction_threshold":3}}}'` |

### Weekly Drawdown Limit

Halts all new trades if portfolio drawdown within the current week exceeds the limit.

| Parameter | Type | Default | Description | Command to change |
|-----------|------|---------|-------------|-------------------|
| `weekly_drawdown_limit_pct` | number | 7.0% | Max weekly drawdown before trading is suspended until next week | `botcore update_paper_basic_settings '{"settings":{"risk":{"weekly_drawdown_limit_pct":7.0}}}'` |

## Execution Settings

| Parameter | Default | Description |
|-----------|---------|-------------|
| `auto_execution` | true | Auto-execute signals |
| `execution_delay_ms` | 100 | Simulated latency (ms) |
| `simulate_slippage` | true | Enable slippage |
| `max_slippage_pct` | 0.05% | Max slippage |
| `simulate_market_impact` | false | Market impact model |
| `simulate_partial_fills` | false | Partial fill simulation |
| `partial_fill_probability` | 0.1 | 10% chance |
| `order_expiration_minutes` | 60 | Auto-cancel stale orders |
| `price_update_frequency_seconds` | 1 | Price refresh rate |

## Strategy Settings

> ⚠️ Query `botcore get_paper_indicator_settings` for LIVE strategy thresholds.

| Parameter | Default | Description |
|-----------|---------|-------------|
| `min_ai_confidence` | 0.6 | Min confidence to trade |
| `combination_method` | AIEnsemble | How strategies combine signals |
| `signal_timeout_minutes` | 30 | Cancel old signals |
| `enable_market_regime_detection` | true | Detect trending/ranging/volatile |
| `enable_optimization` | true | Auto-optimize parameters |
| `optimization_period_days` | 30 | Lookback for optimization |
| `min_trades_for_optimization` | 50 | Min trades before optimizing |

### Per-Strategy Parameters

> ⚠️ Thresholds below are code defaults. Query `botcore get_paper_indicator_settings` for LIVE values.

**RSI** (rsi_strategy.rs):
- `rsi_period`: 14, `oversold_threshold`: 30.0, `overbought_threshold`: 70.0
- `extreme_oversold`: 20.0, `extreme_overbought`: 80.0

**MACD** (macd_strategy.rs):
- `fast_period`: 12, `slow_period`: 26, `signal_period`: 9
- `histogram_threshold`: 0.001

**Bollinger** (bollinger_strategy.rs):
- `bb_period`: 20, `bb_multiplier`: 2.0, `squeeze_threshold`: 0.02

**Volume** (volume_strategy.rs):
- `volume_sma_period`: 20, `volume_spike_threshold`: 2.0
- `price_volume_correlation_period`: 10

**Stochastic** (stochastic_strategy.rs):
- `k_period`: 14, `d_period`: 3
- `oversold_threshold`: 20.0, `overbought_threshold`: 80.0
- `extreme_oversold`: 10.0, `extreme_overbought`: 90.0



| Parameter | Default | Range | Description |
|-----------|---------|-------|-------------|
| `rsi_period` | 14 | 5-50 | RSI calculation period |
| `macd_fast` | 12 | < slow | MACD fast EMA |
| `macd_slow` | 26 | > fast | MACD slow EMA |
| `macd_signal` | 9 | 1-20 | MACD signal line |
| `ema_periods` | [9, 21, 50] | - | EMA trend periods |
| `bollinger_period` | 20 | 5-50 | BB period |
| `bollinger_std` | 2.0 | 1.0-4.0 | BB std dev multiplier |
| `volume_sma_period` | 20 | - | Volume SMA |
| `stochastic_k_period` | 14 | 5-30 | Stochastic %K |
| `stochastic_d_period` | 3 | - | Stochastic %D |

## Signal Generation Settings

| Parameter | Default | Description |
|-----------|---------|-------------|
| `trend_threshold_percent` | 0.8% | Price movement threshold |
| `min_required_timeframes` | 3 of 4 | Timeframes must agree. Strategies use 5M (primary) + 15M (confirmation). 1H for AI bias. |
| `min_required_indicators` | 4 of 5 | Indicators must agree per timeframe |
| `confidence_base` | 0.5 | Base confidence |
| `confidence_per_timeframe` | 0.08 | Added per agreeing timeframe |

## AI Settings

| Parameter | Default | Description |
|-----------|---------|-------------|

| `request_timeout_seconds` | 30 | AI request timeout |
| `signal_refresh_interval_minutes` | 15 | How often to get new AI signals |
| `enable_realtime_signals` | true | Real-time signal updates |
| `enable_feedback_learning` | true | AI learns from trade results |
| `feedback_delay_hours` | 4 | Wait before sending feedback |

## Notification Settings

| Parameter | Default | Description |
|-----------|---------|-------------|
| `enable_trade_notifications` | true | Notify on trades |
| `enable_risk_warnings` | true | Notify on risk events |
| `daily_summary` | true | Daily performance summary |
| `weekly_report` | true | Weekly performance report |
| `channels` | [WebSocket] | WebSocket, Email, Telegram, Discord, Webhook |
| `max_notifications_per_hour` | 20 | Rate limit |

## Symbol-Specific Overrides

> ⚠️ Per-symbol settings OVERRIDE global defaults. Query `botcore get_paper_symbols` for actual values engine uses.

Each symbol (BTCUSDT, ETHUSDT, etc.) can override:
- `enabled`, `leverage`, `position_size_pct`, `stop_loss_pct`, `take_profit_pct`
- `trading_hours` (UTC), `min_price_movement_pct`, `max_positions`

API: `GET|PUT /api/paper-trading/symbols`

## Environment Variables (Key ones)

| Variable | Default | Description |
|----------|---------|-------------|
| `BINANCE_TESTNET` | true | Use testnet (KEEP TRUE for safety!) |
| `TRADING_ENABLED` | false | Enable trading (KEEP FALSE unless sure!) |

| `DATABASE_URL` | mongodb://... | MongoDB connection |
| `BINANCE_API_KEY` | - | Binance API key |
| `BINANCE_SECRET_KEY` | - | Binance secret |
| `ANTHROPIC_API_KEY` | - | Claude API key (for OpenClaw) |
| `TELEGRAM_BOT_TOKEN` | - | Telegram bot token |
| `TELEGRAM_USER_ID` | - | Telegram user ID |
