# Data Flow Architecture

**Spec ID**: ARCH-DATA-FLOW-001 to ARCH-DATA-FLOW-007
**Version**: 1.0
**Status**: ☑ Approved
**Owner**: Architecture Team
**Last Updated**: 2025-10-11

---

## Tasks Checklist

- [x] Requirements gathered
- [x] Design completed
- [x] Implementation done
- [x] Sequence diagrams created
- [x] Data transformation documented
- [ ] Performance benchmarks completed
- [ ] Documentation reviewed
- [ ] Code reviewed

---

## Metadata

**Related Specs**:
- Related FR: FR-TRADING-001, FR-AI-001, FR-MARKET-DATA-001
- Related Design: ARCH-OVERVIEW-001, ARCH-MICROSERVICES-001
- Related NFR: NFR-PERFORMANCE-001, NFR-RELIABILITY-001

**Dependencies**:
- Depends on: DATA_MODELS.md, API_SPEC.md
- Blocks: DEPLOYMENT-SPEC.md, TC-INTEGRATION-001

**Business Value**: Critical
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

This document provides comprehensive data flow specifications for the Bot Core cryptocurrency trading platform. It details how data flows through the system from external sources (Binance WebSocket), through internal processing (Rust Core Engine, Python AI Service), to the frontend dashboard, and back to external services for trade execution.

### Key Data Flow Characteristics

- **Real-time Processing**: Sub-100ms latency for market data updates
- **Bidirectional Communication**: WebSocket for push updates, REST for request/response
- **Event-Driven Architecture**: Broadcast channels for internal communication
- **Caching Strategy**: 5-minute TTL for AI analysis, in-memory cache for market data
- **Error Recovery**: Automatic reconnection with exponential backoff
- **Data Transformation**: Validation, normalization, aggregation at each layer

---

## ARCH-DATA-FLOW-001: Market Data Flow

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-DATA-FLOW-001`
**Implementation**: `rust-core-engine/src/binance/websocket.rs`, `rust-core-engine/src/market_data/processor.rs`

### Description

This flow describes how real-time market data flows from Binance WebSocket streams through the Rust Core Engine to the frontend dashboard and MongoDB storage.

### Data Flow Sequence Diagram

```mermaid
sequenceDiagram
    participant Binance as Binance WebSocket
    participant BinanceWS as Binance WS Client
    participant Processor as Market Data Processor
    participant Validator as Data Validator
    participant Cache as In-Memory Cache
    participant MongoDB as MongoDB
    participant Channel as Broadcast Channel
    participant WSServer as WebSocket Server
    participant Frontend as Frontend Clients

    Note over Binance,Frontend: Initial Connection
    BinanceWS->>Binance: Connect to wss://stream.binance.com:9443/ws
    Binance-->>BinanceWS: Connection established
    BinanceWS->>Binance: Subscribe to streams (btcusdt@kline_1m, ticker, depth)
    Binance-->>BinanceWS: Subscription confirmed

    Note over Binance,Frontend: Continuous Market Data Stream
    loop Every 100ms-1s
        Binance->>BinanceWS: Stream OHLCV data (kline event)
        BinanceWS->>BinanceWS: Parse JSON message
        BinanceWS->>BinanceWS: Deserialize to KlineEvent struct
        BinanceWS->>Processor: Send StreamEvent::Kline(event)

        Processor->>Validator: Validate candlestick data
        Validator->>Validator: Check price ranges (> 0)
        Validator->>Validator: Check volume ranges (>= 0)
        Validator->>Validator: Check timestamp validity
        alt Validation fails
            Validator-->>Processor: ValidationError
            Processor->>Processor: Log error, discard data
        else Validation succeeds
            Validator-->>Processor: Valid data

            Processor->>Processor: Normalize data
            Note right of Processor: Convert string prices to f64<br/>Apply decimal precision (8 places)<br/>Ensure UTC timestamp

            Processor->>Cache: Update latest prices
            Cache->>Cache: Store in HashMap<symbol, Candlestick>
            Cache-->>Processor: Stored (latency: <1ms)

            Processor->>MongoDB: Store historical data
            Note right of MongoDB: Collection: market_data<br/>Index: {symbol: 1, timestamp: -1}<br/>TTL: 90 days
            MongoDB-->>Processor: Acknowledged (latency: 5-20ms)

            Processor->>Channel: Broadcast market update
            Note right of Channel: Message type: MarketData<br/>Payload: {symbol, price, volume, timestamp}
            Channel->>WSServer: Forward to all subscribers

            WSServer->>Frontend: Send WebSocket message
            Note right of Frontend: Message format:<br/>{<br/>  "type": "MarketData",<br/>  "symbol": "BTCUSDT",<br/>  "price": 43250.50,<br/>  "volume": 123.45,<br/>  "timestamp": 1698765432<br/>}
            Frontend->>Frontend: Update charts in real-time
            Frontend->>Frontend: Trigger UI rerender
        end
    end

    Note over Binance,Frontend: Heartbeat Mechanism
    loop Every 30 seconds
        Binance->>BinanceWS: Ping
        BinanceWS-->>Binance: Pong
    end

    Note over Binance,Frontend: Error Recovery
    alt WebSocket disconnected
        BinanceWS->>BinanceWS: Detect connection loss
        BinanceWS->>BinanceWS: Wait 2^attempt seconds (max 64s)
        BinanceWS->>Binance: Reconnect
        Binance-->>BinanceWS: Connection re-established
        BinanceWS->>Binance: Re-subscribe to streams
        BinanceWS->>Processor: Send ReconnectedEvent
        Processor->>MongoDB: Query latest timestamp
        MongoDB-->>Processor: Last known timestamp
        Processor->>Processor: Resume from last known state
    end
```

### Data Sources

**Binance WebSocket Streams**:
- **Endpoint**: `wss://stream.binance.com:9443/ws/stream?streams=`
- **Streams**:
  - Kline (Candlestick): `{symbol}@kline_{interval}` (1m, 5m, 15m, 1h, 4h, 1d)
  - 24hr Ticker: `{symbol}@ticker`
  - Order Book Depth: `{symbol}@depth@100ms`
- **Message Rate**: 100ms to 1s per symbol
- **Data Format**: JSON

### Processing Steps

**Step 1: WebSocket Message Reception**
- **Component**: `BinanceWebSocket` (rust-core-engine/src/binance/websocket.rs)
- **Input**: Raw JSON string from WebSocket
- **Output**: Parsed `StreamEvent` enum
- **Latency**: <5ms
- **Error Handling**: Log parse errors, continue processing

**Step 2: Data Validation**
- **Component**: `MarketDataProcessor::validate()` (rust-core-engine/src/market_data/processor.rs)
- **Validation Rules**:
  - Price > 0 (no negative or zero prices)
  - Volume >= 0 (allow zero volume)
  - Timestamp within 5 minutes of current time (reject stale data)
  - Symbol matches expected format (uppercase, valid pair)
- **Latency**: <1ms
- **Error Handling**: Reject invalid data, log warning, send alert if >10% rejection rate

**Step 3: Data Normalization**
- **Component**: `MarketDataProcessor::normalize()`
- **Transformations**:
  - Convert string prices to `f64`: `"43250.50" -> 43250.50`
  - Apply decimal precision: 8 places for prices, 8 places for volume
  - Ensure UTC timestamps: Convert epoch milliseconds to `chrono::DateTime<Utc>`
  - Standardize symbol casing: Always uppercase (BTCUSDT, ETHUSDT)
- **Latency**: <2ms

**Step 4: Cache Update**
- **Component**: `MarketDataCache` (rust-core-engine/src/market_data/cache.rs)
- **Data Structure**: `HashMap<String, Candlestick>`
- **Capacity**: 1000 symbols (configurable)
- **Eviction Policy**: LRU (Least Recently Used)
- **Latency**: <1ms (in-memory)
- **Thread Safety**: Arc<RwLock<HashMap>>

**Step 5: Database Storage**
- **Component**: MongoDB client
- **Collection**: `market_data`
- **Document Schema**:
  ```json
  {
    "_id": ObjectId,
    "symbol": "BTCUSDT",
    "interval": "1m",
    "open_time": 1698765400000,
    "close_time": 1698765459999,
    "open": 43250.50,
    "high": 43300.00,
    "low": 43200.00,
    "close": 43275.25,
    "volume": 123.456789,
    "quote_volume": 5342187.50,
    "trades": 1250,
    "is_closed": true,
    "created_at": ISODate("2025-10-11T10:30:00Z")
  }
  ```
- **Indexes**:
  - `{symbol: 1, open_time: -1}` (for time-series queries)
  - `{symbol: 1, interval: 1, open_time: -1}` (composite)
- **Write Latency**: 5-20ms
- **Write Concern**: `w:1` (acknowledged)
- **TTL**: 90 days (automatic deletion)

**Step 6: Broadcast to WebSocket Clients**
- **Component**: `broadcast::Sender<String>` channel
- **Capacity**: 1000 messages (configurable)
- **Message Format**:
  ```json
  {
    "type": "MarketData",
    "symbol": "BTCUSDT",
    "interval": "1m",
    "price": 43275.25,
    "volume": 123.456789,
    "timestamp": 1698765459999,
    "change_percent": 0.058
  }
  ```
- **Latency**: <5ms (channel send)
- **Backpressure**: Drop oldest messages if channel full

### Storage Strategy

**MongoDB Collections**:
- **market_data**: Raw OHLCV candlestick data (90 days retention)
- **market_data_aggregated_1h**: 1-hour aggregated data (365 days retention)
- **market_data_aggregated_1d**: 1-day aggregated data (5 years retention)

**Aggregation Pipeline** (runs every hour):
```javascript
db.market_data.aggregate([
  {
    $match: {
      interval: "1m",
      open_time: { $gte: ISODate("2025-10-11T09:00:00Z") }
    }
  },
  {
    $group: {
      _id: { symbol: "$symbol", hour: { $dateTrunc: { date: "$open_time", unit: "hour" } } },
      open: { $first: "$open" },
      high: { $max: "$high" },
      low: { $min: "$low" },
      close: { $last: "$close" },
      volume: { $sum: "$volume" },
      trades: { $sum: "$trades" }
    }
  },
  {
    $out: "market_data_aggregated_1h"
  }
])
```

### Broadcasting Mechanism

**WebSocket Pub/Sub**:
- **Implementation**: Tokio broadcast channel
- **Pattern**: One publisher (MarketDataProcessor), many subscribers (WebSocket connections)
- **Message Flow**:
  1. Processor sends to broadcast channel
  2. Channel clones message to all subscribers
  3. WebSocket handler serializes to JSON
  4. WebSocket sends to client
- **Latency**: <10ms end-to-end

### Error Handling

**WebSocket Disconnection**:
- **Detection**: No ping/pong for 60 seconds
- **Recovery**: Exponential backoff reconnection (2^attempt seconds, max 64s)
- **Max Attempts**: 10 reconnection attempts
- **State Recovery**: Query MongoDB for last known timestamp, resume from there
- **Alert**: Send notification if reconnection fails

**Data Gaps**:
- **Detection**: Missing timestamps in sequence (e.g., 10:00:00, 10:00:01, 10:00:03 - missing 10:00:02)
- **Recovery**: Backfill from Binance REST API
- **Backfill Endpoint**: `GET /api/v3/klines?symbol=BTCUSDT&interval=1m&startTime={start}&endTime={end}`
- **Backfill Limit**: Up to 1000 candles per request

**Invalid Data**:
- **Detection**: Validation fails (price <= 0, invalid timestamp)
- **Handling**: Log warning, discard data, increment error counter
- **Alert**: Send alert if error rate > 10% over 5 minutes

### Performance Characteristics

**Latency Breakdown**:
- WebSocket receive: 10-50ms (network)
- JSON parsing: 1-5ms
- Validation: <1ms
- Normalization: <2ms
- Cache update: <1ms
- MongoDB write: 5-20ms
- Broadcast: <5ms
- WebSocket send to client: 10-30ms (network)
- **Total End-to-End Latency**: 30-100ms (Target: <100ms)

**Throughput**:
- Messages per second: 1000-5000 (depends on number of symbols)
- Bandwidth: ~500 KB/s - 2 MB/s
- MongoDB writes per second: 100-500

**Resource Usage**:
- CPU: 5-10% (single core)
- Memory: 50-100 MB (in-memory cache)
- Network: 500 KB/s - 2 MB/s

### Monitoring & Metrics

**Key Metrics**:
- `market_data_messages_received_total` (counter)
- `market_data_validation_errors_total` (counter)
- `market_data_processing_latency_seconds` (histogram)
- `market_data_mongodb_write_latency_seconds` (histogram)
- `market_data_websocket_broadcast_latency_seconds` (histogram)
- `market_data_cache_hit_rate` (gauge)

**Alerts**:
- WebSocket disconnected for >60 seconds
- Validation error rate >10% over 5 minutes
- MongoDB write latency >100ms (p95) over 1 minute
- End-to-end latency >200ms (p95) over 1 minute

---

## ARCH-DATA-FLOW-002: Trading Execution Flow

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-DATA-FLOW-002`
**Implementation**: `rust-core-engine/src/trading/engine.rs`, `rust-core-engine/src/trading/risk_manager.rs`

### Description

This flow describes the complete trade execution lifecycle from user request through risk validation, exchange execution, position updates, and frontend notification.

### Data Flow Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant RustAPI as Rust Core API
    participant Auth as JWT Middleware
    participant RiskMgr as Risk Manager
    participant PosMgr as Position Manager
    participant BinanceAPI as Binance REST API
    participant MongoDB
    participant Channel as Broadcast Channel
    participant WSServer as WebSocket Server

    Note over User,WSServer: Trade Request Initiation
    User->>Frontend: Click "Execute Trade" button
    Frontend->>Frontend: Validate form inputs
    Frontend->>Frontend: Get JWT token from localStorage
    Frontend->>RustAPI: POST /api/trades/execute<br/>Authorization: Bearer {token}<br/>Body: {symbol, side, quantity, price}

    Note over RustAPI,MongoDB: Authentication Phase
    RustAPI->>Auth: Verify JWT token
    Auth->>Auth: Extract token from "Bearer {token}"
    Auth->>Auth: Decode JWT with secret key
    alt Token invalid or expired
        Auth-->>RustAPI: 401 Unauthorized
        RustAPI-->>Frontend: {success: false, error: "Invalid token"}
        Frontend->>User: Show error notification
    else Token valid
        Auth-->>RustAPI: Claims {user_id, email, is_admin}

        Note over RustAPI,MongoDB: Trade Validation Phase
        RustAPI->>RustAPI: Parse request body
        RustAPI->>RustAPI: Validate required fields
        alt Validation fails
            RustAPI-->>Frontend: 400 Bad Request
            Frontend->>User: Show validation errors
        else Validation succeeds
            RustAPI->>RiskMgr: validate_trade(trade_request)

            Note over RiskMgr,PosMgr: Risk Management Checks
            RiskMgr->>PosMgr: get_current_positions(user_id)
            PosMgr->>MongoDB: Query positions collection
            MongoDB-->>PosMgr: User positions
            PosMgr-->>RiskMgr: Current positions

            RiskMgr->>RiskMgr: Check #1: Position Size Limit
            Note right of RiskMgr: Max position size per symbol: 10% of portfolio<br/>Current: 5%, New: 3%, Total: 8% ✓

            RiskMgr->>RiskMgr: Check #2: Stop Loss Validation
            Note right of RiskMgr: Required: Yes<br/>Min distance: 2%<br/>Provided: 2.5% ✓

            RiskMgr->>RiskMgr: Check #3: Daily Loss Limit
            Note right of RiskMgr: Max daily loss: 5% of portfolio<br/>Current loss: 2%<br/>Potential: 0.5%<br/>Total: 2.5% ✓

            RiskMgr->>RiskMgr: Check #4: Leverage Limit
            Note right of RiskMgr: Max leverage: 10x<br/>Requested: 5x ✓

            RiskMgr->>RiskMgr: Check #5: Open Positions Limit
            Note right of RiskMgr: Max open positions: 10<br/>Current: 6<br/>After trade: 7 ✓

            alt Risk checks fail
                RiskMgr-->>RustAPI: RiskViolation {reason, details}
                RustAPI-->>Frontend: 400 Bad Request<br/>{success: false, error: "Risk violation", details}
                Frontend->>User: Show risk violation message
            else Risk checks pass
                RiskMgr-->>RustAPI: TradeApproved

                Note over RustAPI,BinanceAPI: Exchange Execution Phase
                RustAPI->>BinanceAPI: POST /fapi/v1/order
                Note right of BinanceAPI: Headers:<br/>X-MBX-APIKEY: {api_key}<br/>signature: HMAC-SHA256<br/><br/>Body:<br/>symbol: BTCUSDT<br/>side: BUY<br/>type: LIMIT<br/>quantity: 0.5<br/>price: 43250.50<br/>timeInForce: GTC

                alt Exchange rejects order
                    BinanceAPI-->>RustAPI: 400 Bad Request<br/>{code: -1111, msg: "Insufficient balance"}
                    RustAPI-->>Frontend: 400 Bad Request<br/>{success: false, error: "Exchange error"}
                    Frontend->>User: Show exchange error
                else Order accepted
                    BinanceAPI-->>RustAPI: 200 OK<br/>{orderId, status: "NEW", fills: []}

                    Note over RustAPI,MongoDB: Post-Execution Phase
                    RustAPI->>MongoDB: Insert trade document
                    Note right of MongoDB: Collection: trades<br/>Document: {<br/>  user_id,<br/>  order_id,<br/>  symbol,<br/>  side,<br/>  quantity,<br/>  price,<br/>  status: "NEW",<br/>  created_at<br/>}
                    MongoDB-->>RustAPI: Acknowledged {inserted_id}

                    RustAPI->>PosMgr: update_position(user_id, trade)
                    PosMgr->>PosMgr: Calculate new position
                    Note right of PosMgr: If BUY: position += quantity<br/>If SELL: position -= quantity<br/>Update avg_entry_price
                    PosMgr->>MongoDB: Update positions collection
                    MongoDB-->>PosMgr: Acknowledged
                    PosMgr-->>RustAPI: Position updated

                    Note over RustAPI,WSServer: Real-time Broadcast Phase
                    RustAPI->>Channel: Broadcast TradeExecuted event
                    Note right of Channel: Message: {<br/>  type: "TradeExecuted",<br/>  user_id,<br/>  trade_id,<br/>  symbol,<br/>  side,<br/>  quantity,<br/>  price,<br/>  timestamp<br/>}
                    Channel->>WSServer: Forward to user's connections
                    WSServer->>Frontend: Send WebSocket message
                    Frontend->>Frontend: Update trades table
                    Frontend->>Frontend: Update positions panel
                    Frontend->>Frontend: Show success notification

                    RustAPI-->>Frontend: 200 OK<br/>{success: true, data: {trade_id, order_id, status}}
                    Frontend->>User: Show "Trade executed successfully"
                end
            end
        end
    end

    Note over User,WSServer: Order Status Updates (Async)
    loop Every 5 seconds
        BinanceAPI->>RustAPI: WebSocket order update
        Note right of BinanceAPI: {orderId, status: "FILLED", executedQty}
        RustAPI->>MongoDB: Update trade status
        RustAPI->>Channel: Broadcast OrderStatusUpdate
        Channel->>WSServer: Forward to user
        WSServer->>Frontend: Update UI
        Frontend->>Frontend: Show "Order filled" notification
    end
```

### Trade Request Schema

**HTTP Request**:
```http
POST /api/trades/execute HTTP/1.1
Host: localhost:8080
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
Content-Type: application/json

{
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "LIMIT",
  "quantity": 0.5,
  "price": 43250.50,
  "stop_loss": 42250.00,
  "take_profit": 45000.00,
  "leverage": 5
}
```

**Request Validation Rules**:
- `symbol`: Required, uppercase, valid trading pair (checked against Binance symbol list)
- `side`: Required, enum ["BUY", "SELL"]
- `type`: Required, enum ["LIMIT", "MARKET", "STOP_LOSS", "TAKE_PROFIT"]
- `quantity`: Required, float > 0, respects Binance LOT_SIZE filter
- `price`: Required for LIMIT orders, float > 0, respects Binance PRICE_FILTER
- `stop_loss`: Optional, float > 0, must be 2-10% away from entry price
- `take_profit`: Optional, float > 0, must be >2% away from entry price
- `leverage`: Optional, integer 1-10, default 1

### Risk Management Validation

**Risk Check #1: Position Size Limit**
- **Rule**: Single position cannot exceed 10% of portfolio value
- **Calculation**: `(quantity * price) / portfolio_value <= 0.10`
- **Example**: Portfolio: $100,000, Max position size: $10,000, Requested: $5,000 (0.5 BTC * $43,250) = 5% ✓
- **Rejection**: Return 400 with error "Position size exceeds 10% limit"

**Risk Check #2: Stop Loss Validation**
- **Rule**: Stop loss must be 2-10% away from entry price
- **Calculation BUY**: `(entry_price - stop_loss) / entry_price` between 0.02 and 0.10
- **Calculation SELL**: `(stop_loss - entry_price) / entry_price` between 0.02 and 0.10
- **Example**: Entry: $43,250, Stop Loss: $42,250, Distance: 2.3% ✓
- **Rejection**: Return 400 with error "Stop loss must be 2-10% away from entry"

**Risk Check #3: Daily Loss Limit**
- **Rule**: Total realized + unrealized losses cannot exceed 5% of portfolio in 24 hours
- **Calculation**: Query all trades in last 24h, sum PnL, check if < -5% of portfolio
- **Example**: Portfolio: $100,000, Max loss: $5,000, Current loss: $2,000, Potential: $500, Total: $2,500 (2.5%) ✓
- **Rejection**: Return 400 with error "Daily loss limit reached (5%)"

**Risk Check #4: Leverage Limit**
- **Rule**: Maximum leverage 10x for all positions
- **Calculation**: Check requested leverage <= 10
- **Example**: Requested: 5x, Max: 10x ✓
- **Rejection**: Return 400 with error "Leverage exceeds maximum of 10x"

**Risk Check #5: Open Positions Limit**
- **Rule**: Maximum 10 open positions across all symbols
- **Calculation**: Count positions with quantity > 0
- **Example**: Current: 6, Max: 10, After trade: 7 ✓
- **Rejection**: Return 400 with error "Maximum 10 open positions allowed"

### Binance API Integration

**Order Submission**:
- **Endpoint**: `POST /fapi/v1/order`
- **Authentication**: HMAC SHA256 signature
- **Required Headers**:
  - `X-MBX-APIKEY`: User's Binance API key (encrypted in database)
  - `signature`: HMAC SHA256 of query string
  - `timestamp`: Current timestamp in milliseconds
- **Request Body**:
  ```json
  {
    "symbol": "BTCUSDT",
    "side": "BUY",
    "type": "LIMIT",
    "quantity": 0.5,
    "price": 43250.50,
    "timeInForce": "GTC",
    "timestamp": 1698765432000,
    "signature": "a1b2c3d4..."
  }
  ```
- **Response** (Success):
  ```json
  {
    "orderId": 123456789,
    "symbol": "BTCUSDT",
    "status": "NEW",
    "clientOrderId": "rust_core_1698765432",
    "price": "43250.50",
    "avgPrice": "0.00",
    "origQty": "0.5",
    "executedQty": "0",
    "cumQuote": "0",
    "timeInForce": "GTC",
    "type": "LIMIT",
    "side": "BUY",
    "stopPrice": "0",
    "transactTime": 1698765432500,
    "updateTime": 1698765432500
  }
  ```
- **Response** (Error):
  ```json
  {
    "code": -1111,
    "msg": "Precision is over the maximum defined for this asset."
  }
  ```

**Common Error Codes**:
- `-1111`: Precision error (quantity or price has too many decimals)
- `-2010`: Insufficient balance
- `-2019`: Margin insufficient
- `-1021`: Timestamp outside recv window (clock sync issue)
- `-1022`: Invalid signature

### Position Management

**Position Update Logic**:
```rust
// Pseudocode for position update
fn update_position(user_id: &str, trade: &Trade) -> Result<Position> {
    let mut position = get_position(user_id, trade.symbol)?;

    match trade.side {
        Side::BUY => {
            // Increase position
            let new_qty = position.quantity + trade.quantity;
            let new_avg_price = (
                (position.quantity * position.avg_entry_price) +
                (trade.quantity * trade.price)
            ) / new_qty;

            position.quantity = new_qty;
            position.avg_entry_price = new_avg_price;
        },
        Side::SELL => {
            // Decrease position (or reverse if selling more than held)
            let new_qty = position.quantity - trade.quantity;

            if new_qty >= 0.0 {
                // Normal sell
                position.quantity = new_qty;
                // avg_entry_price stays same
            } else {
                // Position reversal (went short)
                position.quantity = new_qty.abs();
                position.avg_entry_price = trade.price;
            }
        }
    }

    // Calculate unrealized PnL
    let current_price = get_current_price(trade.symbol)?;
    position.unrealized_pnl = (current_price - position.avg_entry_price) * position.quantity;
    position.updated_at = Utc::now();

    save_position(position)
}
```

**Position Document Schema**:
```json
{
  "_id": ObjectId,
  "user_id": ObjectId,
  "symbol": "BTCUSDT",
  "quantity": 0.5,
  "avg_entry_price": 43250.50,
  "leverage": 5,
  "margin": 4325.05,
  "unrealized_pnl": 125.00,
  "realized_pnl": 0.00,
  "stop_loss": 42250.00,
  "take_profit": 45000.00,
  "liquidation_price": 40500.00,
  "created_at": ISODate("2025-10-11T10:30:00Z"),
  "updated_at": ISODate("2025-10-11T10:30:05Z")
}
```

### Broadcasting Mechanism

**Event Types**:
- `TradeExecuted`: Sent when trade is submitted to exchange
- `OrderStatusUpdate`: Sent when order status changes (NEW → PARTIALLY_FILLED → FILLED)
- `PositionUpdate`: Sent when position is updated
- `RiskAlert`: Sent when approaching risk limits

**Message Format**:
```json
{
  "type": "TradeExecuted",
  "user_id": "60f7b3b3e4b0c8f7d8f7b3b3",
  "trade_id": "trade_1698765432",
  "order_id": 123456789,
  "symbol": "BTCUSDT",
  "side": "BUY",
  "quantity": 0.5,
  "price": 43250.50,
  "status": "NEW",
  "timestamp": 1698765432000
}
```

### Error Handling

**Network Errors**:
- **Timeout**: Retry up to 3 times with exponential backoff (1s, 2s, 4s)
- **Connection refused**: Check Binance API status, retry after 5s
- **DNS resolution failure**: Alert admin, fallback to IP address

**Exchange Errors**:
- **Insufficient balance**: Return to user, suggest reducing quantity
- **Invalid order**: Log details, return validation error to user
- **Rate limit exceeded**: Implement exponential backoff, queue order

**Database Errors**:
- **Write failure**: Retry write, if fails mark order as "PENDING_CONFIRMATION"
- **Connection loss**: Use connection pool reconnection logic
- **Timeout**: Increase timeout for critical operations

### Performance Characteristics

**Latency Breakdown**:
- JWT validation: 1-5ms
- Risk checks: 5-15ms (includes DB queries)
- Binance API call: 50-200ms (network + exchange processing)
- MongoDB write (trade): 5-20ms
- MongoDB write (position): 5-20ms
- Broadcast: 1-5ms
- **Total End-to-End Latency**: 70-265ms (Target: <500ms)

**Throughput**:
- Trades per second: 50-100 (limited by Binance API rate limits)
- Rate limit: 1200 orders per minute per API key (Binance limit)
- Concurrent requests: 100 (limited by server resources)

**Resource Usage**:
- CPU: 10-20% during active trading
- Memory: 100-200 MB
- Network: 10-50 KB/s

### Monitoring & Metrics

**Key Metrics**:
- `trades_executed_total` (counter, labels: symbol, side, status)
- `trades_execution_latency_seconds` (histogram)
- `risk_checks_failed_total` (counter, labels: rule)
- `binance_api_errors_total` (counter, labels: error_code)
- `position_updates_total` (counter, labels: symbol)

**Alerts**:
- Trade execution latency >1s (p95) over 1 minute
- Risk check failure rate >5% over 5 minutes
- Binance API error rate >1% over 1 minute
- Position update failure rate >0.1% over 1 minute

---

## ARCH-DATA-FLOW-003: AI Analysis Flow

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-DATA-FLOW-003`
**Implementation**: `python-ai-service/main.py`, `python-ai-service/services/ai_analysis.py`

### Description

This flow describes how AI analysis is triggered, processed through machine learning models and OpenAI GPT-4, cached, and delivered to the Rust Core Engine and frontend.

### Data Flow Sequence Diagram

```mermaid
sequenceDiagram
    participant Trigger as Periodic Trigger (5min)
    participant PythonAPI as Python AI Service
    participant Cache as MongoDB Cache
    participant MarketData as Market Data Query
    participant Indicators as Technical Indicators
    participant LSTM as LSTM Model
    participant GPT4 as OpenAI GPT-4
    participant RustAPI as Rust Core Engine
    participant Channel as Broadcast Channel
    participant WSServer as WebSocket Server
    participant Frontend

    Note over Trigger,Frontend: Periodic AI Analysis Trigger
    Trigger->>PythonAPI: Run analysis for BTCUSDT (every 5 minutes)

    Note over PythonAPI,Cache: Cache Check Phase
    PythonAPI->>Cache: Query cached analysis
    Note right of Cache: Collection: ai_analysis_cache<br/>TTL: 5 minutes<br/>Key: {symbol, interval}

    alt Cache hit (data < 5 min old)
        Cache-->>PythonAPI: Cached analysis result
        Note right of PythonAPI: Latency: 2-10ms<br/>Skip model inference
        PythonAPI->>RustAPI: Return cached result
        RustAPI->>Channel: Broadcast AISignalReceived (cached)
        Channel->>WSServer: Forward to all clients
        WSServer->>Frontend: Update AI signals (latency: 50ms total)
    else Cache miss or expired
        Cache-->>PythonAPI: null (no cached data)

        Note over PythonAPI,MarketData: Data Gathering Phase
        PythonAPI->>MarketData: Fetch last 100 candles
        Note right of MarketData: Query: db.market_data.find({<br/>  symbol: "BTCUSDT",<br/>  interval: "1m"<br/>}).sort({open_time: -1}).limit(100)
        MarketData-->>PythonAPI: 100 candlesticks (2-20ms)

        PythonAPI->>PythonAPI: Validate data completeness
        Note right of PythonAPI: Check: All 100 candles present<br/>No gaps in timestamps<br/>All prices > 0

        alt Data incomplete or invalid
            PythonAPI->>MarketData: Backfill missing data
            MarketData-->>PythonAPI: Complete dataset
        end

        Note over PythonAPI,Indicators: Technical Indicators Calculation
        PythonAPI->>Indicators: Calculate RSI(14)
        Indicators->>Indicators: Compute rolling average of gains/losses
        Indicators-->>PythonAPI: RSI value: 67.5 (latency: 5-10ms)

        PythonAPI->>Indicators: Calculate MACD(12, 26, 9)
        Indicators->>Indicators: Compute EMA12, EMA26, signal line
        Indicators-->>PythonAPI: MACD: {macd: 125, signal: 100, histogram: 25}

        PythonAPI->>Indicators: Calculate Bollinger Bands(20, 2)
        Indicators->>Indicators: Compute SMA20, upper/lower bands
        Indicators-->>PythonAPI: BB: {upper: 43500, middle: 43250, lower: 43000}

        PythonAPI->>Indicators: Calculate Volume indicators
        Indicators-->>PythonAPI: Volume MA, Volume ratio

        PythonAPI->>PythonAPI: Prepare feature vector
        Note right of PythonAPI: Features: [<br/>  normalized_prices (100),<br/>  RSI (1),<br/>  MACD (3),<br/>  BB_distance (1),<br/>  volume_ratio (1)<br/>] = 106 features

        Note over PythonAPI,LSTM: Machine Learning Inference Phase
        PythonAPI->>LSTM: Predict price direction
        Note right of LSTM: Model: LSTM<br/>Input shape: (1, 100, 106)<br/>Output: [buy_prob, sell_prob, hold_prob]

        LSTM->>LSTM: Forward pass through network
        Note right of LSTM: Layer 1: LSTM(256 units)<br/>Layer 2: LSTM(128 units)<br/>Layer 3: Dense(64 units)<br/>Layer 4: Dense(3 units, softmax)

        LSTM-->>PythonAPI: Prediction: [0.82, 0.15, 0.03]
        Note right of PythonAPI: Interpretation:<br/>Buy: 82% confidence<br/>Sell: 15% confidence<br/>Hold: 3% confidence<br/><br/>Decision: BUY (highest probability)

        PythonAPI->>PythonAPI: Calculate confidence score
        Note right of PythonAPI: Confidence = max(probabilities) = 0.82<br/>Threshold: 0.65 (configurable)<br/>0.82 > 0.65 ✓ Signal is strong

        Note over PythonAPI,GPT4: AI Reasoning Phase
        PythonAPI->>GPT4: Analyze market sentiment
        Note right of GPT4: Prompt:<br/>"Analyze the following market data for BTCUSDT:<br/>- Current price: $43,250<br/>- RSI: 67.5 (approaching overbought)<br/>- MACD: Bullish crossover<br/>- Bollinger Bands: Price near upper band<br/>- Volume: Increasing<br/><br/>Provide a brief analysis and trading recommendation."

        GPT4->>GPT4: Process prompt (GPT-4 Turbo)
        Note right of GPT4: Model: gpt-4-turbo-preview<br/>Max tokens: 500<br/>Temperature: 0.3

        GPT4-->>PythonAPI: AI Response (latency: 1-3s)
        Note right of PythonAPI: Response:<br/>"The market shows strong bullish momentum with<br/>MACD crossover and increasing volume. However,<br/>RSI at 67.5 suggests caution as it approaches<br/>overbought territory. Consider entering with<br/>tight stop-loss around $42,500. Target: $45,000."

        Note over PythonAPI,Cache: Result Aggregation & Caching
        PythonAPI->>PythonAPI: Combine ML prediction + GPT-4 analysis
        Note right of PythonAPI: Final signal: {<br/>  symbol: "BTCUSDT",<br/>  direction: "BUY",<br/>  confidence: 0.82,<br/>  ml_prediction: [0.82, 0.15, 0.03],<br/>  indicators: {RSI: 67.5, MACD: ...},<br/>  ai_reasoning: "The market shows...",<br/>  timestamp: 1698765432,<br/>  expires_at: 1698765732 (5 min later)<br/>}

        PythonAPI->>Cache: Store analysis result
        Note right of Cache: Collection: ai_analysis_cache<br/>Index: {symbol: 1, timestamp: 1}<br/>TTL: 300 seconds (5 minutes)
        Cache-->>PythonAPI: Stored successfully

        Note over PythonAPI,Frontend: Distribution Phase
        PythonAPI->>RustAPI: POST /api/ai/signal (internal endpoint)
        Note right of RustAPI: Authentication: Internal API key<br/>Payload: AI signal object

        RustAPI->>RustAPI: Validate AI signal
        RustAPI->>RustAPI: Store in MongoDB (ai_signals collection)
        RustAPI->>Channel: Broadcast AISignalReceived event

        Channel->>WSServer: Forward to all subscribed clients
        WSServer->>Frontend: Send WebSocket message
        Note right of Frontend: Message: {<br/>  type: "AISignalReceived",<br/>  symbol: "BTCUSDT",<br/>  direction: "BUY",<br/>  confidence: 0.82,<br/>  reasoning: "...",<br/>  timestamp: 1698765432<br/>}

        Frontend->>Frontend: Update AI signals panel
        Frontend->>Frontend: Show notification (if confidence > 0.75)
        Frontend->>Frontend: Highlight recommended action

        PythonAPI-->>Trigger: Analysis complete (latency: 2-5s)
    end

    Note over Trigger,Frontend: Continuous Analysis Loop
    loop Every 5 minutes
        Trigger->>PythonAPI: Trigger analysis for all symbols
        Note right of Trigger: Symbols: BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT
    end
```

### Data Gathering

**Market Data Query**:
- **Collection**: `market_data`
- **Query**:
  ```javascript
  db.market_data.find({
    symbol: "BTCUSDT",
    interval: "1m",
    open_time: { $gte: current_time - 100_minutes }
  }).sort({ open_time: -1 }).limit(100)
  ```
- **Latency**: 2-20ms (depends on index usage)
- **Fallback**: If < 100 candles, backfill from Binance REST API

**Data Validation**:
- Check for gaps in timestamps (missing candles)
- Validate price ranges (no zeros or negatives)
- Ensure chronological order
- Verify volume data integrity

### Technical Indicators Calculation

**Implemented Indicators**:

**1. RSI (Relative Strength Index)**
- **Period**: 14
- **Calculation**:
  ```python
  def calculate_rsi(prices, period=14):
      deltas = np.diff(prices)
      gains = np.where(deltas > 0, deltas, 0)
      losses = np.where(deltas < 0, -deltas, 0)

      avg_gain = np.mean(gains[:period])
      avg_loss = np.mean(losses[:period])

      rs = avg_gain / avg_loss if avg_loss != 0 else 0
      rsi = 100 - (100 / (1 + rs))
      return rsi
  ```
- **Interpretation**:
  - RSI > 70: Overbought (potential sell signal)
  - RSI < 30: Oversold (potential buy signal)
  - RSI 40-60: Neutral

**2. MACD (Moving Average Convergence Divergence)**
- **Parameters**: (12, 26, 9)
- **Calculation**:
  ```python
  def calculate_macd(prices, fast=12, slow=26, signal=9):
      ema_fast = prices.ewm(span=fast).mean()
      ema_slow = prices.ewm(span=slow).mean()
      macd_line = ema_fast - ema_slow
      signal_line = macd_line.ewm(span=signal).mean()
      histogram = macd_line - signal_line
      return macd_line, signal_line, histogram
  ```
- **Interpretation**:
  - MACD > Signal: Bullish (buy signal)
  - MACD < Signal: Bearish (sell signal)
  - Histogram increasing: Momentum building

**3. Bollinger Bands**
- **Parameters**: (20, 2) - 20 period SMA, 2 standard deviations
- **Calculation**:
  ```python
  def calculate_bollinger_bands(prices, period=20, std_dev=2):
      sma = prices.rolling(window=period).mean()
      std = prices.rolling(window=period).std()
      upper_band = sma + (std * std_dev)
      lower_band = sma - (std * std_dev)
      return upper_band, sma, lower_band
  ```
- **Interpretation**:
  - Price near upper band: Overbought
  - Price near lower band: Oversold
  - Bands widening: Increased volatility

**4. Volume Analysis**
- **Volume MA**: 20-period moving average of volume
- **Volume Ratio**: Current volume / Volume MA
- **Interpretation**:
  - Ratio > 1.5: High volume (strong signal)
  - Ratio < 0.5: Low volume (weak signal)

### Machine Learning Inference

**LSTM Model Architecture**:
```python
class LSTMPricePredictor(nn.Module):
    def __init__(self, input_size=106, hidden_size_1=256, hidden_size_2=128, output_size=3):
        super().__init__()
        self.lstm1 = nn.LSTM(input_size, hidden_size_1, batch_first=True)
        self.dropout1 = nn.Dropout(0.2)
        self.lstm2 = nn.LSTM(hidden_size_1, hidden_size_2, batch_first=True)
        self.dropout2 = nn.Dropout(0.2)
        self.fc1 = nn.Linear(hidden_size_2, 64)
        self.relu = nn.ReLU()
        self.fc2 = nn.Linear(64, output_size)
        self.softmax = nn.Softmax(dim=1)

    def forward(self, x):
        # x shape: (batch_size, sequence_length, features)
        lstm_out, _ = self.lstm1(x)
        lstm_out = self.dropout1(lstm_out)
        lstm_out, _ = self.lstm2(lstm_out)
        lstm_out = self.dropout2(lstm_out)
        # Take last time step output
        lstm_out = lstm_out[:, -1, :]
        fc_out = self.fc1(lstm_out)
        fc_out = self.relu(fc_out)
        fc_out = self.fc2(fc_out)
        output = self.softmax(fc_out)
        return output
```

**Input Features** (106 total):
- Normalized prices: 100 (last 100 candles)
- RSI: 1
- MACD: 3 (MACD line, signal line, histogram)
- Bollinger Bands: 1 (distance from middle band)
- Volume ratio: 1

**Output**:
- 3-element array: [buy_probability, sell_probability, hold_probability]
- Example: [0.82, 0.15, 0.03]
- Sum = 1.0 (softmax output)

**Model Training**:
- **Dataset**: Historical BTCUSDT data (2020-2025)
- **Training samples**: 1,000,000 sequences
- **Validation split**: 20%
- **Optimizer**: Adam (lr=0.001)
- **Loss function**: CrossEntropyLoss
- **Accuracy**: 78% on validation set
- **Model file**: `models/lstm_btcusdt_v1.0.pth`

**Confidence Thresholding**:
- **High confidence**: probability > 0.75 (strong signal)
- **Medium confidence**: 0.65 < probability <= 0.75 (moderate signal)
- **Low confidence**: 0.50 < probability <= 0.65 (weak signal)
- **No signal**: probability <= 0.50 (ignore)

### OpenAI GPT-4 Integration

**API Call**:
```python
async def get_ai_reasoning(market_data: dict) -> str:
    prompt = f"""Analyze the following cryptocurrency market data for {market_data['symbol']}:

Current Price: ${market_data['current_price']}
RSI (14): {market_data['rsi']} {'(Overbought)' if market_data['rsi'] > 70 else '(Oversold)' if market_data['rsi'] < 30 else ''}
MACD: {market_data['macd']['macd']} (Signal: {market_data['macd']['signal']}, Histogram: {market_data['macd']['histogram']})
Bollinger Bands: Upper ${market_data['bb']['upper']}, Middle ${market_data['bb']['middle']}, Lower ${market_data['bb']['lower']}
Volume Ratio: {market_data['volume_ratio']} ({'High' if market_data['volume_ratio'] > 1.5 else 'Normal' if market_data['volume_ratio'] > 0.8 else 'Low'})
ML Prediction: {market_data['ml_prediction']} with {market_data['confidence']*100:.1f}% confidence

Provide a brief (3-4 sentences) market analysis and trading recommendation. Focus on key insights."""

    response = await openai.ChatCompletion.acreate(
        model="gpt-4-turbo-preview",
        messages=[
            {"role": "system", "content": "You are an expert cryptocurrency trading analyst. Provide concise, actionable insights."},
            {"role": "user", "content": prompt}
        ],
        max_tokens=500,
        temperature=0.3  # Lower temperature for more focused responses
    )

    return response.choices[0].message.content
```

**Example Response**:
```
The market shows strong bullish momentum with a positive MACD crossover (MACD 125 > Signal 100) and increasing volume (ratio: 1.8). However, RSI at 67.5 suggests caution as it approaches overbought territory. The ML model's 82% confidence in a BUY signal aligns with technical indicators. Consider entering a long position with tight stop-loss around $42,500 (near lower Bollinger Band) and target $45,000. Monitor for RSI crossing above 70, which could signal a reversal.
```

**Rate Limiting**:
- **OpenAI Tier**: Tier 2 (50,000 TPM - Tokens Per Minute)
- **Requests per minute**: ~20 (assuming 2,500 tokens per request)
- **Backoff strategy**: Exponential backoff if rate limit exceeded
- **Fallback**: Return "AI analysis temporarily unavailable" if OpenAI fails

**Cost Management**:
- **Model**: GPT-4 Turbo ($0.01 per 1K input tokens, $0.03 per 1K output tokens)
- **Average request**: ~1,000 input tokens, ~200 output tokens
- **Cost per request**: ~$0.016
- **Monthly cost** (4 symbols, 5-min interval): ~$2,300
- **Optimization**: Cache results for 5 minutes, reducing costs by 80%

### Caching Strategy

**Cache Implementation**:
- **Storage**: MongoDB collection `ai_analysis_cache`
- **TTL**: 5 minutes (300 seconds)
- **Index**: `{symbol: 1, timestamp: 1}`, TTL index on `expires_at` field
- **Key structure**: `{symbol: "BTCUSDT", interval: "5m"}`

**Cache Document Schema**:
```json
{
  "_id": ObjectId,
  "symbol": "BTCUSDT",
  "interval": "5m",
  "analysis": {
    "direction": "BUY",
    "confidence": 0.82,
    "ml_prediction": [0.82, 0.15, 0.03],
    "indicators": {
      "rsi": 67.5,
      "macd": {"macd": 125, "signal": 100, "histogram": 25},
      "bollinger_bands": {"upper": 43500, "middle": 43250, "lower": 43000},
      "volume_ratio": 1.8
    },
    "ai_reasoning": "The market shows strong bullish momentum...",
    "model_version": "lstm_v1.0"
  },
  "timestamp": ISODate("2025-10-11T10:30:00Z"),
  "expires_at": ISODate("2025-10-11T10:35:00Z")
}
```

**Cache Hit Rate**:
- **Target**: >80% (most requests hit cache)
- **Actual**: ~85% (with 5-minute analysis interval)
- **Benefit**: Reduces latency from 2-5s to 2-10ms (250x improvement)

### Distribution to Rust Core Engine

**Internal API Call**:
```http
POST /api/ai/signal HTTP/1.1
Host: localhost:8080
Authorization: Bearer {internal_api_key}
Content-Type: application/json

{
  "symbol": "BTCUSDT",
  "direction": "BUY",
  "confidence": 0.82,
  "ml_prediction": [0.82, 0.15, 0.03],
  "indicators": {
    "rsi": 67.5,
    "macd": {"macd": 125, "signal": 100, "histogram": 25},
    "bollinger_bands": {"upper": 43500, "middle": 43250, "lower": 43000},
    "volume_ratio": 1.8
  },
  "ai_reasoning": "The market shows strong bullish momentum...",
  "timestamp": 1698765432,
  "model_version": "lstm_v1.0"
}
```

**Rust Core Engine Processing**:
1. Validate internal API key
2. Validate signal schema
3. Store in MongoDB `ai_signals` collection
4. Broadcast to WebSocket clients
5. Optionally trigger auto-trading (if enabled)

### Error Handling

**Model Inference Errors**:
- **Missing model file**: Return error, send alert to admin
- **Input shape mismatch**: Log error, skip inference, return cached or default signal
- **GPU out of memory**: Fallback to CPU inference (slower but functional)

**OpenAI API Errors**:
- **Rate limit exceeded**: Use exponential backoff, retry after delay
- **API timeout**: Retry once, if fails return partial signal (ML only, no GPT-4 reasoning)
- **Invalid API key**: Alert admin, disable GPT-4 integration
- **Content filter triggered**: Log prompt, adjust prompt template

**Data Issues**:
- **Insufficient candles**: Return error "Insufficient data for analysis (need 100 candles)"
- **Data gaps**: Backfill from Binance API, if backfill fails return "Incomplete data"
- **Invalid prices**: Skip corrupted candles, proceed if >=80 valid candles

### Performance Characteristics

**Latency Breakdown**:
- Cache check: 2-10ms
- Market data query: 2-20ms
- Indicators calculation: 5-10ms
- LSTM inference: 50-200ms (GPU), 500-1000ms (CPU)
- GPT-4 API call: 1000-3000ms
- Cache storage: 5-10ms
- Rust API call: 10-30ms
- Broadcast: 1-5ms
- **Total (cache hit)**: 2-10ms
- **Total (cache miss)**: 1070-3285ms (Target: <5000ms)

**Throughput**:
- Analyses per minute: 60 (limited by 5-min interval)
- Symbols analyzed: 4 (BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT)
- Total analyses per hour: 48 (4 symbols * 12 intervals)

**Resource Usage**:
- CPU (inference): 20-40% (during inference, spikes to 80%)
- GPU memory: 2-4 GB (if using GPU)
- RAM: 1-2 GB (model weights + data)
- Network: 5-10 KB/s (OpenAI API calls)

### Monitoring & Metrics

**Key Metrics**:
- `ai_analysis_requests_total` (counter, labels: symbol, cache_hit)
- `ai_analysis_latency_seconds` (histogram, labels: symbol, cache_hit)
- `ai_ml_inference_latency_seconds` (histogram)
- `ai_gpt4_api_latency_seconds` (histogram)
- `ai_cache_hit_rate` (gauge)
- `ai_confidence_score` (histogram, labels: symbol)

**Alerts**:
- Analysis latency >10s over 5 minutes
- Cache hit rate <70% over 15 minutes
- GPT-4 API error rate >5% over 5 minutes
- ML inference failure rate >1% over 5 minutes

---

## ARCH-DATA-FLOW-004: Authentication Flow

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-DATA-FLOW-004`
**Implementation**: `rust-core-engine/src/auth/handlers.rs`, `rust-core-engine/src/auth/jwt.rs`

### Description

This flow describes the complete user authentication lifecycle including registration, login, token generation, token validation, and session management.

### Data Flow Sequence Diagram

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant RustAuth as Rust Auth Handler
    participant Validator as Input Validator
    participant PasswordSvc as Password Service
    participant MongoDB
    participant JWTSvc as JWT Service

    Note over User,JWTSvc: User Registration Flow
    User->>Frontend: Fill registration form
    Note right of User: Fields:<br/>- Email: user@example.com<br/>- Password: SecurePass123!<br/>- Full Name: John Doe (optional)

    Frontend->>Frontend: Client-side validation
    Note right of Frontend: - Email format check<br/>- Password length >= 8<br/>- Password complexity

    Frontend->>RustAuth: POST /api/auth/register
    Note right of Frontend: Body: {<br/>  email: "user@example.com",<br/>  password: "SecurePass123!",<br/>  full_name: "John Doe"<br/>}

    RustAuth->>Validator: Validate request
    Validator->>Validator: Check email format (regex)
    Validator->>Validator: Check password length (>=8)
    Validator->>Validator: Check password complexity
    Note right of Validator: Requirements:<br/>- Min 8 characters<br/>- At least 1 uppercase<br/>- At least 1 lowercase<br/>- At least 1 number<br/>- At least 1 special char

    alt Validation fails
        Validator-->>RustAuth: ValidationError
        RustAuth-->>Frontend: 400 Bad Request<br/>{success: false, error: "Validation failed", details}
        Frontend->>User: Show validation errors
    else Validation succeeds
        Validator-->>RustAuth: Valid

        RustAuth->>MongoDB: Check if email exists
        MongoDB->>MongoDB: Query users collection
        Note right of MongoDB: db.users.findOne({email: "user@example.com"})

        alt Email already exists
            MongoDB-->>RustAuth: User found
            RustAuth-->>Frontend: 409 Conflict<br/>{success: false, error: "Email already registered"}
            Frontend->>User: Show "Email already exists"
        else Email available
            MongoDB-->>RustAuth: null (not found)

            RustAuth->>PasswordSvc: hash_password("SecurePass123!")
            PasswordSvc->>PasswordSvc: Generate salt (random 16 bytes)
            PasswordSvc->>PasswordSvc: Hash with bcrypt (cost factor 12)
            Note right of PasswordSvc: Algorithm: bcrypt<br/>Cost: 12 (2^12 = 4096 iterations)<br/>Time: 100-200ms
            PasswordSvc-->>RustAuth: Hashed password
            Note right of RustAuth: "$2b$12$abcd...xyz" (60 chars)

            RustAuth->>RustAuth: Create user object
            Note right of RustAuth: {<br/>  email: "user@example.com",<br/>  password_hash: "$2b$12$...",<br/>  full_name: "John Doe",<br/>  is_admin: false,<br/>  is_active: true,<br/>  created_at: UTC::now(),<br/>  last_login: null<br/>}

            RustAuth->>MongoDB: Insert user document
            MongoDB-->>RustAuth: Inserted {user_id}

            RustAuth->>JWTSvc: generate_token(user_id, email, is_admin)
            JWTSvc->>JWTSvc: Create claims
            Note right of JWTSvc: {<br/>  sub: "user_id_123",<br/>  email: "user@example.com",<br/>  is_admin: false,<br/>  iat: 1698765432,<br/>  exp: 1699370232 (7 days later)<br/>}

            JWTSvc->>JWTSvc: Sign with HMAC-SHA256
            Note right of JWTSvc: Secret: ENV.JWT_SECRET<br/>Algorithm: HS256
            JWTSvc-->>RustAuth: JWT token
            Note right of RustAuth: "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..."

            RustAuth-->>Frontend: 201 Created<br/>{success: true, data: {token, user}}
            Frontend->>Frontend: Store token in localStorage
            Frontend->>User: Redirect to dashboard
        end
    end

    Note over User,JWTSvc: User Login Flow
    User->>Frontend: Enter credentials
    Note right of User: Email: user@example.com<br/>Password: SecurePass123!

    Frontend->>Frontend: Client-side validation
    Frontend->>RustAuth: POST /api/auth/login
    Note right of Frontend: Body: {<br/>  email: "user@example.com",<br/>  password: "SecurePass123!"<br/>}

    RustAuth->>Validator: Validate request
    alt Validation fails
        Validator-->>RustAuth: ValidationError
        RustAuth-->>Frontend: 400 Bad Request
        Frontend->>User: Show validation errors
    else Validation succeeds
        Validator-->>RustAuth: Valid

        RustAuth->>MongoDB: Find user by email
        MongoDB->>MongoDB: Query users collection

        alt User not found
            MongoDB-->>RustAuth: null
            RustAuth-->>Frontend: 401 Unauthorized<br/>{success: false, error: "Invalid email or password"}
            Note right of RustAuth: Generic message to prevent<br/>email enumeration
            Frontend->>User: Show "Invalid credentials"
        else User found
            MongoDB-->>RustAuth: User document

            RustAuth->>RustAuth: Check if user is active
            alt User deactivated
                RustAuth-->>Frontend: 403 Forbidden<br/>{success: false, error: "Account deactivated"}
                Frontend->>User: Show "Account is deactivated"
            else User active
                RustAuth->>PasswordSvc: verify_password(password, hash)
                PasswordSvc->>PasswordSvc: Hash input with stored salt
                PasswordSvc->>PasswordSvc: Compare hashes (constant-time)
                Note right of PasswordSvc: Timing: 100-200ms<br/>Prevents timing attacks

                alt Password incorrect
                    PasswordSvc-->>RustAuth: false
                    RustAuth-->>Frontend: 401 Unauthorized<br/>{success: false, error: "Invalid email or password"}
                    Frontend->>User: Show "Invalid credentials"
                else Password correct
                    PasswordSvc-->>RustAuth: true

                    RustAuth->>MongoDB: Update last_login timestamp
                    MongoDB-->>RustAuth: Updated

                    RustAuth->>JWTSvc: generate_token(user_id, email, is_admin)
                    JWTSvc-->>RustAuth: JWT token

                    RustAuth-->>Frontend: 200 OK<br/>{success: true, data: {token, user}}
                    Frontend->>Frontend: Store token in localStorage
                    Frontend->>User: Redirect to dashboard
                end
            end
        end
    end

    Note over User,JWTSvc: Token Verification Flow (Every API Request)
    User->>Frontend: Request protected resource
    Frontend->>Frontend: Get token from localStorage
    Frontend->>RustAuth: GET /api/portfolio<br/>Authorization: Bearer {token}

    RustAuth->>JWTSvc: verify_token(token)
    JWTSvc->>JWTSvc: Decode JWT header
    JWTSvc->>JWTSvc: Verify signature (HMAC-SHA256)
    Note right of JWTSvc: Compare computed signature<br/>with token signature

    alt Signature invalid
        JWTSvc-->>RustAuth: Error: Invalid signature
        RustAuth-->>Frontend: 401 Unauthorized<br/>{success: false, error: "Invalid token"}
        Frontend->>Frontend: Clear token
        Frontend->>User: Redirect to login
    else Signature valid
        JWTSvc->>JWTSvc: Decode claims
        JWTSvc->>JWTSvc: Check expiration

        alt Token expired
            JWTSvc-->>RustAuth: Error: Token expired
            RustAuth-->>Frontend: 401 Unauthorized<br/>{success: false, error: "Token expired"}
            Frontend->>Frontend: Clear token
            Frontend->>User: Redirect to login
        else Token valid
            JWTSvc-->>RustAuth: Claims {user_id, email, is_admin}
            RustAuth->>MongoDB: Fetch user profile
            MongoDB-->>RustAuth: User data
            RustAuth-->>Frontend: 200 OK {data: portfolio}
            Frontend->>User: Display portfolio
        end
    end
```

### JWT Token Structure

**Header**:
```json
{
  "alg": "HS256",
  "typ": "JWT"
}
```

**Payload (Claims)**:
```json
{
  "sub": "60f7b3b3e4b0c8f7d8f7b3b3",
  "email": "user@example.com",
  "is_admin": false,
  "iat": 1698765432,
  "exp": 1699370232
}
```

**Signature**:
```
HMACSHA256(
  base64UrlEncode(header) + "." + base64UrlEncode(payload),
  secret
)
```

**Complete Token**:
```
eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiI2MGY3YjNiM2U0YjBjOGY3ZDhmN2IzYjMiLCJlbWFpbCI6InVzZXJAZXhhbXBsZS5jb20iLCJpc19hZG1pbiI6ZmFsc2UsImlhdCI6MTY5ODc2NTQzMiwiZXhwIjoxNjk5MzcwMjMyfQ.5J8n5F7qF7kZ8F8q5F8n5F7qF7kZ8F8q5F8n5F7qF7k
```

### Password Security

**Hashing Algorithm**: bcrypt
**Cost Factor**: 12 (2^12 = 4096 iterations)
**Salt**: Random 16 bytes (generated per password)
**Hash Format**: `$2b$12${salt}{hash}` (60 characters)

**Password Requirements**:
- Minimum length: 8 characters
- Must contain:
  - At least 1 uppercase letter (A-Z)
  - At least 1 lowercase letter (a-z)
  - At least 1 digit (0-9)
  - At least 1 special character (!@#$%^&*()_+)

**Password Verification**:
- Constant-time comparison (prevents timing attacks)
- Re-hash input password with stored salt
- Compare resulting hash with stored hash
- Time: 100-200ms (bcrypt is intentionally slow)

### Session Management

**Token Lifecycle**:
- **Expiration**: 7 days (604800 seconds)
- **Renewal**: Not implemented (user must re-login after expiration)
- **Revocation**: Not implemented (stateless JWT)
- **Storage**: Client-side localStorage (frontend responsibility)

**Security Measures**:
- **HTTPS only**: Tokens transmitted over TLS 1.3
- **HttpOnly cookies**: Not used (frontend needs access for API calls)
- **SameSite**: Not applicable (token in Authorization header)
- **Token rotation**: Not implemented (future enhancement)

### User Document Schema

```json
{
  "_id": ObjectId("60f7b3b3e4b0c8f7d8f7b3b3"),
  "email": "user@example.com",
  "password_hash": "$2b$12$abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ",
  "full_name": "John Doe",
  "is_admin": false,
  "is_active": true,
  "created_at": ISODate("2025-10-11T10:00:00Z"),
  "last_login": ISODate("2025-10-11T10:30:00Z"),
  "email_verified": false,
  "email_verification_token": null,
  "password_reset_token": null,
  "password_reset_expires": null
}
```

**Indexes**:
- `{email: 1}` (unique)
- `{created_at: -1}`
- `{last_login: -1}`

### Authentication Errors

**Common Error Responses**:

**400 Bad Request** (Validation Error):
```json
{
  "success": false,
  "error": "Validation failed",
  "details": "Password must be at least 8 characters long"
}
```

**401 Unauthorized** (Invalid Credentials):
```json
{
  "success": false,
  "error": "Invalid email or password"
}
```

**403 Forbidden** (Account Deactivated):
```json
{
  "success": false,
  "error": "Account is deactivated"
}
```

**409 Conflict** (Email Already Exists):
```json
{
  "success": false,
  "error": "Email already registered"
}
```

### Security Considerations

**Email Enumeration Prevention**:
- Same error message for "user not found" and "wrong password"
- Constant-time password verification
- Rate limiting on login endpoint (5 attempts per 15 minutes)

**Brute Force Protection**:
- Rate limiting: Max 5 failed login attempts per IP per 15 minutes
- Account lockout: After 10 failed attempts, lock account for 1 hour
- CAPTCHA: Required after 3 failed attempts (frontend implementation)

**Password Reset** (Future Implementation):
- Time-limited reset token (15 minutes expiration)
- Single-use token (deleted after use)
- Email verification required
- Password history check (prevent reusing last 5 passwords)

### Performance Characteristics

**Latency Breakdown**:

**Registration**:
- Input validation: 1-2ms
- Email existence check: 2-10ms (MongoDB query)
- Password hashing: 100-200ms (bcrypt cost 12)
- User insertion: 5-20ms (MongoDB write)
- JWT generation: 1-5ms
- **Total**: 109-237ms (Target: <500ms)

**Login**:
- Input validation: 1-2ms
- User query: 2-10ms (MongoDB, indexed by email)
- Password verification: 100-200ms (bcrypt)
- Last login update: 5-20ms (MongoDB write)
- JWT generation: 1-5ms
- **Total**: 109-237ms (Target: <500ms)

**Token Verification**:
- Token extraction: <1ms
- Signature verification: 1-5ms (HMAC-SHA256)
- Claims decoding: <1ms
- **Total**: 2-7ms (Target: <20ms)

### Monitoring & Metrics

**Key Metrics**:
- `auth_registration_total` (counter, labels: status)
- `auth_login_total` (counter, labels: status)
- `auth_login_latency_seconds` (histogram)
- `auth_token_verification_total` (counter, labels: status)
- `auth_failed_login_attempts_total` (counter, labels: email_hash)
- `auth_account_lockouts_total` (counter)

**Alerts**:
- Failed login rate >10% over 5 minutes
- Account lockouts >5 per hour
- Password hashing latency >300ms (p95) over 1 minute
- Token verification failure rate >5% over 5 minutes

---

## ARCH-DATA-FLOW-005: WebSocket Real-Time Updates

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-DATA-FLOW-005`
**Implementation**: `rust-core-engine/src/api/mod.rs`, `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

### Description

This flow describes how real-time updates are pushed from backend services to frontend clients via WebSocket connections, including connection management, message broadcasting, and heartbeat mechanisms.

### Data Flow Sequence Diagram

```mermaid
sequenceDiagram
    participant Frontend
    participant WSServer as WebSocket Server
    participant Auth as JWT Validator
    participant Channel as Broadcast Channel
    participant TradingLoop as Trading Engine
    participant AIService as AI Service
    participant MarketData as Market Data Processor

    Note over Frontend,MarketData: WebSocket Connection Establishment
    Frontend->>Frontend: Get JWT token from localStorage
    Frontend->>WSServer: Connect to ws://localhost:8080/ws
    Note right of Frontend: Query params: ?token={jwt_token}

    WSServer->>WSServer: Extract token from query
    WSServer->>Auth: Verify JWT token
    Auth->>Auth: Decode and validate signature

    alt Token invalid or expired
        Auth-->>WSServer: Invalid token
        WSServer->>Frontend: Close connection (code: 1008, reason: "Unauthorized")
        Frontend->>Frontend: Retry connection (up to 5 times)
        Frontend->>Frontend: Redirect to login if retries exhausted
    else Token valid
        Auth-->>WSServer: Claims {user_id, email, is_admin}
        WSServer->>WSServer: Store connection metadata
        Note right of WSServer: ConnectionMetadata {<br/>  user_id,<br/>  connected_at,<br/>  last_heartbeat<br/>}

        WSServer->>Channel: Subscribe to broadcast channel
        Note right of Channel: User subscribes to:<br/>- MarketData (all symbols)<br/>- AISignalReceived (all symbols)<br/>- TradeExecuted (user's trades)<br/>- PositionUpdate (user's positions)

        WSServer->>Frontend: Send welcome message
        Note right of Frontend: {<br/>  type: "Connected",<br/>  user_id,<br/>  timestamp<br/>}
        Frontend->>Frontend: Mark WebSocket as ready
        Frontend->>Frontend: Start heartbeat timer
    end

    Note over Frontend,MarketData: Heartbeat Mechanism
    loop Every 30 seconds
        Frontend->>WSServer: Send Ping
        Note right of Frontend: Frame: Opcode 0x9 (Ping)
        WSServer->>WSServer: Update last_heartbeat timestamp
        WSServer->>Frontend: Send Pong
        Note right of WSServer: Frame: Opcode 0xA (Pong)
        Frontend->>Frontend: Reset heartbeat timeout
    end

    Note over Frontend,MarketData: Market Data Broadcasting
    MarketData->>Channel: Publish MarketData event
    Note right of MarketData: Message: {<br/>  type: "MarketData",<br/>  symbol: "BTCUSDT",<br/>  price: 43275.25,<br/>  volume: 123.45,<br/>  timestamp: 1698765459999<br/>}

    Channel->>WSServer: Broadcast to all connections
    Note right of Channel: Fanout pattern:<br/>1 publisher → N subscribers

    WSServer->>WSServer: Serialize to JSON
    WSServer->>Frontend: Send text frame
    Note right of Frontend: Frame: Opcode 0x1 (Text)<br/>Payload: JSON string

    Frontend->>Frontend: Parse JSON message
    Frontend->>Frontend: Dispatch to Redux store
    Frontend->>Frontend: Update chart components
    Frontend->>Frontend: Trigger React re-render

    Note over Frontend,MarketData: AI Signal Broadcasting
    AIService->>Channel: Publish AISignalReceived event
    Note right of AIService: Message: {<br/>  type: "AISignalReceived",<br/>  symbol: "BTCUSDT",<br/>  direction: "BUY",<br/>  confidence: 0.82,<br/>  reasoning: "...",<br/>  timestamp: 1698765432<br/>}

    Channel->>WSServer: Broadcast to all connections
    WSServer->>Frontend: Send text frame
    Frontend->>Frontend: Parse and display AI signal
    Frontend->>Frontend: Show notification (if confidence > 0.75)

    Note over Frontend,MarketData: Trade Execution Broadcasting (User-Specific)
    TradingLoop->>Channel: Publish TradeExecuted event
    Note right of TradingLoop: Message: {<br/>  type: "TradeExecuted",<br/>  user_id: "60f7b3b3...",<br/>  trade_id: "trade_123",<br/>  symbol: "BTCUSDT",<br/>  side: "BUY",<br/>  quantity: 0.5,<br/>  price: 43250.50,<br/>  timestamp: 1698765500<br/>}

    Channel->>WSServer: Broadcast to user's connections
    WSServer->>WSServer: Filter by user_id
    Note right of WSServer: Only send to connections<br/>where connection.user_id == event.user_id

    WSServer->>Frontend: Send text frame
    Frontend->>Frontend: Update trades table
    Frontend->>Frontend: Update positions panel
    Frontend->>Frontend: Show success notification

    Note over Frontend,MarketData: Position Update Broadcasting (User-Specific)
    TradingLoop->>Channel: Publish PositionUpdate event
    Note right of TradingLoop: Message: {<br/>  type: "PositionUpdate",<br/>  user_id: "60f7b3b3...",<br/>  symbol: "BTCUSDT",<br/>  quantity: 1.5,<br/>  avg_entry_price: 43100.00,<br/>  unrealized_pnl: 225.75,<br/>  timestamp: 1698765505<br/>}

    Channel->>WSServer: Broadcast to user's connections
    WSServer->>WSServer: Filter by user_id
    WSServer->>Frontend: Send text frame
    Frontend->>Frontend: Update positions panel
    Frontend->>Frontend: Update PnL display

    Note over Frontend,MarketData: Connection Health Monitoring
    alt No heartbeat for 60 seconds
        WSServer->>WSServer: Detect stale connection
        WSServer->>Frontend: Close connection (code: 1001, reason: "Going away")
        WSServer->>Channel: Unsubscribe from broadcast
        WSServer->>WSServer: Remove connection metadata

        Frontend->>Frontend: Detect connection closed
        Frontend->>Frontend: Show reconnection banner
        Frontend->>Frontend: Attempt reconnection (exponential backoff)
        Note right of Frontend: Delay: 1s, 2s, 4s, 8s, 16s (max 5 attempts)
    end

    Note over Frontend,MarketData: Graceful Disconnection
    User->>Frontend: Close tab or logout
    Frontend->>WSServer: Send Close frame (code: 1000, reason: "Normal closure")
    WSServer->>Channel: Unsubscribe from broadcast
    WSServer->>WSServer: Remove connection metadata
    WSServer->>Frontend: Send Close frame acknowledgment
```

### WebSocket Connection Parameters

**Connection URL**:
- **Endpoint**: `ws://localhost:8080/ws` (development)
- **Production**: `wss://api.botcore.com/ws` (TLS encrypted)
- **Query Parameters**:
  - `token`: JWT authentication token (required)
  - Example: `ws://localhost:8080/ws?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...`

**Subprotocols**:
- None (using standard WebSocket protocol)

**Extensions**:
- `permessage-deflate`: Compression (optional, negotiated during handshake)

### Message Types

**Outbound Messages** (Server → Client):

**1. Connected**
```json
{
  "type": "Connected",
  "user_id": "60f7b3b3e4b0c8f7d8f7b3b3",
  "timestamp": 1698765432000
}
```

**2. MarketData**
```json
{
  "type": "MarketData",
  "symbol": "BTCUSDT",
  "price": 43275.25,
  "volume": 123.456789,
  "timestamp": 1698765459999,
  "change_percent": 0.058
}
```

**3. AISignalReceived**
```json
{
  "type": "AISignalReceived",
  "symbol": "BTCUSDT",
  "direction": "BUY",
  "confidence": 0.82,
  "ml_prediction": [0.82, 0.15, 0.03],
  "indicators": {
    "rsi": 67.5,
    "macd": {"macd": 125, "signal": 100, "histogram": 25}
  },
  "ai_reasoning": "The market shows strong bullish momentum...",
  "timestamp": 1698765432000
}
```

**4. TradeExecuted**
```json
{
  "type": "TradeExecuted",
  "user_id": "60f7b3b3e4b0c8f7d8f7b3b3",
  "trade_id": "trade_1698765432",
  "order_id": 123456789,
  "symbol": "BTCUSDT",
  "side": "BUY",
  "quantity": 0.5,
  "price": 43250.50,
  "status": "NEW",
  "timestamp": 1698765500000
}
```

**5. PositionUpdate**
```json
{
  "type": "PositionUpdate",
  "user_id": "60f7b3b3e4b0c8f7d8f7b3b3",
  "symbol": "BTCUSDT",
  "quantity": 1.5,
  "avg_entry_price": 43100.00,
  "unrealized_pnl": 225.75,
  "realized_pnl": 0.00,
  "timestamp": 1698765505000
}
```

**6. Error**
```json
{
  "type": "Error",
  "code": "RATE_LIMIT_EXCEEDED",
  "message": "Too many messages sent. Please slow down.",
  "timestamp": 1698765510000
}
```

**Inbound Messages** (Client → Server):

**1. Ping** (standard WebSocket ping frame)
- Opcode: 0x9
- No payload

**2. Subscribe** (Future Enhancement)
```json
{
  "type": "Subscribe",
  "channels": ["MarketData:BTCUSDT", "AISignal:ETHUSDT"]
}
```

**3. Unsubscribe** (Future Enhancement)
```json
{
  "type": "Unsubscribe",
  "channels": ["MarketData:BNBUSDT"]
}
```

### Broadcast Channel Implementation

**Rust Implementation**:
```rust
use tokio::sync::broadcast;

// Create broadcast channel with capacity 1000
let (tx, _rx) = broadcast::channel::<String>(1000);

// Publisher (Market Data Processor)
let message = serde_json::to_string(&MarketDataEvent {
    type_: "MarketData".to_string(),
    symbol: "BTCUSDT".to_string(),
    price: 43275.25,
    volume: 123.456789,
    timestamp: Utc::now().timestamp_millis(),
})?;
tx.send(message)?;

// Subscriber (WebSocket Connection)
let mut rx = tx.subscribe();
while let Ok(message) = rx.recv().await {
    // Send to WebSocket client
    ws_sender.send(Message::text(message)).await?;
}
```

**Channel Characteristics**:
- **Type**: `broadcast::Sender<String>` (Tokio broadcast channel)
- **Capacity**: 1000 messages
- **Behavior when full**: Drop oldest messages (FIFO)
- **Latency**: <1ms (in-memory channel)
- **Thread-safe**: Yes (Arc + Send + Sync)

### Heartbeat Mechanism

**Client-Side Implementation** (TypeScript):
```typescript
class WebSocketManager {
  private heartbeatInterval: NodeJS.Timeout | null = null;
  private heartbeatTimeout: NodeJS.Timeout | null = null;

  startHeartbeat() {
    // Send ping every 30 seconds
    this.heartbeatInterval = setInterval(() => {
      if (this.ws?.readyState === WebSocket.OPEN) {
        this.ws.send(JSON.stringify({ type: 'Ping' }));

        // Expect pong within 10 seconds
        this.heartbeatTimeout = setTimeout(() => {
          console.warn('Heartbeat timeout, reconnecting...');
          this.reconnect();
        }, 10000);
      }
    }, 30000);
  }

  onPong() {
    // Clear timeout when pong received
    if (this.heartbeatTimeout) {
      clearTimeout(this.heartbeatTimeout);
      this.heartbeatTimeout = null;
    }
  }

  stopHeartbeat() {
    if (this.heartbeatInterval) {
      clearInterval(this.heartbeatInterval);
      this.heartbeatInterval = null;
    }
    if (this.heartbeatTimeout) {
      clearTimeout(this.heartbeatTimeout);
      this.heartbeatTimeout = null;
    }
  }
}
```

**Server-Side Implementation** (Rust):
```rust
async fn handle_websocket_connection(ws: WebSocket, user_id: String) {
    let (mut ws_sender, mut ws_receiver) = ws.split();
    let mut last_heartbeat = Instant::now();

    // Spawn heartbeat checker
    tokio::spawn(async move {
        let mut interval = interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            if last_heartbeat.elapsed() > Duration::from_secs(60) {
                // No heartbeat for 60 seconds, close connection
                let _ = ws_sender.close().await;
                break;
            }
        }
    });

    // Handle incoming messages
    while let Some(result) = ws_receiver.next().await {
        match result {
            Ok(msg) => {
                if msg.is_ping() || msg.is_pong() {
                    last_heartbeat = Instant::now();
                }
                // Handle other message types
            }
            Err(e) => {
                error!("WebSocket error: {}", e);
                break;
            }
        }
    }
}
```

### Connection Management

**Connection Pool**:
- **Data Structure**: `HashMap<String, Vec<WebSocketConnection>>`
  - Key: user_id
  - Value: List of active connections (supports multiple tabs/devices)
- **Max Connections per User**: 10 (configurable)
- **Total Max Connections**: 1000 (configurable)

**Connection Metadata**:
```rust
struct WebSocketConnection {
    user_id: String,
    connection_id: String,
    ws_sender: SplitSink<WebSocket, Message>,
    connected_at: DateTime<Utc>,
    last_heartbeat: Instant,
    subscriptions: HashSet<String>,
}
```

**Connection Lifecycle Events**:
- `on_connect`: Validate token, add to pool, send welcome message
- `on_message`: Update last_heartbeat, handle subscriptions
- `on_disconnect`: Remove from pool, unsubscribe from channels
- `on_error`: Log error, close connection, send error to client

### Error Handling

**Connection Errors**:
- **Invalid token**: Close with code 1008 (Policy Violation)
- **Max connections exceeded**: Close with code 1008, message "Max connections reached"
- **Rate limit exceeded**: Close with code 1008, message "Rate limit exceeded"

**Message Errors**:
- **Invalid JSON**: Log warning, ignore message
- **Unknown message type**: Send Error message to client
- **Malformed message**: Send Error message, close if repeated

**Network Errors**:
- **Connection dropped**: Client auto-reconnects with exponential backoff
- **Server restart**: All clients reconnect within 30 seconds
- **Network timeout**: Both sides detect via heartbeat mechanism

### Reconnection Strategy

**Client-Side Exponential Backoff**:
```typescript
class WebSocketManager {
  private reconnectAttempts = 0;
  private maxReconnectAttempts = 5;

  reconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      this.showErrorNotification('Connection lost. Please refresh the page.');
      return;
    }

    const delay = Math.min(1000 * Math.pow(2, this.reconnectAttempts), 16000);
    console.log(`Reconnecting in ${delay}ms (attempt ${this.reconnectAttempts + 1}/${this.maxReconnectAttempts})`);

    setTimeout(() => {
      this.reconnectAttempts++;
      this.connect();
    }, delay);
  }

  onConnected() {
    // Reset reconnect attempts on successful connection
    this.reconnectAttempts = 0;
  }
}
```

**Reconnection Delays**:
- Attempt 1: 1 second
- Attempt 2: 2 seconds
- Attempt 3: 4 seconds
- Attempt 4: 8 seconds
- Attempt 5: 16 seconds
- After 5 attempts: Show error, require manual refresh

### Performance Characteristics

**Latency**:
- Connection establishment: 10-50ms
- Message serialization: 1-5ms
- Channel broadcast: <1ms
- Message send to client: 5-20ms (network)
- **Total End-to-End Latency**: 16-76ms (Target: <100ms)

**Throughput**:
- Messages per second per connection: 100-500
- Total messages per second (all connections): 10,000-50,000
- Bandwidth per connection: 10-50 KB/s
- Total bandwidth: 1-5 MB/s

**Resource Usage**:
- Memory per connection: 10-50 KB
- CPU per connection: <1% (mostly idle)
- Total memory (1000 connections): 10-50 MB

### Monitoring & Metrics

**Key Metrics**:
- `websocket_connections_total` (gauge)
- `websocket_connections_by_user` (gauge, labels: user_id)
- `websocket_messages_sent_total` (counter, labels: type)
- `websocket_messages_received_total` (counter)
- `websocket_broadcast_latency_seconds` (histogram)
- `websocket_connection_duration_seconds` (histogram)
- `websocket_disconnections_total` (counter, labels: reason)

**Alerts**:
- WebSocket connection failure rate >5% over 5 minutes
- Message broadcast latency >100ms (p95) over 1 minute
- Total active connections >900 (approaching limit)
- Heartbeat timeout rate >10% over 5 minutes

---

## ARCH-DATA-FLOW-006: Paper Trading Flow

**Priority**: ☑ High
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-DATA-FLOW-006`
**Implementation**: `rust-core-engine/src/paper_trading/mod.rs`

### Description

This flow describes how paper trading (simulated trading) operates, including virtual portfolio management, simulated order execution, and risk management without real money.

### Key Features

- **Virtual Portfolio**: Simulated balance (default: $10,000)
- **Real Market Data**: Uses live Binance prices
- **Simulated Execution**: Instant fills at current market price
- **Risk Management**: Same risk rules as live trading
- **Performance Tracking**: Real-time PnL calculation
- **No Financial Risk**: Zero real money involved

### Data Flow

```mermaid
sequenceDiagram
    participant User
    participant Frontend
    participant RustAPI
    participant PaperEngine as Paper Trading Engine
    participant RiskMgr as Risk Manager
    participant MarketData
    participant MongoDB

    User->>Frontend: Enable paper trading mode
    Frontend->>RustAPI: POST /api/paper-trading/start
    RustAPI->>PaperEngine: Initialize virtual portfolio
    PaperEngine->>MongoDB: Create portfolio {balance: 10000, positions: []}
    MongoDB-->>PaperEngine: Portfolio created
    PaperEngine-->>Frontend: {success: true, balance: 10000}

    User->>Frontend: Execute paper trade
    Frontend->>RustAPI: POST /api/paper-trading/execute
    RustAPI->>RiskMgr: Validate trade
    RiskMgr-->>RustAPI: Approved
    RustAPI->>MarketData: Get current price
    MarketData-->>RustAPI: Price: 43250.50
    RustAPI->>PaperEngine: Simulate order execution
    PaperEngine->>PaperEngine: Update virtual position
    PaperEngine->>MongoDB: Store paper trade
    PaperEngine-->>Frontend: Trade executed (simulated)
```

### Implementation Details

**Virtual Portfolio Schema**:
```json
{
  "_id": ObjectId,
  "user_id": ObjectId,
  "balance": 10000.00,
  "initial_balance": 10000.00,
  "positions": [
    {
      "symbol": "BTCUSDT",
      "quantity": 0.5,
      "avg_entry_price": 43100.00,
      "unrealized_pnl": 75.25,
      "realized_pnl": 0.00
    }
  ],
  "total_pnl": 75.25,
  "total_trades": 5,
  "win_rate": 0.60,
  "created_at": ISODate("2025-10-11T10:00:00Z"),
  "updated_at": ISODate("2025-10-11T10:35:00Z")
}
```

**Simulated Execution**:
- **Fill Price**: Current market price (from MarketDataCache)
- **Slippage**: 0.1% (configurable, simulates market impact)
- **Fees**: 0.04% (Binance taker fee)
- **Latency**: 0ms (instant fill)

---

## ARCH-DATA-FLOW-007: Error Propagation Flow

**Priority**: ☑ Medium
**Status**: ☑ Implemented
**Code Tags**: `@spec:ARCH-DATA-FLOW-007`

### Description

This flow describes how errors are handled, logged, and propagated through the system to provide meaningful feedback to users and administrators.

### Error Handling Strategy

**1. Error Classification**:
- **User Errors** (400-499): Invalid input, authentication failures
- **System Errors** (500-599): Database failures, external API errors
- **Network Errors**: Timeouts, connection failures
- **Business Logic Errors**: Risk violations, insufficient balance

**2. Error Response Format**:
```json
{
  "success": false,
  "error": "Human-readable error message",
  "error_code": "RISK_VIOLATION_POSITION_SIZE",
  "details": {
    "max_allowed": "10%",
    "requested": "15%"
  },
  "timestamp": 1698765432000,
  "request_id": "req_abc123"
}
```

**3. Error Logging**:
- **Level**: ERROR for 5xx, WARN for 4xx
- **Context**: Include request ID, user ID, endpoint
- **Destination**: stdout (JSON format), Elasticsearch (production)
- **Retention**: 90 days

**4. Error Metrics**:
- `errors_total` (counter, labels: status_code, endpoint)
- `errors_by_type` (counter, labels: error_type)
- `database_errors_total` (counter)
- `external_api_errors_total` (counter, labels: service)

---

## Summary

This data flow architecture document provides comprehensive specifications for all critical data flows in the Bot Core platform:

1. **ARCH-DATA-FLOW-001: Market Data Flow** - Real-time market data from Binance to frontend
2. **ARCH-DATA-FLOW-002: Trading Execution Flow** - Complete trade lifecycle with risk management
3. **ARCH-DATA-FLOW-003: AI Analysis Flow** - ML-powered trading signals with GPT-4 reasoning
4. **ARCH-DATA-FLOW-004: Authentication Flow** - User registration, login, and JWT management
5. **ARCH-DATA-FLOW-005: WebSocket Real-Time Updates** - Push notifications to frontend
6. **ARCH-DATA-FLOW-006: Paper Trading Flow** - Simulated trading without real money
7. **ARCH-DATA-FLOW-007: Error Propagation Flow** - Error handling and reporting

Each flow includes:
- Detailed sequence diagrams (Mermaid)
- Data schemas and formats
- Processing steps and transformations
- Performance characteristics
- Error handling strategies
- Monitoring metrics and alerts

This architecture ensures:
- **Low Latency**: <100ms for market data, <500ms for trades
- **High Reliability**: Automatic reconnection, error recovery
- **Scalability**: Broadcast channels, connection pooling
- **Security**: JWT authentication, input validation
- **Observability**: Comprehensive metrics and logging
