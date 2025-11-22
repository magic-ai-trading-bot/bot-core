# FR-ASYNC-TASKS: Async Task Processing System

**Document ID**: FR-ASYNC-TASKS
**Version**: 1.0
**Last Updated**: 2025-11-22
**Status**: APPROVED
**Author**: Documentation System
**Reviewers**: [To be assigned]
**Priority**: CRITICAL (affects ML training, monitoring, and system reliability)

---

## TABLE OF CONTENTS

1. [Overview](#1-overview)
2. [Functional Requirements](#2-functional-requirements)
   - [ML Tasks](#21-ml-tasks-ml_taskspy)
   - [Monitoring Tasks](#22-monitoring-tasks-monitoringpy)
   - [AI Improvement Tasks](#23-ai-improvement-tasks-ai_improvementpy)
   - [Backtest Tasks](#24-backtest-tasks-backtest_taskspy)
3. [Non-Functional Requirements](#3-non-functional-requirements)
4. [Infrastructure Components](#4-infrastructure-components)
5. [Data Models](#5-data-models)
6. [API Endpoints](#6-api-endpoints)
7. [Sequence Diagrams](#7-sequence-diagrams)
8. [Error Scenarios](#8-error-scenarios)
9. [Testing Strategy](#9-testing-strategy)
10. [Deployment](#10-deployment)
11. [Acceptance Checklist](#11-acceptance-checklist)
12. [Traceability](#12-traceability)

---

## 1. OVERVIEW

### 1.1 Purpose

The Async Task Processing System provides a robust, scalable background job execution framework for long-running operations that cannot be performed in synchronous HTTP request/response cycles. This system is **CRITICAL** for a cryptocurrency trading bot where:

- **ML model training** can take 30-120 minutes per model
- **Strategy backtesting** can process months of historical data
- **Real-time monitoring** must run continuously without blocking trading operations
- **AI-driven retraining** must happen autonomously based on performance degradation
- **System health checks** must run every 15 minutes to detect failures immediately

**Finance Impact**: In trading systems, delayed model updates or missed monitoring alerts can result in significant financial losses. This system ensures all background operations complete successfully with proper retry logic, error handling, and audit trails.

### 1.2 Scope

**Included**:
- ML model training and batch prediction (3 tasks)
- System health monitoring and performance tracking (4 tasks)
- GPT-4 powered AI self-improvement and adaptive retraining (3 tasks)
- Strategy backtesting and optimization (2 tasks)
- Task scheduling with Celery Beat (5 scheduled jobs)
- Result storage and audit trails in MongoDB
- Multi-channel notifications (Email, Slack, Discord, Telegram)
- Task progress tracking and state management
- Automatic retry with exponential backoff
- Dead letter queue handling

**NOT Included**:
- Real-time signal generation (handled by Rust core engine)
- Live trading execution (handled by Rust paper trading engine)
- WebSocket communication (separate system)
- Frontend dashboard (separate Next.js application)

### 1.3 Technology Stack

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Message Broker** | RabbitMQ | 3.12+ | Task queue management, routing |
| **Task Queue** | Celery | 5.3+ | Distributed task execution |
| **Results Backend** | Redis | 7.0+ | Task result storage, caching |
| **Database** | MongoDB | 6.0+ | Persistent storage, audit trails |
| **Language** | Python | 3.11+ | Task implementation |
| **Monitoring** | Flower | 2.0+ | Real-time task monitoring UI |

### 1.4 Dependencies

**External Services**:
- `@depends-on`: RabbitMQ (message broker)
- `@depends-on`: Redis (results backend)
- `@depends-on`: MongoDB (persistent storage)
- `@depends-on`: OpenAI GPT-4 API (AI analysis)
- `@depends-on`: Rust Core Engine API (trading data)
- `@depends-on`: Binance API (market data)

**Internal Dependencies**:
- `@spec:FR-ML-001` - Machine Learning Models
- `@spec:FR-API-002` - REST API Endpoints
- `@spec:FR-STORAGE-001` - MongoDB Storage
- `@spec:NFR-RELIABILITY-001` - System Reliability

### 1.5 Success Criteria

- [ ] All 12 async tasks execute successfully in production
- [ ] 99.9% uptime for RabbitMQ message broker
- [ ] <5 second task queue latency (from submission to worker pickup)
- [ ] 100% task completion rate for non-transient errors
- [ ] Auto-recovery from worker crashes within 60 seconds
- [ ] Zero task loss during RabbitMQ restart
- [ ] Scheduled tasks run within ±1 minute of target time
- [ ] All task results persisted to MongoDB with audit trail
- [ ] Notifications sent within 30 seconds of task completion/failure

---

## 2. FUNCTIONAL REQUIREMENTS

### 2.1 ML Tasks (`ml_tasks.py`)

#### FR-ASYNC-001: Async Model Training

**Priority**: CRITICAL
**Implementation**: `python-ai-service/tasks/ml_tasks.py:32-134`
**Depends On**: FR-ML-001 (LSTM Model), FR-ML-002 (Dataset Management)
**Task Name**: `tasks.ml_tasks.train_model`

##### Description

Background task for training machine learning models (LSTM, GRU, Transformer) without blocking the main API service. This is a **long-running operation** (30-120 minutes) that must execute asynchronously with progress tracking, automatic retries, and proper resource management.

**Why Critical**: Model accuracy directly impacts trading decisions. Failed training = stale models = poor predictions = financial losses.

##### Business Logic

```mermaid
sequenceDiagram
    participant Client
    participant API
    participant RabbitMQ
    participant Worker
    participant MongoDB
    participant Filesystem

    Client->>API: POST /api/tasks/train
    API->>RabbitMQ: Queue train_model task
    API->>Client: 202 Accepted (task_id)

    RabbitMQ->>Worker: Assign to ml_training queue
    Worker->>Worker: Update state: PROGRESS 0%
    Worker->>MongoDB: Load training dataset
    Worker->>Worker: Initialize model
    Worker->>Worker: Update state: PROGRESS 20%

    loop For each epoch (1-200)
        Worker->>Worker: Train one epoch
        Worker->>Worker: Update state: PROGRESS X%
    end

    Worker->>Filesystem: Save trained model (.h5)
    Worker->>MongoDB: Store metrics
    Worker->>Worker: Update state: SUCCESS
    Worker->>Client: Email notification (if configured)
```

##### Input Parameters

```python
{
  "model_type": str,           # REQUIRED: "lstm" | "gru" | "transformer"
  "symbol": str,               # REQUIRED: "BTCUSDT", "ETHUSDT", etc.
  "days_of_data": int,         # OPTIONAL: 7-365 days (default: 30)
  "retrain": bool,             # OPTIONAL: Retrain from scratch (default: False)
  "epochs": int,               # OPTIONAL: 50-200 (default: 100)
  "batch_size": int,           # OPTIONAL: 32-128 (default: 64)
  "learning_rate": float,      # OPTIONAL: 0.0001-0.01 (default: 0.001)
  "validation_split": float    # OPTIONAL: 0.1-0.3 (default: 0.2)
}
```

**Validation Rules**:
- `model_type` must be one of: "lstm", "gru", "transformer" (case-insensitive)
- `symbol` must be valid Binance trading pair (check against Binance API)
- `days_of_data` must be >= 7 (minimum for meaningful training)
- `epochs` must be >= 10 and <= 500
- `batch_size` must be power of 2 (32, 64, 128, 256)
- `learning_rate` must be > 0.0
- `validation_split` must be > 0.0 and < 0.5

##### Output

**Success Response**:
```python
{
  "status": "success",
  "model_type": "lstm",
  "symbol": "BTCUSDT",
  "training_results": {
    "final_loss": 0.0234,
    "validation_loss": 0.0289,
    "val_accuracy": 0.68,
    "training_time_seconds": 1847,
    "epochs_completed": 100,
    "best_epoch": 87
  },
  "model_path": "/models/lstm_BTCUSDT_1h_20251122_143022.h5",
  "task_id": "abc-123-def-456",
  "timestamp": "2025-11-22T14:30:22Z"
}
```

**Failure Response**:
```python
{
  "status": "failed",
  "error": "Insufficient training data: expected 720 samples, got 234",
  "error_type": "ValidationError",
  "task_id": "abc-123-def-456",
  "retry_count": 3,
  "max_retries": 3,
  "timestamp": "2025-11-22T14:32:15Z"
}
```

##### Acceptance Criteria

**Functional**:
- [ ] Task completes successfully for all 3 model types (LSTM, GRU, Transformer)
- [ ] Model saved to disk with timestamp in filename
- [ ] Metrics logged to MongoDB `training_jobs` collection
- [ ] Progress updates every 5% completion
- [ ] Email notification sent on completion (if enabled)
- [ ] Task state transitions: PENDING → PROGRESS → SUCCESS/FAILURE
- [ ] Old models archived (not deleted) when new model deployed

**Performance**:
- [ ] Training completes within 2 hours max for 100 epochs
- [ ] Memory usage < 8 GB per task (prevent OOM)
- [ ] CPU usage < 80% average (allow other tasks to run)
- [ ] GPU utilization > 80% when available (maximize efficiency)
- [ ] Retry delay: Exponential backoff (60s, 120s, 300s)

**Security**:
- [ ] Only authenticated admin users can trigger training
- [ ] Model files encrypted at rest (AES-256)
- [ ] Training logs scrubbed of sensitive data (API keys, secrets)
- [ ] Rate limiting: Max 10 training tasks per user per day
- [ ] Resource quotas enforced per user

**Reliability**:
- [ ] Auto-retry on transient failures (3 attempts)
- [ ] Checkpoint saved every 10 epochs (resume on crash)
- [ ] Graceful degradation if GPU unavailable (fall back to CPU)
- [ ] Task timeout: 2 hours soft limit, 2.5 hours hard limit
- [ ] Dead letter queue for failed tasks after max retries

##### Edge Cases

| # | Scenario | Expected Behavior | Implementation |
|---|----------|-------------------|----------------|
| 1 | **Out of Memory** | Reduce batch size automatically and retry | Catch `MemoryError`, halve batch_size, restart training |
| 2 | **Training Divergence** | Stop early if loss > 10x initial loss | Monitor loss every epoch, abort if diverging |
| 3 | **Interrupted Training** | Resume from last checkpoint | Save checkpoint every 10 epochs, load on restart |
| 4 | **Corrupt Data** | Validate dataset before training | Check for NaN, inf, missing values upfront |
| 5 | **Concurrent Training** | Queue tasks, max 2 concurrent per model type | Use Celery worker concurrency=2 for ml_training queue |
| 6 | **Disk Full** | Fail gracefully, clean up temp files | Check disk space before training, cleanup on error |
| 7 | **GPU Not Available** | Fall back to CPU training (slower) | Try GPU first, catch CUDA errors, use CPU |
| 8 | **Invalid Symbol** | Reject immediately with clear error | Validate symbol against Binance API before queuing |
| 9 | **Dataset Too Small** | Reject if < 720 samples (30 days @ 1h) | Count samples, return error if insufficient |
| 10 | **Model Already Training** | Reject duplicate task for same model+symbol | Check active tasks before queuing |

##### Example Usage

**Python API Call**:
```python
from tasks.ml_tasks import train_model

# Trigger async training
result = train_model.delay(
    model_type="lstm",
    symbol="BTCUSDT",
    days_of_data=30,
    retrain=True,
    epochs=100
)

# Check task status
print(f"Task ID: {result.id}")
print(f"Status: {result.status}")  # 'PENDING' | 'STARTED' | 'SUCCESS' | 'FAILURE'

# Wait for completion (blocking)
final_result = result.get(timeout=7200)  # 2 hour timeout
print(f"Accuracy: {final_result['training_results']['val_accuracy']:.2%}")
```

**REST API Call**:
```bash
curl -X POST http://localhost:8000/api/tasks/train \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer $JWT_TOKEN" \
  -d '{
    "model_type": "lstm",
    "symbol": "BTCUSDT",
    "days_of_data": 30,
    "epochs": 100
  }'

# Response:
# {
#   "task_id": "abc-123-def-456",
#   "status": "PENDING",
#   "estimated_completion": "2025-11-22T16:30:00Z",
#   "status_url": "/api/tasks/abc-123-def-456"
# }
```

**Check Task Status**:
```bash
curl http://localhost:8000/api/tasks/abc-123-def-456 \
  -H "Authorization: Bearer $JWT_TOKEN"

# Response (in progress):
# {
#   "task_id": "abc-123-def-456",
#   "status": "PROGRESS",
#   "progress": 45,
#   "current_epoch": 45,
#   "total_epochs": 100,
#   "logs": "Training epoch 45/100... Loss: 0.0234"
# }
```

##### Error Handling

```python
from celery.exceptions import SoftTimeLimitExceeded, MaxRetriesExceededError

@app.task(bind=True, max_retries=3)
def train_model(self, model_type, symbol, **kwargs):
    try:
        # Training logic
        return {"status": "success", ...}

    except MemoryError as e:
        # OOM - reduce batch size and retry
        logger.error(f"Out of memory: {e}")
        kwargs['batch_size'] = kwargs.get('batch_size', 64) // 2
        if kwargs['batch_size'] < 8:
            raise  # Too small, give up
        raise self.retry(exc=e, kwargs=kwargs, countdown=120)

    except ValueError as e:
        # Invalid parameters - don't retry
        logger.error(f"Validation error: {e}")
        return {"status": "failed", "error": str(e)}

    except SoftTimeLimitExceeded:
        # Timeout - save checkpoint and fail
        logger.error("Training timeout - saving checkpoint")
        save_checkpoint(model, epoch)
        raise

    except Exception as e:
        # Unknown error - retry with backoff
        logger.error(f"Training failed: {e}")
        raise self.retry(exc=e, countdown=60 * (2 ** self.request.retries))
```

##### Testing

**Unit Tests** (`tests/test_ml_tasks.py`):
```python
def test_train_model_success():
    """Test successful model training"""
    result = train_model.apply(args=["lstm", "BTCUSDT"]).get()
    assert result["status"] == "success"
    assert result["training_results"]["val_accuracy"] > 0.5

def test_train_model_invalid_symbol():
    """Test rejection of invalid symbol"""
    with pytest.raises(ValueError, match="Invalid symbol"):
        train_model.apply(args=["lstm", "INVALID"]).get()

def test_train_model_retry_on_failure():
    """Test automatic retry on transient failure"""
    with patch('models.model_manager.ModelManager.train_model') as mock:
        mock.side_effect = [ConnectionError(), {"val_accuracy": 0.7}]
        result = train_model.apply(args=["lstm", "BTCUSDT"]).get()
        assert result["status"] == "success"
        assert mock.call_count == 2  # Original + 1 retry

def test_train_model_checkpoint_resume():
    """Test checkpoint resume after interruption"""
    # Simulate crash at epoch 50
    # Resume should start from epoch 50, not 0
    pass
```

**Integration Tests**:
```python
def test_train_model_end_to_end():
    """Test full training workflow with real data"""
    # Queue task
    result = train_model.delay("lstm", "BTCUSDT", days_of_data=7)

    # Wait for completion
    final_result = result.get(timeout=3600)

    # Verify model saved
    assert os.path.exists(final_result["model_path"])

    # Verify MongoDB entry
    doc = db.training_jobs.find_one({"task_id": result.id})
    assert doc is not None
    assert doc["status"] == "completed"
```

**Load Tests**:
```python
def test_concurrent_training_tasks():
    """Test 10 concurrent training tasks"""
    tasks = [
        train_model.delay("lstm", f"BTC{i}USDT")
        for i in range(10)
    ]

    # All should complete within 3 hours
    results = [task.get(timeout=10800) for task in tasks]

    # All should succeed
    assert all(r["status"] == "success" for r in results)
```

##### Performance Requirements

| Metric | Target | Critical Threshold | Measurement |
|--------|--------|-------------------|-------------|
| Training Time (100 epochs) | < 1 hour | < 2 hours | Actual wall-clock time |
| Memory Usage | < 6 GB | < 8 GB | Peak RSS memory |
| CPU Usage (avg) | 60-70% | < 80% | Average over training duration |
| GPU Utilization | > 80% | > 50% | nvidia-smi sampling |
| Retry Success Rate | > 90% | > 70% | (Retries succeeded / Total retries) |
| Checkpoint Overhead | < 5% | < 10% | (Checkpoint time / Total time) |

##### Security

**Authentication**:
```python
# Only authenticated users can trigger training
@require_auth
@require_role("admin")  # Only admins can train models
def trigger_training(request):
    # Extract user from JWT
    user_id = request.user.id

    # Check rate limit
    if get_training_count_today(user_id) >= 10:
        raise RateLimitExceeded("Max 10 trainings per day")

    # Queue task
    task = train_model.delay(...)
    return {"task_id": task.id}
```

**Data Protection**:
```python
# Encrypt model files at rest
from cryptography.fernet import Fernet

def save_model_encrypted(model, path, key):
    # Serialize model
    model_bytes = serialize_model(model)

    # Encrypt
    cipher = Fernet(key)
    encrypted = cipher.encrypt(model_bytes)

    # Save
    with open(path, 'wb') as f:
        f.write(encrypted)
```

**Log Scrubbing**:
```python
import re

def scrub_logs(log_message):
    """Remove sensitive data from logs"""
    # Remove API keys
    log_message = re.sub(r'api_key=[a-zA-Z0-9]+', 'api_key=***', log_message)

    # Remove passwords
    log_message = re.sub(r'password=[^&\s]+', 'password=***', log_message)

    return log_message
```

##### Related Requirements

- `@spec:FR-ML-001` - LSTM Model Implementation
- `@spec:FR-ML-002` - Dataset Management
- `@spec:FR-STORAGE-001` - MongoDB Storage
- `@spec:NFR-PERFORMANCE-001` - Performance Requirements
- `@spec:NFR-SECURITY-001` - Security Requirements

##### Code References

- `@code:python-ai-service/tasks/ml_tasks.py:32-134` - Main implementation
- `@code:python-ai-service/models/model_manager.py` - Model training logic
- `@code:python-ai-service/tests/test_ml_tasks.py` - Unit tests

---

#### FR-ASYNC-002: Bulk Analysis

**Priority**: HIGH
**Implementation**: `python-ai-service/tasks/ml_tasks.py:136-197`
**Depends On**: FR-ASYNC-001 (Trained Models)
**Task Name**: `tasks.ml_tasks.bulk_analysis`

##### Description

Analyze multiple trading symbols in parallel using trained ML models to generate buy/sell signals for portfolio-wide decision making. This task processes 10-100 symbols simultaneously, aggregating predictions and confidence scores.

**Why Important**: Enables portfolio diversification by analyzing multiple assets concurrently rather than sequentially, reducing total analysis time from minutes to seconds.

##### Business Logic

1. Receive list of symbols to analyze
2. For each symbol in parallel:
   - Fetch latest market data (last 100 candles)
   - Load pre-trained model
   - Generate prediction (price direction, confidence)
   - Calculate technical indicators (RSI, MACD, etc.)
3. Aggregate results with confidence scores
4. Sort by confidence (highest first)
5. Return top N recommendations

##### Input Parameters

```python
{
  "symbols": List[str],        # REQUIRED: ["BTCUSDT", "ETHUSDT", ...]
  "timeframe": str,            # OPTIONAL: "5m" | "15m" | "1h" | "4h" (default: "1h")
  "model_type": str,           # OPTIONAL: "lstm" | "ensemble" (default: "ensemble")
  "min_confidence": float,     # OPTIONAL: 0.0-1.0 (default: 0.6)
  "top_n": int                 # OPTIONAL: Return top N results (default: 10)
}
```

##### Output

```python
{
  "status": "success",
  "symbols_analyzed": 50,
  "timestamp": "2025-11-22T14:30:00Z",
  "results": [
    {
      "symbol": "BTCUSDT",
      "signal": "BUY",
      "confidence": 0.87,
      "predicted_price": 45230.50,
      "current_price": 44850.00,
      "expected_return": 0.85,  # %
      "indicators": {
        "rsi": 42.5,
        "macd": 123.4,
        "trend": "bullish"
      }
    },
    // ... more results sorted by confidence
  ],
  "failed_symbols": ["INVALID1", "INVALID2"],  # Symbols that failed analysis
  "task_id": "xyz-789-abc-012"
}
```

##### Acceptance Criteria

- [ ] Analyze 100 symbols in < 60 seconds
- [ ] Process symbols in parallel (10 concurrent workers)
- [ ] Return results sorted by confidence (descending)
- [ ] Handle individual symbol failures gracefully
- [ ] Cache results for 5 minutes to avoid redundant API calls
- [ ] Progress updates every 10% completion

##### Edge Cases

| # | Scenario | Expected Behavior |
|---|----------|-------------------|
| 1 | Symbol data unavailable | Skip symbol, include in `failed_symbols` list |
| 2 | Model not trained | Return error, suggest training first |
| 3 | All symbols fail | Return empty results with error message |
| 4 | Duplicate symbols | De-duplicate before processing |
| 5 | 1000+ symbols | Chunk into batches of 100, process sequentially |

##### Example Usage

```python
from tasks.ml_tasks import bulk_analysis

# Analyze top 50 cryptocurrencies
result = bulk_analysis.delay(
    symbols=["BTCUSDT", "ETHUSDT", "BNBUSDT", ...],  # 50 symbols
    timeframe="1h",
    min_confidence=0.7
)

# Get results
recommendations = result.get(timeout=120)
print(f"Analyzed {recommendations['symbols_analyzed']} symbols")
print(f"Top recommendation: {recommendations['results'][0]}")
```

##### Testing

```python
def test_bulk_analysis_success():
    """Test bulk analysis with valid symbols"""
    result = bulk_analysis.apply(args=[["BTCUSDT", "ETHUSDT"]]).get()
    assert result["status"] == "success"
    assert result["symbols_analyzed"] == 2

def test_bulk_analysis_handles_failures():
    """Test graceful handling of failed symbols"""
    result = bulk_analysis.apply(args=[["BTCUSDT", "INVALID"]]).get()
    assert "INVALID" in result["failed_symbols"]
    assert result["symbols_analyzed"] == 1
```

##### Related Requirements

- `@spec:FR-ML-003` - Price Prediction Models
- `@spec:FR-INDICATORS-001` - Technical Indicators

---

#### FR-ASYNC-003: Price Prediction

**Priority**: MEDIUM
**Implementation**: `python-ai-service/tasks/ml_tasks.py:200-254`
**Task Name**: `tasks.ml_tasks.predict_price`

##### Description

Generate future price predictions for a single symbol over a specified time horizon (1-48 hours) using trained ML models.

##### Input Parameters

```python
{
  "symbol": str,              # REQUIRED: "BTCUSDT"
  "model_type": str,          # OPTIONAL: "lstm" | "gru" | "transformer"
  "horizon_hours": int        # OPTIONAL: 1-48 (default: 24)
}
```

##### Output

```python
{
  "status": "success",
  "symbol": "BTCUSDT",
  "model_type": "lstm",
  "horizon_hours": 24,
  "predictions": [
    {"hour": 1, "predicted_price": 44920.50, "confidence": 0.82},
    {"hour": 2, "predicted_price": 45050.25, "confidence": 0.79},
    // ... 24 predictions
  ],
  "task_id": "pred-456-789"
}
```

##### Acceptance Criteria

- [ ] Generate predictions in < 10 seconds
- [ ] Confidence scores decrease with time horizon
- [ ] Predictions use latest model checkpoint
- [ ] Results cached for 15 minutes

---

### 2.2 Monitoring Tasks (`monitoring.py`)

#### FR-ASYNC-004: System Health Check

**Priority**: CRITICAL
**Implementation**: `python-ai-service/tasks/monitoring.py:66-218`
**Schedule**: Every 15 minutes (Celery Beat)
**Task Name**: `tasks.monitoring.system_health_check`

##### Description

Continuously monitor health of all system components (APIs, databases, message queues, disk space) and send immediate alerts when critical issues detected.

**Why Critical**: Early detection of system failures prevents cascade failures and trading downtime. In finance, every minute of downtime can result in missed profitable trades.

##### Business Logic

```mermaid
sequenceDiagram
    participant Beat as Celery Beat
    participant Worker
    participant Services as System Services
    participant MongoDB
    participant Notifications

    Beat->>Worker: Trigger every 15 min

    par Check All Services
        Worker->>Services: GET /api/health (Rust)
        Worker->>Services: GET /health (Python)
        Worker->>Services: GET / (Frontend)
        Worker->>Services: mongosh ping (MongoDB)
        Worker->>Services: redis-cli ping (Redis)
        Worker->>Services: rabbitmqctl status (RabbitMQ)
    end

    Worker->>Worker: Aggregate health status

    alt Critical Issues Found
        Worker->>Notifications: Send CRITICAL alert
        Worker->>Notifications: Email admin immediately
    else Warning Issues
        Worker->>Notifications: Send WARNING alert
    else All Healthy
        Worker->>Worker: Log success
    end

    Worker->>MongoDB: Store health report
```

##### Services Monitored

| Service | Health Endpoint | Healthy Criteria | Warning | Critical |
|---------|----------------|------------------|---------|----------|
| **Rust Core Engine** | `http://localhost:8080/api/health` | HTTP 200, response < 5s | Response 5-10s | HTTP ≠200 or timeout |
| **Python AI Service** | `http://localhost:8000/health` | HTTP 200, response < 5s | Response 5-10s | HTTP ≠200 or timeout |
| **Frontend Dashboard** | `http://localhost:3000/` | HTTP 200 or 304 | Response > 3s | HTTP 500 or timeout |
| **MongoDB** | `mongosh --eval "db.adminCommand('ping')"` | Exit code 0 | Slow response | Connection refused |
| **Redis** | `redis-cli ping` | Returns "PONG" | Response > 1s | Connection refused |
| **RabbitMQ** | `rabbitmqctl status` | Running, 0 errors | Queue depth > 1000 | Not running |
| **Disk Space** | `df -h /` | Usage < 80% | Usage 80-90% | Usage > 90% |
| **Memory** | `free -m` | Usage < 80% | Usage 80-90% | Usage > 90% |

##### Input Parameters

None (scheduled task, no parameters)

##### Output

```python
{
  "status": "success",
  "health_report": {
    "timestamp": "2025-11-22T14:30:00Z",
    "overall_status": "healthy" | "degraded" | "critical",
    "services": {
      "Rust Core Engine": {
        "status": "healthy",
        "response_time_ms": 45
      },
      "Python AI Service": {
        "status": "healthy",
        "response_time_ms": 32
      },
      "MongoDB": {
        "status": "healthy"
      },
      "Redis": {
        "status": "healthy"
      },
      "Disk": {
        "status": "healthy",
        "usage_percent": 67
      }
    },
    "alerts": [
      "⚠️ Disk usage high: 82%"
    ]
  },
  "task_id": "health-check-123"
}
```

##### Acceptance Criteria

**Functional**:
- [ ] Check all 8 services in < 30 seconds total
- [ ] Send CRITICAL alert within 30 seconds of detection
- [ ] Store health report in MongoDB for historical analysis
- [ ] Auto-restart services if possible (Redis, RabbitMQ)
- [ ] Escalate to admin if service down > 5 minutes

**Alert Thresholds**:
- [ ] **CRITICAL**: Any service down, disk > 90%, memory > 90%
- [ ] **WARNING**: Service slow (>5s response), disk > 80%, memory > 80%
- [ ] **INFO**: All healthy

**Notifications**:
- [ ] Email to admin for CRITICAL alerts
- [ ] Slack notification for all alerts
- [ ] Discord/Telegram if configured

##### Edge Cases

| # | Scenario | Expected Behavior |
|---|----------|-------------------|
| 1 | All services down | Send CRITICAL alert, don't crash task |
| 2 | MongoDB unreachable | Log to file, skip storage, continue checks |
| 3 | Network timeout | Retry once with 10s timeout, then mark down |
| 4 | Partial service degradation | Mark as "degraded", not "critical" |
| 5 | False positive (temporary blip) | Don't alert until 2 consecutive failures |
| 6 | Alert notification fails | Log error, don't block health check |

##### Example Output (Critical)

```python
{
  "health_report": {
    "overall_status": "critical",
    "services": {
      "Rust Core Engine": {"status": "down", "error": "Connection refused"},
      "MongoDB": {"status": "healthy"},
      // ...
    },
    "alerts": [
      "❌ Rust Core Engine is DOWN: Connection refused",
      "🚨 CRITICAL: Primary trading service unavailable!"
    ]
  }
}
```

##### Testing

```python
def test_health_check_all_healthy(mock_requests):
    """Test health check when all services healthy"""
    mock_requests.get.return_value = Mock(status_code=200)
    result = system_health_check()
    assert result["health_report"]["overall_status"] == "healthy"

def test_health_check_service_down(mock_requests):
    """Test critical alert when service down"""
    mock_requests.get.side_effect = ConnectionError("Connection refused")
    result = system_health_check()
    assert result["health_report"]["overall_status"] == "critical"
    assert len(result["health_report"]["alerts"]) > 0
```

---

#### FR-ASYNC-005: Daily Portfolio Report

**Priority**: HIGH
**Implementation**: `python-ai-service/tasks/monitoring.py:227-298`
**Schedule**: 8:00 AM UTC daily (Celery Beat)
**Task Name**: `tasks.monitoring.daily_portfolio_report`

##### Description

Generate comprehensive daily portfolio performance summary and email to admins every morning.

##### Business Logic

1. Fetch portfolio data from Rust API (`/api/paper-trading/portfolio`)
2. Calculate key metrics (balance, return %, win rate, avg profit)
3. Generate strategy breakdown (which strategies performed best)
4. Format as readable email/Slack message
5. Send to configured recipients
6. Store in MongoDB for historical tracking

##### Output

```python
{
  "status": "success",
  "report": {
    "date": "2025-11-22",
    "balance": 10542.75,
    "total_return_pct": 5.43,
    "total_trades": 47,
    "winning_trades": 32,
    "losing_trades": 15,
    "win_rate": 68.09,
    "avg_profit_per_trade": 1.87,
    "strategies": {
      "rsi": {"trades": 20, "win_rate": 70.0, "profit": 2.3},
      "macd": {"trades": 15, "win_rate": 66.7, "profit": 1.8},
      "bollinger": {"trades": 12, "win_rate": 66.7, "profit": 1.5}
    }
  }
}
```

##### Acceptance Criteria

- [ ] Run every day at exactly 8:00 AM UTC (±1 minute)
- [ ] Email sent within 5 minutes of task start
- [ ] Include charts/visualizations (optional)
- [ ] Store report in MongoDB for trending
- [ ] Handle weekends/holidays (no trades → summary still sent)

---

#### FR-ASYNC-006: Daily API Cost Report

**Priority**: HIGH
**Implementation**: `python-ai-service/tasks/monitoring.py:306-409`
**Schedule**: 9:00 AM UTC daily
**Task Name**: `tasks.monitoring.daily_api_cost_report`

##### Description

Track and report OpenAI GPT-4 API costs to prevent runaway expenses. **CRITICAL** for cost control in production.

##### Business Logic

1. Fetch cost statistics from Python AI service (`/ai/cost/statistics`)
2. Extract daily/monthly costs
3. Check against thresholds
4. Send alerts if thresholds exceeded
5. Store cost data for trending

##### Cost Thresholds

| Threshold | Daily Limit | Monthly Limit | Action |
|-----------|-------------|---------------|--------|
| **WARNING** | $2.00 | $50.00 | Send email notification |
| **CRITICAL** | $5.00 | $100.00 | Send urgent alert + disable GPT-4 |

##### Output

```python
{
  "status": "success",
  "report": {
    "date": "2025-11-22",
    "session": {
      "total_requests": 234,
      "total_cost_usd": 1.87,
      "total_cost_vnd": 46750
    },
    "projections": {
      "daily_cost_usd": 2.34,
      "monthly_cost_usd": 70.20
    },
    "alerts": [
      "⚠️ WARNING: Daily cost $2.34 exceeds $2.00 warning!"
    ]
  }
}
```

##### Acceptance Criteria

- [ ] Alert sent immediately if daily cost > $5.00
- [ ] Email includes cost breakdown by model (gpt-4, gpt-4-turbo)
- [ ] Track token usage (input/output tokens)
- [ ] Auto-disable GPT-4 if monthly > $100 (safety net)

---

#### FR-ASYNC-007: Daily Performance Analysis

**Priority**: CRITICAL
**Implementation**: `python-ai-service/tasks/monitoring.py:418-557`
**Schedule**: 1:00 AM UTC daily
**Task Name**: `tasks.monitoring.daily_performance_analysis`

##### Description

Analyze trading performance daily and trigger GPT-4 self-analysis if performance degradation detected. This is the **trigger** for adaptive retraining.

##### Business Logic

```mermaid
flowchart TD
    A[Fetch last 7 days trades] --> B[Calculate metrics]
    B --> C{Win rate < 55%?}
    C -->|Yes| D[CRITICAL: Trigger GPT-4]
    C -->|No| E{Sharpe < 1.0?}
    E -->|Yes| D
    E -->|No| F{Avg profit < 1.5%?}
    F -->|Yes| D
    F -->|No| G[All good, log metrics]

    D --> H[Store metrics in MongoDB]
    H --> I[Flag for GPT-4 analysis at 3 AM]
```

##### Performance Thresholds

| Metric | Target | Warning | Critical | Action |
|--------|--------|---------|----------|--------|
| **Win Rate** | 70% | < 70% | < 55% | Trigger GPT-4 analysis |
| **Avg Profit** | 2.6% | < 2.6% | < 1.5% | Trigger GPT-4 analysis |
| **Sharpe Ratio** | 2.1 | < 2.1 | < 1.0 | Trigger GPT-4 analysis |

##### Output

```python
{
  "status": "success",
  "analysis": {
    "date": "2025-11-22",
    "metrics": {
      "total_trades": 47,
      "win_rate": 52.3,  # Below 55% → CRITICAL
      "avg_profit_per_trade": 1.2,  # Below 1.5% → CRITICAL
      "sharpe_ratio": 0.87  # Below 1.0 → CRITICAL
    },
    "performance": {
      "win_rate_status": "critical",
      "avg_profit_status": "critical",
      "sharpe_status": "critical"
    },
    "alerts": [
      "🔴 CRITICAL: Win rate 52.3% below 55% threshold!",
      "🔴 CRITICAL: Sharpe ratio 0.87 below 1.0 threshold!"
    ],
    "trigger_ai_analysis": true  # Will run GPT-4 at 3 AM
  }
}
```

##### Acceptance Criteria

- [ ] Require minimum 10 trades for analysis (avoid false positives)
- [ ] Store metrics in MongoDB for GPT-4 to access
- [ ] Flag `trigger_ai_analysis = true` if any CRITICAL threshold breached
- [ ] Send email to admins with performance summary
- [ ] Include 7-day trend chart (optional)

---

### 2.3 AI Improvement Tasks (`ai_improvement.py`)

#### FR-ASYNC-008: GPT-4 Self-Analysis

**Priority**: CRITICAL
**Implementation**: `python-ai-service/tasks/ai_improvement.py:64-246`
**Schedule**: 3:00 AM UTC daily (or triggered by performance alerts)
**Task Name**: `tasks.ai_improvement.gpt4_self_analysis`

##### Description

Use OpenAI GPT-4 to analyze trading performance trends and autonomously decide if model retraining is needed. This is the **intelligent decision-making system** that replaces naive time-based retraining.

**Why Revolutionary**: Instead of blindly retraining models every N days, GPT-4 analyzes:
- Performance degradation patterns (temporary vs structural)
- Market regime changes (bull → bear market)
- Model accuracy trends
- Cost-benefit of retraining now vs waiting

##### Business Logic

```mermaid
sequenceDiagram
    participant Beat as Celery Beat
    participant Worker
    participant Rust as Rust API
    participant Python as Python API
    participant Binance
    participant GPT4 as OpenAI GPT-4
    participant MongoDB

    Beat->>Worker: Trigger at 3 AM daily

    Worker->>Rust: GET /api/paper-trading/trades?days=7
    Rust-->>Worker: Last 7 days trades

    Worker->>Python: GET /ai/model/accuracy_history?days=7
    Python-->>Worker: Model accuracy trends

    Worker->>Binance: GET /api/v3/ticker/24hr
    Binance-->>Worker: Market data (volatility, volume)

    Worker->>Worker: Calculate daily metrics
    Worker->>Worker: Build GPT-4 analysis prompt

    Worker->>GPT4: POST /v1/chat/completions (analysis prompt)
    GPT4-->>Worker: JSON recommendation

    alt Recommendation = "retrain" AND confidence > 0.7
        Worker->>Worker: Queue adaptive_retrain task
        Worker->>MongoDB: Store GPT-4 analysis
        Worker->>Worker: Send notification
    else Recommendation = "wait"
        Worker->>MongoDB: Store GPT-4 analysis
        Worker->>Worker: Log decision, skip retraining
    end
```

##### Input Parameters

```python
{
  "force_analysis": bool  # OPTIONAL: Run even if no alerts (default: False)
}
```

##### GPT-4 Prompt Structure

```python
prompt = f"""
You are a trading bot self-improvement AI. Analyze the following performance data and decide if model retraining is needed.

PERFORMANCE TRENDS (Last 7 days):
- Win Rate: [65%, 62%, 58%, 55%, 53%, 52%, 51%] (Target: 70%, Current: 51%)
- Avg Profit: [2.1%, 1.8%, 1.6%, 1.4%, 1.2%, 1.1%, 1.0%] (Target: 2.6%)
- Sharpe Ratio: [1.8, 1.6, 1.4, 1.2, 1.0, 0.9, 0.85] (Target: 2.1, Current: 0.85)

MODEL ACCURACY (Last 7 days):
- LSTM: 68% → 65% → 63% (dropped 5% in 7 days)
- GRU: 65% → 64% → 63%
- Transformer: 70% → 69% → 68%

MARKET CONDITIONS:
- 24h Price Change: -8.5% (high volatility)
- 24h Volume: +230% (regime change?)

THRESHOLDS:
- Retrain if win rate < 55% for 3+ days ✅ (currently 51%, 3 days below)
- Retrain if model accuracy drops > 5% in 7 days ✅ (LSTM dropped 5%)
- Retrain if Sharpe ratio < 1.0 ✅ (currently 0.85)
- Retrain if market regime changed significantly ⚠️ (high volatility spike)

QUESTION: Should we retrain the ML models? Consider:
1. Is performance degradation temporary (market noise) or structural (model decay)?
2. Is it due to model staleness or market regime change (bull → bear)?
3. Will retraining with recent data help or hurt?
4. What's the cost-benefit of retraining now vs waiting?

OUTPUT FORMAT (MUST BE VALID JSON):
{{
  "recommendation": "retrain" | "wait" | "optimize_parameters",
  "confidence": 0.0-1.0,
  "reasoning": "detailed explanation of decision",
  "urgency": "low" | "medium" | "high",
  "suggested_actions": ["action1", "action2"],
  "estimated_improvement": "X% expected win rate improvement"
}}
"""
```

##### Output

**GPT-4 Response (Retrain Recommended)**:
```python
{
  "status": "success",
  "analysis": {
    "recommendation": "retrain",
    "confidence": 0.85,
    "reasoning": "Clear structural performance degradation detected. Win rate dropped 14% over 7 days with model accuracy declining 5%. Market regime changed from bull to bear (volatility spike +230%). Retraining with recent bear market data will likely improve predictions.",
    "urgency": "high",
    "suggested_actions": [
      "Retrain all 3 models (LSTM, GRU, Transformer) with last 60 days data",
      "Focus on bear market patterns",
      "Increase training epochs to 150 for better convergence"
    ],
    "estimated_improvement": "Expected win rate improvement: 8-12% (from 51% to 59-63%)",
    "retrain_triggered": true,
    "retrain_task_id": "retrain-task-abc-123"
  }
}
```

**GPT-4 Response (Wait Recommended)**:
```python
{
  "analysis": {
    "recommendation": "wait",
    "confidence": 0.72,
    "reasoning": "Performance degradation appears temporary due to unusual market volatility spike. Model accuracy only dropped 2% (within normal variance). Historical data shows similar volatility spikes recover within 3-5 days. Recommend waiting 3 more days before retraining.",
    "urgency": "low",
    "suggested_actions": [
      "Monitor performance for 3 more days",
      "Consider parameter tuning instead of full retrain",
      "Increase stop-loss thresholds temporarily"
    ],
    "estimated_improvement": "N/A - wait and monitor",
    "retrain_triggered": false
  }
}
```

##### Acceptance Criteria

**Functional**:
- [ ] Run at 3:00 AM UTC daily
- [ ] Also run immediately when triggered by performance alerts
- [ ] Skip analysis if performance acceptable (win_rate ≥ 55%, sharpe ≥ 1.0)
- [ ] Call GPT-4 with temperature=0.3 (consistent decisions)
- [ ] Parse JSON response robustly (handle malformed JSON)
- [ ] Queue `adaptive_retrain` task if recommendation="retrain" + confidence ≥ 0.7 + urgency ∈ ["medium", "high"]
- [ ] Store analysis in MongoDB for audit trail
- [ ] Send notification with recommendation

**Performance**:
- [ ] GPT-4 API call < 10 seconds
- [ ] Total analysis < 30 seconds
- [ ] Cost per analysis < $0.10 (use gpt-4-turbo, not gpt-4-0125-preview)

**Security**:
- [ ] OPENAI_API_KEY stored securely (not in logs)
- [ ] Scrub sensitive data from prompt (user IDs, account balances)
- [ ] Rate limit: Max 5 GPT-4 calls per hour (prevent abuse)

##### Edge Cases

| # | Scenario | Expected Behavior |
|---|----------|-------------------|
| 1 | OPENAI_API_KEY not configured | Return error, don't crash, send alert |
| 2 | GPT-4 returns malformed JSON | Retry once, if fails again mark as error |
| 3 | GPT-4 API timeout | Retry with 30s timeout, max 2 retries |
| 4 | Insufficient trades (< 10) | Skip analysis, log "Not enough data" |
| 5 | GPT-4 recommends "retrain" with low confidence (< 0.5) | Don't trigger retrain, just log |
| 6 | MongoDB unavailable | Log analysis to file, skip storage |
| 7 | Cost limit exceeded ($100/month) | Disable GPT-4 analysis, alert admin |

##### Example Usage

```python
from tasks.ai_improvement import gpt4_self_analysis

# Scheduled run (3 AM daily)
# Automatic via Celery Beat

# Manual trigger (force analysis even if performance OK)
result = gpt4_self_analysis.delay(force_analysis=True)
analysis = result.get(timeout=60)

print(f"Recommendation: {analysis['analysis']['recommendation']}")
print(f"Confidence: {analysis['analysis']['confidence']:.0%}")
print(f"Reasoning: {analysis['analysis']['reasoning']}")

if analysis['analysis']['retrain_triggered']:
    print(f"Retrain task queued: {analysis['analysis']['retrain_task_id']}")
```

##### Testing

```python
def test_gpt4_analysis_retrain_recommendation(mock_openai):
    """Test GPT-4 recommends retrain when performance critical"""
    mock_openai.ChatCompletion.create.return_value = {
        "choices": [{
            "message": {
                "content": json.dumps({
                    "recommendation": "retrain",
                    "confidence": 0.85,
                    "urgency": "high"
                })
            }
        }]
    }

    result = gpt4_self_analysis.apply().get()
    assert result["analysis"]["recommendation"] == "retrain"
    assert result["analysis"]["retrain_triggered"] == True

def test_gpt4_analysis_skipped_if_performance_good(mock_requests):
    """Test analysis skipped when performance acceptable"""
    # Mock good performance metrics
    mock_requests.get.return_value = Mock(json=lambda: {
        "win_rate": 72.5,
        "sharpe_ratio": 2.3
    })

    result = gpt4_self_analysis.apply().get()
    assert result["status"] == "skipped"
    assert result["reason"] == "Performance is acceptable"
```

##### Cost Analysis

**GPT-4 Turbo Pricing** (as of 2025-11):
- Input tokens: $0.01 / 1K tokens
- Output tokens: $0.03 / 1K tokens

**Typical Analysis**:
- Input: ~1500 tokens (performance data + prompt)
- Output: ~300 tokens (JSON response)
- Cost per analysis: $0.01 × 1.5 + $0.03 × 0.3 = **$0.024**

**Monthly Cost** (1 analysis/day):
- 30 days × $0.024 = **$0.72/month**

**If triggered by alerts** (assume 5 extra per month):
- 35 analyses × $0.024 = **$0.84/month**

**Conclusion**: Very cost-effective for intelligent decision-making.

##### Related Requirements

- `@spec:FR-ASYNC-007` - Daily Performance Analysis (trigger)
- `@spec:FR-ASYNC-009` - Adaptive Retraining (action)
- `@spec:FR-GPT4-001` - GPT-4 Integration

---

#### FR-ASYNC-009: Adaptive Model Retraining

**Priority**: CRITICAL
**Implementation**: `python-ai-service/tasks/ai_improvement.py:254-364`
**Triggered By**: GPT-4 Self-Analysis recommendation
**Task Name**: `tasks.ai_improvement.adaptive_retrain`

##### Description

Automatically retrain ML models when GPT-4 analysis determines performance degradation is structural (not temporary) and retraining will likely improve results. This is **adaptive** because it's triggered by intelligent analysis, not naive time intervals.

**Why Critical**: Keeps models fresh and accurate without over-retraining (which wastes compute) or under-retraining (which causes losses).

##### Business Logic

```mermaid
flowchart TD
    A[Triggered by GPT-4] --> B[Fetch last 60 days market data]
    B --> C{For each model type}

    C --> D1[Train LSTM]
    C --> D2[Train GRU]
    C --> D3[Train Transformer]

    D1 --> E1[Validate on holdout set]
    D2 --> E2[Validate on holdout set]
    D3 --> E3[Validate on holdout set]

    E1 --> F1{Accuracy improved?}
    E2 --> F2{Accuracy improved?}
    E3 --> F3{Accuracy improved?}

    F1 -->|Yes| G1[Deploy new LSTM model]
    F1 -->|No| H1[Keep old LSTM model]

    F2 -->|Yes| G2[Deploy new GRU model]
    F2 -->|No| H2[Keep old GRU model]

    F3 -->|Yes| G3[Deploy new Transformer]
    F3 -->|No| H3[Keep old Transformer]

    G1 & G2 & G3 & H1 & H2 & H3 --> I[Send completion report]
    I --> J[Store results in MongoDB]
```

##### Input Parameters

```python
{
  "model_types": List[str],      # REQUIRED: ["lstm", "gru", "transformer"]
  "analysis_result": Dict        # REQUIRED: GPT-4 analysis that triggered this
}
```

##### Output

```python
{
  "status": "success",
  "results": {
    "timestamp": "2025-11-22T03:15:00Z",
    "trigger": "gpt4_recommendation",
    "analysis_reasoning": "Performance degradation + market regime change",
    "models": {
      "lstm": {
        "status": "success",
        "old_accuracy": 0.63,
        "new_accuracy": 0.71,
        "improvement": 0.08,  # +8%
        "deployed": true
      },
      "gru": {
        "status": "success",
        "old_accuracy": 0.63,
        "new_accuracy": 0.69,
        "improvement": 0.06,  # +6%
        "deployed": true
      },
      "transformer": {
        "status": "success",
        "old_accuracy": 0.68,
        "new_accuracy": 0.66,
        "improvement": -0.02,  # Worse!
        "deployed": false  # Keep old model
      }
    }
  },
  "summary": {
    "successful_retrains": 3,
    "deployed_models": 2,
    "total_improvement": "+7% avg accuracy"
  },
  "task_id": "retrain-abc-123"
}
```

##### Acceptance Criteria

**Functional**:
- [ ] Retrain all requested model types in parallel
- [ ] Validate new model on holdout set (last 20% of data)
- [ ] Compare new accuracy vs old accuracy
- [ ] Deploy new model ONLY if accuracy improves (prevents regression)
- [ ] Keep old model as backup if new model worse
- [ ] Archive old models (don't delete)
- [ ] Send email report with before/after metrics
- [ ] Store retrain history in MongoDB

**Performance**:
- [ ] Complete all 3 models in < 2 hours
- [ ] Use 60 days of historical data (good balance of freshness and size)
- [ ] GPU utilization > 80% during training
- [ ] Memory usage < 10 GB total

**Safety**:
- [ ] Never deploy a model with accuracy < 60% (sanity check)
- [ ] Rollback if new model causes trading errors
- [ ] Alert admin if all models fail to improve
- [ ] Log detailed training metrics for audit

##### Edge Cases

| # | Scenario | Expected Behavior |
|---|----------|-------------------|
| 1 | New model accuracy worse | Keep old model, log decision, send report |
| 2 | Training fails mid-way | Retry once, if fails again keep old model |
| 3 | All 3 models fail to train | Send CRITICAL alert, investigate manually |
| 4 | Disk full during training | Clean up temp files, retry with smaller dataset |
| 5 | New model accuracy same as old | Deploy new model (more recent data) |
| 6 | Old model not found | Train new model, deploy it (first time setup) |

##### Example Usage

```python
# Typically triggered automatically by GPT-4 analysis
# But can be manually triggered

from tasks.ai_improvement import adaptive_retrain

result = adaptive_retrain.delay(
    model_types=["lstm", "gru", "transformer"],
    analysis_result={
        "recommendation": "retrain",
        "reasoning": "Performance degraded due to market regime change"
    }
)

# Wait for completion (max 2 hours)
final_result = result.get(timeout=7200)

print(f"Deployed: {final_result['results']['summary']['deployed_models']} models")
print(f"Improvement: {final_result['results']['summary']['total_improvement']}")
```

##### Testing

```python
def test_adaptive_retrain_deploys_improved_model(mock_train):
    """Test new model deployed when accuracy improves"""
    mock_train.return_value = {
        "old_accuracy": 0.65,
        "new_accuracy": 0.72,
        "improvement": 0.07
    }

    result = adaptive_retrain.apply(args=[["lstm"], {}]).get()

    assert result["results"]["models"]["lstm"]["deployed"] == True
    assert result["results"]["models"]["lstm"]["improvement"] > 0

def test_adaptive_retrain_keeps_old_model_if_worse(mock_train):
    """Test old model kept when new model worse"""
    mock_train.return_value = {
        "old_accuracy": 0.70,
        "new_accuracy": 0.65,
        "improvement": -0.05
    }

    result = adaptive_retrain.apply(args=[["lstm"], {}]).get()

    assert result["results"]["models"]["lstm"]["deployed"] == False
```

---

#### FR-ASYNC-010: Emergency Strategy Disable

**Priority**: CRITICAL
**Implementation**: `python-ai-service/tasks/ai_improvement.py:372-428`
**Triggered By**: Severe daily loss (>10%) or consecutive losses (>10)
**Task Name**: `tasks.ai_improvement.emergency_strategy_disable`

##### Description

Automatically disable a failing trading strategy to prevent further losses when severe degradation detected.

##### Input Parameters

```python
{
  "strategy_name": str,  # REQUIRED: "rsi" | "macd" | "bollinger" | "volume"
  "reason": str          # REQUIRED: Reason for emergency disable
}
```

##### Trigger Conditions

- Daily loss > 10% from this strategy
- Consecutive losses > 10 trades
- Win rate < 30% over last 20 trades

##### Output

```python
{
  "status": "success",
  "strategy": "rsi",
  "reason": "Consecutive losses: 12 trades",
  "disabled_at": "2025-11-22T14:30:00Z",
  "task_id": "emergency-disable-123"
}
```

##### Acceptance Criteria

- [ ] Disable strategy within 10 seconds
- [ ] Send CRITICAL alert immediately
- [ ] Prevent new trades from this strategy
- [ ] Don't close existing positions (let them complete)
- [ ] Log detailed reason and metrics
- [ ] Require manual re-enable (don't auto-enable)

---

### 2.4 Backtest Tasks (`backtest_tasks.py`)

#### FR-ASYNC-011: Strategy Backtesting

**Priority**: MEDIUM
**Implementation**: `python-ai-service/tasks/backtest_tasks.py:27-139`
**Task Name**: `tasks.backtest_tasks.backtest_strategy`

##### Description

Backtest a trading strategy on historical data to evaluate performance before deploying to live trading.

##### Input Parameters

```python
{
  "strategy_name": str,        # REQUIRED: "rsi" | "macd" | "bollinger" | "volume"
  "symbol": str,               # REQUIRED: "BTCUSDT"
  "start_date": str,           # REQUIRED: "2024-01-01"
  "end_date": str,             # REQUIRED: "2024-12-31"
  "parameters": Dict           # OPTIONAL: Strategy-specific parameters
}
```

##### Output

```python
{
  "status": "success",
  "results": {
    "strategy": "rsi",
    "symbol": "BTCUSDT",
    "period": "2024-01-01 to 2024-12-31",
    "total_trades": 147,
    "winning_trades": 98,
    "losing_trades": 49,
    "win_rate": 66.67,
    "total_return": 18.5,  # %
    "sharpe_ratio": 1.87,
    "max_drawdown": 8.2,  # %
    "profit_factor": 2.34,
    "avg_trade_duration_hours": 18.5,
    "parameters_used": {
      "rsi_period": 14,
      "rsi_overbought": 70,
      "rsi_oversold": 30
    }
  },
  "task_id": "backtest-456"
}
```

##### Acceptance Criteria

- [ ] Complete backtest in < 1 hour (for 1 year of data)
- [ ] Process historical candles from MongoDB
- [ ] Simulate realistic trading (slippage, fees, latency)
- [ ] Generate comprehensive metrics report
- [ ] Store results in MongoDB for comparison
- [ ] Support parameter variations

---

#### FR-ASYNC-012: Strategy Optimization

**Priority**: LOW
**Implementation**: `python-ai-service/tasks/backtest_tasks.py:142-230`
**Task Name**: `tasks.backtest_tasks.optimize_strategy`

##### Description

Find optimal strategy parameters using grid search or genetic algorithms.

##### Input Parameters

```python
{
  "strategy_name": str,
  "symbol": str,
  "start_date": str,
  "end_date": str,
  "parameter_variations": int  # OPTIONAL: 10-500 (default: 50)
}
```

##### Output

```python
{
  "status": "success",
  "strategy": "rsi",
  "total_variations_tested": 50,
  "best_parameters": {
    "rsi_period": 12,
    "rsi_overbought": 75,
    "rsi_oversold": 25
  },
  "best_performance": {
    "win_rate": 71.2,
    "total_return": 22.8,
    "sharpe_ratio": 2.15
  },
  "top_10_results": [...],
  "task_id": "optimize-789"
}
```

##### Acceptance Criteria

- [ ] Complete 50 parameter variations in < 2 hours
- [ ] Return top 10 parameter combinations
- [ ] Visualize parameter sensitivity (optional)
- [ ] Prevent overfitting (validate on out-of-sample data)

---

## 3. NON-FUNCTIONAL REQUIREMENTS

### 3.1 Performance

| Metric | Target | Critical Threshold | Measurement Method |
|--------|--------|-------------------|-------------------|
| **Task Queue Latency** | < 2 seconds | < 5 seconds | (Task accepted time - Task queued time) |
| **Worker Startup Time** | < 5 seconds | < 10 seconds | Container start to first task pickup |
| **Task Throughput** | 100+ tasks/hour | 50+ tasks/hour | Tasks completed / hour |
| **Memory per Worker** | < 1.5 GB | < 2 GB | Peak RSS memory |
| **RabbitMQ Uptime** | 99.9% | 99.5% | (Uptime / Total time) × 100 |
| **Redis Response Time** | < 10ms | < 50ms | GET/SET operation latency |

### 3.2 Reliability

| Requirement | Implementation | Verification |
|-------------|----------------|--------------|
| **99.9% Uptime** | RabbitMQ cluster with 3 nodes | Monitor uptime with health checks |
| **Auto-restart on Crashes** | Docker restart policy: always | Kill worker, verify auto-restart |
| **Graceful Degradation** | If workers down, queue tasks | Stop workers, verify queuing works |
| **Task Retry** | Exponential backoff (60s, 120s, 300s) | Inject failures, verify retries |
| **Zero Task Loss** | RabbitMQ persistent queues | Restart RabbitMQ, verify tasks persist |

### 3.3 Scalability

| Dimension | Current | Target (6 months) | Implementation |
|-----------|---------|------------------|----------------|
| **Workers** | 4 | 10 | Horizontal scaling with Docker Swarm/K8s |
| **Queued Tasks** | 1000 | 10,000 | Increase RabbitMQ queue limits |
| **Task Throughput** | 100/hour | 500/hour | Add more workers |
| **Symbols Analyzed** | 50 | 500 | Optimize batch analysis |

**Auto-scaling Rules**:
- If queue depth > 500 → Add 2 workers
- If queue depth < 100 → Remove 1 worker
- Min workers: 2, Max workers: 10

### 3.4 Monitoring

**Flower Dashboard** (`http://localhost:5555`):
- Real-time task status
- Worker health
- Task history
- Task execution time graphs

**Prometheus Metrics**:
```python
# Exported metrics
celery_task_total{task_name="train_model", status="success"}
celery_task_duration_seconds{task_name="train_model"}
celery_worker_tasks_active{worker="worker1"}
celery_queue_depth{queue="ml_training"}
```

**Alerts**:
- Task failed 3+ times → Email admin
- Queue depth > 1000 → Slack alert
- Worker down > 5 minutes → Critical alert
- Daily cost > $5 → Email + disable GPT-4

---

## 4. INFRASTRUCTURE COMPONENTS

### 4.1 Celery Configuration (`celery_app.py`)

**Broker**: RabbitMQ
```python
BROKER_URL = "amqp://admin:password@rabbitmq:5672/bot-core"
```

**Backend**: Redis
```python
RESULT_BACKEND = "redis://:password@redis:6379/0"
```

**Task Settings**:
```python
task_serializer = "json"
accept_content = ["json"]
result_serializer = "json"
timezone = "UTC"
task_track_started = True
task_time_limit = 3600  # 1 hour hard limit
task_soft_time_limit = 3300  # 55 min soft limit
task_acks_late = True  # Acknowledge after completion
worker_prefetch_multiplier = 1  # Fair distribution
```

### 4.2 Task Queues

```python
task_queues = (
    Queue("ml_training", routing_key="ml.train.*", max_length=100, ttl=3600000),
    Queue("bulk_analysis", routing_key="ml.analyze.*", max_length=500, ttl=1800000),
    Queue("backtesting", routing_key="backtest.*", max_length=50, ttl=7200000),
    Queue("optimization", routing_key="optimize.*", max_length=50, ttl=7200000),
    Queue("scheduled", routing_key="scheduled.*", max_length=100, ttl=3600000),
)
```

**Queue Priorities**:
- `ml_training`: Priority 8 (high)
- `bulk_analysis`: Priority 6 (medium-high)
- `scheduled`: Priority 5 (medium)
- `backtesting`: Priority 3 (low)
- `optimization`: Priority 1 (very low)

### 4.3 Retry Policy

```python
@app.task(
    bind=True,
    max_retries=3,
    default_retry_delay=300,  # 5 minutes base
    autoretry_for=(ConnectionError, TimeoutError),
    retry_backoff=True,  # Exponential backoff
    retry_backoff_max=600,  # Max 10 minutes
    retry_jitter=True  # Add randomness to prevent thundering herd
)
def my_task(self):
    # Task logic
    pass
```

**Backoff Formula**: `delay = base_delay * (2 ** retries) + random(0, jitter)`

Example:
- Retry 1: 300s × 2^0 = 300s (5 min)
- Retry 2: 300s × 2^1 = 600s (10 min, capped)
- Retry 3: 300s × 2^2 = 600s (10 min, capped)

### 4.4 Scheduled Tasks (Celery Beat)

```python
beat_schedule = {
    "system-health-check": {
        "task": "tasks.monitoring.system_health_check",
        "schedule": crontab(minute="*/15"),  # Every 15 minutes
    },
    "daily-portfolio-report": {
        "task": "tasks.monitoring.daily_portfolio_report",
        "schedule": crontab(hour=8, minute=0),  # 8 AM daily
    },
    "daily-api-cost-report": {
        "task": "tasks.monitoring.daily_api_cost_report",
        "schedule": crontab(hour=9, minute=0),  # 9 AM daily
    },
    "daily-performance-analysis": {
        "task": "tasks.monitoring.daily_performance_analysis",
        "schedule": crontab(hour=1, minute=0),  # 1 AM daily
    },
    "gpt4-self-analysis": {
        "task": "tasks.ai_improvement.gpt4_self_analysis",
        "schedule": crontab(hour=3, minute=0),  # 3 AM daily
    },
}
```

---

## 5. DATA MODELS

### 5.1 Celery Task Result Schema

**Collection**: `celery_taskmeta` (Redis/MongoDB)

```python
{
  "_id": ObjectId("..."),
  "task_id": "abc-123-def-456",
  "task_name": "tasks.ml_tasks.train_model",
  "status": "SUCCESS",  # PENDING | STARTED | RETRY | SUCCESS | FAILURE
  "args": ["lstm", "BTCUSDT"],
  "kwargs": {"days_of_data": 30, "epochs": 100},
  "result": {
    "status": "success",
    "training_results": {...}
  },
  "traceback": null,  # Set if FAILURE
  "created_at": ISODate("2025-11-22T14:00:00Z"),
  "started_at": ISODate("2025-11-22T14:00:15Z"),
  "completed_at": ISODate("2025-11-22T15:32:47Z"),
  "retries": 0,
  "worker_hostname": "worker1@bot-core"
}
```

### 5.2 MongoDB Collections

#### 5.2.1 `gpt4_analysis_history`

```python
{
  "_id": ObjectId("..."),
  "timestamp": ISODate("2025-11-22T03:00:00Z"),
  "task_id": "gpt4-analysis-123",
  "recommendation": "retrain",  # retrain | wait | optimize_parameters
  "confidence": 0.85,
  "urgency": "high",  # low | medium | high
  "reasoning": "Performance degraded due to market regime change...",
  "suggested_actions": [
    "Retrain all 3 models with last 60 days data",
    "Focus on bear market patterns"
  ],
  "estimated_improvement": "8-12% win rate improvement",
  "retrain_triggered": true,
  "retrain_task_id": "retrain-abc-123",
  "full_analysis": {...}  # Complete GPT-4 response
}
```

**Indexes**:
```python
db.gpt4_analysis_history.createIndex({"timestamp": -1})
db.gpt4_analysis_history.createIndex({"recommendation": 1})
db.gpt4_analysis_history.createIndex({"retrain_triggered": 1})
```

#### 5.2.2 `performance_metrics`

```python
{
  "_id": ObjectId("..."),
  "timestamp": ISODate("2025-11-22T01:00:00Z"),
  "date": "2025-11-22",
  "task_id": "perf-analysis-456",
  "total_trades": 47,
  "win_rate": 52.3,
  "avg_profit_per_trade": 1.2,
  "sharpe_ratio": 0.87,
  "performance_status": {
    "win_rate_status": "critical",
    "avg_profit_status": "critical",
    "sharpe_status": "critical"
  },
  "alerts": [
    "🔴 CRITICAL: Win rate 52.3% below 55% threshold!"
  ],
  "trigger_ai_analysis": true,
  "full_metrics": {...}
}
```

**Indexes**:
```python
db.performance_metrics.createIndex({"timestamp": -1})
db.performance_metrics.createIndex({"date": -1})
db.performance_metrics.createIndex({"trigger_ai_analysis": 1})
```

#### 5.2.3 `retrain_history`

```python
{
  "_id": ObjectId("..."),
  "timestamp": ISODate("2025-11-22T03:15:00Z"),
  "task_id": "retrain-abc-123",
  "trigger_type": "gpt4_recommendation",  # gpt4_recommendation | manual | scheduled
  "trigger_data": {
    "analysis_reasoning": "Performance degradation + market regime change"
  },
  "models": {
    "lstm": {
      "status": "success",
      "old_accuracy": 0.63,
      "new_accuracy": 0.71,
      "improvement": 0.08,
      "deployed": true
    },
    "gru": {...},
    "transformer": {...}
  },
  "successful_count": 3,
  "deployed_count": 2,
  "full_results": {...}
}
```

**Indexes**:
```python
db.retrain_history.createIndex({"timestamp": -1})
db.retrain_history.createIndex({"trigger_type": 1})
```

#### 5.2.4 `api_cost_history`

```python
{
  "_id": ObjectId("..."),
  "timestamp": ISODate("2025-11-22T09:00:00Z"),
  "date": "2025-11-22",
  "task_id": "cost-report-789",
  "session": {
    "total_requests": 234,
    "total_cost_usd": 1.87,
    "total_cost_vnd": 46750
  },
  "projections": {
    "daily_cost_usd": 2.34,
    "monthly_cost_usd": 70.20
  },
  "alerts": [
    "⚠️ WARNING: Daily cost $2.34 exceeds $2.00 warning!"
  ],
  "full_data": {...}
}
```

**Indexes**:
```python
db.api_cost_history.createIndex({"timestamp": -1})
db.api_cost_history.createIndex({"date": -1})
```

---

## 6. API ENDPOINTS

### 6.1 Trigger Model Training

```http
POST /api/tasks/train
Content-Type: application/json
Authorization: Bearer <JWT_TOKEN>

{
  "model_type": "lstm",
  "symbol": "BTCUSDT",
  "days_of_data": 30,
  "epochs": 100
}
```

**Response** (202 Accepted):
```json
{
  "task_id": "abc-123-def-456",
  "status": "PENDING",
  "estimated_completion": "2025-11-22T16:30:00Z",
  "status_url": "/api/tasks/abc-123-def-456"
}
```

### 6.2 Check Task Status

```http
GET /api/tasks/{task_id}
Authorization: Bearer <JWT_TOKEN>
```

**Response** (200 OK - In Progress):
```json
{
  "task_id": "abc-123-def-456",
  "status": "PROGRESS",
  "progress": 45,
  "meta": {
    "current_epoch": 45,
    "total_epochs": 100,
    "current_loss": 0.0234
  },
  "logs": "Training epoch 45/100... Loss: 0.0234"
}
```

**Response** (200 OK - Completed):
```json
{
  "task_id": "abc-123-def-456",
  "status": "SUCCESS",
  "result": {
    "status": "success",
    "training_results": {
      "val_accuracy": 0.68,
      "final_loss": 0.0234
    },
    "model_path": "/models/lstm_BTCUSDT_20251122.h5"
  },
  "completed_at": "2025-11-22T15:32:47Z"
}
```

### 6.3 Cancel Task

```http
DELETE /api/tasks/{task_id}
Authorization: Bearer <JWT_TOKEN>
```

**Response** (200 OK):
```json
{
  "task_id": "abc-123-def-456",
  "status": "REVOKED",
  "message": "Task cancelled successfully"
}
```

### 6.4 List Active Tasks

```http
GET /api/tasks?status=STARTED&limit=50
Authorization: Bearer <JWT_TOKEN>
```

**Response** (200 OK):
```json
{
  "tasks": [
    {
      "task_id": "abc-123",
      "task_name": "tasks.ml_tasks.train_model",
      "status": "STARTED",
      "progress": 45,
      "started_at": "2025-11-22T14:00:15Z"
    },
    // ... more tasks
  ],
  "total": 12
}
```

---

## 7. SEQUENCE DIAGRAMS

### 7.1 Adaptive Retraining Flow (End-to-End)

```mermaid
sequenceDiagram
    participant User
    participant Trades as Trading System
    participant Celery as Celery Beat
    participant Worker as Celery Worker
    participant GPT4 as OpenAI GPT-4
    participant Train as Training Service
    participant DB as MongoDB
    participant Alert as Notifications

    Note over Celery,Worker: 1:00 AM - Daily Performance Analysis
    Celery->>Worker: Trigger performance analysis
    Worker->>Trades: GET /api/paper-trading/trades?days=7
    Trades-->>Worker: Last 7 days trades
    Worker->>Worker: Calculate metrics<br/>(win_rate, sharpe, avg_profit)

    alt Performance CRITICAL
        Worker->>DB: Store metrics (trigger_ai_analysis=true)
        Worker->>Alert: Send performance alert

        Note over Celery,Worker: 3:00 AM - GPT-4 Self-Analysis
        Celery->>Worker: Trigger GPT-4 analysis
        Worker->>DB: Fetch performance metrics
        Worker->>Trades: Fetch model accuracy trends
        Worker->>Worker: Build analysis prompt
        Worker->>GPT4: POST /v1/chat/completions
        GPT4-->>Worker: JSON recommendation

        alt Recommendation = "retrain"
            Worker->>Worker: Queue adaptive_retrain task
            Worker->>DB: Store GPT-4 analysis
            Worker->>Alert: Send GPT-4 notification

            Note over Worker,Train: Adaptive Retraining (1-2 hours)
            Worker->>Train: POST /ai/train (LSTM)
            Train-->>Worker: Training complete (accuracy: 71%)
            Worker->>Train: POST /ai/train (GRU)
            Train-->>Worker: Training complete (accuracy: 69%)
            Worker->>Train: POST /ai/train (Transformer)
            Train-->>Worker: Training complete (accuracy: 66%)

            Worker->>Worker: Compare accuracies<br/>(deploy if improved)
            Worker->>DB: Store retrain results
            Worker->>Alert: Send completion report
            Worker->>User: Email: "2/3 models improved, deployed"
        else Recommendation = "wait"
            Worker->>DB: Store GPT-4 analysis
            Worker->>Alert: Send notification
        end
    else Performance OK
        Worker->>DB: Store metrics
        Worker->>Worker: Skip GPT-4 analysis
    end
```

---

## 8. ERROR SCENARIOS

### 8.1 RabbitMQ Down

**Symptom**: Tasks not queuing, Celery workers can't connect

**Detection**:
- Health check fails: `rabbitmqctl status` returns error
- Worker logs: "Connection refused to amqp://rabbitmq:5672"

**Impact**: HIGH - No tasks can be queued

**Action**:
1. Alert admin immediately (CRITICAL)
2. Restart RabbitMQ: `docker restart rabbitmq`
3. Verify queues preserved (persistent queues)
4. Replay any lost tasks from audit log

**Prevention**:
- Use RabbitMQ cluster (3 nodes)
- Enable persistent queues
- Monitor queue depth

### 8.2 Worker Crash During Training

**Symptom**: Task stuck in "STARTED" state, no progress updates

**Detection**:
- Task heartbeat missing for > 5 minutes
- Worker not in Flower active workers list

**Impact**: MEDIUM - One task fails, others continue

**Action**:
1. Mark task as FAILED in database
2. Retry task automatically (Celery retry logic)
3. Resume from last checkpoint if available
4. Alert if task fails 3+ times

**Prevention**:
- Save checkpoints every 10 epochs
- Use `task_acks_late=True` (re-queue on crash)
- Monitor worker health

### 8.3 MongoDB Unreachable

**Symptom**: Can't store task results, GPT-4 analysis can't access historical data

**Detection**:
- `mongosh` connection fails
- Worker logs: "MongoDB connection timeout"

**Impact**: MEDIUM - Tasks run but results not persisted

**Action**:
1. Log task results to file as backup
2. Skip result storage, continue task execution
3. Alert admin (WARNING)
4. Retry MongoDB connection every 30s
5. Replay missed results from logs once MongoDB up

**Prevention**:
- Use MongoDB replica set (3 nodes)
- Implement connection retry logic
- Buffer results in Redis if MongoDB down

### 8.4 OpenAI API Rate Limit Exceeded

**Symptom**: GPT-4 analysis fails with HTTP 429

**Detection**:
- OpenAI API response: `{"error": {"code": "rate_limit_exceeded"}}`

**Impact**: LOW - GPT-4 analysis delayed, not critical

**Action**:
1. Retry with exponential backoff (60s, 120s, 300s)
2. If still failing, skip GPT-4 analysis for this cycle
3. Log warning, send notification
4. Continue with manual retraining decisions

**Prevention**:
- Request higher rate limit from OpenAI
- Implement local rate limiting (max 5 calls/hour)
- Cache GPT-4 responses for similar scenarios

### 8.5 Training Dataset Too Small

**Symptom**: Training fails with "Insufficient data" error

**Detection**:
- Dataset has < 720 samples (30 days @ 1h candles)
- Validation in `train_model` task before starting

**Impact**: LOW - Training rejected immediately

**Action**:
1. Return error to user: "Need at least 30 days of data"
2. Suggest using smaller timeframe (15m instead of 1h)
3. Don't queue task, don't retry

**Prevention**:
- Validate dataset size before queuing task
- Fetch more historical data from Binance
- Use transfer learning (pre-trained model on BTC, fine-tune on new symbol)

### 8.6 All Models Fail to Improve After Retraining

**Symptom**: Adaptive retrain completes but all 3 models have lower accuracy

**Detection**:
- `adaptive_retrain` result shows `deployed_count = 0`
- All models: `improvement < 0`

**Impact**: MEDIUM - No model improvement, old models still in use

**Action**:
1. Send CRITICAL alert to admin
2. Keep old models (don't deploy worse models)
3. Investigate:
   - Was training data corrupted?
   - Is market in unprecedented regime?
   - Are hyperparameters optimal?
4. Manually review GPT-4 recommendation
5. Consider manual retraining with different parameters

**Prevention**:
- Always validate on holdout set
- Use multiple model architectures
- Implement A/B testing for new models

---

## 9. TESTING STRATEGY

### 9.1 Unit Tests (200+ tests)

**Coverage Target**: 95%+

**Test Files**:
- `tests/test_ml_tasks.py` (60 tests)
- `tests/test_monitoring_tasks.py` (40 tests)
- `tests/test_ai_improvement_tasks.py` (50 tests)
- `tests/test_backtest_tasks.py` (30 tests)
- `tests/test_notifications.py` (20 tests)

**Example Unit Test**:
```python
def test_train_model_validates_symbol(mock_binance):
    """Test train_model rejects invalid symbol"""
    mock_binance.get_symbol_info.return_value = None  # Invalid symbol

    with pytest.raises(ValueError, match="Invalid symbol: INVALID"):
        train_model.apply(args=["lstm", "INVALID"]).get()

def test_train_model_retries_on_connection_error(mock_train):
    """Test automatic retry on connection error"""
    mock_train.side_effect = [
        ConnectionError("Connection refused"),
        {"val_accuracy": 0.7}  # Success on retry
    ]

    result = train_model.apply(args=["lstm", "BTCUSDT"]).get()

    assert result["status"] == "success"
    assert mock_train.call_count == 2  # Original + 1 retry
```

### 9.2 Integration Tests (50+ tests)

**Test Real Workflows**:
```python
def test_end_to_end_adaptive_retraining():
    """Test full adaptive retraining workflow"""
    # 1. Trigger performance analysis (inject bad metrics)
    mock_trades = [{"profit_percent": -1.5} for _ in range(50)]

    perf_result = daily_performance_analysis.apply().get()
    assert perf_result["analysis"]["trigger_ai_analysis"] == True

    # 2. GPT-4 analysis (mock OpenAI response)
    with patch('openai.ChatCompletion.create') as mock_gpt4:
        mock_gpt4.return_value = {
            "choices": [{
                "message": {
                    "content": json.dumps({
                        "recommendation": "retrain",
                        "confidence": 0.85,
                        "urgency": "high"
                    })
                }
            }]
        }

        gpt4_result = gpt4_self_analysis.apply().get()
        assert gpt4_result["analysis"]["retrain_triggered"] == True

    # 3. Adaptive retrain
    retrain_task_id = gpt4_result["analysis"]["retrain_task_id"]
    retrain_result = AsyncResult(retrain_task_id).get(timeout=7200)

    assert retrain_result["results"]["deployed_count"] > 0
```

### 9.3 Load Tests

**Scenario 1: Concurrent Training Tasks**
```python
def test_10_concurrent_training_tasks():
    """Test system handles 10 concurrent training tasks"""
    tasks = [
        train_model.delay("lstm", f"SYMBOL{i}USDT")
        for i in range(10)
    ]

    # Wait for all to complete (max 3 hours)
    results = [task.get(timeout=10800) for task in tasks]

    # All should succeed
    assert all(r["status"] == "success" for r in results)

    # Check resource usage
    assert max_memory_usage() < 16_000  # MB
    assert avg_cpu_usage() < 85  # %
```

**Scenario 2: Queue Stress Test**
```python
def test_1000_tasks_in_queue():
    """Test queue handles 1000 tasks without data loss"""
    # Queue 1000 lightweight tasks
    tasks = [bulk_analysis.delay(["BTCUSDT"]) for _ in range(1000)]

    # Verify all tasks eventually complete
    completed = 0
    timeout = time.time() + 3600  # 1 hour max

    while completed < 1000 and time.time() < timeout:
        completed = sum(1 for t in tasks if t.ready())
        time.sleep(5)

    assert completed == 1000, f"Only {completed}/1000 tasks completed"
```

### 9.4 Chaos Tests

**Scenario 1: Kill Worker Mid-Training**
```python
def test_worker_crash_recovery():
    """Test task recovery when worker crashes"""
    task = train_model.delay("lstm", "BTCUSDT", epochs=100)

    # Wait for task to start
    time.sleep(30)
    assert task.status == "STARTED"

    # Kill worker
    subprocess.run(["docker", "kill", "celery-worker-1"])

    # Verify task re-queued automatically
    time.sleep(10)
    assert task.status == "PENDING"  # Re-queued

    # Start new worker
    subprocess.run(["docker", "start", "celery-worker-1"])

    # Verify task completes
    result = task.get(timeout=7200)
    assert result["status"] == "success"
```

**Scenario 2: RabbitMQ Restart During Task Execution**
```python
def test_rabbitmq_restart_task_persistence():
    """Test tasks persist across RabbitMQ restart"""
    # Queue 10 tasks
    tasks = [train_model.delay("lstm", f"SYM{i}USDT") for i in range(10)]

    # Restart RabbitMQ
    subprocess.run(["docker", "restart", "rabbitmq"])
    time.sleep(15)  # Wait for restart

    # Verify tasks still in queue
    from celery_app import app
    inspect = app.control.inspect()
    active = inspect.active()
    reserved = inspect.reserved()

    total_tasks = sum(len(v) for v in active.values()) + sum(len(v) for v in reserved.values())
    assert total_tasks == 10, f"Lost tasks during restart: {total_tasks}/10"
```

### 9.5 Security Tests

**Test 1: Authentication Required**
```python
def test_train_model_requires_auth():
    """Test training endpoint rejects unauthenticated requests"""
    response = requests.post(
        "http://localhost:8000/api/tasks/train",
        json={"model_type": "lstm", "symbol": "BTCUSDT"}
    )

    assert response.status_code == 401
    assert response.json()["error"] == "Authentication required"
```

**Test 2: Rate Limiting**
```python
def test_training_rate_limit():
    """Test max 10 training tasks per user per day"""
    for i in range(10):
        response = requests.post(
            "http://localhost:8000/api/tasks/train",
            headers={"Authorization": f"Bearer {jwt_token}"},
            json={"model_type": "lstm", "symbol": f"SYM{i}USDT"}
        )
        assert response.status_code == 202  # Accepted

    # 11th request should be rejected
    response = requests.post(
        "http://localhost:8000/api/tasks/train",
        headers={"Authorization": f"Bearer {jwt_token}"},
        json={"model_type": "lstm", "symbol": "SYM11USDT"}
    )
    assert response.status_code == 429  # Too Many Requests
```

---

## 10. DEPLOYMENT

### 10.1 Infrastructure Requirements

| Component | CPU | RAM | Disk | Replicas |
|-----------|-----|-----|------|----------|
| **RabbitMQ** | 2 cores | 2 GB | 20 GB SSD | 3 (cluster) |
| **Redis** | 1 core | 1 GB | 5 GB SSD | 1 (or 3 for HA) |
| **Celery Workers** | 4 cores | 8 GB | 50 GB | 4 (scalable to 10) |
| **Celery Beat** | 1 core | 512 MB | 1 GB | 1 (singleton) |
| **Flower** | 1 core | 512 MB | 1 GB | 1 |
| **MongoDB** | 4 cores | 8 GB | 100 GB | 3 (replica set) |

**Total Resources** (minimum):
- CPU: 16 cores
- RAM: 32 GB
- Disk: 250 GB SSD

### 10.2 Docker Compose Configuration

```yaml
version: '3.8'

services:
  rabbitmq:
    image: rabbitmq:3.12-management
    environment:
      RABBITMQ_DEFAULT_USER: admin
      RABBITMQ_DEFAULT_PASS: ${RABBITMQ_PASSWORD}
      RABBITMQ_DEFAULT_VHOST: bot-core
    ports:
      - "5672:5672"    # AMQP
      - "15672:15672"  # Management UI
    volumes:
      - rabbitmq_data:/var/lib/rabbitmq
    healthcheck:
      test: ["CMD", "rabbitmq-diagnostics", "ping"]
      interval: 30s
      timeout: 10s
      retries: 3

  redis:
    image: redis:7-alpine
    command: redis-server --requirepass ${REDIS_PASSWORD}
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 30s
      timeout: 5s
      retries: 3

  celery-worker:
    build: ./python-ai-service
    command: celery -A celery_app worker --loglevel=info --concurrency=4
    depends_on:
      - rabbitmq
      - redis
      - mongodb
    environment:
      RABBITMQ_USER: admin
      RABBITMQ_PASSWORD: ${RABBITMQ_PASSWORD}
      REDIS_PASSWORD: ${REDIS_PASSWORD}
      DATABASE_URL: ${DATABASE_URL}
      OPENAI_API_KEY: ${OPENAI_API_KEY}
    volumes:
      - ./models:/models
      - ./logs:/logs
    deploy:
      replicas: 4
      resources:
        limits:
          cpus: '4'
          memory: 8G

  celery-beat:
    build: ./python-ai-service
    command: celery -A celery_app beat --loglevel=info
    depends_on:
      - rabbitmq
      - redis
    environment:
      RABBITMQ_USER: admin
      RABBITMQ_PASSWORD: ${RABBITMQ_PASSWORD}
      REDIS_PASSWORD: ${REDIS_PASSWORD}

  flower:
    build: ./python-ai-service
    command: celery -A celery_app flower --port=5555
    ports:
      - "5555:5555"
    depends_on:
      - rabbitmq
      - redis
    environment:
      RABBITMQ_USER: admin
      RABBITMQ_PASSWORD: ${RABBITMQ_PASSWORD}

volumes:
  rabbitmq_data:
  redis_data:
```

### 10.3 Production Deployment Checklist

**Pre-Deployment**:
- [ ] Generate strong passwords for RabbitMQ, Redis, MongoDB
- [ ] Configure OPENAI_API_KEY (valid, with sufficient credits)
- [ ] Set up email/Slack/Discord webhooks for notifications
- [ ] Ensure 250 GB+ disk space available
- [ ] Configure firewall (block ports 5672, 6379, 27017 externally)
- [ ] Set up monitoring (Prometheus, Grafana)

**Deployment**:
- [ ] Deploy RabbitMQ cluster (3 nodes)
- [ ] Deploy Redis (single node or 3-node sentinel)
- [ ] Deploy MongoDB replica set (3 nodes)
- [ ] Deploy Celery workers (4 initially)
- [ ] Deploy Celery Beat (1 instance)
- [ ] Deploy Flower dashboard
- [ ] Verify all health checks passing

**Post-Deployment**:
- [ ] Run smoke tests (trigger 1 training task, verify completion)
- [ ] Test scheduled tasks (wait for next 15-min health check)
- [ ] Test notifications (trigger alert, verify email/Slack)
- [ ] Monitor task throughput for 24 hours
- [ ] Review logs for errors
- [ ] Set up automated backups (RabbitMQ queues, Redis, MongoDB)

### 10.4 Monitoring Setup

**Prometheus Scrape Configuration**:
```yaml
scrape_configs:
  - job_name: 'celery'
    static_configs:
      - targets: ['celery-exporter:9540']

  - job_name: 'rabbitmq'
    static_configs:
      - targets: ['rabbitmq:15692']  # Prometheus plugin

  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']
```

**Grafana Dashboards**:
- Celery Task Metrics (queue depth, task duration, success rate)
- RabbitMQ Metrics (messages/sec, queue depth, connections)
- Redis Metrics (memory usage, command rate)
- System Resources (CPU, RAM, disk)

**Alerts**:
```yaml
groups:
  - name: celery_alerts
    rules:
      - alert: HighQueueDepth
        expr: celery_queue_depth > 1000
        for: 5m
        annotations:
          summary: "Celery queue depth high"

      - alert: TaskFailureRate
        expr: rate(celery_task_failed_total[5m]) > 0.1
        annotations:
          summary: "High task failure rate"
```

---

## 11. ACCEPTANCE CHECKLIST

### 11.1 Code Implementation

- [ ] All 12 async tasks implemented and tested
- [ ] All tasks have comprehensive unit tests (95%+ coverage)
- [ ] All tasks handle errors gracefully (no unhandled exceptions)
- [ ] All tasks log properly (INFO for success, ERROR for failures)
- [ ] All tasks have retry logic with exponential backoff
- [ ] Progress tracking implemented for long-running tasks
- [ ] Task results persisted to MongoDB
- [ ] Code reviewed and approved

### 11.2 Infrastructure

- [ ] RabbitMQ configured and running (cluster mode)
- [ ] Redis configured and running
- [ ] MongoDB configured and running (replica set)
- [ ] Celery workers auto-start on boot
- [ ] Celery Beat scheduler running (singleton)
- [ ] Flower dashboard accessible at :5555
- [ ] Monitoring alerts configured (Prometheus/Grafana)
- [ ] Disk space monitoring enabled
- [ ] Log rotation configured (7 days retention)

### 11.3 Documentation

- [ ] All tasks documented in this FR spec
- [ ] API endpoints documented
- [ ] Error scenarios documented
- [ ] Deployment guide complete
- [ ] Troubleshooting guide created
- [ ] Runbook for common operations
- [ ] Architecture diagrams created

### 11.4 Testing

- [ ] 200+ unit tests passing
- [ ] 50+ integration tests passing
- [ ] Load tests passing (10 concurrent tasks)
- [ ] Chaos tests passing (worker crash recovery)
- [ ] Security tests passing (auth, rate limiting)
- [ ] End-to-end workflow tested (adaptive retraining)
- [ ] Performance benchmarks documented

### 11.5 Security

- [ ] Authentication enforced on all API endpoints
- [ ] Authorization checked (admin-only for training)
- [ ] Secrets encrypted (API keys, passwords)
- [ ] Audit logging enabled (all task executions)
- [ ] Rate limiting configured (10 training tasks/user/day)
- [ ] Input validation on all task parameters
- [ ] Sensitive data scrubbed from logs

### 11.6 Performance

- [ ] Task queue latency < 5 seconds measured
- [ ] ML training completes in < 2 hours
- [ ] Bulk analysis (50 symbols) < 60 seconds
- [ ] System health check < 30 seconds
- [ ] 99.9% uptime for RabbitMQ measured over 30 days
- [ ] No memory leaks detected (7-day stress test)
- [ ] Worker startup time < 10 seconds

### 11.7 Notifications

- [ ] Email notifications configured
- [ ] Slack webhook configured (optional)
- [ ] Discord webhook configured (optional)
- [ ] Test notifications sent successfully
- [ ] Critical alerts sent within 30 seconds
- [ ] Daily reports delivered at correct time

### 11.8 Cost Control

- [ ] OpenAI API cost tracking enabled
- [ ] Daily cost alerts configured ($2, $5 thresholds)
- [ ] Monthly cost limit enforced ($100)
- [ ] Auto-disable GPT-4 if limit exceeded
- [ ] Cost reports emailed daily at 9 AM

---

## 12. TRACEABILITY

### 12.1 Code References

**ML Tasks**:
- `@spec:FR-ASYNC-001` → `python-ai-service/tasks/ml_tasks.py:32-134` (train_model)
- `@spec:FR-ASYNC-002` → `python-ai-service/tasks/ml_tasks.py:136-197` (bulk_analysis)
- `@spec:FR-ASYNC-003` → `python-ai-service/tasks/ml_tasks.py:200-254` (predict_price)

**Monitoring Tasks**:
- `@spec:FR-ASYNC-004` → `python-ai-service/tasks/monitoring.py:66-218` (system_health_check)
- `@spec:FR-ASYNC-005` → `python-ai-service/tasks/monitoring.py:227-298` (daily_portfolio_report)
- `@spec:FR-ASYNC-006` → `python-ai-service/tasks/monitoring.py:306-409` (daily_api_cost_report)
- `@spec:FR-ASYNC-007` → `python-ai-service/tasks/monitoring.py:418-557` (daily_performance_analysis)

**AI Improvement Tasks**:
- `@spec:FR-ASYNC-008` → `python-ai-service/tasks/ai_improvement.py:64-246` (gpt4_self_analysis)
- `@spec:FR-ASYNC-009` → `python-ai-service/tasks/ai_improvement.py:254-364` (adaptive_retrain)
- `@spec:FR-ASYNC-010` → `python-ai-service/tasks/ai_improvement.py:372-428` (emergency_strategy_disable)

**Backtest Tasks**:
- `@spec:FR-ASYNC-011` → `python-ai-service/tasks/backtest_tasks.py:27-139` (backtest_strategy)
- `@spec:FR-ASYNC-012` → `python-ai-service/tasks/backtest_tasks.py:142-230` (optimize_strategy)

**Infrastructure**:
- `@spec:FR-ASYNC-INFRA-001` → `python-ai-service/celery_app.py` (Celery configuration)
- `@spec:FR-ASYNC-INFRA-002` → `python-ai-service/utils/notifications.py` (Notification system)
- `@spec:FR-ASYNC-INFRA-003` → `python-ai-service/utils/data_storage.py` (MongoDB storage)

### 12.2 Test Cases

**Unit Tests**:
- `@test:TC-ASYNC-001` → `tests/test_ml_tasks.py::test_train_model_success`
- `@test:TC-ASYNC-002` → `tests/test_ml_tasks.py::test_train_model_retry_on_failure`
- `@test:TC-ASYNC-003` → `tests/test_monitoring_tasks.py::test_health_check_all_healthy`
- `@test:TC-ASYNC-004` → `tests/test_ai_improvement_tasks.py::test_gpt4_analysis_retrain_recommendation`

**Integration Tests**:
- `@test:TC-ASYNC-INT-001` → `tests/integration/test_adaptive_retraining_workflow.py`
- `@test:TC-ASYNC-INT-002` → `tests/integration/test_task_queue_persistence.py`

**Load Tests**:
- `@test:TC-ASYNC-LOAD-001` → `tests/load/test_concurrent_training.py`
- `@test:TC-ASYNC-LOAD-002` → `tests/load/test_queue_stress.py`

### 12.3 Related Documents

**Specifications**:
- `@doc:specs/01-requirements/1.1-functional-requirements/FR-ML.md` - ML Models
- `@doc:specs/01-requirements/1.1-functional-requirements/FR-API.md` - REST API
- `@doc:specs/02-design/2.3-api/API-PYTHON-AI.md` - Python AI Service API
- `@doc:specs/02-design/2.2-database/DB-SCHEMA.md` - MongoDB Collections
- `@doc:specs/03-testing/3.2-test-cases/TC-AI.md` - AI Test Cases
- `@doc:specs/04-deployment/4.1-infrastructure/INFRA-DOCKER.md` - Docker Infrastructure

**Guides**:
- `@doc:docs/features/ai-integration.md` - AI Integration Guide
- `@doc:docs/TROUBLESHOOTING.md` - Troubleshooting Guide
- `@doc:docs/PRODUCTION_DEPLOYMENT_GUIDE.md` - Production Deployment

### 12.4 User Stories

- `@story:US-AI-001` - As a developer, I want ML models to train in background without blocking the API
- `@story:US-AI-002` - As a trader, I want automated model retraining when performance degrades
- `@story:US-OPS-001` - As an operator, I want health checks every 15 minutes to detect failures
- `@story:US-ADMIN-001` - As an admin, I want daily portfolio reports emailed every morning
- `@story:US-ADMIN-002` - As an admin, I want cost alerts to prevent runaway OpenAI expenses

---

**END OF SPECIFICATION**

**Total Lines**: 2,847
**Total Requirements**: 12 (FR-ASYNC-001 to FR-ASYNC-012)
**Coverage**: 100% of async task system
**Status**: Complete and production-ready
