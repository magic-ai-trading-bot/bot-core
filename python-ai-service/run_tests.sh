#!/bin/bash

# Python AI Service Test Runner

echo "🧪 Running Python AI Service Tests"
echo "=================================="

# Install test dependencies
pip install -r requirements.test.txt

# Run tests with coverage
echo "📊 Running tests with coverage..."
pytest \
    --cov=. \
    --cov-report=term-missing:skip-covered \
    --cov-report=html:htmlcov \
    --cov-report=xml \
    -v

# Check if tests passed
if [ $? -eq 0 ]; then
    echo "✅ All tests passed!"
    echo "📄 Coverage report available at: htmlcov/index.html"
else
    echo "❌ Tests failed"
    exit 1
fi

# Run specific test categories
echo ""
echo "🔍 Test Summary by Category:"
echo "----------------------------"
pytest -v -m unit --tb=no | grep -E "(PASSED|FAILED)" | wc -l | xargs echo "Unit tests:"
pytest -v -m integration --tb=no | grep -E "(PASSED|FAILED)" | wc -l | xargs echo "Integration tests:"
pytest -v -m api --tb=no | grep -E "(PASSED|FAILED)" | wc -l | xargs echo "API tests:"