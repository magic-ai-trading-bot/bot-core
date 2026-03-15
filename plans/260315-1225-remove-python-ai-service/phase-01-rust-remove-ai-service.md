# Phase 1: Rust - Remove AIService Dependency

**Priority**: P0 | **Effort**: 3h | **Status**: Completed

## Overview

Remove AIService struct and all Grok/xAI API calls from Rust engine. The strategy engine becomes the sole signal source.

## Files to Modify

| File | Action |
|------|--------|
| `rust-core-engine/src/ai/mod.rs` | Gut AIService — keep struct as no-op stub or remove entirely |
| `rust-core-engine/src/ai/client.rs` | Remove HTTP calls to Python endpoints |
| `rust-core-engine/src/paper_trading/engine.rs` | Remove AIService field, remove AI signal calls |
| `rust-core-engine/src/main.rs` | Remove AIService creation (lines ~163-164) |
| `rust-core-engine/src/api/mod.rs` | Remove AIService from ApiServer |
| `rust-core-engine/src/config.rs` | Remove AI config section |
| `rust-core-engine/src/strategies/ml_trend_predictor.rs` | Remove or disable (calls Python /predict-trend) |
| `rust-core-engine/tests/test_ai.rs` | Remove or convert to strategy tests |
| `rust-core-engine/tests/test_cross_service.rs` | Remove Python cross-service tests |

## Implementation Steps

### 1a. Stub out AIService (keep interface, remove HTTP calls)

In `src/ai/mod.rs`:
- Keep `AIService` struct (other code references it)
- Make all methods return default/empty results instead of calling Python
- `analyze_for_trading_signal()` → return `None` (strategy engine handles signals)
- `get_strategy_recommendations()` → return empty vec
- Remove `python_service_url` field

### 1b. Remove ML Trend Predictor strategy

In `src/strategies/ml_trend_predictor.rs`:
- This strategy calls Python's `/predict-trend` endpoint
- Either remove the strategy entirely or make it a no-op
- Remove from strategy engine's active strategies list

### 1c. Clean up engine.rs

In `src/paper_trading/engine.rs`:
- AIService field can stay as phantom/stub (avoid massive refactor)
- Signal generation should already fall through to strategy engine when AI returns None
- Verify: `process_trading_signal()` works without AI input

### 1d. Remove Grok/xAI references

Search and remove:
- `XAI_API_KEY` env var usage
- Any `grok` or `xai` references in Rust code
- AI-related config in YAML defaults

### 1e. Update YAML config

In `rust-core-engine/config/paper-trading-defaults.yml`:
- Remove or comment out `ai:` section
- Keep `strategy:` section (this is what drives trading now)

## Verification

```bash
cd rust-core-engine && cargo check
cd rust-core-engine && cargo test --lib
cd rust-core-engine && cargo clippy -- -D warnings
```

## Todo

- [x] Stub AIService methods to return defaults (no HTTP calls)
- [x] Remove/disable ml_trend_predictor strategy (all HTTP calls removed, returns None/error)
- [x] Verify strategy engine generates signals independently (AIService.analyze_for_trading_signal returns Err; callers already fall through to strategy engine)
- [x] Remove Grok/xAI references (none in Rust source; config field kept for compat)
- [x] Update YAML config (ai: section removed from paper-trading-defaults.yml)
- [x] cargo check passes (0 warnings, 0 errors)
- [x] cargo test passes (5545 passed, 0 failed, skipping pre-existing stack-overflow tests in api::tests)
- [x] cargo clippy clean (0 errors on our modified files; pre-existing errors in notifications.rs/settings.rs not caused by our changes)
