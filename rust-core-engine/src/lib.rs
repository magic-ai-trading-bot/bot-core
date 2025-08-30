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
pub use auth::models;
pub use config::Config;
pub use error::{AppError, AppResult};

// Re-export routes for tests
pub mod routes {
    pub use crate::api::*;
    pub use crate::auth::handlers::*;
}
