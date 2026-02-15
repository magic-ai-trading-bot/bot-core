#!/bin/bash
# Master Initialization Script for Core Services
# Waits for services to be healthy and seeds initial data

set -e

echo "================================================"
echo "Bot Core - Service Initialization"
echo "================================================"
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

# Function to wait for service
wait_for_service() {
    local service=$1
    local port=$2
    local max_wait=$3

    echo -e "${YELLOW}Waiting for $service to be ready on port $port...${NC}"

    count=0
    until nc -z localhost $port 2>/dev/null || [ $count -eq $max_wait ]; do
        sleep 2
        count=$((count + 1))
        if [ $((count % 5)) -eq 0 ]; then
            echo "  Still waiting... ($count/${max_wait})"
        fi
    done

    if [ $count -eq $max_wait ]; then
        echo -e "${RED}$service did not start in time${NC}"
        return 1
    fi

    echo -e "${GREEN}$service is ready!${NC}"
    echo ""
}

# Main initialization flow
echo "Step 1/2: Waiting for core services..."
wait_for_service "MongoDB" 27017 30
wait_for_service "Rust Core Engine" 8080 60
wait_for_service "Python AI Service" 8000 60

echo "Step 2/2: Seeding MongoDB..."
if [ -f "scripts/init-mongodb-seed.sh" ]; then
    bash scripts/init-mongodb-seed.sh || echo -e "${YELLOW}Seed script skipped (may already exist)${NC}"
fi
echo ""

echo "================================================"
echo "All Services Initialized Successfully!"
echo "================================================"
echo ""
echo "Service Access URLs:"
echo ""
echo "  Rust API:       http://localhost:8080/api/health"
echo "  Python AI:      http://localhost:8000/health"
echo "  Frontend:       http://localhost:3000"
echo "  MCP Server:     http://localhost:8090/health"
echo ""
echo "================================================"
