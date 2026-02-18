---
name: botcore
description: Control, monitor, and tune the BotCore cryptocurrency trading bot via MCP bridge.
metadata: {"openclaw":{"emoji":"🤖","requires":{"bins":["botcore"],"env":["MCP_URL","MCP_AUTH_TOKEN"]}}}
---

# BotCore Trading Bot Controller — 109 Tools

Run commands via `botcore` CLI:

```bash
botcore <tool_name> '<json-arguments>'
```

Arguments optional for tools with no input. Always single-quote JSON args.

---

## IMPORTANT: Environment Constraints

You are running inside a **Docker container**. These rules are MANDATORY:

1. **NO systemctl/service/journalctl** — There is no systemd in this container. Never use them.
2. **NO apt/yum/apk install** — Do not install packages. Use only what is available.
3. **NO crontab** — Cron jobs are managed by OpenClaw (`openclaw cron`), NOT Linux crontab.
4. **NO direct config file edits** — Config is mounted read-only. Use botcore tools to change settings.
5. **Gateway is managed by entrypoint.sh** — Do NOT try to start/stop/restart the gateway yourself.
6. **Available commands**: `botcore`, `node`, `curl`, `openclaw`. That is it.
7. **Send notifications via botcore** — Use `botcore send_telegram_notification` to send messages to Telegram. You can pass plain text or JSON.
8. **OpenClaw CLI requires special flags** — When using `openclaw` commands, you MUST include these flags:
   ```bash
   openclaw --dev <command> --url ws://localhost:18789 --token $OPENCLAW_GATEWAY_TOKEN
   ```
   Example: `openclaw --dev cron list --url ws://localhost:18789 --token $OPENCLAW_GATEWAY_TOKEN`
   Without these flags, you will get "pairing required" errors. NEVER run bare `openclaw cron` commands.

If a user asks to restart services, tell them to restart the Docker container from the host machine.

---

## 1. System Health & Monitoring (2 tools)

```bash
botcore check_system_health                              # All services health (Rust, Python, MongoDB)
botcore get_service_logs_summary '{"service":"all"}'      # Error/warning logs (all|rust|python)
```

## 2. Market Data (8 tools)

```bash
botcore get_market_prices                                 # Current prices all symbols
botcore get_market_overview                               # Market overview + stats
botcore get_candles '{"symbol":"BTCUSDT","timeframe":"1h","limit":24}'  # Candlestick data
botcore get_chart '{"symbol":"BTCUSDT","timeframe":"4h"}' # Chart + indicators
botcore get_multi_charts '{"symbols":["BTCUSDT","ETHUSDT"]}'  # Multiple charts
botcore get_symbols                                       # List tracked symbols
botcore add_symbol '{"symbol":"SOLUSDT"}'                 # Add symbol to track
botcore remove_symbol '{"symbol":"SOLUSDT"}'              # Remove symbol
```

Timeframes: `1m`, `5m`, `15m`, `30m`, `1h`, `4h`, `1d`

## 3. Paper Trading (34 tools)

### Read (18 tools)
```bash
botcore get_paper_trading_status        # Engine status (running/stopped, P&L, daily stats)
botcore get_paper_portfolio             # Balance, equity, margin, positions, metrics
botcore get_paper_open_trades           # All open positions with unrealized P&L
botcore get_paper_closed_trades         # All closed trades with realized P&L
botcore get_paper_strategy_settings     # Strategy config (RSI, MACD, BB, Volume)
botcore get_paper_basic_settings        # Balance, max positions, leverage, risk settings
botcore get_paper_execution_settings    # Execution: slippage, partial fills, market impact
botcore get_paper_ai_settings           # AI: service URL, signal interval, confidence thresholds
botcore get_paper_notification_settings # Notifications: trade/risk/performance alerts, reports
botcore get_paper_indicator_settings    # Indicator params (RSI period, MACD fast/slow/signal, BB)
botcore get_paper_symbols               # Symbols being traded
botcore get_paper_pending_orders        # Pending limit/stop orders
botcore get_paper_signals_history       # All strategy signals history
botcore get_paper_latest_signals        # Most recent signals
botcore get_paper_trade_analyses        # GPT-4 analyses for ALL closed trades
botcore get_paper_trade_analysis '{"trade_id":"trade_123"}'  # GPT-4 analysis for specific trade
botcore get_paper_config_suggestions    # All AI config suggestions
botcore get_paper_latest_config_suggestions  # Latest AI recommendations
```

### Write (17 tools)
```bash
botcore start_paper_engine              # Start trading engine
botcore stop_paper_engine               # Stop trading engine (positions stay open)
botcore reset_paper_account             # Reset to initial balance, close all
botcore close_paper_trade '{"trade_id":"trade_123"}'  # Close by trade ID
botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT"}'  # Close by symbol (PREFERRED)
botcore create_paper_order '{"symbol":"BTCUSDT","side":"buy","order_type":"market"}'
botcore cancel_paper_order '{"order_id":"order_123"}'
botcore trigger_paper_analysis          # Trigger GPT-4 trade analysis NOW
botcore update_paper_signal_interval '{"interval_seconds":300}'  # Signal generation interval
botcore update_paper_basic_settings '{"settings":{"initial_balance":10000,"max_positions":5}}'
botcore update_paper_execution_settings '{"settings":{"simulate_slippage":true}}'
botcore update_paper_ai_settings '{"settings":{"signal_refresh_interval_minutes":10}}'
botcore update_paper_notification_settings '{"settings":{"daily_summary":true}}'
botcore update_paper_strategy_settings '{"settings":{"rsi_enabled":true}}'
botcore update_paper_indicator_settings '{"settings":{"rsi_period":14,"rsi_oversold":25}}'
botcore update_paper_symbols '{"symbols":["BTCUSDT","ETHUSDT","BNBUSDT"]}'
botcore update_paper_settings '{"settings":{"any_field":"value"}}'  # Generic catch-all
```

### Settings Field Reference

**`update_paper_basic_settings`** — Basic + Risk settings (32 fields):

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| **Basic** | | | |
| initial_balance | number | Starting balance | 10000 |
| max_positions | number | Max open positions | 5 |
| default_position_size_pct | number | Position size % | 2.0 |
| default_leverage | number | Leverage multiplier | 3 |
| trading_fee_rate | number | Trading fee rate | 0.0004 |
| funding_fee_rate | number | Funding fee rate | 0.0001 |
| slippage_pct | number | Slippage simulation % | 0.01 |
| enabled | boolean | Enable/disable engine | true |
| auto_restart | boolean | Auto-restart after reset | false |
| **Risk** | | | |
| max_risk_per_trade_pct | number | Max risk per trade % | 1.0 |
| max_portfolio_risk_pct | number | Max portfolio risk % | 10.0 |
| default_stop_loss_pct | number | Stop loss % (PnL-based) | 5.0 |
| default_take_profit_pct | number | Take profit % (PnL-based) | 10.0 |
| max_leverage | number | Max allowed leverage | 5 |
| min_margin_level | number | Min margin level % | 300.0 |
| max_drawdown_pct | number | Max drawdown % | 10.0 |
| daily_loss_limit_pct | number | Daily loss limit % | 3.0 |
| max_consecutive_losses | number | Losses before cooldown | 3 |
| cool_down_minutes | number | Cooldown after losses (min) | 60 |
| trailing_stop_enabled | boolean | Enable trailing stop | true |
| trailing_stop_pct | number | Trailing stop distance % (price-based) | 3.0 |
| trailing_activation_pct | number | PnL % to activate trailing stop | 5.0 |
| position_sizing_method | string | Sizing method | "RiskBased" |
| min_risk_reward_ratio | number | Min risk/reward ratio | 2.0 |
| correlation_limit | number | Position correlation limit | 0.7 |
| dynamic_sizing | boolean | Dynamic sizing by volatility | true |
| volatility_lookback_hours | number | Volatility lookback hours | 24 |
| enable_signal_reversal | boolean | Auto-reverse on opposite signal | true |
| ai_auto_enable_reversal | boolean | AI decides reversal | true |
| reversal_min_confidence | number | Min confidence for reversal | 0.65 |
| reversal_max_pnl_pct | number | Max P&L % before trailing stop | 10.0 |
| reversal_allowed_regimes | array | Allowed regimes for reversal | ["trending","ranging"] |

`position_sizing_method` values: `FixedPercentage`, `RiskBased`, `VolatilityAdjusted`, `ConfidenceWeighted`, `Composite`

**`update_paper_execution_settings`** — Execution settings (10 fields):

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| auto_execution | boolean | Enable auto trade execution | true |
| execution_delay_ms | number | Execution delay (ms) | 100 |
| simulate_partial_fills | boolean | Enable partial fill sim | false |
| partial_fill_probability | number | Partial fill probability | 0.1 |
| order_expiration_minutes | number | Order expiration (min) | 60 |
| simulate_slippage | boolean | Enable slippage sim | true |
| max_slippage_pct | number | Max slippage % | 0.05 |
| simulate_market_impact | boolean | Enable market impact sim | false |
| market_impact_factor | number | Market impact factor | 0.001 |
| price_update_frequency_seconds | number | Price update freq (sec) | 1 |

**`update_paper_ai_settings`** — AI integration settings (9 fields):

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| service_url | string | Python AI service URL | "http://python-ai-service:8000" |
| request_timeout_seconds | number | Request timeout (sec) | 30 |
| signal_refresh_interval_minutes | number | Signal refresh interval (min) | 15 |
| enable_realtime_signals | boolean | Enable realtime signals | true |
| enable_feedback_learning | boolean | Enable AI feedback loop | true |
| feedback_delay_hours | number | Feedback delay (hours) | 4 |
| enable_strategy_recommendations | boolean | Enable AI strategy recs | true |
| track_model_performance | boolean | Track model performance | true |
| confidence_thresholds | object | Per-regime confidence | {"trending":0.65,"ranging":0.75} |

**`update_paper_notification_settings`** — Notification settings (7 fields):

| Field | Type | Description | Example |
|-------|------|-------------|---------|
| enable_trade_notifications | boolean | Notify on trades | true |
| enable_performance_notifications | boolean | Notify on performance | true |
| enable_risk_warnings | boolean | Notify on risk events | true |
| daily_summary | boolean | Daily summary report | true |
| weekly_report | boolean | Weekly performance report | true |
| min_pnl_notification | number | Min P&L to notify (abs) | 10.0 |
| max_notifications_per_hour | number | Rate limit per hour | 20 |

Examples:
```bash
# Set stop loss to 3%
botcore update_paper_basic_settings '{"settings":{"default_stop_loss_pct":3.0}}'
# Enable trailing stop at 2.5%, activate at 1.5% profit
botcore update_paper_basic_settings '{"settings":{"trailing_stop_enabled":true,"trailing_stop_pct":2.5,"trailing_activation_pct":1.5}}'
# Disable signal reversal
botcore update_paper_basic_settings '{"settings":{"enable_signal_reversal":false}}'
# Change position sizing method
botcore update_paper_basic_settings '{"settings":{"position_sizing_method":"VolatilityAdjusted"}}'
# Enable slippage simulation
botcore update_paper_execution_settings '{"settings":{"simulate_slippage":true,"max_slippage_pct":0.05}}'
# Change signal refresh interval
botcore update_paper_ai_settings '{"settings":{"signal_refresh_interval_minutes":10}}'
# Set per-regime confidence thresholds
botcore update_paper_ai_settings '{"settings":{"confidence_thresholds":{"trending":0.60,"volatile":0.85}}}'
# Disable daily summary, enable risk warnings
botcore update_paper_notification_settings '{"settings":{"daily_summary":false,"enable_risk_warnings":true}}'
```

**`update_paper_strategy_settings`** — Use for strategy enable/disable:
```bash
botcore update_paper_strategy_settings '{"settings":{"rsi_enabled":true,"macd_enabled":false}}'
```

**`update_paper_indicator_settings`** — Use for indicator parameters:
```bash
botcore update_paper_indicator_settings '{"settings":{"rsi_period":14,"rsi_oversold":25,"macd_fast":12}}'
```

## 4. Real Trading (14 tools) — REAL MONEY

### Read (6 tools)
```bash
botcore get_real_trading_status         # Engine status
botcore get_real_portfolio              # Balance & positions
botcore get_real_open_trades            # Open trades with P&L
botcore get_real_closed_trades          # Closed trades history
botcore get_real_trading_settings       # Risk params, strategy config
botcore get_real_orders                 # All active orders
```

### Write (8 tools) — EXTREME CAUTION
```bash
botcore start_real_engine               # Start real trading
botcore stop_real_engine                # Stop real trading
botcore close_real_trade '{"trade_id":"xyz"}'  # Close real trade at market
botcore create_real_order '{"symbol":"BTCUSDT","side":"buy","order_type":"market"}'
botcore cancel_real_order '{"id":"order_123"}'
botcore cancel_all_real_orders          # Cancel ALL pending orders
botcore update_real_trading_settings '{"settings":{"stop_loss_percent":3.0}}'
botcore update_real_position_sltp '{"symbol":"BTCUSDT","stop_loss":40000,"take_profit":50000}'
```

## 5. AI Analysis & ML (12 tools)

### Rust API (6 tools)
```bash
botcore analyze_market '{"symbol":"BTCUSDT","timeframe":"4h"}'  # GPT-4 market analysis
botcore get_strategy_recommendations '{"symbol":"BTCUSDT"}'     # AI strategy advice
botcore get_market_condition '{"symbol":"BTCUSDT"}'             # Bull/bear/neutral assessment
botcore send_ai_feedback '{"signal_id":"sig_123","feedback":"positive"}'
botcore get_ai_info                     # AI service capabilities
botcore get_ai_strategies               # Available AI strategies
```

### Python API (6 tools)
```bash
botcore get_ai_performance              # ML model accuracy metrics
botcore get_ai_cost_statistics          # OpenAI API cost breakdown
botcore get_ai_config_suggestions       # AI config optimization suggestions
botcore get_ai_analysis_history         # GPT-4 analysis history
botcore get_ai_storage_stats            # Model storage usage
botcore clear_ai_storage                # Clear AI cache
```

## 6. AI Tasks & Chat (7 tools)

```bash
botcore trigger_config_analysis         # Trigger AI config optimization (2 min)
botcore predict_trend '{"symbol":"BTCUSDT","timeframe":"4h"}'  # ML trend prediction
botcore get_ai_config_suggestions_python  # Config suggestions from Python
botcore chat_with_project '{"message":"How does the RSI strategy work?"}'
botcore get_chat_suggestions            # Suggested questions
botcore clear_chat_history              # Clear chat history
botcore get_ai_debug_info               # GPT-4 debug info
```

## 7. Monitoring (4 tools)

```bash
botcore get_system_monitoring           # CPU, memory, disk, network
botcore get_trading_metrics             # Win rate, PnL, active positions
botcore get_connection_status           # Binance, MongoDB, WebSocket status
botcore get_python_health               # Python AI service health
```

## 8. Live Trading (4 tools)

```bash
botcore get_trading_positions           # Current open positions with P&L
botcore get_trading_account             # Account balance, equity, margin
botcore get_trading_performance         # Win rate, PnL, Sharpe, max drawdown
botcore close_trading_position '{"symbol":"BTCUSDT"}'  # Close position by symbol
```

## 9. Settings (10 tools)

### API Keys (4 tools)
```bash
botcore get_api_keys                    # Status of configured keys (masked)
botcore save_api_keys '{"exchange":"binance","api_key":"...","secret_key":"..."}'
botcore test_api_keys                   # Test key connectivity
botcore delete_api_keys                 # Delete all stored keys
```

### Notifications (6 tools)
```bash
botcore get_notification_preferences    # Current notification settings
botcore update_notification_preferences '{"preferences":{"trade_notifications":true}}'
botcore test_notification               # Send test notification
botcore subscribe_push_notifications '{"subscription":{"endpoint":"..."}}'
botcore unsubscribe_push_notifications
botcore get_vapid_key                   # VAPID public key for push
```

## 10. Authentication (4 tools)

```bash
botcore login '{"email":"user@example.com","password":"..."}'
botcore register_user '{"email":"user@example.com","password":"...","name":"Dung"}'
botcore get_profile                     # Current user profile
botcore refresh_token                   # Refresh JWT token
```

## 11. Self-Tuning Engine (8 tools)

### Dashboard & Info
```bash
botcore get_tuning_dashboard            # Performance + settings + AI suggestions + positions
botcore get_parameter_bounds            # All tunable params with ranges by tier
botcore get_adjustment_history '{"limit":10}'  # Past parameter changes
```

### GREEN Tier (Auto-apply, notify user)
```bash
botcore apply_green_adjustment '{"parameter":"rsi_oversold","new_value":25,"reasoning":"Bear market"}'
botcore apply_green_adjustment '{"parameter":"rsi_overbought","new_value":75,"reasoning":"Reduce overbought"}'
botcore apply_green_adjustment '{"parameter":"signal_interval_minutes","new_value":10,"reasoning":"Low volatility"}'
botcore apply_green_adjustment '{"parameter":"confidence_threshold","new_value":0.70,"reasoning":"Higher quality signals"}'
```

### YELLOW Tier (Require user confirmation)
```bash
# Step 1: Request (returns confirm_token)
botcore request_yellow_adjustment '{"parameter":"stop_loss_percent","new_value":3.0,"reasoning":"Wider stop loss"}'
# Step 2: User approves → call again with token
botcore request_yellow_adjustment '{"parameter":"stop_loss_percent","new_value":3.0,"reasoning":"...","confirm_token":"TOKEN"}'
```

### RED Tier (Require explicit approval text)
```bash
# Step 1: Request (returns required approval text)
botcore request_red_adjustment '{"parameter":"max_daily_loss_percent","new_value":8.0,"reasoning":"Increase tolerance","risk_assessment":"Higher daily loss"}'
# Step 2: User types approval → call again
botcore request_red_adjustment '{"parameter":"max_daily_loss_percent","new_value":8.0,"reasoning":"...","risk_assessment":"...","approval_text":"APPROVE CHANGE MAX DAILY LOSS %"}'
```

### Snapshot & Rollback
```bash
botcore take_parameter_snapshot         # Save current state
botcore rollback_adjustment             # Revert to previous snapshot
botcore rollback_adjustment '{"snapshot_id":"snap_123"}'  # Revert to specific snapshot
```

---

## 12. Notifications (1 tool)

```bash
botcore send_telegram_notification '{"message":"Your notification text here"}'  # Send to user Telegram
botcore send_telegram_notification "Your plain text message here"               # Also works with plain text
```

Parse modes: `HTML` (default), `Markdown`, `MarkdownV2`

**IMPORTANT**: For ALL scheduled reports, alerts, and cron job outputs, you MUST use `send_telegram_notification` to deliver results to the user on Telegram. Do NOT just print the report — always send it via this tool.

---

## Tunable Parameters Reference

| Parameter | Tier | Range | Default | Cooldown |
|-----------|------|-------|---------|----------|
| rsi_oversold | GREEN | 20-40 | 25 | 6h |
| rsi_overbought | GREEN | 60-80 | 75 | 6h |
| signal_interval_minutes | GREEN | 3-30 | 5 | 1h |
| confidence_threshold | GREEN | 0.50-0.90 | 0.50 | 6h |
| stop_loss_percent | YELLOW | 0.5-10.0 | 5.0 | 6h |
| take_profit_percent | YELLOW | 1.0-20.0 | 10.0 | 6h |
| position_size_percent | YELLOW | 1.0-10.0 | 2.0 | 6h |
| max_positions | YELLOW | 1-10 | 5 | 6h |
| leverage | YELLOW | 1-125 | 3 | 6h |
| max_daily_loss_percent | RED | 1.0-15.0 | 3.0 | 6h |
| engine_running | RED | true/false | false | 1h |

---

## Safety Rules (MUST FOLLOW)

1. **NEVER enable real trading** without explicit user instruction AND the user typing "APPROVE"
2. **Always check `get_tuning_dashboard`** before suggesting parameter changes
3. **GREEN tier**: Auto-apply and notify user
4. **YELLOW tier**: Present change to user, wait for confirmation token
5. **RED tier**: Present with risk assessment, require user to type exact approval text
6. **Cooldowns**: Respect cooldown periods. Check `get_parameter_bounds` for status
7. **Snapshots**: Always `take_parameter_snapshot` before multiple changes
8. **Rollback**: If performance degrades after changes, suggest `rollback_adjustment`
9. **Transparency**: Show current values before proposing changes
10. **Data-driven**: Base recommendations on actual performance data, not speculation

---

## Autonomous Risk Management (FULL AUTHORITY)

You have **full authority** to proactively manage risk and take profit for paper trading. You do NOT need to ask the user before:

1. **Closing positions** — If a position hits your calculated target or risk threshold, close it immediately using `close_paper_trade_by_symbol`
2. **Adjusting TP/SL** — Change `default_take_profit_pct` and `default_stop_loss_pct` based on market conditions
3. **Managing trailing stops** — Enable/disable and adjust `trailing_stop_pct` and `trailing_activation_pct`
4. **Position sizing** — Adjust `default_position_size_pct` and `default_leverage` based on volatility

### Proactive Decision Framework

When monitoring positions, YOU decide:
- **When to take profit**: If PnL is strong and momentum fading, close the position. Don't wait for the auto-TP
- **When to cut losses early**: If the trade thesis is invalidated, close before SL triggers
- **When to tighten stops**: In high volatility, reduce `trailing_stop_pct` to lock in more gains
- **When to widen stops**: In low volatility trends, increase `default_stop_loss_pct` to avoid premature exits

### Recommended Settings by Market Condition

| Condition | TP % | SL % | Trailing Activation | Trailing Distance | Leverage |
|-----------|------|------|--------------------|--------------------|----------|
| Strong trend | 15-20 | 7-10 | 5 | 2-3 | 3-5 |
| Ranging/choppy | 5-8 | 3-5 | 3 | 1.5-2 | 1-2 |
| High volatility | 10-15 | 5-8 | 5 | 3-5 | 1-2 |
| Low volatility | 8-12 | 3-5 | 3 | 1-2 | 3-5 |

### Example: Proactive Profit Taking
```bash
# 1. Check open positions
botcore get_paper_open_trades
# 2. If ETHUSDT is at +12% PnL and momentum weakening:
botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT","reason":"Proactive TP: +12% PnL, momentum fading"}'
# 3. Report to user
botcore send_telegram_notification '{"message":"✅ Closed ETHUSDT at +12% PnL. Reason: momentum fading on 4h chart."}'
```

### Example: Adjusting Risk for Market Conditions
```bash
# High volatility detected → tighten stops, reduce leverage
botcore update_paper_basic_settings '{"settings":{"default_stop_loss_pct":5,"default_take_profit_pct":10,"default_leverage":2,"trailing_stop_pct":2}}'
botcore send_telegram_notification '{"message":"⚠️ High volatility detected. Tightened SL to 5%, reduced leverage to 2x."}'
```

---

## Common Workflows

### Analyze Losing Trades
```bash
botcore get_paper_closed_trades                    # 1. Get trade history
botcore get_paper_trade_analysis '{"trade_id":"ID"}'  # 2. GPT-4 analysis per trade
botcore get_candles '{"symbol":"BTCUSDT","timeframe":"1h","limit":50}'  # 3. Market data at trade time
botcore get_paper_config_suggestions               # 4. AI recommendations
botcore update_paper_strategy_settings '{"settings":{...}}'  # 5. Apply fixes
```

### Monitor Performance
```bash
botcore get_tuning_dashboard             # 1. Overview (settings + performance + suggestions)
botcore get_paper_portfolio              # 2. Current balance & positions
botcore get_trading_performance          # 3. Win rate, PnL, Sharpe
botcore get_paper_latest_signals         # 4. Recent signals
```

### Optimize Parameters
```bash
botcore get_tuning_dashboard             # 1. Assess current state
botcore get_parameter_bounds             # 2. Check what can be adjusted
botcore take_parameter_snapshot          # 3. Snapshot before changes
botcore apply_green_adjustment '{"parameter":"...","new_value":...,"reasoning":"..."}'  # 4. Apply
```

### Send Report to Telegram
```bash
botcore get_paper_portfolio              # 1. Gather data
botcore get_trading_performance            # 2. Get stats
botcore send_telegram_notification '{"message":"Portfolio Report:\nBalance: $10,250\nDaily PnL: +$125 (+1.2%)\nWin Rate: 65%"}'  # 3. Send to Telegram
```

### Close Position / Take Profit
```bash
botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT"}'  # Close by symbol (simplest)
botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT","reason":"take profit at 24%"}'
# If you need trade ID first:
botcore get_paper_open_trades            # Find trade_id
botcore close_paper_trade '{"trade_id":"trade_xxx_ETHUSDT"}'
```

**IMPORTANT**: When user asks to close/take profit/cut loss a position:
1. Use `close_paper_trade_by_symbol` with the symbol — it handles everything automatically
2. Do NOT ask the user for trade_id — just use the symbol they mention
3. After closing, report the realized PnL to the user

**NOTE on TP/SL percentages**: `default_take_profit_pct` and `default_stop_loss_pct` are **PnL-based** (not price-based). The engine automatically adjusts for leverage internally:
- `take_profit_pct=10%` with 3x leverage → closes when PnL reaches +10% (price moves ~3.3%)
- `stop_loss_pct=5%` with 3x leverage → closes when PnL reaches -5% (price moves ~1.7%)
- `trailing_activation_pct=5%` → trailing stop activates when PnL reaches +5%
- `trailing_stop_pct=3%` → trails 3% below peak PRICE (price-based, not PnL)

### Full System Check
```bash
botcore check_system_health              # 1. All services healthy?
botcore get_connection_status            # 2. External connections OK?
botcore get_paper_trading_status         # 3. Engine running?
botcore get_paper_open_trades            # 4. Any open positions?
botcore get_trading_metrics              # 5. Performance metrics
```
