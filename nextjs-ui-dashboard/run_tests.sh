#!/bin/bash

# Next.js UI Dashboard Test Runner

echo "⚛️  Running Next.js Dashboard Tests"
echo "===================================="

# Detect package manager
if command -v bun &> /dev/null; then
    PKG_MANAGER="bun"
    echo "📦 Using bun for package management"
else
    PKG_MANAGER="npm"
    echo "📦 Using npm for package management"
fi

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    $PKG_MANAGER install
fi

# Build first to catch TypeScript errors
echo "🔨 Building project..."
$PKG_MANAGER run build

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

# Type check
echo "🔍 Running TypeScript checks..."
$PKG_MANAGER run type-check || echo "⚠️  Type check had warnings (continuing...)"

# Lint
echo "🧹 Running ESLint..."
$PKG_MANAGER run lint || echo "⚠️  Lint had warnings (continuing...)"

# Run tests with coverage using npm (vitest)
# IMPORTANT: Use npm for tests regardless of package manager
# because this project uses vitest, not bun test
echo "📊 Running tests with coverage (using npm/vitest)..."
npm run test:coverage -- --reporter=verbose

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "✅ All tests passed!"
    echo "📄 Coverage report available at: coverage/index.html"
else
    echo "❌ Tests failed"
    exit 1
fi

# Run specific test categories (also using npm for consistency)
echo ""
echo "📋 Test Summary by Category:"
echo "----------------------------"

echo "Component tests:"
npm run test:run -- --reporter=json src/__tests__/components/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Hook tests:"
npm run test:run -- --reporter=json src/__tests__/hooks/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Utility tests:"
npm run test:run -- --reporter=json src/__tests__/utils/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Context tests:"
npm run test:run -- --reporter=json src/__tests__/contexts/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Page tests:"
npm run test:run -- --reporter=json src/__tests__/pages/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo ""
echo "📊 Detailed coverage report: coverage/index.html"
echo "🎨 Interactive test UI: npm run test:ui"