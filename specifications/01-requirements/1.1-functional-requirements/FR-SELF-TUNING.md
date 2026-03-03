# Self-Tuning Engine - Functional Requirements

**Spec ID**: FR-SELF-TUNING
**Version**: 1.0
**Status**: ☑ Implemented
**Owner**: System
**Last Updated**: 2026-03-03

---

## Tasks Checklist

- [x] 3-tier approval system (GREEN / YELLOW / RED)
- [x] Parameter bounds registry with hard limits
- [x] GREEN tier auto-adjustment with notification
- [x] YELLOW tier confirmation token workflow
- [x] RED tier explicit approval text workflow
- [x] Parameter snapshot before each change
- [x] Audit trail for all adjustments
- [x] Rollback to previous snapshot
- [x] Cooldown enforcement per parameter
- [x] Aggregated tuning dashboard
- [x] Adjustment history query with filters
- [x] MCP tool integration (8 tools)
- [ ] Persistent audit log across restarts
- [ ] Automated performance-triggered suggestions

---

## Metadata

**Related Specs**:
- Related: [FR-PAPER-TRADING.md](./FR-PAPER-TRADING.md)
- Related: [FR-SETTINGS.md](./FR-SETTINGS.md)
- Related: [FR-RISK.md](./FR-RISK.md)

**Dependencies**:
- MCP Server (Node 18, Express, SDK v1.26.0)
- Rust API endpoints: `/api/paper-trading/basic-settings`, `/api/paper-trading/indicator-settings`, `/api/paper-trading/signal-interval`
- In-memory audit log and snapshot store
- Cryptographic confirm token via `security.ts`

**Business Value**: High
**Technical Complexity**: Medium
**Priority**: ☑ High

---

## Overview

The Self-Tuning Engine allows the AI agent (via MCP tools) to adjust trading parameters autonomously within pre-defined safety bounds. Changes are gated by a 3-tier approval system: GREEN parameters are applied automatically, YELLOW parameters require a confirmation token, and RED parameters require the user to type an explicit approval string. All changes are snapshotted and logged, with rollback available at any time.

**Architecture**: MCP tools layer → parameter bounds validation → API write-through to Rust backend → in-memory audit log.

---

## Business Context

**Problem Statement**: Manual parameter tuning is reactive, slow, and inconsistent. The AI agent observes performance metrics continuously and needs a safe, auditable mechanism to adapt trading parameters in response to changing market conditions.

**Business Goals**:
- Enable AI-driven continuous optimization of trading parameters
- Prevent runaway parameter changes through hard bounds and cooldowns
- Maintain full audit trail of every change for accountability
- Allow rapid rollback when a change degrades performance

**Success Metrics**:
- Zero out-of-bounds parameter writes
- Rollback restores all parameters within 5 seconds
- All adjustments logged with before/after values and reasoning
- Cooldown enforcement 100% reliable

---

## Functional Requirements

### FR-SELF-TUNING-001: 3-Tier Approval System

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-001`

**Description**:
Every tunable parameter is classified into one of three tiers that determine the approval workflow. GREEN: applied immediately with user notification. YELLOW: returns a confirmation token; user must supply token to apply. RED: returns a warning; user must type exact approval text.

**Acceptance Criteria**:
- [x] GREEN adjustments applied immediately, return `applied: true` + notification string
- [x] YELLOW adjustments without token return `pending: true` + `confirm_token`
- [x] YELLOW adjustments with valid token are applied
- [x] RED adjustments without approval return `pending: true` + `required_approval` text
- [x] RED adjustments with matching approval text are applied
- [x] Wrong tier tool call returns error (e.g., calling green tool on YELLOW param)

---

### FR-SELF-TUNING-002: Parameter Bounds Registry

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-002`

**Description**:
A static registry defines hard bounds (min, max, step, type, cooldownMs, tier, apiEndpoint, apiField) for every tunable parameter. Adjustments outside these bounds are rejected; numeric values are rounded to the nearest step.

**Tunable Parameters by Tier**:

| Tier | Parameter Key | Name | Range | Default |
|------|--------------|------|-------|---------|
| GREEN | `rsi_oversold` | RSI Oversold Threshold | 20–40 | 30 |
| GREEN | `rsi_overbought` | RSI Overbought Threshold | 60–80 | 70 |
| GREEN | `signal_interval_minutes` | Signal Generation Interval | 3–30 min | 5 |
| GREEN | `confidence_threshold` | Signal Confidence Threshold | 0.50–0.90 | 0.65 |
| GREEN | `data_resolution` | Data Timeframe | enum(1m..1d) | 15m |
| GREEN | `stop_loss_percent` | Stop Loss % (PnL) | 1.0–20.0 | 10.0 |
| GREEN | `take_profit_percent` | Take Profit % (PnL) | 2.0–40.0 | 20.0 |
| GREEN | `min_required_indicators` | Min Indicators | 2–5 | 4 |
| GREEN | `min_required_timeframes` | Min Timeframes | 1–4 | 3 |
| GREEN | `atr_stop_multiplier` | ATR Stop Multiplier | 0.8–2.5 | 1.2 |
| GREEN | `atr_tp_multiplier` | ATR Take Profit Multiplier | 1.5–5.0 | 2.4 |
| GREEN | `sp_min_weighted_threshold` | Min Weighted Threshold | 30–70 | 60 |
| GREEN | `sp_rsi_bull_threshold` | RSI Bull Threshold | 50–65 | 55 |
| GREEN | `sp_rsi_bear_threshold` | RSI Bear Threshold | 35–50 | 45 |
| GREEN | `sp_volume_confirm_multiplier` | Volume Confirm Multiplier | 1.0–2.0 | 1.2 |
| GREEN | `sp_confidence_max` | Confidence Max Cap | 0.70–0.95 | 0.85 |
| GREEN | `sp_neutral_confidence` | Neutral Signal Confidence | 0.30–0.50 | 0.40 |
| GREEN | `sp_counter_trend_block_offset` | Counter-Trend Block Offset | 0.0–0.15 | 0.05 |
| GREEN | `sp_counter_trend_mode` | Counter-Trend Mode | enum(block,reduce) | block |
| GREEN | `funding_spike_threshold` | Funding Spike Threshold | 0.0001–0.001 | 0.0003 |
| GREEN | `atr_spike_multiplier` | ATR Spike Multiplier | 1.5–3.0 | 2.0 |
| GREEN | `consecutive_loss_reduction_pct` | Consecutive Loss Reduction % | 0.1–0.5 | 0.3 |
| YELLOW | `position_size_percent` | Position Size % | 1.0–10.0 | 5.0 |
| YELLOW | `max_positions` | Max Concurrent Positions | 1–8 | 4 |
| YELLOW | `leverage` | Leverage | 1–20 | 10 |
| YELLOW | `base_risk_pct` | Base Risk % per Trade | 0.5–5.0 | 2.0 |
| YELLOW | `kelly_fraction` | Kelly Fraction | 0.25–0.75 | 0.5 |
| YELLOW | `sp_bb_bull_threshold` | BB Bull Threshold | 0.1–0.4 | 0.3 |
| YELLOW | `sp_bb_bear_threshold` | BB Bear Threshold | 0.6–0.9 | 0.7 |
| YELLOW | `sp_stoch_overbought` | Stochastic Overbought | 70–90 | 80 |
| YELLOW | `sp_stoch_oversold` | Stochastic Oversold | 10–30 | 20 |
| YELLOW | `sp_weight_15m` | 15M Timeframe Weight | 0.0–3.0 | 0.5 |
| YELLOW | `sp_weight_30m` | 30M Timeframe Weight | 0.0–3.0 | 1.0 |
| YELLOW | `sp_weight_1h` | 1H Timeframe Weight | 0.0–3.0 | 2.0 |
| YELLOW | `sp_counter_trend_enabled` | Counter-Trend Protection | boolean | true |
| RED | `max_daily_loss_percent` | Max Daily Loss % | 3.0–15.0 | 10.0 |
| RED | `weekly_drawdown_limit_pct` | Weekly Drawdown Limit % | 3.0–15.0 | 7.0 |
| RED | `atr_stop_enabled` | ATR Stop Loss Enabled | boolean | false |
| RED | `kelly_enabled` | Half-Kelly Enabled | boolean | false |
| RED | `engine_running` | Paper Trading Engine On/Off | boolean | false |

**Acceptance Criteria**:
- [x] All parameters registered with min/max/step/tier/cooldown/apiEndpoint/apiField
- [x] Numeric values outside range are rejected with descriptive error
- [x] Values rounded to nearest step size before application
- [x] Enum values validated against allowed list
- [x] `get_parameter_bounds` tool returns all parameters grouped by tier with cooldown status

---

### FR-SELF-TUNING-003: Cooldown Enforcement

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-003`

**Description**:
Each parameter has a minimum cooldown interval between consecutive adjustments (1 hour or 6 hours). The engine rejects adjustments attempted before cooldown expires.

**Acceptance Criteria**:
- [x] Default cooldown: 6 hours for most parameters, 1 hour for fast-cycle params
- [x] Rejection returns remaining cooldown seconds
- [x] Cooldown tracked in-memory per parameter key
- [x] `get_parameter_bounds` shows `inCooldown` and `cooldownRemainingSeconds`

---

### FR-SELF-TUNING-004: Parameter Snapshot

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-004`

**Description**:
Before every adjustment, the system takes a snapshot of all current parameter values and current performance metrics. Snapshots are stored in-memory and used for rollback.

**Acceptance Criteria**:
- [x] Snapshot captures all current API parameter values + performance data (winRate, totalPnl, sharpeRatio, maxDrawdown)
- [x] Snapshot assigned a unique ID and ISO timestamp
- [x] Snapshot taken automatically before GREEN, YELLOW, and RED changes
- [x] Manual `take_parameter_snapshot` tool available
- [x] At least 2 most recent snapshots retained for rollback

---

### FR-SELF-TUNING-005: Audit Trail

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-005`

**Description**:
Every applied adjustment is recorded in an in-memory audit log with full context: parameter key, tier, old value, new value, reasoning, source (auto/confirmed/approved), snapshot ID, and timestamp.

**Acceptance Criteria**:
- [x] Audit entry created for every GREEN, YELLOW, RED, and rollback operation
- [x] `source` field reflects `auto`, `confirmed`, or `approved`
- [x] Rollback logged as `_rollback` parameter entry with RED tier
- [x] `get_adjustment_history` returns entries filterable by tier, parameter, and limit

---

### FR-SELF-TUNING-006: Rollback

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-006`

**Description**:
The rollback tool restores all trading parameters to a previous snapshot state. A pre-rollback snapshot is captured first to allow undoing the rollback itself.

**Acceptance Criteria**:
- [x] `rollback_adjustment` accepts optional `snapshot_id`; defaults to most recent prior snapshot
- [x] Takes current snapshot before restoring
- [x] Returns `rolled_back: true`, both snapshot IDs on success
- [x] Returns error if no prior snapshot exists
- [x] Rollback event logged to audit trail

---

### FR-SELF-TUNING-007: Tuning Dashboard

**Priority**: ☑ Medium
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-007`

**Description**:
The `get_tuning_dashboard` tool aggregates trading performance, current basic and strategy settings, AI config suggestions, open position count, recent adjustment history, and latest snapshot metadata into a single read-only view.

**Acceptance Criteria**:
- [x] Dashboard fetches data from 5 APIs in parallel
- [x] Returns `current_settings`, `strategy_settings`, `performance`, `ai_suggestions`, `open_positions_count`, `recent_adjustments` (last 5), `last_snapshot`
- [x] Partial failures return `{ error: ... }` in that field rather than failing the entire call
- [x] Marked `readOnlyHint: true`

---

### FR-SELF-TUNING-008: MCP Tool Integration

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:FR-SELF-TUNING-008`

**Description**:
8 MCP tools expose the self-tuning engine to AI agents.

| Tool | Tier | Action |
|------|------|--------|
| `get_tuning_dashboard` | — | Read aggregated view |
| `get_parameter_bounds` | — | Read all params + cooldown status |
| `apply_green_adjustment` | GREEN | Auto-apply with notification |
| `request_yellow_adjustment` | YELLOW | Request (get token) or confirm (supply token) |
| `request_red_adjustment` | RED | Request (get warning) or approve (supply text) |
| `get_adjustment_history` | — | Query audit trail |
| `rollback_adjustment` | RED | Restore to snapshot |
| `take_parameter_snapshot` | — | Manual snapshot |

**Acceptance Criteria**:
- [x] All 8 tools registered at MCP server startup
- [x] Read-only tools annotated `readOnlyHint: true`
- [x] All tools validate parameter tier before applying
- [x] All mutating tools take snapshot before API write
- [x] Nested API paths (e.g., `signal_pipeline.*`) use fetch-merge-PUT pattern to avoid clobbering sibling fields

---

## Implementation Notes

**Code Locations**:
- Main tools: `mcp-server/src/tools/tuning.ts`
- Parameter registry: `mcp-server/src/tuning/bounds.ts`
- Types: `mcp-server/src/tuning/types.ts`
- Audit log: `mcp-server/src/tuning/audit.ts`
- Snapshot store: `mcp-server/src/tuning/snapshot.ts`
- Confirm token crypto: `mcp-server/src/security.ts`
- Tool registration: `mcp-server/src/server.ts` → `registerTuningTools()`

**Nested API Paths**: Parameters targeting `indicator-settings` with dotted `apiField` (e.g., `signal_pipeline.min_weighted_threshold`) use a fetch-merge-PUT strategy to preserve sibling fields within the same section.

**Signal Interval**: `signal_interval_minutes` converts minutes to seconds (`interval_seconds = minutes × 60`) before calling `/api/paper-trading/signal-interval`.

**Engine Start/Stop**: `engine_running` RED parameter calls POST `/api/paper-trading/start` or `/api/paper-trading/stop` rather than a settings PUT.

---

## Dependencies

- `mcp-server` must be running (port 8090)
- Rust API must be reachable at `http://localhost:8080`
- In-memory state: audit log and snapshots are lost on MCP server restart
- `@modelcontextprotocol/sdk` v1.26.0
- `zod` for input schema validation
