# Phase 3: Position Tracking

## Context

Reference existing implementations:
- `src/binance/user_data_stream.rs:196-356` - Event handling loop
- `src/binance/types.rs:609-740` - ExecutionReport struct and methods
- `src/trading/position_manager.rs` - Position management pattern

## Requirements

1. **ExecutionReport processing** - Update positions from WebSocket events
2. **Partial fill handling** - Accumulate fills into positions correctly
3. **Position lifecycle** - Open, update, close positions based on fills
4. **PnL tracking** - Calculate realized/unrealized PnL accurately

## Implementation Steps

### 3.1 Start UserDataStream Listener

```rust
// @spec:FR-REAL-030 - User Data Stream Integration
impl RealTradingEngine {
    pub async fn start(&mut self) -> Result<()> {
        // 1. Set running flag
        {
            let mut running = self.is_running.write().await;
            if *running {
                return Err(anyhow!("Engine already running"));
            }
            *running = true;
        }

        // 2. Start UserDataStream
        {
            let mut stream = self.user_data_stream.write().await;
            stream.start().await?;
        }

        // 3. Subscribe to events and spawn handler
        let event_rx = {
            let stream = self.user_data_stream.read().await;
            stream.subscribe()
        };

        let engine = self.clone();
        tokio::spawn(async move {
            engine.process_user_data_events(event_rx).await;
        });

        // 4. Start reconciliation loop
        let engine = self.clone();
        tokio::spawn(async move {
            engine.reconciliation_loop().await;
        });

        // 5. Load initial state from REST API
        self.initial_sync().await?;

        info!("Real trading engine started");
        Ok(())
    }

    async fn process_user_data_events(
        &self,
        mut rx: broadcast::Receiver<UserDataStreamEvent>,
    ) {
        loop {
            match rx.recv().await {
                Ok(event) => {
                    match event {
                        UserDataStreamEvent::ExecutionReport(report) => {
                            if let Err(e) = self.handle_execution_report(*report).await {
                                error!("Failed to process execution report: {}", e);
                            }
                        },
                        UserDataStreamEvent::AccountPosition(pos) => {
                            self.handle_account_position(pos).await;
                        },
                        UserDataStreamEvent::BalanceUpdate(update) => {
                            self.handle_balance_update(update).await;
                        },
                        UserDataStreamEvent::Disconnected => {
                            warn!("UserDataStream disconnected, will reconnect...");
                        },
                        UserDataStreamEvent::Error(e) => {
                            error!("UserDataStream error: {}", e);
                            self.increment_consecutive_errors(&e).await;
                        },
                        _ => {}
                    }
                },
                Err(broadcast::error::RecvError::Lagged(n)) => {
                    warn!("Lagged {} events, may need reconciliation", n);
                },
                Err(broadcast::error::RecvError::Closed) => {
                    error!("UserDataStream channel closed");
                    break;
                }
            }
        }
    }
}
```

### 3.2 Handle Execution Report

```rust
// @spec:FR-REAL-031 - Execution Report Processing
impl RealTradingEngine {
    async fn handle_execution_report(&self, report: ExecutionReport) -> Result<()> {
        info!(
            "Execution report: {} {} {} - Status: {}, Exec: {}, Filled: {}/{}",
            report.symbol, report.side, report.order_type,
            report.order_status, report.execution_type,
            report.cumulative_filled_quantity, report.order_quantity
        );

        // 1. Update order state
        let order_updated = self.update_order_from_report(&report).await;

        // 2. Handle based on execution type
        match report.execution_type.as_str() {
            "NEW" => {
                // Order confirmed by exchange
                debug!("Order {} confirmed", report.client_order_id);
            },
            "TRADE" => {
                // Fill occurred - update position
                self.process_trade_fill(&report).await?;
            },
            "CANCELED" => {
                // Order cancelled
                let _ = self.event_tx.send(RealTradingEvent::OrderCancelled(order_updated));
            },
            "REJECTED" => {
                let _ = self.event_tx.send(RealTradingEvent::OrderRejected {
                    order: order_updated,
                    reason: report.order_reject_reason.clone(),
                });
            },
            "EXPIRED" => {
                debug!("Order {} expired", report.client_order_id);
            },
            _ => {
                warn!("Unknown execution type: {}", report.execution_type);
            }
        }

        Ok(())
    }

    async fn update_order_from_report(&self, report: &ExecutionReport) -> RealOrder {
        let order_id = &report.client_order_id;

        // Find order by client ID or exchange ID
        let order_key = if self.orders.contains_key(order_id) {
            Some(order_id.clone())
        } else {
            // Search by exchange order ID
            self.orders.iter()
                .find(|e| e.value().exchange_order_id == report.order_id)
                .map(|e| e.key().clone())
        };

        if let Some(key) = order_key {
            if let Some(mut order) = self.orders.get_mut(&key) {
                order.exchange_order_id = report.order_id;
                order.state = OrderState::from_status(&report.order_status);
                order.executed_quantity = parse_f64(&report.cumulative_filled_quantity);
                order.remaining_quantity = order.original_quantity - order.executed_quantity;
                order.updated_at = Utc::now();

                // Add fill if this is a trade
                if report.is_trade() {
                    let fill = OrderFill {
                        trade_id: report.trade_id,
                        price: parse_f64(&report.last_executed_price),
                        quantity: parse_f64(&report.last_executed_quantity),
                        commission: parse_f64(&report.commission_amount),
                        commission_asset: report.commission_asset.clone().unwrap_or_default(),
                        timestamp: Utc::now(),
                    };
                    order.fills.push(fill);

                    // Recalculate average fill price
                    order.average_fill_price = order.calculate_average_fill_price();
                }

                return order.clone();
            }
        }

        // Order not found locally - create from report
        warn!("Order {} not found locally, creating from report", order_id);
        let order = RealOrder::from_execution_report(report);
        self.orders.insert(order_id.clone(), order.clone());
        order
    }
}
```

### 3.3 Process Trade Fill (Position Update)

```rust
// @spec:FR-REAL-032 - Position Update from Fills
impl RealTradingEngine {
    async fn process_trade_fill(&self, report: &ExecutionReport) -> Result<()> {
        let symbol = &report.symbol;
        let side = &report.side;
        let fill_qty = parse_f64(&report.last_executed_quantity);
        let fill_price = parse_f64(&report.last_executed_price);

        // Determine if this is opening or closing a position
        let existing_position = self.positions.get(symbol).map(|p| p.clone());

        match existing_position {
            Some(mut position) => {
                // Position exists - check if adding to or reducing
                let is_same_side = (position.side == "LONG" && side == "BUY")
                    || (position.side == "SHORT" && side == "SELL");

                if is_same_side {
                    // Adding to position - update average entry
                    self.add_to_position(symbol, fill_price, fill_qty).await;
                } else {
                    // Reducing/closing position
                    self.reduce_position(symbol, fill_price, fill_qty).await;
                }
            },
            None => {
                // No position - open new one
                self.open_position(&report, fill_price, fill_qty).await;
            }
        }

        // Broadcast position update
        if let Some(position) = self.positions.get(symbol) {
            let _ = self.event_tx.send(RealTradingEvent::PositionUpdated(position.clone()));
        }

        Ok(())
    }

    async fn open_position(&self, report: &ExecutionReport, price: f64, quantity: f64) {
        let position = RealPosition {
            id: Uuid::new_v4().to_string(),
            symbol: report.symbol.clone(),
            side: if report.side == "BUY" { "LONG".to_string() } else { "SHORT".to_string() },
            quantity,
            entry_price: price,
            current_price: price,
            unrealized_pnl: 0.0,
            realized_pnl: 0.0,
            stop_loss: None,
            take_profit: None,
            stop_loss_order_id: None,
            take_profit_order_id: None,
            entry_order_ids: vec![report.client_order_id.clone()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.positions.insert(report.symbol.clone(), position.clone());
        let _ = self.event_tx.send(RealTradingEvent::PositionOpened(position));

        info!(
            "Opened {} position: {} {} @ {}",
            if report.side == "BUY" { "LONG" } else { "SHORT" },
            quantity, report.symbol, price
        );
    }

    async fn add_to_position(&self, symbol: &str, price: f64, quantity: f64) {
        if let Some(mut position) = self.positions.get_mut(symbol) {
            // Calculate new average entry price
            let total_value = position.entry_price * position.quantity + price * quantity;
            let total_qty = position.quantity + quantity;

            position.entry_price = total_value / total_qty;
            position.quantity = total_qty;
            position.updated_at = Utc::now();

            info!(
                "Added to position: {} {} @ {} (avg entry: {})",
                quantity, symbol, price, position.entry_price
            );
        }
    }

    async fn reduce_position(&self, symbol: &str, price: f64, quantity: f64) {
        let should_remove = {
            if let Some(mut position) = self.positions.get_mut(symbol) {
                // Calculate realized PnL for this reduction
                let pnl = if position.side == "LONG" {
                    (price - position.entry_price) * quantity
                } else {
                    (position.entry_price - price) * quantity
                };

                position.realized_pnl += pnl;
                position.quantity -= quantity;
                position.updated_at = Utc::now();

                // Update daily PnL
                {
                    let mut daily = self.daily_pnl.write().await;
                    *daily += pnl;
                }

                info!(
                    "Reduced position: {} {} @ {} (PnL: {:.2})",
                    quantity, symbol, price, pnl
                );

                position.quantity <= 0.0001  // Use small threshold for float comparison
            } else {
                false
            }
        };

        // Close position if fully reduced
        if should_remove {
            if let Some((_, position)) = self.positions.remove(symbol) {
                let _ = self.event_tx.send(RealTradingEvent::PositionClosed {
                    position: position.clone(),
                    pnl: position.realized_pnl,
                });
                info!("Position closed: {} (Total PnL: {:.2})", symbol, position.realized_pnl);
            }
        }
    }
}
```

### 3.4 Handle Account/Balance Updates

```rust
// @spec:FR-REAL-033 - Balance Tracking
impl RealTradingEngine {
    async fn handle_account_position(&self, pos: OutboundAccountPosition) {
        let mut balances = self.balances.write().await;

        for balance in pos.balances {
            let free = parse_f64(&balance.free);
            let locked = parse_f64(&balance.locked);

            balances.insert(balance.asset.clone(), free);

            let _ = self.event_tx.send(RealTradingEvent::BalanceUpdated {
                asset: balance.asset,
                free,
                locked,
            });
        }
    }

    async fn handle_balance_update(&self, update: BalanceUpdate) {
        let delta = parse_f64(&update.balance_delta);

        let mut balances = self.balances.write().await;
        let current = balances.get(&update.asset).copied().unwrap_or(0.0);
        let new_balance = current + delta;
        balances.insert(update.asset.clone(), new_balance);

        info!(
            "Balance update: {} delta {} (new: {})",
            update.asset, delta, new_balance
        );
    }

    pub async fn get_balance(&self, asset: &str) -> f64 {
        let balances = self.balances.read().await;
        balances.get(asset).copied().unwrap_or(0.0)
    }

    pub async fn get_total_equity_usdt(&self) -> f64 {
        let balances = self.balances.read().await;

        // Sum USDT balance + position values
        let usdt_balance = balances.get("USDT").copied().unwrap_or(0.0);

        let position_value: f64 = self.positions
            .iter()
            .map(|p| p.value().quantity * p.value().current_price)
            .sum();

        usdt_balance + position_value
    }
}
```

### 3.5 Price Update Loop

```rust
// @spec:FR-REAL-034 - Position Price Updates
impl RealTradingEngine {
    pub async fn update_position_prices(&self, prices: &HashMap<String, f64>) {
        for mut entry in self.positions.iter_mut() {
            let position = entry.value_mut();

            if let Some(&price) = prices.get(&position.symbol) {
                position.current_price = price;
                position.unrealized_pnl = if position.side == "LONG" {
                    (price - position.entry_price) * position.quantity
                } else {
                    (position.entry_price - price) * position.quantity
                };
                position.updated_at = Utc::now();
            }
        }
    }
}

fn parse_f64(s: &str) -> f64 {
    s.parse().unwrap_or(0.0)
}
```

## Success Criteria

- [ ] ExecutionReport events update orders correctly
- [ ] Partial fills accumulate into positions properly
- [ ] Average entry price calculated correctly on position additions
- [ ] Realized PnL calculated correctly on position reductions
- [ ] Position closed when quantity reaches zero
- [ ] Balance updates reflected in real-time
- [ ] No duplicate position entries per symbol

## Risk Considerations

- **Event ordering**: WebSocket events may arrive out of order
- **Missed events**: Must reconcile periodically to catch missed fills
- **Float precision**: Use threshold comparisons for quantity checks
- **Race conditions**: Multiple fills for same order in quick succession

## Tests to Write

```rust
#[cfg(test)]
mod tests {
    #[tokio::test] async fn test_execution_report_new_order() { }
    #[tokio::test] async fn test_execution_report_partial_fill() { }
    #[tokio::test] async fn test_execution_report_full_fill() { }
    #[tokio::test] async fn test_open_new_position() { }
    #[tokio::test] async fn test_add_to_existing_position() { }
    #[tokio::test] async fn test_reduce_position_partial() { }
    #[tokio::test] async fn test_close_position_full() { }
    #[tokio::test] async fn test_average_entry_price_calculation() { }
    #[tokio::test] async fn test_realized_pnl_calculation() { }
    #[tokio::test] async fn test_balance_update() { }
}
```
