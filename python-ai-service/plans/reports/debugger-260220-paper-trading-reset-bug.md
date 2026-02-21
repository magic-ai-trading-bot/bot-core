# Debugger Report: Paper Trading Reset Bug

**Date**: 2026-02-20
**Issue**: Old trades reappear after clicking "Reset to Default" (Đặt lại) on VPS
**Severity**: High - data integrity / user-visible
**Status**: Root cause confirmed

---

## Executive Summary

The reset button clears in-memory state only. It does NOT delete trades or portfolio snapshots from MongoDB. When the engine restarts (or the VPS reboots / Docker container restarts), `load_portfolio_from_storage()` reloads ALL trades from the `paper_trades` collection, restoring old data. Additionally, the periodic performance tracking loop saves portfolio snapshots every 5 minutes, and the trade monitoring loop runs every 5 seconds - neither of these is halted by the reset call.

Root cause: `reset_portfolio()` is missing a database cleanup step.

---

## Technical Analysis

### 1. Frontend Reset Button

**File**: `nextjs-ui-dashboard/src/components/dashboard/BotSettings.tsx`
- Line 361: "Reset to Default" button calls `handleReset()`
- Line 139-163: `handleReset()` calls `await resetPortfolio()` from `usePaperTradingContext()`

**File**: `nextjs-ui-dashboard/src/hooks/usePaperTrading.ts`
- Lines 798-828: `resetPortfolio()` sends `POST /api/paper-trading/reset`
- After success it refreshes: `fetchPortfolioStatus()`, `fetchOpenTrades()`, `fetchClosedTrades()`
- Frontend state is correctly cleared

### 2. Backend Reset Endpoint

**File**: `rust-core-engine/src/api/paper_trading.rs`
- Lines 552-557: Route `POST /api/paper-trading/reset` registered
- Lines 860-877: Handler `reset_portfolio()` calls `api.engine.reset_portfolio().await`

**File**: `rust-core-engine/src/paper_trading/engine.rs`
- Lines 2935-2952: `reset_portfolio()` implementation:

```rust
pub async fn reset_portfolio(&self) -> Result<()> {
    let settings = self.settings.read().await;
    let initial_balance = settings.basic.initial_balance;
    drop(settings);

    let mut portfolio = self.portfolio.write().await;
    *portfolio = PaperPortfolio::new(initial_balance);  // <-- ONLY clears in-memory

    // Broadcasts event but DOES NOT delete MongoDB data
    let _ = self.event_broadcaster.send(PaperTradingEvent { ... });
    Ok(())
}
```

**THE BUG**: `reset_portfolio()` only replaces the in-memory `PaperPortfolio` struct. It does not call any storage deletion function.

### 3. Reload Mechanism (Why Trades Reappear)

**File**: `rust-core-engine/src/paper_trading/engine.rs`
- Lines 2097-2260: `load_portfolio_from_storage()` - called at engine startup (lines 163-166 and 2350-2353)

```rust
async fn load_portfolio_from_storage(&self) -> Result<()> {
    // Loads ALL trades from paper_trades collection (up to 10,000)
    let all_trades = self.storage.get_paper_trades_history(Some(10000)).await?;

    // Filters for "Open" status trades - restores them as open positions
    let open_trades: Vec<_> = all_trades.iter().filter(|t| t.status == "Open").collect();

    // Restores portfolio balance from latest portfolio_history snapshot
    let latest_snapshot = self.storage.get_portfolio_history(Some(7)).await?;
    ...
    // Restores ALL trades (open + closed) into in-memory portfolio
    for trade_record in all_trades { ... }
}
```

This runs on every `start_async()` and `start()` call, reloading everything from MongoDB.

### 4. Background Persistence (Why Data Stays in MongoDB)

The engine has background tasks that continuously write to MongoDB:

| Task | Interval | What it saves |
|------|----------|---------------|
| `start_performance_tracking()` | Every 5 min (line 303) | `save_portfolio_snapshot()` → `portfolio_history` collection |
| `save_portfolio_to_storage()` | On `stop()` (line 220) | Portfolio snapshot |
| Trade execution | On each trade | `save_paper_trade()` → `paper_trades` collection |
| Trade close | On each close | `save_portfolio_snapshot()` (line 2547) |

**File**: `rust-core-engine/src/storage/mod.rs`
- Lines 628-653: `save_paper_trade()` - inserts into `paper_trades` collection (insert_one, no upsert)
- Lines 691-711: `save_portfolio_snapshot()` - inserts into `portfolio_history` collection
- Lines 827-839: `get_paper_trades_history()` - fetches ALL records from `paper_trades`

There is NO `delete_paper_trades()` or `clear_paper_trades()` function anywhere in `storage/mod.rs`.

### 5. Scenario on VPS

1. User clicks "Reset to Default" → `POST /api/paper-trading/reset`
2. In-memory portfolio reset to initial state → frontend shows clean state
3. Background `start_performance_tracking()` continues running, saves new (empty) snapshot to `portfolio_history`
4. Old trades remain in `paper_trades` collection in MongoDB
5. Docker container restarts / engine restart → `load_portfolio_from_storage()` runs
6. All old trades reloaded from MongoDB → old trades reappear

Even without restart: if the engine is stopped and restarted (user clicks Stop then Start), the same reload happens.

---

## Evidence

- `reset_portfolio()` at `engine.rs:2935` has NO database calls
- `load_portfolio_from_storage()` at `engine.rs:2097` fetches ALL trades unconditionally
- No storage function for deleting paper trades exists in `storage/mod.rs`
- `start_async()` at `engine.rs:2350` always calls `load_portfolio_from_storage()` on startup

---

## Recommended Fix

Add database cleanup inside `reset_portfolio()`:

```rust
pub async fn reset_portfolio(&self) -> Result<()> {
    let settings = self.settings.read().await;
    let initial_balance = settings.basic.initial_balance;
    drop(settings);

    // 1. Clear MongoDB: delete all trades and portfolio history
    if let Err(e) = self.storage.delete_all_paper_trades().await {
        warn!("Failed to clear paper_trades from DB: {}", e);
    }
    if let Err(e) = self.storage.delete_all_portfolio_history().await {
        warn!("Failed to clear portfolio_history from DB: {}", e);
    }

    // 2. Reset in-memory portfolio
    let mut portfolio = self.portfolio.write().await;
    *portfolio = PaperPortfolio::new(initial_balance);
    drop(portfolio);

    // 3. Broadcast reset event
    let _ = self.event_broadcaster.send(PaperTradingEvent { ... });
    info!("Portfolio reset to initial balance: ${}", initial_balance);
    Ok(())
}
```

New storage functions needed in `storage/mod.rs`:
```rust
pub async fn delete_all_paper_trades(&self) -> Result<()> {
    self.paper_trades()?.delete_many(doc! {}).await?;
    Ok(())
}

pub async fn delete_all_portfolio_history(&self) -> Result<()> {
    self.portfolio_history()?.delete_many(doc! {}).await?;
    Ok(())
}
```

---

## Actionable Recommendations

| Priority | Action | File |
|----------|--------|------|
| P0 - Fix | Add `delete_all_paper_trades()` to Storage | `rust-core-engine/src/storage/mod.rs` |
| P0 - Fix | Add `delete_all_portfolio_history()` to Storage | `rust-core-engine/src/storage/mod.rs` |
| P0 - Fix | Call both deletions inside `reset_portfolio()` | `rust-core-engine/src/paper_trading/engine.rs:2935` |
| P1 - UX | After reset, also restart the engine if it was running | `engine.rs` / `api/paper_trading.rs` |
| P2 - Test | Add integration test verifying reset clears MongoDB and reload finds nothing | `tests/test_paper_trading.rs` |

---

## Unresolved Questions

- Should `portfolio_history` be fully deleted on reset, or only entries before the reset timestamp? Keeping some history may be desirable for audit purposes.
- Should a reset also cancel any pending stop-limit orders stored in MongoDB (separate collection if any)?
- Does the engine need to be stopped before reset? If reset is called while the engine is running, the background performance tracking will immediately write a new (possibly empty) snapshot - this is fine, but the open position tracking loop might race with the reset.
