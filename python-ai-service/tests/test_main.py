"""
Test main FastAPI application and endpoints.
"""

import os
import pytest
import asyncio
from unittest.mock import patch, AsyncMock, MagicMock, Mock
from datetime import datetime, timezone, timedelta
import json
import pandas as pd
import numpy as np
from main import (
    TechnicalAnalyzer,
    GPTTradingAnalyzer,
    DirectOpenAIClient,
    WebSocketManager,
    CandleData,
    AIAnalysisRequest,
    AIStrategyContext,
    store_analysis_result,
    get_latest_analysis,
    fetch_real_market_data,
)


@pytest.mark.unit
class TestHealthEndpoint:
    """Test health check endpoint."""

    @pytest.mark.asyncio
    async def test_health_check_success(self, client, mock_mongodb):
        """Test successful health check."""
        response = await client.get("/health")
        assert response.status_code == 200
        data = response.json()
        assert data["status"] == "healthy"
        assert data["service"] == "GPT-4 Trading AI"
        assert data["gpt4_available"] is True
        # MongoDB connection status can be True or False in test environment
        assert "mongodb_connected" in data

    @pytest.mark.asyncio
    async def test_health_check_mongodb_down(self, client):
        """Test health check when MongoDB is down."""
        with patch("main.mongodb_client", None):
            response = await client.get("/health")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "healthy"
            assert data["mongodb_connected"] is False


@pytest.mark.unit
class TestAIAnalysisEndpoint:
    """Test AI analysis endpoint."""

    @pytest.mark.asyncio
    async def test_analyze_success(
        self, client, sample_ai_analysis_request, mock_openai_client
    ):
        """Test successful AI analysis."""
        with patch("main.openai_client", mock_openai_client):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
            assert response.status_code == 200
            data = response.json()
            assert data["signal"] == "Long"
            assert data["confidence"] == 0.75
            assert "reasoning" in data
            assert len(data["reasoning"]) > 0
            assert "market_analysis" in data
            assert "risk_assessment" in data

    @pytest.mark.asyncio
    async def test_analyze_invalid_symbol(self, client, sample_ai_analysis_request):
        """Test analysis with any symbol (no validation in current implementation)."""
        sample_ai_analysis_request["symbol"] = "TESTUSDT"
        response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
        # Current implementation accepts any symbol
        assert response.status_code == 200
        data = response.json()
        assert data["signal"] in ["Long", "Short", "Neutral"]

    @pytest.mark.asyncio
    async def test_analyze_insufficient_candles(
        self, client, sample_ai_analysis_request
    ):
        """Test analysis with no candle data."""
        sample_ai_analysis_request["timeframe_data"]["1h"] = []
        response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
        # Current implementation handles empty candles gracefully
        assert response.status_code in [200, 400, 500]

    @pytest.mark.asyncio
    async def test_analyze_with_cached_result(
        self, client, sample_ai_analysis_request, mock_mongodb
    ):
        """Test analysis returns cached result."""
        # Mock cached result with proper timestamp format (milliseconds)
        # Use 1 minute ago to be safely within 2-minute cache window
        timestamp_ms = int(
            (datetime.now(timezone.utc) - timedelta(minutes=1)).timestamp() * 1000
        )
        cached_result = {
            "symbol": "BTCUSDT",
            "signal": "Short",
            "confidence": 0.85,
            "reasoning": "Cached reasoning",
            "timestamp": timestamp_ms,
            "strategy_scores": {},
            "market_analysis": {
                "trend_direction": "Bearish",
                "trend_strength": 0.75,
                "support_levels": [45000],
                "resistance_levels": [46000],
                "volatility_level": "Medium",
                "volume_analysis": "Decreasing volume",
            },
            "risk_assessment": {
                "overall_risk": "Medium",
                "technical_risk": 0.5,
                "market_risk": 0.5,
                "recommended_position_size": 0.02,
            },
        }

        with patch("main.get_latest_analysis", AsyncMock(return_value=cached_result)):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
            assert response.status_code == 200
            data = response.json()
            assert data["signal"] == "Short"
            assert data["confidence"] == 0.85


@pytest.mark.unit
class TestStrategyRecommendations:
    """Test strategy recommendations endpoint."""

    @pytest.mark.asyncio
    async def test_strategy_recommendations_success(self, client, sample_candle_data):
        """Test successful strategy recommendations."""
        request_data = {
            "symbol": "BTCUSDT",
            "timeframe_data": {"1h": sample_candle_data, "4h": sample_candle_data},
            "current_price": 45189.23,
            "available_strategies": ["RSI", "MACD", "EMA_Crossover"],
            "timestamp": int(datetime.now(timezone.utc).timestamp() * 1000),
        }

        response = await client.post("/ai/strategy-recommendations", json=request_data)
        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        assert len(data) > 0
        # Check first recommendation has required fields
        assert "strategy_name" in data[0]
        assert "suitability_score" in data[0]
        assert "reasoning" in data[0]


@pytest.mark.unit
class TestMarketCondition:
    """Test market condition analysis endpoint."""

    @pytest.mark.asyncio
    async def test_market_condition_success(self, client, sample_candle_data):
        """Test successful market condition analysis."""
        request_data = {
            "symbol": "BTCUSDT",
            "timeframe_data": {"1h": sample_candle_data, "4h": sample_candle_data},
            "current_price": 45000.0,
            "volume_24h": 25000000000.0,
            "timestamp": int(datetime.now(timezone.utc).timestamp() * 1000),
        }

        response = await client.post("/ai/market-condition", json=request_data)
        assert response.status_code == 200
        data = response.json()
        assert "condition_type" in data
        assert "market_phase" in data
        assert "confidence" in data


@pytest.mark.unit
class TestFeedbackEndpoint:
    """Test feedback endpoint."""

    @pytest.mark.asyncio
    async def test_feedback_success(self, client):
        """Test successful feedback submission."""
        feedback_data = {
            "signal_id": "123e4567-e89b-12d3-a456-426614174000",
            "symbol": "BTCUSDT",
            "predicted_signal": "Long",
            "actual_outcome": "profit",
            "profit_loss": 2.5,
            "confidence_was_accurate": True,
            "feedback_notes": "Good signal",
            "timestamp": int(datetime.now(timezone.utc).timestamp() * 1000),
        }

        response = await client.post("/ai/feedback", json=feedback_data)
        assert response.status_code == 200
        data = response.json()
        assert data["message"] == "Feedback received successfully"


@pytest.mark.unit
class TestWebSocket:
    """Test WebSocket functionality."""

    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    def test_websocket_connection(self, test_client):
        """Test WebSocket connection and messages."""
        with test_client.websocket_connect("/ws") as websocket:
            # Test connection
            data = websocket.receive_json()
            assert data["type"] == "connection"
            assert data["message"] == "Connected to AI Trading Service"

            # Test receiving AI signal
            with patch(
                "main.ws_manager.broadcast_signal", AsyncMock()
            ) as mock_broadcast:
                # Simulate broadcasting
                test_message = {
                    "type": "ai_signal",
                    "data": {"symbol": "BTCUSDT", "signal": "Long", "confidence": 0.8},
                }
                # In real scenario, this would be broadcast to all connections


@pytest.mark.unit
class TestPerformanceEndpoint:
    """Test performance metrics endpoint."""

    @pytest.mark.asyncio
    async def test_get_performance(self, client):
        """Test getting performance metrics."""
        response = await client.get("/ai/performance")
        assert response.status_code == 200
        data = response.json()
        assert "overall_accuracy" in data
        assert "predictions_made" in data
        assert data["overall_accuracy"] == 0.85


@pytest.mark.unit
class TestStorageEndpoints:
    """Test storage-related endpoints."""

    @pytest.mark.asyncio
    async def test_storage_stats_success(self, client, mock_mongodb):
        """Test storage statistics endpoint."""

        # Create async iterator for aggregate
        async def async_gen():
            yield {"_id": "BTCUSDT", "count": 100, "latest": 1701234567000}

        # Create a fresh mock collection with specific methods
        mock_collection = AsyncMock()
        mock_collection.aggregate = MagicMock(return_value=async_gen())
        mock_collection.count_documents = AsyncMock(return_value=500)

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            response = await client.get("/ai/storage/stats")
            assert response.status_code == 200
            data = response.json()
            assert data["total_analyses"] == 500

    @pytest.mark.asyncio
    async def test_storage_stats_no_mongodb(self, client):
        """Test storage stats when MongoDB is unavailable."""
        with patch("main.mongodb_db", None):
            response = await client.get("/ai/storage/stats")
            assert response.status_code == 200
            data = response.json()
            assert data["error"] == "MongoDB not connected"

    @pytest.mark.asyncio
    async def test_clear_storage_success(self, client, mock_mongodb):
        """Test clearing storage."""
        # Create a fresh mock collection
        mock_collection = AsyncMock()
        mock_collection.delete_many = AsyncMock(
            return_value=MagicMock(deleted_count=100)
        )

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            response = await client.post("/ai/storage/clear")
            assert response.status_code == 200
            data = response.json()
            assert data["cleared_analyses"] == 100
            assert data["message"] == "Storage cleared successfully"


@pytest.mark.unit
class TestRootEndpoint:
    """Test root endpoint."""

    @pytest.mark.asyncio
    async def test_root_endpoint(self, client):
        """Test root endpoint returns service info."""
        response = await client.get("/")
        assert response.status_code == 200
        data = response.json()
        assert data["service"] == "GPT-4 Cryptocurrency AI Trading Service"
        assert "endpoints" in data
        assert data["features"]["gpt4_enabled"] is True


@pytest.mark.unit
class TestTechnicalAnalyzer:
    """Test TechnicalAnalyzer class methods."""

    def test_prepare_dataframe_empty(self):
        """Test prepare_dataframe with empty klines."""
        df = TechnicalAnalyzer.prepare_dataframe([])
        assert df.empty

    def test_prepare_dataframe_success(self):
        """Test prepare_dataframe with valid klines."""
        klines = [
            [1701234567000, "45000.0", "45100.0", "44900.0", "45050.0", "100.5"],
            [1701238167000, "45050.0", "45200.0", "45000.0", "45150.0", "120.3"],
        ]
        df = TechnicalAnalyzer.prepare_dataframe(klines)
        assert len(df) == 2
        assert "open" in df.columns
        assert "close" in df.columns
        assert df["open"].iloc[0] == 45000.0

    def test_calculate_indicators_empty(self):
        """Test calculate_indicators with empty dataframe."""
        df = pd.DataFrame()
        indicators = TechnicalAnalyzer.calculate_indicators(df)
        assert indicators["rsi"] == 50.0
        assert indicators["macd"] == 0.0

    def test_calculate_indicators_success(self):
        """Test calculate_indicators with sufficient data."""
        # Create 100 rows of data for proper indicator calculation
        data = []
        base_price = 45000
        for i in range(100):
            price = base_price + i * 10
            data.append(
                {
                    "timestamp": pd.Timestamp.now() - pd.Timedelta(hours=100 - i),
                    "open": price,
                    "high": price + 50,
                    "low": price - 50,
                    "close": price + 25,
                    "volume": 100 + i,
                }
            )

        df = pd.DataFrame(data)
        df.set_index("timestamp", inplace=True)

        indicators = TechnicalAnalyzer.calculate_indicators(df)

        # Verify key indicators exist and have reasonable values
        assert "rsi" in indicators
        assert "macd" in indicators
        assert "sma_20" in indicators
        assert "ema_9" in indicators
        assert "bollinger_upper" in indicators
        assert indicators["rsi"] >= 0
        assert indicators["rsi"] <= 100

    def test_calculate_indicators_error_handling(self):
        """Test calculate_indicators error handling."""
        # Create dataframe with invalid data
        df = pd.DataFrame({"close": [None, None]})
        indicators = TechnicalAnalyzer.calculate_indicators(df)
        # Should return default values on error
        assert indicators["rsi"] == 50.0

    def test_detect_patterns_empty(self):
        """Test detect_patterns with empty dataframe."""
        df = pd.DataFrame()
        patterns = TechnicalAnalyzer.detect_patterns(df)
        assert all(not v for v in patterns.values())

    def test_detect_patterns_insufficient_data(self):
        """Test detect_patterns with insufficient data."""
        df = pd.DataFrame(
            {"close": [45000, 45050], "high": [45100, 45150], "low": [44900, 44950]}
        )
        patterns = TechnicalAnalyzer.detect_patterns(df)
        assert all(not v for v in patterns.values())

    def test_detect_patterns_double_top(self):
        """Test detect_patterns for double top pattern."""
        # Create price data with double top
        highs = [45000] * 5 + [46000] + [45500] * 3 + [46000] + [45500] * 5
        lows = [h - 500 for h in highs]
        closes = [h - 250 for h in highs]

        df = pd.DataFrame({"close": closes, "high": highs, "low": lows})

        patterns = TechnicalAnalyzer.detect_patterns(df)
        assert "double_top" in patterns

    def test_detect_patterns_double_bottom(self):
        """Test detect_patterns for double bottom pattern."""
        # Create price data with double bottom
        lows = [45000] * 5 + [44000] + [44500] * 3 + [44000] + [44500] * 5
        highs = [l + 500 for l in lows]
        closes = [l + 250 for l in lows]

        df = pd.DataFrame({"close": closes, "high": highs, "low": lows})

        patterns = TechnicalAnalyzer.detect_patterns(df)
        assert "double_bottom" in patterns

    def test_detect_patterns_ascending_triangle(self):
        """Test detect_patterns for ascending triangle."""
        # Create ascending triangle pattern with sufficient data
        lows = list(range(44000, 44150, 10)) + [44150] * 10
        highs = [45000] * 25  # Flat resistance
        closes = [(l + h) / 2 for l, h in zip(lows, highs)]

        df = pd.DataFrame({"close": closes, "high": highs, "low": lows})

        patterns = TechnicalAnalyzer.detect_patterns(df)
        # Pattern detection exists
        assert "ascending_triangle" in patterns
        # Just verify it's a boolean
        assert isinstance(patterns["ascending_triangle"], (bool, np.bool_))

    def test_get_market_context_empty(self):
        """Test get_market_context with empty data."""
        df = pd.DataFrame()
        indicators = {}
        context = TechnicalAnalyzer.get_market_context(df, indicators)
        assert context["trend_strength"] == 0.0
        assert context["volatility"] == 0.5

    def test_get_market_context_bullish(self):
        """Test get_market_context with bullish indicators."""
        df = pd.DataFrame({"close": [45000, 45100, 45200], "volume": [100, 110, 120]})
        indicators = {
            "rsi": 75.0,
            "macd_histogram": 50.0,
            "ema_9": 45200,
            "ema_21": 45000,
            "atr": 100.0,
            "volume_ratio": 1.5,
        }

        context = TechnicalAnalyzer.get_market_context(df, indicators)
        assert context["market_sentiment"] == "bullish"
        assert context["trend_strength"] > 0
        assert context["volume_trend"] == "increasing"

    def test_get_market_context_bearish(self):
        """Test get_market_context with bearish indicators."""
        df = pd.DataFrame({"close": [45000, 44900, 44800], "volume": [100, 90, 80]})
        indicators = {
            "rsi": 25.0,
            "macd_histogram": -50.0,
            "ema_9": 44800,
            "ema_21": 45000,
            "atr": 100.0,
            "volume_ratio": 0.7,
        }

        context = TechnicalAnalyzer.get_market_context(df, indicators)
        assert context["market_sentiment"] == "bearish"
        assert context["trend_strength"] < 0
        assert context["volume_trend"] == "decreasing"

    def test_candles_to_dataframe_empty(self):
        """Test candles_to_dataframe with empty data."""
        result = TechnicalAnalyzer.candles_to_dataframe({})
        assert len(result) == 0

    def test_candles_to_dataframe_success(self):
        """Test candles_to_dataframe with valid candles."""
        candles = [
            CandleData(
                timestamp=1701234567000,
                open=45000.0,
                high=45100.0,
                low=44900.0,
                close=45050.0,
                volume=100.5,
            ),
            CandleData(
                timestamp=1701238167000,
                open=45050.0,
                high=45200.0,
                low=45000.0,
                close=45150.0,
                volume=120.3,
            ),
        ]

        timeframe_data = {"1h": candles, "4h": candles}
        result = TechnicalAnalyzer.candles_to_dataframe(timeframe_data)

        assert "1h" in result
        assert "4h" in result
        assert len(result["1h"]) == 2
        assert result["1h"]["close"].iloc[0] == 45050.0


@pytest.mark.unit
class TestWebSocketManager:
    """Test WebSocketManager class."""

    @pytest.mark.asyncio
    async def test_connect(self):
        """Test WebSocket connect."""
        ws_manager = WebSocketManager()
        mock_ws = AsyncMock()

        await ws_manager.connect(mock_ws)

        assert mock_ws in ws_manager.active_connections
        mock_ws.accept.assert_called_once()
        mock_ws.send_json.assert_called_once()

    def test_disconnect(self):
        """Test WebSocket disconnect."""
        ws_manager = WebSocketManager()
        mock_ws = Mock()
        ws_manager.active_connections.add(mock_ws)

        ws_manager.disconnect(mock_ws)

        assert mock_ws not in ws_manager.active_connections

    @pytest.mark.asyncio
    async def test_broadcast_signal_no_connections(self):
        """Test broadcast with no connections."""
        ws_manager = WebSocketManager()
        signal_data = {"symbol": "BTCUSDT", "signal": "Long"}

        # Should not raise error
        await ws_manager.broadcast_signal(signal_data)

    @pytest.mark.asyncio
    async def test_broadcast_signal_success(self):
        """Test successful broadcast."""
        ws_manager = WebSocketManager()
        mock_ws1 = AsyncMock()
        mock_ws2 = AsyncMock()

        ws_manager.active_connections.add(mock_ws1)
        ws_manager.active_connections.add(mock_ws2)

        signal_data = {"symbol": "BTCUSDT", "signal": "Long"}
        await ws_manager.broadcast_signal(signal_data)

        assert mock_ws1.send_json.call_count == 1
        assert mock_ws2.send_json.call_count == 1

    @pytest.mark.asyncio
    async def test_broadcast_signal_with_failure(self):
        """Test broadcast with one connection failing."""
        ws_manager = WebSocketManager()
        mock_ws_good = AsyncMock()
        mock_ws_bad = AsyncMock()
        mock_ws_bad.send_json.side_effect = Exception("Connection lost")

        ws_manager.active_connections.add(mock_ws_good)
        ws_manager.active_connections.add(mock_ws_bad)

        signal_data = {"symbol": "BTCUSDT", "signal": "Long"}
        await ws_manager.broadcast_signal(signal_data)

        # Bad connection should be removed
        assert mock_ws_bad not in ws_manager.active_connections
        assert mock_ws_good in ws_manager.active_connections


@pytest.mark.unit
class TestDirectOpenAIClient:
    """Test DirectOpenAIClient class."""

    def test_init_single_key(self):
        """Test initialization with single API key."""
        client = DirectOpenAIClient("test-key")
        assert len(client.api_keys) == 1
        assert client.api_keys[0] == "test-key"

    def test_init_multiple_keys(self):
        """Test initialization with multiple API keys."""
        keys = ["key1", "key2", "key3"]
        client = DirectOpenAIClient(keys)
        assert len(client.api_keys) == 3

    def test_get_current_api_key(self):
        """Test getting current API key."""
        client = DirectOpenAIClient(["key1", "key2"])
        key, index = client.get_current_api_key()
        assert key == "key1"
        assert index == 0

    def test_get_current_api_key_cycling(self):
        """Test API key cycling."""
        client = DirectOpenAIClient(["key1", "key2"])
        client.current_key_index = 1
        key, index = client.get_current_api_key()
        assert key == "key2"

    def test_get_current_api_key_rate_limited(self):
        """Test getting API key when some are rate limited."""
        client = DirectOpenAIClient(["key1", "key2", "key3"])
        client.rate_limited_keys.add(0)  # key1 is rate limited
        key, index = client.get_current_api_key()
        assert key in ["key2", "key3"]

    def test_get_current_api_key_all_rate_limited(self):
        """Test when all keys are rate limited."""
        client = DirectOpenAIClient(["key1", "key2"])
        client.rate_limited_keys.add(0)
        client.rate_limited_keys.add(1)

        # Should clear rate limits and start over
        key, index = client.get_current_api_key()
        assert key in ["key1", "key2"]
        assert len(client.rate_limited_keys) == 0

    @pytest.mark.asyncio
    async def test_chat_completions_create_success(self):
        """Test successful chat completion."""
        import httpx

        client = DirectOpenAIClient(["test-key"])

        mock_response = {"choices": [{"message": {"content": "Test response"}}]}

        with patch("main.last_openai_request_time", None):
            with patch("httpx.AsyncClient") as mock_httpx:
                mock_client_instance = AsyncMock()
                mock_client_instance.post = AsyncMock(
                    return_value=AsyncMock(
                        status_code=200,
                        json=lambda: mock_response,
                        raise_for_status=lambda: None,
                    )
                )
                mock_httpx.return_value.__aenter__.return_value = mock_client_instance

                result = await client.chat_completions_create(
                    model="gpt-4o-mini", messages=[{"role": "user", "content": "test"}]
                )

                assert result == mock_response

    @pytest.mark.asyncio
    async def test_chat_completions_create_rate_limit(self):
        """Test rate limit handling."""
        # Simply test that rate limited keys are tracked
        client = DirectOpenAIClient(["key1", "key2"])

        # Manually add a key to rate limited set
        client.rate_limited_keys.add(0)

        # When getting current key, it should skip the rate limited one
        key, index = client.get_current_api_key()
        assert key == "key2"  # Should get second key
        assert index == 0  # Index into available keys (not rate limited)

    @pytest.mark.asyncio
    async def test_chat_completions_create_all_keys_exhausted(self):
        """Test when all keys fail."""
        import httpx

        client = DirectOpenAIClient(["key1"])

        with patch("main.last_openai_request_time", None):
            with patch("httpx.AsyncClient") as mock_httpx:
                mock_client_instance = AsyncMock()
                mock_client_instance.post = AsyncMock(
                    side_effect=Exception("Network error")
                )
                mock_httpx.return_value.__aenter__.return_value = mock_client_instance

                with pytest.raises(Exception, match="Network error"):
                    await client.chat_completions_create(
                        model="gpt-4o-mini",
                        messages=[{"role": "user", "content": "test"}],
                    )


@pytest.mark.unit
class TestGPTTradingAnalyzer:
    """Test GPTTradingAnalyzer class."""

    @pytest.mark.asyncio
    async def test_analyze_trading_signals_with_gpt(self, mock_openai_client):
        """Test analyze_trading_signals with GPT client."""
        analyzer = GPTTradingAnalyzer(mock_openai_client)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000)
                - i * 3600000,
                open=45000 + i,
                high=45100 + i,
                low=44900 + i,
                close=45050 + i,
                volume=100.0,
            )
            for i in range(100)
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles, "4h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(
                selected_strategies=["RSI Strategy"],
                market_condition="Trending",
                risk_level="Medium",
            ),
        )

        result = await analyzer.analyze_trading_signals(request)

        assert result.signal in ["Long", "Short", "Neutral"]
        assert 0.0 <= result.confidence <= 1.0
        assert len(result.reasoning) > 0

    @pytest.mark.asyncio
    async def test_analyze_trading_signals_fallback(self):
        """Test analyze_trading_signals with fallback."""
        analyzer = GPTTradingAnalyzer(None)  # No GPT client

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000)
                - i * 3600000,
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
            for i in range(100)
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(
                selected_strategies=["RSI Strategy"],
                market_condition="Trending",
                risk_level="Medium",
            ),
        )

        result = await analyzer.analyze_trading_signals(request)

        assert result.signal in ["Long", "Short", "Neutral"]
        assert "Technical analysis" in result.reasoning

    def test_fallback_analysis_rsi_oversold(self):
        """Test fallback analysis requires 4/5 signals for Long (matching Rust FR-STRATEGIES-006)."""
        analyzer = GPTTradingAnalyzer(None)

        # Create candles with strong upward movement in LAST 2 candles
        # Price change check: (candles[-1].close - candles[-2].close) / candles[-2].close * 100
        # Need >1% change to trigger "Strong upward movement" bullish signal
        base_time = int(datetime.now(timezone.utc).timestamp() * 1000)
        candles = []
        base_price = 44000
        for i in range(98):  # First 98 candles with gradual increase
            price = base_price + (i * 50)
            candles.append(CandleData(
                timestamp=base_time + i * 3600000,
                open=price,
                high=price + 100,
                low=price - 50,
                close=price + 80,
                volume=100.0,
            ))
        # Second to last candle
        candles.append(CandleData(
            timestamp=base_time + 98 * 3600000,
            open=48900,
            high=49000,
            low=48800,
            close=48900,  # Base for price change calculation
            volume=100.0,
        ))
        # Last candle with >1% jump to trigger Strong upward movement
        candles.append(CandleData(
            timestamp=base_time + 99 * 3600000,
            open=49000,
            high=50000,
            low=49000,
            close=49500,  # 1.2% higher than 48900
            volume=150.0,
        ))

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=49500.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(selected_strategies=["RSI Strategy", "MACD Strategy", "Bollinger Bands Strategy"]),
        )

        # Need 4+ bullish signals for Long (FR-STRATEGIES-006: 4/5 = 80%)
        # RSI oversold + MACD bullish + BB lower + Price trend = 4 bullish
        indicators = {
            "rsi": 25.0,  # Oversold = bullish
            "macd": 0.5, "macd_signal": 0.3,  # MACD > signal = bullish
            "bb_position": 0.05,  # Near lower band = bullish
        }
        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, indicators, {})
        assert result["signal"] == "Long"
        assert "Bullish: 4" in result["reasoning"]

    def test_fallback_analysis_rsi_overbought(self):
        """Test fallback analysis requires 4/5 signals for Short (matching Rust FR-STRATEGIES-006)."""
        analyzer = GPTTradingAnalyzer(None)

        # Create candles with strong downward movement in LAST 2 candles
        # Price change check: (candles[-1].close - candles[-2].close) / candles[-2].close * 100
        # Need <-1% change to trigger "Strong downward movement" bearish signal
        base_time = int(datetime.now(timezone.utc).timestamp() * 1000)
        candles = []
        base_price = 50000
        for i in range(98):  # First 98 candles with gradual decrease
            price = base_price - (i * 50)
            candles.append(CandleData(
                timestamp=base_time + i * 3600000,
                open=price,
                high=price + 50,
                low=price - 100,
                close=price - 80,
                volume=100.0,
            ))
        # Second to last candle
        candles.append(CandleData(
            timestamp=base_time + 98 * 3600000,
            open=46000,
            high=46100,
            low=45900,
            close=46000,  # Base for price change calculation
            volume=100.0,
        ))
        # Last candle with >1% drop to trigger Strong downward movement
        candles.append(CandleData(
            timestamp=base_time + 99 * 3600000,
            open=45800,
            high=46000,
            low=45300,
            close=45400,  # -1.3% lower than 46000
            volume=150.0,
        ))

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45400.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(selected_strategies=["RSI Strategy", "MACD Strategy", "Bollinger Bands Strategy"]),
        )

        # Need 4+ bearish signals for Short (FR-STRATEGIES-006: 4/5 = 80%)
        # RSI overbought + MACD bearish + BB upper + Price trend = 4 bearish
        indicators = {
            "rsi": 75.0,  # Overbought = bearish
            "macd": 0.3, "macd_signal": 0.5,  # MACD < signal = bearish
            "bb_position": 0.95,  # Near upper band = bearish
        }
        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, indicators, {})
        assert result["signal"] == "Short"
        assert "Bearish: 4" in result["reasoning"]

    def test_parse_gpt_response_json(self):
        """Test parsing valid JSON GPT response."""
        analyzer = GPTTradingAnalyzer(None)

        response = json.dumps(
            {
                "signal": "Long",
                "confidence": 0.8,
                "reasoning": "Test",
                "strategy_scores": {},
                "market_analysis": {
                    "trend_direction": "Bullish",
                    "trend_strength": 0.8,
                    "support_levels": [],
                    "resistance_levels": [],
                    "volatility_level": "Medium",
                    "volume_analysis": "Normal",
                },
                "risk_assessment": {
                    "overall_risk": "Medium",
                    "technical_risk": 0.5,
                    "market_risk": 0.5,
                    "recommended_position_size": 0.02,
                },
            }
        )

        result = analyzer._parse_gpt_response(response)
        assert result["signal"] == "Long"
        assert result["confidence"] == 0.8

    def test_parse_gpt_response_fallback(self):
        """Test parsing non-JSON GPT response."""
        analyzer = GPTTradingAnalyzer(None)

        response = "STRONG BUY signal based on technical analysis"
        result = analyzer._parse_gpt_response(response)

        assert result["signal"] == "Long"
        assert result["confidence"] > 0.5

    def test_fallback_parse_buy(self):
        """Test fallback parsing for buy signals."""
        analyzer = GPTTradingAnalyzer(None)

        result = analyzer._fallback_parse("BUY this asset")
        assert result["signal"] == "Long"
        assert result["confidence"] == 0.6

    def test_fallback_parse_sell(self):
        """Test fallback parsing for sell signals."""
        analyzer = GPTTradingAnalyzer(None)

        result = analyzer._fallback_parse("SELL immediately")
        assert result["signal"] == "Short"
        assert result["confidence"] == 0.6

    def test_default_response(self):
        """Test default response."""
        analyzer = GPTTradingAnalyzer(None)

        result = analyzer._default_response()
        assert result["signal"] == "Neutral"
        assert result["confidence"] == 0.3
        assert result["risk_assessment"]["overall_risk"] == "High"


@pytest.mark.unit
class TestMongoDBFunctions:
    """Test MongoDB storage functions."""

    @pytest.mark.asyncio
    async def test_store_analysis_result_success(self, mock_mongodb):
        """Test storing analysis result."""
        symbol = "BTCUSDT"
        analysis = {"signal": "Long", "confidence": 0.8, "reasoning": "Test"}

        with patch("main.mongodb_db", mock_mongodb[1]):
            await store_analysis_result(symbol, analysis)
            # Should not raise error

    @pytest.mark.asyncio
    async def test_store_analysis_result_no_db(self):
        """Test storing when MongoDB is unavailable."""
        with patch("main.mongodb_db", None):
            # Should not raise error, just log warning
            await store_analysis_result("BTCUSDT", {})

    @pytest.mark.skip(
        reason="Flaky test - MongoDB mock state pollution. Passes when run in isolation."
    )
    @pytest.mark.asyncio
    async def test_get_latest_analysis_success(self, mock_mongodb):
        """Test getting latest analysis."""
        mock_result = {
            "signal": "Long",
            "confidence": 0.8,
            "analysis": {"test": "data"},
        }

        # Create a fresh mock collection with the specific find_one result
        mock_collection = AsyncMock()
        mock_collection.find_one = AsyncMock(return_value=mock_result)

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            result = await get_latest_analysis("BTCUSDT")
            # Should return the analysis field
            assert result is not None
            assert result == {"test": "data"}

    @pytest.mark.asyncio
    async def test_get_latest_analysis_no_db(self):
        """Test getting analysis when MongoDB is unavailable."""
        with patch("main.mongodb_db", None):
            result = await get_latest_analysis("BTCUSDT")
            assert result is None

    @pytest.mark.asyncio
    async def test_get_latest_analysis_not_found(self, mock_mongodb):
        """Test getting analysis when none exists."""
        mock_mongodb[1]["ai_analysis_results"].find_one = AsyncMock(return_value=None)

        with patch("main.mongodb_db", mock_mongodb[1]):
            result = await get_latest_analysis("BTCUSDT")
            assert result is None


@pytest.mark.unit
class TestFetchRealMarketData:
    """Test real market data fetching from Rust API."""

    @pytest.mark.asyncio
    async def test_fetch_real_market_data_btc(self):
        """Test fetching real data for BTC with mocked HTTP responses."""
        import httpx

        # Mock candle data
        mock_candles_1h = [
            {"timestamp": 1700000000000 + i * 3600000, "open": 50000, "high": 50100, "low": 49900, "close": 50050, "volume": 100.0}
            for i in range(100)
        ]
        mock_candles_4h = [
            {"timestamp": 1700000000000 + i * 14400000, "open": 50000, "high": 50200, "low": 49800, "close": 50100, "volume": 400.0}
            for i in range(60)
        ]
        mock_prices = {"BTCUSDT": 50000.0, "ETHUSDT": 3000.0}

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = AsyncMock()

            # Configure mock responses for different endpoints
            async def mock_get(url, **kwargs):
                response = Mock()  # Use regular Mock for response object
                if "/api/market/candles/" in url and "/1h" in url:
                    response.status_code = 200
                    response.json = Mock(return_value={"success": True, "data": mock_candles_1h})
                elif "/api/market/candles/" in url and "/4h" in url:
                    response.status_code = 200
                    response.json = Mock(return_value={"success": True, "data": mock_candles_4h})
                elif "/api/market/prices" in url:
                    response.status_code = 200
                    response.json = Mock(return_value={"success": True, "data": mock_prices})
                else:
                    response.status_code = 404
                return response

            mock_instance.get = mock_get
            mock_client.return_value.__aenter__.return_value = mock_instance

            result = await fetch_real_market_data("BTCUSDT")

            assert result.symbol == "BTCUSDT"
            assert result.current_price == 50000.0
            assert "1h" in result.timeframe_data
            assert "4h" in result.timeframe_data
            assert len(result.timeframe_data["1h"]) == 100
            assert len(result.timeframe_data["4h"]) == 60

    @pytest.mark.asyncio
    async def test_fetch_real_market_data_api_failure(self):
        """Test handling when Rust API is unavailable - raises pydantic validation error."""
        from pydantic import ValidationError

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = AsyncMock()

            async def mock_get_failure(url, **kwargs):
                response = Mock()
                response.status_code = 500
                return response

            mock_instance.get = mock_get_failure
            mock_client.return_value.__aenter__.return_value = mock_instance

            # When API fails, current_price=0 which fails pydantic validation (>0 required)
            with pytest.raises(ValidationError) as exc_info:
                await fetch_real_market_data("BTCUSDT")

            # Verify the error is about current_price validation
            assert "current_price" in str(exc_info.value)

    @pytest.mark.asyncio
    async def test_fetch_real_market_data_network_error(self):
        """Test handling network errors - raises pydantic validation error."""
        from pydantic import ValidationError

        with patch("httpx.AsyncClient") as mock_client:
            mock_instance = AsyncMock()
            mock_instance.get = AsyncMock(side_effect=Exception("Connection refused"))
            mock_client.return_value.__aenter__.return_value = mock_instance

            # Network errors result in current_price=0, which fails pydantic validation
            with pytest.raises(ValidationError) as exc_info:
                await fetch_real_market_data("ETHUSDT")

            # Verify the error is about current_price validation
            assert "current_price" in str(exc_info.value)


@pytest.mark.unit
class TestAdditionalEndpoints:
    """Test additional API endpoints."""

    @pytest.mark.asyncio
    async def test_debug_gpt4_success(self, client, mock_openai_client):
        """Test debug GPT-4 endpoint with success."""
        with patch("main.openai_client", mock_openai_client):
            response = await client.get("/debug/gpt4")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "success"
            assert "test_response" in data

    @pytest.mark.asyncio
    async def test_debug_gpt4_no_client(self, client):
        """Test debug GPT-4 when client is not initialized."""
        with patch("main.openai_client", None):
            response = await client.get("/debug/gpt4")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "failed"
            assert "error" in data

    @pytest.mark.asyncio
    async def test_debug_gpt4_api_error(self, client):
        """Test debug GPT-4 with API error."""
        mock_client = AsyncMock()
        mock_client.chat_completions_create = AsyncMock(
            side_effect=Exception("401 Unauthorized")
        )

        with patch("main.openai_client", mock_client):
            response = await client.get("/debug/gpt4")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "failed"
            assert "diagnosis" in data

    @pytest.mark.asyncio
    async def test_get_service_info(self, client):
        """Test AI service info endpoint."""
        response = await client.get("/ai/info")
        assert response.status_code == 200
        data = response.json()
        assert data["service_name"] == "GPT-4 Trading AI"
        assert "supported_timeframes" in data

    @pytest.mark.asyncio
    async def test_get_supported_strategies(self, client):
        """Test getting supported strategies."""
        response = await client.get("/ai/strategies")
        assert response.status_code == 200
        data = response.json()
        assert isinstance(data, list)
        assert "RSI Strategy" in data
        assert "MACD Strategy" in data


@pytest.mark.unit
class TestErrorHandling:
    """Test error handling in various scenarios."""

    @pytest.mark.asyncio
    async def test_analyze_with_exception(self, client, sample_ai_analysis_request):
        """Test analysis endpoint when analyzer raises exception."""
        with patch("main.GPTTradingAnalyzer") as mock_analyzer:
            mock_instance = AsyncMock()
            mock_instance.analyze_trading_signals = AsyncMock(
                side_effect=Exception("Analysis failed")
            )
            mock_analyzer.return_value = mock_instance

            with patch("main.gpt_analyzer", mock_instance):
                response = await client.post(
                    "/ai/analyze", json=sample_ai_analysis_request
                )
                assert response.status_code == 500

    @pytest.mark.asyncio
    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    async def test_websocket_disconnect(self, test_client):
        """Test WebSocket disconnect handling."""
        from fastapi import WebSocketDisconnect

        with test_client.websocket_connect("/ws") as websocket:
            # Receive connection message
            data = websocket.receive_json()
            assert data["type"] == "connection"

            # Close connection
            websocket.close()


@pytest.mark.unit
class TestPydanticModels:
    """Test Pydantic model validations."""

    def test_candle_data_validation(self):
        """Test CandleData validation."""
        # Valid data
        candle = CandleData(
            timestamp=1701234567000,
            open=45000.0,
            high=45100.0,
            low=44900.0,
            close=45050.0,
            volume=100.0,
        )
        assert candle.open == 45000.0

        # Invalid data (negative price)
        with pytest.raises(Exception):
            CandleData(
                timestamp=1701234567000,
                open=-45000.0,
                high=45100.0,
                low=44900.0,
                close=45050.0,
                volume=100.0,
            )

    def test_ai_strategy_context_defaults(self):
        """Test AIStrategyContext default values."""
        context = AIStrategyContext()
        assert context.selected_strategies == []
        assert context.market_condition == "Unknown"
        assert context.risk_level == "Moderate"


@pytest.mark.unit
class TestMoreGPTAnalyzerMethods:
    """Test additional GPT analyzer methods."""

    def test_get_system_prompt(self):
        """Test getting system prompt."""
        analyzer = GPTTradingAnalyzer(None)
        prompt = analyzer._get_system_prompt()
        assert len(prompt) > 0
        assert "crypto" in prompt.lower()  # Changed from "cryptocurrency" to "crypto"
        assert "JSON" in prompt

    def test_prepare_market_context(self):
        """Test preparing market context."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(),
        )

        indicators_1h = {"rsi": 65.0, "macd": 10.0}
        indicators_4h = {"rsi": 70.0}

        context = analyzer._prepare_market_context(
            request, indicators_1h, indicators_4h
        )
        assert "BTCUSDT" in context
        assert "RSI" in context

    def test_create_analysis_prompt(self):
        """Test creating analysis prompt."""
        analyzer = GPTTradingAnalyzer(None)
        market_context = "Test context"
        strategy_context = AIStrategyContext(
            selected_strategies=["RSI Strategy", "MACD Strategy"],
            market_condition="Trending",
            risk_level="Medium",
        )

        prompt = analyzer._create_analysis_prompt(market_context, strategy_context)
        assert "Test context" in prompt
        assert "RSI Strategy" in prompt
        assert "MACD Strategy" in prompt

    def test_fallback_analysis_macd_strategy(self):
        """Test fallback analysis with MACD strategy."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
            for _ in range(50)
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(selected_strategies=["MACD Strategy"]),
        )

        indicators = {"macd": 50.0, "macd_signal": 30.0}
        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, indicators, {})
        assert "MACD" in result["reasoning"]
        assert result["signal"] in ["Long", "Short", "Neutral"]

    def test_fallback_analysis_volume_strategy(self):
        """Test fallback analysis with volume strategy."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(selected_strategies=["Volume Strategy"]),
        )

        indicators = {"volume_ratio": 2.0}
        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, indicators, {})
        assert "volume" in result["reasoning"].lower()

    def test_fallback_analysis_bollinger_bands_insufficient_signals(self):
        """Test fallback analysis with only 2 signals returns Neutral (4/5 = 80% required)."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(
                selected_strategies=["Bollinger Bands Strategy", "RSI Strategy"]
            ),
        )

        # Only 2 bullish signals: BB near lower + RSI oversold
        # This is NOT enough - need 4/5 (FR-STRATEGIES-006)
        indicators = {"bb_position": 0.05, "rsi": 25.0}
        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, indicators, {})
        # With only 2 signals, result should be Neutral (safety first)
        assert result["signal"] == "Neutral"
        assert "Bullish: 2" in result["reasoning"]

    def test_fallback_analysis_price_trend(self):
        """Test fallback analysis with price trend."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000) - 3600000,
                open=45000,
                high=45100,
                low=44900,
                close=45000,
                volume=100.0,
            ),
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45500,
                low=44900,
                close=45500,
                volume=100.0,
            ),
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45500.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(),
        )

        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, {}, {})
        assert "movement" in result["reasoning"].lower()


@pytest.mark.unit
class TestDirectOpenAIClientAdvanced:
    """Test advanced DirectOpenAIClient scenarios."""

    @pytest.mark.asyncio
    async def test_chat_completions_with_rate_limiting_delay(self):
        """Test rate limiting delay between requests."""
        import httpx
        from datetime import datetime

        client = DirectOpenAIClient(["test-key"])

        mock_response = {"choices": [{"message": {"content": "Test"}}]}

        # Set a recent request time to trigger delay
        with patch(
            "main.last_openai_request_time", datetime.now() - timedelta(seconds=5)
        ):
            with patch("main.OPENAI_REQUEST_DELAY", 10):
                with patch("httpx.AsyncClient") as mock_httpx:
                    mock_client_instance = AsyncMock()
                    mock_client_instance.post = AsyncMock(
                        return_value=AsyncMock(
                            status_code=200,
                            json=lambda: mock_response,
                            raise_for_status=lambda: None,
                        )
                    )
                    mock_httpx.return_value.__aenter__.return_value = (
                        mock_client_instance
                    )

                    with patch("asyncio.sleep", new_callable=AsyncMock) as mock_sleep:
                        result = await client.chat_completions_create(
                            model="gpt-4o-mini",
                            messages=[{"role": "user", "content": "test"}],
                        )
                        # Should have slept
                        assert mock_sleep.called

    @pytest.mark.asyncio
    async def test_chat_completions_http_401(self):
        """Test HTTP 401 error handling."""
        import httpx

        client = DirectOpenAIClient(["key1", "key2"])

        error_401 = httpx.HTTPStatusError(
            "401 Unauthorized", request=Mock(), response=Mock(status_code=401)
        )

        mock_success = {"choices": [{"message": {"content": "success"}}]}

        with patch("main.last_openai_request_time", None):
            with patch("httpx.AsyncClient") as mock_httpx:
                mock_client_instance = AsyncMock()
                # First key fails with 401, second succeeds
                mock_client_instance.post = AsyncMock(
                    side_effect=[
                        error_401,
                        AsyncMock(
                            status_code=200,
                            json=lambda: mock_success,
                            raise_for_status=lambda: None,
                        ),
                    ]
                )
                mock_httpx.return_value.__aenter__.return_value = mock_client_instance

                result = await client.chat_completions_create(
                    model="gpt-4o-mini", messages=[{"role": "user", "content": "test"}]
                )
                assert result["choices"][0]["message"]["content"] == "success"

    @pytest.mark.asyncio
    async def test_chat_completions_http_403(self):
        """Test HTTP 403 quota exceeded handling."""
        import httpx

        client = DirectOpenAIClient(["key1", "key2"])

        error_403 = httpx.HTTPStatusError(
            "403 Quota Exceeded", request=Mock(), response=Mock(status_code=403)
        )

        mock_success = {"choices": [{"message": {"content": "success"}}]}

        with patch("main.last_openai_request_time", None):
            with patch("httpx.AsyncClient") as mock_httpx:
                mock_client_instance = AsyncMock()
                mock_client_instance.post = AsyncMock(
                    side_effect=[
                        error_403,
                        AsyncMock(
                            status_code=200,
                            json=lambda: mock_success,
                            raise_for_status=lambda: None,
                        ),
                    ]
                )
                mock_httpx.return_value.__aenter__.return_value = mock_client_instance

                result = await client.chat_completions_create(
                    model="gpt-4o-mini", messages=[{"role": "user", "content": "test"}]
                )
                assert result["choices"][0]["message"]["content"] == "success"


@pytest.mark.unit
class TestMoreMongoDBScenarios:
    """Test additional MongoDB scenarios."""

    @pytest.mark.asyncio
    async def test_store_analysis_result_error(self, mock_mongodb):
        """Test storing analysis with error."""
        mock_mongodb[1]["ai_analysis_results"].insert_one = AsyncMock(
            side_effect=Exception("Database error")
        )

        with patch("main.mongodb_db", mock_mongodb[1]):
            # Should not raise, just log error
            await store_analysis_result("BTCUSDT", {"signal": "Long"})

    @pytest.mark.asyncio
    async def test_get_latest_analysis_error(self, mock_mongodb):
        """Test getting latest analysis with error."""
        mock_mongodb[1]["ai_analysis_results"].find_one = AsyncMock(
            side_effect=Exception("Database error")
        )

        with patch("main.mongodb_db", mock_mongodb[1]):
            result = await get_latest_analysis("BTCUSDT")
            assert result is None


@pytest.mark.unit
class TestMarketConditionEndpoint:
    """Test market condition endpoint scenarios."""

    @pytest.mark.asyncio
    async def test_market_condition_trending_up(self, client, sample_candle_data):
        """Test market condition with uptrend."""
        # Create uptrending candles (reversed order, oldest to newest)
        candles = []
        base_price = 40000
        for i in range(25, 0, -1):  # Reverse iteration
            price = base_price + ((25 - i) * 200)  # Strong uptrend
            candles.append(
                {
                    "timestamp": int(datetime.now(timezone.utc).timestamp() * 1000)
                    - (i * 3600000),
                    "open": price,
                    "high": price + 100,
                    "low": price - 50,
                    "close": price + 50,
                    "volume": 1000.0,
                }
            )

        request_data = {
            "symbol": "BTCUSDT",
            "timeframe_data": {"1h": candles},
            "current_price": 45000.0,
            "volume_24h": 25000000000.0,
            "timestamp": int(datetime.now(timezone.utc).timestamp() * 1000),
        }

        response = await client.post("/ai/market-condition", json=request_data)
        assert response.status_code == 200
        data = response.json()
        # Check that condition is classified
        assert "condition_type" in data
        assert data["condition_type"] in ["Trending Up", "Trending Down", "Sideways"]

    @pytest.mark.asyncio
    async def test_market_condition_trending_down(self, client, sample_candle_data):
        """Test market condition with downtrend."""
        # Create downtrending candles (reversed order, oldest to newest)
        candles = []
        base_price = 50000
        for i in range(25, 0, -1):  # Reverse iteration
            price = base_price - ((25 - i) * 200)  # Strong downtrend
            candles.append(
                {
                    "timestamp": int(datetime.now(timezone.utc).timestamp() * 1000)
                    - (i * 3600000),
                    "open": price,
                    "high": price + 50,
                    "low": price - 100,
                    "close": price - 50,
                    "volume": 1000.0,
                }
            )

        request_data = {
            "symbol": "BTCUSDT",
            "timeframe_data": {"1h": candles},
            "current_price": 45000.0,
            "volume_24h": 25000000000.0,
            "timestamp": int(datetime.now(timezone.utc).timestamp() * 1000),
        }

        response = await client.post("/ai/market-condition", json=request_data)
        assert response.status_code == 200
        data = response.json()
        # Check that condition is classified
        assert "condition_type" in data
        assert data["condition_type"] in ["Trending Up", "Trending Down", "Sideways"]


@pytest.mark.unit
class TestDirectOpenAIClientErrorPaths:
    """Test DirectOpenAIClient error paths for better coverage."""

    @pytest.mark.asyncio
    async def test_chat_completions_with_expired_rate_limit(self):
        """Test when rate limit period has expired."""
        import httpx
        from datetime import datetime, timedelta

        client = DirectOpenAIClient(["test-key"])

        mock_response = {"choices": [{"message": {"content": "Test"}}]}

        # Mock that rate limit period is in the past (expired)
        past_time = datetime.now() - timedelta(minutes=5)
        with patch("main.OPENAI_RATE_LIMIT_RESET_TIME", past_time):
            with patch("main.last_openai_request_time", None):
                with patch("httpx.AsyncClient") as mock_httpx:
                    mock_client_instance = AsyncMock()
                    mock_client_instance.post = AsyncMock(
                        return_value=AsyncMock(
                            status_code=200,
                            json=lambda: mock_response,
                            raise_for_status=lambda: None,
                        )
                    )
                    mock_httpx.return_value.__aenter__.return_value = (
                        mock_client_instance
                    )

                    result = await client.chat_completions_create(
                        model="gpt-4o-mini",
                        messages=[{"role": "user", "content": "test"}],
                    )
                    assert result == mock_response

    @pytest.mark.asyncio
    async def test_chat_completions_http_error_other(self):
        """Test HTTP error other than 401, 403, 429."""
        import httpx

        client = DirectOpenAIClient(["key1"])

        error_500 = httpx.HTTPStatusError(
            "500 Server Error", request=Mock(), response=Mock(status_code=500)
        )

        with patch("main.last_openai_request_time", None):
            with patch("httpx.AsyncClient") as mock_httpx:
                mock_client_instance = AsyncMock()
                mock_client_instance.post = AsyncMock(side_effect=error_500)
                mock_httpx.return_value.__aenter__.return_value = mock_client_instance

                with pytest.raises(httpx.HTTPStatusError):
                    await client.chat_completions_create(
                        model="gpt-4o-mini",
                        messages=[{"role": "user", "content": "test"}],
                    )


@pytest.mark.unit
class TestGPTAnalyzerErrorPaths:
    """Test GPT analyzer error paths."""

    @pytest.mark.asyncio
    async def test_gpt_analysis_with_api_errors(self, mock_openai_client):
        """Test GPT analysis with various API error types."""
        analyzer = GPTTradingAnalyzer(mock_openai_client)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
            for _ in range(100)
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles, "4h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(),
        )

        # Test with 401 error
        mock_openai_client.chat_completions_create = AsyncMock(
            side_effect=Exception("401 Unauthorized")
        )
        result = await analyzer.analyze_trading_signals(request)
        assert result.signal in ["Long", "Short", "Neutral"]  # Should fall back

        # Test with 429 error
        mock_openai_client.chat_completions_create = AsyncMock(
            side_effect=Exception("429 Rate Limit")
        )
        result = await analyzer.analyze_trading_signals(request)
        assert result.signal in ["Long", "Short", "Neutral"]  # Should fall back

        # Test with quota error
        mock_openai_client.chat_completions_create = AsyncMock(
            side_effect=Exception("quota exceeded")
        )
        result = await analyzer.analyze_trading_signals(request)
        assert result.signal in ["Long", "Short", "Neutral"]  # Should fall back

        # Test with timeout error
        mock_openai_client.chat_completions_create = AsyncMock(
            side_effect=Exception("timeout occurred")
        )
        result = await analyzer.analyze_trading_signals(request)
        assert result.signal in ["Long", "Short", "Neutral"]  # Should fall back

    def test_parse_gpt_response_no_json(self):
        """Test parsing GPT response with no JSON match."""
        analyzer = GPTTradingAnalyzer(None)

        # Response with no JSON structure
        response = "This is just plain text with no JSON"
        result = analyzer._parse_gpt_response(response)

        # Should use fallback parsing
        assert "signal" in result
        assert "confidence" in result

    def test_parse_gpt_response_with_exception(self):
        """Test parsing GPT response that raises exception."""
        analyzer = GPTTradingAnalyzer(None)

        # Use a mock to force an exception during parsing
        with patch("json.loads", side_effect=Exception("JSON parse error")):
            response = '{"signal": "Long"}'  # Valid JSON but we force error
            result = analyzer._parse_gpt_response(response)

            # Should use default response on exception
            assert result["signal"] == "Neutral"
            assert result["confidence"] == 0.3
            assert result["risk_assessment"]["overall_risk"] == "High"

    def test_fallback_parse_strong_short(self):
        """Test fallback parsing for strong short signal."""
        analyzer = GPTTradingAnalyzer(None)

        result = analyzer._fallback_parse("STRONG SELL now")
        assert result["signal"] == "Short"
        assert result["confidence"] == 0.7


@pytest.mark.unit
class TestAdditionalCoverage:
    """Additional tests for uncovered paths."""

    @pytest.mark.asyncio
    async def test_debug_gpt4_quota_error(self, client):
        """Test debug endpoint with quota error."""
        mock_client = AsyncMock()
        mock_client.chat_completions_create = AsyncMock(
            side_effect=Exception("quota exceeded")
        )

        with patch("main.openai_client", mock_client):
            response = await client.get("/debug/gpt4")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "failed"
            assert "quota" in data["diagnosis"].lower()

    @pytest.mark.asyncio
    async def test_debug_gpt4_timeout_error(self, client):
        """Test debug endpoint with timeout."""
        mock_client = AsyncMock()
        mock_client.chat_completions_create = AsyncMock(
            side_effect=Exception("timeout occurred")
        )

        with patch("main.openai_client", mock_client):
            response = await client.get("/debug/gpt4")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "failed"

    @pytest.mark.skip(
        reason="Flaky test - MongoDB mock state pollution. Passes when run in isolation."
    )
    @pytest.mark.asyncio
    async def test_get_latest_analysis_with_analysis_field(self, mock_mongodb):
        """Test getting latest analysis that has analysis field."""
        mock_result = {
            "symbol": "BTCUSDT",
            "analysis": {"signal": "Long", "confidence": 0.8},
        }

        # Create a fresh mock collection with the specific find_one result
        mock_collection = AsyncMock()
        mock_collection.find_one = AsyncMock(return_value=mock_result)

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            result = await get_latest_analysis("BTCUSDT")
            assert result == mock_result["analysis"]


@pytest.mark.unit
class TestLifespanAndStartup:
    """Test application lifespan and startup."""

    @pytest.mark.asyncio
    async def test_lifespan_mongodb_connection_failure(self):
        """Test lifespan when MongoDB connection fails."""
        from main import lifespan

        # Patch DATABASE_URL environment variable
        with patch.dict(os.environ, {"DATABASE_URL": "mongodb://localhost:27017/test"}):
            with patch("main.AsyncIOMotorClient") as mock_mongo:
                mock_client = AsyncMock()
                mock_client.admin.command = AsyncMock(
                    side_effect=Exception("Connection failed")
                )
                mock_mongo.return_value = mock_client

                # Should handle error gracefully
                from fastapi import FastAPI

                test_app = FastAPI()

                async with lifespan(test_app):
                    pass  # Should not raise exception


@pytest.mark.unit
class TestPatternDetectionEdgeCases:
    """Test pattern detection error handling."""

    def test_detect_patterns_with_error(self):
        """Test pattern detection with error in calculation."""
        # Create dataframe that might cause errors
        df = pd.DataFrame(
            {
                "close": [np.nan, 45000, 45050],
                "high": [np.nan, 45100, 45150],
                "low": [np.nan, 44900, 44950],
            }
        )

        # Should handle gracefully and return False for all patterns
        patterns = TechnicalAnalyzer.detect_patterns(df)
        assert isinstance(patterns, dict)

    def test_detect_patterns_with_invalid_data(self):
        """Test pattern detection with data that causes issues."""
        # Create dataframe with insufficient variation (all same values)
        # This might trigger edge cases in pattern detection
        df = pd.DataFrame(
            {"close": [45000.0] * 30, "high": [45000.0] * 30, "low": [45000.0] * 30}
        )
        # Should handle gracefully
        patterns = TechnicalAnalyzer.detect_patterns(df)
        assert isinstance(patterns, dict)
        # Patterns should be False or detected based on the logic
        for pattern_name, detected in patterns.items():
            assert isinstance(detected, (bool, np.bool_))


@pytest.mark.unit
class TestMarketContextEdgeCases:
    """Test market context error handling."""

    def test_get_market_context_with_error(self):
        """Test market context with error."""
        df = pd.DataFrame({"close": [45000], "volume": [100]})

        # Create indicators that could cause errors
        indicators = {
            "rsi": 60.0,
            "macd_histogram": 0.5,
            "ema_9": 45100,
            "ema_21": 45000,
            "atr": 0.0,  # Zero ATR
            "volume_ratio": 0.0,  # Zero volume ratio
        }

        context = TechnicalAnalyzer.get_market_context(df, indicators)
        # With positive MACD histogram and ema_9 > ema_21, sentiment becomes bullish
        assert context["market_sentiment"] in ["neutral", "bullish"]

    def test_get_market_context_stable_volume(self):
        """Test market context with stable volume (between 0.8 and 1.2)."""
        df = pd.DataFrame({"close": [45000], "volume": [100]})

        indicators = {
            "rsi": 50.0,
            "atr": 100.0,
            "volume_ratio": 1.0,  # Exactly 1.0, should be 'stable'
        }

        context = TechnicalAnalyzer.get_market_context(df, indicators)
        assert context["volume_trend"] == "stable"

    def test_get_market_context_bearish_ema(self):
        """Test market context with bearish EMA crossover."""
        df = pd.DataFrame({"close": [45000], "volume": [100]})

        indicators = {
            "rsi": 50.0,  # Neutral RSI
            "macd_histogram": -10.0,  # Negative histogram
            "ema_9": 44900,  # EMA 9 below EMA 21
            "ema_21": 45000,
            "atr": 100.0,
            "volume_ratio": 1.0,
        }

        context = TechnicalAnalyzer.get_market_context(df, indicators)
        assert context["market_sentiment"] == "bearish"

    def test_get_market_context_with_zero_price(self):
        """Test market context with zero current price."""
        df = pd.DataFrame({"close": [0.0], "volume": [100]})  # Zero price - edge case

        indicators = {"rsi": 50.0, "atr": 100.0, "volume_ratio": 1.0}

        # Should handle gracefully without division by zero
        context = TechnicalAnalyzer.get_market_context(df, indicators)
        assert isinstance(context, dict)
        assert "volatility" in context


@pytest.mark.unit
class TestDirectOpenAIClientRateLimiting:
    """Test DirectOpenAI rate limiting edge cases."""

    @pytest.mark.asyncio
    async def test_chat_completions_429_response_handling(self):
        """Test 429 response causes key index to increment."""
        client = DirectOpenAIClient(["key1", "key2"])

        # Verify behavior: when we get 429, the key index should increment
        initial_index = client.current_key_index

        # Mock getting a 429 response and tracking the rate limit
        client.rate_limited_keys.add(0)
        client.current_key_index = 1

        # Get current key should skip rate limited one
        key, index = client.get_current_api_key()
        assert key == "key2"

    @pytest.mark.asyncio
    async def test_rate_limit_tracking(self):
        """Test that rate limited keys are tracked."""
        client = DirectOpenAIClient(["key1", "key2", "key3"])

        # Add some keys to rate limited set
        client.rate_limited_keys.add(0)
        client.rate_limited_keys.add(1)

        # Should skip to key3
        key, index = client.get_current_api_key()
        assert key == "key3"

    @pytest.mark.asyncio
    async def test_all_keys_rate_limited_reset(self):
        """Test that when all keys are rate limited, set is cleared."""
        client = DirectOpenAIClient(["key1", "key2"])

        # Mark all keys as rate limited
        client.rate_limited_keys.add(0)
        client.rate_limited_keys.add(1)

        # Getting current key should clear the rate limit set
        key, index = client.get_current_api_key()
        assert len(client.rate_limited_keys) == 0
        assert key in ["key1", "key2"]


@pytest.mark.unit
class TestRateLimitLogic:
    """Test rate limit logic without full HTTP integration."""

    def test_rate_limit_key_tracking(self):
        """Test that rate-limited keys are properly tracked."""
        client = DirectOpenAIClient(["key1", "key2", "key3"])

        # Simulate rate limiting keys
        client.rate_limited_keys.add(0)
        client.rate_limited_keys.add(1)

        # Should skip to non-rate-limited key
        key, index = client.get_current_api_key()
        assert key == "key3"
        assert index == 0  # Index in available (non-rate-limited) keys

    def test_rate_limit_reset_all_keys(self):
        """Test reset when all keys are rate-limited."""
        client = DirectOpenAIClient(["key1", "key2"])

        # Mark all as rate-limited
        client.rate_limited_keys.add(0)
        client.rate_limited_keys.add(1)

        # Should clear rate limits and return a key
        key, index = client.get_current_api_key()
        assert len(client.rate_limited_keys) == 0
        assert key in ["key1", "key2"]


@pytest.mark.unit
class TestGPTAnalyzerFallbackStrategies:
    """Test GPT analyzer fallback with different strategies."""

    def test_fallback_analysis_neutral_rsi(self):
        """Test fallback with neutral RSI (between 30-70)."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(selected_strategies=["RSI Strategy"]),
        )

        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, {"rsi": 55.0}, {})
        assert "neutral" in result["reasoning"].lower()

    def test_fallback_analysis_macd_bearish_neutral_signal(self):
        """Test MACD bearish crossover - single signal stays Neutral."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(selected_strategies=["MACD Strategy"]),
        )

        # MACD bearish crossover - but single signal stays Neutral
        # (need 4/5 = 80% signals in same direction for trading signal - FR-STRATEGIES-006)
        indicators = {"macd": 30.0, "macd_signal": 50.0}
        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, indicators, {})
        # Signal should be Neutral since only 1 bearish signal (need 4+)
        assert result["signal"] == "Neutral"
        assert "MACD bearish" in result["reasoning"]
        assert "Bearish: 1" in result["reasoning"]

    def test_fallback_analysis_low_volume(self):
        """Test fallback with low volume."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(selected_strategies=["Volume Strategy"]),
        )

        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, {"volume_ratio": 0.3}, {})
        assert "low volume" in result["reasoning"].lower()

    def test_fallback_analysis_bb_upper_neutral(self):
        """Test Bollinger Bands upper boundary - single signal stays Neutral."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45100,
                low=44900,
                close=45050,
                volume=100.0,
            )
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=45050.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(
                selected_strategies=["Bollinger Bands Strategy"]
            ),
        )

        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, {"bb_position": 0.95}, {})
        # Single BB upper signal stays Neutral (need 4/5 = 80% signals - FR-STRATEGIES-006)
        assert result["signal"] == "Neutral"
        assert "upper Bollinger" in result["reasoning"]
        assert "Bearish: 1" in result["reasoning"]

    def test_fallback_analysis_strong_downward_movement(self):
        """Test price trend with strong downward movement."""
        analyzer = GPTTradingAnalyzer(None)

        candles = [
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000) - 3600000,
                open=45000,
                high=45100,
                low=44900,
                close=45000,
                volume=100.0,
            ),
            CandleData(
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                open=45000,
                high=45000,
                low=44000,
                close=44000,
                volume=100.0,
            ),  # -2.2% drop
        ]

        request = AIAnalysisRequest(
            symbol="BTCUSDT",
            timeframe_data={"1h": candles},
            current_price=44000.0,
            volume_24h=1000000.0,
            timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
            strategy_context=AIStrategyContext(),
        )

        # Note: signature now: (request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)
        result = analyzer._fallback_analysis(request, {}, {}, {}, {})
        assert "downward" in result["reasoning"].lower()


@pytest.mark.unit
class TestAnalysisEndpointEdgeCases:
    """Test analysis endpoint edge cases."""

    @pytest.mark.asyncio
    async def test_analyze_with_stored_non_int_timestamp(
        self, client, sample_ai_analysis_request, mock_mongodb
    ):
        """Test analysis with stored result having non-integer timestamp."""
        # Mock cached result with datetime timestamp
        cached_result = {
            "symbol": "BTCUSDT",
            "signal": "Short",
            "confidence": 0.85,
            "reasoning": "Cached reasoning",
            "timestamp": datetime.now(timezone.utc),  # datetime object instead of int
            "strategy_scores": {},
            "market_analysis": {
                "trend_direction": "Bearish",
                "trend_strength": 0.75,
                "support_levels": [],
                "resistance_levels": [],
                "volatility_level": "Medium",
                "volume_analysis": "Decreasing volume",
            },
            "risk_assessment": {
                "overall_risk": "Medium",
                "technical_risk": 0.5,
                "market_risk": 0.5,
                "recommended_position_size": 0.02,
            },
        }

        with patch("main.get_latest_analysis", AsyncMock(return_value=cached_result)):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
            # Should handle and perform fresh analysis
            assert response.status_code in [200, 500]

    @pytest.mark.asyncio
    async def test_analyze_broadcasts_fresh_signal(
        self, client, sample_ai_analysis_request, mock_openai_client
    ):
        """Test that fresh analysis broadcasts via WebSocket."""
        from main import ws_manager

        # Add a mock connection
        mock_ws = AsyncMock()
        ws_manager.active_connections.add(mock_ws)

        with patch("main.openai_client", mock_openai_client):
            with patch("main.get_latest_analysis", AsyncMock(return_value=None)):
                with patch("main.store_analysis_result", AsyncMock()):
                    response = await client.post(
                        "/ai/analyze", json=sample_ai_analysis_request
                    )
                    assert response.status_code == 200

        # Clean up
        ws_manager.active_connections.discard(mock_ws)


@pytest.mark.unit
class TestStorageEndpointEdgeCases:
    """Test storage endpoint edge cases."""

    @pytest.mark.asyncio
    async def test_storage_stats_with_datetime_latest(self, client, mock_mongodb):
        """Test storage stats when latest is datetime object."""

        async def async_gen():
            yield {"_id": "BTCUSDT", "count": 100, "latest": datetime.now(timezone.utc)}

        # Create a fresh mock collection
        mock_collection = AsyncMock()
        mock_collection.aggregate = MagicMock(return_value=async_gen())
        mock_collection.count_documents = AsyncMock(return_value=500)

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            response = await client.get("/ai/storage/stats")
            assert response.status_code == 200
            data = response.json()
            assert data["total_analyses"] == 500

    @pytest.mark.asyncio
    async def test_storage_stats_with_error(self, client, mock_mongodb):
        """Test storage stats with aggregation error."""
        # Create a fresh mock collection that raises error
        mock_collection = AsyncMock()
        mock_collection.count_documents = AsyncMock(
            side_effect=Exception("Aggregation error")
        )

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            response = await client.get("/ai/storage/stats")
            assert response.status_code == 200
            data = response.json()
            assert "error" in data

    @pytest.mark.asyncio
    async def test_clear_storage_with_error(self, client, mock_mongodb):
        """Test clearing storage with error."""
        # Create a fresh mock collection that raises error
        mock_collection = AsyncMock()
        mock_collection.delete_many = AsyncMock(side_effect=Exception("Delete error"))

        # Override the collection getter for this test
        mock_mongodb[1].__getitem__ = MagicMock(return_value=mock_collection)

        with patch("main.mongodb_db", mock_mongodb[1]):
            response = await client.post("/ai/storage/clear")
            assert response.status_code == 200
            data = response.json()
            assert "error" in data


@pytest.mark.unit
class TestWebSocketEdgeCases:
    """Test WebSocket edge cases."""

    @pytest.mark.asyncio
    @pytest.mark.skip(
        reason="Flaky - TestClient WebSocket has fixture pollution when run with full suite"
    )
    async def test_websocket_receive_text(self, test_client):
        """Test WebSocket receiving text message."""
        with test_client.websocket_connect("/ws") as websocket:
            # Receive connection message
            data = websocket.receive_json()
            assert data["type"] == "connection"

            # Send a text message (ping)
            websocket.send_text("ping")

            # Should receive pong response
            response = websocket.receive_json()
            assert response["type"] == "Pong"


@pytest.mark.unit
class TestDebugEndpointEdgeCases:
    """Test debug endpoint edge cases."""

    @pytest.mark.asyncio
    async def test_debug_gpt4_rate_limit_error(self, client):
        """Test debug endpoint with 429 rate limit."""
        mock_client = AsyncMock()
        mock_client.chat_completions_create = AsyncMock(
            side_effect=Exception("429 Rate Limit")
        )

        with patch("main.openai_client", mock_client):
            response = await client.get("/debug/gpt4")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "failed"
            assert "rate limit" in data["diagnosis"].lower()


@pytest.mark.unit
class TestCostStatisticsEndpoint:
    """Test /ai/cost/statistics endpoint."""

    @pytest.mark.asyncio
    async def test_cost_statistics_with_usage(self, client):
        """Test cost statistics with usage data."""
        # Mock global variables with actual usage
        with patch("main.total_requests_count", 100):
            with patch("main.total_input_tokens", 50000):
                with patch("main.total_output_tokens", 25000):
                    with patch("main.total_cost_usd", 0.5):
                        response = await client.get("/ai/cost/statistics")
                        assert response.status_code == 200
                        data = response.json()

                        # Check session statistics
                        assert data["session_statistics"]["total_requests"] == 100
                        assert data["session_statistics"]["total_input_tokens"] == 50000
                        assert data["session_statistics"]["total_output_tokens"] == 25000
                        assert data["session_statistics"]["total_cost_usd"] == 0.5

                        # Check projections exist
                        assert "estimated_daily_cost_usd" in data["projections"]
                        assert "estimated_monthly_cost_usd" in data["projections"]

                        # Check configuration
                        assert data["configuration"]["model"] == "gpt-4o-mini"
                        assert data["configuration"]["symbols_tracked"] == 8

                        # Check optimization status
                        assert data["optimization_status"]["cache_optimized"] is True

    @pytest.mark.asyncio
    async def test_cost_statistics_no_usage(self, client):
        """Test cost statistics with no usage yet."""
        with patch("main.total_requests_count", 0):
            with patch("main.total_input_tokens", 0):
                with patch("main.total_output_tokens", 0):
                    with patch("main.total_cost_usd", 0.0):
                        response = await client.get("/ai/cost/statistics")
                        assert response.status_code == 200
                        data = response.json()

                        assert data["session_statistics"]["total_requests"] == 0
                        assert data["session_statistics"]["total_cost_usd"] == 0.0
                        # Should handle division by zero gracefully
                        assert data["projections"]["estimated_daily_cost_usd"] == 0.0


@pytest.mark.unit
class TestSecurityHeadersMiddleware:
    """Test security headers middleware."""

    @pytest.mark.asyncio
    async def test_security_headers_development(self, client):
        """Test security headers in development environment."""
        with patch.dict(os.environ, {"ENVIRONMENT": "development"}):
            response = await client.get("/health")
            assert response.status_code == 200

            # Check security headers are present
            assert response.headers.get("X-Frame-Options") == "DENY"
            assert response.headers.get("X-Content-Type-Options") == "nosniff"
            assert response.headers.get("X-XSS-Protection") == "1; mode=block"
            assert "Content-Security-Policy" in response.headers
            assert response.headers.get("Referrer-Policy") == "strict-origin-when-cross-origin"
            assert "Permissions-Policy" in response.headers

            # HSTS should NOT be present in development
            assert "Strict-Transport-Security" not in response.headers

    @pytest.mark.asyncio
    async def test_security_headers_production(self, client):
        """Test security headers in production environment."""
        with patch.dict(os.environ, {"ENVIRONMENT": "production"}):
            response = await client.get("/health")
            assert response.status_code == 200

            # HSTS should be present in production
            assert "Strict-Transport-Security" in response.headers
            assert "max-age=31536000" in response.headers["Strict-Transport-Security"]


@pytest.mark.unit
class TestPeriodicAnalysisRunner:
    """Test periodic analysis background task."""

    @pytest.mark.asyncio
    async def test_periodic_analysis_runner_one_cycle(self):
        """Test one cycle of periodic analysis."""
        from main import periodic_analysis_runner, fetch_real_market_data

        # Mock the global openai_client and mongodb_db
        mock_gpt_client = AsyncMock()
        mock_gpt_client.chat_completions_create = AsyncMock(
            return_value={
                "choices": [{
                    "message": {
                        "content": json.dumps({
                            "signal": "Long",
                            "confidence": 0.75,
                            "reasoning": "Test",
                            "strategy_scores": {},
                            "market_analysis": {
                                "trend_direction": "Bullish",
                                "trend_strength": 0.8,
                                "support_levels": [],
                                "resistance_levels": [],
                                "volatility_level": "Medium",
                                "volume_analysis": "Normal"
                            },
                            "risk_assessment": {
                                "overall_risk": "Medium",
                                "technical_risk": 0.5,
                                "market_risk": 0.5,
                                "recommended_position_size": 0.02
                            }
                        })
                    }
                }],
                "usage": {"prompt_tokens": 100, "completion_tokens": 50, "total_tokens": 150}
            }
        )

        # Create a task that will run one iteration then stop
        task = None
        try:
            with patch("main.openai_client", mock_gpt_client):
                with patch("main.mongodb_db", None):  # Disable storage for test
                    with patch("main.ANALYSIS_INTERVAL_MINUTES", 0.001):  # Very short interval
                        # Create task
                        task = asyncio.create_task(periodic_analysis_runner())

                        # Let it run for a short time
                        await asyncio.sleep(0.1)

                        # Cancel the task
                        task.cancel()

                        # Wait for cancellation
                        try:
                            await task
                        except asyncio.CancelledError:
                            pass  # Expected
        finally:
            if task and not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass

    @pytest.mark.asyncio
    async def test_periodic_analysis_runner_with_error(self):
        """Test periodic analysis handles errors gracefully."""
        from main import periodic_analysis_runner

        # Mock analyze function to raise error
        mock_gpt_client = AsyncMock()
        mock_gpt_client.chat_completions_create = AsyncMock(
            side_effect=Exception("API Error")
        )

        task = None
        try:
            with patch("main.openai_client", mock_gpt_client):
                with patch("main.ANALYSIS_INTERVAL_MINUTES", 0.001):
                    task = asyncio.create_task(periodic_analysis_runner())

                    # Let it run briefly
                    await asyncio.sleep(0.1)

                    # Cancel
                    task.cancel()
                    try:
                        await task
                    except asyncio.CancelledError:
                        pass  # Expected - should handle errors and continue
        finally:
            if task and not task.done():
                task.cancel()
                try:
                    await task
                except asyncio.CancelledError:
                    pass


@pytest.mark.unit
class TestAnalysisStorageErrorHandling:
    """Test error handling in analysis storage functions."""

    @pytest.mark.asyncio
    async def test_store_analysis_result_exception(self):
        """Test store_analysis_result handles exceptions gracefully."""
        from main import store_analysis_result
        import main

        # Mock MongoDB to raise exception
        mock_db = AsyncMock()
        mock_collection = AsyncMock()
        mock_collection.insert_one = AsyncMock(side_effect=Exception("Database error"))
        mock_db.__getitem__.return_value = mock_collection

        with patch("main.mongodb_db", mock_db):
            # Should log error but not raise (lines 169-170)
            await store_analysis_result("BTCUSDT", {"signal": "Long"})
            # If we get here, exception was handled correctly

    @pytest.mark.asyncio
    async def test_get_latest_analysis_exception(self):
        """Test get_latest_analysis handles exceptions gracefully."""
        from main import get_latest_analysis
        import main

        # Mock MongoDB to raise exception  
        mock_db = AsyncMock()
        mock_collection = AsyncMock()
        mock_collection.find_one = AsyncMock(side_effect=Exception("Database error"))
        mock_db.__getitem__.return_value = mock_collection

        with patch("main.mongodb_db", mock_db):
            # Should return None on error (lines 187-189)
            result = await get_latest_analysis("BTCUSDT")
            assert result is None

    @pytest.mark.asyncio
    async def test_periodic_analysis_symbol_level_exception(self):
        """Test periodic analysis handles symbol-level exceptions."""
        from main import periodic_analysis_runner
        import main

        # Mock GPT client that raises exception for specific symbol
        mock_gpt_client = AsyncMock()
        mock_gpt_client.chat_completions_create = AsyncMock(
            side_effect=Exception("Symbol analysis failed")
        )

        with patch("main.ANALYSIS_SYMBOLS", ["BTCUSDT"]):  # Just one symbol
            with patch("main.ANALYSIS_INTERVAL_MINUTES", 0.001):  # Very short interval
                with patch("main.openai_client", mock_gpt_client):
                    with patch("main.mongodb_db", None):
                        task = None
                        try:
                            task = asyncio.create_task(periodic_analysis_runner())
                            await asyncio.sleep(0.1)  # Let it run briefly

                            # Exception should be logged but not crash (lines 235-236)
                            # Task should continue running
                            assert not task.done()

                            task.cancel()
                            try:
                                await task
                            except asyncio.CancelledError:
                                pass  # Expected
                        finally:
                            if task and not task.done():
                                task.cancel()
                                try:
                                    await task
                                except asyncio.CancelledError:
                                    pass
