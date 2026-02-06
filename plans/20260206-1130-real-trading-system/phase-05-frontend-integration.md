# Phase 05: Frontend Integration

**Status**: ✅ Complete | **Completed**: 2026-02-06

## Context

RealTrading.tsx page exists with basic layout. useRealTrading hook has placeholders. Need order form, position management, and order history UI.

## Overview

Complete frontend for real trading: order form with confirmation dialog, positions table, order history, real-time updates.

## Key Insights (From Research)

- Existing page uses glassmorphism design with RED theme (danger indicator)
- useRealTrading hook mirrors usePaperTrading structure
- WebSocket already handles MarketData and trade events
- 2-step confirmation pattern required for safety

## Requirements

1. Order form component (symbol, side, type, quantity, price, SL/TP)
2. Confirmation dialog with order summary
3. Open positions table with close/modify buttons
4. Open orders table with cancel button
5. Order history with filters (date, symbol, status)
6. Real-time P&L updates
7. Leverage selector for futures

## Architecture

```
RealTrading.tsx
    ├── OrderForm (enhanced)
    │   ├── SymbolSelector
    │   ├── SideToggle (Buy/Sell, Long/Short)
    │   ├── OrderTypeSelect
    │   ├── QuantityInput
    │   ├── PriceInput (for limit)
    │   ├── SLTPInputs
    │   ├── LeverageSlider
    │   └── SubmitButton (with 2-step confirmation)
    ├── PendingConfirmationDialog (new)
    │   ├── OrderSummary
    │   ├── Timer (60s countdown)
    │   ├── RiskWarning
    │   └── Confirm/Cancel buttons
    ├── PositionsTable (existing)
    │   ├── UnrealizedPnL (real-time)
    │   └── CloseButton
    ├── OrdersTable (new)
    │   ├── PendingOrders
    │   └── CancelButton
    └── TradeHistoryTable (existing)
```

## Related Files

| File | Action | Description |
|------|--------|-------------|
| `nextjs-ui-dashboard/src/pages/RealTrading.tsx` | Modified | Added OrdersTable, PendingConfirmationDialog |
| `nextjs-ui-dashboard/src/hooks/useRealTrading.ts` | Modified | Added placeOrder, cancelOrder, modifySlTp, etc. |

## Implementation Summary

### 1. useRealTrading Hook - New Methods
- `placeOrder(request)` - Place order with 2-step API confirmation
- `confirmOrder()` - Confirm pending order with token
- `cancelOrder(orderId)` - Cancel specific order
- `cancelAllOrders(symbol?)` - Cancel all orders (optional symbol filter)
- `modifySlTp(symbol, stopLoss?, takeProfit?)` - Modify position SL/TP
- `clearPendingConfirmation()` - Clear pending confirmation state
- `fetchOrders()` - Fetch active orders

### 2. New State Fields
- `activeOrders: RealOrder[]` - List of active orders
- `pendingConfirmation: PendingOrderConfirmation | null` - API confirmation state

### 3. New Types
```typescript
interface RealOrder {
  id: string;
  exchange_order_id: number;
  symbol: string;
  side: string;
  order_type: string;
  quantity: number;
  executed_quantity: number;
  price: number | null;
  avg_fill_price: number;
  status: string;
  is_entry: boolean;
  created_at: string;
  updated_at: string;
}

interface PlaceOrderRequest {
  symbol: string;
  side: string;
  order_type: string;
  quantity: number;
  price?: number;
  stop_loss?: number;
  take_profit?: number;
  confirmation_token?: string;
}

interface PendingOrderConfirmation {
  token: string;
  expires_at: string;
  summary: string;
  order_details: PlaceOrderRequest;
}
```

### 4. RealTrading.tsx Components
- **OrdersTable** - Displays active orders with cancel button
- **PendingConfirmationDialog** - API 2-step confirmation with 60s timer
- **Enhanced tabs** - Added "Orders" tab alongside Positions/History

### 5. WebSocket Handlers
- `order_placed` - Refresh orders, portfolio, trades
- `order_filled` - Refresh orders, portfolio, trades
- `order_partially_filled` - Refresh orders, portfolio, trades
- `order_cancelled` - Refresh orders

## Todo

- [x] Add placeOrder to useRealTrading hook (with 2-step confirmation)
- [x] Add cancelOrder to useRealTrading hook
- [x] Add cancelAllOrders to useRealTrading hook
- [x] Add modifySlTp to useRealTrading hook
- [x] Add fetchOrders to useRealTrading hook
- [x] Add confirmOrder to useRealTrading hook
- [x] Create OrdersTable component (pending orders)
- [x] Create PendingConfirmationDialog component
- [x] Add "Orders" tab to RealTrading page
- [x] Wire up OrderForm to call placeOrder API
- [x] Wire up WebSocket order events
- [x] Add loading states for all actions
- [x] Add error handling with toasts
- [ ] Add leverage slider for futures mode (deferred - UI exists, needs futures mode toggle)
- [ ] Add position side toggle for futures (deferred - UI exists, needs futures mode toggle)
- [ ] Create OrderHistoryTable component (deferred - needs DB history endpoint)

## Success Criteria

- [x] Place market order via UI with confirmation
- [x] Place limit order via UI with confirmation
- [x] Cancel pending order via UI
- [x] Close position via UI
- [ ] Modify SL/TP via UI (endpoint ready, needs SL/TP input fields in UI)
- [ ] View order history with filters (deferred - needs DB)
- [x] Real-time P&L updates in positions table
- [ ] Leverage adjustment for futures (UI exists, needs futures mode toggle)

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Confirmation dialog bypassed | Low | High | Token required on backend |
| Stale price on confirm | Medium | Medium | 60s token expiry |
| WebSocket lag | Low | Low | Refresh button available |

## Security Considerations

- Never store API keys in frontend
- Confirmation token has 60s TTL (enforced by backend)
- Show full order summary before confirm
- Disable submit button during processing
- API confirmation dialog with countdown timer

## Next Steps

After completion: Proceed to Phase 06 (Testing & Integration) for E2E tests.
