// @spec:FR-DB-007 - Python AI Service Market Data Migration
// @ref:specs/02-design/2.2-database/DB-SCHEMA.md
// Migration 002: Market data and technical indicators for Python AI Service
// Created: 2025-11-18
// Purpose: Configure time-series collections for market data and indicators

print('========================================');
print('Migration 002: Market Data Schema');
print('Python AI Service Collections');
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

// 1. Market Indicators Collection
print('[1/5] Creating market_indicators collection...');

if (!db.getCollectionNames().includes('market_indicators')) {
    db.createCollection('market_indicators');
}

db.runCommand({
    collMod: 'market_indicators',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['symbol', 'indicator_type', 'timestamp', 'values'],
            properties: {
                symbol: {
                    bsonType: 'string',
                    pattern: '^[A-Z]{6,12}$',
                    description: 'Trading pair (e.g., BTCUSDT)'
                },
                indicator_type: {
                    enum: [
                        // Trend Indicators
                        'SMA', 'EMA', 'MACD', 'ADX',
                        // Momentum Indicators
                        'RSI', 'STOCHASTIC', 'CCI', 'WILLIAMS_R', 'MFI',
                        // Volatility Indicators
                        'BOLLINGER_BANDS', 'ATR', 'KELTNER_CHANNEL',
                        // Volume Indicators
                        'OBV', 'VWAP', 'CMF', 'VOLUME_PROFILE',
                        // Custom Indicators
                        'SENTIMENT', 'CUSTOM'
                    ],
                    description: 'Type of technical indicator'
                },
                timeframe: {
                    enum: ['1m', '5m', '15m', '30m', '1h', '4h', '1d', '1w'],
                    description: 'Indicator timeframe'
                },
                timestamp: {
                    bsonType: 'date',
                    description: 'Indicator calculation timestamp'
                },

                // Indicator Values (flexible structure for different indicators)
                values: {
                    bsonType: 'object',
                    description: 'Indicator-specific values',
                    properties: {
                        // Generic values
                        value: { bsonType: ['double', 'null'] },
                        signal: { bsonType: ['string', 'null'] },

                        // MACD specific
                        macd: { bsonType: ['double', 'null'] },
                        signal_line: { bsonType: ['double', 'null'] },
                        histogram: { bsonType: ['double', 'null'] },

                        // Bollinger Bands specific
                        upper_band: { bsonType: ['double', 'null'] },
                        middle_band: { bsonType: ['double', 'null'] },
                        lower_band: { bsonType: ['double', 'null'] },
                        bandwidth: { bsonType: ['double', 'null'] },

                        // Stochastic specific
                        k: { bsonType: ['double', 'null'] },
                        d: { bsonType: ['double', 'null'] },

                        // ATR, ADX, etc.
                        strength: { bsonType: ['double', 'null'] },
                        direction: { bsonType: ['string', 'null'] }
                    }
                },

                // Context
                price_context: {
                    bsonType: ['object', 'null'],
                    properties: {
                        open: { bsonType: 'double' },
                        high: { bsonType: 'double' },
                        low: { bsonType: 'double' },
                        close: { bsonType: 'double' },
                        volume: { bsonType: 'double' }
                    }
                },

                // Calculation metadata
                calculation_params: {
                    bsonType: ['object', 'null'],
                    description: 'Parameters used for calculation'
                },
                data_points_used: {
                    bsonType: ['int', 'null'],
                    description: 'Number of data points used'
                },

                // Metadata
                created_at: {
                    bsonType: 'date'
                },
                metadata: {
                    bsonType: ['object', 'null']
                }
            }
        }
    }
});
print('✓ Market indicators collection configured');

print('');

// 2. Create indexes for market_indicators
print('[2/5] Creating market_indicators indexes...');

createIndexSafely('market_indicators',
    { symbol: 1, indicator_type: 1, timestamp: -1 },
    { name: 'idx_indicators_symbol_type_time' },
    'market_indicators: symbol + type + timestamp'
);

createIndexSafely('market_indicators',
    { indicator_type: 1, timeframe: 1, timestamp: -1 },
    { name: 'idx_indicators_type_timeframe' },
    'market_indicators: type + timeframe + timestamp'
);

createIndexSafely('market_indicators',
    { timestamp: 1 },
    { expireAfterSeconds: 7776000, name: 'idx_indicators_ttl' },
    'market_indicators: timestamp (TTL 90 days)'
);

createIndexSafely('market_indicators',
    { symbol: 1, timestamp: -1 },
    { name: 'idx_indicators_symbol_time' },
    'market_indicators: symbol + timestamp'
);

print('');

// 3. Create indexes for ML Models
print('[3/5] Creating ml_models indexes...');

createIndexSafely('ml_models',
    { model_name: 1, version: 1 },
    { unique: true, name: 'idx_models_name_version' },
    'ml_models: name + version (unique)'
);

createIndexSafely('ml_models',
    { is_active: 1, is_production: 1 },
    { name: 'idx_models_status' },
    'ml_models: is_active + is_production'
);

createIndexSafely('ml_models',
    { created_at: -1 },
    { name: 'idx_models_created' },
    'ml_models: created_at'
);

createIndexSafely('ml_models',
    { 'performance_metrics.accuracy': -1 },
    { sparse: true, name: 'idx_models_accuracy' },
    'ml_models: accuracy (sparse)'
);

print('');

// 4. Create indexes for Predictions
print('[4/5] Creating predictions indexes...');

createIndexSafely('predictions',
    { symbol: 1, created_at: -1 },
    { name: 'idx_predictions_symbol_time' },
    'predictions: symbol + created_at'
);

createIndexSafely('predictions',
    { model_id: 1, created_at: -1 },
    { name: 'idx_predictions_model_time' },
    'predictions: model_id + created_at'
);

createIndexSafely('predictions',
    { symbol: 1, prediction_type: 1, timeframe: 1, created_at: -1 },
    { name: 'idx_predictions_symbol_type_frame' },
    'predictions: symbol + type + timeframe + created_at'
);

createIndexSafely('predictions',
    { confidence_score: -1 },
    { name: 'idx_predictions_confidence' },
    'predictions: confidence_score'
);

createIndexSafely('predictions',
    { is_correct: 1 },
    { sparse: true, name: 'idx_predictions_verified' },
    'predictions: is_correct (sparse)'
);

createIndexSafely('predictions',
    { created_at: 1 },
    { expireAfterSeconds: 7776000, name: 'idx_predictions_ttl' },
    'predictions: created_at (TTL 90 days)'
);

print('');

// 5. Create indexes for Training Jobs
print('[5/5] Creating training_jobs indexes...');

createIndexSafely('training_jobs',
    { job_id: 1 },
    { unique: true, name: 'idx_jobs_id' },
    'training_jobs: job_id (unique)'
);

createIndexSafely('training_jobs',
    { status: 1, created_at: -1 },
    { name: 'idx_jobs_status_time' },
    'training_jobs: status + created_at'
);

createIndexSafely('training_jobs',
    { model_name: 1, created_at: -1 },
    { name: 'idx_jobs_model_time' },
    'training_jobs: model_name + created_at'
);

createIndexSafely('training_jobs',
    { output_model_id: 1 },
    { sparse: true, name: 'idx_jobs_output_model' },
    'training_jobs: output_model_id (sparse)'
);

createIndexSafely('training_jobs',
    { created_at: 1 },
    { expireAfterSeconds: 15552000, name: 'idx_jobs_ttl' },
    'training_jobs: created_at (TTL 180 days)'
);

// Index for model_performance_history
createIndexSafely('model_performance_history',
    { model_id: 1, date: -1 },
    { name: 'idx_perf_history_model_date' },
    'model_performance_history: model_id + date'
);

createIndexSafely('model_performance_history',
    { date: 1 },
    { expireAfterSeconds: 31536000, name: 'idx_perf_history_ttl' },
    'model_performance_history: date (TTL 365 days)'
);

print('');

// Insert initial AI configuration
print('Inserting AI service configuration...\n');

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

// AI Service Configuration
insertIfNotExists('system_config',
    { _id: 'ai_service_config' },
    {
        _id: 'ai_service_config',
        enabled: true,
        openai_integration: {
            enabled: true,
            model: 'gpt-4-turbo',
            max_tokens: 4000,
            temperature: 0.7
        },
        ml_models: {
            auto_training: false,
            training_schedule: 'weekly',
            min_data_points: 1000,
            validation_split: 0.2,
            test_split: 0.1
        },
        prediction_settings: {
            min_confidence_threshold: 0.6,
            max_predictions_per_minute: 100,
            cache_predictions: true,
            cache_ttl_minutes: 5
        },
        technical_indicators: {
            default_timeframes: ['5m', '15m', '1h', '4h', '1d'],
            calculate_on_demand: true,
            cache_indicators: true,
            cache_ttl_minutes: 1
        },
        created_at: new Date(),
        updated_at: new Date()
    },
    'AI service configuration'
);

// Default ML Model Templates
insertIfNotExists('system_config',
    { _id: 'ml_model_templates' },
    {
        _id: 'ml_model_templates',
        templates: {
            lstm_price_prediction: {
                model_type: 'TIME_SERIES',
                architecture: {
                    layers: 3,
                    units_per_layer: [128, 64, 32],
                    dropout: 0.2,
                    activation: 'relu'
                },
                training_config: {
                    epochs: 50,
                    batch_size: 32,
                    learning_rate: 0.001,
                    optimizer: 'adam',
                    loss_function: 'mse'
                }
            },
            transformer_trend: {
                model_type: 'TIME_SERIES',
                architecture: {
                    num_heads: 8,
                    num_layers: 4,
                    d_model: 256,
                    dropout: 0.1
                },
                training_config: {
                    epochs: 30,
                    batch_size: 64,
                    learning_rate: 0.0001,
                    optimizer: 'adamw',
                    loss_function: 'categorical_crossentropy'
                }
            },
            ensemble_classification: {
                model_type: 'CLASSIFICATION',
                components: ['random_forest', 'gradient_boosting', 'xgboost'],
                voting: 'soft',
                weights: [0.3, 0.4, 0.3]
            }
        },
        created_at: new Date()
    },
    'ML model templates'
);

// Technical Indicator Presets
insertIfNotExists('system_config',
    { _id: 'indicator_presets' },
    {
        _id: 'indicator_presets',
        presets: {
            trend_following: ['SMA', 'EMA', 'MACD', 'ADX'],
            momentum: ['RSI', 'STOCHASTIC', 'CCI', 'WILLIAMS_R'],
            volatility: ['BOLLINGER_BANDS', 'ATR', 'KELTNER_CHANNEL'],
            volume: ['OBV', 'VWAP', 'CMF', 'VOLUME_PROFILE'],
            comprehensive: [
                'SMA', 'EMA', 'MACD', 'RSI', 'BOLLINGER_BANDS',
                'ATR', 'OBV', 'VWAP', 'ADX', 'STOCHASTIC'
            ]
        },
        default_params: {
            RSI: { period: 14 },
            MACD: { fast: 12, slow: 26, signal: 9 },
            BOLLINGER_BANDS: { period: 20, std_dev: 2 },
            SMA: { period: 20 },
            EMA: { period: 20 },
            ATR: { period: 14 },
            STOCHASTIC: { k_period: 14, d_period: 3 }
        },
        created_at: new Date()
    },
    'Technical indicator presets'
);

print('\n========================================');
print('Verifying Collections and Indexes...');
print('========================================\n');

const collections = [
    'market_indicators',
    'ml_models',
    'predictions',
    'training_jobs',
    'model_performance_history'
];

collections.forEach(function(collName) {
    if (db.getCollectionNames().includes(collName)) {
        const indexes = db.getCollection(collName).getIndexes();
        print(collName + ': ' + indexes.length + ' indexes');
    }
});

print('\n========================================');
print('Migration 002 Completed Successfully!');
print('Market data and indicators configured');
print('========================================\n');
