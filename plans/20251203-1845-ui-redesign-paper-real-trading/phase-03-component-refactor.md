# Phase 3: Component Extraction & Refactoring

**Status**: Pending | **Priority**: P0 | **Est. Time**: 3-4 days

---

## Context

- [Main Plan](./plan.md)
- [Phase 2: Mode Infrastructure](./phase-02-mode-infrastructure.md)
- [Frontend Structure Scout](./scout/scout-01-frontend-structure.md)

## Overview

Extract reusable components from TradingPaper.tsx (2,148 lines) into mode-agnostic shared components. Target: 80%+ code reuse between Paper and Real trading interfaces.

## Key Insights

1. **TradingPaper.tsx is monolithic** - Contains ALL trading logic in one file
2. **Most components are mode-agnostic** - Portfolio display, charts, tables can be shared
3. **Mode-specific elements minimal** - Badges, warnings, confirmation flows only
4. **Existing trading/ components underutilized** - Can extend PortfolioStats, RiskMetrics

## Requirements

### Functional
- [ ] Extract 8-10 reusable components from TradingPaper.tsx
- [ ] Create shared TradingDashboard layout
- [ ] Mode-agnostic portfolio display
- [ ] Mode-agnostic trade tables
- [ ] Mode-specific warning banners

### Non-Functional
- [ ] TradingPaper.tsx reduced to <500 lines
- [ ] 80%+ component reuse for TradingReal
- [ ] No functionality regression
- [ ] Maintain existing test coverage

## Architecture

### Component Extraction Map

**FROM TradingPaper.tsx (2,148 lines):**

```
TradingPaper.tsx
├── Header section (lines ~50-150)
│   └── Extract → TradingHeader.tsx
├── Portfolio overview (lines ~150-350)
│   └── Extract → PortfolioOverview.tsx
├── Open positions table (lines ~350-600)
│   └── Extract → PositionsTable.tsx (enhance existing)
├── Closed trades table (lines ~600-850)
│   └── Extract → TradesHistoryTable.tsx (enhance existing)
├── Settings panel (lines ~850-1200)
│   └── Extract → TradingSettingsPanel.tsx (enhance existing)
├── AI Signals section (lines ~1200-1500)
│   └── Use existing → AISignals.tsx
├── Performance chart (lines ~1500-1700)
│   └── Use existing → PerformanceChart.tsx
└── Quick actions (lines ~1700-2148)
    └── Extract → QuickActionsPanel.tsx
```

### New Shared Components

```
src/components/trading/
├── shared/
│   ├── TradingHeader.tsx           # Mode badge, nav, status
│   ├── PortfolioOverview.tsx       # Balance, equity, P&L cards
│   ├── PositionsTable.tsx          # Open positions (enhanced)
│   ├── TradesHistoryTable.tsx      # Closed trades (enhanced)
│   ├── QuickActionsPanel.tsx       # Start/stop, close all
│   ├── TradingDashboardLayout.tsx  # Grid layout wrapper
│   └── WarningBanner.tsx           # Mode-specific warnings
├── paper/
│   └── PaperModeIndicator.tsx      # "SANDBOX" badge + styling
└── real/
    ├── RealModeWarning.tsx         # Persistent warning banner
    └── TradeConfirmation.tsx       # 2-step confirmation dialog
```

### Component Interface Examples

```typescript
// PortfolioOverview.tsx - Mode-agnostic
interface PortfolioOverviewProps {
  balance: number
  equity: number
  dailyPnL: number
  dailyPnLPercent: number
  totalPnL: number
  openPositions: number
  mode: 'paper' | 'real'  // For styling only
}

// PositionsTable.tsx - Mode-agnostic
interface PositionsTableProps {
  positions: Position[]
  onClosePosition: (id: string) => Promise<void>
  onCloseAll: () => Promise<void>
  showRiskWarnings?: boolean  // Real mode shows more warnings
  isLoading: boolean
}

// TradingHeader.tsx - Mode-aware
interface TradingHeaderProps {
  mode: 'paper' | 'real'
  isActive: boolean
  onToggleActive: () => void
  lastUpdated: Date | null
}
```

## Related Files

| Current File | Path | Action |
|--------------|------|--------|
| TradingPaper.tsx | `/nextjs-ui-dashboard/src/pages/TradingPaper.tsx` | Refactor (2,148 → <500 lines) |
| OpenPositionsTable | `/nextjs-ui-dashboard/src/components/trading/OpenPositionsTable.tsx` | Enhance |
| ClosedTradesTable | `/nextjs-ui-dashboard/src/components/trading/ClosedTradesTable.tsx` | Enhance |
| PortfolioStats | `/nextjs-ui-dashboard/src/components/trading/PortfolioStats.tsx` | Enhance |
| TradingSettingsPanel | `/nextjs-ui-dashboard/src/components/trading/TradingSettingsPanel.tsx` | Enhance |

## Implementation Steps

### Step 1: Create TradingDashboardLayout

```typescript
// src/components/trading/shared/TradingDashboardLayout.tsx
import { motion } from 'framer-motion'
import { TradingHeader } from './TradingHeader'
import { WarningBanner } from './WarningBanner'

interface Props {
  mode: 'paper' | 'real'
  children: React.ReactNode
  header: React.ReactNode
  sidebar?: React.ReactNode
}

export function TradingDashboardLayout({ mode, children, header, sidebar }: Props) {
  return (
    <div className="min-h-screen bg-slate-950">
      {mode === 'real' && <WarningBanner />}

      <div className="container mx-auto px-4 py-6">
        {header}

        <motion.div
          initial={{ opacity: 0, y: 20 }}
          animate={{ opacity: 1, y: 0 }}
          className="grid grid-cols-1 lg:grid-cols-4 gap-6 mt-6"
        >
          <div className="lg:col-span-3 space-y-6">
            {children}
          </div>
          {sidebar && (
            <div className="space-y-6">
              {sidebar}
            </div>
          )}
        </motion.div>
      </div>
    </div>
  )
}
```

### Step 2: Create PortfolioOverview Component

```typescript
// src/components/trading/shared/PortfolioOverview.tsx
import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card'
import { AnimatedNumber } from '@/components/ui/AnimatedNumber'
import { TrendingUp, TrendingDown, Wallet, BarChart3 } from 'lucide-react'
import { cn } from '@/lib/utils'

export function PortfolioOverview({
  balance,
  equity,
  dailyPnL,
  dailyPnLPercent,
  totalPnL,
  openPositions,
  mode
}: PortfolioOverviewProps) {
  const isProfitable = dailyPnL >= 0

  return (
    <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
      <Card className="bg-slate-900/50 border-slate-800">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm text-slate-400 flex items-center gap-2">
            <Wallet className="w-4 h-4" />
            Balance
          </CardTitle>
        </CardHeader>
        <CardContent>
          <AnimatedNumber value={balance} className="text-2xl font-bold" />
        </CardContent>
      </Card>

      <Card className="bg-slate-900/50 border-slate-800">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm text-slate-400 flex items-center gap-2">
            <BarChart3 className="w-4 h-4" />
            Equity
          </CardTitle>
        </CardHeader>
        <CardContent>
          <AnimatedNumber value={equity} className="text-2xl font-bold" />
        </CardContent>
      </Card>

      <Card className={cn(
        "bg-slate-900/50 border-slate-800",
        isProfitable ? "border-l-4 border-l-profit" : "border-l-4 border-l-loss"
      )}>
        <CardHeader className="pb-2">
          <CardTitle className="text-sm text-slate-400 flex items-center gap-2">
            {isProfitable ? <TrendingUp className="w-4 h-4 text-profit" /> : <TrendingDown className="w-4 h-4 text-loss" />}
            Daily P&L
          </CardTitle>
        </CardHeader>
        <CardContent>
          <AnimatedNumber
            value={dailyPnL}
            className={cn("text-2xl font-bold", isProfitable ? "text-profit" : "text-loss")}
          />
          <span className={cn("text-sm ml-2", isProfitable ? "text-profit" : "text-loss")}>
            ({dailyPnLPercent.toFixed(2)}%)
          </span>
        </CardContent>
      </Card>

      <Card className="bg-slate-900/50 border-slate-800">
        <CardHeader className="pb-2">
          <CardTitle className="text-sm text-slate-400">Open Positions</CardTitle>
        </CardHeader>
        <CardContent>
          <span className="text-2xl font-bold">{openPositions}</span>
        </CardContent>
      </Card>
    </div>
  )
}
```

### Step 3: Create TradingHeader Component

```typescript
// src/components/trading/shared/TradingHeader.tsx
import { ModeSwitcher } from '../ModeSwitcher'
import { ModeBadge } from '@/components/ui/ModeBadge'
import { Button } from '@/components/ui/button'
import { Play, Pause, RefreshCw } from 'lucide-react'

export function TradingHeader({
  mode,
  isActive,
  onToggleActive,
  onRefresh,
  lastUpdated
}: TradingHeaderProps) {
  return (
    <div className="flex items-center justify-between">
      <div className="flex items-center gap-4">
        <h1 className="text-2xl font-bold">
          {mode === 'paper' ? 'Paper Trading' : 'Live Trading'}
        </h1>
        <ModeBadge mode={mode} />
      </div>

      <div className="flex items-center gap-4">
        <ModeSwitcher />

        <Button
          variant={isActive ? 'destructive' : 'default'}
          onClick={onToggleActive}
          className="gap-2"
        >
          {isActive ? <Pause className="w-4 h-4" /> : <Play className="w-4 h-4" />}
          {isActive ? 'Stop Trading' : 'Start Trading'}
        </Button>

        <Button variant="outline" size="icon" onClick={onRefresh}>
          <RefreshCw className="w-4 h-4" />
        </Button>

        {lastUpdated && (
          <span className="text-sm text-slate-400">
            Last updated: {lastUpdated.toLocaleTimeString()}
          </span>
        )}
      </div>
    </div>
  )
}
```

### Step 4: Create WarningBanner Component

```typescript
// src/components/trading/shared/WarningBanner.tsx
import { AlertTriangle, X } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'
import { useState } from 'react'

export function WarningBanner() {
  const [isDismissed, setIsDismissed] = useState(false)

  // Show again on next session (localStorage with daily reset)
  const shouldShow = !isDismissed

  return (
    <AnimatePresence>
      {shouldShow && (
        <motion.div
          initial={{ height: 0, opacity: 0 }}
          animate={{ height: 'auto', opacity: 1 }}
          exit={{ height: 0, opacity: 0 }}
          className="bg-real-warning/10 border-b border-real-warning/30"
        >
          <div className="container mx-auto px-4 py-3 flex items-center justify-between">
            <div className="flex items-center gap-3">
              <AlertTriangle className="w-5 h-5 text-real-warning animate-pulse" />
              <span className="text-real-warning font-semibold">
                REAL MONEY AT RISK - All trades execute with real funds
              </span>
            </div>
            <button
              onClick={() => setIsDismissed(true)}
              className="text-slate-400 hover:text-white"
            >
              <X className="w-4 h-4" />
            </button>
          </div>
        </motion.div>
      )}
    </AnimatePresence>
  )
}
```

### Step 5: Enhance PositionsTable

```typescript
// Enhance existing src/components/trading/OpenPositionsTable.tsx
// Add mode parameter and conditional risk warnings

export function PositionsTable({
  positions,
  onClosePosition,
  onCloseAll,
  showRiskWarnings = false,
  isLoading
}: PositionsTableProps) {
  // ... existing implementation with added risk warnings for real mode
}
```

### Step 6: Refactor TradingPaper.tsx

After extracting components, TradingPaper.tsx becomes:

```typescript
// src/pages/TradingPaper.tsx (Refactored: <500 lines)
import { TradingDashboardLayout } from '@/components/trading/shared/TradingDashboardLayout'
import { TradingHeader } from '@/components/trading/shared/TradingHeader'
import { PortfolioOverview } from '@/components/trading/shared/PortfolioOverview'
import { PositionsTable } from '@/components/trading/shared/PositionsTable'
import { TradesHistoryTable } from '@/components/trading/shared/TradesHistoryTable'
import { usePaperTradingContext } from '@/contexts/PaperTradingContext'

export default function TradingPaper() {
  const {
    portfolio,
    openTrades,
    closedTrades,
    isActive,
    startTrading,
    stopTrading,
    closeTrade,
    lastUpdated
  } = usePaperTradingContext()

  return (
    <TradingDashboardLayout mode="paper" header={
      <TradingHeader
        mode="paper"
        isActive={isActive}
        onToggleActive={isActive ? stopTrading : startTrading}
        lastUpdated={lastUpdated}
      />
    }>
      <PortfolioOverview
        balance={portfolio.balance}
        equity={portfolio.equity}
        dailyPnL={portfolio.dailyPnL}
        dailyPnLPercent={portfolio.dailyPnLPercent}
        totalPnL={portfolio.totalPnL}
        openPositions={openTrades.length}
        mode="paper"
      />

      <Tabs defaultValue="positions">
        <TabsList>
          <TabsTrigger value="positions">Open Positions</TabsTrigger>
          <TabsTrigger value="history">Trade History</TabsTrigger>
          <TabsTrigger value="settings">Settings</TabsTrigger>
        </TabsList>

        <TabsContent value="positions">
          <PositionsTable
            positions={openTrades}
            onClosePosition={closeTrade}
            showRiskWarnings={false}
          />
        </TabsContent>

        <TabsContent value="history">
          <TradesHistoryTable trades={closedTrades} />
        </TabsContent>

        <TabsContent value="settings">
          <TradingSettingsPanel mode="paper" />
        </TabsContent>
      </Tabs>
    </TradingDashboardLayout>
  )
}
```

## Todo List

- [ ] Create shared/ directory structure
- [ ] Create TradingDashboardLayout.tsx
- [ ] Create TradingHeader.tsx
- [ ] Create PortfolioOverview.tsx
- [ ] Create WarningBanner.tsx
- [ ] Create QuickActionsPanel.tsx
- [ ] Enhance OpenPositionsTable.tsx
- [ ] Enhance ClosedTradesTable.tsx
- [ ] Enhance TradingSettingsPanel.tsx
- [ ] Refactor TradingPaper.tsx to use new components
- [ ] Verify all existing functionality preserved
- [ ] Update imports across codebase
- [ ] Write component tests
- [ ] Update Storybook stories (if exists)

## Success Criteria

1. TradingPaper.tsx reduced from 2,148 to <500 lines
2. All 8+ extracted components working
3. No functionality regression
4. All existing tests pass
5. Components documented with TypeScript interfaces

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Breaking existing functionality | Medium | High | Comprehensive test coverage |
| State management issues | Medium | Medium | Maintain context patterns |
| Performance regression | Low | Medium | Profile before/after |

## Security Considerations

- No new security concerns from component extraction
- Maintain existing data sanitization in tables
- Keep error boundaries around components

## Next Steps

After Phase 3 completion, proceed to [Phase 4: Real Trading UI](./phase-04-real-trading-ui.md)
