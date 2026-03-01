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
- `extreme_oversold`: 25.0, `extreme_overbought`: 75.0

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
- `extreme_oversold`: 15.0, `extreme_overbought`: 85.0

## Indicator Settings (Shared with Python AI)

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
| `service_url` | http://python-ai-service:8000 | Python AI URL |
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
| `XAI_API_KEY` | - | xAI Grok API key (fallback: OPENAI_API_KEY) |
| `DATABASE_URL` | mongodb://... | MongoDB connection |
| `BINANCE_API_KEY` | - | Binance API key |
| `BINANCE_SECRET_KEY` | - | Binance secret |
| `ANTHROPIC_API_KEY` | - | Claude API key (for OpenClaw) |
| `TELEGRAM_BOT_TOKEN` | - | Telegram bot token |
| `TELEGRAM_USER_ID` | - | Telegram user ID |
