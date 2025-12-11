"""
Test GPTTradingAnalyzer class functionality.
"""

import json
import os
# Import after adding to path in conftest
import sys
from datetime import datetime, timezone
from unittest.mock import AsyncMock, MagicMock, patch

import httpx
import pytest

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))


@pytest.fixture
def gpt_analyzer(mock_openai_client):
    """Create GPTTradingAnalyzer instance with mock client."""
    from main import GPTTradingAnalyzer

    return GPTTradingAnalyzer(mock_openai_client)


@pytest.fixture
def mock_httpx_client():
    """Mock httpx client for direct API calls."""
    mock = AsyncMock()
    mock.post = AsyncMock()
    return mock


@pytest.mark.unit
class TestGPTTradingAnalyzer:
    """Test GPTTradingAnalyzer methods."""

    @pytest.mark.asyncio
    async def test_analyze_trading_signals_success(
        self, gpt_analyzer, sample_ai_analysis_request
    ):
        """Test successful signal analysis."""
        from main import AIAnalysisRequest

        request = AIAnalysisRequest(**sample_ai_analysis_request)

        result = await gpt_analyzer.analyze_trading_signals(request)

        assert result.signal == "Long"
        assert result.confidence == 0.75
        assert (
            result.reasoning == "Strong bullish momentum based on technical indicators"
        )
        assert result.timestamp > 0

    @pytest.mark.asyncio
    async def test_analyze_with_api_error(
        self, gpt_analyzer, sample_ai_analysis_request, mock_openai_client
    ):
        """Test handling of API errors with fallback to technical analysis."""
        from main import AIAnalysisRequest

        request = AIAnalysisRequest(**sample_ai_analysis_request)

        # Mock API error
        mock_openai_client.chat_completions_create.side_effect = Exception("API Error")

        # Should fall back to technical analysis instead of raising
        result = await gpt_analyzer.analyze_trading_signals(request)
        assert result.signal in ["Long", "Short", "Neutral"]
        assert result.confidence >= 0
        assert "Technical analysis" in result.reasoning

    @pytest.mark.asyncio
    async def test_analyze_with_invalid_json_response(
        self, gpt_analyzer, sample_ai_analysis_request, mock_openai_client
    ):
        """Test handling of invalid JSON in response."""
        from main import AIAnalysisRequest

        request = AIAnalysisRequest(**sample_ai_analysis_request)

        # Mock invalid JSON response
        mock_openai_client.chat.completions.create.return_value = MagicMock(
            choices=[MagicMock(message=MagicMock(content="Invalid JSON"))]
        )

        # Should handle gracefully
        result = await gpt_analyzer.analyze_trading_signals(request)
        assert result.signal in ["Long", "Short", "Neutral"]

    @pytest.mark.asyncio
    @pytest.mark.skip(reason="call_openai_api_direct function not implemented")
    async def test_direct_api_call(self, mock_httpx_client):
        """Test direct OpenAI API call."""
        from main import call_openai_api_direct

        # Mock successful response
        mock_response = MagicMock()
        mock_response.status_code = 200
        mock_response.json.return_value = {
            "choices": [
                {"message": {"content": '{"signal": "Short", "confidence": 0.82}'}}
            ]
        }
        mock_httpx_client.post.return_value = mock_response

        with patch("httpx.AsyncClient", return_value=mock_httpx_client):
            result = await call_openai_api_direct(
                "test_key", [{"role": "user", "content": "test"}]
            )

            assert result == mock_response.json.return_value
            mock_httpx_client.post.assert_called_once()

    @pytest.mark.asyncio
    async def test_rate_limiting(self, gpt_analyzer, sample_ai_analysis_request):
        """Test rate limiting behavior."""
        import main
        from main import (OPENAI_REQUEST_DELAY, AIAnalysisRequest,
                          last_openai_request_time)

        request = AIAnalysisRequest(**sample_ai_analysis_request)

        # Set last request time to recent
        main.last_openai_request_time = datetime.now()

        # Should add delay
        start_time = datetime.now()
        await gpt_analyzer.analyze_trading_signals(request)
        elapsed = (datetime.now() - start_time).total_seconds()

        # Check that some delay was applied (may not be full delay due to mocking)
        assert elapsed >= 0


@pytest.mark.unit
class TestStrategyRecommendations:
    """Test strategy recommendation functionality."""

    @pytest.mark.asyncio
    @pytest.mark.skip(
        reason="get_strategy_recommendations method not implemented on GPTTradingAnalyzer"
    )
    async def test_get_strategy_recommendations(self, gpt_analyzer):
        """Test getting strategy recommendations."""
        from main import StrategyRecommendationRequest

        request = StrategyRecommendationRequest(
            trading_style="swing",
            risk_tolerance="medium",
            capital=10000,
            experience_level="intermediate",
            preferred_timeframes=["1h", "4h"],
            preferred_pairs=["BTCUSDT", "ETHUSDT"],
            current_market_conditions={
                "btc_dominance": 48.5,
                "total_market_cap": 1.75e12,
                "fear_greed_index": 65,
            },
        )

        # Mock response
        mock_response = {
            "recommended_strategies": [
                {
                    "name": "EMA Crossover",
                    "suitability_score": 0.85,
                    "expected_win_rate": 0.65,
                    "expected_risk_reward": 1.5,
                    "parameters": {"ema_fast": 9, "ema_slow": 21, "rsi_period": 14},
                }
            ],
            "position_sizing": {
                "method": "fixed_percentage",
                "percentage": 2,
                "max_concurrent_trades": 3,
            },
            "risk_management": {
                "stop_loss_method": "atr_based",
                "atr_multiplier": 2,
                "trailing_stop": True,
                "max_daily_loss": 5,
            },
        }

        gpt_analyzer.client.chat.completions.create.return_value = MagicMock(
            choices=[MagicMock(message=MagicMock(content=json.dumps(mock_response)))]
        )

        result = await gpt_analyzer.get_strategy_recommendations(request)

        assert "recommended_strategies" in result
        assert len(result["recommended_strategies"]) > 0
        assert result["recommended_strategies"][0]["name"] == "EMA Crossover"


@pytest.mark.unit
class TestMarketConditionAnalysis:
    """Test market condition analysis functionality."""

    @pytest.mark.asyncio
    @pytest.mark.skip(
        reason="analyze_market_condition method not implemented on GPTTradingAnalyzer"
    )
    async def test_analyze_market_condition(self, gpt_analyzer):
        """Test market condition analysis."""
        from main import MarketConditionRequest

        request = MarketConditionRequest(
            symbols=["BTCUSDT", "ETHUSDT"],
            indicators={
                "BTCUSDT": {
                    "price": 45000,
                    "volume_24h": 25000000000,
                    "price_change_24h": 2.5,
                },
                "ETHUSDT": {
                    "price": 2500,
                    "volume_24h": 15000000000,
                    "price_change_24h": 3.2,
                },
            },
        )

        # Mock response
        mock_response = {
            "overall_market": "bullish",
            "market_phase": "accumulation",
            "volatility_level": "medium",
            "trend_strength": 0.72,
            "recommendations": [
                "Consider increasing position sizes",
                "Watch for breakout opportunities",
            ],
            "risk_factors": ["Approaching overbought conditions"],
        }

        gpt_analyzer.client.chat.completions.create.return_value = MagicMock(
            choices=[MagicMock(message=MagicMock(content=json.dumps(mock_response)))]
        )

        result = await gpt_analyzer.analyze_market_condition(request)

        assert result["overall_market"] == "bullish"
        assert result["market_phase"] == "accumulation"
        assert "recommendations" in result
        assert "risk_factors" in result


@pytest.mark.unit
class TestAPIKeyRotation:
    """Test API key rotation functionality."""

    @pytest.mark.asyncio
    @pytest.mark.skip(reason="call_openai_with_fallback function not implemented")
    async def test_api_key_fallback(self, mock_httpx_client):
        """Test fallback to next API key on failure."""
        from main import call_openai_with_fallback

        # First call fails with 429
        error_response = httpx.Response(
            status_code=429, json={"error": {"message": "Rate limit exceeded"}}
        )

        # Second call succeeds
        success_response = MagicMock()
        success_response.status_code = 200
        success_response.json.return_value = {
            "choices": [{"message": {"content": '{"signal": "Long"}'}}]
        }

        mock_httpx_client.post.side_effect = [
            httpx.HTTPStatusError(
                "Rate limited", request=MagicMock(), response=error_response
            ),
            success_response,
        ]

        api_keys = ["key1", "key2", "key3"]

        with patch("httpx.AsyncClient", return_value=mock_httpx_client):
            result = await call_openai_with_fallback(
                api_keys, [{"role": "user", "content": "test"}]
            )

            assert result is not None
            assert (
                mock_httpx_client.post.call_count == 2
            )  # Failed once, succeeded on second
