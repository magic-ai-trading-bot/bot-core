#!/bin/bash

# Real-time Service Monitoring Dashboard
# Usage: ./scripts/monitor-dashboard.sh

BOLD='\033[1m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Service URLs
RUST_CORE_URL="http://localhost:8080"
FRONTEND_URL="http://localhost:3000"
MCP_URL="http://localhost:8090"

# Function to check service health
check_service() {
    local url=$1
    local path=${2:-"/api/health"}

    if curl -s -f "${url}${path}" > /dev/null 2>&1; then
        echo -e "${GREEN}●${NC}"
    else
        echo -e "${RED}●${NC}"
    fi
}

# Main monitoring loop
while true; do
    clear

    # Header
    echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${CYAN}║              BOT-CORE SERVICE MONITORING DASHBOARD             ║${NC}"
    echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BOLD}Time:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""

    # Service Status
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}SERVICE STATUS${NC}"
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    RUST_STATUS=$(check_service "$RUST_CORE_URL" "/api/health")
    FRONTEND_STATUS=$(check_service "$FRONTEND_URL" "/")
    MCP_STATUS=$(check_service "$MCP_URL" "/health")

    echo -e "  ${RUST_STATUS} Rust Core Engine    ${CYAN}http://localhost:8080${NC}"
    echo -e "  ${FRONTEND_STATUS} Frontend Dashboard  ${CYAN}http://localhost:3000${NC}"
    echo -e "  ${MCP_STATUS} MCP Server          ${CYAN}http://localhost:8090${NC}"
    echo ""

    # Rust API stats
    HEALTH=$(curl -s "${RUST_CORE_URL}/api/health" 2>/dev/null)

    if [ -n "$HEALTH" ] && command -v jq &> /dev/null; then
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BOLD}RUST ENGINE STATUS${NC}"
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        STATUS=$(echo "$HEALTH" | jq -r '.status // "unknown"')
        VERSION=$(echo "$HEALTH" | jq -r '.version // "unknown"')
        UPTIME=$(echo "$HEALTH" | jq -r '.uptime_seconds // 0')

        echo -e "  ${BOLD}Status:${NC}    ${GREEN}${STATUS}${NC}"
        echo -e "  ${BOLD}Version:${NC}   ${CYAN}${VERSION}${NC}"
        echo -e "  ${BOLD}Uptime:${NC}    ${CYAN}${UPTIME}s${NC}"
        echo ""
    fi

    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Press Ctrl+C to exit | Auto-refresh every 10 seconds${NC}"
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    sleep 10
done
