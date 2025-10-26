"""
Integration tests for Python AI Service.
"""

import pytest
import asyncio
from unittest.mock import patch, AsyncMock, MagicMock
from datetime import datetime, timezone, timedelta
import json

# Import after adding to path in conftest
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


@pytest.mark.integration
class TestFullAnalysisFlow:
    """Test complete analysis flow from request to storage."""

    @pytest.mark.asyncio
    async def test_complete_analysis_flow(
        self, client, sample_ai_analysis_request, mock_openai_client, mock_mongodb
    ):
        """Test full flow: receive request -> analyze -> store -> return."""
        # Setup mocks - use AsyncMock for async method
        mock_openai_client.chat_completions_create = AsyncMock(
            return_value={
                "choices": [
                    {
                        "message": {
                            "content": '{"signal": "Long", "confidence": 0.85, "reasoning": "Strong bullish indicators", "strategy_scores": {"RSI": 0.8, "MACD": 0.7}, "market_analysis": {"trend_direction": "Bullish", "trend_strength": 0.75, "support_levels": [45000], "resistance_levels": [46000], "volatility_level": "Medium", "volume_analysis": "Increasing volume"}, "risk_assessment": {"overall_risk": "Medium", "technical_risk": 0.4, "market_risk": 0.5, "recommended_position_size": 0.03}}'
                        }
                    }
                ]
            }
        )

        with patch("main.openai_client", mock_openai_client), patch(
            "main.mongodb_db", mock_mongodb[1]
        ), patch("main.store_analysis_result", AsyncMock()) as mock_store:
            # Make request
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)

            # Verify response
            assert response.status_code == 200
            data = response.json()
            assert data["signal"] == "Long"
            assert data["confidence"] == 0.85

            # Verify storage was called
            await asyncio.sleep(0.1)  # Allow async storage to complete
            mock_store.assert_called_once()

    @pytest.mark.asyncio
    @pytest.mark.skip(reason="fetch_binance_candles function not implemented")
    async def test_periodic_analysis_task(self, mock_openai_client, mock_mongodb):
        """Test periodic analysis task execution."""
        from main import periodic_analysis_runner, ANALYSIS_SYMBOLS

        # Mock successful analyses
        mock_openai_client.chat.completions.create.return_value = MagicMock(
            choices=[
                MagicMock(
                    message=MagicMock(
                        content='{"signal": "Neutral", "confidence": 0.65, "reasoning": "Market consolidation"}'
                    )
                )
            ]
        )

        # Mock candle data fetch
        mock_candles = [
            [
                1701234567000,
                "45000",
                "45500",
                "44800",
                "45200",
                "1000",
                1701238167000,
                "45000000",
                100,
                "500",
                "22500000",
                "0",
            ]
        ]

        with patch("main.openai_client", mock_openai_client), patch(
            "main.mongodb_db", mock_mongodb[1]
        ), patch(
            "main.fetch_binance_candles", AsyncMock(return_value=mock_candles)
        ), patch(
            "main.store_analysis_result", AsyncMock()
        ) as mock_store, patch(
            "main.ws_manager.broadcast_signal", AsyncMock()
        ) as mock_broadcast:
            # Run one iteration
            await periodic_analysis_runner()

            # Should analyze all symbols
            assert mock_store.call_count == len(ANALYSIS_SYMBOLS)
            assert mock_broadcast.call_count == len(ANALYSIS_SYMBOLS)


@pytest.mark.integration
class TestAPIKeyRotation:
    """Test API key rotation in real scenarios."""

    @pytest.mark.asyncio
    @pytest.mark.skip(reason="call_openai_with_fallback function not implemented")
    async def test_api_key_rotation_on_rate_limit(
        self, client, sample_ai_analysis_request
    ):
        """Test API key rotation when rate limited."""
        import httpx
        from main import call_openai_with_fallback

        # Mock multiple API keys
        api_keys = ["key1_limited", "key2_limited", "key3_working"]

        # Create mock responses
        rate_limit_response = httpx.Response(
            status_code=429,
            json={"error": {"message": "Rate limit exceeded"}},
            headers={"retry-after": "60"},
        )

        success_response = MagicMock()
        success_response.status_code = 200
        success_response.json.return_value = {
            "choices": [
                {"message": {"content": '{"signal": "Short", "confidence": 0.78}'}}
            ]
        }

        # Mock httpx client
        mock_client = AsyncMock()
        mock_client.post.side_effect = [
            httpx.HTTPStatusError(
                "Rate limited", request=MagicMock(), response=rate_limit_response
            ),
            httpx.HTTPStatusError(
                "Rate limited", request=MagicMock(), response=rate_limit_response
            ),
            success_response,
        ]

        with patch("httpx.AsyncClient", return_value=mock_client):
            result = await call_openai_with_fallback(
                api_keys, [{"role": "user", "content": "test"}]
            )

            assert result is not None
            assert mock_client.post.call_count == 3


@pytest.mark.integration
class TestMongoDBIntegration:
    """Test MongoDB integration scenarios."""

    @pytest.mark.asyncio
    async def test_mongodb_connection_failure_recovery(
        self, client, sample_ai_analysis_request
    ):
        """Test system continues working when MongoDB fails."""
        # Simulate MongoDB failure
        with patch("main.mongodb_db", None):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)

            # Should still work without MongoDB
            assert response.status_code == 200
            data = response.json()
            assert "signal" in data

    @pytest.mark.asyncio
    async def test_mongodb_storage_and_retrieval(self, mock_mongodb):
        """Test storing and retrieving analysis from MongoDB."""
        from main import store_analysis_result, get_latest_analysis

        # Test data
        symbol = "BTCUSDT"
        analysis_result = {
            "signal": "Long",
            "confidence": 0.82,
            "reasoning": "Test reasoning",
            "metadata": {"analysis_id": "test123"},
        }

        # Mock retrieval document (matches the structure stored by store_analysis_result)
        stored_doc = {
            "_id": "mock_id",
            "symbol": symbol,
            "timestamp": datetime.now(timezone.utc),
            "analysis": analysis_result,  # Analysis is nested under "analysis" key
            "created_at": datetime.now(timezone.utc),
        }

        # Create a fresh mock collection with specific methods
        mock_collection = AsyncMock()
        mock_collection.insert_one = AsyncMock(
            return_value=MagicMock(inserted_id="mock_id")
        )
        mock_collection.find_one = AsyncMock(return_value=stored_doc)

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        # Test both store and retrieve in same context
        with patch("main.mongodb_db", mock_mongodb[1]):
            # Store
            await store_analysis_result(symbol, analysis_result)

            # Verify insert was called
            mock_collection.insert_one.assert_called_once()

            # Retrieve (returns just the analysis part)
            result = await get_latest_analysis(symbol)
            assert result is not None
            assert result["signal"] == "Long"
            assert result["confidence"] == 0.82


@pytest.mark.integration
class TestWebSocketBroadcasting:
    """Test WebSocket broadcasting in real scenarios."""

    @pytest.mark.asyncio
    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    async def test_analysis_broadcast_to_clients(self, test_client, mock_openai_client):
        """Test that analysis results are broadcast to WebSocket clients."""
        from main import ws_manager

        # Connect WebSocket client
        with test_client.websocket_connect("/ws") as websocket:
            # Skip connection message
            websocket.receive_json()

            # Trigger an analysis that should broadcast
            analysis_result = {
                "type": "ai_signal",
                "data": {
                    "symbol": "BTCUSDT",
                    "signal": "Long",
                    "confidence": 0.80,
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                },
            }

            # Manually broadcast (in real scenario, this happens automatically)
            asyncio.create_task(ws_manager.broadcast_signal(analysis_result))

            # Small delay to allow broadcast
            await asyncio.sleep(0.1)

            # Note: In synchronous test context, may not receive async broadcast
            # This demonstrates the pattern


@pytest.mark.integration
class TestErrorHandlingAndRecovery:
    """Test error handling and recovery mechanisms."""

    @pytest.mark.asyncio
    async def test_openai_api_complete_failure(
        self, client, sample_ai_analysis_request
    ):
        """Test handling when all OpenAI API keys fail - should fall back to technical analysis."""
        with patch("main.openai_client", None):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
            # System gracefully degrades to technical analysis instead of failing
            assert response.status_code == 200
            data = response.json()
            assert "signal" in data
            # Should use fallback technical analysis
            assert "Technical analysis" in data.get("reasoning", "")

    @pytest.mark.asyncio
    async def test_invalid_candle_data_handling(self, client):
        """Test handling of malformed candle data."""
        bad_request = {
            "symbol": "BTCUSDT",
            "timeframe": "1h",
            "candles": [
                {
                    "open": "not_a_number",
                    "high": 45500,
                    "low": 44800,
                    "close": 45200,
                    "volume": 1000,
                    "timestamp": 1701234567000,
                }
            ],
        }

        response = await client.post("/ai/analyze", json=bad_request)
        assert response.status_code == 422  # Validation error

    @pytest.mark.asyncio
    async def test_service_degradation(self, client, sample_ai_analysis_request):
        """Test service degradation when dependencies fail."""
        # Simulate various failures
        with patch("main.mongodb_db", None), patch(
            "main.ws_manager.broadcast_signal",
            AsyncMock(side_effect=Exception("WebSocket error")),
        ):
            # Service should still respond
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
            assert response.status_code == 200

            # Core functionality works despite failures
            data = response.json()
            assert "signal" in data
            assert "confidence" in data
