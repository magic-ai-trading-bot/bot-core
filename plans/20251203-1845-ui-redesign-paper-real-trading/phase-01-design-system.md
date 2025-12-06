# Phase 1: Design System Foundation

**Status**: Pending | **Priority**: P0 | **Est. Time**: 2-3 days

---

## Context

- [Main Plan](./plan.md)
- [Trading UI Patterns Research](./research/researcher-01-trading-ui-patterns.md)
- [3D Visualization Research](./research/researcher-02-3d-visualization-design.md)

## Overview

Establish design tokens, color system, typography, and animation primitives for consistent Paper/Real mode visual separation.

## Key Insights

1. **Color-Coded Mode Distinction** - Blue (#0EA5E9) for Paper, standard dark for Real with RED warnings
2. **Dark Mode Primary** - #0F172A background, #1E293B grid lines, #F3F4F6 text
3. **Glassmorphism for Premium Feel** - Frosted glass cards with proper contrast (>4.5:1)
4. **Framer Motion** - 300ms transitions for smooth, professional feel

## Requirements

### Functional
- [ ] Mode-specific color tokens (paper-accent, real-warning)
- [ ] Consistent profit/loss colors across both modes
- [ ] Animation variants for cards, modals, numbers
- [ ] Typography scale for trading data density

### Non-Functional
- [ ] WCAG 2.1 AA contrast compliance
- [ ] Reduced motion support
- [ ] <16ms animation frame time

## Architecture

### Design Token Structure
```
src/styles/
├── tokens/
│   ├── colors.ts         # Color palette with mode variants
│   ├── typography.ts     # Font sizes, weights, line heights
│   ├── spacing.ts        # Consistent spacing scale
│   └── animations.ts     # Framer Motion variants
├── themes/
│   ├── paper-theme.ts    # Paper mode overrides
│   └── real-theme.ts     # Real mode overrides
└── index.ts              # Exports
```

### Color Palette

```typescript
// colors.ts
export const colors = {
  // Backgrounds
  bg: {
    primary: '#0F172A',    // Main background
    secondary: '#1E293B',  // Cards
    tertiary: '#334155',   // Elevated elements
  },

  // Mode-specific
  paper: {
    accent: '#0EA5E9',     // Sky blue
    badge: '#0284C7',      // Darker blue for badge
    border: '#0EA5E9/20',  // Subtle border
  },
  real: {
    warning: '#EF4444',    // Red warning
    banner: '#DC2626',     // Darker red for banner
    border: '#EF4444/30',  // Warning border
  },

  // Trading
  profit: '#10B981',       // Green
  loss: '#EF4444',         // Red
  neutral: '#64748B',      // Slate

  // Text
  text: {
    primary: '#F3F4F6',    // Main text
    secondary: '#94A3B8',  // Muted
    muted: '#64748B',      // Very muted
  },

  // Grid/Lines
  grid: '#374151',
}
```

### Animation Variants

```typescript
// animations.ts
export const fadeIn = {
  initial: { opacity: 0 },
  animate: { opacity: 1 },
  exit: { opacity: 0 },
  transition: { duration: 0.3 }
}

export const slideUp = {
  initial: { opacity: 0, y: 20 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -20 },
  transition: { duration: 0.3, ease: 'easeOut' }
}

export const numberChange = {
  initial: { scale: 1 },
  animate: { scale: [1, 1.05, 1] },
  transition: { duration: 0.3 }
}

export const pulse = {
  animate: {
    backgroundColor: ['rgba(16, 185, 129, 0)', 'rgba(16, 185, 129, 0.2)', 'rgba(16, 185, 129, 0)']
  },
  transition: { duration: 0.5 }
}
```

## Related Files

| File | Action |
|------|--------|
| `/nextjs-ui-dashboard/src/index.css` | Add CSS variables |
| `/nextjs-ui-dashboard/tailwind.config.js` | Extend theme |
| `/nextjs-ui-dashboard/src/styles/tokens/` | Create new |
| `/nextjs-ui-dashboard/src/components/ui/` | Update for variants |

## Implementation Steps

### Step 1: Install Dependencies
```bash
cd /Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard
npm install framer-motion
```

### Step 2: Create Token Files
1. Create `src/styles/tokens/colors.ts` with full palette
2. Create `src/styles/tokens/typography.ts` with scale
3. Create `src/styles/tokens/animations.ts` with Framer variants
4. Create `src/styles/tokens/spacing.ts` with scale

### Step 3: Update TailwindCSS Config
```javascript
// tailwind.config.js - extend theme
theme: {
  extend: {
    colors: {
      paper: { accent: '#0EA5E9', badge: '#0284C7' },
      real: { warning: '#EF4444', banner: '#DC2626' },
      profit: '#10B981',
      loss: '#EF4444',
    },
    animation: {
      'number-change': 'numberChange 0.3s ease-out',
      'price-flash': 'priceFlash 0.5s ease-out',
    }
  }
}
```

### Step 4: Create Mode Badge Component
```typescript
// ModeBadge.tsx
export function ModeBadge({ mode }: { mode: 'paper' | 'real' }) {
  return mode === 'paper' ? (
    <Badge className="bg-paper-accent text-white">SANDBOX</Badge>
  ) : (
    <Badge className="bg-real-warning text-white animate-pulse">REAL MONEY</Badge>
  )
}
```

### Step 5: Create Animated Number Component
```typescript
// AnimatedNumber.tsx
import { motion, AnimatePresence } from 'framer-motion'

export function AnimatedNumber({ value, prefix = '$' }) {
  return (
    <AnimatePresence mode="wait">
      <motion.span
        key={value}
        initial={{ opacity: 0, y: -10 }}
        animate={{ opacity: 1, y: 0 }}
        exit={{ opacity: 0, y: 10 }}
        transition={{ duration: 0.3 }}
      >
        {prefix}{value.toLocaleString()}
      </motion.span>
    </AnimatePresence>
  )
}
```

### Step 6: Create Glassmorphism Card Variant
```typescript
// GlassCard.tsx
export function GlassCard({ children, className }) {
  return (
    <div className={cn(
      "backdrop-blur-md bg-slate-900/70 border border-slate-700/50",
      "rounded-xl shadow-xl",
      className
    )}>
      {children}
    </div>
  )
}
```

## Todo List

- [ ] Install framer-motion dependency
- [ ] Create src/styles/tokens/ directory structure
- [ ] Define color palette in colors.ts
- [ ] Define typography scale in typography.ts
- [ ] Create Framer Motion animation variants
- [ ] Update tailwind.config.js with theme extensions
- [ ] Create ModeBadge component
- [ ] Create AnimatedNumber component
- [ ] Create GlassCard variant component
- [ ] Add CSS variables to index.css
- [ ] Test contrast ratios meet WCAG AA
- [ ] Add reduced motion media query support

## Success Criteria

1. All color tokens defined and exported
2. Mode badge visually distinct (blue vs red)
3. Animations smooth at 60fps
4. Contrast ratios >= 4.5:1
5. Reduced motion respected

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Glassmorphism readability | Medium | Medium | Test contrast, adjust blur |
| Animation performance | Low | Medium | Use will-change, GPU layers |
| Color accessibility | Low | High | Use contrast checker tools |

## Security Considerations

- No security concerns for design tokens
- Ensure no sensitive data in animations/transitions

## Next Steps

After Phase 1 completion, proceed to [Phase 2: Mode Infrastructure](./phase-02-mode-infrastructure.md)
