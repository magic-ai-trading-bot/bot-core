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

        with open(config_file, 'r') as f:
            config = yaml.safe_load(f)

        logger.info(f"✅ Loaded configuration from {config_path}")
        return config

    except Exception as e:
        logger.error(f"❌ Failed to load config: {e}, using defaults")
        return get_default_config()


def get_default_config() -> Dict[str, Any]:
    """Default configuration fallback"""
    return {
        "ai_cache": {
            "enabled": True,
            "duration_minutes": 2,
            "max_entries": 100
        },
        "server": {
            "host": "0.0.0.0",
            "port": 8000
        }
    }


# Load config on module import
CONFIG = load_config()

# Export commonly used values
AI_CACHE_DURATION_MINUTES = CONFIG.get("ai_cache", {}).get("duration_minutes", 2)
AI_CACHE_ENABLED = CONFIG.get("ai_cache", {}).get("enabled", True)
AI_CACHE_MAX_ENTRIES = CONFIG.get("ai_cache", {}).get("max_entries", 100)

logger.info(f"📊 AI Cache Config: duration={AI_CACHE_DURATION_MINUTES}min, enabled={AI_CACHE_ENABLED}")
