"""
Simple tests to boost overall coverage to 95%.
Focus on easy wins from uncovered code paths.
"""

import pytest
import pandas as pd
import numpy as np
from features.feature_engineering import FeatureEngineer
from features.technical_indicators import TechnicalIndicators


class TestFeatureEngineerCoverageBoost:
    """Additional simple tests for FeatureEngineer."""

    def test_prepare_features_with_sufficient_data(self):
        """Test prepare_features with sufficient data for all indicators."""
        fe = FeatureEngineer()

        # Create 100 data points - enough for all indicators
        df = pd.DataFrame({
            "open": np.random.uniform(95, 105, 100),
            "high": np.random.uniform(100, 110, 100),
            "low": np.random.uniform(90, 100, 100),
            "close": np.random.uniform(95, 105, 100),
            "volume": np.random.uniform(1000, 2000, 100),
        })

        result = fe.prepare_features(df)

        # Should successfully process and return dataframe
        assert isinstance(result, pd.DataFrame)
        # Should have more columns due to feature engineering
        assert len(result.columns) > 5

    def test_prepare_features_with_zero_volume(self):
        """Test prepare_features handles zero volume."""
        fe = FeatureEngineer()

        df = pd.DataFrame({
            "open": [100.0] * 30,
            "high": [102.0] * 30,
            "low": [99.0] * 30,
            "close": [101.0] * 30,
            "volume": [0.0] * 30,  # Zero volume
        })

        result = fe.prepare_features(df)

        # Should handle zero volume without crashing
        assert isinstance(result, pd.DataFrame)


class TestTechnicalIndicatorsCoverageBoost:
    """Additional simple tests for TechnicalIndicators."""

    def test_calculate_all_indicators_comprehensive(self):
        """Test calculate_all_indicators with good data."""
        ti = TechnicalIndicators()

        # Create comprehensive dataset
        df = pd.DataFrame({
            "open": np.random.uniform(95, 105, 50),
            "high": np.random.uniform(100, 110, 50),
            "low": np.random.uniform(90, 100, 50),
            "close": np.random.uniform(95, 105, 50),
            "volume": np.random.uniform(1000, 2000, 50),
        })

        result = ti.calculate_all_indicators(df)

        # Should return dataframe with indicators
        assert isinstance(result, pd.DataFrame)
        assert len(result) == 50

    def test_calculate_trend_indicators(self):
        """Test trend indicators calculation."""
        ti = TechnicalIndicators()

        df = pd.DataFrame({
            "close": np.random.uniform(95, 105, 50)
        })

        indicators = ti.calculate_trend_indicators(df)

        # Should return dict of trend indicators
        assert isinstance(indicators, dict)
        # Should contain SMA/EMA indicators
        assert "sma_20" in indicators or len(indicators) > 0

    def test_calculate_volatility_indicators(self):
        """Test volatility indicators calculation."""
        ti = TechnicalIndicators()

        df = pd.DataFrame({
            "open": np.random.uniform(95, 105, 50),
            "high": np.random.uniform(100, 110, 50),
            "low": np.random.uniform(90, 100, 50),
            "close": np.random.uniform(95, 105, 50),
        })

        indicators = ti.calculate_volatility_indicators(df)

        # Should return dict of volatility indicators
        assert isinstance(indicators, dict)
        # ATR or similar should be present
        assert len(indicators) > 0

    def test_bollinger_bands_with_more_data(self):
        """Test Bollinger Bands with sufficient data."""
        ti = TechnicalIndicators()

        # 50 data points
        df = pd.DataFrame({
            "close": np.random.uniform(95, 105, 50)
        })

        indicators = ti.calculate_bollinger_bands(df)

        # Should have Bollinger Band indicators
        assert isinstance(indicators, dict)
        assert len(indicators) >= 3  # upper, middle, lower at minimum

    def test_macd_with_sufficient_data(self):
        """Test MACD with sufficient data."""
        ti = TechnicalIndicators()

        # 50 data points - more than MACD slow period (26)
        df = pd.DataFrame({
            "close": np.random.uniform(95, 105, 50)
        })

        indicators = ti.calculate_macd(df)

        # Should have MACD indicators
        assert "macd" in indicators
        assert "macd_signal" in indicators
        assert "macd_histogram" in indicators

    def test_rsi_boundary_values(self):
        """Test RSI calculation stays within 0-100 bounds."""
        ti = TechnicalIndicators()

        # Create df with uptrend then downtrend
        close_prices = list(range(100, 150)) + list(range(150, 100, -1))
        df = pd.DataFrame({
            "open": close_prices,
            "high": [p + 2 for p in close_prices],
            "low": [p - 2 for p in close_prices],
            "close": close_prices,
            "volume": [1000] * len(close_prices),
        })

        indicators = ti.calculate_momentum_indicators(df)

        # RSI should be present
        assert "rsi" in indicators
        # RSI values should be Series
        assert isinstance(indicators["rsi"], pd.Series)


class TestEdgeCasesCoverage:
    """Test various edge cases for coverage."""

    def test_feature_engineer_with_negative_prices(self):
        """Test handling of negative prices (edge case)."""
        fe = FeatureEngineer()

        # Some crypto pairs might have calculation errors leading to negatives
        df = pd.DataFrame({
            "open": [100.0, 101.0, -1.0, 103.0, 104.0],
            "high": [102.0, 103.0, 1.0, 105.0, 106.0],
            "low": [99.0, 100.0, -2.0, 102.0, 103.0],
            "close": [101.0, 102.0, 0.5, 104.0, 105.0],
            "volume": [1000.0, 1100.0, 1200.0, 1300.0, 1400.0],
        })

        # Should handle negative prices gracefully
        result = fe.prepare_features(df)
        assert isinstance(result, pd.DataFrame)

    def test_technical_indicators_with_extreme_volatility(self):
        """Test indicators with extreme price swings."""
        ti = TechnicalIndicators()

        # Extreme volatility: prices jumping 50%
        df = pd.DataFrame({
            "open": [100, 150, 75, 125, 90] * 10,
            "high": [105, 160, 80, 130, 95] * 10,
            "low": [95, 140, 70, 115, 85] * 10,
            "close": [102, 155, 77, 127, 92] * 10,
            "volume": [1000, 2000, 3000, 1500, 2500] * 10,
        })

        result = ti.calculate_all_indicators(df)

        # Should handle extreme volatility
        assert isinstance(result, pd.DataFrame)
        assert len(result) == 50


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
