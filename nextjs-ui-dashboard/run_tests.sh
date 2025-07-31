#!/bin/bash

# Next.js UI Dashboard Test Runner

echo "⚛️  Running Next.js Dashboard Tests"
echo "===================================="

# Install dependencies if needed
if [ ! -d "node_modules" ]; then
    echo "📦 Installing dependencies..."
    npm ci
fi

# Build first to catch TypeScript errors
echo "🔨 Building project..."
npm run build

if [ $? -ne 0 ]; then
    echo "❌ Build failed"
    exit 1
fi

# Type check
echo "🔍 Running TypeScript checks..."
npm run type-check

if [ $? -ne 0 ]; then
    echo "❌ Type check failed"
    exit 1
fi

# Lint
echo "🧹 Running ESLint..."
npm run lint

if [ $? -ne 0 ]; then
    echo "❌ Lint failed"
    exit 1
fi

# Run tests with coverage
echo "📊 Running tests with coverage..."
npm run test:coverage -- --reporter=verbose

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "✅ All tests passed with >90% coverage!"
    echo "📄 Coverage report available at: coverage/index.html"
else
    echo "❌ Tests failed or coverage below 90%"
    exit 1
fi

# Run specific test categories
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
echo "🎯 Coverage target: >90%"
echo "📊 Detailed coverage report: coverage/index.html"
echo "🎨 Interactive test UI: npm run test:ui"