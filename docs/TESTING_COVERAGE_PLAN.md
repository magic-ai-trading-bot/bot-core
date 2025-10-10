# Test Coverage Improvement Plan - 90%+ Target

## Executive Summary

**Mission**: Achieve 90%+ test coverage across all services in the bot-core trading system.

**Current Status** (Estimated):
- ✅ **Python AI Service**: ~94% coverage (17 test files, 385+ tests) - **EXCELLENT**
- ⚠️ **Rust Core Engine**: ~60-70% coverage (13 test files) - **NEEDS IMPROVEMENT**
- ⚠️ **Frontend (Next.js)**: ~80-85% coverage (27 test files, 565+ tests) - **GOOD, CAN IMPROVE**

**Target Status**:
- 🎯 **Python**: Maintain 94%+ coverage
- 🎯 **Rust**: Improve to 90%+ coverage
- 🎯 **Frontend**: Improve to 90%+ coverage

---

## 1. Rust Core Engine - Coverage Improvement Strategy

### 1.1 Current Test Files
```
✅ tests/test_market_data.rs
✅ tests/test_ai.rs
✅ tests/test_config.rs
✅ tests/test_binance_client.rs
✅ tests/test_auth.rs
✅ tests/test_paper_trading.rs
✅ tests/test_storage.rs (2468 lines, 150+ comprehensive tests)
✅ tests/test_strategies.rs
✅ tests/test_trading.rs
✅ tests/test_websocket.rs
✅ src/strategies/tests.rs
✅ src/error.rs (contains 760+ lines of tests)
```

### 1.2 Coverage Gaps Identified

#### High Priority (Core Business Logic):
1. **Trading Module** (`src/trading/`)
   - Position management edge cases
   - Risk management calculations
   - Order execution error paths
   - Stop-loss/take-profit trigger logic

2. **Strategy Implementations** (`src/strategies/`)
   - RSI strategy edge cases
   - MACD crossover scenarios
   - Bollinger Bands breakout conditions
   - Volume strategy validations
   - Strategy engine orchestration

3. **Paper Trading Engine** (`src/paper_trading/`)
   - Portfolio rebalancing logic
   - Leverage calculations
   - Fee computations
   - Margin call scenarios
   - PnL calculations

4. **Market Data Processing** (`src/market_data/`)
   - Data validation
   - Timeframe aggregation
   - Indicator calculations
   - Missing data handling

#### Medium Priority:
5. **WebSocket Handling** (`src/websocket/`)
   - Reconnection logic
   - Message parsing errors
   - Stream multiplexing

6. **Binance Integration** (`src/binance/`)
   - API error handling
   - Rate limit responses
   - Order status updates

### 1.3 Implementation Plan

#### Phase 1: Core Business Logic Tests (Week 1)
```bash
# 1. Create comprehensive strategy tests
cat > rust-core-engine/tests/test_strategies_comprehensive.rs << 'EOF'
// RSI Strategy Tests
#[tokio::test]
async fn test_rsi_strategy_oversold_condition() { }

#[tokio::test]
async fn test_rsi_strategy_overbought_condition() { }

#[tokio::test]
async fn test_rsi_strategy_with_insufficient_data() { }

// MACD Strategy Tests
#[tokio::test]
async fn test_macd_bullish_crossover() { }

#[tokio::test]
async fn test_macd_bearish_crossover() { }

// Bollinger Bands Tests
#[tokio::test]
async fn test_bollinger_breakout_upper_band() { }

#[tokio::test]
async fn test_bollinger_breakout_lower_band() { }

// Volume Strategy Tests
#[tokio::test]
async fn test_volume_spike_detection() { }

#[tokio::test]
async fn test_volume_strategy_low_volume() { }
EOF

# 2. Create position management tests
cat > rust-core-engine/tests/test_position_management.rs << 'EOF'
#[tokio::test]
async fn test_open_long_position() { }

#[tokio::test]
async fn test_open_short_position() { }

#[tokio::test]
async fn test_close_position_with_profit() { }

#[tokio::test]
async fn test_close_position_with_loss() { }

#[tokio::test]
async fn test_position_liquidation() { }

#[tokio::test]
async fn test_position_margin_call() { }

#[tokio::test]
async fn test_position_with_max_leverage() { }
EOF

# 3. Create risk management tests
cat > rust-core-engine/tests/test_risk_management.rs << 'EOF'
#[tokio::test]
async fn test_calculate_position_size() { }

#[tokio::test]
async fn test_validate_risk_limits() { }

#[tokio::test]
async fn test_stop_loss_calculation() { }

#[tokio::test]
async fn test_take_profit_calculation() { }

#[tokio::test]
async fn test_risk_reward_ratio() { }

#[tokio::test]
async fn test_max_drawdown_limit() { }

#[tokio::test]
async fn test_portfolio_risk_distribution() { }
EOF
```

#### Phase 2: Integration & Edge Cases (Week 2)
```bash
# 4. Create WebSocket integration tests
cat > rust-core-engine/tests/test_websocket_comprehensive.rs << 'EOF'
#[tokio::test]
async fn test_websocket_connection_establishment() { }

#[tokio::test]
async fn test_websocket_reconnection_on_disconnect() { }

#[tokio::test]
async fn test_websocket_message_parsing() { }

#[tokio::test]
async fn test_websocket_subscription_management() { }

#[tokio::test]
async fn test_websocket_error_handling() { }

#[tokio::test]
async fn test_websocket_heartbeat() { }
EOF

# 5. Create Binance API tests
cat > rust-core-engine/tests/test_binance_comprehensive.rs << 'EOF'
#[tokio::test]
async fn test_binance_order_placement() { }

#[tokio::test]
async fn test_binance_order_cancellation() { }

#[tokio::test]
async fn test_binance_rate_limit_handling() { }

#[tokio::test]
async fn test_binance_api_error_responses() { }

#[tokio::test]
async fn test_binance_signature_generation() { }

#[tokio::test]
async fn test_binance_timestamp_sync() { }
EOF
```

#### Phase 3: Run Coverage Analysis
```bash
# Install and run tarpaulin
cargo install cargo-tarpaulin

# Run comprehensive coverage
cd rust-core-engine
cargo tarpaulin \
  --timeout 300 \
  --skip-clean \
  --out Html \
  --out Xml \
  --output-dir coverage \
  -- --test-threads=1

# Generate summary report
cargo tarpaulin --print-summary
```

---

## 2. Frontend (Next.js) - Coverage Improvement Strategy

### 2.1 Current Test Files (27 files, 565+ tests)
```
✅ ChatBot.test.tsx
✅ Dashboard.test.tsx
✅ TradingCharts.test.tsx
✅ hooks/useAIAnalysis.test.ts
✅ hooks/usePaperTrading.test.ts
✅ hooks/useWebSocket.test.ts
... (24 more test files)
```

### 2.2 Coverage Gaps

#### High Priority:
1. **WebSocket Hooks** - Reconnection logic, error states
2. **Trading Hooks** - Order execution, position management
3. **AI Analysis Hooks** - Signal processing, confidence thresholds
4. **Error Boundaries** - Component crash recovery
5. **API Integration** - Request/response handling

### 2.3 Implementation Plan

#### Phase 1: Add Missing Hook Tests
```typescript
// src/hooks/__tests__/useWebSocket.comprehensive.test.ts
describe('useWebSocket - Comprehensive', () => {
  it('should reconnect on connection loss', async () => {
    // Test reconnection logic
  });

  it('should handle message parsing errors', () => {
    // Test error handling
  });

  it('should queue messages during disconnection', () => {
    // Test message queue
  });

  it('should implement exponential backoff', () => {
    // Test backoff strategy
  });
});
```

#### Phase 2: Add Integration Tests
```typescript
// src/__tests__/integration/trading-flow.test.tsx
describe('Trading Flow Integration', () => {
  it('should complete full trading cycle', async () => {
    // 1. Connect to WebSocket
    // 2. Receive market data
    // 3. Generate AI signal
    // 4. Execute trade
    // 5. Monitor position
    // 6. Close position
  });

  it('should handle trading errors gracefully', async () => {
    // Test error scenarios
  });
});
```

#### Phase 3: Add E2E Tests with Mock Service Worker (MSW)
```typescript
// src/__tests__/e2e/paper-trading.test.tsx
import { setupServer } from 'msw/node';
import { rest } from 'msw';

const server = setupServer(
  rest.post('/api/trades', (req, res, ctx) => {
    return res(ctx.json({ success: true, tradeId: '123' }));
  })
);

describe('Paper Trading E2E', () => {
  beforeAll(() => server.listen());
  afterEach(() => server.resetHandlers());
  afterAll(() => server.close());

  it('should execute paper trade from UI', async () => {
    // Full user flow test
  });
});
```

#### Phase 4: Run Coverage
```bash
cd nextjs-ui-dashboard
npm run test:coverage

# Generate HTML report
npm run test:coverage -- --reporter=html
```

---

## 3. Python AI Service - Maintenance Strategy

### 3.1 Current Status
✅ **94% coverage** - Excellent!

**Test Files**:
```
✅ test_basic.py (2 tests)
✅ test_config.py (23 tests)
✅ test_cors_config.py (4 tests)
✅ test_feature_engineering.py
✅ test_gpt_analyzer.py
✅ test_integration.py (5 integration tests)
✅ test_technical_analyzer.py
✅ test_websocket.py
... (17 test files total, 385+ tests)
```

### 3.2 Gaps to Address

#### Missing Coverage:
1. **Error path tests** - Exception handling
2. **Edge case inputs** - Boundary values
3. **Concurrent requests** - Thread safety
4. **Model failure scenarios** - AI model errors

### 3.3 Maintenance Plan

```bash
# Run coverage analysis
cd python-ai-service
python -m pytest tests/ \
  --cov=. \
  --cov-report=html \
  --cov-report=term-missing \
  --cov-report=xml

# View uncovered lines
python -m pytest --cov=. --cov-report=term-missing | grep -v "100%"

# Add tests for uncovered lines
cat > tests/test_edge_cases.py << 'EOF'
import pytest

def test_model_inference_with_invalid_data():
    """Test AI model with malformed input"""
    pass

def test_concurrent_prediction_requests():
    """Test thread safety of model inference"""
    pass

def test_model_loading_failure():
    """Test graceful degradation when model fails to load"""
    pass
EOF
```

---

## 4. Mutation Testing Strategy

### 4.1 Purpose
Mutation testing validates test quality by introducing bugs and ensuring tests catch them.

### 4.2 Rust Mutation Testing

```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing on critical modules
cd rust-core-engine
cargo mutants \
  --test-tool=cargo test \
  --file src/trading/mod.rs \
  --file src/strategies/mod.rs \
  --file src/paper_trading/mod.rs

# Generate report
cargo mutants --json > mutation-report.json
```

### 4.3 Python Mutation Testing

```bash
# Install mutmut
pip install mutmut

# Run mutation testing
cd python-ai-service
mutmut run --paths-to-mutate=services/,models/

# View results
mutmut results
mutmut html

# Target: 75%+ mutation score
```

### 4.4 Frontend Mutation Testing

```bash
# Install Stryker
npm install -D @stryker-mutator/core @stryker-mutator/typescript-checker

# Configure stryker.conf.json
cat > nextjs-ui-dashboard/stryker.conf.json << 'EOF'
{
  "$schema": "./node_modules/@stryker-mutator/core/schema/stryker-schema.json",
  "mutate": [
    "src/hooks/**/*.ts",
    "src/services/**/*.ts",
    "!src/**/*.test.ts"
  ],
  "testRunner": "vitest",
  "coverageAnalysis": "perTest",
  "thresholds": { "high": 80, "low": 60, "break": 50 }
}
EOF

# Run mutation testing
npx stryker run
```

---

## 5. CI/CD Test Automation

### 5.1 GitHub Actions Workflow

```yaml
# .github/workflows/test-coverage.yml
name: Test Coverage

on:
  push:
    branches: [ main, develop ]
  pull_request:
    branches: [ main ]

jobs:
  test-rust:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
          override: true

      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            rust-core-engine/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}

      - name: Install tarpaulin
        run: cargo install cargo-tarpaulin

      - name: Run tests with coverage
        working-directory: ./rust-core-engine
        run: |
          cargo tarpaulin --timeout 300 --out Xml --output-dir coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./rust-core-engine/coverage/cobertura.xml
          flags: rust
          name: rust-coverage

      - name: Check coverage threshold
        run: |
          COVERAGE=$(cargo tarpaulin --print-summary | grep -oP '\d+\.\d+(?=%)')
          if (( $(echo "$COVERAGE < 90.0" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 90% threshold"
            exit 1
          fi

  test-python:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Cache pip
        uses: actions/cache@v3
        with:
          path: ~/.cache/pip
          key: ${{ runner.os }}-pip-${{ hashFiles('**/requirements.txt') }}

      - name: Install dependencies
        working-directory: ./python-ai-service
        run: |
          pip install -r requirements.txt
          pip install pytest pytest-cov pytest-asyncio

      - name: Run tests with coverage
        working-directory: ./python-ai-service
        run: |
          pytest tests/ --cov=. --cov-report=xml --cov-report=term-missing

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./python-ai-service/coverage.xml
          flags: python
          name: python-coverage

      - name: Check coverage threshold
        run: |
          COVERAGE=$(python -m pytest --cov=. --cov-report=term-missing | grep TOTAL | awk '{print $NF}' | sed 's/%//')
          if (( $(echo "$COVERAGE < 94.0" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 94% threshold"
            exit 1
          fi

  test-frontend:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Cache node modules
        uses: actions/cache@v3
        with:
          path: ~/.npm
          key: ${{ runner.os }}-node-${{ hashFiles('**/package-lock.json') }}

      - name: Install dependencies
        working-directory: ./nextjs-ui-dashboard
        run: npm ci

      - name: Run tests with coverage
        working-directory: ./nextjs-ui-dashboard
        run: npm run test:coverage

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./nextjs-ui-dashboard/coverage/coverage-final.json
          flags: frontend
          name: frontend-coverage

      - name: Check coverage threshold
        run: |
          COVERAGE=$(npm run test:coverage -- --silent | grep "All files" | awk '{print $4}' | sed 's/%//')
          if (( $(echo "$COVERAGE < 90.0" | bc -l) )); then
            echo "Coverage $COVERAGE% is below 90% threshold"
            exit 1
          fi

  mutation-testing:
    runs-on: ubuntu-latest
    needs: [test-rust, test-python, test-frontend]
    steps:
      - uses: actions/checkout@v3

      - name: Run Rust mutation tests
        working-directory: ./rust-core-engine
        run: |
          cargo install cargo-mutants
          cargo mutants --test-tool=cargo test -- --lib

      - name: Run Python mutation tests
        working-directory: ./python-ai-service
        run: |
          pip install mutmut
          mutmut run --paths-to-mutate=services/,models/
          mutmut results

      - name: Upload mutation reports
        uses: actions/upload-artifact@v3
        with:
          name: mutation-reports
          path: |
            rust-core-engine/mutants.out
            python-ai-service/.mutmut-cache
```

### 5.2 Coverage Badges

Add to README.md:
```markdown
## Test Coverage

[![Rust Coverage](https://codecov.io/gh/YOUR_USERNAME/bot-core/branch/main/graph/badge.svg?flag=rust)](https://codecov.io/gh/YOUR_USERNAME/bot-core)
[![Python Coverage](https://codecov.io/gh/YOUR_USERNAME/bot-core/branch/main/graph/badge.svg?flag=python)](https://codecov.io/gh/YOUR_USERNAME/bot-core)
[![Frontend Coverage](https://codecov.io/gh/YOUR_USERNAME/bot-core/branch/main/graph/badge.svg?flag=frontend)](https://codecov.io/gh/YOUR_USERNAME/bot-core)

| Service | Coverage | Mutation Score | Tests |
|---------|----------|----------------|-------|
| Rust Core | 90%+ | 75%+ | 200+ |
| Python AI | 94%+ | 75%+ | 385+ |
| Frontend | 90%+ | 75%+ | 600+ |
```

---

## 6. Performance & Benchmark Tests

### 6.1 Rust Benchmarks

```rust
// rust-core-engine/benches/strategy_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use binance_trading_bot::strategies::*;

fn rsi_strategy_benchmark(c: &mut Criterion) {
    let prices: Vec<f64> = (0..1000).map(|i| 50000.0 + i as f64).collect();

    c.bench_function("rsi_calculation_1000_prices", |b| {
        b.iter(|| {
            rsi::calculate_rsi(black_box(&prices), black_box(14))
        });
    });
}

fn macd_strategy_benchmark(c: &mut Criterion) {
    let prices: Vec<f64> = (0..1000).map(|i| 50000.0 + i as f64).collect();

    c.bench_function("macd_calculation_1000_prices", |b| {
        b.iter(|| {
            macd::calculate_macd(black_box(&prices), black_box(12), black_box(26), black_box(9))
        });
    });
}

fn position_management_benchmark(c: &mut Criterion) {
    c.bench_function("open_position", |b| {
        b.iter(|| {
            // Benchmark position opening
        });
    });
}

criterion_group!(
    benches,
    rsi_strategy_benchmark,
    macd_strategy_benchmark,
    position_management_benchmark
);
criterion_main!(benches);
```

### 6.2 Python Performance Tests

```python
# python-ai-service/tests/test_performance.py
import pytest
import time
from concurrent.futures import ThreadPoolExecutor

def test_ai_prediction_latency():
    """Ensure AI predictions complete within 100ms"""
    start = time.time()
    # Run prediction
    elapsed = time.time() - start
    assert elapsed < 0.1, f"Prediction took {elapsed}s, should be < 100ms"

def test_concurrent_predictions():
    """Test handling 100 concurrent prediction requests"""
    def make_prediction(i):
        # Simulate prediction
        return i

    with ThreadPoolExecutor(max_workers=100) as executor:
        results = list(executor.map(make_prediction, range(100)))

    assert len(results) == 100

@pytest.mark.benchmark
def test_indicator_calculation_performance(benchmark):
    """Benchmark indicator calculation speed"""
    data = [float(i) for i in range(1000)]

    result = benchmark(calculate_indicators, data)
    assert result is not None
```

### 6.3 Frontend Performance Tests

```typescript
// nextjs-ui-dashboard/src/__tests__/performance/rendering.test.tsx
import { render } from '@testing-library/react';
import { performance } from 'perf_hooks';

describe('Component Rendering Performance', () => {
  it('should render dashboard in under 100ms', () => {
    const start = performance.now();
    render(<Dashboard />);
    const elapsed = performance.now() - start;

    expect(elapsed).toBeLessThan(100);
  });

  it('should handle 1000 chart updates without lag', async () => {
    const { rerender } = render(<TradingChart data={[]} />);

    const start = performance.now();
    for (let i = 0; i < 1000; i++) {
      rerender(<TradingChart data={generateMockData(i)} />);
    }
    const elapsed = performance.now() - start;

    expect(elapsed).toBeLessThan(1000); // < 1ms per update
  });
});
```

---

## 7. Test Documentation Standards

### 7.1 Test Naming Convention
```
test_<what>_<condition>_<expected_result>

Examples:
- test_rsi_strategy_oversold_generates_buy_signal
- test_position_with_max_leverage_respects_limits
- test_websocket_on_disconnect_reconnects_automatically
```

### 7.2 Test Structure (AAA Pattern)
```rust
#[tokio::test]
async fn test_example() {
    // Arrange - Set up test data and preconditions
    let config = create_test_config();
    let service = TestService::new(config);

    // Act - Execute the code under test
    let result = service.perform_action().await;

    // Assert - Verify the outcome
    assert!(result.is_ok());
    assert_eq!(result.unwrap().value, expected_value);
}
```

### 7.3 Documentation Requirements
Each test file must include:
```rust
//! # Module Name - Test Suite
//!
//! ## Coverage Focus
//! - Core functionality
//! - Edge cases
//! - Error handling
//! - Integration points
//!
//! ## Test Categories
//! - Unit tests: Core business logic
//! - Integration tests: Service interactions
//! - Property tests: Randomized inputs
//! - Benchmark tests: Performance validation

#[cfg(test)]
mod tests {
    // Tests here
}
```

---

## 8. Success Criteria & Metrics

### 8.1 Coverage Metrics
- **Rust Core Engine**: ≥ 90% line coverage
- **Python AI Service**: ≥ 94% line coverage (maintain)
- **Frontend**: ≥ 90% line coverage

### 8.2 Quality Metrics
- **Mutation Score**: ≥ 75% (tests actually catch bugs)
- **Test Count**:
  - Rust: ≥ 200 tests
  - Python: ≥ 385 tests (maintain)
  - Frontend: ≥ 600 tests

### 8.3 Performance Metrics
- **Test Execution Time**:
  - Rust: < 5 minutes
  - Python: < 2 minutes
  - Frontend: < 3 minutes
- **CI Pipeline**: < 15 minutes total

### 8.4 Documentation Metrics
- All test files have header documentation
- All complex tests have inline comments
- Test failures provide actionable error messages

---

## 9. Execution Timeline

### Week 1: Rust Coverage Improvement
- **Day 1-2**: Add strategy tests (RSI, MACD, Bollinger, Volume)
- **Day 3-4**: Add position/risk management tests
- **Day 5**: Run coverage analysis, identify remaining gaps

### Week 2: Frontend & Integration
- **Day 1-2**: Add missing hook tests and error boundaries
- **Day 3-4**: Add integration tests with MSW
- **Day 5**: E2E tests and coverage analysis

### Week 3: Mutation Testing & CI/CD
- **Day 1-2**: Set up mutation testing for all services
- **Day 3-4**: Configure CI/CD pipelines with coverage gates
- **Day 5**: Performance benchmarks and documentation

### Week 4: Final Validation
- **Day 1-2**: Run full test suite, fix any failures
- **Day 3**: Generate final coverage reports
- **Day 4**: Create executive summary and metrics dashboard
- **Day 5**: Knowledge transfer and documentation review

---

## 10. Quick Start Commands

### Run All Tests
```bash
# Rust
cd rust-core-engine && cargo test --all

# Python
cd python-ai-service && pytest tests/ -v

# Frontend
cd nextjs-ui-dashboard && npm test
```

### Generate Coverage Reports
```bash
# Rust
cd rust-core-engine
cargo tarpaulin --out Html --output-dir coverage
open coverage/index.html

# Python
cd python-ai-service
pytest --cov=. --cov-report=html
open htmlcov/index.html

# Frontend
cd nextjs-ui-dashboard
npm run test:coverage
open coverage/index.html
```

### Run Mutation Tests
```bash
# Rust
cd rust-core-engine
cargo mutants --test-tool=cargo test

# Python
cd python-ai-service
mutmut run && mutmut html

# Frontend
cd nextjs-ui-dashboard
npx stryker run
```

---

## 11. Troubleshooting

### Common Issues

**Issue 1: Rust tests timeout**
```bash
# Solution: Increase timeout
cargo test -- --test-threads=1 --nocapture
cargo tarpaulin --timeout 600
```

**Issue 2: MongoDB connection failures in tests**
```bash
# Solution: Use test doubles/mocks
# Mark integration tests with #[ignore]
cargo test -- --ignored  # Run only integration tests when DB available
```

**Issue 3: Frontend tests fail in CI**
```bash
# Solution: Use jsdom environment
# In package.json:
"test": "vitest --environment jsdom"
```

**Issue 4: Coverage drops after refactoring**
```bash
# Solution: Run coverage diff
git diff main -- coverage/
# Add tests for new uncovered code
```

---

## 12. Resources

### Tools
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Rust coverage
- [cargo-mutants](https://github.com/sourcefrog/cargo-mutants) - Rust mutation testing
- [pytest-cov](https://pytest-cov.readthedocs.io/) - Python coverage
- [mutmut](https://mutmut.readthedocs.io/) - Python mutation testing
- [vitest](https://vitest.dev/) - Frontend testing
- [Stryker](https://stryker-mutator.io/) - Frontend mutation testing
- [MSW](https://mswjs.io/) - API mocking
- [Codecov](https://codecov.io/) - Coverage reporting

### Best Practices
- [Rust Testing Best Practices](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Python Testing Best Practices](https://docs.python-guide.org/writing/tests/)
- [React Testing Library Best Practices](https://testing-library.com/docs/react-testing-library/intro/)

---

## Appendix: Coverage Report Template

```markdown
# Test Coverage Report - [Date]

## Overall Coverage
| Service | Line Coverage | Branch Coverage | Mutation Score | Tests | Status |
|---------|---------------|-----------------|----------------|-------|--------|
| Rust    | X%            | X%              | X%             | XXX   | ✅/⚠️/❌ |
| Python  | X%            | X%              | X%             | XXX   | ✅/⚠️/❌ |
| Frontend| X%            | X%              | X%             | XXX   | ✅/⚠️/❌ |

## Detailed Breakdown

### Rust Core Engine
**Coverage**: X% (Target: 90%)
- ✅ Fully covered: storage, error handling, config
- ⚠️ Partially covered: websocket (75%), strategies (80%)
- ❌ Low coverage: [module_name] (50%)

**Action Items**:
1. Add WebSocket reconnection tests
2. Add strategy edge case tests
3. Add [module_name] comprehensive tests

### Python AI Service
**Coverage**: X% (Target: 94%)
- ✅ Excellent coverage maintained
- Action Items: [if any]

### Frontend
**Coverage**: X% (Target: 90%)
- ✅ Fully covered: [modules]
- ⚠️ Partially covered: [modules]
- Action Items: [specific tests needed]

## Performance Metrics
- Total test execution time: X minutes
- CI pipeline duration: X minutes
- Average test stability: X%

## Next Steps
1. [Priority 1 action]
2. [Priority 2 action]
3. [Priority 3 action]
```

---

**Last Updated**: 2025-10-10
**Owner**: Development Team
**Review Cycle**: Weekly during improvement phase, Monthly after targets achieved
