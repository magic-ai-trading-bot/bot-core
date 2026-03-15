# Codebase Summary

**Last Updated**: 2026-03-15
**Project**: Bot Core - AI-Powered Cryptocurrency Trading Platform
**Total Files**: 119 specification documents + source code
**Status**: Consolidated documentation with unified specifications/ structure

---

## Project Overview

Bot Core is a sophisticated multi-service cryptocurrency trading platform with AI/ML integration, real-time WebSocket support, and comprehensive risk management. The system executes trading strategies, manages portfolios, analyzes market data, and provides both paper trading (simulation) and real trading capabilities.

### Core Philosophy
- **Spec-Driven Development**: All features documented before implementation
- **Finance-Critical**: Every decision validated against security and risk standards
- **High Automation**: 1,937+ tests (Rust 1,336 + Frontend 601), 90.4% coverage
- **Multi-Service Architecture**: 6 independent services + 1 MCP server + 1 OpenClaw gateway

---

## Technology Stack

### Backend Services
- **Rust Core Engine** (`rust-core-engine/`)
  - Framework: Actix-web 4.x
  - Database: MongoDB with 17 collections
  - Key Libraries: tokio (async), serde (serialization)
  - Modules: Paper trading, real trading, risk management, strategies, market data

- **Node.js MCP Server** (`mcp-server/`)
  - SDK: @modelcontextprotocol/sdk v1.12.1+
  - Transport: Streamable HTTP via Express
  - Tools: 110 tools across 12 categories
  - Port: 8090
  - Features: Per-session server, health checks, tool registry

- **OpenClaw Gateway** (`openclaw/`)
  - Runtime: Node.js 22+
  - Integration: Telegram bot, AI Claude Sonnet 4.5
  - Transport: WebSocket gateway (port 18789)
  - Bridge: botcore-bridge.mjs (MCP client)
  - Config: openclaw.json + production config

### Frontend
- **Next.js Dashboard** (`nextjs-ui-dashboard/`)
  - Framework: React 18, Vite, Next.js
  - UI: Shadcn/UI, TailwindCSS
  - Components: 71 total, fully typed TypeScript
  - Features: Real-time WebSocket, portfolio visualization, strategy management
  - Tests: 601 comprehensive test cases

### Infrastructure
- **Docker Compose**: 6 services (Rust API, MCP, OpenClaw, Postgres, Redis, Monitoring)
- **MongoDB**: Replica sets for consistency
- **VPS**: 16GB RAM on Viettel (180.93.2.247)
- **CI/CD**: GitHub Actions with automated testing, linting, security scanning

---

## Service Architecture

```
┌─────────────────────────────────────────────────────────┐
│              Frontend Dashboard (3000)                   │
│  React 18 + Shadcn/UI + Real-time WebSocket Updates    │
└────────────────┬────────────────────────────────────────┘
                 │
    ┌────────────┼────────────┬──────────────┐
    │            │            │              │
┌───▼──┐  ┌─────▼──┐  ┌───────▼───┐  ┌──────▼────┐
│Rust  │  │ MCP    │  │ OpenClaw  │  │ Postgres  │
│API   │  │ Server │  │ Gateway   │  │  (SQL)    │
│:8080 │  │ :8090  │  │ :18789    │  │ :5432     │
│      │  │        │  │           │  │           │
│Core  │  │Tools   │  │Telegram   │  │ Metadata  │
│Engine│  │Health  │  │Bridge     │  │ & Config  │
└───┬──┘  └────┬───┘  └─────┬─────┘  └───────────┘
    │          │            │
    └──────────┴────────────┘

    Shared: MongoDB (Trading Data, Portfolios, Strategies, Signals, Market Data)
            Redis (Caching, Real-time Updates)
```

---

## Core Modules

### 1. Paper Trading Engine
**Location**: `rust-core-engine/src/paper_trading/`
**Files**: `engine.rs`, `portfolio.rs`, `trade.rs`, `settings.rs`

**Features**:
- Execution simulation with slippage & market impact
- Partial fill handling
- Risk management (5% daily loss limit, 60-min cool-down)
- Correlation limits (70%), consecutive loss tracking
- Latency tracking for realistic simulation

**Key Types**:
- `PaperTradingEngine` - Main execution engine
- `Trade` - Individual trade record
- `Portfolio` - Aggregated holdings
- `RiskSettings` - Configurable risk parameters

### 2. Trading Strategies
**Location**: `rust-core-engine/src/strategies/`
**Strategies**: RSI, MACD, Bollinger Bands, Volume, Combined Engine
**Performance**: 65% combined win rate, 1.5% avg profit, Sharpe 1.6

**Indicators**: ATR, SMA, EMA, Stochastic, RSI, MACD, Bollinger Bands, Volume

### 3. Real Trading
**Location**: `rust-core-engine/src/trading/`
**Exchange**: Binance API integration
**Modes**: Testnet (default), Production (with explicit enable)

**Features**:
- Order management (market, limit, stop-loss, take-profit)
- Order status tracking
- Partial fill handling
- Fee calculation

### 4. Risk Management
**Location**: `rust-core-engine/src/paper_trading/engine.rs`

**Metrics**:
- Daily loss limit: 5% of portfolio
- Cool-down: 60 minutes after 5 consecutive losses
- Correlation limit: 70% max position correlation
- Consecutive loss tracking: Auto-pause after threshold

### 5. Market Data
**Location**: `rust-core-engine/src/binance/`

**Features**:
- Real-time WebSocket feeds (price, volume, candles)
- Kline (candle) data collection
- Market data caching
- Data validation & normalization

### 6. Authentication & Authorization
**Location**: `rust-core-engine/src/auth/`

**Security**:
- JWT with HS256 (HMAC-SHA256)
- Bcrypt password hashing
- Token refresh flow
- Middleware-based route protection

**Endpoints**:
- `POST /api/auth/login` - Authenticate user
- `POST /api/auth/register` - Create account
- `POST /api/auth/verify` - Verify token
- `GET /api/auth/profile` - User profile

### 7. WebSocket & Real-Time
**Location**: `rust-core-engine/src/binance/websocket.rs` + `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

**Events**:
- `price_update` - Market price change
- `signal_generated` - New trading signal
- `trade_executed` - Trade completion
- `portfolio_update` - Position changes
- `risk_event` - Risk limit triggered

### 8. MCP Server
**Location**: `mcp-server/`

**Architecture**:
- Express HTTP server with MCP SDK
- Per-session server architecture
- Streamable HTTP transport
- 110 tools across 12 categories

**Tool Categories**:
1. Health (3) - Server health checks
2. Market (8) - Market data queries
3. Trading (4) - Trade execution status
4. Paper Trading (39) - Simulation & backtest
5. Real Trading (14) - Binance live trading
6. AI (12) - Model & signal access
7. Tasks (7) - Async job management
8. Monitoring (4) - Server metrics
9. Settings (10) - Configuration management
10. Auth (4) - User management
11. Tuning (8) - Self-tuning engine

**Health Endpoint**: `curl http://localhost:8090/health`

### 9. OpenClaw Gateway
**Location**: `openclaw/`

**Integration**:
- Telegram bot (via `TELEGRAM_BOT_TOKEN`)
- Claude.ai web sessions
- Skill injection via SKILL.md
- Bridge script (botcore-bridge.mjs) as MCP client

**Features**:
- Cron job scheduling
- WebSocket gateway (port 18789)
- Device pairing for auth
- Session-based message routing
- Timeout: 30s per command, 2 retries with backoff

**Config**:
- Dev: `~/.openclaw/openclaw.json`
- Prod: `~/.openclaw/openclaw.production.json`
- Jobs: `~/.openclaw/cron/jobs.json` (loaded at startup)

---

## Database Schema

**Database**: MongoDB with 17 collections

### Collections
1. **users** - User accounts, auth credentials
2. **paper_portfolios** - Simulated portfolio state
3. **paper_trades** - Historical simulation trades
4. **real_portfolios** - Live trading positions
5. **real_trades** - Live trade history
6. **strategies** - Strategy definitions & config
7. **market_data** - Cached OHLCV candles
8. **signals** - Generated trading signals
9. **portfolio_snapshots** - Portfolio P&L history
10. **trade_logs** - Detailed trade event logs
11. **risk_events** - Risk limit breach records
12. **settings** - User configuration
13. **ai_models** - Model metadata & performance
14. **backtests** - Strategy backtest results
15. **monitoring_metrics** - Performance metrics
16. **notifications** - Alert log
17. **async_tasks** - Job queue

### Indexes
**Total**: 37 optimized indexes for common queries
- User lookups (email, username)
- Portfolio queries (date ranges, status)
- Trade history (symbol, date, status)
- Signal queries (timestamp, signal type)
- Strategy lookups (name, active status)

---

## API Endpoints

### Rust Core API (:8080)
- `GET /api/health` - Service health
- `GET /api/strategies/active` - List active strategies
- `GET /api/strategies/signals/{symbol}` - Get signals by symbol
- `POST /api/strategies/backtest` - Run backtest
- `GET /api/portfolio` - Current portfolio
- `GET /api/trades` - Trade history
- `POST /api/trades/close/{id}` - Close position
- `POST /api/auth/login` - User login
- `POST /api/auth/register` - User registration

### MCP Server (:8090)
- `POST /mcp/call` - Tool invocation
- `GET /health` - Server health

### OpenClaw Gateway (:18789)
- WebSocket: `/gateway` - Real-time message routing
- REST: `/health` - Gateway status

---

## Testing Strategy

### Test Coverage
- **Total Tests**: 1,937+
  - Rust: 1,336 (unit, integration, end-to-end)
  - Frontend: 601 (unit, integration, snapshot)
- **Coverage Target**: 95% (current: 90.4%)
- **Mutation Score**: 84%

### Test Types
1. **Unit Tests** - Individual functions/modules
2. **Integration Tests** - Service-to-service interactions
3. **Snapshot Tests** - UI component regression
4. **Performance Tests** - Latency, throughput, memory
5. **Security Tests** - Auth, encryption, data validation
6. **Scenario Tests** - Real-world trading workflows

### Key Test Suites
- `rust-core-engine/tests/test_paper_trading.rs` - Trading engine
- `rust-core-engine/tests/test_auth.rs` - Authentication
- `rust-core-engine/tests/test_strategies.rs` - Strategy logic
- `nextjs-ui-dashboard/src/**/*.test.tsx` - UI tests
- `mcp-server/tests/` - MCP tool tests

---

## Security Standards

### Authentication
- **JWT Algorithm**: HS256 (HMAC-SHA256)
- **Token Lifetime**: 24 hours (configurable)
- **Refresh Token**: Optional rotating refresh tokens
- **Password Hashing**: bcrypt (cost factor 12)

### API Security
- **CORS**: Configured per environment
- **Rate Limiting**: Token-based (10 req/min per endpoint)
- **Request Validation**: JSON Schema validation
- **Response Headers**: Security headers (HSTS, X-Frame-Options, etc.)

### Data Security
- **Encryption in Transit**: HTTPS/TLS
- **Encryption at Rest**: MongoDB native encryption
- **Secret Management**: Environment variables, .env files
- **Audit Logging**: All auth & critical operations logged

### Code Quality
- **Zero Unwrap Policy**: Rust requires `?` operator, no `unwrap()`
- **Type Safety**: Full TypeScript strict mode
- **Linting**: clippy (Rust), ESLint (TypeScript)
- **Security Scanning**: Trivy, TruffleHog (secrets detection)

---

## Development Workflow

### Quick Start
```bash
cp .env.example .env
./scripts/generate-secrets.sh
./scripts/bot.sh start --memory-optimized
./scripts/bot.sh dev    # Hot reload
./scripts/bot.sh status # Check status
```

### Build & Test
```bash
make build          # Build all services
make test           # Run 2,202+ tests
make lint           # Check all code quality
```

### Service-Specific Commands
```bash
# Rust
cd rust-core-engine
cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Frontend
cd nextjs-ui-dashboard
npm run lint && npm run type-check && npm test
```

### Git Workflow
1. Create feature branch: `git checkout -b feat/feature-name`
2. Follow spec-driven workflow: CREATE SPEC → IMPLEMENT → TEST → DOCS
3. Commit with conventional format: `feat:`, `fix:`, `docs:`, `test:`
4. Create PR, pass all checks (lint, test, security)
5. Merge to main

---

## Deployment

### Environments
- **Development**: Testnet trading, live reload, debug logs
- **Staging**: Live data, paper trading, monitored
- **Production**: Live trading, optimized, backup/recovery

### Docker Services
```
bot-core
├── rust-api (Rust core engine)
├── mcp-server (Tool provider)
├── openclaw (Telegram gateway)
├── postgres (SQL metadata)
├── redis (Cache/queue)
└── monitoring (Prometheus/Grafana)
```

### VPS Deployment
- **Host**: Viettel VPS (180.93.2.247)
- **RAM**: 16GB (8GB per core pair)
- **Entrypoint**: `openclaw/scripts/entrypoint.sh`
- **Auto-Start**: Systemd service + Docker restart policy

---

## Key Features

### Paper Trading
- 100% accurate simulation with real-world conditions
- Slippage modeling, partial fills, latency tracking
- Risk management with automated cool-downs
- Backtest engine for strategy validation

### Real Trading
- Live Binance integration
- Testnet validation before live
- Order lifecycle management
- Position & P&L tracking

### AI Signals
- Multi-model ensemble (72% accuracy)
- Real-time prediction updates
- Confidence scoring
- Model performance monitoring

### Risk Management
- Daily loss limits (5%)
- Cool-down periods (60 minutes)
- Correlation-based position sizing
- Automated circuit breakers

### Monitoring
- Real-time WebSocket dashboard
- Portfolio P&L visualization
- Trade history & analytics
- System health monitoring

---

## Configuration

### Environment Variables (`.env`)
```bash
# API Keys
BINANCE_API_KEY=...
BINANCE_API_SECRET=...

# Database
MONGODB_URL=mongodb://localhost:27017/bot-core
DATABASE_NAME=bot-core

# Trading
TRADING_ENABLED=false      # Enable live trading
BINANCE_TESTNET=true       # Use testnet

# MCP Server
MCP_PORT=8090
MCP_HOST=0.0.0.0

# Frontend
NEXT_PUBLIC_API_URL=http://localhost:8080
NEXT_PUBLIC_WS_URL=ws://localhost:8080

# Telegram (OpenClaw)
TELEGRAM_BOT_TOKEN=...
TELEGRAM_USER_ID=...

# Other
LOG_LEVEL=info
MEMORY_LIMIT=8GB
```

---

## File Size Management

**Target Documentation Size**: 800 LOC per file

### Strategy
- Split large topics into `{topic}/` directories
- Each subtopic in separate file: `{topic-1}.md`, `{topic-2}.md`
- Central index: `{topic}/index.md` with navigation

### Example
```
specifications/
├── 06-features/
│   ├── paper-trading.md
│   ├── ai-integration.md
│   ├── trading-strategies.md
│   └── ...
```

---

## Performance Metrics

### System
- API Response Time: <200ms (p95)
- WebSocket Latency: <50ms
- Database Query: <100ms (p95)
- Memory Usage: ~2GB per service (optimized)

### Trading
- Strategy Signal Latency: <1s
- Order Execution: <5s (with network)
- Portfolio Update: Real-time (WebSocket)
- Backtest Speed: 1000 trades/sec

### Testing
- Full test suite: ~2 minutes
- Unit test only: ~30 seconds
- Coverage threshold: 95%

---

## Documentation Structure

All documentation unified under `specifications/`:

```
specifications/
├── 01-requirements/          # 26 functional, non-functional, user story specs
├── 02-design/               # Architecture, API, database, UI design
├── 03-testing/              # Test plans, cases, scenarios
├── 04-deployment/           # Infrastructure, CI/CD, monitoring
├── 05-operations/           # Runbooks, troubleshooting, disaster recovery
├── 06-features/             # Feature guides (paper-trading, AI, etc.)
├── assets/                  # Screenshots, diagrams
├── README.md                # Navigation hub
├── TRACEABILITY_MATRIX.md   # Requirement → code → test mapping
└── _SPEC_TEMPLATE.md        # Template for new specs
```

---

## Troubleshooting

### Common Issues
1. **OOM (Out of Memory)**: Use `--memory-optimized` flag
2. **Port Conflict**: Check with `lsof -i :PORT`
3. **Build Failure**: Run `make build-fast` (sequential)
4. **Test Failure**: Read full output, check for flaky tests
5. **Database Connection**: Verify MongoDB replica sets

### Logs
```bash
./scripts/bot.sh logs --service <name>
# Services: rust-api, mcp-server, openclaw
```

### Health Checks
```bash
curl localhost:8080/api/health      # Rust API
curl localhost:8090/health          # MCP Server
curl localhost:18789/health         # OpenClaw Gateway
```

---

## Contributing

### Before You Code
1. Check `CLAUDE.md` for project navigation
2. Read relevant spec in `specifications/01-requirements/`
3. Check `TRACEABILITY_MATRIX.md` for related code
4. Read existing code in referenced files

### During Development
1. Follow code standards in `specifications/02-design/`
2. Write tests alongside code
3. Ensure 95% code coverage
4. Run linting before commit

### Before You Commit
1. Run `make lint && make test`
2. Ensure zero breaking changes
3. Write clear commit message
4. Update relevant specs if needed

---

## Key Statistics

- **Services**: 6 (Rust, Node.js, OpenClaw, Postgres, Redis, Monitoring)
- **Specification Files**: 119
- **Code Coverage**: 90.4% (target: 95%)
- **Test Count**: 1,937+
- **Code Quality**: A (94/100)
- **Security Score**: A+ (98/100)
- **Mutation Score**: 84%
- **Lines of Code**: ~130K across all services
- **Documentation**: ~100K lines across specifications/

---

## Next Steps & Roadmap

See `specifications/README.md` for complete documentation navigation and `specifications/TASK_TRACKER.md` for development roadmap.

For specific features, consult:
- `specifications/06-features/` for feature guides
- `specifications/01-requirements/` for detailed requirements
- `specifications/03-testing/` for test coverage information
- `CLAUDE.md` for quick navigation by feature

