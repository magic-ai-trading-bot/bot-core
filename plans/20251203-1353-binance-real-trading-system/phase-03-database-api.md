# Phase 3: Database & API

## Context Links
- [Main Plan](./plan.md)
- [Phase 2: Real Trading Engine](./phase-02-real-trading-engine.md)
- [DB Schema Spec](../../specs/02-design/2.2-database/DB-SCHEMA.md)

## Overview

| Field | Value |
|-------|-------|
| Priority | HIGH |
| Status | PENDING |
| Effort | 2 days |
| Dependencies | Phase 2 complete |

Add MongoDB collections for real trading persistence and REST API endpoints for real trading operations. Mirror paper trading storage patterns.

## Key Insights

1. **Existing collections**: `paper_portfolios`, `paper_trades`, `paper_trading_settings`
2. **Storage module**: `rust-core-engine/src/storage/` with MongoDB operations
3. **API handlers**: `rust-core-engine/src/api/handlers/` with Actix-web
4. **Separate collections**: Keep real and paper trades isolated
5. **Same patterns**: Reuse existing CRUD patterns

## Requirements

### Functional
- FR-REAL-020: Create `real_trades` collection for order history
- FR-REAL-021: Create `real_portfolios` collection for account state
- FR-REAL-022: Create `real_trading_settings` collection
- FR-REAL-023: API endpoint to get real portfolio summary
- FR-REAL-024: API endpoint to list real trades (paginated)
- FR-REAL-025: API endpoint to get current trading mode
- FR-REAL-026: API endpoint to switch trading mode
- FR-REAL-027: API endpoint for emergency stop (cancel all, disable)

### Non-Functional
- NFR-REAL-020: Indexes on symbol, status, timestamps
- NFR-REAL-021: Pagination with cursor for large trade history
- NFR-REAL-022: Read-after-write consistency

## Architecture

### Collections Schema

```
real_trades
├── _id: ObjectId
├── id: String (UUID)
├── user_id: String
├── binance_order_id: i64
├── client_order_id: String
├── symbol: String (indexed)
├── trade_type: String ("Long" | "Short")
├── status: String (indexed)
├── entry_price: f64
├── exit_price: f64 (nullable)
├── quantity: f64
├── requested_quantity: f64
├── leverage: u8
├── stop_loss_order_id: i64 (nullable)
├── take_profit_order_id: i64 (nullable)
├── unrealized_pnl: f64
├── realized_pnl: f64 (nullable)
├── pnl_percentage: f64
├── commission_paid: f64
├── order_time: DateTime (indexed)
├── fill_time: DateTime (nullable)
├── close_time: DateTime (nullable, indexed)
├── ai_signal_id: String (nullable)
├── ai_confidence: f64 (nullable)
├── mode: String ("testnet" | "mainnet")
├── created_at: DateTime
└── updated_at: DateTime

real_portfolios
├── _id: ObjectId
├── user_id: String (unique indexed)
├── mode: String ("testnet" | "mainnet")
├── balance: f64
├── available_balance: f64
├── locked_balance: f64
├── total_pnl: f64
├── total_trades: i32
├── winning_trades: i32
├── losing_trades: i32
├── win_rate: f64
├── max_drawdown: f64
├── last_sync_time: DateTime
├── created_at: DateTime
└── updated_at: DateTime

real_trading_settings
├── _id: ObjectId
├── user_id: String (unique indexed)
├── mode: String ("paper" | "testnet" | "mainnet")
├── enabled: bool
├── basic: BasicSettings (embedded)
├── risk: RiskSettings (embedded)
├── strategy: StrategySettings (embedded)
├── created_at: DateTime
└── updated_at: DateTime
```

### API Endpoints

```
GET  /api/trading/mode
     Returns: { mode: "paper" | "testnet" | "mainnet" }

POST /api/trading/mode
     Body: { mode: "paper" | "testnet" | "mainnet" }
     Returns: { success: true, mode: "..." }
     Security: Requires confirmation for "mainnet"

GET  /api/real-trading/portfolio
     Returns: RealPortfolioSummary

GET  /api/real-trading/trades
     Query: ?status=open|closed&symbol=BTCUSDT&limit=50&cursor=...
     Returns: { trades: [...], next_cursor: "..." }

GET  /api/real-trading/trades/:id
     Returns: RealTrade

POST /api/real-trading/emergency-stop
     Returns: { cancelled_orders: 5, disabled: true }
     Security: Requires confirmation header

GET  /api/real-trading/settings
     Returns: RealTradingSettings

PUT  /api/real-trading/settings
     Body: RealTradingSettings
     Returns: { success: true }
```

## Related Code Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/storage/real_trading.rs` | CREATE | Real trading storage |
| `rust-core-engine/src/storage/mod.rs` | MODIFY | Export new module |
| `rust-core-engine/src/api/handlers/real_trading.rs` | CREATE | API handlers |
| `rust-core-engine/src/api/handlers/mod.rs` | MODIFY | Export handlers |
| `rust-core-engine/src/api/routes.rs` | MODIFY | Add routes |

## Implementation Steps

### Step 1: Create Storage Module (Day 1)

1. Create `storage/real_trading.rs`
2. Implement `RealTradingStorage` struct:
   ```rust
   impl RealTradingStorage {
       // Trade operations
       pub async fn save_trade(&self, trade: &RealTrade) -> Result<()>;
       pub async fn update_trade(&self, trade: &RealTrade) -> Result<()>;
       pub async fn get_trade(&self, id: &str) -> Result<Option<RealTrade>>;
       pub async fn get_trades_by_status(&self, status: RealTradeStatus, limit: i64) -> Result<Vec<RealTrade>>;
       pub async fn list_trades(&self, filter: TradeFilter, cursor: Option<&str>, limit: i64) -> Result<(Vec<RealTrade>, Option<String>)>;

       // Portfolio operations
       pub async fn save_portfolio(&self, portfolio: &RealPortfolio) -> Result<()>;
       pub async fn get_portfolio(&self, user_id: &str, mode: &str) -> Result<Option<RealPortfolio>>;
       pub async fn update_portfolio_balance(&self, user_id: &str, mode: &str, balance: f64) -> Result<()>;

       // Settings operations
       pub async fn get_trading_settings(&self, user_id: &str) -> Result<Option<RealTradingSettings>>;
       pub async fn save_trading_settings(&self, settings: &RealTradingSettings) -> Result<()>;
       pub async fn get_current_mode(&self, user_id: &str) -> Result<TradingMode>;
       pub async fn set_current_mode(&self, user_id: &str, mode: TradingMode) -> Result<()>;
   }
   ```
3. Add indexes in `ensure_indexes()` method

### Step 2: Add Indexes (Day 1)

```rust
// real_trades indexes
collection.create_index(
    IndexModel::builder()
        .keys(doc! { "symbol": 1, "status": 1, "order_time": -1 })
        .options(IndexOptions::builder().build())
        .build()
).await?;

collection.create_index(
    IndexModel::builder()
        .keys(doc! { "user_id": 1, "mode": 1, "close_time": -1 })
        .options(IndexOptions::builder().build())
        .build()
).await?;

// real_portfolios indexes
collection.create_index(
    IndexModel::builder()
        .keys(doc! { "user_id": 1, "mode": 1 })
        .options(IndexOptions::builder().unique(true).build())
        .build()
).await?;
```

### Step 3: Create API Handlers (Day 1-2)

1. Create `api/handlers/real_trading.rs`
2. Implement handlers:

```rust
pub async fn get_trading_mode(
    storage: web::Data<Storage>,
) -> Result<HttpResponse, AppError>;

pub async fn set_trading_mode(
    storage: web::Data<Storage>,
    body: web::Json<SetModeRequest>,
) -> Result<HttpResponse, AppError>;

pub async fn get_real_portfolio(
    storage: web::Data<Storage>,
) -> Result<HttpResponse, AppError>;

pub async fn list_real_trades(
    storage: web::Data<Storage>,
    query: web::Query<TradeListQuery>,
) -> Result<HttpResponse, AppError>;

pub async fn get_real_trade(
    storage: web::Data<Storage>,
    path: web::Path<String>,
) -> Result<HttpResponse, AppError>;

pub async fn emergency_stop(
    orchestrator: web::Data<TradingOrchestrator>,
    headers: web::HttpRequest,
) -> Result<HttpResponse, AppError>;

pub async fn get_real_settings(
    storage: web::Data<Storage>,
) -> Result<HttpResponse, AppError>;

pub async fn update_real_settings(
    storage: web::Data<Storage>,
    body: web::Json<RealTradingSettings>,
) -> Result<HttpResponse, AppError>;
```

### Step 4: Add Routes (Day 2)

Update `api/routes.rs`:

```rust
pub fn configure_routes(cfg: &mut web::ServiceConfig) {
    cfg
        // Existing paper trading routes
        .service(/* ... */)
        // Trading mode routes
        .service(
            web::resource("/api/trading/mode")
                .route(web::get().to(handlers::real_trading::get_trading_mode))
                .route(web::post().to(handlers::real_trading::set_trading_mode))
        )
        // Real trading routes
        .service(
            web::scope("/api/real-trading")
                .route("/portfolio", web::get().to(handlers::real_trading::get_real_portfolio))
                .route("/trades", web::get().to(handlers::real_trading::list_real_trades))
                .route("/trades/{id}", web::get().to(handlers::real_trading::get_real_trade))
                .route("/settings", web::get().to(handlers::real_trading::get_real_settings))
                .route("/settings", web::put().to(handlers::real_trading::update_real_settings))
                .route("/emergency-stop", web::post().to(handlers::real_trading::emergency_stop))
        );
}
```

### Step 5: Add Mode Switch Safety (Day 2)

1. Require confirmation for mainnet mode:
   ```rust
   pub async fn set_trading_mode(body: SetModeRequest) -> Result<...> {
       if body.mode == TradingMode::RealMainnet {
           if !body.confirmed {
               return Err(AppError::RequiresConfirmation("mainnet_mode"));
           }
           // Check env var allows mainnet
           if std::env::var("ALLOW_MAINNET_TRADING").is_err() {
               return Err(AppError::MainnetDisabled);
           }
       }
       // Proceed with mode switch
   }
   ```

2. Emergency stop requires header:
   ```rust
   pub async fn emergency_stop(req: HttpRequest) -> Result<...> {
       let confirm = req.headers().get("X-Confirm-Emergency-Stop");
       if confirm != Some("CONFIRM_STOP_ALL_TRADING") {
           return Err(AppError::RequiresConfirmation("emergency_stop"));
       }
       // Cancel all orders, disable trading
   }
   ```

### Step 6: Tests (Day 2)

1. Unit tests for storage operations
2. API integration tests
3. Mode switch confirmation tests

## Todo List

- [ ] Create storage/real_trading.rs module
- [ ] Implement RealTradingStorage struct
- [ ] Add save_trade(), update_trade(), get_trade() methods
- [ ] Add list_trades() with cursor pagination
- [ ] Add portfolio CRUD methods
- [ ] Add settings CRUD methods
- [ ] Add get_current_mode(), set_current_mode()
- [ ] Create collection indexes
- [ ] Create api/handlers/real_trading.rs
- [ ] Implement get_trading_mode handler
- [ ] Implement set_trading_mode with confirmation
- [ ] Implement get_real_portfolio handler
- [ ] Implement list_real_trades with pagination
- [ ] Implement emergency_stop with header check
- [ ] Add routes to routes.rs
- [ ] Write storage unit tests
- [ ] Write API integration tests

## Success Criteria

1. Real trades persist to MongoDB correctly
2. Portfolio syncs with database on update
3. API returns paginated trade history
4. Mode switch requires confirmation for mainnet
5. Emergency stop cancels all orders
6. All tests pass

## Risk Assessment

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Data loss on failure | High | Low | Transactions, retry logic |
| Mode switch during trades | Medium | Low | Block if open positions |
| Emergency stop failure | High | Low | Retry, manual override |

## Security Considerations

1. **Mainnet requires env var** - `ALLOW_MAINNET_TRADING=true`
2. **Confirmation headers** - For destructive operations
3. **No secrets in DB** - API keys stay in env vars
4. **Audit trail** - Log all mode switches

## Next Steps

After Phase 3 complete:
- Proceed to [Phase 4: Frontend UI](./phase-04-frontend-ui.md)
- Build trading mode selector
- Add real trading dashboard
