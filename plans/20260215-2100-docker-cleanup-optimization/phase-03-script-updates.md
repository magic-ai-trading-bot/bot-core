# Phase 3: Script Updates

**Parent Plan**: [plan.md](plan.md)
**Dependencies**: Phase 1 (Docker Compose), Phase 2 (Environment Vars)
**Date**: 2026-02-15
**Effort**: 45 minutes
**Priority**: CRITICAL
**Status**: 🔲 Pending

---

## Overview

Update 4 critical bash scripts to remove Kong, RabbitMQ, Celery, Flower, Prometheus, Grafana logic.

**Scripts to Update**: bot.sh (445 lines), init-all-services.sh, health-check.sh, validate-env.sh

---

## Key Insights

- bot.sh is main orchestration script with service management
- Current line 43: `--with-enterprise` flag enables Kong, RabbitMQ, Monitoring (MUST REMOVE)
- Lines 49-53: Lists all enterprise services (misleading, remove)
- Lines 56-62: Examples show full stack deployment (update)
- init-all-services.sh calls init scripts for Kong, Grafana, RabbitMQ (remove)
- health-check.sh checks all service endpoints (remove 9 services)
- validate-env.sh validates removed env vars (remove)

---

## Requirements

### Must Update
1. **bot.sh**: Remove --with-enterprise flag, update service lists, remove profile logic
2. **init-all-services.sh**: Remove Kong/Grafana/RabbitMQ init calls
3. **health-check.sh**: Remove Kong/RabbitMQ/Celery/Prometheus/Grafana health checks
4. **validate-env.sh**: Remove RabbitMQ/Kong/Grafana env var validation

### Must Keep
- Core service logic (MongoDB, Rust, Python, Frontend, MCP, OpenClaw)
- Redis logic (keep for future use)
- Memory optimization flags
- Dev/prod mode switching

---

## Related Code Files

```
/Users/dungngo97/Documents/bot-core/scripts/bot.sh (445 lines)
/Users/dungngo97/Documents/bot-core/scripts/init-all-services.sh
/Users/dungngo97/Documents/bot-core/scripts/health-check.sh
/Users/dungngo97/Documents/bot-core/scripts/validate-env.sh
```

---

## Implementation Steps

### Step 1: Update bot.sh

**File**: `/Users/dungngo97/Documents/bot-core/scripts/bot.sh`

**Changes**:

1. **Remove --with-enterprise flag** (line 43):
```bash
# OLD
echo "  --with-enterprise   - Explicitly enable all enterprise features (Redis, RabbitMQ, Kong, Monitoring)"

# NEW
# (remove line entirely)
```

2. **Update DEFAULT SERVICES section** (lines 48-53):
```bash
# OLD
echo "  ✅ Core Services: MongoDB, Rust Engine, Python AI, Frontend"
echo "  ✅ Redis Cache"
echo "  ✅ RabbitMQ + Celery Worker + Celery Beat + Flower (Async Jobs)"
echo "  ✅ Kong API Gateway"
echo "  ✅ Prometheus + Grafana (Monitoring)"

# NEW
echo "  ✅ Core Services: MongoDB, Rust Engine, Python AI, Frontend, MCP Server, OpenClaw"
echo "  ✅ Redis Cache (optional, profile: redis)"
```

3. **Update Examples section** (lines 55-62):
```bash
# OLD
echo "  $0 start                      # Start ALL services (full stack)"
echo "  $0 start --memory-optimized   # Start all services with optimized memory"
echo "  $0 dev                        # Start in development mode (all features)"
echo "  $0 logs --service celery-worker  # Show logs for Celery worker"

# NEW
echo "  $0 start                      # Start core services (production)"
echo "  $0 start --memory-optimized   # Start with memory optimization"
echo "  $0 dev                        # Start in development mode"
echo "  $0 logs --service rust-core-engine  # Show logs for specific service"
```

4. **Remove enterprise profile logic** (search for "with-enterprise", "messaging", "api-gateway", "monitoring"):
```bash
# Find and remove all references to:
# - COMPOSE_PROFILES environment variable setting
# - --profile messaging
# - --profile api-gateway
# - --profile monitoring
```

5. **Update service start commands** (find docker compose up commands):
```bash
# OLD (example)
docker compose --profile dev --profile messaging --profile api-gateway --profile monitoring up -d

# NEW
docker compose --profile dev up -d
```

### Step 2: Update init-all-services.sh

**File**: `/Users/dungngo97/Documents/bot-core/scripts/init-all-services.sh`

**Remove**:
```bash
# Remove calls to:
./infrastructure/kong/init-kong.sh
./infrastructure/grafana/init-grafana.sh
./infrastructure/rabbitmq/init-rabbitmq.sh
```

**Keep**:
```bash
# MongoDB init
# Any Redis init (if exists)
```

### Step 3: Update health-check.sh

**File**: `/Users/dungngo97/Documents/bot-core/scripts/health-check.sh`

**Remove health checks for**:
```bash
# Kong
curl -f http://localhost:8001 || kong_status=unhealthy

# RabbitMQ
curl -f http://localhost:15672 || rabbitmq_status=unhealthy

# Celery (via Flower)
curl -f http://localhost:5555 || celery_status=unhealthy

# Prometheus
curl -f http://localhost:9090/-/healthy || prometheus_status=unhealthy

# Grafana
curl -f http://localhost:3001/api/health || grafana_status=unhealthy
```

**Keep health checks for**:
```bash
# MongoDB: localhost:27017
# Rust Engine: localhost:8080/api/health
# Python AI: localhost:8000/health
# Frontend: localhost:3000/health (or /)
# MCP Server: localhost:8090/health
# Redis: redis-cli ping (optional)
```

### Step 4: Update validate-env.sh

**File**: `/Users/dungngo97/Documents/bot-core/scripts/validate-env.sh`

**Remove validations for**:
```bash
# RabbitMQ vars
check_env_var "RABBITMQ_USER"
check_env_var "RABBITMQ_PASSWORD"
check_env_var "RABBITMQ_HOST"
check_env_var "RABBITMQ_PORT"
check_env_var "RABBITMQ_VHOST"

# Kong vars
check_env_var "KONG_DB_PASSWORD"

# Grafana vars
check_env_var "GRAFANA_PASSWORD"

# Prometheus vars
check_env_var "PROMETHEUS_ENDPOINT"
```

**Keep validations for**:
```bash
# Core services (MongoDB, Rust, Python, Frontend, MCP)
# Redis (optional)
# Binance API keys
# OpenAI API key
# JWT secrets
```

### Step 5: Verify Script Updates

```bash
# Verify no references to removed services in bot.sh
grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" scripts/bot.sh
# Expected: ZERO matches (or only in comments for context)

# Verify no removed service health checks
grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)" scripts/health-check.sh
# Expected: ZERO matches

# Verify no removed env var validation
grep -E "(RABBITMQ|KONG_DB|GRAFANA|PROMETHEUS)" scripts/validate-env.sh
# Expected: ZERO matches

# Test scripts
./scripts/validate-env.sh
# Expected: PASS (no errors about missing vars)
```

---

## Todo List

### bot.sh
- [ ] Remove --with-enterprise flag documentation
- [ ] Update DEFAULT SERVICES section (6 core services)
- [ ] Update Examples section (remove celery-worker example)
- [ ] Remove COMPOSE_PROFILES environment variable logic
- [ ] Remove --profile messaging from docker compose commands
- [ ] Remove --profile api-gateway from docker compose commands
- [ ] Remove --profile monitoring from docker compose commands
- [ ] Update any service count references (9 services → 6-7 core)
- [ ] Verify all docker compose up commands use only --profile dev/prod

### init-all-services.sh
- [ ] Remove Kong init call
- [ ] Remove Grafana init call
- [ ] Remove RabbitMQ init call
- [ ] Verify MongoDB init still present
- [ ] Test script runs without errors

### health-check.sh
- [ ] Remove Kong health check (port 8001)
- [ ] Remove RabbitMQ health check (port 15672)
- [ ] Remove Celery/Flower health check (port 5555)
- [ ] Remove Prometheus health check (port 9090)
- [ ] Remove Grafana health check (port 3001)
- [ ] Keep MongoDB, Rust, Python, Frontend, MCP checks
- [ ] Update service count in output
- [ ] Test script with running services

### validate-env.sh
- [ ] Remove RABBITMQ_USER validation
- [ ] Remove RABBITMQ_PASSWORD validation
- [ ] Remove RABBITMQ_HOST validation
- [ ] Remove RABBITMQ_PORT validation
- [ ] Remove RABBITMQ_VHOST validation
- [ ] Remove KONG_DB_PASSWORD validation
- [ ] Remove GRAFANA_PASSWORD validation
- [ ] Remove PROMETHEUS_ENDPOINT validation
- [ ] Keep REDIS_PASSWORD validation (optional)
- [ ] Test script passes with cleaned .env

### Verification
- [ ] grep bot.sh for removed services (zero matches)
- [ ] grep health-check.sh for removed services (zero matches)
- [ ] grep validate-env.sh for removed env vars (zero matches)
- [ ] Test ./scripts/validate-env.sh passes
- [ ] Test ./scripts/bot.sh --help shows updated usage
- [ ] Test ./scripts/bot.sh start works (after Phase 1)

---

## Success Criteria

- ✅ All 4 scripts updated
- ✅ Zero references to removed services in scripts
- ✅ validate-env.sh passes without errors
- ✅ health-check.sh checks only 6-7 core services
- ✅ bot.sh usage shows correct service list
- ✅ Scripts executable and functional

---

## Risk Assessment

**Risk Level**: 🟡 MEDIUM (scripts are critical for operations)

**Risks**:
- Breaking service start commands
- Breaking health checks

**Mitigation**:
- Test each script after changes
- Keep backups of original scripts
- Verify with actual service startup (Phase 7)

---

## Output Artifacts

- Updated scripts (4 files)
- Script verification report
- Test results from validate-env.sh
