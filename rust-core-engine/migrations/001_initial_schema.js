// @spec:FR-DB-003 - Rust Service Initial Schema Migration
// @ref:specs/02-design/2.2-database/DB-SCHEMA.md
// Migration 001: Initial schema for Rust Core Engine collections
// Created: 2025-11-18
// Purpose: Create core trading collections (users, trades, positions, portfolios)

print('========================================');
print('Migration 001: Initial Schema Setup');
print('Rust Core Engine Collections');
print('========================================\n');

// Connect to bot_core database
db = db.getSiblingDB('bot_core');

// 1. Users Collection - Already created in mongo-init.js, verify it exists
print('[1/10] Verifying users collection...');
if (db.getCollectionNames().includes('users')) {
    print('✓ Users collection exists');
} else {
    print('✗ Users collection not found - creating...');
    db.createCollection('users', {
        validator: {
            $jsonSchema: {
                bsonType: 'object',
                required: ['email', 'password_hash', 'created_at'],
                properties: {
                    email: {
                        bsonType: 'string',
                        pattern: '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$'
                    },
                    password_hash: {
                        bsonType: 'string',
                        minLength: 60
                    }
                }
            }
        }
    });
}

// 2. Trades Collection - Verify and update if needed
print('[2/10] Verifying trades collection...');
if (db.getCollectionNames().includes('trades')) {
    print('✓ Trades collection exists');
} else {
    print('✗ Trades collection not found - creating...');
    db.createCollection('trades');
}

// 3. Positions Collection - Active trading positions
print('[3/10] Updating positions collection schema...');
db.runCommand({
    collMod: 'positions',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'symbol', 'side', 'quantity', 'entry_price', 'is_open', 'created_at'],
            properties: {
                user_id: {
                    bsonType: 'objectId',
                    description: 'Reference to users collection'
                },
                symbol: {
                    bsonType: 'string',
                    pattern: '^[A-Z]{6,12}$',
                    description: 'Trading pair (e.g., BTCUSDT)'
                },
                side: {
                    enum: ['LONG', 'SHORT'],
                    description: 'Position side'
                },
                quantity: {
                    bsonType: 'decimal',
                    description: 'Position size'
                },
                entry_price: {
                    bsonType: 'decimal',
                    description: 'Average entry price'
                },
                current_price: {
                    bsonType: ['decimal', 'null'],
                    description: 'Current market price'
                },
                unrealized_pnl: {
                    bsonType: ['decimal', 'null'],
                    description: 'Unrealized profit/loss'
                },
                realized_pnl: {
                    bsonType: ['decimal', 'null'],
                    description: 'Realized profit/loss'
                },
                is_open: {
                    bsonType: 'bool',
                    description: 'Position status'
                },
                stop_loss: {
                    bsonType: ['decimal', 'null'],
                    description: 'Stop loss price'
                },
                take_profit: {
                    bsonType: ['decimal', 'null'],
                    description: 'Take profit price'
                },
                created_at: {
                    bsonType: 'date'
                },
                updated_at: {
                    bsonType: 'date'
                },
                closed_at: {
                    bsonType: ['date', 'null']
                }
            }
        }
    },
    validationLevel: 'moderate'
});
print('✓ Positions collection schema updated');

// 4. Portfolio Snapshots - Historical portfolio states
print('[4/10] Updating portfolio_snapshots collection...');
db.runCommand({
    collMod: 'portfolio_snapshots',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'total_value', 'created_at'],
            properties: {
                user_id: {
                    bsonType: 'objectId'
                },
                total_value: {
                    bsonType: 'decimal',
                    description: 'Total portfolio value in USDT'
                },
                available_balance: {
                    bsonType: 'decimal',
                    description: 'Available cash balance'
                },
                positions_value: {
                    bsonType: 'decimal',
                    description: 'Total value of open positions'
                },
                total_pnl: {
                    bsonType: ['decimal', 'null'],
                    description: 'Total profit/loss'
                },
                pnl_percentage: {
                    bsonType: ['double', 'null'],
                    description: 'P&L percentage'
                },
                positions: {
                    bsonType: 'array',
                    description: 'Array of position snapshots'
                },
                created_at: {
                    bsonType: 'date'
                }
            }
        }
    }
});
print('✓ Portfolio snapshots collection updated');

// 5. Risk Metrics Collection
print('[5/10] Updating risk_metrics collection...');
db.runCommand({
    collMod: 'risk_metrics',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'metric_type', 'value', 'created_at'],
            properties: {
                user_id: {
                    bsonType: 'objectId'
                },
                metric_type: {
                    enum: ['VAR', 'SHARPE_RATIO', 'MAX_DRAWDOWN', 'VOLATILITY', 'BETA'],
                    description: 'Type of risk metric'
                },
                value: {
                    bsonType: 'double',
                    description: 'Metric value'
                },
                timeframe: {
                    enum: ['1D', '7D', '30D', '90D', '1Y'],
                    description: 'Calculation timeframe'
                },
                metadata: {
                    bsonType: 'object',
                    description: 'Additional metric metadata'
                },
                created_at: {
                    bsonType: 'date'
                }
            }
        }
    }
});
print('✓ Risk metrics collection updated');

// 6. Strategy Configurations
print('[6/10] Updating strategy_configs collection...');
db.runCommand({
    collMod: 'strategy_configs',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'strategy_name', 'is_active'],
            properties: {
                user_id: {
                    bsonType: 'objectId'
                },
                strategy_name: {
                    enum: ['RSI', 'MACD', 'BOLLINGER_BANDS', 'VOLUME_ANALYSIS', 'CUSTOM'],
                    description: 'Strategy type'
                },
                is_active: {
                    bsonType: 'bool',
                    description: 'Strategy enabled status'
                },
                parameters: {
                    bsonType: 'object',
                    description: 'Strategy-specific parameters'
                },
                symbols: {
                    bsonType: 'array',
                    items: {
                        bsonType: 'string'
                    },
                    description: 'Symbols this strategy applies to'
                },
                risk_settings: {
                    bsonType: 'object',
                    properties: {
                        max_position_size: { bsonType: 'double' },
                        stop_loss_percentage: { bsonType: 'double' },
                        take_profit_percentage: { bsonType: 'double' }
                    }
                },
                created_at: {
                    bsonType: 'date'
                },
                updated_at: {
                    bsonType: 'date'
                }
            }
        }
    }
});
print('✓ Strategy configs collection updated');

// 7. Sessions Collection
print('[7/10] Updating sessions collection...');
db.runCommand({
    collMod: 'sessions',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'session_token', 'created_at', 'expires_at'],
            properties: {
                user_id: {
                    bsonType: 'objectId'
                },
                session_token: {
                    bsonType: 'string',
                    minLength: 32
                },
                ip_address: {
                    bsonType: ['string', 'null']
                },
                user_agent: {
                    bsonType: ['string', 'null']
                },
                created_at: {
                    bsonType: 'date'
                },
                expires_at: {
                    bsonType: 'date'
                },
                last_activity: {
                    bsonType: 'date'
                }
            }
        }
    }
});
print('✓ Sessions collection updated');

// 8. API Keys Collection
print('[8/10] Updating api_keys collection...');
db.runCommand({
    collMod: 'api_keys',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'exchange', 'key_hash', 'created_at'],
            properties: {
                user_id: {
                    bsonType: 'objectId'
                },
                exchange: {
                    enum: ['BINANCE', 'BINANCE_TESTNET'],
                    description: 'Exchange identifier'
                },
                key_hash: {
                    bsonType: 'string',
                    description: 'Hashed API key (never store plaintext)'
                },
                encrypted_secret: {
                    bsonType: 'string',
                    description: 'Encrypted API secret'
                },
                permissions: {
                    bsonType: 'array',
                    items: {
                        enum: ['TRADING', 'READING', 'WITHDRAWALS']
                    }
                },
                is_active: {
                    bsonType: 'bool'
                },
                last_used: {
                    bsonType: ['date', 'null']
                },
                created_at: {
                    bsonType: 'date'
                },
                expires_at: {
                    bsonType: ['date', 'null']
                }
            }
        }
    }
});
print('✓ API keys collection updated');

// 9. Audit Logs Collection - Already capped in mongo-init.js
print('[9/10] Verifying audit_logs collection...');
if (db.getCollectionNames().includes('audit_logs')) {
    print('✓ Audit logs collection exists (capped)');
}

// 10. Notifications Collection
print('[10/10] Updating notifications collection...');
db.runCommand({
    collMod: 'notifications',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['user_id', 'notification_type', 'message', 'created_at'],
            properties: {
                user_id: {
                    bsonType: 'objectId'
                },
                notification_type: {
                    enum: ['TRADE_EXECUTED', 'POSITION_CLOSED', 'RISK_ALERT', 'SYSTEM', 'ERROR'],
                    description: 'Type of notification'
                },
                severity: {
                    enum: ['INFO', 'WARNING', 'ERROR', 'CRITICAL'],
                    description: 'Notification severity'
                },
                message: {
                    bsonType: 'string',
                    minLength: 1,
                    maxLength: 500
                },
                metadata: {
                    bsonType: 'object',
                    description: 'Additional notification data'
                },
                is_read: {
                    bsonType: 'bool'
                },
                read_at: {
                    bsonType: ['date', 'null']
                },
                created_at: {
                    bsonType: 'date'
                }
            }
        }
    }
});
print('✓ Notifications collection updated');

print('\n========================================');
print('Migration 001 Completed Successfully!');
print('Collections updated: 10');
print('========================================\n');
