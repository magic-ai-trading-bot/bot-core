#!/bin/bash

# Pre-Deployment Validation Script
# Comprehensive checks before deploying the trading bot
# @spec:FR-OPS-002 - Pre-Deployment Validation
# @ref:specs/04-deployment/4.2-deployment-procedures.md

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Counters
ERRORS=0
WARNINGS=0
CHECKS_PASSED=0
TOTAL_CHECKS=0

# Report file
REPORT_FILE="${1:-/tmp/pre-deployment-report.txt}"

# Function to log to both console and file
log() {
    echo -e "$1" | tee -a "$REPORT_FILE"
}

# Function to print section header
print_section() {
    log ""
    log "${CYAN}========================================${NC}"
    log "${CYAN}$1${NC}"
    log "${CYAN}========================================${NC}"
}

# Initialize report
cat > "$REPORT_FILE" << EOF
Pre-Deployment Validation Report
Generated: $(date)
Working Directory: $(pwd)
User: $(whoami)
Hostname: $(hostname)

========================================
VALIDATION RESULTS
========================================

EOF

print_section "PRE-DEPLOYMENT VALIDATION"
log "Report will be saved to: $REPORT_FILE"
log ""

# 1. Environment Validation
print_section "1. Environment Variables"
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
log "Running environment validation..."

if ./scripts/validate-env.sh --skip-connectivity > /tmp/env-check.log 2>&1; then
    log "${GREEN}✓ Environment validation passed${NC}"
    echo "PASSED: Environment validation" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
else
    EXIT_CODE=$?
    if [ $EXIT_CODE -eq 2 ]; then
        log "${YELLOW}⚠ Environment validation passed with warnings${NC}"
        log "See /tmp/env-check.log for details"
        echo "WARNING: Environment validation - warnings present" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    else
        log "${RED}✗ Environment validation failed${NC}"
        log "See /tmp/env-check.log for details"
        echo "FAILED: Environment validation" >> "$REPORT_FILE"
        ERRORS=$((ERRORS + 1))
    fi
fi

# 2. Docker Daemon
print_section "2. Docker Environment"
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if docker info > /dev/null 2>&1; then
    log "${GREEN}✓ Docker daemon is running${NC}"
    echo "PASSED: Docker daemon" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))

    # Get Docker version
    DOCKER_VERSION=$(docker --version | cut -d ' ' -f3 | cut -d ',' -f1)
    log "  Docker version: $DOCKER_VERSION"
else
    log "${RED}✗ Docker daemon is not running${NC}"
    echo "FAILED: Docker daemon" >> "$REPORT_FILE"
    ERRORS=$((ERRORS + 1))
fi

# 3. Docker Compose
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if command -v docker-compose > /dev/null 2>&1; then
    COMPOSE_VERSION=$(docker-compose --version | cut -d ' ' -f3 | cut -d ',' -f1)
    log "${GREEN}✓ Docker Compose is installed (v$COMPOSE_VERSION)${NC}"
    echo "PASSED: Docker Compose - v$COMPOSE_VERSION" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))

    # Check minimum version (1.29.0 or higher)
    MIN_VERSION="1.29.0"
    if [ "$(printf '%s\n' "$MIN_VERSION" "$COMPOSE_VERSION" | sort -V | head -n1)" = "$MIN_VERSION" ]; then
        log "  Version meets minimum requirement ($MIN_VERSION)"
    else
        log "${YELLOW}⚠ Docker Compose version is below recommended ($MIN_VERSION)${NC}"
        echo "WARNING: Docker Compose - old version" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
    fi
elif docker compose version > /dev/null 2>&1; then
    COMPOSE_VERSION=$(docker compose version --short)
    log "${GREEN}✓ Docker Compose V2 is installed (v$COMPOSE_VERSION)${NC}"
    echo "PASSED: Docker Compose V2 - v$COMPOSE_VERSION" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
else
    log "${RED}✗ Docker Compose is not installed${NC}"
    echo "FAILED: Docker Compose - not installed" >> "$REPORT_FILE"
    ERRORS=$((ERRORS + 1))
fi

# 4. System Resources
print_section "3. System Resources"

# Disk Space
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if command -v df > /dev/null 2>&1; then
    DISK_AVAILABLE=$(df -h . | tail -1 | awk '{print $4}')
    DISK_AVAILABLE_GB=$(df -BG . | tail -1 | awk '{print $4}' | sed 's/G//')

    if [ "$DISK_AVAILABLE_GB" -ge 50 ]; then
        log "${GREEN}✓ Sufficient disk space available ($DISK_AVAILABLE)${NC}"
        echo "PASSED: Disk space - $DISK_AVAILABLE" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    elif [ "$DISK_AVAILABLE_GB" -ge 20 ]; then
        log "${YELLOW}⚠ Limited disk space available ($DISK_AVAILABLE)${NC}"
        log "  Recommended: 50GB+, Available: ${DISK_AVAILABLE_GB}GB"
        echo "WARNING: Disk space - only ${DISK_AVAILABLE_GB}GB" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    else
        log "${RED}✗ Insufficient disk space ($DISK_AVAILABLE)${NC}"
        log "  Minimum required: 20GB, Available: ${DISK_AVAILABLE_GB}GB"
        echo "FAILED: Disk space - only ${DISK_AVAILABLE_GB}GB" >> "$REPORT_FILE"
        ERRORS=$((ERRORS + 1))
    fi
else
    log "${YELLOW}⚠ Cannot check disk space${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# Memory
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if command -v free > /dev/null 2>&1; then
    MEMORY_AVAILABLE=$(free -h | grep Mem | awk '{print $7}')
    MEMORY_AVAILABLE_GB=$(free -g | grep Mem | awk '{print $7}')

    if [ "$MEMORY_AVAILABLE_GB" -ge 8 ]; then
        log "${GREEN}✓ Sufficient memory available ($MEMORY_AVAILABLE)${NC}"
        echo "PASSED: Memory - $MEMORY_AVAILABLE" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    elif [ "$MEMORY_AVAILABLE_GB" -ge 4 ]; then
        log "${YELLOW}⚠ Limited memory available ($MEMORY_AVAILABLE)${NC}"
        log "  Recommended: 8GB+, Available: ${MEMORY_AVAILABLE_GB}GB"
        echo "WARNING: Memory - only ${MEMORY_AVAILABLE_GB}GB" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    else
        log "${RED}✗ Insufficient memory ($MEMORY_AVAILABLE)${NC}"
        log "  Minimum required: 4GB, Available: ${MEMORY_AVAILABLE_GB}GB"
        echo "FAILED: Memory - only ${MEMORY_AVAILABLE_GB}GB" >> "$REPORT_FILE"
        ERRORS=$((ERRORS + 1))
    fi
elif command -v vm_stat > /dev/null 2>&1; then
    # macOS
    FREE_PAGES=$(vm_stat | grep "Pages free" | awk '{print $3}' | sed 's/\.//')
    MEMORY_AVAILABLE_GB=$(( FREE_PAGES * 4096 / 1024 / 1024 / 1024 ))

    if [ "$MEMORY_AVAILABLE_GB" -ge 8 ]; then
        log "${GREEN}✓ Sufficient memory available (~${MEMORY_AVAILABLE_GB}GB)${NC}"
        echo "PASSED: Memory - ${MEMORY_AVAILABLE_GB}GB" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    else
        log "${YELLOW}⚠ Limited memory available (~${MEMORY_AVAILABLE_GB}GB)${NC}"
        echo "WARNING: Memory - only ${MEMORY_AVAILABLE_GB}GB" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    fi
else
    log "${YELLOW}⚠ Cannot check memory${NC}"
    WARNINGS=$((WARNINGS + 1))
fi

# 5. Network Connectivity
print_section "4. Network Connectivity"

# Internet connectivity
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if ping -c 1 8.8.8.8 > /dev/null 2>&1; then
    log "${GREEN}✓ Internet connectivity available${NC}"
    echo "PASSED: Internet connectivity" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
else
    log "${RED}✗ No internet connectivity${NC}"
    echo "FAILED: Internet connectivity" >> "$REPORT_FILE"
    ERRORS=$((ERRORS + 1))
fi

# DNS resolution
TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if nslookup google.com > /dev/null 2>&1; then
    log "${GREEN}✓ DNS resolution working${NC}"
    echo "PASSED: DNS resolution" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
else
    log "${RED}✗ DNS resolution failed${NC}"
    echo "FAILED: DNS resolution" >> "$REPORT_FILE"
    ERRORS=$((ERRORS + 1))
fi

# 6. Port Availability
print_section "5. Port Availability"

REQUIRED_PORTS=(3000 8000 8080 27017)
for port in "${REQUIRED_PORTS[@]}"; do
    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    if lsof -Pi :$port -sTCP:LISTEN -t >/dev/null 2>&1; then
        log "${YELLOW}⚠ Port $port is already in use${NC}"

        # Show what's using it
        PROCESS=$(lsof -Pi :$port -sTCP:LISTEN -t | xargs ps -p | tail -1 | awk '{print $NF}')
        log "  Used by: $PROCESS"
        echo "WARNING: Port $port - in use by $PROCESS" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    else
        log "${GREEN}✓ Port $port is available${NC}"
        echo "PASSED: Port $port - available" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    fi
done

# 7. Database Connectivity (optional)
print_section "6. Database Connectivity"

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ ! -z "$DATABASE_URL" ]; then
    if command -v mongosh > /dev/null 2>&1; then
        if mongosh "$DATABASE_URL" --eval "db.adminCommand('ping')" > /dev/null 2>&1; then
            log "${GREEN}✓ Database connection successful${NC}"
            echo "PASSED: Database connectivity" >> "$REPORT_FILE"
            CHECKS_PASSED=$((CHECKS_PASSED + 1))
        else
            log "${YELLOW}⚠ Database connection failed${NC}"
            log "  This may be expected if MongoDB is not running yet"
            echo "WARNING: Database connectivity - failed" >> "$REPORT_FILE"
            WARNINGS=$((WARNINGS + 1))
            CHECKS_PASSED=$((CHECKS_PASSED + 1))
        fi
    else
        log "${YELLOW}⚠ mongosh not installed - skipping database test${NC}"
        echo "SKIPPED: Database connectivity - mongosh not installed" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
    fi
else
    log "${YELLOW}⚠ DATABASE_URL not set - skipping database test${NC}"
    echo "SKIPPED: Database connectivity - URL not set" >> "$REPORT_FILE"
    WARNINGS=$((WARNINGS + 1))
fi

# 8. SSL Certificates (for production)
if [ "$NODE_ENV" = "production" ]; then
    print_section "7. SSL/TLS Certificates"

    TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
    if [ ! -z "$SSL_CERT_PATH" ] && [ ! -z "$SSL_KEY_PATH" ]; then
        if [ -f "$SSL_CERT_PATH" ] && [ -f "$SSL_KEY_PATH" ]; then
            log "${GREEN}✓ SSL certificates found${NC}"
            echo "PASSED: SSL certificates - present" >> "$REPORT_FILE"
            CHECKS_PASSED=$((CHECKS_PASSED + 1))

            # Check certificate expiry
            if command -v openssl > /dev/null 2>&1; then
                EXPIRY_DATE=$(openssl x509 -in "$SSL_CERT_PATH" -noout -enddate 2>/dev/null | cut -d= -f2)
                if [ ! -z "$EXPIRY_DATE" ]; then
                    log "  Certificate expires: $EXPIRY_DATE"

                    # Check if expiring soon (30 days)
                    EXPIRY_EPOCH=$(date -d "$EXPIRY_DATE" +%s 2>/dev/null || date -j -f "%b %d %H:%M:%S %Y %Z" "$EXPIRY_DATE" +%s 2>/dev/null)
                    NOW_EPOCH=$(date +%s)
                    DAYS_UNTIL_EXPIRY=$(( (EXPIRY_EPOCH - NOW_EPOCH) / 86400 ))

                    if [ "$DAYS_UNTIL_EXPIRY" -lt 30 ]; then
                        log "${YELLOW}⚠ Certificate expires in $DAYS_UNTIL_EXPIRY days${NC}"
                        echo "WARNING: SSL certificate - expires in $DAYS_UNTIL_EXPIRY days" >> "$REPORT_FILE"
                        WARNINGS=$((WARNINGS + 1))
                    fi
                fi
            fi
        else
            log "${RED}✗ SSL certificate files not found${NC}"
            log "  CERT: $SSL_CERT_PATH"
            log "  KEY: $SSL_KEY_PATH"
            echo "FAILED: SSL certificates - not found" >> "$REPORT_FILE"
            ERRORS=$((ERRORS + 1))
        fi
    else
        log "${YELLOW}⚠ SSL certificates not configured${NC}"
        echo "WARNING: SSL certificates - not configured" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
    fi
fi

# 9. Build Tests (smoke test)
print_section "8. Build Verification"

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
log "Checking if services can build..."

# Just check if Dockerfiles exist
if [ -f "rust-core-engine/Dockerfile" ] && \
   [ -f "python-ai-service/Dockerfile" ] && \
   [ -f "nextjs-ui-dashboard/Dockerfile" ]; then
    log "${GREEN}✓ All service Dockerfiles present${NC}"
    echo "PASSED: Dockerfiles - all present" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
else
    log "${RED}✗ Some Dockerfiles are missing${NC}"
    echo "FAILED: Dockerfiles - missing" >> "$REPORT_FILE"
    ERRORS=$((ERRORS + 1))
fi

# 10. Configuration Files
print_section "9. Configuration Files"

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ -f "docker-compose.yml" ] || [ -L "docker-compose.yml" ]; then
    log "${GREEN}✓ docker-compose.yml found${NC}"
    echo "PASSED: docker-compose.yml" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
else
    log "${RED}✗ docker-compose.yml not found${NC}"
    echo "FAILED: docker-compose.yml - not found" >> "$REPORT_FILE"
    ERRORS=$((ERRORS + 1))
fi

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ -f ".env" ]; then
    log "${GREEN}✓ .env file found${NC}"
    echo "PASSED: .env file" >> "$REPORT_FILE"
    CHECKS_PASSED=$((CHECKS_PASSED + 1))
else
    log "${RED}✗ .env file not found${NC}"
    echo "FAILED: .env file - not found" >> "$REPORT_FILE"
    ERRORS=$((ERRORS + 1))
fi

# 11. Git Status (optional check)
print_section "10. Version Control"

TOTAL_CHECKS=$((TOTAL_CHECKS + 1))
if [ -d ".git" ]; then
    if git diff-index --quiet HEAD -- 2>/dev/null; then
        log "${GREEN}✓ Working directory is clean${NC}"
        echo "PASSED: Git status - clean" >> "$REPORT_FILE"
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    else
        log "${YELLOW}⚠ Working directory has uncommitted changes${NC}"
        echo "WARNING: Git status - uncommitted changes" >> "$REPORT_FILE"
        WARNINGS=$((WARNINGS + 1))
        CHECKS_PASSED=$((CHECKS_PASSED + 1))
    fi

    # Show current branch and commit
    BRANCH=$(git rev-parse --abbrev-ref HEAD)
    COMMIT=$(git rev-parse --short HEAD)
    log "  Branch: $BRANCH"
    log "  Commit: $COMMIT"
else
    log "${YELLOW}⚠ Not a git repository${NC}"
    echo "WARNING: Git - not a repository" >> "$REPORT_FILE"
    WARNINGS=$((WARNINGS + 1))
fi

# Summary
print_section "VALIDATION SUMMARY"

log "Total Checks: $TOTAL_CHECKS"
log "Passed: ${GREEN}$CHECKS_PASSED${NC}"
log "Warnings: ${YELLOW}$WARNINGS${NC}"
log "Errors: ${RED}$ERRORS${NC}"
log ""

# Write summary to report
cat >> "$REPORT_FILE" << EOF

========================================
SUMMARY
========================================
Total Checks: $TOTAL_CHECKS
Passed: $CHECKS_PASSED
Warnings: $WARNINGS
Errors: $ERRORS
Success Rate: $(( CHECKS_PASSED * 100 / TOTAL_CHECKS ))%

EOF

# Final verdict
if [ $ERRORS -eq 0 ] && [ $WARNINGS -eq 0 ]; then
    log "${GREEN}✓ ALL PRE-DEPLOYMENT CHECKS PASSED!${NC}"
    log "${GREEN}  System is ready for deployment${NC}"
    echo "STATUS: READY - All checks passed" >> "$REPORT_FILE"
    exit 0
elif [ $ERRORS -eq 0 ]; then
    log "${YELLOW}⚠ PRE-DEPLOYMENT CHECKS PASSED WITH WARNINGS${NC}"
    log "${YELLOW}  Review warnings before deployment${NC}"
    echo "STATUS: READY WITH WARNINGS - Review warnings" >> "$REPORT_FILE"
    exit 2
else
    log "${RED}✗ PRE-DEPLOYMENT CHECKS FAILED${NC}"
    log "${RED}  Please fix errors before deployment${NC}"
    log ""
    log "Common fixes:"
    log "  1. Start Docker daemon: sudo systemctl start docker"
    log "  2. Free up disk space: docker system prune -a"
    log "  3. Install Docker Compose: https://docs.docker.com/compose/install/"
    log "  4. Stop conflicting services on ports 3000, 8000, 8080, 27017"
    echo "STATUS: NOT READY - Critical errors present" >> "$REPORT_FILE"
    exit 1
fi
