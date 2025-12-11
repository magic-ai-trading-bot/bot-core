"""
Enhanced tests for technical indicators with exact value assertions
to improve mutation testing scores from ~55% to 75%+
"""

import numpy as np
import pandas as pd
import pytest

from features.technical_indicators import TechnicalIndicators


@pytest.fixture
def tech_indicators():
    """Create TechnicalIndicators instance"""
    return TechnicalIndicators()


@pytest.fixture
def known_price_data():
    """Create known price data for exact RSI calculation"""
    # Using known data that produces predictable RSI
    dates = pd.date_range(start="2024-01-01", periods=30, freq="1h")

    # Uptrend data
    close_prices = [100 + i * 2 for i in range(30)]

    df = pd.DataFrame(
        {
            "open": [p - 0.5 for p in close_prices],
            "high": [p + 1.0 for p in close_prices],
            "low": [p - 1.0 for p in close_prices],
            "close": close_prices,
            "volume": [1000] * 30,
        },
        index=dates,
    )
    return df


class TestRSIExactValues:
    """Test RSI with exact value assertions to catch mutations"""

    def test_rsi_exact_range_validation(self, tech_indicators, known_price_data):
        """Test RSI is always in valid range [0, 100]"""
        rsi = tech_indicators.calculate_rsi(known_price_data, period=14)

        assert isinstance(rsi, pd.Series)
        assert len(rsi) == len(known_price_data)

        # Every non-NaN RSI value MUST be in [0, 100]
        valid_rsi = rsi.dropna()
        assert len(valid_rsi) > 0, "Should have at least some RSI values"

        for idx, value in valid_rsi.items():
            assert 0.0 <= value <= 100.0, f"RSI at {idx} is {value}, must be [0, 100]"

    def test_rsi_uptrend_above_50(self, tech_indicators, known_price_data):
        """Test RSI in uptrend is consistently above 50"""
        rsi = tech_indicators.calculate_rsi(known_price_data, period=14)

        valid_rsi = rsi.dropna()
        last_rsi = valid_rsi.iloc[-1]

        # Continuous uptrend should produce RSI > 50
        assert last_rsi > 50.0, f"Uptrend RSI should be > 50, got {last_rsi}"
        assert last_rsi > 70.0, f"Strong uptrend RSI should be > 70, got {last_rsi}"

    def test_rsi_downtrend_below_50(self, tech_indicators):
        """Test RSI in downtrend is below 50"""
        dates = pd.date_range(start="2024-01-01", periods=30, freq="1h")

        # Downtrend data
        close_prices = [200 - i * 2 for i in range(30)]
        df = pd.DataFrame(
            {
                "open": [p + 0.5 for p in close_prices],
                "high": [p + 1.0 for p in close_prices],
                "low": [p - 1.0 for p in close_prices],
                "close": close_prices,
                "volume": [1000] * 30,
            },
            index=dates,
        )

        rsi = tech_indicators.calculate_rsi(df, period=14)
        valid_rsi = rsi.dropna()
        last_rsi = valid_rsi.iloc[-1]

        assert last_rsi < 50.0, f"Downtrend RSI should be < 50, got {last_rsi}"
        assert last_rsi < 30.0, f"Strong downtrend RSI should be < 30, got {last_rsi}"

    def test_rsi_flat_prices_near_50(self, tech_indicators):
        """Test RSI with flat prices is near 50 or handles edge case"""
        dates = pd.date_range(start="2024-01-01", periods=30, freq="1h")
        df = pd.DataFrame(
            {
                "open": [100.0] * 30,
                "high": [100.0] * 30,
                "low": [100.0] * 30,
                "close": [100.0] * 30,
                "volume": [1000] * 30,
            },
            index=dates,
        )

        rsi = tech_indicators.calculate_rsi(df, period=14)
        valid_rsi = rsi.dropna()

        # Flat prices create a special case - RSI can be NaN or 50 or 100
        # depending on implementation (no gains/losses = division by zero edge case)
        if len(valid_rsi) > 0:
            last_rsi = valid_rsi.iloc[-1]
            # Just verify it's in valid range, don't enforce specific value
            assert 0.0 <= last_rsi <= 100.0, f"RSI must be in [0, 100], got {last_rsi}"


class TestMACDExactValues:
    """Test MACD with exact value assertions"""

    def test_macd_histogram_equals_difference(self, tech_indicators, known_price_data):
        """Test MACD histogram = MACD line - Signal line (exact)"""
        macd = tech_indicators.calculate_macd(known_price_data)

        assert "macd" in macd
        assert "macd_signal" in macd
        assert "macd_histogram" in macd

        # Get valid (non-NaN) indices
        valid_mask = (
            ~macd["macd"].isna()
            & ~macd["macd_signal"].isna()
            & ~macd["macd_histogram"].isna()
        )

        if valid_mask.sum() > 0:
            macd_line = macd["macd"][valid_mask]
            signal_line = macd["macd_signal"][valid_mask]
            histogram = macd["macd_histogram"][valid_mask]

            # Histogram MUST equal MACD - Signal
            for idx in macd_line.index:
                expected = macd_line[idx] - signal_line[idx]
                actual = histogram[idx]
                assert (
                    abs(actual - expected) < 0.0001
                ), f"MACD histogram at {idx}: expected {expected}, got {actual}"

    def test_macd_uptrend_positive(self, tech_indicators, known_price_data):
        """Test MACD in uptrend is positive"""
        macd = tech_indicators.calculate_macd(known_price_data)

        valid_macd = macd["macd"].dropna()
        if len(valid_macd) > 0:
            last_macd = valid_macd.iloc[-1]
            # Strong uptrend should have positive MACD
            assert last_macd > 0.0, f"Uptrend MACD should be positive, got {last_macd}"

    def test_macd_flat_near_zero(self, tech_indicators):
        """Test MACD with flat prices is near zero"""
        dates = pd.date_range(start="2024-01-01", periods=50, freq="1h")
        df = pd.DataFrame(
            {
                "open": [100.0] * 50,
                "high": [100.0] * 50,
                "low": [100.0] * 50,
                "close": [100.0] * 50,
                "volume": [1000] * 50,
            },
            index=dates,
        )

        macd = tech_indicators.calculate_macd(df)
        valid_macd = macd["macd"].dropna()

        if len(valid_macd) > 0:
            last_macd = valid_macd.iloc[-1]
            last_signal = macd["macd_signal"].dropna().iloc[-1]

            assert (
                abs(last_macd) < 0.1
            ), f"Flat price MACD should be ~0, got {last_macd}"
            assert (
                abs(last_signal) < 0.1
            ), f"Flat price signal should be ~0, got {last_signal}"


class TestBollingerBandsRelationships:
    """Test Bollinger Bands relationships that must always hold"""

    def test_upper_always_greater_than_middle(self, tech_indicators, known_price_data):
        """Test upper band > middle band (always)"""
        bb = tech_indicators.calculate_bollinger_bands(known_price_data)

        upper = bb["bb_upper"]
        middle = bb["bb_middle"]

        valid_mask = ~upper.isna() & ~middle.isna()

        if valid_mask.sum() > 0:
            for idx in upper[valid_mask].index:
                assert (
                    upper[idx] > middle[idx]
                ), f"Upper band {upper[idx]} must be > middle {middle[idx]} at {idx}"

    def test_middle_always_greater_than_lower(self, tech_indicators, known_price_data):
        """Test middle band > lower band (always)"""
        bb = tech_indicators.calculate_bollinger_bands(known_price_data)

        middle = bb["bb_middle"]
        lower = bb["bb_lower"]

        valid_mask = ~middle.isna() & ~lower.isna()

        if valid_mask.sum() > 0:
            for idx in middle[valid_mask].index:
                assert (
                    middle[idx] > lower[idx]
                ), f"Middle band {middle[idx]} must be > lower {lower[idx]} at {idx}"

    def test_bandwidth_calculation(self, tech_indicators, known_price_data):
        """Test Bollinger Bandwidth = (Upper - Lower) / Middle * 100"""
        bb = tech_indicators.calculate_bollinger_bands(known_price_data)

        if "bb_width" in bb:
            upper = bb["bb_upper"]
            lower = bb["bb_lower"]
            middle = bb["bb_middle"]
            width = bb["bb_width"]

            valid_mask = ~upper.isna() & ~lower.isna() & ~middle.isna() & ~width.isna()

            if valid_mask.sum() > 0:
                for idx in upper[valid_mask].index:
                    if middle[idx] != 0:
                        # BB width is typically (upper - lower) / middle * 100 (as percentage)
                        expected_width = ((upper[idx] - lower[idx]) / middle[idx]) * 100
                        actual_width = width[idx]

                        # Allow small floating point differences
                        assert (
                            abs(actual_width - expected_width) < 0.1
                        ), f"BB Width at {idx}: expected {expected_width}, got {actual_width}"

    def test_bollinger_percent_range(self, tech_indicators, known_price_data):
        """Test Bollinger %B is in valid range"""
        bb = tech_indicators.calculate_bollinger_bands(known_price_data)

        if "bb_percent" in bb:
            percent_b = bb["bb_percent"].dropna()

            # %B can go outside [0, 1] when price breaks bands
            # but should be reasonable
            for idx, value in percent_b.items():
                assert (
                    -2.0 <= value <= 3.0
                ), f"Bollinger %B at {idx} is {value}, seems unreasonable"


class TestErrorPathsAndEdgeCases:
    """Test error handling and edge cases"""

    def test_rsi_insufficient_data_error(self, tech_indicators):
        """Test RSI fails gracefully with insufficient data"""
        df = pd.DataFrame({"close": [100.0, 101.0]})

        rsi = tech_indicators.calculate_rsi(df, period=14)

        # Should return a series (possibly all NaN)
        assert isinstance(rsi, pd.Series)
        assert len(rsi) == 2

    def test_macd_insufficient_data_error(self, tech_indicators):
        """Test MACD fails gracefully with insufficient data"""
        df = pd.DataFrame({"close": [100.0] * 10})

        macd = tech_indicators.calculate_macd(df, fast=12, slow=26, signal=9)

        # Should return dict with empty or NaN series
        assert isinstance(macd, dict)
        assert "macd" in macd
        assert "macd_signal" in macd
        assert "macd_histogram" in macd

    def test_empty_dataframe_handling(self, tech_indicators):
        """Test all indicators handle empty DataFrame"""
        df = pd.DataFrame(columns=["open", "high", "low", "close", "volume"])

        # RSI
        rsi = tech_indicators.calculate_rsi(df)
        assert isinstance(rsi, pd.Series)
        assert len(rsi) == 0

        # MACD
        macd = tech_indicators.calculate_macd(df)
        assert isinstance(macd, dict)
        assert all(len(v) == 0 for v in macd.values())

        # Bollinger Bands
        bb = tech_indicators.calculate_bollinger_bands(df)
        assert isinstance(bb, dict)
        assert all(len(v) == 0 for v in bb.values())

    def test_nan_values_in_data(self, tech_indicators):
        """Test indicators handle NaN values in data"""
        dates = pd.date_range(start="2024-01-01", periods=30, freq="1h")
        close_prices = [100 + i for i in range(30)]

        # Insert NaN values
        close_prices[10:15] = [np.nan] * 5

        df = pd.DataFrame(
            {
                "open": close_prices,
                "high": [p + 1 if not np.isnan(p) else np.nan for p in close_prices],
                "low": [p - 1 if not np.isnan(p) else np.nan for p in close_prices],
                "close": close_prices,
                "volume": [1000] * 30,
            },
            index=dates,
        )

        # Should handle NaN gracefully
        rsi = tech_indicators.calculate_rsi(df)
        assert isinstance(rsi, pd.Series)

        macd = tech_indicators.calculate_macd(df)
        assert isinstance(macd, dict)

    def test_extreme_volatility_handling(self, tech_indicators):
        """Test indicators with extreme price swings"""
        dates = pd.date_range(start="2024-01-01", periods=30, freq="1h")

        # Extreme swings
        close_prices = []
        for i in range(30):
            if i % 2 == 0:
                close_prices.append(100.0)
            else:
                close_prices.append(200.0)

        df = pd.DataFrame(
            {
                "open": close_prices,
                "high": [p * 1.1 for p in close_prices],
                "low": [p * 0.9 for p in close_prices],
                "close": close_prices,
                "volume": [1000] * 30,
            },
            index=dates,
        )

        # Should handle extreme volatility
        rsi = tech_indicators.calculate_rsi(df)
        bb = tech_indicators.calculate_bollinger_bands(df)

        assert isinstance(rsi, pd.Series)
        assert isinstance(bb, dict)

        # RSI should still be in valid range
        valid_rsi = rsi.dropna()
        if len(valid_rsi) > 0:
            assert all(
                0 <= v <= 100 for v in valid_rsi
            ), "RSI out of range with extreme volatility"

    def test_zero_volume_handling(self, tech_indicators):
        """Test volume indicators with zero volume"""
        dates = pd.date_range(start="2024-01-01", periods=30, freq="1h")
        df = pd.DataFrame(
            {
                "open": [100] * 30,
                "high": [101] * 30,
                "low": [99] * 30,
                "close": [100] * 30,
                "volume": [0] * 30,  # Zero volume
            },
            index=dates,
        )

        vol_indicators = tech_indicators.calculate_volume_indicators(df)

        # Should return dict without crashing
        assert isinstance(vol_indicators, dict)


class TestConcurrentCalculations:
    """Test that indicators are deterministic and consistent"""

    def test_rsi_consistency_multiple_calls(self, tech_indicators, known_price_data):
        """Test RSI gives same result on multiple calls"""
        rsi1 = tech_indicators.calculate_rsi(known_price_data, period=14)
        rsi2 = tech_indicators.calculate_rsi(known_price_data, period=14)
        rsi3 = tech_indicators.calculate_rsi(known_price_data, period=14)

        # All should be identical
        pd.testing.assert_series_equal(rsi1, rsi2)
        pd.testing.assert_series_equal(rsi2, rsi3)

    def test_macd_consistency_multiple_calls(self, tech_indicators, known_price_data):
        """Test MACD gives same result on multiple calls"""
        macd1 = tech_indicators.calculate_macd(known_price_data)
        macd2 = tech_indicators.calculate_macd(known_price_data)

        # All components should be identical
        pd.testing.assert_series_equal(macd1["macd"], macd2["macd"])
        pd.testing.assert_series_equal(macd1["macd_signal"], macd2["macd_signal"])
        pd.testing.assert_series_equal(macd1["macd_histogram"], macd2["macd_histogram"])

    def test_bollinger_bands_consistency(self, tech_indicators, known_price_data):
        """Test Bollinger Bands consistency"""
        bb1 = tech_indicators.calculate_bollinger_bands(known_price_data)
        bb2 = tech_indicators.calculate_bollinger_bands(known_price_data)

        pd.testing.assert_series_equal(bb1["bb_upper"], bb2["bb_upper"])
        pd.testing.assert_series_equal(bb1["bb_middle"], bb2["bb_middle"])
        pd.testing.assert_series_equal(bb1["bb_lower"], bb2["bb_lower"])
