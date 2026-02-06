# Phase 04: Manual Trading API

**Status**: Pending | **Estimated Time**: 1 day

## Context

Existing `real_trading.rs` API has status/portfolio endpoints. Need order placement, cancellation, and order history endpoints for manual trading UI.

## Overview

Add REST API endpoints for manual trading: place orders, cancel orders, modify positions, view order history.

## Key Insights (From Research)

- Existing API uses Warp framework
- Response format: `ApiResponse<T>` with success/data/error
- Safety: 2-step confirmation pattern needed for orders

## Requirements

1. Place order endpoint (spot + futures)
2. Cancel order endpoint
3. Cancel all orders endpoint
4. Modify position SL/TP endpoint
5. Order history endpoint (open + filled + cancelled)
6. Confirmation token system for safety

## Architecture

```
API Endpoints
    ├── POST /api/real-trading/orders              - Place order
    ├── DELETE /api/real-trading/orders/{id}       - Cancel order
    ├── DELETE /api/real-trading/orders            - Cancel all
    ├── PUT /api/real-trading/positions/{id}/sltp  - Modify SL/TP
    ├── GET /api/real-trading/orders               - List orders
    ├── GET /api/real-trading/orders/history       - Order history
    └── POST /api/real-trading/confirm             - Confirm action
```

## Related Files

| File | Action | Description |
|------|--------|-------------|
| `rust-core-engine/src/api/real_trading.rs` | Modify | Add order endpoints |
| `rust-core-engine/src/api/mod.rs` | Modify | Register new routes |
| `rust-core-engine/src/real_trading/engine.rs` | Use | Call existing methods |

## Implementation Steps

### 1. Request/Response Types
```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct PlaceOrderRequest {
    pub symbol: String,
    pub side: String,          // "BUY" or "SELL"
    pub order_type: String,    // "MARKET", "LIMIT", "STOP_LOSS", etc.
    pub quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub leverage: Option<u32>, // For futures only
    pub position_side: Option<String>, // "LONG" or "SHORT"
    pub stop_loss: Option<f64>,
    pub take_profit: Option<f64>,
    pub confirmation_token: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderInfo {
    pub id: String,
    pub exchange_order_id: i64,
    pub symbol: String,
    pub side: String,
    pub order_type: String,
    pub quantity: f64,
    pub filled_quantity: f64,
    pub price: Option<f64>,
    pub stop_price: Option<f64>,
    pub status: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmationRequest {
    pub action: String,    // "place_order", "cancel_all"
    pub order_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConfirmationResponse {
    pub token: String,
    pub expires_at: String,
    pub summary: String,
}
```

### 2. Confirmation System
```rust
// Simple token-based confirmation
// 1. Client sends order -> gets confirmation token + summary
// 2. Client confirms with token -> order executes
// 3. Token expires after 60 seconds

struct PendingConfirmation {
    token: String,
    action: String,
    data: serde_json::Value,
    expires_at: DateTime<Utc>,
}
```

### 3. API Routes
```rust
// Add to routes()
// POST /api/real-trading/orders
let place_order_route = base_path
    .and(warp::path("orders"))
    .and(warp::path::end())
    .and(warp::post())
    .and(warp::body::json())
    .and(with_api(api.clone()))
    .and_then(place_order);

// DELETE /api/real-trading/orders/{id}
let cancel_order_route = base_path
    .and(warp::path("orders"))
    .and(warp::path::param::<String>())
    .and(warp::path::end())
    .and(warp::delete())
    .and(with_api(api.clone()))
    .and_then(cancel_order);

// GET /api/real-trading/orders
let list_orders_route = base_path
    .and(warp::path("orders"))
    .and(warp::path::end())
    .and(warp::get())
    .and(warp::query::<ListOrdersQuery>())
    .and(with_api(api.clone()))
    .and_then(list_orders);
```

### 4. Handler Implementations
```rust
async fn place_order(
    request: PlaceOrderRequest,
    api: Arc<RealTradingApi>,
) -> Result<impl Reply, Rejection> {
    // 1. Validate request
    // 2. If no confirmation token, return confirmation request
    // 3. If valid token, execute order
    // 4. Return order info or error
}

async fn cancel_order(
    order_id: String,
    api: Arc<RealTradingApi>,
) -> Result<impl Reply, Rejection> {
    // Cancel specific order by ID
}

async fn list_orders(
    query: ListOrdersQuery,
    api: Arc<RealTradingApi>,
) -> Result<impl Reply, Rejection> {
    // List orders with optional status filter
}
```

## Todo

- [ ] Add PlaceOrderRequest type
- [ ] Add OrderInfo type
- [ ] Add ConfirmationRequest/Response types
- [ ] Implement confirmation token system
- [ ] Add POST /orders route and handler
- [ ] Add DELETE /orders/{id} route and handler
- [ ] Add DELETE /orders (cancel all) route and handler
- [ ] Add PUT /positions/{id}/sltp route and handler
- [ ] Add GET /orders route and handler
- [ ] Add GET /orders/history route and handler
- [ ] Wire up handlers to RealTradingEngine
- [ ] Add input validation
- [ ] Test place market order via API
- [ ] Test place limit order via API
- [ ] Test cancel order via API
- [ ] Test list orders via API

## Success Criteria

- [ ] Place order returns confirmation token (first call)
- [ ] Place order with valid token executes
- [ ] Cancel order removes pending order
- [ ] Cancel all orders removes all pending
- [ ] Modify SL/TP updates position
- [ ] Order history returns paginated results

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| Token replay attack | Low | Medium | Single-use tokens, short TTL |
| Rate abuse | Low | Low | Existing rate limiter |
| Invalid order params | Medium | Low | Strict validation |

## Security Considerations

- Confirmation token is single-use
- Token expires after 60 seconds
- All order placements logged
- No credentials in request/response

## Next Steps

After completion: Proceed to Phase 05 (Frontend Integration) for UI.
