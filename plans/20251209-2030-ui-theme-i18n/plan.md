# UI Theme + Multi-language Implementation Plan

**Date**: 2025-12-09 | **Status**: Ready | **Est**: 4-6 days

## Context
- [Research: Theming System](./research/researcher-251209-theming-system.md)
- [Research: i18n Best Practices](./research/researcher-251209-i18n-best-practices.md)
- [Scout: Frontend Analysis](./scout/scout-01-frontend-analysis.md)

## Scope
| Feature | Current State | Target State |
|---------|--------------|--------------|
| Theme | Dark-only, no toggle, no context | Light + Dark + System, ThemeProvider, FOUC prevention |
| Languages | 4 (EN/VI/FR/ZH), landing only | 5 (+JA), namespace-based, all 22 pages + 30 components |
| Persistence | None | localStorage for both theme + language |

## Phase Overview

| Phase | File | Priority | Est | Status |
|-------|------|----------|-----|--------|
| 1 | [phase-01-theme-infrastructure.md](./phase-01-theme-infrastructure.md) | P0 | 4h | ✅ Done |
| 2 | [phase-02-light-mode-design.md](./phase-02-light-mode-design.md) | P0 | 3h | ✅ Done |
| 3 | [phase-03-i18n-restructure.md](./phase-03-i18n-restructure.md) | P0 | 6h | ✅ Done |
| 4 | [phase-04-apply-translations.md](./phase-04-apply-translations.md) | P1 | 8h | 🔄 Partial |
| 4b | [phase-04b-remaining-dashboard-i18n.md](./phase-04b-remaining-dashboard-i18n.md) | P0 | 4h | 🔄 In Progress |
| 5 | [phase-05-theme-toggle-ui.md](./phase-05-theme-toggle-ui.md) | P1 | 2h | ✅ Done |
| 6 | [phase-06-testing-polish.md](./phase-06-testing-polish.md) | P1 | 4h | ⏳ Pending |

## Dependencies
```
Phase 1 ─┬─> Phase 2 ─┬─> Phase 5 ─┐
         │            │            │
Phase 3 ─┴─> Phase 4 ─┴────────────├─> Phase 6
```

## Key Decisions
1. **Theme**: Class-based (`dark` on `<html>`), system preference, localStorage
2. **i18n**: Namespace-based (common/trading/dashboard/errors), lazy load
3. **FOUC**: Inline `<head>` script before React mounts
4. **Transitions**: 0.3s ease + prefers-reduced-motion support

## Files Summary
| Action | Count | Examples |
|--------|-------|----------|
| CREATE | ~25 | ThemeContext, LanguageContext, 20 translation JSONs |
| MODIFY | ~55 | 22 pages, 30+ components, index.css, index.html |

## Dependencies to Install
```bash
npm install i18next-browser-languagedetector
```

## Success Criteria
- [ ] Theme toggle works (light/dark/system) with smooth 0.3s transition
- [ ] No FOUC on page reload
- [ ] 5 languages (EN/VI/FR/ZH/JA) with namespace-based loading
- [ ] All 22 pages + 30 components use `t()` function
- [ ] Preferences persist across sessions (localStorage)
- [ ] WCAG AA contrast maintained in both themes
- [ ] All tests pass (unit + integration)

## Risk Summary
| Risk | Impact | Mitigation |
|------|--------|-----------|
| FOUC on reload | High | `<head>` script, no React dependency |
| Missing translations | Medium | EN fallback, dev warnings |
| Contrast failures | High | Validate with WCAG tools before merge |
| Breaking existing UI | High | Test each page after modification |

## Execution Order
1. **Day 1**: Phase 1 + 2 (Theme infrastructure + light mode CSS)
2. **Day 2**: Phase 3 (i18n restructure, create JSON files)
3. **Day 3-4**: Phase 4 (Apply translations to all pages/components)
4. **Day 5**: Phase 5 + 6 (Toggle UI + testing/polish)
