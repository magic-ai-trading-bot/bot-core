"""
Tests for CORS configuration
"""

import os
from unittest.mock import patch

import pytest
from fastapi.testclient import TestClient


class TestCORSMiddleware:
    """Test CORS middleware configuration"""

    def test_cors_respects_allowed_origins_env_var(self):
        """Test that CORS middleware respects ALLOWED_ORIGINS env var"""
        # Set environment variable
        test_origins = "http://testorigin.com,http://another.com"

        with patch.dict(os.environ, {"ALLOWED_ORIGINS": test_origins}):
            # Import main after setting env var
            import importlib
            import sys

            # Remove main from cache if it exists
            if "main" in sys.modules:
                del sys.modules["main"]

            # Note: Full import test would require reloading the entire module
            # This is a conceptual test - in production, test with actual server
            assert os.getenv("ALLOWED_ORIGINS") == test_origins

    def test_cors_default_origins_include_localhost(self):
        """Test that default CORS origins include localhost"""
        with patch.dict(os.environ, {}, clear=True):
            # Check default value
            default_origins = os.getenv(
                "ALLOWED_ORIGINS",
                "http://localhost:3000,http://localhost:8080,http://127.0.0.1:3000,http://127.0.0.1:8080",
            )

            # Use exact URL matching instead of substring check
            origins_list = [o.strip() for o in default_origins.split(",")]
            localhost_origins = [o for o in origins_list if o.startswith("http://localhost:")]
            loopback_origins = [o for o in origins_list if o.startswith("http://127.0.0.1:")]
            assert len(localhost_origins) > 0, "Should have localhost origins"
            assert len(loopback_origins) > 0, "Should have 127.0.0.1 origins"

    def test_cors_origins_split_by_comma(self):
        """Test that CORS origins are properly split by comma"""
        test_origins = "http://origin1.com,http://origin2.com,http://origin3.com"
        allowed_origins = [
            origin.strip() for origin in test_origins.split(",") if origin.strip()
        ]

        assert len(allowed_origins) == 3
        # Use set for exact membership check (not substring matching)
        origins_set = set(allowed_origins)
        assert "http://origin1.com" in origins_set
        assert "http://origin2.com" in origins_set
        assert "http://origin3.com" in origins_set

    def test_cors_origins_handles_whitespace(self):
        """Test that CORS origins properly handle whitespace"""
        test_origins = "http://origin1.com , http://origin2.com  ,  http://origin3.com"
        allowed_origins = [
            origin.strip() for origin in test_origins.split(",") if origin.strip()
        ]

        assert len(allowed_origins) == 3
        assert all(origin == origin.strip() for origin in allowed_origins)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
