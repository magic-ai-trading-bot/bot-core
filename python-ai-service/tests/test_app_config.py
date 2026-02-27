"""
Tests for app.core.config module to boost coverage.
"""

import os
from unittest.mock import patch

import pytest

from app.core.config import (
    AI_ANALYSIS_COLLECTION,
    ALLOWED_ORIGINS,
    ANALYSIS_INTERVAL_MINUTES,
    ANALYSIS_SYMBOLS,
    GROK_INPUT_COST_PER_1M,
    GROK_OUTPUT_COST_PER_1M,
    GROK_REQUEST_DELAY,
    get_mongodb_url,
    get_openai_api_keys,
)


class TestConfigConstants:
    """Test configuration constants."""

    def test_openai_request_delay_is_positive(self):
        """Test GROK_REQUEST_DELAY is positive."""
        assert GROK_REQUEST_DELAY > 0
        assert isinstance(GROK_REQUEST_DELAY, int)

    def test_gpt4o_mini_costs_are_positive(self):
        """Test GPT-4o-mini cost constants are positive."""
        assert GROK_INPUT_COST_PER_1M > 0
        assert GROK_OUTPUT_COST_PER_1M > 0
        assert isinstance(GROK_INPUT_COST_PER_1M, float)
        assert isinstance(GROK_OUTPUT_COST_PER_1M, float)

    def test_ai_analysis_collection_is_string(self):
        """Test AI_ANALYSIS_COLLECTION is a string."""
        assert isinstance(AI_ANALYSIS_COLLECTION, str)
        assert len(AI_ANALYSIS_COLLECTION) > 0

    def test_analysis_interval_is_positive(self):
        """Test ANALYSIS_INTERVAL_MINUTES is positive."""
        assert ANALYSIS_INTERVAL_MINUTES > 0
        assert isinstance(ANALYSIS_INTERVAL_MINUTES, int)

    def test_analysis_symbols_is_list(self):
        """Test ANALYSIS_SYMBOLS is a list."""
        assert isinstance(ANALYSIS_SYMBOLS, list)
        assert len(ANALYSIS_SYMBOLS) > 0
        # All items should be strings
        assert all(isinstance(symbol, str) for symbol in ANALYSIS_SYMBOLS)

    def test_allowed_origins_is_list(self):
        """Test ALLOWED_ORIGINS is a list."""
        assert isinstance(ALLOWED_ORIGINS, list)
        assert len(ALLOWED_ORIGINS) > 0
        # All items should be strings
        assert all(isinstance(origin, str) for origin in ALLOWED_ORIGINS)


class TestGetMongoDBUrl:
    """Test get_mongodb_url function."""

    @patch.dict(os.environ, {"DATABASE_URL": "mongodb://test:27017"}, clear=False)
    def test_get_mongodb_url_returns_url_when_set(self):
        """Test get_mongodb_url returns URL when DATABASE_URL is set."""
        url = get_mongodb_url()
        assert url == "mongodb://test:27017"

    @patch.dict(os.environ, {}, clear=False)
    def test_get_mongodb_url_raises_when_not_set(self):
        """Test get_mongodb_url raises ValueError when DATABASE_URL is not set."""
        # Remove DATABASE_URL if it exists
        if "DATABASE_URL" in os.environ:
            del os.environ["DATABASE_URL"]

        with pytest.raises(ValueError) as exc_info:
            get_mongodb_url()

        assert "DATABASE_URL environment variable is required" in str(exc_info.value)


class TestGetOpenAIAPIKeys:
    """Test get_openai_api_keys function."""

    @patch.dict(os.environ, {"OPENAI_API_KEY": "sk-test-primary"}, clear=False)
    def test_get_openai_api_keys_returns_primary_key(self):
        """Test get_openai_api_keys returns primary key when set."""
        # Clear fallback keys
        for i in range(1, 6):
            key_name = f"OPENAI_API_KEY_FALLBACK_{i}"
            if key_name in os.environ:
                del os.environ[key_name]

        keys = get_openai_api_keys()
        assert len(keys) == 1
        assert keys[0] == "sk-test-primary"

    @patch.dict(
        os.environ,
        {
            "OPENAI_API_KEY": "sk-test-primary",
            "OPENAI_API_KEY_FALLBACK_1": "sk-test-fallback-1",
            "OPENAI_API_KEY_FALLBACK_2": "sk-test-fallback-2",
        },
        clear=False,
    )
    def test_get_openai_api_keys_returns_multiple_keys(self):
        """Test get_openai_api_keys returns all configured keys."""
        # Clear other fallback keys
        for i in range(3, 6):
            key_name = f"OPENAI_API_KEY_FALLBACK_{i}"
            if key_name in os.environ:
                del os.environ[key_name]

        keys = get_openai_api_keys()
        assert len(keys) == 3
        assert "sk-test-primary" in keys
        assert "sk-test-fallback-1" in keys
        assert "sk-test-fallback-2" in keys

    @patch.dict(
        os.environ,
        {
            "OPENAI_API_KEY_FALLBACK_1": "sk-test-fallback-1",
            "OPENAI_API_KEY_FALLBACK_3": "sk-test-fallback-3",
        },
        clear=False,
    )
    def test_get_openai_api_keys_works_without_primary_key(self):
        """Test get_openai_api_keys works with only fallback keys."""
        # Remove primary key if it exists
        if "OPENAI_API_KEY" in os.environ:
            del os.environ["OPENAI_API_KEY"]

        # Clear other fallback keys
        for i in [2, 4, 5]:
            key_name = f"OPENAI_API_KEY_FALLBACK_{i}"
            if key_name in os.environ:
                del os.environ[key_name]

        keys = get_openai_api_keys()
        assert len(keys) == 2
        assert "sk-test-fallback-1" in keys
        assert "sk-test-fallback-3" in keys

    @patch.dict(os.environ, {}, clear=False)
    def test_get_openai_api_keys_raises_when_no_keys(self):
        """Test get_openai_api_keys raises ValueError when no keys are set."""
        # Remove all keys
        if "OPENAI_API_KEY" in os.environ:
            del os.environ["OPENAI_API_KEY"]
        for i in range(1, 6):
            key_name = f"OPENAI_API_KEY_FALLBACK_{i}"
            if key_name in os.environ:
                del os.environ[key_name]

        with pytest.raises(ValueError) as exc_info:
            get_openai_api_keys()

        assert "At least one OpenAI API key is required" in str(exc_info.value)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
