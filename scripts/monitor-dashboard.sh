#!/bin/bash

# Real-time Cost Monitoring Dashboard
# Usage: ./scripts/monitor-dashboard.sh

BOLD='\033[1m'
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Service URLs
PYTHON_AI_URL="http://localhost:8000"
RUST_CORE_URL="http://localhost:8080"
FRONTEND_URL="http://localhost:3000"

# Function to check service health
check_service() {
    local name=$1
    local url=$2

    if curl -s -f "${url}/health" > /dev/null 2>&1 || curl -s -f "${url}/api/health" > /dev/null 2>&1 || curl -s -f "${url}" > /dev/null 2>&1; then
        echo -e "${GREEN}●${NC}"
    else
        echo -e "${RED}●${NC}"
    fi
}

# Function to format number
format_number() {
    printf "%'.0f" "$1" 2>/dev/null || echo "$1"
}

# Main monitoring loop
while true; do
    clear

    # Header
    echo -e "${BOLD}${CYAN}╔════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${BOLD}${CYAN}║          BOT-CORE GPT-4 COST MONITORING DASHBOARD             ║${NC}"
    echo -e "${BOLD}${CYAN}╚════════════════════════════════════════════════════════════════╝${NC}"
    echo ""
    echo -e "${BOLD}Time:${NC} $(date '+%Y-%m-%d %H:%M:%S')"
    echo ""

    # Service Status
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}SERVICE STATUS${NC}"
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    PYTHON_STATUS=$(check_service "Python AI" "$PYTHON_AI_URL")
    RUST_STATUS=$(check_service "Rust Core" "$RUST_CORE_URL")
    FRONTEND_STATUS=$(check_service "Frontend" "$FRONTEND_URL")

    echo -e "  ${PYTHON_STATUS} Python AI Service   ${CYAN}http://localhost:8000${NC}"
    echo -e "  ${RUST_STATUS} Rust Core Engine    ${CYAN}http://localhost:8080${NC}"
    echo -e "  ${FRONTEND_STATUS} Frontend Dashboard  ${CYAN}http://localhost:3000${NC}"
    echo ""

    # Get cost statistics
    STATS=$(curl -s "${PYTHON_AI_URL}/ai/cost/statistics" 2>/dev/null)

    if [ -n "$STATS" ]; then
        # Parse JSON using jq if available, otherwise use grep/sed
        if command -v jq &> /dev/null; then
            TOTAL_REQUESTS=$(echo "$STATS" | jq -r '.session_statistics.total_requests // 0')
            TOTAL_INPUT=$(echo "$STATS" | jq -r '.session_statistics.total_input_tokens // 0')
            TOTAL_OUTPUT=$(echo "$STATS" | jq -r '.session_statistics.total_output_tokens // 0')
            TOTAL_COST_USD=$(echo "$STATS" | jq -r '.session_statistics.total_cost_usd // 0')
            TOTAL_COST_VND=$(echo "$STATS" | jq -r '.session_statistics.total_cost_vnd // 0')
            AVG_COST=$(echo "$STATS" | jq -r '.session_statistics.average_cost_per_request_usd // 0')
            AVG_TOKENS=$(echo "$STATS" | jq -r '.session_statistics.average_tokens_per_request // 0')

            DAILY_USD=$(echo "$STATS" | jq -r '.projections.estimated_daily_cost_usd // 0')
            DAILY_VND=$(echo "$STATS" | jq -r '.projections.estimated_daily_cost_vnd // 0')
            MONTHLY_USD=$(echo "$STATS" | jq -r '.projections.estimated_monthly_cost_usd // 0')
            MONTHLY_VND=$(echo "$STATS" | jq -r '.projections.estimated_monthly_cost_vnd // 0')

            INTERVAL=$(echo "$STATS" | jq -r '.configuration.analysis_interval_minutes // 0')
            CACHE_DURATION=$(echo "$STATS" | jq -r '.configuration.cache_duration_minutes // 0')
            MAX_TOKENS=$(echo "$STATS" | jq -r '.configuration.max_tokens // 0')
            SAVINGS=$(echo "$STATS" | jq -r '.optimization_status.estimated_savings_percent // 0')
        else
            # Fallback parsing without jq
            TOTAL_REQUESTS=$(echo "$STATS" | grep -o '"total_requests": [0-9]*' | grep -o '[0-9]*')
            TOTAL_COST_USD=$(echo "$STATS" | grep -o '"total_cost_usd": [0-9.]*' | grep -o '[0-9.]*')
            DAILY_USD=$(echo "$STATS" | grep -o '"estimated_daily_cost_usd": [0-9.]*' | grep -o '[0-9.]*')
            MONTHLY_USD=$(echo "$STATS" | grep -o '"estimated_monthly_cost_usd": [0-9.]*' | grep -o '[0-9.]*')
        fi

        # Session Statistics
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BOLD}SESSION STATISTICS${NC}"
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        echo -e "  ${BOLD}Total Requests:${NC}        ${GREEN}$(format_number $TOTAL_REQUESTS)${NC}"
        echo -e "  ${BOLD}Input Tokens:${NC}          ${CYAN}$(format_number $TOTAL_INPUT)${NC} tokens"
        echo -e "  ${BOLD}Output Tokens:${NC}         ${CYAN}$(format_number $TOTAL_OUTPUT)${NC} tokens"
        echo -e "  ${BOLD}Total Cost (USD):${NC}      ${MAGENTA}\$${TOTAL_COST_USD}${NC}"
        echo -e "  ${BOLD}Total Cost (VND):${NC}      ${MAGENTA}$(format_number $TOTAL_COST_VND)${NC} VNĐ"
        echo ""
        echo -e "  ${BOLD}Avg Cost/Request:${NC}      ${GREEN}\$$(printf '%.5f' $AVG_COST)${NC}"
        echo -e "  ${BOLD}Avg Tokens/Request:${NC}    ${CYAN}$(printf '%.0f' $AVG_TOKENS)${NC} tokens"
        echo ""

        # Cost Projections
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BOLD}COST PROJECTIONS${NC}"
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        # Color based on cost
        if (( $(echo "$DAILY_USD > 2.0" | bc -l 2>/dev/null || echo 0) )); then
            DAILY_COLOR=$RED
        elif (( $(echo "$DAILY_USD > 1.0" | bc -l 2>/dev/null || echo 0) )); then
            DAILY_COLOR=$YELLOW
        else
            DAILY_COLOR=$GREEN
        fi

        if (( $(echo "$MONTHLY_USD > 50.0" | bc -l 2>/dev/null || echo 0) )); then
            MONTHLY_COLOR=$RED
        elif (( $(echo "$MONTHLY_USD > 30.0" | bc -l 2>/dev/null || echo 0) )); then
            MONTHLY_COLOR=$YELLOW
        else
            MONTHLY_COLOR=$GREEN
        fi

        echo -e "  ${BOLD}Daily Cost (USD):${NC}      ${DAILY_COLOR}\$$(printf '%.2f' $DAILY_USD)${NC}"
        echo -e "  ${BOLD}Daily Cost (VND):${NC}      ${DAILY_COLOR}$(format_number $DAILY_VND)${NC} VNĐ"
        echo ""
        echo -e "  ${BOLD}Monthly Cost (USD):${NC}    ${MONTHLY_COLOR}\$$(printf '%.2f' $MONTHLY_USD)${NC}"
        echo -e "  ${BOLD}Monthly Cost (VND):${NC}    ${MONTHLY_COLOR}$(format_number $MONTHLY_VND)${NC} VNĐ"
        echo ""

        # Configuration
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
        echo -e "${BOLD}OPTIMIZATION CONFIGURATION${NC}"
        echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

        echo -e "  ${BOLD}Analysis Interval:${NC}     ${CYAN}${INTERVAL}${NC} minutes ${GREEN}(optimized)${NC}"
        echo -e "  ${BOLD}Cache Duration:${NC}        ${CYAN}${CACHE_DURATION}${NC} minutes ${GREEN}(optimized)${NC}"
        echo -e "  ${BOLD}Max Tokens:${NC}            ${CYAN}${MAX_TOKENS}${NC} tokens ${GREEN}(optimized)${NC}"
        echo -e "  ${BOLD}Estimated Savings:${NC}     ${GREEN}${SAVINGS}%${NC}"
        echo ""

    else
        echo -e "${RED}⚠️  Unable to fetch cost statistics${NC}"
        echo -e "${YELLOW}Make sure Python AI Service is running on port 8000${NC}"
        echo ""
    fi

    # Recent Cost Logs
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${BOLD}RECENT COST LOGS (Last 5)${NC}"
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    if command -v docker &> /dev/null; then
        docker logs python-ai-service 2>&1 | grep "💰 Cost" | tail -5 | while read line; do
            echo -e "  ${GREEN}${line}${NC}"
        done
    else
        echo -e "${YELLOW}  Docker not available${NC}"
    fi

    echo ""
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
    echo -e "${CYAN}Press Ctrl+C to exit | Auto-refresh every 10 seconds${NC}"
    echo -e "${BOLD}${YELLOW}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"

    sleep 10
done
