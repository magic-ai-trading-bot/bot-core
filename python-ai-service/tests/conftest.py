"""
Pytest configuration and fixtures for Python AI Service tests.
"""

import asyncio
import pytest
import pytest_asyncio
from typing import AsyncGenerator, Generator
from unittest.mock import AsyncMock, MagicMock, patch
from httpx import AsyncClient
from fastapi.testclient import TestClient
from datetime import datetime, timezone

# Import the FastAPI app
import sys
import os
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

@pytest.fixture(scope="session")
def event_loop():
    """Create an instance of the default event loop for the test session."""
    loop = asyncio.new_event_loop()
    yield loop
    loop.close()

@pytest.fixture
def mock_openai_client():
    """Mock OpenAI client."""
    mock = AsyncMock()

    # Mock response as dictionary (as expected by the code)
    mock_response = {
        "choices": [{
            "message": {
                "content": '{"signal": "Long", "confidence": 0.75, "reasoning": "Strong bullish momentum based on technical indicators", "strategy_scores": {"RSI": 0.8, "MACD": 0.7}, "market_analysis": {"trend_direction": "Bullish", "trend_strength": 0.75, "support_levels": [45000], "resistance_levels": [46000], "volatility_level": "Medium", "volume_analysis": "Increasing volume"}, "risk_assessment": {"overall_risk": "Medium", "technical_risk": 0.4, "market_risk": 0.5, "recommended_position_size": 0.03}}'
            }
        }]
    }

    # Mock the custom chat_completions_create method
    mock.chat_completions_create = AsyncMock(return_value=mock_response)

    return mock

@pytest.fixture
def mock_mongodb():
    """Mock MongoDB client and database."""
    mock_client = AsyncMock()
    mock_db = AsyncMock()

    # Mock collection
    mock_collection = AsyncMock()
    mock_collection.insert_one = AsyncMock(
        return_value=MagicMock(inserted_id="test_id_123")
    )
    mock_collection.find_one = AsyncMock(return_value=None)
    mock_collection.count_documents = AsyncMock(return_value=100)
    mock_collection.delete_many = AsyncMock(
        return_value=MagicMock(deleted_count=50)
    )

    # Mock __getitem__ to return collection
    mock_db.__getitem__ = MagicMock(return_value=mock_collection)

    # Mock admin commands
    mock_client.admin.command = AsyncMock(return_value={"ok": 1})
    mock_client.get_default_database = MagicMock(return_value=mock_db)

    return mock_client, mock_db

@pytest.fixture
def app(mock_openai_client, mock_mongodb):
    """Create FastAPI test app with mocked dependencies."""
    import main

    # Get MongoDB mocks
    mongo_client, mongo_db = mock_mongodb

    # Patch the global variables in main module
    main.openai_client = mock_openai_client
    main.mongodb_client = mongo_client
    main.mongodb_db = mongo_db
    main.gpt_analyzer = None  # Reset analyzer

    # Mock WebSocket manager's active_connections
    if hasattr(main, 'ws_manager'):
        main.ws_manager.active_connections = set()

    yield main.app

    # Cleanup
    main.openai_client = None
    main.mongodb_client = None
    main.mongodb_db = None
    main.gpt_analyzer = None

@pytest_asyncio.fixture
async def client(app) -> AsyncGenerator[AsyncClient, None]:
    """Create async test client."""
    async with AsyncClient(app=app, base_url="http://test") as ac:
        yield ac

@pytest.fixture
def test_client(app) -> TestClient:
    """Create sync test client for WebSocket tests."""
    return TestClient(app)

@pytest.fixture
def sample_candle_data():
    """Sample candle data for testing."""
    return [
        {
            "open": 45123.45,
            "high": 45234.56,
            "low": 45012.34,
            "close": 45189.23,
            "volume": 1234.56,
            "timestamp": 1701234567000
        }
    ]

@pytest.fixture
def sample_technical_indicators():
    """Sample technical indicators for testing."""
    return {
        "rsi": 65.5,
        "macd": 123.45,
        "signal": 110.23,
        "histogram": 13.22,
        "ema_9": 45100.00,
        "ema_21": 44900.00,
        "ema_50": 44500.00,
        "bollinger_upper": 45500.00,
        "bollinger_middle": 45000.00,
        "bollinger_lower": 44500.00,
        "volume_sma": 1000.00,
        "atr": 234.56,
        "adx": 25.5,
        "stochastic_k": 75.5,
        "stochastic_d": 72.3
    }

@pytest.fixture
def sample_market_context():
    """Sample market context for testing."""
    return {
        "trend_strength": 0.75,
        "volatility": 0.45,
        "volume_trend": "increasing",
        "market_sentiment": "bullish"
    }

@pytest.fixture
def sample_ai_analysis_request(sample_candle_data, sample_technical_indicators, sample_market_context):
    """Complete AI analysis request."""
    return {
        "symbol": "BTCUSDT",
        "timeframe_data": {
            "1h": sample_candle_data
        },
        "current_price": 45189.23,
        "volume_24h": 25000000000.0,
        "timestamp": 1701234567000,
        "strategy_context": {
            "selected_strategies": ["RSI", "MACD"],
            "risk_tolerance": "medium",
            "trading_style": "swing",
            "technical_indicators": sample_technical_indicators,
            "market_context": sample_market_context
        }
    }

@pytest.fixture
def mock_datetime():
    """Mock datetime for consistent testing."""
    test_time = datetime(2025, 7, 31, 18, 0, 0, tzinfo=timezone.utc)
    with patch('main.datetime') as mock_dt:
        mock_dt.now.return_value = test_time
        mock_dt.utcnow.return_value = test_time
        yield test_time