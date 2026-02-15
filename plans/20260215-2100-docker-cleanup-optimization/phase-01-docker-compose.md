# Phase 1: Docker Compose Cleanup

**Parent Plan**: [plan.md](plan.md)
**Date**: 2026-02-15
**Effort**: 30 minutes
**Priority**: CRITICAL
**Status**: 🔲 Pending

---

## Overview

Remove 9 service definitions, 5 volumes, and 3 profiles from both docker-compose.yml files.

**Services to Remove**: kong, kong-database, kong-migration, rabbitmq, celery-worker, celery-beat, flower, prometheus, grafana
**Volumes to Remove**: rabbitmq_data, kong_data, prometheus_data, grafana_data, flower_tmp
**Profiles to Remove**: api-gateway, messaging, monitoring

---

## Key Insights

- Both `docker-compose.yml` (root) and `infrastructure/docker/docker-compose.yml` are IDENTICAL (854 lines)
- Services isolated by profiles - never start unless profile activated
- Core services have ZERO dependencies on removed services
- Redis service has dual profile (messaging|redis) - keep service, remove "messaging" profile reference
- MCP server has `PROMETHEUS_URL` env var (lines 654, 693) - must remove

---

## Requirements

### Must Remove
1. **9 service definitions** (~470 YAML lines total)
2. **5 volume definitions** (~15 lines)
3. **3 profiles** from service definitions
4. **PROMETHEUS_URL** env var from mcp-server (2 locations)

### Must Keep
- redis service (update profile to just `redis`, remove `messaging`)
- All core services unchanged
- Network configuration unchanged

---

## Related Code Files

```
/Users/dungngo97/Documents/bot-core/docker-compose.yml
/Users/dungngo97/Documents/bot-core/infrastructure/docker/docker-compose.yml
```

---

## Implementation Steps

### Step 1: Remove Service Definitions

**File**: `docker-compose.yml` (both locations)

Remove these service blocks:

```yaml
# Lines 381-409: rabbitmq (29 lines)
# Lines 411-463: celery-worker (53 lines)
# Lines 465-511: celery-beat (47 lines)
# Lines 513-559: flower (47 lines)
# Lines 561-580: kong-database (20 lines)
# Lines 582-598: kong-migration (17 lines)
# Lines 600-635: kong (36 lines)
# Lines 791-804: prometheus (14 lines)
# Lines 806-824: grafana (19 lines)
```

**Total**: ~282 lines removed

### Step 2: Remove Volume Definitions

**File**: `docker-compose.yml` (both locations)
**Location**: volumes section at bottom

Remove these volumes:

```yaml
rabbitmq_data:
  driver: local
kong_data:
  driver: local
prometheus_data:
  driver: local
grafana_data:
  driver: local
flower_tmp:
  driver: local
```

**Keep**: redis_data, mongodb_data, mongodb_config, rust_target_cache, openclaw_data

### Step 3: Update Redis Service Profile

**File**: `docker-compose.yml` (both locations)
**Location**: redis service definition (lines 361-379)

**Change**:
```yaml
# OLD
profiles:
  - messaging
  - redis

# NEW
profiles:
  - redis
```

### Step 4: Remove PROMETHEUS_URL from MCP Server

**File**: `docker-compose.yml` (both locations)
**Locations**:
- mcp-server service (line 654)
- mcp-server-dev service (line 693)

**Remove**:
```yaml
- PROMETHEUS_URL=http://prometheus:9090
```

### Step 5: Update Both Files

Execute changes in:
1. `/Users/dungngo97/Documents/bot-core/docker-compose.yml`
2. `/Users/dungngo97/Documents/bot-core/infrastructure/docker/docker-compose.yml`

**CRITICAL**: Files must remain identical after changes.

### Step 6: Verify Changes

```bash
# Check service count
grep -c "^  [a-z].*:$" docker-compose.yml
# Expected: 13 services (was 22)

# Check volume count
grep -A 1 "^volumes:" docker-compose.yml | grep -c "driver: local"
# Expected: 5 volumes (was 10)

# Verify no references to removed services
grep -E "(kong|rabbitmq|celery|flower|prometheus|grafana)" docker-compose.yml
# Expected: ZERO matches

# Verify files identical
diff docker-compose.yml infrastructure/docker/docker-compose.yml
# Expected: NO differences
```

---

## Todo List

- [ ] Backup both docker-compose.yml files
- [ ] Remove rabbitmq service (lines 381-409)
- [ ] Remove celery-worker service (lines 411-463)
- [ ] Remove celery-beat service (lines 465-511)
- [ ] Remove flower service (lines 513-559)
- [ ] Remove kong-database service (lines 561-580)
- [ ] Remove kong-migration service (lines 582-598)
- [ ] Remove kong service (lines 600-635)
- [ ] Remove prometheus service (lines 791-804)
- [ ] Remove grafana service (lines 806-824)
- [ ] Remove 5 volume definitions
- [ ] Update redis profile (remove "messaging")
- [ ] Remove PROMETHEUS_URL from mcp-server (line 654)
- [ ] Remove PROMETHEUS_URL from mcp-server-dev (line 693)
- [ ] Apply same changes to infrastructure/docker/docker-compose.yml
- [ ] Verify files identical with diff
- [ ] Verify service count (13 services)
- [ ] Verify volume count (5 volumes)
- [ ] Verify zero references to removed services

---

## Success Criteria

- ✅ Both docker-compose.yml files updated identically
- ✅ 9 services removed
- ✅ 5 volumes removed
- ✅ Redis service keeps profile `redis` only
- ✅ PROMETHEUS_URL removed from mcp-server
- ✅ Zero grep matches for removed service names
- ✅ Files pass validation: `docker compose config --quiet`

---

## Risk Assessment

**Risk Level**: 🟢 LOW

**Risks**:
- None - services never used, profile-isolated

**Mitigation**:
- Backup files before changes
- Use git for version control
- Verify with `docker compose config` before proceeding

---

## Output Artifacts

- Updated `docker-compose.yml` (both locations)
- Verification report (service count, volume count, diff check)
