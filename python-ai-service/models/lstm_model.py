import numpy as np
import tensorflow as tf
from tensorflow.keras.models import Sequential, Model
from tensorflow.keras.layers import LSTM, Dense, Dropout, BatchNormalization
from tensorflow.keras.optimizers import Adam
from tensorflow.keras.callbacks import EarlyStopping, ReduceLROnPlateau, ModelCheckpoint
from typing import Dict, Any, Optional, Tuple
from config.config import config
from utils.logger import get_logger



# @spec:FR-AI-001 - LSTM Model Prediction
# @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md
# @test:TC-AI-001, TC-AI-002, TC-AI-003

logger = get_logger("LSTMModel")


class LSTMModel:
    """LSTM model for cryptocurrency price prediction."""

    def __init__(self):
        self.config = config.get_model_config()
        self.model = None
        self.history = None

    def build_model(self, input_shape: Tuple[int, int]) -> Model:
        """Build LSTM model architecture."""
        try:
            sequence_length, features_count = input_shape

            model = Sequential(
                [
                    # First LSTM layer
                    LSTM(
                        units=self.config.get("hidden_size", 64),
                        return_sequences=True,
                        input_shape=(sequence_length, features_count),
                    ),
                    BatchNormalization(),
                    Dropout(self.config.get("dropout", 0.2)),
                    # Second LSTM layer
                    LSTM(
                        units=self.config.get("hidden_size", 64) // 2,
                        return_sequences=True,
                    ),
                    BatchNormalization(),
                    Dropout(self.config.get("dropout", 0.2)),
                    # Third LSTM layer
                    LSTM(
                        units=self.config.get("hidden_size", 64) // 4,
                        return_sequences=False,
                    ),
                    BatchNormalization(),
                    Dropout(self.config.get("dropout", 0.2)),
                    # Dense layers
                    Dense(32, activation="relu"),
                    BatchNormalization(),
                    Dropout(0.1),
                    Dense(16, activation="relu"),
                    BatchNormalization(),
                    Dropout(0.1),
                    # Output layer (sigmoid for probability output)
                    Dense(1, activation="sigmoid"),
                ]
            )

            # Compile model
            model.compile(
                optimizer=Adam(learning_rate=self.config.get("learning_rate", 0.001)),
                loss="binary_crossentropy",
                metrics=["accuracy", "precision", "recall"],
            )

            logger.info(
                f"LSTM model built successfully with input shape: {input_shape}"
            )
            logger.info(f"Model parameters: {model.count_params()}")

            self.model = model
            return model

        except Exception as e:
            logger.error(f"Error building LSTM model: {e}")
            raise

    def train(
        self,
        X_train: np.ndarray,
        y_train: np.ndarray,
        X_val: Optional[np.ndarray] = None,
        y_val: Optional[np.ndarray] = None,
        save_path: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Train the LSTM model."""
        try:
            if self.model is None:
                self.build_model((X_train.shape[1], X_train.shape[2]))

            # Prepare callbacks
            callbacks = []

            # Early stopping
            early_stopping = EarlyStopping(
                monitor="val_loss" if X_val is not None else "loss",
                patience=10,
                restore_best_weights=True,
                verbose=1,
            )
            callbacks.append(early_stopping)

            # Reduce learning rate on plateau
            reduce_lr = ReduceLROnPlateau(
                monitor="val_loss" if X_val is not None else "loss",
                factor=0.5,
                patience=5,
                min_lr=1e-7,
                verbose=1,
            )
            callbacks.append(reduce_lr)

            # Model checkpoint
            if save_path:
                checkpoint = ModelCheckpoint(
                    save_path,
                    monitor="val_loss" if X_val is not None else "loss",
                    save_best_only=True,
                    verbose=1,
                )
                callbacks.append(checkpoint)

            # Train model
            logger.info("Starting LSTM model training")

            validation_data = (
                (X_val, y_val) if X_val is not None and y_val is not None else None
            )

            self.history = self.model.fit(
                X_train,
                y_train,
                epochs=self.config.get("epochs", 100),
                batch_size=self.config.get("batch_size", 32),
                validation_data=validation_data,
                callbacks=callbacks,
                verbose=1,
            )

            # Get training results
            final_loss = self.history.history["loss"][-1]
            final_accuracy = self.history.history["accuracy"][-1]

            results = {
                "final_loss": final_loss,
                "final_accuracy": final_accuracy,
                "epochs_trained": len(self.history.history["loss"]),
                "best_val_loss": min(
                    self.history.history.get("val_loss", [float("inf")])
                ),
                "best_val_accuracy": max(self.history.history.get("val_accuracy", [0])),
            }

            logger.info(f"LSTM training completed. Results: {results}")
            return results

        except Exception as e:
            logger.error(f"Error training LSTM model: {e}")
            raise

    def predict(self, X: np.ndarray) -> np.ndarray:
        """Make predictions using the trained model."""
        try:
            if self.model is None:
                raise ValueError("Model not trained or loaded")

            predictions = self.model.predict(X, verbose=0)
            logger.debug(f"Made predictions for {len(X)} samples")

            return predictions.flatten()

        except Exception as e:
            logger.error(f"Error making predictions: {e}")
            raise

    def predict_single(self, X: np.ndarray) -> float:
        """Make a single prediction."""
        try:
            if len(X.shape) == 2:
                X = X.reshape(1, X.shape[0], X.shape[1])

            prediction = self.predict(X)
            return float(prediction[0])

        except Exception as e:
            logger.error(f"Error making single prediction: {e}")
            return 0.5  # Neutral prediction as fallback

    def evaluate(self, X_test: np.ndarray, y_test: np.ndarray) -> Dict[str, float]:
        """Evaluate model performance."""
        try:
            if self.model is None:
                raise ValueError("Model not trained or loaded")

            results = self.model.evaluate(X_test, y_test, verbose=0)
            metric_names = self.model.metrics_names

            evaluation = dict(zip(metric_names, results))
            logger.info(f"Model evaluation results: {evaluation}")

            return evaluation

        except Exception as e:
            logger.error(f"Error evaluating model: {e}")
            return {}

    def save_model(self, filepath: str) -> bool:
        """Save the trained model."""
        try:
            if self.model is None:
                logger.warning("No model to save")
                return False

            self.model.save(filepath)
            logger.info(f"Model saved to {filepath}")
            return True

        except Exception as e:
            logger.error(f"Error saving model: {e}")
            return False

    def load_model(self, filepath: str) -> bool:
        """Load a saved model."""
        try:
            self.model = tf.keras.models.load_model(filepath)
            logger.info(f"Model loaded from {filepath}")
            return True

        except Exception as e:
            logger.error(f"Error loading model from {filepath}: {e}")
            return False

    def get_model_summary(self) -> Optional[str]:
        """Get model architecture summary."""
        if self.model is None:
            return None

        import io
        import sys

        # Capture model summary
        old_stdout = sys.stdout
        sys.stdout = buffer = io.StringIO()

        try:
            self.model.summary()
            summary = buffer.getvalue()
        finally:
            sys.stdout = old_stdout

        return summary

    def get_training_history(self) -> Optional[Dict[str, list]]:
        """Get training history."""
        if self.history is None:
            return None

        return self.history.history
