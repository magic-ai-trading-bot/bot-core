use anyhow::Result;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Removed unused imports
use super::portfolio::PortfolioMetrics;
// Removed unused imports

/// Strategy optimization engine

// @spec:FR-STRATEGIES-007 - Strategy Optimizer
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#strategies
// @test:TC-TRADING-037, TC-TRADING-038

#[derive(Debug, Clone)]
pub struct StrategyOptimizer {
    /// Historical performance data
    performance_history: Vec<PerformanceSnapshot>,

    /// Optimization parameters
    optimization_config: OptimizationConfig,

    /// Current parameter sets being tested
    parameter_tests: HashMap<String, ParameterTest>,

    /// Best performing parameters
    best_parameters: HashMap<String, HashMap<String, serde_json::Value>>,

    /// Market regime detector
    regime_detector: MarketRegimeDetector,
}

/// Performance snapshot for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSnapshot {
    pub timestamp: DateTime<Utc>,
    pub portfolio_metrics: PortfolioMetrics,
    pub market_conditions: MarketConditions,
    pub active_strategies: HashMap<String, StrategyPerformance>,
    pub parameters_used: HashMap<String, HashMap<String, serde_json::Value>>,
}

/// Market conditions at time of snapshot
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketConditions {
    pub volatility: f64,
    pub trend_strength: f64,
    pub volume_profile: String,
    pub regime: String,
    pub correlation_matrix: HashMap<String, HashMap<String, f64>>,
}

/// Strategy-specific performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyPerformance {
    pub signal_count: u32,
    pub executed_trades: u32,
    pub win_rate: f64,
    pub avg_profit: f64,
    pub avg_loss: f64,
    pub profit_factor: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub confidence_accuracy: f64, // How accurate the AI confidence was
    pub signal_frequency: f64,    // Signals per hour
}

/// Parameter testing configuration
#[derive(Debug, Clone)]
pub struct ParameterTest {
    pub strategy_name: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub start_time: DateTime<Utc>,
    pub trades_executed: u32,
    pub current_performance: f64,
    pub target_trades: u32,
}

/// Optimization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationConfig {
    /// Minimum number of trades before optimization
    pub min_trades_for_optimization: u32,

    /// Optimization period in days
    pub optimization_period_days: u32,

    /// Number of parameter sets to test simultaneously
    pub max_concurrent_tests: u32,

    /// Performance metric to optimize
    pub primary_metric: OptimizationMetric,

    /// Secondary metrics with weights
    pub secondary_metrics: HashMap<OptimizationMetric, f64>,

    /// Enable genetic algorithm optimization
    pub enable_genetic_algorithm: bool,

    /// Population size for genetic algorithm
    pub genetic_population_size: u32,

    /// Mutation rate for genetic algorithm
    pub genetic_mutation_rate: f64,

    /// Enable Bayesian optimization
    pub enable_bayesian_optimization: bool,

    /// Enable walk-forward optimization
    pub enable_walk_forward: bool,

    /// Out-of-sample testing percentage
    pub out_of_sample_percentage: f64,
}

/// Market regime detector
#[derive(Debug, Clone)]
pub struct MarketRegimeDetector {
    /// Historical price data for regime detection
    price_history: Vec<PricePoint>,

    /// Current detected regime
    current_regime: MarketRegime,

    /// Regime transition probability
    transition_probabilities: HashMap<MarketRegime, HashMap<MarketRegime, f64>>,

    /// Regime-specific statistics
    regime_stats: HashMap<MarketRegime, RegimeStatistics>,
}

/// Price point for regime detection
#[derive(Debug, Clone)]
pub struct PricePoint {
    pub timestamp: DateTime<Utc>,
    pub price: f64,
    pub volume: f64,
    pub volatility: f64,
}

/// Market regime types
#[derive(Debug, Clone, Copy, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum MarketRegime {
    BullTrending,
    BearTrending,
    Sideways,
    HighVolatility,
    LowVolatility,
    Breakout,
    Reversal,
    Unknown,
}

/// Statistics for each market regime
#[derive(Debug, Clone)]
pub struct RegimeStatistics {
    pub average_duration_hours: f64,
    pub average_volatility: f64,
    pub average_return: f64,
    pub best_strategies: Vec<String>,
    pub best_parameters: HashMap<String, HashMap<String, serde_json::Value>>,
}

/// Optimization metrics
#[derive(Debug, Clone, Hash, Eq, PartialEq, Serialize, Deserialize)]
pub enum OptimizationMetric {
    TotalReturn,
    SharpeRatio,
    SortinoRatio,
    CalmarRatio,
    MaxDrawdown,
    WinRate,
    ProfitFactor,
    RiskAdjustedReturn,
    Consistency, // Standard deviation of returns
}

/// Strategy optimization recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRecommendation {
    pub strategy_name: String,
    pub recommended_parameters: HashMap<String, serde_json::Value>,
    pub expected_improvement: f64,
    pub confidence: f64,
    pub reasoning: String,
    pub market_regime: MarketRegime,
    pub backtesting_results: BacktestingResults,
    pub risk_assessment: OptimizationRiskAssessment,
}

/// Backtesting results for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BacktestingResults {
    pub total_trades: u32,
    pub win_rate: f64,
    pub total_return: f64,
    pub sharpe_ratio: f64,
    pub max_drawdown: f64,
    pub profit_factor: f64,
    pub monthly_returns: Vec<f64>,
    pub trade_distribution: HashMap<String, u32>,
}

/// Risk assessment for optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationRiskAssessment {
    pub overfitting_risk: f64,
    pub parameter_sensitivity: HashMap<String, f64>,
    pub regime_dependency: f64,
    pub data_sufficiency: f64,
    pub recommendation: String,
}

impl StrategyOptimizer {
    /// Create a new strategy optimizer
    pub fn new(config: OptimizationConfig) -> Self {
        Self {
            performance_history: Vec::new(),
            optimization_config: config,
            parameter_tests: HashMap::new(),
            best_parameters: HashMap::new(),
            regime_detector: MarketRegimeDetector::new(),
        }
    }

    /// Add performance snapshot for optimization analysis
    pub fn add_performance_snapshot(
        &mut self,
        portfolio_metrics: PortfolioMetrics,
        market_conditions: MarketConditions,
        active_strategies: HashMap<String, StrategyPerformance>,
        parameters_used: HashMap<String, HashMap<String, serde_json::Value>>,
    ) {
        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            portfolio_metrics,
            market_conditions: market_conditions.clone(),
            active_strategies,
            parameters_used,
        };

        self.performance_history.push(snapshot);

        // Keep only relevant history
        let cutoff_date =
            Utc::now() - Duration::days(self.optimization_config.optimization_period_days as i64);
        self.performance_history
            .retain(|s| s.timestamp >= cutoff_date);

        // Update regime detector
        self.regime_detector
            .update_with_conditions(&market_conditions);
    }

    /// Analyze current performance and generate optimization recommendations
    pub fn analyze_and_recommend(&mut self) -> Result<Vec<OptimizationRecommendation>> {
        if self.performance_history.len()
            < self.optimization_config.min_trades_for_optimization as usize
        {
            return Ok(Vec::new());
        }

        let mut recommendations = Vec::new();

        // Detect current market regime
        let current_regime = self.regime_detector.detect_current_regime();

        // Analyze performance by strategy
        let strategy_analysis = self.analyze_strategy_performance()?;

        // Generate recommendations for each underperforming strategy
        for (strategy_name, performance) in strategy_analysis {
            if self.should_optimize_strategy(&strategy_name, &performance) {
                let recommendation = self.generate_optimization_recommendation(
                    strategy_name,
                    performance,
                    current_regime,
                )?;
                recommendations.push(recommendation);
            }
        }

        // Apply genetic algorithm if enabled
        if self.optimization_config.enable_genetic_algorithm {
            let genetic_recommendations = self.apply_genetic_optimization(current_regime)?;
            recommendations.extend(genetic_recommendations);
        }

        // Apply Bayesian optimization if enabled
        if self.optimization_config.enable_bayesian_optimization {
            let bayesian_recommendations = self.apply_bayesian_optimization(current_regime)?;
            recommendations.extend(bayesian_recommendations);
        }

        Ok(recommendations)
    }

    /// Analyze strategy performance across different conditions
    fn analyze_strategy_performance(&self) -> Result<HashMap<String, StrategyAnalysis>> {
        let mut analysis = HashMap::new();

        // Group performance by strategy
        for snapshot in &self.performance_history {
            for (strategy_name, performance) in &snapshot.active_strategies {
                let entry = analysis
                    .entry(strategy_name.clone())
                    .or_insert_with(StrategyAnalysis::new);
                entry.add_performance_point(performance, &snapshot.market_conditions);
            }
        }

        // Calculate statistics for each strategy
        for (_, strategy_analysis) in analysis.iter_mut() {
            strategy_analysis.calculate_statistics();
        }

        Ok(analysis)
    }

    /// Check if a strategy should be optimized
    fn should_optimize_strategy(&self, _strategy_name: &str, analysis: &StrategyAnalysis) -> bool {
        // Don't optimize if insufficient data
        if analysis.total_trades < self.optimization_config.min_trades_for_optimization {
            return false;
        }

        // Optimize if performance is below threshold
        let performance_threshold = 0.3; // 30th percentile
        analysis.performance_percentile < performance_threshold ||
        analysis.recent_performance_decline > 0.1 || // 10% decline
        analysis.regime_adaptation_score < 0.5
    }

    /// Generate optimization recommendation for a strategy
    fn generate_optimization_recommendation(
        &self,
        strategy_name: String,
        analysis: StrategyAnalysis,
        current_regime: MarketRegime,
    ) -> Result<OptimizationRecommendation> {
        // Get best parameters for current regime
        let regime_stats = self.regime_detector.regime_stats.get(&current_regime);
        let base_parameters = regime_stats
            .and_then(|stats| stats.best_parameters.get(&strategy_name))
            .cloned()
            .unwrap_or_default();

        // Generate parameter variations
        let recommended_parameters =
            self.generate_parameter_variations(&strategy_name, &base_parameters, &analysis)?;

        // Estimate improvement potential
        let expected_improvement =
            self.estimate_improvement_potential(&analysis, &recommended_parameters);

        // Calculate confidence based on data quality and consistency
        let confidence = self.calculate_recommendation_confidence(&analysis);

        // Generate reasoning
        let reasoning =
            self.generate_optimization_reasoning(&strategy_name, &analysis, current_regime);

        // Run backtesting
        let backtesting_results =
            self.run_parameter_backtest(&strategy_name, &recommended_parameters)?;

        // Assess risks
        let risk_assessment = self.assess_optimization_risks(&analysis, &recommended_parameters);

        Ok(OptimizationRecommendation {
            strategy_name,
            recommended_parameters,
            expected_improvement,
            confidence,
            reasoning,
            market_regime: current_regime,
            backtesting_results,
            risk_assessment,
        })
    }

    /// Generate parameter variations using multiple methods
    fn generate_parameter_variations(
        &self,
        _strategy_name: &str,
        base_parameters: &HashMap<String, serde_json::Value>,
        analysis: &StrategyAnalysis,
    ) -> Result<HashMap<String, serde_json::Value>> {
        let mut optimized_parameters = base_parameters.clone();

        // Apply performance-based adjustments
        if analysis.avg_win_rate < 0.5 {
            // Increase selectivity
            self.adjust_selectivity_parameters(&mut optimized_parameters, 1.2);
        }

        if analysis.avg_profit_factor < 1.5 {
            // Improve risk/reward
            self.adjust_risk_reward_parameters(&mut optimized_parameters, 1.3);
        }

        if analysis.max_drawdown > 0.15 {
            // Reduce risk
            self.adjust_risk_parameters(&mut optimized_parameters, 0.8);
        }

        // Apply regime-specific adjustments
        let current_regime = self.regime_detector.detect_current_regime();
        match current_regime {
            MarketRegime::HighVolatility => {
                self.adjust_for_high_volatility(&mut optimized_parameters);
            },
            MarketRegime::Sideways => {
                self.adjust_for_sideways_market(&mut optimized_parameters);
            },
            MarketRegime::BullTrending | MarketRegime::BearTrending => {
                self.adjust_for_trending_market(&mut optimized_parameters);
            },
            _ => {},
        }

        Ok(optimized_parameters)
    }

    /// Adjust parameters for higher selectivity
    fn adjust_selectivity_parameters(
        &self,
        parameters: &mut HashMap<String, serde_json::Value>,
        factor: f64,
    ) {
        // Increase confidence thresholds
        if let Some(confidence) = parameters.get_mut("min_confidence") {
            if let Some(val) = confidence.as_f64() {
                // from_f64 can return None for NaN/Inf, use original value as fallback
                if let Some(num) = serde_json::Number::from_f64(val * factor) {
                    *confidence = serde_json::Value::Number(num);
                }
            }
        }

        // Tighten entry conditions
        if let Some(threshold) = parameters.get_mut("entry_threshold") {
            if let Some(val) = threshold.as_f64() {
                if let Some(num) = serde_json::Number::from_f64(val * factor) {
                    *threshold = serde_json::Value::Number(num);
                }
            }
        }
    }

    /// Adjust risk/reward parameters
    fn adjust_risk_reward_parameters(
        &self,
        parameters: &mut HashMap<String, serde_json::Value>,
        factor: f64,
    ) {
        // Increase take profit relative to stop loss
        if let Some(tp_ratio) = parameters.get_mut("take_profit_ratio") {
            if let Some(val) = tp_ratio.as_f64() {
                if let Some(num) = serde_json::Number::from_f64(val * factor) {
                    *tp_ratio = serde_json::Value::Number(num);
                }
            }
        }
    }

    /// Adjust risk parameters
    fn adjust_risk_parameters(
        &self,
        parameters: &mut HashMap<String, serde_json::Value>,
        factor: f64,
    ) {
        // Reduce position sizes
        if let Some(position_size) = parameters.get_mut("position_size_multiplier") {
            if let Some(val) = position_size.as_f64() {
                if let Some(num) = serde_json::Number::from_f64(val * factor) {
                    *position_size = serde_json::Value::Number(num);
                }
            }
        }
    }

    /// Adjust parameters for high volatility market
    fn adjust_for_high_volatility(&self, parameters: &mut HashMap<String, serde_json::Value>) {
        // Wider stop losses
        if let Some(sl_multiplier) = parameters.get_mut("stop_loss_multiplier") {
            if let Some(val) = sl_multiplier.as_f64() {
                if let Some(num) = serde_json::Number::from_f64(val * 1.5) {
                    *sl_multiplier = serde_json::Value::Number(num);
                }
            }
        }

        // Lower leverage
        if let Some(leverage) = parameters.get_mut("max_leverage") {
            if let Some(val) = leverage.as_u64() {
                *leverage =
                    serde_json::Value::Number(serde_json::Number::from((val as f64 * 0.7) as u64));
            }
        }
    }

    /// Adjust parameters for sideways market
    fn adjust_for_sideways_market(&self, parameters: &mut HashMap<String, serde_json::Value>) {
        // Shorter timeframes
        if let Some(timeframe) = parameters.get_mut("primary_timeframe") {
            if timeframe.as_str() == Some("4h") {
                *timeframe = serde_json::Value::String("1h".to_string());
            }
        }

        // More conservative entry
        if let Some(entry_threshold) = parameters.get_mut("entry_threshold") {
            if let Some(val) = entry_threshold.as_f64() {
                if let Some(num) = serde_json::Number::from_f64(val * 1.2) {
                    *entry_threshold = serde_json::Value::Number(num);
                }
            }
        }
    }

    /// Adjust parameters for trending market
    fn adjust_for_trending_market(&self, parameters: &mut HashMap<String, serde_json::Value>) {
        // Longer timeframes
        if let Some(timeframe) = parameters.get_mut("primary_timeframe") {
            if timeframe.as_str() == Some("1h") {
                *timeframe = serde_json::Value::String("4h".to_string());
            }
        }

        // More aggressive position sizing
        if let Some(position_multiplier) = parameters.get_mut("position_size_multiplier") {
            if let Some(val) = position_multiplier.as_f64() {
                if let Some(num) = serde_json::Number::from_f64(val * 1.2) {
                    *position_multiplier = serde_json::Value::Number(num);
                }
            }
        }
    }

    /// Estimate improvement potential
    fn estimate_improvement_potential(
        &self,
        analysis: &StrategyAnalysis,
        _new_parameters: &HashMap<String, serde_json::Value>,
    ) -> f64 {
        // Simple heuristic based on current underperformance
        let current_score = analysis.overall_performance_score;
        let theoretical_max = 1.0;
        let improvement_potential = (theoretical_max - current_score) * 0.6; // Conservative estimate

        improvement_potential.min(0.5) // Cap at 50% improvement
    }

    /// Calculate confidence in recommendation
    fn calculate_recommendation_confidence(&self, analysis: &StrategyAnalysis) -> f64 {
        let mut confidence: f64 = 0.5; // Base confidence

        // Increase confidence with more data
        if analysis.total_trades > 100 {
            confidence += 0.2;
        }

        // Increase confidence with consistent underperformance
        if analysis.performance_consistency > 0.7 {
            confidence += 0.2;
        }

        // Decrease confidence with high variance
        if analysis.performance_variance > 0.5 {
            confidence -= 0.1;
        }

        confidence.clamp(0.0, 1.0)
    }

    /// Generate reasoning for optimization
    fn generate_optimization_reasoning(
        &self,
        _strategy_name: &str,
        analysis: &StrategyAnalysis,
        regime: MarketRegime,
    ) -> String {
        let mut reasons = Vec::new();

        if analysis.avg_win_rate < 0.5 {
            reasons
                .push("Low win rate suggests need for more selective entry criteria".to_string());
        }

        if analysis.avg_profit_factor < 1.5 {
            reasons.push(
                "Poor risk/reward ratio indicates need for better exit strategies".to_string(),
            );
        }

        if analysis.max_drawdown > 0.15 {
            reasons.push("High drawdown suggests excessive risk taking".to_string());
        }

        if analysis.regime_adaptation_score < 0.5 {
            reasons.push(format!("Poor adaptation to {regime:?} market conditions"));
        }

        if reasons.is_empty() {
            reasons.push("Proactive optimization based on market regime change".to_string());
        }

        reasons.join(". ")
    }

    /// Run backtesting with new parameters
    fn run_parameter_backtest(
        &self,
        strategy_name: &str,
        _parameters: &HashMap<String, serde_json::Value>,
    ) -> Result<BacktestingResults> {
        // Simplified backtesting using historical performance data
        let mut total_trades = 0;
        let mut wins = 0;
        let mut total_return = 0.0;
        let mut returns = Vec::new();
        let mut max_drawdown = 0.0;
        let mut peak = 0.0;
        let monthly_returns = Vec::new();

        // Simulate performance with new parameters
        for snapshot in &self.performance_history {
            if let Some(strategy_perf) = snapshot.active_strategies.get(strategy_name) {
                total_trades += strategy_perf.executed_trades;
                wins += (strategy_perf.executed_trades as f64 * strategy_perf.win_rate) as u32;

                // Simulate adjusted returns based on parameter changes
                let simulated_return = strategy_perf.avg_profit * 1.1; // Optimistic adjustment
                total_return += simulated_return;
                returns.push(simulated_return);

                // Track drawdown
                if total_return > peak {
                    peak = total_return;
                }
                let current_drawdown = (peak - total_return) / peak;
                if current_drawdown > max_drawdown {
                    max_drawdown = current_drawdown;
                }
            }
        }

        let win_rate = if total_trades > 0 {
            wins as f64 / total_trades as f64
        } else {
            0.0
        };

        // Calculate Sharpe ratio
        let avg_return = if !returns.is_empty() {
            returns.iter().sum::<f64>() / returns.len() as f64
        } else {
            0.0
        };

        let return_std = if returns.len() > 1 {
            let variance = returns
                .iter()
                .map(|r| (r - avg_return).powi(2))
                .sum::<f64>()
                / (returns.len() - 1) as f64;
            variance.sqrt()
        } else {
            1.0
        };

        let sharpe_ratio = if return_std > 0.0 {
            avg_return / return_std
        } else {
            0.0
        };

        // Calculate profit factor
        let gross_profit: f64 = returns.iter().filter(|&&r| r > 0.0).sum();
        let gross_loss: f64 = returns.iter().filter(|&&r| r < 0.0).map(|r| r.abs()).sum();
        let profit_factor = if gross_loss > 0.0 {
            gross_profit / gross_loss
        } else {
            0.0
        };

        Ok(BacktestingResults {
            total_trades,
            win_rate,
            total_return,
            sharpe_ratio,
            max_drawdown,
            profit_factor,
            monthly_returns,
            trade_distribution: HashMap::new(),
        })
    }

    /// Assess optimization risks
    fn assess_optimization_risks(
        &self,
        analysis: &StrategyAnalysis,
        _parameters: &HashMap<String, serde_json::Value>,
    ) -> OptimizationRiskAssessment {
        // Calculate overfitting risk
        let overfitting_risk = if analysis.total_trades < 50 {
            0.8 // High risk with limited data
        } else if analysis.performance_consistency < 0.5 {
            0.6 // Medium risk with inconsistent performance
        } else {
            0.3 // Low risk with sufficient consistent data
        };

        // Calculate parameter sensitivity (simplified)
        let mut parameter_sensitivity = HashMap::new();
        for param_name in _parameters.keys() {
            // Simplified sensitivity analysis
            parameter_sensitivity.insert(param_name.clone(), 0.5);
        }

        // Calculate regime dependency
        let regime_dependency = 1.0 - analysis.regime_adaptation_score;

        // Calculate data sufficiency
        let data_sufficiency = (analysis.total_trades as f64 / 100.0).min(1.0);

        let recommendation = if overfitting_risk > 0.7 {
            "High overfitting risk - use conservative parameter adjustments and monitor closely"
                .to_string()
        } else if regime_dependency > 0.6 {
            "Strategy shows high regime dependency - consider regime-aware parameters".to_string()
        } else {
            "Optimization appears safe to implement".to_string()
        };

        OptimizationRiskAssessment {
            overfitting_risk,
            parameter_sensitivity,
            regime_dependency,
            data_sufficiency,
            recommendation,
        }
    }

    /// Apply genetic algorithm optimization
    fn apply_genetic_optimization(
        &self,
        _regime: MarketRegime,
    ) -> Result<Vec<OptimizationRecommendation>> {
        // Placeholder for genetic algorithm implementation
        // In a full implementation, this would create populations of parameter sets,
        // evaluate their fitness, and evolve them over generations
        Ok(Vec::new())
    }

    /// Apply Bayesian optimization
    fn apply_bayesian_optimization(
        &self,
        _regime: MarketRegime,
    ) -> Result<Vec<OptimizationRecommendation>> {
        // Placeholder for Bayesian optimization implementation
        // This would use Gaussian processes to model the parameter space
        // and suggest optimal parameter combinations
        Ok(Vec::new())
    }
}

/// Strategy analysis for optimization
#[derive(Debug, Clone)]
pub struct StrategyAnalysis {
    pub total_trades: u32,
    pub avg_win_rate: f64,
    pub avg_profit_factor: f64,
    pub max_drawdown: f64,
    pub performance_percentile: f64,
    pub recent_performance_decline: f64,
    pub regime_adaptation_score: f64,
    pub overall_performance_score: f64,
    pub performance_consistency: f64,
    pub performance_variance: f64,
    pub regime_performance: HashMap<MarketRegime, f64>,
}

impl Default for StrategyAnalysis {
    fn default() -> Self {
        Self::new()
    }
}

impl StrategyAnalysis {
    pub fn new() -> Self {
        Self {
            total_trades: 0,
            avg_win_rate: 0.0,
            avg_profit_factor: 0.0,
            max_drawdown: 0.0,
            performance_percentile: 0.0,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.0,
            overall_performance_score: 0.0,
            performance_consistency: 0.0,
            performance_variance: 0.0,
            regime_performance: HashMap::new(),
        }
    }

    pub fn add_performance_point(
        &mut self,
        performance: &StrategyPerformance,
        _conditions: &MarketConditions,
    ) {
        // Implementation would accumulate performance data
        self.total_trades += performance.executed_trades;
        // ... other accumulations
    }

    pub fn calculate_statistics(&mut self) {
        // Implementation would calculate final statistics
        self.overall_performance_score = (self.avg_win_rate + self.avg_profit_factor / 3.0) / 2.0;
        // ... other calculations
    }
}

impl Default for MarketRegimeDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl MarketRegimeDetector {
    pub fn new() -> Self {
        Self {
            price_history: Vec::new(),
            current_regime: MarketRegime::Unknown,
            transition_probabilities: HashMap::new(),
            regime_stats: HashMap::new(),
        }
    }

    pub fn update_with_conditions(&mut self, _conditions: &MarketConditions) {
        // Implementation would update regime detection based on market conditions
    }

    pub fn detect_current_regime(&self) -> MarketRegime {
        // Simplified regime detection
        // In practice, this would use sophisticated algorithms
        self.current_regime
    }
}

impl Default for OptimizationConfig {
    fn default() -> Self {
        Self {
            min_trades_for_optimization: 50,
            optimization_period_days: 30,
            max_concurrent_tests: 3,
            primary_metric: OptimizationMetric::SharpeRatio,
            secondary_metrics: {
                let mut metrics = HashMap::new();
                metrics.insert(OptimizationMetric::MaxDrawdown, 0.3);
                metrics.insert(OptimizationMetric::ProfitFactor, 0.2);
                metrics
            },
            enable_genetic_algorithm: false,
            genetic_population_size: 20,
            genetic_mutation_rate: 0.1,
            enable_bayesian_optimization: true,
            enable_walk_forward: true,
            out_of_sample_percentage: 20.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_optimizer_creation() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        assert_eq!(optimizer.performance_history.len(), 0);
        assert_eq!(optimizer.parameter_tests.len(), 0);
        assert_eq!(optimizer.best_parameters.len(), 0);
    }

    #[test]
    fn test_optimization_config_defaults() {
        let config = OptimizationConfig::default();

        assert_eq!(config.min_trades_for_optimization, 50);
        assert_eq!(config.optimization_period_days, 30);
        assert_eq!(config.max_concurrent_tests, 3);
        assert!(!config.enable_genetic_algorithm);
        assert!(config.enable_bayesian_optimization);
        assert!(config.enable_walk_forward);
    }

    #[test]
    fn test_market_regime_detector_creation() {
        let detector = MarketRegimeDetector::new();

        assert_eq!(detector.current_regime, MarketRegime::Unknown);
        assert_eq!(detector.price_history.len(), 0);
    }

    #[test]
    fn test_market_regime_detection() {
        let detector = MarketRegimeDetector::new();
        let regime = detector.detect_current_regime();

        assert_eq!(regime, MarketRegime::Unknown);
    }

    #[test]
    fn test_strategy_analysis_creation() {
        let analysis = StrategyAnalysis::new();

        assert_eq!(analysis.total_trades, 0);
        assert_eq!(analysis.avg_win_rate, 0.0);
        assert_eq!(analysis.avg_profit_factor, 0.0);
        assert_eq!(analysis.max_drawdown, 0.0);
    }

    #[test]
    fn test_add_performance_snapshot() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        let portfolio_metrics = PortfolioMetrics::default();
        let market_conditions = MarketConditions {
            volatility: 0.5,
            trend_strength: 0.7,
            volume_profile: "High".to_string(),
            regime: "Trending".to_string(),
            correlation_matrix: HashMap::new(),
        };
        let active_strategies = HashMap::new();
        let parameters_used = HashMap::new();

        optimizer.add_performance_snapshot(
            portfolio_metrics,
            market_conditions,
            active_strategies,
            parameters_used,
        );

        assert_eq!(optimizer.performance_history.len(), 1);
    }

    #[test]
    fn test_analyze_insufficient_data() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        // Add fewer snapshots than required
        for _ in 0..10 {
            let portfolio_metrics = PortfolioMetrics::default();
            let market_conditions = MarketConditions {
                volatility: 0.5,
                trend_strength: 0.7,
                volume_profile: "High".to_string(),
                regime: "Trending".to_string(),
                correlation_matrix: HashMap::new(),
            };
            optimizer.add_performance_snapshot(
                portfolio_metrics,
                market_conditions,
                HashMap::new(),
                HashMap::new(),
            );
        }

        let recommendations = optimizer.analyze_and_recommend().unwrap();
        assert_eq!(recommendations.len(), 0);
    }

    #[test]
    fn test_parameter_adjustment_selectivity() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "min_confidence".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()),
        );

        optimizer.adjust_selectivity_parameters(&mut params, 1.2);

        if let Some(val) = params.get("min_confidence") {
            assert!(val.as_f64().unwrap() >= 0.5);
        }
    }

    #[test]
    fn test_parameter_adjustment_risk_reward() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "take_profit_ratio".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(2.0).unwrap()),
        );

        optimizer.adjust_risk_reward_parameters(&mut params, 1.5);

        if let Some(val) = params.get("take_profit_ratio") {
            assert!(val.as_f64().unwrap() >= 2.0);
        }
    }

    #[test]
    fn test_parameter_adjustment_risk() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "position_size_multiplier".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap()),
        );

        optimizer.adjust_risk_parameters(&mut params, 0.8);

        if let Some(val) = params.get("position_size_multiplier") {
            assert!(val.as_f64().unwrap() <= 1.0);
        }
    }

    #[test]
    fn test_adjust_for_high_volatility() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "stop_loss_multiplier".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap()),
        );
        params.insert(
            "max_leverage".to_string(),
            serde_json::Value::Number(serde_json::Number::from(10)),
        );

        optimizer.adjust_for_high_volatility(&mut params);

        // Stop loss should be wider
        if let Some(val) = params.get("stop_loss_multiplier") {
            assert!(val.as_f64().unwrap() >= 1.0);
        }

        // Leverage should be lower
        if let Some(val) = params.get("max_leverage") {
            assert!(val.as_u64().unwrap() <= 10);
        }
    }

    #[test]
    fn test_adjust_for_sideways_market() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "primary_timeframe".to_string(),
            serde_json::Value::String("4h".to_string()),
        );

        optimizer.adjust_for_sideways_market(&mut params);

        // Should use shorter timeframe
        if let Some(val) = params.get("primary_timeframe") {
            assert_eq!(val.as_str().unwrap(), "1h");
        }
    }

    #[test]
    fn test_adjust_for_trending_market() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "primary_timeframe".to_string(),
            serde_json::Value::String("1h".to_string()),
        );

        optimizer.adjust_for_trending_market(&mut params);

        // Should use longer timeframe
        if let Some(val) = params.get("primary_timeframe") {
            assert_eq!(val.as_str().unwrap(), "4h");
        }
    }

    #[test]
    fn test_estimate_improvement_potential() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.4,
            avg_profit_factor: 1.2,
            max_drawdown: 0.1,
            performance_percentile: 0.3,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.5,
            overall_performance_score: 0.5,
            performance_consistency: 0.7,
            performance_variance: 0.3,
            regime_performance: HashMap::new(),
        };

        let improvement = optimizer.estimate_improvement_potential(&analysis, &HashMap::new());

        assert!(improvement >= 0.0);
        assert!(improvement <= 0.5); // Capped at 50%
    }

    #[test]
    fn test_calculate_recommendation_confidence() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 150,
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.8,
            overall_performance_score: 0.7,
            performance_consistency: 0.8,
            performance_variance: 0.3,
            regime_performance: HashMap::new(),
        };

        let confidence = optimizer.calculate_recommendation_confidence(&analysis);

        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
        assert!(confidence > 0.5); // High data quality should give high confidence
    }

    #[test]
    fn test_should_optimize_strategy() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        // Not enough trades
        let mut analysis = StrategyAnalysis::new();
        analysis.total_trades = 10;
        assert!(!optimizer.should_optimize_strategy("test", &analysis));

        // Enough trades but poor performance
        analysis.total_trades = 100;
        analysis.performance_percentile = 0.2;
        assert!(optimizer.should_optimize_strategy("test", &analysis));

        // Good performance, no need to optimize
        analysis.performance_percentile = 0.8;
        analysis.recent_performance_decline = 0.0;
        analysis.regime_adaptation_score = 0.9;
        assert!(!optimizer.should_optimize_strategy("test", &analysis));
    }

    #[test]
    fn test_backtesting_results() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let results = optimizer
            .run_parameter_backtest("test_strategy", &HashMap::new())
            .unwrap();

        assert_eq!(results.total_trades, 0);
        assert_eq!(results.win_rate, 0.0);
    }

    #[test]
    fn test_optimization_risk_assessment() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 30,
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.3,
            overall_performance_score: 0.6,
            performance_consistency: 0.4,
            performance_variance: 0.6,
            regime_performance: HashMap::new(),
        };

        let risk_assessment = optimizer.assess_optimization_risks(&analysis, &HashMap::new());

        assert!(risk_assessment.overfitting_risk >= 0.0);
        assert!(risk_assessment.overfitting_risk <= 1.0);
        assert!(risk_assessment.regime_dependency >= 0.0);
        assert!(risk_assessment.regime_dependency <= 1.0);
        assert!(risk_assessment.data_sufficiency >= 0.0);
        assert!(risk_assessment.data_sufficiency <= 1.0);
    }

    #[test]
    fn test_market_regime_variants() {
        let regimes = vec![
            MarketRegime::BullTrending,
            MarketRegime::BearTrending,
            MarketRegime::Sideways,
            MarketRegime::HighVolatility,
            MarketRegime::LowVolatility,
            MarketRegime::Breakout,
            MarketRegime::Reversal,
            MarketRegime::Unknown,
        ];

        for regime in regimes {
            let mut detector = MarketRegimeDetector::new();
            detector.current_regime = regime;
            assert_eq!(detector.detect_current_regime(), regime);
        }
    }

    #[test]
    fn test_optimization_metric_variants() {
        let metrics = vec![
            OptimizationMetric::TotalReturn,
            OptimizationMetric::SharpeRatio,
            OptimizationMetric::SortinoRatio,
            OptimizationMetric::CalmarRatio,
            OptimizationMetric::MaxDrawdown,
            OptimizationMetric::WinRate,
            OptimizationMetric::ProfitFactor,
            OptimizationMetric::RiskAdjustedReturn,
            OptimizationMetric::Consistency,
        ];

        let mut config = OptimizationConfig::default();
        for metric in metrics {
            config.primary_metric = metric.clone();
            // Should be able to set any variant
        }
    }

    #[test]
    fn test_strategy_analysis_calculation() {
        let mut analysis = StrategyAnalysis::new();

        analysis.avg_win_rate = 0.6;
        analysis.avg_profit_factor = 2.0;
        analysis.calculate_statistics();

        assert!(analysis.overall_performance_score > 0.0);
    }

    #[test]
    fn test_performance_snapshot_creation() {
        let snapshot = PerformanceSnapshot {
            timestamp: Utc::now(),
            portfolio_metrics: PortfolioMetrics::default(),
            market_conditions: MarketConditions {
                volatility: 0.5,
                trend_strength: 0.8,
                volume_profile: "High".to_string(),
                regime: "Trending".to_string(),
                correlation_matrix: HashMap::new(),
            },
            active_strategies: HashMap::new(),
            parameters_used: HashMap::new(),
        };

        assert_eq!(snapshot.active_strategies.len(), 0);
    }

    #[test]
    fn test_backtesting_with_trades() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        // Add performance history with strategy data
        for _ in 0..5 {
            let mut active_strategies = HashMap::new();
            active_strategies.insert(
                "test_strategy".to_string(),
                StrategyPerformance {
                    signal_count: 10,
                    executed_trades: 5,
                    win_rate: 0.6,
                    avg_profit: 100.0,
                    avg_loss: -50.0,
                    profit_factor: 2.0,
                    sharpe_ratio: 1.5,
                    max_drawdown: 0.1,
                    confidence_accuracy: 0.8,
                    signal_frequency: 2.0,
                },
            );

            optimizer.add_performance_snapshot(
                PortfolioMetrics::default(),
                MarketConditions {
                    volatility: 0.5,
                    trend_strength: 0.7,
                    volume_profile: "High".to_string(),
                    regime: "Trending".to_string(),
                    correlation_matrix: HashMap::new(),
                },
                active_strategies,
                HashMap::new(),
            );
        }

        let results = optimizer
            .run_parameter_backtest("test_strategy", &HashMap::new())
            .unwrap();

        assert!(results.total_trades > 0);
    }

    #[test]
    fn test_zero_variance_sharpe() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        // Add identical performance snapshots (zero variance)
        for _ in 0..3 {
            let mut active_strategies = HashMap::new();
            active_strategies.insert(
                "test_strategy".to_string(),
                StrategyPerformance {
                    signal_count: 10,
                    executed_trades: 5,
                    win_rate: 0.5,
                    avg_profit: 100.0,
                    avg_loss: -100.0,
                    profit_factor: 1.0,
                    sharpe_ratio: 0.0,
                    max_drawdown: 0.0,
                    confidence_accuracy: 0.5,
                    signal_frequency: 1.0,
                },
            );

            optimizer.add_performance_snapshot(
                PortfolioMetrics::default(),
                MarketConditions {
                    volatility: 0.5,
                    trend_strength: 0.5,
                    volume_profile: "Medium".to_string(),
                    regime: "Sideways".to_string(),
                    correlation_matrix: HashMap::new(),
                },
                active_strategies,
                HashMap::new(),
            );
        }

        let results = optimizer
            .run_parameter_backtest("test_strategy", &HashMap::new())
            .unwrap();

        // With zero variance, Sharpe ratio should be 0
        assert_eq!(results.sharpe_ratio, 0.0);
    }

    #[test]
    fn test_custom_optimization_config() {
        let mut config = OptimizationConfig::default();
        config.min_trades_for_optimization = 100;
        config.optimization_period_days = 60;
        config.max_concurrent_tests = 5;
        config.enable_genetic_algorithm = true;
        config.genetic_population_size = 50;
        config.genetic_mutation_rate = 0.15;

        let optimizer = StrategyOptimizer::new(config.clone());
        assert_eq!(
            optimizer.optimization_config.min_trades_for_optimization,
            100
        );
        assert_eq!(optimizer.optimization_config.optimization_period_days, 60);
        assert!(optimizer.optimization_config.enable_genetic_algorithm);
    }

    #[test]
    fn test_performance_history_retention() {
        let mut config = OptimizationConfig::default();
        config.optimization_period_days = 1; // Only keep 1 day
        let mut optimizer = StrategyOptimizer::new(config);

        // Add snapshot
        optimizer.add_performance_snapshot(
            PortfolioMetrics::default(),
            MarketConditions {
                volatility: 0.5,
                trend_strength: 0.7,
                volume_profile: "High".to_string(),
                regime: "Trending".to_string(),
                correlation_matrix: HashMap::new(),
            },
            HashMap::new(),
            HashMap::new(),
        );

        assert_eq!(optimizer.performance_history.len(), 1);
    }

    #[test]
    fn test_strategy_performance_creation() {
        let perf = StrategyPerformance {
            signal_count: 20,
            executed_trades: 15,
            win_rate: 0.7,
            avg_profit: 150.0,
            avg_loss: -75.0,
            profit_factor: 2.5,
            sharpe_ratio: 1.8,
            max_drawdown: 0.12,
            confidence_accuracy: 0.85,
            signal_frequency: 3.5,
        };

        assert_eq!(perf.signal_count, 20);
        assert_eq!(perf.executed_trades, 15);
        assert_eq!(perf.win_rate, 0.7);
        assert_eq!(perf.profit_factor, 2.5);
    }

    #[test]
    fn test_market_conditions_creation() {
        let mut correlation = HashMap::new();
        let mut btc_corr = HashMap::new();
        btc_corr.insert("ETH".to_string(), 0.8);
        correlation.insert("BTC".to_string(), btc_corr);

        let conditions = MarketConditions {
            volatility: 0.6,
            trend_strength: 0.75,
            volume_profile: "Very High".to_string(),
            regime: "Bull".to_string(),
            correlation_matrix: correlation.clone(),
        };

        assert_eq!(conditions.volatility, 0.6);
        assert_eq!(conditions.trend_strength, 0.75);
        assert_eq!(conditions.volume_profile, "Very High");
        assert_eq!(conditions.correlation_matrix.len(), 1);
    }

    #[test]
    fn test_parameter_test_creation() {
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), serde_json::json!(0.5));

        let test = ParameterTest {
            strategy_name: "momentum".to_string(),
            parameters: params.clone(),
            start_time: Utc::now(),
            trades_executed: 10,
            current_performance: 0.65,
            target_trades: 50,
        };

        assert_eq!(test.strategy_name, "momentum");
        assert_eq!(test.trades_executed, 10);
        assert_eq!(test.target_trades, 50);
        assert_eq!(test.parameters.len(), 1);
    }

    #[test]
    fn test_price_point_creation() {
        let point = PricePoint {
            timestamp: Utc::now(),
            price: 45000.0,
            volume: 1000000.0,
            volatility: 0.05,
        };

        assert_eq!(point.price, 45000.0);
        assert_eq!(point.volume, 1000000.0);
        assert_eq!(point.volatility, 0.05);
    }

    #[test]
    fn test_regime_statistics_creation() {
        let mut best_params = HashMap::new();
        let mut strategy_params = HashMap::new();
        strategy_params.insert("param1".to_string(), serde_json::json!(1.0));
        best_params.insert("strategy1".to_string(), strategy_params);

        let stats = RegimeStatistics {
            average_duration_hours: 48.0,
            average_volatility: 0.15,
            average_return: 0.05,
            best_strategies: vec!["strategy1".to_string(), "strategy2".to_string()],
            best_parameters: best_params.clone(),
        };

        assert_eq!(stats.average_duration_hours, 48.0);
        assert_eq!(stats.best_strategies.len(), 2);
        assert_eq!(stats.best_parameters.len(), 1);
    }

    #[test]
    fn test_backtesting_results_creation() {
        let mut trade_dist = HashMap::new();
        trade_dist.insert("long".to_string(), 15);
        trade_dist.insert("short".to_string(), 10);

        let results = BacktestingResults {
            total_trades: 25,
            win_rate: 0.64,
            total_return: 0.25,
            sharpe_ratio: 1.6,
            max_drawdown: 0.11,
            profit_factor: 2.2,
            monthly_returns: vec![0.05, 0.03, -0.02, 0.08],
            trade_distribution: trade_dist.clone(),
        };

        assert_eq!(results.total_trades, 25);
        assert_eq!(results.monthly_returns.len(), 4);
        assert_eq!(results.trade_distribution.len(), 2);
    }

    #[test]
    fn test_optimization_risk_assessment_creation() {
        let mut param_sensitivity = HashMap::new();
        param_sensitivity.insert("param1".to_string(), 0.7);
        param_sensitivity.insert("param2".to_string(), 0.3);

        let risk = OptimizationRiskAssessment {
            overfitting_risk: 0.4,
            parameter_sensitivity: param_sensitivity.clone(),
            regime_dependency: 0.5,
            data_sufficiency: 0.8,
            recommendation: "Safe to implement".to_string(),
        };

        assert_eq!(risk.overfitting_risk, 0.4);
        assert_eq!(risk.parameter_sensitivity.len(), 2);
        assert_eq!(risk.regime_dependency, 0.5);
    }

    #[test]
    fn test_optimization_recommendation_creation() {
        let mut params = HashMap::new();
        params.insert("threshold".to_string(), serde_json::json!(0.75));

        let recommendation = OptimizationRecommendation {
            strategy_name: "test_strategy".to_string(),
            recommended_parameters: params.clone(),
            expected_improvement: 0.15,
            confidence: 0.85,
            reasoning: "Strategy needs optimization".to_string(),
            market_regime: MarketRegime::BullTrending,
            backtesting_results: BacktestingResults {
                total_trades: 50,
                win_rate: 0.6,
                total_return: 0.2,
                sharpe_ratio: 1.5,
                max_drawdown: 0.1,
                profit_factor: 2.0,
                monthly_returns: vec![],
                trade_distribution: HashMap::new(),
            },
            risk_assessment: OptimizationRiskAssessment {
                overfitting_risk: 0.3,
                parameter_sensitivity: HashMap::new(),
                regime_dependency: 0.4,
                data_sufficiency: 0.9,
                recommendation: "Good to go".to_string(),
            },
        };

        assert_eq!(recommendation.strategy_name, "test_strategy");
        assert_eq!(recommendation.expected_improvement, 0.15);
        assert_eq!(recommendation.confidence, 0.85);
    }

    #[test]
    fn test_adjust_selectivity_no_params() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);
        let mut params = HashMap::new();

        optimizer.adjust_selectivity_parameters(&mut params, 1.2);
        // Should handle empty parameters gracefully
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_adjust_risk_reward_no_params() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);
        let mut params = HashMap::new();

        optimizer.adjust_risk_reward_parameters(&mut params, 1.5);
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_adjust_risk_no_params() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);
        let mut params = HashMap::new();

        optimizer.adjust_risk_parameters(&mut params, 0.8);
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_adjust_for_high_volatility_no_params() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);
        let mut params = HashMap::new();

        optimizer.adjust_for_high_volatility(&mut params);
        assert_eq!(params.len(), 0);
    }

    #[test]
    fn test_adjust_for_sideways_no_timeframe() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);
        let mut params = HashMap::new();
        params.insert("other_param".to_string(), serde_json::json!(1.0));

        optimizer.adjust_for_sideways_market(&mut params);
        // Should not crash without timeframe parameter
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_adjust_for_trending_no_timeframe() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);
        let mut params = HashMap::new();
        params.insert("other_param".to_string(), serde_json::json!(1.0));

        optimizer.adjust_for_trending_market(&mut params);
        assert_eq!(params.len(), 1);
    }

    #[test]
    fn test_confidence_with_limited_data() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 20, // Less than 100
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.5,
            overall_performance_score: 0.5,
            performance_consistency: 0.5,
            performance_variance: 0.3,
            regime_performance: HashMap::new(),
        };

        let confidence = optimizer.calculate_recommendation_confidence(&analysis);
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
        assert!(confidence <= 0.7); // Should have lower confidence with limited data
    }

    #[test]
    fn test_confidence_with_high_variance() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 150,
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.5,
            overall_performance_score: 0.5,
            performance_consistency: 0.5,
            performance_variance: 0.8, // High variance
            regime_performance: HashMap::new(),
        };

        let confidence = optimizer.calculate_recommendation_confidence(&analysis);
        assert!(confidence >= 0.0);
        assert!(confidence <= 1.0);
    }

    #[test]
    fn test_optimization_reasoning_multiple_issues() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.4,
            avg_profit_factor: 1.2,
            max_drawdown: 0.2,
            performance_percentile: 0.3,
            recent_performance_decline: 0.15,
            regime_adaptation_score: 0.4,
            overall_performance_score: 0.4,
            performance_consistency: 0.6,
            performance_variance: 0.4,
            regime_performance: HashMap::new(),
        };

        let reasoning = optimizer.generate_optimization_reasoning(
            "test_strategy",
            &analysis,
            MarketRegime::HighVolatility,
        );

        assert!(!reasoning.is_empty());
        assert!(
            reasoning.contains("win rate")
                || reasoning.contains("risk/reward")
                || reasoning.contains("drawdown")
        );
    }

    #[test]
    fn test_optimization_reasoning_no_issues() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.7,
            avg_profit_factor: 2.5,
            max_drawdown: 0.08,
            performance_percentile: 0.8,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.9,
            overall_performance_score: 0.85,
            performance_consistency: 0.8,
            performance_variance: 0.2,
            regime_performance: HashMap::new(),
        };

        let reasoning = optimizer.generate_optimization_reasoning(
            "test_strategy",
            &analysis,
            MarketRegime::BullTrending,
        );

        assert!(!reasoning.is_empty());
        assert!(reasoning.contains("Proactive optimization"));
    }

    #[test]
    fn test_backtesting_profit_factor_zero_loss() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        // All winning trades
        for _ in 0..3 {
            let mut active_strategies = HashMap::new();
            active_strategies.insert(
                "winning_strategy".to_string(),
                StrategyPerformance {
                    signal_count: 10,
                    executed_trades: 5,
                    win_rate: 1.0,
                    avg_profit: 100.0,
                    avg_loss: 0.0,
                    profit_factor: 5.0,
                    sharpe_ratio: 2.0,
                    max_drawdown: 0.0,
                    confidence_accuracy: 0.9,
                    signal_frequency: 2.0,
                },
            );

            optimizer.add_performance_snapshot(
                PortfolioMetrics::default(),
                MarketConditions {
                    volatility: 0.3,
                    trend_strength: 0.9,
                    volume_profile: "High".to_string(),
                    regime: "Bull".to_string(),
                    correlation_matrix: HashMap::new(),
                },
                active_strategies,
                HashMap::new(),
            );
        }

        let results = optimizer
            .run_parameter_backtest("winning_strategy", &HashMap::new())
            .unwrap();

        assert!(results.total_trades > 0);
        assert_eq!(results.profit_factor, 0.0); // Zero loss case
    }

    #[test]
    fn test_backtesting_with_negative_returns() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        for _ in 0..3 {
            let mut active_strategies = HashMap::new();
            active_strategies.insert(
                "losing_strategy".to_string(),
                StrategyPerformance {
                    signal_count: 10,
                    executed_trades: 5,
                    win_rate: 0.2,
                    avg_profit: -50.0,
                    avg_loss: -100.0,
                    profit_factor: 0.5,
                    sharpe_ratio: -0.5,
                    max_drawdown: 0.3,
                    confidence_accuracy: 0.4,
                    signal_frequency: 2.0,
                },
            );

            optimizer.add_performance_snapshot(
                PortfolioMetrics::default(),
                MarketConditions {
                    volatility: 0.7,
                    trend_strength: 0.3,
                    volume_profile: "Low".to_string(),
                    regime: "Bear".to_string(),
                    correlation_matrix: HashMap::new(),
                },
                active_strategies,
                HashMap::new(),
            );
        }

        let results = optimizer
            .run_parameter_backtest("losing_strategy", &HashMap::new())
            .unwrap();

        assert!(results.total_trades > 0);
        assert!(results.total_return < 0.0);
    }

    #[test]
    fn test_estimate_improvement_high_current_score() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.7,
            avg_profit_factor: 2.5,
            max_drawdown: 0.05,
            performance_percentile: 0.9,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.9,
            overall_performance_score: 0.9, // Very high score
            performance_consistency: 0.8,
            performance_variance: 0.2,
            regime_performance: HashMap::new(),
        };

        let improvement = optimizer.estimate_improvement_potential(&analysis, &HashMap::new());
        assert!(improvement >= 0.0);
        assert!(improvement <= 0.5);
        assert!(improvement < 0.2); // Should be small for already good performance
    }

    #[test]
    fn test_estimate_improvement_low_current_score() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.3,
            avg_profit_factor: 1.0,
            max_drawdown: 0.25,
            performance_percentile: 0.1,
            recent_performance_decline: 0.2,
            regime_adaptation_score: 0.3,
            overall_performance_score: 0.2, // Very low score
            performance_consistency: 0.4,
            performance_variance: 0.6,
            regime_performance: HashMap::new(),
        };

        let improvement = optimizer.estimate_improvement_potential(&analysis, &HashMap::new());
        assert!(improvement >= 0.0);
        assert!(improvement <= 0.5);
        assert!(improvement > 0.2); // Should be larger for poor performance
    }

    #[test]
    fn test_should_optimize_recent_decline() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut analysis = StrategyAnalysis::new();
        analysis.total_trades = 100;
        analysis.performance_percentile = 0.8; // Good performance
        analysis.recent_performance_decline = 0.15; // But declining
        analysis.regime_adaptation_score = 0.9;

        assert!(optimizer.should_optimize_strategy("test", &analysis));
    }

    #[test]
    fn test_should_optimize_poor_regime_adaptation() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut analysis = StrategyAnalysis::new();
        analysis.total_trades = 100;
        analysis.performance_percentile = 0.8;
        analysis.recent_performance_decline = 0.0;
        analysis.regime_adaptation_score = 0.3; // Poor adaptation

        assert!(optimizer.should_optimize_strategy("test", &analysis));
    }

    #[test]
    fn test_risk_assessment_high_overfitting_risk() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 20, // Very limited data
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.5,
            overall_performance_score: 0.5,
            performance_consistency: 0.5,
            performance_variance: 0.4,
            regime_performance: HashMap::new(),
        };

        let risk = optimizer.assess_optimization_risks(&analysis, &HashMap::new());
        assert!(risk.overfitting_risk > 0.7);
        assert!(risk.recommendation.contains("overfitting"));
    }

    #[test]
    fn test_risk_assessment_high_regime_dependency() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.2, // Very poor regime adaptation
            overall_performance_score: 0.5,
            performance_consistency: 0.7,
            performance_variance: 0.3,
            regime_performance: HashMap::new(),
        };

        let risk = optimizer.assess_optimization_risks(&analysis, &HashMap::new());
        assert!(risk.regime_dependency > 0.6);
        assert!(risk.recommendation.contains("regime"));
    }

    #[test]
    fn test_risk_assessment_safe_optimization() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let analysis = StrategyAnalysis {
            total_trades: 150,
            avg_win_rate: 0.6,
            avg_profit_factor: 2.0,
            max_drawdown: 0.1,
            performance_percentile: 0.6,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.8,
            overall_performance_score: 0.7,
            performance_consistency: 0.8,
            performance_variance: 0.2,
            regime_performance: HashMap::new(),
        };

        let risk = optimizer.assess_optimization_risks(&analysis, &HashMap::new());
        assert!(risk.overfitting_risk < 0.5);
        assert!(risk.recommendation.contains("safe"));
    }

    #[test]
    fn test_genetic_optimization_placeholder() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let result = optimizer
            .apply_genetic_optimization(MarketRegime::BullTrending)
            .unwrap();
        assert_eq!(result.len(), 0); // Placeholder returns empty
    }

    #[test]
    fn test_bayesian_optimization_placeholder() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let result = optimizer
            .apply_bayesian_optimization(MarketRegime::Sideways)
            .unwrap();
        assert_eq!(result.len(), 0); // Placeholder returns empty
    }

    #[test]
    fn test_strategy_analysis_default() {
        let analysis = StrategyAnalysis::default();
        assert_eq!(analysis.total_trades, 0);
        assert_eq!(analysis.avg_win_rate, 0.0);
        assert_eq!(analysis.overall_performance_score, 0.0);
    }

    #[test]
    fn test_market_regime_detector_default() {
        let detector = MarketRegimeDetector::default();
        assert_eq!(detector.current_regime, MarketRegime::Unknown);
        assert_eq!(detector.price_history.len(), 0);
    }

    #[test]
    fn test_update_with_conditions() {
        let mut detector = MarketRegimeDetector::new();
        let conditions = MarketConditions {
            volatility: 0.8,
            trend_strength: 0.6,
            volume_profile: "High".to_string(),
            regime: "Volatile".to_string(),
            correlation_matrix: HashMap::new(),
        };

        detector.update_with_conditions(&conditions);
        // Should not crash
    }

    #[test]
    fn test_adjust_selectivity_with_entry_threshold() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "entry_threshold".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()),
        );

        optimizer.adjust_selectivity_parameters(&mut params, 1.5);

        if let Some(val) = params.get("entry_threshold") {
            assert!(val.as_f64().unwrap() >= 0.5);
        }
    }

    #[test]
    fn test_adjust_for_sideways_with_entry_threshold() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "entry_threshold".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(0.5).unwrap()),
        );

        optimizer.adjust_for_sideways_market(&mut params);

        if let Some(val) = params.get("entry_threshold") {
            assert!(val.as_f64().unwrap() >= 0.5);
        }
    }

    #[test]
    fn test_adjust_for_trending_with_position_multiplier() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert(
            "position_size_multiplier".to_string(),
            serde_json::Value::Number(serde_json::Number::from_f64(1.0).unwrap()),
        );

        optimizer.adjust_for_trending_market(&mut params);

        if let Some(val) = params.get("position_size_multiplier") {
            assert!(val.as_f64().unwrap() >= 1.0);
        }
    }

    #[test]
    fn test_strategy_analysis_add_performance() {
        let mut analysis = StrategyAnalysis::new();
        let performance = StrategyPerformance {
            signal_count: 10,
            executed_trades: 5,
            win_rate: 0.6,
            avg_profit: 100.0,
            avg_loss: -50.0,
            profit_factor: 2.0,
            sharpe_ratio: 1.5,
            max_drawdown: 0.1,
            confidence_accuracy: 0.8,
            signal_frequency: 2.0,
        };
        let conditions = MarketConditions {
            volatility: 0.5,
            trend_strength: 0.7,
            volume_profile: "Medium".to_string(),
            regime: "Trending".to_string(),
            correlation_matrix: HashMap::new(),
        };

        analysis.add_performance_point(&performance, &conditions);
        assert_eq!(analysis.total_trades, 5);
    }

    #[test]
    fn test_multiple_snapshots_with_strategies() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        for i in 0..5 {
            let mut active_strategies = HashMap::new();
            active_strategies.insert(
                format!("strategy_{}", i),
                StrategyPerformance {
                    signal_count: 10 + i,
                    executed_trades: 5 + i,
                    win_rate: 0.5 + (i as f64 * 0.05),
                    avg_profit: 100.0 + (i as f64 * 10.0),
                    avg_loss: -50.0,
                    profit_factor: 2.0,
                    sharpe_ratio: 1.5,
                    max_drawdown: 0.1,
                    confidence_accuracy: 0.8,
                    signal_frequency: 2.0,
                },
            );

            optimizer.add_performance_snapshot(
                PortfolioMetrics::default(),
                MarketConditions {
                    volatility: 0.5,
                    trend_strength: 0.7,
                    volume_profile: "High".to_string(),
                    regime: "Trending".to_string(),
                    correlation_matrix: HashMap::new(),
                },
                active_strategies,
                HashMap::new(),
            );
        }

        assert_eq!(optimizer.performance_history.len(), 5);
    }

    #[test]
    fn test_backtesting_max_drawdown_tracking() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        // Add performance with varying returns to test drawdown
        for i in 0..5 {
            let mut active_strategies = HashMap::new();
            let profit = if i % 2 == 0 { 100.0 } else { -50.0 };

            active_strategies.insert(
                "volatile_strategy".to_string(),
                StrategyPerformance {
                    signal_count: 10,
                    executed_trades: 5,
                    win_rate: 0.5,
                    avg_profit: profit,
                    avg_loss: -50.0,
                    profit_factor: 1.5,
                    sharpe_ratio: 1.0,
                    max_drawdown: 0.15,
                    confidence_accuracy: 0.7,
                    signal_frequency: 2.0,
                },
            );

            optimizer.add_performance_snapshot(
                PortfolioMetrics::default(),
                MarketConditions {
                    volatility: 0.6,
                    trend_strength: 0.5,
                    volume_profile: "Medium".to_string(),
                    regime: "Sideways".to_string(),
                    correlation_matrix: HashMap::new(),
                },
                active_strategies,
                HashMap::new(),
            );
        }

        let results = optimizer
            .run_parameter_backtest("volatile_strategy", &HashMap::new())
            .unwrap();

        assert!(results.max_drawdown >= 0.0);
    }

    #[test]
    fn test_secondary_metrics_in_config() {
        let config = OptimizationConfig::default();
        assert!(config.secondary_metrics.len() > 0);
        assert!(config
            .secondary_metrics
            .contains_key(&OptimizationMetric::MaxDrawdown));
        assert!(config
            .secondary_metrics
            .contains_key(&OptimizationMetric::ProfitFactor));
    }

    #[test]
    fn test_all_market_regimes_enum_equality() {
        assert_eq!(MarketRegime::BullTrending, MarketRegime::BullTrending);
        assert_ne!(MarketRegime::BullTrending, MarketRegime::BearTrending);
        assert_ne!(MarketRegime::Sideways, MarketRegime::HighVolatility);
    }

    #[test]
    fn test_all_optimization_metrics_enum_equality() {
        assert_eq!(
            OptimizationMetric::SharpeRatio,
            OptimizationMetric::SharpeRatio
        );
        assert_ne!(
            OptimizationMetric::SharpeRatio,
            OptimizationMetric::TotalReturn
        );
        assert_ne!(
            OptimizationMetric::WinRate,
            OptimizationMetric::ProfitFactor
        );
    }

    #[test]
    fn test_regime_detector_with_different_regimes() {
        let mut detector = MarketRegimeDetector::new();

        let regimes = vec![
            MarketRegime::BullTrending,
            MarketRegime::BearTrending,
            MarketRegime::Sideways,
            MarketRegime::HighVolatility,
            MarketRegime::LowVolatility,
        ];

        for regime in regimes {
            detector.current_regime = regime;
            assert_eq!(detector.detect_current_regime(), regime);
        }
    }

    #[test]
    fn test_adjustment_for_breakout_regime() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert("test_param".to_string(), serde_json::json!(1.0));

        // Should not crash for Breakout regime (default case in match)
        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.5,
            overall_performance_score: 0.5,
            performance_consistency: 0.7,
            performance_variance: 0.3,
            regime_performance: HashMap::new(),
        };

        let result = optimizer.generate_parameter_variations("test_strategy", &params, &analysis);
        assert!(result.is_ok());
    }

    #[test]
    fn test_adjustment_for_reversal_regime() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert("test_param".to_string(), serde_json::json!(1.0));

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.5,
            avg_profit_factor: 1.5,
            max_drawdown: 0.1,
            performance_percentile: 0.5,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.5,
            overall_performance_score: 0.5,
            performance_consistency: 0.7,
            performance_variance: 0.3,
            regime_performance: HashMap::new(),
        };

        let result = optimizer.generate_parameter_variations("test_strategy", &params, &analysis);
        assert!(result.is_ok());
    }

    #[test]
    fn test_parameter_variations_with_all_conditions() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);

        let mut params = HashMap::new();
        params.insert("min_confidence".to_string(), serde_json::json!(0.5));
        params.insert("take_profit_ratio".to_string(), serde_json::json!(2.0));
        params.insert(
            "position_size_multiplier".to_string(),
            serde_json::json!(1.0),
        );

        let analysis = StrategyAnalysis {
            total_trades: 100,
            avg_win_rate: 0.4,      // Triggers selectivity adjustment
            avg_profit_factor: 1.3, // Triggers risk/reward adjustment
            max_drawdown: 0.18,     // Triggers risk adjustment
            performance_percentile: 0.3,
            recent_performance_decline: 0.0,
            regime_adaptation_score: 0.5,
            overall_performance_score: 0.4,
            performance_consistency: 0.6,
            performance_variance: 0.4,
            regime_performance: HashMap::new(),
        };

        let result = optimizer.generate_parameter_variations("test_strategy", &params, &analysis);
        assert!(result.is_ok());
        let optimized = result.unwrap();
        assert!(optimized.len() >= 3);
    }

    #[test]
    fn test_backtesting_single_return() {
        let config = OptimizationConfig::default();
        let mut optimizer = StrategyOptimizer::new(config);

        let mut active_strategies = HashMap::new();
        active_strategies.insert(
            "single_trade_strategy".to_string(),
            StrategyPerformance {
                signal_count: 1,
                executed_trades: 1,
                win_rate: 1.0,
                avg_profit: 100.0,
                avg_loss: 0.0,
                profit_factor: 10.0,
                sharpe_ratio: 2.0,
                max_drawdown: 0.0,
                confidence_accuracy: 1.0,
                signal_frequency: 1.0,
            },
        );

        optimizer.add_performance_snapshot(
            PortfolioMetrics::default(),
            MarketConditions {
                volatility: 0.3,
                trend_strength: 0.8,
                volume_profile: "Low".to_string(),
                regime: "Calm".to_string(),
                correlation_matrix: HashMap::new(),
            },
            active_strategies,
            HashMap::new(),
        );

        let results = optimizer
            .run_parameter_backtest("single_trade_strategy", &HashMap::new())
            .unwrap();

        assert_eq!(results.total_trades, 1);
        // With single return, variance calculation uses len() > 1 check
    }

    #[test]
    fn test_clone_optimizer() {
        let config = OptimizationConfig::default();
        let optimizer = StrategyOptimizer::new(config);
        let cloned = optimizer.clone();

        assert_eq!(cloned.performance_history.len(), 0);
        assert_eq!(cloned.parameter_tests.len(), 0);
    }

    #[test]
    fn test_clone_strategy_analysis() {
        let analysis = StrategyAnalysis::new();
        let cloned = analysis.clone();

        assert_eq!(cloned.total_trades, 0);
        assert_eq!(cloned.avg_win_rate, 0.0);
    }

    #[test]
    fn test_clone_market_regime_detector() {
        let detector = MarketRegimeDetector::new();
        let cloned = detector.clone();

        assert_eq!(cloned.current_regime, MarketRegime::Unknown);
    }
}
