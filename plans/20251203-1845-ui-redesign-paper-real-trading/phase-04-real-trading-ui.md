# Phase 4: Real Trading UI Implementation

**Status**: Pending | **Priority**: P0 | **Est. Time**: 3-4 days

---

## Context

- [Main Plan](./plan.md)
- [Phase 3: Component Refactor](./phase-03-component-refactor.md)
- [Trading UI Patterns Research](./research/researcher-01-trading-ui-patterns.md)

## Overview

Build complete Real Trading dashboard with safety-first design: persistent warnings, 2-step confirmations, strict validation, and clear visual distinction from Paper mode.

## Key Insights

1. **2-Step Trade Confirmation Required** - Preview order details, then confirm with checkbox
2. **Persistent Warning Banner** - "REAL MONEY AT RISK" always visible
3. **Red/Warning Color Scheme** - Distinct from Paper's blue theme
4. **Fee + Slippage Preview** - Show estimated costs before execution
5. **Daily Loss Limit UI** - Prominent display of remaining risk budget

## Requirements

### Functional
- [ ] TradingReal.tsx page (mirrors TradingPaper structure)
- [ ] TradeConfirmationDialog with 2-step flow
- [ ] Real-time fee estimation display
- [ ] Daily loss limit indicator
- [ ] Exchange connection status
- [ ] Position close confirmation
- [ ] Order preview before execution

### Non-Functional
- [ ] No accidental trade execution possible
- [ ] Clear visual hierarchy (warnings prominent)
- [ ] Mobile-responsive warning banners
- [ ] Accessible (WCAG AA)

## Architecture

### Real Trading Page Structure

```
TradingReal.tsx
├── WarningBanner (persistent, red)
├── TradingHeader (mode="real")
├── ExchangeConnectionStatus
├── PortfolioOverview (with real balance)
├── DailyLossLimitIndicator
├── Tabs
│   ├── Open Positions (with close confirmation)
│   ├── Order History
│   ├── Pending Orders
│   └── Settings (risk limits)
└── TradeConfirmationDialog (2-step)
```

### Safety Components

```
src/components/trading/real/
├── RealModeWarning.tsx           # Persistent banner
├── TradeConfirmationDialog.tsx   # 2-step confirmation
├── FeeEstimateCard.tsx           # Fee/slippage preview
├── DailyLossLimitBar.tsx         # Visual progress bar
├── ExchangeStatus.tsx            # Connection indicator
├── ClosePositionDialog.tsx       # Confirm position close
└── OrderPreview.tsx              # Order details before submit
```

### TradeConfirmationDialog Flow

```
Step 1: Preview
┌─────────────────────────────────────┐
│  Order Preview                      │
│  ─────────────────                  │
│  Symbol: BTCUSDT                    │
│  Side: BUY                          │
│  Quantity: 0.01 BTC                 │
│  Est. Price: $67,432.50             │
│  ─────────────────                  │
│  Est. Fee: $3.37 (0.05%)            │
│  Est. Slippage: $6.74 (0.01%)       │
│  Total Cost: ~$684.72               │
│  ─────────────────                  │
│  [ ] I understand this uses REAL    │
│      MONEY and cannot be undone     │
│                                     │
│  [Cancel]        [Review Order →]   │
└─────────────────────────────────────┘

Step 2: Confirm
┌─────────────────────────────────────┐
│  ⚠️ Confirm Real Trade              │
│  ─────────────────                  │
│  You are about to execute:          │
│                                     │
│  BUY 0.01 BTC @ ~$67,432.50         │
│                                     │
│  This action is IRREVERSIBLE        │
│                                     │
│  [← Back]    [Execute Trade]        │
└─────────────────────────────────────┘
```

## Related Files

| File | Path | Action |
|------|------|--------|
| TradingReal | `/nextjs-ui-dashboard/src/pages/TradingReal.tsx` | Create |
| TradeConfirmationDialog | `/nextjs-ui-dashboard/src/components/trading/real/TradeConfirmationDialog.tsx` | Create |
| RealModeWarning | `/nextjs-ui-dashboard/src/components/trading/real/RealModeWarning.tsx` | Create |
| DailyLossLimitBar | `/nextjs-ui-dashboard/src/components/trading/real/DailyLossLimitBar.tsx` | Create |
| ExchangeStatus | `/nextjs-ui-dashboard/src/components/trading/real/ExchangeStatus.tsx` | Create |
| ClosePositionDialog | `/nextjs-ui-dashboard/src/components/trading/real/ClosePositionDialog.tsx` | Create |
| OrderPreview | `/nextjs-ui-dashboard/src/components/trading/real/OrderPreview.tsx` | Create |
| FeeEstimateCard | `/nextjs-ui-dashboard/src/components/trading/real/FeeEstimateCard.tsx` | Create |

## Implementation Steps

### Step 1: Create TradeConfirmationDialog

```typescript
// src/components/trading/real/TradeConfirmationDialog.tsx
import { useState } from 'react'
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { Checkbox } from '@/components/ui/checkbox'
import { AlertTriangle, ArrowRight, ArrowLeft } from 'lucide-react'
import { motion, AnimatePresence } from 'framer-motion'

interface Props {
  isOpen: boolean
  order: OrderPreview
  onConfirm: () => Promise<void>
  onCancel: () => void
  isSubmitting: boolean
}

export function TradeConfirmationDialog({
  isOpen,
  order,
  onConfirm,
  onCancel,
  isSubmitting
}: Props) {
  const [step, setStep] = useState<1 | 2>(1)
  const [acknowledged, setAcknowledged] = useState(false)

  const handleConfirm = async () => {
    if (step === 1) {
      setStep(2)
    } else {
      await onConfirm()
    }
  }

  const handleCancel = () => {
    if (step === 2) {
      setStep(1)
    } else {
      onCancel()
      setAcknowledged(false)
      setStep(1)
    }
  }

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && handleCancel()}>
      <DialogContent className="max-w-md bg-slate-900 border-real-warning/30">
        <DialogHeader>
          <div className="flex items-center gap-3 mb-4">
            <AlertTriangle className="w-8 h-8 text-real-warning" />
            <DialogTitle>
              {step === 1 ? 'Order Preview' : 'Confirm Real Trade'}
            </DialogTitle>
          </div>
        </DialogHeader>

        <AnimatePresence mode="wait">
          {step === 1 ? (
            <motion.div
              key="step1"
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: -20 }}
              className="space-y-4"
            >
              {/* Order Details */}
              <div className="bg-slate-800 rounded-lg p-4 space-y-2">
                <div className="flex justify-between">
                  <span className="text-slate-400">Symbol</span>
                  <span className="font-mono">{order.symbol}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-400">Side</span>
                  <span className={order.side === 'BUY' ? 'text-profit' : 'text-loss'}>
                    {order.side}
                  </span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-400">Quantity</span>
                  <span>{order.quantity}</span>
                </div>
                <div className="flex justify-between">
                  <span className="text-slate-400">Est. Price</span>
                  <span>${order.estimatedPrice.toLocaleString()}</span>
                </div>
              </div>

              {/* Fee Estimate */}
              <div className="bg-slate-800/50 rounded-lg p-4 space-y-2 border border-slate-700">
                <div className="flex justify-between text-sm">
                  <span className="text-slate-400">Est. Fee</span>
                  <span>${order.estimatedFee.toFixed(2)} ({order.feePercent}%)</span>
                </div>
                <div className="flex justify-between text-sm">
                  <span className="text-slate-400">Est. Slippage</span>
                  <span>${order.estimatedSlippage.toFixed(2)}</span>
                </div>
                <div className="flex justify-between font-semibold border-t border-slate-700 pt-2 mt-2">
                  <span>Total Cost</span>
                  <span>~${order.totalCost.toLocaleString()}</span>
                </div>
              </div>

              {/* Acknowledgment */}
              <div className="flex items-start gap-3 p-3 bg-real-warning/10 rounded-lg border border-real-warning/30">
                <Checkbox
                  id="acknowledge"
                  checked={acknowledged}
                  onCheckedChange={(checked) => setAcknowledged(!!checked)}
                />
                <label htmlFor="acknowledge" className="text-sm text-slate-300 cursor-pointer">
                  I understand this uses <strong className="text-real-warning">REAL MONEY</strong>{' '}
                  and cannot be undone
                </label>
              </div>
            </motion.div>
          ) : (
            <motion.div
              key="step2"
              initial={{ opacity: 0, x: 20 }}
              animate={{ opacity: 1, x: 0 }}
              exit={{ opacity: 0, x: 20 }}
              className="space-y-4"
            >
              <div className="text-center py-4">
                <p className="text-lg">You are about to execute:</p>
                <p className="text-2xl font-bold mt-2">
                  <span className={order.side === 'BUY' ? 'text-profit' : 'text-loss'}>
                    {order.side}
                  </span>{' '}
                  {order.quantity} {order.symbol.replace('USDT', '')}
                </p>
                <p className="text-slate-400 mt-1">@ ~${order.estimatedPrice.toLocaleString()}</p>
              </div>

              <div className="p-4 bg-real-warning/20 rounded-lg border border-real-warning text-center">
                <p className="text-real-warning font-semibold">
                  This action is IRREVERSIBLE
                </p>
              </div>
            </motion.div>
          )}
        </AnimatePresence>

        <div className="flex gap-4 mt-6">
          <Button variant="outline" onClick={handleCancel} className="flex-1">
            {step === 2 ? <ArrowLeft className="w-4 h-4 mr-2" /> : null}
            {step === 1 ? 'Cancel' : 'Back'}
          </Button>
          <Button
            variant="destructive"
            onClick={handleConfirm}
            disabled={step === 1 && !acknowledged || isSubmitting}
            className="flex-1"
          >
            {isSubmitting ? 'Executing...' : step === 1 ? (
              <>Review Order <ArrowRight className="w-4 h-4 ml-2" /></>
            ) : (
              'Execute Trade'
            )}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
```

### Step 2: Create DailyLossLimitBar

```typescript
// src/components/trading/real/DailyLossLimitBar.tsx
import { Progress } from '@/components/ui/progress'
import { AlertTriangle } from 'lucide-react'
import { cn } from '@/lib/utils'

interface Props {
  currentLoss: number
  dailyLimit: number
  currency?: string
}

export function DailyLossLimitBar({ currentLoss, dailyLimit, currency = 'USD' }: Props) {
  const usedPercent = Math.min((Math.abs(currentLoss) / dailyLimit) * 100, 100)
  const remaining = dailyLimit - Math.abs(currentLoss)
  const isWarning = usedPercent >= 70
  const isCritical = usedPercent >= 90

  return (
    <div className={cn(
      "p-4 rounded-lg border",
      isCritical ? "bg-real-warning/20 border-real-warning" :
      isWarning ? "bg-yellow-500/10 border-yellow-500/30" :
      "bg-slate-800 border-slate-700"
    )}>
      <div className="flex items-center justify-between mb-2">
        <div className="flex items-center gap-2">
          {(isWarning || isCritical) && (
            <AlertTriangle className={cn(
              "w-4 h-4",
              isCritical ? "text-real-warning" : "text-yellow-500"
            )} />
          )}
          <span className="text-sm font-medium">Daily Loss Limit</span>
        </div>
        <span className={cn(
          "text-sm font-mono",
          isCritical ? "text-real-warning" : isWarning ? "text-yellow-500" : "text-slate-400"
        )}>
          ${remaining.toFixed(2)} remaining
        </span>
      </div>

      <Progress
        value={usedPercent}
        className={cn(
          "h-2",
          isCritical ? "[&>div]:bg-real-warning" :
          isWarning ? "[&>div]:bg-yellow-500" :
          "[&>div]:bg-profit"
        )}
      />

      <div className="flex justify-between mt-1 text-xs text-slate-500">
        <span>${Math.abs(currentLoss).toFixed(2)} used</span>
        <span>${dailyLimit.toFixed(2)} limit</span>
      </div>
    </div>
  )
}
```

### Step 3: Create ExchangeStatus Component

```typescript
// src/components/trading/real/ExchangeStatus.tsx
import { Badge } from '@/components/ui/badge'
import { Wifi, WifiOff, AlertCircle } from 'lucide-react'
import { cn } from '@/lib/utils'

type Status = 'connected' | 'disconnected' | 'error' | 'connecting'

interface Props {
  status: Status
  exchangeName: string
  latency?: number
}

export function ExchangeStatus({ status, exchangeName, latency }: Props) {
  const statusConfig = {
    connected: { icon: Wifi, color: 'text-profit', bg: 'bg-profit/20', label: 'Connected' },
    disconnected: { icon: WifiOff, color: 'text-slate-400', bg: 'bg-slate-700', label: 'Disconnected' },
    error: { icon: AlertCircle, color: 'text-real-warning', bg: 'bg-real-warning/20', label: 'Error' },
    connecting: { icon: Wifi, color: 'text-yellow-500', bg: 'bg-yellow-500/20', label: 'Connecting...' }
  }

  const config = statusConfig[status]
  const Icon = config.icon

  return (
    <div className={cn("flex items-center gap-2 px-3 py-1.5 rounded-lg", config.bg)}>
      <Icon className={cn("w-4 h-4", config.color, status === 'connecting' && 'animate-pulse')} />
      <span className="text-sm font-medium">{exchangeName}</span>
      <Badge variant="outline" className={cn("text-xs", config.color)}>
        {config.label}
      </Badge>
      {latency !== undefined && status === 'connected' && (
        <span className="text-xs text-slate-400">{latency}ms</span>
      )}
    </div>
  )
}
```

### Step 4: Create TradingReal Page

```typescript
// src/pages/TradingReal.tsx
import { TradingDashboardLayout } from '@/components/trading/shared/TradingDashboardLayout'
import { TradingHeader } from '@/components/trading/shared/TradingHeader'
import { PortfolioOverview } from '@/components/trading/shared/PortfolioOverview'
import { PositionsTable } from '@/components/trading/shared/PositionsTable'
import { DailyLossLimitBar } from '@/components/trading/real/DailyLossLimitBar'
import { ExchangeStatus } from '@/components/trading/real/ExchangeStatus'
import { TradeConfirmationDialog } from '@/components/trading/real/TradeConfirmationDialog'
import { useRealTrading } from '@/hooks/useRealTrading'
import { useTradingMode } from '@/contexts/TradingModeContext'
import { Navigate } from 'react-router-dom'

export default function TradingReal() {
  const { isRealMode } = useTradingMode()
  const {
    portfolio,
    openPositions,
    orderHistory,
    exchangeStatus,
    dailyPnL,
    dailyLossLimit,
    placeOrder,
    closePosition,
    isLoading,
    lastUpdated
  } = useRealTrading()

  const [confirmDialog, setConfirmDialog] = useState<OrderPreview | null>(null)

  // Redirect if not in real mode
  if (!isRealMode) {
    return <Navigate to="/trading/paper" replace />
  }

  const handlePlaceOrder = async (order: OrderRequest) => {
    // Show confirmation dialog first
    const preview = await calculateOrderPreview(order)
    setConfirmDialog(preview)
  }

  const handleConfirmOrder = async () => {
    if (!confirmDialog) return
    await placeOrder(confirmDialog)
    setConfirmDialog(null)
  }

  return (
    <TradingDashboardLayout mode="real" header={
      <div className="space-y-4">
        <TradingHeader
          mode="real"
          isActive={true}
          onToggleActive={() => {}}
          lastUpdated={lastUpdated}
        />
        <div className="flex items-center gap-4">
          <ExchangeStatus
            status={exchangeStatus}
            exchangeName="Binance"
            latency={45}
          />
          <DailyLossLimitBar
            currentLoss={dailyPnL}
            dailyLimit={dailyLossLimit}
          />
        </div>
      </div>
    }>
      <PortfolioOverview
        balance={portfolio.balance}
        equity={portfolio.equity}
        dailyPnL={portfolio.dailyPnL}
        dailyPnLPercent={portfolio.dailyPnLPercent}
        totalPnL={portfolio.totalPnL}
        openPositions={openPositions.length}
        mode="real"
      />

      <Tabs defaultValue="positions">
        <TabsList>
          <TabsTrigger value="positions">Open Positions ({openPositions.length})</TabsTrigger>
          <TabsTrigger value="orders">Order History</TabsTrigger>
          <TabsTrigger value="settings">Risk Settings</TabsTrigger>
        </TabsList>

        <TabsContent value="positions">
          <PositionsTable
            positions={openPositions}
            onClosePosition={closePosition}
            showRiskWarnings={true}
            requireConfirmation={true}
          />
        </TabsContent>

        <TabsContent value="orders">
          <OrderHistoryTable orders={orderHistory} />
        </TabsContent>

        <TabsContent value="settings">
          <RiskSettingsPanel />
        </TabsContent>
      </Tabs>

      {confirmDialog && (
        <TradeConfirmationDialog
          isOpen={!!confirmDialog}
          order={confirmDialog}
          onConfirm={handleConfirmOrder}
          onCancel={() => setConfirmDialog(null)}
          isSubmitting={isLoading}
        />
      )}
    </TradingDashboardLayout>
  )
}
```

## Todo List

- [ ] Create TradingReal.tsx page
- [ ] Create TradeConfirmationDialog.tsx
- [ ] Create DailyLossLimitBar.tsx
- [ ] Create ExchangeStatus.tsx
- [ ] Create ClosePositionDialog.tsx
- [ ] Create OrderPreview.tsx
- [ ] Create FeeEstimateCard.tsx
- [ ] Create RiskSettingsPanel.tsx
- [ ] Add route in App.tsx
- [ ] Connect to useRealTrading hook
- [ ] Implement order preview calculation
- [ ] Add position close confirmation
- [ ] Test 2-step confirmation flow
- [ ] Test daily loss limit UI
- [ ] Mobile responsive testing
- [ ] Accessibility audit

## Success Criteria

1. 2-step confirmation prevents accidental trades
2. Warning banner always visible in real mode
3. Daily loss limit clearly displayed
4. Fee estimates shown before execution
5. All confirmation flows work on mobile
6. No way to bypass safety confirmations

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Accidental real trade | Low | Critical | 2-step + checkbox confirmation |
| Missing fee display | Low | High | Require preview before confirm |
| Banner dismissal abuse | Medium | Medium | Re-show daily, log dismissals |
| Exchange connection loss | Medium | High | Clear status indicator |

## Security Considerations

- CRITICAL: All trade confirmations must be explicit
- Fee estimates must come from backend (not frontend calculation)
- Position close requires same 2-step flow
- Rate limit trade submissions
- Log all confirmation dismissals for audit

## Next Steps

After Phase 4 completion, proceed to [Phase 5: 3D Visualization](./phase-05-3d-visualization.md)
