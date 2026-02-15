# BotCore REST API Endpoints - MCP Integration Mapping

**Last Updated**: 2026-02-15
**Scope**: Complete REST API inventory for MCP exposure
**Services**: Rust Core Engine (8080), Python AI Service (8000), Frontend (3000)

---

## SERVICE ARCHITECTURE & PORTS

| Service | Port | Protocol | Purpose |
|---------|------|----------|---------|
| Rust Core Engine | 8080 | HTTP/WS | Main trading engine, paper trading, market data |
| Python AI Service | 8000 | HTTP/WS | GPT-4 analysis, ML predictions, signals |
| Frontend Dashboard | 3000 | HTTP | Next.js UI (reads-only for MCP) |
| MongoDB | 27017 | TCP | Database (internal only) |

---

## ENDPOINT SUMMARY STATISTICS

| Category | Count | Read-Only | Write |
|----------|-------|-----------|-------|
| Market Data | 8 | 7 | 1 |
| Trading | 4 | 2 | 2 |
| Monitoring | 3 | 3 | 0 |
| Settings | 4 | 1 | 3 |
| Notifications | 6 | 2 | 4 |
| Paper Trading | 28 | 16 | 12 |
| Real Trading | 15 | 3 | 12 |
| AI Analysis | 12 | 9 | 3 |
| **TOTAL** | **80** | **43** | **37** |

## MCP SECURITY TIERS

### Tier 1: PUBLIC (No Auth)
- `GET /api/health`, `GET /api/market/*`, `GET /api/monitoring/*`
- `GET /api/ai/info`, `GET /api/paper-trading/status`, `GET /api/paper-trading/portfolio`

### Tier 2: AUTHENTICATED (JWT Token Required)
- All AI endpoints, Paper trading settings/config, Notifications, Account info

### Tier 3: SENSITIVE (2FA Required)
- API key operations, Password changes, Session management, Engine control (start/stop)

### Tier 4: CRITICAL (MFA + Rate Limit + Audit Log)
- Real trading operations (all `/api/real-trading/*`), API key save/delete, Security settings

## RATE LIMITING

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/ai/analyze` | 600 req/min | 1 minute |
| Real trading | 30 req/min | 1 minute |
| Market data | 1000 req/min | 1 minute |
