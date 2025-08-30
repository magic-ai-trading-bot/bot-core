// Re-export modules for tests
pub mod ai;
pub mod api;
pub mod auth;
pub mod binance;
pub mod config;
pub mod error;
pub mod market_data;
pub mod monitoring;
pub mod paper_trading;
pub mod storage;
pub mod strategies;
pub mod trading;

// Re-export commonly used types
pub use config::Config;
pub use error::{AppError, AppResult};

// Re-export models
pub mod models {
    pub use crate::auth::models::*;
    pub use crate::binance::types::{Candle, OrderSide, OrderType, TimeInForce};
    pub use crate::strategies::types::{Signal, SignalType};
}

// Re-export websocket for tests
pub mod websocket {
    pub use crate::binance::websocket::*;
}

// Re-export routes for tests
pub mod routes {
    pub use crate::api::*;
    pub use crate::auth::handlers::*;
}
