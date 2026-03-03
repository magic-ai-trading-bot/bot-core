# Test Cases - AI/ML Module

**Document ID:** TC-AI-001
**Version:** 1.0
**Last Updated:** 2025-10-11
**Status:** Active
**Related FR:** FR-AI (Functional Requirements - AI/ML)

---

## Table of Contents

1. [Test Case Summary](#test-case-summary)
2. [ML Model Prediction Test Cases](#ml-model-prediction-test-cases)
3. [Technical Indicator Test Cases](#technical-indicator-test-cases)
4. [GPT-4 Integration Test Cases](#gpt-4-integration-test-cases)
5. [Signal Generation Test Cases](#signal-generation-test-cases)
6. [Caching Mechanism Test Cases](#caching-mechanism-test-cases)
7. [Feature Engineering Test Cases](#feature-engineering-test-cases)
8. [Traceability Matrix](#traceability-matrix)

---

## Test Case Summary

| Category | Total Tests | Priority | Coverage |
|----------|-------------|----------|----------|
| ML Model Predictions | 9 | Critical | 100% |
| Technical Indicators | 12 | Critical | 100% |
| GPT-4 Integration | 7 | High | 100% |
| Signal Generation | 6 | Critical | 100% |
| Caching Mechanism | 4 | Medium | 100% |
| Feature Engineering | 5 | High | 100% |
| **TOTAL** | **43** | - | **100%** |

**Test File Locations:**
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_models.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_analyzer.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_gpt_analyzer.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_indicators.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_redis_cache.py`
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_feature_engineering.py`

---

## ML Model Prediction Test Cases

### TC-AI-001: LSTM Model Price Prediction

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AI-001

**Test Scenario (Gherkin):**
```gherkin
Feature: ML Model Predictions
  As the AI service
  I want to predict future prices using ML models
  So that traders can make informed decisions

  Scenario: LSTM model predicts next price
    Given I have 100 hours of BTC/USDT price history
    And prices range from 48000 to 52000
    When I input price data to LSTM model
    Then model should predict next price
    And prediction should be within reasonable range (45000-55000)
    And confidence score should be between 0 and 1
```

**Test Steps:**
1. Load historical price data (100 candles)
2. Preprocess data (normalization)
3. Feed data to LSTM model
4. Get prediction
5. Validate prediction range
6. Validate confidence score

**Expected Results:**
- ✅ Prediction generated within 5000ms
- ✅ Prediction within ±10% of current price
- ✅ Confidence score returned
- ✅ No NaN or Inf values

**Code Location:**
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/models/lstm_model.py`
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_models.py`

---

### TC-AI-002: GRU Model Price Prediction

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AI-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: GRU model predicts next price
    Given historical price data
    When I use GRU model for prediction
    Then prediction should be generated
    And prediction should differ from LSTM (model diversity)
    And both models' average may be more accurate
```

---

### TC-AI-003: Transformer Model Price Prediction

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Transformer model predicts with attention mechanism
    Given historical price data
    When I use Transformer model
    Then model should use attention mechanism
    And prediction should capture long-term dependencies
    And prediction accuracy should be >= LSTM accuracy
```

---

### TC-AI-004: Ensemble Model Prediction

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-002

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Ensemble combines multiple model predictions
    Given LSTM predicts 50100
    And GRU predicts 50300
    And Transformer predicts 50200
    When I create ensemble prediction
    Then ensemble should average predictions
    And result should be 50200
    And ensemble confidence should be higher than individual
```

---

### TC-AI-005: Model Prediction with Insufficient Data

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle insufficient data gracefully
    Given I have only 10 candles (need 50+)
    When I attempt prediction
    Then model should return error
    And error should be "Insufficient data: need 50+, got 10"
    And no prediction should be generated
```

---

### TC-AI-006: Model Prediction with Missing Values

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle missing values in input data
    Given price data has NaN values
    When I preprocess data
    Then missing values should be filled (forward fill)
    And model should predict successfully
```

---

### TC-AI-007: Model Retraining

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-003

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Retrain model with new data
    Given model was trained on data until 2025-10-01
    And new data is available until 2025-10-11
    When I trigger model retraining
    Then model should incorporate new data
    And model weights should be updated
    And new model should be saved
```

---

### TC-AI-008: Model Performance Metrics

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AI-004

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate model performance metrics
    Given I have test dataset with actual prices
    When I compare predictions to actuals
    Then I should calculate:
      - RMSE (Root Mean Square Error)
      - MAE (Mean Absolute Error)
      - R² score
      - Accuracy percentage
```

---

### TC-AI-009: Model Extreme Price Handling

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-001

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle extreme price movements
    Given price suddenly spikes 1000% (flash crash scenario)
    When model receives this data
    Then model should detect anomaly
    And prediction should be marked as low confidence
    Or prediction should be rejected
```

---

## Technical Indicator Test Cases

### TC-AI-010: RSI Calculation

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
Feature: Technical Indicator Calculations
  As the AI service
  I want to calculate technical indicators
  So that I can provide comprehensive analysis

  Scenario: Calculate RSI (Relative Strength Index)
    Given I have 100 price candles
    When I calculate RSI with period 14
    Then RSI values should be between 0 and 100
    And RSI > 70 indicates overbought
    And RSI < 30 indicates oversold
```

**Test Steps:**
1. Create sample price data
2. Calculate RSI with period=14
3. Verify RSI values in valid range
4. Test oversold/overbought conditions

**Expected Results:**
- ✅ RSI values: 0 ≤ RSI ≤ 100
- ✅ Calculation matches manual calculation
- ✅ NaN only for first 13 periods

**Code Location:**
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/features/technical_indicators.py::calculate_rsi`
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_indicators.py::TestCalculateRSI`

---

### TC-AI-011: MACD Calculation

**Priority:** Critical
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate MACD (Moving Average Convergence Divergence)
    Given price data
    When I calculate MACD with default params (12, 26, 9)
    Then I should receive:
      - MACD line
      - Signal line
      - Histogram (MACD - Signal)
    And crossover should indicate buy/sell signals
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_indicators.py::TestCalculateMACD`

---

### TC-AI-012: Bollinger Bands Calculation

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate Bollinger Bands
    Given price data
    When I calculate Bollinger Bands (period=20, std=2)
    Then I should receive:
      - Upper band (SMA + 2*std)
      - Middle band (SMA)
      - Lower band (SMA - 2*std)
    And price touching upper band indicates overbought
    And price touching lower band indicates oversold
```

---

### TC-AI-013: EMA Calculation

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate EMA (Exponential Moving Average)
    Given price data
    When I calculate EMA with periods [9, 21, 50, 200]
    Then all EMAs should be calculated
    And EMA should give more weight to recent prices
    And EMA crossovers should indicate trends
```

---

### TC-AI-014: ATR Calculation

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate ATR (Average True Range)
    Given OHLC data
    When I calculate ATR with period=14
    Then ATR should measure volatility
    And higher ATR indicates higher volatility
```

---

### TC-AI-015: ADX Calculation

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate ADX (Average Directional Index)
    Given OHLC data
    When I calculate ADX
    Then ADX > 25 indicates strong trend
    And ADX < 20 indicates weak trend
```

---

### TC-AI-016: Stochastic Oscillator Calculation

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate Stochastic Oscillator
    Given OHLC data
    When I calculate Stochastic (K%, D%)
    Then values should be between 0 and 100
    And %K > 80 indicates overbought
    And %K < 20 indicates oversold
```

---

### TC-AI-017: Volume SMA Calculation

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate Volume SMA
    Given volume data
    When I calculate volume SMA
    Then high volume indicates strong moves
    And volume above SMA confirms trend
```

---

### TC-AI-018: Indicator Calculation with Edge Cases

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-005

**Test Scenario (Gherkin):**
```gherkin
  Scenario Outline: Handle edge cases in indicator calculation
    Given price data is <data_type>
    When I calculate indicators
    Then result should be <result>

    Examples:
      | data_type          | result                    |
      | empty              | empty series              |
      | single candle      | NaN for most indicators   |
      | all same prices    | indicators = 0 or neutral |
      | extreme volatility | valid but extreme values  |
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_analyzer.py::TestEdgeCases`

---

### TC-AI-019: Prepare DataFrame from Klines

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-006

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Convert Binance klines to DataFrame
    Given Binance klines data (array of arrays)
    When I prepare DataFrame
    Then DataFrame should have columns: open, high, low, close, volume
    And data types should be float64
    And index should be datetime
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_analyzer.py::test_prepare_dataframe`

---

### TC-AI-020: Detect Chart Patterns

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-007

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Detect chart patterns
    Given price data
    When I detect patterns
    Then I should identify:
      - Double top/bottom
      - Head and shoulders
      - Triangles (ascending/descending)
      - Flags (bullish/bearish)
      - Cup and handle
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_analyzer.py::test_detect_patterns`

---

### TC-AI-021: Get Market Context

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-008

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Generate market context
    Given price data and indicators
    When I get market context
    Then context should include:
      - Trend strength (-1 to 1)
      - Volatility (0 to 1)
      - Volume trend (increasing/decreasing/stable)
      - Market sentiment (bullish/bearish/neutral)
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_technical_analyzer.py::test_get_market_context`

---

## GPT-4 Integration Test Cases

### TC-AI-022: GPT-4 Trading Signal Analysis

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-009

**Test Scenario (Gherkin):**
```gherkin
Feature: GPT-4 Integration
  As the AI service
  I want to use GPT-4 for market analysis
  So that I can provide intelligent trading insights

  Scenario: Analyze trading signals with GPT-4
    Given I have technical indicators
    And I have market context
    When I send data to GPT-4 for analysis
    Then GPT-4 should return:
      - Signal (Long/Short/Neutral)
      - Confidence (0-1)
      - Reasoning (explanation)
      - Timestamp
```

**Test Steps:**
1. Prepare sample technical indicators
2. Create market context
3. Call GPT-4 API
4. Parse response
5. Validate signal format

**Expected Results:**
- ✅ Response within 5 seconds
- ✅ Signal is one of: Long, Short, Neutral
- ✅ Confidence between 0 and 1
- ✅ Reasoning is non-empty string

**Code Location:**
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/services/gpt_analyzer.py`
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_gpt_analyzer.py::test_analyze_trading_signals_success`

---

### TC-AI-023: GPT-4 API Error Handling

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AI-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle GPT-4 API errors gracefully
    Given GPT-4 API is unavailable
    When I attempt analysis
    Then system should fall back to technical analysis
    And signal should still be generated
    And reasoning should indicate "Technical analysis" fallback
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_gpt_analyzer.py::test_analyze_with_api_error`

---

### TC-AI-024: GPT-4 Invalid JSON Response

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle invalid JSON from GPT-4
    Given GPT-4 returns non-JSON text
    When I parse response
    Then system should handle gracefully
    And fall back to technical analysis
```

**Code Location:**
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_gpt_analyzer.py::test_analyze_with_invalid_json_response`

---

### TC-AI-025: GPT-4 Rate Limiting

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle GPT-4 rate limiting
    Given I exceed GPT-4 API rate limit
    When API returns 429 Too Many Requests
    Then system should implement exponential backoff
    And retry after delay
    Or fall back to cached/technical analysis
```

---

### TC-AI-026: GPT-4 Cost Optimization

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Optimize GPT-4 API costs
    Given I have analysis request
    When I check cache first
    And cached result exists (< 5 minutes old)
    Then use cached result instead of calling GPT-4
    And save API costs
```

---

### TC-AI-027: GPT-4 Prompt Engineering

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Use optimized prompts for GPT-4
    Given technical indicators
    When I construct GPT-4 prompt
    Then prompt should be concise
    And include relevant context only
    And request structured JSON output
    And include confidence requirement
```

---

### TC-AI-028: GPT-4 Context Window Management

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-009

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Manage GPT-4 context window limits
    Given very large indicator dataset
    When dataset exceeds token limit
    Then summarize or truncate data
    And ensure prompt fits within limits
```

---

## Signal Generation Test Cases

### TC-AI-029: Generate Buy Signal

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AI-010

**Test Scenario (Gherkin):**
```gherkin
Feature: Trading Signal Generation
  As the AI service
  I want to generate trading signals
  So that traders know when to buy/sell

  Scenario: Generate buy signal with high confidence
    Given RSI is 25 (oversold)
    And MACD crosses above signal line
    And price bounces off lower Bollinger Band
    And GPT-4 confirms "Long" signal
    When I generate trading signal
    Then signal should be "BUY"
    And confidence should be > 0.75
    And reasoning should list all confirming factors
```

---

### TC-AI-030: Generate Sell Signal

**Priority:** Critical
**Test Type:** Integration
**Related FR:** FR-AI-010

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Generate sell signal
    Given RSI is 80 (overbought)
    And MACD crosses below signal line
    And price touches upper Bollinger Band
    And GPT-4 confirms "Short" signal
    When I generate trading signal
    Then signal should be "SELL"
    And confidence should be > 0.70
```

---

### TC-AI-031: Generate Neutral Signal

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-010

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Generate neutral signal in uncertain market
    Given indicators show mixed signals
    And trend is unclear
    And GPT-4 returns "Neutral"
    When I generate trading signal
    Then signal should be "NEUTRAL" or "HOLD"
    And confidence should be moderate (0.4-0.6)
```

---

### TC-AI-032: Signal Confidence Calculation

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-011

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Calculate signal confidence score
    Given multiple indicators agree on direction
    And GPT-4 confidence is high
    And model predictions align
    When I calculate overall confidence
    Then confidence should be weighted average
    And range should be 0 to 1
```

---

### TC-AI-033: Signal Filtering

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-012

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Filter low confidence signals
    Given minimum confidence threshold is 0.6
    When signal confidence is 0.45
    Then signal should be filtered out
    And "NEUTRAL" should be returned instead
```

---

### TC-AI-034: Signal History Tracking

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-013

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Track signal performance history
    Given I generated "BUY" signal at price 50000
    When price moves to 51000 after 1 hour
    Then signal should be marked as successful
    And performance metrics should be updated
```

---

## Caching Mechanism Test Cases

### TC-AI-035: Redis Cache Hit

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-014

**Test Scenario (Gherkin):**
```gherkin
Feature: Caching Mechanism
  As the AI service
  I want to cache analysis results
  So that I can reduce API calls and improve performance

  Scenario: Retrieve cached analysis result
    Given I analyzed BTC/USDT 2 minutes ago
    And result is cached in Redis
    When I request analysis again for same symbol
    Then cached result should be returned
    And no new GPT-4 call should be made
    And response time should be < 50ms
```

**Code Location:**
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/services/redis_cache.py`
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_redis_cache.py`

---

### TC-AI-036: Redis Cache Miss

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-014

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle cache miss
    Given no cached result exists
    When I request analysis
    Then fresh analysis should be performed
    And result should be cached for future use
    And cache TTL should be 5 minutes
```

---

### TC-AI-037: Redis Cache Expiration

**Priority:** Medium
**Test Type:** Integration
**Related FR:** FR-AI-014

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Cache expires after TTL
    Given I cached result 6 minutes ago
    And cache TTL is 5 minutes
    When I request analysis
    Then cached result should be expired
    And fresh analysis should be performed
```

---

### TC-AI-038: Redis Connection Failure

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-014

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Handle Redis connection failure
    Given Redis is unavailable
    When I attempt to cache result
    Then system should continue without cache
    And analysis should still complete
    And error should be logged
```

---

## Feature Engineering Test Cases

### TC-AI-039: Feature Extraction

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-015

**Test Scenario (Gherkin):**
```gherkin
Feature: Feature Engineering
  As the AI service
  I want to engineer features from raw data
  So that ML models can learn effectively

  Scenario: Extract features from price data
    Given raw OHLCV data
    When I extract features
    Then I should generate:
      - Price changes (absolute and percentage)
      - Moving averages
      - Volatility measures
      - Volume features
      - Time-based features
```

**Code Location:**
- Python: `/Users/dungngo97/Documents/bot-core/python-ai-service/features/feature_engineering.py`
- Test: `/Users/dungngo97/Documents/bot-core/python-ai-service/tests/test_feature_engineering.py`

---

### TC-AI-040: Feature Normalization

**Priority:** High
**Test Type:** Unit
**Related FR:** FR-AI-015

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Normalize features for ML model
    Given raw features with different scales
    When I normalize features
    Then all features should be scaled to [0, 1] or [-1, 1]
    And normalization params should be saved for inverse transform
```

---

### TC-AI-041: Feature Selection

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AI-015

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Select most important features
    Given 50 engineered features
    When I perform feature selection
    Then top 20 most correlated features should be selected
    And multicollinearity should be reduced
```

---

### TC-AI-042: Lagged Features Creation

**Priority:** Medium
**Test Type:** Unit
**Related FR:** FR-AI-015

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Create lagged features for time series
    Given price data
    When I create lagged features
    Then I should have features for t-1, t-2, ..., t-n
    And lagged features help model capture temporal patterns
```

---

### TC-AI-043: Feature Engineering Pipeline

**Priority:** High
**Test Type:** Integration
**Related FR:** FR-AI-015

**Test Scenario (Gherkin):**
```gherkin
  Scenario: Run complete feature engineering pipeline
    Given raw market data
    When I run feature engineering pipeline
    Then pipeline should:
      1. Extract raw features
      2. Create lagged features
      3. Normalize features
      4. Select important features
      5. Return feature matrix ready for ML
```

---

## Traceability Matrix

| Test Case ID | Related FR | Priority | Code Location | Status |
|--------------|------------|----------|---------------|--------|
| TC-AI-001 | FR-AI-001 | Critical | `models/lstm_model.py` | ✅ |
| TC-AI-010 | FR-AI-005 | Critical | `features/technical_indicators.py::calculate_rsi` | ✅ |
| TC-AI-011 | FR-AI-005 | Critical | `features/technical_indicators.py::calculate_macd` | ✅ |
| TC-AI-018 | FR-AI-005 | High | `tests/test_technical_analyzer.py::TestEdgeCases` | ✅ |
| TC-AI-019 | FR-AI-006 | High | `tests/test_technical_analyzer.py::test_prepare_dataframe` | ✅ |
| TC-AI-020 | FR-AI-007 | Medium | `tests/test_technical_analyzer.py::test_detect_patterns` | ✅ |
| TC-AI-021 | FR-AI-008 | High | `tests/test_technical_analyzer.py::test_get_market_context` | ✅ |
| TC-AI-022 | FR-AI-009 | High | `tests/test_gpt_analyzer.py::test_analyze_trading_signals_success` | ✅ |
| TC-AI-023 | FR-AI-009 | Critical | `tests/test_gpt_analyzer.py::test_analyze_with_api_error` | ✅ |
| TC-AI-035 | FR-AI-014 | Medium | `tests/test_redis_cache.py` | ✅ |
| TC-AI-039 | FR-AI-015 | High | `tests/test_feature_engineering.py` | ✅ |

---

## Acceptance Criteria

AI/ML module is considered complete when:

- [ ] All 43 test cases pass
- [ ] Code coverage >= 85% for AI service
- [ ] ML models predict with reasonable accuracy
- [ ] Technical indicators calculate correctly
- [ ] GPT-4 integration works with error handling
- [ ] Signal generation produces actionable insights
- [ ] Caching reduces API calls by 50%+
- [ ] Feature engineering pipeline complete
- [ ] Performance: Analysis completes in < 5000ms
- [ ] No critical bugs in AI logic

---

**Document Control:**
- **Created by**: AI/ML Team
- **Reviewed by**: Data Science Team
- **Approved by**: Product Owner
- **Next Review Date**: 2025-11-11

---

*End of AI/ML Test Cases Document*
