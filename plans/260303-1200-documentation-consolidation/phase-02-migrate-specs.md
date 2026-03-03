# Phase 2: Migrate specs/ Content

**Priority**: High | **Status**: Pending | **Effort**: Medium

## Overview

Move all `specs/` content into `specifications/`. The specs/ structure is already A+ quality — mostly a direct move with minor cleanup.

## Steps

1. Move `specs/01-requirements/` → `specifications/01-requirements/`
2. Move `specs/02-design/` → `specifications/02-design/`
3. Move `specs/03-testing/` → `specifications/03-testing/`
4. Move `specs/04-deployment/` → `specifications/04-deployment/`
5. Move `specs/05-operations/` → `specifications/05-operations/`
6. Move root meta files:
   - `TRACEABILITY_MATRIX.md` → `specifications/`
   - `TASK_TRACKER.md` → `specifications/`
   - `ANALYSIS_SUMMARY.md` → `specifications/`
   - `API_SPEC.md`, `DATA_MODELS.md`, `BUSINESS_RULES.md`, `INTEGRATION_SPEC.md` → `specifications/` root
   - `CODE_TAGGING_COMPLETION_REPORT.md` → archive
   - `PHASE5-DEPLOYMENT-OPERATIONS-SUMMARY.md` → archive
7. Remove template files that won't be needed: `_TEMPLATE_FR.md`, `_TEMPLATE_NFR.md`, `_TEMPLATE_ARCH.md`, `_TEMPLATE_TC.md` (keep only `_SPEC_TEMPLATE.md`)
8. Delete old `specs/` directory

## Todo

- [ ] Move all 5 phase directories
- [ ] Move root meta files
- [ ] Archive completion reports
- [ ] Remove old specs/ directory
- [ ] Verify no broken internal links

## Success Criteria

- All 79 spec files present in `specifications/`
- No files lost during migration
- Internal links within specs still work
