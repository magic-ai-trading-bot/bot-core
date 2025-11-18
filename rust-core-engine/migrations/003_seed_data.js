// @spec:FR-DB-005 - Rust Service Seed Data Migration
// @ref:specs/02-design/2.2-database/DB-SCHEMA.md
// Migration 003: Seed data for Rust Core Engine
// Created: 2025-11-18
// Purpose: Insert initial configuration and test data

print('========================================');
print('Migration 003: Seed Data');
print('Rust Core Engine - Initial Data');
print('========================================\n');

// Connect to bot_core database
db = db.getSiblingDB('bot_core');

// Helper function to insert if not exists
function insertIfNotExists(collection, query, document, description) {
    try {
        const coll = db.getCollection(collection);
        const exists = coll.findOne(query);

        if (exists) {
            print('  Data already exists: ' + description);
            return false;
        }

        coll.insertOne(document);
        print('✓ Inserted: ' + description);
        return true;
    } catch (e) {
        print('✗ Failed to insert: ' + description);
        print('  Error: ' + e.message);
        return false;
    }
}

// 1. System Configuration
print('[1/6] Inserting system configuration...');

insertIfNotExists('system_config',
    { _id: 'global_config' },
    {
        _id: 'global_config',
        trading_enabled: false,
        maintenance_mode: false,
        max_concurrent_trades: 100,
        max_positions_per_user: 10,
        default_risk_level: 'Medium',
        api_version: 'v1.0.0',
        features: {
            paper_trading: true,
            live_trading: false,
            ai_signals: true,
            websocket_enabled: true,
            rate_limiting: true
        },
        created_at: new Date(),
        updated_at: new Date()
    },
    'Global system configuration'
);

insertIfNotExists('system_config',
    { _id: 'rate_limits' },
    {
        _id: 'rate_limits',
        api_calls_per_minute: 60,
        api_calls_per_hour: 1000,
        trades_per_minute: 10,
        trades_per_hour: 100,
        websocket_connections_per_user: 5,
        max_concurrent_orders: 20,
        created_at: new Date(),
        updated_at: new Date()
    },
    'Rate limit configuration'
);

insertIfNotExists('system_config',
    { _id: 'risk_limits' },
    {
        _id: 'risk_limits',
        max_position_size_usd: 10000,
        max_daily_loss_usd: 1000,
        max_leverage: 5,
        default_stop_loss_percentage: 2.0,
        default_take_profit_percentage: 3.0,
        max_portfolio_risk_percentage: 10.0,
        created_at: new Date(),
        updated_at: new Date()
    },
    'Risk limit configuration'
);

insertIfNotExists('system_config',
    { _id: 'trading_pairs' },
    {
        _id: 'trading_pairs',
        enabled_pairs: [
            'BTCUSDT',
            'ETHUSDT',
            'BNBUSDT',
            'ADAUSDT',
            'DOGEUSDT',
            'XRPUSDT',
            'DOTUSDT',
            'UNIUSDT',
            'SOLUSDT',
            'MATICUSDT'
        ],
        default_pair: 'BTCUSDT',
        min_trade_amounts: {
            'BTCUSDT': 0.0001,
            'ETHUSDT': 0.001,
            'BNBUSDT': 0.01,
            'ADAUSDT': 1.0,
            'DOGEUSDT': 10.0
        },
        created_at: new Date(),
        updated_at: new Date()
    },
    'Trading pairs configuration'
);

print('');

// 2. Default Strategy Configurations
print('[2/6] Inserting default strategy templates...');

const defaultStrategies = [
    {
        _id: 'strategy_template_rsi',
        strategy_name: 'RSI',
        description: 'Relative Strength Index strategy',
        default_parameters: {
            rsi_period: 14,
            oversold_threshold: 30,
            overbought_threshold: 70,
            timeframe: '15m'
        },
        risk_settings: {
            max_position_size: 0.1,
            stop_loss_percentage: 2.0,
            take_profit_percentage: 3.0
        },
        is_template: true,
        created_at: new Date()
    },
    {
        _id: 'strategy_template_macd',
        strategy_name: 'MACD',
        description: 'Moving Average Convergence Divergence strategy',
        default_parameters: {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
            timeframe: '1h'
        },
        risk_settings: {
            max_position_size: 0.1,
            stop_loss_percentage: 2.5,
            take_profit_percentage: 4.0
        },
        is_template: true,
        created_at: new Date()
    },
    {
        _id: 'strategy_template_bollinger',
        strategy_name: 'BOLLINGER_BANDS',
        description: 'Bollinger Bands breakout strategy',
        default_parameters: {
            period: 20,
            std_dev: 2,
            timeframe: '30m'
        },
        risk_settings: {
            max_position_size: 0.15,
            stop_loss_percentage: 1.5,
            take_profit_percentage: 2.5
        },
        is_template: true,
        created_at: new Date()
    },
    {
        _id: 'strategy_template_volume',
        strategy_name: 'VOLUME_ANALYSIS',
        description: 'Volume-based trading strategy',
        default_parameters: {
            volume_threshold: 1.5,
            lookback_period: 20,
            timeframe: '5m'
        },
        risk_settings: {
            max_position_size: 0.08,
            stop_loss_percentage: 3.0,
            take_profit_percentage: 5.0
        },
        is_template: true,
        created_at: new Date()
    }
];

defaultStrategies.forEach(function(strategy) {
    insertIfNotExists('system_config',
        { _id: strategy._id },
        strategy,
        'Strategy template: ' + strategy.strategy_name
    );
});

print('');

// 3. Default Notification Templates
print('[3/6] Inserting notification templates...');

const notificationTemplates = [
    {
        _id: 'notification_template_trade_executed',
        notification_type: 'TRADE_EXECUTED',
        template: 'Trade executed: {side} {quantity} {symbol} at {price}',
        severity: 'INFO'
    },
    {
        _id: 'notification_template_position_closed',
        notification_type: 'POSITION_CLOSED',
        template: 'Position closed: {symbol} | P&L: {pnl} ({pnl_percentage}%)',
        severity: 'INFO'
    },
    {
        _id: 'notification_template_risk_alert',
        notification_type: 'RISK_ALERT',
        template: 'Risk alert: {metric} exceeded threshold ({value})',
        severity: 'WARNING'
    },
    {
        _id: 'notification_template_system',
        notification_type: 'SYSTEM',
        template: 'System notification: {message}',
        severity: 'INFO'
    },
    {
        _id: 'notification_template_error',
        notification_type: 'ERROR',
        template: 'Error: {error_message}',
        severity: 'ERROR'
    }
];

notificationTemplates.forEach(function(template) {
    insertIfNotExists('system_config',
        { _id: template._id },
        template,
        'Notification template: ' + template.notification_type
    );
});

print('');

// 4. Default Risk Thresholds
print('[4/6] Inserting risk threshold configurations...');

insertIfNotExists('system_config',
    { _id: 'risk_thresholds' },
    {
        _id: 'risk_thresholds',
        max_drawdown_warning: 10.0,      // 10% warning
        max_drawdown_critical: 20.0,     // 20% critical
        var_95_threshold: 5.0,            // 5% Value at Risk
        sharpe_ratio_min: 0.5,            // Minimum acceptable Sharpe ratio
        volatility_max: 50.0,             // 50% max volatility
        correlation_warning: 0.8,         // High correlation warning
        created_at: new Date(),
        updated_at: new Date()
    },
    'Risk threshold configuration'
);

print('');

// 5. Feature Flags
print('[5/6] Inserting feature flags...');

insertIfNotExists('system_config',
    { _id: 'feature_flags' },
    {
        _id: 'feature_flags',
        paper_trading_enabled: true,
        live_trading_enabled: false,
        ai_predictions_enabled: true,
        automated_trading_enabled: false,
        advanced_charting_enabled: true,
        social_trading_enabled: false,
        copy_trading_enabled: false,
        margin_trading_enabled: false,
        futures_trading_enabled: false,
        options_trading_enabled: false,
        created_at: new Date(),
        updated_at: new Date()
    },
    'Feature flags configuration'
);

print('');

// 6. Exchange Configuration
print('[6/6] Inserting exchange configuration...');

insertIfNotExists('system_config',
    { _id: 'exchange_config' },
    {
        _id: 'exchange_config',
        default_exchange: 'BINANCE',
        testnet_enabled: true,
        exchanges: {
            binance: {
                enabled: true,
                testnet_url: 'https://testnet.binance.vision/api',
                production_url: 'https://api.binance.com/api',
                websocket_testnet: 'wss://testnet.binance.vision/ws',
                websocket_production: 'wss://stream.binance.com:9443/ws',
                rate_limits: {
                    requests_per_minute: 1200,
                    orders_per_second: 10,
                    orders_per_day: 200000
                }
            }
        },
        created_at: new Date(),
        updated_at: new Date()
    },
    'Exchange configuration'
);

print('');

// Verify seed data
print('========================================');
print('Verifying Seed Data...');
print('========================================\n');

const configCount = db.system_config.countDocuments();
print('Total system configurations: ' + configCount);

print('\nConfiguration items:');
db.system_config.find({}, { _id: 1 }).forEach(function(doc) {
    print('  - ' + doc._id);
});

print('\n========================================');
print('Migration 003 Completed Successfully!');
print('Seed data inserted');
print('========================================\n');
