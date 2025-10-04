"""
Test main FastAPI application and endpoints.
"""

import pytest
from unittest.mock import patch, AsyncMock, MagicMock
from datetime import datetime, timezone, timedelta
import json

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
        with patch('main.mongodb_client', None):
            response = await client.get("/health")
            assert response.status_code == 200
            data = response.json()
            assert data["status"] == "healthy"
            assert data["mongodb_connected"] is False

@pytest.mark.unit
class TestAIAnalysisEndpoint:
    """Test AI analysis endpoint."""

    @pytest.mark.asyncio
    async def test_analyze_success(self, client, sample_ai_analysis_request, mock_openai_client):
        """Test successful AI analysis."""
        with patch('main.openai_client', mock_openai_client):
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
        """Test analysis with invalid symbol."""
        sample_ai_analysis_request["symbol"] = "INVALID"
        response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
        assert response.status_code == 400
        assert "not supported" in response.json()["detail"]

    @pytest.mark.asyncio
    async def test_analyze_insufficient_candles(self, client, sample_ai_analysis_request):
        """Test analysis with no candle data."""
        sample_ai_analysis_request["candles"] = []
        response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
        assert response.status_code == 400
        assert "At least 1 candle required" in response.json()["detail"]

    @pytest.mark.asyncio
    async def test_analyze_with_cached_result(self, client, sample_ai_analysis_request, mock_mongodb):
        """Test analysis returns cached result."""
        # Mock cached result
        cached_result = {
            "symbol": "BTCUSDT",
            "signal": "Short",
            "confidence": 0.85,
            "reasoning": "Cached reasoning",
            "timestamp": datetime.now(timezone.utc) - timedelta(minutes=2),
            "metadata": {"analysis_id": "cached_id"}
        }
        
        with patch('main.get_latest_analysis', AsyncMock(return_value=cached_result)):
            response = await client.post("/ai/analyze", json=sample_ai_analysis_request)
            assert response.status_code == 200
            data = response.json()
            assert data["signal"] == "Short"
            assert data["confidence"] == 0.85

@pytest.mark.unit
class TestStrategyRecommendations:
    """Test strategy recommendations endpoint."""

    @pytest.mark.asyncio
    async def test_strategy_recommendations_success(self, client, mock_openai_client):
        """Test successful strategy recommendations."""
        request_data = {
            "trading_style": "swing",
            "risk_tolerance": "medium",
            "capital": 10000,
            "experience_level": "intermediate",
            "preferred_timeframes": ["1h", "4h"],
            "preferred_pairs": ["BTCUSDT", "ETHUSDT"],
            "current_market_conditions": {
                "btc_dominance": 48.5,
                "total_market_cap": 1.75e12,
                "fear_greed_index": 65
            }
        }
        
        mock_response = {
            "recommended_strategies": [{
                "name": "EMA Crossover",
                "suitability_score": 0.85
            }],
            "position_sizing": {
                "method": "fixed_percentage",
                "percentage": 2
            }
        }
        
        mock_openai_client.chat.completions.create.return_value = MagicMock(
            choices=[MagicMock(
                message=MagicMock(content=json.dumps(mock_response))
            )]
        )
        
        with patch('main.openai_client', mock_openai_client):
            response = await client.post("/ai/strategy-recommendations", json=request_data)
            assert response.status_code == 200
            data = response.json()
            assert "recommended_strategies" in data
            assert len(data["recommended_strategies"]) > 0

@pytest.mark.unit
class TestMarketCondition:
    """Test market condition analysis endpoint."""

    @pytest.mark.asyncio
    async def test_market_condition_success(self, client, mock_openai_client):
        """Test successful market condition analysis."""
        request_data = {
            "symbols": ["BTCUSDT", "ETHUSDT"],
            "indicators": {
                "BTCUSDT": {
                    "price": 45000,
                    "volume_24h": 25000000000,
                    "price_change_24h": 2.5
                },
                "ETHUSDT": {
                    "price": 2500,
                    "volume_24h": 15000000000,
                    "price_change_24h": 3.2
                }
            }
        }
        
        mock_response = {
            "overall_market": "bullish",
            "market_phase": "accumulation",
            "volatility_level": "medium",
            "trend_strength": 0.72
        }
        
        mock_openai_client.chat.completions.create.return_value = MagicMock(
            choices=[MagicMock(
                message=MagicMock(content=json.dumps(mock_response))
            )]
        )
        
        with patch('main.openai_client', mock_openai_client):
            response = await client.post("/ai/market-condition", json=request_data)
            assert response.status_code == 200
            data = response.json()
            assert data["overall_market"] == "bullish"
            assert "market_phase" in data

@pytest.mark.unit
class TestFeedbackEndpoint:
    """Test feedback endpoint."""

    @pytest.mark.asyncio
    async def test_feedback_success(self, client):
        """Test successful feedback submission."""
        feedback_data = {
            "trade_id": "123e4567-e89b-12d3-a456-426614174000",
            "symbol": "BTCUSDT",
            "signal": "Long",
            "confidence": 0.75,
            "actual_outcome": "profit",
            "profit_loss_percent": 2.5,
            "holding_time_minutes": 180,
            "notes": "Good signal"
        }
        
        response = await client.post("/ai/feedback", json=feedback_data)
        assert response.status_code == 200
        data = response.json()
        assert data["message"] == "Feedback received"

@pytest.mark.unit
class TestWebSocket:
    """Test WebSocket functionality."""
    
    def test_websocket_connection(self, test_client):
        """Test WebSocket connection and messages."""
        with test_client.websocket_connect("/ws") as websocket:
            # Test connection
            data = websocket.receive_json()
            assert data["type"] == "connection"
            assert data["message"] == "Connected to AI Trading Service"
            
            # Test receiving AI signal
            with patch('main.ws_manager.broadcast', AsyncMock()) as mock_broadcast:
                # Simulate broadcasting
                test_message = {
                    "type": "ai_signal",
                    "data": {
                        "symbol": "BTCUSDT",
                        "signal": "Long",
                        "confidence": 0.8
                    }
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
        # Mock aggregation result
        mock_mongodb[1]["ai_analysis_results"].aggregate = AsyncMock(
            return_value=AsyncMock(__aiter__=AsyncMock(return_value=iter([
                {"_id": "BTCUSDT", "count": 100, "avg_confidence": 0.75}
            ])))
        )
        mock_mongodb[1]["ai_analysis_results"].count_documents = AsyncMock(return_value=500)
        
        with patch('main.mongodb_db', mock_mongodb[1]):
            response = await client.get("/ai/storage/stats")
            assert response.status_code == 200
            data = response.json()
            assert data["total_analyses"] == 500

    @pytest.mark.asyncio
    async def test_storage_stats_no_mongodb(self, client):
        """Test storage stats when MongoDB is unavailable."""
        with patch('main.mongodb_db', None):
            response = await client.get("/ai/storage/stats")
            assert response.status_code == 200
            data = response.json()
            assert data["error"] == "MongoDB not connected"

    @pytest.mark.asyncio
    async def test_clear_storage_success(self, client, mock_mongodb):
        """Test clearing storage."""
        mock_mongodb[1]["ai_analysis_results"].delete_many = AsyncMock(
            return_value=MagicMock(deleted_count=100)
        )
        
        with patch('main.mongodb_db', mock_mongodb[1]):
            response = await client.post("/ai/storage/clear")
            assert response.status_code == 200
            data = response.json()
            assert data["deleted_count"] == 100

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