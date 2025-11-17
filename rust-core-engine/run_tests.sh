#!/bin/bash

# Rust Core Engine Test Runner

echo "🦀 Running Rust Core Engine Tests"
echo "=================================="

# Skip separate build - tarpaulin will build with instrumentation
# Building separately is redundant and wastes time

# Run tests with coverage using cargo-tarpaulin
echo "📊 Running tests with coverage..."

# Check if tarpaulin is installed
if ! command -v cargo-tarpaulin &> /dev/null; then
    echo "📦 Installing cargo-tarpaulin..."
    cargo install cargo-tarpaulin
fi

# Run tests with coverage
# Increased timeout from 120 to 300 seconds
# Added --skip-clean to avoid rebuilding unnecessarily
# Using --lib to reduce build size (integration tests already run in CI)
cargo tarpaulin \
    --lib \
    --timeout 300 \
    --skip-clean \
    --out Xml \
    --out Html \
    --output-dir ./target/tarpaulin \
    --verbose

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "✅ All tests passed!"
    echo "📄 Coverage report available at: target/tarpaulin/tarpaulin-report.html"
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