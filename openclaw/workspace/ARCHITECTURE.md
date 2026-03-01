# ARCHITECTURE.md - System Architecture
# Source: docker-compose.yml, source code, API routes — NOT docs (docs may be outdated)

## Services & Ports

| Service | Port | Tech | Role |
|---------|------|------|------|
| **MongoDB** | 27017 | MongoDB 7+ | Database (replica set) |
| **Rust Backend** | 8080 | Rust/Warp | Trading engine, strategies, risk mgmt, WebSocket, API |
| **Python AI** | 8000 | Python/FastAPI | xAI Grok analysis, technical indicators fallback |
| **Frontend** | 3000 | Next.js/React/Vite | Dashboard UI |
| **MCP Server** | 8090 | Node.js/TypeScript | 103 tools bridge (Model Context Protocol) |
| **OpenClaw** | 18789 | Node.js | AI gateway (xAI Grok → Telegram/WebSocket) |
| **Redis** | 6379 | Redis | Caching, rate limiting (optional) |

## Data Flow

```
Binance WSS ──→ Rust Backend ──→ Strategy Engine (5 strategies)
                    │                    │
                    │              Signal Generated
                    │                    │
                    ├──→ Python AI ──→ xAI Grok Analysis ──→ AI Signal
                    │                    │
                    │              Risk Check (7 layers)
                    │                    │
                    ├──→ Paper Trading Engine ──→ Execute/Reject
                    │         │
                    │    Trade Result ──→ MongoDB
                    │         │
                    ├──→ WebSocket Broadcast ──→ Frontend Dashboard
                    │
                    └──→ MCP Server ──→ OpenClaw ──→ Telegram Bot
```

## REST API Endpoints (Key Groups)

### Paper Trading (28 endpoints) — `/api/paper-trading/`
- `GET /status` - Engine status (running/stopped)
- `GET /portfolio` - Portfolio state (equity, positions, PnL)
- `GET /trades/open` - Open positions
- `GET /trades/closed` - Closed trade history
- `POST /trades/{id}/close` - Close specific trade
- `PUT /settings` - Update all settings
- `GET|PUT /basic-settings` - Basic params (balance, leverage, position size)
- `GET|PUT /strategy-settings` - Strategy configuration
- `GET|PUT /indicator-settings` - Indicator params (RSI period, MACD, etc.)
- `GET|PUT /symbols` - Enabled trading symbols
- `POST /start` / `POST /stop` - Start/stop engine
- `POST /orders` - Place manual order
- `GET /pending-orders` - List pending orders
- `DELETE /pending-orders/{id}` - Cancel order
- `POST /trigger-analysis` - Force AI analysis now
- `PUT /signal-interval` - Change signal frequency
- `GET /signals-history` / `GET /latest-signals` - Signal history
- `GET /trade-analyses` / `GET /trade-analyses/{id}` - Trade analysis
- `GET /config-suggestions` / `GET /config-suggestions/latest` - AI recommendations
- `POST /reset` - Reset account

### Real Trading (14 endpoints) — `/api/real-trading/` ⚠️ CAUTION
- Same pattern as paper trading but uses REAL money via Binance
- `POST /orders` has 2-step confirmation for safety
- `PUT /positions/{symbol}/sltp` - Modify stop loss / take profit

### Market Data — `/api/`
- `GET /prices` - Latest prices all symbols
- `GET /overview` - Market overview
- `GET /candles/:symbol/:timeframe` - OHLCV candles
- `GET /chart/:symbol/:timeframe` - Chart data with indicators

### Auth — `/api/auth/`
- `POST /register`, `POST /login`, `POST /logout`, `POST /refresh`, `GET /me`
- RS256 JWT, access token 15min, refresh 7 days, bcrypt cost 12

### Settings — `/api/settings/`
- API key management (get/save/test/delete Binance keys)
- Notification preferences, push subscriptions

### AI — `/api/ai/` (Python service)
- `POST /analyze` - xAI Grok market analysis (MAIN endpoint, working)
- `GET /signals/{symbol}` - Recent AI signals

## WebSocket Events — `ws://localhost:8080/ws`

| Event | Direction | Data |
|-------|-----------|------|
| **Connected** | Server→Client | `{ message }` |
| **PositionUpdate** | Server→Client | `{ symbol, side, pnl, current_price, unrealized_pnl }` |
| **TradeExecuted** | Server→Client | `{ symbol, side, quantity, price, pnl }` |
| **AISignalReceived** | Server→Client | `{ symbol, signal, confidence, reasoning, strategy_scores }` |
| **BotStatusUpdate** | Server→Client | `{ status, active_positions, total_pnl, uptime }` |
| **ChartUpdate** | Server→Client | `{ symbol, timeframe, candle, latest_price }` |
| **MarketData** | Server→Client | `{ symbol, price, price_change_24h, volume_24h }` |
| **Ping/Pong** | Bidirectional | Heartbeat (30s interval, 10s timeout) |
| **Error** | Server→Client | `{ message, code, details }` |

## MongoDB Collections (21 total)

**Core Trading:**
- `users` - Accounts (email, password_hash, roles, 2FA)
- `paper_trading_accounts` - Virtual balances, equity, metrics (win_rate, sharpe_ratio)
- `paper_trading_trades` - Trade history (entry/exit price, PnL, AI signal link)
- `paper_trading_settings` - Serialized settings
- `trades` - Live trades
- `positions` - Open live positions
- `strategy_configs` - Strategy configurations

**Market Data:**
- `market_data` - OHLCV candles (time-series, 90 day TTL)
- `portfolio_history` - Portfolio snapshots over time

**AI:**
- `ai_analysis_results` - xAI Grok signals (30 day TTL)
- `ai_signals` - Signal execution log
- `performance_metrics` - Daily performance aggregation

**System:**
- `audit_logs` - Audit trail (180 day TTL)
- `monitoring_alerts` - System alerts
- `risk_metrics` - Real-time risk calculations
- `system_config` - Feature flags, global settings
- `sessions` - User sessions
- `notifications` - Notification queue (7 day TTL)
- `api_keys` - Encrypted exchange API keys
- `training_jobs` - ML training jobs
- `backtest_results` - Backtest results

## MCP Server — 110 Tools, 12 Categories

| Category | Tools | Examples |
|----------|-------|---------|
| Health | 4 | check_rust_health, check_docker_status |
| Market | 8 | get_market_prices, get_candles, add_symbol |
| Paper Trading | 28 | get_portfolio, update_settings, start/stop |
| Real Trading | 14 | get_positions, place_order (2-step confirm) |
| AI | 12 | predict_price, analyze_market, chat_with_ai |
| Tasks | 7 | create_task, get_task_status, cancel_task |
| Monitoring | 5 | get_system_metrics, get_alerts |
| Settings | 10 | manage API keys, notifications |
| Auth | 4 | login, register, refresh_token |
| Tuning | 8 | get_tuning_dashboard, apply_adjustment |
| Backtests | 3 | run_backtest, get_results |

**Security**: 4-tier (PUBLIC → AUTHENTICATED → SENSITIVE → CRITICAL)
**Transport**: Streamable HTTP on port 8090

## Self-Tuning Engine — 3-Tier Safety

| Tier | Action | Parameters | Cooldown |
|------|--------|-----------|----------|
| **GREEN** (auto) | Auto-apply + notify | RSI thresholds, signal interval, confidence | 1-6h |
| **YELLOW** (confirm) | Need confirmation token | Stop loss, take profit, position size, leverage | 6h |
| **RED** (approve) | Need explicit approval text | Max daily loss, engine on/off | 1-6h |

Tuning tools: `get_tuning_dashboard`, `get_parameter_bounds`, `apply_green_adjustment`, `request_yellow_adjustment`, `request_red_adjustment`, `get_audit_history`, `take_snapshot`, `restore_from_snapshot`
