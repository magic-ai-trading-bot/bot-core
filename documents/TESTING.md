# Testing Guide

This document provides comprehensive guidance for testing the Bot Core microservices trading bot.

## 🎯 Testing Goals

- **>90% code coverage** across all services
- **Unit tests** for individual components
- **Integration tests** for service interactions
- **End-to-end tests** for complete workflows
- **Performance tests** for scalability
- **Security tests** for vulnerability detection

## 🏗️ Test Architecture

### Services Overview

| Service | Language | Test Framework | Coverage Tool | Location |
|---------|----------|---------------|---------------|----------|
| **Python AI Service** | Python | pytest | coverage.py | `python-ai-service/tests/` |
| **Rust Core Engine** | Rust | cargo test | tarpaulin | `rust-core-engine/tests/` |
| **Next.js Dashboard** | TypeScript | Vitest | v8 | `nextjs-ui-dashboard/src/__tests__/` |

## 🚀 Quick Start

### Run All Tests

```bash
# Run complete test suite with coverage
make test

# Or use the main runner directly
./run_all_tests.sh

# Run individual services
make test-python
make test-rust  
make test-frontend
```

### Check Coverage Reports

After running tests, coverage reports are available at:

- **Python**: `python-ai-service/htmlcov/index.html`
- **Rust**: `rust-core-engine/target/tarpaulin/tarpaulin-report.html`
- **Next.js**: `nextjs-ui-dashboard/coverage/index.html`

## 📝 Test Categories

### 1. Unit Tests

Test individual functions, methods, and components in isolation.

#### Python AI Service
```bash
cd python-ai-service
pytest tests/unit/ -v
```

**Coverage Areas:**
- ML model predictions (`test_models.py`)
- Technical indicators (`test_indicators.py`) 
- API endpoints (`test_api.py`)
- Data validation (`test_validation.py`)

#### Rust Core Engine
```bash
cd rust-core-engine
cargo test --lib
```

**Coverage Areas:**
- Authentication (`test_auth.rs`)
- Trading logic (`test_trading.rs`)
- Strategy engine (`test_strategies.rs`)
- WebSocket handling (`test_websocket.rs`)

#### Next.js Dashboard
```bash
cd nextjs-ui-dashboard
npm run test:run
```

**Coverage Areas:**
- React components (`__tests__/components/`)
- Custom hooks (`__tests__/hooks/`)
- Utility functions (`__tests__/utils/`)
- Context providers (`__tests__/contexts/`)

### 2. Integration Tests

Test interactions between services and external dependencies.

#### Service Communication
```bash
# Test Rust ↔ Python AI communication
make test-rust-python

# Test Dashboard ↔ Rust API communication  
make test-dashboard-rust

# Test WebSocket real-time updates
make test-websocket
```

**Test Scenarios:**
- API request/response flows
- Database operations
- Message queuing
- External API integrations (Binance, OpenAI)

### 3. End-to-End Tests

Test complete user workflows from frontend to backend.

**Key Workflows:**
- User registration and authentication
- Trade execution flow
- Portfolio monitoring
- Strategy configuration
- Real-time data updates

## 🔧 Test Configuration

### Environment Setup

Each service requires specific test environment variables:

#### Python AI Service (`python-ai-service/.env.test`)
```env
DATABASE_URL=mongodb://localhost:27017/test_trading_bot
OPENAI_API_KEYS=test_key_1,test_key_2,test_key_3
ENVIRONMENT=test
LOG_LEVEL=DEBUG
```

#### Rust Core Engine (`rust-core-engine/config.test.toml`)
```toml
[database]
uri = \"mongodb://localhost:27017/test_trading_bot\"

[jwt]
secret = \"test_jwt_secret_key_for_testing_only\"

[binance]
testnet = true
```

#### Next.js Dashboard (`nextjs-ui-dashboard/.env.test`)
```env
VITE_API_BASE_URL=http://localhost:8080
VITE_WS_URL=ws://localhost:8080/ws
VITE_ENVIRONMENT=test
```

### Mock Data

Test data generators are available in each service:

- **Python**: `tests/fixtures/` - Sample market data, AI responses
- **Rust**: `tests/common/mod.rs` - Mock trades, users, candles
- **Next.js**: `src/test/utils.tsx` - Component props, API responses

## 📊 Coverage Requirements

### Minimum Coverage Thresholds

All services must maintain **>90% coverage** in:

- **Lines**: 90%
- **Functions**: 90% 
- **Branches**: 90%
- **Statements**: 90%

### Coverage Exclusions

Files excluded from coverage requirements:

- Configuration files (`*.config.*`)
- Type definitions (`*.d.ts`)
- Test files themselves
- Main entry points (`main.tsx`, `main.py`)
- Migration scripts

## 🏃‍♂️ Running Tests

### Local Development

```bash
# Setup test environment
make setup-dev

# Run tests with watch mode
cd python-ai-service && pytest --watch
cd rust-core-engine && cargo watch -x test
cd nextjs-ui-dashboard && npm run test:ui
```

### CI/CD Pipeline

Tests run automatically on:

- **Push** to `main` or `develop` branches
- **Pull requests** to `main` or `develop` 
- **Scheduled** nightly builds

**Pipeline Stages:**
1. Unit tests (parallel)
2. Integration tests
3. Security scanning
4. Performance tests
5. Deployment (on success)

### Docker Testing

Run tests in isolated containers:

```bash
# Build test images
docker-compose -f docker-compose.test.yml build

# Run test suite
docker-compose -f docker-compose.test.yml up --abort-on-container-exit

# View results
docker-compose -f docker-compose.test.yml logs
```

## 🧪 Writing Tests

### Python Test Example

```python
import pytest
from unittest.mock import patch, MagicMock
from fastapi.testclient import TestClient

@pytest.mark.asyncio
async def test_ai_analysis_endpoint(client, sample_candles, mock_openai):
    \"\"\"Test AI analysis endpoint with mocked OpenAI.\"\"\"
    
    # Setup mock
    mock_openai.chat.completions.create.return_value = MagicMock(
        choices=[MagicMock(
            message=MagicMock(
                content='{\"signal\": \"Long\", \"confidence\": 0.85}'
            )
        )]
    )
    
    # Make request
    response = await client.post(\"/ai/analyze\", json={
        \"symbol\": \"BTCUSDT\",
        \"candles\": sample_candles
    })
    
    # Assertions
    assert response.status_code == 200
    data = response.json()
    assert data[\"signal\"] == \"Long\"
    assert data[\"confidence\"] == 0.85
```

### Rust Test Example

```rust
#[actix_web::test]
async fn test_execute_trade_success() {
    let db = setup_test_db().await;
    let storage = Storage::new_with_db(Some(db.clone())).await;
    
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(storage))
            .configure(routes::configure_trading_routes)
    ).await;

    let req = test::TestRequest::post()
        .uri(\"/api/trades/execute\")
        .insert_header((\"Authorization\", format!(\"Bearer {}\", create_test_jwt(\"user123\"))))
        .set_json(sample_trade_request())
        .to_request();

    let resp = test::call_service(&app, req).await;
    assert_success_response!(resp);

    cleanup_test_db(db).await;
}
```

### React Test Example

```typescript
import { describe, it, expect } from 'vitest'
import { render, screen, userEvent } from '../../test/utils'
import TradingInterface from '../../components/TradingInterface'

describe('TradingInterface', () => {
  it('executes trade successfully', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    // Fill form
    await user.selectOptions(screen.getByLabelText(/symbol/i), 'BTCUSDT')
    await user.type(screen.getByLabelText(/quantity/i), '0.001')
    
    // Submit
    await user.click(screen.getByRole('button', { name: /execute/i }))
    
    // Verify
    expect(mockExecuteTrade).toHaveBeenCalledWith({
      symbol: 'BTCUSDT',
      quantity: 0.001,
      side: 'BUY'
    })
  })
})
```

## 🐛 Debugging Tests

### Common Issues

1. **Database Connection Failures**
   ```bash
   # Start test database
   docker run -d -p 27017:27017 mongo:7.0
   ```

2. **Port Conflicts**
   ```bash
   # Check port usage
   lsof -i :8080
   lsof -i :8000
   lsof -i :3000
   ```

3. **Environment Variables**
   ```bash
   # Verify test environment
   env | grep -E \"(DATABASE_URL|JWT_SECRET|OPENAI_API_KEYS)\"
   ```

### Debug Commands

```bash
# Run single test with debug output
cd python-ai-service && pytest tests/test_specific.py::test_function -v -s

# Run Rust test with debug
cd rust-core-engine && cargo test test_specific -- --nocapture

# Run React test with debug
cd nextjs-ui-dashboard && npm run test -- --reporter=verbose test_specific
```

## 📈 Performance Testing

### Load Testing

```bash
# API load testing with Artillery
cd python-ai-service
artillery run tests/load/api-load-test.yml

# WebSocket connection testing
cd nextjs-ui-dashboard  
npm run test:websocket:load
```

### Memory Testing

```bash
# Monitor memory usage during tests
cd rust-core-engine
cargo test --release -- --test-threads=1 --nocapture | grep -i memory
```

## 🔒 Security Testing

### Vulnerability Scanning

```bash
# Run security audit
make security-audit

# Dependency vulnerability check
cd python-ai-service && safety check
cd rust-core-engine && cargo audit
cd nextjs-ui-dashboard && npm audit
```

### Authentication Testing

All services include tests for:

- JWT token validation
- API key rotation
- Rate limiting
- Input sanitization
- SQL/NoSQL injection prevention

## 📋 Test Checklist

Before pushing code, ensure:

- [ ] All tests pass locally
- [ ] Coverage thresholds met (>90%)
- [ ] No security vulnerabilities
- [ ] Performance within acceptable limits
- [ ] Documentation updated
- [ ] Integration tests pass
- [ ] E2E workflows functional

## 🆘 Troubleshooting

### Test Failures

1. **Check logs**: Each test runner provides detailed output
2. **Verify environment**: Ensure all required services are running
3. **Clear cache**: Remove `node_modules`, `target`, `__pycache__`
4. **Database state**: Tests may leave data, clear between runs
5. **Port conflicts**: Ensure test ports are available

### Getting Help

- **CI/CD Issues**: Check GitHub Actions logs
- **Local Setup**: Run `make setup-dev` 
- **Coverage Problems**: Review exclusion patterns
- **Performance Issues**: Use profiling tools

---

## 📚 Additional Resources

- [Python Testing Best Practices](https://docs.pytest.org/)
- [Rust Testing Guide](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Vitest Documentation](https://vitest.dev/)
- [GitHub Actions Guide](https://docs.github.com/en/actions)

**Happy Testing! 🧪✨**