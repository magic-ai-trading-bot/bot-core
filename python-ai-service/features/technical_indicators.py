import numpy as np
import pandas as pd
import ta
from typing import Dict, Any, Optional, Tuple
from config.config import config
from utils.logger import get_logger

logger = get_logger("TechnicalIndicators")


class TechnicalIndicators:
    """Technical indicators calculator for cryptocurrency market data."""

    def __init__(self):
        self.config = config.get_indicators_config()

    def calculate_rsi(
        self, df: pd.DataFrame, period: Optional[int] = None
    ) -> pd.Series:
        """Calculate Relative Strength Index (RSI)."""
        period = period or self.config.get("rsi_period", 14)
        try:
            return ta.momentum.RSIIndicator(close=df["close"], window=period).rsi()
        except Exception as e:
            logger.error(f"Error calculating RSI: {e}")
            return pd.Series(index=df.index, dtype=float)

    def calculate_macd(
        self,
        df: pd.DataFrame,
        fast: Optional[int] = None,
        slow: Optional[int] = None,
        signal: Optional[int] = None,
    ) -> Dict[str, pd.Series]:
        """Calculate MACD (Moving Average Convergence Divergence)."""
        fast = fast or self.config.get("macd_fast", 12)
        slow = slow or self.config.get("macd_slow", 26)
        signal = signal or self.config.get("macd_signal", 9)

        try:
            macd_indicator = ta.trend.MACD(
                close=df["close"],
                window_fast=fast,
                window_slow=slow,
                window_sign=signal,
            )
            return {
                "macd": macd_indicator.macd(),
                "macd_signal": macd_indicator.macd_signal(),
                "macd_histogram": macd_indicator.macd_diff(),
            }
        except Exception as e:
            logger.error(f"Error calculating MACD: {e}")
            return {
                "macd": pd.Series(index=df.index, dtype=float),
                "macd_signal": pd.Series(index=df.index, dtype=float),
                "macd_histogram": pd.Series(index=df.index, dtype=float),
            }

    def calculate_ema(
        self, df: pd.DataFrame, periods: Optional[list] = None
    ) -> Dict[str, pd.Series]:
        """Calculate Exponential Moving Averages for multiple periods."""
        periods = periods or self.config.get("ema_periods", [9, 21, 50])
        emas = {}

        for period in periods:
            try:
                emas[f"ema_{period}"] = ta.trend.EMAIndicator(
                    close=df["close"], window=period
                ).ema_indicator()
            except Exception as e:
                logger.error(f"Error calculating EMA {period}: {e}")
                emas[f"ema_{period}"] = pd.Series(index=df.index, dtype=float)

        return emas

    def calculate_bollinger_bands(
        self,
        df: pd.DataFrame,
        period: Optional[int] = None,
        std_dev: Optional[float] = None,
    ) -> Dict[str, pd.Series]:
        """Calculate Bollinger Bands."""
        period = period or self.config.get("bollinger_period", 20)
        std_dev = std_dev or self.config.get("bollinger_std", 2)

        try:
            bb_indicator = ta.volatility.BollingerBands(
                close=df["close"], window=period, window_dev=std_dev
            )
            return {
                "bb_upper": bb_indicator.bollinger_hband(),
                "bb_middle": bb_indicator.bollinger_mavg(),
                "bb_lower": bb_indicator.bollinger_lband(),
                "bb_width": bb_indicator.bollinger_wband(),
                "bb_percent": bb_indicator.bollinger_pband(),
            }
        except Exception as e:
            logger.error(f"Error calculating Bollinger Bands: {e}")
            return {
                "bb_upper": pd.Series(index=df.index, dtype=float),
                "bb_middle": pd.Series(index=df.index, dtype=float),
                "bb_lower": pd.Series(index=df.index, dtype=float),
                "bb_width": pd.Series(index=df.index, dtype=float),
                "bb_percent": pd.Series(index=df.index, dtype=float),
            }

    def calculate_volume_indicators(
        self, df: pd.DataFrame, period: Optional[int] = None
    ) -> Dict[str, pd.Series]:
        """Calculate volume-based indicators."""
        period = period or self.config.get("volume_sma_period", 20)

        try:
            indicators = {}

            # Volume SMA
            indicators["volume_sma"] = df["volume"].rolling(window=period).mean()

            # Volume Weighted Average Price (VWAP)
            indicators["vwap"] = ta.volume.VolumeSMAIndicator(
                close=df["close"], volume=df["volume"], window=period
            ).volume_sma()

            # On Balance Volume (OBV)
            indicators["obv"] = ta.volume.OnBalanceVolumeIndicator(
                close=df["close"], volume=df["volume"]
            ).on_balance_volume()

            # Volume Rate of Change
            indicators["volume_roc"] = df["volume"].pct_change(periods=period) * 100

            return indicators
        except Exception as e:
            logger.error(f"Error calculating volume indicators: {e}")
            return {
                "volume_sma": pd.Series(index=df.index, dtype=float),
                "vwap": pd.Series(index=df.index, dtype=float),
                "obv": pd.Series(index=df.index, dtype=float),
                "volume_roc": pd.Series(index=df.index, dtype=float),
            }

    def calculate_stochastic(
        self, df: pd.DataFrame, k_period: int = 14, d_period: int = 3
    ) -> Dict[str, pd.Series]:
        """Calculate Stochastic Oscillator."""
        try:
            stoch_indicator = ta.momentum.StochasticOscillator(
                high=df["high"],
                low=df["low"],
                close=df["close"],
                window=k_period,
                smooth_window=d_period,
            )
            return {
                "stoch_k": stoch_indicator.stoch(),
                "stoch_d": stoch_indicator.stoch_signal(),
            }
        except Exception as e:
            logger.error(f"Error calculating Stochastic: {e}")
            return {
                "stoch_k": pd.Series(index=df.index, dtype=float),
                "stoch_d": pd.Series(index=df.index, dtype=float),
            }

    def calculate_atr(self, df: pd.DataFrame, period: int = 14) -> pd.Series:
        """Calculate Average True Range (ATR)."""
        try:
            return ta.volatility.AverageTrueRange(
                high=df["high"], low=df["low"], close=df["close"], window=period
            ).average_true_range()
        except Exception as e:
            logger.error(f"Error calculating ATR: {e}")
            return pd.Series(index=df.index, dtype=float)

    def detect_price_patterns(self, df: pd.DataFrame) -> Dict[str, pd.Series]:
        """Detect basic price action patterns."""
        try:
            patterns = {}

            # Support and Resistance levels (simplified)
            window = 20
            patterns["local_high"] = (
                df["high"].rolling(window=window, center=True).max() == df["high"]
            )
            patterns["local_low"] = (
                df["low"].rolling(window=window, center=True).min() == df["low"]
            )

            # Price breakouts
            patterns["breakout_high"] = df["close"] > df["high"].rolling(
                window=20
            ).max().shift(1)
            patterns["breakout_low"] = df["close"] < df["low"].rolling(
                window=20
            ).min().shift(1)

            # Doji patterns (simplified)
            body_size = abs(df["close"] - df["open"])
            candle_range = df["high"] - df["low"]
            patterns["doji"] = body_size < (candle_range * 0.1)

            # Hammer patterns (simplified)
            lower_shadow = df[["open", "close"]].min(axis=1) - df["low"]
            upper_shadow = df["high"] - df[["open", "close"]].max(axis=1)
            patterns["hammer"] = (lower_shadow > 2 * body_size) & (
                upper_shadow < body_size
            )

            return patterns
        except Exception as e:
            logger.error(f"Error detecting price patterns: {e}")
            return {}

    def calculate_momentum_indicators(self, df: pd.DataFrame) -> Dict[str, pd.Series]:
        """Calculate additional momentum indicators."""
        try:
            indicators = {}

            # Rate of Change (ROC)
            indicators["roc"] = ta.momentum.ROCIndicator(
                close=df["close"], window=12
            ).roc()

            # Williams %R
            indicators["williams_r"] = ta.momentum.WilliamsRIndicator(
                high=df["high"], low=df["low"], close=df["close"], window=14
            ).williams_r()

            # Commodity Channel Index (CCI)
            indicators["cci"] = ta.trend.CCIIndicator(
                high=df["high"], low=df["low"], close=df["close"], window=20
            ).cci()

            return indicators
        except Exception as e:
            logger.error(f"Error calculating momentum indicators: {e}")
            return {}

    def calculate_all_indicators(self, df: pd.DataFrame) -> pd.DataFrame:
        """Calculate all technical indicators and return enriched DataFrame."""
        logger.info("Calculating all technical indicators")

        # Create a copy of the original DataFrame
        enriched_df = df.copy()

        try:
            # RSI
            enriched_df["rsi"] = self.calculate_rsi(df)

            # MACD
            macd_data = self.calculate_macd(df)
            for key, value in macd_data.items():
                enriched_df[key] = value

            # EMAs
            ema_data = self.calculate_ema(df)
            for key, value in ema_data.items():
                enriched_df[key] = value

            # Bollinger Bands
            bb_data = self.calculate_bollinger_bands(df)
            for key, value in bb_data.items():
                enriched_df[key] = value

            # Volume indicators
            volume_data = self.calculate_volume_indicators(df)
            for key, value in volume_data.items():
                enriched_df[key] = value

            # Stochastic
            stoch_data = self.calculate_stochastic(df)
            for key, value in stoch_data.items():
                enriched_df[key] = value

            # ATR
            enriched_df["atr"] = self.calculate_atr(df)

            # Price patterns
            pattern_data = self.detect_price_patterns(df)
            for key, value in pattern_data.items():
                enriched_df[key] = value.astype(int)  # Convert boolean to int

            # Momentum indicators
            momentum_data = self.calculate_momentum_indicators(df)
            for key, value in momentum_data.items():
                enriched_df[key] = value

            logger.info(
                f"Successfully calculated indicators. DataFrame shape: {enriched_df.shape}"
            )
            return enriched_df

        except Exception as e:
            logger.error(f"Error calculating all indicators: {e}")
            return enriched_df
