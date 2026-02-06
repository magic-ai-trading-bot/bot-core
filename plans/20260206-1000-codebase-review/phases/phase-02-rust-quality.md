# Phase 02: Rust Code Quality Improvements

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: Phase 01 (Security fixes must complete first)
**Blocks**: Phase 07 (Testing)

---

## Overview

| Field | Value |
|-------|-------|
| Date | 2026-02-06 |
| Priority | P1-HIGH |
| Status | Pending |
| Effort | Large (5-7 days) |
| Risk | HIGH - Trading logic changes require careful testing |

---

## Key Insights (From Reports)

**Source**: `reports/code-reviewer-260206-rust-engine-review.md`

**Overall Grade**: B+ (27 issues found)
- **Critical**: 1 (mutex poisoning)
- **High**: 8 (unwraps, panics, large files, race conditions)
- **Medium**: 12 (error handling, validation, logging)
- **Low**: 6 (naming, docs, code organization)

**Statistics**:
- 133+ `.unwrap()` and `.expect()` calls
- 35 `panic!()` calls (11 in production paths)
- 4 files >3000 lines (god objects)

---

## Requirements

### CRITICAL-01: Fix Mutex Poisoning in WebSocket
- **File**: `rust-core-engine/src/binance/websocket.rs:139`
- **Issue**: `.expect()` on mutex lock can crash WebSocket
- **Fix**: Handle poisoned mutex gracefully with recovery
- **Ref**: Rust Review Issue #1

### HIGH-02: Eliminate Production unwrap() Calls
- **Files**: Multiple (47 files, 133+ instances)
- **Priority Targets**:
  - `storage/mod.rs:359-363, 413-418, 421` - Price parsing
  - `paper_trading/engine.rs` - 10+ instances
  - `binance/websocket.rs` - 40+ instances
- **Fix**: Use `?` operator, `map_err()`, or `unwrap_or_else()` with logging
- **Ref**: Rust Review Issue #2

### HIGH-03: Remove panic!() from Production Paths
- **File**: `rust-core-engine/src/binance/websocket.rs:634, 679, 709`
- **Issue**: Test assertions in production code paths
- **Fix**: Move to `#[cfg(test)]` modules or convert to errors
- **Ref**: Rust Review Issue #3

### HIGH-04: Refactor Large Files (God Objects)
- **Files**:
  - `paper_trading/engine.rs`: 4,842 lines
  - `api/paper_trading.rs`: 3,589 lines
  - `storage/mod.rs`: 3,286 lines
  - `real_trading/engine.rs`: 3,174 lines
- **Fix**: Split into focused modules <1000 lines each
- **Ref**: Rust Review Issue #4

### HIGH-05: Reduce Deep Nesting in process_trading_signal
- **File**: `rust-core-engine/src/paper_trading/engine.rs:579-799`
- **Issue**: 220+ lines with 5 levels of nesting
- **Fix**: Extract to smaller functions, early returns
- **Ref**: Rust Review Issue #5

### HIGH-06: Document Lock Ordering (Race Condition Prevention)
- **File**: `rust-core-engine/src/paper_trading/engine.rs:233-250`
- **Issue**: Multiple background tasks access shared state
- **Fix**: Document lock acquisition order, add timeout guards
- **Ref**: Rust Review Issue #6

### HIGH-07: Add Error Context to Storage Operations
- **File**: `rust-core-engine/src/storage/mod.rs`
- **Lines**: 144, 232, 310-312
- **Issue**: Generic error messages, hard to debug
- **Fix**: Use `with_context()` trait on all DB operations
- **Ref**: Rust Review Issue #7

### MEDIUM-08: Add Input Validation to API Handlers
- **File**: `rust-core-engine/src/api/paper_trading.rs`
- **Issue**: Request structs accept unvalidated input
- **Fix**: Add `validator` crate with field constraints
- **Ref**: Rust Review Issue #10

### MEDIUM-09: Add Rate Limiting to API Routes
- **File**: `rust-core-engine/src/api/paper_trading.rs`, `api/mod.rs`
- **Issue**: No protection against API abuse
- **Fix**: Add `governor` crate for rate limiting middleware
- **Ref**: Rust Review Issue #11

### MEDIUM-10: Move Hardcoded Values to Config
- **Files**:
  - `paper_trading/engine.rs:238` - 1 second interval
  - `paper_trading/engine.rs:280` - 5 second interval
  - `binance/websocket.rs:93` - max reconnect attempts
- **Fix**: Add to PaperTradingSettings or config.toml
- **Ref**: Rust Review Issue #12

---

## Related Code Files

```
rust-core-engine/src/
├── binance/
│   └── websocket.rs             # Mutex fix, panic removal
├── paper_trading/
│   ├── engine.rs                # 4842 lines → split into:
│   │   ├── engine.rs            # Core orchestration (~800 lines)
│   │   ├── risk_manager.rs      # Risk management functions
│   │   ├── trade_executor.rs    # Trade execution logic
│   │   ├── signal_processor.rs  # Signal processing
│   │   └── performance.rs       # Performance tracking
│   └── settings.rs              # Add config values
├── api/
│   ├── paper_trading.rs         # Input validation, rate limiting
│   └── mod.rs                   # Rate limiting middleware
├── storage/
│   └── mod.rs                   # Error context, split modules
└── error.rs                     # Add new error variants
```

---

## Implementation Steps

### Step 1: Fix Critical Mutex Poisoning
```rust
// In binance/websocket.rs:139
// FROM:
let mut cmd_rx = self.command_receiver.lock()
    .expect("Command receiver mutex poisoned")
    .take();

// TO:
let mut cmd_rx = match self.command_receiver.lock() {
    Ok(mut guard) => guard.take(),
    Err(poisoned) => {
        error!("Command receiver mutex poisoned, attempting recovery");
        poisoned.into_inner().take()
    }
};
```

### Step 2: Fix Storage unwrap() Calls
```rust
// In storage/mod.rs:359-363
// FROM:
"open_price": kline.open.parse::<f64>().unwrap_or(0.0),

// TO:
"open_price": kline.open.parse::<f64>().map_err(|e| {
    warn!("Invalid price data for {} at {}: {}", symbol, kline.open_time, e);
    e
}).unwrap_or_else(|_| {
    warn!("Using fallback price 0.0 for {}", symbol);
    0.0
}),
```

### Step 3: Extract Risk Manager from engine.rs
```rust
// Create: paper_trading/risk_manager.rs
pub struct RiskManager {
    settings: Arc<RwLock<PaperTradingSettings>>,
    portfolio: Arc<RwLock<Portfolio>>,
}

impl RiskManager {
    pub async fn check_daily_loss_limit(&self) -> Result<bool> { ... }
    pub async fn is_in_cooldown(&self) -> bool { ... }
    pub async fn check_correlation_limit(&self, direction: &str) -> Result<bool> { ... }
}
```

### Step 4: Add Input Validation
```rust
// Add to Cargo.toml:
// validator = { version = "0.16", features = ["derive"] }

// In api/paper_trading.rs:
use validator::Validate;

#[derive(Debug, Deserialize, Validate)]
pub struct CreateOrderRequest {
    #[validate(length(min = 1, max = 20))]
    pub symbol: String,

    #[validate(range(min = 0.00001, max = 100000.0))]
    pub quantity: f64,

    #[validate(range(min = 1, max = 125))]
    pub leverage: Option<u8>,
}
```

### Step 5: Add Rate Limiting
```rust
// Add to Cargo.toml:
// governor = "0.6"

// In api/mod.rs:
use governor::{Quota, RateLimiter};

let limiter = Arc::new(RateLimiter::direct(
    Quota::per_minute(nonzero!(100u32))
));

// Add middleware to routes
```

---

## Todo List

- [ ] Fix mutex poisoning in websocket.rs:139
- [ ] Fix storage/mod.rs unwrap() calls (lines 359-363, 413-418)
- [ ] Fix paper_trading/engine.rs unwrap() calls (10+ instances)
- [ ] Move panic!() calls to #[cfg(test)] modules
- [ ] Create paper_trading/risk_manager.rs (extract from engine.rs)
- [ ] Create paper_trading/trade_executor.rs (extract from engine.rs)
- [ ] Create paper_trading/signal_processor.rs (extract from engine.rs)
- [ ] Refactor process_trading_signal to <50 lines per function
- [ ] Add lock ordering documentation to code-standards.md
- [ ] Add with_context() to all storage DB operations
- [ ] Add validator crate and validate API requests
- [ ] Add governor rate limiting to API routes
- [ ] Move hardcoded intervals to config
- [ ] Add comprehensive documentation to complex functions
- [ ] Run clippy with `clippy::unwrap_used` warning
- [ ] Run all tests to verify refactoring

---

## Success Criteria

| Criteria | Metric | Target |
|----------|--------|--------|
| unwrap() in production | grep count | <10 |
| panic!() in production | grep count | 0 |
| Max file size | line count | <1500 lines |
| Max function size | line count | <60 lines |
| Max nesting depth | visual inspection | 3 levels |
| Clippy warnings | clippy output | 0 |
| Test pass rate | cargo test | 100% |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Refactoring breaks trading | Medium | Critical | Extensive testing, staged rollout |
| Race condition introduced | Low | High | Lock ordering, timeout guards |
| Performance regression | Low | Medium | Benchmark before/after |

---

## Security Considerations

- Rate limiting prevents DoS attacks
- Input validation prevents injection attacks
- Error context must not leak sensitive data
- Lock timeouts prevent denial of service via lock holding

---

## Estimated Completion

- **Mutex fix + unwrap fixes**: 1 day
- **File splitting (engine.rs)**: 2 days
- **Function refactoring**: 1 day
- **Validation + rate limiting**: 1 day
- **Testing + documentation**: 1-2 days

**Total**: 6-7 days
