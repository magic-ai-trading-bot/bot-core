"""Tests for utils/helpers.py"""
import pytest
import pandas as pd
import tempfile
import shutil
from pathlib import Path
from datetime import datetime, timezone
from utils.helpers import (
    ensure_directory_exists,
    validate_ohlcv_data,
    convert_timeframe_to_minutes,
    create_dataframe_from_ohlcv,
    get_current_timestamp,
    calculate_percentage_change,
    format_confidence_score
)


class TestEnsureDirectoryExists:
    """Test ensure_directory_exists function"""

    def test_creates_directory(self):
        """Test that directory is created if it doesn't exist"""
        with tempfile.TemporaryDirectory() as tmpdir:
            test_path = Path(tmpdir) / "test_dir" / "nested"
            ensure_directory_exists(str(test_path))
            assert test_path.exists()
            assert test_path.is_dir()

    def test_existing_directory(self):
        """Test that no error occurs if directory already exists"""
        with tempfile.TemporaryDirectory() as tmpdir:
            ensure_directory_exists(tmpdir)
            # Should not raise any exception
            assert Path(tmpdir).exists()


class TestValidateOHLCVData:
    """Test validate_ohlcv_data function"""

    def test_valid_data(self):
        """Test with valid OHLCV data"""
        valid_data = {
            'open': 100,
            'high': 110,
            'low': 90,
            'close': 105,
            'volume': 1000,
            'timestamp': 1234567890,
            'candles': [
                {
                    'open': 100,
                    'high': 110,
                    'low': 90,
                    'close': 105,
                    'volume': 1000,
                    'timestamp': 1234567890
                }
            ]
        }
        assert validate_ohlcv_data(valid_data) == True

    def test_missing_required_fields(self):
        """Test with missing required fields"""
        invalid_data = {
            'open': 100,
            'high': 110,
            'candles': []
        }
        assert validate_ohlcv_data(invalid_data) == False

    def test_invalid_candles_structure(self):
        """Test with invalid candles structure"""
        invalid_data = {
            'open': 100,
            'high': 110,
            'low': 90,
            'close': 105,
            'volume': 1000,
            'timestamp': 1234567890,
            'candles': 'not_a_list'
        }
        assert validate_ohlcv_data(invalid_data) == False

    def test_invalid_price_range(self):
        """Test with invalid price range (high < low)"""
        invalid_data = {
            'open': 100,
            'high': 110,
            'low': 90,
            'close': 105,
            'volume': 1000,
            'timestamp': 1234567890,
            'candles': [
                {
                    'open': 100,
                    'high': 80,  # High less than low
                    'low': 90,
                    'close': 105,
                    'volume': 1000,
                    'timestamp': 1234567890
                }
            ]
        }
        assert validate_ohlcv_data(invalid_data) == False

    def test_negative_volume(self):
        """Test with negative volume"""
        invalid_data = {
            'open': 100,
            'high': 110,
            'low': 90,
            'close': 105,
            'volume': 1000,
            'timestamp': 1234567890,
            'candles': [
                {
                    'open': 100,
                    'high': 110,
                    'low': 90,
                    'close': 105,
                    'volume': -1000,  # Negative volume
                    'timestamp': 1234567890
                }
            ]
        }
        assert validate_ohlcv_data(invalid_data) == False

    def test_invalid_data_types(self):
        """Test with invalid data types"""
        invalid_data = {
            'open': 100,
            'high': 110,
            'low': 90,
            'close': 105,
            'volume': 1000,
            'timestamp': 1234567890,
            'candles': [
                {
                    'open': 'not_a_number',
                    'high': 110,
                    'low': 90,
                    'close': 105,
                    'volume': 1000,
                    'timestamp': 1234567890
                }
            ]
        }
        assert validate_ohlcv_data(invalid_data) == False


class TestConvertTimeframeToMinutes:
    """Test convert_timeframe_to_minutes function"""

    @pytest.mark.parametrize("timeframe,expected", [
        ('1m', 1),
        ('5m', 5),
        ('15m', 15),
        ('30m', 30),
        ('1h', 60),
        ('4h', 240),
        ('1d', 1440),
    ])
    def test_valid_timeframes(self, timeframe, expected):
        """Test conversion of valid timeframes"""
        assert convert_timeframe_to_minutes(timeframe) == expected

    def test_invalid_timeframe(self):
        """Test conversion of invalid timeframe returns default"""
        assert convert_timeframe_to_minutes('invalid') == 1


class TestCreateDataframeFromOHLCV:
    """Test create_dataframe_from_ohlcv function"""

    def test_valid_conversion(self):
        """Test successful conversion of OHLCV data to DataFrame"""
        data = {
            'candles': [
                {
                    'timestamp': 1609459200000,  # 2021-01-01
                    'open': 100.0,
                    'high': 110.0,
                    'low': 90.0,
                    'close': 105.0,
                    'volume': 1000.0
                },
                {
                    'timestamp': 1609545600000,  # 2021-01-02
                    'open': 105.0,
                    'high': 115.0,
                    'low': 95.0,
                    'close': 110.0,
                    'volume': 1500.0
                }
            ]
        }

        df = create_dataframe_from_ohlcv(data)

        assert df is not None
        assert isinstance(df, pd.DataFrame)
        assert len(df) == 2
        assert list(df.columns) == ['open', 'high', 'low', 'close', 'volume']
        assert df['open'].iloc[0] == 100.0
        assert df['close'].iloc[1] == 110.0

    def test_empty_candles(self):
        """Test with empty candles list"""
        data = {'candles': []}
        df = create_dataframe_from_ohlcv(data)
        assert df is None

    def test_missing_candles_key(self):
        """Test with missing candles key"""
        data = {}
        df = create_dataframe_from_ohlcv(data)
        assert df is None

    def test_invalid_data_format(self):
        """Test with invalid data format"""
        data = {
            'candles': [
                {
                    'timestamp': 'invalid',
                    'open': 'not_a_number',
                    'high': 110.0,
                    'low': 90.0,
                    'close': 105.0,
                    'volume': 1000.0
                }
            ]
        }
        df = create_dataframe_from_ohlcv(data)
        assert df is None


class TestGetCurrentTimestamp:
    """Test get_current_timestamp function"""

    def test_returns_iso_format(self):
        """Test that timestamp is in ISO format"""
        timestamp = get_current_timestamp()
        assert isinstance(timestamp, str)
        # Should be parseable as datetime
        parsed = datetime.fromisoformat(timestamp.replace('Z', '+00:00'))
        assert isinstance(parsed, datetime)

    def test_returns_utc_timezone(self):
        """Test that timestamp is in UTC timezone"""
        timestamp = get_current_timestamp()
        # ISO format should contain timezone info
        assert '+' in timestamp or 'Z' in timestamp or timestamp.endswith('+00:00')


class TestCalculatePercentageChange:
    """Test calculate_percentage_change function"""

    def test_positive_change(self):
        """Test positive percentage change"""
        result = calculate_percentage_change(100, 150)
        assert result == 50.0

    def test_negative_change(self):
        """Test negative percentage change"""
        result = calculate_percentage_change(100, 75)
        assert result == -25.0

    def test_no_change(self):
        """Test zero percentage change"""
        result = calculate_percentage_change(100, 100)
        assert result == 0.0

    def test_zero_old_value(self):
        """Test with zero old value"""
        result = calculate_percentage_change(0, 100)
        assert result == 0.0

    def test_floating_point_values(self):
        """Test with floating point values"""
        result = calculate_percentage_change(50.5, 75.75)
        assert abs(result - 50.0) < 0.1  # Allow small floating point error


class TestFormatConfidenceScore:
    """Test format_confidence_score function"""

    def test_formats_to_percentage(self):
        """Test conversion to percentage with 2 decimal places"""
        assert format_confidence_score(0.75) == 75.0
        assert format_confidence_score(0.5) == 50.0
        assert format_confidence_score(1.0) == 100.0

    def test_rounds_to_two_decimals(self):
        """Test rounding to 2 decimal places"""
        assert format_confidence_score(0.12345) == 12.35
        assert format_confidence_score(0.6789) == 67.89

    def test_handles_zero(self):
        """Test with zero value"""
        assert format_confidence_score(0.0) == 0.0
