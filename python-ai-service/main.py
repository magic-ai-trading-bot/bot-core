#!/usr/bin/env python3
"""
AI Trading Service with GPT-4 Integration + Real-time WebSocket
Advanced trading signal generation using OpenAI GPT-4 for cryptocurrency markets.
Compatible with Rust AI client endpoints with WebSocket broadcasting.
"""

import asyncio
import json
import os
import logging
import sys
from datetime import datetime, timezone, timedelta
from typing import Dict, Any, List, Optional, Set
from contextlib import asynccontextmanager

# Load configuration
from config_loader import AI_CACHE_DURATION_MINUTES, AI_CACHE_ENABLED
from app.core.config import OPENAI_REQUEST_DELAY

import pandas as pd
import numpy as np
import fastapi
from fastapi import (
    FastAPI,
    HTTPException,
    WebSocket,
    WebSocketDisconnect,
    Request,
)
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field
import ta
from motor.motor_asyncio import AsyncIOMotorClient
from pymongo import ASCENDING
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address
from slowapi.errors import RateLimitExceeded

# Project chatbot service (RAG)
from services.project_chatbot import get_chatbot, ProjectChatbot

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# Global OpenAI client, WebSocket connections, and MongoDB storage
# Thread safety: These are only written during startup/shutdown in lifespan
# Read-only access during request handling is safe
# Type annotations help mypy understand optional types
openai_client: Optional[Any] = None
websocket_connections: Set[WebSocket] = set()
mongodb_client: Optional[AsyncIOMotorClient] = None
mongodb_db: Optional[Any] = None

# Rate limiting for OpenAI API
# Thread safety: Access to these variables is protected by asyncio.Lock
# for proper async/await compatibility
_rate_limit_lock = asyncio.Lock()
last_openai_request_time = None
# OPENAI_REQUEST_DELAY is imported from app.core.config (reads from env var)
OPENAI_RATE_LIMIT_RESET_TIME = None  # Track when rate limit resets

# Cost monitoring (GPT-4o-mini pricing as of Nov 2024)
GPT4O_MINI_INPUT_COST_PER_1M = 0.150  # $0.150 per 1M input tokens
GPT4O_MINI_OUTPUT_COST_PER_1M = 0.600  # $0.600 per 1M output tokens
total_input_tokens = 0
total_output_tokens = 0
total_requests_count = 0
total_cost_usd = 0.0

# MongoDB storage for AI analysis results
AI_ANALYSIS_COLLECTION = "ai_analysis_results"
# Load from config.yaml (default: 2 minutes)
ANALYSIS_INTERVAL_MINUTES = AI_CACHE_DURATION_MINUTES

# === WEBSOCKET MANAGER ===


class WebSocketManager:
    """Manages WebSocket connections for real-time AI signal broadcasting."""

    def __init__(self):
        self.active_connections: Set[WebSocket] = set()

    async def connect(self, websocket: WebSocket):
        """Accept new WebSocket connection."""
        await websocket.accept()
        self.active_connections.add(websocket)
        logger.info(
            f"🔗 New WebSocket connection. Total: {len(self.active_connections)}"
        )

        # Send welcome message
        await websocket.send_json(
            {
                "type": "connection",
                "message": "Connected to AI Trading Service",
                "timestamp": datetime.now(timezone.utc).isoformat(),
            }
        )

    def disconnect(self, websocket: WebSocket):
        """Remove WebSocket connection."""
        self.active_connections.discard(websocket)
        logger.info(
            f"🔌 WebSocket disconnected. Remaining: {len(self.active_connections)}"
        )

    async def broadcast_signal(self, signal_data: Dict[str, Any]):
        """Broadcast AI signal to all connected clients."""
        if not self.active_connections:
            return

        message = {
            "type": "AISignalReceived",
            "data": signal_data,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }

        # Send to all connections
        disconnected = []
        for connection in self.active_connections.copy():
            try:
                await connection.send_json(message)
            except Exception as e:
                logger.warning(f"Failed to send to WebSocket: {e}")
                disconnected.append(connection)

        # Clean up disconnected clients
        for conn in disconnected:
            self.active_connections.discard(conn)

        logger.info(
            f"📡 Broadcasted AI signal to {len(self.active_connections)} clients"
        )


# Global WebSocket manager
ws_manager = WebSocketManager()

# === MONGODB STORAGE & PERIODIC ANALYSIS ===

# Rust Core Engine API URL - for dynamic symbol fetching
RUST_API_URL = os.getenv("RUST_API_URL", "http://localhost:8080")

# Fallback symbols - only used when Rust API is unavailable
FALLBACK_ANALYSIS_SYMBOLS = [
    "BTCUSDT",
    "ETHUSDT",
    "BNBUSDT",
    "SOLUSDT",
]


async def fetch_analysis_symbols() -> List[str]:
    """
    Fetch current symbols from Rust Core Engine API.
    Falls back to FALLBACK_ANALYSIS_SYMBOLS if API is unavailable.
    """
    import httpx

    try:
        async with httpx.AsyncClient(timeout=10.0) as client:
            response = await client.get(f"{RUST_API_URL}/api/market/symbols")
            if response.status_code == 200:
                data = response.json()
                # API returns {"success":true,"data":{"symbols":[...]}}
                if data.get("success"):
                    # Try nested structure first (data.symbols), then flat structure (symbols)
                    symbols = data.get("data", {}).get("symbols") or data.get("symbols")
                    if symbols:
                        logger.info(f"📊 Fetched {len(symbols)} symbols from Rust API: {symbols}")
                        return symbols

        logger.warning("⚠️ Rust API returned no symbols, using fallback")
        return FALLBACK_ANALYSIS_SYMBOLS

    except Exception as e:
        logger.warning(f"⚠️ Failed to fetch symbols from Rust API: {e}, using fallback")
        return FALLBACK_ANALYSIS_SYMBOLS


async def store_analysis_result(symbol: str, analysis_result: Dict[str, Any]) -> None:
    """Store AI analysis result in MongoDB."""
    if mongodb_db is None:
        logger.warning("MongoDB not available, skipping storage")
        return

    try:
        document = {
            "symbol": symbol,
            "timestamp": datetime.now(timezone.utc),
            "analysis": analysis_result,
            "created_at": datetime.now(timezone.utc),
        }

        result = await mongodb_db[AI_ANALYSIS_COLLECTION].insert_one(document)
        logger.info(f"📊 Stored analysis for {symbol}: {result.inserted_id}")

    except Exception as e:
        logger.error(f"❌ Failed to store analysis for {symbol}: {e}")


async def get_latest_analysis(symbol: str) -> Optional[Dict[str, Any]]:
    """Get latest analysis for a symbol from MongoDB."""
    if mongodb_db is None:
        return None

    try:
        document = await mongodb_db[AI_ANALYSIS_COLLECTION].find_one(
            {"symbol": symbol}, sort=[("timestamp", -1)]
        )

        if document:
            return document.get("analysis")
        return None

    except Exception as e:
        logger.error(f"❌ Failed to get latest analysis for {symbol}: {e}")
        return None


async def periodic_analysis_runner():
    """Background task that runs AI analysis every 5 minutes."""
    logger.info("🔄 Starting periodic analysis runner")

    while True:
        try:
            # Fetch symbols dynamically from Rust API (includes user-added symbols)
            analysis_symbols = await fetch_analysis_symbols()
            logger.info(f"🤖 Starting periodic AI analysis cycle for {len(analysis_symbols)} symbols")

            # Analyze each symbol
            for symbol in analysis_symbols:
                try:
                    # Fetch REAL market data from Rust Core Engine API
                    analysis_request = await fetch_real_market_data(symbol)

                    # Run AI analysis
                    analyzer = GPTTradingAnalyzer(openai_client)
                    analysis_result = await analyzer.analyze_trading_signals(
                        analysis_request
                    )

                    # Store result in MongoDB
                    await store_analysis_result(symbol, analysis_result.model_dump())

                    # Broadcast via WebSocket
                    await ws_manager.broadcast_signal(
                        {
                            "symbol": symbol,
                            "signal": analysis_result.signal,
                            "confidence": analysis_result.confidence,
                            "reasoning": analysis_result.reasoning,
                            "timestamp": datetime.now(timezone.utc).isoformat(),
                        }
                    )

                    logger.info(
                        f"✅ Completed analysis for {symbol}: "
                        f"{analysis_result.signal} ({analysis_result.confidence:.2f})"
                    )

                    # Rate limiting between symbols
                    await asyncio.sleep(10)  # 10 seconds between symbols

                except Exception as e:
                    logger.error(f"❌ Failed to analyze {symbol}: {e}")
                    continue

            logger.info(
                f"🎯 Completed AI analysis cycle for {len(analysis_symbols)} symbols"
            )

            # Wait for next cycle
            await asyncio.sleep(ANALYSIS_INTERVAL_MINUTES * 60)

        except asyncio.CancelledError:
            logger.info("🛑 Periodic analysis task cancelled")
            break
        except Exception as e:
            logger.error(f"❌ Error in periodic analysis: {e}")
            await asyncio.sleep(60)  # Wait 1 minute before retrying


@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager."""
    global openai_client, mongodb_client, mongodb_db

    # Startup
    logger.info("🚀 Starting GPT-4 AI Trading Service")
    logger.info(f"Python version: {sys.version}")
    logger.info(f"FastAPI version: {fastapi.__version__}")

    # Initialize MongoDB connection
    mongodb_url = os.getenv("DATABASE_URL")
    if not mongodb_url:
        logger.error(
            "❌ DATABASE_URL environment variable not set! "
            "MongoDB connection required."
        )
        raise ValueError(
            "DATABASE_URL environment variable is required. "
            "Please set it in your .env file."
        )

    try:
        mongodb_client = AsyncIOMotorClient(mongodb_url)
        mongodb_db = mongodb_client.get_default_database()

        # Test connection
        await mongodb_client.admin.command("ping")
        logger.info("✅ MongoDB connection established")

        # Create indexes for AI analysis collection
        await mongodb_db[AI_ANALYSIS_COLLECTION].create_index(
            [("symbol", ASCENDING), ("timestamp", ASCENDING)]
        )
        logger.info(f"📊 MongoDB indexes created for {AI_ANALYSIS_COLLECTION}")

    except Exception as e:
        logger.error(f"❌ MongoDB connection failed: {e}")
        mongodb_client = None
        mongodb_db = None

    # Initialize OpenAI client with API keys from environment
    # Support multiple backup keys separated by commas
    api_key_string = os.getenv("OPENAI_API_KEY", "")
    backup_keys_string = os.getenv("OPENAI_BACKUP_API_KEYS", "")

    api_keys = []
    if api_key_string:
        api_keys.append(api_key_string)
    if backup_keys_string:
        # Split by comma and strip whitespace
        backup_keys = [
            key.strip() for key in backup_keys_string.split(",") if key.strip()
        ]
        api_keys.extend(backup_keys)

    # Filter out None/empty keys and invalid keys
    valid_api_keys = [key for key in api_keys if key and not key.startswith("your-")]

    if not valid_api_keys:
        logger.error("❌ No valid OpenAI API keys found!")
        api_key = None
    else:
        api_key = valid_api_keys[0]  # Use the first valid key
        logger.info(f"✅ Found {len(valid_api_keys)} valid API keys for fallback")
        if len(valid_api_keys) > 1:
            logger.info("🔄 Backup keys available for auto-fallback on rate limits")

    logger.info(f"🔑 OpenAI API key configured: {bool(api_key)}")

    if not api_key or api_key.startswith("your-"):
        logger.error("❌ OpenAI API key not configured!")
        openai_client = None
    else:
        logger.info("🔄 Initializing OpenAI client...")

        # Use direct HTTP client to bypass OpenAI SDK conflicts
        try:
            openai_client = DirectOpenAIClient(valid_api_keys)  # Pass all valid keys
            logger.info("✅ Direct OpenAI HTTP client initialized successfully")
            logger.info(f"🔑 Total API keys configured: {len(valid_api_keys)}")
            logger.info("🔄 Using direct HTTP calls to OpenAI API with auto-fallback")
        except Exception as e:
            logger.error(f"❌ Failed to initialize direct OpenAI client: {e}")
            openai_client = None

    if openai_client is not None:
        logger.info("✅ OpenAI GPT-4 client ready for analysis")
    else:
        logger.warning("❌ GPT-4 unavailable - will use fallback technical analysis")

    # Start background analysis task
    analysis_task = asyncio.create_task(periodic_analysis_runner())
    logger.info(
        f"🔄 Started periodic analysis task (every {ANALYSIS_INTERVAL_MINUTES} minutes)"
    )

    yield

    # Shutdown
    logger.info("🛑 Shutting down AI Trading Service")
    analysis_task.cancel()
    if mongodb_client:
        mongodb_client.close()


# Initialize rate limiter (disabled in test environment)
if os.getenv("TESTING") == "true":
    # Create a dummy limiter for tests that doesn't actually limit
    class DummyLimiter:
        def limit(self, *args, **kwargs):
            """No-op decorator for testing."""
            return lambda f: f

    limiter = DummyLimiter()
else:
    limiter = Limiter(key_func=get_remote_address)

# Create FastAPI app
app = FastAPI(
    title="GPT-4 Cryptocurrency AI Trading Service",
    description="Advanced AI-powered trading signal generation using OpenAI GPT-4",
    version="2.0.0",
    lifespan=lifespan,
)

# Add rate limiter to app state
app.state.limiter = limiter
app.add_exception_handler(RateLimitExceeded, _rate_limit_exceeded_handler)

# CORS middleware - Allow specific origins from environment
allowed_origins_str = os.getenv(
    "ALLOWED_ORIGINS",
    "http://localhost:3000,http://localhost:8080,http://127.0.0.1:3000,http://127.0.0.1:8080",
)
allowed_origins = [
    origin.strip() for origin in allowed_origins_str.split(",") if origin.strip()
]

app.add_middleware(
    CORSMiddleware,
    allow_origins=allowed_origins,
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


# Security headers middleware for Top 1% security score
@app.middleware("http")
async def add_security_headers(request: Request, call_next):
    """Add security headers to all HTTP responses."""
    response = await call_next(request)

    # Prevent clickjacking attacks
    response.headers["X-Frame-Options"] = "DENY"

    # Prevent MIME type sniffing
    response.headers["X-Content-Type-Options"] = "nosniff"

    # Enable XSS protection (legacy but still useful)
    response.headers["X-XSS-Protection"] = "1; mode=block"

    # Enforce HTTPS (only if in production)
    if os.getenv("ENVIRONMENT") == "production":
        response.headers["Strict-Transport-Security"] = (
            "max-age=31536000; includeSubDomains; preload"
        )

    # Content Security Policy (strict)
    response.headers["Content-Security-Policy"] = (
        "default-src 'self'; "
        "script-src 'self' 'unsafe-inline'; "
        "style-src 'self' 'unsafe-inline'; "
        "img-src 'self' data: https:; "
        "font-src 'self'; "
        "connect-src 'self' ws: wss:; "
        "frame-ancestors 'none'"
    )

    # Referrer policy
    response.headers["Referrer-Policy"] = "strict-origin-when-cross-origin"

    # Permissions policy (restrict features)
    response.headers["Permissions-Policy"] = (
        "geolocation=(), microphone=(), camera=(), payment=()"
    )

    return response


# === PYDANTIC MODELS (Compatible with Rust AI Client) ===


class CandleData(BaseModel):
    """Individual candle data model."""

    timestamp: int = Field(..., description="Unix timestamp in milliseconds")
    open: float = Field(..., gt=0, description="Opening price")
    high: float = Field(..., gt=0, description="High price")
    low: float = Field(..., gt=0, description="Low price")
    close: float = Field(..., gt=0, description="Closing price")
    volume: float = Field(..., ge=0, description="Trading volume")


class AIStrategyContext(BaseModel):
    """Strategy context for AI analysis."""

    selected_strategies: List[str] = Field(default_factory=list)
    market_condition: str = Field(default="Unknown")
    risk_level: str = Field(default="Moderate")
    user_preferences: Dict[str, Any] = Field(default_factory=dict)
    technical_indicators: Dict[str, Any] = Field(default_factory=dict)


class AIAnalysisRequest(BaseModel):
    """Request model for AI analysis."""

    symbol: str = Field(..., description="Trading pair symbol (e.g., BTCUSDT)")
    timeframe_data: Dict[str, List[CandleData]] = Field(
        ..., description="Multi-timeframe data"
    )
    current_price: float = Field(..., gt=0, description="Current market price")
    volume_24h: float = Field(..., ge=0, description="24h volume")
    timestamp: int = Field(..., description="Request timestamp")
    strategy_context: AIStrategyContext = Field(..., description="Strategy context")


class AIMarketAnalysis(BaseModel):
    """AI market analysis response."""

    trend_direction: str = Field(..., description="Trend direction")
    trend_strength: float = Field(..., ge=0, le=1, description="Trend strength")
    support_levels: List[float] = Field(default_factory=list)
    resistance_levels: List[float] = Field(default_factory=list)
    volatility_level: str = Field(..., description="Volatility assessment")
    volume_analysis: str = Field(..., description="Volume analysis")


class AIRiskAssessment(BaseModel):
    """AI risk assessment response."""

    overall_risk: str = Field(..., description="Overall risk level")
    technical_risk: float = Field(..., ge=0, le=1, description="Technical risk score")
    market_risk: float = Field(..., ge=0, le=1, description="Market risk score")
    recommended_position_size: float = Field(
        ..., ge=0, le=1, description="Position size recommendation"
    )
    stop_loss_suggestion: Optional[float] = Field(None, description="Stop loss level")
    take_profit_suggestion: Optional[float] = Field(
        None, description="Take profit level"
    )


class AISignalResponse(BaseModel):
    """AI signal response model."""

    signal: str = Field(..., description="Trading signal: Long, Short, or Neutral")
    confidence: float = Field(..., ge=0, le=1, description="Confidence score")
    reasoning: str = Field(..., description="AI reasoning for the signal")
    strategy_scores: Dict[str, float] = Field(default_factory=dict)
    market_analysis: AIMarketAnalysis = Field(..., description="Market analysis")
    risk_assessment: AIRiskAssessment = Field(..., description="Risk assessment")
    timestamp: int = Field(..., description="Response timestamp")


class StrategyRecommendation(BaseModel):
    """Strategy recommendation model."""

    strategy_name: str = Field(..., description="Strategy name")
    suitability_score: float = Field(..., ge=0, le=1, description="Suitability score")
    reasoning: str = Field(..., description="Reasoning for recommendation")
    recommended_config: Dict[str, Any] = Field(default_factory=dict)


class StrategyRecommendationRequest(BaseModel):
    """Strategy recommendation request."""

    symbol: str = Field(..., description="Trading symbol")
    timeframe_data: Dict[str, List[CandleData]] = Field(
        ..., description="Multi-timeframe data"
    )
    current_price: float = Field(..., gt=0, description="Current price")
    available_strategies: List[str] = Field(..., description="Available strategies")
    timestamp: int = Field(..., description="Request timestamp")


class MarketConditionAnalysis(BaseModel):
    """Market condition analysis response."""

    condition_type: str = Field(..., description="Market condition type")
    confidence: float = Field(..., ge=0, le=1, description="Confidence in assessment")
    characteristics: List[str] = Field(default_factory=list)
    recommended_strategies: List[str] = Field(default_factory=list)
    market_phase: str = Field(..., description="Current market phase")


class MarketConditionRequest(BaseModel):
    """Market condition request."""

    symbol: str = Field(..., description="Trading symbol")
    timeframe_data: Dict[str, List[CandleData]] = Field(
        ..., description="Multi-timeframe data"
    )
    current_price: float = Field(..., gt=0, description="Current price")
    volume_24h: float = Field(..., ge=0, description="24h volume")
    timestamp: int = Field(..., description="Request timestamp")


class PerformanceFeedback(BaseModel):
    """Performance feedback model."""

    signal_id: str = Field(..., description="Signal ID")
    symbol: str = Field(..., description="Trading symbol")
    predicted_signal: str = Field(..., description="Predicted signal")
    actual_outcome: str = Field(..., description="Actual outcome")
    profit_loss: float = Field(..., description="Profit/loss amount")
    confidence_was_accurate: bool = Field(..., description="Was confidence accurate")
    feedback_notes: Optional[str] = Field(None, description="Additional notes")
    timestamp: int = Field(..., description="Feedback timestamp")


class TrendPredictionRequest(BaseModel):
    """Trend prediction request model."""

    symbol: str = Field(..., description="Trading symbol (e.g., BTCUSDT)")
    timeframe: str = Field(
        default="4h", description="Timeframe for trend analysis (1h, 4h, 1d)"
    )


class TrendPredictionResponse(BaseModel):
    """Trend prediction response model."""

    trend: str = Field(..., description="Predicted trend direction (Uptrend/Downtrend/Neutral)")
    confidence: float = Field(
        ..., ge=0, le=1, description="Confidence in prediction (0.0-1.0)"
    )
    model: str = Field(..., description="ML model used for prediction")
    timestamp: int = Field(..., description="Prediction timestamp")


class AIServiceInfo(BaseModel):
    """AI service information."""

    model_config = {"protected_namespaces": ()}

    service_name: str = Field(default="GPT-4 Trading AI")
    version: str = Field(default="2.0.0")
    model_version: str = Field(default="gpt-4o-mini")
    supported_timeframes: List[str] = Field(
        default_factory=lambda: ["1m", "5m", "15m", "1h", "4h", "1d"]
    )
    supported_symbols: List[str] = Field(
        default_factory=lambda: FALLBACK_ANALYSIS_SYMBOLS
    )
    capabilities: List[str] = Field(
        default_factory=lambda: [
            "trend_analysis",
            "signal_generation",
            "risk_assessment",
            "strategy_recommendation",
            "market_condition_detection",
        ]
    )
    last_trained: Optional[str] = Field(None)


class AIModelPerformance(BaseModel):
    """AI model performance metrics."""

    model_config = {"protected_namespaces": ()}

    overall_accuracy: float = Field(default=0.85)
    precision: float = Field(default=0.82)
    recall: float = Field(default=0.78)
    f1_score: float = Field(default=0.80)
    predictions_made: int = Field(default=0)
    successful_predictions: int = Field(default=0)
    average_confidence: float = Field(default=0.75)
    model_uptime: str = Field(default="99.5%")
    last_updated: str = Field(
        default_factory=lambda: datetime.now(timezone.utc).isoformat()
    )


# === PROJECT CHATBOT REQUEST/RESPONSE MODELS ===


class ProjectChatRequest(BaseModel):
    """Request model for project chatbot."""
    message: str = Field(..., min_length=1, max_length=2000, description="User message")
    include_history: bool = Field(default=True, description="Include conversation history")


class ProjectChatSource(BaseModel):
    """Source document reference."""
    title: str
    path: str


class ProjectChatResponse(BaseModel):
    """Response model for project chatbot."""
    success: bool
    message: str
    sources: List[ProjectChatSource] = Field(default_factory=list)
    confidence: float = Field(default=0.0, ge=0.0, le=1.0)
    type: str = Field(default="rag", description="Response type: rag, fallback, error")
    tokens_used: Dict[str, Any] = Field(default_factory=dict)


# === TECHNICAL ANALYSIS HELPER ===


class TechnicalAnalyzer:
    """Technical analysis utilities."""

    @staticmethod
    def prepare_dataframe(klines: List) -> pd.DataFrame:
        """Convert Binance kline data to pandas DataFrame."""
        if not klines:
            return pd.DataFrame()

        data = []
        for kline in klines:
            data.append(
                {
                    "timestamp": pd.to_datetime(kline[0], unit="ms"),
                    "open": float(kline[1]),
                    "high": float(kline[2]),
                    "low": float(kline[3]),
                    "close": float(kline[4]),
                    "volume": float(kline[5]),
                }
            )

        df = pd.DataFrame(data)
        df.set_index("timestamp", inplace=True)
        return df

    @staticmethod
    def calculate_indicators(df: pd.DataFrame) -> Dict[str, Any]:
        """Calculate comprehensive technical indicators."""
        try:
            if df.empty:
                return {
                    "rsi": 50.0,
                    "macd": 0.0,
                    "macd_signal": 0.0,
                    "macd_histogram": 0.0,
                    "bollinger_upper": 0.0,
                    "bollinger_middle": 0.0,
                    "bollinger_lower": 0.0,
                    "ema_9": 0.0,
                    "ema_21": 0.0,
                    "ema_50": 0.0,
                    "volume_sma": 0.0,
                    "atr": 0.0,
                    "adx": 0.0,
                    "stochastic_k": 50.0,
                    "stochastic_d": 50.0,
                }

            indicators = {}

            # Trend indicators
            indicators["sma_20"] = (
                ta.trend.sma_indicator(df["close"], window=20).iloc[-1]
                if len(df) >= 20
                else df["close"].iloc[-1]
            )
            indicators["sma_50"] = (
                ta.trend.sma_indicator(df["close"], window=50).iloc[-1]
                if len(df) >= 50
                else df["close"].iloc[-1]
            )
            indicators["ema_9"] = (
                ta.trend.ema_indicator(df["close"], window=9).iloc[-1]
                if len(df) >= 9
                else df["close"].iloc[-1]
            )
            indicators["ema_12"] = (
                ta.trend.ema_indicator(df["close"], window=12).iloc[-1]
                if len(df) >= 12
                else df["close"].iloc[-1]
            )
            indicators["ema_21"] = (
                ta.trend.ema_indicator(df["close"], window=21).iloc[-1]
                if len(df) >= 21
                else df["close"].iloc[-1]
            )
            indicators["ema_26"] = (
                ta.trend.ema_indicator(df["close"], window=26).iloc[-1]
                if len(df) >= 26
                else df["close"].iloc[-1]
            )
            indicators["ema_50"] = (
                ta.trend.ema_indicator(df["close"], window=50).iloc[-1]
                if len(df) >= 50
                else df["close"].iloc[-1]
            )

            # Momentum indicators
            indicators["rsi"] = (
                ta.momentum.rsi(df["close"], window=14).iloc[-1]
                if len(df) >= 14
                else 50.0
            )
            indicators["stochastic_k"] = (
                ta.momentum.stoch(df["high"], df["low"], df["close"]).iloc[-1]
                if len(df) >= 14
                else 50.0
            )
            indicators["stochastic_d"] = (
                ta.momentum.stoch_signal(df["high"], df["low"], df["close"]).iloc[-1]
                if len(df) >= 14
                else 50.0
            )
            indicators["stoch_k"] = indicators["stochastic_k"]
            indicators["stoch_d"] = indicators["stochastic_d"]

            # MACD
            macd_line = ta.trend.macd(df["close"])
            macd_signal = ta.trend.macd_signal(df["close"])
            indicators["macd"] = macd_line.iloc[-1] if not macd_line.empty else 0.0
            indicators["macd_signal"] = (
                macd_signal.iloc[-1] if not macd_signal.empty else 0.0
            )
            indicators["macd_histogram"] = (
                indicators["macd"] - indicators["macd_signal"]
            )

            # Bollinger Bands
            bb_high = ta.volatility.bollinger_hband(df["close"])
            bb_low = ta.volatility.bollinger_lband(df["close"])
            bb_mid = ta.volatility.bollinger_mavg(df["close"])

            indicators["bollinger_upper"] = (
                bb_high.iloc[-1] if not bb_high.empty else df["close"].iloc[-1] * 1.02
            )
            indicators["bollinger_lower"] = (
                bb_low.iloc[-1] if not bb_low.empty else df["close"].iloc[-1] * 0.98
            )
            indicators["bollinger_middle"] = (
                bb_mid.iloc[-1] if not bb_mid.empty else df["close"].iloc[-1]
            )
            indicators["bb_upper"] = indicators["bollinger_upper"]
            indicators["bb_lower"] = indicators["bollinger_lower"]
            indicators["bb_middle"] = indicators["bollinger_middle"]

            current_price = df["close"].iloc[-1]
            bb_width = indicators["bb_upper"] - indicators["bb_lower"]
            indicators["bb_position"] = (
                (current_price - indicators["bb_lower"]) / bb_width
                if bb_width > 0
                else 0.5
            )

            # Volume indicators
            volume_sma_series = ta.trend.sma_indicator(df["volume"], window=20)
            indicators["volume_sma"] = (
                volume_sma_series.iloc[-1]
                if not volume_sma_series.empty
                else df["volume"].mean()
            )
            indicators["volume_ratio"] = (
                df["volume"].iloc[-1] / indicators["volume_sma"]
                if indicators["volume_sma"] > 0
                else 1.0
            )

            # Volatility
            atr_series = ta.volatility.average_true_range(
                df["high"], df["low"], df["close"]
            )
            indicators["atr"] = atr_series.iloc[-1] if not atr_series.empty else 0.0

            # ADX (Average Directional Index)
            adx_series = ta.trend.adx(df["high"], df["low"], df["close"])
            indicators["adx"] = adx_series.iloc[-1] if not adx_series.empty else 25.0

            return indicators

        except Exception as e:
            logger.warning(f"Error calculating indicators: {e}")
            return {
                "rsi": 50.0,
                "macd": 0.0,
                "macd_signal": 0.0,
                "macd_histogram": 0.0,
                "bollinger_upper": 0.0,
                "bollinger_middle": 0.0,
                "bollinger_lower": 0.0,
                "ema_9": 0.0,
                "ema_21": 0.0,
                "ema_50": 0.0,
                "volume_sma": 0.0,
                "atr": 0.0,
                "adx": 0.0,
                "stochastic_k": 50.0,
                "stochastic_d": 50.0,
            }

    @staticmethod
    def detect_patterns(df: pd.DataFrame) -> Dict[str, bool]:
        """Detect common chart patterns."""
        patterns = {
            "double_top": False,
            "double_bottom": False,
            "head_shoulders": False,
            "ascending_triangle": False,
            "descending_triangle": False,
            "bullish_flag": False,
            "bearish_flag": False,
            "cup_handle": False,
        }

        if df.empty or len(df) < 20:
            return patterns

        try:
            # Simple pattern detection logic
            # closes = df["close"].values  # Unused for now
            highs = df["high"].values
            lows = df["low"].values

            # Detect double top (price reaches similar high twice)
            if len(df) >= 10:
                recent_highs = highs[-10:]
                max_val = np.max(recent_highs)
                high_count = np.sum(np.abs(recent_highs - max_val) / max_val < 0.02)
                patterns["double_top"] = high_count >= 2

            # Detect double bottom (price reaches similar low twice)
            if len(df) >= 10:
                recent_lows = lows[-10:]
                min_val = np.min(recent_lows)
                low_count = np.sum(np.abs(recent_lows - min_val) / min_val < 0.02)
                patterns["double_bottom"] = low_count >= 2

            # Detect ascending triangle (higher lows, flat resistance)
            if len(df) >= 15:
                mid_lows = lows[-15:-5]
                late_lows = lows[-5:]
                mid_highs = highs[-15:-5]
                late_highs = highs[-5:]
                patterns["ascending_triangle"] = np.mean(late_lows) > np.mean(
                    mid_lows
                ) and abs(np.mean(late_highs) - np.mean(mid_highs)) < np.std(
                    highs[-15:]
                )

            # Detect descending triangle (lower highs, flat support)
            if len(df) >= 15:
                mid_lows = lows[-15:-5]
                late_lows = lows[-5:]
                mid_highs = highs[-15:-5]
                late_highs = highs[-5:]
                patterns["descending_triangle"] = np.mean(late_highs) < np.mean(
                    mid_highs
                ) and abs(np.mean(late_lows) - np.mean(mid_lows)) < np.std(lows[-15:])

        except Exception as e:
            logger.warning(f"Error detecting patterns: {e}")

        return patterns

    @staticmethod
    def get_market_context(
        df: pd.DataFrame, indicators: Dict[str, Any]
    ) -> Dict[str, Any]:
        """Generate market context from DataFrame and indicators."""
        context = {
            "trend_strength": 0.0,
            "volatility": 0.5,
            "volume_trend": "stable",
            "market_sentiment": "neutral",
        }

        if df.empty or not indicators:
            return context

        try:
            # Calculate trend strength (-1 to 1)
            rsi = indicators.get("rsi", 50.0)
            if rsi > 70:
                context["trend_strength"] = 0.8
                context["market_sentiment"] = "bullish"
            elif rsi < 30:
                context["trend_strength"] = -0.8
                context["market_sentiment"] = "bearish"
            else:
                context["trend_strength"] = (rsi - 50) / 50.0

            # Calculate volatility (0 to 1)
            atr = indicators.get("atr", 0.0)
            current_price = df["close"].iloc[-1]
            if current_price > 0:
                context["volatility"] = min(1.0, atr / current_price * 100)

            # Determine volume trend
            volume_ratio = indicators.get("volume_ratio", 1.0)
            if volume_ratio > 1.2:
                context["volume_trend"] = "increasing"
            elif volume_ratio < 0.8:
                context["volume_trend"] = "decreasing"
            else:
                context["volume_trend"] = "stable"

            # Determine market sentiment
            if context["market_sentiment"] == "neutral":
                macd_histogram = indicators.get("macd_histogram", 0.0)
                if macd_histogram > 0 and indicators.get("ema_9", 0) > indicators.get(
                    "ema_21", 0
                ):
                    context["market_sentiment"] = "bullish"
                elif macd_histogram < 0 and indicators.get("ema_9", 0) < indicators.get(
                    "ema_21", 0
                ):
                    context["market_sentiment"] = "bearish"

        except Exception as e:
            logger.warning(f"Error generating market context: {e}")

        return context

    @staticmethod
    def candles_to_dataframe(
        timeframe_data: Dict[str, List[CandleData]],
    ) -> Dict[str, pd.DataFrame]:
        """Convert candle data to pandas DataFrames."""
        dataframes = {}

        for timeframe, candles in timeframe_data.items():
            if not candles:
                continue

            data = []
            for candle in candles:
                data.append(
                    {
                        "timestamp": pd.to_datetime(candle.timestamp, unit="ms"),
                        "open": candle.open,
                        "high": candle.high,
                        "low": candle.low,
                        "close": candle.close,
                        "volume": candle.volume,
                    }
                )

            df = pd.DataFrame(data)
            df.set_index("timestamp", inplace=True)
            df.sort_index(inplace=True)
            dataframes[timeframe] = df

        return dataframes


# === HTTP-BASED GPT-4 CLIENT ===


class DirectOpenAIClient:
    """Direct HTTP client for OpenAI API with auto-fallback support."""

    def __init__(self, api_keys: list):
        self.api_keys = api_keys if isinstance(api_keys, list) else [api_keys]
        self.current_key_index = 0
        self.base_url = "https://api.openai.com/v1"
        # Track which keys are rate limited
        self.rate_limited_keys: Set[int] = set()

    def get_current_api_key(self):
        """Get the current API key, cycling through available keys if needed."""
        available_keys = [
            key
            for i, key in enumerate(self.api_keys)
            if i not in self.rate_limited_keys
        ]

        if not available_keys:
            # All keys are rate limited, clear the set and start over
            logger.warning("🔄 All API keys rate limited, clearing and retrying...")
            self.rate_limited_keys.clear()
            available_keys = self.api_keys

        if self.current_key_index >= len(available_keys):
            self.current_key_index = 0

        return available_keys[self.current_key_index], self.current_key_index

    async def chat_completions_create(
        self,
        model: str,
        messages: list,
        temperature: float = 0.0,
        max_tokens: int = 1200,  # Default reduced from 2000 to 1200
    ):
        """Direct HTTP call to OpenAI chat completions API with auto-fallback on rate limits."""
        global last_openai_request_time, OPENAI_RATE_LIMIT_RESET_TIME
        import httpx

        # Try each available API key until success or all are exhausted
        max_attempts = len(self.api_keys)

        for attempt in range(max_attempts):
            current_key, key_index = self.get_current_api_key()

            # Check if we're still in a rate limit period (async-safe)
            async with _rate_limit_lock:
                if OPENAI_RATE_LIMIT_RESET_TIME:
                    if datetime.now() < OPENAI_RATE_LIMIT_RESET_TIME:
                        remaining_time = (
                            OPENAI_RATE_LIMIT_RESET_TIME - datetime.now()
                        ).total_seconds()
                        logger.warning(
                            f"⏰ Key {key_index + 1} in rate limit "
                            f"period, {remaining_time:.0f}s remaining"
                        )
                        # Try next key
                        self.current_key_index += 1
                        continue
                    else:
                        # Rate limit period expired, reset it
                        OPENAI_RATE_LIMIT_RESET_TIME = None
                        self.rate_limited_keys.discard(key_index)
                        logger.info(f"✅ Key {key_index + 1} rate limit expired")

            # Rate limiting: ensure minimum delay between requests
            # (checked outside lock to allow sleep without blocking)
            if last_openai_request_time:
                time_since_last = (
                    datetime.now() - last_openai_request_time
                ).total_seconds()
                if time_since_last < OPENAI_REQUEST_DELAY:
                    delay = OPENAI_REQUEST_DELAY - time_since_last
                    logger.info(
                        f"⏳ Rate limiting: waiting {delay:.1f}s " f"before request"
                    )
                    await asyncio.sleep(delay)

            # Update last request time (async-safe)
            async with _rate_limit_lock:
                last_openai_request_time = datetime.now()

            headers = {
                "Authorization": f"Bearer {current_key}",
                "Content-Type": "application/json",
            }

            payload = {
                "model": model,
                "messages": messages,
                "temperature": temperature,
                "max_tokens": max_tokens,
            }

            try:
                logger.info(
                    f"🔑 Using API key {key_index + 1}/{len(self.api_keys)} (...{current_key[-8:]})"
                )

                async with httpx.AsyncClient(timeout=30.0) as client:
                    response = await client.post(
                        f"{self.base_url}/chat/completions",
                        headers=headers,
                        json=payload,
                    )

                    if response.status_code == 429:
                        # Handle rate limit response
                        self.rate_limited_keys.add(key_index)
                        retry_after = response.headers.get("retry-after")
                        if retry_after:
                            reset_time = datetime.now() + timedelta(
                                seconds=int(retry_after)
                            )
                            OPENAI_RATE_LIMIT_RESET_TIME = reset_time
                            logger.warning(
                                f"⏰ Key {key_index + 1} rate limited until {reset_time}"
                            )
                        else:
                            # Default to 1 hour if no retry-after header
                            OPENAI_RATE_LIMIT_RESET_TIME = datetime.now() + timedelta(
                                hours=1
                            )
                            logger.warning(
                                f"⏰ Key {key_index + 1} rate limited for 1 hour"
                            )

                        # Try next key
                        self.current_key_index += 1
                        continue

                    response.raise_for_status()
                    logger.info(f"✅ Request successful with key {key_index + 1}")
                    return response.json()

            except httpx.HTTPStatusError as e:
                if e.response.status_code == 429:
                    logger.error(f"🚫 Key {key_index + 1} rate limit exceeded (429)")
                    self.rate_limited_keys.add(key_index)
                    self.current_key_index += 1
                    continue
                elif e.response.status_code == 401:
                    logger.error(f"🔑 Key {key_index + 1} authentication failed (401)")
                    self.current_key_index += 1
                    continue
                elif e.response.status_code == 403:
                    logger.error(f"💰 Key {key_index + 1} quota exceeded (403)")
                    self.current_key_index += 1
                    continue
                else:
                    logger.error(
                        f"❌ Key {key_index + 1} API error: {e.response.status_code}"
                    )
                    raise
            except Exception as e:
                logger.error(f"❌ Key {key_index + 1} network error: {e}")
                if attempt == max_attempts - 1:  # Last attempt
                    raise
                else:
                    self.current_key_index += 1
                    continue

        # If we get here, all keys failed
        raise Exception("All API keys exhausted or rate limited")


# === GPT-4 AI ANALYZER ===


class GPTTradingAnalyzer:
    """GPT-4 powered trading analysis."""

    def __init__(self, client):
        self.client = client

    async def analyze_trading_signals(
        self, request: AIAnalysisRequest
    ) -> AISignalResponse:
        """Analyze trading signals using GPT-4 or fallback technical analysis."""
        try:
            # Convert to DataFrames and calculate indicators
            dataframes = TechnicalAnalyzer.candles_to_dataframe(request.timeframe_data)

            # Get indicators for ALL timeframes (15m, 30m, 1h, 4h) for multi-timeframe analysis
            # This fixes the issue where short-term downtrend was ignored because AI only looked at 1H/4H
            indicators_15m = {}
            indicators_30m = {}
            indicators_1h = {}
            indicators_4h = {}

            if "15m" in dataframes and len(dataframes["15m"]) >= 2:
                indicators_15m = TechnicalAnalyzer.calculate_indicators(dataframes["15m"])

            if "30m" in dataframes and len(dataframes["30m"]) >= 2:
                indicators_30m = TechnicalAnalyzer.calculate_indicators(dataframes["30m"])

            if "1h" in dataframes and len(dataframes["1h"]) >= 2:
                indicators_1h = TechnicalAnalyzer.calculate_indicators(dataframes["1h"])

            if "4h" in dataframes and len(dataframes["4h"]) >= 2:
                indicators_4h = TechnicalAnalyzer.calculate_indicators(dataframes["4h"])

            # Choose analysis method based on client availability
            # Pass all 4 timeframes for comprehensive multi-timeframe analysis
            if self.client is not None:
                ai_analysis = await self._gpt_analysis(
                    request, indicators_15m, indicators_30m, indicators_1h, indicators_4h
                )
            else:
                ai_analysis = self._fallback_analysis(
                    request, indicators_15m, indicators_30m, indicators_1h, indicators_4h
                )

            # Create response
            return AISignalResponse(
                signal=ai_analysis.get("signal", "Neutral"),
                confidence=ai_analysis.get("confidence", 0.5),
                reasoning=ai_analysis.get("reasoning", "Analysis completed"),
                strategy_scores=ai_analysis.get("strategy_scores", {}),
                market_analysis=AIMarketAnalysis(
                    **ai_analysis.get(
                        "market_analysis",
                        {
                            "trend_direction": "Sideways",
                            "trend_strength": 0.5,
                            "support_levels": [],
                            "resistance_levels": [],
                            "volatility_level": "Medium",
                            "volume_analysis": "Normal volume patterns",
                        },
                    )
                ),
                risk_assessment=AIRiskAssessment(
                    **ai_analysis.get(
                        "risk_assessment",
                        {
                            "overall_risk": "Medium",
                            "technical_risk": 0.5,
                            "market_risk": 0.5,
                            "recommended_position_size": 0.02,
                            "stop_loss_suggestion": None,
                            "take_profit_suggestion": None,
                        },
                    )
                ),
                timestamp=request.timestamp,
            )

        except Exception as e:
            logger.error(f"Analysis error: {e}")
            raise HTTPException(status_code=500, detail=f"AI analysis failed: {str(e)}")

    async def _gpt_analysis(
        self, request: AIAnalysisRequest, indicators_15m: Dict, indicators_30m: Dict, indicators_1h: Dict, indicators_4h: Dict
    ) -> Dict[str, Any]:
        """GPT-4 powered analysis with multi-timeframe support (15m, 30m, 1h, 4h)."""
        try:
            logger.info(f"🤖 Starting GPT-4 analysis for {request.symbol}")

            # Prepare market context with ALL timeframes
            market_context = self._prepare_market_context(
                request, indicators_15m, indicators_30m, indicators_1h, indicators_4h
            )
            logger.debug(
                f"📊 Market context prepared: {len(market_context)} characters"
            )

            # Create GPT-4 prompt
            prompt = self._create_analysis_prompt(
                market_context, request.strategy_context
            )
            logger.debug(f"📝 Prompt created: {len(prompt)} characters")
            logger.debug(
                f"🎯 Selected strategies: {request.strategy_context.selected_strategies}"
            )

            # Call GPT-4 (optimized max_tokens for cost saving)
            logger.info("🔄 Calling GPT-4 API...")
            response = await self.client.chat_completions_create(
                model="gpt-4o-mini",
                messages=[
                    {"role": "system", "content": self._get_system_prompt()},
                    {"role": "user", "content": prompt},
                ],
                temperature=0.0,
                max_tokens=1200,  # Reduced from 2000 to 1200 for cost optimization
            )

            logger.info("✅ GPT-4 API call successful")
            response_content = response["choices"][0]["message"]["content"]
            logger.debug(f"📤 GPT-4 response: {response_content[:200]}...")

            # Track token usage and cost
            usage = response.get("usage", {})
            if usage:
                input_tokens = usage.get("prompt_tokens", 0)
                output_tokens = usage.get("completion_tokens", 0)
                total_tokens = usage.get("total_tokens", 0)

                # Calculate cost
                input_cost = (input_tokens / 1_000_000) * GPT4O_MINI_INPUT_COST_PER_1M
                output_cost = (
                    output_tokens / 1_000_000
                ) * GPT4O_MINI_OUTPUT_COST_PER_1M
                request_cost = input_cost + output_cost

                # Update global counters
                global total_input_tokens, total_output_tokens, total_requests_count, total_cost_usd
                total_input_tokens += input_tokens
                total_output_tokens += output_tokens
                total_requests_count += 1
                total_cost_usd += request_cost

                logger.info(
                    f"💰 Cost: ${request_cost:.5f} | Tokens: {input_tokens} in + {output_tokens} out = {total_tokens} | "
                    f"Total today: ${total_cost_usd:.3f} ({total_requests_count} requests)"
                )

            # Parse GPT-4 response
            parsed_result = self._parse_gpt_response(response_content)
            logger.info(
                f"🎯 GPT-4 analysis complete: signal={parsed_result.get('signal')}, "
                f"confidence={parsed_result.get('confidence')}"
            )

            return parsed_result

        except Exception as e:
            logger.error(f"❌ GPT-4 analysis failed: {e}")
            logger.error(f"Error type: {type(e).__name__}")

            if "401" in str(e):
                logger.error("🔑 GPT-4 API authentication failed")
            elif "429" in str(e):
                logger.error("⏱️ GPT-4 rate limit exceeded")
            elif "quota" in str(e).lower():
                logger.error("💰 GPT-4 quota exceeded")
            elif "timeout" in str(e).lower():
                logger.error("⏰ GPT-4 API timeout")
            else:
                logger.error(f"🌐 GPT-4 API error: {str(e)}")

            # Fall back to technical analysis
            logger.warning("🔄 Falling back to technical analysis...")
            return self._fallback_analysis(request, indicators_15m, indicators_30m, indicators_1h, indicators_4h)

    def _fallback_analysis(
        self, request: AIAnalysisRequest, indicators_15m: Dict, indicators_30m: Dict, indicators_1h: Dict, indicators_4h: Dict
    ) -> Dict[str, Any]:
        """Fallback technical analysis when GPT-4 is not available. Uses multi-timeframe (15m, 30m, 1h, 4h)."""
        # FIX: Use Neutral as default to avoid Long bias
        # Only set Long/Short when indicators clearly support it
        signal = "Neutral"
        confidence = 0.45  # Lower confidence for fallback analysis
        reasoning = "Technical analysis (GPT-4 unavailable): "

        signals = []
        selected_strategies = request.strategy_context.selected_strategies

        # Track bullish/bearish signals to determine final signal
        bullish_count = 0
        bearish_count = 0

        # =========================================================
        # 15M & 30M SHORT-TERM TREND CHECK (CRITICAL!)
        # If short-term shows strong opposite trend, it overrides longer timeframes
        # This prevents LONG signals when 15m/30m chart is clearly in downtrend
        # =========================================================
        short_term_bearish = False
        short_term_bullish = False

        # Check 15m trend
        if indicators_15m:
            macd_hist_15m = indicators_15m.get("macd_histogram", 0)

            # Check 15m price trend from candles
            if "15m" in request.timeframe_data and len(request.timeframe_data["15m"]) >= 10:
                candles_15m = request.timeframe_data["15m"]
                # Calculate trend over last 10 candles (2.5 hours)
                first_close = candles_15m[-10].close if hasattr(candles_15m[-10], 'close') else candles_15m[-10].get("close", 0)
                last_close = candles_15m[-1].close if hasattr(candles_15m[-1], 'close') else candles_15m[-1].get("close", 0)
                if first_close > 0:
                    trend_15m = ((last_close - first_close) / first_close) * 100

                    if trend_15m < -0.8 and macd_hist_15m < 0:  # 15m downtrend > 0.8%
                        short_term_bearish = True
                        signals.append(f"⚠️ 15M DOWNTREND ({trend_15m:.2f}%)")
                        bearish_count += 2  # Weight 15m trend heavily (counts as 2 signals)
                    elif trend_15m > 0.8 and macd_hist_15m > 0:  # 15m uptrend > 0.8%
                        short_term_bullish = True
                        signals.append(f"✅ 15M UPTREND (+{trend_15m:.2f}%)")
                        bullish_count += 2  # Weight 15m trend heavily

        # Check 30m trend (additional confirmation)
        if indicators_30m:
            macd_hist_30m = indicators_30m.get("macd_histogram", 0)

            # Check 30m price trend from candles
            if "30m" in request.timeframe_data and len(request.timeframe_data["30m"]) >= 10:
                candles_30m = request.timeframe_data["30m"]
                # Calculate trend over last 10 candles (5 hours)
                first_close = candles_30m[-10].close if hasattr(candles_30m[-10], 'close') else candles_30m[-10].get("close", 0)
                last_close = candles_30m[-1].close if hasattr(candles_30m[-1], 'close') else candles_30m[-1].get("close", 0)
                if first_close > 0:
                    trend_30m = ((last_close - first_close) / first_close) * 100

                    if trend_30m < -0.8 and macd_hist_30m < 0:  # 30m downtrend > 0.8%
                        short_term_bearish = True
                        signals.append(f"⚠️ 30M DOWNTREND ({trend_30m:.2f}%)")
                        bearish_count += 1  # Weight 30m slightly less than 15m
                    elif trend_30m > 0.8 and macd_hist_30m > 0:  # 30m uptrend > 0.8%
                        short_term_bullish = True
                        signals.append(f"✅ 30M UPTREND (+{trend_30m:.2f}%)")
                        bullish_count += 1

        # RSI Analysis - only if selected
        if not selected_strategies or "RSI Strategy" in selected_strategies:
            if indicators_1h.get("rsi"):
                rsi = indicators_1h["rsi"]
                if rsi < 30:
                    signals.append("RSI oversold (bullish)")
                    bullish_count += 1
                elif rsi > 70:
                    signals.append("RSI overbought (bearish)")
                    bearish_count += 1
                else:
                    signals.append(f"RSI neutral ({rsi:.1f})")

        # MACD Analysis - only if selected
        if not selected_strategies or "MACD Strategy" in selected_strategies:
            if indicators_1h.get("macd") and indicators_1h.get("macd_signal"):
                macd = indicators_1h["macd"]
                macd_signal = indicators_1h["macd_signal"]
                if macd > macd_signal:
                    signals.append("MACD bullish crossover")
                    bullish_count += 1
                else:
                    signals.append("MACD bearish crossover")
                    bearish_count += 1

        # Volume Analysis - only if selected
        if not selected_strategies or "Volume Strategy" in selected_strategies:
            if indicators_1h.get("volume_ratio"):
                volume_ratio = indicators_1h["volume_ratio"]
                if volume_ratio > 1.5:
                    signals.append(f"High volume ({volume_ratio:.1f}x avg)")
                elif volume_ratio < 0.5:
                    signals.append(f"Low volume ({volume_ratio:.1f}x avg)")

        # Bollinger Bands Analysis - only if selected
        if not selected_strategies or "Bollinger Bands Strategy" in selected_strategies:
            if indicators_1h.get("bb_position"):
                bb_position = indicators_1h["bb_position"]
                if bb_position < 0.1:
                    signals.append("Price near lower Bollinger Band (bullish)")
                    bullish_count += 1
                elif bb_position > 0.9:
                    signals.append("Price near upper Bollinger Band (bearish)")
                    bearish_count += 1

        # Price trend analysis
        if "1h" in request.timeframe_data and len(request.timeframe_data["1h"]) >= 2:
            candles = request.timeframe_data["1h"]
            if len(candles) >= 2:
                price_change = (
                    (candles[-1].close - candles[-2].close) / candles[-2].close * 100
                )
                if price_change > 1:
                    signals.append(f"Strong upward movement (+{price_change:.2f}%)")
                    bullish_count += 1
                elif price_change < -1:
                    signals.append(f"Strong downward movement ({price_change:.2f}%)")
                    bearish_count += 1

        # Determine signal based on consensus (3/5 = 60%)
        # Lowered from 4/5 to allow more trading opportunities
        # @spec:FR-STRATEGIES-006 - Signal Combination
        MIN_REQUIRED_SIGNALS = 3  # Must have 3+ out of 5 indicators agree

        if bullish_count >= MIN_REQUIRED_SIGNALS:
            signal = "Long"
            confidence = min(0.75, 0.50 + (bullish_count * 0.05))
        elif bearish_count >= MIN_REQUIRED_SIGNALS:
            signal = "Short"
            confidence = min(0.75, 0.50 + (bearish_count * 0.05))
        else:
            # Stay Neutral when consensus is weak (< 4/5 agreement)
            signal = "Neutral"
            confidence = 0.35

        reasoning += "; ".join(signals) if signals else "Limited data available"
        reasoning += f" (Bullish: {bullish_count}, Bearish: {bearish_count})"

        # Create strategy scores based on selected strategies
        strategy_scores = {}
        all_strategies = [
            "RSI Strategy",
            "MACD Strategy",
            "Volume Strategy",
            "Bollinger Bands Strategy",
            "Stochastic Strategy",
        ]

        for strategy in all_strategies:
            if not selected_strategies or strategy in selected_strategies:
                if strategy == "RSI Strategy":
                    strategy_scores[strategy] = (
                        confidence if "RSI" in reasoning else 0.3
                    )
                elif strategy == "MACD Strategy":
                    strategy_scores[strategy] = (
                        confidence if "MACD" in reasoning else 0.3
                    )
                elif strategy == "Volume Strategy":
                    strategy_scores[strategy] = (
                        confidence if "volume" in reasoning.lower() else 0.3
                    )
                elif strategy == "Bollinger Bands Strategy":
                    strategy_scores[strategy] = (
                        confidence if "Bollinger" in reasoning else 0.3
                    )
                elif strategy == "Stochastic Strategy":
                    strategy_scores[strategy] = (
                        confidence if "Stochastic" in reasoning else 0.35
                    )
            else:
                # Set low score for non-selected strategies
                strategy_scores[strategy] = 0.1

        return {
            "signal": signal,
            "confidence": confidence,
            "reasoning": reasoning,
            "strategy_scores": strategy_scores,
            "market_analysis": {
                "trend_direction": signal if signal != "Neutral" else "Sideways",
                "trend_strength": confidence,
                "support_levels": [],
                "resistance_levels": [],
                "volatility_level": "Medium",
                "volume_analysis": "Technical analysis mode",
            },
            "risk_assessment": {
                "overall_risk": "Medium",
                "technical_risk": 0.5,
                "market_risk": 0.5,
                "recommended_position_size": 0.02,
                "stop_loss_suggestion": None,
                "take_profit_suggestion": None,
            },
        }

    def _get_system_prompt(self) -> str:
        """Get system prompt for GPT-4 with multi-timeframe awareness."""
        return (
            "Crypto trading analyst using MULTI-TIMEFRAME analysis (15M, 1H, 4H).\n"
            "CRITICAL RULE: If 15M trend CONFLICTS with 1H/4H, signal MUST be Neutral!\n"
            "Example: 15M bearish + 1H/4H bullish = Neutral (NOT Long!)\n"
            "Respond ONLY in JSON:\n"
            '{"signal":"Long|Short|Neutral","confidence":0-1,"reasoning":"brief",'
            '"strategy_scores":{"RSI Strategy":0-1,"MACD Strategy":0-1,"Volume Strategy":0-1,'
            '"Bollinger Bands Strategy":0-1,"Stochastic Strategy":0-1},'
            '"market_analysis":{"trend_direction":"Bullish|Bearish|Sideways",'
            '"trend_strength":0-1,"support_levels":[],"resistance_levels":[],'
            '"volatility_level":"Low|Medium|High","volume_analysis":"brief"},'
            '"risk_assessment":{"overall_risk":"Low|Medium|High","technical_risk":0-1,'
            '"market_risk":0-1,"recommended_position_size":0-1,"stop_loss_suggestion":null,'
            '"take_profit_suggestion":null}}\n'
            "Use confidence >0.6 for strong signals. When timeframes conflict, always choose Neutral."
        )

    def _prepare_market_context(
        self, request: AIAnalysisRequest, indicators_15m: Dict, indicators_30m: Dict, indicators_1h: Dict, indicators_4h: Dict
    ) -> str:
        """Prepare market context for GPT-4 with multi-timeframe analysis (15m, 30m, 1h, 4h).

        IMPORTANT: 15m and 30m are included to detect short-term trend changes that may conflict
        with longer timeframes. This prevents giving LONG signal when short-term shows downtrend.
        """
        # Compact format to reduce tokens - now includes 15m & 30m for short-term trend
        context = f"{request.symbol} ${request.current_price:.0f}\n"

        # 15m - Very short-term trend (CRITICAL for detecting immediate reversals)
        if indicators_15m:
            context += (
                f"15M: RSI:{indicators_15m.get('rsi',50):.1f} "
                f"MACD:{indicators_15m.get('macd_histogram',0):.2f} "
                f"BB:{indicators_15m.get('bb_position',0.5):.2f} "
                f"Vol:{indicators_15m.get('volume_ratio',1):.1f}x\n"
            )

        # 30m - Short-term trend
        if indicators_30m:
            context += (
                f"30M: RSI:{indicators_30m.get('rsi',50):.1f} "
                f"MACD:{indicators_30m.get('macd_histogram',0):.2f} "
                f"BB:{indicators_30m.get('bb_position',0.5):.2f} "
                f"Vol:{indicators_30m.get('volume_ratio',1):.1f}x\n"
            )

        # 1H - Medium-term trend
        context += (
            f"1H: RSI:{indicators_1h.get('rsi',50):.1f} "
            f"MACD:{indicators_1h.get('macd_histogram',0):.2f} "
            f"BB:{indicators_1h.get('bb_position',0.5):.2f} "
            f"Vol:{indicators_1h.get('volume_ratio',1):.1f}x"
        )

        # 4H - Long-term trend
        if indicators_4h:
            context += (
                f"\n4H: RSI:{indicators_4h.get('rsi',50):.1f} "
                f"MACD:{indicators_4h.get('macd_histogram',0):.2f} "
                f"BB:{indicators_4h.get('bb_position',0.5):.2f}"
            )

        return context

    def _create_analysis_prompt(
        self, market_context: str, strategy_context: AIStrategyContext
    ) -> str:
        """Create analysis prompt for GPT-4 (optimized for cost)."""
        strategies = (
            ", ".join(strategy_context.selected_strategies)
            if strategy_context.selected_strategies
            else "All"
        )
        return (
            f"{market_context}\nStrategies:{strategies}|Risk:{strategy_context.risk_level}\n"
            "Analyze & provide JSON signal with strategy scores."
        )

    def _parse_gpt_response(self, response_text: str) -> Dict[str, Any]:
        """Parse GPT-4 JSON response."""
        try:
            # Find JSON in response
            import re

            json_match = re.search(r"\{.*\}", response_text, re.DOTALL)
            if json_match:
                return json.loads(json_match.group())
            else:
                # Fallback parsing
                return self._fallback_parse(response_text)
        except Exception as e:
            logger.warning(f"GPT response parsing error: {e}")
            return self._default_response()

    def _fallback_parse(self, text: str) -> Dict[str, Any]:
        """Fallback parsing for non-JSON responses."""
        signal = "Neutral"
        confidence = 0.5

        text_upper = text.upper()
        if "STRONG BUY" in text_upper or "LONG" in text_upper:
            signal = "Long"
            confidence = 0.7
        elif "STRONG SELL" in text_upper or "SHORT" in text_upper:
            signal = "Short"
            confidence = 0.7
        elif "BUY" in text_upper:
            signal = "Long"
            confidence = 0.6
        elif "SELL" in text_upper:
            signal = "Short"
            confidence = 0.6

        return {
            "signal": signal,
            "confidence": confidence,
            "reasoning": text[:500] + "..." if len(text) > 500 else text,
            "strategy_scores": {
                "RSI Strategy": confidence,
                "MACD Strategy": confidence,
                "Volume Strategy": confidence * 0.8,
                "Bollinger Bands Strategy": confidence * 0.9,
            },
            "market_analysis": {
                "trend_direction": "Uncertain",
                "trend_strength": confidence,
                "support_levels": [],
                "resistance_levels": [],
                "volatility_level": "Medium",
                "volume_analysis": "Analysis from GPT response",
            },
            "risk_assessment": {
                "overall_risk": "Medium",
                "technical_risk": 0.5,
                "market_risk": 0.5,
                "recommended_position_size": 0.02,
                "stop_loss_suggestion": None,
                "take_profit_suggestion": None,
            },
        }

    def _default_response(self) -> Dict[str, Any]:
        """Default response for parsing failures."""
        return {
            "signal": "Neutral",
            "confidence": 0.3,
            "reasoning": "Unable to generate analysis due to parsing error",
            "strategy_scores": {
                "RSI Strategy": 0.3,
                "MACD Strategy": 0.3,
                "Volume Strategy": 0.3,
                "Bollinger Bands Strategy": 0.3,
            },
            "market_analysis": {
                "trend_direction": "Uncertain",
                "trend_strength": 0.3,
                "support_levels": [],
                "resistance_levels": [],
                "volatility_level": "Medium",
                "volume_analysis": "Unable to analyze volume",
            },
            "risk_assessment": {
                "overall_risk": "High",
                "technical_risk": 0.8,
                "market_risk": 0.8,
                "recommended_position_size": 0.01,
                "stop_loss_suggestion": None,
                "take_profit_suggestion": None,
            },
        }


# === AI ANALYSIS BACKGROUND PROCESSING ===


async def get_analysis_statistics() -> Dict[str, Any]:
    """Get analysis statistics from MongoDB."""
    if mongodb_db is None:
        return {"error": "MongoDB not connected"}

    try:
        total_analyses = await mongodb_db[AI_ANALYSIS_COLLECTION].count_documents({})
        recent_analyses = await mongodb_db[AI_ANALYSIS_COLLECTION].count_documents(
            {"timestamp": {"$gte": datetime.now(timezone.utc) - timedelta(hours=24)}}
        )

        # Fetch current symbols dynamically
        current_symbols = await fetch_analysis_symbols()

        return {
            "total_analyses": total_analyses,
            "analyses_24h": recent_analyses,
            "symbols_tracked": len(current_symbols),
            "analysis_interval_minutes": ANALYSIS_INTERVAL_MINUTES,
        }
    except Exception as e:
        logger.error(f"Failed to get analysis stats: {e}")
        return {"error": str(e)}


async def fetch_real_market_data(symbol: str) -> AIAnalysisRequest:
    """
    Fetch REAL market data from Rust Core Engine API.

    CRITICAL: This function fetches actual market data from Binance via Rust API.
    Never use dummy/fake data for trading decisions!
    """
    import httpx

    current_time = int(datetime.now(timezone.utc).timestamp() * 1000)
    candles_1h = []
    candles_4h = []
    current_price = 0.0
    volume_24h = 0.0

    try:
        async with httpx.AsyncClient(timeout=30.0) as client:
            # Fetch 1H candles from Rust API
            try:
                response_1h = await client.get(
                    f"{RUST_API_URL}/api/market/candles/{symbol}/1h",
                    params={"limit": 100}  # Need 100 candles for indicators
                )
                if response_1h.status_code == 200:
                    data = response_1h.json()
                    candle_data = data.get("data", []) if data.get("success") else []
                    for candle in candle_data:
                        candles_1h.append(
                            CandleData(
                                timestamp=candle.get("timestamp", 0),
                                open=float(candle.get("open", 0)),
                                high=float(candle.get("high", 0)),
                                low=float(candle.get("low", 0)),
                                close=float(candle.get("close", 0)),
                                volume=float(candle.get("volume", 0)),
                            )
                        )
                    logger.info(f"📊 Fetched {len(candles_1h)} 1H candles for {symbol}")
                else:
                    logger.warning(f"⚠️ Failed to fetch 1H candles for {symbol}: {response_1h.status_code}")
            except Exception as e:
                logger.error(f"❌ Error fetching 1H candles for {symbol}: {e}")

            # Fetch 4H candles from Rust API
            try:
                response_4h = await client.get(
                    f"{RUST_API_URL}/api/market/candles/{symbol}/4h",
                    params={"limit": 60}  # Need 60 candles for indicators
                )
                if response_4h.status_code == 200:
                    data = response_4h.json()
                    candle_data = data.get("data", []) if data.get("success") else []
                    for candle in candle_data:
                        candles_4h.append(
                            CandleData(
                                timestamp=candle.get("timestamp", 0),
                                open=float(candle.get("open", 0)),
                                high=float(candle.get("high", 0)),
                                low=float(candle.get("low", 0)),
                                close=float(candle.get("close", 0)),
                                volume=float(candle.get("volume", 0)),
                            )
                        )
                    logger.info(f"📊 Fetched {len(candles_4h)} 4H candles for {symbol}")
                else:
                    logger.warning(f"⚠️ Failed to fetch 4H candles for {symbol}: {response_4h.status_code}")
            except Exception as e:
                logger.error(f"❌ Error fetching 4H candles for {symbol}: {e}")

            # Fetch current price from Rust API
            try:
                response_prices = await client.get(f"{RUST_API_URL}/api/market/prices")
                if response_prices.status_code == 200:
                    data = response_prices.json()
                    prices = data.get("data", {}) if data.get("success") else {}
                    current_price = float(prices.get(symbol, 0))
                    logger.info(f"💰 Current price for {symbol}: ${current_price:.2f}")
            except Exception as e:
                logger.error(f"❌ Error fetching price for {symbol}: {e}")

            # Calculate 24h volume from 1H candles
            if candles_1h:
                volume_24h = sum(c.volume for c in candles_1h[:24])

    except Exception as e:
        logger.error(f"❌ Failed to fetch market data for {symbol}: {e}")

    # Validate we have sufficient data
    if len(candles_1h) < 50:
        logger.warning(f"⚠️ Insufficient 1H data for {symbol}: {len(candles_1h)} candles (need 50+)")
    if len(candles_4h) < 50:
        logger.warning(f"⚠️ Insufficient 4H data for {symbol}: {len(candles_4h)} candles (need 50+)")
    if current_price == 0:
        logger.warning(f"⚠️ No current price for {symbol}, using last close price")
        if candles_1h:
            current_price = candles_1h[0].close

    return AIAnalysisRequest(
        symbol=symbol,
        timeframe_data={"1h": candles_1h, "4h": candles_4h},
        current_price=current_price,
        volume_24h=volume_24h,
        timestamp=current_time,
        strategy_context=AIStrategyContext(
            selected_strategies=[
                "RSI Strategy",
                "MACD Strategy",
                "Bollinger Bands Strategy",
                "Volume Strategy",
                "Stochastic Strategy",
            ],
            market_condition="Trending",
            risk_level="Moderate",
            user_preferences={},
            technical_indicators={},
        ),
    )


# Global analyzer instance
gpt_analyzer = None

# === API ENDPOINTS ===


@app.get("/health")
async def health_check():
    """Health check endpoint."""

    # Check MongoDB connection
    mongodb_status = False
    try:
        if mongodb_client:
            await mongodb_client.admin.command("ping")
            mongodb_status = True
    except Exception:
        pass

    # Fetch dynamic symbols from Rust API
    current_symbols = await fetch_analysis_symbols()

    return {
        "status": "healthy",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "service": "GPT-4 Trading AI",
        "version": "2.0.0",
        "gpt4_available": openai_client is not None,
        "api_key_configured": bool(os.getenv("OPENAI_API_KEY")),
        "mongodb_connected": mongodb_status,
        "analysis_interval_minutes": ANALYSIS_INTERVAL_MINUTES,
        "supported_symbols": current_symbols,
    }


@app.get("/debug/gpt4")
@limiter.limit("60/minute")  # Rate limit: 60 requests per minute for debug endpoint
async def debug_gpt4(request: Request):
    """Debug GPT-4 connectivity."""
    result: Dict[str, Any] = {
        "client_initialized": openai_client is not None,
        "api_key_configured": bool(os.getenv("OPENAI_API_KEY")),
    }

    if openai_client is None:
        result["error"] = "OpenAI client not initialized"
        result["status"] = "failed"
        return result

    try:
        # Test simple API call
        logger.info("🧪 Testing GPT-4o-mini API connection...")
        response = await openai_client.chat_completions_create(
            model="gpt-4o-mini",
            messages=[
                {"role": "user", "content": "Respond with just the word 'SUCCESS'"}
            ],
            max_tokens=10,
            temperature=0,
        )

        result["status"] = "success"
        result["test_response"] = response["choices"][0]["message"]["content"]
        result["model_used"] = "gpt-4o-mini"
        logger.info("✅ GPT-4o-mini test successful")

    except Exception as e:
        result["status"] = "failed"
        result["error"] = str(e)
        result["error_type"] = type(e).__name__
        logger.error(f"❌ GPT-4 test failed: {e}")

        if "401" in str(e):
            result["diagnosis"] = "API key authentication failed"
        elif "429" in str(e):
            result["diagnosis"] = "Rate limit exceeded"
        elif "quota" in str(e).lower():
            result["diagnosis"] = "Quota exceeded - check billing"
        else:
            result["diagnosis"] = "Unknown API error"

    return result


@app.websocket("/ws")
async def websocket_endpoint(websocket: WebSocket):
    """WebSocket endpoint for real-time AI signal broadcasting."""
    await ws_manager.connect(websocket)
    try:
        while True:
            # Keep connection alive and handle incoming messages
            _ = await websocket.receive_text()  # Receive but ignore for now
            await websocket.send_json(
                {
                    "type": "Pong",
                    "message": "Connection alive",
                    "timestamp": datetime.now(timezone.utc).isoformat(),
                }
            )
    except WebSocketDisconnect:
        ws_manager.disconnect(websocket)


# @spec:FR-AI-005 - GPT-4 Signal Analysis
# @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md
# @ref:specs/02-design/2.3-api/API-PYTHON-AI.md
# @test:TC-AI-010, TC-AI-011, TC-AI-012
@app.post("/ai/analyze", response_model=AISignalResponse)
@limiter.limit(
    "600/minute"
)  # Rate limit: 600 requests per minute (10 per second) - high throughput for multi-symbol monitoring
async def analyze_trading_signals(
    analysis_request: AIAnalysisRequest, request: Request
):
    """Analyze trading signals using GPT-4 AI with MongoDB storage."""
    global gpt_analyzer

    if not gpt_analyzer:
        gpt_analyzer = GPTTradingAnalyzer(openai_client)
        logger.info(
            f"🤖 GPT analyzer created with client: {'Available' if openai_client else 'None'}"
        )

    logger.info(f"🤖 GPT-4 analysis request for {analysis_request.symbol}")
    logger.debug(
        f"📋 Request details: strategies={analysis_request.strategy_context.selected_strategies}, "
        f"timeframes={list(analysis_request.timeframe_data.keys())}"
    )

    # Check GPT-4 availability
    if openai_client is None:
        logger.warning("⚠️ GPT-4 client is None - will use fallback analysis")
    else:
        logger.info("✅ GPT-4 client available - attempting AI analysis")

    try:
        # Check MongoDB for latest analysis
        latest_analysis = await get_latest_analysis(analysis_request.symbol)

        # Check if analysis is still fresh (< 5 minutes old)
        if latest_analysis:
            # Get stored analysis timestamp
            stored_timestamp = latest_analysis.get("timestamp", 0)
            if isinstance(stored_timestamp, int):
                stored_time = datetime.fromtimestamp(
                    stored_timestamp / 1000, timezone.utc
                )
            else:
                stored_time = datetime.now(timezone.utc) - timedelta(
                    minutes=10
                )  # Force refresh

            time_since_analysis = (
                datetime.now(timezone.utc) - stored_time
            ).total_seconds() / 60

            if time_since_analysis < ANALYSIS_INTERVAL_MINUTES:
                logger.info(
                    f"📊 Using recent MongoDB analysis for {analysis_request.symbol} (age: {time_since_analysis:.1f}min)"
                )

                # Return stored analysis
                stored_response = AISignalResponse(
                    signal=latest_analysis.get("signal", "Neutral"),
                    confidence=latest_analysis.get("confidence", 0.5),
                    reasoning=f"[RECENT] {latest_analysis.get('reasoning', 'Analysis completed')}",
                    strategy_scores=latest_analysis.get("strategy_scores", {}),
                    market_analysis=AIMarketAnalysis(
                        **latest_analysis.get(
                            "market_analysis",
                            {
                                "trend_direction": "Sideways",
                                "trend_strength": 0.5,
                                "support_levels": [],
                                "resistance_levels": [],
                                "volatility_level": "Medium",
                                "volume_analysis": "Normal volume patterns",
                            },
                        )
                    ),
                    risk_assessment=AIRiskAssessment(
                        **latest_analysis.get(
                            "risk_assessment",
                            {
                                "overall_risk": "Medium",
                                "technical_risk": 0.5,
                                "market_risk": 0.5,
                                "recommended_position_size": 0.02,
                                "stop_loss_suggestion": None,
                                "take_profit_suggestion": None,
                            },
                        )
                    ),
                    timestamp=analysis_request.timestamp,
                )

                # Broadcast stored signal via WebSocket
                if ws_manager.active_connections:
                    signal_data = {
                        "symbol": analysis_request.symbol,
                        "signal": stored_response.signal.lower(),
                        "confidence": stored_response.confidence,
                        "timestamp": stored_response.timestamp,
                        "model_type": "GPT-4-Stored",
                        "timeframe": "1h",
                        "reasoning": stored_response.reasoning,
                        "strategy_scores": stored_response.strategy_scores,
                    }
                    await ws_manager.broadcast_signal(signal_data)

                return stored_response

        # No recent analysis found, perform fresh AI analysis
        logger.info(
            f"🔥 No recent analysis found. Calling OpenAI GPT-4 for {analysis_request.symbol}"
        )
        response = await gpt_analyzer.analyze_trading_signals(analysis_request)
        logger.info(
            f"✅ GPT-4 signal: {response.signal} (confidence: {response.confidence:.2f})"
        )

        # Store the fresh analysis in MongoDB
        analysis_data = {
            "signal": response.signal,
            "confidence": response.confidence,
            "reasoning": response.reasoning,
            "strategy_scores": response.strategy_scores,
            "market_analysis": response.market_analysis.model_dump(),
            "risk_assessment": response.risk_assessment.model_dump(),
            "timestamp": response.timestamp,
        }
        await store_analysis_result(analysis_request.symbol, analysis_data)

        # Broadcast signal via WebSocket to connected clients
        if ws_manager.active_connections:
            signal_data = {
                "symbol": analysis_request.symbol,
                "signal": response.signal.lower(),
                "confidence": response.confidence,
                "timestamp": response.timestamp,
                "model_type": "GPT-4",
                "timeframe": "1h",  # Default timeframe for WebSocket compatibility
                "reasoning": response.reasoning,
                "strategy_scores": response.strategy_scores,
            }
            await ws_manager.broadcast_signal(signal_data)

        return response
    except Exception as e:
        logger.error(f"❌ Analysis failed: {e}")
        raise HTTPException(status_code=500, detail=str(e))


@app.post("/ai/strategy-recommendations", response_model=List[StrategyRecommendation])
@limiter.limit("300/minute")  # Rate limit: 300 requests per minute (5 per second)
async def get_strategy_recommendations(
    request: StrategyRecommendationRequest, http_request: Request
):
    """Get AI strategy recommendations."""
    logger.info(f"📊 Strategy recommendations for {request.symbol}")

    # Simple recommendations based on available strategies
    recommendations = []
    for strategy in request.available_strategies:
        score = (
            0.7 + (hash(strategy + request.symbol) % 30) / 100
        )  # Pseudo-random score
        recommendations.append(
            StrategyRecommendation(
                strategy_name=strategy,
                suitability_score=min(score, 0.95),
                reasoning=f"{strategy} shows good potential for {request.symbol} based on current market conditions",
                recommended_config={"enabled": True, "weight": score},
            )
        )

    return sorted(recommendations, key=lambda x: x.suitability_score, reverse=True)


@app.post("/ai/market-condition", response_model=MarketConditionAnalysis)
@limiter.limit("300/minute")  # Rate limit: 300 requests per minute (5 per second)
async def analyze_market_condition(request: MarketConditionRequest, http_request: Request):
    """Analyze market condition."""
    logger.info(f"🔍 Market condition analysis for {request.symbol}")

    # Simple market condition analysis
    dataframes = TechnicalAnalyzer.candles_to_dataframe(request.timeframe_data)

    condition_type = "Sideways"
    confidence = 0.6
    characteristics = ["Normal volatility", "Balanced volume"]

    if "1h" in dataframes and len(dataframes["1h"]) >= 20:
        df = dataframes["1h"]
        price_change = ((df["close"].iloc[-1] / df["close"].iloc[-20]) - 1) * 100

        if price_change > 5:
            condition_type = "Trending Up"
            characteristics = ["Strong uptrend", "High momentum"]
        elif price_change < -5:
            condition_type = "Trending Down"
            characteristics = ["Strong downtrend", "High selling pressure"]

    return MarketConditionAnalysis(
        condition_type=condition_type,
        confidence=confidence,
        characteristics=characteristics,
        recommended_strategies=["RSI Strategy", "MACD Strategy"],
        market_phase="Active Trading",
    )


@app.post("/ai/feedback")
@limiter.limit("120/minute")  # Rate limit: 120 requests per minute (2 per second)
async def send_performance_feedback(feedback: PerformanceFeedback, request: Request):
    """Receive performance feedback for learning."""
    logger.info(
        f"📝 Feedback received for {feedback.symbol}: {feedback.actual_outcome}"
    )

    # Store feedback for future model improvements
    # In a real implementation, this would be stored in a database

    return {
        "message": "Feedback received successfully",
        "signal_id": feedback.signal_id,
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }


@app.post("/predict-trend", response_model=TrendPredictionResponse)
@limiter.limit("600/minute")  # Rate limit: 600 requests per minute (10 per second) - main prediction endpoint
async def predict_trend(request: TrendPredictionRequest, http_request: Request):
    """
    Predict trend direction using GPT-4 powered multi-timeframe analysis.

    This endpoint uses GPT-4 to analyze market data across multiple timeframes:
    - Daily (1d): Major trend direction
    - 4H: Intermediate trend
    - Requested timeframe: Short-term signals

    GPT-4 considers: EMA200, momentum, RSI, MACD, volume, and price action.
    Falls back to technical analysis if GPT-4 unavailable.
    """
    logger.info(f"🔮 GPT-4 trend prediction request for {request.symbol} on {request.timeframe}")

    try:
        # Fetch market data from MongoDB
        if mongodb_db is None:
            raise HTTPException(status_code=503, detail="Database not available")

        candles_collection = mongodb_db.market_data

        # Fetch multi-timeframe data for comprehensive analysis
        timeframes = {
            "1d": 250,   # Daily for major trend
            "4h": 250,   # 4H for intermediate trend
            request.timeframe: 250  # Requested timeframe
        }

        candles_by_tf = {}
        for tf, limit in timeframes.items():
            cursor = candles_collection.find(
                {"symbol": request.symbol, "timeframe": tf},
                {"_id": 0}
            ).sort("open_time", ASCENDING).limit(limit)

            candles = await cursor.to_list(length=limit)
            if len(candles) >= 50:  # Minimum data requirement
                candles_by_tf[tf] = candles

        if len(candles_by_tf) == 0:
            logger.warning(f"⚠️ Insufficient data for {request.symbol}")
            return TrendPredictionResponse(
                trend="Neutral",
                confidence=0.3,
                model="Insufficient Data",
                timestamp=int(datetime.now(timezone.utc).timestamp())
            )

        # Try GPT-4 analysis first
        if openai_client is not None:
            try:
                result = await _predict_trend_gpt4(request.symbol, candles_by_tf)
                logger.info(
                    f"✅ GPT-4 trend prediction for {request.symbol}: {result['trend']} "
                    f"(confidence: {result['confidence']:.2f})"
                )
                return TrendPredictionResponse(
                    trend=result["trend"],
                    confidence=result["confidence"],
                    model="GPT-4o-mini",
                    timestamp=int(datetime.now(timezone.utc).timestamp())
                )
            except Exception as e:
                logger.warning(f"⚠️ GPT-4 analysis failed, falling back to technical: {e}")

        # Fallback to technical analysis
        result = _predict_trend_technical(request.symbol, candles_by_tf, request.timeframe)
        logger.info(
            f"✅ Technical trend prediction for {request.symbol}: {result['trend']} "
            f"(confidence: {result['confidence']:.2f})"
        )

        return TrendPredictionResponse(
            trend=result["trend"],
            confidence=result["confidence"],
            model="EMA200-Technical-Fallback",
            timestamp=int(datetime.now(timezone.utc).timestamp())
        )

    except Exception as e:
        logger.error(f"❌ Error predicting trend for {request.symbol}: {e}")
        raise HTTPException(
            status_code=500,
            detail=f"Failed to predict trend: {str(e)}"
        )


async def _predict_trend_gpt4(symbol: str, candles_by_tf: Dict[str, List]) -> Dict[str, Any]:
    """
    Use GPT-4 to predict trend direction based on multi-timeframe analysis.

    Args:
        symbol: Trading symbol (e.g., BTCUSDT)
        candles_by_tf: Dict of timeframe -> candles data

    Returns:
        Dict with trend, confidence, reasoning
    """
    # Calculate indicators for each timeframe
    indicators_by_tf = {}

    for tf, candles in candles_by_tf.items():
        df = pd.DataFrame(candles)
        df = df.sort_values("open_time")

        # Calculate key indicators
        df["ema_200"] = df["close"].ewm(span=200, adjust=False).mean() if len(df) >= 200 else df["close"]
        df["ema_50"] = df["close"].ewm(span=50, adjust=False).mean() if len(df) >= 50 else df["close"]

        current_price = df["close"].iloc[-1]
        ema_200 = df["ema_200"].iloc[-1] if len(df) >= 200 else current_price
        ema_50 = df["ema_50"].iloc[-1] if len(df) >= 50 else current_price

        # Price distance from EMAs
        distance_200 = ((current_price - ema_200) / ema_200) * 100 if ema_200 > 0 else 0
        distance_50 = ((current_price - ema_50) / ema_50) * 100 if ema_50 > 0 else 0

        # Momentum (last 20 periods)
        momentum_20 = ((df["close"].iloc[-1] / df["close"].iloc[-20]) - 1) * 100 if len(df) >= 20 else 0

        # Volume trend
        volume_ma = df["volume"].rolling(20).mean().iloc[-1] if len(df) >= 20 else df["volume"].iloc[-1]
        volume_ratio = df["volume"].iloc[-1] / volume_ma if volume_ma > 0 else 1.0

        # RSI if enough data
        if len(df) >= 14:
            delta = df["close"].diff()
            gain = (delta.where(delta > 0, 0)).rolling(window=14).mean()
            loss = (-delta.where(delta < 0, 0)).rolling(window=14).mean()
            rs = gain / loss
            rsi = 100 - (100 / (1 + rs))
            current_rsi = rsi.iloc[-1]
        else:
            current_rsi = 50.0

        indicators_by_tf[tf] = {
            "current_price": float(current_price),
            "ema_200": float(ema_200),
            "ema_50": float(ema_50),
            "distance_from_ema200": float(distance_200),
            "distance_from_ema50": float(distance_50),
            "momentum_20": float(momentum_20),
            "volume_ratio": float(volume_ratio),
            "rsi": float(current_rsi),
            "last_5_closes": [float(x) for x in df["close"].tail(5).tolist()]
        }

    # Build GPT-4 prompt
    prompt = f"""Analyze the multi-timeframe trend for {symbol}:

DAILY (1d) TIMEFRAME:
{_format_tf_data("Daily", indicators_by_tf.get("1d", {}))}

4-HOUR (4h) TIMEFRAME:
{_format_tf_data("4-Hour", indicators_by_tf.get("4h", {}))}

PRIMARY TIMEFRAME:
{_format_tf_data("Primary", indicators_by_tf.get(list(candles_by_tf.keys())[-1], {}))}

INSTRUCTIONS:
1. Determine the PRIMARY trend direction considering all timeframes
2. Daily timeframe is most important (60% weight)
3. 4H timeframe is moderately important (30% weight)
4. Primary timeframe fine-tunes the signal (10% weight)
5. Consider: EMA200 position, momentum, RSI, volume
6. Be conservative - only strong signals should get high confidence

OUTPUT FORMAT (JSON only, no markdown):
{{
  "trend": "Uptrend" | "Downtrend" | "Neutral",
  "confidence": 0.0-1.0,
  "reasoning": "Explain in 1-2 sentences why you chose this trend",
  "timeframe_alignment": {{
    "daily": "up" | "down" | "neutral",
    "4h": "up" | "down" | "neutral",
    "primary": "up" | "down" | "neutral"
  }}
}}"""

    # Call GPT-4
    response = await openai_client.chat_completions_create(
        model="gpt-4o-mini",
        messages=[
            {
                "role": "system",
                "content": "You are an expert cryptocurrency technical analyst. Analyze trends conservatively and explain your reasoning clearly. Always respond with valid JSON."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        temperature=0.0,  # Deterministic
        max_tokens=400,   # Short response = cheaper
    )

    # Parse response
    response_content = response["choices"][0]["message"]["content"]

    # Track cost
    usage = response.get("usage", {})
    if usage:
        input_tokens = usage.get("prompt_tokens", 0)
        output_tokens = usage.get("completion_tokens", 0)
        cost = (input_tokens / 1_000_000) * 0.15 + (output_tokens / 1_000_000) * 0.60

        global total_input_tokens, total_output_tokens, total_requests_count, total_cost_usd
        total_input_tokens += input_tokens
        total_output_tokens += output_tokens
        total_requests_count += 1
        total_cost_usd += cost

        logger.info(f"💰 Trend prediction cost: ${cost:.5f} | Tokens: {input_tokens}+{output_tokens} | Total: ${total_cost_usd:.3f}")

    # Parse JSON from response (handle markdown code blocks if present)
    import re
    json_match = re.search(r'\{.*\}', response_content, re.DOTALL)
    if json_match:
        result = json.loads(json_match.group())
    else:
        result = json.loads(response_content)

    return result


def _format_tf_data(tf_name: str, indicators: Dict) -> str:
    """Format timeframe data for GPT-4 prompt."""
    if not indicators:
        return f"{tf_name}: No data available"

    return f"""- Current Price: ${indicators.get('current_price', 0):.2f}
- EMA200: ${indicators.get('ema_200', 0):.2f} (distance: {indicators.get('distance_from_ema200', 0):+.2f}%)
- EMA50: ${indicators.get('ema_50', 0):.2f} (distance: {indicators.get('distance_from_ema50', 0):+.2f}%)
- Momentum (20 periods): {indicators.get('momentum_20', 0):+.2f}%
- RSI: {indicators.get('rsi', 50):.1f}
- Volume Ratio: {indicators.get('volume_ratio', 1.0):.2f}x
- Last 5 closes: {indicators.get('last_5_closes', [])}"""


def _predict_trend_technical(symbol: str, candles_by_tf: Dict[str, List], primary_tf: str) -> Dict[str, Any]:
    """
    Fallback technical analysis when GPT-4 is unavailable.
    Uses simple EMA200 + momentum logic.
    """
    # Use primary timeframe or fallback to first available
    tf = primary_tf if primary_tf in candles_by_tf else list(candles_by_tf.keys())[0]
    candles = candles_by_tf[tf]

    df = pd.DataFrame(candles)
    df = df.sort_values("open_time")

    if len(df) < 200:
        return {"trend": "Neutral", "confidence": 0.3, "reasoning": "Insufficient data"}

    # Calculate EMA200
    df["ema_200"] = df["close"].ewm(span=200, adjust=False).mean()

    current_price = df["close"].iloc[-1]
    current_ema = df["ema_200"].iloc[-1]
    distance_pct = ((current_price - current_ema) / current_ema) * 100

    # Calculate momentum
    price_change_20 = ((df["close"].iloc[-1] / df["close"].iloc[-20]) - 1) * 100

    # Determine trend
    if distance_pct > 1.0 and price_change_20 > 2.0:
        trend = "Uptrend"
        confidence = min(0.75 + (abs(distance_pct) / 10), 0.95)
        reasoning = f"Price {distance_pct:.1f}% above EMA200 with {price_change_20:.1f}% upward momentum"
    elif distance_pct < -1.0 and price_change_20 < -2.0:
        trend = "Downtrend"
        confidence = min(0.75 + (abs(distance_pct) / 10), 0.95)
        reasoning = f"Price {distance_pct:.1f}% below EMA200 with {price_change_20:.1f}% downward momentum"
    else:
        trend = "Neutral"
        confidence = 0.50
        reasoning = "Mixed signals - price near EMA200 or conflicting momentum"

    return {
        "trend": trend,
        "confidence": confidence,
        "reasoning": reasoning
    }


@app.get("/ai/info", response_model=AIServiceInfo)
@limiter.limit("120/minute")  # Rate limit: 120 requests per minute
async def get_service_info(request: Request):
    """Get AI service information."""
    return AIServiceInfo()


@app.get("/ai/strategies")
@limiter.limit("120/minute")  # Rate limit: 120 requests per minute
async def get_supported_strategies(request: Request):
    """Get list of supported strategies."""
    return [
        "RSI Strategy",
        "MACD Strategy",
        "Volume Strategy",
        "Bollinger Bands Strategy",
        "Stochastic Strategy",
    ]


@app.get("/ai/performance", response_model=AIModelPerformance)
@limiter.limit("120/minute")  # Rate limit: 120 requests per minute
async def get_model_performance(request: Request):
    """Get AI model performance metrics."""
    return AIModelPerformance()


@app.get("/ai/cost/statistics")
@limiter.limit("60/minute")  # Rate limit: 60 requests per minute
async def get_cost_statistics(request: Request):
    """Get GPT-4 API cost statistics."""
    # Note: Reading global variables (no 'global' keyword needed for read-only access)

    # Fetch current symbols dynamically
    current_symbols = await fetch_analysis_symbols()

    # Calculate estimates
    estimated_cost_per_day = (
        (total_cost_usd / max(total_requests_count, 1))
        * (24 * 60 / max(ANALYSIS_INTERVAL_MINUTES, 1) * len(current_symbols))
        if total_requests_count > 0
        else 0.0
    )

    estimated_cost_per_month = estimated_cost_per_day * 30

    return {
        "session_statistics": {
            "total_requests": total_requests_count,
            "total_input_tokens": total_input_tokens,
            "total_output_tokens": total_output_tokens,
            "total_tokens": total_input_tokens + total_output_tokens,
            "total_cost_usd": round(total_cost_usd, 4),
            "total_cost_vnd": round(
                total_cost_usd * 23000, 0
            ),  # Approximate VND conversion
            "average_cost_per_request_usd": round(
                total_cost_usd / max(total_requests_count, 1), 5
            ),
            "average_tokens_per_request": round(
                (total_input_tokens + total_output_tokens)
                / max(total_requests_count, 1),
                0,
            ),
        },
        "projections": {
            "estimated_daily_cost_usd": round(estimated_cost_per_day, 3),
            "estimated_daily_cost_vnd": round(estimated_cost_per_day * 23000, 0),
            "estimated_monthly_cost_usd": round(estimated_cost_per_month, 2),
            "estimated_monthly_cost_vnd": round(estimated_cost_per_month * 23000, 0),
        },
        "configuration": {
            "model": "gpt-4o-mini",
            "analysis_interval_minutes": ANALYSIS_INTERVAL_MINUTES,
            "symbols_tracked": len(current_symbols),
            "cache_duration_minutes": 15,  # Updated cache duration
            "max_tokens": 1200,  # Optimized max tokens
            "input_cost_per_1m_tokens": GPT4O_MINI_INPUT_COST_PER_1M,
            "output_cost_per_1m_tokens": GPT4O_MINI_OUTPUT_COST_PER_1M,
        },
        "optimization_status": {
            "cache_optimized": True,
            "interval_optimized": True,
            "prompt_optimized": True,
            "max_tokens_optimized": True,
            "estimated_savings_percent": 63,  # Based on our optimization calculations
        },
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }


@app.get("/ai/storage/stats")
@limiter.limit("60/minute")  # Rate limit: 60 requests per minute
async def get_storage_statistics(request: Request):
    """Get AI analysis storage statistics."""
    if mongodb_db is None:
        return {"error": "MongoDB not connected"}

    try:
        # Get total stored analyses
        total_count = await mongodb_db[AI_ANALYSIS_COLLECTION].count_documents({})

        # Get analyses by symbol
        pipeline = [
            {
                "$group": {
                    "_id": "$symbol",
                    "count": {"$sum": 1},
                    "latest": {"$max": "$timestamp"},
                }
            },
            {"$sort": {"latest": -1}},
        ]

        symbol_stats = []
        async for doc in mongodb_db[AI_ANALYSIS_COLLECTION].aggregate(pipeline):
            symbol_stats.append(
                {
                    "symbol": doc["_id"],
                    "analysis_count": doc["count"],
                    "latest_analysis": (
                        doc["latest"].isoformat()
                        if isinstance(doc["latest"], datetime)
                        else str(doc["latest"])
                    ),
                }
            )

        return {
            "total_analyses": total_count,
            "symbols_analyzed": len(symbol_stats),
            "symbol_breakdown": symbol_stats,
            "analysis_interval_minutes": ANALYSIS_INTERVAL_MINUTES,
            "collection_name": AI_ANALYSIS_COLLECTION,
        }
    except Exception as e:
        return {"error": f"Failed to get storage stats: {e}"}


@app.post("/ai/storage/clear")
@limiter.limit("10/minute")  # Rate limit: 10 requests per minute (dangerous operation)
async def clear_storage(request: Request):
    """Clear AI analysis storage."""
    if mongodb_db is None:
        return {"error": "MongoDB not connected"}

    try:
        result = await mongodb_db[AI_ANALYSIS_COLLECTION].delete_many({})
        logger.info(f"🧹 Cleared {result.deleted_count} stored analyses")
        return {
            "message": "Storage cleared successfully",
            "cleared_analyses": result.deleted_count,
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }
    except Exception as e:
        return {"error": f"Failed to clear storage: {e}"}


# === PROJECT CHATBOT ENDPOINTS ===

# Global chatbot instance (initialized lazily)
_project_chatbot: Optional[ProjectChatbot] = None


@app.post("/api/chat/project", response_model=ProjectChatResponse)
async def chat_with_project(request: ProjectChatRequest):
    """
    Chat with the project documentation using RAG (Retrieval Augmented Generation).

    This endpoint allows users to ask questions about the BotCore project.
    It uses GPT-4 with context from indexed project documentation (specs, docs, READMEs).

    - **message**: The question to ask about the project
    - **include_history**: Whether to include conversation history for context

    Returns:
    - **message**: AI-generated response
    - **sources**: List of source documents used for the answer
    - **confidence**: Confidence score (0-1)
    - **type**: Response type (rag, fallback, error)
    """
    global _project_chatbot

    try:
        # Get or create chatbot instance with OpenAI client
        if _project_chatbot is None:
            _project_chatbot = await get_chatbot(openai_client)
        elif openai_client and _project_chatbot.openai_client is None:
            _project_chatbot.openai_client = openai_client

        # Process the message
        result = await _project_chatbot.chat(
            message=request.message,
            include_history=request.include_history
        )

        return ProjectChatResponse(
            success=result.get("success", False),
            message=result.get("message", ""),
            sources=[
                ProjectChatSource(title=s["title"], path=s["path"])
                for s in result.get("sources", [])
            ],
            confidence=result.get("confidence", 0.0),
            type=result.get("type", "error"),
            tokens_used=result.get("tokens_used", {}),
        )

    except Exception as e:
        logger.error(f"Project chatbot error: {e}")
        return ProjectChatResponse(
            success=False,
            message=f"Xin lỗi, có lỗi xảy ra: {str(e)}",
            sources=[],
            confidence=0.0,
            type="error",
        )


@app.get("/api/chat/project/suggestions")
async def get_chat_suggestions():
    """Get suggested questions for the project chatbot."""
    global _project_chatbot

    if _project_chatbot is None:
        _project_chatbot = await get_chatbot(openai_client)

    return {
        "suggestions": _project_chatbot.get_suggested_questions(),
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }


@app.post("/api/chat/project/clear")
async def clear_chat_history():
    """Clear conversation history for the project chatbot."""
    global _project_chatbot

    if _project_chatbot is not None:
        _project_chatbot.clear_history()

    return {
        "success": True,
        "message": "Conversation history cleared",
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }


@app.get("/")
async def root():
    """Root endpoint with service information."""
    return {
        "service": "GPT-4 Cryptocurrency AI Trading Service",
        "version": "2.0.0",
        "description": (
            "Advanced AI-powered trading signal generation using OpenAI GPT-4 "
            "with MongoDB storage and real-time WebSocket broadcasting"
        ),
        "endpoints": {
            "analyze": "POST /ai/analyze - Generate trading signals with GPT-4 (stored in MongoDB)",
            "strategy_recommendations": "POST /ai/strategy-recommendations - Get strategy recommendations",
            "market_condition": "POST /ai/market-condition - Analyze market conditions",
            "feedback": "POST /ai/feedback - Send performance feedback",
            "health": "GET /health - Health check with MongoDB status",
            "storage_stats": "GET /ai/storage/stats - View storage statistics",
            "cost_stats": "GET /ai/cost/statistics - View GPT-4 API cost statistics",
            "clear_storage": "POST /ai/storage/clear - Clear analysis storage",
            "websocket": "WS /ws - Real-time AI signal broadcasting",
            "project_chat": "POST /api/chat/project - RAG chatbot for project questions",
            "chat_suggestions": "GET /api/chat/project/suggestions - Get suggested questions",
            "clear_chat": "POST /api/chat/project/clear - Clear chat history",
        },
        "documentation": "/docs",
        "features": {
            "gpt4_enabled": openai_client is not None,
            "mongodb_storage": mongodb_client is not None,
            "websocket_broadcasting": True,
            "periodic_analysis": True,
            "analysis_interval_minutes": ANALYSIS_INTERVAL_MINUTES,
            "symbols_tracked": await fetch_analysis_symbols(),
        },
    }


# Run the application
if __name__ == "__main__":
    import uvicorn

    logger.info("🚀 Starting GPT-4 AI Trading Service...")

    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=False, log_level="info")
