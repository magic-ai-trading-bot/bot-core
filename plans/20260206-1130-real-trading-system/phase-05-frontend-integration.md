# Phase 05: Frontend Integration

**Status**: Pending | **Estimated Time**: 1.5 days

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
    ├── OrderForm (new)
    │   ├── SymbolSelector
    │   ├── SideToggle (Buy/Sell, Long/Short)
    │   ├── OrderTypeSelect
    │   ├── QuantityInput
    │   ├── PriceInput (for limit)
    │   ├── SLTPInputs
    │   ├── LeverageSlider (futures)
    │   └── SubmitButton
    ├── ConfirmationDialog (new)
    │   ├── OrderSummary
    │   ├── RiskWarning
    │   └── ConfirmButton
    ├── PositionsTable (enhance)
    │   ├── UnrealizedPnL (real-time)
    │   ├── CloseButton
    │   └── ModifySLTPButton
    ├── OrdersTable (new)
    │   ├── PendingOrders
    │   └── CancelButton
    └── OrderHistoryTable (new)
        ├── Filters
        └── PaginatedList
```

## Related Files

| File | Action | Description |
|------|--------|-------------|
| `nextjs-ui-dashboard/src/pages/RealTrading.tsx` | Modify | Add order form, tables |
| `nextjs-ui-dashboard/src/hooks/useRealTrading.ts` | Modify | Add placeOrder, cancelOrder |
| `nextjs-ui-dashboard/src/components/trading/OrderForm.tsx` | Create | Order form component |
| `nextjs-ui-dashboard/src/components/trading/ConfirmationDialog.tsx` | Create | Confirmation dialog |
| `nextjs-ui-dashboard/src/components/trading/OrdersTable.tsx` | Create | Orders list |
| `nextjs-ui-dashboard/src/components/trading/OrderHistoryTable.tsx` | Create | History with filters |

## Implementation Steps

### 1. OrderForm Component
```tsx
interface OrderFormProps {
  onSubmit: (order: OrderFormData) => void;
  isLoading: boolean;
  mode: 'spot' | 'futures';
}

interface OrderFormData {
  symbol: string;
  side: 'BUY' | 'SELL';
  orderType: 'MARKET' | 'LIMIT' | 'STOP_LOSS' | 'TAKE_PROFIT' | 'OCO';
  quantity: number;
  price?: number;
  stopPrice?: number;
  stopLoss?: number;
  takeProfit?: number;
  leverage?: number;
  positionSide?: 'LONG' | 'SHORT';
}
```

### 2. ConfirmationDialog Component
```tsx
interface ConfirmationDialogProps {
  isOpen: boolean;
  onConfirm: () => void;
  onCancel: () => void;
  orderSummary: {
    symbol: string;
    side: string;
    type: string;
    quantity: number;
    estimatedValue: number;
    estimatedFee: number;
  };
  riskWarnings: string[];
}
```

### 3. Hook - placeOrder Method
```typescript
const placeOrder = useCallback(async (orderData: OrderFormData) => {
  // Step 1: Get confirmation token
  const confirmResponse = await fetch(`${API_BASE}/api/real-trading/orders`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify(orderData),
  });

  const { data } = await confirmResponse.json();
  if (data.confirmation_required) {
    // Show confirmation dialog
    setConfirmationData({
      token: data.token,
      summary: data.summary,
      expiresAt: data.expires_at,
    });
    return;
  }

  // Step 2: Execute with token (if already confirmed)
  // ... handle success/error
}, [API_BASE]);
```

### 4. PositionsTable Enhancements
```tsx
// Add to existing PositionsTable
<td>
  <Button
    variant="outline"
    size="sm"
    onClick={() => handleModifySLTP(position.id)}
  >
    SL/TP
  </Button>
  <Button
    variant="destructive"
    size="sm"
    onClick={() => handleClosePosition(position.id)}
  >
    Close
  </Button>
</td>
```

### 5. OrderHistoryTable Component
```tsx
interface OrderHistoryTableProps {
  orders: OrderInfo[];
  onLoadMore: () => void;
  hasMore: boolean;
  filters: {
    symbol?: string;
    status?: string;
    startDate?: Date;
    endDate?: Date;
  };
  onFilterChange: (filters: Filters) => void;
}
```

## Todo

- [ ] Create OrderForm component
- [ ] Create ConfirmationDialog component
- [ ] Add placeOrder to useRealTrading hook
- [ ] Add cancelOrder to useRealTrading hook
- [ ] Add fetchOrderHistory to useRealTrading hook
- [ ] Create OrdersTable component (pending orders)
- [ ] Create OrderHistoryTable component
- [ ] Add leverage slider for futures mode
- [ ] Add position side toggle for futures
- [ ] Enhance PositionsTable with actions
- [ ] Add real-time P&L updates in positions
- [ ] Wire up WebSocket order updates
- [ ] Add loading states for all actions
- [ ] Add error handling with toasts
- [ ] Test order placement flow
- [ ] Test order cancellation flow
- [ ] Test modify SL/TP flow

## Success Criteria

- [ ] Place market order via UI with confirmation
- [ ] Place limit order via UI with confirmation
- [ ] Cancel pending order via UI
- [ ] Close position via UI
- [ ] Modify SL/TP via UI
- [ ] View order history with filters
- [ ] Real-time P&L updates in positions table
- [ ] Leverage adjustment for futures

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Confirmation dialog bypassed | Low | High | Token required on backend |
| Stale price on confirm | Medium | Medium | Show warning if > 5s old |
| WebSocket lag | Low | Low | Refresh button available |

## Security Considerations

- Never store API keys in frontend
- Confirmation token has 60s TTL
- Show full order summary before confirm
- Disable submit button during processing

## Next Steps

After completion: Proceed to Phase 06 (Testing & Integration) for E2E tests.
