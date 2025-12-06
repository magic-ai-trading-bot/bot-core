// @spec:FR-REAL-001 - Real Trading Module
// @ref:specs/01-requirements/1.1-functional-requirements/FR-TRADING.md
// @test:TC-REAL-001

//! Real Trading Module
//!
//! This module provides production-ready real trading capabilities using the Binance API.
//! It mirrors the architecture of the paper trading engine but executes actual orders.
//!
//! # Features
//!
//! - **Real Order Execution**: Place and manage spot orders via Binance API
//! - **Position Tracking**: Track positions with real fills from ExecutionReports
//! - **Risk Management**: Pre-trade risk checks, position limits, daily loss limits
//! - **Circuit Breaker**: Automatic trading halt on consecutive errors
//! - **Event Broadcasting**: Real-time events for order fills, position updates
//!
//! # Safety
//!
//! - Defaults to testnet mode
//! - Configuration validation before trading
//! - Circuit breaker prevents cascade failures
//! - Execution lock prevents race conditions
//!
//! # Example
//!
//! ```rust,ignore
//! use binance_trading_bot::real_trading::{RealTradingEngine, RealTradingConfig};
//! use binance_trading_bot::binance::types::{SpotSide};
//!
//! // Create engine with testnet config
//! let config = RealTradingConfig::testnet_default();
//! let engine = RealTradingEngine::new(config, binance_client, risk_manager).await?;
//!
//! // Start engine
//! engine.start().await?;
//!
//! // Place a market buy order
//! let order = engine.place_market_order(
//!     "BTCUSDT",
//!     SpotSide::Buy,
//!     0.001,
//!     None,
//!     true,
//! ).await?;
//!
//! // Stop engine (cancels open orders)
//! engine.stop().await?;
//! ```

mod config;
mod engine;
mod order;
mod position;
mod risk;

// Re-export main types
pub use config::RealTradingConfig;
pub use engine::{
    Balance, CircuitBreakerState, DailyMetrics, RealTradingEngine, RealTradingEvent,
    ReconciliationMetrics,
};
// Also re-export Balance for API usage
pub use engine::Balance as RealTradingBalance;
pub use order::{OrderFill, OrderState, RealOrder};
pub use position::{PositionSide, RealPosition};
pub use risk::{RealTradingRiskManager, RiskValidationResult};
