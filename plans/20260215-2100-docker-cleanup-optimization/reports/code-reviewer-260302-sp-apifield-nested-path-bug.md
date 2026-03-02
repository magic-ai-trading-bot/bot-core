# Code Review: sp_* apiField Nested Path Bug

**Date**: 2026-03-02
**Files reviewed**:
- `mcp-server/src/tuning/bounds.ts` (376 lines)
- `mcp-server/src/tools/tuning.ts` (434 lines)
- `mcp-server/src/tuning/snapshot.ts` (87 lines)
- `mcp-server/src/tuning/types.ts` (51 lines)

**Review focus**: Whether dotted `apiField` paths like `"signal_pipeline.min_weighted_threshold"` are correctly serialized into nested request bodies.

---

## Code Review Summary

### Overall Assessment

**CRITICAL BUG CONFIRMED.** The self-tuning engine does NOT support nested dotted paths in `apiField`. Every `sp_*` parameter in `bounds.ts` (15 parameters total) will silently send a malformed request body to the Rust API when adjusted, with no error returned to the caller. The adjusted value is lost.

---

## Critical Issues

### [CRITICAL] Flat bracket notation creates a literal dotted key instead of a nested object

**Location**: `mcp-server/src/tools/tuning.ts`, lines 127-129, 207-209, 299-303

All three adjustment paths (`apply_green_adjustment`, `request_yellow_adjustment`, `request_red_adjustment`) build the request body using the same pattern:

```typescript
// apply_green_adjustment (line 127-129)
const body = parameter === "signal_interval_minutes"
  ? { interval_seconds: (effectiveValue as number) * 60 }
  : { [bound.apiField]: effectiveValue };

// request_yellow_adjustment (line 207-209)
body: { [bound.apiField]: effectiveValue },

// request_red_adjustment (line 299-303)
body: { [bound.apiField]: effectiveValue },
```

When `bound.apiField` is `"signal_pipeline.min_weighted_threshold"`, JavaScript computed property syntax `{ [bound.apiField]: effectiveValue }` produces:

```json
{ "signal_pipeline.min_weighted_threshold": 60 }
```

The Rust API expects:

```json
{ "signal_pipeline": { "min_weighted_threshold": 60 } }
```

These are structurally different. The Rust `serde` deserializer will not recognize the dotted-key form and will either ignore the field silently or return an error. Either way, **the parameter change is not applied.**

**Affected parameters** (all 15 `sp_*` entries in `bounds.ts`):

| Key | apiField |
|-----|----------|
| `sp_min_weighted_threshold` | `signal_pipeline.min_weighted_threshold` |
| `sp_rsi_bull_threshold` | `signal_pipeline.rsi_bull_threshold` |
| `sp_rsi_bear_threshold` | `signal_pipeline.rsi_bear_threshold` |
| `sp_volume_confirm_multiplier` | `signal_pipeline.volume_confirm_multiplier` |
| `sp_confidence_max` | `signal_pipeline.confidence_max` |
| `sp_neutral_confidence` | `signal_pipeline.neutral_confidence` |
| `sp_counter_trend_mode` | `signal_pipeline.counter_trend_mode` |
| `sp_bb_bull_threshold` | `signal_pipeline.bb_bull_threshold` |
| `sp_bb_bear_threshold` | `signal_pipeline.bb_bear_threshold` |
| `sp_stoch_overbought` | `signal_pipeline.stoch_overbought` |
| `sp_stoch_oversold` | `signal_pipeline.stoch_oversold` |
| `sp_weight_15m` | `signal_pipeline.weight_15m` |
| `sp_weight_30m` | `signal_pipeline.weight_30m` |
| `sp_weight_1h` | `signal_pipeline.weight_1h` |
| `sp_counter_trend_enabled` | `signal_pipeline.counter_trend_enabled` |

**Why it's silent**: The `apiRequest` call returns `res.success` and the tool returns `toolSuccess(...)` as if the change applied. There is no validation that the server actually accepted the nested field.

---

### [CRITICAL] Snapshot reads wrong endpoint for sp_* parameters

**Location**: `mcp-server/src/tuning/snapshot.ts`, line 17-19

```typescript
const [settingsRes, performanceRes] = await Promise.all([
  apiRequest("rust", "/api/paper-trading/basic-settings", { timeoutMs: 10_000 }),
  ...
]);
```

The snapshot **only reads `/api/paper-trading/basic-settings`**, but all `sp_*` parameters live at `/api/paper-trading/indicator-settings`. As a result:

1. `snapshot.parameters["signal_pipeline.min_weighted_threshold"]` is always `undefined`.
2. The audit log records `oldValue: undefined` for every `sp_*` adjustment.
3. `rollback_adjustment` only restores basic-settings; it never restores signal pipeline parameters.

---

### [HIGH] Audit log records `apiField` dotted string as key, not nested path

**Location**: `mcp-server/src/tools/tuning.ts`, lines 141, 214, 308

```typescript
const oldValue = snapshot.parameters[bound.apiField];
```

For `sp_min_weighted_threshold`, this looks up `snapshot.parameters["signal_pipeline.min_weighted_threshold"]` which is always `undefined` (since the snapshot doesn't contain that key at all). The audit trail is therefore useless for `sp_*` parameters.

---

## Recommended Fix

### Option A — Build nested body from dotted path (minimal change, correct approach)

Add a helper function in `mcp-server/src/tuning/` (e.g., `utils.ts`):

```typescript
/**
 * Convert a dotted apiField path into a nested object.
 * "signal_pipeline.min_weighted_threshold" → { signal_pipeline: { min_weighted_threshold: value } }
 * "rsi_oversold" → { rsi_oversold: value }
 */
export function buildApiBody(apiField: string, value: unknown): Record<string, unknown> {
  const parts = apiField.split(".");
  if (parts.length === 1) return { [apiField]: value };

  const body: Record<string, unknown> = {};
  let cursor = body;
  for (let i = 0; i < parts.length - 1; i++) {
    cursor[parts[i]] = {};
    cursor = cursor[parts[i]] as Record<string, unknown>;
  }
  cursor[parts[parts.length - 1]] = value;
  return body;
}
```

Then replace all three body-construction sites in `tuning.ts`:

```typescript
// apply_green_adjustment (replaces lines 127-129)
const body = parameter === "signal_interval_minutes"
  ? { interval_seconds: (effectiveValue as number) * 60 }
  : buildApiBody(bound.apiField, effectiveValue);

// request_yellow_adjustment (replaces line 208)
body: buildApiBody(bound.apiField, effectiveValue),

// request_red_adjustment (replaces line 302)
body: buildApiBody(bound.apiField, effectiveValue),
```

### Option B — Store flat apiField for leaf key and add a separate nestedPath (alternative design)

Add an optional `apiBody?: (value: unknown) => Record<string, unknown>` factory to `ParameterBound`. More flexible but more verbose. Option A is preferred.

### Also fix snapshot.ts

Update `takeSnapshot()` to also fetch `/api/paper-trading/indicator-settings` and merge its `signal_pipeline` sub-object into `snapshot.parameters` with the dotted-key convention, so `oldValue` lookups work:

```typescript
const [settingsRes, indicatorRes, performanceRes] = await Promise.all([
  apiRequest("rust", "/api/paper-trading/basic-settings", { timeoutMs: 10_000 }),
  apiRequest("rust", "/api/paper-trading/indicator-settings", { timeoutMs: 10_000 }),
  apiRequest("rust", "/api/trading/performance", { timeoutMs: 10_000 }),
]);

// Flatten signal_pipeline sub-fields into dotted keys for audit lookup
const indicatorData = indicatorRes.success
  ? (indicatorRes.data as Record<string, unknown>)
  : {};
const flatIndicators: Record<string, unknown> = {};
const sp = indicatorData["signal_pipeline"] as Record<string, unknown> | undefined;
if (sp) {
  for (const [k, v] of Object.entries(sp)) {
    flatIndicators[`signal_pipeline.${k}`] = v;
  }
}

const snapshot: ParameterSnapshot = {
  id: randomUUID(),
  timestamp: new Date().toISOString(),
  parameters: {
    ...(settingsRes.success ? (settingsRes.data as Record<string, unknown>) : {}),
    ...flatIndicators,
  },
  ...
};
```

Also update `restoreFromSnapshot` to POST signal_pipeline fields back to `/api/paper-trading/indicator-settings` separately.

---

## Positive Observations

- The tier system (GREEN/YELLOW/RED) and cooldown enforcement are correctly implemented.
- Bounds validation (`validateAdjustment`) correctly handles number, enum, and boolean types.
- The `signal_interval_minutes` special case (lines 127-128) shows the intended pattern for parameter-specific body construction — the same pattern just needs to be generalized to all dotted paths.
- Audit log structure and confirm-token security for YELLOW tier are well designed.

---

## Recommended Actions

1. **[CRITICAL, do now]** Add `buildApiBody()` helper in `mcp-server/src/tuning/utils.ts` and apply it in all three body-construction sites in `tuning.ts`. This unblocks all 15 `sp_*` parameters.
2. **[CRITICAL, do now]** Update `takeSnapshot()` to also read `/api/paper-trading/indicator-settings` and flatten `signal_pipeline.*` keys into `snapshot.parameters`.
3. **[HIGH]** Update `restoreFromSnapshot()` to restore signal pipeline settings via `/api/paper-trading/indicator-settings` using the rebuilt nested body.
4. **[MEDIUM]** Add a unit test that calls `apply_green_adjustment` for `sp_min_weighted_threshold` and asserts the HTTP body sent is `{ signal_pipeline: { min_weighted_threshold: 60 } }`, not the dotted-string form.
5. **[LOW]** Consider adding a startup validation that iterates `PARAMETER_BOUNDS` and asserts every dotted `apiField` maps to an `apiEndpoint` containing `indicator-settings` (and vice versa for flat fields). This prevents future regressions when new parameters are added.

---

## Metrics

- **Files reviewed**: 4
- **Lines analyzed**: ~950
- **Critical issues**: 2
- **High issues**: 1
- **Affected parameters**: 15 of 30 total tunable parameters (50%)
- **Type coverage**: No TypeScript errors in the code itself; the bug is a logic/runtime issue, not a type error (`string` is a valid computed key).

---

## Unresolved Questions

1. Does the Rust `/api/paper-trading/indicator-settings` PUT handler currently accept a partial `signal_pipeline` object (merge), or does it require the full struct? This affects whether Option A's `{ signal_pipeline: { min_weighted_threshold: 60 } }` body would clobber other signal_pipeline fields. If it's a full-replace, the fix must first GET current values and merge before PUT.
2. Does `restoreFromSnapshot` need to handle the signal_pipeline endpoint ordering (e.g., basic-settings PUT before indicator-settings PUT)?
