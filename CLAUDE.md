# CLAUDE.md - Smart Navigation Hub

**Purpose**: This file helps Claude Code quickly find exact code locations and documentation for any feature without reading entire codebase.

---

## ⚠️ CRITICAL: SPEC-DRIVEN DEVELOPMENT WORKFLOW

**THIS IS A FINANCE PROJECT - MISTAKES = MONEY LOSS**

### Mandatory Workflow for ALL New Features

```
USER REQUEST → CREATE SPEC → USE AGENTS → UPDATE SPECS → DONE
```

#### Step 1: CREATE SPEC FIRST (Before ANY code)

When user requests a new feature:

1. **Search existing specs** to understand current system:
   ```bash
   # Read relevant existing specs first
   specs/01-requirements/1.1-functional-requirements/FR-*.md
   specs/02-design/2.3-api/API-*.md
   specs/02-design/2.5-components/COMP-*.md
   ```

2. **Create new requirement spec** using template:
   - Location: `specs/01-requirements/1.1-functional-requirements/FR-XXX.md`
   - Use: `specs/_SPEC_TEMPLATE.md` as base
   - Include: Acceptance criteria (☐ checkboxes), examples, edge cases
   - Reference: Related specs, design docs, dependencies

3. **Add to TRACEABILITY_MATRIX.md**:
   - Add new FR-XXX entry to appropriate module section
   - Link to design docs, test cases, code locations
   - Maintain 100% traceability

4. **Create test cases** (TC-XXX.md):
   - Location: `specs/03-testing/3.2-test-cases/TC-XXX.md`
   - Format: Gherkin (Given/When/Then)
   - Coverage: Happy path + edge cases + errors

**DO NOT write ANY code until specs are complete and reviewed!**

#### Step 2: USE AGENTS FOR IMPLEMENTATION (MANDATORY)

**NEVER code manually for complex features. ALWAYS use specialized agents:**

```bash
# For planning & architecture
/plan [feature-description]  # Creates implementation plan

# For actual implementation
/cook [step-by-step-tasks]   # Implements feature following spec

# For testing
/test                        # Runs all tests, verifies coverage

# For code review
# Automatic after significant code changes

# For documentation
/docs:update                 # Updates docs after implementation
```

**Agent Selection Guide:**

| Feature Type | Use This Agent | Example |
|--------------|----------------|---------|
| **New trading strategy** | `/plan` → `/cook` | "Add MACD strategy" |
| **API endpoint** | `/plan` → `/cook` | "Add /api/positions endpoint" |
| **Database change** | `/plan` → `/cook` | "Add trailing_stops collection" |
| **Frontend feature** | `/design:good` → `/cook` | "Add dark mode toggle" |
| **Bug fix** | `/fix:fast` or `/fix:hard` | "Fix order execution bug" |
| **Performance issue** | `/debug` | "Optimize slow queries" |

**Why agents are MANDATORY:**
- ✅ Follow specs precisely
- ✅ Add proper @spec tags automatically
- ✅ Run tests and validation
- ✅ Update documentation
- ✅ Maintain code quality standards
- ✅ Prevent finance-critical mistakes

#### Step 3: ADD @spec TAGS TO CODE (REQUIRED)

After implementation, verify ALL code has @spec tags:

**Python:**
```python
# @spec:FR-ASYNC-001 - Async ML Model Training
# @ref:specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md
# @test:TC-ASYNC-001, TC-ASYNC-002, TC-ASYNC-003
async def train_model(...):
    ...
```

**Rust:**
```rust
// @spec:FR-RISK-007 - Trailing Stop Loss (Long Positions)
// @ref:specs/01-requirements/1.1-functional-requirements/FR-RISK.md
// @test:TC-TRADING-054, TC-TRADING-055, TC-TRADING-056
pub fn update_trailing_stop(...) -> Result<...> {
    ...
}
```

**TypeScript:**
```typescript
// @spec:FR-DASHBOARD-006 - WebSocket Integration
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-INTEGRATION-040
export function useWebSocket() {
  ...
}
```

#### Step 4: UPDATE SPECS & DOCS (Mark Complete)

After implementation is tested and working:

1. **Update requirement spec**:
   - Change checkboxes: `- [ ]` → `- [x]`
   - Add implementation notes
   - Update code locations

2. **Update TRACEABILITY_MATRIX.md**:
   - Verify FR-XXX entry has correct code location
   - Update status: "Pending" → "✅ Implemented"
   - Add actual test case IDs

3. **Update feature documentation**:
   - Update `docs/features/*.md` if needed
   - Add to `CHANGELOG.md`
   - Update `README.md` if user-facing

4. **Run validation**:
   ```bash
   python3 scripts/validate-specs.py
   # Must show 0 errors for new requirement
   ```

#### Step 5: VERIFY TRACEABILITY (100% Required)

Before marking feature complete, verify:

```bash
# Check: Requirement exists
grep -r "FR-XXX-YYY" specs/01-requirements/

# Check: In traceability matrix
grep "FR-XXX-YYY" specs/TRACEABILITY_MATRIX.md

# Check: Has @spec tag in code
grep -r "@spec:FR-XXX-YYY" {rust-core-engine,python-ai-service,nextjs-ui-dashboard}/

# Check: Test cases exist
grep "TC-XXX-YYY" specs/03-testing/3.2-test-cases/

# Run validation
python3 scripts/validate-specs.py
```

**All checks must pass ✅**

---

## 📖 HOW CLAUDE SHOULD UNDERSTAND THE PROJECT

### ALWAYS Read Specs First (Before ANY task)

**When user asks about existing feature:**

1. **Check specs first** (NOT code):
   ```bash
   # Search requirements
   grep -r "feature-name" specs/01-requirements/

   # Read full spec
   cat specs/01-requirements/1.1-functional-requirements/FR-XXX.md

   # Check traceability
   grep "FR-XXX" specs/TRACEABILITY_MATRIX.md
   ```

2. **Then read code** (to verify implementation):
   - Use code location from TRACEABILITY_MATRIX.md
   - Verify @spec tags match
   - Check implementation matches spec

3. **Answer based on SPEC** (not just code):
   - Spec = source of truth
   - Code should match spec
   - If mismatch → code is wrong, not spec

**When user asks to modify existing feature:**

1. **Read current spec** to understand requirements
2. **Plan changes** (what needs updating in spec)
3. **Update spec FIRST** (before code)
4. **Then use agents** to update code
5. **Verify** code matches updated spec

**When explaining how system works:**

1. **Reference specs** for authoritative info
2. **Use code locations** from TRACEABILITY_MATRIX.md
3. **Cite specific FR-XXX** requirement IDs
4. **Show examples** from spec, not just code

### Spec File Reading Order

**For new features:**
```
1. FR-XXX.md (requirement) → understand WHAT to build
2. COMP-XXX.md (component) → understand HOW it's architected
3. API-XXX.md (API design) → understand interface
4. DB-SCHEMA.md (database) → understand data model
5. TC-XXX.md (test cases) → understand validation
6. Code (implementation) → see actual code
```

**For bug fixes:**
```
1. TRACEABILITY_MATRIX.md → find which FR-XXX covers this
2. FR-XXX.md → understand intended behavior
3. TC-XXX.md → understand test cases
4. Code → identify bug
5. Fix code to match spec (not vice versa)
```

**For questions:**
```
1. Search specs: grep -r "keyword" specs/
2. Read matched FR-XXX.md files
3. Check TRACEABILITY_MATRIX.md for code location
4. Read code to verify
5. Answer based on spec + code verification
```

---

## 🎯 VALIDATION RULES (Must Follow)

### Before Starting ANY Task

```bash
# 1. Validate current state
python3 scripts/validate-specs.py

# 2. Read relevant specs
cat specs/01-requirements/1.1-functional-requirements/FR-*.md

# 3. Check traceability
cat specs/TRACEABILITY_MATRIX.md | grep "FR-XXX"
```

### After Completing ANY Task

```bash
# 1. Verify @spec tags added
grep -r "@spec:FR-XXX" .

# 2. Update traceability matrix
# Edit: specs/TRACEABILITY_MATRIX.md

# 3. Run validation
python3 scripts/validate-specs.py
# Must show 0 new errors

# 4. Verify 100% traceability maintained
# Check: All new FRs in matrix
# Check: All code has @spec tags
```

### Continuous Validation

- ✅ Specs must be written BEFORE code
- ✅ All code must have @spec tags
- ✅ TRACEABILITY_MATRIX.md must be 100% complete
- ✅ Validation script must pass (0 critical errors)
- ✅ Finance-critical features must have A+ safety grade

**If any validation fails → DO NOT PROCEED until fixed**

---

---

## 🎯 QUICK FEATURE LOCATION MAP

### Paper Trading (Execution + Risk Management)
📄 **Doc**: `docs/features/paper-trading.md` (comprehensive guide)
📂 **Code**: `rust-core-engine/src/paper_trading/`
- **engine.rs**
  - `738-845`: Execution simulation (slippage, market impact, partial fills)
  - `847-1039`: Risk management (daily loss limit, cool-down, correlation)
  - `509-560`: process_trading_signal() - Risk checks before execution
  - `1041-1197`: execute_trade() - Full execution with all simulations
  - `1425-1452`: close_trade() - Consecutive loss tracking
- **portfolio.rs**
  - `77-81`: Cool-down state fields (consecutive_losses, cool_down_until)
  - `223-224`: Field initialization
- **trade.rs**
  - `145-152`: Latency tracking fields (signal_timestamp, execution_latency_ms)
  - `223-225`: Field initialization
- **settings.rs**: All configuration options

🧪 **Tests**: `rust-core-engine/tests/test_paper_trading.rs`
📊 **Quality**: 98% realism, 94.5/100 overall (A+)

**Common Tasks**:
- Enable slippage: Set `execution.simulate_slippage = true`
- Check daily loss: See `engine.rs:847 check_daily_loss_limit()`
- Monitor execution: `docker logs -f | grep "💸|⏳|📊|⚡"`

---

### Authentication & Authorization
📄 **Doc**: `docs/features/authentication.md`
📂 **Code**: `rust-core-engine/src/auth/`
- **jwt.rs**: JWT generation, validation, refresh
- **handlers.rs**: Login, logout, register endpoints
- **middleware.rs**: Auth middleware, token extraction
- **database.rs**: User database operations

🔑 **API**:
- `POST /api/auth/login` - Login with email/password
- `POST /api/auth/register` - Create new user
- `POST /api/auth/refresh` - Refresh expired token
- `GET /api/auth/me` - Get current user (protected)

🧪 **Tests**: `rust-core-engine/tests/test_auth.rs`
🔒 **Security**: 98/100 (A+), RS256 JWT, bcrypt hashing

**Common Tasks**:
- Generate keys: `openssl genrsa -out private_key.pem 2048`
- Test login: `curl -X POST /api/auth/login -d '{"email":...}'`

---

### AI & ML Integration
📄 **Doc**: `docs/features/ai-integration.md`
📂 **Code**: `python-ai-service/`
- **models/**: LSTM, GRU, Transformer implementations
- **main.py**:
  - `150-250`: GPT-4 analysis endpoint
  - `predict_price()`: ML model predictions
  - `analyze_market_sentiment()`: Sentiment analysis
- **features/**: Technical indicators, feature engineering

🤖 **Models**: LSTM (68%), GRU (65%), Transformer (70%), Ensemble (72%)

🔌 **API**:
- `POST /predict` - Price prediction
- `POST /analyze` - GPT-4 market analysis
- `POST /sentiment` - Sentiment analysis
- `POST /train` - Retrain models

🧪 **Tests**: `python-ai-service/tests/`
🎯 **Accuracy**: 70% average directional accuracy

**Common Tasks**:
- Get prediction: `curl -X POST /predict -d '{"symbol":"BTCUSDT"}'`
- Check OpenAI key: `echo $OPENAI_API_KEY`

---

### Trading Strategies
📄 **Doc**: `docs/features/trading-strategies.md`
📂 **Code**: `rust-core-engine/src/strategies/`
- **rsi_strategy.rs**: RSI strategy (62% win rate)
- **macd_strategy.rs**: MACD strategy (58% win rate)
- **bollinger_strategy.rs**: Bollinger Bands (60% win rate)
- **volume_strategy.rs**: Volume-based trading (52% win rate)
- **strategy_engine.rs**: Strategy orchestration
- **indicators.rs**: Technical calculations (RSI, MACD, EMA, etc.)

📊 **Performance**: 65% combined win rate, 1.5% avg profit, Sharpe 1.6

🔌 **API**:
- `GET /api/strategies/active` - List active strategies
- `GET /api/strategies/signals/:symbol` - Get signals
- `POST /api/strategies/backtest` - Run backtest

🧪 **Tests**: `rust-core-engine/tests/test_strategies.rs`

**Common Tasks**:
- Enable strategy: Set `strategies.rsi_enabled = true` in config.toml
- Backtest: `curl -X POST /api/strategies/backtest -d '{"strategy":"rsi"}'`

---

### WebSocket & Real-Time Communication
📄 **Doc**: `docs/features/websocket-realtime.md`
📂 **Code**:
- **Backend**: `rust-core-engine/src/binance/websocket.rs`, `src/websocket/`
- **Frontend**: `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

📡 **Endpoints**:
- Input: `wss://stream.binance.com:9443/ws` (Binance)
- Output: `ws://localhost:8080/ws` (Frontend)

📨 **Events**: price_update, signal_generated, trade_executed, portfolio_update, risk_event

⚡ **Latency**: <100ms end-to-end, 99.9% uptime

🧪 **Tests**: `nextjs-ui-dashboard/src/hooks/useWebSocket.test.tsx`

**Common Tasks**:
- Connect: `const { socket, connected } = useWebSocket();`
- Broadcast: `broadcaster.send(WebSocketEvent {...})`
- Monitor: `docker logs -f | grep "WebSocket"`

---

### Risk Management
📄 **Doc**: `docs/features/paper-trading.md#risk-management` (part of paper trading)
📂 **Code**: `rust-core-engine/src/paper_trading/engine.rs:847-1039`

🛡️ **Features**:
1. Daily Loss Limit (5% max)
2. Cool-Down Mechanism (60 min after 5 losses)
3. Position Correlation Limits (70% max directional)
4. Consecutive Loss Tracking (auto-reset on profit)

**Common Tasks**:
- Check daily loss: `check_daily_loss_limit()` at line 847
- Check cool-down: `is_in_cooldown()` at line 892
- Update settings: Modify `risk` section in config.toml

---

### Frontend Dashboard
📄 **Doc**: `nextjs-ui-dashboard/README.md`
📂 **Code**: `nextjs-ui-dashboard/src/`
- **pages/**: Route components
- **components/**: Reusable UI components (71 total)
- **hooks/**: Custom React hooks (useWebSocket, useAIAnalysis, usePaperTrading)
- **contexts/**: Global state (AuthContext, WebSocketContext)
- **lib/**: Utilities, API client

🎨 **UI Library**: Shadcn/UI + TailwindCSS
📦 **Bundle**: 400KB (optimized with code splitting)

🧪 **Tests**: `nextjs-ui-dashboard/src/**/*.test.tsx` (601 tests)

**Common Tasks**:
- Start dev: `cd nextjs-ui-dashboard && npm run dev`
- Build: `npm run build`
- Test: `npm run test`

---

### Database Schema
📄 **Doc**: `specs/02-design/2.2-database/DB-SCHEMA.md`
📂 **Code**: Database operations in each service

📊 **Collections** (17 total):
- `users` - User accounts
- `paper_portfolios` - Paper trading portfolios
- `paper_trades` - Executed paper trades
- `strategies` - Strategy configurations
- `market_data` - Historical candles
- `signals` - AI trading signals
- See full schema in DB-SCHEMA.md

**Common Tasks**:
- Connect: MongoDB running on localhost:27017
- View data: Use MongoDB Compass or mongosh
- Indexes: See `DB-INDEXES.md` for 37 indexes

---

## 📚 DOCUMENTATION STRUCTURE

**Two main directories** - Clean and organized:

### 1️⃣ `/docs/` - Operational Documentation (for users & developers)
- **`features/`** - Feature-specific guides (5 docs, <500 lines each)
  - `paper-trading.md` - Paper trading system
  - `authentication.md` - Auth & JWT
  - `ai-integration.md` - ML models & GPT-4
  - `trading-strategies.md` - RSI, MACD, Bollinger, Volume
  - `websocket-realtime.md` - Real-time communication
- **`guides/`** - User guides & how-to documents
- **`reports/`** - Implementation reports, phase summaries
- **`plans/`** - Planning documents, validation guides
- **`testing/`** - Testing documentation
- **`certificates/`** - Quality certificates & achievements
- **`archive/`** - Legacy documentation (old `/documents` content)
- **Root docs**: `CONTRIBUTING.md`, `TESTING_GUIDE.md`, `TROUBLESHOOTING.md`, `PRODUCTION_DEPLOYMENT_GUIDE.md`

### 2️⃣ `/specs/` - Technical Specifications (for spec-driven development)
- **`01-requirements/`** - 24 docs (194 requirements, 63 user stories)
- **`02-design/`** - 20 docs (Architecture, API, DB schema)
- **`03-testing/`** - 12 docs (186 test cases, 45 scenarios)
- **`04-deployment/`** - 7 docs (Infrastructure, CI/CD)
- **`05-operations/`** - 3 docs (Operations, DR plan)
- **Root specs**: `TRACEABILITY_MATRIX.md`, `TASK_TRACKER.md`, `README.md`

**Why 2 directories?**
- `/docs` = Operational docs for **daily use** (guides, troubleshooting, reports)
- `/specs` = Formal specifications for **development** (requirements, design, traceability)

---

## 🚀 DEVELOPMENT WORKFLOW

### Quick Start Commands
```bash
# Setup
cp .env.example .env && ./scripts/generate-secrets.sh

# Start all services (memory-optimized)
./scripts/bot.sh start --memory-optimized

# Development mode with hot reload
./scripts/bot.sh dev

# Status & logs
./scripts/bot.sh status
./scripts/bot.sh logs --service rust-core-engine
```

### Build & Test
```bash
# Build all
make build              # Or: make build-fast (sequential, memory-safe)

# Test all (2,202+ tests)
make test               # Rust (1,336) + Python (409) + Frontend (601)

# Quality checks
make lint               # Zero errors required
make quality-metrics    # Current: 94/100 (Grade A)
```

### Service-Specific
```bash
# Rust
cd rust-core-engine
cargo fmt --check && cargo clippy -- -D warnings && cargo test

# Python
cd python-ai-service
black . && flake8 . && pytest --cov

# Frontend
cd nextjs-ui-dashboard
npm run lint && npm run type-check && npm test
```

---

## 🎯 PROJECT STATUS

### Quality Metrics
- **Overall**: 94/100 (Grade A)
- **Security**: 98/100 (A+)
- **Test Coverage**: 90.4% average (Rust 90%, Python 95%, Frontend 90%+)
- **Mutation Score**: 84% average (Rust 78%, Python 76%, Frontend 75%)
- **Code Quality**: PERFECT 10/10
- **Documentation**: 96/100 (A+)
- **Performance**: 95/100 (A+)

### Production Readiness
- ✅ Zero HIGH/CRITICAL vulnerabilities
- ✅ All tests passing (2,202+ tests)
- ✅ Zero ESLint errors
- ✅ Zero compiler warnings
- ✅ Comprehensive documentation (15,000+ lines)
- ✅ Production deployment guide ready

### Tech Stack
- **Backend**: Rust 1.86+ (Actix-web, MongoDB)
- **AI/ML**: Python 3.11+ (FastAPI, TensorFlow, PyTorch, OpenAI GPT-4)
- **Frontend**: TypeScript, React 18, Vite, Shadcn/UI, TailwindCSS
- **Database**: MongoDB with replica sets
- **Real-Time**: WebSocket (Binance + Frontend)

---

## ⚡ COMMON QUESTIONS (Quick Answers)

### "Where is paper trading execution simulation?"
→ `rust-core-engine/src/paper_trading/engine.rs:1041-1197`
→ Read `docs/features/paper-trading.md` for details

### "How do I enable slippage?"
→ Set `execution.simulate_slippage = true` in settings
→ Or: `curl -X POST /api/paper-trading/settings -d '{"execution":{"simulate_slippage":true}}'`

### "Where are JWT tokens generated?"
→ `rust-core-engine/src/auth/jwt.rs`
→ Read `docs/features/authentication.md` for API usage

### "How do I get AI price predictions?"
→ `POST http://localhost:8000/predict` with `{"symbol":"BTCUSDT"}`
→ Read `docs/features/ai-integration.md` for all endpoints

### "Where are trading strategies defined?"
→ `rust-core-engine/src/strategies/` (4 strategies)
→ Read `docs/features/trading-strategies.md` for performance metrics

### "How do I monitor WebSocket connections?"
→ `docker logs -f rust-core-engine-dev | grep "WebSocket"`
→ Read `docs/features/websocket-realtime.md` for event types

### "Where is the database schema?"
→ `specs/02-design/2.2-database/DB-SCHEMA.md` (17 collections)

### "How do I deploy to production?"
→ Read `docs/PRODUCTION_DEPLOYMENT_GUIDE.md` (1,300+ lines, comprehensive)

### "Where are test files?"
→ Rust: `rust-core-engine/tests/`
→ Python: `python-ai-service/tests/`
→ Frontend: `nextjs-ui-dashboard/src/**/*.test.tsx`

### "How do I troubleshoot issues?"
→ Read `docs/TROUBLESHOOTING.md` for common issues
→ Or: Check feature-specific docs in `docs/features/`

---

## 🔒 SECURITY & BEST PRACTICES

### Secrets Management
- **NEVER** commit: API keys, passwords, JWT secrets, tokens
- **ALWAYS** use `.env`: `cp .env.example .env`
- **Generate** secure secrets: `./scripts/generate-secrets.sh`
- **Validate**: `make validate-secrets`

### Code Quality Standards
- **Rust**: Zero unwrap()/expect() in production, use `?` operator
- **Python**: Black formatted, 98%+ type hints, Flake8 compliant
- **TypeScript**: ESLint clean, strict mode, 0 errors/warnings

### Before Committing
```bash
make lint               # Must pass (zero errors)
make test               # All tests must pass
make quality-metrics    # Must maintain ≥94/100
make security-check     # Zero HIGH/CRITICAL vulns
```

### Trading Safety (CRITICAL!)
- ✅ Testnet by default: `BINANCE_TESTNET=true`
- ✅ Trading disabled: `TRADING_ENABLED=false`
- ⚠️ **NEVER** enable production trading without explicit user request
- ⚠️ **ALWAYS** test with testnet first

---

## 📋 SPEC-DRIVEN DEVELOPMENT

This project follows **spec-driven development**. All features must conform to specifications BEFORE implementation.

### Specification System (100% Complete)
- **Location**: `specs/` directory (75 documents, 2.6MB)
- **Traceability**: `specs/TRACEABILITY_MATRIX.md` (100% bidirectional)
- **Code Tagging**: 47 @spec tags across 30 files

### Code Tagging Convention
```rust
// @doc:docs/features/paper-trading.md#execution-simulation
// @spec:FR-PAPER-001
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md
async fn execute_trade(...) { ... }
```

### Development Workflow
1. **Read spec first**: Check `specs/01-requirements/`, `specs/02-design/`
2. **Verify code tagging**: Look for @spec tags
3. **Update spec if needed**: BEFORE coding
4. **Add @spec tags**: To new code
5. **Test against spec**: Validate requirements

**Key Principles**:
- Spec is source of truth (code matches spec)
- No undocumented features
- Code tagging required
- 100% traceability

---

## 🎖️ ACHIEVEMENTS

**World-Class Status**:
- 🏆 PERFECT 10/10 quality score
- ⭐ 94/100 overall (Grade A)
- 🔒 98/100 security (A+)
- 📊 90.4% coverage, 2,202+ tests
- 🧬 84% mutation score
- 📚 96/100 documentation (A+)
- ⚡ 95/100 performance (A+)
- 🎯 **Top 10% worldwide**

---

## 🆘 GETTING HELP

### Quick Reference
- **Service URLs**: Frontend (3000), Rust API (8080), Python AI (8000)
- **Logs**: `./scripts/bot.sh logs --service <name>`
- **Health**: `curl http://localhost:8080/api/health`

### Documentation
- **Features**: `docs/features/` (5 focused guides)
- **Specs**: `specs/` (75 comprehensive docs)
- **Guides**: `docs/CONTRIBUTING.md`, `docs/TESTING_GUIDE.md`, `docs/TROUBLESHOOTING.md`

### Common Issues
1. **Out of Memory**: Use `./scripts/bot.sh start --memory-optimized`
2. **Port Conflicts**: Check with `lsof -i :3000/8000/8080`
3. **Service Unhealthy**: View logs `./scripts/bot.sh logs --service <name>`
4. **Build Failures**: Use `make build-fast` (sequential)

---

**Last Updated**: 2025-11-22
**Status**: PRODUCTION-READY | WORLD-CLASS QUALITY | SPEC-DRIVEN
**Version**: 3.0 (Spec-Driven Development Workflow + 100% Traceability)

**Major Changes in v3.0**:
- ✅ Added MANDATORY spec-driven development workflow
- ✅ Specs MUST be written BEFORE code
- ✅ All code MUST use agents for implementation
- ✅ All code MUST have @spec tags
- ✅ 100% traceability REQUIRED (256 requirements tracked)
- ✅ Validation script enforces quality standards
- ✅ Claude MUST read specs first (not just code)
