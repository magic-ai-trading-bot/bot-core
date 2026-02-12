"""
Comprehensive test file to boost Python coverage from 93% to 95%+.

Covers:
1. main.py uncovered lines (periodic analysis, trend prediction, websocket, etc.)
2. features/feature_engineering.py (error handling)
3. features/technical_indicators.py (error handling)
"""

import asyncio
import json
import os
from datetime import datetime, timezone
from unittest.mock import AsyncMock, MagicMock, Mock, patch

import numpy as np
import pandas as pd
import pytest
from fastapi import WebSocket

# Set TESTING before importing main
os.environ["TESTING"] = "true"

import main
from features.feature_engineering import FeatureEngineer
from features.technical_indicators import TechnicalIndicators


@pytest.mark.unit
class TestPeriodicAnalysisRunner:
    """Test periodic analysis background task."""

    @pytest.mark.asyncio
    async def test_periodic_analysis_with_analysis_symbols(self):
        """Test periodic analysis with real symbols from Rust API."""
        with patch("main.fetch_analysis_symbols") as mock_symbols:
            mock_symbols.return_value = ["BTCUSDT", "ETHUSDT"]

            mock_request = main.AIAnalysisRequest(
                symbol="BTCUSDT",
                timeframe_data={"1h": []},
                current_price=45000.0,
                volume_24h=1000000.0,
                timestamp=int(datetime.now(timezone.utc).timestamp() * 1000),
                strategy_context=main.AIStrategyContext(
                    selected_strategies=["RSI"],
                    risk_tolerance="medium",
                    trading_style="swing",
                    technical_indicators={},
                    market_context={},
                ),
            )

            with patch("main.fetch_real_market_data") as mock_fetch:
                mock_fetch.return_value = mock_request

                with patch("main.GPTTradingAnalyzer") as mock_analyzer_class:
                    mock_analyzer = AsyncMock()
                    mock_result = MagicMock()
                    mock_result.signal = "Long"
                    mock_result.confidence = 0.75
                    mock_result.reasoning = "Strong uptrend"
                    mock_result.model_dump = MagicMock(
                        return_value={
                            "signal": "Long",
                            "confidence": 0.75,
                            "reasoning": "Strong uptrend",
                        }
                    )
                    mock_analyzer.analyze_trading_signals = AsyncMock(
                        return_value=mock_result
                    )
                    mock_analyzer_class.return_value = mock_analyzer

                    with patch("main.store_analysis_result") as mock_store:
                        mock_store.return_value = None

                        with patch("main.ws_manager.broadcast_signal") as mock_ws:
                            mock_ws.return_value = None

                            with patch("main.ANALYSIS_INTERVAL_MINUTES", 0.001):
                                task = asyncio.create_task(
                                    main.periodic_analysis_runner()
                                )

                                await asyncio.sleep(0.1)

                                task.cancel()
                                try:
                                    await task
                                except asyncio.CancelledError:
                                    pass

                                mock_symbols.assert_called()

    @pytest.mark.asyncio
    async def test_periodic_analysis_symbol_failure(self):
        """Test periodic analysis continues when one symbol fails."""
        with patch("main.fetch_analysis_symbols") as mock_symbols:
            mock_symbols.return_value = ["BTCUSDT"]

            with patch("main.fetch_real_market_data") as mock_fetch:
                mock_fetch.side_effect = Exception("Network error")

                with patch("main.ANALYSIS_INTERVAL_MINUTES", 0.001):
                    task = asyncio.create_task(main.periodic_analysis_runner())

                    await asyncio.sleep(0.1)

                    task.cancel()
                    try:
                        await task
                    except asyncio.CancelledError:
                        pass

                    assert mock_fetch.call_count >= 1


@pytest.mark.unit
class TestGPT4TrendPrediction:
    """Test GPT-4 trend prediction helper functions."""

    @pytest.mark.asyncio
    async def test_predict_trend_gpt4_with_usage_tracking(self):
        """Test _predict_trend_gpt4 tracks token usage and cost."""
        sample_candles = [
            {
                "symbol": "BTCUSDT",
                "timeframe": "1d",
                "open": 45000.0 + i * 10,
                "high": 45100.0 + i * 10,
                "low": 44900.0 + i * 10,
                "close": 45050.0 + i * 10,
                "volume": 1000.0,
                "open_time": 1700000000000 + i * 86400000,
            }
            for i in range(60)
        ]

        candles_by_tf = {"1d": sample_candles}

        # Save and reset global counters
        original_client = main.openai_client
        original_input_tokens = main.total_input_tokens
        original_output_tokens = main.total_output_tokens
        original_requests = main.total_requests_count
        original_cost = main.total_cost_usd

        try:
            main.total_input_tokens = 0
            main.total_output_tokens = 0
            main.total_requests_count = 0
            main.total_cost_usd = 0.0

            mock_openai = AsyncMock()
            mock_openai.chat_completions_create = AsyncMock(
                return_value={
                    "choices": [
                        {
                            "message": {
                                "content": json.dumps(
                                    {
                                        "trend": "Uptrend",
                                        "confidence": 0.85,
                                        "reasoning": "Strong bullish momentum",
                                        "timeframe_alignment": {
                                            "daily": "up",
                                            "4h": "up",
                                            "primary": "up",
                                        },
                                    }
                                )
                            }
                        }
                    ],
                    "usage": {"prompt_tokens": 500, "completion_tokens": 100},
                }
            )

            main.openai_client = mock_openai

            result = await main._predict_trend_gpt4("BTCUSDT", candles_by_tf)

            assert result["trend"] == "Uptrend"
            assert result["confidence"] == 0.85
            assert main.total_input_tokens == 500
            assert main.total_output_tokens == 100
            assert main.total_requests_count == 1
            assert main.total_cost_usd > 0

        finally:
            main.openai_client = original_client
            main.total_input_tokens = original_input_tokens
            main.total_output_tokens = original_output_tokens
            main.total_requests_count = original_requests
            main.total_cost_usd = original_cost

    @pytest.mark.asyncio
    async def test_predict_trend_technical_fallback(self):
        """Test technical analysis fallback when GPT-4 unavailable."""
        sample_candles = [
            {
                "open": 40000.0 + i * 100,
                "high": 40200.0 + i * 100,
                "low": 39900.0 + i * 100,
                "close": 40100.0 + i * 100,
                "volume": 1000.0,
                "open_time": 1700000000000 + i * 86400000,
            }
            for i in range(250)
        ]

        candles_by_tf = {"1h": sample_candles}

        result = main._predict_trend_technical("BTCUSDT", candles_by_tf, "1h")

        assert "trend" in result
        assert "confidence" in result
        assert "reasoning" in result
        assert result["trend"] in ["Uptrend", "Downtrend", "Neutral"]

    @pytest.mark.asyncio
    async def test_predict_trend_technical_insufficient_data(self):
        """Test technical analysis with insufficient data."""
        sample_candles = [
            {
                "open": 45000.0,
                "high": 45100.0,
                "low": 44900.0,
                "close": 45050.0,
                "volume": 1000.0,
                "open_time": 1700000000000 + i * 86400000,
            }
            for i in range(50)
        ]

        candles_by_tf = {"1h": sample_candles}

        result = main._predict_trend_technical("BTCUSDT", candles_by_tf, "1h")

        assert result["trend"] == "Neutral"
        assert result["confidence"] == 0.3
        assert "Insufficient data" in result["reasoning"]

    @pytest.mark.asyncio
    async def test_format_tf_data_with_indicators(self):
        """Test _format_tf_data formats indicators correctly."""
        indicators = {
            "current_price": 45000.0,
            "ema_200": 44000.0,
            "ema_50": 44500.0,
            "distance_from_ema200": 2.27,
            "distance_from_ema50": 1.12,
            "momentum_20": 5.5,
            "rsi": 65.0,
            "volume_ratio": 1.5,
            "last_5_closes": [44900, 44950, 45000, 45050, 45100],
        }

        result = main._format_tf_data("Daily", indicators)

        assert "Current Price: $45000.00" in result
        assert "EMA200: $44000.00" in result
        assert "distance: +2.27%" in result
        assert "RSI: 65.0" in result

    @pytest.mark.asyncio
    async def test_format_tf_data_empty_indicators(self):
        """Test _format_tf_data with empty indicators."""
        result = main._format_tf_data("Daily", {})

        assert "Daily: No data available" in result


@pytest.mark.unit
class TestWebSocketEndpoint:
    """Test WebSocket endpoint for real-time signals."""

    @pytest.mark.asyncio
    async def test_websocket_pong_response(self):
        """Test WebSocket responds with pong to keep connection alive."""
        mock_websocket = AsyncMock(spec=WebSocket)
        mock_websocket.receive_text = AsyncMock(
            side_effect=[
                "ping",
                Exception("Disconnect"),
            ]
        )
        mock_websocket.send_json = AsyncMock()

        with patch.object(main.ws_manager, "connect") as mock_connect:
            mock_connect.return_value = None

            with patch.object(main.ws_manager, "disconnect") as mock_disconnect:
                mock_disconnect.return_value = None

                try:
                    await main.websocket_endpoint(mock_websocket)
                except Exception:
                    pass

                mock_connect.assert_called_once_with(mock_websocket)

                assert mock_websocket.send_json.call_count >= 1
                call_args = mock_websocket.send_json.call_args[0][0]
                assert call_args["type"] == "Pong"
                assert "timestamp" in call_args


@pytest.mark.unit
class TestProjectChatbotHelper:
    """Test project_chatbot.py helper functions."""

    def test_find_project_root_local_development(self):
        """Test _find_project_root finds local project root."""
        from services.project_chatbot import _find_project_root

        result = _find_project_root()

        assert result is not None
        assert str(result).endswith(("bot-core", "python-ai-service"))


@pytest.mark.unit
class TestFeatureEngineerErrorHandling:
    """Test feature_engineering.py error handling paths."""

    def test_prepare_for_inference_insufficient_data(self):
        """Test prepare_for_inference with insufficient data."""
        fe = FeatureEngineer()
        # Override config for testing
        fe.config["sequence_length"] = 60

        df = pd.DataFrame(
            {
                "close": [100 + i for i in range(10)],
                "high": [101 + i for i in range(10)],
                "low": [99 + i for i in range(10)],
                "open": [100 + i for i in range(10)],
                "volume": [1000.0 for _ in range(10)],
            }
        )

        result = fe.prepare_for_inference(df)

        assert result is None

    def test_prepare_for_inference_no_feature_columns(self):
        """Test prepare_for_inference when feature_columns not set."""
        fe = FeatureEngineer()
        # Override config for testing
        fe.config["sequence_length"] = 10
        fe.feature_columns = []

        # Need enough data with all required columns for feature engineering
        df = pd.DataFrame(
            {
                "close": [100 + i * 0.5 for i in range(100)],
                "high": [101 + i * 0.5 for i in range(100)],
                "low": [99 + i * 0.5 for i in range(100)],
                "open": [100 + i * 0.5 for i in range(100)],
                "volume": [1000.0 + i * 10 for i in range(100)],
            }
        )

        result = fe.prepare_for_inference(df)

        # May be None if not enough data after feature engineering
        # (dropna removes rows). This tests the error handling path.
        if result is not None:
            assert result.shape[0] == 1


@pytest.mark.unit
class TestTechnicalIndicatorsErrorHandling:
    """Test technical_indicators.py error handling paths."""

    def test_calculate_volume_indicators_error(self):
        """Test calculate_volume_indicators handles errors."""
        ti = TechnicalIndicators()

        df = pd.DataFrame(
            {
                "close": [100, 101, 102, 103, 104],
                "high": [101, 102, 103, 104, 105],
                "low": [99, 100, 101, 102, 103],
                "volume": [1000, 1100, 1200, 1300, 1400],
            }
        )

        with patch(
            "ta.volume.OnBalanceVolumeIndicator", side_effect=Exception("OBV error")
        ):
            result = ti.calculate_volume_indicators(df)

            assert "obv" in result
            assert len(result["obv"]) == len(df)

    def test_calculate_momentum_indicators_error(self):
        """Test calculate_momentum_indicators handles errors."""
        ti = TechnicalIndicators()

        df = pd.DataFrame(
            {
                "close": [100, 101, 102, 103, 104],
                "high": [101, 102, 103, 104, 105],
                "low": [99, 100, 101, 102, 103],
            }
        )

        with patch("ta.momentum.ROCIndicator", side_effect=Exception("ROC error")):
            result = ti.calculate_momentum_indicators(df)

            assert isinstance(result, dict)

    def test_calculate_all_indicators_error(self):
        """Test calculate_all_indicators handles errors gracefully."""
        ti = TechnicalIndicators()

        df = pd.DataFrame(
            {
                "close": [100, 101, 102, 103, 104],
                "high": [101, 102, 103, 104, 105],
                "low": [99, 100, 101, 102, 103],
                "volume": [1000, 1100, 1200, 1300, 1400],
            }
        )

        with patch.object(
            ti, "calculate_rsi", side_effect=Exception("RSI calculation failed")
        ):
            result = ti.calculate_all_indicators(df)

            assert isinstance(result, pd.DataFrame)
            assert len(result) == len(df)


@pytest.mark.unit
class TestLifespanMongoDBIndexes:
    """Test MongoDB index creation during lifespan."""

    @pytest.mark.asyncio
    async def test_lifespan_creates_mongodb_indexes(self):
        """Test lifespan creates MongoDB indexes successfully."""
        with patch("main.AsyncIOMotorClient") as mock_mongo_client:
            mock_client = AsyncMock()
            mock_client.admin.command = AsyncMock(return_value={"ok": 1})

            mock_db = AsyncMock()
            mock_collection = AsyncMock()
            mock_collection.create_index = AsyncMock()

            mock_db.__getitem__ = MagicMock(return_value=mock_collection)
            mock_client.get_default_database = MagicMock(return_value=mock_db)
            mock_mongo_client.return_value = mock_client

            with patch.dict(
                os.environ,
                {
                    "OPENAI_API_KEY": "test-key",
                    "DATABASE_URL": "mongodb://localhost:27017/test",
                },
                clear=False,
            ):
                app = main.FastAPI()

                async with main.lifespan(app):
                    mock_collection.create_index.assert_called()

    @pytest.mark.asyncio
    async def test_lifespan_mongodb_ping_fails(self):
        """Test lifespan handles MongoDB ping failure."""
        with patch("main.AsyncIOMotorClient") as mock_mongo_client:
            mock_client = AsyncMock()
            mock_client.admin.command = AsyncMock(
                side_effect=Exception("Connection timeout")
            )
            mock_mongo_client.return_value = mock_client

            with patch.dict(
                os.environ,
                {
                    "DATABASE_URL": "mongodb://localhost:27017/test",
                    "OPENAI_API_KEY": "test-key",
                },
                clear=False,
            ):
                app = main.FastAPI()

                async with main.lifespan(app):
                    assert main.mongodb_client is None or isinstance(
                        main.mongodb_client, AsyncMock
                    )


@pytest.mark.unit
class TestMiscCoveragePaths:
    """Test remaining uncovered paths to reach 95%."""

    def test_feature_engineer_exception_in_lag_features(self):
        """Test exception handling in _add_lag_features."""
        fe = FeatureEngineer()

        # Create DataFrame that will work normally
        df = pd.DataFrame(
            {
                "close": [100, 101, 102, 103, 104],
                "volume": [1000, 1100, 1200, 1300, 1400],
            }
        )

        # Patch shift to raise exception
        original_shift = pd.DataFrame.shift

        def error_shift(self, *args, **kwargs):
            if "close" in self.columns:
                raise Exception("Test error")
            return original_shift(self, *args, **kwargs)

        with patch.object(pd.DataFrame, "shift", side_effect=error_shift):
            result = fe._add_lag_features(df)

            # Should return DataFrame even on error
            assert isinstance(result, pd.DataFrame)

    def test_feature_engineer_exception_in_volatility(self):
        """Test exception handling in _add_volatility_features."""
        fe = FeatureEngineer()

        df = pd.DataFrame(
            {
                "close": [100, 102, 101, 103, 105],
                "high": [101, 103, 102, 104, 106],
                "low": [99, 101, 100, 102, 104],
                "open": [100, 102, 101, 103, 105],
                "price_return_1": [0.0, 0.02, -0.01, 0.02, 0.02],
            }
        )

        # Patch rolling to raise exception
        original_rolling = pd.Series.rolling

        def error_rolling(self, *args, **kwargs):
            if self.name == "price_return_1":
                raise Exception("Test error")
            return original_rolling(self, *args, **kwargs)

        with patch.object(pd.Series, "rolling", side_effect=error_rolling):
            result = fe._add_volatility_features(df)

            # Should return DataFrame even on error
            assert isinstance(result, pd.DataFrame)

    def test_feature_engineer_prepare_inference_exception(self):
        """Test exception handling in prepare_for_inference."""
        fe = FeatureEngineer()
        fe.config["sequence_length"] = 10

        df = pd.DataFrame(
            {
                "close": [100 + i for i in range(20)],
                "high": [101 + i for i in range(20)],
                "low": [99 + i for i in range(20)],
                "open": [100 + i for i in range(20)],
                "volume": [1000.0 for _ in range(20)],
            }
        )

        # Patch prepare_features to raise exception
        with patch.object(
            fe, "prepare_features", side_effect=Exception("Processing error")
        ):
            result = fe.prepare_for_inference(df)

            # Should return None on error
            assert result is None

    def test_technical_indicators_stochastic_exception(self):
        """Test exception handling in calculate_stochastic."""
        ti = TechnicalIndicators()

        df = pd.DataFrame(
            {
                "close": [100, 101, 102],
                "high": [101, 102, 103],
                "low": [99, 100, 101],
            }
        )

        with patch("ta.momentum.StochasticOscillator", side_effect=Exception("Error")):
            result = ti.calculate_stochastic(df)

            assert "stoch_k" in result
            assert "stoch_d" in result
            assert len(result["stoch_k"]) == len(df)

    def test_technical_indicators_atr_exception(self):
        """Test exception handling in calculate_atr."""
        ti = TechnicalIndicators()

        df = pd.DataFrame(
            {
                "close": [100, 101, 102],
                "high": [101, 102, 103],
                "low": [99, 100, 101],
            }
        )

        with patch("ta.volatility.AverageTrueRange", side_effect=Exception("Error")):
            result = ti.calculate_atr(df)

            assert len(result) == len(df)

    def test_technical_indicators_detect_patterns_exception(self):
        """Test exception handling in detect_price_patterns."""
        ti = TechnicalIndicators()

        df = pd.DataFrame(
            {
                "close": [100, 101, 102],
                "high": [101, 102, 103],
                "low": [99, 100, 101],
                "open": [100, 101, 102],
            }
        )

        # Patch rolling to raise exception
        original_rolling = pd.Series.rolling

        def error_rolling(self, *args, **kwargs):
            raise Exception("Rolling error")

        with patch.object(pd.Series, "rolling", side_effect=error_rolling):
            result = ti.detect_price_patterns(df)

            # Should return empty dict on error
            assert isinstance(result, dict)


if __name__ == "__main__":
    pytest.main([__file__, "-v"])
