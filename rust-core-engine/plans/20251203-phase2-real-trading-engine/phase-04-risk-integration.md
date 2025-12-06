# Phase 4: Risk Integration

## Context

Reference existing implementations:
- `src/trading/risk_manager.rs:41-167` - RiskManager methods
- `src/paper_trading/engine.rs:847-1039` - Risk checks in paper trading
- `src/config.rs` - TradingConfig structure

## Requirements

1. **Pre-trade validation** - Check risk limits before order placement
2. **Post-trade updates** - Update exposure/PnL tracking after fills
3. **Daily loss limit** - Stop trading when daily loss threshold hit
4. **Circuit breaker** - Pause on consecutive errors
5. **Exposure limits** - Max position size, total exposure caps

## Implementation Steps

### 4.1 Enhanced Risk Manager

```rust
// @spec:FR-REAL-040 - Real Trading Risk Manager
// src/real_trading/risk.rs

pub struct RealTradingRiskManager {
    config: RealTradingConfig,
    daily_loss: Arc<RwLock<f64>>,
    daily_trades: Arc<RwLock<u32>>,
    last_reset: Arc<RwLock<DateTime<Utc>>>,
}

impl RealTradingRiskManager {
    pub fn new(config: RealTradingConfig) -> Self {
        Self {
            config,
            daily_loss: Arc::new(RwLock::new(0.0)),
            daily_trades: Arc::new(RwLock::new(0)),
            last_reset: Arc::new(RwLock::new(Utc::now())),
        }
    }

    /// Check if new day, reset counters
    pub async fn check_daily_reset(&self) {
        let last = *self.last_reset.read().await;
        let now = Utc::now();

        if now.date_naive() != last.date_naive() {
            *self.daily_loss.write().await = 0.0;
            *self.daily_trades.write().await = 0;
            *self.last_reset.write().await = now;
            info!("Daily risk counters reset");
        }
    }

    /// Record a trade result
    pub async fn record_trade(&self, pnl: f64) {
        let mut daily_loss = self.daily_loss.write().await;
        *daily_loss += if pnl < 0.0 { pnl.abs() } else { 0.0 };

        let mut trades = self.daily_trades.write().await;
        *trades += 1;
    }
}
```

### 4.2 Pre-Trade Validation

```rust
// @spec:FR-REAL-041 - Pre-Trade Risk Validation
impl RealTradingRiskManager {
    pub async fn validate_order(
        &self,
        symbol: &str,
        side: OrderSide,
        quantity: f64,
        price: f64,
        current_positions: &DashMap<String, RealPosition>,
        balances: &HashMap<String, f64>,
    ) -> Result<()> {
        self.check_daily_reset().await;

        // 1. Check daily loss limit
        let daily_loss = *self.daily_loss.read().await;
        if daily_loss >= self.config.max_daily_loss_usdt {
            return Err(anyhow!(
                "Daily loss limit reached: ${:.2} >= ${:.2}",
                daily_loss, self.config.max_daily_loss_usdt
            ));
        }

        // 2. Check max positions
        let position_count = current_positions.len();
        let has_position = current_positions.contains_key(symbol);
        if !has_position && position_count >= self.config.max_positions as usize {
            return Err(anyhow!(
                "Max positions reached: {} >= {}",
                position_count, self.config.max_positions
            ));
        }

        // 3. Check position size limit
        let order_value = quantity * price;
        if order_value > self.config.max_position_size_usdt {
            return Err(anyhow!(
                "Order value ${:.2} exceeds max position size ${:.2}",
                order_value, self.config.max_position_size_usdt
            ));
        }

        // 4. Check total exposure limit
        let current_exposure: f64 = current_positions
            .iter()
            .map(|p| p.value().quantity * p.value().current_price)
            .sum();

        if current_exposure + order_value > self.config.max_total_exposure_usdt {
            return Err(anyhow!(
                "Total exposure ${:.2} + ${:.2} exceeds limit ${:.2}",
                current_exposure, order_value, self.config.max_total_exposure_usdt
            ));
        }

        // 5. Check available balance
        let usdt_balance = balances.get("USDT").copied().unwrap_or(0.0);
        if side == OrderSide::Buy && order_value > usdt_balance {
            return Err(anyhow!(
                "Insufficient balance: need ${:.2}, have ${:.2}",
                order_value, usdt_balance
            ));
        }

        // 6. Check risk per trade (% of balance)
        let max_risk = usdt_balance * (self.config.risk_per_trade_percent / 100.0);
        if order_value > max_risk * 5.0 {  // Allow 5x leverage
            return Err(anyhow!(
                "Order value ${:.2} exceeds risk limit ${:.2}",
                order_value, max_risk * 5.0
            ));
        }

        Ok(())
    }
}
```

### 4.3 Calculate Position Size

```rust
// @spec:FR-REAL-042 - Risk-Based Position Sizing
impl RealTradingRiskManager {
    /// Calculate position size based on risk parameters
    pub fn calculate_position_size(
        &self,
        entry_price: f64,
        stop_loss: f64,
        account_balance: f64,
    ) -> f64 {
        if entry_price <= 0.0 || account_balance <= 0.0 {
            return 0.0;
        }

        // Risk amount = account * risk_percentage
        let risk_amount = account_balance * (self.config.risk_per_trade_percent / 100.0);

        // Stop loss distance as decimal
        let stop_distance = (entry_price - stop_loss).abs() / entry_price;

        // Minimum stop distance to prevent huge positions
        let min_stop_distance = 0.005;  // 0.5%
        if stop_distance < min_stop_distance {
            warn!("Stop loss too tight, using minimum distance");
            let position_value = risk_amount / min_stop_distance;
            return (position_value / entry_price).min(self.config.max_position_size_usdt / entry_price);
        }

        // Position size = risk_amount / (entry * stop_distance)
        let position_value = risk_amount / stop_distance;
        let position_size = position_value / entry_price;

        // Apply limits
        let max_size = self.config.max_position_size_usdt / entry_price;
        position_size.min(max_size)
    }
}
```

### 4.4 Circuit Breaker

```rust
// @spec:FR-REAL-043 - Circuit Breaker
impl RealTradingEngine {
    async fn is_circuit_breaker_open(&self) -> bool {
        let state = self.circuit_breaker.read().await;
        if !state.is_open {
            return false;
        }

        // Check if cooldown expired
        if let Some(opened_at) = state.opened_at {
            let cooldown = Duration::from_secs(self.config.read().await.circuit_breaker_cooldown_secs);
            if Utc::now().signed_duration_since(opened_at).to_std().unwrap_or_default() > cooldown {
                drop(state);
                self.close_circuit_breaker().await;
                return false;
            }
        }

        true
    }

    async fn increment_consecutive_errors(&self, error: &str) {
        let mut errors = self.consecutive_errors.write().await;
        *errors += 1;

        let threshold = self.config.read().await.circuit_breaker_errors;
        if *errors >= threshold {
            drop(errors);
            self.open_circuit_breaker(error.to_string()).await;
        }
    }

    async fn reset_consecutive_errors(&self) {
        let mut errors = self.consecutive_errors.write().await;
        *errors = 0;
    }

    async fn open_circuit_breaker(&self, reason: String) {
        let mut state = self.circuit_breaker.write().await;
        if state.is_open {
            return;  // Already open
        }

        state.is_open = true;
        state.opened_at = Some(Utc::now());
        state.last_error = Some(reason.clone());

        warn!("Circuit breaker OPENED: {}", reason);

        let _ = self.event_tx.send(RealTradingEvent::CircuitBreakerOpened(reason));
    }

    async fn close_circuit_breaker(&self) {
        let mut state = self.circuit_breaker.write().await;
        state.is_open = false;
        state.error_count = 0;
        state.opened_at = None;
        state.last_error = None;

        info!("Circuit breaker CLOSED");

        let _ = self.event_tx.send(RealTradingEvent::CircuitBreakerClosed);
    }
}
```

### 4.5 Post-Trade Risk Updates

```rust
// @spec:FR-REAL-044 - Post-Trade Risk Tracking
impl RealTradingEngine {
    async fn post_trade_risk_update(&self, pnl: f64) {
        // Record in risk manager
        // (Assuming risk_manager has a record_trade method)
        let daily_pnl = {
            let mut daily = self.daily_pnl.write().await;
            *daily += pnl;
            *daily
        };

        // Check if daily loss limit hit
        let config = self.config.read().await;
        if daily_pnl < -config.max_daily_loss_usdt {
            warn!(
                "Daily loss limit hit: ${:.2} < -${:.2}",
                daily_pnl, config.max_daily_loss_usdt
            );
            self.open_circuit_breaker("Daily loss limit reached".to_string()).await;
        }

        // Log metrics
        info!(
            "Trade PnL: ${:.2}, Daily PnL: ${:.2}",
            pnl, daily_pnl
        );
    }
}
```

### 4.6 Stop-Loss/Take-Profit Management

```rust
// @spec:FR-REAL-045 - SL/TP Order Management
impl RealTradingEngine {
    pub async fn set_stop_loss(
        &self,
        symbol: &str,
        stop_price: f64,
    ) -> Result<RealOrder> {
        let position = self.positions.get(symbol)
            .ok_or_else(|| anyhow!("No position for {}", symbol))?
            .clone();

        // Cancel existing SL order if any
        if let Some(sl_order_id) = &position.stop_loss_order_id {
            let _ = self.cancel_order(sl_order_id).await;
        }

        // Place new SL order
        let side = if position.side == "LONG" { OrderSide::Sell } else { OrderSide::Buy };
        let sl_order = self.place_stop_loss_order(
            symbol,
            side,
            position.quantity,
            stop_price,
            stop_price * 0.99,  // Limit price slightly below stop
        ).await?;

        // Update position
        if let Some(mut pos) = self.positions.get_mut(symbol) {
            pos.stop_loss = Some(stop_price);
            pos.stop_loss_order_id = Some(sl_order.id.clone());
        }

        Ok(sl_order)
    }

    pub async fn set_take_profit(
        &self,
        symbol: &str,
        take_profit_price: f64,
    ) -> Result<RealOrder> {
        let position = self.positions.get(symbol)
            .ok_or_else(|| anyhow!("No position for {}", symbol))?
            .clone();

        // Cancel existing TP order if any
        if let Some(tp_order_id) = &position.take_profit_order_id {
            let _ = self.cancel_order(tp_order_id).await;
        }

        // Place new TP order
        let side = if position.side == "LONG" { OrderSide::Sell } else { OrderSide::Buy };
        let tp_order = self.place_take_profit_order(
            symbol,
            side,
            position.quantity,
            take_profit_price,
            take_profit_price * 1.01,  // Limit price slightly above for safety
        ).await?;

        // Update position
        if let Some(mut pos) = self.positions.get_mut(symbol) {
            pos.take_profit = Some(take_profit_price);
            pos.take_profit_order_id = Some(tp_order.id.clone());
        }

        Ok(tp_order)
    }
}
```

## Success Criteria

- [ ] Pre-trade validation catches all limit violations
- [ ] Position size calculation respects risk parameters
- [ ] Circuit breaker opens after N consecutive errors
- [ ] Circuit breaker auto-closes after cooldown period
- [ ] Daily loss tracking resets at midnight UTC
- [ ] Stop-loss/take-profit orders placed correctly
- [ ] All risk checks run in <10ms

## Risk Considerations

- **Race conditions**: Lock order during risk check -> placement
- **Clock drift**: Use server time for daily reset
- **Partial fills**: SL/TP orders may need quantity adjustments
- **False positives**: Circuit breaker shouldn't trigger on expected errors

## Tests to Write

```rust
#[cfg(test)]
mod tests {
    #[tokio::test] async fn test_validate_order_success() { }
    #[tokio::test] async fn test_validate_order_daily_loss_limit() { }
    #[tokio::test] async fn test_validate_order_max_positions() { }
    #[tokio::test] async fn test_validate_order_position_size_limit() { }
    #[tokio::test] async fn test_validate_order_exposure_limit() { }
    #[tokio::test] async fn test_validate_order_insufficient_balance() { }
    #[tokio::test] async fn test_calculate_position_size() { }
    #[tokio::test] async fn test_circuit_breaker_opens() { }
    #[tokio::test] async fn test_circuit_breaker_closes_after_cooldown() { }
    #[tokio::test] async fn test_daily_reset() { }
    #[tokio::test] async fn test_set_stop_loss() { }
    #[tokio::test] async fn test_set_take_profit() { }
}
```
