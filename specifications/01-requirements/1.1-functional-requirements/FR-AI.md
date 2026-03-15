# AI Signal Generation - Functional Requirements

**Spec ID**: FR-AI
**Version**: 2.0
**Status**: Updated
**Owner**: Core Engine Team
**Last Updated**: 2026-03-15

---

## Tasks Checklist

- [x] Requirements gathered
- [x] Design completed
- [x] Implementation done
- [x] Tests written
- [x] Documentation updated
- [ ] Code reviewed
- [x] Deployed to staging
- [x] Deployed to production

---

## Metadata

**Related Specs**:
- Related FR: [FR-CORE-001](../FR-CORE.md) - Trading Engine Integration
- Related Design: [ARCH-AI-001](../../02-design/ARCHITECTURE.md) - Strategy Architecture
- Related FR: [FR-RISK.md](FR-RISK.md) - Risk Management Integration

**Dependencies**:
- Depends on: Binance Market Data (WebSocket), MongoDB
- Blocks: FR-CORE-002 (Strategy Optimization), FR-UI-003 (Signal Display)

**Business Value**: High
**Technical Complexity**: Medium
**Priority**: Critical

---

## Overview

Signal generation is handled entirely by the Rust core engine using technical analysis strategies. No external AI/ML service exists. All signals are derived from indicator-based strategies with configurable confidence thresholds.

---

## Business Context

**Problem Statement**:
Trading requires reliable, low-latency signal generation from technical indicators. Strategy-based signals provide deterministic, auditable, and fast signal generation without dependency on external APIs.

**Business Goals**:
- Generate trading signals with >= 65% win rate
- Signal latency < 100ms from market data to signal
- Configurable confidence thresholds via self-tuning engine
- Support multiple timeframes (15m, 30m, 1h, 4h)

**Success Metrics**:
- Combined win rate: >= 65%
- Signal latency: < 100ms
- Sharpe ratio: >= 1.5
- System uptime: >= 99.5%

---

## Functional Requirements

### FR-AI-001: Technical Indicator Calculation

**Priority**: Critical
**Status**: Completed
**Code Tags**: `@spec:FR-AI-001`

**Description**:
Calculate technical indicators from OHLCV market data for signal generation.

**Indicators**:
- RSI (14-period) — overbought/oversold detection
- MACD — trend and momentum
- Bollinger Bands — volatility and breakout
- EMA (9, 21, 50, 200) — trend direction
- ADX — trend strength
- Stochastic — momentum confirmation
- ATR — volatility for position sizing
- OBV — volume confirmation

**Code Location**: `rust-core-engine/src/strategies/indicators.rs`

---

### FR-AI-002: Multi-Strategy Signal Generation

**Priority**: Critical
**Status**: Completed
**Code Tags**: `@spec:FR-AI-002`

**Description**:
Each strategy module produces a directional signal (Long/Short/Neutral) with a confidence score. The strategy engine aggregates signals from all active strategies.

**Strategies**:
- RSI Strategy (`rsi.rs`)
- MACD Strategy (`macd.rs`)
- Bollinger Band Strategy (`bollinger.rs`)
- Volume Strategy (`volume.rs`)

**Code Location**: `rust-core-engine/src/strategies/`

---

### FR-AI-003: Signal Confidence Scoring

**Priority**: Critical
**Status**: Completed
**Code Tags**: `@spec:FR-AI-003`

**Description**:
Aggregate individual strategy signals into a combined confidence score (0.0–1.0). Signal is acted upon only if confidence exceeds the configurable threshold.

**Algorithm**:
1. Collect signals from all active strategies
2. Weight signals by strategy performance history
3. Calculate weighted average confidence
4. Filter by minimum threshold (default: 0.6, GREEN-tier tunable)

**Code Location**: `rust-core-engine/src/strategies/strategy_engine.rs`

---

### FR-AI-004: Multi-Timeframe Analysis

**Priority**: High
**Status**: Completed
**Code Tags**: `@spec:FR-AI-004`

**Description**:
Analyze signals across multiple timeframes (15m, 30m, 1h, 4h) to confirm signal direction before execution.

**Requirement**: Signal must align across at least `min_required_timeframes` timeframes (configurable, GREEN-tier tunable).

---

### FR-AI-005: Signal Rate Limiting

**Priority**: High
**Status**: Completed
**Code Tags**: `@spec:FR-AI-005`

**Description**:
Enforce minimum interval between signals per symbol to prevent overtrading.

**Configuration**: `signal_interval_minutes` (GREEN-tier tunable, default: 15 minutes)

---

## Non-Functional Requirements

| Requirement | Target |
|-------------|--------|
| Signal latency | < 100ms |
| Indicator calculation memory | < 50MB per symbol |
| Concurrent symbols | >= 20 |
| Uptime | >= 99.5% |

---

## API Endpoints

All signal endpoints are on the Rust API (port 8080):

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/strategies/active` | List active strategies |
| GET | `/api/strategies/signals/:symbol` | Latest signals for symbol |
| POST | `/api/strategies/backtest` | Run strategy backtest |

---

## Self-Tuning Integration

Signal pipeline parameters are tunable via the MCP self-tuning engine (GREEN tier — auto-adjust):
- `confidence_threshold`
- `rsi_oversold` / `rsi_overbought`
- `signal_interval_minutes`
- `min_required_indicators`
- `min_required_timeframes`
- `sp_*` signal pipeline params (13 params)

See `mcp-server/src/tuning/bounds.ts` for full parameter bounds.

---

## Tests

- `rust-core-engine/tests/test_strategies.rs`
- `rust-core-engine/tests/test_all_5_strategies_live.rs`
- `rust-core-engine/tests/test_indicators_comprehensive.rs`
