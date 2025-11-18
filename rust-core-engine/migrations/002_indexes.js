// @spec:FR-DB-004 - Rust Service Performance Indexes Migration
// @ref:specs/02-design/2.2-database/DB-SCHEMA.md
// Migration 002: Performance indexes for Rust Core Engine
// Created: 2025-11-18
// Purpose: Create optimized indexes for high-performance trading operations

print('========================================');
print('Migration 002: Performance Indexes');
print('Rust Core Engine Collections');
print('========================================\n');

// Connect to bot_core database
db = db.getSiblingDB('bot_core');

// Helper function to create index safely
function createIndexSafely(collection, indexSpec, options, description) {
    try {
        const collectionName = typeof collection === 'string' ? collection : collection.getName();
        const coll = db.getCollection(collectionName);

        coll.createIndex(indexSpec, options || {});
        print('✓ Created index: ' + description);
        return true;
    } catch (e) {
        if (e.code === 85 || e.message.includes('already exists')) {
            print('  Index already exists: ' + description);
            return true;
        }
        print('✗ Failed to create index: ' + description);
        print('  Error: ' + e.message);
        return false;
    }
}

// 1. Positions Collection Indexes
print('[1/9] Creating positions indexes...');

createIndexSafely('positions',
    { user_id: 1, symbol: 1 },
    { unique: true, name: 'idx_positions_user_symbol' },
    'positions: user_id + symbol (unique)'
);

createIndexSafely('positions',
    { is_open: 1, user_id: 1 },
    { name: 'idx_positions_open_user' },
    'positions: is_open + user_id'
);

createIndexSafely('positions',
    { user_id: 1, created_at: -1 },
    { name: 'idx_positions_user_created' },
    'positions: user_id + created_at'
);

createIndexSafely('positions',
    { symbol: 1, is_open: 1, created_at: -1 },
    { name: 'idx_positions_symbol_status' },
    'positions: symbol + is_open + created_at'
);

createIndexSafely('positions',
    { unrealized_pnl: -1 },
    { sparse: true, name: 'idx_positions_unrealized_pnl' },
    'positions: unrealized_pnl (sparse)'
);

createIndexSafely('positions',
    { closed_at: 1 },
    { sparse: true, expireAfterSeconds: 7776000, name: 'idx_positions_closed_ttl' },
    'positions: closed_at (TTL 90 days)'
);

print('');

// 2. Risk Metrics Collection Indexes
print('[2/9] Creating risk_metrics indexes...');

createIndexSafely('risk_metrics',
    { user_id: 1, metric_type: 1, created_at: -1 },
    { name: 'idx_risk_user_type_date' },
    'risk_metrics: user_id + metric_type + created_at'
);

createIndexSafely('risk_metrics',
    { metric_type: 1, timeframe: 1, created_at: -1 },
    { name: 'idx_risk_type_timeframe' },
    'risk_metrics: metric_type + timeframe + created_at'
);

createIndexSafely('risk_metrics',
    { created_at: 1 },
    { expireAfterSeconds: 15552000, name: 'idx_risk_ttl' },
    'risk_metrics: created_at (TTL 180 days)'
);

print('');

// 3. Strategy Configs Collection Indexes
print('[3/9] Creating strategy_configs indexes...');

createIndexSafely('strategy_configs',
    { user_id: 1, strategy_name: 1 },
    { name: 'idx_strategy_user_name' },
    'strategy_configs: user_id + strategy_name'
);

createIndexSafely('strategy_configs',
    { is_active: 1, user_id: 1 },
    { name: 'idx_strategy_active_user' },
    'strategy_configs: is_active + user_id'
);

createIndexSafely('strategy_configs',
    { 'symbols': 1 },
    { name: 'idx_strategy_symbols' },
    'strategy_configs: symbols (array)'
);

createIndexSafely('strategy_configs',
    { updated_at: -1 },
    { name: 'idx_strategy_updated' },
    'strategy_configs: updated_at'
);

print('');

// 4. Sessions Collection Indexes
print('[4/9] Creating sessions indexes...');

createIndexSafely('sessions',
    { session_token: 1 },
    { unique: true, name: 'idx_sessions_token' },
    'sessions: session_token (unique)'
);

createIndexSafely('sessions',
    { user_id: 1, created_at: -1 },
    { name: 'idx_sessions_user_created' },
    'sessions: user_id + created_at'
);

createIndexSafely('sessions',
    { expires_at: 1 },
    { expireAfterSeconds: 0, name: 'idx_sessions_expires' },
    'sessions: expires_at (TTL)'
);

createIndexSafely('sessions',
    { last_activity: -1 },
    { name: 'idx_sessions_activity' },
    'sessions: last_activity'
);

print('');

// 5. API Keys Collection Indexes
print('[5/9] Creating api_keys indexes...');

createIndexSafely('api_keys',
    { key_hash: 1 },
    { unique: true, name: 'idx_apikeys_hash' },
    'api_keys: key_hash (unique)'
);

createIndexSafely('api_keys',
    { user_id: 1, exchange: 1 },
    { name: 'idx_apikeys_user_exchange' },
    'api_keys: user_id + exchange'
);

createIndexSafely('api_keys',
    { is_active: 1, user_id: 1 },
    { name: 'idx_apikeys_active' },
    'api_keys: is_active + user_id'
);

createIndexSafely('api_keys',
    { expires_at: 1 },
    { sparse: true, name: 'idx_apikeys_expires' },
    'api_keys: expires_at (sparse)'
);

print('');

// 6. Notifications Collection Indexes
print('[6/9] Creating notifications indexes...');

createIndexSafely('notifications',
    { user_id: 1, created_at: -1 },
    { name: 'idx_notifications_user_date' },
    'notifications: user_id + created_at'
);

createIndexSafely('notifications',
    { is_read: 1, user_id: 1, created_at: -1 },
    { name: 'idx_notifications_read_status' },
    'notifications: is_read + user_id + created_at'
);

createIndexSafely('notifications',
    { notification_type: 1, severity: 1, created_at: -1 },
    { name: 'idx_notifications_type_severity' },
    'notifications: notification_type + severity + created_at'
);

createIndexSafely('notifications',
    { created_at: 1 },
    { expireAfterSeconds: 2592000, name: 'idx_notifications_ttl' },
    'notifications: created_at (TTL 30 days)'
);

print('');

// 7. Performance Metrics Collection Indexes (for aggregated data)
print('[7/9] Creating performance_metrics indexes...');

createIndexSafely('performance_metrics',
    { user_id: 1, date: -1 },
    { name: 'idx_performance_user_date' },
    'performance_metrics: user_id + date'
);

createIndexSafely('performance_metrics',
    { date: -1, metric_type: 1 },
    { name: 'idx_performance_date_type' },
    'performance_metrics: date + metric_type'
);

createIndexSafely('performance_metrics',
    { date: 1 },
    { expireAfterSeconds: 31536000, name: 'idx_performance_ttl' },
    'performance_metrics: date (TTL 365 days)'
);

print('');

// 8. System Config Collection Indexes
print('[8/9] Creating system_config indexes...');

createIndexSafely('system_config',
    { _id: 1 },
    { name: 'idx_sysconfig_id' },
    'system_config: _id'
);

createIndexSafely('system_config',
    { updated_at: -1 },
    { name: 'idx_sysconfig_updated' },
    'system_config: updated_at'
);

print('');

// 9. Compound Indexes for Complex Queries
print('[9/9] Creating compound indexes for complex queries...');

// Trading activity query optimization
createIndexSafely('trades',
    { user_id: 1, status: 1, created_at: -1 },
    { name: 'idx_trades_user_status_date' },
    'trades: user_id + status + created_at (compound)'
);

// Portfolio performance queries
createIndexSafely('portfolio_snapshots',
    { user_id: 1, created_at: -1 },
    { name: 'idx_portfolio_user_date_compound' },
    'portfolio_snapshots: user_id + created_at (compound)'
);

// Position monitoring queries
createIndexSafely('positions',
    { is_open: 1, symbol: 1, user_id: 1 },
    { name: 'idx_positions_monitoring' },
    'positions: is_open + symbol + user_id (monitoring)'
);

print('');

// List all indexes created
print('========================================');
print('Verifying Indexes...');
print('========================================\n');

const collections = [
    'positions', 'risk_metrics', 'strategy_configs',
    'sessions', 'api_keys', 'notifications',
    'performance_metrics', 'system_config', 'trades',
    'portfolio_snapshots'
];

collections.forEach(function(collName) {
    if (db.getCollectionNames().includes(collName)) {
        const indexes = db.getCollection(collName).getIndexes();
        print(collName + ': ' + indexes.length + ' indexes');
    }
});

print('\n========================================');
print('Migration 002 Completed Successfully!');
print('Performance indexes created');
print('========================================\n');
