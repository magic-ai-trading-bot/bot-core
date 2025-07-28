#!/bin/bash

# Demo script to showcase Bot Core features
# This script demonstrates different deployment options

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
PURPLE='\033[0;35m'
NC='\033[0m'

print_demo() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE}🎯 BOT CORE DEMO${NC}"
    echo -e "${PURPLE}========================================${NC}"
    echo ""
}

print_option() {
    echo -e "${BLUE}$1${NC}"
    echo -e "${YELLOW}$2${NC}"
    echo ""
}

print_demo

echo -e "${GREEN}Welcome to Bot Core - Cryptocurrency Trading Platform${NC}"
echo ""
echo "This demo will show you different ways to run the system:"
echo ""

print_option "1. Basic Mode (7.5/10)" "
   ./scripts/bot.sh start
   - Core trading engine
   - AI predictions
   - Web dashboard"

print_option "2. Memory Optimized Mode" "
   ./scripts/bot.sh start --memory-optimized
   - Same features as Basic
   - Optimized for < 4GB RAM systems"

print_option "3. Development Mode" "
   ./scripts/bot.sh dev
   - Hot reload enabled
   - Debug logging
   - Source code mounting"

print_option "4. Enterprise Mode (10/10)" "
   ./scripts/bot.sh start --with-enterprise
   - Everything in Basic +
   - Redis caching
   - RabbitMQ message queue
   - Kong API Gateway
   - Prometheus & Grafana monitoring"

print_option "5. Custom Features" "
   ./scripts/bot.sh start --with-redis --with-monitoring
   - Pick specific features
   - Mix and match as needed"

echo -e "${GREEN}Quick Commands:${NC}"
echo "  Status:  ./scripts/bot.sh status"
echo "  Logs:    ./scripts/bot.sh logs"
echo "  Stop:    ./scripts/bot.sh stop"
echo "  Verify:  ./scripts/bot.sh verify"
echo ""

echo -e "${YELLOW}First Time Setup:${NC}"
echo "  1. cp .env.example .env"
echo "  2. ./scripts/generate-secrets.sh"
echo "  3. Edit .env with your API keys"
echo "  4. ./scripts/bot.sh verify"
echo "  5. ./scripts/bot.sh start --with-enterprise"
echo ""

echo -e "${GREEN}Try it now! Which mode would you like to demo?${NC}"