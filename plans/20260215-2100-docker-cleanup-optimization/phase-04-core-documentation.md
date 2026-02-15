# Phase 4: Core Documentation Updates

**Parent Plan**: [plan.md](plan.md)
**Dependencies**: Phase 1-3 (Docker, Env, Scripts)
**Date**: 2026-02-15
**Effort**: 90 minutes (longest phase)
**Priority**: HIGH
**Status**: 🔲 Pending

---

## Overview

Update 45+ documentation files to remove references to Kong, RabbitMQ, Celery, Flower, Prometheus, Grafana.

**Priority 1**: CLAUDE.md, README.md, TRACEABILITY_MATRIX.md (15 min)
**Priority 2**: Delete async task docs (5 min)
**Priority 3**: Update 15+ spec files (50 min)
**Priority 4**: Update production guides (20 min)

---

## Key Insights

- CLAUDE.md is project navigation hub - MUST update service references
- README.md has architecture diagram - update tech stack
- TRACEABILITY_MATRIX.md tracks 256 requirements - mark FR-ASYNC-* as DEPRECATED
- FR-ASYNC-TASKS.md, TC-ASYNC.md can be DELETED (no code exists)
- Monitoring specs (MON-METRICS.md, MON-LOGGING.md) - DELETE or mark deprecated
- Production guides reference all services - remove 9 services
- Archive docs - leave as-is (historical record)

---

## Requirements

### Priority 1: Critical Core Docs (MUST UPDATE)
1. CLAUDE.md - Project navigation hub
2. README.md - Main project documentation
3. specs/TRACEABILITY_MATRIX.md - Requirement tracking

### Priority 2: Delete Deprecated Docs
1. docs/fixes/RABBITMQ_PASSWORD_FIX.md - DELETE
2. docs/guides/ASYNC_TASKS_README.md - DELETE
3. specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md - DELETE
4. specs/03-testing/3.2-test-cases/TC-ASYNC.md - DELETE
5. specs/04-deployment/4.3-monitoring/MON-METRICS.md - DELETE
6. specs/04-deployment/4.3-monitoring/MON-LOGGING.md - DELETE

### Priority 3: Update Spec Files (15+ files)
- Architecture: ARCH-OVERVIEW.md, ARCH-SECURITY.md
- API: API-PYTHON-AI.md (remove Celery endpoints)
- Database: DB-SCHEMA.md, DB-INDEXES.md (remove async collections)
- Components: COMP-PYTHON-ML.md (remove Celery component)
- Infrastructure: INFRA-REQUIREMENTS.md, INFRA-DOCKER.md
- Operations: OPS-MANUAL.md, TROUBLESHOOTING.md
- DR: DR-PLAN.md

### Priority 4: Production Guides
- PRODUCTION_DEPLOYMENT_GUIDE.md
- OPERATIONS_MANUAL.md
- TROUBLESHOOTING.md
- HEALTH_CHECK_ENDPOINTS.md
- BACKUP_RESTORE_GUIDE.md

---

## Related Code Files

```
/Users/dungngo97/Documents/bot-core/CLAUDE.md
/Users/dungngo97/Documents/bot-core/README.md
/Users/dungngo97/Documents/bot-core/specs/TRACEABILITY_MATRIX.md
/Users/dungngo97/Documents/bot-core/docs/fixes/RABBITMQ_PASSWORD_FIX.md
/Users/dungngo97/Documents/bot-core/docs/guides/ASYNC_TASKS_README.md
/Users/dungngo97/Documents/bot-core/specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md
/Users/dungngo97/Documents/bot-core/specs/03-testing/3.2-test-cases/TC-ASYNC.md
/Users/dungngo97/Documents/bot-core/specs/04-deployment/4.3-monitoring/MON-METRICS.md
/Users/dungngo97/Documents/bot-core/specs/04-deployment/4.3-monitoring/MON-LOGGING.md
/Users/dungngo97/Documents/bot-core/specs/02-design/2.1-architecture/ARCH-OVERVIEW.md
/Users/dungngo97/Documents/bot-core/specs/02-design/2.1-architecture/ARCH-SECURITY.md
/Users/dungngo97/Documents/bot-core/specs/02-design/2.3-api/API-PYTHON-AI.md
/Users/dungngo97/Documents/bot-core/specs/02-design/2.2-database/DB-SCHEMA.md
/Users/dungngo97/Documents/bot-core/specs/02-design/2.2-database/DB-INDEXES.md
/Users/dungngo97/Documents/bot-core/specs/02-design/2.5-components/COMP-PYTHON-ML.md
/Users/dungngo97/Documents/bot-core/specs/04-deployment/4.1-infrastructure/INFRA-REQUIREMENTS.md
/Users/dungngo97/Documents/bot-core/specs/04-deployment/4.1-infrastructure/INFRA-DOCKER.md
/Users/dungngo97/Documents/bot-core/specs/05-operations/5.1-operations-manual/OPS-MANUAL.md
/Users/dungngo97/Documents/bot-core/specs/05-operations/5.2-troubleshooting/TROUBLESHOOTING.md
/Users/dungngo97/Documents/bot-core/specs/05-operations/5.3-disaster-recovery/DR-PLAN.md
/Users/dungngo97/Documents/bot-core/docs/PRODUCTION_DEPLOYMENT_GUIDE.md
/Users/dungngo97/Documents/bot-core/docs/OPERATIONS_MANUAL.md
/Users/dungngo97/Documents/bot-core/docs/TROUBLESHOOTING.md
/Users/dungngo97/Documents/bot-core/docs/HEALTH_CHECK_ENDPOINTS.md
/Users/dungngo97/Documents/bot-core/docs/BACKUP_RESTORE_GUIDE.md
```

---

## Implementation Steps

### Step 1: Update CLAUDE.md

**File**: `/Users/dungngo97/Documents/bot-core/CLAUDE.md`

**Changes**:
1. Update "Tech Stack" section - remove Kong, RabbitMQ, Celery, Prometheus, Grafana
2. Update "Service URLs" section - list only 6 core services
3. Remove any async task references
4. Update docker-compose examples

**Example**:
```markdown
# OLD
- **Backend**: Rust, Python FastAPI + Celery
- **Message Queue**: RabbitMQ
- **API Gateway**: Kong
- **Monitoring**: Prometheus + Grafana

# NEW
- **Backend**: Rust, Python FastAPI
- **Cache**: Redis (optional)
- **Database**: MongoDB
```

### Step 2: Update README.md

**File**: `/Users/dungngo97/Documents/bot-core/README.md`

**Changes**:
1. Update architecture diagram (remove 9 services)
2. Update tech stack list
3. Update "Getting Started" - remove enterprise service setup
4. Update docker-compose commands (remove profiles)

### Step 3: Update TRACEABILITY_MATRIX.md

**File**: `/Users/dungngo97/Documents/bot-core/specs/TRACEABILITY_MATRIX.md`

**Changes**:
1. Mark FR-ASYNC-* requirements as **DEPRECATED/REMOVED**
2. Update status column: "✅ Implemented" → "⚠️ DEPRECATED (Service Removed 2026-02-15)"
3. Add note: "Kong, RabbitMQ, Celery, Prometheus, Grafana removed - see [Docker Cleanup Plan](../plans/20260215-2100-docker-cleanup-optimization/plan.md)"
4. Remove from active requirement count

### Step 4: Delete Deprecated Documentation

**Delete these files**:
```bash
rm docs/fixes/RABBITMQ_PASSWORD_FIX.md
rm docs/guides/ASYNC_TASKS_README.md
rm specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md
rm specs/03-testing/3.2-test-cases/TC-ASYNC.md
rm specs/04-deployment/4.3-monitoring/MON-METRICS.md
rm specs/04-deployment/4.3-monitoring/MON-LOGGING.md
```

**Optional**: Create deprecation notice at top of files before deletion:
```markdown
# ⚠️ DEPRECATED - SERVICE REMOVED 2026-02-15

This document is no longer applicable. Kong/RabbitMQ/Celery/Prometheus/Grafana services were removed from the project.
See: [Docker Cleanup Plan](../../plans/20260215-2100-docker-cleanup-optimization/plan.md)

---

[Original content below...]
```

### Step 5: Update Architecture Specs

**File**: `specs/02-design/2.1-architecture/ARCH-OVERVIEW.md`

**Changes**:
1. Update system architecture diagram
2. Remove Kong API Gateway section
3. Remove RabbitMQ/Celery async processing section
4. Remove Prometheus/Grafana monitoring section
5. Update service count: 20 containers → 6-7 core services
6. Update component diagram

**File**: `specs/02-design/2.1-architecture/ARCH-SECURITY.md`

**Changes**:
1. Remove Kong security section
2. Remove RabbitMQ credentials section
3. Update to direct API authentication (Rust JWT only)

### Step 6: Update API & Database Specs

**File**: `specs/02-design/2.3-api/API-PYTHON-AI.md`

**Changes**:
1. Remove Celery task endpoints (if any)
2. Keep only synchronous prediction endpoints
3. Update API architecture (no message queue)

**File**: `specs/02-design/2.2-database/DB-SCHEMA.md`

**Changes**:
1. Check for async task collections (celery_tasks, task_results)
2. Remove if present
3. Update collection count

**File**: `specs/02-design/2.2-database/DB-INDEXES.md`

**Changes**:
1. Remove indexes for async task collections
2. Update index count

### Step 7: Update Component Specs

**File**: `specs/02-design/2.5-components/COMP-PYTHON-ML.md`

**Changes**:
1. Remove Celery worker component
2. Update component diagram
3. Update to synchronous API only

### Step 8: Update Infrastructure Specs

**File**: `specs/04-deployment/4.1-infrastructure/INFRA-REQUIREMENTS.md`

**Changes**:
1. Remove Kong infrastructure requirements (Postgres)
2. Remove RabbitMQ requirements
3. Remove Prometheus/Grafana requirements
4. Update memory requirements (total down from ~10GB to ~5GB)
5. Update CPU requirements

**File**: `specs/04-deployment/4.1-infrastructure/INFRA-DOCKER.md`

**Changes**:
1. Update service list (20 → 6-7 services)
2. Remove profile documentation (api-gateway, messaging, monitoring)
3. Update docker-compose examples

### Step 9: Update Operations Specs

**File**: `specs/05-operations/5.1-operations-manual/OPS-MANUAL.md`

**Changes**:
1. Remove Kong operations section
2. Remove RabbitMQ operations section
3. Remove Celery operations section
4. Remove Prometheus/Grafana operations section
5. Update service management procedures (6-7 services only)

**File**: `specs/05-operations/5.2-troubleshooting/TROUBLESHOOTING.md`

**Changes**:
1. Remove Kong troubleshooting
2. Remove RabbitMQ troubleshooting
3. Remove Celery troubleshooting
4. Remove Prometheus/Grafana troubleshooting

**File**: `specs/05-operations/5.3-disaster-recovery/DR-PLAN.md`

**Changes**:
1. Remove RabbitMQ backup/recovery
2. Remove Prometheus data backup
3. Keep only MongoDB backup

### Step 10: Update Production Guides

**File**: `docs/PRODUCTION_DEPLOYMENT_GUIDE.md`

**Changes**:
1. Remove Kong deployment section
2. Remove RabbitMQ setup section
3. Remove Celery worker deployment
4. Remove Prometheus/Grafana monitoring setup
5. Update deployment checklist
6. Update docker-compose production commands

**File**: `docs/OPERATIONS_MANUAL.md`

**Changes**:
1. Remove service management for 9 services
2. Update health check procedures
3. Update monitoring procedures (no Prometheus)

**File**: `docs/TROUBLESHOOTING.md`

**Changes**:
1. Remove async task troubleshooting
2. Remove Kong troubleshooting
3. Remove monitoring troubleshooting

**File**: `docs/HEALTH_CHECK_ENDPOINTS.md`

**Changes**:
1. Remove Kong health check (port 8001)
2. Remove RabbitMQ health check (port 15672)
3. Remove Flower health check (port 5555)
4. Remove Prometheus health check (port 9090)
5. Remove Grafana health check (port 3001)
6. Keep only: MongoDB, Rust, Python, Frontend, MCP, Redis

**File**: `docs/BACKUP_RESTORE_GUIDE.md`

**Changes**:
1. Remove RabbitMQ queue backup section
2. Keep MongoDB backup only

---

## Todo List

### Priority 1: Critical Core Docs
- [ ] Update CLAUDE.md tech stack
- [ ] Update CLAUDE.md service URLs
- [ ] Update CLAUDE.md docker-compose examples
- [ ] Update README.md architecture diagram
- [ ] Update README.md tech stack
- [ ] Update README.md getting started
- [ ] Update TRACEABILITY_MATRIX.md (mark FR-ASYNC-* deprecated)
- [ ] Add deprecation note to traceability matrix

### Priority 2: Delete Deprecated Docs
- [ ] Delete docs/fixes/RABBITMQ_PASSWORD_FIX.md
- [ ] Delete docs/guides/ASYNC_TASKS_README.md
- [ ] Delete specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md
- [ ] Delete specs/03-testing/3.2-test-cases/TC-ASYNC.md
- [ ] Delete specs/04-deployment/4.3-monitoring/MON-METRICS.md
- [ ] Delete specs/04-deployment/4.3-monitoring/MON-LOGGING.md

### Priority 3: Update Specs (15+ files)
- [ ] Update ARCH-OVERVIEW.md (system architecture)
- [ ] Update ARCH-SECURITY.md (remove Kong)
- [ ] Update API-PYTHON-AI.md (remove Celery endpoints)
- [ ] Update DB-SCHEMA.md (remove async collections)
- [ ] Update DB-INDEXES.md (remove task indexes)
- [ ] Update COMP-PYTHON-ML.md (remove Celery)
- [ ] Update INFRA-REQUIREMENTS.md (remove 9 services)
- [ ] Update INFRA-DOCKER.md (update service count)
- [ ] Update OPS-MANUAL.md (remove 9 services)
- [ ] Update TROUBLESHOOTING.md (remove sections)
- [ ] Update DR-PLAN.md (remove backups)

### Priority 4: Production Guides
- [ ] Update PRODUCTION_DEPLOYMENT_GUIDE.md (remove 9 services)
- [ ] Update OPERATIONS_MANUAL.md (remove service management)
- [ ] Update TROUBLESHOOTING.md (remove troubleshooting)
- [ ] Update HEALTH_CHECK_ENDPOINTS.md (remove checks)
- [ ] Update BACKUP_RESTORE_GUIDE.md (remove RabbitMQ)

### Verification
- [ ] grep docs/ for "Kong" (expect zero active doc matches)
- [ ] grep docs/ for "RabbitMQ" (expect zero active doc matches)
- [ ] grep docs/ for "Celery" (expect zero active doc matches)
- [ ] grep docs/ for "Prometheus" (expect zero active doc matches)
- [ ] grep docs/ for "Grafana" (expect zero active doc matches)
- [ ] grep specs/ for removed services (expect only deprecated/archive)
- [ ] Verify TRACEABILITY_MATRIX.md updated
- [ ] Verify FR-ASYNC-* marked deprecated

---

## Success Criteria

- ✅ CLAUDE.md updated with correct service list
- ✅ README.md updated with new architecture
- ✅ TRACEABILITY_MATRIX.md marks FR-ASYNC-* deprecated
- ✅ 6 deprecated docs deleted
- ✅ 15+ spec files updated
- ✅ 5+ production guides updated
- ✅ Zero references to removed services in active docs
- ✅ Archive docs left untouched

---

## Risk Assessment

**Risk Level**: 🟢 LOW

**Risks**:
- Breaking documentation links
- Missing some references

**Mitigation**:
- Use grep to find all references
- Update systematically (priority order)
- Keep archive docs for reference

---

## Output Artifacts

- Updated core docs (CLAUDE.md, README.md, TRACEABILITY_MATRIX.md)
- Deleted deprecated docs (6 files)
- Updated spec files (15+ files)
- Updated production guides (5+ files)
- Verification report (grep results)
