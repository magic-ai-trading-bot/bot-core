# Testing Guide - Trading Bot Platform

## Table of Contents
1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Test Types](#test-types)
4. [Running Tests](#running-tests)
5. [Writing Tests](#writing-tests)
6. [Coverage Requirements](#coverage-requirements)
7. [Best Practices](#best-practices)
8. [Troubleshooting](#troubleshooting)

---

## Overview

This guide covers testing practices for the cryptocurrency trading bot platform across all three services:
- **Rust Core Engine** - High-performance trading execution
- **Python AI Service** - Machine learning and analysis
- **Frontend Dashboard** - User interface

### Test Philosophy
- **Quality over quantity** - Well-designed tests catch real bugs
- **Fast feedback** - Tests should run quickly
- **Clear failures** - Test failures should be actionable
- **Maintainable** - Tests should be easy to update

---

## Quick Start

### Run All Tests
```bash
# From project root
make test

# Or individually:
cd rust-core-engine && cargo test
cd python-ai-service && pytest tests/
cd nextjs-ui-dashboard && npm test
```

### Generate Coverage Reports
```bash
# Rust
cd rust-core-engine
cargo tarpaulin --out Html
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

---

## Test Types

### 1. Unit Tests
**Purpose**: Test individual functions/methods in isolation

**Examples**:
```rust
// Rust unit test
#[test]
fn test_calculate_rsi() {
    let prices = vec![100.0, 101.0, 99.0, 102.0];
    let rsi = calculate_rsi(&prices, 14);
    assert!(rsi >= 0.0 && rsi <= 100.0);
}
```

```python
# Python unit test
def test_calculate_moving_average():
    prices = [100, 101, 102, 103]
    ma = calculate_moving_average(prices, period=3)
    assert ma == 102.0
```

```typescript
// TypeScript unit test
test('formatPrice should format correctly', () => {
  expect(formatPrice(1234.56)).toBe('$1,234.56');
  expect(formatPrice(0.00123)).toBe('$0.0012');
});
```

### 2. Integration Tests
**Purpose**: Test interactions between components

```rust
#[tokio::test]
async fn test_websocket_and_storage_integration() {
    let storage = Storage::new(&config).await.unwrap();
    let ws = WebSocketClient::connect().await.unwrap();

    // Test data flow: WebSocket → Storage
    let data = ws.receive_market_data().await.unwrap();
    storage.store_market_data(&data).await.unwrap();

    let retrieved = storage.get_market_data().await.unwrap();
    assert_eq!(data, retrieved);
}
```

### 3. End-to-End Tests
**Purpose**: Test complete user workflows

```typescript
describe('Paper Trading E2E', () => {
  it('should complete full trading cycle', async () => {
    // 1. User opens dashboard
    await page.goto('/dashboard');

    // 2. Connect to market data
    await page.click('[data-testid="connect-button"]');

    // 3. View AI signal
    await page.waitForSelector('[data-testid="ai-signal"]');

    // 4. Execute trade
    await page.click('[data-testid="execute-trade"]');

    // 5. Verify position opened
    const position = await page.textContent('[data-testid="open-position"]');
    expect(position).toContain('BTCUSDT');
  });
});
```

### 4. Property-Based Tests
**Purpose**: Test with randomized inputs to find edge cases

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_rsi_always_between_0_and_100(prices in prop::collection::vec(0.0f64..10000.0, 14..100)) {
        let rsi = calculate_rsi(&prices, 14);
        prop_assert!(rsi >= 0.0 && rsi <= 100.0);
    }
}
```

### 5. Performance Tests
**Purpose**: Ensure performance requirements are met

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_strategy_execution(c: &mut Criterion) {
    c.bench_function("execute_rsi_strategy", |b| {
        b.iter(|| execute_strategy(black_box(&market_data)));
    });
}
```

---

## Running Tests

### Rust Core Engine

#### Run all tests
```bash
cd rust-core-engine
cargo test --all
```

#### Run specific test
```bash
cargo test test_calculate_rsi
```

#### Run tests with output
```bash
cargo test -- --nocapture
```

#### Run tests in parallel
```bash
cargo test -- --test-threads=4
```

#### Run integration tests only
```bash
cargo test --test '*'
```

#### Run with coverage
```bash
cargo tarpaulin --out Html --output-dir coverage
```

#### Run benchmarks
```bash
cargo bench
```

### Python AI Service

#### Run all tests
```bash
cd python-ai-service
pytest tests/
```

#### Run specific test file
```bash
pytest tests/test_gpt_analyzer.py
```

#### Run specific test function
```bash
pytest tests/test_gpt_analyzer.py::test_analyze_market_signal
```

#### Run with coverage
```bash
pytest --cov=. --cov-report=html --cov-report=term-missing
```

#### Run tests in parallel
```bash
pytest -n auto
```

#### Run only unit tests
```bash
pytest -m unit
```

#### Run only integration tests
```bash
pytest -m integration
```

### Frontend Dashboard

#### Run all tests
```bash
cd nextjs-ui-dashboard
npm test
```

#### Run tests in watch mode
```bash
npm test -- --watch
```

#### Run specific test file
```bash
npm test -- ChatBot.test.tsx
```

#### Run with coverage
```bash
npm run test:coverage
```

#### Run E2E tests
```bash
npm run test:e2e
```

---

## Writing Tests

### Test Naming Convention

**Pattern**: `test_<what>_<condition>_<expected_result>`

**Good Examples**:
```rust
#[test]
fn test_rsi_with_uptrend_returns_high_value() { }

#[test]
fn test_position_with_insufficient_margin_fails() { }

#[test]
fn test_websocket_on_disconnect_reconnects_automatically() { }
```

**Bad Examples**:
```rust
#[test]
fn test1() { }  // ❌ Not descriptive

#[test]
fn test_rsi() { }  // ❌ Too vague

#[test]
fn test_the_rsi_calculation_when_prices_are_trending_upward() { }  // ❌ Too verbose
```

### AAA Pattern (Arrange-Act-Assert)

```rust
#[tokio::test]
async fn test_open_position_with_valid_parameters_succeeds() {
    // Arrange - Set up test data
    let config = create_test_config();
    let portfolio = PaperPortfolio::new(10000.0);

    // Act - Execute the code under test
    let result = portfolio.open_position(
        "BTCUSDT",
        PositionType::Long,
        0.1,
        50000.0,
        10
    ).await;

    // Assert - Verify the outcome
    assert!(result.is_ok());
    let position = result.unwrap();
    assert_eq!(position.symbol, "BTCUSDT");
    assert_eq!(position.leverage, 10);
}
```

### Test Data Builders

Create reusable test data factories:

```rust
// tests/common/mod.rs
pub struct PositionBuilder {
    symbol: String,
    position_type: PositionType,
    quantity: f64,
    entry_price: f64,
    leverage: u8,
}

impl PositionBuilder {
    pub fn new() -> Self {
        Self {
            symbol: "BTCUSDT".to_string(),
            position_type: PositionType::Long,
            quantity: 0.1,
            entry_price: 50000.0,
            leverage: 10,
        }
    }

    pub fn symbol(mut self, symbol: &str) -> Self {
        self.symbol = symbol.to_string();
        self
    }

    pub fn short(mut self) -> Self {
        self.position_type = PositionType::Short;
        self
    }

    pub fn build(self) -> Position {
        Position::new(
            self.symbol,
            self.position_type,
            self.quantity,
            self.entry_price,
            self.leverage,
        )
    }
}

// Usage in tests
#[test]
fn test_position_creation() {
    let position = PositionBuilder::new()
        .symbol("ETHUSDT")
        .short()
        .build();

    assert_eq!(position.symbol, "ETHUSDT");
}
```

### Mocking External Dependencies

#### Rust - Using mockall
```rust
use mockall::predicate::*;
use mockall::*;

#[automock]
trait BinanceClient {
    async fn get_price(&self, symbol: &str) -> Result<f64>;
}

#[tokio::test]
async fn test_with_mock_binance() {
    let mut mock = MockBinanceClient::new();

    mock.expect_get_price()
        .with(eq("BTCUSDT"))
        .times(1)
        .returning(|_| Ok(50000.0));

    let strategy = TradingStrategy::new(Box::new(mock));
    let price = strategy.fetch_price("BTCUSDT").await.unwrap();

    assert_eq!(price, 50000.0);
}
```

#### Python - Using unittest.mock
```python
from unittest.mock import Mock, patch, AsyncMock

@pytest.mark.asyncio
async def test_analyze_with_mock_openai():
    mock_openai = AsyncMock()
    mock_openai.chat.completions.create.return_value = Mock(
        choices=[Mock(message=Mock(content='{"signal": "BUY"}'))]
    )

    analyzer = GPTTradingAnalyzer(mock_openai)
    result = await analyzer.analyze_market("BTCUSDT", market_data)

    assert result.signal == "BUY"
    mock_openai.chat.completions.create.assert_called_once()
```

#### Frontend - Using MSW
```typescript
import { rest } from 'msw';
import { setupServer } from 'msw/node';

const server = setupServer(
  rest.get('/api/market-data', (req, res, ctx) => {
    return res(ctx.json({
      symbol: 'BTCUSDT',
      price: 50000,
    }));
  })
);

beforeAll(() => server.listen());
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

test('fetches market data', async () => {
  const data = await fetchMarketData('BTCUSDT');
  expect(data.price).toBe(50000);
});
```

### Async Testing

#### Rust
```rust
#[tokio::test]
async fn test_async_operation() {
    let result = async_function().await;
    assert!(result.is_ok());
}

#[tokio::test]
#[should_panic(expected = "Connection failed")]
async fn test_async_panic() {
    failing_async_function().await;
}
```

#### Python
```python
@pytest.mark.asyncio
async def test_async_analysis():
    analyzer = GPTTradingAnalyzer(client)
    result = await analyzer.analyze_market_signal(request)
    assert result.confidence > 0.5
```

#### TypeScript
```typescript
test('async hook updates state', async () => {
  const { result, waitForNextUpdate } = renderHook(() => useMarketData());

  expect(result.current.loading).toBe(true);

  await waitForNextUpdate();

  expect(result.current.loading).toBe(false);
  expect(result.current.data).toBeDefined();
});
```

### Error Testing

```rust
#[test]
fn test_invalid_position_size_returns_error() {
    let result = create_position(-1.0);  // Negative size
    assert!(result.is_err());
    assert!(matches!(result.unwrap_err(), Error::InvalidInput(_)));
}

#[test]
#[should_panic(expected = "Division by zero")]
fn test_division_by_zero_panics() {
    calculate_average(&[]);  // Empty array causes panic
}
```

---

## Coverage Requirements

### Targets
- **Rust Core Engine**: ≥ 90% line coverage
- **Python AI Service**: ≥ 94% line coverage
- **Frontend Dashboard**: ≥ 90% line coverage

### What to Cover

#### ✅ Must Cover (Critical)
- Core business logic (trading strategies, risk management)
- Data transformations (price calculations, indicators)
- Error handling paths
- Security-sensitive code (authentication, validation)

#### ⚠️ Should Cover (Important)
- Integration points (API calls, database operations)
- Edge cases (boundary values, null inputs)
- Configuration handling
- WebSocket communication

#### ⬜ Optional Coverage (Nice to Have)
- Logging statements
- Simple getters/setters
- Type conversions
- Constants

### Viewing Coverage

#### Rust
```bash
# HTML report
cargo tarpaulin --out Html
open coverage/index.html

# Terminal summary
cargo tarpaulin --print-summary

# Identify uncovered lines
cargo tarpaulin --out Stdout | grep "^0|"
```

#### Python
```bash
# HTML report
pytest --cov=. --cov-report=html
open htmlcov/index.html

# Terminal with missing lines
pytest --cov=. --cov-report=term-missing

# Show only uncovered files
pytest --cov=. --cov-report=term-missing | grep -v "100%"
```

#### Frontend
```bash
# HTML report
npm run test:coverage
open coverage/index.html

# Terminal summary
npm test -- --coverage --coverageReporters=text
```

---

## Best Practices

### 1. Test Isolation
Each test should be independent:
```rust
// ❌ Bad - Tests depend on each other
static mut COUNTER: i32 = 0;

#[test]
fn test_increment() {
    unsafe { COUNTER += 1; }
    assert_eq!(unsafe { COUNTER }, 1);
}

#[test]
fn test_double_increment() {
    unsafe { COUNTER += 2; }
    assert_eq!(unsafe { COUNTER }, 3);  // Fails if tests run in different order!
}

// ✅ Good - Each test is independent
#[test]
fn test_counter_increment() {
    let mut counter = Counter::new();
    counter.increment();
    assert_eq!(counter.value(), 1);
}

#[test]
fn test_counter_double_increment() {
    let mut counter = Counter::new();
    counter.increment();
    counter.increment();
    assert_eq!(counter.value(), 2);
}
```

### 2. Use Descriptive Assertions
```rust
// ❌ Bad
assert!(result > 0);

// ✅ Good
assert!(
    result > 0,
    "Expected positive position size, got {}",
    result
);

// ✅ Better - Use specific assertion
assert_eq!(result, expected_size, "Position size mismatch");
```

### 3. Test One Thing at a Time
```rust
// ❌ Bad - Tests multiple concerns
#[test]
fn test_position_management() {
    let position = open_position();
    assert_eq!(position.size, 0.1);
    position.update_price(51000.0);
    assert_eq!(position.pnl, 100.0);
    position.close();
    assert!(position.is_closed);
}

// ✅ Good - Separate tests
#[test]
fn test_position_opening_sets_correct_size() {
    let position = open_position();
    assert_eq!(position.size, 0.1);
}

#[test]
fn test_position_pnl_updates_with_price() {
    let mut position = open_position();
    position.update_price(51000.0);
    assert_eq!(position.pnl, 100.0);
}

#[test]
fn test_position_close_marks_as_closed() {
    let mut position = open_position();
    position.close();
    assert!(position.is_closed);
}
```

### 4. Avoid Testing Implementation Details
```rust
// ❌ Bad - Tests internal implementation
#[test]
fn test_rsi_uses_correct_formula() {
    // Checks how RSI is calculated internally
}

// ✅ Good - Tests behavior
#[test]
fn test_rsi_identifies_overbought_condition() {
    let prices = generate_overbought_prices();
    let rsi = calculate_rsi(&prices, 14);
    assert!(rsi > 70.0, "RSI should indicate overbought");
}
```

### 5. Use Test Fixtures
```rust
// Rust
#[fixture]
fn sample_market_data() -> MarketData {
    MarketData {
        symbol: "BTCUSDT".to_string(),
        prices: vec![50000.0; 100],
        volumes: vec![1000.0; 100],
    }
}

#[rstest]
fn test_with_fixture(sample_market_data: MarketData) {
    let result = analyze(&sample_market_data);
    assert!(result.is_ok());
}
```

```python
# Python
@pytest.fixture
def sample_market_data():
    return {
        "symbol": "BTCUSDT",
        "prices": [50000.0] * 100,
        "volumes": [1000.0] * 100
    }

def test_with_fixture(sample_market_data):
    result = analyze(sample_market_data)
    assert result is not None
```

### 6. Parameterized Tests
```rust
// Rust
use rstest::rstest;

#[rstest]
#[case(30.0, false)]  // Not oversold
#[case(25.0, true)]   // Oversold
#[case(20.0, true)]   // Oversold
fn test_rsi_oversold_detection(#[case] rsi: f64, #[case] expected: bool) {
    assert_eq!(is_oversold(rsi), expected);
}
```

```python
# Python
@pytest.mark.parametrize("rsi,expected", [
    (30.0, False),  # Not oversold
    (25.0, True),   # Oversold
    (20.0, True),   # Oversold
])
def test_rsi_oversold_detection(rsi, expected):
    assert is_oversold(rsi) == expected
```

```typescript
// TypeScript
test.each([
  [30, false],  // Not oversold
  [25, true],   // Oversold
  [20, true],   // Oversold
])('RSI %d should return oversold=%s', (rsi, expected) => {
  expect(isOversold(rsi)).toBe(expected);
});
```

---

## Troubleshooting

### Common Issues

#### 1. Tests Timeout
```bash
# Rust - Increase timeout
cargo test -- --test-threads=1 --nocapture
cargo tarpaulin --timeout 600

# Python - Increase timeout
pytest --timeout=300

# Frontend - Increase Jest timeout
jest.setTimeout(10000);
```

#### 2. Flaky Tests
```rust
// Add retry logic
#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[retry(times = 3, delay = 100)]
async fn test_websocket_connection() {
    // Test implementation
}
```

#### 3. Database/External Service Tests
```bash
# Use Docker for test databases
docker run -d -p 27017:27017 mongo:latest

# Or mark tests as ignored
#[test]
#[ignore]
fn test_requires_database() {
    // Integration test
}

# Run ignored tests explicitly
cargo test -- --ignored
```

#### 4. Coverage Reports Missing Files
```bash
# Rust - Ensure all files are included
cargo tarpaulin --all-features --workspace

# Python - Add coverage config
# .coveragerc
[run]
source = .
omit =
    */tests/*
    */venv/*

# Frontend - Update jest config
collectCoverageFrom: [
  'src/**/*.{ts,tsx}',
  '!src/**/*.d.ts',
  '!src/**/*.stories.tsx',
]
```

### Debug Commands

```bash
# Rust - Show test output
cargo test -- --nocapture --test-threads=1

# Rust - Run single test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Python - Show print statements
pytest -s

# Python - Debug with pdb
pytest --pdb

# Frontend - Debug in Chrome
npm test -- --debug
```

---

## Additional Resources

### Documentation
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [pytest Documentation](https://docs.pytest.org/)
- [Testing Library](https://testing-library.com/docs/react-testing-library/intro/)

### Tools
- [cargo-tarpaulin](https://github.com/xd009642/tarpaulin) - Rust coverage
- [cargo-nextest](https://nexte.st/) - Fast Rust test runner
- [pytest-xdist](https://pypi.org/project/pytest-xdist/) - Parallel Python tests
- [MSW](https://mswjs.io/) - API mocking

### CI/CD
- See `.github/workflows/test-coverage.yml` for automated testing
- Coverage reports are uploaded to Codecov
- Tests run on every PR and push to main

---

**Last Updated**: 2025-10-10
**Maintainers**: Development Team
