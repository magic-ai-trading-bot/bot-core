#!/bin/bash

# Bot Core - Complete Test Suite Runner

echo "🤖 Bot Core - Running All Tests"
echo "==============================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Track test results
PYTHON_RESULT=0
RUST_RESULT=0
NEXTJS_RESULT=0

# Function to print colored output
print_status() {
    if [ $2 -eq 0 ]; then
        echo -e "${GREEN}✅ $1 - PASSED${NC}"
    else
        echo -e "${RED}❌ $1 - FAILED${NC}"
    fi
}

# Function to run tests with timeout
run_with_timeout() {
    timeout 300 "$@"
    return $?
}

echo -e "${BLUE}🧪 Test Execution Summary${NC}"
echo "=========================="

# 1. Python AI Service Tests
echo ""
echo -e "${YELLOW}1. Running Python AI Service Tests...${NC}"
echo "-------------------------------------"
cd python-ai-service
if [ -f "run_tests.sh" ]; then
    run_with_timeout ./run_tests.sh
    PYTHON_RESULT=$?
else
    echo "❌ Python test runner not found"
    PYTHON_RESULT=1
fi
cd ..

# 2. Rust Core Engine Tests
echo ""
echo -e "${YELLOW}2. Running Rust Core Engine Tests...${NC}"
echo "------------------------------------"
cd rust-core-engine
if [ -f "run_tests.sh" ]; then
    run_with_timeout ./run_tests.sh
    RUST_RESULT=$?
else
    echo "❌ Rust test runner not found"
    RUST_RESULT=1
fi
cd ..

# 3. Next.js Dashboard Tests
echo ""
echo -e "${YELLOW}3. Running Next.js Dashboard Tests...${NC}"
echo "------------------------------------"
cd nextjs-ui-dashboard
if [ -f "run_tests.sh" ]; then
    run_with_timeout ./run_tests.sh
    NEXTJS_RESULT=$?
else
    echo "❌ Next.js test runner not found"
    NEXTJS_RESULT=1
fi
cd ..

# Summary
echo ""
echo -e "${BLUE}📊 Final Test Results${NC}"
echo "====================="
echo ""

print_status "Python AI Service Tests" $PYTHON_RESULT
print_status "Rust Core Engine Tests" $RUST_RESULT
print_status "Next.js Dashboard Tests" $NEXTJS_RESULT

echo ""
echo -e "${BLUE}📈 Coverage Reports${NC}"
echo "=================="
echo "• Python Coverage: python-ai-service/htmlcov/index.html"
echo "• Rust Coverage: rust-core-engine/target/tarpaulin/tarpaulin-report.html" 
echo "• Next.js Coverage: nextjs-ui-dashboard/coverage/index.html"

# Calculate overall result
TOTAL_FAILED=$((PYTHON_RESULT + RUST_RESULT + NEXTJS_RESULT))

echo ""
if [ $TOTAL_FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 ALL TESTS PASSED! 🎉${NC}"
    echo -e "${GREEN}Your codebase has achieved >90% test coverage across all services.${NC}"
    exit 0
else
    echo -e "${RED}💥 $TOTAL_FAILED TEST SUITE(S) FAILED${NC}"
    echo ""
    echo "Services that failed:"
    [ $PYTHON_RESULT -ne 0 ] && echo -e "${RED}  • Python AI Service${NC}"
    [ $RUST_RESULT -ne 0 ] && echo -e "${RED}  • Rust Core Engine${NC}"
    [ $NEXTJS_RESULT -ne 0 ] && echo -e "${RED}  • Next.js Dashboard${NC}"
    echo ""
    echo "Please check the individual test outputs above for details."
    exit 1
fi