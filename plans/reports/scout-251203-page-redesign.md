# Scout Report: Dashboard Pages for Redesign
**Date**: 2025-12-03  
**Scope**: Frontend pages requiring redesign in nextjs-ui-dashboard  
**Complexity**: Medium-High (6 pages, ~1,200 LOC total)

---

## Executive Summary

All 6 target pages located in `/nextjs-ui-dashboard/src/pages/` directory. Current state:
- **Dashboard.tsx** (121 LOC): Bento grid layout with 5 dashboard widgets, fully functional
- **PaperTrading.tsx** (173 LOC): Complete trading interface with chart, order form, positions panel
- **RealTrading.tsx** (285 LOC): Extended trading interface with 2-step confirmation, warning banner
- **Portfolio.tsx** (140 LOC): Placeholder "Coming Soon" with feature cards
- **AISignals.tsx** (195 LOC): Placeholder "Coming Soon" with AI model showcase
- **Settings.tsx** (358 LOC): Comprehensive 7-tab settings interface

---

## PAGE-BY-PAGE ANALYSIS

### 1. Dashboard Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Dashboard.tsx`

#### Current Structure
```
Dashboard (Main Container)
├── DashboardContentHeader (portfolio summary)
│   ├── Balance
│   ├── P&L
│   └── P&L Percentage
├── PriceTickerRow (horizontal price ticker)
└── BentoGrid (responsive widget layout)
    ├── PerformanceWidget (large, chart widget)
    ├── AISignalsWidget (medium, signals list)
    ├── RecentTradesWidget (medium, trades list)
    ├── RiskMetricsWidget (small, risk metrics)
    └── MarketOverviewWidget (small, market data)
```

#### Key Components Used
- `DashboardContentHeader`: Portfolio stats display
- `PriceTickerRow`: Live price tickers
- `BentoGrid` & `BentoWidget`: Responsive layout system
- Dashboard Widgets:
  - `PerformanceWidget`: Performance chart
  - `AISignalsWidget`: AI trading signals
  - `RecentTradesWidget`: Recent executed trades
  - `RiskMetricsWidget`: Risk indicators (exposure %, daily loss %, positions)
  - `MarketOverviewWidget`: Market overview data

#### Data & State
- **Portfolio Data**: balance, pnl, pnlPercentage (from WebSocket botStatus)
- **WebSocket State**: 
  - `wsState.botStatus` (total_pnl)
  - `wsState.positions` (open positions array)
  - `wsState.aiSignals` (trading signals)
  - `wsState.recentTrades` (executed trades)
- **Loading State**: `isLoading` (boolean)
- **Risk Metrics**: Calculated from positions & P&L

#### Current Issues/Opportunities
- Widgets are static size (large/medium/small) - limited customization
- No drag-drop reordering
- Risk metrics calculation is hard-coded
- No customizable thresholds
- Performance widget data source undefined
- Market overview widget data source undefined

---

### 2. Paper Trading Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/PaperTrading.tsx`

#### Current Structure
```
PaperTrading (Main Container)
├── Page Header
│   ├── Title: "Paper Trading"
│   └── Badge: "Paper Mode"
├── Portfolio Overview Card
│   ├── Balance
│   ├── Equity
│   ├── Total P&L
│   ├── Win Rate
│   ├── Total Trades
│   └── Sharpe Ratio
├── TradingLayout (3-column layout)
│   ├── Left: Chart + Order Book/Recent Trades
│   │   ├── TradingViewChart
│   │   ├── OrderBook
│   │   └── RecentTradesList
│   ├── Center: Order Form + Risk Warning
│   │   ├── OrderForm
│   │   └── RiskWarningCard
│   └── Right: Positions/History/AI Insights (tabbed)
│       ├── OpenPositions
│       ├── TradeHistory
│       └── AIInsightsPanel
```

#### Key Components Used
- `TradingLayout`: 3-column trading interface
- `TradingViewChart`: Interactive chart
- `OrderForm`: Order submission form (side, quantity, price, type)
- `OrderBook`: Market order book display
- `RecentTradesList`: Recent trades list
- `RiskWarningCard`: Risk metrics display
- `OpenPositions`: Active positions table
- `TradeHistory`: Closed trades history
- `AIInsightsPanel`: AI signals and insights
- UI: Card, Badge, Tabs

#### Data & Props
- **Portfolio Data** (from usePaperTrading hook):
  - `portfolio.current_balance`
  - `portfolio.equity`
  - `portfolio.total_pnl`
  - `portfolio.win_rate`
  - `portfolio.total_trades`
  - `portfolio.sharpe_ratio`
- **Trade Data**:
  - `openTrades` (current positions)
  - `closedTrades` (trade history)
  - `recentSignals` (AI signals)
  - `recentTrades` (recent executions)
- **Order Form Data**: symbol, side, quantity, price, orderType
- **Risk Settings**: dailyLossLimit (5%), maxDrawdown (15%)
- **UI State**: selectedSymbol

#### Current Issues/Opportunities
- Hard-coded symbol selection ("BTCUSDT")
- OrderForm has no actual submission integration (TODO comment)
- Portfolio overview uses grid layout (hard to customize)
- No real-time profit/loss updates
- Price click handler not connected to OrderForm
- No trade confirmation for paper trading

---

### 3. Real Trading Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/RealTrading.tsx`

#### Current Structure
```
RealTrading (Main Container)
├── RealModeWarningBanner (fixed at top)
├── Page Header + Mode Badge
│   ├── Title: "Real Trading"
│   ├── Warning: "All trades execute with real funds"
│   └── Badge: "REAL MONEY MODE" (pulsing animation)
├── Conditional Render: If NOT in real mode
│   ├── Mode Switch Card (warning + feature list)
│   └── Features Preview Card
└── [If in real mode] TradingLayout (same as Paper Trading)
    ├── Chart Section (same structure)
    ├── Order Form Section (with confirmation required)
    └── Positions/History/AI Section (same structure)
└── TradeConfirmationDialog (modal overlay)
```

#### Key Components Used
- `RealModeWarningBanner`: Always-visible warning
- `TradingLayout`, `TradingViewChart`, `OrderForm`, etc. (same as PaperTrading)
- `TradeConfirmationDialog`: 2-step order confirmation dialog
- `ModeSwitchCard`: Conditional display for mode switching

#### Data & Props
- **Same as PaperTrading**, plus:
- **Confirmation Dialog State**:
  - `isConfirmationOpen` (boolean)
  - `pendingOrder` (OrderFormData | null)
- **Risk Settings**: More strict dailyLossLimit (3%), maxDrawdown (10%)
- **Mode Check**: If mode !== 'real', show mode switch prompt

#### Current Issues/Opportunities
- Two different risk limits (paper: 5%/15%, real: 3%/10%)
- Confirmation dialog state managed in page (could be extracted)
- Mode switch card only shown if not in real mode (UX could be clearer)
- API submission still has TODO comment
- No persistence of order confirmation decisions

---

### 4. Portfolio Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Portfolio.tsx`

#### Current Structure
```
Portfolio (Main Container - Coming Soon Placeholder)
├── Header
│   ├── Title: "Portfolio"
│   └── Subtitle: "Track and manage your cryptocurrency holdings"
├── Hero Coming Soon Card
│   ├── Animated background glow
│   ├── Glass morphism card with corner accents
│   ├── Icon + Badge: "Coming Soon"
│   ├── Title: "Portfolio Management"
│   ├── Description text
│   └── Stats Row
│       ├── Supported Exchanges: 10+
│       ├── Real-time Updates: <100ms
│       └── Security Grade: A+
└── Upcoming Features Grid (4 cards)
    ├── Asset Allocation
    ├── Performance Tracking
    ├── Balance Overview
    └── Risk Analytics
```

#### Key Components Used
- Lucide Icons: PieChart, TrendingUp, Wallet, BarChart3, Sparkles, ArrowUpRight, Shield, Zap
- Custom styled cards with:
  - Glassmorphism effects
  - Gradient backgrounds
  - Animated glows
  - Decorative corner accents

#### Data & Props
- **Upcoming Features** (static): 4 feature cards with icons, titles, descriptions
- **Stats** (static): 3 stat displays with icons
- **No dynamic data** - placeholder only

#### Current Issues/Opportunities
- Placeholder only - needs full implementation
- No actual portfolio data displayed
- No real-time updates
- No data integration
- Could include: holdings, allocations, performance, risk metrics

---

### 5. AI Signals Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/AISignals.tsx`

#### Current Structure
```
AISignals (Main Container - Coming Soon Placeholder)
├── Header
│   ├── Title: "AI Signals"
│   └── Subtitle: "AI-powered trading signals and market predictions"
├── Hero Coming Soon Card
│   ├── Animated background glow (purple/AI themed)
│   ├── Glass morphism card with corner accents
│   ├── Neural network SVG decoration
│   ├── Icon + Badge: "Coming Soon"
│   ├── Title: "AI Trading Signals"
│   ├── Description text
│   └── Stats Row
│       ├── Data Points: 10M+
│       ├── Model Accuracy: 70%
│       └── Signal Latency: <50ms
├── AI Models Section (4 model cards)
│   ├── LSTM (68%)
│   ├── GRU (65%)
│   ├── Transformer (70%)
│   └── GPT-4 (72%)
└── Upcoming Features Grid (4 cards)
    ├── ML Price Predictions
    ├── Real-time Signals
    ├── Entry & Exit Points
    └── Sentiment Analysis
```

#### Key Components Used
- Lucide Icons: Brain, Zap, Target, LineChart, Sparkles, Activity, Cpu, Network
- Custom styled cards with:
  - Glassmorphism effects
  - Gradient backgrounds
  - Animated glows
  - Neural network SVG
  - Hover effects

#### Data & Props
- **AI Models** (static): 4 models with accuracy percentages
- **Upcoming Features** (static): 4 feature cards
- **Stats** (static): 3 stat displays
- **No dynamic data** - placeholder only

#### Current Issues/Opportunities
- Placeholder only - needs full implementation
- No actual AI signals displayed
- No real trading signals
- No signal history/performance
- Could include: latest signals, signal history, model performance, accuracy metrics

---

### 6. Settings Page
**File**: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Settings.tsx`

#### Current Structure
```
Settings (Main Container)
├── Page Header
│   ├── Title: "Bot Settings"
│   └── Subtitle: "Manage configuration and preferences"
├── Tabs Navigation (7 tabs)
│   ├── Bot Settings
│   ├── Per-Symbol
│   ├── Strategy Tuning
│   ├── System Health
│   ├── API Keys
│   ├── Notifications
│   └── Security
├── Tab 1: Bot Settings
│   └── <BotSettings /> component
├── Tab 2: Per-Symbol Settings
│   └── <PerSymbolSettings /> component
├── Tab 3: Strategy Tuning
│   └── <StrategyTuningSettings /> component
├── Tab 4: System Monitoring
│   └── <SystemMonitoring /> component
├── Tab 5: API Keys
│   ├── Binance API Configuration (API Key + Secret)
│   ├── Security Note
│   ├── Test Connection + Save buttons
│   └── Trading Permissions (Spot, Futures, Margin, Options toggles)
├── Tab 6: Notifications
│   ├── Email, Push, Telegram, Discord toggles
│   ├── Conditional Telegram Token input
│   └── Save button
└── Tab 7: Security
    ├── 2FA Status
    ├── Change Password
    └── Active Sessions
├── ChatBot Widget (footer)
```

#### Key Components Used
- `BotSettings`: Complex bot configuration component
- `PerSymbolSettings`: Per-symbol trading settings
- `StrategyTuningSettings`: Strategy tuning parameters
- `SystemMonitoring`: System health & monitoring
- UI: Card, CardHeader, CardTitle, CardContent, Tabs, TabsList, TabsTrigger, TabsContent
- Form: Input, Label, Button, Badge, Switch
- Custom: ChatBot widget

#### Data & State
- **API Keys**: apiKey, secretKey (masked)
- **Notifications**: email, push, telegram, discord (toggles)
- **Notification Settings**: telegram token (conditional)
- **Portfolio Data**: current_balance (from context)
- **Form State**: All input fields

#### Current Issues/Opportunities
- 7 tabs is too many - navigation crowded on mobile
- API keys are hard-coded demo strings (no real integration)
- Notification settings not integrated with backend
- Per-symbol and strategy settings offloaded to separate components
- ChatBot widget at footer may be distracting
- No form validation shown
- No success/error feedback

---

## COMPONENT DEPENDENCY MAP

### Dashboard Page Dependencies
```
Dashboard.tsx
├── DashboardContentHeader (portfolio display)
├── PriceTickerRow
├── BentoGrid + BentoWidget (layout system)
└── Dashboard Widgets/
    ├── PerformanceWidget (chart data undefined)
    ├── AISignalsWidget (uses wsState.aiSignals)
    ├── RecentTradesWidget (uses wsState.recentTrades)
    ├── RiskMetricsWidget (calculated metrics)
    └── MarketOverviewWidget (data undefined)
```

### Trading Pages (PaperTrading + RealTrading)
```
Trading Page
├── TradingLayout (3-column wrapper)
├── TradingViewChart (chart integration)
├── OrderForm (order entry)
├── OrderBook (market depth)
├── RecentTradesList
├── RiskWarningCard
├── OpenPositions (active trades)
├── TradeHistory (closed trades)
├── AIInsightsPanel
└── [RealTrading only] TradeConfirmationDialog
```

### Settings Page
```
Settings.tsx
├── BotSettings (large config component)
├── PerSymbolSettings (symbol-specific settings)
├── StrategyTuningSettings (strategy tuning)
├── SystemMonitoring (health metrics)
├── API Configuration (inline)
├── Notifications Configuration (inline)
├── Security Configuration (inline)
└── ChatBot (footer widget)
```

---

## KEY INSIGHTS FOR REDESIGN

### What's Working Well
1. **Dashboard**: Bento grid layout is flexible and modern
2. **Trading Pages**: 3-column layout works for trading scenario
3. **Settings**: Tabbed interface is comprehensive
4. **Visual Design**: Glass morphism effects on placeholders are polished

### Design Opportunities
1. **Dashboard**: Could add customizable widgets, drag-drop reordering
2. **Paper/Real Trading**: Could streamline UI, reduce clutter
3. **Portfolio & AI Signals**: Placeholders need full-featured implementations
4. **Settings**: Tab overflow on mobile, could use drawer/accordion

### Technical Considerations
1. **Data Sources**: Many components have undefined data sources (need integration)
2. **WebSocket State**: Dashboard tightly coupled to WebSocket context
3. **Responsive**: Current layouts may need mobile optimization
4. **Performance**: BentoGrid with 5 widgets could be heavy

### Dependencies & Constraints
- Settings depends on: BotSettings, PerSymbolSettings, StrategyTuningSettings (complex subcomponents)
- Trading pages tightly coupled to: usePaperTrading, useRealTrading hooks
- Dashboard depends on: WebSocket state, multiple custom widgets
- Portfolio & AI Signals: Currently independent, minimal dependencies

---

## FILE LOCATIONS SUMMARY

| Page | File Path | LOC | Status | Dependencies |
|------|-----------|-----|--------|--------------|
| Dashboard | `src/pages/Dashboard.tsx` | 121 | Functional | WebSocket, 5 widgets |
| Paper Trading | `src/pages/PaperTrading.tsx` | 173 | Functional | usePaperTrading hook, 7 components |
| Real Trading | `src/pages/RealTrading.tsx` | 285 | Functional | useRealTrading hook, confirmation dialog |
| Portfolio | `src/pages/Portfolio.tsx` | 140 | Placeholder | None (static) |
| AI Signals | `src/pages/AISignals.tsx` | 195 | Placeholder | None (static) |
| Settings | `src/pages/Settings.tsx` | 358 | Functional | 4 complex subcomponents + ChatBot |

---

## COMPONENT DIRECTORY STRUCTURE

```
src/components/
├── dashboard/
│   ├── BentoGrid.tsx (layout system)
│   ├── DashboardContentHeader.tsx
│   ├── PriceTickerRow.tsx
│   ├── BotSettings.tsx (complex)
│   ├── PerSymbolSettings.tsx (complex)
│   ├── StrategyTuningSettings.tsx (complex)
│   ├── SystemMonitoring.tsx
│   ├── widgets/
│   │   ├── AISignalsWidget.tsx
│   │   ├── PerformanceWidget.tsx
│   │   ├── RecentTradesWidget.tsx
│   │   ├── RiskMetricsWidget.tsx
│   │   └── MarketOverviewWidget.tsx
│   └── [other dashboard components]
├── trading/
│   ├── TradingLayout.tsx (3-column wrapper)
│   ├── TradingViewChart.tsx
│   ├── OrderForm.tsx
│   ├── OrderBook.tsx
│   ├── RecentTradesList.tsx
│   ├── OpenPositions.tsx
│   ├── TradeHistory.tsx
│   ├── RiskWarningCard.tsx
│   ├── AIInsightsPanel.tsx
│   ├── TradeConfirmationDialog.tsx
│   └── [other trading components]
└── [other component directories]
```

---

## RECOMMENDED REDESIGN APPROACH

### Phase 1: High-Impact Redesigns (Quick Wins)
1. **Dashboard**: Add customization features, improve widget loading
2. **Settings**: Reorganize tabs, improve mobile layout, add form validation

### Phase 2: Feature-Complete Placeholders
1. **Portfolio**: Implement full portfolio tracking with real data
2. **AI Signals**: Implement full signals display with history

### Phase 3: Enhanced Trading UX
1. **Paper Trading**: Streamline interface, add quick actions
2. **Real Trading**: Improve confirmation flow, better warnings

### Phase 4: Polish & Optimization
1. Performance optimization (reduce re-renders, optimize widgets)
2. Accessibility improvements
3. Mobile responsiveness enhancements

---

## UNRESOLVED QUESTIONS

1. **Performance Widget Data**: Where does performance chart data come from? (undefined source)
2. **Market Overview Widget**: Data source for market overview? (undefined source)
3. **Portfolio Page**: Should show paper portfolio or real portfolio or both?
4. **AI Signals Page**: Should show real signals from ML backend or placeholder signals?
5. **Settings**: Are complex subcomponents (BotSettings, PerSymbolSettings, etc.) finalized?
6. **Data Persistence**: Should settings/preferences be persisted to localStorage or backend?
7. **WebSocket Integration**: Is websocket state management finalized?
8. **Mobile Layout**: Current 3-column trading layout works on mobile?

---

**Report Generated**: 2025-12-03  
**Total Pages Analyzed**: 6  
**Total Lines of Code**: ~1,272 LOC  
**Components Identified**: 30+  
**Status**: Ready for redesign planning
