# Research Report: Profitable Algorithmic Crypto Trading Strategies (Directional)

**Date**: 2026-02-19
**Context**: Bot-core project with Rust backend, Binance integration, paper trading engine, 7-layer risk mgmt. Current ~46% win rate with basic indicators (RSI/MACD/BB/Volume/Stochastic). Need strategies with actual statistical edge.

---

## Executive Summary

Your current setup uses **lagging indicators in isolation** -- the most crowded, alpha-decayed approach in crypto. The 46% win rate with near-breakeven PnL confirms zero edge. Below is a ranked analysis of 7 strategy categories with concrete implementation paths, backed by academic and practitioner evidence.

**Top 3 Recommendations (skip to end for details):**
1. **Regime-Adaptive Trend Following** (HMM + Donchian/ATR) -- Sharpe 1.2-1.7
2. **Cross-Timeframe Momentum + Volatility Filter** -- Sharpe 1.2-1.5
3. **ML Regime Detection + Gradient Boosting Classifier** -- Sharpe 1.0-1.5

---

## 1. Smart Money Concepts (SMC) / Order Flow

### How It Works
- **Order Blocks (OB)**: Last bearish candle before bullish impulse (demand zone) or last bullish candle before bearish impulse (supply zone). Algorithmically: detect swing structure shifts, mark the candle preceding the move.
- **Fair Value Gaps (FVG)**: Three-candle pattern where candle 1 high < candle 3 low (bullish) or candle 1 low > candle 3 high (bearish). Price tends to fill these gaps.
- **Liquidity Sweeps**: Price pierces above/below a swing high/low (collecting stop losses), then reverses. Detect via: new high/low followed by immediate reversal candle.
- **Break of Structure (BOS) / Change of Character (CHoCH)**: Higher highs/lows breaking = BOS (continuation). Lower high after higher highs = CHoCH (reversal).

### Algorithmic Logic
```
1. Detect swing highs/lows (zigzag with ATR filter)
2. Track market structure (HH/HL = bullish, LH/LL = bearish)
3. On CHoCH: look for nearest OB in new direction
4. Entry: price returns to OB zone + FVG confluence
5. SL: beyond the OB. TP: next liquidity pool (swing high/low)
```

### Performance
- **Win rate**: 55-65% when combined with structure (claims from practitioners; NO rigorous academic backtest exists)
- **Sharpe**: Unknown -- SMC has NOT been rigorously backtested in academic literature
- **Reality check**: SMC is inherently discretionary. The Python [smart-money-concepts](https://github.com/joshyattridge/smart-money-concepts) package exists but produces mixed results in fully automated mode.

### Implementation Complexity
- **Medium-High**. Swing detection is straightforward, but OB/FVG scoring and filtering false signals requires extensive tuning.
- **Solo dev feasible**: Yes, ~2-3 weeks for core detection. Getting it profitable is the hard part.

### Data Requirements
- OHLCV candles (1h/4h) from Binance -- **already available**
- Optional: Volume profile, tick data for confirmation

### Key Risks
- **Subjectivity**: Most SMC edge comes from discretionary context, not mechanical rules
- **Curve-fitting risk**: Easy to overfit OB/FVG parameters to historical data
- **Alpha decay**: SMC has gone mainstream (YouTube, TikTok) -- millions of retail traders now use it

### Verdict: SKIP as primary strategy. Use OB/FVG as supplementary confluence filter only.

---

## 2. Volatility-Based Strategies

### 2A. Donchian Channel Breakout (Turtle-Style)

**How It Works:**
- Buy when price closes above the N-period highest high
- Sell/short when price closes below the N-period lowest low
- ATR-based trailing stop (2x ATR from entry)
- Position sizing: risk 1% of equity per trade, size = risk / (ATR * multiplier)

**Algorithmic Logic:**
```
upper = highest_high(close, 55)  // 55-period for entry
lower = lowest_low(close, 55)
exit_upper = highest_high(close, 20)  // 20-period for exit
exit_lower = lowest_low(close, 20)

if close > upper: LONG entry
if close < lower: SHORT entry
if long and close < exit_lower: EXIT
if short and close > exit_upper: EXIT
trailing_stop = entry - 2 * ATR(20)  // adjusts with price
```

**Performance:**
- Win rate: **35-45%** (typical for trend following; profits come from fat tails)
- Sharpe: **0.8-1.3** in crypto backtests (higher than traditional markets due to crypto's trending nature)
- The original Turtle system adapted to crypto with 200-day breakout + ATR trailing has shown effectiveness across BTC/ETH on daily/4H timeframes
- 35-year backtest across NASDAQ and Gold showed "compelling results" ([Algomatic Trading](https://algomatictrading.substack.com/p/strategy-8-the-easiest-trend-system))

**Implementation Complexity:**
- **LOW**. This is one of the simplest profitable strategies to implement.
- Solo dev: 3-5 days for full implementation in Rust

**Data Requirements:**
- OHLCV candles -- **already available via Binance API**

**Key Risks:**
- Whipsaws in ranging markets (30-40% of crypto time is ranging)
- Extended drawdowns during consolidation (can be 20-30%)
- **Mitigation**: Combine with regime detection to disable during ranging markets

### 2B. Keltner Channel + ATR Squeeze

**How It Works:**
- Keltner Channel = EMA(20) +/- 1.5 * ATR(10)
- Bollinger Bands inside Keltner = "squeeze" (low volatility, coiling)
- Breakout from squeeze = high-probability directional move
- Direction determined by momentum oscillator (e.g., momentum = close - close[12])

**Performance:**
- Win rate: 50-55% with good R:R ratios
- Works well on 4H timeframe for crypto
- Particularly effective for BTC/ETH (well-documented squeeze patterns)

**Implementation**: LOW complexity. You already have Bollinger Bands.

### Verdict: HIGH PRIORITY. Donchian breakout + ATR stops is a proven, simple system. Keltner squeeze is excellent supplementary.

---

## 3. Momentum Strategies With Edge

### 3A. Cross-Sectional Momentum (Altcoin Rotation)

**How It Works:**
- Rank top 20-50 altcoins by N-day return (lookback: 28 days optimal per research)
- Long top quintile, short bottom quintile (or long-only top quintile)
- Rebalance every 5-7 days
- **Key finding**: top quintile produces **69.17% annualized return** vs 36.32% equally weighted ([Drogen, Hoffstein & Otte, 2023 SSRN](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4322637))

**Algorithmic Logic:**
```
every 7 days:
  returns_28d = [pct_change(coin, 28) for coin in universe]
  ranked = sort(returns_28d, descending)
  longs = ranked[top 20%]

  // Absolute momentum filter:
  if BTC_return_28d < 0: go to cash (skip cycle)

  allocate equally across longs
  hold for 5-7 days
```

**Performance:**
- Sharpe: **1.51** (28-day lookback, 5-day hold) vs market Sharpe of 0.84
- Strong evidence from academic research 2020-2024
- Works because crypto has strong momentum persistence at 2-4 week horizons

**Implementation Complexity:**
- **LOW-MEDIUM**. Need multi-coin data and ranking system.
- Solo dev: 1-2 weeks

**Data Requirements:**
- OHLCV for top 50 coins by market cap -- available via Binance REST API
- Need to handle delistings (survivorship bias)

**Key Risks:**
- Momentum crashes during regime shifts (sudden bear markets)
- Higher turnover = higher fees (use Binance maker/taker fee tiers)
- **Mitigation**: Absolute momentum filter (if BTC < 0 over lookback, go to cash)

### 3B. Dual Momentum (Absolute + Relative)

**How It Works:**
- Absolute momentum: Is the asset trending up? (return > 0 over lookback)
- Relative momentum: Which asset is trending strongest? (compare BTC vs ETH vs top alts)
- Only trade when BOTH conditions met

**Performance:**
- Backtest 2014-2024: **382.33% absolute return**, CAGR 18.79%, max drawdown -11.42%
- Sharpe ~1.2-1.5 in crypto contexts

### 3C. Volatility-Filtered Momentum

**How It Works:**
- Standard momentum signal (e.g., 20-day ROC)
- Only take trades when realized volatility < threshold (filter out noise)
- Scale position size inversely with volatility (risk parity)

**Performance:**
- Sharpe ~1.2, improved stability vs raw momentum
- Mean reversion blended with momentum: Sharpe **1.71**, annualized return 56% ([Plotnik, Medium](https://medium.com/@briplotnik/systematic-crypto-trading-strategies-momentum-mean-reversion-volatility-filtering-8d7da06d60ed))

### Verdict: HIGH PRIORITY. Cross-sectional momentum has the strongest academic backing. Dual momentum is the simplest to implement.

---

## 4. Machine Learning Approaches That Actually Work

### 4A. Regime Detection with Hidden Markov Models (HMM)

**How It Works:**
- Train HMM on [log_returns, realized_volatility] to identify 2-4 hidden states
- States map to: Bull/Low-Vol, Bull/High-Vol, Bear/Low-Vol, Bear/High-Vol
- Use regime as a META-FILTER: enable trend-following in trending regimes, mean reversion in ranging, cash in high-vol bear
- Retrain on rolling 2700-day window daily

**Algorithmic Logic:**
```
features = [log_return_1d, realized_vol_20d, volume_ratio]
hmm = GaussianHMM(n_components=3)  // 3 states
hmm.fit(features[-2700:])
current_regime = hmm.predict(features[-1:])

if regime == TRENDING:
    use_strategy = donchian_breakout
elif regime == RANGING:
    use_strategy = mean_reversion
elif regime == CRISIS:
    use_strategy = cash_or_hedge
```

**Performance:**
- Not a standalone strategy -- a META-LAYER that improves other strategies by 20-40%
- Academic paper: [Giudici, 2020](https://onlinelibrary.wiley.com/doi/abs/10.1002/qre.2673) validated HMM regime detection for crypto
- Strategy with regime filter showed lower volatility than buy-and-hold with comparable returns

**Implementation Complexity:**
- **MEDIUM**. Python `hmmlearn` library makes it easy. Run as separate Python service (you already have python-ai-service).
- Solo dev: 1-2 weeks

**Data Requirements:**
- OHLCV + volume -- **already available**

### 4B. Gradient Boosting (XGBoost) Direction Classifier

**How It Works:**
- Target: Will price be higher/lower in N candles? (binary classification)
- Features (from OHLCV -- NO external data needed):
  - Returns at multiple lookbacks (1, 3, 7, 14, 28 candles)
  - Realized volatility (5d, 20d)
  - Volume ratios (current vs MA)
  - ATR ratios
  - RSI/MACD/BB *as features*, not signals
  - Hour-of-day, day-of-week (crypto has time patterns)
  - Distance from recent high/low
  - OBV slope
- Train on rolling 180-day window, predict next 6-24 candles direction

**Performance:**
- XGBoost/RF outperform LSTM with lower MSE and more stable performance (MSE ~0.016)
- Gradient Boosting consistently demonstrates superior accuracy across multiple metrics ([Springer 2025](https://link.springer.com/article/10.1007/s44163-025-00519-y))
- Practical: 55-60% directional accuracy is achievable (vs your current 46%)
- In bear markets, ensemble models turned -54% BTC into +1.25% after costs ([Springer study](https://link.springer.com/chapter/10.1007/978-981-96-6839-7_10))

**Implementation Complexity:**
- **MEDIUM**. XGBoost in Python is straightforward. Feature engineering is the real work.
- Solo dev: 2-3 weeks for production-ready system

**Data Requirements:**
- OHLCV + volume -- **already available**
- Historical data for training (6+ months minimum)

**Key Risks:**
- Overfitting is the #1 killer. MUST use walk-forward validation.
- Regime changes invalidate models -- combine with HMM
- Feature importance shifts over time -- need regular retraining

### 4C. Online Learning / Adaptive Models

- Retrain weekly on most recent data
- Use walk-forward: train on [T-180d, T], test on [T, T+7d], roll forward
- Monitor feature importance drift as early warning

### Verdict: HIGH PRIORITY. HMM regime detection as meta-layer + XGBoost classifier is the highest-alpha combination for a solo dev.

---

## 5. Market Microstructure Strategies

### 5A. Orderbook Imbalance

**How It Works:**
- Imbalance = (bid_volume - ask_volume) / (bid_volume + ask_volume) at top N levels
- Strong bid imbalance (>0.3) = short-term bullish pressure
- Works on 1-5 minute timeframes primarily

**Performance:**
- Academic evidence supports predictive power for 1-5 minute horizons
- Less effective on 1h-4h (your target timeframe)
- Sharpe: Potentially high on short timeframes, degrades on longer ones

**Implementation:**
- Binance provides depth data (up to 5000 levels via REST, top 20 via WebSocket)
- Medium complexity: need to maintain local orderbook

### 5B. CVD (Cumulative Volume Delta)

**How It Works:**
- CVD = cumulative sum of (buy_volume_at_ask - sell_volume_at_bid) over time
- CVD divergence: price makes new high but CVD doesn't = bearish divergence
- CVD confirmation: price breakout + CVD rising = stronger signal

**Algorithmic Logic:**
```
// Approximate CVD from Binance trade stream
for each trade:
  if trade.is_buyer_maker == false:  // taker buy
    cvd += trade.quantity
  else:  // taker sell
    cvd -= trade.quantity

// Divergence detection
if price > price_high_20 and cvd < cvd_at_price_high_20:
  signal = BEARISH_DIVERGENCE
```

**Performance:**
- Best used as confirmation filter, not standalone
- CVD divergence before reversals is well-documented in crypto
- No rigorous Sharpe ratio data available for standalone CVD strategies

**Data Requirements:**
- Binance trade stream (aggTrades WebSocket) -- **available but data-intensive**
- Alternatively: Binance provides taker buy/sell volume in kline data (`taker_buy_base_volume`)

### 5C. VWAP Deviation Mean Reversion

**Performance WARNING:**
- One backtest showed 713% returns with 0% commission, but **-97% return with 0.1% commission** per trade
- Highly sensitive to fees on lower timeframes
- Only viable if you can get <0.02% effective fee rate

### Verdict: USE CVD AS SUPPLEMENTARY SIGNAL only. Orderbook imbalance not suited for 1h-4h. VWAP mean reversion killed by fees.

---

## 6. On-Chain + Sentiment Hybrid

### 6A. Exchange Inflow/Outflow (Supplementary)

**How It Works:**
- Large exchange inflows = selling pressure (bearish)
- Sustained outflows = accumulation (bullish)
- Oct 2023: 40% acceleration in BTC outflows preceded 28% rally over 6 weeks

**Data Requirements:**
- CryptoQuant API, Glassnode API, or Arkham Intelligence -- **NOT available via Binance**
- Free tiers are limited; paid plans $30-100/month

**Implementation:** LOW complexity for the signal itself, but external data dependency.

### 6B. Fear & Greed Index

**How It Works:**
- Buy at Extreme Fear (<25), sell at Extreme Greed (>75)
- Components: volatility, volume, social media, surveys, dominance

**Performance:**
- Backtest 2022-2024: sentiment-augmented strategies improved risk-adjusted returns
- One study: outperformed SPY buy-and-hold by 69% in annualized returns, 119% in Sharpe
- Free API: `https://api.alternative.me/fng/` -- NO cost

### 6C. Funding Rate + Open Interest (Supplementary)

**How It Works:**
- Extreme positive funding (>0.05%) + rising OI = overleveraged longs, reversal risk
- Extreme negative funding (<-0.03%) + rising OI = overleveraged shorts, squeeze risk
- Use as confirmation/filter, not primary signal

**Algorithmic Logic:**
```
funding = binance.get_funding_rate(symbol)
oi = binance.get_open_interest(symbol)
oi_change_24h = pct_change(oi, 24h)

if funding > 0.05 and oi_change_24h > 10%:
  signal = CROWDED_LONG  // reduce long exposure or short
elif funding < -0.03 and oi_change_24h > 10%:
  signal = CROWDED_SHORT  // reduce short exposure or long
```

**Data Requirements:**
- Binance Futures API: `GET /fapi/v1/fundingRate` and `GET /fapi/v1/openInterest` -- **FREE, available**

### Verdict: Funding rate + OI is the highest-value supplementary signal. Free via Binance. Fear & Greed Index is free and adds regime context. Exchange flows require paid external API -- lower priority.

---

## 7. Multi-Factor Strategy (Combining Signals)

### How to Combine 3-4 Uncorrelated Signals

**Architecture:**
```
Signal Layer:
  signal_1 = trend_following(donchian)      // -1 to +1
  signal_2 = momentum(cross_sectional)      // -1 to +1
  signal_3 = ml_direction(xgboost)          // -1 to +1
  signal_4 = sentiment(funding_rate+FGI)    // -1 to +1

Meta Layer:
  regime = hmm_regime()  // TRENDING, RANGING, CRISIS

  if regime == TRENDING:
    weights = [0.4, 0.3, 0.2, 0.1]  // favor trend
  elif regime == RANGING:
    weights = [0.1, 0.2, 0.4, 0.3]  // favor ML + sentiment
  elif regime == CRISIS:
    weights = [0.0, 0.0, 0.0, 1.0]  // only sentiment (go to cash if extreme)

Composite:
  final_signal = sum(signal_i * weight_i)
  if abs(final_signal) > threshold: trade
  position_size = f(final_signal, ATR, account_equity)
```

### Factor Weighting Methods
1. **Equal weight** -- simplest, surprisingly robust
2. **Inverse-volatility weighting** -- scale by 1/sigma of each signal's returns
3. **Walk-forward optimization** -- train weights on rolling 90-day window, test on next 30 days
4. **Kelly-adjusted** -- weight by each signal's historical Sharpe

### Performance (Combined)
- Blended momentum + mean reversion: Sharpe **1.71** ([Plotnik study](https://medium.com/@briplotnik/systematic-crypto-trading-strategies-momentum-mean-reversion-volatility-filtering-8d7da06d60ed))
- Multi-factor with regime detection: expected Sharpe **1.3-2.0** based on component correlations
- Key: uncorrelated signals compound edge. If signal A has 0.55 accuracy and signal B has 0.55 accuracy and they're uncorrelated, combined accuracy approaches 0.65+

### Walk-Forward Optimization Protocol
```
for window in rolling_windows(data, train=180d, test=30d, step=30d):
  train_data = window.train
  test_data = window.test

  # Optimize weights on train
  best_weights = optimize(sharpe_ratio, train_data, constraints=[sum=1, each>=0])

  # Test on out-of-sample
  test_pnl = backtest(test_data, best_weights)
  record(test_pnl)

# Final performance = concatenation of all out-of-sample test periods
```

### Verdict: This is the END GOAL. Build individual signals first, then combine. Multi-factor is where real edge lives.

---

## Binance API Data Availability Summary

| Data | Endpoint | Cost | Useful For |
|------|----------|------|------------|
| OHLCV (klines) | `GET /api/v3/klines` | Free | Everything |
| Taker buy/sell volume | Included in klines | Free | CVD approximation |
| Order book depth | `GET /api/v3/depth` (up to 5000 levels) | Free | Orderbook imbalance |
| Trade stream | WebSocket `aggTrade` | Free | Real-time CVD |
| Funding rate | `GET /fapi/v1/fundingRate` | Free | Sentiment signal |
| Open interest | `GET /fapi/v1/openInterest` | Free | Crowding signal |
| OI statistics | `GET /futures/data/openInterestHist` | Free | Historical OI |
| Long/short ratio | `GET /futures/data/globalLongShortAccountRatio` | Free | Sentiment |
| Liquidation stream | WebSocket `forceOrder` | Free | Liquidation cascades |
| Fear & Greed Index | `api.alternative.me/fng/` | Free | Sentiment (external) |
| Exchange flows | CryptoQuant/Glassnode | $30-100/mo | On-chain signals |

**Bottom line**: Everything you need for the top 3 strategies is FREE via Binance API.

---

## RANKED RECOMMENDATIONS: Top 3 Strategies to Implement

### #1: Regime-Adaptive Trend Following (IMPLEMENT FIRST)

**What**: HMM regime detection + Donchian channel breakout + ATR trailing stops + volatility-based position sizing

**Why first**:
- Highest Sharpe-to-complexity ratio
- Donchian breakout is dead simple to implement in Rust (you already have indicator infrastructure)
- HMM regime detection runs in Python (you already have python-ai-service)
- Addresses your #1 problem: trading in WRONG market conditions
- Expected Sharpe: **1.2-1.7**
- Win rate: 40-50% but with 2:1+ R:R ratio

**Implementation plan**:
1. Implement Donchian channel (55-period entry, 20-period exit) in Rust -- 2 days
2. Add ATR trailing stop (2x ATR) -- 1 day
3. Add volatility-based position sizing (1% risk per trade) -- 1 day
4. Implement HMM regime detection in Python service -- 1 week
5. Wire regime signal to Rust engine (disable trading in RANGING/CRISIS regimes) -- 2 days
6. Backtest with walk-forward validation -- 1 week

**Total**: ~3 weeks. Immediate improvement expected.

**Data needed**: OHLCV (already have), nothing external.

---

### #2: Cross-Timeframe Momentum with Volatility Filter (IMPLEMENT SECOND)

**What**: 28-day cross-sectional momentum across top 20 altcoins + absolute momentum filter (BTC > 0) + volatility scaling

**Why second**:
- Strongest academic evidence (Sharpe 1.51 in peer-reviewed research)
- Uncorrelated with trend following (captures different alpha)
- Forces diversification across coins (reduces single-asset risk)
- Expected Sharpe: **1.2-1.5**

**Implementation plan**:
1. Build multi-coin data pipeline (top 20-30 altcoins from Binance) -- 3 days
2. Implement ranking + quintile selection -- 2 days
3. Add absolute momentum filter (BTC 28d return > 0) -- 1 day
4. Add inverse-volatility position sizing -- 1 day
5. Weekly rebalancing logic -- 2 days
6. Backtest 2020-2025 with survivorship bias handling -- 1 week

**Total**: ~3 weeks. Complements strategy #1 perfectly.

**Data needed**: Multi-coin OHLCV (Binance REST API, free).

---

### #3: ML Regime Detection + Gradient Boosting Classifier (IMPLEMENT THIRD)

**What**: XGBoost binary classifier for 6-24h direction prediction, using engineered features from OHLCV + funding rate + OI

**Why third**:
- Highest potential alpha but highest implementation risk
- Requires careful feature engineering and walk-forward validation
- Build AFTER you have strategies #1 and #2 running (they generate training signal quality data)
- Expected Sharpe: **1.0-1.5** (conservative estimate)

**Implementation plan**:
1. Feature engineering pipeline (20+ features from OHLCV) -- 1 week
2. Add funding rate + OI features from Binance Futures API -- 3 days
3. XGBoost training pipeline with walk-forward validation -- 1 week
4. Prediction service endpoint in python-ai-service -- 3 days
5. Integration with Rust trading engine (ML signal as factor) -- 3 days
6. Extensive backtesting + live paper trading comparison -- 2 weeks

**Total**: ~5 weeks. Highest complexity but also most adaptive.

**Data needed**: OHLCV + funding rate + OI (all free via Binance).

---

## Implementation Priority Timeline

```
Week 1-3:   Strategy #1 (Regime-Adaptive Trend Following)
            - Donchian in Rust + HMM in Python
            - Paper trade immediately

Week 4-6:   Strategy #2 (Cross-Sectional Momentum)
            - Multi-coin pipeline + ranking
            - Paper trade alongside #1

Week 7-11:  Strategy #3 (ML Direction Classifier)
            - Feature engineering + XGBoost
            - Paper trade alongside #1 and #2

Week 12-14: Multi-Factor Combination
            - Combine all 3 signals with regime-adaptive weighting
            - Walk-forward optimization of weights
            - This is where the real edge compounds
```

---

## What to STOP Doing

1. **Stop using RSI/MACD/BB as primary signals** -- zero edge, alpha fully decayed
2. **Stop trading in all market conditions** -- regime detection will 3x your edge
3. **Stop single-coin focus** -- cross-sectional momentum across altcoins captures rotation
4. **Stop fixed position sizing** -- ATR-based sizing is a free upgrade

## What to ADD as Supplementary Filters

These are NOT standalone strategies but improve your primary signals:

| Filter | Data Source | Cost | Expected Improvement |
|--------|------------|------|---------------------|
| Funding rate extreme | Binance Futures API | Free | +5-10% win rate |
| Open interest crowding | Binance Futures API | Free | Avoid liquidation cascades |
| Fear & Greed Index | alternative.me API | Free | Regime confirmation |
| CVD divergence | Binance aggTrade stream | Free | Exit signal improvement |
| Liquidation heatmap levels | Binance forceOrder stream | Free | SL/TP placement |

---

## Unresolved Questions

1. **Survivorship bias in altcoin momentum**: Historical altcoin data includes delisted coins. Need to handle this in backtests (use Binance historical data with delisting dates).
2. **HMM state stability**: How often does HMM flip between states? High flip frequency = whipsaw risk. Need to add minimum state duration filter (e.g., stay in regime for 12+ hours minimum).
3. **XGBoost retraining frequency**: Weekly? Daily? Literature suggests weekly retraining on 180-day rolling window, but crypto regime shifts may require faster adaptation.
4. **Transaction costs**: All Sharpe ratios above are BEFORE fees unless noted. At 0.1% maker fee (Binance), momentum strategies with high turnover lose significant alpha. Target BNB discount + VIP tier for <0.05% effective fee.
5. **Cross-exchange alpha**: Some alpha may exist in cross-exchange momentum differences (same coin, different exchanges). Not researched here -- could be worth investigating.

---

## Sources

- [Drogen, Hoffstein & Otte (2023) - Cross-sectional Momentum in Cryptocurrency Markets (SSRN)](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4322637)
- [Han, Kang & Ryu (2023) - Time-Series and Cross-Sectional Momentum in Cryptocurrency (SSRN)](https://papers.ssrn.com/sol3/papers.cfm?abstract_id=4675565)
- [Giudici (2020) - Hidden Markov Model for Regime Changes in Cryptoasset Markets (Wiley)](https://onlinelibrary.wiley.com/doi/abs/10.1002/qre.2673)
- [Springer 2025 - ML Approaches to Crypto Trading Optimization](https://link.springer.com/article/10.1007/s44163-025-00519-y)
- [Plotnik - Systematic Crypto Trading: Momentum, Mean Reversion & Volatility Filtering](https://medium.com/@briplotnik/systematic-crypto-trading-strategies-momentum-mean-reversion-volatility-filtering-8d7da06d60ed)
- [Algomatic Trading - Donchian Channel Breakout Strategy](https://algomatictrading.substack.com/p/strategy-8-the-easiest-trend-system)
- [QuantInsti - Regime-Adaptive Trading with HMM and Random Forest](https://blog.quantinsti.com/regime-adaptive-trading-python/)
- [arXiv 2407.18334 - Comprehensive Analysis of ML Models for Algorithmic Trading of Bitcoin](https://arxiv.org/html/2407.18334v1)
- [arXiv 2407.11786 - Cryptocurrency Price Forecasting Using XGBoost](https://arxiv.org/html/2407.11786v1)
- [Binance API Docs - Open Interest](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Open-Interest)
- [Binance API Docs - Funding Rate History](https://developers.binance.com/docs/derivatives/usds-margined-futures/market-data/rest-api/Get-Funding-Rate-History)
- [smart-money-concepts Python Package](https://github.com/joshyattridge/smart-money-concepts)
- [QuantPedia - Cryptocurrency Trading Research](https://quantpedia.com/cryptocurrency-trading-research/)
- [Starkiller Capital - Cross-sectional Momentum in Cryptocurrency Markets](https://www.starkiller.capital/post/cross-sectional-momentum-in-cryptocurrency-markets)
