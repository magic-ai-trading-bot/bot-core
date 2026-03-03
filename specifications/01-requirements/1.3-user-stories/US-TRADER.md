# Trader User Stories

**Spec ID**: US-TRADER
**Version**: 1.0
**Status**: ☑ Approved
**Owner**: Product Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] Requirements gathered from trader personas
- [x] User stories documented with acceptance criteria
- [x] Linked to functional requirements
- [x] Prioritized by business value
- [x] Reviewed with stakeholders
- [ ] Validated with end users
- [ ] Test scenarios defined
- [ ] Implementation tracking

---

## Metadata

**Related Specs**:
- Related FR: [FR-AUTH](../1.1-functional-requirements/FR-AUTH.md) - Authentication
- Related FR: [FR-TRADING](../1.1-functional-requirements/FR-TRADING.md) - Trading Engine
- Related FR: [FR-AI](../1.1-functional-requirements/FR-AI.md) - AI Analysis
- Related FR: [FR-PAPER-TRADING](../1.1-functional-requirements/FR-PAPER-TRADING.md) - Paper Trading
- Related FR: [FR-PORTFOLIO](../1.1-functional-requirements/FR-PORTFOLIO.md) - Portfolio Management
- Related FR: [FR-RISK](../1.1-functional-requirements/FR-RISK.md) - Risk Management

**Dependencies**:
- Depends on: Functional Requirements, Business Rules, Data Models
- Blocks: Test Case creation, UI/UX design

**Business Value**: Critical
**Technical Complexity**: N/A
**Priority**: ☑ Critical

---

## Overview

This specification documents all user stories from the **Trader** perspective. Traders are the primary users of the Bot Core platform who use the system to execute cryptocurrency trades, manage their portfolios, configure trading strategies, and monitor performance. These user stories capture the needs, goals, and workflows of traders at all experience levels.

---

## Business Context

**Problem Statement**:
Traders need an intuitive, reliable platform to automate cryptocurrency trading based on AI-powered signals while maintaining control over risk parameters, position sizes, and trading strategies.

**Target Users**:
- **Novice Traders**: Learning cryptocurrency trading with paper trading
- **Intermediate Traders**: Using AI signals with manual oversight
- **Advanced Traders**: Customizing strategies and optimizing parameters
- **Professional Traders**: Managing multiple strategies and portfolios

**Business Goals**:
- Enable traders to execute profitable trades with AI assistance
- Provide comprehensive risk management controls
- Support learning through paper trading mode
- Deliver real-time insights and analytics
- Build trust through transparency and control

**Success Metrics**:
- User registration rate: 100+ per month
- Active traders: 70% of registered users
- Average profitability: 15%+ per quarter
- User retention: 80%+ after 3 months
- Paper-to-live conversion: 40% within 2 weeks

---

## User Stories

### US-TRADER-001: Account Registration

**User Story:**
As a **trader**, I want to **register an account with email and password** so that **I can access the trading platform and start using trading features**.

**Acceptance Criteria:**
- [ ] Given I am on the registration page
- [ ] When I enter a valid email address (RFC 5322 format)
- [ ] And I enter a password with minimum 6 characters
- [ ] And I confirm the password matches
- [ ] And I optionally enter my full name
- [ ] Then my account is created with default settings
- [ ] And I receive a JWT token valid for 7 days
- [ ] And I am automatically logged in
- [ ] And I am redirected to the dashboard
- [ ] And I see a welcome message with my email

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: Low
**Related FR**: FR-AUTH-004
**Test Cases**: TC-AUTH-009, TC-AUTH-010, TC-E2E-001

---

### US-TRADER-002: User Login

**User Story:**
As a **trader**, I want to **log in with my email and password** so that **I can access my trading account and resume trading activities**.

**Acceptance Criteria:**
- [ ] Given I am on the login page
- [ ] When I enter my registered email address
- [ ] And I enter my correct password
- [ ] And my account is active (not deactivated)
- [ ] Then I am authenticated with a JWT token
- [ ] And my last_login timestamp is updated
- [ ] And I am redirected to the dashboard
- [ ] And I see my portfolio balance and open positions
- [ ] And I can access all protected features

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: Low
**Related FR**: FR-AUTH-005
**Test Cases**: TC-AUTH-012, TC-AUTH-013, TC-E2E-002

---

### US-TRADER-003: Paper Trading Practice

**User Story:**
As a **trader**, I want to **practice trading with virtual money** so that **I can test strategies and learn without risking real capital**.

**Acceptance Criteria:**
- [ ] Given I have a registered account
- [ ] When I navigate to the paper trading section
- [ ] Then I see a paper portfolio with $10,000 starting balance
- [ ] And I can execute paper trades with real market prices
- [ ] And I see realistic slippage (0.05%) and fees (0.04%)
- [ ] And my paper trades are tracked separately from live trades
- [ ] And I can view my paper trading performance metrics
- [ ] And I can reset my paper portfolio at any time
- [ ] And paper trading results do not affect my real balance

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-PAPER-TRADING-001, FR-PAPER-TRADING-002
**Test Cases**: TC-PAPER-001, TC-PAPER-002, TC-E2E-010

---

### US-TRADER-004: Live Trading Activation

**User Story:**
As a **trader**, I want to **enable live trading after paper trading success** so that **I can start making real trades with my capital**.

**Acceptance Criteria:**
- [ ] Given I have completed at least 10 successful paper trades
- [ ] And my paper trading win rate is above 50%
- [ ] When I navigate to trading settings
- [ ] And I toggle "Enable Live Trading" switch
- [ ] Then I see a confirmation dialog with warnings
- [ ] And I must accept the risks and terms
- [ ] And my Binance API keys are validated
- [ ] And trading_enabled flag is set to true
- [ ] And I can now execute live trades
- [ ] And I receive a notification confirming live trading activation

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-TRADING-001, FR-RISK-001
**Test Cases**: TC-TRADING-001, TC-E2E-015

---

### US-TRADER-005: AI Trading Signal Review

**User Story:**
As a **trader**, I want to **review AI-generated trading signals before execution** so that **I can make informed decisions and maintain control over trades**.

**Acceptance Criteria:**
- [ ] Given the AI service has generated a trading signal
- [ ] When I view the signal on the dashboard
- [ ] Then I see the signal type (Long/Short/Neutral)
- [ ] And I see the confidence score (0.0-1.0)
- [ ] And I see detailed reasoning for the signal
- [ ] And I see multi-timeframe analysis (1H, 4H indicators)
- [ ] And I see strategy scores for each trading strategy
- [ ] And I see market analysis (trend, support/resistance, volatility)
- [ ] And I see risk assessment with stop-loss and take-profit suggestions
- [ ] And I can accept or reject the signal
- [ ] And rejected signals are logged for analysis

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-AI-006, FR-TRADING-001
**Test Cases**: TC-AI-015, TC-TRADING-005, TC-E2E-020

---

### US-TRADER-006: Manual Trade Execution

**User Story:**
As a **trader**, I want to **manually execute trades based on my analysis** so that **I can override AI signals or trade independently**.

**Acceptance Criteria:**
- [ ] Given I am viewing a trading symbol (e.g., BTCUSDT)
- [ ] When I click "Execute Manual Trade" button
- [ ] Then I see a trade execution form
- [ ] And I can select side (BUY or SELL)
- [ ] And I can enter quantity (validated against min order size)
- [ ] And I can set stop-loss percentage (max 10%)
- [ ] And I can set take-profit percentage
- [ ] And I can select leverage (1x-125x based on symbol)
- [ ] And I see estimated position value and margin required
- [ ] When I submit the trade
- [ ] Then risk validation is performed
- [ ] And the order is sent to Binance
- [ ] And I receive confirmation with order details
- [ ] And the position appears in my portfolio

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-TRADING-001, FR-TRADING-003, FR-RISK-001
**Test Cases**: TC-TRADING-040, TC-E2E-016

---

### US-TRADER-007: Portfolio Overview

**User Story:**
As a **trader**, I want to **see my complete portfolio at a glance** so that **I can understand my current positions, equity, and performance**.

**Acceptance Criteria:**
- [ ] Given I am logged in and on the dashboard
- [ ] When I view the portfolio section
- [ ] Then I see my total equity value (cash + unrealized PnL)
- [ ] And I see my available balance for new trades
- [ ] And I see total margin used across all positions
- [ ] And I see free margin available
- [ ] And I see margin level percentage (equity/margin * 100)
- [ ] And I see a list of all open positions with:
  - Symbol, side (LONG/SHORT), quantity
  - Entry price, current price
  - Unrealized PnL ($ and %)
  - Stop-loss and take-profit levels
- [ ] And I see total unrealized PnL across all positions
- [ ] And I see position count (e.g., 3/10 positions)
- [ ] And portfolio values update in real-time (every 5 seconds)

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-PORTFOLIO-001, FR-TRADING-003
**Test Cases**: TC-PORTFOLIO-001, TC-E2E-012

---

### US-TRADER-008: Trade History Review

**User Story:**
As a **trader**, I want to **review my past trades with detailed information** so that **I can analyze my performance and learn from successes and mistakes**.

**Acceptance Criteria:**
- [ ] Given I have executed trades (paper or live)
- [ ] When I navigate to the Trade History page
- [ ] Then I see a chronological list of all closed trades
- [ ] And each trade shows:
  - Symbol, side (BUY/SELL), quantity
  - Entry date/time and exit date/time
  - Entry price and exit price
  - Stop-loss and take-profit levels
  - Realized PnL ($ and %)
  - Close reason (StopLoss/TakeProfit/Manual)
  - Strategy used (RSI/MACD/etc.)
- [ ] And I can filter trades by:
  - Date range
  - Symbol
  - Profit/Loss
  - Strategy
- [ ] And I can sort trades by date, PnL, symbol
- [ ] And I can export trade history to CSV
- [ ] And I see summary statistics (total trades, win rate, total PnL)

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-TRADING-008, FR-PORTFOLIO-002
**Test Cases**: TC-TRADING-018, TC-E2E-013

---

### US-TRADER-009: Performance Analytics

**User Story:**
As a **trader**, I want to **see comprehensive performance metrics and analytics** so that **I can evaluate my trading effectiveness and identify areas for improvement**.

**Acceptance Criteria:**
- [ ] Given I have completed at least 5 trades
- [ ] When I view the Analytics/Performance page
- [ ] Then I see key metrics:
  - Total trades executed
  - Win rate (winning trades / total trades)
  - Total realized PnL ($ and %)
  - Average win amount
  - Average loss amount
  - Largest win and largest loss
  - Profit factor (gross profit / gross loss)
  - Maximum drawdown ($ and %)
  - Sharpe ratio (risk-adjusted returns)
- [ ] And I see performance charts:
  - Equity curve over time
  - Drawdown chart
  - PnL distribution histogram
  - Win/loss streak chart
- [ ] And I see strategy comparison (performance by strategy)
- [ ] And I see symbol performance breakdown
- [ ] And metrics update after each trade closure

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-TRADING-019, FR-PORTFOLIO-003
**Test Cases**: TC-TRADING-047, TC-E2E-014

---

### US-TRADER-010: Real-Time Price Charts

**User Story:**
As a **trader**, I want to **view real-time price charts with technical indicators** so that **I can analyze market conditions and make informed trading decisions**.

**Acceptance Criteria:**
- [ ] Given I am viewing a trading symbol
- [ ] When I access the charts section
- [ ] Then I see a candlestick chart with real-time price updates
- [ ] And I can select timeframes: 1m, 5m, 15m, 1h, 4h, 1d
- [ ] And I can toggle technical indicators:
  - Moving averages (SMA20, SMA50, EMA9, EMA21)
  - RSI (14-period)
  - MACD with signal line and histogram
  - Bollinger Bands
  - Volume bars
- [ ] And indicators update with each new candle
- [ ] And I can zoom and pan the chart
- [ ] And I see support/resistance levels marked
- [ ] And I see my position entry price marked on the chart
- [ ] And I see my stop-loss and take-profit levels marked
- [ ] And charts are responsive and performant (60fps)

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-DASHBOARD-002, FR-MARKET-DATA-001
**Test Cases**: TC-DASHBOARD-010, TC-E2E-025

---

### US-TRADER-011: Stop-Loss Configuration

**User Story:**
As a **trader**, I want to **set and adjust stop-loss levels for my positions** so that **I can limit potential losses and protect my capital**.

**Acceptance Criteria:**
- [ ] Given I am opening a new position
- [ ] When I configure trade parameters
- [ ] Then I must set a stop-loss level (mandatory)
- [ ] And stop-loss can be 0.5% to 10% from entry price
- [ ] And I see default stop-loss based on risk level:
  - Conservative: 2%
  - Moderate: 3%
  - Aggressive: 5%
- [ ] And I can set stop-loss as:
  - Percentage from entry
  - Absolute price level
  - ATR-based (volatility adjusted)
- [ ] And I see estimated loss amount at stop-loss
- [ ] And stop-loss is validated before trade execution
- [ ] Given I have an open position
- [ ] When I modify the stop-loss
- [ ] Then the new stop-loss is updated immediately
- [ ] And I receive confirmation of the change

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-TRADING-004, FR-RISK-002
**Test Cases**: TC-TRADING-008, TC-TRADING-009, TC-E2E-017

---

### US-TRADER-012: Automatic Stop-Loss Execution

**User Story:**
As a **trader**, I want **stop-loss orders to execute automatically when price hits my threshold** so that **my losses are limited without manual intervention**.

**Acceptance Criteria:**
- [ ] Given I have an open LONG position with stop-loss at $49,000
- [ ] When the market price drops to $49,000 or below
- [ ] Then the system automatically creates a market SELL order
- [ ] And the order is marked as reduce_only (closing position)
- [ ] And the position is closed at current market price
- [ ] And I see the close reason as "StopLoss"
- [ ] And realized PnL is calculated and recorded
- [ ] And I receive a notification of stop-loss execution
- [ ] And the position is removed from my portfolio
- [ ] And margin is released for new trades
- [ ] And execution happens within 5 seconds of trigger

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-TRADING-004, FR-SYSTEM-004
**Test Cases**: TC-TRADING-010, TC-E2E-018

---

### US-TRADER-013: Take-Profit Configuration

**User Story:**
As a **trader**, I want to **set take-profit targets for my positions** so that **I can automatically lock in profits at my desired levels**.

**Acceptance Criteria:**
- [ ] Given I am opening a new position
- [ ] When I configure trade parameters
- [ ] Then I can optionally set a take-profit level
- [ ] And take-profit is typically 1.5x to 3x the stop-loss distance
- [ ] And I see default take-profit suggestions based on:
  - Risk-reward ratio (1.5:1, 2:1, 3:1)
  - AI signal suggestion
  - Resistance levels
- [ ] And I can set take-profit as:
  - Percentage from entry
  - Absolute price level
  - Multiple targets (25%, 50%, 100% position)
- [ ] And I see estimated profit amount at take-profit
- [ ] And I can enable trailing take-profit
- [ ] Given I have an open position
- [ ] When I modify the take-profit
- [ ] Then the new target is updated immediately

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-TRADING-005
**Test Cases**: TC-TRADING-011, TC-TRADING-012

---

### US-TRADER-014: Position Manual Closure

**User Story:**
As a **trader**, I want to **manually close positions at any time** so that **I can exit trades based on my judgment or changing market conditions**.

**Acceptance Criteria:**
- [ ] Given I have one or more open positions
- [ ] When I view a specific position
- [ ] And I click "Close Position" button
- [ ] Then I see a confirmation dialog showing:
  - Current price
  - Unrealized PnL
  - Estimated realized PnL (after fees)
- [ ] When I confirm closure
- [ ] Then a market order is created to close the position
- [ ] And the order executes immediately at current price
- [ ] And the close reason is "Manual"
- [ ] And realized PnL is calculated and displayed
- [ ] And I receive confirmation of successful closure
- [ ] And the position is removed from my portfolio
- [ ] And the trade record is updated in history

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-TRADING-018
**Test Cases**: TC-TRADING-045, TC-TRADING-046, TC-E2E-019

---

### US-TRADER-015: Risk Level Configuration

**User Story:**
As a **trader**, I want to **configure my personal risk tolerance** so that **the system adjusts trading parameters to match my risk appetite**.

**Acceptance Criteria:**
- [ ] Given I am in account settings
- [ ] When I access risk management section
- [ ] Then I can select a risk level:
  - Conservative (low risk, lower returns)
  - Moderate (balanced risk/reward)
  - Aggressive (high risk, higher potential returns)
- [ ] And I see how each level affects:
  - Default stop-loss percentage (2%, 3%, 5%)
  - Maximum leverage used
  - Position size limits
  - Maximum daily loss limit
- [ ] When I change risk level
- [ ] Then the system updates default parameters
- [ ] And existing positions are not affected
- [ ] And new trades use updated parameters
- [ ] And I receive confirmation of the change

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-RISK-003, FR-TRADING-010
**Test Cases**: TC-RISK-010, TC-E2E-021

---

### US-TRADER-016: Trading Strategy Selection

**User Story:**
As a **trader**, I want to **select which trading strategies to use** so that **I can customize my trading approach based on my preferences and market conditions**.

**Acceptance Criteria:**
- [ ] Given I am in trading settings
- [ ] When I view available strategies
- [ ] Then I see a list of strategies:
  - RSI Strategy (oversold/overbought)
  - MACD Strategy (trend following)
  - Bollinger Bands Strategy (volatility)
  - Volume Strategy (volume confirmation)
  - Moving Average Strategy (trend)
- [ ] And I see a description of each strategy
- [ ] And I see historical performance metrics for each
- [ ] And I can enable/disable individual strategies
- [ ] And I can adjust strategy parameters (e.g., RSI period)
- [ ] When I save strategy selection
- [ ] Then AI analysis uses only selected strategies
- [ ] And strategy scores reflect my selection
- [ ] And I can revert to default settings

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-STRATEGIES-001, FR-AI-006
**Test Cases**: TC-STRATEGIES-001, TC-E2E-022

---

### US-TRADER-017: WebSocket Real-Time Updates

**User Story:**
As a **trader**, I want to **receive real-time updates for prices and positions** so that **I can react quickly to market changes without refreshing the page**.

**Acceptance Criteria:**
- [ ] Given I am logged in and viewing the dashboard
- [ ] When the WebSocket connection is established
- [ ] Then I receive real-time updates for:
  - Price changes (every 1 second)
  - Position PnL updates (every 5 seconds)
  - Trade executions (immediate)
  - AI signal generation (immediate)
  - Stop-loss/take-profit triggers (immediate)
- [ ] And updates are reflected in the UI instantly
- [ ] And WebSocket reconnects automatically on disconnect
- [ ] And I see connection status indicator
- [ ] And no page refresh is required for updates
- [ ] And updates continue while I navigate between pages

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-WEBSOCKET-001, FR-WEBSOCKET-002
**Test Cases**: TC-WEBSOCKET-001, TC-E2E-030

---

### US-TRADER-018: Notification Preferences

**User Story:**
As a **trader**, I want to **configure my notification preferences** so that **I receive alerts for important events without being overwhelmed**.

**Acceptance Criteria:**
- [ ] Given I am in account settings
- [ ] When I access notification preferences
- [ ] Then I can toggle notifications for:
  - Trade executions (on/off)
  - Stop-loss triggers (on/off)
  - Take-profit triggers (on/off)
  - High-confidence AI signals (on/off)
  - Risk warnings (on/off, mandatory for certain critical alerts)
  - System alerts (on/off)
- [ ] And I can choose notification channels:
  - In-app notifications
  - Email alerts
  - Push notifications (future)
- [ ] And I can set quiet hours (no notifications)
- [ ] When I save preferences
- [ ] Then notifications are delivered according to settings
- [ ] And critical alerts override quiet hours

**Priority**: ☑ Medium
**Status**: ☑ Implemented
**Complexity**: Low
**Related FR**: FR-AUTH-012 (UserSettings)
**Test Cases**: TC-AUTH-035, TC-E2E-040

---

### US-TRADER-019: Leverage Adjustment

**User Story:**
As a **trader**, I want to **adjust leverage for futures trading** so that **I can control my position size and risk exposure**.

**Acceptance Criteria:**
- [ ] Given I am configuring a trade or in settings
- [ ] When I adjust leverage slider/input
- [ ] Then I see available leverage range based on symbol:
  - BTC/USDT: up to 125x (production), 20x (testnet)
  - ETH/USDT: up to 100x (production), 20x (testnet)
  - Altcoins: up to 50x (production), 20x (testnet)
- [ ] And I see how leverage affects:
  - Initial margin required
  - Liquidation price
  - Maximum position size
  - Potential profit/loss
- [ ] And I see warnings for high leverage (>10x)
- [ ] When I confirm leverage
- [ ] Then Binance leverage is updated for the symbol
- [ ] And new trades use the updated leverage
- [ ] And I cannot change leverage while position is open

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-TRADING-006
**Test Cases**: TC-TRADING-013, TC-TRADING-014, TC-E2E-023

---

### US-TRADER-020: Account Balance Overview

**User Story:**
As a **trader**, I want to **view my complete account balance breakdown** so that **I understand my capital allocation and available funds**.

**Acceptance Criteria:**
- [ ] Given I am on the dashboard or account page
- [ ] When I view balance information
- [ ] Then I see:
  - Total wallet balance (all funds)
  - Available balance (free for new trades)
  - Total margin used (locked in positions)
  - Unrealized PnL (across all positions)
  - Total equity (balance + unrealized PnL)
  - Free margin (equity - margin used)
  - Margin level percentage (equity/margin * 100)
- [ ] And I see breakdown by:
  - Cash (USDT balance)
  - Open positions value
  - Pending orders margin
- [ ] And I see warnings when:
  - Margin level < 200% (warning)
  - Margin level < 150% (danger, no new positions)
- [ ] And values update in real-time

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-TRADING-009, FR-PORTFOLIO-001
**Test Cases**: TC-TRADING-021, TC-E2E-011

---

### US-TRADER-021: Multi-Timeframe Analysis

**User Story:**
As a **trader**, I want to **see analysis across multiple timeframes** so that **I can make decisions based on short-term and long-term trends**.

**Acceptance Criteria:**
- [ ] Given I am viewing an AI trading signal
- [ ] When I review the analysis
- [ ] Then I see indicators for multiple timeframes:
  - 1-hour timeframe (short-term)
  - 4-hour timeframe (medium-term)
  - (Future: 1-day timeframe for long-term)
- [ ] And each timeframe shows:
  - RSI value
  - MACD line, signal line, histogram
  - Moving averages (SMA, EMA)
  - Bollinger Bands position
  - Volume analysis
  - ATR (volatility)
- [ ] And I see trend alignment across timeframes
- [ ] And I see divergences between timeframes
- [ ] And the AI signal considers all timeframes
- [ ] And confidence is higher when timeframes align

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: High
**Related FR**: FR-AI-006, FR-MARKET-DATA-002
**Test Cases**: TC-AI-020, TC-E2E-026

---

### US-TRADER-022: Paper Portfolio Reset

**User Story:**
As a **trader**, I want to **reset my paper trading portfolio** so that **I can start fresh with new strategies or after poor performance**.

**Acceptance Criteria:**
- [ ] Given I am in paper trading mode
- [ ] When I click "Reset Portfolio" button
- [ ] Then I see a confirmation dialog warning:
  - All open positions will be closed
  - All trade history will be cleared (or archived)
  - Portfolio will reset to $10,000 starting balance
  - Performance metrics will reset to zero
- [ ] When I confirm reset
- [ ] Then all paper positions are closed at market price
- [ ] And final PnL is calculated for each position
- [ ] And portfolio is reset to initial balance
- [ ] And performance metrics are cleared
- [ ] And I receive confirmation of successful reset
- [ ] And I can optionally archive old trades for reference

**Priority**: ☑ Medium
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-PAPER-TRADING-005
**Test Cases**: TC-PAPER-010, TC-E2E-031

---

### US-TRADER-023: Symbol Search and Filtering

**User Story:**
As a **trader**, I want to **search and filter available trading symbols** so that **I can quickly find assets I want to trade**.

**Acceptance Criteria:**
- [ ] Given I am on the trading dashboard
- [ ] When I use the symbol search bar
- [ ] Then I can search by:
  - Symbol (e.g., "BTC", "ETH")
  - Full name (e.g., "Bitcoin")
  - Category (e.g., "DeFi", "Layer 1")
- [ ] And I see matching symbols as I type
- [ ] And I can filter symbols by:
  - Market cap (large, mid, small)
  - Volume (high, medium, low)
  - Volatility (high, medium, low)
  - Price change 24h (gainers, losers)
- [ ] And I can sort symbols by:
  - Price
  - 24h volume
  - 24h change percentage
  - Market cap
- [ ] And I see key metrics for each symbol:
  - Current price
  - 24h change
  - 24h volume
  - AI signal (if available)

**Priority**: ☑ Medium
**Status**: ☐ Planned
**Complexity**: Medium
**Related FR**: FR-MARKET-DATA-003, FR-DASHBOARD-001
**Test Cases**: TC-DASHBOARD-015, TC-E2E-035

---

### US-TRADER-024: Position Size Calculator

**User Story:**
As a **trader**, I want to **see recommended position sizes based on risk** so that **I can size my trades appropriately for my account**.

**Acceptance Criteria:**
- [ ] Given I am configuring a new trade
- [ ] When I view position sizing
- [ ] Then I see calculated position sizes based on:
  - Account balance
  - Risk percentage (default 2%)
  - Stop-loss distance
  - Leverage
- [ ] And I see the calculator show:
  - Recommended quantity
  - Position value (notional)
  - Margin required
  - Maximum loss at stop-loss
  - Percentage of account at risk
- [ ] And I can adjust risk percentage (0.5% to 10%)
- [ ] And the calculator updates in real-time
- [ ] And I see warnings if position size is too large
- [ ] And I can use the calculated size or enter custom

**Priority**: ☑ High
**Status**: ☐ In Progress
**Complexity**: Medium
**Related FR**: FR-TRADING-014, FR-RISK-004
**Test Cases**: TC-TRADING-034, TC-RISK-015

---

### US-TRADER-025: Trade Journal Notes

**User Story:**
As a **trader**, I want to **add notes to my trades** so that **I can document my reasoning and learn from past decisions**.

**Acceptance Criteria:**
- [ ] Given I have executed or plan to execute a trade
- [ ] When I view the trade details
- [ ] Then I can add a note/comment
- [ ] And notes can include:
  - Entry reasoning
  - Strategy rationale
  - Market conditions observed
  - Emotional state
  - Exit reasoning (for closed trades)
- [ ] And I can edit notes after creation
- [ ] And notes are saved with timestamp
- [ ] And I can view notes in trade history
- [ ] And I can search trades by note content
- [ ] And notes are private (only visible to me)

**Priority**: ☑ Medium
**Status**: ☐ Planned
**Complexity**: Low
**Related FR**: FR-PORTFOLIO-004
**Test Cases**: TC-PORTFOLIO-020, TC-E2E-045

---

### US-TRADER-026: Daily Loss Limit Protection

**User Story:**
As a **trader**, I want **automatic trading suspension when I hit daily loss limits** so that **I am protected from catastrophic losses during bad trading days**.

**Acceptance Criteria:**
- [ ] Given I have configured a daily loss limit (default 5% of account)
- [ ] When my realized losses for the day reach the limit
- [ ] Then all new trades are blocked automatically
- [ ] And I receive a notification of suspension
- [ ] And I see a warning message on the dashboard
- [ ] And I can view current loss amount vs limit
- [ ] And open positions remain active with stop-loss protection
- [ ] And trading automatically resumes the next day (UTC midnight)
- [ ] And I can manually override (with confirmation) if needed
- [ ] And loss limit resets at start of each trading day

**Priority**: ☑ Critical
**Status**: ☑ Implemented
**Complexity**: Medium
**Related FR**: FR-RISK-005, FR-TRADING-010
**Test Cases**: TC-RISK-008, TC-E2E-027

---

### US-TRADER-027: AI Signal Confidence Filter

**User Story:**
As a **trader**, I want to **filter AI signals by confidence threshold** so that **I only see high-quality trading opportunities**.

**Acceptance Criteria:**
- [ ] Given AI is generating trading signals
- [ ] When I configure signal settings
- [ ] Then I can set minimum confidence threshold (0.45 to 0.90)
- [ ] And default threshold is 0.70 (high confidence)
- [ ] And I see a slider or input to adjust threshold
- [ ] And I see expected signal frequency for each threshold:
  - 0.45: More signals, lower quality
  - 0.70: Balanced signals, good quality
  - 0.85: Fewer signals, highest quality
- [ ] When I set a threshold
- [ ] Then only signals meeting the threshold are shown
- [ ] And lower confidence signals are hidden or grayed out
- [ ] And I can temporarily view all signals
- [ ] And automated trading only executes above threshold

**Priority**: ☑ High
**Status**: ☑ Implemented
**Complexity**: Low
**Related FR**: FR-AI-006, FR-TRADING-010
**Test Cases**: TC-AI-025, TC-TRADING-025

---

### US-TRADER-028: Export Trading Data

**User Story:**
As a **trader**, I want to **export my trading data and performance metrics** so that **I can analyze it in external tools or for tax reporting**.

**Acceptance Criteria:**
- [ ] Given I have trading history and performance data
- [ ] When I navigate to export section
- [ ] Then I can export:
  - Trade history (all closed trades)
  - Open positions (current portfolio)
  - Performance metrics
  - Account balance history
- [ ] And I can choose export format:
  - CSV (for Excel)
  - JSON (for APIs)
  - PDF (for reports)
- [ ] And I can filter export by:
  - Date range
  - Symbol
  - Trade type (live/paper)
- [ ] And exported data includes all relevant fields
- [ ] And export completes within 30 seconds
- [ ] And I can download or email the export

**Priority**: ☑ Medium
**Status**: ☐ Planned
**Complexity**: Medium
**Related FR**: FR-PORTFOLIO-005
**Test Cases**: TC-PORTFOLIO-025, TC-E2E-050

---

## Use Cases

### UC-TRADER-001: Complete Trading Workflow

**Actor**: Intermediate Trader
**Preconditions**:
- User has registered account
- User has completed paper trading
- Live trading is enabled
- Account has $5,000 balance

**Main Flow**:
1. User logs into the platform
2. User views dashboard with real-time market data
3. AI service generates LONG signal for BTCUSDT (confidence 0.78)
4. User reviews AI signal:
   - Confidence: 0.78 (High)
   - Reasoning: "RSI at 32 (oversold), bullish MACD crossover"
   - 1H indicators: Bullish alignment
   - 4H indicators: Neutral
   - Stop-loss suggestion: $49,200 (2%)
   - Take-profit suggestion: $52,000 (4%)
5. User decides to accept the signal
6. System calculates position size: 0.05 BTC (5% risk, 2% stop-loss)
7. User confirms trade execution
8. System validates:
   - Risk checks pass (3/10 positions, confidence > 0.70)
   - Sufficient margin available
   - Stop-loss within limits (2% < 10%)
9. System executes market BUY order on Binance
10. Order fills at $50,000
11. Position appears in portfolio with:
    - Quantity: 0.05 BTC
    - Entry: $50,000
    - Stop-loss: $49,000
    - Take-profit: $52,000
12. User monitors position via real-time WebSocket updates
13. Price rises to $52,100
14. System triggers take-profit execution
15. Position closes automatically at $52,100
16. User sees notification: "Position closed - Take Profit"
17. Realized PnL: +$105 (2.1% gain after fees)
18. Trade recorded in history
19. Portfolio equity updated

**Alternative Flows**:
- **Alt 1 - Low Confidence Signal**: User sets minimum confidence to 0.80, signal is filtered out
- **Alt 2 - Stop-Loss Trigger**: Price drops to $49,000, position closes with -$50 loss
- **Alt 3 - Manual Override**: User manually closes position early at $51,000

**Postconditions**:
- Trade completed and recorded
- Portfolio updated with realized PnL
- Margin released for new trades
- User can review trade in history

---

### UC-TRADER-002: Paper Trading to Live Transition

**Actor**: Novice Trader
**Preconditions**:
- User registered 2 weeks ago
- User has practiced with paper trading
- Paper portfolio at $12,500 (25% gain)

**Main Flow**:
1. User reviews paper trading performance:
   - 20 total trades
   - 65% win rate
   - $2,500 profit
   - Max drawdown: 8%
2. User feels confident to trade live
3. User navigates to Settings > Trading
4. User toggles "Enable Live Trading"
5. System shows warning dialog:
   - "Live trading involves real money and risk"
   - "You can lose all your capital"
   - "Ensure Binance API keys are correct"
6. User acknowledges risks
7. User enters Binance API keys
8. System validates API keys with test request
9. System enables live trading
10. User receives confirmation email
11. User starts with conservative settings:
    - Risk level: Conservative
    - Minimum confidence: 0.75
    - Maximum positions: 3
    - Daily loss limit: 3%
12. User executes first live trade with small size
13. User monitors closely and adjusts settings based on results

**Postconditions**:
- Live trading enabled
- User aware of risks
- Conservative settings applied
- Ready for real trading

---

## Traceability

**Functional Requirements Coverage**:
- FR-AUTH: US-TRADER-001, US-TRADER-002
- FR-TRADING: US-TRADER-004, US-TRADER-006, US-TRADER-011, US-TRADER-012, US-TRADER-014, US-TRADER-019
- FR-AI: US-TRADER-005, US-TRADER-021, US-TRADER-027
- FR-PAPER-TRADING: US-TRADER-003, US-TRADER-022
- FR-PORTFOLIO: US-TRADER-007, US-TRADER-008, US-TRADER-009, US-TRADER-020
- FR-RISK: US-TRADER-015, US-TRADER-026
- FR-STRATEGIES: US-TRADER-016
- FR-WEBSOCKET: US-TRADER-017
- FR-DASHBOARD: US-TRADER-010, US-TRADER-023

**Test Cases**:
- User stories map to 100+ test cases across unit, integration, and E2E tests
- Each acceptance criterion should have corresponding test case(s)

**Business Rules**:
- BUSINESS_RULES.md#MaximumPositions -> US-TRADER-007
- BUSINESS_RULES.md#StopLossRequirements -> US-TRADER-011, US-TRADER-012
- BUSINESS_RULES.md#DailyLossLimit -> US-TRADER-026
- BUSINESS_RULES.md#MinimumConfidence -> US-TRADER-027

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Users lose money in live trading | High | Medium | Mandatory paper trading, risk warnings, stop-loss enforcement, daily loss limits |
| Users overwhelmed by complexity | Medium | High | Progressive disclosure, tutorials, conservative defaults, in-app guidance |
| Users ignore AI signals | Low | Medium | Show signal accuracy metrics, education on signal interpretation |
| Users set excessive leverage | High | Medium | Leverage warnings, maximum limits, risk education |
| Users trade too frequently | Medium | Medium | Daily loss limits, cooldown periods after losses |

---

## Open Questions

- [ ] Should we implement a points/gamification system for paper trading? **Resolution needed by**: 2025-11-15
- [ ] What is the minimum paper trading period before allowing live trading? **Resolution needed by**: 2025-11-01
- [ ] Should we add social trading features (copy trading)? **Resolution needed by**: 2025-12-01
- [ ] How to handle users who consistently ignore risk warnings? **Resolution needed by**: 2025-11-01

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Product Team | Initial comprehensive trader user stories document |

---

## Appendix

**References**:
- [FR-AUTH](../1.1-functional-requirements/FR-AUTH.md) - Authentication specifications
- [FR-TRADING](../1.1-functional-requirements/FR-TRADING.md) - Trading engine specifications
- [FR-AI](../1.1-functional-requirements/FR-AI.md) - AI service specifications
- [BUSINESS_RULES.md](../../BUSINESS_RULES.md) - Business rules and constraints

**Glossary**:
- **Paper Trading**: Simulated trading with virtual money
- **Stop-Loss**: Automatic order to limit losses
- **Take-Profit**: Automatic order to lock in profits
- **Confidence**: AI signal confidence score (0.0-1.0)
- **Leverage**: Position size multiplier using borrowed funds
- **Margin**: Collateral required for leveraged positions
- **PnL**: Profit and Loss
- **Unrealized PnL**: Profit/loss on open positions
- **Realized PnL**: Actual profit/loss from closed positions

---

**Remember**: User stories evolve based on feedback. Update this document as new trader needs emerge and existing stories are refined.
