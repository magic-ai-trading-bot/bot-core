# Dark OLED Luxury - Implementation Guide
## Cryptocurrency Trading Dashboard Components

**Document Type**: Implementation Guide
**Version**: 1.0
**Date**: 2025-12-03
**Status**: Ready for Development

---

## Quick Start

1. **Replace Tailwind Config**:
   ```bash
   cp nextjs-ui-dashboard/tailwind-dark-oled-config.js nextjs-ui-dashboard/tailwind.config.js
   ```

2. **Install Dependencies**:
   ```bash
   cd nextjs-ui-dashboard
   npm install
   ```

3. **Use Design Tokens**:
   ```jsx
   import { useDesignSystem } from '@/hooks/useDesignSystem'
   // OR use Tailwind classes directly
   <div className="bg-bg-primary text-text-primary">
   ```

---

## Component Examples

### 1. Trading Card (Glassmorphic)

**Usage**: Display price data, trade information, portfolio holdings

```jsx
// components/TradingCard.tsx
export function TradingCard({ symbol, price, change, trend }) {
  const isProfit = trend === 'up';

  return (
    <div className="
      glassmorphic
      rounded-md
      p-6
      border border-border-secondary
      hover:border-border-accent
      transition-all duration-300
      group
    ">
      {/* Header */}
      <div className="flex items-center justify-between mb-4">
        <h3 className="text-heading-4 font-bold text-text-primary">
          {symbol}
        </h3>
        <span className={`
          px-3 py-1 rounded-full text-ui-sm font-medium
          ${isProfit
            ? 'bg-financial-profit-bg text-financial-profit border border-financial-profit-border'
            : 'bg-financial-loss-bg text-financial-loss border border-financial-loss-border'
          }
        `}>
          {isProfit ? '📈 Up' : '📉 Down'}
        </span>
      </div>

      {/* Price */}
      <div className="mb-4">
        <p className="text-text-secondary text-ui-md mb-2">
          Current Price
        </p>
        <p className="font-mono text-heading-2 font-semibold text-text-primary">
          ${price.toFixed(2)}
        </p>
      </div>

      {/* Change */}
      <div className={`
        flex items-center gap-2
        font-mono text-body-lg font-medium
        ${isProfit ? 'text-financial-profit' : 'text-financial-loss'}
      `}>
        <span>{isProfit ? '+' : ''}{change}%</span>
        {isProfit ? '↗️' : '↘️'}
      </div>

      {/* Hover effect */}
      <div className="
        absolute inset-0 rounded-md
        bg-gradient-cyan opacity-0
        group-hover:opacity-5
        transition-opacity duration-300
        pointer-events-none
      " />
    </div>
  );
}
```

**Tailwind Classes Applied**:
- `glassmorphic` → Frosted glass effect
- `text-heading-4` → Typography scale
- `text-financial-profit` → Financial status color
- `transition-all duration-300` → Smooth hover

---

### 2. Buy/Sell Button Pair

**Usage**: Trading action buttons with clear visual distinction

```jsx
// components/TradingActionButtons.tsx
export function TradingActionButtons({ onBuy, onSell, disabled = false }) {
  return (
    <div className="flex gap-4">
      {/* Buy Button */}
      <button
        onClick={onBuy}
        disabled={disabled}
        className="
          flex-1
          btn-buy
          px-6 py-3
          bg-financial-profit hover:bg-financial-profit-hover
          text-white font-semibold text-ui-lg
          rounded-base
          shadow-[0_4px_12px_rgba(16,185,129,0.3)]
          hover:shadow-[0_6px_20px_rgba(16,185,129,0.4)]
          transition-all duration-200
          active:shadow-[0_2px_8px_rgba(16,185,129,0.3)]
          active:translate-y-0
          hover:-translate-y-0.5
          disabled:opacity-50
          disabled:cursor-not-allowed
          disabled:shadow-none
        "
      >
        Buy Long
      </button>

      {/* Sell Button */}
      <button
        onClick={onSell}
        disabled={disabled}
        className="
          flex-1
          btn-sell
          px-6 py-3
          bg-financial-loss hover:bg-financial-loss-hover
          text-white font-semibold text-ui-lg
          rounded-base
          shadow-[0_4px_12px_rgba(239,68,68,0.3)]
          hover:shadow-[0_6px_20px_rgba(239,68,68,0.4)]
          transition-all duration-200
          active:shadow-[0_2px_8px_rgba(239,68,68,0.3)]
          active:translate-y-0
          hover:-translate-y-0.5
          disabled:opacity-50
          disabled:cursor-not-allowed
          disabled:shadow-none
        "
      >
        Sell Short
      </button>
    </div>
  );
}
```

**Key Features**:
- Buy (green) vs Sell (red) visual distinction
- Hover lift animation (`hover:-translate-y-0.5`)
- Glow effects on hover
- Disabled state with reduced opacity
- Active state with shadow reduction

---

### 3. Chart Container (TradingView-like)

**Usage**: Wrap TradingView Lightweight Charts or custom charts

```jsx
// components/ChartContainer.tsx
import { TradingViewChart } from '@/components/TradingViewChart';

export function ChartContainer({ symbol, timeframe = '1h' }) {
  return (
    <div className="
      bg-bg-secondary
      border border-border-primary
      rounded-md
      p-6
      min-h-[400px]
      hover:border-border-secondary
      transition-colors duration-300
    ">
      {/* Chart Header */}
      <div className="flex items-center justify-between mb-4 pb-4 border-b border-border-primary">
        <div>
          <h3 className="text-heading-3 font-bold text-text-primary">
            {symbol}
          </h3>
          <p className="text-text-secondary text-ui-md mt-1">
            {timeframe} Timeframe
          </p>
        </div>
        <div className="flex gap-2">
          <button className="text-ui-md text-text-secondary hover:text-text-primary px-3 py-1 rounded transition-colors">
            1h
          </button>
          <button className="text-ui-md text-text-secondary hover:text-text-primary px-3 py-1 rounded transition-colors">
            4h
          </button>
          <button className="text-ui-md text-brand-blue px-3 py-1 rounded bg-brand-blue-bg">
            1d
          </button>
        </div>
      </div>

      {/* Chart */}
      <div className="h-[400px] rounded">
        <TradingViewChart symbol={symbol} interval={timeframe} />
      </div>

      {/* Chart Legend */}
      <div className="mt-4 pt-4 border-t border-border-primary grid grid-cols-3 gap-4">
        <div>
          <p className="text-text-secondary text-ui-sm">High</p>
          <p className="text-text-primary font-mono text-body-md">$45,230</p>
        </div>
        <div>
          <p className="text-text-secondary text-ui-sm">Current</p>
          <p className="text-text-primary font-mono text-body-md">$44,850</p>
        </div>
        <div>
          <p className="text-text-secondary text-ui-sm">Low</p>
          <p className="text-text-primary font-mono text-body-md">$43,920</p>
        </div>
      </div>
    </div>
  );
}
```

**Chart Color Configuration**:
```javascript
// For TradingView Lightweight Charts
const chartOptions = {
  layout: {
    background: { color: '#000000' },
    textColor: '#B0B0C0',
    font: {
      family: '-apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif',
      size: 12,
    },
  },
  grid: {
    vertLines: { color: 'rgba(176, 176, 192, 0.1)' },
    hLines: { color: 'rgba(176, 176, 192, 0.1)' },
  },
  watermark: {
    color: 'rgba(176, 176, 192, 0.1)',
    visible: true,
    fontSize: 14,
    horzAlign: 'right',
    vertAlign: 'bottom',
  },
};

// Candlestick colors
const candleColors = {
  upColor: '#10B981',      // Bullish green
  downColor: '#EF4444',    // Bearish red
  borderUpColor: '#10B981',
  borderDownColor: '#EF4444',
  wickUpColor: '#B0B0C0',
  wickDownColor: '#B0B0C0',
};

// Indicator colors
const indicatorColors = {
  ma5: '#00D9FF',          // Cyan - fast
  ma20: '#A855F7',         // Purple - intermediate
  ma50: '#F59E0B',         // Orange - slower
  ma200: '#60A5FA',        // Blue - long-term
  rsi: '#2962FF',          // Brand blue
};
```

---

### 4. Data Table (Watchlist)

**Usage**: Display multiple assets in table format

```jsx
// components/WatchlistTable.tsx
export function WatchlistTable({ assets }) {
  return (
    <div className="
      bg-bg-secondary
      border border-border-primary
      rounded-md
      overflow-hidden
    ">
      {/* Table Head */}
      <div className="
        bg-bg-tertiary
        border-b border-border-secondary
        grid grid-cols-5 gap-4
        px-6 py-4
      ">
        <div className="text-ui-md font-semibold text-text-secondary uppercase tracking-wider">
          Symbol
        </div>
        <div className="text-ui-md font-semibold text-text-secondary uppercase tracking-wider text-right">
          Price
        </div>
        <div className="text-ui-md font-semibold text-text-secondary uppercase tracking-wider text-right">
          24h Change
        </div>
        <div className="text-ui-md font-semibold text-text-secondary uppercase tracking-wider text-right">
          Volume
        </div>
        <div className="text-ui-md font-semibold text-text-secondary uppercase tracking-wider text-right">
          Action
        </div>
      </div>

      {/* Table Body */}
      <div className="divide-y divide-border-primary">
        {assets.map((asset) => (
          <div
            key={asset.symbol}
            className="
              grid grid-cols-5 gap-4
              px-6 py-4
              border-b border-border-primary
              hover:bg-[rgba(41,98,255,0.05)]
              transition-colors duration-200
              items-center
            "
          >
            {/* Symbol */}
            <div>
              <p className="text-body-md font-semibold text-text-primary">
                {asset.symbol}
              </p>
              <p className="text-ui-sm text-text-tertiary mt-1">
                {asset.name}
              </p>
            </div>

            {/* Price */}
            <div className="text-right">
              <p className="font-mono text-body-md font-medium text-text-primary">
                ${asset.price.toFixed(2)}
              </p>
            </div>

            {/* 24h Change */}
            <div className="text-right">
              <span className={`
                inline-block px-2 py-1 rounded text-ui-md font-semibold
                ${asset.change24h >= 0
                  ? 'text-financial-profit bg-financial-profit-bg'
                  : 'text-financial-loss bg-financial-loss-bg'
                }
              `}>
                {asset.change24h >= 0 ? '+' : ''}{asset.change24h}%
              </span>
            </div>

            {/* Volume */}
            <div className="text-right">
              <p className="font-mono text-body-md text-text-secondary">
                ${(asset.volume / 1000000).toFixed(2)}M
              </p>
            </div>

            {/* Action */}
            <div className="text-right">
              <button className="
                px-3 py-1
                text-ui-md font-medium
                text-brand-blue
                border border-brand-blue-bg
                rounded
                hover:bg-brand-blue-bg
                transition-all duration-200
              ">
                Trade
              </button>
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
```

---

### 5. Portfolio Status Card

**Usage**: Show portfolio overview with PnL

```jsx
// components/PortfolioStatusCard.tsx
export function PortfolioStatusCard({ balance, pnl, pnlPercent }) {
  const isProfit = pnl >= 0;

  return (
    <div className="
      relative
      bg-gradient-premium
      rounded-md
      p-8
      border border-border-accent
      overflow-hidden
      group
    ">
      {/* Background glow */}
      <div className="
        absolute -top-32 -right-32 w-64 h-64
        bg-blue-500 opacity-5
        rounded-full blur-3xl
        group-hover:opacity-10
        transition-opacity duration-500
      " />

      {/* Content */}
      <div className="relative z-10">
        <p className="text-text-secondary text-ui-md mb-2">
          Portfolio Balance
        </p>
        <h2 className="text-heading-1 font-bold text-text-primary mb-6">
          ${balance.toFixed(2)}
        </h2>

        {/* PnL Section */}
        <div className="flex items-end gap-4">
          <div>
            <p className="text-text-secondary text-ui-md mb-1">
              Today's P&L
            </p>
            <p className={`
              font-mono text-heading-3 font-bold
              ${isProfit ? 'text-financial-profit' : 'text-financial-loss'}
            `}>
              {isProfit ? '+' : ''}{pnl.toFixed(2)}
            </p>
          </div>

          <div className={`
            ml-auto px-4 py-2 rounded-lg
            text-ui-lg font-semibold
            ${isProfit
              ? 'bg-financial-profit-bg text-financial-profit'
              : 'bg-financial-loss-bg text-financial-loss'
            }
          `}>
            {isProfit ? '+' : ''}{pnlPercent}%
          </div>
        </div>

        {/* Details */}
        <div className="
          mt-6 pt-6
          border-t border-border-accent
          grid grid-cols-3 gap-4
        ">
          <div>
            <p className="text-text-tertiary text-ui-sm">Open Trades</p>
            <p className="text-body-lg font-semibold text-text-primary mt-1">
              12
            </p>
          </div>
          <div>
            <p className="text-text-tertiary text-ui-sm">Win Rate</p>
            <p className="text-body-lg font-semibold text-financial-profit mt-1">
              68%
            </p>
          </div>
          <div>
            <p className="text-text-tertiary text-ui-sm">Risk/Reward</p>
            <p className="text-body-lg font-semibold text-text-primary mt-1">
              1:2.5
            </p>
          </div>
        </div>
      </div>
    </div>
  );
}
```

---

### 6. Form Input (Dark Mode Optimized)

**Usage**: Login, settings, order placement forms

```jsx
// components/FormInput.tsx
export function FormInput({
  label,
  placeholder,
  value,
  onChange,
  error,
  success,
  type = 'text',
  disabled = false,
}) {
  return (
    <div className="mb-4">
      {label && (
        <label className="
          block text-ui-md font-medium text-text-secondary mb-2
        ">
          {label}
        </label>
      )}
      <input
        type={type}
        value={value}
        onChange={onChange}
        placeholder={placeholder}
        disabled={disabled}
        className={`
          w-full
          bg-bg-secondary hover:bg-bg-tertiary
          text-text-primary placeholder-text-muted
          border rounded-base
          px-4 py-2
          font-body-md
          transition-all duration-200
          focus-ring
          ${error
            ? 'border-financial-loss focus:border-financial-loss'
            : success
            ? 'border-financial-profit focus:border-financial-profit'
            : 'border-border-primary focus:border-brand-blue'
          }
          ${disabled ? 'opacity-50 cursor-not-allowed' : ''}
        `}
      />
      {error && (
        <p className="text-financial-loss text-ui-sm mt-2">
          {error}
        </p>
      )}
      {success && (
        <p className="text-financial-profit text-ui-sm mt-2">
          {success}
        </p>
      )}
    </div>
  );
}
```

---

### 7. Status Indicator (Live)

**Usage**: Show real-time connection status, trade status

```jsx
// components/StatusIndicator.tsx
export function StatusIndicator({ status = 'connected', label }) {
  const statusConfig = {
    connected: {
      color: 'bg-financial-profit',
      pulse: true,
      label: 'Connected',
    },
    disconnected: {
      color: 'bg-financial-loss',
      pulse: false,
      label: 'Disconnected',
    },
    pending: {
      color: 'bg-financial-warning',
      pulse: true,
      label: 'Pending',
    },
  };

  const config = statusConfig[status];

  return (
    <div className="flex items-center gap-2">
      <div className="relative flex h-2.5 w-2.5">
        <span className={`
          inline-flex rounded-full h-full w-full
          ${config.color}
          ${config.pulse ? 'animate-pulse' : ''}
        `} />
      </div>
      <span className="text-text-secondary text-ui-sm">
        {label || config.label}
      </span>
    </div>
  );
}
```

---

## Responsive Design

### Breakpoints
```tailwind
xs: 480px   (Smartphones)
sm: 640px   (Small tablets)
md: 768px   (Tablets)
lg: 1024px  (Desktops)
xl: 1280px  (Large desktops)
2xl: 1536px (Ultra-wide)
```

### Mobile-First Example
```jsx
<div className="
  grid
  grid-cols-1    /* Mobile: 1 column */
  sm:grid-cols-2 /* Small tablets: 2 columns */
  lg:grid-cols-4 /* Desktops: 4 columns */
  gap-4
">
  {/* Cards */}
</div>
```

---

## Dark Theme Gotchas & Solutions

### 1. Text Contrast Issues

**Problem**: Pure white text on pure black fatigues eyes
**Solution**: Use `text-text-secondary` (#B0B0C0) for body content, not white

```jsx
// ❌ Bad
<p className="text-white">Long content paragraph...</p>

// ✅ Good
<p className="text-text-secondary">Long content paragraph...</p>
```

### 2. Background Hierarchy Lost

**Problem**: Dark cards disappear on dark backgrounds
**Solution**: Use opacity borders + shadows to create separation

```jsx
// ❌ Bad
<div className="bg-bg-secondary">
  <div className="bg-bg-tertiary">Invisible difference</div>
</div>

// ✅ Good
<div className="bg-bg-tertiary border border-border-secondary rounded-md shadow-card">
  Content with clear separation
</div>
```

### 3. Financial Data Difficult to Scan

**Problem**: Green/red indicators hard to distinguish
**Solution**: Use color + icons + typography weight

```jsx
// ❌ Bad
<span className="text-green-500">+2.5%</span>

// ✅ Good
<span className="
  text-financial-profit
  font-mono
  font-semibold
  before:content-['📈_']
">
  +2.5%
</span>
```

### 4. Glow Effects Too Subtle

**Problem**: Blue glow unnoticeable on dark background
**Solution**: Increase glow intensity and use multiple shadow layers

```jsx
// ✅ Good glow
box-shadow: 0 0 20px rgba(41, 98, 255, 0.3), 0 0 40px rgba(41, 98, 255, 0.15)
```

---

## Animation & Performance

### Safe Animations
```jsx
// Use CSS transitions for state changes
className="transition-all duration-300 ease-out"

// Respect prefers-reduced-motion
className="motion-safe:animate-fade-in no-motion:opacity-100"
```

### GPU-Accelerated Transforms
```jsx
// Animate these properties (GPU accelerated)
className="hover:translate-y-[-2px] hover:scale-105"

// Avoid these (CPU heavy)
// - width, height, left, right, margin, padding
```

---

## Accessibility Checklist

- [ ] All text has sufficient contrast (4.5:1 minimum)
- [ ] Focus states visible on all interactive elements
- [ ] Forms have associated labels
- [ ] Images have alt text
- [ ] Color not sole indicator of status (use icons/text too)
- [ ] Animations respect `prefers-reduced-motion`
- [ ] Keyboard navigation works throughout
- [ ] Error messages clear and actionable

---

## Migration Guide (Existing Projects)

### Step 1: Update Tailwind Config
```bash
# Backup current config
cp tailwind.config.js tailwind.config.js.backup

# Use new config
cp nextjs-ui-dashboard/tailwind-dark-oled-config.js tailwind.config.js
```

### Step 2: Update Color Usage
```jsx
// Before
className="bg-gray-900 text-white"

// After
className="bg-bg-primary text-text-primary"
```

### Step 3: Replace Components
Use provided component examples as templates for existing components.

### Step 4: Test Contrast
```bash
npm install --save-dev axe-core
# Test with accessibility tools
```

---

## Performance Tips

1. **Use CSS classes, not inline styles**:
   ```jsx
   // ✅ Good - CSS is parsed once
   className="bg-brand-blue hover:bg-brand-blue-hover"

   // ❌ Bad - parsed every render
   style={{ backgroundColor: color }}
   ```

2. **Lazy load components**:
   ```jsx
   const Chart = dynamic(() => import('@/components/Chart'), {
     loading: () => <ChartSkeleton />,
   });
   ```

3. **Memoize expensive components**:
   ```jsx
   const TradingCard = memo(({ data }) => {
     return <div>{/* Chart rendering */}</div>;
   });
   ```

---

## File Structure

```
nextjs-ui-dashboard/
├── src/
│   ├── components/
│   │   ├── TradingCard.tsx
│   │   ├── ChartContainer.tsx
│   │   ├── TradingActionButtons.tsx
│   │   ├── WatchlistTable.tsx
│   │   ├── PortfolioStatusCard.tsx
│   │   ├── FormInput.tsx
│   │   ├── StatusIndicator.tsx
│   │   └── index.ts
│   ├── hooks/
│   │   ├── useDesignSystem.ts
│   │   └── useColorMode.ts
│   ├── styles/
│   │   ├── globals.css
│   │   └── variables.css
│   ├── pages/
│   │   ├── index.tsx (Dashboard)
│   │   ├── trading.tsx (Trading interface)
│   │   └── portfolio.tsx (Portfolio view)
│   └── types/
│       └── design.ts
├── tailwind.config.js (← Replace with -dark-oled-config.js)
├── tsconfig.json
└── next.config.js
```

---

## Design System Hook (Optional)

```typescript
// hooks/useDesignSystem.ts
export const useDesignSystem = () => {
  return {
    colors: {
      bg: {
        primary: '#000000',
        secondary: '#0F0F1E',
        tertiary: '#1A1A2E',
        quaternary: '#16213E',
        surface: '#0D0D14',
      },
      text: {
        primary: '#FFFFFF',
        secondary: '#B0B0C0',
        tertiary: '#7A7A8E',
        muted: '#4A4A5E',
      },
      financial: {
        profit: '#10B981',
        loss: '#EF4444',
        warning: '#F59E0B',
        success: '#10B981',
      },
      brand: {
        blue: '#2962FF',
      },
      accent: {
        cyan: '#00D9FF',
        purple: '#A855F7',
        gold: '#F3BA2F',
      },
    },
    spacing: {
      xs: '4px',
      sm: '8px',
      md: '16px',
      lg: '24px',
      xl: '32px',
      '2xl': '48px',
    },
    shadows: {
      card: '0 4px 6px rgba(0, 0, 0, 0.1)',
      glow: {
        blue: '0 0 20px rgba(41, 98, 255, 0.3)',
        green: '0 0 15px rgba(16, 185, 129, 0.3)',
        red: '0 0 15px rgba(239, 68, 68, 0.3)',
      },
    },
  };
};
```

---

## Troubleshooting

### Colors look wrong?
1. Verify `tailwind.config.js` is using dark-oled-config.js
2. Clear cache: `rm -rf .next node_modules && npm install`
3. Check browser DevTools → computed styles

### Glassmorphism not working?
1. Ensure `backdrop-filter` is enabled in tailwind config
2. Add `-webkit-backdrop-filter` for Safari support
3. Use `bg-opacity-70` + `backdrop-blur-md`

### OLED burns not showing real black?
1. Some OLED monitors need true #000000
2. If subtle blue is preferred: use #0F0F1E
3. Test on actual OLED device before shipping

---

## Resources

- [Tailwind CSS Docs](https://tailwindcss.com/docs)
- [Design System Reference](./design-system-cryptocurrency-trading-dashboard.md)
- [Component Examples in Figma](#) (if available)
- [Glassmorphism Best Practices](https://www.nngroup.com/articles/glassmorphism/)

---

**Status**: Ready for implementation in Next.js dashboard
**Last Updated**: 2025-12-03
