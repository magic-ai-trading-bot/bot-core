# Phase 2: Environment Variable Cleanup

**Parent Plan**: [plan.md](plan.md)
**Dependencies**: Phase 1 (Docker Compose)
**Date**: 2026-02-15
**Effort**: 20 minutes
**Priority**: CRITICAL
**Status**: 🔲 Pending

---

## Overview

Remove unused environment variables for Kong, RabbitMQ, Celery, Prometheus, Grafana from 5 .env files.

**Variables to Remove**: RABBITMQ_* (5 vars), KONG_DB_PASSWORD, GRAFANA_PASSWORD, PROMETHEUS_ENDPOINT
**Files to Update**: .env, .env.example, .env.example.secure, .env.production.example, python-ai-service/.env.example

---

## Key Insights

- .env contains actual secrets (generated passwords)
- .env.example, .env.example.secure have placeholder values
- python-ai-service/.env.example has full RabbitMQ config (5 env vars)
- .env.production.example has PROMETHEUS_ENDPOINT
- MCP server PROMETHEUS_URL already removed in Phase 1

---

## Requirements

### Must Remove
1. **RABBITMQ_USER** (5 files)
2. **RABBITMQ_PASSWORD** (5 files)
3. **RABBITMQ_HOST** (1 file: python-ai-service/.env.example)
4. **RABBITMQ_PORT** (1 file: python-ai-service/.env.example)
5. **RABBITMQ_VHOST** (1 file: python-ai-service/.env.example)
6. **KONG_DB_PASSWORD** (3 files)
7. **GRAFANA_PASSWORD** (3 files)
8. **PROMETHEUS_ENDPOINT** (1 file: .env.production.example)

### Must Keep
- All core service env vars (MongoDB, Rust, Python, Frontend, MCP)
- REDIS_PASSWORD (redis service kept for future use)

---

## Related Code Files

```
/Users/dungngo97/Documents/bot-core/.env
/Users/dungngo97/Documents/bot-core/.env.example
/Users/dungngo97/Documents/bot-core/.env.example.secure
/Users/dungngo97/Documents/bot-core/.env.production.example
/Users/dungngo97/Documents/bot-core/python-ai-service/.env.example
```

---

## Implementation Steps

### Step 1: Update .env (Production Secrets)

**File**: `/Users/dungngo97/Documents/bot-core/.env`

**Remove**:
```bash
RABBITMQ_USER=admin
RABBITMQ_PASSWORD=Yw9cex26PgWRWqa3SkgBDQCKuqeGCj9Xx0g2+dUAyWc=
GRAFANA_PASSWORD=r4+HoOmcKdu93QyrxM17ncFvKiMG1OEO5mLcOs/Rvqk=
KONG_DB_PASSWORD=mqw3Zk/OOVf051YE8leNG2gg5eH3g76FAghfKfn0aAc=
```

**Keep**: All other vars (MongoDB, Binance, OpenAI, Redis, etc.)

### Step 2: Update .env.example

**File**: `/Users/dungngo97/Documents/bot-core/.env.example`

**Remove**:
```bash
GRAFANA_PASSWORD=admin
RABBITMQ_USER=admin
RABBITMQ_PASSWORD=your-secure-password
KONG_DB_PASSWORD=your-secure-password
```

### Step 3: Update .env.example.secure

**File**: `/Users/dungngo97/Documents/bot-core/.env.example.secure`

**Remove**:
```bash
RABBITMQ_USER=admin
RABBITMQ_PASSWORD=REPLACE_WITH_GENERATED_PASSWORD
GRAFANA_PASSWORD=REPLACE_WITH_GENERATED_PASSWORD
KONG_DB_PASSWORD=REPLACE_WITH_GENERATED_PASSWORD
```

### Step 4: Update .env.production.example

**File**: `/Users/dungngo97/Documents/bot-core/.env.production.example`

**Remove**:
```bash
PROMETHEUS_ENDPOINT=/metrics
```

### Step 5: Update python-ai-service/.env.example

**File**: `/Users/dungngo97/Documents/bot-core/python-ai-service/.env.example`

**Remove**:
```bash
# RabbitMQ Configuration (for async tasks)
RABBITMQ_USER=admin
RABBITMQ_PASSWORD=rabbitmq_default_password
RABBITMQ_HOST=rabbitmq
RABBITMQ_PORT=5672
RABBITMQ_VHOST=bot-core
```

**Keep**: All Python service vars (OpenAI, MongoDB, Rust API URL, etc.)

### Step 6: Verify Cleanup

```bash
# Verify no RabbitMQ references
grep -r "RABBITMQ" .env* python-ai-service/.env*
# Expected: ZERO matches

# Verify no Kong references
grep -r "KONG_DB" .env*
# Expected: ZERO matches

# Verify no Grafana references
grep -r "GRAFANA_PASSWORD" .env*
# Expected: ZERO matches

# Verify no Prometheus references
grep -r "PROMETHEUS_ENDPOINT" .env*
# Expected: ZERO matches

# Verify Redis kept
grep -r "REDIS_PASSWORD" .env*
# Expected: 2-3 matches (Redis still used)
```

---

## Todo List

### .env
- [ ] Remove RABBITMQ_USER
- [ ] Remove RABBITMQ_PASSWORD
- [ ] Remove KONG_DB_PASSWORD
- [ ] Remove GRAFANA_PASSWORD
- [ ] Verify REDIS_PASSWORD still present

### .env.example
- [ ] Remove RABBITMQ_USER
- [ ] Remove RABBITMQ_PASSWORD
- [ ] Remove KONG_DB_PASSWORD
- [ ] Remove GRAFANA_PASSWORD
- [ ] Verify example format correct

### .env.example.secure
- [ ] Remove RABBITMQ_USER
- [ ] Remove RABBITMQ_PASSWORD
- [ ] Remove KONG_DB_PASSWORD
- [ ] Remove GRAFANA_PASSWORD

### .env.production.example
- [ ] Remove PROMETHEUS_ENDPOINT

### python-ai-service/.env.example
- [ ] Remove RABBITMQ_USER
- [ ] Remove RABBITMQ_PASSWORD
- [ ] Remove RABBITMQ_HOST
- [ ] Remove RABBITMQ_PORT
- [ ] Remove RABBITMQ_VHOST
- [ ] Remove "RabbitMQ Configuration" comment section

### Verification
- [ ] Verify zero RABBITMQ references
- [ ] Verify zero KONG_DB references
- [ ] Verify zero GRAFANA_PASSWORD references
- [ ] Verify zero PROMETHEUS_ENDPOINT references
- [ ] Verify REDIS_PASSWORD kept in .env files

---

## Success Criteria

- ✅ All 5 .env files updated
- ✅ 12+ environment variables removed
- ✅ Zero grep matches for removed variables
- ✅ REDIS_PASSWORD preserved
- ✅ Core service variables unchanged
- ✅ Files pass validation: `./scripts/validate-env.sh` (after Phase 3 script update)

---

## Risk Assessment

**Risk Level**: 🟢 LOW

**Risks**:
- None - variables unused by core services

**Mitigation**:
- Backup .env before changes (contains real secrets)
- Keep .env in .gitignore (security)

---

## Output Artifacts

- Updated .env (5 files)
- Verification report (grep zero matches)
