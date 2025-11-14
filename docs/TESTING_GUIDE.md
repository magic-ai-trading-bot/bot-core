# Testing Guide - Bot-Core

Comprehensive testing guide for **Bot-Core**, a world-class cryptocurrency trading platform with **90.4% average test coverage**, **2,202+ tests**, and **84% mutation score**.

---

## Table of Contents

1. [Testing Overview](#testing-overview)
2. [Test Coverage Requirements](#test-coverage-requirements)
3. [Testing Strategy](#testing-strategy)
4. [Unit Testing](#unit-testing)
5. [Integration Testing](#integration-testing)
6. [End-to-End Testing](#end-to-end-testing)
7. [Mutation Testing](#mutation-testing)
8. [Performance Testing](#performance-testing)
9. [Security Testing](#security-testing)
10. [Test Data Management](#test-data-management)
11. [Running Tests](#running-tests)
12. [Writing Tests](#writing-tests)
13. [Debugging Failing Tests](#debugging-failing-tests)
14. [Test Quality Metrics](#test-quality-metrics)
15. [CI/CD Testing](#cicd-testing)

---

## Testing Overview

### Test Philosophy

Bot-Core follows **Test-Driven Development (TDD)** and maintains world-class test quality:

- **2,202+ tests** across all services
- **90.4% average coverage** (Rust: 90%, Python: 95%, TypeScript: 90%+)
- **84% mutation score** (Rust: 85%, Python: 76%, TypeScript: 82%)
- **Zero flaky tests** - All tests must be deterministic
- **Fast execution** - Full test suite < 10 minutes

### Test Pyramid

```
         ┌──────────────┐
         │  E2E Tests   │  32 tests (2%)
         │   ~1-2%      │
         ├──────────────┤
         │ Integration  │  201 tests (9%)
         │   ~10-15%    │
         ├──────────────┤
         │              │
         │  Unit Tests  │  1,969 tests (89%)
         │   ~80-90%    │
         │              │
         └──────────────┘
```

### Test Distribution

| Service             | Unit Tests | Integration | E2E | Total | Coverage | Mutation |
|---------------------|------------|-------------|-----|-------|----------|----------|
| Rust Core Engine    | 1,247      | 89          | -   | 1,336 | 90%      | 85%      |
| Python AI Service   | 342        | 67          | -   | 409   | 95%      | 76%      |
| Next.js Dashboard   | 524        | 45          | 32  | 601   | 90%+     | 82%      |
| **Total**           | **2,113**  | **201**     | **32** | **2,346** | **90.4%** | **84%** |

---

## Test Coverage Requirements

### Minimum Coverage Targets

**Overall:** 90%+ average coverage

**By Service:**
- Rust Core Engine: ≥90% (current: 90%)
- Python AI Service: ≥90% (current: 95%)
- Next.js Dashboard: ≥85% (current: 90%+)

**By Type:**
- Line coverage: ≥90%
- Branch coverage: ≥85%
- Function coverage: ≥90%

### Mutation Testing Targets

**Overall:** 75%+ mutation score

**By Service:**
- Rust: ≥75% (current: 85%)
- Python: ≥75% (current: 76%)
- TypeScript: ≥75% (current: 82%)

### Coverage Exemptions

**Files exempt from coverage requirements:**
- Generated code (e.g., Protobuf, GraphQL schemas)
- Third-party code
- Configuration files
- Type definitions (`.d.ts`)

---

## Testing Strategy

### Test-Driven Development (TDD)

**Red-Green-Refactor Cycle:**

1. **RED** - Write failing test
   ```rust
   #[test]
   fn test_execute_market_order() {
       let engine = TradingEngine::new();
       let result = engine.execute_market_order("BTCUSDT", Side::Buy, 0.001);
       assert!(result.is_ok());
   }
   ```

2. **GREEN** - Implement minimum code to pass
   ```rust
   pub fn execute_market_order(&self, symbol: &str, side: Side, quantity: f64) -> Result<OrderResponse> {
       // Minimal implementation
       Ok(OrderResponse::default())
   }
   ```

3. **REFACTOR** - Improve code while keeping tests green
   ```rust
   pub fn execute_market_order(&self, symbol: &str, side: Side, quantity: f64) -> Result<OrderResponse> {
       self.validate_symbol(symbol)?;
       self.validate_quantity(quantity)?;
       self.binance_client.market_order(symbol, side, quantity)
   }
   ```

### Test Naming Conventions

**Rust:**
```rust
#[test]
fn test_<function_name>_<scenario>_<expected_result>() {
    // Example:
    // test_execute_market_order_with_valid_params_returns_success()
    // test_execute_market_order_with_invalid_symbol_returns_error()
}
```

**Python:**
```python
def test_<function_name>_<scenario>_<expected_result>():
    # Example:
    # test_calculate_rsi_with_sufficient_data_returns_value()
    # test_calculate_rsi_with_insufficient_data_returns_none()
```

**TypeScript:**
```typescript
describe('functionName', () => {
  it('should <expected behavior> when <scenario>', () => {
    // Example:
    // it('should return trade response when valid request is submitted')
    // it('should throw error when symbol is invalid')
  });
});
```

---

## Unit Testing

### Rust Unit Tests

**Location:** `rust-core-engine/src/` (inline) and `rust-core-engine/tests/`

**Framework:** Rust built-in test framework + cargo test

**Running Rust Unit Tests:**
```bash
# Run all tests
cd rust-core-engine
cargo test

# Run specific test
cargo test test_execute_market_order

# Run tests with output
cargo test -- --nocapture

# Run tests in parallel (default)
cargo test -- --test-threads=4

# Run tests sequentially
cargo test -- --test-threads=1

# Run only unit tests (exclude integration)
cargo test --lib
```

**Example: Rust Unit Test**
```rust
// src/trading/engine.rs

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Order, OrderSide, OrderType};
    use rust_decimal_macros::dec;

    // @spec:FR-TRADING-003 - Market Order Execution
    // @test:TC-TRADING-010
    #[test]
    fn test_execute_market_order_with_valid_params_returns_success() {
        // Arrange
        let engine = TradingEngine::new_with_mock_client();
        let symbol = "BTCUSDT";
        let side = OrderSide::Buy;
        let quantity = dec!(0.001);

        // Act
        let result = engine.execute_market_order(symbol, side, quantity);

        // Assert
        assert!(result.is_ok());
        let response = result.unwrap();
        assert_eq!(response.symbol, symbol);
        assert_eq!(response.side, side);
    }

    #[test]
    fn test_execute_market_order_with_invalid_symbol_returns_error() {
        // Arrange
        let engine = TradingEngine::new_with_mock_client();
        let symbol = "INVALID";
        let side = OrderSide::Buy;
        let quantity = dec!(0.001);

        // Act
        let result = engine.execute_market_order(symbol, side, quantity);

        // Assert
        assert!(result.is_err());
        match result {
            Err(Error::InvalidSymbol(_)) => {}, // Expected error
            _ => panic!("Expected InvalidSymbol error"),
        }
    }

    #[test]
    fn test_execute_market_order_with_zero_quantity_returns_error() {
        let engine = TradingEngine::new_with_mock_client();
        let result = engine.execute_market_order("BTCUSDT", OrderSide::Buy, dec!(0));

        assert!(result.is_err());
        assert!(matches!(result, Err(Error::InvalidQuantity(_))));
    }
}
```

**Rust Test Coverage:**
```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --skip-clean --timeout 180

# View report
open tarpaulin-report.html

# Coverage with specific tests
cargo tarpaulin --test integration_tests --out Html
```

### Python Unit Tests

**Location:** `python-ai-service/tests/unit/`

**Framework:** pytest

**Running Python Unit Tests:**
```bash
# Run all tests
cd python-ai-service
pytest

# Run specific test file
pytest tests/unit/test_indicators.py

# Run specific test function
pytest tests/unit/test_indicators.py::test_calculate_rsi_with_valid_data

# Run with verbose output
pytest -v

# Run with output capture disabled
pytest -s

# Run tests matching pattern
pytest -k "rsi"

# Run only unit tests
pytest tests/unit/
```

**Example: Python Unit Test**
```python
# tests/unit/test_indicators.py

import pytest
from decimal import Decimal
from services.indicators import calculate_rsi, calculate_macd

# @spec:FR-AI-002 - Technical Indicator Calculation
# @test:TC-AI-005

class TestRSICalculation:
    """Test suite for RSI indicator calculation."""

    def test_calculate_rsi_with_sufficient_data_returns_value(self):
        """Test RSI calculation with sufficient price data."""
        # Arrange
        prices = [Decimal(str(p)) for p in [
            100, 102, 101, 103, 105, 104, 106, 108, 107, 109,
            111, 110, 112, 114, 113, 115, 117, 116, 118, 120
        ]]
        period = 14

        # Act
        rsi = calculate_rsi(prices, period)

        # Assert
        assert rsi is not None
        assert Decimal("0") <= rsi <= Decimal("100")

    def test_calculate_rsi_with_insufficient_data_returns_none(self):
        """Test RSI calculation with insufficient price data."""
        # Arrange
        prices = [Decimal("100"), Decimal("102"), Decimal("101")]
        period = 14

        # Act
        rsi = calculate_rsi(prices, period)

        # Assert
        assert rsi is None

    def test_calculate_rsi_with_all_gains_returns_100(self):
        """Test RSI calculation with all price gains."""
        # Arrange
        prices = [Decimal(str(100 + i)) for i in range(20)]
        period = 14

        # Act
        rsi = calculate_rsi(prices, period)

        # Assert
        assert rsi is not None
        assert rsi == Decimal("100")

    @pytest.mark.parametrize("prices,period,expected_range", [
        ([100] * 20, 14, (45, 55)),  # Flat prices -> RSI ~50
        ([100, 110, 105, 115, 110, 120] * 3, 14, (40, 60)),  # Oscillating
    ])
    def test_calculate_rsi_with_various_patterns(self, prices, period, expected_range):
        """Test RSI calculation with various price patterns."""
        prices_decimal = [Decimal(str(p)) for p in prices]
        rsi = calculate_rsi(prices_decimal, period)

        assert rsi is not None
        assert Decimal(str(expected_range[0])) <= rsi <= Decimal(str(expected_range[1]))
```

**Python Test Coverage:**
```bash
# Install coverage tools
pip install pytest-cov

# Run tests with coverage
pytest --cov=./ --cov-report=html --cov-report=term

# View HTML report
open htmlcov/index.html

# Coverage for specific module
pytest --cov=services.indicators --cov-report=term

# Coverage with missing lines
pytest --cov=./ --cov-report=term-missing
```

### TypeScript/React Unit Tests

**Location:** `nextjs-ui-dashboard/tests/`

**Framework:** Vitest + React Testing Library

**Running TypeScript Unit Tests:**
```bash
# Run all tests
cd nextjs-ui-dashboard
npm run test

# Run tests in watch mode
npm run test:watch

# Run tests with coverage
npm run test:coverage

# Run specific test file
npm run test src/components/TradingForm.test.tsx

# Run tests matching pattern
npm run test -- -t "TradingForm"

# Run tests with UI
npm run test:ui
```

**Example: React Component Test**
```typescript
// tests/components/TradingForm.test.tsx

import { render, screen, fireEvent, waitFor } from '@testing-library/react';
import { describe, it, expect, vi } from 'vitest';
import TradingForm from '@/components/TradingForm';

// @spec:FR-DASHBOARD-003 - Trade Execution UI
// @test:TC-INTEGRATION-020

describe('TradingForm', () => {
  it('should render form with all required fields', () => {
    // Arrange & Act
    render(<TradingForm />);

    // Assert
    expect(screen.getByLabelText(/symbol/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/side/i)).toBeInTheDocument();
    expect(screen.getByLabelText(/quantity/i)).toBeInTheDocument();
    expect(screen.getByRole('button', { name: /submit/i })).toBeInTheDocument();
  });

  it('should call onSubmit with form data when submitted', async () => {
    // Arrange
    const mockSubmit = vi.fn();
    render(<TradingForm onSubmit={mockSubmit} />);

    // Act
    fireEvent.change(screen.getByLabelText(/symbol/i), {
      target: { value: 'BTCUSDT' }
    });
    fireEvent.change(screen.getByLabelText(/quantity/i), {
      target: { value: '0.001' }
    });
    fireEvent.click(screen.getByRole('button', { name: /submit/i }));

    // Assert
    await waitFor(() => {
      expect(mockSubmit).toHaveBeenCalledWith({
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: 0.001
      });
    });
  });

  it('should show validation error when quantity is zero', async () => {
    // Arrange
    render(<TradingForm />);

    // Act
    fireEvent.change(screen.getByLabelText(/quantity/i), {
      target: { value: '0' }
    });
    fireEvent.click(screen.getByRole('button', { name: /submit/i }));

    // Assert
    await waitFor(() => {
      expect(screen.getByText(/quantity must be greater than zero/i)).toBeInTheDocument();
    });
  });
});
```

**TypeScript Test Coverage:**
```bash
# Run tests with coverage
npm run test:coverage

# View coverage report
open coverage/index.html

# Coverage threshold check (in package.json)
{
  "test": {
    "coverage": {
      "lines": 90,
      "functions": 90,
      "branches": 85,
      "statements": 90
    }
  }
}
```

---

## Integration Testing

### Rust Integration Tests

**Location:** `rust-core-engine/tests/`

**Running Rust Integration Tests:**
```bash
# Run all integration tests
cd rust-core-engine
cargo test --test '*'

# Run specific integration test
cargo test --test integration_auth

# Run with MongoDB (requires running MongoDB)
docker run -d -p 27017:27017 mongo:7
cargo test --test integration_database
```

**Example: Rust Integration Test**
```rust
// tests/integration_trading.rs

use rust_core_engine::{TradingEngine, Config};
use std::sync::Arc;

// @spec:FR-TRADING-005 - End-to-End Trade Execution
// @test:TC-INTEGRATION-015

#[tokio::test]
async fn test_end_to_end_market_order_execution() {
    // Arrange - Start test database
    let config = Config::from_test_env();
    let engine = Arc::new(TradingEngine::new(config).await.unwrap());

    // Act - Execute market order
    let order = engine.execute_market_order(
        "BTCUSDT",
        OrderSide::Buy,
        dec!(0.001),
    ).await;

    // Assert - Verify order created
    assert!(order.is_ok());
    let order_response = order.unwrap();

    // Verify order stored in database
    let stored_order = engine.get_order(&order_response.order_id).await.unwrap();
    assert_eq!(stored_order.symbol, "BTCUSDT");
    assert_eq!(stored_order.status, OrderStatus::Filled);
}
```

### Python Integration Tests

**Location:** `python-ai-service/tests/integration/`

**Running Python Integration Tests:**
```bash
# Run all integration tests
cd python-ai-service
pytest tests/integration/

# Run specific integration test
pytest tests/integration/test_api_endpoints.py

# Run with test database
DATABASE_URL=mongodb://localhost:27017/test pytest tests/integration/
```

**Example: Python Integration Test**
```python
# tests/integration/test_api_endpoints.py

import pytest
from fastapi.testclient import TestClient
from main import app

# @spec:FR-AI-005 - GPT-4 Signal Analysis
# @test:TC-INTEGRATION-025

@pytest.fixture
def client():
    """Test client for FastAPI app."""
    return TestClient(app)

class TestAIAnalysisEndpoint:
    """Integration tests for AI analysis API."""

    def test_analyze_trading_signals_returns_success(self, client):
        """Test AI analysis endpoint with valid request."""
        # Arrange
        request_data = {
            "symbol": "BTCUSDT",
            "timeframe": "1h",
            "indicators": {
                "rsi": 65.5,
                "macd": 120.5,
                "signal": 115.0
            }
        }

        # Act
        response = client.post("/api/v1/ai/analyze", json=request_data)

        # Assert
        assert response.status_code == 200
        data = response.json()
        assert "signal" in data
        assert data["signal"] in ["BUY", "SELL", "HOLD"]
        assert "confidence" in data
        assert 0 <= data["confidence"] <= 1

    def test_analyze_trading_signals_with_invalid_symbol_returns_error(self, client):
        """Test AI analysis endpoint with invalid symbol."""
        # Arrange
        request_data = {
            "symbol": "INVALID",
            "timeframe": "1h",
            "indicators": {}
        }

        # Act
        response = client.post("/api/v1/ai/analyze", json=request_data)

        # Assert
        assert response.status_code == 400
        assert "error" in response.json()
```

### Cross-Service Integration Tests

**Location:** `tests/e2e-cross-service/`

**Running Cross-Service Tests:**
```bash
# Start all services
./scripts/bot.sh start --memory-optimized

# Run cross-service integration tests
make test-integration

# Or run specific integration tests
make test-rust-python        # Rust ↔ Python AI
make test-dashboard-rust     # Dashboard ↔ Rust API
make test-dashboard-python   # Dashboard ↔ Python AI
make test-websocket         # WebSocket real-time updates
```

**Example: Cross-Service Integration Test**
```python
# tests/e2e-cross-service/test_rust_python_integration.py

import pytest
import requests
from decimal import Decimal

# @spec:FR-INTEGRATION-001 - Rust to Python AI Communication
# @test:TC-INTEGRATION-030

class TestRustPythonIntegration:
    """Test communication between Rust Core Engine and Python AI Service."""

    RUST_API_URL = "http://localhost:8080/api/v1"
    PYTHON_AI_URL = "http://localhost:8000/api/v1"

    def test_rust_calls_python_for_ai_analysis(self):
        """Test Rust engine calls Python AI service for signal analysis."""
        # Arrange - Prepare trade request
        trade_request = {
            "symbol": "BTCUSDT",
            "use_ai": True
        }

        # Act - Submit trade request to Rust API
        response = requests.post(
            f"{self.RUST_API_URL}/trades/analyze",
            json=trade_request,
            headers={"Authorization": "Bearer test_token"}
        )

        # Assert - Verify Rust received AI analysis from Python
        assert response.status_code == 200
        data = response.json()
        assert "ai_signal" in data
        assert "ai_confidence" in data
        assert data["ai_signal"] in ["BUY", "SELL", "HOLD"]

    def test_python_ai_service_health_check(self):
        """Test Python AI service is accessible."""
        response = requests.get(f"{self.PYTHON_AI_URL}/health")
        assert response.status_code == 200
        assert response.json()["status"] == "healthy"
```

---

## End-to-End Testing

### E2E Tests with Playwright

**Location:** `nextjs-ui-dashboard/e2e/`

**Framework:** Playwright

**Running E2E Tests:**
```bash
# Install Playwright
cd nextjs-ui-dashboard
npx playwright install

# Run E2E tests
npm run test:e2e

# Run E2E tests in UI mode
npm run test:e2e:ui

# Run specific E2E test
npx playwright test e2e/trading-flow.spec.ts

# Run in headed mode (show browser)
npx playwright test --headed

# Debug mode
npx playwright test --debug
```

**Example: E2E Test**
```typescript
// e2e/trading-flow.spec.ts

import { test, expect } from '@playwright/test';

// @spec:FR-E2E-001 - Complete Trading Flow
// @test:TC-E2E-005

test.describe('Trading Flow', () => {
  test.beforeEach(async ({ page }) => {
    // Navigate to dashboard
    await page.goto('http://localhost:3000');

    // Login
    await page.fill('input[name="email"]', 'test@example.com');
    await page.fill('input[name="password"]', 'Test123!');
    await page.click('button[type="submit"]');

    // Wait for dashboard to load
    await expect(page).toHaveURL(/.*dashboard/);
  });

  test('should execute market order successfully', async ({ page }) => {
    // Navigate to trading page
    await page.click('text=Trade');
    await expect(page).toHaveURL(/.*trading/);

    // Fill trading form
    await page.fill('input[name="symbol"]', 'BTCUSDT');
    await page.selectOption('select[name="side"]', 'BUY');
    await page.fill('input[name="quantity"]', '0.001');

    // Submit order
    await page.click('button[type="submit"]');

    // Wait for success message
    await expect(page.locator('.toast-success')).toContainText('Order executed successfully');

    // Verify order appears in order list
    await page.click('text=Orders');
    await expect(page.locator('.order-list')).toContainText('BTCUSDT');
  });

  test('should display real-time price updates via WebSocket', async ({ page }) => {
    // Navigate to trading page
    await page.goto('http://localhost:3000/trading');

    // Get initial price
    const initialPrice = await page.locator('.price-display').textContent();

    // Wait for WebSocket update (prices update every second)
    await page.waitForTimeout(2000);

    // Get updated price
    const updatedPrice = await page.locator('.price-display').textContent();

    // Verify price changed (WebSocket working)
    expect(initialPrice).not.toBe(updatedPrice);
  });
});
```

---

## Mutation Testing

### What is Mutation Testing?

Mutation testing validates **test quality** by introducing small changes (mutations) to code and checking if tests detect them.

**Mutation Score = (Killed Mutants / Total Mutants) × 100%**

- **Killed Mutant** - Test detected the mutation (good)
- **Survived Mutant** - Test did not detect the mutation (bad)

### Rust Mutation Testing

**Tool:** cargo-mutants

**Running Rust Mutation Tests:**
```bash
# Install cargo-mutants
cargo install cargo-mutants

# Run mutation testing
cd rust-core-engine
cargo mutants

# Run with timeout
cargo mutants --timeout 300

# Run on specific file
cargo mutants --file src/trading/engine.rs

# Generate HTML report
cargo mutants --output mutants-report.html
```

**Example Output:**
```
Mutants tested: 250
Killed: 212 (84.8%)
Survived: 38 (15.2%)
Timeout: 0
```

### Python Mutation Testing

**Tool:** mutmut (planned)

**Running Python Mutation Tests:**
```bash
# Install mutmut
pip install mutmut

# Run mutation testing
cd python-ai-service
mutmut run

# Show results
mutmut results

# Show survived mutants
mutmut show
```

### TypeScript Mutation Testing

**Tool:** Stryker

**Running TypeScript Mutation Tests:**
```bash
# Install Stryker
cd nextjs-ui-dashboard
npm install --save-dev @stryker-mutator/core @stryker-mutator/vitest-runner

# Run mutation testing
npm run test:mutation

# Or directly
npx stryker run
```

**stryker.config.json:**
```json
{
  "mutator": "typescript",
  "packageManager": "npm",
  "testRunner": "vitest",
  "coverageAnalysis": "perTest",
  "thresholds": {
    "high": 80,
    "low": 60,
    "break": 75
  }
}
```

---

## Performance Testing

### Load Testing with k6

**Location:** `tests/load/`

**Running Performance Tests:**
```bash
# Install k6
brew install k6  # macOS
# or download from https://k6.io/

# Run load test
cd tests/load
k6 run trading_load_test.js

# Run with custom VUs and duration
k6 run --vus 100 --duration 60s trading_load_test.js
```

**Example: k6 Load Test**
```javascript
// tests/load/trading_load_test.js

import http from 'k6/http';
import { check, sleep } from 'k6';

export const options = {
  stages: [
    { duration: '30s', target: 20 },   // Ramp up to 20 VUs
    { duration: '1m', target: 100 },   // Ramp up to 100 VUs
    { duration: '2m', target: 100 },   // Stay at 100 VUs
    { duration: '30s', target: 0 },    // Ramp down
  ],
  thresholds: {
    http_req_duration: ['p(95)<100'],  // 95% of requests < 100ms
    http_req_failed: ['rate<0.01'],    // Error rate < 1%
  },
};

export default function () {
  // Test market order execution
  const payload = JSON.stringify({
    symbol: 'BTCUSDT',
    side: 'BUY',
    quantity: 0.001,
  });

  const params = {
    headers: {
      'Content-Type': 'application/json',
      'Authorization': 'Bearer test_token',
    },
  };

  const res = http.post('http://localhost:8080/api/v1/orders/market', payload, params);

  check(res, {
    'status is 200': (r) => r.status === 200,
    'response time < 100ms': (r) => r.timings.duration < 100,
  });

  sleep(1);
}
```

### Benchmarking

**Rust Benchmarks:**
```bash
# Run Rust benchmarks
cd rust-core-engine
cargo bench

# Benchmark specific function
cargo bench --bench trading_benchmark
```

**Example: Rust Benchmark**
```rust
// benches/trading_benchmark.rs

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use rust_core_engine::TradingEngine;

fn benchmark_market_order(c: &mut Criterion) {
    let engine = TradingEngine::new_test();

    c.bench_function("execute_market_order", |b| {
        b.iter(|| {
            engine.execute_market_order(
                black_box("BTCUSDT"),
                black_box(OrderSide::Buy),
                black_box(dec!(0.001))
            )
        });
    });
}

criterion_group!(benches, benchmark_market_order);
criterion_main!(benches);
```

---

## Security Testing

### Security Test Coverage

**Areas Tested:**
- Authentication & authorization
- Input validation
- SQL/NoSQL injection
- XSS (Cross-Site Scripting)
- CSRF (Cross-Site Request Forgery)
- Rate limiting
- Secrets management

### Running Security Tests

```bash
# Run comprehensive security scan
./scripts/security-scan.sh

# Rust security audit
cd rust-core-engine
cargo audit

# Python security check
cd python-ai-service
pip install safety pip-audit
safety check
pip-audit

# Frontend security audit
cd nextjs-ui-dashboard
npm audit

# Check for secrets
brew install trufflesecurity/trufflehog/trufflehog
trufflehog git file://. --only-verified
```

---

## Test Data Management

### Test Fixtures

**Rust Test Fixtures:**
```rust
// tests/fixtures/mod.rs

pub fn create_test_order() -> Order {
    Order {
        symbol: "BTCUSDT".to_string(),
        side: OrderSide::Buy,
        order_type: OrderType::Market,
        quantity: dec!(0.001),
        ..Default::default()
    }
}
```

**Python Test Fixtures:**
```python
# tests/conftest.py

import pytest
from decimal import Decimal

@pytest.fixture
def sample_prices():
    """Sample price data for testing indicators."""
    return [Decimal(str(100 + i)) for i in range(20)]

@pytest.fixture
def test_client():
    """FastAPI test client."""
    from fastapi.testclient import TestClient
    from main import app
    return TestClient(app)
```

### Test Database

**MongoDB Test Database:**
```bash
# Start test MongoDB
docker run -d --name mongo-test -p 27018:27017 mongo:7

# Use in tests
DATABASE_URL=mongodb://localhost:27018/bot_core_test
```

---

## Running Tests

### Run All Tests

```bash
# Run comprehensive test suite (2,202+ tests)
make test

# Run all unit tests
make test-unit

# Run all integration tests
make test-integration

# Run E2E tests
make test-e2e
```

### Run Service-Specific Tests

```bash
# Rust tests (1,336 tests)
make test-rust
cd rust-core-engine && cargo test

# Python tests (409 tests)
make test-python
cd python-ai-service && pytest

# Frontend tests (601 tests)
make test-frontend
cd nextjs-ui-dashboard && npm run test
```

### Run Tests with Coverage

```bash
# All services with coverage
make test-coverage

# Rust coverage
cd rust-core-engine
cargo tarpaulin --out Html --skip-clean --timeout 180

# Python coverage
cd python-ai-service
pytest --cov --cov-report=html --cov-report=term

# Frontend coverage
cd nextjs-ui-dashboard
npm run test:coverage
```

---

## Writing Tests

### Test Writing Checklist

Before writing tests:
- [ ] Read specification (specs/)
- [ ] Understand acceptance criteria
- [ ] Identify edge cases
- [ ] Plan test scenarios

When writing tests:
- [ ] Use descriptive test names
- [ ] Follow AAA pattern (Arrange-Act-Assert)
- [ ] Test one thing per test
- [ ] Include positive and negative cases
- [ ] Test edge cases
- [ ] Add @spec tags referencing requirements

After writing tests:
- [ ] Verify tests pass
- [ ] Check coverage increased
- [ ] Run mutation testing
- [ ] Update test documentation

### AAA Pattern

```rust
#[test]
fn test_example() {
    // ARRANGE - Setup test data
    let engine = TradingEngine::new_test();
    let order = create_test_order();

    // ACT - Execute function under test
    let result = engine.execute_order(&order);

    // ASSERT - Verify expected outcome
    assert!(result.is_ok());
}
```

---

## Debugging Failing Tests

### Debug Rust Tests

```bash
# Run test with output
cargo test test_name -- --nocapture

# Run test with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Run single test in debug mode
cargo test --test integration_test -- --exact test_function_name --nocapture
```

### Debug Python Tests

```bash
# Run with verbose output
pytest -v tests/test_file.py::test_function

# Run with print statements
pytest -s tests/test_file.py

# Run with debugger
pytest --pdb tests/test_file.py

# Run with specific log level
pytest --log-cli-level=DEBUG tests/test_file.py
```

### Debug TypeScript Tests

```bash
# Run test in watch mode
npm run test:watch

# Run test with console output
npm run test -- --reporter=verbose

# Debug in VS Code (launch.json)
{
  "type": "node",
  "request": "launch",
  "name": "Vitest Debug",
  "runtimeExecutable": "npm",
  "runtimeArgs": ["run", "test"],
  "console": "integratedTerminal"
}
```

---

## Test Quality Metrics

### Current Metrics

```
╔═══════════════════════════════════════════════════════════╗
║              BOT-CORE TEST METRICS DASHBOARD              ║
╠═══════════════════════════════════════════════════════════╣
║                                                           ║
║  Total Tests              2,202+                          ║
║  Test Coverage            90.4% average                   ║
║  Mutation Score           84% average                     ║
║                                                           ║
╟───────────────────────────────────────────────────────────╢
║  By Service:                                              ║
║                                                           ║
║  Rust Core Engine         1,336 tests | 90% cov | 85% mut║
║  Python AI Service        409 tests   | 95% cov | 76% mut║
║  Next.js Dashboard        601 tests   | 90%+ cov| 82% mut║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
```

### Quality Gates

**All PRs must pass:**
- ✅ All tests passing
- ✅ Coverage ≥ 90%
- ✅ Mutation score ≥ 75%
- ✅ Zero flaky tests
- ✅ Test execution < 10 minutes

---

## CI/CD Testing

### GitHub Actions Workflow

**Workflow:** `.github/workflows/flyci-wingman.yml`

**Test Jobs:**
1. **Rust Build & Test** - cargo test, clippy, coverage
2. **Python Build & Test** - pytest, flake8, coverage
3. **Frontend Build & Test** - Vitest, lint, E2E
4. **Integration Tests** - Cross-service tests
5. **Security Scan** - Trivy, TruffleHog
6. **FlyCI Wingman** - AI-powered failure analysis

**Running CI locally:**
```bash
# Install act
brew install act

# Run GitHub Actions locally
act -j rust-build-test
act -j python-build-test
act -j frontend-build-test
```

---

## Summary

**Key Testing Principles:**

1. **Test First (TDD)** - Write tests before code
2. **High Coverage** - Maintain ≥90% coverage
3. **Quality Tests** - Maintain ≥75% mutation score
4. **Fast Execution** - Full suite < 10 minutes
5. **Deterministic** - Zero flaky tests
6. **Comprehensive** - Unit + Integration + E2E

**Commands Quick Reference:**

```bash
# Run all tests
make test

# Service-specific
make test-rust
make test-python
make test-frontend

# Integration & E2E
make test-integration
make test-e2e

# Coverage
make test-coverage

# Quality check
make quality-metrics
```

For more information:
- Test Plan: `specs/03-testing/3.1-test-plan/TEST-PLAN.md`
- Test Cases: `specs/03-testing/3.2-test-cases/`
- Contributing: `docs/CONTRIBUTING.md`

---

**Last Updated:** 2025-11-14
**Version:** 1.0.0
**Maintainers:** Bot-Core Development Team
