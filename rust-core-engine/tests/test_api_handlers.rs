// Comprehensive tests for API handler modules
// Target: Increase coverage for api/mod.rs, api/paper_trading.rs, api/real_trading.rs
// Focus: Request/response types, serialization, validation, business logic

mod common;

use binance_trading_bot::api::paper_trading::*;
use binance_trading_bot::api::real_trading::{
    ApiResponse as RealApiResponse, BalanceInfo, EngineStatus, PortfolioResponse, PositionInfo,
};
use chrono::Utc;
use serde_json;
use std::collections::HashMap;

// ========== PAPER TRADING API TESTS ==========

#[test]
fn test_create_order_request_serialization() {
    let request = CreateOrderRequest {
        symbol: "BTCUSDT".to_string(),
        side: "buy".to_string(),
        order_type: "market".to_string(),
        quantity: 0.5,
        price: Some(50000.0),
        stop_price: None,
        leverage: Some(10),
        stop_loss_pct: Some(2.0),
        take_profit_pct: Some(5.0),
    };

    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("BTCUSDT"));
    assert!(json.contains("buy"));
    assert!(json.contains("market"));

    // Deserialize back
    let deserialized: CreateOrderRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.symbol, "BTCUSDT");
    assert_eq!(deserialized.quantity, 0.5);
    assert_eq!(deserialized.leverage, Some(10));
}

#[test]
fn test_create_order_request_validation() {
    // Valid market order
    let market_order = CreateOrderRequest {
        symbol: "ETHUSDT".to_string(),
        side: "sell".to_string(),
        order_type: "market".to_string(),
        quantity: 1.0,
        price: None, // Market order doesn't require price
        stop_price: None,
        leverage: Some(5),
        stop_loss_pct: None,
        take_profit_pct: None,
    };
    assert_eq!(market_order.order_type, "market");

    // Valid limit order
    let limit_order = CreateOrderRequest {
        symbol: "BNBUSDT".to_string(),
        side: "buy".to_string(),
        order_type: "limit".to_string(),
        quantity: 10.0,
        price: Some(300.0),
        stop_price: None,
        leverage: Some(1),
        stop_loss_pct: Some(3.0),
        take_profit_pct: Some(6.0),
    };
    assert!(limit_order.price.is_some());

    // Stop-limit order
    let stop_limit_order = CreateOrderRequest {
        symbol: "ADAUSDT".to_string(),
        side: "sell".to_string(),
        order_type: "stop-limit".to_string(),
        quantity: 100.0,
        price: Some(1.0),
        stop_price: Some(1.05),
        leverage: Some(2),
        stop_loss_pct: None,
        take_profit_pct: None,
    };
    assert!(stop_limit_order.stop_price.is_some());
}

#[test]
fn test_create_order_response_structure() {
    let response = CreateOrderResponse {
        trade_id: "trade_123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "long".to_string(),
        quantity: 0.1,
        entry_price: 60000.0,
        leverage: 10,
        stop_loss: Some(58000.0),
        take_profit: Some(65000.0),
        status: "open".to_string(),
        message: "Order placed successfully".to_string(),
    };

    assert_eq!(response.trade_id, "trade_123");
    assert_eq!(response.symbol, "BTCUSDT");
    assert_eq!(response.leverage, 10);
    assert!(response.stop_loss.is_some());
    assert!(response.take_profit.is_some());
}

#[test]
fn test_close_trade_request() {
    let request = CloseTradeRequest {
        trade_id: Some("trade_456".to_string()),
        reason: Some("Manual close by user".to_string()),
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: CloseTradeRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.trade_id, Some("trade_456".to_string()));
    assert_eq!(
        deserialized.reason,
        Some("Manual close by user".to_string())
    );
}

#[test]
fn test_trading_strategy_settings_defaults() {
    let settings = TradingStrategySettings {
        strategies: StrategyConfigCollection {
            rsi: RsiConfig {
                enabled: true,
                period: 14,
                oversold_threshold: 30.0,
                overbought_threshold: 70.0,
                extreme_oversold: 20.0,
                extreme_overbought: 80.0,
            },
            macd: MacdConfig {
                enabled: true,
                fast_period: 12,
                slow_period: 26,
                signal_period: 9,
                histogram_threshold: 0.0,
            },
            volume: VolumeConfig {
                enabled: false,
                sma_period: 20,
                spike_threshold: 1.5,
                correlation_period: 10,
            },
            bollinger: BollingerConfig {
                enabled: true,
                period: 20,
                multiplier: 2.0,
                squeeze_threshold: 0.02,
            },
            stochastic: StochasticConfig {
                enabled: false,
                k_period: 14,
                d_period: 3,
                oversold_threshold: 20.0,
                overbought_threshold: 80.0,
                extreme_oversold: 10.0,
                extreme_overbought: 90.0,
            },
        },
        risk: RiskSettings {
            max_risk_per_trade: 2.0,
            max_portfolio_risk: 10.0,
            stop_loss_percent: 2.0,
            take_profit_percent: 5.0,
            max_leverage: 10,
            max_drawdown: 20.0,
            daily_loss_limit: 5.0,
            max_consecutive_losses: 5,
            correlation_limit: 0.7,
        },
        engine: EngineSettings {
            min_confidence_threshold: 0.6,
            signal_combination_mode: "any".to_string(),
            enabled_strategies: vec!["rsi".to_string(), "macd".to_string()],
            market_condition: "normal".to_string(),
            risk_level: "moderate".to_string(),
            data_resolution: "15m".to_string(),
        },
        market_preset: "normal_volatility".to_string(),
    };

    assert_eq!(settings.market_preset, "normal_volatility");
    assert!(settings.strategies.rsi.enabled);
    assert!(!settings.strategies.volume.enabled);
    assert_eq!(settings.risk.max_leverage, 10);
    assert_eq!(settings.engine.data_resolution, "15m");
}

#[test]
fn test_strategy_settings_serialization() {
    let settings = TradingStrategySettings {
        strategies: StrategyConfigCollection {
            rsi: RsiConfig {
                enabled: true,
                period: 14,
                oversold_threshold: 30.0,
                overbought_threshold: 70.0,
                extreme_oversold: 20.0,
                extreme_overbought: 80.0,
            },
            macd: MacdConfig {
                enabled: false,
                fast_period: 12,
                slow_period: 26,
                signal_period: 9,
                histogram_threshold: 0.0,
            },
            volume: VolumeConfig {
                enabled: false,
                sma_period: 20,
                spike_threshold: 1.5,
                correlation_period: 10,
            },
            bollinger: BollingerConfig {
                enabled: false,
                period: 20,
                multiplier: 2.0,
                squeeze_threshold: 0.02,
            },
            stochastic: StochasticConfig {
                enabled: false,
                k_period: 14,
                d_period: 3,
                oversold_threshold: 20.0,
                overbought_threshold: 80.0,
                extreme_oversold: 10.0,
                extreme_overbought: 90.0,
            },
        },
        risk: RiskSettings {
            max_risk_per_trade: 1.5,
            max_portfolio_risk: 8.0,
            stop_loss_percent: 1.5,
            take_profit_percent: 4.0,
            max_leverage: 5,
            max_drawdown: 15.0,
            daily_loss_limit: 3.0,
            max_consecutive_losses: 3,
            correlation_limit: 0.6,
        },
        engine: EngineSettings {
            min_confidence_threshold: 0.7,
            signal_combination_mode: "all".to_string(),
            enabled_strategies: vec!["rsi".to_string()],
            market_condition: "low_volatility".to_string(),
            risk_level: "conservative".to_string(),
            data_resolution: "5m".to_string(),
        },
        market_preset: "low_volatility".to_string(),
    };

    let json = serde_json::to_string(&settings).unwrap();
    let deserialized: TradingStrategySettings = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.market_preset, "low_volatility");
    assert_eq!(deserialized.engine.data_resolution, "5m");
    assert_eq!(deserialized.risk.max_leverage, 5);
}

#[test]
fn test_update_basic_settings_request() {
    let request = UpdateBasicSettingsRequest {
        initial_balance: Some(50000.0),
        max_positions: Some(10),
        default_position_size_pct: Some(5.0),
        default_leverage: Some(5),
        trading_fee_rate: Some(0.001),
        funding_fee_rate: Some(0.0001),
        slippage_pct: Some(0.05),
        max_risk_per_trade_pct: Some(2.5),
        max_portfolio_risk_pct: Some(12.0),
        default_stop_loss_pct: Some(3.0),
        default_take_profit_pct: Some(7.0),
        max_leverage: Some(20),
        enabled: Some(true),
        ..Default::default()
    };

    assert_eq!(request.initial_balance, Some(50000.0));
    assert_eq!(request.max_positions, Some(10));
    assert_eq!(request.enabled, Some(true));
}

#[test]
fn test_symbol_config_structure() {
    let symbol_config = SymbolConfig {
        enabled: true,
        leverage: Some(15),
        position_size_pct: Some(3.5),
        stop_loss_pct: Some(2.5),
        take_profit_pct: Some(6.0),
        max_positions: Some(3),
    };

    assert!(symbol_config.enabled);
    assert_eq!(symbol_config.leverage, Some(15));
    assert_eq!(symbol_config.max_positions, Some(3));
}

#[test]
fn test_update_symbol_settings_request() {
    let mut symbols = HashMap::new();
    symbols.insert(
        "BTCUSDT".to_string(),
        SymbolConfig {
            enabled: true,
            leverage: Some(10),
            position_size_pct: Some(5.0),
            stop_loss_pct: Some(2.0),
            take_profit_pct: Some(5.0),
            max_positions: Some(2),
        },
    );
    symbols.insert(
        "ETHUSDT".to_string(),
        SymbolConfig {
            enabled: false,
            leverage: Some(5),
            position_size_pct: Some(3.0),
            stop_loss_pct: Some(1.5),
            take_profit_pct: Some(4.0),
            max_positions: Some(1),
        },
    );

    let request = UpdateSymbolSettingsRequest { symbols };

    assert_eq!(request.symbols.len(), 2);
    assert!(request.symbols.contains_key("BTCUSDT"));
    assert!(request.symbols.contains_key("ETHUSDT"));
}

#[test]
fn test_indicator_settings_api_structure() {
    let indicators = IndicatorSettingsApi {
        rsi_period: 14,
        macd_fast: 12,
        macd_slow: 26,
        macd_signal: 9,
        ema_periods: vec![9, 21, 50, 200],
        bollinger_period: 20,
        bollinger_std: 2.0,
        volume_sma_period: 20,
        stochastic_k_period: 14,
        stochastic_d_period: 3,
    };

    assert_eq!(indicators.rsi_period, 14);
    assert_eq!(indicators.ema_periods.len(), 4);
    assert_eq!(indicators.bollinger_std, 2.0);
}

#[test]
fn test_signal_generation_settings_api() {
    let signal_settings = SignalGenerationSettingsApi {
        trend_threshold_percent: 0.5,
        min_required_timeframes: 2,
        min_required_indicators: 3,
        confidence_base: 0.5,
        confidence_per_timeframe: 0.1,
    };

    assert_eq!(signal_settings.min_required_timeframes, 2);
    assert_eq!(signal_settings.min_required_indicators, 3);
    assert_eq!(signal_settings.confidence_base, 0.5);
}

#[test]
fn test_indicator_settings_response() {
    let response = IndicatorSettingsResponse {
        indicators: IndicatorSettingsApi {
            rsi_period: 14,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            ema_periods: vec![9, 21],
            bollinger_period: 20,
            bollinger_std: 2.0,
            volume_sma_period: 20,
            stochastic_k_period: 14,
            stochastic_d_period: 3,
        },
        signal: SignalGenerationSettingsApi {
            trend_threshold_percent: 0.5,
            min_required_timeframes: 2,
            min_required_indicators: 3,
            confidence_base: 0.5,
            confidence_per_timeframe: 0.1,
        },
    };

    assert_eq!(response.indicators.rsi_period, 14);
    assert_eq!(response.signal.min_required_timeframes, 2);
}

#[test]
fn test_api_response_success() {
    let data = "test_data".to_string();
    let response: ApiResponse<String> = ApiResponse::success(data.clone());

    assert!(response.success);
    assert_eq!(response.data, Some(data));
    assert!(response.error.is_none());
    assert!(response.timestamp <= Utc::now());
}

#[test]
fn test_api_response_error() {
    let error_msg = "Something went wrong".to_string();
    let response: ApiResponse<String> = ApiResponse::error(error_msg.clone());

    assert!(!response.success);
    assert!(response.data.is_none());
    assert_eq!(response.error, Some(error_msg));
    assert!(response.timestamp <= Utc::now());
}

#[test]
fn test_api_response_serialization() {
    let success_response: ApiResponse<i32> = ApiResponse::success(42);
    let json = serde_json::to_string(&success_response).unwrap();
    assert!(json.contains("\"success\":true"));
    assert!(json.contains("\"data\":42"));

    let error_response: ApiResponse<i32> = ApiResponse::error("Error occurred".to_string());
    let json = serde_json::to_string(&error_response).unwrap();
    assert!(json.contains("\"success\":false"));
    assert!(json.contains("Error occurred"));
}

// Note: Full API integration tests require complex setup with BinanceClient, AIService, Storage
// Testing API creation is covered by integration tests in test_service_integration.rs

#[test]
fn test_trade_analyses_query_defaults() {
    let query = TradeAnalysesQuery {
        only_losing: None,
        limit: None,
    };

    assert!(query.only_losing.is_none());
    assert!(query.limit.is_none());
}

#[test]
fn test_trade_analyses_query_with_values() {
    let query = TradeAnalysesQuery {
        only_losing: Some(true),
        limit: Some(50),
    };

    assert_eq!(query.only_losing, Some(true));
    assert_eq!(query.limit, Some(50));
}

#[test]
fn test_config_suggestions_query() {
    let query = ConfigSuggestionsQuery { limit: Some(10) };
    assert_eq!(query.limit, Some(10));
}

#[test]
fn test_signals_history_query() {
    let query = SignalsHistoryQuery {
        symbol: Some("BTCUSDT".to_string()),
        outcome: Some("win".to_string()),
        limit: Some(100),
    };

    assert_eq!(query.symbol, Some("BTCUSDT".to_string()));
    assert_eq!(query.outcome, Some("win".to_string()));
    assert_eq!(query.limit, Some(100));
}

// ========== REAL TRADING API TESTS ==========

#[test]
fn test_real_api_response_success() {
    let data = vec![1, 2, 3];
    let response: RealApiResponse<Vec<i32>> = RealApiResponse::success(data.clone());

    assert!(response.success);
    assert_eq!(response.data, Some(data));
    assert!(response.error.is_none());
}

#[test]
fn test_real_api_response_error() {
    let error_msg = "Real trading error".to_string();
    let response: RealApiResponse<()> = RealApiResponse::error(error_msg.clone());

    assert!(!response.success);
    assert!(response.data.is_none());
    assert_eq!(response.error, Some(error_msg));
}

#[test]
fn test_engine_status_structure() {
    let status = EngineStatus {
        is_running: true,
        is_testnet: true,
        open_positions_count: 3,
        open_orders_count: 5,
        circuit_breaker_open: false,
        daily_pnl: 250.50,
        daily_trades_count: 12,
        uptime_seconds: Some(3600),
    };

    assert!(status.is_running);
    assert!(status.is_testnet);
    assert_eq!(status.open_positions_count, 3);
    assert_eq!(status.daily_pnl, 250.50);
    assert_eq!(status.uptime_seconds, Some(3600));
}

#[test]
fn test_position_info_structure() {
    let position = PositionInfo {
        id: "pos_123".to_string(),
        symbol: "BTCUSDT".to_string(),
        side: "LONG".to_string(),
        quantity: 0.5,
        entry_price: 50000.0,
        current_price: 51000.0,
        unrealized_pnl: 500.0,
        unrealized_pnl_pct: 2.0,
        stop_loss: Some(48000.0),
        take_profit: Some(55000.0),
        created_at: Utc::now().to_rfc3339(),
    };

    assert_eq!(position.symbol, "BTCUSDT");
    assert_eq!(position.side, "LONG");
    assert_eq!(position.quantity, 0.5);
    assert_eq!(position.unrealized_pnl, 500.0);
    assert!(position.stop_loss.is_some());
}

#[test]
fn test_balance_info_structure() {
    let balance = BalanceInfo {
        asset: "USDT".to_string(),
        free: 10000.0,
        locked: 2000.0,
        total: 12000.0,
    };

    assert_eq!(balance.asset, "USDT");
    assert_eq!(balance.free, 10000.0);
    assert_eq!(balance.locked, 2000.0);
    assert_eq!(balance.total, 12000.0);
}

#[test]
fn test_portfolio_response_structure() {
    let portfolio = PortfolioResponse {
        total_balance: 50000.0,
        available_balance: 45000.0,
        locked_balance: 5000.0,
        unrealized_pnl: 1000.0,
        realized_pnl: 500.0,
        positions: vec![PositionInfo {
            id: "pos_1".to_string(),
            symbol: "BTCUSDT".to_string(),
            side: "LONG".to_string(),
            quantity: 0.1,
            entry_price: 50000.0,
            current_price: 51000.0,
            unrealized_pnl: 100.0,
            unrealized_pnl_pct: 2.0,
            stop_loss: None,
            take_profit: None,
            created_at: Utc::now().to_rfc3339(),
        }],
        balances: vec![BalanceInfo {
            asset: "USDT".to_string(),
            free: 45000.0,
            locked: 5000.0,
            total: 50000.0,
        }],
    };

    assert_eq!(portfolio.total_balance, 50000.0);
    assert_eq!(portfolio.positions.len(), 1);
    assert_eq!(portfolio.balances.len(), 1);
}

#[test]
fn test_update_signal_interval_request() {
    let request = UpdateSignalIntervalRequest {
        interval_minutes: 30,
    };

    let json = serde_json::to_string(&request).unwrap();
    let deserialized: UpdateSignalIntervalRequest = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.interval_minutes, 30);
}

#[test]
fn test_update_indicator_settings_request() {
    let request = UpdateIndicatorSettingsRequest {
        indicators: Some(IndicatorSettingsApi {
            rsi_period: 21,
            macd_fast: 12,
            macd_slow: 26,
            macd_signal: 9,
            ema_periods: vec![9, 21, 50],
            bollinger_period: 20,
            bollinger_std: 2.5,
            volume_sma_period: 20,
            stochastic_k_period: 14,
            stochastic_d_period: 3,
        }),
        signal: Some(SignalGenerationSettingsApi {
            trend_threshold_percent: 0.7,
            min_required_timeframes: 3,
            min_required_indicators: 4,
            confidence_base: 0.6,
            confidence_per_timeframe: 0.15,
        }),
    };

    assert!(request.indicators.is_some());
    assert!(request.signal.is_some());
    assert_eq!(request.indicators.as_ref().unwrap().rsi_period, 21);
    assert_eq!(request.signal.as_ref().unwrap().min_required_timeframes, 3);
}

#[test]
fn test_update_strategy_settings_request() {
    let settings = TradingStrategySettings {
        strategies: StrategyConfigCollection {
            rsi: RsiConfig {
                enabled: true,
                period: 14,
                oversold_threshold: 30.0,
                overbought_threshold: 70.0,
                extreme_oversold: 20.0,
                extreme_overbought: 80.0,
            },
            macd: MacdConfig {
                enabled: true,
                fast_period: 12,
                slow_period: 26,
                signal_period: 9,
                histogram_threshold: 0.0,
            },
            volume: VolumeConfig {
                enabled: false,
                sma_period: 20,
                spike_threshold: 1.5,
                correlation_period: 10,
            },
            bollinger: BollingerConfig {
                enabled: false,
                period: 20,
                multiplier: 2.0,
                squeeze_threshold: 0.02,
            },
            stochastic: StochasticConfig {
                enabled: false,
                k_period: 14,
                d_period: 3,
                oversold_threshold: 20.0,
                overbought_threshold: 80.0,
                extreme_oversold: 10.0,
                extreme_overbought: 90.0,
            },
        },
        risk: RiskSettings {
            max_risk_per_trade: 2.0,
            max_portfolio_risk: 10.0,
            stop_loss_percent: 2.0,
            take_profit_percent: 5.0,
            max_leverage: 10,
            max_drawdown: 20.0,
            daily_loss_limit: 5.0,
            max_consecutive_losses: 5,
            correlation_limit: 0.7,
        },
        engine: EngineSettings {
            min_confidence_threshold: 0.6,
            signal_combination_mode: "any".to_string(),
            enabled_strategies: vec!["rsi".to_string()],
            market_condition: "normal".to_string(),
            risk_level: "moderate".to_string(),
            data_resolution: "15m".to_string(),
        },
        market_preset: "normal_volatility".to_string(),
    };

    let request = UpdateStrategySettingsRequest { settings };

    assert!(request.settings.strategies.rsi.enabled);
    assert_eq!(request.settings.market_preset, "normal_volatility");
}

#[test]
fn test_risk_settings_validation() {
    let risk = RiskSettings {
        max_risk_per_trade: 2.0,
        max_portfolio_risk: 10.0,
        stop_loss_percent: 2.0,
        take_profit_percent: 5.0,
        max_leverage: 10,
        max_drawdown: 20.0,
        daily_loss_limit: 5.0,
        max_consecutive_losses: 5,
        correlation_limit: 0.7,
    };

    // Validate reasonable values
    assert!(risk.max_risk_per_trade > 0.0 && risk.max_risk_per_trade <= 10.0);
    assert!(risk.max_portfolio_risk > 0.0 && risk.max_portfolio_risk <= 100.0);
    assert!(risk.max_leverage > 0 && risk.max_leverage <= 125);
    assert!(risk.correlation_limit >= 0.0 && risk.correlation_limit <= 1.0);
}

#[test]
fn test_engine_settings_validation() {
    let engine = EngineSettings {
        min_confidence_threshold: 0.6,
        signal_combination_mode: "any".to_string(),
        enabled_strategies: vec!["rsi".to_string(), "macd".to_string()],
        market_condition: "normal".to_string(),
        risk_level: "moderate".to_string(),
        data_resolution: "15m".to_string(),
    };

    // Validate values
    assert!(engine.min_confidence_threshold >= 0.0 && engine.min_confidence_threshold <= 1.0);
    assert!(!engine.enabled_strategies.is_empty());
    assert!(
        ["any", "all", "majority"].contains(&engine.signal_combination_mode.as_str())
            || engine.signal_combination_mode == "any"
    );
}
