# Phase 2: Mode Separation Infrastructure

**Status**: Pending | **Priority**: P0 | **Est. Time**: 2-3 days

---

## Context

- [Main Plan](./plan.md)
- [Phase 1: Design System](./phase-01-design-system.md)
- [Frontend Structure Scout](./scout/scout-01-frontend-structure.md)

## Overview

Build core infrastructure for trading mode separation: context providers, hooks, routing, and mode switcher component with safety confirmations.

## Key Insights

1. **Route-based Mode Selection** - `/trading/paper` and `/trading/real` for clean URL structure
2. **Context Provider Pattern** - TradingModeContext wraps existing providers
3. **Parallel Hook Structure** - useRealTrading mirrors usePaperTrading
4. **Safety-First Mode Switching** - Confirmation dialog required

## Requirements

### Functional
- [ ] TradingModeContext with mode state
- [ ] useRealTrading hook (parallel to usePaperTrading)
- [ ] Mode switcher component with confirmation
- [ ] Route configuration for /trading/paper and /trading/real
- [ ] Persist mode preference in localStorage

### Non-Functional
- [ ] Mode switch takes <100ms
- [ ] No data loss during mode switch
- [ ] Works with browser back/forward

## Architecture

### New Component Architecture

```
src/
├── contexts/
│   ├── TradingModeContext.tsx     # NEW - Mode state
│   ├── RealTradingContext.tsx     # NEW - Real trading provider
│   └── PaperTradingContext.tsx    # EXISTS - Keep as-is
├── hooks/
│   ├── useTradingMode.ts          # NEW - Mode selection
│   └── useRealTrading.ts          # NEW - Real trading state
├── components/
│   └── trading/
│       ├── ModeSwitcher.tsx       # NEW - Toggle with confirmation
│       ├── ModeConfirmDialog.tsx  # NEW - Safety confirmation
│       └── TradingLayout.tsx      # NEW - Shared layout wrapper
└── pages/
    ├── TradingPaper.tsx           # Refactored
    └── TradingReal.tsx            # NEW
```

### Provider Hierarchy

```
QueryClientProvider
└── AuthProvider
    └── WebSocketProvider
        └── TradingModeProvider       # NEW (wraps both modes)
            ├── PaperTradingProvider  # When mode='paper'
            └── RealTradingProvider   # When mode='real'
                └── AIAnalysisProvider
                    └── App Routes
```

### TradingModeContext Interface

```typescript
interface TradingModeContextValue {
  mode: 'paper' | 'real'
  setMode: (mode: 'paper' | 'real') => void
  isPaperMode: boolean
  isRealMode: boolean
  confirmModeSwitch: (targetMode: 'paper' | 'real') => Promise<boolean>
  isConfirmDialogOpen: boolean
}
```

### useRealTrading Interface

```typescript
interface RealTradingState {
  // Portfolio
  portfolio: RealPortfolioMetrics
  balance: number
  equity: number

  // Trades
  openPositions: RealPosition[]
  orderHistory: RealOrder[]

  // Exchange
  exchangeConnected: boolean
  exchangeStatus: 'connected' | 'disconnected' | 'error'

  // Risk
  dailyPnL: number
  dailyLossLimit: number
  isLimitReached: boolean

  // Actions
  placeOrder: (order: OrderRequest) => Promise<OrderResult>
  cancelOrder: (orderId: string) => Promise<void>
  closePosition: (positionId: string) => Promise<void>

  // State
  isLoading: boolean
  error: string | null
  lastUpdated: Date | null
}
```

## Related Files

| File | Path | Action |
|------|------|--------|
| TradingModeContext | `/nextjs-ui-dashboard/src/contexts/TradingModeContext.tsx` | Create |
| RealTradingContext | `/nextjs-ui-dashboard/src/contexts/RealTradingContext.tsx` | Create |
| useTradingMode | `/nextjs-ui-dashboard/src/hooks/useTradingMode.ts` | Create |
| useRealTrading | `/nextjs-ui-dashboard/src/hooks/useRealTrading.ts` | Create |
| ModeSwitcher | `/nextjs-ui-dashboard/src/components/trading/ModeSwitcher.tsx` | Create |
| ModeConfirmDialog | `/nextjs-ui-dashboard/src/components/trading/ModeConfirmDialog.tsx` | Create |
| App.tsx | `/nextjs-ui-dashboard/src/App.tsx` | Update routes |

## Implementation Steps

### Step 1: Create TradingModeContext

```typescript
// src/contexts/TradingModeContext.tsx
import { createContext, useContext, useState, useCallback } from 'react'

const TradingModeContext = createContext<TradingModeContextValue | null>(null)

export function TradingModeProvider({ children }: { children: React.ReactNode }) {
  const [mode, setModeState] = useState<'paper' | 'real'>(() => {
    return (localStorage.getItem('tradingMode') as 'paper' | 'real') || 'paper'
  })
  const [isConfirmDialogOpen, setIsConfirmDialogOpen] = useState(false)
  const [pendingMode, setPendingMode] = useState<'paper' | 'real' | null>(null)

  const setMode = useCallback((newMode: 'paper' | 'real') => {
    setModeState(newMode)
    localStorage.setItem('tradingMode', newMode)
  }, [])

  const confirmModeSwitch = useCallback(async (targetMode: 'paper' | 'real') => {
    return new Promise<boolean>((resolve) => {
      setPendingMode(targetMode)
      setIsConfirmDialogOpen(true)
      // Resolution handled by dialog
    })
  }, [])

  return (
    <TradingModeContext.Provider value={{
      mode,
      setMode,
      isPaperMode: mode === 'paper',
      isRealMode: mode === 'real',
      confirmModeSwitch,
      isConfirmDialogOpen
    }}>
      {children}
    </TradingModeContext.Provider>
  )
}

export const useTradingMode = () => {
  const context = useContext(TradingModeContext)
  if (!context) throw new Error('useTradingMode must be used within TradingModeProvider')
  return context
}
```

### Step 2: Create ModeConfirmDialog

```typescript
// src/components/trading/ModeConfirmDialog.tsx
import { Dialog, DialogContent, DialogHeader, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { AlertTriangle, CheckCircle } from 'lucide-react'

interface Props {
  isOpen: boolean
  targetMode: 'paper' | 'real'
  onConfirm: () => void
  onCancel: () => void
}

export function ModeConfirmDialog({ isOpen, targetMode, onConfirm, onCancel }: Props) {
  const isToReal = targetMode === 'real'

  return (
    <Dialog open={isOpen} onOpenChange={(open) => !open && onCancel()}>
      <DialogContent className="max-w-md">
        <DialogHeader>
          {isToReal ? (
            <AlertTriangle className="w-12 h-12 text-real-warning mx-auto mb-4" />
          ) : (
            <CheckCircle className="w-12 h-12 text-paper-accent mx-auto mb-4" />
          )}
          <DialogTitle className="text-center">
            {isToReal ? 'Switch to Real Trading?' : 'Switch to Paper Trading?'}
          </DialogTitle>
        </DialogHeader>

        <div className="space-y-4 py-4">
          {isToReal ? (
            <>
              <p className="text-real-warning font-semibold text-center">
                WARNING: You will be trading with REAL MONEY
              </p>
              <ul className="text-sm text-slate-400 space-y-2">
                <li>* Trades will execute on your connected exchange</li>
                <li>* Losses are real and cannot be undone</li>
                <li>* Daily loss limits will be enforced</li>
              </ul>
            </>
          ) : (
            <p className="text-slate-400 text-center">
              Switch to sandbox mode. No real money will be used.
            </p>
          )}
        </div>

        <div className="flex gap-4 justify-end">
          <Button variant="outline" onClick={onCancel}>Cancel</Button>
          <Button
            variant={isToReal ? 'destructive' : 'default'}
            onClick={onConfirm}
          >
            {isToReal ? 'I Understand, Switch to Real' : 'Switch to Paper'}
          </Button>
        </div>
      </DialogContent>
    </Dialog>
  )
}
```

### Step 3: Create ModeSwitcher Component

```typescript
// src/components/trading/ModeSwitcher.tsx
import { useTradingMode } from '@/contexts/TradingModeContext'
import { Button } from '@/components/ui/button'
import { cn } from '@/lib/utils'

export function ModeSwitcher() {
  const { mode, confirmModeSwitch } = useTradingMode()

  const handleSwitch = async () => {
    const targetMode = mode === 'paper' ? 'real' : 'paper'
    await confirmModeSwitch(targetMode)
  }

  return (
    <div className="flex items-center gap-2 p-1 bg-slate-800 rounded-lg">
      <Button
        size="sm"
        variant={mode === 'paper' ? 'default' : 'ghost'}
        className={cn(
          mode === 'paper' && 'bg-paper-accent hover:bg-paper-accent/90'
        )}
        onClick={() => mode !== 'paper' && handleSwitch()}
      >
        Paper
      </Button>
      <Button
        size="sm"
        variant={mode === 'real' ? 'default' : 'ghost'}
        className={cn(
          mode === 'real' && 'bg-real-warning hover:bg-real-warning/90'
        )}
        onClick={() => mode !== 'real' && handleSwitch()}
      >
        Real
      </Button>
    </div>
  )
}
```

### Step 4: Create useRealTrading Hook

```typescript
// src/hooks/useRealTrading.ts
import { useState, useEffect, useCallback } from 'react'

export function useRealTrading() {
  const [state, setState] = useState<RealTradingState>({
    portfolio: null,
    balance: 0,
    equity: 0,
    openPositions: [],
    orderHistory: [],
    exchangeConnected: false,
    exchangeStatus: 'disconnected',
    dailyPnL: 0,
    dailyLossLimit: 0,
    isLimitReached: false,
    isLoading: true,
    error: null,
    lastUpdated: null
  })

  // Fetch real trading data from backend
  const fetchPortfolio = useCallback(async () => {
    try {
      const response = await fetch(`${API_URL}/api/real-trading/portfolio`)
      const data = await response.json()
      setState(prev => ({ ...prev, portfolio: data, isLoading: false }))
    } catch (error) {
      setState(prev => ({ ...prev, error: error.message, isLoading: false }))
    }
  }, [])

  // Place order with 2-step confirmation
  const placeOrder = useCallback(async (order: OrderRequest) => {
    // Implementation with safety checks
  }, [])

  return { ...state, fetchPortfolio, placeOrder }
}
```

### Step 5: Update App.tsx Routes

```typescript
// App.tsx - Add trading routes
<Routes>
  {/* Existing routes */}
  <Route path="/" element={<Index />} />
  <Route path="/dashboard" element={<Dashboard />} />

  {/* New trading routes */}
  <Route path="/trading" element={<Navigate to="/trading/paper" replace />} />
  <Route path="/trading/paper" element={<TradingPaper />} />
  <Route path="/trading/real" element={<TradingReal />} />

  {/* Keep legacy route for backwards compatibility */}
  <Route path="/trading-paper" element={<Navigate to="/trading/paper" replace />} />
</Routes>
```

### Step 6: Update Provider Hierarchy in App.tsx

```typescript
// App.tsx
<TradingModeProvider>
  <ConditionalTradingProvider>
    {/* Rest of app */}
  </ConditionalTradingProvider>
</TradingModeProvider>

// ConditionalTradingProvider.tsx
function ConditionalTradingProvider({ children }) {
  const { mode } = useTradingMode()

  if (mode === 'paper') {
    return <PaperTradingProvider>{children}</PaperTradingProvider>
  }
  return <RealTradingProvider>{children}</RealTradingProvider>
}
```

## Todo List

- [ ] Create TradingModeContext.tsx
- [ ] Create RealTradingContext.tsx
- [ ] Create useTradingMode.ts hook
- [ ] Create useRealTrading.ts hook
- [ ] Create ModeSwitcher.tsx component
- [ ] Create ModeConfirmDialog.tsx component
- [ ] Update App.tsx with new routes
- [ ] Update provider hierarchy
- [ ] Add localStorage persistence for mode
- [ ] Create TradingLayout.tsx wrapper
- [ ] Add mode to DashboardHeader
- [ ] Write unit tests for contexts
- [ ] Write unit tests for mode switching

## Success Criteria

1. Mode persists across page refreshes
2. Confirmation dialog appears on mode switch
3. Routes work with browser back/forward
4. No data loss when switching modes
5. All tests pass

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Data loss on mode switch | Medium | High | Save state before switch |
| Accidental mode switch | Low | Critical | Require explicit confirmation |
| Context not available | Low | High | Add error boundaries |

## Security Considerations

- Mode switch to real requires confirmation
- Real trading requires additional auth (future: 2FA)
- No sensitive data in localStorage (mode preference only)
- Rate limit mode switches (prevent spam)

## Next Steps

After Phase 2 completion, proceed to [Phase 3: Component Extraction](./phase-03-component-refactor.md)
