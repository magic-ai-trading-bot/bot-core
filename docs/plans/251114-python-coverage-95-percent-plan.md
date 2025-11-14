# Python Coverage Improvement Plan: 34% → 95%+

**Status:** DRAFT
**Created:** 2025-11-14
**Priority:** HIGH
**Effort Estimate:** 40-60 hours

---

## Executive Summary

Current Python AI Service coverage is **34.2%** (647/1892 lines covered). Target is **95%+ coverage** per CLAUDE.md standards. This requires covering **1,152 additional lines** across 14 files.

**Gap Analysis:**
- Current: 34.2% (647 lines covered)
- Target: 95%+ (1,798 lines covered)
- Gap: **-60.8%** (1,151 lines to cover)

---

## Current Coverage Breakdown

### Files Prioritized by Impact (Lowest → Highest Coverage)

| File | Current | Missing Lines | Priority | Impact |
|------|---------|--------------|----------|--------|
| **models/model_manager.py** | 13.2% | 211 | P1 | HIGH |
| **models/transformer_model.py** | 15.3% | 122 | P2 | HIGH |
| **models/gru_model.py** | 18.1% | 95 | P2 | HIGH |
| **models/lstm_model.py** | 18.1% | 95 | P2 | HIGH |
| **utils/helpers.py** | 22.2% | 42 | P3 | MEDIUM |
| **main.py** | 25.2% | 625 | **P1** | **CRITICAL** |
| **utils/logger.py** | 50.0% | 8 | P4 | LOW |
| **features/technical_indicators.py** | 84.0% | 21 | P4 | LOW |
| **features/feature_engineering.py** | 84.2% | 26 | P4 | LOW |

**Total Missing:** 1,245 lines across 9 files

---

## Implementation Strategy

### Phase 1: Critical Path - main.py (Priority 1)

**Target:** 25% → 95% (+70% increase, 625 lines)
**Effort:** 20-25 hours
**Dependencies:** mongomock, fakeredis, pytest-asyncio, httpx

#### 1.1 Application Lifecycle Tests (Lines 247-344)

**Uncovered Areas:**
- Lines 252-280: MongoDB connection failure handling
- Lines 281-325: OpenAI client initialization with multiple API keys
- Lines 326-344: Background task startup/shutdown

**Test File:** `tests/test_main_lifecycle.py`

**Test Cases:**

```python
# Test 1: MongoDB Connection Failure (Lines 276-279)
async def test_lifespan_mongodb_connection_failure():
    """Test graceful handling when MongoDB fails to connect."""
    # Mock: AsyncIOMotorClient raises ConnectionFailure
    # Assert: mongodb_client = None, mongodb_db = None
    # Assert: Service continues (no crash)
    # Lines covered: 276-279

# Test 2: MongoDB Ping Timeout (Lines 267-268)
async def test_lifespan_mongodb_ping_timeout():
    """Test MongoDB ping command timeout."""
    # Mock: admin.command("ping") raises timeout
    # Assert: Error logged, fallback to None
    # Lines covered: 267-268

# Test 3: Multiple OpenAI API Keys (Lines 283-307)
async def test_lifespan_multiple_api_keys():
    """Test initialization with primary + backup keys."""
    # Mock: OPENAI_API_KEY + OPENAI_BACKUP_API_KEYS env vars
    # Assert: valid_api_keys list has 3+ keys
    # Assert: Backup keys available message logged
    # Lines covered: 283-307

# Test 4: Invalid OpenAI API Key Format (Lines 299-312)
async def test_lifespan_invalid_api_key_format():
    """Test filtering of invalid API keys (starts with 'your-')."""
    # Mock: OPENAI_API_KEY="your-api-key-here"
    # Assert: openai_client = None (filtered out)
    # Lines covered: 299-312

# Test 5: Direct OpenAI Client Initialization Failure (Lines 317-324)
async def test_lifespan_direct_client_init_failure():
    """Test DirectOpenAIClient initialization exception."""
    # Mock: DirectOpenAIClient.__init__ raises Exception
    # Assert: openai_client = None
    # Assert: Error logged
    # Lines covered: 317-324

# Test 6: Background Task Startup (Lines 332-335)
async def test_lifespan_background_task_creation():
    """Test periodic_analysis_runner task is created."""
    # Assert: asyncio.create_task called
    # Assert: Task is not cancelled during startup
    # Lines covered: 332-335

# Test 7: Graceful Shutdown with Active MongoDB (Lines 339-343)
async def test_lifespan_shutdown_with_mongodb():
    """Test shutdown closes MongoDB connection."""
    # Assert: analysis_task.cancel() called
    # Assert: mongodb_client.close() called
    # Lines covered: 339-343
```

**Effort:** 6 hours

---

#### 1.2 Periodic Analysis Background Task (Lines 187-244)

**Uncovered Areas:**
- Lines 191-230: Analysis loop logic
- Lines 196-219: Per-symbol analysis processing
- Lines 221-226: Rate limiting between symbols
- Lines 239-244: Error handling and retry logic

**Test File:** `tests/test_main_periodic_analysis.py`

**Test Cases:**

```python
# Test 8: Periodic Analysis Full Cycle (Lines 191-234)
async def test_periodic_analysis_full_cycle():
    """Test complete analysis cycle for all symbols."""
    # Mock: generate_dummy_market_data, GPTTradingAnalyzer
    # Assert: analyze_trading_signals called for each ANALYSIS_SYMBOLS
    # Assert: store_analysis_result called 8 times (8 symbols)
    # Assert: ws_manager.broadcast_signal called 8 times
    # Lines covered: 191-234

# Test 9: Analysis Symbol Failure Recovery (Lines 228-230)
async def test_periodic_analysis_symbol_failure_continue():
    """Test analysis continues after one symbol fails."""
    # Mock: analyze_trading_signals raises Exception for BTCUSDT
    # Assert: Error logged for BTCUSDT
    # Assert: Analysis continues for ETHUSDT, BNBUSDT, etc.
    # Assert: All 8 symbols attempted despite failure
    # Lines covered: 228-230

# Test 10: Rate Limiting Between Symbols (Lines 225-226)
async def test_periodic_analysis_rate_limit_delay():
    """Test 10-second delay between symbol analyses."""
    # Mock: asyncio.sleep
    # Assert: asyncio.sleep(10) called between each symbol
    # Lines covered: 225-226

# Test 11: Cycle Interval Wait (Lines 236-237)
async def test_periodic_analysis_cycle_interval():
    """Test wait for ANALYSIS_INTERVAL_MINUTES between cycles."""
    # Mock: asyncio.sleep
    # Assert: asyncio.sleep(ANALYSIS_INTERVAL_MINUTES * 60) called
    # Lines covered: 236-237

# Test 12: Task Cancellation (Lines 239-241)
async def test_periodic_analysis_cancelled_error():
    """Test task gracefully handles CancelledError."""
    # Mock: asyncio.CancelledError raised
    # Assert: Loop exits cleanly
    # Assert: "Periodic analysis task cancelled" logged
    # Lines covered: 239-241

# Test 13: General Exception Retry (Lines 242-244)
async def test_periodic_analysis_exception_retry():
    """Test retry after general exception."""
    # Mock: Unexpected exception raised
    # Assert: Error logged
    # Assert: asyncio.sleep(60) called (1 minute retry delay)
    # Lines covered: 242-244
```

**Effort:** 5 hours

---

#### 1.3 MongoDB Storage Functions (Lines 147-185)

**Uncovered Areas:**
- Lines 149-165: store_analysis_result with errors
- Lines 170-184: get_latest_analysis error handling

**Test File:** `tests/test_main_mongodb_storage.py`

**Test Cases:**

```python
# Test 14: Store Analysis MongoDB Unavailable (Lines 149-151)
async def test_store_analysis_mongodb_unavailable():
    """Test store when MongoDB is None."""
    # Mock: mongodb_db = None
    # Assert: Warning logged "MongoDB not available"
    # Assert: Function returns without error
    # Lines covered: 149-151

# Test 15: Store Analysis Insert Failure (Lines 153-165)
async def test_store_analysis_insert_exception():
    """Test exception during MongoDB insert_one."""
    # Mock: collection.insert_one raises Exception
    # Assert: Error logged with symbol name
    # Lines covered: 153-165

# Test 16: Get Latest Analysis Not Found (Lines 173-180)
async def test_get_latest_analysis_not_found():
    """Test when no analysis exists for symbol."""
    # Mock: collection.find_one returns None
    # Assert: Function returns None
    # Lines covered: 173-180

# Test 17: Get Latest Analysis Exception (Lines 182-184)
async def test_get_latest_analysis_exception():
    """Test exception during find_one."""
    # Mock: collection.find_one raises Exception
    # Assert: Error logged
    # Assert: Returns None
    # Lines covered: 182-184
```

**Effort:** 3 hours

---

#### 1.4 DirectOpenAIClient HTTP Logic (Lines 912-1074)

**Uncovered Areas:**
- Lines 921-938: API key cycling and rate limit key tracking
- Lines 949-1074: HTTP request handling with auto-fallback

**Test File:** `tests/test_main_openai_client.py`

**Test Cases:**

```python
# Test 18: Get Current API Key with Rate Limited Keys (Lines 923-933)
def test_direct_client_get_api_key_rate_limited():
    """Test cycling through keys when some are rate limited."""
    # Setup: 3 API keys, keys 0 and 1 are rate limited
    # Assert: Returns key 2 (only available key)
    # Lines covered: 923-933

# Test 19: All Keys Rate Limited Recovery (Lines 929-933)
def test_direct_client_all_keys_rate_limited():
    """Test clearing rate limited keys when all exhausted."""
    # Setup: All 3 keys rate limited
    # Assert: rate_limited_keys.clear() called
    # Assert: Returns first key (retry)
    # Lines covered: 929-933

# Test 20: Rate Limit Delay Check (Lines 957-991)
async def test_direct_client_rate_limit_delay():
    """Test enforces OPENAI_REQUEST_DELAY between requests."""
    # Mock: last_openai_request_time = 15 seconds ago
    # Assert: asyncio.sleep(5) called (20s - 15s = 5s remaining)
    # Lines covered: 957-991

# Test 21: HTTP 429 Rate Limit with Retry-After Header (Lines 1017-1029)
async def test_direct_client_http_429_retry_after():
    """Test 429 response with retry-after header."""
    # Mock: httpx response 429, retry-after: 3600
    # Assert: OPENAI_RATE_LIMIT_RESET_TIME set to +1 hour
    # Assert: rate_limited_keys.add(key_index)
    # Assert: Tries next key
    # Lines covered: 1017-1029

# Test 22: HTTP 429 Rate Limit Without Retry-After (Lines 1030-1036)
async def test_direct_client_http_429_no_retry_after():
    """Test 429 response without retry-after header."""
    # Mock: httpx response 429, no retry-after header
    # Assert: OPENAI_RATE_LIMIT_RESET_TIME set to +1 hour (default)
    # Lines covered: 1030-1036

# Test 23: HTTP 401 Authentication Failure (Lines 1052-1055)
async def test_direct_client_http_401_auth_failure():
    """Test 401 authentication failure."""
    # Mock: httpx response 401
    # Assert: Error logged "authentication failed"
    # Assert: current_key_index incremented (try next key)
    # Lines covered: 1052-1055

# Test 24: HTTP 403 Quota Exceeded (Lines 1056-1059)
async def test_direct_client_http_403_quota():
    """Test 403 quota exceeded."""
    # Mock: httpx response 403
    # Assert: Error logged "quota exceeded"
    # Assert: Tries next key
    # Lines covered: 1056-1059

# Test 25: Network Error Retry (Lines 1065-1071)
async def test_direct_client_network_error_retry():
    """Test network error on non-final attempt."""
    # Mock: httpx.RequestError on attempt 1 of 3
    # Assert: Error logged
    # Assert: current_key_index incremented
    # Assert: Continues to next key
    # Lines covered: 1065-1071

# Test 26: All Keys Exhausted Exception (Lines 1073-1074)
async def test_direct_client_all_keys_exhausted():
    """Test exception when all keys fail."""
    # Mock: All 3 keys return errors
    # Assert: Raises Exception "All API keys exhausted or rate limited"
    # Lines covered: 1073-1074
```

**Effort:** 6 hours

---

#### 1.5 TechnicalAnalyzer Edge Cases (Lines 565-906)

**Uncovered Areas:**
- Lines 571-589: prepare_dataframe with empty klines
- Lines 768-815: Pattern detection logic
- Lines 830-875: Market context generation

**Test File:** `tests/test_main_technical_analyzer.py`

**Test Cases:**

```python
# Test 27: Prepare DataFrame Empty Klines (Lines 571-572)
def test_technical_analyzer_empty_klines():
    """Test prepare_dataframe with empty input."""
    # Input: []
    # Assert: Returns empty DataFrame
    # Lines covered: 571-572

# Test 28: Detect Patterns Double Top (Lines 777-782)
def test_technical_analyzer_detect_double_top():
    """Test double top pattern detection."""
    # Setup: DataFrame with two similar highs
    # Assert: patterns["double_top"] = True
    # Lines covered: 777-782

# Test 29: Detect Patterns Double Bottom (Lines 784-789)
def test_technical_analyzer_detect_double_bottom():
    """Test double bottom pattern detection."""
    # Setup: DataFrame with two similar lows
    # Assert: patterns["double_bottom"] = True
    # Lines covered: 784-789

# Test 30: Detect Patterns Ascending Triangle (Lines 791-801)
def test_technical_analyzer_ascending_triangle():
    """Test ascending triangle pattern."""
    # Setup: Higher lows, flat resistance
    # Assert: patterns["ascending_triangle"] = True
    # Lines covered: 791-801

# Test 31: Detect Patterns Descending Triangle (Lines 803-811)
def test_technical_analyzer_descending_triangle():
    """Test descending triangle pattern."""
    # Setup: Lower highs, flat support
    # Assert: patterns["descending_triangle"] = True
    # Lines covered: 803-811

# Test 32: Detect Patterns Exception Handling (Lines 813-814)
def test_technical_analyzer_detect_patterns_exception():
    """Test exception handling in pattern detection."""
    # Mock: df raises exception during processing
    # Assert: Warning logged
    # Assert: Returns default patterns (all False)
    # Lines covered: 813-814

# Test 33: Market Context RSI Oversold (Lines 836-841)
def test_technical_analyzer_market_context_rsi_oversold():
    """Test market context with RSI < 30."""
    # Setup: indicators["rsi"] = 25
    # Assert: trend_strength = -0.8
    # Assert: market_sentiment = "bearish"
    # Lines covered: 836-841

# Test 34: Market Context MACD Bullish Crossover (Lines 862-866)
def test_technical_analyzer_market_context_macd_bullish():
    """Test bullish MACD crossover."""
    # Setup: macd_histogram > 0, ema_9 > ema_21
    # Assert: market_sentiment = "bullish"
    # Lines covered: 862-866

# Test 35: Market Context MACD Bearish Crossover (Lines 867-870)
def test_technical_analyzer_market_context_macd_bearish():
    """Test bearish MACD crossover."""
    # Setup: macd_histogram < 0, ema_9 < ema_21
    # Assert: market_sentiment = "bearish"
    # Lines covered: 867-870

# Test 36: Market Context Exception Handling (Lines 872-873)
def test_technical_analyzer_market_context_exception():
    """Test exception handling in market context."""
    # Mock: df raises exception
    # Assert: Warning logged
    # Lines covered: 872-873
```

**Effort:** 4 hours

---

### Phase 2: ML Models (Priority 2)

**Target:** Models coverage 13-18% → 90%+
**Effort:** 15-20 hours

#### 2.1 model_manager.py (13.2% → 90%, 211 lines)

**Test File:** `tests/test_model_manager_complete.py`

**Critical Missing Areas:**
- Lines 23-58: ModelManager initialization
- Lines 62-119: load_model with error handling
- Lines 123-173: train_model lifecycle
- Lines 177-213: predict with preprocessing
- Lines 217-273: save_model edge cases

**Test Cases:**
- Test model file not found (Lines 23-25, 65-66)
- Test invalid model format (Lines 68-70)
- Test training with insufficient data (Lines 125-131)
- Test training timeout/memory errors (Lines 150-155)
- Test prediction with missing scaler (Lines 178-181)
- Test prediction shape mismatch (Lines 185-191)
- Test save_model directory creation (Lines 259-262)
- Test save_model permission denied (Lines 264-266)

**Effort:** 8 hours

---

#### 2.2 transformer_model.py (15.3% → 90%, 122 lines)

**Test File:** `tests/test_transformer_complete.py`

**Critical Missing Areas:**
- Lines 30-51: Model architecture build
- Lines 55-112: Training loop with callbacks
- Lines 123-166: Prediction with preprocessing
- Lines 170-195: Model persistence

**Test Cases:**
- Test build with invalid input shape (Lines 30-32)
- Test training with early stopping (Lines 78-87)
- Test training with reduce LR on plateau (Lines 90-96)
- Test predict with uninitialized model (Lines 123-125)
- Test save/load model with missing files (Lines 170-177)

**Effort:** 6 hours

---

#### 2.3 gru_model.py & lstm_model.py (18.1% → 90%, 95 lines each)

**Test Files:**
- `tests/test_gru_complete.py`
- `tests/test_lstm_complete.py`

**Critical Missing Areas (similar for both):**
- Lines 23-25, 29-32: Model initialization
- Lines 69-83: Build architecture with layers
- Lines 94-166: Training with validation
- Lines 170-212: Prediction pipeline
- Lines 216-238: Model save/load

**Test Cases (each model):**
- Test build with different layer configs
- Test training with validation split
- Test training convergence monitoring
- Test predict with batch processing
- Test model checkpointing

**Effort:** 3 hours each (6 hours total)

---

### Phase 3: Utilities & Features (Priority 3-4)

**Target:** Remaining files to 90%+
**Effort:** 8-10 hours

#### 3.1 utils/helpers.py (22.2% → 90%, 42 lines)

**Test File:** `tests/test_helpers_complete.py`

**Missing Areas:**
- Lines 10, 15, 18-36: Utility functions
- Lines 43, 46-52: Data processing helpers
- Lines 57, 66, 71-78: Validation functions
- Lines 89-95, 100, 105-107, 112: Edge case handling

**Effort:** 3 hours

---

#### 3.2 utils/logger.py (50.0% → 90%, 8 lines)

**Test File:** `tests/test_logger_complete.py`

**Missing Areas:**
- Lines 10, 13, 16-17, 20, 31, 43, 50: Logger initialization and config

**Effort:** 2 hours

---

#### 3.3 features/* (84%+ → 95%, 47 lines total)

**Test Files:**
- `tests/test_feature_engineering_complete.py` (26 lines)
- `tests/test_technical_indicators_complete.py` (21 lines)

**Missing Areas:**
- feature_engineering.py: Lines 121-123, 166, 169-189, 262-282, 312, 316
- technical_indicators.py: Lines 29-31, 57-59, 77-79, 104-106, 132, 137, 139, 237, 241, 293, 300-302

**Effort:** 3 hours

---

## Testing Infrastructure

### Required Mocks & Fixtures

```python
# conftest.py additions

@pytest.fixture
def mock_mongodb_with_failures():
    """MongoDB mock that simulates various failures."""
    # Connection timeout
    # Ping failure
    # Insert/find exceptions

@pytest.fixture
def mock_openai_multi_keys():
    """OpenAI client with 3 API keys."""
    # Primary key: valid
    # Backup key 1: rate limited
    # Backup key 2: valid

@pytest.fixture
def mock_httpx_responses():
    """Mock httpx responses for DirectOpenAIClient."""
    # 200 success
    # 429 rate limit with/without retry-after
    # 401 auth failure
    # 403 quota exceeded
    # Network errors
```

### Test Execution Strategy

**Phase 1 Tests (main.py):**
```bash
# Run incrementally as tests are written
pytest tests/test_main_lifecycle.py -v --cov=main --cov-report=term-missing
pytest tests/test_main_periodic_analysis.py -v --cov=main --cov-report=term-missing
pytest tests/test_main_mongodb_storage.py -v --cov=main --cov-report=term-missing
pytest tests/test_main_openai_client.py -v --cov=main --cov-report=term-missing
pytest tests/test_main_technical_analyzer.py -v --cov=main --cov-report=term-missing
```

**Full Coverage Check:**
```bash
# After all tests written, verify overall coverage
pytest --cov --cov-report=html --cov-report=term-missing
# Target: 95%+ overall, 90%+ per file
```

---

## Acceptance Criteria

### Overall Targets
- ✅ **Overall coverage: ≥95%** (current: 34.2%)
- ✅ **main.py coverage: ≥95%** (current: 25.2%)
- ✅ **models/* coverage: ≥90%** (current: 13-18%)
- ✅ **utils/* coverage: ≥90%** (current: 22-50%)
- ✅ **features/* coverage: ≥95%** (current: 84%)

### Quality Gates
- ✅ All new tests pass
- ✅ No flaky tests (100% pass rate)
- ✅ Test execution time: <5 minutes
- ✅ Zero regression in existing tests
- ✅ Mutation score maintained: ≥75%

---

## Risk Assessment

### High Risk Areas
1. **main.py lifecycle tests** - Complex async behavior, multiple dependencies
   - Mitigation: Use pytest-asyncio, comprehensive mocks

2. **DirectOpenAIClient HTTP logic** - Network errors, rate limiting
   - Mitigation: Mock httpx thoroughly, test all error codes

3. **Model training tests** - Can be slow, resource-intensive
   - Mitigation: Use small datasets, mock TensorFlow operations

### Medium Risk Areas
1. **MongoDB storage tests** - Requires mongomock setup
   - Mitigation: Use existing conftest.py fixtures

2. **Periodic analysis background task** - Timing-sensitive
   - Mitigation: Mock asyncio.sleep, use asyncio.wait_for

---

## Implementation Order

### Week 1: Critical Path (main.py)
- Day 1-2: Lifecycle tests (Tests 1-7)
- Day 3: Periodic analysis tests (Tests 8-13)
- Day 4: MongoDB storage tests (Tests 14-17)
- Day 5: OpenAI client tests (Tests 18-26)

### Week 2: Models
- Day 1-2: model_manager.py
- Day 3: transformer_model.py
- Day 4: gru_model.py
- Day 5: lstm_model.py

### Week 3: Utilities & Verification
- Day 1: helpers.py + logger.py
- Day 2: features/* edge cases
- Day 3: TechnicalAnalyzer edge cases (Tests 27-36)
- Day 4: Full coverage verification
- Day 5: Documentation & cleanup

---

## Progress Tracking

### Phase 1: main.py (0/36 tests)
- [ ] Test 1-7: Lifecycle (0/7)
- [ ] Test 8-13: Periodic analysis (0/6)
- [ ] Test 14-17: MongoDB storage (0/4)
- [ ] Test 18-26: OpenAI client (0/9)
- [ ] Test 27-36: TechnicalAnalyzer (0/10)

### Phase 2: Models (0/50 tests estimated)
- [ ] model_manager.py (0/20)
- [ ] transformer_model.py (0/12)
- [ ] gru_model.py (0/9)
- [ ] lstm_model.py (0/9)

### Phase 3: Utilities (0/20 tests estimated)
- [ ] helpers.py (0/10)
- [ ] logger.py (0/4)
- [ ] features/* (0/6)

---

## Verification Commands

```bash
# Check current coverage
cd python-ai-service
pytest --cov --cov-report=term-missing

# Generate detailed HTML report
pytest --cov --cov-report=html
open htmlcov/index.html

# Check specific file coverage
pytest --cov=main --cov-report=term-missing tests/test_main*.py

# Verify mutation score maintained
mutmut run
mutmut results
```

---

## Dependencies

**Required Packages:**
- pytest ✅ (installed)
- pytest-asyncio ✅ (installed)
- pytest-cov ✅ (installed)
- mongomock ✅ (installed)
- fakeredis ✅ (installed)
- httpx ✅ (installed)
- pytest-mock ✅ (installed)

**New Dependencies:**
- None required (all available)

---

## Unresolved Questions

1. **Model training tests:** Should we mock TensorFlow entirely or use small real datasets?
   - **Recommendation:** Use small real datasets (100 samples) for integration value

2. **Background task tests:** How to test infinite loop without actual delays?
   - **Recommendation:** Mock asyncio.sleep, break loop after 1-2 iterations

3. **Performance impact:** Will 100+ new tests significantly slow CI/CD?
   - **Recommendation:** Optimize by using minimal datasets, parallel test execution

4. **Mutation testing:** Will new tests improve mutation score?
   - **Recommendation:** Run mutation testing after Phase 1 to verify quality

---

## Success Metrics

### Before Implementation
- Overall Coverage: **34.2%**
- main.py Coverage: **25.2%**
- Models Coverage: **13-18%**
- Total Tests: **572**

### After Implementation (Target)
- Overall Coverage: **≥95%** (+60.8%)
- main.py Coverage: **≥95%** (+69.8%)
- Models Coverage: **≥90%** (+72-77%)
- Total Tests: **≥670** (+98 tests)
- Test Execution Time: **<5 minutes**
- Mutation Score: **≥75%** (maintained)

---

**Plan Status:** READY FOR IMPLEMENTATION
**Next Steps:**
1. Review plan with team
2. Begin Phase 1, Test 1 (Lifecycle tests)
3. Track progress daily
4. Adjust estimates after Phase 1 completion
