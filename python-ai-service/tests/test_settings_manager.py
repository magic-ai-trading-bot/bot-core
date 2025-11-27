"""
Tests for settings_manager.py

@spec:FR-SETTINGS-001 - Unified indicator settings
@spec:FR-SETTINGS-002 - Unified signal generation settings
"""

import pytest
import asyncio
from unittest.mock import patch, AsyncMock, MagicMock
from datetime import datetime, timedelta
import httpx

from settings_manager import (
    SettingsManager,
    settings_manager,
    refresh_settings_periodically,
)


@pytest.mark.unit
class TestSettingsManagerInit:
    """Test SettingsManager initialization."""

    def test_init_with_default_url(self):
        """Test initialization with default URL."""
        with patch.dict("os.environ", {}, clear=True):
            sm = SettingsManager()
            assert sm.rust_api_url == "http://rust-core-engine:8080"
            assert sm.settings_cache is None
            assert sm.last_fetch is None
            assert sm._initialized is False

    def test_init_with_env_url(self):
        """Test initialization with environment variable URL."""
        with patch.dict("os.environ", {"RUST_API_URL": "http://custom-api:9000"}):
            sm = SettingsManager()
            assert sm.rust_api_url == "http://custom-api:9000"

    def test_init_with_explicit_url(self):
        """Test initialization with explicit URL parameter."""
        sm = SettingsManager(rust_api_url="http://explicit:8080")
        assert sm.rust_api_url == "http://explicit:8080"

    def test_init_with_custom_cache_duration(self):
        """Test initialization with custom cache duration."""
        sm = SettingsManager(cache_duration_minutes=10)
        assert sm.cache_duration == timedelta(minutes=10)


@pytest.mark.unit
class TestSettingsManagerGetDefaultSettings:
    """Test _get_default_settings method."""

    def test_default_settings_structure(self):
        """Test default settings have correct structure."""
        sm = SettingsManager()
        defaults = sm._get_default_settings()

        assert "indicators" in defaults
        assert "signal" in defaults

    def test_default_indicator_settings(self):
        """Test default indicator settings values."""
        sm = SettingsManager()
        defaults = sm._get_default_settings()
        indicators = defaults["indicators"]

        assert indicators["rsi_period"] == 14
        assert indicators["macd_fast"] == 12
        assert indicators["macd_slow"] == 26
        assert indicators["macd_signal"] == 9
        assert indicators["ema_periods"] == [9, 21, 50]
        assert indicators["bollinger_period"] == 20
        assert indicators["bollinger_std"] == 2.0
        assert indicators["volume_sma_period"] == 20
        assert indicators["stochastic_k_period"] == 14
        assert indicators["stochastic_d_period"] == 3

    def test_default_signal_settings(self):
        """Test default signal settings values."""
        sm = SettingsManager()
        defaults = sm._get_default_settings()
        signal = defaults["signal"]

        assert signal["trend_threshold_percent"] == 0.8
        assert signal["min_required_timeframes"] == 3
        assert signal["min_required_indicators"] == 4
        assert signal["confidence_base"] == 0.5
        assert signal["confidence_per_timeframe"] == 0.08


@pytest.mark.unit
class TestSettingsManagerGetIndicatorValue:
    """Test get_indicator_value method."""

    def test_get_indicator_value_no_cache(self):
        """Test get_indicator_value without cache uses defaults."""
        sm = SettingsManager()
        assert sm.get_indicator_value("rsi_period") == 14
        assert sm.get_indicator_value("macd_fast") == 12

    def test_get_indicator_value_with_cache(self):
        """Test get_indicator_value with cache returns cached value."""
        sm = SettingsManager()
        sm.settings_cache = {
            "indicators": {"rsi_period": 20, "macd_fast": 15},
            "signal": {}
        }
        assert sm.get_indicator_value("rsi_period") == 20
        assert sm.get_indicator_value("macd_fast") == 15

    def test_get_indicator_value_missing_key_returns_default(self):
        """Test get_indicator_value with missing key returns default."""
        sm = SettingsManager()
        assert sm.get_indicator_value("nonexistent", default=42) == 42

    def test_get_indicator_value_missing_key_no_default(self):
        """Test get_indicator_value with missing key and no default."""
        sm = SettingsManager()
        assert sm.get_indicator_value("nonexistent") is None


@pytest.mark.unit
class TestSettingsManagerGetSignalValue:
    """Test get_signal_value method."""

    def test_get_signal_value_no_cache(self):
        """Test get_signal_value without cache uses defaults."""
        sm = SettingsManager()
        assert sm.get_signal_value("trend_threshold_percent") == 0.8
        assert sm.get_signal_value("min_required_timeframes") == 3

    def test_get_signal_value_with_cache(self):
        """Test get_signal_value with cache returns cached value."""
        sm = SettingsManager()
        sm.settings_cache = {
            "indicators": {},
            "signal": {"trend_threshold_percent": 1.0, "min_required_timeframes": 4}
        }
        assert sm.get_signal_value("trend_threshold_percent") == 1.0
        assert sm.get_signal_value("min_required_timeframes") == 4

    def test_get_signal_value_missing_key_returns_default(self):
        """Test get_signal_value with missing key returns default."""
        sm = SettingsManager()
        assert sm.get_signal_value("nonexistent", default=99) == 99


@pytest.mark.unit
class TestSettingsManagerGetAllSettings:
    """Test get_all_indicator_settings and get_all_signal_settings."""

    def test_get_all_indicator_settings_no_cache(self):
        """Test get_all_indicator_settings without cache."""
        sm = SettingsManager()
        indicators = sm.get_all_indicator_settings()
        assert indicators["rsi_period"] == 14
        assert "macd_fast" in indicators

    def test_get_all_indicator_settings_with_cache(self):
        """Test get_all_indicator_settings with cache."""
        sm = SettingsManager()
        sm.settings_cache = {
            "indicators": {"rsi_period": 21, "custom": "value"},
            "signal": {}
        }
        indicators = sm.get_all_indicator_settings()
        assert indicators["rsi_period"] == 21
        assert indicators["custom"] == "value"

    def test_get_all_signal_settings_no_cache(self):
        """Test get_all_signal_settings without cache."""
        sm = SettingsManager()
        signal = sm.get_all_signal_settings()
        assert signal["trend_threshold_percent"] == 0.8
        assert "min_required_timeframes" in signal

    def test_get_all_signal_settings_with_cache(self):
        """Test get_all_signal_settings with cache."""
        sm = SettingsManager()
        sm.settings_cache = {
            "indicators": {},
            "signal": {"trend_threshold_percent": 1.5, "custom": "value"}
        }
        signal = sm.get_all_signal_settings()
        assert signal["trend_threshold_percent"] == 1.5
        assert signal["custom"] == "value"


@pytest.mark.asyncio
@pytest.mark.unit
class TestSettingsManagerInitialize:
    """Test initialize method."""

    async def test_initialize_already_initialized(self):
        """Test initialize when already initialized returns True."""
        sm = SettingsManager()
        sm._initialized = True
        result = await sm.initialize()
        assert result is True

    async def test_initialize_success(self):
        """Test successful initialization."""
        sm = SettingsManager()

        mock_response = MagicMock()
        mock_response.json.return_value = {
            "success": True,
            "data": {
                "indicators": {"rsi_period": 14},
                "signal": {"trend_threshold_percent": 0.8}
            }
        }
        mock_response.raise_for_status = MagicMock()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                return_value=mock_response
            )
            result = await sm.initialize()

        assert result is True
        assert sm._initialized is True

    async def test_initialize_failure(self):
        """Test initialization failure still returns True (fallback to defaults)."""
        sm = SettingsManager()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                side_effect=Exception("Connection failed")
            )
            result = await sm.initialize()

        # Initialize returns True even on failure (uses defaults as fallback)
        assert result is True
        assert sm._initialized is True


@pytest.mark.asyncio
@pytest.mark.unit
class TestSettingsManagerGetSettings:
    """Test get_settings method."""

    async def test_get_settings_cache_hit(self):
        """Test get_settings returns cached settings when valid."""
        sm = SettingsManager()
        sm.settings_cache = {"indicators": {"rsi_period": 14}, "signal": {}}
        sm.last_fetch = datetime.now()

        result = await sm.get_settings()
        assert result == sm.settings_cache

    async def test_get_settings_cache_expired(self):
        """Test get_settings fetches new settings when cache expired."""
        sm = SettingsManager()
        sm.settings_cache = {"indicators": {"rsi_period": 14}, "signal": {}}
        sm.last_fetch = datetime.now() - timedelta(minutes=10)  # Expired

        mock_response = MagicMock()
        mock_response.json.return_value = {
            "success": True,
            "data": {"indicators": {"rsi_period": 20}, "signal": {}}
        }
        mock_response.raise_for_status = MagicMock()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                return_value=mock_response
            )
            result = await sm.get_settings()

        assert result["data"]["indicators"]["rsi_period"] == 20

    async def test_get_settings_force_refresh(self):
        """Test get_settings with force_refresh bypasses cache."""
        sm = SettingsManager()
        sm.settings_cache = {"indicators": {"rsi_period": 14}, "signal": {}}
        sm.last_fetch = datetime.now()

        mock_response = MagicMock()
        mock_response.json.return_value = {
            "success": True,
            "data": {"indicators": {"rsi_period": 25}, "signal": {}}
        }
        mock_response.raise_for_status = MagicMock()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                return_value=mock_response
            )
            result = await sm.get_settings(force_refresh=True)

        assert result["data"]["indicators"]["rsi_period"] == 25

    async def test_get_settings_api_error_response(self):
        """Test get_settings handles API error response."""
        sm = SettingsManager()

        mock_response = MagicMock()
        mock_response.json.return_value = {
            "success": False,
            "error": "Internal error"
        }
        mock_response.raise_for_status = MagicMock()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                return_value=mock_response
            )
            result = await sm.get_settings(force_refresh=True)

        # Should use default settings
        assert result["success"] is True
        assert result["data"]["indicators"]["rsi_period"] == 14

    async def test_get_settings_connect_error_no_cache(self):
        """Test get_settings handles connection error without cache."""
        sm = SettingsManager()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                side_effect=httpx.ConnectError("Connection refused")
            )
            result = await sm.get_settings(force_refresh=True)

        # Should use default settings
        assert result["success"] is True
        assert result["data"]["indicators"]["rsi_period"] == 14

    async def test_get_settings_connect_error_with_stale_cache(self):
        """Test get_settings returns stale cache on connection error."""
        sm = SettingsManager()
        sm.settings_cache = {"indicators": {"rsi_period": 30}, "signal": {}}
        sm.last_fetch = datetime.now() - timedelta(minutes=10)

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                side_effect=httpx.ConnectError("Connection refused")
            )
            result = await sm.get_settings(force_refresh=True)

        # Should use stale cache
        assert result["success"] is True
        assert result["data"]["indicators"]["rsi_period"] == 30

    async def test_get_settings_http_status_error(self):
        """Test get_settings handles HTTP status error."""
        sm = SettingsManager()

        mock_request = MagicMock()
        mock_response = MagicMock()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                side_effect=httpx.HTTPStatusError(
                    "Server error", request=mock_request, response=mock_response
                )
            )
            result = await sm.get_settings(force_refresh=True)

        # Should use default settings
        assert result["success"] is True

    async def test_get_settings_generic_exception(self):
        """Test get_settings handles generic exception."""
        sm = SettingsManager()

        with patch("httpx.AsyncClient") as mock_client:
            mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                side_effect=Exception("Unknown error")
            )
            result = await sm.get_settings(force_refresh=True)

        # Should use default settings
        assert result["success"] is True


@pytest.mark.asyncio
@pytest.mark.unit
class TestRefreshSettingsPeriodically:
    """Test refresh_settings_periodically function."""

    async def test_refresh_settings_periodically_cancelled(self):
        """Test refresh_settings_periodically handles cancellation gracefully."""
        # The function catches CancelledError and breaks, so it exits cleanly
        with patch("asyncio.sleep", new_callable=AsyncMock) as mock_sleep:
            mock_sleep.side_effect = asyncio.CancelledError()
            # Function should exit without raising
            await refresh_settings_periodically()

    async def test_refresh_settings_periodically_success(self):
        """Test refresh_settings_periodically refreshes settings."""
        sm = SettingsManager()

        mock_response = MagicMock()
        mock_response.json.return_value = {
            "success": True,
            "data": {"indicators": {"rsi_period": 14}, "signal": {}}
        }
        mock_response.raise_for_status = MagicMock()

        call_count = 0

        async def mock_sleep_fn(seconds):
            nonlocal call_count
            call_count += 1
            if call_count >= 2:
                raise asyncio.CancelledError()

        with patch("settings_manager.settings_manager", sm):
            with patch("httpx.AsyncClient") as mock_client:
                mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                    return_value=mock_response
                )
                with patch("asyncio.sleep", side_effect=mock_sleep_fn):
                    # Function should exit cleanly after cancellation
                    await refresh_settings_periodically()

    async def test_refresh_settings_periodically_error_continues(self):
        """Test refresh_settings_periodically continues after error."""
        sm = SettingsManager()
        call_count = 0

        async def mock_sleep_fn(seconds):
            nonlocal call_count
            call_count += 1
            if call_count >= 2:
                raise asyncio.CancelledError()

        with patch("settings_manager.settings_manager", sm):
            with patch("httpx.AsyncClient") as mock_client:
                mock_client.return_value.__aenter__.return_value.get = AsyncMock(
                    side_effect=Exception("Refresh failed")
                )
                with patch("asyncio.sleep", side_effect=mock_sleep_fn):
                    # Function should continue after error and exit on cancellation
                    await refresh_settings_periodically()


@pytest.mark.unit
class TestGlobalSettingsManager:
    """Test global settings_manager singleton."""

    def test_global_settings_manager_exists(self):
        """Test global settings_manager singleton is created."""
        assert settings_manager is not None
        assert isinstance(settings_manager, SettingsManager)

    def test_global_settings_manager_default_url(self):
        """Test global settings_manager uses default URL."""
        # The global instance should have the default URL
        assert "rust-core-engine" in settings_manager.rust_api_url or "RUST_API_URL" in str(settings_manager.rust_api_url)
