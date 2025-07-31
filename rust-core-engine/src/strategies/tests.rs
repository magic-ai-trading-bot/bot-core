#[cfg(test)]
mod tests {
    use super::*;
    use crate::strategies::{
        macd_strategy::MacdStrategy,
        rsi_strategy::RsiStrategy,
        strategy_engine::StrategyEngine,
        types::{MarketData, TradingSignal},
    };
    use chrono::Utc;

    fn create_test_market_data(close: f64, volume: f64) -> MarketData {
        MarketData {
            symbol: "BTCUSDT".to_string(),
            timestamp: Utc::now().timestamp_millis(),
            open: close - 10.0,
            high: close + 10.0,
            low: close - 10.0,
            close,
            volume,
            quote_volume: volume * close,
            trades: 1000,
            taker_buy_base: volume * 0.5,
            taker_buy_quote: volume * close * 0.5,
        }
    }

    #[test]
    fn test_rsi_strategy_oversold() {
        let mut strategy = RsiStrategy::new();

        // Create market data that should trigger oversold condition
        let mut data_points = vec![];
        for i in 0..20 {
            let price = 50000.0 - (i as f64 * 100.0); // Decreasing prices
            data_points.push(create_test_market_data(price, 1000.0));
        }

        let signal = strategy.analyze(&data_points);

        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal, TradingSignal::Buy);
        assert!(signal.confidence > 0.5);
    }

    #[test]
    fn test_rsi_strategy_overbought() {
        let mut strategy = RsiStrategy::new();

        // Create market data that should trigger overbought condition
        let mut data_points = vec![];
        for i in 0..20 {
            let price = 50000.0 + (i as f64 * 100.0); // Increasing prices
            data_points.push(create_test_market_data(price, 1000.0));
        }

        let signal = strategy.analyze(&data_points);

        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal, TradingSignal::Sell);
        assert!(signal.confidence > 0.5);
    }

    #[test]
    fn test_macd_strategy_bullish_crossover() {
        let mut strategy = MacdStrategy::new();

        // Create market data for bullish MACD crossover
        let mut data_points = vec![];

        // First create downtrend
        for i in 0..15 {
            let price = 50000.0 - (i as f64 * 50.0);
            data_points.push(create_test_market_data(price, 1000.0));
        }

        // Then create uptrend
        for i in 0..15 {
            let price = 49250.0 + (i as f64 * 100.0);
            data_points.push(create_test_market_data(price, 1200.0));
        }

        let signal = strategy.analyze(&data_points);

        assert!(signal.is_some());
        let signal = signal.unwrap();
        assert_eq!(signal.signal, TradingSignal::Buy);
    }

    #[test]
    fn test_strategy_engine_consensus() {
        let mut engine = StrategyEngine::new();

        // Create market data that should trigger buy signals from multiple strategies
        let mut data_points = vec![];

        // Create V-shaped recovery pattern
        for i in 0..15 {
            let price = 50000.0 - (i as f64 * 200.0);
            data_points.push(create_test_market_data(price, 1000.0 + i as f64 * 50.0));
        }

        for i in 0..15 {
            let price = 47000.0 + (i as f64 * 300.0);
            data_points.push(create_test_market_data(price, 1500.0 + i as f64 * 100.0));
        }

        let combined_signal = engine.analyze_all(&data_points).await;

        assert!(combined_signal.is_some());
        let signal = combined_signal.unwrap();
        assert_eq!(signal.final_signal, TradingSignal::Buy);
        assert!(signal.combined_confidence > 0.6);
        assert!(signal.strategy_signals.len() > 2);
    }

    #[test]
    fn test_insufficient_data() {
        let mut strategy = RsiStrategy::new();

        // Test with insufficient data points
        let data_points = vec![
            create_test_market_data(50000.0, 1000.0),
            create_test_market_data(50100.0, 1000.0),
        ];

        let signal = strategy.analyze(&data_points);
        assert!(signal.is_none());
    }

    #[test]
    fn test_risk_management_stop_loss() {
        use crate::trading::risk_manager::{RiskManager, RiskParameters};

        let risk_params = RiskParameters {
            max_position_size: 0.1,
            max_risk_per_trade: 0.02,
            stop_loss_percentage: 0.02,
            take_profit_percentage: 0.04,
            max_open_positions: 3,
            max_daily_loss: 0.05,
            max_drawdown: 0.10,
        };

        let risk_manager = RiskManager::new(risk_params);

        let position_size = risk_manager.calculate_position_size(
            10000.0, // account balance
            50000.0, // entry price
            0.7,     // signal confidence
        );

        assert!(position_size > 0.0);
        assert!(position_size <= 1000.0); // 10% of account

        let stop_loss = risk_manager.calculate_stop_loss(50000.0, true);
        assert_eq!(stop_loss, 49000.0); // 2% below entry

        let take_profit = risk_manager.calculate_take_profit(50000.0, true);
        assert_eq!(take_profit, 52000.0); // 4% above entry
    }

    #[tokio::test]
    async fn test_paper_trading_execution() {
        use crate::paper_trading::{PaperTradingEngine, PaperTradingSettings};

        let settings = PaperTradingSettings::default();
        let mut engine = PaperTradingEngine::new(settings);

        // Execute a buy trade
        let trade_result = engine
            .execute_trade("BTCUSDT", TradingSignal::Buy, 50000.0, 0.8, 0.001)
            .await;

        assert!(trade_result.is_ok());

        let portfolio = engine.get_portfolio();
        assert!(portfolio.positions.contains_key("BTCUSDT"));
        assert!(portfolio.balance < 10000.0); // Some balance used
    }
}
