# Contributing to Bot-Core

Thank you for your interest in contributing to **Bot-Core**, a world-class cryptocurrency trading platform with **Perfect 10/10 quality score** and **94/100 overall metrics (Grade A)**!

This guide will help you get started with contributing to the project, whether you're fixing bugs, adding features, improving documentation, or enhancing tests.

---

## Table of Contents

1. [Getting Started](#getting-started)
2. [Development Environment Setup](#development-environment-setup)
3. [Project Architecture](#project-architecture)
4. [Coding Standards](#coding-standards)
5. [Spec-Driven Development](#spec-driven-development)
6. [Testing Requirements](#testing-requirements)
7. [Commit Message Format](#commit-message-format)
8. [Pull Request Process](#pull-request-process)
9. [Code Review Guidelines](#code-review-guidelines)
10. [ClaudeKit Agent Usage](#claudekit-agent-usage)
11. [Quality Gates](#quality-gates)
12. [Getting Help](#getting-help)

---

## Getting Started

### Prerequisites

Before contributing, ensure you have:

- **Rust** 1.86+ (for Rust Core Engine)
- **Python** 3.11+ (for Python AI Service)
- **Node.js** 18+ or **Bun** latest (for Next.js Dashboard)
- **Docker** & **Docker Compose** (for running full stack)
- **Git** (version control)
- **MongoDB** (local or Docker)

### Fork and Clone

1. **Fork the repository** on GitHub
2. **Clone your fork** locally:
   ```bash
   git clone https://github.com/YOUR_USERNAME/bot-core.git
   cd bot-core
   ```

3. **Add upstream remote**:
   ```bash
   git remote add upstream https://github.com/magic-ai-trading-bot/bot-core.git
   ```

4. **Keep your fork in sync**:
   ```bash
   git fetch upstream
   git checkout main
   git merge upstream/main
   ```

---

## Development Environment Setup

### 1. Environment Configuration

```bash
# Copy environment template
cp config.env .env

# Generate secure secrets
./scripts/generate-secrets.sh

# Edit .env with your API keys
nano .env
```

**Required Environment Variables:**
- `DATABASE_URL` - MongoDB connection string
- `BINANCE_API_KEY` - Binance API key (use testnet for development)
- `BINANCE_API_SECRET` - Binance API secret
- `BINANCE_TESTNET=true` - **ALWAYS true for development**
- `TRADING_ENABLED=false` - **ALWAYS false for development**
- `JWT_SECRET` - JWT signing secret (auto-generated)
- `INTER_SERVICE_TOKEN` - Inter-service authentication token

### 2. Start Development Environment

```bash
# Start all services in development mode with hot reload
./scripts/bot.sh dev

# Or start in memory-optimized mode
./scripts/bot.sh start --memory-optimized

# Check service status
./scripts/bot.sh status

# View logs
./scripts/bot.sh logs --service rust-core-engine
./scripts/bot.sh logs --service python-ai-service
./scripts/bot.sh logs --service nextjs-ui-dashboard
```

### 3. Service-Specific Development

**Rust Core Engine (Port 8080):**
```bash
cd rust-core-engine

# Run locally (without Docker)
cargo run -- --config config.toml

# Run tests
cargo test

# Run tests with coverage
cargo tarpaulin --out Html

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

**Python AI Service (Port 8000):**
```bash
cd python-ai-service

# Create virtual environment
python3 -m venv venv
source venv/bin/activate  # On Windows: venv\Scripts\activate

# Install dependencies
pip install -r requirements.txt
pip install -r requirements.dev.txt

# Run locally
python main.py

# Run tests
pytest

# Run tests with coverage
pytest --cov --cov-report=html

# Format code
black .

# Lint code
flake8 .
```

**Next.js Dashboard (Port 3000):**
```bash
cd nextjs-ui-dashboard

# Install dependencies
npm install
# or with Bun
bun install

# Run development server
npm run dev
# or with Bun
bun run dev

# Run tests
npm run test

# Run tests with coverage
npm run test:coverage

# Run E2E tests
npm run test:e2e

# Lint code
npm run lint

# Type check
npm run type-check

# Build
npm run build
```

---

## Project Architecture

### Service Overview

```
┌─────────────────────────────────────────────────────────────┐
│                   Bot-Core Architecture                     │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  Next.js Dashboard (3000)                                   │
│         │                                                   │
│         │ HTTP/WebSocket                                    │
│         ▼                                                   │
│  Rust Core Engine (8080)  ◄────────► Python AI (8000)      │
│         │                                                   │
│         │                                                   │
│         ▼                                                   │
│    MongoDB (27017)                                          │
│         │                                                   │
│         ▼                                                   │
│  Binance WebSocket                                          │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### Directory Structure

```
bot-core/
├── rust-core-engine/      # Trading engine (Rust/Actix-web)
│   ├── src/               # Source code
│   ├── tests/             # Integration tests
│   └── benches/           # Performance benchmarks
├── python-ai-service/     # AI/ML service (Python/FastAPI)
│   ├── models/            # ML models
│   ├── services/          # Business logic
│   └── tests/             # Unit & integration tests
├── nextjs-ui-dashboard/   # Frontend (React/Vite)
│   ├── src/               # Source code
│   ├── tests/             # Unit tests
│   └── e2e/               # E2E tests (Playwright)
├── specs/                 # Specifications (60 docs)
│   ├── 01-requirements/   # Functional & non-functional requirements
│   ├── 02-design/         # Architecture & design docs
│   ├── 03-testing/        # Test plans & test cases
│   ├── 04-deployment/     # Infrastructure & CI/CD
│   └── 05-operations/     # Operations & troubleshooting
├── docs/                  # Documentation
│   ├── reports/           # Quality & test reports
│   ├── certificates/      # Achievement certificates
│   └── testing/           # Testing documentation
└── scripts/               # Utility scripts
```

---

## Coding Standards

### Rust Standards

**Code Quality Rules:**
- Zero `unwrap()` or `expect()` in production code (use `?` operator)
- Comprehensive error handling (use custom error types)
- Zero compiler warnings
- Clippy clean with strict settings
- 90%+ test coverage
- Mutation score: 75%+ (current: 78%)

**Before Committing:**
```bash
cd rust-core-engine

# Format code
cargo fmt

# Check formatting
cargo fmt --check

# Lint with strict rules
cargo clippy --all-targets --all-features -- -D warnings

# Run all tests
cargo test

# Check coverage
cargo tarpaulin --out Html --skip-clean --timeout 180
```

**Example: Good Error Handling**
```rust
// ✅ GOOD - Use ? operator
pub fn execute_trade(&self, order: &Order) -> Result<TradeResult> {
    let account = self.get_account()?;
    let balance = account.get_balance(&order.symbol)?;
    self.validate_order(order, &balance)?;
    self.submit_order(order)
}

// ❌ BAD - Don't use unwrap
pub fn execute_trade(&self, order: &Order) -> TradeResult {
    let account = self.get_account().unwrap();  // Never do this!
    let balance = account.get_balance(&order.symbol).expect("balance");  // Never do this!
    self.submit_order(order)
}
```

**Example: @spec Tags**
```rust
// @spec:FR-TRADING-003 - Market Order Execution
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md
// @test:TC-TRADING-010, TC-TRADING-011, TC-TRADING-012
pub async fn execute_market_order(
    &self,
    symbol: &str,
    side: OrderSide,
    quantity: Decimal,
) -> Result<OrderResponse> {
    // Implementation...
}
```

### Python Standards

**Code Quality Rules:**
- Black formatted (100% compliance)
- Flake8 compliant (PEP 8)
- Type hints: 98%+ coverage
- 95%+ test coverage
- Zero HIGH/CRITICAL vulnerabilities
- Mutation score: 75%+ (current: 76%)

**Before Committing:**
```bash
cd python-ai-service

# Format code
black .

# Check formatting
black --check .

# Lint code
flake8 .

# Type check
mypy . --ignore-missing-imports

# Run tests with coverage
pytest --cov --cov-report=html --cov-report=term
```

**Example: Type Hints**
```python
# ✅ GOOD - Use type hints
from typing import List, Dict, Optional
from decimal import Decimal

def calculate_rsi(
    prices: List[Decimal],
    period: int = 14
) -> Optional[Decimal]:
    """Calculate RSI indicator.

    @spec:FR-AI-002 - Technical Indicator Calculation
    @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md
    @test:TC-AI-005, TC-AI-006
    """
    if len(prices) < period:
        return None
    # Implementation...

# ❌ BAD - No type hints
def calculate_rsi(prices, period=14):
    if len(prices) < period:
        return None
    # Implementation...
```

### TypeScript/React Standards

**Code Quality Rules:**
- ESLint: 0 errors, 0 warnings
- TypeScript strict mode enabled
- 90%+ test coverage
- Zero flaky tests
- Bundle optimized (< 500KB target)
- Mutation score: 75%+ (current: 75%)

**Before Committing:**
```bash
cd nextjs-ui-dashboard

# Lint code
npm run lint

# Type check
npm run type-check

# Run tests
npm run test

# Run tests with coverage
npm run test:coverage

# Build to verify
npm run build
```

**Example: TypeScript Strict Mode**
```typescript
// ✅ GOOD - Strict types
interface TradeRequest {
  symbol: string;
  side: 'BUY' | 'SELL';
  quantity: number;
  price?: number;
}

// @spec:FR-DASHBOARD-003 - Trade Execution UI
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-TRADING.md
// @test:TC-INTEGRATION-020
export const executeTrade = async (
  request: TradeRequest
): Promise<TradeResponse> => {
  // Implementation...
};

// ❌ BAD - Any types
export const executeTrade = async (request: any): Promise<any> => {
  // Implementation...
};
```

---

## Spec-Driven Development

**Bot-Core follows spec-driven development** - All features must conform to specifications BEFORE implementation.

### Specification System

**Location:** `specs/` directory (60 documents, 77,574 lines)

**Structure:**
- `01-requirements/` - Functional & non-functional requirements (194 requirements)
- `02-design/` - Architecture, database, API, UI/UX (20 docs)
- `03-testing/` - Test plans, test cases, scenarios (186 test cases)
- `04-deployment/` - Infrastructure, CI/CD, monitoring
- `05-operations/` - Operations manual, troubleshooting, DR plan

### Workflow: Read Spec → Update Spec → Tag Code → Test

**1. Read the Spec First**
```bash
# Check requirements
cat specs/01-requirements/1.1-functional/FR-TRADING.md

# Check design
cat specs/02-design/2.3-api-design/API-SPEC.md

# Check test cases
cat specs/03-testing/3.2-test-cases/TC-TRADING.md
```

**2. Update Spec if Needed**

If your feature requires spec changes:
- Update spec documents BEFORE coding
- Update `specs/TRACEABILITY_MATRIX.md`
- Get review/approval for spec changes

**3. Add @spec Tags to Code**

All production code must have @spec tags:

```rust
// @spec:FR-TRADING-005 - Stop-Loss Order
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md
// @test:TC-TRADING-020, TC-TRADING-021
```

```python
# @spec:FR-AI-010 - LSTM Price Prediction
# @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md
# @test:TC-AI-025, TC-AI-026
```

```typescript
// @spec:FR-DASHBOARD-008 - Real-time Price Chart
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-INTEGRATION-035
```

**4. Validate @spec Tags**

```bash
# Auto-tag code
python3 scripts/auto-tag-code.py

# Validate all tags
python3 scripts/validate-spec-tags.py
```

---

## Testing Requirements

### Coverage Requirements

**Minimum Coverage:**
- Rust: 90% (current: 90%)
- Python: 90% (current: 95%)
- TypeScript: 85% (current: 90%+)
- Overall: 90% (current: 90.4%)

### Test Types

**1. Unit Tests** - Test individual functions/methods
```bash
# Rust
cd rust-core-engine && cargo test

# Python
cd python-ai-service && pytest tests/unit/

# TypeScript
cd nextjs-ui-dashboard && npm run test
```

**2. Integration Tests** - Test service communication
```bash
# All integration tests
make test-integration

# Specific integrations
make test-rust-python        # Rust ↔ Python AI
make test-dashboard-rust     # Dashboard ↔ Rust API
make test-dashboard-python   # Dashboard ↔ Python AI
```

**3. E2E Tests** - Test complete user flows
```bash
cd nextjs-ui-dashboard
npm run test:e2e
npm run test:e2e:ui  # With Playwright UI
```

**4. Mutation Testing** - Test quality of tests

Target: 75%+ mutation score

```bash
# Rust (cargo-mutants)
cd rust-core-engine
cargo install cargo-mutants
cargo mutants

# TypeScript (Stryker)
cd nextjs-ui-dashboard
npm run test:mutation
```

### Test Quality Standards

**All new code must have:**
- Unit tests for all functions/methods
- Integration tests for service boundaries
- E2E tests for critical user flows
- Edge case testing
- Error handling testing
- Performance testing (where applicable)

**Test-Driven Development (TDD):**

1. **Write failing test first**
   ```rust
   #[test]
   fn test_market_order_execution() {
       let engine = TradingEngine::new();
       let result = engine.execute_market_order("BTCUSDT", Side::Buy, 0.001);
       assert!(result.is_ok());
   }
   ```

2. **Implement feature**
   ```rust
   pub fn execute_market_order(&self, symbol: &str, side: Side, quantity: f64) -> Result<OrderResponse> {
       // Implementation...
   }
   ```

3. **Verify test passes**
   ```bash
   cargo test test_market_order_execution
   ```

4. **Refactor if needed**

---

## Commit Message Format

**Bot-Core uses Conventional Commits** with semantic release automation.

### Format

```
<type>(<scope>): <subject>

<body>

<footer>
```

### Types

- `feat` - New feature (minor version bump)
- `fix` - Bug fix (patch version bump)
- `docs` - Documentation changes
- `style` - Code style changes (formatting, semicolons, etc.)
- `refactor` - Code refactoring (no functional changes)
- `perf` - Performance improvements
- `test` - Adding/updating tests
- `build` - Build system changes
- `ci` - CI/CD changes
- `chore` - Other changes (dependencies, etc.)
- `revert` - Revert previous commit

### Scopes

- `rust` - Rust Core Engine
- `python` - Python AI Service
- `frontend` - Next.js Dashboard
- `specs` - Specification updates
- `docs` - Documentation
- `ci` - CI/CD pipeline
- `docker` - Docker configuration

### Examples

**Feature:**
```
feat(rust): add WebSocket authentication middleware

Implement JWT-based authentication for WebSocket connections to secure
real-time price updates and trading signals.

@spec:FR-AUTH-005
Closes #123
```

**Bug Fix:**
```
fix(python): resolve LSTM model memory leak

Fix memory leak in LSTM prediction pipeline caused by TensorFlow session
not being properly closed after batch predictions.

@spec:FR-AI-010
Fixes #456
```

**Documentation:**
```
docs: update CONTRIBUTING guide with mutation testing requirements

Add section on mutation testing requirements and how to run cargo-mutants
and Stryker for code quality validation.
```

**Breaking Change:**
```
feat(rust): redesign order execution API

BREAKING CHANGE: Order execution endpoint now requires authentication
token in header. Update clients to include Authorization header.

@spec:FR-TRADING-003
```

### Commit Message Validation

**Automated validation via Husky:**
- `.commitlintrc.json` - Commit message linting rules
- `.husky/` - Git hooks for pre-commit validation

**Manual validation:**
```bash
# Commitlint checks commit message format
npm install -g @commitlint/cli @commitlint/config-conventional
echo "feat(rust): add new feature" | commitlint
```

---

## Pull Request Process

### 1. Create Feature Branch

```bash
# Create branch from main
git checkout main
git pull upstream main
git checkout -b feature/your-feature-name

# Or for bug fixes
git checkout -b bugfix/issue-description

# Or for hotfixes
git checkout -b hotfix/critical-issue
```

### 2. Make Changes

- Follow coding standards
- Add @spec tags to code
- Write tests (TDD approach)
- Update documentation
- Keep commits atomic and well-described

### 3. Run Quality Checks

```bash
# Lint all services
make lint

# Run all tests
make test

# Run integration tests
make test-integration

# Run quality metrics
make quality-metrics

# Run security scan
make security-check
```

**All checks must pass:**
- ✅ Zero lint errors/warnings
- ✅ All tests passing
- ✅ Coverage maintained (≥90%)
- ✅ Security scan clean (zero HIGH/CRITICAL)
- ✅ Quality metrics maintained (≥94/100)

### 4. Commit Changes

```bash
# Stage files
git add .

# Commit with conventional format
git commit -m "feat(rust): add your feature description"

# Push to your fork
git push origin feature/your-feature-name
```

### 5. Create Pull Request

1. **Go to GitHub** and create PR from your fork to upstream `main`
2. **Fill out PR template:**
   - Description of changes
   - Related issues/specs
   - Testing performed
   - Screenshots (for UI changes)
   - Breaking changes (if any)

3. **PR Title Format:**
   ```
   feat(rust): implement WebSocket authentication
   ```

4. **PR Description Template:**
   ```markdown
   ## Summary
   Brief description of what this PR does.

   ## Motivation
   Why is this change needed? What problem does it solve?

   ## Changes
   - Change 1
   - Change 2
   - Change 3

   ## Related Issues
   Closes #123
   Related to #456

   ## Specification
   @spec:FR-AUTH-005
   @ref:specs/02-design/2.5-components/COMP-RUST-AUTH.md

   ## Testing
   - [x] Unit tests added/updated
   - [x] Integration tests added/updated
   - [x] E2E tests added/updated (if applicable)
   - [x] All tests passing
   - [x] Coverage maintained

   ## Screenshots
   (If UI changes)

   ## Breaking Changes
   - None / List breaking changes

   ## Checklist
   - [x] Code follows project standards
   - [x] @spec tags added to new code
   - [x] Tests written and passing
   - [x] Documentation updated
   - [x] Commit messages follow conventional format
   - [x] All quality gates passed
   ```

### 6. Address Review Feedback

- Respond to all comments
- Make requested changes
- Push updates to same branch
- Request re-review when ready

### 7. Merge

Once approved:
- **Squash and merge** (preferred for clean history)
- **Merge commit** (for complex PRs)
- **Rebase and merge** (for simple PRs)

---

## Code Review Guidelines

### For Authors

**Before requesting review:**
- ✅ All quality checks passed
- ✅ PR description complete
- ✅ Tests added/updated
- ✅ Documentation updated
- ✅ No unrelated changes
- ✅ Commits follow conventional format

**During review:**
- Respond to feedback promptly
- Ask clarifying questions
- Explain design decisions
- Be open to suggestions
- Keep discussions professional

### For Reviewers

**What to check:**
1. **Functionality** - Does it work as expected?
2. **Spec Compliance** - Does it match specifications?
3. **Code Quality** - Follows coding standards?
4. **Tests** - Adequate test coverage?
5. **Performance** - No performance regressions?
6. **Security** - No security vulnerabilities?
7. **Documentation** - Is documentation updated?

**Review checklist:**
- [ ] Code follows project standards
- [ ] @spec tags present and correct
- [ ] Tests comprehensive and passing
- [ ] No hardcoded secrets
- [ ] Error handling comprehensive
- [ ] Performance acceptable
- [ ] Documentation updated
- [ ] Breaking changes documented

**Feedback style:**
- Be constructive and respectful
- Explain reasoning
- Suggest alternatives
- Approve when satisfied

---

## ClaudeKit Agent Usage

**Bot-Core includes ClaudeKit's AI agent orchestration** for enhanced development workflow.

### Available Agents

**Core Development Agents:**
- **planner** - Research & create implementation plans
- **researcher** - Technical investigation & analysis
- **tester** - Test execution & validation
- **code-reviewer** - Comprehensive code quality assessment
- **debugger** - Issue analysis & root cause finder

**Management Agents:**
- **docs-manager** - Documentation sync & maintenance
- **git-manager** - Version control & conventional commits
- **project-manager** - Progress tracking & roadmaps

### Custom Commands

**Essential commands:**
```bash
/plan [task]         # Create implementation plan
/cook [tasks]        # Implement features step-by-step
/test                # Run comprehensive test suite
/debug [issue]       # Debug issues
/docs                # Update documentation
/git [operation]     # Git operations with conventional commits
/watzup              # Project status check
```

### Agent Workflow Example

**Feature development:**
```bash
# 1. Plan the feature
/plan "implement JWT token refresh mechanism"

# 2. Implement following the plan
/cook "implement JWT refresh as per plan"

# 3. Test the implementation
/test

# 4. Review code quality
# (code-reviewer agent auto-invoked)

# 5. Update documentation
/docs

# 6. Commit with semantic versioning
/git "commit changes with conventional message"
```

**See `.claude/BOT_CORE_INSTRUCTIONS.md` for detailed agent usage.**

---

## Quality Gates

### Pre-Commit Quality Gates

**All commits must pass:**

1. **Linting** (zero errors/warnings)
   ```bash
   make lint
   ```

2. **Tests** (all passing)
   ```bash
   make test
   ```

3. **Coverage** (maintained ≥90%)
   ```bash
   # Rust
   cd rust-core-engine && cargo tarpaulin --out Stdout

   # Python
   cd python-ai-service && pytest --cov --cov-report=term

   # Frontend
   cd nextjs-ui-dashboard && npm run test:coverage
   ```

4. **Security** (zero HIGH/CRITICAL)
   ```bash
   make security-check
   ```

5. **Quality Metrics** (≥94/100)
   ```bash
   make quality-metrics
   ```

### Pre-PR Quality Gates

**All PRs must pass:**

1. **All pre-commit gates**
2. **Integration tests**
   ```bash
   make test-integration
   ```

3. **E2E tests**
   ```bash
   cd nextjs-ui-dashboard && npm run test:e2e
   ```

4. **Build verification**
   ```bash
   make build
   ```

5. **Documentation updated**
   - API changes documented
   - README updated (if needed)
   - Specs updated (if needed)

### CI/CD Quality Gates

**GitHub Actions checks:**
- ✅ Rust: format, clippy, test, build
- ✅ Python: flake8, black, pytest, coverage
- ✅ Frontend: lint, type-check, test, build
- ✅ Integration tests
- ✅ Security scan (Trivy, TruffleHog)
- ✅ Quality metrics
- ✅ FlyCI Wingman analysis (if installed)

**See `.github/workflows/flyci-wingman.yml` for full CI/CD pipeline.**

---

## Getting Help

### Resources

- **Documentation:** `docs/` directory
- **Specifications:** `specs/` directory (60 docs)
- **Testing Guide:** `docs/TESTING_GUIDE.md`
- **Troubleshooting:** `docs/TROUBLESHOOTING.md`
- **FlyCI Setup:** `docs/FLYCI_SETUP.md`
- **Claude Instructions:** `CLAUDE.md`

### Communication

- **GitHub Issues** - Bug reports & feature requests
- **GitHub Discussions** - Questions & discussions
- **Pull Requests** - Code contributions

### Common Questions

**Q: How do I run just one service?**
```bash
# Rust only
cd rust-core-engine && cargo run

# Python only
cd python-ai-service && python main.py

# Frontend only
cd nextjs-ui-dashboard && npm run dev
```

**Q: How do I reset my development environment?**
```bash
./scripts/bot.sh stop
./scripts/bot.sh clean
./scripts/bot.sh start --memory-optimized
```

**Q: How do I add a new dependency?**
```rust
// Rust: Edit Cargo.toml
[dependencies]
new_crate = "1.0"
```
```bash
# Python: Edit requirements.in, then
pip-compile requirements.in

# Frontend:
npm install new-package
# or
bun add new-package
```

**Q: Where do I add new tests?**
- Rust: `rust-core-engine/tests/`
- Python: `python-ai-service/tests/`
- Frontend: `nextjs-ui-dashboard/tests/` or `nextjs-ui-dashboard/e2e/`

**Q: How do I update documentation?**
- API docs: `specs/02-design/2.3-api-design/API-SPEC.md`
- General docs: `docs/` directory
- Component docs: Service-specific `docs/` folders

---

## Summary

**Key principles for contributing:**

1. **Read specs first** - Spec-driven development
2. **Write tests first** - TDD approach
3. **Follow standards** - Coding standards for Rust/Python/TypeScript
4. **Tag your code** - Add @spec tags to all production code
5. **Conventional commits** - Semantic commit messages
6. **Quality gates** - All checks must pass
7. **Professional PRs** - Complete PR descriptions with testing evidence

**Quality standards:**
- Test coverage: ≥90%
- Mutation score: ≥75%
- Lint: Zero errors/warnings
- Security: Zero HIGH/CRITICAL vulnerabilities
- Overall quality: ≥94/100 (Grade A)

**Thank you for contributing to Bot-Core!** Your contributions help maintain our **Perfect 10/10 quality score** and world-class status.

---

**Last Updated:** 2025-11-14
**Version:** 1.0.0
**Maintainers:** Bot-Core Development Team
