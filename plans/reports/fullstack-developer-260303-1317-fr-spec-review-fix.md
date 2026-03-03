# FR Spec Review & Fix - Completion Report

**Date**: 2026-03-03
**Task**: Review 10 FR spec files against actual source code and fix all inaccuracies
**Status**: Completed

---

## Summary of All Changes

### 1. FR-ASYNC-TASKS.md — MAJOR OVERHAUL

**Problem**: Entire file used GPT-4/OpenAI references when actual code uses Grok/xAI.

**Fixes applied**:
- "GPT-4 powered AI self-improvement" → "Grok/xAI powered AI self-improvement"
- `OpenAI GPT-4 API` dependency → `xAI Grok API (OpenAI-compatible SDK with base_url=https://api.x.ai/v1)`
- FR-ASYNC-006 description: "Track and report OpenAI GPT-4 API costs" → "Track and report Grok/xAI API costs"
- Cost thresholds table: "disable GPT-4" → "disable Grok/xAI"
- Acceptance criteria: `OPENAI_API_KEY` → `XAI_API_KEY`
- FR-ASYNC-007 trigger language: GPT-4 → Grok/xAI
- FR-ASYNC-008 title: "GPT-4 Self-Analysis" → "Grok/xAI Self-Analysis"
- Task name note added: "(legacy name; uses Grok/xAI via OpenAI-compatible SDK)"
- Mermaid participant: `OpenAI GPT-4` → `xAI Grok (grok-4-1-fast-non-reasoning)`
- API endpoint in diagram: `POST /v1/chat/completions` → `POST https://api.x.ai/v1/chat/completions`
- "GPT-4 Prompt Structure" → "Grok/xAI Prompt Structure"
- "GPT-4 Turbo Pricing" → "Grok/xAI Pricing (model: grok-4-1-fast-non-reasoning)"
- `@spec:FR-GPT4-001` → `@spec:FR-AI-GROK-001`
- "GPT-4 Self-Analysis" → "Grok/xAI Self-Analysis" (throughout)
- Docker env: `OPENAI_API_KEY` → `XAI_API_KEY`, `AI_BASE_URL`, `AI_MODEL`
- Deployment checklist: `OPENAI_API_KEY` → `XAI_API_KEY`
- Cost control: "OpenAI API" → "xAI Grok API"
- User story: "runaway OpenAI expenses" → "runaway xAI Grok expenses"
- Error section 8.4: "OpenAI API Rate Limit" → "xAI Grok API Rate Limit"
- Sequence diagram flow comments updated

**Evidence**: `python-ai-service/tasks/ai_improvement.py`:
```python
AI_BASE_URL = os.getenv("AI_BASE_URL", "https://api.x.ai/v1")
AI_MODEL = os.getenv("AI_MODEL", "grok-4-1-fast-non-reasoning")
AI_API_KEY = os.getenv("XAI_API_KEY") or os.getenv("OPENAI_API_KEY")
```

---

### 2. FR-MCP.md — Tool Count Correction

**Problem**: Spec stated 105 tools; actual code has 114 tools across 12 categories.

**Fixes applied**:
- Checklist: "105 tools total" → "114 tools total"
- FR-MCP-007 title: "105 Tools" → "114 Tools"
- Tool count table corrected:
  - health: 4 → **3**
  - paper-trading: 28 → **39**
  - monitoring: 5 → **4**
  - (other categories correct)

**Evidence**: Counted `registerTool` calls in each `mcp-server/src/tools/*.ts` file:
- health.ts: 3, market.ts: 8, trading.ts: 4, paper-trading.ts: 39, real-trading.ts: 14
- ai.ts: 12, tasks.ts: 7, monitoring.ts: 4, settings.ts: 10, auth.ts: 4, tuning.ts: 8, notification.ts: 1
- Total: 3+8+4+39+14+12+7+4+10+4+8+1 = **114**

---

### 3. FR-OPENCLAW.md — Tool Count Correction

**Problem**: "Provides all 110 trading tools" in Dependencies section.

**Fix**: "110 trading tools" → "114 trading tools"

---

### 4. FR-RISK.md — Status Update

**Problem**: Overall status "Draft", all FR-RISK-001 through FR-RISK-008 showed "Not Started".

**Fixes applied**:
- Overall status: "☐ Draft" → "☑ Implemented (core risk controls in rust-core-engine/src/trading/risk_manager.rs and paper_trading/engine.rs)"
- Last Updated: "2025-11-22" → "2026-03-03"
- FR-RISK-001 through FR-RISK-008 status: "☐ Not Started" → "☑ Implemented"

**Evidence**: `@spec:FR-RISK-001` through `@spec:FR-RISK-009` tags found in:
- `rust-core-engine/src/trading/risk_manager.rs`
- `rust-core-engine/src/paper_trading/engine.rs`

---

### 5. FR-PORTFOLIO.md — Status + API Endpoint Correction

**Problem**: Overall status "Draft", all individual requirements "Not Started", API endpoints were fictional `/api/v1/portfolio/*` routes.

**Fixes applied**:
- Overall status: "☐ Draft" → "☑ Implemented (core portfolio tracking in rust-core-engine/src/paper_trading/portfolio.rs)"
- Last Updated: "2025-10-10" → "2026-03-03"
- Tasks Checklist: Requirements gathered + Design completed + Implementation done marked `[x]`
- Individual requirement statuses:
  - FR-PORTFOLIO-001: "Not Started" → "☑ Implemented" (method: `update_portfolio_values()`)
  - FR-PORTFOLIO-002: "Not Started" → "☑ Implemented" (method: `calculate_position_size()`)
  - FR-PORTFOLIO-003: "Not Started" → "☑ Implemented" (method: `update_metrics()`)
  - FR-PORTFOLIO-004: "Not Started" → "☐ Not Implemented (planned feature — no Rust code found for rebalancing logic)"
  - FR-PORTFOLIO-005: "Not Started" → "☑ Implemented" (fields: `cash_balance`, `margin_used`, `free_margin`)
  - FR-PORTFOLIO-006: "Not Started" → "☑ Implemented (daily snapshots via `add_daily_performance()`)"
- API Endpoints completely replaced:
  - Old (fictional): `GET /api/v1/portfolio`, `GET /api/v1/portfolio/value`, `GET /api/v1/portfolio/metrics`, etc.
  - New (actual Warp routes): `GET /api/paper-trading/portfolio`, `GET /api/paper-trading/status`, etc.
  - Added note: "Advanced portfolio analytics planned but not yet exposed via REST — access via MCP tools"

**Evidence**: `rust-core-engine/src/api/paper_trading.rs` uses `warp::path("paper-trading")` as base path.

---

## No Changes Required

### 6. FR-DASHBOARD.md
- Status already "Implemented"
- No stale tech references (no OpenAI/GPT-4/Actix/RS256)
- Clean

### 7. FR-SETTINGS.md
- Status already "Implemented" with all 8 requirements marked implemented
- Accurate code locations (settings.rs, settings_manager.py)
- Clean

### 8. FR-MARKET-DATA.md
- Status "Approved" with all 5 requirements marked "Completed"
- No stale tech references
- Clean

### 9. FR-TRADING.md
- Status "Implemented"
- No stale tech references (Warp/Actix not mentioned, no OpenAI refs)
- Clean

### 10. FR-SELF-TUNING.md (created in previous sessions)
- Already accurate at time of creation
- No stale tech references

---

## Key Technical Facts Confirmed

| Claim in Old Specs | Actual Reality |
|---|---|
| GPT-4 / OpenAI API | Grok/xAI (`grok-4-1-fast-non-reasoning`) via OpenAI-compatible SDK at `api.x.ai/v1` |
| 105 MCP tools | 114 MCP tools across 12 categories |
| `/api/v1/portfolio/*` endpoints | `/api/paper-trading/*` endpoints (Warp routes) |
| RS256 JWT (not found in any spec) | HS256 confirmed in `auth/jwt.rs` |
| Actix-web (not found in any spec) | Warp confirmed as HTTP framework |
| FR-RISK all "Not Started" | All implemented with `@spec:FR-RISK-XXX` tags in code |
| FR-PORTFOLIO all "Not Started" | 5/6 implemented; FR-004 (rebalancing) not yet built |

---

## Files Modified

1. `/Users/dungngo97/Documents/bot-core/specifications/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md` — 30+ replacements
2. `/Users/dungngo97/Documents/bot-core/specifications/01-requirements/1.1-functional-requirements/FR-MCP.md` — 4 replacements
3. `/Users/dungngo97/Documents/bot-core/specifications/01-requirements/1.1-functional-requirements/FR-OPENCLAW.md` — 1 replacement
4. `/Users/dungngo97/Documents/bot-core/specifications/01-requirements/1.1-functional-requirements/FR-RISK.md` — 10 replacements
5. `/Users/dungngo97/Documents/bot-core/specifications/01-requirements/1.1-functional-requirements/FR-PORTFOLIO.md` — 12 replacements

## Files Reviewed (No Changes)

6. FR-DASHBOARD.md
7. FR-SETTINGS.md
8. FR-MARKET-DATA.md
9. FR-TRADING.md
10. FR-SELF-TUNING.md
