# Test Plan - Bot Core Trading Platform

**Document ID:** TEST-PLAN-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active

---

## Table of Contents

1. [Test Strategy Overview](#test-strategy-overview)
2. [Test Coverage Goals](#test-coverage-goals)
3. [Test Execution Strategy](#test-execution-strategy)
4. [Test Data Management](#test-data-management)
5. [Defect Management](#defect-management)
6. [Test Metrics](#test-metrics)
7. [Test Environments](#test-environments)
8. [Testing Tools and Frameworks](#testing-tools-and-frameworks)
9. [Test Schedule](#test-schedule)
10. [Risks and Mitigation](#risks-and-mitigation)

---

## 1. Test Strategy Overview

### 1.1 Testing Objectives

The primary objectives of testing the Bot Core trading platform are:

1. **Functional Correctness**: Verify all features work according to specifications
2. **Security Assurance**: Ensure authentication, authorization, and data protection
3. **Performance Validation**: Confirm system meets latency and throughput requirements
4. **Reliability**: Validate fault tolerance and error recovery mechanisms
5. **Integration Integrity**: Test communication between all microservices
6. **User Experience**: Ensure frontend components render correctly and are responsive

### 1.2 Testing Scope

#### In Scope:
- **Rust Core Engine** (`/Users/dungngo97/Documents/bot-core/rust-core-engine/`)
  - Authentication and authorization (JWT, password hashing)
  - Trading execution (market, limit, stop-loss orders)
  - Risk management (position sizing, leverage limits)
  - Binance API integration
  - WebSocket real-time data streaming
  - MongoDB persistence layer
  - Strategy engines (RSI, MACD, Bollinger, Volume)

- **Python AI Service** (`/Users/dungngo97/Documents/bot-core/python-ai-service/`)
  - Machine learning models (LSTM, GRU, Transformer)
  - Technical indicator calculations
  - GPT-4 integration
  - Feature engineering
  - Redis caching
  - FastAPI endpoints

- **Next.js Dashboard** (`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/`)
  - User authentication flows
  - Trading interface components
  - Real-time WebSocket updates
  - AI signal visualization
  - Portfolio management UI
  - Settings and configuration

- **Cross-Service Integration**
  - Rust ↔ Python AI communication
  - Rust ↔ Frontend API integration
  - End-to-end trade execution flows
  - Real-time data propagation

#### Out of Scope:
- Third-party libraries (assumed tested by vendors)
- Binance exchange infrastructure
- OpenAI GPT-4 API internals
- Browser-specific rendering (limited to Chrome/Firefox)

### 1.3 Testing Types

#### Unit Testing
- **Purpose**: Test individual functions and methods in isolation
- **Coverage Target**: 90%+ line coverage
- **Tools**:
  - Rust: cargo test, cargo tarpaulin
  - Python: pytest, pytest-cov
  - Frontend: Vitest, React Testing Library
- **Responsibility**: Developers

#### Integration Testing
- **Purpose**: Test interaction between modules and services
- **Coverage Target**: All service boundaries
- **Tools**:
  - Rust: Integration test files in `tests/`
  - Python: pytest with test fixtures
  - Frontend: Vitest integration tests
- **Responsibility**: Developers + QA

#### End-to-End Testing
- **Purpose**: Test complete user workflows
- **Coverage Target**: All critical user journeys
- **Tools**: Playwright
- **Responsibility**: QA Team

#### Performance Testing
- **Purpose**: Validate latency, throughput, and resource usage
- **Coverage Target**: All API endpoints and critical paths
- **Tools**: k6, Apache JMeter
- **Responsibility**: Performance Engineers

#### Security Testing
- **Purpose**: Identify vulnerabilities and security flaws
- **Coverage Target**: OWASP Top 10
- **Tools**: OWASP ZAP, Burp Suite
- **Responsibility**: Security Team

#### Mutation Testing
- **Purpose**: Validate test suite effectiveness
- **Coverage Target**: 75%+ mutation score
- **Tools**:
  - Rust: cargo-mutants
  - Python: mutmut
- **Responsibility**: Developers

---

## 2. Test Coverage Goals

### 2.1 Current Coverage Status

Based on analysis of existing test files:

#### Rust Core Engine
- **Unit Test Files**: 15 files in `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/`
- **Test Coverage**: 90.4% (target: 90%+) ✅
- **Mutation Score**: 76% (target: 75%+) ✅
- **Key Test Files**:
  - `test_auth.rs` - Authentication and JWT (11 tests)
  - `test_trading.rs` - Trading calculations (10 tests)
  - `test_strategies.rs` - Strategy engines
  - `test_binance_client.rs` - Binance API integration
  - `test_websocket.rs` - WebSocket functionality
  - `test_storage.rs` - MongoDB operations
  - `test_position_risk_comprehensive.rs` - Risk management
  - `test_indicators_comprehensive.rs` - Technical indicators
  - `test_paper_trading.rs` - Paper trading simulation
  - `test_cross_service.rs` - Cross-service integration
  - `test_service_integration.rs` - Full service integration

#### Python AI Service
- **Unit Test Files**: 20+ files in `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/`
- **Test Coverage**: 85%+ (target: 85%+) ✅
- **Key Test Files**:
  - `test_models.py` - ML model testing
  - `test_technical_analyzer.py` - Technical analysis (11 tests)
  - `test_gpt_analyzer.py` - GPT-4 integration
  - `test_technical_indicators.py` - Indicator calculations
  - `test_feature_engineering.py` - Feature engineering
  - `test_redis_cache.py` - Caching mechanism
  - `test_integration.py` - Service integration
  - `test_full_integration.py` - End-to-end flows
  - `test_security_fixes.py` - Security validations
  - `test_ml_performance.py` - Performance benchmarks

#### Next.js Dashboard
- **Unit Test Files**: 25+ files in `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/`
- **Test Coverage**: 80%+ (target: 80%+) ✅
- **Key Test Files**:
  - `pages/Login.test.tsx` - Login page
  - `pages/Register.test.tsx` - Registration
  - `pages/Dashboard.test.tsx` - Dashboard
  - `pages/TradingPaper.test.tsx` - Paper trading
  - `components/TradingInterface.test.tsx` - Trading UI
  - `components/dashboard/TradingCharts.test.tsx` - Charts
  - `components/dashboard/AISignals.test.tsx` - AI signals
  - `contexts/AuthContext.test.tsx` - Auth context
  - `hooks/useWebSocket.test.tsx` - WebSocket hook
  - `hooks/useAIAnalysis.test.tsx` - AI analysis hook
  - `hooks/usePaperTrading.test.tsx` - Paper trading hook
  - `integration/TradingFlow.test.tsx` - Trading workflows

### 2.2 Coverage Targets by Component

| Component | Unit Tests | Integration Tests | E2E Tests |
|-----------|------------|-------------------|-----------|
| Authentication | 95%+ | 100% | All login flows |
| Trading Engine | 90%+ | 100% | All order types |
| AI/ML Models | 85%+ | 100% | All prediction flows |
| Risk Management | 95%+ | 100% | All risk scenarios |
| WebSocket | 90%+ | 100% | All real-time updates |
| Frontend UI | 80%+ | 90% | Critical user paths |

### 2.3 Critical Path Coverage

Must have 100% test coverage for:
- JWT token generation and validation
- Password hashing and verification
- Order execution logic
- Position size calculations
- Risk limit enforcement
- Balance updates
- PnL calculations
- Database transactions
- API authentication middleware

---

## 3. Test Execution Strategy

### 3.1 Continuous Integration (CI/CD)

#### GitHub Actions Workflow
Location: `.github/workflows/test.yml`

**Automated Test Runs:**
1. **On Pull Request**: Run all unit and integration tests
2. **On Commit to Main**: Run full test suite including E2E
3. **Nightly Builds**: Run performance and security tests
4. **Pre-Release**: Run complete regression suite

**CI Pipeline Stages:**
```yaml
stages:
  - lint
  - unit-tests
  - integration-tests
  - security-scan
  - e2e-tests
  - mutation-tests
  - coverage-report
```

**Success Criteria for PR Merge:**
- All unit tests pass (100%)
- Integration tests pass (100%)
- Code coverage >= 90% (Rust), 85% (Python), 80% (Frontend)
- No critical security vulnerabilities
- Linting passes with zero errors

### 3.2 Manual Testing Procedures

#### Test Execution Workflow:
1. **Test Preparation**
   - Set up test environment (Docker containers)
   - Load test data
   - Configure test parameters

2. **Test Execution**
   - Execute test suites
   - Monitor test runs
   - Capture screenshots/logs for failures

3. **Result Analysis**
   - Review test results
   - Categorize failures
   - Create bug reports

4. **Test Reporting**
   - Generate coverage reports
   - Document test results
   - Update test metrics

#### Manual Test Schedule:
- **Daily**: Smoke tests on development environment
- **Weekly**: Regression tests on staging environment
- **Pre-Release**: Full manual testing cycle

### 3.3 Regression Testing

**Regression Test Strategy:**
- Maintain regression test suite covering all critical features
- Execute regression suite before each release
- Automate regression tests in CI/CD pipeline

**Regression Test Categories:**
1. **Core Functionality**: Authentication, trading, AI analysis
2. **Bug Fixes**: Tests for previously fixed bugs
3. **Integration Points**: Service communication
4. **Performance**: Latency and throughput benchmarks

**Regression Test Execution:**
```bash
# Rust regression tests
cd /Users/dungngo97/Documents/bot-core/rust-core-engine
cargo test --release

# Python regression tests
cd /Users/dungngo97/Documents/bot-core/python-ai-service
pytest tests/ --cov=./ --cov-report=html

# Frontend regression tests
cd /Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard
npm run test:coverage
```

### 3.4 Smoke Testing

**Smoke Test Checklist:**
- [ ] Services start successfully
- [ ] Health check endpoints respond
- [ ] Database connections established
- [ ] User can log in
- [ ] User can view dashboard
- [ ] Market data updates in real-time
- [ ] AI analysis returns predictions
- [ ] Paper trading executes orders

**Smoke Test Execution:**
```bash
# Start all services
cd /Users/dungngo97/Documents/bot-core
./scripts/bot.sh start --memory-optimized

# Run smoke tests
./scripts/smoke-test.sh
```

---

## 4. Test Data Management

### 4.1 Test Data Generation

#### Synthetic Test Data:
- **Market Data**: Generate realistic kline/candlestick data
- **User Data**: Create test users with various roles
- **Trade Data**: Simulate trade history
- **Portfolio Data**: Generate portfolio positions

**Test Data Generator:**
```python
# Location: /Users/dungngo97/Documents/bot-core/tests/data_generator.py
def generate_market_data(symbol, interval, count):
    """Generate synthetic market data for testing."""
    pass

def generate_test_users(count):
    """Create test user accounts."""
    pass

def generate_trade_history(user_id, count):
    """Generate trade history for testing."""
    pass
```

#### Mock Data:
- **Binance API Responses**: Mock API responses for testing
- **OpenAI GPT Responses**: Mock GPT-4 responses
- **WebSocket Messages**: Mock real-time data streams

### 4.2 Test Data Storage

**Test Data Locations:**
- `/Users/dungngo97/Documents/bot-core/tests/fixtures/` - Static test fixtures
- `/Users/dungngo97/Documents/bot-core/tests/data/` - Generated test data
- MongoDB test database: `bot_core_test`
- Redis test cache: Database 1 (DB0 for production)

### 4.3 Test Data Cleanup

**Cleanup Strategy:**
- Clean up test data after each test run
- Use database transactions for rollback
- Implement teardown hooks in test frameworks

**Cleanup Procedures:**
```rust
// Rust cleanup example
#[tokio::test]
async fn test_with_cleanup() {
    let db = setup_test_db().await;

    // Test code here

    cleanup_test_db(&db).await;
}
```

```python
# Python cleanup example
@pytest.fixture
def db_session():
    session = create_test_session()
    yield session
    cleanup_test_data(session)
    session.close()
```

### 4.4 Test Data Privacy

**Privacy Considerations:**
- Never use production data in testing
- Anonymize any real user data
- Use fake email addresses and usernames
- Generate random API keys (never use real keys)

---

## 5. Defect Management

### 5.1 Bug Tracking Process

**Bug Report Template:**
```markdown
### Bug ID: BUG-XXXX
**Severity**: Critical / High / Medium / Low
**Priority**: P0 / P1 / P2 / P3
**Component**: Rust Engine / Python AI / Frontend
**Environment**: Dev / Staging / Production

**Description:**
Clear description of the issue

**Steps to Reproduce:**
1. Step one
2. Step two
3. Step three

**Expected Result:**
What should happen

**Actual Result:**
What actually happened

**Screenshots/Logs:**
Attach relevant evidence

**Related Test Case:**
TC-XXX-YYY
```

### 5.2 Severity Levels

| Severity | Definition | Example | SLA |
|----------|------------|---------|-----|
| **Critical** | System down, data loss, security breach | Authentication bypass | Fix within 4 hours |
| **High** | Major feature broken, significant impact | Trading execution fails | Fix within 24 hours |
| **Medium** | Feature partially broken, workaround exists | Chart not updating | Fix within 3 days |
| **Low** | Minor issue, cosmetic problem | UI alignment issue | Fix within 1 week |

### 5.3 Bug Lifecycle

```
New → Assigned → In Progress → Fixed → Testing → Verified → Closed
                                    ↓
                               Reopened (if bug persists)
```

**Bug States:**
1. **New**: Bug reported, awaiting triage
2. **Assigned**: Developer assigned to fix
3. **In Progress**: Developer working on fix
4. **Fixed**: Fix committed, awaiting testing
5. **Testing**: QA testing the fix
6. **Verified**: Fix confirmed, bug resolved
7. **Closed**: Bug archived
8. **Reopened**: Bug recurs after fix

### 5.4 Bug Metrics

**Tracked Metrics:**
- Bugs found per release
- Bug fix rate
- Bug reopen rate
- Average time to fix (by severity)
- Bugs by component
- Bugs by test phase (unit, integration, E2E)

---

## 6. Test Metrics

### 6.1 Code Coverage Metrics

**Coverage Tools:**
- **Rust**: cargo-tarpaulin
- **Python**: pytest-cov, coverage.py
- **Frontend**: Vitest coverage (v8)

**Coverage Reports:**
```bash
# Generate Rust coverage
cd /Users/dungngo97/Documents/bot-core/rust-core-engine
cargo tarpaulin --out Html --output-dir coverage

# Generate Python coverage
cd /Users/dungngo97/Documents/bot-core/python-ai-service
pytest --cov=./ --cov-report=html

# Generate Frontend coverage
cd /Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard
npm run test:coverage
```

**Current Coverage Status:**
- Rust Core: 90.4% line coverage ✅
- Python AI: 85%+ line coverage ✅
- Frontend: 80%+ line coverage ✅

### 6.2 Test Pass/Fail Rates

**Metrics to Track:**
- Total tests executed
- Tests passed
- Tests failed
- Tests skipped
- Pass rate percentage
- Trend over time

**Target Pass Rates:**
- Unit tests: 100%
- Integration tests: 100%
- E2E tests: 95%+ (some flakiness acceptable)

### 6.3 Defect Density

**Definition**: Number of defects per 1000 lines of code (KLOC)

**Formula**:
```
Defect Density = (Total Defects / Total KLOC)
```

**Industry Benchmark**: < 1 defect per KLOC
**Target**: < 0.5 defects per KLOC

### 6.4 Test Execution Time

**Metrics to Track:**
- Unit test execution time
- Integration test execution time
- E2E test execution time
- Total test suite execution time

**Current Benchmarks:**
- Rust unit tests: ~30 seconds
- Python unit tests: ~45 seconds
- Frontend unit tests: ~20 seconds
- Full E2E suite: ~10 minutes

**Optimization Targets:**
- Keep unit tests under 1 minute total
- Integration tests under 5 minutes
- Full test suite under 15 minutes

### 6.5 Mutation Testing Scores

**Mutation Testing**: Modify code to introduce bugs and verify tests catch them

**Current Mutation Scores:**
- Rust Core: 76% (target: 75%+) ✅

**Mutation Testing Tools:**
- **Rust**: cargo-mutants
- **Python**: mutmut

**Mutation Test Execution:**
```bash
# Rust mutation testing
cd /Users/dungngo97/Documents/bot-core/rust-core-engine
cargo mutants

# Python mutation testing
cd /Users/dungngo97/Documents/bot-core/python-ai-service
mutmut run
```

---

## 7. Test Environments

### 7.1 Development Environment

**Purpose**: Developer local testing

**Configuration:**
- Docker Compose: `docker-compose.yml`
- Environment: `.env` file
- Services: All services running locally
- Database: MongoDB test instance
- Cache: Redis test instance

**Access:**
```bash
# Start dev environment
cd /Users/dungngo97/Documents/bot-core
./scripts/bot.sh dev
```

**Endpoints:**
- Frontend: http://localhost:3000
- Rust API: http://localhost:8080
- Python API: http://localhost:8000
- MongoDB: mongodb://localhost:27017

### 7.2 Staging Environment

**Purpose**: Pre-production testing

**Configuration:**
- Binance Testnet enabled
- Realistic data volumes
- Production-like configuration
- Separate database instance

**Access:** (To be configured)
- Frontend: https://staging.botcore.example
- APIs: https://api-staging.botcore.example

### 7.3 Production Environment

**Purpose**: Live trading (disabled by default)

**Configuration:**
- `TRADING_ENABLED=false` (safety flag)
- `BINANCE_TESTNET=true` (testnet by default)
- Real-time monitoring
- Backup and disaster recovery

**Safety Measures:**
- Manual activation required for live trading
- Multi-factor authentication
- Audit logging
- Circuit breakers

### 7.4 CI/CD Environment

**Purpose**: Automated testing in GitHub Actions

**Configuration:**
- GitHub Actions runners
- Ephemeral Docker containers
- Mocked external services
- Isolated test databases

**Workflow Files:**
- `.github/workflows/test.yml`
- `.github/workflows/deploy.yml`

---

## 8. Testing Tools and Frameworks

### 8.1 Rust Testing Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| **cargo test** | Unit & integration testing | `cargo test` |
| **cargo tarpaulin** | Code coverage | `cargo tarpaulin --out Html` |
| **cargo-mutants** | Mutation testing | `cargo mutants` |
| **cargo clippy** | Linting | `cargo clippy` |
| **mockall** | Mocking framework | Used in test files |

**Example Test:**
```rust
// /Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_auth.rs
#[test]
fn test_password_hash_and_verify() {
    let password = "SecurePassword123!@#";
    let hash = PasswordService::hash_password(password).unwrap();
    assert!(PasswordService::verify_password(password, &hash).unwrap());
}
```

### 8.2 Python Testing Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| **pytest** | Testing framework | `pytest tests/` |
| **pytest-cov** | Code coverage | `pytest --cov=./` |
| **pytest-asyncio** | Async testing | Async test support |
| **unittest.mock** | Mocking | Mock external dependencies |
| **mutmut** | Mutation testing | `mutmut run` |
| **flake8** | Linting | `flake8 .` |
| **black** | Code formatting | `black .` |

**Example Test:**
```python
# /Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_analyzer.py
@pytest.mark.unit
def test_calculate_indicators(technical_analyzer, sample_klines):
    df = technical_analyzer.prepare_dataframe(sample_klines)
    indicators = technical_analyzer.calculate_indicators(df)

    assert "rsi" in indicators
    assert isinstance(indicators["rsi"], (int, float))
```

### 8.3 Frontend Testing Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| **Vitest** | Testing framework | `npm run test` |
| **React Testing Library** | Component testing | Render components |
| **@testing-library/jest-dom** | DOM matchers | Assertions |
| **MSW (Mock Service Worker)** | API mocking | Mock API calls |
| **Playwright** | E2E testing | `npm run test:e2e` |
| **ESLint** | Linting | `npm run lint` |

**Example Test:**
```typescript
// /Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/pages/Login.test.tsx
describe('Login Page', () => {
  it('should render login form', () => {
    render(<Login />);
    expect(screen.getByText(/login/i)).toBeInTheDocument();
  });
});
```

### 8.4 Integration Testing Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| **Docker Compose** | Service orchestration | `docker-compose up` |
| **Testcontainers** | Containerized testing | Spin up dependencies |
| **Postman/Newman** | API testing | Collection runner |

### 8.5 Performance Testing Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| **k6** | Load testing | `k6 run script.js` |
| **Apache JMeter** | Performance testing | GUI/CLI execution |
| **Artillery** | Load testing | `artillery run scenario.yml` |
| **hey** | HTTP load generator | `hey -n 1000 URL` |

### 8.6 Security Testing Tools

| Tool | Purpose | Usage |
|------|---------|-------|
| **OWASP ZAP** | Security scanning | Automated scans |
| **Burp Suite** | Penetration testing | Manual testing |
| **SQLMap** | SQL injection testing | Injection detection |
| **cargo-audit** | Dependency scanning | `cargo audit` |
| **Safety** | Python security check | `safety check` |

---

## 9. Test Schedule

### 9.1 Development Phase Testing

**Daily:**
- Run unit tests before committing
- Fix failing tests immediately
- Update tests for new features

**Weekly:**
- Review code coverage reports
- Execute integration test suite
- Update test documentation

### 9.2 Release Testing Schedule

**2 Weeks Before Release:**
- Feature freeze
- Begin integration testing
- Start E2E test execution

**1 Week Before Release:**
- Complete regression testing
- Execute performance tests
- Conduct security scans

**3 Days Before Release:**
- Final regression suite
- Smoke tests on staging
- UAT (User Acceptance Testing)

**Release Day:**
- Deploy to production
- Execute smoke tests
- Monitor for issues

**Post-Release:**
- Monitor error logs
- Track performance metrics
- Gather user feedback

### 9.3 Continuous Testing Timeline

```
Commit → Unit Tests (30 sec)
  ↓
PR Created → Integration Tests (5 min)
  ↓
PR Merged → Full Test Suite (15 min)
  ↓
Nightly → Performance + Security Tests (1 hour)
  ↓
Weekly → Mutation Testing (2 hours)
```

---

## 10. Risks and Mitigation

### 10.1 Testing Risks

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Insufficient test coverage | High | Medium | Enforce 90%+ coverage requirement |
| Flaky E2E tests | Medium | High | Implement retry logic, improve test stability |
| Slow test execution | Medium | Medium | Parallelize tests, optimize slow tests |
| Missing edge cases | High | Medium | Conduct code reviews, add property-based testing |
| Production data leakage | Critical | Low | Strict data isolation policies |
| Third-party API failures | Medium | Medium | Mock external APIs, use test accounts |
| Resource constraints | Medium | High | Use --memory-optimized flag, optimize containers |

### 10.2 Risk Mitigation Strategies

#### Test Stability
- **Issue**: Flaky tests cause false negatives
- **Mitigation**:
  - Use deterministic test data
  - Avoid time-dependent tests
  - Implement proper test isolation
  - Use retry mechanisms for E2E tests

#### Test Performance
- **Issue**: Slow tests delay development
- **Mitigation**:
  - Parallelize test execution
  - Use faster test doubles (mocks, stubs)
  - Optimize database operations
  - Cache test dependencies

#### Test Maintenance
- **Issue**: Tests become outdated
- **Mitigation**:
  - Review tests during code reviews
  - Update tests with feature changes
  - Remove obsolete tests
  - Document test intentions

---

## Appendix A: Test File Locations

### Rust Core Engine Tests
```
/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/
├── common/mod.rs
├── test_auth.rs
├── test_trading.rs
├── test_strategies.rs
├── test_binance_client.rs
├── test_websocket.rs
├── test_storage.rs
├── test_market_data.rs
├── test_ai.rs
├── test_config.rs
├── test_position_risk_comprehensive.rs
├── test_indicators_comprehensive.rs
├── test_paper_trading.rs
├── test_cross_service.rs
└── test_service_integration.rs
```

### Python AI Service Tests
```
/Users/dungngo97/Documents/bot-core/python-ai-service/tests/
├── __init__.py
├── conftest.py
├── test_basic.py
├── test_logger.py
├── test_config.py
├── test_models.py
├── test_technical_analyzer.py
├── test_technical_indicators.py
├── test_gpt_analyzer.py
├── test_feature_engineering.py
├── test_redis_cache.py
├── test_integration.py
├── test_full_integration.py
├── test_security_fixes.py
├── test_ml_performance.py
└── test_websocket.py
```

### Frontend Tests
```
/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/
├── components/
│   ├── TradingInterface.test.tsx
│   ├── ErrorBoundary.test.tsx
│   └── dashboard/
│       ├── TradingCharts.test.tsx
│       ├── TradingSettings.test.tsx
│       ├── AISignals.test.tsx
│       ├── PerformanceChart.test.tsx
│       └── DashboardHeader.test.tsx
├── pages/
│   ├── Login.test.tsx
│   ├── Register.test.tsx
│   ├── Dashboard.test.tsx
│   ├── TradingPaper.test.tsx
│   └── Settings.test.tsx
├── hooks/
│   ├── useWebSocket.test.tsx
│   ├── useAIAnalysis.test.tsx
│   ├── usePaperTrading.test.tsx
│   └── useMarketData.test.tsx
├── services/
│   ├── api.test.ts
│   └── chatbot.test.ts
└── integration/
    ├── TradingFlow.test.tsx
    └── api-integration.test.tsx
```

---

## Appendix B: Test Execution Commands

### Rust Tests
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_password_hash_and_verify

# Run tests with coverage
cargo tarpaulin --out Html --output-dir coverage

# Run mutation tests
cargo mutants

# Run tests in release mode (faster)
cargo test --release
```

### Python Tests
```bash
# Run all tests
pytest tests/

# Run with coverage
pytest --cov=./ --cov-report=html

# Run specific test file
pytest tests/test_technical_analyzer.py

# Run with markers
pytest -m unit

# Run mutation tests
mutmut run
```

### Frontend Tests
```bash
# Run all tests
npm run test

# Run with coverage
npm run test:coverage

# Run in watch mode
npm run test:watch

# Run E2E tests
npm run test:e2e

# Run specific test file
npm run test -- Login.test.tsx
```

### Integration Tests
```bash
# Start services and run integration tests
./scripts/bot.sh start --memory-optimized
./scripts/run-integration-tests.sh
```

---

## Appendix C: Test Metrics Dashboard

**Metrics to Track:**
1. Code coverage percentage (by service)
2. Test pass/fail rates
3. Test execution time trends
4. Defect density
5. Mutation test scores
6. Bug resolution time
7. Test flakiness rate

**Dashboard Tools:**
- SonarQube (code quality)
- Codecov (coverage visualization)
- GitHub Actions insights (CI/CD metrics)

---

## Document Approval

| Role | Name | Signature | Date |
|------|------|-----------|------|
| QA Lead | [Name] | [Signature] | [Date] |
| Engineering Manager | [Name] | [Signature] | [Date] |
| Product Owner | [Name] | [Signature] | [Date] |

---

**Document Control:**
- **Created by**: QA Team
- **Reviewed by**: Engineering Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Test Plan Document*
