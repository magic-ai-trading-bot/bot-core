# Phase 03: Python AI Service Quality Improvements

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: Phase 01 (Security)
**Blocks**: Phase 07 (Testing)

---

## Overview

| Field | Value |
|-------|-------|
| Date | 2026-02-06 |
| Priority | P1-HIGH |
| Status | Pending |
| Effort | Medium (3-4 days) |
| Risk | MEDIUM - AI service non-critical to trading execution |

---

## Key Insights (From Reports)

**Source**: `reports/02-python-code-review.md`

**Overall Grade**: B+ (87/100)
- **Critical**: 3 (bare except, fire-and-forget tasks, MongoDB cleanup)
- **High**: 4 (global state, incomplete backtest, type hints, Redis)
- **Medium**: 4 (large functions, magic numbers, SSRF, error formats)
- **Low**: 3 (docstrings, MongoDB indexes, error formats)

**ML-Specific Issues**: 3 (model versioning, scaler persistence, input validation)

---

## Requirements

### HIGH-01: Fix Bare Exception Handlers
- **File**: `tasks/monitoring.py:148`
- **Issue**: `except: pass` swallows all exceptions
- **Fix**: Use specific exception types with logging
- **Ref**: Python Review ISSUE-01

### HIGH-02: Add Error Handlers to Background Tasks
- **File**: `main.py:486-493`
- **Issue**: asyncio.create_task without error callbacks
- **Fix**: Add done_callback for exception handling
- **Ref**: Python Review ISSUE-02

### HIGH-03: Fix MongoDB Connection Cleanup
- **File**: `main.py:501-502`
- **Issue**: Tasks cancelled but not awaited, MongoDB sync close
- **Fix**: Properly await task cancellation, handle exceptions
- **Ref**: Python Review ISSUE-03

### MEDIUM-04: Refactor Global State to App State
- **File**: `main.py:56-78`
- **Issue**: 10+ global variables with `global` keyword
- **Fix**: Create AppState class, use dependency injection
- **Ref**: Python Review ISSUE-04

### MEDIUM-05: Fix Incomplete Backtest with Dummy Data
- **File**: `tasks/backtest_tasks.py:74-113`
- **Issue**: Returns random fake results (3 TODOs)
- **Fix**: Either implement real backtest or return 501 error
- **Ref**: Python Review ISSUE-05

### MEDIUM-06: Add Type Hints to Critical Functions
- **Files**:
  - `main.py:1206-1250` - OpenAI rate limiting
  - `models/model_manager.py:122-162` - predict()
  - `features/feature_engineering.py:248-288` - prepare_for_inference()
- **Fix**: Add return type hints, use Optional correctly
- **Ref**: Python Review ISSUE-06

### MEDIUM-07: Add Redis Reconnection Logic
- **File**: `utils/redis_cache.py:30-48`
- **Issue**: Connection failure sets `_redis = None` with no retry
- **Fix**: Add exponential backoff retry, health check
- **Ref**: Python Review ISSUE-07

### MEDIUM-08: Add Model Hash Verification
- **File**: `models/model_manager.py:216-255`
- **Issue**: No checksum to detect corrupted/tampered models
- **Fix**: Add SHA256 hash verification on model load
- **Ref**: Python Review ISSUE-11

### MEDIUM-09: Bundle Scaler with Model Metadata
- **Files**: `features/feature_engineering.py:224-246`, `models/model_manager.py:276-287`
- **Issue**: Scaler saved separately, can be orphaned
- **Fix**: Include scaler state in model metadata JSON
- **Ref**: Python Review ISSUE-12

### MEDIUM-10: Add ML Input Validation
- **File**: `models/model_manager.py:122-162`
- **Issue**: No validation of DataFrame schema before prediction
- **Fix**: Validate required columns, check shape matches model
- **Ref**: Python Review ISSUE-13

### LOW-11: Standardize Error Response Format
- **File**: `main.py` (various endpoints)
- **Issue**: Mix of `{"error": ...}`, `{"detail": ...}`, HTTPException
- **Fix**: Create ErrorResponse model, use consistently
- **Ref**: Python Review ISSUE-14

### LOW-12: Extract Magic Numbers
- **File**: `features/feature_engineering.py:207-214`
- **Issue**: Hardcoded 0.005, -0.005, 50 thresholds
- **Fix**: Move to configurable settings
- **Ref**: Python Review ISSUE-09

---

## Related Code Files

```
python-ai-service/
├── main.py                         # Global state, task handlers, shutdown
├── tasks/
│   ├── monitoring.py               # Bare except fix
│   └── backtest_tasks.py           # Dummy data fix
├── models/
│   └── model_manager.py            # Model verification, input validation
├── features/
│   └── feature_engineering.py      # Magic numbers, scaler bundling
├── utils/
│   └── redis_cache.py              # Reconnection logic
├── app/
│   └── core/
│       └── state.py                # NEW: AppState class
└── error_handlers.py               # NEW: Standardized error responses
```

---

## Implementation Steps

### Step 1: Fix Bare Except Handlers
```python
# In tasks/monitoring.py:148
# FROM:
except:
    pass

# TO:
except (httpx.ConnectError, httpx.RemoteProtocolError) as retry_err:
    logger.warning(f"Retry failed for {service_name}: {retry_err}")
except Exception as e:
    logger.error(f"Unexpected error in health check: {e}", exc_info=True)
```

### Step 2: Add Task Error Handlers
```python
# In main.py
def handle_task_exception(task: asyncio.Task):
    try:
        task.result()
    except asyncio.CancelledError:
        pass  # Expected on shutdown
    except Exception as e:
        logger.error(f"Background task {task.get_name()} failed: {e}", exc_info=True)

settings_refresh_task = asyncio.create_task(refresh_settings_periodically())
settings_refresh_task.add_done_callback(handle_task_exception)
```

### Step 3: Create AppState Class
```python
# NEW FILE: app/core/state.py
from dataclasses import dataclass
from typing import Optional, Set
from motor.motor_asyncio import AsyncIOMotorClient
from fastapi import WebSocket

@dataclass
class AppState:
    openai_client: Optional[Any] = None
    mongodb_client: Optional[AsyncIOMotorClient] = None
    mongodb_db: Optional[Any] = None
    websocket_connections: Set[WebSocket] = field(default_factory=set)
    rate_limiter: RateLimiter = field(default_factory=RateLimiter)
    cost_tracker: CostTracker = field(default_factory=CostTracker)

# In main.py:
app.state.app_state = AppState()
```

### Step 4: Fix Backtest Dummy Data
```python
# In tasks/backtest_tasks.py:74
# Option: Return 501 until implemented
raise HTTPException(
    status_code=501,
    detail={
        "success": False,
        "error": "Backtest feature not yet implemented",
        "warning": "DO NOT use simulated results for trading decisions"
    }
)
```

### Step 5: Add Model Hash Verification
```python
# In models/model_manager.py
import hashlib

def _calculate_file_hash(self, filepath: str) -> str:
    sha256_hash = hashlib.sha256()
    with open(filepath, "rb") as f:
        for byte_block in iter(lambda: f.read(4096), b""):
            sha256_hash.update(byte_block)
    return sha256_hash.hexdigest()

def load_model(self, model_path: Optional[str] = None) -> bool:
    # ... existing code ...

    # Verify hash
    expected_hash = self.model_metadata.get("model_hash")
    if expected_hash:
        actual_hash = self._calculate_file_hash(model_path)
        if actual_hash != expected_hash:
            logger.error(f"Model hash mismatch: expected {expected_hash[:16]}...")
            return False
```

### Step 6: Add ML Input Validation
```python
# In models/model_manager.py:predict()
def predict(self, df: pd.DataFrame) -> Dict[str, Any]:
    # Validate input
    required_cols = ["timestamp", "open", "high", "low", "close", "volume"]
    missing_cols = [c for c in required_cols if c not in df.columns]
    if missing_cols:
        raise ValueError(f"Missing required columns: {missing_cols}")

    min_rows = self.config.get("sequence_length", 60)
    if len(df) < min_rows:
        raise ValueError(f"Insufficient data: need {min_rows} rows, got {len(df)}")
```

---

## Todo List

- [ ] Fix bare except in tasks/monitoring.py:148
- [ ] Check tests/test_feature_engineering.py:272,356 for bare except
- [ ] Add done_callback to settings_refresh_task in main.py
- [ ] Add done_callback to analysis_task in main.py
- [ ] Fix MongoDB cleanup in main.py shutdown (await tasks)
- [ ] Create app/core/state.py with AppState class
- [ ] Refactor main.py to use AppState instead of globals
- [ ] Fix backtest_tasks.py to return 501 or warning
- [ ] Add type hints to handle_openai_rate_limit()
- [ ] Add type hints to model_manager.predict()
- [ ] Add type hints to prepare_for_inference()
- [ ] Add retry logic to redis_cache.py connect()
- [ ] Add model hash to save_model() metadata
- [ ] Add hash verification to load_model()
- [ ] Bundle scaler state in model metadata
- [ ] Add input validation to predict()
- [ ] Create ErrorResponse Pydantic model
- [ ] Apply ErrorResponse to all exception handlers
- [ ] Extract magic numbers to config in feature_engineering.py
- [ ] Run pytest to verify all changes
- [ ] Run mypy for type checking

---

## Success Criteria

| Criteria | Metric | Target |
|----------|--------|--------|
| Bare except clauses | grep count | 0 |
| Global variables | grep "global " | 0 in main.py |
| Type hint coverage | mypy report | 95%+ |
| Background task handlers | code review | All tasks have callbacks |
| ML model verification | unit test | Hash check works |
| Test pass rate | pytest | 100% |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Global refactor breaks startup | Medium | Medium | Incremental changes, test each |
| Model hash breaks existing models | Low | Medium | Optional verification initially |
| Backtest 501 upsets users | Low | Low | Clear error message, timeline |

---

## Security Considerations

- Model hash prevents tampering/corruption attacks
- Global state refactor improves testability (easier security audits)
- Error messages should not leak internal paths/details
- Redis reconnection prevents cache poisoning via disconnection

---

## Estimated Completion

- **Exception handling fixes**: 0.5 day
- **Global state refactor**: 1 day
- **ML improvements (hash, validation)**: 1 day
- **Type hints + error formats**: 0.5 day
- **Testing + documentation**: 1 day

**Total**: 3-4 days
