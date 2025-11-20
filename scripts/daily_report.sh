#!/bin/bash
#
# Daily Performance Report Generator
# Tự động tạo report hàng ngày về performance
#
# Usage:
#   ./scripts/daily_report.sh                  # Show today's report
#   ./scripts/daily_report.sh --email          # Email report (future)
#   ./scripts/daily_report.sh --week           # Show weekly report
#

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# API endpoints
RUST_API="http://localhost:8080"
PYTHON_API="http://localhost:8000"

echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║           📊 DAILY PERFORMANCE REPORT                                 ║${NC}"
echo -e "${MAGENTA}╠══════════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${MAGENTA}║   Date: $(date '+%Y-%m-%d %H:%M:%S')                                         ║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Check if services are running
echo -e "${BLUE}[INFO]${NC} Checking service health..."
if ! curl -s --max-time 5 "$RUST_API/api/health" > /dev/null 2>&1; then
    echo -e "${RED}❌ Rust API not responding${NC}"
    echo -e "${YELLOW}⚠️  Start services with: ./scripts/bot.sh start${NC}"
    exit 1
fi
echo -e "${GREEN}✅ Services are running${NC}"
echo ""

# Fetch portfolio data
echo -e "${BLUE}[INFO]${NC} Fetching portfolio data..."
PORTFOLIO=$(curl -s --max-time 10 "$RUST_API/api/paper-trading/portfolio" || echo "{}")

if [ "$PORTFOLIO" = "{}" ] || [ -z "$PORTFOLIO" ]; then
    echo -e "${YELLOW}⚠️  No portfolio data available yet${NC}"
    echo -e "${CYAN}💡 Make sure paper trading is enabled and has executed some trades${NC}"
    exit 0
fi

# Parse JSON (requires jq)
if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}⚠️  jq not installed, showing raw data:${NC}"
    echo "$PORTFOLIO" | python3 -m json.tool || echo "$PORTFOLIO"
    exit 0
fi

# Extract metrics
BALANCE=$(echo "$PORTFOLIO" | jq -r '.balance // 0')
INITIAL_BALANCE=$(echo "$PORTFOLIO" | jq -r '.initial_balance // 10000')
UNREALIZED_PNL=$(echo "$PORTFOLIO" | jq -r '.unrealized_pnl // 0')
TRADE_COUNT=$(echo "$PORTFOLIO" | jq -r '.closed_trades | length // 0')

# Calculate total return
TOTAL_RETURN=$(echo "scale=2; (($BALANCE - $INITIAL_BALANCE) / $INITIAL_BALANCE) * 100" | bc)

# Display Portfolio Summary
echo -e "${CYAN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${CYAN}║  💰 PORTFOLIO SUMMARY                                                ║${NC}"
echo -e "${CYAN}╠══════════════════════════════════════════════════════════════════════╣${NC}"
printf "${CYAN}║${NC}  Current Balance:      ${GREEN}\$%-12.2f${NC}                             ${CYAN}║${NC}\n" "$BALANCE"
printf "${CYAN}║${NC}  Initial Balance:      \$%-12.2f                             ${CYAN}║${NC}\n" "$INITIAL_BALANCE"
printf "${CYAN}║${NC}  Total Return:         ${GREEN}%+6.2f%%${NC}                                 ${CYAN}║${NC}\n" "$TOTAL_RETURN"
printf "${CYAN}║${NC}  Unrealized P&L:       ${YELLOW}\$%+10.2f${NC}                              ${CYAN}║${NC}\n" "$UNREALIZED_PNL"
printf "${CYAN}║${NC}  Total Trades:         %-3d                                      ${CYAN}║${NC}\n" "$TRADE_COUNT"
echo -e "${CYAN}╚══════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

# Calculate Win Rate if we have enough trades
if [ "$TRADE_COUNT" -ge 10 ]; then
    echo -e "${BLUE}[INFO]${NC} Calculating performance metrics..."

    # Extract closed trades and calculate metrics
    WINNING_TRADES=$(echo "$PORTFOLIO" | jq '[.closed_trades[] | select(.pnl > 0)] | length')
    WIN_RATE=$(echo "scale=1; ($WINNING_TRADES / $TRADE_COUNT) * 100" | bc)

    # Average profit
    AVG_PROFIT=$(echo "$PORTFOLIO" | jq '[.closed_trades[] | select(.pnl > 0) | .pnl_percentage] | add / length' 2>/dev/null || echo "0")

    echo -e "${GREEN}╔══════════════════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║  📈 PERFORMANCE METRICS                                              ║${NC}"
    echo -e "${GREEN}╠══════════════════════════════════════════════════════════════════════╣${NC}"
    printf "${GREEN}║${NC}  Win Rate:             "
    if (( $(echo "$WIN_RATE >= 70" | bc -l) )); then
        printf "${GREEN}%-6.1f%%${NC} 🎯 ${GREEN}(Target: 70%%)${NC}                  ${GREEN}║${NC}\n" "$WIN_RATE"
    elif (( $(echo "$WIN_RATE >= 65" | bc -l) )); then
        printf "${YELLOW}%-6.1f%%${NC} ⚠️  ${YELLOW}(Target: 70%%)${NC}                  ${GREEN}║${NC}\n" "$WIN_RATE"
    else
        printf "${RED}%-6.1f%%${NC} ❌ ${RED}(Target: 70%%)${NC}                  ${GREEN}║${NC}\n" "$WIN_RATE"
    fi

    printf "${GREEN}║${NC}  Avg Profit/Trade:     ${GREEN}%-5.2f%%${NC} (Target: 2.6%%)                   ${GREEN}║${NC}\n" "$AVG_PROFIT"
    printf "${GREEN}║${NC}  Winning Trades:       %-3d / %-3d                                  ${GREEN}║${NC}\n" "$WINNING_TRADES" "$TRADE_COUNT"
    echo -e "${GREEN}╚══════════════════════════════════════════════════════════════════════╝${NC}"
    echo ""

    # Performance Status
    if (( $(echo "$WIN_RATE >= 70" | bc -l) )); then
        echo -e "${GREEN}✅ Status: EXCELLENT${NC} - Meeting optimization targets!"
    elif (( $(echo "$WIN_RATE >= 65" | bc -l) )); then
        echo -e "${YELLOW}⚠️  Status: GOOD${NC} - Close to targets, keep monitoring"
    else
        echo -e "${RED}❌ Status: NEEDS ATTENTION${NC} - Below baseline performance"
    fi
else
    echo -e "${YELLOW}⏳ Performance metrics will be available after 10+ trades${NC}"
    echo -e "${CYAN}   Current trades: $TRADE_COUNT / 10${NC}"
fi

echo ""
echo -e "${MAGENTA}╔══════════════════════════════════════════════════════════════════════╗${NC}"
echo -e "${MAGENTA}║  📝 NEXT STEPS                                                        ║${NC}"
echo -e "${MAGENTA}╠══════════════════════════════════════════════════════════════════════╣${NC}"
echo -e "${MAGENTA}║${NC}  1. Monitor this report daily: ${CYAN}./scripts/daily_report.sh${NC}        ${MAGENTA}║${NC}"
echo -e "${MAGENTA}║${NC}  2. Full monitoring: ${CYAN}python3 scripts/monitor_performance.py${NC}      ${MAGENTA}║${NC}"
echo -e "${MAGENTA}║${NC}  3. Continuous monitoring: ${CYAN}... --continuous --alert${NC}              ${MAGENTA}║${NC}"
echo -e "${MAGENTA}║${NC}  4. View logs: ${CYAN}./scripts/bot.sh logs --service rust-core-engine${NC} ${MAGENTA}║${NC}"
echo -e "${MAGENTA}╚══════════════════════════════════════════════════════════════════════╝${NC}"
echo ""

echo -e "${BLUE}[INFO]${NC} Report generated successfully!"
echo -e "${CYAN}💡 Run this script daily to track your optimization progress${NC}"
echo ""
