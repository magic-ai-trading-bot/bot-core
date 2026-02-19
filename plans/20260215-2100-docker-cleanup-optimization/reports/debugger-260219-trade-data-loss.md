# Debugger Report: Trade Data Loss on VPS Deploy

**Date**: 2026-02-19
**Severity**: HIGH (finance-critical — open positions not restored after redeploy)
**Status**: Root cause identified, fix ready

---

## Executive Summary

Every VPS redeploy causes all paper trade history to appear "lost" from the engine's in-memory state, even though the data physically exists in MongoDB. The engine logs a deserialization error on startup (`invalid type: map, expected an RFC 3339 formatted date and time string`) and silently continues without restoring any trades. The MongoDB volume and data are intact — this is NOT a data loss problem but a **data loading failure** caused by a BSON type mismatch.

---

## Root Cause

**`PaperTradingRecord.open_time` and `created_at` are stored as BSON strings, but deserialized as `DateTime<Utc>` (BSON Date).**

### Chain of Events

1. Rust saves a trade via `save_paper_trade()`: calls `insert_one(record)`.
2. The `PaperTradingRecord` struct has `open_time: DateTime<Utc>` with NO `#[serde(with = ...)]` annotation.
3. Without the annotation, `bson` crate serializes `DateTime<Utc>` as an **RFC 3339 string** (e.g. `"2026-02-16T03:35:36.402043472Z"`) instead of a BSON Date object.
4. On startup, `load_portfolio_from_storage()` calls `get_paper_trades_history()` → `cursor.try_collect::<Vec<PaperTradingRecord>>()`.
5. MongoDB cursor tries to deserialize the string field into `DateTime<Utc>` expecting a BSON Date — **fails with**: `invalid type: map, expected an RFC 3339 formatted date and time string`.
6. `load_portfolio_from_storage()` catches the error, logs a WARN, and returns `Ok(())` — **skipping all trade restoration silently**.
7. Engine starts with an empty portfolio, acting as if no trades ever existed.

### Evidence

**VPS log (rust-core-engine startup):**
```
INFO  📂 Loading portfolio from database...
WARN  ⚠️ Failed to load trades from database: Kind: invalid type: map, expected an RFC 3339 formatted date and time string
```

**MongoDB raw document inspection (via mongosh):**
```
open_time type: string, constructor=String, value=2026-02-16T03:35:36.402043472Z
close_time type: object, constructor=Date   ← correctly stored as BSON Date
created_at type: string, constructor=String ← also wrong
```

`close_time` is stored correctly as a BSON Date (it is set via a `$set` update doc that implicitly converts), but `open_time` and `created_at` are set at insert time from the Rust struct directly — which serializes them as strings.

**Current MongoDB state (data is intact):**
- `paper_trades`: 17 records (oldest: 2026-02-16, spanning 3 days)
- `portfolio_history`: 24 snapshots
- MongoDB volume: `bot-core_mongodb_data` exists and is mounted correctly

### Contrast with Working Structs

`TradeAnalysisRecord` and `ConfigSuggestionsRecord` in the same file correctly use:
```rust
#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
pub created_at: DateTime<Utc>,
```

`PaperTradingRecord`, `PortfolioHistoryRecord`, `AISignalRecord`, `PerformanceMetricsRecord` are **missing this annotation** on all `DateTime<Utc>` fields.

---

## Affected Files

- `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/storage/mod.rs`
  - `PaperTradingRecord` (lines 1630–1651): `open_time`, `close_time`, `created_at`
  - `PortfolioHistoryRecord` (lines 1654–1671): `timestamp`, `created_at`
  - `AISignalRecord` (lines 1673–1704): `created_at`, `timestamp`, `closed_at`
  - `PerformanceMetricsRecord` (lines 1706–1726): `date`, `created_at`

---

## Fix

### Option A — Add serde annotation to all DateTime fields (Correct, Long-term)

Add `#[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]` to every `DateTime<Utc>` field in the four structs above. For `Option<DateTime<Utc>>` fields, use the `deserialize_with` / `serialize_with` pair or the `bson::serde_helpers::chrono_datetime_as_bson_datetime` in an option wrapper.

**`PaperTradingRecord` fix:**
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaperTradingRecord {
    #[serde(rename = "_id", skip_serializing_if = "Option::is_none")]
    pub id: Option<mongodb::bson::oid::ObjectId>,
    pub trade_id: String,
    // ... other string/numeric fields unchanged ...
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub open_time: DateTime<Utc>,
    #[serde(
        default,
        serialize_with = "bson::serde_helpers::serialize_chrono_datetime_as_bson_datetime_opt",
        deserialize_with = "bson::serde_helpers::deserialize_chrono_datetime_from_bson_datetime_opt",
    )]
    pub close_time: Option<DateTime<Utc>>,
    pub ai_signal_id: Option<String>,
    pub ai_confidence: Option<f64>,
    pub close_reason: Option<String>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
}
```

Apply same pattern to `PortfolioHistoryRecord`, `AISignalRecord`, `PerformanceMetricsRecord`.

**IMPORTANT**: After this fix, existing string-format documents in MongoDB will still fail to deserialize (they were stored as strings, not BSON Dates). A one-time migration is required (see Option B).

### Option B — MongoDB Migration (Required after Option A)

Run this once on the VPS to convert string datetimes to proper BSON Dates:

```javascript
// Run in mongosh as admin:
db = db.getSiblingDB('bot_core');

// Fix paper_trades
db.paper_trades.find().forEach(function(doc) {
  var update = {};
  if (typeof doc.open_time === 'string') update.open_time = new Date(doc.open_time);
  if (typeof doc.created_at === 'string') update.created_at = new Date(doc.created_at);
  if (Object.keys(update).length > 0) {
    db.paper_trades.updateOne({_id: doc._id}, {$set: update});
  }
});
print('paper_trades migrated: ' + db.paper_trades.countDocuments({}));

// Fix portfolio_history
db.portfolio_history.find().forEach(function(doc) {
  var update = {};
  if (typeof doc.timestamp === 'string') update.timestamp = new Date(doc.timestamp);
  if (typeof doc.created_at === 'string') update.created_at = new Date(doc.created_at);
  if (Object.keys(update).length > 0) {
    db.portfolio_history.updateOne({_id: doc._id}, {$set: update});
  }
});
print('portfolio_history migrated: ' + db.portfolio_history.countDocuments({}));

// Fix ai_signals
db.ai_signals.find().forEach(function(doc) {
  var update = {};
  if (typeof doc.created_at === 'string') update.created_at = new Date(doc.created_at);
  if (typeof doc.timestamp === 'string') update.timestamp = new Date(doc.timestamp);
  if (Object.keys(update).length > 0) {
    db.ai_signals.updateOne({_id: doc._id}, {$set: update});
  }
});
print('ai_signals migrated: ' + db.ai_signals.countDocuments({}));
```

### Option C — Quick Workaround (No migration needed, but weaker)

Change `PaperTradingRecord` `open_time` / `created_at` fields to `String`, parse them manually after loading. This avoids the bson annotation complexity but requires more changes in `load_portfolio_from_storage`. Not recommended for long-term.

---

## Deployment Pipeline Analysis

The deploy script (`deploy-vps.yml`) is **NOT the cause** of data loss:

- Uses `docker compose up -d` (rolling restart) — never runs `down -v`
- MongoDB volume `bot-core_mongodb_data` is a named Docker volume — persists across container restarts
- Script explicitly checks volume existence and warns if missing
- Comment in script: `⚠️ NEVER DELETE VOLUMES!`
- No destructive volume operations found

The pipeline is sound. Only the application-level deserialization bug causes the apparent data loss.

---

## Impact Assessment

| Area | Impact |
|---|---|
| Trade history visibility | HIGH — all trade history disappears from UI on every restart |
| Open positions | HIGH — open positions not tracked after restart (margin not accounted) |
| Portfolio balance | HIGH — balance resets to `initial_balance` (10,000 USDT) because snapshot restore also fails |
| Historical PnL/metrics | HIGH — win rate, total PnL, drawdown all reset |
| Data persistence in MongoDB | NONE — all 17 trades and 24 portfolio snapshots are intact |

---

## Recommended Fix Sequence

1. **Immediate** (5 min): Add `serde(with = ...)` annotations to all `DateTime<Utc>` fields in the four structs in `storage/mod.rs`.
2. **Before restart**: Run MongoDB migration script on VPS to convert existing string timestamps to BSON Dates.
3. **After restart**: Verify engine logs show `✅ Loaded X trades from database` and `🎉 Portfolio restore complete`.
4. **Test**: Trigger a redeploy and confirm trades survive.

---

## Unresolved Questions

- `close_time` in `paper_trades` is already stored as a proper BSON Date (written via `$set` update doc in `update_paper_trade()`). After the serde annotation is added and migration runs, will `close_time` deserialize correctly without needing migration? Likely yes, but verify.
- `PortfolioHistoryRecord.timestamp` — also stored as string currently. After migration and annotation fix, `get_portfolio_history()` will start returning correct balance snapshots. Verify the balance restoration path uses the correct (most recent) snapshot.
- `total_trades` in `PortfolioHistoryRecord` is `u32` but stored as `Long` in BSON. Not a current issue but check if this causes deserialization errors too.
