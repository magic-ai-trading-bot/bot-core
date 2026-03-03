# Phase 5 Remaining: Review & Fix Remaining ~40% Specs

**Status**: ✅ Complete (2026-03-03)
**Context**: Phase 5 reviewed ~60% of specs. These files still need accuracy review against actual code.

## Files Still Needing Review

### FR Specs (10 files)
- `specifications/01-requirements/1.1-functional-requirements/FR-DASHBOARD.md`
- `specifications/01-requirements/1.1-functional-requirements/FR-PORTFOLIO.md`
- `specifications/01-requirements/1.1-functional-requirements/FR-SETTINGS.md`
- `specifications/01-requirements/1.1-functional-requirements/FR-MARKET-DATA.md`
- `specifications/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md`
- `specifications/01-requirements/1.1-functional-requirements/FR-TRADING.md`
- `specifications/01-requirements/1.1-functional-requirements/FR-RISK.md`
- `specifications/01-requirements/1.1-functional-requirements/FR-MCP.md` (newly created, verify)
- `specifications/01-requirements/1.1-functional-requirements/FR-OPENCLAW.md` (newly created, verify)
- `specifications/01-requirements/1.1-functional-requirements/FR-SELF-TUNING.md` (newly created, verify)

### NFR Specs (5 files)
- `specifications/01-requirements/1.2-non-functional-requirements/NFR-PERFORMANCE.md`
- `specifications/01-requirements/1.2-non-functional-requirements/NFR-SECURITY.md`
- `specifications/01-requirements/1.2-non-functional-requirements/NFR-SCALABILITY.md`
- `specifications/01-requirements/1.2-non-functional-requirements/NFR-RELIABILITY.md`
- `specifications/01-requirements/1.2-non-functional-requirements/NFR-MAINTAINABILITY.md`

### User Stories & System Requirements (6 files)
- `specifications/01-requirements/1.3-user-stories/US-TRADER.md`
- `specifications/01-requirements/1.3-user-stories/US-ADMIN.md`
- `specifications/01-requirements/1.3-user-stories/US-SYSTEM.md`
- `specifications/01-requirements/1.4-system-requirements/SYS-HARDWARE.md`
- `specifications/01-requirements/1.4-system-requirements/SYS-SOFTWARE.md`
- `specifications/01-requirements/1.4-system-requirements/SYS-NETWORK.md`

### Design Specs (7 files - not yet reviewed)
- `specifications/02-design/2.1-architecture/ARCH-MICROSERVICES.md`
- `specifications/02-design/2.1-architecture/ARCH-DATA-FLOW.md`
- `specifications/02-design/2.1-architecture/ARCH-SECURITY.md`
- `specifications/02-design/2.4-ui-ux/UI-COMPONENTS.md`
- `specifications/02-design/2.4-ui-ux/UX-FLOWS.md`
- `specifications/02-design/2.4-ui-ux/UI-WIREFRAMES.md`
- `specifications/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md`
- `specifications/02-design/2.5-components/COMP-RUST-AUTH.md`
- `specifications/02-design/2.5-components/COMP-PYTHON-ML.md`

### Testing Specs (9 files)
- `specifications/03-testing/3.1-test-plan/TEST-PLAN.md`
- `specifications/03-testing/3.2-test-cases/TC-ASYNC.md`
- `specifications/03-testing/3.2-test-cases/TC-TRADING.md`
- `specifications/03-testing/3.2-test-cases/TC-AUTH.md`
- `specifications/03-testing/3.2-test-cases/TC-AI.md`
- `specifications/03-testing/3.2-test-cases/TC-INTEGRATION.md`
- `specifications/03-testing/3.3-test-scenarios/TS-HAPPY-PATH.md`
- `specifications/03-testing/3.3-test-scenarios/TS-EDGE-CASES.md`
- `specifications/03-testing/3.3-test-scenarios/TS-ERROR-HANDLING.md`

### Deployment Specs (7 files)
- `specifications/04-deployment/4.1-infrastructure/INFRA-REQUIREMENTS.md`
- `specifications/04-deployment/4.1-infrastructure/INFRA-DOCKER.md`
- `specifications/04-deployment/4.1-infrastructure/INFRA-KUBERNETES.md`
- `specifications/04-deployment/4.2-cicd/CICD-PIPELINE.md`
- `specifications/04-deployment/4.2-cicd/CICD-WORKFLOWS.md`
- `specifications/04-deployment/4.3-monitoring/MON-LOGGING.md`
- `specifications/04-deployment/4.3-monitoring/MON-METRICS.md`

### Operations Specs (3 files)
- `specifications/05-operations/5.1-operations-manual/OPS-MANUAL.md`
- `specifications/05-operations/5.2-troubleshooting/TROUBLESHOOTING.md`
- `specifications/05-operations/5.3-disaster-recovery/DR-PLAN.md`

## Known Issues from Previous Reviews (apply same fixes)
- GPT-4/OpenAI → Grok/xAI everywhere
- Actix-web → Warp everywhere
- RS256 → HS256 everywhere
- Line numbers likely stale in many specs
- DB collection names may be wrong (use findings from DB-SCHEMA fix)

## Review Process
For each file: read spec → grep actual code → compare → fix inaccuracies
