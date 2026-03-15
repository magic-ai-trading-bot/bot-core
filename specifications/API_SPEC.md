# Bot Trading API Specification

## Overview
This document defines the API contracts for the cryptocurrency trading bot system. All services must adhere to these specifications to ensure proper integration.

## Service Architecture

### Service Endpoints
- **Rust Core Engine**: `http://localhost:8080`
- **Frontend Dashboard**: `http://localhost:3000`

### Inter-Service Communication
- Services communicate via REST APIs over HTTP
- WebSocket for real-time updates
- MongoDB Atlas for shared data storage
- JWT tokens for authentication

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