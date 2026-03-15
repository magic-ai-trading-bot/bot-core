# MCP Server (Model Context Protocol)

## Quick Reference

### Code Locations
```
mcp-server/
├── src/
│   ├── index.ts              Express server, /health, /mcp routes, session map
│   ├── server.ts             createMcpServer(), all register*Tools() calls
│   ├── auth.ts               validateBearerToken(), getJwtToken() with caching
│   ├── client.ts             apiRequest() — HTTP proxy with timeout + retry
│   ├── types.ts              toolSuccess(), toolError(), log()
│   └── tools/
│       ├── health.ts         3 tools: check_system_health, get_service_logs_summary, check_market_condition_health
│       ├── market.ts         8 tools: prices, overview, candles, charts, symbols
│       ├── trading.ts        4 tools: positions, account, performance (live Binance)
│       ├── paper-trading.ts  39 tools: full paper trading engine control
│       ├── real-trading.ts   14 tools: real money trading (CRITICAL)
│       ├── ai.ts             12 tools: analysis, predictions, storage, history
│       ├── tasks.ts          7 tools: async tasks, chat
│       ├── monitoring.ts     3 tools: system/trading/connection metrics
│       ├── settings.ts       10 tools: API keys, notifications, push
│       ├── auth-tools.ts     4 tools: login, register, profile, refresh
│       ├── tuning.ts         8 tools: self-tuning parameter adjustments
│       └── notification.ts   1 tool: send_telegram_notification
├── tests/                    89 tests (InMemoryTransport pairs)
└── package.json              @modelcontextprotocol/sdk ^1.12.1
```

### Quick Commands
```bash
curl http://localhost:8090/health          # Health check (no auth)
cd mcp-server && npm test                  # Run 89 tests
cd mcp-server && npm run build             # Build TypeScript
```

### Configuration
| Env Var | Default | Purpose |
|---|---|---|
| `MCP_PORT` | `8090` | HTTP listen port |
| `MCP_AUTH_TOKEN` | (unset) | Bearer token for incoming requests |
| `RUST_API_URL` | `http://localhost:8080` | Rust engine base URL |
| `BOTCORE_EMAIL` | (unset) | Credentials for downstream JWT |
| `BOTCORE_PASSWORD` | (unset) | Credentials for downstream JWT |

---

## Architecture

### Per-Session Server Model

SDK requirement: ONE `McpServer` + ONE `StreamableHTTPServerTransport` per session. Multi-client support via session map keyed by UUID.

```
OpenClaw/Claude
    |
    | POST /mcp  (Authorization: Bearer <MCP_AUTH_TOKEN>)
    v
Express (auth middleware)
    |
    v
Session Map  ─── new session ──►  McpServer + StreamableHTTPServerTransport
    |                                    |
    └─── existing session-id ────────────┘
                                         |
                                    Tool Handler
                                         |
                              apiRequest("rust")
                                         |
                              ┌──────────┴──────────┐
```

### Key Implementation Details

- `transport.sessionId` set **during** `handleRequest()` — store in map **after** call
- `express.json()` body must be passed as 3rd arg: `transport.handleRequest(req, res, req.body)`
- `registerTool()` takes Zod raw shape `{ param: z.string() }`, not JSON Schema
- Session cleanup: `transport.onclose` auto-removes from map; `DELETE /mcp` explicit close
- New session: `POST /mcp` without `mcp-session-id` header

### Request Flow

```
1. POST /mcp
2. validateBearerToken(req.headers.authorization)  → 401 if invalid
3. Get/create session transport
4. transport.handleRequest(req, res, req.body)
5. Tool handler fires → apiRequest("rust", path, opts)
6. getJwtToken()  → auto-login if expired (POST /api/auth/login)
7. fetch(RUST_API_URL + path, {Authorization: Bearer <jwt>})
8. Normalize response → toolSuccess(data) | toolError(msg)
```

---

## Security Model

### 4-Tier Classification

| Tier | Confirmation | Examples |
|---|---|---|
| **PUBLIC** | None — read-only, anonymous | `get_market_prices`, `check_system_health` |
| **AUTHENTICATED** | None — reads requiring JWT | `get_paper_portfolio`, `get_paper_open_trades` |
| **SENSITIVE** | Recommended user confirmation | `update_paper_basic_settings`, `update_real_trading_settings` |
| **CRITICAL** | Required user typing "APPROVE" | `create_real_order`, `start_real_engine`, `cancel_all_real_orders` |

Real-trading tools carry `CAUTION` / `CRITICAL` labels in their descriptions. The server instructions injected into the AI system prompt warn against executing write/critical tools without explicit user confirmation.

### Authentication

- **Inbound**: Bearer token (`MCP_AUTH_TOKEN` env). Missing token → all requests allowed with warning (dev mode).
- **Outbound**: JWT auto-login via `BOTCORE_EMAIL`/`BOTCORE_PASSWORD`. Token cached in memory; refreshed when expiry < `now + 1h`.

---

## API Client (`src/client.ts`)

```typescript
apiRequest(
  target: "rust",
  path: string,
  opts?: { method?, body?, skipAuth?, timeoutMs? }
): Promise<{ success: boolean, data?: unknown, error?: string }>
```

- Default timeout: 30 seconds (`AbortController`)
- Auto-retry: 1 retry on 5xx GET errors (2-second delay)
- Response normalization: Rust `{success, data}` format
- `skipAuth: true` bypasses JWT injection (health checks)

---

## Tool Inventory (113 tools)

### health (3 tools) — `tools/health.ts`

| Tool | Description |
|---|---|
| `get_service_logs_summary` | Recent errors from Rust services |
| `check_market_condition_health` | Deep AI pipeline check: MongoDB → indicators → model |

### market (8 tools) — `tools/market.ts`

| Tool | Description |
|---|---|
| `get_market_prices` | Current prices for all tracked symbols |
| `get_market_overview` | 24h stats: change %, volume, trending |
| `get_candles` | OHLCV candles by symbol + interval |
| `get_chart` | Single-symbol chart data |
| `get_multi_charts` | Multi-symbol chart comparison |
| `get_symbols` | List all tracked symbols |
| `add_symbol` | Add symbol to tracking list |
| `remove_symbol` | Remove symbol from tracking list |

### trading (4 tools) — `tools/trading.ts`

| Tool | Description |
|---|---|
| `get_trading_positions` | Live Binance futures positions |
| `get_trading_account` | Binance account balances |
| `close_trading_position` | Close a live position by symbol |
| `get_trading_performance` | Win rate, PnL, trade stats |

### paper-trading (39 tools) — `tools/paper-trading.ts`

**Read tools (17)**:

| Tool | Description |
|---|---|
| `get_paper_trading_status` | Engine running/stopped, active symbol count |
| `get_paper_portfolio` | Balance, equity, unrealized PnL |
| `get_paper_open_trades` | All open paper positions |
| `get_paper_closed_trades` | Trade history with PnL |
| `get_paper_strategy_settings` | RSI, MACD, Bollinger config |
| `get_paper_basic_settings` | Risk params: SL, TP, leverage, mode flags |
| `get_paper_symbols` | Symbols the paper engine monitors |
| `get_paper_indicator_settings` | Indicator thresholds |
| `get_paper_trade_analyses` | All AI-generated trade analyses |
| `get_paper_trade_analysis` | Single trade analysis by ID |
| `get_paper_config_suggestions` | AI config improvement suggestions |
| `get_paper_latest_config_suggestions` | Most recent suggestions |
| `get_paper_signals_history` | Historical signal log |
| `get_paper_latest_signals` | Most recent signals per symbol |
| `get_paper_pending_orders` | Pending/queued orders |
| `get_atr_diagnostics` | ATR indicator diagnostics |
| `get_signal_quality_report` | Signal quality metrics |

**Settings tools (7)**:

| Tool | Description |
|---|---|
| `get_paper_signal_pipeline_settings` | Signal pipeline config |
| `get_paper_execution_settings` | Slippage, fill simulation settings |
| `get_paper_ai_settings` | AI service config for paper engine |
| `get_paper_notification_settings` | Alert preferences |
| `update_paper_strategy_settings` | Modify RSI/MACD/Bollinger params |
| `update_paper_basic_settings` | Modify risk params (SL, TP, leverage, mode) |
| `update_paper_symbols` | Add/remove monitored symbols |

**Action tools (8)**:

| Tool | Description |
|---|---|
| `update_paper_indicator_settings` | Modify indicator thresholds |
| `update_paper_signal_pipeline_settings` | Modify signal pipeline |
| `update_paper_signal_interval` | Change signal polling interval |
| `update_paper_settings` | Generic settings update |
| `update_paper_execution_settings` | Modify execution simulation |
| `update_paper_ai_settings` | Modify AI config |
| `update_paper_notification_settings` | Modify alert preferences |
| `reset_paper_account` | Reset paper balance to default |

**Engine/order tools (7)**:

| Tool | Description |
|---|---|
| `start_paper_engine` | Start the paper trading engine |
| `stop_paper_engine` | Stop the paper trading engine |
| `create_paper_order` | Manually create a paper trade |
| `cancel_paper_order` | Cancel a pending paper order |
| `close_paper_trade` | Close a paper trade by ID |
| `close_paper_trade_by_symbol` | Close all positions for a symbol |
| `trigger_paper_analysis` | Trigger AI analysis for a trade |

### real-trading (14 tools) — `tools/real-trading.ts` — CRITICAL

| Tool | Security | Description |
|---|---|---|
| `get_real_trading_status` | AUTHENTICATED | Engine status |
| `get_real_portfolio` | AUTHENTICATED | Live balance/equity |
| `get_real_open_trades` | AUTHENTICATED | Live open positions |
| `get_real_closed_trades` | AUTHENTICATED | Live trade history |
| `get_real_trading_settings` | AUTHENTICATED | Current real settings |
| `get_real_orders` | AUTHENTICATED | Open orders on Binance |
| `start_real_engine` | CRITICAL | Start real trading (money at risk) |
| `stop_real_engine` | CRITICAL | Stop real trading engine |
| `close_real_trade` | CRITICAL | Close a live trade |
| `update_real_trading_settings` | SENSITIVE | Modify real trading params |
| `create_real_order` | CRITICAL | Place a real order on Binance |
| `cancel_real_order` | CRITICAL | Cancel a real Binance order |
| `cancel_all_real_orders` | CRITICAL | Cancel all open orders |
| `update_real_position_sltp` | CRITICAL | Modify SL/TP on live position |

### ai (12 tools) — `tools/ai.ts`

| Tool | Description |
|---|---|
| `analyze_market` | Full AI market analysis for a symbol |
| `get_strategy_recommendations` | AI-recommended strategy params |
| `get_market_condition` | Market direction + confidence score |
| `send_ai_feedback` | Submit feedback on AI decisions |
| `get_ai_info` | AI service version, model info |
| `get_ai_strategies` | Active AI strategy configs |
| `get_ai_performance` | AI prediction accuracy stats |
| `get_ai_storage_stats` | AI data storage usage |
| `clear_ai_storage` | Clear cached AI data |
| `get_ai_cost_statistics` | Token/API cost breakdown |
| `get_ai_config_suggestions` | AI-generated config improvements |
| `get_ai_analysis_history` | Historical analysis records |

### tasks (7 tools) — `tools/tasks.ts`

| Tool | Description |
|---|---|
| `trigger_config_analysis` | Kick off async config analysis |
| `predict_trend` | Request ML trend prediction |
| `get_ai_config_suggestions_python` | Config suggestions from AI analysis |
| `chat_with_project` | Chat with project context |
| `get_chat_suggestions` | Get suggested follow-up questions |
| `clear_chat_history` | Clear conversation history |
| `get_ai_debug_info` | Debug info for AI service |

### monitoring (3 tools) — `tools/monitoring.ts`

| Tool | Description |
|---|---|
| `get_system_monitoring` | CPU, memory, uptime, cache from Rust |
| `get_trading_metrics` | Win rate, PnL, positions from Rust |
| `get_connection_status` | WebSocket + API connection status |

### settings (10 tools) — `tools/settings.ts`

| Tool | Description |
|---|---|
| `get_api_keys` | List configured Binance API keys |
| `save_api_keys` | Store Binance API credentials |
| `delete_api_keys` | Remove API keys |
| `test_api_keys` | Validate API keys against Binance |
| `get_notification_preferences` | Alert channel preferences |
| `update_notification_preferences` | Modify alert settings |
| `subscribe_push_notifications` | Register push notification endpoint |
| `unsubscribe_push_notifications` | Remove push endpoint |
| `test_notification` | Send test notification |
| `get_vapid_key` | Get VAPID public key for push |

### auth (4 tools) — `tools/auth-tools.ts`

| Tool | Description |
|---|---|
| `login` | Authenticate and get JWT |
| `register_user` | Create new user account |
| `get_profile` | Get current user profile |
| `refresh_token` | Refresh JWT token |

### tuning (8 tools) — `tools/tuning.ts`

| Tool | Description |
|---|---|
| `get_tuning_dashboard` | All params + recent performance |
| `get_parameter_bounds` | Min/max bounds per parameter |
| `apply_green_adjustment` | Auto-apply tier-1 safe param change |
| `request_yellow_adjustment` | Request tier-2 param change (needs confirm) |
| `request_red_adjustment` | Request tier-3 critical change (needs approve) |
| `get_adjustment_history` | All past tuning actions |
| `rollback_adjustment` | Revert to previous parameter value |
| `take_parameter_snapshot` | Snapshot current params for rollback |

### notification (1 tool) — `tools/notification.ts`

| Tool | Description |
|---|---|
| `send_telegram_notification` | Send message to configured Telegram chat |

---

## Session Management

### Lifecycle

```
POST /mcp (no mcp-session-id)
  → create McpServer + StreamableHTTPServerTransport
  → transport.handleRequest(req, res, req.body)
  → transport.sessionId available now → store in map
  → response includes mcp-session-id header

POST /mcp (with mcp-session-id: <id>)
  → look up existing transport from map
  → transport.handleRequest(req, res, req.body)

DELETE /mcp (with mcp-session-id: <id>)
  → transport.close()
  → remove from map

transport.onclose
  → auto-remove from map (handles dropped connections)
```

### Session Store

```typescript
const sessions = new Map<string, {
  server: McpServer,
  transport: StreamableHTTPServerTransport
}>();
```

---

## Example MCP Call / Response

### Request (POST /mcp)
```http
POST /mcp HTTP/1.1
Authorization: Bearer <MCP_AUTH_TOKEN>
Content-Type: application/json

{
  "jsonrpc": "2.0",
  "method": "tools/call",
  "params": {
    "name": "get_paper_portfolio",
    "arguments": {}
  },
  "id": 1
}
```

### Response
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{
      "type": "text",
      "text": "{\"balance\": 10000, \"equity\": 10342.5, \"unrealized_pnl\": 342.5}"
    }]
  },
  "id": 1
}
```

### Error Response
```json
{
  "jsonrpc": "2.0",
  "result": {
    "content": [{
      "type": "text",
      "text": "Error: Failed to fetch portfolio: connection refused"
    }],
    "isError": true
  },
  "id": 1
}
```

---

## Testing

- **Framework**: Vitest + `InMemoryTransport.createLinkedPair()` for in-process client-server
- **Count**: 89 tests
- **Coverage**: Tool registration, session lifecycle, auth, error paths

```bash
cd mcp-server && npm test
cd mcp-server && npm run test:coverage
```

---

## Dependencies

| Package | Version | Purpose |
|---|---|---|
| `@modelcontextprotocol/sdk` | ^1.12.1 | MCP protocol (McpServer, StreamableHTTPServerTransport) |
| `express` | ^4.21.0 | HTTP server |
| `zod` | ^3.25.1 | Tool input schema validation |
| Node.js | >= 18 (>= 22 recommended) | Runtime |

---

## Related Specs

- `FR-MCP.md` — Functional requirements (protocol, auth, session, tools)
- `API-RUST-CORE.md` — Rust engine API that tools proxy to

**Last Updated**: 2026-03-03
