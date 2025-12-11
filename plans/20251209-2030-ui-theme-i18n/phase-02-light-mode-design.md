# Phase 02: Light Mode Design

**Priority**: Critical | **Status**: Pending | **Est. Effort**: 3 hours

---

## Context Links

- [Main Plan](./plan.md)
- [Phase 01: Theme Infrastructure](./phase-01-theme-infrastructure.md)
- [Theming Research](./research/researcher-251209-theming-system.md)

---

## Overview

Design and implement luxury light mode CSS variables that complement existing dark OLED theme. Maintain premium trading aesthetic while ensuring WCAG AA contrast ratios.

---

## Key Insights

1. **Current dark theme** is OLED-optimized (#000000 background) with premium blue/cyan accents
2. **Light mode must feel equally premium** - not generic white but sophisticated off-white/cream
3. **Trading colors (profit/loss)** must remain consistent across themes for recognition
4. **17 CSS variables** defined in `:root` need light mode equivalents
5. **Shadcn/UI pattern** - Light in `:root`, dark in `.dark` class (current is inverted)

---

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | Light mode CSS variables for all 17 Shadcn variables | Critical |
| R2 | WCAG AA contrast (4.5:1 text, 3:1 UI elements) | Critical |
| R3 | Luxury aesthetic (soft whites, refined accents) | High |
| R4 | Trading colors unchanged (green profit, red loss) | High |
| R5 | Sidebar variables for both modes | High |
| R6 | Chart colors optimized for light background | Medium |
| R7 | Glassmorphism utilities for light mode | Medium |

---

## Architecture

**Color Strategy**:
```
DARK MODE (current :root)          LIGHT MODE (new :root, move dark to .dark)
--background: 0 0% 0%          →   --background: 0 0% 99%     (soft white)
--foreground: 0 0% 100%        →   --foreground: 0 0% 8%      (near black)
--card: 240 35% 9%             →   --card: 0 0% 100%          (pure white)
--primary: 224 100% 58%        →   --primary: 224 100% 50%    (slightly deeper)
--accent: 189 100% 50%         →   --accent: 189 90% 40%      (muted cyan)
```

---

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/index.css` | MODIFY | Restructure CSS vars (light in :root, dark in .dark) |

---

## Implementation Steps

### Step 1: Restructure index.css CSS Variables

Move current `:root` values to `.dark` class, define new light mode in `:root`:

```css
@layer base {
  :root {
    /* ============================================================
       Light Mode - Premium Trading Dashboard
       Sophisticated off-white with refined accents
       ============================================================ */

    /* Core backgrounds - Soft luxury whites */
    --background: 0 0% 99%;           /* Off-white, not pure white */
    --foreground: 0 0% 8%;            /* Near black for text */

    /* Card - Pure white for elevation against bg */
    --card: 0 0% 100%;
    --card-foreground: 0 0% 8%;

    /* Popover - Same as card */
    --popover: 0 0% 100%;
    --popover-foreground: 0 0% 8%;

    /* Primary - Slightly deeper blue for light bg */
    --primary: 224 90% 48%;
    --primary-foreground: 0 0% 100%;

    /* Secondary - Light gray surface */
    --secondary: 240 10% 96%;
    --secondary-foreground: 240 10% 25%;

    /* Muted - Very light gray */
    --muted: 240 10% 96%;
    --muted-foreground: 240 5% 45%;

    /* Accent - Muted cyan for light mode */
    --accent: 189 80% 42%;
    --accent-foreground: 0 0% 100%;

    /* Destructive - Slightly darker red for light bg */
    --destructive: 0 80% 50%;
    --destructive-foreground: 0 0% 100%;

    /* Borders - Subtle gray */
    --border: 240 10% 90%;
    --input: 240 10% 96%;
    --ring: 224 90% 48%;

    /* Trading colors - CONSISTENT across themes */
    --profit: 160 84% 35%;            /* Slightly darker green */
    --profit-foreground: 0 0% 100%;
    --loss: 0 84% 55%;                /* Slightly darker red */
    --warning: 38 92% 45%;
    --info: 224 90% 48%;

    /* Chart colors - Optimized for white bg */
    --chart-1: 160 84% 35%;
    --chart-2: 0 84% 55%;
    --chart-3: 44 89% 50%;
    --chart-4: 224 90% 48%;
    --chart-5: 270 70% 55%;

    --radius: 0.5rem;

    /* Paper/Real mode - Keep consistent */
    --paper-accent: #0891B2;
    --paper-badge: #0891B2;
    --paper-border: #0891B2;
    --paper-background: #E0F7FA;
    --paper-hover: #22D3EE;

    --real-warning: #DC2626;
    --real-banner: #DC2626;
    --real-border: #DC2626;
    --real-background: #FEE2E2;
    --real-hover: #EF4444;

    /* Design System Extended Colors - Light */
    --bg-primary: #FAFAFA;
    --bg-secondary: #F5F5F7;
    --bg-tertiary: #EEEEF0;
    --bg-surface: #FFFFFF;

    --text-primary: #1A1A1A;
    --text-secondary: #6B6B7A;
    --text-muted: #9A9AAE;

    --grid-color: #E8E8EC;
    --border-color: rgba(0, 0, 0, 0.08);

    /* Brand Colors - Adjusted for light */
    --brand-blue: #1D4ED8;
    --brand-cyan: #0891B2;
    --brand-gold: #D97706;
    --brand-purple: #7C3AED;

    /* Sidebar - Light luxury */
    --sidebar-background: 0 0% 100%;
    --sidebar-foreground: 0 0% 8%;
    --sidebar-primary: 224 90% 48%;
    --sidebar-primary-foreground: 0 0% 100%;
    --sidebar-accent: 240 10% 96%;
    --sidebar-accent-foreground: 240 10% 25%;
    --sidebar-border: 240 10% 90%;
    --sidebar-ring: 224 90% 48%;
  }

  .dark {
    /* ============================================================
       Dark OLED Luxury Theme - Premium Trading Dashboard
       (Move existing :root values here)
       ============================================================ */
    --background: 0 0% 0%;
    --foreground: 0 0% 100%;
    --card: 240 35% 9%;
    --card-foreground: 0 0% 100%;
    /* ... (copy all existing :root values) ... */
  }
}
```

### Step 2: Update Glassmorphism Utilities

```css
@layer utilities {
  /* Light mode glassmorphism */
  .glass {
    @apply backdrop-blur-md bg-white/70 border border-gray-200/50 rounded-xl shadow-xl;
  }
  .dark .glass {
    @apply bg-slate-900/70 border-slate-700/50;
  }

  .glass-sm {
    @apply backdrop-blur-sm bg-white/60 border border-gray-200/40 rounded-lg shadow-lg;
  }
  .dark .glass-sm {
    @apply bg-slate-900/60 border-slate-700/40;
  }
}
```

### Step 3: Verify Contrast Ratios

Test all color combinations meet WCAG AA:
- Primary text on background: 4.5:1 minimum
- Secondary text on background: 4.5:1 minimum
- UI elements: 3:1 minimum
- Focus indicators: 3:1 minimum

### Step 4: Update Scrollbar Colors for Light Mode

```css
/* Light mode scrollbar */
.custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(0, 0, 0, 0.03);
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0, 120, 180, 0.25);
}
.dark .custom-scrollbar::-webkit-scrollbar-track {
  background: rgba(255, 255, 255, 0.05);
}
.dark .custom-scrollbar::-webkit-scrollbar-thumb {
  background: rgba(0, 217, 255, 0.3);
}
```

---

## Color Palette Reference

### Light Mode Luxury Palette

| Variable | HSL | Hex | Use |
|----------|-----|-----|-----|
| background | 0 0% 99% | #FCFCFC | Page background |
| foreground | 0 0% 8% | #141414 | Primary text |
| card | 0 0% 100% | #FFFFFF | Card surfaces |
| primary | 224 90% 48% | #1E5AE8 | CTA buttons, links |
| accent | 189 80% 42% | #1498B7 | Highlights |
| profit | 160 84% 35% | #0F9964 | Positive values |
| loss | 0 84% 55% | #E8313D | Negative values |

### Dark Mode (Existing)

| Variable | HSL | Hex | Use |
|----------|-----|-----|-----|
| background | 0 0% 0% | #000000 | OLED black |
| foreground | 0 0% 100% | #FFFFFF | Primary text |
| card | 240 35% 9% | #0F0F1E | Card surfaces |
| primary | 224 100% 58% | #2962FF | CTA buttons |
| accent | 189 100% 50% | #00D9FF | Neon highlights |
| profit | 160 84% 39% | #10B981 | Positive values |
| loss | 0 84% 60% | #EF4444 | Negative values |

---

## Todo List

- [ ] Move current `:root` dark theme values to `.dark` class
- [ ] Define new light mode values in `:root`
- [ ] Update extended design system colors for light mode
- [ ] Update glassmorphism utilities with theme variants
- [ ] Update scrollbar colors with theme variants
- [ ] Verify WCAG AA contrast ratios with browser tools
- [ ] Test profit/loss colors visibility in both modes
- [ ] Test charts readability in light mode

---

## Success Criteria

- [ ] Light mode looks premium, not generic
- [ ] All text passes WCAG AA contrast (4.5:1)
- [ ] UI elements pass WCAG AA contrast (3:1)
- [ ] Trading colors (green/red) equally visible in both modes
- [ ] Glassmorphism effects work in both modes
- [ ] Charts are readable in light mode
- [ ] No hardcoded colors in components (all use CSS vars)

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Contrast failures | Medium | High | Use contrast checker tool before finalizing |
| Hardcoded colors in components | Medium | Medium | Grep for rgb/hex values, refactor |
| Charts unreadable | Low | Medium | Test all 5 chart colors against white |
| Trading colors confused | Low | High | Keep profit/loss consistent across themes |

---

## Security Considerations

- No security implications - purely visual CSS changes
- No user data involved

---

## Test Cases

| ID | Test Case | Expected Result |
|----|-----------|-----------------|
| TC-01 | View dashboard in light mode | All text readable, no contrast issues |
| TC-02 | View trading page profit/loss | Green/red clearly distinguishable |
| TC-03 | View charts in light mode | All 5 chart colors visible |
| TC-04 | Toggle theme rapidly | No visual glitches, smooth transition |
| TC-05 | Test glassmorphism in light mode | Blur effect visible, not washed out |
| TC-06 | Inspect contrast with DevTools | All combinations >= 4.5:1 for text |
| TC-07 | View in sunlight (high brightness) | Light mode remains readable |

---

## Next Steps

After this phase:
1. [Phase 03: i18n Restructure](./phase-03-i18n-restructure.md) - Namespace-based i18n
