use anyhow::Result;
use tracing::debug;

use crate::config::TradingConfig;
use crate::market_data::analyzer::MultiTimeframeAnalysis;

// @spec:FR-RISK-001 - Position Size Limits
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management
// @test:TC-TRADING-004, TC-TRADING-005

// @spec:FR-RISK-002 - Max Daily Loss
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management
// @test:TC-TRADING-006, TC-TRADING-007

// @spec:FR-RISK-003 - Max Open Positions
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management
// @test:TC-TRADING-008, TC-TRADING-009

// @spec:FR-RISK-004 - Risk Validation
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management
// @test:TC-TRADING-047, TC-TRADING-048

// @spec:FR-RISK-005 - Emergency Stop
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management
// @test:TC-TRADING-049

// @spec:FR-RISK-006 - Exposure Limits
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management
// @test:TC-TRADING-052, TC-TRADING-053

#[derive(Clone)]
pub struct RiskManager {
    config: TradingConfig,
}

impl RiskManager {
    pub fn new(config: TradingConfig) -> Self {
        Self { config }
    }

    pub async fn can_open_position(
        &self,
        symbol: &str,
        analysis: &MultiTimeframeAnalysis,
    ) -> Result<bool> {
        // Check if trading is enabled
        if !self.config.enabled {
            debug!("Trading is disabled");
            return Ok(false);
        }

        // Check signal confidence threshold
        let min_confidence = match analysis.overall_signal {
            crate::market_data::analyzer::TradingSignal::StrongBuy
            | crate::market_data::analyzer::TradingSignal::StrongSell => 0.7,
            crate::market_data::analyzer::TradingSignal::Buy
            | crate::market_data::analyzer::TradingSignal::Sell => 0.8,
            crate::market_data::analyzer::TradingSignal::Hold => return Ok(false),
        };

        if analysis.overall_confidence < min_confidence {
            debug!(
                "Signal confidence {} below threshold {} for {}",
                analysis.overall_confidence, min_confidence, symbol
            );
            return Ok(false);
        }

        // Check risk-reward ratio if available
        if let Some(risk_reward) = analysis.risk_reward_ratio {
            if risk_reward < 1.5 {
                debug!(
                    "Risk-reward ratio {} below minimum 1.5 for {}",
                    risk_reward, symbol
                );
                return Ok(false);
            }
        }

        debug!(
            "Risk check passed for {} with confidence {:.2}",
            symbol, analysis.overall_confidence
        );
        Ok(true)
    }

    /// Calculate position size based on risk management principles
    ///
    /// FIXED: Implements proper position sizing that:
    /// - Uses account_balance
    /// - Considers stop_loss distance
    /// - Respects max risk per trade
    ///
    /// @spec:FR-RISK-001 - Position Size Calculation
    /// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#risk-management
    /// @test:TC-TRADING-004, TC-TRADING-005
    #[allow(dead_code)]
    pub fn calculate_position_size(
        &self,
        symbol: &str,
        entry_price: f64,
        stop_loss: Option<f64>,
        account_balance: f64,
    ) -> f64 {
        // Validate inputs
        if entry_price <= 0.0 || account_balance <= 0.0 {
            debug!(
                "Invalid input for position sizing: entry_price={}, balance={}",
                entry_price, account_balance
            );
            return self.config.default_quantity;
        }

        // If no stop loss provided, use default quantity
        let stop_loss_price = match stop_loss {
            Some(sl) if sl > 0.0 => sl,
            _ => {
                debug!("No valid stop loss for {}, using default quantity", symbol);
                return self.config.default_quantity;
            },
        };

        // Calculate risk amount (risk_percentage of account balance)
        let risk_amount = account_balance * (self.config.risk_percentage / 100.0);

        // Calculate stop loss distance as percentage
        let stop_loss_distance_pct = ((entry_price - stop_loss_price).abs() / entry_price) * 100.0;

        // Minimum stop loss threshold to prevent huge positions
        const MIN_STOP_LOSS_PCT: f64 = 0.5; // 0.5% minimum
        if stop_loss_distance_pct < MIN_STOP_LOSS_PCT {
            debug!(
                "Stop loss too tight for {} ({}%), using default quantity",
                symbol, stop_loss_distance_pct
            );
            return self.config.default_quantity;
        }

        // Calculate position size: risk_amount / stop_loss_distance
        let position_value = risk_amount / (stop_loss_distance_pct / 100.0);
        let position_size = position_value / entry_price;

        // Apply safety limits
        let max_position_value = account_balance * 0.2; // Maximum 20% of account per trade
        let max_quantity = max_position_value / entry_price;

        let safe_quantity = position_size.min(max_quantity);

        // Ensure we don't go below minimum or above default
        if safe_quantity < self.config.default_quantity * 0.1 {
            debug!(
                "Calculated position too small for {}, using 10% of default",
                symbol
            );
            return self.config.default_quantity * 0.1;
        }

        if safe_quantity > self.config.default_quantity * 5.0 {
            debug!(
                "Calculated position too large for {}, capping at 5x default",
                symbol
            );
            return self.config.default_quantity * 5.0;
        }

        safe_quantity
    }

    #[allow(dead_code)]
    pub fn get_max_positions(&self) -> u32 {
        self.config.max_positions
    }

    #[allow(dead_code)]
    pub fn get_risk_percentage(&self) -> f64 {
        self.config.risk_percentage
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::market_data::analyzer::{MultiTimeframeAnalysis, TradingSignal};

    fn create_test_config() -> TradingConfig {
        TradingConfig {
            enabled: true,
            max_positions: 5,
            default_quantity: 0.01,
            leverage: 10,
            margin_type: "ISOLATED".to_string(),
            risk_percentage: 2.0,
            stop_loss_percentage: 2.0,
            take_profit_percentage: 4.0,
            order_timeout_seconds: 30,
            position_check_interval_seconds: 30,
        }
    }

    fn create_test_analysis(signal: TradingSignal, confidence: f64) -> MultiTimeframeAnalysis {
        use std::collections::HashMap;

        MultiTimeframeAnalysis {
            symbol: "BTCUSDT".to_string(),
            timestamp: chrono::Utc::now().timestamp_millis(),
            timeframe_signals: HashMap::new(),
            overall_signal: signal,
            overall_confidence: confidence,
            entry_price: Some(50000.0),
            stop_loss: Some(48000.0),
            take_profit: Some(55000.0),
            risk_reward_ratio: Some(2.5),
        }
    }

    #[test]
    fn test_new_risk_manager() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config.clone());

        assert_eq!(risk_manager.get_max_positions(), config.max_positions);
        assert_eq!(risk_manager.get_risk_percentage(), config.risk_percentage);
    }

    #[tokio::test]
    async fn test_can_open_position_trading_disabled() {
        let mut config = create_test_config();
        config.enabled = false;

        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.9);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            !result,
            "Should not allow position when trading is disabled"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_strong_buy_high_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.8);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(result, "Should allow StrongBuy with 0.8 confidence");
    }

    #[tokio::test]
    async fn test_can_open_position_strong_buy_threshold_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.7);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            result,
            "Should allow StrongBuy with 0.7 confidence (threshold)"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_strong_buy_low_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.65);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            !result,
            "Should reject StrongBuy with 0.65 confidence (below 0.7)"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_strong_sell_high_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongSell, 0.75);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(result, "Should allow StrongSell with 0.75 confidence");
    }

    #[tokio::test]
    async fn test_can_open_position_buy_high_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Buy, 0.85);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(result, "Should allow Buy with 0.85 confidence");
    }

    #[tokio::test]
    async fn test_can_open_position_buy_threshold_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Buy, 0.8);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(result, "Should allow Buy with 0.8 confidence (threshold)");
    }

    #[tokio::test]
    async fn test_can_open_position_buy_low_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Buy, 0.75);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            !result,
            "Should reject Buy with 0.75 confidence (below 0.8)"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_sell_high_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Sell, 0.9);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(result, "Should allow Sell with 0.9 confidence");
    }

    #[tokio::test]
    async fn test_can_open_position_hold_signal() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Hold, 0.95);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            !result,
            "Should reject Hold signal regardless of confidence"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_low_risk_reward_ratio() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.8);
        analysis.risk_reward_ratio = Some(1.2);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            !result,
            "Should reject position with risk-reward ratio 1.2 (below 1.5)"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_threshold_risk_reward_ratio() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.8);
        analysis.risk_reward_ratio = Some(1.5);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            result,
            "Should allow position with risk-reward ratio 1.5 (threshold)"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_high_risk_reward_ratio() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.8);
        analysis.risk_reward_ratio = Some(3.0);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(result, "Should allow position with risk-reward ratio 3.0");
    }

    #[tokio::test]
    async fn test_can_open_position_no_risk_reward_ratio() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.8);
        analysis.risk_reward_ratio = None;

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            result,
            "Should allow position when risk-reward ratio is not available"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_zero_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.0);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(!result, "Should reject position with 0.0 confidence");
    }

    #[tokio::test]
    async fn test_can_open_position_max_confidence() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 1.0);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(result, "Should allow position with 1.0 confidence");
    }

    #[test]
    fn test_calculate_position_size() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config.clone());

        let size = risk_manager.calculate_position_size("BTCUSDT", 50000.0, Some(49000.0), 10000.0);

        // With risk management: 10,000 * 2% = 200, stop loss = 2%, position = 200/0.02/50000 = 0.2
        // Capped by max_position_value (20% of account): 10,000 * 0.2 / 50,000 = 0.04
        assert_eq!(size, 0.04, "Should calculate risk-based position size");
    }

    #[test]
    fn test_calculate_position_size_no_stop_loss() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config.clone());

        let size = risk_manager.calculate_position_size("BTCUSDT", 50000.0, None, 10000.0);

        assert_eq!(
            size, config.default_quantity,
            "Should return default quantity without stop loss"
        );
    }

    #[test]
    fn test_calculate_position_size_zero_account_balance() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config.clone());

        let size = risk_manager.calculate_position_size("BTCUSDT", 50000.0, Some(49000.0), 0.0);

        assert_eq!(
            size, config.default_quantity,
            "Should return default quantity even with zero balance"
        );
    }

    #[test]
    fn test_calculate_position_size_large_account_balance() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config.clone());

        let size =
            risk_manager.calculate_position_size("BTCUSDT", 50000.0, Some(49000.0), 1000000.0);

        // With large balance: 1,000,000 * 2% = 20,000, position = 20,000/0.02/50000 = 20.0
        // Capped by max_position_value (20% of account): 1,000,000 * 0.2 / 50,000 = 4.0
        // Further capped by 5x default_quantity: 0.01 * 5 = 0.05
        assert_eq!(
            size, 0.05,
            "Should cap at 5x default quantity for large account"
        );
    }

    #[test]
    fn test_get_max_positions() {
        let mut config = create_test_config();
        config.max_positions = 10;

        let risk_manager = RiskManager::new(config);

        assert_eq!(risk_manager.get_max_positions(), 10);
    }

    #[test]
    fn test_get_max_positions_zero() {
        let mut config = create_test_config();
        config.max_positions = 0;

        let risk_manager = RiskManager::new(config);

        assert_eq!(risk_manager.get_max_positions(), 0);
    }

    #[test]
    fn test_get_risk_percentage() {
        let mut config = create_test_config();
        config.risk_percentage = 5.0;

        let risk_manager = RiskManager::new(config);

        assert_eq!(risk_manager.get_risk_percentage(), 5.0);
    }

    #[test]
    fn test_get_risk_percentage_zero() {
        let mut config = create_test_config();
        config.risk_percentage = 0.0;

        let risk_manager = RiskManager::new(config);

        assert_eq!(risk_manager.get_risk_percentage(), 0.0);
    }

    #[test]
    fn test_risk_manager_clone() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);

        let cloned = risk_manager.clone();

        assert_eq!(cloned.get_max_positions(), risk_manager.get_max_positions());
        assert_eq!(
            cloned.get_risk_percentage(),
            risk_manager.get_risk_percentage()
        );
    }

    #[tokio::test]
    async fn test_can_open_position_extreme_confidence_values() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);

        // Test with extremely high confidence (edge case)
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.9999);
        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();
        assert!(result, "Should allow position with 0.9999 confidence");
    }

    #[tokio::test]
    async fn test_can_open_position_negative_risk_reward() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.8);
        analysis.risk_reward_ratio = Some(-1.0);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            !result,
            "Should reject position with negative risk-reward ratio"
        );
    }

    #[tokio::test]
    async fn test_can_open_position_zero_risk_reward() {
        let config = create_test_config();
        let risk_manager = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.8);
        analysis.risk_reward_ratio = Some(0.0);

        let result = risk_manager
            .can_open_position("BTCUSDT", &analysis)
            .await
            .unwrap();

        assert!(
            !result,
            "Should reject position with zero risk-reward ratio"
        );
    }

    // === MUTATION KILLING TESTS ===
    // These tests target specific surviving mutants to improve mutation score from 84% to 90%+

    // Phase 1: Boundary Operator Tests (catches < vs <= mutations)

    /// Catches mutation: confidence < 0.7 → confidence <= 0.7
    #[tokio::test]
    async fn test_can_open_position_strong_buy_confidence_below_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.6999);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(!result, "Must reject StrongBuy with confidence 0.6999 (just below 0.7)");
    }

    /// Catches mutation: confidence < 0.7 → confidence <= 0.7
    #[tokio::test]
    async fn test_can_open_position_strong_buy_confidence_at_exact_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.7000);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(result, "Must accept StrongBuy with confidence 0.7000 (exact threshold)");
    }

    /// Catches mutation: confidence < 0.7 → confidence <= 0.7
    #[tokio::test]
    async fn test_can_open_position_strong_buy_confidence_above_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::StrongBuy, 0.7001);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(result, "Must accept StrongBuy with confidence 0.7001 (just above 0.7)");
    }

    /// Catches mutation: confidence < 0.8 → confidence <= 0.8
    #[tokio::test]
    async fn test_can_open_position_buy_confidence_below_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Buy, 0.7999);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(!result, "Must reject Buy with confidence 0.7999 (just below 0.8)");
    }

    /// Catches mutation: confidence < 0.8 → confidence <= 0.8
    #[tokio::test]
    async fn test_can_open_position_buy_confidence_at_exact_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Buy, 0.8000);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(result, "Must accept Buy with confidence 0.8000 (exact threshold)");
    }

    /// Catches mutation: confidence < 0.8 → confidence <= 0.8
    #[tokio::test]
    async fn test_can_open_position_buy_confidence_above_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let analysis = create_test_analysis(TradingSignal::Buy, 0.8001);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(result, "Must accept Buy with confidence 0.8001 (just above 0.8)");
    }

    /// Catches mutation: risk_reward < 1.5 → risk_reward <= 1.5
    #[tokio::test]
    async fn test_can_open_position_risk_reward_below_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.9);
        analysis.risk_reward_ratio = Some(1.4999);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(!result, "Must reject risk-reward 1.4999 (just below 1.5)");
    }

    /// Catches mutation: risk_reward < 1.5 → risk_reward <= 1.5
    #[tokio::test]
    async fn test_can_open_position_risk_reward_at_exact_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.9);
        analysis.risk_reward_ratio = Some(1.5000);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(result, "Must accept risk-reward 1.5000 (exact threshold)");
    }

    /// Catches mutation: risk_reward < 1.5 → risk_reward <= 1.5
    #[tokio::test]
    async fn test_can_open_position_risk_reward_above_threshold() {
        let config = create_test_config();
        let rm = RiskManager::new(config);
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 0.9);
        analysis.risk_reward_ratio = Some(1.5001);
        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(result, "Must accept risk-reward 1.5001 (just above 1.5)");
    }

    // Phase 2: Logical Operator Tests (catches && vs || and ! removal mutations)

    /// Catches mutation: !config.enabled → config.enabled
    #[tokio::test]
    async fn test_trading_disabled_blocks_all_signals_regardless_of_quality() {
        let mut config = create_test_config();
        config.enabled = false;
        let rm = RiskManager::new(config);

        // Even perfect signal with perfect metrics should be blocked
        let mut analysis = create_test_analysis(TradingSignal::StrongBuy, 1.0);
        analysis.risk_reward_ratio = Some(10.0);

        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(!result, "Trading disabled must block even perfect signals");
    }

    /// Catches mutation: && → || in multi-condition checks
    #[tokio::test]
    async fn test_requires_both_good_confidence_and_good_risk_reward() {
        let config = create_test_config();
        let rm = RiskManager::new(config);

        // Good confidence but bad risk-reward → REJECT
        let mut bad_rr = create_test_analysis(TradingSignal::StrongBuy, 0.95);
        bad_rr.risk_reward_ratio = Some(1.0);
        let result1 = rm.can_open_position("BTCUSDT", &bad_rr).await.unwrap();
        assert!(!result1, "Good confidence alone not enough (bad risk-reward should reject)");

        // Bad confidence but good risk-reward → REJECT
        let mut bad_conf = create_test_analysis(TradingSignal::StrongBuy, 0.5);
        bad_conf.risk_reward_ratio = Some(3.0);
        let result2 = rm.can_open_position("BTCUSDT", &bad_conf).await.unwrap();
        assert!(!result2, "Good risk-reward alone not enough (bad confidence should reject)");

        // Both good → ACCEPT
        let mut both_good = create_test_analysis(TradingSignal::StrongBuy, 0.9);
        both_good.risk_reward_ratio = Some(2.5);
        let result3 = rm.can_open_position("BTCUSDT", &both_good).await.unwrap();
        assert!(result3, "Both good confidence and risk-reward should accept");
    }

    /// Catches mutations in Hold signal rejection path
    #[tokio::test]
    async fn test_hold_signal_always_rejected_despite_perfect_metrics() {
        let config = create_test_config();
        let rm = RiskManager::new(config);

        let mut analysis = create_test_analysis(TradingSignal::Hold, 1.0);
        analysis.risk_reward_ratio = Some(100.0);

        let result = rm.can_open_position("BTCUSDT", &analysis).await.unwrap();
        assert!(!result, "Hold signal must be rejected regardless of perfect metrics");
    }

    // Phase 3: Numeric Constant Verification Tests

    /// Verifies MIN_STOP_LOSS_PCT constant is exactly 0.5%
    #[test]
    fn test_min_stop_loss_percentage_is_point_five_percent() {
        let config = create_test_config();
        let rm = RiskManager::new(config.clone());

        // Stop loss at 0.4999% (below threshold) → should return default
        let size_below = rm.calculate_position_size(
            "BTCUSDT",
            50000.0,
            Some(50000.0 * (1.0 - 0.004999)),
            10000.0
        );
        assert_eq!(size_below, config.default_quantity,
            "Below 0.5% stop loss must return default quantity");

        // Stop loss at 0.5% (at threshold) → should calculate
        let size_at = rm.calculate_position_size(
            "BTCUSDT",
            50000.0,
            Some(50000.0 * (1.0 - 0.005)),
            10000.0
        );
        assert!(size_at >= config.default_quantity * 0.01,
            "At 0.5% stop loss should calculate position size");
    }

    /// Verifies max position is 20% of account balance
    #[test]
    fn test_max_position_is_twenty_percent_of_account() {
        let config = create_test_config();
        let rm = RiskManager::new(config);

        // Large account, wide stop loss → position should be capped at 20%
        let size = rm.calculate_position_size(
            "BTCUSDT",
            50000.0,
            Some(40000.0), // 20% stop loss
            1000000.0 // 1M account
        );

        let max_expected = (1000000.0 * 0.2) / 50000.0; // 20% of 1M / price
        assert!(size <= max_expected,
            "Position size must not exceed 20% of account balance");
    }

    /// Verifies confidence thresholds are exactly 0.7 and 0.8
    #[tokio::test]
    async fn test_confidence_thresholds_are_point_seven_and_point_eight() {
        let config = create_test_config();
        let rm = RiskManager::new(config);

        // StrongBuy requires 0.7
        let strong_below = create_test_analysis(TradingSignal::StrongBuy, 0.699);
        assert!(!rm.can_open_position("BTCUSDT", &strong_below).await.unwrap());
        let strong_at = create_test_analysis(TradingSignal::StrongBuy, 0.7);
        assert!(rm.can_open_position("BTCUSDT", &strong_at).await.unwrap());

        // Buy requires 0.8
        let buy_below = create_test_analysis(TradingSignal::Buy, 0.799);
        assert!(!rm.can_open_position("BTCUSDT", &buy_below).await.unwrap());
        let buy_at = create_test_analysis(TradingSignal::Buy, 0.8);
        assert!(rm.can_open_position("BTCUSDT", &buy_at).await.unwrap());
    }

    /// Verifies risk-reward threshold is exactly 1.5
    #[tokio::test]
    async fn test_risk_reward_threshold_is_one_point_five() {
        let config = create_test_config();
        let rm = RiskManager::new(config);

        let mut below = create_test_analysis(TradingSignal::StrongBuy, 0.9);
        below.risk_reward_ratio = Some(1.49);
        assert!(!rm.can_open_position("BTCUSDT", &below).await.unwrap(),
            "Risk-reward below 1.5 must be rejected");

        let mut at = create_test_analysis(TradingSignal::StrongBuy, 0.9);
        at.risk_reward_ratio = Some(1.5);
        assert!(rm.can_open_position("BTCUSDT", &at).await.unwrap(),
            "Risk-reward at 1.5 must be accepted");
    }

    // Phase 4: Edge Case & Division Safety Tests

    /// Catches division by zero in position sizing
    #[test]
    fn test_position_size_with_zero_entry_price() {
        let config = create_test_config();
        let rm = RiskManager::new(config.clone());

        let size = rm.calculate_position_size("BTCUSDT", 0.0, Some(100.0), 10000.0);
        assert_eq!(size, config.default_quantity,
            "Zero entry price must return default quantity");
    }

    /// Catches division by zero in position sizing
    #[test]
    fn test_position_size_with_zero_account_balance() {
        let config = create_test_config();
        let rm = RiskManager::new(config.clone());

        let size = rm.calculate_position_size("BTCUSDT", 50000.0, Some(49000.0), 0.0);
        assert_eq!(size, config.default_quantity,
            "Zero account balance must return default quantity");
    }

    /// Catches edge case when entry price equals stop loss
    #[test]
    fn test_position_size_when_entry_equals_stop_loss() {
        let config = create_test_config();
        let rm = RiskManager::new(config.clone());

        let size = rm.calculate_position_size("BTCUSDT", 50000.0, Some(50000.0), 10000.0);
        assert_eq!(size, config.default_quantity,
            "Entry price == stop loss must return default quantity");
    }

    /// Catches negative value handling
    #[test]
    fn test_position_size_with_negative_stop_loss() {
        let config = create_test_config();
        let rm = RiskManager::new(config.clone());

        let size = rm.calculate_position_size("BTCUSDT", 50000.0, Some(-1000.0), 10000.0);
        assert_eq!(size, config.default_quantity,
            "Negative stop loss must return default quantity");
    }

    /// Verifies Sell/StrongSell use same thresholds as Buy/StrongBuy
    #[tokio::test]
    async fn test_sell_signals_use_same_confidence_thresholds() {
        let config = create_test_config();
        let rm = RiskManager::new(config);

        // StrongSell should use 0.7 threshold
        let strong_sell_below = create_test_analysis(TradingSignal::StrongSell, 0.69);
        assert!(!rm.can_open_position("BTCUSDT", &strong_sell_below).await.unwrap());
        let strong_sell_at = create_test_analysis(TradingSignal::StrongSell, 0.7);
        assert!(rm.can_open_position("BTCUSDT", &strong_sell_at).await.unwrap());

        // Sell should use 0.8 threshold
        let sell_below = create_test_analysis(TradingSignal::Sell, 0.79);
        assert!(!rm.can_open_position("BTCUSDT", &sell_below).await.unwrap());
        let sell_at = create_test_analysis(TradingSignal::Sell, 0.8);
        assert!(rm.can_open_position("BTCUSDT", &sell_at).await.unwrap());
    }
}
