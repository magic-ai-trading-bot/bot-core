# Design System Quick Reference
## Bot Core Trading Dashboard

**Version:** 1.0 | **Last Updated:** 2025-11-19

---

## Color Palette

### Trading Colors
```css
/* Profit/Gain */
--profit: hsl(142, 76%, 36%)          /* #22c55e */
text-profit, bg-profit, border-profit

/* Loss/Negative */
--loss: hsl(0, 84%, 60%)              /* #ef4444 */
text-loss, bg-loss, border-loss

/* Warning/Alert */
--warning: hsl(47, 96%, 53%)          /* #f59e0b */
text-warning, bg-warning, border-warning

/* Information */
--info: hsl(217, 91%, 60%)            /* #3b82f6 */
text-info, bg-info, border-info
```

### Semantic Colors
```css
/* Background */
--background: hsl(222, 15%, 8%)       /* Deep dark */
--foreground: hsl(210, 40%, 98%)      /* Near white */

/* Primary (Green - Trading focus) */
--primary: hsl(142, 76%, 36%)
--primary-foreground: hsl(222, 15%, 8%)

/* Secondary (Blue-gray) */
--secondary: hsl(217, 33%, 17%)
--secondary-foreground: hsl(210, 40%, 98%)

/* Muted (Low emphasis) */
--muted: hsl(217, 33%, 17%)
--muted-foreground: hsl(215, 20%, 65%)

/* Accent (Yellow) */
--accent: hsl(47, 96%, 53%)
--accent-foreground: hsl(222, 15%, 8%)

/* Destructive (Red) */
--destructive: hsl(0, 84%, 60%)
--destructive-foreground: hsl(210, 40%, 98%)
```

### Chart Colors
```css
--chart-1: hsl(142, 76%, 36%)  /* Green */
--chart-2: hsl(0, 84%, 60%)    /* Red */
--chart-3: hsl(47, 96%, 53%)   /* Yellow */
--chart-4: hsl(217, 91%, 60%)  /* Blue */
--chart-5: hsl(267, 57%, 78%)  /* Purple */
```

---

## Typography

### Font Families
```css
font-sans: system-ui, -apple-system, sans-serif (default)
font-mono: monospace (for financial data)
```

**Recommended:** Inter or Manrope for custom font

### Type Scale
```tsx
/* Headings */
<h1 className="text-2xl lg:text-3xl font-bold">Main Title</h1>
<h2 className="text-xl lg:text-2xl font-bold">Section Title</h2>
<h3 className="text-lg font-semibold">Subsection</h3>

/* Body Text */
<p className="text-sm lg:text-base">Body copy</p>
<p className="text-xs lg:text-sm">Small text</p>

/* Financial Data */
<span className="text-lg font-mono">$12,345.67</span>

/* Labels */
<label className="text-xs text-muted-foreground">Label</label>
```

### Font Weights
- `font-normal` (400) - Body text
- `font-medium` (500) - Emphasis
- `font-semibold` (600) - Headings
- `font-bold` (700) - Strong emphasis

---

## Spacing System

### Base Unit: 4px (0.25rem)

```tsx
/* Gaps (between elements) */
gap-2  /* 8px  - Tight spacing */
gap-4  /* 16px - Standard spacing */
gap-6  /* 24px - Loose spacing */

/* Padding (internal) */
p-2    /* 8px  - Tight */
p-4    /* 16px - Standard */
p-6    /* 24px - Loose */

/* Responsive padding */
<div className="p-4 lg:p-6">Container</div>

/* Margin (external) */
m-2    /* 8px */
m-4    /* 16px */
m-6    /* 24px */

/* Vertical spacing between sections */
space-y-4  /* 16px */
space-y-6  /* 24px */
```

### Layout Guidelines
- **Mobile:** `p-4` (16px padding)
- **Desktop:** `lg:p-6` (24px padding)
- **Container max-width:** `2xl:1400px`
- **Grid gaps:** `gap-4 lg:gap-6`

---

## Border Radius

```css
--radius: 0.75rem /* 12px - Default */

/* Utilities */
rounded-sm  /* calc(var(--radius) - 4px) = 8px */
rounded-md  /* calc(var(--radius) - 2px) = 10px */
rounded-lg  /* var(--radius) = 12px */
rounded-xl  /* 16px */
rounded-2xl /* 24px */
rounded-full /* 9999px - Pills/circles */
```

**Component Standards:**
- Cards: `rounded-lg`
- Buttons: `rounded-md`
- Inputs: `rounded-md`
- Badges: `rounded-full` or `rounded-md`

---

## Component Patterns

### Button Variants
```tsx
import { Button } from "@/components/ui/button";

/* Primary Action (Profit green) */
<Button className="bg-profit hover:bg-profit/90">
  Execute Trade
</Button>

/* Secondary Action */
<Button variant="outline">Cancel</Button>

/* Ghost/Minimal */
<Button variant="ghost">More Options</Button>

/* Destructive */
<Button variant="destructive">Delete</Button>

/* Sizes */
<Button size="sm">Small</Button>
<Button size="default">Default</Button>
<Button size="lg">Large</Button>
```

### Badge Patterns
```tsx
import { Badge } from "@/components/ui/badge";

/* Status Indicators */
<Badge className="bg-profit text-profit-foreground">Active</Badge>
<Badge className="bg-loss text-loss-foreground">Inactive</Badge>
<Badge className="bg-warning text-warning-foreground">Pending</Badge>

/* Outline Style */
<Badge variant="outline" className="text-info border-info/20">
  Live Analysis
</Badge>

/* With Icon & Pulse */
<Badge className="bg-profit/10 text-profit border-profit/20">
  <div className="w-2 h-2 bg-profit rounded-full mr-2 animate-pulse" />
  Bot Active
</Badge>
```

### Card Patterns
```tsx
import { Card, CardHeader, CardTitle, CardContent } from "@/components/ui/card";

/* Standard Card */
<Card>
  <CardHeader>
    <CardTitle className="text-lg">Account Balance</CardTitle>
  </CardHeader>
  <CardContent>
    {/* Content here */}
  </CardContent>
</Card>

/* With Background */
<Card className="bg-secondary/50 border-border/50">
  {/* Content */}
</Card>
```

### Form Inputs
```tsx
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";

<div className="space-y-2">
  <Label htmlFor="email">Email</Label>
  <Input
    id="email"
    type="email"
    placeholder="admin@tradingbot.com"
    className="bg-background/50"
  />
</div>
```

---

## Responsive Breakpoints

```tsx
/* Mobile First Approach */

/* Base (0px+) - Mobile */
<div className="p-4">...</div>

/* sm (640px+) - Large mobile */
<div className="p-4 sm:p-6">...</div>

/* md (768px+) - Tablet */
<div className="grid-cols-1 md:grid-cols-2">...</div>

/* lg (1024px+) - Desktop */
<div className="p-4 lg:p-6 lg:grid-cols-3">...</div>

/* xl (1280px+) - Large desktop */
<div className="xl:grid-cols-4">...</div>

/* 2xl (1400px+) - Wide screen */
<div className="2xl:max-w-7xl">...</div>
```

### Common Responsive Patterns
```tsx
/* Responsive Grid */
<div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4 lg:gap-6">

/* Responsive Flex */
<div className="flex flex-col lg:flex-row items-start lg:items-center gap-4">

/* Responsive Typography */
<h1 className="text-2xl lg:text-3xl font-bold">

/* Responsive Spacing */
<div className="p-4 lg:p-6 space-y-4 lg:space-y-6">
```

---

## Icon System

**Library:** Lucide React (v0.554.0)

### Trading Icons
```tsx
import {
  TrendingUp,      // Profit/Bullish
  TrendingDown,    // Loss/Bearish
  Activity,        // Neutral/Activity
  Zap,             // AI/Quick actions
  BarChart3,       // Charts
  Target,          // Goals/Targets
  Shield,          // Security/Risk
  AlertCircle,     // Warnings
  Info,            // Information
  RefreshCw,       // Reload/Refresh
} from "lucide-react";

/* Usage */
<TrendingUp className="h-4 w-4 text-profit" />
```

### Icon Sizes
```tsx
className="h-3 w-3"   /* 12px - Small */
className="h-4 w-4"   /* 16px - Standard */
className="h-5 w-5"   /* 20px - Medium */
className="h-6 w-6"   /* 24px - Large */
```

---

## Animation Guidelines

### Built-in Animations
```tsx
/* Pulse (for status indicators) */
<div className="animate-pulse">Live</div>

/* Spin (for loading) */
<RefreshCw className="animate-spin" />

/* Bounce (for scroll indicators) */
<div className="animate-bounce">↓</div>

/* Transitions */
className="transition-all duration-200 hover:shadow-lg"
className="transition-colors duration-150 hover:bg-primary/90"
```

### Custom Animations (from config)
```tsx
/* Accordion */
<Accordion>
  {/* Uses accordion-down/up animations */}
</Accordion>
```

### Reduced Motion Support
Always respect user preferences:
```css
@media (prefers-reduced-motion: reduce) {
  /* Disable animations */
}
```

---

## Accessibility Standards

### Color Contrast Requirements
- **Normal text:** ≥ 4.5:1 contrast ratio (WCAG AA)
- **Large text:** ≥ 3:1 contrast ratio
- **All current colors:** Pass ✅

### Touch Targets
- **Minimum size:** 44x44px (mobile)
- **Recommended:** 48x48px

### Focus States
```tsx
/* Always add focus styles */
className="focus:outline-none focus:ring-2 focus:ring-profit focus:ring-offset-2"

/* High contrast mode */
className="focus-visible:ring-2 focus-visible:ring-profit"
```

### Screen Reader Support
```tsx
/* Hidden text for screen readers */
<span className="sr-only">Price increased</span>

/* Hide decorative elements */
<div aria-hidden="true">Icon</div>

/* ARIA labels */
<button aria-label="Close position">×</button>
```

---

## Common Component Combinations

### Stat Card
```tsx
<Card>
  <CardHeader>
    <CardTitle className="text-base lg:text-lg">Total Balance</CardTitle>
  </CardHeader>
  <CardContent>
    <div className="text-2xl lg:text-3xl font-bold font-mono">
      $12,450.32
    </div>
    <div className="text-sm text-profit flex items-center gap-1 mt-1">
      <TrendingUp className="h-3 w-3" />
      +2.5%
    </div>
  </CardContent>
</Card>
```

### Signal Card (Clickable)
```tsx
<div className="p-4 rounded-lg border bg-secondary/50 hover:bg-secondary/70 cursor-pointer transition-all">
  <div className="flex justify-between items-start mb-3">
    <Badge className="bg-profit text-profit-foreground">LONG</Badge>
    <div className="font-bold text-lg text-profit">85%</div>
  </div>
  <p className="text-sm">Strong bullish momentum detected</p>
</div>
```

### Position Display
```tsx
<div className="p-3 rounded-lg bg-secondary/50 border">
  <div className="flex justify-between items-center mb-2">
    <div className="flex items-center gap-2">
      <Badge className="bg-profit">LONG</Badge>
      <span className="font-semibold">BTC/USDT</span>
    </div>
    <div className="text-profit font-bold">+$767.39</div>
  </div>
  <div className="text-xs text-muted-foreground">
    Entry: $42,800.50 • Size: 0.1 BTC
  </div>
</div>
```

---

## Best Practices

### ✅ Do's
- Use semantic color names (profit/loss) not red/green
- Always add responsive variants (lg:, md:)
- Include loading and error states
- Add proper ARIA labels
- Use font-mono for financial data
- Implement keyboard navigation
- Add alt text to images
- Use icons with color for better accessibility

### ❌ Don'ts
- Don't hardcode colors (use CSS variables)
- Don't skip mobile testing
- Don't rely on color alone for information
- Don't forget focus indicators
- Don't use small touch targets (< 44px)
- Don't ignore reduced motion preferences
- Don't mix different icon libraries

---

## Development Workflow

### Creating a New Component

1. **Check existing components first**
   ```bash
   ls src/components/ui/
   ```

2. **Use Shadcn CLI if available**
   ```bash
   npx shadcn-ui@latest add [component-name]
   ```

3. **Follow naming conventions**
   - PascalCase for components: `TradingSignal.tsx`
   - Lowercase for utilities: `formatters.ts`

4. **Include TypeScript types**
   ```tsx
   interface TradingSignalProps {
     symbol: string;
     confidence: number;
     signal: "LONG" | "SHORT" | "NEUTRAL";
   }
   ```

5. **Add responsive styles**
   ```tsx
   <div className="p-4 lg:p-6">...</div>
   ```

6. **Test accessibility**
   - Run axe DevTools
   - Test keyboard navigation
   - Check screen reader

---

## Quick Tips

### Financial Data Formatting
```tsx
// Use font-mono for numbers
<span className="font-mono">${value.toLocaleString()}</span>

// Always show + for positive changes
{pnl >= 0 ? '+' : ''}${pnl.toFixed(2)}

// Use appropriate colors
<span className={pnl >= 0 ? 'text-profit' : 'text-loss'}>
  {pnl >= 0 ? '+' : ''}{pnl.toFixed(2)}%
</span>
```

### Loading States
```tsx
// Skeleton for cards
<div className="h-64 bg-muted/20 rounded-lg animate-pulse" />

// Spinner for buttons
<RefreshCw className="h-4 w-4 animate-spin mr-2" />
```

### Toast Notifications
```tsx
import { toast } from "sonner";

// Success
toast.success("Trade executed", {
  description: "Your order has been placed",
});

// Error
toast.error("Trade failed", {
  description: error.message,
});

// Loading
toast.loading("Processing...", { id: "trade-loading" });
```

---

## Resources

- **Shadcn/UI Docs:** https://ui.shadcn.com/
- **TailwindCSS Docs:** https://tailwindcss.com/docs
- **Lucide Icons:** https://lucide.dev/
- **Radix UI:** https://www.radix-ui.com/
- **WCAG Guidelines:** https://www.w3.org/WAI/WCAG21/quickref/

---

**Maintained by:** UI/UX Design Team
**Last Updated:** 2025-11-19
**Version:** 1.0
