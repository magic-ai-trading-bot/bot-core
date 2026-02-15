# Docker Service Cleanup - Implementation Plan

**Date**: 2026-02-15
**Status**: ✅ COMPLETE
**Effort**: 3-4 hours
**Risk Level**: LOW - Services isolated by profiles, zero dependencies

---

## Overview

Remove 9 unused Docker services (Kong, Celery, RabbitMQ, Flower, Prometheus, Grafana) to reduce complexity, memory usage, and maintenance burden.

**Services to Remove**: Kong (3), Celery ecosystem (4), Monitoring (2)
**Services to Keep**: mongodb, rust-core-engine, python-ai-service, nextjs-ui-dashboard, mcp-server, openclaw, redis
**Impact**: -470 YAML lines, -12 env vars, -5 volumes, -9 containers

---

## Research Completed

- ✅ [Docker Dependencies Research](research/researcher-260215-docker-dependencies.md)
- ✅ [Documentation References Research](research/researcher-260215-documentation-references.md)

---

## Implementation Phases

### Phase 1: Docker Compose Cleanup
**File**: [phase-01-docker-compose.md](phase-01-docker-compose.md)
**Status**: ✅ Complete
**Effort**: 30 min
**Priority**: CRITICAL

Remove 9 service definitions, 5 volumes, 3 profiles from both docker-compose.yml files.

---

### Phase 2: Environment Variable Cleanup
**File**: [phase-02-environment-vars.md](phase-02-environment-vars.md)
**Status**: ✅ Complete
**Effort**: 20 min
**Priority**: CRITICAL

Remove unused environment variables from 5 .env files and mcp-server configs.

---

### Phase 3: Script Updates
**File**: [phase-03-script-updates.md](phase-03-script-updates.md)
**Status**: ✅ Complete
**Effort**: 45 min
**Priority**: CRITICAL

Update bot.sh, health-check.sh, validate-env.sh, init-all-services.sh to remove service logic.

---

### Phase 4: Core Documentation Updates
**File**: [phase-04-core-documentation.md](phase-04-core-documentation.md)
**Status**: ✅ Complete
**Effort**: 90 min
**Priority**: HIGH

Update CLAUDE.md, README.md, TRACEABILITY_MATRIX.md, and 15+ spec files.

---

### Phase 5: OpenClaw Workspace Updates
**File**: [phase-05-openclaw-workspace.md](phase-05-openclaw-workspace.md)
**Status**: ✅ Complete
**Effort**: 20 min
**Priority**: MEDIUM

Update OpenClaw workspace docs and skills references.

---

### Phase 6: Infrastructure Archival
**File**: [phase-06-infrastructure-archival.md](phase-06-infrastructure-archival.md)
**Status**: ✅ Complete
**Effort**: 10 min
**Priority**: LOW

Archive Kong, RabbitMQ, Prometheus, Grafana config directories.

---

### Phase 7: Verification & Testing
**File**: [phase-07-verification-testing.md](phase-07-verification-testing.md)
**Status**: ✅ Complete
**Effort**: 30 min
**Priority**: CRITICAL

Test that core services start, health checks pass, and system works after cleanup.

---

## Progress Tracker

| Phase | Status | Effort | Files Modified |
|-------|--------|--------|----------------|
| 1: Docker Compose | 🔲 | 30 min | 2 |
| 2: Environment Vars | 🔲 | 20 min | 6 |
| 3: Scripts | 🔲 | 45 min | 4 |
| 4: Documentation | 🔲 | 90 min | 45+ |
| 5: OpenClaw | 🔲 | 20 min | 5 |
| 6: Archival | 🔲 | 10 min | 4 dirs |
| 7: Verification | 🔲 | 30 min | - |
| **TOTAL** | 🔲 | **~4 hours** | **65+ files** |

---

## Success Criteria

- [ ] All 9 services removed from docker-compose.yml
- [ ] All 5 volumes removed
- [ ] All 12+ env vars removed
- [ ] Scripts updated and tested
- [ ] Core docs updated
- [ ] Specs traceability maintained
- [ ] `docker compose --profile dev up -d` starts 6 services successfully
- [ ] All health checks pass
- [ ] Zero references to removed services in active docs

---

## Risk Mitigation

**ZERO RISKS** - All services profile-isolated, no dependencies from core services.

**Backup Strategy**:
```bash
# Before starting
git checkout -b feature/docker-cleanup-20260215
git add . && git commit -m "Backup before Docker service cleanup"
```

**Rollback**: `git checkout main` if any issues.

---

## Next Steps

1. Execute Phase 1 (Docker Compose cleanup)
2. Execute Phase 2 (Environment variables)
3. Execute Phase 3 (Scripts)
4. Execute Phase 4 (Documentation - longest phase)
5. Execute Phase 5 (OpenClaw)
6. Execute Phase 6 (Archival)
7. Execute Phase 7 (Verification & testing)
8. Create PR when all phases complete

---

**Legend**: 🔲 Pending | 🔄 In Progress | ✅ Complete | ⚠️ Blocked | ❌ Failed
