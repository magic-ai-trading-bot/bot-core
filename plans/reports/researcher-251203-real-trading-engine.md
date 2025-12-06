# Research Report: Real Trading Engine Implementation with Binance API

**Date**: 2025-12-03
**Research Period**: 2024-2025
**Focus**: Phase 2 Real Trading Implementation for Rust Crypto Trading Bot

---

## Executive Summary

Real trading engines require careful state management, thread-safe concurrent order tracking, and robust error recovery. Key architectural patterns: (1) **Arc<Mutex<T>> + DashMap** for position/order state with lock-free lookups; (2) **Tokio actor model** with message passing for event-driven order processing; (3) **WebSocket reconciliation** with REST API fallback; (4) **ExecutionReport event streaming** for real-time fill tracking; (5) **Circuit breaker + exponential backoff** for error recovery. Critical: partial fill handling, timeout resilience, and order reconciliation before any execution.

---

## Key Findings

### 1. State Management & Concurrency Architecture

**Best Pattern**: **Arc<Mutex<T>> for shared state + DashMap for concurrent lookups**

Binance/crypto trading systems consistently use:
- **Arc** (Atomic Reference Counting) for thread-safe shared ownership across async tasks
- **Mutex** for position/portfolio state (read-heavy with occasional writes)
- **DashMap** for fast concurrent order lookups by ID (lock-free hash operations)
- **RwLock** for price map (read-heavy structures)
- **BTreeMap** for price level hierarchy (sorted best bid/ask)

**Why not alternatives?**
- Rc<T>: Not Send/Sync—only single-threaded
- HashMap + Mutex: Contention bottleneck under concurrent order updates
- DashMap alone: No protection for composite state (order + position atomicity)

**Code Structure**:
```rust
pub struct RealTradingEngine {
    // Shared state across all async tasks
    positions: Arc<Mutex<HashMap<Symbol, Position>>>,

    // Fast concurrent lookups for order queries
    orders: DashMap<OrderId, OrderState>,

    // Event stream for WebSocket updates
    execution_rx: tokio::sync::mpsc::Receiver<ExecutionReport>,
}
```

---

### 2. Order Lifecycle & Partial Fill Handling

**Binance ExecutionReport states**:
- **NEW** → Order accepted, not yet matched
- **TRADE** → Partial or full fill occurred
- **CANCELED** → User-initiated cancel
- **REJECTED** → Order not processed
- **EXPIRED** → Auto-canceled (FOK order with no fill, IOC with partial, liquidation)

**Partial Fill Reality**:
- Market orders with low liquidity = EXPIRED (partial fill)
- FOK (Fill or Kill) = all-or-nothing required
- IOC (Immediate or Cancel) = fill what's available, cancel remainder
- GTC (Good Till Canceled) = default, partial fills OK

**Implementation Strategy**:
1. Track `executedQty` vs `origQty` from every ExecutionReport
2. If `executedQty < origQty`, order is PARTIALLY_FILLED
3. Cancel PARTIALLY_FILLED positions with `cancelRestrictions: ONLY_PARTIALLY_FILLED`
4. Use FOK for "all-or-nothing" requirements
5. Handle timeout (10s API limit): query ExecutionReport via WebSocket; if missing, REST query order status

**Risk**: Binance API timeout (-1007) doesn't mean failed—check User Data Stream first, then REST query.

---

### 3. Thread-Safe Position Management

**Architecture for concurrent updates**:

```rust
// Protects overall position (read-heavy, occasional updates)
pub positions: Arc<Mutex<HashMap<Symbol, Position>>>,

// Fast concurrent order tracking (frequent reads/writes)
pub orders: DashMap<OrderId, Arc<Mutex<OrderMetadata>>>,

// Real-time event channel from WebSocket
pub execution_rx: mpsc::Receiver<ExecutionReport>,
```

**Lock Strategy**:
- Acquire position lock ONCE per ExecutionReport, not per order lookup
- Use DashMap for quick order ID → metadata (no lock)
- Update position atomically: lock → read current qty → update → write balance → unlock

**Concurrency Issues to Avoid**:
- Lock order → then lock position (deadlock risk if reversed elsewhere)
- Holding position lock during external API calls (timeouts)
- Race condition: ExecutionReport arrives → update position → balance query returns stale data

**Solution**: Event-driven updates. WebSocket ExecutionReport → update position/orders → emit position_updated event. Queries read current state (always fresh).

---

### 4. Error Recovery & Resilience

**Multi-layer recovery strategy**:

**Layer 1: WebSocket Disconnect**
- Implement exponential backoff (1s, 2s, 4s, 8s max)
- Maintain listenKey (User Data Stream token, valid 1hr)
- Store last processed event_id to detect message gaps
- Fallback: REST API query on reconnect

**Layer 2: Order Reconciliation**
- After reconnect: fetch open orders from REST API
- Compare with local order state (DashMap)
- If REST has order bot doesn't: replay ExecutionReport via REST historical query
- If bot has order REST doesn't: assume filled; check balance

**Layer 3: Circuit Breaker**
- Pause trading if WebSocket offline >30s or 3+ consecutive API errors
- Resume only after clean reconnect + balance reconciliation

**Layer 4: Timeout Handling**
- 10s API timeout doesn't mean failed—query User Data Stream for ExecutionReport
- If no event: REST query order status
- Never assume order is gone without explicit confirmation

---

### 5. ExecutionReport Integration

**WebSocket Event Stream** (User Data Stream):
- Real-time ExecutionReport, BalanceUpdate, OutboundAccountPosition
- Order fill granularity: executedQty, executedPrice per TRADE event
- Position updates: absolute balances in OutboundAccountPosition

**Integration Pattern**:
```rust
// WebSocket handler
while let Some(ExecutionReport { event_type, executed_qty, executed_price, order_status, ... }) = stream.next().await {
    // Update order state
    orders.insert(order_id, OrderState::new(order_status, executed_qty));

    // Update position (lock once)
    let mut pos = positions.lock().await;
    pos.update_fill(executed_qty * executed_price);
    drop(pos); // explicit unlock before external calls

    // Emit position_updated event for subscribers
    event_bus.send(PositionUpdate { ... });
}
```

**Critical**: Process ExecutionReport sequentially (ordered queue), not in parallel. Out-of-order fills = inconsistent state.

---

## Architecture Recommendation

**Actor Model with Tokio** (recommended for Phase 2):

1. **OrderManagementActor**: Handles order placement/cancellation via Binance REST
2. **ExecutionActor**: Listens to WebSocket ExecutionReport → updates order/position state
3. **PositionActor**: Manages position state (Arc<Mutex>), answers queries
4. **ReconciliationActor**: Periodic checks (every 5min) comparing local vs Binance state
5. **ErrorRecoveryActor**: Monitors WebSocket health, triggers reconnects/reconciliation

**Message-Passing Communication**:
- OrderManagementActor → ExecutionActor: "PlaceOrder { ... }" → receives OrderPlaced event
- ExecutionActor → PositionActor: "UpdatePosition { ... }"
- PositionActor → observers: "PositionChanged { ... }"
- RiskManager → OrderManagementActor: "CancelIfExceeds { ... }"

Benefits: No shared mutable state deadlocks, testable, resilient to failures.

---

## Risk Considerations for Real Money

**Critical** (must handle before production):

1. **Partial Fill Ambiguity**: Order partially fills → market moves → decision to add/cancel problematic
   - Mitigation: Always use FOK for critical trades or implement "completion logic" (auto-add orders)

2. **ExecutionReport Gaps**: WebSocket loses message → position state diverges
   - Mitigation: Reconciliation every 5min + hashsum of expected state

3. **API Timeout Ambiguity**: 10s timeout doesn't confirm success/failure
   - Mitigation: Query User Data Stream first (has all fills), then REST if needed

4. **Concurrent Cancellation**: Bot tries to cancel while Binance is filling
   - Mitigation: Check `cancelRestrictions` in cancel request; handle "OrderNotFound" gracefully

5. **Balance Drift**: Multiple fills in rapid succession → balance query lags
   - Mitigation: Use OutboundAccountPosition event (more reliable than balance query)

6. **Dead Connections**: Long-running bots with "silent" disconnects
   - Mitigation: WebSocket ping/pong keep-alive every 3min + timeout detection

---

## Implementation Checklist for Phase 2

- [ ] Arc<Mutex> pattern for positions + DashMap for orders
- [ ] Tokio mpsc channel for ExecutionReport events (ordered queue, no parallel processing)
- [ ] Order state machine: NEW → PARTIALLY_FILLED → FILLED / CANCELED / EXPIRED
- [ ] Exponential backoff reconnection (1s, 2s, 4s, 8s) with listenKey renewal
- [ ] REST API fallback for order queries on WebSocket outage
- [ ] ReconciliationJob: compare local orders vs `GET /openOrders` every 5min
- [ ] Circuit breaker: pause trading on 30s+ offline or 3+ consecutive errors
- [ ] Timeout handling: query User Data Stream → REST query on no response
- [ ] ExecutionReport hashsum validation (detect missed events)
- [ ] Graceful shutdown: cancel all orders, close positions, save state

---

## References

### Rust Concurrency & Order Management
- [Building Concurrent Orderbook in Rust (Oct 2025)](https://medium.com/@jayakrishnanashok/building-a-concurrent-orderbook-in-rust-lessons-from-a-multi-threaded-challenge-3905be468e18)
- [Barter-rs: Event-Driven Trading Framework](https://github.com/barter-rs/barter-rs)
- [OrderBook-rs: Thread-Safe Matching Engine](https://github.com/joaquinbejar/OrderBook-rs)
- [Arc & Mutex Thread Safety (Ferrous Systems)](https://rust-training.ferrous-systems.com/latest/book/thread-safety)
- [Shared-State Concurrency (Rust Book)](https://doc.rust-lang.org/book/ch16-03-shared-state.html)

### Binance API Order Lifecycle
- [Order Update Events (Binance Docs)](https://developers.binance.com/docs/margin_trading/trade-data-stream/Event-Order-Update)
- [Trading Endpoints (Binance Spot API)](https://developers.binance.com/docs/binance-spot-api-docs/rest-api/trading-endpoints)
- [Partial Fill Handling (Stack Overflow)](https://stackoverflow.com/questions/71991754/with-the-python-binance-api-my-limit-order-is-only-partially-filled)

### Error Recovery & WebSocket Patterns
- [Freqtrade FAQ: Order Reconciliation](https://www.freqtrade.io/en/stable/faq/)
- [WebSocket Reliability in Trading (Medium)](https://konstantinmb.medium.com/how-to-utilize-websockets-in-creating-a-profitable-trading-bot-with-python-5cb840e6c753)
- [Hyperliquid Trading Bot: Error Handling](https://chainstack.com/how-to-build-a-hyperliquid-trading-bot/)
- [SockTrader: WebSocket-Based Bot](https://github.com/SockTrader/SockTrader)

### Tokio Actor Model
- [Building Actors with Tokio (Alice Ryhl)](https://ryhl.io/blog/actors-with-tokio/)
- [Tokio Async Actor Model (Medium)](https://medium.com/@p4524888/leveraging-rusts-tokio-library-for-asynchronous-actor-model-cf6d477afb19)
- [Actix Framework](https://github.com/actix/actix)
- [Async Rust Book: The Actor Model (O'Reilly)](https://www.oreilly.com/library/view/async-rust/9781098149086/ch08.html)

---

## Unresolved Questions

1. **Latency tolerance**: What's acceptable round-trip time for order placement (REST vs WebSocket)?
2. **State machine serialization**: How to persist order state for recovery across restarts?
3. **Binance API rate limits**: How to structure requests to stay within 1200 req/min limit during reconnects?
4. **Multi-pair coordination**: How to order place requests across 10+ pairs without locks?
5. **Position hedging**: How to track cross-pair correlations for risk management?

---

**End of Report**
