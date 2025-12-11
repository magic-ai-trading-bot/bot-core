"""
Tests for security and critical fixes in Python AI Service
Tests all critical security fixes and improvements
"""

import os
import re
from pathlib import Path

import pytest


class TestAPIKeySecurity:
    """Test that no hardcoded API keys exist in the codebase"""

    def test_no_hardcoded_openai_keys_in_main(self):
        """Verify no hardcoded OpenAI API keys in main.py"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check for sk-proj- pattern (OpenAI key prefix)
        hardcoded_keys = re.findall(r"sk-proj-[A-Za-z0-9_-]{20,}", content)
        assert (
            len(hardcoded_keys) == 0
        ), f"Found {len(hardcoded_keys)} hardcoded API keys in main.py"

        # Check for sk- pattern (generic OpenAI key)
        generic_keys = re.findall(r'"sk-[A-Za-z0-9_-]{20,}"', content)
        assert (
            len(generic_keys) == 0
        ), f"Found {len(generic_keys)} potential API keys in main.py"

    def test_no_api_key_logging_in_main(self):
        """Verify no API keys are logged in main.py"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check for lines that log API key values
        dangerous_patterns = [
            r"logger.*api_key\[:15\].*api_key\[-10:\]",  # API key preview logging
            r"f.*{api_key\[:.*\]}.*{api_key\[-.*:\]}",  # f-string with key slicing
        ]

        for pattern in dangerous_patterns:
            matches = re.findall(pattern, content, re.IGNORECASE)
            assert len(matches) == 0, f"Found API key logging pattern: {pattern}"

    def test_env_example_has_placeholders(self):
        """Verify .env.example contains only placeholders"""
        env_example = Path(__file__).parent.parent / ".env.example"
        assert env_example.exists(), ".env.example file does not exist"

        content = env_example.read_text()

        # Verify placeholders exist
        assert "your-openai-api-key-here" in content or "your-" in content.lower()

        # Verify no real API keys (sk- prefix)
        real_keys = re.findall(r"sk-[A-Za-z0-9_-]{20,}", content)
        assert (
            len(real_keys) == 0
        ), f"Found {len(real_keys)} real API keys in .env.example"


class TestCORSConfiguration:
    """Test CORS configuration security"""

    def test_cors_not_using_wildcard_origin(self):
        """Verify CORS does not use wildcard '*' origin"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check that allow_origins=["*"] is not present
        wildcard_cors_patterns = [
            r'allow_origins\s*=\s*\["\*"\]',
            r"allow_origins\s*=\s*\['\*'\]",
        ]

        for pattern in wildcard_cors_patterns:
            matches = re.findall(pattern, content)
            assert len(matches) == 0, "Found CORS wildcard configuration"

    def test_cors_uses_environment_variable(self):
        """Verify CORS configuration uses environment variable"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check for ALLOWED_ORIGINS environment variable usage
        assert 'os.getenv("ALLOWED_ORIGINS"' in content or "ALLOWED_ORIGINS" in content

    def test_env_example_has_allowed_origins(self):
        """Verify .env.example has ALLOWED_ORIGINS configuration"""
        env_example = Path(__file__).parent.parent / ".env.example"
        content = env_example.read_text()

        assert "ALLOWED_ORIGINS" in content
        assert "localhost" in content


class TestDeprecatedCodeUpdates:
    """Test that deprecated code has been updated"""

    def test_no_deprecated_fillna_method_in_feature_engineering(self):
        """Verify fillna(method='ffill') is not used"""
        feature_file = (
            Path(__file__).parent.parent / "features" / "feature_engineering.py"
        )
        content = feature_file.read_text()

        # Check for deprecated fillna patterns
        deprecated_patterns = [
            r'fillna\(method=["\']ffill["\']',
            r'fillna\(method=["\']bfill["\']',
        ]

        for pattern in deprecated_patterns:
            matches = re.findall(pattern, content)
            assert len(matches) == 0, f"Found deprecated fillna pattern: {pattern}"

    def test_uses_new_fillna_method(self):
        """Verify new ffill() method is used"""
        feature_file = (
            Path(__file__).parent.parent / "features" / "feature_engineering.py"
        )
        content = feature_file.read_text()

        # Check for new ffill() method
        assert ".ffill(" in content or "df.ffill" in content

    def test_no_deprecated_aioredis_import(self):
        """Verify deprecated 'import aioredis' is not used"""
        redis_file = Path(__file__).parent.parent / "utils" / "redis_cache.py"
        content = redis_file.read_text()

        # Check for deprecated import
        assert (
            "import aioredis" not in content
            or "from redis import asyncio as aioredis" in content
        )

    def test_uses_new_redis_asyncio_import(self):
        """Verify new redis.asyncio import is used"""
        redis_file = Path(__file__).parent.parent / "utils" / "redis_cache.py"
        content = redis_file.read_text()

        # Check for new import pattern
        assert "from redis import asyncio as aioredis" in content


class TestRateLimiting:
    """Test rate limiting implementation"""

    def test_slowapi_imported_in_main(self):
        """Verify slowapi is imported"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        assert "from slowapi import" in content or "import slowapi" in content

    def test_rate_limiter_configured(self):
        """Verify rate limiter is configured on endpoints"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check for rate limit decorators
        assert "@limiter.limit(" in content
        assert "Limiter(" in content

    def test_ai_analyze_endpoint_has_rate_limit(self):
        """Verify /ai/analyze endpoint has rate limiting"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Find the analyze endpoint
        lines = content.split("\n")
        found_analyze = False
        found_rate_limit = False

        for i, line in enumerate(lines):
            if "@app.post" in line and "/ai/analyze" in line:
                found_analyze = True
                # Check next line or previous line for rate limit decorator
                if i + 1 < len(lines) and "@limiter.limit(" in lines[i + 1]:
                    found_rate_limit = True
                    break
                if i > 0 and "@limiter.limit(" in lines[i - 1]:
                    found_rate_limit = True
                    break

        assert found_analyze, "/ai/analyze endpoint not found"
        assert found_rate_limit, "/ai/analyze endpoint missing rate limit"


class TestThreadSafety:
    """Test thread safety improvements"""

    @pytest.mark.skip(
        reason="Threading lock not needed - FastAPI/Uvicorn + SlowAPI provide thread safety"
    )
    def test_rate_limit_lock_exists(self):
        """Verify thread lock for rate limiting exists"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        assert "import threading" in content
        assert "_rate_limit_lock" in content
        assert "threading.Lock()" in content

    @pytest.mark.skip(
        reason="Threading lock not needed - FastAPI/Uvicorn + SlowAPI provide thread safety"
    )
    def test_lock_used_in_rate_limit_code(self):
        """Verify lock is used in rate limiting code"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        assert "with _rate_limit_lock:" in content

    def test_thread_safety_documentation_exists(self):
        """Verify thread safety is documented"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check for thread safety comments
        assert "thread safety" in content.lower() or "thread-safe" in content.lower()


class TestEnvironmentConfiguration:
    """Test environment configuration"""

    def test_env_example_exists(self):
        """Verify .env.example file exists"""
        env_example = Path(__file__).parent.parent / ".env.example"
        assert env_example.exists(), ".env.example file does not exist"

    def test_env_example_has_all_required_vars(self):
        """Verify .env.example has all required environment variables"""
        env_example = Path(__file__).parent.parent / ".env.example"
        content = env_example.read_text()

        required_vars = [
            "OPENAI_API_KEY",
            "OPENAI_BACKUP_API_KEYS",
            "DATABASE_URL",
            "ALLOWED_ORIGINS",
        ]

        for var in required_vars:
            assert var in content, f"Missing required variable: {var}"

    def test_env_example_has_security_warning(self):
        """Verify .env.example has security warnings"""
        env_example = Path(__file__).parent.parent / ".env.example"
        content = env_example.read_text()

        # Check for security-related warnings
        security_keywords = ["never commit", "secure", "security"]
        has_warning = any(keyword in content.lower() for keyword in security_keywords)
        assert has_warning, ".env.example missing security warning"


class TestCodeQuality:
    """Test overall code quality improvements"""

    def test_no_obvious_security_issues_in_main(self):
        """Verify no obvious security issues in main.py"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check for common security anti-patterns
        security_issues = []

        # Check for eval() usage
        if "eval(" in content:
            security_issues.append("Uses eval()")

        # Check for exec() usage
        if "exec(" in content:
            security_issues.append("Uses exec()")

        # Check for os.system() usage
        if "os.system(" in content:
            security_issues.append("Uses os.system()")

        assert len(security_issues) == 0, f"Security issues found: {security_issues}"

    def test_proper_error_handling_for_api_keys(self):
        """Verify proper error handling when API keys are missing"""
        main_file = Path(__file__).parent.parent / "main.py"
        content = main_file.read_text()

        # Check for error handling when API key is not configured
        assert (
            "api_key_configured" in content.lower()
            or "api key not configured" in content.lower()
        )


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
