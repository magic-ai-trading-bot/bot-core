# Debug Report: leverage=10 and tight SL bug in paper trading engine

**Date:** 2026-03-03
**Severity:** CRITICAL — trades executing at 2x the max_leverage cap; SL hit in 20-54s
**Status:** Root cause identified, fixes specified

---

## Executive Summary

Two compounding bugs caused all 6 trades to execute with leverage=10 (above max_leverage=5) and SL at ~0.22% price distance (should be ~1.67% price at default 5% PnL / 3x):

1. **Bug A — `add_symbol_to_settings` hardcodes `leverage: Some(10)` and `stop_loss_pct: Some(2.0)`** for any new symbol added dynamically. These per-symbol values override `default_leverage: 3` and `default_stop_loss_pct: 5.0`. If symbols were added before the "OPTIMIZED" defaults were changed (or if the DB still holds old entries), every trade for those symbols uses stale bad values.

2. **Bug B — `max_leverage` is never enforced at execution time.** `max_leverage: 5` exists in `RiskSettings` and is validated in `PaperTradingSettings::validate()` only to check it doesn't exceed 125 — it is never used to cap actual trade leverage. So even if leverage ends up as 10 (from either signal or symbol settings), nothing stops it.

---

## Technical Analysis

### Bug A — Hardcoded leverage=10 and stop_loss=2.0 in `add_symbol_to_settings`

**File:** `rust-core-engine/src/paper_trading/engine.rs`
**Lines:** 3828–3838

```rust
// Add with default settings
let symbol_settings = crate::paper_trading::settings::SymbolSettings {
    enabled: true,
    leverage: Some(10),          // BUG: hardcoded 10, should defer to basic.default_leverage
    position_size_pct: Some(5.0),
    stop_loss_pct: Some(2.0),    // BUG: hardcoded 2.0, should defer to risk.default_stop_loss_pct
    take_profit_pct: Some(4.0),  // BUG: hardcoded 4.0, should defer to risk.default_take_profit_pct
    trading_hours: None,
    min_price_movement_pct: None,
    max_positions: Some(1),
    custom_params: std::collections::HashMap::new(),
};
```

These are **old pre-optimization values** (the comment in `settings.rs:729` explicitly says `default_leverage` was changed from 10 → 3 as a "CRITICAL CHANGE"). When `add_symbol_to_settings` was called for BTC/ETH/SOL/BNB, it stored `leverage: Some(10)` and `stop_loss_pct: Some(2.0)` into the DB. These were then loaded on startup (`engine.rs:149`) and returned by `get_symbol_settings()` at `settings.rs:1096-1098` as the effective leverage.

**Execution path:**
```
process_trading_signal()
  → get_symbol_settings(&signal.symbol)           # settings.rs:1091
      → symbol_specific.leverage = Some(10)       # DB-loaded stale value
      → returns EffectiveSymbolSettings { leverage: 10, stop_loss_pct: 2.0 }
  → let leverage = symbol_settings.leverage       # engine.rs:1423 → 10
  → let lev = leverage as f64                     # engine.rs:1443 → 10.0
  → SL = entry_price * (1.0 - 2.0/(10*100))      # engine.rs:1513 → -0.20% price
```

**Math verification against trade data:**
- ETH: `1996.57 * (1 - 2.0/1000) = 1996.57 * 0.998 = 1992.58` (observed SL: 1992.20, diff due to slippage/price move during execution)
- BTC: `68163 * (1 - 2.0/1000) = 68163 * 0.998 = 67999` (observed SL: 67996) ✓
- SOL: `85.52 * (1 - 2.0/1000) = 85.52 * 0.998 = 85.35` (observed SL: 85.34) ✓

The formula is correct for PnL-based SL. The inputs are wrong.

### Bug B — `max_leverage` never enforced at execution

**File:** `rust-core-engine/src/paper_trading/engine.rs`

There is **no reference to `max_leverage`** anywhere in `engine.rs`. `grep "max_leverage" engine.rs` returns zero results. The field exists in `RiskSettings` and is serialized/deserialized, but it is only validated to be ≤125 (not used as a runtime cap).

Both `process_trading_signal` (line 1423) and `close_and_reverse_position` (line 2400) do:
```rust
let leverage = symbol_settings.leverage;  // no .min(max_leverage) applied
```

Even if a signal provides `suggested_leverage`, that field is also **completely ignored** — the engine only uses `symbol_settings.leverage` for actual execution.

### SL formula itself

The formula `entry_price * (1.0 - stop_loss_pct / (lev * 100.0))` is correct for PnL-based SL:
- `stop_loss_pct / (lev * 100)` = price movement % at which PnL% is reached
- With correct inputs (lev=3, sl_pct=5): `5/(3*100) = 1.67%` price → reasonable
- With buggy inputs (lev=10, sl_pct=2): `2/(10*100) = 0.20%` price → too tight

The formula is not a bug; the symbol-specific settings are.

---

## Root Cause Chain

```
2026-03-03 ~16:00 UTC
  │
  ├─ symbols (BTC/ETH/SOL/BNB) were previously added via add_symbol_to_settings()
  │   └─ stored to DB: leverage=10, stop_loss_pct=2.0 (pre-optimization values)
  │
  ├─ Engine started, loaded settings from DB (engine.rs:149)
  │   └─ symbol entries loaded with old stale values
  │
  ├─ 16:05-16:15 UTC: 6 signals generated → process_trading_signal() called
  │   └─ get_symbol_settings() returns stale symbol settings (lev=10, sl=2.0)
  │   └─ max_leverage cap NEVER checked → 10 > 5 cap silently ignored
  │
  └─ All 6 trades: leverage=10, SL distance=0.20-0.24%, hit in 20-54s
```

---

## File:Line Summary

| Bug | File | Line(s) | Description |
|-----|------|---------|-------------|
| A (primary) | `engine.rs` | 3831 | `leverage: Some(10)` hardcoded in add_symbol_to_settings |
| A (secondary) | `engine.rs` | 3833 | `stop_loss_pct: Some(2.0)` hardcoded |
| A (secondary) | `engine.rs` | 3834 | `take_profit_pct: Some(4.0)` hardcoded |
| B | `engine.rs` | 1423 | `let leverage = symbol_settings.leverage` — no max_leverage cap |
| B | `engine.rs` | 2400 | Same in `close_and_reverse_position` |
| B | `settings.rs` | 931-933 | `max_leverage` only validated ≤125, never used as cap |

---

## Recommended Fixes

### Fix 1 — `add_symbol_to_settings`: use `None` to defer to global defaults

`rust-core-engine/src/paper_trading/engine.rs:3829-3838`

```rust
// BEFORE
let symbol_settings = crate::paper_trading::settings::SymbolSettings {
    enabled: true,
    leverage: Some(10),          // ← remove hardcode
    position_size_pct: Some(5.0),
    stop_loss_pct: Some(2.0),    // ← remove hardcode
    take_profit_pct: Some(4.0),  // ← remove hardcode
    trading_hours: None,
    min_price_movement_pct: None,
    max_positions: Some(1),
    custom_params: std::collections::HashMap::new(),
};

// AFTER
let symbol_settings = crate::paper_trading::settings::SymbolSettings {
    enabled: true,
    leverage: None,              // defer to basic.default_leverage (3)
    position_size_pct: None,     // defer to basic.default_position_size_pct (2.0)
    stop_loss_pct: None,         // defer to risk.default_stop_loss_pct (5.0)
    take_profit_pct: None,       // defer to risk.default_take_profit_pct (10.0)
    trading_hours: None,
    min_price_movement_pct: None,
    max_positions: Some(1),
    custom_params: std::collections::HashMap::new(),
};
```

### Fix 2 — Enforce `max_leverage` cap at execution

`rust-core-engine/src/paper_trading/engine.rs:1422-1423` and `2399-2400`

```rust
// BEFORE (line 1423 and 2400)
let leverage = symbol_settings.leverage;

// AFTER — cap at max_leverage
let leverage = symbol_settings.leverage.min(settings.risk.max_leverage);
```

Note: `settings` is still in scope at line 1423 (dropped at line 1610), so this is a one-liner fix.

### Fix 3 — Clear stale DB symbol settings (operational)

After deploying the code fix, reset the stale DB entries. Two options:

**Option A** — Reset via existing API:
```bash
curl -X POST http://localhost:8080/api/paper-trading/reset
```
This wipes portfolio and reloads settings. After reset, symbols will be re-added with `None` values (deferring to correct defaults).

**Option B** — Remove stale symbol overrides from DB directly:
```javascript
// MongoDB shell
db.paper_trading_settings.updateMany({}, {
  $set: {
    "symbols.BTCUSDT.leverage": null,
    "symbols.ETHUSDT.leverage": null,
    "symbols.SOLUSDT.leverage": null,
    "symbols.BNBUSDT.leverage": null,
    "symbols.BTCUSDT.stop_loss_pct": null,
    "symbols.ETHUSDT.stop_loss_pct": null,
    "symbols.SOLUSDT.stop_loss_pct": null,
    "symbols.BNBUSDT.stop_loss_pct": null,
  }
})
```

### Fix 4 — Migration guard (optional, defense-in-depth)

Add a startup migration in `PaperTradingEngine::new()` that resets any symbol-specific leverage > `max_leverage` to `None`:

```rust
// After loading settings from DB (engine.rs ~line 155)
// Clamp any stale symbol leverages that exceed max_leverage
{
    let max_lev = settings.risk.max_leverage;
    for sym_settings in settings.symbols.values_mut() {
        if let Some(lev) = sym_settings.leverage {
            if lev > max_lev {
                warn!("Symbol leverage {} exceeds max_leverage {}, resetting to None", lev, max_lev);
                sym_settings.leverage = None;
            }
        }
    }
}
```

---

## Priority

| Fix | Priority | Effort |
|-----|----------|--------|
| Fix 1 (hardcoded defaults) | P0 — deploy immediately | 5 min |
| Fix 2 (max_leverage cap) | P0 — deploy immediately | 2 min |
| Fix 3 (clear DB) | P0 — after deploy | 2 min |
| Fix 4 (migration guard) | P1 — next sprint | 30 min |

---

## Unresolved Questions

1. Are there other call sites that add symbols with hardcoded leverage (e.g., strategy_optimizer.rs or market data API handlers)?
   → Run: `grep -r "leverage: Some(" rust-core-engine/src/` to audit all hardcoded symbol leverage values.

2. Does `close_and_reverse_position` (line 2399-2400) also need the max_leverage cap? Yes — same fix applies.

3. Is `suggested_leverage` in `AITradingSignal` intentionally ignored? If AI signals are expected to influence leverage, a deliberate decision is needed on whether to honor `signal.suggested_leverage` (with max_leverage cap) or always use symbol settings. Current behavior: always ignored.

4. The `position_size_pct: Some(5.0)` hardcoded in `add_symbol_to_settings` (line 3832) is 2.5x the `default_position_size_pct: 2.0`. This wasn't mentioned in the bug report but may also be causing over-sized positions.
