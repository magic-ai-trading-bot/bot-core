# Phase 4: Frontend UI

## Context Links
- [Main Plan](./plan.md)
- [Phase 3: Database & API](./phase-03-database-api.md)
- [Existing Paper Trading UI](../../nextjs-ui-dashboard/src/pages/TradingPaper.tsx)

## Overview

| Field | Value |
|-------|-------|
| Priority | HIGH |
| Status | PENDING |
| Effort | 3 days |
| Dependencies | Phase 3 complete |

Create frontend components for real trading with clear visual distinction from paper trading. Mode selector, confirmation dialogs, emergency stop button.

## Key Insights

1. **Existing UI stack**: React, Shadcn/UI, TailwindCSS, Lucide icons
2. **Paper trading page**: TradingPaper.tsx with positions table, portfolio summary, settings
3. **Context pattern**: PaperTradingContext for state management
4. **Color scheme**: Profit (green), Loss (red), Warning (amber)
5. **Confirmation**: Use Dialog component for critical actions

## Requirements

### Functional
- FR-REAL-030: Trading mode selector in header (Paper/Testnet/Mainnet)
- FR-REAL-031: Visual distinction between modes (colors, badges, borders)
- FR-REAL-032: Real trading page mirroring paper trading layout
- FR-REAL-033: Confirmation dialog for mode switch to mainnet
- FR-REAL-034: Confirmation dialog for real order placement
- FR-REAL-035: Emergency stop button with confirmation
- FR-REAL-036: Separate trade history by mode
- FR-REAL-037: Mode indicator always visible in header

### Non-Functional
- NFR-REAL-030: Clear affordance that real money is at risk
- NFR-REAL-031: Consistent UI patterns with paper trading
- NFR-REAL-032: Mobile responsive

## Architecture

### Component Structure

```
components/
├── trading/
│   ├── TradingModeSelector.tsx     # Mode dropdown with colors
│   ├── TradingModeIndicator.tsx    # Header badge showing current mode
│   ├── RealTradingContext.tsx      # Real trading state
│   ├── RealPositionsTable.tsx      # Open positions (real)
│   ├── RealTradeHistory.tsx        # Closed trades (real)
│   ├── RealPortfolioSummary.tsx    # Balance, P&L (real)
│   ├── EmergencyStopButton.tsx     # Red panic button
│   └── OrderConfirmationDialog.tsx # Confirm before order
│
pages/
├── TradingPaper.tsx                # Existing paper trading
└── TradingReal.tsx                 # New real trading page
```

### Color Scheme by Mode

```
Paper Trading:
  - Primary: Blue (bg-blue-500)
  - Border: Blue (border-blue-300)
  - Badge: "PAPER" bg-blue-100 text-blue-800

Testnet:
  - Primary: Amber (bg-amber-500)
  - Border: Amber (border-amber-300)
  - Badge: "TESTNET" bg-amber-100 text-amber-800
  - Header stripe: Amber

Mainnet:
  - Primary: Emerald (bg-emerald-500)
  - Border: Emerald (border-emerald-300)
  - Badge: "LIVE" bg-emerald-100 text-emerald-800
  - Header stripe: Emerald + pulsing warning icon
```

### Component Specifications

#### TradingModeSelector

```tsx
interface TradingModeSelectorProps {
  currentMode: TradingMode;
  onModeChange: (mode: TradingMode) => Promise<void>;
  disabled?: boolean;
}

// Renders:
// - Dropdown with three options (Paper, Testnet, Mainnet)
// - Each option shows color indicator and name
// - Mainnet option shows warning icon
// - Opens confirmation dialog for Mainnet selection
```

#### TradingModeIndicator

```tsx
interface TradingModeIndicatorProps {
  mode: TradingMode;
}

// Renders:
// - Badge in header showing current mode
// - Color-coded background
// - For Mainnet: pulsing ring animation
// - Tooltip explains mode
```

#### EmergencyStopButton

```tsx
interface EmergencyStopButtonProps {
  onStop: () => Promise<void>;
  disabled?: boolean;
}

// Renders:
// - Red button with stop icon
// - Requires two clicks to activate (click -> confirm dialog)
// - Shows confirmation dialog with checkbox
// - Cancels all orders, disables trading
```

#### OrderConfirmationDialog

```tsx
interface OrderConfirmationDialogProps {
  order: PendingOrder;
  onConfirm: () => void;
  onCancel: () => void;
  open: boolean;
}

// Renders:
// - Dialog showing order details
// - Symbol, side, quantity, price, estimated cost
// - Commission estimate
// - "I understand this uses real funds" checkbox
// - Confirm/Cancel buttons
```

## Related Code Files

| File | Action | Description |
|------|--------|-------------|
| `nextjs-ui-dashboard/src/components/trading/TradingModeSelector.tsx` | CREATE | Mode dropdown |
| `nextjs-ui-dashboard/src/components/trading/TradingModeIndicator.tsx` | CREATE | Header badge |
| `nextjs-ui-dashboard/src/components/trading/EmergencyStopButton.tsx` | CREATE | Panic button |
| `nextjs-ui-dashboard/src/components/trading/OrderConfirmationDialog.tsx` | CREATE | Order confirm |
| `nextjs-ui-dashboard/src/contexts/RealTradingContext.tsx` | CREATE | Real state |
| `nextjs-ui-dashboard/src/pages/TradingReal.tsx` | CREATE | Real trading page |
| `nextjs-ui-dashboard/src/hooks/useRealTrading.ts` | CREATE | Real API hook |
| `nextjs-ui-dashboard/src/components/dashboard/DashboardHeader.tsx` | MODIFY | Add mode indicator |
| `nextjs-ui-dashboard/src/lib/api.ts` | MODIFY | Add real trading API |

## Implementation Steps

### Step 1: Create API Functions (Day 1)

Add to `lib/api.ts`:

```typescript
// Trading mode
export async function getTradingMode(): Promise<TradingMode>;
export async function setTradingMode(mode: TradingMode, confirmed?: boolean): Promise<void>;

// Real trading
export async function getRealPortfolio(): Promise<RealPortfolio>;
export async function listRealTrades(filter?: TradeFilter): Promise<RealTradeList>;
export async function emergencyStop(): Promise<EmergencyStopResult>;
export async function getRealSettings(): Promise<RealTradingSettings>;
export async function updateRealSettings(settings: RealTradingSettings): Promise<void>;
```

### Step 2: Create RealTradingContext (Day 1)

```typescript
interface RealTradingState {
  mode: TradingMode;
  portfolio: RealPortfolio | null;
  openTrades: RealTrade[];
  closedTrades: RealTrade[];
  settings: RealTradingSettings | null;
  isLoading: boolean;
  error: string | null;
}

const RealTradingContext = createContext<{
  state: RealTradingState;
  actions: {
    setMode: (mode: TradingMode) => Promise<void>;
    refreshPortfolio: () => Promise<void>;
    emergencyStop: () => Promise<void>;
    closeTrade: (tradeId: string) => Promise<void>;
  };
}>();
```

### Step 3: Create TradingModeSelector (Day 1)

1. Dropdown with three mode options
2. Color indicators for each mode
3. Warning icon for mainnet
4. Opens confirmation dialog for mainnet switch
5. Disables options with open positions (optional)

### Step 4: Create TradingModeIndicator (Day 1)

1. Badge showing current mode
2. Color-coded (blue/amber/emerald)
3. Mainnet shows pulsing animation
4. Click navigates to relevant trading page

### Step 5: Create EmergencyStopButton (Day 2)

1. Large red button with stop icon
2. Click opens confirmation dialog
3. Dialog shows:
   - Warning message
   - List of pending orders to cancel
   - Checkbox: "I confirm emergency stop"
4. Calls emergency-stop API
5. Shows success/failure toast

### Step 6: Create OrderConfirmationDialog (Day 2)

1. Shows order details:
   - Symbol, Side (BUY/SELL), Type (MARKET/LIMIT)
   - Quantity, Price (for limit), Total value
   - Estimated commission
   - Current balance after order
2. Checkbox: "I understand this uses real funds"
3. Confirm/Cancel buttons
4. Only shown for testnet/mainnet mode

### Step 7: Create TradingReal Page (Day 2-3)

1. Copy TradingPaper.tsx as base
2. Replace PaperTradingContext with RealTradingContext
3. Add mode-specific styling (border color based on mode)
4. Add emergency stop button in header
5. Add order confirmation for manual orders
6. Show mode indicator prominently

### Step 8: Update DashboardHeader (Day 3)

1. Add TradingModeIndicator component
2. Add TradingModeSelector dropdown
3. Style header border based on mode:
   - Paper: default
   - Testnet: amber top border
   - Mainnet: emerald top border + warning

### Step 9: Add Navigation (Day 3)

1. Add "Real Trading" link in sidebar
2. Conditionally show based on enabled modes
3. Add mode indicator next to link

### Step 10: Tests (Day 3)

1. Component unit tests
2. Mode switch integration test
3. Confirmation dialog tests

## Todo List

- [ ] Add API functions to lib/api.ts
- [ ] Create RealTradingContext with state and actions
- [ ] Create useRealTrading hook
- [ ] Create TradingModeSelector component
- [ ] Create TradingModeIndicator component
- [ ] Create EmergencyStopButton component
- [ ] Create OrderConfirmationDialog component
- [ ] Create TradingReal page (copy from TradingPaper)
- [ ] Update DashboardHeader with mode indicator
- [ ] Add mode-based styling (border colors)
- [ ] Add navigation link for Real Trading
- [ ] Write component tests

## Success Criteria

1. Mode selector shows all three modes with correct colors
2. Mainnet selection requires confirmation dialog
3. Mode indicator visible in header at all times
4. Real trading page shows live positions
5. Emergency stop works and shows confirmation
6. Order placement shows confirmation dialog
7. UI clearly distinguishes paper from real

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Accidental mainnet switch | CRITICAL | Low | Confirmation + env var |
| Confusion between modes | High | Medium | Strong visual distinction |
| Emergency stop not visible | High | Low | Always-visible red button |
| Order placed without review | High | Medium | Mandatory confirmation dialog |

## Security Considerations

1. **Double confirmation for mainnet** - Dialog + checkbox
2. **Clear visual distinction** - Colors, badges, animations
3. **Emergency stop prominent** - Always visible, easy access
4. **No auto-submit** - All real orders require confirmation

## UI/UX Mockups

### Mode Selector States

```
[Paper] --------v    (Blue dropdown button)
├── Paper       ✓    (Blue indicator, selected)
├── Testnet          (Amber indicator)
└── Mainnet ⚠️       (Emerald indicator + warning)
```

### Header with Mode Indicator

```
Paper Mode:
┌─────────────────────────────────────────────────┐
│  Logo   Dashboard   Trading   Settings   [PAPER]│
└─────────────────────────────────────────────────┘

Mainnet Mode:
┌═══════════════════════════════════════════════════┐ ← Emerald border
│  Logo   Dashboard   Trading   Settings   [⚠️ LIVE]│
└═══════════════════════════════════════════════════┘
```

### Emergency Stop Button

```
┌──────────────────────────────────┐
│  🛑 EMERGENCY STOP               │  ← Large red button
└──────────────────────────────────┘

Dialog:
┌──────────────────────────────────────────┐
│  ⚠️ Emergency Stop Trading               │
│                                          │
│  This will:                              │
│  - Cancel 3 pending orders               │
│  - Close 2 open positions at market      │
│  - Disable trading until re-enabled      │
│                                          │
│  [ ] I confirm emergency stop            │
│                                          │
│          [Cancel]  [STOP NOW]            │
└──────────────────────────────────────────┘
```

## Next Steps

After Phase 4 complete:
- Proceed to [Phase 5: Safety & Testing](./phase-05-safety-testing.md)
- Add comprehensive safety mechanisms
- Test on testnet before mainnet rollout
