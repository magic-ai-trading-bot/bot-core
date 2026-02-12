// @spec:FR-DB-001 - MongoDB Initialization Script
// @ref:specs/02-design/2.2-database/DB-SCHEMA.md
// MongoDB Database Initialization Script for Bot Core
// This script runs on first container startup to initialize the database

print('==================================================');
print('Bot Core - MongoDB Initialization Starting...');
print('==================================================');

// Get environment variables (set via Docker environment)
const mongoUser = process.env.MONGO_ROOT_USER || 'admin';
const mongoPassword = process.env.MONGO_ROOT_PASSWORD || 'secure_mongo_password_change_me';

// Switch to the bot_core database
db = db.getSiblingDB('bot_core');

print('\n[1/5] Creating bot_core database...');

// Create application users with specific roles
print('\n[2/5] Creating database users...');

db.createUser({
  user: 'bot_core_admin',
  pwd: mongoPassword,
  roles: [
    { role: 'dbOwner', db: 'bot_core' },
    { role: 'readWrite', db: 'bot_core' }
  ]
});
print('✓ Created bot_core_admin user');

db.createUser({
  user: 'bot_core_app',
  pwd: mongoPassword,
  roles: [
    { role: 'readWrite', db: 'bot_core' }
  ]
});
print('✓ Created bot_core_app user (read/write)');

db.createUser({
  user: 'bot_core_readonly',
  pwd: mongoPassword,
  roles: [
    { role: 'read', db: 'bot_core' }
  ]
});
print('✓ Created bot_core_readonly user');

// Create collections with validation rules
print('\n[3/5] Creating collections with schema validation...');

// Users collection
db.createCollection('users', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['email', 'password_hash', 'created_at'],
      properties: {
        email: {
          bsonType: 'string',
          pattern: '^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$',
          description: 'must be a valid email address'
        },
        password_hash: {
          bsonType: 'string',
          minLength: 60,
          description: 'must be a bcrypt hash (60+ chars)'
        },
        is_active: {
          bsonType: 'bool',
          description: 'account active status'
        },
        is_admin: {
          bsonType: 'bool',
          description: 'admin privileges flag'
        }
      }
    }
  }
});
print('✓ Created users collection');

// Trades collection
db.createCollection('trades', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['user_id', 'symbol', 'side', 'order_type', 'quantity', 'status', 'created_at'],
      properties: {
        symbol: {
          bsonType: 'string',
          pattern: '^[A-Z]{6,12}$',
          description: 'trading pair symbol (e.g., BTCUSDT)'
        },
        side: {
          enum: ['BUY', 'SELL'],
          description: 'order side'
        },
        order_type: {
          enum: ['MARKET', 'LIMIT', 'STOP_LOSS', 'TAKE_PROFIT'],
          description: 'order type'
        },
        status: {
          enum: ['PENDING', 'FILLED', 'PARTIALLY_FILLED', 'CANCELLED', 'FAILED'],
          description: 'order status'
        }
      }
    }
  }
});
print('✓ Created trades collection');

// Positions collection
db.createCollection('positions');
print('✓ Created positions collection');

// Market data collection (time-series optimized)
db.createCollection('market_data', {
  timeseries: {
    timeField: 'timestamp',
    metaField: 'symbol',
    granularity: 'minutes'
  },
  expireAfterSeconds: 31536000  // 1 year TTL
});
print('✓ Created market_data time-series collection');

// AI analysis results
db.createCollection('ai_analysis_results');
print('✓ Created ai_analysis_results collection');

// Portfolio snapshots
db.createCollection('portfolio_snapshots');
print('✓ Created portfolio_snapshots collection');

// Risk metrics
db.createCollection('risk_metrics');
print('✓ Created risk_metrics collection');

// Strategy configs
db.createCollection('strategy_configs');
print('✓ Created strategy_configs collection');

// Paper trading collections
db.createCollection('paper_trading_accounts');
print('✓ Created paper_trading_accounts collection');

db.createCollection('paper_trading_trades');
print('✓ Created paper_trading_trades collection');

db.createCollection('paper_trading_settings');
print('✓ Created paper_trading_settings collection');

db.createCollection('portfolio_history');
print('✓ Created portfolio_history collection');

// AI signals
db.createCollection('ai_signals');
print('✓ Created ai_signals collection');

// Performance metrics
db.createCollection('performance_metrics');
print('✓ Created performance_metrics collection');

// Audit logs
db.createCollection('audit_logs', {
  capped: true,
  size: 104857600,  // 100MB
  max: 1000000      // 1 million documents
});
print('✓ Created audit_logs collection (capped)');

// Sessions (optional, for session management)
db.createCollection('sessions');
print('✓ Created sessions collection');

// Notifications
db.createCollection('notifications');
print('✓ Created notifications collection');

// System config
db.createCollection('system_config');
print('✓ Created system_config collection');

// API keys
db.createCollection('api_keys');
print('✓ Created api_keys collection');

// ML Models metadata (for Python service)
db.createCollection('ml_models');
print('✓ Created ml_models collection');

// Predictions
db.createCollection('predictions');
print('✓ Created predictions collection');

// Training jobs
db.createCollection('training_jobs');
print('✓ Created training_jobs collection');

// Market indicators
db.createCollection('market_indicators');
print('✓ Created market_indicators collection');

// Create indexes for performance
print('\n[4/5] Creating performance indexes...');

// Users indexes
db.users.createIndex({ email: 1 }, { unique: true });
db.users.createIndex({ created_at: -1 });
db.users.createIndex({ is_active: 1 });
print('✓ Created users indexes');

// Trades indexes
db.trades.createIndex({ user_id: 1, created_at: -1 });
db.trades.createIndex({ symbol: 1, created_at: -1 });
db.trades.createIndex({ status: 1 });
db.trades.createIndex({ binance_order_id: 1 }, { sparse: true });
db.trades.createIndex({ ai_signal_id: 1 }, { sparse: true });
db.trades.createIndex({ created_at: 1 }, { expireAfterSeconds: 31536000 }); // 1 year TTL
print('✓ Created trades indexes');

// Positions indexes
db.positions.createIndex({ user_id: 1, symbol: 1 }, { unique: true });
db.positions.createIndex({ is_open: 1 });
db.positions.createIndex({ created_at: -1 });
print('✓ Created positions indexes');

// Market data indexes (handled by time-series collection)
db.market_data.createIndex({ symbol: 1, timestamp: -1 });
db.market_data.createIndex({ interval: 1, timestamp: -1 });
print('✓ Created market_data indexes');

// AI analysis results indexes
db.ai_analysis_results.createIndex({ symbol: 1, created_at: -1 });
db.ai_analysis_results.createIndex({ signal_type: 1 });
db.ai_analysis_results.createIndex({ confidence_score: -1 });
print('✓ Created ai_analysis_results indexes');

// Portfolio snapshots indexes
db.portfolio_snapshots.createIndex({ user_id: 1, created_at: -1 });
db.portfolio_snapshots.createIndex({ created_at: 1 }, { expireAfterSeconds: 7776000 }); // 90 days TTL
print('✓ Created portfolio_snapshots indexes');

// Strategy configs indexes
db.strategy_configs.createIndex({ user_id: 1, strategy_name: 1 });
db.strategy_configs.createIndex({ is_active: 1 });
print('✓ Created strategy_configs indexes');

// Paper trading indexes
db.paper_trading_accounts.createIndex({ user_id: 1 }, { unique: true });
db.paper_trading_trades.createIndex({ user_id: 1, created_at: -1 });
db.paper_trading_trades.createIndex({ symbol: 1, created_at: -1 });
print('✓ Created paper_trading indexes');

// AI signals indexes
db.ai_signals.createIndex({ symbol: 1, created_at: -1 });
db.ai_signals.createIndex({ executed: 1 });
print('✓ Created ai_signals indexes');

// Performance metrics indexes
db.performance_metrics.createIndex({ user_id: 1, date: -1 });
db.performance_metrics.createIndex({ date: -1 });
print('✓ Created performance_metrics indexes');

// Audit logs indexes
db.audit_logs.createIndex({ user_id: 1, created_at: -1 });
db.audit_logs.createIndex({ action_type: 1, created_at: -1 });
print('✓ Created audit_logs indexes');

// Sessions indexes
db.sessions.createIndex({ user_id: 1 });
db.sessions.createIndex({ expires_at: 1 }, { expireAfterSeconds: 0 }); // Auto-cleanup expired sessions
print('✓ Created sessions indexes');

// Notifications indexes
db.notifications.createIndex({ user_id: 1, created_at: -1 });
db.notifications.createIndex({ is_read: 1 });
print('✓ Created notifications indexes');

// API keys indexes
db.api_keys.createIndex({ user_id: 1 });
db.api_keys.createIndex({ key_hash: 1 }, { unique: true });
print('✓ Created api_keys indexes');

// ML Models indexes
db.ml_models.createIndex({ model_name: 1, version: 1 }, { unique: true });
db.ml_models.createIndex({ created_at: -1 });
db.ml_models.createIndex({ is_active: 1 });
print('✓ Created ml_models indexes');

// Predictions indexes
db.predictions.createIndex({ symbol: 1, created_at: -1 });
db.predictions.createIndex({ model_id: 1, created_at: -1 });
print('✓ Created predictions indexes');

// Training jobs indexes
db.training_jobs.createIndex({ status: 1, created_at: -1 });
db.training_jobs.createIndex({ model_name: 1, created_at: -1 });
print('✓ Created training_jobs indexes');

// Market indicators indexes
db.market_indicators.createIndex({ symbol: 1, timestamp: -1 });
db.market_indicators.createIndex({ indicator_type: 1, timestamp: -1 });
print('✓ Created market_indicators indexes');

// Seed demo users (password: password123)
print('\n[5/7] Seeding demo users...');

const demoPasswordHash = '$2b$12$MIXXJQAdvglFlvqdlkB7nOaByVpPwEZeXSmpgrABBkLXFiqkVjtbi';
const now = new Date();

db.users.insertOne({
  email: 'trader@botcore.com',
  password_hash: demoPasswordHash,
  is_active: true,
  is_admin: false,
  two_factor_enabled: false,
  created_at: now,
  updated_at: now,
  settings: {
    trading_enabled: false,
    risk_level: 'Medium',
    max_positions: 3,
    default_quantity: 0.01,
    notifications: { email_alerts: true, trade_notifications: true, system_alerts: true }
  }
});
print('✓ Created demo trader (trader@botcore.com / password123)');

db.users.insertOne({
  email: 'admin@botcore.com',
  password_hash: demoPasswordHash,
  is_active: true,
  is_admin: true,
  two_factor_enabled: false,
  created_at: now,
  updated_at: now,
  settings: {
    trading_enabled: false,
    risk_level: 'Medium',
    max_positions: 5,
    default_quantity: 0.01,
    notifications: { email_alerts: true, trade_notifications: true, system_alerts: true }
  }
});
print('✓ Created demo admin (admin@botcore.com / password123)');

// Insert initial system configuration
print('\n[6/7] Inserting initial configuration...');

db.system_config.insertOne({
  _id: 'global_config',
  trading_enabled: false,
  maintenance_mode: false,
  max_concurrent_trades: 100,
  default_risk_level: 'Medium',
  api_version: 'v1',
  created_at: new Date(),
  updated_at: new Date()
});
print('✓ Inserted global system configuration');

db.system_config.insertOne({
  _id: 'rate_limits',
  api_calls_per_minute: 60,
  trades_per_minute: 10,
  websocket_connections_per_user: 5,
  created_at: new Date()
});
print('✓ Inserted rate limit configuration');

print('\n==================================================');
print('MongoDB Initialization Completed Successfully!');
print('==================================================');
print('\nDatabase: bot_core');
print('Collections created: 21');
print('Indexes created: 50+');
print('DB Users created: 3');
print('Demo app users seeded: 2 (trader@botcore.com, admin@botcore.com)');
print('\nConnection strings:');
print('  Admin:     mongodb://bot_core_admin:<password>@localhost:27017/bot_core');
print('  App:       mongodb://bot_core_app:<password>@localhost:27017/bot_core');
print('  Read-only: mongodb://bot_core_readonly:<password>@localhost:27017/bot_core');
print('\n==================================================\n');
