# Bot-Core Specific Instructions for All Agents

This file provides bot-core specific guidance for all ClaudeKit agents working with the bot-core cryptocurrency trading platform.

## Project Context

**Bot-Core** is a **PERFECT 10/10 quality, production-ready** cryptocurrency trading bot with:
- Overall Quality: **94/100 (Grade A)** - Top 10% of software projects
- Security: **98/100 (A+)** - Zero HIGH/CRITICAL vulnerabilities
- Test Coverage: **90.4% average** (2,202+ tests)
- Mutation Testing: **84% average**
- Documentation: **96/100 (A+)** - 15,000+ lines

**Tech Stack:**
- Rust 1.86+ (trading engine, Port 8080)
- Python 3.11+ (AI/ML service, Port 8000)
- TypeScript/React (dashboard, Port 3000)
- MongoDB (database)
- Docker (containerization)

---

## CRITICAL: Spec-Driven Development System

### ⚠️ MUST FOLLOW: Specification-First Approach

**Bot-core follows 100% spec-driven development:**

1. **Specifications = Source of Truth**
   - Code MUST match spec (not vice versa)
   - No undocumented features allowed
   - All features have complete specs BEFORE implementation

2. **Specification Directory Structure**
   ```
   specs/
   ├── README.md                    # Master specification index
   ├── TRACEABILITY_MATRIX.md       # Requirements-to-code mapping
   ├── TASK_TRACKER.md              # 100% completion tracking
   ├── 01-requirements/             # 24 docs (FR, NFR, US, SYS)
   ├── 02-design/                   # 20 docs (Architecture, DB, API, UI)
   ├── 03-testing/                  # 12 docs (Test cases & scenarios)
   ├── 04-deployment/               # 7 docs (Infrastructure, CI/CD)
   └── 05-operations/               # 3 docs (Operations, DR plan)
   ```

3. **Code Tagging Convention (MANDATORY)**
   - All production code MUST have @spec tags
   - Format:
     ```rust
     // @spec:FR-AUTH-001 - JWT Token Generation
     // @ref:specs/02-design/2.5-components/COMP-RUST-AUTH.md
     // @test:TC-AUTH-001, TC-AUTH-002
     ```
   - Validate tags: `python3 scripts/validate-spec-tags.py`

4. **Development Workflow**
   - Read spec FIRST (specs/01-requirements/, specs/02-design/)
   - Verify code tagging exists
   - Update spec BEFORE coding if changes needed
   - Add @spec tags to new code
   - Test against spec requirements
   - Maintain 100% traceability

---

## File Organization Rules (STRICT)

### ✅ ALLOWED in Root (ONLY 2 .md files)
- `README.md` - Project overview
- `CLAUDE.md` - Claude Code instructions (this file's parent)

### ❌ FORBIDDEN in Root
- Any other `.md` files
- Reports, certificates, documentation
- Temporary files

### 📁 Correct Documentation Locations

**All documentation goes to `docs/` directory:**
```
docs/
├── reports/                       # ALL reports
│   ├── QUALITY_METRICS_SUMMARY.md
│   ├── TEST_COVERAGE_REPORT.md
│   └── SECURITY_AUDIT_REPORT.md
├── certificates/                  # Achievements
│   └── PERFECT_10_10_CERTIFICATE.md
├── testing/                       # Testing docs
├── architecture/                  # Architecture docs
├── CONTRIBUTING.md
├── SECURITY_CREDENTIALS.md
├── TESTING_GUIDE.md
└── TROUBLESHOOTING.md
```

**Service-specific documentation:**
- `rust-core-engine/docs/` - Rust-specific
- `python-ai-service/docs/` - Python-specific
- `nextjs-ui-dashboard/docs/` - Frontend-specific

**Rule:** If it's a `.md` file and not `README.md` or `CLAUDE.md`, it belongs in `docs/` or `{service}/docs/`.

---

## Quality Gates (MANDATORY)

### Before Every Commit

**All commits MUST pass:**

1. **Linting (Zero errors/warnings)**
   ```bash
   make lint              # All services
   make lint-rust         # Rust: cargo fmt + clippy
   make lint-python       # Python: black + flake8 + mypy
   make lint-frontend     # TypeScript: ESLint + Prettier
   ```

2. **Testing (All tests pass)**
   ```bash
   make test              # All services (2,202+ tests)
   make test-rust         # Rust: cargo test (1,336 tests)
   make test-python       # Python: pytest (409 tests)
   make test-frontend     # TypeScript: vitest (601 tests)
   ```

3. **Coverage (90%+ required)**
   - Overall: 90.4% average
   - Rust: 90% (target: 90%+) ✅
   - Python: 95% (target: 90%+) ✅
   - Frontend: 90%+ (target: 85%+) ✅

4. **Quality Metrics (94/100 minimum)**
   ```bash
   make quality-metrics   # Comprehensive quality check
   make quality-report    # Generate report
   ```

5. **Security (Zero HIGH/CRITICAL)**
   ```bash
   make security-check    # Comprehensive security scan
   ./scripts/security-scan.sh
   ```

### Quality Standards

**Rust (rust-core-engine/):**
- Zero `unwrap()` or `expect()` in production
- Comprehensive error handling (37+ error types)
- Zero compiler warnings
- Clippy clean with strict settings
- 90%+ coverage, 75%+ mutation score

**Python (python-ai-service/):**
- Black formatted (100%)
- Flake8 compliant (PEP 8)
- Type hints: 98%+ coverage
- 95%+ test coverage
- Zero HIGH/CRITICAL vulnerabilities

**TypeScript/React (nextjs-ui-dashboard/):**
- ESLint: 0 errors, 0 warnings
- TypeScript strict mode enabled
- 90%+ test coverage
- Zero flaky tests
- Bundle < 500KB

---

## Agent-Specific Instructions

### For `planner` Agent

**When creating plans:**
1. **Read specs FIRST:**
   - `specs/01-requirements/` - Functional/non-functional requirements
   - `specs/02-design/` - Architecture, API, database design
   - `specs/TRACEABILITY_MATRIX.md` - Requirements mapping

2. **Save plans to:** `docs/plans/` (NOT `./plans/`)

3. **Plan must include:**
   - Spec references (FR-XXX-YYY)
   - @spec tag strategy
   - Quality gate compliance
   - Test coverage targets
   - File organization adherence

4. **Use `scout` agents to search:**
   - `specs/` - Existing specifications
   - `docs/` - Project documentation
   - Source code for patterns

### For `tester` Agent

**Test execution commands:**
```bash
# Run all tests
make test

# Service-specific
make test-rust         # Rust: cargo test
make test-python       # Python: pytest --cov
make test-frontend     # Frontend: npm run test

# Coverage reports
make test-rust         # Generates HTML coverage
make test-python       # pytest --cov-report=html
make test-frontend     # npm run test:coverage

# Integration tests
make test-integration
```

**Coverage requirements:**
- Rust: ≥90% (lib + tests)
- Python: ≥95% (all modules)
- Frontend: ≥90% (components + hooks)

**Test report location:** `docs/testing/`

### For `code-reviewer` Agent

**Quality check commands:**
```bash
# Lint checks
make lint              # All services
make lint-rust         # Zero warnings required
make lint-python       # Black + flake8 + mypy
make lint-frontend     # ESLint + Prettier

# Quality metrics
make quality-metrics   # Overall quality check
make quality-report    # Generate detailed report

# Security
make security-check    # Comprehensive scan
```

**Review checklist:**
- [ ] All linters pass (zero errors/warnings)
- [ ] All tests pass
- [ ] Coverage maintained (≥90%)
- [ ] @spec tags present in new code
- [ ] File organization rules followed
- [ ] No .md files in root (except README/CLAUDE)
- [ ] Security scan clean
- [ ] Quality score ≥94/100

**Report location:** `docs/reports/CODE_REVIEW_REPORT.md`

### For `docs-manager` Agent

**Documentation structure:**
- **Specifications:** `specs/` (60 docs, 2.6MB, 77,574 lines)
- **General docs:** `docs/`
- **Service docs:** `{service}/docs/`
- **Reports:** `docs/reports/`
- **Testing:** `docs/testing/`

**Update these when code changes:**
- `specs/TRACEABILITY_MATRIX.md` - Requirements mapping
- `specs/TASK_TRACKER.md` - Task completion
- `docs/reports/QUALITY_METRICS_SUMMARY.md` - Quality metrics
- Relevant API specs in `specs/02-design/2.3-api/`

**Never create .md files in root** (except README.md, CLAUDE.md)

### For `git-manager` Agent

**Commit message format (Conventional Commits):**
```bash
# Features (minor bump)
feat: add new authentication system

# Bug fixes (patch bump)
fix: resolve memory leak in user service

# Breaking changes (major bump)
feat!: redesign API endpoints

# Other types (patch bump)
docs: update installation guide
refactor: simplify database queries
test: add integration tests
ci: update GitHub Actions workflow
```

**Git workflow:**
```bash
# Check status
git status
git diff

# Stage changes
git add <files>

# Commit (will auto-lint via husky)
git commit -m "feat: description"

# Push
git push origin <branch>
```

**IMPORTANT:**
- Husky will auto-run commit-msg validation
- Commitlint enforces conventional commits
- Semantic-release auto-generates CHANGELOG.md
- Include co-author: `Co-Authored-By: Claude <noreply@anthropic.com>`

### For `debugger` Agent

**Log locations:**
```bash
# Docker logs
docker logs bot-rust-core
docker logs bot-python-ai
docker logs bot-nextjs-dashboard

# Application logs
./scripts/bot.sh logs
./scripts/bot.sh logs --service rust-core-engine
```

**Debug commands:**
```bash
# Service health
make health
./scripts/bot.sh status

# Run specific tests
make test-rust
make test-python
make test-frontend
```

### For `database-admin` Agent

**Database:** MongoDB (Port 27017)

**Collections (17 total):**
- users, sessions, api_keys
- portfolios, positions, orders
- strategies, signals, backtests
- market_data, klines, tickers
- trades, paper_trades, risk_metrics
- system_logs, audit_logs

**Schema location:** `specs/02-design/2.2-database/DB-SCHEMA.md`

**Access:**
```bash
# Via Docker
docker exec -it bot-mongodb mongosh

# Via connection string (from .env)
mongosh mongodb://localhost:27017/bot_core
```

---

## Trading Safety Rules (CRITICAL)

### ⚠️ NEVER Enable Production Trading Without Explicit User Request

**Default settings:**
```bash
BINANCE_TESTNET=true        # ALWAYS testnet by default
TRADING_ENABLED=false       # MUST be manually enabled
```

**Safety checklist:**
- ✅ Testnet MUST be default
- ✅ Trading MUST be disabled by default
- ⚠️ NEVER enable production without user request
- ⚠️ ALWAYS test with testnet first
- ⚠️ VERIFY all strategies in paper trading mode

---

## Common Commands Reference

### Development
```bash
# Start services
./scripts/bot.sh start --memory-optimized
./scripts/bot.sh dev

# Stop/restart
./scripts/bot.sh stop
./scripts/bot.sh restart

# Status
./scripts/bot.sh status
./scripts/bot.sh logs
```

### Testing
```bash
make test                    # All tests (2,202+)
make test-integration        # Integration tests
make test-rust               # Rust only
make test-python             # Python only
make test-frontend           # Frontend only
```

### Quality
```bash
make lint                    # All linters
make quality-metrics         # Quality check
make quality-report          # Generate report
make security-check          # Security scan
```

### Building
```bash
make build                   # All services
make build-rust              # Rust only
make build-python            # Python only
make build-frontend          # Frontend only
make build-fast              # Sequential (memory-optimized)
```

---

## Key Principles

1. **Spec-Driven Development**
   - Spec BEFORE code
   - Code MUST match spec
   - 100% traceability required
   - All code tagged with @spec

2. **Quality First**
   - Zero lint errors/warnings
   - All tests passing
   - 90%+ coverage maintained
   - Quality score ≥94/100

3. **File Organization**
   - No .md in root (except README/CLAUDE)
   - All docs in docs/
   - All specs in specs/
   - Service docs in {service}/docs/

4. **Security & Safety**
   - No hardcoded secrets
   - Zero HIGH/CRITICAL vulnerabilities
   - Testnet by default
   - Trading disabled by default

5. **Production Readiness**
   - Perfect 10/10 quality maintained
   - All quality gates pass
   - Comprehensive testing
   - Complete documentation

---

## For More Information

**Read these files:**
- `CLAUDE.md` - Complete project overview and guidelines
- `specs/README.md` - Master specification index
- `docs/CONTRIBUTING.md` - Contribution guide
- `docs/TESTING_GUIDE.md` - Testing documentation
- `docs/TROUBLESHOOTING.md` - Common issues

**Key Reports:**
- `docs/reports/QUALITY_METRICS_SUMMARY.md` - Quality dashboard
- `docs/reports/PERFECT_10_10_VALIDATION_REPORT.md` - Perfect score validation
- `docs/reports/TEST_COVERAGE_REPORT.md` - Coverage details
- `specs/TRACEABILITY_MATRIX.md` - Complete traceability

---

**Last Updated:** 2025-11-14
**Status:** PRODUCTION-READY | WORLD-CLASS QUALITY | PERFECT 10/10
