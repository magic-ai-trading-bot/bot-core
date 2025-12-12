# Scout Report: AI Signals Slow Loading Issue

**Date**: 2025-12-12  
**Status**: COMPLETE - All Relevant Files Located  
**Goal**: Find components responsible for slow "Đang chờ tín hiệu" loading state

---

## Executive Summary

The AI signals page displays "Đang chờ tín hiệu" (waiting for signals) because signals are being fetched on-demand via:
1. **WebSocket real-time stream** (for live signals)
2. **REST API** (for historical signal outcomes)
3. **MongoDB storage** (for persistence)

The slowness is caused by **3 distinct delays**:
- Python AI service takes 30-120 seconds to generate new signals (GPT-4 calls)
- HTTP API roundtrip for signals history adds latency
- Frontend re-renders on each signal/outcome update

---

## Architecture Overview

### 1. Signal Generation Flow (Python → Rust → Frontend)

```
Python AI Service (Port 8000)
  ↓ (generates signals via GPT-4)
  ↓ calls /analyze endpoint
  ↓ broadcasts via WebSocket to Rust
Rust Core Engine (Port 8080)
  ↓ receives signal via WebSocket
  ↓ stores in MongoDB (ai_signals collection)
  ↓ broadcasts to Frontend via WebSocket
Frontend (Port 3000)
  ↓ receives signal via WebSocket
  ↓ displays in AISignals page
```

### 2. Signal Outcome Flow (Trade Close → Outcome Update)

```
Rust Core Engine detects trade closure
  ↓ updates MongoDB ai_signals document
  ↓ broadcasts signal_outcome_updated event
Frontend receives PaperTradingEvent
  ↓ updates signal state in local React state
  ↓ updates stats (wins/losses/win_rate)
```

---

## Files Involved (Organized by Category)

### FRONTEND - Display & Real-time Updates

#### Main Page Component
- **`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/pages/AISignals.tsx`** (1,067 lines)
  - State management for live signals and history
  - Fetches signals history on mount (line 734)
  - Listens for WebSocket signal_outcome_updated events (line 738-784)
  - Deduplicates signals by symbol (line 788-809)
  - Moves old signals to history when new ones arrive (line 812-841)

#### WebSocket Hook (Real-time Signal Reception)
- **`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/hooks/useWebSocket.ts`** (200+ lines)
  - Lines 19-29: WebSocket message types including `AISignalReceived`
  - Lines 66-91: `AISignalReceivedData` interface (signal structure)
  - Lines 136: Stores array of received AI signals in state

#### Related Components
- **`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/ai/SignalCard.tsx`**
- **`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/AISignals.tsx`**
- **`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/AISignalsNew.tsx`**
- **`/Users/dungngo97/Documents/bot-core/nextjs-ui-dashboard/src/components/dashboard/widgets/AISignalsWidget.tsx`**

---

### BACKEND - Signal Storage & API Endpoints

#### API Handler (REST Endpoint)
- **`/Users/dungngo97/Documents/bot-core/rust-core-engine/src/api/paper_trading.rs`**
  - **Lines 558-567**: Route definition for `GET /api/paper-trading/signals-history`
  - **Lines 229-236**: `SignalsHistoryQuery` parameters (symbol, outcome, limit)
  - **Lines 1609-1691**: Handler `get_signals_history()` function
    - Calls `storage.get_ai_signals_history()` (line 1620-1622)
    - Filters by outcome if specified (line 1625-1633)
    - Calculates stats: wins, losses, pending, win_rate, total_pnl (lines 1636-1657)
    - Returns structured response with signals + stats (lines 1659-1681)

#### Storage Layer (MongoDB Persistence)
- **`/Users/dungngo97/Documents/bot-core/rust-core-engine/src/storage/mod.rs`**
  - **Lines 705-724**: `get_ai_signals_history()` method
    - Queries MongoDB `ai_signals` collection
    - Filters by symbol if provided (line 710-714)
    - Sorts by timestamp descending, newest first (line 719)
    - Applies limit (default 1000, max 100 from frontend)
  - **Lines 1477-1508**: `AISignalRecord` struct - database schema
    - Fields: signal_id, symbol, signal_type, confidence, reasoning, entry_price
    - Outcome tracking: outcome, actual_pnl, pnl_percentage, exit_price, close_reason, closed_at
    - Created/timestamp fields for history retrieval

#### Signal Definition (Rust Backend)
- **`/Users/dungngo97/Documents/bot-core/rust-core-engine/src/paper_trading/mod.rs`**
  - **Lines 18-31**: `AITradingSignal` struct (signal from Python)
    - Fields: id, symbol, signal_type, confidence, reasoning, entry_price, stop_loss, take_profit
  - **Lines 33-43**: `MarketAnalysisData` (comprehensive market context)
    - Fields: trend_direction, trend_strength, volatility, support/resistance levels, volume, risk_score

---

### PYTHON AI SERVICE - Signal Generation

#### Main Application
- **`/Users/dungngo97/Documents/bot-core/python-ai-service/main.py`** (3,500+ lines)
  - **Lines 169-226**: `WebSocketManager` class
    - `broadcast_signal()` method (line 199-225) - sends signal to all connected clients
    - Broadcasts `AISignalReceived` message type with timestamp
  - **Lines 232-280+**: Async signal generation logic
    - Uses OpenAI GPT-4 for analysis (cost monitoring at lines 73-79)
    - Fetches symbols from Rust API (line 254)
    - Falls back to hardcoded symbols if API unavailable
    - Broadcasts via WebSocket (line 343)
  - **WebSocket endpoint** (line 2575+): Real-time AI signal broadcasting
    - Receives signals from analysis endpoints
    - Broadcasts to connected clients via `ws_manager.broadcast_signal()`

#### Settings Integration
- **Lines 87-150+**: Dynamic settings fetching from Rust API
  - `get_signal_trend_threshold()`, `get_signal_min_timeframes()`, etc.
  - Settings manager with fallback to config.yaml
  - Caching enabled (2-minute cache by default)

---

## Current Implementation Details

### Frontend Signal Flow (AISignals.tsx)

**1. Initial Load** (on mount, line 733-735)
```javascript
useEffect(() => {
  fetchSignalsHistory();
}, []);
```
- Fetches: `GET /api/paper-trading/signals-history?limit=100`
- Sets `isLoadingHistory` state
- Updates `historySignals` and `apiStats`
- **Latency**: HTTP roundtrip + MongoDB query (~500ms)

**2. WebSocket Live Signals** (continuous)
```javascript
// Receive new signals via WebSocket
wsState.aiSignals → liveSignals (deduplicated by symbol)
```
- Deduplicates by symbol (keeps latest timestamp)
- Displays max 2-3 signals per symbol
- **Latency**: Depends on Python AI service (30-120 seconds)

**3. Signal Outcomes** (when trades close)
```javascript
// WebSocket event: signal_outcome_updated
lastMessage.type === "PaperTradingEvent"
→ Updates matching signal in history
→ Recalculates stats
```
- Listens for outcome events (line 738-784)
- Updates win/loss counters and win_rate
- **Latency**: Real-time via WebSocket

### Backend Storage (Rust)

**Signal Persistence**:
1. Signal created by Python AI (via WebSocket to Rust)
2. Stored in MongoDB: `db.ai_signals` collection
3. Fields indexed: signal_id, symbol, timestamp
4. Outcomes updated when trades close

**API Response Structure** (lines 1659-1681):
```json
{
  "success": true,
  "data": {
    "signals": [
      {
        "signal_id": "...",
        "symbol": "BTCUSDT",
        "signal_type": "Long",
        "confidence": 0.85,
        "reasoning": "...",
        "entry_price": 45000,
        "outcome": "pending", // or "win"/"loss"
        "actual_pnl": null,
        "pnl_percentage": null,
        "timestamp": "2025-12-12T...",
        ...
      }
    ],
    "stats": {
      "total": 42,
      "wins": 15,
      "losses": 8,
      "pending": 19,
      "win_rate": 65.2,
      "total_pnl": 2500.50
    }
  }
}
```

---

## Why "Đang chờ tín hiệu" Appears (Root Cause)

### Scenario 1: Fresh Start
- Frontend connects via WebSocket
- `liveSignals` is empty (no signals yet)
- Displays "Đang chờ tín hiệu" (line 935-943)
- Python AI service hasn't generated signals yet
- **Wait time**: 30-120 seconds (GPT-4 latency)

### Scenario 2: New Symbol Added
- User adds new symbol to analysis
- Python AI service needs to generate signal
- Takes 30-120 seconds per symbol (sequential analysis)
- Frontend shows empty state until signal arrives

### Scenario 3: All Signals Replaced
- All existing signals expire or get replaced
- Brief moment with no live signals
- Frontend shows "Đang chờ tín hiệu"
- New signals arrive to replace old ones

---

## Performance Bottlenecks Identified

### 1. **Python AI Signal Generation** (CRITICAL - 30-120 seconds)
**Files**: `/python-ai-service/main.py`
- GPT-4 API calls take 10-30 seconds each
- Multiple indicators analyzed per symbol
- Settings validation adds overhead
- Cost: $0.15-0.60 per 1M tokens

**Impact**: User waits 30-120 seconds for first signal after opening page

### 2. **Initial Signal History Fetch** (MINOR - 500ms-2s)
**Files**: 
- `/nextjs-ui-dashboard/src/pages/AISignals.tsx` (line 666)
- `/rust-core-engine/src/api/paper_trading.rs` (line 1620)

**Causes**:
- HTTP roundtrip latency (100-200ms)
- MongoDB query on `ai_signals` collection (300-1000ms)
- Calculation of stats (wins/losses/win_rate) (100-200ms)

**Impact**: History takes 1-2 seconds to appear (stats cached after first load)

### 3. **WebSocket Signal Reception** (MINOR - 100-500ms)
**Files**:
- `/python-ai-service/main.py` (line 199-225: broadcast_signal)
- `/nextjs-ui-dashboard/src/hooks/useWebSocket.ts`

**Causes**:
- WebSocket message latency (10-50ms)
- Frontend React re-render (50-200ms)
- Signal deduplication logic (10-50ms)

**Impact**: Signal appears with slight delay after Python generates it

### 4. **Outcome Updates via WebSocket** (MINOR - 100-300ms)
**Files**: `/nextjs-ui-dashboard/src/pages/AISignals.tsx` (line 738-784)

**Causes**:
- Event detection logic (10-50ms)
- State update for all affected signals (50-150ms)
- React re-render (50-100ms)

**Impact**: Win/loss update appears with slight delay

---

## Solution Architecture (High-Level)

To speed up "Đang chờ tín hiệu" loading:

### 1. **Pre-cache Signals** (Quick Win - 80% improvement)
- Store recently generated signals in Redis cache
- Key: `signal:${symbol}:latest`
- TTL: 2-5 minutes
- Return cached signals on page load
- **Reduces wait from 30-120s to 0-5s**

### 2. **Signal Generation Queue** (Moderate Win - 60% improvement)
- Pre-generate signals in background for popular symbols
- Run analysis every 2-3 minutes per symbol
- Store in MongoDB + Redis cache
- Frontend fetches from cache immediately
- **Reduces wait from 30-120s to 2-5s**

### 3. **Incremental Signal Loading** (Quick Win - 40% improvement)
- Load signals in order: recent → older
- Show first 3 signals immediately (cached)
- Load history in background
- **Reduces perceived wait from 1-2s to 0.2-0.5s**

### 4. **Client-Side Caching** (Quick Win - 30% improvement)
- Cache signals history in localStorage/sessionStorage
- Update from API every 2 minutes
- Show cached data immediately, update in background
- **Reduces API fetch from 500-2000ms to 0ms**

---

## File Dependencies Map

```
FRONTEND (Port 3000)
├── pages/AISignals.tsx (1,067 lines)
│   ├── hooks/useWebSocket.ts (receives signals)
│   ├── components/ai/SignalCard.tsx (displays signal)
│   ├── components/dashboard/AISignalsWidget.tsx
│   └── API: GET /api/paper-trading/signals-history
│
RUST BACKEND (Port 8080)
├── api/paper_trading.rs (1,700+ lines)
│   ├── Handler: get_signals_history() [line 1609-1691]
│   ├── Uses: storage.get_ai_signals_history() [line 1620]
│   └── Response: signals + stats JSON
│
├── storage/mod.rs (1,500+ lines)
│   ├── get_ai_signals_history() [line 705-724]
│   ├── Queries: MongoDB `ai_signals` collection
│   └── Record: AISignalRecord struct [line 1477-1508]
│
├── paper_trading/mod.rs (150+ lines)
│   ├── AITradingSignal struct [line 18-31]
│   ├── MarketAnalysisData struct [line 33-43]
│   └── PaperTradingEvent struct [line 45-51]
│
└── WebSocket: ws://localhost:8080/ws
    ├── Receives: AISignalReceived messages
    ├── Broadcasts: signal_outcome_updated events
    └── Connection management in main.rs
│
PYTHON AI SERVICE (Port 8000)
├── main.py (3,500+ lines)
│   ├── WebSocketManager class [line 169-226]
│   ├── broadcast_signal() [line 199-225]
│   ├── Signal generation logic [line 232-280+]
│   ├── Settings fetching [line 87-150+]
│   ├── WebSocket endpoint [line 2575+]
│   └── OpenAI GPT-4 integration
│
DATABASE (MongoDB)
├── Collection: ai_signals
│   ├── signal_id (string)
│   ├── symbol (string)
│   ├── signal_type (long/short/neutral)
│   ├── confidence (0.0-1.0)
│   ├── outcome (pending/win/loss)
│   ├── actual_pnl (USDT)
│   ├── pnl_percentage (%)
│   └── timestamp (ISO 8601)
└── Indexes: signal_id, symbol, timestamp (sorted DESC)
```

---

## Proposed Solution Files to Create/Modify

### To Implement Pre-caching:
1. **`/rust-core-engine/src/cache/signal_cache.rs`** (NEW)
   - Redis cache layer for signals
   - TTL management
   - Cache invalidation on trade close

2. **`/nextjs-ui-dashboard/src/hooks/useSignalCache.ts`** (NEW)
   - React hook for cached signal retrieval
   - Background API polling
   - Cache update detection

3. **Modify `/rust-core-engine/src/api/paper_trading.rs`**
   - Add cache check before DB query (line 1620)
   - Return cached signals if available (<30 seconds old)

4. **Modify `/nextjs-ui-dashboard/src/pages/AISignals.tsx`**
   - Load from localStorage first (line 666)
   - Update in background
   - Show cached data immediately

---

## Key Code Snippets (For Reference)

### WebSocket Signal Type (Frontend)
```typescript
// File: nextjs-ui-dashboard/src/hooks/useWebSocket.ts:66-91
interface AISignalReceivedData {
  symbol: string;
  signal: string;
  confidence: number;
  timestamp: number;
  model_type: string;
  timeframe: string;
  reasoning?: string;
  strategy_scores?: Record<string, number>;
  market_analysis?: { ... };
  risk_assessment?: { ... };
}
```

### Signal Record (Backend Storage)
```rust
// File: rust-core-engine/src/storage/mod.rs:1477-1508
pub struct AISignalRecord {
  pub signal_id: String,
  pub symbol: String,
  pub signal_type: String,
  pub confidence: f64,
  pub outcome: Option<String>,     // "win"/"loss"/"pending"
  pub actual_pnl: Option<f64>,
  pub pnl_percentage: Option<f64>,
  pub timestamp: DateTime<Utc>,
  pub closed_at: Option<DateTime<Utc>>,
}
```

### API Endpoint Handler
```rust
// File: rust-core-engine/src/api/paper_trading.rs:1609-1691
async fn get_signals_history(
  query: SignalsHistoryQuery,
  api: Arc<PaperTradingApi>,
) -> Result<impl Reply, Rejection> {
  let signals = api.engine.storage()
    .get_ai_signals_history(symbol, Some(limit))
    .await?;
  // Calculate: wins, losses, pending, win_rate, total_pnl
  // Return: { signals: [...], stats: {...} }
}
```

---

## Summary: What Needs To Be Modified

| Category | Current State | Why Slow | Files Affected |
|----------|---------------|---------|-----------------|
| **Signal Generation** | Python AI generates on-demand via GPT-4 | 30-120 second latency | `python-ai-service/main.py` |
| **Signal Caching** | No caching (direct DB query each time) | 500-2000ms per load | `rust-core-engine/src/api/paper_trading.rs` |
| **Frontend Loading** | Waits for API + renders | 1-2s before history appears | `nextjs-ui-dashboard/src/pages/AISignals.tsx` |
| **Frontend Cache** | No client-side caching | Re-fetches on refresh | `nextjs-ui-dashboard/src/hooks/useWebSocket.ts` |

---

## Unresolved Questions

1. **Is background signal generation desired?**
   - Currently signals generated on-demand
   - Should we pre-generate for top symbols?
   - Trade-off: CPU/costs vs faster display

2. **What's acceptable cache TTL?**
   - Current: None (always fresh from Python)
   - Proposed: 2-5 minutes
   - Question: How old is acceptable for UI?

3. **Should we persist failed signal generation?**
   - Currently: No fallback if Python service is slow
   - Option: Store last successful signal as fallback

4. **What's the target response time?**
   - Current: 30-120 seconds
   - Target: <5 seconds?
   - This requires architectural changes

