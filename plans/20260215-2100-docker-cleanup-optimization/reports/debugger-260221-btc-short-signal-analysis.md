# Debugger Report: BTC/USDT SHORT Signal at $67,720

**Date**: 2026-02-21
**Issue**: Unexpected SHORT signal generated for BTC/USDT around $67,720
**Severity**: Medium (paper trading only, no real money lost)

---

## Executive Summary

A SHORT trade was executed at **03:02:14 UTC** on 2026-02-21 for BTCUSDT at **$67,720.06** (signal price $67,772.49) with **74% confidence** and **10x leverage**.

Root cause: **grok-4-1-fast via python-ai-service** returned a genuine SHORT signal with 74.5% confidence at price ~$67,881 approximately 1-2 minutes before execution. The signal was cached in MongoDB, then retrieved and dispatched by the mcp-server periodic poller, triggering execution. The `trade-guardian` cron job (which ran at 03:00 UTC) is correlated in timing but NOT the direct cause - it only ran `get_paper_portfolio`, `get_paper_open_trades`, `get_paper_closed_trades`, `get_paper_basic_settings` (read-only botcore calls, no signal injection).

---

## Technical Timeline

| Time (UTC) | Event |
|---|---|
| ~02:58-03:00 | python-ai-service periodic cycle fetches BTC candles, price=$67,881.80 |
| ~03:00:00 | grok-4-1-fast returns: `signal=Short, 4H=BEARISH, bull_score=0%, bear_score=70% → confidence=0.74` |
| 03:00:00.807 | Analysis stored to MongoDB: doc `699920181967fc44464b5b0e` |
| 03:00:02 | trade-guardian cron fires (health-check + loss-analysis + trade-guardian sessions) |
| 03:00:09 - 03:00:34 | trade-guardian agent (grok-4-1-fast, session a17ce26b) runs 4 exec tool calls (read-only: portfolio, open_trades, closed_trades, settings) |
| 03:00:39 | trade-guardian session completes (no trades placed, no signals sent) |
| 03:02:14 | rust-core-engine mcp-server periodic AI poller calls `/ai/analyze` for BTCUSDT |
| 03:02:14.765 | rust engine broadcasts BTCUSDT SHORT signal via WebSocket |
| 03:02:14.767 | paper_trading engine receives BTCUSDT SHORT 74% — passes confidence check (74% >= 50%) |
| 03:02:14.871 | **Trade executed: Short 0.02952... BTCUSDT @ $67,720.06 (signal: $67,772.49), 10x** |
| 03:02:15 - 03:02:50 | 6+ more SHORT signals received and rejected ("Maximum positions reached") |
| 03:10+ | Signals revert to NEUTRAL (50% confidence) |

---

## Root Cause Analysis

### Why did the SHORT signal occur?

**grok-4-1-fast** (the AI model used by python-ai-service) analyzed BTCUSDT at price **$67,881** and concluded:

```
signal=Short, 4H=BEARISH, bull_score=0.0%, bear_score=70.0% → confidence=0.74
Strategy scores: RSI=0.6, MACD=0.7, Volume=0.6, Bollinger=0.7, Stochastic=0.6
```

This was a **legitimate AI signal** — the model saw a 4H bearish trend and all 5 technical indicators tilted bearish (0.6-0.7). The weighted confidence formula: `bear_score=70%` + `4H=BEARISH` → pushed confidence to 0.745.

The signal was then polled by the mcp-server/rust-engine AI signal poller ~2 minutes later (when price had dipped ~$150 to $67,720) and executed.

### Why the execution was "at market bottom"

- Signal was generated at $67,881 (bearish conditions visible at that price)
- By execution time (03:02:14), price had slipped to $67,720 ($161 lower)
- This $150 gap between signal price ($67,772.49 recorded) and fill ($67,720.06) suggests the market had already moved down briefly before stabilizing/recovering

### Contributing factors

1. **Stale signal execution gap**: ~2min lag between signal generation and execution. Signal generated at ~$67,881 but filled at $67,720 — a $161 gap. The rust engine executed a signal that was stale by ~2 minutes.

2. **4H trend misread**: The 4H trend was flagged as BEARISH, but BTC was actually in a consolidation/recovery zone at ~$67,700-$67,900. The 4H bearish label likely reflected the broader downtrend from higher levels, not the immediate price action.

3. **Signal deduplication gap**: The same SHORT signal (74%, cached in MongoDB) was dispatched **multiple times** to the rust engine within the 03:02-03:02:50 window (6+ repeat signals). Only the first succeeded; rest rejected as "Maximum positions reached." This indicates the mcp-server poller was hammering the same cached analysis.

4. **trade-guardian cron ran at 03:00**: Coincidentally timed with the short signal, but the guardian only ran read-only commands. No causal link. The cron completing and causing extra API load on the python-ai-service during its periodic cycle is a weak contributing factor at most.

---

## Signal Generation Chain

```
grok-4-1-fast (xai API)
    → python-ai-service periodic analysis cycle
    → MongoDB cache (doc: 699920181967fc44464b5b0e)
    → mcp-server AI signal poller (POST /ai/analyze)
    → rust-core-engine API broadcast
    → paper_trading engine execution
    → Short 0.02952 BTCUSDT @ $67,720.06
```

The python-ai-service itself is NOT at fault — it correctly evaluated technical indicators and the 4H trend. The AI's assessment was internally consistent. The question is whether 4H=BEARISH + 5 indicator scores of 0.6-0.7 should have generated a SHORT at ~$67,800 given broader market context.

---

## Current State (as of report time ~03:28 UTC)

- Signal has reverted to NEUTRAL (50% confidence) — all subsequent signals since 03:10 are NEUTRAL
- The SHORT position at $67,720 remains OPEN (subject to engine's auto SL/TP)
- Open positions restored from previous session include multiple BTCUSDT SHORTs at $66,642-$67,160 and LONGs at $66,878-$66,922 — this portfolio has conflicting positions
- Binance client experiencing rate limits (403 Forbidden) — recurring with retry backoff
- Market data cache running repeated historical candle fetches (500 candles per symbol/timeframe) — unusually high fetch rate

---

## Issues Found

### Issue 1: Signal staleness (2-min lag)
The rust engine executed a signal generated ~2 minutes earlier. At $67,881 → $67,720 = $161 drop in 2 min. A freshness check should be implemented.

**Recommendation**: Add signal age check — reject signals older than 60s for volatile assets.

### Issue 2: Duplicate signal dispatching
The same SHORT signal was sent 6+ times in rapid succession (03:02:14 to 03:02:50). The position limit guard stopped all but the first execution, but this indicates a polling loop without deduplication.

**Recommendation**: Deduplicate AI signal dispatch by (symbol, signal, timestamp_minute) before sending to engine.

### Issue 3: Conflicting open positions
Engine state shows simultaneously open BTCUSDT SHORTs ($66,642-$67,160) AND LONGs ($66,878-$66,922) from prior to the container restart at ~16:03 UTC Feb 20. This is a hedging situation that may be intentional but is unclear.

**Recommendation**: Review position conflict policy — should LONGs be closed before opening SHORT or vice versa?

### Issue 4: High rate limiting from Binance (403)
Frequent `Rate limited (403 Forbidden)` warnings with retry. The repeated historical candle fetching (every ~5s for all symbols/timeframes) appears to be causing the rate limit.

**Recommendation**: Add exponential backoff + cache deduplication for historical candle fetches.

### Issue 5: Auth failures from mcp-server
`admin@botcore.local` login fails repeatedly with "user not found" — happening in bulk at each cron cycle startup. This suggests the botcore-bridge is retrying auth with incorrect credentials.

**Recommendation**: Fix `BOTCORE_EMAIL`/`BOTCORE_PASSWORD` env vars in mcp-server container, or pre-create the admin@botcore.local user.

---

## Unresolved Questions

1. What was the exact BTC price at 03:02:14 UTC on the actual market? (The $67,720 fill vs $67,881 generation price suggests either the market had dipped or there's slippage modeling in the paper engine.)

2. Why did the same BTCUSDT SHORT analysis get dispatched 6+ times in ~36 seconds (03:02:14 to 03:02:50)? What is the polling interval of the mcp-server AI signal poller?

3. Is the simultaneous LONG+SHORT position on BTCUSDT intentional (delta-neutral hedging) or an engine bug? The restored positions from Feb 20 include both directions.

4. The `market_data::analyzer` was generating `Failed to analyze: error decoding response body` errors at container start (16:03-16:09 UTC). Did this cause any stale or corrupted signal data to persist?
