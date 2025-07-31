# AI Analysis API Examples

## Basic Analysis Request

### Request
```bash
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{
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
  }'
```

### Successful Response
```json
{
  "signal": "Long",
  "confidence": 0.75,
  "reasoning": "Strong bullish momentum with RSI showing strength without being overbought. MACD histogram positive and increasing. Price above all EMAs indicating uptrend. Volume trend supports bullish continuation.",
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

## Minimal Request Example

### Request (Minimum Required Fields)
```bash
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "ETHUSDT",
    "timeframe": "4h",
    "candles": [
      {
        "open": 2500.00,
        "high": 2550.00,
        "low": 2480.00,
        "close": 2520.00,
        "volume": 50000.00,
        "timestamp": 1701234567000
      }
    ]
  }'
```

### Response
```json
{
  "signal": "Neutral",
  "confidence": 0.55,
  "reasoning": "Insufficient data for strong signal. Price action shows indecision with small body candle. More historical data needed for accurate analysis.",
  "metadata": {
    "analysis_id": "650e8400-e29b-41d4-a716-446655440001",
    "timestamp": "2025-07-31T18:35:29.169981+00:00",
    "model_version": "gpt-4o-mini",
    "processing_time_ms": 890
  }
}
```

## Error Examples

### Invalid Symbol
```bash
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "INVALID",
    "timeframe": "1h",
    "candles": []
  }'
```

**Response:**
```json
{
  "error": {
    "code": "INVALID_SYMBOL",
    "message": "Symbol 'INVALID' is not supported",
    "details": {
      "supported_symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT", "DOTUSDT", "XRPUSDT", "LINKUSDT"]
    },
    "timestamp": "2025-07-31T18:36:29.169981+00:00"
  }
}
```

### Insufficient Candles
```bash
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candles": []
  }'
```

**Response:**
```json
{
  "error": {
    "code": "INSUFFICIENT_DATA",
    "message": "Not enough candle data for analysis",
    "details": {
      "provided": 0,
      "minimum_required": 1
    },
    "timestamp": "2025-07-31T18:37:29.169981+00:00"
  }
}
```

## WebSocket Real-time Signals

### Connection
```javascript
const ws = new WebSocket('ws://localhost:8000/ws');

ws.onopen = () => {
  console.log('Connected to AI service WebSocket');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

### Sample Messages
```json
// AI Signal Generated
{
  "type": "ai_signal",
  "data": {
    "symbol": "BTCUSDT",
    "signal": "Long",
    "confidence": 0.82,
    "timestamp": "2025-07-31T18:40:00.000Z"
  }
}

// Analysis Update
{
  "type": "analysis_update",
  "data": {
    "symbols_analyzed": ["BTCUSDT", "ETHUSDT"],
    "next_analysis": "2025-07-31T18:45:00.000Z"
  }
}

// Error Notification
{
  "type": "error",
  "data": {
    "message": "AI analysis temporarily unavailable",
    "retry_after": 60
  }
}
```

## Batch Analysis Example

### Request (Multiple Symbols)
```bash
curl -X POST http://localhost:8000/ai/analyze-batch \
  -H "Content-Type: application/json" \
  -d '{
    "analyses": [
      {
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "candles": [...]
      },
      {
        "symbol": "ETHUSDT",
        "timeframe": "1h",
        "candles": [...]
      }
    ]
  }'
```

### Response
```json
{
  "results": [
    {
      "symbol": "BTCUSDT",
      "signal": "Long",
      "confidence": 0.75,
      "reasoning": "..."
    },
    {
      "symbol": "ETHUSDT",
      "signal": "Short",
      "confidence": 0.68,
      "reasoning": "..."
    }
  ],
  "metadata": {
    "batch_id": "750e8400-e29b-41d4-a716-446655440000",
    "total_processing_time_ms": 2500
  }
}
```

## Rate Limiting Example

### Exceeded Rate Limit
```bash
# After making too many requests
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{...}'
```

**Response:**
```json
{
  "error": {
    "code": "RATE_LIMITED",
    "message": "Too many requests. Please try again later.",
    "details": {
      "limit": 10,
      "window": "1 minute",
      "retry_after": 45
    },
    "timestamp": "2025-07-31T18:45:29.169981+00:00"
  }
}
```

**Response Headers:**
```
X-RateLimit-Limit: 10
X-RateLimit-Remaining: 0
X-RateLimit-Reset: 1701234567
```