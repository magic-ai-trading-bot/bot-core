# Binance Real Trading System - Implementation Plan

**Created**: 2025-12-03 | **Status**: PLANNING | **Priority**: HIGH

## Executive Summary

Extend existing paper trading system to support real trading on Binance (testnet first, mainnet later). Mirror paper trading architecture with dual-mode execution, shared strategy signals, and comprehensive safety mechanisms.

## Architecture Overview

```
Strategy Engine ─────────► TradingMode Enum
       │                        │
       ▼                        ▼
   AI Signals ────┬─────────────┼─────────────┐
                  │             │             │
            ┌─────▼─────┐ ┌─────▼─────┐ ┌─────▼─────┐
            │   Paper   │ │  Testnet  │ │  Mainnet  │
            │  Trading  │ │   Real    │ │   Real    │
            │  Engine   │ │  Trading  │ │  Trading  │
            └─────┬─────┘ └─────┬─────┘ └─────┬─────┘
                  │             │             │
                  ▼             ▼             ▼
            Simulated      Binance       Binance
             Orders       Testnet       Mainnet
```

## Phases

| Phase | Title | Status | Effort | Dependencies |
|-------|-------|--------|--------|--------------|
| 1 | [Binance Order API](./phase-01-binance-order-api.md) | PENDING | 3d | Research |
| 2 | [Real Trading Engine](./phase-02-real-trading-engine.md) | PENDING | 4d | Phase 1 |
| 3 | [Database & API](./phase-03-database-api.md) | PENDING | 2d | Phase 2 |
| 4 | [Frontend UI](./phase-04-frontend-ui.md) | PENDING | 3d | Phase 3 |
| 5 | [Safety & Testing](./phase-05-safety-testing.md) | PENDING | 3d | Phase 4 |

**Total Estimated Effort**: 15 days

## Key Design Decisions

1. **Testnet by default** - Config flag `binance.testnet = true`
2. **Shared risk management** - Same checks as paper trading
3. **Separate storage** - `real_trades`, `real_portfolios` collections
4. **Mode indicator in UI** - Clear visual distinction
5. **Confirmation dialogs** - Required for real orders

## Research Documents

- [Binance REST API](./research/researcher-01-binance-rest-api.md) - Order placement, auth, rate limits
- [WebSocket User Data](./research/researcher-02-binance-websocket-userdata.md) - Order updates, account sync

## Related Code

| Component | Path |
|-----------|------|
| Paper Trading Engine | `rust-core-engine/src/paper_trading/engine.rs` |
| Binance Client | `rust-core-engine/src/binance/client.rs` |
| Config | `rust-core-engine/src/config.rs` |
| Frontend Trading | `nextjs-ui-dashboard/src/pages/TradingPaper.tsx` |

## Success Criteria

- [ ] All order types work on testnet (MARKET, LIMIT, STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT)
- [ ] Real-time order updates via WebSocket user data stream
- [ ] Dual-mode architecture with shared signals
- [ ] Clear UI distinction between paper and real
- [ ] Emergency stop functionality
- [ ] 100% test coverage for order placement

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Accidental mainnet trades | CRITICAL | Testnet by default, env var protection |
| API key exposure | HIGH | Encryption, env vars only |
| Rate limit bans | MEDIUM | Backoff strategy, weight tracking |
| Order sync failure | MEDIUM | WebSocket + REST reconciliation |

---

**Next Step**: Begin Phase 1 - Binance Order API implementation
