# Scout Report: Frontend Files for Theme + i18n Implementation

**Date**: 2025-12-09 | **Status**: Complete

---

## Current State Analysis

### i18n Setup (EXISTING - Needs Enhancement)
| File | Purpose | Status |
|------|---------|--------|
| `src/i18n/index.ts` | i18n config | ✅ Exists (4 langs: EN, VI, FR, ZH) |
| `src/components/LanguageSelector.tsx` | Language switcher | ✅ Exists |
| `src/main.tsx` | Entry point | ✅ Imports i18n |

**Gaps**:
- ❌ Missing Japanese (JA) translations
- ❌ All translations inline in single file (no namespaces)
- ❌ No localStorage persistence for language preference
- ❌ No lazy loading of translations
- ❌ Limited to landing page strings only

### Theme Setup (EXISTING - Needs Enhancement)
| File | Purpose | Status |
|------|---------|--------|
| `tailwind.config.ts` | Theme config | ✅ `darkMode: ["class"]` enabled |
| `src/index.css` | CSS variables | ✅ Dark theme defined, `.dark` class exists but unused |

**Gaps**:
- ❌ No ThemeProvider/Context
- ❌ No theme toggle component
- ❌ No localStorage persistence
- ❌ No FOUC prevention script
- ❌ Light mode variables not fully defined (only dark luxury theme)
- ❌ No system preference detection

---

## Files to MODIFY

### Priority 1: Core Infrastructure
```
nextjs-ui-dashboard/
├── src/index.css                    # Add light mode CSS variables
├── src/main.tsx                     # Add ThemeProvider + LanguageProvider
├── index.html                       # Add FOUC prevention script
└── tailwind.config.ts               # Already correct (class mode)
```

### Priority 2: i18n Restructure
```
nextjs-ui-dashboard/
├── src/i18n/index.ts               # Refactor: namespaces, lazy loading
├── src/components/LanguageSelector.tsx  # Update UI + persist to localStorage
```

### Priority 3: Component Updates (Apply translations)
```
nextjs-ui-dashboard/src/
├── pages/                           # All 16+ pages need t() calls
│   ├── Index.tsx                    # Landing page
│   ├── Dashboard.tsx
│   ├── PaperTrading.tsx
│   ├── RealTrading.tsx
│   ├── Settings.tsx
│   ├── Profile.tsx
│   ├── Portfolio.tsx
│   └── ... (12 more)
├── components/
│   ├── TradingInterface.tsx
│   ├── dashboard/*.tsx (10+ files)
│   ├── ai/*.tsx
│   └── trading/*.tsx
└── hooks/                           # useWebSocket, useTradingApi error messages
```

---

## Files to CREATE

### Theme System
```
nextjs-ui-dashboard/src/
├── contexts/ThemeContext.tsx        # ThemeProvider + useTheme hook
├── components/ThemeToggle.tsx       # Toggle button (light/dark/system)
└── lib/theme-utils.ts               # resolvedTheme, localStorage helpers
```

### i18n System Enhancement
```
nextjs-ui-dashboard/src/i18n/
├── locales/
│   ├── en/
│   │   ├── common.json              # Shared UI strings
│   │   ├── trading.json             # Trading-specific
│   │   ├── dashboard.json           # Dashboard page
│   │   └── errors.json              # Error messages
│   ├── vi/
│   │   └── (same structure)
│   ├── fr/
│   │   └── (same structure)
│   ├── zh/
│   │   └── (same structure)
│   └── ja/                          # NEW: Japanese
│       └── (same structure)
├── config.ts                        # Main i18next config
└── i18n.d.ts                        # TypeScript augmentation
```

### Additional
```
nextjs-ui-dashboard/src/
├── contexts/LanguageContext.tsx     # Language preference persistence
└── hooks/useLanguage.ts             # Custom hook
```

---

## Key Files Content Summary

### tailwind.config.ts (Line 4)
```typescript
darkMode: ["class"], // ✅ Already configured for class-based theme
```

### src/index.css (Lines 8-114)
- `:root` - Dark luxury theme as default (OLED black base)
- `.dark` - Alternative dark theme (different values - INCONSISTENT)
- **Issue**: `.dark` class exists but no light mode defined

### src/i18n/index.ts (Lines 1-234)
- 4 languages: en, vi, fr, zh
- All translations inline (not modular)
- Landing page strings only (nav, hero, features, pricing, cta)
- **Missing**: Dashboard, trading, errors, settings, ja (Japanese)

### src/components/LanguageSelector.tsx (Lines 1-44)
- Uses Shadcn Select component
- 4 languages: EN, VI, FR, ZH
- **Missing**: JA, localStorage persistence

### src/App.tsx (Lines 1-216)
- 17 lazy-loaded pages
- 6 context providers (Auth, WebSocket, AI, PaperTrading, TradingMode, Notification)
- **Missing**: ThemeProvider, LanguageProvider

---

## Dependencies Analysis

### Already Installed
- `i18next` + `react-i18next` ✅
- `@radix-ui/*` (all Shadcn components) ✅
- `tailwindcss` + `tailwindcss-animate` ✅

### To Install
```bash
npm install i18next-browser-languagedetector  # Browser language detection
# Optional: i18next-http-backend (if using external JSON files)
```

---

## Affected Routes Count

| Category | Count | Files |
|----------|-------|-------|
| Public pages | 12 | Index, Login, Register, Features, Pricing, API, Documentation, About, Blog, Careers, Contact, Privacy, Terms, Security, Compliance |
| Protected pages | 10 | Dashboard, PaperTrading, RealTrading, Profile, Portfolio, AISignals, Settings, TradeAnalyses, HowItWorks, TradingPaper |
| **Total** | **22 pages** | Need i18n + theme support |

---

## Estimated Scope

| Task | Files | Complexity |
|------|-------|------------|
| ThemeProvider + Toggle | 4 | Medium |
| Light mode CSS variables | 1 | Medium |
| FOUC prevention | 1 | Low |
| i18n restructure (namespaces) | 8 | High |
| Add Japanese translations | 4 | Medium |
| Apply t() to all pages | 22+ | High |
| Apply t() to components | 30+ | Medium |
| Update LanguageSelector | 1 | Low |
| Testing | - | Medium |

**Total estimated files**: ~70+ files to touch
**Estimated effort**: 4-6 implementation days

---

## Next Steps

1. Create ThemeContext + light mode CSS variables
2. Refactor i18n to namespace-based structure
3. Add Japanese translations
4. Apply translations to pages (prioritize: Index, Dashboard, Trading)
5. Test theme + language switching
6. Write spec + test cases
