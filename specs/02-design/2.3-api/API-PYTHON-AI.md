# API-PYTHON-AI.md - Python AI Service API Specification

**Version:** 2.0.0
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
7. [WebSocket Real-time Updates](#websocket-real-time-updates)
8. [Error Codes](#error-codes)
9. [Rate Limiting](#rate-limiting)

---

## Overview

The Python AI Service provides GPT-4 powered trading signal generation, technical analysis, strategy recommendations, and market condition analysis. It integrates with OpenAI's GPT-4o-mini model for intelligent trading decisions.

**Service Architecture:**
- **Language:** Python 3.11+
- **Framework:** FastAPI
- **AI Model:** GPT-4o-mini (OpenAI)
- **Database:** MongoDB (for caching analysis results)
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
| 400 | Bad Request | Invalid request parameters |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | OpenAI API unavailable |

### Common Error Messages

| Error | Description | Solution |
|-------|-------------|----------|
| `AI analysis failed: <reason>` | Analysis request failed | Check request format and OpenAI status |
| `OpenAI rate limit exceeded` | Too many OpenAI API calls | Wait for rate limit reset or use backup keys |
| `MongoDB not connected` | Database unavailable | Check MongoDB connection |
| `GPT-4 client not initialized` | OpenAI client setup failed | Verify API key configuration |
| `All API keys exhausted or rate limited` | All backup keys rate limited | Wait 1 hour or add more keys |

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

## Related Documentation

- [API-RUST-CORE.md](./API-RUST-CORE.md) - Rust Core Engine API
- [API-WEBSOCKET.md](./API-WEBSOCKET.md) - WebSocket Protocol
- [API-SEQUENCES.mermaid](./API-SEQUENCES.mermaid) - API Sequence Diagrams
- [Functional Requirements](/specs/01-requirements/1.2-functional/FUNCTIONAL_REQUIREMENTS.md)

---

**Document Version:** 2.0.0
**Last Updated:** 2025-10-10
**Author:** Claude Code
**Status:** Complete
