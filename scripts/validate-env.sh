#!/bin/bash

# Environment Variable Validation Script
# Checks that all required environment variables are set and valid

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Counter for errors
ERRORS=0
WARNINGS=0

echo "=========================================="
echo "Environment Variable Validation"
echo "=========================================="
echo ""

# Check if .env file exists
if [ ! -f .env ]; then
    echo -e "${RED}ERROR: .env file not found${NC}"
    echo "Please copy config.env to .env and fill in the required values."
    echo "Run: cp config.env .env"
    exit 1
fi

# Source the .env file
set -a
source .env 2>/dev/null || true
set +a

echo "Checking required environment variables..."
echo ""

# Function to check if variable is set and not empty
check_required() {
    local var_name=$1
    local var_value="${!var_name}"

    if [ -z "$var_value" ]; then
        echo -e "${RED}✗ $var_name is not set or empty${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    else
        echo -e "${GREEN}✓ $var_name is set${NC}"
        return 0
    fi
}

# Function to check optional variable
check_optional() {
    local var_name=$1
    local var_value="${!var_name}"

    if [ -z "$var_value" ]; then
        echo -e "${YELLOW}⚠ $var_name is not set (optional)${NC}"
        WARNINGS=$((WARNINGS + 1))
    else
        echo -e "${GREEN}✓ $var_name is set${NC}"
    fi
}

# Function to check variable length
check_length() {
    local var_name=$1
    local min_length=$2
    local var_value="${!var_name}"

    if [ ! -z "$var_value" ] && [ ${#var_value} -lt $min_length ]; then
        echo -e "${RED}✗ $var_name is too short (${#var_value} chars, minimum $min_length)${NC}"
        ERRORS=$((ERRORS + 1))
        return 1
    fi
    return 0
}

# Function to check for default/weak values
check_not_default() {
    local var_name=$1
    local var_value="${!var_name}"
    shift
    local defaults=("$@")

    for default in "${defaults[@]}"; do
        if [ "$var_value" = "$default" ]; then
            echo -e "${RED}✗ $var_name contains default/placeholder value: $default${NC}"
            ERRORS=$((ERRORS + 1))
            return 1
        fi
    done
    return 0
}

echo "=== Core Services ==="
check_required "BINANCE_API_KEY"
check_length "BINANCE_API_KEY" 32
check_not_default "BINANCE_API_KEY" "your_api_key_here" "changeme"

check_required "BINANCE_SECRET_KEY"
check_length "BINANCE_SECRET_KEY" 32
check_not_default "BINANCE_SECRET_KEY" "your_secret_key_here" "changeme"

check_required "DATABASE_URL"
check_not_default "DATABASE_URL" "mongodb://localhost:27017/bot_core"

echo ""
echo "=== Inter-Service Authentication ==="
check_required "INTER_SERVICE_TOKEN"
check_length "INTER_SERVICE_TOKEN" 32

check_required "RUST_API_KEY"
check_length "RUST_API_KEY" 32

check_required "PYTHON_API_KEY"
check_length "PYTHON_API_KEY" 32

echo ""
echo "=== Dashboard ==="
check_required "DASHBOARD_SESSION_SECRET"
check_length "DASHBOARD_SESSION_SECRET" 32

echo ""
echo "=== Optional Services ==="
check_optional "OPENAI_API_KEY"
check_optional "REDIS_PASSWORD"
check_optional "RABBITMQ_PASSWORD"
check_optional "KONG_DB_PASSWORD"
check_optional "GRAFANA_PASSWORD"

echo ""
echo "=== Configuration Settings ==="

# Check BINANCE_TESTNET
if [ "${BINANCE_TESTNET}" = "false" ]; then
    echo -e "${YELLOW}⚠ BINANCE_TESTNET is set to 'false' - LIVE TRADING MODE${NC}"
    echo -e "${YELLOW}  Make sure this is intentional!${NC}"
    WARNINGS=$((WARNINGS + 1))
else
    echo -e "${GREEN}✓ BINANCE_TESTNET is 'true' - safe testnet mode${NC}"
fi

# Check TRADING_ENABLED
if [ "${TRADING_ENABLED}" = "true" ]; then
    echo -e "${YELLOW}⚠ TRADING_ENABLED is set to 'true'${NC}"
    echo -e "${YELLOW}  Trading is ENABLED!${NC}"
    WARNINGS=$((WARNINGS + 1))
else
    echo -e "${GREEN}✓ TRADING_ENABLED is 'false' - safe mode${NC}"
fi

# Check LOG_LEVEL
if [ ! -z "$LOG_LEVEL" ]; then
    case "$LOG_LEVEL" in
        DEBUG|INFO|WARNING|ERROR|CRITICAL)
            echo -e "${GREEN}✓ LOG_LEVEL is valid: $LOG_LEVEL${NC}"
            ;;
        *)
            echo -e "${YELLOW}⚠ LOG_LEVEL has unusual value: $LOG_LEVEL${NC}"
            WARNINGS=$((WARNINGS + 1))
            ;;
    esac
fi

echo ""
echo "=========================================="
echo "Validation Summary"
echo "=========================================="

if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    echo -e "${GREEN}✓ All checks passed!${NC}"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    echo -e "${YELLOW}⚠ Validation completed with $WARNINGS warning(s)${NC}"
    echo ""
    echo "You can proceed, but please review the warnings above."
    exit 0
else
    echo -e "${RED}✗ Validation failed with $ERRORS error(s) and $WARNINGS warning(s)${NC}"
    echo ""
    echo "Please fix the errors above before starting services."
    echo ""
    echo "Quick fixes:"
    echo "  - Run 'make generate-secrets' to generate secure tokens"
    echo "  - Update BINANCE_API_KEY and BINANCE_SECRET_KEY with real values"
    echo "  - Ensure DATABASE_URL points to your MongoDB instance"
    exit 1
fi
