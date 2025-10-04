#!/bin/bash

# Next.js UI Dashboard Test Runner

echo "⚛️  Running Next.js Dashboard Tests"
echo "===================================="

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    bun install
fi

# Build first to catch TypeScript errors
echo "🔨 Building project..."
bun run build

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

# Type check
echo "🔍 Running TypeScript checks..."
bun run type-check || echo "⚠️  Type check had warnings (continuing...)"

# Lint
echo "🧹 Running ESLint..."
bun run lint || echo "⚠️  Lint had warnings (continuing...)"

# Run tests with coverage
echo "📊 Running tests with coverage..."
bun run test:coverage -- --reporter=verbose

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "✅ All tests passed!"
    echo "📄 Coverage report available at: coverage/index.html"
else
    echo "❌ Tests failed"
    exit 1
fi

# Run specific test categories
echo ""
echo "📋 Test Summary by Category:"
echo "----------------------------"

echo "Component tests:"
bun run test:run -- --reporter=json src/__tests__/components/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Hook tests:"
bun run test:run -- --reporter=json src/__tests__/hooks/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Utility tests:"
bun run test:run -- --reporter=json src/__tests__/utils/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Context tests:"
bun run test:run -- --reporter=json src/__tests__/contexts/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo "Page tests:"
bun run test:run -- --reporter=json src/__tests__/pages/ 2>/dev/null | jq -r '.testResults | length as $total | map(select(.status == "passed")) | length as $passed | "\($passed)/\($total) passed"' || echo "Check manually"

echo ""
echo "📊 Detailed coverage report: coverage/index.html"
echo "🎨 Interactive test UI: bun run test:ui"