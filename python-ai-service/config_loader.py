"""
Configuration loader for Python AI Service
Loads settings from config.yaml
"""

import yaml
import logging
from pathlib import Path
from typing import Dict, Any

logger = logging.getLogger(__name__)


def load_config(config_path: str = "config.yaml") -> Dict[str, Any]:
    """Load configuration from YAML file"""
    try:
        config_file = Path(config_path)
        if not config_file.exists():
            logger.warning(f"Config file {config_path} not found, using defaults")
            return get_default_config()

        with open(config_file, "r") as f:
            config = yaml.safe_load(f)

        logger.info(f"✅ Loaded configuration from {config_path}")
        return config

    except Exception as e:
        logger.error(f"❌ Failed to load config: {e}, using defaults")
        return get_default_config()


def get_default_config() -> Dict[str, Any]:
    """Default configuration fallback"""
    return {
        "ai_cache": {"enabled": True, "duration_minutes": 2, "max_entries": 100},
        "server": {"host": "0.0.0.0", "port": 8000},
        "signal": {
            "trend_threshold_percent": 0.8,  # Trend must be > 0.8% to count as bullish/bearish
            "min_required_timeframes": 3,  # Need 3/4 timeframes to agree for signal
            "min_required_indicators": 4,  # Need 4/5 indicators to agree per timeframe
            "confidence_base": 0.5,  # Base confidence for signals
            "confidence_per_timeframe": 0.08,  # Add 0.08 per agreeing timeframe (max 0.82)
        },
    }


# Load config on module import
CONFIG = load_config()

# Export commonly used values
AI_CACHE_DURATION_MINUTES = CONFIG.get("ai_cache", {}).get("duration_minutes", 2)
AI_CACHE_ENABLED = CONFIG.get("ai_cache", {}).get("enabled", True)
AI_CACHE_MAX_ENTRIES = CONFIG.get("ai_cache", {}).get("max_entries", 100)

# Signal generation config (used by both GPT-4 prompt and fallback code)
SIGNAL_TREND_THRESHOLD = CONFIG.get("signal", {}).get("trend_threshold_percent", 0.8)
SIGNAL_MIN_TIMEFRAMES = CONFIG.get("signal", {}).get("min_required_timeframes", 3)
SIGNAL_MIN_INDICATORS = CONFIG.get("signal", {}).get("min_required_indicators", 4)
SIGNAL_CONFIDENCE_BASE = CONFIG.get("signal", {}).get("confidence_base", 0.5)
SIGNAL_CONFIDENCE_PER_TF = CONFIG.get("signal", {}).get(
    "confidence_per_timeframe", 0.08
)

logger.info(
    f"📊 AI Cache Config: duration={AI_CACHE_DURATION_MINUTES}min, enabled={AI_CACHE_ENABLED}"
)
logger.info(
    f"📈 Signal Config: threshold={SIGNAL_TREND_THRESHOLD}%, min_timeframes={SIGNAL_MIN_TIMEFRAMES}, min_indicators={SIGNAL_MIN_INDICATORS}"
)
