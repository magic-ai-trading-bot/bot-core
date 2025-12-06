# Phase 6: Polish & Testing

**Status**: Pending | **Priority**: P1 | **Est. Time**: 2-3 days

---

## Context

- [Main Plan](./plan.md)
- [Phase 4: Real Trading UI](./phase-04-real-trading-ui.md)
- [Phase 5: 3D Visualization](./phase-05-3d-visualization.md)

## Overview

Final polish pass: animation refinements, E2E tests for both trading modes, performance optimization, accessibility audit, and mobile responsiveness verification.

## Key Insights

1. **Animation Polish** - 300ms transitions, easing functions, micro-interactions
2. **E2E Critical Paths** - Mode switching, trade confirmation, safety flows
3. **Performance Budget** - <100ms interaction, <3s initial load
4. **Accessibility** - WCAG 2.1 AA compliance, keyboard navigation

## Requirements

### Functional
- [ ] E2E tests for paper trading flow
- [ ] E2E tests for real trading flow
- [ ] E2E tests for mode switching
- [ ] Animation polish pass
- [ ] Mobile responsive verification

### Non-Functional
- [ ] <100ms interaction response
- [ ] <3s initial page load
- [ ] 90%+ test coverage
- [ ] WCAG 2.1 AA compliance
- [ ] Zero accessibility violations

## Architecture

### Test Structure

```
__tests__/
├── e2e/
│   ├── paper-trading.spec.ts     # Paper trading flow
│   ├── real-trading.spec.ts      # Real trading flow
│   ├── mode-switching.spec.ts    # Mode transitions
│   ├── safety-confirmations.spec.ts # 2-step confirmations
│   └── mobile-responsive.spec.ts # Mobile viewports
├── integration/
│   ├── trading-mode-context.test.tsx
│   ├── use-real-trading.test.ts
│   └── use-paper-trading.test.ts
└── components/
    ├── ModeSwitcher.test.tsx
    ├── TradeConfirmation.test.tsx
    └── PortfolioOverview.test.tsx
```

### Performance Budget

| Metric | Budget | Measurement |
|--------|--------|-------------|
| First Contentful Paint | <1.5s | Lighthouse |
| Time to Interactive | <3s | Lighthouse |
| Largest Contentful Paint | <2.5s | Lighthouse |
| Cumulative Layout Shift | <0.1 | Lighthouse |
| Interaction Response | <100ms | Custom timing |
| Animation Frame Rate | 60fps | Chrome DevTools |

## Related Files

| File | Path | Action |
|------|------|--------|
| E2E Paper Trading | `/nextjs-ui-dashboard/__tests__/e2e/paper-trading.spec.ts` | Create |
| E2E Real Trading | `/nextjs-ui-dashboard/__tests__/e2e/real-trading.spec.ts` | Create |
| E2E Mode Switching | `/nextjs-ui-dashboard/__tests__/e2e/mode-switching.spec.ts` | Create |
| Component Tests | `/nextjs-ui-dashboard/src/components/**/*.test.tsx` | Create/Update |
| Animation Utils | `/nextjs-ui-dashboard/src/lib/animations.ts` | Create |

## Implementation Steps

### Step 1: Create E2E Test for Mode Switching

```typescript
// __tests__/e2e/mode-switching.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Trading Mode Switching', () => {
  test.beforeEach(async ({ page }) => {
    await page.goto('/trading/paper')
  })

  test('displays paper mode by default', async ({ page }) => {
    await expect(page.getByTestId('mode-badge')).toContainText('SANDBOX')
    await expect(page).toHaveURL('/trading/paper')
  })

  test('requires confirmation to switch to real mode', async ({ page }) => {
    // Click real mode button
    await page.getByRole('button', { name: 'Real' }).click()

    // Confirmation dialog should appear
    await expect(page.getByText('Switch to Real Trading?')).toBeVisible()
    await expect(page.getByText('WARNING: You will be trading with REAL MONEY')).toBeVisible()

    // Cancel should close dialog
    await page.getByRole('button', { name: 'Cancel' }).click()
    await expect(page.getByText('Switch to Real Trading?')).not.toBeVisible()
    await expect(page).toHaveURL('/trading/paper')
  })

  test('switches to real mode after confirmation', async ({ page }) => {
    await page.getByRole('button', { name: 'Real' }).click()
    await page.getByRole('button', { name: 'I Understand, Switch to Real' }).click()

    // Should navigate and show real mode indicators
    await expect(page).toHaveURL('/trading/real')
    await expect(page.getByTestId('mode-badge')).toContainText('REAL MONEY')
    await expect(page.getByTestId('warning-banner')).toBeVisible()
  })

  test('persists mode preference across page reloads', async ({ page }) => {
    // Switch to real mode
    await page.getByRole('button', { name: 'Real' }).click()
    await page.getByRole('button', { name: 'I Understand, Switch to Real' }).click()

    // Reload page
    await page.reload()

    // Should still be in real mode
    await expect(page).toHaveURL('/trading/real')
    await expect(page.getByTestId('mode-badge')).toContainText('REAL MONEY')
  })

  test('browser back button works correctly', async ({ page }) => {
    // Switch to real mode
    await page.getByRole('button', { name: 'Real' }).click()
    await page.getByRole('button', { name: 'I Understand, Switch to Real' }).click()
    await expect(page).toHaveURL('/trading/real')

    // Go back
    await page.goBack()
    await expect(page).toHaveURL('/trading/paper')
  })
})
```

### Step 2: Create E2E Test for Real Trading Safety

```typescript
// __tests__/e2e/real-trading.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Real Trading Safety', () => {
  test.beforeEach(async ({ page }) => {
    // Start in real mode
    await page.goto('/trading/real')
  })

  test('warning banner is always visible', async ({ page }) => {
    await expect(page.getByTestId('warning-banner')).toBeVisible()
    await expect(page.getByText('REAL MONEY AT RISK')).toBeVisible()
  })

  test('trade requires 2-step confirmation', async ({ page }) => {
    // Trigger a trade
    await page.getByRole('button', { name: 'Place Order' }).click()

    // Step 1: Preview with acknowledgment
    await expect(page.getByText('Order Preview')).toBeVisible()
    await expect(page.getByRole('checkbox')).toBeVisible()

    // Cannot proceed without acknowledgment
    await expect(page.getByRole('button', { name: 'Review Order' })).toBeDisabled()

    // Check acknowledgment
    await page.getByRole('checkbox').check()
    await expect(page.getByRole('button', { name: 'Review Order' })).toBeEnabled()

    // Step 2: Final confirmation
    await page.getByRole('button', { name: 'Review Order' }).click()
    await expect(page.getByText('Confirm Real Trade')).toBeVisible()
    await expect(page.getByText('This action is IRREVERSIBLE')).toBeVisible()
  })

  test('position close requires confirmation', async ({ page }) => {
    // Assuming there's an open position
    await page.getByTestId('close-position-btn').first().click()

    // Confirmation dialog should appear
    await expect(page.getByText('Close Position?')).toBeVisible()
    await expect(page.getByText('This will close your position at market price')).toBeVisible()
  })

  test('daily loss limit is displayed', async ({ page }) => {
    await expect(page.getByTestId('daily-loss-limit')).toBeVisible()
    await expect(page.getByText('Daily Loss Limit')).toBeVisible()
  })

  test('exchange connection status is visible', async ({ page }) => {
    await expect(page.getByTestId('exchange-status')).toBeVisible()
  })
})
```

### Step 3: Animation Polish Pass

```typescript
// src/lib/animations.ts
import { Variants } from 'framer-motion'

// Standard easing
export const easing = {
  smooth: [0.4, 0, 0.2, 1],
  bounce: [0.68, -0.55, 0.265, 1.55],
  snappy: [0.23, 1, 0.32, 1]
}

// Page transitions
export const pageVariants: Variants = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0, transition: { duration: 0.3, ease: easing.smooth } },
  exit: { opacity: 0, y: -20, transition: { duration: 0.2 } }
}

// Card variants
export const cardVariants: Variants = {
  initial: { opacity: 0, scale: 0.95 },
  animate: { opacity: 1, scale: 1, transition: { duration: 0.3 } },
  hover: { scale: 1.02, transition: { duration: 0.2 } },
  tap: { scale: 0.98 }
}

// Number change animation
export const numberVariants: Variants = {
  initial: { opacity: 0, y: -10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: 10 }
}

// Stagger children
export const staggerContainer: Variants = {
  animate: {
    transition: {
      staggerChildren: 0.1
    }
  }
}

// Price flash (profit/loss)
export const priceFlash = (isProfit: boolean): Variants => ({
  flash: {
    backgroundColor: [
      'rgba(0,0,0,0)',
      isProfit ? 'rgba(16, 185, 129, 0.2)' : 'rgba(239, 68, 68, 0.2)',
      'rgba(0,0,0,0)'
    ],
    transition: { duration: 0.5 }
  }
})

// Warning pulse
export const warningPulse: Variants = {
  pulse: {
    opacity: [1, 0.7, 1],
    transition: {
      duration: 2,
      repeat: Infinity,
      ease: 'easeInOut'
    }
  }
}
```

### Step 4: Accessibility Audit Checklist

```typescript
// __tests__/a11y/accessibility.spec.ts
import { test, expect } from '@playwright/test'
import AxeBuilder from '@axe-core/playwright'

test.describe('Accessibility', () => {
  test('paper trading page has no violations', async ({ page }) => {
    await page.goto('/trading/paper')
    const results = await new AxeBuilder({ page }).analyze()
    expect(results.violations).toEqual([])
  })

  test('real trading page has no violations', async ({ page }) => {
    await page.goto('/trading/real')
    const results = await new AxeBuilder({ page }).analyze()
    expect(results.violations).toEqual([])
  })

  test('mode switcher is keyboard accessible', async ({ page }) => {
    await page.goto('/trading/paper')

    // Tab to mode switcher
    await page.keyboard.press('Tab')
    await page.keyboard.press('Tab')

    // Focus should be on mode switcher
    const focusedElement = await page.locator(':focus')
    await expect(focusedElement).toHaveAttribute('role', 'button')

    // Space/Enter should activate
    await page.keyboard.press('Enter')
    await expect(page.getByText('Switch to Real Trading?')).toBeVisible()
  })

  test('confirmation dialog traps focus', async ({ page }) => {
    await page.goto('/trading/paper')
    await page.getByRole('button', { name: 'Real' }).click()

    // Tab should cycle within dialog
    await page.keyboard.press('Tab')
    await page.keyboard.press('Tab')
    await page.keyboard.press('Tab')

    // Focus should still be within dialog
    const focusedElement = await page.locator(':focus')
    const dialog = page.getByRole('dialog')
    await expect(dialog).toContainElement(focusedElement)
  })

  test('escape closes dialogs', async ({ page }) => {
    await page.goto('/trading/paper')
    await page.getByRole('button', { name: 'Real' }).click()
    await expect(page.getByRole('dialog')).toBeVisible()

    await page.keyboard.press('Escape')
    await expect(page.getByRole('dialog')).not.toBeVisible()
  })
})
```

### Step 5: Performance Testing

```typescript
// __tests__/performance/metrics.spec.ts
import { test, expect } from '@playwright/test'

test.describe('Performance', () => {
  test('paper trading loads under 3s', async ({ page }) => {
    const startTime = Date.now()
    await page.goto('/trading/paper', { waitUntil: 'networkidle' })
    const loadTime = Date.now() - startTime

    expect(loadTime).toBeLessThan(3000)
  })

  test('mode switch responds under 100ms', async ({ page }) => {
    await page.goto('/trading/paper')

    // Measure click to dialog open
    const startTime = Date.now()
    await page.getByRole('button', { name: 'Real' }).click()
    await page.getByText('Switch to Real Trading?').waitFor()
    const responseTime = Date.now() - startTime

    expect(responseTime).toBeLessThan(100)
  })

  test('animations run at 60fps', async ({ page }) => {
    await page.goto('/trading/paper')

    // Start performance tracing
    await page.tracing.start({ screenshots: true })

    // Trigger animations
    await page.getByRole('button', { name: 'Real' }).click()
    await page.waitForTimeout(500)
    await page.getByRole('button', { name: 'Cancel' }).click()

    // Stop tracing and analyze
    const trace = await page.tracing.stop()
    // Analyze trace for dropped frames (done externally)
  })
})
```

### Step 6: Mobile Responsive Testing

```typescript
// __tests__/e2e/mobile-responsive.spec.ts
import { test, expect, devices } from '@playwright/test'

const mobileDevices = [
  devices['iPhone 12'],
  devices['iPhone SE'],
  devices['Pixel 5'],
  devices['Galaxy S8']
]

for (const device of mobileDevices) {
  test.describe(`Mobile: ${device.viewport.width}x${device.viewport.height}`, () => {
    test.use({ ...device })

    test('trading page is responsive', async ({ page }) => {
      await page.goto('/trading/paper')

      // No horizontal scrollbar
      const body = await page.locator('body')
      const scrollWidth = await body.evaluate(el => el.scrollWidth)
      const clientWidth = await body.evaluate(el => el.clientWidth)
      expect(scrollWidth).toBeLessThanOrEqual(clientWidth)
    })

    test('warning banner is visible on mobile', async ({ page }) => {
      await page.goto('/trading/real')
      await expect(page.getByTestId('warning-banner')).toBeVisible()
    })

    test('mode switcher works on mobile', async ({ page }) => {
      await page.goto('/trading/paper')

      // Tap mode switcher
      await page.tap('[data-testid="mode-switcher"] button:has-text("Real")')
      await expect(page.getByRole('dialog')).toBeVisible()
    })

    test('confirmation dialog fits screen', async ({ page }) => {
      await page.goto('/trading/paper')
      await page.tap('[data-testid="mode-switcher"] button:has-text("Real")')

      const dialog = page.getByRole('dialog')
      const dialogBox = await dialog.boundingBox()

      expect(dialogBox?.width).toBeLessThanOrEqual(device.viewport.width)
      expect(dialogBox?.height).toBeLessThanOrEqual(device.viewport.height)
    })
  })
}
```

## Todo List

### Testing
- [ ] Create E2E tests for mode switching
- [ ] Create E2E tests for paper trading flow
- [ ] Create E2E tests for real trading safety
- [ ] Create E2E tests for 2-step confirmation
- [ ] Create accessibility tests with axe-core
- [ ] Create performance measurement tests
- [ ] Create mobile responsive tests
- [ ] Achieve 90%+ test coverage

### Animation Polish
- [ ] Create animation utility library
- [ ] Apply page transition animations
- [ ] Add card hover/tap animations
- [ ] Implement number change animations
- [ ] Add price flash effects
- [ ] Polish warning pulse animation
- [ ] Test reduced motion support

### Performance
- [ ] Run Lighthouse audit
- [ ] Optimize bundle size
- [ ] Implement code splitting
- [ ] Add lazy loading for 3D components
- [ ] Profile and fix memory leaks
- [ ] Verify <100ms interaction response

### Accessibility
- [ ] Run axe-core audit
- [ ] Fix all WCAG AA violations
- [ ] Test keyboard navigation
- [ ] Verify screen reader support
- [ ] Test focus management in dialogs
- [ ] Add ARIA labels where needed

### Mobile
- [ ] Test on iPhone (Safari)
- [ ] Test on Android (Chrome)
- [ ] Verify touch targets >= 44px
- [ ] Test dialog responsiveness
- [ ] Verify warning banner on small screens

## Success Criteria

1. All E2E tests pass (100%)
2. Test coverage >= 90%
3. Zero accessibility violations (axe-core)
4. Lighthouse Performance score >= 90
5. All animations at 60fps
6. Works on mobile devices (iOS/Android)
7. Mode switching flow verified safe

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Flaky E2E tests | Medium | Medium | Retry logic, stable selectors |
| Accessibility issues | Low | High | Early audit, fix iteratively |
| Performance regression | Low | Medium | Continuous monitoring |
| Mobile bugs | Medium | Medium | Real device testing |

## Security Considerations

- E2E tests should not expose credentials
- Use test environment, not production
- Mock real trading API in tests
- Verify confirmation dialogs cannot be bypassed

## Final Checklist

Before release:

- [ ] All phases complete (1-6)
- [ ] All tests passing
- [ ] Accessibility audit passed
- [ ] Performance budget met
- [ ] Mobile testing complete
- [ ] Documentation updated
- [ ] Code review completed
- [ ] Staging environment tested
- [ ] Rollback plan prepared

## Next Steps

After Phase 6 completion:
1. Deploy to staging environment
2. Conduct UAT (User Acceptance Testing)
3. Gradual rollout to production
4. Monitor error rates and user feedback
