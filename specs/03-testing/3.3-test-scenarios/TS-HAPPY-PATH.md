# Test Scenarios - Happy Path

**Document ID:** TS-HAPPY-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active

---

## Overview

This document contains happy path test scenarios - the most common, successful user workflows through the Bot Core trading platform. These scenarios represent the ideal user experience when everything works as expected.

---

## Scenario Index

| ID | Scenario Name | Priority | Duration | Status |
|----|---------------|----------|----------|--------|
| TS-HAPPY-001 | Complete Trading Workflow | Critical | 5 min | ✅ |
| TS-HAPPY-002 | AI-Driven Automated Trading | Critical | 10 min | ✅ |
| TS-HAPPY-003 | User Registration to First Trade | High | 8 min | ✅ |
| TS-HAPPY-004 | Portfolio Rebalancing | Medium | 3 min | ✅ |
| TS-HAPPY-005 | Paper Trading Practice Session | High | 15 min | ✅ |
| TS-HAPPY-006 | Set and Trigger Stop-Loss | Critical | 5 min | ✅ |
| TS-HAPPY-007 | Multi-Asset Trading Session | Medium | 10 min | ✅ |
| TS-HAPPY-008 | Real-Time Market Monitoring | Medium | Continuous | ✅ |
| TS-HAPPY-009 | Export and Analyze Trade History | Low | 2 min | ✅ |
| TS-HAPPY-010 | Switch from Paper to Live Trading | High | 3 min | ✅ |

---

## TS-HAPPY-001: Complete Trading Workflow

### Description
A registered user logs in, views market data, requests AI analysis, executes a trade, monitors the position, and closes it with profit.

### Gherkin Scenario

```gherkin
Feature: Complete Trading Workflow
  As a trader
  I want to execute a complete trading cycle
  So that I can profit from market movements

  Background:
    Given I am a registered user with email "trader@example.com"
    And my account has 10000 USDT balance
    And paper trading is enabled for safety

  Scenario: Execute successful trade from start to finish
    # Step 1: Login
    When I navigate to login page
    And I enter my credentials
    And I click "Login"
    Then I should be redirected to dashboard
    And I should see my balance: 10000 USDT

    # Step 2: View Market Data
    When I navigate to "Trading" page
    And I select symbol "BTC/USDT"
    Then I should see real-time price updates via WebSocket
    And I should see current price around 50000 USDT
    And candlestick chart should display with 1h interval

    # Step 3: Request AI Analysis
    When I click "Get AI Analysis" button
    Then Rust engine requests analysis from Python AI service
    And Python calculates technical indicators (RSI, MACD, Bollinger)
    And Python runs ML models (LSTM, GRU, Transformer)
    And Python queries GPT-4 for intelligent analysis
    And AI returns signal: "Long" with confidence: 0.78
    And reasoning: "Strong bullish momentum: RSI oversold (28), MACD golden cross, price bounced off lower Bollinger Band"
    And I should see AI recommendation on screen

    # Step 4: Execute Market Buy Order
    When I click "Buy" button
    And I enter quantity: 0.1 BTC
    And I select order type: "Market"
    And I set stop-loss: 2% (49000 USDT)
    And I set take-profit: 5% (52500 USDT)
    And I click "Confirm Order"
    Then order should be submitted to Rust engine
    And Rust validates: balance sufficient, risk limits OK
    And Rust executes order (paper trade simulation)
    And order fills at price: 50100 (with slippage)
    And I should see success message: "Order executed successfully"
    And order should appear in "Open Orders" with status "FILLED"

    # Step 5: Monitor Position
    When I navigate to "Positions" tab
    Then I should see my BTC position:
      | Symbol   | Quantity | Entry Price | Current Price | Unrealized PnL | Status |
      | BTCUSDT  | 0.1      | 50100       | 50100         | 0              | OPEN   |
    When price updates to 51000 via WebSocket
    Then unrealized PnL should update to 90 USDT ((51000-50100)*0.1)
    And PnL color should be green (profit)

    # Step 6: Close Position with Profit
    When price reaches 52000
    And I see unrealized PnL: 190 USDT
    And I decide to take profit
    And I click "Close Position" button
    Then market sell order should execute at 52000
    And position should close
    And realized PnL should be: 190 USDT (minus fees ~2 USDT = 188 USDT net)
    And position status should change to "CLOSED"
    And trade should be recorded in trade history
    And my new balance should be: 10188 USDT

    # Step 7: View Trade in History
    When I navigate to "Trade History" page
    Then I should see both trades:
      | Type | Symbol  | Side | Quantity | Price | PnL    | Status | Timestamp  |
      | BUY  | BTCUSDT | BUY  | 0.1      | 50100 | -      | FILLED | 2025-10-11 10:00 |
      | SELL | BTCUSDT | SELL | 0.1      | 52000 | +188   | FILLED | 2025-10-11 11:30 |
```

### Acceptance Criteria

- [ ] User can log in successfully
- [ ] WebSocket provides real-time price updates
- [ ] AI analysis returns within 5 seconds
- [ ] Order executes within 1 second (paper mode)
- [ ] Position PnL updates in real-time
- [ ] Position closes successfully with profit
- [ ] Trade history shows both transactions
- [ ] Final balance reflects profit accurately

### Prerequisites

- All services running (Rust, Python, Frontend, MongoDB, Redis)
- Test user account exists
- Paper trading mode enabled
- Market data available from Binance (testnet)

---

## TS-HAPPY-002: AI-Driven Automated Trading

### Description
The trading bot automatically executes trades based on AI signals without manual intervention.

### Gherkin Scenario

```gherkin
Feature: AI-Driven Automated Trading
  As a bot operator
  I want the system to trade automatically based on AI signals
  So that I can profit 24/7 without manual intervention

  Background:
    Given automated trading is enabled
    And bot is configured with:
      | Parameter          | Value |
      | symbols            | BTCUSDT, ETHUSDT |
      | analysis_interval  | 5 minutes |
      | min_confidence     | 0.70 |
      | max_position_size  | 30% of balance |
      | stop_loss          | 2% |
      | take_profit        | 5% |

  Scenario: Bot executes automated trade cycle
    # Step 1: Bot Monitors Market
    When current time is 10:00 AM
    Then bot fetches market data for BTC and ETH
    And subscribes to Binance WebSocket for real-time updates
    And updates local price cache every second

    # Step 2: Periodic AI Analysis (every 5 minutes)
    When current time is 10:05 AM
    Then bot triggers AI analysis for BTCUSDT
    And Rust requests analysis from Python AI service with:
      | symbol   | BTCUSDT |
      | interval | 1h      |
      | limit    | 100     |
    And Python calculates indicators:
      - RSI: 32 (oversold)
      - MACD: -150 (bullish crossover imminent)
      - Bollinger: price at lower band
      - ADX: 45 (strong trend)
    And Python runs ensemble ML prediction:
      - LSTM predicts: 51500
      - GRU predicts: 51300
      - Transformer predicts: 51400
      - Ensemble: 51400
    And Python queries GPT-4:
      ```
      Prompt: "Analyze BTC/USDT with RSI=32, MACD=-150 (bullish cross), price=50000 at lower Bollinger. Predict signal."
      Response: {"signal": "Long", "confidence": 0.78, "reasoning": "Strong buy opportunity: oversold RSI, MACD golden cross, price bounce off support"}
      ```
    And AI service returns to Rust:
      ```json
      {
        "signal": "Long",
        "confidence": 0.78,
        "price_prediction": 51400,
        "reasoning": "Strong buy opportunity",
        "timestamp": 1728640500
      }
      ```

    # Step 3: Signal Evaluation
    When Rust receives AI signal
    Then Rust evaluates signal:
      - Confidence 0.78 > minimum 0.70 ✓
      - Signal is "Long" (actionable) ✓
      - No existing BTC position ✓
      - Risk limits OK ✓
    And Rust decides to execute trade

    # Step 4: Automated Order Execution
    When Rust executes automated buy order
    Then Rust calculates position size:
      - Account balance: 10000 USDT
      - Max position size: 30% = 3000 USDT
      - Current BTC price: 50000
      - Quantity: 3000 / 50000 = 0.06 BTC
    And Rust places market buy order:
      - Symbol: BTCUSDT
      - Side: BUY
      - Quantity: 0.06 BTC
      - Type: MARKET
    And order executes at 50050 (slight slippage)
    And Rust automatically sets protective orders:
      - Stop-loss: 50050 * 0.98 = 49049
      - Take-profit: 50050 * 1.05 = 52552

    # Step 5: Position Monitoring
    When position is open
    Then bot monitors position continuously
    And updates unrealized PnL every second
    When price moves to 51000
    Then unrealized PnL = (51000 - 50050) * 0.06 = 57 USDT

    # Step 6: Automated Exit (Take-Profit Triggered)
    When price reaches 52552 (take-profit level)
    Then bot detects take-profit trigger
    And bot executes market sell automatically:
      - Symbol: BTCUSDT
      - Side: SELL
      - Quantity: 0.06 BTC
      - Execution price: 52550
    And position closes
    And realized PnL = (52550 - 50050) * 0.06 = 150 USDT (minus fees ~1.5 = 148.5 net)

    # Step 7: Trade Logging and Notification
    Then trade details are saved to MongoDB
    And bot sends notification:
      ```
      Trade Closed: BTCUSDT
      Entry: 50050, Exit: 52550
      Profit: +148.5 USDT (+4.95%)
      Duration: 2 hours 30 minutes
      ```
    And bot is ready for next signal

    # Step 8: Continuous Operation
    When current time is 10:10 AM
    Then bot waits for next analysis interval (10:10 AM)
    And repeats analysis cycle
```

### Acceptance Criteria

- [ ] Bot fetches market data every 5 minutes
- [ ] AI analysis completes within 5 seconds
- [ ] Bot only trades signals with confidence ≥ 0.70
- [ ] Position size respects 30% limit
- [ ] Stop-loss and take-profit set automatically
- [ ] Position monitored in real-time
- [ ] Take-profit triggers automatic exit
- [ ] All trades logged to database
- [ ] Bot operates continuously without intervention

---

## TS-HAPPY-003: User Registration to First Trade

### Description
A new user's complete journey from registration to placing their first trade.

### Gherkin Scenario

```gherkin
Feature: User Registration to First Trade
  As a new user
  I want to register and place my first trade
  So that I can start trading cryptocurrencies

  Scenario: New user onboarding and first trade
    # Step 1: Registration
    When I visit the homepage
    And I click "Sign Up" button
    Then I should see registration form
    When I enter:
      | Field            | Value                  |
      | Username         | newtrader123           |
      | Email            | newtrader@example.com  |
      | Password         | SecurePass123!@#       |
      | Confirm Password | SecurePass123!@#       |
    And I check "I accept terms and conditions"
    And I click "Register"
    Then account should be created
    And verification email should be sent
    And I should see "Account created! Check your email."

    # Step 2: Email Verification (simulated)
    When I verify my email
    Then my account status should change to "verified"

    # Step 3: First Login
    When I navigate to login page
    And I enter email and password
    And I click "Login"
    Then JWT token should be generated
    And token should be stored in localStorage
    And I should be redirected to dashboard

    # Step 4: Dashboard Welcome
    Then I should see welcome message: "Welcome, newtrader123!"
    And I should see onboarding tutorial
    And I should see account summary:
      | Total Balance | 0 USDT (Paper: 10000 USDT) |
      | Open Positions | 0 |
      | Total PnL | 0 |

    # Step 5: Enable Paper Trading
    When I see prompt "Start with paper trading?"
    And I click "Yes, practice first"
    Then paper trading mode should be enabled
    And I should see paper balance: 10000 USDT
    And banner should show: "Paper Trading Mode Active"

    # Step 6: First Trade Walkthrough
    When I navigate to "Trading" page
    Then I should see trading interface tutorial
    When I select symbol: BTCUSDT
    Then I should see:
      - Live price chart
      - Order form
      - Positions panel
      - AI analysis panel
    When I click "Get AI Recommendation"
    Then AI analysis should load
    And show signal: "Long" with confidence 0.75

    # Step 7: Place First Order
    When I click "Buy BTC" button
    And I enter quantity: 0.05 BTC
    And I click "Place Order"
    Then order confirmation modal should appear:
      ```
      You are about to BUY 0.05 BTC
      Estimated cost: 2500 USDT (paper money)
      This is a PAPER TRADE - no real money involved
      Continue?
      ```
    When I click "Confirm"
    Then order should execute
    And I should see success message
    And confetti animation should play 🎉
    And achievement unlocked: "First Trade!"

    # Step 8: View First Position
    When I navigate to "Positions" tab
    Then I should see my first position:
      - Symbol: BTCUSDT
      - Quantity: 0.05 BTC
      - Entry Price: 50000
      - Status: OPEN
    And tutorial should highlight: "This is your position. Watch it grow!"

  Scenario Complete: New user successfully placed first trade in paper mode
```

---

## TS-HAPPY-004: Portfolio Rebalancing

### Gherkin Scenario

```gherkin
Feature: Portfolio Rebalancing
  As a trader
  I want to rebalance my portfolio to target allocation
  So that my risk is properly diversified

  Background:
    Given I have portfolio:
      | Asset | Value  | Allocation |
      | BTC   | 7000   | 70%        |
      | ETH   | 2000   | 20%        |
      | USDT  | 1000   | 10%        |
    And my target allocation is:
      | Asset | Target |
      | BTC   | 50%    |
      | ETH   | 30%    |
      | USDT  | 20%    |

  Scenario: Rebalance portfolio to target
    When I navigate to "Portfolio" page
    And I click "Rebalance" button
    Then system should calculate required trades:
      - Sell 0.04 BTC (reduce from 70% to 50%)
      - Buy 0.33 ETH (increase from 20% to 30%)
      - Hold USDT (will increase to 20% after sells)
    And show rebalancing plan
    When I click "Execute Rebalancing"
    Then system executes trades in sequence:
      1. Sell 0.04 BTC at 50000 = 2000 USDT
      2. Buy 0.33 ETH at 3000 = 990 USDT
    And final allocation should be:
      | Asset | Value  | Allocation |
      | BTC   | 5000   | 50%        |
      | ETH   | 3000   | 30%        |
      | USDT  | 2000   | 20%        |
```

---

## TS-HAPPY-005: Paper Trading Practice Session

### Gherkin Scenario

```gherkin
Feature: Paper Trading Practice
  As a novice trader
  I want to practice trading without risk
  So that I can learn before using real money

  Scenario: 15-minute paper trading session
    Given paper trading is enabled
    And I have 10000 USDT paper balance
    When I practice trading for 15 minutes:
      # Trade 1: BTC Long
      - Buy 0.1 BTC at 50000 → +100 USDT profit
      # Trade 2: ETH Long
      - Buy 1 ETH at 3000 → +50 USDT profit
      # Trade 3: SOL Short (loss)
      - Short 10 SOL at 100 → -30 USDT loss
    Then final balance: 10120 USDT
    And win rate: 66.67% (2/3)
    And system shows performance summary
    And suggests: "Good start! Ready for live trading?"
```

---

## TS-HAPPY-006: Set and Trigger Stop-Loss

### Gherkin Scenario

```gherkin
Feature: Stop-Loss Protection
  As a risk-conscious trader
  I want stop-losses to protect my capital
  So that losses are limited

  Scenario: Stop-loss successfully limits loss
    Given I buy 0.2 BTC at 50000
    And I set stop-loss at 49000 (2% protection)
    When price drops to 49000
    Then stop-loss triggers immediately
    And market sell executes at ~49000
    And position closes automatically
    And loss is limited to 200 USDT (2%)
    And I receive notification: "Stop-loss triggered for BTCUSDT"
```

---

## TS-HAPPY-007: Multi-Asset Trading Session

### Gherkin Scenario

```gherkin
Feature: Multi-Asset Trading
  As an advanced trader
  I want to trade multiple assets simultaneously
  So that I can diversify and maximize opportunities

  Scenario: Trade BTC, ETH, and SOL concurrently
    Given I have 10000 USDT balance
    When I open positions:
      - BTC: 0.08 BTC (4000 USDT)
      - ETH: 1.0 ETH (3000 USDT)
      - SOL: 20 SOL (2000 USDT)
    Then I have 3 active positions
    And each position tracks PnL independently
    When I view portfolio summary
    Then total portfolio PnL shows combined result
```

---

## TS-HAPPY-008: Real-Time Market Monitoring

### Gherkin Scenario

```gherkin
Feature: Real-Time Market Monitoring
  As a trader
  I want to see live market updates
  So that I can react quickly to opportunities

  Scenario: Monitor live market data via WebSocket
    When I open trading dashboard
    Then WebSocket connects to Rust server
    And subscribes to BTCUSDT stream
    And I receive price updates every second
    And candlestick chart updates in real-time
    And order book shows live bids/asks
    And I can monitor without manual refresh
```

---

## TS-HAPPY-009: Export and Analyze Trade History

### Gherkin Scenario

```gherkin
Feature: Trade History Export
  As a trader
  I want to export my trade history
  So that I can analyze performance offline

  Scenario: Export trades to CSV
    Given I have 50 completed trades
    When I navigate to "Trade History"
    And I select date range: Last 30 days
    And I click "Export CSV"
    Then CSV file should download
    And contain all 50 trades with columns:
      - Date, Symbol, Side, Quantity, Entry, Exit, PnL, Fees
    And I can open in Excel for analysis
```

---

## TS-HAPPY-010: Switch from Paper to Live Trading

### Gherkin Scenario

```gherkin
Feature: Switch to Live Trading
  As an experienced paper trader
  I want to switch to live trading
  So that I can trade with real money

  Scenario: Graduate from paper to live mode
    Given I have been paper trading for 2 weeks
    And my paper PnL is +15%
    And win rate is 68%
    When I navigate to Settings
    And I click "Switch to Live Trading"
    Then I should see warning modal:
      ```
      You are about to enable LIVE TRADING with real money.
      - Paper balance will be archived
      - You will trade with real USDT
      - Losses are real and cannot be reversed
      - Ensure you understand the risks
      ```
    When I check "I understand the risks"
    And I enter password for confirmation
    And I click "Enable Live Trading"
    Then paper trading should be disabled
    And I should see my real Binance balance
    And all new trades will be real
```

---

## Test Execution Checklist

For each happy path scenario:

- [ ] All prerequisites met
- [ ] Services running and healthy
- [ ] Test data prepared
- [ ] Execute scenario step-by-step
- [ ] Verify all expected results
- [ ] Check UI updates correctly
- [ ] Validate database records
- [ ] Confirm no errors in logs
- [ ] Document actual results
- [ ] Mark as Pass/Fail

---

**Document Control:**
- **Created by**: QA Team
- **Reviewed by**: Product Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Happy Path Scenarios Document*
