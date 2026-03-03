# Test Cases - Trading Module

**Document ID:** TC-TRADING-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active
**Related FR:** FR-TRADING (Functional Requirements - Trading)

---

## Table of Contents

1. [Test Case Summary](#test-case-summary)
2. [Market Order Test Cases](#market-order-test-cases)
3. [Limit Order Test Cases](#limit-order-test-cases)
4. [Stop-Loss Order Test Cases](#stop-loss-order-test-cases)
5. [Position Management Test Cases](#position-management-test-cases)
6. [Portfolio Tracking Test Cases](#portfolio-tracking-test-cases)
7. [Risk Validation Test Cases](#risk-validation-test-cases)
8. [Binance API Integration Test Cases](#binance-api-integration-test-cases)
9. [Trade History Test Cases](#trade-history-test-cases)
10. [Paper Trading Test Cases](#paper-trading-test-cases)
11. [Traceability Matrix](#traceability-matrix)

---

## Test Case Summary

| Category | Total Tests | Priority | Coverage |
|----------|-------------|----------|----------|
| Market Orders | 6 | Critical | 100% |
| Limit Orders | 6 | Critical | 100% |
| Stop-Loss Orders | 5 | Critical | 100% |
| Position Management | 7 | Critical | 100% |
| Portfolio Tracking | 5 | High | 100% |
| Risk Validation | 8 | Critical | 100% |
| Binance API Integration | 6 | Critical | 100% |
| Trade History | 4 | Medium | 100% |
| Paper Trading | 6 | High | 100% |
| **TOTAL** | **53** | - | **100%** |

**Test File Locations:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_trading.rs`
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_paper_trading.rs`
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_position_risk_comprehensive.rs`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/pages/TradingPaper.test.tsx`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/hooks/usePaperTrading.test.tsx`

---

## Market Order Test Cases

### TC-TRADING-001: Execute Market Buy Order

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-001

**Prerequisites:**
- User is authenticated
- Trading is enabled (`TRADING_ENABLED=true` or paper trading mode)
- Account has sufficient balance
- Market is open and liquid

**Test Scenario (Gherkin):**
```gherkin
Feature: Market Order Execution
  As a trader
  I want to execute market orders
  So that I can buy/sell assets at current market price

  Scenario: Execute market buy order successfully
    Given I am logged in as "trader@example.com"
    And my account balance is 10000 USDT
    And BTC/USDT market price is 50000
    When I place market buy order for 0.1 BTC
    Then order should be executed immediately
    And I should own 0.1 BTC
    And my USDT balance should decrease by 5000 (0.1 * 50000)
    And transaction fees should be deducted
    And order status should be "FILLED"
    And trade should be recorded in history
```

**Test Steps:**
1. Set up test account with 10000 USDT balance
2. Mock Binance API to return BTC/USDT price: 50000
3. Submit market buy order: symbol="BTCUSDT", side="BUY", quantity=0.1
4. Wait for order execution
5. Verify account balance updated
6. Verify position created
7. Verify trade history recorded

**Expected Results:**
- ✅ Order executed within 1000ms
- ✅ BTC position: 0.1 BTC
- ✅ USDT balance: 10000 - 5000 - fees = ~4995 USDT (0.1% fee)
- ✅ Order status: FILLED
- ✅ Trade record created with timestamp
- ✅ Fees calculated correctly: 5000 * 0.001 = 5 USDT

**Actual Results:** [To be filled during execution]

**Status:** [ ] Pass [ ] Fail [ ] Blocked

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/trading/order_executor.rs::execute_market_order`

---

### TC-TRADING-002: Execute Market Sell Order

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Execute market sell order successfully
    Given I am logged in
    And I own 0.5 BTC
    And BTC/USDT market price is 50000
    When I place market sell order for 0.2 BTC
    Then order should be executed immediately
    And I should own 0.3 BTC (0.5 - 0.2)
    And my USDT balance should increase by ~9980 (0.2 * 50000 * 0.999)
    And order status should be "FILLED"
```

**Test Steps:**
1. Set up account with 0.5 BTC
2. Mock market price: 50000 USDT
3. Submit market sell order: quantity=0.2 BTC
4. Verify execution
5. Verify balances updated

**Expected Results:**
- ✅ BTC balance: 0.3 BTC
- ✅ USDT balance increased by: 10000 - 20 = 9980 USDT (after fees)

---

### TC-TRADING-003: Market Order with Insufficient Balance

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-001, FR-RISK-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Market buy order fails with insufficient balance
    Given my USDT balance is 1000
    And BTC/USDT price is 50000
    When I attempt to buy 0.1 BTC (requires 5000 USDT)
    Then order should be rejected
    And I should see error "Insufficient balance"
    And no order should be placed
    And my balance should remain 1000 USDT
```

**Expected Results:**
- ✅ Order rejected before API call
- ✅ Error: "Insufficient balance: required 5000, available 1000"
- ✅ Balance unchanged
- ✅ No trade record created

---

### TC-TRADING-004: Market Order in Paper Trading Mode

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-001, FR-PAPER-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Execute market order in paper trading mode
    Given paper trading mode is enabled
    And my paper account has 10000 USDT
    When I place market buy order for 0.1 BTC
    Then order should be simulated (not sent to Binance)
    And paper balance should be updated
    And trade should be marked as "PAPER"
    And no real funds should be affected
```

**Test Steps:**
1. Enable paper trading: `TRADING_MODE=paper`
2. Initialize paper account with 10000 USDT
3. Execute market buy order
4. Verify order not sent to Binance API
5. Verify paper balance updated

**Expected Results:**
- ✅ No actual Binance API call made
- ✅ Paper balance updated correctly
- ✅ Trade marked with `is_paper: true`

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_paper_trading.rs`

---

### TC-TRADING-005: Market Order Price Slippage

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Market order handles price slippage
    Given market price is 50000 USDT
    And I place market buy for 0.1 BTC
    But actual execution price is 50100 (0.2% slippage)
    Then order should execute at 50100
    And I should pay 5010 USDT (instead of 5000)
    And slippage should be recorded
```

**Expected Results:**
- ✅ Order executed despite slippage
- ✅ Actual execution price recorded
- ✅ Slippage percentage calculated: 0.2%

---

### TC-TRADING-006: Market Order Validation

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-TRADING-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Market order validation
    Given I attempt to place market order
    When order parameters are <parameters>
    Then validation should <result>

    Examples:
      | parameters                     | result |
      | quantity = 0                   | fail   |
      | quantity < 0                   | fail   |
      | symbol = ""                    | fail   |
      | symbol = "INVALID"             | fail   |
      | quantity = 0.1, symbol = "BTCUSDT" | pass   |
```

---

## Limit Order Test Cases

### TC-TRADING-007: Place Limit Buy Order

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-002

**Test Scenario (Gherkin):**
```gherkin
Feature: Limit Order Execution
  As a trader
  I want to place limit orders
  So that I can buy/sell at specific prices

  Scenario: Place limit buy order below market price
    Given I am logged in
    And my USDT balance is 10000
    And BTC/USDT market price is 50000
    When I place limit buy order at 49000 for 0.1 BTC
    Then order should be placed successfully
    And order status should be "NEW" or "PENDING"
    And 4900 USDT should be reserved
    And order should appear in open orders
    When market price drops to 49000
    Then order should be executed automatically
    And order status should change to "FILLED"
```

**Test Steps:**
1. Set up account with 10000 USDT
2. Current market price: 50000
3. Place limit buy: price=49000, quantity=0.1
4. Verify order placed
5. Simulate price drop to 49000
6. Verify order execution

**Expected Results:**
- ✅ Order placed with status: NEW
- ✅ Reserved balance: 4900 USDT
- ✅ Available balance: 5100 USDT
- ✅ Order auto-executes when price reached

---

### TC-TRADING-008: Place Limit Sell Order

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Place limit sell order above market price
    Given I own 0.5 BTC
    And market price is 50000
    When I place limit sell order at 51000 for 0.2 BTC
    Then order should be placed
    And 0.2 BTC should be reserved
    And available BTC should be 0.3
    When market price rises to 51000
    Then order should execute
    And I receive ~10180 USDT (0.2 * 51000 * 0.999)
```

---

### TC-TRADING-009: Cancel Limit Order

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Cancel pending limit order
    Given I have limit buy order at 49000 for 0.1 BTC
    And order status is "NEW"
    And 4900 USDT is reserved
    When I cancel the order
    Then order status should change to "CANCELED"
    And reserved USDT should be released
    And available balance should increase by 4900
    And order should be removed from open orders
```

**Expected Results:**
- ✅ Order canceled successfully
- ✅ Funds released immediately
- ✅ Order removed from open orders list

---

### TC-TRADING-010: Limit Order Partial Fill

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Limit order partially filled
    Given I place limit buy for 1.0 BTC at 50000
    When market price reaches 50000
    But only 0.6 BTC available at that price
    Then order should be partially filled with 0.6 BTC
    And order status should be "PARTIALLY_FILLED"
    And remaining 0.4 BTC order should stay open
```

---

### TC-TRADING-011: Limit Order Expiration

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Limit order expires after time limit
    Given I place limit buy with Time-In-Force = GTT (Good Till Time)
    And expiration time is 1 hour
    When 1 hour passes without execution
    Then order should be automatically canceled
    And status should be "EXPIRED"
    And funds should be released
```

---

### TC-TRADING-012: Limit Order Price Validation

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-TRADING-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Limit order price validation
    Given market price is 50000
    When I place limit <side> order at <price>
    Then validation should <result>

    Examples:
      | side | price  | result                        |
      | BUY  | 0      | fail (price must be positive) |
      | BUY  | -1000  | fail (negative price)         |
      | SELL | 0      | fail (price must be positive) |
      | BUY  | 49000  | pass                          |
      | SELL | 51000  | pass                          |
```

---

## Stop-Loss Order Test Cases

### TC-TRADING-013: Place Stop-Loss Sell Order

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-003, FR-RISK-002

**Test Scenario (Gherkin):**
```gherkin
Feature: Stop-Loss Orders
  As a trader
  I want to set stop-loss orders
  So that I can limit my losses

  Scenario: Stop-loss triggers on price drop
    Given I own 0.5 BTC bought at 50000
    And current price is 50000
    When I place stop-loss sell at 49000 for 0.5 BTC
    Then stop-loss order should be created
    When price drops to 49000
    Then stop-loss should trigger
    And market sell order should execute at ~49000
    And position should be closed
```

**Test Steps:**
1. Set up position: 0.5 BTC at entry price 50000
2. Place stop-loss: trigger=49000, quantity=0.5
3. Simulate price drop to 49000
4. Verify stop-loss triggered
5. Verify market sell executed

**Expected Results:**
- ✅ Stop-loss created successfully
- ✅ Triggers at correct price
- ✅ Market order executed immediately
- ✅ Position closed
- ✅ Loss limited to ~2% (1000/50000)

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/trading/stop_loss.rs`

---

### TC-TRADING-014: Stop-Loss Prevents Larger Losses

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-RISK-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Stop-loss limits losses during crash
    Given I own 1.0 BTC at entry 50000
    And I set stop-loss at 49000 (2% loss)
    When price crashes to 45000
    Then stop-loss should trigger at 49000
    And I should sell at ~49000 (not 45000)
    And loss should be limited to ~1000 USDT
    And I avoid additional 4000 USDT loss
```

---

### TC-TRADING-015: Trailing Stop-Loss

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Trailing stop-loss adjusts with price
    Given I own 1.0 BTC at 50000
    And I set trailing stop at 2% below highest price
    When price rises to 52000
    Then stop-loss should adjust to 50960 (52000 * 0.98)
    When price rises to 55000
    Then stop-loss should adjust to 53900 (55000 * 0.98)
    When price drops to 53900
    Then stop-loss should trigger
    And I lock in profit of ~3900 (53900 - 50000)
```

---

### TC-TRADING-016: Stop-Loss with Slippage

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Stop-loss executes with slippage during volatility
    Given I have stop-loss at 49000
    When price drops rapidly from 50000 to 48000
    Then stop-loss triggers at 49000
    But execution may occur at 48900 (1% slippage)
    And slippage should be recorded
```

---

### TC-TRADING-017: Multiple Stop-Loss Orders

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Multiple stop-loss orders for same position
    Given I own 1.0 BTC
    When I set stop-loss #1 at 49000 for 0.5 BTC
    And I set stop-loss #2 at 48000 for 0.5 BTC
    Then both orders should be active
    When price drops to 49000
    Then only stop-loss #1 should trigger
    And stop-loss #2 should remain active
```

---

## Position Management Test Cases

### TC-TRADING-018: Open Long Position

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-004

**Test Scenario (Gherkin):**
```gherkin
Feature: Position Management
  As a trader
  I want to manage my positions
  So that I can track my trades

  Scenario: Open long position
    Given I have no open positions
    When I buy 0.5 BTC at 50000
    Then a long position should be created
    And position details should be:
      | Field        | Value     |
      | symbol       | BTCUSDT   |
      | side         | LONG      |
      | quantity     | 0.5       |
      | entry_price  | 50000     |
      | current_pnl  | 0         |
      | status       | OPEN      |
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/trading/position_manager.rs`

---

### TC-TRADING-019: Close Long Position

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-TRADING-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Close long position with profit
    Given I have long position: 0.5 BTC at entry 50000
    And current price is 52000
    When I sell 0.5 BTC at 52000
    Then position should be closed
    And realized PnL should be 1000 USDT (0.5 * (52000 - 50000))
    And position status should be "CLOSED"
```

**Expected Results:**
- ✅ Position closed completely
- ✅ PnL calculated: (exit_price - entry_price) * quantity = 1000 USDT
- ✅ Fees deducted from profit

---

### TC-TRADING-020: Calculate Unrealized PnL

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-TRADING-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate unrealized PnL for open position
    Given I have long position: 0.5 BTC at entry 50000
    When current price is 51000
    Then unrealized PnL should be 500 USDT
    When current price is 49000
    Then unrealized PnL should be -500 USDT
```

**Formula:**
```
Unrealized PnL = (Current Price - Entry Price) * Quantity
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_trading.rs::test_pnl_calculation` (line 18-30)

---

### TC-TRADING-021: Position Averaging (DCA)

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Average down position (Dollar Cost Averaging)
    Given I have long position: 0.5 BTC at entry 50000
    When I buy additional 0.5 BTC at 48000
    Then position quantity should be 1.0 BTC
    And average entry price should be 49000
    And calculation: (0.5*50000 + 0.5*48000) / 1.0 = 49000
```

---

### TC-TRADING-022: Position Liquidation Check

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-RISK-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Check position for liquidation
    Given I have leveraged position: 1.0 BTC at 50000 with 5x leverage
    And liquidation price is 46000
    When current price is 47000
    Then position should be safe (above liquidation)
    When current price drops to 46000
    Then liquidation warning should trigger
    When price drops to 45900
    Then position should be liquidated
    And all collateral should be lost
```

---

### TC-TRADING-023: Multiple Concurrent Positions

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Manage multiple positions simultaneously
    Given I open position #1: 0.5 BTC/USDT
    And I open position #2: 10 ETH/USDT
    And I open position #3: 100 SOL/USDT
    Then I should have 3 active positions
    And each position should track PnL independently
    And total portfolio PnL should be sum of all positions
```

---

### TC-TRADING-024: Position Size Calculation

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-RISK-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate position size based on risk
    Given account balance is 10000 USDT
    And I want to risk 2% per trade (200 USDT)
    And stop-loss is 5% from entry
    When I calculate position size
    Then position size should be 4000 USDT
    And calculation: 200 / 0.05 = 4000
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_trading.rs::test_trading_calculations` (line 4-15)

---

## Portfolio Tracking Test Cases

### TC-TRADING-025: Portfolio Value Calculation

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-005

**Test Scenario (Gherkin):**
```gherkin
Feature: Portfolio Tracking
  As a trader
  I want to track my portfolio value
  So that I know my total net worth

  Scenario: Calculate total portfolio value
    Given I have:
      | Asset | Quantity | Current Price | Value  |
      | BTC   | 0.5      | 50000         | 25000  |
      | ETH   | 10       | 3000          | 30000  |
      | USDT  | 5000     | 1             | 5000   |
    Then total portfolio value should be 60000 USDT
```

---

### TC-TRADING-026: Portfolio PnL Tracking

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Track portfolio total PnL
    Given initial portfolio value was 50000 USDT
    And current portfolio value is 55000 USDT
    Then total PnL should be +5000 USDT
    And PnL percentage should be +10%
```

---

### TC-TRADING-027: Portfolio Diversification

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate portfolio allocation percentages
    Given total portfolio value is 100000 USDT
    And BTC holdings worth 40000 USDT
    And ETH holdings worth 30000 USDT
    And cash holdings worth 30000 USDT
    Then allocation should be:
      | Asset | Percentage |
      | BTC   | 40%        |
      | ETH   | 30%        |
      | USDT  | 30%        |
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_trading.rs::test_portfolio_allocation` (line 89-115)

---

### TC-TRADING-028: Portfolio History Tracking

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Track portfolio value over time
    Given I track portfolio value daily
    When I query portfolio history for last 30 days
    Then I should receive 30 data points
    And each data point should include:
      - Date
      - Total value
      - Daily PnL
      - Daily PnL percentage
```

---

### TC-TRADING-029: Portfolio Rebalancing

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Rebalance portfolio to target allocation
    Given target allocation is 50% BTC, 50% USDT
    And current allocation is 70% BTC, 30% USDT
    When I rebalance portfolio
    Then system should suggest:
      - Sell 20% of BTC holdings
      - Hold proceeds in USDT
```

---

## Risk Validation Test Cases

### TC-TRADING-030: Risk Per Trade Limit

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-RISK-001

**Test Scenario (Gherkin):**
```gherkin
Feature: Risk Management
  As the trading system
  I want to enforce risk limits
  So that traders don't lose too much capital

  Scenario: Enforce maximum risk per trade
    Given account balance is 10000 USDT
    And maximum risk per trade is 2% (200 USDT)
    When I attempt trade that risks 500 USDT (5%)
    Then trade should be rejected
    And error should be "Risk exceeds limit: 500 > 200"
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/risk/validator.rs`

---

### TC-TRADING-031: Maximum Position Size Limit

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-RISK-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Enforce maximum position size
    Given account balance is 10000 USDT
    And max position size is 30% of account (3000 USDT)
    When I attempt to buy BTC worth 5000 USDT
    Then trade should be rejected
    And error should be "Position size exceeds limit"
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_trading.rs::test_max_position_size` (line 148-160)

---

### TC-TRADING-032: Leverage Limit

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-RISK-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Enforce maximum leverage
    Given maximum allowed leverage is 10x
    When I attempt trade with 20x leverage
    Then trade should be rejected
    And error should be "Leverage exceeds maximum: 20x > 10x"
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_trading.rs::test_leverage_calculation` (line 33-44)

---

### TC-TRADING-033: Daily Loss Limit

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-RISK-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Stop trading after daily loss limit reached
    Given account balance is 10000 USDT
    And daily loss limit is 5% (500 USDT)
    And I have already lost 500 USDT today
    When I attempt another trade
    Then trade should be blocked
    And error should be "Daily loss limit reached"
    And trading should be disabled until next day
```

---

### TC-TRADING-034: Margin Call Warning

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-RISK-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Trigger margin call warning
    Given I have leveraged position worth 50000 USDT
    And my margin is 10000 USDT
    And margin call threshold is 30% (3000 USDT)
    When my margin drops to 3500 USDT
    Then margin call warning should trigger
    And notification should be sent
    When margin drops to 3000 USDT
    Then I should be required to add margin
```

---

### TC-TRADING-035: Risk/Reward Ratio Validation

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-RISK-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Enforce minimum risk/reward ratio
    Given minimum risk/reward ratio is 1:2
    And entry price is 100
    And stop-loss is 95 (risk: 5)
    When take-profit is 105 (reward: 5)
    Then risk/reward is 1:1 (fails minimum)
    And trade should be rejected
    When take-profit is 110 (reward: 10)
    Then risk/reward is 1:2 (meets minimum)
    And trade should be allowed
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_trading.rs::test_risk_reward_ratio` (line 74-86)

---

### TC-TRADING-036: Concentration Risk

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-RISK-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Limit concentration in single asset
    Given maximum allocation per asset is 40%
    And I already have 35% in BTC
    When I attempt trade that increases BTC to 45%
    Then trade should be rejected
    And error should be "Asset concentration exceeds limit"
```

---

### TC-TRADING-037: Drawdown Limit

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-RISK-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Stop trading after maximum drawdown
    Given initial account balance was 10000 USDT
    And maximum drawdown is 20% (2000 USDT)
    When account balance drops to 8000 USDT
    Then drawdown is 20% (at limit)
    And warning should be issued
    When balance drops to 7900 USDT
    Then drawdown exceeds limit
    And all positions should be closed
    And trading should be disabled
```

---

## Binance API Integration Test Cases

### TC-TRADING-038: Binance API Connection

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INTEGRATION-001

**Test Scenario (Gherkin):**
```gherkin
Feature: Binance API Integration
  As the trading system
  I want to connect to Binance API
  So that I can execute trades

  Scenario: Successfully connect to Binance API
    Given Binance API credentials are configured
    When I initialize Binance client
    Then connection should be established
    And API should be reachable
    And I should receive server time
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_binance_client.rs`

---

### TC-TRADING-039: Binance Testnet vs Production

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INTEGRATION-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Use Binance Testnet by default
    Given BINANCE_TESTNET environment variable is true
    When I initialize trading system
    Then Binance client should connect to testnet
    And base URL should be "https://testnet.binance.vision"
    And no real funds should be at risk

  Scenario: Switch to production requires explicit flag
    Given BINANCE_TESTNET is false
    And TRADING_ENABLED is false (safety)
    When I initialize trading system
    Then system should block production trading
    And error should be "Production trading requires explicit enablement"
```

---

### TC-TRADING-040: Binance API Rate Limiting

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INTEGRATION-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle Binance API rate limits
    Given Binance API rate limit is 1200 requests/minute
    When I make 1200 requests in 1 minute
    Then all requests should succeed
    When I make request #1201
    Then request should be rate limited
    And I should receive 429 Too Many Requests
    And system should back off and retry
```

---

### TC-TRADING-041: Binance WebSocket Connection

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-INTEGRATION-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Connect to Binance WebSocket for real-time data
    Given I subscribe to BTC/USDT ticker
    When WebSocket connection is established
    Then I should receive real-time price updates
    And updates should arrive every second
    And data should include: price, volume, timestamp
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_websocket.rs`

---

### TC-TRADING-042: Binance API Error Handling

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INTEGRATION-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Handle Binance API errors gracefully
    When Binance API returns <error_code>
    Then system should handle error
    And error message should be <message>

    Examples:
      | error_code | message                          |
      | 401        | Invalid API key                  |
      | 403        | Insufficient permissions         |
      | 429        | Rate limit exceeded              |
      | -1021      | Timestamp out of sync            |
      | -2010      | Insufficient balance             |
```

---

### TC-TRADING-043: Binance Order Status Sync

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-INTEGRATION-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Sync order status with Binance
    Given I place order on Binance
    And order status is "NEW"
    When order is partially filled on Binance
    Then local status should update to "PARTIALLY_FILLED"
    When order is fully filled on Binance
    Then local status should update to "FILLED"
```

---

## Trade History Test Cases

### TC-TRADING-044: Record Trade in History

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-TRADING-006

**Test Scenario (Gherkin):**
```gherkin
Feature: Trade History
  As a trader
  I want to view my trade history
  So that I can analyze past trades

  Scenario: Record trade execution in history
    Given I execute market buy of 0.1 BTC at 50000
    Then trade should be recorded with:
      | Field          | Value           |
      | symbol         | BTCUSDT         |
      | side           | BUY             |
      | type           | MARKET          |
      | quantity       | 0.1             |
      | price          | 50000           |
      | timestamp      | <current_time>  |
      | fees           | 5 USDT          |
      | status         | FILLED          |
```

---

### TC-TRADING-045: Query Trade History

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Query trades by date range
    Given I have 100 trades in history
    When I query trades from "2025-10-01" to "2025-10-11"
    Then I should receive trades within that date range
    And trades should be sorted by timestamp descending
```

---

### TC-TRADING-046: Trade History Pagination

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-TRADING-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Paginate through large trade history
    Given I have 1000 trades in history
    When I query with page=1, limit=50
    Then I should receive 50 trades
    And response should include total_pages = 20
    When I query page=2
    Then I should receive trades 51-100
```

---

### TC-TRADING-047: Export Trade History

**Priority:** Low
**Test Type:** Integration
**Related FR:** FR-TRADING-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Export trade history to CSV
    Given I have trade history
    When I export trades to CSV
    Then CSV should include all trade fields
    And CSV should be downloadable
```

---

## Paper Trading Test Cases

### TC-TRADING-048: Initialize Paper Trading Account

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-PAPER-001

**Test Scenario (Gherkin):**
```gherkin
Feature: Paper Trading
  As a new user
  I want to practice with paper trading
  So that I can learn without risking real money

  Scenario: Initialize paper trading account
    Given I enable paper trading mode
    When system initializes paper account
    Then I should have virtual balance of 10000 USDT
    And account should be marked as "PAPER"
    And no real API credentials required
```

**Code Location:**
- Rust: `/Users/dungngo97/Documents/bot-core/rust-core-engine/tests/test_paper_trading.rs`
- Frontend: `/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/__tests__/pages/TradingPaper.test.tsx`

---

### TC-TRADING-049: Paper Trade Execution

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-PAPER-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Execute trade in paper mode
    Given paper trading is enabled
    And paper balance is 10000 USDT
    When I place market buy for 0.1 BTC at 50000
    Then trade should be simulated
    And no actual Binance API call made
    And paper balance should update
    And trade should be marked with is_paper: true
```

---

### TC-TRADING-050: Paper Trading Uses Real Market Data

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-PAPER-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Paper trading uses real-time market prices
    Given paper trading is enabled
    When I query market price for BTC/USDT
    Then price should come from real Binance data
    And paper trades should execute at real prices
```

---

### TC-TRADING-051: Paper Trading Performance Tracking

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-PAPER-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Track paper trading performance
    Given I start with 10000 USDT paper balance
    And I execute multiple paper trades
    When I check performance
    Then I should see:
      - Current paper balance
      - Total PnL
      - Win rate
      - Number of trades
```

---

### TC-TRADING-052: Switch from Paper to Live Trading

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-PAPER-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Switch from paper to live trading
    Given I have been paper trading
    When I switch to live trading
    Then I should see warning about real money
    And I should confirm I understand risks
    And paper positions should not transfer
    And I start with real account balance
```

---

### TC-TRADING-053: Paper Trading Reset

**Priority:** Low
**Test Type:** Integration
**Related FR:** FR-PAPER-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Reset paper trading account
    Given my paper account has 5000 USDT (down from 10000)
    When I reset paper account
    Then balance should reset to 10000 USDT
    And all paper positions should be closed
    And paper trade history should be archived
```

---

## Traceability Matrix

| Test Case ID | Related FR | Priority | Code Location | Status |
|--------------|------------|----------|---------------|--------|
| TC-TRADING-001 | FR-TRADING-001 | Critical | `trading/order_executor.rs` | ✅ |
| TC-TRADING-007 | FR-TRADING-002 | Critical | `trading/limit_orders.rs` | ✅ |
| TC-TRADING-013 | FR-TRADING-003 | Critical | `trading/stop_loss.rs` | ✅ |
| TC-TRADING-018 | FR-TRADING-004 | Critical | `trading/position_manager.rs` | ✅ |
| TC-TRADING-020 | FR-TRADING-004 | High | `tests/test_trading.rs::test_pnl_calculation` | ✅ |
| TC-TRADING-024 | FR-RISK-004 | Critical | `tests/test_trading.rs::test_trading_calculations` | ✅ |
| TC-TRADING-027 | FR-TRADING-005 | Medium | `tests/test_trading.rs::test_portfolio_allocation` | ✅ |
| TC-TRADING-030 | FR-RISK-001 | Critical | `risk/validator.rs` | ✅ |
| TC-TRADING-038 | FR-INTEGRATION-001 | Critical | `tests/test_binance_client.rs` | ✅ |
| TC-TRADING-041 | FR-INTEGRATION-002 | Critical | `tests/test_websocket.rs` | ✅ |
| TC-TRADING-048 | FR-PAPER-001 | High | `tests/test_paper_trading.rs` | ✅ |

---

## Acceptance Criteria

Trading module is considered complete when:

- [ ] All 53 test cases pass
- [ ] Code coverage >= 90% for trading module
- [ ] All order types (market, limit, stop-loss) work correctly
- [ ] Position management tracks PnL accurately
- [ ] Risk limits enforced on all trades
- [ ] Binance API integration functional
- [ ] Paper trading mode works without real money
- [ ] Trade history recorded correctly
- [ ] Performance: Order execution < 1000ms (P95)
- [ ] No critical bugs in production

---

## Test Execution Summary

**Last Execution Date:** [To be filled]
**Executed By:** [Name]

| Category | Total | Passed | Failed | Blocked |
|----------|-------|--------|--------|---------|
| Market Orders | 6 | - | - | - |
| Limit Orders | 6 | - | - | - |
| Stop-Loss | 5 | - | - | - |
| Position Mgmt | 7 | - | - | - |
| Portfolio | 5 | - | - | - |
| Risk Validation | 8 | - | - | - |
| Binance API | 6 | - | - | - |
| Trade History | 4 | - | - | - |
| Paper Trading | 6 | - | - | - |
| **TOTAL** | **53** | **-** | **-** | **-** |

---

**Document Control:**
- **Created by**: QA Team
- **Reviewed by**: Trading Team, Risk Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of Trading Test Cases Document*
