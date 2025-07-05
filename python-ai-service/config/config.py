import os
import yaml
from typing import Dict, Any, List
from pathlib import Path
from dotenv import load_dotenv

# Load environment variables
load_dotenv()

class Config:
    """Central configuration management for the AI trading service."""
    
    _instance = None
    _config = None
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(Config, cls).__new__(cls)
            cls._instance._load_config()
        return cls._instance
    
    def _load_config(self) -> None:
        """Load configuration from YAML file and environment variables."""
        config_path = Path("config.yaml")
        
        # Load default config from YAML
        if config_path.exists():
            with open(config_path, "r") as f:
                self._config = yaml.safe_load(f)
        else:
            self._config = self._get_default_config()
        
        # Override with environment variables if they exist
        self._override_with_env()
    
    def _override_with_env(self) -> None:
        """Override configuration with environment variables."""
        env_mappings = {
            "SERVER_HOST": ("server", "host"),
            "SERVER_PORT": ("server", "port"),
            "MODEL_TYPE": ("model", "type"),
            "LONG_THRESHOLD": ("trading", "long_threshold"),
            "SHORT_THRESHOLD": ("trading", "short_threshold"),
            "LOG_LEVEL": ("logging", "level"),
            "MODEL_SAVE_PATH": ("model_management", "model_save_path"),
        }
        
        for env_var, (section, key) in env_mappings.items():
            env_value = os.getenv(env_var)
            if env_value:
                if section not in self._config:
                    self._config[section] = {}
                
                # Convert to appropriate type
                if key == "port":
                    self._config[section][key] = int(env_value)
                elif key in ["long_threshold", "short_threshold"]:
                    self._config[section][key] = float(env_value)
                else:
                    self._config[section][key] = env_value
    
    def _get_default_config(self) -> Dict[str, Any]:
        """Return default configuration if YAML file is not found."""
        return {
            "server": {"host": "0.0.0.0", "port": 8000, "reload": False},
            "model": {
                "type": "lstm",
                "sequence_length": 60,
                "features_count": 15,
                "hidden_size": 64,
                "num_layers": 2,
                "dropout": 0.2,
                "learning_rate": 0.001,
                "batch_size": 32,
                "epochs": 100,
                "validation_split": 0.2
            },
            "trading": {
                "long_threshold": 0.6,
                "short_threshold": 0.4,
                "neutral_zone": 0.1,
                "confidence_threshold": 0.55
            },
            "technical_indicators": {
                "rsi_period": 14,
                "macd_fast": 12,
                "macd_slow": 26,
                "macd_signal": 9,
                "ema_periods": [9, 21, 50],
                "bollinger_period": 20,
                "bollinger_std": 2,
                "volume_sma_period": 20
            },
            "data": {
                "supported_timeframes": ["1m", "5m", "15m", "1h", "4h", "1d"],
                "min_candles_required": 100,
                "max_candles_per_request": 1000
            },
            "ai_cache": {
                "enabled": True,
                "duration_minutes": 5,
                "max_entries": 100
            },
            "model_management": {
                "model_save_path": "./models/saved/",
                "retrain_interval_hours": 24,
                "backup_count": 5,
                "auto_retrain": True
            },
            "logging": {
                "level": "INFO",
                "format": "{time:YYYY-MM-DD HH:mm:ss} | {level} | {name}:{function}:{line} | {message}",
                "file": "./logs/trading_ai.log",
                "rotation": "10 MB",
                "retention": "7 days"
            }
        }
    
    def get(self, section: str, key: str = None, default: Any = None) -> Any:
        """Get configuration value."""
        if key is None:
            return self._config.get(section, default)
        return self._config.get(section, {}).get(key, default)
    
    def get_server_config(self) -> Dict[str, Any]:
        """Get server configuration."""
        return self._config.get("server", {})
    
    def get_model_config(self) -> Dict[str, Any]:
        """Get model configuration."""
        return self._config.get("model", {})
    
    def get_trading_config(self) -> Dict[str, Any]:
        """Get trading configuration."""
        return self._config.get("trading", {})
    
    def get_indicators_config(self) -> Dict[str, Any]:
        """Get technical indicators configuration."""
        return self._config.get("technical_indicators", {})
    
    def get_data_config(self) -> Dict[str, Any]:
        """Get data configuration."""
        return self._config.get("data", {})
    
    def get_model_management_config(self) -> Dict[str, Any]:
        """Get model management configuration."""
        return self._config.get("model_management", {})
    
    def get_logging_config(self) -> Dict[str, Any]:
        """Get logging configuration."""
        return self._config.get("logging", {})
    
    def get_supported_timeframes(self) -> List[str]:
        """Get supported timeframes."""
        return self.get_data_config().get("supported_timeframes", [])
    
    def is_valid_timeframe(self, timeframe: str) -> bool:
        """Check if timeframe is supported."""
        return timeframe in self.get_supported_timeframes()

# Singleton instance
config = Config() 