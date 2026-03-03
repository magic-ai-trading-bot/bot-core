# CI Failure: Test Coverage & Quality Gates

**Run**: 22617404150 | **Date**: 2026-03-03 | **Branch**: main

## Summary

Two jobs failing due to coverage dropping below thresholds after `feat(real-trading): port 5 advanced features` (commit `5f32504`).

## Failed Jobs

| Job | Coverage | Threshold | Delta |
|-----|----------|-----------|-------|
| Rust Library Tests + Coverage | 89.79% | 90% | -0.21% |
| Frontend (Next.js) Tests | 94.60% | 95% | -0.40% |

All other jobs pass (Python 93%+, integrations, security, quality checks).

## Root Cause

Commit `5f32504 feat(real-trading): port 5 advanced features from paper trading` added significant untested code:

**Rust** (`rust-core-engine/`):
- `src/real_trading/engine.rs` +421 lines
- `src/real_trading/config.rs` +167 lines
- `src/real_trading/position.rs` +96 lines
- `src/api/real_trading.rs` +179 lines

**Frontend** (`nextjs-ui-dashboard/`):
- `src/hooks/useRealTrading.ts` +89 lines
- `src/components/trading/AutoTradingPanel.tsx` +178 lines

The new code added ~930 Rust lines and ~267 TS lines without proportional test coverage to compensate. Note: `rust-core-engine/src/real_trading/engine.rs` has 567 inline unit tests but coverage still dropped, meaning the 421 new lines aren't fully exercised by `cargo llvm-cov --lib`.

## Fix Options

**Option A (Recommended) — Lower thresholds temporarily**

Edit `.github/workflows/test-coverage.yml`:
```yaml
env:
  RUST_COVERAGE_THRESHOLD: 89   # was 90, drop by 1
  FRONTEND_COVERAGE_THRESHOLD: 94   # was 95, drop by 1
```

Quick unblock, restore after adding tests.

**Option B — Add targeted tests**

Rust: Add unit tests for the new functions in `src/real_trading/engine.rs` that cover the 421 new lines. Focus on the methods added for: trailing stop, ATR-based stops, Kelly sizing, regime detection, or whichever 5 features were ported.

Frontend: Add tests for `useRealTrading.ts` new code paths and `AutoTradingPanel.tsx` component. Tests already exist at:
- `src/__tests__/hooks/useRealTrading.*.test.ts` (multiple files)
- `src/__tests__/components/trading/AutoTradingPanel.test.tsx`

The existing test files likely need expansion to cover the +89 lines added to `useRealTrading.ts` and +178 lines in `AutoTradingPanel.tsx`.

**Option C — Exclude new files from coverage temporarily**

Add to `vite.config.ts` coverage excludes:
```ts
exclude: [
  ...existing,
  'src/hooks/useRealTrading.ts',  // remove after tests added
]
```

Not recommended — masks the real gap.

## Recommended Action

Use Option A immediately to unblock CI, then add tests for the new real-trading features (Option B) and restore thresholds.

## Unresolved Questions

- Which exact 5 features were ported? (needed to know what to test)
- Were the existing `useRealTrading.*.test.ts` files updated alongside the feature addition?
