# Phase 2: Order Execution

## Context

Reference existing implementations:
- `src/binance/client.rs:398-499` - `place_spot_order()`, `cancel_spot_order()`
- `src/binance/types.rs:398-509` - `SpotOrderRequest` builder methods
- `src/paper_trading/engine.rs:1041-1197` - `execute_trade()` pattern

## Requirements

1. **Order placement** - Market, limit, stop-loss, take-profit orders
2. **Order cancellation** - Cancel pending/open orders
3. **Order lifecycle** - Track state from placement to terminal state
4. **Client order IDs** - Generate unique IDs for order tracking

## Implementation Steps

### 2.1 Client Order ID Generator

```rust
// src/real_trading/engine.rs
impl RealTradingEngine {
    /// Generate unique client order ID
    /// Format: RT_<symbol>_<side>_<timestamp>_<random>
    fn generate_client_order_id(&self, symbol: &str, side: &str) -> String {
        let ts = Utc::now().timestamp_millis();
        let rand: u32 = rand::random();
        format!("RT_{}_{}_{}_{:08X}", symbol, side, ts, rand)
    }
}
```

### 2.2 Place Market Order

```rust
// @spec:FR-REAL-020 - Market Order Execution
impl RealTradingEngine {
    pub async fn place_market_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
    ) -> Result<RealOrder> {
        // 1. Acquire execution lock
        let _lock = self.execution_lock.lock().await;

        // 2. Check circuit breaker
        if self.is_circuit_breaker_open().await {
            return Err(anyhow!("Circuit breaker is open"));
        }

        // 3. Pre-trade risk check
        self.risk_manager.validate_order(symbol, side, quantity).await?;

        // 4. Generate client order ID
        let client_order_id = self.generate_client_order_id(symbol, side.to_string().as_str());

        // 5. Create local order record (PENDING state)
        let order = RealOrder::new_market(
            client_order_id.clone(),
            symbol.to_string(),
            side.to_string(),
            quantity,
        );
        self.orders.insert(client_order_id.clone(), order.clone());

        // 6. Place order via Binance API
        let request = SpotOrderRequest::market(
            symbol,
            side,
            &format_quantity(quantity),
        );

        match self.binance_client.place_spot_order(request).await {
            Ok(response) => {
                // 7. Update order with exchange response
                if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                    order.exchange_order_id = response.order_id;
                    order.state = match response.status.as_str() {
                        "NEW" => OrderState::New,
                        "FILLED" => OrderState::Filled,
                        "PARTIALLY_FILLED" => OrderState::PartiallyFilled,
                        _ => OrderState::Pending,
                    };
                    order.executed_quantity = response.executed_qty.parse().unwrap_or(0.0);
                    order.updated_at = Utc::now();

                    // Calculate average fill price from fills
                    if !response.fills.is_empty() {
                        order.average_fill_price = calculate_average_fill_price(&response.fills);
                        order.fills = response.fills.iter().map(|f| OrderFill::from(f)).collect();
                    }
                }

                // 8. Broadcast event
                let order = self.orders.get(&client_order_id).unwrap().clone();
                let _ = self.event_tx.send(RealTradingEvent::OrderPlaced(order.clone()));

                // 9. Reset error counter on success
                self.reset_consecutive_errors().await;

                Ok(order)
            },
            Err(e) => {
                // Update order state to rejected
                if let Some(mut order) = self.orders.get_mut(&client_order_id) {
                    order.state = OrderState::Rejected;
                    order.updated_at = Utc::now();
                }

                // Increment error counter
                self.increment_consecutive_errors(&e.to_string()).await;

                Err(e)
            }
        }
    }
}
```

### 2.3 Place Limit Order

```rust
// @spec:FR-REAL-021 - Limit Order Execution
impl RealTradingEngine {
    pub async fn place_limit_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
        time_in_force: TimeInForce,  // GTC, IOC, FOK
    ) -> Result<RealOrder> {
        let _lock = self.execution_lock.lock().await;

        if self.is_circuit_breaker_open().await {
            return Err(anyhow!("Circuit breaker is open"));
        }

        self.risk_manager.validate_order(symbol, side, quantity).await?;

        let client_order_id = self.generate_client_order_id(symbol, side.to_string().as_str());

        let order = RealOrder::new_limit(
            client_order_id.clone(),
            symbol.to_string(),
            side.to_string(),
            quantity,
            price,
        );
        self.orders.insert(client_order_id.clone(), order.clone());

        let request = SpotOrderRequest::limit(
            symbol,
            side,
            &format_quantity(quantity),
            &format_price(price),
        );

        // Similar error handling as market order...
        self.execute_order_request(request, &client_order_id).await
    }
}
```

### 2.4 Place Stop-Loss Order

```rust
// @spec:FR-REAL-022 - Stop-Loss Order
impl RealTradingEngine {
    pub async fn place_stop_loss_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        stop_price: f64,
        limit_price: f64,  // Price to execute at when stop triggers
    ) -> Result<RealOrder> {
        let _lock = self.execution_lock.lock().await;

        if self.is_circuit_breaker_open().await {
            return Err(anyhow!("Circuit breaker is open"));
        }

        let client_order_id = self.generate_client_order_id(symbol, "SL");

        let request = SpotOrderRequest::stop_loss_limit(
            symbol,
            side,
            &format_quantity(quantity),
            &format_price(limit_price),
            &format_price(stop_price),
        );

        self.execute_order_request(request, &client_order_id).await
    }
}
```

### 2.5 Cancel Order

```rust
// @spec:FR-REAL-023 - Order Cancellation
impl RealTradingEngine {
    pub async fn cancel_order(&self, client_order_id: &str) -> Result<RealOrder> {
        // 1. Get order from local state
        let order = self.orders.get(client_order_id)
            .ok_or_else(|| anyhow!("Order not found: {}", client_order_id))?
            .clone();

        // 2. Check if order can be cancelled
        if order.is_terminal() {
            return Err(anyhow!("Order already in terminal state: {:?}", order.state));
        }

        // 3. Cancel via API
        match self.binance_client.cancel_spot_order(
            &order.symbol,
            Some(order.exchange_order_id),
            Some(client_order_id),
        ).await {
            Ok(_) => {
                // 4. Update local state
                if let Some(mut order) = self.orders.get_mut(client_order_id) {
                    order.state = OrderState::Cancelled;
                    order.updated_at = Utc::now();
                }

                let updated_order = self.orders.get(client_order_id).unwrap().clone();
                let _ = self.event_tx.send(RealTradingEvent::OrderCancelled(updated_order.clone()));

                Ok(updated_order)
            },
            Err(e) => {
                self.increment_consecutive_errors(&e.to_string()).await;
                Err(e)
            }
        }
    }

    pub async fn cancel_all_orders(&self, symbol: Option<&str>) -> Result<Vec<String>> {
        let orders_to_cancel: Vec<String> = self.orders
            .iter()
            .filter(|entry| {
                let order = entry.value();
                order.is_active() && symbol.map_or(true, |s| order.symbol == s)
            })
            .map(|entry| entry.key().clone())
            .collect();

        let mut cancelled = Vec::new();
        for order_id in orders_to_cancel {
            if let Ok(_) = self.cancel_order(&order_id).await {
                cancelled.push(order_id);
            }
        }

        Ok(cancelled)
    }
}
```

### 2.6 Helper Methods

```rust
impl RealTradingEngine {
    async fn execute_order_request(
        &self,
        request: SpotOrderRequest,
        client_order_id: &str,
    ) -> Result<RealOrder> {
        match self.binance_client.place_spot_order(request).await {
            Ok(response) => {
                self.update_order_from_response(client_order_id, &response).await;
                self.reset_consecutive_errors().await;
                Ok(self.orders.get(client_order_id).unwrap().clone())
            },
            Err(e) => {
                if let Some(mut order) = self.orders.get_mut(client_order_id) {
                    order.state = OrderState::Rejected;
                }
                self.increment_consecutive_errors(&e.to_string()).await;
                Err(e)
            }
        }
    }

    async fn update_order_from_response(
        &self,
        client_order_id: &str,
        response: &SpotOrderResponse,
    ) {
        if let Some(mut order) = self.orders.get_mut(client_order_id) {
            order.exchange_order_id = response.order_id;
            order.state = OrderState::from_status(&response.status);
            order.executed_quantity = parse_quantity(&response.executed_qty);
            order.remaining_quantity = order.original_quantity - order.executed_quantity;
            order.updated_at = Utc::now();

            if !response.fills.is_empty() {
                order.average_fill_price = calculate_average_fill_price(&response.fills);
                for fill in &response.fills {
                    order.fills.push(OrderFill::from(fill));
                }
            }
        }
    }
}

fn format_quantity(qty: f64) -> String {
    format!("{:.8}", qty).trim_end_matches('0').trim_end_matches('.').to_string()
}

fn format_price(price: f64) -> String {
    format!("{:.8}", price).trim_end_matches('0').trim_end_matches('.').to_string()
}

fn calculate_average_fill_price(fills: &[Fill]) -> f64 {
    let total_qty: f64 = fills.iter().map(|f| f.qty.parse::<f64>().unwrap_or(0.0)).sum();
    let total_value: f64 = fills.iter().map(|f| {
        let qty = f.qty.parse::<f64>().unwrap_or(0.0);
        let price = f.price.parse::<f64>().unwrap_or(0.0);
        qty * price
    }).sum();

    if total_qty > 0.0 { total_value / total_qty } else { 0.0 }
}
```

## Success Criteria

- [ ] Market orders execute within 100ms (network latency)
- [ ] Client order IDs are unique and traceable
- [ ] Order state machine transitions correctly
- [ ] Failed orders update local state to Rejected
- [ ] Cancellation works for active orders, fails for terminal orders
- [ ] Circuit breaker checked before every order

## Risk Considerations

- **Double execution**: Use execution lock to prevent race conditions
- **Stale orders**: Implement timeout for orders not confirmed
- **API errors**: Distinguish between retryable and permanent errors
- **Order ID collisions**: Use timestamp + random for uniqueness

## Tests to Write

```rust
#[cfg(test)]
mod tests {
    #[tokio::test] async fn test_market_order_success() { }
    #[tokio::test] async fn test_market_order_circuit_breaker_open() { }
    #[tokio::test] async fn test_limit_order_success() { }
    #[tokio::test] async fn test_cancel_active_order() { }
    #[tokio::test] async fn test_cancel_terminal_order_fails() { }
    #[tokio::test] async fn test_cancel_all_orders() { }
    #[tokio::test] async fn test_order_id_generation_unique() { }
}
```
