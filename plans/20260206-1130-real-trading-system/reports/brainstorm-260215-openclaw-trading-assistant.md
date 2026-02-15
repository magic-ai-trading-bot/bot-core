# Brainstorm: OpenClaw Integration with BotCore Trading System

**Date**: 2026-02-15
**Status**: Comprehensive Analysis Complete
**Participants**: Solution Brainstormer + User

---

## 1. Problem Statement

BotCore is a production cryptocurrency trading bot with a Rust backend (port 8080), Python AI service (port 8000), Next.js dashboard (port 3000), and MongoDB, deployed on a 4GB RAM VPS. The user wants to integrate **OpenClaw** -- an open-source personal AI assistant -- to enable conversational control, monitoring, reporting, and AI-enhanced decision-making across messaging platforms (WhatsApp, Telegram, Discord, etc.).

### Key Constraints

- **VPS RAM**: 4GB total, currently ~3GB allocated (MongoDB 512M, Python 1G, Rust 1G, Next.js 512M)
- **OpenClaw requires**: Node 22+, local SQLite, Chromium (browser tool), potentially another LLM API (Claude/OpenAI)
- **Finance-critical**: Wrong commands = money loss. Security is paramount.
- **Existing notification system**: Already has Telegram bot token/chat ID and Discord webhook fields in `NotificationPreferences`

### What OpenClaw Brings

| Capability | Detail |
|---|---|
| **Multi-channel messaging** | WhatsApp (Baileys), Telegram (grammY), Slack, Discord, Signal, iMessage, Matrix, Zalo |
| **WebSocket Gateway** | `ws://127.0.0.1:18789` control plane |
| **Webhook ingestion** | `POST /hooks/wake` and `POST /hooks/agent` for external triggers |
| **Cron scheduler** | Built-in cron with delivery to any channel |
| **Shell execution** | Run arbitrary commands on host |
| **Browser control** | Headless Chromium for web scraping |
| **Custom skills** | `~/.openclaw/workspace/skills/<name>/SKILL.md` |
| **Plugin system** | In-process TypeScript plugins with RPC, tools, CLI extensions |
| **AI model routing** | Claude Opus, Sonnet, OpenAI GPT-4, with failover |
| **Tailscale integration** | Secure remote access without port exposure |
| **DM pairing + allowlists** | Security model for who can interact |

---

## 2. Integration Architecture Analysis

### Approach A: OpenClaw as "Thin Proxy" (Skills + Shell Exec)

OpenClaw skills call `curl` against BotCore's existing REST APIs.

```
User (WhatsApp) --> OpenClaw Gateway --> SKILL.md --> bash(curl http://localhost:8080/api/...) --> BotCore
                                                                                                    |
User (WhatsApp) <-- OpenClaw Gateway <-- AI formats response <--------------------------------------+
```

**Pros**:
- Zero changes to BotCore codebase
- Skills are just Markdown files with instructions
- Quick to implement (days, not weeks)
- OpenClaw's AI naturally formats responses for chat
- Each skill is isolated and independently testable

**Cons**:
- Latency: message -> OpenClaw AI inference -> shell exec -> curl -> Rust -> response -> AI formats -> send
- LLM token cost on every interaction (Claude/OpenAI API call per message)
- Shell exec security surface
- No streaming/real-time push (polling only)

### Approach B: OpenClaw Plugin (TypeScript, In-Process)

Build a custom OpenClaw plugin that directly calls BotCore APIs with proper TypeScript types.

```
User (Telegram) --> OpenClaw Gateway --> Plugin (TS) --> HTTP to BotCore APIs --> Response
                                                    |
                                                    +--> Register agent tools (get_portfolio, execute_trade, etc.)
                                                    +--> Register auto-reply commands (/status, /portfolio, /close)
                                                    +--> Register Gateway RPC methods
```

**Pros**:
- Type-safe, structured tool definitions
- Auto-reply commands bypass AI (instant, zero token cost for simple queries)
- Plugin runs in-process with Gateway (fast)
- Can register slash commands: `/status`, `/portfolio`, `/close BTCUSDT`
- Can subscribe to BotCore WebSocket for real-time push

**Cons**:
- More development effort (TypeScript plugin)
- Must follow OpenClaw plugin API conventions
- Tighter coupling to OpenClaw internals
- Plugin needs maintenance when OpenClaw updates

### Approach C: Webhook Bridge (BotCore pushes to OpenClaw)

BotCore sends events TO OpenClaw's webhook endpoint, which triggers AI-mediated alerts.

```
BotCore Event (trade_executed, daily_loss_limit, etc.)
    |
    v
POST http://localhost:18789/hooks/agent
    { message: "Trade executed: LONG BTCUSDT at $95,000, PnL context...",
      deliver: true, channel: "telegram", to: "chat:123456" }
    |
    v
OpenClaw AI processes event, formats human-readable alert, sends to Telegram
```

**Pros**:
- BotCore drives the conversation (event-driven, not polling)
- AI contextualizes alerts (not just raw data dumps)
- Can combine with Approach A/B for bidirectional flow
- Minimal changes to OpenClaw (just webhook config)

**Cons**:
- Requires adding webhook calls to BotCore Rust code
- Each event = LLM API call (cost consideration)
- Webhook is HTTP, not persistent connection

### RECOMMENDED: Hybrid Approach (A + B + C)

Use all three in a layered architecture:

| Layer | Mechanism | Use Case |
|---|---|---|
| **Instant commands** | Plugin auto-reply (B) | `/status`, `/balance`, `/prices` -- zero AI cost, instant |
| **Complex queries** | Skills + AI (A) | "Analyze my portfolio risk" -- needs AI reasoning |
| **Real-time alerts** | Webhook bridge (C) | Trade executed, risk limit hit -- BotCore pushes to OpenClaw |
| **Scheduled reports** | Cron jobs | Morning briefing, daily P&L, weekly performance |

---

## 3. Comprehensive Feature Map

### 3.1 Control & Commands via Chat

#### Tier 1: Read-Only Commands (Safe, No AI Needed)

These should be **auto-reply plugin commands** -- instant, zero LLM cost.

| Command | Maps To | BotCore API |
|---|---|---|
| `/status` | System health + engine state | `GET /api/health` + `GET /api/paper-trading/status` |
| `/portfolio` | Current portfolio summary | `GET /api/paper-trading/portfolio` |
| `/balance` | Balance + unrealized P&L | `GET /api/paper-trading/portfolio` (extract balance) |
| `/positions` | Open positions list | `GET /api/paper-trading/trades/open` |
| `/prices [symbol]` | Current prices | `GET /api/market/prices` |
| `/signals` | Latest AI signals | `GET /api/paper-trading/latest-signals` |
| `/pnl` | Performance metrics | `GET /api/trading/performance` |
| `/orders` | Pending orders | `GET /api/paper-trading/pending-orders` |
| `/history [n]` | Last N closed trades | `GET /api/paper-trading/trades/closed` |
| `/strategies` | Active strategies + stats | `GET /api/strategies/active` |
| `/monitoring` | System metrics | `GET /api/monitoring/system` |

#### Tier 2: Write Commands (Dangerous, Need Confirmation)

These should require **2-step confirmation** via the AI skill layer.

| Command | Maps To | BotCore API | Risk Level |
|---|---|---|---|
| `/close BTCUSDT` | Close specific position | `POST /api/paper-trading/trades/{id}/close` | MEDIUM |
| `/close-all` | Close all positions | Multiple close calls | HIGH |
| `/start` | Start trading engine | `POST /api/paper-trading/start` | MEDIUM |
| `/stop` | Stop trading engine | `POST /api/paper-trading/stop` | LOW |
| `/reset` | Reset portfolio | `POST /api/paper-trading/reset` | CRITICAL |
| `/set-sl BTCUSDT 94000` | Set stop loss | `POST /api/real-trading/positions/{id}/sltp` | HIGH |
| `/add-symbol ETHUSDT` | Add trading pair | `POST /api/market/symbols` | LOW |
| `/settings risk.daily_loss_limit 3` | Update settings | `POST /api/paper-trading/basic-settings` | HIGH |

**Confirmation Flow for Dangerous Commands:**

```
User: /close-all
Bot: WARNING: This will close 5 open positions:
     - LONG BTCUSDT: +2.3% ($230 unrealized)
     - SHORT ETHUSDT: -0.8% ($-40 unrealized)
     - LONG SOLUSDT: +1.1% ($55 unrealized)
     - LONG BNBUSDT: +0.3% ($15 unrealized)
     - SHORT DOGEUSDT: -0.1% ($-5 unrealized)

     Total unrealized: +$255
     Reply CONFIRM to proceed, or CANCEL to abort.

User: CONFIRM
Bot: Closing 5 positions... Done.
     - BTCUSDT closed at $96,230 (+$230)
     - ETHUSDT closed at $3,210 (-$40)
     [...]
     Net realized: +$255
```

#### Tier 3: AI-Powered Natural Language Commands

These use OpenClaw's AI reasoning via skills.

| Natural Language | What Happens |
|---|---|
| "What's happening with Bitcoin right now?" | Fetches price, recent signals, open positions, formats analysis |
| "Should I close my ETH position?" | Gets position details, current price trend, AI signal, risk metrics, advises |
| "Show me everything about my portfolio risk" | Portfolio + daily loss status + correlation check + cooldown status |
| "How did my RSI strategy perform this week?" | Fetches performance, calculates win rate, compares to other strategies |
| "Run AI analysis on BTCUSDT" | Triggers `POST /api/ai/analyze` and formats result for chat |
| "What would you recommend for market conditions?" | `POST /api/ai/market-condition` + `POST /api/ai/strategy-recommendations` |

### 3.2 Real-Time Alerts (BotCore --> OpenClaw --> User)

Map BotCore's `PaperTradingEvent` types to alert categories:

#### Critical Alerts (Immediate, All Channels)

| Event Type | Alert Message | Priority |
|---|---|---|
| `daily_loss_limit_reached` | "RISK ALERT: Daily loss limit reached ({pct}%). Trading halted." | P0 |
| `cooldown_activated` | "COOLDOWN: {n} consecutive losses. Trading paused for 60 min." | P0 |
| `portfolio_risk_limit_exceeded` | "RISK: Portfolio risk limit exceeded. New trades blocked." | P0 |
| `correlation_limit_exceeded` | "WARNING: Position correlation too high ({pct}%). Diversify." | P1 |

#### Trade Alerts (Per Preference)

| Event Type | Alert Message | Priority |
|---|---|---|
| `trade_executed` | "NEW TRADE: {direction} {symbol} at ${price} | Size: ${size} | SL: ${sl} | TP: ${tp}" | P2 |
| `trade_closed` | "CLOSED: {symbol} | PnL: {pnl_pct}% (${pnl_usd}) | Duration: {duration}" | P2 |
| `stop_limit_order_executed` | "ORDER FILLED: Stop-limit on {symbol} triggered at ${price}" | P2 |
| `stop_limit_order_cancelled` | "ORDER CANCELLED: {symbol} stop-limit expired/cancelled" | P3 |
| `position_reversed` | "REVERSAL: {symbol} flipped from {old_dir} to {new_dir}" | P2 |

#### Signal Alerts

| Event Type | Alert Message | Priority |
|---|---|---|
| `AISignalReceived` | "AI SIGNAL: {direction} {symbol} | Confidence: {conf}% | Strategy: {strategy}" | P2 |
| `signal_outcome_updated` | "SIGNAL RESULT: {symbol} signal was {correct/incorrect} | Actual: {outcome}" | P3 |

#### System Alerts

| Event Type | Alert Message | Priority |
|---|---|---|
| `engine_started` | "Engine started. Monitoring {n} symbols." | P3 |
| `engine_stopped` | "Engine stopped. All monitoring paused." | P3 |
| `settings_updated` | "Settings updated: {changed_fields}" | P3 |

#### Smart Alert Enhancement (AI-Mediated)

Instead of raw event data, OpenClaw's AI can contextualize alerts:

**Raw event**: `trade_closed: BTCUSDT, PnL: -2.1%`

**AI-enhanced alert**:
```
TRADE CLOSED: SHORT BTCUSDT at $96,100

Loss: -2.1% (-$210)
Duration: 4h 23m

Context:
- This is your 3rd consecutive loss today (total: -4.8%)
- Daily loss limit at 5% -- you have 0.2% remaining
- RSI strategy generated this signal (RSI was 72, now 68)
- BTC moved +3.2% in last 4 hours against your short

Recommendation: Consider pausing trading. You're approaching
daily loss limit and the trend is strongly bullish.
```

### 3.3 Scheduled Reports via Cron

#### Morning Briefing (Daily, 7:00 AM User's TZ)

```bash
openclaw cron add --name "Morning Brief" --cron "0 7 * * *" \
  --tz "Asia/Ho_Chi_Minh" --session isolated \
  --message "Generate morning trading briefing" \
  --model opus --thinking medium --announce \
  --channel telegram --to "chat:USER_CHAT_ID"
```

The skill fetches: portfolio status, overnight trades, current prices, AI market condition analysis, open signals.

**Example output**:
```
MORNING BRIEFING - Feb 15, 2026

Portfolio: $10,230 (+2.3% from yesterday)
Overnight: 2 trades closed (+$180), 1 new position opened

Open Positions (3):
  LONG BTCUSDT: +1.2% ($120) - entered 6h ago
  LONG ETHUSDT: +0.8% ($40) - entered 12h ago
  SHORT DOGEUSDT: -0.3% (-$6) - entered 2h ago

Market Conditions:
  BTC: $96,500 (bullish, RSI 62)
  ETH: $3,280 (neutral, RSI 55)
  Overall: Moderately bullish. Low volatility.

AI Recommendation: Hold current positions. No high-confidence
signals pending. Watch BTC resistance at $97,000.

Risk Status: 1.8% daily loss used (limit: 5%). Healthy.
```

#### End-of-Day Summary (Daily, 11:00 PM)

Fetches: all trades today, total P&L, strategy performance comparison, risk events.

#### Weekly Performance Report (Sunday, 10:00 AM)

Fetches: weekly P&L chart data, strategy comparison, best/worst trades, Sharpe ratio trends, AI model accuracy.

#### Monthly Deep Analysis (1st of month)

Uses AI to generate comprehensive report: monthly P&L, strategy ROI breakdown, risk event analysis, optimization suggestions, model accuracy trends.

### 3.4 AI Enhancement of Trading Decisions

#### Multi-Source Market Intelligence

OpenClaw skill that combines:
1. BotCore's `POST /api/ai/analyze` (GPT-4 analysis)
2. BotCore's `POST /api/ai/market-condition`
3. BotCore's `POST /api/ai/strategy-recommendations`
4. OpenClaw's browser tool to scrape Fear & Greed Index, funding rates, whale alerts
5. OpenClaw's web search for breaking crypto news

**Example Interaction**:
```
User: Deep analysis on BTC right now
Bot: Running multi-source analysis...

[1/4] BotCore AI Analysis: BULLISH (confidence 78%)
  - RSI: 62 (neutral-bullish)
  - MACD: Bullish crossover 2h ago
  - Bollinger: Price near upper band

[2/4] Market Condition: TRENDING_UP
  - Recommended strategies: RSI (high conf), MACD (medium conf)

[3/4] External Intelligence:
  - Fear & Greed: 72 (Greed)
  - Funding rate: +0.01% (slightly long-biased)
  - Whale activity: 3 large buys in last hour (Whale Alert)

[4/4] News Sentiment:
  - "SEC approves new BTC ETF" (positive)
  - "Major exchange reports record volume" (positive)
  - No negative catalysts found

SYNTHESIS: Strong bullish conviction (82%).
Current LONG position aligns with analysis.
Consider adding to position if BTC breaks $97,000.
Stop loss recommendation: $94,500 (-2.1%)
```

#### Trade Pre-Approval Gate

A skill that intercepts "should I trade?" queries and runs a full risk check:

```
User: I want to go long ETH with $500
Bot: Running pre-trade analysis...

Risk Check:
  - Current exposure: 3 open positions ($2,100 total)
  - Adding $500 ETH would make 4 positions ($2,600)
  - Correlation with existing BTCUSDT long: 78% (WARNING: above 70% limit)
  - Daily loss used: 3.1% (limit: 5%, only 1.9% remaining)

AI Signal: No active BUY signal for ETH. Last signal was SELL (2h ago).
Strategy consensus: RSI neutral, MACD bearish, Bollinger neutral.

Recommendation: DO NOT open this trade.
Reasons:
1. High correlation with existing BTC long
2. No supporting AI signal
3. Limited daily loss budget remaining

If you still want to proceed, reply FORCE TRADE.
```

### 3.5 Automation Workflows

#### Auto-Recovery from Risk Events

```
Event: daily_loss_limit_reached
  --> OpenClaw webhook receives event
  --> AI analyzes: what trades caused losses, market conditions
  --> Sends alert to user with analysis
  --> Automatically stops engine: POST /api/paper-trading/stop
  --> Schedules cron to re-check in 2 hours
  --> After 2 hours: checks if market conditions changed, advises restart
```

#### Trailing Stop Notification Chain

```
Event: price_update (monitored position reaching +5%)
  --> OpenClaw skill detects significant unrealized profit
  --> Sends: "BTCUSDT is up +5.2% ($520). Consider tightening stop loss."
  --> User replies: "Move stop to breakeven"
  --> Skill calls: POST /api/real-trading/positions/{id}/sltp { stop_loss: entry_price }
  --> Confirms: "Stop loss moved to breakeven at $95,000"
```

#### Strategy Performance Watchdog

Cron job (every 4 hours):
```
- Fetch strategy performance for each active strategy
- If any strategy's win rate drops below 40% over last 20 trades:
  --> Alert: "RSI strategy win rate dropped to 38% (last 20 trades). Consider disabling."
- If a strategy is outperforming:
  --> Alert: "MACD strategy at 72% win rate. Consider increasing allocation."
```

#### Market Regime Change Detector

Cron job (every 1 hour):
```
- Fetch AI market condition
- Compare with last known regime
- If regime changed (TRENDING_UP -> RANGING -> TRENDING_DOWN):
  --> Alert: "Market regime change detected: TRENDING_UP -> RANGING"
  --> AI recommends strategy adjustments
  --> If user enabled auto-adjust: automatically update strategy weights
```

### 3.6 Security Model

#### Authentication Architecture

```
                    Allowlist
User (WhatsApp) ----[DM Pairing]----> OpenClaw Gateway
                                          |
                                    [Session = main]
                                          |
                                    Skill/Plugin
                                          |
                                    [JWT Auth Token]
                                          |
                                    BotCore API (port 8080)
```

**Layer 1: OpenClaw Channel Security**
- DM pairing required: unknown senders get pairing codes
- Only allowlisted phone numbers/accounts can interact
- Group chats: owner-only commands
- `agents.defaults.sandbox.mode: "non-main"` for group isolation

**Layer 2: Command Authorization**
- Read-only commands: no additional auth
- Write commands: require explicit confirmation flow in chat
- Critical commands (`/reset`, `/close-all`, settings changes): require a PIN/passphrase
- Real trading commands: require both confirmation + PIN

**Layer 3: BotCore API Auth**
- Plugin/skill authenticates with BotCore using JWT token
- Token stored in OpenClaw's encrypted config (not in SKILL.md)
- Token auto-refreshes via `POST /api/auth/refresh`
- Rate limiting on trading endpoints

**Layer 4: Operational Security**
- OpenClaw bound to loopback (127.0.0.1) -- not exposed to internet
- Tailscale for secure remote access (if needed)
- Webhook token for BotCore-to-OpenClaw communication
- All trading commands logged with sender ID, timestamp, channel
- Emergency kill switch: user can text "EMERGENCY STOP" from any channel

#### PIN/Passphrase Flow for Dangerous Commands

```
User: /close-all
Bot: This will close 5 positions (est. +$255).
     Enter your trading PIN to confirm:

User: 7842
Bot: PIN verified. Closing positions...
     [results]
```

#### Audit Trail

Every command executed via OpenClaw should be logged:

```json
{
  "timestamp": "2026-02-15T10:30:00Z",
  "channel": "whatsapp",
  "sender": "+84987654321",
  "command": "close_position",
  "params": { "symbol": "BTCUSDT" },
  "result": "success",
  "api_endpoint": "POST /api/paper-trading/trades/12345/close",
  "response_time_ms": 150
}
```

### 3.7 Unique / Creative Ideas

#### 1. Voice Trading via WhatsApp Voice Notes

OpenClaw supports ElevenLabs TTS. Extend this:
- User sends voice message on WhatsApp: "What's my portfolio looking like?"
- OpenClaw transcribes (Whisper) -> processes -> responds with voice note
- For alerts, option to receive spoken summaries instead of text

#### 2. Photo-Based Chart Analysis

- User screenshots a TradingView chart, sends via WhatsApp
- OpenClaw uses AI multimodal (Claude Vision) to analyze the chart
- Responds with technical analysis observations
- Compares with BotCore's internal signals: "Your chart shows a head-and-shoulders pattern. BotCore's RSI strategy agrees -- current signal is SELL with 68% confidence."

#### 3. Group Trading Room

- Create a Telegram/Discord group
- OpenClaw broadcasts trades, signals, and analysis to the group
- Multiple users can observe (read-only)
- Only the owner can execute commands
- Good for learning or sharing strategies with friends

#### 4. "What-If" Scenario Simulator

```
User: What if I had followed only MACD signals this week?
Bot: Backtesting MACD-only for Feb 10-15...

Results if MACD-only:
  Trades: 12 (vs your actual 18)
  Win rate: 67% (vs your actual 58%)
  Net P&L: +$450 (vs your actual +$230)
  Max drawdown: -1.8% (vs your actual -3.2%)

The MACD strategy alone would have outperformed your
multi-strategy approach this week. Consider increasing
MACD weight in your strategy mix.
```

#### 5. Smart Price Alerts with Context

Instead of dumb price alerts ("BTC hit $100K"), provide context:

```
BTC just crossed $100,000!

Your positions affected:
  - LONG BTCUSDT: now +8.3% ($830) -- consider taking profit
  - Your TP was set at $101,000 -- only 1% away

Market context:
  - Major psychological level, expect volatility
  - RSI at 78 (overbought territory)
  - Funding rate spiked to +0.05% (very long-biased)

Suggestion: Consider closing 50% of position and
trailing stop on the rest.
```

#### 6. "Teach Me" Mode

OpenClaw explains every trade decision the bot makes:

```
User: Why did the bot just short ETH?
Bot: Trade explanation for SHORT ETHUSDT at $3,280:

Signals that triggered this trade:
  1. RSI Strategy: RSI at 74 (overbought) -> SELL signal
  2. Bollinger Strategy: Price touched upper band -> SELL signal
  3. MACD Strategy: No signal (neutral)
  4. Volume Strategy: Declining volume on rise -> bearish divergence

Consensus: 3/4 strategies bearish (75% agreement)
AI confidence: 72%

Risk parameters applied:
  - Position size: $500 (5% of portfolio)
  - Stop loss: $3,380 (+3.05%)
  - Take profit: $3,150 (-3.96%)
  - Risk/reward ratio: 1:1.3

This trade follows the bot's standard multi-strategy
consensus approach. Want me to explain any part in detail?
```

#### 7. Cross-Platform Notification Routing

Different alert types go to different platforms based on urgency:

| Urgency | Platform | Rationale |
|---|---|---|
| P0 (Critical risk) | WhatsApp + Telegram + SMS | Maximum reach |
| P1 (Warnings) | Telegram | Primary trading channel |
| P2 (Trade alerts) | Discord | Can be batch-viewed |
| P3 (Info) | Discord #logs channel | Low noise |
| Reports | Email (via webhook) | Archivable, searchable |

#### 8. Natural Language Settings

```
User: Make the bot more conservative
Bot: I'll adjust settings for a more conservative profile:

Current -> Proposed changes:
  - Daily loss limit: 5% -> 3%
  - Max concurrent positions: 10 -> 5
  - Minimum signal confidence: 60% -> 75%
  - Position size multiplier: 1.0 -> 0.7

This should reduce risk by ~40% but may also reduce
returns by ~25%.

Apply these changes? (CONFIRM/CANCEL)
```

---

## 4. Resource & Feasibility Analysis

### VPS Memory Impact

Current allocation (4GB VPS):
| Service | Memory |
|---|---|
| MongoDB | 512MB |
| Python AI | 1GB |
| Rust Core | 1GB |
| Next.js | 512MB |
| **Total** | **3GB** |
| OS + overhead | ~500MB |
| **Available** | **~500MB** |

OpenClaw Gateway requirements:
| Component | Memory |
|---|---|
| Node.js Gateway | ~200-300MB |
| SQLite | ~10MB |
| Chromium (if used) | ~300-500MB |

**Problem**: OpenClaw with Chromium does not fit in 500MB free RAM.

**Solutions** (ranked by preference):

1. **Run OpenClaw on a separate machine** (recommended)
   - Your local Mac, a cheap $5/mo VPS, or Raspberry Pi
   - OpenClaw connects to BotCore VPS via Tailscale
   - Zero impact on trading performance
   - Clean separation of concerns

2. **Run OpenClaw on same VPS without Chromium**
   - Disable browser tool (not needed for trading commands)
   - Gateway alone: ~200-300MB -- fits in available RAM
   - Reduce Next.js to 256MB or disable it if using only chat interface
   - Tight but workable

3. **Upgrade VPS to 8GB**
   - Cost: typically $10-20/mo more
   - Removes all constraints
   - Allows full OpenClaw features including browser

### LLM API Cost Estimate

| Interaction Type | Frequency | Tokens/call | Model | Cost/month |
|---|---|---|---|---|
| Auto-reply commands | ~50/day | 0 (no LLM) | None | $0 |
| AI-powered queries | ~10/day | ~2K | Claude Sonnet | ~$2 |
| Alert contextualization | ~20/day | ~1K | Claude Sonnet | ~$2 |
| Morning briefing | 1/day | ~3K | Claude Opus | ~$3 |
| Weekly report | 1/week | ~5K | Claude Opus | ~$2 |
| Deep analysis | ~3/day | ~4K | Claude Opus | ~$10 |
| **Total estimate** | | | | **~$19/month** |

This is very reasonable. The key optimization is using auto-reply commands for frequent simple queries to avoid LLM costs entirely.

---

## 5. Recommended Implementation Plan

### Phase 1: Foundation (Week 1-2) -- Low Risk, High Value

**Goal**: Basic chat control and read-only monitoring.

1. **Install OpenClaw on separate machine** (Mac or secondary VPS)
   - `npm install -g openclaw@latest && openclaw onboard --install-daemon`
   - Configure Telegram as primary channel (you already have bot token/chat ID in BotCore)
   - Configure Tailscale for secure connection to BotCore VPS

2. **Create BotCore Plugin** (TypeScript)
   - Register auto-reply commands for all Tier 1 read-only operations
   - Plugin fetches from `http://<botcore-vps>:8080/api/*` via Tailscale
   - No AI cost for these commands

3. **Create "BotCore Status" skill**
   - `~/.openclaw/workspace/skills/botcore-status/SKILL.md`
   - Handles natural language queries about portfolio, positions, prices
   - Falls back to curl commands against BotCore API

**Deliverables**: `/status`, `/portfolio`, `/positions`, `/prices`, `/pnl`, `/signals` working via Telegram.

### Phase 2: Alerts (Week 3-4) -- Medium Effort, Critical Value

**Goal**: Real-time trading alerts pushed to messaging.

1. **Add webhook endpoint to BotCore Rust code**
   - New module: `src/api/openclaw_webhook.rs`
   - Subscribe to `PaperTradingEvent` broadcast channel
   - Filter events by type, POST to OpenClaw's `/hooks/agent` endpoint
   - Include event context (portfolio state, recent trades) in webhook payload

2. **Configure OpenClaw webhook ingestion**
   ```json
   {
     "hooks": {
       "enabled": true,
       "token": "<shared-secret>",
       "path": "/hooks"
     }
   }
   ```

3. **Create alert formatting skill**
   - Takes raw event data, produces human-readable contextualized alerts
   - Different verbosity levels (brief for P3, detailed for P0)

**Deliverables**: Real-time trade/risk/signal alerts on Telegram.

### Phase 3: Trading Control (Week 5-6) -- High Effort, High Risk

**Goal**: Execute trades and manage positions via chat.

1. **Add write commands to plugin**
   - `/close`, `/start`, `/stop`, `/close-all`
   - Confirmation flow with 2-step verification
   - PIN-based auth for dangerous operations

2. **Create trade execution skill**
   - Natural language: "Close my Bitcoin position"
   - AI parses intent, maps to API call, asks for confirmation
   - Logs all actions to audit trail

3. **Security hardening**
   - Rate limiting on write commands
   - Cooldown after failed PIN attempts
   - Emergency stop command

**Deliverables**: Full trading control via Telegram with security.

### Phase 4: Intelligence (Week 7-8) -- Medium Effort, Differentiating

**Goal**: AI-enhanced trading intelligence.

1. **Set up cron reports**
   - Morning briefing (7:00 AM)
   - End-of-day summary (11:00 PM)
   - Weekly report (Sunday 10:00 AM)

2. **Multi-source analysis skill**
   - Combines BotCore AI + external data (Fear & Greed, news)
   - "Deep analysis on BTC" command

3. **What-if simulator skill**
   - Backtests alternative strategy scenarios
   - Compares with actual performance

**Deliverables**: Scheduled reports, deep analysis, what-if scenarios.

### Phase 5: Advanced Features (Week 9+) -- Optional, Creative

- Voice trading via WhatsApp
- Photo chart analysis
- Group trading room (Discord/Telegram)
- Cross-platform notification routing
- Natural language settings adjustment
- "Teach me" trade explanation mode
- Strategy performance watchdog (auto-alert)
- Market regime change detector

---

## 6. Key Technical Decisions

### Decision 1: Where to Run OpenClaw

**Recommendation**: Separate machine (your Mac for development, then a cheap $5/mo VPS for production).

**Rationale**: The 4GB VPS is already near capacity. Running OpenClaw on the same machine risks OOM kills on the trading engine -- unacceptable for a finance system. A $5/mo Tailscale-connected VPS is cheap insurance.

### Decision 2: Primary Messaging Channel

**Recommendation**: Start with Telegram.

**Rationale**: You already have `TelegramSettings` (bot_token, chat_id) in your notification preferences. Telegram has the best bot ecosystem, supports rich formatting, inline keyboards for confirmations, and is the default choice for crypto trading bots. Add WhatsApp/Discord later.

### Decision 3: Plugin vs Skills vs Both

**Recommendation**: Plugin for structured commands, Skills for AI-powered interactions.

**Rationale**: Auto-reply commands via the plugin eliminate LLM costs for 80% of interactions (simple queries). Skills handle the 20% that actually benefit from AI reasoning. This hybrid approach optimizes both cost and capability.

### Decision 4: How BotCore Sends Events to OpenClaw

**Recommendation**: Add a lightweight HTTP webhook sender to the Rust codebase that subscribes to the existing `PaperTradingEvent` broadcast channel.

**Rationale**: The broadcast channel already exists. Adding a subscriber that POSTs to OpenClaw's webhook endpoint requires minimal code (~100 lines of Rust). No new dependencies, no architectural changes. The webhook fires only on actual events (not polling), so it's efficient.

### Decision 5: Security for Trade Execution via Chat

**Recommendation**: Three-layer security: allowlist + confirmation + PIN.

**Rationale**: For a finance system, defense in depth is mandatory. The allowlist prevents unauthorized access. The confirmation flow prevents accidental commands. The PIN prevents someone who borrows your phone from executing trades. All three together make it extremely unlikely that an unauthorized trade is executed.

---

## 7. Risks and Mitigations

| Risk | Impact | Probability | Mitigation |
|---|---|---|---|
| OpenClaw Gateway crash = missed alerts | Medium | Low | Run as systemd service with auto-restart. Fallback to BotCore's native Telegram/Discord notifications. |
| LLM API outage = no AI-enhanced alerts | Low | Low | Auto-reply commands still work. Raw alerts forwarded without AI formatting. |
| Security breach via chat | Critical | Very Low | Three-layer auth. Audit logs. Emergency stop. Read-only by default. |
| VPS OOM from running both | High | Medium | Run OpenClaw separately. Monitor with `htop`. Set memory limits. |
| Message ordering issues | Medium | Low | Sequence numbers on commands. Ignore duplicate/stale messages. |
| Latency on trade commands | Medium | Low | Auto-reply for reads (<100ms). AI-mediated writes acceptable at 2-3s. |
| OpenClaw breaking changes on update | Low | Medium | Pin OpenClaw version. Test updates on staging first. |

---

## 8. Success Metrics

| Metric | Target | How to Measure |
|---|---|---|
| Command response time (read) | <500ms | Plugin auto-reply latency |
| Command response time (write) | <5s | End-to-end including confirmation |
| Alert delivery latency | <10s from event | Timestamp comparison |
| False alert rate | <1% | Monitor alert accuracy |
| Uptime | 99.9% | OpenClaw + BotCore combined |
| Monthly LLM cost | <$25 | OpenAI/Anthropic billing |
| User satisfaction | Daily active usage | Engagement tracking |
| Security incidents | 0 | Audit log review |

---

## 9. Files and Code Locations Reference

### Existing BotCore Code to Hook Into

| Purpose | File | Key Lines/Functions |
|---|---|---|
| API routes (all) | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/api/mod.rs` | `start()` function defines all routes |
| Paper trading API | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/api/paper_trading.rs` | 25+ endpoints under `/api/paper-trading/` |
| Real trading API | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/api/real_trading.rs` | Endpoints under `/api/real-trading/` |
| Event broadcast | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/paper_trading/engine.rs` | `event_broadcaster: broadcast::Sender<PaperTradingEvent>` |
| Event types | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/paper_trading/mod.rs` | `PaperTradingEvent { event_type, data, timestamp }` |
| Notification prefs | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/api/notifications.rs` | `TelegramSettings`, `DiscordSettings` |
| Auth middleware | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/middleware.rs` | `with_auth()`, `with_admin_auth()` |
| JWT tokens | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/auth/jwt.rs` | `generate_token()`, `verify_token()` |
| AI analysis | `/Users/dungngo97/Documents/bot-core/python-ai-service/main.py` | Lines 2628+ (`/ai/analyze`, `/ai/market-condition`, etc.) |
| Docker compose | `/Users/dungngo97/Documents/bot-core/docker-compose-vps.yml` | Full deployment config (4GB VPS) |
| Main entry | `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/main.rs` | Broadcast channel setup at lines 105-107 |

### New Files to Create (When Implementing)

| File | Purpose |
|---|---|
| `openclaw-plugin/package.json` | OpenClaw plugin manifest |
| `openclaw-plugin/openclaw.plugin.json` | Plugin configuration |
| `openclaw-plugin/src/index.ts` | Plugin entry point (auto-reply commands + tool definitions) |
| `openclaw-plugin/src/botcore-api.ts` | BotCore REST API client |
| `openclaw-plugin/src/formatters.ts` | Message formatters for different platforms |
| `~/.openclaw/workspace/skills/botcore-trading/SKILL.md` | Main trading skill |
| `~/.openclaw/workspace/skills/botcore-analysis/SKILL.md` | AI analysis skill |
| `~/.openclaw/workspace/skills/botcore-reports/SKILL.md` | Scheduled report skill |
| `rust-core-engine/src/api/openclaw_webhook.rs` | Webhook sender to OpenClaw |

---

## 10. Summary

OpenClaw is an excellent fit for BotCore. The key insight is the **hybrid approach**: use OpenClaw's plugin system for instant zero-cost commands (80% of interactions), and its skill/AI system for the 20% that genuinely benefit from LLM reasoning. The webhook bridge from BotCore to OpenClaw enables real-time alerting without polling.

The main constraint is VPS memory -- running OpenClaw on a separate machine is strongly recommended. Start with Telegram, add other platforms later. Phase the implementation over 8+ weeks, starting with read-only monitoring (safest) and gradually adding write capabilities with proper security layers.

**Estimated total effort**: 6-8 weeks for Phases 1-4
**Estimated monthly cost**: ~$19-25 for LLM APIs + $5 for secondary VPS
**Risk level**: Low (read-only first, security-hardened write commands later)

This integration transforms BotCore from a "check the dashboard" experience into a "your AI trading assistant is always in your pocket" experience -- which is exactly what a personal trading bot should be.
