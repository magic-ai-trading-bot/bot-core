import numpy as np
import pandas as pd
from typing import Dict, Optional, Tuple, List
from sklearn.preprocessing import StandardScaler
from config.config import config
from utils.logger import get_logger
from .technical_indicators import TechnicalIndicators


# @spec:FR-AI-006 - Feature Engineering
# @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md
# @test:TC-AI-015, TC-AI-016

logger = get_logger("FeatureEngineer")


class FeatureEngineer:
    """Feature engineering for cryptocurrency market data."""

    def __init__(self):
        self.config = config.get_model_config()
        self.trading_config = config.get_trading_config()
        self.technical_indicators = TechnicalIndicators()
        self.scaler = None
        self.feature_columns = []

    def prepare_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """Prepare features from raw OHLCV data."""
        logger.info("Starting feature preparation")

        # Calculate technical indicators
        enriched_df = self.technical_indicators.calculate_all_indicators(df)

        # Add price-based features
        enriched_df = self._add_price_features(enriched_df)

        # Add time-based features
        enriched_df = self._add_time_features(enriched_df)

        # Add lag features
        enriched_df = self._add_lag_features(enriched_df)

        # Add volatility features
        enriched_df = self._add_volatility_features(enriched_df)

        # Clean data
        enriched_df = self._clean_data(enriched_df)

        logger.info(f"Feature preparation completed. Final shape: {enriched_df.shape}")
        return enriched_df

    def _add_price_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """Add price-based features."""
        try:
            # Price returns (fill_method=None to avoid FutureWarning)
            df["price_return_1"] = df["close"].pct_change(1, fill_method=None)
            df["price_return_5"] = df["close"].pct_change(5, fill_method=None)
            df["price_return_10"] = df["close"].pct_change(10, fill_method=None)

            # Price position within range
            df["price_position"] = (df["close"] - df["low"]) / (df["high"] - df["low"])

            # High-Low spread
            df["hl_spread"] = (df["high"] - df["low"]) / df["close"]

            # Open-Close spread
            df["oc_spread"] = (df["close"] - df["open"]) / df["open"]

            # Volume-Price Trend
            df["vpt"] = (df["volume"] * df["price_return_1"]).cumsum()

            # Price momentum
            df["price_momentum_5"] = df["close"] / df["close"].shift(5) - 1
            df["price_momentum_10"] = df["close"] / df["close"].shift(10) - 1

            return df
        except Exception as e:
            logger.error(f"Error adding price features: {e}")
            return df

    def _add_time_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """Add time-based features."""
        try:
            # Extract time components
            df["hour"] = df.index.hour
            df["day_of_week"] = df.index.dayofweek
            df["day_of_month"] = df.index.day
            df["month"] = df.index.month

            # Cyclical encoding for time features
            df["hour_sin"] = np.sin(2 * np.pi * df["hour"] / 24)
            df["hour_cos"] = np.cos(2 * np.pi * df["hour"] / 24)
            df["day_sin"] = np.sin(2 * np.pi * df["day_of_week"] / 7)
            df["day_cos"] = np.cos(2 * np.pi * df["day_of_week"] / 7)
            df["month_sin"] = np.sin(2 * np.pi * df["month"] / 12)
            df["month_cos"] = np.cos(2 * np.pi * df["month"] / 12)

            # Drop original time features
            df.drop(["hour", "day_of_week", "day_of_month", "month"], axis=1, inplace=True)

            return df
        except Exception as e:
            logger.error(f"Error adding time features: {e}")
            return df

    def _add_lag_features(
        self, df: pd.DataFrame, lags: List[int] = [1, 2, 3, 5, 10]
    ) -> pd.DataFrame:
        """Add lag features for important indicators."""
        try:
            lag_columns = ["close", "volume", "rsi", "macd"]

            for col in lag_columns:
                if col in df.columns:
                    for lag in lags:
                        df[f"{col}_lag_{lag}"] = df[col].shift(lag)

            return df
        except Exception as e:
            logger.error(f"Error adding lag features: {e}")
            return df

    def _add_volatility_features(self, df: pd.DataFrame) -> pd.DataFrame:
        """Add volatility-based features."""
        try:
            # Rolling volatility
            df["volatility_5"] = df["price_return_1"].rolling(window=5).std()
            df["volatility_10"] = df["price_return_1"].rolling(window=10).std()
            df["volatility_20"] = df["price_return_1"].rolling(window=20).std()

            # Volatility ratios
            df["volatility_ratio_5_10"] = df["volatility_5"] / df["volatility_10"]
            df["volatility_ratio_10_20"] = df["volatility_10"] / df["volatility_20"]

            # Price dispersion
            df["price_dispersion"] = (df["high"] - df["low"]) / df["close"]

            return df
        except Exception as e:
            logger.error(f"Error adding volatility features: {e}")
            return df

    def _clean_data(self, df: pd.DataFrame) -> pd.DataFrame:
        """Clean and prepare data for modeling."""
        try:
            # Handle infinite values
            df.replace([np.inf, -np.inf], np.nan, inplace=True)

            # Forward fill missing values (updated to use ffill() instead of deprecated method)
            df.ffill(inplace=True)

            # Drop remaining NaN values
            df.dropna(inplace=True)

            return df
        except Exception as e:
            logger.error(f"Error cleaning data: {e}")
            return df

    def create_sequences(
        self, df: pd.DataFrame, sequence_length: Optional[int] = None
    ) -> Tuple[np.ndarray, np.ndarray]:
        """Create sequences for time series modeling."""
        sequence_length = sequence_length or self.config.get("sequence_length", 60)

        # Select feature columns (exclude target if it exists)
        feature_cols = [col for col in df.columns if col not in ["target", "signal"]]
        self.feature_columns = feature_cols

        # Prepare features
        features = df[feature_cols].values

        # Create target (next period price movement)
        target = self._create_target(df)

        # Create sequences
        X, y = [], []

        for i in range(sequence_length, len(features)):
            X.append(features[i - sequence_length : i])
            y.append(target[i])

        X = np.array(X)
        y = np.array(y)

        logger.info(f"Created sequences: X shape {X.shape}, y shape {y.shape}")
        return X, y

    def _create_target(self, df: pd.DataFrame) -> np.ndarray:
        """Create target variable for price direction prediction."""
        try:
            # Calculate future price movement
            future_return = df["close"].shift(-1) / df["close"] - 1

            # Create classification target
            # Note: Using fixed thresholds instead of config for now
            # long_threshold = self.trading_config.get("long_threshold", 0.6)
            # short_threshold = self.trading_config.get("short_threshold", 0.4)

            # Convert to probabilities for long position
            # 1.0 for strong upward movement, 0.0 for strong downward movement
            target = np.where(
                future_return > 0.005,
                1.0,  # Strong buy signal
                np.where(
                    future_return < -0.005,
                    0.0,  # Strong sell signal
                    0.5 + (future_return * 50),
                ),
            )  # Neutral to weak signals

            # Clip values to [0, 1] range
            target = np.clip(target, 0.0, 1.0)

            return target
        except Exception as e:
            logger.error(f"Error creating target: {e}")
            return np.zeros(len(df))

    def scale_features(self, X: np.ndarray, fit_scaler: bool = True) -> np.ndarray:
        """Scale features for neural network training."""
        try:
            if fit_scaler or self.scaler is None:
                self.scaler = StandardScaler()
                # Reshape for scaling
                original_shape = X.shape
                X_reshaped = X.reshape(-1, X.shape[-1])
                X_scaled = self.scaler.fit_transform(X_reshaped)
                X_scaled = X_scaled.reshape(original_shape)
                logger.info("Fitted new scaler and transformed features")
            else:
                # Transform using existing scaler
                original_shape = X.shape
                X_reshaped = X.reshape(-1, X.shape[-1])
                X_scaled = self.scaler.transform(X_reshaped)
                X_scaled = X_scaled.reshape(original_shape)
                logger.info("Transformed features using existing scaler")

            return X_scaled
        except Exception as e:
            logger.error(f"Error scaling features: {e}")
            return X

    def prepare_for_inference(self, df: pd.DataFrame) -> np.ndarray:
        """Prepare data for model inference."""
        try:
            # Prepare features
            processed_df = self.prepare_features(df)

            # Get the required sequence length
            sequence_length = self.config.get("sequence_length", 60)

            # Check if we have enough data
            if len(processed_df) < sequence_length:
                logger.warning(
                    f"Not enough data for inference. Required: {sequence_length}, Available: {len(processed_df)}"
                )
                return None

            # Select feature columns
            if not self.feature_columns:
                # If feature columns not set, use all except target columns
                feature_cols = [
                    col for col in processed_df.columns if col not in ["target", "signal"]
                ]
            else:
                feature_cols = self.feature_columns

            # Get the last sequence
            features = processed_df[feature_cols].values[-sequence_length:]

            # Reshape for model input
            X = features.reshape(1, sequence_length, -1)

            # Scale features
            if self.scaler is not None:
                X = self.scale_features(X, fit_scaler=False)

            return X
        except Exception as e:
            logger.error(f"Error preparing data for inference: {e}")
            return None

    def get_feature_importance(self, df: pd.DataFrame) -> Dict[str, float]:
        """Calculate feature importance based on correlation with target."""
        try:
            # Prepare features
            processed_df = self.prepare_features(df)

            # Create target
            target = self._create_target(processed_df)
            processed_df["target"] = target

            # Calculate correlations
            correlations = processed_df.corr()["target"].abs().sort_values(ascending=False)

            # Convert to dictionary
            feature_importance = correlations.drop("target").to_dict()

            return feature_importance
        except Exception as e:
            logger.error(f"Error calculating feature importance: {e}")
            return {}

    def get_feature_columns(self) -> List[str]:
        """Get list of feature columns."""
        return self.feature_columns

    def get_features_count(self) -> int:
        """Get number of features."""
        return len(self.feature_columns) if self.feature_columns else 0
