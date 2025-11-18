#!/bin/bash

# Local Deployment Automation Script
# Automated deployment with health checks, backup, and rollback capability
# @spec:FR-OPS-004 - Deployment Automation
# @ref:specs/04-deployment/4.2-deployment-procedures.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
BACKUP_DIR="${BACKUP_DIR:-./backups}"
DEPLOYMENT_LOG="${DEPLOYMENT_LOG:-./logs/deployment.log}"
MAX_ROLLBACK_ATTEMPTS=3
HEALTH_CHECK_RETRIES=5
HEALTH_CHECK_DELAY=10

# Deployment mode
MODE="${1:-production}"
SKIP_BACKUP="${SKIP_BACKUP:-false}"
SKIP_TESTS="${SKIP_TESTS:-false}"

# Function to log to both console and file
log() {
    echo -e "$1" | tee -a "$DEPLOYMENT_LOG"
}

# Function to print section header
print_section() {
    log ""
    log "${CYAN}========================================${NC}"
    log "${CYAN}$1${NC}"
    log "${CYAN}========================================${NC}"
}

# Function to handle errors
handle_error() {
    local exit_code=$1
    local error_msg=$2

    log "${RED}✗ ERROR: $error_msg${NC}"
    log "${RED}  Deployment failed at $(date)${NC}"
    log ""
    log "${YELLOW}Starting automatic rollback...${NC}"

    # Attempt rollback
    if [ -f "./scripts/rollback.sh" ]; then
        ./scripts/rollback.sh
    else
        log "${RED}✗ Rollback script not found${NC}"
        log "${YELLOW}  Please manually restore from backup: $BACKUP_DIR${NC}"
    fi

    exit "$exit_code"
}

# Set up error handler
trap 'handle_error $? "Unexpected error occurred"' ERR

# Create necessary directories
mkdir -p "$BACKUP_DIR"
mkdir -p "$(dirname "$DEPLOYMENT_LOG")"

# Initialize log
cat > "$DEPLOYMENT_LOG" << EOF
Deployment Log
Started: $(date)
Mode: $MODE
User: $(whoami)
Hostname: $(hostname)

========================================
DEPLOYMENT PROCESS
========================================

EOF

print_section "TRADING BOT DEPLOYMENT"
log "Mode: $MODE"
log "Backup directory: $BACKUP_DIR"
log "Log file: $DEPLOYMENT_LOG"
log ""

# Step 1: Pre-deployment checks
print_section "1. Pre-Deployment Validation"

log "Running pre-deployment checks..."
if ./scripts/pre-deployment-check.sh > /tmp/pre-deploy.log 2>&1; then
    log "${GREEN}✓ Pre-deployment checks passed${NC}"
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 2 ]; then
        log "${YELLOW}⚠ Pre-deployment checks passed with warnings${NC}"
        log "See /tmp/pre-deploy.log for details"

        read -p "Continue despite warnings? (y/N) " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            log "Deployment cancelled by user"
            exit 1
        fi
    else
        log "${RED}✗ Pre-deployment checks failed${NC}"
        log "See /tmp/pre-deploy.log for details"
        exit 1
    fi
fi

# Step 2: Environment validation
print_section "2. Environment Validation"

log "Validating environment variables..."
if ./scripts/validate-env.sh /tmp/env-validation.log; then
    log "${GREEN}✓ Environment validation passed${NC}"
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 2 ]; then
        log "${YELLOW}⚠ Environment validation passed with warnings${NC}"
    else
        log "${RED}✗ Environment validation failed${NC}"
        log "See /tmp/env-validation.log for details"
        exit 1
    fi
fi

# Step 3: Backup current deployment
if [ "$SKIP_BACKUP" != "true" ]; then
    print_section "3. Backing Up Current Deployment"

    BACKUP_NAME="backup-$(date +%Y%m%d-%H%M%S)"
    BACKUP_PATH="$BACKUP_DIR/$BACKUP_NAME"

    log "Creating backup: $BACKUP_NAME"
    mkdir -p "$BACKUP_PATH"

    # Backup docker-compose state
    if command -v docker-compose > /dev/null 2>&1; then
        log "  Saving Docker Compose state..."
        docker-compose ps > "$BACKUP_PATH/docker-compose-state.txt" 2>/dev/null || true
        docker-compose config > "$BACKUP_PATH/docker-compose-config.yml" 2>/dev/null || true
    fi

    # Backup database if MongoDB is running
    if docker ps | grep -q mongodb; then
        log "  Backing up MongoDB database..."
        docker exec mongodb mongodump --out /tmp/backup > /dev/null 2>&1 || true
        docker cp mongodb:/tmp/backup "$BACKUP_PATH/mongodb-dump" 2>/dev/null || true
    fi

    # Save current git commit
    if [ -d ".git" ]; then
        git rev-parse HEAD > "$BACKUP_PATH/git-commit.txt" 2>/dev/null || true
        git diff > "$BACKUP_PATH/git-changes.diff" 2>/dev/null || true
    fi

    # Save .env file
    if [ -f ".env" ]; then
        cp .env "$BACKUP_PATH/.env.backup"
    fi

    # Create backup metadata
    cat > "$BACKUP_PATH/metadata.txt" << EOF
Backup Created: $(date)
Mode: $MODE
User: $(whoami)
Hostname: $(hostname)
Git Commit: $(git rev-parse HEAD 2>/dev/null || echo "N/A")
Git Branch: $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A")
EOF

    log "${GREEN}✓ Backup created: $BACKUP_PATH${NC}"

    # Keep only last 10 backups
    log "  Cleaning old backups (keeping last 10)..."
    ls -t "$BACKUP_DIR" | tail -n +11 | xargs -I {} rm -rf "$BACKUP_DIR/{}" 2>/dev/null || true
else
    log "${YELLOW}⚠ Skipping backup (SKIP_BACKUP=true)${NC}"
fi

# Step 4: Stop old services
print_section "4. Stopping Current Services"

log "Gracefully stopping services..."

if docker-compose ps | grep -q "Up"; then
    # Stop services gracefully
    docker-compose down --timeout 30 || true
    log "${GREEN}✓ Services stopped${NC}"
else
    log "${YELLOW}⚠ No running services found${NC}"
fi

# Wait for ports to be released
log "Waiting for ports to be released..."
sleep 5

# Step 5: Pull latest images (if using registry)
if [ "$MODE" = "production" ]; then
    print_section "5. Pulling Latest Images"

    log "Pulling latest Docker images..."
    docker-compose pull || log "${YELLOW}⚠ Failed to pull images (will use local)${NC}"
fi

# Step 6: Build services
print_section "6. Building Services"

log "Building Docker images..."

if [ "$MODE" = "production" ]; then
    docker-compose -f docker-compose.yml build --no-cache || handle_error 1 "Build failed"
else
    docker-compose build || handle_error 1 "Build failed"
fi

log "${GREEN}✓ Build completed${NC}"

# Step 7: Start new services
print_section "7. Starting New Services"

log "Starting services..."

if [ "$MODE" = "production" ]; then
    docker-compose -f docker-compose.yml up -d || handle_error 1 "Failed to start services"
else
    docker-compose up -d || handle_error 1 "Failed to start services"
fi

log "${GREEN}✓ Services started${NC}"

# Step 8: Wait for services to initialize
print_section "8. Waiting for Services to Initialize"

log "Waiting for services to initialize (30 seconds)..."
sleep 30

# Step 9: Health checks
print_section "9. Running Health Checks"

HEALTH_PASSED=false
for i in $(seq 1 $HEALTH_CHECK_RETRIES); do
    log "Health check attempt $i/$HEALTH_CHECK_RETRIES..."

    if ./scripts/health-check.sh /tmp/health-check.log; then
        log "${GREEN}✓ All health checks passed${NC}"
        HEALTH_PASSED=true
        break
    else
        EXIT_CODE=$?
        if [ $EXIT_CODE -eq 2 ]; then
            log "${YELLOW}⚠ Some services are degraded${NC}"
            if [ $i -eq $HEALTH_CHECK_RETRIES ]; then
                read -p "Continue with degraded services? (y/N) " -n 1 -r
                echo
                if [[ $REPLY =~ ^[Yy]$ ]]; then
                    HEALTH_PASSED=true
                    break
                fi
            fi
        else
            log "${RED}✗ Health checks failed${NC}"
        fi

        if [ $i -lt $HEALTH_CHECK_RETRIES ]; then
            log "Waiting ${HEALTH_CHECK_DELAY}s before retry..."
            sleep $HEALTH_CHECK_DELAY
        fi
    fi
done

if [ "$HEALTH_PASSED" != "true" ]; then
    handle_error 1 "Health checks failed after $HEALTH_CHECK_RETRIES attempts"
fi

# Step 10: Run smoke tests (if not skipped)
if [ "$SKIP_TESTS" != "true" ]; then
    print_section "10. Running Smoke Tests"

    log "Running basic smoke tests..."

    # Test Rust API
    if curl -s -f http://localhost:8080/api/health > /dev/null 2>&1; then
        log "${GREEN}✓ Rust API responding${NC}"
    else
        handle_error 1 "Rust API smoke test failed"
    fi

    # Test Python API
    if curl -s -f http://localhost:8000/health > /dev/null 2>&1; then
        log "${GREEN}✓ Python API responding${NC}"
    else
        handle_error 1 "Python API smoke test failed"
    fi

    # Test Frontend
    if curl -s -f http://localhost:3000 > /dev/null 2>&1; then
        log "${GREEN}✓ Frontend responding${NC}"
    else
        handle_error 1 "Frontend smoke test failed"
    fi

    log "${GREEN}✓ All smoke tests passed${NC}"
else
    log "${YELLOW}⚠ Skipping smoke tests (SKIP_TESTS=true)${NC}"
fi

# Step 11: Log deployment
print_section "11. Finalizing Deployment"

DEPLOYMENT_ID="deploy-$(date +%Y%m%d-%H%M%S)"

cat > "./logs/$DEPLOYMENT_ID.log" << EOF
Deployment ID: $DEPLOYMENT_ID
Completed: $(date)
Mode: $MODE
User: $(whoami)
Hostname: $(hostname)
Git Commit: $(git rev-parse HEAD 2>/dev/null || echo "N/A")
Git Branch: $(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo "N/A")
Backup: $BACKUP_PATH
Status: SUCCESS

Services:
$(docker-compose ps)

Health Check:
See /tmp/health-check.log
EOF

log "${GREEN}✓ Deployment logged: ./logs/$DEPLOYMENT_ID.log${NC}"

# Summary
print_section "DEPLOYMENT SUMMARY"

log "${GREEN}✓ DEPLOYMENT COMPLETED SUCCESSFULLY${NC}"
log ""
log "Deployment ID: $DEPLOYMENT_ID"
log "Backup: $BACKUP_PATH"
log "Log: $DEPLOYMENT_LOG"
log ""
log "Service URLs:"
log "  - Frontend:  http://localhost:3000"
log "  - Rust API:  http://localhost:8080/api/health"
log "  - Python AI: http://localhost:8000/health"
log ""
log "Next steps:"
log "  1. Monitor logs: ./scripts/bot.sh logs"
log "  2. Check health: ./scripts/health-check.sh"
log "  3. Test functionality through dashboard"
log ""
log "To rollback: ./scripts/rollback.sh"
log ""

exit 0
