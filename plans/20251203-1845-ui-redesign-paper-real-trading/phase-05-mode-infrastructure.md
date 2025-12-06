# Phase 5: Trading Mode Infrastructure

## Context
- **Parent Plan**: [plan.md](./plan.md)
- **Dependencies**: Phase 1-4
- **Research**: [Trading UI Patterns](./research/researcher-01-trading-ui-patterns.md)

## Overview
| Field | Value |
|-------|-------|
| Priority | P0 - Critical |
| Status | Pending |
| Est. Time | 2-3 days |
| Description | Create TradingModeContext, useRealTrading hook, mode switching with safety confirmations |

## Key Insights
- Mode stored in context + localStorage
- Switch requires explicit confirmation dialog
- Real mode shows persistent warning banner
- Different API endpoints per mode
- Route-based: /trading/paper vs /trading/real

## Requirements

### Functional
- TradingModeContext (paper | real)
- Mode switch with confirmation dialog
- Persistent mode across sessions
- useRealTrading hook (parallel to usePaperTrading)
- Warning banner in real mode

### Non-Functional
- Mode switch < 100ms
- No accidental mode changes
- Clear visual feedback

## Architecture

```
contexts/
├── TradingModeContext.tsx
│   ├── mode: 'paper' | 'real'
│   ├── setMode(mode, confirmed)
│   └── showModeSwitch()
hooks/
├── useTradingMode.ts
├── useRealTrading.ts (NEW)
└── usePaperTrading.ts (existing)
components/
├── ModeSwitchDialog.tsx
└── RealModeWarningBanner.tsx
```

## Related Code Files

### Create
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/TradingModeContext.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/useTradingMode.ts`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/useRealTrading.ts`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/ModeSwitchDialog.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/RealModeWarningBanner.tsx`
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/ModeToggle.tsx`

### Modify
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/App.tsx` - Add TradingModeProvider
- `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/usePaperTrading.ts` - Ensure compatibility

## Implementation Steps

1. **Create TradingModeContext**
   ```tsx
   type TradingMode = 'paper' | 'real';

   interface TradingModeContextType {
     mode: TradingMode;
     setMode: (mode: TradingMode) => void;
     requestModeSwitch: (targetMode: TradingMode) => void;
     isModeSwitchOpen: boolean;
   }
   ```

2. **Create ModeSwitchDialog**
   - Title: "Switch to Real Trading?"
   - Warning text about real money
   - Checkbox: "I understand this involves real money"
   - Cancel / Confirm buttons
   - Confirm disabled until checkbox checked

3. **Create RealModeWarningBanner**
   - Sticky banner at top
   - Red background (#EF4444)
   - Text: "⚠️ REAL MONEY MODE - All trades execute with real funds"
   - Cannot be dismissed

4. **Create useRealTrading hook**
   - Mirror usePaperTrading API
   - Connect to real trading endpoints
   - Include safety checks

5. **Create ModeToggle component**
   - Toggle switch in header
   - Shows current mode
   - Triggers confirmation dialog

6. **Update App.tsx**
   - Wrap with TradingModeProvider
   - Route protection for real mode

## Todo List

- [ ] Create TradingModeContext
- [ ] Create useTradingMode hook
- [ ] Create ModeSwitchDialog with confirmation
- [ ] Create RealModeWarningBanner
- [ ] Create useRealTrading hook
- [ ] Create ModeToggle component
- [ ] Update App.tsx with provider
- [ ] Add localStorage persistence
- [ ] Test mode switching flow
- [ ] Test warning banner display
- [ ] Write context tests

## Success Criteria

- [ ] Mode persists across page refresh
- [ ] Cannot switch to real mode without confirmation
- [ ] Warning banner always visible in real mode
- [ ] useRealTrading works parallel to usePaperTrading
- [ ] Mode toggle clearly shows current state

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Accidental real trade | CRITICAL | 2-step confirmation, checkbox |
| Mode state desync | High | Single source of truth in context |
| API endpoint confusion | High | Mode-aware API client |

## Security Considerations
- Real mode requires re-authentication if session > 1hr
- Log all mode switches for audit
- Rate limit mode switching

## Next Steps
→ Phase 6: Trading Pages
