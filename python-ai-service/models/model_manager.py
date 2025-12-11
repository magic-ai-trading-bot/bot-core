import os
from datetime import datetime, timedelta
from typing import Any, Dict, Optional

import joblib
import pandas as pd

from config.config import config
from features.feature_engineering import FeatureEngineer
from utils.helpers import ensure_directory_exists, get_current_timestamp
from utils.logger import get_logger

from .gru_model import GRUModel
from .lstm_model import LSTMModel
from .transformer_model import TransformerModel

logger = get_logger("ModelManager")


class ModelManager:
    """Centralized model management for the AI trading service."""

    def __init__(self):
        self.config = config.get_model_config()
        self.management_config = config.get_model_management_config()
        self.trading_config = config.get_trading_config()

        self.feature_engineer = FeatureEngineer()
        self.current_model = None
        self.model_type = self.config.get("type", "lstm")
        self.model_metadata = {}

        # Ensure model directory exists
        self.model_save_path = self.management_config.get(
            "model_save_path", "./models/saved/"
        )
        ensure_directory_exists(self.model_save_path)

    def create_model(self, model_type: Optional[str] = None):
        """Create a new model instance."""
        model_type = model_type or self.model_type

        try:
            if model_type.lower() == "lstm":
                self.current_model = LSTMModel()
            elif model_type.lower() == "gru":
                self.current_model = GRUModel()
            elif model_type.lower() == "transformer":
                self.current_model = TransformerModel()
            else:
                raise ValueError(f"Unsupported model type: {model_type}")

            self.model_type = model_type.lower()
            logger.info(f"Created {model_type} model")

            return self.current_model
        except Exception as e:
            logger.error(f"Error creating model: {e}")
            raise

    def train_model(self, df: pd.DataFrame, retrain: bool = False) -> Dict[str, Any]:
        """Train the model with provided data."""
        try:
            logger.info(f"Starting model training with {len(df)} samples")

            # Prepare features
            processed_df = self.feature_engineer.prepare_features(df)

            # Create sequences
            X, y = self.feature_engineer.create_sequences(processed_df)

            if len(X) == 0:
                raise ValueError("No sequences created from data")

            # Scale features
            X_scaled = self.feature_engineer.scale_features(X, fit_scaler=True)

            # Split data
            train_size = int(
                len(X_scaled) * (1 - self.config.get("validation_split", 0.2))
            )
            X_train, X_val = X_scaled[:train_size], X_scaled[train_size:]
            y_train, y_val = y[:train_size], y[train_size:]

            # Create model if not exists
            if self.current_model is None or retrain:
                self.create_model()

            # Prepare save path
            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            model_filename = f"{self.model_type}_model_{timestamp}.h5"
            model_path = os.path.join(self.model_save_path, model_filename)

            # Train model
            training_results = self.current_model.train(
                X_train, y_train, X_val, y_val, save_path=model_path
            )

            # Save metadata
            self.model_metadata = {
                "model_type": self.model_type,
                "trained_timestamp": get_current_timestamp(),
                "training_samples": len(X_train),
                "validation_samples": len(X_val),
                "feature_count": self.feature_engineer.get_features_count(),
                "sequence_length": self.config.get("sequence_length", 60),
                "training_results": training_results,
                "model_path": model_path,
            }

            # Save feature engineer and metadata
            self._save_feature_engineer(timestamp)
            self._save_metadata(timestamp)

            logger.info("Model training completed successfully")
            return training_results

        except Exception as e:
            logger.error(f"Error training model: {e}")
            raise

    def predict(self, df: pd.DataFrame) -> Dict[str, Any]:
        """Make prediction using the current model."""
        try:
            if self.current_model is None:
                raise ValueError("No model loaded for prediction")

            # Prepare data for inference
            X = self.feature_engineer.prepare_for_inference(df)

            if X is None:
                raise ValueError("Failed to prepare data for inference")

            # Make prediction
            prediction_prob = self.current_model.predict_single(X)

            # Determine signal based on thresholds
            signal = self._determine_signal(prediction_prob)

            # Calculate confidence
            confidence = self._calculate_confidence(prediction_prob)

            result = {
                "signal": signal,
                "confidence": confidence,
                "probability": prediction_prob,
                "timestamp": get_current_timestamp(),
                "model_type": self.model_type,
            }

            logger.info(f"Prediction made: {result}")
            return result

        except Exception as e:
            logger.error(f"Error making prediction: {e}")
            return {
                "signal": "neutral",
                "confidence": 0.0,
                "probability": 0.5,
                "timestamp": get_current_timestamp(),
                "error": str(e),
            }

    def _determine_signal(self, probability: float) -> str:
        """Determine trading signal based on probability."""
        long_threshold = self.trading_config.get("long_threshold", 0.6)
        short_threshold = self.trading_config.get("short_threshold", 0.4)

        if probability >= long_threshold:
            return "long"
        elif probability <= short_threshold:
            return "short"
        else:
            return "neutral"

    def _calculate_confidence(self, probability: float) -> float:
        """Calculate confidence score based on probability."""
        # Distance from neutral (0.5)
        distance_from_neutral = abs(probability - 0.5)
        # Convert to percentage (0-50% becomes 0-100%)
        confidence = (distance_from_neutral * 2) * 100
        return round(confidence, 2)

    def save_model(self, model_name: Optional[str] = None) -> bool:
        """Save the current model and associated components."""
        try:
            if self.current_model is None:
                logger.warning("No model to save")
                return False

            timestamp = datetime.now().strftime("%Y%m%d_%H%M%S")
            model_name = model_name or f"{self.model_type}_model_{timestamp}"

            # Save model
            model_path = os.path.join(self.model_save_path, f"{model_name}.h5")
            success = self.current_model.save_model(model_path)

            if success:
                # Update metadata
                self.model_metadata["model_path"] = model_path
                self.model_metadata["saved_timestamp"] = get_current_timestamp()

                # Save feature engineer and metadata
                self._save_feature_engineer(timestamp)
                self._save_metadata(timestamp)

                logger.info(f"Model saved successfully as {model_name}")
                return True

            return False

        except Exception as e:
            logger.error(f"Error saving model: {e}")
            return False

    def load_model(self, model_path: Optional[str] = None) -> bool:
        """Load a saved model and associated components."""
        try:
            if model_path is None:
                # Find the latest model
                model_path = self._find_latest_model()

            if not model_path or not os.path.exists(model_path):
                logger.error(f"Model file not found: {model_path}")
                return False

            # Extract timestamp from filename
            filename = os.path.basename(model_path)
            timestamp = filename.split("_")[-1].replace(".h5", "")

            # Load metadata
            if not self._load_metadata(timestamp):
                logger.warning("Could not load metadata, proceeding with defaults")

            # Determine model type from metadata or filename
            model_type = self.model_metadata.get("model_type", "lstm")

            # Create model
            self.create_model(model_type)

            # Load model weights
            success = self.current_model.load_model(model_path)

            if success:
                # Load feature engineer
                self._load_feature_engineer(timestamp)

                logger.info(f"Model loaded successfully from {model_path}")
                return True

            return False

        except Exception as e:
            logger.error(f"Error loading model: {e}")
            return False

    def _find_latest_model(self) -> Optional[str]:
        """Find the latest saved model."""
        try:
            model_files = []
            for file in os.listdir(self.model_save_path):
                if file.endswith(".h5"):
                    model_files.append(os.path.join(self.model_save_path, file))

            if not model_files:
                return None

            # Sort by modification time
            model_files.sort(key=os.path.getmtime, reverse=True)
            return model_files[0]

        except Exception as e:
            logger.error(f"Error finding latest model: {e}")
            return None

    def _save_feature_engineer(self, timestamp: str) -> bool:
        """Save feature engineer."""
        try:
            fe_path = os.path.join(
                self.model_save_path, f"feature_engineer_{timestamp}.pkl"
            )
            joblib.dump(self.feature_engineer, fe_path)
            logger.debug(f"Feature engineer saved to {fe_path}")
            return True
        except Exception as e:
            logger.error(f"Error saving feature engineer: {e}")
            return False

    def _load_feature_engineer(self, timestamp: str) -> bool:
        """Load feature engineer."""
        try:
            fe_path = os.path.join(
                self.model_save_path, f"feature_engineer_{timestamp}.pkl"
            )
            if os.path.exists(fe_path):
                self.feature_engineer = joblib.load(fe_path)
                logger.debug(f"Feature engineer loaded from {fe_path}")
                return True
            else:
                logger.warning(f"Feature engineer file not found: {fe_path}")
                return False
        except Exception as e:
            logger.error(f"Error loading feature engineer: {e}")
            return False

    def _save_metadata(self, timestamp: str) -> bool:
        """Save model metadata."""
        try:
            metadata_path = os.path.join(
                self.model_save_path, f"metadata_{timestamp}.pkl"
            )
            joblib.dump(self.model_metadata, metadata_path)
            logger.debug(f"Metadata saved to {metadata_path}")
            return True
        except Exception as e:
            logger.error(f"Error saving metadata: {e}")
            return False

    def _load_metadata(self, timestamp: str) -> bool:
        """Load model metadata."""
        try:
            metadata_path = os.path.join(
                self.model_save_path, f"metadata_{timestamp}.pkl"
            )
            if os.path.exists(metadata_path):
                self.model_metadata = joblib.load(metadata_path)
                logger.debug(f"Metadata loaded from {metadata_path}")
                return True
            else:
                logger.warning(f"Metadata file not found: {metadata_path}")
                return False
        except Exception as e:
            logger.error(f"Error loading metadata: {e}")
            return False

    def should_retrain(self) -> bool:
        """Check if model should be retrained based on configured interval."""
        try:
            if not self.model_metadata:
                return True

            last_training = self.model_metadata.get("trained_timestamp")
            if not last_training:
                return True

            # Parse timestamp
            last_training_dt = datetime.fromisoformat(
                last_training.replace("Z", "+00:00")
            )

            # Check if retrain interval has passed
            retrain_interval = self.management_config.get("retrain_interval_hours", 24)
            next_retrain = last_training_dt + timedelta(hours=retrain_interval)

            return datetime.now() >= next_retrain

        except Exception as e:
            logger.error(f"Error checking retrain schedule: {e}")
            return False

    def get_model_info(self) -> Dict[str, Any]:
        """Get information about the current model."""
        info = {
            "model_type": self.model_type,
            "model_loaded": self.current_model is not None,
            "metadata": self.model_metadata,
        }

        if self.current_model:
            info["summary"] = self.current_model.get_model_summary()
            info["training_history"] = self.current_model.get_training_history()

        return info

    def cleanup_old_models(self, keep_count: Optional[int] = None) -> int:
        """Clean up old model files, keeping only the most recent ones."""
        try:
            keep_count = keep_count or self.management_config.get("backup_count", 5)

            # Get all model files
            model_files = []
            for file in os.listdir(self.model_save_path):
                if file.endswith(".h5"):
                    filepath = os.path.join(self.model_save_path, file)
                    model_files.append((filepath, os.path.getmtime(filepath)))

            # Sort by modification time (newest first)
            model_files.sort(key=lambda x: x[1], reverse=True)

            # Delete old files
            deleted_count = 0
            for filepath, _ in model_files[keep_count:]:
                try:
                    os.remove(filepath)

                    # Also remove associated files
                    base_name = os.path.basename(filepath).replace(".h5", "")
                    timestamp = base_name.split("_")[-1]

                    # Remove feature engineer file
                    fe_file = os.path.join(
                        self.model_save_path, f"feature_engineer_{timestamp}.pkl"
                    )
                    if os.path.exists(fe_file):
                        os.remove(fe_file)

                    # Remove metadata file
                    metadata_file = os.path.join(
                        self.model_save_path, f"metadata_{timestamp}.pkl"
                    )
                    if os.path.exists(metadata_file):
                        os.remove(metadata_file)

                    deleted_count += 1
                    logger.debug(f"Deleted old model: {filepath}")

                except Exception as e:
                    logger.error(f"Error deleting {filepath}: {e}")

            if deleted_count > 0:
                logger.info(f"Cleaned up {deleted_count} old model files")

            return deleted_count

        except Exception as e:
            logger.error(f"Error cleaning up old models: {e}")
            return 0
