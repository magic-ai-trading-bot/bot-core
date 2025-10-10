# FR-STRATEGIES: Trading Strategies Functional Requirements

**Document Version:** 1.0
**Last Updated:** 2025-10-10
**Status:** Draft
**Owner:** Strategy Engine Team

---

## Table of Contents

1. [Introduction](#1-introduction)
2. [Scope](#2-scope)
3. [Functional Requirements](#3-functional-requirements)
   - [FR-STRATEGIES-001: RSI Strategy](#fr-strategies-001-rsi-strategy)
   - [FR-STRATEGIES-002: MACD Strategy](#fr-strategies-002-macd-strategy)
   - [FR-STRATEGIES-003: Bollinger Bands Strategy](#fr-strategies-003-bollinger-bands-strategy)
   - [FR-STRATEGIES-004: Volume Strategy](#fr-strategies-004-volume-strategy)
   - [FR-STRATEGIES-005: Strategy Engine](#fr-strategies-005-strategy-engine)
   - [FR-STRATEGIES-006: Technical Indicators](#fr-strategies-006-technical-indicators)
   - [FR-STRATEGIES-007: Strategy Backtesting](#fr-strategies-007-strategy-backtesting)
   - [FR-STRATEGIES-008: Strategy Configuration](#fr-strategies-008-strategy-configuration)
4. [Data Models](#4-data-models)
5. [Business Rules](#5-business-rules)
6. [Acceptance Criteria](#6-acceptance-criteria)
7. [Traceability](#7-traceability)

---

## 1. Introduction

### 1.1 Purpose
This document specifies the functional requirements for the Trading Strategies system within the cryptocurrency trading bot. It defines the behavior, configuration, and integration of technical analysis-based trading strategies including RSI, MACD, Bollinger Bands, and Volume strategies.

### 1.2 Background
The trading strategies system provides algorithmic decision-making capabilities based on technical indicators and market data analysis. Each strategy implements specific trading logic to generate buy, sell, or neutral signals with confidence scores. The Strategy Engine coordinates multiple strategies and combines their signals for robust trading decisions.

### 1.3 System Context
The strategy system is implemented in Rust (`rust-core-engine/src/strategies/`) and integrates with:
- **Market Data Service**: Consumes real-time candlestick data across multiple timeframes
- **Paper Trading Engine**: Executes simulated trades based on strategy signals
- **Live Trading Engine**: Executes real trades based on strategy signals
- **Strategy Optimizer**: Optimizes strategy parameters through backtesting

---

## 2. Scope

### 2.1 In Scope
- ☑ RSI (Relative Strength Index) Strategy implementation
- ☑ MACD (Moving Average Convergence Divergence) Strategy
- ☑ Bollinger Bands Strategy
- ☑ Volume-based Strategy
- ☑ Technical indicator calculations (RSI, MACD, BB, SMA, EMA, ATR, Stochastic)
- ☑ Strategy Engine for multi-strategy coordination
- ☑ Signal combination modes (weighted average, consensus, best confidence, conservative)
- ☑ Strategy configuration and parameter tuning
- ☑ Multi-timeframe analysis support
- ☑ Strategy validation and data requirements
- ☑ Strategy backtesting integration
- ☑ Real-time signal generation
- ☑ Strategy metadata and reasoning

### 2.2 Out of Scope
- ☐ Machine learning strategies (covered in Python AI Service)
- ☐ Sentiment analysis strategies
- ☐ Order book analysis strategies
- ☐ Social media-based strategies
- ☐ News-based trading strategies

---

## 3. Functional Requirements

### FR-STRATEGIES-001: RSI Strategy

**Priority:** CRITICAL
**Spec ID:** @spec:FR-STRATEGIES-001
**Related APIs:** POST `/api/v1/strategies/rsi/analyze`

#### 3.1.1 Description
The RSI (Relative Strength Index) Strategy identifies oversold and overbought market conditions to generate reversal trading signals. It uses momentum oscillator analysis across multiple timeframes for enhanced accuracy.

#### 3.1.2 Detailed Requirements

##### 3.1.2.1 RSI Calculation
☐ **REQ-001-001**: System shall calculate RSI using the standard Wilder's Smoothing method:

**RSI Formula:**
```rust
// Step 1: Calculate price changes
let changes: Vec<f64> = prices.windows(2)
    .map(|w| w[1] - w[0])
    .collect();

// Step 2: Separate gains and losses
let gains: Vec<f64> = changes.iter()
    .map(|&c| if c > 0.0 { c } else { 0.0 })
    .collect();

let losses: Vec<f64> = changes.iter()
    .map(|&c| if c < 0.0 { -c } else { 0.0 })
    .collect();

// Step 3: Calculate initial average gain/loss
let avg_gain = gains[0..period].iter().sum::<f64>() / period as f64;
let avg_loss = losses[0..period].iter().sum::<f64>() / period as f64;

// Step 4: Calculate smoothed averages (Wilder's method)
for i in period..gains.len() {
    avg_gain = ((avg_gain * (period - 1) as f64) + gains[i]) / period as f64;
    avg_loss = ((avg_loss * (period - 1) as f64) + losses[i]) / period as f64;

    let rs = if avg_loss == 0.0 {
        100.0
    } else {
        avg_gain / avg_loss
    };

    let rsi = 100.0 - (100.0 / (1.0 + rs));
    rsi_values.push(rsi);
}
```

☐ **REQ-001-002**: System shall use default RSI period of 14 candles (configurable)

☐ **REQ-001-003**: System shall require minimum `period + 5` candles for valid calculation

☐ **REQ-001-004**: RSI values shall always be in range [0, 100]

##### 3.1.2.2 Default Configuration
☐ **REQ-001-005**: System shall initialize RSI strategy with default parameters:
```rust
RsiStrategy {
    config: StrategyConfig {
        enabled: true,
        weight: 1.0,
        parameters: {
            "rsi_period": 14,
            "oversold_threshold": 30.0,
            "overbought_threshold": 70.0,
            "extreme_oversold": 20.0,
            "extreme_overbought": 80.0,
        }
    }
}
```

☐ **REQ-001-006**: System shall support parameter customization via configuration:
- **rsi_period**: 5 to 50 (default: 14)
- **oversold_threshold**: 20.0 to 40.0 (default: 30.0)
- **overbought_threshold**: 60.0 to 80.0 (default: 70.0)
- **extreme_oversold**: 10.0 to 25.0 (default: 20.0)
- **extreme_overbought**: 75.0 to 90.0 (default: 80.0)

##### 3.1.2.3 Multi-Timeframe Analysis
☐ **REQ-001-007**: System shall require two timeframes for analysis:
- **Primary timeframe**: 1h (for signal generation)
- **Confirmation timeframe**: 4h (for trend confirmation)

☐ **REQ-001-008**: System shall calculate RSI for both timeframes independently

☐ **REQ-001-009**: System shall validate minimum data requirements:
```rust
fn validate_data(&self, data: &StrategyInput) -> Result<(), StrategyError> {
    let min_required = self.rsi_period + 5;

    for timeframe in ["1h", "4h"] {
        let candles = data.timeframe_data.get(timeframe)?;
        if candles.len() < min_required {
            return Err(StrategyError::InsufficientData(
                format!("Need {} candles for {}, got {}",
                    min_required, timeframe, candles.len())
            ));
        }
    }
    Ok(())
}
```

##### 3.1.2.4 Signal Generation Logic
☐ **REQ-001-010**: System shall generate LONG signals when:
```rust
fn should_generate_long_signal(
    rsi_1h: f64,
    rsi_4h: f64,
    prev_rsi_1h: f64,
    oversold: f64,
    extreme_oversold: f64,
) -> bool {
    // Primary condition: 1h RSI in oversold zone
    let primary_oversold = rsi_1h < oversold;

    // Confirmation: 4h RSI also oversold or neutral
    let confirmation = rsi_4h < oversold + 10.0;

    // Momentum: RSI turning up
    let turning_up = rsi_1h > prev_rsi_1h;

    // Extreme opportunity: Very oversold condition
    let extreme_condition = rsi_1h < extreme_oversold;

    (primary_oversold && confirmation && turning_up) || extreme_condition
}
```

**Signal Conditions:**
- **Strong Long (confidence > 0.8)**:
  - 1h RSI < extreme_oversold (20) AND
  - 4h RSI < oversold (30) AND
  - RSI turning upward (current > previous)

- **Medium Long (confidence 0.5-0.8)**:
  - 1h RSI < oversold (30) AND
  - 4h RSI < 40 AND
  - RSI turning upward

- **Weak Long (confidence 0.3-0.5)**:
  - 1h RSI < oversold (30) BUT
  - 4h RSI not confirming OR
  - RSI not turning up yet

☐ **REQ-001-011**: System shall generate SHORT signals when:
```rust
fn should_generate_short_signal(
    rsi_1h: f64,
    rsi_4h: f64,
    prev_rsi_1h: f64,
    overbought: f64,
    extreme_overbought: f64,
) -> bool {
    // Primary condition: 1h RSI in overbought zone
    let primary_overbought = rsi_1h > overbought;

    // Confirmation: 4h RSI also overbought or neutral
    let confirmation = rsi_4h > overbought - 10.0;

    // Momentum: RSI turning down
    let turning_down = rsi_1h < prev_rsi_1h;

    // Extreme opportunity: Very overbought condition
    let extreme_condition = rsi_1h > extreme_overbought;

    (primary_overbought && confirmation && turning_down) || extreme_condition
}
```

**Signal Conditions:**
- **Strong Short (confidence > 0.8)**:
  - 1h RSI > extreme_overbought (80) AND
  - 4h RSI > overbought (70) AND
  - RSI turning downward

- **Medium Short (confidence 0.5-0.8)**:
  - 1h RSI > overbought (70) AND
  - 4h RSI > 60 AND
  - RSI turning downward

- **Weak Short (confidence 0.3-0.5)**:
  - 1h RSI > overbought (70) BUT
  - 4h RSI not confirming OR
  - RSI not turning down yet

☐ **REQ-001-012**: System shall generate NEUTRAL signal when:
- RSI between oversold and overbought thresholds on both timeframes
- No clear trend direction
- Conflicting signals between timeframes

##### 3.1.2.5 Confidence Calculation
☐ **REQ-001-013**: System shall calculate signal confidence based on:
```rust
fn calculate_confidence(
    rsi_1h: f64,
    rsi_4h: f64,
    prev_rsi_1h: f64,
    prev_rsi_4h: f64,
    signal_type: TradingSignal,
) -> f64 {
    let mut confidence = 0.0;

    match signal_type {
        TradingSignal::Long => {
            // Base confidence from 1h RSI depth
            let oversold_depth = (30.0 - rsi_1h) / 30.0;  // 0.0 to 1.0
            confidence += oversold_depth * 0.4;  // Max 40%

            // 4h confirmation bonus
            if rsi_4h < 35.0 {
                confidence += 0.2;  // +20%
            }

            // Momentum bonus (RSI turning up)
            let momentum_1h = rsi_1h - prev_rsi_1h;
            let momentum_4h = rsi_4h - prev_rsi_4h;
            if momentum_1h > 0.0 && momentum_4h > 0.0 {
                confidence += 0.2;  // +20%
            } else if momentum_1h > 0.0 {
                confidence += 0.1;  // +10%
            }

            // Extreme oversold bonus
            if rsi_1h < 20.0 {
                confidence += 0.2;  // +20%
            }
        },
        TradingSignal::Short => {
            // Mirror logic for short signals
            let overbought_depth = (rsi_1h - 70.0) / 30.0;
            confidence += overbought_depth * 0.4;

            if rsi_4h > 65.0 {
                confidence += 0.2;
            }

            let momentum_1h = prev_rsi_1h - rsi_1h;
            let momentum_4h = prev_rsi_4h - rsi_4h;
            if momentum_1h > 0.0 && momentum_4h > 0.0 {
                confidence += 0.2;
            } else if momentum_1h > 0.0 {
                confidence += 0.1;
            }

            if rsi_1h > 80.0 {
                confidence += 0.2;
            }
        },
        TradingSignal::Neutral => {
            confidence = 0.5;  // Neutral has medium confidence
        },
    }

    confidence.min(1.0)  // Cap at 100%
}
```

##### 3.1.2.6 Signal Output
☐ **REQ-001-014**: System shall return StrategyOutput with:
```rust
StrategyOutput {
    signal: TradingSignal,  // Long, Short, or Neutral
    confidence: f64,  // 0.0 to 1.0
    reasoning: String,  // Human-readable explanation
    timeframe: "1h".to_string(),
    timestamp: current_timestamp,
    metadata: {
        "rsi_1h": current_rsi_1h,
        "rsi_4h": current_rsi_4h,
        "prev_rsi_1h": prev_rsi_1h,
        "prev_rsi_4h": prev_rsi_4h,
        "oversold_threshold": 30.0,
        "overbought_threshold": 70.0,
    }
}
```

☐ **REQ-001-015**: System shall provide detailed reasoning:
```rust
// Example reasoning strings:
"Strong LONG: 1h RSI at 18.5 (extreme oversold), 4h RSI at 28.3 (oversold), both turning upward"
"Medium SHORT: 1h RSI at 74.2 (overbought), 4h RSI at 68.1, 1h turning downward"
"NEUTRAL: RSI at 52.3 (1h) and 55.7 (4h), no clear reversal signal"
```

##### 3.1.2.7 RSI Divergence Detection (Advanced)
☐ **REQ-001-016**: System shall detect bullish divergence:
- Price making lower lows
- RSI making higher lows
- Indicates potential reversal to upside

☐ **REQ-001-017**: System shall detect bearish divergence:
- Price making higher highs
- RSI making lower highs
- Indicates potential reversal to downside

#### 3.1.3 Business Rules
- **BR-001-001**: RSI must be between 0 and 100 (inclusive)
- **BR-001-002**: Minimum 19 candles required (14 period + 5 buffer)
- **BR-001-003**: Both timeframes must have valid RSI before signal generation
- **BR-001-004**: Confidence capped at 1.0 (100%)
- **BR-001-005**: Default to NEUTRAL when data insufficient

#### 3.1.4 Acceptance Criteria
```gherkin
Scenario: RSI generates LONG signal on oversold condition
  Given BTCUSDT 1h candles with RSI at 25.0
  And BTCUSDT 4h candles with RSI at 32.0
  And previous 1h RSI was 23.5 (turning up)
  When RSI strategy analyzes market
  Then strategy returns:
    ☐ Signal = LONG
    ☐ Confidence > 0.7
    ☐ Reasoning mentions "oversold" and "turning upward"
    ☐ Metadata includes RSI values for both timeframes
    ☐ Timeframe = "1h"
    ☐ Timestamp = current time
```

---

### FR-STRATEGIES-002: MACD Strategy

**Priority:** CRITICAL
**Spec ID:** @spec:FR-STRATEGIES-002
**Related APIs:** POST `/api/v1/strategies/macd/analyze`

#### 3.2.1 Description
The MACD (Moving Average Convergence Divergence) Strategy identifies trend changes and momentum shifts through the relationship between two exponential moving averages. It generates signals based on MACD line crossovers with the signal line and histogram analysis.

#### 3.2.2 Detailed Requirements

##### 3.2.2.1 MACD Calculation
☐ **REQ-002-001**: System shall calculate MACD using standard parameters:

**MACD Components:**
```rust
pub struct MacdResult {
    pub macd_line: Vec<f64>,      // Fast EMA - Slow EMA
    pub signal_line: Vec<f64>,    // EMA of MACD line
    pub histogram: Vec<f64>,      // MACD line - Signal line
}

// Step 1: Calculate Fast EMA (default 12-period)
let ema_fast = calculate_ema(&close_prices, fast_period)?;

// Step 2: Calculate Slow EMA (default 26-period)
let ema_slow = calculate_ema(&close_prices, slow_period)?;

// Step 3: Calculate MACD Line = Fast EMA - Slow EMA
let macd_line: Vec<f64> = ema_fast.iter()
    .zip(ema_slow.iter())
    .map(|(fast, slow)| fast - slow)
    .collect();

// Step 4: Calculate Signal Line = EMA of MACD (default 9-period)
let signal_line = calculate_ema(&macd_line, signal_period)?;

// Step 5: Calculate Histogram = MACD - Signal
let histogram: Vec<f64> = macd_line.iter()
    .zip(signal_line.iter())
    .map(|(macd, signal)| macd - signal)
    .collect();
```

**EMA Calculation (Exponential Moving Average):**
```rust
fn calculate_ema(prices: &[f64], period: usize) -> Result<Vec<f64>> {
    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema_values = Vec::new();

    // First EMA is SMA
    let first_sma = prices[0..period].iter().sum::<f64>() / period as f64;
    ema_values.push(first_sma);

    // Subsequent EMA: EMA = (Price - Previous EMA) × Multiplier + Previous EMA
    for price in prices.iter().skip(period) {
        let last_ema = *ema_values.last().unwrap();
        let ema = (price - last_ema) * multiplier + last_ema;
        ema_values.push(ema);
    }

    Ok(ema_values)
}
```

☐ **REQ-002-002**: System shall use default MACD parameters:
- **Fast Period**: 12
- **Slow Period**: 26
- **Signal Period**: 9

☐ **REQ-002-003**: System shall support parameter customization:
- **Fast Period**: 8 to 20 (default: 12)
- **Slow Period**: 20 to 40 (default: 26)
- **Signal Period**: 5 to 15 (default: 9)
- **Histogram Threshold**: 0.0001 to 0.01 (default: 0.001)

##### 3.2.2.2 Default Configuration
☐ **REQ-002-004**: System shall initialize MACD strategy with:
```rust
MacdStrategy {
    config: StrategyConfig {
        enabled: true,
        weight: 1.0,
        parameters: {
            "fast_period": 12,
            "slow_period": 26,
            "signal_period": 9,
            "histogram_threshold": 0.001,
        }
    }
}
```

##### 3.2.2.3 Multi-Timeframe Analysis
☐ **REQ-002-005**: System shall require two timeframes:
- **Primary timeframe**: 1h (for signal generation)
- **Confirmation timeframe**: 4h (for trend confirmation)

☐ **REQ-002-006**: System shall validate minimum data:
```rust
let min_required = slow_period + signal_period;  // 26 + 9 = 35 candles minimum
```

##### 3.2.2.4 Signal Generation Logic
☐ **REQ-002-007**: System shall detect MACD crossovers:

**Bullish Crossover (LONG signal):**
```rust
fn is_bullish_crossover(
    current_macd: f64,
    current_signal: f64,
    prev_macd: f64,
    prev_signal: f64,
) -> bool {
    // MACD crosses above signal line
    (prev_macd <= prev_signal) && (current_macd > current_signal)
}
```

**Bearish Crossover (SHORT signal):**
```rust
fn is_bearish_crossover(
    current_macd: f64,
    current_signal: f64,
    prev_macd: f64,
    prev_signal: f64,
) -> bool {
    // MACD crosses below signal line
    (prev_macd >= prev_signal) && (current_macd < current_signal)
}
```

☐ **REQ-002-008**: System shall analyze histogram momentum:
```rust
fn analyze_histogram_momentum(
    current_histogram: f64,
    prev_histogram: f64,
) -> HistogramMomentum {
    if current_histogram > 0.0 {
        if current_histogram > prev_histogram {
            HistogramMomentum::BullishStrengthening
        } else {
            HistogramMomentum::BullishWeakening
        }
    } else if current_histogram < 0.0 {
        if current_histogram < prev_histogram {
            HistogramMomentum::BearishStrengthening
        } else {
            HistogramMomentum::BearishWeakening
        }
    } else {
        HistogramMomentum::Neutral
    }
}
```

☐ **REQ-002-009**: System shall generate LONG signals when:

**Strong Long (confidence > 0.8):**
- Bullish crossover on 1h timeframe AND
- MACD line above zero (uptrend) AND
- 4h histogram positive and growing AND
- Histogram above threshold

**Medium Long (confidence 0.5-0.8):**
- Bullish crossover on 1h timeframe AND
- 4h MACD also positive OR
- 1h histogram growing

**Weak Long (confidence 0.3-0.5):**
- Bullish crossover on 1h BUT
- 4h showing conflicting signal OR
- Histogram barely above threshold

☐ **REQ-002-010**: System shall generate SHORT signals when:

**Strong Short (confidence > 0.8):**
- Bearish crossover on 1h timeframe AND
- MACD line below zero (downtrend) AND
- 4h histogram negative and declining AND
- Histogram magnitude above threshold

**Medium Short (confidence 0.5-0.8):**
- Bearish crossover on 1h timeframe AND
- 4h MACD also negative OR
- 1h histogram declining

**Weak Short (confidence 0.3-0.5):**
- Bearish crossover on 1h BUT
- 4h showing conflicting signal OR
- Histogram barely below threshold

##### 3.2.2.5 Confidence Calculation
☐ **REQ-002-011**: System shall calculate confidence:
```rust
fn calculate_macd_confidence(
    macd_1h: f64,
    signal_1h: f64,
    histogram_1h: f64,
    histogram_4h: f64,
    prev_histogram_1h: f64,
    prev_histogram_4h: f64,
    signal_type: TradingSignal,
) -> f64 {
    let mut confidence = 0.0;

    match signal_type {
        TradingSignal::Long => {
            // Base: Crossover strength (histogram magnitude)
            let crossover_strength = (histogram_1h / macd_1h.abs()).min(1.0);
            confidence += crossover_strength * 0.3;  // Max 30%

            // MACD position bonus (above zero line)
            if macd_1h > 0.0 {
                confidence += 0.2;  // +20%
            }

            // 4h confirmation
            if histogram_4h > 0.0 && histogram_4h > prev_histogram_4h {
                confidence += 0.3;  // +30%
            } else if histogram_4h > 0.0 {
                confidence += 0.15;  // +15%
            }

            // Momentum (histogram growing)
            if histogram_1h > prev_histogram_1h {
                confidence += 0.2;  // +20%
            }
        },
        TradingSignal::Short => {
            // Mirror logic for short
            let crossover_strength = (histogram_1h.abs() / macd_1h.abs()).min(1.0);
            confidence += crossover_strength * 0.3;

            if macd_1h < 0.0 {
                confidence += 0.2;
            }

            if histogram_4h < 0.0 && histogram_4h < prev_histogram_4h {
                confidence += 0.3;
            } else if histogram_4h < 0.0 {
                confidence += 0.15;
            }

            if histogram_1h < prev_histogram_1h {
                confidence += 0.2;
            }
        },
        TradingSignal::Neutral => {
            confidence = 0.5;
        },
    }

    confidence.min(1.0)
}
```

##### 3.2.2.6 MACD Divergence Detection
☐ **REQ-002-012**: System shall detect bullish divergence:
- Price making lower lows
- MACD histogram making higher lows
- Strong reversal signal

☐ **REQ-002-013**: System shall detect bearish divergence:
- Price making higher highs
- MACD histogram making lower highs
- Strong reversal signal

##### 3.2.2.7 Signal Output
☐ **REQ-002-014**: System shall return StrategyOutput with metadata:
```rust
metadata: {
    "macd_line_1h": current_macd_1h,
    "signal_line_1h": current_signal_1h,
    "histogram_1h": current_histogram_1h,
    "macd_line_4h": current_macd_4h,
    "signal_line_4h": current_signal_4h,
    "histogram_4h": current_histogram_4h,
    "prev_histogram_1h": prev_histogram_1h,
    "prev_histogram_4h": prev_histogram_4h,
    "crossover_type": "bullish" | "bearish" | "none",
}
```

#### 3.2.3 Business Rules
- **BR-002-001**: MACD requires minimum 35 candles (26 + 9)
- **BR-002-002**: Crossover valid only when histogram crosses zero
- **BR-002-003**: Both timeframes must have valid MACD
- **BR-002-004**: Histogram threshold prevents noise trading

#### 3.2.4 Acceptance Criteria
```gherkin
Scenario: MACD generates LONG on bullish crossover
  Given BTCUSDT 1h data with:
    - Previous MACD: 45.2, Signal: 46.1 (MACD below signal)
    - Current MACD: 47.8, Signal: 46.5 (MACD above signal)
    - Current histogram: 1.3 (positive and growing)
  And BTCUSDT 4h histogram: 2.5 (positive)
  When MACD strategy analyzes market
  Then strategy returns:
    ☐ Signal = LONG
    ☐ Confidence > 0.7
    ☐ Reasoning mentions "bullish crossover"
    ☐ Metadata includes MACD, signal, histogram for both timeframes
```

---

### FR-STRATEGIES-003: Bollinger Bands Strategy

**Priority:** HIGH
**Spec ID:** @spec:FR-STRATEGIES-003
**Related APIs:** POST `/api/v1/strategies/bollinger/analyze`

#### 3.3.1 Description
The Bollinger Bands Strategy identifies volatility expansion/contraction and mean reversion opportunities. It uses price position relative to dynamic support/resistance bands to generate trading signals.

#### 3.3.2 Detailed Requirements

##### 3.3.2.1 Bollinger Bands Calculation
☐ **REQ-003-001**: System shall calculate Bollinger Bands:

**BB Formula:**
```rust
pub struct BollingerBands {
    pub upper: Vec<f64>,    // SMA + (std_dev × multiplier)
    pub middle: Vec<f64>,   // Simple Moving Average
    pub lower: Vec<f64>,    // SMA - (std_dev × multiplier)
}

// Step 1: Calculate SMA (Simple Moving Average)
let sma = calculate_sma(&close_prices, period)?;

// Step 2: Calculate Standard Deviation for each window
for (i, &mean) in sma.iter().enumerate() {
    let start = i;
    let end = i + period;
    let window = &close_prices[start..end];

    // Variance = average of squared differences from mean
    let variance = window.iter()
        .map(|&price| (price - mean).powi(2))
        .sum::<f64>() / period as f64;

    let std_dev = variance.sqrt();

    // Step 3: Calculate bands
    upper.push(mean + (multiplier × std_dev));
    middle.push(mean);
    lower.push(mean - (multiplier × std_dev));
}
```

**SMA Calculation:**
```rust
fn calculate_sma(prices: &[f64], period: usize) -> Result<Vec<f64>> {
    let mut sma_values = Vec::new();

    for i in 0..=prices.len() - period {
        let sum: f64 = prices[i..i + period].iter().sum();
        sma_values.push(sum / period as f64);
    }

    Ok(sma_values)
}
```

☐ **REQ-003-002**: System shall use default BB parameters:
- **Period**: 20 candles
- **Multiplier**: 2.0 standard deviations
- **Squeeze Threshold**: 0.02 (2% band width)

☐ **REQ-003-003**: System shall support parameter customization:
- **BB Period**: 10 to 50 (default: 20)
- **BB Multiplier**: 1.5 to 3.0 (default: 2.0)
- **Squeeze Threshold**: 0.01 to 0.05 (default: 0.02)

##### 3.3.2.2 Default Configuration
☐ **REQ-003-004**: System shall initialize with:
```rust
BollingerStrategy {
    config: StrategyConfig {
        enabled: true,
        weight: 1.0,
        parameters: {
            "bb_period": 20,
            "bb_multiplier": 2.0,
            "squeeze_threshold": 0.02,
        }
    }
}
```

##### 3.3.2.3 Band Width and Position Calculations
☐ **REQ-003-005**: System shall calculate band width (normalized):
```rust
fn calculate_band_width(
    upper_band: f64,
    lower_band: f64,
    middle_band: f64,
) -> f64 {
    // Normalized by middle band for percentage comparison
    (upper_band - lower_band) / middle_band
}
```

☐ **REQ-003-006**: System shall calculate price position within bands:
```rust
fn calculate_bb_position(
    current_price: f64,
    upper_band: f64,
    lower_band: f64,
) -> f64 {
    // Returns 0.0 to 1.0 (0 = at lower band, 1 = at upper band)
    (current_price - lower_band) / (upper_band - lower_band)
}
```

☐ **REQ-003-007**: System shall detect band squeeze:
```rust
fn is_band_squeeze(
    band_width: f64,
    squeeze_threshold: f64,
) -> bool {
    band_width < squeeze_threshold  // Volatility contraction
}
```

☐ **REQ-003-008**: System shall detect band expansion:
```rust
fn is_band_expanding(
    current_width: f64,
    previous_width: f64,
) -> bool {
    current_width > previous_width * 1.05  // 5% increase
}
```

##### 3.3.2.4 Signal Generation Logic
☐ **REQ-003-009**: System shall generate LONG signals when:

**Strong Long (confidence > 0.8):**
- Price touches or breaks below lower band (BB position < 0.1) AND
- Band squeeze detected (low volatility) AND
- Bands starting to expand (volatility breakout) AND
- 4h price also near lower band

**Medium Long (confidence 0.5-0.8):**
- Price near lower band (BB position < 0.2) AND
- Normal volatility (no squeeze) AND
- 4h trending up or neutral

**Weak Long (confidence 0.3-0.5):**
- Price approaching lower band (BB position < 0.3) BUT
- High volatility OR
- 4h conflicting signal

☐ **REQ-003-010**: System shall generate SHORT signals when:

**Strong Short (confidence > 0.8):**
- Price touches or breaks above upper band (BB position > 0.9) AND
- Band squeeze detected AND
- Bands starting to expand AND
- 4h price also near upper band

**Medium Short (confidence 0.5-0.8):**
- Price near upper band (BB position > 0.8) AND
- Normal volatility AND
- 4h trending down or neutral

**Weak Short (confidence 0.3-0.5):**
- Price approaching upper band (BB position > 0.7) BUT
- High volatility OR
- 4h conflicting

##### 3.3.2.5 Bollinger Band Squeeze Strategy
☐ **REQ-003-011**: System shall detect squeeze breakout:
```rust
fn detect_squeeze_breakout(
    current_price: f64,
    middle_band: f64,
    is_squeeze: bool,
    band_expanding: bool,
) -> Option<TradingSignal> {
    if !is_squeeze || !band_expanding {
        return None;
    }

    // Breakout direction
    if current_price > middle_band {
        Some(TradingSignal::Long)  // Bullish breakout
    } else {
        Some(TradingSignal::Short)  // Bearish breakout
    }
}
```

☐ **REQ-003-012**: System shall increase confidence for squeeze breakouts:
```rust
if is_squeeze && band_expanding {
    confidence += 0.2;  // Squeeze breakout bonus
}
```

##### 3.3.2.6 Mean Reversion Logic
☐ **REQ-003-013**: System shall implement mean reversion:
```rust
fn calculate_mean_reversion_potential(
    bb_position: f64,
    band_width: f64,
) -> f64 {
    // Higher reversion potential when:
    // 1. Price far from middle band (extreme position)
    // 2. Bands not expanding (stable volatility)

    let distance_from_middle = (bb_position - 0.5).abs() * 2.0;  // 0 to 1
    let volatility_factor = 1.0 - band_width.min(0.1) / 0.1;  // Lower is better

    (distance_from_middle + volatility_factor) / 2.0
}
```

##### 3.3.2.7 Confidence Calculation
☐ **REQ-003-014**: System shall calculate confidence:
```rust
fn calculate_bb_confidence(
    bb_position_1h: f64,
    bb_position_4h: f64,
    band_width_1h: f64,
    is_squeeze: bool,
    is_expanding: bool,
    signal_type: TradingSignal,
) -> f64 {
    let mut confidence = 0.0;

    match signal_type {
        TradingSignal::Long => {
            // How far below lower band (0.0 = at lower, negative = below)
            let band_touch = (0.1 - bb_position_1h).max(0.0);
            confidence += (band_touch / 0.1) * 0.4;  // Max 40%

            // 4h confirmation
            if bb_position_4h < 0.3 {
                confidence += 0.2;  // +20%
            }

            // Squeeze breakout bonus
            if is_squeeze && is_expanding {
                confidence += 0.3;  // +30%
            } else if is_squeeze {
                confidence += 0.1;  // +10% for squeeze
            }

            // Mean reversion potential
            if band_width_1h < 0.03 {  // Tight bands
                confidence += 0.1;  // +10%
            }
        },
        TradingSignal::Short => {
            // Mirror logic for short
            let band_touch = (bb_position_1h - 0.9).max(0.0);
            confidence += (band_touch / 0.1) * 0.4;

            if bb_position_4h > 0.7 {
                confidence += 0.2;
            }

            if is_squeeze && is_expanding {
                confidence += 0.3;
            } else if is_squeeze {
                confidence += 0.1;
            }

            if band_width_1h < 0.03 {
                confidence += 0.1;
            }
        },
        TradingSignal::Neutral => {
            confidence = 0.5;
        },
    }

    confidence.min(1.0)
}
```

##### 3.3.2.8 Signal Output
☐ **REQ-003-015**: System shall return metadata:
```rust
metadata: {
    "bb_upper_1h": upper_1h,
    "bb_middle_1h": middle_1h,
    "bb_lower_1h": lower_1h,
    "bb_position_1h": bb_position_1h,  // 0.0 to 1.0
    "bb_position_4h": bb_position_4h,
    "bb_width_1h": bb_width_1h,
    "bb_width_4h": bb_width_4h,
    "is_squeeze_1h": is_squeeze,
    "is_squeeze_4h": is_squeeze_4h,
    "bb_expanding": is_expanding,
}
```

#### 3.3.3 Business Rules
- **BR-003-001**: BB requires minimum 20 candles (default period)
- **BR-003-002**: Squeeze defined as band width < 2% (configurable)
- **BR-003-003**: Band expansion requires 5% width increase
- **BR-003-004**: Mean reversion stronger in low volatility

#### 3.3.4 Acceptance Criteria
```gherkin
Scenario: BB generates LONG on lower band touch during squeeze
  Given BTCUSDT 1h data with:
    - Current price: $49,500
    - Upper band: $51,000
    - Middle band: $50,000
    - Lower band: $49,000
    - BB position: 0.05 (touching lower band)
    - Band width: 0.015 (squeeze condition)
    - Bands expanding from previous candle
  When Bollinger strategy analyzes market
  Then strategy returns:
    ☐ Signal = LONG
    ☐ Confidence > 0.8 (squeeze breakout bonus)
    ☐ Reasoning mentions "lower band touch" and "squeeze breakout"
    ☐ Metadata shows squeeze = true, expanding = true
```

---

### FR-STRATEGIES-004: Volume Strategy

**Priority:** MEDIUM
**Spec ID:** @spec:FR-STRATEGIES-004
**Related APIs:** POST `/api/v1/strategies/volume/analyze`

#### 3.4.1 Description
The Volume Strategy analyzes trading volume patterns to identify accumulation/distribution phases, volume spikes, and volume-price divergence for confirming trend strength and potential reversals.

#### 3.4.2 Detailed Requirements

##### 3.4.2.1 Volume Indicators
☐ **REQ-004-001**: System shall calculate volume moving average:
```rust
// Volume SMA - average volume over period
let volumes: Vec<f64> = candles.iter().map(|c| c.volume).collect();
let volume_sma = calculate_sma(&volumes, period)?;
```

☐ **REQ-004-002**: System shall calculate volume ratio:
```rust
fn calculate_volume_ratio(
    current_volume: f64,
    average_volume: f64,
) -> f64 {
    if average_volume == 0.0 {
        return 1.0;
    }
    current_volume / average_volume
}
```

☐ **REQ-004-003**: System shall calculate volume profile:
```rust
pub struct VolumeProfile {
    pub price_levels: Vec<f64>,
    pub volumes: Vec<f64>,
    pub poc: f64,  // Point of Control (highest volume price)
}

fn calculate_volume_profile(
    candles: &[CandleData],
    num_levels: usize,
) -> Result<VolumeProfile> {
    // Find price range
    let min_price = candles.iter().map(|c| c.low).min();
    let max_price = candles.iter().map(|c| c.high).max();
    let price_step = (max_price - min_price) / num_levels as f64;

    // Create price buckets
    let mut volumes = vec![0.0; num_levels];

    // Distribute volume across price levels
    for candle in candles {
        let avg_price = (candle.high + candle.low + candle.close) / 3.0;
        let level_index = ((avg_price - min_price) / price_step) as usize;
        let level_index = level_index.min(num_levels - 1);
        volumes[level_index] += candle.volume;
    }

    // Find Point of Control (max volume level)
    let max_volume_idx = volumes.iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap())
        .map(|(idx, _)| idx)
        .unwrap_or(0);

    let poc = min_price + (max_volume_idx as f64 * price_step);

    Ok(VolumeProfile {
        price_levels: (0..num_levels)
            .map(|i| min_price + (i as f64 * price_step))
            .collect(),
        volumes,
        poc,
    })
}
```

##### 3.4.2.2 Default Configuration
☐ **REQ-004-004**: System shall initialize with:
```rust
VolumeStrategy {
    config: StrategyConfig {
        enabled: true,
        weight: 0.8,  // Lower weight than price-based strategies
        parameters: {
            "volume_sma_period": 20,
            "volume_spike_threshold": 2.0,  // 2x average volume
            "price_volume_correlation_period": 10,
        }
    }
}
```

##### 3.4.2.3 Volume Spike Detection
☐ **REQ-004-005**: System shall detect volume spikes:
```rust
fn is_volume_spike(
    current_volume: f64,
    average_volume: f64,
    spike_threshold: f64,
) -> bool {
    current_volume >= average_volume * spike_threshold
}
```

☐ **REQ-004-006**: System shall classify spike intensity:
```rust
enum SpikeIntensity {
    None,            // < 1.5x average
    Moderate,        // 1.5x to 2.0x
    Strong,          // 2.0x to 3.0x
    Extreme,         // > 3.0x
}

fn classify_spike_intensity(volume_ratio: f64) -> SpikeIntensity {
    match volume_ratio {
        r if r >= 3.0 => SpikeIntensity::Extreme,
        r if r >= 2.0 => SpikeIntensity::Strong,
        r if r >= 1.5 => SpikeIntensity::Moderate,
        _ => SpikeIntensity::None,
    }
}
```

##### 3.4.2.4 Accumulation/Distribution Analysis
☐ **REQ-004-007**: System shall detect accumulation (buying pressure):
```rust
fn detect_accumulation(
    recent_candles: &[CandleData],
    recent_volumes: &[f64],
) -> f64 {
    let mut accumulation_score = 0.0;

    for (candle, &volume) in recent_candles.iter().zip(recent_volumes) {
        let price_change = candle.close - candle.open;

        if price_change > 0.0 {
            // Bullish candle with high volume = accumulation
            let normalized_volume = volume / recent_volumes.iter().sum::<f64>();
            accumulation_score += normalized_volume;
        } else if price_change < 0.0 {
            // Bearish candle with high volume = distribution
            let normalized_volume = volume / recent_volumes.iter().sum::<f64>();
            accumulation_score -= normalized_volume;
        }
    }

    accumulation_score  // Positive = accumulation, Negative = distribution
}
```

☐ **REQ-004-008**: System shall calculate volume-weighted price momentum:
```rust
fn calculate_volume_weighted_momentum(
    recent_candles: &[CandleData],
    recent_volumes: &[f64],
) -> f64 {
    let mut bullish_volume = 0.0;
    let mut bearish_volume = 0.0;

    for (candle, &volume) in recent_candles.iter().zip(recent_volumes) {
        let price_change = candle.close - candle.open;

        if price_change > 0.0 {
            bullish_volume += volume;
        } else if price_change < 0.0 {
            bearish_volume += volume;
        }
    }

    let total_volume = bullish_volume + bearish_volume;
    if total_volume == 0.0 {
        return 0.0;
    }

    (bullish_volume - bearish_volume) / total_volume  // -1.0 to 1.0
}
```

##### 3.4.2.5 Volume-Price Divergence
☐ **REQ-004-009**: System shall detect bullish volume divergence:
- Price making lower lows
- Volume decreasing on downward moves (selling exhaustion)
- Volume increasing on upward moves (buying interest)
- Signals potential reversal to upside

☐ **REQ-004-010**: System shall detect bearish volume divergence:
- Price making higher highs
- Volume decreasing on upward moves (buying exhaustion)
- Volume increasing on downward moves (selling pressure)
- Signals potential reversal to downside

##### 3.4.2.6 Point of Control (POC) Analysis
☐ **REQ-004-011**: System shall use POC as support/resistance:
```rust
fn analyze_poc_relationship(
    current_price: f64,
    poc: f64,
) -> PocRelationship {
    let distance_pct = ((current_price - poc) / current_price).abs();

    if distance_pct < 0.01 {  // Within 1% of POC
        PocRelationship::AtPoc  // Strong support/resistance
    } else if current_price > poc {
        PocRelationship::AbovePoc  // Price above high-volume zone
    } else {
        PocRelationship::BelowPoc  // Price below high-volume zone
    }
}
```

##### 3.4.2.7 Signal Generation Logic
☐ **REQ-004-012**: System shall generate LONG signals when:

**Strong Long (confidence > 0.7):**
- Extreme volume spike (>3x average) on bullish candle AND
- Accumulation detected over last 10 periods AND
- Price breaking above POC with high volume AND
- Bullish volume divergence detected

**Medium Long (confidence 0.5-0.7):**
- Strong volume spike (>2x average) AND
- Volume-weighted momentum > 0.5 (bullish) AND
- Price near or above POC

**Weak Long (confidence 0.3-0.5):**
- Moderate volume spike AND
- Some accumulation detected BUT
- No clear divergence

☐ **REQ-004-013**: System shall generate SHORT signals when:

**Strong Short (confidence > 0.7):**
- Extreme volume spike on bearish candle AND
- Distribution detected AND
- Price breaking below POC with high volume AND
- Bearish volume divergence

**Medium Short (confidence 0.5-0.7):**
- Strong volume spike AND
- Volume-weighted momentum < -0.5 (bearish) AND
- Price near or below POC

**Weak Short (confidence 0.3-0.5):**
- Moderate volume spike AND
- Some distribution BUT
- No clear divergence

##### 3.4.2.8 Confidence Calculation
☐ **REQ-004-014**: System shall calculate confidence:
```rust
fn calculate_volume_confidence(
    volume_ratio: f64,
    accumulation_score: f64,
    vw_momentum: f64,
    poc_distance: f64,
    spike_threshold: f64,
    signal_type: TradingSignal,
) -> f64 {
    let mut confidence = 0.0;

    match signal_type {
        TradingSignal::Long => {
            // Volume spike intensity
            if volume_ratio >= spike_threshold * 1.5 {
                confidence += 0.3;  // Extreme spike
            } else if volume_ratio >= spike_threshold {
                confidence += 0.2;  // Strong spike
            } else if volume_ratio >= spike_threshold * 0.75 {
                confidence += 0.1;  // Moderate spike
            }

            // Accumulation strength
            if accumulation_score > 0.5 {
                confidence += 0.3;
            } else if accumulation_score > 0.2 {
                confidence += 0.15;
            }

            // Volume-weighted momentum
            if vw_momentum > 0.5 {
                confidence += 0.2;
            } else if vw_momentum > 0.0 {
                confidence += 0.1;
            }

            // POC proximity bonus
            if poc_distance < 0.01 {
                confidence += 0.2;  // Near POC support
            }
        },
        TradingSignal::Short => {
            // Mirror logic for short
            if volume_ratio >= spike_threshold * 1.5 {
                confidence += 0.3;
            } else if volume_ratio >= spike_threshold {
                confidence += 0.2;
            } else if volume_ratio >= spike_threshold * 0.75 {
                confidence += 0.1;
            }

            if accumulation_score < -0.5 {
                confidence += 0.3;
            } else if accumulation_score < -0.2 {
                confidence += 0.15;
            }

            if vw_momentum < -0.5 {
                confidence += 0.2;
            } else if vw_momentum < 0.0 {
                confidence += 0.1;
            }

            if poc_distance < 0.01 {
                confidence += 0.2;
            }
        },
        TradingSignal::Neutral => {
            confidence = 0.5;
        },
    }

    confidence.min(1.0)
}
```

##### 3.4.2.9 Signal Output
☐ **REQ-004-015**: System shall return metadata:
```rust
metadata: {
    "current_volume": current_volume,
    "avg_volume": avg_volume,
    "volume_ratio": volume_ratio,
    "volume_spike": is_volume_spike,
    "spike_intensity": "None" | "Moderate" | "Strong" | "Extreme",
    "accumulation_score": accumulation_score,
    "vw_momentum": vw_momentum,
    "poc": poc,
    "poc_distance": poc_distance,
}
```

#### 3.4.3 Business Rules
- **BR-004-001**: Volume strategy confirms price-based signals
- **BR-004-002**: Volume spike threshold default 2.0x average
- **BR-004-003**: Accumulation/distribution requires min 10 periods
- **BR-004-004**: POC calculated over last 20 periods (configurable)
- **BR-004-005**: Volume strategy has lower weight (0.8) than price strategies

#### 3.4.4 Acceptance Criteria
```gherkin
Scenario: Volume strategy detects accumulation
  Given BTCUSDT 1h data with:
    - Current volume: 5,000 BTC (2.5x average)
    - Average volume: 2,000 BTC
    - Last 10 candles showing net buying (accumulation score: 0.6)
    - Volume-weighted momentum: 0.7 (bullish)
    - Current price near POC (0.5% distance)
  When Volume strategy analyzes market
  Then strategy returns:
    ☐ Signal = LONG
    ☐ Confidence > 0.7
    ☐ Reasoning mentions "volume spike" and "accumulation"
    ☐ Metadata shows spike_intensity = "Strong"
```

---

### FR-STRATEGIES-005: Strategy Engine

**Priority:** CRITICAL
**Spec ID:** @spec:FR-STRATEGIES-005
**Related APIs:** POST `/api/v1/strategies/analyze`, GET `/api/v1/strategies/signals`

#### 3.5.1 Description
The Strategy Engine coordinates multiple trading strategies, combines their signals using configurable combination modes, and produces a unified trading decision with aggregated confidence.

#### 3.5.2 Detailed Requirements

##### 3.5.2.1 Strategy Registration
☐ **REQ-005-001**: System shall maintain registry of available strategies:
```rust
pub struct StrategyEngine {
    strategies: Vec<Box<dyn Strategy>>,
    config: StrategyEngineConfig,
    signal_history: Arc<RwLock<Vec<CombinedSignal>>>,
}
```

☐ **REQ-005-002**: System shall support dynamic strategy addition/removal:
```rust
pub fn add_strategy(&mut self, strategy: Box<dyn Strategy>) {
    self.strategies.push(strategy);
}

pub fn remove_strategy(&mut self, name: &str) {
    self.strategies.retain(|s| s.name() != name);
}
```

☐ **REQ-005-003**: System shall initialize with default strategies:
```rust
impl StrategyEngine {
    pub fn new() -> Self {
        let mut engine = Self::with_config(StrategyEngineConfig::default());

        // Register default strategies
        engine.add_strategy(Box::new(RsiStrategy::new()));
        engine.add_strategy(Box::new(MacdStrategy::new()));
        engine.add_strategy(Box::new(BollingerStrategy::new()));
        engine.add_strategy(Box::new(VolumeStrategy::new()));

        engine
    }
}
```

##### 3.5.2.2 Engine Configuration
☐ **REQ-005-004**: System shall support engine configuration:
```rust
pub struct StrategyEngineConfig {
    pub enabled_strategies: Vec<String>,  // Empty = all enabled
    pub min_confidence_threshold: f64,    // Minimum to act on signal
    pub signal_combination_mode: SignalCombinationMode,
    pub max_history_size: usize,
}

impl Default for StrategyEngineConfig {
    fn default() -> Self {
        Self {
            enabled_strategies: vec![],  // All enabled
            min_confidence_threshold: 0.6,
            signal_combination_mode: SignalCombinationMode::WeightedAverage,
            max_history_size: 1000,
        }
    }
}
```

##### 3.5.2.3 Signal Combination Modes
☐ **REQ-005-005**: System shall support multiple combination modes:
```rust
pub enum SignalCombinationMode {
    WeightedAverage,   // Combine using strategy weights
    Consensus,         // Majority vote
    BestConfidence,    // Use highest confidence signal
    Conservative,      // Require all strategies to agree
}
```

**Weighted Average Mode:**
☐ **REQ-005-006**: System shall calculate weighted average:
```rust
fn combine_weighted_average(
    &self,
    results: &[StrategySignalResult],
) -> (TradingSignal, f64, String) {
    let mut long_score = 0.0;
    let mut short_score = 0.0;
    let mut neutral_score = 0.0;
    let mut total_weight = 0.0;

    for result in results {
        let weighted_confidence = result.confidence * result.weight;
        total_weight += result.weight;

        match result.signal {
            TradingSignal::Long => long_score += weighted_confidence,
            TradingSignal::Short => short_score += weighted_confidence,
            TradingSignal::Neutral => neutral_score += weighted_confidence,
        }
    }

    // Final signal = highest weighted score
    let final_signal = if long_score > short_score && long_score > neutral_score {
        TradingSignal::Long
    } else if short_score > long_score && short_score > neutral_score {
        TradingSignal::Short
    } else {
        TradingSignal::Neutral
    };

    let avg_confidence = if total_weight > 0.0 {
        match final_signal {
            TradingSignal::Long => long_score / total_weight,
            TradingSignal::Short => short_score / total_weight,
            TradingSignal::Neutral => neutral_score / total_weight,
        }
    } else {
        0.0
    };

    let reasoning = format!(
        "Weighted average: Long={:.2}, Short={:.2}, Neutral={:.2}",
        long_score, short_score, neutral_score
    );

    (final_signal, avg_confidence, reasoning)
}
```

**Consensus Mode:**
☐ **REQ-005-007**: System shall implement majority vote:
```rust
fn combine_consensus(
    &self,
    results: &[StrategySignalResult],
) -> (TradingSignal, f64, String) {
    let long_count = results.iter()
        .filter(|r| r.signal == TradingSignal::Long)
        .count();

    let short_count = results.iter()
        .filter(|r| r.signal == TradingSignal::Short)
        .count();

    let total_count = results.len();
    let majority_threshold = total_count / 2;

    let final_signal = if long_count > majority_threshold {
        TradingSignal::Long
    } else if short_count > majority_threshold {
        TradingSignal::Short
    } else {
        TradingSignal::Neutral  // No clear majority
    };

    // Confidence = consensus strength
    let max_count = long_count.max(short_count);
    let consensus_strength = max_count as f64 / total_count as f64;

    // Average confidence of agreeing strategies
    let agreeing_confidences: Vec<f64> = results.iter()
        .filter(|r| r.signal == final_signal)
        .map(|r| r.confidence)
        .collect();

    let avg_confidence = if !agreeing_confidences.is_empty() {
        agreeing_confidences.iter().sum::<f64>() / agreeing_confidences.len() as f64
    } else {
        0.5
    };

    let combined_confidence = consensus_strength * avg_confidence;

    let reasoning = format!(
        "Consensus: {} of {} strategies agree ({})",
        max_count, total_count, final_signal
    );

    (final_signal, combined_confidence, reasoning)
}
```

**Best Confidence Mode:**
☐ **REQ-005-008**: System shall use highest confidence signal:
```rust
fn combine_best_confidence(
    &self,
    results: &[StrategySignalResult],
) -> (TradingSignal, f64, String) {
    let best_result = results.iter()
        .max_by(|a, b| {
            a.confidence.partial_cmp(&b.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .unwrap();

    let reasoning = format!(
        "Best confidence: {} from '{}' with {:.2}% confidence",
        best_result.signal,
        best_result.strategy_name,
        best_result.confidence * 100.0
    );

    (
        best_result.signal.clone(),
        best_result.confidence,
        reasoning
    )
}
```

**Conservative Mode:**
☐ **REQ-005-009**: System shall require unanimous agreement:
```rust
fn combine_conservative(
    &self,
    results: &[StrategySignalResult],
) -> (TradingSignal, f64, String) {
    // Check if all strategies agree
    let first_signal = &results[0].signal;
    let all_agree = results.iter().all(|r| &r.signal == first_signal);

    if all_agree && first_signal != &TradingSignal::Neutral {
        let avg_confidence = results.iter()
            .map(|r| r.confidence)
            .sum::<f64>() / results.len() as f64;

        let reasoning = format!(
            "Conservative: All {} strategies agree on {}",
            results.len(),
            first_signal
        );

        (first_signal.clone(), avg_confidence, reasoning)
    } else {
        // No agreement = neutral
        let reasoning = format!(
            "Conservative: No unanimous agreement, staying neutral"
        );

        (TradingSignal::Neutral, 0.5, reasoning)
    }
}
```

##### 3.5.2.4 Market Analysis Workflow
☐ **REQ-005-010**: System shall execute analysis workflow:
```rust
pub async fn analyze_market(
    &self,
    data: &StrategyInput,
) -> Result<CombinedSignal, StrategyError> {
    let mut strategy_results = Vec::new();

    // Step 1: Execute all enabled strategies
    for strategy in &self.strategies {
        let strategy_name = strategy.name();

        // Check if enabled
        if !self.config.enabled_strategies.is_empty()
            && !self.config.enabled_strategies.contains(&strategy_name.to_string())
        {
            continue;  // Skip disabled strategy
        }

        // Validate data requirements
        if let Err(e) = strategy.validate_data(data) {
            warn!("Strategy '{}' validation failed: {}", strategy_name, e);
            continue;
        }

        // Execute strategy analysis
        match strategy.analyze(data).await {
            Ok(output) => {
                let weight = strategy.config().weight;
                strategy_results.push(StrategySignalResult {
                    strategy_name: strategy_name.to_string(),
                    signal: output.signal,
                    confidence: output.confidence,
                    reasoning: output.reasoning,
                    weight,
                    metadata: output.metadata,
                });

                info!(
                    "Strategy '{}' signal: {:?} (confidence: {:.2})",
                    strategy_name, output.signal, output.confidence
                );
            },
            Err(e) => {
                warn!("Strategy '{}' analysis failed: {}", strategy_name, e);
                continue;
            },
        }
    }

    // Step 2: Check if we have any valid signals
    if strategy_results.is_empty() {
        return Err(StrategyError::InsufficientData(
            "No strategies produced valid signals".to_string()
        ));
    }

    // Step 3: Combine signals
    let combined_signal = self.combine_signals(&strategy_results, data)?;

    // Step 4: Store in history
    self.add_to_history(combined_signal.clone()).await;

    Ok(combined_signal)
}
```

##### 3.5.2.5 Combined Signal Output
☐ **REQ-005-011**: System shall produce combined signal:
```rust
pub struct CombinedSignal {
    pub final_signal: TradingSignal,
    pub combined_confidence: f64,
    pub strategy_signals: Vec<StrategySignalResult>,
    pub reasoning: String,
    pub symbol: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, serde_json::Value>,
}

fn combine_signals(
    &self,
    results: &[StrategySignalResult],
    data: &StrategyInput,
) -> Result<CombinedSignal> {
    let (final_signal, combined_confidence, reasoning) =
        match self.config.signal_combination_mode {
            SignalCombinationMode::WeightedAverage =>
                self.combine_weighted_average(results),
            SignalCombinationMode::Consensus =>
                self.combine_consensus(results),
            SignalCombinationMode::BestConfidence =>
                self.combine_best_confidence(results),
            SignalCombinationMode::Conservative =>
                self.combine_conservative(results),
        };

    // Create metadata summary
    let mut metadata = HashMap::new();
    metadata.insert("total_strategies", json!(results.len()));
    metadata.insert("combination_mode",
        json!(format!("{:?}", self.config.signal_combination_mode)));
    metadata.insert("long_signals",
        json!(results.iter().filter(|r| r.signal == TradingSignal::Long).count()));
    metadata.insert("short_signals",
        json!(results.iter().filter(|r| r.signal == TradingSignal::Short).count()));
    metadata.insert("neutral_signals",
        json!(results.iter().filter(|r| r.signal == TradingSignal::Neutral).count()));

    Ok(CombinedSignal {
        final_signal,
        combined_confidence,
        strategy_signals: results.to_vec(),
        reasoning,
        symbol: data.symbol.clone(),
        timestamp: data.timestamp,
        metadata,
    })
}
```

##### 3.5.2.6 Signal History Tracking
☐ **REQ-005-012**: System shall maintain signal history:
```rust
async fn add_to_history(&self, signal: CombinedSignal) {
    let mut history = self.signal_history.write().await;

    history.push(signal);

    // Trim to max size
    if history.len() > self.config.max_history_size {
        history.remove(0);
    }
}

pub async fn get_signal_history(
    &self,
    limit: Option<usize>,
) -> Vec<CombinedSignal> {
    let history = self.signal_history.read().await;

    if let Some(n) = limit {
        history.iter()
            .rev()
            .take(n)
            .cloned()
            .collect()
    } else {
        history.clone()
    }
}
```

##### 3.5.2.7 Strategy Configuration Updates
☐ **REQ-005-013**: System shall support runtime config updates:
```rust
pub fn update_strategy_config(
    &mut self,
    strategy_name: &str,
    config: StrategyConfig,
) -> Result<()> {
    for strategy in &mut self.strategies {
        if strategy.name() == strategy_name {
            strategy.update_config(config);
            return Ok(());
        }
    }

    Err(StrategyError::InvalidConfiguration(
        format!("Strategy '{}' not found", strategy_name)
    ))
}
```

#### 3.5.3 Business Rules
- **BR-005-001**: Minimum 1 strategy must produce valid signal
- **BR-005-002**: Disabled strategies skipped during analysis
- **BR-005-003**: Signal history limited to 1000 entries (default)
- **BR-005-004**: Conservative mode requires 100% agreement
- **BR-005-005**: Weighted average uses strategy-specific weights

#### 3.5.4 Acceptance Criteria
```gherkin
Scenario: Strategy engine combines multiple signals
  Given enabled strategies:
    - RSI: LONG signal, confidence 0.8, weight 1.0
    - MACD: LONG signal, confidence 0.7, weight 1.0
    - Bollinger: NEUTRAL signal, confidence 0.5, weight 1.0
    - Volume: LONG signal, confidence 0.6, weight 0.8
  And combination mode = WeightedAverage
  When engine analyzes market
  Then combined signal shows:
    ☐ Final signal = LONG
    ☐ Combined confidence = weighted average of LONG signals
    ☐ Metadata shows 3 LONG, 0 SHORT, 1 NEUTRAL
    ☐ All 4 strategy results included
    ☐ Reasoning explains weighted average calculation
```

---

### FR-STRATEGIES-006: Technical Indicators

**Priority:** CRITICAL
**Spec ID:** @spec:FR-STRATEGIES-006
**Related APIs:** Utility functions (no direct API)

#### 3.6.1 Description
The system shall provide a comprehensive library of technical indicators used by trading strategies for market analysis. All indicators must be mathematically accurate, well-tested, and optimized for performance.

#### 3.6.2 Detailed Requirements

##### 3.6.2.1 RSI (Relative Strength Index)
☐ **REQ-006-001**: System shall implement RSI using Wilder's smoothing (documented in FR-STRATEGIES-001)

##### 3.6.2.2 MACD (Moving Average Convergence Divergence)
☐ **REQ-006-002**: System shall implement MACD (documented in FR-STRATEGIES-002)

##### 3.6.2.3 Bollinger Bands
☐ **REQ-006-003**: System shall implement Bollinger Bands (documented in FR-STRATEGIES-003)

##### 3.6.2.4 Simple Moving Average (SMA)
☐ **REQ-006-004**: System shall calculate SMA:
```rust
pub fn calculate_sma(prices: &[f64], period: usize) -> Result<Vec<f64>> {
    if prices.len() < period {
        return Err("Insufficient data for SMA calculation".to_string());
    }

    let mut sma_values = Vec::new();

    for i in 0..=prices.len() - period {
        let sum: f64 = prices[i..i + period].iter().sum();
        sma_values.push(sum / period as f64);
    }

    Ok(sma_values)
}
```

##### 3.6.2.5 Exponential Moving Average (EMA)
☐ **REQ-006-005**: System shall calculate EMA with proper initialization:
```rust
pub fn calculate_ema(prices: &[f64], period: usize) -> Result<Vec<f64>> {
    if prices.len() < period {
        return Err("Insufficient data for EMA calculation".to_string());
    }

    let multiplier = 2.0 / (period as f64 + 1.0);
    let mut ema_values = Vec::new();

    // First EMA = SMA of first 'period' prices
    let first_sma = prices[0..period].iter().sum::<f64>() / period as f64;
    ema_values.push(first_sma);

    // Subsequent EMA values
    for price in prices.iter().skip(period) {
        let last_ema = *ema_values.last().unwrap();
        let ema = (price * multiplier) + (last_ema * (1.0 - multiplier));
        ema_values.push(ema);
    }

    Ok(ema_values)
}
```

##### 3.6.2.6 Average True Range (ATR)
☐ **REQ-006-006**: System shall calculate ATR for volatility measurement:
```rust
pub fn calculate_atr(
    candles: &[CandleData],
    period: usize,
) -> Result<Vec<f64>> {
    if candles.len() < period + 1 {
        return Err("Insufficient data for ATR calculation".to_string());
    }

    let mut true_ranges = Vec::new();

    // Calculate True Range for each candle (except first)
    for i in 1..candles.len() {
        let high_low = candles[i].high - candles[i].low;
        let high_close_prev = (candles[i].high - candles[i - 1].close).abs();
        let low_close_prev = (candles[i].low - candles[i - 1].close).abs();

        // True Range = max of three values
        let true_range = high_low.max(high_close_prev).max(low_close_prev);
        true_ranges.push(true_range);
    }

    // ATR = SMA of True Range
    calculate_sma(&true_ranges, period)
}
```

##### 3.6.2.7 Stochastic Oscillator
☐ **REQ-006-007**: System shall calculate Stochastic %K and %D:
```rust
pub struct StochasticResult {
    pub k_percent: Vec<f64>,
    pub d_percent: Vec<f64>,
}

pub fn calculate_stochastic(
    candles: &[CandleData],
    k_period: usize,
    d_period: usize,
) -> Result<StochasticResult> {
    if candles.len() < k_period + d_period {
        return Err("Insufficient data for Stochastic calculation".to_string());
    }

    let mut k_percent = Vec::new();

    // Calculate %K
    for i in k_period - 1..candles.len() {
        let window = &candles[i + 1 - k_period..=i];

        let highest_high = window.iter()
            .map(|c| c.high)
            .fold(f64::NEG_INFINITY, f64::max);

        let lowest_low = window.iter()
            .map(|c| c.low)
            .fold(f64::INFINITY, f64::min);

        let current_close = candles[i].close;

        // %K = ((Current Close - Lowest Low) / (Highest High - Lowest Low)) × 100
        let k = if highest_high == lowest_low {
            50.0  // Avoid division by zero
        } else {
            ((current_close - lowest_low) / (highest_high - lowest_low)) * 100.0
        };

        k_percent.push(k);
    }

    // Calculate %D = SMA of %K
    let d_percent = calculate_sma(&k_percent, d_period)?;

    Ok(StochasticResult {
        k_percent,
        d_percent,
    })
}
```

##### 3.6.2.8 Volume Profile
☐ **REQ-006-008**: System shall calculate volume profile (documented in FR-STRATEGIES-004)

##### 3.6.2.9 Standard Deviation
☐ **REQ-006-009**: System shall calculate standard deviation:
```rust
pub fn calculate_std_deviation(values: &[f64]) -> f64 {
    if values.is_empty() {
        return 0.0;
    }

    let mean = values.iter().sum::<f64>() / values.len() as f64;

    let variance = values.iter()
        .map(|&x| (x - mean).powi(2))
        .sum::<f64>() / values.len() as f64;

    variance.sqrt()
}
```

##### 3.6.2.10 Indicator Validation
☐ **REQ-006-010**: All indicators shall validate input:
- Minimum data length requirements
- Valid period values (> 0)
- No NaN or infinite values in input
- Handle edge cases (zero division, empty arrays)

☐ **REQ-006-011**: All indicators shall return Result type:
```rust
fn calculate_indicator(...) -> Result<Vec<f64>, String> {
    // Validation
    if data.len() < min_required {
        return Err(format!("Need at least {} data points", min_required));
    }

    // Calculation
    // ...

    Ok(result)
}
```

#### 3.6.3 Business Rules
- **BR-006-001**: All indicators use consistent error handling
- **BR-006-002**: Indicators optimized for real-time calculation
- **BR-006-003**: Results cached where appropriate
- **BR-006-004**: All formulas match industry standards

#### 3.6.4 Acceptance Criteria
```gherkin
Scenario: Calculate EMA correctly
  Given price array [10, 11, 12, 13, 14, 15, 16, 17, 18, 19]
  And period = 5
  When calculate_ema is called
  Then result contains:
    ☐ First value = SMA of first 5 prices (12.0)
    ☐ Subsequent values calculated with EMA formula
    ☐ All values > 0
    ☐ Result length = 6 (10 prices - 5 period + 1)
    ☐ No errors
```

---

### FR-STRATEGIES-007: Strategy Backtesting

**Priority:** HIGH
**Spec ID:** @spec:FR-STRATEGIES-007
**Related APIs:** POST `/api/v1/backtest/run`

#### 3.7.1 Description
The system shall provide backtesting capabilities to test strategies against historical data, evaluate performance, and optimize parameters.

#### 3.7.2 Detailed Requirements

##### 3.7.2.1 Backtest Configuration
☐ **REQ-007-001**: System shall accept backtest configuration (documented in FR-PAPER-TRADING-005)

##### 3.7.2.2 Historical Data Replay
☐ **REQ-007-002**: System shall replay historical candles chronologically

☐ **REQ-007-003**: System shall maintain multi-timeframe sync:
```rust
// Ensure all required timeframes available for each timestamp
for timestamp in backtest_period {
    let candles_1h = get_candles("1h", timestamp, lookback);
    let candles_4h = get_candles("4h", timestamp, lookback);

    let input = StrategyInput {
        symbol: config.symbol.clone(),
        timeframe_data: {
            "1h" => candles_1h,
            "4h" => candles_4h,
        },
        current_price: latest_candle.close,
        timestamp,
    };

    let signal = strategy.analyze(&input).await?;
}
```

##### 3.7.2.3 Performance Metrics
☐ **REQ-007-004**: System shall calculate comprehensive metrics (documented in FR-PAPER-TRADING-008)

##### 3.7.2.4 Parameter Optimization
☐ **REQ-007-005**: System shall support grid search optimization (documented in FR-PAPER-TRADING-005)

#### 3.7.3 Business Rules
- **BR-007-001**: Backtest uses paper trading engine
- **BR-007-002**: No lookahead bias allowed
- **BR-007-003**: Slippage and fees must be simulated
- **BR-007-004**: Minimum 30 trades for statistical validity

#### 3.7.4 Acceptance Criteria
```gherkin
Scenario: Backtest RSI strategy
  Given RSI strategy with default parameters
  And historical BTCUSDT 1h data for 90 days
  When backtest runs
  Then results show:
    ☐ Total trades executed
    ☐ Win rate percentage
    ☐ Total return percentage
    ☐ Maximum drawdown
    ☐ Sharpe ratio
    ☐ Equity curve
    ☐ All trades with details
```

---

### FR-STRATEGIES-008: Strategy Configuration

**Priority:** MEDIUM
**Spec ID:** @spec:FR-STRATEGIES-008
**Related APIs:** PATCH `/api/v1/strategies/{name}/config`

#### 3.8.1 Description
The system shall provide flexible strategy configuration allowing parameter tuning, enabling/disabling strategies, and adjusting weights.

#### 3.8.2 Detailed Requirements

##### 3.8.2.1 Strategy Configuration Model
☐ **REQ-008-001**: System shall define strategy config:
```rust
pub struct StrategyConfig {
    pub enabled: bool,
    pub weight: f64,  // For weighted combination
    pub parameters: HashMap<String, serde_json::Value>,
}
```

##### 3.8.2.2 Parameter Validation
☐ **REQ-008-002**: System shall validate parameters:
```rust
impl RsiStrategy {
    fn validate_config(&self, config: &StrategyConfig) -> Result<()> {
        // Validate RSI period
        let period = config.parameters.get("rsi_period")
            .and_then(|v| v.as_u64())
            .ok_or("Missing rsi_period")?;

        if period < 5 || period > 50 {
            return Err("RSI period must be 5-50".into());
        }

        // Validate thresholds
        let oversold = config.parameters.get("oversold_threshold")
            .and_then(|v| v.as_f64())
            .ok_or("Missing oversold_threshold")?;

        if oversold < 20.0 || oversold > 40.0 {
            return Err("Oversold threshold must be 20-40".into());
        }

        Ok(())
    }
}
```

##### 3.8.2.3 Runtime Configuration Updates
☐ **REQ-008-003**: System shall support hot configuration reload:
```rust
pub fn update_config(&mut self, config: StrategyConfig) {
    self.validate_config(&config)?;
    self.config = config;
}
```

##### 3.8.2.4 Configuration Persistence
☐ **REQ-008-004**: System shall persist config to database

☐ **REQ-008-005**: System shall load config on startup

#### 3.8.3 Business Rules
- **BR-008-001**: Parameter changes validated before applying
- **BR-008-002**: Weight must be 0.0 to 2.0
- **BR-008-003**: Config changes logged for audit

#### 3.8.4 Acceptance Criteria
```gherkin
Scenario: Update RSI strategy configuration
  Given RSI strategy with default config
  When user updates config:
    - rsi_period: 21
    - oversold_threshold: 25.0
    - weight: 1.5
  Then strategy uses new parameters
  And configuration persisted to database
  And audit log entry created
```

---

## 4. Data Models

### 4.1 Strategy Trait
```rust
#[async_trait]
pub trait Strategy: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn required_timeframes(&self) -> Vec<&'static str>;
    async fn analyze(&self, data: &StrategyInput) -> Result<StrategyOutput, StrategyError>;
    fn config(&self) -> &StrategyConfig;
    fn update_config(&mut self, config: StrategyConfig);
    fn validate_data(&self, data: &StrategyInput) -> Result<(), StrategyError>;
}
```

### 4.2 StrategyInput
```rust
pub struct StrategyInput {
    pub symbol: String,
    pub timeframe_data: HashMap<String, Vec<CandleData>>,
    pub current_price: f64,
    pub timestamp: i64,
}
```

### 4.3 StrategyOutput
```rust
pub struct StrategyOutput {
    pub signal: TradingSignal,
    pub confidence: f64,
    pub reasoning: String,
    pub timeframe: String,
    pub timestamp: i64,
    pub metadata: HashMap<String, serde_json::Value>,
}
```

### 4.4 TradingSignal
```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TradingSignal {
    Long,
    Short,
    Neutral,
}
```

---

## 5. Business Rules

### 5.1 General Strategy Rules
- **BR-GEN-001**: All strategies must implement Strategy trait
- **BR-GEN-002**: Strategies must validate input data before analysis
- **BR-GEN-003**: Confidence must be in range [0.0, 1.0]
- **BR-GEN-004**: Multi-timeframe strategies require all timeframes

### 5.2 Signal Generation Rules
- **BR-SIG-001**: Signal must have accompanying confidence score
- **BR-SIG-002**: Reasoning must explain signal rationale
- **BR-SIG-003**: Metadata must include indicator values used

### 5.3 Engine Rules
- **BR-ENG-001**: Minimum one strategy must execute successfully
- **BR-ENG-002**: Disabled strategies skipped
- **BR-ENG-003**: Conservative mode requires unanimous agreement

---

## 6. Acceptance Criteria

### 6.1 Overall Strategy System
☐ **AC-001**: All strategies implement Strategy trait correctly
☐ **AC-002**: Strategies generate signals with valid confidence
☐ **AC-003**: Multi-timeframe analysis works correctly
☐ **AC-004**: Strategy Engine combines signals accurately
☐ **AC-005**: All indicators calculate correctly
☐ **AC-006**: Backtesting produces valid results
☐ **AC-007**: Configuration updates apply without restart
☐ **AC-008**: Strategies handle edge cases gracefully
☐ **AC-009**: Performance acceptable for real-time use (<100ms per strategy)
☐ **AC-010**: All formulas match industry standards

---

## 7. Traceability

### 7.1 Related Specifications
- **FR-PAPER-TRADING**: Paper trading integration for strategy testing
- **API-SPEC**: Strategy API endpoints
- **DATA-MODELS**: Market data structures

### 7.2 Source Code References
- `rust-core-engine/src/strategies/rsi_strategy.rs`: RSI implementation
- `rust-core-engine/src/strategies/macd_strategy.rs`: MACD implementation
- `rust-core-engine/src/strategies/bollinger_strategy.rs`: Bollinger Bands
- `rust-core-engine/src/strategies/volume_strategy.rs`: Volume analysis
- `rust-core-engine/src/strategies/strategy_engine.rs`: Strategy coordination
- `rust-core-engine/src/strategies/indicators.rs`: Technical indicators
- `rust-core-engine/src/strategies/mod.rs`: Strategy module definitions

### 7.3 API Endpoints
- `POST /api/v1/strategies/rsi/analyze`: RSI analysis
- `POST /api/v1/strategies/macd/analyze`: MACD analysis
- `POST /api/v1/strategies/bollinger/analyze`: Bollinger analysis
- `POST /api/v1/strategies/volume/analyze`: Volume analysis
- `POST /api/v1/strategies/analyze`: Combined analysis (all strategies)
- `GET /api/v1/strategies/signals`: Get signal history
- `PATCH /api/v1/strategies/{name}/config`: Update strategy config

### 7.4 Test Coverage
- Unit tests: `rust-core-engine/tests/strategies/`
- Integration tests: `rust-core-engine/tests/integration/strategies.rs`
- Indicator tests: `rust-core-engine/src/strategies/indicators.rs` (inline tests)

---

**Document End**

**Revision History:**
- v1.0 (2025-10-10): Initial draft - Complete functional requirements for trading strategies
