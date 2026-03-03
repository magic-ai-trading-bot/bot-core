# Design Specification Accuracy Review

**Date:** 2026-03-03
**Reviewer:** code-reviewer
**Scope:** 5 specification files in `specifications/02-design/`
**Method:** Cross-referenced each spec against actual source code, docker-compose files, and dependency manifests

---

## 1. ARCH-OVERVIEW.md

**File:** `specifications/02-design/2.1-architecture/ARCH-OVERVIEW.md`

### Status: NEEDS UPDATE

### Issues Found

**CRITICAL - Framework Mismatch (ARCH-OVERVIEW-001 & ARCH-OVERVIEW-002):**
- Spec describes Rust as using **Actix-web 4.0** as the web framework (line 92, line 264)
- Actual code uses **Warp 0.4** (see `Cargo.toml` line 24: `warp = { version = "0.4" }`)
- `actix-web` only appears in `[dev-dependencies]` for testing, not as the production framework
- The API-RUST-CORE spec correctly identifies Warp, but ARCH-OVERVIEW still says Actix-web. This inconsistency is confusing.

**CRITICAL - AI Model Provider Mismatch:**
- Spec states the Python AI service uses **OpenAI GPT-4** / **GPT-4o-mini** throughout (lines 99, 152, 282, 543)
- Actual code uses **xAI Grok** (`grok-4-1-fast-non-reasoning`) as the primary AI provider
  - `AI_BASE_URL = "https://api.x.ai/v1"` (main.py line 57)
  - `AI_MODEL = "grok-4-1-fast-non-reasoning"` (main.py line 58)
  - docker-compose-vps.yml sets `AI_MODEL=grok-4-1-fast-non-reasoning`
- OpenAI is only a fallback key source, not the primary provider

**HIGH - Missing Services in Architecture Diagram:**
- The architecture diagram (ARCH-OVERVIEW-001, lines 84-130) shows only 3 services: Frontend, Rust Core, Python AI
- Actual deployment has **7 services** (docker-compose-vps.yml):
  1. MongoDB (port 27017)
  2. Python AI Service (port 8000)
  3. Rust Core Engine (port 8080)
  4. Next.js UI Dashboard (port 3000)
  5. Redis Cache
  6. MCP Server (port 8090)
  7. OpenClaw Gateway (port 18789)
- MCP Server and OpenClaw are significant components not represented in the architecture overview

**HIGH - Dependency Versions Outdated (ARCH-OVERVIEW-002):**
- Spec: `reqwest 0.11` -> Actual: `reqwest 0.12`
- Spec: `MongoDB Driver 3.3` -> Actual: `mongodb 3.5`
- Spec: `JWT 9.1` -> Actual: `jsonwebtoken 10.3`
- Spec: `BCrypt 0.15` -> Actual: `bcrypt 0.17`
- Spec: `Rust Decimal 1.33` -> Actual: `rust_decimal 1.40`
- Spec: `tokio 1.0` -> Actual: `tokio 1.49`
- Spec: `uuid 1.11` -> Actual: `uuid 1.21`
- Spec: `Warp 0.3` -> Actual: `warp 0.4` (also listed as Actix in some places)

**MEDIUM - Docker Compose File Location:**
- Spec references `docker-compose.yml:1-460` as a single file
- Actual setup: `infrastructure/docker/docker-compose.yml` (main), `docker-compose-vps.yml` (VPS deployment), `infrastructure/docker/docker-compose.prod.yml` (production overrides)
- Root `docker-compose.yml` is a symlink to `infrastructure/docker/docker-compose.yml`

**MEDIUM - Database Description:**
- Spec says "MongoDB Atlas" (managed cloud) throughout
- Actual VPS deployment uses **local MongoDB 7.0 container** (docker-compose-vps.yml line 10)
- Atlas is only relevant for cloud deployments, not the primary deployment method

**MEDIUM - Redis Not Optional:**
- Spec marks Redis as "Optional" (line 107)
- In docker-compose-vps.yml, Redis is a required service (lines 160-180), not optional
- Python AI Service depends on Redis for caching (`REDIS_HOST=redis-cache`)

### Fixes Needed
1. Replace all "Actix-web" references with "Warp" for the production framework
2. Update AI provider from "OpenAI GPT-4" to "xAI Grok" (with OpenAI as fallback)
3. Add MCP Server and OpenClaw Gateway to architecture diagram
4. Update all dependency versions to match Cargo.toml
5. Clarify MongoDB deployment (local container vs Atlas)
6. Update Redis status from "Optional" to "Required" for VPS deployment

---

## 2. DB-SCHEMA.md

**File:** `specifications/02-design/2.2-database/DB-SCHEMA.md`

### Status: NEEDS UPDATE

### Issues Found

**HIGH - Collection Name Mismatches:**
The spec documents 22 collections, but several names do not match actual code:

| Spec Collection | Actual Collection | Status |
|---|---|---|
| `trades` | `trade_records` | WRONG name |
| `positions` | (no dedicated collection; in-memory DashMap) | WRONG - does not exist as collection |
| `paper_trading_accounts` | (no match; portfolio data is in `portfolio_history`) | WRONG name |
| `paper_trading_trades` | `paper_trades` | WRONG name |
| `portfolio_snapshots` | `portfolio_history` | WRONG name |
| `risk_metrics` | (no collection found in code) | DOES NOT EXIST |
| `strategy_configs` | (no collection found in code) | DOES NOT EXIST |
| `audit_logs` | (no collection found in code) | DOES NOT EXIST |
| `notifications` | `notification_preferences` + `push_subscriptions` | WRONG - split into 2 |
| `system_config` | (no collection found in code) | DOES NOT EXIST |
| `celery_task_meta` | (no collection found in code) | DOES NOT EXIST |
| `training_jobs` | (no collection found in code) | DOES NOT EXIST |
| `backtest_results` | (no collection found in code) | DOES NOT EXIST |
| `monitoring_alerts` | (no collection found in code) | DOES NOT EXIST |
| `task_schedules` | (no collection found in code) | DOES NOT EXIST |

**HIGH - Missing Collections in Spec:**
These collections exist in actual code but are NOT documented in the spec:

| Actual Collection | Used In |
|---|---|
| `trade_records` | `rust-core-engine/src/storage/mod.rs` |
| `analysis_results` | `rust-core-engine/src/storage/mod.rs` |
| `price_history` | `rust-core-engine/src/storage/mod.rs` |
| `signals_history` | `rust-core-engine/src/storage/mod.rs` |
| `user_symbols` | `rust-core-engine/src/storage/mod.rs` |
| `push_subscriptions` | `rust-core-engine/src/storage/mod.rs` |
| `trade_analyses` | `rust-core-engine/src/storage/mod.rs` + `python-ai-service/utils/data_storage.py` |
| `config_suggestions` | `rust-core-engine/src/storage/mod.rs` + `python-ai-service/utils/data_storage.py` |
| `gpt4_analysis_history` | `python-ai-service/utils/data_storage.py` |
| `model_accuracy_history` | `python-ai-service/utils/data_storage.py` |
| `api_cost_history` | `python-ai-service/utils/data_storage.py` |
| `retrain_history` | `python-ai-service/utils/data_storage.py` |

**HIGH - Async Task Collections Are Aspirational:**
- The spec documents 5 "NEW" async task collections (celery_task_meta, training_jobs, backtest_results, monitoring_alerts, task_schedules)
- None of these exist in actual code. The Python service does NOT use Celery/RabbitMQ -- these appear to be planned but unimplemented features
- The spec claims "Total Collections: 22" but actual count from code is ~18 distinct collections across both services

**MEDIUM - Duplicate AI Analysis Collections:**
- Rust uses `analysis_results` and `ai_signals` collections
- Python uses `ai_analysis_results` (main.py line 86) AND `gpt4_analysis_history` (data_storage.py)
- Spec documents `ai_analysis_results` but not `analysis_results` or `gpt4_analysis_history`
- There appears to be data fragmentation across multiple collections for the same concept

### Fixes Needed
1. Rename all spec collections to match actual code collection names
2. Remove the 5 async task collections that do not exist (or mark as "Planned")
3. Add the 12 missing collections that exist in code
4. Correct the total collection count
5. Document the actual Python collections from `data_storage.py`
6. Remove or mark `positions`, `risk_metrics`, `strategy_configs`, `audit_logs`, `system_config` as "Not Implemented"

---

## 3. API-RUST-CORE.md

**File:** `specifications/02-design/2.3-api/API-RUST-CORE.md`

### Status: NEEDS UPDATE (Minor)

### Issues Found

**MEDIUM - Endpoint Count:**
- Spec says "Total Endpoints: 37"
- Actual route count from code analysis:
  - Auth: 4 (register, login, verify, profile)
  - Auth Security: 3 (change-password, profile/update, sessions)
  - Market: 7 (prices, overview, candles, chart, charts, symbols GET, symbols POST, symbols DELETE)
  - Trading: 4 (positions, account, close position, performance)
  - Paper Trading: ~30 endpoints (status, portfolio, trades/open, trades/closed, close trade, settings, strategy-settings GET/PUT, basic-settings GET/PUT, execution-settings GET/PUT, ai-settings GET/PUT, notification-settings GET/PUT, symbols GET/PUT, reset, start, stop, orders, pending-orders GET/DELETE, trigger-analysis, signal-interval, indicator-settings GET/PUT, trade-analyses, config-suggestions, atr-diagnostics, signals-history, latest-signals)
  - Real Trading: ~12 (status, portfolio, trades/open, trades/closed, start, stop, close trade, settings GET/PUT, orders GET/POST/DELETE, orders/all, positions/sltp)
  - AI: 6 (analyze, strategy-recommendations, market-condition, feedback, info, strategies, market-bias)
  - Monitoring: 3 (system, trading, connection)
  - Settings: 4 (api-keys GET/POST/DELETE, test)
  - Notifications: 4 (preferences GET/PUT, push/subscribe POST/DELETE, test, vapid-key)
  - Health: 1
  - WebSocket: 1
- Actual total is **~75+ endpoints**, far more than the documented 37

**MEDIUM - Missing Endpoint Categories:**
- Real Trading endpoints (entire `/api/real-trading/*` namespace) are not documented
- Settings endpoints (`/api/settings/api-keys/*`) are not documented
- Notifications endpoints (`/api/notifications/*`) are not documented
- Auth security endpoints (`/api/auth/change-password`, `/api/auth/sessions`) are not documented

**LOW - Code Location References:**
- Many code location references (e.g., `rust-core-engine/src/api/mod.rs:187-201`) may have line number drift as the codebase has grown significantly
- The API mod.rs file is very large; line numbers in spec are unlikely to match current state

**POSITIVE - Framework Correctly Identified:**
- Unlike ARCH-OVERVIEW, this spec correctly identifies Warp as the framework (line 32)

### Fixes Needed
1. Update endpoint count from 37 to actual (~75+)
2. Add Real Trading, Settings, Notifications, and Auth Security endpoint documentation
3. Update or remove specific line number references
4. Document paper trading advanced endpoints (indicator-settings, trade-analyses, config-suggestions, etc.)

---

## 4. API-PYTHON-AI.md

**File:** `specifications/02-design/2.3-api/API-PYTHON-AI.md`

### Status: NEEDS UPDATE

### Issues Found

**CRITICAL - AI Provider Mismatch:**
- Spec title says "GPT-4 Trading AI" and references OpenAI GPT-4o-mini throughout
- Actual code uses **xAI Grok** as primary provider:
  - Service title in code: `"Grok AI Cryptocurrency Trading Service"` (main.py line 694)
  - Model: `grok-4-1-fast-non-reasoning` (main.py line 58)
  - Client class: `GrokClient` (main.py line 1358)
- All "GPT-4" references should be updated to "Grok/xAI"

**HIGH - Async Task Endpoints Do Not Exist:**
- Spec documents entire "Async Task Management" section with endpoints:
  - `POST /api/tasks/train`
  - `GET /api/tasks/{task_id}`
  - `DELETE /api/tasks/{task_id}`
  - `GET /api/tasks`
- Also documents Training Management, Backtest Management, Monitoring & Alerts sections
- **None of these endpoints exist in actual code** (grep for `/api/tasks`, `/api/training`, `/api/backtest`, `/api/monitoring` returns no matches)
- The spec references `python-ai-service/api/tasks.py` which does not exist as a router
- Python tasks exist in `python-ai-service/tasks/` directory but are not exposed as API endpoints

**HIGH - Missing Actual Endpoints:**
The following endpoints exist in code but are NOT documented in the spec:
- `GET /ai/health/market-condition` (main.py line 2773)
- `GET /api/ai/signal-quality` (main.py line 2851)
- `POST /predict-trend` (main.py line 3677)
- `GET /ai/cost/statistics` (main.py line 4049)
- `POST /ai/config-analysis/trigger` (main.py line 4185)
- `GET /ai/config-suggestions` (main.py line 4241)
- `GET /ai/gpt4-analysis-history` (main.py line 4282)
- `POST /ai/analyze-trade` (main.py line 4543)
- `POST /api/chat/project` (main.py line 4576)
- `GET /api/chat/project/suggestions` (main.py line 4630)
- `POST /api/chat/project/clear` (main.py line 4644)

**MEDIUM - RabbitMQ/Celery Architecture Not Implemented:**
- Spec states "Task Queue: RabbitMQ + Celery (async task execution)" and "Result Backend: Redis (task result storage)"
- No RabbitMQ service in docker-compose files
- No Celery worker service in docker-compose files
- The task files in `python-ai-service/tasks/` exist but are not integrated as Celery async tasks with API endpoints

**MEDIUM - Service Description:**
- Spec describes the service as "GPT-4 Trading AI"
- Actual FastAPI title: `"Grok AI Cryptocurrency Trading Service"` (main.py line 694)

### Fixes Needed
1. Replace all "GPT-4" / "OpenAI" references with "xAI Grok" (keeping OpenAI as documented fallback)
2. Remove or mark as "Planned/Not Implemented" the entire Async Task Management section
3. Remove or mark Training Management, Backtest Management, Monitoring & Alerts as "Planned"
4. Add documentation for 11 undocumented endpoints
5. Remove RabbitMQ/Celery references from architecture description
6. Update service name and description

---

## 5. COMP-RUST-TRADING.md

**File:** `specifications/02-design/2.5-components/COMP-RUST-TRADING.md`

### Status: NEEDS UPDATE (Minor-Medium)

### Issues Found

**HIGH - Missing `real_trading/` Module:**
- Spec documents `trading/` and `paper_trading/` modules extensively
- Does NOT document the `real_trading/` module which is a significant new component:
  ```
  rust-core-engine/src/real_trading/
  |- config.rs    (26KB)
  |- engine.rs    (339KB - largest file in codebase)
  |- mod.rs       (2KB)
  |- order.rs     (11KB)
  |- position.rs  (26KB)
  |- risk.rs      (37KB)
  ```
- This is a major component with its own engine, position management, risk management, and order handling

**MEDIUM - Missing `strategy_optimizer.rs` in Paper Trading:**
- Spec lists paper_trading modules as: engine.rs, portfolio.rs, trade.rs, settings.rs
- Actual directory also includes `strategy_optimizer.rs` (82KB) which is not documented

**MEDIUM - Missing Strategy Modules:**
- Spec section 2.1 lists `strategies/` as "Related Modules" but does not detail the strategy modules
- Actual strategies directory has grown significantly:
  - `stochastic_strategy.rs` (25KB) - not documented
  - `ml_trend_predictor.rs` (27KB) - not documented
  - `hybrid_filter.rs` (28KB) - not documented
  - `trend_filter.rs` (13KB) - not documented
  - `tests.rs` (5KB)

**MEDIUM - File Size Estimates Outdated:**
- Spec: `engine.rs` (trading) is "600+ lines" -> Actual: 162KB file
- Spec: `position_manager.rs` is "400+ lines" -> Actual: 15KB
- Spec: `risk_manager.rs` is "350+ lines" -> Actual: 34KB
- Spec: `paper_trading/engine.rs` is "1500+ lines" -> Actual: 486KB (massive growth)
- Spec: `paper_trading/trade.rs` is "2214 lines" -> Actual: 64KB
- Spec says "Total Lines of Code: ~6,000+ lines" -> Actual is easily 50,000+ lines

**LOW - Binance Module Missing `user_data_stream.rs`:**
- Spec lists binance module as: client.rs, types.rs, websocket.rs
- Actual also includes `user_data_stream.rs` (78KB) and `mod.rs` (23KB)

**POSITIVE - Core Architectural Descriptions Accurate:**
- TradingEngine struct description matches actual code
- PositionManager using DashMap is accurate
- RiskManager pattern is correctly described
- PaperTradingEngine component relationships are correct

### Fixes Needed
1. Add entire `real_trading/` module documentation (config, engine, order, position, risk)
2. Add `strategy_optimizer.rs` to paper_trading module list
3. Add missing strategy modules (stochastic, ml_trend_predictor, hybrid_filter, trend_filter)
4. Update file size estimates to reflect current codebase
5. Add `user_data_stream.rs` to binance module list
6. Update total LOC estimate

---

## Summary Table

| Spec File | Status | Critical | High | Medium | Low |
|---|---|---|---|---|---|
| ARCH-OVERVIEW.md | NEEDS UPDATE | 2 | 2 | 3 | 0 |
| DB-SCHEMA.md | NEEDS UPDATE | 0 | 3 | 1 | 0 |
| API-RUST-CORE.md | NEEDS UPDATE | 0 | 0 | 3 | 1 |
| API-PYTHON-AI.md | NEEDS UPDATE | 1 | 2 | 2 | 0 |
| COMP-RUST-TRADING.md | NEEDS UPDATE | 0 | 1 | 3 | 1 |

### Priority Actions

1. **[CRITICAL]** Fix AI provider references everywhere: GPT-4/OpenAI -> xAI Grok (ARCH-OVERVIEW + API-PYTHON-AI)
2. **[CRITICAL]** Fix framework reference: Actix-web -> Warp in ARCH-OVERVIEW
3. **[HIGH]** Remove or mark as "Planned" the 5 nonexistent async task DB collections and their API endpoints
4. **[HIGH]** Fix 15+ collection name mismatches in DB-SCHEMA
5. **[HIGH]** Add MCP Server and OpenClaw to architecture diagram
6. **[HIGH]** Document `real_trading/` module in COMP-RUST-TRADING
7. **[MEDIUM]** Update all dependency versions in ARCH-OVERVIEW
8. **[MEDIUM]** Document ~40 undocumented API endpoints across Rust and Python specs
9. **[MEDIUM]** Update file size estimates and module lists in COMP-RUST-TRADING

### Unresolved Questions

1. Are the Celery/RabbitMQ async task features intentionally removed or postponed? If postponed, specs should mark them as "Planned - Not Implemented".
2. Several Python collections (gpt4_analysis_history, model_accuracy_history, api_cost_history, retrain_history) appear to overlap conceptually with the Rust-side collections. Is this intentional data separation or accidental duplication?
3. The `positions` collection documented in DB-SCHEMA does not exist -- positions are tracked in memory (DashMap). Should it be created for persistence, or should the spec remove it?
4. The spec references `config.env` template but the actual template file is `.env.example`. Which is correct?
