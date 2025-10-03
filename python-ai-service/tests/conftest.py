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
    mock.chat.completions.create = AsyncMock(return_value=MagicMock(
        choices=[MagicMock(
            message=MagicMock(
                content='{"signal": "Long", "confidence": 0.75, "reasoning": "Test reasoning"}'
            )
        )]
    ))
    return mock

@pytest.fixture
def mock_mongodb():
    """Mock MongoDB client and database."""
    mock_client = AsyncMock()
    mock_db = AsyncMock()
    
    # Mock collections
    mock_db.__getitem__ = MagicMock(return_value=AsyncMock())
    mock_db["ai_analysis_results"].insert_one = AsyncMock(
        return_value=MagicMock(inserted_id="test_id")
    )
    mock_db["ai_analysis_results"].find_one = AsyncMock(return_value=None)
    mock_db["ai_analysis_results"].count_documents = AsyncMock(return_value=0)
    
    # Mock admin commands
    mock_client.admin.command = AsyncMock(return_value={"ok": 1})
    mock_client.get_default_database = MagicMock(return_value=mock_db)
    
    return mock_client, mock_db

@pytest.fixture
def app(mock_openai_client, mock_mongodb):
    """Create FastAPI test app."""
    with patch('main.openai_client') as mock_openai, \
         patch('main.mongodb_client') as mock_mongo_client, \
         patch('main.mongodb_db') as mock_mongo_db:

        # Import app after patching
        from main import app as fastapi_app

        # Set up mocks
        mock_openai.return_value = mock_openai_client
        mongo_client, mongo_db = mock_mongodb
        mock_mongo_client.return_value = mongo_client
        mock_mongo_db.return_value = mongo_db

        yield fastapi_app

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
        "timeframe": "1h",
        "candles": sample_candle_data,
        "technical_indicators": sample_technical_indicators,
        "market_context": sample_market_context
    }

@pytest.fixture
def mock_datetime():
    """Mock datetime for consistent testing."""
    test_time = datetime(2025, 7, 31, 18, 0, 0, tzinfo=timezone.utc)
    with patch('main.datetime') as mock_dt:
        mock_dt.now.return_value = test_time
        mock_dt.utcnow.return_value = test_time
        yield test_time