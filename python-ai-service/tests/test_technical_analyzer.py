"""
Test TechnicalAnalyzer class functionality.
"""

import os

# Import after adding to path in conftest
import sys
from datetime import datetime, timezone
from unittest.mock import MagicMock, patch

import numpy as np
import pandas as pd
import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


@pytest.fixture
def sample_klines():
    """Generate sample kline data for testing."""
    return [
        [
            1701234567000,
            "45000",
            "45500",
            "44800",
            "45200",
            "1000",
            1701238167000,
            "45000000",
            100,
            "500",
            "22500000",
            "0",
        ],
        [
            1701238167000,
            "45200",
            "45600",
            "45000",
            "45400",
            "1200",
            1701241767000,
            "54240000",
            120,
            "600",
            "27120000",
            "0",
        ],
        [
            1701241767000,
            "45400",
            "45800",
            "45200",
            "45600",
            "1100",
            1701245367000,
            "50160000",
            110,
            "550",
            "25080000",
            "0",
        ],
        [
            1701245367000,
            "45600",
            "46000",
            "45400",
            "45800",
            "1300",
            1701248967000,
            "59540000",
            130,
            "650",
            "29770000",
            "0",
        ],
        [
            1701248967000,
            "45800",
            "46200",
            "45600",
            "46000",
            "1400",
            1701252567000,
            "64400000",
            140,
            "700",
            "32200000",
            "0",
        ],
    ]


@pytest.fixture
def technical_analyzer():
    """Create TechnicalAnalyzer instance."""
    from main import TechnicalAnalyzer

    return TechnicalAnalyzer()


@pytest.mark.unit
class TestTechnicalAnalyzer:
    """Test TechnicalAnalyzer methods."""

    def test_prepare_dataframe(self, technical_analyzer, sample_klines):
        """Test DataFrame preparation from klines."""
        df = technical_analyzer.prepare_dataframe(sample_klines)

        assert isinstance(df, pd.DataFrame)
        assert len(df) == 5
        assert all(
            col in df.columns for col in ["open", "high", "low", "close", "volume"]
        )
        assert df["close"].iloc[-1] == 46000.0

    def test_calculate_indicators(self, technical_analyzer, sample_klines):
        """Test indicator calculation."""
        df = technical_analyzer.prepare_dataframe(sample_klines)
        indicators = technical_analyzer.calculate_indicators(df)

        # Check all required indicators are present
        required_indicators = [
            "rsi",
            "macd",
            "macd_signal",
            "macd_histogram",
            "bollinger_upper",
            "bollinger_middle",
            "bollinger_lower",
            "ema_9",
            "ema_21",
            "ema_50",
            "volume_sma",
            "atr",
            "adx",
            "stochastic_k",
            "stochastic_d",
        ]

        for indicator in required_indicators:
            assert indicator in indicators
            assert isinstance(indicators[indicator], (int, float))
            assert not np.isnan(indicators[indicator])

    def test_calculate_indicators_insufficient_data(self, technical_analyzer):
        """Test indicator calculation with insufficient data."""
        # Create minimal klines (only 2 candles)
        klines = [
            [
                1701234567000,
                "45000",
                "45500",
                "44800",
                "45200",
                "1000",
                1701238167000,
                "45000000",
                100,
                "500",
                "22500000",
                "0",
            ],
            [
                1701238167000,
                "45200",
                "45600",
                "45000",
                "45400",
                "1200",
                1701241767000,
                "54240000",
                120,
                "600",
                "27120000",
                "0",
            ],
        ]

        df = technical_analyzer.prepare_dataframe(klines)
        indicators = technical_analyzer.calculate_indicators(df)

        # Should still return indicators, but some may be NaN or default values
        assert isinstance(indicators, dict)
        assert "rsi" in indicators

    def test_detect_patterns(self, technical_analyzer, sample_klines):
        """Test pattern detection."""
        df = technical_analyzer.prepare_dataframe(sample_klines)
        patterns = technical_analyzer.detect_patterns(df)

        expected_patterns = [
            "double_top",
            "double_bottom",
            "head_shoulders",
            "ascending_triangle",
            "descending_triangle",
            "bullish_flag",
            "bearish_flag",
            "cup_handle",
        ]

        assert isinstance(patterns, dict)
        for pattern in expected_patterns:
            assert pattern in patterns
            assert isinstance(patterns[pattern], bool)

    def test_get_market_context(self, technical_analyzer, sample_klines):
        """Test market context generation."""
        df = technical_analyzer.prepare_dataframe(sample_klines)
        indicators = technical_analyzer.calculate_indicators(df)
        context = technical_analyzer.get_market_context(df, indicators)

        assert isinstance(context, dict)
        assert "trend_strength" in context
        assert "volatility" in context
        assert "volume_trend" in context
        assert "market_sentiment" in context

        # Validate value ranges
        assert -1.0 <= context["trend_strength"] <= 1.0
        assert 0.0 <= context["volatility"] <= 1.0
        assert context["volume_trend"] in ["increasing", "decreasing", "stable"]
        assert context["market_sentiment"] in ["bullish", "bearish", "neutral"]


@pytest.mark.unit
class TestEdgeCases:
    """Test edge cases for TechnicalAnalyzer."""

    def test_empty_klines(self, technical_analyzer):
        """Test with empty klines."""
        df = technical_analyzer.prepare_dataframe([])
        assert df.empty

        indicators = technical_analyzer.calculate_indicators(df)
        assert isinstance(indicators, dict)
        # Should have default values
        assert indicators["rsi"] == 50.0

    def test_single_candle(self, technical_analyzer):
        """Test with single candle."""
        klines = [
            [
                1701234567000,
                "45000",
                "45500",
                "44800",
                "45200",
                "1000",
                1701238167000,
                "45000000",
                100,
                "500",
                "22500000",
                "0",
            ]
        ]

        df = technical_analyzer.prepare_dataframe(klines)
        assert len(df) == 1

        indicators = technical_analyzer.calculate_indicators(df)
        patterns = technical_analyzer.detect_patterns(df)
        context = technical_analyzer.get_market_context(df, indicators)

        assert isinstance(indicators, dict)
        assert isinstance(patterns, dict)
        assert isinstance(context, dict)

    def test_extreme_values(self, technical_analyzer):
        """Test with extreme price values."""
        klines = [
            [
                1701234567000,
                "1",
                "1000000",
                "0.0001",
                "500000",
                "999999999",
                1701238167000,
                "45000000",
                100,
                "500",
                "22500000",
                "0",
            ],
            [
                1701238167000,
                "500000",
                "1000000",
                "1",
                "999999",
                "999999999",
                1701241767000,
                "54240000",
                120,
                "600",
                "27120000",
                "0",
            ],
        ]

        df = technical_analyzer.prepare_dataframe(klines)
        indicators = technical_analyzer.calculate_indicators(df)

        # Should handle extreme values without crashing
        assert isinstance(indicators, dict)
        assert all(key in indicators for key in ["rsi", "macd", "bollinger_upper"])
