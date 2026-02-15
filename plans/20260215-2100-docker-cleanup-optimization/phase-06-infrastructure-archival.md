# Phase 6: Infrastructure Archival

**Parent Plan**: [plan.md](plan.md)
**Dependencies**: Phase 1-5 (All previous phases)
**Date**: 2026-02-15
**Effort**: 10 minutes
**Priority**: LOW
**Status**: 🔲 Pending

---

## Overview

Archive infrastructure configuration directories for Kong, RabbitMQ, Prometheus, Grafana for future reference.

**Action**: Add README.DEPRECATED.md to each directory
**Reason**: Low disk usage, may be useful in future, provides historical context

---

## Key Insights

- Infrastructure config directories exist but unused
- Low disk usage (~10MB total)
- May be useful for future reference or different deployment
- Better to archive than delete (preserves institutional knowledge)

---

## Requirements

### Must Archive (Not Delete)
1. `infrastructure/kong/` - Kong API gateway configs
2. `infrastructure/rabbitmq/` - RabbitMQ configs
3. `infrastructure/monitoring/` - Prometheus configs
4. `infrastructure/grafana/` - Grafana init scripts

### Archive Method
- Add `README.DEPRECATED.md` to each directory
- Explain when/why deprecated
- Link to Docker cleanup plan
- Keep all config files intact

---

## Related Code Files

```
/Users/dungngo97/Documents/bot-core/infrastructure/kong/
/Users/dungngo97/Documents/bot-core/infrastructure/rabbitmq/
/Users/dungngo97/Documents/bot-core/infrastructure/monitoring/
/Users/dungngo97/Documents/bot-core/infrastructure/grafana/
```

---

## Implementation Steps

### Step 1: Create Deprecation Notice Template

Create template for README.DEPRECATED.md:

```markdown
# ⚠️ DEPRECATED - SERVICE REMOVED

**Date**: 2026-02-15
**Reason**: Service removed from BotCore architecture
**Plan**: [Docker Cleanup Plan](../../plans/20260215-2100-docker-cleanup-optimization/plan.md)

---

## What Happened

This directory contains configuration for [SERVICE_NAME] which was removed from the BotCore project on 2026-02-15 as part of a Docker service optimization effort.

**Removed Services**:
- Kong API Gateway (3 containers)
- RabbitMQ + Celery + Flower (4 containers)
- Prometheus + Grafana (2 containers)

**Reason for Removal**:
- Services were never used in production
- Core services communicate directly without API gateway
- No async task processing implemented
- No Prometheus metrics endpoints exist

---

## Current Architecture

**Active Services** (6-7 containers):
- MongoDB - Database
- Rust Core Engine - Trading engine API
- Python AI Service - ML predictions
- Next.js UI Dashboard - Frontend
- MCP Server - OpenClaw bridge
- OpenClaw - Telegram/WhatsApp AI assistant
- Redis - Cache (optional, profile: redis)

---

## Historical Context

These configurations were part of initial architecture planning but never integrated:
- Kong: API gateway for microservices (all services talk directly)
- RabbitMQ/Celery: Async task queue (Python runs synchronous API only)
- Prometheus/Grafana: Monitoring (no /metrics endpoints implemented)

---

## If You Need This Service

If you want to re-enable [SERVICE_NAME]:

1. Check git history for removed docker-compose.yml sections:
   ```bash
   git log --all --full-history -- docker-compose.yml
   git show <commit-hash>:docker-compose.yml
   ```

2. Review configurations in this directory

3. Add service back to docker-compose.yml with appropriate profile

4. Update environment variables in .env

5. Update scripts (bot.sh, health-check.sh, etc.)

---

**Status**: ARCHIVED (not deleted, for reference only)
```

### Step 2: Add Deprecation Notice to infrastructure/kong/

**File**: `/Users/dungngo97/Documents/bot-core/infrastructure/kong/README.DEPRECATED.md`

**Content**: Use template above, replace `[SERVICE_NAME]` with "Kong API Gateway"

### Step 3: Add Deprecation Notice to infrastructure/rabbitmq/

**File**: `/Users/dungngo97/Documents/bot-core/infrastructure/rabbitmq/README.DEPRECATED.md`

**Content**: Use template above, replace `[SERVICE_NAME]` with "RabbitMQ Message Queue"

### Step 4: Add Deprecation Notice to infrastructure/monitoring/

**File**: `/Users/dungngo97/Documents/bot-core/infrastructure/monitoring/README.DEPRECATED.md`

**Content**: Use template above, replace `[SERVICE_NAME]` with "Prometheus Monitoring"

### Step 5: Add Deprecation Notice to infrastructure/grafana/

**File**: `/Users/dungngo97/Documents/bot-core/infrastructure/grafana/README.DEPRECATED.md`

**Content**: Use template above, replace `[SERVICE_NAME]` with "Grafana Dashboard"

### Step 6: Verify Archival

```bash
# Check deprecation notices exist
ls -la infrastructure/kong/README.DEPRECATED.md
ls -la infrastructure/rabbitmq/README.DEPRECATED.md
ls -la infrastructure/monitoring/README.DEPRECATED.md
ls -la infrastructure/grafana/README.DEPRECATED.md

# Verify all config files still present
ls infrastructure/kong/
ls infrastructure/rabbitmq/
ls infrastructure/monitoring/
ls infrastructure/grafana/

# Check disk usage
du -sh infrastructure/kong infrastructure/rabbitmq infrastructure/monitoring infrastructure/grafana
```

---

## Todo List

### Create Deprecation Notices
- [ ] Create infrastructure/kong/README.DEPRECATED.md
- [ ] Create infrastructure/rabbitmq/README.DEPRECATED.md
- [ ] Create infrastructure/monitoring/README.DEPRECATED.md
- [ ] Create infrastructure/grafana/README.DEPRECATED.md

### Verify Archival
- [ ] Verify all 4 README.DEPRECATED.md files created
- [ ] Verify all original config files still present
- [ ] Check disk usage (should be ~10MB total)

### Optional: Update infrastructure/README.md
- [ ] Add "Deprecated Directories" section
- [ ] List archived directories with dates
- [ ] Link to cleanup plan

---

## Success Criteria

- ✅ 4 README.DEPRECATED.md files created
- ✅ All original config files preserved
- ✅ Deprecation notices explain when/why removed
- ✅ Deprecation notices link to cleanup plan
- ✅ Instructions for re-enabling services included

---

## Risk Assessment

**Risk Level**: 🟢 ZERO RISK

**Risks**:
- None - just adding documentation

**Benefits**:
- Preserves institutional knowledge
- Easy to reference in future
- Clear deprecation status

---

## Output Artifacts

- 4 README.DEPRECATED.md files
- Preserved config directories
- Archival verification report
