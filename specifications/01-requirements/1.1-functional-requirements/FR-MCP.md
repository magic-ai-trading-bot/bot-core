# MCP Server - Functional Requirements

**Spec ID**: FR-MCP
**Version**: 1.0
**Status**: ☑ Implemented
**Owner**: System
**Last Updated**: 2026-03-03

---

## Tasks Checklist

- [x] MCP protocol compliance (v2024-11-05, Streamable HTTP)
- [x] Per-session McpServer instance management
- [x] Bearer token authentication for incoming requests
- [x] JWT auto-login for outgoing BotCore API calls
- [x] Health check endpoint (unauthenticated)
- [x] Session lifecycle management (create/reuse/delete)
- [x] 12 tool categories registered (105 tools total)
- [x] API client proxy to Rust (:8080) and Python (:8000) backends
- [x] Request timeout and 5xx retry handling
- [x] Response normalization for mixed Rust/Python formats
- [x] 4-tier security model (PUBLIC/AUTHENTICATED/SENSITIVE/CRITICAL)
- [ ] Rate limiting per session
- [ ] Tool-level audit logging
- [ ] Metrics/observability endpoint

---

## Metadata

**Related Specs**:
- Related Auth: [FR-AUTH.md](./FR-AUTH.md)
- Related API: `specs/02-technical-specs/2.1-api-specifications/API_SPEC.md`

**Dependencies**:
- `@modelcontextprotocol/sdk` v1.12+ (McpServer, StreamableHTTPServerTransport)
- `express` v4 — HTTP server framework
- `zod` v3 — Tool input schema validation
- Rust Core Engine at `RUST_API_URL` (default: `http://localhost:8080`)
- Python AI Service at `PYTHON_API_URL` (default: `http://localhost:8000`)

**Business Value**: Critical
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

The BotCore MCP Server is a Node.js/TypeScript service that exposes cryptocurrency trading bot capabilities through the Model Context Protocol (MCP). It acts as a bridge between AI clients (OpenClaw/Claude) and the BotCore backend services (Rust trading engine, Python AI service). The server uses Streamable HTTP transport on port 8090 and manages isolated per-session MCP server instances as required by the SDK.

**Architecture**: Stateless HTTP gateway with session-keyed transport map. Each MCP session gets its own `McpServer` + `StreamableHTTPServerTransport` pair. Bearer token guards the `/mcp` endpoint; JWT auto-login handles downstream API auth.

---

## Functional Requirements

### FR-MCP-001: Protocol Compliance

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-001`

**Description**:
Implement MCP protocol version 2024-11-05 using the official `@modelcontextprotocol/sdk`. Server must advertise `tools` and `resources` capabilities and respond to MCP initialization handshake.

**Acceptance Criteria**:
- [x] Protocol version: MCP v2024-11-05
- [x] Transport: `StreamableHTTPServerTransport` (Streamable HTTP)
- [x] Server name: `botcore-mcp-server`, version `1.0.0`
- [x] Capabilities declared: `tools: {}`, `resources: {}`
- [x] Server instructions injected into AI system prompt

---

### FR-MCP-002: Bearer Token Authentication

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-002`

**Description**:
Validate incoming requests to `/mcp` using a Bearer token from `Authorization` header. Token value is configured via `MCP_AUTH_TOKEN` environment variable. If token is unset, all requests are permitted (dev mode with warning).

**Acceptance Criteria**:
- [x] Returns HTTP 401 when token is invalid or missing
- [x] Accepts `Bearer <token>` and bare token formats
- [x] Logs warning when `MCP_AUTH_TOKEN` is unset
- [x] Health check endpoint bypasses auth

---

### FR-MCP-003: API Client Proxy

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-003`

**Description**:
Centralized HTTP client proxies MCP tool calls to Rust or Python backend APIs. Handles JWT injection, 30-second default timeout, abort signals, and one automatic retry on 5xx GET errors.

**Acceptance Criteria**:
- [x] Routes to `rust` (`RUST_API_URL`) or `python` (`PYTHON_API_URL`) targets
- [x] Injects JWT `Authorization: Bearer` header on all requests (except `skipAuth`)
- [x] 30-second default timeout with `AbortController`
- [x] Single automatic retry on 5xx for GET requests (2-second delay)
- [x] Normalizes responses: Rust `{success, data}` and raw Python responses both supported
- [x] Returns `{success: false, error: string}` on all failure paths

---

### FR-MCP-004: JWT Auto-Login

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-004`

**Description**:
The MCP server authenticates with the Rust API using `BOTCORE_EMAIL`/`BOTCORE_PASSWORD` credentials and caches the JWT token. Token is refreshed proactively when within 1 hour of expiry (6-day refresh window for 7-day tokens).

**Acceptance Criteria**:
- [x] Calls `POST /api/auth/login` on first request or near-expiry
- [x] Caches JWT in memory; reuses until expiry threshold
- [x] Refresh triggered when `expiry < now + 1h`
- [x] Returns empty string (not throw) when credentials unset or login fails
- [x] Logs errors without crashing the server

---

### FR-MCP-005: Session Management

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-005`

**Description**:
Each MCP client session gets an isolated `McpServer` + `StreamableHTTPServerTransport` pair (SDK requirement). Sessions are keyed by UUID session ID. Session ID is set during the `initialize` request and stored post-`handleRequest`.

**Acceptance Criteria**:
- [x] New session: `POST /mcp` without `mcp-session-id` header creates session
- [x] Existing session: requests with valid `mcp-session-id` reuse transport
- [x] Session cleanup: `DELETE /mcp` with session ID closes transport and removes from map
- [x] Auto-cleanup: `transport.onclose` removes session from map
- [x] Session ID generated via `randomUUID()`

---

### FR-MCP-006: Health Check Endpoint

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-006`

**Description**:
Unauthenticated `GET /health` endpoint returns service status and current timestamp. Used by container orchestration (Docker health checks) and OpenClaw startup wait scripts.

**Acceptance Criteria**:
- [x] `GET /health` returns HTTP 200 with `{status: "ok", service: "botcore-mcp-server", timestamp: ISO8601}`
- [x] No authentication required
- [x] Response time < 100ms

---

### FR-MCP-007: Tool Registration (105 Tools, 12 Categories)

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-007`

**Description**:
All trading bot capabilities are exposed as MCP tools grouped into 12 categories. Tools use Zod schemas for input validation. Each category is registered via a dedicated `register*Tools(server)` function.

**Tool Categories**:

| Category | Count | File | Purpose |
|---|---|---|---|
| health | 4 | `tools/health.ts` | System health & Docker monitoring |
| market | 8 | `tools/market.ts` | Market data & symbols |
| trading | 4 | `tools/trading.ts` | Live trading positions |
| paper-trading | 28 | `tools/paper-trading.ts` | Paper trading engine |
| real-trading | 14 | `tools/real-trading.ts` | Real trading (CAUTION) |
| ai | 12 | `tools/ai.ts` | AI analysis & predictions |
| tasks | 7 | `tools/tasks.ts` | AI tasks & chat |
| monitoring | 5 | `tools/monitoring.ts` | System & trading metrics |
| settings | 10 | `tools/settings.ts` | API keys & notifications |
| auth | 4 | `tools/auth-tools.ts` | Authentication |
| tuning | 8 | `tools/tuning.ts` | Self-tuning engine |
| notification | 1 | `tools/notification.ts` | Telegram notifications |

**Acceptance Criteria**:
- [x] All 12 `register*Tools(server)` functions called in `createMcpServer()`
- [x] Tool inputs validated with Zod schemas
- [x] Tool descriptions follow security tier labeling (PUBLIC/CAUTION/CRITICAL)
- [x] Tool errors returned as MCP text content (not thrown)

---

### FR-MCP-008: 4-Tier Security Model

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-MCP-008`

**Description**:
Tools are classified into security tiers communicated via tool descriptions. Write and destructive operations require explicit user confirmation before execution.

**Tiers**:
- **PUBLIC**: Read-only queries (market data, health, status)
- **AUTHENTICATED**: Authenticated reads (portfolio, trades, settings)
- **SENSITIVE**: Write operations (settings changes, strategy config)
- **CRITICAL**: Destructive/financial operations (real trading, fund transfers)

**Acceptance Criteria**:
- [x] Tool descriptions label security tier
- [x] Real-trading tools marked CAUTION/CRITICAL in descriptions
- [x] Server instructions warn AI about write operation confirmation

---

## Implementation Notes

| File | Spec Tag | Purpose |
|---|---|---|
| `mcp-server/src/index.ts` | FR-MCP-006, FR-MCP-005 | Express app, `/health`, `/mcp` routing, session map |
| `mcp-server/src/server.ts` | FR-MCP-004, FR-MCP-007 | `createMcpServer()`, tool registration |
| `mcp-server/src/auth.ts` | FR-MCP-002, FR-MCP-004 | Bearer validation, JWT auto-login |
| `mcp-server/src/client.ts` | FR-MCP-003 | `apiRequest()`, timeout, retry, normalization |
| `mcp-server/src/tools/*.ts` | FR-MCP-007, FR-MCP-008 | Tool implementations by category |

**Runtime Config** (env vars):
- `MCP_PORT` — HTTP port (default: `8090`)
- `MCP_AUTH_TOKEN` — Bearer token for incoming requests
- `RUST_API_URL` — Rust engine URL (default: `http://localhost:8080`)
- `PYTHON_API_URL` — Python AI URL (default: `http://localhost:8000`)
- `BOTCORE_EMAIL` / `BOTCORE_PASSWORD` — Credentials for downstream JWT login

---

## Dependencies

- `@modelcontextprotocol/sdk` ^1.12.1 — MCP protocol implementation
- `express` ^4.21.0 — HTTP server
- `zod` ^3.25.1 — Tool input schema validation
- Node.js >= 18 (>= 22 recommended for OpenClaw compatibility)
- Rust Core Engine (FR-AUTH, FR-TRADING specs)
- Python AI Service (FR-AI spec)
