# OpenClaw Gateway (AI via Telegram)

## Quick Reference

### Code Locations
```
openclaw/
├── config/
│   ├── openclaw.json               Dev config (gateway + model settings)
│   ├── openclaw.production.json    Production config (Telegram enabled)
│   └── cron/                       11 cron job definitions
│       ├── jobs.json               Empty registry (managed by entrypoint.sh)
│       ├── health-check.json       Every 30 min — service health
│       ├── trade-manager.json      Every 30 min — manage open positions
│       ├── trade-guardian.json     Hourly — market regime + SL tuning
│       ├── loss-analysis.json      Every 2h — analyze losing trades
│       ├── hourly-pnl.json         Every 6h — PnL report to Telegram
│       ├── self-tuning.json        3x daily — parameter optimization
│       ├── market-regime.json      Every 4h — BTCUSDT regime detection
│       ├── morning-briefing.json   Weekdays 09:00 (UTC+7) — market summary
│       ├── daily-portfolio.json    Daily 20:00 (UTC+7) — portfolio report
│       └── weekly-review.json      Monday 10:00 (UTC+7) — weekly analysis
├── workspace/
│   ├── skills/
│   │   ├── botcore/SKILL.md        Main skill: 110-tool MCP bridge (injected into AI prompt)
│   │   └── billing/SKILL.md        Billing skill: AI cost reporting
│   │       └── scripts/billing.py  Cost aggregation from session logs
│   ├── SOUL.md                     Trading philosophy + decision matrices
│   ├── IDENTITY.md                 AI persona definition
│   ├── TOOLS.md                    Tool usage reference
│   ├── USER.md                     User preferences and context
│   ├── ARCHITECTURE.md             System architecture overview
│   ├── FEATURES.md                 Feature catalog
│   ├── CONFIG.md                   Configuration reference
│   ├── STRATEGIES.md               Trading strategy documentation
│   ├── AGENTS.md                   Agent behavior rules
│   ├── BOOTSTRAP.md                Startup procedures
│   ├── HEARTBEAT.md                Health monitoring rules
│   ├── DEPLOYMENT.md               Deployment procedures
│   └── TOOLS.md                    Tool catalog for AI
└── scripts/
    ├── entrypoint.sh               Docker entrypoint: sync config, wait for MCP, register cron
    └── botcore-bridge.mjs          MCP client CLI bridge
```

### Configuration
- **Gateway Port**: 18789 (WebSocket, LAN-bound)
- **AI Model**: `xai/grok-4-1-fast` (set in `openclaw.production.json`)
- **Auth**: xAI API key via `XAI_API_KEY` env var
- **Timezone**: TZ=Asia/Ho_Chi_Minh (all cron schedules in local time)
- **Tools allowed**: `exec`, `read`, `write`

---

## Architecture

### Bridge Pattern (NOT native MCP client)

OpenClaw does not support native MCP client (issue #4834 closed "not planned" Feb 2026). Bridge approach used instead:

```
Telegram user
    |
    v
OpenClaw AI (Grok model + workspace skills injected into system prompt)
    |
    | exec tool call
    v
botcore-bridge.mjs  ─── MCP client ──►  MCP Server (:8090)
    |                                         |
    |                                    Tool handler
    |                                         |
    |◄────── tool result ─────────────────────┘
    |
    v
AI formats response → Telegram (via telegram plugin)
```

### Bridge Script (`scripts/botcore-bridge.mjs`)

- Invoked as: `botcore <tool_name> '{"param": "value"}'`
- Arguments optional for no-input tools
- **Timeout**: 30 seconds
- **Retries**: 2 with exponential backoff
- **Auto-wraps** plain text responses for `send_telegram_notification`
- **Field normalization**: `text`/`content`/`body` → `message`
- MCP connection via `MCP_URL` + `MCP_AUTH_TOKEN` env vars

---

## Telegram Integration

### Channel Configuration (`openclaw.production.json`)

```json
{
  "channels": {
    "telegram": {
      "enabled": true,
      "botToken": "${TELEGRAM_BOT_TOKEN}",
      "dmPolicy": "allowlist",
      "allowFrom": ["${TELEGRAM_USER_ID}"],
      "streamMode": "partial",
      "textChunkLimit": 4000
    }
  }
}
```

### Security
- **Allowlist-only**: only `TELEGRAM_USER_ID` can send commands
- **Gateway token**: `OPENCLAW_GATEWAY_TOKEN` required for WebSocket connections
- **LAN-bound**: gateway binds to LAN interface only (not public internet)
- **Trusted proxies**: `192.168.65.0/24`, `172.16.0.0/12`, `127.0.0.0/8`

---

## Cron Jobs

11 jobs registered at container startup via `entrypoint.sh`. All use `no_deliver: true` — the AI decides when to send Telegram notifications based on conditions.

### Job Table

| Job Name | Schedule (UTC+7) | Timeout | Delivers | Purpose |
|---|---|---|---|---|
| `health-check` | Every 30 min | 180s | Conditional | Check all services; alert only on failure or unhealthy AI pipeline |
| `trade-manager` | Every 30 min | 180s | Conditional | Review open trades; close only on profit >8%, loss >10%, or stale+reversal |
| `trade-guardian` | Every hour | 300s | Conditional | Market regime check + per-symbol WR analysis + SL/TP auto-tuning |
| `loss-analysis` | Every 2h | 300s | Conditional | Find unanalyzed losing trades → trigger AI analysis → report |
| `hourly-pnl` | Every 6h | 180s | Always | Portfolio balance + PnL + open positions report |
| `self-tuning` | 02:00, 08:00, 16:00 | 300s | Always | Regime detection + GREEN adjustments + weekly summary |
| `market-regime` | Every 4h | 300s | Conditional | BTCUSDT direction → update long_only/short_only mode |
| `morning-briefing` | Mon-Fri 09:00 | 180s | Always | Market prices + 24h change + open positions briefing |
| `daily-portfolio` | Daily 20:00 | 180s | Always | Daily PnL + win rate + closed + open trades report |
| `weekly-review` | Monday 10:00 | 180s | Always | Weekly PnL + tuning dashboard + parameter history analysis |

### Cron Expressions (actual from JSON files)

| Job | Cron Expression | Human readable |
|---|---|---|
| `health-check` | `*/30 * * * *` | Every 30 minutes |
| `trade-manager` | `*/30 * * * *` | Every 30 minutes |
| `trade-guardian` | `0 * * * *` | Top of every hour |
| `loss-analysis` | `0 */2 * * *` | Every 2 hours |
| `hourly-pnl` | `0 */6 * * *` | Every 6 hours |
| `self-tuning` | `0 2,8,16 * * *` | 02:00, 08:00, 16:00 |
| `market-regime` | `0 */4 * * *` | Every 4 hours |
| `morning-briefing` | `0 2 * * 1-5` | Mon-Fri 02:00 UTC (= 09:00 UTC+7) |
| `daily-portfolio` | `0 13 * * *` | Daily 13:00 UTC (= 20:00 UTC+7) |
| `weekly-review` | `0 3 * * 1` | Monday 03:00 UTC (= 10:00 UTC+7) |

### Job Behaviors

**health-check** — Runs `check_system_health` + `check_market_condition_health`. Only sends Telegram on failure. If AI pipeline broken (`healthy=false`): auto-stops paper engine + sends URGENT alert.

**trade-manager** — Evaluates each open trade. Close conditions (need at least 1):
- Profit > +8% AND price reversing
- Loss < -10% AND price continuing against
- Open > 2h AND PnL between -2% to +2% AND reversal signal confirmed
Never closes: trades open < 2h, loss -0.5% to -10% (engine handles SL).

**trade-guardian** — Per-hour multi-step:
1. Market regime check → update `short_only_mode`/`long_only_mode` per DECISION MATRIX
2. Per-symbol WR check (10+ trades) → alert if WR < 40%
3. Closed trades in last 1h: hit SL in < 30 min → widen stop; 2+ consecutive losses → analyze
4. Win rate < 40% last 10 trades → raise `confidence_threshold` (GREEN adjustment)

**self-tuning** — 3× daily optimization:
1. Regime detection (BTCUSDT direction via `get_market_condition`)
2. Apply DECISION MATRIX to `short_only_mode`/`long_only_mode`
3. Analyze WR trends; raise `confidence_threshold` if WR < 45%
4. Up to 3 GREEN adjustments per cycle (SL, TP, confidence, indicators)
5. Always reports via Telegram with params changed + WR + regime

**market-regime** — DECISION MATRIX:
- direction ≥ +0.70 → `long_only_mode=true`, `short_only_mode=false`
- direction ≤ -0.70 → `short_only_mode=true`, `long_only_mode=false`
- -0.69 to +0.69 (including 0.0 / NEUTRAL) → BOTH false
- Only sends Telegram on mode change.

### Cron Registration Gotchas

- NO `--file` flag in `openclaw cron add` — prompt passed inline via `--message`
- Must use `--dev` flag + explicit `--url`/`--token`
- Gateway needs 90–120s to start (model loading, plugin init)
- `entrypoint.sh` waits additional 15s after gateway ready for config stabilization
- SIGUSR1 self-restart wipes in-memory cron — entrypoint detects and re-registers
- `ln -sfn $OPENCLAW_HOME /home/node/.openclaw-dev` — symlinks dev profile so cron sub-agents share pairing data with gateway

---

## Workspace Skills

Skills are injected into the AI system prompt. Two skills active in production:

### botcore skill (`workspace/skills/botcore/SKILL.md`)

- **Role**: Trading Manager with full authority over paper trading
- **Command format**: `botcore <tool_name> '<json-args>'`
- **Requires**: `botcore` binary in PATH, `MCP_URL`, `MCP_AUTH_TOKEN` env vars
- **Tools**: All 110 paper+monitoring+health tools documented inline
- **Authority rules**:
  - No permission needed for paper trading operations (close, open, change settings, start/stop)
  - Real trading (`*_real_*` tools) requires user to type "APPROVE"
- **Behavior rules injected**:
  - Rule 1: Query real data first, never guess (`get_paper_basic_settings` before reporting SL)
  - Rule 2: Act first, report after (no "are you sure?" for paper operations)
  - Rule 3: Analyze with real data (fetch open trades before explaining behavior)
  - SL/TP conversion: user says "1.5% price" → multiply by leverage → PnL%

### billing skill (`workspace/skills/billing/SKILL.md`)

- **Trigger**: user asks about costs, spending, tokens, budget
- **Command**: `python3 {baseDir}/scripts/billing.py [ARGS]`
- **Args**: `today`, `week`, `month`, `7d`, `30d`, `total`, `models` (default: dashboard)
- **Output**: raw stdout piped directly to reply (no reformatting)

---

## Workspace Knowledge Docs

13 markdown files injected as context for the AI agent:

| File | Purpose |
|---|---|
| `SOUL.md` | Trading philosophy, DECISION MATRIX for regime, risk rules, golden principles |
| `IDENTITY.md` | AI persona: Trading Manager role, communication style |
| `TOOLS.md` | Complete tool reference with examples |
| `USER.md` | User preferences, language (Vietnamese), timezone |
| `ARCHITECTURE.md` | System components and data flow |
| `FEATURES.md` | Feature catalog and capabilities |
| `CONFIG.md` | All configurable parameters and valid ranges |
| `STRATEGIES.md` | RSI, MACD, Bollinger strategy configs and signals |
| `AGENTS.md` | Multi-agent behavior rules |
| `BOOTSTRAP.md` | Startup checklist and initialization procedures |
| `HEARTBEAT.md` | Health monitoring rules and alert thresholds |
| `DEPLOYMENT.md` | VPS deployment and Docker procedures |

---

## Docker Deployment

### Container Startup Sequence (`entrypoint.sh`)

```
1. Sync /config-source → ~/.openclaw/ (config, cron, workspace)
2. Remove stale session lock files
3. Symlink --dev profile ← ~/.openclaw (pairing data sharing)
4. Wait for MCP server health (max 60 × 5s = 5 min)
5. Start openclaw gateway --dev --port 18789 --bind lan --token $TOKEN (background)
6. Wait for gateway canvas endpoint (max 80 × 3s = 4 min)
7. Wait 15s for config stabilization (detect SIGUSR1 restart)
8. Register 10 cron jobs
9. wait $GATEWAY_PID (keep container alive)
```

### Environment Variables

| Variable | Required | Purpose |
|---|---|---|
| `TELEGRAM_BOT_TOKEN` | Yes | Telegram bot API token |
| `TELEGRAM_USER_ID` | Yes | Authorized user ID (allowlist) |
| `OPENCLAW_GATEWAY_TOKEN` | Yes | WebSocket gateway auth token |
| `XAI_API_KEY` | Yes | xAI API key for Grok model |
| `MCP_URL` | Yes | MCP server URL (e.g. `http://mcp-server:8090`) |
| `MCP_AUTH_TOKEN` | Yes | Bearer token for MCP requests |
| `NODE_ENV` | No | Set to `production` to use production config |

### Config File Selection

- `NODE_ENV=production` + `openclaw.production.json` exists → use production config (Telegram enabled)
- Otherwise → use `openclaw.json` (dev config)

### Dependencies

- MCP Server (`:8090`) — must be healthy before OpenClaw starts
- Node.js >= 22 (NOT 18 — required by OpenClaw)
- Docker with docker-compose-vps.yml

---

## Related Specs

- `FR-OPENCLAW.md` — Functional requirements
- `FR-MCP.md` — MCP Server requirements (dependency)

**Last Updated**: 2026-03-03
