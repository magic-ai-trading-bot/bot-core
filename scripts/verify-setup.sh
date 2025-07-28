#!/bin/bash

# Verify Bot Core Setup Script
# Checks all configurations and dependencies

set -e

echo "🔍 Bot Core Configuration Verification"
echo "====================================="

# Colors
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
NC='\033[0m'

# Function to check if file exists
check_file() {
    if [ -f "$1" ]; then
        echo -e "${GREEN}✓${NC} $1 exists"
        return 0
    else
        echo -e "${RED}✗${NC} $1 missing"
        return 1
    fi
}

# Function to check if directory exists
check_dir() {
    if [ -d "$1" ]; then
        echo -e "${GREEN}✓${NC} $1 directory exists"
        return 0
    else
        echo -e "${RED}✗${NC} $1 directory missing"
        return 1
    fi
}

# Function to check Docker
check_docker() {
    if command -v docker &> /dev/null && docker ps &> /dev/null; then
        echo -e "${GREEN}✓${NC} Docker is running"
        return 0
    else
        echo -e "${RED}✗${NC} Docker is not running"
        return 1
    fi
}

# 1. Check Docker
echo ""
echo "1. Checking Docker..."
check_docker

# 2. Check Core Files
echo ""
echo "2. Checking Core Configuration Files..."
check_file ".env.example"
check_file "docker-compose.yml"
check_file "docker-compose.prod.yml"
# docker-compose.replicas.yml removed (PostgreSQL not needed)
check_file "Makefile"

# 3. Check Service Configs
echo ""
echo "3. Checking Service Configurations..."
check_dir "rust-core-engine"
check_dir "python-ai-service"
check_dir "nextjs-ui-dashboard"

# 4. Check New Service Configs
echo ""
echo "4. Checking Enterprise Feature Configurations..."
check_dir "rabbitmq"
check_file "rabbitmq/rabbitmq.conf"
check_file "rabbitmq/definitions.json"

check_dir "kong"
check_file "kong/kong.yml"

check_dir "monitoring"
check_file "monitoring/prometheus.yml"
check_file "monitoring/alerts/alerts.yml"

check_dir "nginx"
check_file "nginx/nginx.conf"

check_dir "mongodb"
check_file "mongodb/replica.key"
check_file "mongodb/init-replica.js"

# PostgreSQL removed - using MongoDB only

# 5. Check Scripts
echo ""
echo "5. Checking Scripts..."
check_file "scripts/bot.sh"
check_file "scripts/generate-secrets.sh"
check_file "scripts/verify-setup.sh"

# 6. Check Environment
echo ""
echo "6. Checking Environment Setup..."
if [ -f ".env" ]; then
    echo -e "${GREEN}✓${NC} .env file exists"
    # Check for critical variables
    required_vars=("DATABASE_URL" "REDIS_PASSWORD" "RABBITMQ_PASSWORD" "KONG_DB_PASSWORD")
    for var in "${required_vars[@]}"; do
        if grep -q "^$var=" .env; then
            echo -e "${GREEN}✓${NC} $var is set"
        else
            echo -e "${YELLOW}⚠${NC} $var not found in .env"
        fi
    done
else
    echo -e "${YELLOW}⚠${NC} .env file not found - run: cp .env.example .env"
fi

# 7. Port Availability
echo ""
echo "7. Checking Port Availability..."
ports=(3000 8080 8000 5672 15672 8001 8100 9090 3001)
for port in "${ports[@]}"; do
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        echo -e "${RED}✗${NC} Port $port is already in use"
    else
        echo -e "${GREEN}✓${NC} Port $port is available"
    fi
done

# 8. Docker Network
echo ""
echo "8. Checking Docker Network..."
if docker network ls | grep -q bot-network; then
    echo -e "${GREEN}✓${NC} bot-network exists"
else
    echo -e "${YELLOW}⚠${NC} bot-network will be created on first run"
fi

# Summary
echo ""
echo "====================================="
echo "Summary:"

# Count issues
issues=0
if ! check_docker &> /dev/null; then ((issues++)); fi
if [ ! -f ".env" ]; then ((issues++)); fi

if [ $issues -eq 0 ]; then
    echo -e "${GREEN}✅ All checks passed! System is ready.${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Create .env file: cp .env.example .env"
    echo "2. Generate secrets: ./scripts/generate-secrets.sh"
    echo "3. Start services: ./scripts/bot.sh start --memory-optimized"
    echo ""
    echo "Optional enterprise features:"
    echo "- Redis: docker-compose --profile redis up -d"
    echo "- RabbitMQ: docker-compose --profile messaging up -d"
    echo "- Kong Gateway: docker-compose --profile api-gateway up -d"
    echo "- Monitoring: docker-compose --profile monitoring up -d"
else
    echo -e "${RED}❌ Found $issues issues. Please fix them before starting.${NC}"
fi