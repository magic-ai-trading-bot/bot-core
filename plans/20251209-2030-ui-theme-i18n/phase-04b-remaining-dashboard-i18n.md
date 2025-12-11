# Phase 04B: Remaining Dashboard Pages i18n

**Priority**: High | **Status**: In Progress | **Est. Effort**: 4 hours

---

## Context Links

- [Main Plan](./plan.md)
- [Phase 04: Apply Translations](./phase-04-apply-translations.md)
- Continues from previous session work

---

## Overview

Complete i18n for remaining 8 pages that don't have `useTranslation()`.

---

## Current State Analysis

### ✅ COMPLETED (20 pages)
| Page | Namespace | Status |
|------|-----------|--------|
| Features.tsx | pages | ✅ |
| Pricing.tsx | pages | ✅ |
| About.tsx | pages | ✅ |
| Blog.tsx | pages | ✅ |
| Careers.tsx | pages | ✅ |
| Contact.tsx | pages | ✅ |
| Privacy.tsx | pages | ✅ |
| Terms.tsx | pages | ✅ |
| SecurityPage.tsx | pages | ✅ |
| Compliance.tsx | pages | ✅ |
| API.tsx | pages | ✅ |
| Documentation.tsx | pages | ✅ |
| Login.tsx | auth | ✅ |
| Register.tsx | auth | ✅ |
| Dashboard.tsx | dashboard | ✅ |
| Settings.tsx | settings | ✅ |
| PaperTrading.tsx | trading | ✅ |
| TradingPaper.tsx | trading | ✅ |
| Index.tsx | landing | ✅ |

### ❌ REMAINING (8 pages - NO useTranslation)
| Page | Lines | Est. Strings | Target Namespace |
|------|-------|--------------|------------------|
| Portfolio.tsx | 993 | ~40 | dashboard |
| AISignals.tsx | 1069 | ~50 | dashboard |
| TradeAnalyses.tsx | 1005 | ~60 | dashboard |
| Profile.tsx | 744 | ~35 | dashboard |
| RealTrading.tsx | 2000+ | ~80 | dashboard |
| HowItWorks.tsx | 656 | ~70 | pages |
| NotFound.tsx | ~100 | ~5 | errors |
| Error.tsx | ~100 | ~5 | errors |

---

## Translation Keys Structure

### dashboard.json additions
```json
{
  "portfolio": {
    "header": { "title": "", "subtitle": "" },
    "stats": { "balance": "", "winRate": "", "totalTrades": "", "profitFactor": "", "totalPnl": "" },
    "sections": { "allocation": "", "recentTrades": "", "performance": "" },
    "table": { "symbol": "", "type": "", "entry": "", "exit": "", "pnl": "", "date": "" },
    "empty": { "noPositions": "", "startTrading": "" },
    "metrics": { "bestTrade": "", "worstTrade": "", "maxDrawdown": "", "sharpeRatio": "" }
  },
  "aiSignals": {
    "header": { "title": "", "subtitle": "" },
    "engine": { "status": "", "active": "", "offline": "", "description": "" },
    "accuracy": { "label": "", "value": "" },
    "signals": { "live": "", "active": "", "waiting": "", "waitingDesc": "" },
    "history": { "title": "", "all": "", "win": "", "loss": "" },
    "stats": { "wins": "", "losses": "", "winRate": "", "totalPnl": "" },
    "card": { "confidence": "", "entry": "", "target": "", "stopLoss": "" },
    "actions": { "viewAnalysis": "", "aiReasoning": "", "strategyScores": "" }
  },
  "tradeAnalyses": {
    "header": { "title": "", "subtitle": "" },
    "stats": { "total": "", "losing": "", "winning": "", "totalPnl": "", "avgPnl": "" },
    "tabs": { "analyses": "", "suggestions": "" },
    "filter": { "losingOnly": "", "allTrades": "" },
    "empty": { "noAnalyses": "", "analysesDesc": "" },
    "suggestion": { "latest": "", "rootCause": "", "applied": "" },
    "analysis": { "verdict": "", "summary": "", "whatWentWrong": "", "keyFactors": "", "lessons": "", "recommendations": "" },
    "details": { "entry": "", "exit": "", "tradeId": "", "closeReason": "" }
  },
  "profile": {
    "header": { "title": "", "subtitle": "" },
    "stats": { "totalTrades": "", "winRate": "", "totalPnl": "", "bestTrade": "" },
    "metrics": { "avgProfit": "", "avgLoss": "", "profitFactor": "", "sharpeRatio": "" },
    "achievements": { "title": "", "subtitle": "", "firstTrade": "", "tenTrades": "", "inProfit": "", "winningStreak": "" },
    "activity": { "title": "", "subtitle": "", "noActivity": "", "startTrading": "" },
    "memberSince": ""
  },
  "realTrading": {
    "header": { "title": "", "subtitle": "" },
    "warning": { "title": "", "message": "", "confirm": "" },
    "orderBook": { "title": "", "price": "", "amount": "", "total": "" },
    "orderForm": { "buy": "", "sell": "", "market": "", "limit": "", "amount": "", "price": "", "total": "", "submit": "" },
    "positions": { "title": "", "symbol": "", "side": "", "size": "", "entry": "", "pnl": "", "close": "" },
    "history": { "title": "", "time": "", "type": "", "filled": "", "status": "" }
  }
}
```

### pages.json additions (HowItWorks)
```json
{
  "howItWorks": {
    "badge": "",
    "hero": { "title": "", "titleHighlight": "", "subtitle": "" },
    "stats": { "strategies": "", "riskLayers": "", "uptime": "", "aiAccuracy": "" },
    "steps": {
      "title": "",
      "subtitle": "",
      "dataCollection": { "title": "", "subtitle": "", "desc": "", "details": [] },
      "technicalAnalysis": { "title": "", "subtitle": "", "desc": "", "details": [] },
      "signalGeneration": { "title": "", "subtitle": "", "desc": "", "details": [] },
      "riskManagement": { "title": "", "subtitle": "", "desc": "", "details": [] }
    },
    "strategies": { "title": "", "subtitle": "", "items": [], "note": "" },
    "risk": { "title": "", "subtitle": "", "layers": [] },
    "trailingStop": { "title": "", "subtitle": "", "result": "" }
  }
}
```

### errors.json additions
```json
{
  "notFound": {
    "title": "",
    "description": "",
    "backHome": ""
  },
  "error": {
    "title": "",
    "description": "",
    "retry": "",
    "goBack": ""
  }
}
```

---

## Implementation Steps

### Step 1: Update Translation Files
1. Add keys to `en/dashboard.json`
2. Add Vietnamese to `vi/dashboard.json`
3. Add keys to `en/pages.json` for HowItWorks
4. Add Vietnamese to `vi/pages.json`
5. Add keys to `en/errors.json`
6. Add Vietnamese to `vi/errors.json`
7. Copy EN structure to fr/ja/zh with same English values (placeholder)

### Step 2: Implement Portfolio.tsx i18n
1. Add `import { useTranslation } from 'react-i18next';`
2. Add `const { t } = useTranslation('dashboard');`
3. Replace all hardcoded strings with `t('portfolio.xxx')`

### Step 3: Implement AISignals.tsx i18n
Same pattern as Step 2

### Step 4: Implement TradeAnalyses.tsx i18n
Same pattern as Step 2

### Step 5: Implement Profile.tsx i18n
Same pattern as Step 2

### Step 6: Implement RealTrading.tsx i18n
Same pattern as Step 2 (largest file, most strings)

### Step 7: Implement HowItWorks.tsx i18n
1. Add `const { t } = useTranslation('pages');`
2. Replace with `t('howItWorks.xxx')`

### Step 8: Implement NotFound.tsx & Error.tsx i18n
1. Add `const { t } = useTranslation('errors');`
2. Replace with `t('notFound.xxx')` / `t('error.xxx')`

---

## Todo List

- [ ] Update en/dashboard.json with portfolio, aiSignals, tradeAnalyses, profile, realTrading keys
- [ ] Update vi/dashboard.json with Vietnamese translations
- [ ] Update en/pages.json with howItWorks keys
- [ ] Update vi/pages.json with Vietnamese translations
- [ ] Update en/errors.json with notFound, error keys
- [ ] Update vi/errors.json with Vietnamese translations
- [ ] Portfolio.tsx - add useTranslation('dashboard')
- [ ] AISignals.tsx - add useTranslation('dashboard')
- [ ] TradeAnalyses.tsx - add useTranslation('dashboard')
- [ ] Profile.tsx - add useTranslation('dashboard')
- [ ] RealTrading.tsx - add useTranslation('dashboard')
- [ ] HowItWorks.tsx - add useTranslation('pages')
- [ ] NotFound.tsx - add useTranslation('errors')
- [ ] Error.tsx - add useTranslation('errors')
- [ ] Test all pages with Vietnamese locale

---

## Success Criteria

- [ ] All 8 pages have `useTranslation()` hook
- [ ] No hardcoded English text visible in UI
- [ ] Vietnamese translations display correctly
- [ ] No console warnings for missing keys
- [ ] TypeScript compiles without errors

---

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|-----------|
| Large file changes (RealTrading 2000+ lines) | High | Careful incremental changes |
| Missing translation keys | Medium | Verify with test script |
| Breaking existing functionality | High | Test each page after changes |

---

## Next Steps

After completion:
1. Run comprehensive i18n test script
2. Verify all 28 pages work with Vietnamese
3. Commit changes
