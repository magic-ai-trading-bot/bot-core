# Spec Validation Results - Nov 22, 2025

**Run Date**: 2025-11-22
**Script**: `scripts/validate-specs.py`
**Purpose**: Validate spec-code traceability after comprehensive spec update (Phases 1-5)

---

## Summary

| Metric | Value |
|--------|-------|
| **Requirements Found** | 107 |
| **Test Cases Found** | 241 |
| **@spec Tags Found** | 76 tags (57 unique reqs) |
| **Traceability Mappings** | 90 |
| **Checks Passed** | 0 ✗ |
| **Checks Warning** | 2 ⚠️ |
| **Checks Failed** | 3 ✗ |

**Overall Status**: 🔴 **NEEDS IMPROVEMENT** - Critical issues found

---

## Issues Found

### 1. Invalid @spec Tags (21 errors) - CRITICAL ⚠️

**Root Cause**: Old tag naming convention (FR-STRATEGY vs FR-STRATEGIES, FR-PAPER vs FR-TRADING)

**Files affected**:
```
rust-core-engine/src/market_data/cache.rs:10          → FR-MARKET-004
rust-core-engine/src/strategies/rsi_strategy.rs:8     → FR-STRATEGY-001
rust-core-engine/src/strategies/macd_strategy.rs:8    → FR-STRATEGY-002
rust-core-engine/src/strategies/bollinger_strategy.rs:8 → FR-STRATEGY-003
rust-core-engine/src/strategies/volume_strategy.rs:8  → FR-STRATEGY-004
rust-core-engine/src/strategies/strategy_engine.rs:3  → FR-STRATEGY-005
rust-core-engine/src/paper_trading/strategy_optimizer.rs:12 → FR-STRATEGY-007
rust-core-engine/src/paper_trading/portfolio.rs:26    → FR-PAPER-002
rust-core-engine/src/paper_trading/engine.rs:18+      → FR-PAPER-001 (7 locations)
nextjs-ui-dashboard/src/components/dashboard/PerSymbolSettings.tsx:24 → FR-PAPER-002
nextjs-ui-dashboard/src/hooks/usePaperTrading.ts:8    → FR-PAPER-001
```

**Fix Required**: Update tags to use correct FR-XXX naming:
- `FR-STRATEGY-XXX` → `FR-STRATEGIES-XXX`
- `FR-PAPER-XXX` → `FR-TRADING-XXX` (paper trading is part of trading module)
- `FR-MARKET-004` → Verify correct requirement ID

---

### 2. Requirements Missing Code (50 warnings)

**Coverage**: 53.3% (57 out of 107 requirements have code)

**Major gaps**:
- FR-AI-008, FR-AI-009 (AI prediction variants)
- FR-AUTH-010, FR-AUTH-015 (advanced auth features)
- FR-WEBSOCKET-004, 005, 007 (WebSocket features)
- FR-STRATEGIES-004, 006, 007, 009 (strategy variants)
- FR-DASHBOARD-007, 008, 009, 010 (dashboard features)
- FR-TRADING-010, 012, 016, 018 (trading features)
- FR-ASYNC-011, 012 (async backtest tasks) - **JUST ADDED in Phase 5!**

**Note**: Many of these are genuinely not implemented yet OR the code exists but lacks @spec tags.

---

### 3. Missing from Traceability Matrix (17 warnings)

**Requirements in specs but not in matrix**:
```
FR-ASYNC-001 to FR-ASYNC-012 (12 requirements) - JUST ADDED!
FR-RISK-007, FR-RISK-008 (2 requirements) - JUST ADDED!
FR-STRAT-017, FR-STRAT-018 (2 requirements) - JUST ADDED!
FR-STRATEGIES-017, FR-STRATEGIES-018 (duplicates?)
```

**Root Cause**: Agent 1 updated TRACEABILITY_MATRIX.md but used different naming convention (FR-STRAT vs FR-STRATEGIES).

**Fix**: Verify naming convention consistency.

---

### 4. Invalid Test Case References (55 warnings)

**Examples**:
- FR-AI-001 references TC-AI-004 (doesn't exist)
- FR-AUTH-001 references TC-AUTH-001, 002, 003 (don't exist)
- FR-TRADING-001 references TC-TRADING-001, 002, 003 (don't exist)

**Root Cause**: Test case IDs in TRACEABILITY_MATRIX.md don't match actual TC-*.md files.

**Possible causes**:
1. Test case renumbering
2. Test cases not yet created
3. Mismatch between planned and actual test cases

---

### 5. Missing Design Documents (65 warnings)

**Pattern**: Many requirements reference design docs with "✅" or bare filenames not in /specs/02-design/

**Examples**:
```
FR-RISK-007 references "FR-RISK.md" (should be in 01-requirements, not 02-design)
FR-STRAT-001 references "FR-STRATEGIES.md" (should be in 01-requirements)
```

**Root Cause**: TRACEABILITY_MATRIX.md references requirement files as design docs.

**Fix**: Update matrix to only reference actual design docs (COMP-*.md, API-*.md, ARCH-*.md, etc.)

---

## Recommendations

### Phase 6.1: Fix Critical Errors (High Priority)

1. **Update old @spec tags** (21 files):
   ```bash
   # Find all old FR-STRATEGY tags
   grep -r "FR-STRATEGY-" rust-core-engine/src/strategies/
   grep -r "FR-PAPER-" rust-core-engine/src/paper_trading/

   # Replace with correct tags
   # FR-STRATEGY-XXX → FR-STRATEGIES-XXX
   # FR-PAPER-XXX → verify correct module (FR-TRADING-XXX)
   ```

2. **Add missing @spec tags** (priority files):
   ```
   python-ai-service/tasks/backtest_tasks.py (FR-ASYNC-011, 012)
   rust-core-engine/src/strategies/* (FR-STRATEGIES-XXX)
   ```

3. **Fix TRACEABILITY_MATRIX.md naming**:
   - Change FR-STRAT-XXX to FR-STRATEGIES-XXX throughout
   - Remove FR-RISK.md, FR-STRATEGIES.md from "Design Docs" column
   - Only reference actual design docs

### Phase 6.2: Address Warnings (Medium Priority)

4. **Verify test case mappings**:
   - Cross-reference TC-*.md files with TRACEABILITY_MATRIX.md
   - Fix test case ID mismatches
   - Add missing test cases or remove invalid references

5. **Complete design doc references**:
   - Remove "✅" from Design Docs column
   - Add proper references (COMP-*.md, API-*.md, etc.)

### Phase 6.3: Improve Coverage (Low Priority)

6. **Tag remaining code**:
   - Add @spec tags to untagged implemented features
   - Target: 70%+ coverage (currently 53%)

---

## Validation Script Performance

✅ **Script works perfectly!** Found real issues.

**Checks implemented**:
1. ✅ @spec tags reference valid requirements
2. ✅ Requirements have code implementation
3. ✅ Traceability matrix completeness
4. ✅ Test case references validity
5. ✅ Design document existence

**Future improvements**:
- Add `--fix` mode to auto-correct simple issues
- Add code location validation (verify line numbers still valid)
- Check for outdated information (last modified dates)
- Validate cross-references between specs

---

## Next Steps

1. **Run docs-manager agent** to fix TRACEABILITY_MATRIX.md naming issues
2. **Run batch edit** to update old @spec tags (21 files)
3. **Re-run validation** to verify fixes
4. **Proceed to Phase 7** (peer review) once critical errors resolved

**Target**: 0 errors, <10 warnings before Phase 7

---

**Generated**: 2025-11-22
**Author**: Claude Code (Phase 6 - Validation Script)
**Status**: Critical issues identified, fixes planned
