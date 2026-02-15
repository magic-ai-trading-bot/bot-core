#!/bin/bash
# Docker Service Cleanup - Verification Script
# Run this after each phase to verify cleanup is correct

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

ERRORS=0

print_header() {
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
    ((ERRORS++))
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

# Change to project root
cd /Users/dungngo97/Documents/bot-core

print_header "Docker Service Cleanup Verification"
echo ""

# ============================================
# Phase 1: Docker Compose
# ============================================
print_header "Phase 1: Docker Compose"

# Check service count
SERVICE_COUNT=$(grep -c "^  [a-z].*:$" docker-compose.yml || echo 0)
if [ "$SERVICE_COUNT" -eq 13 ]; then
    print_success "Service count correct: 13 services"
else
    print_error "Service count wrong: $SERVICE_COUNT (expected 13)"
fi

# Check volumes removed
VOLUME_COUNT=$(grep -A 1 "^volumes:" docker-compose.yml | grep -c "driver: local" || echo 0)
if [ "$VOLUME_COUNT" -eq 5 ]; then
    print_success "Volume count correct: 5 volumes"
else
    print_error "Volume count wrong: $VOLUME_COUNT (expected 5)"
fi

# Check for removed services
REMOVED_SERVICES=$(grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" docker-compose.yml | grep -v "# " | wc -l || echo 0)
if [ "$REMOVED_SERVICES" -eq 0 ]; then
    print_success "No removed services found in docker-compose.yml"
else
    print_error "Found $REMOVED_SERVICES references to removed services"
    grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" docker-compose.yml | grep -v "# " | head -5
fi

# Check files identical
if diff docker-compose.yml infrastructure/docker/docker-compose.yml > /dev/null 2>&1; then
    print_success "docker-compose.yml files are identical"
else
    print_error "docker-compose.yml files are NOT identical"
fi

echo ""

# ============================================
# Phase 2: Environment Variables
# ============================================
print_header "Phase 2: Environment Variables"

# Check for removed vars
RABBITMQ_REFS=$(grep -r "RABBITMQ" .env* python-ai-service/.env* 2>/dev/null | wc -l || echo 0)
if [ "$RABBITMQ_REFS" -eq 0 ]; then
    print_success "No RABBITMQ env vars found"
else
    print_error "Found $RABBITMQ_REFS RABBITMQ references"
fi

KONG_REFS=$(grep -r "KONG_DB" .env* 2>/dev/null | wc -l || echo 0)
if [ "$KONG_REFS" -eq 0 ]; then
    print_success "No KONG_DB env vars found"
else
    print_error "Found $KONG_REFS KONG_DB references"
fi

GRAFANA_REFS=$(grep -r "GRAFANA_PASSWORD" .env* 2>/dev/null | wc -l || echo 0)
if [ "$GRAFANA_REFS" -eq 0 ]; then
    print_success "No GRAFANA_PASSWORD env vars found"
else
    print_error "Found $GRAFANA_REFS GRAFANA_PASSWORD references"
fi

PROMETHEUS_REFS=$(grep -r "PROMETHEUS_ENDPOINT" .env* 2>/dev/null | wc -l || echo 0)
if [ "$PROMETHEUS_REFS" -eq 0 ]; then
    print_success "No PROMETHEUS_ENDPOINT env vars found"
else
    print_error "Found $PROMETHEUS_REFS PROMETHEUS_ENDPOINT references"
fi

# Check Redis kept
REDIS_REFS=$(grep -r "REDIS_PASSWORD" .env* 2>/dev/null | wc -l || echo 0)
if [ "$REDIS_REFS" -gt 0 ]; then
    print_success "REDIS_PASSWORD still present"
else
    print_warning "REDIS_PASSWORD not found (optional, OK if not using Redis)"
fi

echo ""

# ============================================
# Phase 3: Scripts
# ============================================
print_header "Phase 3: Scripts"

# Check bot.sh
if grep -qi "with-enterprise" scripts/bot.sh; then
    print_error "bot.sh still has --with-enterprise flag"
else
    print_success "bot.sh --with-enterprise flag removed"
fi

BOTSH_REFS=$(grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" scripts/bot.sh | grep -v "^#" | wc -l || echo 0)
if [ "$BOTSH_REFS" -eq 0 ]; then
    print_success "No removed services in bot.sh"
else
    print_error "Found $BOTSH_REFS references in bot.sh"
fi

# Check health-check.sh
if [ -f scripts/health-check.sh ]; then
    HEALTH_REFS=$(grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" scripts/health-check.sh | wc -l || echo 0)
    if [ "$HEALTH_REFS" -eq 0 ]; then
        print_success "No removed services in health-check.sh"
    else
        print_error "Found $HEALTH_REFS references in health-check.sh"
    fi
else
    print_warning "health-check.sh not found (OK if doesn't exist)"
fi

# Check validate-env.sh
if [ -f scripts/validate-env.sh ]; then
    VALIDATE_REFS=$(grep -E "(RABBITMQ|KONG_DB|GRAFANA_PASSWORD|PROMETHEUS_ENDPOINT)" scripts/validate-env.sh | wc -l || echo 0)
    if [ "$VALIDATE_REFS" -eq 0 ]; then
        print_success "No removed env vars in validate-env.sh"
    else
        print_error "Found $VALIDATE_REFS removed env var validations"
    fi
else
    print_warning "validate-env.sh not found (OK if doesn't exist)"
fi

echo ""

# ============================================
# Phase 4: Documentation
# ============================================
print_header "Phase 4: Documentation"

# Check core docs
CLAUDE_REFS=$(grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" CLAUDE.md 2>/dev/null | wc -l || echo 0)
if [ "$CLAUDE_REFS" -eq 0 ]; then
    print_success "CLAUDE.md clean"
else
    print_warning "CLAUDE.md has $CLAUDE_REFS references (may be OK in archive sections)"
fi

README_REFS=$(grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" README.md 2>/dev/null | wc -l || echo 0)
if [ "$README_REFS" -eq 0 ]; then
    print_success "README.md clean"
else
    print_warning "README.md has $README_REFS references (may be OK in historical sections)"
fi

# Check deprecated docs deleted
if [ ! -f docs/fixes/RABBITMQ_PASSWORD_FIX.md ]; then
    print_success "RABBITMQ_PASSWORD_FIX.md deleted"
else
    print_error "RABBITMQ_PASSWORD_FIX.md still exists"
fi

if [ ! -f docs/guides/ASYNC_TASKS_README.md ]; then
    print_success "ASYNC_TASKS_README.md deleted"
else
    print_warning "ASYNC_TASKS_README.md still exists (delete if Phase 4 complete)"
fi

if [ ! -f specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md ]; then
    print_success "FR-ASYNC-TASKS.md deleted"
else
    print_warning "FR-ASYNC-TASKS.md still exists (delete if Phase 4 complete)"
fi

echo ""

# ============================================
# Phase 6: Infrastructure Archival
# ============================================
print_header "Phase 6: Infrastructure Archival"

# Check deprecation notices
if [ -f infrastructure/kong/README.DEPRECATED.md ]; then
    print_success "Kong deprecation notice created"
else
    print_warning "Kong deprecation notice missing"
fi

if [ -f infrastructure/rabbitmq/README.DEPRECATED.md ]; then
    print_success "RabbitMQ deprecation notice created"
else
    print_warning "RabbitMQ deprecation notice missing"
fi

if [ -f infrastructure/monitoring/README.DEPRECATED.md ]; then
    print_success "Prometheus deprecation notice created"
else
    print_warning "Prometheus deprecation notice missing"
fi

if [ -f infrastructure/grafana/README.DEPRECATED.md ]; then
    print_success "Grafana deprecation notice created"
else
    print_warning "Grafana deprecation notice missing"
fi

echo ""

# ============================================
# Phase 7: Service Tests (Optional)
# ============================================
print_header "Phase 7: Service Tests (Optional)"
print_info "Run these manually:"
echo "  docker compose config --quiet"
echo "  docker compose --profile dev up -d"
echo "  docker compose ps"
echo "  ./scripts/health-check.sh"
echo ""

# ============================================
# Summary
# ============================================
print_header "Summary"

if [ "$ERRORS" -eq 0 ]; then
    print_success "All checks passed! ✨"
    exit 0
else
    print_error "$ERRORS error(s) found"
    exit 1
fi
