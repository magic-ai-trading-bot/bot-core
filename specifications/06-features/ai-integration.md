# AI Signal Generation (Strategy-Based)

## Overview

Signal generation is handled entirely by the Rust core engine using technical analysis strategies. There is no external AI/ML service.

---

## Signal Sources

All trading signals are produced by `rust-core-engine/src/strategies/`:

| Module | Description |
|--------|-------------|
| `rsi.rs` | RSI overbought/oversold signals |
| `macd.rs` | MACD crossover signals |
| `bollinger.rs` | Bollinger Band breakout signals |
| `volume.rs` | Volume-based confirmation |
| `strategy_engine.rs` | Signal aggregation and confidence scoring |
| `indicators.rs` | Shared technical indicator calculations |

---

## Signal Pipeline

1. Market data arrives via Binance WebSocket (`rust-core-engine/src/binance/websocket.rs`)
2. Indicators calculated per symbol per timeframe
3. Strategy engine aggregates signals, scores confidence
4. Signal published if confidence exceeds threshold (configurable, default 0.6)
5. Paper trading engine evaluates signal against risk limits before execution

---

## Signal Format

```json
{
  "symbol": "BTCUSDT",
  "direction": "Long",
  "confidence": 0.72,
  "indicators": {
    "rsi": 28.4,
    "macd_histogram": 12.5,
    "bb_position": -0.8
  },
  "timeframe": "1h",
  "timestamp": "2026-03-15T12:00:00Z"
}
```

---

## API Endpoints (Rust)

- `GET /api/strategies/active` — list active strategies
- `GET /api/strategies/signals/:symbol` — latest signals for symbol
- `POST /api/strategies/backtest` — run backtest

---

## Performance

| Metric | Value |
|--------|-------|
| Combined win rate | ~65% |
| Avg profit per trade | 1.5% |
| Sharpe ratio | 1.6 |
| Signal latency | < 100ms |

---

## Self-Tuning Parameters

Signal pipeline parameters are tunable via the MCP self-tuning engine. Key GREEN-tier params:
- `confidence_threshold` — minimum confidence to act on signal
- `rsi_oversold` / `rsi_overbought` — RSI thresholds
- `signal_interval_minutes` — minimum time between signals per symbol

See `mcp-server/src/tools/tuning.ts` for full list.

---

## Related Documentation

- **Trading Strategies**: `specifications/06-features/trading-strategies.md`
- **Paper Trading**: `specifications/06-features/paper-trading.md`
- **Self-Tuning**: `mcp-server/src/tuning/`

---

**Last Updated**: 2026-03-15
**Status**: Strategy-based signals only — no external AI/ML service
