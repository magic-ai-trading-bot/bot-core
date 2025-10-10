# Database Schema Specification

**Version:** 1.0.0
**Last Updated:** 2025-10-10
**Database:** MongoDB 7.0+
**Author:** Bot Core Team

## Overview

This document defines the complete MongoDB database schema for the Bot Core trading platform. The database supports high-frequency trading operations, AI-driven analysis, paper trading simulations, and comprehensive user management.

### Key Design Principles

1. **Denormalization for Performance** - Critical trading data is denormalized for sub-millisecond reads
2. **Time-Series Optimization** - Market data uses time-series collections with TTL indexes
3. **Flexible Schema** - Metadata fields allow extensibility without migrations
4. **Audit Trail** - All user actions and trades maintain comprehensive audit logs
5. **Scalability** - Sharding-ready design for horizontal scaling

### Database Architecture

```
bot_core_db/
├── users                          # User accounts and authentication
├── trades                         # Live trading records
├── positions                      # Active trading positions
├── market_data                    # OHLCV candlestick data
├── ai_analysis_results            # AI-generated trading signals
├── portfolio_snapshots            # Historical portfolio states
├── risk_metrics                   # Real-time risk calculations
├── strategy_configs               # User strategy configurations
├── paper_trading_accounts         # Paper trading account balances
├── paper_trading_trades           # Simulated trade records
├── paper_trading_settings         # Paper trading configurations
├── portfolio_history              # Paper trading portfolio history
├── ai_signals                     # AI signal execution log
├── performance_metrics            # Daily performance aggregations
├── audit_logs                     # System-wide audit trail
├── sessions                       # Active user sessions (optional)
├── notifications                  # User notifications queue
├── system_config                  # Global system settings
└── api_keys                       # User API keys for exchanges
```

---

## Collection Schemas

### 1. users Collection

**Purpose:** User accounts, authentication, and profile management

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  email: string,                    // Unique, indexed
  password_hash: string,            // bcrypt hash
  full_name: string | null,         // Optional display name
  is_active: boolean,               // Account status (default: true)
  is_admin: boolean,                // Admin privileges (default: false)
  created_at: DateTime,             // Account creation timestamp
  updated_at: DateTime,             // Last profile update
  last_login: DateTime | null,      // Last successful login

  // Trading Settings (embedded document)
  settings: {
    trading_enabled: boolean,       // Master trading switch (default: false)
    risk_level: enum,              // "Low" | "Medium" | "High"
    max_positions: number,          // Maximum concurrent positions (default: 3)
    default_quantity: number,       // Default trade quantity (default: 0.01)

    // Notification Preferences
    notifications: {
      email_alerts: boolean,        // Email notifications (default: true)
      trade_notifications: boolean, // Trade execution alerts (default: true)
      system_alerts: boolean        // System notifications (default: true)
    }
  }
}
```

**Indexes:**
```javascript
db.users.createIndex({ "email": 1 }, { unique: true })
db.users.createIndex({ "created_at": -1 })
db.users.createIndex({ "is_active": 1 })
```

**Validation Rules:**
- `email`: Must be valid email format, unique
- `password_hash`: Minimum 60 characters (bcrypt)
- `settings.max_positions`: Range 1-10
- `settings.default_quantity`: Positive number > 0

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439011"),
  "email": "trader@example.com",
  "password_hash": "$2b$12$KIXxLVRZ8YjE.VGH3lKP5OZq.hR5ZCqGvxEwFxH8yJ0RvMj5KqY8S",
  "full_name": "John Doe",
  "is_active": true,
  "is_admin": false,
  "created_at": ISODate("2025-01-15T10:30:00Z"),
  "updated_at": ISODate("2025-10-10T14:22:00Z"),
  "last_login": ISODate("2025-10-10T14:22:00Z"),
  "settings": {
    "trading_enabled": true,
    "risk_level": "Medium",
    "max_positions": 5,
    "default_quantity": 0.05,
    "notifications": {
      "email_alerts": true,
      "trade_notifications": true,
      "system_alerts": true
    }
  }
}
```

**Growth Projections:**
- Expected: 10,000 users in year 1
- Growth rate: ~500 users/month
- Storage: ~2KB per user = 20MB for 10K users

---

### 2. trades Collection

**Purpose:** Live trading execution records with complete audit trail

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  user_id: ObjectId,                // Reference to users collection
  symbol: string,                   // Trading pair (e.g., "BTCUSDT")
  side: enum,                       // "BUY" | "SELL"
  order_type: enum,                 // "MARKET" | "LIMIT" | "STOP_LOSS" | "TAKE_PROFIT"
  quantity: Decimal128,             // Order quantity in base asset
  price: Decimal128 | null,         // Limit price (null for market orders)
  executed_price: Decimal128 | null,// Actual execution price
  status: enum,                     // "PENDING" | "FILLED" | "PARTIALLY_FILLED" | "CANCELLED" | "FAILED"

  // Binance Integration
  binance_order_id: string | null,  // Binance order ID
  client_order_id: string,          // Client-generated ID
  time_in_force: enum,              // "GTC" | "IOC" | "FOK"

  // P&L Tracking
  pnl: Decimal128 | null,           // Realized profit/loss
  pnl_percentage: number | null,    // P&L as percentage
  fees: Decimal128,                 // Trading fees paid
  fee_asset: string,                // Fee currency (usually USDT)

  // Strategy Information
  strategy_name: string | null,     // Strategy that generated signal
  ai_signal_id: string | null,      // Reference to AI signal
  ai_confidence: number | null,     // AI confidence score (0-1)

  // Timestamps
  created_at: DateTime,             // Order creation time
  updated_at: DateTime,             // Last status update
  executed_at: DateTime | null,     // Execution timestamp
  cancelled_at: DateTime | null,    // Cancellation timestamp

  // Additional Data
  metadata: {
    ip_address: string,             // Client IP
    user_agent: string,             // Client user agent
    notes: string | null            // User notes
  }
}
```

**Indexes:**
```javascript
db.trades.createIndex({ "user_id": 1, "created_at": -1 })
db.trades.createIndex({ "symbol": 1, "created_at": -1 })
db.trades.createIndex({ "status": 1 })
db.trades.createIndex({ "binance_order_id": 1 }, { sparse: true })
db.trades.createIndex({ "ai_signal_id": 1 }, { sparse: true })
db.trades.createIndex({ "created_at": 1 }, { expireAfterSeconds: 31536000 })  // TTL 1 year
```

**Validation Rules:**
- `quantity`: Must be positive Decimal128
- `status`: Must be valid enum value
- `pnl_percentage`: Range -100 to infinity
- `fees`: Non-negative Decimal128

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439012"),
  "user_id": ObjectId("507f1f77bcf86cd799439011"),
  "symbol": "BTCUSDT",
  "side": "BUY",
  "order_type": "LIMIT",
  "quantity": NumberDecimal("0.05"),
  "price": NumberDecimal("50000.00"),
  "executed_price": NumberDecimal("49995.50"),
  "status": "FILLED",
  "binance_order_id": "28457",
  "client_order_id": "web_1234567890",
  "time_in_force": "GTC",
  "pnl": NumberDecimal("125.50"),
  "pnl_percentage": 5.02,
  "fees": NumberDecimal("2.50"),
  "fee_asset": "USDT",
  "strategy_name": "RSI_MACD_Strategy",
  "ai_signal_id": "ai_signal_789",
  "ai_confidence": 0.87,
  "created_at": ISODate("2025-10-10T10:00:00Z"),
  "updated_at": ISODate("2025-10-10T10:00:15Z"),
  "executed_at": ISODate("2025-10-10T10:00:05Z"),
  "cancelled_at": null,
  "metadata": {
    "ip_address": "192.168.1.100",
    "user_agent": "Mozilla/5.0...",
    "notes": "Strong bullish signal"
  }
}
```

**Growth Projections:**
- Expected: 1M+ trades per year
- Average: ~2KB per trade
- Storage with TTL: ~2GB (capped at 1 year retention)

---

### 3. positions Collection

**Purpose:** Currently open trading positions with real-time P&L

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  user_id: ObjectId,                // Reference to users collection
  symbol: string,                   // Trading pair
  side: enum,                       // "LONG" | "SHORT"

  // Position Details
  entry_price: Decimal128,          // Average entry price
  current_price: Decimal128,        // Current market price (updated frequently)
  quantity: Decimal128,             // Position size
  leverage: number,                 // Leverage multiplier (1-125)

  // Risk Management
  stop_loss: Decimal128 | null,     // Stop loss price
  take_profit: Decimal128 | null,   // Take profit price
  liquidation_price: Decimal128 | null, // Liquidation price (for futures)

  // P&L Tracking
  unrealized_pnl: Decimal128,       // Current unrealized P&L
  realized_pnl: Decimal128,         // Realized P&L from partial closes
  total_pnl: Decimal128,            // unrealized + realized
  pnl_percentage: number,           // P&L as percentage of margin

  // Margin & Collateral
  margin_type: enum,                // "cross" | "isolated"
  margin_used: Decimal128,          // Margin allocated to position
  maintenance_margin: Decimal128,   // Minimum margin requirement
  margin_ratio: number,             // Current margin / maintenance margin

  // Fees
  total_fees: Decimal128,           // Cumulative trading fees
  funding_fees: Decimal128,         // Funding rate fees (futures)

  // Strategy & AI
  strategy_name: string | null,     // Strategy managing position
  ai_signal_id: string | null,      // Original AI signal

  // Timestamps
  opened_at: DateTime,              // Position open time
  updated_at: DateTime,             // Last update time
  last_price_update: DateTime,      // Last market price update

  // Metadata
  metadata: {
    max_favorable_excursion: Decimal128,  // Maximum profit reached
    max_adverse_excursion: Decimal128,    // Maximum loss reached
    entry_volatility: number,             // Market volatility at entry
    risk_score: number                    // Risk assessment (0-1)
  }
}
```

**Indexes:**
```javascript
db.positions.createIndex({ "user_id": 1, "symbol": 1 })
db.positions.createIndex({ "symbol": 1, "opened_at": -1 })
db.positions.createIndex({ "margin_ratio": 1 })  // For liquidation monitoring
db.positions.createIndex({ "updated_at": -1 })
```

**Validation Rules:**
- `quantity`: Must be positive
- `leverage`: Range 1-125
- `margin_ratio`: Should trigger alerts below 1.5
- `liquidation_price`: Must be calculated for leveraged positions

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439013"),
  "user_id": ObjectId("507f1f77bcf86cd799439011"),
  "symbol": "BTCUSDT",
  "side": "LONG",
  "entry_price": NumberDecimal("50000.00"),
  "current_price": NumberDecimal("51500.00"),
  "quantity": NumberDecimal("0.1"),
  "leverage": 10,
  "stop_loss": NumberDecimal("48000.00"),
  "take_profit": NumberDecimal("55000.00"),
  "liquidation_price": NumberDecimal("45000.00"),
  "unrealized_pnl": NumberDecimal("148.00"),
  "realized_pnl": NumberDecimal("0.00"),
  "total_pnl": NumberDecimal("148.00"),
  "pnl_percentage": 29.6,
  "margin_type": "isolated",
  "margin_used": NumberDecimal("500.00"),
  "maintenance_margin": NumberDecimal("125.00"),
  "margin_ratio": 5.184,
  "total_fees": NumberDecimal("2.00"),
  "funding_fees": NumberDecimal("0.50"),
  "strategy_name": "Momentum_Breakout",
  "ai_signal_id": "ai_signal_789",
  "opened_at": ISODate("2025-10-10T08:00:00Z"),
  "updated_at": ISODate("2025-10-10T14:30:00Z"),
  "last_price_update": ISODate("2025-10-10T14:30:00Z"),
  "metadata": {
    "max_favorable_excursion": NumberDecimal("200.00"),
    "max_adverse_excursion": NumberDecimal("-50.00"),
    "entry_volatility": 0.35,
    "risk_score": 0.42
  }
}
```

**Growth Projections:**
- Expected: Average 50 positions per user
- Storage: ~1.5KB per position
- Total: 50 * 10,000 users = 500K positions = ~750MB

---

### 4. market_data Collection

**Purpose:** OHLCV candlestick data for technical analysis (Time-Series Collection)

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  symbol: string,                   // Trading pair
  timeframe: enum,                  // "1m" | "5m" | "15m" | "1h" | "4h" | "1d"

  // OHLCV Data
  open_time: DateTime,              // Candle open time
  close_time: DateTime,             // Candle close time
  open_price: Decimal128,           // Opening price
  high_price: Decimal128,           // Highest price
  low_price: Decimal128,            // Lowest price
  close_price: Decimal128,          // Closing price
  volume: Decimal128,               // Volume in base asset
  quote_volume: Decimal128,         // Volume in quote asset
  trades_count: number,             // Number of trades in candle

  // Volume Analysis
  taker_buy_base_volume: Decimal128,    // Taker buy volume (base)
  taker_buy_quote_volume: Decimal128,   // Taker buy volume (quote)

  // Derived Indicators (cached)
  indicators: {
    rsi_14: number | null,          // RSI with 14 period
    macd: number | null,            // MACD line
    macd_signal: number | null,     // Signal line
    bollinger_upper: Decimal128 | null,
    bollinger_middle: Decimal128 | null,
    bollinger_lower: Decimal128 | null,
    ema_20: Decimal128 | null,
    sma_50: Decimal128 | null
  },

  // Metadata
  created_at: DateTime,             // Data ingestion time
  source: string                    // "binance" | "manual" | "backfill"
}
```

**Indexes:**
```javascript
// Compound index for efficient time-range queries
db.market_data.createIndex({ "symbol": 1, "timeframe": 1, "open_time": -1 })
db.market_data.createIndex({ "symbol": 1, "timeframe": 1, "close_time": -1 })
// TTL index - Delete data older than 90 days
db.market_data.createIndex({ "open_time": 1 }, { expireAfterSeconds: 7776000 })
```

**Time-Series Collection Options:**
```javascript
db.createCollection("market_data", {
  timeseries: {
    timeField: "open_time",
    metaField: "symbol",
    granularity: "minutes"
  },
  expireAfterSeconds: 7776000  // 90 days
})
```

**Validation Rules:**
- `high_price` >= `low_price`
- `high_price` >= `open_price` and `close_price`
- `low_price` <= `open_price` and `close_price`
- `volume` and `quote_volume` must be non-negative

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439014"),
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "open_time": ISODate("2025-10-10T14:00:00Z"),
  "close_time": ISODate("2025-10-10T14:59:59Z"),
  "open_price": NumberDecimal("51000.00"),
  "high_price": NumberDecimal("51500.00"),
  "low_price": NumberDecimal("50800.00"),
  "close_price": NumberDecimal("51200.00"),
  "volume": NumberDecimal("125.50"),
  "quote_volume": NumberDecimal("6425000.00"),
  "trades_count": 4521,
  "taker_buy_base_volume": NumberDecimal("65.25"),
  "taker_buy_quote_volume": NumberDecimal("3340000.00"),
  "indicators": {
    "rsi_14": 62.5,
    "macd": 125.30,
    "macd_signal": 118.45,
    "bollinger_upper": NumberDecimal("52000.00"),
    "bollinger_middle": NumberDecimal("51000.00"),
    "bollinger_lower": NumberDecimal("50000.00"),
    "ema_20": NumberDecimal("50950.00"),
    "sma_50": NumberDecimal("50500.00")
  },
  "created_at": ISODate("2025-10-10T15:00:01Z"),
  "source": "binance"
}
```

**Growth Projections:**
- Expected: 8 symbols * 6 timeframes = 48 data streams
- 1-minute data: 60 * 24 * 90 = 129,600 candles per symbol (90 days)
- Storage: ~1KB per candle
- Total: 48 streams * 129,600 candles = 6.2M candles = ~6.2GB

---

### 5. ai_analysis_results Collection

**Purpose:** AI-generated trading signals and market analysis

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  analysis_id: string,              // UUID for external reference
  symbol: string,                   // Trading pair analyzed
  timeframe: string,                // Analysis timeframe

  // Signal Output
  signal: enum,                     // "BUY" | "SELL" | "HOLD" | "NEUTRAL"
  signal_strength: enum,            // "STRONG" | "MODERATE" | "WEAK"
  confidence: number,               // 0.0 to 1.0
  reasoning: string,                // AI-generated explanation

  // Trade Recommendations
  entry_price: Decimal128 | null,   // Suggested entry price
  stop_loss: Decimal128 | null,     // Suggested stop loss
  take_profit: Decimal128 | null,   // Suggested take profit
  position_size: number | null,     // % of portfolio (0-100)
  risk_reward_ratio: number | null, // R:R ratio

  // Market Analysis
  market_analysis: {
    trend_direction: string,        // "BULLISH" | "BEARISH" | "SIDEWAYS"
    trend_strength: number,         // 0-100
    volatility: number,             // 0-100
    volume_trend: string,           // "INCREASING" | "DECREASING" | "STABLE"
    support_levels: Decimal128[],   // Key support prices
    resistance_levels: Decimal128[], // Key resistance prices
    risk_score: number              // 0-100 (higher = riskier)
  },

  // Technical Indicators Used
  indicators: {
    rsi: number | null,
    macd: number | null,
    adx: number | null,
    bollinger_position: number | null,  // Price position in bands (0-1)
    volume_ratio: number | null,
    moving_averages: {
      sma_20: Decimal128 | null,
      ema_50: Decimal128 | null,
      ema_200: Decimal128 | null
    }
  },

  // AI Model Information
  model_metadata: {
    model_name: string,             // "gpt-4o-mini" | "custom_lstm"
    model_version: string,          // "v1.2.0"
    processing_time_ms: number,     // Analysis duration
    tokens_used: number | null,     // For GPT models
    temperature: number | null       // For GPT models
  },

  // Timestamps
  timestamp: DateTime,              // Analysis generation time
  created_at: DateTime,             // Database insertion time
  expires_at: DateTime | null,      // Signal expiration (optional)

  // Execution Status
  executed: boolean,                // Was signal acted upon
  trade_id: string | null,          // Reference to executed trade

  // Performance Tracking
  performance: {
    outcome: enum | null,           // "WIN" | "LOSS" | "BREAKEVEN" | null
    actual_pnl: Decimal128 | null,  // Actual P&L if executed
    signal_accuracy: number | null   // Retrospective accuracy score
  }
}
```

**Indexes:**
```javascript
db.ai_analysis_results.createIndex({ "symbol": 1, "timestamp": -1 })
db.ai_analysis_results.createIndex({ "signal": 1, "confidence": -1 })
db.ai_analysis_results.createIndex({ "executed": 1 })
db.ai_analysis_results.createIndex({ "trade_id": 1 }, { sparse: true })
db.ai_analysis_results.createIndex({ "created_at": 1 }, { expireAfterSeconds: 2592000 })  // TTL 30 days
```

**Validation Rules:**
- `confidence`: Range 0.0-1.0
- `risk_reward_ratio`: Must be positive if set
- `position_size`: Range 0-100
- `signal_strength`: Must be valid enum

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439015"),
  "analysis_id": "ai_signal_789",
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "signal": "BUY",
  "signal_strength": "STRONG",
  "confidence": 0.87,
  "reasoning": "Strong bullish momentum detected. RSI showing oversold conditions with MACD bullish crossover. Volume increasing, indicating strong buying pressure. Key resistance at $52,000 with support at $50,000.",
  "entry_price": NumberDecimal("51000.00"),
  "stop_loss": NumberDecimal("49500.00"),
  "take_profit": NumberDecimal("54000.00"),
  "position_size": 5.0,
  "risk_reward_ratio": 2.0,
  "market_analysis": {
    "trend_direction": "BULLISH",
    "trend_strength": 75,
    "volatility": 35,
    "volume_trend": "INCREASING",
    "support_levels": [NumberDecimal("50000.00"), NumberDecimal("49000.00")],
    "resistance_levels": [NumberDecimal("52000.00"), NumberDecimal("54000.00")],
    "risk_score": 42
  },
  "indicators": {
    "rsi": 62.5,
    "macd": 125.30,
    "adx": 28.5,
    "bollinger_position": 0.65,
    "volume_ratio": 1.45,
    "moving_averages": {
      "sma_20": NumberDecimal("50800.00"),
      "ema_50": NumberDecimal("50200.00"),
      "ema_200": NumberDecimal("48500.00")
    }
  },
  "model_metadata": {
    "model_name": "gpt-4o-mini",
    "model_version": "v1.2.0",
    "processing_time_ms": 2450,
    "tokens_used": 1250,
    "temperature": 0.7
  },
  "timestamp": ISODate("2025-10-10T14:00:00Z"),
  "created_at": ISODate("2025-10-10T14:00:02Z"),
  "expires_at": ISODate("2025-10-10T18:00:00Z"),
  "executed": true,
  "trade_id": "trade_12345",
  "performance": {
    "outcome": "WIN",
    "actual_pnl": NumberDecimal("125.50"),
    "signal_accuracy": 0.92
  }
}
```

**Growth Projections:**
- Expected: ~10,000 analyses per day (8 symbols * hourly)
- TTL: 30 days retention
- Storage: ~3KB per analysis
- Total: 10,000 * 30 = 300K documents = ~900MB

---

### 6. portfolio_snapshots Collection

**Purpose:** Historical portfolio value tracking for performance charts

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  user_id: ObjectId,                // Reference to users collection

  // Portfolio Values
  total_value: Decimal128,          // Total portfolio value in USDT
  cash_balance: Decimal128,         // Available cash
  positions_value: Decimal128,      // Value of all open positions
  margin_used: Decimal128,          // Total margin allocated
  free_margin: Decimal128,          // Available margin

  // P&L Summary
  unrealized_pnl: Decimal128,       // Total unrealized P&L
  realized_pnl: Decimal128,         // Total realized P&L (lifetime)
  daily_pnl: Decimal128,            // P&L for current day
  total_pnl: Decimal128,            // unrealized + realized
  total_pnl_percentage: number,     // P&L as % of initial investment

  // Position Summary
  open_positions_count: number,     // Number of open positions
  long_positions_count: number,     // Number of long positions
  short_positions_count: number,    // Number of short positions

  // Risk Metrics
  portfolio_leverage: number,       // Average leverage across positions
  max_drawdown: Decimal128,         // Maximum historical drawdown
  max_drawdown_percentage: number,  // Drawdown as percentage
  var_95: Decimal128 | null,        // Value at Risk (95% confidence)
  sharpe_ratio: number | null,      // Risk-adjusted return

  // Timestamps
  snapshot_time: DateTime,          // Snapshot timestamp
  created_at: DateTime              // Database insertion time
}
```

**Indexes:**
```javascript
db.portfolio_snapshots.createIndex({ "user_id": 1, "snapshot_time": -1 })
db.portfolio_snapshots.createIndex({ "snapshot_time": -1 })
db.portfolio_snapshots.createIndex({ "created_at": 1 }, { expireAfterSeconds: 7776000 })  // TTL 90 days
```

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439016"),
  "user_id": ObjectId("507f1f77bcf86cd799439011"),
  "total_value": NumberDecimal("12500.00"),
  "cash_balance": NumberDecimal("7500.00"),
  "positions_value": NumberDecimal("5000.00"),
  "margin_used": NumberDecimal("2000.00"),
  "free_margin": NumberDecimal("5500.00"),
  "unrealized_pnl": NumberDecimal("250.00"),
  "realized_pnl": NumberDecimal("1500.00"),
  "daily_pnl": NumberDecimal("125.00"),
  "total_pnl": NumberDecimal("1750.00"),
  "total_pnl_percentage": 17.5,
  "open_positions_count": 3,
  "long_positions_count": 2,
  "short_positions_count": 1,
  "portfolio_leverage": 2.5,
  "max_drawdown": NumberDecimal("450.00"),
  "max_drawdown_percentage": 4.5,
  "var_95": NumberDecimal("320.00"),
  "sharpe_ratio": 1.85,
  "snapshot_time": ISODate("2025-10-10T14:00:00Z"),
  "created_at": ISODate("2025-10-10T14:00:01Z")
}
```

**Growth Projections:**
- Expected: Snapshots every 15 minutes per user
- 96 snapshots/day * 10,000 users = 960K snapshots/day
- TTL: 90 days = 86.4M snapshots
- Storage: ~400 bytes per snapshot = ~35GB

---

### 7. paper_trading_accounts Collection

**Purpose:** Paper trading account balances and configurations

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  user_id: ObjectId,                // Reference to users collection (unique)

  // Account Balances
  initial_balance: Decimal128,      // Starting balance (default: 10000 USDT)
  current_balance: Decimal128,      // Current cash balance
  equity: Decimal128,               // Balance + unrealized P&L
  margin_used: Decimal128,          // Margin in open positions
  free_margin: Decimal128,          // Available for trading

  // Performance Metrics (embedded)
  metrics: {
    total_trades: number,           // Lifetime trade count
    winning_trades: number,         // Profitable trades
    losing_trades: number,          // Losing trades
    win_rate: number,               // Percentage (0-100)

    // P&L Statistics
    total_pnl: Decimal128,          // Lifetime P&L
    total_pnl_percentage: number,   // P&L as % of initial
    average_win: Decimal128,        // Average profit per winning trade
    average_loss: Decimal128,       // Average loss per losing trade
    largest_win: Decimal128,        // Biggest single profit
    largest_loss: Decimal128,       // Biggest single loss

    // Risk Metrics
    profit_factor: number,          // Gross profit / gross loss
    sharpe_ratio: number,           // Risk-adjusted returns
    max_drawdown: Decimal128,       // Maximum peak-to-trough decline
    max_drawdown_percentage: number,
    max_consecutive_wins: number,
    max_consecutive_losses: number,

    // Trade Duration
    average_trade_duration_ms: number,
    shortest_trade_ms: number,
    longest_trade_ms: number
  },

  // Current Positions
  open_positions: number,           // Count of open trades
  open_trade_ids: string[],         // Array of active trade IDs

  // Settings
  settings: {
    max_leverage: number,           // Maximum allowed leverage (default: 10)
    max_positions: number,          // Maximum concurrent positions
    default_quantity: Decimal128,   // Default trade size
    trading_fees_rate: number,      // Simulated fee rate (default: 0.0004)
    enable_ai_trading: boolean,     // Allow AI to execute trades
    risk_per_trade: number          // % of balance to risk per trade
  },

  // Timestamps
  created_at: DateTime,             // Account creation
  updated_at: DateTime,             // Last activity
  last_trade_at: DateTime | null,  // Last trade execution
  reset_at: DateTime | null         // Last account reset
}
```

**Indexes:**
```javascript
db.paper_trading_accounts.createIndex({ "user_id": 1 }, { unique: true })
db.paper_trading_accounts.createIndex({ "metrics.total_pnl": -1 })
db.paper_trading_accounts.createIndex({ "metrics.win_rate": -1 })
```

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439017"),
  "user_id": ObjectId("507f1f77bcf86cd799439011"),
  "initial_balance": NumberDecimal("10000.00"),
  "current_balance": NumberDecimal("8500.00"),
  "equity": NumberDecimal("12250.00"),
  "margin_used": NumberDecimal("2000.00"),
  "free_margin": NumberDecimal("10250.00"),
  "metrics": {
    "total_trades": 45,
    "winning_trades": 28,
    "losing_trades": 17,
    "win_rate": 62.22,
    "total_pnl": NumberDecimal("2250.00"),
    "total_pnl_percentage": 22.5,
    "average_win": NumberDecimal("125.50"),
    "average_loss": NumberDecimal("-75.25"),
    "largest_win": NumberDecimal("450.00"),
    "largest_loss": NumberDecimal("-220.00"),
    "profit_factor": 2.75,
    "sharpe_ratio": 1.82,
    "max_drawdown": NumberDecimal("450.00"),
    "max_drawdown_percentage": 4.5,
    "max_consecutive_wins": 7,
    "max_consecutive_losses": 3,
    "average_trade_duration_ms": 7200000,
    "shortest_trade_ms": 300000,
    "longest_trade_ms": 86400000
  },
  "open_positions": 2,
  "open_trade_ids": ["trade_uuid_1", "trade_uuid_2"],
  "settings": {
    "max_leverage": 10,
    "max_positions": 5,
    "default_quantity": NumberDecimal("0.05"),
    "trading_fees_rate": 0.0004,
    "enable_ai_trading": true,
    "risk_per_trade": 2.0
  },
  "created_at": ISODate("2025-08-01T00:00:00Z"),
  "updated_at": ISODate("2025-10-10T14:30:00Z"),
  "last_trade_at": ISODate("2025-10-10T10:15:00Z"),
  "reset_at": null
}
```

**Growth Projections:**
- Expected: 1 account per user
- 10,000 users = 10,000 accounts
- Storage: ~2KB per account = 20MB

---

### 8. paper_trading_trades Collection

**Purpose:** Complete paper trading execution history

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  trade_id: string,                 // UUID (unique)
  user_id: ObjectId,                // Reference to users collection
  account_id: ObjectId,             // Reference to paper_trading_accounts
  symbol: string,                   // Trading pair

  // Trade Type
  trade_type: enum,                 // "LONG" | "SHORT"
  status: enum,                     // "OPEN" | "CLOSED" | "CANCELLED"

  // Entry Details
  entry_price: Decimal128,          // Entry price
  quantity: Decimal128,             // Trade quantity
  leverage: number,                 // Leverage used (1-125)
  initial_margin: Decimal128,       // Margin required

  // Exit Details
  exit_price: Decimal128 | null,    // Exit price (null if open)
  close_reason: enum | null,        // "TAKE_PROFIT" | "STOP_LOSS" | "MANUAL" | "AI_SIGNAL" | "MARGIN_CALL"

  // P&L
  pnl: Decimal128 | null,           // Realized P&L (null if open)
  pnl_percentage: number,           // P&L as % of margin
  unrealized_pnl: Decimal128,       // Current unrealized P&L

  // Fees
  trading_fees: Decimal128,         // Entry + exit fees
  funding_fees: Decimal128,         // Accumulated funding fees
  total_fees: Decimal128,           // Total fees paid

  // Risk Management
  stop_loss: Decimal128 | null,     // Stop loss price
  take_profit: Decimal128 | null,   // Take profit price

  // Performance Metrics
  max_favorable_excursion: Decimal128,  // Best price reached
  max_adverse_excursion: Decimal128,    // Worst price reached

  // AI Information
  ai_signal_id: string | null,      // Reference to AI signal
  ai_confidence: number | null,     // AI confidence at entry
  ai_reasoning: string | null,      // AI explanation

  // Timestamps
  open_time: DateTime,              // Position open time
  close_time: DateTime | null,      // Position close time
  duration_ms: number | null,       // Trade duration
  created_at: DateTime,             // Database insertion
  updated_at: DateTime              // Last update
}
```

**Indexes:**
```javascript
db.paper_trading_trades.createIndex({ "trade_id": 1 }, { unique: true })
db.paper_trading_trades.createIndex({ "user_id": 1, "created_at": -1 })
db.paper_trading_trades.createIndex({ "account_id": 1, "status": 1 })
db.paper_trading_trades.createIndex({ "symbol": 1, "created_at": -1 })
db.paper_trading_trades.createIndex({ "status": 1 })
db.paper_trading_trades.createIndex({ "ai_signal_id": 1 }, { sparse: true })
```

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439018"),
  "trade_id": "trade_uuid_12345",
  "user_id": ObjectId("507f1f77bcf86cd799439011"),
  "account_id": ObjectId("507f1f77bcf86cd799439017"),
  "symbol": "BTCUSDT",
  "trade_type": "LONG",
  "status": "CLOSED",
  "entry_price": NumberDecimal("50000.00"),
  "quantity": NumberDecimal("0.1"),
  "leverage": 10,
  "initial_margin": NumberDecimal("500.00"),
  "exit_price": NumberDecimal("51500.00"),
  "close_reason": "TAKE_PROFIT",
  "pnl": NumberDecimal("148.00"),
  "pnl_percentage": 29.6,
  "unrealized_pnl": NumberDecimal("0.00"),
  "trading_fees": NumberDecimal("2.00"),
  "funding_fees": NumberDecimal("0.50"),
  "total_fees": NumberDecimal("2.50"),
  "stop_loss": NumberDecimal("48000.00"),
  "take_profit": NumberDecimal("55000.00"),
  "max_favorable_excursion": NumberDecimal("200.00"),
  "max_adverse_excursion": NumberDecimal("-25.00"),
  "ai_signal_id": "ai_signal_789",
  "ai_confidence": 0.87,
  "ai_reasoning": "Strong bullish momentum with MACD crossover",
  "open_time": ISODate("2025-10-10T08:00:00Z"),
  "close_time": ISODate("2025-10-10T12:30:00Z"),
  "duration_ms": 16200000,
  "created_at": ISODate("2025-10-10T08:00:00Z"),
  "updated_at": ISODate("2025-10-10T12:30:01Z")
}
```

**Growth Projections:**
- Expected: 50 trades per user per month
- 10,000 users * 50 = 500K trades/month = 6M trades/year
- Storage: ~1.5KB per trade = 9GB/year

---

### 9. audit_logs Collection

**Purpose:** System-wide security and activity audit trail

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  user_id: ObjectId | null,         // User who performed action (null for system)

  // Action Details
  action: string,                   // Action type (e.g., "USER_LOGIN", "TRADE_EXECUTED")
  resource: string,                 // Resource affected (e.g., "user", "trade", "position")
  resource_id: string | null,       // ID of affected resource

  // Request Context
  method: string | null,            // HTTP method (GET, POST, etc.)
  endpoint: string | null,          // API endpoint
  ip_address: string,               // Client IP address
  user_agent: string | null,        // Client user agent

  // Status
  status: enum,                     // "SUCCESS" | "FAILURE" | "ERROR"
  error_message: string | null,     // Error details if failed

  // Additional Data
  metadata: object,                 // Action-specific data

  // Timestamps
  timestamp: DateTime,              // Action timestamp
  created_at: DateTime              // Log insertion time
}
```

**Indexes:**
```javascript
db.audit_logs.createIndex({ "user_id": 1, "timestamp": -1 })
db.audit_logs.createIndex({ "action": 1, "timestamp": -1 })
db.audit_logs.createIndex({ "resource": 1, "resource_id": 1 })
db.audit_logs.createIndex({ "ip_address": 1 })
db.audit_logs.createIndex({ "timestamp": -1 })
db.audit_logs.createIndex({ "created_at": 1 }, { expireAfterSeconds: 15552000 })  // TTL 180 days
```

**Example Document:**
```json
{
  "_id": ObjectId("507f1f77bcf86cd799439019"),
  "user_id": ObjectId("507f1f77bcf86cd799439011"),
  "action": "TRADE_EXECUTED",
  "resource": "trade",
  "resource_id": "trade_12345",
  "method": "POST",
  "endpoint": "/api/v1/trades",
  "ip_address": "192.168.1.100",
  "user_agent": "Mozilla/5.0 (Windows NT 10.0; Win64; x64)...",
  "status": "SUCCESS",
  "error_message": null,
  "metadata": {
    "symbol": "BTCUSDT",
    "side": "BUY",
    "quantity": "0.05",
    "order_type": "MARKET",
    "estimated_value": "2500.00"
  },
  "timestamp": ISODate("2025-10-10T14:30:00Z"),
  "created_at": ISODate("2025-10-10T14:30:00Z")
}
```

---

### 10. system_config Collection

**Purpose:** Global system configuration and feature flags

**Schema:**
```typescript
{
  _id: ObjectId,                    // Primary key
  key: string,                      // Config key (unique)
  value: any,                       // Config value (flexible type)
  category: string,                 // "trading" | "security" | "features" | "limits"
  description: string,              // Human-readable description
  is_active: boolean,               // Whether config is active
  updated_by: string,               // Admin username
  updated_at: DateTime,             // Last update time
  created_at: DateTime              // Creation time
}
```

**Indexes:**
```javascript
db.system_config.createIndex({ "key": 1 }, { unique: true })
db.system_config.createIndex({ "category": 1 })
```

**Example Documents:**
```json
[
  {
    "_id": ObjectId("507f1f77bcf86cd79943901a"),
    "key": "TRADING_ENABLED",
    "value": false,
    "category": "trading",
    "description": "Global trading on/off switch",
    "is_active": true,
    "updated_by": "admin",
    "updated_at": ISODate("2025-10-10T00:00:00Z"),
    "created_at": ISODate("2025-01-01T00:00:00Z")
  },
  {
    "_id": ObjectId("507f1f77bcf86cd79943901b"),
    "key": "MAX_LEVERAGE_GLOBAL",
    "value": 125,
    "category": "limits",
    "description": "Maximum leverage allowed platform-wide",
    "is_active": true,
    "updated_by": "admin",
    "updated_at": ISODate("2025-10-10T00:00:00Z"),
    "created_at": ISODate("2025-01-01T00:00:00Z")
  }
]
```

---

## Additional Collections (Brief)

### 11. paper_trading_settings Collection
Stores serialized paper trading settings configurations.

### 12. portfolio_history Collection
Time-series data for paper trading portfolio snapshots.

### 13. ai_signals Collection
Log of all AI signals and their execution status.

### 14. performance_metrics Collection
Daily aggregated performance metrics for reporting.

### 15. notifications Collection
User notification queue for alerts and system messages.

### 16. sessions Collection (Optional)
Active user sessions for authentication tracking.

### 17. api_keys Collection
User-stored API keys for exchange integrations (encrypted).

---

## Data Retention Policies

| Collection | Retention Period | Mechanism |
|-----------|-----------------|-----------|
| users | Permanent | Manual deletion only |
| trades | 1 year | TTL index |
| positions | Until closed | Manual deletion on close |
| market_data | 90 days | TTL index + time-series |
| ai_analysis_results | 30 days | TTL index |
| portfolio_snapshots | 90 days | TTL index |
| paper_trading_trades | Permanent | User-controlled reset |
| audit_logs | 180 days | TTL index |
| notifications | 7 days | TTL index |

---

## Cross-References

- See `DB-INDEXES.md` for detailed index strategies
- See `DB-ERD.mermaid` for visual relationship diagram
- See `DB-MIGRATIONS.md` for schema migration procedures
- See `../DATA_MODELS.md` for API-level data models

---

**Document Version:** 1.0.0
**Schema Version:** 1.0.0
**Last Updated:** 2025-10-10
