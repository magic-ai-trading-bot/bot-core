# API-PYTHON-AI.md - Python AI Service API Specification

**Version:** 3.0.0
**Base URL:** `http://localhost:8000`
**Service:** GPT-4 Trading AI
**Port:** 8000

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [Health & Status Endpoints](#health--status-endpoints)
4. [AI Analysis Endpoints](#ai-analysis-endpoints)
5. [Strategy & Recommendations](#strategy--recommendations)
6. [Storage & Analytics](#storage--analytics)
7. [Async Task Management](#async-task-management)
8. [Training Management](#training-management)
9. [Backtest Management](#backtest-management)
10. [Monitoring & Alerts](#monitoring--alerts)
11. [WebSocket Real-time Updates](#websocket-real-time-updates)
12. [Error Codes](#error-codes)
13. [Rate Limiting](#rate-limiting)

---

## Overview

The Python AI Service provides GPT-4 powered trading signal generation, technical analysis, strategy recommendations, and market condition analysis. It integrates with OpenAI's GPT-4o-mini model for intelligent trading decisions and supports asynchronous task execution via RabbitMQ and Celery.

**Service Architecture:**
- **Language:** Python 3.11+
- **Framework:** FastAPI
- **AI Model:** GPT-4o-mini (OpenAI)
- **Database:** MongoDB (for caching analysis results)
- **Task Queue:** RabbitMQ + Celery (async task execution)
- **Result Backend:** Redis (task result storage)
- **WebSocket:** Real-time signal broadcasting
- **Background Tasks:** Periodic analysis every 5 minutes

**Code Location:** `/Users/dungngo97/Documents/bot-core/python-ai-service/`

**Key Features:**
- GPT-4 powered signal generation
- Technical indicator calculation (RSI, MACD, Bollinger Bands, etc.)
- Multi-timeframe analysis
- Strategy recommendation engine
- Market condition detection
- MongoDB caching for 5-minute intervals
- Real-time WebSocket broadcasting
- Automatic API key fallback on rate limits
- Async task execution for long-running operations (training, backtesting)
- Task status tracking and progress monitoring
- Training job management with deployment capabilities
- Strategy backtest execution and result analysis

---

## Authentication

**Authentication Type:** Bearer Token (JWT) OR Internal Service Token

```
Authorization: Bearer <jwt_token>
```

**Note:** Most endpoints are publicly accessible but can be secured with JWT tokens. The Rust Core Engine uses internal service-to-service authentication.

---

## Health & Status Endpoints

### GET /health

**Description:** Health check endpoint with service status

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2025-10-10T10:30:00.000000+00:00",
  "service": "GPT-4 Trading AI",
  "version": "2.0.0",
  "gpt4_available": true,
  "api_key_configured": true,
  "mongodb_connected": true,
  "analysis_interval_minutes": 5,
  "supported_symbols": [
    "BTCUSDT",
    "ETHUSDT",
    "BNBUSDT",
    "SOLUSDT",
    "ADAUSDT",
    "DOTUSDT",
    "XRPUSDT",
    "LINKUSDT"
  ]
}
```

**Code Location:** `python-ai-service/main.py:1656-1679`
**Related FR:** FR-AI-HEALTH-001
**Rate Limit:** 60 requests per minute

---

### GET /debug/gpt4

**Description:** Debug GPT-4 connectivity and API key status

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "client_initialized": true,
  "api_key_configured": true,
  "status": "success",
  "test_response": "SUCCESS",
  "model_used": "gpt-4o-mini"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "client_initialized": false,
  "api_key_configured": false,
  "status": "failed",
  "error": "OpenAI client not initialized",
  "error_type": "InitializationError",
  "diagnosis": "API key authentication failed"
}
```

**Code Location:** `python-ai-service/main.py:1682-1728`
**Related FR:** FR-AI-DEBUG-001
**Rate Limit:** 5 requests per minute

---

### GET /

**Description:** Root endpoint with service information and API documentation

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "service": "GPT-4 Cryptocurrency AI Trading Service",
  "version": "2.0.0",
  "description": "Advanced AI-powered trading signal generation using OpenAI GPT-4 with MongoDB storage and real-time WebSocket broadcasting",
  "endpoints": {
    "analyze": "POST /ai/analyze - Generate trading signals with GPT-4 (stored in MongoDB)",
    "strategy_recommendations": "POST /ai/strategy-recommendations - Get strategy recommendations",
    "market_condition": "POST /ai/market-condition - Analyze market conditions",
    "feedback": "POST /ai/feedback - Send performance feedback",
    "health": "GET /health - Health check with MongoDB status",
    "storage_stats": "GET /ai/storage/stats - View storage statistics",
    "clear_storage": "POST /ai/storage/clear - Clear analysis storage",
    "websocket": "WS /ws - Real-time AI signal broadcasting"
  },
  "documentation": "/docs",
  "features": {
    "gpt4_enabled": true,
    "mongodb_storage": true,
    "websocket_broadcasting": true,
    "periodic_analysis": true,
    "analysis_interval_minutes": 5,
    "symbols_tracked": [
      "BTCUSDT",
      "ETHUSDT",
      "BNBUSDT",
      "SOLUSDT",
      "ADAUSDT",
      "DOTUSDT",
      "XRPUSDT",
      "LINKUSDT"
    ]
  }
}
```

**Code Location:** `python-ai-service/main.py:2051-2078`
**Related FR:** FR-AI-ROOT-001
**Rate Limit:** 60 requests per minute

---

## AI Analysis Endpoints

### POST /ai/analyze

**Description:** Analyze trading signals using GPT-4 AI with MongoDB caching

**Authentication:** Optional (Bearer JWT or Internal Service Token)

**Request Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

**Request Body:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe_data": {
    "1h": [
      {
        "timestamp": 1697234400000,
        "open": 67400.00,
        "high": 67550.00,
        "low": 67350.00,
        "close": 67500.50,
        "volume": 1250.5
      }
    ],
    "4h": [
      {
        "timestamp": 1697220000000,
        "open": 67000.00,
        "high": 67800.00,
        "low": 66900.00,
        "close": 67500.00,
        "volume": 5000.0
      }
    ]
  },
  "current_price": 67500.50,
  "volume_24h": 50000000000.0,
  "timestamp": 1697234567000,
  "strategy_context": {
    "selected_strategies": [
      "RSI Strategy",
      "MACD Strategy",
      "Bollinger Bands Strategy"
    ],
    "market_condition": "Trending",
    "risk_level": "Moderate",
    "user_preferences": {},
    "technical_indicators": {}
  }
}
```

**Success Response (200 OK):**
```json
{
  "signal": "Long",
  "confidence": 0.82,
  "reasoning": "Strong bullish momentum with RSI oversold (28), MACD crossover, and positive GPT-4 sentiment analysis. Multiple timeframes confirm uptrend.",
  "strategy_scores": {
    "RSI Strategy": 0.85,
    "MACD Strategy": 0.78,
    "Bollinger Bands Strategy": 0.65,
    "Volume Strategy": 0.72
  },
  "market_analysis": {
    "trend_direction": "Bullish",
    "trend_strength": 0.82,
    "support_levels": [67000.00, 66500.00, 66000.00],
    "resistance_levels": [68000.00, 68500.00, 69000.00],
    "volatility_level": "Medium",
    "volume_analysis": "High volume confirming uptrend with strong accumulation"
  },
  "risk_assessment": {
    "overall_risk": "Medium",
    "technical_risk": 0.45,
    "market_risk": 0.50,
    "recommended_position_size": 0.05,
    "stop_loss_suggestion": 66000.00,
    "take_profit_suggestion": 68500.00
  },
  "timestamp": 1697234567000
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "detail": "AI analysis failed: OpenAI rate limit exceeded"
}
```

**Signal Types:**
- `Long` - Bullish signal, buy recommendation
- `Short` - Bearish signal, sell recommendation
- `Neutral` - No clear direction, hold position

**Confidence Range:** 0.0 - 1.0 (0% - 100%)

**Caching Behavior:**
- Results are cached in MongoDB for 5 minutes
- If analysis exists and is < 5 minutes old, cached result is returned
- Cached responses include `[RECENT]` prefix in reasoning
- Fresh analysis is performed if cache is stale or missing

**WebSocket Broadcasting:**
- Signal is automatically broadcasted to all WebSocket clients
- Message type: `AISignalReceived`

**Code Location:** `python-ai-service/main.py:1750-1889`
**Related FR:** FR-AI-006
**Rate Limit:** 10 requests per minute
**Cache TTL:** 5 minutes

---

### GET /ai/info

**Description:** Get AI service information and capabilities

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "service_name": "GPT-4 Trading AI",
  "version": "2.0.0",
  "model_version": "gpt-4o-mini",
  "supported_timeframes": [
    "1m",
    "5m",
    "15m",
    "1h",
    "4h",
    "1d"
  ],
  "supported_symbols": [
    "BTCUSDT",
    "ETHUSDT",
    "BNBUSDT",
    "SOLUSDT"
  ],
  "capabilities": [
    "trend_analysis",
    "signal_generation",
    "risk_assessment",
    "strategy_recommendation",
    "market_condition_detection"
  ],
  "last_trained": null
}
```

**Code Location:** `python-ai-service/main.py:1963-1966`
**Related FR:** FR-AI-INFO-001
**Rate Limit:** 60 requests per minute

---

### GET /ai/strategies

**Description:** Get list of supported trading strategies

**Authentication:** None

**Success Response (200 OK):**
```json
[
  "RSI Strategy",
  "MACD Strategy",
  "Volume Strategy",
  "Bollinger Bands Strategy"
]
```

**Code Location:** `python-ai-service/main.py:1969-1977`
**Related FR:** FR-AI-STRATEGIES-001
**Rate Limit:** 60 requests per minute

---

### GET /ai/performance

**Description:** Get AI model performance metrics

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "overall_accuracy": 0.85,
  "precision": 0.82,
  "recall": 0.78,
  "f1_score": 0.80,
  "predictions_made": 0,
  "successful_predictions": 0,
  "average_confidence": 0.75,
  "model_uptime": "99.5%",
  "last_updated": "2025-10-10T10:30:00.000000+00:00"
}
```

**Note:** Current implementation returns default values. Future versions will track actual performance.

**Code Location:** `python-ai-service/main.py:1980-1983`
**Related FR:** FR-AI-PERF-001
**Rate Limit:** 60 requests per minute

---

## Strategy & Recommendations

### POST /ai/strategy-recommendations

**Description:** Get AI-powered strategy recommendations for current market conditions

**Authentication:** None

**Request Body:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe_data": {
    "1h": [...],
    "4h": [...]
  },
  "current_price": 67500.00,
  "timestamp": 1697234567000,
  "available_strategies": [
    "RSI Strategy",
    "MACD Strategy",
    "Bollinger Bands Strategy",
    "Volume Strategy"
  ]
}
```

**Success Response (200 OK):**
```json
[
  {
    "strategy_name": "RSI Strategy",
    "suitability_score": 0.85,
    "reasoning": "RSI Strategy shows good potential for BTCUSDT based on current market conditions. Strong momentum indicators suggest favorable conditions.",
    "recommended_config": {
      "enabled": true,
      "weight": 0.85
    }
  },
  {
    "strategy_name": "MACD Strategy",
    "suitability_score": 0.78,
    "reasoning": "MACD Strategy shows good potential for BTCUSDT based on current market conditions. Trend following suitable for current phase.",
    "recommended_config": {
      "enabled": true,
      "weight": 0.78
    }
  },
  {
    "strategy_name": "Bollinger Bands Strategy",
    "suitability_score": 0.72,
    "reasoning": "Bollinger Bands Strategy shows good potential for BTCUSDT based on current market conditions. Mean reversion opportunities present.",
    "recommended_config": {
      "enabled": true,
      "weight": 0.72
    }
  }
]
```

**Code Location:** `python-ai-service/main.py:1891-1911`
**Related FR:** FR-AI-STRATEGY-001
**Rate Limit:** 5 requests per minute

---

### POST /ai/market-condition

**Description:** Analyze current market conditions and phase

**Authentication:** None

**Request Body:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe_data": {
    "1h": [...]
  },
  "current_price": 67500.00,
  "volume_24h": 50000000000.0,
  "timestamp": 1697234567000
}
```

**Success Response (200 OK):**
```json
{
  "condition_type": "Trending Up",
  "confidence": 0.75,
  "characteristics": [
    "Strong uptrend",
    "High momentum"
  ],
  "recommended_strategies": [
    "RSI Strategy",
    "MACD Strategy"
  ],
  "market_phase": "Active Trading"
}
```

**Market Condition Types:**
- `Trending Up` - Strong upward momentum (price change > 5%)
- `Trending Down` - Strong downward momentum (price change < -5%)
- `Sideways` - Consolidation, no clear trend

**Market Phases:**
- `Active Trading` - High liquidity and volume
- `Consolidation` - Low volatility range
- `Breakout` - Price breaking key levels

**Code Location:** `python-ai-service/main.py:1914-1943`
**Related FR:** FR-AI-MARKET-001
**Rate Limit:** 5 requests per minute

---

### POST /ai/feedback

**Description:** Send performance feedback for model learning

**Authentication:** None

**Request Body:**
```json
{
  "signal_id": "signal_123456",
  "symbol": "BTCUSDT",
  "predicted_signal": "Long",
  "actual_outcome": "PROFIT",
  "profit_loss": 50.00,
  "confidence_was_accurate": true,
  "feedback_notes": "Excellent signal timing and accuracy",
  "timestamp": 1697234567000
}
```

**Success Response (200 OK):**
```json
{
  "message": "Feedback received successfully",
  "signal_id": "signal_123456",
  "timestamp": "2025-10-10T10:30:00.000000+00:00"
}
```

**Feedback Types:**
- `actual_outcome`: PROFIT, LOSS, NEUTRAL
- `confidence_was_accurate`: true/false

**Future Enhancement:** Feedback will be stored for model retraining and performance tracking.

**Code Location:** `python-ai-service/main.py:1946-1960`
**Related FR:** FR-AI-FEEDBACK-001
**Rate Limit:** 20 requests per minute

---

## Storage & Analytics

### GET /ai/storage/stats

**Description:** Get AI analysis storage statistics from MongoDB

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "total_analyses": 1250,
  "symbols_analyzed": 8,
  "symbol_breakdown": [
    {
      "symbol": "BTCUSDT",
      "analysis_count": 450,
      "latest_analysis": "2025-10-10T10:30:00.000000+00:00"
    },
    {
      "symbol": "ETHUSDT",
      "analysis_count": 380,
      "latest_analysis": "2025-10-10T10:28:00.000000+00:00"
    },
    {
      "symbol": "BNBUSDT",
      "analysis_count": 220,
      "latest_analysis": "2025-10-10T10:25:00.000000+00:00"
    }
  ],
  "analysis_interval_minutes": 5,
  "collection_name": "ai_analysis_results"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "error": "MongoDB not connected"
}
```

**Code Location:** `python-ai-service/main.py:1986-2030`
**Related FR:** FR-AI-STORAGE-001
**Rate Limit:** 10 requests per minute

---

### POST /ai/storage/clear

**Description:** Clear all AI analysis results from MongoDB storage

**Authentication:** None (Should be admin-only in production)

**Success Response (200 OK):**
```json
{
  "message": "Storage cleared successfully",
  "cleared_analyses": 1250,
  "timestamp": "2025-10-10T10:30:00.000000+00:00"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "error": "MongoDB not connected"
}
```

**Warning:** This operation is irreversible and clears all cached analysis results.

**Code Location:** `python-ai-service/main.py:2033-2048`
**Related FR:** FR-AI-STORAGE-002
**Rate Limit:** 1 request per minute

---

## Async Task Management

### POST /api/tasks/train

**Description:** Trigger async model training job

**Authentication:** Optional (Bearer JWT)

**Request Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

**Request Body:**
```json
{
  "model_type": "lstm",
  "symbol": "BTCUSDT",
  "timeframe": "15m",
  "epochs": 100,
  "batch_size": 64,
  "learning_rate": 0.001
}
```

**Request Parameters:**
- `model_type` (required): Model architecture - `lstm`, `gru`, `transformer`, `ensemble`
- `symbol` (required): Trading pair - `BTCUSDT`, `ETHUSDT`, etc.
- `timeframe` (required): Candle timeframe - `1m`, `5m`, `15m`, `1h`, `4h`, `1d`
- `epochs` (optional): Training epochs (default: 100)
- `batch_size` (optional): Batch size (default: 64)
- `learning_rate` (optional): Learning rate (default: 0.001)

**Success Response (202 Accepted):**
```json
{
  "task_id": "abc-123-def-456",
  "status": "PENDING",
  "estimated_completion": "2025-11-22T15:30:00Z",
  "estimated_duration_seconds": 3600,
  "check_status_url": "/api/tasks/abc-123-def-456"
}
```

**Error Response (400 Bad Request):**
```json
{
  "detail": "Invalid model_type. Supported: lstm, gru, transformer, ensemble"
}
```

**Error Response (429 Too Many Requests):**
```json
{
  "detail": "Training task limit reached. Maximum 10 tasks per day."
}
```

**Code Location:** `python-ai-service/api/tasks.py:train_model_endpoint`
**Related FR:** FR-ASYNC-TRAIN-001
**Rate Limit:** 10 tasks per day per user

---

### GET /api/tasks/{task_id}

**Description:** Check task status and retrieve results

**Authentication:** Optional (Bearer JWT)

**Path Parameters:**
- `task_id` (required): Task identifier returned from task creation

**Success Response (200 OK) - PENDING:**
```json
{
  "task_id": "abc-123-def-456",
  "task_name": "ml_tasks.train_model_async",
  "status": "PENDING",
  "progress": 0,
  "created_at": "2025-11-22T12:00:00Z",
  "started_at": null,
  "estimated_completion": "2025-11-22T15:30:00Z",
  "queue_position": 3
}
```

**Success Response (200 OK) - STARTED:**
```json
{
  "task_id": "abc-123-def-456",
  "task_name": "ml_tasks.train_model_async",
  "status": "STARTED",
  "progress": 45,
  "created_at": "2025-11-22T12:00:00Z",
  "started_at": "2025-11-22T12:05:00Z",
  "estimated_completion": "2025-11-22T13:00:00Z",
  "current_operation": "Training epoch 45/100",
  "metrics": {
    "current_epoch": 45,
    "current_loss": 0.0345,
    "current_accuracy": 0.68
  }
}
```

**Success Response (200 OK) - SUCCESS:**
```json
{
  "task_id": "abc-123-def-456",
  "task_name": "ml_tasks.train_model_async",
  "status": "SUCCESS",
  "progress": 100,
  "result": {
    "model_id": "lstm_BTCUSDT_15m_20251122",
    "accuracy": 0.72,
    "precision": 0.70,
    "recall": 0.68,
    "f1_score": 0.69,
    "final_loss": 0.0234,
    "training_time_seconds": 3600,
    "epochs_completed": 100,
    "best_epoch": 87,
    "model_path": "/models/lstm_BTCUSDT_15m_20251122.h5"
  },
  "created_at": "2025-11-22T12:00:00Z",
  "started_at": "2025-11-22T12:05:00Z",
  "completed_at": "2025-11-22T13:05:00Z",
  "execution_time_seconds": 3600
}
```

**Success Response (200 OK) - FAILURE:**
```json
{
  "task_id": "abc-123-def-456",
  "task_name": "ml_tasks.train_model_async",
  "status": "FAILURE",
  "progress": 47,
  "error": "Out of memory during training at epoch 47",
  "error_type": "MemoryError",
  "traceback": "Traceback (most recent call last):\n  File \"/app/ml_tasks.py\", line 234...",
  "retries": 3,
  "max_retries": 3,
  "created_at": "2025-11-22T12:00:00Z",
  "started_at": "2025-11-22T12:05:00Z",
  "failed_at": "2025-11-22T12:30:00Z",
  "execution_time_seconds": 1500
}
```

**Success Response (200 OK) - REVOKED:**
```json
{
  "task_id": "abc-123-def-456",
  "task_name": "ml_tasks.train_model_async",
  "status": "REVOKED",
  "progress": 25,
  "created_at": "2025-11-22T12:00:00Z",
  "started_at": "2025-11-22T12:05:00Z",
  "revoked_at": "2025-11-22T12:15:00Z",
  "message": "Task cancelled by user"
}
```

**Error Response (404 Not Found):**
```json
{
  "detail": "Task not found: abc-123-def-456"
}
```

**Task Status Values:**
- `PENDING` - Task queued, waiting to start
- `STARTED` - Task currently executing
- `SUCCESS` - Task completed successfully
- `FAILURE` - Task failed with error
- `RETRY` - Task failed, retrying
- `REVOKED` - Task cancelled by user

**Code Location:** `python-ai-service/api/tasks.py:get_task_status`
**Related FR:** FR-ASYNC-STATUS-001
**Rate Limit:** 60 requests per minute

---

### DELETE /api/tasks/{task_id}

**Description:** Cancel a running or pending task

**Authentication:** Optional (Bearer JWT)

**Path Parameters:**
- `task_id` (required): Task identifier to cancel

**Success Response (200 OK):**
```json
{
  "success": true,
  "task_id": "abc-123-def-456",
  "status": "REVOKED",
  "message": "Task cancelled successfully",
  "cancelled_at": "2025-11-22T14:00:00Z"
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "task_id": "abc-123-def-456",
  "status": "SUCCESS",
  "message": "Cannot cancel completed task"
}
```

**Error Response (404 Not Found):**
```json
{
  "detail": "Task not found: abc-123-def-456"
}
```

**Note:** Only tasks with status PENDING or STARTED can be cancelled.

**Code Location:** `python-ai-service/api/tasks.py:cancel_task`
**Related FR:** FR-ASYNC-CANCEL-001
**Rate Limit:** 20 requests per minute

---

### GET /api/tasks

**Description:** List all tasks with optional filtering and pagination

**Authentication:** Optional (Bearer JWT)

**Query Parameters:**
- `status` (optional): Filter by status - `PENDING`, `STARTED`, `SUCCESS`, `FAILURE`, `REVOKED`
- `task_name` (optional): Filter by task name - `ml_tasks.train_model_async`, `backtest_tasks.run_backtest_async`
- `limit` (optional): Maximum results per page (default: 50, max: 200)
- `offset` (optional): Pagination offset (default: 0)
- `sort_by` (optional): Sort field - `created_at`, `started_at`, `completed_at` (default: `created_at`)
- `sort_order` (optional): Sort order - `asc`, `desc` (default: `desc`)

**Success Response (200 OK):**
```json
{
  "total": 150,
  "limit": 50,
  "offset": 0,
  "tasks": [
    {
      "task_id": "abc-123",
      "task_name": "ml_tasks.train_model_async",
      "status": "SUCCESS",
      "progress": 100,
      "created_at": "2025-11-22T10:00:00Z",
      "started_at": "2025-11-22T10:05:00Z",
      "completed_at": "2025-11-22T11:05:00Z",
      "execution_time_seconds": 3600
    },
    {
      "task_id": "def-456",
      "task_name": "backtest_tasks.run_backtest_async",
      "status": "STARTED",
      "progress": 60,
      "created_at": "2025-11-22T11:00:00Z",
      "started_at": "2025-11-22T11:02:00Z",
      "estimated_completion": "2025-11-22T11:20:00Z"
    },
    {
      "task_id": "ghi-789",
      "task_name": "ml_tasks.train_model_async",
      "status": "FAILURE",
      "progress": 23,
      "error": "Insufficient training data",
      "created_at": "2025-11-22T09:00:00Z",
      "failed_at": "2025-11-22T09:15:00Z"
    }
  ]
}
```

**Code Location:** `python-ai-service/api/tasks.py:list_tasks`
**Related FR:** FR-ASYNC-LIST-001
**Rate Limit:** 60 requests per minute

---

### POST /api/tasks/retry/{task_id}

**Description:** Retry a failed task with a new task ID

**Authentication:** Optional (Bearer JWT)

**Path Parameters:**
- `task_id` (required): Failed task identifier to retry

**Success Response (202 Accepted):**
```json
{
  "success": true,
  "new_task_id": "xyz-789-abc-012",
  "original_task_id": "abc-123-def-456",
  "status": "PENDING",
  "message": "Task retried with new ID",
  "check_status_url": "/api/tasks/xyz-789-abc-012",
  "created_at": "2025-11-22T14:30:00Z"
}
```

**Error Response (400 Bad Request):**
```json
{
  "detail": "Cannot retry task with status SUCCESS. Only FAILURE tasks can be retried."
}
```

**Error Response (404 Not Found):**
```json
{
  "detail": "Task not found: abc-123-def-456"
}
```

**Note:** Only tasks with status FAILURE can be retried. The retry creates a new task with the same parameters.

**Code Location:** `python-ai-service/api/tasks.py:retry_task`
**Related FR:** FR-ASYNC-RETRY-001
**Rate Limit:** 10 requests per minute

---

### GET /api/tasks/stats

**Description:** Get task execution statistics and system metrics

**Authentication:** Optional (Bearer JWT)

**Success Response (200 OK):**
```json
{
  "total_tasks": 1234,
  "pending": 5,
  "running": 2,
  "successful": 1150,
  "failed": 77,
  "revoked": 0,
  "success_rate": 0.937,
  "failure_rate": 0.063,
  "avg_execution_time_seconds": 1800,
  "median_execution_time_seconds": 1650,
  "min_execution_time_seconds": 120,
  "max_execution_time_seconds": 7200,
  "last_24h": {
    "total": 48,
    "successful": 46,
    "failed": 2,
    "success_rate": 0.958
  },
  "last_7d": {
    "total": 312,
    "successful": 295,
    "failed": 17,
    "success_rate": 0.945
  },
  "by_task_type": {
    "ml_tasks.train_model_async": {
      "total": 856,
      "successful": 812,
      "failed": 44,
      "success_rate": 0.949
    },
    "backtest_tasks.run_backtest_async": {
      "total": 378,
      "successful": 338,
      "failed": 33,
      "success_rate": 0.895
    }
  },
  "system_health": {
    "celery_workers_active": 4,
    "queue_depth": 5,
    "avg_queue_wait_time_seconds": 15
  },
  "last_updated": "2025-11-22T14:30:00Z"
}
```

**Code Location:** `python-ai-service/api/tasks.py:get_task_stats`
**Related FR:** FR-ASYNC-STATS-001
**Rate Limit:** 60 requests per minute

---

## Training Management

### GET /api/training/jobs

**Description:** List all training jobs with their results and status

**Authentication:** Optional (Bearer JWT)

**Query Parameters:**
- `status` (optional): Filter by status - `PENDING`, `TRAINING`, `COMPLETED`, `FAILED`, `DEPLOYED`
- `model_type` (optional): Filter by model type - `lstm`, `gru`, `transformer`, `ensemble`
- `symbol` (optional): Filter by trading pair - `BTCUSDT`, `ETHUSDT`
- `limit` (optional): Maximum results (default: 50)
- `offset` (optional): Pagination offset (default: 0)

**Success Response (200 OK):**
```json
{
  "total": 25,
  "limit": 50,
  "offset": 0,
  "jobs": [
    {
      "job_id": "training_lstm_BTCUSDT_20251122",
      "model_type": "lstm",
      "symbol": "BTCUSDT",
      "timeframe": "15m",
      "status": "COMPLETED",
      "accuracy": 0.72,
      "precision": 0.70,
      "recall": 0.68,
      "f1_score": 0.69,
      "deployed": true,
      "task_id": "abc-123-def-456",
      "created_at": "2025-11-22T10:00:00Z",
      "started_at": "2025-11-22T10:05:00Z",
      "completed_at": "2025-11-22T11:05:00Z",
      "training_time_seconds": 3600
    },
    {
      "job_id": "training_gru_ETHUSDT_20251122",
      "model_type": "gru",
      "symbol": "ETHUSDT",
      "timeframe": "1h",
      "status": "TRAINING",
      "progress": 65,
      "deployed": false,
      "task_id": "def-456-ghi-789",
      "created_at": "2025-11-22T12:00:00Z",
      "started_at": "2025-11-22T12:05:00Z",
      "estimated_completion": "2025-11-22T13:30:00Z"
    },
    {
      "job_id": "training_transformer_BTCUSDT_20251121",
      "model_type": "transformer",
      "symbol": "BTCUSDT",
      "timeframe": "4h",
      "status": "FAILED",
      "error": "Out of memory during training",
      "deployed": false,
      "task_id": "ghi-789-jkl-012",
      "created_at": "2025-11-21T14:00:00Z",
      "failed_at": "2025-11-21T14:30:00Z"
    }
  ]
}
```

**Code Location:** `python-ai-service/api/training.py:list_training_jobs`
**Related FR:** FR-TRAIN-LIST-001
**Rate Limit:** 60 requests per minute

---

### GET /api/training/jobs/{job_id}

**Description:** Get detailed information about a specific training job

**Authentication:** Optional (Bearer JWT)

**Path Parameters:**
- `job_id` (required): Training job identifier

**Success Response (200 OK):**
```json
{
  "job_id": "training_lstm_BTCUSDT_20251122",
  "model_type": "lstm",
  "symbol": "BTCUSDT",
  "timeframe": "15m",
  "status": "COMPLETED",
  "task_id": "abc-123-def-456",
  "parameters": {
    "epochs": 100,
    "batch_size": 64,
    "learning_rate": 0.001,
    "hidden_units": 128,
    "dropout_rate": 0.2,
    "sequence_length": 60
  },
  "metrics": {
    "accuracy": 0.72,
    "precision": 0.70,
    "recall": 0.68,
    "f1_score": 0.69,
    "final_loss": 0.0234,
    "best_epoch": 87,
    "best_validation_accuracy": 0.73,
    "training_samples": 45000,
    "validation_samples": 5000,
    "test_samples": 10000
  },
  "performance": {
    "sharpe_ratio": 1.85,
    "max_drawdown": 0.12,
    "win_rate": 0.68,
    "profit_factor": 2.1
  },
  "model_info": {
    "model_id": "lstm_BTCUSDT_15m_20251122",
    "model_path": "/models/lstm_BTCUSDT_15m_20251122.h5",
    "model_size_mb": 15.4,
    "parameters_count": 2450000
  },
  "deployment": {
    "deployed": true,
    "deployed_at": "2025-11-22T14:00:00Z",
    "deployed_by": "admin@example.com",
    "previous_model": "lstm_BTCUSDT_15m_20251115",
    "is_active": true
  },
  "created_at": "2025-11-22T10:00:00Z",
  "started_at": "2025-11-22T10:05:00Z",
  "completed_at": "2025-11-22T11:05:00Z",
  "training_time_seconds": 3600
}
```

**Error Response (404 Not Found):**
```json
{
  "detail": "Training job not found: training_lstm_BTCUSDT_20251122"
}
```

**Code Location:** `python-ai-service/api/training.py:get_training_job`
**Related FR:** FR-TRAIN-GET-001
**Rate Limit:** 60 requests per minute

---

### POST /api/training/jobs/{job_id}/deploy

**Description:** Deploy a completed training job to production

**Authentication:** Required (Bearer JWT with admin role)

**Path Parameters:**
- `job_id` (required): Training job identifier to deploy

**Request Body (Optional):**
```json
{
  "force_deploy": false,
  "activate_immediately": true,
  "notes": "Deploying improved LSTM model with 72% accuracy"
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "job_id": "training_lstm_BTCUSDT_20251122",
  "model_id": "lstm_BTCUSDT_15m_20251122",
  "deployed_at": "2025-11-22T14:00:00Z",
  "previous_model": "lstm_BTCUSDT_15m_20251115",
  "previous_accuracy": 0.68,
  "new_accuracy": 0.72,
  "accuracy_improvement": 0.04,
  "is_active": true,
  "message": "Model deployed to production successfully"
}
```

**Error Response (400 Bad Request) - Not Completed:**
```json
{
  "success": false,
  "detail": "Cannot deploy incomplete training job. Status: TRAINING"
}
```

**Error Response (400 Bad Request) - Low Accuracy:**
```json
{
  "success": false,
  "detail": "Model accuracy (0.55) below minimum threshold (0.60). Use force_deploy=true to override."
}
```

**Error Response (403 Forbidden):**
```json
{
  "detail": "Insufficient permissions. Admin role required for model deployment."
}
```

**Error Response (404 Not Found):**
```json
{
  "detail": "Training job not found: training_lstm_BTCUSDT_20251122"
}
```

**Deployment Rules:**
- Minimum accuracy threshold: 0.60 (can be overridden with `force_deploy`)
- Only COMPLETED jobs can be deployed
- Previous model is automatically archived
- Deployment triggers WebSocket notification

**Code Location:** `python-ai-service/api/training.py:deploy_training_job`
**Related FR:** FR-TRAIN-DEPLOY-001
**Rate Limit:** 5 requests per minute

---

## Backtest Management

### POST /api/backtests

**Description:** Trigger async strategy backtest execution

**Authentication:** Optional (Bearer JWT)

**Request Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json
```

**Request Body:**
```json
{
  "strategy_name": "stochastic",
  "symbol": "BTCUSDT",
  "timeframe": "15m",
  "start_date": "2025-05-22",
  "end_date": "2025-11-22",
  "parameters": {
    "k_period": 14,
    "d_period": 3,
    "overbought": 80,
    "oversold": 20
  },
  "initial_capital": 10000.0,
  "position_size": 0.1,
  "commission": 0.001
}
```

**Request Parameters:**
- `strategy_name` (required): Strategy identifier - `stochastic`, `rsi`, `macd`, `bollinger`, `volume`
- `symbol` (required): Trading pair
- `timeframe` (required): Candle timeframe
- `start_date` (required): Backtest start date (YYYY-MM-DD)
- `end_date` (required): Backtest end date (YYYY-MM-DD)
- `parameters` (required): Strategy-specific parameters
- `initial_capital` (optional): Starting capital (default: 10000.0)
- `position_size` (optional): Position size as fraction (default: 0.1)
- `commission` (optional): Trading commission rate (default: 0.001)

**Success Response (202 Accepted):**
```json
{
  "backtest_id": "backtest_stochastic_BTCUSDT_20251122",
  "task_id": "xyz-789-abc-012",
  "status": "PENDING",
  "estimated_completion_minutes": 15,
  "check_status_url": "/api/tasks/xyz-789-abc-012",
  "check_results_url": "/api/backtests/backtest_stochastic_BTCUSDT_20251122",
  "created_at": "2025-11-22T14:00:00Z"
}
```

**Error Response (400 Bad Request):**
```json
{
  "detail": "Invalid date range. start_date must be before end_date."
}
```

**Error Response (429 Too Many Requests):**
```json
{
  "detail": "Backtest limit reached. Maximum 20 backtests per day."
}
```

**Code Location:** `python-ai-service/api/backtests.py:create_backtest`
**Related FR:** FR-BACKTEST-CREATE-001
**Rate Limit:** 20 backtests per day per user

---

### GET /api/backtests/{backtest_id}

**Description:** Get detailed backtest results and performance metrics

**Authentication:** Optional (Bearer JWT)

**Path Parameters:**
- `backtest_id` (required): Backtest identifier

**Success Response (200 OK):**
```json
{
  "backtest_id": "backtest_stochastic_BTCUSDT_20251122",
  "strategy_name": "stochastic",
  "symbol": "BTCUSDT",
  "timeframe": "15m",
  "status": "COMPLETED",
  "task_id": "xyz-789-abc-012",
  "parameters": {
    "k_period": 14,
    "d_period": 3,
    "overbought": 80,
    "oversold": 20,
    "initial_capital": 10000.0,
    "position_size": 0.1,
    "commission": 0.001
  },
  "date_range": {
    "start_date": "2025-05-22",
    "end_date": "2025-11-22",
    "total_days": 184
  },
  "performance": {
    "total_trades": 156,
    "winning_trades": 88,
    "losing_trades": 68,
    "win_rate": 0.564,
    "total_profit": 1840.50,
    "total_loss": -842.30,
    "net_profit": 998.20,
    "net_profit_percent": 9.98,
    "gross_profit": 1840.50,
    "gross_loss": -842.30,
    "profit_factor": 2.18,
    "average_win": 20.91,
    "average_loss": -12.39,
    "largest_win": 145.60,
    "largest_loss": -78.20,
    "avg_trade": 6.40,
    "max_consecutive_wins": 8,
    "max_consecutive_losses": 5
  },
  "risk_metrics": {
    "sharpe_ratio": 1.52,
    "sortino_ratio": 2.14,
    "max_drawdown": 0.084,
    "max_drawdown_duration_days": 12,
    "calmar_ratio": 1.89,
    "recovery_factor": 11.88,
    "risk_reward_ratio": 1.69
  },
  "execution": {
    "total_candles_analyzed": 17760,
    "signals_generated": 312,
    "trades_executed": 156,
    "signal_to_trade_ratio": 0.5,
    "execution_time_seconds": 245
  },
  "equity_curve": [
    {
      "date": "2025-05-22",
      "equity": 10000.00,
      "drawdown": 0.0
    },
    {
      "date": "2025-05-23",
      "equity": 10085.40,
      "drawdown": 0.0
    }
  ],
  "trades": [
    {
      "trade_id": 1,
      "entry_date": "2025-05-22T10:30:00Z",
      "exit_date": "2025-05-22T14:15:00Z",
      "direction": "LONG",
      "entry_price": 67500.00,
      "exit_price": 67850.50,
      "quantity": 0.0148,
      "profit": 5.19,
      "profit_percent": 0.52,
      "commission": 1.00,
      "net_profit": 4.19
    }
  ],
  "created_at": "2025-11-22T14:00:00Z",
  "started_at": "2025-11-22T14:02:00Z",
  "completed_at": "2025-11-22T14:06:05Z"
}
```

**Success Response (200 OK) - PENDING/RUNNING:**
```json
{
  "backtest_id": "backtest_stochastic_BTCUSDT_20251122",
  "strategy_name": "stochastic",
  "symbol": "BTCUSDT",
  "status": "RUNNING",
  "task_id": "xyz-789-abc-012",
  "progress": 68,
  "current_operation": "Analyzing candles 12000/17760",
  "created_at": "2025-11-22T14:00:00Z",
  "started_at": "2025-11-22T14:02:00Z",
  "estimated_completion": "2025-11-22T14:15:00Z"
}
```

**Error Response (404 Not Found):**
```json
{
  "detail": "Backtest not found: backtest_stochastic_BTCUSDT_20251122"
}
```

**Code Location:** `python-ai-service/api/backtests.py:get_backtest`
**Related FR:** FR-BACKTEST-GET-001
**Rate Limit:** 60 requests per minute

---

### GET /api/backtests

**Description:** List all backtests with summary information

**Authentication:** Optional (Bearer JWT)

**Query Parameters:**
- `strategy_name` (optional): Filter by strategy
- `symbol` (optional): Filter by trading pair
- `status` (optional): Filter by status - `PENDING`, `RUNNING`, `COMPLETED`, `FAILED`
- `limit` (optional): Maximum results (default: 50)
- `offset` (optional): Pagination offset (default: 0)
- `sort_by` (optional): Sort field - `created_at`, `win_rate`, `sharpe_ratio`, `net_profit` (default: `created_at`)
- `sort_order` (optional): Sort order - `asc`, `desc` (default: `desc`)

**Success Response (200 OK):**
```json
{
  "total": 85,
  "limit": 50,
  "offset": 0,
  "backtests": [
    {
      "backtest_id": "backtest_stochastic_BTCUSDT_20251122",
      "strategy_name": "stochastic",
      "symbol": "BTCUSDT",
      "timeframe": "15m",
      "status": "COMPLETED",
      "date_range": {
        "start_date": "2025-05-22",
        "end_date": "2025-11-22",
        "total_days": 184
      },
      "performance_summary": {
        "total_trades": 156,
        "win_rate": 0.564,
        "net_profit": 998.20,
        "net_profit_percent": 9.98,
        "sharpe_ratio": 1.52,
        "max_drawdown": 0.084
      },
      "created_at": "2025-11-22T14:00:00Z",
      "completed_at": "2025-11-22T14:06:05Z"
    },
    {
      "backtest_id": "backtest_rsi_ETHUSDT_20251121",
      "strategy_name": "rsi",
      "symbol": "ETHUSDT",
      "timeframe": "1h",
      "status": "COMPLETED",
      "date_range": {
        "start_date": "2025-04-01",
        "end_date": "2025-11-21",
        "total_days": 234
      },
      "performance_summary": {
        "total_trades": 89,
        "win_rate": 0.618,
        "net_profit": 1245.80,
        "net_profit_percent": 12.46,
        "sharpe_ratio": 1.87,
        "max_drawdown": 0.056
      },
      "created_at": "2025-11-21T10:00:00Z",
      "completed_at": "2025-11-21T10:12:30Z"
    },
    {
      "backtest_id": "backtest_macd_BTCUSDT_20251120",
      "strategy_name": "macd",
      "symbol": "BTCUSDT",
      "timeframe": "4h",
      "status": "FAILED",
      "error": "Insufficient historical data for date range",
      "created_at": "2025-11-20T15:00:00Z",
      "failed_at": "2025-11-20T15:02:45Z"
    }
  ]
}
```

**Code Location:** `python-ai-service/api/backtests.py:list_backtests`
**Related FR:** FR-BACKTEST-LIST-001
**Rate Limit:** 60 requests per minute

---

## Monitoring & Alerts

### GET /api/monitoring/health

**Description:** Get comprehensive system health status including all services

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "status": "healthy",
  "timestamp": "2025-11-22T14:30:00Z",
  "services": {
    "celery_workers": {
      "status": "healthy",
      "active_workers": 4,
      "queued_tasks": 5,
      "processed_tasks_total": 12456,
      "failed_tasks_total": 234,
      "workers": [
        {
          "worker_id": "celery@worker1",
          "status": "online",
          "current_task": "ml_tasks.train_model_async",
          "tasks_processed": 3245
        },
        {
          "worker_id": "celery@worker2",
          "status": "online",
          "current_task": null,
          "tasks_processed": 3102
        }
      ]
    },
    "rabbitmq": {
      "status": "healthy",
      "queue_depth": 5,
      "messages_ready": 5,
      "messages_unacknowledged": 2,
      "consumers": 4,
      "connection_status": "connected"
    },
    "mongodb": {
      "status": "healthy",
      "connections": 12,
      "connections_available": 88,
      "operations_per_second": 145,
      "replication_lag_ms": 5,
      "database_size_mb": 2450
    },
    "redis": {
      "status": "healthy",
      "memory_usage_mb": 256,
      "memory_max_mb": 2048,
      "memory_utilization": 0.125,
      "connected_clients": 8,
      "keys_count": 1245,
      "hit_rate": 0.94
    },
    "openai": {
      "status": "healthy",
      "api_key_configured": true,
      "rate_limit_status": "ok",
      "requests_last_hour": 45,
      "requests_remaining": 955
    }
  },
  "system_resources": {
    "cpu_usage_percent": 45.2,
    "memory_usage_percent": 62.8,
    "disk_usage_percent": 38.5,
    "network_io_mbps": 12.4
  },
  "performance_metrics": {
    "avg_task_execution_time_seconds": 1800,
    "avg_queue_wait_time_seconds": 15,
    "tasks_per_hour": 24,
    "error_rate": 0.019
  },
  "last_check": "2025-11-22T14:30:00Z"
}
```

**Degraded Response (200 OK):**
```json
{
  "status": "degraded",
  "timestamp": "2025-11-22T14:30:00Z",
  "services": {
    "celery_workers": {
      "status": "degraded",
      "active_workers": 2,
      "queued_tasks": 45,
      "warning": "Only 2 of 4 workers active, queue depth high"
    },
    "rabbitmq": {
      "status": "healthy",
      "queue_depth": 45
    }
  },
  "warnings": [
    "High queue depth (45 tasks)",
    "Reduced worker capacity (2/4 workers)"
  ],
  "last_check": "2025-11-22T14:30:00Z"
}
```

**Unhealthy Response (503 Service Unavailable):**
```json
{
  "status": "unhealthy",
  "timestamp": "2025-11-22T14:30:00Z",
  "services": {
    "celery_workers": {
      "status": "unhealthy",
      "active_workers": 0,
      "error": "No workers available"
    },
    "rabbitmq": {
      "status": "unhealthy",
      "error": "Connection refused"
    }
  },
  "errors": [
    "No Celery workers available",
    "RabbitMQ connection failed"
  ],
  "last_check": "2025-11-22T14:30:00Z"
}
```

**Health Status Values:**
- `healthy` - All systems operational
- `degraded` - Some issues but service operational
- `unhealthy` - Critical issues affecting service

**Code Location:** `python-ai-service/api/monitoring.py:get_health`
**Related FR:** FR-MONITOR-HEALTH-001
**Rate Limit:** 60 requests per minute

---

### GET /api/monitoring/alerts

**Description:** Get recent system alerts and warnings

**Authentication:** Optional (Bearer JWT)

**Query Parameters:**
- `severity` (optional): Filter by severity - `LOW`, `MEDIUM`, `HIGH`, `CRITICAL`
- `status` (optional): Filter by status - `OPEN`, `ACKNOWLEDGED`, `RESOLVED`
- `limit` (optional): Maximum results (default: 50)
- `offset` (optional): Pagination offset (default: 0)

**Success Response (200 OK):**
```json
{
  "total": 12,
  "limit": 50,
  "offset": 0,
  "alerts": [
    {
      "alert_id": "alert_high_cpu_20251122_143000",
      "severity": "HIGH",
      "title": "High CPU Usage",
      "description": "CPU usage exceeded 85% threshold (current: 92.4%)",
      "status": "OPEN",
      "service": "celery_workers",
      "metric": "cpu_usage_percent",
      "current_value": 92.4,
      "threshold": 85.0,
      "created_at": "2025-11-22T14:30:00Z",
      "acknowledged_at": null,
      "resolved_at": null
    },
    {
      "alert_id": "alert_queue_depth_20251122_142500",
      "severity": "MEDIUM",
      "title": "High Queue Depth",
      "description": "RabbitMQ queue depth exceeded 30 tasks (current: 45)",
      "status": "ACKNOWLEDGED",
      "service": "rabbitmq",
      "metric": "queue_depth",
      "current_value": 45,
      "threshold": 30,
      "created_at": "2025-11-22T14:25:00Z",
      "acknowledged_at": "2025-11-22T14:26:00Z",
      "acknowledged_by": "admin@example.com",
      "resolved_at": null
    },
    {
      "alert_id": "alert_memory_20251122_140000",
      "severity": "LOW",
      "title": "Elevated Memory Usage",
      "description": "Memory usage exceeded 70% threshold (current: 75.2%)",
      "status": "RESOLVED",
      "service": "redis",
      "metric": "memory_usage_percent",
      "current_value": 68.5,
      "threshold": 70.0,
      "created_at": "2025-11-22T14:00:00Z",
      "acknowledged_at": "2025-11-22T14:02:00Z",
      "resolved_at": "2025-11-22T14:15:00Z",
      "resolution_notes": "Memory usage returned to normal after cache cleanup"
    }
  ],
  "summary": {
    "critical": 0,
    "high": 1,
    "medium": 3,
    "low": 8,
    "open": 4,
    "acknowledged": 5,
    "resolved": 3
  }
}
```

**Alert Severity Levels:**
- `CRITICAL` - Immediate action required, service disruption
- `HIGH` - Urgent attention needed, potential service impact
- `MEDIUM` - Should be addressed soon, minor impact
- `LOW` - Informational, monitor situation

**Code Location:** `python-ai-service/api/monitoring.py:get_alerts`
**Related FR:** FR-MONITOR-ALERTS-001
**Rate Limit:** 60 requests per minute

---

## WebSocket Real-time Updates

### WS /ws

**Description:** WebSocket endpoint for real-time AI signal broadcasting

**Connection URL:** `ws://localhost:8000/ws`

**Connection Example:**
```javascript
const ws = new WebSocket('ws://localhost:8000/ws');

ws.onopen = () => {
  console.log('Connected to AI WebSocket');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('AI Signal:', message);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('Disconnected from AI WebSocket');
};
```

**Connection Message:**
```json
{
  "type": "connection",
  "message": "Connected to AI Trading Service",
  "timestamp": "2025-10-10T10:30:00.000000+00:00"
}
```

**AI Signal Broadcast:**
```json
{
  "type": "AISignalReceived",
  "data": {
    "symbol": "BTCUSDT",
    "signal": "long",
    "confidence": 0.82,
    "timestamp": 1697234567000,
    "model_type": "GPT-4",
    "timeframe": "1h",
    "reasoning": "Strong bullish momentum with RSI oversold (28), MACD crossover",
    "strategy_scores": {
      "RSI Strategy": 0.85,
      "MACD Strategy": 0.78,
      "Bollinger Bands Strategy": 0.65,
      "Volume Strategy": 0.72
    }
  },
  "timestamp": "2025-10-10T10:30:00.000000+00:00"
}
```

**Pong Response (Heartbeat):**
```json
{
  "type": "Pong",
  "message": "Connection alive",
  "timestamp": "2025-10-10T10:30:00.000000+00:00"
}
```

**Broadcasting Behavior:**
- Signals are broadcasted automatically after `/ai/analyze` requests
- Periodic analysis (every 5 minutes) broadcasts signals for all tracked symbols
- All connected clients receive broadcasts simultaneously

**Code Location:** `python-ai-service/main.py:1731-1747`
**Related FR:** FR-AI-WS-001

---

## Technical Indicators

The AI service calculates the following technical indicators:

### Trend Indicators
- **SMA (Simple Moving Average)**: 20, 50 periods
- **EMA (Exponential Moving Average)**: 9, 12, 21, 26, 50 periods

### Momentum Indicators
- **RSI (Relative Strength Index)**: 14 periods
- **Stochastic**: %K and %D lines
- **MACD**: MACD line, Signal line, Histogram

### Volatility Indicators
- **Bollinger Bands**: Upper, Middle (SMA 20), Lower bands
- **ATR (Average True Range)**: 14 periods

### Volume Indicators
- **Volume SMA**: 20 periods
- **Volume Ratio**: Current volume / SMA

### Trend Strength
- **ADX (Average Directional Index)**: 14 periods

**Code Location:** `python-ai-service/main.py:583-743`

---

## Error Codes

### HTTP Status Codes

| Status Code | Meaning | Usage |
|-------------|---------|-------|
| 200 | OK | Successful request |
| 202 | Accepted | Async task created successfully |
| 400 | Bad Request | Invalid request parameters |
| 403 | Forbidden | Insufficient permissions |
| 404 | Not Found | Resource not found |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | Service or dependency unavailable |

### Common Error Messages

| Error | Description | Solution |
|-------|-------------|----------|
| `AI analysis failed: <reason>` | Analysis request failed | Check request format and OpenAI status |
| `OpenAI rate limit exceeded` | Too many OpenAI API calls | Wait for rate limit reset or use backup keys |
| `MongoDB not connected` | Database unavailable | Check MongoDB connection |
| `GPT-4 client not initialized` | OpenAI client setup failed | Verify API key configuration |
| `All API keys exhausted or rate limited` | All backup keys rate limited | Wait 1 hour or add more keys |
| `Task not found: <task_id>` | Task ID does not exist | Verify task ID is correct |
| `Cannot cancel completed task` | Task already finished | Only PENDING/STARTED tasks can be cancelled |
| `Training task limit reached` | Daily training limit exceeded | Wait for limit reset or request increase |
| `Backtest limit reached` | Daily backtest limit exceeded | Wait for limit reset or request increase |
| `Invalid model_type` | Unsupported model architecture | Use: lstm, gru, transformer, ensemble |
| `Invalid date range` | Backtest date range invalid | Ensure start_date < end_date |
| `Cannot deploy incomplete training job` | Training not finished | Wait for training completion |
| `Model accuracy below minimum threshold` | Trained model accuracy too low | Use force_deploy or retrain with better parameters |
| `Insufficient permissions` | Admin role required | Contact administrator for permissions |
| `No workers available` | All Celery workers offline | Check Celery worker status |
| `RabbitMQ connection failed` | Message queue unavailable | Check RabbitMQ service status |

---

## Rate Limiting

### Endpoint Rate Limits

| Endpoint | Limit | Window |
|----------|-------|--------|
| `/ai/analyze` | 10 requests | 1 minute |
| `/debug/gpt4` | 5 requests | 1 minute |
| `/ai/strategy-recommendations` | 5 requests | 1 minute |
| `/ai/market-condition` | 5 requests | 1 minute |
| `/ai/feedback` | 20 requests | 1 minute |
| `/ai/storage/clear` | 1 request | 1 minute |
| `/api/tasks/train` | 10 tasks | 1 day |
| `/api/tasks/{task_id}` (GET) | 60 requests | 1 minute |
| `/api/tasks/{task_id}` (DELETE) | 20 requests | 1 minute |
| `/api/tasks` (GET) | 60 requests | 1 minute |
| `/api/tasks/retry/{task_id}` | 10 requests | 1 minute |
| `/api/tasks/stats` | 60 requests | 1 minute |
| `/api/training/jobs` (GET) | 60 requests | 1 minute |
| `/api/training/jobs/{job_id}` (GET) | 60 requests | 1 minute |
| `/api/training/jobs/{job_id}/deploy` | 5 requests | 1 minute |
| `/api/backtests` (POST) | 20 tasks | 1 day |
| `/api/backtests/{backtest_id}` (GET) | 60 requests | 1 minute |
| `/api/backtests` (GET) | 60 requests | 1 minute |
| `/api/monitoring/health` | 60 requests | 1 minute |
| `/api/monitoring/alerts` | 60 requests | 1 minute |
| Other endpoints | 60 requests | 1 minute |

### OpenAI Rate Limiting

**GPT-4o-mini Rate Limits:**
- Minimum 20 seconds between requests (configurable)
- Automatic rate limit detection and retry
- Multi-key fallback on 429 errors
- Rate limit reset tracking

**Configuration:**
```python
OPENAI_REQUEST_DELAY = 20  # seconds
OPENAI_RATE_LIMIT_RESET_TIME = None  # tracked dynamically
```

### Rate Limit Response

**Status:** 429 Too Many Requests

```json
{
  "detail": "Rate limit exceeded. Please try again later."
}
```

**Code Location:** `python-ai-service/main.py:34-36, 347-359`

---

## Background Tasks

### Periodic Analysis Runner

**Description:** Automatic AI analysis for all tracked symbols every 5 minutes

**Symbols Analyzed:**
- BTCUSDT
- ETHUSDT
- BNBUSDT
- SOLUSDT
- ADAUSDT
- DOTUSDT
- XRPUSDT
- LINKUSDT

**Analysis Interval:** 5 minutes (300 seconds)

**Behavior:**
1. Generates market data for each symbol
2. Runs GPT-4 analysis
3. Stores results in MongoDB
4. Broadcasts signals via WebSocket
5. Waits 10 seconds between symbols (rate limiting)
6. Repeats every 5 minutes

**Code Location:** `python-ai-service/main.py:187-245`

---

## API Key Management

### Multi-Key Configuration

**Environment Variables:**
```bash
OPENAI_API_KEY=sk-primary-key-here
OPENAI_BACKUP_API_KEYS=sk-backup-1,sk-backup-2,sk-backup-3
```

**Automatic Fallback:**
- Primary key used first
- On 429 error, switches to next available key
- Tracks rate-limited keys
- Resets rate limit status after timeout

**Code Location:** `python-ai-service/main.py:903-1066`

---

## MongoDB Schema

### AI Analysis Results Collection

**Collection Name:** `ai_analysis_results`

**Document Structure:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439011"),
  "symbol": "BTCUSDT",
  "timestamp": ISODate("2025-10-10T10:30:00Z"),
  "analysis": {
    "signal": "Long",
    "confidence": 0.82,
    "reasoning": "...",
    "strategy_scores": {...},
    "market_analysis": {...},
    "risk_assessment": {...},
    "timestamp": 1697234567000
  },
  "created_at": ISODate("2025-10-10T10:30:00Z")
}
```

**Indexes:**
```javascript
db.ai_analysis_results.createIndex({ "symbol": 1, "timestamp": 1 })
```

**Code Location:** `python-ai-service/main.py:147-184`

---

## Code References

| Component | File Path | Description |
|-----------|-----------|-------------|
| Main API | `python-ai-service/main.py` | FastAPI application and all endpoints |
| GPT Analyzer | `python-ai-service/main.py:1071-1535` | GPT-4 trading analysis engine |
| Technical Analyzer | `python-ai-service/main.py:556-898` | Indicator calculation |
| WebSocket Manager | `python-ai-service/main.py:70-130` | WebSocket connection handler |
| Direct OpenAI Client | `python-ai-service/main.py:903-1066` | HTTP-based OpenAI client |

---

## Additional Endpoints

### POST /predict-trend

**Description:** Legacy ML-based trend prediction (may be deprecated in favor of `/ai/analyze`)

**Authentication:** Optional (Bearer JWT)

**Request Body:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "candles": 100
}
```

**Success Response (200 OK):**
```json
{
  "symbol": "BTCUSDT",
  "trend": "BULLISH",
  "confidence": 0.75,
  "prediction_horizon": "24h"
}
```

**Code Location:** `python-ai-service/main.py:2862`
**Status:** Legacy endpoint, consider using `/ai/analyze` instead

---

### GET /ai/cost/statistics

**Description:** Get OpenAI API cost statistics and usage tracking

**Authentication:** None (Should be admin-only in production)

**Success Response (200 OK):**
```json
{
  "total_requests": 1250,
  "total_tokens": 450000,
  "estimated_cost_usd": 12.50,
  "requests_by_model": {
    "gpt-4o-mini": 1250
  },
  "requests_today": 48,
  "cost_today_usd": 0.48,
  "average_cost_per_request": 0.01
}
```

**Code Location:** `python-ai-service/main.py:3205`
**Related FR:** FR-AI-COST-001
**Rate Limit:** 60 requests per minute

---

### POST /ai/config-analysis/trigger

**Description:** Trigger AI-powered configuration optimization analysis

**Authentication:** None (Should be admin-only in production)

**Request Body:**
```json
{
  "analyze_strategies": true,
  "analyze_risk_settings": true,
  "days_historical": 30
}
```

**Success Response (200 OK):**
```json
{
  "analysis_id": "config_analysis_20260206_140000",
  "status": "COMPLETED",
  "suggestions_generated": 12,
  "timestamp": "2026-02-06T14:00:00Z"
}
```

**Code Location:** `python-ai-service/main.py:3341`
**Related FR:** FR-AI-CONFIG-001
**Rate Limit:** 5 requests per hour

---

### GET /ai/config-suggestions

**Description:** Get AI-generated configuration optimization suggestions

**Authentication:** None (Should be admin-only in production)

**Query Parameters:**
- `days` (optional): Historical data analysis period (default: 30)
- `limit` (optional): Maximum suggestions (default: 20)

**Success Response (200 OK):**
```json
{
  "suggestions": [
    {
      "category": "risk_management",
      "current_value": 0.05,
      "suggested_value": 0.03,
      "reasoning": "Daily loss limit too high based on historical volatility",
      "impact": "HIGH",
      "confidence": 0.85
    },
    {
      "category": "strategy_weight",
      "strategy": "RSI Strategy",
      "current_weight": 0.25,
      "suggested_weight": 0.35,
      "reasoning": "RSI shows 68% win rate, should increase allocation",
      "impact": "MEDIUM",
      "confidence": 0.78
    }
  ],
  "total_suggestions": 12,
  "analysis_date": "2026-02-06T14:00:00Z"
}
```

**Code Location:** `python-ai-service/main.py:3385`
**Related FR:** FR-AI-CONFIG-002
**Rate Limit:** 60 requests per minute

---

### GET /ai/gpt4-analysis-history

**Description:** View historical GPT-4 analysis results with pagination

**Authentication:** None

**Query Parameters:**
- `days` (optional): Historical period in days (default: 30)
- `limit` (optional): Maximum results (default: 20)

**Success Response (200 OK):**
```json
{
  "total": 450,
  "limit": 20,
  "analyses": [
    {
      "symbol": "BTCUSDT",
      "signal": "Long",
      "confidence": 0.82,
      "reasoning": "Strong bullish momentum...",
      "timestamp": "2026-02-06T14:00:00Z"
    },
    {
      "symbol": "ETHUSDT",
      "signal": "Neutral",
      "confidence": 0.65,
      "reasoning": "Mixed signals, consolidating...",
      "timestamp": "2026-02-06T13:55:00Z"
    }
  ]
}
```

**Code Location:** `python-ai-service/main.py:3426`
**Related FR:** FR-AI-HISTORY-001
**Rate Limit:** 60 requests per minute

---

### POST /api/chat/project

**Description:** Chat with AI about project codebase and documentation

**Authentication:** None (Should be admin-only in production)

**Request Body:**
```json
{
  "message": "How does the paper trading risk management work?",
  "conversation_id": "conv_123456"
}
```

**Success Response (200 OK):**
```json
{
  "response": "The paper trading system includes three main risk management features: 1) Daily loss limit (5% max)...",
  "conversation_id": "conv_123456",
  "sources": [
    "docs/features/paper-trading.md",
    "rust-core-engine/src/paper_trading/engine.rs"
  ],
  "timestamp": "2026-02-06T14:00:00Z"
}
```

**Code Location:** `python-ai-service/main.py:3473`
**Related FR:** FR-CHAT-001
**Rate Limit:** 20 requests per minute

---

### GET /api/chat/project/suggestions

**Description:** Get suggested questions for project chatbot

**Authentication:** None

**Success Response (200 OK):**
```json
{
  "suggestions": [
    "How does trailing stop loss work?",
    "What are the available trading strategies?",
    "How do I enable paper trading?",
    "What are the risk management features?"
  ]
}
```

**Code Location:** `python-ai-service/main.py:3527`
**Related FR:** FR-CHAT-002
**Rate Limit:** 60 requests per minute

---

### POST /api/chat/project/clear

**Description:** Clear project chat history

**Authentication:** None (Should be admin-only in production)

**Success Response (200 OK):**
```json
{
  "message": "Chat history cleared successfully",
  "conversations_deleted": 15,
  "timestamp": "2026-02-06T14:00:00Z"
}
```

**Code Location:** `python-ai-service/main.py:3541`
**Related FR:** FR-CHAT-003
**Rate Limit:** 5 requests per minute

---

## Error Format Standardization

### Current State

**Rust Core Engine:**
```json
{
  "success": false,
  "error": "Error message",
  "data": null
}
```

**Python AI Service:**
```json
{
  "detail": "Error message"
}
```

### Recommendation

Python service should implement custom exception handler to match Rust format for consistency. Frontend must currently handle both formats.

**Code Location for Fix:** `python-ai-service/error_handlers.py` (to be created)

---

## Related Documentation

- [API-RUST-CORE.md](./API-RUST-CORE.md) - Rust Core Engine API
- [API-WEBSOCKET.md](./API-WEBSOCKET.md) - WebSocket Protocol
- [API-SEQUENCES.mermaid](./API-SEQUENCES.mermaid) - API Sequence Diagrams
- [Functional Requirements](/specs/01-requirements/1.2-functional/FUNCTIONAL_REQUIREMENTS.md)

---

## Changelog

### Version 3.1.0 (2026-02-06)
- Added 8 previously undocumented endpoints:
  - `/predict-trend` - Legacy trend prediction
  - `/ai/cost/statistics` - OpenAI cost tracking
  - `/ai/config-analysis/trigger` - Config optimization trigger
  - `/ai/config-suggestions` - AI config recommendations
  - `/ai/gpt4-analysis-history` - Historical analysis viewer
  - `/api/chat/project` - Project chatbot interface
  - `/api/chat/project/suggestions` - Chat suggestions
  - `/api/chat/project/clear` - Clear chat history
- Added error format standardization section
- Updated version to 3.1.0

### Version 3.0.0 (2025-11-22)
- Added async task management endpoints
- Added training management endpoints
- Added backtest management endpoints
- Added monitoring & alerts endpoints

---

**Document Version:** 3.1.0
**Last Updated:** 2026-02-06
**Author:** Claude Code
**Status:** Complete
