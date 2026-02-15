# Docker Cleanup - Quick Reference Card

**Plan**: `/Users/dungngo97/Documents/bot-core/plans/20260215-2100-docker-cleanup-optimization/`
**Status**: READY
**Effort**: 3-4 hours
**Risk**: LOW

---

## 📋 Execution Checklist

### Before Starting
```bash
cd /Users/dungngo97/Documents/bot-core
git checkout -b feature/docker-cleanup-20260215
git add . && git commit -m "Backup before Docker cleanup"
```

### Phase Execution
- [ ] **Phase 1** (30 min) - Docker Compose → [phase-01-docker-compose.md](phase-01-docker-compose.md)
- [ ] **Phase 2** (20 min) - Environment Vars → [phase-02-environment-vars.md](phase-02-environment-vars.md)
- [ ] **Phase 3** (45 min) - Scripts → [phase-03-script-updates.md](phase-03-script-updates.md)
- [ ] **Phase 4** (90 min) - Documentation → [phase-04-core-documentation.md](phase-04-core-documentation.md)
- [ ] **Phase 5** (20 min) - OpenClaw → [phase-05-openclaw-workspace.md](phase-05-openclaw-workspace.md)
- [ ] **Phase 6** (10 min) - Archival → [phase-06-infrastructure-archival.md](phase-06-infrastructure-archival.md)
- [ ] **Phase 7** (30 min) - Verification → [phase-07-verification-testing.md](phase-07-verification-testing.md)

### After Each Phase
```bash
# Run verification script
./plans/20260215-2100-docker-cleanup-optimization/verify-cleanup.sh
```

---

## 🎯 What to Remove

### Services (9)
- Kong: `kong`, `kong-database`, `kong-migration`
- Celery: `rabbitmq`, `celery-worker`, `celery-beat`, `flower`
- Monitoring: `prometheus`, `grafana`

### Volumes (5)
- `rabbitmq_data`, `kong_data`, `prometheus_data`, `grafana_data`, `flower_tmp`

### Env Vars (12+)
- `RABBITMQ_USER`, `RABBITMQ_PASSWORD`, `RABBITMQ_HOST`, `RABBITMQ_PORT`, `RABBITMQ_VHOST`
- `KONG_DB_PASSWORD`
- `GRAFANA_PASSWORD`
- `PROMETHEUS_ENDPOINT`
- `PROMETHEUS_URL` (mcp-server only)

### Profiles (3)
- `api-gateway`, `messaging`, `monitoring`

---

## 🔍 What to Keep

### Services (6-7)
- `mongodb`, `rust-core-engine`, `python-ai-service`, `nextjs-ui-dashboard`
- `mcp-server`, `openclaw`
- `redis` (optional, profile: `redis`)

### Profiles (3)
- `dev`, `prod`, `redis`

### Env Vars
- All core service vars (MongoDB, Rust, Python, Frontend, MCP, OpenClaw)
- `REDIS_PASSWORD` (keep for future use)

---

## 📝 Files to Update

### Docker (2 files)
- `docker-compose.yml`
- `infrastructure/docker/docker-compose.yml`

### Environment (5 files)
- `.env`
- `.env.example`
- `.env.example.secure`
- `.env.production.example`
- `python-ai-service/.env.example`

### Scripts (4 files)
- `scripts/bot.sh`
- `scripts/init-all-services.sh`
- `scripts/health-check.sh`
- `scripts/validate-env.sh`

### Core Docs (3 files)
- `CLAUDE.md`
- `README.md`
- `specs/TRACEABILITY_MATRIX.md`

### Deprecated Docs (6 files to DELETE)
- `docs/fixes/RABBITMQ_PASSWORD_FIX.md`
- `docs/guides/ASYNC_TASKS_README.md`
- `specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md`
- `specs/03-testing/3.2-test-cases/TC-ASYNC.md`
- `specs/04-deployment/4.3-monitoring/MON-METRICS.md`
- `specs/04-deployment/4.3-monitoring/MON-LOGGING.md`

### Specs (15+ files)
- Architecture: `ARCH-OVERVIEW.md`, `ARCH-SECURITY.md`
- API: `API-PYTHON-AI.md`
- Database: `DB-SCHEMA.md`, `DB-INDEXES.md`
- Components: `COMP-PYTHON-ML.md`
- Infrastructure: `INFRA-REQUIREMENTS.md`, `INFRA-DOCKER.md`
- Operations: `OPS-MANUAL.md`, `TROUBLESHOOTING.md`, `DR-PLAN.md`

### Production Guides (5 files)
- `docs/PRODUCTION_DEPLOYMENT_GUIDE.md`
- `docs/OPERATIONS_MANUAL.md`
- `docs/TROUBLESHOOTING.md`
- `docs/HEALTH_CHECK_ENDPOINTS.md`
- `docs/BACKUP_RESTORE_GUIDE.md`

### OpenClaw (5 files)
- `openclaw/workspace/ARCHITECTURE.md`
- `.claude/skills/backend-development/*` (6 files)
- `.claude/skills/devops/references/docker-compose.md`

### Infrastructure (4 directories)
- Add `README.DEPRECATED.md` to:
  - `infrastructure/kong/`
  - `infrastructure/rabbitmq/`
  - `infrastructure/monitoring/`
  - `infrastructure/grafana/`

---

## ✅ Verification Commands

```bash
# Docker validation
docker compose config --quiet

# Env validation
./scripts/validate-env.sh

# Service startup
./scripts/bot.sh dev

# Health checks
./scripts/health-check.sh

# API tests
curl -f http://localhost:8080/api/health
curl -f http://localhost:8000/health
curl -f http://localhost:3000/

# Grep verification
grep -ri "kong\|rabbitmq\|celery\|prometheus\|grafana" \
  --exclude-dir=archive \
  --exclude-dir=infrastructure \
  .
```

---

## 🚨 Rollback Plan

```bash
# If anything breaks
docker compose down
git checkout main
git branch -D feature/docker-cleanup-20260215
./scripts/bot.sh dev
```

---

## 📊 Expected Results

### Before
- Services: 20 containers
- YAML lines: 854
- Env vars: 40+
- Volumes: 10

### After
- Services: 6-7 containers (65% reduction)
- YAML lines: ~380 (55% reduction)
- Env vars: 28 (30% reduction)
- Volumes: 5 (50% reduction)

### Savings
- Memory: ~5GB (if services were running)
- Disk: ~2GB (volumes + images)
- Complexity: Much simpler architecture

---

## 🎯 Final Verification

All must pass:
- [ ] `docker compose config --quiet` ✅
- [ ] `docker compose --profile dev up -d` starts 6-7 services ✅
- [ ] All health checks pass ✅
- [ ] Rust API responds ✅
- [ ] Python API responds ✅
- [ ] Frontend loads ✅
- [ ] Zero grep matches for removed services ✅
- [ ] `./verify-cleanup.sh` passes ✅

---

## 📚 Documentation

- **Main Plan**: [plan.md](plan.md)
- **Summary**: [IMPLEMENTATION_SUMMARY.md](IMPLEMENTATION_SUMMARY.md)
- **Research**: [research/](research/)
- **Phases**: [phase-01](phase-01-docker-compose.md) through [phase-07](phase-07-verification-testing.md)
- **Verification**: [verify-cleanup.sh](verify-cleanup.sh)

---

**Status**: READY FOR IMPLEMENTATION
**Next**: Execute Phase 1
