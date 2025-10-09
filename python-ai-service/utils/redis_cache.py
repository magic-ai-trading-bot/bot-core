import json
import hashlib
import asyncio
from typing import Any, Optional, Union
from datetime import timedelta
from functools import wraps
from redis import asyncio as aioredis
from utils.logger import get_logger

logger = get_logger("RedisCache")


class RedisCache:
    """Redis caching implementation for AI service"""

    def __init__(
        self,
        host: str = "redis",
        port: int = 6379,
        password: Optional[str] = None,
        db: int = 0,
    ):
        self.host = host
        self.port = port
        self.password = password
        self.db = db
        self._redis = None

    async def connect(self):
        """Initialize Redis connection"""
        try:
            # Updated to use redis.asyncio instead of deprecated aioredis
            redis_url = f"redis://{self.host}:{self.port}/{self.db}"
            if self.password:
                redis_url = (
                    f"redis://:{self.password}@{self.host}:{self.port}/{self.db}"
                )

            self._redis = await aioredis.from_url(
                redis_url,
                encoding="utf-8",
                decode_responses=True,
            )
            logger.info("✅ Connected to Redis cache")
        except Exception as e:
            logger.error(f"❌ Failed to connect to Redis: {e}")
            self._redis = None

    async def disconnect(self):
        """Close Redis connection"""
        if self._redis:
            await self._redis.close()

    def _generate_key(self, prefix: str, *args, **kwargs) -> str:
        """Generate cache key from prefix and parameters"""
        key_data = f"{prefix}:{str(args)}:{str(sorted(kwargs.items()))}"
        return hashlib.md5(key_data.encode()).hexdigest()

    async def get(self, key: str) -> Optional[Any]:
        """Get value from cache"""
        if not self._redis:
            return None

        try:
            value = await self._redis.get(key)
            if value:
                return json.loads(value)
        except Exception as e:
            logger.warning(f"Cache get error: {e}")
        return None

    async def set(self, key: str, value: Any, ttl: int = 300):
        """Set value in cache with TTL (seconds)"""
        if not self._redis:
            return

        try:
            await self._redis.set(key, json.dumps(value), ex=ttl)
        except Exception as e:
            logger.warning(f"Cache set error: {e}")

    async def delete(self, key: str):
        """Delete key from cache"""
        if not self._redis:
            return

        try:
            await self._redis.delete(key)
        except Exception as e:
            logger.warning(f"Cache delete error: {e}")

    async def clear_pattern(self, pattern: str):
        """Clear all keys matching pattern"""
        if not self._redis:
            return

        try:
            keys = await self._redis.keys(pattern)
            if keys:
                await self._redis.delete(*keys)
        except Exception as e:
            logger.warning(f"Cache clear pattern error: {e}")


# Decorator for caching function results
def cache_result(prefix: str, ttl: int = 300):
    """Decorator to cache function results"""

    def decorator(func):
        @wraps(func)
        async def wrapper(self, *args, **kwargs):
            # Check if instance has cache
            if not hasattr(self, "_cache") or not self._cache:
                return await func(self, *args, **kwargs)

            # Generate cache key
            cache_key = self._cache._generate_key(prefix, *args, **kwargs)

            # Try to get from cache
            cached = await self._cache.get(cache_key)
            if cached is not None:
                logger.debug(f"Cache hit for {prefix}")
                return cached

            # Execute function and cache result
            result = await func(self, *args, **kwargs)
            await self._cache.set(cache_key, result, ttl)

            return result

        return wrapper

    return decorator


class CacheManager:
    """Manages different cache strategies"""

    def __init__(self, redis_cache: RedisCache):
        self._cache = redis_cache

    async def cache_market_data(self, symbol: str, data: dict, ttl: int = 60):
        """Cache market data with short TTL"""
        key = f"market:{symbol}"
        await self._cache.set(key, data, ttl)

    async def get_market_data(self, symbol: str) -> Optional[dict]:
        """Get cached market data"""
        key = f"market:{symbol}"
        return await self._cache.get(key)

    async def cache_ai_prediction(self, symbol: str, prediction: dict, ttl: int = 300):
        """Cache AI predictions with medium TTL"""
        key = f"prediction:{symbol}"
        await self._cache.set(key, prediction, ttl)

    async def get_ai_prediction(self, symbol: str) -> Optional[dict]:
        """Get cached AI prediction"""
        key = f"prediction:{symbol}"
        return await self._cache.get(key)

    async def cache_technical_indicators(
        self, symbol: str, timeframe: str, indicators: dict, ttl: int = 180
    ):
        """Cache technical indicators"""
        key = f"indicators:{symbol}:{timeframe}"
        await self._cache.set(key, indicators, ttl)

    async def get_technical_indicators(
        self, symbol: str, timeframe: str
    ) -> Optional[dict]:
        """Get cached technical indicators"""
        key = f"indicators:{symbol}:{timeframe}"
        return await self._cache.get(key)

    async def invalidate_symbol_cache(self, symbol: str):
        """Invalidate all cache for a symbol"""
        patterns = [
            f"market:{symbol}*",
            f"prediction:{symbol}*",
            f"indicators:{symbol}*",
        ]
        for pattern in patterns:
            await self._cache.clear_pattern(pattern)
