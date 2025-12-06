# Binance REST API Research Report
**Date**: 2025-12-03 | **Topic**: REST API for Trading (Testnet & Mainnet)

---

## 1. Testnet vs Mainnet Overview

| Aspect | Testnet | Mainnet |
|--------|---------|---------|
| **Base URL** | `https://testnet.binance.vision/api` | `https://api.binance.com` |
| **Token Type** | Test tokens (no real value) | Real assets |
| **Data Sync** | NOT synced with mainnet; isolated environment | Live exchange data |
| **Data Reset** | Periodic resets per schedule | Permanent |
| **Purpose** | Feature testing, debugging, development | Live trading, real transactions |

**Key Implication**: Testnet architecture matches production but data != mainnet. Use testnet for API testing before mainnet deployment.

---

## 2. Order Placement API: POST /api/v3/order

### Available Order Types
- **MARKET** - Execute immediately at best available price
- **LIMIT** - Execute at specified price or better (waits in order book)
- **LIMIT_MAKER** - Limit order (fails if would execute immediately)
- **STOP_LOSS_LIMIT** - Limit order triggered at stop price
- **TAKE_PROFIT_LIMIT** - Limit order triggered at take profit price
- **TRAILING_STOP_MARKET** - Market order triggered at trailing stop price

### MARKET Order Example
```
POST /api/v3/order
{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "MARKET",
  "quantity": 0.001
}
```

### LIMIT Order Example
```
POST /api/v3/order
{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "LIMIT",
  "timeInForce": "GTC",
  "quantity": 0.001,
  "price": "40000.00"
}
```

### STOP_LOSS_LIMIT Order Example
```
POST /api/v3/order
{
  "symbol": "BTCUSDT",
  "side": "SELL",
  "type": "STOP_LOSS_LIMIT",
  "quantity": 0.001,
  "price": "39000.00",        # Execution price (limit)
  "stopPrice": "39500.00",     # Trigger price (stop)
  "timeInForce": "GTC"
}
```

### TAKE_PROFIT_LIMIT Order Example
```
POST /api/v3/order
{
  "symbol": "BTCUSDT",
  "side": "SELL",
  "type": "TAKE_PROFIT_LIMIT",
  "quantity": 0.001,
  "price": "45000.00",         # Execution price (limit)
  "stopPrice": "44500.00",     # Trigger price (take profit)
  "timeInForce": "GTC"
}
```

### Common Parameters
- **symbol**: Trading pair (e.g., BTCUSDT)
- **side**: BUY or SELL
- **type**: Order type (MARKET, LIMIT, etc.)
- **quantity**: Amount to trade
- **price**: Execution price (LIMIT orders)
- **stopPrice**: Trigger price (STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT)
- **timeInForce**: GTC (Good-Till-Cancel), IOC (Immediate-Or-Cancel), FOK (Fill-Or-Kill)

### Validation Rules
- Must pass FILTER_FAILURE checks (minimum notional, tick size)
- Price must conform to pair's tickSize
- Quantity must conform to pair's stepSize
- Use `GET /api/v3/exchangeInfo` to validate constraints

### Important Note (2025 Update)
Timestamp validation: If `timestamp > serverTime + 1 second`, request rejected and counts against rate limits.

---

## 3. Order Management Endpoints

### Query Order
```
GET /api/v3/order?symbol=BTCUSDT&orderId=12345
```
Response: Full order details (status, filled qty, etc.)

### Cancel Order
```
DELETE /api/v3/order?symbol=BTCUSDT&orderId=12345
```
Response: Canceled order details

### List Open Orders
```
GET /api/v3/openOrders?symbol=BTCUSDT
```
Response: Array of all open orders for symbol

### Order History
```
GET /api/v3/allOrders?symbol=BTCUSDT
```
Response: All orders (filled/canceled) for symbol, paginated

---

## 4. Authentication: HMAC SHA256 + API Key

### X-MBX-APIKEY Header
All signed requests require API key in header:
```
X-MBX-APIKEY: your-api-key-here
```

### HMAC SHA256 Signature Generation
1. Create query string from parameters: `symbol=BTCUSDT&quantity=0.001&price=40000`
2. Generate signature: `HMAC-SHA256(query_string, api_secret_key)`
3. Add signature to request: `&signature=abc123...`

### Python Example
```python
import hmac
import hashlib
import requests
from urllib.parse import urlencode

api_key = "your-api-key"
api_secret = "your-api-secret"

params = {
    "symbol": "BTCUSDT",
    "side": "BUY",
    "type": "MARKET",
    "quantity": 0.001,
    "timestamp": int(time.time() * 1000)
}

query_string = urlencode(params)
signature = hmac.new(
    api_secret.encode(),
    query_string.encode(),
    hashlib.sha256
).hexdigest()

params["signature"] = signature

headers = {"X-MBX-APIKEY": api_key}
response = requests.post(
    "https://api.binance.com/api/v3/order",
    params=params,
    headers=headers
)
```

### Timestamp Requirements
- Include `timestamp` parameter (milliseconds since epoch)
- Server check: Must be within `recvWindow` (default 5000ms)
- Validation: Reject if `timestamp > serverTime + 1 second`

### Key Types (Security Recommendation)
- ✅ **Ed25519 keys** (recommended): Best performance & security
- **RSA keys**: PKCS#8 format supported
- **HMAC** (legacy): Still supported but less secure

### Security Best Practices
- Store secrets in environment variables or HSM, NOT in code
- Use Ed25519 keys for new integrations
- Never log or expose API keys
- Implement key rotation policy

---

## 5. Rate Limits & Best Practices

### Request Weight Limits
- **Limit**: 6,000 weight per minute (IP-based)
- **Exceeded**: HTTP 429 (Too Many Requests)
- **Repeated violation**: HTTP 418 (IP banned, duration: 2 min - 3 days)
- Each endpoint has different weight (e.g., GET /order = 2, POST /order = 1)

### Order Rate Limits
- **Placement**: 50 orders per 10 seconds (account-based)
- **Daily**: 160,000 orders per 24 hours
- **Exceeded**: Restricted from creating orders (website + API)

### Rate Limit Headers
Every response includes:
```
X-MBX-USED-WEIGHT-1m: 1234      # Current minute weight used
X-MBX-ORDER-COUNT-1s: 2          # Orders placed this second
X-MBX-ORDER-COUNT-10s: 5         # Orders placed last 10 seconds
```

### Monitoring & Avoidance
```
GET /api/v3/exchangeInfo
# Returns rateLimits array with RAW_REQUESTS, REQUEST_WEIGHT, ORDERS
```

### Backoff Strategy (CRITICAL!)
1. Receive HTTP 429 → **Back off immediately** (exponential backoff)
2. Do NOT retry immediately or spam API
3. Respect `Retry-After` header if present
4. Failure to backoff = IP ban escalation (2 min → 3 days)

### WebSocket Rate Limits
- **Message rate**: 5 incoming messages/second (per connection)
- **Max streams**: 1,024 per connection
- **Connection limit**: 300 per 5 minutes per IP

---

## 6. Critical Timestamp & recvWindow Changes (2025)

Starting 2025-06-06, Binance added stricter timestamp validation:
- Requests with `timestamp > serverTime + 1 second` are **rejected**
- Rejected requests **count against rate limits**
- Recommendation: Use UTC time from `GET /api/v3/time` endpoint

```
GET /api/v3/time
Response: {"serverTime": 1701620400000}
```

---

## Summary: Implementation Checklist

- [ ] Use testnet.binance.vision for development/testing
- [ ] Implement all order types: MARKET, LIMIT, STOP_LOSS_LIMIT, TAKE_PROFIT_LIMIT
- [ ] Generate HMAC-SHA256 signatures with api_secret
- [ ] Include X-MBX-APIKEY header for signed endpoints
- [ ] Add timestamp (UTC milliseconds) to every request
- [ ] Monitor rate limit headers in responses
- [ ] Implement exponential backoff for HTTP 429
- [ ] Validate orders against exchange constraints (GET /exchangeInfo)
- [ ] Use Ed25519 keys for maximum security
- [ ] Store API secrets in environment variables only

---

## Sources

- [Binance Spot Testnet Documentation](https://developers.binance.com/docs/binance-spot-api-docs/testnet/rest-api/general-api-information)
- [Binance Trading Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints)
- [Binance Request Security (HMAC & Authentication)](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/request-security)
- [Binance API Rate Limits](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/limits)
- [Binance HMAC Signature Guide](https://academy.binance.com/en/articles/hmac-signature-what-it-is-and-how-to-use-it-for-binance-api-security)
- [How to Avoid Rate Limit Bans](https://academy.binance.com/en/articles/how-to-avoid-getting-banned-by-rate-limits)
