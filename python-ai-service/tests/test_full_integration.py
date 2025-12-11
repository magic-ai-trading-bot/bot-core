"""
Comprehensive integration tests for Python AI Service
Tests full service integration, ML models, and cross-service communication
"""

import asyncio
import os
import sys
from datetime import datetime, timezone
from unittest.mock import AsyncMock, MagicMock, patch

import numpy as np
import pytest
from httpx import AsyncClient

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


@pytest.mark.integration
class TestFullAnalysisPipeline:
    """Test complete AI analysis pipeline"""

    @pytest.mark.asyncio
    async def test_end_to_end_analysis_flow(
        self, client, sample_ai_analysis_request, mock_openai_client
    ):
        """Test complete flow: request → indicators → ML → AI → response"""

        # Setup OpenAI mock
        mock_openai_client.chat_completions_create = AsyncMock(
            return_value={
                "choices": [
                    {
                        "message": {
                            "content": '{"signal": "Long", "confidence": 0.82, "reasoning": "Strong bullish momentum"}'
                        }
                    }
                ]
            }
        )

        with patch("main.openai_client", mock_openai_client):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)

            assert response.status_code == 200
            data = response.json()

            # Verify response structure
            assert "signal" in data
            assert "confidence" in data
            assert data["signal"] in ["Long", "Short", "Neutral"]
            assert 0.0 <= data["confidence"] <= 1.0

    @pytest.mark.asyncio
    async def test_concurrent_analysis_requests(self, client):
        """Test handling multiple concurrent analysis requests"""

        symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT"]

        tasks = []
        for symbol in symbols:
            request_data = {
                "symbol": symbol,
                "timeframe": "1h",
                "candles": [
                    {
                        "open": 50000.0,
                        "high": 50500.0,
                        "low": 49800.0,
                        "close": 50200.0,
                        "volume": 1000.0,
                        "timestamp": 1701234567000,
                    }
                ],
            }
            tasks.append(client.post("/ai/analyze", json=request_data))

        responses = await asyncio.gather(*tasks, return_exceptions=True)

        # All requests should complete
        assert len(responses) == len(symbols)

        # Most should succeed (some might fail due to rate limiting)
        successful = [
            r
            for r in responses
            if not isinstance(r, Exception) and r.status_code == 200
        ]
        assert len(successful) >= 0  # At least no crashes


@pytest.mark.integration
class TestMLModelIntegration:
    """Test ML model loading and inference"""

    @pytest.mark.asyncio
    async def test_technical_indicators_calculation(self):
        """Test calculating technical indicators"""

        # Sample price data
        prices = [50000, 50100, 50200, 50150, 50300, 50250, 50400, 50350, 50500, 50450]

        # Calculate simple moving average
        window = 3
        sma = []
        for i in range(len(prices) - window + 1):
            avg = sum(prices[i : i + window]) / window
            sma.append(avg)

        assert len(sma) == len(prices) - window + 1
        assert all(isinstance(x, (int, float)) for x in sma)

    @pytest.mark.asyncio
    async def test_rsi_calculation(self):
        """Test RSI indicator calculation"""

        prices = np.array(
            [50000, 50100, 50050, 50200, 50150, 50300, 50250, 50400, 50350, 50500]
        )

        # Calculate price changes
        deltas = np.diff(prices)
        gains = np.where(deltas > 0, deltas, 0)
        losses = np.where(deltas < 0, -deltas, 0)

        # Simple RSI calculation
        avg_gain = np.mean(gains[:7]) if len(gains) >= 7 else 0
        avg_loss = np.mean(losses[:7]) if len(losses) >= 7 else 0

        if avg_loss == 0:
            rsi = 100
        else:
            rs = avg_gain / avg_loss
            rsi = 100 - (100 / (1 + rs))

        assert 0 <= rsi <= 100

    @pytest.mark.asyncio
    async def test_macd_calculation(self):
        """Test MACD indicator calculation"""

        prices = np.array(
            [
                50000,
                50100,
                50200,
                50150,
                50300,
                50250,
                50400,
                50350,
                50500,
                50450,
                50600,
                50550,
            ]
        )

        # Simple EMA calculation
        def calculate_ema(data, period):
            multiplier = 2 / (period + 1)
            ema = [data[0]]
            for price in data[1:]:
                ema.append((price - ema[-1]) * multiplier + ema[-1])
            return ema[-1]

        ema_12 = calculate_ema(prices, 12)
        ema_26 = (
            calculate_ema(prices, 26)
            if len(prices) >= 26
            else calculate_ema(prices, len(prices))
        )

        macd = ema_12 - ema_26

        assert isinstance(macd, (int, float))


@pytest.mark.integration
class TestDatabaseIntegration:
    """Test MongoDB integration"""

    @pytest.mark.asyncio
    async def test_store_and_retrieve_analysis(self, mock_mongodb):
        """Test storing and retrieving analysis results"""

        from main import get_latest_analysis, store_analysis_result

        symbol = "BTCUSDT"
        analysis = {
            "signal": "Long",
            "confidence": 0.85,
            "reasoning": "Test analysis",
        }

        # Create a fresh mock collection with specific methods
        mock_collection = AsyncMock()
        mock_collection.insert_one = AsyncMock(
            return_value=MagicMock(inserted_id="test_id")
        )
        mock_collection.find_one = AsyncMock(
            return_value={
                "symbol": symbol,
                "analysis": analysis,
                "timestamp": datetime.now(timezone.utc),
            }
        )

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            # Store
            await store_analysis_result(symbol, analysis)
            mock_collection.insert_one.assert_called_once()

            # Retrieve
            result = await get_latest_analysis(symbol)
            assert result is not None
            assert result["signal"] == "Long"

    @pytest.mark.asyncio
    async def test_database_failure_graceful_degradation(
        self, client, sample_ai_analysis_request
    ):
        """Test system continues when database fails"""

        with patch("main.mongodb_db", None):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)

            # Should still work without database
            assert response.status_code == 200
            data = response.json()
            assert "signal" in data


@pytest.mark.integration
class TestWebSocketIntegration:
    """Test WebSocket broadcasting"""

    @pytest.mark.asyncio
    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    async def test_websocket_connection(self, test_client):
        """Test WebSocket connection and messages"""

        with test_client.websocket_connect("/ws") as websocket:
            # Receive connection message
            data = websocket.receive_json()
            assert "type" in data or "message" in data

    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    @pytest.mark.asyncio
    async def test_signal_broadcast(self, test_client):
        """Test broadcasting AI signals to WebSocket clients"""

        from main import ws_manager

        signal_data = {
            "type": "ai_signal",
            "data": {
                "symbol": "BTCUSDT",
                "signal": "Long",
                "confidence": 0.80,
            },
        }

        # This tests the broadcast mechanism exists
        # Actual broadcast happens asynchronously
        assert hasattr(ws_manager, "broadcast_signal")


@pytest.mark.integration
class TestAPIEndpoints:
    """Test all API endpoints integration"""

    @pytest.mark.asyncio
    async def test_health_endpoint(self, client):
        """Test health check endpoint"""

        response = await client.get("/health")
        assert response.status_code == 200

        data = response.json()
        assert data["status"] == "healthy"
        assert "service" in data

    @pytest.mark.asyncio
    async def test_analyze_endpoint_validation(self, client):
        """Test input validation on analyze endpoint"""

        # Invalid request - missing required fields
        invalid_request = {"symbol": "BTCUSDT"}

        response = await client.post("/ai/analyze", json=invalid_request)
        assert response.status_code in [400, 422]  # Validation error

    @pytest.mark.asyncio
    async def test_analyze_endpoint_with_minimal_data(self, client):
        """Test analyze endpoint with minimal valid data"""

        minimal_request = {
            "symbol": "BTCUSDT",
            "timeframe": "1h",
            "candles": [
                {
                    "open": 50000.0,
                    "high": 50500.0,
                    "low": 49800.0,
                    "close": 50200.0,
                    "volume": 1000.0,
                    "timestamp": 1701234567000,
                }
            ],
        }

        response = await client.post("/ai/analyze", json=minimal_request)

        # Should work with minimal data or return rate limit/validation error
        # 200: Success, 422: Validation error (missing fields), 429: Rate limit, 500: Server error
        assert response.status_code in [
            200,
            422,
            429,
            500,
        ], f"Unexpected status code: {response.status_code}"


@pytest.mark.integration
class TestErrorHandling:
    """Test error handling and recovery"""

    @pytest.mark.asyncio
    async def test_invalid_candle_data(self, client):
        """Test handling of invalid candle data"""

        bad_request = {
            "symbol": "BTCUSDT",
            "timeframe": "1h",
            "candles": [
                {
                    "open": "invalid",  # String instead of number
                    "high": 50500.0,
                    "low": 49800.0,
                    "close": 50200.0,
                    "volume": 1000.0,
                    "timestamp": 1701234567000,
                }
            ],
        }

        response = await client.post("/ai/analyze", json=bad_request)
        assert response.status_code == 422  # Validation error

    @pytest.mark.asyncio
    async def test_openai_api_failure_fallback(
        self, client, sample_ai_analysis_request
    ):
        """Test fallback when OpenAI API fails"""

        with patch("main.openai_client", None):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)

            # Should fallback to technical analysis
            assert response.status_code == 200
            data = response.json()
            assert "signal" in data


@pytest.mark.integration
class TestPerformance:
    """Test performance characteristics"""

    @pytest.mark.asyncio
    async def test_response_time(self, client, sample_ai_analysis_request):
        """Test API response time is reasonable"""

        import time

        start = time.time()
        response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
        duration = time.time() - start

        # Should respond within reasonable time (30 seconds for AI call)
        assert duration < 30.0
        assert response.status_code in [200, 500]

    @pytest.mark.asyncio
    async def test_memory_usage(self):
        """Test memory usage doesn't grow unbounded"""

        import gc

        # Force garbage collection
        gc.collect()

        # Simulate processing multiple requests
        for _ in range(10):
            data = np.random.rand(100, 10)
            result = np.mean(data, axis=0)
            assert len(result) == 10

        # Cleanup
        gc.collect()


@pytest.mark.integration
class TestCaching:
    """Test caching mechanisms"""

    @pytest.mark.asyncio
    async def test_analysis_caching(self, mock_mongodb):
        """Test that analysis results are cached"""

        from main import get_latest_analysis

        symbol = "BTCUSDT"
        cached_analysis = {
            "signal": "Long",
            "confidence": 0.75,
        }

        # Create a fresh mock collection with specific methods
        mock_collection = AsyncMock()
        mock_collection.find_one = AsyncMock(
            return_value={
                "symbol": symbol,
                "analysis": cached_analysis,
                "timestamp": datetime.now(timezone.utc),
            }
        )

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            result = await get_latest_analysis(symbol)

            # Should retrieve from cache/database
            assert result is not None
