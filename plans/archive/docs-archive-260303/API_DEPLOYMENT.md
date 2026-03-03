# API Deployment Documentation - Bot Core

**Version:** 2.0.0
**Last Updated:** 2025-11-18
**Base URL (Production):** `https://api.your-domain.com`
**Base URL (Staging):** `https://staging-api.your-domain.com`

---

## Table of Contents

1. [API Overview](#api-overview)
2. [Authentication](#authentication)
3. [API Endpoints](#api-endpoints)
4. [WebSocket Protocol](#websocket-protocol)
5. [Rate Limiting](#rate-limiting)
6. [Error Responses](#error-responses)
7. [Testing Endpoints](#testing-endpoints)
8. [Deployment Configuration](#deployment-configuration)

---

## API Overview

### Service Architecture

```
┌─────────────────────────────────────────────┐
│         Kong API Gateway (8100)             │
│  Rate Limiting, Auth, API Versioning        │
└────────────┬────────────────────────────────┘
             │
        ┌────┴────┬─────────────┐
        │         │             │
   ┌────▼────┐┌───▼─────┐┌─────▼────┐
   │ Rust    ││ Python  ││ Next.js  │
   │ Core    ││ AI      ││ UI       │
   │ :8080   ││ :8000   ││ :3000    │
   └─────────┘└─────────┘└──────────┘
```

### API Services

| Service | Port | Protocol | Purpose |
|---------|------|----------|---------|
| Rust Core Engine | 8080 | HTTP/WS | Trading, Market Data, Auth |
| Python AI Service | 8000 | HTTP | ML Predictions, Sentiment |
| Next.js Dashboard | 3000 | HTTP | Frontend UI |
| Kong Gateway | 8100 | HTTP | API Management |

### API Versions

- **v1** (Current): `/api/v1/*`
- **v2** (Beta): `/api/v2/*` - New features, backward compatible

---

## Authentication

### JWT Authentication (RS256)

All protected endpoints require JWT token in the `Authorization` header.

#### 1. Login

**Endpoint:** `POST /api/v1/auth/login`

**Request:**
```json
{
  "email": "user@example.com",
  "password": "your_password"
}
```

**Response:**
```json
{
  "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
  "expiresIn": 86400,
  "user": {
    "id": "user_123",
    "email": "user@example.com",
    "createdAt": "2025-01-01T00:00:00Z"
  }
}
```

**cURL Example:**
```bash
curl -X POST https://api.your-domain.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"user@example.com","password":"your_password"}'
```

#### 2. Register (If Enabled)

**Endpoint:** `POST /api/v1/auth/register`

**Request:**
```json
{
  "email": "newuser@example.com",
  "password": "secure_password",
  "name": "John Doe"
}
```

#### 3. Using JWT Token

**Add to all authenticated requests:**
```bash
curl -X GET https://api.your-domain.com/api/v1/portfolio \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

### API Key Authentication (Service-to-Service)

For service-to-service communication:

**Header:** `X-API-Key: your_api_key`

```bash
curl -X GET https://api.your-domain.com/api/v1/internal/health \
  -H "X-API-Key: your_rust_api_key"
```

---

## API Endpoints

### Health & Status

#### GET /api/health
**Authentication:** None
**Description:** Basic health check

**Response:**
```json
{
  "status": "healthy",
  "version": "2.0.0",
  "timestamp": "2025-11-18T10:00:00Z"
}
```

#### GET /api/v1/health
**Authentication:** None
**Description:** Detailed system health

**Response:**
```json
{
  "status": "healthy",
  "version": "2.0.0",
  "services": {
    "database": "connected",
    "binance": "connected",
    "pythonAi": "connected"
  },
  "uptime": 86400,
  "timestamp": "2025-11-18T10:00:00Z"
}
```

### Trading Endpoints

#### GET /api/v1/portfolio
**Authentication:** Required (JWT)
**Description:** Get user portfolio

**Response:**
```json
{
  "userId": "user_123",
  "totalValue": 10000.50,
  "positions": [
    {
      "symbol": "BTCUSDT",
      "quantity": 0.5,
      "avgPrice": 45000.00,
      "currentPrice": 46000.00,
      "profitLoss": 500.00,
      "profitLossPercent": 2.22
    }
  ],
  "cash": 5000.00,
  "lastUpdated": "2025-11-18T10:00:00Z"
}
```

#### POST /api/v1/trades
**Authentication:** Required (JWT)
**Description:** Execute trade (paper or live)

**Request:**
```json
{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "quantity": 0.01,
  "orderType": "MARKET",
  "timeInForce": "GTC"
}
```

**Response:**
```json
{
  "tradeId": "trade_456",
  "orderId": "binance_789",
  "symbol": "BTCUSDT",
  "side": "BUY",
  "quantity": 0.01,
  "price": 46000.00,
  "status": "FILLED",
  "executedAt": "2025-11-18T10:00:00Z"
}
```

#### GET /api/v1/trades
**Authentication:** Required (JWT)
**Description:** Get trade history

**Query Parameters:**
- `limit` (default: 50, max: 100)
- `offset` (default: 0)
- `symbol` (optional)
- `startDate` (optional)
- `endDate` (optional)

**Response:**
```json
{
  "trades": [
    {
      "tradeId": "trade_456",
      "symbol": "BTCUSDT",
      "side": "BUY",
      "quantity": 0.01,
      "price": 46000.00,
      "status": "FILLED",
      "executedAt": "2025-11-18T10:00:00Z"
    }
  ],
  "total": 150,
  "limit": 50,
  "offset": 0
}
```

### Market Data Endpoints

#### GET /api/v1/market/price/:symbol
**Authentication:** Required (JWT)
**Description:** Get current price for symbol

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "price": 46000.00,
  "timestamp": "2025-11-18T10:00:00Z"
}
```

#### GET /api/v1/market/klines/:symbol
**Authentication:** Required (JWT)
**Description:** Get candlestick data

**Query Parameters:**
- `interval` (1m, 5m, 15m, 1h, 4h, 1d)
- `limit` (default: 100, max: 1000)

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "interval": "1h",
  "klines": [
    {
      "openTime": 1700000000000,
      "open": "45900.00",
      "high": "46100.00",
      "low": "45800.00",
      "close": "46000.00",
      "volume": "123.45",
      "closeTime": 1700003600000
    }
  ]
}
```

### Strategy Endpoints

#### GET /api/v1/strategies
**Authentication:** Required (JWT)
**Description:** List user strategies

**Response:**
```json
{
  "strategies": [
    {
      "strategyId": "strategy_123",
      "name": "RSI Scalping",
      "type": "RSI",
      "symbols": ["BTCUSDT", "ETHUSDT"],
      "enabled": true,
      "performance": {
        "totalTrades": 50,
        "winRate": 65.5,
        "profitLoss": 1250.00
      }
    }
  ]
}
```

#### POST /api/v1/strategies
**Authentication:** Required (JWT)
**Description:** Create new strategy

**Request:**
```json
{
  "name": "My RSI Strategy",
  "type": "RSI",
  "symbols": ["BTCUSDT"],
  "parameters": {
    "rsiPeriod": 14,
    "oversold": 30,
    "overbought": 70
  },
  "riskManagement": {
    "maxPositionSize": 0.1,
    "stopLoss": 2.0,
    "takeProfit": 5.0
  }
}
```

### AI Prediction Endpoints

#### POST /api/v1/ai/predict
**Authentication:** Required (JWT)
**Description:** Get price prediction

**Request:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "horizon": 24
}
```

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "currentPrice": 46000.00,
  "predictions": [
    {
      "timestamp": "2025-11-18T11:00:00Z",
      "predictedPrice": 46100.00,
      "confidence": 0.85
    },
    {
      "timestamp": "2025-11-18T12:00:00Z",
      "predictedPrice": 46150.00,
      "confidence": 0.80
    }
  ],
  "modelUsed": "LSTM",
  "generatedAt": "2025-11-18T10:00:00Z"
}
```

#### POST /api/v1/ai/sentiment
**Authentication:** Required (JWT)
**Description:** Get market sentiment analysis

**Request:**
```json
{
  "symbol": "BTCUSDT",
  "sources": ["twitter", "news"]
}
```

**Response:**
```json
{
  "symbol": "BTCUSDT",
  "sentiment": {
    "score": 0.65,
    "label": "BULLISH",
    "confidence": 0.80
  },
  "sources": {
    "twitter": 0.70,
    "news": 0.60
  },
  "generatedAt": "2025-11-18T10:00:00Z"
}
```

---

## WebSocket Protocol

### Connection

**URL:** `wss://api.your-domain.com/ws`

**Authentication:** Send JWT token in first message

```javascript
const ws = new WebSocket('wss://api.your-domain.com/ws');

ws.onopen = () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'YOUR_JWT_TOKEN'
  }));
};
```

### Message Types

#### Subscribe to Market Data

**Send:**
```json
{
  "type": "subscribe",
  "channel": "market",
  "symbols": ["BTCUSDT", "ETHUSDT"]
}
```

**Receive:**
```json
{
  "type": "market_update",
  "symbol": "BTCUSDT",
  "price": 46000.00,
  "volume": 123.45,
  "timestamp": "2025-11-18T10:00:00Z"
}
```

#### Subscribe to Trade Updates

**Send:**
```json
{
  "type": "subscribe",
  "channel": "trades"
}
```

**Receive:**
```json
{
  "type": "trade_update",
  "tradeId": "trade_456",
  "symbol": "BTCUSDT",
  "status": "FILLED",
  "timestamp": "2025-11-18T10:00:00Z"
}
```

#### Heartbeat

**Send/Receive:**
```json
{
  "type": "ping"
}
```

**Response:**
```json
{
  "type": "pong",
  "timestamp": "2025-11-18T10:00:00Z"
}
```

### WebSocket Error Codes

| Code | Description |
|------|-------------|
| 1000 | Normal Closure |
| 1001 | Going Away |
| 1002 | Protocol Error |
| 1003 | Unsupported Data |
| 1008 | Policy Violation (Auth Failed) |
| 1011 | Internal Server Error |

---

## Rate Limiting

### Limits

| Endpoint Type | Rate Limit | Window |
|---------------|------------|--------|
| Public (Health) | 100 req/min | Per IP |
| Authenticated | 1000 req/min | Per User |
| Trading | 100 req/min | Per User |
| WebSocket | 50 msg/sec | Per Connection |

### Rate Limit Headers

**Response Headers:**
```
X-RateLimit-Limit: 1000
X-RateLimit-Remaining: 995
X-RateLimit-Reset: 1700000060
```

### Rate Limit Exceeded

**Status Code:** 429 Too Many Requests

**Response:**
```json
{
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Too many requests. Please try again in 30 seconds.",
  "retryAfter": 30,
  "limit": 1000,
  "window": 60
}
```

---

## Error Responses

### Standard Error Format

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable error message",
  "details": {
    "field": "specific error details"
  },
  "timestamp": "2025-11-18T10:00:00Z",
  "requestId": "req_abc123"
}
```

### Common Error Codes

| Code | Status | Description |
|------|--------|-------------|
| UNAUTHORIZED | 401 | Invalid or missing JWT token |
| FORBIDDEN | 403 | Insufficient permissions |
| NOT_FOUND | 404 | Resource not found |
| VALIDATION_ERROR | 400 | Invalid request data |
| RATE_LIMIT_EXCEEDED | 429 | Too many requests |
| INTERNAL_ERROR | 500 | Server error |
| SERVICE_UNAVAILABLE | 503 | Service temporarily unavailable |

### Example Error Responses

**401 Unauthorized:**
```json
{
  "error": "UNAUTHORIZED",
  "message": "Invalid or expired JWT token",
  "timestamp": "2025-11-18T10:00:00Z"
}
```

**400 Validation Error:**
```json
{
  "error": "VALIDATION_ERROR",
  "message": "Invalid request data",
  "details": {
    "quantity": "Must be greater than 0",
    "symbol": "Invalid trading pair"
  },
  "timestamp": "2025-11-18T10:00:00Z"
}
```

---

## Testing Endpoints

### Development Environment

**Base URL:** `http://localhost:8080`

### Staging Environment

**Base URL:** `https://staging-api.your-domain.com`

### Testing Tools

#### cURL Examples

```bash
# Health check
curl https://api.your-domain.com/api/health

# Login
TOKEN=$(curl -X POST https://api.your-domain.com/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}' | jq -r '.token')

# Get portfolio
curl https://api.your-domain.com/api/v1/portfolio \
  -H "Authorization: Bearer $TOKEN"
```

#### Postman Collection

Import the provided Postman collection:
```bash
# Download collection
curl -o bot-core-api.postman_collection.json \
  https://api.your-domain.com/docs/postman-collection
```

#### API Documentation (Swagger)

**URL:** `https://api.your-domain.com/api/docs`

Interactive API documentation with try-it-out functionality.

---

## Deployment Configuration

### Environment Variables

```bash
# API Configuration
API_HOST=0.0.0.0
API_PORT=8080
API_BASE_URL=https://api.your-domain.com

# CORS Configuration
CORS_ALLOWED_ORIGINS=https://your-domain.com,https://app.your-domain.com

# Rate Limiting
RATE_LIMIT_WINDOW_MS=60000
RATE_LIMIT_MAX_REQUESTS=1000

# JWT Configuration
JWT_SECRET=your_jwt_secret_key
JWT_EXPIRATION=86400
JWT_ALGORITHM=RS256

# External APIs
BINANCE_API_KEY=your_binance_api_key
BINANCE_SECRET_KEY=your_binance_secret_key
OPENAI_API_KEY=your_openai_api_key
```

### Nginx Reverse Proxy Configuration

```nginx
server {
    listen 443 ssl http2;
    server_name api.your-domain.com;

    ssl_certificate /etc/ssl/certs/api.your-domain.com.crt;
    ssl_certificate_key /etc/ssl/private/api.your-domain.com.key;

    # API Proxy
    location /api/ {
        proxy_pass http://localhost:8080;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;

        # WebSocket support
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";

        # Timeouts
        proxy_connect_timeout 60s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
    }

    # WebSocket endpoint
    location /ws {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

### Kong API Gateway Configuration

```yaml
_format_version: "2.1"
services:
  - name: rust-core-engine
    url: http://rust-core-engine:8080
    routes:
      - name: api-routes
        paths:
          - /api
    plugins:
      - name: rate-limiting
        config:
          minute: 1000
          policy: local
      - name: jwt
        config:
          key_claim_name: iss
      - name: cors
        config:
          origins:
            - https://your-domain.com
          credentials: true
```

---

## API Client Examples

### JavaScript/TypeScript

```typescript
class BotCoreClient {
  private baseUrl: string;
  private token: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  async login(email: string, password: string) {
    const response = await fetch(`${this.baseUrl}/api/v1/auth/login`, {
      method: 'POST',
      headers: { 'Content-Type': 'application/json' },
      body: JSON.stringify({ email, password })
    });
    const data = await response.json();
    this.token = data.token;
    return data;
  }

  async getPortfolio() {
    const response = await fetch(`${this.baseUrl}/api/v1/portfolio`, {
      headers: { 'Authorization': `Bearer ${this.token}` }
    });
    return response.json();
  }
}

// Usage
const client = new BotCoreClient('https://api.your-domain.com');
await client.login('user@example.com', 'password');
const portfolio = await client.getPortfolio();
```

### Python

```python
import requests

class BotCoreClient:
    def __init__(self, base_url):
        self.base_url = base_url
        self.token = None

    def login(self, email, password):
        response = requests.post(
            f"{self.base_url}/api/v1/auth/login",
            json={"email": email, "password": password}
        )
        data = response.json()
        self.token = data["token"]
        return data

    def get_portfolio(self):
        response = requests.get(
            f"{self.base_url}/api/v1/portfolio",
            headers={"Authorization": f"Bearer {self.token}"}
        )
        return response.json()

# Usage
client = BotCoreClient("https://api.your-domain.com")
client.login("user@example.com", "password")
portfolio = client.get_portfolio()
```

---

## Support

- **API Status:** https://status.your-domain.com
- **Documentation:** https://docs.your-domain.com
- **Support Email:** support@your-domain.com

---

**Document Version:** 2.0.0
**API Version:** v1 (stable)
**Last Updated:** 2025-11-18
