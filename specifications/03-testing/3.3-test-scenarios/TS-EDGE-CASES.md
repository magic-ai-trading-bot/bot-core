# Test Scenarios - Edge Cases

**Document ID:** TS-EDGE-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active

---

## Overview

This document contains edge case test scenarios - unusual, boundary, and extreme conditions that may occur in the Bot Core trading platform. Edge cases test system robustness and error handling.

---

## Scenario Index

| ID | Scenario Name | Priority | Severity | Status |
|----|---------------|----------|----------|--------|
| TS-EDGE-001 | Division by Zero in Indicators | High | Critical | ✅ |
| TS-EDGE-002 | Empty Market Data | High | High | ✅ |
| TS-EDGE-003 | Extreme Volatility (1000% Price Swing) | Critical | Critical | ✅ |
| TS-EDGE-004 | Zero Balance Account | Medium | Medium | ✅ |
| TS-EDGE-005 | Maximum Positions Limit | Medium | Medium | ✅ |
| TS-EDGE-006 | Liquidation Scenarios | Critical | Critical | ✅ |
| TS-EDGE-007 | API Rate Limiting | High | High | ✅ |
| TS-EDGE-008 | Network Timeouts | High | High | ✅ |
| TS-EDGE-009 | Database Connection Loss | Critical | Critical | ✅ |
| TS-EDGE-010 | Concurrent Trade Requests | High | High | ✅ |
| TS-EDGE-011 | Invalid Decimal Precision | Medium | Medium | ✅ |
| TS-EDGE-012 | Unicode and Special Characters | Low | Low | ✅ |
| TS-EDGE-013 | Clock Skew and Time Sync Issues | Medium | High | ✅ |
| TS-EDGE-014 | WebSocket Message Flood | High | High | ✅ |
| TS-EDGE-015 | Very Long Running Session | Low | Medium | ✅ |
| TS-EDGE-016 | Memory Leak Under Load | High | Critical | ✅ |
| TS-EDGE-017 | Circular Trading Loop | Medium | High | ✅ |
| TS-EDGE-018 | Price with More Than 8 Decimal Places | Low | Low | ✅ |
| TS-EDGE-019 | AI Model Returns NaN | High | High | ✅ |
| TS-EDGE-020 | Simultaneous Buy and Sell Same Asset | Medium | Medium | ✅ |

---

## TS-EDGE-001: Division by Zero in Indicators

### Gherkin Scenario

```gherkin
Feature: Handle Division by Zero in Technical Indicators
  As the AI service
  I want to handle division by zero gracefully
  So that indicator calculations don't crash

  Scenario: RSI calculation with zero price movement
    Given price data where all prices are identical (50000, 50000, 50000...)
    When I calculate RSI
    Then average gain = 0
    And average loss = 0
    And RSI formula involves division by zero: RS = avg_gain / avg_loss
    And system should handle gracefully
    And return RSI = 50 (neutral) as default
    And no exception should be thrown

  Scenario: MACD with constant prices
    Given all prices are 50000 (no movement)
    When I calculate MACD
    Then EMA calculations produce same value
    And MACD line = 0
    And Signal line = 0
    And Histogram = 0
    And no division by zero occurs

  Scenario: Standard deviation is zero
    Given prices: [50000, 50000, 50000]
    When I calculate Bollinger Bands
    Then standard deviation = 0
    And Bollinger Bands collapse to SMA line
    And upper band = middle band = lower band = 50000
```

**Expected Behavior:**
- No crashes or exceptions
- Return sensible defaults (RSI=50, MACD=0)
- Log warnings for unusual conditions
- Continue processing other indicators

---

## TS-EDGE-002: Empty Market Data

### Gherkin Scenario

```gherkin
Feature: Handle Empty Market Data
  As the trading system
  I want to handle empty data gracefully
  So that missing data doesn't break functionality

  Scenario: Request analysis with no klines
    Given I request AI analysis for BTCUSDT
    But Binance returns empty array []
    When Python AI service attempts analysis
    Then system should detect empty data
    And return error: "Insufficient data for analysis"
    And not attempt to calculate indicators
    And not crash with IndexError

  Scenario: Single candle insufficient for indicators
    Given market data has only 1 candle
    When I attempt to calculate RSI (needs 14+ periods)
    Then system should return: "Insufficient data: need 14+ candles, got 1"
    And default values should be returned
    And analysis should gracefully degrade

  Scenario: Order book is empty
    Given order book for obscure trading pair is empty
    When user attempts to trade
    Then system should warn: "No liquidity available"
    And prevent trade execution
```

---

## TS-EDGE-003: Extreme Volatility (1000% Price Swing)

### Gherkin Scenario

```gherkin
Feature: Handle Extreme Price Movements
  As the trading system
  I want to handle flash crashes and pumps
  So that users don't lose everything in extreme events

  Scenario: Flash crash - price drops 90% instantly
    Given BTC price is 50000
    When price suddenly drops to 5000 (90% crash) in 1 second
    Then system should detect anomaly
    And flag as "potential data error or flash crash"
    And pause automated trading temporarily
    And notify user: "Extreme volatility detected - verify market conditions"
    And existing stop-losses should trigger
    But new orders should require manual confirmation

  Scenario: Flash pump - price spikes 10x
    Given BTC price is 50000
    When price spikes to 500000 (10x) in 1 minute
    Then system should detect anomaly
    And verify price with multiple sources
    And if confirmed, allow stop-loss triggers
    And limit position entry to prevent chasing
    And require admin verification for large trades

  Scenario: Circuit breaker activation
    Given volatility exceeds 50% in 5 minutes
    When circuit breaker threshold is hit
    Then system should:
      - Pause all automated trading
      - Close high-risk positions
      - Notify all users
      - Wait for volatility to normalize
      - Resume after 15 minute cooldown
```

---

## TS-EDGE-004: Zero Balance Account

### Gherkin Scenario

```gherkin
Feature: Handle Zero Balance
  As the trading system
  I want to handle accounts with zero balance
  So that users can't trade without funds

  Scenario: Attempt to trade with zero balance
    Given my account balance is 0 USDT
    When I attempt to buy 0.1 BTC (requires 5000 USDT)
    Then system should reject immediately
    And error: "Insufficient balance: 0 USDT available, 5000 USDT required"
    And suggest: "Please deposit funds to start trading"

  Scenario: Balance becomes zero during session
    Given I started with 1000 USDT
    And lost all funds trading
    And balance is now 0.00 USDT
    When I view dashboard
    Then I should see "No funds available"
    And all trade buttons should be disabled
    And I should see "Deposit" call-to-action

  Scenario: Dust balance (less than minimum trade size)
    Given I have 0.50 USDT (below minimum trade: 10 USDT)
    When I attempt to trade
    Then system should reject
    And error: "Balance below minimum trade amount"
```

---

## TS-EDGE-005: Maximum Positions Limit

### Gherkin Scenario

```gherkin
Feature: Maximum Positions Limit
  As the risk management system
  I want to limit concurrent positions
  So that users don't over-leverage

  Scenario: Reach maximum position limit
    Given maximum positions allowed = 10
    And I have 10 open positions
    When I attempt to open 11th position
    Then system should reject
    And error: "Maximum positions limit reached (10/10)"
    And suggest: "Close an existing position to open a new one"

  Scenario: Position limit per symbol
    Given max positions per symbol = 3
    And I have 3 open BTCUSDT positions
    When I attempt 4th BTCUSDT position
    Then system should reject
    And error: "Symbol position limit reached for BTCUSDT"
```

---

## TS-EDGE-006: Liquidation Scenarios

### Gherkin Scenario

```gherkin
Feature: Liquidation Handling
  As the risk system
  I want to liquidate risky positions
  So that losses don't exceed collateral

  Scenario: Position reaches liquidation price
    Given I have leveraged position:
      - Entry: 50000 BTC
      - Leverage: 10x
      - Collateral: 500 USDT
      - Liquidation price: 45500 (10% drop)
    When price drops to 45500
    Then system should automatically liquidate
    And close position at market price
    And collateral is lost
    And user is notified: "Position liquidated at 45500"

  Scenario: Cascade liquidation prevention
    Given multiple users have overlapping liquidation prices
    When price approaches liquidation zone
    Then system should:
      - Spread liquidations over time
      - Avoid cascading sell-offs
      - Use limit orders where possible
```

---

## TS-EDGE-007: API Rate Limiting

### Gherkin Scenario

```gherkin
Feature: Handle API Rate Limits
  As the trading system
  I want to handle rate limits gracefully
  So that API access isn't permanently blocked

  Scenario: Hit Binance API rate limit
    Given Binance rate limit is 1200 requests/minute
    When I make 1201 requests in 1 minute
    Then request #1201 receives 429 Too Many Requests
    And system should implement exponential backoff
    And wait 60 seconds before retrying
    And queue pending requests
    And resume after cooldown

  Scenario: Hit OpenAI API rate limit
    Given OpenAI rate limit is 60 requests/minute
    When I exceed limit
    Then system should:
      - Cache recent responses (use cache if <5 min old)
      - Fall back to technical analysis
      - Retry with backoff
      - Don't block user trading
```

---

## TS-EDGE-008: Network Timeouts

### Gherkin Scenario

```gherkin
Feature: Network Timeout Handling
  As the trading system
  I want to handle network timeouts
  So that slow connections don't break functionality

  Scenario: Binance API timeout
    Given network latency is 30+ seconds
    When I request market data
    And request times out after 30s
    Then system should:
      - Retry up to 3 times
      - Use cached data if available
      - Fall back to alternative data source
      - Notify user of connectivity issues

  Scenario: Python AI service timeout
    Given AI analysis takes >30 seconds
    When Rust waits for response
    And timeout occurs
    Then Rust should:
      - Cancel request
      - Use local technical analysis
      - Log timeout error
      - Continue trading with degraded AI
```

---

## TS-EDGE-009: Database Connection Loss

### Gherkin Scenario

```gherkin
Feature: Database Connection Loss
  As the trading system
  I want to handle database failures
  So that trading can continue temporarily

  Scenario: MongoDB connection drops mid-trade
    Given trade is executing
    When MongoDB connection is lost
    Then trade should complete on Binance
    And trade data should be queued for retry
    And background job retries database save
    And user sees success (trade did execute)
    And system logs database failure

  Scenario: Database unavailable on startup
    Given Rust service starts
    But MongoDB is down
    Then Rust should:
      - Retry connection 5 times
      - Wait 5 seconds between retries
      - If all fail, enter read-only mode
      - Allow viewing, but not trading
      - Monitor and reconnect when available
```

---

## TS-EDGE-010: Concurrent Trade Requests

### Gherkin Scenario

```gherkin
Feature: Concurrent Trade Requests
  As the trading system
  I want to handle simultaneous trades
  So that race conditions don't cause errors

  Scenario: User double-clicks "Buy" button
    Given user clicks "Buy" button twice rapidly
    When two buy requests are sent simultaneously
    Then system should:
      - Use idempotency key to detect duplicate
      - Execute only one trade
      - Return same order ID for both requests
      - Prevent double-spending

  Scenario: Multiple bot instances trade same account
    Given two bot instances run for same account
    When both attempt to trade BTCUSDT simultaneously
    Then system should:
      - Use database locks to prevent conflicts
      - Execute trades sequentially
      - Maintain accurate balance
      - Prevent overdraft
```

---

## TS-EDGE-011: Invalid Decimal Precision

### Gherkin Scenario

```gherkin
Feature: Decimal Precision Handling
  As the trading system
  I want to handle decimal precision correctly
  So that tiny rounding errors don't accumulate

  Scenario: Quantity with excessive decimals
    Given Binance allows max 8 decimal places
    When user enters quantity: 0.123456789 BTC (9 decimals)
    Then system should round to: 0.12345678 BTC
    And inform user of rounding

  Scenario: Price with infinite decimals
    Given price calculation results in 50000.333333333...
    When system stores price
    Then should round to appropriate precision
    And avoid floating-point errors
```

---

## TS-EDGE-012: Unicode and Special Characters

### Gherkin Scenario

```gherkin
Feature: Unicode and Special Characters
  As the security system
  I want to handle special characters safely
  So that XSS and injection attacks fail

  Scenario: Username with emoji
    Given user enters username: "trader🚀💰"
    When system processes registration
    Then username should be stored correctly
    And displayed without breaking UI
    And no XSS exploit should occur

  Scenario: Password with special characters
    Given password: "P@ssw0rd!<>\"'&"
    When system hashes password
    Then all characters should be preserved
    And login should work correctly
```

---

## TS-EDGE-013: Clock Skew and Time Sync Issues

### Gherkin Scenario

```gherkin
Feature: Time Synchronization
  As the trading system
  I want to handle time sync issues
  So that Binance API signatures remain valid

  Scenario: Server time drifts from Binance
    Given server clock is 10 seconds behind
    When I make signed Binance API request
    Then Binance rejects with error -1021 (timestamp out of sync)
    And system should detect error
    And sync time with Binance server time
    And retry request with corrected timestamp
```

---

## TS-EDGE-014: WebSocket Message Flood

### Gherkin Scenario

```gherkin
Feature: WebSocket Message Flood
  As the WebSocket server
  I want to handle message floods
  So that server doesn't crash

  Scenario: Binance sends 1000 price updates/second
    Given volatile market conditions
    When Binance WebSocket sends 1000 messages/second
    Then system should:
      - Throttle to reasonable rate (10/second)
      - Drop intermediate messages
      - Send only latest price to client
      - Avoid server overload
```

---

## TS-EDGE-015: Very Long Running Session

### Gherkin Scenario

```gherkin
Feature: Long Running Session
  As a user
  I want my session to work for hours
  So that I don't get logged out unexpectedly

  Scenario: 24-hour trading session
    Given user logs in and trades for 24 hours
    When JWT token approaches expiration (24h)
    Then system should:
      - Refresh token automatically
      - Maintain session without interruption
      - Prevent memory leaks
      - Keep WebSocket connected
```

---

## TS-EDGE-016: Memory Leak Under Load

### Gherkin Scenario

```gherkin
Feature: Memory Leak Detection
  As the DevOps team
  I want to detect memory leaks
  So that services don't crash over time

  Scenario: Run system for 72 hours under load
    Given system handles 1000 trades/hour
    When running continuously for 72 hours
    Then memory usage should remain stable
    And not exceed 2GB
    And no gradual increase (leak)
    And garbage collection should work
```

---

## TS-EDGE-017: Circular Trading Loop

### Gherkin Scenario

```gherkin
Feature: Prevent Circular Trading
  As the trading system
  I want to prevent infinite trade loops
  So that fees don't drain account

  Scenario: Detect buy-sell-buy-sell loop
    Given bot buys and sells BTCUSDT repeatedly
    When more than 10 trades on same symbol in 1 minute
    Then system should detect loop
    And pause trading for symbol
    And require manual intervention
```

---

## TS-EDGE-018: Price with More Than 8 Decimal Places

### Gherkin Scenario

```gherkin
Feature: Ultra-Precise Prices
  As the trading system
  I want to handle very small prices
  So that low-value tokens trade correctly

  Scenario: Trade token worth 0.000000001 USDT
    Given SHIB price is 0.000000001 USDT (9 decimals)
    When I calculate trade value
    Then system should use appropriate precision
    And avoid rounding to zero
```

---

## TS-EDGE-019: AI Model Returns NaN

### Gherkin Scenario

```gherkin
Feature: Invalid AI Predictions
  As the AI service
  I want to handle invalid predictions
  So that trading doesn't break

  Scenario: LSTM model outputs NaN
    Given model predicts: NaN (Not a Number)
    When system receives prediction
    Then should detect invalid value
    And fall back to ensemble of other models
    And log model error for investigation
    And continue trading
```

---

## TS-EDGE-020: Simultaneous Buy and Sell Same Asset

### Gherkin Scenario

```gherkin
Feature: Prevent Conflicting Orders
  As the trading system
  I want to prevent simultaneous opposite orders
  So that users don't cancel themselves out

  Scenario: User places buy and sell orders simultaneously
    Given user has 0.5 BTC
    When user submits:
      - Sell order: 0.5 BTC at 50000
      - Buy order: 0.5 BTC at 50000 (at same time)
    Then system should detect conflict
    And execute in sequence (FIFO)
    And warn user of conflicting orders
```

---

## Edge Case Checklist

When testing edge cases:

- [ ] Identify boundary conditions
- [ ] Test with invalid/extreme inputs
- [ ] Verify error messages are helpful
- [ ] Ensure no crashes or exceptions
- [ ] Check system recovers gracefully
- [ ] Validate default fallback behavior
- [ ] Log errors for debugging
- [ ] Update documentation with findings

---

**Document Control:**
- **Created by**: QA Team
- **Reviewed by**: Engineering Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Edge Cases Document*
