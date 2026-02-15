#!/bin/bash

# Environment Variable Validation Script
# Comprehensive checks for all required environment variables
# @spec:FR-SEC-001 - Environment Validation
# @ref:specs/02-design/2.5-components/COMP-SECURITY.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
ERRORS=0
WARNINGS=0
CHECKS_PASSED=0
TOTAL_CHECKS=0

# Validation report file
REPORT_FILE="${1:-/tmp/env-validation-report.txt}"

# Function to log to both console and file
log() {
    echo -e "$1" | tee -a "$REPORT_FILE"
}

echo "=========================================="
echo "Environment Variable Validation"
echo "=========================================="
echo ""
echo "Report will be saved to: $REPORT_FILE"
echo ""

# Initialize report
cat > "$REPORT_FILE" << EOF
Environment Validation Report
Generated: $(date)
Working Directory: $(pwd)
User: $(whoami)

========================================
VALIDATION RESULTS
========================================

EOF

# Check if .env file exists
if [ ! -f .env ]; then
    log "${RED}ERROR: .env file not found${NC}"
    log "Please copy .env.example to .env and fill in the required values."
    log "Run: cp .env.example .env"
    echo "CRITICAL: .env file missing" >> "$REPORT_FILE"
    exit 1
fi

# Source the .env file
set -a
source .env 2>/dev/null || true
set +a

# Function to check if variable is set and not empty
check_required() {
    local var_name=$1
    local var_value="${!var_name}"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [ -z "$var_value" ]; then
        log "${RED}✗ $var_name is not set or empty${NC}"
        echo "FAILED: $var_name - not set" >> "$REPORT_FILE"
        ERRORS=$((ERRORS + 1))
        return 1
    else
        log "${GREEN}✓ $var_name is set${NC}"
        echo "PASSED: $var_name - set" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
        return 0
    fi
}

# Function to check optional variable
check_optional() {
    local var_name=$1
    local var_value="${!var_name}"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [ -z "$var_value" ]; then
        log "${YELLOW}⚠ $var_name is not set (optional)${NC}"
        echo "WARNING: $var_name - not set (optional)" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
    else
        log "${GREEN}✓ $var_name is set${NC}"
        echo "PASSED: $var_name - set" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    fi
}

# Function to check variable length
check_length() {
    local var_name=$1
    local min_length=$2
    local var_value="${!var_name}"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [ ! -z "$var_value" ] && [ ${#var_value} -lt $min_length ]; then
        log "${RED}✗ $var_name is too short (${#var_value} chars, minimum $min_length)${NC}"
        echo "FAILED: $var_name - length ${#var_value} < $min_length" >> "$REPORT_FILE"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
    echo "PASSED: $var_name - length >= $min_length" >> "$REPORT_FILE"
    return 0
}

# Function to check for default/weak values
check_not_default() {
    local var_name=$1
    local var_value="${!var_name}"
    shift
    local defaults=("$@")
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    for default in "${defaults[@]}"; do
        if echo "$var_value" | grep -q "$default"; then
            log "${RED}✗ $var_name contains default/placeholder value: $default${NC}"
            echo "FAILED: $var_name - contains default value: $default" >> "$REPORT_FILE"
            ERRORS=$((ERRORS + 1))
            return 1
        fi
    done
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
    echo "PASSED: $var_name - no default values" >> "$REPORT_FILE"
    return 0
}

# Function to check format pattern
check_format() {
    local var_name=$1
    local pattern=$2
    local var_value="${!var_name}"
    local description=${3:-"format"}
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [ -z "$var_value" ]; then
        return 0
    fi

    if ! echo "$var_value" | grep -qE "$pattern"; then
        log "${RED}✗ $var_name does not match expected $description${NC}"
        echo "FAILED: $var_name - invalid $description" >> "$REPORT_FILE"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
    echo "PASSED: $var_name - valid $description" >> "$REPORT_FILE"
    return 0
}

# Function to test database connectivity (optional)
test_database_connection() {
    if [ "$1" = "--skip-connectivity" ]; then
        return 0
    fi

    log ""
    log "${BLUE}=== Testing Database Connectivity ===${NC}"
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))

    if [ -z "$DATABASE_URL" ]; then
        log "${YELLOW}⚠ Skipping database test - DATABASE_URL not set${NC}"
        echo "SKIPPED: Database connectivity - URL not set" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
        return 0
    fi

    # Try to connect using mongosh or mongo
    if command -v mongosh &> /dev/null; then
        if mongosh "$DATABASE_URL" --eval "db.adminCommand('ping')" &> /dev/null; then
            log "${GREEN}✓ Database connection successful${NC}"
            echo "PASSED: Database connectivity - successful" >> "$REPORT_FILE"
            CHECKS_PASSED=$((CHECKS_PASSED + 1))
            return 0
        else
            log "${YELLOW}⚠ Database connection failed (non-critical)${NC}"
            echo "WARNING: Database connectivity - failed" >> "$REPORT_FILE"
            WARNINGS=$((WARNINGS + 1))
            return 0
        fi
    else
        log "${YELLOW}⚠ mongosh not installed - skipping database test${NC}"
        echo "SKIPPED: Database connectivity - mongosh not installed" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
    fi
}

# Function to test API connectivity (optional)
test_api_connectivity() {
    if [ "$1" = "--skip-connectivity" ]; then
        return 0
    fi

    log ""
    log "${BLUE}=== Testing External API Connectivity ===${NC}"

    # Test OpenAI API if key is set
    if [ ! -z "$OPENAI_API_KEY" ]; then
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        if curl -s -H "Authorization: Bearer $OPENAI_API_KEY" \
            https://api.openai.com/v1/models &> /dev/null; then
            log "${GREEN}✓ OpenAI API connection successful${NC}"
            echo "PASSED: OpenAI API connectivity" >> "$REPORT_FILE"
            CHECKS_PASSED=$((CHECKS_PASSED + 1))
        else
            log "${YELLOW}⚠ OpenAI API connection failed (check key)${NC}"
            echo "WARNING: OpenAI API connectivity - failed" >> "$REPORT_FILE"
            WARNINGS=$((WARNINGS + 1))
        fi
    fi

    # Test Binance API if keys are set
    if [ ! -z "$BINANCE_API_KEY" ]; then
        TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
        local BINANCE_URL="https://api.binance.com/api/v3/ping"
        if [ "${BINANCE_TESTNET}" = "true" ]; then
            BINANCE_URL="https://testnet.binance.vision/api/v3/ping"
        fi

        if curl -s "$BINANCE_URL" &> /dev/null; then
            log "${GREEN}✓ Binance API connection successful${NC}"
            echo "PASSED: Binance API connectivity" >> "$REPORT_FILE"
            CHECKS_PASSED=$((CHECKS_PASSED + 1))
        else
            log "${YELLOW}⚠ Binance API connection failed${NC}"
            echo "WARNING: Binance API connectivity - failed" >> "$REPORT_FILE"
            WARNINGS=$((WARNINGS + 1))
        fi
    fi
}

log "${BLUE}=== Core Services ===${NC}"
check_required "BINANCE_API_KEY"
check_length "BINANCE_API_KEY" 32
check_not_default "BINANCE_API_KEY" "your-binance-api-key" "your_api_key" "changeme"

check_required "BINANCE_SECRET_KEY"
check_length "BINANCE_SECRET_KEY" 32
check_not_default "BINANCE_SECRET_KEY" "your-binance-secret-key" "your_secret" "changeme"

check_required "DATABASE_URL"
check_not_default "DATABASE_URL" "mongodb://admin:password@" "your-username" "your-password" "your-cluster"
check_format "DATABASE_URL" "^mongodb" "MongoDB connection string"

log ""
log "${BLUE}=== Security Tokens ===${NC}"
check_required "INTER_SERVICE_TOKEN"
check_length "INTER_SERVICE_TOKEN" 32
check_not_default "INTER_SERVICE_TOKEN" "generate-a-secure" "your-token" "changeme"

check_required "RUST_API_KEY"
check_length "RUST_API_KEY" 32
check_not_default "RUST_API_KEY" "generate-a-secure" "your-token" "changeme"

check_required "PYTHON_API_KEY"
check_length "PYTHON_API_KEY" 32
check_not_default "PYTHON_API_KEY" "generate-a-secure" "your-token" "changeme"

check_required "JWT_SECRET"
check_length "JWT_SECRET" 64
check_not_default "JWT_SECRET" "generate-a-secure" "your-secret" "changeme"

check_required "DASHBOARD_SESSION_SECRET"
check_length "DASHBOARD_SESSION_SECRET" 32
check_not_default "DASHBOARD_SESSION_SECRET" "generate-a-secure" "your-secret" "changeme"

log ""
log "${BLUE}=== Optional Services ===${NC}"
check_optional "OPENAI_API_KEY"
if [ ! -z "$OPENAI_API_KEY" ]; then
    check_format "OPENAI_API_KEY" "^sk-" "OpenAI API key format"
    check_not_default "OPENAI_API_KEY" "your-openai-api-key" "changeme"
fi

check_optional "REDIS_PASSWORD"
if [ ! -z "$REDIS_PASSWORD" ]; then
    check_not_default "REDIS_PASSWORD" "your-secure-password" "changeme"
fi

check_optional "MONGO_ROOT_PASSWORD"
if [ ! -z "$MONGO_ROOT_PASSWORD" ]; then
    check_not_default "MONGO_ROOT_PASSWORD" "your-mongo-root-password" "changeme"
fi

log ""
log "${BLUE}=== Configuration Settings ===${NC}"

# Check BINANCE_TESTNET
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ "${BINANCE_TESTNET}" = "false" ]; then
    log "${YELLOW}⚠ BINANCE_TESTNET is set to 'false' - LIVE TRADING MODE${NC}"
    log "${YELLOW}  Make sure this is intentional!${NC}"
    echo "WARNING: BINANCE_TESTNET - LIVE TRADING MODE ENABLED" >> "$REPORT_FILE"
    WARNINGS=$((WARNINGS + 1))
else
    log "${GREEN}✓ BINANCE_TESTNET is 'true' - safe testnet mode${NC}"
    echo "PASSED: BINANCE_TESTNET - testnet mode" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
fi

# Check TRADING_ENABLED
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ "${TRADING_ENABLED}" = "true" ]; then
    log "${YELLOW}⚠ TRADING_ENABLED is set to 'true'${NC}"
    log "${YELLOW}  Trading is ENABLED!${NC}"
    echo "WARNING: TRADING_ENABLED - trading enabled" >> "$REPORT_FILE"
    WARNINGS=$((WARNINGS + 1))
else
    log "${GREEN}✓ TRADING_ENABLED is 'false' - safe mode${NC}"
    echo "PASSED: TRADING_ENABLED - safe mode" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
fi

# Check LOG_LEVEL
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ ! -z "$LOG_LEVEL" ]; then
    case "$LOG_LEVEL" in
        debug|DEBUG|info|INFO|warning|WARNING|error|ERROR|critical|CRITICAL)
            log "${GREEN}✓ LOG_LEVEL is valid: $LOG_LEVEL${NC}"
            echo "PASSED: LOG_LEVEL - valid value" >> "$REPORT_FILE"
            CHECKS_PASSED=$((CHECKS_PASSED + 1))
            ;;
        *)
            log "${YELLOW}⚠ LOG_LEVEL has unusual value: $LOG_LEVEL${NC}"
            echo "WARNING: LOG_LEVEL - unusual value: $LOG_LEVEL" >> "$REPORT_FILE"
            WARNINGS=$((WARNINGS + 1))
            ;;
    esac
else
    log "${YELLOW}⚠ LOG_LEVEL is not set${NC}"
    echo "WARNING: LOG_LEVEL - not set" >> "$REPORT_FILE"
    WARNINGS=$((WARNINGS + 1))
fi

# Check PORT availability
log ""
log "${BLUE}=== Port Availability ===${NC}"
for port in 3000 8000 8080 27017; do
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        log "${YELLOW}⚠ Port $port is already in use${NC}"
        echo "WARNING: Port $port - already in use" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
    else
        log "${GREEN}✓ Port $port is available${NC}"
        echo "PASSED: Port $port - available" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    fi
done

# Test connectivity if not skipped
test_database_connection "$@"
test_api_connectivity "$@"

# Summary
log ""
log "=========================================="
log "Validation Summary"
log "=========================================="
log "Total Checks: $TOTAL_CHECKS"
log "Passed: ${GREEN}$CHECKS_PASSED${NC}"
log "Warnings: ${YELLOW}$WARNINGS${NC}"
log "Errors: ${RED}$ERRORS${NC}"
log ""

# Write summary to report
cat >> "$REPORT_FILE" << EOF

========================================
SUMMARY
========================================
Total Checks: $TOTAL_CHECKS
Passed: $CHECKS_PASSED
Warnings: $WARNINGS
Errors: $ERRORS
Success Rate: $(( CHECKS_PASSED * 100 / TOTAL_CHECKS ))%

EOF

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    log "${GREEN}✓ All checks passed!${NC}"
    echo "STATUS: PASSED - All checks successful" >> "$REPORT_FILE"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    log "${YELLOW}⚠ Validation completed with $WARNINGS warning(s)${NC}"
    log ""
    log "You can proceed, but please review the warnings above."
    echo "STATUS: WARNING - Validation passed with warnings" >> "$REPORT_FILE"
    exit 2
else
    log "${RED}✗ Validation failed with $ERRORS error(s) and $WARNINGS warning(s)${NC}"
    log ""
    log "Please fix the errors above before starting services."
    log ""
    log "Quick fixes:"
    log "  - Run './scripts/generate-secrets.sh' to generate secure tokens"
    log "  - Update BINANCE_API_KEY and BINANCE_SECRET_KEY with real values"
    log "  - Ensure DATABASE_URL points to your MongoDB instance"
    echo "STATUS: FAILED - Critical errors found" >> "$REPORT_FILE"
    exit 1
fi
