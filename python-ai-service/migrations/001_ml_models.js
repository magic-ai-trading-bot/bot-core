// @spec:FR-DB-006 - Python AI Service ML Models Schema Migration
// @ref:specs/02-design/2.2-database/DB-SCHEMA.md
// Migration 001: ML Models and predictions schema for Python AI Service
// Created: 2025-11-18
// Purpose: Create collections for ML models, predictions, and training jobs

print('========================================');
print('Migration 001: ML Models Schema');
print('Python AI Service Collections');
print('========================================\n');

// Connect to bot_core database
db = db.getSiblingDB('bot_core');

// 1. ML Models Collection
print('[1/4] Updating ml_models collection schema...');
db.runCommand({
    collMod: 'ml_models',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['model_name', 'model_type', 'version', 'created_at'],
            properties: {
                model_name: {
                    enum: ['LSTM', 'GRU', 'TRANSFORMER', 'RANDOM_FOREST', 'GRADIENT_BOOSTING', 'ENSEMBLE'],
                    description: 'Type of ML model'
                },
                model_type: {
                    enum: ['CLASSIFICATION', 'REGRESSION', 'TIME_SERIES', 'REINFORCEMENT_LEARNING'],
                    description: 'Model task type'
                },
                version: {
                    bsonType: 'string',
                    pattern: '^\\d+\\.\\d+\\.\\d+$',
                    description: 'Semantic version (e.g., 1.0.0)'
                },
                description: {
                    bsonType: ['string', 'null'],
                    description: 'Model description'
                },

                // Model Architecture
                architecture: {
                    bsonType: 'object',
                    description: 'Model architecture details',
                    properties: {
                        input_shape: { bsonType: 'array' },
                        output_shape: { bsonType: 'array' },
                        layers: { bsonType: 'int' },
                        parameters: { bsonType: 'int' },
                        activation_functions: { bsonType: 'array' }
                    }
                },

                // Training Configuration
                training_config: {
                    bsonType: 'object',
                    properties: {
                        epochs: { bsonType: 'int' },
                        batch_size: { bsonType: 'int' },
                        learning_rate: { bsonType: 'double' },
                        optimizer: { bsonType: 'string' },
                        loss_function: { bsonType: 'string' },
                        validation_split: { bsonType: 'double' }
                    }
                },

                // Performance Metrics
                performance_metrics: {
                    bsonType: 'object',
                    properties: {
                        accuracy: { bsonType: 'double' },
                        precision: { bsonType: 'double' },
                        recall: { bsonType: 'double' },
                        f1_score: { bsonType: 'double' },
                        mae: { bsonType: 'double' },
                        mse: { bsonType: 'double' },
                        rmse: { bsonType: 'double' },
                        r2_score: { bsonType: 'double' }
                    }
                },

                // Model Files
                model_path: {
                    bsonType: 'string',
                    description: 'Path to saved model file'
                },
                weights_path: {
                    bsonType: ['string', 'null'],
                    description: 'Path to model weights'
                },
                config_path: {
                    bsonType: ['string', 'null'],
                    description: 'Path to model configuration'
                },

                // Status and Metadata
                is_active: {
                    bsonType: 'bool',
                    description: 'Is model currently in use'
                },
                is_production: {
                    bsonType: 'bool',
                    description: 'Is model deployed to production'
                },
                training_duration_seconds: {
                    bsonType: ['int', 'null'],
                    description: 'Total training time'
                },
                training_samples: {
                    bsonType: ['int', 'null'],
                    description: 'Number of training samples'
                },
                validation_samples: {
                    bsonType: ['int', 'null'],
                    description: 'Number of validation samples'
                },

                // Timestamps
                created_at: {
                    bsonType: 'date'
                },
                updated_at: {
                    bsonType: 'date'
                },
                deployed_at: {
                    bsonType: ['date', 'null']
                },

                // Additional metadata
                metadata: {
                    bsonType: 'object',
                    description: 'Additional model metadata'
                }
            }
        }
    },
    validationLevel: 'moderate'
});
print('✓ ML models collection schema updated');

print('');

// 2. Predictions Collection
print('[2/4] Updating predictions collection schema...');
db.runCommand({
    collMod: 'predictions',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['model_id', 'symbol', 'prediction_type', 'created_at'],
            properties: {
                model_id: {
                    bsonType: 'objectId',
                    description: 'Reference to ml_models collection'
                },
                model_name: {
                    bsonType: 'string',
                    description: 'Model name for quick reference'
                },
                model_version: {
                    bsonType: 'string',
                    description: 'Model version used'
                },

                // Trading Symbol
                symbol: {
                    bsonType: 'string',
                    pattern: '^[A-Z]{6,12}$',
                    description: 'Trading pair (e.g., BTCUSDT)'
                },

                // Prediction Details
                prediction_type: {
                    enum: ['PRICE_DIRECTION', 'PRICE_VALUE', 'TREND', 'VOLATILITY', 'SUPPORT_RESISTANCE'],
                    description: 'Type of prediction'
                },
                timeframe: {
                    enum: ['1m', '5m', '15m', '30m', '1h', '4h', '1d'],
                    description: 'Prediction timeframe'
                },

                // Prediction Results
                predicted_value: {
                    bsonType: ['double', 'null'],
                    description: 'Predicted numerical value (for regression)'
                },
                predicted_direction: {
                    enum: ['UP', 'DOWN', 'NEUTRAL', null],
                    description: 'Predicted direction (for classification)'
                },
                confidence_score: {
                    bsonType: 'double',
                    minimum: 0.0,
                    maximum: 1.0,
                    description: 'Prediction confidence (0-1)'
                },
                probability_distribution: {
                    bsonType: ['object', 'null'],
                    description: 'Probability distribution for multi-class'
                },

                // Context Data
                input_features: {
                    bsonType: 'object',
                    description: 'Features used for prediction'
                },
                technical_indicators: {
                    bsonType: ['object', 'null'],
                    description: 'Technical indicators at prediction time'
                },
                market_context: {
                    bsonType: ['object', 'null'],
                    properties: {
                        current_price: { bsonType: 'double' },
                        volume_24h: { bsonType: 'double' },
                        volatility: { bsonType: 'double' },
                        trend: { bsonType: 'string' }
                    }
                },

                // Verification (for backtesting)
                actual_value: {
                    bsonType: ['double', 'null'],
                    description: 'Actual outcome (for accuracy tracking)'
                },
                actual_direction: {
                    enum: ['UP', 'DOWN', 'NEUTRAL', null],
                    description: 'Actual direction'
                },
                is_correct: {
                    bsonType: ['bool', 'null'],
                    description: 'Was prediction correct'
                },
                error_margin: {
                    bsonType: ['double', 'null'],
                    description: 'Prediction error'
                },

                // Timestamps
                created_at: {
                    bsonType: 'date',
                    description: 'Prediction creation time'
                },
                verified_at: {
                    bsonType: ['date', 'null'],
                    description: 'When prediction was verified'
                },

                // Metadata
                metadata: {
                    bsonType: 'object',
                    description: 'Additional prediction metadata'
                }
            }
        }
    }
});
print('✓ Predictions collection schema updated');

print('');

// 3. Training Jobs Collection
print('[3/4] Updating training_jobs collection schema...');
db.runCommand({
    collMod: 'training_jobs',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['job_id', 'model_name', 'status', 'created_at'],
            properties: {
                job_id: {
                    bsonType: 'string',
                    description: 'Unique job identifier'
                },
                model_name: {
                    bsonType: 'string',
                    description: 'Name of model being trained'
                },
                model_version: {
                    bsonType: 'string',
                    description: 'Target model version'
                },

                // Job Status
                status: {
                    enum: ['PENDING', 'RUNNING', 'COMPLETED', 'FAILED', 'CANCELLED'],
                    description: 'Training job status'
                },
                progress_percentage: {
                    bsonType: ['double', 'null'],
                    minimum: 0.0,
                    maximum: 100.0,
                    description: 'Training progress (0-100)'
                },
                current_epoch: {
                    bsonType: ['int', 'null'],
                    description: 'Current training epoch'
                },
                total_epochs: {
                    bsonType: ['int', 'null'],
                    description: 'Total epochs to train'
                },

                // Training Configuration
                training_config: {
                    bsonType: 'object',
                    description: 'Training hyperparameters'
                },
                dataset_info: {
                    bsonType: 'object',
                    properties: {
                        training_samples: { bsonType: 'int' },
                        validation_samples: { bsonType: 'int' },
                        test_samples: { bsonType: 'int' },
                        features_count: { bsonType: 'int' },
                        date_range_start: { bsonType: 'date' },
                        date_range_end: { bsonType: 'date' }
                    }
                },

                // Results
                metrics: {
                    bsonType: ['object', 'null'],
                    description: 'Training metrics per epoch'
                },
                final_metrics: {
                    bsonType: ['object', 'null'],
                    description: 'Final evaluation metrics'
                },
                best_epoch: {
                    bsonType: ['int', 'null'],
                    description: 'Epoch with best performance'
                },

                // Resource Usage
                resource_usage: {
                    bsonType: ['object', 'null'],
                    properties: {
                        cpu_usage_percent: { bsonType: 'double' },
                        memory_usage_mb: { bsonType: 'double' },
                        gpu_usage_percent: { bsonType: 'double' },
                        gpu_memory_mb: { bsonType: 'double' }
                    }
                },

                // Error Handling
                error_message: {
                    bsonType: ['string', 'null'],
                    description: 'Error message if failed'
                },
                error_stack: {
                    bsonType: ['string', 'null'],
                    description: 'Error stack trace'
                },

                // Timestamps
                created_at: {
                    bsonType: 'date'
                },
                started_at: {
                    bsonType: ['date', 'null']
                },
                completed_at: {
                    bsonType: ['date', 'null']
                },
                duration_seconds: {
                    bsonType: ['int', 'null']
                },

                // Output
                output_model_id: {
                    bsonType: ['objectId', 'null'],
                    description: 'Reference to created model'
                },

                // Metadata
                created_by: {
                    bsonType: ['string', 'null'],
                    description: 'User or system that created job'
                },
                metadata: {
                    bsonType: 'object'
                }
            }
        }
    }
});
print('✓ Training jobs collection schema updated');

print('');

// 4. Model Performance History
print('[4/4] Creating model_performance_history collection...');
if (!db.getCollectionNames().includes('model_performance_history')) {
    db.createCollection('model_performance_history');
}

db.runCommand({
    collMod: 'model_performance_history',
    validator: {
        $jsonSchema: {
            bsonType: 'object',
            required: ['model_id', 'date', 'metrics', 'created_at'],
            properties: {
                model_id: {
                    bsonType: 'objectId',
                    description: 'Reference to ml_models'
                },
                model_name: {
                    bsonType: 'string'
                },
                date: {
                    bsonType: 'date',
                    description: 'Performance tracking date'
                },
                metrics: {
                    bsonType: 'object',
                    properties: {
                        total_predictions: { bsonType: 'int' },
                        correct_predictions: { bsonType: 'int' },
                        accuracy: { bsonType: 'double' },
                        avg_confidence: { bsonType: 'double' },
                        avg_error: { bsonType: 'double' }
                    }
                },
                symbol_breakdown: {
                    bsonType: ['object', 'null'],
                    description: 'Per-symbol performance metrics'
                },
                created_at: {
                    bsonType: 'date'
                }
            }
        }
    }
});
print('✓ Model performance history collection created');

print('\n========================================');
print('Migration 001 Completed Successfully!');
print('ML Models collections configured');
print('========================================\n');
