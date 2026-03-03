# Self-Tuning Engine

**Spec**: FR-SELF-TUNING | **Status**: Implemented | **Updated**: 2026-03-03

## Quick Reference

| Item | Value |
|------|-------|
| Code | `mcp-server/src/tools/tuning.ts`, `mcp-server/src/tuning/` |
| Spec | `specifications/01-requirements/1.1-functional-requirements/FR-SELF-TUNING.md` |
| MCP tools | 8 tools (`get_tuning_dashboard`, `get_parameter_bounds`, `apply_green_adjustment`, ...) |
| Cron | 3x daily: `0 2,8,16 * * *` (Asia/Ho_Chi_Minh) |
| Audit log | `/data/audit/tuning-audit.jsonl` (JSONL, append-only) |
| Snapshots | In-memory, max 48 snapshots |

### Code Locations

| File | Purpose |
|------|---------|
| `src/tools/tuning.ts` | 8 MCP tool definitions |
| `src/tuning/bounds.ts` | Parameter registry with hard limits per tier |
| `src/tuning/audit.ts` | JSONL audit logger + cooldown tracking |
| `src/tuning/snapshot.ts` | Parameter state snapshots (pre-change) |
| `src/tuning/types.ts` | TypeScript interfaces |
| `src/security.ts` | Confirm token generation/validation |
| `openclaw/config/cron/self-tuning.json` | Cron schedule + AI prompt |

---

## How It Works

### 3-Tier Approval System

```
AI agent wants to adjust parameter
            |
            v
    Determine tier from bounds.ts
            |
    ┌───────┴────────┬────────────────┐
    v                v                v
  GREEN           YELLOW            RED
  (auto)       (confirm token)   (approval text)
    |                |                |
  Apply           Generate         Generate
  immediately     token →          prompt →
  + notify        user types       user types
                  token back       "CONFIRM ..."
```

### GREEN Tier Flow

1. AI calls `apply_green_adjustment(parameter, new_value, reasoning)`
2. Validates: tier == GREEN, within bounds, not in cooldown
3. Takes parameter snapshot via `takeSnapshot()`
4. Calls Rust API to apply change (handles nested `signal_pipeline.*` paths)
5. Writes to audit log (`/data/audit/tuning-audit.jsonl`)
6. Returns notification string for Telegram delivery

### YELLOW Tier Flow

1. AI calls `request_yellow_adjustment(parameter, new_value, reasoning)`
2. Returns `confirm_token` (HMAC-signed, short-lived)
3. User reviews and provides token back to AI
4. AI calls same tool with `confirm_token` field set
5. Token validated → change applied + audited

### RED Tier Flow

1. AI calls `request_red_adjustment(parameter, new_value, reasoning)`
2. Returns required approval string (e.g., "CONFIRM APPROVE leverage=15")
3. User explicitly types approval string
4. AI passes it back → validated → applied

### Rollback

`rollback_to_snapshot(snapshot_id)` restores all parameters from a snapshot via PUT to Rust API. Snapshots stored in-memory (max 48, FIFO eviction). Each change auto-takes a snapshot before applying.

### Cooldown Enforcement

Per-parameter cooldowns tracked in `Map<string, number>` (last adjustment time). Default: 6 hours for most params, 1 hour for signal_interval and minor params. Enforced in `isInCooldown()`. Cooldown resets on MCP server restart (in-memory only).

---

## Tunable Parameters

### GREEN Tier (Auto-adjust + notify)

| Key | Name | Range | Default | Cooldown |
|-----|------|-------|---------|---------|
| `rsi_oversold` | RSI Oversold | 20–40 | 30 | 6h |
| `rsi_overbought` | RSI Overbought | 60–80 | 70 | 6h |
| `signal_interval_minutes` | Signal Interval | 3–30 min | 5 | 1h |
| `confidence_threshold` | Min Confidence | 0.50–0.90 | 0.65 | 6h |
| `data_resolution` | Timeframe | 1m/3m/5m/15m/30m/1h/4h/1d | 15m | 1h |
| `stop_loss_percent` | Stop Loss % (PnL) | 1.0–20.0 | 10.0 | 6h |
| `take_profit_percent` | Take Profit % (PnL) | 2.0–40.0 | 20.0 | 6h |
| `min_required_indicators` | Min Indicators | 2–5 | 4 | 6h |
| `min_required_timeframes` | Min Timeframes | 1–4 | 3 | 6h |
| `sp_min_weighted_threshold` | Min Weighted % | 30–70 | 60 | 6h |
| `sp_rsi_bull_threshold` | RSI Bull Level | 50–65 | 55 | 6h |
| `sp_rsi_bear_threshold` | RSI Bear Level | 35–50 | 45 | 6h |
| `sp_volume_confirm_multiplier` | Volume Confirm | 1.0–2.0 | 1.2 | 6h |
| `sp_confidence_max` | Confidence Cap | 0.70–0.95 | 0.85 | 6h |
| `sp_neutral_confidence` | Neutral Confidence | 0.30–0.50 | 0.40 | 6h |
| `sp_counter_trend_block_offset` | Counter-Trend Offset | 0.0–0.15 | 0.05 | 1h |
| `sp_counter_trend_mode` | Counter-Trend Mode | block/reduce | block | 1h |

> Note: `stop_loss_percent` and `take_profit_percent` are PnL-based, not price%.
> Price move = pnl% / leverage. Always verify price_move in 0.5–3% range for crypto.

### YELLOW Tier (User confirmation required)

| Key | Name | Range | Default |
|-----|------|-------|---------|
| `position_size_percent` | Position Size % | 1.0–10.0 | 5.0 |
| `max_positions` | Max Positions | 1–8 | 4 |
| `leverage` | Leverage | 1–20 | 10 |
| `sp_bb_bull_threshold` | BB Bull Level | 0.1–0.4 | 0.3 |
| `sp_bb_bear_threshold` | BB Bear Level | 0.6–0.9 | 0.7 |
| `sp_stoch_overbought` | Stoch Overbought | 70–90 | 80 |
| `sp_stoch_oversold` | Stoch Oversold | 10–30 | 20 |
| `sp_weight_15m` | 15M Weight | 0.0–3.0 | 0.5 |
| `sp_weight_30m` | 30M Weight | 0.0–3.0 | 1.0 |
| `sp_weight_1h` | 1H Weight | 0.0–3.0 | 2.0 |
| `sp_counter_trend_enabled` | Counter-Trend Protection | bool | true |

---

## MCP Tools

| Tool | Tier access | Purpose |
|------|------------|---------|
| `get_tuning_dashboard` | Read | Aggregated view: settings, performance, AI suggestions, recent adjustments |
| `get_parameter_bounds` | Read | All params grouped by tier with current cooldown status |
| `apply_green_adjustment` | GREEN write | Auto-apply with reasoning |
| `request_yellow_adjustment` | YELLOW write | Request with token workflow |
| `confirm_yellow_adjustment` | YELLOW write | Apply after user confirms token |
| `request_red_adjustment` | RED write | Request with approval text workflow |
| `rollback_to_snapshot` | Write | Restore all params from snapshot |
| `get_adjustment_history` | Read | Filter by tier, parameter, date |

---

## Cron Schedule

**File**: `openclaw/config/cron/self-tuning.json`

```json
{
  "schedule": "0 2,8,16 * * *",  // 3x daily at 02:00, 08:00, 16:00 VN time
  "timeout_seconds": 300,
  "no_deliver": true              // AI executes; sends Telegram itself via tool
}
```

**AI Decision Rules** (from cron prompt):

| Condition | Action |
|-----------|--------|
| WR (10 recent) < 45% | Increase `confidence_threshold` by 0.05 (mandatory) |
| WR (10 recent) < 35% | Increase `confidence_threshold` by 0.10 + set `min_required_indicators = 5` |
| Symbol WR < 40% (10+ trades) | Send Telegram warning |
| SL hits too fast (< 30 min) | Increase `stop_loss_percent` |
| TP never hit | Decrease `take_profit_percent` |
| Too few signals | Decrease `confidence_threshold` or `min_required_indicators` |
| Market direction >= +0.70 | Set `long_only_mode = true` |
| Market direction <= -0.70 | Set `short_only_mode = true` |
| Otherwise | Both modes = false |

Max 3 GREEN adjustments per cron run. Always reports via Telegram.

---

## Audit Trail

**Location**: `/data/audit/tuning-audit.jsonl`

Each entry (one JSON per line):
```json
{
  "id": "uuid",
  "timestamp": "ISO-8601",
  "parameter": "confidence_threshold",
  "tier": "GREEN",
  "oldValue": 0.65,
  "newValue": 0.70,
  "reasoning": "WR dropped to 42%, increasing confidence filter",
  "source": "auto",
  "snapshotId": "uuid"
}
```

Fallback to `/tmp/tuning-audit.jsonl` if `/data/audit/` not writable.

---

## Snapshot Format

Snapshots read from Rust APIs before each change:
- `GET /api/paper-trading/basic-settings`
- `GET /api/paper-trading/indicator-settings`
- `GET /api/trading/performance`

Signal pipeline nested fields flattened to dotted keys (e.g., `signal_pipeline.min_weighted_threshold`) for audit trail lookup.

---

## API Endpoints (Rust backend)

| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/paper-trading/basic-settings` | GET/PUT | Main trading params |
| `/api/paper-trading/indicator-settings` | GET/PUT | Signal pipeline params |
| `/api/paper-trading/signal-interval` | PUT | Signal generation interval |
| `/api/trading/performance` | GET | Performance metrics for dashboard |
| `/api/paper-trading/config-suggestions/latest` | GET | AI-generated suggestions |

---

## Troubleshooting

| Symptom | Cause | Fix |
|---------|-------|-----|
| `in cooldown` error | Parameter adjusted < 6h ago | Wait for cooldown or check `get_parameter_bounds` for remaining time |
| `Unknown parameter` error | Wrong key name | Run `get_parameter_bounds` to list valid keys |
| Audit log empty | `/data/audit/` not mounted | Check Docker volume for mcp-server |
| Snapshots lost after restart | In-memory only | Expected; rollback unavailable after restart |
| YELLOW token expired | Token is short-lived | Request a new token |
| `Failed to apply adjustment` | Rust API down | Check `curl localhost:8080/api/health` |
