# OpenClaw Gateway - Functional Requirements

**Spec ID**: FR-OPENCLAW
**Version**: 1.0
**Status**: ☑ Implemented
**Owner**: System
**Last Updated**: 2026-03-03

---

## Tasks Checklist

- [x] Telegram bot channel with allowlist DM policy
- [x] WebSocket gateway on port 18789 with LAN binding
- [x] Token-based gateway authentication
- [x] BotCore bridge CLI (botcore-bridge.mjs) as MCP client
- [x] Cron job scheduling with per-job delivery control
- [x] Skill injection (botcore + billing) into AI system prompt
- [x] AI model configuration (xai/grok-4-1-fast)
- [x] Docker entrypoint with MCP server dependency wait
- [x] Config sync from git-tracked source to named volume
- [x] Stale session lock cleanup on container restart
- [x] Dev/default profile symlink for cron sub-agent pairing
- [ ] Multi-user Telegram allowlist management via MCP tool
- [ ] Gateway health endpoint exposed externally

---

## Metadata

**Related Specs**:
- MCP Server: `FR-MCP.md`
- Risk Management: `FR-RISK.md`
- Paper Trading: `FR-PAPER-TRADING.md`

**Dependencies**:
- MCP Server running at `$MCP_URL` (default: `http://mcp-server:8090`)
- OpenClaw Gateway (Node >= 22)
- Telegram Bot Token (`TELEGRAM_BOT_TOKEN`)
- Claude AI session key (`CLAUDE_AI_SESSION_KEY`)
- Environment vars: `TELEGRAM_USER_ID`, `OPENCLAW_GATEWAY_TOKEN`, `MCP_AUTH_TOKEN`

**Business Value**: High
**Technical Complexity**: Medium
**Priority**: ☑ High

---

## Overview

OpenClaw is an AI gateway that connects Telegram to the BotCore MCP server. It runs as a Docker service, exposes a WebSocket gateway on port 18789, and delivers automated trading reports, health alerts, and interactive commands to a whitelisted Telegram user. The `botcore-bridge.mjs` CLI bridges OpenClaw's `exec` tool to MCP JSON-RPC calls. Cron jobs drive periodic AI analysis sessions that push results to Telegram.

**Architecture**: OpenClaw Gateway → botcore-bridge.mjs (exec) → MCP Server (HTTP/SSE) → Rust/Python backends

---

## Functional Requirements

### FR-OPENCLAW-001: Telegram Channel Integration

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-001`

**Description**:
Accept inbound Telegram messages and send outbound notifications to a whitelisted user ID. All messages outside the allowlist are silently rejected.

**Acceptance Criteria**:
- [x] `channels.telegram.enabled: true` activates the Telegram plugin
- [x] `dmPolicy: "allowlist"` — only users in `allowFrom` can send messages
- [x] `allowFrom` contains `$TELEGRAM_USER_ID` (single-user allowlist)
- [x] `streamMode: "partial"` — responses stream incrementally
- [x] `textChunkLimit: 4000` — messages split at 4000 chars to respect Telegram limits
- [x] Bot token loaded from `$TELEGRAM_BOT_TOKEN` env var

---

### FR-OPENCLAW-002: WebSocket Gateway

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-002`

**Description**:
Expose a WebSocket gateway on port 18789, bound to LAN interfaces, enabling OpenClaw clients (cron, bridge) to connect with token authentication.

**Acceptance Criteria**:
- [x] Gateway listens on port 18789
- [x] `bind: "lan"` — binds to all LAN interfaces (not loopback-only)
- [x] Auth mode `token` — clients must supply `$OPENCLAW_GATEWAY_TOKEN`
- [x] Trusted proxy CIDR list: `192.168.65.0/24`, `172.16.0.0/12`, `127.0.0.0/8`
- [x] Gateway started via `openclaw gateway --dev --port 18789 --bind lan --token $TOKEN`

---

### FR-OPENCLAW-003: Bridge CLI (MCP Client)

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-003`

**Description**:
`botcore-bridge.mjs` provides a CLI interface so the AI can invoke any MCP tool via shell `exec`. It handles MCP session initialization, SSE response parsing, retries, and argument normalization.

**Acceptance Criteria**:
- [x] Usage: `botcore <tool-name> [json-args]`
- [x] Initializes MCP session (JSON-RPC `initialize`) before every tool call
- [x] Supports JSON args, `--key value` flags, and plain-text auto-wrap for `send_telegram_notification`
- [x] Normalizes notification fields: `text`/`content`/`body` → `message`
- [x] Request timeout: 30s (configurable via `MCP_TIMEOUT_MS`)
- [x] Retry policy: 2 retries with 2s/4s backoff on failure
- [x] Bearer token auth via `MCP_AUTH_TOKEN` env var
- [x] `botcore --list` enumerates all available MCP tools
- [x] Exits with code 1 on error; prints JSON-formatted success output

---

### FR-OPENCLAW-004: Cron Job Scheduling

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-004`

**Description**:
Periodic AI sessions are driven by cron jobs defined as JSON files in `openclaw/config/cron/`. Each job specifies a schedule, AI prompt, timeout, and delivery mode. Jobs are registered into the gateway after startup.

**Acceptance Criteria**:
- [x] Cron files in `config/cron/*.json` with fields: `name`, `schedule`, `timeout_seconds`, `no_deliver`, `prompt`
- [x] Jobs registered via `openclaw --dev cron add --url --token --name --cron --message --timeout-seconds`
- [x] `no_deliver: true` suppresses Telegram delivery (silent background jobs)
- [x] Entrypoint waits 15s after gateway ready for config stabilization before registration
- [x] Stale cron files removed on container restart (git-source is authoritative)
- [x] Jobs defined: `health-check` (*/30 min), `morning-briefing` (weekdays 02:00), `hourly-pnl`, `daily-portfolio`, `weekly-review`, `loss-analysis`, `market-regime`, `self-tuning`, `trade-guardian`, `trade-manager`

---

### FR-OPENCLAW-005: Skill Injection

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-005`

**Description**:
The `botcore` skill is injected into the AI's system prompt, providing it with knowledge of available MCP tools, trading identity, strategies, and operational procedures.

**Acceptance Criteria**:
- [x] Skill `botcore` enabled with env vars `MCP_URL` and `MCP_AUTH_TOKEN` passed through
- [x] Skill `billing` enabled for token usage awareness
- [x] Workspace files synced from `openclaw/workspace/` to `~/.openclaw/workspace/` on startup
- [x] Workspace contains: `SKILL.md`, `IDENTITY.md`, `TOOLS.md`, `STRATEGIES.md`, `SOUL.md`, `CONFIG.md`, `USER.md`, `HEARTBEAT.md`, `FEATURES.md`, `AGENTS.md`, `ARCHITECTURE.md`, `BOOTSTRAP.md`, `DEPLOYMENT.md`
- [x] Allowed tools for AI: `exec`, `read`, `write`

---

### FR-OPENCLAW-006: AI Model Configuration

**Priority**: ☑ Medium
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-006`

**Description**:
Configure the primary AI model used by all OpenClaw agents for trading analysis, report generation, and automated decisions.

**Acceptance Criteria**:
- [x] Primary model: `xai/grok-4-1-fast` (set in `agents.defaults.model.primary`)
- [x] Model config applies to all agent sessions including cron-triggered sessions
- [x] Browser tool disabled (`browser.enabled: false`)
- [x] Native commands and skills set to `auto` discovery

---

### FR-OPENCLAW-007: Docker Startup & Config Sync

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-007`

**Description**:
The container entrypoint orchestrates config sync, MCP health wait, gateway startup, cron registration, and stale lock cleanup. Config is read-only mounted and copied to a named volume to prevent git operations from triggering unwanted restarts.

**Acceptance Criteria**:
- [x] MCP server health polled at `$MCP_URL/health` before gateway start (max 60 retries × 5s = 5 min)
- [x] Production config (`openclaw.production.json`) used when `NODE_ENV=production`
- [x] Config file set to `chmod 600` after copy
- [x] Stale `*.lock` session files removed on startup
- [x] `ln -sfn ~/.openclaw ~/.openclaw-dev` symlink ensures cron sub-agents share pairing data
- [x] Gateway startup timeout: 80 retries × 3s = 240s max
- [x] Container exits if MCP server never becomes healthy

---

### FR-OPENCLAW-008: Security

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-OPENCLAW-008`

**Description**:
Protect the gateway and Telegram channel against unauthorized access via token authentication, user ID allowlisting, and secret management through environment variables.

**Acceptance Criteria**:
- [x] Gateway auth token required for all WebSocket connections (`OPENCLAW_GATEWAY_TOKEN`)
- [x] Telegram DM allowlist enforces single authorized user (`TELEGRAM_USER_ID`)
- [x] MCP calls authenticated via Bearer token (`MCP_AUTH_TOKEN`)
- [x] AI session keys stored in env vars, never hardcoded
- [x] Config file permissions set to 600 (owner read/write only)
- [x] No secrets committed to git — all resolved at runtime from environment

---

## Implementation Notes

| Component | Location |
|-----------|----------|
| Dev config | `openclaw/config/openclaw.json` |
| Production config | `openclaw/config/openclaw.production.json` |
| Entrypoint script | `openclaw/scripts/entrypoint.sh` |
| MCP bridge CLI | `openclaw/scripts/botcore-bridge.mjs` |
| Cron job definitions | `openclaw/config/cron/*.json` |
| AI workspace/skills | `openclaw/workspace/` |
| Dockerfile | `openclaw/Dockerfile` |

---

## Dependencies

- **MCP Server** (`mcp-server/`): Must be healthy before gateway starts. Provides all 114 trading tools.
- **Rust API** (`:8080`): Indirect dependency via MCP server for trading operations.
- **Python AI Service** (`:8000`): Indirect dependency via MCP server for AI predictions.
- **Telegram Bot API**: External dependency; requires valid `TELEGRAM_BOT_TOKEN`.
- **Node.js >= 22**: Required by OpenClaw runtime.
- **Timezone**: `TZ=Asia/Ho_Chi_Minh` set in Docker environment for correct cron scheduling.
