# Binance API Research: Spot & Futures Trading
**Date**: 2026-02-06 | **Status**: Comprehensive Analysis

## 1. SPOT TRADING API ENDPOINTS

### Core Order Endpoints
| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/api/v3/order` | POST | Place new order (LIMIT, MARKET, STOP_LOSS, TAKE_PROFIT, etc.) |
| `/api/v3/order/test` | POST | Test order without execution |
| `/api/v3/order` | DELETE | Cancel active order |
| `/api/v3/openOrders` | DELETE | Cancel all orders on symbol |
| `/api/v3/order/cancelReplace` | POST | Atomic cancel + place new order |

### Order List Endpoints (Complex Orders)
- **OCO**: `POST /api/v3/orderList/oco` - One-Cancels-Other (2 orders)
- **OTO**: `POST /api/v3/orderList/oto` - One-Triggers-Other (2 orders)
- **OTOCO**: `POST /api/v3/orderList/otoco` - One-Triggers-One-Cancels-Other (3 orders)
- **SOR**: `POST /api/v3/sor/order` - Smart Order Routing (best execution)

### Account Endpoints
- `GET /api/v3/account` - Get account info (balances, trading status)
- `GET /api/v3/openOrders` - List all open orders
- `GET /api/v3/allOrders` - Get order history
- `GET /api/v3/myTrades` - Get trade history

**Auth**: All endpoints require API key + signature (TRADE permission)

---

## 2. SPOT ORDER TYPES SUPPORTED

1. **MARKET** - Instant execution at best market price
2. **LIMIT** - Execute only at specified price or better
3. **STOP_LOSS** - Trigger when price falls below stopPrice
4. **STOP_LOSS_LIMIT** - Trigger + limit order combo
5. **TAKE_PROFIT** - Trigger when price rises above stopPrice
6. **TAKE_PROFIT_LIMIT** - Trigger + limit order combo
7. **LIMIT_MAKER** - Limit order (rejected if would match immediately)
8. **TRAILING_STOP_LOSS** - Dynamic stop loss (trailingDelta parameter)
9. **OCO** - One-Cancels-Other (stop + limit pair)

**Trailing Stop**: Available for STOP_LOSS, STOP_LOSS_LIMIT, TAKE_PROFIT, TAKE_PROFIT_LIMIT orders. Uses `trailingDelta` parameter instead of fixed stopPrice. Automatically adjusts stop price as market price moves favorably.

---

## 3. FUTURES (USDⓈ-M) API ENDPOINTS

### Base URL
- **Production**: `https://fapi.binance.com`
- **Testnet**: `https://testnet.binancefuture.com`
- **WebSocket**: `wss://fstream.binance.com` / `wss://stream-fapi.binance.com`

### Key Endpoints
| Endpoint | Method | Purpose |
|----------|--------|---------|
| `/fapi/v1/order` | POST | Place futures order |
| `/fapi/v1/order` | DELETE | Cancel order |
| `/fapi/v1/openOrders` | GET | List open orders |
| `/fapi/v1/positionRisk` | GET | Get position info & risk |
| `/fapi/v1/account` | GET | Account details & balances |
| `/fapi/v1/allOrders` | GET | Order history |

### Order Types in Futures
- MARKET, LIMIT, STOP, TAKE_PROFIT, TAKE_PROFIT_MARKET, STOP_MARKET
- TRAILING_STOP_MARKET (dynamic trailing stop)
- Conditional orders (stopPrice, triggerType)

---

## 4. TESTNET VS PRODUCTION

### Spot Testnet
- **URL**: `https://testnet.binance.vision`
- **Virtual Balances**: Auto-receive test assets
- **Data Sync**: Not always synced with live exchange
- **Resets**: Periodic resets to blank state
- **API Keys**: Separate from production

### Futures Testnet
- **URL**: `https://testnet.binancefuture.com`
- **Mock Data**: May not perfectly match live trading
- **Rate Limits**: Same as production (for strategy testing)
- **API Keys**: Separate from production

**Critical**: All testnet funds are virtual and cannot transfer in/out. Test thoroughly before enabling production trading.

---

## 5. RATE LIMITS

### REST API (Both Spot & Futures)
- **RAW_REQUEST**: 1,200/min per IP
- **REQUEST_WEIGHT**: 6,000 weight/min (varies by endpoint: 1-10 weight)
- **ORDER_RATE**: 100 orders/10s per symbol

**Headers**: Response includes `X-MBX-USED-WEIGHT-(intervalNum)(intervalLetter)` showing current usage

### WebSocket Limits
- **Message Rate**: 10 incoming messages/sec per connection
- **Stream Subscription**: Max 1,024 streams per connection
- **Ping/Pong**: Server pings every 3 min; disconnects if no pong in 10 min

**HTTP 429**: Returned when breaking rate limit

---

## 6. WEBSOCKET ORDER UPDATES

### Event Streams for Trading
- **Balance Update**: User balance changes
- **Order Update**: Order placement, execution, cancellation
- **Position Update** (Futures): Leverage, margin, position changes
- **Account Update**: Account configuration changes

### Connection Protocol
```
wss://stream.binance.com:9443/ws  (Spot)
wss://fstream.binance.com          (Futures)
```
- Connection requires API key for private streams
- Each stream has specific namespace (e.g., `execution`, `balanceUpdate`)
- Real-time order fill/cancel notifications

---

## 7. KEY IMPLEMENTATION NOTES

### Security Best Practices
1. Use IP whitelisting for API keys
2. Enable API key restrictions (SPOT, MARGIN, FUTURES separately)
3. Use post-only orders (LIMIT_MAKER) to avoid taker fees
4. Implement request signing: HMAC-SHA256(payload, secretKey)
5. Store API keys in secure .env files (never commit)

### Strategy Considerations
- **Slippage**: Factor in for MARKET orders; use LIMIT for precision
- **Rate Limits**: Monitor weight usage; batch requests when possible
- **Order Atomicity**: Use cancelReplace for atomic order updates
- **OCO Safety**: Perfect for stop-loss + take-profit automation
- **Trailing Stops**: Superior to fixed stops in trending markets

### Error Handling
- -1000: Invalid request (validation error)
- -1001: Too many requests (rate limit)
- -2010: Insufficient balance
- -2015: Invalid API key
- Implement exponential backoff for 429 responses

---

## SOURCES

- [Spot Trading Endpoints](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints)
- [Spot Order Types & Trailing Stop FAQ](https://developers.binance.com/docs/binance-spot-api-docs/faqs/trailing-stop-faq)
- [Futures WebSocket API](https://developers.binance.com/docs/derivatives/usds-margined-futures/websocket-api-general-info)
- [Rate Limits Reference](https://developers.binance.com/docs/binance-spot-api-docs/websocket-api/rate-limits)
- [Testnet Environments](https://dev.binance.vision/t/binance-testnet-environments/99)
