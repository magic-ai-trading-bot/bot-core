# Phase 04: Frontend Quality Improvements

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: Phase 01 (Security)
**Blocks**: Phase 07 (Testing)

---

## Overview

| Field | Value |
|-------|-------|
| Date | 2026-02-06 |
| Priority | P2-MEDIUM |
| Status | Pending |
| Effort | Medium (3-4 days) |
| Risk | LOW - UI changes don't affect trading logic |

---

## Key Insights (From Reports)

**Source**: `reports/03-frontend-code-review.md`

**Overall Grade**: GOOD (0 critical issues)
- **Critical**: 0
- **High**: 3 (missing AbortController, WebSocket reconnect, race condition)
- **Medium**: 8 (memoization, ErrorBoundary, API URLs, retry logic)
- **Low**: 6 (PropTypes, magic numbers, console logs, large components)

**TypeScript Status**: ✅ PASS (0 errors)

---

## Requirements

### HIGH-01: Add AbortController to API Calls
- **File**: `src/hooks/useAIAnalysis.ts:91-133`
- **Issue**: Parallel API calls without AbortSignal
- **Fix**: Pass signal to all fetch operations, abort on unmount
- **Ref**: Frontend Review H-2

### HIGH-02: Add WebSocket Reconnection to usePaperTrading
- **File**: `src/hooks/usePaperTrading.ts:897-1100`
- **Issue**: No reconnection logic unlike useWebSocket hook
- **Fix**: Add reconnection with exponential backoff
- **Ref**: Frontend Review H-3

### MEDIUM-03: Fix Race Condition in Signal Deduplication
- **File**: `src/hooks/usePaperTrading.ts:573-582`
- **Issue**: Multiple async operations can overlap, break deduplication
- **Fix**: Use request ID pattern to discard stale responses
- **Ref**: Frontend Review M-1

### MEDIUM-04: Add ErrorBoundary Around Suspense
- **File**: `src/App.tsx:78-202`
- **Issue**: Lazy route load failures show blank screen
- **Fix**: Wrap Suspense in ErrorBoundary
- **Ref**: Frontend Review M-2

### MEDIUM-05: Add useMemo to AISignalsDashboard
- **File**: `src/components/ai/AISignalsDashboard.tsx:19-89`
- **Issue**: Heavy array transformations on every render
- **Fix**: Memoize signal processing, sorting, deduplication
- **Ref**: Frontend Review M-4

### MEDIUM-06: Add React.memo to BotSettings
- **File**: `src/components/dashboard/BotSettings.tsx:234-400`
- **Issue**: Re-renders when parent context updates
- **Fix**: Wrap in React.memo, memoize expensive calculations
- **Ref**: Frontend Review M-5

### MEDIUM-07: Centralize API URL Configuration
- **Files**: `BotSettings.tsx:13`, `usePaperTrading.ts:234`
- **Issue**: API_BASE duplicated across files
- **Fix**: Export from services/api.ts, import everywhere
- **Ref**: Frontend Review M-6

### MEDIUM-08: Extract WebSocket Message Handlers
- **File**: `src/hooks/usePaperTrading.ts:913-1077`
- **Issue**: 164-line onmessage handler, hard to maintain
- **Fix**: Extract handlers per event type
- **Ref**: Frontend Review M-7

### MEDIUM-09: Fix API Retry Logic
- **File**: `src/services/api.ts:426-445`
- **Issue**: Retries on all errors including 4xx
- **Fix**: Only retry on network/5xx errors
- **Ref**: Frontend Review M-8

### LOW-10: Replace console.* with logger.*
- **Files**: 13 files with console.log/warn/error
- **Issue**: Sensitive data may leak to browser console
- **Fix**: Use logger utility everywhere
- **Ref**: Frontend Review L-3

### LOW-11: Split BotSettings Component
- **File**: `src/components/dashboard/BotSettings.tsx` (400 lines)
- **Issue**: Component too large
- **Fix**: Extract sub-components for each section
- **Ref**: Frontend Review L-4

### LOW-12: Extract Magic Numbers to Constants
- **Files**: `useAIAnalysis.ts:42`, `usePaperTrading.ts:467-476`
- **Issue**: Hardcoded 30000, 30 * 60 * 1000
- **Fix**: Define named constants
- **Ref**: Frontend Review L-2

---

## Related Code Files

```
nextjs-ui-dashboard/src/
├── hooks/
│   ├── useAIAnalysis.ts            # AbortController
│   ├── usePaperTrading.ts          # WebSocket reconnect, race condition
│   └── useWebSocket.ts             # Reference for reconnection pattern
├── components/
│   ├── ai/
│   │   └── AISignalsDashboard.tsx  # useMemo optimization
│   ├── dashboard/
│   │   ├── BotSettings.tsx         # React.memo, split
│   │   ├── BotStatusSection.tsx    # NEW: extracted
│   │   ├── CapitalSection.tsx      # NEW: extracted
│   │   └── RiskSection.tsx         # NEW: extracted
│   └── ErrorBoundary.tsx           # Existing
├── services/
│   └── api.ts                      # Export API URL, fix retry
├── constants/
│   └── trading.ts                  # NEW: magic numbers
├── pages/
│   └── Settings.tsx                # Replace console with logger
└── App.tsx                         # ErrorBoundary around Suspense
```

---

## Implementation Steps

### Step 1: Add AbortController to useAIAnalysis
```typescript
// In useAIAnalysis.ts
const fetchRealCandles = useCallback(
  async (symbol: string, signal?: AbortSignal): Promise<...> => {
    const [chartData15m, chartData30m, chartData1h, chartData4h] = await Promise.all([
      apiClient.rust.getChartData(symbol, "15m", 100, { signal }),
      apiClient.rust.getChartData(symbol, "30m", 100, { signal }),
      apiClient.rust.getChartData(symbol, "1h", 100, { signal }),
      apiClient.rust.getChartData(symbol, "4h", 100, { signal }),
    ]);
    // ...
  },
  []
);

// In useEffect:
useEffect(() => {
  const controller = new AbortController();
  fetchRealCandles(symbol, controller.signal);
  return () => controller.abort();
}, [symbol, fetchRealCandles]);
```

### Step 2: Add WebSocket Reconnection to usePaperTrading
```typescript
// In usePaperTrading.ts - add reconnection similar to useWebSocket
const reconnectTimeoutRef = useRef<NodeJS.Timeout>();
const reconnectAttemptsRef = useRef(0);
const MAX_RECONNECT_ATTEMPTS = 5;

const connect = useCallback(() => {
  const ws = new WebSocket(wsUrl);

  ws.onclose = (event) => {
    if (!event.wasClean && reconnectAttemptsRef.current < MAX_RECONNECT_ATTEMPTS) {
      const delay = Math.pow(2, reconnectAttemptsRef.current) * 1000;
      reconnectTimeoutRef.current = setTimeout(() => {
        reconnectAttemptsRef.current++;
        connect();
      }, delay);
    }
  };
  // ...
}, [wsUrl]);
```

### Step 3: Add Request ID for Race Condition
```typescript
// In usePaperTrading.ts
const fetchRequestIdRef = useRef(0);

const fetchAISignals = useCallback(async () => {
  const requestId = ++fetchRequestIdRef.current;
  // ... fetch logic ...

  setState((prev) => {
    if (requestId !== fetchRequestIdRef.current) return prev; // Stale
    return {
      ...prev,
      recentSignals: deduplicateSignals([...validSignals, ...prev.recentSignals]),
    };
  });
}, []);
```

### Step 4: Wrap Suspense in ErrorBoundary
```tsx
// In App.tsx
<ErrorBoundary fallback={<ErrorPage />}>
  <Suspense fallback={<LoadingFallback />}>
    <Routes>
      {/* routes */}
    </Routes>
  </Suspense>
</ErrorBoundary>
```

### Step 5: Add useMemo to AISignalsDashboard
```tsx
// In AISignalsDashboard.tsx
const allSignals = useMemo(() => {
  const allSignalsRaw = [
    ...aiState.signals.map((s) => ({ ...s, source: "api" as const })),
    ...wsState.aiSignals.map((s) => ({
      // ... transformation
    })),
  ];

  const normalized = allSignalsRaw.map(/*...*/).sort(/*...*/);
  const uniqueMap = new Map(/*...*/);
  return Array.from(uniqueMap.values()).sort(/*...*/);
}, [aiState.signals, wsState.aiSignals]);
```

### Step 6: Fix API Retry Logic
```typescript
// In services/api.ts:requestWithRetry
protected async requestWithRetry<T>(
  request: () => Promise<T>,
  maxRetries: number = 2,
  backoffMs: number = 200
): Promise<T> {
  for (let attempt = 1; attempt <= maxRetries; attempt++) {
    try {
      return await request();
    } catch (error: any) {
      const shouldRetry =
        !error.response || // Network error
        error.response.status >= 500; // Server error

      if (attempt === maxRetries || !shouldRetry) {
        throw error;
      }

      await new Promise((r) => setTimeout(r, backoffMs * Math.pow(2, attempt - 1)));
    }
  }
  throw new Error("Unreachable"); // TypeScript satisfaction
}
```

### Step 7: Create Trading Constants
```typescript
// NEW: constants/trading.ts
export const SIGNAL_MAX_AGE_MS = 30 * 60 * 1000; // 30 minutes
export const REFRESH_INTERVAL_MS = 30 * 1000; // 30 seconds
export const WS_RECONNECT_MAX_ATTEMPTS = 5;
export const WS_RECONNECT_BASE_DELAY_MS = 1000;
```

---

## Todo List

- [ ] Add AbortController to fetchRealCandles in useAIAnalysis.ts
- [ ] Update apiClient methods to accept signal parameter
- [ ] Add WebSocket reconnection logic to usePaperTrading.ts
- [ ] Add request ID pattern to fetchAISignals
- [ ] Wrap Suspense in ErrorBoundary in App.tsx
- [ ] Add useMemo to AISignalsDashboard signal processing
- [ ] Add React.memo to BotSettings component
- [ ] Add useMemo to expensive calculations in BotSettings
- [ ] Export RUST_API_URL from services/api.ts
- [ ] Update BotSettings.tsx to import API URL
- [ ] Update usePaperTrading.ts to import API URL
- [ ] Extract handleMarketDataMessage from onmessage handler
- [ ] Extract handleTradeExecuted from onmessage handler
- [ ] Extract handlePositionUpdate from onmessage handler
- [ ] Fix requestWithRetry to only retry on network/5xx
- [ ] Replace console.* with logger.* in 13 files
- [ ] Create constants/trading.ts with magic numbers
- [ ] Split BotSettings into sub-components (if time permits)
- [ ] Run npm test to verify changes
- [ ] Run npm run type-check

---

## Success Criteria

| Criteria | Metric | Target |
|----------|--------|--------|
| AbortController usage | code review | All fetch calls |
| WebSocket reconnection | test manual disconnect | Auto-reconnects |
| Race conditions | concurrent requests test | No stale data |
| Memoization | React DevTools profiler | <50% re-renders |
| Console logs | grep count | 0 console.* |
| TypeScript errors | tsc output | 0 |
| Test pass rate | npm test | 100% |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| AbortController breaks flows | Low | Low | Test unmount scenarios |
| Memoization breaks updates | Low | Medium | Verify dependency arrays |
| Reconnection storms server | Low | Medium | Exponential backoff |

---

## Security Considerations

- AbortController prevents sensitive data from completing after logout
- Console log removal prevents data leakage
- Error boundaries prevent stack trace exposure

---

## Estimated Completion

- **AbortController + reconnection**: 1 day
- **Memoization + optimizations**: 1 day
- **Refactoring (handlers, constants)**: 0.5 day
- **Console cleanup + testing**: 0.5 day

**Total**: 3 days
