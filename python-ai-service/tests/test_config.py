"""Tests for config/config.py"""

import os
import tempfile
from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest
import yaml

from config.config import Config


class TestConfig:
    """Test Config class"""

    def setup_method(self):
        """Reset Config singleton before each test"""
        Config._instance = None
        Config._config = None

    def test_singleton_pattern(self):
        """Test that Config implements singleton pattern"""
        config1 = Config()
        config2 = Config()
        assert config1 is config2

    @patch("config.config.Path.exists")
    @patch("builtins.open")
    def test_load_config_from_yaml(self, mock_open, mock_exists):
        """Test loading configuration from YAML file"""
        mock_exists.return_value = True
        mock_config_data = {
            "server": {"host": "localhost", "port": 9000},
            "model": {"type": "gru"},
        }
        mock_open.return_value.__enter__.return_value.read.return_value = yaml.dump(
            mock_config_data
        )

        with patch("yaml.safe_load", return_value=mock_config_data):
            config = Config()
            assert config.get_server_config()["host"] == "localhost"
            assert config.get_server_config()["port"] == 9000

    @patch("config.config.Path.exists", return_value=False)
    def test_load_default_config_when_yaml_not_exists(self, mock_exists):
        """Test loading default configuration when YAML file doesn't exist"""
        config = Config()
        default_config = config._get_default_config()

        assert default_config["server"]["host"] == "0.0.0.0"
        assert default_config["server"]["port"] == 8000
        assert default_config["model"]["type"] == "lstm"

    @patch("config.config.Path.exists", return_value=False)
    @patch.dict(
        os.environ,
        {
            "SERVER_HOST": "custom.host",
            "SERVER_PORT": "9999",
            "MODEL_TYPE": "transformer",
            "LONG_THRESHOLD": "0.7",
            "SHORT_THRESHOLD": "0.3",
            "LOG_LEVEL": "DEBUG",
        },
    )
    def test_override_with_environment_variables(self, mock_exists):
        """Test that environment variables override config"""
        config = Config()

        assert config.get_server_config()["host"] == "custom.host"
        assert config.get_server_config()["port"] == 9999
        assert config.get_model_config()["type"] == "transformer"
        assert config.get_trading_config()["long_threshold"] == 0.7
        assert config.get_trading_config()["short_threshold"] == 0.3
        assert config.get_logging_config()["level"] == "DEBUG"

    @patch("config.config.Path.exists", return_value=False)
    @patch.dict(os.environ, {"MODEL_SAVE_PATH": "/custom/path"})
    def test_override_model_save_path(self, mock_exists):
        """Test overriding model save path from environment"""
        config = Config()
        assert config.get_model_management_config()["model_save_path"] == "/custom/path"

    @patch("config.config.Path.exists", return_value=False)
    def test_get_method_with_section_only(self, mock_exists):
        """Test get method with section only"""
        config = Config()
        server_config = config.get("server")
        assert server_config is not None
        assert "host" in server_config
        assert "port" in server_config

    @patch("config.config.Path.exists", return_value=False)
    def test_get_method_with_section_and_key(self, mock_exists):
        """Test get method with section and key"""
        config = Config()
        host = config.get("server", "host")
        assert host == "0.0.0.0"

    @patch("config.config.Path.exists", return_value=False)
    def test_get_method_with_default(self, mock_exists):
        """Test get method with default value"""
        config = Config()
        value = config.get("nonexistent", "key", default="default_value")
        assert value == "default_value"

    @patch("config.config.Path.exists", return_value=False)
    def test_get_server_config(self, mock_exists):
        """Test get_server_config method"""
        config = Config()
        server_config = config.get_server_config()
        assert server_config["host"] == "0.0.0.0"
        assert server_config["port"] == 8000
        assert "reload" in server_config

    @patch("config.config.Path.exists", return_value=False)
    def test_get_model_config(self, mock_exists):
        """Test get_model_config method"""
        config = Config()
        model_config = config.get_model_config()
        assert model_config["type"] == "lstm"
        assert model_config["sequence_length"] == 60
        assert model_config["hidden_size"] == 64

    @patch("config.config.Path.exists", return_value=False)
    def test_get_trading_config(self, mock_exists):
        """Test get_trading_config method"""
        config = Config()
        trading_config = config.get_trading_config()
        assert trading_config["long_threshold"] == 0.6
        assert trading_config["short_threshold"] == 0.4
        assert trading_config["neutral_zone"] == 0.1

    @patch("config.config.Path.exists", return_value=False)
    def test_get_indicators_config(self, mock_exists):
        """Test get_indicators_config method"""
        config = Config()
        indicators_config = config.get_indicators_config()
        assert indicators_config["rsi_period"] == 14
        assert indicators_config["macd_fast"] == 12
        assert indicators_config["bollinger_period"] == 20

    @patch("config.config.Path.exists", return_value=False)
    def test_get_data_config(self, mock_exists):
        """Test get_data_config method"""
        config = Config()
        data_config = config.get_data_config()
        assert "supported_timeframes" in data_config
        assert data_config["min_candles_required"] == 100
        assert data_config["max_candles_per_request"] == 1000

    @patch("config.config.Path.exists", return_value=False)
    def test_get_model_management_config(self, mock_exists):
        """Test get_model_management_config method"""
        config = Config()
        model_mgmt_config = config.get_model_management_config()
        assert model_mgmt_config["model_save_path"] == "./models/saved/"
        assert model_mgmt_config["retrain_interval_hours"] == 24
        assert model_mgmt_config["backup_count"] == 5

    @patch("config.config.Path.exists", return_value=False)
    def test_get_logging_config(self, mock_exists):
        """Test get_logging_config method"""
        config = Config()
        logging_config = config.get_logging_config()
        assert logging_config["level"] == "INFO"
        assert logging_config["rotation"] == "10 MB"
        assert logging_config["retention"] == "7 days"

    @patch("config.config.Path.exists", return_value=False)
    def test_get_supported_timeframes(self, mock_exists):
        """Test get_supported_timeframes method"""
        config = Config()
        timeframes = config.get_supported_timeframes()
        assert isinstance(timeframes, list)
        assert "1m" in timeframes
        assert "5m" in timeframes
        assert "1h" in timeframes
        assert "1d" in timeframes

    @patch("config.config.Path.exists", return_value=False)
    def test_is_valid_timeframe(self, mock_exists):
        """Test is_valid_timeframe method"""
        config = Config()
        assert config.is_valid_timeframe("1m") == True
        assert config.is_valid_timeframe("5m") == True
        assert config.is_valid_timeframe("1h") == True
        assert config.is_valid_timeframe("invalid") == False
        assert config.is_valid_timeframe("10m") == False

    @patch("config.config.Path.exists", return_value=False)
    def test_default_config_structure(self, mock_exists):
        """Test that default config has all required sections"""
        config = Config()
        default_config = config._get_default_config()

        required_sections = [
            "server",
            "model",
            "trading",
            "technical_indicators",
            "data",
            "ai_cache",
            "model_management",
            "logging",
        ]

        for section in required_sections:
            assert section in default_config

    @patch("config.config.Path.exists", return_value=False)
    @patch.dict(os.environ, {"SERVER_PORT": "invalid_port"})  # Invalid port number
    def test_override_with_invalid_port(self, mock_exists):
        """Test handling of invalid port in environment variable"""
        with pytest.raises(ValueError):
            config = Config()

    @patch("config.config.Path.exists", return_value=False)
    @patch.dict(os.environ, {"NEW_SECTION_KEY": "value"})
    def test_env_var_creates_new_section(self, mock_exists):
        """Test that overriding creates new section if it doesn't exist"""
        # This tests the code path where section is not in config
        config = Config()
        # Since NEW_SECTION_KEY is not in env_mappings, it won't be set
        # But we test that the code doesn't crash

    @patch("config.config.Path.exists", return_value=False)
    def test_get_with_nested_default(self, mock_exists):
        """Test get method with nested key that doesn't exist"""
        config = Config()
        result = config.get("server", "nonexistent_key", default="fallback")
        assert result == "fallback"

    @patch("config.config.Path.exists", return_value=False)
    def test_ai_cache_config(self, mock_exists):
        """Test AI cache configuration"""
        config = Config()
        ai_cache = config.get("ai_cache")
        assert ai_cache["enabled"] == True
        assert ai_cache["duration_minutes"] == 5
        assert ai_cache["max_entries"] == 100
