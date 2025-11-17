#!/bin/bash

# Rust Core Engine Test Runner

echo "🦀 Running Rust Core Engine Tests"
echo "=================================="

# Run tests with coverage using cargo-llvm-cov (faster than tarpaulin)
echo "📊 Running tests with coverage..."

# Check if llvm-tools and cargo-llvm-cov are installed
if ! rustup component list | grep -q "llvm-tools-preview (installed)"; then
    echo "📦 Installing llvm-tools-preview..."
    rustup component add llvm-tools-preview
fi

if ! command -v cargo-llvm-cov &> /dev/null; then
    echo "📦 Installing cargo-llvm-cov..."
    cargo install cargo-llvm-cov
fi

# Run tests with coverage using llvm-cov
# Much faster than tarpaulin (2-3x)
# Uses LLVM's native coverage instrumentation
cargo llvm-cov --lib --lcov --output-path coverage.lcov

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "✅ All tests passed!"
    echo "📄 Coverage report: coverage.lcov"
    # Show coverage summary
    cargo llvm-cov report --lib
else
    echo "❌ Tests failed"
    exit 1
fi

# Run clippy for additional checks
echo ""
echo "🔍 Running clippy checks..."
cargo clippy --all-features --all-targets -- -D warnings

# Run rustfmt check
echo ""
echo "📝 Running format check..."
cargo fmt -- --check

# Run specific test categories
echo ""
echo "📋 Test Summary by Module:"
echo "-------------------------"
echo "Auth tests:"
cargo test test_auth --quiet | grep -E "(test result:|passed|failed)" | tail -1

echo "Trading tests:"
cargo test test_trading --quiet | grep -E "(test result:|passed|failed)" | tail -1

echo "Strategy tests:"
cargo test test_strategies --quiet | grep -E "(test result:|passed|failed)" | tail -1

echo "Paper trading tests:"
cargo test test_paper_trading --quiet | grep -E "(test result:|passed|failed)" | tail -1

echo "WebSocket tests:"
cargo test test_websocket --quiet | grep -E "(test result:|passed|failed)" | tail -1

echo "Storage tests:"
cargo test test_storage --quiet | grep -E "(test result:|passed|failed)" | tail -1

echo ""
echo "📊 Detailed coverage report: target/tarpaulin/tarpaulin-report.html"