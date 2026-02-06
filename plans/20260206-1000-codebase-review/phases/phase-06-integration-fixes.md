# Phase 06: Integration & API Consistency Fixes

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: Phase 02 (Rust), Phase 03 (Python)
**Blocks**: Phase 07 (Testing)

---

## Overview

| Field | Value |
|-------|-------|
| Date | 2026-02-06 |
| Priority | P2-MEDIUM |
| Status | Pending |
| Effort | Medium (3-4 days) |
| Risk | MEDIUM - API changes may break frontend |

---

## Key Insights (From Reports)

**Source**: `reports/06-integration-review.md`

**Overall Health Score**: 87/100 (B+)
- API Contract Consistency: 85/100
- Data Model Alignment: 90/100
- WebSocket Protocol: 88/100
- Authentication Flow: 92/100
- Error Handling: 80/100

**Critical Issues**:
1. 8 undocumented Python endpoints
2. Database schema drift (User collection)
3. WebSocket payload mismatch (PositionUpdate)
4. Missing auth endpoints (refresh, logout)
5. Inconsistent error formats (Python vs Rust)

---

## Requirements

### HIGH-01: Document Undocumented Python Endpoints
- **File**: `specs/02-design/2.3-api/API-PYTHON-AI.md`
- **Endpoints** (8 total):
  - `/predict-trend`
  - `/ai/cost/statistics`
  - `/ai/config-analysis/trigger`
  - `/ai/config-suggestions`
  - `/ai/gpt4-analysis-history`
  - `/api/chat/project`
  - `/api/chat/project/suggestions`
  - `/api/chat/project/clear`
- **Fix**: Add to API spec or mark as deprecated
- **Ref**: Integration Review Section 1.2

### HIGH-02: Fix Database Schema Drift
- **File**: `specs/02-design/2.2-database/DB-SCHEMA.md`
- **Missing Fields** (User collection):
  - `display_name` (implemented in Rust + Frontend)
  - `avatar_url` (implemented in Rust)
  - `two_factor_enabled` (implemented in Rust + Frontend)
  - `two_factor_secret` (implemented in Rust)
- **Fix**: Update DB-SCHEMA.md to v2.1.0
- **Ref**: Integration Review Section 2.1

### HIGH-03: Fix WebSocket PositionUpdate Payload
- **Files**:
  - `specs/02-design/2.3-api/API-WEBSOCKET.md`
  - `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`
- **Issue**: Frontend expects 6 fields, spec defines 15+ fields
- **Missing**: action, id, quantity, leverage, stop_loss, take_profit, etc.
- **Fix**: Update PositionUpdateData interface
- **Ref**: Integration Review Section 3.2

### MEDIUM-04: Add Token Refresh Endpoint
- **Files**: `rust-core-engine/src/auth/handlers.rs`
- **Issue**: No refresh mechanism for expired tokens
- **Fix**: Add POST /api/auth/refresh endpoint
- **Ref**: Integration Review Section 4.2

### MEDIUM-05: Standardize Error Response Format
- **Files**: `python-ai-service/main.py` (error handlers)
- **Issue**: Python uses `{"detail": ...}`, Rust uses `{"error": ...}`
- **Fix**: Create custom exception handler to match Rust format
- **Ref**: Integration Review Section 5.1

### MEDIUM-06: Fix Position Model Hybrid Fields
- **File**: `nextjs-ui-dashboard/src/services/api.ts`
- **Issue**: Position has both `side` and `trade_type`
- **Fix**: Complete migration to `trade_type`, deprecate `side`
- **Ref**: Integration Review Section 1.3

### MEDIUM-07: Standardize Python WebSocket Messages
- **File**: `python-ai-service/main.py` (WebSocket handlers)
- **Issue**: Simple "connection" message instead of typed event
- **Fix**: Match Rust event structure
- **Ref**: Integration Review Section 3.1

### LOW-08: Add Rate Limiting Implementation
- **Files**: Rust and Python API handlers
- **Issue**: Documented in spec but not verified
- **Fix**: Verify and document rate limits
- **Ref**: Integration Review Section 5.2

### LOW-09: Fix Hardcoded localhost in Docs
- **Files**: Various spec files
- **Issue**: Examples show `localhost:8080` instead of env variable
- **Fix**: Use placeholder or env var in examples
- **Ref**: Integration Review Section 6.3

---

## Related Code Files

```
# Specs to Update
specs/02-design/
├── 2.2-database/
│   └── DB-SCHEMA.md                # Add missing User fields
├── 2.3-api/
│   ├── API-PYTHON-AI.md            # Add 8 endpoints
│   └── API-WEBSOCKET.md            # Fix PositionUpdate payload

# Rust Code
rust-core-engine/src/
├── auth/
│   └── handlers.rs                 # Add refresh endpoint

# Python Code
python-ai-service/
├── main.py                         # Error handlers, WebSocket messages
└── error_handlers.py               # NEW: Standardized responses

# Frontend Code
nextjs-ui-dashboard/src/
├── hooks/
│   └── useWebSocket.ts             # Fix PositionUpdateData
└── services/
    └── api.ts                      # Migrate side → trade_type
```

---

## Implementation Steps

### Step 1: Update DB-SCHEMA.md
```markdown
## users Collection (v2.1.0)

| Field | Type | Description |
|-------|------|-------------|
| ... existing fields ... |
| display_name | string? | User display name (optional) |
| avatar_url | string? | Profile picture URL (optional) |
| two_factor_enabled | boolean | 2FA enabled status |
| two_factor_secret | string? | TOTP secret (encrypted) |
```

### Step 2: Document Python Endpoints
```markdown
## Cost Monitoring Endpoints

### GET /ai/cost/statistics
Returns OpenAI API usage statistics.

**Response:**
```json
{
  "total_requests": 1234,
  "total_input_tokens": 456789,
  "total_output_tokens": 123456,
  "total_cost_usd": 12.34
}
```

### POST /api/chat/project
Project-specific RAG chatbot.

**Request:**
```json
{
  "message": "How do I configure risk limits?",
  "context": "paper_trading"
}
```
```

### Step 3: Fix PositionUpdateData Interface
```typescript
// In useWebSocket.ts
export interface PositionUpdateData {
  action: "OPENED" | "CLOSED" | "UPDATED";
  position: {
    id: string;
    symbol: string;
    side: "LONG" | "SHORT";
    quantity: number;
    entry_price: number;
    current_price: number;
    unrealized_pnl: number;
    unrealized_pnl_percent: number;
    leverage: number;
    stop_loss?: number;
    take_profit?: number;
    liquidation_price?: number;
    margin: number;
    entry_time: number;
  };
  timestamp: number;
}
```

### Step 4: Add Token Refresh Endpoint
```rust
// In auth/handlers.rs
pub async fn refresh_token(
    auth_header: Option<String>,
    db: Arc<Database>,
    jwt_manager: Arc<JwtManager>,
) -> Result<impl Reply, Rejection> {
    let refresh_token = extract_bearer_token(auth_header)?;
    let claims = jwt_manager.verify_refresh_token(&refresh_token)?;

    let new_access_token = jwt_manager.generate_access_token(&claims.sub)?;
    let new_refresh_token = jwt_manager.generate_refresh_token(&claims.sub)?;

    Ok(warp::reply::json(&ApiResponse::success(json!({
        "token": new_access_token,
        "refresh_token": new_refresh_token,
        "expires_in": 604800
    }))))
}
```

### Step 5: Standardize Python Error Format
```python
# NEW: error_handlers.py
from fastapi import Request
from fastapi.responses import JSONResponse
from fastapi.exceptions import RequestValidationError

@app.exception_handler(Exception)
async def global_exception_handler(request: Request, exc: Exception):
    return JSONResponse(
        status_code=500,
        content={
            "success": False,
            "error": str(exc),
            "details": None
        }
    )

@app.exception_handler(RequestValidationError)
async def validation_exception_handler(request: Request, exc: RequestValidationError):
    return JSONResponse(
        status_code=400,
        content={
            "success": False,
            "error": "Validation failed",
            "details": exc.errors()
        }
    )
```

### Step 6: Migrate Position side to trade_type
```typescript
// In api.ts - during API response parsing
function normalizePosition(data: any): Position {
  return {
    ...data,
    // Prefer new field, fallback to old
    trade_type: data.trade_type || (data.side === "BUY" ? "Long" : "Short"),
    // Deprecation: remove side mapping after full migration
  };
}
```

---

## Todo List

### High Priority
- [ ] Update DB-SCHEMA.md with missing User fields (v2.1.0)
- [ ] Document 8 undocumented Python endpoints in API-PYTHON-AI.md
- [ ] Update PositionUpdateData interface in useWebSocket.ts
- [ ] Update API-WEBSOCKET.md PositionUpdate payload definition

### Medium Priority
- [ ] Implement POST /api/auth/refresh in Rust
- [ ] Add refresh token generation to jwt.rs
- [ ] Create error_handlers.py with standardized format
- [ ] Register exception handlers in main.py
- [ ] Migrate Position.side to Position.trade_type
- [ ] Update Python WebSocket to use typed events

### Low Priority
- [ ] Document rate limiting in API specs
- [ ] Replace localhost in spec examples with placeholders
- [ ] Add logout endpoint (token blacklist)
- [ ] Review and close Position.side deprecation

---

## Success Criteria

| Criteria | Metric | Target |
|----------|--------|--------|
| Documented endpoints | API spec coverage | 100% |
| Schema alignment | DB-SCHEMA vs code | 100% match |
| WebSocket payload | TypeScript vs spec | Full match |
| Error format | Python responses | Match Rust |
| Test pass | Integration tests | 100% |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Frontend breaks on payload change | Medium | Medium | Add backward compat |
| Token refresh breaks sessions | Low | High | Test extensively |
| Error format change breaks clients | Low | Medium | Version API |

---

## Security Considerations

- Refresh tokens should have longer expiry than access tokens
- Refresh token rotation (new refresh token on each use)
- Token blacklist for logout functionality
- Rate limiting on refresh endpoint (prevent brute force)

---

## Estimated Completion

- **Documentation updates**: 1 day
- **WebSocket/TypeScript fixes**: 1 day
- **Auth refresh endpoint**: 1 day
- **Error format + testing**: 1 day

**Total**: 3-4 days
