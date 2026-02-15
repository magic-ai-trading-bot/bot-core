# Phase 03: Self-Tuning Engine & Guardrails

**Status**: Pending | **Est.**: 2 days | **Priority**: P1 (high-value feature)

## Context Links

- [Research: Self-Tuning Patterns (AEGIS)](../research/researcher-02-mcp-server-patterns.md#5-self-tuning-ai-agent-patterns)
- [BotCore Paper Trading Settings API](../../../../specs/02-design/2.3-api/API-RUST-CORE.md#paper-trading-endpoints)
- [Python AI Config Suggestions API](../../../../specs/02-design/2.3-api/API-PYTHON-AI.md#post-aiconfiganalysistrigger)
- Phase 02 (MCP tools available)

## Overview

Build a self-tuning engine as a set of MCP tools + resources that enables Claude to analyze trading performance, identify suboptimal parameters, and adjust them within bounded guardrails. The system uses a 3-tier autonomy model (GREEN/YELLOW/RED) to balance automation with user control.

## Key Insights

1. **Claude does the reasoning** -- the self-tuning "engine" is really a set of MCP tools that provide Claude with performance data + bounded adjustment capabilities. Claude decides WHAT to adjust and WHY.
2. **No autonomous loop** -- Claude only acts when triggered (by cron job or user message). No background daemon adjusting params without Claude's reasoning.
3. **Guardrails are hard limits** -- parameter bounds are enforced server-side in MCP tools, not just "suggested" to Claude. Even if Claude tries to set RSI oversold to 5, the tool rejects it.
4. **Every adjustment is logged** -- immutable audit trail with before/after values, reasoning, timestamp. Enables rollback.
5. **Rollback is always available** -- any parameter change can be undone within 24h, reverting to previous snapshot.

## Requirements

- GREEN tier: Auto-adjust with notification (RSI thresholds, signal interval, analysis frequency)
- YELLOW tier: Require user confirmation via Telegram/WhatsApp (stop loss %, position size, max trades)
- RED tier: Require explicit approval + reason text (real trading toggle, API keys, risk limits)
- All adjustments bounded within hard min/max ranges
- Full audit log of every parameter change
- Rollback capability (revert to any previous state within 24h)
- Performance snapshot before/after each adjustment

## Architecture

```
Claude (via OpenClaw cron or user message)
  |
  v
MCP Tools:
  1. get_tuning_dashboard -----> Aggregates performance data from multiple APIs
  2. get_parameter_bounds -----> Returns current values + allowed ranges
  3. suggest_adjustments ------> Claude's analysis formatted as adjustment proposals
  4. apply_green_adjustment ---> Auto-apply + notify user
  5. request_yellow_adjustment -> Send confirmation request to user
  6. request_red_adjustment ---> Send approval request with warnings
  7. get_adjustment_history ---> View past adjustments
  8. rollback_adjustment ------> Revert to previous parameter state
  |
  v
BotCore APIs (via existing Phase 02 tools):
  - GET /api/paper-trading/settings
  - PUT /api/paper-trading/basic-settings
  - PUT /api/paper-trading/strategy-settings
  - GET /api/trading/performance
  - GET /ai/config-suggestions
  - POST /ai/config-analysis/trigger
```

## Parameter Classification

### GREEN (Auto-Adjust + Notify)

| Parameter | API Endpoint | Min | Max | Step | Default |
|-----------|-------------|-----|-----|------|---------|
| RSI oversold threshold | PUT /paper-trading/basic-settings | 20 | 40 | 1 | 30 |
| RSI overbought threshold | PUT /paper-trading/basic-settings | 60 | 80 | 1 | 70 |
| Signal interval (minutes) | PUT /paper-trading/signal-interval | 3 | 30 | 1 | 5 |
| Analysis frequency (minutes) | Internal config | 3 | 60 | 1 | 5 |
| Strategy weights | PUT /paper-trading/strategy-settings | 0.1 | 1.0 | 0.05 | 1.0 |
| Confidence threshold | PUT /paper-trading/basic-settings | 0.50 | 0.90 | 0.05 | 0.65 |

### YELLOW (Require Confirmation)

| Parameter | API Endpoint | Min | Max | Step | Default |
|-----------|-------------|-----|-----|------|---------|
| Stop loss % | PUT /paper-trading/basic-settings | 0.5 | 5.0 | 0.5 | 2.0 |
| Take profit % | PUT /paper-trading/basic-settings | 1.0 | 10.0 | 0.5 | 4.0 |
| Position size % | PUT /paper-trading/basic-settings | 1.0 | 10.0 | 0.5 | 5.0 |
| Max concurrent positions | PUT /paper-trading/basic-settings | 1 | 8 | 1 | 4 |
| Leverage | PUT /paper-trading/basic-settings | 1 | 20 | 1 | 10 |
| Trailing stop % | PUT /paper-trading/trailing-stops/settings | 0.5 | 5.0 | 0.1 | 1.5 |

### RED (Require Explicit Approval)

| Parameter | API Endpoint | Min | Max | Step | Default |
|-----------|-------------|-----|-----|------|---------|
| Max daily loss % | PUT /paper-trading/basic-settings | 3.0 | 15.0 | 1.0 | 10.0 |
| Enable/disable strategies | PUT /paper-trading/strategy-settings | -- | -- | -- | -- |
| Paper trading start/stop | POST /paper-trading/start|stop | -- | -- | -- | -- |
| Real trading enable | N/A (future) | -- | -- | -- | false |

## Related Code Files

| File | Purpose |
|------|---------|
| `mcp-server/src/tools/tuning.ts` | Self-tuning MCP tools (8 tools) |
| `mcp-server/src/tuning/bounds.ts` | Parameter bounds definition and validation |
| `mcp-server/src/tuning/audit.ts` | Audit log for parameter changes |
| `mcp-server/src/tuning/snapshot.ts` | Parameter snapshot capture and rollback |
| `mcp-server/src/tuning/types.ts` | Tuning-specific types |

## Implementation Steps

### 1. Parameter Bounds Registry (~2h)

**`src/tuning/bounds.ts`**:
```typescript
interface ParameterBound {
  name: string;
  tier: 'GREEN' | 'YELLOW' | 'RED';
  currentValue: number | boolean | string;
  min?: number;
  max?: number;
  step?: number;
  type: 'number' | 'boolean' | 'enum';
  apiEndpoint: string;
  apiField: string;
  description: string;
}

// Registry of all tunable parameters with hard bounds
const PARAMETER_BOUNDS: Record<string, ParameterBound> = { ... };

function validateAdjustment(param: string, newValue: unknown): ValidationResult {
  // Enforce hard min/max bounds
  // Reject if outside allowed range
  // Return clamped value if within range
}
```

### 2. Audit Logger (~2h)

**`src/tuning/audit.ts`**:
```typescript
interface AuditEntry {
  id: string;            // UUID
  timestamp: string;     // ISO 8601
  parameter: string;     // e.g., "rsi_oversold"
  tier: 'GREEN' | 'YELLOW' | 'RED';
  oldValue: unknown;
  newValue: unknown;
  reasoning: string;     // Claude's explanation
  source: 'auto' | 'confirmed' | 'approved';
  approvedBy?: string;   // User ID for YELLOW/RED
  snapshotId: string;    // Reference to full parameter snapshot
}

// Append-only log stored as JSON lines file
// Mounted as Docker volume for persistence
// Path: /data/audit/tuning-audit.jsonl
```

### 3. Parameter Snapshots (~2h)

**`src/tuning/snapshot.ts`**:
```typescript
interface ParameterSnapshot {
  id: string;
  timestamp: string;
  parameters: Record<string, unknown>;  // Full parameter state
  performance: {                        // Performance at snapshot time
    winRate: number;
    totalPnl: number;
    sharpeRatio: number;
    maxDrawdown: number;
  };
}

// Take snapshot before any adjustment
// Store last 48 snapshots (24h at 30-min intervals)
// Enable rollback to any snapshot
```

### 4. Tuning Dashboard Tool (~2h)

**`get_tuning_dashboard`**:
- Aggregates data from multiple BotCore APIs in one call:
  - Current settings (`GET /api/paper-trading/settings`)
  - Trading performance (`GET /api/trading/performance`)
  - AI config suggestions (`GET /ai/config-suggestions`)
  - Open positions (`GET /api/paper-trading/trades/open`)
  - Correlation analysis (`GET /api/paper-trading/correlation-analysis`)
- Returns structured dashboard that Claude can reason about
- Includes trend indicators: "win rate trending down over last 7 days" etc.

### 5. Adjustment Tools (~4h)

**`apply_green_adjustment`**:
- Takes: `{ parameter, newValue, reasoning }`
- Validates against bounds (hard reject if out of range)
- Takes parameter snapshot before change
- Applies change via BotCore API
- Logs to audit trail
- Returns: `{ applied: true, notification: "RSI oversold changed: 30 -> 25. Reason: ..." }`
- OpenClaw sends notification to Telegram/WhatsApp

**`request_yellow_adjustment`**:
- Takes: `{ parameter, newValue, reasoning }`
- Validates against bounds
- Does NOT apply -- generates confirmation request
- Returns: `{ pending: true, message: "Confirm: Change stop_loss from 2.0% to 1.5%? Reason: ...", confirm_token: "..." }`
- User replies "yes" or "confirm" -> Claude calls `apply_green_adjustment` with confirmed token

**`request_red_adjustment`**:
- Takes: `{ parameter, newValue, reasoning, riskAssessment }`
- Validates against bounds
- Returns warning with full risk assessment
- Requires user to explicitly type approval text (not just "yes")
- Example: "Type 'APPROVE STOP ENGINE' to confirm stopping paper trading engine"

**`rollback_adjustment`**:
- Takes: `{ snapshotId }` or `{ lastN: 1 }`
- Restores all parameters from snapshot
- Logs rollback in audit trail
- Takes new snapshot after rollback

### 6. History & Analysis Tools (~1h)

**`get_adjustment_history`**:
- Returns last N audit entries with filtering by tier, parameter, date range
- Includes performance delta (was the adjustment beneficial?)

**`get_parameter_bounds`**:
- Returns all tunable parameters with current values and allowed ranges
- Grouped by tier (GREEN/YELLOW/RED)
- Claude uses this to understand what it can adjust

### 7. Prompt Template Resource (~1h)

Add MCP resource `tuning://system-prompt` that provides Claude with instructions for self-tuning:
- How to interpret the tuning dashboard
- When to suggest adjustments (e.g., "if win rate drops below 55% for 3 consecutive days")
- How to format adjustment reasoning
- Guardrail rules (never adjust RED params without explicit user instruction)

## Todo List

- [ ] Define all tunable parameters with bounds in `src/tuning/bounds.ts`
- [ ] Implement parameter validation (hard min/max enforcement)
- [ ] Implement audit logger (append-only JSON lines)
- [ ] Implement parameter snapshot capture and storage
- [ ] Implement rollback mechanism (restore from snapshot)
- [ ] Implement `get_tuning_dashboard` tool (aggregated performance view)
- [ ] Implement `get_parameter_bounds` tool (current values + ranges)
- [ ] Implement `apply_green_adjustment` tool (auto-apply + notify)
- [ ] Implement `request_yellow_adjustment` tool (confirmation flow)
- [ ] Implement `request_red_adjustment` tool (approval flow with risk warning)
- [ ] Implement `get_adjustment_history` tool (audit trail viewer)
- [ ] Implement `rollback_adjustment` tool (revert to snapshot)
- [ ] Create `tuning://system-prompt` MCP resource
- [ ] Add `/data/audit/` volume mount to Docker compose
- [ ] Write unit tests for bounds validation (every edge case)
- [ ] Write unit tests for audit log integrity
- [ ] Write integration test: GREEN adjustment end-to-end
- [ ] Write integration test: YELLOW confirmation flow
- [ ] Write integration test: rollback after bad adjustment

## Success Criteria

1. GREEN adjustments auto-apply within bounds and send notification
2. YELLOW adjustments require user confirmation before applying
3. RED adjustments require explicit approval text
4. All adjustments logged in immutable audit trail
5. Rollback works -- restores exact previous parameter state
6. Out-of-bounds values are rejected (not clamped silently)
7. `get_tuning_dashboard` provides enough data for Claude to make informed decisions
8. Performance snapshots captured before every adjustment

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Claude adjusts too aggressively | Medium | High | Hard bounds, step limits, max 3 adjustments/day per GREEN param |
| Audit log corruption | Low | High | Append-only, Docker volume, daily backup |
| Cascading bad adjustments | Medium | High | Max 1 GREEN adjustment per parameter per 6 hours |
| User ignores YELLOW confirmations | Low | Medium | Expire pending adjustments after 1 hour |

## Security Considerations

- Parameter bounds are server-side enforced -- Claude cannot bypass them
- Audit log is append-only (no delete/modify)
- RED tier changes require specific approval text to prevent accidental confirmation
- Rate limit on GREEN auto-adjustments: max 3 per parameter per day
- Cooldown period: 6 hours between adjustments to same parameter
- Snapshot retention: 48 snapshots max (auto-prune oldest)

## Next Steps

After this phase: proceed to Phase 04 to deploy OpenClaw and configure Telegram/WhatsApp channels so users can interact with the tuning system.
