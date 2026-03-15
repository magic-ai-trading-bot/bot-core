# UI Theme + Multi-language Implementation Plan

**Date**: 2025-12-09 | **Status**: Complete | **Est**: 4-6 days

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

| Phase | Name | Status | Link |
|-------|------|--------|------|
| 1 | Theme Infrastructure | ✅ Done | [phase-01](./phase-01-theme-infrastructure.md) |
| 2 | Light Mode Design | ✅ Done | [phase-02](./phase-02-light-mode-design.md) |
| 3 | i18n Restructure | ✅ Done | [phase-03](./phase-03-i18n-restructure.md) |
| 4 | Apply Translations | ✅ Done | [phase-04](./phase-04-apply-translations.md) |
| 5 | Theme Toggle UI | ✅ Done | [phase-05](./phase-05-theme-toggle-ui.md) |
| 6 | Testing & Polish | ✅ Done | [phase-06](./phase-06-testing-polish.md) |

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
- [x] Theme toggle works (light/dark/system) with smooth 0.3s transition
- [x] No FOUC on page reload
- [x] 5 languages (EN/VI/FR/ZH/JA) with namespace-based loading
- [x] All 22 pages + 30 components use `t()` function
- [x] Preferences persist across sessions (localStorage)
- [x] WCAG AA contrast maintained in both themes
- [x] All tests pass (unit + integration)

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
