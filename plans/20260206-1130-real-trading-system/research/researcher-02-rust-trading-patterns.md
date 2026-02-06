# Rust Trading Systems: Best Practices Research

**Date**: 2026-02-06 | **Status**: Complete | **Scope**: Order execution, error handling, position management, risk management, state management, testing

---

## 1. Order Execution Patterns

### Async-First Architecture (Recommended)
- **Pattern**: Event-driven async/await with Tokio runtime
- **Benefits**: Handle thousands of concurrent orders, non-blocking market data ingestion, session multiplexing
- **Key**: Decouple components via async channels for fault isolation

### Sync vs Async Decision
- **Async**: Market data feeds, multi-venue aggregation, batch order processing
- **Sync**: Per-order state updates, validation checks (keep latency predictable)
- **Hybrid**: Async coordination with sync execution boundaries

### Retry Logic
- **Pattern**: Exponential backoff with jitter (crate: `backoff`)
- **Failures**: Network timeouts, rate limits, temporary service unavailability
- **Config**: Max 3-5 retries, 100ms-1s backoff, circuit breaker for cascading failures

---

## 2. Error Handling for Trading Operations

### Layered Error Strategy
1. **Execution Layer**: Transient errors (network) → retry; Permanent errors (bad order) → fail-fast
2. **Order Layer**: Invalid state transition → reject; Validation failure → log/alert
3. **Portfolio Layer**: Insufficient balance → block; Risk limit hit → force cooling period

### Critical Pattern: Result<T, E> with Custom Enum
```rust
enum OrderError {
    InsufficientBalance,
    InvalidState(CurrentState, RequestedState),
    RiskViolation(String),
    ExecutionTimeout,
    ExchangeRejected(String),
}
```
- Never use `.unwrap()/.expect()` in production paths
- Use `?` operator for error propagation
- Instrument errors with context (order_id, timestamp, state)

---

## 3. Position Management & Tracking

### State Machine Approach (Type-Safe)
Positions have explicit state: `Pending → Open → Closing → Closed`
- Type system encodes valid transitions (compile-time safety)
- Prevents invalid state mutations (risk control)

### Data Structure (O(1) Lookups)
- **Primary**: HashMap<OrderId, Order> for fast access
- **Secondary**: BTreeMap<Symbol, Vec<Position>> for aggregation
- **Index**: By status, timestamp for efficient filtering

### Tracking Elements
- Entry price + timestamp
- Current quantity, value, unrealized P&L
- Associated risk metadata (volatility, correlation)
- Execution latency (for slippage correlation)

---

## 4. Risk Management Implementation

### State-Based Risk Model
- **Normal State**: Full trading enabled
- **Warning Level**: Position aggregation, P&L monitoring, reduced leverage
- **Cool-Down State**: Zero new positions, closeout-only mode, alert escalation

### Core Checks (Pre-Execution)
1. **Daily Loss Limit**: Track cumulative loss, block trading if threshold crossed
2. **Correlation Limits**: Measure directional exposure across positions
3. **Leverage Caps**: Position size relative to account equity
4. **Drawdown Tracking**: Rolling max-loss calculation

### Implementation: Lock-Free with Atomic Operations
- Use `AtomicU64` for balance snapshots (minimal contention)
- Dedicated risk evaluation thread (avoid blocking order thread)
- Fail-safe defaults: assume worst case if data unavailable

---

## 5. State Management for Orders/Positions

### Actor Model Pattern (Recommended)
- Each position = independent actor with isolated state
- Messages: market_update, execution_signal, risk_event
- Isolation prevents cascading failures

### State Transitions Table
```
PendingOrder ──submit──> AwaitingFill
AwaitingFill ──fill──> Open
Open ──closeSignal──> Closing
Closing ──filled──> Closed
* ──error──> Error
```

### Persistence Strategy
- **Snapshot**: Write portfolio state every 30 seconds to MongoDB
- **WAL**: Append order events for recovery
- **Recovery**: Reload latest snapshot + replay events on startup

---

## 6. Testing Strategies for Trading Systems

### Unit Tests (Fast, Deterministic)
- Order state transitions (all 15+ edge cases)
- Risk limit checks (boundary conditions)
- P&L calculations (against known baselines)

### Integration Tests
- Multi-order scenarios (correlation calculations)
- State recovery (snapshot + replay)
- Error resilience (network failure simulation)

### Property-Based Tests
- Use `proptest` crate: generate random order sequences
- Invariants: account_equity = position_sum + cash, daily_loss ≤ limit
- Detects edge cases in risk calculations

### Backtesting
- Deterministic execution: replay known market data
- Measure: win rate, Sharpe ratio, max drawdown
- Key: separate backtest engine from live engine (data source only difference)

---

## 7. Recommended Crates

| Crate | Purpose | Notes |
|-------|---------|-------|
| **tokio** | Async runtime | Industry standard, proven at scale |
| **backoff** | Retry logic | Exponential backoff + jitter |
| **uuid** | Order IDs | Unique, sortable identifiers |
| **serde** | Serialization | Fast, flexible, widely adopted |
| **chrono** | Timestamps | Timezone-aware, ISO 8601 support |
| **barter** | Framework | Event-driven trading (optional, mature) |
| **ccxt-rust** | Exchange API | Multi-exchange connectivity |
| **proptest** | Testing | Property-based testing |
| **tracing** | Observability | Async-aware logging/instrumentation |

---

## 8. Performance Bottlenecks & Mitigation

| Issue | Root Cause | Solution |
|-------|-----------|----------|
| Order latency | Mutex contention | Lock-free queues (crossbeam) for hot paths |
| State deserialization | Large position lists | Indexed caching, lazy loading |
| Risk calculation | O(n) correlation check | Pre-compute deltas, check only new positions |
| DB writes | Synchronous I/O | Async MongoDB, batch writes every 5s |
| Event processing | Single-threaded | Tokio work-stealing scheduler (auto-balanced) |

---

## 9. Key Architectural Rules

✅ **DO**:
- Use async/await for network-bound operations
- Enforce state machines via types (generics)
- Log all state transitions + decisions with context
- Separate business logic from infrastructure
- Test risk calculations extensively

❌ **DON'T**:
- Use `unwrap()` in production paths
- Block trading threads on I/O operations
- Mix sync/async without clear boundaries
- Assume exchange APIs are reliable (plan for failures)
- Trust single state representation (use checksums for critical data)

---

## Sources

- [Event-Driven Systems in Rust](https://oneuptime.com/blog/post/2026-02-01-rust-event-driven-systems/view)
- [Retry Logic with Exponential Backoff](https://oneuptime.com/blog/post/2026-01-07-rust-retry-exponential-backoff/view)
- [Barter: Rust Trading Framework](https://github.com/barter-rs/barter-rs)
- [State Machine Patterns in Rust](https://hoverbear.org/blog/rust-state-machine-pattern/)
- [Building Trading Platforms in Rust](https://cprimozic.net/blog/building-an-algorithmic-trading-platform-in-rust/)
- [Rust vs C++ for Trading](https://databento.com/blog/rust-vs-cpp)
- [Crypto Crates Registry](https://crates.io/keywords/trading)
