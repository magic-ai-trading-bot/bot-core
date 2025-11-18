#!/usr/bin/env bash
# @spec:FR-DB-002 - Database Initialization and Migration Script
# @ref:specs/02-design/2.2-database/DB-SCHEMA.md
# MongoDB Database Initialization Script for Bot Core
# This script initializes the MongoDB database and runs migrations

set -euo pipefail

# Color codes for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
MONGO_HOST="${MONGO_HOST:-localhost}"
MONGO_PORT="${MONGO_PORT:-27017}"
MONGO_DB="${MONGO_DB:-bot_core}"
MAX_WAIT_TIME=60
WAIT_INTERVAL=2

# Load environment variables if .env exists
if [[ -f "$PROJECT_ROOT/.env" ]]; then
    echo -e "${BLUE}Loading environment variables from .env...${NC}"
    export $(grep -v '^#' "$PROJECT_ROOT/.env" | xargs)
fi

# Get MongoDB credentials from environment
MONGO_ROOT_USER="${MONGO_ROOT_USER:-admin}"
MONGO_ROOT_PASSWORD="${MONGO_ROOT_PASSWORD:-secure_mongo_password_change_me}"
MONGO_CONNECTION_STRING="mongodb://${MONGO_ROOT_USER}:${MONGO_ROOT_PASSWORD}@${MONGO_HOST}:${MONGO_PORT}/${MONGO_DB}?authSource=admin"

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Print banner
print_banner() {
    echo -e "${GREEN}"
    cat << "EOF"
╔═══════════════════════════════════════════════════════════╗
║                                                           ║
║           Bot Core - Database Initialization             ║
║                MongoDB Setup & Migrations                 ║
║                                                           ║
╚═══════════════════════════════════════════════════════════╝
EOF
    echo -e "${NC}"
}

# Check if mongosh is installed
check_mongosh() {
    log_info "Checking for mongosh (MongoDB Shell)..."

    if command -v mongosh &> /dev/null; then
        MONGOSH_VERSION=$(mongosh --version | head -n 1)
        log_success "mongosh found: $MONGOSH_VERSION"
        return 0
    else
        log_error "mongosh not found. Please install MongoDB Shell."
        log_info "Installation instructions: https://www.mongodb.com/docs/mongodb-shell/install/"
        return 1
    fi
}

# Wait for MongoDB to be ready
wait_for_mongo() {
    log_info "Waiting for MongoDB to be ready at ${MONGO_HOST}:${MONGO_PORT}..."

    local elapsed=0
    local is_ready=false

    while [[ $elapsed -lt $MAX_WAIT_TIME ]]; do
        if mongosh --quiet --host "$MONGO_HOST" --port "$MONGO_PORT" \
            --username "$MONGO_ROOT_USER" --password "$MONGO_ROOT_PASSWORD" \
            --authenticationDatabase admin \
            --eval "db.adminCommand('ping')" &> /dev/null; then
            is_ready=true
            break
        fi

        echo -n "."
        sleep $WAIT_INTERVAL
        elapsed=$((elapsed + WAIT_INTERVAL))
    done

    echo ""

    if [[ "$is_ready" == true ]]; then
        log_success "MongoDB is ready!"
        return 0
    else
        log_error "MongoDB did not become ready within ${MAX_WAIT_TIME} seconds"
        return 1
    fi
}

# Initialize replica set if needed
initialize_replica_set() {
    log_info "Checking replica set status..."

    local rs_status
    rs_status=$(mongosh --quiet --host "$MONGO_HOST" --port "$MONGO_PORT" \
        --username "$MONGO_ROOT_USER" --password "$MONGO_ROOT_PASSWORD" \
        --authenticationDatabase admin \
        --eval "try { rs.status().ok } catch(e) { 0 }" 2>&1 || echo "0")

    if [[ "$rs_status" == *"1"* ]]; then
        log_success "Replica set already initialized"
        return 0
    fi

    log_info "Initializing replica set 'rs0'..."

    mongosh --quiet --host "$MONGO_HOST" --port "$MONGO_PORT" \
        --username "$MONGO_ROOT_USER" --password "$MONGO_ROOT_PASSWORD" \
        --authenticationDatabase admin \
        --eval "rs.initiate({
            _id: 'rs0',
            members: [{ _id: 0, host: '${MONGO_HOST}:${MONGO_PORT}' }]
        })" > /dev/null

    if [[ $? -eq 0 ]]; then
        log_success "Replica set initialized"
        log_info "Waiting for replica set to stabilize..."
        sleep 10
        return 0
    else
        log_error "Failed to initialize replica set"
        return 1
    fi
}

# Run migrations for Rust service
run_rust_migrations() {
    log_info "Running Rust service migrations..."

    local migrations_dir="$PROJECT_ROOT/rust-core-engine/migrations"

    if [[ ! -d "$migrations_dir" ]]; then
        log_warning "Rust migrations directory not found: $migrations_dir"
        return 0
    fi

    # Run migration files in order
    for migration_file in "$migrations_dir"/*.js; do
        if [[ -f "$migration_file" ]]; then
            local filename=$(basename "$migration_file")
            log_info "Applying migration: $filename"

            if mongosh "$MONGO_CONNECTION_STRING" < "$migration_file" > /dev/null 2>&1; then
                log_success "Migration applied: $filename"
            else
                log_error "Failed to apply migration: $filename"
                return 1
            fi
        fi
    done

    log_success "Rust service migrations completed"
    return 0
}

# Run migrations for Python service
run_python_migrations() {
    log_info "Running Python service migrations..."

    local migrations_dir="$PROJECT_ROOT/python-ai-service/migrations"

    if [[ ! -d "$migrations_dir" ]]; then
        log_warning "Python migrations directory not found: $migrations_dir"
        return 0
    fi

    # Run migration files in order
    for migration_file in "$migrations_dir"/*.js; do
        if [[ -f "$migration_file" ]]; then
            local filename=$(basename "$migration_file")
            log_info "Applying migration: $filename"

            if mongosh "$MONGO_CONNECTION_STRING" < "$migration_file" > /dev/null 2>&1; then
                log_success "Migration applied: $filename"
            else
                log_error "Failed to apply migration: $filename"
                return 1
            fi
        fi
    done

    log_success "Python service migrations completed"
    return 0
}

# Verify database setup
verify_setup() {
    log_info "Verifying database setup..."

    # Check if database exists
    local db_exists
    db_exists=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "db.getName()" 2>&1)

    if [[ "$db_exists" == *"$MONGO_DB"* ]]; then
        log_success "Database '$MONGO_DB' exists"
    else
        log_error "Database '$MONGO_DB' not found"
        return 1
    fi

    # Count collections
    local collection_count
    collection_count=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "db.getCollectionNames().length" 2>&1)

    log_info "Collections found: $collection_count"

    # List all collections
    log_info "Collections in database:"
    mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "db.getCollectionNames().forEach(function(c) { print('  - ' + c); })" 2>&1

    # Check for critical collections
    local critical_collections=("users" "trades" "positions" "market_data")
    for collection in "${critical_collections[@]}"; do
        local exists
        exists=$(mongosh --quiet "$MONGO_CONNECTION_STRING" \
            --eval "db.getCollectionNames().includes('$collection')" 2>&1)

        if [[ "$exists" == *"true"* ]]; then
            log_success "Collection '$collection' exists"
        else
            log_warning "Collection '$collection' not found"
        fi
    done

    # Show database stats
    log_info "Database statistics:"
    mongosh --quiet "$MONGO_CONNECTION_STRING" \
        --eval "printjson(db.stats())" 2>&1 | grep -E "(db|collections|dataSize|indexSize)" || true

    log_success "Database verification completed"
    return 0
}

# Create database backup
create_backup() {
    log_info "Creating database backup..."

    local backup_dir="$PROJECT_ROOT/backups"
    local backup_file="$backup_dir/mongodb_backup_$(date +%Y%m%d_%H%M%S)"

    mkdir -p "$backup_dir"

    if command -v mongodump &> /dev/null; then
        mongodump --uri="$MONGO_CONNECTION_STRING" --out="$backup_file" --quiet

        if [[ $? -eq 0 ]]; then
            log_success "Backup created: $backup_file"

            # Compress backup
            tar -czf "${backup_file}.tar.gz" -C "$backup_dir" "$(basename "$backup_file")" 2>/dev/null
            rm -rf "$backup_file"
            log_success "Backup compressed: ${backup_file}.tar.gz"
        else
            log_warning "Backup failed (mongodump not available or failed)"
        fi
    else
        log_warning "mongodump not found. Skipping backup."
    fi
}

# Print connection strings
print_connection_info() {
    echo -e "\n${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║              Database Connection Information              ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}\n"

    echo -e "${BLUE}Database:${NC} $MONGO_DB"
    echo -e "${BLUE}Host:${NC} ${MONGO_HOST}:${MONGO_PORT}"
    echo -e ""
    echo -e "${BLUE}Connection Strings:${NC}"
    echo -e "  ${YELLOW}Admin:${NC}"
    echo -e "    mongodb://bot_core_admin:<password>@${MONGO_HOST}:${MONGO_PORT}/${MONGO_DB}"
    echo -e "  ${YELLOW}Application:${NC}"
    echo -e "    mongodb://bot_core_app:<password>@${MONGO_HOST}:${MONGO_PORT}/${MONGO_DB}"
    echo -e "  ${YELLOW}Read-only:${NC}"
    echo -e "    mongodb://bot_core_readonly:<password>@${MONGO_HOST}:${MONGO_PORT}/${MONGO_DB}"
    echo -e ""
    echo -e "${BLUE}Mongo Express (if enabled):${NC}"
    echo -e "  http://localhost:8081"
    echo -e ""
}

# Main execution
main() {
    print_banner

    # Check prerequisites
    check_mongosh || exit 1

    # Wait for MongoDB
    wait_for_mongo || exit 1

    # Initialize replica set
    initialize_replica_set || exit 1

    # Run migrations
    run_rust_migrations || log_warning "Rust migrations had issues"
    run_python_migrations || log_warning "Python migrations had issues"

    # Verify setup
    verify_setup || exit 1

    # Print connection info
    print_connection_info

    # Offer to create backup
    if [[ "${CREATE_BACKUP:-no}" == "yes" ]]; then
        create_backup
    fi

    echo -e "${GREEN}╔═══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${GREEN}║         Database Initialization Completed! ✓              ║${NC}"
    echo -e "${GREEN}╚═══════════════════════════════════════════════════════════╝${NC}\n"
}

# Run main function
main "$@"
