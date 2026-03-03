# SPECS UPDATE PLAN - COMPREHENSIVE
# Finance Project: Cryptocurrency Trading Bot

**Date**: 2025-11-22
**Last Spec Update**: 2025-10-11 (1.5 months gap)
**Current Code Date**: 2025-11-22
**Total Commits Since Last Update**: 351 commits
**Status**: CRITICAL - Specs outdated, code ahead
**Priority**: EXTREME HIGH (Finance = Money at stake)

---

## EXECUTIVE SUMMARY

**Gap Analysis**: 1.5 months (Oct 11 → Nov 22), 351 commits
**Impact**: Critical features implemented without specs (violations of spec-driven development)
**Risk Level**: HIGH - Trading logic changes not documented in specs
**Estimated Effort**: 24-32 hours (3-4 full days) across multiple work sessions
**Success Criteria**: 100% code-to-spec traceability, all features documented, finance-critical features have extreme detail

**Key Findings**:
- 7 MAJOR features without specs (CRITICAL)
- 11 existing specs need updates (HIGH)
- 4 new infrastructure components (MEDIUM)
- 100+ bug fixes and improvements (documentation needed)

---

## 1. GAP ANALYSIS

### 1.1 Features NEW in Code BUT NOT in Specs (CRITICAL)

**Priority: EXTREME HIGH - These are spec violations**

| Feature | Location | Lines | Priority | Complexity | Reason |
|---------|----------|-------|----------|------------|--------|
| **Async Tasks System (RabbitMQ + Celery)** | `python-ai-service/tasks/`, `celery_app.py` | ~2,500 | CRITICAL | VERY HIGH | Changes trading execution flow, monitoring, AI retraining |
| **Trailing Stop Loss** | `rust-core-engine/src/paper_trading/` | ~300 | CRITICAL | MEDIUM | Risk management feature - affects money |
| **Stochastic Strategy** | `rust-core-engine/src/strategies/stochastic_strategy.rs` | ~250 | CRITICAL | MEDIUM | 5th trading strategy - affects trading decisions |
| **Instant Warmup System** | `rust-core-engine/src/paper_trading/engine.rs`, `binance/mod.rs` | ~400 | HIGH | MEDIUM | Pre-loads historical data for instant strategy execution |
| **E2E Tests (Playwright)** | `nextjs-ui-dashboard/e2e/` | ~800 | HIGH | MEDIUM | Testing infrastructure - quality assurance |
| **Visual Regression Tests** | `nextjs-ui-dashboard/e2e/visual-regression.spec.ts` | ~200 | MEDIUM | LOW | UI testing - prevents visual bugs |
| **External AI Signal Processing** | `rust-core-engine/src/paper_trading/engine.rs` | ~150 | HIGH | MEDIUM | Frontend-Rust integration for AI signals |

**Total**: 7 major features, ~4,600 lines of undocumented code

---

### 1.2 Features in Specs BUT Code Changed (OUTDATED)

**Priority: HIGH - Specs are lying**

| Spec ID | Feature | Last Update | Status | What Changed |
|---------|---------|-------------|--------|--------------|
| FR-STRATEGIES-001 to FR-STRATEGIES-004 | RSI, MACD, Bollinger, Volume | Oct 10 | OUTDATED | Missing Stochastic (5th strategy), missing multi-timeframe support (15m default) |
| FR-PAPER-TRADING-001 to FR-PAPER-TRADING-010 | Paper Trading Engine | Oct 10 | OUTDATED | Missing trailing stop, instant warmup, external AI signals, data resolution selector |
| FR-RISK-001 to FR-RISK-006 | Risk Management | Oct 10 | OUTDATED | Missing trailing stop settings, correlation limit (70%), cool-down mechanism details |
| FR-AI-001 to FR-AI-010 | AI/ML Service | Oct 10 | OUTDATED | Missing async tasks (11 jobs), GPT-4 self-improvement, adaptive retraining |
| FR-DASHBOARD-001 to FR-DASHBOARD-010 | Frontend Dashboard | Oct 10 | OUTDATED | Missing per-symbol settings UI, timeframe selector, AI analysis improvements |
| FR-TRADING-001 to FR-TRADING-010 | Trading Engine | Oct 10 | PARTIAL | Missing external signal processing |
| NFR-PERFORMANCE-001 to NFR-PERFORMANCE-005 | Performance Requirements | Oct 10 | OUTDATED | Missing async job performance targets |
| NFR-SECURITY-001 to NFR-SECURITY-010 | Security Requirements | Oct 10 | PARTIAL | Added bun security checks, npm audit replacement |
| DB-SCHEMA.md | Database Schema | Oct 10 | OUTDATED | Missing 5 new collections (async tasks, performance metrics, etc.) |
| API-RUST-CORE.md | Rust API Spec | Oct 10 | OUTDATED | Missing endpoints for trailing stop, external signals, settings |
| API-PYTHON-AI.md | Python AI API Spec | Oct 10 | OUTDATED | Missing async task endpoints, new scheduled jobs |

**Total**: 11 specs need significant updates

---

### 1.3 Features CORRECTLY Documented (No Update Needed)

**Priority: LOW - These are accurate**

| Spec ID | Feature | Status | Reason |
|---------|---------|--------|--------|
| FR-AUTH-001 to FR-AUTH-010 | Authentication & JWT | ✅ ACCURATE | No changes to auth system |
| FR-WEBSOCKET-001 to FR-WEBSOCKET-005 | WebSocket Communication | ✅ ACCURATE | Core WS logic unchanged |
| FR-MARKET-DATA-001 to FR-MARKET-DATA-005 | Market Data Processing | ✅ MOSTLY ACCURATE | Minor caching updates (documented in code) |
| FR-PORTFOLIO-001 to FR-PORTFOLIO-006 | Portfolio Management | ✅ MOSTLY ACCURATE | Margin calculation fixes (minor) |
| NFR-RELIABILITY-001 to NFR-RELIABILITY-005 | Reliability Requirements | ✅ ACCURATE | No major changes |
| NFR-SCALABILITY-001 to NFR-SCALABILITY-005 | Scalability Requirements | ✅ ACCURATE | Async tasks improve scalability (will update) |
| ARCH-OVERVIEW.md | System Architecture | ✅ MOSTLY ACCURATE | Need to add RabbitMQ/Redis/Celery |
| ARCH-SECURITY.md | Security Architecture | ✅ ACCURATE | Core security unchanged |

**Total**: 8 specs remain accurate

---

### 1.4 Infrastructure Changes (Need Documentation)

**Priority: HIGH - New dependencies**

| Component | Version | Purpose | Impact |
|-----------|---------|---------|--------|
| **RabbitMQ** | 3.12 | Message broker for async tasks | Critical - new infrastructure dependency |
| **Redis** | 7-alpine | Results backend for Celery | Critical - new data store |
| **Celery** | 5.4.0 | Distributed task queue | Critical - core async processing |
| **Flower** | Latest | Celery monitoring UI | Medium - operations tool |
| **Playwright** | Latest | E2E testing framework | Medium - testing infrastructure |

**Total**: 5 new infrastructure components

---

## 2. UPDATE STRATEGY

### 2.1 Prioritization Matrix

**CRITICAL (Week 1 - Days 1-2)**: Finance-critical features that affect money

1. **Async Tasks System** (FR-ASYNC-TASKS.md) - Changes trading execution, retraining
2. **Trailing Stop Loss** (Update FR-RISK.md, FR-PAPER-TRADING.md) - Risk management
3. **Stochastic Strategy** (Update FR-STRATEGIES.md) - 5th trading strategy
4. **Database Schema Updates** (Update DB-SCHEMA.md) - 5 new collections

**Rationale**: These features directly affect trading decisions, money at risk, and system behavior. Must be documented first.

---

**HIGH (Week 1 - Days 3-4)**: Important features that affect system behavior

5. **Instant Warmup System** (Update FR-PAPER-TRADING.md)
6. **External AI Signal Processing** (Update FR-TRADING.md, FR-PAPER-TRADING.md)
7. **Multi-Timeframe Support** (Update FR-STRATEGIES.md, FR-MARKET-DATA.md)
8. **API Updates** (Update API-RUST-CORE.md, API-PYTHON-AI.md)

**Rationale**: System functionality changes that need documentation for developers.

---

**MEDIUM (Week 2 - Days 5-6)**: Testing and UI improvements

9. **E2E Tests (Playwright)** (Create TEST-E2E.md in 03-testing/)
10. **Visual Regression Tests** (Update TEST-PLAN.md)
11. **Frontend UI Updates** (Update FR-DASHBOARD.md, UI-COMPONENTS.md)
12. **Performance Updates** (Update NFR-PERFORMANCE.md)

**Rationale**: Quality assurance and user experience improvements.

---

**LOW (Week 2 - Days 7-8)**: Infrastructure and operations

13. **Infrastructure Updates** (Update INFRA-DOCKER.md)
14. **Monitoring Updates** (Update MON-LOGGING.md, MON-METRICS.md)
15. **CI/CD Updates** (Update CICD-PIPELINE.md)
16. **Operations Manual** (Update OPS-MANUAL.md)

**Rationale**: DevOps and operations documentation.

---

### 2.2 Specs Files to CREATE (NEW)

**Phase 1 (CRITICAL)**:

1. **`specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md`**
   - Complete async tasks system documentation
   - 11 jobs (5 scheduled, 4 on-demand, 2 alert-triggered)
   - RabbitMQ queues, Celery configuration
   - Notification system (Email, Slack, Discord, Telegram)
   - Data storage (5 MongoDB collections)
   - GPT-4 self-improvement logic
   - Estimated: 1,500-2,000 lines (finance-critical, extreme detail)

2. **`specs/03-testing/3.2-test-cases/TC-E2E.md`**
   - E2E test cases with Playwright
   - Critical user flows (auth, trading, settings, dashboard)
   - Visual regression test scenarios
   - Test data and setup requirements
   - Estimated: 800-1,000 lines

3. **`specs/03-testing/3.5-security/SEC-TEST-E2E.md`**
   - Security-focused E2E tests
   - Auth flow security, XSS prevention, CSRF protection
   - Estimated: 500 lines

**Phase 2 (HIGH)**:

4. **`specs/04-deployment/4.1-infrastructure/INFRA-ASYNC-TASKS.md`**
   - RabbitMQ deployment and configuration
   - Celery worker scaling strategy
   - Redis configuration and persistence
   - Flower monitoring setup
   - Estimated: 600-800 lines

**Total**: 4 new spec files, ~3,400-4,300 lines

---

### 2.3 Specs Files to UPDATE (EXISTING)

**Phase 1 (CRITICAL - Finance Impact)**:

| File | Current Lines | Add Lines | Update Type | Priority | Reason |
|------|---------------|-----------|-------------|----------|--------|
| `FR-RISK.md` | 49,469 | +2,000 | MAJOR | CRITICAL | Add trailing stop sections (FR-RISK-007, FR-RISK-008) |
| `FR-PAPER-TRADING.md` | 67,357 | +1,500 | MAJOR | CRITICAL | Add trailing stop, instant warmup, external signals |
| `FR-STRATEGIES.md` | 71,350 | +2,500 | MAJOR | CRITICAL | Add Stochastic strategy (FR-STRATEGIES-009 to FR-STRATEGIES-012), multi-timeframe |
| `DB-SCHEMA.md` | ~20,000 | +3,000 | MAJOR | CRITICAL | Add 5 collections (gpt4_analysis, performance_metrics, model_accuracy, api_costs, retrain_history) |
| `FR-AI.md` | 85,815 | +2,000 | MAJOR | CRITICAL | Update with async tasks, GPT-4 self-improvement |

**Phase 2 (HIGH - System Behavior)**:

| File | Current Lines | Add Lines | Update Type | Priority |
|------|---------------|-----------|-------------|----------|
| `API-RUST-CORE.md` | ~30,000 | +1,500 | MAJOR | HIGH |
| `API-PYTHON-AI.md` | ~25,000 | +1,200 | MAJOR | HIGH |
| `FR-DASHBOARD.md` | 58,334 | +1,000 | MEDIUM | HIGH |
| `FR-MARKET-DATA.md` | 27,636 | +500 | MINOR | HIGH |

**Phase 3 (MEDIUM - Testing & UI)**:

| File | Current Lines | Add Lines | Update Type | Priority |
|------|---------------|-----------|-------------|----------|
| `TEST-PLAN.md` | ~15,000 | +800 | MEDIUM | MEDIUM |
| `UI-COMPONENTS.md` | ~12,000 | +600 | MINOR | MEDIUM |
| `NFR-PERFORMANCE.md` | ~18,000 | +400 | MINOR | MEDIUM |

**Phase 4 (LOW - Infrastructure)**:

| File | Current Lines | Add Lines | Update Type | Priority |
|------|---------------|-----------|-------------|----------|
| `INFRA-DOCKER.md` | ~10,000 | +800 | MEDIUM | LOW |
| `CICD-PIPELINE.md` | ~8,000 | +300 | MINOR | LOW |
| `OPS-MANUAL.md` | ~20,000 | +500 | MINOR | LOW |

**Total**: 15 files to update, ~18,100 new lines

---

### 2.4 Traceability Updates Required

**MANDATORY for ALL updates**:

1. **Update TRACEABILITY_MATRIX.md**:
   - Add new requirement IDs (FR-ASYNC-001 to FR-ASYNC-015, FR-RISK-007, FR-RISK-008, etc.)
   - Link to implementation files
   - Link to test cases
   - Estimated: +150 rows

2. **Update @spec tags in code**:
   - Current: 48 @spec tags in 20 files
   - Need to add: ~30 new @spec tags
   - Files needing tags:
     - `python-ai-service/tasks/*.py` (7 files)
     - `rust-core-engine/src/paper_trading/` (trailing stop logic)
     - `rust-core-engine/src/strategies/stochastic_strategy.rs`
     - `nextjs-ui-dashboard/e2e/*.spec.ts` (5 files)

3. **Update TASK_TRACKER.md**:
   - Mark completed tasks
   - Add new feature tracking
   - Update completion percentages
   - Estimated: +50 rows

**Total Traceability Work**: ~200 updates across 3 files + 15 code files

---

## 3. IMPLEMENTATION PLAN

### Phase 1: CRITICAL Features (Days 1-2, 16-20 hours)

**DAY 1 (8-10 hours)**:

#### Morning (4-5 hours):
- [ ] **Task 1.1**: Create `FR-ASYNC-TASKS.md` (1,800 lines)
  - Section 1: Overview and business case (GPT-4 cost savings: $430/year)
  - Section 2: Architecture (RabbitMQ, Celery, Redis, Flower)
  - Section 3: 11 Async Jobs (5 scheduled, 4 on-demand, 2 alert-triggered)
    - Each job: Description, schedule, inputs, outputs, error handling
    - Job 1-5: Scheduled jobs (health check, portfolio report, cost report, performance analysis, GPT-4 self-analysis)
    - Job 6-9: On-demand jobs (train model, backtest, optimize, bulk analysis)
    - Job 10-11: Alert-triggered jobs (adaptive retrain, emergency disable)
  - Section 4: Notification system (4 channels)
  - Section 5: Data storage (5 MongoDB collections)
  - Section 6: Error handling and retry logic
  - Section 7: Monitoring with Flower
  - Section 8: Security (task authentication, rate limiting)
  - Section 9: Test cases (TC-ASYNC-001 to TC-ASYNC-030)
  - **WHY critical**: Changes trading execution flow, costs $1/month in GPT-4 → saves $430/year in compute
  - **Finance impact**: Auto-disables failing strategies, prevents losses

#### Afternoon (4-5 hours):
- [ ] **Task 1.2**: Update `FR-RISK.md` (+2,000 lines)
  - Add FR-RISK-007: Trailing Stop Configuration
    - Settings: `trailing_stop_enabled`, `trailing_stop_pct`, `trailing_activation_pct`
    - Defaults: 3% trail, activate after 5% profit
    - Per-symbol overrides
  - Add FR-RISK-008: Trailing Stop Execution Logic
    - Algorithm: Track `highest_price_achieved`, update stop loss dynamically
    - Long trades: Stop = highest_price * (1 - trailing_pct/100)
    - Short trades: Stop = highest_price * (1 + trailing_pct/100)
    - Activation threshold: Only trail after profit > activation_pct
    - Edge cases: Never move stop loss in unfavorable direction
  - Add FR-RISK-009: Trailing Stop State Management
    - Fields: `trailing_stop_active`, `highest_price_achieved`
    - State transitions: Inactive → Active (on profit threshold)
    - Persistence: Save state in `paper_trades` collection
  - Add FR-RISK-010: Trailing Stop Monitoring and Logging
    - Log activation events
    - Log stop updates
    - Log stop-loss triggers
  - Update existing risk rules to integrate trailing stops
  - **WHY critical**: Trailing stops lock in profits, prevent premature exits
  - **Finance impact**: +20-30% profit capture (based on implementation plan estimates)

**DAY 2 (8-10 hours)**:

#### Morning (4-5 hours):
- [ ] **Task 1.3**: Update `FR-STRATEGIES.md` (+2,500 lines)
  - Add FR-STRATEGIES-009: Stochastic Strategy
    - Description: Stochastic Oscillator (%K, %D) for overbought/oversold
    - Parameters: k_period=14, d_period=3, oversold=20, overbought=80
    - Signal generation: Crossovers, extreme zones, divergence
    - Timeframe support: All timeframes (1m, 5m, 15m, 1h, 4h, 1d)
  - Add FR-STRATEGIES-010: Multi-Timeframe Analysis
    - Default timeframe: 15m (optimized for crypto day trading)
    - Configurable via UI: 1m, 5m, 15m, 1h, 4h, 1d
    - Data resolution setting per symbol
  - Add FR-STRATEGIES-011: Strategy Performance Metrics
    - Win rate tracking
    - Sharpe ratio calculation
    - Average profit/loss
    - Consecutive loss tracking
  - Add FR-STRATEGIES-012: Strategy Auto-Disable
    - Trigger: Daily loss >10% OR consecutive losses >10
    - Action: Disable strategy, send critical alert
    - Re-enable: Manual only (admin intervention)
  - Update FR-STRATEGIES-005: Strategy Engine to include 5th strategy
  - Update all strategy examples with multi-timeframe support
  - **WHY critical**: 5th strategy changes trading decisions, affects win rate
  - **Finance impact**: 65% combined win rate (from docs), new strategy contributes

#### Afternoon (4-5 hours):
- [ ] **Task 1.4**: Update `DB-SCHEMA.md` (+3,000 lines)
  - Add `gpt4_analysis` collection:
    - Fields: analysis_id, timestamp, strategy, metrics, recommendation, reasoning, confidence
    - Indexes: timestamp, strategy, recommendation
    - Purpose: Audit trail for AI retraining decisions
  - Add `performance_metrics` collection:
    - Fields: metric_id, date, win_rate, sharpe_ratio, avg_profit, total_trades
    - Indexes: date, win_rate
    - Purpose: Daily performance tracking for GPT-4 analysis
  - Add `model_accuracy` collection:
    - Fields: model_id, timestamp, accuracy, confidence, predictions_count
    - Indexes: timestamp, model_id
    - Purpose: Track ML model performance degradation
  - Add `api_costs` collection:
    - Fields: date, openai_requests, openai_tokens, openai_cost_usd, threshold_warnings
    - Indexes: date
    - Purpose: GPT-4 API cost tracking and alerts
  - Add `retrain_history` collection:
    - Fields: retrain_id, timestamp, trigger, old_accuracy, new_accuracy, duration, status
    - Indexes: timestamp, trigger, status
    - Purpose: Audit trail for model retraining events
  - Update `paper_trades` collection:
    - Add fields: `trailing_stop_active`, `highest_price_achieved`, `data_resolution`
  - Update `paper_trading_settings` collection:
    - Add risk fields: `trailing_stop_enabled`, `trailing_stop_pct`, `trailing_activation_pct`
  - **WHY critical**: Database schema changes affect all async jobs and trading logic
  - **Finance impact**: Audit trail for all AI decisions that affect trading

---

### Phase 2: HIGH Priority Features (Days 3-4, 12-16 hours)

**DAY 3 (6-8 hours)**:

#### Morning (3-4 hours):
- [ ] **Task 2.1**: Update `FR-PAPER-TRADING.md` (+1,500 lines)
  - Add FR-PAPER-011: Instant Warmup System
    - Pre-load historical data (100-500 candles) at startup
    - Warm strategies immediately (no 2-hour wait)
    - Configurable warmup periods per strategy
  - Add FR-PAPER-012: External AI Signal Processing
    - Accept signals from frontend via API
    - Process GPT-4 recommendations from UI
    - Validation and risk checks before execution
  - Add FR-PAPER-013: Data Resolution Selector
    - UI component for timeframe selection
    - Per-symbol settings
    - Real-time updates on change
  - Update FR-PAPER-003: Simulated Execution with trailing stops
  - **Finance impact**: Instant warmup = faster testing, external signals = user control

#### Afternoon (3-4 hours):
- [ ] **Task 2.2**: Update `API-RUST-CORE.md` (+1,500 lines)
  - Add endpoints:
    - `POST /api/paper-trading/external-signal` - Process external AI signals
    - `GET /api/paper-trading/settings` - Get current settings (with trailing stop)
    - `PUT /api/paper-trading/settings` - Update settings (with trailing stop)
    - `GET /api/paper-trading/trailing-stops` - Get trailing stop status for all trades
  - Update existing endpoints with new fields
  - Add error responses for new validation rules
  - **Developer impact**: API changes require frontend updates

**DAY 4 (6-8 hours)**:

#### Morning (3-4 hours):
- [ ] **Task 2.3**: Update `API-PYTHON-AI.md` (+1,200 lines)
  - Add async task endpoints:
    - `POST /api/tasks/train` - Trigger model training
    - `POST /api/tasks/backtest` - Trigger backtesting
    - `POST /api/tasks/optimize` - Trigger optimization
    - `POST /api/tasks/bulk-analysis` - Trigger bulk analysis
    - `GET /api/tasks/{task_id}` - Get task status
    - `GET /api/tasks` - List all tasks
  - Add monitoring endpoints:
    - `GET /api/health` - Health check (includes async workers)
    - `GET /api/metrics/performance` - Get performance metrics
    - `GET /api/metrics/costs` - Get API costs
  - Update existing endpoints with async response patterns
  - **Developer impact**: Async API requires new frontend patterns

#### Afternoon (3-4 hours):
- [ ] **Task 2.4**: Update `FR-DASHBOARD.md` (+1,000 lines)
  - Add FR-DASHBOARD-011: Per-Symbol Settings UI
    - Timeframe selector dropdown
    - Risk settings per symbol
    - Real-time preview
  - Add FR-DASHBOARD-012: Trailing Stop Visualization
    - Chart overlay showing trailing stop levels
    - Historical trail path
  - Add FR-DASHBOARD-013: Async Task Monitoring
    - Task queue status
    - Running tasks display
    - Task history and logs
  - **User impact**: New UI features require user documentation

---

### Phase 3: MEDIUM Priority (Days 5-6, 8-12 hours)

**DAY 5 (4-6 hours)**:

#### Morning (2-3 hours):
- [ ] **Task 3.1**: Create `TC-E2E.md` (800-1,000 lines)
  - Test cases for critical flows:
    - TC-E2E-001: User authentication flow
    - TC-E2E-002: Paper trading execution flow
    - TC-E2E-003: Settings update flow
    - TC-E2E-004: Dashboard real-time updates
    - TC-E2E-005: AI signal processing flow
  - Visual regression test cases:
    - TC-VR-001: Landing page visual consistency
    - TC-VR-002: Login page visual consistency
    - TC-VR-003: Dashboard page visual consistency
    - TC-VR-004: Paper trading page visual consistency
  - Test data requirements
  - CI/CD integration
  - **Quality impact**: E2E tests prevent regressions

#### Afternoon (2-3 hours):
- [ ] **Task 3.2**: Update `TEST-PLAN.md` (+800 lines)
  - Add E2E testing strategy section
  - Add visual regression testing section
  - Update test coverage targets (include E2E)
  - Add Playwright setup instructions
  - **QA impact**: Testing strategy changes

**DAY 6 (4-6 hours)**:

#### Morning (2-3 hours):
- [ ] **Task 3.3**: Update `UI-COMPONENTS.md` (+600 lines)
  - Add new components:
    - TimeframeSelector component
    - TrailingStopVisualization component
    - AsyncTaskMonitor component
    - PerSymbolSettings component
  - Update existing components with new props
  - **Developer impact**: Component library updates

#### Afternoon (2-3 hours):
- [ ] **Task 3.4**: Update `NFR-PERFORMANCE.md` (+400 lines)
  - Add async job performance targets:
    - Health check: <5s
    - Portfolio report: <10s
    - Performance analysis: <30s
    - GPT-4 analysis: <60s (API dependent)
    - Model training: <10 minutes
  - Add queue latency targets:
    - Task enqueue: <100ms
    - Task pickup: <1s
    - Result retrieval: <500ms
  - **Operations impact**: Performance monitoring

---

### Phase 4: LOW Priority (Days 7-8, 6-8 hours)

**DAY 7 (3-4 hours)**:

#### Morning (2 hours):
- [ ] **Task 4.1**: Create `INFRA-ASYNC-TASKS.md` (600-800 lines)
  - RabbitMQ deployment configuration
  - Celery worker scaling strategy (4 workers default)
  - Redis persistence configuration
  - Flower monitoring setup (port 5555)
  - Health checks for all components
  - **DevOps impact**: Infrastructure documentation

#### Afternoon (1-2 hours):
- [ ] **Task 4.2**: Update `INFRA-DOCKER.md` (+800 lines)
  - Add RabbitMQ service configuration
  - Add Redis service configuration
  - Add Celery worker service
  - Add Celery beat scheduler
  - Add Flower service
  - Update docker-compose.yml documentation
  - **DevOps impact**: Deployment changes

**DAY 8 (3-4 hours)**:

#### Morning (2 hours):
- [ ] **Task 4.3**: Update `CICD-PIPELINE.md` (+300 lines)
  - Add E2E test stage (Playwright)
  - Add visual regression test stage
  - Update security checks (bun vs npm audit)
  - Add async task health checks
  - **CI/CD impact**: Pipeline changes

#### Afternoon (1-2 hours):
- [ ] **Task 4.4**: Update `OPS-MANUAL.md` (+500 lines)
  - Add async tasks troubleshooting section
  - Add RabbitMQ operations (queue management, purging)
  - Add Celery worker management (scaling, restarting)
  - Add monitoring with Flower
  - **Operations impact**: Day-to-day operations

---

### Phase 5: Traceability & Finalization (Day 9, 4-6 hours)

**DAY 9 (4-6 hours)**:

#### Morning (2-3 hours):
- [ ] **Task 5.1**: Update `TRACEABILITY_MATRIX.md` (+150 rows)
  - Add new requirement IDs:
    - FR-ASYNC-001 to FR-ASYNC-015 (async tasks)
    - FR-RISK-007 to FR-RISK-010 (trailing stops)
    - FR-STRATEGIES-009 to FR-STRATEGIES-012 (stochastic, multi-TF)
    - FR-PAPER-011 to FR-PAPER-013 (warmup, external signals, data resolution)
    - TC-E2E-001 to TC-E2E-010 (E2E tests)
  - Link requirements to:
    - Design docs
    - Implementation files
    - Test cases
  - Update coverage percentages
  - **Quality impact**: 100% traceability required

#### Afternoon (2-3 hours):
- [ ] **Task 5.2**: Add @spec tags to code (30 new tags)
  - Files to tag:
    - `python-ai-service/tasks/monitoring.py` (4 tags)
    - `python-ai-service/tasks/ai_improvement.py` (3 tags)
    - `python-ai-service/tasks/ml_tasks.py` (2 tags)
    - `python-ai-service/tasks/backtest_tasks.py` (2 tags)
    - `python-ai-service/utils/notifications.py` (1 tag)
    - `python-ai-service/utils/data_storage.py` (1 tag)
    - `rust-core-engine/src/paper_trading/trade.rs` (trailing stop methods, 3 tags)
    - `rust-core-engine/src/paper_trading/engine.rs` (warmup, external signals, 4 tags)
    - `rust-core-engine/src/strategies/stochastic_strategy.rs` (already tagged ✓)
    - `nextjs-ui-dashboard/e2e/*.spec.ts` (5 tags)
    - `nextjs-ui-dashboard/src/components/settings/PerSymbolSettings.tsx` (1 tag)
  - Validate with `python3 scripts/validate-spec-tags.py`
  - **Developer impact**: Code-to-spec linking

- [ ] **Task 5.3**: Update `TASK_TRACKER.md` (+50 rows)
  - Mark completed features
  - Add new feature tracking
  - Update completion percentages
  - **Project management impact**: Progress tracking

- [ ] **Task 5.4**: Final validation and review
  - Run spec validation script
  - Check for broken links
  - Verify all @spec tags match requirements
  - Generate compliance report
  - **Quality gate**: Must pass before marking complete

---

## 4. QUALITY GATES

Each phase MUST pass these gates before proceeding:

### Gate 1: Content Quality (CRITICAL)
- [ ] All new specs follow template structure (`_TEMPLATE_FR.md`)
- [ ] All specs have detailed requirement IDs (FR-XXX-YYY)
- [ ] All finance-critical features have:
  - WHY section (business justification)
  - RISK section (what could go wrong)
  - ERROR HANDLING section (edge cases)
  - EXAMPLES section (code snippets)
- [ ] All specs include acceptance criteria
- [ ] All specs include test case references

### Gate 2: Traceability (MANDATORY)
- [ ] All new requirements in TRACEABILITY_MATRIX.md
- [ ] All requirements linked to design docs
- [ ] All requirements linked to implementation files
- [ ] All requirements linked to test cases
- [ ] All @spec tags in code match requirements
- [ ] Validation script passes: `python3 scripts/validate-spec-tags.py`

### Gate 3: Accuracy (CRITICAL)
- [ ] All code examples tested and working
- [ ] All API endpoints verified against actual implementation
- [ ] All database schemas match actual MongoDB collections
- [ ] All configuration options match actual settings
- [ ] NO aspirational features (only document what exists)

### Gate 4: Finance Safety (EXTREME CRITICAL)
- [ ] All trading logic changes documented with examples
- [ ] All risk management changes include edge cases
- [ ] All profit/loss calculations have formulas
- [ ] All error scenarios documented with recovery steps
- [ ] All money-affecting features peer reviewed

### Gate 5: Completeness (HIGH)
- [ ] All 7 new features documented
- [ ] All 11 outdated specs updated
- [ ] All 5 infrastructure components documented
- [ ] All test cases created
- [ ] All API changes documented

---

## 5. ESTIMATED EFFORT

### By Phase:
| Phase | Days | Hours | Priority | Notes |
|-------|------|-------|----------|-------|
| Phase 1: Critical Features | 2 | 16-20 | EXTREME HIGH | Async tasks, trailing stops, stochastic, DB |
| Phase 2: High Priority | 2 | 12-16 | HIGH | Warmup, APIs, dashboard |
| Phase 3: Medium Priority | 2 | 8-12 | MEDIUM | E2E tests, UI, performance |
| Phase 4: Low Priority | 2 | 6-8 | LOW | Infrastructure, operations |
| Phase 5: Traceability | 1 | 4-6 | MANDATORY | Validation and linking |
| **TOTAL** | **9 days** | **46-62 hours** | | **~6-8 full work days** |

### By Role:
| Role | Tasks | Hours | Notes |
|------|-------|-------|-------|
| **Spec Writer** | Create/update all specs | 40-50 | Main effort |
| **Code Reviewer** | Verify accuracy, add @spec tags | 4-6 | Validate against code |
| **QA Engineer** | Create test cases, verify gates | 2-4 | Quality assurance |
| **DevOps** | Infrastructure specs | 2-4 | Docker, CI/CD |
| **Total** | | **48-64 hours** | |

### By Deliverable:
| Deliverable | Lines | Hours | Priority |
|-------------|-------|-------|----------|
| FR-ASYNC-TASKS.md (NEW) | 1,800 | 6-8 | CRITICAL |
| FR-RISK.md (UPDATE) | +2,000 | 5-6 | CRITICAL |
| FR-STRATEGIES.md (UPDATE) | +2,500 | 6-7 | CRITICAL |
| DB-SCHEMA.md (UPDATE) | +3,000 | 6-8 | CRITICAL |
| FR-PAPER-TRADING.md (UPDATE) | +1,500 | 4-5 | HIGH |
| API-RUST-CORE.md (UPDATE) | +1,500 | 4-5 | HIGH |
| API-PYTHON-AI.md (UPDATE) | +1,200 | 3-4 | HIGH |
| TC-E2E.md (NEW) | 1,000 | 3-4 | MEDIUM |
| Other updates (11 files) | +4,500 | 8-10 | MEDIUM/LOW |
| Traceability updates | 200 | 4-6 | MANDATORY |
| **TOTAL** | **~21,000 lines** | **49-67 hours** | |

---

## 6. SUCCESS CRITERIA

### Completion Criteria (100% required):
- [x] ✅ All 7 new features documented with complete specs
- [ ] ✅ All 11 outdated specs updated to match current code
- [ ] ✅ All 5 infrastructure components documented
- [ ] ✅ 4 new spec files created
- [ ] ✅ 15 existing spec files updated
- [ ] ✅ TRACEABILITY_MATRIX.md updated with 150+ new rows
- [ ] ✅ 30 new @spec tags added to code
- [ ] ✅ All quality gates passed
- [ ] ✅ Validation script passes (100% compliance)

### Quality Criteria (Must maintain):
- [ ] ✅ Zero spec-code mismatches
- [ ] ✅ 100% traceability (requirements ↔ design ↔ code ↔ tests)
- [ ] ✅ All finance-critical features have extreme detail
- [ ] ✅ All error scenarios documented
- [ ] ✅ All API changes backward compatible (or breaking changes flagged)

### Business Criteria:
- [ ] ✅ Developers can implement new features from specs alone
- [ ] ✅ QA can create test cases from specs alone
- [ ] ✅ Users can understand system behavior from specs
- [ ] ✅ Auditors can verify compliance from specs
- [ ] ✅ New team members can onboard from specs

---

## 7. RISKS & MITIGATIONS

### Risk 1: Spec-Code Drift During Update
**Probability**: MEDIUM
**Impact**: HIGH
**Mitigation**:
- Work in spec-update branch
- Freeze code changes during spec update (communicate to team)
- Daily sync with code reviews
- Use git blame to verify implementation dates

### Risk 2: Missing Features Not Documented
**Probability**: MEDIUM
**Impact**: MEDIUM
**Mitigation**:
- Review all 351 commits manually
- Grep codebase for TODO/FIXME/NOTE comments
- Interview developers for undocumented changes
- Use git diff --stat to find large file changes

### Risk 3: Finance-Critical Features Under-Documented
**Probability**: LOW
**Impact**: EXTREME HIGH
**Mitigation**:
- Peer review all trading/risk specs (2 reviewers minimum)
- Include worked examples with real numbers
- Document edge cases exhaustively
- Add "WHAT COULD GO WRONG" sections

### Risk 4: Traceability Validation Fails
**Probability**: MEDIUM
**Impact**: HIGH
**Mitigation**:
- Run validation script after each spec update
- Fix broken links immediately
- Maintain spreadsheet of requirements (backup)
- Automated CI check for spec compliance

### Risk 5: Time Overrun (>9 days)
**Probability**: MEDIUM
**Impact**: MEDIUM
**Mitigation**:
- Daily progress tracking
- Time-box each task strictly
- Defer LOW priority items if needed
- Spread work across 2-3 weeks (avoid burnout)

---

## 8. DEPENDENCIES & CONSTRAINTS

### External Dependencies:
- ✅ Code is stable (no major features in flight)
- ✅ Git history is complete (all commits preserved)
- ✅ Implementation reports exist (ASYNC_TASKS_README.md, etc.)
- ⚠️ Need developer interviews for undocumented changes

### Internal Dependencies:
- Spec update BLOCKS new feature development (communicate to team)
- Must complete before next release (v2.2.0)
- Must complete before external audit (if planned)

### Constraints:
- MUST follow existing spec structure (no major redesign)
- MUST use existing templates (`_TEMPLATE_FR.md`)
- MUST maintain backward compatibility with v1.0 specs
- MUST not break existing @spec tags (60+ existing tags)

---

## 9. COMMUNICATION PLAN

### Stakeholders:
| Stakeholder | Updates Needed | Frequency | Method |
|-------------|----------------|-----------|--------|
| Development Team | Spec freeze, completion status | Daily | Slack |
| QA Team | New test cases available | After Phase 3 | Email |
| DevOps Team | Infrastructure specs ready | After Phase 4 | Meeting |
| Project Manager | Progress report | Daily | Dashboard |
| Product Owner | Feature documentation complete | After Phase 1 | Demo |

### Milestones to Communicate:
1. **Day 2**: Critical specs complete (async tasks, trailing stops, stochastic)
2. **Day 4**: High priority complete (APIs, warmup, dashboard)
3. **Day 6**: Medium priority complete (E2E tests, UI)
4. **Day 8**: Low priority complete (infrastructure, operations)
5. **Day 9**: Final validation passed (100% compliant)

---

## 10. POST-UPDATE ACTIONS

After spec update completion:

### Immediate (Day 10):
- [ ] Announce spec update completion to team
- [ ] Publish updated specs to internal wiki
- [ ] Update README.md with new spec version (2.2.0)
- [ ] Tag specs repository: `git tag -a specs-v2.2.0 -m "Complete spec update after 1.5 months"`
- [ ] Generate compliance report and share with stakeholders

### Short-term (Week 2):
- [ ] Train developers on new specs (1-hour session)
- [ ] Train QA on new test cases (30-min session)
- [ ] Update CI/CD to validate @spec tags on every commit
- [ ] Create spec maintenance schedule (monthly reviews)

### Long-term (Month 1):
- [ ] Establish spec-first culture (no code without spec)
- [ ] Create automated spec generation tools (if feasible)
- [ ] Set up spec drift alerts (monthly git diff checks)
- [ ] Plan next spec audit (3 months)

---

## 11. LESSONS LEARNED (Post-Mortem Items)

Document these after completion:

### What Went Well:
- TBD after completion

### What Went Wrong:
- 1.5 months gap allowed 7 major features without specs (VIOLATION)
- 351 commits = too much to catch up on
- TBD after completion

### Process Improvements:
- [ ] Implement bi-weekly spec reviews (mandatory)
- [ ] Add pre-commit hook to check @spec tags
- [ ] Require spec update in PR template (GitHub)
- [ ] Create spec update checklist for developers
- [ ] Automate spec drift detection (CI alert)

---

## 12. APPENDIX

### A. File Change Summary

**New Files (4)**:
1. `specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md` (1,800 lines)
2. `specs/03-testing/3.2-test-cases/TC-E2E.md` (1,000 lines)
3. `specs/03-testing/3.5-security/SEC-TEST-E2E.md` (500 lines)
4. `specs/04-deployment/4.1-infrastructure/INFRA-ASYNC-TASKS.md` (800 lines)

**Updated Files (15)**:
1. `specs/01-requirements/1.1-functional-requirements/FR-RISK.md` (+2,000 lines)
2. `specs/01-requirements/1.1-functional-requirements/FR-PAPER-TRADING.md` (+1,500 lines)
3. `specs/01-requirements/1.1-functional-requirements/FR-STRATEGIES.md` (+2,500 lines)
4. `specs/01-requirements/1.1-functional-requirements/FR-AI.md` (+2,000 lines)
5. `specs/01-requirements/1.1-functional-requirements/FR-DASHBOARD.md` (+1,000 lines)
6. `specs/01-requirements/1.1-functional-requirements/FR-MARKET-DATA.md` (+500 lines)
7. `specs/02-design/2.2-database/DB-SCHEMA.md` (+3,000 lines)
8. `specs/02-design/2.3-api/API-RUST-CORE.md` (+1,500 lines)
9. `specs/02-design/2.3-api/API-PYTHON-AI.md` (+1,200 lines)
10. `specs/02-design/2.4-ui-ux/UI-COMPONENTS.md` (+600 lines)
11. `specs/03-testing/3.1-test-plan/TEST-PLAN.md` (+800 lines)
12. `specs/01-requirements/1.2-non-functional-requirements/NFR-PERFORMANCE.md` (+400 lines)
13. `specs/04-deployment/4.1-infrastructure/INFRA-DOCKER.md` (+800 lines)
14. `specs/04-deployment/4.2-cicd/CICD-PIPELINE.md` (+300 lines)
15. `specs/05-operations/5.1-operations-manual/OPS-MANUAL.md` (+500 lines)

**Traceability Updates (3)**:
1. `specs/TRACEABILITY_MATRIX.md` (+150 rows)
2. `specs/TASK_TRACKER.md` (+50 rows)
3. Code @spec tags (30 new tags in 15 files)

**Total**: 19 files, ~21,000 new lines

---

### B. Requirement ID Ranges

**New Requirement IDs to Assign**:

**FR-ASYNC-TASKS**:
- FR-ASYNC-001 to FR-ASYNC-015 (15 requirements)

**FR-RISK** (additions):
- FR-RISK-007: Trailing Stop Configuration
- FR-RISK-008: Trailing Stop Execution Logic
- FR-RISK-009: Trailing Stop State Management
- FR-RISK-010: Trailing Stop Monitoring

**FR-STRATEGIES** (additions):
- FR-STRATEGIES-009: Stochastic Strategy
- FR-STRATEGIES-010: Multi-Timeframe Analysis
- FR-STRATEGIES-011: Strategy Performance Metrics
- FR-STRATEGIES-012: Strategy Auto-Disable

**FR-PAPER-TRADING** (additions):
- FR-PAPER-011: Instant Warmup System
- FR-PAPER-012: External AI Signal Processing
- FR-PAPER-013: Data Resolution Selector

**FR-DASHBOARD** (additions):
- FR-DASHBOARD-011: Per-Symbol Settings UI
- FR-DASHBOARD-012: Trailing Stop Visualization
- FR-DASHBOARD-013: Async Task Monitoring

**TC-E2E** (new test cases):
- TC-E2E-001 to TC-E2E-010 (10 test cases)

**TC-VR** (visual regression):
- TC-VR-001 to TC-VR-004 (4 test cases)

**Total New IDs**: ~50 requirement IDs

---

### C. Git Commit Analysis Summary

**Total Commits Since Oct 11**: 351 commits

**Major Feature Commits** (grep "feat"):
1. Async tasks system (RabbitMQ + Celery) - 8dd7a25, 2ce29ac
2. Stochastic strategy (5th strategy) - b0829ba
3. Instant warmup system - 90a3739, 31f052d
4. Trailing stop loss - (embedded in paper trading commits)
5. E2E tests (Playwright) - 74b7964
6. Visual regression tests - 74b7964, b6d5dc6
7. Multi-timeframe optimization (15m) - 19ec766

**Bug Fix Commits** (grep "fix"):
- 100+ bug fixes across frontend, backend, tests

**Documentation Commits** (grep "docs"):
- 10+ documentation updates (CLAUDE.md, feature docs)

**Test Commits** (grep "test"):
- 20+ test improvements (coverage boost, mutation testing)

**CI/CD Commits** (grep "ci"):
- 10+ CI/CD improvements (security checks, E2E in CI)

---

### D. Code Files Needing @spec Tags

**Python (13 files, 13 tags needed)**:
1. `python-ai-service/tasks/monitoring.py` (4 tags: health check, portfolio report, cost report, performance)
2. `python-ai-service/tasks/ai_improvement.py` (3 tags: GPT-4 analysis, adaptive retrain, emergency disable)
3. `python-ai-service/tasks/ml_tasks.py` (2 tags: train model, bulk analysis)
4. `python-ai-service/tasks/backtest_tasks.py` (2 tags: backtest, optimize)
5. `python-ai-service/utils/notifications.py` (1 tag: notification system)
6. `python-ai-service/utils/data_storage.py` (1 tag: MongoDB storage)

**Rust (5 files, 10 tags needed)**:
1. `rust-core-engine/src/paper_trading/trade.rs` (3 tags: trailing stop methods)
2. `rust-core-engine/src/paper_trading/engine.rs` (4 tags: warmup, external signals, trailing stop execution)
3. `rust-core-engine/src/paper_trading/settings.rs` (1 tag: trailing stop settings)
4. `rust-core-engine/src/strategies/stochastic_strategy.rs` (1 tag: already exists ✓)
5. `rust-core-engine/src/binance/mod.rs` (1 tag: instant warmup data loading)

**TypeScript (7 files, 7 tags needed)**:
1. `nextjs-ui-dashboard/e2e/auth.spec.ts` (1 tag)
2. `nextjs-ui-dashboard/e2e/settings.spec.ts` (1 tag)
3. `nextjs-ui-dashboard/e2e/paper-trading.spec.ts` (1 tag)
4. `nextjs-ui-dashboard/e2e/critical-flows.spec.ts` (1 tag)
5. `nextjs-ui-dashboard/e2e/visual-regression.spec.ts` (1 tag)
6. `nextjs-ui-dashboard/src/components/settings/PerSymbolSettings.tsx` (1 tag)
7. `nextjs-ui-dashboard/src/hooks/usePaperTrading.ts` (1 tag: external signals)

**Total**: 25 files, 30 new @spec tags

---

### E. Validation Checklist

Before marking specs complete, verify:

**Content Validation**:
- [ ] All specs follow template structure
- [ ] All requirement IDs are unique
- [ ] All cross-references are valid (no broken links)
- [ ] All code examples are tested
- [ ] All formulas are mathematically correct
- [ ] All API endpoints match implementation
- [ ] All database schemas match MongoDB

**Traceability Validation**:
- [ ] All requirements in TRACEABILITY_MATRIX.md
- [ ] All requirements linked to design
- [ ] All requirements linked to code
- [ ] All requirements linked to tests
- [ ] All @spec tags in code match requirements
- [ ] Validation script passes: `python3 scripts/validate-spec-tags.py`

**Completeness Validation**:
- [ ] All 7 new features documented
- [ ] All 11 outdated specs updated
- [ ] All 5 infrastructure components documented
- [ ] All API changes documented
- [ ] All DB changes documented
- [ ] All UI changes documented
- [ ] All test cases created

**Finance Safety Validation**:
- [ ] All trading logic has examples
- [ ] All risk calculations have formulas
- [ ] All error scenarios documented
- [ ] All edge cases covered
- [ ] All money-affecting features peer reviewed

---

## 13. UNRESOLVED QUESTIONS

**To Clarify Before Starting**:

1. **Infrastructure Freeze**: Can we freeze code commits during spec update (9 days)?
   - Suggested answer: Yes for major features, no for bug fixes
   - Action: Communicate to team, use feature branch protection

2. **Spec Version Numbering**: Bump to v2.1 or v2.2?
   - Current: v2.0 (Oct 11, 2025)
   - Suggested: v2.2 (major updates to 11 specs + 4 new specs)
   - Action: Confirm with project manager

3. **Backward Compatibility**: Do we need to maintain v1.0 spec format?
   - v1.0 specs: API_SPEC.md, DATA_MODELS.md, BUSINESS_RULES.md, INTEGRATION_SPEC.md
   - v2.0 specs: Hierarchical structure (01-requirements/, 02-design/, etc.)
   - Suggested: Keep both, update both
   - Action: Confirm maintenance strategy

4. **External Review**: Do we need external audit/review of specs?
   - Finance project = high stakes
   - Suggested: Yes, at least for CRITICAL specs (FR-ASYNC, FR-RISK, FR-STRATEGIES)
   - Action: Identify reviewers (senior dev, trading expert)

5. **Spec Automation**: Should we build spec generation tools?
   - Current: Manual spec writing (time-consuming)
   - Future: Auto-generate from code comments + @spec tags
   - Suggested: Phase 2 effort (after this update)
   - Action: Add to backlog

6. **Internationalization**: Do specs need non-English versions?
   - Current: English only
   - Potential: Vietnamese version (team location)
   - Action: Confirm with stakeholders

---

## 14. SUMMARY

**This plan provides**:
- Complete gap analysis (7 new features, 11 outdated specs)
- Prioritized update strategy (CRITICAL → HIGH → MEDIUM → LOW)
- Detailed phase breakdown (9 days, 46-62 hours)
- Quality gates (finance safety, traceability, accuracy)
- Success criteria (100% compliance)
- Risk mitigation strategies
- Communication plan
- Post-update actions

**Next Steps**:
1. Review and approve this plan
2. Clarify unresolved questions (#13)
3. Communicate spec freeze to team
4. Start Phase 1 (Day 1-2): CRITICAL features
5. Daily progress updates
6. Quality gate checks after each phase
7. Final validation and compliance report (Day 9)

**Expected Outcome**:
- 100% spec-code traceability
- All features documented with extreme detail
- Finance-critical features peer reviewed
- Developers can implement from specs alone
- QA can test from specs alone
- Auditors can verify compliance
- Zero spec-code mismatches

**Confidence Level**: HIGH (based on existing implementation reports and code analysis)

---

**END OF PLAN**

**Plan saved to**: `docs/plans/251122-specs-update-comprehensive-plan.md`
**Next action**: Review plan, clarify questions, get approval, START PHASE 1
