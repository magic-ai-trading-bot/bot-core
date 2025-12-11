"""Tests for utils/logger.py"""

from pathlib import Path
from unittest.mock import MagicMock, patch

import pytest
from loguru import logger

from utils.logger import get_logger, setup_logger


class TestSetupLogger:
    """Test setup_logger function"""

    @patch("utils.logger.config")
    def test_setup_logger_with_defaults(self, mock_config):
        """Test logger setup with default configuration"""
        mock_config.get_logging_config.return_value = {
            "file": "./logs/test.log",
            "format": "{time} | {level} | {message}",
            "level": "INFO",
            "rotation": "10 MB",
            "retention": "7 days",
        }

        # Reset logger state
        logger.remove()

        setup_logger()

        # Verify config was called
        mock_config.get_logging_config.assert_called_once()

    @patch("utils.logger.config")
    def test_setup_logger_creates_log_directory(self, mock_config):
        """Test that log directory is created"""
        import os
        import tempfile

        with tempfile.TemporaryDirectory() as tmpdir:
            log_file = os.path.join(tmpdir, "nested", "test.log")
            mock_config.get_logging_config.return_value = {
                "file": log_file,
                "format": "{time} | {level} | {message}",
                "level": "DEBUG",
                "rotation": "10 MB",
                "retention": "7 days",
            }

            # Reset logger state
            logger.remove()

            setup_logger()

            # Check that directory was created
            assert Path(log_file).parent.exists()

    @patch("utils.logger.config")
    def test_setup_logger_with_different_levels(self, mock_config):
        """Test logger setup with different log levels"""
        for level in ["DEBUG", "INFO", "WARNING", "ERROR"]:
            mock_config.get_logging_config.return_value = {
                "file": "./logs/test.log",
                "format": "{time} | {level} | {message}",
                "level": level,
                "rotation": "10 MB",
                "retention": "7 days",
            }

            logger.remove()
            setup_logger()

            # Should not raise any exceptions
            assert True


class TestGetLogger:
    """Test get_logger function"""

    def test_get_logger_without_name(self):
        """Test getting logger without name"""
        log = get_logger()
        assert log is not None

    def test_get_logger_with_name(self):
        """Test getting logger with name"""
        log = get_logger("TestModule")
        assert log is not None

    def test_get_logger_returns_logger_instance(self):
        """Test that get_logger returns a logger instance"""
        log = get_logger("TestModule")
        # Should have logger methods
        assert hasattr(log, "info")
        assert hasattr(log, "debug")
        assert hasattr(log, "warning")
        assert hasattr(log, "error")

    def test_get_logger_with_different_names(self):
        """Test getting loggers with different names"""
        log1 = get_logger("Module1")
        log2 = get_logger("Module2")

        # Both should be valid logger instances
        assert log1 is not None
        assert log2 is not None
