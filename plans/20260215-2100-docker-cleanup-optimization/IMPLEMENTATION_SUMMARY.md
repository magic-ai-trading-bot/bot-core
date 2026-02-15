# Docker Service Cleanup - Implementation Summary

**Date Created**: 2026-02-15
**Plan Location**: `/Users/dungngo97/Documents/bot-core/plans/20260215-2100-docker-cleanup-optimization/`
**Status**: READY FOR IMPLEMENTATION
**Total Effort**: 3-4 hours
**Risk Level**: LOW

---

## Executive Summary

Remove 9 unused Docker services (Kong, Celery, RabbitMQ, Flower, Prometheus, Grafana) from BotCore to reduce complexity, memory usage, and maintenance burden.

**Impact**:
- Services removed: 9 (Kong 3, Celery ecosystem 4, Monitoring 2)
- Services kept: 6-7 core (MongoDB, Rust, Python, Frontend, MCP, OpenClaw, Redis)
- YAML lines removed: ~470
- Env vars removed: 12+
- Volumes removed: 5
- Files updated: 65+

**Safety**: ZERO RISK - All services isolated by profiles, no dependencies from core services.

---

## Plan Structure

### Main Plan
- **[plan.md](plan.md)** - Overview, progress tracker, success criteria

### Research (Completed)
- **[research/researcher-260215-docker-dependencies.md](research/researcher-260215-docker-dependencies.md)** - Docker compose analysis
- **[research/researcher-260215-documentation-references.md](research/researcher-260215-documentation-references.md)** - 65+ docs to update

### Implementation Phases (7 Phases)

1. **[phase-01-docker-compose.md](phase-01-docker-compose.md)** - 30 min, CRITICAL
   - Remove 9 services from docker-compose.yml (both locations)
   - Remove 5 volumes
   - Update redis profile, remove PROMETHEUS_URL from mcp-server

2. **[phase-02-environment-vars.md](phase-02-environment-vars.md)** - 20 min, CRITICAL
   - Remove 12+ env vars from 5 .env files
   - RABBITMQ_*, KONG_DB_PASSWORD, GRAFANA_PASSWORD, PROMETHEUS_ENDPOINT

3. **[phase-03-script-updates.md](phase-03-script-updates.md)** - 45 min, CRITICAL
   - Update bot.sh (remove --with-enterprise, profiles)
   - Update init-all-services.sh, health-check.sh, validate-env.sh

4. **[phase-04-core-documentation.md](phase-04-core-documentation.md)** - 90 min, HIGH
   - Update CLAUDE.md, README.md, TRACEABILITY_MATRIX.md
   - Delete 6 deprecated docs
   - Update 15+ spec files

5. **[phase-05-openclaw-workspace.md](phase-05-openclaw-workspace.md)** - 20 min, MEDIUM
   - Update OpenClaw ARCHITECTURE.md
   - Update backend skills (6 files)
   - Update docker-compose skill

6. **[phase-06-infrastructure-archival.md](phase-06-infrastructure-archival.md)** - 10 min, LOW
   - Add README.DEPRECATED.md to 4 config directories
   - Preserve configs for future reference

7. **[phase-07-verification-testing.md](phase-07-verification-testing.md)** - 30 min, CRITICAL
   - Docker compose validation
   - Service startup test (6-7 services)
   - Health checks, API tests, grep verification

---

## Quick Start Guide

### Prerequisites
```bash
# Backup current state
cd /Users/dungngo97/Documents/bot-core
git checkout -b feature/docker-cleanup-20260215
git add . && git commit -m "Backup before Docker service cleanup"
```

### Execution Order
```bash
# Phase 1: Docker Compose
# - Edit docker-compose.yml (both locations)
# - Remove 9 services, 5 volumes
# - Update redis profile, remove PROMETHEUS_URL

# Phase 2: Environment Variables
# - Edit 5 .env files
# - Remove RABBITMQ_*, KONG_DB_PASSWORD, GRAFANA_PASSWORD, PROMETHEUS_ENDPOINT

# Phase 3: Scripts
# - Edit bot.sh, init-all-services.sh, health-check.sh, validate-env.sh
# - Remove service logic

# Phase 4: Documentation
# - Update CLAUDE.md, README.md, TRACEABILITY_MATRIX.md
# - Delete 6 deprecated docs
# - Update 15+ spec files

# Phase 5: OpenClaw
# - Update OpenClaw workspace docs
# - Update backend skills

# Phase 6: Archival
# - Add README.DEPRECATED.md to 4 infrastructure dirs

# Phase 7: Verification
# - Test docker compose config
# - Start services: ./scripts/bot.sh dev
# - Run health checks
# - Verify all tests pass
```

---

## Success Criteria Checklist

### Code Changes
- [ ] 9 services removed from docker-compose.yml (both files)
- [ ] 5 volumes removed
- [ ] 12+ env vars removed from 5 .env files
- [ ] 4 scripts updated (bot.sh, init-all-services.sh, health-check.sh, validate-env.sh)

### Documentation Updates
- [ ] CLAUDE.md updated
- [ ] README.md updated
- [ ] TRACEABILITY_MATRIX.md updated (FR-ASYNC-* deprecated)
- [ ] 6 deprecated docs deleted
- [ ] 15+ spec files updated
- [ ] OpenClaw workspace updated
- [ ] 4 infrastructure dirs archived

### Verification
- [ ] `docker compose config --quiet` passes
- [ ] `docker compose --profile dev up -d` starts 6-7 services
- [ ] All health checks pass
- [ ] Rust API responds (localhost:8080/api/health)
- [ ] Python API responds (localhost:8000/health)
- [ ] Frontend loads (localhost:3000/)
- [ ] MCP Server responds (localhost:8090/health)
- [ ] Zero grep matches for removed services
- [ ] All scripts work correctly

---

## Risk Mitigation

**Risk Level**: 🟢 LOW

### Why Low Risk?
1. Services isolated by profiles - never started
2. Zero dependencies from core services
3. No code references removed services
4. Easy rollback via git

### Backup Strategy
```bash
# Before starting
git checkout -b feature/docker-cleanup-20260215
git add . && git commit -m "Backup before cleanup"
```

### Rollback Plan
```bash
# If any issues
docker compose down
git checkout main
git branch -D feature/docker-cleanup-20260215
./scripts/bot.sh dev
```

---

## Expected Improvements

### Reduced Complexity
- Containers: 20 → 6-7 (65% reduction)
- Docker profiles: 6 → 3
- Env vars: 40+ → 28 (30% reduction)
- YAML lines: 854 → ~380 (55% reduction)

### Reduced Resource Usage
- Memory: ~5GB saved (if services were running)
- Disk: ~2GB saved (volumes + images)
- Network: Simpler topology (no API gateway)

### Improved Maintainability
- Fewer services to monitor
- Simpler deployment
- Clearer architecture
- Less documentation to maintain

---

## Next Steps After Completion

1. **Create PR**:
   ```bash
   git add .
   git commit -m "refactor: remove unused Docker services (Kong, Celery, RabbitMQ, Prometheus, Grafana)"
   gh pr create --title "Docker Service Cleanup" --body "$(cat <<'EOF'
   ## Summary
   - Remove 9 unused Docker services (Kong, Celery, RabbitMQ, Flower, Prometheus, Grafana)
   - Update scripts, documentation, specs
   - Archive infrastructure configs for reference

   ## Changes
   - Docker: -9 services, -5 volumes, -470 YAML lines
   - Environment: -12 env vars
   - Scripts: Updated 4 files
   - Docs: Updated 45+ files, deleted 6 deprecated

   ## Test Plan
   - [x] Docker compose validation passes
   - [x] All 6-7 core services start
   - [x] Health checks pass
   - [x] API tests pass
   - [x] Zero grep matches for removed services

   ## Risk: LOW
   Services were isolated, unused, zero dependencies.

   See: [Implementation Plan](plans/20260215-2100-docker-cleanup-optimization/plan.md)
   EOF
   )"
   ```

2. **Update CHANGELOG.md**:
   ```markdown
   ## [Unreleased]
   ### Removed
   - Kong API Gateway (3 containers) - unused, direct API communication
   - RabbitMQ + Celery + Flower (4 containers) - unused, no async tasks
   - Prometheus + Grafana (2 containers) - unused, no metrics endpoints

   ### Changed
   - Simplified docker-compose.yml (20 → 6-7 services)
   - Reduced environment variables (40+ → 28)
   - Updated scripts for core services only
   ```

3. **Announce to team** (if applicable)

---

## Files Created

### Plan Files (8 files)
- plan.md - Main plan overview
- phase-01-docker-compose.md - Docker compose cleanup
- phase-02-environment-vars.md - Environment variables
- phase-03-script-updates.md - Script updates
- phase-04-core-documentation.md - Documentation updates
- phase-05-openclaw-workspace.md - OpenClaw workspace
- phase-06-infrastructure-archival.md - Infrastructure archival
- phase-07-verification-testing.md - Verification & testing
- IMPLEMENTATION_SUMMARY.md - This file

### Research Files (2 files)
- research/researcher-260215-docker-dependencies.md
- research/researcher-260215-documentation-references.md

### Output Location
- `/Users/dungngo97/Documents/bot-core/plans/20260215-2100-docker-cleanup-optimization/`
- Active plan state: `.claude/active-plan`

---

## Support

**Questions?** Read the detailed phase files for step-by-step instructions.

**Issues?** Check [phase-07-verification-testing.md](phase-07-verification-testing.md) for rollback plan.

**Finance Safety**: This cleanup does NOT touch trading logic, database, or core engine. Zero impact on trading functionality.

---

**Status**: PLAN COMPLETE - READY FOR IMPLEMENTATION

Execute phases in order, verify after each phase, create PR when all tests pass.
