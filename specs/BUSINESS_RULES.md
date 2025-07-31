# Business Rules Specification

## Overview
This document defines the business logic and rules that govern the cryptocurrency trading bot's behavior. All services must enforce these rules consistently.

## Trading Rules

### 1. Position Management Rules

#### 1.1 Maximum Positions
- **Rule**: Maximum concurrent open positions per user
- **Default**: 10 positions
- **Configurable**: Yes, per user settings
- **Enforcement**: Rust Core Engine
- **Validation**:
  ```
  IF count(open_positions) >= max_positions THEN
    REJECT new_position WITH error "MAX_POSITIONS_EXCEEDED"
  ```

#### 1.2 Position Size Limits
- **Rule**: Position size must be within allowed range
- **Minimum**: 0.001 BTC or $10 USDT equivalent
- **Maximum**: 10% of account balance or $100,000 USDT
- **Calculation**:
  ```
  min_position_size = MAX(0.001 * btc_price, 10)
  max_position_size = MIN(account_balance * 0.1, 100000)
  ```

#### 1.3 Leverage Limits
- **Rule**: Leverage must not exceed maximum allowed
- **Maximum**: 
  - BTC/USDT: 125x
  - ETH/USDT: 100x
  - Other major pairs: 75x
  - Alt coins: 50x
- **Default**: 10x
- **Testnet Override**: Maximum 20x for all pairs

### 2. Risk Management Rules

#### 2.1 Stop Loss Requirements
- **Rule**: All positions must have stop loss
- **Exception**: Manual trades with explicit override
- **Default Stop Loss**:
  - Conservative: 2%
  - Medium: 3%
  - Aggressive: 5%
- **Maximum Stop Loss**: 10%
- **Enforcement**:
  ```
  IF position.stop_loss IS NULL AND NOT manual_override THEN
    SET position.stop_loss = entry_price * (1 - default_stop_loss_percent)
  ```

#### 2.2 Daily Loss Limit
- **Rule**: Daily losses cannot exceed limit
- **Default**: 5% of account balance
- **Calculation Period**: Rolling 24 hours
- **Action**: Disable trading for 24 hours
- **Implementation**:
  ```
  daily_loss = SUM(realized_pnl WHERE timestamp > NOW() - 24h)
  IF daily_loss < -(account_balance * 0.05) THEN
    DISABLE trading
    SEND notification
  ```

#### 2.3 Drawdown Protection
- **Rule**: Account drawdown protection
- **Maximum Drawdown**: 15%
- **Calculation**: From account high water mark
- **Action**: 
  - Reduce position sizes by 50%
  - Increase stop loss to 1%
  - Alert user

#### 2.4 Margin Requirements
- **Rule**: Maintain minimum margin level
- **Minimum Margin Level**: 150%
- **Warning Level**: 200%
- **Calculation**:
  ```
  margin_level = (account_equity / used_margin) * 100
  ```
- **Actions**:
  - < 200%: Warning notification
  - < 150%: Prevent new positions
  - < 110%: Auto-close most losing position

### 3. AI Signal Rules

#### 3.1 Signal Confidence Threshold
- **Rule**: Only act on high-confidence signals
- **Minimum Confidence**: 
  - Production: 0.70 (70%)
  - Paper Trading: 0.60 (60%)
  - Backtesting: 0.50 (50%)
- **Override**: Manual trades ignore confidence

#### 3.2 Signal Freshness
- **Rule**: Signals expire after time limit
- **Expiration Time**:
  - 1m timeframe: 2 minutes
  - 5m timeframe: 10 minutes
  - 15m+ timeframe: 30 minutes
- **Validation**:
  ```
  signal_age = NOW() - signal.timestamp
  IF signal_age > expiration_time THEN
    REJECT trade WITH error "SIGNAL_EXPIRED"
  ```

#### 3.3 Signal Frequency Limit
- **Rule**: Limit AI analysis frequency
- **Minimum Interval**: 5 minutes per symbol
- **Rate Limit**: 10 analyses per minute total
- **Purpose**: Prevent API overuse and cost control

### 4. Order Execution Rules

#### 4.1 Order Types Allowed
- **Market Orders**: 
  - Allowed for positions < $1000
  - Slippage protection: 0.5%
- **Limit Orders**:
  - Required for positions > $1000
  - Price must be within 0.1% of current market
- **Stop Orders**:
  - Always allowed for risk management

#### 4.2 Order Validation
- **Price Validation**:
  ```
  IF order.type == LIMIT THEN
    IF ABS(order.price - market_price) / market_price > 0.5 THEN
      REJECT WITH error "PRICE_TOO_FAR_FROM_MARKET"
  ```
- **Quantity Validation**:
  ```
  IF order.quantity < minimum_order_size THEN
    REJECT WITH error "QUANTITY_TOO_SMALL"
  IF order.quantity > maximum_position_size THEN
    REJECT WITH error "QUANTITY_TOO_LARGE"
  ```

#### 4.3 Order Retry Logic
- **Rule**: Automatic retry for failed orders
- **Max Retries**: 3
- **Retry Delay**: 1s, 2s, 4s (exponential backoff)
- **Retry Conditions**:
  - Network errors
  - Temporary exchange errors
  - Not for: Insufficient balance, invalid parameters

### 5. Money Management Rules

#### 5.1 Position Sizing
- **Fixed Percentage Method**:
  ```
  position_size = account_balance * position_size_percent
  ```
- **Risk-Based Method**:
  ```
  position_size = (account_balance * risk_percent) / stop_loss_distance
  ```
- **Kelly Criterion** (advanced):
  ```
  position_size = account_balance * kelly_fraction
  kelly_fraction = (win_rate * avg_win - loss_rate * avg_loss) / avg_win
  ```

#### 5.2 Capital Allocation
- **Rule**: Never risk more than X% on single trade
- **Maximum Risk Per Trade**: 2%
- **Maximum Capital Per Trade**: 10%
- **Reserve Balance**: Keep 10% as reserve

#### 5.3 Profit Taking Rules
- **Partial Profit Taking**:
  - Take 25% profit at 1.5x risk
  - Take 50% profit at 2x risk
  - Let remainder run with trailing stop
- **Trailing Stop**:
  - Activate after 1.5x risk profit
  - Trail by 1% on 1h timeframe
  - Trail by 2% on 4h+ timeframe

### 6. Trading Time Rules

#### 6.1 Trading Hours
- **24/7 Trading**: Crypto markets never close
- **Maintenance Windows**: 
  - Daily: 00:00-00:15 UTC (Binance maintenance)
  - Weekly: Sunday 23:00-23:30 UTC
- **Reduced Activity**:
  - Major holidays: Reduce position sizes by 50%
  - Low liquidity periods: Increase spreads

#### 6.2 Timeframe Restrictions
- **Scalping** (1m-5m): 
  - Only during high volume hours
  - Require spread < 0.05%
- **Day Trading** (15m-1h):
  - Standard rules apply
- **Swing Trading** (4h+):
  - Relaxed frequency limits
  - Wider stop losses allowed

### 7. Security Rules

#### 7.1 Authentication Requirements
- **API Access**: Valid JWT token required
- **Token Expiration**: 24 hours
- **Refresh Token**: 7 days
- **2FA Required**: For withdrawals and settings changes

#### 7.2 Rate Limiting
- **Per User Limits**:
  - Orders: 10 per second
  - Queries: 100 per minute
  - AI Analysis: 10 per minute
- **Global Limits**:
  - Total API: 1000 per second
  - Database: 5000 per second

#### 7.3 IP Restrictions
- **Whitelist IPs**: For production accounts
- **Geographic Restrictions**: Block sanctioned countries
- **VPN Detection**: Warning but not blocking

### 8. Compliance Rules

#### 8.1 Trading Restrictions
- **Prohibited Pairs**: 
  - Privacy coins in restricted jurisdictions
  - Delisted tokens
  - Tokens under investigation
- **Jurisdiction Checks**:
  - US users: No leverage > 50x
  - EU users: Risk warnings required

#### 8.2 Reporting Requirements
- **Trade Reporting**: All trades logged
- **PnL Reporting**: Daily summaries
- **Tax Reporting**: Annual statements
- **Audit Trail**: 7 year retention

### 9. Paper Trading Rules

#### 9.1 Simulation Accuracy
- **Price Simulation**:
  - Use real market prices
  - Add realistic slippage (0.05%)
  - Include actual fees
- **Order Filling**:
  - Market orders: Immediate at market price
  - Limit orders: Only when price touches
  
#### 9.2 Paper Trading Limits
- **Starting Balance**: $10,000 USDT
- **Reset**: Monthly or on demand
- **Features**: All features except withdrawals
- **Data**: Separate from real trading data

### 10. AI-Specific Rules

#### 10.1 Model Selection
- **Primary Model**: GPT-4 for analysis
- **Fallback Models**: GPT-3.5 if rate limited
- **Confidence Adjustment**:
  ```
  IF model == "gpt-3.5" THEN
    confidence = confidence * 0.9
  ```

#### 10.2 Cost Control
- **Monthly Budget**: $100 for AI API calls
- **Per-User Limit**: $10/month
- **Optimization**:
  - Cache responses for 5 minutes
  - Batch similar requests
  - Use cheaper models for low-value trades

#### 10.3 AI Decision Override
- **Human Override**: Always possible
- **Confidence Override**: 
  - If confidence < 0.6: Require confirmation
  - If confidence < 0.5: Block automatic execution
- **Conflicting Signals**: Use ensemble voting

## Business Logic Flows

### Trade Execution Flow
```
1. Receive AI Signal
2. Validate Signal Freshness
3. Check Account Balance
4. Check Position Limits
5. Check Risk Limits
6. Calculate Position Size
7. Validate Order Parameters
8. Execute Order
9. Set Stop Loss/Take Profit
10. Log Trade
11. Send Notifications
```

### Risk Check Flow
```
1. Calculate Current Exposure
2. Check Daily Loss
3. Check Drawdown
4. Check Margin Level
5. Check Correlation Risk
6. Apply Risk Adjustments
7. Approve/Reject Trade
```

### AI Analysis Flow
```
1. Check Rate Limits
2. Validate Input Data
3. Check Cache
4. Call AI API
5. Validate Response
6. Apply Confidence Rules
7. Store Result
8. Broadcast Signal
```

## Error Handling Rules

### 1. Graceful Degradation
- If AI service down: Use last known good signal
- If exchange API down: Queue orders for retry
- If database down: Use in-memory cache

### 2. Circuit Breakers
- 5 failures in 1 minute: Disable feature for 5 minutes
- 10 failures in 5 minutes: Disable feature for 1 hour
- 20 failures in 1 hour: Alert admin and disable

### 3. Rollback Procedures
- Failed trades: Automatic rollback
- Partial fills: Complete or cancel based on strategy
- System errors: Revert to last known good state

## Monitoring Rules

### 1. Key Metrics to Track
- Win rate (must be > 45%)
- Average PnL (must be positive)
- Maximum drawdown (must be < 15%)
- AI accuracy (must be > 60%)
- System uptime (must be > 99%)

### 2. Alerting Thresholds
- Drawdown > 10%: Warning
- Drawdown > 15%: Critical
- Daily loss > 3%: Warning
- Daily loss > 5%: Critical
- AI confidence < 60%: Warning
- System error rate > 1%: Warning

### 3. Automated Actions
- Performance degradation: Reduce position sizes
- System instability: Switch to safe mode
- Critical errors: Stop all trading