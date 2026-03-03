# Bot Trading API Specification

## Overview
This document defines the API contracts for the cryptocurrency trading bot system. All services must adhere to these specifications to ensure proper integration.

## Service Architecture

### Service Endpoints
- **Python AI Service**: `http://localhost:8000`
- **Rust Core Engine**: `http://localhost:8080`
- **Frontend Dashboard**: `http://localhost:3000`

### Inter-Service Communication
- Services communicate via REST APIs over HTTP
- WebSocket for real-time updates
- MongoDB Atlas for shared data storage
- JWT tokens for authentication

## Python AI Service API

### 1. Health Check
```
GET /health
```

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-07-31T18:33:29.169981+00:00",
  "service": "GPT-4 Trading AI",
  "version": "2.0.0",
  "gpt4_available": true,
  "mongodb_connected": true,
  "analysis_interval_minutes": 5,
  "supported_symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT", "DOTUSDT", "XRPUSDT", "LINKUSDT"]
}
```

### 2. AI Trading Signal Analysis
```
POST /ai/analyze
```

**Request:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "candles": [
    {
      "open": 45123.45,
      "high": 45234.56,
      "low": 45012.34,
      "close": 45189.23,
      "volume": 1234.56,
      "timestamp": 1701234567000
    }
  ],
  "technical_indicators": {
    "rsi": 65.5,
    "macd": 123.45,
    "signal": 110.23,
    "histogram": 13.22,
    "ema_9": 45100.00,
    "ema_21": 44900.00,
    "ema_50": 44500.00,
    "bollinger_upper": 45500.00,
    "bollinger_middle": 45000.00,
    "bollinger_lower": 44500.00,
    "volume_sma": 1000.00,
    "atr": 234.56,
    "adx": 25.5,
    "stochastic_k": 75.5,
    "stochastic_d": 72.3
  },
  "market_context": {
    "trend_strength": 0.75,
    "volatility": 0.45,
    "volume_trend": "increasing",
    "market_sentiment": "bullish"
  }
}
```

**Response:**
```json
{
  "signal": "Long",
  "confidence": 0.75,
  "reasoning": "Strong bullish momentum with RSI showing strength without being overbought. MACD histogram positive and increasing. Price above all EMAs indicating uptrend.",
  "suggested_entry": 45190.00,
  "suggested_stop_loss": 44900.00,
  "suggested_take_profit": 45600.00,
  "risk_reward_ratio": 1.37,
  "position_size_recommendation": 0.02,
  "additional_insights": {
    "key_levels": [45000, 45500, 46000],
    "market_condition": "trending",
    "recommended_timeframe": "1h"
  },
  "metadata": {
    "analysis_id": "550e8400-e29b-41d4-a716-446655440000",
    "timestamp": "2025-07-31T18:33:29.169981+00:00",
    "model_version": "gpt-4o-mini",
    "processing_time_ms": 1234
  }
}
```

### 3. Strategy Recommendations
```
POST /ai/strategy-recommendations
```

**Request:**
```json
{
  "trading_style": "swing",
  "risk_tolerance": "medium",
  "capital": 10000,
  "experience_level": "intermediate",
  "preferred_timeframes": ["1h", "4h"],
  "preferred_pairs": ["BTCUSDT", "ETHUSDT"],
  "current_market_conditions": {
    "btc_dominance": 48.5,
    "total_market_cap": 1.75e12,
    "fear_greed_index": 65
  }
}
```

**Response:**
```json
{
  "recommended_strategies": [
    {
      "name": "EMA Crossover with RSI Filter",
      "suitability_score": 0.85,
      "expected_win_rate": 0.65,
      "expected_risk_reward": 1.5,
      "parameters": {
        "ema_fast": 9,
        "ema_slow": 21,
        "rsi_period": 14,
        "rsi_oversold": 30,
        "rsi_overbought": 70
      }
    }
  ],
  "position_sizing": {
    "method": "fixed_percentage",
    "percentage": 2,
    "max_concurrent_trades": 3
  },
  "risk_management": {
    "stop_loss_method": "atr_based",
    "atr_multiplier": 2,
    "trailing_stop": true,
    "max_daily_loss": 5
  }
}
```

### 4. Market Condition Analysis
```
POST /ai/market-condition
```

**Request:**
```json
{
  "symbols": ["BTCUSDT", "ETHUSDT"],
  "indicators": {
    "BTCUSDT": {
      "price": 45000,
      "volume_24h": 25000000000,
      "price_change_24h": 2.5
    },
    "ETHUSDT": {
      "price": 2500,
      "volume_24h": 15000000000,
      "price_change_24h": 3.2
    }
  }
}
```

**Response:**
```json
{
  "overall_market": "bullish",
  "market_phase": "accumulation",
  "volatility_level": "medium",
  "trend_strength": 0.72,
  "recommendations": [
    "Consider increasing position sizes in strong trends",
    "Watch for breakout opportunities above key resistance"
  ],
  "risk_factors": [
    "Approaching overbought conditions",
    "Low volume on recent rallies"
  ]
}
```

### 5. WebSocket Real-time Signals
```
WS /ws
```

**Connection:**
```javascript
ws://localhost:8000/ws
```

**Message Format (Server → Client):**
```json
{
  "type": "ai_signal",
  "data": {
    "symbol": "BTCUSDT",
    "signal": "Long",
    "confidence": 0.75,
    "timestamp": "2025-07-31T18:33:29.169981+00:00"
  }
}
```

## Rust Core Engine API

### 1. Health Check
```
GET /api/health
```

**Response:**
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "connected_services": {
    "python_ai": true,
    "binance_websocket": true,
    "mongodb": true
  }
}
```

### 2. Execute Trade
```
POST /api/trades/execute
```

**Headers:**
```
Authorization: Bearer <jwt_token>
Content-Type: application/json
```

**Request:**
```json
{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "LIMIT",
  "quantity": 0.001,
  "price": 45000.00,
  "time_in_force": "GTC",
  "reduce_only": false,
  "ai_signal_id": "550e8400-e29b-41d4-a716-446655440000",
  "strategy": "ai_ensemble"
}
```

**Response:**
```json
{
  "trade_id": "123e4567-e89b-12d3-a456-426614174000",
  "order_id": "binance_order_123456",
  "status": "FILLED",
  "executed_quantity": 0.001,
  "executed_price": 45000.00,
  "commission": 0.045,
  "commission_asset": "USDT",
  "timestamp": "2025-07-31T18:33:29.169981+00:00"
}
```

### 3. Get Open Positions
```
GET /api/positions
```

**Response:**
```json
{
  "positions": [
    {
      "symbol": "BTCUSDT",
      "side": "LONG",
      "quantity": 0.001,
      "entry_price": 45000.00,
      "current_price": 45200.00,
      "pnl": 0.20,
      "pnl_percentage": 0.44,
      "margin_used": 45.00,
      "liquidation_price": 40000.00
    }
  ],
  "total_margin_used": 45.00,
  "free_margin": 955.00,
  "margin_level": 2222.22
}
```

### 4. Trading History
```
GET /api/trades/history?symbol=BTCUSDT&limit=10
```

**Response:**
```json
{
  "trades": [
    {
      "trade_id": "123e4567-e89b-12d3-a456-426614174000",
      "symbol": "BTCUSDT",
      "side": "BUY",
      "price": 45000.00,
      "quantity": 0.001,
      "realized_pnl": 0,
      "commission": 0.045,
      "timestamp": "2025-07-31T18:33:29.169981+00:00"
    }
  ],
  "total": 150,
  "page": 1,
  "limit": 10
}
```

### 5. Account Information
```
GET /api/account
```

**Response:**
```json
{
  "balance": {
    "USDT": {
      "free": 955.00,
      "locked": 45.00,
      "total": 1000.00
    }
  },
  "total_balance_usdt": 1000.00,
  "total_pnl": 50.25,
  "total_pnl_percentage": 5.025,
  "positions_count": 1,
  "can_trade": true
}
```

## Error Responses

All services should return consistent error responses:

```json
{
  "error": {
    "code": "INSUFFICIENT_BALANCE",
    "message": "Insufficient balance for this trade",
    "details": {
      "required": 100.00,
      "available": 50.00
    },
    "timestamp": "2025-07-31T18:33:29.169981+00:00"
  }
}
```

### Standard Error Codes
- `INVALID_REQUEST`: Invalid request parameters
- `UNAUTHORIZED`: Missing or invalid authentication
- `FORBIDDEN`: User doesn't have permission
- `NOT_FOUND`: Resource not found
- `RATE_LIMITED`: Too many requests
- `INSUFFICIENT_BALANCE`: Not enough balance
- `INVALID_SYMBOL`: Trading pair not supported
- `SERVICE_UNAVAILABLE`: Service temporarily unavailable
- `INTERNAL_ERROR`: Internal server error

## Rate Limiting

### Python AI Service
- `/ai/analyze`: 10 requests per minute
- `/ai/strategy-recommendations`: 5 requests per minute
- Other endpoints: 60 requests per minute

### Rust Core Engine
- Trade execution: 10 requests per second
- Other endpoints: 100 requests per second

Rate limit headers:
```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1701234567
```

## Authentication

### JWT Token Structure
```json
{
  "sub": "user_id",
  "email": "user@example.com",
  "role": "trader",
  "permissions": ["trade", "view_history"],
  "exp": 1701234567,
  "iat": 1701230967
}
```

### Token Usage
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

## Webhooks

Services can register webhooks for events:

```
POST /api/webhooks
```

**Request:**
```json
{
  "url": "https://example.com/webhook",
  "events": ["trade_executed", "position_closed", "ai_signal_generated"],
  "secret": "webhook_secret_key"
}
```

**Webhook Payload:**
```json
{
  "event": "trade_executed",
  "data": {
    "trade_id": "123e4567-e89b-12d3-a456-426614174000",
    "symbol": "BTCUSDT",
    "side": "BUY",
    "price": 45000.00,
    "quantity": 0.001
  },
  "timestamp": "2025-07-31T18:33:29.169981+00:00",
  "signature": "sha256_hmac_signature"
}
```