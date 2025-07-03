use anyhow::Result;
use tracing::{debug, warn};

use crate::config::TradingConfig;
use crate::market_data::analyzer::MultiTimeframeAnalysis;

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
            crate::market_data::analyzer::TradingSignal::StrongBuy | 
            crate::market_data::analyzer::TradingSignal::StrongSell => 0.7,
            crate::market_data::analyzer::TradingSignal::Buy | 
            crate::market_data::analyzer::TradingSignal::Sell => 0.8,
            crate::market_data::analyzer::TradingSignal::Hold => return Ok(false),
        };

        if analysis.overall_confidence < min_confidence {
            debug!("Signal confidence {} below threshold {} for {}", 
                   analysis.overall_confidence, min_confidence, symbol);
            return Ok(false);
        }

        // Check risk-reward ratio if available
        if let Some(risk_reward) = analysis.risk_reward_ratio {
            if risk_reward < 1.5 {
                debug!("Risk-reward ratio {} below minimum 1.5 for {}", risk_reward, symbol);
                return Ok(false);
            }
        }

        debug!("Risk check passed for {} with confidence {:.2}", 
               symbol, analysis.overall_confidence);
        Ok(true)
    }

    pub fn calculate_position_size(
        &self,
        _symbol: &str,
        _entry_price: f64,
        _stop_loss: Option<f64>,
        _account_balance: f64,
    ) -> f64 {
        // Simple fixed size for now
        self.config.default_quantity
    }

    pub fn get_max_positions(&self) -> u32 {
        self.config.max_positions
    }

    pub fn get_risk_percentage(&self) -> f64 {
        self.config.risk_percentage
    }
} 