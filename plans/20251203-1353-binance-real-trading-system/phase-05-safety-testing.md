# Phase 5: Safety & Testing

## Context Links
- [Main Plan](./plan.md)
- [Phase 4: Frontend UI](./phase-04-frontend-ui.md)
- [Testing Guide](../../docs/TESTING_GUIDE.md)

## Overview

| Field | Value |
|-------|-------|
| Priority | CRITICAL |
| Status | PENDING |
| Effort | 3 days |
| Dependencies | Phases 1-4 complete |

Implement comprehensive safety mechanisms and testing strategy for real trading. This is a finance project - mistakes mean money loss.

## Key Insights

1. **Existing test patterns**: Rust (cargo test), Python (pytest), Frontend (vitest)
2. **Paper trading has risk checks**: Daily loss limit, consecutive loss cool-down, correlation limit
3. **Binance testnet**: Full API simulation, no real money
4. **Rate limits**: Must avoid 418 IP ban
5. **Order validation**: Size limits, price bands

## Requirements

### Safety Requirements
- SR-001: Testnet by default - mainnet requires explicit env var
- SR-002: Max order size limit - configurable, default $1000
- SR-003: Max daily loss limit - stop trading if exceeded
- SR-004: Emergency stop - cancel all orders, disable trading
- SR-005: API key encryption - never log or expose
- SR-006: Rate limit tracking - abort before ban threshold
- SR-007: Order validation - reject invalid orders before API call
- SR-008: Position limit - max positions per symbol and total
- SR-009: Startup self-test - verify API connectivity before trading

### Testing Requirements
- TR-001: Unit tests for all order types
- TR-002: Integration tests with testnet
- TR-003: Mock tests for rate limit handling
- TR-004: Frontend component tests
- TR-005: E2E test for order flow
- TR-006: Chaos testing - network failures, API errors

## Architecture

### Safety Layers

```
Layer 1: Configuration Safety
├── Testnet by default (config.toml)
├── ALLOW_MAINNET_TRADING env var required
└── Max order size in config

Layer 2: Pre-Order Validation
├── Order size check
├── Balance sufficiency check
├── Position limit check
└── Price sanity check (within bands)

Layer 3: Runtime Protection
├── Rate limit tracking
├── Daily loss limit enforcement
├── Consecutive loss cool-down
└── Emergency stop capability

Layer 4: Post-Order Verification
├── Order confirmation via WebSocket
├── Position sync every 5 minutes
└── Balance reconciliation

Layer 5: Monitoring & Alerts
├── Error logging with context
├── Alert on unexpected failures
└── Audit trail for all orders
```

### Safety Manager

```rust
pub struct SafetyManager {
    config: SafetyConfig,
    daily_loss: AtomicF64,
    daily_trades: AtomicU32,
    rate_limit_weight: AtomicU32,
    last_weight_reset: RwLock<DateTime<Utc>>,
    trading_enabled: AtomicBool,
}

impl SafetyManager {
    pub fn check_order_allowed(&self, order: &OrderRequest) -> Result<(), SafetyError>;
    pub fn record_order(&self, weight: u32) -> Result<(), SafetyError>;
    pub fn record_loss(&self, amount: f64);
    pub fn record_trade(&self);
    pub fn emergency_stop(&self);
    pub fn enable_trading(&self) -> Result<(), SafetyError>;
    pub fn is_trading_enabled(&self) -> bool;
    pub fn get_safety_status(&self) -> SafetyStatus;
}

pub struct SafetyConfig {
    pub max_order_size_usd: f64,        // Default: 1000
    pub max_daily_loss_usd: f64,        // Default: 500
    pub max_daily_loss_pct: f64,        // Default: 5%
    pub max_daily_trades: u32,          // Default: 100
    pub max_positions_per_symbol: u32,  // Default: 1
    pub max_total_positions: u32,       // Default: 5
    pub rate_limit_threshold: u32,      // Default: 5000 (out of 6000)
    pub price_deviation_max_pct: f64,   // Default: 2% from mark
}

pub enum SafetyError {
    OrderTooLarge { size: f64, max: f64 },
    DailyLossExceeded { loss: f64, max: f64 },
    DailyTradesExceeded { count: u32, max: u32 },
    PositionLimitReached { current: u32, max: u32 },
    RateLimitApproaching { weight: u32, threshold: u32 },
    PriceDeviationTooHigh { deviation: f64, max: f64 },
    InsufficientBalance { required: f64, available: f64 },
    TradingDisabled,
    MainnetNotAllowed,
}
```

## Related Code Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/safety/mod.rs` | CREATE | Safety module |
| `rust-core-engine/src/safety/manager.rs` | CREATE | SafetyManager |
| `rust-core-engine/src/safety/config.rs` | CREATE | SafetyConfig |
| `rust-core-engine/src/safety/validator.rs` | CREATE | Order validation |
| `rust-core-engine/tests/test_real_trading.rs` | CREATE | Integration tests |
| `rust-core-engine/tests/test_safety.rs` | CREATE | Safety tests |
| `nextjs-ui-dashboard/src/components/trading/__tests__/` | CREATE | Component tests |

## Implementation Steps

### Step 1: Create SafetyConfig (Day 1)

1. Create `safety/config.rs`
2. Define SafetyConfig struct with defaults
3. Load from config.toml with validation
4. Add env var overrides

### Step 2: Create SafetyManager (Day 1)

1. Create `safety/manager.rs`
2. Implement atomic counters for tracking
3. Implement `check_order_allowed()`:
   - Check order size
   - Check daily loss
   - Check position limits
   - Check rate limit
4. Implement `record_order()`, `record_loss()`, `record_trade()`
5. Implement `emergency_stop()` and `enable_trading()`

### Step 3: Create Order Validator (Day 1)

1. Create `safety/validator.rs`
2. Implement pre-order checks:
   ```rust
   pub fn validate_order(order: &OrderRequest, portfolio: &Portfolio) -> Result<(), SafetyError>;
   ```
3. Check balance sufficiency
4. Check price sanity (compare to mark price)
5. Check symbol is allowed

### Step 4: Integrate Safety into Engine (Day 1-2)

1. Inject SafetyManager into RealTradingEngine
2. Call `check_order_allowed()` before every order
3. Call `record_order()` after order placement
4. Call `record_loss()` when trade closes at loss
5. Handle SafetyError and log with context

### Step 5: Add Startup Self-Test (Day 2)

1. Create `safety/self_test.rs`
2. On startup, verify:
   - API connectivity (ping endpoint)
   - Account access (get account info)
   - WebSocket connectivity (test listen key)
   - Time sync (server time within 1 second)
3. If any check fails, disable trading

### Step 6: Unit Tests (Day 2)

1. Test SafetyManager limits:
   - Order size check
   - Daily loss accumulation
   - Position limit enforcement
   - Rate limit tracking
2. Test SafetyError messages
3. Test config loading and validation

### Step 7: Integration Tests with Testnet (Day 2-3)

1. Create `tests/test_real_trading.rs`
2. Test full order flow:
   - Place market order
   - Verify fill via WebSocket
   - Place limit order
   - Cancel limit order
   - Place SL/TP orders
3. Test error handling:
   - Insufficient balance
   - Invalid symbol
   - Price too far from mark

### Step 8: Chaos Testing (Day 3)

1. Test network disconnect during order
2. Test WebSocket reconnection
3. Test rate limit backoff
4. Test emergency stop during active orders

### Step 9: Frontend Tests (Day 3)

1. Test TradingModeSelector component
2. Test EmergencyStopButton confirmation
3. Test OrderConfirmationDialog
4. Test mode indicator rendering

### Step 10: Documentation (Day 3)

1. Document safety configuration
2. Document emergency procedures
3. Create testnet-to-mainnet checklist

## Todo List

- [ ] Create safety/config.rs with SafetyConfig
- [ ] Create safety/manager.rs with SafetyManager
- [ ] Implement check_order_allowed() with all checks
- [ ] Implement rate limit tracking with weight
- [ ] Create safety/validator.rs for pre-order validation
- [ ] Create safety/self_test.rs for startup checks
- [ ] Integrate SafetyManager into RealTradingEngine
- [ ] Write unit tests for SafetyManager
- [ ] Write unit tests for order validation
- [ ] Write integration tests with testnet
- [ ] Write chaos tests for error scenarios
- [ ] Write frontend component tests
- [ ] Document safety procedures
- [ ] Create mainnet rollout checklist

## Success Criteria

1. All safety checks prevent invalid orders
2. Daily loss limit stops trading automatically
3. Emergency stop cancels all orders in <5 seconds
4. Rate limit never exceeded (no 418 bans)
5. All tests pass on testnet
6. Startup self-test catches API issues
7. 100% code coverage for safety module

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Safety bypass | CRITICAL | Very Low | Code review, no bypass path |
| False positive blocks | Medium | Low | Tunable thresholds, alerts |
| Rate limit ban | High | Low | Conservative threshold (5000/6000) |
| Test flakiness | Low | Medium | Retry logic, isolated tests |

## Security Considerations

1. **No mainnet without env var** - Compile-time check
2. **API key never logged** - Masked in all output
3. **Audit trail** - All orders logged with context
4. **Alert on anomalies** - Unexpected large losses

## Rollout Plan

### Phase A: Testnet Only (Week 1-2)
1. Deploy all features with mainnet disabled
2. Run with paper trading signals but real testnet execution
3. Verify order flow, fills, cancellations
4. Verify risk management stops trading at limits

### Phase B: Testnet Extended (Week 3)
1. Run continuously for 7 days
2. Monitor for any issues
3. Verify daily loss resets correctly
4. Verify position limits enforced

### Phase C: Mainnet Small (Week 4)
1. Enable mainnet with $100 max order size
2. $50 max daily loss
3. Manual monitoring first 24 hours
4. Gradually increase limits

### Phase D: Mainnet Normal (Week 5+)
1. Increase to $1000 max order size
2. $500 max daily loss
3. Automated monitoring with alerts
4. Weekly review of performance

## Mainnet Rollout Checklist

Before enabling mainnet:

- [ ] All testnet tests pass for 7 days
- [ ] No safety check bypasses in code
- [ ] API keys stored securely (env vars only)
- [ ] Emergency stop tested and working
- [ ] Rate limit tracking verified
- [ ] Daily loss limit tested
- [ ] Position limits tested
- [ ] WebSocket reconnection tested
- [ ] Startup self-test passing
- [ ] Monitoring and alerts configured
- [ ] Rollback procedure documented
- [ ] Team notified of go-live
- [ ] `ALLOW_MAINNET_TRADING=true` set

## Next Steps

After Phase 5 complete:
- Testnet deployment and monitoring
- Gather metrics for 1-2 weeks
- Security review
- Gradual mainnet rollout
