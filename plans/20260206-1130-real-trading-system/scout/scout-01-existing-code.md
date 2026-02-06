# Scout Report: Existing Code Analysis

## Summary
Comprehensive analysis of existing code that can be reused for Real Trading System.

## Rust Core Engine

### Real Trading Module (EXISTING - 5,379 lines)
Location: `rust-core-engine/src/real_trading/`

| File | Lines | Description |
|------|-------|-------------|
| engine.rs | 3,210 | Main trading engine, order execution, event handling |
| risk.rs | 874 | Risk management, validation, limits |
| position.rs | 500 | Position tracking and management |
| order.rs | 372 | Order types, states, fills |
| config.rs | 356 | Configuration (testnet support) |
| mod.rs | 67 | Module exports |

**Key Features Already Implemented:**
- RealTradingEngine with testnet support
- Market order execution
- Position tracking with fills
- Risk management (pre-trade checks)
- Circuit breaker for cascade failures
- Event broadcasting (order fills, position updates)

### Paper Trading Module (REUSABLE - 14,758 lines)
Location: `rust-core-engine/src/paper_trading/`

**Reusable Components:**
- Daily loss limit logic
- Cool-down mechanism
- Position correlation limits
- Trailing stop implementation
- Strategy execution

### Binance Module (EXISTING)
Location: `rust-core-engine/src/binance/`

| File | Description |
|------|-------------|
| client.rs | HTTP client with testnet config |
| websocket.rs | WebSocket for market data |
| user_data_stream.rs | User data stream (orders, positions) |
| types.rs | Order types, sides, symbols |

**Testnet URLs Already Configured:**
- Spot: `https://testnet.binance.vision`
- Futures: `https://testnet.binancefuture.com`
- WebSocket: `wss://testnet.binance.vision/ws`

## Frontend Dashboard

### Pages (EXISTING)
Location: `nextjs-ui-dashboard/src/pages/`

| Page | Status | Description |
|------|--------|-------------|
| RealTrading.tsx | ✅ EXISTS | Real trading terminal |
| PaperTrading.tsx | ✅ EXISTS | Paper trading page |
| Portfolio.tsx | ✅ EXISTS | Portfolio overview |
| Settings.tsx | ✅ EXISTS | Settings page |
| Dashboard.tsx | ✅ EXISTS | Main dashboard |

### Hooks (CHECK NEEDED)
- `useRealTrading` - Real trading hook (referenced in RealTrading.tsx)
- `usePaperTrading` - Paper trading hook
- `useTradingMode` - Mode switching hook

### Components (CHECK NEEDED)
- `TradingViewChart` - Chart component
- `OrderForm` - Order form component

## What Needs to Be Built

### Backend (Rust)
1. **Futures Trading Support**
   - Futures order placement
   - Leverage management
   - Position side (LONG/SHORT)
   - Margin management

2. **Additional Order Types**
   - Limit orders
   - Stop-Loss/Take-Profit
   - Trailing Stop (Futures)
   - OCO orders

3. **API Endpoints**
   - REST endpoints for manual trading
   - Real-time position updates via WebSocket

### Frontend
1. **Futures Trading UI**
   - Leverage selector
   - Long/Short buttons
   - Margin mode toggle

2. **Order Type Selector**
   - Market/Limit/Stop-Loss/TP/Trailing/OCO

3. **Position Management**
   - Open positions table
   - Close position buttons
   - Modify TP/SL

## Estimated Work

| Area | Existing | To Build | Effort |
|------|----------|----------|--------|
| Rust Spot Trading | 80% | 20% | Low |
| Rust Futures Trading | 30% | 70% | Medium |
| Frontend Real Trading | 60% | 40% | Medium |
| Tests | 40% | 60% | Medium |
| Integration | 20% | 80% | High |

**Total Estimate:** 70% new code, 30% reuse existing
