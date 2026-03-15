# Phase 4: MCP Server - Remove AI Tools

**Priority**: P1 | **Effort**: 30min | **Status**: Completed

## Files to Modify

| File | Action |
|------|--------|
| `mcp-server/src/tools/ai.ts` | Remove Python-proxying tools |
| `mcp-server/src/client.ts` | Remove PYTHON_API_URL config |

## Implementation Steps

### 4a. Remove Python proxy tools from ai.ts
- Remove `get_ai_performance` tool (calls Python `/ai/performance`)
- Remove `get_ai_storage_stats` tool (calls Python `/ai/storage/stats`)
- Keep any tools that call Rust API directly

### 4b. Clean client.ts
- Remove `PYTHON_API_URL` environment variable
- Remove Python HTTP client configuration

### 4c. Update tool count in docs
- CLAUDE.md references "114 tools" — update count after removal

## Verification

```bash
cd mcp-server && npx tsc --noEmit
```

## Todo

- [x] Remove Python proxy tools from ai.ts (updated descriptions; Python-proxying tools `get_ai_performance`/`get_ai_storage_stats` were already absent; updated titles/descriptions to remove GPT-4/Python references)
- [x] Remove PYTHON_API_URL from client.ts (already clean — only RUST_API_URL; ServiceTarget = "rust")
- [x] Verify TypeScript compiles (npx tsc --noEmit: 0 errors)
- [x] Update tests to reflect current tool set (89/89 pass)
