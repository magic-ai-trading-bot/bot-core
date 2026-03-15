# Phase 03: i18n Restructure

**Priority**: High | **Status**: Pending | **Est. Effort**: 6 hours

---

## Context Links

- [Main Plan](./plan.md)
- [i18n Best Practices Research](./research/researcher-251209-i18n-best-practices.md)
- [Scout Report](./scout/scout-01-frontend-analysis.md)

---

## Overview

Restructure i18n from single inline file to namespace-based modular structure. Add Japanese (JA) language. Implement LanguageContext for persistence and lazy loading.

---

## Key Insights

1. **Current structure** - All translations inline in `src/i18n/index.ts` (234 lines)
2. **Only landing page** - nav, hero, features, pricing, cta keys
3. **Missing** - Japanese, dashboard, trading, errors, settings translations
4. **No persistence** - Language resets on page reload
5. **No TypeScript safety** - Translation keys not type-checked

---

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | Namespace-based structure (common, trading, dashboard, errors) | Critical |
| R2 | Add Japanese (JA) as 5th language | Critical |
| R3 | LanguageContext with localStorage persistence | Critical |
| R4 | TypeScript augmentation for type-safe keys | High |
| R5 | Browser language detection (i18next-browser-languagedetector) | High |
| R6 | Lazy loading of non-default namespaces | Medium |
| R7 | Fallback to English for missing keys | High |

---

## Architecture

**New Folder Structure**:
```
src/i18n/
├── config.ts                 # Main i18next config
├── index.ts                  # Re-export for backward compat
├── i18n.d.ts                 # TypeScript augmentation
└── locales/
    ├── en/
    │   ├── common.json       # Shared UI (buttons, labels, nav)
    │   ├── landing.json      # Landing page (move existing)
    │   ├── trading.json      # Trading-specific
    │   ├── dashboard.json    # Dashboard page
    │   └── errors.json       # Error messages
    ├── vi/
    │   └── (same structure)
    ├── fr/
    │   └── (same structure)
    ├── zh/
    │   └── (same structure)
    └── ja/                   # NEW
        └── (same structure)
```

---

## Related Code Files

| File | Action | Purpose |
|------|--------|---------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/index.ts` | REFACTOR | Split into config.ts + locales |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/config.ts` | CREATE | Main i18next configuration |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/i18n.d.ts` | CREATE | TypeScript augmentation |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/en/common.json` | CREATE | English common strings |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/en/landing.json` | CREATE | English landing page |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/en/trading.json` | CREATE | English trading terms |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/en/dashboard.json` | CREATE | English dashboard strings |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/en/errors.json` | CREATE | English error messages |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/ja/common.json` | CREATE | Japanese common strings |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/ja/landing.json` | CREATE | Japanese landing page |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/ja/trading.json` | CREATE | Japanese trading terms |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/ja/dashboard.json` | CREATE | Japanese dashboard strings |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/i18n/locales/ja/errors.json` | CREATE | Japanese error messages |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/contexts/LanguageContext.tsx` | CREATE | Language preference context |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/LanguageSelector.tsx` | MODIFY | Add JA, use LanguageContext |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/App.tsx` | MODIFY | Wrap with LanguageProvider |

---

## Implementation Steps

### Step 1: Install Language Detector

```bash
cd nextjs-ui-dashboard
npm install i18next-browser-languagedetector
```

### Step 2: Create Namespace JSON Files

**en/common.json** - Shared UI elements:
```json
{
  "button": {
    "save": "Save",
    "cancel": "Cancel",
    "delete": "Delete",
    "confirm": "Confirm",
    "close": "Close",
    "submit": "Submit",
    "loading": "Loading..."
  },
  "label": {
    "language": "Language",
    "theme": "Theme",
    "settings": "Settings",
    "profile": "Profile",
    "logout": "Log Out"
  },
  "nav": {
    "dashboard": "Dashboard",
    "trading": "Trading",
    "portfolio": "Portfolio",
    "signals": "AI Signals",
    "settings": "Settings"
  },
  "status": {
    "active": "Active",
    "inactive": "Inactive",
    "pending": "Pending",
    "completed": "Completed",
    "failed": "Failed"
  }
}
```

**en/trading.json** - Trading-specific:
```json
{
  "mode": {
    "paper": "Paper Trading",
    "real": "Real Trading",
    "paperDescription": "Simulate trades without capital",
    "realDescription": "Live trading with real funds"
  },
  "signal": {
    "buy": "Buy Signal",
    "sell": "Sell Signal",
    "hold": "Hold",
    "strength": "Signal Strength: {{value}}%"
  },
  "order": {
    "market": "Market Order",
    "limit": "Limit Order",
    "stopLoss": "Stop Loss",
    "takeProfit": "Take Profit"
  },
  "position": {
    "long": "Long",
    "short": "Short",
    "size": "Position Size",
    "leverage": "Leverage"
  },
  "portfolio": {
    "balance": "Balance",
    "equity": "Equity",
    "margin": "Margin",
    "pnl": "P&L",
    "return": "Return ({{currency}})"
  }
}
```

**en/dashboard.json** - Dashboard page:
```json
{
  "title": "Dashboard",
  "welcome": "Welcome back, {{name}}",
  "overview": "Overview",
  "performance": "Performance",
  "recentTrades": "Recent Trades",
  "marketSummary": "Market Summary",
  "aiInsights": "AI Insights",
  "stats": {
    "totalProfit": "Total Profit",
    "winRate": "Win Rate",
    "totalTrades": "Total Trades",
    "activeTrades": "Active Trades"
  }
}
```

**en/errors.json** - Error messages:
```json
{
  "generic": "Something went wrong. Please try again.",
  "network": "Network error. Check your connection.",
  "unauthorized": "Session expired. Please log in again.",
  "notFound": "Page not found.",
  "validation": {
    "required": "This field is required",
    "email": "Invalid email address",
    "password": "Password must be at least 8 characters"
  },
  "trading": {
    "insufficientBalance": "Insufficient balance",
    "orderFailed": "Order failed. Please try again.",
    "connectionLost": "Trading connection lost"
  }
}
```

### Step 3: Create i18n config.ts

```typescript
// src/i18n/config.ts
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Vite glob imports for all locales
const localeModules = import.meta.glob('./locales/**/*.json', { eager: true });

// Transform glob imports to i18next resources format
function loadResources() {
  const resources: Record<string, Record<string, unknown>> = {};

  Object.entries(localeModules).forEach(([path, module]) => {
    // Extract language and namespace from path
    // ./locales/en/common.json -> lang: 'en', ns: 'common'
    const match = path.match(/\.\/locales\/(\w+)\/(\w+)\.json$/);
    if (match) {
      const [, lang, ns] = match;
      if (!resources[lang]) resources[lang] = {};
      resources[lang][ns] = (module as { default: unknown }).default;
    }
  });

  return resources;
}

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: loadResources(),
    fallbackLng: 'en',
    defaultNS: 'common',
    ns: ['common', 'landing', 'trading', 'dashboard', 'errors'],
    interpolation: {
      escapeValue: false,
    },
    detection: {
      order: ['localStorage', 'navigator'],
      caches: ['localStorage'],
      lookupLocalStorage: 'preferredLanguage',
    },
  });

export default i18n;
```

### Step 4: Create TypeScript Augmentation

```typescript
// src/i18n/i18n.d.ts
import 'react-i18next';

import common from './locales/en/common.json';
import landing from './locales/en/landing.json';
import trading from './locales/en/trading.json';
import dashboard from './locales/en/dashboard.json';
import errors from './locales/en/errors.json';

declare module 'react-i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common';
    resources: {
      common: typeof common;
      landing: typeof landing;
      trading: typeof trading;
      dashboard: typeof dashboard;
      errors: typeof errors;
    };
  }
}
```

### Step 5: Create LanguageContext

```typescript
// src/contexts/LanguageContext.tsx
import React, { createContext, useContext, useCallback, useMemo, useState } from 'react';
import { useTranslation } from 'react-i18next';

interface LanguageContextType {
  language: string;
  setLanguage: (lang: string) => Promise<void>;
  isChanging: boolean;
  supportedLanguages: { code: string; name: string; flag: string }[];
}

const supportedLanguages = [
  { code: 'en', name: 'English', flag: '🇺🇸' },
  { code: 'vi', name: 'Tiếng Việt', flag: '🇻🇳' },
  { code: 'fr', name: 'Français', flag: '🇫🇷' },
  { code: 'zh', name: '中文', flag: '🇨🇳' },
  { code: 'ja', name: '日本語', flag: '🇯🇵' },
];

const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const { i18n } = useTranslation();
  const [isChanging, setIsChanging] = useState(false);

  const setLanguage = useCallback(async (lang: string) => {
    setIsChanging(true);
    try {
      localStorage.setItem('preferredLanguage', lang);
      await i18n.changeLanguage(lang);
    } finally {
      setIsChanging(false);
    }
  }, [i18n]);

  const value = useMemo(() => ({
    language: i18n.language,
    setLanguage,
    isChanging,
    supportedLanguages,
  }), [i18n.language, setLanguage, isChanging]);

  return (
    <LanguageContext.Provider value={value}>
      {children}
    </LanguageContext.Provider>
  );
}

export function useLanguage() {
  const ctx = useContext(LanguageContext);
  if (!ctx) throw new Error('useLanguage must be used within LanguageProvider');
  return ctx;
}
```

### Step 6: Update LanguageSelector Component

```typescript
// Modify src/components/LanguageSelector.tsx
// - Replace hardcoded languages array with useLanguage().supportedLanguages
// - Use setLanguage from context instead of direct i18n.changeLanguage
// - Add loading state during language change
```

### Step 7: Create Japanese Translations

Create all 5 JSON files in `src/i18n/locales/ja/`:
- common.json
- landing.json
- trading.json
- dashboard.json
- errors.json

**Example ja/common.json**:
```json
{
  "button": {
    "save": "保存",
    "cancel": "キャンセル",
    "delete": "削除",
    "confirm": "確認",
    "close": "閉じる",
    "submit": "送信",
    "loading": "読み込み中..."
  },
  "label": {
    "language": "言語",
    "theme": "テーマ",
    "settings": "設定",
    "profile": "プロフィール",
    "logout": "ログアウト"
  }
}
```

### Step 8: Update App.tsx Provider Hierarchy

```typescript
// Add LanguageProvider around other providers
import { LanguageProvider } from "@/contexts/LanguageContext";

<ThemeProvider>
  <LanguageProvider>
    <AuthProvider>
      ...
    </AuthProvider>
  </LanguageProvider>
</ThemeProvider>
```

---

## Translation Key Naming Convention

| Pattern | Example | Use |
|---------|---------|-----|
| `{namespace}:{section}.{key}` | `trading:signal.buy` | Specific context |
| `{namespace}:{key}` | `common:save` | Simple keys |
| Interpolation: `{{variable}}` | `Hello, {{name}}` | Dynamic values |
| Plural: `_one`, `_other` | `item_one`, `item_other` | Pluralization |

---

## Todo List

- [x] Install `i18next-browser-languagedetector`
- [x] Create `src/i18n/locales/` folder structure
- [x] Extract existing en translations to `en/landing.json`
- [x] Create `en/common.json` with shared UI strings
- [x] Create `en/trading.json` with trading-specific strings
- [x] Create `en/dashboard.json` with dashboard strings
- [x] Create `en/errors.json` with error messages
- [x] Translate and create all 5 `vi/*.json` files
- [x] Translate and create all 5 `fr/*.json` files
- [x] Translate and create all 5 `zh/*.json` files
- [x] Translate and create all 5 `ja/*.json` files (NEW)
- [x] Create `src/i18n/config.ts` with glob imports
- [x] Create `src/i18n/i18n.d.ts` for TypeScript
- [x] Create `src/contexts/LanguageContext.tsx`
- [x] Update `src/components/LanguageSelector.tsx`
- [x] Add LanguageProvider to App.tsx
- [x] Update `src/i18n/index.ts` to re-export config
- [x] Verify language detection works (browser preference)
- [x] Verify localStorage persistence works

---

## Success Criteria

- [x] 5 languages available in selector (EN, VI, FR, ZH, JA)
- [x] Language persists after page reload
- [x] Browser language auto-detected on first visit
- [x] TypeScript autocomplete works for translation keys
- [x] Fallback to English for missing keys
- [x] No runtime errors when switching languages
- [x] Bundle size increase < 20KB per language

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Missing translations | High | Medium | Fallback to EN, add warning in dev |
| Glob import issues | Medium | High | Test with Vite, verify paths |
| TypeScript errors | Medium | Low | Ensure d.ts file correct |
| Large bundle size | Low | Medium | Use namespace lazy loading |
| RTL issues | N/A | N/A | No RTL languages (no Arabic/Hebrew) |

---

## Security Considerations

- **No user input** in translation keys - prevents injection
- **localStorage** only stores language code string ('en', 'vi', etc.)
- **No sensitive data** in translation files

---

## Test Cases

| ID | Test Case | Expected Result |
|----|-----------|-----------------|
| TC-01 | First visit (no localStorage) | Detect browser language or fallback to EN |
| TC-02 | Select Japanese, reload page | Japanese persists |
| TC-03 | Missing key in VI | Shows English fallback |
| TC-04 | Interpolation `{{name}}` | Name replaced correctly |
| TC-05 | Switch lang during page session | All visible text updates |
| TC-06 | TypeScript autocomplete | Shows available keys |
| TC-07 | Console check | No i18next warnings for missing keys |

---

## Next Steps

After this phase:
1. [Phase 04: Apply Translations](./phase-04-apply-translations.md) - Apply t() to all pages
