# Data Models Specification

## Overview
This document defines all data structures used across the trading bot system. These models ensure consistency between services and storage.

## Core Trading Models

### 1. Candle Data
Represents OHLCV (Open, High, Low, Close, Volume) data for a specific time period.

```typescript
interface CandleData {
  symbol: string          // Trading pair (e.g., "BTCUSDT")
  timeframe: string       // Timeframe (e.g., "1m", "5m", "15m", "1h", "4h", "1d")
  open_time: number       // Unix timestamp in milliseconds
  open: number           // Opening price
  high: number           // Highest price
  low: number            // Lowest price
  close: number          // Closing price
  volume: number         // Volume in base asset
  close_time: number     // Unix timestamp in milliseconds
  quote_volume: number   // Volume in quote asset
  trades_count: number   // Number of trades
  taker_buy_base: number // Taker buy base asset volume
  taker_buy_quote: number // Taker buy quote asset volume
}
```

### 2. Trading Signal
Represents a trading recommendation from any strategy or AI.

```typescript
enum TradingSignal {
  Long = "Long",
  Short = "Short", 
  Neutral = "Neutral"
}

interface TradingSignalData {
  symbol: string
  signal: TradingSignal
  confidence: number          // 0.0 to 1.0
  reasoning: string
  suggested_entry?: number
  suggested_stop_loss?: number
  suggested_take_profit?: number
  risk_reward_ratio?: number
  position_size?: number      // As percentage of capital
  metadata: {
    strategy_name: string
    timestamp: string         // ISO 8601
    processing_time_ms: number
    indicators_used?: string[]
  }
}
```

### 3. Trade/Order
Represents a trading order or executed trade.

```typescript
enum OrderType {
  Market = "MARKET",
  Limit = "LIMIT",
  StopLoss = "STOP_LOSS",
  TakeProfit = "TAKE_PROFIT",
  StopLossLimit = "STOP_LOSS_LIMIT",
  TakeProfitLimit = "TAKE_PROFIT_LIMIT"
}

enum OrderSide {
  Buy = "BUY",
  Sell = "SELL"
}

enum OrderStatus {
  New = "NEW",
  PartiallyFilled = "PARTIALLY_FILLED",
  Filled = "FILLED",
  Canceled = "CANCELED",
  Rejected = "REJECTED",
  Expired = "EXPIRED"
}

enum TimeInForce {
  GTC = "GTC",    // Good Till Cancel
  IOC = "IOC",    // Immediate Or Cancel
  FOK = "FOK",    // Fill Or Kill
  GTX = "GTX"     // Good Till Crossing
}

interface Order {
  order_id: string
  symbol: string
  side: OrderSide
  type: OrderType
  quantity: number
  price?: number              // For limit orders
  stop_price?: number         // For stop orders
  time_in_force: TimeInForce
  status: OrderStatus
  executed_quantity: number
  executed_price?: number
  commission: number
  commission_asset: string
  created_at: string          // ISO 8601
  updated_at: string          // ISO 8601
  ai_signal_id?: string       // Reference to AI signal
  strategy?: string
}
```

### 4. Position
Represents an open trading position.

```typescript
enum PositionSide {
  Long = "LONG",
  Short = "SHORT"
}

interface Position {
  position_id: string
  symbol: string
  side: PositionSide
  quantity: number
  entry_price: number
  current_price: number
  mark_price: number
  liquidation_price?: number
  unrealized_pnl: number
  realized_pnl: number
  margin_type: "cross" | "isolated"
  leverage: number
  margin_used: number
  stop_loss?: number
  take_profit?: number
  opened_at: string           // ISO 8601
  updated_at: string          // ISO 8601
}
```

### 5. Account
Represents user account information.

```typescript
interface Balance {
  asset: string
  free: number               // Available balance
  locked: number             // In orders
  total: number              // free + locked
}

interface Account {
  user_id: string
  email: string
  balances: Balance[]
  total_balance_usdt: number
  total_pnl: number
  total_pnl_percentage: number
  positions_count: number
  can_trade: boolean
  can_withdraw: boolean
  can_deposit: boolean
  maker_commission: number    // e.g., 0.001 = 0.1%
  taker_commission: number
  created_at: string          // ISO 8601
  updated_at: string          // ISO 8601
}
```

## AI Service Models

### 6. Technical Indicators
Standard technical analysis indicators.

```typescript
interface TechnicalIndicators {
  // Momentum
  rsi: number                // Relative Strength Index (0-100)
  stochastic_k: number       // Stochastic %K (0-100)
  stochastic_d: number       // Stochastic %D (0-100)
  
  // Trend
  macd: number               // MACD line
  macd_signal: number        // Signal line
  macd_histogram: number     // Histogram
  adx: number                // Average Directional Index
  
  // Moving Averages
  sma_20: number
  ema_9: number
  ema_21: number
  ema_50: number
  ema_200: number
  
  // Volatility
  atr: number                // Average True Range
  bollinger_upper: number
  bollinger_middle: number
  bollinger_lower: number
  bollinger_width: number
  
  // Volume
  volume_sma: number
  volume_ratio: number       // Current volume / average volume
  obv: number                // On Balance Volume
  
  // Support/Resistance
  pivot_point: number
  resistance_1: number
  resistance_2: number
  resistance_3: number
  support_1: number
  support_2: number
  support_3: number
}
```

### 7. Market Context
Broader market conditions and sentiment.

```typescript
interface MarketContext {
  trend_strength: number      // -1.0 to 1.0 (negative = downtrend)
  volatility: number          // 0.0 to 1.0
  volume_trend: "increasing" | "decreasing" | "stable"
  market_sentiment: "bullish" | "bearish" | "neutral"
  fear_greed_index?: number   // 0-100
  btc_dominance?: number      // Percentage
  total_market_cap?: number
  major_news?: string[]
}
```

### 8. AI Analysis Result
Complete AI analysis output stored in MongoDB.

```typescript
interface AIAnalysisResult {
  _id: string                 // MongoDB ObjectId
  analysis_id: string         // UUID
  symbol: string
  timeframe: string
  signal: TradingSignal
  confidence: number
  reasoning: string
  suggested_entry?: number
  suggested_stop_loss?: number
  suggested_take_profit?: number
  risk_reward_ratio?: number
  position_size_recommendation?: number
  technical_indicators: TechnicalIndicators
  market_context: MarketContext
  additional_insights: {
    key_levels: number[]
    market_condition: "trending" | "ranging" | "volatile"
    recommended_timeframe: string
    warnings?: string[]
  }
  metadata: {
    model_version: string
    processing_time_ms: number
    timestamp: string         // ISO 8601
    created_at: string        // ISO 8601
  }
}
```

## Strategy Models

### 9. Strategy Configuration
Configuration for trading strategies.

```typescript
interface StrategyConfig {
  name: string
  enabled: boolean
  weight: number              // For ensemble strategies
  parameters: {
    [key: string]: any        // Strategy-specific parameters
  }
  risk_settings: {
    max_position_size: number
    stop_loss_percentage: number
    take_profit_percentage: number
    max_leverage: number
  }
}
```

### 10. Strategy Performance
Performance metrics for strategies.

```typescript
interface StrategyPerformance {
  strategy_name: string
  total_trades: number
  winning_trades: number
  losing_trades: number
  win_rate: number
  profit_factor: number
  sharpe_ratio: number
  max_drawdown: number
  total_pnl: number
  average_pnl: number
  best_trade: number
  worst_trade: number
  average_holding_time: number  // in minutes
  last_updated: string          // ISO 8601
}
```

## Risk Management Models

### 11. Risk Parameters
Risk management settings.

```typescript
interface RiskParameters {
  max_positions: number
  max_position_size_percent: number
  max_leverage: number
  max_daily_loss_percent: number
  max_drawdown_percent: number
  min_risk_reward_ratio: number
  correlation_limit: number
  position_sizing_method: "fixed" | "kelly" | "risk_based"
}
```

### 12. Risk Metrics
Current risk exposure metrics.

```typescript
interface RiskMetrics {
  current_positions: number
  total_exposure_usdt: number
  portfolio_heat: number        // Percentage of capital at risk
  correlation_risk: number      // 0.0 to 1.0
  margin_level: number          // Percentage
  liquidation_risk: "low" | "medium" | "high"
  var_95: number                // Value at Risk (95% confidence)
  expected_shortfall: number
}
```

## Event Models

### 13. Trading Event
Events for audit trail and webhooks.

```typescript
enum EventType {
  OrderPlaced = "order_placed",
  OrderFilled = "order_filled",
  OrderCanceled = "order_canceled",
  PositionOpened = "position_opened",
  PositionClosed = "position_closed",
  StopLossTriggered = "stop_loss_triggered",
  TakeProfitTriggered = "take_profit_triggered",
  MarginCall = "margin_call",
  Liquidation = "liquidation",
  AISignalGenerated = "ai_signal_generated",
  StrategyTriggered = "strategy_triggered"
}

interface TradingEvent {
  event_id: string
  event_type: EventType
  user_id: string
  symbol?: string
  data: any                    // Event-specific data
  timestamp: string            // ISO 8601
  ip_address?: string
  user_agent?: string
}
```

## WebSocket Message Models

### 14. WebSocket Messages
Real-time communication messages.

```typescript
enum WSMessageType {
  AISignal = "ai_signal",
  PriceUpdate = "price_update",
  OrderUpdate = "order_update",
  PositionUpdate = "position_update",
  AccountUpdate = "account_update",
  MarketDepth = "market_depth",
  Trade = "trade",
  Error = "error"
}

interface WSMessage<T = any> {
  type: WSMessageType
  data: T
  timestamp: string            // ISO 8601
  sequence?: number            // For message ordering
}

// Specific message types
interface WSAISignal {
  symbol: string
  signal: TradingSignal
  confidence: number
  reasoning: string
}

interface WSPriceUpdate {
  symbol: string
  price: number
  bid: number
  ask: number
  volume_24h: number
  change_24h: number
}

interface WSOrderUpdate {
  order: Order
  event: "created" | "updated" | "filled" | "canceled"
}
```

## Database Schemas

### MongoDB Collections

#### 1. users
```javascript
{
  _id: ObjectId,
  email: String,
  password_hash: String,
  full_name: String,
  is_active: Boolean,
  is_admin: Boolean,
  created_at: Date,
  updated_at: Date,
  last_login: Date,
  settings: {
    trading_enabled: Boolean,
    risk_level: "low" | "medium" | "high",
    max_positions: Number,
    default_quantity: Number,
    notifications: {
      email_alerts: Boolean,
      trade_notifications: Boolean,
      system_alerts: Boolean
    }
  }
}
```

#### 2. ai_analysis_results
```javascript
{
  _id: ObjectId,
  symbol: String,
  timeframe: String,
  signal: String,
  confidence: Number,
  reasoning: String,
  technical_indicators: Object,
  market_context: Object,
  additional_insights: Object,
  metadata: {
    analysis_id: String,
    model_version: String,
    processing_time_ms: Number,
    timestamp: Date
  },
  created_at: Date
}
```

#### 3. trades
```javascript
{
  _id: ObjectId,
  user_id: ObjectId,
  trade_id: String,
  order_id: String,
  symbol: String,
  side: String,
  type: String,
  quantity: Number,
  price: Number,
  executed_quantity: Number,
  executed_price: Number,
  status: String,
  commission: Number,
  commission_asset: String,
  pnl: Number,
  ai_signal_id: String,
  strategy: String,
  created_at: Date,
  updated_at: Date
}
```

#### 4. positions
```javascript
{
  _id: ObjectId,
  user_id: ObjectId,
  position_id: String,
  symbol: String,
  side: String,
  quantity: Number,
  entry_price: Number,
  current_price: Number,
  unrealized_pnl: Number,
  realized_pnl: Number,
  margin_used: Number,
  leverage: Number,
  stop_loss: Number,
  take_profit: Number,
  opened_at: Date,
  updated_at: Date,
  closed_at: Date,
  close_reason: String
}
```

## Validation Rules

### Symbol Validation
- Must be uppercase
- Must end with USDT, BUSD, or BTC
- Must be in supported symbols list

### Price Validation
- Must be positive number
- Maximum 8 decimal places
- Must be within 50% of current market price

### Quantity Validation
- Must be positive number
- Must meet minimum order size
- Must not exceed maximum position size

### Timeframe Validation
- Must be one of: 1m, 3m, 5m, 15m, 30m, 1h, 2h, 4h, 6h, 8h, 12h, 1d, 3d, 1w, 1M

### Percentage Validation
- Must be between 0 and 100
- Stop loss typically 0.5% to 10%
- Take profit typically 1% to 20%