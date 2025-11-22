# Risk Management - Functional Requirements

**Spec ID**: FR-RISK-001
**Version**: 2.0
**Status**: ☐ Draft
**Owner**: Risk Management Team
**Last Updated**: 2025-11-22

---

## Tasks Checklist

- [ ] Requirements gathered
- [ ] Design completed
- [ ] Implementation done
- [ ] Tests written
- [ ] Documentation updated
- [ ] Code reviewed
- [ ] Deployed to staging
- [ ] Deployed to production

---

## Metadata

**Related Specs**:
- Related FR: [FR-PORTFOLIO-001](./FR-PORTFOLIO.md)
- Related Design: [DATA_MODELS.md](../../DATA_MODELS.md)
- Related API: [API_SPEC.md](../../API_SPEC.md)
- Related Business Rules: [BUSINESS_RULES.md](../../BUSINESS_RULES.md)

**Dependencies**:
- Depends on: [FR-PORTFOLIO-001] - Portfolio management for exposure calculations
- Blocks: [FR-TRADING-001] - Trading execution requires risk approval

**Business Value**: Critical
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

The Risk Management system provides comprehensive risk controls to protect trader capital and enforce regulatory compliance. It validates all trades against position limits, stop-loss requirements, daily loss limits, leverage restrictions, and performs real-time risk monitoring. This system is the first line of defense against catastrophic losses and ensures sustainable trading practices.

Risk management is critical for both paper trading (to simulate realistic constraints) and live trading (to prevent actual capital loss).

### 1.5 Trailing Stop Loss

Advanced profit protection mechanism that allows trades to capture extended moves while protecting against reversals. Particularly valuable in trending markets, increasing profit capture by 20-30% compared to fixed take-profit levels. The trailing stop automatically adjusts as the market moves in favor of the position, locking in profits while allowing the position to benefit from continued favorable price movement.

---

## Business Context

**Problem Statement**:
Without proper risk management, traders can quickly lose substantial capital through over-leveraging, lack of stop-losses, concentration risk, and emotional decision-making. Cryptocurrency markets are highly volatile (daily swings of 10%+ are common), making risk controls essential. Exchanges and regulators also impose risk limits that must be enforced. Additionally, traders often close profitable positions too early or let them reverse completely, missing optimal profit opportunities.

**Business Goals**:
- Prevent any single trade from risking more than 2% of account balance
- Ensure no trader can lose more than 5% of their account in a single day
- Limit maximum concurrent positions to prevent over-diversification and concentration risk
- Enforce stop-loss requirements on all positions to cap downside risk
- Maintain leverage within safe and regulatory limits
- Provide real-time risk alerts before violations occur
- Maximize profit capture in trending markets through intelligent trailing stops (20-30% improvement)
- Protect accumulated profits from market reversals

**Success Metrics**:
- Risk check latency: < 50ms per trade validation (Target: 25ms)
- False positive rate: < 1% (trades incorrectly rejected)
- Risk limit breach rate: 0% (all limits enforced with zero failures)
- Maximum daily loss across all users: < 5% of account balance
- Risk alert response time: < 1 second from breach detection
- User adherence to risk limits: > 95%
- Profit capture improvement with trailing stops: 20-30% vs fixed take-profit
- Trailing stop activation rate: > 40% of profitable trades

---

## Functional Requirements

### FR-RISK-001: Position Risk Limits

**Priority**: ☑ Critical
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-001`

**Description**:
The system shall enforce position-level risk limits including maximum concurrent positions, maximum position size per trade, and concentration limits per asset. These limits prevent over-diversification, excessive exposure to single assets, and help maintain manageable portfolio complexity.

**Acceptance Criteria**:
- [ ] Enforce maximum 10 concurrent open positions per user (configurable 1-100)
- [ ] Reject new positions when max positions limit reached
- [ ] Calculate current position count across all symbols
- [ ] Enforce maximum position size: 10% of account balance per trade
- [ ] Enforce minimum position size: 0.1% of account balance or $10 USDT
- [ ] Calculate position size percentage: (Position Value / Account Balance) × 100
- [ ] Enforce maximum positions per symbol (default: 1, configurable)
- [ ] Validate position size before trade execution
- [ ] Support different limits for different user tiers (Basic, Pro, VIP)
- [ ] Track exposure by symbol and prevent concentration risk
- [ ] Enforce maximum exposure per symbol: 20% of portfolio value
- [ ] Support temporary limit overrides with admin approval
- [ ] Log all limit checks with pass/fail result
- [ ] Provide clear rejection reasons when limits exceeded
- [ ] Calculate aggregate position risk across correlated assets

**Dependencies**: Position manager, portfolio system
**Test Cases**: [TC-RISK-001, TC-RISK-002, TC-RISK-003]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 1.1: Maximum Positions (10 concurrent)
- BUSINESS_RULES.md Section 1.2: Position Size Limits (0.1% - 10%)

**Implementation Notes**:
- Code Location: `rust-core-engine/src/trading/risk_manager.rs:63-79`
- Method: `get_max_positions()`
- Config: `rust-core-engine/src/config.rs:41` - max_positions field
- Settings: `rust-core-engine/src/paper_trading/settings.rs:36` - max_positions
- Validation occurs in pre-trade risk check before order submission

---

### FR-RISK-002: Stop-Loss Requirements

**Priority**: ☑ Critical
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-002`

**Description**:
The system shall enforce mandatory stop-loss requirements on all positions to limit maximum loss per trade. Stop-loss orders must be placed at the time of position opening and cannot be removed (only moved closer to entry or tighter). Maximum stop-loss distance is 10% from entry price.

**Acceptance Criteria**:
- [ ] Require stop-loss on all new positions (mandatory, no exceptions)
- [ ] Allow manual override flag for experienced traders (logged and monitored)
- [ ] Set default stop-loss based on risk profile: Conservative 2%, Medium 3%, Aggressive 5%
- [ ] Enforce maximum stop-loss distance: 10% from entry price
- [ ] Calculate stop-loss distance: |Entry Price - Stop Loss| / Entry Price × 100
- [ ] Reject trades with stop-loss > 10% away from entry
- [ ] Validate stop-loss is on correct side (below entry for LONG, above for SHORT)
- [ ] Allow stop-loss modification only to tighter levels (closer to entry)
- [ ] Prevent stop-loss removal once position is open
- [ ] Support trailing stop-loss with configurable trail distance (see FR-RISK-007, FR-RISK-008)
- [ ] Activate trailing stop after minimum profit achieved (default: 1.5x risk)
- [ ] Calculate risk amount: Position Size × |Entry - Stop Loss|
- [ ] Enforce maximum risk per trade: 2% of account balance
- [ ] Provide stop-loss recommendations based on volatility (ATR-based)
- [ ] Auto-set stop-loss if trader forgets (use default percentage)
- [ ] Monitor stop-loss execution and alert if not filled at expected price

**Dependencies**: Trading engine, position manager
**Test Cases**: [TC-RISK-010, TC-RISK-011, TC-RISK-012]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 2.1: Stop Loss Requirements (mandatory, max 10%)
- BUSINESS_RULES.md Section 5.1: Position Sizing (risk-based)

**Implementation Notes**:
- Stop-loss validation in `can_open_position()` method
- Stop-loss stored in Position struct: `stop_loss: Option<f64>`
- Automatic stop-loss execution in portfolio's `check_automatic_closures()` method
- Code Location: `rust-core-engine/src/paper_trading/portfolio.rs:300-337`
- Default stop-loss percentages in settings: `rust-core-engine/src/paper_trading/settings.rs:70-71`

---

### FR-RISK-003: Daily Loss Limits

**Priority**: ☑ Critical
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-003`

**Description**:
The system shall enforce a daily loss limit to prevent catastrophic losses from a series of bad trades or unfavorable market conditions. When the limit is reached, all new trading is halted for 24 hours, and the user is notified. This is a circuit breaker to force traders to step away and reassess.

**Acceptance Criteria**:
- [ ] Calculate daily loss: Sum of all realized PnL in rolling 24-hour period
- [ ] Set default daily loss limit: 5% of account balance
- [ ] Allow user configuration of daily loss limit (range: 2% - 10%)
- [ ] Calculate loss percentage: (Daily Loss / Account Balance) × 100
- [ ] Halt all new trading when daily loss limit exceeded
- [ ] Allow closing existing positions even after limit reached
- [ ] Prevent opening any new positions for 24 hours after breach
- [ ] Send immediate notification to user when limit reached
- [ ] Display prominent warning banner on dashboard
- [ ] Log daily loss limit events for compliance
- [ ] Reset daily loss calculation at 00:00 UTC each day
- [ ] Support manual reset by admin in exceptional circumstances
- [ ] Calculate daily loss excluding unrealized PnL (closed trades only)
- [ ] Track separate daily loss by strategy if multiple strategies active
- [ ] Provide daily loss limit status in API responses
- [ ] Generate daily loss report at end of each trading day
- [ ] Escalate to admin if daily loss > 7% (breach of safety margin)

**Dependencies**: Trade history, portfolio system, notification system
**Test Cases**: [TC-RISK-020, TC-RISK-021, TC-RISK-022]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 2.2: Daily Loss Limit (5% of balance)
- BUSINESS_RULES.md Section 2.3: Drawdown Protection (15% max)

**Implementation Notes**:
- Daily loss calculation in portfolio metrics
- Check in `can_open_position()` before approving new trades
- Settings: `rust-core-engine/src/paper_trading/settings.rs:86` - daily_loss_limit_pct
- Reset logic to be implemented with scheduled job (daily at 00:00 UTC)
- Store last breach timestamp to enforce 24-hour lockout

---

### FR-RISK-004: Leverage Limits

**Priority**: ☑ Critical
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-004`

**Description**:
The system shall enforce leverage limits to control position risk and margin usage. Leverage amplifies both gains and losses, making it a critical risk parameter. Limits vary by trading pair based on liquidity and volatility. Testnet has stricter limits (max 20x) to encourage responsible testing.

**Acceptance Criteria**:
- [ ] Enforce maximum leverage by trading pair:
  - BTC/USDT: 125x (production), 20x (testnet)
  - ETH/USDT: 100x (production), 20x (testnet)
  - Major pairs: 75x (production), 20x (testnet)
  - Altcoins: 50x (production), 20x (testnet)
- [ ] Set default leverage: 10x (conservative, recommended for beginners)
- [ ] Validate leverage is between 1 and maximum for pair
- [ ] Calculate effective leverage: Position Size / Margin Used
- [ ] Enforce leverage in testnet mode: max 20x for all pairs
- [ ] Prevent leverage changes on open positions (close and reopen required)
- [ ] Calculate margin requirement: (Position Size × Entry Price) / Leverage
- [ ] Warn user when using leverage > 20x (high risk warning)
- [ ] Apply leverage limits based on account type (Basic, Pro, VIP)
- [ ] Enforce regulatory leverage limits by jurisdiction (e.g., US: max 50x)
- [ ] Support isolated margin (leverage per position) and cross margin (shared)
- [ ] Calculate liquidation price based on leverage and maintenance margin
- [ ] Provide leverage recommendations based on volatility
- [ ] Track average leverage used across all positions
- [ ] Alert when portfolio-wide leverage exceeds safe threshold (3x average)
- [ ] Reduce maximum leverage during extreme market volatility

**Dependencies**: Exchange API (Binance Futures), configuration system
**Test Cases**: [TC-RISK-030, TC-RISK-031, TC-RISK-032]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 1.3: Leverage Limits (max 20x testnet)
- BUSINESS_RULES.md Section 2.4: Margin Requirements (min 150% level)

**Implementation Notes**:
- Code Location: `rust-core-engine/src/config.rs:48` - leverage field
- Settings: `rust-core-engine/src/paper_trading/settings.rs:43, 77` - default_leverage, max_leverage
- Leverage validation in trade execution pipeline
- Testnet override: Check BINANCE_TESTNET env var, apply 20x cap
- Liquidation price calculation in PaperTrade struct

---

### FR-RISK-005: Pre-Trade Risk Checks

**Priority**: ☑ Critical
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-005`

**Description**:
The system shall perform comprehensive pre-trade risk validation before every trade execution. This is a multi-step validation process that checks all risk parameters and rejects trades that violate any risk rule. Pre-trade checks are the primary mechanism for risk enforcement.

**Acceptance Criteria**:
- [ ] Validate trading is enabled (not disabled by admin or user)
- [ ] Check maximum concurrent positions limit
- [ ] Validate position size within allowed range (0.1% - 10% of balance)
- [ ] Verify stop-loss is set and within maximum distance (10%)
- [ ] Check daily loss limit not exceeded
- [ ] Validate leverage is within limits for trading pair
- [ ] Verify sufficient free margin for new position
- [ ] Check margin level will remain above minimum (150%) after trade
- [ ] Validate signal confidence meets minimum threshold (0.7 for StrongBuy/Sell, 0.8 for Buy/Sell)
- [ ] Check risk-reward ratio is acceptable (minimum 1.5:1)
- [ ] Verify no trading during maintenance windows
- [ ] Validate symbol is enabled for trading
- [ ] Check correlation risk (max positions in correlated assets)
- [ ] Verify position doesn't exceed symbol-specific limits
- [ ] Validate order parameters (price, quantity, type)
- [ ] Check account status (active, not suspended)
- [ ] Calculate risk score: 0-100 based on multiple risk factors
- [ ] Reject trade if risk score > 80 (high risk)
- [ ] Return detailed rejection reason if any check fails
- [ ] Log all pre-trade checks with timestamp and result
- [ ] Performance: Complete all checks in < 50ms

**Dependencies**: Position manager, portfolio system, market data
**Test Cases**: [TC-RISK-040, TC-RISK-041, TC-RISK-042, TC-RISK-043, TC-RISK-044]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 3: AI Signal Rules (confidence thresholds)
- BUSINESS_RULES.md Section 4: Order Execution Rules
- All risk management rules (Sections 1-2)

**Implementation Notes**:
- Code Location: `rust-core-engine/src/trading/risk_manager.rs:17-61`
- Method: `can_open_position()`
- Checks trading enabled: `self.config.enabled`
- Checks signal confidence: Different thresholds for signal types
- Checks risk-reward ratio: Minimum 1.5:1
- Returns `Result<bool>` - Ok(true) if passed, Ok(false) if failed, Err if error
- Integration point: Called before every trade execution

---

### FR-RISK-006: Real-Time Risk Monitoring

**Priority**: ☑ High
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-006`

**Description**:
The system shall continuously monitor risk metrics in real-time and generate alerts when thresholds are approached or breached. This proactive monitoring helps traders and admins respond to risk situations before they become critical. Monitoring includes portfolio-level, position-level, and market-level risk indicators.

**Acceptance Criteria**:
- [ ] Monitor margin level continuously (every price update)
- [ ] Alert when margin level < 200% (warning level)
- [ ] Critical alert when margin level < 150% (liquidation risk)
- [ ] Auto-close most losing position if margin level < 110%
- [ ] Monitor daily loss approaching limit (alert at 80% of limit)
- [ ] Track current drawdown from peak equity
- [ ] Alert when drawdown > 10% (warning)
- [ ] Critical alert when drawdown > 15% (max drawdown)
- [ ] Monitor open position count approaching limit
- [ ] Alert when 80% of max positions used
- [ ] Track correlation risk across open positions
- [ ] Alert when correlated positions exceed threshold
- [ ] Monitor individual position risk (current loss vs. stop-loss)
- [ ] Alert when position reaches 50% of stop-loss distance
- [ ] Track leverage utilization across portfolio
- [ ] Alert when average leverage > 10x
- [ ] Monitor market volatility and adjust risk thresholds
- [ ] Calculate Value at Risk (VaR) for portfolio
- [ ] Alert when VaR exceeds acceptable threshold
- [ ] Generate risk score for entire portfolio (0-100)
- [ ] Update risk metrics every second (real-time)
- [ ] Provide risk dashboard with real-time indicators
- [ ] Send push notifications for critical risk events
- [ ] Escalate to admin if risk cannot be mitigated automatically

**Dependencies**: Portfolio system, notification system, market data
**Test Cases**: [TC-RISK-050, TC-RISK-051, TC-RISK-052]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 2.4: Margin Requirements (200% warning, 150% critical)
- BUSINESS_RULES.md Section 2.3: Drawdown Protection (15% max)
- BUSINESS_RULES.md Section 10.2: Alerting Thresholds

**Implementation Notes**:
- Monitoring logic in portfolio's `update_portfolio_values()` method
- Margin level calculation: `(equity / margin_used) * 100.0`
- Automatic closure in `check_automatic_closures()` method
- WebSocket push for real-time alerts to dashboard
- Risk score calculation to be implemented in risk_manager module

---

### FR-RISK-007: Trailing Stop Loss (Long Positions)

**Priority**: ☑ Critical
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-007`

**Description**:
The system shall implement trailing stop loss functionality for LONG positions to automatically lock in profits as the market moves favorably. The trailing stop moves up with the price, maintaining a fixed percentage distance below the highest price achieved since activation. This allows unlimited upside potential while protecting against reversals.

**Purpose**:
Automatically move stop loss up as price moves favorably, locking in profits while allowing upside to run. Critical for maximizing profit capture in trending markets.

**How It Works**:
1. Trade enters profit (current price > entry price)
2. When profit reaches activation threshold (e.g., +2%), trailing starts
3. Stop loss moves up, maintaining fixed percentage below highest price
4. If price drops by trailing percentage (e.g., 1.5%), exit with profit
5. Allows unlimited upside while protecting downside

**Settings**:
```rust
pub trailing_stop_enabled: bool,           // Enable/disable feature
pub trailing_stop_pct: f64,                // Trail distance (1.0-5.0%, default: 1.5%)
pub trailing_activation_pct: f64,          // Min profit to activate (0.5-5.0%, default: 2.0%)
```

**Example Scenario (LONG BTCUSDT)**:
```
Entry: $50,000
Activation threshold: +2% = $51,000
Trailing distance: 1.5%

Price movement:
$50,000 → Entry, no trailing yet
$51,000 → +2% profit, TRAILING ACTIVATES
        → Stop loss set at: $51,000 - 1.5% = $50,235
$52,000 → New high!
        → Stop loss moves to: $52,000 - 1.5% = $51,220
$53,000 → New high!
        → Stop loss moves to: $53,000 - 1.5% = $52,205
$52,500 → Price dips but still above stop ($52,205)
$52,000 → Price drops to $52,000, STOP TRIGGERED at $52,205
        → Exit with +4.4% profit ($2,205)

Without trailing stop: Might have held and lost more if price continued dropping.
With trailing stop: Locked in $2,205 profit (+4.4%)!
```

**Acceptance Criteria**:
- [ ] Only activate after minimum profit threshold reached (default: 2.0%)
- [ ] Stop loss NEVER moves down for long positions (only up)
- [ ] Stop loss updated on every price tick when trailing is active
- [ ] Exit executed immediately when trailing stop hit
- [ ] Works correctly with leveraged positions
- [ ] Coordinates with other risk controls (daily loss limit, cool-down)
- [ ] Trailing stop persists across system restarts
- [ ] Track highest price achieved since activation
- [ ] Calculate stop price: highest_price × (1 - trailing_stop_pct / 100)
- [ ] Log trailing stop activations and adjustments
- [ ] Display trailing stop status in position view
- [ ] Support different activation thresholds per symbol
- [ ] Validate trailing_stop_pct is between 1.0% and 5.0%
- [ ] Validate trailing_activation_pct is between 0.5% and 5.0%
- [ ] Deactivate trailing stop if manually disabled during trade
- [ ] Generate alert when trailing stop activates
- [ ] Generate alert when trailing stop executes

**Edge Cases**:
1. **Sudden price spike then crash**: Stop should lock in most of spike
   - Example: Price spikes to $55,000 (stop moves to $54,175), then crashes to $53,000
   - Result: Exit at $54,175 instead of crash price
2. **Price whipsaw**: Multiple activations/deactivations handled correctly
   - Example: Price oscillates around activation threshold
   - Result: Trailing only activates when threshold is exceeded, not on every oscillation
3. **Gap down**: Stop loss might execute at worse price due to slippage
   - Example: Price gaps down from $52,500 to $51,500 overnight
   - Result: Exit at best available price (may be below calculated stop)
4. **Multiple positions**: Each position has independent trailing stop
   - Example: 3 BTCUSDT positions with different entry prices
   - Result: Each tracks its own highest price and stop level
5. **Position size change**: Trailing stop adjusts proportionally if position is partially closed
   - Example: Close 50% of position, trailing stop continues on remaining 50%
   - Result: Stop calculation unchanged, applies to remaining quantity

**Testing Requirements**:
```rust
// Test 1: Activation threshold
test_trailing_activates_at_threshold() -> assert activation at +2.0% profit

// Test 2: Stop moves up with price
test_stop_moves_up_correctly() -> assert stop = highest_price × (1 - 1.5%)

// Test 3: Stop never moves down
test_stop_never_moves_down() -> assert stop only increases for longs

// Test 4: Exit when stop hit
test_exits_at_stop_price() -> assert position closed when price < trailing_stop

// Test 5: No activation before threshold
test_no_activation_before_threshold() -> assert trailing inactive at +1.9% profit

// Test 6: Multiple highs
test_handles_multiple_highs() -> assert stop adjusts with each new high

// Test 7: Price whipsaw
test_price_whipsaw_handling() -> assert stable behavior during oscillations

// Test 8: Gap down execution
test_gap_down_execution() -> assert exits at available price during gap

// Test 9: Multiple positions
test_multiple_positions_independent_stops() -> assert each position independent

// Test 10: Partial position close
test_position_size_change_adjustment() -> assert trailing continues after partial close

// Test 11: Disabled trailing stop
test_disabled_trailing_stop() -> assert no trailing when disabled

// Test 12: Extreme volatility
test_extreme_volatility() -> assert stable behavior in volatile markets

// Test 13: Leverage positions
test_leverage_positions() -> assert correct calculation with leverage

// Test 14: Coordination with daily loss limit
test_coordination_with_daily_loss_limit() -> assert both controls work together

// Test 15: System restart
test_persistence_across_restarts() -> assert trailing state restored after restart
```

**Performance Impact**:
- Profit capture improvement: +20-30% compared to fixed take-profit
- Win rate impact: Minimal (same entry logic)
- Average profit per winning trade: +25% increase
- Risk-reward ratio improvement: From 2:1 to 2.6:1 average
- Calculation overhead: < 1ms per price update
- Memory overhead: 24 bytes per position (highest_price, trailing_active flag)

**Dependencies**: Position manager, portfolio system, real-time price updates
**Test Cases**: [TC-RISK-070, TC-RISK-071, TC-RISK-072, TC-RISK-073, TC-RISK-074]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 2.1: Stop Loss Requirements
- BUSINESS_RULES.md Section 5.2: Profit Protection Strategies

**Implementation Notes**:
- Code Location: `rust-core-engine/src/paper_trading/trade.rs:334-395`
- Method: `update_trailing_stop(current_price, activation_pct, trailing_pct)`
- Settings: `rust-core-engine/src/paper_trading/settings.rs:109-116`
- Fields in PaperTrade struct: `trailing_stop_active: bool`, highest price tracking
- Integration: Called from `engine.rs:397-406` on every price update
- Calculation: `stop_price = highest_price * (1.0 - trailing_pct / 100.0)`

---

### FR-RISK-008: Trailing Stop Loss (Short Positions)

**Priority**: ☑ Critical
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-008`

**Description**:
The system shall implement trailing stop loss functionality for SHORT positions to automatically lock in profits as the market moves favorably (price decreases). The trailing stop moves DOWN with the price, maintaining a fixed percentage distance above the lowest price achieved since activation. This is the inverse of FR-RISK-007 for long positions.

**Purpose**:
Automatically move stop loss down as price moves favorably (decreases) for short positions, locking in profits while allowing further downside to run.

**How It Works (SHORT positions)**:
1. Trade enters profit (current price < entry price)
2. When profit reaches activation threshold (e.g., -2%), trailing starts
3. Stop loss moves DOWN, maintaining fixed percentage above lowest price
4. If price rises by trailing percentage, exit with profit
5. Allows unlimited downside capture while protecting against upward reversals

**Example Scenario (SHORT BTCUSDT)**:
```
Entry: $50,000 (short)
Activation threshold: -2% = $49,000
Trailing distance: 1.5%

Price movement:
$50,000 → Entry (short), no trailing yet
$49,000 → -2% (profit for short!), TRAILING ACTIVATES
        → Stop loss set at: $49,000 + 1.5% = $49,735
$48,000 → New low!
        → Stop loss moves to: $48,000 + 1.5% = $48,720
$47,000 → New low!
        → Stop loss moves to: $47,000 + 1.5% = $47,705
$48,000 → Price rises but still below stop ($48,720)
$48,800 → Price continues rising, STOP TRIGGERED at $48,720
        → Exit with +2.56% profit ($1,280)

Without trailing stop: Might have held and lost profit if price continued rising.
With trailing stop: Locked in $1,280 profit (+2.56%)!
```

**Acceptance Criteria**:
- [ ] Inverted logic from longs (stop moves DOWN as price decreases)
- [ ] Only activate after minimum profit threshold reached (price below entry)
- [ ] Stop loss NEVER moves up for short positions (only down)
- [ ] Stop loss updated on every price tick when trailing is active
- [ ] Exit executed immediately when trailing stop hit
- [ ] Works correctly with leveraged short positions
- [ ] Coordinates with other risk controls
- [ ] Track lowest price achieved since activation
- [ ] Calculate stop price: lowest_price × (1 + trailing_stop_pct / 100)
- [ ] Profit calculation: entry_price - current_price (positive when price drops)
- [ ] Activation when: (entry_price - current_price) / entry_price × 100 ≥ activation_pct
- [ ] Stop triggered when: current_price ≥ trailing_stop_price
- [ ] Log trailing stop activations and adjustments
- [ ] Display trailing stop status in position view
- [ ] Support different activation thresholds per symbol
- [ ] Validate trailing_stop_pct is between 1.0% and 5.0%
- [ ] Validate trailing_activation_pct is between 0.5% and 5.0%
- [ ] Deactivate trailing stop if manually disabled during trade
- [ ] Generate alert when trailing stop activates
- [ ] Generate alert when trailing stop executes

**Edge Cases**:
1. **Sudden price crash then recovery**: Stop should lock in most of crash
   - Example: Price crashes to $45,000 (stop moves to $45,675), then recovers to $47,000
   - Result: Exit at $45,675 instead of recovery price
2. **Price whipsaw**: Multiple activations/deactivations handled correctly
   - Example: Price oscillates around activation threshold
   - Result: Trailing only activates when threshold is exceeded
3. **Gap up**: Stop loss might execute at worse price due to slippage
   - Example: Price gaps up from $48,000 to $49,500 overnight
   - Result: Exit at best available price (may be above calculated stop)
4. **Multiple short positions**: Each position has independent trailing stop
   - Example: 3 BTCUSDT short positions with different entry prices
   - Result: Each tracks its own lowest price and stop level
5. **Position size change**: Trailing stop adjusts proportionally if position is partially closed
   - Example: Close 50% of short position, trailing stop continues on remaining 50%
   - Result: Stop calculation unchanged, applies to remaining quantity

**Testing Requirements**:
```rust
// Test 1: Activation threshold for shorts
test_trailing_activates_at_threshold_short() -> assert activation at -2.0% (price drop)

// Test 2: Stop moves down with price
test_stop_moves_down_with_price_short() -> assert stop = lowest_price × (1 + 1.5%)

// Test 3: Stop never moves up for shorts
test_stop_never_moves_up_short() -> assert stop only decreases for shorts

// Test 4: Exit when stop hit
test_exits_at_stop_price_short() -> assert position closed when price > trailing_stop

// Test 5: No activation before threshold
test_no_activation_before_threshold_short() -> assert trailing inactive at -1.9% profit

// Test 6: Multiple lows
test_handles_multiple_lows_short() -> assert stop adjusts with each new low

// Test 7: Price whipsaw
test_price_whipsaw_handling_short() -> assert stable behavior during oscillations

// Test 8: Gap up execution
test_gap_up_execution_short() -> assert exits at available price during gap

// Test 9: Multiple short positions
test_multiple_short_positions_independent_stops() -> assert each position independent

// Test 10: Partial position close
test_position_size_change_adjustment_short() -> assert trailing continues after partial close

// Test 11: Disabled trailing stop
test_disabled_trailing_stop_short() -> assert no trailing when disabled

// Test 12: Extreme volatility
test_extreme_volatility_short() -> assert stable behavior in volatile markets

// Test 13: Leverage short positions
test_leverage_short_positions() -> assert correct calculation with leverage

// Test 14: Long and short coordination
test_long_and_short_trailing_coordination() -> assert both work simultaneously

// Test 15: System restart
test_persistence_across_restarts_short() -> assert trailing state restored after restart
```

**Performance Impact**:
- Same as FR-RISK-007 (20-30% profit improvement)
- Works symmetrically for short positions
- No additional overhead compared to long trailing stops

**Dependencies**: Position manager, portfolio system, real-time price updates
**Test Cases**: [TC-RISK-075, TC-RISK-076, TC-RISK-077, TC-RISK-078, TC-RISK-079]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 2.1: Stop Loss Requirements
- BUSINESS_RULES.md Section 5.2: Profit Protection Strategies

**Implementation Notes**:
- Code Location: `rust-core-engine/src/paper_trading/trade.rs:334-395`
- Same method as FR-RISK-007, logic inverted based on `trade_type`
- Settings: `rust-core-engine/src/paper_trading/settings.rs:109-116`
- Fields in PaperTrade struct: `trailing_stop_active: bool`, lowest price tracking for shorts
- Integration: Called from `engine.rs:397-406` on every price update
- Calculation: `stop_price = lowest_price * (1.0 + trailing_pct / 100.0)`

---

### FR-RISK-009: Risk Score Calculation

**Priority**: ☐ Medium
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-009`

**Description**:
The system shall calculate a comprehensive risk score (0-100) for each trade and the overall portfolio. The risk score aggregates multiple risk factors into a single, easy-to-understand metric. Higher scores indicate higher risk. Scores above 80 should trigger manual review or rejection. With trailing stops enabled, the risk score is reduced by 20% to reflect improved profit protection.

**Acceptance Criteria**:
- [ ] Calculate trade risk score based on factors:
  - Position size relative to balance (weight: 25%)
  - Leverage used (weight: 20%)
  - Stop-loss distance (weight: 15%)
  - Signal confidence (weight: 15%)
  - Market volatility (weight: 10%)
  - Correlation with existing positions (weight: 10%)
  - Account drawdown status (weight: 5%)
- [ ] Normalize each factor to 0-100 scale
- [ ] Calculate weighted average: Σ(Factor × Weight)
- [ ] Apply trailing stop benefit: risk_score × (1 - 0.2) if trailing enabled
- [ ] Calculate portfolio risk score based on:
  - Current drawdown percentage
  - Margin level
  - Number of open positions
  - Average position risk
  - Portfolio concentration (HHI index)
- [ ] Categorize risk levels:
  - 0-20: Very Low Risk (green)
  - 21-40: Low Risk (light green)
  - 41-60: Moderate Risk (yellow)
  - 61-80: High Risk (orange)
  - 81-100: Very High Risk (red)
- [ ] Reject trades automatically if risk score > 90
- [ ] Require confirmation if risk score 80-90
- [ ] Display risk score prominently in UI before trade execution
- [ ] Track historical risk scores for analysis
- [ ] Generate risk score reports showing score distribution
- [ ] Adjust risk score weights based on market conditions
- [ ] Support custom risk scoring formulas per strategy

**Dependencies**: Portfolio system, market data, volatility calculation
**Test Cases**: [TC-RISK-060, TC-RISK-061]

**Business Rules Referenced**:
- Custom risk assessment logic (to be defined)

**Implementation Notes**:
- New module to be created: `risk_score_calculator.rs`
- Integration with pre-trade risk checks
- Score displayed in UI before trade confirmation
- Historical scores stored in trade records
- Trailing stop reduces risk score by 20%

---

### FR-RISK-010: Correlation Risk Management

**Priority**: ☐ Medium
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-010`

**Description**:
The system shall monitor and limit exposure to correlated assets to prevent concentration risk. Cryptocurrencies often move together (e.g., altcoins follow BTC), so holding multiple correlated positions doesn't provide true diversification. System shall calculate correlation and enforce limits.

**Acceptance Criteria**:
- [ ] Calculate correlation coefficients between all trading pairs
- [ ] Use rolling 30-day price data for correlation calculation
- [ ] Update correlation matrix daily at market close
- [ ] Define correlation threshold: 0.7 (highly correlated)
- [ ] Group assets by correlation: BTC-dominant, ETH-dominant, independent
- [ ] Limit total exposure to correlated assets: 30% of portfolio value
- [ ] Count positions in same correlation group
- [ ] Enforce maximum positions per correlation group (default: 5)
- [ ] Display correlation information in position analysis
- [ ] Warn user when opening position in correlated asset
- [ ] Provide diversification score: 0-100 (higher = more diversified)
- [ ] Calculate portfolio beta relative to BTC
- [ ] Support manual correlation overrides by admin
- [ ] Track correlation breakdowns (when correlations change dramatically)
- [ ] Adjust position sizing based on correlation (reduce size for correlated positions)

**Dependencies**: Market data, statistical analysis library
**Test Cases**: [TC-RISK-070, TC-RISK-071]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 2: Risk Management Rules (general principles)
- Settings: `rust-core-engine/src/paper_trading/settings.rs:101` - correlation_limit

**Implementation Notes**:
- Correlation calculation to be implemented in market data analyzer
- Pearson correlation coefficient using daily returns
- Store correlation matrix in Redis cache for fast access
- Integration with pre-trade risk checks

---

### FR-RISK-011: Emergency Risk Controls

**Priority**: ☑ High
**Status**: ☐ Not Started
**Code Tags**: `@spec:FR-RISK-011`

**Description**:
The system shall provide emergency risk controls that can be activated manually or automatically during extreme market conditions. These "panic button" features help limit losses during black swan events, flash crashes, or major security incidents.

**Acceptance Criteria**:
- [ ] Implement "Close All Positions" emergency button
- [ ] Execute market orders to close all positions within 30 seconds
- [ ] Implement "Halt All Trading" emergency button
- [ ] Prevent all new positions, allow only closures
- [ ] Auto-trigger emergency close if:
  - Portfolio drawdown > 20% in 1 hour
  - Margin level < 100% (liquidation imminent)
  - Suspected security breach detected
- [ ] Implement "Reduce Leverage" emergency action
- [ ] Close highest leverage positions first
- [ ] Implement "Reduce to Top N Positions" emergency action
- [ ] Close smallest and most losing positions
- [ ] Require admin authentication for manual emergency actions
- [ ] Log all emergency actions with timestamp, trigger, and result
- [ ] Send immediate notifications for all emergency actions
- [ ] Prevent emergency actions from being triggered repeatedly (cooldown: 5 minutes)
- [ ] Generate incident report after each emergency action
- [ ] Support rollback of emergency actions if triggered erroneously
- [ ] Test emergency controls regularly (monthly drill)
- [ ] Measure emergency action latency: < 5 seconds from trigger to first order

**Dependencies**: Trading engine, notification system, authentication
**Test Cases**: [TC-RISK-080, TC-RISK-081]

**Business Rules Referenced**:
- BUSINESS_RULES.md Section 8.2: Circuit Breakers
- BUSINESS_RULES.md Section 9: Error Handling Rules

**Implementation Notes**:
- Emergency controls API endpoint: POST /api/v1/risk/emergency
- Requires special admin permission level
- Bypass normal rate limits for emergency orders
- WebSocket broadcast to all connected clients
- Integration with exchange's emergency close features if available

---

## Use Cases

### UC-RISK-001: Validate New Trade

**Actor**: Trading System
**Preconditions**:
- User has submitted trade signal
- Market data is available

**Main Flow**:
1. System receives trade signal (symbol, side, entry, stop-loss, quantity)
2. System calls `risk_manager.can_open_position()`
3. System checks trading is enabled
4. System validates signal confidence meets threshold (0.7+ for StrongBuy/Sell)
5. System checks risk-reward ratio is acceptable (1.5:1 minimum)
6. System verifies stop-loss is set and within 10% of entry
7. System counts current open positions
8. System validates position count < max_positions (10)
9. System calculates position risk amount
10. System verifies risk < 2% of account balance
11. System checks daily loss limit not exceeded
12. System returns approval: Ok(true)
13. Trading engine proceeds with order execution

**Alternative Flows**:
- **Alt 1**: Trading disabled
  1. System returns Ok(false) with reason "Trading is disabled"
  2. Trade is rejected, user is notified
- **Alt 2**: Signal confidence too low
  1. System checks confidence < threshold
  2. System returns Ok(false) with reason "Confidence below threshold"
  3. Trade is rejected, user can override manually
- **Alt 3**: Max positions reached
  1. System counts open positions = 10
  2. System returns Ok(false) with reason "MAX_POSITIONS_EXCEEDED"
  3. User must close a position before opening new one
- **Alt 4**: Daily loss limit reached
  1. System calculates daily loss > 5%
  2. System returns Ok(false) with reason "Daily loss limit exceeded"
  3. Trading is halted for 24 hours

**Postconditions**:
- Trade is approved or rejected
- Rejection reason logged
- User notified of outcome

**Exception Handling**:
- Market data unavailable: Reject trade with "Cannot validate risk"
- Calculation error: Log error, default to rejection

---

### UC-RISK-002: Monitor Margin Level

**Actor**: Risk Monitoring Service
**Preconditions**:
- Portfolio has open leveraged positions
- Price updates streaming in real-time

**Main Flow**:
1. Service receives price update for symbol
2. Service updates position unrealized PnL
3. Service recalculates portfolio equity
4. Service calculates margin level: (Equity / Margin Used) × 100
5. Service checks if margin level < 200% (warning threshold)
6. Service checks if margin level < 150% (critical threshold)
7. If critical, service triggers liquidation risk alert
8. Service sends push notification to user
9. Service logs margin level event
10. Service continues monitoring

**Alternative Flows**:
- **Alt 1**: Margin level < 200% (warning)
  1. Service sends warning notification
  2. Service suggests actions: "Add funds or close positions"
  3. Service increases monitoring frequency to every second
- **Alt 2**: Margin level < 110% (liquidation)
  1. Service auto-closes most losing position
  2. Service recalculates margin level
  3. If still < 110%, service closes next worst position
  4. Service sends critical alert to user and admin
  5. Service logs emergency action taken

**Postconditions**:
- Margin level monitored and recorded
- Alerts sent if thresholds breached
- Emergency actions taken if necessary

**Exception Handling**:
- Cannot close position (no liquidity): Escalate to admin
- Alert system failure: Log to file, retry notification

---

### UC-RISK-003: Enforce Daily Loss Limit

**Actor**: Risk Manager
**Preconditions**:
- Trader is actively trading
- Some trades have been closed today

**Main Flow**:
1. Trader attempts to open new position
2. System calls `can_open_position()` pre-trade check
3. System queries all trades closed in last 24 hours
4. System calculates sum of realized PnL: daily_loss = -300 USDT
5. System calculates daily loss percentage: (300 / 10000) × 100 = 3%
6. System compares to limit: 3% < 5%
7. System allows trade to proceed
8. Position is opened successfully

**Alternative Flows**:
- **Alt 1**: Daily loss limit exceeded
  1. System calculates daily loss = -550 USDT (5.5%)
  2. System compares to limit: 5.5% > 5%
  3. System rejects trade with reason "Daily loss limit exceeded"
  4. System sends notification to trader
  5. System displays lockout time remaining: "Trading locked for 18 hours"
  6. System logs daily loss limit event
  7. Trader can only close existing positions
- **Alt 2**: Near daily loss limit (80% of limit)
  1. System calculates daily loss = 4.2%
  2. System allows trade but sends warning
  3. Notification: "You have used 84% of your daily loss limit"
  4. Trader proceeds with caution

**Postconditions**:
- Daily loss tracked accurately
- Limit enforced, trading halted if exceeded
- Trader notified of status

**Exception Handling**:
- Clock sync issue: Use exchange server time as source of truth
- Trade history incomplete: Query database for complete history

---

### UC-RISK-004: Calculate Risk Score

**Actor**: Risk Calculation Service
**Preconditions**:
- Trader is reviewing trade before execution
- All risk factors can be calculated

**Main Flow**:
1. UI requests risk score for proposed trade
2. Service receives trade parameters
3. Service calculates position size factor: 8% of balance = 32 points (scaled)
4. Service calculates leverage factor: 10x leverage = 40 points
5. Service calculates stop-loss factor: 3% stop = 30 points
6. Service calculates signal confidence factor: 0.75 confidence = 25 points
7. Service calculates volatility factor: Current ATR = 20 points
8. Service calculates correlation factor: One correlated position = 15 points
9. Service weights and sums: (32×0.25 + 40×0.20 + 30×0.15 + 25×0.15 + 20×0.10 + 15×0.10) = 31 points
10. Service checks if trailing stop enabled: YES
11. Service applies trailing stop benefit: 31 × (1 - 0.2) = 24.8 points
12. Service returns risk score: 25 (Low Risk - Green)
13. UI displays risk score with color indicator and trailing stop badge
14. Trader sees "Low Risk (with Trailing Stop)" and proceeds with trade

**Alternative Flows**:
- **Alt 1**: High risk score (> 80)
  1. Service calculates risk score = 85
  2. Service returns score with "Very High Risk" label
  3. UI displays red warning banner
  4. UI prompts: "This trade is very risky. Are you sure?"
  5. If trader confirms, trade requires second approval
  6. If score > 90, trade is automatically rejected
- **Alt 2**: Missing factors
  1. Service cannot calculate volatility (data unavailable)
  2. Service uses default value or skips factor
  3. Service notes incomplete calculation in response
  4. Risk score marked as "approximate"

**Postconditions**:
- Risk score calculated and displayed
- Trader informed of risk level and trailing stop benefit
- Score logged for historical analysis

**Exception Handling**:
- Calculation error: Return conservative high score (80)
- Timeout: Return "Cannot calculate, try again"

---

### UC-RISK-005: Trigger Emergency Close All

**Actor**: Admin / Automated System
**Preconditions**:
- Extreme market event detected OR admin manually triggered
- Portfolio has multiple open positions

**Main Flow**:
1. Emergency trigger activated (manual button OR automatic threshold)
2. System authenticates admin user (if manual)
3. System broadcasts "Emergency Close in Progress" to all clients
4. System pauses all new order submissions
5. System retrieves all open positions (10 positions)
6. System creates market close orders for each position
7. System submits orders in parallel for speed
8. System monitors order fills in real-time
9. All positions closed within 30 seconds
10. System calculates total loss from emergency close: -450 USDT
11. System generates incident report with details
12. System sends notifications to user, admin, compliance
13. System logs complete emergency action timeline
14. System enables trading after cooldown (5 minutes)

**Alternative Flows**:
- **Alt 1**: Some positions fail to close
  1. System identifies failed closures (e.g., low liquidity pair)
  2. System retries with larger slippage tolerance
  3. If still fails, system reduces position size and retries
  4. System escalates to admin for manual intervention
- **Alt 2**: Emergency close triggered erroneously
  1. System has 5-second confirmation window
  2. Admin can cancel within 5 seconds
  3. If cancelled, positions remain open
  4. Incident logged as "Emergency close cancelled"

**Postconditions**:
- All or most positions closed
- Portfolio risk significantly reduced
- Complete audit trail of emergency action
- Cooldown period active

**Exception Handling**:
- Exchange API failure: Retry with exponential backoff
- Network timeout: Use backup connection
- Cannot close critical position: Log issue, notify admin immediately

---

### UC-RISK-006: Activate Trailing Stop on Profitable Trade

**Actor**: Risk Monitoring Service
**Preconditions**:
- Position is open and in profit
- Trailing stop is enabled in settings
- Price updates are streaming in real-time

**Main Flow**:
1. Service receives price update for symbol with open position
2. Service calculates current profit percentage
3. Service checks if profit ≥ activation threshold (default: 2.0%)
4. Profit is 2.1%, ACTIVATION THRESHOLD REACHED
5. Service activates trailing stop
6. Service sets trailing_stop_active = true
7. Service records highest price achieved (for long) or lowest price (for short)
8. Service calculates initial trailing stop price
9. Service logs: "Trailing stop activated for [SYMBOL] at [PRICE]"
10. Service sends notification to user: "Trailing stop activated!"
11. Service updates position display with trailing stop badge
12. Service continues monitoring price on every tick

**Alternative Flows**:
- **Alt 1**: Trailing stop already active
  1. Service checks trailing_stop_active = true
  2. Service updates highest/lowest price if new extreme reached
  3. Service recalculates trailing stop price
  4. Service logs stop adjustment
  5. No activation notification sent
- **Alt 2**: Price reverses before activation
  1. Profit reaches 1.9% (below threshold)
  2. Price reverses, profit drops to 1.0%
  3. Trailing stop never activates
  4. Regular stop-loss remains in effect

**Postconditions**:
- Trailing stop activated and monitoring price
- User notified of activation
- Stop price calculated and ready to execute

**Exception Handling**:
- Price data unavailable: Skip update, use last known price
- Calculation error: Log error, keep previous stop price

---

### UC-RISK-007: Execute Trailing Stop on Reversal

**Actor**: Risk Monitoring Service
**Preconditions**:
- Position is open with active trailing stop
- Price has moved favorably and trailing stop is tracking
- Price now reverses

**Main Flow**:
1. Service receives price update
2. Service checks if trailing stop is active: YES
3. Service retrieves current trailing stop price
4. For LONG: Service checks if current_price ≤ trailing_stop_price
5. Current price = $52,000, Trailing stop = $52,205
6. Condition NOT met, continue monitoring
7. Next price update: $52,150
8. Still above trailing stop, continue monitoring
9. Next price update: $52,205
10. Price EQUALS trailing stop, TRIGGER EXECUTION
11. Service immediately closes position at market price
12. Service calculates final profit: +4.4% ($2,205)
13. Service logs: "Trailing stop executed for [SYMBOL] at [PRICE], profit: +4.4%"
14. Service sends notification: "Position closed by trailing stop, profit: +4.4%"
15. Service updates portfolio metrics
16. Service saves closed trade to database

**Alternative Flows**:
- **Alt 1**: Gap down below trailing stop (slippage)
  1. Price gaps from $52,500 to $51,800 (below stop at $52,205)
  2. Service detects gap, executes at best available price
  3. Execution price: $51,850 (slippage applied)
  4. Final profit: +3.7% instead of +4.4%
  5. Service logs slippage event
  6. User notified of execution with slippage note
- **Alt 2**: Execution fails (low liquidity)
  1. Market order submitted
  2. Order only 50% filled due to low liquidity
  3. Service retries with remaining quantity
  4. Second order fills remaining 50%
  5. Average execution price calculated
  6. User notified of partial fill execution

**Postconditions**:
- Position closed with profit
- Trailing stop deactivated
- Portfolio updated with realized PnL
- Trade record saved to database

**Exception Handling**:
- Exchange API failure: Retry with exponential backoff
- Network timeout: Use backup connection
- Cannot close position: Escalate to admin immediately

---

## Data Requirements

**Input Data**:
- config: TradingConfig, Required, Validated, "Risk management configuration"
- analysis: MultiTimeframeAnalysis, Required, Valid signals, "AI trading signals"
- account_balance: f64, Required, > 0, "Current account balance"
- open_positions_count: u32, Required, >= 0, "Number of open positions"
- daily_realized_pnl: f64, Required, Any value, "24-hour realized profit/loss"
- margin_level: f64, Required, >= 0, "Current margin level percentage"
- position_size: f64, Required, > 0, "Proposed position size"
- leverage: u8, Required, 1-125, "Leverage for position"
- stop_loss: f64, Required, > 0, "Stop-loss price"
- entry_price: f64, Required, > 0, "Entry price for position"
- current_price: f64, Required, > 0, "Current market price for trailing stop"
- trailing_stop_enabled: bool, Required, "Whether trailing stop is enabled"
- trailing_stop_pct: f64, Optional, 1.0-5.0, "Trailing stop distance percentage"
- trailing_activation_pct: f64, Optional, 0.5-5.0, "Profit threshold for activation"

**Output Data**:
- can_trade: bool, "Whether trade is approved"
- rejection_reason: String, "Reason if trade rejected"
- risk_score: f64, "Calculated risk score (0-100)"
- risk_score_with_trailing: f64, "Risk score adjusted for trailing stop benefit"
- max_positions: u32, "Maximum allowed positions"
- risk_percentage: f64, "Risk percentage configuration"
- margin_requirement: f64, "Required margin for trade"
- trailing_stop_price: f64, "Current trailing stop price if active"
- trailing_stop_active: bool, "Whether trailing stop is currently active"
- highest_price: f64, "Highest price achieved since trailing activation (longs)"
- lowest_price: f64, "Lowest price achieved since trailing activation (shorts)"

**Data Validation**:
- Rule 1: entry_price must be positive and within 10% of current market price
- Rule 2: stop_loss must be on correct side (below entry for LONG, above for SHORT)
- Rule 3: stop_loss distance must be <= 10% of entry price
- Rule 4: leverage must be within allowed range for trading pair
- Rule 5: position_size must be >= min_size and <= max_size
- Rule 6: account_balance must be positive
- Rule 7: margin_level must be >= 150% to open new positions
- Rule 8: Signal confidence must be valid: 0.0 to 1.0 range
- Rule 9: trailing_stop_pct must be between 1.0% and 5.0% if enabled
- Rule 10: trailing_activation_pct must be between 0.5% and 5.0% if enabled
- Rule 11: trailing_stop_pct must be less than activation_pct (logical constraint)

**Data Models** (reference to DATA_MODELS.md):
- TradingConfig: Risk management configuration
- MultiTimeframeAnalysis: AI signal analysis
- RiskSettings: Comprehensive risk parameters including trailing stop settings
- Position: Position risk data including trailing stop state
- PaperTrade: Trade execution data including trailing stop fields

---

## Interface Requirements

**API Endpoints** (reference to API_SPEC.md):
```
POST   /api/v1/risk/validate-trade           # Pre-trade risk validation
GET    /api/v1/risk/limits                    # Get current risk limits
PUT    /api/v1/risk/limits                    # Update risk limits (admin)
GET    /api/v1/risk/daily-loss                # Get daily loss status
GET    /api/v1/risk/score                     # Calculate risk score
GET    /api/v1/risk/margin-status             # Get margin level and status
POST   /api/v1/risk/emergency/close-all       # Emergency close all positions
POST   /api/v1/risk/emergency/halt-trading    # Emergency halt all trading
GET    /api/v1/risk/alerts                    # Get active risk alerts
PUT    /api/v1/risk/alerts/{id}/acknowledge   # Acknowledge risk alert
GET    /api/v1/risk/trailing-stop/status      # Get trailing stop status for positions
PUT    /api/v1/risk/trailing-stop/settings    # Update trailing stop settings
POST   /api/v1/risk/trailing-stop/activate    # Manually activate trailing stop
POST   /api/v1/risk/trailing-stop/deactivate  # Manually deactivate trailing stop
```

**UI Screens** (reference to UI-COMPONENTS.md):
- Risk Dashboard: Real-time risk metrics and alerts
- Trade Confirmation: Risk score display before execution with trailing stop indicator
- Emergency Controls: Admin panel for emergency actions
- Risk Alerts: Notification center for risk warnings
- Risk Settings: User configuration of risk parameters including trailing stop
- Position View: Display trailing stop status, activation price, current stop price

**External Systems** (reference to INTEGRATION_SPEC.md):
- Portfolio System: Current exposure and position data
- Trading Engine: Trade execution coordination
- Market Data: Real-time prices for risk calculations and trailing stop updates
- Notification Service: Risk alert delivery
- Binance Exchange: Leverage limits and margin requirements

---

## Non-Functional Requirements

**Performance**:
- Pre-trade risk validation: < 50ms (target: 25ms)
- Risk score calculation: < 100ms (including trailing stop benefit)
- Margin level monitoring: Update every 1 second
- Trailing stop calculation: < 1ms per price update
- Trailing stop activation latency: < 10ms from threshold reached
- Emergency close execution: < 30 seconds for all positions
- Concurrent risk checks: Support 1000 trades/second validation
- Risk alert delivery: < 1 second from detection to notification

**Security**:
- Authentication: Admin-level JWT required for emergency actions
- Authorization: Users cannot modify own risk limits beyond safe ranges
- Audit logging: All risk decisions logged with full context
- Data encryption: Risk settings encrypted at rest
- Rate limiting: Emergency endpoints limited to prevent abuse
- Trailing stop manipulation: Prevent unauthorized stop price modifications

**Scalability**:
- Horizontal scaling: Risk validation is stateless, easily distributed
- Load balancing: Distribute risk checks across multiple instances
- Caching: Cache risk limits for 1 minute to reduce database load
- Database: Index on user_id, created_at for fast daily loss queries
- Trailing stop state: Store in memory with periodic persistence

**Reliability**:
- Uptime target: 99.99% availability (more critical than other services)
- Error rate: < 0.01% of risk validations (10x stricter than general services)
- Recovery time objective (RTO): 1 minute
- Recovery point objective (RPO): 0 seconds (no data loss acceptable)
- Fail-safe: If risk service unavailable, default to rejecting all trades
- Trailing stop persistence: Survive system restarts without data loss

**Maintainability**:
- Code coverage: > 95% for risk module (critical code)
- Technical debt: Zero tolerance for risk calculation bugs
- Documentation: Every risk rule documented with business justification
- Testing: Daily automated tests of all risk scenarios including trailing stops

---

## Implementation Notes

**Code Locations**:
- Rust: `rust-core-engine/src/trading/risk_manager.rs` (Core risk logic)
- Rust: `rust-core-engine/src/paper_trading/settings.rs` (Risk settings including trailing stop)
- Rust: `rust-core-engine/src/config.rs` (System configuration)
- Rust: `rust-core-engine/src/paper_trading/portfolio.rs` (Margin monitoring)
- Rust: `rust-core-engine/src/paper_trading/trade.rs:334-395` (Trailing stop logic)
- Rust: `rust-core-engine/src/paper_trading/engine.rs:397-406` (Trailing stop integration)

**Dependencies**:
- External libraries:
  - anyhow 1.0 (error handling)
  - tracing 0.1 (structured logging)
- Internal modules:
  - position_manager: Position counting and exposure
  - portfolio: Balance and margin calculations
  - market_data: Price data for risk calculations and trailing stop updates

**Design Patterns**:
- Strategy Pattern: Different risk validation strategies per account type
- Chain of Responsibility: Sequential risk checks with early exit
- Circuit Breaker: Emergency controls to prevent cascading failures
- Observer Pattern: Risk alert notifications to multiple channels
- State Pattern: Trailing stop activation and tracking states

**Configuration**:
- max_positions: Default 10 (range: 1 - 100)
- max_risk_per_trade_pct: Default 2% (range: 0.5% - 5%)
- daily_loss_limit_pct: Default 5% (range: 2% - 10%)
- max_leverage: Default 50 (range: 1 - 125, testnet max: 20)
- min_margin_level: Default 200% (range: 150% - 500%)
- max_stop_loss_pct: Default 10% (range: 2% - 20%)
- trailing_stop_enabled: Default true
- trailing_stop_pct: Default 1.5% (range: 1.0% - 5.0%)
- trailing_activation_pct: Default 2.0% (range: 0.5% - 5.0%)

---

## Testing Strategy

**Unit Tests**:
- Test class: `risk_manager::tests` module
- Coverage target: 95%
- Key test scenarios:
  1. can_open_position() with trading enabled/disabled
  2. Signal confidence validation (StrongBuy, Buy, Sell, Hold)
  3. Risk-reward ratio validation (below, at, above 1.5:1)
  4. Max positions enforcement (at limit, below limit, overflow)
  5. Stop-loss validation (correct side, distance, missing)
  6. Daily loss limit calculation and enforcement
  7. Leverage validation by trading pair
  8. Margin level calculations
  9. Edge cases: zero positions, maximum leverage, negative balance
  10. Trailing stop activation at threshold
  11. Trailing stop price calculation (long and short)
  12. Trailing stop never moves against position
  13. Trailing stop execution on reversal
  14. Trailing stop with slippage and gaps
  15. Multiple positions with independent trailing stops

**Integration Tests**:
- Test suite: `tests/risk_integration_tests.rs`
- Integration points tested:
  1. Risk Manager + Portfolio: Coordinate on position limits
  2. Risk Manager + Trading Engine: Reject invalid trades
  3. Risk Manager + Market Data: Use real-time prices for calculations
  4. Risk Manager + Notification: Send alerts on limit breaches
  5. Emergency Controls + Trading Engine: Close all positions
  6. Trailing Stop + Price Updates: Real-time stop adjustment
  7. Trailing Stop + Position Close: Automatic execution on trigger

**E2E Tests**:
- Test scenarios: `e2e/risk_workflows.spec.ts`
- User flows tested:
  1. Attempt trade with max positions → verify rejection
  2. Approach daily loss limit → verify warning then lockout
  3. Margin level drops → verify alerts escalate
  4. Emergency close triggered → verify all positions closed
  5. Risk score displayed → verify accurate calculation
  6. Trailing stop activates → verify notification and display
  7. Trailing stop executes → verify position closed with profit
  8. Multiple trailing stops → verify independent operation

**Performance Tests**:
- Load test: 10,000 risk validations in parallel, all complete in < 1 second
- Stress test: Emergency close 100 positions simultaneously
- Endurance test: Run risk monitoring for 7 days without degradation
- Spike test: Handle 10x normal trade validation load
- Trailing stop stress: 1000 price updates/second with active trailing stops

**Security Tests**:
- Penetration test: Attempt to bypass risk limits via API manipulation
- Authorization test: Verify non-admin cannot trigger emergency actions
- Injection test: SQL injection attempts on risk queries
- Fuzzing: Random invalid inputs to risk validation
- Trailing stop manipulation: Attempt unauthorized stop price changes

---

## Deployment

**Environment Requirements**:
- Development: Single instance, in-memory limits, test data
- Staging: 2 instances, Redis for shared state, production-like settings
- Production: 4+ instances, Redis cluster, strict limits enforced

**Configuration Changes**:
- Add RISK_VALIDATION_TIMEOUT environment variable (default: 50ms)
- Add EMERGENCY_CLOSE_TIMEOUT (default: 30s)
- Add MAX_LEVERAGE_TESTNET (default: 20)
- Add RISK_ALERT_CHANNELS (WebSocket, Email, SMS)
- Add TRAILING_STOP_ENABLED (default: true)
- Add TRAILING_STOP_PCT (default: 1.5)
- Add TRAILING_ACTIVATION_PCT (default: 2.0)

**Database Migrations**:
- Migration 1: Create `risk_events` collection for audit trail
- Migration 2: Add indexes on user_id, timestamp for fast daily loss queries
- Migration 3: Create `emergency_actions` collection for incident tracking
- Migration 4: Add trailing stop fields to positions collection (trailing_stop_active, highest_price, lowest_price)
- Rollback plan: Drop new collections and fields, no impact on existing data

**Rollout Strategy**:
- Phase 1: Deploy to development, run comprehensive test suite (Day 1)
- Phase 2: Deploy to staging, shadow mode (log but don't enforce) (Day 3-7)
- Phase 3: Staging with enforcement enabled, monitor for false positives (Day 8-10)
- Phase 4: Canary production deployment to 10% users (Day 11-13)
- Phase 5: Full production if error rate < 0.1% (Day 14)
- Rollback trigger: False positive rate > 1%, or any critical bug

---

## Monitoring & Observability

**Metrics to Track**:
- risk_validation_duration_ms: P50, P95, P99 latency, Alert if P95 > 75ms
- risk_validation_rejections_total: Count by reason, Alert on spikes
- daily_loss_limit_breaches: Count per day, Alert if > 5% of users
- margin_level_alerts: Count by severity (warning, critical)
- emergency_actions_triggered: Count by type, Alert on any occurrence
- risk_score_distribution: Histogram of risk scores
- false_positive_rate: Percentage of incorrectly rejected trades
- trailing_stop_activations: Count per day, Target > 40% of profitable trades
- trailing_stop_executions: Count per day with profit distribution
- trailing_stop_calculation_duration_ms: P50, P95, P99, Alert if P95 > 5ms
- trailing_stop_profit_improvement_pct: Average vs fixed take-profit, Target +20-30%

**Logging**:
- Log level: INFO for decisions, DEBUG for calculations
- Key log events:
  1. Risk validation started: trade_params, user_id
  2. Risk check failed: check_name, reason, values
  3. Risk validation passed: duration_ms, all_checks
  4. Daily loss limit approached: current_loss, limit, percentage_used
  5. Margin level warning: margin_level, threshold, action_taken
  6. Emergency action triggered: trigger_type, positions_affected, outcome
  7. Risk alert sent: alert_type, user_id, channel
  8. Configuration changed: old_value, new_value, changed_by
  9. Trailing stop activated: symbol, activation_price, profit_pct
  10. Trailing stop adjusted: symbol, new_stop_price, highest/lowest_price
  11. Trailing stop executed: symbol, execution_price, profit_amount, profit_pct

**Alerts**:
- Critical: Emergency action triggered → Notify admin immediately
- Critical: Risk service unavailable → Page on-call engineer
- Warning: Risk validation latency > 100ms → Investigate performance
- Warning: High rejection rate (> 20%) → Review risk parameters
- Info: Daily loss limit reached for user → Notify user
- Info: Trailing stop activated → Notify user
- Info: Trailing stop executed with profit → Notify user

**Dashboards**:
- Risk Operations Dashboard: Validation latency, rejection reasons, throughput
- Risk Health Dashboard: Margin levels across users, daily loss distribution
- Emergency Response Dashboard: Emergency action history, response times
- Compliance Dashboard: Risk limit adherence, audit trail completeness
- Trailing Stop Dashboard: Activation rate, execution rate, profit improvement, current active stops

---

## Traceability

**Requirements**:
- User Story: "As a trader, I want risk limits to prevent catastrophic losses"
- User Story: "As a trader, I want to maximize profits from winning trades while protecting gains"
- User Story: "As a compliance officer, I need audit trail of all risk decisions"
- Business Rule: [BUSINESS_RULES.md#1.1](../../BUSINESS_RULES.md) - Maximum Positions
- Business Rule: [BUSINESS_RULES.md#2.1](../../BUSINESS_RULES.md) - Stop-Loss Requirements
- Business Rule: [BUSINESS_RULES.md#2.2](../../BUSINESS_RULES.md) - Daily Loss Limit
- Business Rule: [BUSINESS_RULES.md#1.3](../../BUSINESS_RULES.md) - Leverage Limits
- Business Rule: [BUSINESS_RULES.md#2.4](../../BUSINESS_RULES.md) - Margin Requirements
- Business Rule: [BUSINESS_RULES.md#5.2](../../BUSINESS_RULES.md) - Profit Protection Strategies

**Design**:
- Architecture: Risk management as cross-cutting concern
- API Spec: [API_SPEC.md#risk-endpoints](../../API_SPEC.md)
- Data Model: [DATA_MODELS.md#risk-settings](../../DATA_MODELS.md)
- Integration: [INTEGRATION_SPEC.md#risk-trading-integration](../../INTEGRATION_SPEC.md)

**Test Cases**:
- Unit: [TC-RISK-001] Max positions enforcement
- Unit: [TC-RISK-010] Stop-loss validation
- Unit: [TC-RISK-020] Daily loss limit calculation
- Unit: [TC-RISK-070] Trailing stop activation at threshold
- Unit: [TC-RISK-071] Trailing stop price calculation
- Unit: [TC-RISK-072] Trailing stop never moves against position
- Unit: [TC-RISK-073] Trailing stop execution on reversal
- Unit: [TC-RISK-074] Trailing stop with slippage
- Integration: [TC-RISK-040] Pre-trade validation workflow
- Integration: [TC-RISK-075] Trailing stop with real-time prices
- E2E: [TC-RISK-050] Complete risk management lifecycle
- E2E: [TC-RISK-076] Trailing stop activation and execution workflow

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Risk service becomes bottleneck at scale | High | Medium | Implement caching, optimize calculations, horizontal scaling |
| False positives reject legitimate trades | High | Medium | Extensive testing, adjustable thresholds, override mechanism |
| Race conditions in concurrent risk checks | High | Low | Use atomic operations, implement distributed locks |
| Daily loss calculation includes unrealized PnL incorrectly | Medium | Low | Clear specification: only realized PnL counts, extensive unit tests |
| Emergency close fails during extreme market volatility | Critical | Low | Multiple fallback mechanisms, manual override capability |
| Leverage limits not synced with exchange limits | Medium | Medium | Daily sync job, validation against exchange API |
| Margin level calculation drift from exchange calculation | High | Low | Periodic reconciliation with exchange, alert on discrepancy |
| Risk limits can be bypassed by API manipulation | Critical | Low | Server-side validation only, audit all limit changes |
| Trailing stop not activated due to price update lag | Medium | Medium | Ensure price updates < 1 second, backup activation on next tick |
| Trailing stop executed prematurely due to flash crash | Medium | Low | Validate price movement, require confirmation on extreme moves |
| Trailing stop state lost on system restart | High | Low | Persist state to database, restore on startup |
| Multiple trailing stops cause performance degradation | Medium | Low | Optimize calculation, limit max active trailing stops per user |

---

## Open Questions

- [ ] Should risk limits differ for paper trading vs. live trading? Currently same limits. (Resolution needed by 2025-11-30)
- [ ] What is the acceptable false positive rate for risk rejections? Proposed: < 1%. (Resolution needed by 2025-11-30)
- [ ] Should we implement dynamic risk limits based on market volatility? (Resolution needed by 2025-12-05)
- [ ] How should we handle risk limits during exchange maintenance windows? (Resolution needed by 2025-12-05)
- [ ] Should correlation risk use fundamental crypto grouping or purely statistical correlation? (Resolution needed by 2025-12-10)
- [ ] What is the risk score calculation formula for production? Current is placeholder. (Resolution needed by 2025-12-15)
- [ ] Should trailing stop activation threshold vary by market regime (trending vs ranging)? (Resolution needed by 2025-12-10)
- [ ] Should we allow users to manually adjust trailing stop distance after activation? (Resolution needed by 2025-12-10)
- [ ] What is the maximum number of concurrent trailing stops per user to prevent performance issues? (Resolution needed by 2025-12-15)

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Risk Management Team | Initial version based on codebase analysis |
| 2.0 | 2025-11-22 | Risk Management Team | Added FR-RISK-007 and FR-RISK-008 for trailing stop loss functionality; Updated risk score calculation to include trailing stop benefit; Added trailing stop use cases, test requirements, and monitoring metrics; Updated performance requirements and implementation notes |

---

## Appendix

**References**:
- Binance Futures Risk Management: https://www.binance.com/en/support/faq/futures-risk
- Professional Risk Management Guidelines: https://www.investopedia.com/risk-management
- Leverage and Margin: https://www.investopedia.com/terms/l/leverage.asp
- Stop-Loss Best Practices: https://www.investopedia.com/articles/trading/09/stop-loss-strategies.asp
- Trailing Stop Loss Strategies: https://www.investopedia.com/articles/trading/08/trailing-stop-loss.asp
- Position Sizing and Risk Management: https://www.investopedia.com/articles/trading/09/position-sizing.asp

**Glossary**:
- Leverage: Multiplier for position size relative to margin (e.g., 10x leverage = control $10,000 with $1,000)
- Margin: Collateral required to open and maintain leveraged positions
- Margin Level: Health indicator = (Equity / Used Margin) × 100
- Liquidation: Forced position closure by exchange when margin insufficient
- Stop-Loss: Automatic order to close position at predetermined price to limit loss
- Trailing Stop Loss: Dynamic stop-loss that moves with favorable price movement to lock in profits
- Activation Threshold: Minimum profit percentage required before trailing stop activates
- Trailing Distance: Percentage below highest price (long) or above lowest price (short) where stop is placed
- Risk-Reward Ratio: Ratio of potential profit to potential loss (e.g., 2:1 = risk $100 to make $200)
- Daily Loss Limit: Maximum percentage of balance that can be lost in 24-hour period
- Correlation: Statistical measure of how two assets move together (-1 to +1)
- Drawdown: Peak-to-trough decline in account value
- Circuit Breaker: Automatic trading halt during extreme conditions

**Code Examples**:

```rust
// @spec:FR-RISK-005
// Pre-trade risk validation
pub async fn can_open_position(
    &self,
    symbol: &str,
    analysis: &MultiTimeframeAnalysis,
) -> Result<bool> {
    // Check if trading is enabled
    if !self.config.enabled {
        debug!("Trading is disabled");
        return Ok(false);
    }

    // Check signal confidence threshold
    let min_confidence = match analysis.overall_signal {
        TradingSignal::StrongBuy | TradingSignal::StrongSell => 0.7,
        TradingSignal::Buy | TradingSignal::Sell => 0.8,
        TradingSignal::Hold => return Ok(false),
    };

    if analysis.overall_confidence < min_confidence {
        debug!("Signal confidence {} below threshold {}",
               analysis.overall_confidence, min_confidence);
        return Ok(false);
    }

    // Check risk-reward ratio
    if let Some(risk_reward) = analysis.risk_reward_ratio {
        if risk_reward < 1.5 {
            debug!("Risk-reward ratio {} below minimum 1.5", risk_reward);
            return Ok(false);
        }
    }

    Ok(true)
}

// @spec:FR-RISK-002
// Stop-loss validation
fn validate_stop_loss(entry: f64, stop_loss: f64, side: &str) -> Result<()> {
    let distance = ((entry - stop_loss).abs() / entry) * 100.0;

    if distance > 10.0 {
        return Err(anyhow!("Stop-loss distance {}% exceeds maximum 10%", distance));
    }

    if side == "BUY" && stop_loss >= entry {
        return Err(anyhow!("Stop-loss must be below entry price for LONG positions"));
    }

    if side == "SELL" && stop_loss <= entry {
        return Err(anyhow!("Stop-loss must be above entry price for SHORT positions"));
    }

    Ok(())
}

// @spec:FR-RISK-003
// Daily loss calculation
fn calculate_daily_loss(&self) -> f64 {
    let now = Utc::now();
    let day_ago = now - chrono::Duration::hours(24);

    self.closed_trades
        .iter()
        .filter(|trade| trade.close_time.is_some_and(|t| t >= day_ago))
        .map(|trade| trade.realized_pnl.unwrap_or(0.0))
        .sum()
}

// @spec:FR-RISK-006
// Margin level monitoring
fn check_margin_level(&self) -> MarginStatus {
    let margin_level = if self.margin_used > 0.0 {
        (self.equity / self.margin_used) * 100.0
    } else {
        0.0
    };

    if margin_level < 110.0 {
        MarginStatus::Liquidation
    } else if margin_level < 150.0 {
        MarginStatus::Critical
    } else if margin_level < 200.0 {
        MarginStatus::Warning
    } else {
        MarginStatus::Healthy
    }
}

// @spec:FR-RISK-007, FR-RISK-008
// Trailing stop loss implementation
pub fn update_trailing_stop(
    &mut self,
    current_price: f64,
    activation_pct: f64,
    trailing_pct: f64,
) -> Result<()> {
    // Calculate current profit percentage
    let profit_pct = match self.trade_type {
        TradeType::Long => ((current_price - self.entry_price) / self.entry_price) * 100.0,
        TradeType::Short => ((self.entry_price - current_price) / self.entry_price) * 100.0,
    };

    // Check if we should activate trailing stop
    if !self.trailing_stop_active && profit_pct >= activation_pct {
        self.trailing_stop_active = true;
        info!("🎯 Trailing stop activated for {} at profit {:.2}%",
              self.symbol, profit_pct);
    }

    if self.trailing_stop_active {
        match self.trade_type {
            TradeType::Long => {
                // For longs: track highest price, stop below it
                let highest = self.highest_price.unwrap_or(current_price).max(current_price);
                self.highest_price = Some(highest);

                let new_stop = highest * (1.0 - trailing_pct / 100.0);

                // Stop only moves up for longs
                if let Some(current_stop) = self.stop_loss {
                    if new_stop > current_stop {
                        self.stop_loss = Some(new_stop);
                        debug!("📈 Trailing stop adjusted UP to {:.2} (highest: {:.2})",
                               new_stop, highest);
                    }
                } else {
                    self.stop_loss = Some(new_stop);
                }
            }
            TradeType::Short => {
                // For shorts: track lowest price, stop above it
                let lowest = self.lowest_price
                    .map(|l| l.min(current_price))
                    .unwrap_or(current_price);
                self.lowest_price = Some(lowest);

                let new_stop = lowest * (1.0 + trailing_pct / 100.0);

                // Stop only moves down for shorts
                if let Some(current_stop) = self.stop_loss {
                    if new_stop < current_stop {
                        self.stop_loss = Some(new_stop);
                        debug!("📉 Trailing stop adjusted DOWN to {:.2} (lowest: {:.2})",
                               new_stop, lowest);
                    }
                } else {
                    self.stop_loss = Some(new_stop);
                }
            }
        }
    }

    Ok(())
}

// @spec:FR-RISK-009
// Risk score calculation with trailing stop benefit
fn calculate_risk_score_with_trailing(
    &self,
    base_risk_score: f64,
    trailing_enabled: bool,
) -> f64 {
    if trailing_enabled {
        // Reduce risk score by 20% when trailing stop is enabled
        base_risk_score * 0.8
    } else {
        base_risk_score
    }
}
```

---

**Remember**: Update TRACEABILITY_MATRIX.md when implementation is complete!
