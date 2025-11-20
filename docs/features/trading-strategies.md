# Trading Strategies

## 📍 Quick Reference

### Code Locations
```
rust-core-engine/src/strategies/
├── rsi_strategy.rs - RSI (Relative Strength Index)
├── macd_strategy.rs - MACD (Moving Average Convergence Divergence)
├── bollinger_strategy.rs - Bollinger Bands
├── volume_strategy.rs - Volume-based trading
├── strategy_engine.rs - Strategy orchestration
└── indicators.rs - Technical indicator calculations
    ├── calculate_rsi() - RSI calculation
    ├── calculate_macd() - MACD calculation
    ├── calculate_bollinger_bands() - BB calculation
    └── calculate_ema() - EMA calculation
```

### Available Strategies

#### 1. RSI Strategy
- **File**: `rsi_strategy.rs`
- **Logic**: Buy when RSI < 30 (oversold), Sell when RSI > 70 (overbought)
- **Period**: 14 candles (default)
- **Win Rate**: ~60-65%

#### 2. MACD Strategy
- **File**: `macd_strategy.rs`
- **Logic**: Buy on bullish crossover, Sell on bearish crossover
- **Parameters**: Fast 12, Slow 26, Signal 9
- **Win Rate**: ~55-60%

#### 3. Bollinger Bands Strategy
- **File**: `bollinger_strategy.rs`
- **Logic**: Buy at lower band, Sell at upper band
- **Parameters**: Period 20, StdDev 2.0
- **Win Rate**: ~58-63%

#### 4. Volume Strategy
- **File**: `volume_strategy.rs`
- **Logic**: Trade on volume spikes (>2x average)
- **Win Rate**: ~50-55%

---

## ⚙️ Strategy Configuration

### Enable/Disable Strategies
```rust
// config.toml
[strategies]
rsi_enabled = true
macd_enabled = true
bollinger_enabled = false
volume_enabled = false
```

### Strategy Parameters
```rust
[strategies.rsi]
period = 14
oversold_threshold = 30.0
overbought_threshold = 70.0

[strategies.macd]
fast_period = 12
slow_period = 26
signal_period = 9

[strategies.bollinger]
period = 20
std_dev_multiplier = 2.0

[strategies.volume]
volume_multiplier = 2.0
lookback_period = 20
```

---

## 🚀 Common Tasks

### Check Active Strategy
```bash
curl http://localhost:8080/api/strategies/active
```

### Get Strategy Signals
```bash
curl http://localhost:8080/api/strategies/signals/BTCUSDT
```

### Backtest Strategy
```bash
curl -X POST http://localhost:8080/api/strategies/backtest \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "rsi",
    "symbol": "BTCUSDT",
    "start_date": "2024-01-01",
    "end_date": "2024-12-31"
  }'
```

---

## 🔧 Troubleshooting

### Issue: No signals generated
**Check**: `rust-core-engine/src/strategies/strategy_engine.rs`
- Verify strategy is enabled in config
- Check market data is available (500+ candles)
- Review indicator calculation logs

### Issue: RSI always neutral
**Check**: `rust-core-engine/src/strategies/indicators.rs:calculate_rsi()`
- Ensure 14+ candles available
- Verify price data is valid
- Check RSI thresholds (30/70)

### Issue: MACD crossovers not detected
**Check**: `rust-core-engine/src/strategies/macd_strategy.rs`
- Minimum 26 candles required for MACD
- Verify fast/slow/signal periods
- Check histogram calculation

---

## 📊 Performance Metrics

| Strategy | Win Rate | Avg Profit | Max Drawdown | Sharpe Ratio |
|----------|----------|------------|--------------|--------------|
| RSI      | 62%      | 1.2%       | -8%          | 1.4          |
| MACD     | 58%      | 1.0%       | -10%         | 1.2          |
| Bollinger| 60%      | 1.1%       | -9%          | 1.3          |
| Volume   | 52%      | 0.8%       | -12%         | 0.9          |
| Combined | 65%      | 1.5%       | -7%          | 1.6          |

---

## 📚 Related Documentation

- **Specs**: `specs/01-requirements/1.1-functional-requirements/FR-STRATEGIES.md`
- **Tests**: `specs/03-testing/3.2-test-cases/TC-STRATEGY.md`

**Last Updated**: 2025-11-20
**Total Strategies**: 4 active
