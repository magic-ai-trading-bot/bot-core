# Quick Reference - Page Redesign Scout Report
**Generated**: 2025-12-03

## Page Overview Table

| # | Page | File | LOC | Status | Key Structure | Main Issue |
|---|------|------|-----|--------|---------------|-----------|
| 1 | **Dashboard** | Dashboard.tsx | 121 | ✅ Functional | Bento grid, 5 widgets | Widget data undefined |
| 2 | **Paper Trading** | PaperTrading.tsx | 173 | ✅ Functional | 3-column layout, tabs | Symbol hard-coded |
| 3 | **Real Trading** | RealTrading.tsx | 285 | ✅ Functional | 3-column + confirmation | No order persistence |
| 4 | **Portfolio** | Portfolio.tsx | 140 | ⏳ Placeholder | Glass hero card | Needs implementation |
| 5 | **AI Signals** | AISignals.tsx | 195 | ⏳ Placeholder | Neural network card | Needs implementation |
| 6 | **Settings** | Settings.tsx | 358 | ✅ Functional | 7 tabs | Too many tabs (mobile) |

**Total LOC**: 1,272 | **Total Pages**: 6 | **Functional**: 4 | **Placeholder**: 2

---

## Dashboard Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Dashboard.tsx`

### Structure
```
Dashboard
├── DashboardContentHeader (balance, pnl, pnlPercentage)
├── PriceTickerRow
└── BentoGrid (5 widgets)
    ├── PerformanceWidget (large)
    ├── AISignalsWidget (medium)
    ├── RecentTradesWidget (medium)
    ├── RiskMetricsWidget (small)
    └── MarketOverviewWidget (small)
```

### Data Flow
- **State**: balance, pnl, pnlPercentage, isLoading
- **WebSocket**: botStatus, positions, aiSignals, recentTrades
- **Calculation**: Risk metrics derived from positions & P&L

### Quick Fixes
- [ ] Define data sources for PerformanceWidget & MarketOverviewWidget
- [ ] Add widget customization options
- [ ] Add drag-drop reordering capability
- [ ] Improve loading states

---

## Paper Trading Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/PaperTrading.tsx`

### Structure
```
PaperTrading
├── Page Header (title + "Paper Mode" badge)
├── Portfolio Overview Card (6 metrics in grid)
└── TradingLayout (3-column)
    ├── Left: Chart + OrderBook + RecentTrades
    ├── Center: OrderForm + RiskWarningCard
    └── Right: Tabs (Positions | History | AI Insights)
```

### Data Flow
- **usePaperTrading Hook**: portfolio, openTrades, closedTrades, recentSignals, isLoading
- **Form Input**: symbol, side, quantity, price, orderType
- **Risk Settings**: dailyLossLimit (5%), maxDrawdown (15%)

### Quick Fixes
- [ ] Make symbol selection dynamic (dropdown, favorites)
- [ ] Connect OrderForm to actual API
- [ ] Link price click to OrderForm
- [ ] Add real-time P&L updates
- [ ] Improve portfolio overview layout

---

## Real Trading Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/RealTrading.tsx`

### Structure
```
RealTrading
├── RealModeWarningBanner (fixed top)
├── Page Header (title + pulsing "REAL MONEY MODE" badge)
├── Conditional Render
│   ├── [If NOT real mode] Mode Switch Card
│   └── [If real mode] TradingLayout (same as Paper)
│       + TradeConfirmationDialog (2-step)
```

### Data Flow
- **Same as PaperTrading** with stricter risk limits
- **Mode Check**: If tradingMode !== 'real', show switch prompt
- **Confirmation State**: isConfirmationOpen, pendingOrder

### Quick Fixes
- [ ] Standardize risk limits (decide: 3% or 5% daily loss?)
- [ ] Extract confirmation dialog state to context
- [ ] Connect API submission (currently TODO)
- [ ] Persist confirmation preferences

---

## Portfolio Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Portfolio.tsx`

### Current State
- **Status**: Coming Soon placeholder
- **Components**: Glass morphism hero card + 4 upcoming features
- **No Data**: Static only - needs implementation

### Needs Implementation
- [ ] Real portfolio data fetching
- [ ] Asset allocation pie chart
- [ ] Performance tracking charts
- [ ] Real-time balance updates
- [ ] Risk analytics display

---

## AI Signals Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/AISignals.tsx`

### Current State
- **Status**: Coming Soon placeholder
- **Components**: Glass morphism hero card + 4 AI models + 4 upcoming features
- **No Data**: Static only - needs implementation

### Needs Implementation
- [ ] Real trading signals from backend
- [ ] Signal history & performance
- [ ] Model accuracy display
- [ ] Entry/exit point recommendations
- [ ] Sentiment analysis display

---

## Settings Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Settings.tsx`

### Structure
```
Settings
├── Page Header
└── 7 Tabs (overcrowded on mobile)
    ├── Bot Settings (→ BotSettings component)
    ├── Per-Symbol (→ PerSymbolSettings component)
    ├── Strategy Tuning (→ StrategyTuningSettings component)
    ├── System Health (→ SystemMonitoring component)
    ├── API Keys (inline)
    ├── Notifications (inline)
    └── Security (inline)
└── ChatBot Widget (footer)
```

### Quick Fixes
- [ ] Reduce tab count (combine related tabs)
- [ ] Use drawer/accordion on mobile
- [ ] Add form validation
- [ ] Add success/error feedback
- [ ] Move ChatBot to side or modal
- [ ] Connect API keys to real backend

---

## Component Dependencies

### Most Complex
- **Settings.tsx** (3 heavy subcomponents)
  - BotSettings
  - PerSymbolSettings
  - StrategyTuningSettings

### Most Data-Dependent
- **Dashboard.tsx** (WebSocket + 5 widgets)
- **PaperTrading.tsx** (usePaperTrading hook)
- **RealTrading.tsx** (useRealTrading + mode checking)

### Most Independent
- **Portfolio.tsx** (static only)
- **AISignals.tsx** (static only)

---

## Redesign Priority

### 🔥 Quick Wins (Week 1)
1. **Settings** - Reduce tabs (5-7 → 4-5), add mobile drawer
2. **Dashboard** - Fix widget data sources, improve loading
3. **Paper Trading** - Make symbol dynamic, fix data flow

### 📦 Features (Week 2)
1. **Portfolio** - Full implementation with real data
2. **AI Signals** - Full implementation with real signals

### ✨ Polish (Week 3)
1. **Trading Pages** - Mobile optimization, UX improvements
2. **All Pages** - Accessibility, performance optimization

---

## Key Numbers

| Metric | Value |
|--------|-------|
| Total LOC | 1,272 |
| Average LOC per page | 212 |
| Largest page | Settings (358 LOC) |
| Smallest page | Dashboard (121 LOC) |
| Components in Dashboard | 8 |
| Components in Settings | 14 |
| WebSocket dependencies | 3 (Dashboard, Paper, Real) |
| Hook dependencies | 5 different hooks used |

---

## Report Files
- **Full Report**: `scout-251203-page-redesign.md` (2.5KB, detailed analysis)
- **JSON Data**: `scout-251203-page-redesign.json` (4.2KB, structured data)
- **Quick Ref**: `scout-251203-QUICK-REFERENCE.md` (this file, ~2KB)

---

**Next Steps**:
1. Review this quick reference
2. Read full markdown report for deep dive
3. Use JSON file for programmatic access
4. Start with quick wins in Settings page
5. Plan feature implementations for Portfolio & AI Signals

