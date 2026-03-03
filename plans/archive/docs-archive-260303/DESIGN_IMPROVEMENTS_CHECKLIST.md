# Design Improvements Checklist
## Bot Core Trading Dashboard - Developer Action Items

**Sprint Planning Guide** | **Last Updated:** 2025-11-19

---

## Sprint 1: Accessibility Fixes (CRITICAL) 🚨

**Goal:** Achieve WCAG 2.1 AA compliance
**Total Effort:** 14 hours
**Priority:** P0 (Must fix before production)

### Task 1.1: Add Alt Text to Images (2 hours)
**Files to modify:**
- `src/pages/Login.tsx` - Logo alt text
- `src/components/dashboard/DashboardHeader.tsx` - Logo alt text
- `src/components/landing/HeroSection.tsx` - Hero images

**Code example:**
```tsx
// Before
<div className="w-12 h-12 bg-gradient-to-br from-primary to-accent rounded-2xl">
  <span className="text-primary-foreground font-bold">BT</span>
</div>

// After
<div
  className="w-12 h-12 bg-gradient-to-br from-primary to-accent rounded-2xl"
  role="img"
  aria-label="Bot Core Trading Bot Logo"
>
  <span className="text-primary-foreground font-bold" aria-hidden="true">BT</span>
</div>
```

**Checklist:**
- [ ] Login page logo
- [ ] Dashboard header logo
- [ ] Landing page hero images
- [ ] Strategy chart SVGs (aria-label)
- [ ] All decorative images (aria-hidden="true")

---

### Task 1.2: Add Profit/Loss Icons (3 hours)
**Files to modify:**
- `src/components/dashboard/BotStatus.tsx`
- `src/components/dashboard/AISignals.tsx`
- `src/components/dashboard/TradingCharts.tsx`

**Code example:**
```tsx
import { TrendingUp, TrendingDown } from "lucide-react";

// Before
<div className="text-profit">+2.5%</div>

// After
<div className="text-profit flex items-center gap-1">
  <TrendingUp className="h-4 w-4" aria-hidden="true" />
  <span>+2.5%</span>
  <span className="sr-only">Profit increase</span>
</div>
```

**Checklist:**
- [ ] BotStatus PnL display (add ↑/↓ icons)
- [ ] AISignals confidence indicators
- [ ] TradingCharts price change
- [ ] PerformanceChart trend indicators
- [ ] All percentage change displays

**Icon mapping:**
- Profit/Positive: `<TrendingUp />` (green)
- Loss/Negative: `<TrendingDown />` (red)
- Neutral: `<Activity />` (yellow)

---

### Task 1.3: Custom Focus Indicators (2 hours)
**Files to modify:**
- `src/index.css` - Add focus ring utilities

**Code to add:**
```css
@layer utilities {
  /* Custom focus styles matching brand */
  .focus-custom {
    @apply focus:outline-none focus:ring-2 focus:ring-profit focus:ring-offset-2 focus:ring-offset-background;
  }

  .focus-danger {
    @apply focus:outline-none focus:ring-2 focus:ring-loss focus:ring-offset-2 focus:ring-offset-background;
  }

  /* High contrast for accessibility */
  .focus-high-contrast {
    @apply focus:outline-4 focus:outline-offset-2 focus:outline-profit;
  }
}
```

**Update components:**
```tsx
// Before
<Button onClick={...}>Trade</Button>

// After
<Button onClick={...} className="focus-custom">Trade</Button>
```

**Checklist:**
- [ ] Add focus utilities to index.css
- [ ] Apply to all Button components
- [ ] Apply to all Input components
- [ ] Apply to all interactive elements
- [ ] Test keyboard navigation flow
- [ ] Verify focus visible on dark background

---

### Task 1.4: Accessible Chart Data Tables (4 hours)
**Files to modify:**
- `src/components/dashboard/TradingCharts.tsx`
- `src/components/dashboard/PerformanceChart.tsx`

**Code example:**
```tsx
// Add hidden data table for screen readers
<div className="sr-only">
  <table>
    <caption>BTC/USDT Price Data</caption>
    <thead>
      <tr>
        <th>Time</th>
        <th>Open</th>
        <th>High</th>
        <th>Low</th>
        <th>Close</th>
      </tr>
    </thead>
    <tbody>
      {candles.map((candle, i) => (
        <tr key={i}>
          <td>{new Date(candle.timestamp).toLocaleString()}</td>
          <td>${candle.open}</td>
          <td>${candle.high}</td>
          <td>${candle.low}</td>
          <td>${candle.close}</td>
        </tr>
      ))}
    </tbody>
  </table>
</div>
```

**Checklist:**
- [ ] TradingCharts candlestick data table
- [ ] PerformanceChart data table
- [ ] Strategy SVG charts descriptions
- [ ] All Recharts components with alt text

---

### Task 1.5: ARIA Live Regions (3 hours)
**Files to modify:**
- `src/components/dashboard/TradingCharts.tsx`
- `src/components/dashboard/BotStatus.tsx`

**Code example:**
```tsx
// Add live region for price updates
<div
  aria-live="polite"
  aria-atomic="true"
  className="sr-only"
>
  {symbol} price updated to ${latestPrice}
</div>
```

**Checklist:**
- [ ] Price update announcements
- [ ] Signal alert announcements
- [ ] Position change announcements
- [ ] Error/success toast announcements
- [ ] WebSocket connection status

---

## Sprint 2: Responsive Optimization (HIGH) ⚠️

**Goal:** Improve mobile/tablet experience
**Total Effort:** 10 hours
**Priority:** P1 (Fix in next sprint)

### Task 2.1: Tablet Hamburger Menu (4 hours)
**Files to modify:**
- `src/components/dashboard/DashboardHeader.tsx`

**Implementation:**
```tsx
import { Menu, X } from "lucide-react";
import { Sheet, SheetContent, SheetTrigger } from "@/components/ui/sheet";

// Add mobile menu for tablet sizes
<div className="lg:hidden">
  <Sheet>
    <SheetTrigger asChild>
      <Button variant="ghost" size="sm">
        <Menu className="h-5 w-5" />
        <span className="sr-only">Open menu</span>
      </Button>
    </SheetTrigger>
    <SheetContent side="right">
      <nav className="flex flex-col gap-4 mt-8">
        <Link to="/dashboard">Dashboard</Link>
        <Link to="/trading-paper">Trading Paper</Link>
        <Link to="/settings">Settings</Link>
        <Button onClick={handleLogout}>Logout</Button>
      </nav>
    </SheetContent>
  </Sheet>
</div>
```

**Checklist:**
- [ ] Add Sheet component for mobile menu
- [ ] Hide desktop nav on tablet (lg:hidden)
- [ ] Add menu icon button
- [ ] Style mobile menu items
- [ ] Test on iPad (768px)
- [ ] Test on iPad Pro (1024px)

---

### Task 2.2: Fix Touch Target Sizes (3 hours)
**Files to scan:**
- All components with buttons/links
- Settings page tabs
- Trading chart controls

**Rule:** All interactive elements must be ≥ 44x44px

**Code fixes:**
```tsx
// Before (too small)
<Button size="sm" className="h-6 w-6 p-0">
  <X className="h-3 w-3" />
</Button>

// After (minimum 44px)
<Button size="sm" className="h-11 w-11 p-0">
  <X className="h-4 w-4" />
</Button>
```

**Checklist:**
- [ ] Audit all Button components
- [ ] Check Settings page tabs
- [ ] Check chart remove buttons
- [ ] Check navigation links
- [ ] Check form inputs
- [ ] Test on physical mobile device
- [ ] Measure with browser dev tools

---

### Task 2.3: Add Custom Font (1 hour)
**Files to modify:**
- `src/index.css`
- `tailwind.config.ts`

**Implementation:**
```css
/* src/index.css */
@import url('https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap');

@layer base {
  body {
    font-family: 'Inter', system-ui, -apple-system, sans-serif;
  }
}
```

```ts
// tailwind.config.ts
export default {
  theme: {
    extend: {
      fontFamily: {
        sans: ['Inter', 'system-ui', 'sans-serif'],
        mono: ['JetBrains Mono', 'monospace'], // For financial data
      },
    },
  },
}
```

**Checklist:**
- [ ] Add Google Fonts import
- [ ] Update Tailwind config
- [ ] Test Vietnamese characters
- [ ] Verify font weights (300-700)
- [ ] Check bundle size impact
- [ ] Add font-display: swap

---

### Task 2.4: Reduced Motion Support (2 hours)
**Files to modify:**
- `src/index.css`

**Code to add:**
```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
  }

  /* Disable specific animations */
  .animate-pulse,
  .animate-spin,
  .animate-bounce {
    animation: none !important;
  }
}
```

**Checklist:**
- [ ] Add CSS media query
- [ ] Test with browser settings
- [ ] Verify pulse animations stop
- [ ] Verify transitions still work (instant)
- [ ] Document for designers

---

## Sprint 3: UX Enhancement (MEDIUM) 💡

**Goal:** Improve onboarding and activation
**Total Effort:** 17 hours
**Priority:** P2 (Nice to have)

### Task 3.1: Interactive Product Tour (8 hours)
**Dependencies:** Install `react-joyride`

```bash
npm install react-joyride
```

**Files to create:**
- `src/components/ProductTour.tsx`

**Implementation:**
```tsx
import Joyride from 'react-joyride';

const steps = [
  {
    target: '.bot-status',
    content: 'Monitor your account balance and open positions here',
  },
  {
    target: '.ai-signals',
    content: 'AI-powered trading signals with confidence scores',
  },
  // ... more steps
];

export function ProductTour() {
  const [run, setRun] = useState(false);

  return (
    <Joyride
      steps={steps}
      run={run}
      continuous
      showProgress
      showSkipButton
      styles={{
        options: {
          primaryColor: 'hsl(var(--profit))',
        },
      }}
    />
  );
}
```

**Checklist:**
- [ ] Install react-joyride
- [ ] Create ProductTour component
- [ ] Define 8-10 tour steps
- [ ] Style tooltips to match theme
- [ ] Add "Start Tour" button to dashboard
- [ ] Save tour completion to localStorage
- [ ] Test on mobile/tablet/desktop

---

### Task 3.2: Portfolio Quick Actions (3 hours)
**Files to modify:**
- `src/components/dashboard/BotStatus.tsx`

**Add quick action buttons:**
```tsx
<div className="flex gap-2 mt-2">
  <Button size="sm" variant="outline" onClick={() => closePosition(position.id)}>
    Close Position
  </Button>
  <Button size="sm" variant="ghost" onClick={() => editStopLoss(position.id)}>
    Edit SL/TP
  </Button>
</div>
```

**Checklist:**
- [ ] Add Close Position button
- [ ] Add Edit Stop Loss button
- [ ] Add confirmation dialog
- [ ] Implement API calls
- [ ] Add loading states
- [ ] Add success/error toasts

---

### Task 3.3: Strategy Comparison Tool (6 hours)
**Files to create:**
- `src/components/dashboard/StrategyComparison.tsx`

**Feature:** Side-by-side comparison of multiple strategies

**Checklist:**
- [ ] Create comparison UI
- [ ] Allow selecting 2-4 strategies
- [ ] Show pros/cons side-by-side
- [ ] Display performance metrics
- [ ] Add visual comparison chart
- [ ] Save comparisons to localStorage

---

## Sprint 4+: Long-term Enhancements (LOW) 🔮

### Task 4.1: Dark/Light Mode Toggle (8 hours)
**Implementation:** Use `next-themes` (already in package.json)

**Checklist:**
- [ ] Configure theme provider
- [ ] Add toggle button to header
- [ ] Define light theme colors
- [ ] Test all components in light mode
- [ ] Add theme persistence
- [ ] Update documentation

---

### Task 4.2: Keyboard Shortcuts (6 hours)
**Dependencies:** Install `react-hotkeys-hook`

**Shortcuts to implement:**
- `Ctrl+K` - Command palette
- `Ctrl+D` - Dashboard
- `Ctrl+S` - Settings
- `Ctrl+T` - Trading Paper
- `Esc` - Close dialogs

**Checklist:**
- [ ] Install react-hotkeys-hook
- [ ] Create KeyboardShortcuts component
- [ ] Add shortcuts documentation modal
- [ ] Test on Mac (Cmd) and Windows (Ctrl)
- [ ] Add visual hints (tooltips)

---

## Testing Checklist

### Accessibility Testing
- [ ] Run axe DevTools (0 violations)
- [ ] Run Lighthouse (score ≥ 90)
- [ ] Test with NVDA screen reader
- [ ] Test keyboard-only navigation
- [ ] Test high contrast mode
- [ ] Test with browser zoom (200%)

### Responsive Testing
- [ ] iPhone SE (375px)
- [ ] iPhone 12 Pro (390px)
- [ ] iPad (768px)
- [ ] iPad Pro (1024px)
- [ ] Desktop 1440px
- [ ] Desktop 1920px

### Cross-browser Testing
- [ ] Chrome (latest)
- [ ] Firefox (latest)
- [ ] Safari (latest)
- [ ] Edge (latest)
- [ ] Mobile Safari (iOS)
- [ ] Chrome Mobile (Android)

### Performance Testing
- [ ] Lighthouse Performance ≥ 90
- [ ] Bundle size < 500KB
- [ ] First Contentful Paint < 1.5s
- [ ] Time to Interactive < 3s
- [ ] No layout shifts (CLS = 0)

---

## Progress Tracking

**Sprint 1 Progress:** 0/14 hours (0%)
- [ ] Task 1.1: Alt text (2h)
- [ ] Task 1.2: Icons (3h)
- [ ] Task 1.3: Focus (2h)
- [ ] Task 1.4: Tables (4h)
- [ ] Task 1.5: ARIA live (3h)

**Sprint 2 Progress:** 0/10 hours (0%)
- [ ] Task 2.1: Hamburger (4h)
- [ ] Task 2.2: Touch targets (3h)
- [ ] Task 2.3: Font (1h)
- [ ] Task 2.4: Reduced motion (2h)

**Sprint 3 Progress:** 0/17 hours (0%)
- [ ] Task 3.1: Product tour (8h)
- [ ] Task 3.2: Quick actions (3h)
- [ ] Task 3.3: Comparison tool (6h)

**Total Progress:** 0/41 hours (0%)

---

## Reference Links

- **Full Evaluation:** `/docs/UI_UX_DESIGN_EVALUATION.md`
- **Summary:** `/docs/DESIGN_EVALUATION_SUMMARY.md`
- **WCAG Guidelines:** https://www.w3.org/WAI/WCAG21/quickref/
- **Shadcn/UI Docs:** https://ui.shadcn.com/
- **Tailwind Docs:** https://tailwindcss.com/docs

---

**Last Updated:** 2025-11-19
**Maintained by:** UI/UX Design Team
**Sprint Planning:** Use this checklist for sprint estimation and tracking
