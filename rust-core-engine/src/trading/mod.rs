pub mod engine;
pub mod position_manager;
pub mod risk_manager;

pub use engine::TradingEngine;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that we can access the exported types
        // This ensures the module structure is correct
        let _position_manager = position_manager::PositionManager::new();
        assert!(true, "PositionManager is accessible");
    }

    #[test]
    fn test_trading_engine_type_exported() {
        // Verify TradingEngine is properly re-exported
        use crate::trading::TradingEngine;
        // If this compiles, the export is working
        let type_name = std::any::type_name::<TradingEngine>();
        assert!(type_name.contains("TradingEngine"));
    }
}
