#!/usr/bin/env bash
# @spec:FR-DB-008 - Database Validation Script
# @ref:specs/02-design/2.2-database/DB-SCHEMA.md
# MongoDB Database Validation Script for Bot Core
# This script validates database setup, schema, indexes, and configuration

set -euo pipefail

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MONGO_HOST="${MONGO_HOST:-localhost}"
MONGO_PORT="${MONGO_PORT:-27017}"
MONGO_DB="${MONGO_DB:-bot_core}"

# Load environment
if [[ -f "$PROJECT_ROOT/.env" ]]; then
    export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)
fi

MONGO_ROOT_USER="${MONGO_ROOT_USER:-admin}"
MONGO_ROOT_PASSWORD="${MONGO_ROOT_PASSWORD:-secure_mongo_password_change_me}"
MONGO_CONNECTION_STRING="mongodb://${MONGO_ROOT_USER}:${MONGO_ROOT_PASSWORD}@${MONGO_HOST}:${MONGO_PORT}/${MONGO_DB}?authSource=admin"

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
WARNING_TESTS=0

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[PASS]${NC} $1"; ((PASSED_TESTS++)); ((TOTAL_TESTS++)); }
log_error() { echo -e "${RED}[FAIL]${NC} $1"; ((FAILED_TESTS++)); ((TOTAL_TESTS++)); }
log_warning() { echo -e "${YELLOW}[WARN]${NC} $1"; ((WARNING_TESTS++)); ((TOTAL_TESTS++)); }

# Print banner
print_banner() {
    echo -e "${GREEN}"
    cat << "EOF"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║        Bot Core - Database Validation Suite              ║
║              Comprehensive Schema & Setup Check          ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

# Test MongoDB connection
test_connection() {
    log_info "Testing MongoDB connection..."

    if mongosh --quiet --host "$MONGO_HOST" --port "$MONGO_PORT" \
        --username "$MONGO_ROOT_USER" --password "$MONGO_ROOT_PASSWORD" \
        --authenticationDatabase admin \
        --eval "db.adminCommand('ping')" &> /dev/null; then
        log_success "MongoDB connection successful"
        return 0
    else
        log_error "MongoDB connection failed"
        return 1
    fi
}

# Test replica set
test_replica_set() {
    log_info "Testing replica set configuration..."

    local rs_status
    rs_status=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "try { rs.status().ok } catch(e) { 0 }" 2>&1 || echo "0")

    if [[ "$rs_status" == *"1"* ]]; then
        log_success "Replica set is healthy"
        return 0
    else
        log_error "Replica set is not properly configured"
        return 1
    fi
}

# Test database exists
test_database() {
    log_info "Testing database existence..."

    local db_name
    db_name=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "db.getName()" 2>&1)

    if [[ "$db_name" == *"$MONGO_DB"* ]]; then
        log_success "Database '$MONGO_DB' exists"
        return 0
    else
        log_error "Database '$MONGO_DB' not found"
        return 1
    fi
}

# Test critical collections
test_collections() {
    log_info "Testing collection existence..."

    local critical_collections=(
        "users" "trades" "positions" "market_data"
        "ml_models" "predictions" "system_config"
        "portfolio_snapshots" "risk_metrics" "strategy_configs"
        "sessions" "api_keys" "notifications"
        "paper_trading_accounts" "paper_trading_trades"
        "audit_logs" "ai_signals" "performance_metrics"
        "training_jobs" "market_indicators" "model_performance_history"
    )

    local missing_count=0

    for collection in "${critical_collections[@]}"; do
        local exists
        exists=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
            --eval "db.getCollectionNames().includes('$collection')" 2>&1)

        if [[ "$exists" == *"true"* ]]; then
            log_success "Collection '$collection' exists"
        else
            log_error "Collection '$collection' is missing"
            ((missing_count++))
        fi
    done

    if [[ $missing_count -eq 0 ]]; then
        return 0
    else
        return 1
    fi
}

# Test indexes
test_indexes() {
    log_info "Testing index existence..."

    # Test critical indexes
    local test_cases=(
        "users|{\"email\":1}|Email index"
        "trades|{\"user_id\":1,\"created_at\":-1}|Trades user+date index"
        "positions|{\"user_id\":1,\"symbol\":1}|Positions unique index"
        "ml_models|{\"model_name\":1,\"version\":1}|ML models unique index"
        "predictions|{\"symbol\":1,\"created_at\":-1}|Predictions index"
        "sessions|{\"session_token\":1}|Sessions token index"
    )

    for test_case in "${test_cases[@]}"; do
        IFS='|' read -r collection index_spec description <<< "$test_case"

        local has_index
        has_index=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
            --eval "
                db.${collection}.getIndexes().some(idx =>
                    JSON.stringify(idx.key) === JSON.stringify($index_spec)
                )
            " 2>&1)

        if [[ "$has_index" == *"true"* ]]; then
            log_success "Index exists: $description"
        else
            log_error "Index missing: $description"
        fi
    done
}

# Test time-series collection
test_timeseries() {
    log_info "Testing time-series collection configuration..."

    local is_timeseries
    is_timeseries=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "
            const info = db.getCollectionInfos({name: 'market_data'})[0];
            info && info.type === 'timeseries'
        " 2>&1)

    if [[ "$is_timeseries" == *"true"* ]]; then
        log_success "market_data is configured as time-series"
    else
        log_warning "market_data is not a time-series collection"
    fi
}

# Test schema validation
test_schema_validation() {
    log_info "Testing schema validation rules..."

    local collections_with_validation=("users" "trades" "positions" "ml_models" "predictions")
    local validated=0

    for collection in "${collections_with_validation[@]}"; do
        local has_validator
        has_validator=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
            --eval "
                const info = db.getCollectionInfos({name: '$collection'})[0];
                info && info.options && info.options.validator ? true : false
            " 2>&1)

        if [[ "$has_validator" == *"true"* ]]; then
            log_success "Schema validation enabled for '$collection'"
            ((validated++))
        else
            log_warning "No schema validation for '$collection'"
        fi
    done

    [[ $validated -gt 0 ]]
}

# Test TTL indexes
test_ttl_indexes() {
    log_info "Testing TTL indexes for automatic cleanup..."

    local ttl_collections=("trades" "sessions" "notifications" "market_data")
    local ttl_count=0

    for collection in "${ttl_collections[@]}"; do
        local has_ttl
        has_ttl=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
            --eval "
                db.${collection}.getIndexes().some(idx => idx.expireAfterSeconds != null)
            " 2>&1)

        if [[ "$has_ttl" == *"true"* ]]; then
            log_success "TTL index configured for '$collection'"
            ((ttl_count++))
        else
            log_warning "No TTL index for '$collection'"
        fi
    done

    [[ $ttl_count -gt 0 ]]
}

# Test system configuration
test_system_config() {
    log_info "Testing system configuration..."

    local config_items=(
        "global_config"
        "rate_limits"
        "risk_limits"
        "trading_pairs"
        "ai_service_config"
    )

    for config_id in "${config_items[@]}"; do
        local exists
        exists=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
            --eval "db.system_config.findOne({_id: '$config_id'}) != null" 2>&1)

        if [[ "$exists" == *"true"* ]]; then
            log_success "System config exists: '$config_id'"
        else
            log_error "System config missing: '$config_id'"
        fi
    done
}

# Test database users
test_database_users() {
    log_info "Testing database users..."

    local admin_db_conn="mongodb://${MONGO_ROOT_USER}:${MONGO_ROOT_PASSWORD}@${MONGO_HOST}:${MONGO_PORT}/admin?authSource=admin"

    local expected_users=("bot_core_admin" "bot_core_app" "bot_core_readonly")
    local user_count=0

    for username in "${expected_users[@]}"; do
        local exists
        exists=$(mongosh --quiet "$admin_db_conn" \
            --eval "db.getSiblingDB('bot_core').getUsers().find(u => u.user === '$username') != null" 2>&1)

        if [[ "$exists" == *"true"* ]]; then
            log_success "Database user exists: '$username'"
            ((user_count++))
        else
            log_warning "Database user missing: '$username'"
        fi
    done

    [[ $user_count -gt 0 ]]
}

# Performance check
test_performance() {
    log_info "Testing database performance..."

    # Test query performance (should be < 100ms for indexed queries)
    local query_time
    query_time=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "
            const start = new Date();
            db.system_config.findOne({_id: 'global_config'});
            const end = new Date();
            end - start
        " 2>&1 | tail -1)

    if [[ "$query_time" =~ ^[0-9]+$ ]] && [[ $query_time -lt 100 ]]; then
        log_success "Query performance: ${query_time}ms (< 100ms target)"
    else
        log_warning "Query performance: ${query_time}ms (slower than expected)"
    fi

    # Test index usage
    local uses_index
    uses_index=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "
            const explain = db.users.find({email: 'test@example.com'}).explain('executionStats');
            explain.executionStats.executionSuccess &&
            explain.executionStats.totalDocsExamined < 100
        " 2>&1)

    if [[ "$uses_index" == *"true"* ]]; then
        log_success "Queries are using indexes efficiently"
    else
        log_warning "Some queries may not be using indexes"
    fi
}

# Test data integrity
test_data_integrity() {
    log_info "Testing data integrity..."

    # Check for orphaned references (positions without valid user_id)
    local orphaned_count
    orphaned_count=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "
            db.positions.countDocuments({
                user_id: { \$not: { \$in: db.users.distinct('_id') } }
            })
        " 2>&1 | tail -1)

    if [[ "$orphaned_count" == "0" ]]; then
        log_success "No orphaned position records found"
    else
        log_warning "Found $orphaned_count orphaned position records"
    fi
}

# Print summary
print_summary() {
    echo -e "\n${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              Validation Results Summary                   ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}\n"

    echo -e "${BLUE}Total Tests:${NC} $TOTAL_TESTS"
    echo -e "${GREEN}Passed:${NC} $PASSED_TESTS"
    echo -e "${YELLOW}Warnings:${NC} $WARNING_TESTS"
    echo -e "${RED}Failed:${NC} $FAILED_TESTS"

    local pass_rate=$((PASSED_TESTS * 100 / TOTAL_TESTS))
    echo -e "\n${BLUE}Pass Rate:${NC} ${pass_rate}%"

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo -e "\n${GREEN}✓ Database validation PASSED${NC}"
        return 0
    else
        echo -e "\n${RED}✗ Database validation FAILED${NC}"
        echo -e "${YELLOW}Run './scripts/init-db.sh' to fix issues${NC}"
        return 1
    fi
}

# Main execution
main() {
    print_banner

    echo -e "${BLUE}Database:${NC} $MONGO_DB"
    echo -e "${BLUE}Host:${NC} ${MONGO_HOST}:${MONGO_PORT}"
    echo -e ""

    # Run all tests
    test_connection || exit 1
    test_replica_set
    test_database || exit 1
    test_collections
    test_indexes
    test_timeseries
    test_schema_validation
    test_ttl_indexes
    test_system_config
    test_database_users
    test_performance
    test_data_integrity

    # Print summary
    print_summary
}

# Run main
main "$@"
