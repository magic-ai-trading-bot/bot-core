# Phase 01: Theme Infrastructure

**Priority**: Critical | **Status**: Pending | **Est. Effort**: 4 hours

---

## Context Links

- [Main Plan](./plan.md)
- [Theming Research](./research/researcher-251209-theming-system.md)
- [Scout Report](./scout/scout-01-frontend-analysis.md)

---

## Overview

Create ThemeContext provider with React Context, localStorage persistence, system preference detection, and FOUC prevention script in index.html.

---

## Key Insights

1. **Tailwind already configured** - `darkMode: ["class"]` in tailwind.config.ts (line 4)
2. **No ThemeProvider exists** - Must create from scratch
3. **Current default is dark** - `:root` in index.css defines dark luxury theme
4. **FOUC risk** - React hydration delay can cause flash; need blocking script in `<head>`
5. **System preference** - Use `matchMedia('(prefers-color-scheme: dark)')` + listener

---

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | ThemeContext with `theme`, `resolvedTheme`, `setTheme` | Critical |
| R2 | Support 3 modes: 'light', 'dark', 'system' | Critical |
| R3 | localStorage persistence key: 'theme' | Critical |
| R4 | FOUC prevention via blocking `<head>` script | Critical |
| R5 | System preference listener for real-time changes | High |
| R6 | Smooth transition class (0.3s) on theme change | Medium |
| R7 | Respect `prefers-reduced-motion` | Medium |

---

## Architecture

```typescript
// Type definitions
type Theme = 'light' | 'dark' | 'system';

interface ThemeContextType {
  theme: Theme;                    // User's preference
  resolvedTheme: 'light' | 'dark'; // Actual applied theme
  setTheme: (theme: Theme) => void;
}
```

**State Flow**:
1. index.html script runs → reads localStorage → applies 'dark' class (or not)
2. React mounts → ThemeProvider initializes → reads same localStorage
3. User toggles → setTheme updates localStorage + DOM class
4. System preference changes → listener updates resolvedTheme

---

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/ThemeContext.tsx` | CREATE | Theme provider + hook |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/index.html` | MODIFY | Add FOUC prevention script |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/App.tsx` | MODIFY | Wrap with ThemeProvider |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/index.css` | MODIFY | Add transition utilities |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/tailwind.config.ts` | VERIFY | Confirm darkMode: ["class"] |

---

## Implementation Steps

### Step 1: Create ThemeContext.tsx

Location: `src/contexts/ThemeContext.tsx`

```typescript
// Key implementation points:
// 1. Use useState with lazy initializer for localStorage read
// 2. useMemo for resolvedTheme computation
// 3. useEffect for system preference listener
// 4. useCallback for setTheme to prevent re-renders
// 5. Apply 'dark' class to document.documentElement
```

### Step 2: Add FOUC Prevention Script to index.html

Insert in `<head>` BEFORE any React scripts:

```html
<script>
  (function() {
    const theme = localStorage.getItem('theme');
    const systemDark = window.matchMedia('(prefers-color-scheme: dark)').matches;
    const isDark = theme === 'dark' || (theme === 'system' || !theme) && systemDark;
    if (isDark) document.documentElement.classList.add('dark');
  })();
</script>
```

### Step 3: Add Transition Utilities to index.css

```css
/* Theme transition class - applied during toggle */
html.theme-transitioning,
html.theme-transitioning * {
  transition: background-color 0.3s ease, color 0.3s ease, border-color 0.3s ease !important;
}

/* Respect reduced motion */
@media (prefers-reduced-motion: reduce) {
  html.theme-transitioning,
  html.theme-transitioning * {
    transition: none !important;
  }
}
```

### Step 4: Wrap App with ThemeProvider

In `src/App.tsx`, add ThemeProvider to provider hierarchy:

```typescript
import { ThemeProvider } from "@/contexts/ThemeContext";

// Wrap inside ErrorBoundary, outside AuthProvider
<ThemeProvider>
  <AuthProvider>
    ...
  </AuthProvider>
</ThemeProvider>
```

### Step 5: Create useTheme Hook Export

Export `useTheme` hook from ThemeContext for component usage:

```typescript
export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider');
  return ctx;
}
```

---

## Todo List

- [ ] Create `src/contexts/ThemeContext.tsx` with full TypeScript types
- [ ] Add FOUC prevention script to `index.html` `<head>`
- [ ] Add `.theme-transitioning` CSS class to `index.css`
- [ ] Add reduced-motion media query for transitions
- [ ] Import and wrap App with ThemeProvider in `App.tsx`
- [ ] Add system preference change listener in ThemeContext
- [ ] Verify dark class toggles on `document.documentElement`
- [ ] Test localStorage persistence across page reloads

---

## Success Criteria

- [ ] `useTheme()` returns correct `theme` and `resolvedTheme`
- [ ] Theme persists in localStorage after page reload
- [ ] No FOUC visible on hard refresh
- [ ] System preference changes reflect immediately
- [ ] Transition is smooth (0.3s) without jarring flash
- [ ] Reduced motion users see instant theme change

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| FOUC despite script | Low | High | Test in production build, not dev mode |
| Context re-renders | Medium | Medium | Use useMemo/useCallback in provider |
| SSR mismatch | Low | Medium | N/A - Vite is client-only |
| localStorage unavailable | Low | Low | Fallback to 'system' default |

---

## Security Considerations

- **No sensitive data** in localStorage (only 'light'/'dark'/'system' string)
- **XSS safe** - Script is inline, not user-controlled
- **No CORS** - All operations are client-side

---

## Test Cases

| ID | Test Case | Expected Result |
|----|-----------|-----------------|
| TC-01 | Load page first time (no localStorage) | Uses system preference |
| TC-02 | Set theme to 'dark', reload page | Dark theme persists |
| TC-03 | Set theme to 'light', reload page | Light theme persists |
| TC-04 | Set theme to 'system', change OS preference | Theme updates immediately |
| TC-05 | Hard refresh in dark mode | No FOUC (no flash of white) |
| TC-06 | Toggle theme with reduced motion enabled | Instant change, no transition |
| TC-07 | Call useTheme outside ThemeProvider | Throws descriptive error |

---

## Next Steps

After this phase:
1. [Phase 02: Light Mode Design](./phase-02-light-mode-design.md) - Define CSS variables for light mode
