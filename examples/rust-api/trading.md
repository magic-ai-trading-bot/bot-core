# Rust Core Trading API Examples

## Authentication

### Login Request
```bash
curl -X POST http://localhost:8080/api/auth/login \
  -H "Content-Type: application/json" \
  -d '{
    "email": "trader@example.com",
    "password": "secure_password123"
  }'
```

### Login Response
```json
{
  "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
  "user": {
    "id": "507f1f77bcf86cd799439011",
    "email": "trader@example.com",
    "full_name": "John Trader",
    "is_active": true,
    "is_admin": false,
    "created_at": "2025-01-15T10:30:00Z",
    "last_login": "2025-07-31T18:00:00Z",
    "settings": {
      "trading_enabled": true,
      "risk_level": "medium",
      "max_positions": 3,
      "default_quantity": 0.01,
      "notifications": {
        "email_alerts": true,
        "trade_notifications": true,
        "system_alerts": true
      }
    }
  }
}
```

## Execute Trade

### Buy Order Request
```bash
curl -X POST http://localhost:8080/api/trades/execute \
  -H "Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9..." \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "side": "BUY",
    "type": "LIMIT",
    "quantity": 0.001,
    "price": 45000.00,
    "time_in_force": "GTC",
    "reduce_only": false,
    "ai_signal_id": "550e8400-e29b-41d4-a716-446655440000",
    "strategy": "ai_ensemble"
  }'
```

### Successful Trade Response
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

### Market Order Example
```bash
curl -X POST http://localhost:8080/api/trades/execute \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "ETHUSDT",
    "side": "SELL",
    "type": "MARKET",
    "quantity": 0.1,
    "reduce_only": true
  }'
```

### Stop Loss Order
```bash
curl -X POST http://localhost:8080/api/trades/execute \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "side": "SELL",
    "type": "STOP_LOSS_LIMIT",
    "quantity": 0.001,
    "price": 44500.00,
    "stop_price": 44600.00,
    "time_in_force": "GTC"
  }'
```

## Get Account Information

### Request
```bash
curl -X GET http://localhost:8080/api/account \
  -H "Authorization: Bearer {token}"
```

### Response
```json
{
  "balance": {
    "USDT": {
      "free": 9550.00,
      "locked": 450.00,
      "total": 10000.00
    },
    "BTC": {
      "free": 0.005,
      "locked": 0.001,
      "total": 0.006
    }
  },
  "total_balance_usdt": 10270.00,
  "total_pnl": 270.00,
  "total_pnl_percentage": 2.70,
  "positions_count": 2,
  "can_trade": true
}
```

## Get Open Positions

### Request
```bash
curl -X GET http://localhost:8080/api/positions \
  -H "Authorization: Bearer {token}"
```

### Response
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
      "liquidation_price": 40500.00,
      "stop_loss": 44100.00,
      "take_profit": 46350.00
    },
    {
      "symbol": "ETHUSDT",
      "side": "SHORT",
      "quantity": 0.1,
      "entry_price": 2520.00,
      "current_price": 2510.00,
      "pnl": 1.00,
      "pnl_percentage": 0.40,
      "margin_used": 25.20,
      "liquidation_price": 2772.00
    }
  ],
  "total_margin_used": 70.20,
  "free_margin": 9929.80,
  "margin_level": 14614.25
}
```

## Close Position

### Request
```bash
curl -X POST http://localhost:8080/api/positions/close \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "quantity": 0.001,
    "reason": "manual_close"
  }'
```

### Response
```json
{
  "position_id": "pos_123456",
  "symbol": "BTCUSDT",
  "closed_price": 45200.00,
  "realized_pnl": 0.20,
  "commission": 0.045,
  "close_timestamp": "2025-07-31T19:00:00.000Z"
}
```

## Trading History

### Request with Filters
```bash
curl -X GET "http://localhost:8080/api/trades/history?symbol=BTCUSDT&start_date=2025-07-01&end_date=2025-07-31&limit=10&page=1" \
  -H "Authorization: Bearer {token}"
```

### Response
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
      "timestamp": "2025-07-31T18:33:29.169981+00:00",
      "order_type": "LIMIT",
      "strategy": "ai_ensemble"
    },
    {
      "trade_id": "223e4567-e89b-12d3-a456-426614174001",
      "symbol": "BTCUSDT",
      "side": "SELL",
      "price": 45200.00,
      "quantity": 0.001,
      "realized_pnl": 0.20,
      "commission": 0.045,
      "timestamp": "2025-07-31T19:00:00.000000+00:00",
      "order_type": "MARKET",
      "close_reason": "manual_close"
    }
  ],
  "pagination": {
    "total": 150,
    "page": 1,
    "limit": 10,
    "pages": 15
  },
  "summary": {
    "total_trades": 150,
    "profitable_trades": 89,
    "losing_trades": 61,
    "total_pnl": 270.00,
    "total_commission": 6.75
  }
}
```

## WebSocket Subscription

### Connection and Subscribe
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  // Authenticate
  ws.send(JSON.stringify({
    type: 'auth',
    token: 'eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...'
  }));

  // Subscribe to updates
  ws.send(JSON.stringify({
    type: 'subscribe',
    channels: ['trades', 'positions', 'account']
  }));
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};
```

### Sample WebSocket Messages

#### Price Update
```json
{
  "type": "price_update",
  "data": {
    "symbol": "BTCUSDT",
    "price": 45250.00,
    "bid": 45249.50,
    "ask": 45250.50,
    "volume_24h": 25000000000,
    "change_24h": 2.5
  },
  "timestamp": "2025-07-31T19:05:00.000Z"
}
```

#### Trade Execution Update
```json
{
  "type": "trade_executed",
  "data": {
    "trade_id": "323e4567-e89b-12d3-a456-426614174002",
    "symbol": "ETHUSDT",
    "side": "BUY",
    "price": 2515.00,
    "quantity": 0.1,
    "status": "FILLED"
  },
  "timestamp": "2025-07-31T19:05:15.000Z"
}
```

#### Position Update
```json
{
  "type": "position_update",
  "data": {
    "symbol": "BTCUSDT",
    "current_price": 45300.00,
    "pnl": 0.30,
    "pnl_percentage": 0.67,
    "margin_level": 14700.00
  },
  "timestamp": "2025-07-31T19:05:30.000Z"
}
```

## Error Responses

### Insufficient Balance
```json
{
  "error": {
    "code": "INSUFFICIENT_BALANCE",
    "message": "Insufficient balance for this trade",
    "details": {
      "required": 100.00,
      "available": 50.00,
      "asset": "USDT"
    },
    "timestamp": "2025-07-31T19:10:00.000Z"
  }
}
```

### Invalid Order Parameters
```json
{
  "error": {
    "code": "INVALID_REQUEST",
    "message": "Order quantity below minimum",
    "details": {
      "provided": 0.0001,
      "minimum": 0.001,
      "symbol": "BTCUSDT"
    },
    "timestamp": "2025-07-31T19:11:00.000Z"
  }
}
```

### Unauthorized
```json
{
  "error": {
    "code": "UNAUTHORIZED",
    "message": "Invalid or expired token",
    "timestamp": "2025-07-31T19:12:00.000Z"
  }
}
```

## Health Check

### Request
```bash
curl -X GET http://localhost:8080/api/health
```

### Response
```json
{
  "status": "healthy",
  "version": "1.0.0",
  "uptime_seconds": 3600,
  "connected_services": {
    "python_ai": true,
    "binance_websocket": true,
    "mongodb": true
  },
  "system_status": {
    "trading_enabled": true,
    "paper_trading": false,
    "maintenance_mode": false
  }
}
```

## Advanced Trading Examples

### OCO (One-Cancels-Other) Order
```bash
curl -X POST http://localhost:8080/api/trades/oco \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "side": "SELL",
    "quantity": 0.001,
    "price": 46000.00,
    "stop_price": 44500.00,
    "stop_limit_price": 44400.00
  }'
```

### Trailing Stop Order
```bash
curl -X POST http://localhost:8080/api/trades/trailing-stop \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "side": "SELL",
    "quantity": 0.001,
    "callback_rate": 1.0
  }'
```

### Batch Orders
```bash
curl -X POST http://localhost:8080/api/trades/batch \
  -H "Authorization: Bearer {token}" \
  -H "Content-Type: application/json" \
  -d '{
    "orders": [
      {
        "symbol": "BTCUSDT",
        "side": "BUY",
        "type": "LIMIT",
        "quantity": 0.001,
        "price": 44800.00
      },
      {
        "symbol": "ETHUSDT",
        "side": "BUY",
        "type": "LIMIT",
        "quantity": 0.1,
        "price": 2480.00
      }
    ]
  }'
```