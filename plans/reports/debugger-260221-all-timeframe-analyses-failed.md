# Debugger Report: "All timeframe analyses failed" - Root Cause Analysis

**Date**: 2026-02-21
**Issue**: Rust strategy engine logs "All timeframe analyses failed for BTCUSDT" every 5 minutes
**Severity**: HIGH - Rust native analysis completely broken, no signals being generated

---

## Executive Summary

The Rust engine's periodic analysis fails 100% of the time across all symbols (BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT) due to a **schema mismatch** between what Rust sends and what the Python AI service returns from `POST /ai/analyze`.

Two distinct bugs compound each other:

1. **PRIMARY (fatal)**: Rust `AnalysisResponse` struct does not match Python `AISignalResponse` JSON shape → `error decoding response body` on every call
2. **SECONDARY**: Rust `StrategyContext.active_strategies` field is serialized as `active_strategies` but Python `AIStrategyContext` expects `selected_strategies` — meaning strategy context is silently ignored (Python uses default empty list)
3. **TERTIARY**: Binance rate-limited with 403 Forbidden during live kline refresh (cache updates), though this does NOT block analysis since DB-cached data is used

---

## Error Chain (Exact)

```
processor.rs:start_periodic_analysis()
  └─► analyzer.rs:analyze_multi_timeframe(symbol, timeframes, "trend_analysis", Some(100))
        └─► for each timeframe:
              └─► analyze_single_timeframe(symbol, timeframe, ...)
                    ├─► cache.get_candles(symbol, timeframe, limit)  [OK: 500 candles in cache]
                    ├─► POST http://python-ai-service:8000/ai/analyze [HTTP 200 OK]
                    └─► response.json::<AnalysisResponse>()  [FAILS: error decoding response body]
                          └─► warn!("Failed to analyze {} {}: {}", symbol, timeframe, e)
        └─► timeframe_signals.is_empty() == true (ALL failed)
              └─► Err("All timeframe analyses failed for {}")
                    └─► warn!("Analysis failed for {}: {}", symbol, e)
```

---

## Root Cause #1: Response Schema Mismatch (Fatal)

### What Rust expects (`AnalysisResponse` in `analyzer.rs:55-63`):
```rust
pub struct AnalysisResponse {
    pub symbol: String,       // REQUIRED
    pub timeframe: String,    // REQUIRED
    pub timestamp: i64,       // REQUIRED
    pub signal: TradingSignal, // enum: "BUY","SELL","HOLD","STRONG_BUY","STRONG_SELL"
    pub confidence: f64,
    pub indicators: HashMap<String, f64>,
    pub analysis_details: serde_json::Value,
}
```

### What Python returns (`AISignalResponse` in `main.py:694-703`):
```json
{
  "signal": "Neutral",          // string "Long"/"Short"/"Neutral" (NOT "BUY"/"SELL"/"HOLD")
  "confidence": 0.5,
  "reasoning": "...",           // EXTRA field (not in Rust struct)
  "strategy_scores": {...},     // EXTRA field
  "market_analysis": {...},     // EXTRA field (nested object)
  "risk_assessment": {...},     // EXTRA field (nested object)
  "timestamp": 1000
  // MISSING: symbol, timeframe, indicators, analysis_details
}
```

Rust's `serde_json` fails to deserialize because:
- `symbol` is required but missing
- `timeframe` is required but missing
- `indicators` is required but missing
- `analysis_details` is required but missing
- `signal` value "Neutral" cannot deserialize into `TradingSignal` enum (expects "BUY"/"SELL"/"HOLD"/"STRONG_BUY"/"STRONG_SELL")

**Confirmed live on VPS**: `curl http://localhost:8000/ai/analyze` returns the Python shape, not the Rust expected shape.

---

## Root Cause #2: Request Field Name Mismatch (Silent)

### Rust sends (`StrategyContext` in `analyzer.rs:21-29`):
```json
{
  "strategy_context": {
    "active_strategies": ["trend_analysis"],  // field: active_strategies
    "portfolio_size": 10000.0,                // EXTRA field Python ignores
    "risk_tolerance": "moderate",              // EXTRA field Python ignores
    "market_condition": "Unknown",
    "risk_level": "Moderate",
    "user_preferences": {},
    "technical_indicators": {}
  }
}
```

### Python expects (`AIStrategyContext` in `main.py:645-652`):
```python
class AIStrategyContext(BaseModel):
    selected_strategies: List[str]   # field: selected_strategies (DIFFERENT!)
    market_condition: str
    risk_level: str
    user_preferences: Dict
    technical_indicators: Dict
```

FastAPI silently accepts `active_strategies` in the JSON and maps it to nothing → Python always uses `selected_strategies=[]` (default empty list). This means Python never knows which strategy to run.

---

## Root Cause #3: Binance 403 Rate Limiting (Secondary)

Logs show repeated `Rate limited (403 Forbidden), retrying 2s/4s/6s` during the live kline refresh portion of `load_historical_klines`. However, this does NOT block analysis because:
- DB-cached data is loaded first (500 candles shown in logs)
- The failure only affects the top-up fetch of 5 new candles
- The 403 suggests the Binance testnet or the VPS IP may be rate-limited or blocked

---

## Evidence from VPS Logs

```
2026-02-21T04:09:04Z WARN analyzer: Failed to analyze ETHUSDT 1d: error decoding response body
2026-02-21T04:09:04Z WARN processor: Analysis failed for ETHUSDT: All timeframe analyses failed for ETHUSDT
2026-02-21T04:14:13Z WARN analyzer: Failed to analyze BTCUSDT 1m: error decoding response body
2026-02-21T04:14:13Z WARN analyzer: Failed to analyze BTCUSDT 3m: error decoding response body
...  (all 8 timeframes: 1m,3m,5m,15m,30m,1h,4h,1d)
2026-02-21T04:14:13Z WARN processor: Analysis failed for BTCUSDT: All timeframe analyses failed for BTCUSDT
```

Cache IS populated (500 candles per symbol/timeframe - not a data problem):
```
2026-02-21T04:18:27Z INFO cache: Added 5 historical candles for BTCUSDT 1m, total: 500
2026-02-21T04:18:27Z INFO cache: Added 5 historical candles for BTCUSDT 3m, total: 500
```

Python AI service returns HTTP 200 (not a connectivity problem):
```
INFO: 172.20.0.5:53248 - "POST /ai/analyze HTTP/1.1" 200 OK  (repeated)
```

---

## Fix Options

### Option A: Fix Rust `AnalysisResponse` to match Python output (RECOMMENDED)

Update `rust-core-engine/src/market_data/analyzer.rs` to deserialize the actual Python response format:

```rust
// Replace AnalysisResponse with:
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisResponse {
    pub signal: String,          // "Long", "Short", "Neutral"
    pub confidence: f64,
    pub reasoning: String,
    pub strategy_scores: HashMap<String, f64>,
    pub market_analysis: serde_json::Value,
    pub risk_assessment: serde_json::Value,
    pub timestamp: i64,
    // Derived fields (not from Python, computed locally):
    #[serde(skip_deserializing)]
    pub symbol: String,
    #[serde(skip_deserializing)]
    pub timeframe: String,
}

// Update TradingSignal to handle Python's values:
pub enum TradingSignal {
    #[serde(rename = "Long")]
    Buy,
    #[serde(rename = "Short")]
    Sell,
    #[serde(rename = "Neutral")]
    Hold,
    // Keep old names for backward compat if needed
    #[serde(rename = "BUY")]
    StrongBuy,
    #[serde(rename = "SELL")]
    StrongSell,
}
```

Also fix `StrategyContext.active_strategies` → rename to `selected_strategies` in serialization:

```rust
pub struct StrategyContext {
    #[serde(rename = "selected_strategies")]
    pub active_strategies: Vec<String>,
    // Remove portfolio_size, risk_tolerance (Python ignores them)
    pub market_condition: String,
    pub risk_level: String,
    pub user_preferences: HashMap<String, serde_json::Value>,
    pub technical_indicators: HashMap<String, serde_json::Value>,
}
```

### Option B: Fix Python `/ai/analyze` to match Rust struct

Add `symbol`, `timeframe`, `indicators`, `analysis_details` fields to `AISignalResponse` and change signal values to "BUY"/"SELL"/"HOLD". More invasive - breaks other clients.

### Option C: Add a Rust-compatible adapter endpoint in Python

Add `POST /ai/analyze/rust` that wraps the existing endpoint and returns the Rust-expected format. Isolates the concern but adds complexity.

---

## Recommended Fix Plan

**Priority 1 (Immediate - fixes the crash)**:
- Fix `AnalysisResponse` in `rust-core-engine/src/market_data/analyzer.rs` to match Python response schema (Option A)
- Fix `StrategyContext` field serialization: `active_strategies` → `selected_strategies`
- Update `combine_signals()` and downstream code to handle `String` signal instead of enum, or add a mapping layer

**Priority 2 (Follow-up)**:
- Investigate Binance 403 rate limiting on VPS — may need exponential backoff or IP rotation
- Add better error logging: log the actual response body text when deserialization fails (use `response.text().await?` before parsing)
- Add a health check that verifies the Python `/ai/analyze` response schema at startup

---

## Unresolved Questions

1. When did the Python `AISignalResponse` schema diverge from what Rust expects? Was there a Python service update that changed the response format without a corresponding Rust update?
2. The Binance 403 errors — is this testnet rate limiting (IP-based) or an API key issue? The `REQUEST_DELAY_MS` delay and rate limiter should prevent this.
3. Are there other Rust endpoints that call Python AI service with the same schema mismatch (e.g., `strategy_engine.rs`)?
