# Phase 7: Verification & Testing

**Parent Plan**: [plan.md](plan.md)
**Dependencies**: Phase 1-6 (ALL previous phases complete)
**Date**: 2026-02-15
**Effort**: 30 minutes
**Priority**: CRITICAL
**Status**: 🔲 Pending

---

## Overview

Verify that all cleanup is complete, core services start successfully, and system works after removing 9 Docker services.

**Critical Tests**:
1. Docker compose validation
2. Service startup (dev mode)
3. Health checks (all services)
4. API functionality
5. Documentation accuracy

---

## Key Insights

- This is FINAL verification before PR
- Must test actual service startup, not just syntax
- Finance project - ZERO tolerance for broken services
- All 6-7 core services MUST start and pass health checks

---

## Requirements

### Must Verify
1. Docker compose files valid (syntax check)
2. 6-7 core services start successfully
3. All health checks pass
4. Rust API responds
5. Python API responds
6. Frontend loads
7. MCP server connects
8. Zero grep matches for removed services
9. Scripts work (bot.sh, health-check.sh, validate-env.sh)

### Success Metrics
- ✅ `docker compose config --quiet` passes
- ✅ `docker compose --profile dev up -d` starts 6-7 services
- ✅ All health checks green within 5 minutes
- ✅ `curl http://localhost:8080/api/health` returns 200
- ✅ `curl http://localhost:8000/health` returns 200
- ✅ `curl http://localhost:3000/` returns 200
- ✅ Zero grep matches for removed services in active code

---

## Related Code Files

```
/Users/dungngo97/Documents/bot-core/docker-compose.yml
/Users/dungngo97/Documents/bot-core/scripts/bot.sh
/Users/dungngo97/Documents/bot-core/scripts/health-check.sh
/Users/dungngo97/Documents/bot-core/scripts/validate-env.sh
```

---

## Implementation Steps

### Step 1: Docker Compose Validation

```bash
# Validate syntax
docker compose config --quiet
# Expected: NO output (means valid)

# Check for removed services
docker compose config | grep -iE "(kong|rabbitmq|celery|flower|prometheus|grafana)"
# Expected: ZERO matches

# Count services
docker compose config --services | wc -l
# Expected: 13 services (6 prod + 6 dev + mongodb)
```

### Step 2: Environment Variable Validation

```bash
# Run validation script
./scripts/validate-env.sh
# Expected: PASS (all required vars present, no removed vars)

# Check for removed vars
grep -E "(RABBITMQ|KONG_DB|GRAFANA_PASSWORD|PROMETHEUS_ENDPOINT)" .env .env.example
# Expected: ZERO matches
```

### Step 3: Start Services in Dev Mode

```bash
# Stop any running services
docker compose down

# Start dev services
./scripts/bot.sh dev
# OR
docker compose --profile dev up -d

# Expected output:
# Creating mongodb ... done
# Creating python-ai-service-dev ... done
# Creating rust-core-engine-dev ... done
# Creating nextjs-ui-dashboard-dev ... done
# Creating mcp-server-dev ... done
# Creating openclaw-dev ... done
# (6-7 services total)
```

### Step 4: Monitor Service Health

```bash
# Watch service status
watch -n 5 "docker compose ps"

# Check logs for errors
docker compose logs --tail=50 --timestamps

# Wait for all health checks (up to 5 minutes)
# MongoDB: ~30s
# Python AI: ~60s
# Rust Engine: ~120s (compile time)
# Frontend: ~30s
# MCP: ~15s
# OpenClaw: ~10s
```

### Step 5: Run Health Check Script

```bash
# Run health check script
./scripts/health-check.sh

# Expected output:
# ✅ MongoDB: healthy (localhost:27017)
# ✅ Rust Core Engine: healthy (localhost:8080/api/health)
# ✅ Python AI Service: healthy (localhost:8000/health)
# ✅ Frontend: healthy (localhost:3000/)
# ✅ MCP Server: healthy (localhost:8090/health)
# ✅ Redis: healthy (optional)
#
# NO output for Kong, RabbitMQ, Celery, Prometheus, Grafana
```

### Step 6: API Functionality Tests

```bash
# Test Rust API
curl -f http://localhost:8080/api/health
# Expected: {"status":"ok"}

# Test Python API
curl -f http://localhost:8000/health
# Expected: {"status":"healthy"}

# Test Frontend
curl -f http://localhost:3000/
# Expected: HTML response (React app)

# Test MCP Server
curl -f http://localhost:8090/health
# Expected: {"status":"ok"}

# Test WebSocket (optional)
wscat -c ws://localhost:8080/ws
# Expected: Connection established
```

### Step 7: Code Grep Verification

```bash
# Verify no removed services in active code
grep -r "kong\|rabbitmq\|celery\|flower\|prometheus\|grafana" \
  --exclude-dir=node_modules \
  --exclude-dir=target \
  --exclude-dir=.git \
  --exclude-dir=archive \
  --exclude-dir=infrastructure \
  --exclude="*.lock" \
  --exclude="*.log" \
  .

# Expected: ZERO matches (except in README.DEPRECATED.md files)
```

### Step 8: Documentation Verification

```bash
# Verify CLAUDE.md updated
grep -i "kong\|rabbitmq\|celery\|prometheus\|grafana" CLAUDE.md
# Expected: ZERO matches

# Verify README.md updated
grep -i "kong\|rabbitmq\|celery\|prometheus\|grafana" README.md
# Expected: ZERO matches

# Verify traceability matrix updated
grep "FR-ASYNC" specs/TRACEABILITY_MATRIX.md
# Expected: Shows DEPRECATED status
```

### Step 9: Script Functionality Tests

```bash
# Test bot.sh
./scripts/bot.sh --help
# Expected: Shows usage, NO mention of Kong/RabbitMQ/Celery/Prometheus/Grafana

./scripts/bot.sh status
# Expected: Shows 6-7 services running

# Test health-check.sh
./scripts/health-check.sh
# Expected: Checks only 6-7 core services

# Test validate-env.sh
./scripts/validate-env.sh
# Expected: PASS (no errors about missing removed vars)
```

### Step 10: Final Cleanup Verification

```bash
# Check volumes removed
docker volume ls | grep -iE "(rabbitmq|kong|prometheus|grafana|flower)"
# Expected: ZERO matches

# Check images
docker images | grep -iE "(rabbitmq|kong|prometheus|grafana)"
# Expected: OK to have images (can remove with docker image prune)

# Check disk usage saved
docker system df
# Note: Compare before/after cleanup
```

---

## Todo List

### Docker Validation
- [ ] Run `docker compose config --quiet` (must pass)
- [ ] Verify no removed services in config
- [ ] Count services (13 total)

### Environment Validation
- [ ] Run `./scripts/validate-env.sh` (must pass)
- [ ] Verify no removed vars in .env files

### Service Startup
- [ ] Stop all services
- [ ] Start dev mode: `./scripts/bot.sh dev`
- [ ] Verify 6-7 services start
- [ ] Monitor health checks (5 min max)
- [ ] Check logs for errors

### Health Checks
- [ ] Run `./scripts/health-check.sh`
- [ ] Verify only 6-7 services checked
- [ ] All health checks must pass

### API Tests
- [ ] Test Rust API: `curl localhost:8080/api/health`
- [ ] Test Python API: `curl localhost:8000/health`
- [ ] Test Frontend: `curl localhost:3000/`
- [ ] Test MCP: `curl localhost:8090/health`

### Code Verification
- [ ] Grep for removed services (zero matches)
- [ ] Verify CLAUDE.md updated
- [ ] Verify README.md updated
- [ ] Verify TRACEABILITY_MATRIX.md shows FR-ASYNC deprecated

### Script Tests
- [ ] Test `./scripts/bot.sh --help`
- [ ] Test `./scripts/bot.sh status`
- [ ] Test `./scripts/health-check.sh`
- [ ] Test `./scripts/validate-env.sh`

### Cleanup Verification
- [ ] Check no removed service volumes
- [ ] Check disk space saved
- [ ] Document before/after metrics

---

## Success Criteria

- ✅ Docker compose validation passes
- ✅ All 6-7 core services start successfully
- ✅ All health checks pass within 5 minutes
- ✅ Rust API responds (200 OK)
- ✅ Python API responds (200 OK)
- ✅ Frontend loads (200 OK)
- ✅ MCP Server responds (200 OK)
- ✅ Health check script checks only core services
- ✅ Validate-env script passes
- ✅ Zero grep matches for removed services
- ✅ bot.sh --help shows correct usage
- ✅ No removed service volumes exist

---

## Risk Assessment

**Risk Level**: 🟢 LOW (if all previous phases complete)

**Risks**:
- Service startup failure due to missing dependencies
- Health check script errors

**Mitigation**:
- Test each service individually if failure
- Check logs for error messages
- Rollback to git backup if critical failure

---

## Rollback Plan

**If ANY test fails**:

```bash
# Stop services
docker compose down

# Rollback git changes
git checkout main
git branch -D feature/docker-cleanup-20260215

# Restart services
./scripts/bot.sh dev

# Report issue with error logs
```

---

## Output Artifacts

- Service startup logs
- Health check results
- API test results
- Grep verification report
- Before/after disk usage comparison
- Verification summary report

---

## Verification Summary Template

```markdown
# Docker Service Cleanup - Verification Report

**Date**: 2026-02-15
**Status**: [PASS/FAIL]

## Tests Performed

### 1. Docker Compose Validation
- Syntax check: [PASS/FAIL]
- Service count: [13 expected]
- Removed services: [0 found]

### 2. Environment Variables
- Validation script: [PASS/FAIL]
- Removed vars: [0 found]

### 3. Service Startup
- MongoDB: [PASS/FAIL]
- Rust Engine: [PASS/FAIL]
- Python AI: [PASS/FAIL]
- Frontend: [PASS/FAIL]
- MCP Server: [PASS/FAIL]
- OpenClaw: [PASS/FAIL]

### 4. Health Checks
- All services healthy: [YES/NO]
- Time to healthy: [X minutes]

### 5. API Tests
- Rust API: [200 OK / ERROR]
- Python API: [200 OK / ERROR]
- Frontend: [200 OK / ERROR]
- MCP: [200 OK / ERROR]

### 6. Code Verification
- Removed service references: [0 found]
- CLAUDE.md updated: [YES/NO]
- README.md updated: [YES/NO]

### 7. Scripts
- bot.sh: [PASS/FAIL]
- health-check.sh: [PASS/FAIL]
- validate-env.sh: [PASS/FAIL]

## Disk Usage
- Before: [X GB]
- After: [Y GB]
- Saved: [Z GB]

## Conclusion
[READY FOR PR / NEEDS FIXES]

## Issues Found
[List any issues or NONE]
```

---

**Next Step After Verification**: Create PR when all tests pass.
