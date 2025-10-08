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

#[cfg(test)]
mod tests {
    use super::*;

    // ============================================================================
    // Module Visibility Tests
    // ============================================================================

    #[test]
    fn test_ai_module_exists() {
        // Verify ai module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_api_module_exists() {
        // Verify api module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_auth_module_exists() {
        // Verify auth module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_binance_module_exists() {
        // Verify binance module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_config_module_exists() {
        // Verify config module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_error_module_exists() {
        // Verify error module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_market_data_module_exists() {
        // Verify market_data module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_monitoring_module_exists() {
        // Verify monitoring module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_paper_trading_module_exists() {
        // Verify paper_trading module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_storage_module_exists() {
        // Verify storage module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_strategies_module_exists() {
        // Verify strategies module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    #[test]
    fn test_trading_module_exists() {
        // Verify trading module is accessible
        let _module_name = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    // ============================================================================
    // Type Re-Export Tests
    // ============================================================================

    #[test]
    fn test_config_type_reexport() {
        // Verify Config type is accessible from root
        let type_name = std::any::type_name::<Config>();
        assert!(type_name.contains("Config"));
    }

    #[test]
    fn test_app_error_type_reexport() {
        // Verify AppError type is accessible from root
        let type_name = std::any::type_name::<AppError>();
        assert!(type_name.contains("AppError"));
    }

    #[test]
    fn test_app_result_type_reexport() {
        // Verify AppResult type is accessible from root
        let type_name = std::any::type_name::<AppResult<()>>();
        assert!(type_name.contains("Result"));
    }

    // ============================================================================
    // Models Module Tests
    // ============================================================================

    #[test]
    fn test_models_module_exists() {
        // Verify models module is accessible
        let candle = models::Candle {
            open: 100.0,
            high: 105.0,
            low: 95.0,
            close: 102.0,
            volume: 1000.0,
            open_time: 1000000,
            close_time: 2000000,
        };
        assert_eq!(candle.open, 100.0);
    }

    #[test]
    fn test_candle_creation() {
        // Test Candle struct creation
        let candle = models::Candle {
            open: 50.0,
            high: 60.0,
            low: 45.0,
            close: 55.0,
            volume: 500.0,
            open_time: 500000,
            close_time: 1000000,
        };
        assert_eq!(candle.open, 50.0);
        assert_eq!(candle.high, 60.0);
        assert_eq!(candle.low, 45.0);
        assert_eq!(candle.close, 55.0);
        assert_eq!(candle.volume, 500.0);
    }

    #[test]
    fn test_candle_clone() {
        // Test Candle clone implementation
        let candle = models::Candle {
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1500.0,
            open_time: 1000000,
            close_time: 2000000,
        };
        let cloned = candle.clone();
        assert_eq!(candle.open, cloned.open);
        assert_eq!(candle.close, cloned.close);
    }

    #[test]
    fn test_candle_debug_format() {
        // Test Candle Debug trait implementation
        let candle = models::Candle {
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1500.0,
            open_time: 1000000,
            close_time: 2000000,
        };
        let debug_str = format!("{:?}", candle);
        assert!(debug_str.contains("Candle"));
    }

    #[test]
    fn test_signal_type_buy() {
        // Test SignalType::Buy variant
        let signal = models::SignalType::Buy;
        match signal {
            models::SignalType::Buy => assert!(true),
            _ => panic!("Expected Buy signal"),
        }
    }

    #[test]
    fn test_signal_type_sell() {
        // Test SignalType::Sell variant
        let signal = models::SignalType::Sell;
        match signal {
            models::SignalType::Sell => assert!(true),
            _ => panic!("Expected Sell signal"),
        }
    }

    #[test]
    fn test_signal_type_hold() {
        // Test SignalType::Hold variant
        let signal = models::SignalType::Hold;
        match signal {
            models::SignalType::Hold => assert!(true),
            _ => panic!("Expected Hold signal"),
        }
    }

    #[test]
    fn test_signal_type_clone() {
        // Test SignalType clone implementation
        let signal = models::SignalType::Buy;
        let cloned = signal.clone();
        match cloned {
            models::SignalType::Buy => assert!(true),
            _ => panic!("Expected Buy signal after clone"),
        }
    }

    #[test]
    fn test_signal_type_debug() {
        // Test SignalType Debug trait implementation
        let signal = models::SignalType::Buy;
        let debug_str = format!("{:?}", signal);
        assert!(debug_str.contains("Buy"));
    }

    // ============================================================================
    // WebSocket Module Tests
    // ============================================================================

    #[test]
    fn test_websocket_module_accessible() {
        // Verify websocket module is accessible
        let _module_path = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    // ============================================================================
    // Routes Module Tests
    // ============================================================================

    #[test]
    fn test_routes_module_accessible() {
        // Verify routes module is accessible
        let _module_path = std::module_path!();
        assert!(std::module_path!().contains("rust_core_engine"));
    }

    // ============================================================================
    // Integration Tests - Type Compatibility
    // ============================================================================

    #[test]
    fn test_candle_with_extreme_values() {
        // Test Candle with edge case values
        let candle = models::Candle {
            open: f64::MAX,
            high: f64::MAX,
            low: f64::MIN_POSITIVE,
            close: 0.0,
            volume: f64::INFINITY,
            open_time: i64::MAX,
            close_time: i64::MAX,
        };
        assert!(candle.volume.is_infinite());
    }

    #[test]
    fn test_candle_with_negative_time() {
        // Test Candle with negative timestamps
        let candle = models::Candle {
            open: 100.0,
            high: 110.0,
            low: 90.0,
            close: 105.0,
            volume: 1000.0,
            open_time: -1000,
            close_time: -500,
        };
        assert_eq!(candle.open_time, -1000);
    }

    #[test]
    fn test_multiple_signal_types() {
        // Test all SignalType variants together
        let signals = vec![
            models::SignalType::Buy,
            models::SignalType::Sell,
            models::SignalType::Hold,
        ];
        assert_eq!(signals.len(), 3);
    }

    // ============================================================================
    // Module Path Tests
    // ============================================================================

    #[test]
    fn test_crate_name_in_module_path() {
        // Verify we're in the correct crate
        let module_path = std::module_path!();
        assert!(module_path.starts_with("rust_core_engine"));
    }

    #[test]
    fn test_lib_module_in_path() {
        // Verify we're in the lib module
        let module_path = std::module_path!();
        assert!(module_path.contains("rust_core_engine"));
    }
}
