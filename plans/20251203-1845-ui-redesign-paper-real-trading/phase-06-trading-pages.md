# Phase 6: Trading Pages

## Context
- **Parent Plan**: [plan.md](./plan.md)
- **Dependencies**: Phase 1-5
- **Research**: [Trading UI Patterns](./research/researcher-01-trading-ui-patterns.md)

## Overview
| Field | Value |
|-------|-------|
| Priority | P0 - Critical |
| Status | Pending |
| Est. Time | 4-5 days |
| Description | Complete paper trading and real trading pages with mode-specific features, TradingView charts, order forms |

## Key Insights
- Paper Trading: Full feature access, practice mode
- Real Trading: Safety-first, confirmation dialogs, warning states
- Shared components: Charts, order book, trade history
- Mode-specific: Order form behavior, confirmation flows

## Requirements

### Functional
- TradingView-style chart integration
- Order form (market, limit, stop-limit)
- Order book display
- Recent trades list
- Position management
- Trade history with PnL
- Paper: Instant execution simulation
- Real: 2-step confirmation, risk warnings

### Non-Functional
- Chart updates < 50ms
- Order submission < 100ms perceived
- Responsive on all devices
- Keyboard shortcuts for traders

## Architecture

```
TradingPages/
├── PaperTradingPage/
│   ├── TradingLayout (shared)
│   ├── ChartSection
│   │   ├── TradingViewChart
│   │   └── ChartToolbar
│   ├── OrderSection
│   │   ├── OrderForm (paper variant)
│   │   ├── OrderBook
│   │   └── RecentTrades
│   ├── PositionSection
│   │   ├── OpenPositions
│   │   └── TradeHistory
│   └── AIInsightsPanel
│
├── RealTradingPage/
│   ├── RealModeWarningBanner (always visible)
│   ├── TradingLayout (shared)
│   ├── ChartSection (same)
│   ├── OrderSection
│   │   ├── OrderForm (real variant + confirmations)
│   │   ├── RiskWarningCard
│   │   ├── OrderBook
│   │   └── RecentTrades
│   ├── PositionSection
│   │   ├── OpenPositions (with real PnL)
│   │   └── TradeHistory
│   └── AccountBalanceCard
│
└── SharedComponents/
    ├── TradingViewChart.tsx
    ├── OrderBook.tsx
    ├── RecentTradesList.tsx
    ├── PositionCard.tsx
    ├── TradeConfirmationDialog.tsx
    └── RiskWarningCard.tsx
```

## Related Code Files

### Create
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/PaperTrading.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/RealTrading.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/TradingLayout.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/TradingViewChart.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/OrderForm.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/OrderBook.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/RecentTradesList.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/OpenPositions.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/TradeHistory.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/TradeConfirmationDialog.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/RiskWarningCard.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/AIInsightsPanel.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/AccountBalanceCard.tsx`

### Modify
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/App.tsx` - Add trading routes

## Implementation Steps

1. **Create TradingLayout**
   ```tsx
   // 3-column layout: Chart | Order | Positions
   // Responsive: Stack on mobile
   // Resizable panels (drag to resize)
   ```

2. **Create TradingViewChart**
   - Integrate lightweight-charts or TradingView widget
   - Candlestick + line chart options
   - Drawing tools
   - Indicator overlays
   - Time range selector

3. **Create OrderForm**
   ```tsx
   interface OrderFormProps {
     mode: 'paper' | 'real';
     onSubmit: (order: Order) => void;
   }
   // Paper: Direct submission
   // Real: Opens confirmation dialog first
   ```

4. **Create OrderBook**
   - Bid/ask levels with depth
   - Color-coded (green bids, red asks)
   - Click to set price
   - Real-time updates via WebSocket

5. **Create OpenPositions**
   - List of current positions
   - Live PnL updates
   - Close position button
   - Paper: Simulated PnL
   - Real: Actual PnL with warnings

6. **Create TradeConfirmationDialog (Real mode)**
   - Order summary
   - Risk calculation
   - "I understand this is real money" checkbox
   - Final confirm/cancel

7. **Create RiskWarningCard**
   - Current exposure
   - Daily loss status
   - Margin usage
   - Circuit breaker status

8. **Create AIInsightsPanel**
   - Latest AI signals
   - Confidence scores
   - Recommended actions
   - Signal history

9. **Compose Pages**
   - PaperTradingPage: Full features, blue accent
   - RealTradingPage: Warning banner, red accents for caution

## Todo List

- [ ] Create TradingLayout with resizable panels
- [ ] Integrate TradingView/lightweight-charts
- [ ] Create OrderForm component
- [ ] Create OrderBook with real-time updates
- [ ] Create RecentTradesList
- [ ] Create OpenPositions component
- [ ] Create TradeHistory with filters
- [ ] Create TradeConfirmationDialog
- [ ] Create RiskWarningCard
- [ ] Create AIInsightsPanel
- [ ] Create AccountBalanceCard
- [ ] Compose PaperTradingPage
- [ ] Compose RealTradingPage with safety features
- [ ] Add keyboard shortcuts
- [ ] Connect WebSocket for live data
- [ ] Responsive testing
- [ ] Write component tests

## Success Criteria

- [ ] Charts render correctly with live data
- [ ] Order form submits correctly per mode
- [ ] Real mode always shows confirmation dialog
- [ ] Positions update in real-time
- [ ] Keyboard shortcuts work (B=buy, S=sell)
- [ ] All components responsive on mobile
- [ ] No accidental real trades possible

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Accidental real trade | CRITICAL | 2-step confirmation, checkbox |
| Chart library size | Medium | Lazy load, CDN |
| WebSocket disconnect | Medium | Reconnect + stale data indicator |
| Order form errors | High | Validation, error states |

## Security Considerations
- Real order submission requires fresh auth token
- Rate limit order submissions
- Log all order attempts
- Validate order params server-side

## Next Steps
→ Phase 7: Settings & Profile
