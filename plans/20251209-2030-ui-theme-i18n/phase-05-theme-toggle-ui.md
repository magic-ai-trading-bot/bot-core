# Phase 05: Theme Toggle UI Component

**Priority**: Medium | **Status**: Pending | **Est. Effort**: 2 hours

---

## Context Links

- [Main Plan](./plan.md)
- [Phase 01: Theme Infrastructure](./phase-01-theme-infrastructure.md)
- [Phase 02: Light Mode Design](./phase-02-light-mode-design.md)
- [Theming Research](./research/researcher-251209-theming-system.md)

---

## Overview

Create accessible, visually polished theme toggle component using Shadcn/UI primitives. Place in header/navbar alongside language selector.

---

## Key Insights

1. **3 modes supported** - Light, Dark, System
2. **Use Shadcn DropdownMenu** - Consistent with existing UI patterns
3. **Icons** - Sun (light), Moon (dark), Laptop/Monitor (system)
4. **Animations** - Subtle icon rotation on toggle
5. **Placement** - Header right side, next to LanguageSelector

---

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | Toggle button with current theme icon | Critical |
| R2 | Dropdown with 3 options (light/dark/system) | Critical |
| R3 | Accessible - keyboard navigable, aria labels | Critical |
| R4 | Smooth icon transition animation | Medium |
| R5 | Tooltip showing current mode | Medium |
| R6 | Mobile responsive (touch-friendly) | High |
| R7 | Integrate with ThemeContext | Critical |

---

## Architecture

**Component Structure**:
```typescript
<ThemeToggle>
  <DropdownMenu>
    <DropdownMenuTrigger>
      <Button variant="ghost" size="icon">
        <SunIcon /> or <MoonIcon /> or <MonitorIcon />
      </Button>
    </DropdownMenuTrigger>
    <DropdownMenuContent>
      <DropdownMenuItem onClick={() => setTheme('light')}>
        <SunIcon /> Light
      </DropdownMenuItem>
      <DropdownMenuItem onClick={() => setTheme('dark')}>
        <MoonIcon /> Dark
      </DropdownMenuItem>
      <DropdownMenuItem onClick={() => setTheme('system')}>
        <MonitorIcon /> System
      </DropdownMenuItem>
    </DropdownMenuContent>
  </DropdownMenu>
</ThemeToggle>
```

---

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ThemeToggle.tsx` | CREATE | Theme toggle component |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/MainLayout.tsx` | MODIFY | Add ThemeToggle to header |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/LandingHeader.tsx` | MODIFY | Add ThemeToggle for public pages |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Index.tsx` | MODIFY | Add ThemeToggle to landing nav |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/*/common.json` | MODIFY | Add theme label translations |

---

## Implementation Steps

### Step 1: Create ThemeToggle Component

```typescript
// src/components/ThemeToggle.tsx
import { Moon, Sun, Monitor } from 'lucide-react';
import { useTheme } from '@/contexts/ThemeContext';
import { useTranslation } from 'react-i18next';
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from '@/components/ui/dropdown-menu';
import { Button } from '@/components/ui/button';

export function ThemeToggle() {
  const { theme, setTheme, resolvedTheme } = useTheme();
  const { t } = useTranslation('common');

  const getIcon = () => {
    if (theme === 'system') {
      return <Monitor className="h-5 w-5" />;
    }
    return resolvedTheme === 'dark'
      ? <Moon className="h-5 w-5" />
      : <Sun className="h-5 w-5" />;
  };

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="ghost"
          size="icon"
          className="h-9 w-9"
          aria-label={t('label.theme')}
        >
          {getIcon()}
          <span className="sr-only">{t('label.theme')}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end">
        <DropdownMenuItem
          onClick={() => setTheme('light')}
          className={theme === 'light' ? 'bg-accent' : ''}
        >
          <Sun className="mr-2 h-4 w-4" />
          {t('theme.light')}
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => setTheme('dark')}
          className={theme === 'dark' ? 'bg-accent' : ''}
        >
          <Moon className="mr-2 h-4 w-4" />
          {t('theme.dark')}
        </DropdownMenuItem>
        <DropdownMenuItem
          onClick={() => setTheme('system')}
          className={theme === 'system' ? 'bg-accent' : ''}
        >
          <Monitor className="mr-2 h-4 w-4" />
          {t('theme.system')}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
```

### Step 2: Add Translation Keys

Update `common.json` for all 5 languages:

```json
{
  "label": {
    "theme": "Theme"
  },
  "theme": {
    "light": "Light",
    "dark": "Dark",
    "system": "System"
  }
}
```

**Japanese (ja/common.json)**:
```json
{
  "label": {
    "theme": "テーマ"
  },
  "theme": {
    "light": "ライト",
    "dark": "ダーク",
    "system": "システム"
  }
}
```

**Vietnamese (vi/common.json)**:
```json
{
  "label": {
    "theme": "Giao dien"
  },
  "theme": {
    "light": "Sang",
    "dark": "Toi",
    "system": "He thong"
  }
}
```

### Step 3: Add Animation to Icon

```typescript
// Add to ThemeToggle.tsx
import { motion, AnimatePresence } from 'framer-motion';

// Or simpler CSS approach in index.css:
.theme-icon {
  transition: transform 0.3s ease;
}

.theme-icon:hover {
  transform: rotate(15deg);
}
```

### Step 4: Integrate into MainLayout Header

```typescript
// In MainLayout header section
import { ThemeToggle } from '@/components/ThemeToggle';
import { LanguageSelector } from '@/components/LanguageSelector';

// Header right section
<div className="flex items-center gap-2">
  <ThemeToggle />
  <LanguageSelector />
  <UserMenu />
</div>
```

### Step 5: Integrate into Landing Page Header

```typescript
// In Index.tsx or landing header component
// Add ThemeToggle next to LanguageSelector in navigation
```

### Step 6: Add Keyboard Support

Ensure DropdownMenu handles:
- Tab to focus trigger
- Enter/Space to open
- Arrow keys to navigate
- Escape to close
- Enter to select

(Shadcn/UI handles this by default via Radix)

### Step 7: Mobile Touch Target

Ensure button meets WCAG 2.5.5 touch target (44x44px):
```typescript
<Button
  variant="ghost"
  size="icon"
  className="h-11 w-11 md:h-9 md:w-9" // 44px mobile, 36px desktop
/>
```

---

## Visual Design

**Light Mode Toggle Button**:
- Icon: Sun (yellow/amber)
- Background: transparent
- Hover: light gray bg

**Dark Mode Toggle Button**:
- Icon: Moon (blue/purple)
- Background: transparent
- Hover: dark gray bg

**System Mode Toggle Button**:
- Icon: Monitor/Laptop
- Background: transparent
- Shows current resolved theme icon

---

## Todo List

- [ ] Create `src/components/ThemeToggle.tsx`
- [ ] Add theme translations to all 5 `common.json` files
- [ ] Add icon rotation animation (CSS or Framer Motion)
- [ ] Integrate into MainLayout header
- [ ] Integrate into landing page header
- [ ] Ensure mobile touch target (44px)
- [ ] Test keyboard navigation
- [ ] Test with screen reader
- [ ] Add visual indicator for current selection

---

## Success Criteria

- [ ] Toggle visible in header on all pages
- [ ] Dropdown opens on click/Enter/Space
- [ ] Theme changes immediately on selection
- [ ] Current theme has visual indicator
- [ ] Icons animate smoothly
- [ ] Works on mobile (touch-friendly)
- [ ] Screen reader announces theme options
- [ ] Translations show for all 5 languages

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Animation jank | Low | Low | Use CSS transitions, not JS |
| Dropdown Z-index issues | Low | Medium | Use Shadcn/UI defaults |
| Mobile tap target too small | Medium | Medium | Explicit 44px min size |

---

## Security Considerations

- No security implications - purely UI component
- No user data involved

---

## Test Cases

| ID | Test Case | Expected Result |
|----|-----------|-----------------|
| TC-01 | Click theme toggle button | Dropdown opens |
| TC-02 | Select "Dark" in light mode | Theme switches to dark |
| TC-03 | Select "System" then change OS | Theme follows OS |
| TC-04 | Tab to toggle, press Enter | Dropdown opens |
| TC-05 | Use arrow keys in dropdown | Options highlighted |
| TC-06 | Press Escape | Dropdown closes |
| TC-07 | Tap on mobile | 44px touch target responsive |
| TC-08 | Switch to Japanese | Shows Japanese labels |

---

## Accessibility Checklist

- [ ] `aria-label` on trigger button
- [ ] `sr-only` text for screen readers
- [ ] Keyboard navigation works
- [ ] Focus visible on all elements
- [ ] Color not sole indicator (has text labels)
- [ ] Touch target >= 44px

---

## Next Steps

After this phase:
1. [Phase 06: Testing & Polish](./phase-06-testing-polish.md) - Final testing and refinements
