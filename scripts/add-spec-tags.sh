#!/bin/bash

# Add @spec tags to source code files
# This script adds traceability tags linking code to specifications

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo "═══════════════════════════════════════════════════════════════"
echo "  Adding @spec Tags to Source Code"
echo "═══════════════════════════════════════════════════════════════"
echo ""
echo "Project Root: $PROJECT_ROOT"
echo ""

# Color codes
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Counters
rust_files=0
python_files=0
ts_files=0

# Function to add @spec tag to a file at a specific line
add_spec_tag() {
    local file="$1"
    local line_num="$2"
    local spec_id="$3"
    local description="$4"
    local ref_doc="$5"
    local test_cases="$6"

    # Detect file type and use appropriate comment syntax
    if [[ "$file" == *.rs ]]; then
        comment_start="//"
        ((rust_files++))
    elif [[ "$file" == *.py ]]; then
        comment_start="#"
        ((python_files++))
    elif [[ "$file" == *.ts ]] || [[ "$file" == *.tsx ]]; then
        comment_start="//"
        ((ts_files++))
    else
        return
    fi

    # Create the tag comment
    tag="$comment_start @spec:$spec_id - $description\n$comment_start @ref:$ref_doc\n$comment_start @test:$test_cases"

    # Check if tag already exists
    if grep -q "@spec:$spec_id" "$file" 2>/dev/null; then
        echo -e "${YELLOW}  ⊙ Tag already exists: $spec_id in $(basename $file)${NC}"
        return
    fi

    # File exists check
    if [[ ! -f "$file" ]]; then
        echo -e "${YELLOW}  ⚠ File not found: $file${NC}"
        return
    fi

    echo -e "${GREEN}  ✓ Adding tag: $spec_id to $(basename $file):$line_num${NC}"
}

echo -e "${BLUE}Phase 1: Adding tags to Rust Core Engine files${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Authentication module
echo -e "\n${BLUE}Module: Authentication${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/jwt.rs" 45 "FR-AUTH-001" "JWT Token Generation" "COMP-RUST-AUTH.md#jwt-implementation" "TC-AUTH-001, TC-AUTH-002, TC-AUTH-003"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/jwt.rs" 89 "FR-AUTH-005" "Token Expiration" "COMP-RUST-AUTH.md#token-expiration" "TC-AUTH-011, TC-AUTH-012"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/handlers.rs" 78 "FR-AUTH-002" "User Registration" "API-RUST-CORE.md#authentication" "TC-AUTH-004, TC-AUTH-005"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/handlers.rs" 122 "FR-AUTH-003" "User Login" "API-RUST-CORE.md#authentication" "TC-AUTH-006, TC-AUTH-007, TC-AUTH-008"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/handlers.rs" 167 "FR-AUTH-007" "Profile Retrieval" "API-RUST-CORE.md#user-profile" "TC-AUTH-015, TC-AUTH-016"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/middleware.rs" 23 "FR-AUTH-004" "JWT Validation" "ARCH-SECURITY.md#authentication" "TC-AUTH-009, TC-AUTH-010"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/middleware.rs" 60 "FR-AUTH-008" "Authorization Middleware" "ARCH-SECURITY.md#authorization" "TC-AUTH-017, TC-AUTH-018, TC-AUTH-019"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/middleware.rs" 90 "FR-AUTH-009" "Role-Based Access Control" "ARCH-SECURITY.md#rbac" "TC-AUTH-020, TC-AUTH-021"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/auth/password.rs" 15 "FR-AUTH-006" "Password Hashing" "NFR-SECURITY.md#password-security" "TC-AUTH-013, TC-AUTH-014"

# Trading Engine module
echo -e "\n${BLUE}Module: Trading Engine${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/engine.rs" 150 "FR-TRADING-001" "Market Order Execution" "COMP-RUST-TRADING.md#order-execution" "TC-TRADING-001, TC-TRADING-002, TC-TRADING-003"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/engine.rs" 202 "FR-TRADING-006" "Market vs Limit Orders" "API-RUST-CORE.md#order-types" "TC-TRADING-035, TC-TRADING-036"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/position_manager.rs" 45 "FR-TRADING-002" "Position Management" "COMP-RUST-TRADING.md#position-management" "TC-TRADING-010, TC-TRADING-011, TC-TRADING-012"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/orderbook.rs" 30 "FR-TRADING-003" "Order Book Processing" "ARCH-DATA-FLOW.md#orderbook" "TC-TRADING-020, TC-TRADING-021"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/history.rs" 22 "FR-TRADING-004" "Trade History Tracking" "DB-SCHEMA.md#trades-collection" "TC-TRADING-025, TC-TRADING-026"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/orders.rs" 78 "FR-TRADING-007" "Stop-Loss Orders" "COMP-RUST-TRADING.md#stop-loss" "TC-TRADING-040, TC-TRADING-041"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/orders.rs" 127 "FR-TRADING-008" "Take-Profit Orders" "COMP-RUST-TRADING.md#take-profit" "TC-TRADING-042, TC-TRADING-043"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/validator.rs" 34 "FR-TRADING-009" "Trade Validation" "ARCH-SECURITY.md#input-validation" "TC-TRADING-045, TC-TRADING-046"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/logger.rs" 15 "FR-TRADING-010" "Trade Execution Logging" "MON-LOGGING.md#trade-logs" "TC-TRADING-050, TC-TRADING-051"

# Risk Management module
echo -e "\n${BLUE}Module: Risk Management${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/risk_manager.rs" 45 "FR-RISK-001" "Position Size Limits" "COMP-RUST-TRADING.md#risk-limits" "TC-TRADING-004, TC-TRADING-005"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/risk_manager.rs" 80 "FR-RISK-002" "Max Daily Loss" "COMP-RUST-TRADING.md#daily-loss-limit" "TC-TRADING-006, TC-TRADING-007"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/risk_manager.rs" 114 "FR-RISK-003" "Max Open Positions" "COMP-RUST-TRADING.md#position-limits" "TC-TRADING-008, TC-TRADING-009"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/risk_manager.rs" 147 "FR-RISK-004" "Risk Validation" "ARCH-SECURITY.md#risk-validation" "TC-TRADING-047, TC-TRADING-048"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/risk_manager.rs" 191 "FR-RISK-005" "Emergency Stop" "COMP-RUST-TRADING.md#emergency-controls" "TC-TRADING-049"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/trading/risk_manager.rs" 225 "FR-RISK-006" "Exposure Limits" "COMP-RUST-TRADING.md#exposure-management" "TC-TRADING-052, TC-TRADING-053"

# Binance Integration
echo -e "\n${BLUE}Module: Binance Integration${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/binance/client.rs" 89 "FR-TRADING-005" "Binance Integration" "API-RUST-CORE.md#binance-api" "TC-TRADING-030, TC-TRADING-031, TC-TRADING-032"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/binance/websocket.rs" 89 "FR-WEBSOCKET-001" "Binance WebSocket Connection" "API-WEBSOCKET.md#binance-connection" "TC-INTEGRATION-008, TC-INTEGRATION-009"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/binance/reconnection.rs" 23 "FR-WEBSOCKET-005" "Reconnection Logic" "NFR-RELIABILITY.md#reconnection" "TC-INTEGRATION-016, TC-INTEGRATION-017"

# Paper Trading module
echo -e "\n${BLUE}Module: Paper Trading${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/engine.rs" 56 "FR-PAPER-001" "Paper Trading Engine" "COMP-RUST-TRADING.md#paper-trading" "TC-INTEGRATION-025, TC-INTEGRATION-026"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/portfolio.rs" 23 "FR-PAPER-002" "Virtual Portfolio" "COMP-RUST-TRADING.md#virtual-portfolio" "TC-INTEGRATION-027, TC-INTEGRATION-028"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/portfolio.rs" 38 "FR-PORTFOLIO-001" "Portfolio Creation" "DB-SCHEMA.md#portfolio-collection" "TC-TRADING-013, TC-TRADING-014"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/portfolio.rs" 91 "FR-PORTFOLIO-002" "Balance Tracking" "COMP-RUST-TRADING.md#balance-tracking" "TC-TRADING-015, TC-TRADING-016"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/portfolio.rs" 136 "FR-PORTFOLIO-003" "P&L Calculation" "API-RUST-CORE.md#portfolio-api" "TC-TRADING-017, TC-TRADING-018"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/portfolio.rs" 180 "FR-PORTFOLIO-004" "Asset Allocation" "COMP-RUST-TRADING.md#asset-allocation" "TC-TRADING-019"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/execution.rs" 34 "FR-PAPER-003" "Simulated Execution" "ARCH-DATA-FLOW.md#paper-trading-flow" "TC-INTEGRATION-029, TC-INTEGRATION-030"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/history.rs" 18 "FR-PAPER-004" "Paper Trade History" "DB-SCHEMA.md#paper-trades-collection" "TC-INTEGRATION-031"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/analytics.rs" 25 "FR-PORTFOLIO-005" "Historical Performance" "API-RUST-CORE.md#portfolio-analytics" "TC-INTEGRATION-020, TC-INTEGRATION-021"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/analytics.rs" 91 "FR-PORTFOLIO-006" "Portfolio Analytics" "COMP-RUST-TRADING.md#analytics" "TC-INTEGRATION-022"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/analytics.rs" 25 "FR-PAPER-005" "Performance Analytics" "API-RUST-CORE.md#paper-trading-analytics" "TC-INTEGRATION-032, TC-INTEGRATION-033"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/mode_manager.rs" 22 "FR-PAPER-006" "Mode Switching" "COMP-RUST-TRADING.md#mode-switching" "TC-INTEGRATION-034"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/paper_trading/strategy_optimizer.rs" 56 "FR-STRATEGY-007" "Strategy Optimizer" "COMP-RUST-TRADING.md#strategy-optimization" "TC-TRADING-037, TC-TRADING-038"

# Trading Strategies
echo -e "\n${BLUE}Module: Trading Strategies${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/strategies/rsi_strategy.rs" 45 "FR-STRATEGY-001" "RSI Strategy" "COMP-RUST-TRADING.md#rsi-strategy" "TC-TRADING-022, TC-TRADING-023"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/strategies/macd_strategy.rs" 38 "FR-STRATEGY-002" "MACD Strategy" "COMP-RUST-TRADING.md#macd-strategy" "TC-TRADING-024"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/strategies/bollinger_strategy.rs" 42 "FR-STRATEGY-003" "Bollinger Bands Strategy" "COMP-RUST-TRADING.md#bollinger-strategy" "TC-TRADING-027"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/strategies/volume_strategy.rs" 35 "FR-STRATEGY-004" "Volume Strategy" "COMP-RUST-TRADING.md#volume-strategy" "TC-TRADING-028"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/strategies/strategy_engine.rs" 67 "FR-STRATEGY-005" "Strategy Parameters" "DB-SCHEMA.md#strategies-collection" "TC-TRADING-029"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/strategies/backtester.rs" 45 "FR-STRATEGY-006" "Strategy Backtesting" "COMP-RUST-TRADING.md#backtesting" "TC-TRADING-033, TC-TRADING-034"

# WebSocket module
echo -e "\n${BLUE}Module: WebSocket${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/websocket/server.rs" 45 "FR-WEBSOCKET-002" "Client-Server WebSocket" "API-WEBSOCKET.md#client-server-ws" "TC-INTEGRATION-010, TC-INTEGRATION-011"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/websocket/broadcast.rs" 28 "FR-WEBSOCKET-003" "Real-time Updates" "ARCH-DATA-FLOW.md#websocket-broadcast" "TC-INTEGRATION-012, TC-INTEGRATION-013"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/websocket/connection_manager.rs" 34 "FR-WEBSOCKET-004" "Connection Management" "NFR-RELIABILITY.md#connection-management" "TC-INTEGRATION-014, TC-INTEGRATION-015"

# Market Data module
echo -e "\n${BLUE}Module: Market Data${NC}"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/market_data/live_feed.rs" 45 "FR-MARKET-001" "Real-time Price Feed" "ARCH-DATA-FLOW.md#market-data-flow" "TC-INTEGRATION-001, TC-INTEGRATION-002"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/market_data/historical.rs" 34 "FR-MARKET-002" "Historical Data Retrieval" "API-RUST-CORE.md#market-data" "TC-INTEGRATION-003, TC-INTEGRATION-004"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/market_data/klines.rs" 28 "FR-MARKET-003" "Kline/Candlestick Data" "DB-SCHEMA.md#klines-collection" "TC-INTEGRATION-005"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/market_data/cache.rs" 23 "FR-MARKET-004" "Market Data Caching" "NFR-PERFORMANCE.md#caching-strategy" "TC-INTEGRATION-006"
add_spec_tag "$PROJECT_ROOT/rust-core-engine/src/market_data/validator.rs" 18 "FR-MARKET-005" "Data Validation" "ARCH-DATA-FLOW.md#data-validation" "TC-INTEGRATION-007"

echo -e "\n${BLUE}Phase 2: Python AI Service files (will be handled next)${NC}"
echo -e "${BLUE}Phase 3: Frontend TypeScript files (will be handled next)${NC}"

echo ""
echo "═══════════════════════════════════════════════════════════════"
echo "  Summary"
echo "═══════════════════════════════════════════════════════════════"
echo -e "Rust files tagged:       ${rust_files}"
echo -e "Python files tagged:     ${python_files}"
echo -e "TypeScript files tagged: ${ts_files}"
echo -e "Total files processed:   $((rust_files + python_files + ts_files))"
echo ""
echo -e "${GREEN}✓ Spec tagging preparation complete!${NC}"
echo ""
echo "Note: This script shows what tags will be added."
echo "      Run add-spec-tags-rust.sh to actually add tags to Rust files."
echo ""
