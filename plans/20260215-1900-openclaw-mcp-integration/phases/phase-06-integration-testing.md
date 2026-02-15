# Phase 06: Integration Testing & Security Audit

**Status**: Pending | **Est.**: 1.5 days | **Priority**: P0 (quality gate)

## Context Links

- [BotCore Testing Guide](../../../../docs/TESTING_GUIDE.md)
- [Security Standards](../../../../CLAUDE.md#security--best-practices)
- All previous phases (01-05)

## Overview

End-to-end integration testing of the complete OpenClaw + MCP + BotCore pipeline. Security audit of all attack surfaces. Load testing of MCP server. Verification of all guardrails and safety mechanisms. This phase is the quality gate before production deployment.

## Key Insights

1. Testing must cover the full chain: User message -> Telegram/WhatsApp -> OpenClaw -> Claude API -> MCP Server -> BotCore API -> response back.
2. Security audit focuses on 3 attack surfaces: channel auth (Telegram/WhatsApp), MCP auth (bearer token), and BotCore auth (JWT).
3. Self-tuning guardrails are the highest-risk component -- must be fuzz-tested.
4. Finance system = no shortcuts on testing. Every write operation must be tested.

## Requirements

- End-to-end test for each tool category (8 categories)
- Security audit of all auth layers
- Guardrail fuzz testing (attempt to bypass parameter bounds)
- Load test: MCP server handling concurrent tool calls
- Failure mode testing: service down, network partition, timeout
- Rollback testing: verify parameter restoration works
- Audit log integrity verification
- Memory usage profiling under load
- Documentation of all test results

## Architecture (Test Topology)

```
Test Suite
  |
  +-- E2E Tests (Playwright or manual)
  |     User -> Telegram -> OpenClaw -> Claude -> MCP -> BotCore -> Response
  |
  +-- Integration Tests (automated)
  |     MCP Client -> MCP Server -> Mock BotCore APIs
  |
  +-- Security Tests (automated + manual)
  |     Unauthorized access, injection, token replay, bounds bypass
  |
  +-- Load Tests (k6 or artillery)
  |     Concurrent MCP tool calls, sustained load
  |
  +-- Failure Mode Tests (docker compose manipulation)
        Kill services, network partition, timeout scenarios
```

## Related Code Files

| File | Purpose |
|------|---------|
| `mcp-server/tests/` (new) | MCP server test directory |
| `mcp-server/tests/integration/` | Integration tests per tool category |
| `mcp-server/tests/security/` | Security-focused tests |
| `mcp-server/tests/load/` | Load test scripts |
| `mcp-server/tests/e2e/` | End-to-end test scripts |
| `mcp-server/tests/fixtures/` | Mock API responses |

## Implementation Steps

### 1. Integration Test Framework Setup (~2h)

- Use `vitest` for TypeScript tests (consistent with frontend)
- Mock BotCore APIs using `msw` (Mock Service Worker) or simple HTTP mock server
- Create fixture files for each API response type
- Helper: `createTestMcpClient()` that connects to MCP server in test mode

### 2. Tool Integration Tests (~4h)

**Test each tool category** (8 suites, ~5 tests each = ~40 tests):

```typescript
// Example: paper-trading tools test
describe('Paper Trading Tools', () => {
  it('get_status returns engine status', async () => {
    mockApi('GET /api/paper-trading/status', { success: true, data: { is_running: true, ... } });
    const result = await mcpClient.callTool('get_paper_trading_status', {});
    expect(result.isError).toBeFalsy();
    expect(JSON.parse(result.content[0].text).is_running).toBe(true);
  });

  it('update_basic_settings requires confirmation for YELLOW tier', async () => {
    const result = await mcpClient.callTool('update_basic_settings', {
      stop_loss_percent: 1.5
    });
    // First call should return confirmation request
    expect(result.content[0].text).toContain('CONFIRM REQUIRED');
    expect(result.content[0].text).toContain('confirm_token');
  });

  it('update_basic_settings applies after confirmation', async () => {
    mockApi('PUT /api/paper-trading/basic-settings', { success: true });
    const confirmResult = await mcpClient.callTool('update_basic_settings', {
      stop_loss_percent: 1.5,
      confirm_token: validToken
    });
    expect(confirmResult.isError).toBeFalsy();
  });
});
```

**Categories to test**:
1. Market tools (8 tests): prices, candles, charts, symbols CRUD
2. Paper trading tools (15 tests): status, portfolio, trades, settings CRUD, engine start/stop
3. AI tools (8 tests): analyze, recommendations, condition, feedback, history
4. Task tools (5 tests): trigger, status, cancel, retry, stats
5. Backtest tools (3 tests): create, get, list
6. Trading tools (4 tests): positions, account, close, performance
7. Monitoring tools (5 tests): system, trading, connection, health, alerts
8. Tuning tools (8 tests): dashboard, bounds, adjustments, rollback

### 3. Security Tests (~4h)

**3a. Authentication Tests**:
```typescript
describe('Security: Authentication', () => {
  it('rejects requests without bearer token', async () => {
    const result = await fetch('http://mcp-server:8090/mcp', {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ method: 'tools/list' })
    });
    expect(result.status).toBe(401);
  });

  it('rejects requests with invalid bearer token', async () => { ... });
  it('rejects expired confirmation tokens', async () => { ... });
  it('rejects replayed confirmation tokens (single-use)', async () => { ... });
});
```

**3b. Authorization Tests**:
```typescript
describe('Security: Authorization Tiers', () => {
  it('Tier 3 tool rejects without elevated auth', async () => { ... });
  it('Tier 4 tool requires explicit approval text', async () => { ... });
  it('confirmation token cannot be used for different tool', async () => { ... });
  it('confirmation token cannot be used with different params', async () => { ... });
});
```

**3c. Guardrail Fuzz Tests**:
```typescript
describe('Security: Parameter Bounds', () => {
  const fuzzValues = [
    -1, 0, 0.001, 999999, Infinity, -Infinity, NaN,
    'string', null, undefined, [], {}, true
  ];

  it.each(fuzzValues)('rejects out-of-bounds RSI oversold: %s', async (val) => {
    const result = await mcpClient.callTool('apply_green_adjustment', {
      parameter: 'rsi_oversold',
      newValue: val,
      reasoning: 'fuzz test'
    });
    if (typeof val !== 'number' || val < 20 || val > 40) {
      expect(result.isError).toBe(true);
    }
  });

  // Test every parameter in bounds registry
  it('validates all GREEN parameters have enforced bounds', async () => { ... });
  it('validates all YELLOW parameters require confirmation', async () => { ... });
  it('validates all RED parameters require approval text', async () => { ... });
});
```

**3d. Input Validation Tests**:
```typescript
describe('Security: Input Validation', () => {
  it('rejects SQL injection in symbol param', async () => { ... });
  it('rejects XSS in reasoning field', async () => { ... });
  it('rejects oversized request body (>1MB)', async () => { ... });
  it('handles malformed JSON gracefully', async () => { ... });
});
```

### 4. Load Testing (~2h)

**Using k6 or custom script**:
```javascript
// Simulate 10 concurrent users making tool calls
// Duration: 5 minutes sustained
// Tool mix: 60% reads, 30% writes (with confirm), 10% analysis

export const options = {
  vus: 10,
  duration: '5m',
  thresholds: {
    http_req_duration: ['p95 < 500'],  // 95% under 500ms
    http_req_failed: ['rate < 0.01'],   // <1% error rate
  },
};
```

**Metrics to capture**:
- Response time p50, p95, p99
- Error rate
- Memory usage during load
- BotCore API call rate (ensure within limits)
- Rate limiter triggering correctly

### 5. Failure Mode Tests (~2h)

```bash
# Test: MCP server handles Rust service down
docker stop rust-core-engine
# -> MCP tools should return isError: true with "service unavailable"
# -> OpenClaw/Claude should inform user: "Rust Core Engine is down"
docker start rust-core-engine

# Test: MCP server handles Python service down
docker stop python-ai-service
# -> AI tools return isError, other tools still work

# Test: MCP server handles MongoDB down
docker stop mongodb
# -> Graceful degradation, cached data still served

# Test: Network timeout (simulate slow response)
# -> 30s timeout for regular calls, 120s for AI analysis

# Test: MCP server crash recovery
docker restart mcp-server
# -> OpenClaw reconnects, no state loss
```

### 6. Audit Log & Rollback Tests (~1h)

```typescript
describe('Audit & Rollback', () => {
  it('every GREEN adjustment creates audit entry', async () => {
    await mcpClient.callTool('apply_green_adjustment', { ... });
    const history = await mcpClient.callTool('get_adjustment_history', { lastN: 1 });
    expect(history).toHaveLength(1);
    expect(history[0]).toHaveProperty('oldValue');
    expect(history[0]).toHaveProperty('newValue');
    expect(history[0]).toHaveProperty('reasoning');
  });

  it('rollback restores exact previous state', async () => {
    // Get snapshot before adjustment
    // Apply adjustment
    // Verify parameter changed
    // Rollback
    // Verify parameter restored to exact previous value
  });

  it('audit log is append-only (no gaps)', async () => {
    // Verify sequential IDs
    // Verify no missing entries
  });
});
```

### 7. E2E Smoke Test (~2h)

Manual or scripted end-to-end test via Telegram:

| # | User Message | Expected Behavior |
|---|-------------|-------------------|
| 1 | "What's the system health?" | Claude calls `check_system_health`, reports status |
| 2 | "Show my portfolio" | Claude calls `get_portfolio`, formats nicely |
| 3 | "What's BTC price?" | Claude calls `get_market_prices`, shows BTC |
| 4 | "Analyze BTC market" | Claude calls `analyze_market`, shows AI signal |
| 5 | "Change stop loss to 1.5%" | Claude calls `request_yellow_adjustment`, asks confirm |
| 6 | "Yes, confirm" | Claude applies change, shows before/after |
| 7 | "Undo that change" | Claude calls `rollback_adjustment`, confirms |
| 8 | "Stop paper trading" | Claude calls `request_red_adjustment`, asks explicit text |
| 9 | "APPROVE STOP ENGINE" | Claude stops engine, confirms |
| 10 | "Start paper trading" | Claude restarts engine |

### 8. Memory Profiling (~1h)

```bash
# Monitor all containers for 1 hour under normal cron load
docker stats --format "table {{.Name}}\t{{.MemUsage}}\t{{.MemPerc}}" > mem-profile.log

# Verify:
# - MCP server < 384MB
# - OpenClaw < 768MB
# - Total all containers < 3.8GB
# - No memory leaks (trend should be flat)
```

## Todo List

- [ ] Set up test framework (vitest + msw for mocking)
- [ ] Create mock API response fixtures for all 80 endpoints
- [ ] Write market tools integration tests (8 tests)
- [ ] Write paper trading tools integration tests (15 tests)
- [ ] Write AI tools integration tests (8 tests)
- [ ] Write task/backtest tools integration tests (8 tests)
- [ ] Write trading/monitoring tools integration tests (9 tests)
- [ ] Write tuning tools integration tests (8 tests)
- [ ] Write authentication security tests (4 tests)
- [ ] Write authorization tier security tests (4 tests)
- [ ] Write guardrail fuzz tests (all parameters)
- [ ] Write input validation tests (injection, XSS, overflow)
- [ ] Create and run load test (10 VUs, 5 min)
- [ ] Run failure mode tests (service down, timeout, crash)
- [ ] Run audit log integrity tests
- [ ] Run rollback verification tests
- [ ] Execute E2E smoke test via Telegram (10 scenarios)
- [ ] Run memory profiling for 1 hour
- [ ] Document all test results in report
- [ ] Fix any issues found during testing
- [ ] Re-test after fixes

## Success Criteria

1. All integration tests pass (56+ tests across 8 categories)
2. All security tests pass (12+ tests)
3. Guardrail fuzz tests: 100% rejection of out-of-bounds values
4. Load test: p95 < 500ms, error rate < 1%
5. Failure mode: graceful degradation with informative error messages
6. Audit log: zero gaps, zero corruption
7. Rollback: exact state restoration verified
8. E2E: all 10 smoke test scenarios pass
9. Memory: all containers within limits after 1 hour
10. No HIGH/CRITICAL security findings

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Tests reveal deep architectural issues | Low | High | Phase 01-05 built incrementally, issues caught early |
| Load test shows MCP can't handle cron burst | Medium | Medium | Stagger cron schedules, optimize tool calls |
| Security audit finds auth bypass | Low | Critical | Fix immediately, do not deploy until resolved |
| Memory profiling shows leak | Medium | High | Heap dump analysis, fix in affected container |

## Security Considerations

- Security test results must be documented but NOT committed to public repo
- Any findings rated HIGH or CRITICAL block deployment
- Fuzz test results help build regression test suite
- Audit log tests verify tamper-evidence (append-only)
- Rate limit tests verify DoS protection

## Deliverables

1. **Test Report**: `plans/20260215-1900-openclaw-mcp-integration/reports/test-report.md`
   - Test counts (pass/fail/skip)
   - Coverage metrics
   - Performance benchmarks
   - Security findings

2. **Security Audit Report**: `plans/20260215-1900-openclaw-mcp-integration/reports/security-audit.md`
   - Attack surface analysis
   - Findings with severity ratings
   - Remediation status

3. **Deployment Checklist**: `plans/20260215-1900-openclaw-mcp-integration/reports/deployment-checklist.md`
   - Pre-deployment verification steps
   - Environment variable checklist
   - Rollback procedure

## Next Steps

After this phase: **DEPLOY** -- all services go live on VPS. Monitor for 48 hours with increased logging. Then reduce log verbosity and move to steady-state operation.
