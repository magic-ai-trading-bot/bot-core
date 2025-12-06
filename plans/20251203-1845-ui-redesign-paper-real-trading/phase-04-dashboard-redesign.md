# Phase 4: Dashboard Redesign

## Context
- **Parent Plan**: [plan.md](./plan.md)
- **Dependencies**: Phase 1, 2, 3
- **Research**: [Trading UI Patterns](./research/researcher-01-trading-ui-patterns.md), [Landing & Navigation](./research/researcher-03-landing-navigation.md)

## Overview
| Field | Value |
|-------|-------|
| Priority | P0 - Critical |
| Status | Pending |
| Est. Time | 3-4 days |
| Description | Modern dashboard with bento grid layout, real-time widgets, and mode-aware styling |

## Key Insights
- Hybrid layout: Traditional 2-column + bento grid for widgets
- Portfolio summary top-center (most important)
- Real-time price updates with flash animations
- Mode-aware: Different accent colors for paper/real
- Widget-based for future customization

## Requirements

### Functional
- Portfolio summary card (balance, PnL, %)
- Live price ticker
- AI signals widget
- Recent trades widget
- Performance chart
- Quick actions (trade, deposit)
- Mode indicator prominent

### Non-Functional
- Real-time updates < 100ms perceived
- Responsive grid
- Skeleton loading
- WebSocket efficient updates

## Architecture

```
Dashboard/
├── DashboardHeader/
│   ├── PortfolioSummary
│   ├── ModeIndicator
│   └── QuickActions
├── MainContent/
│   ├── PriceTickerRow
│   ├── BentoGrid/
│   │   ├── PerformanceChart (large)
│   │   ├── AISignalsWidget (medium)
│   │   ├── RecentTradesWidget (medium)
│   │   ├── RiskMetricsWidget (small)
│   │   └── MarketOverviewWidget (small)
│   └── BottomSection/
│       └── NewsWidget
└── Sidebar (from Phase 2)
```

## Related Code Files

### Create
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Dashboard.tsx` (rewrite)
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/DashboardHeader.tsx` (rewrite)
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/PortfolioSummaryCard.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/PriceTickerRow.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/BentoGrid.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/widgets/PerformanceWidget.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/widgets/AISignalsWidget.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/widgets/RecentTradesWidget.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/widgets/RiskMetricsWidget.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/widgets/MarketOverviewWidget.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/QuickActionsBar.tsx`

### Modify
- Existing dashboard components to fit new design system

## Implementation Steps

1. **Create BentoGrid layout**
   ```tsx
   // CSS Grid with named areas
   // Responsive breakpoints
   // Gap and padding from design tokens
   ```

2. **Create PortfolioSummaryCard**
   - Total balance with currency
   - PnL (absolute + percentage)
   - Change indicator (up/down arrow)
   - Glassmorphism style
   - Mode-aware accent color

3. **Create PriceTickerRow**
   - Horizontal scroll of top coins
   - Price + 24h change
   - Flash animation on update (50ms green/red)
   - Click to navigate to trading

4. **Create Widget Components**
   - PerformanceWidget: Line chart of portfolio value
   - AISignalsWidget: Latest signals with confidence
   - RecentTradesWidget: Last 5 trades with PnL
   - RiskMetricsWidget: Exposure gauge, daily loss limit
   - MarketOverviewWidget: Market sentiment, trending

5. **Create QuickActionsBar**
   - "Trade Now" button
   - "Deposit" button
   - Mode-specific styling

6. **Create Dashboard page**
   - Compose all widgets
   - Real-time WebSocket updates
   - Skeleton loading states

7. **Add Animations**
   - Stagger animation on load
   - Number animations for stats
   - Smooth chart transitions

## Todo List

- [ ] Create BentoGrid layout component
- [ ] Create PortfolioSummaryCard
- [ ] Create PriceTickerRow with flash animations
- [ ] Create PerformanceWidget with Recharts
- [ ] Create AISignalsWidget
- [ ] Create RecentTradesWidget
- [ ] Create RiskMetricsWidget with gauges
- [ ] Create MarketOverviewWidget
- [ ] Create QuickActionsBar
- [ ] Compose Dashboard page
- [ ] Connect WebSocket for real-time data
- [ ] Add skeleton loading states
- [ ] Add stagger animations
- [ ] Responsive testing
- [ ] Write widget tests

## Success Criteria

- [ ] Dashboard loads with skeleton < 500ms
- [ ] Real-time updates smooth (no flicker)
- [ ] Bento grid responsive on all breakpoints
- [ ] Mode indicator clearly visible
- [ ] All widgets display correct data
- [ ] Quick actions work correctly

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| WebSocket reconnection | Medium | Reconnect logic + loading state |
| Data overload | Medium | Throttle updates, memoization |
| Chart performance | Low | Use canvas renderer |

## Security Considerations
- Balance/PnL data from authenticated API only
- No caching of sensitive portfolio data

## Next Steps
→ Phase 5: Mode Infrastructure
