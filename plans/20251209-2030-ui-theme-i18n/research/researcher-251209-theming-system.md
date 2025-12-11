# Dark/Light Mode Theme System Research
**Date**: 2025-12-09 | **Status**: Complete | **Tech Stack**: React 18 + TypeScript + Tailwind CSS + Shadcn/UI

---

## Executive Summary
Implement class-based theme switching with Tailwind CSS, CSS variables for luxury colors, and React Context for state management. Shadcn/UI provides framework-agnostic foundation; leverage Vite-specific patterns for optimal performance.

---

## Key Findings

### 1. Tailwind CSS Dark Mode Strategy
**Configuration**: Enable class mode in `tailwind.config.js`:
```js
export default {
  darkMode: 'class', // Toggle 'dark' class on root element
  theme: { extend: { /* custom luxury colors */ } }
}
```

**Why Class Mode?**
- Allows manual toggle + system preference detection
- Better control than media-query mode
- Essential for luxury aesthetic consistency

**Color Definition Pattern**:
```css
:root {
  --background: 0 0% 100%;      /* Light: off-white */
  --foreground: 0 0% 3.9%;      /* Light: near-black */
  --primary: 45 93% 47%;        /* Gold accent */
  --accent: 259 94% 51%;        /* Deep purple */
}

.dark {
  --background: 0 0% 3.9%;      /* Dark: near-black */
  --foreground: 0 0% 98%;       /* Dark: off-white */
  --primary: 45 100% 55%;       /* Gold (brighter in dark) */
  --accent: 259 95% 60%;        /* Purple (adjusted for dark) */
}
```

**Use oklch() for consistency** across light/dark (better perceptual uniformity than hex).

---

### 2. Shadcn/UI Theming Approach
Shadcn/UI uses **class-based CSS variables** with framework-specific implementations:

**Core Pattern**:
- Initialization script detects: localStorage → system preference → defaults to 'light'
- Applies 'dark' class to document root
- CSS variables update automatically
- No page reload needed

**For Vite (Your Setup)**:
1. No `next-themes` dependency required
2. Create custom `ThemeProvider` using React Context
3. Add script in `<head>` to prevent FOUC
4. localStorage persists user preference

**Shadcn Color Variables** (17 CSS variables):
- Core: background, foreground, card, popover
- Interactive: primary, secondary, muted, accent, destructive
- UI: border, input, ring
- Charts: chart-1 through chart-5

---

### 3. CSS Variables for Luxury Palette
**Strategy**: Use oklch() color space for sophisticated control.

**Luxury Color Ranges**:
- **Light Mode**: Soft whites (100%), deep blacks (5%), gold accents (45°), deep purples (259°)
- **Dark Mode**: Deep backgrounds (5%), bright foregrounds (95%), enhanced saturation for accents

**Example Luxury Setup**:
```css
:root {
  /* Neutral Hierarchy */
  --bg-primary: 0 0% 99.5%;     /* Almost white */
  --bg-secondary: 0 0% 96%;     /* Light gray */
  --text-primary: 0 0% 8%;      /* Near black */
  --text-secondary: 0 0% 45%;   /* Medium gray */

  /* Luxury Accents */
  --gold: 45 85% 52%;           /* Warm gold */
  --sapphire: 270 90% 50%;      /* Deep blue */
  --emerald: 145 80% 45%;       /* Rich green */
}

.dark {
  --bg-primary: 0 0% 4%;        /* Near black */
  --bg-secondary: 0 0% 12%;     /* Dark gray */
  --text-primary: 0 0% 96%;     /* Near white */
  --text-secondary: 0 0% 65%;   /* Light gray */

  --gold: 45 95% 62%;           /* Brighter gold */
  --sapphire: 270 95% 60%;      /* Brighter blue */
  --emerald: 145 85% 55%;       /* Brighter green */
}
```

**Apply in Tailwind**:
```js
// tailwind.config.ts
theme: {
  extend: {
    colors: {
      background: 'hsl(var(--background) / <alpha-value>)',
      foreground: 'hsl(var(--foreground) / <alpha-value>)',
      gold: 'hsl(var(--gold) / <alpha-value>)',
    }
  }
}
```

---

### 4. React Context Pattern (TypeScript)
**Recommended: Optimized Context with Performance Guards**

```typescript
// lib/theme-context.tsx
import { createContext, useContext, useCallback, useMemo, ReactNode } from 'react';

type Theme = 'light' | 'dark' | 'system';

interface ThemeContextType {
  theme: Theme;
  resolvedTheme: 'light' | 'dark'; // Computed: system resolved to actual
  setTheme: (theme: Theme) => void;
}

const ThemeContext = createContext<ThemeContextType | undefined>(undefined);

export function ThemeProvider({ children }: { children: ReactNode }) {
  const [theme, setThemeState] = useState<Theme>(() => {
    const saved = localStorage.getItem('theme') as Theme | null;
    return saved || 'system';
  });

  const resolvedTheme = useMemo(() => {
    if (theme === 'system') {
      return window.matchMedia('(prefers-color-scheme: dark)').matches
        ? 'dark'
        : 'light';
    }
    return theme;
  }, [theme]);

  const setTheme = useCallback((newTheme: Theme) => {
    localStorage.setItem('theme', newTheme);
    setThemeState(newTheme);
    document.documentElement.classList.toggle('dark', resolvedTheme === 'dark');
  }, [resolvedTheme]);

  const value = useMemo(
    () => ({ theme, resolvedTheme, setTheme }),
    [theme, resolvedTheme, setTheme]
  );

  return (
    <ThemeContext.Provider value={value}>
      {children}
    </ThemeContext.Provider>
  );
}

export function useTheme() {
  const ctx = useContext(ThemeContext);
  if (!ctx) throw new Error('useTheme must be used within ThemeProvider');
  return ctx;
}
```

**Key Points**:
- `useCallback` prevents unnecessary re-renders
- `useMemo` optimizes context value stability
- Type-safe enum alternatives available if preferred
- Error boundary for debugging

---

### 5. Animation/Transition Best Practices
**FOUC Prevention** (Flash of Unstyled Content):
```html
<!-- In <head> BEFORE React mount -->
<script>
  const theme = localStorage.getItem('theme') || 'system';
  const isDark = theme === 'dark' ||
    (theme === 'system' && window.matchMedia('(prefers-color-scheme: dark)').matches);
  if (isDark) document.documentElement.classList.add('dark');
</script>
```

**Smooth Transitions** (2 Approaches):

**Approach A - Global CSS Transition** (Simple, recommended for luxury):
```css
/* Global transition on theme switch */
html.theme-transitioning,
html.theme-transitioning * {
  transition: background-color 0.3s ease-in-out,
              color 0.3s ease-in-out,
              border-color 0.3s ease-in-out !important;
}
```

**Approach B - Selective Component Transitions** (Fine-grained control):
```tsx
const ThemeToggle = () => {
  const { setTheme } = useTheme();

  const handleChange = (newTheme: Theme) => {
    document.documentElement.classList.add('theme-transitioning');
    setTheme(newTheme);
    setTimeout(() => {
      document.documentElement.classList.remove('theme-transitioning');
    }, 300);
  };
};
```

**Respect Prefers-Reduced-Motion**:
```css
@media (prefers-reduced-motion: reduce) {
  html.theme-transitioning,
  html.theme-transitioning * {
    transition: none !important;
  }
}
```

---

## Implementation Roadmap

### Phase 1: Setup (Day 1)
- [ ] Update `tailwind.config.ts`: enable class mode + custom luxury colors
- [ ] Create CSS variables in `globals.css` (light & dark modes)
- [ ] Add FOUC prevention script to `index.html`

### Phase 2: Context (Day 1-2)
- [ ] Create `lib/theme-context.tsx` with optimized provider
- [ ] Wrap root component in `ThemeProvider`
- [ ] Create `useTheme()` hook for components

### Phase 3: UI Component (Day 2)
- [ ] Build theme toggle with Shadcn Button + DropdownMenu
- [ ] Test system preference detection
- [ ] Verify localStorage persistence

### Phase 4: Testing (Day 2-3)
- [ ] Test light → dark transition smoothness
- [ ] Verify FOUC prevention
- [ ] Test localStorage across sessions
- [ ] Accessibility audit (ARIA, reduced motion)
- [ ] Test across devices (mobile, tablet, desktop)

### Phase 5: Luxury Polish (Day 3)
- [ ] Fine-tune color values for both modes
- [ ] Add micro-animations to toggle
- [ ] Update design guidelines doc
- [ ] Complete @spec tags in code

---

## Potential Pitfalls & Mitigations

| Pitfall | Risk | Mitigation |
|---------|------|-----------|
| FOUC on reload | CLS, poor UX | Use <head> script (no React dependency) |
| Context re-render spam | Performance | Use useCallback + useMemo in provider |
| Hardcoded colors breaking theme | Maintenance nightmare | Use CSS variables everywhere, zero hardcoded colors |
| localStorage override | Wrong theme applied | Check: saved → system → default priority |
| Contrast issues in luxury palette | WCAG failures | Validate all text colors (4.5:1 minimum for AA) |
| Missing transitions | Jarring UX | Global transition rule + reduced-motion support |
| System preference change ignored | UX issue | Watch `prefers-color-scheme` with matchMedia |

---

## Recommended Tools & Validation

**Color Contrast Testing**:
- [WCAG Contrast Checker](https://webaim.org/resources/contrastchecker/) (validate luxury colors)
- Use DevTools "Inspect" → Colors tab for real-time testing

**CSS Variable Inspector**:
```bash
# Check if all colors are using CSS variables (not hardcoded)
grep -r "rgb\|#[0-9a-f]" src --include="*.tsx" --include="*.ts" --exclude-dir=node_modules
# Should return 0 results for production colors
```

**Accessibility Testing**:
- Test with `prefers-reduced-motion: reduce` in DevTools
- Screen reader testing (NVDA/JAWS)
- Keyboard-only navigation

---

## References & Sources

- [Tailwind CSS Dark Mode Documentation](https://tailwindcss.com/docs/dark-mode)
- [shadcn/ui Dark Mode Guide](https://ui.shadcn.com/docs/dark-mode)
- [shadcn/ui Theming Documentation](https://ui.shadcn.com/docs/theming)
- [CSS-Tricks: Easy Dark Mode in React](https://css-tricks.com/easy-dark-mode-and-multiple-color-themes-in-react/)
- [React Context with TypeScript Best Practices](https://www.pluralsight.com/resources/blog/guides/using-reacts-context-api-with-typescript)
- [Implementing Theme in React Using Context API](https://medium.com/@riteshbhagat/implementing-theme-in-react-using-context-api-196149967c9d)
- [Josh W. Comeau: The Quest for the Perfect Dark Mode](https://www.joshwcomeau.com/react/dark-mode/)

---

**Status**: ✅ Ready for implementation
**Confidence**: High (based on production patterns from Shadcn/UI + Tailwind ecosystem)
**Next Step**: Create FR-THEME-001 spec + TC-THEME-001 test cases
