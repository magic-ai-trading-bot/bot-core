# Tailwind CSS v3 to v4 Migration Research Report

**Date**: 2026-02-25
**Scope**: Complete migration guide for nextjs-ui-dashboard project
**Status**: Comprehensive research complete — ready for implementation planning

---

## Executive Summary

Migration from Tailwind v3 to v4 is a **breaking change with moderate complexity**. Key impacts:

1. **PostCSS plugin moved** to separate `@tailwindcss/postcss` package
2. **CSS directives completely redesigned** — single `@import "tailwindcss"` replaces `@tailwind base/components/utilities`
3. **Utility class names canonicalized** — aliases removed (65+ classes renamed)
4. **Config file now optional** — but loadable via `@config "./tailwind.config.ts"` directive
5. **Autoprefixer no longer needed** — Lightning CSS handles prefixing
6. **Plugin ecosystem mostly compatible** — but requires v4-specific versions

---

## 1. PostCSS Configuration Migration

### Current State (v3)
```javascript
// postcss.config.js
export default {
  plugins: {
    tailwindcss: {},
    autoprefixer: {},
  },
};
```

### v4 Configuration
```javascript
// postcss.config.js
export default {
  plugins: {
    '@tailwindcss/postcss': {},
  },
};
```

### Key Changes
- **Remove**: `autoprefixer: {}` — no longer needed (Lightning CSS handles it)
- **Replace**: `tailwindcss: {}` → `'@tailwindcss/postcss': {}`
- **Dependency**: Install `@tailwindcss/postcss` (should already be installed per your project notes)

### Migration Path
```bash
# Install new plugin (already done in your project)
npm install @tailwindcss/postcss

# Remove old dependency (if present)
npm uninstall autoprefixer
```

---

## 2. CSS File Directives

### Current State (v3)
```css
/* index.css */
@tailwind base;
@tailwind components;
@tailwind utilities;
```

### v4 Conversion
```css
/* index.css */
@import "tailwindcss";
```

### Optional: Load Config
If you want to keep `tailwind.config.ts` for custom animations/colors:
```css
/* index.css */
@import "tailwindcss";
@config "./tailwind.config.ts";
```

### Critical Notes
- The `@config` directive **MUST come after** `@import "tailwindcss"` statement
- Removes all `@tailwind base/components/utilities` directives
- One-liner import replaces three directives

### Custom Utilities in v4
Old way (v3):
```css
@layer utilities {
  .tab-4 {
    tab-size: 4;
  }
}
```

New way (v4):
```css
@utility tab-4 {
  tab-size: 4;
}
```

The `@utility` directive automatically applies variant support (hover:, dark:, responsive prefixes).

---

## 3. Loading tailwind.config.ts in v4

### Option A: Direct CSS Loading (Recommended)
```css
/* index.css */
@import "tailwindcss";
@config "./tailwind.config.ts";
```

### Option B: Keep JavaScript Config File
```javascript
// tailwind.config.ts (unchanged from v3)
export default {
  theme: {
    extend: {
      colors: {
        // Your CSS vars...
      },
      animation: {
        // Your animations...
      },
    },
  },
  plugins: [
    // Your plugins...
  ],
};
```

### Limitations ⚠️
The following v3 config options **are NOT supported in v4** CSS-only mode:
- `corePlugins` — must use `@config` to JS file
- `safelist` — must use `@config` to JS file
- `separator` — must use `@config` to JS file

**For your project**: Since you have custom colors (CSS vars) and animations, **you MUST keep `tailwind.config.ts`** and use `@config "./tailwind.config.ts"` in your CSS file.

---

## 4. tailwindcss-animate Compatibility

### Current Status
❌ **BREAKING**: The original `tailwindcss-animate` package is **incompatible with v4** as a PostCSS plugin.

### Replacement Options

#### Option 1: tw-animate-css (Official v4 Compatible)
```bash
npm install tw-animate-css
```

**Usage in tailwind.config.ts**:
```typescript
plugins: [require("tw-animate-css")],
```

**Pros**:
- Official TailwindCSS v4 compatible replacement
- CSS-first approach aligned with v4 philosophy
- Same animation utilities as original

#### Option 2: CSS-Only Custom Animations (Native v4)
Define directly in your CSS using `@theme`:
```css
@import "tailwindcss";

@theme {
  --animate-bounce: bounce 1s infinite;
  --animate-spin: spin 1s linear infinite;
}

@keyframes bounce {
  0%, 100% { transform: translateY(0); }
  50% { transform: translateY(-25%); }
}
```

Then use as normal: `animate-bounce`, `animate-spin`

**Pros**:
- No plugin dependency
- Pure CSS approach (v4 native)
- Smaller bundle size

#### Option 3: Keep JS Config + Original Plugin
If using `@config "./tailwind.config.ts"` with JS config:
```typescript
// tailwind.config.ts
import animate from "tailwindcss-animate";

export default {
  plugins: [animate],
};
```

**Pros**:
- Backward compatible
- No code changes to existing animations

**Cons**:
- Not the v4 recommended approach
- Still uses legacy plugin system

### Recommendation for Your Project
**Use Option 1 (tw-animate-css)**:
- Cleanest v4 integration path
- Drop-in replacement (same API)
- Official ecosystem support
- No CSS rewrites needed

---

## 5. Autoprefixer Removal

### Current Dependency (v3)
```javascript
// postcss.config.js
plugins: {
  tailwindcss: {},
  autoprefixer: {},  // ← This is no longer needed
}
```

### v4 Built-In Handling
✅ **YES** — Autoprefixer is now built-in via Lightning CSS

**What changed**:
- Tailwind v4 uses Lightning CSS for CSS compilation
- Vendor prefixing is automatic (no configuration needed)
- PostCSS chain simplified to just `@tailwindcss/postcss`

### Migration Action
```bash
# Remove autoprefixer from package.json
npm uninstall autoprefixer

# Update postcss.config.js
# ✅ Before: { tailwindcss: {}, autoprefixer: {} }
# ✅ After:  { '@tailwindcss/postcss': {} }
```

### Impact on Your Code
- No CSS changes required
- Vendor prefixes generated automatically
- Example: `transform: translateX(10px)` becomes `-webkit-transform: translateX(10px)` automatically

---

## 6. Utility Class Name Breaking Changes

### Most Impactful Changes (65+ classes affected)

#### Size Scale Changes (shadow, blur, rounded, etc.)
| v3 Class | v4 Class | Reason |
|----------|----------|--------|
| `shadow-sm` | `shadow-xs` | Scale realignment |
| `shadow` | `shadow-sm` | Gap removed |
| `blur-sm` | `blur-xs` | Consistent sizing |
| `blur` | `blur-sm` | Consistent sizing |
| `rounded-sm` | `rounded-xs` | Consistent sizing |
| `rounded` | `rounded-sm` | Gap removed |

#### Opacity Modifiers (Removed)
| v3 Class | v4 Replacement |
|----------|----------------|
| `bg-opacity-50` | `bg-black/50` (opacity modifier) |
| `text-opacity-75` | `text-white/75` (opacity modifier) |
| `border-opacity-25` | `border-red-500/25` (opacity modifier) |
| `ring-opacity-50` | `ring-blue-500/50` (opacity modifier) |

#### Flex Shorthand (Renamed)
| v3 Class | v4 Class |
|----------|----------|
| `flex-shrink-0` | `shrink-0` |
| `flex-grow-1` | `grow-1` |
| `flex-shrink-1` | `shrink-1` |

#### Ring/Outline Defaults Changed
| v3 | v4 | Change |
|----|----|----|
| `outline` (width 2px) | `outline-2` | Explicitly set width |
| `outline-none` | `outline-hidden` | Correct CSS semantics |
| `ring ring-blue-500` | `ring-3 ring-blue-500` | Default ring width 3px → must be explicit |

#### Selector Changes (Performance)
- `space-x-4`, `space-y-4` child selector changed
- `divide-x-4`, `divide-y-4` child selector changed
- From: `:not([hidden]) ~ :not([hidden])`
- To: `:not(:last-child)`

#### Miscellaneous Removals
| v3 | v4 |
|----|----|
| `overflow-ellipsis` | `text-ellipsis` |
| `decoration-slice` | `box-decoration-slice` |
| `decoration-clone` | `box-decoration-clone` |
| `rotate-45` (transform syntax) | `rotate-45` (new CSS syntax) |
| `scale-150 focus:transform-none` | `scale-150 focus:scale-none` |

### Important: Arbitrary Values with CSS Variables
```html
<!-- v3 -->
<div class="bg-[--brand-color]">

<!-- v4 -->
<div class="bg-(--brand-color)">
```

Parentheses required for CSS variable syntax in arbitrary values.

### Automated Fix Available
```bash
# Run Tailwind's official codemod (handles ~90% of changes)
npx @tailwindcss/upgrade
```

This scans HTML, JSX, TSX, and CSS files — automatically renames classes.

---

## 7. @tailwindcss/typography Compatibility

### Status
✅ **COMPATIBLE** — with v4-specific version required

### Installation & Setup
```bash
npm install @tailwindcss/typography
```

**v3 Configuration** (in tailwind.config.js):
```javascript
export default {
  plugins: [require('@tailwindcss/typography')],
};
```

**v4 Configuration** (CSS import):
```css
/* index.css */
@import "tailwindcss";
@plugin "@tailwindcss/typography";
```

### Important: Version Compatibility
Ensure you have a recent release of `@tailwindcss/typography` that explicitly supports v4. Update if needed:
```bash
npm update @tailwindcss/typography
```

### Customization in v4
If you need to customize typography styles (colors, font sizes, etc.):

1. Enable JS config:
   ```css
   @import "tailwindcss";
   @config "./tailwind.config.ts";
   @plugin "@tailwindcss/typography";
   ```

2. In tailwind.config.ts:
   ```typescript
   export default {
     theme: {
       extend: {
         typography: {
           DEFAULT: {
             css: {
               color: '#your-color',
               // ... customizations
             },
           },
         },
       },
     },
     plugins: [require('@tailwindcss/typography')],
   };
   ```

### Alternative: CSS-Only Solution
For simpler cases, use **tw-prose** (pure CSS typography plugin):
```bash
npm install tw-prose
```

```css
@import "tailwindcss";
@plugin "tw-prose";
```

**Pros**: No JS config needed, smaller footprint
**Cons**: Less customizable than official plugin

---

## 8. Configuration File (@config) Deep Dive

### When to Use @config
**Use `@config` if your project has**:
- Custom colors (CSS variables) ✓ *Your project has this*
- Custom animations ✓ *Your project has this*
- Plugin dependencies (tailwindcss-animate, typography) ✓ *Your project has this*
- `corePlugins`, `safelist`, or `separator` customization

### Correct Syntax
```css
/* index.css */
@import "tailwindcss";
@config "./tailwind.config.ts";
```

**Critical rule**: `@config` MUST come AFTER `@import "tailwindcss"`

### File Resolution
- Relative paths resolved from CSS file location
- Can use `.js` or `.ts` extensions
- CommonJS or ES modules both supported

---

## 9. Migration Execution Checklist

### Phase 1: Dependencies (1-2 min)
- [ ] `npm install @tailwindcss/postcss`
- [ ] `npm uninstall autoprefixer` (optional, but recommended)
- [ ] `npm install tw-animate-css` (or keep original plugin)
- [ ] `npm update @tailwindcss/typography`

### Phase 2: Configuration (3-5 min)
- [ ] Update `postcss.config.js` — replace `tailwindcss: {}` with `'@tailwindcss/postcss': {}`
- [ ] Update `index.css` — replace three `@tailwind` directives with `@import "tailwindcss"`
- [ ] Add `@config "./tailwind.config.ts"` (AFTER import)
- [ ] Update animation plugin in `tailwind.config.ts` (if using tw-animate-css)

### Phase 3: Code Fixes (5-15 min, depends on codebase size)
- [ ] Run `npx @tailwindcss/upgrade` codemod to fix class names (~90% automated)
- [ ] Manual search for custom `.css` files with `@layer utilities` → convert to `@utility`
- [ ] Search for deprecated opacity-* classes → convert to `/` syntax
- [ ] Verify arbitrary value CSS variables: `[--var]` → `(--var)`

### Phase 4: Testing & Validation (10-20 min)
- [ ] Full browser visual regression test (all pages/components)
- [ ] Check dark mode switching
- [ ] Check responsive breakpoints (mobile, tablet, desktop)
- [ ] Verify animation playback
- [ ] Check custom utility classes work with variants
- [ ] Run unit tests for any CSS-dependent logic

### Phase 5: Optional Optimizations
- [ ] Remove `autoprefixer` from package.json and postcss.config.js
- [ ] Review bundle size (should be similar or smaller)
- [ ] Verify typography plugin customizations still work

---

## 10. Risk Assessment

### Low Risk (Proceed Confidently)
- Migrating postcss.config.js ✓ Mechanical, well-documented
- Updating CSS directives ✓ Single-file change
- Using @config ✓ Optional, backward compatible
- Removing autoprefixer ✓ Automatic in v4

### Medium Risk (Requires Testing)
- Class name aliases ✓ Automated by `@tailwindcss/upgrade` tool
- Typography plugin ✓ Usually works but verify output
- Custom animations ✓ Works with tw-animate-css or CSS-only

### Mitigation Strategies
1. **Use automated codemod** for utility class names (handles 90%)
2. **Run full visual regression test** after migration
3. **Keep git branch** to revert if issues found
4. **Test in browser DevTools** for responsive/dark mode
5. **Validate component library** output

---

## 11. Project-Specific Action Items

### Your Stack
- **Framework**: Next.js + React 18 + TypeScript + Vite
- **UI**: Shadcn/UI + TailwindCSS + Tailwind Animate
- **CSS Location**: `nextjs-ui-dashboard/src/index.css`
- **Config Location**: Root-level `tailwind.config.ts`

### Recommended Migration Path

1. **Create feature branch**:
   ```bash
   git checkout -b chore/tailwind-v4-migration
   ```

2. **Update dependencies**:
   ```bash
   npm install @tailwindcss/postcss tw-animate-css
   npm uninstall autoprefixer
   ```

3. **Update configuration files**:
   - `postcss.config.js`: Replace plugin names
   - `index.css`: Replace directives
   - `tailwind.config.ts`: Update animate plugin

4. **Run automated fixes**:
   ```bash
   npx @tailwindcss/upgrade
   ```

5. **Manual code review**:
   - Check for missed opacity-* classes
   - Verify arbitrary CSS variable syntax
   - Test shadcn/ui components rendering

6. **Test coverage**:
   - Visual regression across all pages
   - Component library rendering
   - Responsive breakpoints
   - Dark mode toggle
   - Animation playback

7. **Commit & document**:
   - Update CHANGELOG
   - Document any custom changes made
   - Create PR for review

---

## 12. Unresolved Questions / Edge Cases

1. **Shadcn/UI compatibility** — Need to verify exact version compatibility with Tailwind v4 (likely already v4-ready)

2. **Custom theme variables** — Your CSS variables approach should work, but recommend testing custom color outputs

3. **Bundle size impact** — v4 claims smaller bundle, but worth measuring post-migration

4. **IDE IntelliSense** — Tailwind v4 support in IDEs may require plugin updates

5. **Storybook/other tools** — If using Storybook, may need configuration updates for v4

---

## Sources

- [Tailwind CSS v4 Upgrade Guide](https://tailwindcss.com/docs/upgrade-guide)
- [Tailwind CSS v4.0 Release Blog](https://tailwindcss.com/blog/tailwindcss-v4)
- [PostCSS Plugin Migration Issue](https://github.com/tailwindlabs/tailwindcss/issues/15735)
- [tw-animate-css GitHub](https://github.com/Wombosvideo/tw-animate-css)
- [Tailwind CSS v4 Functions & Directives](https://tailwindcss.com/docs/functions-and-directives)
- [Tailwind Typography Plugin Docs](https://github.com/tailwindlabs/tailwindcss-typography)
- [DesignRevision Tailwind v4 Migration Guide](https://designrevision.com/blog/tailwind-4-migration)
- [HeroUI Tailwind v4 Guide](https://www.heroui.com/docs/guide/tailwind-v4)

