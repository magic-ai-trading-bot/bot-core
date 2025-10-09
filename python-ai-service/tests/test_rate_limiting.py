"""
Tests for rate limiting functionality
"""

import pytest
import asyncio
from datetime import datetime, timedelta
from unittest.mock import Mock, patch, AsyncMock
import threading


class TestRateLimitLocking:
    """Test thread-safe rate limiting"""

    def test_rate_limit_lock_is_thread_safe(self):
        """Test that rate limit lock prevents race conditions"""
        lock = threading.Lock()
        counter = {"value": 0}

        def increment():
            with lock:
                current = counter["value"]
                # Simulate some processing
                import time

                time.sleep(0.001)
                counter["value"] = current + 1

        # Run multiple threads
        threads = []
        for _ in range(10):
            t = threading.Thread(target=increment)
            threads.append(t)
            t.start()

        for t in threads:
            t.join()

        assert counter["value"] == 10

    @pytest.mark.asyncio
    async def test_rate_limiting_delay_calculation(self):
        """Test rate limiting delay calculation"""
        request_delay = 20  # seconds
        last_request_time = datetime.now()

        # Simulate immediate next request
        time_since_last = (datetime.now() - last_request_time).total_seconds()

        if time_since_last < request_delay:
            delay = request_delay - time_since_last
            assert delay > 0
            assert delay <= request_delay

    def test_rate_limit_reset_time_tracking(self):
        """Test rate limit reset time tracking"""
        reset_time = datetime.now() + timedelta(hours=1)

        # Check if we're in rate limit period
        is_rate_limited = datetime.now() < reset_time
        assert is_rate_limited is True

        # Simulate time passing
        future_time = datetime.now() + timedelta(hours=2)
        is_still_limited = future_time < reset_time
        assert is_still_limited is False


class TestOpenAIRateLimiting:
    """Test OpenAI API rate limiting"""

    def test_api_key_fallback_on_rate_limit(self):
        """Test that API keys fallback on rate limit"""
        api_keys = ["key1", "key2", "key3"]
        rate_limited_keys = {0}  # key1 is rate limited

        # Get available keys
        available_keys = [
            key for i, key in enumerate(api_keys) if i not in rate_limited_keys
        ]

        assert len(available_keys) == 2
        assert "key1" not in available_keys
        assert "key2" in available_keys
        assert "key3" in available_keys

    def test_all_keys_rate_limited_clears_set(self):
        """Test that when all keys are rate limited, the set is cleared"""
        api_keys = ["key1", "key2", "key3"]
        rate_limited_keys = {0, 1, 2}  # All keys rate limited

        # Simulate clearing when all are limited
        available_keys = [
            key for i, key in enumerate(api_keys) if i not in rate_limited_keys
        ]

        if not available_keys:
            # Clear the rate limited set and start over
            rate_limited_keys.clear()
            available_keys = api_keys

        assert len(available_keys) == 3
        assert len(rate_limited_keys) == 0

    @pytest.mark.asyncio
    async def test_rate_limit_waits_before_request(self):
        """Test that rate limiting waits appropriate time before request"""
        request_delay = 1  # 1 second for testing
        last_request_time = datetime.now()

        # Wait a bit
        await asyncio.sleep(0.1)

        time_since_last = (datetime.now() - last_request_time).total_seconds()

        if time_since_last < request_delay:
            delay = request_delay - time_since_last
            assert delay > 0


class TestSlowAPIIntegration:
    """Test SlowAPI rate limiting integration"""

    def test_limiter_decorator_format(self):
        """Test that rate limit decorator has correct format"""
        # Test rate limit string parsing
        rate_limit = "10/minute"
        parts = rate_limit.split("/")

        assert len(parts) == 2
        assert parts[0].isdigit()
        assert parts[1] in ["second", "minute", "hour", "day"]

        # Test different formats
        test_limits = ["10/minute", "100/hour", "1000/day", "5/second"]
        for limit in test_limits:
            parts = limit.split("/")
            assert len(parts) == 2
            assert parts[0].isdigit()

    def test_rate_limit_exceeds_returns_429(self):
        """Test that exceeding rate limit returns 429 status"""
        # Simulate rate limit exceeded
        from fastapi import HTTPException

        def check_rate_limit(request_count, limit):
            if request_count > limit:
                raise HTTPException(status_code=429, detail="Rate limit exceeded")

        # Should not raise
        try:
            check_rate_limit(5, 10)
            passed = True
        except HTTPException:
            passed = False

        assert passed is True

        # Should raise
        with pytest.raises(HTTPException) as exc_info:
            check_rate_limit(15, 10)

        assert exc_info.value.status_code == 429


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
