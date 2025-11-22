# API-RUST-CORE.md - Rust Core Engine API Specification

**Version:** 2.0.0
**Base URL:** `http://localhost:8080`
**Service:** Rust Core Engine
**Port:** 8080
**Total Endpoints:** 37

## Table of Contents

1. [Overview](#overview)
2. [Authentication](#authentication)
3. [Authentication Endpoints](#authentication-endpoints)
4. [Market Data Endpoints](#market-data-endpoints)
5. [Trading Endpoints](#trading-endpoints)
6. [Paper Trading Endpoints](#paper-trading-endpoints)
7. [Paper Trading - Advanced Features](#paper-trading---advanced-features)
8. [AI Integration Endpoints](#ai-integration-endpoints)
9. [Monitoring Endpoints](#monitoring-endpoints)
10. [WebSocket](#websocket)
11. [Error Codes](#error-codes)
12. [Rate Limiting](#rate-limiting)

---

## Overview

The Rust Core Engine API provides high-performance trading execution, market data management, risk management, and paper trading simulation capabilities. All endpoints follow RESTful conventions and return JSON responses.

**Service Architecture:**
- **Language:** Rust
- **Framework:** Warp (async web framework)
- **Database:** MongoDB
- **Authentication:** JWT Bearer tokens
- **WebSocket:** Real-time updates on `/ws`

**Code Location:** `/Users/dungngo97/Documents/bot-core/rust-core-engine/`

---

## Authentication

All protected endpoints require a JWT token in the Authorization header:

```
Authorization: Bearer <jwt_token>
```

**Token Structure:**
```json
{
  "sub": "user_id",
  "email": "user@example.com",
  "is_admin": false,
  "exp": 1701234567,
  "iat": 1701230967
}
```

**Token Expiration:** 7 days (604800 seconds)

---

## Authentication Endpoints

### POST /api/auth/register

**Description:** Register a new user account

**Authentication:** None required

**Request Body:**
```json
{
  "email": "trader@example.com",
  "password": "SecurePass123!",
  "full_name": "John Doe"
}
```

**Validation Rules:**
- `email`: Valid email format, required
- `password`: Minimum 8 characters, required
- `full_name`: Optional string

**Success Response (201 Created):**
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "507f1f77bcf86cd799439011",
      "email": "trader@example.com",
      "full_name": "John Doe",
      "is_admin": false,
      "is_active": true,
      "created_at": "2025-10-10T10:30:00Z",
      "updated_at": "2025-10-10T10:30:00Z"
    }
  }
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Validation failed",
  "details": "Email already registered"
}
```

**Error Response (409 Conflict):**
```json
{
  "success": false,
  "error": "Email already registered"
}
```

**Code Location:** `rust-core-engine/src/auth/handlers.rs:85-200`
**Related FR:** FR-AUTH-002
**Rate Limit:** 5 requests per hour per IP

---

### POST /api/auth/login

**Description:** Authenticate user and receive JWT token

**Authentication:** None required

**Request Body:**
```json
{
  "email": "trader@example.com",
  "password": "SecurePass123!"
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "token": "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...",
    "user": {
      "id": "507f1f77bcf86cd799439011",
      "email": "trader@example.com",
      "full_name": "John Doe",
      "is_admin": false,
      "is_active": true,
      "created_at": "2025-10-10T10:30:00Z",
      "updated_at": "2025-10-10T10:30:00Z",
      "last_login": "2025-10-10T10:30:00Z"
    }
  }
}
```

**Error Response (401 Unauthorized):**
```json
{
  "success": false,
  "error": "Invalid email or password"
}
```

**Error Response (403 Forbidden):**
```json
{
  "success": false,
  "error": "Account is deactivated"
}
```

**Code Location:** `rust-core-engine/src/auth/handlers.rs:202-324`
**Related FR:** FR-AUTH-001
**Rate Limit:** 10 requests per minute per IP

---

### GET /api/auth/verify

**Description:** Verify JWT token validity

**Authentication:** Required (Bearer JWT)

**Request Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "user_id": "507f1f77bcf86cd799439011",
    "email": "trader@example.com",
    "is_admin": false,
    "exp": 1701834567
  }
}
```

**Error Response (401 Unauthorized):**
```json
{
  "success": false,
  "error": "Invalid or expired token"
}
```

**Code Location:** `rust-core-engine/src/auth/handlers.rs:326-364`
**Related FR:** FR-AUTH-003
**Rate Limit:** 100 requests per minute

---

### GET /api/auth/profile

**Description:** Get authenticated user's profile

**Authentication:** Required (Bearer JWT)

**Request Headers:**
```
Authorization: Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9...
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "id": "507f1f77bcf86cd799439011",
    "email": "trader@example.com",
    "full_name": "John Doe",
    "is_admin": false,
    "is_active": true,
    "created_at": "2025-10-10T10:30:00Z",
    "updated_at": "2025-10-10T10:30:00Z",
    "last_login": "2025-10-10T10:30:00Z"
  }
}
```

**Error Response (404 Not Found):**
```json
{
  "success": false,
  "error": "User not found"
}
```

**Code Location:** `rust-core-engine/src/auth/handlers.rs:366-435`
**Related FR:** FR-AUTH-004
**Rate Limit:** 60 requests per minute

---

## Market Data Endpoints

### GET /api/market/prices

**Description:** Get latest prices for all tracked symbols

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "BTCUSDT": 67500.50,
    "ETHUSDT": 3800.25,
    "BNBUSDT": 625.75,
    "SOLUSDT": 185.40
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:187-201`
**Related FR:** FR-MARKET-001
**Rate Limit:** 100 requests per second

---

### GET /api/market/overview

**Description:** Get comprehensive market overview with statistics

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "total_symbols": 4,
    "active_symbols": 4,
    "total_volume_24h": 125000000000.50,
    "timestamp": 1697234567000,
    "market_status": "open",
    "symbols": {
      "BTCUSDT": {
        "price": 67500.50,
        "change_24h": 2.5,
        "volume_24h": 50000000000.0,
        "high_24h": 68000.00,
        "low_24h": 66500.00
      }
    }
  }
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to fetch market overview"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:204-217`
**Related FR:** FR-MARKET-002
**Rate Limit:** 10 requests per second

---

### GET /api/market/candles/:symbol/:timeframe?limit=100

**Description:** Get candlestick (OHLCV) data for a symbol and timeframe

**Authentication:** Optional

**Path Parameters:**
- `symbol`: Trading symbol (e.g., BTCUSDT)
- `timeframe`: Timeframe (e.g., 1m, 5m, 15m, 1h, 4h, 1d)

**Query Parameters:**
- `limit`: Number of candles to return (default: 100, max: 1000)

**Request Example:**
```
GET /api/market/candles/BTCUSDT/1h?limit=200
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "timestamp": 1697234400000,
      "open": 67400.00,
      "high": 67550.00,
      "low": 67350.00,
      "close": 67500.50,
      "volume": 1250.5,
      "close_time": 1697238000000,
      "quote_volume": 84378125.00,
      "trades": 15234,
      "taker_buy_base": 625.25,
      "taker_buy_quote": 42189062.50
    }
  ]
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:220-238`
**Related FR:** FR-MARKET-003
**Rate Limit:** 50 requests per second

---

### GET /api/market/chart/:symbol/:timeframe?limit=100

**Description:** Get comprehensive chart data with technical indicators

**Authentication:** Optional

**Path Parameters:**
- `symbol`: Trading symbol
- `timeframe`: Timeframe

**Query Parameters:**
- `limit`: Number of candles (default: 100)

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candles": [...],
    "indicators": {
      "rsi": 65.5,
      "macd": {
        "macd": 123.45,
        "signal": 110.23,
        "histogram": 13.22
      },
      "bollinger_bands": {
        "upper": 68000.00,
        "middle": 67500.00,
        "lower": 67000.00
      },
      "ema_9": 67450.00,
      "ema_21": 67300.00,
      "ema_50": 67000.00,
      "volume_sma": 1200.0,
      "atr": 234.56
    },
    "timestamp": 1697234567000
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:241-265`
**Related FR:** FR-MARKET-004
**Rate Limit:** 30 requests per second

---

### GET /api/market/charts?symbols=BTCUSDT,ETHUSDT&timeframes=1h,4h&limit=100

**Description:** Get chart data for multiple symbols and timeframes

**Authentication:** Optional

**Query Parameters:**
- `symbols`: Comma-separated list of symbols
- `timeframes`: Comma-separated list of timeframes
- `limit`: Number of candles per chart (optional)

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "BTCUSDT": {
      "1h": { "candles": [...], "indicators": {...} },
      "4h": { "candles": [...], "indicators": {...} }
    },
    "ETHUSDT": {
      "1h": { "candles": [...], "indicators": {...} },
      "4h": { "candles": [...], "indicators": {...} }
    }
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:268-298`
**Related FR:** FR-MARKET-005
**Rate Limit:** 20 requests per second

---

### GET /api/market/symbols

**Description:** Get all supported symbols and timeframes

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"],
    "available_timeframes": ["1m", "5m", "15m", "1h", "4h", "1d"]
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:343-354`
**Related FR:** FR-MARKET-006
**Rate Limit:** 100 requests per second

---

### POST /api/market/symbols

**Description:** Add new symbol to track

**Authentication:** Required (Admin)

**Request Body:**
```json
{
  "symbol": "ADAUSDT",
  "timeframes": ["1h", "4h", "1d"]
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Symbol added successfully"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to add symbol: <reason>"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:301-320`
**Related FR:** FR-MARKET-007
**Rate Limit:** 10 requests per minute

---

### DELETE /api/market/symbols/:symbol

**Description:** Remove symbol from tracking

**Authentication:** Required (Admin)

**Path Parameters:**
- `symbol`: Symbol to remove

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Symbol removed successfully"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:323-340`
**Related FR:** FR-MARKET-008
**Rate Limit:** 10 requests per minute

---

## Trading Endpoints

### GET /api/trading/positions

**Description:** Get all open positions

**Authentication:** Required

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
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
      "entry_time": 1697230000000
    }
  ]
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:372-378`
**Related FR:** FR-TRADE-001
**Rate Limit:** 50 requests per second

---

### GET /api/trading/account

**Description:** Get account balance and trading information

**Authentication:** Required

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "total_balance": 10000.00,
    "available_balance": 9500.00,
    "used_margin": 500.00,
    "unrealized_pnl": 50.00,
    "total_equity": 10050.00,
    "margin_level": 2010.00,
    "positions_count": 2,
    "can_trade": true,
    "max_leverage": 125,
    "account_type": "MARGIN"
  }
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to fetch account info"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:381-394`
**Related FR:** FR-TRADE-002
**Rate Limit:** 10 requests per second

---

### POST /api/trading/positions/:symbol/close

**Description:** Force close an open position

**Authentication:** Required

**Path Parameters:**
- `symbol`: Symbol of position to close

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Position closed"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to close position: <reason>"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:397-412`
**Related FR:** FR-TRADE-003
**Rate Limit:** 10 requests per second

---

### GET /api/trading/performance

**Description:** Get trading performance statistics

**Authentication:** Required

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "total_trades": 150,
    "winning_trades": 95,
    "losing_trades": 55,
    "win_rate": 0.6333,
    "total_pnl": 1250.50,
    "total_pnl_percent": 12.51,
    "average_win": 25.50,
    "average_loss": -15.25,
    "profit_factor": 1.67,
    "max_drawdown": -250.00,
    "max_drawdown_percent": -2.5,
    "sharpe_ratio": 1.85,
    "best_trade": 150.00,
    "worst_trade": -75.00,
    "average_trade_duration_minutes": 240
  }
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to fetch performance stats"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:415-428`
**Related FR:** FR-TRADE-004
**Rate Limit:** 10 requests per second

---

## Paper Trading Endpoints

### GET /api/paper-trading/status

**Description:** Get paper trading engine status

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "is_running": true,
    "initial_balance": 10000.00,
    "current_balance": 10500.50,
    "total_pnl": 500.50,
    "total_pnl_percent": 5.005,
    "open_positions": 2,
    "total_trades": 25,
    "winning_trades": 18,
    "losing_trades": 7,
    "win_rate": 0.72,
    "confidence_threshold": 0.65,
    "enabled_strategies": ["RSI Strategy", "MACD Strategy", "Bollinger Bands Strategy"],
    "tracked_symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"],
    "last_update": 1697234567000
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:193-196`
**Related FR:** FR-PAPER-001
**Rate Limit:** 10 requests per second

---

### GET /api/paper-trading/portfolio

**Description:** Get detailed portfolio information

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "initial_balance": 10000.00,
    "current_balance": 10500.50,
    "available_balance": 10200.00,
    "used_margin": 300.50,
    "total_pnl": 500.50,
    "total_pnl_percent": 5.005,
    "unrealized_pnl": 50.00,
    "realized_pnl": 450.50,
    "positions": [
      {
        "symbol": "BTCUSDT",
        "side": "LONG",
        "quantity": 0.001,
        "entry_price": 67000.00,
        "current_price": 67500.00,
        "unrealized_pnl": 0.50,
        "unrealized_pnl_percent": 0.75,
        "entry_time": "2025-10-10T10:30:00Z"
      }
    ],
    "last_updated": "2025-10-10T10:35:00Z"
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:201-204`
**Related FR:** FR-PAPER-002
**Rate Limit:** 10 requests per second

---

### GET /api/paper-trading/trades/open

**Description:** Get all open paper trades

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "trade_123456",
      "symbol": "BTCUSDT",
      "side": "LONG",
      "quantity": 0.001,
      "entry_price": 67000.00,
      "current_price": 67500.00,
      "stop_loss": 66000.00,
      "take_profit": 68000.00,
      "unrealized_pnl": 0.50,
      "unrealized_pnl_percent": 0.75,
      "entry_time": "2025-10-10T10:30:00Z",
      "strategy": "RSI Strategy",
      "confidence": 0.75,
      "ai_signal": {
        "signal": "LONG",
        "confidence": 0.75,
        "reasoning": "RSI oversold, MACD bullish crossover"
      }
    }
  ]
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:209-212`
**Related FR:** FR-PAPER-003
**Rate Limit:** 10 requests per second

---

### GET /api/paper-trading/trades/closed

**Description:** Get all closed paper trades

**Authentication:** Optional

**Query Parameters:**
- `limit`: Number of trades to return (default: 100)
- `offset`: Offset for pagination (default: 0)

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "id": "trade_123455",
      "symbol": "ETHUSDT",
      "side": "LONG",
      "quantity": 0.01,
      "entry_price": 3800.00,
      "exit_price": 3850.00,
      "realized_pnl": 0.50,
      "realized_pnl_percent": 1.32,
      "entry_time": "2025-10-10T09:00:00Z",
      "exit_time": "2025-10-10T10:00:00Z",
      "duration_minutes": 60,
      "strategy": "MACD Strategy",
      "exit_reason": "TAKE_PROFIT",
      "confidence": 0.68
    }
  ],
  "meta": {
    "total": 23,
    "limit": 100,
    "offset": 0
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:218-221`
**Related FR:** FR-PAPER-004
**Rate Limit:** 10 requests per second

---

### POST /api/paper-trading/trades/:symbol/close

**Description:** Manually close a paper trading position

**Authentication:** Optional

**Path Parameters:**
- `symbol`: Symbol of trade to close

**Request Body (Optional):**
```json
{
  "reason": "Manual close"
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "trade_id": "trade_123456",
    "symbol": "BTCUSDT",
    "closed_at": "2025-10-10T10:35:00Z",
    "exit_price": 67500.00,
    "realized_pnl": 0.50,
    "realized_pnl_percent": 0.75
  }
}
```

**Error Response (404 Not Found):**
```json
{
  "success": false,
  "error": "No open position found for BTCUSDT"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:227-232`
**Related FR:** FR-PAPER-005
**Rate Limit:** 10 requests per second

---

### GET /api/paper-trading/settings

**Description:** Get all paper trading settings

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "basic_settings": {
      "initial_balance": 10000.00,
      "leverage": 10,
      "default_position_size_percent": 5.0,
      "max_positions": 4,
      "confidence_threshold": 0.65,
      "enable_stop_loss": true,
      "enable_take_profit": true,
      "default_stop_loss_percent": 2.0,
      "default_take_profit_percent": 4.0
    },
    "strategy_settings": {
      "enabled_strategies": ["RSI Strategy", "MACD Strategy", "Bollinger Bands Strategy"],
      "strategy_weights": {
        "RSI Strategy": 1.0,
        "MACD Strategy": 1.0,
        "Bollinger Bands Strategy": 0.8
      }
    },
    "symbol_settings": {
      "BTCUSDT": {
        "enabled": true,
        "leverage": 10,
        "position_size_pct": 5.0,
        "stop_loss_pct": 2.0,
        "take_profit_pct": 4.0
      }
    }
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:238-241`
**Related FR:** FR-PAPER-006
**Rate Limit:** 10 requests per second

---

### GET /api/paper-trading/strategy-settings

**Description:** Get strategy-specific settings

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "enabled_strategies": ["RSI Strategy", "MACD Strategy", "Bollinger Bands Strategy"],
    "strategy_weights": {
      "RSI Strategy": 1.0,
      "MACD Strategy": 1.0,
      "Bollinger Bands Strategy": 0.8,
      "Volume Strategy": 0.5
    },
    "strategy_parameters": {
      "RSI Strategy": {
        "period": 14,
        "oversold": 30,
        "overbought": 70
      },
      "MACD Strategy": {
        "fast_period": 12,
        "slow_period": 26,
        "signal_period": 9
      }
    }
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:247-250`
**Related FR:** FR-PAPER-007
**Rate Limit:** 10 requests per second

---

### PUT /api/paper-trading/strategy-settings

**Description:** Update strategy settings

**Authentication:** Optional

**Request Body:**
```json
{
  "enabled_strategies": ["RSI Strategy", "MACD Strategy"],
  "strategy_weights": {
    "RSI Strategy": 1.0,
    "MACD Strategy": 0.8
  }
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Strategy settings updated successfully"
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Invalid strategy name: InvalidStrategy"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:255-258`
**Related FR:** FR-PAPER-008
**Rate Limit:** 10 requests per minute

---

### GET /api/paper-trading/basic-settings

**Description:** Get basic trading settings

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "initial_balance": 10000.00,
    "leverage": 10,
    "default_position_size_percent": 5.0,
    "max_positions": 4,
    "confidence_threshold": 0.65,
    "enable_stop_loss": true,
    "enable_take_profit": true,
    "default_stop_loss_percent": 2.0,
    "default_take_profit_percent": 4.0,
    "min_trade_interval_minutes": 5,
    "max_daily_loss_percent": 10.0
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:264-267`
**Related FR:** FR-PAPER-009
**Rate Limit:** 10 requests per second

---

### PUT /api/paper-trading/basic-settings

**Description:** Update basic trading settings

**Authentication:** Optional

**Request Body:**
```json
{
  "leverage": 15,
  "default_position_size_percent": 3.0,
  "confidence_threshold": 0.70,
  "default_stop_loss_percent": 1.5,
  "default_take_profit_percent": 3.0
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Basic settings updated successfully"
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Invalid leverage: must be between 1 and 125"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:272-275`
**Related FR:** FR-PAPER-010
**Rate Limit:** 10 requests per minute

---

### GET /api/paper-trading/symbols

**Description:** Get symbol-specific settings

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "BTCUSDT": {
      "enabled": true,
      "leverage": 10,
      "position_size_pct": 5.0,
      "stop_loss_pct": 2.0,
      "take_profit_pct": 4.0,
      "max_positions": 1
    },
    "ETHUSDT": {
      "enabled": true,
      "leverage": 10,
      "position_size_pct": 5.0,
      "stop_loss_pct": 2.0,
      "take_profit_pct": 4.0,
      "max_positions": 1
    }
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:281-284`
**Related FR:** FR-PAPER-011
**Rate Limit:** 10 requests per second

---

### PUT /api/paper-trading/symbols

**Description:** Update symbol-specific settings

**Authentication:** Optional

**Request Body:**
```json
{
  "BTCUSDT": {
    "enabled": true,
    "leverage": 15,
    "position_size_pct": 3.0,
    "stop_loss_pct": 1.5,
    "take_profit_pct": 3.0,
    "max_positions": 2
  }
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Symbol settings updated successfully"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:289-292`
**Related FR:** FR-PAPER-012
**Rate Limit:** 10 requests per minute

---

### POST /api/paper-trading/reset

**Description:** Reset paper trading account to initial state

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "message": "Paper trading reset successfully",
    "initial_balance": 10000.00,
    "all_positions_closed": true,
    "trade_history_cleared": false
  }
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:298-301`
**Related FR:** FR-PAPER-013
**Rate Limit:** 1 request per minute

---

### POST /api/paper-trading/start

**Description:** Start paper trading engine

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Paper trading started successfully"
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Paper trading already running"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:306-309`
**Related FR:** FR-PAPER-014
**Rate Limit:** 10 requests per minute

---

### POST /api/paper-trading/stop

**Description:** Stop paper trading engine

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Paper trading stopped successfully"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:314-317`
**Related FR:** FR-PAPER-015
**Rate Limit:** 10 requests per minute

---

### POST /api/paper-trading/trigger-analysis

**Description:** Manually trigger AI analysis for all symbols

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "AI analysis triggered for 4 symbols"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:322-325`
**Related FR:** FR-PAPER-016
**Rate Limit:** 5 requests per minute

---

### PUT /api/paper-trading/signal-interval

**Description:** Update signal generation interval

**Authentication:** Optional

**Request Body:**
```json
{
  "interval_minutes": 10
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Signal interval updated to 10 minutes"
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Interval must be between 1 and 60 minutes"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs:330-333`
**Related FR:** FR-PAPER-017
**Rate Limit:** 10 requests per minute

---

## Paper Trading - Advanced Features

### Trailing Stop Management

#### GET /api/paper-trading/trailing-stops

**Description:** Get trailing stop configuration and current status for all positions

**Authentication:** Optional

**Request:**
```http
GET /api/paper-trading/trailing-stops HTTP/1.1
Authorization: Bearer <jwt_token>
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "global_settings": {
      "enabled": true,
      "trailing_pct": 1.5,
      "activation_pct": 2.0
    },
    "active_trailing_stops": [
      {
        "position_id": "pos_12345",
        "symbol": "BTCUSDT",
        "direction": "LONG",
        "entry_price": 50000.0,
        "current_price": 53000.0,
        "highest_price": 53500.0,
        "trailing_stop_price": 52705.0,
        "profit_pct": 6.0,
        "activated_at": "2025-11-22T14:30:00Z",
        "updates_count": 15
      }
    ],
    "inactive_positions": []
  }
}
```

**Error Response (401 Unauthorized):**
```json
{
  "success": false,
  "error": "Invalid or expired token"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to fetch trailing stops"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs`
**Related FR:** FR-PAPER-018
**Rate Limit:** 10 requests per second

---

#### PUT /api/paper-trading/trailing-stops/settings

**Description:** Update global trailing stop settings

**Authentication:** Optional

**Request:**
```http
PUT /api/paper-trading/trailing-stops/settings HTTP/1.1
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "enabled": true,
  "trailing_pct": 1.8,
  "activation_pct": 2.5
}
```

**Validation Rules:**
- `enabled`: boolean (required)
- `trailing_pct`: float, 0.5-5.0 (required)
- `activation_pct`: float, 0.5-10.0 (required)
- `activation_pct` must be >= `trailing_pct`

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "settings": {
      "enabled": true,
      "trailing_pct": 1.8,
      "activation_pct": 2.5
    },
    "affected_positions": 3,
    "message": "Trailing stop settings updated. 3 positions affected."
  }
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Validation error: activation_pct must be >= trailing_pct"
}
```

**Error Response (401 Unauthorized):**
```json
{
  "success": false,
  "error": "Invalid or expired token"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to update trailing stop settings"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs`
**Related FR:** FR-PAPER-019
**Rate Limit:** 100 requests per minute

---

#### POST /api/paper-trading/positions/:id/trailing-stop/manual-adjust

**Description:** Manually adjust trailing stop for a specific position

**Authentication:** Optional

**Path Parameters:**
- `id`: Position ID

**Request:**
```http
POST /api/paper-trading/positions/pos_12345/trailing-stop/manual-adjust HTTP/1.1
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "new_stop_price": 52500.0,
  "reason": "Manual risk adjustment"
}
```

**Validation Rules:**
- `new_stop_price`: float (required), must be reasonable relative to current price
- `reason`: string (optional)

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "position_id": "pos_12345",
    "old_stop_price": 52705.0,
    "new_stop_price": 52500.0,
    "distance_from_current": 0.94,
    "message": "Trailing stop manually adjusted"
  }
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Invalid stop price: must be below current price for LONG positions"
}
```

**Error Response (404 Not Found):**
```json
{
  "success": false,
  "error": "Position not found: pos_12345"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to adjust trailing stop"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs`
**Related FR:** FR-PAPER-020
**Rate Limit:** 100 requests per minute

---

### AI Signal Processing

#### POST /api/paper-trading/process-ai-signal

**Description:** Process external AI signal from frontend (triggers GPT-4 analysis)

**Authentication:** Optional

**Request:**
```http
POST /api/paper-trading/process-ai-signal HTTP/1.1
Authorization: Bearer <jwt_token>
Content-Type: application/json

{
  "symbol": "BTCUSDT",
  "timeframe": "15m",
  "trigger_source": "frontend_button",
  "user_id": "user_123"
}
```

**Validation Rules:**
- `symbol`: string (required), must be a tracked symbol
- `timeframe`: string (required), must be one of: 1m, 5m, 15m, 1h, 4h, 1d
- `trigger_source`: string (optional), default: "manual"
- `user_id`: string (optional)

**Success Response (202 Accepted):**
```json
{
  "success": true,
  "data": {
    "message": "AI signal analysis queued",
    "analysis_id": "ai_analysis_20251122_143000",
    "estimated_completion_seconds": 15,
    "status_endpoint": "/api/ai-analysis/status/ai_analysis_20251122_143000"
  }
}
```

**Error Response (400 Bad Request - Analysis In Progress):**
```json
{
  "success": false,
  "error": "AI_ANALYSIS_IN_PROGRESS",
  "message": "AI analysis already in progress for BTCUSDT. Please wait.",
  "retry_after_seconds": 10
}
```

**Error Response (400 Bad Request - Invalid Symbol):**
```json
{
  "success": false,
  "error": "Invalid symbol: INVALIDUSDT"
}
```

**Error Response (429 Too Many Requests):**
```json
{
  "success": false,
  "error": "RATE_LIMIT_EXCEEDED",
  "message": "Max 6 AI analyses per hour exceeded. Try again later.",
  "retry_after_seconds": 600
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to queue AI signal analysis"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs`
**Related FR:** FR-PAPER-021
**Rate Limit:** 6 requests per hour

---

### Data Resolution Management

#### GET /api/paper-trading/data-resolutions

**Description:** Get available data resolutions and current setting

**Authentication:** Optional

**Request:**
```http
GET /api/paper-trading/data-resolutions HTTP/1.1
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "available_resolutions": ["1m", "5m", "15m", "1h", "4h", "1d"],
    "current_resolution": "15m",
    "recommended": "15m",
    "reason": "Optimal for crypto day trading"
  }
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to fetch data resolutions"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs`
**Related FR:** FR-PAPER-022
**Rate Limit:** 10 requests per second

---

#### PUT /api/paper-trading/data-resolution

**Description:** Change data resolution for trading

**Authentication:** Optional

**Request:**
```http
PUT /api/paper-trading/data-resolution HTTP/1.1
Content-Type: application/json

{
  "resolution": "15m"
}
```

**Validation Rules:**
- `resolution`: string (required), must be one of: 1m, 5m, 15m, 1h, 4h, 1d

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "old_resolution": "1h",
    "new_resolution": "15m",
    "requires_restart": false,
    "warmup_required": true,
    "estimated_warmup_minutes": 1,
    "message": "Data resolution updated to 15m. Warmup in progress."
  }
}
```

**Error Response (400 Bad Request):**
```json
{
  "success": false,
  "error": "Invalid resolution: 3m. Must be one of: 1m, 5m, 15m, 1h, 4h, 1d"
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to update data resolution"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs`
**Related FR:** FR-PAPER-023
**Rate Limit:** 10 requests per minute

---

### Position Correlation Analysis

#### GET /api/paper-trading/correlation-analysis

**Description:** Get current position correlation analysis

**Authentication:** Optional

**Request:**
```http
GET /api/paper-trading/correlation-analysis HTTP/1.1
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "total_positions": 5,
    "long_positions": 3,
    "short_positions": 2,
    "directional_correlation": 0.68,
    "correlation_limit": 0.70,
    "within_limit": true,
    "can_open_long": true,
    "can_open_short": true,
    "positions": [
      {
        "symbol": "BTCUSDT",
        "direction": "LONG",
        "correlation_with_others": 0.85
      },
      {
        "symbol": "ETHUSDT",
        "direction": "LONG",
        "correlation_with_others": 0.72
      },
      {
        "symbol": "BNBUSDT",
        "direction": "LONG",
        "correlation_with_others": 0.65
      },
      {
        "symbol": "SOLUSDT",
        "direction": "SHORT",
        "correlation_with_others": -0.45
      },
      {
        "symbol": "ADAUSDT",
        "direction": "SHORT",
        "correlation_with_others": -0.38
      }
    ],
    "warnings": []
  }
}
```

**Success Response (200 OK - With Warnings):**
```json
{
  "success": true,
  "data": {
    "total_positions": 4,
    "long_positions": 4,
    "short_positions": 0,
    "directional_correlation": 0.85,
    "correlation_limit": 0.70,
    "within_limit": false,
    "can_open_long": false,
    "can_open_short": true,
    "positions": [
      {
        "symbol": "BTCUSDT",
        "direction": "LONG",
        "correlation_with_others": 0.95
      },
      {
        "symbol": "ETHUSDT",
        "direction": "LONG",
        "correlation_with_others": 0.88
      },
      {
        "symbol": "BNBUSDT",
        "direction": "LONG",
        "correlation_with_others": 0.82
      },
      {
        "symbol": "SOLUSDT",
        "direction": "LONG",
        "correlation_with_others": 0.75
      }
    ],
    "warnings": [
      "High directional correlation (85%). Opening new LONG positions is restricted.",
      "Consider closing some correlated positions or opening SHORT positions for balance."
    ]
  }
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "Failed to calculate correlation analysis"
}
```

**Code Location:** `rust-core-engine/src/api/paper_trading.rs`
**Related FR:** FR-PAPER-024
**Rate Limit:** 10 requests per second

---

## AI Integration Endpoints

### POST /api/ai/analyze

**Description:** Request AI analysis for trading signals

**Authentication:** Optional

**Request Body:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe_data": {
    "1h": [
      {
        "timestamp": 1697234400000,
        "open": 67400.00,
        "high": 67550.00,
        "low": 67350.00,
        "close": 67500.50,
        "volume": 1250.5
      }
    ]
  },
  "current_price": 67500.50,
  "volume_24h": 50000000000.0,
  "timestamp": 1697234567000,
  "strategy_context": {
    "selected_strategies": ["RSI Strategy", "MACD Strategy"],
    "market_condition": "Trending",
    "risk_level": "Moderate"
  }
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "signal": "LONG",
    "confidence": 0.75,
    "reasoning": "Strong bullish momentum with RSI showing strength without being overbought. MACD histogram positive and increasing.",
    "strategy_scores": {
      "RSI Strategy": 0.80,
      "MACD Strategy": 0.75,
      "Bollinger Bands Strategy": 0.65,
      "Volume Strategy": 0.70
    },
    "market_analysis": {
      "trend_direction": "Bullish",
      "trend_strength": 0.75,
      "support_levels": [67000.00, 66500.00],
      "resistance_levels": [68000.00, 68500.00],
      "volatility_level": "Medium",
      "volume_analysis": "High volume confirming uptrend"
    },
    "risk_assessment": {
      "overall_risk": "Medium",
      "technical_risk": 0.45,
      "market_risk": 0.50,
      "recommended_position_size": 0.05,
      "stop_loss_suggestion": 66000.00,
      "take_profit_suggestion": 68500.00
    },
    "timestamp": 1697234567000
  }
}
```

**Error Response (500 Internal Server Error):**
```json
{
  "success": false,
  "error": "AI service unavailable"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:551-588`
**Related FR:** FR-AI-001
**Rate Limit:** 10 requests per minute

---

### POST /api/ai/strategy-recommendations

**Description:** Get AI-powered strategy recommendations

**Authentication:** Optional

**Request Body:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe_data": {
    "1h": [...]
  },
  "current_price": 67500.00,
  "timestamp": 1697234567000,
  "available_strategies": ["RSI Strategy", "MACD Strategy", "Bollinger Bands Strategy"]
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": [
    {
      "strategy_name": "RSI Strategy",
      "suitability_score": 0.85,
      "reasoning": "Current RSI indicates oversold conditions, suitable for mean reversion",
      "recommended_config": {
        "enabled": true,
        "weight": 1.0
      }
    },
    {
      "strategy_name": "MACD Strategy",
      "suitability_score": 0.78,
      "reasoning": "MACD showing bullish crossover, momentum strategy suitable",
      "recommended_config": {
        "enabled": true,
        "weight": 0.9
      }
    }
  ]
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:591-612`
**Related FR:** FR-AI-002
**Rate Limit:** 5 requests per minute

---

### POST /api/ai/market-condition

**Description:** Analyze current market conditions using AI

**Authentication:** Optional

**Request Body:**
```json
{
  "symbol": "BTCUSDT",
  "timeframe_data": {
    "1h": [...],
    "4h": [...]
  },
  "current_price": 67500.00,
  "volume_24h": 50000000000.0,
  "timestamp": 1697234567000
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "condition_type": "Trending Up",
    "confidence": 0.80,
    "characteristics": ["Strong uptrend", "High momentum", "Increasing volume"],
    "recommended_strategies": ["RSI Strategy", "MACD Strategy"],
    "market_phase": "Expansion"
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:615-634`
**Related FR:** FR-AI-003
**Rate Limit:** 5 requests per minute

---

### POST /api/ai/feedback

**Description:** Send performance feedback to AI service

**Authentication:** Optional

**Request Body:**
```json
{
  "signal_id": "signal_123456",
  "symbol": "BTCUSDT",
  "predicted_signal": "LONG",
  "actual_outcome": "PROFIT",
  "profit_loss": 50.00,
  "confidence_was_accurate": true,
  "feedback_notes": "Good signal timing",
  "timestamp": 1697234567000
}
```

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": "Feedback sent successfully"
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:637-653`
**Related FR:** FR-AI-004
**Rate Limit:** 20 requests per minute

---

### GET /api/ai/info

**Description:** Get AI service information and capabilities

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "service_name": "GPT-4 Trading AI",
    "version": "2.0.0",
    "model_version": "gpt-4o-mini",
    "supported_timeframes": ["1m", "5m", "15m", "1h", "4h", "1d"],
    "supported_symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"],
    "capabilities": [
      "trend_analysis",
      "signal_generation",
      "risk_assessment",
      "strategy_recommendation",
      "market_condition_detection"
    ],
    "last_trained": null
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:657-670`
**Related FR:** FR-AI-005
**Rate Limit:** 60 requests per minute

---

### GET /api/ai/strategies

**Description:** Get list of supported AI strategies

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": [
    "RSI Strategy",
    "MACD Strategy",
    "Volume Strategy",
    "Bollinger Bands Strategy"
  ]
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:673-686`
**Related FR:** FR-AI-006
**Rate Limit:** 60 requests per minute

---

## Monitoring Endpoints

### GET /api/monitoring/system

**Description:** Get system metrics and health status

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "cpu_usage_percent": 25.5,
    "memory_usage_mb": 512.0,
    "memory_usage_percent": 32.0,
    "active_positions": 2,
    "cache_size": 1250,
    "uptime_seconds": 3600,
    "last_update": 1697234567000
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:439-446`
**Related FR:** FR-MONITOR-001
**Rate Limit:** 10 requests per second

---

### GET /api/monitoring/trading

**Description:** Get trading-specific metrics

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "total_trades_today": 15,
    "successful_trades_today": 11,
    "failed_trades_today": 4,
    "average_execution_time_ms": 125.5,
    "total_volume_traded_24h": 150000.00,
    "api_calls_per_minute": 25,
    "websocket_messages_per_second": 10
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:449-457`
**Related FR:** FR-MONITOR-002
**Rate Limit:** 10 requests per second

---

### GET /api/monitoring/connection

**Description:** Get connection status for external services

**Authentication:** Optional

**Success Response (200 OK):**
```json
{
  "success": true,
  "data": {
    "binance_websocket": {
      "connected": true,
      "last_heartbeat": 1697234567000,
      "reconnect_count": 0
    },
    "python_ai_service": {
      "connected": true,
      "last_request": 1697234500000,
      "response_time_ms": 250
    },
    "mongodb": {
      "connected": true,
      "last_query": 1697234550000,
      "response_time_ms": 15
    },
    "api_responsive": true,
    "websocket_clients": 3
  }
}
```

**Code Location:** `rust-core-engine/src/api/mod.rs:460-468`
**Related FR:** FR-MONITOR-003
**Rate Limit:** 10 requests per second

---

## WebSocket

### WS /ws

**Description:** WebSocket endpoint for real-time updates

**Connection URL:** `ws://localhost:8080/ws`

**Connection Example:**
```javascript
const ws = new WebSocket('ws://localhost:8080/ws');

ws.onopen = () => {
  console.log('WebSocket connected');
};

ws.onmessage = (event) => {
  const message = JSON.parse(event.data);
  console.log('Received:', message);
};

ws.onerror = (error) => {
  console.error('WebSocket error:', error);
};

ws.onclose = () => {
  console.log('WebSocket disconnected');
};
```

**Message Types:**

#### price_update
Real-time price updates for tracked symbols.

```json
{
  "event": "price_update",
  "data": {
    "symbol": "BTCUSDT",
    "price": 67500.50,
    "timestamp": 1697234567000
  }
}
```

#### signal_generated
New trading signal generated by AI or strategies.

```json
{
  "event": "signal_generated",
  "data": {
    "symbol": "BTCUSDT",
    "signal": "LONG",
    "confidence": 0.75,
    "strategy": "RSI Strategy",
    "timestamp": 1697234567000
  }
}
```

#### trade_executed
Paper trade executed.

```json
{
  "event": "trade_executed",
  "data": {
    "trade_id": "trade_123456",
    "symbol": "BTCUSDT",
    "side": "LONG",
    "entry_price": 67500.00,
    "quantity": 0.001,
    "timestamp": 1697234567000
  }
}
```

#### portfolio_update
Portfolio balance or position update.

```json
{
  "event": "portfolio_update",
  "data": {
    "current_balance": 10500.50,
    "total_pnl": 500.50,
    "open_positions": 2,
    "timestamp": 1697234567000
  }
}
```

#### risk_event
Risk management event (daily loss limit, cool-down, etc.).

```json
{
  "event": "risk_event",
  "data": {
    "type": "COOLDOWN_ACTIVATED",
    "message": "Trading paused: 5 consecutive losses",
    "cooldown_until": 1697238167000,
    "timestamp": 1697234567000
  }
}
```

#### trailing_stop_updated
Trailing stop price updated for a position.

```json
{
  "event": "trailing_stop_updated",
  "data": {
    "position_id": "pos_12345",
    "symbol": "BTCUSDT",
    "new_stop_price": 52705.0,
    "profit_pct": 5.4,
    "timestamp": 1697234567000
  }
}
```

#### ai_signal_completed
AI signal analysis completed (GPT-4 analysis).

```json
{
  "event": "ai_signal_completed",
  "data": {
    "analysis_id": "ai_analysis_20251122_143000",
    "symbol": "BTCUSDT",
    "signal": "BUY",
    "confidence": 0.85,
    "reasoning": "Strong bullish momentum...",
    "timestamp": 1697234567000
  }
}
```

See [API-WEBSOCKET.md](./API-WEBSOCKET.md) for complete WebSocket protocol documentation.

**Code Location:** `rust-core-engine/src/api/mod.rs:486-534`
**Related FR:** FR-WS-001

---

## Error Codes

All error responses follow this structure:

```json
{
  "success": false,
  "error": "Error message describing the issue"
}
```

### HTTP Status Codes

| Status Code | Meaning | Usage |
|-------------|---------|-------|
| 200 | OK | Successful GET, PUT, POST request |
| 201 | Created | Successful resource creation |
| 202 | Accepted | Request accepted for async processing |
| 400 | Bad Request | Invalid request parameters or validation error |
| 401 | Unauthorized | Missing or invalid authentication token |
| 403 | Forbidden | User doesn't have required permissions |
| 404 | Not Found | Resource not found |
| 409 | Conflict | Resource conflict (e.g., duplicate email) |
| 429 | Too Many Requests | Rate limit exceeded |
| 500 | Internal Server Error | Server-side error |
| 503 | Service Unavailable | Service temporarily unavailable |

### Common Error Messages

| Error Code | Message | Description |
|------------|---------|-------------|
| VALIDATION_ERROR | Validation failed | Request body validation failed |
| INVALID_TOKEN | Invalid or expired token | JWT token is invalid or expired |
| EMAIL_EXISTS | Email already registered | Registration failed due to duplicate email |
| INVALID_CREDENTIALS | Invalid email or password | Login failed due to incorrect credentials |
| ACCOUNT_DEACTIVATED | Account is deactivated | User account is not active |
| USER_NOT_FOUND | User not found | User doesn't exist in database |
| POSITION_NOT_FOUND | No open position found | Trading position doesn't exist |
| INSUFFICIENT_BALANCE | Insufficient balance | Not enough balance for operation |
| RATE_LIMIT_EXCEEDED | Too many requests | API rate limit exceeded |
| AI_SERVICE_ERROR | AI service unavailable | Python AI service is down or unreachable |
| AI_ANALYSIS_IN_PROGRESS | AI analysis already in progress | Duplicate AI analysis request |
| MARKET_DATA_ERROR | Failed to fetch market data | Market data retrieval failed |

---

## Rate Limiting

All endpoints have rate limits to prevent abuse and ensure fair usage.

### Rate Limit Headers

Every response includes rate limit information:

```
X-RateLimit-Limit: 60
X-RateLimit-Remaining: 45
X-RateLimit-Reset: 1697234567
```

### Rate Limit Tiers

| Endpoint Category | Limit | Window |
|------------------|-------|--------|
| Authentication (register) | 5 requests | 1 hour |
| Authentication (login) | 10 requests | 1 minute |
| Authentication (other) | 60 requests | 1 minute |
| Market Data (read) | 100 requests | 1 second |
| Market Data (write) | 10 requests | 1 minute |
| Trading | 10 requests | 1 second |
| Paper Trading (read) | 10 requests | 1 second |
| Paper Trading (write) | 10 requests | 1 minute |
| AI Analysis | 10 requests | 1 minute |
| AI Signal Processing | 6 requests | 1 hour |
| Trailing Stop Updates | 100 requests | 1 minute |
| Monitoring | 10 requests | 1 second |

### Rate Limit Exceeded Response

When rate limit is exceeded:

**Status:** 429 Too Many Requests

```json
{
  "success": false,
  "error": "Rate limit exceeded. Please try again in 30 seconds.",
  "retry_after": 30
}
```

---

## Versioning

**Current Version:** 2.0.0

API versioning is not currently implemented. Future versions will use URL-based versioning:

```
/api/v2/market/prices
```

---

## CORS Policy

CORS is enabled for the following origins:
- `http://localhost:3000` (Frontend Dashboard)
- All origins in development mode

**Allowed Methods:** GET, POST, PUT, DELETE, OPTIONS
**Allowed Headers:** content-type, x-client, authorization, accept

---

## Code References

| Component | File Path | Description |
|-----------|-----------|-------------|
| API Server | `rust-core-engine/src/api/mod.rs` | Main API server implementation |
| Auth Handlers | `rust-core-engine/src/auth/handlers.rs` | Authentication endpoints |
| Paper Trading API | `rust-core-engine/src/api/paper_trading.rs` | Paper trading endpoints |
| Trading Engine | `rust-core-engine/src/trading/engine.rs` | Trading execution logic |
| Market Data | `rust-core-engine/src/market_data/mod.rs` | Market data processing |
| AI Service | `rust-core-engine/src/ai/mod.rs` | AI integration client |

---

## Related Documentation

- [API-PYTHON-AI.md](./API-PYTHON-AI.md) - Python AI Service API
- [API-WEBSOCKET.md](./API-WEBSOCKET.md) - WebSocket Protocol
- [API-SEQUENCES.mermaid](./API-SEQUENCES.mermaid) - API Sequence Diagrams
- [Functional Requirements](/specs/01-requirements/1.2-functional/FUNCTIONAL_REQUIREMENTS.md)
- [Data Models](/specs/DATA_MODELS.md)
- [Business Rules](/specs/BUSINESS_RULES.md)

---

## Changelog

### Version 2.0.0 (2025-11-22)

**New Endpoints (7):**
1. `GET /api/paper-trading/trailing-stops` - Get trailing stop status
2. `PUT /api/paper-trading/trailing-stops/settings` - Update trailing stop settings
3. `POST /api/paper-trading/positions/:id/trailing-stop/manual-adjust` - Manual trailing stop adjustment
4. `POST /api/paper-trading/process-ai-signal` - Process external AI signal
5. `GET /api/paper-trading/data-resolutions` - Get available data resolutions
6. `PUT /api/paper-trading/data-resolution` - Update data resolution
7. `GET /api/paper-trading/correlation-analysis` - Get position correlation analysis

**New WebSocket Events (2):**
- `trailing_stop_updated` - Trailing stop price updated
- `ai_signal_completed` - AI signal analysis completed

**Rate Limit Updates:**
- AI signal processing: 6 requests/hour
- Trailing stop updates: 100 requests/minute

**Total Endpoints:** 37 (30 → 37)

---

**Document Version:** 2.0.0
**Last Updated:** 2025-11-22
**Author:** Claude Code
**Status:** Complete
