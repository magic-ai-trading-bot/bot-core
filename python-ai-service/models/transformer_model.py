from typing import Any, Dict, Optional, Tuple

import numpy as np
import tensorflow as tf
from tensorflow.keras.callbacks import (EarlyStopping, ModelCheckpoint,
                                        ReduceLROnPlateau)
from tensorflow.keras.layers import (  # @spec:FR-AI-003 - Transformer Model; @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md; @test:TC-AI-006, TC-AI-007
    Add, BatchNormalization, Dense, Dropout, GlobalAveragePooling1D, Input,
    LayerNormalization, MultiHeadAttention)
from tensorflow.keras.models import Model
from tensorflow.keras.optimizers import Adam

from config.config import config
from utils.logger import get_logger

logger = get_logger("TransformerModel")


class TransformerModel:
    """Transformer model for cryptocurrency price prediction."""

    def __init__(self):
        self.config = config.get_model_config()
        self.model = None
        self.history = None

    def transformer_encoder(self, inputs, head_size, num_heads, ff_dim, dropout=0):
        """Transformer encoder block."""
        # Attention and Normalization
        x = MultiHeadAttention(key_dim=head_size, num_heads=num_heads, dropout=dropout)(
            inputs, inputs
        )
        x = Dropout(dropout)(x)
        x = LayerNormalization(epsilon=1e-6)(x)
        res = Add()([x, inputs])

        # Feed Forward Network
        x = Dense(ff_dim, activation="relu")(res)
        x = Dropout(dropout)(x)
        x = Dense(inputs.shape[-1])(x)
        x = Dropout(dropout)(x)
        x = LayerNormalization(epsilon=1e-6)(x)

        return Add()([x, res])

    def build_model(self, input_shape: Tuple[int, int]) -> Model:
        """Build Transformer model architecture."""
        try:
            sequence_length, features_count = input_shape

            # Input layer
            inputs = Input(shape=(sequence_length, features_count))

            # Transformer parameters
            head_size = self.config.get("hidden_size", 64)
            num_heads = 4
            ff_dim = head_size * 2
            dropout_rate = self.config.get("dropout", 0.2)
            num_transformer_blocks = self.config.get("num_layers", 2)

            # Initial dense layer to adjust feature dimension if needed
            x = Dense(head_size)(inputs)

            # Multiple transformer encoder blocks
            for _ in range(num_transformer_blocks):
                x = self.transformer_encoder(
                    x, head_size, num_heads, ff_dim, dropout_rate
                )

            # Global pooling
            x = GlobalAveragePooling1D(data_format="channels_first")(x)

            # Classification head
            x = Dense(64, activation="relu")(x)
            x = BatchNormalization()(x)
            x = Dropout(dropout_rate)(x)

            x = Dense(32, activation="relu")(x)
            x = BatchNormalization()(x)
            x = Dropout(dropout_rate * 0.5)(x)

            # Output layer (sigmoid for probability output)
            outputs = Dense(1, activation="sigmoid")(x)

            # Create model
            model = Model(inputs, outputs)

            # Compile model
            model.compile(
                optimizer=Adam(learning_rate=self.config.get("learning_rate", 0.001)),
                loss="binary_crossentropy",
                metrics=["accuracy", "precision", "recall"],
            )

            logger.info(
                f"Transformer model built successfully with input shape: {input_shape}"
            )
            logger.info(f"Model parameters: {model.count_params()}")

            self.model = model
            return model

        except Exception as e:
            logger.error(f"Error building Transformer model: {e}")
            raise

    def train(
        self,
        X_train: np.ndarray,
        y_train: np.ndarray,
        X_val: Optional[np.ndarray] = None,
        y_val: Optional[np.ndarray] = None,
        save_path: Optional[str] = None,
    ) -> Dict[str, Any]:
        """Train the Transformer model."""
        try:
            if self.model is None:
                self.build_model((X_train.shape[1], X_train.shape[2]))

            # Prepare callbacks
            callbacks = []

            # Early stopping
            early_stopping = EarlyStopping(
                monitor="val_loss" if X_val is not None else "loss",
                patience=15,  # Transformer may need more patience
                restore_best_weights=True,
                verbose=1,
            )
            callbacks.append(early_stopping)

            # Reduce learning rate on plateau
            reduce_lr = ReduceLROnPlateau(
                monitor="val_loss" if X_val is not None else "loss",
                factor=0.2,
                patience=7,
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
            logger.info("Starting Transformer model training")

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

            logger.info(f"Transformer training completed. Results: {results}")
            return results

        except Exception as e:
            logger.error(f"Error training Transformer model: {e}")
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
