# Real Trading System Implementation Plan

**Created**: 2026-02-06 | **Status**: Planning | **Estimated Effort**: 5-7 days

## Overview

Implement full-featured Real Trading System with Binance Testnet supporting Spot + Futures, hybrid trading mode (Auto AI + Manual), comprehensive order types, and reusable risk management.

## Phases

| Phase | Name | Status | Progress | Est. Time |
|-------|------|--------|----------|-----------|
| 01 | [Core Spot Trading](./phase-01-core-spot-trading.md) | Pending | 0% | 1 day |
| 02 | [Advanced Order Types](./phase-02-advanced-order-types.md) | Pending | 0% | 1 day |
| 03 | [Futures Trading](./phase-03-futures-trading.md) | Pending | 0% | 1.5 days |
| 04 | [Manual Trading API](./phase-04-manual-trading-api.md) | Pending | 0% | 1 day |
| 05 | [Frontend Integration](./phase-05-frontend-integration.md) | Pending | 0% | 1.5 days |
| 06 | [Testing & Integration](./phase-06-testing-integration.md) | Pending | 0% | 1 day |

## Key Decisions

1. **Testnet First**: All development on testnet; mainnet requires explicit config change
2. **Safety**: 2-step confirmation for orders, circuit breaker, daily loss limits
3. **Reuse**: Leverage existing paper trading risk management patterns
4. **Hybrid Mode**: Support both AI signals + manual trading in parallel

## Existing Code Leverage

- `rust-core-engine/src/real_trading/` - 5,379 lines (engine, risk, config, order, position)
- `rust-core-engine/src/binance/` - HTTP client with testnet, WebSocket, user data stream
- `rust-core-engine/src/paper_trading/` - Risk patterns to port (trailing stop, daily limits)
- `nextjs-ui-dashboard/src/hooks/useRealTrading.ts` - Frontend hook (needs endpoints)
- `nextjs-ui-dashboard/src/pages/RealTrading.tsx` - UI page (needs order form)

## Success Criteria

- [ ] Place/cancel Spot market + limit orders on testnet
- [ ] Place/cancel Futures orders with leverage up to 10x
- [ ] OCO and trailing stop orders working
- [ ] Manual trading UI with order form + confirmation
- [ ] Real-time position updates via WebSocket
- [ ] Order history with P&L tracking
- [ ] Risk management blocking trades when limits hit
- [ ] All tests passing (unit + integration)

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Testnet API differences | Medium | Test all endpoints early |
| Rate limiting | Low | Existing rate limiter in client |
| WebSocket disconnects | Medium | Auto-reconnect + reconciliation loop |
| Order execution failures | High | Retry logic + circuit breaker |

## Related Specs

- `specs/01-requirements/1.1-functional-requirements/FR-TRADING.md`
- `specs/02-design/2.5-components/COMP-RUST-TRADING.md`
- `specs/02-design/2.3-api/API-RUST-CORE.md`
