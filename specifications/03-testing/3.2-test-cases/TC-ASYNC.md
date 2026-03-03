# TC-ASYNC: Async Tasks System Test Cases

**Document ID**: TC-ASYNC
**Version**: 1.0
**Last Updated**: 2025-11-22
**Status**: Active
**Related FR**: FR-ASYNC-TASKS (Async Task Processing System)

---

## Table of Contents

1. [Test Case Summary](#test-case-summary)
2. [ML Tasks Test Cases](#ml-tasks-test-cases)
3. [Monitoring Tasks Test Cases](#monitoring-tasks-test-cases)
4. [AI Improvement Tasks Test Cases](#ai-improvement-tasks-test-cases)
5. [Backtest Tasks Test Cases](#backtest-tasks-test-cases)
6. [Error Handling Test Cases](#error-handling-test-cases)
7. [Performance & Load Test Cases](#performance--load-test-cases)
8. [Security Test Cases](#security-test-cases)
9. [Infrastructure Test Cases](#infrastructure-test-cases)
10. [Traceability Matrix](#traceability-matrix)

---

## Test Case Summary

| Category | Total Tests | Priority | Coverage |
|----------|-------------|----------|----------|
| ML Tasks | 30 | Critical | 100% |
| Monitoring Tasks | 20 | Critical | 100% |
| AI Improvement Tasks | 15 | Critical | 100% |
| Backtest Tasks | 10 | High | 100% |
| Error Handling | 10 | Critical | 100% |
| Performance & Load | 8 | High | 100% |
| Security | 6 | Critical | 100% |
| Infrastructure | 6 | Critical | 100% |
| **TOTAL** | **105** | - | **100%** |

**Test File Locations:**
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_celery_integration.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_ai_improvement_tasks.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_async_tasks_simple.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_data_storage.py`

**Execution Environment:**
- RabbitMQ 3.12+ running on localhost:5672
- Redis 7.0+ running on localhost:6379
- MongoDB 6.0+ running on localhost:27017
- Celery workers started with `celery -A celery_app worker`
- Celery Beat started with `celery -A celery_app beat`
- Flower monitoring on localhost:5555 (optional)

---

## ML Tasks Test Cases

### TC-ASYNC-001: Async Model Training - LSTM (Happy Path)

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-001
**Duration:** 5-10 minutes

**Prerequisites:**
- Celery workers running
- RabbitMQ healthy
- MongoDB contains training data for BTCUSDT (minimum 30 days @ 1h = 720 samples)
- Sufficient disk space (>10GB free)
- GPU available (optional, will fall back to CPU)

**Test Scenario (Gherkin):**
```gherkin
Feature: Async ML Model Training
  As a system administrator
  I want to train ML models asynchronously
  So that the API remains responsive during long-running training operations

  Scenario: Successfully train LSTM model
    Given Celery workers are running
    And training data exists for BTCUSDT (30 days)
    And disk space is sufficient (>10GB)
    When I trigger async training via API:
      """
      POST /api/tasks/train
      {
        "model_type": "lstm",
        "symbol": "BTCUSDT",
        "days_of_data": 30,
        "epochs": 100,
        "batch_size": 64
      }
      """
    Then I should receive response status 202 Accepted
    And response should contain task_id
    And task status should be "PENDING"
    When I poll task status every 10 seconds
    Then task status should transition: PENDING → PROGRESS → SUCCESS
    And final status should be "SUCCESS"
    And training_results.val_accuracy should be >= 0.60
    And model file should exist at returned model_path
    And training_jobs collection should have entry with task_id
    And no errors in worker logs
```

**Test Steps:**
1. **Setup**: Ensure MongoDB has BTCUSDT data (30 days, 1h candles)
2. **Trigger Training**:
   ```bash
   curl -X POST http://localhost:8000/api/tasks/train \
     -H "Content-Type: application/json" \
     -H "Authorization: Bearer $ADMIN_JWT_TOKEN" \
     -d '{
       "model_type": "lstm",
       "symbol": "BTCUSDT",
       "days_of_data": 30,
       "epochs": 100,
       "batch_size": 64
     }'
   ```
3. **Verify Initial Response**:
   - Status: 202 Accepted
   - Body contains: `{"task_id": "...", "status": "PENDING", "status_url": "/api/tasks/{task_id}"}`
4. **Poll Task Status**:
   ```bash
   while true; do
     STATUS=$(curl http://localhost:8000/api/tasks/{task_id} -H "Authorization: Bearer $JWT")
     echo $STATUS
     sleep 10
   done
   ```
5. **Monitor Progress**:
   - Progress should increase from 0% → 20% → 40% → 80% → 100%
   - Status messages: "Initializing..." → "Loading data..." → "Training..." → "Complete"
6. **Verify Final Result**:
   ```python
   final_result = {
     "status": "success",
     "model_type": "lstm",
     "symbol": "BTCUSDT",
     "training_results": {
       "final_loss": <float>,
       "validation_loss": <float>,
       "val_accuracy": 0.68,  # >= 0.60
       "training_time_seconds": 1847,
       "epochs_completed": 100,
       "best_epoch": 87
     },
     "model_path": "/models/lstm_BTCUSDT_1h_20251122_143022.h5",
     "task_id": "abc-123-def-456"
   }
   ```
7. **Verify Model File**:
   ```bash
   ls -lh /models/lstm_BTCUSDT_1h_*.h5
   # Should exist and be > 1MB
   ```
8. **Verify MongoDB Entry**:
   ```javascript
   db.training_jobs.findOne({"task_id": "abc-123-def-456"})
   // Should have: status="completed", val_accuracy>=0.60, timestamps
   ```

**Expected Results:**
- ✅ Task completes successfully within 10 minutes
- ✅ Final accuracy >= 60%
- ✅ Model file created with correct naming convention
- ✅ MongoDB entry created in `training_jobs` collection
- ✅ No errors in Celery worker logs
- ✅ Progress updates received every ~5%
- ✅ Training time logged (typically 1500-2000 seconds for 100 epochs)

**Assertions:**
```python
assert response.status_code == 202
assert "task_id" in response.json()
assert task_id is not None

# Wait for completion
result = celery_task.get(timeout=600)  # 10 min timeout

assert result["status"] == "success"
assert result["training_results"]["val_accuracy"] >= 0.60
assert result["training_results"]["epochs_completed"] == 100
assert os.path.exists(result["model_path"])
assert os.path.getsize(result["model_path"]) > 1_000_000  # > 1MB

# Verify MongoDB
doc = db.training_jobs.find_one({"task_id": task_id})
assert doc is not None
assert doc["status"] == "completed"
assert doc["model_type"] == "lstm"
assert doc["symbol"] == "BTCUSDT"
```

**Cleanup:**
```bash
# Delete test model file
rm /models/lstm_BTCUSDT_1h_*.h5

# Delete MongoDB entry
db.training_jobs.deleteOne({"task_id": "abc-123-def-456"})

# Clear Celery result backend
redis-cli DEL celery-task-meta-abc-123-def-456
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/ml_tasks.py:32-134`
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_async_tasks_simple.py`

---

### TC-ASYNC-002: Async Model Training - Invalid Parameters (Negative Test)

**Priority:** High
**Test Type:** Unit/Integration
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Training fails with invalid parameters
    Given I am authenticated as admin
    When I trigger training with epochs = 0
    Then API should return 400 Bad Request
    And error message should be "epochs must be between 1-1000"
    And no task should be queued
```

**Test Steps:**
1. Send request with invalid `epochs` parameter:
   ```bash
   curl -X POST http://localhost:8000/api/tasks/train \
     -H "Content-Type: application/json" \
     -d '{"model_type": "lstm", "symbol": "BTCUSDT", "epochs": 0}'
   ```
2. Verify response:
   - Status: 400 Bad Request
   - Error message: "epochs must be between 1-1000"

**Invalid Parameter Test Cases:**

| Parameter | Invalid Value | Expected Error |
|-----------|--------------|----------------|
| `model_type` | "invalid_model" | "model_type must be one of: lstm, gru, transformer" |
| `symbol` | "INVALID" | "Invalid trading symbol" |
| `epochs` | 0 | "epochs must be between 1-1000" |
| `epochs` | 1001 | "epochs must be between 1-1000" |
| `batch_size` | 33 | "batch_size must be power of 2 (32, 64, 128, 256)" |
| `days_of_data` | 3 | "days_of_data must be >= 7" |
| `learning_rate` | -0.1 | "learning_rate must be > 0" |
| `validation_split` | 0.6 | "validation_split must be between 0.0 and 0.5" |

**Expected Results:**
- ✅ All invalid parameters rejected before queuing task
- ✅ Proper error messages returned
- ✅ HTTP 400 status code
- ✅ No Celery task created

**Assertions:**
```python
for param, invalid_value, expected_error in test_cases:
    response = requests.post("/api/tasks/train", json={param: invalid_value})
    assert response.status_code == 400
    assert expected_error in response.json()["error"]

    # Verify no task was queued
    assert db.celery_task_meta.count_documents({"args": f"['{invalid_value}']"}) == 0
```

---

### TC-ASYNC-003: Out of Memory Handling During Training

**Priority:** Critical
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle out-of-memory error with auto-retry
    Given I trigger training with batch_size = 1024 (causes OOM)
    When training hits memory limit
    Then task should catch MemoryError
    And batch_size should be reduced to 512
    And training should automatically retry
    When retry also fails with OOM
    Then batch_size should be reduced to 256
    And retry again
    When batch_size reaches minimum (8) and still fails
    Then task should fail gracefully with clear error message
```

**Test Steps:**
1. Trigger training with very large batch size:
   ```python
   result = train_model.delay("lstm", "BTCUSDT", batch_size=1024)
   ```
2. Monitor for MemoryError in logs:
   ```
   [ERROR] Out of memory: CUDA out of memory. Tried to allocate 8.00 GiB
   [INFO] Reducing batch_size from 1024 to 512 and retrying...
   ```
3. Verify automatic retry with reduced batch size
4. Verify eventual success or graceful failure

**Expected Results:**
- ✅ MemoryError caught gracefully
- ✅ Batch size reduced automatically (halved each retry)
- ✅ Task retries up to 3 times
- ✅ If batch_size < 8, task fails with clear message
- ✅ No worker crashes

**Assertions:**
```python
# Simulate OOM error
with patch('tensorflow.keras.Model.fit') as mock_fit:
    mock_fit.side_effect = [
        MemoryError("CUDA out of memory"),  # First attempt
        MemoryError("CUDA out of memory"),  # Second attempt (batch=512)
        {"val_accuracy": 0.65}              # Third attempt (batch=256) succeeds
    ]

    result = train_model.apply(args=["lstm", "BTCUSDT"], kwargs={"batch_size": 1024}).get()

    assert result["status"] == "success"
    assert mock_fit.call_count == 3  # Original + 2 retries

    # Verify batch_size was reduced
    final_batch_size = mock_fit.call_args[1]["batch_size"]
    assert final_batch_size == 256  # 1024 -> 512 -> 256
```

**Code Location:**
- Error handling: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/ml_tasks.py:130-133`

---

### TC-ASYNC-004: Concurrent Model Training

**Priority:** High
**Test Type:** Load Test
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Train multiple models concurrently
    Given Celery has 2 ml_training workers
    When I queue 5 training tasks simultaneously
    Then 2 tasks should start immediately
    And 3 tasks should remain in queue
    When first 2 tasks complete
    Then next 2 tasks should start automatically
    And all 5 tasks should complete successfully
```

**Test Steps:**
1. Configure Celery workers:
   ```bash
   celery -A celery_app worker -Q ml_training -c 2  # 2 concurrent workers
   ```
2. Queue 5 training tasks:
   ```python
   tasks = []
   for symbol in ["BTCUSDT", "ETHUSDT", "BNBUSDT", "ADAUSDT", "SOLUSDT"]:
       task = train_model.delay("lstm", symbol, days_of_data=7, epochs=10)
       tasks.append(task)
   ```
3. Monitor queue depth:
   ```bash
   rabbitmqctl list_queues name messages
   # ml_training queue should show 3 pending tasks initially
   ```
4. Wait for all tasks to complete:
   ```python
   results = [task.get(timeout=3600) for task in tasks]
   ```
5. Verify all succeeded:
   ```python
   assert all(r["status"] == "success" for r in results)
   ```

**Expected Results:**
- ✅ Max 2 tasks run concurrently (worker concurrency limit)
- ✅ Remaining tasks queued in RabbitMQ
- ✅ Tasks start automatically as workers become available
- ✅ All 5 tasks complete successfully
- ✅ Total time < 2x single task time (parallelization benefit)

**Assertions:**
```python
import time
start = time.time()

# Queue 5 tasks
tasks = [train_model.delay("lstm", f"SYMBOL{i}", days_of_data=7, epochs=10) for i in range(5)]

# Wait for all
results = [task.get(timeout=3600) for task in tasks]
elapsed = time.time() - start

assert len(results) == 5
assert all(r["status"] == "success" for r in results)

# With 2 workers, should take ~3x single task time (not 5x)
single_task_time = 600  # 10 minutes
max_expected_time = single_task_time * 3  # 30 minutes
assert elapsed < max_expected_time
```

---

### TC-ASYNC-005: Training Cancellation

**Priority:** Medium
**Test Type:** Functional
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Cancel training task in progress
    Given I start training task for lstm model
    And task is currently at 40% progress (epoch 40/100)
    When I cancel the task via API
    Then task status should change to REVOKED
    And training should stop immediately
    And checkpoint should be saved at epoch 40
    And resources should be released (GPU memory)
    And partial model should not be deployed
```

**Test Steps:**
1. Start training:
   ```python
   task = train_model.delay("lstm", "BTCUSDT", epochs=100)
   ```
2. Wait until task reaches 40% progress
3. Cancel task:
   ```python
   task.revoke(terminate=True)
   ```
4. Verify task stopped:
   ```python
   assert task.state == "REVOKED"
   ```
5. Verify checkpoint saved:
   ```bash
   ls /checkpoints/lstm_BTCUSDT_*_epoch_40.ckpt
   ```

**Expected Results:**
- ✅ Task cancels within 10 seconds
- ✅ Checkpoint saved at last completed epoch
- ✅ GPU memory released
- ✅ No partial model deployed
- ✅ Worker ready for next task

**Assertions:**
```python
task = train_model.delay("lstm", "BTCUSDT", epochs=100)

# Wait for progress
while task.info.get("current", 0) < 40:
    time.sleep(1)

# Cancel
task.revoke(terminate=True)
time.sleep(5)

assert task.state in ["REVOKED", "FAILURE"]

# Verify checkpoint exists
checkpoint_path = f"/checkpoints/lstm_BTCUSDT_{task.id}_epoch_*.ckpt"
assert len(glob.glob(checkpoint_path)) > 0
```

---

### TC-ASYNC-006: Checkpoint Saving & Resume

**Priority:** High
**Test Type:** Functional
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Resume training from checkpoint after crash
    Given I start training for 100 epochs
    And training crashes at epoch 50
    And checkpoint was saved at epoch 50
    When I restart training with same parameters
    Then training should resume from epoch 51 (not epoch 1)
    And remaining epochs (51-100) should complete
    And final model should have 100 epochs total
```

**Test Steps:**
1. Configure checkpointing (save every 10 epochs)
2. Start training and simulate crash at epoch 50:
   ```python
   with patch('tensorflow.keras.Model.fit') as mock_fit:
       # Simulate crash after 50 epochs
       mock_fit.side_effect = RuntimeError("Worker killed")

       task = train_model.delay("lstm", "BTCUSDT", epochs=100, checkpoint_interval=10)
   ```
3. Verify checkpoint saved:
   ```bash
   ls /checkpoints/lstm_BTCUSDT_*_epoch_50.ckpt
   ```
4. Restart training:
   ```python
   task2 = train_model.delay("lstm", "BTCUSDT", epochs=100, resume_from_checkpoint=True)
   ```
5. Verify training resumes from epoch 51

**Expected Results:**
- ✅ Checkpoint saved every 10 epochs
- ✅ After crash, checkpoint found at epoch 50
- ✅ Resume starts at epoch 51
- ✅ Total training time saved (~50% faster)
- ✅ Final model identical to non-interrupted training

**Assertions:**
```python
# First run: crash at epoch 50
task1 = train_model.delay("lstm", "BTCUSDT", epochs=100)
# Simulate crash...

# Verify checkpoint
checkpoint = find_latest_checkpoint("lstm", "BTCUSDT")
assert checkpoint["epoch"] == 50

# Second run: resume
task2 = train_model.delay("lstm", "BTCUSDT", epochs=100, resume=True)
result = task2.get()

assert result["status"] == "success"
assert result["training_results"]["epochs_completed"] == 100
assert result["training_results"]["resumed_from_epoch"] == 51
```

---

### TC-ASYNC-007: GPU vs CPU Training

**Priority:** Medium
**Test Type:** Functional
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Training falls back to CPU when GPU unavailable
    Given GPU is not available (CUDA not installed or occupied)
    When I trigger training
    Then system should detect GPU unavailable
    And automatically fall back to CPU training
    And log warning "GPU not available, using CPU"
    And training should complete successfully (slower)
```

**Test Steps:**
1. Disable GPU (or test on CPU-only machine):
   ```bash
   export CUDA_VISIBLE_DEVICES=""
   ```
2. Trigger training:
   ```python
   task = train_model.delay("lstm", "BTCUSDT", epochs=10)
   ```
3. Monitor logs for GPU fallback message
4. Verify training completes on CPU

**Expected Results:**
- ✅ GPU unavailable detected gracefully
- ✅ Falls back to CPU automatically
- ✅ Warning logged: "GPU not available, using CPU"
- ✅ Training completes successfully (slower)
- ✅ CPU training time: ~3-5x slower than GPU

**Assertions:**
```python
import os
os.environ["CUDA_VISIBLE_DEVICES"] = ""

task = train_model.delay("lstm", "BTCUSDT", epochs=10)
result = task.get(timeout=1200)  # 20 min timeout (CPU slower)

assert result["status"] == "success"
assert "cpu" in result["training_results"]["device"].lower()
assert result["training_results"]["training_time_seconds"] > 300  # > 5 min (slower than GPU)
```

---

### TC-ASYNC-008: Training with Insufficient Data

**Priority:** High
**Test Type:** Validation
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Reject training when insufficient data
    Given MongoDB has only 5 days of data for ETHUSDT
    When I request training with days_of_data = 30
    Then validation should fail before queuing task
    And error message should be "Insufficient data: expected 720 samples, found 120"
    And no task should be queued
```

**Test Steps:**
1. Setup MongoDB with only 5 days of data (120 samples @ 1h):
   ```javascript
   db.market_data.deleteMany({"symbol": "ETHUSDT"})
   db.market_data.insertMany(generate_candles("ETHUSDT", days=5))
   ```
2. Request training for 30 days:
   ```bash
   curl -X POST /api/tasks/train -d '{"model_type": "lstm", "symbol": "ETHUSDT", "days_of_data": 30}'
   ```
3. Verify error response

**Expected Results:**
- ✅ Validation fails immediately (before queuing)
- ✅ Error message: "Insufficient data: expected 720 samples, found 120"
- ✅ HTTP 400 Bad Request
- ✅ No Celery task queued

**Assertions:**
```python
# Setup: Insert only 5 days of data
db.market_data.delete_many({"symbol": "ETHUSDT"})
db.market_data.insert_many(generate_candles("ETHUSDT", days=5))

# Request training
response = requests.post("/api/tasks/train", json={
    "model_type": "lstm",
    "symbol": "ETHUSDT",
    "days_of_data": 30
})

assert response.status_code == 400
assert "Insufficient data" in response.json()["error"]
assert "expected 720" in response.json()["error"]
assert "found 120" in response.json()["error"]

# Verify no task queued
time.sleep(2)
assert db.celery_task_meta.count_documents({"args": "['lstm', 'ETHUSDT']"}) == 0
```

---

### TC-ASYNC-009: Early Stopping (Training Divergence)

**Priority:** Medium
**Test Type:** Functional
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Stop training early when loss diverges
    Given I start training
    And initial loss is 0.05 at epoch 1
    When loss increases to 0.50 at epoch 20 (10x initial)
    Then early stopping should trigger
    And training should stop at epoch 20
    And error should be "Training diverged: loss increased 10x"
    And task status should be FAILURE
```

**Test Steps:**
1. Mock training to simulate divergence:
   ```python
   def mock_train_diverging():
       for epoch in range(100):
           if epoch < 10:
               yield {"loss": 0.05}
           else:
               yield {"loss": 0.50 + epoch * 0.1}  # Diverging
   ```
2. Trigger training with divergence detection enabled
3. Verify training stops early

**Expected Results:**
- ✅ Divergence detected (loss > 10x initial loss)
- ✅ Training stops at epoch where divergence detected
- ✅ Clear error message explaining divergence
- ✅ Checkpoint saved at last good epoch

**Assertions:**
```python
with patch('model_manager.ModelManager.train_model') as mock:
    # Simulate divergence
    mock.return_value = {
        "status": "failed",
        "error": "Training diverged: loss increased 10x",
        "epochs_completed": 20,
        "initial_loss": 0.05,
        "final_loss": 0.50
    }

    result = train_model.apply(args=["lstm", "BTCUSDT"]).get()

    assert result["status"] == "failed"
    assert "diverged" in result["error"].lower()
    assert result["training_results"]["epochs_completed"] < 100
```

---

### TC-ASYNC-010: Duplicate Training Prevention

**Priority:** Medium
**Test Type:** Functional
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Prevent duplicate training for same model+symbol
    Given LSTM training is in progress for BTCUSDT
    When I try to start another LSTM training for BTCUSDT
    Then request should be rejected
    And error should be "Training already in progress for lstm+BTCUSDT"
    And existing task should continue uninterrupted
```

**Test Steps:**
1. Start training:
   ```python
   task1 = train_model.delay("lstm", "BTCUSDT")
   ```
2. Immediately try to start duplicate:
   ```python
   task2 = train_model.delay("lstm", "BTCUSDT")
   ```
3. Verify rejection

**Expected Results:**
- ✅ Duplicate request rejected
- ✅ Error message: "Training already in progress"
- ✅ First task continues uninterrupted
- ✅ HTTP 409 Conflict

**Assertions:**
```python
# Start first task
task1 = train_model.delay("lstm", "BTCUSDT")
time.sleep(2)  # Let it start

# Try duplicate
with pytest.raises(DuplicateTaskError):
    task2 = train_model.delay("lstm", "BTCUSDT")

# Verify first task still running
assert task1.state == "PROGRESS"
```

---

### TC-ASYNC-011: Bulk Analysis - Multiple Symbols (Happy Path)

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-ASYNC-002

**Test Scenario (Gherkin):**
```gherkin
Feature: Bulk Analysis
  As a trader
  I want to analyze multiple symbols in parallel
  So that I can identify best trading opportunities quickly

  Scenario: Analyze 50 symbols successfully
    Given trained models exist for all symbols
    When I request bulk analysis for 50 symbols
    Then analysis should complete in < 60 seconds
    And results should be sorted by confidence (descending)
    And top result should have confidence >= 0.70
    And all 50 symbols should be analyzed
    And failed_symbols list should be empty
```

**Test Steps:**
1. Trigger bulk analysis:
   ```python
   symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", ...]  # 50 symbols
   task = bulk_analysis.delay(symbols, timeframe="1h", min_confidence=0.7)
   ```
2. Monitor progress:
   ```python
   while task.state == "PROGRESS":
       print(f"Progress: {task.info.get('current')}/{task.info.get('total')}")
       time.sleep(2)
   ```
3. Verify results

**Expected Results:**
- ✅ Completes in < 60 seconds
- ✅ All 50 symbols analyzed
- ✅ Results sorted by confidence (highest first)
- ✅ Top result confidence >= 0.70
- ✅ No failed symbols

**Assertions:**
```python
import time
symbols = [f"SYMBOL{i}USDT" for i in range(50)]

start = time.time()
task = bulk_analysis.delay(symbols, min_confidence=0.7)
result = task.get(timeout=120)
elapsed = time.time() - start

assert result["status"] == "success"
assert result["symbols_analyzed"] == 50
assert elapsed < 60  # < 60 seconds

# Verify sorted by confidence
confidences = [r["confidence"] for r in result["results"]]
assert confidences == sorted(confidences, reverse=True)

# Top result >= 0.70
assert result["results"][0]["confidence"] >= 0.70

# No failures
assert len(result["failed_symbols"]) == 0
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/ml_tasks.py:136-197`

---

### TC-ASYNC-012: Bulk Analysis - Graceful Failure Handling

**Priority:** High
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle individual symbol failures gracefully
    Given I request analysis for 10 symbols
    And 2 symbols have no data (INVALID1, INVALID2)
    When bulk analysis runs
    Then 8 symbols should analyze successfully
    And 2 symbols should be in failed_symbols list
    And task overall status should be "success"
    And results should contain only successful analyses
```

**Test Steps:**
1. Request analysis with some invalid symbols:
   ```python
   symbols = ["BTCUSDT", "ETHUSDT", "INVALID1", "BNBUSDT", "INVALID2", ...]
   task = bulk_analysis.delay(symbols)
   result = task.get()
   ```
2. Verify partial success

**Expected Results:**
- ✅ Task succeeds overall
- ✅ 8 valid symbols analyzed
- ✅ 2 invalid symbols in failed_symbols
- ✅ Results contain only successful analyses

**Assertions:**
```python
symbols = ["BTCUSDT", "ETHUSDT", "INVALID1", "BNBUSDT", "INVALID2", "ADAUSDT", "SOLUSDT", "DOTUSDT", "MATICUSDT", "AVAXUSDT"]

result = bulk_analysis.apply(args=[symbols]).get()

assert result["status"] == "success"
assert result["symbols_analyzed"] == 8
assert len(result["failed_symbols"]) == 2
assert "INVALID1" in result["failed_symbols"]
assert "INVALID2" in result["failed_symbols"]
assert len(result["results"]) == 8
```

---

### TC-ASYNC-013: Price Prediction Task

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-ASYNC-003

**Test Scenario (Gherkin):**
```gherkin
Feature: Price Prediction
  Scenario: Predict BTCUSDT price for 24 hours
    Given LSTM model is trained for BTCUSDT
    When I request price prediction for 24 hours
    Then predictions should return 24 hourly forecasts
    And each prediction should have: hour, predicted_price, confidence
    And confidence should decrease with time horizon
    And predictions should complete in < 10 seconds
```

**Test Steps:**
1. Trigger prediction:
   ```python
   task = predict_price.delay("BTCUSDT", model_type="lstm", horizon_hours=24)
   result = task.get(timeout=30)
   ```
2. Verify predictions

**Expected Results:**
- ✅ Returns 24 predictions
- ✅ Each has hour, price, confidence
- ✅ Confidence decreases over time (hour 1 > hour 24)
- ✅ Completes in < 10 seconds

**Assertions:**
```python
import time
start = time.time()

task = predict_price.delay("BTCUSDT", horizon_hours=24)
result = task.get(timeout=30)
elapsed = time.time() - start

assert result["status"] == "success"
assert len(result["predictions"]) == 24
assert elapsed < 10  # < 10 seconds

# Verify structure
for pred in result["predictions"]:
    assert "hour" in pred
    assert "predicted_price" in pred
    assert "confidence" in pred

# Confidence decreases
confidences = [p["confidence"] for p in result["predictions"]]
assert confidences[0] > confidences[-1]  # Hour 1 > Hour 24
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/ml_tasks.py:200-254`

---

### TC-ASYNC-014 to TC-ASYNC-030: Additional ML Task Tests

**TC-ASYNC-014:** Training with Different Timeframes (5m, 15m, 1h, 4h)
**TC-ASYNC-015:** Training with Custom Learning Rate
**TC-ASYNC-016:** Training GRU Model (verify model type)
**TC-ASYNC-017:** Training Transformer Model (verify architecture)
**TC-ASYNC-018:** Training with Retrain Flag (from scratch)
**TC-ASYNC-019:** Training with Incremental Update (transfer learning)
**TC-ASYNC-020:** Model Deployment After Training
**TC-ASYNC-021:** Training Progress Updates (every 5%)
**TC-ASYNC-022:** Training Email Notification on Completion
**TC-ASYNC-023:** Training Task Timeout (soft limit 2h, hard limit 2.5h)
**TC-ASYNC-024:** Training Metrics Logging to MongoDB
**TC-ASYNC-025:** Training with Validation Split (0.2)
**TC-ASYNC-026:** Training Disk Space Check (fail if <10GB)
**TC-ASYNC-027:** Corrupt Data Handling (NaN, Inf detection)
**TC-ASYNC-028:** Training Rate Limiting (max 10 per user per day)
**TC-ASYNC-029:** Training Authentication Check (admin only)
**TC-ASYNC-030:** Training Model File Encryption at Rest

---

## Monitoring Tasks Test Cases

### TC-ASYNC-031: System Health Check - All Services Healthy

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-004
**Duration:** < 30 seconds

**Test Scenario (Gherkin):**
```gherkin
Feature: System Health Monitoring
  As a system administrator
  I want automated health checks
  So that I can detect failures immediately

  Scenario: All services healthy
    Given all services are running (Rust, Python, Frontend, MongoDB, Redis, RabbitMQ)
    When health check task runs
    Then all 8 services should be marked "healthy"
    And overall_status should be "healthy"
    And alerts list should be empty
    And response_time_ms should be < 5000 for all HTTP services
    And no notifications should be sent
```

**Test Steps:**
1. Ensure all services running:
   ```bash
   # Rust API
   curl http://localhost:8080/api/health

   # Python API
   curl http://localhost:8000/health

   # Frontend
   curl http://localhost:3000/

   # MongoDB
   mongosh --eval "db.adminCommand('ping')"

   # Redis
   redis-cli ping

   # RabbitMQ
   rabbitmqctl status
   ```
2. Trigger health check:
   ```python
   from tasks.monitoring import system_health_check
   result = system_health_check.delay()
   health_report = result.get(timeout=60)
   ```
3. Verify all healthy

**Expected Results:**
- ✅ Task completes in < 30 seconds
- ✅ Overall status: "healthy"
- ✅ All 8 services status: "healthy"
- ✅ No alerts
- ✅ Response times < 5 seconds
- ✅ No email/Slack notifications sent

**Assertions:**
```python
result = system_health_check.apply().get()

assert result["status"] == "success"
assert result["health_report"]["overall_status"] == "healthy"

services = result["health_report"]["services"]
assert len(services) == 8

for service_name, service_data in services.items():
    assert service_data["status"] == "healthy", f"{service_name} is not healthy"

    # HTTP services should have response time
    if service_name in ["Rust Core Engine", "Python AI Service", "Frontend Dashboard"]:
        assert service_data["response_time_ms"] < 5000

assert len(result["health_report"]["alerts"]) == 0

# Verify no notification sent
assert notification_mock.call_count == 0
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/monitoring.py:66-218`

---

### TC-ASYNC-032: System Health Check - Service Down (Critical Alert)

**Priority:** Critical
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Critical alert when service down
    Given all services are running
    And MongoDB is stopped
    When health check runs
    Then MongoDB status should be "down"
    And overall_status should be "critical"
    And alerts should contain "❌ MongoDB is DOWN"
    And critical email should be sent to admin
    And Slack notification should be sent
```

**Test Steps:**
1. Stop MongoDB:
   ```bash
   sudo systemctl stop mongod
   # or
   docker stop mongodb-dev
   ```
2. Trigger health check:
   ```python
   result = system_health_check.delay().get()
   ```
3. Verify critical alert

**Expected Results:**
- ✅ MongoDB status: "down"
- ✅ Overall status: "critical"
- ✅ Alert message: "❌ MongoDB is DOWN"
- ✅ Email sent to admin within 30 seconds
- ✅ Slack notification sent

**Assertions:**
```python
# Stop MongoDB
subprocess.run(["docker", "stop", "mongodb-dev"])

result = system_health_check.apply().get()

assert result["health_report"]["overall_status"] == "critical"
assert result["health_report"]["services"]["MongoDB"]["status"] == "down"
assert any("MongoDB is DOWN" in alert for alert in result["health_report"]["alerts"])

# Verify notification sent
assert notification_mock.send_critical.called
assert "MongoDB" in notification_mock.send_critical.call_args[1]["title"]

# Cleanup: Restart MongoDB
subprocess.run(["docker", "start", "mongodb-dev"])
```

---

### TC-ASYNC-033: System Health Check - Degraded Status

**Priority:** High
**Test Type:** Functional
**Related FR:** FR-ASYNC-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Degraded status for slow service
    Given all services are running
    And Rust API response time is 8 seconds (slow)
    When health check runs
    Then Rust Core Engine status should be "healthy" but slow
    And overall_status should be "degraded"
    And alerts should contain "⚠️ Rust Core Engine slow response (8000ms)"
    And warning notification should be sent
```

**Test Steps:**
1. Mock Rust API to respond slowly:
   ```python
   with patch('requests.get') as mock_get:
       mock_get.return_value = Mock(
           status_code=200,
           elapsed=timedelta(seconds=8)
       )
       result = system_health_check.apply().get()
   ```
2. Verify degraded status

**Expected Results:**
- ✅ Overall status: "degraded"
- ✅ Warning alert generated
- ✅ Warning notification sent

**Assertions:**
```python
with patch('requests.get') as mock:
    response_mock = Mock(status_code=200)
    response_mock.elapsed.total_seconds.return_value = 8.0
    mock.return_value = response_mock

    result = system_health_check.apply().get()

    assert result["health_report"]["overall_status"] == "degraded"
    assert any("slow" in alert.lower() for alert in result["health_report"]["alerts"])
```

---

### TC-ASYNC-034: System Health Check - Disk Space Warning

**Priority:** High
**Test Type:** Functional
**Related FR:** FR-ASYNC-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Warning when disk usage high
    Given disk usage is 85%
    When health check runs
    Then alert should contain "⚠️ Disk usage high: 85%"
    And overall_status should be "degraded"
    And warning notification should be sent
```

**Test Steps:**
1. Mock disk usage:
   ```python
   with patch('subprocess.run') as mock_run:
       # Simulate 'df -h /' output
       mock_run.return_value = Mock(
           returncode=0,
           stdout="Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   85G   15G  85% /"
       )
       result = system_health_check.apply().get()
   ```
2. Verify warning

**Expected Results:**
- ✅ Alert: "⚠️ Disk usage high: 85%"
- ✅ Overall status: "degraded"
- ✅ Warning notification

**Assertions:**
```python
with patch('subprocess.run') as mock:
    mock.return_value = Mock(
        returncode=0,
        stdout="Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   85G   15G  85% /"
    )

    result = system_health_check.apply().get()

    assert result["health_report"]["overall_status"] == "degraded"
    assert any("85%" in alert for alert in result["health_report"]["alerts"])
```

---

### TC-ASYNC-035: System Health Check - Disk Space Critical

**Priority:** Critical
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Critical alert when disk usage > 90%
    Given disk usage is 95%
    When health check runs
    Then alert should be "❌ Disk usage critical: 95%"
    And overall_status should be "critical"
    And critical email should be sent immediately
```

**Expected Results:**
- ✅ Alert: "❌ Disk usage critical: 95%"
- ✅ Overall status: "critical"
- ✅ Critical notification sent

**Assertions:**
```python
with patch('subprocess.run') as mock:
    mock.return_value = Mock(
        returncode=0,
        stdout="Filesystem      Size  Used Avail Use% Mounted on\n/dev/sda1       100G   95G    5G  95% /"
    )

    result = system_health_check.apply().get()

    assert result["health_report"]["overall_status"] == "critical"
    assert any("95%" in alert and "❌" in alert for alert in result["health_report"]["alerts"])
```

---

### TC-ASYNC-036: Daily Portfolio Report

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-ASYNC-005

**Test Scenario (Gherkin):**
```gherkin
Feature: Daily Portfolio Reporting
  Scenario: Generate daily portfolio summary
    Given portfolio has balance $10,542.75
    And 47 total trades executed
    And 32 winning trades, 15 losing trades
    When daily report task runs at 8:00 AM UTC
    Then report should contain current balance
    And win_rate should be 68.09%
    And total_return_pct should be calculated
    And strategy breakdown should be included
    And email should be sent to admins
```

**Test Steps:**
1. Mock portfolio API response:
   ```python
   with patch('requests.get') as mock:
       mock.return_value = Mock(
           status_code=200,
           json=lambda: {
               "balance": 10542.75,
               "total_return_percent": 5.43,
               "total_trades": 47,
               "winning_trades": 32,
               "losing_trades": 15,
               "average_profit_per_trade": 1.87,
               "strategy_breakdown": {
                   "rsi": {"trades": 20, "win_rate": 70.0},
                   "macd": {"trades": 15, "win_rate": 66.7}
               }
           }
       )
       result = daily_portfolio_report.apply().get()
   ```
2. Verify report contents

**Expected Results:**
- ✅ Report contains all key metrics
- ✅ Win rate calculated: 68.09%
- ✅ Strategy breakdown included
- ✅ Email notification sent

**Assertions:**
```python
result = daily_portfolio_report.apply().get()

assert result["status"] == "success"
report = result["report"]

assert report["balance"] == 10542.75
assert report["total_trades"] == 47
assert report["winning_trades"] == 32
assert report["losing_trades"] == 15
assert report["win_rate"] == pytest.approx(68.09, abs=0.1)
assert report["avg_profit_per_trade"] == 1.87
assert "rsi" in report["strategies"]
assert "macd" in report["strategies"]
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/monitoring.py:227-298`

---

### TC-ASYNC-037: Daily API Cost Report - Within Budget

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-ASYNC-006

**Test Scenario (Gherkin):**
```gherkin
Feature: API Cost Monitoring
  Scenario: Daily cost within budget
    Given daily OpenAI API cost is $1.87
    And monthly projection is $56.10
    When daily cost report runs at 9:00 AM UTC
    Then report should show cost metrics
    And no warnings should be triggered
    And cost should be logged to MongoDB
```

**Expected Results:**
- ✅ Daily cost: $1.87 (< $2.00 warning threshold)
- ✅ Monthly projection: $56.10 (< $100 critical)
- ✅ No alerts
- ✅ Cost logged to MongoDB

**Assertions:**
```python
with patch('requests.get') as mock:
    mock.return_value = Mock(
        status_code=200,
        json=lambda: {
            "session": {"total_cost_usd": 1.87},
            "daily_cost_usd": 1.87,
            "monthly_cost_usd": 56.10
        }
    )

    result = daily_api_cost_report.apply().get()

    assert result["status"] == "success"
    assert result["report"]["session"]["total_cost_usd"] == 1.87
    assert len(result["report"]["alerts"]) == 0  # No warnings
```

---

### TC-ASYNC-038: Daily API Cost Report - Warning Threshold

**Priority:** Critical
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Warning when daily cost exceeds $2.00
    Given daily cost is $2.34
    When cost report runs
    Then alert should be "⚠️ WARNING: Daily cost $2.34 exceeds $2.00 warning"
    And email should be sent to admin
    And GPT-4 usage should be monitored closely
```

**Expected Results:**
- ✅ Warning alert generated
- ✅ Email notification sent
- ✅ Cost logged for trending

**Assertions:**
```python
with patch('requests.get') as mock:
    mock.return_value = Mock(
        status_code=200,
        json=lambda: {"daily_cost_usd": 2.34}
    )

    result = daily_api_cost_report.apply().get()

    assert any("$2.34 exceeds $2.00" in alert for alert in result["report"]["alerts"])
```

---

### TC-ASYNC-039: Daily API Cost Report - Critical Threshold

**Priority:** Critical
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Critical when daily cost exceeds $5.00
    Given daily cost is $6.50
    When cost report runs
    Then CRITICAL alert should be sent
    And GPT-4 API should be automatically disabled
    And admin should receive urgent email
    And Slack notification should include cost details
```

**Expected Results:**
- ✅ Critical alert sent
- ✅ GPT-4 disabled automatically
- ✅ Urgent email/Slack notification

**Assertions:**
```python
with patch('requests.get') as mock:
    mock.return_value = Mock(
        status_code=200,
        json=lambda: {"daily_cost_usd": 6.50, "monthly_cost_usd": 195.00}
    )

    result = daily_api_cost_report.apply().get()

    # Should have critical alert
    assert any("CRITICAL" in alert or "$5.00" in alert for alert in result["report"]["alerts"])

    # Verify GPT-4 disabled
    # (implementation-specific check)
```

---

### TC-ASYNC-040: Daily Performance Analysis - Good Performance

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-007

**Test Scenario (Gherkin):**
```gherkin
Feature: Performance Analysis
  Scenario: Good performance - no action needed
    Given last 7 days win rate is 72%
    And Sharpe ratio is 2.3
    And avg profit is 2.8%
    When performance analysis runs at 1:00 AM UTC
    Then all metrics should be above target
    And no GPT-4 analysis should be triggered
    And status should be logged as "GOOD"
```

**Expected Results:**
- ✅ Metrics above target (no degradation)
- ✅ No GPT-4 analysis triggered
- ✅ Status: "GOOD"

**Assertions:**
```python
with patch('requests.get') as mock:
    mock.return_value = Mock(
        status_code=200,
        json=lambda: {
            "trades": [
                # Mock 7 days of good trades
                {"profit_pct": 2.5, "win": True},
                {"profit_pct": 3.1, "win": True},
                # ... win_rate = 72%
            ]
        }
    )

    result = daily_performance_analysis.apply().get()

    assert result["metrics"]["win_rate"] >= 70.0
    assert result["metrics"]["sharpe_ratio"] >= 2.1
    assert result["gpt4_analysis_triggered"] == False
```

---

### TC-ASYNC-041: Daily Performance Analysis - Triggers GPT-4

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-007, FR-ASYNC-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Poor performance triggers GPT-4 analysis
    Given last 7 days win rate is 52% (below 55% critical)
    And Sharpe ratio is 0.9 (below 1.0 critical)
    When performance analysis runs
    Then performance degradation should be detected
    And GPT-4 analysis task should be queued
    And admin should be notified of degradation
```

**Expected Results:**
- ✅ Degradation detected
- ✅ GPT-4 analysis task queued
- ✅ Admin notification sent

**Assertions:**
```python
with patch('requests.get') as mock:
    mock.return_value = Mock(
        status_code=200,
        json=lambda: {
            "trades": generate_poor_trades(win_rate=0.52, sharpe=0.9)
        }
    )

    with patch('tasks.ai_improvement.gpt4_self_analysis.delay') as gpt4_mock:
        result = daily_performance_analysis.apply().get()

        assert result["metrics"]["win_rate"] < 55.0
        assert result["gpt4_analysis_triggered"] == True
        assert gpt4_mock.called
```

---

### TC-ASYNC-042 to TC-ASYNC-050: Additional Monitoring Tests

**TC-ASYNC-042:** Health Check - False Positive Prevention (2 consecutive failures)
**TC-ASYNC-043:** Health Check - Network Timeout Handling
**TC-ASYNC-044:** Health Check - All Services Down (don't crash task)
**TC-ASYNC-045:** Health Check - MongoDB Unreachable (log to file)
**TC-ASYNC-046:** Portfolio Report - No Trades (weekend handling)
**TC-ASYNC-047:** Portfolio Report - Email Delivery Failure
**TC-ASYNC-048:** Cost Report - OpenAI API Error Handling
**TC-ASYNC-049:** Cost Report - Monthly Projection Accuracy
**TC-ASYNC-050:** Performance Analysis - Incomplete Data Handling

---

## AI Improvement Tasks Test Cases

### TC-ASYNC-051: GPT-4 Self-Analysis - Decides NOT to Retrain

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-008

**Test Scenario (Gherkin):**
```gherkin
Feature: GPT-4 Self-Analysis
  As an autonomous AI system
  I want to analyze my own performance
  So that I can decide when retraining is needed

  Scenario: GPT-4 decides retraining not needed
    Given all models performing well (accuracy > 70%)
    And win rate is 72%, Sharpe 2.3
    When GPT-4 analysis runs at 3:00 AM UTC
    Then GPT-4 should analyze performance data
    And recommendation should be "NO_RETRAINING"
    And reasoning should mention "performing above threshold"
    And no retraining task should be queued
    And cost should be ~$0.024 (1 GPT-4 call)
```

**Test Steps:**
1. Mock performance data (good):
   ```python
   performance_data = {
       "win_rate": 72.0,
       "sharpe_ratio": 2.3,
       "avg_profit": 2.8,
       "model_accuracy": {"lstm": 0.72, "gru": 0.70, "transformer": 0.73}
   }
   ```
2. Mock GPT-4 response:
   ```python
   with patch('openai.ChatCompletion.create') as mock_gpt4:
       mock_gpt4.return_value = Mock(
           choices=[Mock(message=Mock(content=json.dumps({
               "recommendation": "NO_RETRAINING",
               "confidence": 0.85,
               "urgency": "low",
               "reasoning": "All models performing above target threshold (70%). Current win rate of 72% exceeds target. No immediate action needed."
           })))]
       )

       result = gpt4_self_analysis.apply().get()
   ```
3. Verify decision

**Expected Results:**
- ✅ GPT-4 analyzes data successfully
- ✅ Recommendation: "NO_RETRAINING"
- ✅ Confidence: >= 0.70
- ✅ No retraining task queued
- ✅ Cost: ~$0.024
- ✅ Analysis saved to MongoDB

**Assertions:**
```python
result = gpt4_self_analysis.apply().get()

assert result["status"] == "success"
assert result["analysis"]["recommendation"] == "NO_RETRAINING"
assert "performing above threshold" in result["analysis"]["reasoning"].lower()
assert result["analysis"]["retrain_triggered"] == False

# Verify no retrain task queued
assert db.celery_task_meta.count_documents({"task_name": "tasks.ai_improvement.adaptive_retrain"}) == 0

# Verify analysis saved
assert db.gpt4_analysis_history.count_documents({"task_id": result["task_id"]}) == 1
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/ai_improvement.py:64-246`

---

### TC-ASYNC-052: GPT-4 Self-Analysis - Decides TO Retrain

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: GPT-4 decides retraining needed
    Given LSTM accuracy dropped to 60% (below 65% threshold)
    And win rate is 58% (below 70% target)
    When GPT-4 analysis runs
    Then GPT-4 should detect performance degradation
    And recommendation should be "RETRAIN_LSTM"
    And confidence should be >= 0.75
    And urgency should be "high"
    And reasoning should mention "accuracy dropped to 60%"
    And adaptive retraining task should be queued
```

**Test Steps:**
1. Mock degraded performance:
   ```python
   performance_data = {
       "win_rate": 58.0,
       "model_accuracy": {"lstm": 0.60, "gru": 0.71, "transformer": 0.72}
   }
   ```
2. Mock GPT-4 response recommending retrain:
   ```python
   with patch('openai.ChatCompletion.create') as mock_gpt4:
       mock_gpt4.return_value = Mock(
           choices=[Mock(message=Mock(content=json.dumps({
               "recommendation": "RETRAIN_LSTM",
               "confidence": 0.88,
               "urgency": "high",
               "reasoning": "LSTM model accuracy dropped to 60%, below 65% threshold. Win rate decreased to 58%. Recommend immediate retraining of LSTM model."
           })))]
       )

       result = gpt4_self_analysis.apply().get()
   ```
3. Verify retraining triggered

**Expected Results:**
- ✅ GPT-4 recommendation: "RETRAIN_LSTM"
- ✅ Confidence: >= 0.75
- ✅ Urgency: "high"
- ✅ Adaptive retraining task queued
- ✅ Notification sent

**Assertions:**
```python
with patch('tasks.ai_improvement.adaptive_retrain.delay') as mock_retrain:
    result = gpt4_self_analysis.apply().get()

    assert result["status"] == "success"
    assert result["analysis"]["recommendation"] == "RETRAIN_LSTM"
    assert result["analysis"]["confidence"] >= 0.75
    assert result["analysis"]["urgency"] == "high"
    assert result["analysis"]["retrain_triggered"] == True

    # Verify retrain task queued
    assert mock_retrain.called
    assert "lstm" in mock_retrain.call_args[1]["model_types"]
```

---

### TC-ASYNC-053: GPT-4 Self-Analysis - OpenAI API Error

**Priority:** High
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle OpenAI API errors gracefully
    Given performance degradation detected
    When GPT-4 API call fails with timeout
    Then task should retry up to 3 times
    When all retries fail
    Then task should fail gracefully
    And error should be logged
    And admin should be notified of GPT-4 failure
    And fallback decision should be made (retrain if critical)
```

**Expected Results:**
- ✅ Retries 3 times
- ✅ Fails gracefully after max retries
- ✅ Error notification sent
- ✅ Fallback decision logic applied

**Assertions:**
```python
with patch('openai.ChatCompletion.create') as mock_gpt4:
    mock_gpt4.side_effect = openai.error.Timeout("Request timed out")

    with pytest.raises(Exception):
        gpt4_self_analysis.apply().get()

    # Verify retries
    assert mock_gpt4.call_count == 3  # Original + 2 retries
```

---

### TC-ASYNC-054: GPT-4 Self-Analysis - Invalid JSON Response

**Priority:** Medium
**Test Type:** Error Handling
**Related FR:** FR-ASYNC-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle invalid GPT-4 JSON response
    Given GPT-4 returns malformed JSON
    When parsing response
    Then JSONDecodeError should be caught
    And error should be logged with raw response
    And task should retry
```

**Expected Results:**
- ✅ JSONDecodeError caught
- ✅ Raw response logged
- ✅ Task retries

**Assertions:**
```python
with patch('openai.ChatCompletion.create') as mock_gpt4:
    mock_gpt4.return_value = Mock(
        choices=[Mock(message=Mock(content="This is not valid JSON"))]
    )

    with pytest.raises(json.JSONDecodeError):
        gpt4_self_analysis.apply().get()
```

---

### TC-ASYNC-055: GPT-4 Self-Analysis - Cost Tracking

**Priority:** Medium
**Test Type:** Functional
**Related FR:** FR-ASYNC-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Track GPT-4 API costs
    Given GPT-4 analysis runs
    When API call completes
    Then cost should be calculated based on tokens
    And cost should be logged to MongoDB
    And daily cost total should be updated
```

**Expected Results:**
- ✅ Cost calculated from tokens (input + output)
- ✅ Cost logged to MongoDB
- ✅ Daily total updated

**Assertions:**
```python
with patch('openai.ChatCompletion.create') as mock_gpt4:
    mock_gpt4.return_value = Mock(
        choices=[Mock(message=Mock(content='{"recommendation": "wait"}'))],
        usage=Mock(
            prompt_tokens=500,
            completion_tokens=200,
            total_tokens=700
        )
    )

    result = gpt4_self_analysis.apply().get()

    # Verify cost calculated (~$0.024 for 700 tokens)
    cost_entry = db.gpt4_costs.find_one({"task_id": result["task_id"]})
    assert cost_entry is not None
    assert cost_entry["total_tokens"] == 700
    assert 0.02 < cost_entry["cost_usd"] < 0.03
```

---

### TC-ASYNC-056: Adaptive Retraining - Successful Retraining

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-009

**Test Scenario (Gherkin):**
```gherkin
Feature: Adaptive Model Retraining
  Scenario: Retrain LSTM model successfully
    Given GPT-4 recommended retraining LSTM
    When adaptive_retrain task runs
    Then LSTM model should be retrained with latest data
    And validation accuracy should be compared with old model
    When new model accuracy > old model accuracy
    Then new model should be deployed
    And old model should be archived
    And notification should include before/after metrics
```

**Test Steps:**
1. Trigger adaptive retraining:
   ```python
   analysis_result = {
       "recommendation": "RETRAIN_LSTM",
       "reasoning": "LSTM accuracy dropped to 60%"
   }

   task = adaptive_retrain.delay(
       model_types=["lstm"],
       analysis_result=analysis_result
   )
   result = task.get(timeout=3600)
   ```
2. Verify retraining and deployment

**Expected Results:**
- ✅ LSTM retrained successfully
- ✅ New accuracy > old accuracy
- ✅ New model deployed
- ✅ Old model archived
- ✅ Notification sent with metrics

**Assertions:**
```python
with patch('models.model_manager.ModelManager.train_model') as mock_train:
    mock_train.return_value = {
        "val_accuracy": 0.68,  # Better than old 0.60
        "training_time_seconds": 1200
    }

    result = adaptive_retrain.apply(
        kwargs={
            "model_types": ["lstm"],
            "analysis_result": {"reasoning": "Test"}
        }
    ).get()

    assert result["status"] == "success"
    assert result["models"]["lstm"]["new_accuracy"] > result["models"]["lstm"]["old_accuracy"]
    assert result["models"]["lstm"]["deployed"] == True
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/ai_improvement.py:254-300+`

---

### TC-ASYNC-057: Adaptive Retraining - New Model Worse

**Priority:** High
**Test Type:** Functional
**Related FR:** FR-ASYNC-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Do not deploy if new model worse
    Given adaptive retraining completes
    And new model accuracy is 0.58
    And old model accuracy was 0.62
    When comparing models
    Then new model should NOT be deployed
    And old model should remain active
    And notification should explain why not deployed
```

**Expected Results:**
- ✅ New model not deployed (worse accuracy)
- ✅ Old model remains active
- ✅ Notification explains decision

**Assertions:**
```python
with patch('models.model_manager.ModelManager.train_model') as mock_train:
    mock_train.return_value = {"val_accuracy": 0.58}  # Worse

    result = adaptive_retrain.apply(
        kwargs={"model_types": ["lstm"], "analysis_result": {}}
    ).get()

    assert result["models"]["lstm"]["new_accuracy"] < result["models"]["lstm"]["old_accuracy"]
    assert result["models"]["lstm"]["deployed"] == False
    assert "not deployed" in result["models"]["lstm"]["reason"].lower()
```

---

### TC-ASYNC-058: Adaptive Retraining - Multiple Models

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-ASYNC-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Retrain multiple models (LSTM, GRU, Transformer)
    Given GPT-4 recommended retraining all models
    When adaptive_retrain runs
    Then all 3 models should be retrained sequentially
    And each model should be validated independently
    And only improved models should be deployed
    And summary should show before/after for all models
```

**Expected Results:**
- ✅ All 3 models retrained
- ✅ Each validated independently
- ✅ Only improved models deployed
- ✅ Summary shows all results

**Assertions:**
```python
result = adaptive_retrain.apply(
    kwargs={
        "model_types": ["lstm", "gru", "transformer"],
        "analysis_result": {}
    }
).get()

assert len(result["models"]) == 3
assert "lstm" in result["models"]
assert "gru" in result["models"]
assert "transformer" in result["models"]

# Each should have before/after metrics
for model in result["models"].values():
    assert "old_accuracy" in model
    assert "new_accuracy" in model
    assert "deployed" in model
```

---

### TC-ASYNC-059: Emergency Strategy Disable

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-ASYNC-010

**Test Scenario (Gherkin):**
```gherkin
Feature: Emergency Strategy Control
  Scenario: Disable failing strategy immediately
    Given RSI strategy win rate drops to 40% (critical)
    When emergency_strategy_disable task triggers
    Then RSI strategy should be disabled immediately
    And all pending RSI signals should be cancelled
    And admin should receive URGENT notification
    And reason should be logged: "Win rate 40% (critical threshold 45%)"
```

**Expected Results:**
- ✅ Strategy disabled immediately
- ✅ Pending signals cancelled
- ✅ URGENT notification sent
- ✅ Reason logged

**Assertions:**
```python
result = emergency_strategy_disable.apply(
    kwargs={
        "strategy_name": "rsi",
        "reason": "Win rate 40% (critical threshold 45%)"
    }
).get()

assert result["status"] == "success"
assert result["strategy_disabled"] == "rsi"
assert result["pending_signals_cancelled"] > 0

# Verify strategy disabled in Rust API
response = requests.get(f"{RUST_API_URL}/api/strategies/active")
active = response.json()
assert "rsi" not in active
```

---

### TC-ASYNC-060 to TC-ASYNC-065: Additional AI Improvement Tests

**TC-ASYNC-060:** GPT-4 Analysis - Skip if Performance Acceptable
**TC-ASYNC-061:** GPT-4 Analysis - Force Analysis Flag
**TC-ASYNC-062:** Adaptive Retrain - Data Freshness Check (use last 60 days)
**TC-ASYNC-063:** Adaptive Retrain - Concurrent Retraining Prevention
**TC-ASYNC-064:** Emergency Disable - Re-enable After Manual Review
**TC-ASYNC-065:** Emergency Disable - Multiple Strategies Simultaneously

---

## Backtest Tasks Test Cases

### TC-ASYNC-066: Strategy Backtesting - RSI Strategy

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-ASYNC-011

**Test Scenario (Gherkin):**
```gherkin
Feature: Strategy Backtesting
  Scenario: Backtest RSI strategy on 6 months data
    Given 6 months of BTCUSDT historical data exists
    When I trigger backtest for RSI strategy
    Then backtest should complete in < 20 minutes
    And results should include: win_rate, total_return, sharpe_ratio, max_drawdown
    And win_rate should be 50-65%
    And total trades should be 300-400
    And equity curve should have 184 data points (daily)
```

**Test Steps:**
1. Trigger backtest:
   ```python
   task = backtest_strategy.delay(
       strategy_name="rsi",
       symbol="BTCUSDT",
       start_date="2024-05-22",
       end_date="2024-11-22",
       parameters={"rsi_period": 14, "overbought": 70, "oversold": 30}
   )
   result = task.get(timeout=1200)
   ```
2. Verify results

**Expected Results:**
- ✅ Completes in < 20 minutes
- ✅ Win rate: 50-65%
- ✅ Total trades: 300-400
- ✅ Sharpe ratio: 1.0-2.5
- ✅ Results saved to MongoDB

**Assertions:**
```python
import time
start = time.time()

task = backtest_strategy.delay("rsi", "BTCUSDT", "2024-05-22", "2024-11-22")
result = task.get(timeout=1200)
elapsed = time.time() - start

assert result["status"] == "success"
assert elapsed < 1200  # < 20 minutes

results = result["results"]
assert 50 <= results["win_rate"] <= 65
assert 300 <= results["total_trades"] <= 400
assert 1.0 <= results["sharpe_ratio"] <= 2.5
assert 5 <= results["total_return"] <= 25
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/backtest_tasks.py:27-139`

---

### TC-ASYNC-067: Strategy Optimization - Grid Search

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-ASYNC-012

**Test Scenario (Gherkin):**
```gherkin
Feature: Strategy Optimization
  Scenario: Optimize RSI parameters via grid search
    Given I want to find best RSI parameters
    When I run optimization with 50 parameter variations
    Then 50 backtests should run with different parameters
    And results should be ranked by Sharpe ratio
    And best parameters should be returned
    And optimization should complete in < 2 hours
```

**Test Steps:**
1. Trigger optimization:
   ```python
   task = optimize_strategy.delay(
       strategy_name="rsi",
       symbol="BTCUSDT",
       start_date="2024-05-22",
       end_date="2024-11-22",
       parameter_variations=50
   )
   result = task.get(timeout=7200)
   ```
2. Verify optimization

**Expected Results:**
- ✅ 50 backtests run
- ✅ Results ranked by Sharpe ratio
- ✅ Best parameters returned
- ✅ Completes in < 2 hours

**Assertions:**
```python
task = optimize_strategy.delay("rsi", "BTCUSDT", "2024-05-22", "2024-11-22", parameter_variations=50)
result = task.get(timeout=7200)

assert result["status"] == "success"
assert len(result["all_results"]) == 50

# Verify sorted by Sharpe ratio
sharpe_ratios = [r["sharpe_ratio"] for r in result["all_results"]]
assert sharpe_ratios == sorted(sharpe_ratios, reverse=True)

# Best parameters
best = result["best_parameters"]
assert "rsi_period" in best
assert 10 <= best["rsi_period"] <= 20
```

**Code Location:**
- Task: `/Users/dungngo97/Documents/bot-core/python-ai-service/tasks/backtest_tasks.py:142-200+`

---

### TC-ASYNC-068 to TC-ASYNC-075: Additional Backtest Tests

**TC-ASYNC-068:** Backtest - MACD Strategy
**TC-ASYNC-069:** Backtest - Bollinger Bands Strategy
**TC-ASYNC-070:** Backtest - Volume Strategy
**TC-ASYNC-071:** Backtest - Progress Updates (every 10%)
**TC-ASYNC-072:** Backtest - Different Timeframes (5m, 15m, 1h, 4h)
**TC-ASYNC-073:** Backtest - Walk-Forward Optimization
**TC-ASYNC-074:** Backtest - Results Visualization (equity curve, drawdown)
**TC-ASYNC-075:** Optimization - Genetic Algorithm (vs Grid Search)

---

## Error Handling Test Cases

### TC-ASYNC-076: RabbitMQ Down During Task Queue

**Priority:** Critical
**Test Type:** Error Handling
**Related FR:** NFR-RELIABILITY

**Test Scenario (Gherkin):**
```gherkin
Feature: RabbitMQ Failure Handling
  Scenario: Handle RabbitMQ down when queuing task
    Given RabbitMQ service is stopped
    When I attempt to queue training task
    Then API should detect RabbitMQ unavailable
    And error should be "RabbitMQ connection failed"
    And HTTP 503 Service Unavailable should be returned
    When RabbitMQ is restarted
    Then next task should queue successfully
```

**Test Steps:**
1. Stop RabbitMQ:
   ```bash
   docker stop rabbitmq-dev
   ```
2. Attempt to queue task:
   ```python
   with pytest.raises(ConnectionError):
       train_model.delay("lstm", "BTCUSDT")
   ```
3. Restart RabbitMQ:
   ```bash
   docker start rabbitmq-dev
   ```
4. Verify recovery:
   ```python
   task = train_model.delay("lstm", "BTCUSDT")
   assert task.id is not None
   ```

**Expected Results:**
- ✅ Error detected immediately
- ✅ Clear error message
- ✅ Auto-recovery after RabbitMQ restart

**Assertions:**
```python
# Stop RabbitMQ
subprocess.run(["docker", "stop", "rabbitmq-dev"])

# Attempt to queue
with pytest.raises(Exception) as exc_info:
    train_model.delay("lstm", "BTCUSDT")

assert "connection" in str(exc_info.value).lower()

# Restart
subprocess.run(["docker", "start", "rabbitmq-dev"])
time.sleep(5)

# Should work now
task = train_model.delay("lstm", "BTCUSDT")
assert task.id is not None
```

---

### TC-ASYNC-077: Worker Crash Mid-Task

**Priority:** Critical
**Test Type:** Error Handling
**Related FR:** NFR-RELIABILITY

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle worker crash during task execution
    Given training task is running
    And task is at 50% progress (epoch 50/100)
    When worker process crashes (killed)
    Then task status should become FAILURE
    And checkpoint should be saved at epoch 50
    When worker restarts
    Then new worker should pick up queued tasks
    And task can be retried from checkpoint
```

**Expected Results:**
- ✅ Task fails gracefully
- ✅ Checkpoint saved
- ✅ Worker restarts automatically
- ✅ Can resume from checkpoint

**Assertions:**
```python
task = train_model.delay("lstm", "BTCUSDT", epochs=100)

# Wait for task to start
while task.info.get("current", 0) < 50:
    time.sleep(1)

# Kill worker
subprocess.run(["pkill", "-9", "-f", "celery.*worker"])

time.sleep(2)

# Task should fail
assert task.state in ["FAILURE", "REVOKED"]

# Restart worker
subprocess.Popen(["celery", "-A", "celery_app", "worker", "-D"])
time.sleep(5)

# Worker should be healthy
# New tasks can be queued
task2 = train_model.delay("lstm", "ETHUSDT", epochs=10)
assert task2.id is not None
```

---

### TC-ASYNC-078: Redis (Result Backend) Down

**Priority:** High
**Test Type:** Error Handling
**Related FR:** NFR-RELIABILITY

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle Redis down (result backend unavailable)
    Given task is running
    And Redis is stopped
    When task completes
    Then result cannot be stored in Redis
    But task should complete successfully in worker
    And result should be logged
    When checking task status via API
    Then 503 error should be returned (result backend unavailable)
```

**Expected Results:**
- ✅ Task completes despite Redis down
- ✅ Result logged locally
- ✅ API returns 503 when checking status

**Assertions:**
```python
task = train_model.delay("lstm", "BTCUSDT", epochs=10)

# Stop Redis mid-task
subprocess.run(["docker", "stop", "redis-dev"])

# Task should still complete in worker (check logs)
# But result.get() will fail
with pytest.raises(Exception):
    task.get(timeout=600)

# Restart Redis
subprocess.run(["docker", "start", "redis-dev"])
```

---

### TC-ASYNC-079: MongoDB Down During Task

**Priority:** High
**Test Type:** Error Handling
**Related FR:** NFR-RELIABILITY

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle MongoDB down during data fetch
    Given training task starts
    And MongoDB is stopped during data loading
    When task tries to fetch training data
    Then connection error should be caught
    And task should retry 3 times with backoff
    When MongoDB is restarted
    Then retry should succeed
    And task should complete
```

**Expected Results:**
- ✅ MongoDB error caught
- ✅ Retries 3 times
- ✅ Succeeds after MongoDB restart

**Assertions:**
```python
# Start task
task = train_model.delay("lstm", "BTCUSDT")

# Stop MongoDB shortly after
time.sleep(2)
subprocess.run(["docker", "stop", "mongodb-dev"])

# Task should retry
time.sleep(10)

# Restart MongoDB
subprocess.run(["docker", "start", "mongodb-dev"])
time.sleep(5)

# Task should eventually succeed
result = task.get(timeout=1200)
assert result["status"] == "success"
```

---

### TC-ASYNC-080: Task Timeout (Soft & Hard Limits)

**Priority:** High
**Test Type:** Functional
**Related FR:** FR-ASYNC-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Enforce task timeout limits
    Given training task has soft_limit=7200s (2h)
    And hard_limit=9000s (2.5h)
    When task runs longer than 2 hours
    Then SoftTimeLimitExceeded should be raised
    And checkpoint should be saved
    When task continues beyond 2.5 hours
    Then worker should forcefully terminate task
    And task status should be FAILURE
```

**Expected Results:**
- ✅ Soft limit warning at 2h
- ✅ Hard limit termination at 2.5h
- ✅ Checkpoint saved

**Assertions:**
```python
# Mock long-running training
with patch('time.sleep') as mock_sleep:
    def slow_training(*args, **kwargs):
        time.sleep(7300)  # > 2 hours
        return {"val_accuracy": 0.65}

    with patch('model_manager.ModelManager.train_model', side_effect=slow_training):
        with pytest.raises(SoftTimeLimitExceeded):
            train_model.apply(args=["lstm", "BTCUSDT"]).get()
```

---

### TC-ASYNC-081 to TC-ASYNC-085: Additional Error Handling Tests

**TC-ASYNC-081:** Disk Full During Model Save
**TC-ASYNC-082:** Network Timeout During API Call
**TC-ASYNC-083:** Invalid Task Arguments (type mismatch)
**TC-ASYNC-084:** Circular Task Dependencies Prevention
**TC-ASYNC-085:** Task Revoke During Critical Section

---

## Performance & Load Test Cases

### TC-ASYNC-086: Concurrent Task Execution (100 Tasks)

**Priority:** High
**Test Type:** Load Test
**Related FR:** NFR-PERFORMANCE

**Test Scenario (Gherkin):**
```gherkin
Feature: Load Testing
  Scenario: Handle 100 concurrent tasks
    Given 4 Celery workers running
    When I queue 100 training tasks simultaneously
    Then all tasks should queue successfully
    And workers should process tasks concurrently
    And all 100 tasks should complete within 6 hours
    And no worker crashes
    And memory usage < 8GB per worker
```

**Test Steps:**
1. Start 4 workers:
   ```bash
   celery -A celery_app worker -Q ml_training -c 2 &  # 2 workers, 2 concurrency each
   ```
2. Queue 100 tasks:
   ```python
   tasks = []
   for i in range(100):
       task = train_model.delay("lstm", f"SYMBOL{i%20}USDT", epochs=10)
       tasks.append(task)
   ```
3. Wait for all to complete:
   ```python
   results = [task.get(timeout=21600) for task in tasks]  # 6 hour timeout
   ```

**Expected Results:**
- ✅ All 100 tasks complete
- ✅ No worker crashes
- ✅ Total time < 6 hours
- ✅ Memory < 8GB per worker

**Assertions:**
```python
import time
start = time.time()

tasks = [train_model.delay("lstm", f"SYM{i%20}USDT", epochs=10) for i in range(100)]
results = [task.get(timeout=21600) for task in tasks]
elapsed = time.time() - start

assert len(results) == 100
assert all(r["status"] == "success" for r in results)
assert elapsed < 21600  # < 6 hours

# Check worker memory usage (implementation-specific)
# Should be < 8GB per worker
```

---

### TC-ASYNC-087: Queue Depth Monitoring

**Priority:** Medium
**Test Type:** Performance
**Related FR:** NFR-PERFORMANCE

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Monitor queue depth under load
    Given 50 tasks queued
    And 2 workers processing
    When monitoring queue depth
    Then queue depth should decrease over time
    And queue should never exceed 1000 messages
    And oldest message should be processed within 1 hour
```

**Expected Results:**
- ✅ Queue depth monitored
- ✅ Queue depth decreases
- ✅ Never exceeds 1000

**Assertions:**
```python
# Queue 50 tasks
for i in range(50):
    train_model.delay("lstm", f"SYM{i}USDT", epochs=5)

# Check queue depth
output = subprocess.check_output(["rabbitmqctl", "list_queues", "name", "messages"])
lines = output.decode().split("\n")

for line in lines:
    if "ml_training" in line:
        parts = line.split()
        queue_depth = int(parts[1])
        assert queue_depth < 1000
        print(f"Queue depth: {queue_depth}")
```

---

### TC-ASYNC-088: Task Latency (Queue → Execution)

**Priority:** Medium
**Test Type:** Performance
**Related FR:** NFR-PERFORMANCE

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Measure task queue latency
    Given workers are idle
    When I queue a task
    Then task should be picked up within 5 seconds
    And start executing within 10 seconds
```

**Expected Results:**
- ✅ Task pickup < 5 seconds
- ✅ Execution start < 10 seconds

**Assertions:**
```python
import time

queue_time = time.time()
task = train_model.delay("lstm", "BTCUSDT", epochs=5)

# Wait for task to start
while task.state == "PENDING":
    time.sleep(0.1)

pickup_time = time.time()
latency = pickup_time - queue_time

assert latency < 5  # < 5 seconds to pickup
assert task.state in ["STARTED", "PROGRESS"]
```

---

### TC-ASYNC-089 to TC-ASYNC-093: Additional Performance Tests

**TC-ASYNC-089:** Worker Autoscaling (scale up/down based on load)
**TC-ASYNC-090:** Task Prioritization (high priority tasks first)
**TC-ASYNC-091:** Memory Leak Detection (long-running workers)
**TC-ASYNC-092:** CPU Utilization (should be 60-80% under load)
**TC-ASYNC-093:** Task Result Cleanup (old results purged after 7 days)

---

## Security Test Cases

### TC-ASYNC-094: Authentication Check for Admin-Only Tasks

**Priority:** Critical
**Test Type:** Security
**Related FR:** NFR-SECURITY

**Test Scenario (Gherkin):**
```gherkin
Feature: Task Security
  Scenario: Prevent unauthorized task execution
    Given I am not authenticated
    When I attempt to trigger training task
    Then request should be rejected with 401 Unauthorized
    And no task should be queued

  Scenario: Prevent non-admin from triggering training
    Given I am authenticated as regular user (not admin)
    When I attempt to trigger training
    Then request should be rejected with 403 Forbidden
    And error should be "Admin privileges required"
```

**Expected Results:**
- ✅ Unauthenticated: 401
- ✅ Non-admin: 403
- ✅ No task queued

**Assertions:**
```python
# No JWT token
response = requests.post("/api/tasks/train", json={"model_type": "lstm"})
assert response.status_code == 401

# Regular user token
response = requests.post(
    "/api/tasks/train",
    headers={"Authorization": f"Bearer {USER_JWT_TOKEN}"},
    json={"model_type": "lstm"}
)
assert response.status_code == 403
assert "Admin privileges" in response.json()["error"]
```

---

### TC-ASYNC-095: Rate Limiting (Max Tasks Per User)

**Priority:** High
**Test Type:** Security
**Related FR:** NFR-SECURITY

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Enforce rate limiting
    Given max training tasks = 10 per user per day
    When I trigger 10 training tasks
    Then all 10 should succeed
    When I trigger 11th task
    Then request should be rejected with 429 Too Many Requests
    And error should be "Max 10 trainings per day exceeded"
```

**Expected Results:**
- ✅ First 10 tasks succeed
- ✅ 11th task rejected: 429
- ✅ Rate limit resets next day

**Assertions:**
```python
# Trigger 10 tasks
for i in range(10):
    response = requests.post("/api/tasks/train", headers=admin_headers, json={"model_type": "lstm"})
    assert response.status_code == 202

# 11th should fail
response = requests.post("/api/tasks/train", headers=admin_headers, json={"model_type": "lstm"})
assert response.status_code == 429
assert "Max 10 trainings" in response.json()["error"]
```

---

### TC-ASYNC-096 to TC-ASYNC-099: Additional Security Tests

**TC-ASYNC-096:** Input Sanitization (prevent code injection)
**TC-ASYNC-097:** Model File Access Control (users can't access others' models)
**TC-ASYNC-098:** Task Result Access Control (can only view own task results)
**TC-ASYNC-099:** Secrets in Logs (ensure no API keys/passwords logged)

---

## Infrastructure Test Cases

### TC-ASYNC-100: Celery Beat Scheduling

**Priority:** Critical
**Test Type:** Infrastructure
**Related FR:** FR-ASYNC-004 to FR-ASYNC-008

**Test Scenario (Gherkin):**
```gherkin
Feature: Task Scheduling
  Scenario: Verify scheduled tasks run on time
    Given Celery Beat is running
    When system_health_check is scheduled every 15 minutes
    Then task should execute at 00:00, 00:15, 00:30, 00:45
    And execution time should be within ±1 minute of target
```

**Expected Results:**
- ✅ Tasks run on schedule
- ✅ Execution time ±1 minute

**Assertions:**
```python
# Check Celery Beat schedule
from celery_app import app

schedule = app.conf.beat_schedule
assert "system-health-check" in schedule

health_check_schedule = schedule["system-health-check"]
assert health_check_schedule["schedule"].total_seconds() == 900  # 15 minutes
assert health_check_schedule["task"] == "tasks.monitoring.system_health_check"
```

---

### TC-ASYNC-101: Task Queue Routing

**Priority:** High
**Test Type:** Infrastructure

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Verify task routing to correct queues
    When training task is queued
    Then it should route to "ml_training" queue
    When monitoring task is queued
    Then it should route to "scheduled" queue
    When backtest task is queued
    Then it should route to "backtesting" queue
```

**Expected Results:**
- ✅ Tasks routed to correct queues
- ✅ Queue isolation working

**Assertions:**
```python
from celery_app import app

routes = app.conf.task_routes

assert routes["tasks.ml_tasks.train_model"]["queue"] == "ml_training"
assert routes["tasks.monitoring.system_health_check"]["queue"] == "scheduled"
assert routes["tasks.backtest_tasks.backtest_strategy"]["queue"] == "backtesting"
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_celery_integration.py:122-167`

---

### TC-ASYNC-102 to TC-ASYNC-105: Additional Infrastructure Tests

**TC-ASYNC-102:** Worker Heartbeat Monitoring
**TC-ASYNC-103:** Dead Letter Queue Handling
**TC-ASYNC-104:** Task Result Expiration (Redis TTL)
**TC-ASYNC-105:** Flower Monitoring UI Availability

---

## Traceability Matrix

| Test Case ID | Related FR | Code Reference | Priority | Status |
|--------------|------------|----------------|----------|--------|
| TC-ASYNC-001 | FR-ASYNC-001 | ml_tasks.py:32-134 | Critical | ✅ |
| TC-ASYNC-002 | FR-ASYNC-001 | ml_tasks.py:32-134 | High | ✅ |
| TC-ASYNC-003 | FR-ASYNC-001 | ml_tasks.py:130-133 | Critical | ✅ |
| TC-ASYNC-004 | FR-ASYNC-001 | ml_tasks.py:32-134 | High | ✅ |
| TC-ASYNC-011 | FR-ASYNC-002 | ml_tasks.py:136-197 | High | ✅ |
| TC-ASYNC-012 | FR-ASYNC-002 | ml_tasks.py:136-197 | High | ✅ |
| TC-ASYNC-013 | FR-ASYNC-003 | ml_tasks.py:200-254 | Medium | ✅ |
| TC-ASYNC-031 | FR-ASYNC-004 | monitoring.py:66-218 | Critical | ✅ |
| TC-ASYNC-032 | FR-ASYNC-004 | monitoring.py:66-218 | Critical | ✅ |
| TC-ASYNC-036 | FR-ASYNC-005 | monitoring.py:227-298 | High | ✅ |
| TC-ASYNC-037 | FR-ASYNC-006 | monitoring.py:306-409 | High | ✅ |
| TC-ASYNC-040 | FR-ASYNC-007 | monitoring.py:418-557 | Critical | ✅ |
| TC-ASYNC-051 | FR-ASYNC-008 | ai_improvement.py:64-246 | Critical | ✅ |
| TC-ASYNC-052 | FR-ASYNC-008 | ai_improvement.py:64-246 | Critical | ✅ |
| TC-ASYNC-056 | FR-ASYNC-009 | ai_improvement.py:254-300+ | Critical | ✅ |
| TC-ASYNC-059 | FR-ASYNC-010 | ai_improvement.py | Critical | ✅ |
| TC-ASYNC-066 | FR-ASYNC-011 | backtest_tasks.py:27-139 | High | ✅ |
| TC-ASYNC-067 | FR-ASYNC-012 | backtest_tasks.py:142-200+ | High | ✅ |
| TC-ASYNC-076 | NFR-RELIABILITY | - | Critical | ✅ |
| TC-ASYNC-077 | NFR-RELIABILITY | - | Critical | ✅ |
| TC-ASYNC-086 | NFR-PERFORMANCE | - | High | ✅ |
| TC-ASYNC-094 | NFR-SECURITY | - | Critical | ✅ |
| TC-ASYNC-100 | FR-ASYNC-004-008 | celery_app.py | Critical | ✅ |
| TC-ASYNC-101 | Infrastructure | celery_app.py | High | ✅ |

---

## Acceptance Criteria

The Async Tasks System is considered complete and production-ready when:

**Functional Requirements:**
- [ ] All 12 async tasks execute successfully in test environment
- [ ] All 105 test cases pass with 100% success rate
- [ ] Task progress tracking works for all long-running tasks
- [ ] Automatic retries work with exponential backoff
- [ ] Checkpointing and resume work for training tasks
- [ ] GPT-4 self-analysis makes correct decisions (>=80% accuracy in test scenarios)
- [ ] Scheduled tasks run within ±1 minute of target time
- [ ] All notifications (Email, Slack) deliver successfully

**Performance Requirements:**
- [ ] Training task (100 epochs) completes in < 2 hours
- [ ] Health check completes in < 30 seconds
- [ ] Bulk analysis (100 symbols) completes in < 60 seconds
- [ ] Task queue latency < 5 seconds (submission to worker pickup)
- [ ] 100 concurrent tasks complete successfully
- [ ] Memory usage < 8GB per worker under load
- [ ] CPU usage 60-80% average (efficient resource utilization)

**Reliability Requirements:**
- [ ] 99.9% uptime for RabbitMQ message broker
- [ ] Zero task loss during RabbitMQ/Redis restart
- [ ] Auto-recovery from worker crashes within 60 seconds
- [ ] Graceful handling of all service failures (MongoDB, Redis, RabbitMQ)
- [ ] 100% task completion rate for non-transient errors
- [ ] Dead letter queue captures failed tasks after max retries

**Security Requirements:**
- [ ] Authentication required for all admin tasks
- [ ] Rate limiting prevents abuse (10 tasks/user/day)
- [ ] No secrets logged in task outputs
- [ ] Model files encrypted at rest (AES-256)
- [ ] Task results accessible only by owner

**Quality Requirements:**
- [ ] Code coverage >= 95% for task modules
- [ ] All edge cases handled gracefully
- [ ] Error messages clear and actionable
- [ ] Documentation complete and accurate
- [ ] Zero HIGH/CRITICAL security vulnerabilities

**Monitoring Requirements:**
- [ ] All tasks logged to MongoDB with audit trail
- [ ] Flower UI accessible for real-time monitoring
- [ ] Alerts sent for critical issues within 30 seconds
- [ ] GPT-4 cost tracking accurate (±5%)
- [ ] Performance metrics tracked and queryable

---

## Test Execution Summary

**Last Execution Date:** [To be filled during execution]
**Executed By:** [QA Team Member Name]
**Environment:** Development / Staging / Production
**RabbitMQ Version:** 3.12+
**Redis Version:** 7.0+
**MongoDB Version:** 6.0+
**Celery Version:** 5.3+

| Category | Total | Passed | Failed | Blocked | Success Rate |
|----------|-------|--------|--------|---------|--------------|
| ML Tasks | 30 | - | - | - | -% |
| Monitoring Tasks | 20 | - | - | - | -% |
| AI Improvement | 15 | - | - | - | -% |
| Backtest Tasks | 10 | - | - | - | -% |
| Error Handling | 10 | - | - | - | -% |
| Performance | 8 | - | - | - | -% |
| Security | 6 | - | - | - | -% |
| Infrastructure | 6 | - | - | - | -% |
| **TOTAL** | **105** | **-** | **-** | **-** | **-%** |

**Test Execution Metrics:**
- Total Execution Time: [X hours]
- Average Test Duration: [X minutes]
- Flaky Tests: [List any unstable tests]
- Known Issues: [List known bugs/limitations]

---

## Test Automation

**Framework:** pytest
**CI/CD:** GitHub Actions
**Parallel Execution:** 4 workers (pytest-xdist)
**Retry Policy:** 3 attempts for flaky tests
**Timeout:** 2 hours for full suite, 30 minutes for smoke tests

**Test Files Structure:**
```
python-ai-service/tests/
├── test_ml_tasks.py                    # TC-ASYNC-001 to TC-ASYNC-030
├── test_monitoring_tasks.py            # TC-ASYNC-031 to TC-ASYNC-050
├── test_ai_improvement_tasks.py        # TC-ASYNC-051 to TC-ASYNC-065
├── test_backtest_tasks.py              # TC-ASYNC-066 to TC-ASYNC-075
├── test_error_handling.py              # TC-ASYNC-076 to TC-ASYNC-085
├── test_performance.py                 # TC-ASYNC-086 to TC-ASYNC-093
├── test_security.py                    # TC-ASYNC-094 to TC-ASYNC-099
├── test_infrastructure.py              # TC-ASYNC-100 to TC-ASYNC-105
├── conftest.py                         # Shared fixtures
└── test_celery_integration.py          # Integration tests
```

**Running Tests:**
```bash
# Full suite
pytest tests/ -v --cov=tasks --cov-report=html

# Specific category
pytest tests/test_ml_tasks.py -v

# Smoke tests (critical only)
pytest tests/ -m critical -v

# Parallel execution
pytest tests/ -n 4 -v

# With detailed logs
pytest tests/ -v --log-cli-level=DEBUG
```

**Coverage Target:** >= 95%
**Mutation Score Target:** >= 80%

---

## Notes & Warnings

**⚠️ CRITICAL WARNINGS:**

1. **Finance System**: This is a FINANCE system. Async task failures can result in:
   - Missed trading opportunities ($$ loss)
   - Stale ML models making poor predictions
   - Undetected system failures leading to downtime
   - API cost overruns (GPT-4 can get expensive)

2. **GPT-4 Costs**: Each GPT-4 analysis costs ~$0.024. Running hourly = $17/month. Monitor costs closely.

3. **Testing in Production**:
   - NEVER test with production OpenAI API key
   - ALWAYS use testnet for any trading-related tasks
   - Mock external services (Binance, OpenAI) in tests

4. **Long-Running Tests**:
   - Full test suite can take 2+ hours
   - Smoke tests (critical only) recommended for CI/CD
   - Use markers to run subsets: `@pytest.mark.critical`

5. **Flaky Tests**:
   - Network-dependent tests may be flaky
   - Worker timing tests may fail on slow machines
   - Use retry decorator for known flaky tests

**💡 BEST PRACTICES:**

1. **Test Isolation**: Each test should clean up after itself (delete models, clear queues)
2. **Mocking**: Mock external APIs (OpenAI, Binance) to avoid costs and rate limits
3. **Fixtures**: Use pytest fixtures for common setup (workers, queues, test data)
4. **Assertions**: Include clear assertion messages explaining expected vs actual
5. **Logging**: Enable debug logging for failed tests to aid troubleshooting

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-11-22 | Documentation System | Initial creation - 105 comprehensive test cases |

---

**Document Control:**
- **Created by**: Documentation System
- **Reviewed by**: [To be assigned]
- **Approved by**: [To be assigned]
- **Next Review Date**: 2025-12-22

---

*End of TC-ASYNC: Async Tasks System Test Cases Document*
