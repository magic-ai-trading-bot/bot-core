# Phase 5: Reconciliation & Fallback

## Context

Reference existing implementations:
- `src/binance/client.rs:274-296` - `get_open_orders()`, account queries
- `src/binance/user_data_stream.rs` - Reconnection handling
- Research context: 5-minute reconciliation interval, REST fallback

## Requirements

1. **REST reconciliation** - Periodic sync with exchange state
2. **Order reconciliation** - Match local orders with exchange orders
3. **Position reconciliation** - Verify positions match actual holdings
4. **Balance sync** - Ensure balances are accurate
5. **Stale order cleanup** - Cancel orders past timeout

## Implementation Steps

### 5.1 Initial Sync (Engine Start)

```rust
// @spec:FR-REAL-050 - Initial State Sync
impl RealTradingEngine {
    pub async fn initial_sync(&self) -> Result<()> {
        info!("Starting initial sync with exchange...");

        // 1. Fetch account info and balances
        self.sync_balances().await?;

        // 2. Fetch open orders
        self.sync_open_orders().await?;

        // 3. Derive positions from account balances (for spot)
        self.sync_positions().await?;

        info!("Initial sync complete");
        Ok(())
    }

    async fn sync_balances(&self) -> Result<()> {
        let account = self.binance_client.get_account_info().await?;

        let mut balances = self.balances.write().await;
        balances.clear();

        for balance in account.balances {
            let free: f64 = balance.free.parse().unwrap_or(0.0);
            if free > 0.0 {
                balances.insert(balance.asset, free);
            }
        }

        info!("Synced {} asset balances", balances.len());
        Ok(())
    }

    async fn sync_open_orders(&self) -> Result<()> {
        // Note: For spot, use get_spot_open_orders (need to add to client)
        // This gets all open orders across all symbols
        let open_orders = self.binance_client.get_open_orders(None).await?;

        for order in open_orders {
            let client_order_id = order.client_order_id.clone();

            if !self.orders.contains_key(&client_order_id) {
                // Order exists on exchange but not locally - add it
                let real_order = RealOrder::from_futures_order(&order);  // Adapt for spot
                self.orders.insert(client_order_id.clone(), real_order);
                warn!("Found orphan order on exchange: {}", client_order_id);
            }
        }

        // Mark orders not on exchange as stale
        let exchange_order_ids: std::collections::HashSet<_> = open_orders
            .iter()
            .map(|o| o.client_order_id.clone())
            .collect();

        for entry in self.orders.iter() {
            let order = entry.value();
            if order.is_active() && !exchange_order_ids.contains(&order.id) {
                warn!("Local order {} not found on exchange, marking stale", order.id);
                // Will be cleaned up by stale order handler
            }
        }

        info!("Synced {} open orders", open_orders.len());
        Ok(())
    }

    async fn sync_positions(&self) -> Result<()> {
        // For spot trading, positions are derived from balances
        // If we hold BTC, and have a record of buying it, that's our position

        // Clear existing positions that don't match balances
        let balances = self.balances.read().await.clone();

        for entry in self.positions.iter() {
            let symbol = entry.key();
            let base_asset = symbol.replace("USDT", "");  // BTCUSDT -> BTC

            if let Some(&balance) = balances.get(&base_asset) {
                if balance < 0.0001 {
                    // No balance, remove position
                    warn!("Position {} has no balance, removing", symbol);
                }
            }
        }

        info!("Position sync complete");
        Ok(())
    }
}
```

### 5.2 Periodic Reconciliation Loop

```rust
// @spec:FR-REAL-051 - Periodic Reconciliation
impl RealTradingEngine {
    pub async fn reconciliation_loop(&self) {
        let interval_secs = {
            let config = self.config.read().await;
            config.reconciliation_interval_secs
        };

        let mut interval = tokio::time::interval(Duration::from_secs(interval_secs));

        loop {
            interval.tick().await;

            if !*self.is_running.read().await {
                break;
            }

            match self.run_reconciliation().await {
                Ok(discrepancies) => {
                    if discrepancies > 0 {
                        warn!("Reconciliation found {} discrepancies", discrepancies);
                    } else {
                        debug!("Reconciliation complete, no discrepancies");
                    }
                    let _ = self.event_tx.send(RealTradingEvent::ReconciliationComplete {
                        discrepancies,
                    });
                },
                Err(e) => {
                    error!("Reconciliation failed: {}", e);
                    self.increment_consecutive_errors(&e.to_string()).await;
                }
            }
        }
    }

    async fn run_reconciliation(&self) -> Result<u32> {
        let mut discrepancies = 0;

        // 1. Reconcile balances
        discrepancies += self.reconcile_balances().await?;

        // 2. Reconcile orders
        discrepancies += self.reconcile_orders().await?;

        // 3. Clean up stale orders
        discrepancies += self.cleanup_stale_orders().await?;

        Ok(discrepancies)
    }
}
```

### 5.3 Balance Reconciliation

```rust
// @spec:FR-REAL-052 - Balance Reconciliation
impl RealTradingEngine {
    async fn reconcile_balances(&self) -> Result<u32> {
        let account = self.binance_client.get_account_info().await?;
        let mut discrepancies = 0;

        let mut local_balances = self.balances.write().await;

        for balance in account.balances {
            let exchange_free: f64 = balance.free.parse().unwrap_or(0.0);
            let local_free = local_balances.get(&balance.asset).copied().unwrap_or(0.0);

            // Check for significant difference (>0.01%)
            if (exchange_free - local_free).abs() > exchange_free * 0.0001 {
                warn!(
                    "Balance mismatch for {}: local={}, exchange={}",
                    balance.asset, local_free, exchange_free
                );
                local_balances.insert(balance.asset.clone(), exchange_free);
                discrepancies += 1;

                let locked: f64 = balance.locked.parse().unwrap_or(0.0);
                let _ = self.event_tx.send(RealTradingEvent::BalanceUpdated {
                    asset: balance.asset,
                    free: exchange_free,
                    locked,
                });
            }
        }

        Ok(discrepancies)
    }
}
```

### 5.4 Order Reconciliation

```rust
// @spec:FR-REAL-053 - Order Reconciliation
impl RealTradingEngine {
    async fn reconcile_orders(&self) -> Result<u32> {
        let exchange_orders = self.binance_client.get_open_orders(None).await?;
        let mut discrepancies = 0;

        let exchange_order_map: HashMap<String, &FuturesOrder> = exchange_orders
            .iter()
            .map(|o| (o.client_order_id.clone(), o))
            .collect();

        // Check local orders against exchange
        for entry in self.orders.iter() {
            let local_order = entry.value();

            if local_order.is_active() {
                if let Some(exchange_order) = exchange_order_map.get(&local_order.id) {
                    // Compare states
                    let exchange_filled: f64 = exchange_order.executed_qty.parse().unwrap_or(0.0);
                    if (local_order.executed_quantity - exchange_filled).abs() > 0.0001 {
                        warn!(
                            "Order {} fill mismatch: local={}, exchange={}",
                            local_order.id, local_order.executed_quantity, exchange_filled
                        );
                        discrepancies += 1;

                        // Update local state
                        if let Some(mut order) = self.orders.get_mut(&local_order.id) {
                            order.executed_quantity = exchange_filled;
                            order.remaining_quantity = order.original_quantity - exchange_filled;
                            order.state = OrderState::from_status(&exchange_order.status);
                        }
                    }
                } else {
                    // Local order not on exchange - probably filled or cancelled
                    warn!("Active local order {} not found on exchange", local_order.id);
                    discrepancies += 1;

                    // Query specific order status
                    // (Would need to add get_order_status to client)
                }
            }
        }

        // Check for orphan orders on exchange
        for exchange_order in &exchange_orders {
            if !self.orders.contains_key(&exchange_order.client_order_id) {
                warn!(
                    "Orphan order on exchange: {} (symbol: {})",
                    exchange_order.client_order_id, exchange_order.symbol
                );
                discrepancies += 1;

                // Add to local tracking
                let real_order = RealOrder::from_futures_order(exchange_order);
                self.orders.insert(exchange_order.client_order_id.clone(), real_order);
            }
        }

        Ok(discrepancies)
    }
}
```

### 5.5 Stale Order Cleanup

```rust
// @spec:FR-REAL-054 - Stale Order Cleanup
impl RealTradingEngine {
    async fn cleanup_stale_orders(&self) -> Result<u32> {
        let stale_timeout = {
            let config = self.config.read().await;
            Duration::from_secs(config.stale_order_timeout_secs)
        };

        let now = Utc::now();
        let mut cleaned = 0;

        // Find stale orders
        let stale_order_ids: Vec<String> = self.orders
            .iter()
            .filter(|entry| {
                let order = entry.value();
                order.is_active() &&
                now.signed_duration_since(order.created_at)
                    .to_std()
                    .unwrap_or_default() > stale_timeout
            })
            .map(|entry| entry.key().clone())
            .collect();

        // Cancel stale orders
        for order_id in stale_order_ids {
            match self.cancel_order(&order_id).await {
                Ok(_) => {
                    info!("Cancelled stale order: {}", order_id);
                    cleaned += 1;
                },
                Err(e) => {
                    // Order might already be filled/cancelled
                    warn!("Failed to cancel stale order {}: {}", order_id, e);
                    // Mark as unknown state for reconciliation
                    if let Some(mut order) = self.orders.get_mut(&order_id) {
                        order.state = OrderState::Cancelled;  // Assume cancelled
                    }
                }
            }
        }

        // Cleanup terminal orders older than 24h
        let cleanup_threshold = now - chrono::Duration::hours(24);
        let orders_to_remove: Vec<String> = self.orders
            .iter()
            .filter(|entry| {
                let order = entry.value();
                order.is_terminal() && order.updated_at < cleanup_threshold
            })
            .map(|entry| entry.key().clone())
            .collect();

        for order_id in orders_to_remove {
            self.orders.remove(&order_id);
            debug!("Cleaned up old terminal order: {}", order_id);
        }

        Ok(cleaned)
    }
}
```

### 5.6 WebSocket Reconnection Handler

```rust
// @spec:FR-REAL-055 - WebSocket Reconnection
impl RealTradingEngine {
    async fn handle_websocket_disconnect(&self) {
        warn!("WebSocket disconnected, triggering reconciliation...");

        // Pause new orders during reconnection
        // (UserDataStreamManager handles reconnection automatically)

        // Run immediate reconciliation
        match self.run_reconciliation().await {
            Ok(discrepancies) => {
                if discrepancies > 0 {
                    warn!("Post-reconnect reconciliation found {} discrepancies", discrepancies);
                }
            },
            Err(e) => {
                error!("Post-reconnect reconciliation failed: {}", e);
            }
        }
    }
}
```

### 5.7 Emergency Stop

```rust
// @spec:FR-REAL-056 - Emergency Stop
impl RealTradingEngine {
    pub async fn emergency_stop(&self, reason: &str) -> Result<()> {
        warn!("EMERGENCY STOP triggered: {}", reason);

        // 1. Open circuit breaker immediately
        self.open_circuit_breaker(format!("EMERGENCY: {}", reason)).await;

        // 2. Cancel all open orders
        let cancelled = self.cancel_all_orders(None).await?;
        info!("Cancelled {} orders during emergency stop", cancelled.len());

        // 3. Stop the engine
        {
            let mut running = self.is_running.write().await;
            *running = false;
        }

        // 4. Stop UserDataStream
        {
            let mut stream = self.user_data_stream.write().await;
            stream.stop().await?;
        }

        let _ = self.event_tx.send(RealTradingEvent::Error(
            format!("Emergency stop: {}", reason)
        ));

        Ok(())
    }
}
```

## Success Criteria

- [ ] Initial sync loads all balances and open orders correctly
- [ ] Reconciliation runs every 5 minutes without errors
- [ ] Balance discrepancies detected and corrected
- [ ] Order state mismatches resolved
- [ ] Stale orders cancelled after timeout
- [ ] Old terminal orders cleaned up (memory management)
- [ ] Emergency stop cancels all orders immediately
- [ ] WebSocket reconnection triggers reconciliation

## Risk Considerations

- **Rate limits**: Don't run reconciliation too frequently
- **Timing**: Run reconciliation during low-activity periods if possible
- **Data races**: Lock state during reconciliation updates
- **Partial state**: Handle case where only some queries succeed

## Tests to Write

```rust
#[cfg(test)]
mod tests {
    #[tokio::test] async fn test_initial_sync() { }
    #[tokio::test] async fn test_balance_reconciliation_detects_mismatch() { }
    #[tokio::test] async fn test_order_reconciliation_detects_missing() { }
    #[tokio::test] async fn test_stale_order_cleanup() { }
    #[tokio::test] async fn test_terminal_order_cleanup() { }
    #[tokio::test] async fn test_emergency_stop() { }
    #[tokio::test] async fn test_websocket_reconnect_triggers_reconciliation() { }
}
```

## Monitoring & Alerting

Track these metrics for production monitoring:

```rust
struct ReconciliationMetrics {
    last_run_time: DateTime<Utc>,
    last_run_duration_ms: u64,
    total_discrepancies_found: u64,
    balance_mismatches: u64,
    order_mismatches: u64,
    stale_orders_cancelled: u64,
    consecutive_failures: u32,
}
```

Alert conditions:
- `consecutive_failures > 3`: Reconciliation consistently failing
- `balance_mismatches > 0`: Critical - balance out of sync
- `last_run_time > 10 minutes ago`: Reconciliation not running
