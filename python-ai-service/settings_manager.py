"""
Settings manager for Python AI Service
Fetches indicator and signal settings from Rust API

@spec:FR-SETTINGS-001 - Unified indicator settings
@spec:FR-SETTINGS-002 - Unified signal generation settings
@ref:docs/plans/251128-python-ai-settings-unification-plan.md

This module provides a centralized way to fetch and cache settings
from the Rust API. Settings are fetched on startup and refreshed
periodically to ensure Python AI service always uses the latest
configuration set by users via the frontend.

Architecture:
    Frontend UI → Rust API (validates & persists) → MongoDB
    Python AI Service ← Rust API (fetches & caches)

The Rust API is the single source of truth for all settings.
This ensures consistency across all services.
"""

import httpx
import logging
import os
from typing import Dict, Any, Optional
from datetime import datetime, timedelta
import asyncio

logger = logging.getLogger(__name__)


class SettingsManager:
    """
    Manages settings fetched from Rust API with caching.

    Features:
    - Fetches settings from Rust API on startup
    - Caches settings for fast access during calculations
    - Refreshes settings periodically (every 5 minutes)
    - Falls back to defaults if Rust API is unavailable
    - Thread-safe with asyncio lock
    """

    def __init__(
        self, rust_api_url: str = None, cache_duration_minutes: int = 5
    ):
        """
        Initialize settings manager.

        Args:
            rust_api_url: URL of the Rust API. Defaults to env var or docker compose name.
            cache_duration_minutes: How long to cache settings before refresh.
        """
        # Get URL from env or use default docker compose service name
        self.rust_api_url = rust_api_url or os.environ.get(
            "RUST_API_URL", "http://rust-core-engine:8080"
        )
        self.settings_cache: Optional[Dict[str, Any]] = None
        self.last_fetch: Optional[datetime] = None
        self.cache_duration = timedelta(minutes=cache_duration_minutes)
        self.lock = asyncio.Lock()
        self._initialized = False

        logger.info(f"📊 SettingsManager initialized with Rust API: {self.rust_api_url}")

    async def initialize(self) -> bool:
        """
        Initialize settings on startup.
        Returns True if settings were successfully fetched from Rust API.
        """
        if self._initialized:
            return True

        try:
            settings = await self.get_settings(force_refresh=True)
            self._initialized = True
            logger.info(f"✅ SettingsManager initialized successfully")
            logger.info(f"📊 RSI period: {settings.get('data', {}).get('indicators', {}).get('rsi_period', 'N/A')}")
            logger.info(f"📊 Trend threshold: {settings.get('data', {}).get('signal', {}).get('trend_threshold_percent', 'N/A')}%")
            return True
        except Exception as e:
            logger.error(f"❌ Failed to initialize SettingsManager: {e}")
            self._initialized = True  # Mark as initialized to avoid repeated attempts
            return False

    async def get_settings(self, force_refresh: bool = False) -> Dict[str, Any]:
        """
        Get settings with caching.

        Args:
            force_refresh: If True, bypass cache and fetch fresh settings.

        Returns:
            Dict containing 'indicators' and 'signal' settings.
        """
        async with self.lock:
            # Check cache
            if not force_refresh and self.settings_cache is not None:
                if self.last_fetch and datetime.now() - self.last_fetch < self.cache_duration:
                    logger.debug("✅ Using cached settings")
                    return self.settings_cache

            # Fetch from Rust API
            try:
                async with httpx.AsyncClient(timeout=10.0) as client:
                    response = await client.get(
                        f"{self.rust_api_url}/api/paper-trading/indicator-settings"
                    )
                    response.raise_for_status()

                    api_response = response.json()
                    # API wraps in ApiResponse {success, data, error, timestamp}
                    if api_response.get("success"):
                        settings = api_response.get("data", self._get_default_settings())
                    else:
                        logger.warning(f"⚠️ API returned error: {api_response.get('error')}")
                        settings = self._get_default_settings()

                    self.settings_cache = settings
                    self.last_fetch = datetime.now()

                    logger.info(f"✅ Fetched settings from Rust API")
                    logger.debug(f"📊 Settings: {settings}")
                    return {"success": True, "data": settings}

            except httpx.ConnectError as e:
                logger.warning(f"⚠️ Cannot connect to Rust API: {e}")
            except httpx.HTTPStatusError as e:
                logger.error(f"❌ Rust API returned error status: {e}")
            except Exception as e:
                logger.error(f"❌ Failed to fetch settings from Rust API: {e}")

            # Fallback to defaults if no cache
            if self.settings_cache is None:
                logger.warning("⚠️ Using default settings as fallback")
                return {"success": True, "data": self._get_default_settings()}

            # Return stale cache
            logger.warning("⚠️ Using stale cached settings")
            return {"success": True, "data": self.settings_cache}

    def _get_default_settings(self) -> Dict[str, Any]:
        """
        Default settings fallback.
        These values match the Rust defaults in settings.rs.
        """
        return {
            "indicators": {
                "rsi_period": 14,
                "macd_fast": 12,
                "macd_slow": 26,
                "macd_signal": 9,
                "ema_periods": [9, 21, 50],
                "bollinger_period": 20,
                "bollinger_std": 2.0,
                "volume_sma_period": 20,
                "stochastic_k_period": 14,
                "stochastic_d_period": 3,
            },
            "signal": {
                "trend_threshold_percent": 0.8,
                "min_required_timeframes": 3,
                "min_required_indicators": 4,
                "confidence_base": 0.5,
                "confidence_per_timeframe": 0.08,
            }
        }

    def get_indicator_value(self, key: str, default: Any = None) -> Any:
        """
        Get a specific indicator value synchronously.
        Safe to call from sync code - uses cached settings.

        Args:
            key: Setting key (e.g., 'rsi_period', 'macd_fast')
            default: Default value if key not found

        Returns:
            Setting value or default
        """
        if self.settings_cache is None:
            defaults = self._get_default_settings()
            return defaults.get("indicators", {}).get(key, default)
        return self.settings_cache.get("indicators", {}).get(key, default)

    def get_signal_value(self, key: str, default: Any = None) -> Any:
        """
        Get a specific signal value synchronously.
        Safe to call from sync code - uses cached settings.

        Args:
            key: Setting key (e.g., 'trend_threshold_percent')
            default: Default value if key not found

        Returns:
            Setting value or default
        """
        if self.settings_cache is None:
            defaults = self._get_default_settings()
            return defaults.get("signal", {}).get(key, default)
        return self.settings_cache.get("signal", {}).get(key, default)

    def get_all_indicator_settings(self) -> Dict[str, Any]:
        """
        Get all indicator settings.
        Returns cached settings or defaults.
        """
        if self.settings_cache is None:
            return self._get_default_settings()["indicators"]
        return self.settings_cache.get("indicators", self._get_default_settings()["indicators"])

    def get_all_signal_settings(self) -> Dict[str, Any]:
        """
        Get all signal generation settings.
        Returns cached settings or defaults.
        """
        if self.settings_cache is None:
            return self._get_default_settings()["signal"]
        return self.settings_cache.get("signal", self._get_default_settings()["signal"])


# Global singleton instance
# Used throughout the application for consistent settings access
settings_manager = SettingsManager()


async def refresh_settings_periodically():
    """
    Background task to refresh settings every 5 minutes.

    This ensures Python AI service picks up any changes made
    via the frontend without requiring a restart.
    """
    while True:
        try:
            await asyncio.sleep(300)  # 5 minutes
            result = await settings_manager.get_settings(force_refresh=True)
            if result.get("success"):
                logger.info("🔄 Settings refreshed successfully")
            else:
                logger.warning("⚠️ Settings refresh returned error")
        except asyncio.CancelledError:
            logger.info("🛑 Settings refresh task cancelled")
            break
        except Exception as e:
            logger.error(f"❌ Failed to refresh settings: {e}")
            # Continue loop - don't crash on refresh failure
