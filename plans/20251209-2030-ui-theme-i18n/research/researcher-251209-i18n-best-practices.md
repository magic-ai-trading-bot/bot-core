# Research Report: React 18 i18n Best Practices (EN, VI, FR, ZH, JP)

**Date**: 2025-12-09
**Scope**: Multi-language i18n architecture for trading dashboard
**Sources Consulted**: 5 primary searches across official docs, community resources, performance analysis

---

## Executive Summary

**Recommendation: Use react-i18next** (22KB total: i18next 15KB + react-i18next 7KB) over react-intl (17KB) for your trading dashboard.

Key advantages: native lazy loading, namespace-based organization perfect for feature modules, superior TypeScript support, dynamic language switching without full reload, and plugin ecosystem. While react-intl is heavier in some respects, react-i18next's flexibility and trading-specific optimizations (currency/number formatting via Intl API) make it ideal.

Trade-offs: Simpler ICU Message Format (react-intl) vs. cleaner i18next syntax that's less prone to errors.

---

## Library Comparison: react-i18next vs react-intl

| Feature | react-i18next | react-intl |
|---------|---------------|-----------|
| **Bundle Size** | 22KB (i18next 15 + hook 7) | 17KB |
| **Message Format** | Custom (cleaner) | ICU standard (CLDR) |
| **Lazy Loading** | Native + plugins | Manual setup |
| **Dynamic Switching** | Yes, no reload | Component-level |
| **Namespace Support** | Built-in (best-in-class) | Not included |
| **TypeScript** | Strict typing + generators | Basic (self-managed) |
| **Popularity** | 1.5M+ weekly downloads | 200K+ weekly downloads |
| **Best For** | React apps, trading UIs | Large enterprises (Yahoo, Dropbox, Mozilla) |

**Decision**: Use **react-i18next** for performance, flexibility, and trading context suitability.

---

## Recommended Folder Structure

```
src/
├── i18n/
│   ├── config.ts                    # Main i18next initialization
│   ├── locales/
│   │   ├── en/
│   │   │   ├── common.json          # Shared UI (buttons, labels)
│   │   │   ├── trading.json         # Trading-specific terms
│   │   │   ├── dashboard.json       # Dashboard components
│   │   │   ├── errors.json          # Error messages
│   │   │   └── numbers.json         # Number/currency formats
│   │   ├── vi/
│   │   ├── fr/
│   │   ├── zh/                      # Chinese Simplified (zh-Hans or zh-CN)
│   │   └── ja/
│   ├── i18n.d.ts                    # TypeScript augmentation
│   └── hooks/
│       └── useTranslation.ts         # Custom hook wrapper (optional)
├── contexts/
│   └── LanguageContext.tsx           # Language preference + persistence
└── hooks/
    └── useLanguageDetection.ts       # Browser language detection
```

**Rationale**:
- **Namespaces** by feature (trading, dashboard, common) reduce initial load
- **i18n.d.ts** enables TypeScript autocomplete for translation keys
- **LanguageContext** centralizes preference logic (localStorage persistence)
- **Separate JSON files** for dates/numbers vs. strings improves maintainability

---

## Code Patterns

### 1. i18n Configuration (config.ts)

```typescript
import i18n from 'i18next';
import { initReactI18next } from 'react-i18next';
import LanguageDetector from 'i18next-browser-languagedetector';

// Vite glob imports for automatic resource discovery
const enResources = import.meta.glob<{ default: Record<string, string> }>(
  './locales/en/**/*.json',
  { eager: true }
);

const viResources = import.meta.glob<{ default: Record<string, string> }>(
  './locales/vi/**/*.json',
  { eager: true }
);

// Transform Vite glob to i18next format
const transformResources = (modules: any) => {
  return Object.keys(modules).reduce((acc, path) => {
    const namespace = path.split('/').pop()?.replace('.json', '') || 'default';
    acc[namespace] = modules[path].default;
    return acc;
  }, {});
};

i18n
  .use(LanguageDetector)
  .use(initReactI18next)
  .init({
    resources: {
      en: transformResources(enResources),
      vi: transformResources(viResources),
      fr: transformResources(frResources),
      zh: transformResources(zhResources),
      ja: transformResources(jaResources),
    },
    fallbackLng: 'en',
    defaultNS: 'common',
    interpolation: { escapeValue: false },
  });

export default i18n;
```

### 2. TypeScript Augmentation (i18n.d.ts)

```typescript
import 'react-i18next';
import en_common from './locales/en/common.json';
import en_trading from './locales/en/trading.json';
import en_errors from './locales/en/errors.json';

type CommonResources = typeof en_common;
type TradingResources = typeof en_trading;
type ErrorResources = typeof en_errors;

declare module 'react-i18next' {
  interface CustomTypeOptions {
    defaultNS: 'common';
    resources: {
      common: CommonResources;
      trading: TradingResources;
      errors: ErrorResources;
    };
  }
}
```

### 3. Language Context (LanguageContext.tsx)

```typescript
import React, { createContext, useCallback } from 'react';
import { useTranslation } from 'react-i18next';

interface LanguageContextType {
  language: string;
  setLanguage: (lang: string) => Promise<void>;
  isLoading: boolean;
}

export const LanguageContext = createContext<LanguageContextType | undefined>(undefined);

export function LanguageProvider({ children }: { children: React.ReactNode }) {
  const { i18n } = useTranslation();
  const [isLoading, setIsLoading] = React.useState(false);

  const setLanguage = useCallback(async (lang: string) => {
    setIsLoading(true);
    try {
      // Persist to localStorage
      localStorage.setItem('preferredLanguage', lang);
      await i18n.changeLanguage(lang);
    } finally {
      setIsLoading(false);
    }
  }, [i18n]);

  return (
    <LanguageContext.Provider value={{
      language: i18n.language,
      setLanguage,
      isLoading,
    }}>
      {children}
    </LanguageContext.Provider>
  );
}
```

### 4. Component Usage (useTranslation Hook)

```typescript
import { useTranslation } from 'react-i18next';

export function TradingCard() {
  const { t } = useTranslation(['trading', 'common']);

  return (
    <div>
      <h2>{t('card.title', { ns: 'trading' })}</h2>
      <button>{t('button.save', { ns: 'common' })}</button>
    </div>
  );
}
```

### 5. Number/Currency Formatting (Trading Context)

```typescript
import { useTranslation } from 'react-i18next';

export function CurrencyDisplay({ amount }: { amount: number }) {
  const { i18n } = useTranslation();

  const formatter = new Intl.NumberFormat(i18n.language, {
    style: 'currency',
    currency: 'USD',
    minimumFractionDigits: 2,
    maximumFractionDigits: 2,
  });

  return <span>{formatter.format(amount)}</span>;
}

export function DateDisplay({ date }: { date: Date }) {
  const { i18n } = useTranslation();

  return date.toLocaleDateString(i18n.language, {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
```

---

## Translation File Structure (JSON)

### en/common.json
```json
{
  "button": {
    "save": "Save",
    "cancel": "Cancel",
    "delete": "Delete"
  },
  "label": {
    "language": "Language",
    "theme": "Theme"
  }
}
```

### en/trading.json
```json
{
  "card": {
    "title": "Paper Trading",
    "subtitle": "Simulate trades without capital"
  },
  "signal": {
    "buy": "Buy Signal",
    "sell": "Sell Signal",
    "strength": "Signal Strength: {{value}}%"
  },
  "portfolio": {
    "balance": "Balance",
    "return": "Return ({{currency}})"
  }
}
```

---

## Performance Optimization: Lazy Loading

### Namespace-based Loading

```typescript
// Only load 'common' and 'trading' initially
i18n.init({
  ns: ['common', 'trading'],
  defaultNS: 'common',
  backend: {
    loadPath: '/locales/{{lng}}/{{ns}}.json'
  },
  react: {
    useSuspense: false // Optional: disable for gradual loading
  }
});

// Lazy load 'errors' namespace on demand
export function ErrorPage() {
  const { t } = useTranslation(['errors'], {
    useSuspense: true
  });

  return <div>{t('not_found')}</div>;
}
```

### Benefits
- **Initial load**: Only 2-3 namespaces (5-10KB vs 40KB)
- **Code splitting**: Error messages load only when needed
- **Memory efficient**: Unused translations never loaded

---

## Language Codes for Your Target Languages

| Language | ISO 639-1 | ISO 639-3 | Notes |
|----------|-----------|-----------|-------|
| English | `en` | `eng` | Standard |
| Vietnamese | `vi` | `vie` | Use `vi` |
| French | `fr` | `fra` | Use `fr` (or `fr-FR` for regional) |
| Chinese | `zh-CN` or `zh-Hans` | `zho` | Simplified (not traditional `zh-Hant`) |
| Japanese | `ja` | `jpn` | Use `ja` |

---

## Key Challenges & Solutions

| Challenge | Impact | Solution |
|-----------|--------|----------|
| **RTL languages** | Not applicable (no Arabic/Hebrew) | Document for future; use `dir` attribute if needed |
| **Pluralization** | Complex in Japanese/Chinese | Use i18next plural rules + Intl.PluralRules |
| **Number formatting** | Critical for trading | Use `Intl.NumberFormat` (zero-dependency native API) |
| **Bundle size for 5 langs** | ~50-60KB base + translation data | Lazy load namespaces; gzip reduces to ~15KB |
| **TypeScript strict typing** | DX issue | Use i18next-scanner + type augmentation |
| **Locale data for dates** | Browser handles natively | Use `toLocaleDateString()` with i18n.language |

---

## Installation & Quick Start

```bash
# Install dependencies
npm install i18next react-i18next i18next-browser-languagedetector

# Optional: For HTTP backend loading
npm install i18next-http-backend

# Optional: Type safety tooling
npm install --save-dev i18next-scanner
```

```typescript
// main.tsx
import i18n from './i18n/config';
import { LanguageProvider } from './contexts/LanguageContext';
import App from './App';

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <LanguageProvider>
      <App />
    </LanguageProvider>
  </React.StrictMode>,
);
```

---

## Unresolved Questions

1. **CMS Integration**: Will you use Phrase/Crowdin for translation management, or Git-based workflow? (Affects backend setup)
2. **Pseudo-translation**: Need process for QA testing all languages? (i18next-pseudo-backend recommended)
3. **Right-to-Left future**: Plan support for Arabic/Hebrew in future roadmap? (Requires dir attribute management)
4. **Pluralization rules**: Japanese doesn't use plurals—how to handle "1 trade" vs "3 trades"? (Document pluralization policy)

---

## References

- [react-i18next vs react-intl: The Ultimate Comparison](https://www.locize.com/blog/react-intl-vs-react-i18next)
- [Multiple Translation Files | react-i18next docs](https://react.i18next.com/guides/multiple-translation-files)
- [TypeScript Support | i18next docs](https://www.i18next.com/overview/typescript)
- [A Guide to React Localization with i18next](https://phrase.com/blog/posts/localizing-react-apps-with-i18next/)
- [Lazy Loading Translations in React](https://localizely.com/i18n-questions/react/how-to-do-lazy-loading-of-translations-in-react/)
- [Intl.NumberFormat API | MDN](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat)
- [react-i18next TypeScript Support](https://react.i18next.com/latest/typescript)
- [Simplify Currency Formatting with Intl API](https://dev.to/josephciullo/simplify-currency-formatting-in-react-a-zero-dependency-solution-with-intl-api-3kok)

---

**Next Steps**:
1. Set up i18n config with 5 language files
2. Create TypeScript augmentation (i18n.d.ts)
3. Implement LanguageContext for persistence
4. Extract trading-specific terms into namespace
5. Test with Vietnamese (CJK character handling) and Japanese (complex pluralization)
