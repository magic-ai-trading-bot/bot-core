# CLAUDE.md - Smart Navigation Hub

**Purpose**: Quickly find exact code locations and documentation for any feature.

---

## CRITICAL: SPEC-DRIVEN DEVELOPMENT WORKFLOW

**THIS IS A FINANCE PROJECT - MISTAKES = MONEY LOSS**

### Mandatory Workflow: `USER REQUEST → CREATE SPEC → USE AGENTS → UPDATE SPECS → DONE`

#### Step 1: CREATE SPEC FIRST (Before ANY code)

1. **Search existing specs**: `specifications/01-requirements/1.1-functional-requirements/FR-*.md`
2. **Create new spec**: `specifications/01-requirements/1.1-functional-requirements/FR-XXX.md` (use `specifications/_SPEC_TEMPLATE.md`)
3. **Add to** `TRACEABILITY_MATRIX.md`
4. **Create test cases**: `specifications/03-testing/3.2-test-cases/TC-XXX.md` (Gherkin format)

**DO NOT write ANY code until specs are complete!**

#### Step 2: USE AGENTS (MANDATORY for complex features)

See Agent Quick Reference below. **NEVER code manually** for complex features.

#### Step 3: ADD @spec TAGS TO CODE

```rust
// @spec:FR-RISK-007 - Trailing Stop Loss
// @ref:specifications/01-requirements/1.1-functional-requirements/FR-RISK.md
// @test:TC-TRADING-054, TC-TRADING-055
pub fn update_trailing_stop(...) -> Result<...> { ... }
```

#### Step 4: UPDATE SPECS & DOCS

- Update requirement checkboxes: `- [ ]` → `- [x]`
- Update `TRACEABILITY_MATRIX.md` status
- Run: `python3 scripts/validate-specs.py` (must show 0 errors)

#### Step 5: VERIFY TRACEABILITY

```bash
grep -r "FR-XXX-YYY" specifications/01-requirements/          # Requirement exists
grep "FR-XXX-YYY" specifications/TRACEABILITY_MATRIX.md        # In matrix
grep -r "@spec:FR-XXX-YYY" rust-core-engine/          # Has @spec tag
python3 scripts/validate-specs.py                      # Validation passes
```

---

## Agent Quick Reference

### Core Development
| Command | Purpose |
|---------|---------|
| `/plan [task]` | Research & create implementation plan → `./plans/` |
| `/cook [tasks]` | Full-cycle: research → plan → implement → test → review → docs |
| `/test` | Run all tests locally, analyze coverage |
| `/debug [issue]` | Find root cause with debugger subagent |
| `/watzup` | Review recent changes, wrap up session |

### Questions & Research
| Command | Purpose |
|---------|---------|
| `/ask [question]` | Architectural consultation (4 expert advisors) |
| `/brainstorm [question]` | Explore approaches with pros/cons analysis |
| `/scout [prompt] [scale]` | Fast codebase search (scale: 1-3 gemini, 4-5 opencode, 6+ parallel) |

### Fixing Issues
| Command | Purpose |
|---------|---------|
| `/fix:fast [issue]` | Quick fix for simple bugs |
| `/fix:hard [issue]` | Complex fix with full subagent orchestration |
| `/fix:test` | Run test suite and fix failures |
| `/fix:types` | Fix TypeScript type errors |
| `/fix:ui [issue]` | Fix UI/UX issues with designer subagent |
| `/fix:ci [url]` | Analyze & fix CI/CD failures from GitHub Actions |
| `/fix:logs [issue]` | Analyze logs and fix issues |

### Design & UI/UX
| Command | Purpose |
|---------|---------|
| `/design:good [tasks]` | High-quality design (Dribbble/Awwwards level) |
| `/design:fast [tasks]` | Quick functional design |
| `/design:3d [tasks]` | 3D interactive with Three.js |
| `/design:screenshot [img]` | Design based on screenshot |
| `/design:video [vid]` | Design based on video |
| `/design:describe [img]` | Analyze design elements |

### Documentation & Git
| Command | Purpose |
|---------|---------|
| `/docs:init` | Create initial documentation |
| `/docs:update` | Update all docs to match codebase |
| `/git:cm` | Stage all & commit |
| `/git:cp` | Stage, commit & push |
| `/git:pr [branch]` | Create pull request |

### Content & Integration
| Command | Purpose |
|---------|---------|
| `/content:good [req]` | High-quality copy |
| `/content:fast [req]` | Quick copy |
| `/integrate:sepay [tasks]` | SePay.vn payment integration |
| `/integrate:polar [tasks]` | Polar.sh payment integration |
| `/bootstrap:auto [req]` | Bootstrap new project |

---

## Proactive Agent Suggestions

Claude MUST suggest agents when detecting these patterns:

| User Pattern | Suggest |
|---|---|
| bugs/errors/broken/crash | Simple → `/fix:fast`, Complex → `/fix:hard`, UI → `/fix:ui`, CI → `/fix:ci` |
| add/implement/create/build | Complex → `/plan` then `/cook`, Simple → `/cook` directly |
| how to/should I/architecture | `/ask` or `/brainstorm` |
| design/UI/UX/layout/style | `/design:good` or `/fix:ui` |
| commit/push/git | `/git:cm`, `/git:cp`, or `/git:pr` |
| recent work/summary | `/watzup` |
| docs/documentation | `/docs:update` or `/docs:init` |
| test/coverage | `/test` or `/fix:test` |

**Rules**: Always suggest agent + explain why + offer workflow + ask permission. Prefer agents over direct implementation (90% of cases).

---

## How Claude Should Understand the Project

1. **Check specs first** (NOT code): `grep -r "feature-name" specifications/01-requirements/`
2. **Then read code** using locations from `TRACEABILITY_MATRIX.md`
3. **Spec = source of truth**. If code mismatches spec → code is wrong

**Reading order for new features**: FR-XXX.md → COMP-XXX.md → API-XXX.md → DB-SCHEMA.md → TC-XXX.md → Code
**Reading order for bug fixes**: TRACEABILITY_MATRIX.md → FR-XXX.md → TC-XXX.md → Code → Fix to match spec

---

## Quick Feature Location Map

### Paper Trading (Execution + Risk Management)
- **Doc**: `specifications/06-features/paper-trading.md`
- **Code**: `rust-core-engine/src/paper_trading/`
  - `engine.rs` Execution simulation (slippage, market impact, partial fills)
  - `engine.rs` Risk management (daily loss, cool-down, correlation)
  - `engine.rs` process_trading_signal() - risk checks
  - `engine.rs` execute_trade() - full execution
  - `engine.rs` close_trade() - consecutive loss tracking
  - `portfolio.rs` Cool-down state fields
  - `trade.rs` Latency tracking fields
  - `settings.rs` All configuration options
- **Tests**: `rust-core-engine/tests/test_paper_trading.rs`

### Authentication & Authorization
- **Doc**: `specifications/06-features/authentication.md`
- **Code**: `rust-core-engine/src/auth/` (jwt.rs, handlers.rs, middleware.rs, database.rs)
- **API**: POST `/api/auth/login`, `/api/auth/register`, `/api/auth/verify`, GET `/api/auth/profile`
- **Tests**: `rust-core-engine/tests/test_auth.rs`
- **Security**: HS256 JWT, bcrypt hashing

### AI & ML Integration
- **Doc**: `specifications/06-features/ai-integration.md`
- **Code**: `python-ai-service/` (models/, main.py, features/)
- **API**: POST `/predict-trend`, `/analyze`
- **Models**: LSTM 68%, GRU 65%, Transformer 70%, Ensemble 72% | Grok/xAI for signal generation
- **Tests**: `python-ai-service/tests/`

### Trading Strategies
- **Doc**: `specifications/06-features/trading-strategies.md`
- **Code**: `rust-core-engine/src/strategies/` (rsi, macd, bollinger, volume, strategy_engine, indicators)
- **API**: GET `/api/strategies/active`, `/api/strategies/signals/:symbol`, POST `/api/strategies/backtest`
- **Performance**: 65% combined win rate, 1.5% avg profit, Sharpe 1.6
- **Tests**: `rust-core-engine/tests/test_strategies.rs`

### WebSocket & Real-Time
- **Doc**: `specifications/06-features/websocket-realtime.md`
- **Code**: `rust-core-engine/src/binance/websocket.rs`, `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`
- **Events**: price_update, signal_generated, trade_executed, portfolio_update, risk_event
- **Tests**: `nextjs-ui-dashboard/src/hooks/useWebSocket.test.tsx`

### Risk Management
- **Doc**: `specifications/06-features/paper-trading.md#risk-management`
- **Code**: `rust-core-engine/src/paper_trading/engine.rs`
- **Features**: Daily Loss Limit (5%), Cool-Down (60min/5losses), Correlation Limits (70%), Consecutive Loss Tracking

### Frontend Dashboard
- **Doc**: `nextjs-ui-dashboard/README.md`
- **Code**: `nextjs-ui-dashboard/src/` (pages/, components/ 71 total, hooks/, contexts/, lib/)
- **Stack**: Shadcn/UI + TailwindCSS, React 18, Vite
- **Tests**: `nextjs-ui-dashboard/src/**/*.test.tsx` (601 tests)

### Database Schema
- **Doc**: `specifications/02-design/2.2-database/DB-SCHEMA.md`
- **Collections** (17): users, paper_portfolios, paper_trades, strategies, market_data, signals, etc.
- **Indexes**: See `DB-INDEXES.md` (37 indexes)

### MCP Server (Model Context Protocol)
- **Spec**: `specifications/01-requirements/1.1-functional-requirements/FR-MCP.md`
- **Code**: `mcp-server/`
  - `src/index.ts` Express + MCP SDK, Streamable HTTP
  - `src/tools/` 114 tools: health(3), market(8), trading(4), paper-trading(39), real-trading(14), ai(12), tasks(7), monitoring(4), settings(10), auth(4), tuning(8), notification(1)
  - `src/client.ts` HTTP client for backends
- **Protocol**: MCP v2024-11-05, port 8090
- **SDK**: `@modelcontextprotocol/sdk ^1.12.1`
- **Tests**: `mcp-server/tests/` (89 tests)
- **Health**: `curl http://localhost:8090/health`

### OpenClaw Gateway (AI via Telegram)
- **Code**: `openclaw/`
  - `config/openclaw.json` Dev config
  - `config/openclaw.production.json` Production with Telegram
  - `scripts/entrypoint.sh` Waits for MCP, registers cron, starts gateway
  - `scripts/botcore-bridge.mjs` MCP client CLI
- **Gateway**: WebSocket port 18789, LAN binding with auth token
- **Config**: `agents.defaults.model.primary`, `gateway.bind`, `channels.telegram.*`
- **Production env**: `TELEGRAM_BOT_TOKEN`, `TELEGRAM_USER_ID`, `OPENCLAW_GATEWAY_TOKEN`

### Self-Tuning Engine
- **Code**: `mcp-server/src/tools/tuning.ts`, `mcp-server/src/tuning/` (bounds.ts, audit.ts, snapshot.ts, types.ts)
- **GREEN tier** (auto-adjust, 22 params): rsi_oversold/overbought, signal_interval_minutes, confidence_threshold, data_resolution, stop_loss_percent, take_profit_percent, min_required_indicators/timeframes, atr_stop/tp_multiplier, sp_* signal pipeline params (13 params), funding_spike_threshold, atr_spike_multiplier, consecutive_loss_reduction_pct
- **YELLOW tier** (confirm token, 13 params): position_size_percent, max_positions, leverage, base_risk_pct, kelly_fraction, sp_bb/stoch/weight params (9 params)
- **RED tier** (explicit approval text, 5 params): max_daily_loss_percent, weekly_drawdown_limit_pct, atr_stop_enabled, kelly_enabled, engine_running
- **MCP tools** (8): `get_tuning_dashboard`, `get_parameter_bounds`, `apply_green_adjustment`, `request_yellow_adjustment`, `request_red_adjustment`, `get_adjustment_history`, `rollback_adjustment`, `take_parameter_snapshot`

---

## Documentation Structure

### `/specifications/` - Unified Documentation (specs + operational docs)
- `01-requirements/` 24 docs (194 requirements, 63 user stories)
- `02-design/` 20 docs (Architecture, API, DB schema)
- `03-testing/` 12 docs (186 test cases, 45 scenarios) — includes TESTING_GUIDE
- `04-deployment/` 7 docs — includes PRODUCTION_DEPLOYMENT_GUIDE
- `05-operations/` docs (ops manual, troubleshooting, DR plan, guides, CONTRIBUTING)
- `06-features/` 9 feature guides (paper-trading, auth, ai, strategies, websocket, mcp-server, openclaw, signal-reversal, ai-auto-reversal)
- Root: `TRACEABILITY_MATRIX.md`, `TASK_TRACKER.md`, `README.md`

---

## Development Workflow

### Quick Start
```bash
cp .env.example .env && ./scripts/generate-secrets.sh
./scripts/bot.sh start --memory-optimized   # Start all
./scripts/bot.sh dev                         # Dev with hot reload
./scripts/bot.sh status                      # Check status
```

### Build & Test
```bash
make build          # Build all (or: make build-fast for sequential)
make test           # All 2,202+ tests: Rust(1,336) + Python(409) + Frontend(601)
make lint           # Zero errors required
```

### Service-Specific
```bash
# Rust
cd rust-core-engine && cargo fmt --check && cargo clippy -- -D warnings && cargo test
# Python
cd python-ai-service && black . && flake8 . && pytest --cov
# Frontend
cd nextjs-ui-dashboard && npm run lint && npm run type-check && npm test
```

---

## Tech Stack & Quality

**Stack**: Rust 1.86+ (Actix-web, MongoDB) | Python 3.11+ (FastAPI, TensorFlow, PyTorch, Grok/xAI) | TypeScript/React 18/Vite/Shadcn/TailwindCSS | MCP Server (Node 18, Express, SDK ^1.12.1) | OpenClaw (Node 22, Telegram, Claude Sonnet 4.5) | MongoDB replica sets | WebSocket

**Quality**: 94/100 overall (A) | 98/100 security (A+) | 90.4% coverage | 84% mutation score | 2,202+ tests | PERFECT 10/10 code quality

**Services**: Frontend :3000 | Rust API :8080 | Python AI :8000 | MCP Server :8090 | OpenClaw :18789

---

## Security & Safety

- **NEVER** commit API keys, passwords, JWT secrets, tokens. Use `.env`
- **Rust**: Zero unwrap()/expect() in production, use `?` operator
- **Trading**: Testnet by default (`BINANCE_TESTNET=true`, `TRADING_ENABLED=false`). NEVER enable production trading without explicit request
- **Before commit**: `make lint && make test && make security-check`

---

## Getting Help

- **Logs**: `./scripts/bot.sh logs --service <name>`
- **Health**: `curl localhost:8080/api/health` | `curl localhost:8090/health`
- **Troubleshooting**: `specifications/05-operations/5.2-troubleshooting/`
- **Deploy**: `specifications/04-deployment/`
- **Common issues**: OOM → `--memory-optimized`, Port conflict → `lsof -i :PORT`, Build fail → `make build-fast`
