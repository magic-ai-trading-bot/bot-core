# FEATURES.md - All System Features
# Source: actual source code — NOT docs (docs may be outdated)

## 1. Paper Trading Engine ✅ Production
**Code**: `rust-core-engine/src/paper_trading/`

Simulates realistic trading without real money. Includes:
- **Execution Simulation**: Slippage (0-0.05%), 100ms delay, market impact, partial fills
- **8-Layer Risk Management**: See STRATEGIES.md for full details (8th layer: regime filters)
- **Position Management**: Open/close/reverse trades, trailing stops
- **8 Close Reasons**: TakeProfit, StopLoss, TrailingStop, Manual, AISignal, RiskManagement, MarginCall, TimeBasedExit
- **Signal Reversal**: Auto close + open opposite when high-confidence reversal detected
- **Realism Score**: 98/100 (A+)

**Key API**: `/api/paper-trading/*` (28 endpoints)
**Database**: `paper_trading_accounts`, `paper_trading_trades`, `paper_trading_settings`

## 2. Trading Strategies (5 Active) ✅ Production
**Code**: `rust-core-engine/src/strategies/`

| Strategy | Win Rate | Key Signal |
|----------|----------|-----------|
| RSI | 65% | Oversold < 25 / Overbought > 75 |
| MACD | 61% | Bullish/bearish crossover |
| Bollinger | 63% | Band touch / squeeze breakout |
| Volume | 58% | Volume spike ≥ 2x avg |
| Stochastic | 64% | %K crosses %D at 15/85 |

- **Orchestration**: 4/5 strategies must agree
- **Multi-timeframe**: All strategies use 5M (primary) + 15M (confirmation). 1H loaded for AI bias analysis.
- **Minimum data**: 50 candles required before trading
- Full signal conditions: See STRATEGIES.md

## 3. AI Integration ✅ Production (xAI Grok)
**Code**: `python-ai-service/main.py`

**WORKING:**
- **xAI Grok 4.1 Fast** (`grok-4-1-fast-non-reasoning`) for market analysis
- Multi-timeframe analysis (5m, 15m, 1h)
- Signal generation (Long/Short/Neutral + confidence score)
- Rate limiting with auto-fallback to technical indicators
- Technical indicators fallback (RSI, MACD, BB, EMA, ADX, Stoch, ATR, OBV)
- AI bias as secondary filter (Rust strategies are primary signal source)
- Signal refresh interval: 15 minutes

**⚠️ NOT WORKING (code exists but UNUSED):**
- LSTM model (`models/lstm_model.py`) — not integrated
- GRU model (`models/gru_model.py`) — not integrated
- Transformer model (`models/transformer_model.py`) — not integrated
- Model training endpoints — not functional

**Key API**: `POST /api/ai/analyze` (main endpoint)
**Accuracy**: ~65% directional, varies for fallback indicators
**Cost**: Significantly lower than GPT-4 (xAI pricing)
**Env**: `XAI_API_KEY` (primary), fallback to `OPENAI_API_KEY`

## 4. Authentication & Security ✅ Production
**Code**: `rust-core-engine/src/auth/`

- **RS256 JWT**: Asymmetric encryption (2048-bit RSA keys)
- Access token: 15 min, Refresh token: 7 days
- bcrypt password hashing (cost 12)
- Rate limiting: 5 login attempts per 15 min
- Roles: admin, trader, viewer
- 2FA support (TOTP)

**Key API**: `/api/auth/*` (register, login, logout, refresh, me)

## 5. WebSocket Real-Time ✅ Production
**Code**: `rust-core-engine/src/websocket/`, `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

- **Input**: Binance WSS (`wss://stream.binance.com:9443/ws`)
- **Output**: Frontend (`ws://localhost:8080/ws`)
- 9 event types (see ARCHITECTURE.md)
- Heartbeat: 30s ping, 10s pong timeout
- Auto-reconnect: Exponential backoff up to 30s
- Latency: ~70ms end-to-end
- 100+ concurrent clients

## 6. Smart Signal Reversal ✅ Production
**Code**: `rust-core-engine/src/paper_trading/engine.rs:1259-1450`

Automatically closes position + opens opposite when:
- Signal confidence ≥ 65% (configurable)
- Market regime = trending (or allowed regimes)
- Position P&L < 10% (large profits use trailing stop instead)
- All risk checks still apply

**Config**: `enable_signal_reversal`, `reversal_min_confidence`, `reversal_max_pnl_pct`
**Latency added**: ~5-6ms

## 7. AI Auto-Enable Reversal 🚧 Specified (not yet coded)
**Spec**: `docs/features/ai-auto-reversal.md`

AI decides when to enable/disable signal reversal based on:
- AI Accuracy ≥ 65%, Win Rate ≥ 55%
- Market regime = trending
- Consecutive wins ≥ 3, Volatility < 0.6
- Zero user configuration needed

**Config**: `ai_auto_enable_reversal = true`

## 8. Self-Tuning Engine ✅ Production
**Code**: `mcp-server/src/tuning/`

11 tunable parameters with 3-tier safety:
- **GREEN** (auto): RSI thresholds, signal interval, confidence threshold, stop loss, take profit, min indicators, min timeframes
- **YELLOW** (confirm): Position size, leverage, max positions
- **RED** (approve): Max daily loss, engine on/off

Full audit trail, snapshot/restore, cooldown between changes.

## 9. MCP Server ✅ Production
**Code**: `mcp-server/src/`

114 tools across 12 categories bridging all BotCore APIs.
Allows AI assistants (Claude, OpenClaw) to control the entire trading system.
4-tier security (PUBLIC → CRITICAL).
Streamable HTTP transport on port 8090.

## 10. Real Trading ⚠️ Available but USE WITH CAUTION
**Code**: `rust-core-engine/src/api/real_trading.rs`, `rust-core-engine/src/real_trading/`

14 endpoints mirroring paper trading but using real Binance account.
- 2-step order confirmation for safety
- SL/TP modification on open positions
- Uses `BINANCE_API_KEY` / `BINANCE_SECRET_KEY`

**Auto-Trading Automation** (when `auto_trading_enabled = true`):
- **Strategy Signal Loop** (30s): 5 strategies × 3 timeframes → 5-layer filtering → auto-execute
- **SL/TP Monitor Loop** (5s): Auto-close positions at stop loss / take profit thresholds
- **Price Update Loop** (5s): Real-time unrealized PnL tracking from Binance
- **5-Layer Signal Filtering**: Confidence → Direction mode → Choppy market → AI bias → Signal confirmation
- **Risk Checks**: Daily loss limit, cool-down after consecutive losses, correlation limit, portfolio risk cap, max positions

**Key Config** (via `update_real_trading_settings`):
- `auto_trading_enabled`: Master toggle for automation (default: false)
- `auto_trade_symbols`: Symbols to auto-trade (default: BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT)
- `min_signal_confidence`: Minimum signal strength (default: 0.7)
- `max_consecutive_losses`: Triggers cool-down (default: 3)
- `cool_down_minutes`: Trading pause duration (default: 60)
- `long_only_mode` / `short_only_mode`: Direction restriction

**Dashboard UI**: Auto-Trading Panel on Real Trading page with toggle, symbols config, risk settings.

**Safety defaults**: `BINANCE_TESTNET=true`, `TRADING_ENABLED=false`, `auto_trading_enabled=false`
**NEVER enable production trading without explicit user approval.**

## 11. Frontend Dashboard ✅ Production
**Code**: `nextjs-ui-dashboard/src/`

- Next.js + React + Vite + TypeScript
- Shadcn/UI + TailwindCSS (dark mode)
- Real-time WebSocket updates
- Pages: Dashboard, Trading, Portfolio, Strategies, Settings, HowItWorks
- 71 components, 601 tests

## 12. Notification System ✅ Production
**Code**: `rust-core-engine/src/api/notifications.rs`

Channels: WebSocket, Email, Telegram, Discord, Webhook
- Trade notifications, performance updates, risk warnings
- Daily summary, weekly report
- Push notification support (VAPID)
- Rate limited: max 20/hour

## 13. ATR-Based Position Sizing ✅ Production
**Code**: `rust-core-engine/src/paper_trading/`

Volatility-adaptive position sizing using Average True Range (ATR) instead of fixed percentages.

- **How it works**: Position size = `(equity × base_risk_pct) / (ATR × atr_stop_multiplier × leverage)`
- SL distance = `ATR × atr_stop_multiplier`, TP distance = `ATR × atr_tp_multiplier`
- Ensures consistent dollar risk per trade regardless of current volatility
- Falls back to `default_position_size_pct` when disabled

**Config**: `atr_stop_enabled` (default: false), `atr_period` (14), `atr_stop_multiplier` (1.2), `atr_tp_multiplier` (2.4), `base_risk_pct` (2.0%)
**Diagnostic**: `botcore get_atr_diagnostics` — shows ATR values + computed sizes per symbol

## 14. Half-Kelly Criterion Position Sizing ✅ Production
**Code**: `rust-core-engine/src/paper_trading/`

Dynamically scales position sizes based on historical win rate and average R-multiple using the Kelly Criterion formula. Uses half-Kelly (50%) for safety margin.

- **Formula**: `Kelly% = (win_rate × avg_win − (1 − win_rate) × avg_loss) / avg_win × kelly_fraction`
- Requires minimum `kelly_min_trades` (default 200) closed trades before activating
- Uses last `kelly_lookback` (default 100) trades for the calculation
- Falls back to `default_position_size_pct` until sufficient trade history exists

**Config**: `kelly_enabled` (default: false), `kelly_min_trades` (200), `kelly_fraction` (0.5), `kelly_lookback` (100)

## 15. Regime Filters ✅ Production
**Code**: `rust-core-engine/src/paper_trading/`

Four independent filters that reduce or halt trading when adverse market conditions are detected. Each filter applies a position size multiplier — they compose (multiply together) when multiple filters trigger simultaneously.

### Funding Rate Spike Filter
Reduces position size when Binance perpetual funding rate exceeds threshold. High funding = crowded trade = elevated reversal risk.
- `funding_spike_filter_enabled` (default: false), `funding_spike_threshold` (0.0003 = 0.03%/8h), `funding_spike_reduction` (0.5 = 50% size)

### ATR Spike Filter
Reduces position size when current ATR is abnormally high relative to recent average. Protects against sudden volatility explosions.
- `atr_spike_filter_enabled` (default: false), `atr_spike_multiplier` (2.0x avg = spike), `atr_spike_reduction` (0.5 = 50% size)

### Consecutive Loss Reduction
Progressively reduces position size after consecutive losses, before the hard cool-down triggers. Applies beyond the configured threshold.
- `consecutive_loss_reduction_enabled` (default: false), `consecutive_loss_reduction_threshold` (3 losses), `consecutive_loss_reduction_pct` (0.3 = 30% reduction per extra loss)

### Weekly Drawdown Limit
Suspends all new trades if portfolio drawdown within the current week exceeds the configured limit. Resets at start of new week.
- `weekly_drawdown_limit_pct` (default: 7.0%)

**All regime filter config**: `botcore get_paper_basic_settings` to read, `botcore update_paper_basic_settings` to change.

## 16. Signal Pipeline Weighted Voting ✅ Production
**Code**: `rust-core-engine/src/strategies/`

The signal aggregation pipeline uses a weighted voting system across strategies and timeframes rather than simple majority counting.

- Each strategy's vote is weighted by its historical win rate and recent performance
- Timeframe agreement is scored: 5M primary (weight 1.0) + 15M confirmation (weight 0.7)
- AI bias acts as a secondary multiplier on the combined score (not a vote)
- Final signal confidence = weighted sum of strategy scores × timeframe agreement × AI bias adjustment
- `combination_method` setting controls aggregation mode (default: `AIEnsemble`)
- `get_signal_quality_report` tool provides breakdown of per-strategy contribution to the last N signals
