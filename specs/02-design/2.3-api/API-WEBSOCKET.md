# API-WEBSOCKET.md - WebSocket Protocol Specification

**Version:** 1.0.0
**Services:** Rust Core Engine (Port 8080), Python AI Service (Port 8000)

## Table of Contents

1. [Overview](#overview)
2. [Connection Establishment](#connection-establishment)
3. [Message Types](#message-types)
4. [Heartbeat Protocol](#heartbeat-protocol)
5. [Error Handling](#error-handling)
6. [Frontend Integration](#frontend-integration)
7. [Security Considerations](#security-considerations)

---

## Overview

The Bot Core trading platform uses WebSocket connections for real-time bidirectional communication between services and clients. Two WebSocket servers are available:

1. **Rust Core Engine WebSocket** (`ws://localhost:8080/ws`)
   - Market data updates
   - Trading signals
   - Position updates
   - Bot status changes
   - Chart updates

2. **Python AI Service WebSocket** (`ws://localhost:8000/ws`)
   - AI signal generation
   - Real-time analysis results
   - Connection status

**Protocol:** WebSocket (RFC 6455)
**Data Format:** JSON
**Encoding:** UTF-8

---

## Connection Establishment

### Rust Core Engine WebSocket

**Endpoint:** `ws://localhost:8080/ws`

**Connection Request:**
```http
GET /ws HTTP/1.1
Host: localhost:8080
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
```

**Optional Authentication Header:**
```http
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Connection Response:**
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

**JavaScript Example:**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('✅ Connected to Rust Core WebSocket');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('📨 Received:', message);
  handleMessage(message);
};

ws.onerror = (error) => {
  console.error('❌ WebSocket error:', error);
};

ws.onclose = (event) => {
  console.log('🔌 Disconnected:', event.code, event.reason);
  // Implement reconnection logic
  setTimeout(() => connectWebSocket(), 5000);
};
```

**Code Location:** `rust-core-engine/src/api/mod.rs:142-145, 486-534`

---

### Python AI Service WebSocket

**Endpoint:** `ws://localhost:8000/ws`

**Connection Request:**
```http
GET /ws HTTP/1.1
Host: localhost:8000
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==
Sec-WebSocket-Version: 13
```

**Connection Response:**
```http
HTTP/1.1 101 Switching Protocols
Upgrade: websocket
Connection: Upgrade
Sec-WebSocket-Accept: s3pPLMBiTxaQ9kYGzzhZRbK+xOo=
```

**Welcome Message:**
```json
{
  "type": "connection",
  "message": "Connected to AI Trading Service",
  "timestamp": "2025-10-10T10:30:00.000000+00:00"
}
```

**Python Example:**
```python
import asyncio
import websockets
import json

async def connect_ai_websocket():
    uri = "ws://localhost:8000/ws"
    async with websockets.connect(uri) as websocket:
        print("✅ Connected to AI WebSocket")

        async for message in websocket:
            data = json.loads(message)
            print(f"📨 Received: {data}")
            await handle_ai_signal(data)
```

**Code Location:** `python-ai-service/main.py:1731-1747`

---

## Message Types

### 1. Connected

**Direction:** Server → Client (Rust Core Only)
**Description:** Initial connection confirmation
**Frequency:** Once per connection

**Message Structure:**
```json
{
  "type": "Connected",
  "data": {
    "client_id": "client_123456",
    "server_time": 1697234567000,
    "version": "1.0.0"
  },
  "timestamp": 1697234567000
}
```

**Frontend Handling:**
```javascript
case 'Connected':
  console.log('Connected to server:', message.data.client_id);
  setConnectionStatus('connected');
  break;
```

---

### 2. MarketData

**Direction:** Server → Client (Rust Core)
**Description:** Real-time market price updates for tracked symbols
**Frequency:** Every 1 second (or on price change)

**Message Structure:**
```json
{
  "type": "MarketData",
  "data": {
    "symbol": "BTCUSDT",
    "price": 67500.50,
    "change_24h": 2.5,
    "volume_24h": 50000000000.0,
    "high_24h": 68000.00,
    "low_24h": 66500.00,
    "bid": 67500.25,
    "ask": 67500.75,
    "last_update": 1697234567000
  },
  "timestamp": 1697234567000
}
```

**Frontend Handling:**
```javascript
case 'MarketData':
  updateMarketPrice(message.data.symbol, message.data.price);
  updatePriceChart(message.data.symbol, message.data);
  updateTickerDisplay(message.data);
  break;
```

**Related FR:** FR-WS-MARKET-001

---

### 3. ChartUpdate

**Direction:** Server → Client (Rust Core)
**Description:** New candlestick data or chart updates
**Frequency:** On candle close (1m, 5m, 15m, 1h, 4h, 1d)

**Message Structure:**
```json
{
  "type": "ChartUpdate",
  "data": {
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candle": {
      "timestamp": 1697234400000,
      "open": 67400.00,
      "high": 67550.00,
      "low": 67350.00,
      "close": 67500.50,
      "volume": 1250.5,
      "close_time": 1697238000000
    },
    "indicators": {
      "rsi": 65.5,
      "macd": 123.45,
      "macd_signal": 110.23,
      "bollinger_upper": 68000.00,
      "bollinger_middle": 67500.00,
      "bollinger_lower": 67000.00
    }
  },
  "timestamp": 1697234567000
}
```

**Frontend Handling:**
```javascript
case 'ChartUpdate':
  appendCandleToChart(
    message.data.symbol,
    message.data.timeframe,
    message.data.candle
  );
  updateIndicators(message.data.symbol, message.data.indicators);
  break;
```

**Related FR:** FR-WS-CHART-001

---

### 4. PositionUpdate

**Direction:** Server → Client (Rust Core)
**Description:** Position opened, updated, or closed
**Frequency:** On position change

**Message Structure:**
```json
{
  "type": "PositionUpdate",
  "data": {
    "action": "OPENED",
    "position": {
      "id": "pos_123456",
      "symbol": "BTCUSDT",
      "side": "LONG",
      "entry_price": 67000.00,
      "current_price": 67500.50,
      "quantity": 0.001,
      "leverage": 10,
      "unrealized_pnl": 0.50,
      "unrealized_pnl_percent": 0.75,
      "liquidation_price": 60300.00,
      "margin": 6.70,
      "stop_loss": 66000.00,
      "take_profit": 68000.00,
      "entry_time": 1697230000000
    }
  },
  "timestamp": 1697234567000
}
```

**Action Types:**
- `OPENED` - New position opened
- `UPDATED` - Position parameters updated (price, PnL, etc.)
- `CLOSED` - Position closed

**Frontend Handling:**
```javascript
case 'PositionUpdate':
  switch (message.data.action) {
    case 'OPENED':
      addPositionToList(message.data.position);
      showNotification('Position opened: ' + message.data.position.symbol);
      break;
    case 'UPDATED':
      updatePositionInList(message.data.position);
      updatePnLDisplay(message.data.position);
      break;
    case 'CLOSED':
      removePositionFromList(message.data.position.id);
      showNotification('Position closed: ' + message.data.position.symbol);
      break;
  }
  break;
```

**Related FR:** FR-WS-POSITION-001

---

### 5. TradeExecuted

**Direction:** Server → Client (Rust Core)
**Description:** Trade order executed
**Frequency:** On order fill

**Message Structure:**
```json
{
  "type": "TradeExecuted",
  "data": {
    "trade_id": "trade_123456",
    "order_id": "binance_order_789",
    "symbol": "BTCUSDT",
    "side": "BUY",
    "type": "LIMIT",
    "quantity": 0.001,
    "price": 67500.00,
    "executed_quantity": 0.001,
    "executed_price": 67500.00,
    "commission": 0.0675,
    "commission_asset": "USDT",
    "status": "FILLED",
    "timestamp": 1697234567000,
    "strategy": "RSI Strategy",
    "ai_signal_id": "signal_123"
  },
  "timestamp": 1697234567000
}
```

**Trade Status:**
- `FILLED` - Order completely filled
- `PARTIALLY_FILLED` - Order partially filled
- `CANCELED` - Order canceled
- `REJECTED` - Order rejected by exchange

**Frontend Handling:**
```javascript
case 'TradeExecuted':
  addTradeToHistory(message.data);
  updatePortfolioBalance();
  playTradeSound(message.data.side);
  showNotification(
    `Trade executed: ${message.data.side} ${message.data.quantity} ${message.data.symbol} @ ${message.data.executed_price}`
  );
  break;
```

**Related FR:** FR-WS-TRADE-001

---

### 6. AISignalReceived

**Direction:** Server → Client (Both Rust Core and Python AI)
**Description:** AI trading signal generated
**Frequency:** On signal generation (every 5 minutes or on-demand)

**Message Structure:**
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
    "reasoning": "Strong bullish momentum with RSI oversold (28), MACD crossover, and positive GPT-4 sentiment analysis",
    "strategy_scores": {
      "RSI Strategy": 0.85,
      "MACD Strategy": 0.78,
      "Bollinger Bands Strategy": 0.65,
      "Volume Strategy": 0.72
    },
    "market_analysis": {
      "trend_direction": "Bullish",
      "trend_strength": 0.82,
      "volatility_level": "Medium"
    },
    "risk_assessment": {
      "overall_risk": "Medium",
      "recommended_position_size": 0.05,
      "stop_loss_suggestion": 66000.00,
      "take_profit_suggestion": 68500.00
    }
  },
  "timestamp": 1697234567000
}
```

**Signal Values:**
- `long` - Buy/Long signal
- `short` - Sell/Short signal
- `neutral` - No clear direction, hold

**Confidence Range:** 0.0 - 1.0 (0% - 100%)

**Frontend Handling:**
```javascript
case 'AISignalReceived':
  displayAISignal(message.data);

  if (message.data.confidence > 0.70 && autoTradingEnabled) {
    executeTradeFromSignal(message.data);
  }

  addSignalToHistory(message.data);
  updateSignalChart(message.data.symbol, message.data.signal);

  if (notificationsEnabled) {
    showNotification(
      `AI Signal: ${message.data.signal.toUpperCase()} ${message.data.symbol} (${(message.data.confidence * 100).toFixed(0)}%)`
    );
  }
  break;
```

**Code Location (Rust):** `rust-core-engine/src/api/mod.rs:556-587`
**Code Location (Python):** `python-ai-service/main.py:100-126, 1871-1883`
**Related FR:** FR-WS-AI-001

---

### 7. BotStatusUpdate

**Direction:** Server → Client (Rust Core)
**Description:** Bot operational status changed
**Frequency:** On status change

**Message Structure:**
```json
{
  "type": "BotStatusUpdate",
  "data": {
    "status": "RUNNING",
    "mode": "PAPER_TRADING",
    "uptime_seconds": 3600,
    "total_positions": 2,
    "total_balance": 10500.50,
    "total_pnl": 500.50,
    "total_pnl_percent": 5.005,
    "enabled_strategies": [
      "RSI Strategy",
      "MACD Strategy",
      "Bollinger Bands Strategy"
    ],
    "last_signal_time": 1697234567000,
    "next_signal_time": 1697234867000
  },
  "timestamp": 1697234567000
}
```

**Status Values:**
- `RUNNING` - Bot is active and trading
- `PAUSED` - Bot is paused, no new trades
- `STOPPED` - Bot is stopped
- `ERROR` - Bot encountered an error

**Mode Values:**
- `PAPER_TRADING` - Simulated trading
- `LIVE_TRADING` - Real money trading

**Frontend Handling:**
```javascript
case 'BotStatusUpdate':
  updateBotStatusIndicator(message.data.status);
  updateBotMetrics(message.data);

  if (message.data.status === 'ERROR') {
    showErrorAlert('Bot encountered an error');
  }

  if (message.data.status === 'STOPPED') {
    showNotification('Bot has been stopped');
  }
  break;
```

**Related FR:** FR-WS-STATUS-001

---

### 8. Error

**Direction:** Server → Client (Both services)
**Description:** Error notification
**Frequency:** On error occurrence

**Message Structure:**
```json
{
  "type": "Error",
  "data": {
    "code": "TRADE_EXECUTION_FAILED",
    "message": "Insufficient balance for trade execution",
    "severity": "ERROR",
    "details": {
      "required_balance": 100.00,
      "available_balance": 50.00,
      "symbol": "BTCUSDT"
    },
    "timestamp": 1697234567000,
    "recoverable": true
  },
  "timestamp": 1697234567000
}
```

**Severity Levels:**
- `INFO` - Informational message
- `WARNING` - Warning, may require attention
- `ERROR` - Error occurred, operation failed
- `CRITICAL` - Critical error, service disruption

**Error Codes:**
- `TRADE_EXECUTION_FAILED` - Trade order failed
- `INSUFFICIENT_BALANCE` - Not enough balance
- `INVALID_POSITION` - Position not found or invalid
- `MARKET_DATA_ERROR` - Market data fetch failed
- `AI_SERVICE_ERROR` - AI service unavailable
- `WEBSOCKET_ERROR` - WebSocket connection error

**Frontend Handling:**
```javascript
case 'Error':
  console.error('Server error:', message.data);

  switch (message.data.severity) {
    case 'CRITICAL':
      showCriticalErrorModal(message.data);
      break;
    case 'ERROR':
      showErrorToast(message.data.message);
      break;
    case 'WARNING':
      showWarningToast(message.data.message);
      break;
    case 'INFO':
      showInfoToast(message.data.message);
      break;
  }

  logError(message.data);
  break;
```

**Related FR:** FR-WS-ERROR-001

---

### 9. Pong

**Direction:** Server → Client (Both services)
**Description:** Heartbeat response to Ping
**Frequency:** In response to client Ping

**Message Structure:**
```json
{
  "type": "Pong",
  "data": {
    "timestamp": 1697234567000
  },
  "timestamp": 1697234567000
}
```

**Frontend Handling:**
```javascript
case 'Pong':
  updateLastHeartbeat(Date.now());
  resetConnectionTimeout();
  break;
```

**Related FR:** FR-WS-HEARTBEAT-001

---

## Heartbeat Protocol

### Purpose

Heartbeat messages ensure the WebSocket connection is alive and detect disconnections.

### Client-Side Ping

**Recommended Interval:** 30 seconds

**Ping Message (Client → Server):**
```json
{
  "type": "Ping",
  "timestamp": 1697234567000
}
```

**JavaScript Implementation:**
```javascript
let heartbeatInterval;
let connectionTimeout;

function startHeartbeat(ws) {
  // Send ping every 30 seconds
  heartbeatInterval = setInterval(() => {
    if (ws.readyState === WebSocket.OPEN) {
      ws.send(JSON.stringify({
        type: 'Ping',
        timestamp: Date.now()
      }));

      // Set timeout for pong response
      connectionTimeout = setTimeout(() => {
        console.error('No pong received, connection may be dead');
        ws.close();
      }, 10000); // 10 seconds timeout
    }
  }, 30000); // 30 seconds
}

function stopHeartbeat() {
  clearInterval(heartbeatInterval);
  clearTimeout(connectionTimeout);
}

function resetConnectionTimeout() {
  clearTimeout(connectionTimeout);
}
```

### Server-Side Pong

The server responds to Ping messages with Pong messages containing the current timestamp.

**Code Location (Rust):** `rust-core-engine/src/api/mod.rs:486-534`
**Code Location (Python):** `python-ai-service/main.py:1736-1745`

---

## Error Handling

### Connection Errors

**Error Types:**
- Network timeout
- Server unavailable
- Authentication failure
- Protocol error

**Reconnection Strategy:**

```javascript
class WebSocketManager {
  constructor(url) {
    this.url = url;
    this.ws = null;
    this.reconnectAttempts = 0;
    this.maxReconnectAttempts = 10;
    this.reconnectDelay = 1000; // Start with 1 second
    this.maxReconnectDelay = 30000; // Max 30 seconds
  }

  connect() {
    this.ws = new WebSocket(this.url);

    this.ws.onopen = () => {
      console.log('✅ WebSocket connected');
      this.reconnectAttempts = 0;
      this.reconnectDelay = 1000;
      startHeartbeat(this.ws);
    };

    this.ws.onclose = (event) => {
      console.log('🔌 WebSocket closed:', event.code, event.reason);
      stopHeartbeat();
      this.attemptReconnect();
    };

    this.ws.onerror = (error) => {
      console.error('❌ WebSocket error:', error);
    };

    this.ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        this.handleMessage(message);
      } catch (error) {
        console.error('Failed to parse message:', error);
      }
    };
  }

  attemptReconnect() {
    if (this.reconnectAttempts >= this.maxReconnectAttempts) {
      console.error('Max reconnection attempts reached');
      return;
    }

    this.reconnectAttempts++;

    console.log(
      `Reconnecting... Attempt ${this.reconnectAttempts}/${this.maxReconnectAttempts}`
    );

    setTimeout(() => {
      this.connect();
    }, this.reconnectDelay);

    // Exponential backoff
    this.reconnectDelay = Math.min(
      this.reconnectDelay * 2,
      this.maxReconnectDelay
    );
  }

  handleMessage(message) {
    // Handle different message types
  }

  disconnect() {
    if (this.ws) {
      this.ws.close();
      stopHeartbeat();
    }
  }
}

// Usage
const wsManager = new WebSocketManager('ws://localhost:8080/ws');
wsManager.connect();
```

### Message Parsing Errors

**Error Handling:**
```javascript
ws.onmessage = (event) => {
  try {
    const message = JSON.parse(event.data);

    if (!message.type) {
      throw new Error('Message missing type field');
    }

    handleMessage(message);
  } catch (error) {
    console.error('Failed to parse WebSocket message:', error);
    console.error('Raw message:', event.data);

    // Optionally send error report to server
    reportMessageError(error, event.data);
  }
};
```

---

## Frontend Integration

### React Hook Example

```typescript
import { useEffect, useRef, useState } from 'react';

interface WebSocketMessage {
  type: string;
  data: any;
  timestamp: number;
}

export function useWebSocket(url: string) {
  const ws = useRef<WebSocket | null>(null);
  const [isConnected, setIsConnected] = useState(false);
  const [lastMessage, setLastMessage] = useState<WebSocketMessage | null>(null);

  useEffect(() => {
    ws.current = new WebSocket(url);

    ws.current.onopen = () => {
      console.log('WebSocket connected');
      setIsConnected(true);
    };

    ws.current.onclose = () => {
      console.log('WebSocket disconnected');
      setIsConnected(false);
    };

    ws.current.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        setLastMessage(message);
      } catch (error) {
        console.error('Failed to parse message:', error);
      }
    };

    ws.current.onerror = (error) => {
      console.error('WebSocket error:', error);
    };

    return () => {
      ws.current?.close();
    };
  }, [url]);

  const send = (data: any) => {
    if (ws.current?.readyState === WebSocket.OPEN) {
      ws.current.send(JSON.stringify(data));
    }
  };

  return { isConnected, lastMessage, send };
}

// Usage in component
function TradingDashboard() {
  const { isConnected, lastMessage, send } = useWebSocket('ws://localhost:8080/ws');

  useEffect(() => {
    if (lastMessage?.type === 'AISignalReceived') {
      handleAISignal(lastMessage.data);
    }
  }, [lastMessage]);

  return (
    <div>
      <div>Status: {isConnected ? 'Connected' : 'Disconnected'}</div>
      {/* Rest of component */}
    </div>
  );
}
```

**Code Location (Frontend):** `nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

---

## Security Considerations

### 1. Authentication

**JWT Token in Query Parameter (Future Enhancement):**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws?token=eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...');
```

**JWT Token in First Message (Current Approach):**
```javascript
ws.onopen = () => {
  ws.send(JSON.stringify({
    type: 'Authenticate',
    token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...'
  }));
};
```

### 2. Message Validation

**Server-Side Validation:**
- All incoming messages are validated
- Invalid messages are rejected
- Malformed JSON is handled gracefully

**Client-Side Validation:**
- Validate message structure before processing
- Sanitize data before rendering in UI
- Prevent XSS attacks from message content

### 3. Rate Limiting

**WebSocket Message Limits:**
- Client messages: 100 per minute
- Ping messages: 1 per 30 seconds minimum
- Excessive messages result in disconnection

### 4. Data Encryption

**Production Recommendations:**
- Use WSS (WebSocket Secure) with TLS/SSL
- Encrypt sensitive data in messages
- Implement message signing for critical operations

---

## Related Documentation

- [API-RUST-CORE.md](./API-RUST-CORE.md) - Rust Core Engine API
- [API-PYTHON-AI.md](./API-PYTHON-AI.md) - Python AI Service API
- [API-SEQUENCES.mermaid](./API-SEQUENCES.mermaid) - API Sequence Diagrams
- [Functional Requirements](/specs/01-requirements/1.2-functional/FUNCTIONAL_REQUIREMENTS.md)

---

**Document Version:** 1.0.0
**Last Updated:** 2025-10-10
**Author:** Claude Code
**Status:** Complete
