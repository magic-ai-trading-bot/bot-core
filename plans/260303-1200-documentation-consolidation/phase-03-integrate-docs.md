# Phase 3: Review & Integrate docs/ Content

**Priority**: High | **Status**: Pending | **Effort**: Large

## Overview

Review every file in `docs/`, decide keep/merge/archive/delete, integrate valuable content into `specifications/`.

## Decision Matrix

### KEEP & MERGE (valuable content → specifications/)

| Source File | Target | Action |
|-------------|--------|--------|
| `docs/features/paper-trading.md` | `specifications/06-features/paper-trading.md` | Review & update |
| `docs/features/ai-integration.md` | `specifications/06-features/ai-integration.md` | Review & update |
| `docs/features/trading-strategies.md` | `specifications/06-features/trading-strategies.md` | Review & update |
| `docs/features/authentication.md` | `specifications/06-features/authentication.md` | Review & update |
| `docs/features/websocket-realtime.md` | `specifications/06-features/websocket-realtime.md` | Review & update |
| `docs/features/ai-auto-reversal.md` | Merge into `ai-integration.md` | Consolidate |
| `docs/features/signal-reversal.md` | Merge into `trading-strategies.md` | Consolidate |
| `docs/CONTRIBUTING.md` | `specifications/05-operations/5.4-guides/CONTRIBUTING.md` | Review & update |
| `docs/TESTING_GUIDE.md` | Merge into `specifications/03-testing/3.1-test-plan/` | Cross-check with TEST-PLAN.md |
| `docs/TROUBLESHOOTING.md` | Merge into `specifications/05-operations/5.2-troubleshooting/` | Merge with existing TROUBLESHOOTING.md |
| `docs/PRODUCTION_DEPLOYMENT_GUIDE.md` | `specifications/04-deployment/PRODUCTION_DEPLOYMENT_GUIDE.md` | Review & update |
| `docs/MONITORING_GUIDE.md` | Merge into `specifications/04-deployment/4.3-monitoring/` | Merge with MON-* docs |
| `docs/OPERATIONS_MANUAL.md` | Merge into `specifications/05-operations/5.1-operations-manual/` | Merge with OPS-MANUAL.md |
| `docs/BACKUP_RESTORE_GUIDE.md` | `specifications/05-operations/5.3-disaster-recovery/` | Merge with DR-PLAN.md |
| `docs/design-system-cryptocurrency-trading-dashboard.md` | `specifications/02-design/2.4-ui-ux/DESIGN-SYSTEM.md` | Consolidate 4 design docs into 1 |
| `docs/design-system-implementation-guide.md` | Merge into DESIGN-SYSTEM.md | Consolidate |
| `docs/design-system-reference.md` | Merge into DESIGN-SYSTEM.md | Consolidate |
| `docs/design-color-palette-reference.md` | Merge into DESIGN-SYSTEM.md | Consolidate |
| `docs/database/DATABASE_SETUP.md` | Merge into `specifications/02-design/2.2-database/` | Merge with DB-SCHEMA.md |
| `docs/HEALTH_CHECK_ENDPOINTS.md` | `specifications/02-design/2.3-api/API-HEALTH-CHECKS.md` | Review & update |
| `docs/quickstart/DEVELOPMENT_QUICKSTART.md` | `specifications/05-operations/5.4-guides/QUICKSTART.md` | Consolidate 3 quickstarts into 1 |
| `docs/quickstart/PRODUCTION_QUICKSTART.md` | Merge into QUICKSTART.md | Consolidate |
| `docs/quickstart/STAGING_QUICKSTART.md` | Merge into QUICKSTART.md | Consolidate |
| `docs/guides/BOT_SCRIPT_GUIDE.md` | `specifications/05-operations/5.4-guides/BOT_SCRIPT_GUIDE.md` | Review & update |
| `docs/guides/VIETTEL_VPS_DEPLOYMENT_GUIDE.md` | `specifications/04-deployment/VPS_DEPLOYMENT.md` | Review & update |
| `docs/guides/START_WITH_NEW_KEY.md` | `specifications/05-operations/5.4-guides/` | Review & keep |
| `docs/runbooks/DEPLOYMENT_RUNBOOK.md` | `specifications/05-operations/5.1-operations-manual/` | Merge with OPS-MANUAL |
| `docs/runbooks/BACKUP_FAILURE_RUNBOOK.md` | `specifications/05-operations/5.3-disaster-recovery/` | Merge with DR-PLAN |
| `docs/screenshots/` | `specifications/assets/screenshots/` | Move all PNGs |

### ARCHIVE (historical value, not specs)

Move to `plans/archive/docs-archive-260303/`:
- `docs/reports/` (97 files) — Historical technical reports
- `docs/plans/` (9 files) — Old implementation plans
- `docs/testing/` (8 files) — Old test reports
- `docs/analysis/` (3 files) — Old code analysis
- `docs/fixes/` (2 files) — Bug fix summaries
- `docs/certificates/` (1 file) — Quality certificate
- `docs/guides/CACH_HOAT_DONG_CUA_BOT.md` (2,812 lines) — Vietnamese bot guide (archive, too large)
- Remaining `docs/guides/` files (historical quick guides)
- Remaining top-level docs that are reports/summaries

### DELETE (superseded duplicates)

- `docs/archive/` (78 files) — All legacy duplicates

## Steps

1. Create target directories in `specifications/`
2. Process KEEP & MERGE files (review each, update content, move)
3. Move screenshots to `specifications/assets/screenshots/`
4. Archive historical content to `plans/archive/`
5. Delete `docs/archive/` (78 legacy files)
6. Delete remaining `docs/` directory

## Todo

- [ ] Create 06-features/ directory
- [ ] Create 5.4-guides/ directory
- [ ] Merge feature docs (7 files → 8 feature docs)
- [ ] Merge operational guides
- [ ] Merge design system docs (4 → 1)
- [ ] Merge quickstart docs (3 → 1)
- [ ] Archive 120+ historical files
- [ ] Delete 78 legacy archive files
- [ ] Move screenshots
- [ ] Delete old docs/ directory

## Success Criteria

- All valuable content preserved in `specifications/`
- No duplicate content remaining
- Historical content archived properly
- `docs/` directory removed
