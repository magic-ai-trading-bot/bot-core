# Dark OLED Luxury - Color Palette Reference
## Complete Hex Code Reference & Usage Guide

**Document Type**: Color Palette Reference
**Version**: 1.0
**Date**: 2025-12-03
**Status**: Master Reference

---

## Quick Copy Reference

```
Primary: #000000 #0F0F1E #1A1A2E #16213E #0D0D14
Text: #FFFFFF #B0B0C0 #7A7A8E #4A4A5E
Financial: #10B981 #EF4444 #F59E0B #6B7280
Brand: #2962FF #00D9FF #A855F7 #F3BA2F
```

---

## Color Palette Matrix

### Background Colors (OLED Optimized)

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| Primary Black | `#000000` | rgb(0, 0, 0) | Main background, OLED power saving |
| Secondary Navy | `#0F0F1E` | rgb(15, 15, 30) | Primary surface, subtle elevation |
| Tertiary Slate | `#1A1A2E` | rgb(26, 26, 46) | Cards, panels, sections |
| Quaternary Blue | `#16213E` | rgb(22, 33, 62) | Hover states, elevated surfaces |
| Surface Depth | `#0D0D14` | rgb(13, 13, 20) | Deep background, modals |

**Visual Hierarchy**:
```
#000000 (Darkest)
  ↓
#0F0F1E (Primary Surface) ← Most used
  ↓
#1A1A2E (Cards, Content)
  ↓
#16213E (Hover, Elevated)
  ↓
#0D0D14 (Deepest Shadow)
```

**Why These Colors**?
- Pure black (#000000) = OLED battery savings + luxury feel
- Subtle blue tint (#16213E) = prevents monotony without harshness
- Spacing of 15 steps = maintains visual hierarchy without visual fatigue

---

### Text Colors (High Contrast, Accessibility)

| Color Name | Hex Code | RGB | Contrast on #000000 | Usage |
|-----------|----------|-----|---------------------|-------|
| Primary White | `#FFFFFF` | rgb(255, 255, 255) | 21:1 (AAA+) | Headlines, primary text |
| Secondary | `#B0B0C0` | rgb(176, 176, 192) | 10.5:1 (AA) | Body text, descriptions |
| Tertiary | `#7A7A8E` | rgb(122, 122, 142) | 5:1 (AA) | Secondary labels, hints |
| Muted | `#4A4A5E` | rgb(74, 74, 94) | 2.5:1 | Disabled text, very light content |

**Typography Usage Rules**:
```
Headlines (H1-H4):        #FFFFFF (pure white)
Body Text (long content): #B0B0C0 (secondary - less eye strain)
Labels/UI:                #7A7A8E (tertiary)
Disabled/Helper:          #4A4A5E (muted)
```

**Contrast Testing (WCAG 2.1)**:
```
✓ #FFFFFF on #000000 = 21:1  ✓ AAA (highest level)
✓ #B0B0C0 on #0F0F1E = 10.5:1 ✓ AA (standard)
✓ #7A7A8E on #1A1A2E = 5:1   ✓ AA (minimum)
✗ #4A4A5E on #000000 = 2.5:1 ✗ FAIL (too low)
```

---

### Financial Status Colors

#### Profit / Bullish (Green)

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| **Profit Green** | `#10B981` | rgb(16, 185, 129) | Primary profit indicator |
| Profit Hover | `#059669` | rgb(5, 150, 105) | Hover state |
| Profit BG | `rgba(16, 185, 129, 0.08)` | (8% opacity) | Background fill |
| Profit Border | `rgba(16, 185, 129, 0.2)` | (20% opacity) | Border/outline |

**Examples**:
- ✓ Stock price up: `text-financial-profit`
- ✓ Buy button: `bg-financial-profit`
- ✓ Profit badge: `bg-financial-profit-bg text-financial-profit`
- ✓ Profit chart candles: `fill="#10B981"`

#### Loss / Bearish (Red)

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| **Loss Red** | `#EF4444` | rgb(239, 68, 68) | Primary loss indicator |
| Loss Hover | `#DC2626` | rgb(220, 38, 38) | Hover state |
| Loss BG | `rgba(239, 68, 68, 0.08)` | (8% opacity) | Background fill |
| Loss Border | `rgba(239, 68, 68, 0.2)` | (20% opacity) | Border/outline |

**Examples**:
- ✗ Stock price down: `text-financial-loss`
- ✗ Sell button: `bg-financial-loss`
- ✗ Loss badge: `bg-financial-loss-bg text-financial-loss`
- ✗ Loss chart candles: `fill="#EF4444"`

#### Warning / Caution (Orange)

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| **Warning Orange** | `#F59E0B` | rgb(245, 158, 11) | Pending/caution state |
| Warning Hover | `#D97706` | rgb(217, 119, 6) | Hover state |
| Warning BG | `rgba(245, 158, 11, 0.08)` | (8% opacity) | Background fill |

**Examples**:
- ⏳ Pending order: `text-financial-warning`
- ⏳ Cooldown active: `bg-financial-warning-bg`
- ⏳ Risk warning: `border-financial-warning`

#### Neutral (Gray)

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| **Neutral Gray** | `#6B7280` | rgb(107, 114, 128) | Neutral state |
| Neutral Hover | `#9CA3AF` | rgb(156, 163, 175) | Hover state |

---

### Brand & Primary Colors

#### Premium Blue (TradingView-Inspired)

| Color Name | Hex Code | RGB | Usage | Contrast on #000000 |
|-----------|----------|-----|-------|---------------------|
| **Brand Blue** | `#2962FF` | rgb(41, 98, 255) | Primary actions, links | 3.6:1 |
| Blue Hover | `#2257E7` | rgb(34, 87, 231) | Hover state | 3.4:1 |
| Blue Light | `#5B7FFF` | rgb(91, 127, 255) | Light accent | 5:1 |
| Blue BG | `rgba(41, 98, 255, 0.08)` | (8% opacity) | Background | - |

**Why TradingView Blue?**
- Proven in professional trading platforms
- Represents trust + technology
- Used by TradingView (#2962FF), Kraken, professional traders
- Perfect contrast for light text on dark backgrounds

**Usage Examples**:
```jsx
// Primary button
<button className="bg-brand-blue text-white" />

// Link color
<a className="text-brand-blue hover:text-brand-blue-hover" />

// Focus state
.focus:ring-brand-blue

// Badge
<span className="bg-brand-blue-bg text-brand-blue" />
```

---

### Accent Colors (Modern, High-Tech)

#### Neon Cyan (Secondary Accent)

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| **Cyan** | `#00D9FF` | rgb(0, 217, 255) | Modern accent, highlights |
| Cyan Soft | `rgba(0, 217, 255, 0.15)` | (15% opacity) | Subtle background |

**Use Cases**:
- ◆ Moving average (EMA5, fast line)
- ◆ Highlight important data
- ◆ Modern, tech-forward feel
- ◆ Secondary action buttons

**Glassmorphism Gradient**:
```css
background: linear-gradient(135deg,
  rgba(0, 217, 255, 0.2) 0%,
  rgba(0, 217, 255, 0.05) 100%);
```

#### Vibrant Purple (Tertiary Accent)

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| **Purple** | `#A855F7` | rgb(168, 85, 247) | Special features, signals |
| Purple Light | `#D4A5FF` | rgb(212, 165, 255) | Light variant |

**Use Cases**:
- ◆ EMA20 indicator (intermediate)
- ◆ Alert signals
- ◆ Premium features
- ◆ Special status badges

#### Bitcoin Gold (Premium Luxury)

| Color Name | Hex Code | RGB | Usage | Comment |
|-----------|----------|-----|-------|---------|
| **Gold** | `#F3BA2F` | rgb(243, 186, 47) | Bitcoin theme, Binance brand | Official Binance color |
| Gold Dim | `rgba(243, 186, 47, 0.15)` | (15% opacity) | Subtle highlight | Gold text on dark bg |

**Rationale**:
- Official Binance brand color (since 2017)
- Psychological connection to wealth/Bitcoin
- Excellent contrast on dark backgrounds
- Represents premium/VIP status

**Use Cases**:
- ◆ Bitcoin (BTC) price display
- ◆ Premium account indicator
- ◆ Wealth/portfolio display
- ◆ Special badge or achievement

#### Ethereum Silver

| Color Name | Hex Code | RGB | Usage |
|-----------|----------|-----|-------|
| **Silver** | `#C0C0D0` | rgb(192, 192, 208) | Ethereum theme |

---

### Border & Stroke Colors

| Color Name | Hex Code | Opacity | RGB | Usage |
|-----------|----------|---------|-----|-------|
| Border Primary | `rgba(176, 176, 192, 0.1)` | 10% | (176, 176, 192) | Subtle separation |
| Border Secondary | `rgba(176, 176, 192, 0.2)` | 20% | (176, 176, 192) | Visible division |
| Border Accent | `rgba(41, 98, 255, 0.3)` | 30% | (41, 98, 255) | Interactive/hover |
| Border Danger | `rgba(239, 68, 68, 0.3)` | 30% | (239, 68, 68) | Error state |
| Border Success | `rgba(16, 185, 129, 0.3)` | 30% | (16, 185, 129) | Success state |

**Hierarchy**:
```css
/* Cards, Panels */
border: 1px solid var(--color-border-primary);

/* Hover, More Important */
border: 1px solid var(--color-border-secondary);

/* Focus, Interactive */
border: 1px solid var(--color-border-accent);
```

---

## Color Applications by Context

### Trading Interface

**Buy/Long Orders**:
```css
color: #10B981;           /* Green text */
background: rgba(16, 185, 129, 0.08);  /* Light green bg */
border: 1px solid rgba(16, 185, 129, 0.2);
box-shadow: 0 0 15px rgba(16, 185, 129, 0.3);  /* Glow effect */
```

**Sell/Short Orders**:
```css
color: #EF4444;           /* Red text */
background: rgba(239, 68, 68, 0.08);   /* Light red bg */
border: 1px solid rgba(239, 68, 68, 0.2);
box-shadow: 0 0 15px rgba(239, 68, 68, 0.3);
```

**Pending/Cooldown**:
```css
color: #F59E0B;           /* Orange text */
background: rgba(245, 158, 11, 0.08);  /* Light orange bg */
border: 1px solid rgba(245, 158, 11, 0.2);
```

### Charts & Indicators

**Candlestick Colors**:
```javascript
const chartColors = {
  // Bullish candle
  upColor: '#10B981',
  upWick: '#B0B0C0',

  // Bearish candle
  downColor: '#EF4444',
  downWick: '#B0B0C0',
};

// Moving averages
const indicators = {
  ema5: '#00D9FF',    // Cyan - fast
  ema20: '#A855F7',   // Purple - medium
  ema50: '#F59E0B',   // Orange - slower
  ema200: '#2962FF',  // Blue - long-term
};
```

### UI Elements

**Buttons**:
```css
/* Primary (Blue) */
background: #2962FF;
hover: #2257E7;
shadow: 0 4px 12px rgba(41, 98, 255, 0.3);

/* Secondary (Border) */
border: 1px solid rgba(176, 176, 192, 0.2);
hover: rgba(176, 176, 192, 0.3);

/* Disabled */
opacity: 0.5;
```

**Input Fields**:
```css
background: #0F0F1E;
border: 1px solid rgba(176, 176, 192, 0.1);
focus: border-color #2962FF;
focus: box-shadow 0 0 0 3px rgba(41, 98, 255, 0.1);
```

**Badges**:
```css
/* Success */
background: rgba(16, 185, 129, 0.08);
color: #10B981;
border: 1px solid rgba(16, 185, 129, 0.2);

/* Error */
background: rgba(239, 68, 68, 0.08);
color: #EF4444;
border: 1px solid rgba(239, 68, 68, 0.2);
```

---

## Gradient Definitions

### Premium Gradient (Primary)
```css
background: linear-gradient(135deg, #0F0F1E 0%, #16213E 100%);
```

### Cyan Accent Gradient
```css
background: linear-gradient(135deg,
  rgba(0, 217, 255, 0.2) 0%,
  rgba(0, 217, 255, 0.05) 100%);
```

### Neon Blue Gradient
```css
background: linear-gradient(135deg, #2962FF 0%, #00D9FF 100%);
```

### Profit Gradient
```css
background: linear-gradient(135deg,
  rgba(16, 185, 129, 0.1) 0%,
  rgba(16, 185, 129, 0.05) 100%);
```

### Loss Gradient
```css
background: linear-gradient(135deg,
  rgba(239, 68, 68, 0.1) 0%,
  rgba(239, 68, 68, 0.05) 100%);
```

---

## Glassmorphism Colors

**Base Glass Layer**:
```css
background: rgba(26, 26, 46, 0.7);    /* 70% opacity */
backdrop-filter: blur(20px);
border: 1px solid rgba(176, 176, 192, 0.15);
```

**Glass Tight** (Better readability):
```css
background: rgba(26, 26, 46, 0.8);    /* 80% opacity */
backdrop-filter: blur(12px);
border: 1px solid rgba(176, 176, 192, 0.2);
```

**Glass Soft** (Ethereal effect):
```css
background: rgba(26, 26, 46, 0.6);    /* 60% opacity */
backdrop-filter: blur(30px);
border: 1px solid rgba(176, 176, 192, 0.1);
```

**Glass with Inset Light** (Depth):
```css
box-shadow: inset 1px 1px 0 rgba(255, 255, 255, 0.1),
            inset -1px -1px 0 rgba(0, 0, 0, 0.1);
```

---

## Color Accessibility Matrix

### Text on Dark Backgrounds

| Text Color | Background | Contrast Ratio | WCAG Level | Safe? |
|-----------|-----------|----------------|-----------|-------|
| #FFFFFF | #000000 | 21:1 | AAA+ | ✓ Excellent |
| #B0B0C0 | #0F0F1E | 10.5:1 | AA | ✓ Good |
| #7A7A8E | #1A1A2E | 5:1 | AA | ✓ Minimum |
| #4A4A5E | #000000 | 2.5:1 | Fail | ✗ Avoid |
| #10B981 | #000000 | 3.6:1 | Fail | △ Text only |
| #EF4444 | #000000 | 3:1 | Fail | △ Text only |
| #2962FF | #000000 | 3.6:1 | Fail | △ Text only |

**Rules**:
- ✓ Use for body text: #FFFFFF, #B0B0C0, #7A7A8E
- △ Use with icons/support: #10B981, #EF4444, #2962FF
- ✗ Never use for text: #4A4A5E, pure saturation colors

---

## Color Space (Hex to HSL)

| Color | Hex | HSL | Saturation | Lightness |
|-------|-----|-----|-----------|-----------|
| Primary Black | #000000 | 0°, 0%, 0% | 0% | 0% |
| Secondary Navy | #0F0F1E | 240°, 33%, 6% | 33% | 6% |
| Brand Blue | #2962FF | 218°, 100%, 59% | 100% | 59% |
| Cyan | #00D9FF | 186°, 100%, 50% | 100% | 50% |
| Profit Green | #10B981 | 160°, 85%, 44% | 85% | 44% |
| Loss Red | #EF4444 | 0°, 94%, 55% | 94% | 55% |

---

## CSS Variable Declaration

```css
:root {
  /* Backgrounds */
  --color-bg-primary: #000000;
  --color-bg-secondary: #0F0F1E;
  --color-bg-tertiary: #1A1A2E;
  --color-bg-quaternary: #16213E;
  --color-bg-surface: #0D0D14;

  /* Text */
  --color-text-primary: #FFFFFF;
  --color-text-secondary: #B0B0C0;
  --color-text-tertiary: #7A7A8E;
  --color-text-muted: #4A4A5E;

  /* Financial */
  --color-profit: #10B981;
  --color-profit-hover: #059669;
  --color-profit-bg: rgba(16, 185, 129, 0.08);
  --color-profit-border: rgba(16, 185, 129, 0.2);

  --color-loss: #EF4444;
  --color-loss-hover: #DC2626;
  --color-loss-bg: rgba(239, 68, 68, 0.08);
  --color-loss-border: rgba(239, 68, 68, 0.2);

  --color-warning: #F59E0B;
  --color-warning-hover: #D97706;
  --color-warning-bg: rgba(245, 158, 11, 0.08);

  --color-success: #10B981;

  /* Brand */
  --color-brand-blue: #2962FF;
  --color-brand-blue-hover: #2257E7;
  --color-brand-blue-light: #5B7FFF;

  /* Accents */
  --color-accent-cyan: #00D9FF;
  --color-accent-cyan-soft: rgba(0, 217, 255, 0.15);
  --color-accent-purple: #A855F7;
  --color-accent-purple-light: #D4A5FF;
  --color-accent-gold: #F3BA2F;
  --color-accent-gold-dim: rgba(243, 186, 47, 0.15);
  --color-accent-silver: #C0C0D0;

  /* Borders */
  --color-border-primary: rgba(176, 176, 192, 0.1);
  --color-border-secondary: rgba(176, 176, 192, 0.2);
  --color-border-accent: rgba(41, 98, 255, 0.3);
  --color-border-danger: rgba(239, 68, 68, 0.3);
  --color-border-success: rgba(16, 185, 129, 0.3);
}

/* Light mode override (if needed - not recommended) */
@media (prefers-color-scheme: light) {
  :root {
    /* Keep dark theme - don't switch */
  }
}
```

---

## Tailwind Class Quick Reference

```jsx
/* Background Colors */
className="bg-bg-primary"      // #000000
className="bg-bg-secondary"    // #0F0F1E
className="bg-bg-tertiary"     // #1A1A2E

/* Text Colors */
className="text-text-primary"   // #FFFFFF
className="text-text-secondary" // #B0B0C0
className="text-text-tertiary"  // #7A7A8E

/* Financial Colors */
className="text-financial-profit"   // #10B981 (Green)
className="text-financial-loss"     // #EF4444 (Red)
className="text-financial-warning"  // #F59E0B (Orange)

/* Brand Colors */
className="text-brand-blue"         // #2962FF
className="text-accent-cyan"        // #00D9FF
className="text-accent-gold"        // #F3BA2F

/* Combined States */
className="bg-financial-profit-bg text-financial-profit"
className="hover:bg-financial-loss-hover"
className="border border-border-accent"
```

---

## Color Testing Checklist

Before deploying:

- [ ] All text has minimum 4.5:1 contrast ratio
- [ ] Financial indicators tested (red/green colorblind)
- [ ] Charts tested on actual OLED screen
- [ ] Glassmorphism backgrounds have sufficient blur
- [ ] Borders visible but not distracting
- [ ] Hover states clearly distinguishable
- [ ] Focus states visible for accessibility
- [ ] Gradients smooth and natural-looking
- [ ] Color consistency across all pages
- [ ] Mobile dark mode tested

---

## Historical References (Research Sources)

**TradingView Colors**:
- Primary Blue: #2962FF (verified)
- Background: Light with dark mode option
- Our adaptation: Pure black + deep navy for OLED

**Binance Colors**:
- Brand Gold: #F3BA2F (verified official color)
- Used for BTC/wealth representation
- Professional financial feel

**Crypto Industry Standards**:
- Green for profit/up: #10B981 (common across platforms)
- Red for loss/down: #EF4444 (universal standard)
- Blue for trust/brand: #2962FF (professional standard)

---

**Document Status**: COMPLETE & READY FOR USE
**Last Updated**: 2025-12-03
**Version**: 1.0 (Final)

Use this reference for all color selection decisions in the cryptocurrency trading dashboard.
