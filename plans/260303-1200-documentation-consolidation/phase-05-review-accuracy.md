# Phase 5: Review & Fix All Content for Accuracy

**Priority**: Critical | **Status**: Pending | **Effort**: Large

## Overview

This is the most important phase. Every specification document must be validated against the actual codebase. Outdated content gets updated, incorrect specs get rewritten.

## Review Strategy

For each spec file, verify:
1. **Code locations** — Do referenced files/line numbers still exist?
2. **API endpoints** — Do URLs, methods, request/response match actual code?
3. **Data models** — Do schema fields, types, validations match MongoDB collections?
4. **Configuration** — Do env vars, ports, defaults match actual .env and configs?
5. **Features** — Do described behaviors match actual implementation?
6. **Metrics** — Do test counts, coverage %, quality scores match reality?

## Review Checklist by Section

### 01-requirements/ (26 files)

| File | Key Validations |
|------|----------------|
| FR-AI.md | Verify model types (LSTM, GRU, Transformer, Ensemble), API endpoints, accuracy metrics |
| FR-AUTH.md | Verify RS256 JWT, bcrypt, token refresh flow, middleware |
| FR-PAPER-TRADING.md | Verify engine.rs line numbers, risk params (5% daily loss, 60min cooldown) |
| FR-REAL-TRADING.md | Verify Binance API integration, testnet config, order types |
| FR-RISK.md | Verify risk limits, correlation checks, consecutive loss tracking |
| FR-STRATEGIES.md | Verify 5 strategies (RSI, MACD, Bollinger, Volume, Combined), indicators |
| FR-WEBSOCKET.md | Verify event types, connection handling, reconnection logic |
| FR-DASHBOARD.md | Verify component count (71), pages, hooks |
| FR-TRADING.md | Verify trade execution flow, slippage model, partial fills |
| FR-MARKET-DATA.md | Verify Binance WebSocket feeds, data processing |
| FR-PORTFOLIO.md | Verify portfolio tracking, P&L calculation |
| FR-SETTINGS.md | Verify settings API, configuration options |
| FR-ASYNC-TASKS.md | Verify async task system, job types |
| NFR-*.md (5 files) | Verify performance targets, security measures |
| US-*.md (3 files) | Cross-check user stories against implemented features |
| SYS-*.md (3 files) | Verify hardware/software/network requirements |

### 02-design/ (18 files)

| File | Key Validations |
|------|----------------|
| ARCH-OVERVIEW.md | Verify service architecture matches docker-compose |
| ARCH-MICROSERVICES.md | Verify service boundaries, communication patterns |
| ARCH-DATA-FLOW.md | Verify data flow matches actual request/response patterns |
| ARCH-SECURITY.md | Verify security implementation (JWT, CORS, rate limiting) |
| DB-SCHEMA.md | Verify all 17 collections, field names, types against Rust structs |
| DB-INDEXES.md | Verify 37 indexes match actual MongoDB indexes |
| API-RUST-CORE.md | Verify all endpoints match Actix-web routes |
| API-PYTHON-AI.md | Verify all endpoints match FastAPI routes |
| API-WEBSOCKET.md | Verify WebSocket message types and handlers |
| UI-COMPONENTS.md | Verify component list matches actual React components |
| COMP-*.md (4 files) | Verify component architecture against code |

### 03-testing/ (9 files)

| File | Key Validations |
|------|----------------|
| TEST-PLAN.md | Verify test count (2,202+), coverage targets |
| TC-*.md (5 files) | Verify test case IDs reference actual test functions |

### 04-deployment/ (7 files)

| File | Key Validations |
|------|----------------|
| INFRA-DOCKER.md | Verify against docker-compose.yml, Dockerfiles |
| CICD-*.md (2 files) | Verify against .github/workflows/ |
| MON-*.md (2 files) | Verify monitoring config |

### 05-operations/ (3+ files)

| File | Key Validations |
|------|----------------|
| OPS-MANUAL.md | Verify commands, scripts match scripts/bot.sh |
| TROUBLESHOOTING.md | Verify error messages, solutions still apply |
| DR-PLAN.md | Verify backup/restore procedures |

### 06-features/ (8 files)

| File | Key Validations |
|------|----------------|
| All feature docs | Verify code locations, line numbers, behavior descriptions |

## Process Per File

1. Read the spec document
2. Read the corresponding source code
3. Compare spec claims vs actual code
4. Mark findings: ✅ Accurate / ⚠️ Outdated / ❌ Wrong
5. Fix outdated content in-place
6. Rewrite incorrect sections

## Todo

- [ ] Review all FR-*.md files (13 files)
- [ ] Review all NFR-*.md files (5 files)
- [ ] Review user stories and system requirements (6 files)
- [ ] Review all architecture docs (4 files)
- [ ] Review database specs (3 files)
- [ ] Review API specs (3 files)
- [ ] Review UI/UX specs (3 files)
- [ ] Review component specs (4 files)
- [ ] Review testing specs (9 files)
- [ ] Review deployment specs (7 files)
- [ ] Review operations specs (3 files)
- [ ] Review feature docs (8 files)
- [ ] Fix all outdated line numbers
- [ ] Update all metrics/counts

## Success Criteria

- Every spec file validated against actual code
- No incorrect information remains
- All line number references updated
- All metrics (test count, coverage, etc.) match reality
