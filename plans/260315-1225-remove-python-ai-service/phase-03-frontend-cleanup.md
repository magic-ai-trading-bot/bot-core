# Phase 3: Frontend - Remove AI Endpoints

**Priority**: P1 | **Effort**: 1.5h | **Status**: Pending

## Files to Modify

| File | Action |
|------|--------|
| `nextjs-ui-dashboard/src/services/api.ts` | Remove PYTHON_AI_URL references |
| `nextjs-ui-dashboard/src/services/chatbot.ts` | Remove Python chatbot calls |
| `nextjs-ui-dashboard/src/pages/AISignals.tsx` | Convert to show strategy signals only |
| `nextjs-ui-dashboard/src/hooks/useAIAnalysis.ts` | Remove or stub (return empty data) |
| `nextjs-ui-dashboard/src/hooks/usePaperTrading.ts` | Remove AI signal calls |
| `nextjs-ui-dashboard/src/hooks/useRealTrading.ts` | Remove AI signal calls |
| `nextjs-ui-dashboard/.env.example` | Remove VITE_PYTHON_AI_URL |
| Tests: `src/__tests__/hooks/useAIAnalysis.*.test.ts` | Remove or update |
| Tests: `src/__tests__/services/api.test.ts` | Remove Python endpoint tests |

## Implementation Steps

### 3a. Remove Python URL from api.ts
- Delete `PYTHON_AI_URL` constant
- Remove any fetch calls to port 8000

### 3b. AISignals page
- Keep page but show strategy-generated signals from Rust API
- Remove direct Python API calls
- Data source: `/api/strategies/signals/:symbol` (already exists in Rust)

### 3c. Remove chatbot.ts Python calls
- Remove `/api/chat/project` calls to Python
- If chatbot feature needed, it goes through MCP/OpenClaw instead

### 3d. Clean hooks
- `useAIAnalysis.ts` → stub or remove
- Trading hooks should use Rust API exclusively

## Verification

```bash
cd nextjs-ui-dashboard && npm run type-check
cd nextjs-ui-dashboard && npm test
```

## Todo

- [x] Remove PYTHON_AI_URL from api.ts
- [x] Update AISignals page to use Rust strategy API
- [x] Remove chatbot Python calls
- [x] Clean trading hooks
- [x] Remove .env.example Python URL
- [x] Update/remove AI-related tests
- [x] type-check passes
- [x] npm test passes

## Status: COMPLETED
