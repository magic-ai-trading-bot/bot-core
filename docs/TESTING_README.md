# Testing - Quick Reference

## 📋 Quick Links
- **[Complete Coverage Plan](./TESTING_COVERAGE_PLAN.md)** - Detailed roadmap to 90%+ coverage
- **[Testing Guide](./TESTING_GUIDE.md)** - Developer testing handbook
- **[Coverage Report](../TEST_COVERAGE_REPORT.md)** - Executive summary & current status

## 🚀 Quick Start

### Run Tests

```bash
# All services
make test

# Individual services
cd rust-core-engine && cargo test --all
cd python-ai-service && pytest tests/ -v
cd nextjs-ui-dashboard && npm test
```

### Generate Coverage

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

## 📊 Current Status

| Service | Coverage | Tests | Status |
|---------|----------|-------|--------|
| **Python AI** | 94% | 385+ | ✅ Excellent |
| **Rust Core** | 70% | 150+ | ⚠️ Needs work |
| **Frontend** | 82% | 565+ | ⚠️ Good |

## 🎯 Coverage Targets

- **Python AI Service**: 94%+ (maintain)
- **Rust Core Engine**: 90%+ (improve from 70%)
- **Frontend Dashboard**: 90%+ (improve from 82%)

## 🛠️ Tools Installed

### Rust
- **cargo-tarpaulin** - Coverage analysis
- **cargo-mutants** - Mutation testing
- **criterion** - Performance benchmarks

### Python
- **pytest-cov** - Coverage reporting
- **mutmut** - Mutation testing
- **pytest-asyncio** - Async test support

### Frontend
- **vitest** - Test runner
- **@vitest/coverage-v8** - Coverage
- **@stryker-mutator** - Mutation testing
- **MSW** - API mocking

## 📝 Writing Tests

### Naming Convention
```
test_<what>_<condition>_<expected_result>
```

Examples:
- `test_rsi_with_uptrend_returns_high_value`
- `test_position_with_insufficient_margin_fails`
- `test_websocket_on_disconnect_reconnects`

### AAA Pattern
```rust
#[test]
fn test_example() {
    // Arrange - Setup
    let data = create_test_data();

    // Act - Execute
    let result = function_under_test(data);

    // Assert - Verify
    assert_eq!(result, expected);
}
```

## 🔧 Troubleshooting

### Rust Tests Timeout
```bash
cargo test -- --test-threads=1 --nocapture
cargo tarpaulin --timeout 600
```

### Python Import Errors
```bash
pip install -r requirements.txt
pytest tests/ --tb=short
```

### Frontend Test Failures
```bash
npm ci  # Clean install
npm test -- --no-cache
```

## 📈 CI/CD Pipeline

Tests run automatically on:
- ✅ Every pull request
- ✅ Every push to main
- ✅ Nightly for mutation testing

Pipeline includes:
- Unit & integration tests
- Coverage reporting (Codecov)
- Security scanning (Trivy)
- Performance benchmarks
- Code quality checks

**View pipeline**: `.github/workflows/test-coverage.yml`

## 🎯 Next Steps

1. **Fix compilation issues** (if any remain)
2. **Run baseline coverage analysis**
3. **Implement missing tests** (see Coverage Plan)
4. **Achieve 90%+ coverage** across all services

## 📚 Resources

- [Rust Testing](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [pytest Docs](https://docs.pytest.org/)
- [Testing Library](https://testing-library.com/)
- [Coverage Plan](./TESTING_COVERAGE_PLAN.md)
- [Testing Guide](./TESTING_GUIDE.md)

---

**Last Updated**: 2025-10-10
**Status**: Infrastructure complete, implementation in progress
