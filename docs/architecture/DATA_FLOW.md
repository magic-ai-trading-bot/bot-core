# Data Flow Architecture

## Overview

This document describes how data flows through the Bot Core system, including request/response patterns, event-driven communication, and real-time data streaming.

## Primary Data Flows

### 1. User Authentication Flow

```mermaid
sequenceDiagram
    participant User
    participant Browser
    participant Next.js
    participant Rust
    participant MongoDB

    User->>Browser: Enter email/password
    Browser->>Next.js: Submit login form
    Next.js->>Rust: POST /api/auth/login
    activate Rust
    Rust->>MongoDB: Query user by email
    MongoDB-->>Rust: User document
    Rust->>Rust: Verify password hash
    Rust->>Rust: Generate JWT token
    Rust->>MongoDB: Update last_login
    Rust-->>Next.js: {token, refresh_token, user}
    deactivate Rust
    Next.js->>Next.js: Store in local storage
    Next.js-->>Browser: Redirect to dashboard
    Browser-->>User: Show dashboard
```

**Data Elements**:
```json
{
  "request": {
    "email": "user@example.com",
    "password": "hashed_password"
  },
  "response": {
    "token": "eyJhbGci...",
    "refresh_token": "eyJhbGci...",
    "user": {
      "id": "user_123",
      "email": "user@example.com",
      "full_name": "John Doe"
    }
  }
}
```

### 2. Real-Time Market Data Flow

```mermaid
sequenceDiagram
    participant Binance
    participant Rust WS
    participant Redis
    participant Dashboard WS
    participant User

    Binance->>Rust WS: Stream: Price Update
    activate Rust WS
    Rust WS->>Rust WS: Parse message
    Rust WS->>Redis: Cache latest price
    Rust WS->>Dashboard WS: Broadcast to subscribers
    deactivate Rust WS
    Dashboard WS-->>User: Update chart

    Note over Rust WS,Redis: Cache TTL: 1 second
    Note over Dashboard WS,User: WebSocket push
```

**Message Format**:
```json
{
  "type": "price_update",
  "data": {
    "symbol": "BTCUSDT",
    "price": 45123.45,
    "bid": 45123.40,
    "ask": 45123.50,
    "volume_24h": 25000000000,
    "change_24h": 2.5,
    "timestamp": "2025-10-10T12:00:00Z"
  }
}
```

### 3. AI Analysis Request Flow

```mermaid
sequenceDiagram
    participant User
    participant Dashboard
    participant Rust
    participant Redis
    participant Python
    participant OpenAI
    participant MongoDB

    User->>Dashboard: Click "Analyze BTCUSDT"
    Dashboard->>Rust: POST /api/ai/analyze
    activate Rust
    Rust->>Rust: Validate request
    Rust->>Rust: Check rate limits
    Rust->>Python: POST /ai/analyze
    deactivate Rust

    activate Python
    Python->>Redis: Check cache
    alt Cache Hit
        Redis-->>Python: Cached analysis
        Python-->>Rust: Return cached result
    else Cache Miss
        Python->>Python: Calculate indicators
        Python->>OpenAI: Request analysis
        activate OpenAI
        OpenAI-->>Python: AI response
        deactivate OpenAI
        Python->>Python: Parse & validate
        Python->>Redis: Cache (TTL: 5 min)
        Python->>MongoDB: Store analysis
        Python-->>Rust: Return analysis
    end
    deactivate Python

    activate Rust
    Rust->>MongoDB: Log request
    Rust-->>Dashboard: Analysis result
    deactivate Rust
    Dashboard-->>User: Display analysis
```

**Request/Response**:
```json
{
  "request": {
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candles": [...],
    "technical_indicators": {...}
  },
  "response": {
    "signal": "Long",
    "confidence": 0.85,
    "reasoning": "Strong bullish momentum...",
    "suggested_entry": 45190.00,
    "suggested_stop_loss": 44900.00,
    "suggested_take_profit": 45600.00,
    "risk_reward_ratio": 1.37
  }
}
```

### 4. Trade Execution Flow

```mermaid
sequenceDiagram
    participant User
    participant Dashboard
    participant Rust
    participant Risk Mgr
    participant Binance
    participant MongoDB
    participant RabbitMQ
    participant WebSocket

    User->>Dashboard: Place order
    Dashboard->>Rust: POST /api/trades/execute
    activate Rust

    Rust->>Rust: Authenticate user
    Rust->>Risk Mgr: Validate order
    activate Risk Mgr
    Risk Mgr->>Risk Mgr: Check position limits
    Risk Mgr->>Risk Mgr: Check balance
    Risk Mgr->>Risk Mgr: Check daily loss
    Risk Mgr-->>Rust: Validation result
    deactivate Risk Mgr

    alt Validation Failed
        Rust-->>Dashboard: Error: Risk limit exceeded
    else Validation Passed
        Rust->>Binance: Place order
        activate Binance
        Binance-->>Rust: Order confirmation
        deactivate Binance

        par Store Trade
            Rust->>MongoDB: Insert trade record
        and Publish Event
            Rust->>RabbitMQ: Publish trade_executed
        and Notify User
            Rust->>WebSocket: Broadcast order update
        end

        Rust-->>Dashboard: Trade confirmed
    end
    deactivate Rust

    WebSocket-->>User: Real-time notification
```

**Trade Record**:
```json
{
  "trade_id": "123e4567-e89b-12d3-a456-426614174000",
  "user_id": "user_123",
  "symbol": "BTCUSDT",
  "side": "BUY",
  "type": "LIMIT",
  "quantity": 0.001,
  "price": 45000.00,
  "executed_quantity": 0.001,
  "executed_price": 45000.00,
  "status": "FILLED",
  "commission": 0.045,
  "timestamp": "2025-10-10T12:00:00Z"
}
```

### 5. Paper Trading Flow

```mermaid
sequenceDiagram
    participant User
    participant Dashboard
    participant Rust
    participant Paper Engine
    participant MongoDB
    participant Market Data

    User->>Dashboard: Enable paper trading
    Dashboard->>Rust: POST /api/paper-trading/start
    activate Rust
    Rust->>MongoDB: Create paper account
    Rust->>Paper Engine: Initialize session
    Rust-->>Dashboard: Session created
    deactivate Rust

    User->>Dashboard: Place paper trade
    Dashboard->>Rust: POST /api/paper-trading/execute
    activate Rust
    Rust->>Paper Engine: Execute virtual trade
    activate Paper Engine
    Paper Engine->>Market Data: Get current price
    Market Data-->>Paper Engine: Real price
    Paper Engine->>Paper Engine: Simulate slippage
    Paper Engine->>Paper Engine: Calculate fees
    Paper Engine->>Paper Engine: Update virtual balance
    Paper Engine->>MongoDB: Store paper trade
    Paper Engine-->>Rust: Trade result
    deactivate Paper Engine
    Rust-->>Dashboard: Virtual trade executed
    deactivate Rust
```

## Event-Driven Communication

### RabbitMQ Message Flow

```mermaid
graph LR
    subgraph "Publishers"
        RUST[Rust Core]
        PYTHON[Python AI]
    end

    subgraph "RabbitMQ"
        EXCHANGE[Topic Exchange]
        Q1[Trading Queue]
        Q2[AI Signals Queue]
        Q3[Notifications Queue]
    end

    subgraph "Consumers"
        LOGGER[Logging Service]
        NOTIF[Notification Service]
        ANALYTICS[Analytics Service]
    end

    RUST -->|trade.executed| EXCHANGE
    RUST -->|trade.failed| EXCHANGE
    PYTHON -->|ai.signal.generated| EXCHANGE

    EXCHANGE --> Q1
    EXCHANGE --> Q2
    EXCHANGE --> Q3

    Q1 --> LOGGER
    Q2 --> ANALYTICS
    Q3 --> NOTIF
```

**Message Types**:

1. **Trade Events**:
```json
{
  "event": "trade_executed",
  "routing_key": "trade.executed",
  "data": {
    "trade_id": "123e4567",
    "symbol": "BTCUSDT",
    "side": "BUY",
    "quantity": 0.001,
    "price": 45000.00
  },
  "timestamp": "2025-10-10T12:00:00Z"
}
```

2. **AI Signal Events**:
```json
{
  "event": "ai_signal_generated",
  "routing_key": "ai.signal.generated",
  "data": {
    "analysis_id": "550e8400",
    "symbol": "BTCUSDT",
    "signal": "Long",
    "confidence": 0.85
  },
  "timestamp": "2025-10-10T12:00:00Z"
}
```

3. **Risk Events**:
```json
{
  "event": "risk_limit_exceeded",
  "routing_key": "risk.limit.exceeded",
  "data": {
    "user_id": "user_123",
    "limit_type": "daily_loss",
    "current_value": -5.5,
    "limit_value": -5.0
  },
  "timestamp": "2025-10-10T12:00:00Z"
}
```

## WebSocket Communication

### Connection Lifecycle

```mermaid
stateDiagram-v2
    [*] --> Connecting: Client initiates
    Connecting --> Connected: Handshake success
    Connecting --> Failed: Handshake failed
    Connected --> Authenticated: Auth message
    Authenticated --> Subscribed: Subscribe to channels
    Subscribed --> Receiving: Receive messages
    Receiving --> Subscribed: Continue
    Subscribed --> Unsubscribed: Unsubscribe
    Unsubscribed --> Subscribed: Resubscribe
    Authenticated --> Disconnected: Close
    Receiving --> Disconnected: Error/Close
    Disconnected --> Connecting: Reconnect
    Disconnected --> [*]
    Failed --> [*]
```

### WebSocket Message Types

```mermaid
graph LR
    CLIENT[Client] -->|Subscribe| SERVER[Server]
    SERVER -->|price_update| CLIENT
    SERVER -->|order_update| CLIENT
    SERVER -->|position_update| CLIENT
    SERVER -->|ai_signal| CLIENT
    SERVER -->|account_update| CLIENT
    CLIENT -->|ping| SERVER
    SERVER -->|pong| CLIENT
```

**Message Examples**:

1. **Subscribe**:
```json
{
  "type": "subscribe",
  "channels": ["BTCUSDT@ticker", "ETHUSDT@ticker"],
  "auth_token": "eyJhbGci..."
}
```

2. **Price Update**:
```json
{
  "type": "price_update",
  "channel": "BTCUSDT@ticker",
  "data": {
    "symbol": "BTCUSDT",
    "price": 45123.45,
    "volume": 1234.56
  },
  "timestamp": "2025-10-10T12:00:00.123Z",
  "sequence": 12345
}
```

3. **Order Update**:
```json
{
  "type": "order_update",
  "data": {
    "order_id": "order_123",
    "status": "FILLED",
    "executed_quantity": 0.001,
    "executed_price": 45000.00
  },
  "timestamp": "2025-10-10T12:00:00Z"
}
```

## Caching Strategy

### Cache Hierarchy

```mermaid
graph TB
    REQUEST[Client Request]

    subgraph "L1: Browser Cache"
        BROWSER[LocalStorage/SessionStorage]
    end

    subgraph "L2: CDN Cache"
        CDN[CloudFront]
    end

    subgraph "L3: Application Cache"
        REDIS[Redis Cache]
    end

    subgraph "L4: Database"
        MONGO[(MongoDB)]
    end

    REQUEST --> BROWSER
    BROWSER -->|Miss| CDN
    CDN -->|Miss| REDIS
    REDIS -->|Miss| MONGO

    MONGO -->|Store| REDIS
    REDIS -->|Store| CDN
    CDN -->|Store| BROWSER
```

### Cache Invalidation

```mermaid
sequenceDiagram
    participant Service
    participant Redis
    participant MongoDB
    participant Subscribers

    Service->>MongoDB: Update data
    MongoDB-->>Service: Update confirmed
    Service->>Redis: PUBLISH invalidate:key
    Redis->>Subscribers: Broadcast invalidation
    Subscribers->>Subscribers: Clear local cache
    Service->>Redis: DEL key
```

## Data Transformation Pipeline

### Market Data Processing

```mermaid
graph LR
    A[Raw Binance Data] --> B[Parse & Validate]
    B --> C[Normalize Format]
    C --> D[Calculate Indicators]
    D --> E[Enrich with Metadata]
    E --> F[Store in MongoDB]
    E --> G[Cache in Redis]
    E --> H[Broadcast via WebSocket]
```

**Example Transformation**:

**Input (Binance)**:
```json
{
  "e": "kline",
  "E": 1701234567000,
  "s": "BTCUSDT",
  "k": {
    "t": 1701234560000,
    "o": "45000.00",
    "h": "45100.00",
    "l": "44950.00",
    "c": "45050.00",
    "v": "123.456"
  }
}
```

**Output (Internal)**:
```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1m",
  "open_time": 1701234560000,
  "open": 45000.00,
  "high": 45100.00,
  "low": 44950.00,
  "close": 45050.00,
  "volume": 123.456,
  "close_time": 1701234619999,
  "indicators": {
    "rsi": 65.5,
    "macd": 12.34
  }
}
```

## Database Access Patterns

### Read Patterns

```mermaid
graph LR
    subgraph "Read Operations"
        R1[Get User Profile]
        R2[Get Open Positions]
        R3[Get Trade History]
        R4[Get AI Analysis]
    end

    subgraph "MongoDB Collections"
        USERS[(users)]
        POSITIONS[(positions)]
        TRADES[(trades)]
        ANALYSIS[(ai_analysis)]
    end

    R1 --> USERS
    R2 --> POSITIONS
    R3 --> TRADES
    R4 --> ANALYSIS
```

**Query Examples**:

1. **Get Open Positions**:
```javascript
db.positions.find({
  user_id: "user_123",
  closed_at: null
}).sort({ opened_at: -1 })
```

2. **Get Recent Trades**:
```javascript
db.trades.find({
  user_id: "user_123",
  created_at: { $gte: ISODate("2025-10-09T00:00:00Z") }
}).sort({ created_at: -1 }).limit(100)
```

### Write Patterns

```mermaid
sequenceDiagram
    participant App
    participant Primary
    participant Secondary1
    participant Secondary2

    App->>Primary: Write operation
    activate Primary
    Primary->>Primary: Write to oplog
    Primary-->>App: Write acknowledged
    deactivate Primary

    par Replication
        Primary->>Secondary1: Replicate oplog
        Primary->>Secondary2: Replicate oplog
    end

    Note over Primary,Secondary2: Async replication
```

## Data Consistency Patterns

### Eventual Consistency

Used for:
- AI analysis results
- Historical trade data
- Analytical reports

```mermaid
sequenceDiagram
    participant Service A
    participant MongoDB
    participant Service B

    Service A->>MongoDB: Write data
    MongoDB-->>Service A: Acknowledged

    Note over MongoDB,Service B: Replication lag (< 100ms)

    Service B->>MongoDB: Read data
    MongoDB-->>Service B: Data (may be stale)
```

### Strong Consistency

Used for:
- Account balances
- Open positions
- Order placement

```mermaid
sequenceDiagram
    participant Service
    participant MongoDB Primary

    Service->>MongoDB Primary: Write with majority
    MongoDB Primary->>MongoDB Primary: Replicate to majority
    MongoDB Primary-->>Service: Acknowledged (consistent)
```

## Data Retention Policy

```mermaid
graph TB
    subgraph "Hot Data (< 30 days)"
        MONGO[(MongoDB)]
    end

    subgraph "Warm Data (30-180 days)"
        ARCHIVE[(Archive Collection)]
    end

    subgraph "Cold Data (> 180 days)"
        S3[(S3 Glacier)]
    end

    MONGO -->|30 days| ARCHIVE
    ARCHIVE -->|180 days| S3
```

**Retention Schedule**:

| Data Type | Hot Storage | Archive | Cold Storage |
|-----------|-------------|---------|--------------|
| Trade records | 30 days | 6 months | 7 years |
| Price data | 7 days | 1 month | 1 year |
| AI analysis | 30 days | 3 months | Deleted |
| User activity | 90 days | 1 year | 7 years |
| System logs | 7 days | 1 month | Deleted |

## Performance Metrics

### Target Latencies

| Operation | Target | P95 | P99 |
|-----------|--------|-----|-----|
| REST API call | < 100ms | < 200ms | < 500ms |
| WebSocket message | < 50ms | < 100ms | < 200ms |
| Trade execution | < 10ms | < 20ms | < 50ms |
| AI analysis | < 2s | < 3s | < 5s |
| Database query | < 50ms | < 100ms | < 200ms |
| Cache hit | < 1ms | < 2ms | < 5ms |

## Error Handling

### Retry Strategy

```mermaid
graph LR
    REQUEST[Request] --> ATTEMPT1[Attempt 1]
    ATTEMPT1 -->|Fail| WAIT1[Wait 1s]
    WAIT1 --> ATTEMPT2[Attempt 2]
    ATTEMPT2 -->|Fail| WAIT2[Wait 2s]
    WAIT2 --> ATTEMPT3[Attempt 3]
    ATTEMPT3 -->|Fail| ERROR[Return Error]

    ATTEMPT1 -->|Success| SUCCESS[Success]
    ATTEMPT2 -->|Success| SUCCESS
    ATTEMPT3 -->|Success| SUCCESS
```

**Retryable Errors**:
- Network timeouts
- Temporary service unavailability
- Rate limit errors (with backoff)

**Non-Retryable Errors**:
- Invalid request parameters
- Authentication failures
- Insufficient balance

## References

- [System Architecture](./SYSTEM_ARCHITECTURE.md)
- [Security Architecture](./SECURITY_ARCHITECTURE.md)
- [API Specification](../../specs/API_SPEC.md)
- [Data Models](../../specs/DATA_MODELS.md)
