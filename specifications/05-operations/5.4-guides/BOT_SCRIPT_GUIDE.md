# Bot Core Scripts Inventory

**Updated**: 2026-03-03 | **Script count**: 52 scripts in `scripts/`
**Related specs**: `specifications/04-deployment/`, `specifications/05-operations/`

---

## Quick Reference: bot.sh (Main Control)

```bash
./scripts/bot.sh [COMMAND] [OPTIONS]
```

| Command | Purpose | Key Options |
|---------|---------|------------|
| `start` | Start services (production mode) | `--memory-optimized`, `--with-rabbitmq`, `--with-enterprise` |
| `dev` | Start services (dev mode, hot reload) | `--memory-optimized`, `--with-rabbitmq` |
| `stop` | Stop all containers (data preserved) | — |
| `restart` | Stop then start | same as start |
| `build` | Build Docker images | `--service <name>` |
| `test` | Run async task tests | `--coverage`, `--all` |
| `status` | Show container status + resource usage | — |
| `logs` | Follow logs | `--service <name>` |
| `clean` | Remove all containers + volumes (DATA LOST) | — |
| `verify` | Check prerequisites + env | — |
| `help` | Show usage | — |

Service URLs: Frontend :3000 | Rust API :8080 | Python AI :8000 | MCP :8090 | OpenClaw :18789

---

## All Scripts by Category

### Deployment

| Script | Purpose | Usage |
|--------|---------|-------|
| `bot.sh` | Main system control script | `./scripts/bot.sh start --memory-optimized` |
| `deploy.sh` | Deploy all services to Fly.io | `./scripts/deploy.sh` |
| `deploy-to-viettel-vps.sh` | Automated deploy to Viettel VPS | `./scripts/deploy-to-viettel-vps.sh` |
| `deploy-local.sh` | Local deployment with health checks, backup, rollback | `./scripts/deploy-local.sh` |
| `rollback.sh` | Restore from most recent backup | `./scripts/rollback.sh` |
| `pre-deployment-check.sh` | Validate all checks before deploy | `./scripts/pre-deployment-check.sh` |
| `build-and-push.sh` | Build Docker images and push to registry | `./scripts/build-and-push.sh` |
| `pull-images.sh` | Pull pre-built images from registry | `./scripts/pull-images.sh` |
| `demo.sh` | Showcase different deployment options | `./scripts/demo.sh` |

**Related spec**: `specifications/04-deployment/`

---

### VPS Setup & Initialization

| Script | Purpose | Usage |
|--------|---------|-------|
| `vps-auto-setup.sh` | Full VPS setup from scratch (Ubuntu 22.04) | `./scripts/vps-auto-setup.sh` |
| `vps-init-services.sh` | Initialize/restart services on VPS | `./scripts/vps-init-services.sh` |
| `init-all-services.sh` | Wait for services healthy + seed initial data | `./scripts/init-all-services.sh` |
| `init-db.sh` | Initialize MongoDB database + run migrations | `./scripts/init-db.sh` |
| `init-mongodb-seed.sh` | Seed MongoDB with sample data on first startup | `./scripts/init-mongodb-seed.sh` |
| `seed-mongodb.js` | MongoDB seed data (run via mongosh) | `docker exec mongodb mongosh ... < scripts/seed-mongodb.js` |
| `generate-secrets.sh` | Generate all secure secrets for .env | `./scripts/generate-secrets.sh` |
| `verify-setup.sh` | Verify Docker, env, connectivity | `./scripts/verify-setup.sh` |
| `validate-env.sh` | Validate all required environment variables | `./scripts/validate-env.sh` |
| `validate-db.sh` | Validate DB setup, schema, indexes | `./scripts/validate-db.sh` |
| `reorganize-structure.sh` | Reorganize folder structure (optional, one-time) | `./scripts/reorganize-structure.sh` |

**Related spec**: `specifications/04-deployment/4.1-infrastructure/`

---

### Docker Management

| Script | Purpose | Usage |
|--------|---------|-------|
| `docker-cleanup.sh` | Remove stopped containers, dangling images, unused volumes | `./scripts/docker-cleanup.sh` |
| `docker-registry-setup.sh` | Configure auth for Docker registries (GitHub/DockerHub/private) | `./scripts/docker-registry-setup.sh` |
| `verify-docker-registry-setup.sh` | Verify Docker registry infrastructure is configured | `./scripts/verify-docker-registry-setup.sh` |
| `manage.sh` | Manage Fly.io deployed services | `./scripts/manage.sh` |

**Cron recommendation** for `docker-cleanup.sh`: `0 3 * * 0` (weekly Sunday 3am)

---

### SSL / TLS

| Script | Purpose | Usage |
|--------|---------|-------|
| `generate-ssl-certs.sh` | Generate self-signed SSL certs for dev | `./scripts/generate-ssl-certs.sh` |
| `setup-letsencrypt.sh` | Set up Let's Encrypt for production | `./scripts/setup-letsencrypt.sh` |
| `renew-ssl.sh` | Renew Let's Encrypt certificates | `./scripts/renew-ssl.sh` |
| `verify-ssl-security.sh` | Verify SSL configuration and security | `./scripts/verify-ssl-security.sh` |

**Cron recommendation** for `renew-ssl.sh`: `0 2 * * *` (daily 2am)
**Related spec**: `ARCH-SECURITY-004`

---

### Backup & Disaster Recovery

| Script | Purpose | Usage |
|--------|---------|-------|
| `backup/` | Backup scripts directory | `ls scripts/backup/` |
| `backup-storage/` | Backup storage utilities directory | — |
| `backup-status-report.sh` | Generate daily backup status report | `./scripts/backup-status-report.sh` |
| `check-backup-health.sh` | Check backup health and completeness | `./scripts/check-backup-health.sh` |
| `verify-backups.sh` | Verify backup integrity and restorability | `./scripts/verify-backups.sh` |
| `cleanup-old-backups.sh` | Enforce retention policy, remove old backups | `./scripts/cleanup-old-backups.sh` |
| `test-dr.sh` | Run disaster recovery drill | `./scripts/test-dr.sh` |
| `restore/` | Restore scripts directory | `ls scripts/restore/` |

**Related spec**: `specifications/05-operations/5.3-disaster-recovery/DR-PLAN.md`
**NFR specs**: NFR-OPS-013 (verify), NFR-OPS-014 (cleanup), NFR-OPS-016 (health), NFR-OPS-017 (report), NFR-OPS-018 (DR testing)

---

### Security

| Script | Purpose | Usage |
|--------|---------|-------|
| `security-scan.sh` | Full security scan: secrets, deps, OWASP | `./scripts/security-scan.sh` |
| `setup-openai-key.sh` | Safely configure xAI/OpenAI API key in .env | `./scripts/setup-openai-key.sh` |
| `dismiss-alerts.sh` | Dismiss specific GitHub code scanning alerts | `./scripts/dismiss-alerts.sh` |
| `dismiss-security-alerts.sh` | Dismiss known safe GitHub Dependabot alerts | `./scripts/dismiss-security-alerts.sh` |

**Related spec**: `specifications/02-design/2.1-architecture/ARCH-SECURITY.md`

---

### Monitoring & Performance

| Script | Purpose | Usage |
|--------|---------|-------|
| `health-check.sh` | Check health of all services (HTTP + process) | `./scripts/health-check.sh` |
| `monitor_performance.py` | Track win rate, avg profit, Sharpe vs targets | `python3 scripts/monitor_performance.py --continuous` |
| `monitor-dashboard.sh` | Real-time cost monitoring terminal dashboard | `./scripts/monitor-dashboard.sh` |
| `daily_report.sh` | Generate daily performance report | `./scripts/daily_report.sh [--week]` |
| `quality-metrics.sh` | Code quality analysis (lint, complexity, duplication) | `./scripts/quality-metrics.sh` |

**Health check targets**: Rust API :8080, Python :8000, MCP :8090, Frontend :3000
**Related spec**: `specifications/05-operations/5.1-monitoring/`, FR-OPS-003

---

### Code Quality & Spec Validation

| Script | Purpose | Usage |
|--------|---------|-------|
| `check-rust.sh` | Rust fmt + clippy + tests before commit | `./scripts/check-rust.sh` |
| `validate-specs.py` | Validate all spec files in sync with code | `python scripts/validate-specs.py [--verbose] [--fix]` |
| `validate-spec-tags.py` | Verify @spec tags in source code match spec files | `python scripts/validate-spec-tags.py` |
| `add-spec-tags.sh` | Add @spec traceability tags to source files | `./scripts/add-spec-tags.sh` |
| `auto-tag-code.py` | Auto-add @spec tags based on TRACEABILITY_MATRIX.md | `python scripts/auto-tag-code.py` |

**Required before commit**: `./scripts/check-rust.sh` for Rust changes
**Required before push**: `python scripts/validate-specs.py` (must show 0 errors)

---

### Dev Tools

| Script | Purpose | Usage |
|--------|---------|-------|
| `sync-openclaw-knowledge.sh` | Generate OpenClaw workspace knowledge from codebase | `./scripts/sync-openclaw-knowledge.sh` |
| `test_strategies_live.py` | Test all 5 strategies with real Binance market data | `python3 scripts/test_strategies_live.py` |

**sync-openclaw-knowledge.sh** outputs:
- `openclaw/workspace/STRATEGIES.md` — actual strategy params from source
- `openclaw/workspace/SOUL.md` — system prompt with architecture knowledge

---

## Common Workflows

### First-Time Setup (Local)
```bash
cp .env.example .env
./scripts/generate-secrets.sh
./scripts/validate-env.sh
./scripts/bot.sh start --memory-optimized
./scripts/health-check.sh
```

### First-Time VPS Setup
```bash
# On fresh Ubuntu 22.04 VPS
./scripts/vps-auto-setup.sh
./scripts/generate-secrets.sh
./scripts/deploy-to-viettel-vps.sh
./scripts/vps-init-services.sh
```

### Pre-Deploy Checklist
```bash
./scripts/pre-deployment-check.sh
./scripts/validate-env.sh
./scripts/validate-specs.py
./scripts/security-scan.sh
```

### Backup Verification (Weekly)
```bash
./scripts/verify-backups.sh
./scripts/check-backup-health.sh
./scripts/backup-status-report.sh
```

### After Code Changes (Rust)
```bash
./scripts/check-rust.sh           # fmt + clippy + tests
./scripts/validate-spec-tags.py   # ensure @spec tags present
./scripts/bot.sh restart
```

### Sync AI Knowledge
```bash
./scripts/sync-openclaw-knowledge.sh
# Then rebuild openclaw container to pick up changes
./scripts/bot.sh build --service openclaw
./scripts/bot.sh restart
```

---

## Environment Variables Reference

Key vars expected by most scripts:

| Variable | Description | Required |
|----------|-------------|---------|
| `BINANCE_API_KEY` | Binance mainnet API key | Real trading only |
| `BINANCE_SECRET_KEY` | Binance mainnet secret | Real trading only |
| `BINANCE_FUTURES_TESTNET_API_KEY` | Testnet futures key | Default mode |
| `BINANCE_TESTNET` | `true`/`false` | Always set |
| `TRADING_ENABLED` | Enable auto-trading | Default: false |
| `DATABASE_URL` | MongoDB connection string | All services |
| `JWT_SECRET` | JWT signing secret | Rust API |
| `MCP_AUTH_TOKEN` | MCP server bearer token | MCP + OpenClaw |
| `TELEGRAM_BOT_TOKEN` | Telegram bot token | OpenClaw |
| `XAI_API_KEY` | xAI/Grok API key | AI service |

Generate all secrets: `./scripts/generate-secrets.sh`

---

## Script Exit Codes

All scripts use `set -e` (exit on error). Standard codes:
- `0` — Success
- `1` — General error
- `2` — Critical error (validate-specs.py specific)

`validate-specs.py` exits:
- `0` — All validations passed
- `1` — Validation errors found
- `2` — Critical errors (missing files, parse errors)
