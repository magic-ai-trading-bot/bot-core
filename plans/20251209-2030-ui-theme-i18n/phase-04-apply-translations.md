# Phase 04: Apply Translations to All Pages and Components

**Priority**: High | **Status**: Pending | **Est. Effort**: 8 hours

---

## Context Links

- [Main Plan](./plan.md)
- [Phase 03: i18n Restructure](./phase-03-i18n-restructure.md)
- [Scout Report](./scout/scout-01-frontend-analysis.md)

---

## Overview

Systematically apply `useTranslation()` hook and `t()` function to all 22 pages and 30+ key components. Replace hardcoded strings with translation keys. This is the most labor-intensive phase.

---

## Key Insights

1. **22 pages** total - 12 public, 10 protected
2. **30+ components** with user-visible text
3. **Shadcn/UI components** (42 files) - mostly no text, skip
4. **Current landing page** - Already has some t() calls
5. **Strategy** - Batch by page category, prioritize user-facing

---

## Requirements

| ID | Requirement | Priority |
|----|-------------|----------|
| R1 | All pages use t() for visible text | Critical |
| R2 | All error messages use t('errors:...') | Critical |
| R3 | Trading components use t('trading:...') | High |
| R4 | Dashboard components use t('dashboard:...') | High |
| R5 | Forms use t() for labels and placeholders | High |
| R6 | Navigation uses t('common:nav...') | High |
| R7 | Dates/numbers use Intl formatters | Medium |

---

## Architecture

**Translation Pattern**:
```typescript
import { useTranslation } from 'react-i18next';

export function MyComponent() {
  const { t } = useTranslation(['common', 'trading']); // Load needed namespaces

  return (
    <div>
      <h1>{t('trading:mode.paper')}</h1>
      <button>{t('common:button.save')}</button>
    </div>
  );
}
```

**Interpolation Pattern**:
```typescript
// Static: t('key')
// With variable: t('welcome', { name: user.name })
// Pluralization: t('items', { count: 5 })
```

---

## Related Code Files - Pages (22)

### Priority 1: Core Protected Pages (6)
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Dashboard.tsx` | dashboard, common | ~25 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/PaperTrading.tsx` | trading, common | ~30 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/RealTrading.tsx` | trading, common | ~30 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Portfolio.tsx` | trading, dashboard | ~20 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/AISignals.tsx` | trading, dashboard | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Settings.tsx` | common | ~20 |

### Priority 2: Auth & User Pages (3)
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Login.tsx` | common, errors | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Register.tsx` | common, errors | ~20 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Profile.tsx` | common | ~15 |

### Priority 3: Landing & Marketing Pages (7)
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Index.tsx` | landing | ~50 (partial) |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Features.tsx` | landing | ~30 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Pricing.tsx` | landing | ~40 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/About.tsx` | landing | ~25 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Contact.tsx` | common | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Blog.tsx` | landing | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Careers.tsx` | landing | ~20 |

### Priority 4: Legal & Info Pages (6)
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Privacy.tsx` | common | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Terms.tsx` | common | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/SecurityPage.tsx` | common | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Compliance.tsx` | common | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/API.tsx` | common | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Documentation.tsx` | common | ~20 |

### Priority 5: Utility Pages (3)
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/Error.tsx` | errors | ~5 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/NotFound.tsx` | errors | ~5 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/HowItWorks.tsx` | landing | ~20 |

---

## Related Code Files - Components (30+)

### Trading Components
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/TradingInterface.tsx` | trading | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/OrderForm.tsx` | trading | ~20 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/PortfolioStats.tsx` | trading | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/RiskMetrics.tsx` | trading | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/ClosedTradesTable.tsx` | trading | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/ModeToggle.tsx` | trading | ~5 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/ModeSwitchDialog.tsx` | trading | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/trading/RealModeWarningBanner.tsx` | trading | ~5 |

### Dashboard Components
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/BotStatus.tsx` | dashboard | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/PerformanceChart.tsx` | dashboard | ~5 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/PortfolioSummaryCard.tsx` | dashboard | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/QuickActionsBar.tsx` | common | ~5 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/SystemMonitoring.tsx` | dashboard | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/AISignalsNew.tsx` | trading | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/StrategyComparison.tsx` | trading | ~10 |

### AI Components
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ai/SignalCard.tsx` | trading | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ai/DetailedSignalDialog.tsx` | trading | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ai/StrategyExplanation.tsx` | trading | ~10 |

### Landing Components
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/Hero3D.tsx` | landing | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/FeaturesSection.tsx` | landing | ~15 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/TestimonialsSection.tsx` | landing | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/FAQSection.tsx` | landing | ~20 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/landing/LandingFooter.tsx` | landing | ~15 |

### Layout & Settings
| File | Namespace(s) | Est. Strings |
|------|-------------|--------------|
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/MainLayout.tsx` | common | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/layout/ModeIndicatorBadge.tsx` | trading | ~3 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/SettingsUI.tsx` | common | ~20 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/settings/SettingsTabs.tsx` | common | ~10 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/LanguageSelector.tsx` | common | ~5 |
| `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ProtectedRoute.tsx` | errors | ~3 |

---

## Implementation Steps

### Step 1: Create Comprehensive Translation Keys

Before modifying components, ensure all translation files have complete keys. Estimate ~400 total strings across all namespaces.

### Step 2: Apply Translations to Core Pages (Batch 1)

**Dashboard.tsx Example**:
```typescript
// Before
<h1>Dashboard</h1>
<p>Welcome back, {user.name}</p>

// After
import { useTranslation } from 'react-i18next';
const { t } = useTranslation('dashboard');
<h1>{t('title')}</h1>
<p>{t('welcome', { name: user.name })}</p>
```

### Step 3: Apply Translations to Trading Pages (Batch 2)

Focus on trading-specific terminology consistency:
- "Paper Trading" / "Real Trading"
- "Buy Signal" / "Sell Signal"
- "Stop Loss" / "Take Profit"
- "P&L" / "Balance" / "Equity"

### Step 4: Apply Translations to Auth Pages (Batch 3)

Include form validation messages:
```typescript
// errors.json
{
  "validation": {
    "required": "This field is required",
    "email": "Please enter a valid email",
    "passwordMin": "Password must be at least {{min}} characters"
  }
}
```

### Step 5: Apply Translations to Landing Pages (Batch 4)

Already partially done in Index.tsx - extend to Features, Pricing, About.

### Step 6: Apply Translations to Trading Components

Focus on components used in trading flow:
- OrderForm
- PortfolioStats
- RiskMetrics
- ClosedTradesTable

### Step 7: Apply Translations to Dashboard Components

Focus on dashboard widgets:
- BotStatus
- PerformanceChart
- PortfolioSummaryCard
- SystemMonitoring

### Step 8: Add Number/Currency Formatters

```typescript
// Use Intl.NumberFormat for currency
const formatCurrency = (amount: number, lang: string) => {
  return new Intl.NumberFormat(lang, {
    style: 'currency',
    currency: 'USD',
  }).format(amount);
};

// In component
const { i18n } = useTranslation();
<span>{formatCurrency(portfolio.balance, i18n.language)}</span>
```

### Step 9: Add Date Formatters

```typescript
// Use Intl.DateTimeFormat
const formatDate = (date: Date, lang: string) => {
  return new Intl.DateTimeFormat(lang, {
    dateStyle: 'medium',
    timeStyle: 'short',
  }).format(date);
};
```

---

## String Extraction Checklist

Per component, check and replace:
- [ ] Page titles
- [ ] Section headers
- [ ] Button labels
- [ ] Form labels and placeholders
- [ ] Error messages
- [ ] Success messages
- [ ] Table headers
- [ ] Empty states
- [ ] Loading states
- [ ] Tooltips
- [ ] Modal titles and content
- [ ] Navigation items
- [ ] Status indicators

---

## Todo List

### Pages
- [ ] Dashboard.tsx (~25 strings)
- [ ] PaperTrading.tsx (~30 strings)
- [ ] RealTrading.tsx (~30 strings)
- [ ] Portfolio.tsx (~20 strings)
- [ ] AISignals.tsx (~15 strings)
- [ ] Settings.tsx (~20 strings)
- [ ] Login.tsx (~15 strings)
- [ ] Register.tsx (~20 strings)
- [ ] Profile.tsx (~15 strings)
- [ ] Index.tsx (verify/complete ~50 strings)
- [ ] Features.tsx (~30 strings)
- [ ] Pricing.tsx (~40 strings)
- [ ] About.tsx (~25 strings)
- [ ] Contact.tsx (~15 strings)
- [ ] Blog.tsx (~10 strings)
- [ ] Careers.tsx (~20 strings)
- [ ] Privacy.tsx (~10 strings)
- [ ] Terms.tsx (~10 strings)
- [ ] SecurityPage.tsx (~15 strings)
- [ ] Compliance.tsx (~10 strings)
- [ ] Error.tsx (~5 strings)
- [ ] NotFound.tsx (~5 strings)

### Components
- [ ] TradingInterface.tsx
- [ ] OrderForm.tsx
- [ ] PortfolioStats.tsx
- [ ] RiskMetrics.tsx
- [ ] ClosedTradesTable.tsx
- [ ] ModeToggle.tsx
- [ ] ModeSwitchDialog.tsx
- [ ] BotStatus.tsx
- [ ] PerformanceChart.tsx
- [ ] PortfolioSummaryCard.tsx
- [ ] SystemMonitoring.tsx
- [ ] SignalCard.tsx
- [ ] DetailedSignalDialog.tsx
- [ ] Hero3D.tsx
- [ ] FeaturesSection.tsx
- [ ] FAQSection.tsx
- [ ] LandingFooter.tsx
- [ ] MainLayout.tsx
- [ ] SettingsUI.tsx
- [ ] LanguageSelector.tsx (update for JA)

---

## Success Criteria

- [ ] No hardcoded user-visible strings in pages
- [ ] All components use t() from useTranslation
- [ ] All 5 languages show correct translations
- [ ] Currency/number formatting respects locale
- [ ] Date formatting respects locale
- [ ] No console warnings for missing keys
- [ ] TypeScript compilation succeeds

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|-----------|--------|-----------|
| Missing translation keys | High | Medium | Run i18next-scanner to find missing |
| Incorrect translations (JA, ZH) | Medium | Medium | Review by native speaker or use professional service |
| Breaking existing functionality | Medium | High | Test each page after modification |
| Performance impact | Low | Low | Translations are cached, minimal overhead |

---

## Security Considerations

- **No user input in translation keys** - prevents key injection
- **Interpolation is safe** - react-i18next escapes by default
- **No sensitive data** in translation values

---

## Test Cases

| ID | Test Case | Expected Result |
|----|-----------|-----------------|
| TC-01 | View Dashboard in each language | All text translated, no keys visible |
| TC-02 | Submit form with invalid data | Error message in current language |
| TC-03 | View trading page currency values | Currency formatted per locale |
| TC-04 | View dates in Japanese | Date in Japanese format |
| TC-05 | Switch language mid-session | All visible text updates |
| TC-06 | Check browser console | No i18next missing key warnings |
| TC-07 | Run TypeScript compile | No translation key type errors |

---

## Next Steps

After this phase:
1. [Phase 05: Theme Toggle UI](./phase-05-theme-toggle-ui.md) - Create theme switcher component
