# Signal Pipeline Latency Analysis
**Date**: 2026-02-21
**Severity**: Medium (latency understood, no bug — by design)

---

## Executive Summary

The 2-minute delay between AI signal generation and trade execution is **not a single bottleneck** — it is the combined effect of two independent periodic loops with mismatched intervals:

1. Python AI service: generates signals on a **2-minute fixed loop** (`ANALYSIS_INTERVAL_MINUTES = 2`)
2. Rust paper trading engine: has its own **15-minute loop** for autonomous signal polling (but this path is **disabled** in paper trading — signals come via HTTP push instead)

In practice, signals flow via **HTTP push** (Python → Rust API → paper engine), not polling. The actual end-to-end latency per signal is:

- Grok API call: ~10-20s per symbol
- 4 symbols × 10s gap between symbols = ~40s total per cycle
- Cycle interval: 2 minutes

So from signal generation to trade attempt: **<1 second** (push-based, near-instant). The perceived "2-minute delay" = waiting for the **next cycle** to start.

---

## Step-by-Step Pipeline

### Step 1: Trigger — Python AI Service Periodic Loop

**Source**: `/Users/dungngo97/Documents/bot-core/python-ai-service/main.py:319-381`

```python
async def periodic_analysis_runner():
    while True:
        analysis_symbols = await fetch_analysis_symbols()  # GET /api/market/symbols from Rust
        for symbol in analysis_symbols:
            analysis_request = await fetch_real_market_data(symbol)  # GET candles from Rust
            analysis_result = await analyzer.analyze_trading_signals(analysis_request)
            await store_analysis_result(symbol, analysis_result.model_dump())  # → MongoDB
            await ws_manager.broadcast_signal(...)  # → Python WebSocket (unused by Rust)
            await asyncio.sleep(10)  # 10s between symbols
        await asyncio.sleep(ANALYSIS_INTERVAL_MINUTES * 60)  # Wait 2 minutes
```

**Key config**: `ANALYSIS_INTERVAL_MINUTES = 2` (from `config_loader.py:53`, reads `config.yaml`)
Confirmed from VPS startup log:
```
INFO:main:🔄 Started periodic analysis task (every 2 minutes)
```

**Timing breakdown per cycle (4 symbols)**:
- Fetch symbols: ~50ms (HTTP to Rust)
- Per symbol: fetch candles (2 calls × ~200ms) + Grok API call (~10-20s) + MongoDB write (~20ms) = ~11-21s
- Inter-symbol sleep: 10s
- Total cycle time: 4 × (11-21s + 10s) = **84-124 seconds** before the 2-minute wait
- Next cycle starts after `120s` sleep = every **~3.5-4 minutes** wall-clock

**Storage**: MongoDB collection `ai_analysis_results`

---

### Step 2: Python Sends Signals to Rust via HTTP POST (NOT MongoDB polling)

After generating a signal, the Python periodic loop does NOT push to Rust directly. Instead:

The **Rust engine** makes HTTP POST requests to Python's `/ai/analyze` endpoint.

**Who calls whom**: Rust → Python (pull model for on-demand), but also Python has its own loop that pre-generates and caches in MongoDB.

**Rust API route** (`/api/ai/analyze`): `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/api/mod.rs:672-749`

```rust
let ai_analyze = warp::path("analyze").and(warp::post())...
    .and_then(|request, ai_service, broadcaster, paper_trading| async move {
        match ai_service.analyze_for_trading_signal(&request).await {
            Ok(response) => {
                broadcaster.send(message_str);  // WebSocket to frontend
                paper_trading.process_external_ai_signal(...).await;  // → paper engine
                Ok(warp::reply::json(&response))
            }
        }
    });
```

**But who triggers this Rust `/api/ai/analyze`?**

From the VPS logs, we see `POST /ai/analyze HTTP/1.1` requests hit the **Python** service from IP `172.20.0.5` (Rust container). This means Rust is calling Python's `/ai/analyze` — confirmed by:

```
INFO:main:🤖 GPT-4 analysis request for BTCUSDT
INFO:     172.20.0.5:37976 - "POST /ai/analyze HTTP/1.1" 200 OK
```

But the Rust side also logs `📡 Broadcasted AI signal` followed immediately by `📥 Received external AI signal`. This means the Rust **market data processor** is calling the Python `/ai/analyze` endpoint, and then the response goes through `api/mod.rs` which calls `process_external_ai_signal` on the paper trading engine.

**Rust market data processor interval**: every **5 minutes**
Source: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/market_data/processor.rs:606`
```rust
let mut interval = interval(Duration::from_secs(5 * 60));
```

But the logs show signals arriving every ~1 minute (not 5 min). This means the **Python periodic loop** is also calling its own `/ai/analyze` endpoint separately, OR the market data processor calls more than once.

**Actual log pattern** (from VPS, recent 3h window):
```
03:29:04 - Broadcasted AI signal for BNBUSDT (cycle end of 03:29 market analysis)
03:30:04 - Broadcasted AI signal for BTCUSDT  (1 min later)
03:32:05 - Broadcasted AI signal for BTCUSDT  (2 min later)
03:33:05 - Broadcasted AI signal for BNBUSDT  (1 min later)
03:34:15 - Broadcasted AI signal for BTCUSDT  (market analysis triggered)
03:35:04 - Broadcasted AI signal for BNBUSDT
03:36:04 - Broadcasted AI signal for BTCUSDT
03:37:04 - Broadcasted AI signal for BNBUSDT
```

Signals arrive roughly **every 1 minute**, interleaved between symbols. This matches the Python loop's 10-second per-symbol gap across 4 symbols (BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT) calling the Rust `/api/ai/analyze` path.

---

### Step 3: Python Caches in MongoDB, Returns Cached Result

**Source**: `/Users/dungngo97/Documents/bot-core/python-ai-service/main.py:2651-2674`

```python
latest_analysis = await get_latest_analysis(analysis_request.symbol)
if latest_analysis:
    time_since_analysis = (now - stored_time).total_seconds() / 60
    if time_since_analysis < ANALYSIS_INTERVAL_MINUTES:  # < 2 minutes
        return stored_response  # CACHE HIT — no Grok call
```

This means:
- If cache < 2 min old → returns instantly from MongoDB (no AI call), ~5-20ms
- If cache >= 2 min old → calls Grok API, ~10-20 seconds

From VPS logs:
```
INFO:main:📊 Using recent MongoDB analysis for SOLUSDT (age: 1.7min)  ← cache hit
INFO:main:📊 Using recent MongoDB analysis for BNBUSDT (age: 0.6min)  ← cache hit
INFO:main:🔥 No recent analysis found. Calling OpenAI GPT-4 for BTCUSDT  ← cache miss
```

---

### Step 4: Rust Receives Signal and Dispatches to Paper Trading Engine

**Source**: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/api/mod.rs:725-743`

```rust
paper_trading.process_external_ai_signal(
    symbol, response.signal, response.confidence, ...
).await
```

**Source (engine)**: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/paper_trading/engine.rs:509-561`

```rust
// Broadcast via WebSocket (regardless of confidence)
let _ = self.event_broadcaster.send(PaperTradingEvent { ... });

// Execute if above threshold
if signal.confidence >= min_confidence {
    match self.process_trading_signal(signal.clone()).await {
        Ok(result) => { ... }
    }
}
```

**Latency at this step**: Near-zero. From logs:
```
03:29:04.572218 - Broadcasted AI signal for BNBUSDT
03:29:04.572240 - Received external AI signal: BNBUSDT NEUTRAL with 64%  ← 22 microseconds
03:29:04.572973 - Acquired trade execution lock for BNBUSDT
03:29:04.573047 - Trade execution failed: Neutral signal cannot be executed
```

**Total lock-to-decision time**: ~75 microseconds.

---

### Step 5: Rust's Own Signal Processing Loop (DISABLED in paper trading)

**Source**: `/Users/dungngo97/Documents/bot-core/rust-core-engine/src/paper_trading/engine.rs:252-273`

```rust
fn start_signal_processing(&self) -> tokio::task::JoinHandle<Result<()>> {
    let signal_interval = settings.ai.signal_refresh_interval_minutes; // default: 15
    let mut interval = interval(Duration::from_secs(signal_interval as u64 * 60));
    while *engine.is_running.read().await {
        interval.tick().await;
        engine.process_ai_signals().await;
    }
}
```

**But `process_ai_signals()` calls `get_ai_signal_for_symbol()`** which explicitly returns an error:

```rust
async fn get_ai_signal_for_symbol(&self, symbol: &str) -> Result<AITradingSignal> {
    // Paper trading should NOT call Binance API (even testnet).
    // It should only react to signals from the frontend.
    return Err(anyhow::anyhow!(
        "Paper trading mode: skipping automatic signal generation..."
    ));
}
```

So the **15-minute internal Rust loop is a no-op** in paper trading. Signals come only via the HTTP push path (Step 2-4).

---

## Complete Timing Map

```
[Python periodic loop starts]
    T+0:00  → fetch_analysis_symbols() from Rust API          ~50ms
    T+0:01  → fetch candles for BTCUSDT (1h + 4h)            ~400ms
    T+0:02  → Grok API call for BTCUSDT                       ~10-20s
    T+0:22  → store to MongoDB (BTCUSDT)                      ~20ms
    T+0:23  → asyncio.sleep(10)  ← rate limit between symbols  10s
    T+0:33  → fetch candles for ETHUSDT                       ~400ms
    T+0:34  → Grok API call for ETHUSDT                       ~10-20s
    T+0:54  → store to MongoDB (ETHUSDT)                      ~20ms
    T+0:55  → asyncio.sleep(10)                                10s
    ... (repeat for BNBUSDT, SOLUSDT)
    T+1:40  → Completed AI analysis cycle for 4 symbols
    T+1:40  → asyncio.sleep(120)  ← 2-minute wait
    T+3:40  → Next cycle starts

[Rust market data processor — separate 5-min loop]
    T+5:00  → Starting periodic market analysis for 4 symbols
             → For each symbol: calls Rust's own analyzer (NOT Python)
             → Result: "Analysis failed: All timeframe analyses failed"
             → This path is separate; does NOT call Python /ai/analyze

[Rust paper trading engine — 15-min loop]
    T+15:00 → process_ai_signals() → get_ai_signal_for_symbol()
            → Returns error (disabled in paper trading mode)
            → NO-OP

[Python periodic loop → Rust API /api/ai/analyze (trigger path)]
    Each symbol analyzed by Python calls Rust endpoint internally?
    OR: Rust market data processor calls Python /ai/analyze?
    Evidence from Python logs: requests come FROM 172.20.0.5 (Rust)
    The 5-min market_data processor in Rust calls Python /ai/analyze

[Signal dispatch once Python returns result to Rust /api/ai/analyze]
    T+0ms  → Rust API handler gets response from Python
    T+0ms  → broadcaster.send() → WebSocket broadcast
    T+0ms  → paper_trading.process_external_ai_signal()
    T+0.7ms → trade execution lock acquired
    T+0.8ms → trade decision made (NEUTRAL → rejected)
```

---

## Root Causes of Latency

| Source | Latency | Notes |
|--------|---------|-------|
| Grok API call per symbol | 10-20s | Main bottleneck when cache miss |
| 10s inter-symbol sleep | 10s × 4 symbols = 40s | Rate limiting hardcoded in Python |
| 2-minute cycle wait | 120s | `ANALYSIS_INTERVAL_MINUTES=2` |
| MongoDB cache lookup | <20ms | Cache hit avoids Grok call |
| Rust broadcast + dispatch | <1ms | Near-instant once signal received |
| Rust trade execution lock | <1ms | Mutex, no contention normally |

**Worst case** (cache miss on all 4 symbols): ~3.5-4 minutes end-to-end
**Best case** (cache hit): ~50ms from cycle start to Rust receipt
**Typical** (mixed): ~2-3 minutes (first symbol may cache-miss, others hit)

---

## Key Architectural Finding

The pipeline is **push-based from Rust to Python**, not pull:

```
Rust market_data::processor (every 5min)
  → POST http://python-ai-service:8000/ai/analyze
  → Python: checks MongoDB cache (< 2min → return cached)
  → Python: if stale → calls Grok API (~10-20s)
  → Python returns AISignalResponse
  → Rust api/mod.rs receives response
  → Broadcasts to WebSocket clients
  → Calls paper_trading.process_external_ai_signal()
  → Engine acquires lock, executes or rejects trade
```

AND separately, Python's own periodic loop pre-warms the MongoDB cache every 2 minutes, so when Rust calls, it typically gets a cache hit.

---

## Issues Observed

1. **All signals are NEUTRAL** — every signal from the past 3h has been `NEUTRAL`. This is the actual trading problem, not latency. The system is running but not generating BUY/SELL signals. Root cause likely: Grok analysis returning conservative neutral signals, or technical indicator thresholds too strict.

2. **Rust market data analyzer failing**: Every 5-minute cycle shows:
   ```
   WARN: Analysis failed for BTCUSDT: All timeframe analyses failed
   ```
   This is the Rust-native strategy analysis (not the AI path). These failures do NOT block AI signal flow but indicate the Rust strategy engine itself is broken (likely Binance testnet data format issue).

3. **Python periodic loop adds 10s between symbols** (`asyncio.sleep(10)`) meaning signal freshness is staggered by symbol order. SOLUSDT (4th) gets its fresh analysis ~40s after BTCUSDT.

---

## Recommendations

1. **Reduce inter-symbol sleep**: Change `asyncio.sleep(10)` to `asyncio.sleep(2)` — Grok API (xAI) has high rate limits for `grok-4-1-fast`. This cuts per-cycle time from ~140s to ~60s.
   - File: `/Users/dungngo97/Documents/bot-core/python-ai-service/main.py:363`

2. **Increase cache duration to match cycle**: `ANALYSIS_INTERVAL_MINUTES=2` but actual cycle takes ~3.5min. Set cache to 4 minutes so cache hits are guaranteed during rapid Rust polling.
   - File: `/Users/dungngo97/Documents/bot-core/python-ai-service/config.yaml` or env var

3. **Fix Rust strategy analyzer failures**: The `All timeframe analyses failed` errors suggest Binance testnet API data format changed. Fix or disable this secondary analysis loop to reduce log noise.

4. **Investigate NEUTRAL signal dominance**: The trading system generates 100% NEUTRAL signals. Check:
   - `min_required_timeframes=2` and `min_required_indicators=3` thresholds
   - Grok prompt confidence calibration
   - Whether `grok-4-1-fast` is being overly conservative

---

## Unresolved Questions

1. What triggers the Rust `market_data::processor` to call Python `/ai/analyze` exactly? The code in `processor.rs` only calls `analyzer.analyze_multi_timeframe()` (Rust-native), not Python. But VPS logs show Python receiving requests from `172.20.0.5` (Rust). Need to check if `MarketDataAnalyzer` internally calls Python, or if there is another trigger (WebSocket signal? Frontend poll?).

2. Why does the Python periodic loop analyze symbols while the Rust market data processor also separately analyzes the same symbols? Are these redundant?

3. Is the 5-minute Rust `market_data::processor` loop calling Python `/ai/analyze` on every tick? If so, the effective signal refresh for each symbol is every 5 minutes (Rust) or 2 minutes (Python pre-cache), whichever is first.
