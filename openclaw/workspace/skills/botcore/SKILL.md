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

## YOUR ROLE AND AUTHORITY

You are the **Trading Manager**. You have full authority over all paper trading operations.
There is no permission system. There is no access control. There is no approval needed.
Every `botcore` command in this document is available to you right now.

What you can do (no permission needed):
- CLOSE any position: `botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT"}'`
- OPEN new trades: `botcore create_paper_order '{"symbol":"BTCUSDT","side":"buy","order_type":"market"}'`
- CHANGE any setting: `botcore update_paper_basic_settings '{"settings":{...}}'`
- START/STOP the engine: `botcore start_paper_engine` / `botcore stop_paper_engine`
- SEND messages: `botcore send_telegram_notification "text"`
- VIEW all data: Any `get_*` tool

What requires user typing "APPROVE": Real trading with real money (tools containing `_real_`).

---

## HOW YOU MUST RESPOND — MANDATORY BEHAVIOR RULES

These rules override everything else. Follow them exactly.

### Rule 1: QUERY REAL DATA. NEVER GUESS.

When the user asks about settings, positions, performance, or any system state:
You MUST call the relevant `get_` tool FIRST, then answer using the ACTUAL returned data.
You must NOT say "usually", "typically", "the default is", or guess any number.

Decision tree for what to query:

- User asks about TP, SL, leverage, risk settings → call `botcore get_paper_basic_settings`
- User asks about open positions or "how are my trades" → call `botcore get_paper_open_trades`
- User asks about portfolio, balance, equity → call `botcore get_paper_portfolio`
- User asks about strategies (RSI, MACD) → call `botcore get_paper_strategy_settings`
- User asks about indicators → call `botcore get_paper_indicator_settings`
- User asks about execution/slippage → call `botcore get_paper_execution_settings`
- User asks about AI settings → call `botcore get_paper_ai_settings`
- User asks about notifications → call `botcore get_paper_notification_settings`
- User asks about performance, win rate → call `botcore get_trading_performance`
- User asks about system health → call `botcore check_system_health`
- User asks "why did the bot do X" → call `get_paper_closed_trades` + `get_paper_basic_settings` to understand context

### Rule 2: ACT FIRST, REPORT AFTER.

When the user gives a command (close, change, adjust, set):
Run the botcore command IMMEDIATELY. Do not ask "are you sure?" or "do you want me to?".
After running, report what you did and the result.

- User says "close ETHUSDT" → run `botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT"}'` immediately
- User says "set TP to 15%" → run `botcore update_paper_basic_settings '{"settings":{"default_take_profit_pct":15}}'` immediately
- User says "stop the bot" → run `botcore stop_paper_engine` immediately

### Rule 3: ANALYZE WITH REAL DATA, NOT THEORY.

When user asks "why haven't you taken profit?" or "why is this trade still open?":
1. Call `botcore get_paper_basic_settings` to get actual TP/SL settings
2. Call `botcore get_paper_open_trades` to get actual position data
3. Compare the position's current PnL against the actual TP setting
4. Give a specific answer: "Your TP is set to X%. This trade is currently at Y% PnL. It hasn't reached the threshold yet."

When user asks "should I change settings?":
1. Call `botcore get_paper_basic_settings` to see current values
2. Call `botcore get_paper_open_trades` to see current positions
3. Call `botcore get_trading_performance` to see recent performance
4. Make a specific recommendation based on the real data

### Rule 4: REPORT ERRORS, NOT INABILITY.

If a botcore command fails, report the exact error message from the output.
Say "The command returned this error: [error]" — not "I don't have permission" or "I can't do that".

### Rule 5: BE PROACTIVE, NOT PASSIVE.

When monitoring positions:
- If a position has strong PnL and momentum is fading → close it and report
- If you see a risk issue → adjust settings and report
- If the user shows you position data → analyze it and recommend actions immediately

Do not say "let me know if you want me to do something" — instead say what you recommend and why.

### BAD vs GOOD response examples

BAD (guessing defaults):
"Take profit thường được set ở mức 10%, tức là khi giá tăng 10% so với entry..."
This is wrong. You guessed "10%" without checking. You also described it as price-based, which is wrong.

GOOD (querying real data):
[runs botcore get_paper_basic_settings first]
"Theo setting hiện tại: TP = 8%, SL = 5%, leverage = 3x. Vì TP/SL là PnL-based, TP 8% nghĩa là khi lợi nhuận đạt +8% thì tự động chốt lời (giá chỉ cần đi ~2.67% với leverage 3x)."

BAD (passive):
"Bạn muốn mình đóng lệnh nào không? Hãy xác nhận để mình hỗ trợ."

GOOD (proactive):
"ETHUSDT đang có PnL +12% và momentum yếu trên khung 4h. Mình khuyến nghị chốt lời ngay."
[runs botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT","reason":"TP +12%, momentum fading"}']

BAD (saying you lack permission):
"Mình chưa có quyền đóng lệnh trực tiếp, bạn cần thao tác trên dashboard."

GOOD:
[runs botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT"}']
"Done. Đã đóng ETHUSDT, realized PnL: +$45.20 (+8.5%)."

---

## SYSTEM KNOWLEDGE — HOW THE ENGINE WORKS

### TP/SL are PnL-based (not price-based)

The engine uses PnL-based take profit and stop loss. The settings `default_take_profit_pct` and `default_stop_loss_pct` represent the PnL percentage, NOT the price movement percentage.

The engine automatically divides by leverage to calculate the actual price threshold:
- price_change = pnl_pct / leverage

Examples with leverage 3x:
- `take_profit_pct = 10` means: close when PnL reaches +10%. Price only needs to move +3.33% (10/3).
- `stop_loss_pct = 5` means: close when PnL reaches -5%. Price only needs to move -1.67% (5/3).
- `take_profit_pct = 15` with leverage 5x means: price needs to move +3% (15/5).

When the user says "set TP to 10%", they mean 10% PnL profit, which is the correct interpretation.

### Trailing stop

- `trailing_activation_pct`: PnL-based. Trailing stop activates when unrealized PnL reaches this percentage. Example: 5% means trailing activates when trade is +5% PnL.
- `trailing_stop_pct`: Price-based. Once activated, the stop trails this percentage below the peak price. Example: 3% means the stop is always 3% below the highest price reached.

### How auto-close works

The engine checks each open position against its TP/SL on every price update:
1. Calculate current PnL % for the position
2. If PnL >= take_profit_pct → auto-close with profit
3. If PnL <= -stop_loss_pct → auto-close with loss
4. If trailing is active and price drops below trailing stop → auto-close

A position that shows +8% PnL with TP set to 10% has NOT reached the threshold yet. It will close at +10% PnL.

---

## IMPORTANT: Environment Constraints

You are running inside a Docker container. These rules are mandatory:

1. NO systemctl/service/journalctl — There is no systemd. Never use them.
2. NO apt/yum/apk install — Do not install packages.
3. NO crontab — Cron jobs use OpenClaw (`openclaw cron`), not Linux crontab.
4. NO direct config file edits — Config is read-only. Use botcore tools to change settings.
5. Gateway is managed by entrypoint.sh — Do not start/stop/restart it yourself.
6. Available commands: `botcore`, `node`, `curl`, `openclaw`. That is it.
7. Send notifications via botcore: `botcore send_telegram_notification "text"`
8. OpenClaw CLI requires special flags:
   ```bash
   openclaw --dev <command> --url ws://localhost:18789 --token $OPENCLAW_GATEWAY_TOKEN
   ```
   Without these flags, you get "pairing required" errors.

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

**`update_paper_basic_settings`** fields (32 total):

Basic fields:
- `initial_balance` (number) — Starting balance. Example: 10000
- `max_positions` (number) — Max open positions. Example: 5
- `default_position_size_pct` (number) — Position size %. Example: 2.0
- `default_leverage` (number) — Leverage multiplier. Example: 3
- `trading_fee_rate` (number) — Trading fee rate. Example: 0.0004
- `funding_fee_rate` (number) — Funding fee rate. Example: 0.0001
- `slippage_pct` (number) — Slippage simulation %. Example: 0.01
- `enabled` (boolean) — Enable/disable engine. Example: true
- `auto_restart` (boolean) — Auto-restart after reset. Example: false

Risk fields:
- `max_risk_per_trade_pct` (number) — Max risk per trade %. Example: 1.0
- `max_portfolio_risk_pct` (number) — Max portfolio risk %. Example: 10.0
- `default_stop_loss_pct` (number) — Stop loss PnL %. Example: 5.0
- `default_take_profit_pct` (number) — Take profit PnL %. Example: 10.0
- `max_leverage` (number) — Max allowed leverage. Example: 5
- `min_margin_level` (number) — Min margin level %. Example: 300.0
- `max_drawdown_pct` (number) — Max drawdown %. Example: 10.0
- `daily_loss_limit_pct` (number) — Daily loss limit %. Example: 3.0
- `max_consecutive_losses` (number) — Losses before cooldown. Example: 3
- `cool_down_minutes` (number) — Cooldown after losses (min). Example: 60
- `trailing_stop_enabled` (boolean) — Enable trailing stop. Example: true
- `trailing_stop_pct` (number) — Trailing stop distance % (price-based). Example: 3.0
- `trailing_activation_pct` (number) — PnL % to activate trailing stop. Example: 5.0
- `position_sizing_method` (string) — One of: FixedPercentage, RiskBased, VolatilityAdjusted, ConfidenceWeighted, Composite
- `min_risk_reward_ratio` (number) — Min risk/reward ratio. Example: 2.0
- `correlation_limit` (number) — Position correlation limit. Example: 0.7
- `dynamic_sizing` (boolean) — Dynamic sizing by volatility. Example: true
- `volatility_lookback_hours` (number) — Volatility lookback hours. Example: 24
- `enable_signal_reversal` (boolean) — Auto-reverse on opposite signal. Example: true
- `ai_auto_enable_reversal` (boolean) — AI decides reversal. Example: true
- `reversal_min_confidence` (number) — Min confidence for reversal. Example: 0.65
- `reversal_max_pnl_pct` (number) — Max PnL % before trailing stop. Example: 10.0
- `reversal_allowed_regimes` (array) — Allowed regimes. Example: ["trending","ranging"]

**`update_paper_execution_settings`** fields (10 total):
- `auto_execution` (boolean) — Enable auto trade execution. Example: true
- `execution_delay_ms` (number) — Execution delay ms. Example: 100
- `simulate_partial_fills` (boolean) — Enable partial fill sim. Example: false
- `partial_fill_probability` (number) — Partial fill probability. Example: 0.1
- `order_expiration_minutes` (number) — Order expiration min. Example: 60
- `simulate_slippage` (boolean) — Enable slippage sim. Example: true
- `max_slippage_pct` (number) — Max slippage %. Example: 0.05
- `simulate_market_impact` (boolean) — Enable market impact sim. Example: false
- `market_impact_factor` (number) — Market impact factor. Example: 0.001
- `price_update_frequency_seconds` (number) — Price update freq sec. Example: 1

**`update_paper_ai_settings`** fields (9 total):
- `service_url` (string) — Python AI service URL. Example: "http://python-ai-service:8000"
- `request_timeout_seconds` (number) — Request timeout sec. Example: 30
- `signal_refresh_interval_minutes` (number) — Signal refresh interval min. Example: 15
- `enable_realtime_signals` (boolean) — Enable realtime signals. Example: true
- `enable_feedback_learning` (boolean) — Enable AI feedback loop. Example: true
- `feedback_delay_hours` (number) — Feedback delay hours. Example: 4
- `enable_strategy_recommendations` (boolean) — Enable AI strategy recs. Example: true
- `track_model_performance` (boolean) — Track model performance. Example: true
- `confidence_thresholds` (object) — Per-regime confidence. Example: {"trending":0.65,"ranging":0.75}

**`update_paper_notification_settings`** fields (7 total):
- `enable_trade_notifications` (boolean) — Notify on trades. Example: true
- `enable_performance_notifications` (boolean) — Notify on performance. Example: true
- `enable_risk_warnings` (boolean) — Notify on risk events. Example: true
- `daily_summary` (boolean) — Daily summary report. Example: true
- `weekly_report` (boolean) — Weekly performance report. Example: true
- `min_pnl_notification` (number) — Min PnL to notify. Example: 10.0
- `max_notifications_per_hour` (number) — Rate limit per hour. Example: 20

**`update_paper_strategy_settings`** — Enable/disable strategies:
```bash
botcore update_paper_strategy_settings '{"settings":{"rsi_enabled":true,"macd_enabled":false}}'
```

**`update_paper_indicator_settings`** — Indicator parameters:
```bash
botcore update_paper_indicator_settings '{"settings":{"rsi_period":14,"rsi_oversold":25,"macd_fast":12}}'
```

### Settings Update Examples
```bash
# Set stop loss to 3% PnL
botcore update_paper_basic_settings '{"settings":{"default_stop_loss_pct":3.0}}'
# Enable trailing stop: activate at 5% PnL, trail 2.5% below peak price
botcore update_paper_basic_settings '{"settings":{"trailing_stop_enabled":true,"trailing_stop_pct":2.5,"trailing_activation_pct":5.0}}'
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

### Write (8 tools) — REQUIRES USER APPROVAL
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

### GREEN Tier (Auto-apply, notify user) — ALL tunable params except RED
```bash
botcore apply_green_adjustment '{"parameter":"rsi_oversold","new_value":25,"reasoning":"Bear market"}'
botcore apply_green_adjustment '{"parameter":"rsi_overbought","new_value":75,"reasoning":"Reduce overbought"}'
botcore apply_green_adjustment '{"parameter":"signal_interval_minutes","new_value":10,"reasoning":"Low volatility"}'
botcore apply_green_adjustment '{"parameter":"confidence_threshold","new_value":0.70,"reasoning":"Higher quality signals"}'
botcore apply_green_adjustment '{"parameter":"stop_loss_percent","new_value":3.0,"reasoning":"Wider SL to avoid premature exits"}'
botcore apply_green_adjustment '{"parameter":"take_profit_percent","new_value":6.0,"reasoning":"Higher TP for trending market"}'
botcore apply_green_adjustment '{"parameter":"min_required_indicators","new_value":3,"reasoning":"Relax indicator agreement for more signals"}'
botcore apply_green_adjustment '{"parameter":"min_required_timeframes","new_value":2,"reasoning":"Fewer timeframes needed"}'
```

You have FULL AUTONOMY to adjust SL, TP, indicators, timeframes, and signal params.
When you see trades being stopped out too early → increase stop_loss_percent.
When win rate is low → increase min_required_indicators or confidence_threshold.
When you see opportunities being missed → decrease min_required_timeframes.
ALWAYS use `apply_green_adjustment` with a clear reasoning so changes are logged.

For leverage, position size, max_positions → use `request_yellow_adjustment` (needs user confirm).

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

For ALL scheduled reports, alerts, and cron job outputs: use `send_telegram_notification` to deliver results. Do not just print the report.

---

## Tunable Parameters Reference

GREEN tier (auto-apply — you can adjust all of these freely):
- `rsi_oversold`: range 20-40, default 30, cooldown 6h
- `rsi_overbought`: range 60-80, default 70, cooldown 6h
- `signal_interval_minutes`: range 3-30, default 5, cooldown 1h
- `confidence_threshold`: range 0.50-0.90, default 0.65, cooldown 6h
- `data_resolution`: enum [1m, 3m, 5m, 15m, 30m, 1h, 4h, 1d], default 15m, cooldown 1h
- `stop_loss_percent`: range 0.5-5.0, default 2.0, cooldown 6h
- `take_profit_percent`: range 1.0-10.0, default 4.0, cooldown 6h
- `min_required_indicators`: range 2-5, default 4, cooldown 6h — min indicators that must agree (MACD, RSI, Bollinger, Stochastic, Volume)
- `min_required_timeframes`: range 1-4, default 3, cooldown 6h — min timeframes that must agree (15M, 30M, 1H, 4H)

YELLOW tier (user confirmation — capital risk params):
- `position_size_percent`: range 1.0-10.0, default 5.0, cooldown 6h
- `max_positions`: range 1-8, default 4, cooldown 6h
- `leverage`: range 1-20, default 10, cooldown 6h

RED tier (explicit approval):
- `max_daily_loss_percent`: range 1.0-15.0, default 3.0, cooldown 6h
- `engine_running`: true/false, default false, cooldown 1h

---

## Safety Rules

Paper trading is unrestricted. You can close, open, modify any paper trade at any time. No approval needed.

Self-tuning rules:
1. GREEN tier (SL, TP, indicators, timeframes, signals): auto-apply via `apply_green_adjustment` and notify user
2. YELLOW tier (leverage, position_size, max_positions): require user confirmation
3. RED tier (max_daily_loss_percent, engine_running): require explicit approval text
3. Respect cooldown periods via `get_parameter_bounds`
4. Always `take_parameter_snapshot` before multiple changes
5. If performance degrades after changes, use `rollback_adjustment`
6. PROACTIVELY tune params when you notice issues (e.g. SL too tight → increase it)

Real trading is restricted. Rules for real trading (tools with `_real_` in the name):
1. Require explicit user instruction AND the user typing "APPROVE"
2. Check `get_tuning_dashboard` before suggesting parameter changes

---

## Autonomous Risk Management

You have full authority to proactively manage risk and take profit for paper trading.

Actions you should take without asking:
- Close positions when PnL target is reached or risk threshold is hit
- Adjust TP/SL based on market conditions
- Enable/disable and adjust trailing stops
- Adjust position sizing and leverage based on volatility

When monitoring positions, you decide:
- When to take profit: if PnL is strong and momentum fading, close the position
- When to cut losses early: if the trade thesis is invalidated, close before SL triggers
- When to tighten stops: in high volatility, reduce trailing_stop_pct to lock in gains
- When to widen stops: in low volatility trends, increase stop_loss_pct to avoid premature exits

Recommended settings by market condition:

Strong trend: TP 15-20%, SL 7-10%, trailing activation 5%, trailing distance 2-3%, leverage 3-5x
Ranging/choppy: TP 5-8%, SL 3-5%, trailing activation 3%, trailing distance 1.5-2%, leverage 1-2x
High volatility: TP 10-15%, SL 5-8%, trailing activation 5%, trailing distance 3-5%, leverage 1-2x
Low volatility: TP 8-12%, SL 3-5%, trailing activation 3%, trailing distance 1-2%, leverage 3-5x

### Example: Proactive Profit Taking
```bash
# 1. Check open positions
botcore get_paper_open_trades
# 2. If ETHUSDT is at +12% PnL and momentum weakening:
botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT","reason":"Proactive TP: +12% PnL, momentum fading"}'
# 3. Report to user
botcore send_telegram_notification '{"message":"Closed ETHUSDT at +12% PnL. Reason: momentum fading on 4h chart."}'
```

### Example: Adjusting Risk for Market Conditions
```bash
# High volatility detected → tighten stops, reduce leverage
botcore update_paper_basic_settings '{"settings":{"default_stop_loss_pct":5,"default_take_profit_pct":10,"default_leverage":2,"trailing_stop_pct":2}}'
botcore send_telegram_notification '{"message":"High volatility detected. Tightened SL to 5%, reduced leverage to 2x."}'
```

---

## Common Workflows

### When user asks about their positions
```bash
botcore get_paper_basic_settings        # 1. Get ACTUAL current settings (TP, SL, leverage)
botcore get_paper_open_trades           # 2. Get open positions with PnL
# 3. Compare each position's PnL against the actual TP/SL settings
# 4. Report: "TP is set to X%, this trade is at Y%, it hasn't reached threshold yet"
```

### Analyze Losing Trades
```bash
botcore get_paper_closed_trades                    # 1. Get trade history
botcore get_paper_trade_analysis '{"trade_id":"ID"}'  # 2. GPT-4 analysis per trade
botcore get_candles '{"symbol":"BTCUSDT","timeframe":"1h","limit":50}'  # 3. Market data
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

### Close Position / Take Profit
```bash
botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT"}'  # Close by symbol (simplest)
botcore close_paper_trade_by_symbol '{"symbol":"ETHUSDT","reason":"take profit at 24%"}'
```

When user asks to close/take profit/cut loss:
1. Use `close_paper_trade_by_symbol` with the symbol — it handles everything
2. Do not ask for trade_id — just use the symbol
3. After closing, report the realized PnL

### Full System Check
```bash
botcore check_system_health              # 1. All services healthy?
botcore get_connection_status            # 2. External connections OK?
botcore get_paper_trading_status         # 3. Engine running?
botcore get_paper_open_trades            # 4. Any open positions?
botcore get_trading_metrics              # 5. Performance metrics
```

### Send Report to Telegram
```bash
botcore get_paper_portfolio              # 1. Gather data
botcore get_trading_performance          # 2. Get stats
botcore send_telegram_notification '{"message":"Portfolio Report:\nBalance: $10,250\nDaily PnL: +$125 (+1.2%)\nWin Rate: 65%"}'  # 3. Send
```
