# MCP Server (Model Context Protocol)

## Quick Reference

### Code Locations
```
mcp-server/
├── src/
│   ├── index.ts              Express server + MCP SDK setup, per-session architecture
│   ├── services/
│   │   └── api-client.ts     HTTP client proxying to Rust (:8080) and Python (:8000)
│   └── tools/
│       ├── ai.ts             AI/ML tools (12 tools)
│       ├── auth-tools.ts     Authentication tools (4 tools)
│       ├── health.ts         Health check tools (3 tools)
│       ├── market.ts         Market data tools (8 tools)
│       ├── monitoring.ts     Monitoring tools (4 tools)
│       ├── notification.ts   Notification tools (1 tool)
│       ├── paper-trading.ts  Paper trading tools (39 tools)
│       ├── real-trading.ts   Real trading tools (14 tools)
│       ├── settings.ts       Settings tools (10 tools)
│       ├── tasks.ts          Task management tools (7 tools)
│       ├── trading.ts        Trading/strategy tools (4 tools)
│       └── tuning.ts         Self-tuning tools (8 tools)
├── tests/                    ~85 tests
└── package.json              @modelcontextprotocol/sdk ^1.12.1
```

### Configuration
- **Port**: 8090
- **Protocol**: MCP v2024-11-05
- **Transport**: Streamable HTTP (Express + StreamableHTTPServerTransport)
- **SDK**: `@modelcontextprotocol/sdk` ^1.12.1

## Architecture

### Per-Session Server Model
SDK requires ONE transport per McpServer instance. Multi-session support = new McpServer per HTTP session.

```
Client Request -> Express -> StreamableHTTPServerTransport -> McpServer (per session)
                                                               |
                                                          Tool Handler
                                                               |
                                                          ApiClient -> Rust API (:8080)
                                                                     -> Python AI (:8000)
```

### Key Implementation Details
- `transport.sessionId` is set DURING `handleRequest()`, not before
- When using `express.json()`, must pass `req.body` as 3rd arg to `transport.handleRequest(req, res, req.body)`
- `registerTool()` expects Zod raw shape (`{ param: z.string() }`), NOT JSON Schema

## Tool Categories (114 tools total)

| Category | Count | Description |
|----------|-------|-------------|
| Paper Trading | 39 | Create/close trades, portfolio, signals |
| Real Trading | 14 | Live Binance trading operations |
| AI/ML | 12 | Predictions, sentiment, analysis |
| Settings | 10 | Configuration management |
| Market Data | 8 | Price feeds, candlesticks, orderbook |
| Self-Tuning | 8 | Parameter optimization (3-tier system) |
| Tasks | 7 | Async task management |
| Strategies | 4 | Active strategies, signals, backtest |
| Monitoring | 4 | System health, service status |
| Auth | 4 | Login, register, verify, user info |
| Health | 3 | Health check endpoints |
| Notifications | 1 | Alert delivery |

## Health Check
```bash
curl http://localhost:8090/health
```

## Testing
- ~85 tests using `InMemoryTransport.createLinkedPair()` for in-process MCP client-server testing
- Run: `cd mcp-server && npm test`

## Dependencies
- Rust Core Engine API (:8080) — primary backend
- Python AI Service (:8000) — ML predictions
- MongoDB — data persistence (via Rust API)

## Related Specs
- `FR-MCP.md` — Functional requirements
- `API-RUST-CORE.md` — Backend API spec
- `API-PYTHON-AI.md` — AI service API spec

**Last Updated**: 2026-03-03
