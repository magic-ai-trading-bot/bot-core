# Phase 2: Real Trading Engine

## Context Links
- [Main Plan](./plan.md)
- [Phase 1: Binance Order API](./phase-01-binance-order-api.md)
- [Paper Trading Engine](../../rust-core-engine/src/paper_trading/engine.rs)

## Overview

| Field | Value |
|-------|-------|
| Priority | HIGH |
| Status | PENDING |
| Effort | 4 days |
| Dependencies | Phase 1 complete |

Create `RealTradingEngine` mirroring `PaperTradingEngine` structure. Shared strategy signals, same risk checks, real order execution via Binance API.

## Key Insights

1. **Paper trading engine structure**: PaperTradingEngine has portfolio, settings, optimizer, binance_client, ai_service, storage, event_broadcaster
2. **Risk checks**: Daily loss limit, consecutive loss cool-down, position correlation, max positions
3. **Trade model**: PaperTrade has entry/exit, SL/TP, leverage, fees, timestamps
4. **Execution simulation**: Paper trading simulates slippage, partial fills, latency
5. **Dual-mode**: Need shared signal processing, different execution paths

## Requirements

### Functional
- FR-REAL-010: Create RealTradingEngine mirroring PaperTradingEngine
- FR-REAL-011: Implement TradingMode enum (PaperTrading, RealTestnet, RealMainnet)
- FR-REAL-012: Share AI signal processing between modes
- FR-REAL-013: Apply same risk management rules as paper trading
- FR-REAL-014: Track real trades with actual fill prices from Binance
- FR-REAL-015: Sync portfolio with Binance account balance
- FR-REAL-016: Handle partial fills from exchange

### Non-Functional
- NFR-REAL-010: Mode switch via config/API without restart
- NFR-REAL-011: Real-time position sync via WebSocket
- NFR-REAL-012: Graceful shutdown - cancel pending orders

## Architecture

```
TradingMode Enum
├── PaperTrading   → PaperTradingEngine (existing)
├── RealTestnet    → RealTradingEngine + testnet config
└── RealMainnet    → RealTradingEngine + mainnet config

RealTradingEngine
├── portfolio: RealPortfolio (synced with Binance)
├── settings: RealTradingSettings (extended from PaperTradingSettings)
├── binance_client: BinanceClient (with order methods)
├── user_data_stream: UserDataStreamManager
├── order_manager: OrderManager (pending/active orders)
├── ai_service: AIService (shared)
├── storage: Storage (real_trades collection)
└── event_broadcaster: broadcast::Sender<TradingEvent>
```

### Shared Components

```rust
// Unified trading mode
pub enum TradingMode {
    PaperTrading,
    RealTestnet,
    RealMainnet,
}

// Shared signal source
pub struct TradingOrchestrator {
    mode: TradingMode,
    paper_engine: Option<Arc<PaperTradingEngine>>,
    real_engine: Option<Arc<RealTradingEngine>>,
    ai_service: AIService,
}

impl TradingOrchestrator {
    pub async fn process_signal(&self, signal: AITradingSignal) -> Result<()> {
        match self.mode {
            TradingMode::PaperTrading => {
                self.paper_engine.as_ref().unwrap().execute_signal(signal).await
            },
            TradingMode::RealTestnet | TradingMode::RealMainnet => {
                self.real_engine.as_ref().unwrap().execute_signal(signal).await
            },
        }
    }
}
```

### RealTrade Model

```rust
pub struct RealTrade {
    // Identification
    pub id: String,                    // Internal UUID
    pub binance_order_id: i64,         // Binance order ID
    pub client_order_id: String,       // For tracking

    // Trade info (same as PaperTrade)
    pub symbol: String,
    pub trade_type: TradeType,         // Long/Short
    pub status: RealTradeStatus,
    pub entry_price: f64,              // Actual fill price
    pub exit_price: Option<f64>,
    pub quantity: f64,                 // Filled quantity
    pub requested_quantity: f64,       // Originally requested
    pub leverage: u8,

    // Orders
    pub stop_loss_order_id: Option<i64>,
    pub take_profit_order_id: Option<i64>,

    // P&L
    pub unrealized_pnl: f64,
    pub realized_pnl: Option<f64>,
    pub pnl_percentage: f64,
    pub commission_paid: f64,          // Actual commission from Binance

    // Timestamps
    pub order_time: DateTime<Utc>,
    pub fill_time: Option<DateTime<Utc>>,
    pub close_time: Option<DateTime<Utc>>,

    // AI context
    pub ai_signal_id: Option<String>,
    pub ai_confidence: Option<f64>,
}

pub enum RealTradeStatus {
    PendingOpen,        // Order placed, not filled
    PartiallyFilled,    // Partial fill
    Open,               // Fully filled, position open
    PendingClose,       // Close order placed
    Closed,             // Position closed
    Cancelled,          // Order cancelled before fill
    Rejected,           // Order rejected by exchange
}
```

## Related Code Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/real_trading/mod.rs` | CREATE | Module exports |
| `rust-core-engine/src/real_trading/engine.rs` | CREATE | RealTradingEngine |
| `rust-core-engine/src/real_trading/portfolio.rs` | CREATE | RealPortfolio |
| `rust-core-engine/src/real_trading/trade.rs` | CREATE | RealTrade model |
| `rust-core-engine/src/real_trading/order_manager.rs` | CREATE | Order tracking |
| `rust-core-engine/src/real_trading/settings.rs` | CREATE | Settings |
| `rust-core-engine/src/trading_orchestrator.rs` | CREATE | Mode switching |
| `rust-core-engine/src/paper_trading/engine.rs` | MODIFY | Extract shared logic |
| `rust-core-engine/src/lib.rs` | MODIFY | Add modules |

## Implementation Steps

### Step 1: Create TradingMode and Shared Types (Day 1)

1. Create `TradingMode` enum
2. Create shared traits for trading engines:
   ```rust
   pub trait TradingEngine {
       async fn execute_signal(&self, signal: AITradingSignal) -> Result<()>;
       async fn close_trade(&self, trade_id: &str) -> Result<()>;
       async fn get_open_positions(&self) -> Result<Vec<Position>>;
       async fn get_portfolio_summary(&self) -> Result<PortfolioSummary>;
   }
   ```
3. Extract shared risk management to `risk_manager.rs`

### Step 2: Create RealTrade Model (Day 1)

1. Create `real_trading/trade.rs` with RealTrade struct
2. Add `RealTradeStatus` enum
3. Implement conversion from Binance order response
4. Add methods: `update_from_execution_report()`, `calculate_pnl()`

### Step 3: Create RealPortfolio (Day 1-2)

1. Create `real_trading/portfolio.rs`
2. Sync with Binance account balance on startup
3. Update from `outboundAccountPosition` WebSocket events
4. Track: available balance, locked balance, positions
5. Methods: `can_open_position()`, `reserve_margin()`, `release_margin()`

### Step 4: Create OrderManager (Day 2)

1. Create `real_trading/order_manager.rs`
2. Track pending orders (by client_order_id)
3. Match executionReport events to pending orders
4. Handle partial fills - update quantities
5. Handle cancels/rejects - clean up and notify

### Step 5: Create RealTradingEngine (Day 2-3)

1. Mirror PaperTradingEngine structure
2. Inject UserDataStreamManager (from Phase 1)
3. Implement `execute_signal()`:
   - Check risk limits (same as paper)
   - Place order via BinanceClient
   - Track in OrderManager
   - Wait for fill or timeout
4. Implement `close_trade()`:
   - Place close order
   - Cancel SL/TP orders
   - Update trade status
5. Start background tasks:
   - Price updates (same as paper)
   - User data stream processing
   - Order timeout checker
   - Position sync (every 5 min via REST)

### Step 6: Create TradingOrchestrator (Day 3)

1. Create `trading_orchestrator.rs`
2. Hold both engines (optional, based on mode)
3. Route signals to active engine
4. Provide API for mode switching
5. Handle graceful mode transition

### Step 7: Extract Shared Risk Management (Day 3-4)

1. Create `shared/risk_manager.rs`
2. Move from PaperTradingEngine:
   - `check_daily_loss_limit()`
   - `is_in_cooldown()`
   - `check_position_correlation()`
   - `calculate_position_size()`
3. Use same RiskManager in both engines

### Step 8: Integration and Tests (Day 4)

1. Wire up in main application
2. Add API endpoint for mode switching
3. Test on testnet with small orders
4. Verify risk checks apply to real orders

## Todo List

- [ ] Create TradingMode enum
- [ ] Define TradingEngine trait
- [ ] Create RealTrade model with RealTradeStatus
- [ ] Create RealPortfolio with Binance sync
- [ ] Create OrderManager for pending order tracking
- [ ] Implement RealTradingEngine.execute_signal()
- [ ] Implement RealTradingEngine.close_trade()
- [ ] Implement user data stream event processing
- [ ] Create TradingOrchestrator for mode routing
- [ ] Extract shared RiskManager
- [ ] Add mode switch API endpoint
- [ ] Write integration tests with testnet

## Success Criteria

1. Can place real orders on testnet via AI signals
2. Orders fill and positions tracked correctly
3. SL/TP orders placed automatically
4. Risk checks prevent excessive positions
5. Mode switch works without restart
6. WebSocket updates positions in real-time

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Order placement failure | High | Medium | Retry with backoff, alert |
| Position sync drift | Medium | Low | Periodic REST reconciliation |
| Mode switch during open orders | High | Low | Require close positions first |
| Partial fill handling | Medium | Medium | Test thoroughly on testnet |

## Security Considerations

1. **Mainnet disabled by default** - Require explicit env var
2. **Balance verification** - Check sufficient before order
3. **Order size limits** - Max order size in config
4. **Emergency stop** - Cancel all orders, disable trading

## Next Steps

After Phase 2 complete:
- Proceed to [Phase 3: Database & API](./phase-03-database-api.md)
- Add persistence for real trades
- Create REST API for real trading operations
