# Frontend Issues Tracker

**Generated:** 2025-11-19
**Total Issues:** 15
**Critical:** 0 | **High:** 5 | **Medium:** 5 | **Low:** 5

---

## Quick Reference Table

| # | Priority | Issue | File(s) | Effort | Status |
|---|----------|-------|---------|--------|--------|
| 1 | HIGH | Bundle size optimization (1.19MB Three.js) | `vite.config.ts` | 2h | 🔴 TODO |
| 2 | HIGH | TradingPaper.tsx too large (2055 lines) | `pages/TradingPaper.tsx` | 8-12h | 🔴 TODO |
| 3 | HIGH | Large dashboard components (>1000 lines) | `components/dashboard/*.tsx` | 6-10h each | 🔴 TODO |
| 4 | HIGH | Missing memoization in trade tables | `pages/TradingPaper.tsx` | 3-4h | 🔴 TODO |
| 5 | HIGH | WebSocket connection health monitoring | `hooks/useWebSocket.ts` | 2-3h | 🔴 TODO |
| 6 | MEDIUM | Hardcoded API URLs | `pages/TradingPaper.tsx` | 2-3h | 🟡 TODO |
| 7 | MEDIUM | Missing error boundaries on lazy components | `pages/Dashboard.tsx` | 1-2h | 🟡 TODO |
| 8 | MEDIUM | Magic numbers throughout | Multiple files | 2-3h | 🟡 TODO |
| 9 | MEDIUM | Duplicate formatting functions | `pages/TradingPaper.tsx` | 1-2h | 🟡 TODO |
| 10 | MEDIUM | Missing debouncing for rapid updates | `pages/TradingPaper.tsx` | 1h | 🟡 TODO |
| 11 | LOW | Missing ARIA labels | Various | 4-6h | ⚪ BACKLOG |
| 12 | LOW | Keyboard navigation improvements | Various | 2-3h | ⚪ BACKLOG |
| 13 | LOW | Mobile responsiveness edge cases | `pages/Settings.tsx` | 3-4h | ⚪ BACKLOG |
| 14 | LOW | Outdated dependencies (4 packages) | `package.json` | 15min | ⚪ BACKLOG |
| 15 | LOW | Virtualization for long lists | `pages/TradingPaper.tsx` | 3-4h | ⚪ BACKLOG |

---

## Priority Breakdown

### 🔴 HIGH Priority (5 issues - 21-30 hours)
Fix in next sprint. These impact performance, maintainability, and user experience.

### 🟡 MEDIUM Priority (5 issues - 7-11 hours)
Fix in next 2 sprints. These improve code quality and consistency.

### ⚪ LOW Priority (5 issues - 13-21 hours)
Backlog. These are nice-to-have improvements.

---

## Effort Summary

| Priority | Issues | Min Hours | Max Hours |
|----------|--------|-----------|-----------|
| HIGH | 5 | 21 | 30 |
| MEDIUM | 5 | 7 | 11 |
| LOW | 5 | 13 | 21 |
| **TOTAL** | **15** | **41** | **62** |

---

## Sprint Planning

### **Sprint 1 (Week 1-2)** - Critical Path
- [ ] #14: Update dependencies (15min) ✅ QUICK WIN
- [ ] #2: Split TradingPaper.tsx (8-12h)
- [ ] #4: Add memoization (3-4h)
- [ ] #6: Centralize API calls (2-3h)
- [ ] #8: Extract constants (2-3h)

**Total:** 16-23 hours

---

### **Sprint 2 (Week 3-4)** - Optimization
- [ ] #1: Bundle size optimization (2h)
- [ ] #3: Break down AISignals.tsx (6-10h)
- [ ] #3: Break down StrategyTuningSettings.tsx (6-10h)
- [ ] #7: Add error boundaries (1-2h)
- [ ] #9: Remove duplicate formatters (1-2h)

**Total:** 16-26 hours

---

### **Sprint 3 (Week 5-6)** - Enhancement
- [ ] #5: WebSocket health monitoring (2-3h)
- [ ] #10: Add debouncing (1h)
- [ ] #15: Add virtualization (3-4h)
- [ ] #11: ARIA labels (4-6h)
- [ ] #12: Keyboard navigation (2-3h)

**Total:** 12-17 hours

---

## Component Size Report

### 🔴 Critical (>1500 lines)
- `TradingPaper.tsx` - 2055 lines
- `AISignals.tsx` - 1488 lines

### 🟡 Warning (1000-1500 lines)
- `StrategyTuningSettings.tsx` - 1192 lines
- `TradingSettings.tsx` - 1108 lines
- `AIStrategySelector.tsx` - 1058 lines

### ⚪ OK (<1000 lines)
- All other components ✅

**Target:** All components <500 lines

---

## Bundle Size Analysis

| Chunk | Size (min) | Size (gzip) | Status |
|-------|-----------|-------------|--------|
| three-vendor | 1,190KB | 342KB | 🔴 TOO LARGE |
| chart-vendor | 330KB | 97KB | 🟡 LARGE |
| radix-vendor | 143KB | 46KB | ✅ OK |
| index | 105KB | 33KB | ✅ EXCELLENT |
| Settings | 73KB | 15KB | ✅ GOOD |
| Dashboard | 72KB | 14KB | ✅ GOOD |
| TradingPaper | 56KB | 12KB | ✅ GOOD |

**Total:** 1.97MB (550KB gzipped)
**Target:** <1MB (<400KB gzipped)

---

## Performance Metrics

| Metric | Current | Target | Status |
|--------|---------|--------|--------|
| FCP | 1.2s | <1.5s | ✅ GOOD |
| TTI | 2.8s | <3.5s | ✅ GOOD |
| LCP | 2.5s | <2.5s | ✅ GOOD |
| Bundle (main) | 104KB | <150KB | ✅ EXCELLENT |
| Bundle (vendor) | 1.5MB | <1MB | 🔴 NEEDS WORK |
| Bundle (gzip) | 550KB | <500KB | 🟡 CLOSE |

---

## Testing Gaps

- [ ] WebSocket reconnection edge cases
- [ ] Error recovery flows
- [ ] Real-time update race conditions
- [ ] Performance with 1000+ trades
- [ ] Mobile touch interactions

**Effort:** 8-12 hours

---

## Missing Features (Backend exists, no UI)

- [ ] Model Management UI
- [ ] Backtesting visualization
- [ ] Advanced Risk Analytics
- [ ] Trade Journal (notes/tags)
- [ ] Market Overview dashboard

**Effort:** 50-70 hours

---

## Security Checklist

- [x] No hardcoded secrets
- [x] JWT tokens properly managed
- [x] Input validation
- [x] XSS protection
- [ ] Generic error messages in production
- [ ] Consider httpOnly cookies for JWT

---

## Browser Compatibility

- [x] Chrome 120+
- [x] Firefox 121+
- [x] Safari 17+
- [ ] Edge 120+ (not tested)
- [ ] Add browser compatibility notice

---

## Documentation Needed

- [ ] Component API documentation
- [ ] Hook usage examples
- [ ] WebSocket protocol spec
- [ ] API error code mapping
- [ ] Detailed .env.example
- [ ] Frontend development guide

**Effort:** 8-12 hours

---

**Last Updated:** 2025-11-19
**Next Review:** 2025-12-01

