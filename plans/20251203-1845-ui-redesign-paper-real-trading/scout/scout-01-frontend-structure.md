# Scout Report: Frontend UI Structure Analysis
**Date**: 2025-12-03  
**Focus**: Paper Trading UI & Mode Separation Architecture  
**Status**: Complete

---

## Executive Summary

The frontend uses a **single-page React application (Vite + React 18)** with clear separation between paper trading and other features. Current structure is **monolithic** with paper trading integrated directly into pages, making mode separation challenging for the redesign.

**Key Finding**: No existing real/live trading UI exists. The system is currently **paper-trading only**, requiring greenfield implementation for mode separation.

---

## 1. Page Components Structure

### Location
`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/`

### Current Pages (9 total)
| Page | Purpose | Current Implementation | Redesign Impact |
|------|---------|------------------------|-----------------|
| **Dashboard.tsx** | Main dashboard with bot status, AI signals, performance | No mode separation (AI-centric) | May need light updates for mode-agnostic nav |
| **TradingPaper.tsx** | Paper trading interface (2,148 lines - LARGEST) | Hardcoded paper trading UI with tabs (Overview, Signals, Trades, Settings) | CRITICAL: Core of redesign - need to extract into mode-agnostic components |
| **TradingPaperNew.tsx** | Alternative paper trading page (likely experimental) | Parallel implementation | Consolidate during redesign |
| **Settings.tsx** | Global app settings | Shared settings | Extend for trading mode configuration |
| **Index.tsx** | Landing page | Landing content | No changes needed |
| **Login.tsx** | Authentication | Auth flow | No changes needed |
| **Register.tsx** | User registration | Auth flow | No changes needed |
| **NotFound.tsx** | 404 page | Error handling | No changes needed |
| **HowItWorks.tsx** | Help/tutorial page | Informational | No changes needed |
| **TradeAnalyses.tsx** | Trade history analysis | Trade analysis UI | May need mode parameter |

### Key Observation
- **TradingPaper.tsx is the primary target** for redesign (2,148 lines)
- Contains ALL paper trading logic: portfolio display, trade management, settings, AI signals
- Tightly coupled with PaperTradingContext
- No mode parameter or conditional rendering for real trading

---

## 2. Dashboard Components Structure

### Location
`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/`

### Components (16 total)
| Component | Purpose | Dependencies | Redesign Note |
|-----------|---------|--------------|----------------|
| **DashboardHeader.tsx** | Top nav with status and controls | Auth context | Shared component - add mode toggle |
| **BotStatus.tsx** | Bot operational status display | Paper trading context | Already mode-agnostic |
| **AISignals.tsx** | AI trading signals grid | Paper trading context | Already mode-agnostic |
| **AISignalsNew.tsx** | Alternate signals component | Paper trading context | Consolidate |
| **AIStrategySelector.tsx** | Strategy selection interface | Paper trading context | Already mode-agnostic |
| **PerformanceChart.tsx** | Portfolio performance graph | Paper trading context | Already mode-agnostic |
| **TradingCharts.tsx** | Price/volume charts | Paper trading context | Already mode-agnostic |
| **TradingSettings.tsx** | Strategy configuration | Paper trading context | Already mode-agnostic |
| **BotSettings.tsx** | Bot-level settings | Paper trading context | Already mode-agnostic |
| **PerSymbolSettings.tsx** | Per-symbol configuration | Paper trading context | Already mode-agnostic |
| **StrategyComparison.tsx** | Multi-strategy comparison | Paper trading context | Already mode-agnostic |
| **StrategyTuningSettings.tsx** | Strategy parameter tuning | Paper trading context | Already mode-agnostic |
| **SystemMonitoring.tsx** | System health monitoring | Paper trading context | Already mode-agnostic |
| **TransactionHistory.tsx** | Trade transaction log | Paper trading context | Needs mode parameter |
| **PortfolioQuickActions.tsx** | Quick action buttons | Paper trading context | Shared component |
| **MobileNav.tsx** | Mobile navigation | Auth context | Shared component - add mode toggle |

---

## 3. Custom Hooks Architecture

### Location
`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/`

### Trading-Related Hooks (9 total)
| Hook | Return Type | Purpose | Redesign Note |
|------|-------------|---------|----------------|
| **usePaperTrading.ts** | PaperTradingState | Paper trading state management, API calls to Rust backend | CRITICAL: Currently paper-only. Need to create parallel useRealTrading or parameterize this |
| **useTradingApi.ts** | Trading API methods | API communication wrapper | Need to support both modes |
| **useTrades.ts** | Trade data | Fetch trades from backend | Need mode parameter |
| **usePositions.ts** | Position data | Current open positions | Need mode parameter |
| **useMarketData.ts** | Market prices | Real-time price data | Already mode-agnostic (market data) |
| **useWebSocket.ts** | WebSocket connection | Real-time data stream | Already mode-agnostic (websocket protocol) |
| **useAccount.ts** | Account info | User account details | Need mode parameter (for real trading with exchange account) |
| **useAIAnalysis.ts** | AI signals | GPT-4 analysis | Already mode-agnostic |
| **use-mobile.tsx** | Boolean | Mobile viewport detection | Shared utility |

### Key Finding
**usePaperTrading.ts** is the single source of truth for paper trading state. Currently returns:
```typescript
{
  portfolio: PortfolioMetrics
  openTrades: PaperTrade[]
  closedTrades: PaperTrade[]
  settings: PaperTradingSettings
  recentSignals: AISignal[]
  isActive: boolean
  isLoading: boolean
  error: string | null
  lastUpdated: Date | null
  startTrading(): Promise<void>
  stopTrading(): Promise<void>
  updateSettings(settings): Promise<void>
  resetPortfolio(): Promise<void>
  closeTrade(id): Promise<void>
  refreshAISignals(): Promise<void>
  refreshSettings(): Promise<void>
}
```

---

## 4. Context Providers Architecture

### Location
`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/`

### Providers (4 total)
| Provider | Purpose | Scope | Redesign Note |
|----------|---------|-------|----------------|
| **PaperTradingContext.tsx** | Shared paper trading state | Global - wraps app | CRITICAL: Need to extend for mode selection |
| **WebSocketContext.tsx** | Real-time data stream | Global - wraps app | Shared component - already handles both modes |
| **AIAnalysisContext.tsx** | AI signal analysis | Global - wraps app | Shared component - mode-agnostic |
| **AuthContext.tsx** | User authentication | Global - wraps app | May need extension for real trading account info |

### Provider Hierarchy (from App.tsx)
```
QueryClientProvider
  └─ AuthProvider
      └─ WebSocketProvider
          └─ AIAnalysisProvider
              └─ PaperTradingProvider
                  └─ TooltipProvider
                      └─ BrowserRouter
```

### Key Finding
**All 4 providers are required**. PaperTradingContext is at the deepest level, making it perfect for mode detection.

---

## 5. Trading Component Library

### Location
`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/`

### Reusable Trading Components (7 files)
| Component | Purpose | Usage | Redesign Note |
|-----------|---------|-------|----------------|
| **OpenPositionsTable.tsx** | Display open trades in table | TradingPaper page | Already mode-agnostic - can be reused |
| **ClosedTradesTable.tsx** | Display closed trades in table | TradingPaper page | Already mode-agnostic - can be reused |
| **PortfolioStats.tsx** | Portfolio metrics display | Dashboard, TradingPaper | Already mode-agnostic - can be reused |
| **RiskMetrics.tsx** | Risk assessment display | TradingPaper settings | Need mode parameter |
| **TradingChartPanel.tsx** | Price chart with indicators | Not currently used | Can be used for real trading |
| **TradingSettingsPanel.tsx** | Strategy settings form | TradingPaper page | Already mode-agnostic - can be reused |
| **types.ts** | TypeScript definitions | All components | Shared types |

---

## 6. UI Component Library

### Location
`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ui/`

### Available Components (50+ Shadcn UI components)
**Form & Input**: Input, Label, Button, Checkbox, Radio, Switch, Select, Textarea, Form, Slider, Input-OTP  
**Display**: Card, Badge, Alert, Skeleton, Progress, Pagination, Breadcrumb, Avatar, Tooltip  
**Layout**: Dialog, Drawer, Sheet, Popover, Collapsible, Tabs, Accordion, Sidebar, Resizable  
**Navigation**: Navigation-Menu, Dropdown-Menu, Context-Menu, Menubar  
**Charts**: Chart (Recharts integration)  
**Visual**: Carousel, Aspect-Ratio, Separator, Scroll-Area, Logo  

### Status
All components are **properly exported and ready for use**. No custom components needed.

---

## 7. Trading Mode Integration Points

### Current State Analysis

#### Paper Trading Only
- **TradingPaper.tsx**: Hardcoded paper trading implementation
- **usePaperTrading.ts**: No mode parameter
- **PaperTradingContext**: No mode selector
- **All dashboard components**: Assume paper trading by default

#### No Real Trading UI
- No live/real trading page
- No exchange account connection UI
- No live portfolio display
- No real market execution interface

### Required Changes for Mode Separation

1. **Add mode detection mechanism**
   - Option A: Route parameter (e.g., `/trading/paper`, `/trading/real`)
   - Option B: Global mode context (new context)
   - Option C: URL query parameter (e.g., `/trading-paper?mode=real`)
   - **RECOMMENDATION**: Option A (cleanest, supports back button)

2. **Refactor TradingPaper.tsx**
   - Extract into mode-agnostic components
   - Create separate real trading page
   - Share common UI patterns

3. **Extend hooks**
   - Create useRealTrading.ts (parallel to usePaperTrading.ts)
   - Or parameterize usePaperTrading() to accept mode

4. **Extend contexts**
   - Create TradingModeContext for mode selection
   - Or extend PaperTradingContext to be more generic

---

## 8. Routing Structure

### Current Routes (from App.tsx)
```
/                     → Landing page
/login                → Login page
/register             → Register page
/dashboard            → Main dashboard (paper trading AI view)
/settings             → Global settings
/trading-paper        → Paper trading interface (LARGEST: 2,148 lines)
/trading-paper-new    → Alternative paper trading (experimental)
/trade-analyses       → Trade history analysis
/how-it-works         → Help/tutorial
/*                    → 404 Not Found
```

### Proposed Routes for Redesign
```
/                     → Landing page (unchanged)
/login                → Login page (unchanged)
/register             → Register page (unchanged)
/dashboard            → Main dashboard (unchanged)
/settings             → Global settings + mode config (extended)
/trading              → Router for both modes
  /trading/paper      → Paper trading interface (refactored TradingPaper.tsx)
  /trading/real       → Real trading interface (NEW - parallel structure)
/trade-analyses       → Trade history (unchanged)
/how-it-works         → Help/tutorial (unchanged)
/*                    → 404 Not Found (unchanged)
```

---

## 9. WebSocket Integration

### Current Implementation
- **useWebSocket.ts**: Establishes connection to backend
- **WebSocketContext**: Provides global access
- **Events**: Real-time price updates, trade execution, portfolio updates
- **Status**: Working in TradingPaper.tsx (connection indicators visible)

### For Redesign
- Already mode-agnostic (doesn't care about paper vs. real)
- Works with both modes automatically
- No changes needed

---

## 10. API Integration Points

### Current Backend Integration
- **Base URL**: `import.meta.env.VITE_RUST_API_URL` (typically `http://localhost:8080`)
- **Endpoints Used**:
  - `GET /api/paper-trading/status` - Get portfolio state
  - `POST /api/paper-trading/start` - Start trading
  - `POST /api/paper-trading/stop` - Stop trading
  - `GET /api/paper-trading/trades` - Get trades
  - `GET /api/paper-trading/settings` - Get settings
  - `PUT /api/paper-trading/settings` - Update settings
  - `POST /api/paper-trading/trades/close` - Close trade
  - `GET /api/paper-trading/symbols` - Get symbol config
  - `PUT /api/paper-trading/symbols` - Update symbol config
  - `GET /api/ai/signals` - Get AI signals
  - WebSocket: `ws://localhost:8080/ws` - Real-time updates

### For Redesign
- Paper trading endpoints are segregated under `/paper-trading/`
- Real trading will need parallel endpoints under `/real-trading/` or similar
- API communication is already abstracted in `useTradingApi.ts`

---

## 11. Component Dependency Map

### TradingPaper.tsx Dependencies
```
TradingPaper
├─ usePaperTradingContext()
├─ useCallback, useEffect, useState, useMemo, memo
├─ Components:
│  ├─ DashboardHeader
│  ├─ ErrorBoundary
│  ├─ Tabs (UI)
│  ├─ Cards (UI)
│  ├─ Tables (UI)
│  ├─ Dialogs (UI)
│  ├─ Buttons (UI)
│  ├─ Badges (UI)
│  └─ ChatBot (lazy-loaded)
└─ Utils:
   ├─ Logger
   └─ Formatters (currency, date, percentage)
```

### Dashboard.tsx Dependencies
```
Dashboard
├─ Components:
│  ├─ DashboardHeader
│  ├─ ErrorBoundary
│  ├─ BotStatus
│  ├─ TradingCharts (lazy)
│  ├─ PerformanceChart (lazy)
│  ├─ AIStrategySelector
│  ├─ AISignals
│  ├─ TransactionHistory
│  └─ ChatBot (lazy)
├─ Contexts:
│  └─ (all auto-provided by App.tsx)
└─ Hooks:
   └─ (all accessed via contexts)
```

---

## 12. State Management Pattern

### Current Pattern
1. **Hooks manage state**: `usePaperTrading()` handles all trading state
2. **Context provides access**: `PaperTradingContext` shares hook instance
3. **Components consume context**: `usePaperTradingContext()` in components
4. **Components never call hooks directly**: Only via context

### For Redesign
- Maintain same pattern for consistency
- Create `useRealTrading()` hook (parallel structure)
- Create `useTradingMode()` hook (mode selector)
- Extend `PaperTradingContext` OR create new `TradingContext` wrapper

---

## 13. File Size & Complexity Analysis

### Largest Files (Refactoring Priority)
| File | Lines | Complexity | Action |
|------|-------|-----------|--------|
| **TradingPaper.tsx** | 2,148 | Very High | REFACTOR: Extract components |
| **usePaperTrading.ts** | 400+ | High | EXTEND: Add mode parameter |
| **PerformanceChart.tsx** | 200+ | Medium | REVIEW: Ensure mode-agnostic |
| **BotStatus.tsx** | 180+ | Medium | REVIEW: Ensure mode-agnostic |
| **TransactionHistory.tsx** | 150+ | Medium | REVIEW: Ensure mode-agnostic |

### Critical Refactoring Target
**TradingPaper.tsx needs to be split into:**
1. Mode-agnostic trading container
2. Paper trading specific view
3. Real trading specific view (NEW)
4. Shared portfolio display components
5. Shared settings components

---

## 14. Testing Coverage

### Test Files Location
`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/`

### Test Files (25 total)
- `__tests__/pages/TradingPaper.test.tsx` - Paper trading page tests
- `__tests__/components/dashboard/*.test.tsx` - Dashboard component tests
- `__tests__/hooks/usePaperTrading.test.ts` - Hook tests
- `__tests__/contexts/AuthContext.test.tsx` - Context tests

### For Redesign
- Create new test files for:
  - Real trading page
  - Trading mode selector
  - Shared components (mode-agnostic)
- Update existing tests to support both modes

---

## 15. Styling & Theme

### Tech Stack
- **CSS Framework**: TailwindCSS
- **Component Library**: Shadcn/UI
- **Colors**: Custom CSS variables (in index.css)
- **Dark Mode**: Supported

### Key Styles Used
- `.text-profit` (green) - Positive P&L
- `.text-loss` (red) - Negative P&L
- `.text-warning` (yellow) - Warnings/Caution
- `.text-info` (blue) - Information
- `.animate-pulse` - Loading/live updates
- `.border-green-500/20` - Live status indicators

### For Redesign
- All existing styles can be reused
- No theme changes needed
- Same color system for both modes

---

## 16. Key Findings & Recommendations

### Findings
1. **Paper trading is isolated** - No real trading exists, making redesign greenfield for real mode
2. **TradingPaper.tsx is monolithic** - 2,148 lines contain ALL paper trading logic
3. **Context pattern is clean** - Easy to extend with mode selection
4. **Dashboard is mode-agnostic** - Already works with any trading mode data
5. **No existing mode separation** - Every component assumes paper trading
6. **Hooks are well-structured** - Easy to create parallel real trading hooks
7. **UI components are reusable** - 50+ Shadcn components available
8. **WebSocket is already shared** - Works for both modes automatically

### Recommendations for Redesign

#### Phase 1: Architecture Setup
1. Create new route: `/trading/paper` and `/trading/real`
2. Create `useTradingMode()` context hook
3. Create `useRealTrading()` hook (parallel to `usePaperTrading()`)
4. Create `TradingModeContext` for global mode selection

#### Phase 2: Component Extraction
1. **Extract from TradingPaper.tsx** (2,148 lines):
   - Portfolio stats card component
   - Trade table component (reusable)
   - Settings form component (reusable)
   - Performance chart component (already exists)
   - AI signals component (already exists, shared)

2. **Create new shared components**:
   - `<ModeToggle />` - Switch between paper/real
   - `<TradingModeRouter />` - Route wrapper

#### Phase 3: Implementation
1. Refactor TradingPaper.tsx into reusable components
2. Create TradingReal.tsx (mirror structure)
3. Create unified Trading.tsx router component
4. Extend hooks to support mode parameter

#### Phase 4: Integration
1. Update navigation to show mode toggle
2. Add mode parameter to all trading-related hooks
3. Update tests for both modes
4. Documentation and guides

---

## 17. Directory Tree (Relevant Files)

```
nextjs-ui-dashboard/src/
├── pages/
│   ├── Dashboard.tsx                 [Mode-agnostic, needs nav update]
│   ├── TradingPaper.tsx              [CRITICAL - 2,148 lines, paper only]
│   ├── TradingPaperNew.tsx           [Experimental, consolidate]
│   ├── Settings.tsx                  [Extend for mode config]
│   ├── TradeAnalyses.tsx             [Add mode parameter]
│   ├── Index.tsx, Login.tsx, Register.tsx, HowItWorks.tsx, NotFound.tsx
│
├── components/
│   ├── dashboard/                    [16 components, mostly mode-agnostic]
│   │   ├── DashboardHeader.tsx       [Add mode toggle]
│   │   ├── BotStatus.tsx
│   │   ├── AISignals.tsx
│   │   ├── PerformanceChart.tsx
│   │   ├── TradingCharts.tsx
│   │   ├── TradingSettings.tsx
│   │   ├── BotSettings.tsx
│   │   └── [13 more...]
│   ├── trading/                      [7 components, mode-agnostic]
│   │   ├── OpenPositionsTable.tsx
│   │   ├── ClosedTradesTable.tsx
│   │   ├── PortfolioStats.tsx
│   │   ├── RiskMetrics.tsx
│   │   ├── TradingChartPanel.tsx
│   │   ├── TradingSettingsPanel.tsx
│   │   └── types.ts
│   ├── ui/                           [50+ Shadcn components]
│   ├── ProtectedRoute.tsx
│   ├── ErrorBoundary.tsx
│   ├── TradingInterface.tsx           [Stub, can be extended]
│   ├── ChatBot.tsx
│   └── [landing/, ai/, settings/ components...]
│
├── hooks/
│   ├── usePaperTrading.ts            [Paper only, needs extension]
│   ├── useRealTrading.ts             [NEW - create parallel]
│   ├── useTradingMode.ts             [NEW - mode selector]
│   ├── useTradingApi.ts
│   ├── useTrades.ts                  [Needs mode parameter]
│   ├── usePositions.ts               [Needs mode parameter]
│   ├── useWebSocket.ts               [Shared, mode-agnostic]
│   ├── useMarketData.ts              [Shared, mode-agnostic]
│   ├── useAccount.ts                 [Needs mode parameter]
│   ├── useAIAnalysis.ts              [Shared, mode-agnostic]
│   └── [other utility hooks...]
│
├── contexts/
│   ├── PaperTradingContext.tsx       [Extend or replace]
│   ├── TradingModeContext.tsx        [NEW - mode selector]
│   ├── WebSocketContext.tsx          [Shared, mode-agnostic]
│   ├── AIAnalysisContext.tsx         [Shared, mode-agnostic]
│   └── AuthContext.tsx               [Extend for real trading]
│
├── App.tsx                            [Add mode route]
├── main.tsx
├── index.css
└── [other utility files...]
```

---

## Summary Table: Component Redesign Impact

| Component | Impact | Action | Priority |
|-----------|--------|--------|----------|
| **TradingPaper.tsx** | CRITICAL | Refactor into mode-agnostic + mode-specific | P0 |
| **usePaperTrading.ts** | HIGH | Create useRealTrading parallel or parameterize | P0 |
| **PaperTradingContext** | HIGH | Create TradingModeContext or extend | P0 |
| **DashboardHeader** | MEDIUM | Add mode toggle button | P1 |
| **Dashboard.tsx** | LOW | Update nav only | P1 |
| **TradingCharts, PerformanceChart** | LOW | Already mode-agnostic | P2 |
| **PortfolioStats, RiskMetrics** | LOW | Already mode-agnostic | P2 |
| **OpenPositionsTable, ClosedTradesTable** | LOW | Already mode-agnostic | P2 |
| **Settings.tsx** | LOW | Extend with mode config | P2 |

---

## Conclusion

The frontend is **well-structured for expansion**. The main work involves:

1. **Creating parallel real trading hooks** (low effort - copy + modify usePaperTrading)
2. **Extracting reusable components** from TradingPaper.tsx (medium effort - ~5-8 new components)
3. **Adding routing** for mode selection (low effort - simple route wrapper)
4. **Extending contexts** for mode management (low effort - simple context provider)
5. **Testing both modes** (medium effort - duplicate tests for real mode)

**Total Redesign Effort**: 15-20% of frontend rewrite, 80% component reuse from paper trading.

