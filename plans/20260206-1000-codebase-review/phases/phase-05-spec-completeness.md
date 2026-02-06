# Phase 05: Specification System Completeness

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: None (Can run parallel with Phase 01)
**Blocks**: Production deployment, Live trading enablement

---

## Overview

| Field | Value |
|-------|-------|
| Date | 2026-02-06 |
| Priority | P0-CRITICAL |
| Status | Pending |
| Effort | Large (5-7 days) |
| Risk | CRITICAL - Finance project requires complete specs |

---

## Key Insights (From Reports)

**Source**: `reports/260206-spec-validation-report.md`

**Overall Grade**: B+ (Pass with minor gaps)
- **Orphan @spec tags**: 87 (reference non-existent requirements)
- **Missing requirements**: 8 (code exists, no spec)
- **Incomplete requirements**: 10 (marked "In Progress")
- **Broken design doc refs**: 76 (emoji/unicode issue)

**Critical Gap**: FR-REAL-* (Real Trading Module) completely unspecified - 23 @spec tags but no spec files.

---

## Requirements

### CRITICAL-01: Create FR-REAL-TRADING.md Specification
- **Files Affected**: `rust-core-engine/src/real_trading/*.rs` (7 files)
- **Tags Found**: FR-REAL-001 through FR-REAL-053 (23 tags)
- **Issue**: Live trading features have NO formal requirements
- **Impact**: Finance risk - real money without spec review
- **Fix**: Create comprehensive FR-REAL-TRADING.md
- **Ref**: Spec Validation Gap 1

### CRITICAL-02: Document FR-PAPER-003 Execution Logic
- **Files**: `paper_trading/engine.rs` (21 tags)
- **Issue**: Core execution simulation lacks formal spec
- **Impact**: Cannot validate slippage, fees, partial fills
- **Fix**: Add FR-PAPER-003 section to FR-PAPER-TRADING.md
- **Ref**: Spec Validation Gap 2

### CRITICAL-03: Create FR-SETTINGS-001/002 Specifications
- **Files**: `settings.rs`, `api/settings.rs`, `main.py` (26 tags)
- **Issue**: Settings persistence undocumented
- **Fix**: Create FR-SETTINGS.md specification
- **Ref**: Spec Validation Gap 3

### HIGH-04: Document FR-AI-012/013 Requirements
- **Files**: `storage/mod.rs`, `AISignals.tsx` (17 tags)
- **Issue**: AI signal storage/analytics features undocumented
- **Fix**: Add FR-AI-012, FR-AI-013 to FR-AI.md
- **Ref**: Spec Validation Priority 4

### HIGH-05: Fix Traceability Matrix Formatting
- **File**: `specs/TRACEABILITY_MATRIX.md`
- **Issue**: 76 design doc references show emoji instead of filename
- **Fix**: Replace checkmarks with actual file paths
- **Ref**: Spec Validation Issue

### HIGH-06: Add Missing @spec Tags to Code
- **Requirements**: 8 without code tags
- **List**:
  - FR-TRADING-019/020 (Performance, Account info)
  - FR-AI-008/009 (Prediction confidence, Real-time inference)
  - FR-PORTFOLIO-005 (Historical performance)
  - FR-RISK-011 (Emergency controls)
  - FR-TRADING-016 (Execution validation)
  - FR-WEBSOCKET-007 (Performance/Scalability)
- **Fix**: Add @spec tags to implementing code
- **Ref**: Spec Validation Section 2.2

### MEDIUM-07: Create FR-RISK-012 Network Failure Recovery
- **Issue**: Reconnection logic exists but unspecified
- **Fix**: Document network failure handling, order reconciliation
- **Ref**: Spec Validation Gap Analysis 4.2

### MEDIUM-08: Create FR-TRADING-021 Partial Fill Management
- **Issue**: Partial fill logic exists but unspecified
- **Fix**: Document aggregation, average price, cancellation
- **Ref**: Spec Validation Gap Analysis 4.2

### MEDIUM-09: Create FR-DATA-001 Data Integrity Requirements
- **Issue**: Data validation scattered, hard to audit
- **Fix**: Consolidate trade state, reconciliation, audit logging
- **Ref**: Spec Validation Gap Analysis 4.2

### LOW-10: Close "In Progress" Requirements
- **Requirements** (10 total):
  - FR-TRADING-012, FR-TRADING-014
  - FR-AUTH-010
  - FR-DASHBOARD-011
  - US-TRADER-019/020, US-ADMIN-005
- **Fix**: Mark complete or scope down

---

## FR-REAL-TRADING.md Requirements (New)

**14+ requirements needed for real trading module**:

1. **FR-REAL-001**: Real order placement and execution
2. **FR-REAL-010**: Order types and validation
3. **FR-REAL-011**: Position tracking and reconciliation
4. **FR-REAL-012**: Configuration and API key management
5. **FR-REAL-013**: Order lifecycle management
6. **FR-REAL-030**: Margin and leverage for live trading
7. **FR-REAL-033**: Stop-loss execution (live)
8. **FR-REAL-034**: Take-profit execution (live)
9. **FR-REAL-040**: Risk validation for live orders
10. **FR-REAL-041**: Daily loss limits (live)
11. **FR-REAL-042**: Position size limits (live)
12. **FR-REAL-051**: Order status monitoring
13. **FR-REAL-052**: Trade settlement tracking
14. **FR-REAL-053**: Liquidation monitoring

---

## Related Code Files (Orphan Tags)

```
# FR-REAL-* (23 tags - NO SPEC)
rust-core-engine/src/real_trading/mod.rs:1 - FR-REAL-001
rust-core-engine/src/real_trading/order.rs:1 - FR-REAL-010
rust-core-engine/src/real_trading/position.rs:1 - FR-REAL-011
rust-core-engine/src/real_trading/config.rs:1 - FR-REAL-012
rust-core-engine/src/real_trading/engine.rs:1 - FR-REAL-013
rust-core-engine/src/real_trading/engine.rs:1092 - FR-REAL-033
rust-core-engine/src/real_trading/engine.rs:1175 - FR-REAL-034
rust-core-engine/src/real_trading/risk.rs:1-3 - FR-REAL-040/041/042

# FR-PAPER-003 (21 tags)
rust-core-engine/src/paper_trading/engine.rs:75,464,2558,2606,3187
rust-core-engine/src/api/paper_trading.rs:28,455,465,474,594,793,910,920

# FR-SETTINGS-001/002 (26 tags)
rust-core-engine/src/paper_trading/settings.rs:30,35,43,84,697,746
rust-core-engine/src/api/paper_trading.rs:239,255,501,511,600,1390,1391

# FR-AI-012/013 (17 tags)
rust-core-engine/src/storage/mod.rs:654,788
nextjs-ui-dashboard/src/pages/AISignals.tsx:663,671,757,812
```

---

## Implementation Steps

### Step 1: Create FR-REAL-TRADING.md (CRITICAL)

```markdown
# FR-REAL-TRADING: Real Trading Module Requirements

**Status**: Draft
**Version**: 1.0.0
**Last Updated**: 2026-02-06

## FR-REAL-001: Real Order Placement

### Description
Execute actual trades on Binance exchange via API.

### Acceptance Criteria
- [ ] Support market, limit, stop-limit orders
- [ ] Validate order parameters before submission
- [ ] Handle API response with order ID
- [ ] Store order in database immediately
- [ ] Broadcast order status via WebSocket

### Code Location
- `rust-core-engine/src/real_trading/engine.rs`
- `rust-core-engine/src/binance/client.rs:393`

### Test Cases
- TC-REAL-001: Market order execution
- TC-REAL-002: Limit order placement
- TC-REAL-003: Order validation failure
...
```

### Step 2: Add FR-PAPER-003 to FR-PAPER-TRADING.md

```markdown
## FR-PAPER-003: Execution Simulation

### Description
Simulate realistic trade execution with slippage, fees, and latency.

### Acceptance Criteria
- [ ] Slippage: 0.05% - 0.15% based on market volatility
- [ ] Fees: 0.1% maker/taker (configurable)
- [ ] Latency simulation: 10-100ms random delay
- [ ] Partial fill probability based on order size
- [ ] Market impact for large orders (>1% of volume)

### Code Location
- `rust-core-engine/src/paper_trading/engine.rs:738-845` (execution simulation)
- `rust-core-engine/src/paper_trading/engine.rs:1041-1197` (execute_trade)
```

### Step 3: Create FR-SETTINGS.md

```markdown
# FR-SETTINGS: Configuration Persistence

## FR-SETTINGS-001: Settings Storage

### Description
Persist user trading configuration to MongoDB.

### Acceptance Criteria
- [ ] Save settings on change
- [ ] Load settings on startup
- [ ] Validate settings before save
- [ ] Merge partial updates
- [ ] Support per-symbol settings

## FR-SETTINGS-002: Settings Validation

### Description
Validate settings before applying.

### Acceptance Criteria
- [ ] Type validation (numbers, strings, booleans)
- [ ] Range validation (leverage 1-125, risk 0.1-5.0)
- [ ] Dependency validation (stop loss < take profit)
- [ ] Migration for schema changes
```

### Step 4: Fix TRACEABILITY_MATRIX.md

```bash
# Replace checkmarks with actual design doc references
sed -i '' 's/✅/COMP-TRADING.md/g' specs/TRACEABILITY_MATRIX.md
# Manual review needed to correct each entry
```

### Step 5: Add Missing @spec Tags

```rust
// In trading performance code:
// @spec:FR-TRADING-019 - Performance Metrics
// @spec:FR-TRADING-020 - Account Information

// In AI prediction code:
// @spec:FR-AI-008 - Prediction Confidence
// @spec:FR-AI-009 - Real-time Inference
```

---

## Todo List

### Critical (Week 1)
- [ ] Create specs/01-requirements/1.1-functional-requirements/FR-REAL-TRADING.md
- [ ] Document all 14 FR-REAL-* requirements with acceptance criteria
- [ ] Add FR-PAPER-003 section to FR-PAPER-TRADING.md
- [ ] Create FR-SETTINGS.md with FR-SETTINGS-001/002
- [ ] Review FR-REAL-TRADING.md with team (security audit)

### High (Week 2)
- [ ] Add FR-AI-012 to FR-AI.md (AI signal storage)
- [ ] Add FR-AI-013 to FR-AI.md (AI analytics)
- [ ] Fix 76 broken design doc refs in TRACEABILITY_MATRIX.md
- [ ] Add @spec tags to 8 orphan requirements in code
- [ ] Add FR-REAL-* entries to TRACEABILITY_MATRIX.md

### Medium (Week 3)
- [ ] Create FR-RISK-012 (Network failure recovery)
- [ ] Create FR-TRADING-021 (Partial fill management)
- [ ] Create FR-DATA-001 (Data integrity)
- [ ] Close 10 "In Progress" requirements
- [ ] Run validate-specs.py to verify all fixes

---

## Success Criteria

| Criteria | Metric | Target |
|----------|--------|--------|
| Orphan @spec tags | validate-specs.py | <5 |
| Requirements coverage | code with @spec | 95%+ |
| Traceability matrix | broken refs | 0 |
| FR-REAL-* specs | document count | 14+ |
| Validation pass | script output | PASS |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| FR-REAL-* incomplete | Medium | Critical | External audit required |
| Spec drift continues | Medium | High | CI/CD validation gate |
| Team resistance | Low | Medium | Show finance risk |

---

## Security Considerations

- FR-REAL-TRADING.md MUST be reviewed before enabling live trading
- External security audit recommended for real trading specs
- Spec changes require approval process for finance-critical features
- Audit trail of spec changes via git history

---

## Estimated Completion

- **FR-REAL-TRADING.md creation**: 2 days
- **Other spec updates**: 2 days
- **Matrix/tag fixes**: 1 day
- **Review + validation**: 1-2 days

**Total**: 6-7 days

---

## Quality Gates (Proposed)

**Pre-Production Checklist**:
- [ ] All FR-REAL-* requirements documented
- [ ] @spec tag accuracy >= 95%
- [ ] Design doc references fixed (0 broken)
- [ ] All "In Progress" requirements closed
- [ ] External audit for real trading (recommended)
