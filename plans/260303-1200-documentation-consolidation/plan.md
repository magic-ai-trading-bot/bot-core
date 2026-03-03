# Documentation Consolidation Plan

**Date**: 2026-03-03
**Status**: Draft
**Priority**: High
**Scope**: Merge `docs/` + `specs/` → `specifications/`, review & fix all content

---

## Problem Statement

Project has 2 overlapping documentation folders:
- `specs/` (79 files, ~54K lines) — Well-structured but missing MCP/OpenClaw specs
- `docs/` (234+ files) — Messy: 97 reports, 78 archived, duplicates, outdated content

**Goal**: Single `specifications/` folder with accurate, comprehensive, up-to-date documentation.

---

## Phase Overview

| # | Phase | Status | Effort |
|---|-------|--------|--------|
| 1 | [Design new structure](./phase-01-design-structure.md) | Pending | Small |
| 2 | [Migrate specs/ content](./phase-02-migrate-specs.md) | Pending | Medium |
| 3 | [Review & integrate docs/ content](./phase-03-integrate-docs.md) | Pending | Large |
| 4 | [Create missing specs](./phase-04-create-missing-specs.md) | Pending | Medium |
| 5 | [Review & fix all content for accuracy](./phase-05-review-accuracy.md) | Pending | Large |
| 6 | [Update references & clean up](./phase-06-update-references.md) | Pending | Medium |

---

## Key Decisions

1. **Base structure**: Use `specs/` hierarchy (already A+ quality) as foundation
2. **Feature guides**: Merge `docs/features/` into requirements or new section
3. **Reports**: Archive to `plans/archive/` — not specification material
4. **Operational guides**: Keep in `specifications/05-operations/`
5. **Design system docs**: Move to `specifications/02-design/2.4-ui-ux/`
6. **Screenshots**: Move to `specifications/assets/screenshots/`
7. **Old archive**: Delete `docs/archive/` (78 legacy duplicates)

## New Structure

```
specifications/
├── README.md                              # Navigation hub
├── TRACEABILITY_MATRIX.md                 # Requirement ↔ code ↔ test mapping
├── _SPEC_TEMPLATE.md                      # Template
│
├── 01-requirements/
│   ├── 1.1-functional-requirements/       # 15 FR specs (existing 13 + MCP + OpenClaw)
│   ├── 1.2-non-functional-requirements/   # 5 NFR specs (as-is)
│   ├── 1.3-user-stories/                  # 3 user story specs (as-is)
│   └── 1.4-system-requirements/           # 3 system req specs (as-is)
│
├── 02-design/
│   ├── 2.1-architecture/                  # Architecture docs + system-architecture.md
│   ├── 2.2-database/                      # DB schema, indexes, migrations
│   ├── 2.3-api/                           # API specs (Rust, Python, WebSocket, MCP)
│   ├── 2.4-ui-ux/                         # UI components + design system docs
│   └── 2.5-components/                    # Component specs
│
├── 03-testing/
│   ├── 3.1-test-plan/                     # Master test plan
│   ├── 3.2-test-cases/                    # Test case specs
│   ├── 3.3-test-scenarios/                # Scenario specs
│   ├── 3.4-performance/                   # Performance test spec
│   └── 3.5-security/                      # Security test spec
│
├── 04-deployment/
│   ├── 4.1-infrastructure/                # Docker, K8s, VPS setup
│   ├── 4.2-cicd/                          # CI/CD pipelines, workflows
│   └── 4.3-monitoring/                    # Logging, metrics, alerting
│
├── 05-operations/
│   ├── 5.1-operations-manual/             # Daily operations + guides
│   ├── 5.2-troubleshooting/               # Troubleshooting guide
│   ├── 5.3-disaster-recovery/             # DR plan + backup/restore
│   └── 5.4-guides/                        # Quickstart, contributing, VPS deploy
│
├── 06-features/                           # NEW: Feature-specific documentation
│   ├── paper-trading.md
│   ├── real-trading.md
│   ├── ai-integration.md
│   ├── trading-strategies.md
│   ├── authentication.md
│   ├── websocket-realtime.md
│   ├── mcp-server.md
│   └── openclaw.md
│
└── assets/
    └── screenshots/                       # UI screenshots
```

## What Gets Deleted

1. `docs/archive/` (78 files) — Legacy duplicates, superseded content
2. `docs/reports/` (97 files) → Move to `plans/archive/reports/`
3. `docs/plans/` (9 files) → Already duplicated in `plans/`
4. `docs/certificates/` — Historical, not specification material
5. `docs/fixes/` (2 files) — Historical bug fix notes
6. `docs/testing/` (8 files) → Content merged into `03-testing/`
7. `docs/analysis/` (3 files) → Historical analysis, archive

## What Gets Merged

| Source | Destination | Action |
|--------|-------------|--------|
| `docs/features/*.md` | `specifications/06-features/` | Review & update |
| `docs/CONTRIBUTING.md` | `specifications/05-operations/5.4-guides/` | Review & update |
| `docs/TESTING_GUIDE.md` | `specifications/03-testing/` | Merge with TEST-PLAN.md |
| `docs/TROUBLESHOOTING.md` | `specifications/05-operations/5.2-troubleshooting/` | Merge with existing |
| `docs/PRODUCTION_DEPLOYMENT_GUIDE.md` | `specifications/04-deployment/` | Merge with INFRA docs |
| `docs/MONITORING_GUIDE.md` | `specifications/04-deployment/4.3-monitoring/` | Merge with MON docs |
| `docs/OPERATIONS_MANUAL.md` | `specifications/05-operations/5.1-operations-manual/` | Merge with OPS-MANUAL |
| `docs/design-system-*.md` | `specifications/02-design/2.4-ui-ux/` | Merge with UI docs |
| `docs/database/DATABASE_SETUP.md` | `specifications/02-design/2.2-database/` | Merge with DB-SCHEMA |
| `docs/guides/` (select) | `specifications/05-operations/5.4-guides/` | Review & keep useful |
| `docs/quickstart/` | `specifications/05-operations/5.4-guides/` | Consolidate |

## Risk Assessment

- **High**: Broken references in CLAUDE.md, code @spec tags, other docs
  - Mitigation: Phase 6 systematically updates all references
- **Medium**: Losing useful information during merge
  - Mitigation: Archive before delete, review each file
- **Low**: Merge conflicts if other branches reference old paths
  - Mitigation: Do in single commit on main branch

## Success Criteria

- [ ] Single `specifications/` folder replaces both `docs/` and `specs/`
- [ ] All content reviewed for accuracy against actual codebase
- [ ] Missing specs created (FR-MCP, FR-OPENCLAW)
- [ ] No broken references in CLAUDE.md or code
- [ ] All outdated content updated or removed
- [ ] TRACEABILITY_MATRIX updated
- [ ] `grep -r "docs/" CLAUDE.md` returns no stale references
- [ ] `grep -r "specs/" CLAUDE.md` returns no stale references
