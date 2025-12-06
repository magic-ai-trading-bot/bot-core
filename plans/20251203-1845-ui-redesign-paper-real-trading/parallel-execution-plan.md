# Parallel Execution Plan - UI Redesign

## Dependency Analysis

```
Phase 1 (Design System) ─┬─► Phase 2 (Navigation) ─┬─► Phase 3 (Landing)
                         │                         │
                         │                         └─► Phase 4 (Dashboard)
                         │
                         └─► Phase 5 (Mode) ─────────► Phase 6 (Trading)
                                                      │
                                                      ├─► Phase 7 (Settings)
                                                      │
                                                      └─► Phase 8 (3D)
                                                           │
                                                           └─► Phase 9 (Polish)
```

## Optimized Execution Rounds

### Round 1: Foundation (Sequential)
| Phase | Description | Agent | Files Owned |
|-------|-------------|-------|-------------|
| Phase 1 | Design System | Agent-01 | `src/styles/`, `tailwind.config.*`, `index.css` |

**Blocking**: All other phases depend on design tokens

---

### Round 2: Core Infrastructure (2 Parallel Agents)
| Phase | Description | Agent | Files Owned |
|-------|-------------|-------|-------------|
| Phase 2 | Navigation & Layout | Agent-02 | `src/components/layout/`, `src/hooks/useSidebar.ts` |
| Phase 5 | Mode Infrastructure | Agent-03 | `src/contexts/TradingModeContext.tsx`, `src/hooks/useTradingMode.ts`, `src/hooks/useRealTrading.ts` |

**Blocking**: Phases 3, 4, 6, 7, 8 depend on these

---

### Round 3: Main Pages (2 Parallel Agents)
| Phase | Description | Agent | Files Owned |
|-------|-------------|-------|-------------|
| Phase 3 | Landing Page | Agent-04 | `src/pages/Landing.tsx`, `src/components/landing/` |
| Phase 4 | Dashboard | Agent-05 | `src/pages/Dashboard.tsx`, `src/components/dashboard/` |

**Blocking**: Phase 9

---

### Round 4: Feature Pages (3 Parallel Agents)
| Phase | Description | Agent | Files Owned |
|-------|-------------|-------|-------------|
| Phase 6 | Trading Pages | Agent-06 | `src/pages/PaperTrading.tsx`, `src/pages/RealTrading.tsx`, `src/components/trading/` |
| Phase 7 | Settings & Profile | Agent-07 | `src/pages/Settings.tsx`, `src/pages/Profile.tsx`, `src/components/settings/`, `src/components/profile/` |
| Phase 8 | 3D Visualizations | Agent-08 | `src/components/3d/` |

**Blocking**: Phase 9

---

### Round 5: Final Polish (Sequential)
| Phase | Description | Agent | Files Owned |
|-------|-------------|-------|-------------|
| Phase 9 | Polish & Testing | Agent-09 | All files (review), test files |

---

## File Ownership Matrix

| Directory/File | Owner Phase | Conflicts With |
|----------------|-------------|----------------|
| `src/styles/tokens/` | Phase 1 | None |
| `tailwind.config.*` | Phase 1 | None |
| `src/index.css` | Phase 1 | None |
| `src/components/layout/` | Phase 2 | None |
| `src/hooks/useSidebar.ts` | Phase 2 | None |
| `src/contexts/TradingModeContext.tsx` | Phase 5 | None |
| `src/hooks/useTradingMode.ts` | Phase 5 | None |
| `src/hooks/useRealTrading.ts` | Phase 5 | None |
| `src/pages/Landing.tsx` | Phase 3 | None |
| `src/components/landing/` | Phase 3 | None |
| `src/pages/Dashboard.tsx` | Phase 4 | None |
| `src/components/dashboard/` | Phase 4 | None |
| `src/pages/PaperTrading.tsx` | Phase 6 | None |
| `src/pages/RealTrading.tsx` | Phase 6 | None |
| `src/components/trading/` | Phase 6 | None |
| `src/pages/Settings.tsx` | Phase 7 | None |
| `src/pages/Profile.tsx` | Phase 7 | None |
| `src/components/settings/` | Phase 7 | None |
| `src/components/profile/` | Phase 7 | None |
| `src/components/3d/` | Phase 8 | None |
| `src/App.tsx` | Phase 2 (layout), Phase 5 (provider) | Sequential updates |

## Shared Resources (Require Coordination)

1. **App.tsx** - Multiple phases need to modify:
   - Phase 2: Add MainLayout wrapper
   - Phase 5: Add TradingModeProvider
   - Phase 3,4,6,7: Add routes
   → Solution: Phase 2 owns initial structure, others add incrementally

2. **package.json** - Dependencies:
   - Phase 1: framer-motion
   - Phase 8: three, @react-three/fiber, @react-three/drei
   → Solution: Install all upfront in Round 1

## Execution Timeline

```
Round 1 ──────► Round 2 ──────► Round 3 ──────► Round 4 ──────► Round 5
[Phase 1]      [Phase 2]       [Phase 3]       [Phase 6]       [Phase 9]
               [Phase 5]       [Phase 4]       [Phase 7]
                                               [Phase 8]

Est. Time:
~1 day         ~1 day          ~1 day          ~2 days         ~1 day

Total: ~6 days (parallel) vs ~25 days (sequential)
```

## Pre-Implementation Checklist

- [ ] Install all dependencies upfront:
  ```bash
  npm install framer-motion three @react-three/fiber @react-three/drei
  ```
- [ ] Create directory structure:
  ```bash
  mkdir -p src/styles/tokens src/components/{layout,landing,dashboard,trading,settings,profile,3d}
  ```
- [ ] Verify existing files won't conflict

## Risk Mitigation

1. **Merge conflicts**: Each phase owns distinct files
2. **Shared dependencies**: Install all upfront
3. **App.tsx coordination**: Sequential updates, not parallel
4. **Test failures**: Run tests after each round
