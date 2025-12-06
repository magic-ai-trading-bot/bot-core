# Phase 5 Integration Guide

## Overview

Phase 5 provides trading mode infrastructure. This guide shows how other phases integrate it.

---

## For Phase 2 (Layout)

### App.tsx Integration

```tsx
import { TradingModeProvider } from '@/contexts/TradingModeContext';
import { ModeSwitchDialog } from '@/components/trading/ModeSwitchDialog';
import { RealModeWarningBanner } from '@/components/trading/RealModeWarningBanner';

function App() {
  return (
    <TradingModeProvider>
      {/* Warning banner at top (sticky) */}
      <RealModeWarningBanner />

      {/* Confirmation dialog (modal overlay) */}
      <ModeSwitchDialog />

      {/* Rest of app */}
      <Router>
        <Layout>
          <Routes>
            {/* Your routes */}
          </Routes>
        </Layout>
      </Router>
    </TradingModeProvider>
  );
}
```

**Key Points**:
- TradingModeProvider wraps entire app
- RealModeWarningBanner must be at root level (sticky positioning)
- ModeSwitchDialog must be at root level (modal overlay)
- Order matters: Banner → Dialog → Content

---

## For Phase 3 (Header)

### Header Component Integration

```tsx
import { ModeToggle } from '@/components/trading/ModeToggle';

export function Header() {
  return (
    <header>
      <div className="flex items-center justify-between">
        {/* Logo */}
        <Logo />

        {/* Mode Toggle (right side) */}
        <div className="flex items-center gap-4">
          <ModeToggle />
          <UserMenu />
        </div>
      </div>
    </header>
  );
}
```

**Key Points**:
- Place ModeToggle in visible location (usually top-right)
- No props needed (uses context internally)
- Toggle automatically triggers confirmation for real mode

---

## For Phase 6 (Trading Pages)

### Using Mode-Aware Hooks

```tsx
import { useTradingMode } from '@/hooks/useTradingMode';
import { usePaperTrading } from '@/hooks/usePaperTrading';
import { useRealTrading } from '@/hooks/useRealTrading';

export function TradingDashboard() {
  const { mode } = useTradingMode();

  // Get both hooks
  const paperTrading = usePaperTrading();
  const realTrading = useRealTrading();

  // Use current mode's hook
  const trading = mode === 'paper' ? paperTrading : realTrading;

  return (
    <div>
      <h1>Trading Dashboard ({mode} mode)</h1>

      {/* All these work regardless of mode */}
      <Portfolio portfolio={trading.portfolio} />
      <TradeList trades={trading.openTrades} />
      <button onClick={trading.startTrading}>Start Trading</button>
    </div>
  );
}
```

**Key Points**:
- Both hooks have identical API
- Switch between them based on mode
- No code changes needed when switching modes
- useRealTrading only fetches when mode === 'real'

### Alternative: Single Hook Pattern

```tsx
// For cleaner code, create a unified hook
export function useTrading() {
  const { mode } = useTradingMode();
  const paperTrading = usePaperTrading();
  const realTrading = useRealTrading();

  return mode === 'paper' ? paperTrading : realTrading;
}

// Usage
export function TradingDashboard() {
  const trading = useTrading(); // Auto-switches based on mode

  return (
    <div>
      <Portfolio portfolio={trading.portfolio} />
    </div>
  );
}
```

---

## Testing Mode Switching

### Manual Testing Checklist

1. **Initial Load**
   - [ ] App defaults to paper mode
   - [ ] No warning banner visible
   - [ ] Toggle shows blue (paper)

2. **Switch to Real Mode**
   - [ ] Click toggle → Dialog opens
   - [ ] Confirm button disabled initially
   - [ ] Check checkbox → Confirm enabled
   - [ ] Click confirm → Mode switches
   - [ ] Warning banner appears at top
   - [ ] Toggle shows red (real) with LIVE badge

3. **Switch to Paper Mode**
   - [ ] Click toggle → Mode switches immediately (no dialog)
   - [ ] Warning banner disappears
   - [ ] Toggle shows blue (paper)

4. **Persistence**
   - [ ] Refresh page → Mode persists
   - [ ] Open new tab → Same mode
   - [ ] Close and reopen → Mode preserved

5. **Error Handling**
   - [ ] Try to close real trade in paper mode → Error toast
   - [ ] Try to start real trading when mode is paper → Error toast

---

## localStorage Format

```javascript
// Key
'trading-mode'

// Values
'paper' | 'real'

// Example
localStorage.getItem('trading-mode') // 'paper'
```

---

## Context API Reference

```typescript
interface TradingModeContextType {
  // Current mode
  mode: 'paper' | 'real';

  // Set mode directly (internal use)
  setMode: (mode: TradingMode) => void;

  // Request mode switch (shows confirmation if needed)
  requestModeSwitch: (targetMode: TradingMode) => void;

  // Dialog state
  isModeSwitchOpen: boolean;
  closeModeSwitchDialog: () => void;
  confirmModeSwitch: () => void;
  pendingMode: TradingMode | null;
}
```

**Usage**:
```tsx
const { mode, requestModeSwitch } = useTradingMode();
```

---

## Component Props

### ModeToggle
**Props**: None (uses context)
**Usage**: `<ModeToggle />`

### ModeSwitchDialog
**Props**: None (uses context)
**Usage**: `<ModeSwitchDialog />`
**Note**: Must be at root level for modal overlay

### RealModeWarningBanner
**Props**: None (uses context)
**Usage**: `<RealModeWarningBanner />`
**Note**: Must be at root level for sticky positioning

---

## Safety Features Summary

1. **2-Step Confirmation**
   - Paper → Real: Dialog + Checkbox
   - Real → Paper: Immediate (safe direction)

2. **Persistent Warning**
   - Red banner always visible in real mode
   - Cannot be dismissed
   - Reminds user of real money risk

3. **Mode Verification**
   - useRealTrading checks mode before API calls
   - Clear error messages if wrong mode
   - Prevents accidental real trades

4. **Conservative Defaults**
   - Lower leverage (5x vs 10x)
   - Tighter risk limits (1% vs 2%)
   - Stricter daily loss (3% vs 5%)
   - Longer cool-down (120min vs 60min)

---

## Common Patterns

### Conditional UI Based on Mode

```tsx
const { mode } = useTradingMode();

return (
  <div>
    {mode === 'real' && (
      <div className="text-red-500">
        ⚠️ Real money mode active
      </div>
    )}

    <button
      style={{
        backgroundColor: mode === 'paper'
          ? colors.paper.accent
          : colors.real.warning
      }}
    >
      Execute Trade
    </button>
  </div>
);
```

### Conditional Features

```tsx
const { mode } = useTradingMode();
const trading = useTrading();

// Reset only available in paper mode
{mode === 'paper' && (
  <button onClick={trading.resetPortfolio}>
    Reset Portfolio
  </button>
)}
```

### Mode-Specific Styling

```tsx
import { getModeColor } from '@/styles/tokens/colors';

const { mode } = useTradingMode();
const accentColor = getModeColor(mode, 'accent');

<div style={{ borderColor: accentColor }}>
  {content}
</div>
```

---

## Performance Considerations

- **Mode Switch**: <100ms (localStorage + state update)
- **Hook Updates**: Only active mode hook fetches data
- **WebSocket**: Separate connections per mode
- **Re-renders**: Minimal (context uses useCallback)

---

## Troubleshooting

### Issue: Mode not persisting
**Solution**: Check localStorage permissions, try incognito mode

### Issue: Confirmation dialog not showing
**Solution**: Verify TradingModeProvider wraps app

### Issue: Warning banner not sticky
**Solution**: Check z-index conflicts, ensure at root level

### Issue: Wrong hook fetching data
**Solution**: Verify mode check in hook's useEffect

---

## Future Enhancements (Optional)

1. **Re-authentication**
   ```typescript
   // TODO: Add re-auth check if session > 1hr in real mode
   if (mode === 'real' && sessionAge > 60 * 60 * 1000) {
     requestReAuthentication();
   }
   ```

2. **Audit Log**
   ```typescript
   // Log all mode switches for security
   logModeSwitch({
     from: mode,
     to: targetMode,
     timestamp: new Date(),
     userId: user.id,
   });
   ```

3. **Biometric Confirmation**
   ```typescript
   // Optional: Add biometric auth for real mode
   await authenticateWithBiometric();
   ```

---

**Status**: Ready for Integration
**Dependencies**: None (Phase 5 is standalone)
**Next**: Phase 2 (Layout) + Phase 3 (Header) + Phase 6 (Trading Pages)
