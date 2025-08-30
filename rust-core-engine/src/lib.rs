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
    // Types from binance module (if they exist)
    // pub use crate::binance::types::{Candle, OrderSide, OrderType, TimeInForce};
    
    // Types from strategies module
    // pub use crate::strategies::types::{Signal, SignalType};
    
    // Define Candle type for tests
    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
    pub struct Candle {
        pub open: f64,
        pub high: f64,
        pub low: f64,
        pub close: f64,
        pub volume: f64,
        pub open_time: i64,
        pub close_time: i64,
    }
    
    #[derive(Debug, Clone)]
    pub enum SignalType {
        Buy,
        Sell,
        Hold,
    }
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
