pub mod analyzer;
pub mod cache;
pub mod processor;

pub use analyzer::*;
pub use processor::MarketDataProcessor;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_module_exports() {
        // Test that module exports are accessible
        // This is a basic test to ensure the module structure is correct
    }

    #[test]
    fn test_analyzer_types_accessible() {
        // Verify analyzer types are re-exported
        use crate::market_data::TradingSignal;

        // Test that we can construct TradingSignal
        let signal = TradingSignal::Buy;
        match signal {
            TradingSignal::Buy => assert!(true),
            _ => panic!("Signal should be Buy"),
        }
    }

    #[test]
    fn test_cache_types_accessible() {
        // Verify cache types are accessible
        use crate::market_data::cache::MarketDataCache;

        let cache = MarketDataCache::new(100);
        assert!(cache.get_supported_symbols().is_empty());
    }

    #[test]
    fn test_processor_type_accessible() {
        // Verify processor type is re-exported
        // This is mainly a compile-time check
        let _type_check: Option<MarketDataProcessor> = None;
    }
}
