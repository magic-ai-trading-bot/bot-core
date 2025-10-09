"""Tests for utils/redis_cache.py"""
import pytest
import json
from unittest.mock import AsyncMock, MagicMock, patch

# Skip redis tests if dependencies not available
pytest.importorskip("redis", reason="redis not installed")
pytest.importorskip("aioredis", reason="aioredis not installed")

from utils.redis_cache import RedisCache, cache_result, CacheManager


class TestRedisCache:
    """Test RedisCache class"""

    def test_init(self):
        """Test RedisCache initialization"""
        cache = RedisCache(host="localhost", port=6379, password="secret", db=1)
        assert cache.host == "localhost"
        assert cache.port == 6379
        assert cache.password == "secret"
        assert cache.db == 1
        assert cache._redis is None

    def test_init_defaults(self):
        """Test RedisCache initialization with defaults"""
        cache = RedisCache()
        assert cache.host == "redis"
        assert cache.port == 6379
        assert cache.password is None
        assert cache.db == 0

    @pytest.mark.asyncio
    @patch('utils.redis_cache.aioredis.create_redis_pool')
    async def test_connect_success(self, mock_create_pool):
        """Test successful Redis connection"""
        mock_pool = AsyncMock()
        mock_create_pool.return_value = mock_pool

        cache = RedisCache()
        await cache.connect()

        assert cache._redis == mock_pool
        mock_create_pool.assert_called_once_with(
            'redis://redis:6379/0',
            password=None,
            encoding='utf-8'
        )

    @pytest.mark.asyncio
    @patch('utils.redis_cache.aioredis.create_redis_pool')
    async def test_connect_failure(self, mock_create_pool):
        """Test Redis connection failure"""
        mock_create_pool.side_effect = Exception("Connection failed")

        cache = RedisCache()
        await cache.connect()

        assert cache._redis is None

    @pytest.mark.asyncio
    async def test_disconnect(self):
        """Test Redis disconnection"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        cache._redis = mock_redis

        await cache.disconnect()

        mock_redis.close.assert_called_once()
        mock_redis.wait_closed.assert_called_once()

    @pytest.mark.asyncio
    async def test_disconnect_without_connection(self):
        """Test disconnect when no connection exists"""
        cache = RedisCache()
        # Should not raise any error
        await cache.disconnect()

    def test_generate_key(self):
        """Test cache key generation"""
        cache = RedisCache()
        key1 = cache._generate_key("test", "arg1", "arg2", param1="value1")
        key2 = cache._generate_key("test", "arg1", "arg2", param1="value1")
        key3 = cache._generate_key("test", "arg1", "arg3", param1="value1")

        # Same inputs should generate same key
        assert key1 == key2
        # Different inputs should generate different keys
        assert key1 != key3
        # Key should be MD5 hash (32 chars)
        assert len(key1) == 32

    @pytest.mark.asyncio
    async def test_get_success(self):
        """Test successful cache get"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.get.return_value = json.dumps({"test": "value"})
        cache._redis = mock_redis

        result = await cache.get("test_key")

        assert result == {"test": "value"}
        mock_redis.get.assert_called_once_with("test_key")

    @pytest.mark.asyncio
    async def test_get_not_found(self):
        """Test cache get when key doesn't exist"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.get.return_value = None
        cache._redis = mock_redis

        result = await cache.get("test_key")

        assert result is None

    @pytest.mark.asyncio
    async def test_get_without_connection(self):
        """Test cache get without Redis connection"""
        cache = RedisCache()
        result = await cache.get("test_key")
        assert result is None

    @pytest.mark.asyncio
    async def test_get_error(self):
        """Test cache get with error"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.get.side_effect = Exception("Redis error")
        cache._redis = mock_redis

        result = await cache.get("test_key")

        assert result is None

    @pytest.mark.asyncio
    async def test_set_success(self):
        """Test successful cache set"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        cache._redis = mock_redis

        await cache.set("test_key", {"test": "value"}, ttl=60)

        mock_redis.setex.assert_called_once_with(
            "test_key",
            60,
            json.dumps({"test": "value"})
        )

    @pytest.mark.asyncio
    async def test_set_without_connection(self):
        """Test cache set without Redis connection"""
        cache = RedisCache()
        # Should not raise error
        await cache.set("test_key", {"test": "value"})

    @pytest.mark.asyncio
    async def test_set_error(self):
        """Test cache set with error"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.setex.side_effect = Exception("Redis error")
        cache._redis = mock_redis

        # Should not raise error
        await cache.set("test_key", {"test": "value"})

    @pytest.mark.asyncio
    async def test_delete_success(self):
        """Test successful cache delete"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        cache._redis = mock_redis

        await cache.delete("test_key")

        mock_redis.delete.assert_called_once_with("test_key")

    @pytest.mark.asyncio
    async def test_delete_without_connection(self):
        """Test cache delete without Redis connection"""
        cache = RedisCache()
        # Should not raise error
        await cache.delete("test_key")

    @pytest.mark.asyncio
    async def test_delete_error(self):
        """Test cache delete with error"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.delete.side_effect = Exception("Redis error")
        cache._redis = mock_redis

        # Should not raise error
        await cache.delete("test_key")

    @pytest.mark.asyncio
    async def test_clear_pattern_success(self):
        """Test successful cache clear by pattern"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.keys.return_value = ["key1", "key2", "key3"]
        cache._redis = mock_redis

        await cache.clear_pattern("test:*")

        mock_redis.keys.assert_called_once_with("test:*")
        mock_redis.delete.assert_called_once_with("key1", "key2", "key3")

    @pytest.mark.asyncio
    async def test_clear_pattern_no_keys(self):
        """Test cache clear by pattern with no matching keys"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.keys.return_value = []
        cache._redis = mock_redis

        await cache.clear_pattern("test:*")

        mock_redis.keys.assert_called_once_with("test:*")
        mock_redis.delete.assert_not_called()

    @pytest.mark.asyncio
    async def test_clear_pattern_without_connection(self):
        """Test cache clear pattern without Redis connection"""
        cache = RedisCache()
        # Should not raise error
        await cache.clear_pattern("test:*")

    @pytest.mark.asyncio
    async def test_clear_pattern_error(self):
        """Test cache clear pattern with error"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.keys.side_effect = Exception("Redis error")
        cache._redis = mock_redis

        # Should not raise error
        await cache.clear_pattern("test:*")


class TestCacheResultDecorator:
    """Test cache_result decorator"""

    @pytest.mark.asyncio
    async def test_decorator_without_cache(self):
        """Test decorator when instance has no cache"""
        class TestClass:
            async def test_method(self, arg):
                return f"result_{arg}"

        # Apply decorator
        TestClass.test_method = cache_result("test", ttl=60)(TestClass.test_method)

        instance = TestClass()
        result = await instance.test_method("value")

        assert result == "result_value"

    @pytest.mark.asyncio
    async def test_decorator_cache_miss(self):
        """Test decorator with cache miss"""
        class TestClass:
            def __init__(self):
                self._cache = RedisCache()
                mock_redis = AsyncMock()
                mock_redis.get.return_value = None
                self._cache._redis = mock_redis

            async def test_method(self, arg):
                return f"result_{arg}"

        # Apply decorator
        TestClass.test_method = cache_result("test", ttl=60)(TestClass.test_method)

        instance = TestClass()
        result = await instance.test_method("value")

        assert result == "result_value"
        instance._cache._redis.get.assert_called_once()
        instance._cache._redis.setex.assert_called_once()

    @pytest.mark.asyncio
    async def test_decorator_cache_hit(self):
        """Test decorator with cache hit"""
        class TestClass:
            def __init__(self):
                self._cache = RedisCache()
                mock_redis = AsyncMock()
                mock_redis.get.return_value = json.dumps("cached_result")
                self._cache._redis = mock_redis
                self.call_count = 0

            async def test_method(self, arg):
                self.call_count += 1
                return f"result_{arg}"

        # Apply decorator
        TestClass.test_method = cache_result("test", ttl=60)(TestClass.test_method)

        instance = TestClass()
        result = await instance.test_method("value")

        assert result == "cached_result"
        # Method should not be called when cache hits
        assert instance.call_count == 0


class TestCacheManager:
    """Test CacheManager class"""

    def test_init(self):
        """Test CacheManager initialization"""
        cache = RedisCache()
        manager = CacheManager(cache)
        assert manager._cache == cache

    @pytest.mark.asyncio
    async def test_cache_market_data(self):
        """Test caching market data"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        cache._redis = mock_redis
        manager = CacheManager(cache)

        data = {"price": 50000, "volume": 1000}
        await manager.cache_market_data("BTCUSDT", data, ttl=60)

        mock_redis.setex.assert_called_once()
        call_args = mock_redis.setex.call_args
        assert call_args[0][0] == "market:BTCUSDT"
        assert call_args[0][1] == 60

    @pytest.mark.asyncio
    async def test_get_market_data(self):
        """Test getting cached market data"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.get.return_value = json.dumps({"price": 50000})
        cache._redis = mock_redis
        manager = CacheManager(cache)

        result = await manager.get_market_data("BTCUSDT")

        assert result == {"price": 50000}
        mock_redis.get.assert_called_once_with("market:BTCUSDT")

    @pytest.mark.asyncio
    async def test_cache_ai_prediction(self):
        """Test caching AI prediction"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        cache._redis = mock_redis
        manager = CacheManager(cache)

        prediction = {"direction": "up", "confidence": 0.85}
        await manager.cache_ai_prediction("BTCUSDT", prediction, ttl=300)

        mock_redis.setex.assert_called_once()
        call_args = mock_redis.setex.call_args
        assert call_args[0][0] == "prediction:BTCUSDT"
        assert call_args[0][1] == 300

    @pytest.mark.asyncio
    async def test_get_ai_prediction(self):
        """Test getting cached AI prediction"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.get.return_value = json.dumps({"direction": "up"})
        cache._redis = mock_redis
        manager = CacheManager(cache)

        result = await manager.get_ai_prediction("BTCUSDT")

        assert result == {"direction": "up"}
        mock_redis.get.assert_called_once_with("prediction:BTCUSDT")

    @pytest.mark.asyncio
    async def test_cache_technical_indicators(self):
        """Test caching technical indicators"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        cache._redis = mock_redis
        manager = CacheManager(cache)

        indicators = {"rsi": 65, "macd": 120}
        await manager.cache_technical_indicators("BTCUSDT", "1h", indicators, ttl=180)

        mock_redis.setex.assert_called_once()
        call_args = mock_redis.setex.call_args
        assert call_args[0][0] == "indicators:BTCUSDT:1h"
        assert call_args[0][1] == 180

    @pytest.mark.asyncio
    async def test_get_technical_indicators(self):
        """Test getting cached technical indicators"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.get.return_value = json.dumps({"rsi": 65})
        cache._redis = mock_redis
        manager = CacheManager(cache)

        result = await manager.get_technical_indicators("BTCUSDT", "1h")

        assert result == {"rsi": 65}
        mock_redis.get.assert_called_once_with("indicators:BTCUSDT:1h")

    @pytest.mark.asyncio
    async def test_invalidate_symbol_cache(self):
        """Test invalidating all cache for a symbol"""
        cache = RedisCache()
        mock_redis = AsyncMock()
        mock_redis.keys.return_value = []
        cache._redis = mock_redis
        manager = CacheManager(cache)

        await manager.invalidate_symbol_cache("BTCUSDT")

        # Should call keys for each pattern
        assert mock_redis.keys.call_count == 3
        call_args_list = [call[0][0] for call in mock_redis.keys.call_args_list]
        assert "market:BTCUSDT*" in call_args_list
        assert "prediction:BTCUSDT*" in call_args_list
        assert "indicators:BTCUSDT*" in call_args_list
