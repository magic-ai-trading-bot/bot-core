#!/usr/bin/env python3
"""
AI Trading Service with Grok (xAI) Integration + Real-time WebSocket
Advanced trading signal generation using xAI Grok for cryptocurrency markets.
Compatible with Rust AI client endpoints with WebSocket broadcasting.
"""

import asyncio
import json
import logging
import os
import sys
from contextlib import asynccontextmanager
from datetime import datetime, timedelta, timezone
from typing import Any, Dict, List, Optional, Set

import fastapi
import numpy as np
import pandas as pd
import ta
from fastapi import FastAPI, HTTPException, Request, WebSocket, WebSocketDisconnect
from fastapi.middleware.cors import CORSMiddleware
from motor.motor_asyncio import AsyncIOMotorClient
from pydantic import BaseModel, Field
from pymongo import ASCENDING
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.errors import RateLimitExceeded
from slowapi.util import get_remote_address

from app.core.config import GROK_REQUEST_DELAY

# Load configuration
from config_loader import AI_CACHE_DURATION_MINUTES, AI_CACHE_ENABLED
from config_loader import SIGNAL_CONFIDENCE_BASE as CONFIG_SIGNAL_CONFIDENCE_BASE
from config_loader import SIGNAL_CONFIDENCE_PER_TF as CONFIG_SIGNAL_CONFIDENCE_PER_TF
from config_loader import SIGNAL_MIN_INDICATORS as CONFIG_SIGNAL_MIN_INDICATORS
from config_loader import SIGNAL_MIN_TIMEFRAMES as CONFIG_SIGNAL_MIN_TIMEFRAMES
from config_loader import SIGNAL_TREND_THRESHOLD as CONFIG_SIGNAL_TREND_THRESHOLD

# Project chatbot service (RAG)
from services.project_chatbot import ProjectChatbot, get_chatbot

# @spec:FR-SETTINGS-001, FR-SETTINGS-002 - Unified settings from Rust API
# Settings manager fetches indicator/signal settings from Rust API with caching
# Falls back to config.yaml values if Rust API is unavailable
from settings_manager import refresh_settings_periodically, settings_manager

# Note: These config values are now fetched from Rust API via settings_manager
# Falls back to config.yaml values if Rust API is unavailable


# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# AI Provider Configuration (supports xAI Grok, OpenAI, etc.)
AI_BASE_URL = os.getenv("AI_BASE_URL", "https://api.x.ai/v1")
AI_MODEL = os.getenv("AI_MODEL", "grok-4-1-fast-non-reasoning")

# Global AI client, WebSocket connections, and MongoDB storage
# Thread safety: These are only written during startup/shutdown in lifespan
# Read-only access during request handling is safe
# Type annotations help mypy understand optional types
grok_client: Optional[Any] = None
websocket_connections: Set[WebSocket] = set()
mongodb_client: Optional[AsyncIOMotorClient] = None
mongodb_db: Optional[Any] = None

# Rate limiting for AI API
# Thread safety: Access to these variables is protected by asyncio.Lock
# for proper async/await compatibility
_rate_limit_lock = asyncio.Lock()
last_grok_request_time = None
# GROK_REQUEST_DELAY is imported from app.core.config (reads from env var)
GROK_RATE_LIMIT_RESET_TIME = None  # Track when rate limit resets

# Cost monitoring (xAI Grok 4.1 Fast pricing)
AI_INPUT_COST_PER_1M = 0.200  # $0.200 per 1M input tokens
AI_OUTPUT_COST_PER_1M = 0.500  # $0.500 per 1M output tokens
total_input_tokens = 0
total_output_tokens = 0
total_requests_count = 0
total_cost_usd = 0.0

# MongoDB storage for AI analysis results
AI_ANALYSIS_COLLECTION = "ai_analysis_results"
# Load from config.yaml (default: 2 minutes)
ANALYSIS_INTERVAL_MINUTES = AI_CACHE_DURATION_MINUTES


# @spec:FR-SETTINGS-001, FR-SETTINGS-002 - Dynamic settings from Rust API
# These helper functions get settings from settings_manager with fallback to config.yaml
def get_signal_trend_threshold() -> float:
    """Get trend threshold with fallback to config.yaml"""
    return settings_manager.get_signal_value(
        "trend_threshold_percent", CONFIG_SIGNAL_TREND_THRESHOLD
    )


def get_signal_min_timeframes() -> int:
    """Get min required timeframes with fallback to config.yaml"""
    return settings_manager.get_signal_value(
        "min_required_timeframes", CONFIG_SIGNAL_MIN_TIMEFRAMES
    )


def get_signal_min_indicators() -> int:
    """Get min required indicators with fallback to config.yaml"""
    return settings_manager.get_signal_value(
        "min_required_indicators", CONFIG_SIGNAL_MIN_INDICATORS
    )


def get_signal_confidence_base() -> float:
    """Get confidence base with fallback to config.yaml"""
    return settings_manager.get_signal_value(
        "confidence_base", CONFIG_SIGNAL_CONFIDENCE_BASE
    )


def get_signal_confidence_per_tf() -> float:
    """Get confidence per timeframe with fallback to config.yaml"""
    return settings_manager.get_signal_value(
        "confidence_per_timeframe", CONFIG_SIGNAL_CONFIDENCE_PER_TF
    )


def get_rsi_period() -> int:
    """Get RSI period with fallback to default 14"""
    return settings_manager.get_indicator_value("rsi_period", 14)


def get_macd_periods() -> tuple:
    """Get MACD periods (fast, slow, signal) with fallback to defaults"""
    return (
        settings_manager.get_indicator_value("macd_fast", 12),
        settings_manager.get_indicator_value("macd_slow", 26),
        settings_manager.get_indicator_value("macd_signal", 9),
    )


def get_bollinger_settings() -> tuple:
    """Get Bollinger Bands settings (period, std) with fallback to defaults"""
    return (
        settings_manager.get_indicator_value("bollinger_period", 20),
        settings_manager.get_indicator_value("bollinger_std", 2.0),
    )


def get_stochastic_periods() -> tuple:
    """Get Stochastic periods (k, d) with fallback to defaults"""
    return (
        settings_manager.get_indicator_value("stochastic_k_period", 14),
        settings_manager.get_indicator_value("stochastic_d_period", 3),
    )


# For backward compatibility, create module-level aliases that use the helper functions
# These are evaluated at module load time, so they'll use config.yaml values initially
# After settings_manager is initialized, the helper functions should be used directly
SIGNAL_TREND_THRESHOLD = CONFIG_SIGNAL_TREND_THRESHOLD  # Use helper function in code
SIGNAL_MIN_TIMEFRAMES = CONFIG_SIGNAL_MIN_TIMEFRAMES  # Use helper function in code
SIGNAL_MIN_INDICATORS = CONFIG_SIGNAL_MIN_INDICATORS  # Use helper function in code
SIGNAL_CONFIDENCE_BASE = CONFIG_SIGNAL_CONFIDENCE_BASE  # Use helper function in code
SIGNAL_CONFIDENCE_PER_TF = (
    CONFIG_SIGNAL_CONFIDENCE_PER_TF  # Use helper function in code
)


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
                        logger.info(
                            f"📊 Fetched {len(symbols)} symbols from Rust API: {symbols}"
                        )
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


async def push_market_bias_to_rust(symbol: str, analysis_result) -> None:
    """Push AI market bias to Rust core engine for zero-latency signal filtering."""
    try:
        import httpx

        # Convert signal to direction bias
        signal_str = (
            analysis_result.signal.lower()
            if isinstance(analysis_result.signal, str)
            else str(analysis_result.signal).lower()
        )
        if signal_str in ("long", "buy", "bullish"):
            direction_bias = 1.0
        elif signal_str in ("short", "sell", "bearish"):
            direction_bias = -1.0
        else:
            direction_bias = 0.0

        bias_data = {
            "symbol": symbol,
            "direction_bias": direction_bias,
            "bias_strength": (
                analysis_result.market_analysis.trend_strength
                if hasattr(analysis_result, "market_analysis")
                and hasattr(analysis_result.market_analysis, "trend_strength")
                else abs(direction_bias) * analysis_result.confidence
            ),
            "bias_confidence": analysis_result.confidence,
            "ttl_seconds": 600,
        }

        async with httpx.AsyncClient(timeout=5.0) as client:
            response = await client.post(
                f"{RUST_API_URL}/api/ai/market-bias", json=bias_data
            )
            if response.status_code == 200:
                logger.info(
                    f"📡 Pushed market bias to Rust: {symbol} dir={direction_bias:.1f} conf={analysis_result.confidence:.2f}"
                )
            else:
                logger.warning(
                    f"⚠️ Failed to push market bias for {symbol}: HTTP {response.status_code}"
                )
    except Exception as e:
        logger.warning(f"⚠️ Failed to push market bias for {symbol}: {e}")


async def periodic_analysis_runner():
    """Background task that runs AI analysis every 5 minutes."""
    logger.info("🔄 Starting periodic analysis runner")

    while True:
        try:
            # Fetch symbols dynamically from Rust API (includes user-added symbols)
            analysis_symbols = await fetch_analysis_symbols()
            logger.info(
                f"🤖 Starting periodic AI analysis cycle for {len(analysis_symbols)} symbols"
            )

            # Analyze each symbol
            for symbol in analysis_symbols:
                try:
                    # Fetch REAL market data from Rust Core Engine API
                    analysis_request = await fetch_real_market_data(symbol)

                    # Run AI analysis
                    analyzer = GrokTradingAnalyzer(grok_client)
                    analysis_result = await analyzer.analyze_trading_signals(
                        analysis_request
                    )

                    # Store result in MongoDB
                    await store_analysis_result(symbol, analysis_result.model_dump())

                    # Push market bias to Rust for zero-latency signal filtering
                    await push_market_bias_to_rust(symbol, analysis_result)

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
    global grok_client, mongodb_client, mongodb_db

    # Startup
    logger.info("🚀 Starting Grok AI Trading Service")
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

    # Initialize AI client with API keys from environment
    # Support xAI (XAI_API_KEY) with fallback to OpenAI-compatible (OPENAI_API_KEY)
    api_key_string = os.getenv("XAI_API_KEY", "") or os.getenv("OPENAI_API_KEY", "")
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
        logger.error("❌ No valid AI API keys found!")
        api_key = None
    else:
        api_key = valid_api_keys[0]  # Use the first valid key
        logger.info(f"✅ Found {len(valid_api_keys)} valid API keys for fallback")
        if len(valid_api_keys) > 1:
            logger.info("🔄 Backup keys available for auto-fallback on rate limits")

    logger.info(f"🔑 AI API key configured: {bool(api_key)}")
    logger.info(f"🌐 AI Base URL: {AI_BASE_URL}")
    logger.info(f"🤖 AI Model: {AI_MODEL}")

    if not api_key or api_key.startswith("your-"):
        logger.error("❌ AI API key not configured!")
        grok_client = None
    else:
        logger.info(f"🔄 Initializing AI client ({AI_BASE_URL})...")

        # Use direct HTTP client (xAI/Grok-compatible API)
        try:
            grok_client = GrokClient(valid_api_keys)
            logger.info(f"✅ AI HTTP client initialized ({AI_MODEL} via {AI_BASE_URL})")
            logger.info(f"🔑 Total API keys configured: {len(valid_api_keys)}")
        except Exception as e:
            logger.error(f"❌ Failed to initialize AI client: {e}")
            grok_client = None

    if grok_client is not None:
        logger.info(f"✅ AI client ready ({AI_MODEL})")
    else:
        logger.warning("❌ AI unavailable - will use fallback technical analysis")

    # @spec:FR-SETTINGS-001, FR-SETTINGS-002 - Initialize settings from Rust API
    # Fetch indicator/signal settings from Rust API with fallback to config.yaml
    logger.info("📊 Initializing settings from Rust API...")
    settings_initialized = await settings_manager.initialize()
    if settings_initialized:
        logger.info("✅ Settings loaded from Rust API")
        # Log the actual values being used
        logger.info(f"📊 RSI period: {get_rsi_period()}")
        logger.info(f"📊 Trend threshold: {get_signal_trend_threshold()}%")
        logger.info(f"📊 Min timeframes: {get_signal_min_timeframes()}")
        logger.info(f"📊 Min indicators: {get_signal_min_indicators()}")
    else:
        logger.warning("⚠️ Using fallback settings from config.yaml")

    # Error handler for background tasks
    def handle_task_exception(task: asyncio.Task) -> None:
        """Handle exceptions from background tasks to prevent silent failures."""
        try:
            task.result()
        except asyncio.CancelledError:
            pass  # Expected during shutdown
        except Exception as e:
            task_name = task.get_name()
            # Log with full traceback to internal logs only (not exposed to clients)
            logger.error(
                f"❌ Background task '{task_name}' failed: {type(e).__name__}",
                exc_info=False,
            )
            # TODO: Add notification system to alert on critical task failures

    # Start background settings refresh task (every 5 minutes)
    settings_refresh_task = asyncio.create_task(
        refresh_settings_periodically(), name="settings_refresh"
    )
    settings_refresh_task.add_done_callback(handle_task_exception)
    logger.info("🔄 Started settings refresh background task")

    # Start background analysis task
    analysis_task = asyncio.create_task(
        periodic_analysis_runner(), name="periodic_analysis"
    )
    analysis_task.add_done_callback(handle_task_exception)
    logger.info(
        f"🔄 Started periodic analysis task (every {ANALYSIS_INTERVAL_MINUTES} minutes)"
    )

    yield

    # Shutdown
    logger.info("🛑 Shutting down AI Trading Service")

    # Cancel tasks
    analysis_task.cancel()
    settings_refresh_task.cancel()

    # Wait for tasks to finish cancellation
    try:
        await asyncio.gather(
            analysis_task, settings_refresh_task, return_exceptions=True
        )
        logger.info("✅ Background tasks cancelled successfully")
    except Exception as e:
        logger.warning(f"⚠️ Error during task cleanup: {e}")

    # Close MongoDB connection
    if mongodb_client:
        mongodb_client.close()
        logger.info("✅ MongoDB connection closed")


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
    title="Grok AI Cryptocurrency Trading Service",
    description="Advanced AI-powered trading signal generation using xAI Grok",
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
    direction: float = Field(
        default=0.0, description="Direction score: -1.0 (bearish) to +1.0 (bullish)"
    )
    trend_strength: float = Field(
        default=0.0, ge=0, le=1, description="Trend strength from ADX (0-1)"
    )
    characteristics: List[str] = Field(default_factory=list)
    recommended_strategies: List[str] = Field(default_factory=list)
    market_phase: str = Field(..., description="Current market phase")
    timeframe_analysis: Dict[str, Any] = Field(
        default_factory=dict, description="Per-timeframe breakdown"
    )
    indicators_summary: Dict[str, Any] = Field(
        default_factory=dict, description="Key indicator values"
    )


class MarketConditionRequest(BaseModel):
    """Market condition request."""

    symbol: str = Field(..., description="Trading symbol")
    timeframe_data: Dict[str, List[CandleData]] = Field(
        default_factory=dict,
        description="Multi-timeframe data (optional, will fetch from DB if empty)",
    )
    current_price: float = Field(default=0.0, ge=0, description="Current price")
    volume_24h: float = Field(default=0.0, ge=0, description="24h volume")
    timestamp: int = Field(default=0, description="Request timestamp")


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

    trend: str = Field(
        ..., description="Predicted trend direction (Uptrend/Downtrend/Neutral)"
    )
    confidence: float = Field(
        ..., ge=0, le=1, description="Confidence in prediction (0.0-1.0)"
    )
    model: str = Field(..., description="ML model used for prediction")
    timestamp: int = Field(..., description="Prediction timestamp")


class AIServiceInfo(BaseModel):
    """AI service information."""

    model_config = {"protected_namespaces": ()}

    service_name: str = Field(default="Grok Trading AI")
    version: str = Field(default="3.0.0")
    model_version: str = Field(default="grok-4-1-fast-non-reasoning")
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
    include_history: bool = Field(
        default=True, description="Include conversation history"
    )


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
            # @spec:FR-SETTINGS-001 - Use dynamic RSI period from Rust API
            rsi_period = get_rsi_period()
            indicators["rsi"] = (
                ta.momentum.rsi(df["close"], window=rsi_period).iloc[-1]
                if len(df) >= rsi_period
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


# === HTTP-BASED GROK CLIENT ===


class GrokClient:
    """Direct HTTP client for AI API (xAI Grok / OpenAI compatible) with auto-fallback support."""

    def __init__(self, api_keys: list, base_url: str = None):
        self.api_keys = api_keys if isinstance(api_keys, list) else [api_keys]
        self.current_key_index = 0
        self.base_url = base_url or AI_BASE_URL
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
    ) -> Dict[str, Any]:
        """Direct HTTP call to OpenAI chat completions API with auto-fallback on rate limits."""
        global last_grok_request_time, GROK_RATE_LIMIT_RESET_TIME
        import httpx

        # Try each available API key until success or all are exhausted
        max_attempts = len(self.api_keys)

        for attempt in range(max_attempts):
            current_key, key_index = self.get_current_api_key()

            # Check if we're still in a rate limit period (async-safe)
            async with _rate_limit_lock:
                if GROK_RATE_LIMIT_RESET_TIME:
                    if datetime.now() < GROK_RATE_LIMIT_RESET_TIME:
                        remaining_time = (
                            GROK_RATE_LIMIT_RESET_TIME - datetime.now()
                        ).total_seconds()
                        logger.warning(
                            f"⏰ API key in rate limit "
                            f"period, {remaining_time:.0f}s remaining"
                        )
                        # Try next key
                        self.current_key_index += 1
                        continue
                    else:
                        # Rate limit period expired, reset it
                        GROK_RATE_LIMIT_RESET_TIME = None
                        self.rate_limited_keys.discard(key_index)
                        logger.info(f"✅ API key rate limit expired")

            # Rate limiting: ensure minimum delay between requests
            # (checked outside lock to allow sleep without blocking)
            if last_grok_request_time:
                time_since_last = (
                    datetime.now() - last_grok_request_time
                ).total_seconds()
                if time_since_last < GROK_REQUEST_DELAY:
                    delay = GROK_REQUEST_DELAY - time_since_last
                    logger.info(
                        f"⏳ Rate limiting: waiting {delay:.1f}s " f"before request"
                    )
                    await asyncio.sleep(delay)

            # Update last request time (async-safe)
            async with _rate_limit_lock:
                last_grok_request_time = datetime.now()

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
                logger.info(f"🔑 Using API key (attempt {attempt + 1}/{max_attempts})")

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
                            GROK_RATE_LIMIT_RESET_TIME = reset_time
                            logger.warning(
                                f"⏰ API key rate limited until {reset_time}"
                            )
                        else:
                            # Default to 1 hour if no retry-after header
                            GROK_RATE_LIMIT_RESET_TIME = datetime.now() + timedelta(
                                hours=1
                            )
                            logger.warning(f"⏰ API key rate limited for 1 hour")

                        # Try next key
                        self.current_key_index += 1
                        continue

                    response.raise_for_status()
                    logger.info("✅ Request successful with API key")
                    return response.json()

            except httpx.HTTPStatusError as e:
                if e.response.status_code == 429:
                    logger.error(f"🚫 API key rate limit exceeded (429)")
                    self.rate_limited_keys.add(key_index)
                    self.current_key_index += 1
                    continue
                elif e.response.status_code == 401:
                    logger.error(f"🔑 API key authentication failed (401)")
                    self.current_key_index += 1
                    continue
                elif e.response.status_code == 403:
                    logger.error(f"💰 API key quota exceeded (403)")
                    self.current_key_index += 1
                    continue
                else:
                    logger.error(f"❌ API key API error: {e.response.status_code}")
                    raise
            except Exception as e:
                logger.error(f"❌ API key network error: {e}")
                if attempt == max_attempts - 1:  # Last attempt
                    raise
                else:
                    self.current_key_index += 1
                    continue

        # If we get here, all keys failed
        raise Exception("All API keys exhausted or rate limited")


# === GROK AI ANALYZER ===


class GrokTradingAnalyzer:
    """Grok (xAI) powered trading analysis."""

    def __init__(self, client):
        self.client = client

    async def analyze_trading_signals(
        self, request: AIAnalysisRequest
    ) -> AISignalResponse:
        """Analyze trading signals using Grok or fallback technical analysis."""
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
                indicators_15m = TechnicalAnalyzer.calculate_indicators(
                    dataframes["15m"]
                )

            if "30m" in dataframes and len(dataframes["30m"]) >= 2:
                indicators_30m = TechnicalAnalyzer.calculate_indicators(
                    dataframes["30m"]
                )

            if "1h" in dataframes and len(dataframes["1h"]) >= 2:
                indicators_1h = TechnicalAnalyzer.calculate_indicators(dataframes["1h"])

            if "4h" in dataframes and len(dataframes["4h"]) >= 2:
                indicators_4h = TechnicalAnalyzer.calculate_indicators(dataframes["4h"])

            # Choose analysis method based on client availability
            # Pass all 4 timeframes for comprehensive multi-timeframe analysis
            if self.client is not None:
                ai_analysis = await self._gpt_analysis(
                    request,
                    indicators_15m,
                    indicators_30m,
                    indicators_1h,
                    indicators_4h,
                )
            else:
                ai_analysis = self._fallback_analysis(
                    request,
                    indicators_15m,
                    indicators_30m,
                    indicators_1h,
                    indicators_4h,
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
            raise HTTPException(
                status_code=500,
                detail="AI analysis failed. Check server logs for details.",
            )

    async def _gpt_analysis(
        self,
        request: AIAnalysisRequest,
        indicators_15m: Dict,
        indicators_30m: Dict,
        indicators_1h: Dict,
        indicators_4h: Dict,
    ) -> Dict[str, Any]:
        """Grok powered analysis with multi-timeframe support (15m, 30m, 1h, 4h)."""
        try:
            logger.info(f"🤖 Starting Grok analysis for {request.symbol}")

            # Prepare market context with ALL timeframes
            market_context = self._prepare_market_context(
                request, indicators_15m, indicators_30m, indicators_1h, indicators_4h
            )
            logger.debug(
                f"📊 Market context prepared: {len(market_context)} characters"
            )

            # Create Grok prompt
            prompt = self._create_analysis_prompt(
                market_context, request.strategy_context
            )
            logger.debug(f"📝 Prompt created: {len(prompt)} characters")
            logger.debug(
                f"🎯 Selected strategies: {request.strategy_context.selected_strategies}"
            )

            # Call Grok API (optimized max_tokens for cost saving)
            logger.info("🔄 Calling Grok API...")
            response = await self.client.chat_completions_create(
                model=AI_MODEL,
                messages=[
                    {"role": "system", "content": self._get_system_prompt()},
                    {"role": "user", "content": prompt},
                ],
                temperature=0.0,
                max_tokens=1200,  # Reduced from 2000 to 1200 for cost optimization
            )

            logger.info("✅ Grok API call successful")
            response_content = response["choices"][0]["message"]["content"]
            logger.debug(f"📤 Grok response: {response_content[:200]}...")

            # Track token usage and cost
            usage = response.get("usage", {})
            if usage:
                input_tokens = usage.get("prompt_tokens", 0)
                output_tokens = usage.get("completion_tokens", 0)
                total_tokens = usage.get("total_tokens", 0)

                # Calculate cost
                input_cost = (input_tokens / 1_000_000) * AI_INPUT_COST_PER_1M
                output_cost = (output_tokens / 1_000_000) * AI_OUTPUT_COST_PER_1M
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

            # Parse Grok response
            parsed_result = self._parse_gpt_response(response_content)
            strategy_scores = parsed_result.get("strategy_scores", {})
            gpt_confidence = parsed_result.get("confidence", 0.5)
            gpt_signal = parsed_result.get("signal", "Neutral")

            # POST-PROCESS: Recalculate confidence based on timeframe agreement
            # Grok often returns 0.5 as default, so we calculate proper confidence ourselves
            recalculated_confidence = self._recalculate_confidence(
                gpt_signal,
                request,
                indicators_15m,
                indicators_30m,
                indicators_1h,
                indicators_4h,
            )

            # Use recalculated confidence for directional signals, keep GPT's for Neutral
            if gpt_signal in ["Long", "Short"] and gpt_confidence == 0.5:
                parsed_result["confidence"] = recalculated_confidence
                logger.info(
                    f"📈 Confidence recalculated: GPT returned {gpt_confidence} -> adjusted to {recalculated_confidence}"
                )

            logger.info(
                f"🎯 Grok analysis complete: signal={parsed_result.get('signal')}, "
                f"confidence={parsed_result.get('confidence')} (GPT original: {gpt_confidence})"
            )
            logger.info(f"📊 Strategy scores: {strategy_scores}")

            return parsed_result

        except Exception as e:
            logger.error(f"❌ Grok analysis failed: {e}")
            logger.error(f"Error type: {type(e).__name__}")

            if "401" in str(e):
                logger.error("🔑 Grok API authentication failed")
            elif "429" in str(e):
                logger.error("⏱️ Grok rate limit exceeded")
            elif "quota" in str(e).lower():
                logger.error("💰 Grok quota exceeded")
            elif "timeout" in str(e).lower():
                logger.error("⏰ Grok API timeout")
            else:
                logger.error(f"🌐 Grok API error: {str(e)}")

            # Fall back to technical analysis
            logger.warning("🔄 Falling back to technical analysis...")
            return self._fallback_analysis(
                request, indicators_15m, indicators_30m, indicators_1h, indicators_4h
            )

    def _classify_timeframe(
        self, tf_name: str, indicators: Dict, candles: List, threshold: float
    ) -> tuple[str, str]:
        """
        Classify a timeframe as BULLISH, BEARISH, or NEUTRAL.

        Uses unified logic matching Grok prompt (5 INDICATORS SCORING):
        - MACD: histogram > 0 = +1 bullish, < 0 = +1 bearish
        - RSI: > 55 = +1 bullish, < 45 = +1 bearish
        - BB: position < 0.3 = +1 bullish (near lower), > 0.7 = +1 bearish (near upper)
        - Stochastic: K > D and K < 80 = +1 bullish, K < D and K > 20 = +1 bearish
        - Volume: ratio > 1.2x confirms trend direction

        Classification (STRICT - requires SIGNAL_MIN_INDICATORS/5):
        - BULLISH if: SIGNAL_MIN_INDICATORS+ indicators are bullish (default 4/5)
        - BEARISH if: SIGNAL_MIN_INDICATORS+ indicators are bearish (default 4/5)
        - NEUTRAL otherwise

        Returns: (classification, reason_string)
        """
        if not indicators:
            return "NEUTRAL", f"{tf_name}: insufficient data"

        # Get indicator values
        macd_hist = indicators.get("macd_histogram", 0)
        rsi = indicators.get("rsi", 50)
        bb_position = indicators.get("bb_position", 0.5)
        stoch_k = indicators.get("stochastic_k", 50)
        stoch_d = indicators.get("stochastic_d", 50)
        volume_ratio = indicators.get("volume_ratio", 1.0)

        # Count bullish/bearish points
        bullish_points = 0
        bearish_points = 0
        signals = []

        # 1. MACD
        if macd_hist > 0:
            bullish_points += 1
            signals.append("MACD↑")
        elif macd_hist < 0:
            bearish_points += 1
            signals.append("MACD↓")

        # 2. RSI
        if rsi > 55:
            bullish_points += 1
            signals.append(f"RSI{rsi:.0f}↑")
        elif rsi < 45:
            bearish_points += 1
            signals.append(f"RSI{rsi:.0f}↓")

        # 3. Bollinger Bands
        if bb_position < 0.3:
            bullish_points += 1
            signals.append("BB↑")
        elif bb_position > 0.7:
            bearish_points += 1
            signals.append("BB↓")

        # 4. Stochastic
        if stoch_k > stoch_d and stoch_k < 80:
            bullish_points += 1
            signals.append(f"Stoch{stoch_k:.0f}↑")
        elif stoch_k < stoch_d and stoch_k > 20:
            bearish_points += 1
            signals.append(f"Stoch{stoch_k:.0f}↓")

        # 5. Volume (confirms trend direction)
        if volume_ratio > 1.2:
            if bullish_points > bearish_points:
                bullish_points += 1
                signals.append(f"Vol{volume_ratio:.1f}x↑")
            elif bearish_points > bullish_points:
                bearish_points += 1
                signals.append(f"Vol{volume_ratio:.1f}x↓")

        signal_str = " ".join(signals) if signals else "no signals"

        # @spec:FR-SETTINGS-002 - Use dynamic indicator threshold from Rust API
        min_indicators = get_signal_min_indicators()

        # Apply classification rules (min_indicators required, default 4/5)
        if bullish_points >= min_indicators:
            return (
                "BULLISH",
                f"{tf_name}: ✅ BULLISH ({bullish_points}/5 bull) [{signal_str}]",
            )
        elif bearish_points >= min_indicators:
            return (
                "BEARISH",
                f"{tf_name}: ⚠️ BEARISH ({bearish_points}/5 bear) [{signal_str}]",
            )
        else:
            return (
                "NEUTRAL",
                f"{tf_name}: ➖ NEUTRAL ({bullish_points}↑ {bearish_points}↓) [{signal_str}]",
            )

    def _recalculate_confidence(
        self,
        signal: str,
        request: "AIAnalysisRequest",
        indicators_15m: Dict,
        indicators_30m: Dict,
        indicators_1h: Dict,
        indicators_4h: Dict,
    ) -> float:
        """
        Recalculate confidence using WEIGHTED timeframe voting.

        Grok often returns 0.5 as default confidence. This method calculates
        proper confidence using WEIGHTED voting (same as fallback analysis):
        - 4H has 2.0 weight (main trend - most important)
        - 1H has 1.5 weight
        - 30M has 1.0 weight
        - 15M has 0.5 weight (short-term noise)

        Formula: confidence = 0.50 + (weighted_score / 100) * 0.35
        - Max 0.85 confidence
        - Counter-trend signals get lower confidence (0.45)

        @spec:FR-AI-WEIGHTED - Weighted timeframe voting for signal confidence
        """
        # Get dynamic settings from config
        trend_threshold = get_signal_trend_threshold()
        confidence_base = get_signal_confidence_base()

        # Classify all timeframes
        candles_15m = request.timeframe_data.get("15m", [])
        candles_30m = request.timeframe_data.get("30m", [])
        candles_1h = request.timeframe_data.get("1h", [])
        candles_4h = request.timeframe_data.get("4h", [])

        tf_15m, _ = self._classify_timeframe(
            "15M", indicators_15m, candles_15m, trend_threshold
        )
        tf_30m, _ = self._classify_timeframe(
            "30M", indicators_30m, candles_30m, trend_threshold
        )
        tf_1h, _ = self._classify_timeframe(
            "1H", indicators_1h, candles_1h, trend_threshold
        )
        tf_4h, _ = self._classify_timeframe(
            "4H", indicators_4h, candles_4h, trend_threshold
        )

        # ==================== WEIGHTED TIMEFRAME VOTING ====================
        # Same weights as fallback_analysis for consistency
        TIMEFRAME_WEIGHTS = {
            "15M": 0.5,  # Short-term noise, least important
            "30M": 1.0,  # Short-term trend
            "1H": 1.5,  # Medium-term trend
            "4H": 2.0,  # MAIN TREND - most important
        }

        tf_results = {"15M": tf_15m, "30M": tf_30m, "1H": tf_1h, "4H": tf_4h}

        # Calculate weighted scores
        weighted_bullish = 0.0
        weighted_bearish = 0.0
        total_weight = 0.0

        for tf_name, tf_class in tf_results.items():
            weight = TIMEFRAME_WEIGHTS.get(tf_name, 1.0)
            total_weight += weight

            if tf_class == "BULLISH":
                weighted_bullish += weight
            elif tf_class == "BEARISH":
                weighted_bearish += weight

        # Normalize to percentage (0-100%)
        bullish_score = (
            (weighted_bullish / total_weight) * 100 if total_weight > 0 else 0
        )
        bearish_score = (
            (weighted_bearish / total_weight) * 100 if total_weight > 0 else 0
        )

        # Main trend (4H) for counter-trend detection
        main_trend = tf_4h

        # Calculate confidence based on signal and weighted score
        if signal == "Long":
            if main_trend == "BEARISH":
                # Counter-trend Long = lower confidence
                confidence = 0.45
                logger.debug(
                    f"🔄 Confidence: Long but 4H BEARISH (counter-trend) → {confidence}"
                )
            else:
                confidence = min(0.85, confidence_base + (bullish_score / 100) * 0.35)
                logger.debug(
                    f"🔄 Confidence: Long, bullish_score={bullish_score:.1f}% → {confidence:.2f}"
                )
        elif signal == "Short":
            if main_trend == "BULLISH":
                # Counter-trend Short = lower confidence
                confidence = 0.45
                logger.debug(
                    f"🔄 Confidence: Short but 4H BULLISH (counter-trend) → {confidence}"
                )
            else:
                confidence = min(0.85, confidence_base + (bearish_score / 100) * 0.35)
                logger.debug(
                    f"🔄 Confidence: Short, bearish_score={bearish_score:.1f}% → {confidence:.2f}"
                )
        else:
            # Neutral signal
            confidence = 0.40

        logger.info(
            f"🔄 Weighted confidence: signal={signal}, 4H={main_trend}, "
            f"bull_score={bullish_score:.1f}%, bear_score={bearish_score:.1f}% → {confidence:.2f}"
        )

        return confidence

    def _fallback_analysis(
        self,
        request: AIAnalysisRequest,
        indicators_15m: Dict,
        indicators_30m: Dict,
        indicators_1h: Dict,
        indicators_4h: Dict,
    ) -> Dict[str, Any]:
        """
        Fallback technical analysis when Grok is not available.

        Uses UNIFIED LOGIC matching Grok prompt exactly (5 INDICATORS SCORING):
        - Each timeframe classified using 5 indicators: MACD, RSI, BB, Stochastic, Volume
        - Timeframe is BULLISH if SIGNAL_MIN_INDICATORS+ indicators are bullish
        - Timeframe is BEARISH if SIGNAL_MIN_INDICATORS+ indicators are bearish
        - Signal: bullish_timeframes >= MIN_REQUIRED → Long

        @spec:FR-SETTINGS-002 - Uses dynamic settings from Rust API
        Config values fetched from Rust API (fallback to config.yaml):
        - min_required_indicators: minimum indicators per timeframe (default 4/5)
        - min_required_timeframes: minimum timeframes to agree (default 3)
        """
        symbol = request.symbol
        reasoning = "Technical analysis (Grok unavailable): "
        timeframe_results = []

        # @spec:FR-SETTINGS-002 - Get dynamic settings from Rust API
        trend_threshold = get_signal_trend_threshold()
        min_timeframes = get_signal_min_timeframes()

        # Classify each timeframe using unified logic
        # 15M
        candles_15m = request.timeframe_data.get("15m", [])
        tf_15m, reason_15m = self._classify_timeframe(
            "15M", indicators_15m, candles_15m, trend_threshold
        )
        timeframe_results.append((tf_15m, reason_15m))

        # 30M
        candles_30m = request.timeframe_data.get("30m", [])
        tf_30m, reason_30m = self._classify_timeframe(
            "30M", indicators_30m, candles_30m, trend_threshold
        )
        timeframe_results.append((tf_30m, reason_30m))

        # 1H
        candles_1h = request.timeframe_data.get("1h", [])
        tf_1h, reason_1h = self._classify_timeframe(
            "1H", indicators_1h, candles_1h, trend_threshold
        )
        timeframe_results.append((tf_1h, reason_1h))

        # 4H
        candles_4h = request.timeframe_data.get("4h", [])
        tf_4h, reason_4h = self._classify_timeframe(
            "4H", indicators_4h, candles_4h, trend_threshold
        )
        timeframe_results.append((tf_4h, reason_4h))

        # Count bullish/bearish timeframes (for logging)
        bullish_count = sum(1 for tf, _ in timeframe_results if tf == "BULLISH")
        bearish_count = sum(1 for tf, _ in timeframe_results if tf == "BEARISH")
        neutral_count = sum(1 for tf, _ in timeframe_results if tf == "NEUTRAL")

        # @spec:FR-SETTINGS-002 - Get confidence settings from Rust API
        confidence_base = get_signal_confidence_base()
        confidence_per_tf = get_signal_confidence_per_tf()

        # ==================== WEIGHTED TIMEFRAME VOTING ====================
        # Higher timeframes have MORE weight (main trend is more important)
        # This prevents counter-trend trades (e.g., shorting during 4H uptrend)
        #
        # Weights: 4H (2.0) > 1H (1.5) > 30M (1.0) > 15M (0.5)
        # Total weight = 5.0
        # ===================================================================
        TIMEFRAME_WEIGHTS = {
            "15M": 0.5,  # Short-term noise, least important
            "30M": 1.0,  # Short-term trend
            "1H": 1.5,  # Medium-term trend
            "4H": 2.0,  # MAIN TREND - most important
        }

        # Calculate weighted scores
        weighted_bullish = 0.0
        weighted_bearish = 0.0
        total_weight = 0.0

        tf_names = ["15M", "30M", "1H", "4H"]
        for i, (tf_class, _) in enumerate(timeframe_results):
            tf_name = tf_names[i]
            weight = TIMEFRAME_WEIGHTS.get(tf_name, 1.0)
            total_weight += weight

            if tf_class == "BULLISH":
                weighted_bullish += weight
            elif tf_class == "BEARISH":
                weighted_bearish += weight

        # Normalize to percentage (0-100%)
        bullish_score = (
            (weighted_bullish / total_weight) * 100 if total_weight > 0 else 0
        )
        bearish_score = (
            (weighted_bearish / total_weight) * 100 if total_weight > 0 else 0
        )

        # SAFETY RULE: Don't trade against main trend (4H)
        # If 4H is BULLISH, don't SHORT. If 4H is BEARISH, don't LONG.
        main_trend = tf_4h  # 4H is the main trend

        # Determine signal using WEIGHTED voting
        # Need 60%+ weighted score to trigger directional signal
        MIN_WEIGHTED_THRESHOLD = 60.0  # 60% weighted agreement

        if bullish_score >= MIN_WEIGHTED_THRESHOLD:
            if main_trend == "BEARISH":
                # 4H is BEARISH but lower TFs bullish = potential reversal, stay NEUTRAL
                signal = "Neutral"
                confidence = 0.45
                logger.info(
                    f"⚠️ {symbol}: Bullish score {bullish_score:.1f}% but 4H is BEARISH - staying NEUTRAL"
                )
            else:
                signal = "Long"
                confidence = min(0.85, confidence_base + (bullish_score / 100) * 0.35)
        elif bearish_score >= MIN_WEIGHTED_THRESHOLD:
            if main_trend == "BULLISH":
                # 4H is BULLISH but lower TFs bearish = pullback, stay NEUTRAL (DON'T SHORT!)
                signal = "Neutral"
                confidence = 0.45
                logger.info(
                    f"⚠️ {symbol}: Bearish score {bearish_score:.1f}% but 4H is BULLISH - staying NEUTRAL (no counter-trend)"
                )
            else:
                signal = "Short"
                confidence = min(0.85, confidence_base + (bearish_score / 100) * 0.35)
        else:
            signal = "Neutral"
            confidence = 0.40

        # Log weighted analysis
        logger.info(
            f"📊 {symbol} Weighted Analysis: Bullish={bullish_score:.1f}%, Bearish={bearish_score:.1f}%, "
            f"4H_Trend={main_trend}, Signal={signal}"
        )

        # Build reasoning string with weighted analysis
        reasons = [reason for _, reason in timeframe_results]
        reasoning += "; ".join(reasons)
        reasoning += (
            f" | Weighted: Bullish={bullish_score:.0f}%, Bearish={bearish_score:.0f}%"
        )
        reasoning += f" | 4H_Trend={main_trend}"
        if main_trend == "BULLISH" and bearish_score >= MIN_WEIGHTED_THRESHOLD:
            reasoning += " | ⚠️ Blocked SHORT (counter-trend protection)"
        elif main_trend == "BEARISH" and bullish_score >= MIN_WEIGHTED_THRESHOLD:
            reasoning += " | ⚠️ Blocked LONG (counter-trend protection)"

        # Debug logging for signal generation
        logger.info(
            f"📊 {symbol} Fallback Signal: {signal} | "
            f"Bullish={bullish_count}, Bearish={bearish_count}, Neutral={neutral_count} | "
            f"Threshold={trend_threshold}%, MinRequired={min_timeframes}"
        )

        # Create strategy scores based on actual indicator values (not keyword matching)
        # Average scores across all available timeframes for more accurate assessment
        all_indicators = [
            ind
            for ind in [indicators_15m, indicators_30m, indicators_1h, indicators_4h]
            if ind
        ]
        selected_strategies = (
            request.strategy_context.selected_strategies
            if request.strategy_context
            else None
        )

        strategy_scores = self._calculate_strategy_scores_from_indicators(
            all_indicators, selected_strategies
        )
        logger.info(f"📊 {symbol} Fallback strategy scores: {strategy_scores}")

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

    def _calculate_strategy_scores_from_indicators(
        self, all_indicators: List[Dict], selected_strategies: List[str] = None
    ) -> Dict[str, float]:
        """
        Calculate strategy scores based on actual indicator values.

        Averages scores across all available timeframes for more accurate assessment.
        Uses the same scoring logic as Grok prompt for consistency.

        Returns scores in range [0.3, 1.0] for all 5 strategies.
        """
        all_strategies = [
            "RSI Strategy",
            "MACD Strategy",
            "Volume Strategy",
            "Bollinger Bands Strategy",
            "Stochastic Strategy",
        ]

        strategy_scores = {}

        for strategy in all_strategies:
            if selected_strategies and strategy not in selected_strategies:
                # Low score for non-selected strategies
                strategy_scores[strategy] = 0.1
                continue

            scores_per_tf = []

            for indicators in all_indicators:
                if not indicators:
                    continue

                if strategy == "RSI Strategy":
                    rsi = indicators.get("rsi", 50)
                    # 0.3 if neutral (45-55), 0.5-0.7 if moderate, 0.8-1.0 if extreme
                    if 45 <= rsi <= 55:
                        score = 0.3
                    elif rsi < 30 or rsi > 70:
                        score = 0.8 + min(0.2, abs(rsi - 50) / 50)  # Max 1.0
                    else:
                        score = 0.5 + (abs(rsi - 50) - 5) / 30 * 0.2  # 0.5-0.7
                    scores_per_tf.append(min(1.0, score))

                elif strategy == "MACD Strategy":
                    macd_hist = abs(indicators.get("macd_histogram", 0))
                    # 0.3 if histogram~0, 0.5-0.7 if moderate, 0.8-1.0 if strong
                    if macd_hist < 0.001:
                        score = 0.3
                    elif macd_hist > 0.01:
                        score = 0.8 + min(0.2, macd_hist / 0.05)  # Max 1.0
                    else:
                        score = 0.5 + (macd_hist / 0.01) * 0.2  # 0.5-0.7
                    scores_per_tf.append(min(1.0, score))

                elif strategy == "Volume Strategy":
                    vol_ratio = indicators.get("volume_ratio", 1.0)
                    # 0.3 if ratio<1.0, 0.5-0.7 if 1.0-1.5x, 0.8-1.0 if >1.5x
                    if vol_ratio < 1.0:
                        score = 0.3
                    elif vol_ratio > 1.5:
                        score = 0.8 + min(0.2, (vol_ratio - 1.5) / 2)  # Max 1.0
                    else:
                        score = 0.5 + (vol_ratio - 1.0) * 0.4  # 0.5-0.7
                    scores_per_tf.append(min(1.0, score))

                elif strategy == "Bollinger Bands Strategy":
                    bb_pos = indicators.get("bb_position", 0.5)
                    # 0.3 if middle band (0.4-0.6), 0.5-0.7 if near bands, 0.8-1.0 if touching
                    dist_from_center = abs(bb_pos - 0.5)
                    if dist_from_center < 0.1:
                        score = 0.3
                    elif dist_from_center > 0.4:  # Near/outside bands
                        score = 0.8 + min(0.2, (dist_from_center - 0.4) / 0.3)
                    else:
                        score = 0.5 + (dist_from_center - 0.1) / 0.3 * 0.2  # 0.5-0.7
                    scores_per_tf.append(min(1.0, score))

                elif strategy == "Stochastic Strategy":
                    stoch_k = indicators.get("stochastic_k", 50)
                    # 0.3 if neutral (30-70), 0.5-0.7 if moderate, 0.8-1.0 if overbought/oversold
                    if 30 <= stoch_k <= 70:
                        # Score based on distance from center (50)
                        dist_from_center = abs(stoch_k - 50)
                        score = 0.3 + (dist_from_center / 20) * 0.2  # Max 0.5
                    elif stoch_k < 20 or stoch_k > 80:
                        score = 0.9 + min(0.1, abs(stoch_k - 50) / 100)
                    else:
                        score = 0.6 + abs(stoch_k - 50) / 50 * 0.2  # 0.6-0.8
                    scores_per_tf.append(min(1.0, score))

            # Average scores across timeframes, default to 0.3 if no data
            if scores_per_tf:
                strategy_scores[strategy] = round(
                    sum(scores_per_tf) / len(scores_per_tf), 2
                )
            else:
                strategy_scores[strategy] = 0.3

        return strategy_scores

    def _get_system_prompt(self) -> str:
        """Get system prompt for Grok with multi-timeframe awareness.

        @spec:FR-SETTINGS-002 - Uses dynamic settings from Rust API
        Config values fetched from Rust API (fallback to config.yaml):
        - min_required_indicators: minimum indicators per timeframe (default 4/5)
        - min_required_timeframes: minimum timeframes needed to agree
        """
        # @spec:FR-SETTINGS-002 - Get settings from Rust API for Grok prompt
        min_indicators = get_signal_min_indicators()
        min_timeframes = get_signal_min_timeframes()
        confidence_base = get_signal_confidence_base()
        confidence_per_tf = get_signal_confidence_per_tf()

        return (
            f"Crypto trading analyst using WEIGHTED MULTI-TIMEFRAME + 5 STRATEGIES analysis.\n"
            f"INDICATOR SCORING (per timeframe - count bullish/bearish points):\n"
            f"- MACD: histogram > 0 = +1 bullish, < 0 = +1 bearish\n"
            f"- RSI: > 55 = +1 bullish, < 45 = +1 bearish (50-55/45-50 = neutral)\n"
            f"- BB: position < 0.3 = +1 bullish (near lower), > 0.7 = +1 bearish (near upper)\n"
            f"- Stochastic: K > D and K < 80 = +1 bullish, K < D and K > 20 = +1 bearish\n"
            f"- Volume: ratio > 1.2x confirms trend direction\n"
            f"TIMEFRAME CLASSIFICATION (STRICT - requires {min_indicators}/5 indicators):\n"
            f"- Timeframe is BULLISH if: {min_indicators}+ indicators are bullish\n"
            f"- Timeframe is BEARISH if: {min_indicators}+ indicators are bearish\n"
            f"- Otherwise NEUTRAL\n"
            f"WEIGHTED TIMEFRAME VOTING (CRITICAL - higher TF = more important):\n"
            f"- 4H weight: 2.0 (MAIN TREND - most important)\n"
            f"- 1H weight: 1.5 (medium-term)\n"
            f"- 30M weight: 1.0 (short-term)\n"
            f"- 15M weight: 0.5 (noise)\n"
            f"- Total weight: 5.0, need 60%+ weighted score for directional signal\n"
            f"COUNTER-TREND PROTECTION (CRITICAL):\n"
            f"- NEVER give SHORT if 4H is BULLISH (pullback in uptrend)\n"
            f"- NEVER give LONG if 4H is BEARISH (bounce in downtrend)\n"
            f"- 4H is the MAIN TREND - always follow it!\n"
            f"SIGNAL DECISION:\n"
            f"- LONG: bullish_weighted_score >= 60% AND 4H NOT BEARISH\n"
            f"- SHORT: bearish_weighted_score >= 60% AND 4H NOT BULLISH\n"
            f"- NEUTRAL: otherwise or counter-trend situation\n"
            f"STRATEGY SCORE CALCULATION (score ALL strategies 0.3-1.0):\n"
            f"- RSI: 0.3 neutral, 0.5-0.7 moderate, 0.8-1.0 extreme\n"
            f"- MACD: 0.3 weak, 0.5-0.7 moderate, 0.8-1.0 strong\n"
            f"- Bollinger: 0.3 middle, 0.5-0.7 near bands, 0.8-1.0 extreme\n"
            f"- Stochastic: 0.3 neutral, 0.5-0.7 moderate, 0.8-1.0 overbought/oversold\n"
            f"- Volume: 0.3 low, 0.5-0.7 moderate, 0.8-1.0 high\n"
            "Respond ONLY in JSON:\n"
            '{"signal":"Long|Short|Neutral","confidence":0-1,"reasoning":"brief",'
            '"strategy_scores":{"RSI Strategy":0.3-1,"MACD Strategy":0.3-1,"Volume Strategy":0.3-1,'
            '"Bollinger Bands Strategy":0.3-1,"Stochastic Strategy":0.3-1},'
            '"market_analysis":{"trend_direction":"Bullish|Bearish|Sideways",'
            '"trend_strength":0-1,"support_levels":[],"resistance_levels":[],'
            '"volatility_level":"Low|Medium|High","volume_analysis":"brief"},'
            '"risk_assessment":{"overall_risk":"Low|Medium|High","technical_risk":0-1,'
            '"market_risk":0-1,"recommended_position_size":0-1,"stop_loss_suggestion":null,'
            '"take_profit_suggestion":null}}\n'
            f"Confidence: {confidence_base} base + (weighted_score/100)*0.35. Max 0.85."
        )

    def _prepare_market_context(
        self,
        request: AIAnalysisRequest,
        indicators_15m: Dict,
        indicators_30m: Dict,
        indicators_1h: Dict,
        indicators_4h: Dict,
    ) -> str:
        """Prepare market context for Grok with multi-timeframe analysis (15m, 30m, 1h, 4h).

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
                f"Vol:{indicators_15m.get('volume_ratio',1):.1f}x "
                f"Stoch:{indicators_15m.get('stochastic_k',50):.1f}/{indicators_15m.get('stochastic_d',50):.1f}\n"
            )

        # 30m - Short-term trend
        if indicators_30m:
            context += (
                f"30M: RSI:{indicators_30m.get('rsi',50):.1f} "
                f"MACD:{indicators_30m.get('macd_histogram',0):.2f} "
                f"BB:{indicators_30m.get('bb_position',0.5):.2f} "
                f"Vol:{indicators_30m.get('volume_ratio',1):.1f}x "
                f"Stoch:{indicators_30m.get('stochastic_k',50):.1f}/{indicators_30m.get('stochastic_d',50):.1f}\n"
            )

        # 1H - Medium-term trend
        context += (
            f"1H: RSI:{indicators_1h.get('rsi',50):.1f} "
            f"MACD:{indicators_1h.get('macd_histogram',0):.2f} "
            f"BB:{indicators_1h.get('bb_position',0.5):.2f} "
            f"Vol:{indicators_1h.get('volume_ratio',1):.1f}x "
            f"Stoch:{indicators_1h.get('stochastic_k',50):.1f}/{indicators_1h.get('stochastic_d',50):.1f}"
        )

        # 4H - Long-term trend
        if indicators_4h:
            context += (
                f"\n4H: RSI:{indicators_4h.get('rsi',50):.1f} "
                f"MACD:{indicators_4h.get('macd_histogram',0):.2f} "
                f"BB:{indicators_4h.get('bb_position',0.5):.2f} "
                f"Stoch:{indicators_4h.get('stochastic_k',50):.1f}/{indicators_4h.get('stochastic_d',50):.1f}"
            )

        return context

    def _create_analysis_prompt(
        self, market_context: str, strategy_context: AIStrategyContext
    ) -> str:
        """Create analysis prompt for Grok (optimized for cost)."""
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
        """Parse Grok JSON response."""
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
                "Stochastic Strategy": confidence * 0.85,
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
                "Stochastic Strategy": 0.3,
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
        return {"error": "Failed to get analysis stats. Check server logs for details."}


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
                    params={"limit": 100},  # Need 100 candles for indicators
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
                    logger.warning(
                        f"⚠️ Failed to fetch 1H candles for {symbol}: {response_1h.status_code}"
                    )
            except Exception as e:
                logger.error(f"❌ Error fetching 1H candles for {symbol}: {e}")

            # Fetch 4H candles from Rust API
            try:
                response_4h = await client.get(
                    f"{RUST_API_URL}/api/market/candles/{symbol}/4h",
                    params={"limit": 60},  # Need 60 candles for indicators
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
                    logger.warning(
                        f"⚠️ Failed to fetch 4H candles for {symbol}: {response_4h.status_code}"
                    )
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
        logger.warning(
            f"⚠️ Insufficient 1H data for {symbol}: {len(candles_1h)} candles (need 50+)"
        )
    if len(candles_4h) < 50:
        logger.warning(
            f"⚠️ Insufficient 4H data for {symbol}: {len(candles_4h)} candles (need 50+)"
        )
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
    """Health check endpoint.

    NOTE: This endpoint should be fast and NOT call external APIs (like Rust API)
    because during Docker startup, other services may not be ready yet.
    Calling external APIs here can cause circular dependency issues and
    health check timeouts.
    """

    # Check MongoDB connection with short timeout
    mongodb_status = False
    try:
        if mongodb_client:
            # Use a short timeout to avoid blocking health check
            await asyncio.wait_for(
                mongodb_client.admin.command("ping"),
                timeout=3.0,  # 3 second timeout for MongoDB ping
            )
            mongodb_status = True
    except (Exception, asyncio.TimeoutError):
        pass

    # Use fallback symbols instead of calling Rust API
    # This avoids circular dependency during startup:
    # python-ai-service health check -> calls rust-core-engine
    # but rust-core-engine depends on python-ai-service being healthy first
    current_symbols = FALLBACK_ANALYSIS_SYMBOLS

    return {
        "status": "healthy",
        "timestamp": datetime.now(timezone.utc).isoformat(),
        "service": "Grok Trading AI",
        "version": "3.0.0",
        "grok_available": grok_client is not None,
        "api_key_configured": bool(
            os.getenv("XAI_API_KEY") or os.getenv("OPENAI_API_KEY")
        ),
        "mongodb_connected": mongodb_status,
        "analysis_interval_minutes": ANALYSIS_INTERVAL_MINUTES,
        "supported_symbols": current_symbols,
    }


@app.get("/ai/health/market-condition")
async def deep_health_market_condition():
    """
    Deep health check for the AI market condition pipeline.

    Trả 503 nếu BẤT KỲ bước nào fail:
    - MongoDB connectivity + candle fetch
    - DataFrame creation (field name correctness)
    - TechnicalAnalyzer.calculate_indicators()
    - _get_macd_series() + _calculate_direction_score()
    """
    try:
        if mongodb_db is None:
            raise HTTPException(status_code=503, detail="MongoDB not connected")

        candles_collection = mongodb_db.market_data
        cursor = (
            candles_collection.find(
                {"symbol": "BTCUSDT", "timeframe": "1h"}, {"_id": 0}
            )
            .sort("open_time", ASCENDING)
            .limit(50)
        )
        candles = await asyncio.wait_for(cursor.to_list(length=50), timeout=5.0)

        if len(candles) < 20:
            raise HTTPException(
                status_code=503,
                detail=f"Insufficient candle data: {len(candles)} candles (need 20+)",
            )

        data = []
        for c in candles:
            close = float(c.get("close_price", c.get("close", 0)))
            if close == 0:
                raise HTTPException(
                    status_code=503,
                    detail="Candle data has zero close price — wrong field names",
                )
            data.append(
                {
                    "timestamp": pd.to_datetime(c.get("open_time", 0), unit="ms"),
                    "open": float(c.get("open_price", c.get("open", 0))),
                    "high": float(c.get("high_price", c.get("high", 0))),
                    "low": float(c.get("low_price", c.get("low", 0))),
                    "close": close,
                    "volume": float(c.get("volume", 0)),
                }
            )

        df = pd.DataFrame(data)
        df.set_index("timestamp", inplace=True)
        df.sort_index(inplace=True)

        indicators = TechnicalAnalyzer.calculate_indicators(df)
        macd_series = _get_macd_series(df)
        direction = _calculate_direction_score(df, indicators, macd_series)

        return {
            "status": "healthy",
            "pipeline": "ok",
            "test_direction": round(direction, 4),
            "candles_fetched": len(candles),
            "timestamp": datetime.now(timezone.utc).isoformat(),
        }

    except HTTPException:
        raise
    except asyncio.TimeoutError:
        raise HTTPException(
            status_code=503, detail="MongoDB timeout — candle fetch took >5s"
        )
    except Exception as e:
        raise HTTPException(
            status_code=503, detail=f"Market condition pipeline error: {str(e)}"
        )


@app.get("/debug/gpt4")
@limiter.limit("60/minute")  # Rate limit: 60 requests per minute for debug endpoint
async def debug_gpt4(request: Request):
    """Debug Grok connectivity."""
    result: Dict[str, Any] = {
        "client_initialized": grok_client is not None,
        "api_key_configured": bool(os.getenv("OPENAI_API_KEY")),
    }

    if grok_client is None:
        result["error"] = "Grok client not initialized"
        result["status"] = "failed"
        return result

    try:
        # Test simple API call
        logger.info("🧪 Testing GPT-5-mini API connection...")
        response = await grok_client.chat_completions_create(
            model=AI_MODEL,
            messages=[
                {"role": "user", "content": "Respond with just the word 'SUCCESS'"}
            ],
            max_tokens=10,
            temperature=0,
        )

        result["status"] = "success"
        result["test_response"] = response["choices"][0]["message"]["content"]
        result["model_used"] = AI_MODEL
        logger.info("✅ GPT-5-mini test successful")

    except Exception as e:
        result["status"] = "failed"
        result["error"] = "Grok test failed. Check server logs for details."
        result["error_type"] = type(e).__name__
        logger.error(f"❌ Grok test failed: {e}")

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


# @spec:FR-AI-005 - Grok Signal Analysis
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
    """Analyze trading signals using Grok AI with MongoDB storage."""
    global gpt_analyzer

    if not gpt_analyzer:
        grok_analyzer = GrokTradingAnalyzer(grok_client)
        logger.info(
            f"🤖 Grok analyzer created with client: {'Available' if grok_client else 'None'}"
        )

    logger.info(f"🤖 Grok analysis request for {analysis_request.symbol}")
    logger.debug(
        f"📋 Request details: strategies={analysis_request.strategy_context.selected_strategies}, "
        f"timeframes={list(analysis_request.timeframe_data.keys())}"
    )

    # Check Grok availability
    if grok_client is None:
        logger.warning("⚠️ Grok client is None - will use fallback analysis")
    else:
        logger.info("✅ Grok client available - attempting AI analysis")

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
                        "model_type": "AI-Stored",
                        "timeframe": "1h",
                        "reasoning": stored_response.reasoning,
                        "strategy_scores": stored_response.strategy_scores,
                    }
                    await ws_manager.broadcast_signal(signal_data)

                return stored_response

        # No recent analysis found, perform fresh AI analysis
        logger.info(
            f"🔥 No recent analysis found. Calling Grok for {analysis_request.symbol}"
        )
        response = await grok_analyzer.analyze_trading_signals(analysis_request)
        logger.info(
            f"✅ Grok signal: {response.signal} (confidence: {response.confidence:.2f})"
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
                "model_type": "Grok-AI",
                "timeframe": "1h",  # Default timeframe for WebSocket compatibility
                "reasoning": response.reasoning,
                "strategy_scores": response.strategy_scores,
            }
            await ws_manager.broadcast_signal(signal_data)

        return response
    except Exception as e:
        logger.error(f"❌ Analysis failed: {e}")
        raise HTTPException(
            status_code=500, detail="Analysis failed. Check server logs for details."
        )


@app.post("/ai/strategy-recommendations", response_model=List[StrategyRecommendation])
@limiter.limit("300/minute")  # Rate limit: 300 requests per minute (5 per second)
async def get_strategy_recommendations(
    body: StrategyRecommendationRequest, request: Request
):
    """Get AI strategy recommendations."""
    logger.info(f"📊 Strategy recommendations for {body.symbol}")

    # Simple recommendations based on available strategies
    recommendations = []
    for strategy in body.available_strategies:
        score = 0.7 + (hash(strategy + body.symbol) % 30) / 100  # Pseudo-random score
        recommendations.append(
            StrategyRecommendation(
                strategy_name=strategy,
                suitability_score=min(score, 0.95),
                reasoning=f"{strategy} shows good potential for {body.symbol} based on current market conditions",
                recommended_config={"enabled": True, "weight": score},
            )
        )

    return sorted(recommendations, key=lambda x: x.suitability_score, reverse=True)


def _clamp(value: float, min_val: float, max_val: float) -> float:
    """Clamp a value between min and max."""
    return max(min_val, min(max_val, value))


def _sign_scale(value: float, threshold: float) -> float:
    """Scale a value based on threshold: maps to roughly -1.0 to +1.0."""
    if threshold <= 0:
        return 0.0
    scaled = value / threshold
    return _clamp(scaled, -1.0, 1.0)


def _get_macd_series(df: pd.DataFrame) -> Dict[str, float]:
    """Get MACD series data including previous histogram value."""
    try:
        macd_line = ta.trend.macd(df["close"])
        macd_signal = ta.trend.macd_signal(df["close"])
        histogram = macd_line - macd_signal
        return {
            "histogram_current": (
                float(histogram.iloc[-1]) if len(histogram) >= 1 else 0.0
            ),
            "histogram_prev": float(histogram.iloc[-2]) if len(histogram) >= 2 else 0.0,
            "macd_current": float(macd_line.iloc[-1]) if len(macd_line) >= 1 else 0.0,
            "signal_current": (
                float(macd_signal.iloc[-1]) if len(macd_signal) >= 1 else 0.0
            ),
        }
    except Exception:
        return {
            "histogram_current": 0.0,
            "histogram_prev": 0.0,
            "macd_current": 0.0,
            "signal_current": 0.0,
        }


def _calculate_direction_score(
    df: pd.DataFrame, indicators: Dict[str, Any], macd_series: Dict[str, float]
) -> float:
    """Calculate direction score for a single timeframe from -1.0 to +1.0."""
    current_price = float(df["close"].iloc[-1])

    # --- EMA Trend (25%) ---
    ema_50 = float(indicators.get("ema_50", current_price))
    # Calculate EMA 200 if enough data, otherwise use EMA 50 as fallback
    if len(df) >= 200:
        ema_200 = float(ta.trend.ema_indicator(df["close"], window=200).iloc[-1])
    else:
        ema_200 = ema_50  # Fallback: treat EMA200 same as EMA50

    price_vs_ema50 = (current_price - ema_50) / ema_50 if ema_50 > 0 else 0.0
    price_vs_ema200 = (current_price - ema_200) / ema_200 if ema_200 > 0 else 0.0
    ema_cross = (ema_50 - ema_200) / ema_200 if ema_200 > 0 else 0.0

    ema_score = _clamp(
        0.4 * _sign_scale(price_vs_ema50, 0.01)
        + 0.3 * _sign_scale(price_vs_ema200, 0.02)
        + 0.3 * _sign_scale(ema_cross, 0.01),
        -1.0,
        1.0,
    )

    # --- MACD (20%) ---
    hist = macd_series["histogram_current"]
    hist_prev = macd_series["histogram_prev"]

    if hist > 0 and hist > hist_prev:
        macd_score = 0.9
    elif hist > 0 and hist <= hist_prev:
        macd_score = 0.3
    elif hist < 0 and hist > hist_prev:
        macd_score = -0.3
    elif hist < 0 and hist <= hist_prev:
        macd_score = -0.9
    else:
        macd_score = 0.0

    # --- RSI Momentum (15%) ---
    rsi = float(indicators.get("rsi", 50.0))
    if rsi > 70:
        rsi_score = 0.3
    elif rsi > 55:
        rsi_score = 0.7
    elif rsi > 45:
        rsi_score = 0.0
    elif rsi > 30:
        rsi_score = -0.7
    else:
        rsi_score = -0.3

    # --- Price Action (15%) ---
    if len(df) >= 20:
        mom_20 = current_price / float(df["close"].iloc[-20]) - 1
    else:
        mom_20 = 0.0
    if len(df) >= 10:
        mom_10 = current_price / float(df["close"].iloc[-10]) - 1
    else:
        mom_10 = 0.0
    price_score = _clamp(
        0.5 * _sign_scale(mom_10, 0.03) + 0.5 * _sign_scale(mom_20, 0.05),
        -1.0,
        1.0,
    )

    # --- Volume (10%) ---
    volume_ratio = float(indicators.get("volume_ratio", 1.0))
    # Volume confirms direction: high volume + price up = bullish confirmation
    if len(df) >= 2:
        price_dir = (
            1.0 if float(df["close"].iloc[-1]) > float(df["close"].iloc[-2]) else -1.0
        )
    else:
        price_dir = 0.0
    vol_factor = min(volume_ratio, 3.0) / 3.0  # Normalize to 0-1
    volume_score = _clamp(price_dir * vol_factor, -1.0, 1.0)

    # --- ADX/Stochastic (15%) ---
    stoch_k = float(indicators.get("stochastic_k", 50.0))
    stoch_d = float(indicators.get("stochastic_d", 50.0))
    adx = float(indicators.get("adx", 25.0))

    # Stochastic direction
    if stoch_k > stoch_d:
        stoch_score = _clamp((stoch_k - stoch_d) / 20.0, 0.0, 1.0)
    else:
        stoch_score = _clamp((stoch_k - stoch_d) / 20.0, -1.0, 0.0)

    # ADX amplifies the stochastic signal (strong trend = more weight)
    adx_factor = min(adx, 50.0) / 50.0  # Normalize to 0-1
    stoch_adx_score = _clamp(stoch_score * (0.5 + 0.5 * adx_factor), -1.0, 1.0)

    # --- Weighted combination ---
    direction = (
        0.25 * ema_score
        + 0.20 * macd_score
        + 0.15 * rsi_score
        + 0.15 * price_score
        + 0.10 * volume_score
        + 0.15 * stoch_adx_score
    )

    return _clamp(direction, -1.0, 1.0)


def _combine_timeframe_directions(tf_results: Dict[str, Dict[str, Any]]) -> float:
    """Combine direction scores across timeframes with weighting."""
    weights = {"1d": 0.45, "4h": 0.35, "1h": 0.20}
    total_weight = 0.0
    weighted_direction = 0.0

    for tf, data in tf_results.items():
        w = weights.get(tf, 0.15)
        weighted_direction += w * data["direction"]
        total_weight += w

    if total_weight > 0:
        return _clamp(weighted_direction / total_weight, -1.0, 1.0)
    return 0.0


def _calculate_confidence(tf_results: Dict[str, Dict[str, Any]]) -> float:
    """Calculate real confidence from indicator agreement and cross-timeframe consistency."""
    if not tf_results:
        return 0.2

    # 1. Indicator agreement within each timeframe
    tf_agreements = []
    for tf, data in tf_results.items():
        indicators = data.get("indicators", {})
        scores = []

        # Reconstruct component signs from indicators
        ema_50 = indicators.get("ema_50", 0)
        current_price = data.get("current_price", ema_50)
        if ema_50 > 0 and current_price > 0:
            scores.append(1.0 if current_price > ema_50 else -1.0)

        macd_hist = indicators.get("macd_histogram", 0)
        scores.append(1.0 if macd_hist > 0 else -1.0)

        rsi = indicators.get("rsi", 50)
        scores.append(1.0 if rsi > 50 else -1.0)

        volume_ratio = indicators.get("volume_ratio", 1.0)
        scores.append(1.0 if volume_ratio > 1.0 else -1.0)

        stoch_k = indicators.get("stochastic_k", 50)
        stoch_d = indicators.get("stochastic_d", 50)
        scores.append(1.0 if stoch_k > stoch_d else -1.0)

        if scores:
            pos = sum(1 for s in scores if s > 0)
            neg = sum(1 for s in scores if s < 0)
            agreement = max(pos, neg) / len(scores)
            tf_agreements.append(agreement)

    base_agreement = sum(tf_agreements) / len(tf_agreements) if tf_agreements else 0.5

    # 2. Cross-timeframe agreement
    directions = [data["direction"] for data in tf_results.values()]
    if len(directions) >= 2:
        all_positive = all(d > 0 for d in directions)
        all_negative = all(d < 0 for d in directions)
        cross_tf = 1.0 if (all_positive or all_negative) else 0.4
    else:
        cross_tf = 0.6

    # 3. ADX boost (stronger trend = higher confidence)
    adx_values = [
        data.get("indicators", {}).get("adx", 25.0) for data in tf_results.values()
    ]
    avg_adx = sum(adx_values) / len(adx_values) if adx_values else 25.0
    adx_factor = min(avg_adx, 50.0) / 50.0

    confidence = base_agreement * 0.6 + cross_tf * 0.3 + adx_factor * 0.1
    return _clamp(confidence, 0.1, 0.95)


def _direction_to_condition(direction: float) -> str:
    """Map direction score to condition type string."""
    if direction >= 0.50:
        return "Strong Bullish"
    elif direction >= 0.20:
        return "Mildly Bullish"
    elif direction >= -0.19:
        return "Neutral"
    elif direction >= -0.49:
        return "Mildly Bearish"
    else:
        return "Strong Bearish"


def _avg_adx_to_strength(tf_results: Dict[str, Dict[str, Any]]) -> float:
    """Convert average ADX across timeframes to trend strength 0-1."""
    adx_values = [
        data.get("indicators", {}).get("adx", 25.0) for data in tf_results.values()
    ]
    if not adx_values:
        return 0.0
    avg_adx = sum(adx_values) / len(adx_values)
    return _clamp(avg_adx / 50.0, 0.0, 1.0)


def _determine_market_phase(
    tf_results: Dict[str, Dict[str, Any]], direction: float
) -> str:
    """Determine market phase from ADX and direction."""
    adx_values = [
        data.get("indicators", {}).get("adx", 25.0) for data in tf_results.values()
    ]
    avg_adx = sum(adx_values) / len(adx_values) if adx_values else 25.0

    if avg_adx > 30 and abs(direction) > 0.5:
        return "Strong Trend"
    elif avg_adx > 25 and abs(direction) > 0.3:
        return "Trending"
    elif avg_adx < 20:
        return "Ranging/Consolidation"
    else:
        return "Transitioning"


def _build_characteristics(
    tf_results: Dict[str, Dict[str, Any]], direction: float
) -> List[str]:
    """Build human-readable characteristics list."""
    chars = []
    if direction > 0.3:
        chars.append("Bullish momentum across timeframes")
    elif direction < -0.3:
        chars.append("Bearish pressure across timeframes")
    else:
        chars.append("Mixed signals / sideways action")

    # Check volume
    vol_ratios = [
        data.get("indicators", {}).get("volume_ratio", 1.0)
        for data in tf_results.values()
    ]
    avg_vol = sum(vol_ratios) / len(vol_ratios) if vol_ratios else 1.0
    if avg_vol > 1.5:
        chars.append("High volume confirming move")
    elif avg_vol < 0.7:
        chars.append("Low volume - weak conviction")

    # Check ADX
    adx_values = [
        data.get("indicators", {}).get("adx", 25.0) for data in tf_results.values()
    ]
    avg_adx = sum(adx_values) / len(adx_values) if adx_values else 25.0
    if avg_adx > 30:
        chars.append("Strong trend (ADX > 30)")
    elif avg_adx < 20:
        chars.append("Weak trend (ADX < 20)")

    return chars


def _recommend_strategies(direction: float, confidence: float) -> List[str]:
    """Recommend strategies based on direction and confidence."""
    if abs(direction) > 0.5 and confidence > 0.6:
        return ["MACD Strategy", "EMA Crossover", "Trend Following"]
    elif abs(direction) < 0.2:
        return ["Bollinger Bands", "RSI Strategy", "Mean Reversion"]
    else:
        return ["RSI Strategy", "MACD Strategy", "Stochastic Strategy"]


async def _fetch_candles_from_db(symbol: str) -> Dict[str, pd.DataFrame]:
    """Fetch candle data from MongoDB for multiple timeframes."""
    if mongodb_db is None:
        logger.warning("MongoDB not available for market condition analysis")
        return {}

    candles_collection = mongodb_db.market_data
    timeframes = {"1h": 100, "4h": 100, "1d": 250}
    dataframes = {}

    for tf, limit in timeframes.items():
        try:
            cursor = (
                candles_collection.find({"symbol": symbol, "timeframe": tf}, {"_id": 0})
                .sort("open_time", ASCENDING)
                .limit(limit)
            )
            candles = await cursor.to_list(length=limit)

            if len(candles) < 20:
                logger.debug(
                    f"Insufficient {tf} data for {symbol}: {len(candles)} candles (need 20+)"
                )
                continue

            data = []
            for c in candles:
                data.append(
                    {
                        "timestamp": pd.to_datetime(c.get("open_time", 0), unit="ms"),
                        "open": float(c.get("open_price", c.get("open", 0))),
                        "high": float(c.get("high_price", c.get("high", 0))),
                        "low": float(c.get("low_price", c.get("low", 0))),
                        "close": float(c.get("close_price", c.get("close", 0))),
                        "volume": float(c.get("volume", 0)),
                    }
                )

            df = pd.DataFrame(data)
            df.set_index("timestamp", inplace=True)
            df.sort_index(inplace=True)
            dataframes[tf] = df
            logger.debug(f"Fetched {len(df)} {tf} candles for {symbol} from DB")

        except Exception as e:
            logger.warning(f"Error fetching {tf} candles for {symbol}: {e}")
            continue

    return dataframes


@app.post("/ai/market-condition", response_model=MarketConditionAnalysis)
@limiter.limit("300/minute")  # Rate limit: 300 requests per minute (5 per second)
async def analyze_market_condition(body: MarketConditionRequest, request: Request):
    """Analyze market condition using multi-indicator direction analysis."""
    logger.info(f"🔍 Market condition analysis for {body.symbol}")

    # 1. Get candle data: use provided or fetch from MongoDB
    if body.timeframe_data:
        dataframes = TechnicalAnalyzer.candles_to_dataframe(body.timeframe_data)
    else:
        dataframes = await _fetch_candles_from_db(body.symbol)

    if not dataframes:
        logger.warning(f"No data available for {body.symbol} market condition")
        return MarketConditionAnalysis(
            condition_type="Neutral",
            confidence=0.2,
            direction=0.0,
            trend_strength=0.0,
            characteristics=["Insufficient data for analysis"],
            recommended_strategies=["RSI Strategy", "MACD Strategy"],
            market_phase="Unknown",
        )

    # 2. Calculate indicators + direction per timeframe
    tf_results = {}
    for tf, df in dataframes.items():
        if len(df) < 20:
            continue
        try:
            indicators = TechnicalAnalyzer.calculate_indicators(df)
            macd_series = _get_macd_series(df)
            direction = _calculate_direction_score(df, indicators, macd_series)
            tf_results[tf] = {
                "direction": direction,
                "indicators": indicators,
                "macd_series": macd_series,
                "current_price": float(df["close"].iloc[-1]),
            }
        except Exception as e:
            logger.warning(f"Error analyzing {tf} for {body.symbol}: {e}")
            continue

    if not tf_results:
        logger.warning(f"Could not analyze any timeframe for {body.symbol}")
        return MarketConditionAnalysis(
            condition_type="Neutral",
            confidence=0.2,
            direction=0.0,
            trend_strength=0.0,
            characteristics=["Analysis failed for all timeframes"],
            recommended_strategies=["RSI Strategy", "MACD Strategy"],
            market_phase="Unknown",
        )

    # 3. Combine across timeframes
    final_direction = _combine_timeframe_directions(tf_results)

    # 4. Calculate real confidence
    confidence = _calculate_confidence(tf_results)

    # 5. Build response
    condition_type = _direction_to_condition(final_direction)
    trend_strength = _avg_adx_to_strength(tf_results)
    market_phase = _determine_market_phase(tf_results, final_direction)
    characteristics = _build_characteristics(tf_results, final_direction)
    recommended_strategies = _recommend_strategies(final_direction, confidence)

    # Build timeframe analysis breakdown
    timeframe_analysis = {}
    for tf, data in tf_results.items():
        timeframe_analysis[tf] = {
            "direction": round(data["direction"], 4),
            "rsi": round(data["indicators"].get("rsi", 50.0), 2),
            "macd_histogram": round(data["indicators"].get("macd_histogram", 0.0), 4),
            "adx": round(data["indicators"].get("adx", 25.0), 2),
            "ema_50": round(data["indicators"].get("ema_50", 0.0), 2),
            "stoch_k": round(data["indicators"].get("stochastic_k", 50.0), 2),
        }

    # Build indicators summary
    primary_tf = "4h" if "4h" in tf_results else list(tf_results.keys())[0]
    primary_indicators = tf_results[primary_tf]["indicators"]
    indicators_summary = {
        "primary_timeframe": primary_tf,
        "rsi": round(primary_indicators.get("rsi", 50.0), 2),
        "macd_histogram": round(primary_indicators.get("macd_histogram", 0.0), 4),
        "adx": round(primary_indicators.get("adx", 25.0), 2),
        "volume_ratio": round(primary_indicators.get("volume_ratio", 1.0), 2),
        "stochastic_k": round(primary_indicators.get("stochastic_k", 50.0), 2),
    }

    logger.info(
        f"✅ Market condition for {body.symbol}: {condition_type} "
        f"(direction={final_direction:.3f}, confidence={confidence:.3f}, "
        f"trend_strength={trend_strength:.3f}, timeframes={list(tf_results.keys())})"
    )

    return MarketConditionAnalysis(
        condition_type=condition_type,
        confidence=round(confidence, 4),
        direction=round(final_direction, 4),
        trend_strength=round(trend_strength, 4),
        characteristics=characteristics,
        recommended_strategies=recommended_strategies,
        market_phase=market_phase,
        timeframe_analysis=timeframe_analysis,
        indicators_summary=indicators_summary,
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
@limiter.limit(
    "600/minute"
)  # Rate limit: 600 requests per minute (10 per second) - main prediction endpoint
async def predict_trend(body: TrendPredictionRequest, request: Request):
    """
    Predict trend direction using Grok powered multi-timeframe analysis.

    This endpoint uses Grok to analyze market data across multiple timeframes:
    - Daily (1d): Major trend direction
    - 4H: Intermediate trend
    - Requested timeframe: Short-term signals

    Grok considers: EMA200, momentum, RSI, MACD, volume, and price action.
    Falls back to technical analysis if Grok unavailable.
    """
    logger.info(
        f"🔮 Grok trend prediction request for {body.symbol} on {body.timeframe}"
    )

    try:
        # Fetch market data from MongoDB
        if mongodb_db is None:
            raise HTTPException(status_code=503, detail="Database not available")

        candles_collection = mongodb_db.market_data

        # Fetch multi-timeframe data for comprehensive analysis
        timeframes = {
            "1d": 250,  # Daily for major trend
            "4h": 250,  # 4H for intermediate trend
            body.timeframe: 250,  # Requested timeframe
        }

        candles_by_tf = {}
        for tf, limit in timeframes.items():
            cursor = (
                candles_collection.find(
                    {"symbol": body.symbol, "timeframe": tf}, {"_id": 0}
                )
                .sort("open_time", ASCENDING)
                .limit(limit)
            )

            candles = await cursor.to_list(length=limit)
            if len(candles) >= 50:  # Minimum data requirement
                candles_by_tf[tf] = candles

        if len(candles_by_tf) == 0:
            logger.warning(f"⚠️ Insufficient data for {body.symbol}")
            return TrendPredictionResponse(
                trend="Neutral",
                confidence=0.3,
                model="Insufficient Data",
                timestamp=int(datetime.now(timezone.utc).timestamp()),
            )

        # Try Grok analysis first
        if grok_client is not None:
            try:
                result = await _predict_trend_gpt4(body.symbol, candles_by_tf)
                logger.info(
                    f"✅ Grok trend prediction for {body.symbol}: {result['trend']} "
                    f"(confidence: {result['confidence']:.2f})"
                )
                return TrendPredictionResponse(
                    trend=result["trend"],
                    confidence=result["confidence"],
                    model=AI_MODEL,
                    timestamp=int(datetime.now(timezone.utc).timestamp()),
                )
            except Exception as e:
                logger.warning(
                    f"⚠️ Grok analysis failed, falling back to technical: {e}"
                )

        # Fallback to technical analysis
        result = _predict_trend_technical(body.symbol, candles_by_tf, body.timeframe)
        logger.info(
            f"✅ Technical trend prediction for {body.symbol}: {result['trend']} "
            f"(confidence: {result['confidence']:.2f})"
        )

        return TrendPredictionResponse(
            trend=result["trend"],
            confidence=result["confidence"],
            model="EMA200-Technical-Fallback",
            timestamp=int(datetime.now(timezone.utc).timestamp()),
        )

    except Exception as e:
        logger.error(f"❌ Error predicting trend for {body.symbol}: {e}")
        raise HTTPException(
            status_code=500,
            detail="Failed to predict trend. Check server logs for details.",
        )


async def _predict_trend_gpt4(
    symbol: str, candles_by_tf: Dict[str, List]
) -> Dict[str, Any]:
    """
    Use Grok to predict trend direction based on multi-timeframe analysis.

    Args:
        symbol: Trading symbol (e.g., BTCUSDT)
        candles_by_tf: Dict of timeframe -> candles data

    Returns:
        Dict with trend, confidence, reasoning
    """
    # Calculate indicators for each timeframe
    indicators_by_tf = {}

    for tf, candles in candles_by_tf.items():
        # Normalize MongoDB field names (close_price -> close, etc.)
        normalized = []
        for c in candles:
            normalized.append(
                {
                    "open_time": c.get("open_time", 0),
                    "open": float(c.get("open_price", c.get("open", 0))),
                    "high": float(c.get("high_price", c.get("high", 0))),
                    "low": float(c.get("low_price", c.get("low", 0))),
                    "close": float(c.get("close_price", c.get("close", 0))),
                    "volume": float(c.get("volume", 0)),
                }
            )
        df = pd.DataFrame(normalized)
        df = df.sort_values("open_time")

        # Calculate key indicators
        df["ema_200"] = (
            df["close"].ewm(span=200, adjust=False).mean()
            if len(df) >= 200
            else df["close"]
        )
        df["ema_50"] = (
            df["close"].ewm(span=50, adjust=False).mean()
            if len(df) >= 50
            else df["close"]
        )

        current_price = df["close"].iloc[-1]
        ema_200 = df["ema_200"].iloc[-1] if len(df) >= 200 else current_price
        ema_50 = df["ema_50"].iloc[-1] if len(df) >= 50 else current_price

        # Price distance from EMAs
        distance_200 = ((current_price - ema_200) / ema_200) * 100 if ema_200 > 0 else 0
        distance_50 = ((current_price - ema_50) / ema_50) * 100 if ema_50 > 0 else 0

        # Momentum (last 20 periods)
        momentum_20 = (
            ((df["close"].iloc[-1] / df["close"].iloc[-20]) - 1) * 100
            if len(df) >= 20
            else 0
        )

        # Volume trend
        volume_ma = (
            df["volume"].rolling(20).mean().iloc[-1]
            if len(df) >= 20
            else df["volume"].iloc[-1]
        )
        volume_ratio = df["volume"].iloc[-1] / volume_ma if volume_ma > 0 else 1.0

        # RSI if enough data
        # @spec:FR-SETTINGS-001 - Use dynamic RSI period from Rust API
        rsi_period = get_rsi_period()
        if len(df) >= rsi_period:
            delta = df["close"].diff()
            gain = (delta.where(delta > 0, 0)).rolling(window=rsi_period).mean()
            loss = (-delta.where(delta < 0, 0)).rolling(window=rsi_period).mean()
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
            "last_5_closes": [float(x) for x in df["close"].tail(5).tolist()],
        }

    # Build Grok prompt
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

    # Call Grok
    response = await grok_client.chat_completions_create(
        model=AI_MODEL,
        messages=[
            {
                "role": "system",
                "content": "You are an expert cryptocurrency technical analyst. Analyze trends conservatively and explain your reasoning clearly. Always respond with valid JSON.",
            },
            {"role": "user", "content": prompt},
        ],
        temperature=0.0,  # Deterministic
        max_tokens=400,  # Short response = cheaper
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

        logger.info(
            f"💰 Trend prediction cost: ${cost:.5f} | Tokens: {input_tokens}+{output_tokens} | Total: ${total_cost_usd:.3f}"
        )

    # Parse JSON from response (handle markdown code blocks if present)
    import re

    json_match = re.search(r"\{.*\}", response_content, re.DOTALL)
    if json_match:
        result = json.loads(json_match.group())
    else:
        result = json.loads(response_content)

    return result


def _format_tf_data(tf_name: str, indicators: Dict) -> str:
    """Format timeframe data for Grok prompt."""
    if not indicators:
        return f"{tf_name}: No data available"

    return f"""- Current Price: ${indicators.get('current_price', 0):.2f}
- EMA200: ${indicators.get('ema_200', 0):.2f} (distance: {indicators.get('distance_from_ema200', 0):+.2f}%)
- EMA50: ${indicators.get('ema_50', 0):.2f} (distance: {indicators.get('distance_from_ema50', 0):+.2f}%)
- Momentum (20 periods): {indicators.get('momentum_20', 0):+.2f}%
- RSI: {indicators.get('rsi', 50):.1f}
- Volume Ratio: {indicators.get('volume_ratio', 1.0):.2f}x
- Last 5 closes: {indicators.get('last_5_closes', [])}"""


def _predict_trend_technical(
    symbol: str, candles_by_tf: Dict[str, List], primary_tf: str
) -> Dict[str, Any]:
    """
    Fallback technical analysis when Grok is unavailable.
    Uses simple EMA200 + momentum logic.
    """
    # Use primary timeframe or fallback to first available
    tf = primary_tf if primary_tf in candles_by_tf else list(candles_by_tf.keys())[0]
    candles = candles_by_tf[tf]

    # Normalize MongoDB field names (close_price -> close, etc.)
    normalized = []
    for c in candles:
        normalized.append(
            {
                "open_time": c.get("open_time", 0),
                "open": float(c.get("open_price", c.get("open", 0))),
                "high": float(c.get("high_price", c.get("high", 0))),
                "low": float(c.get("low_price", c.get("low", 0))),
                "close": float(c.get("close_price", c.get("close", 0))),
                "volume": float(c.get("volume", 0)),
            }
        )
    df = pd.DataFrame(normalized)
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

    return {"trend": trend, "confidence": confidence, "reasoning": reasoning}


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
    """Get Grok API cost statistics."""
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
            "model": AI_MODEL,
            "analysis_interval_minutes": ANALYSIS_INTERVAL_MINUTES,
            "symbols_tracked": len(current_symbols),
            "cache_duration_minutes": 15,  # Updated cache duration
            "max_tokens": 1200,  # Optimized max tokens
            "input_cost_per_1m_tokens": AI_INPUT_COST_PER_1M,
            "output_cost_per_1m_tokens": AI_OUTPUT_COST_PER_1M,
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
        logger.error(f"Failed to get storage stats: {e}")
        return {"error": "Failed to get storage stats. Check server logs for details."}


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
        logger.error(f"Failed to clear storage: {e}")
        return {"error": "Failed to clear storage. Check server logs for details."}


# === CONFIG SUGGESTIONS ENDPOINTS ===


@app.post("/ai/config-analysis/trigger")
@limiter.limit("10/minute")  # Rate limit: 10 requests per minute (Grok API call)
async def trigger_config_analysis(request: Request):
    """
    Trigger Grok config analysis DIRECTLY (bypass Celery/Redis).

    This endpoint runs Grok to analyze current trading config and suggest improvements.
    Use this when:
    - Redis/Celery is unavailable
    - You want to manually trigger config analysis
    - Testing config optimization

    Returns:
        Config improvement suggestions from Grok
    """
    try:
        from tasks.ai_improvement import _run_config_analysis_direct

        logger.info("🧠 Manually triggering config analysis...")
        result = _run_config_analysis_direct(analysis_result=None)

        if result.get("status") == "success":
            # Sanitize output - only return known safe fields
            suggestions = result.get("suggestions")
            trade_stats = result.get("trade_stats")
            timestamp = result.get("timestamp")
            return {
                "success": True,
                "message": "Config analysis completed successfully",
                "suggestions": (
                    suggestions
                    if isinstance(suggestions, (list, dict, type(None)))
                    else None
                ),
                "trade_stats": (
                    trade_stats if isinstance(trade_stats, (dict, type(None))) else None
                ),
                "timestamp": str(timestamp) if timestamp else None,
            }
        else:
            logger.warning(
                f"Config analysis returned non-success: {result.get('message', 'unknown')}"
            )
            return {
                "success": False,
                "message": "Config analysis failed. Check server logs for details.",
            }

    except Exception as e:
        logger.error(f"Failed to trigger config analysis: {e}")
        return {
            "success": False,
            "error": "Config analysis failed. Check server logs for details.",
        }


@app.get("/ai/config-suggestions")
@limiter.limit("60/minute")  # Rate limit: 60 requests per minute
async def get_config_suggestions(request: Request, days: int = 30, limit: int = 20):
    """
    Get config improvement suggestions history.

    This endpoint returns past Grok config analysis results stored in MongoDB.

    Args:
        days: Number of days to look back (default: 30)
        limit: Maximum number of records (default: 20)

    Returns:
        List of config suggestions with timestamps
    """
    try:
        from utils.data_storage import storage

        suggestions = storage.get_config_suggestions_history(days=days, limit=limit)

        # Convert ObjectId to string for JSON serialization
        formatted = []
        for s in suggestions:
            s["_id"] = str(s.get("_id", ""))
            formatted.append(s)

        return {
            "success": True,
            "count": len(formatted),
            "suggestions": formatted,
        }

    except Exception as e:
        logger.error(f"Failed to get config suggestions: {e}")
        return {
            "success": False,
            "error": "Failed to get config suggestions. Check server logs for details.",
            "suggestions": [],
        }


@app.get("/ai/gpt4-analysis-history")
@limiter.limit("60/minute")  # Rate limit: 60 requests per minute
async def get_gpt4_analysis_history(request: Request, days: int = 30, limit: int = 20):
    """
    Get Grok self-analysis history.

    This endpoint returns past Grok self-analysis results (retrain recommendations).

    Args:
        days: Number of days to look back (default: 30)
        limit: Maximum number of records (default: 20)

    Returns:
        List of Grok analysis results with timestamps
    """
    try:
        from utils.data_storage import storage

        analyses = storage.get_gpt4_analysis_history(days=days, limit=limit)

        # Convert ObjectId to string for JSON serialization
        formatted = []
        for a in analyses:
            a["_id"] = str(a.get("_id", ""))
            formatted.append(a)

        return {
            "success": True,
            "count": len(formatted),
            "analyses": formatted,
        }

    except Exception as e:
        logger.error(f"Failed to get Grok analysis history: {e}")
        return {
            "success": False,
            "error": "Failed to get analysis history. Check server logs for details.",
            "analyses": [],
        }


# === TRADE ANALYSIS ENDPOINT ===


class TradeAnalysisRequest(BaseModel):
    """Request model for analyzing a closed trade via xAI Grok."""

    trade_id: str
    symbol: str
    side: str  # "Long" or "Short"
    entry_price: float
    exit_price: float
    quantity: float
    leverage: int = 1
    pnl_usdt: float
    pnl_percentage: float
    duration_seconds: Optional[int] = None
    close_reason: Optional[str] = None
    open_time: Optional[str] = None
    close_time: Optional[str] = None
    strategy_name: Optional[str] = None
    ai_confidence: Optional[float] = None
    ai_reasoning: Optional[str] = None


async def perform_trade_analysis(trade_data: Dict[str, Any]) -> Dict[str, Any]:
    """
    Standalone trade analysis function using xAI Grok.
    Extracted from Celery task for direct HTTP endpoint use.

    Args:
        trade_data: Trade data dict with keys matching TradeAnalysisRequest fields

    Returns:
        Analysis result dict
    """
    from utils.data_storage import storage as sync_storage

    trade_id = trade_data.get("trade_id", "unknown")
    logger.info(f"Analyzing trade {trade_id}...")

    # Check if already analyzed (cache hit)
    existing = sync_storage.get_trade_analysis(trade_id)
    if existing:
        logger.info(f"Trade {trade_id} already analyzed, returning cached result")
        return {
            "status": "cached",
            "trade_id": trade_id,
            "analysis": existing.get("analysis"),
        }

    # Check AI API key
    api_key = os.getenv("XAI_API_KEY") or os.getenv("OPENAI_API_KEY")
    if not api_key:
        logger.warning("AI API key not configured - skipping trade analysis")
        return {
            "status": "skipped",
            "reason": "AI API key not configured (XAI_API_KEY or OPENAI_API_KEY)",
            "trade_id": trade_id,
        }

    import httpx as _httpx

    class _SimpleGrokClient:
        def __init__(self, key, base_url):
            self._key = key
            self._base_url = base_url

        async def chat_completions_create(
            self, model, messages, temperature=0.0, max_tokens=1200
        ):
            async with _httpx.AsyncClient(timeout=60.0) as c:
                r = await c.post(
                    f"{self._base_url}/chat/completions",
                    headers={
                        "Authorization": f"Bearer {self._key}",
                        "Content-Type": "application/json",
                    },
                    json={
                        "model": model,
                        "messages": messages,
                        "temperature": temperature,
                        "max_tokens": max_tokens,
                    },
                )
                r.raise_for_status()
                return r.json()

    client = _SimpleGrokClient(api_key, AI_BASE_URL)

    # Determine if winning or losing
    pnl = trade_data.get("pnl_usdt", 0)
    pnl_pct = trade_data.get("pnl_percentage", 0)
    is_winning = pnl > 0

    # Fetch current market conditions
    symbol = trade_data.get("symbol", "BTCUSDT")
    market_data = {}
    try:
        import httpx

        async with httpx.AsyncClient(timeout=5) as http_client:
            response = await http_client.get(
                "https://api.binance.com/api/v3/ticker/24hr",
                params={"symbol": symbol},
            )
            if response.status_code == 200:
                market_data = response.json()
    except Exception as e:
        logger.warning(f"Failed to fetch market data for {symbol}: {e}")

    # Build analysis prompt
    trade_type = "WINNING" if is_winning else "LOSING"

    prompt = f"""
You are a professional trading analyst. Analyze this {trade_type} trade and provide actionable insights.

## TRADE DETAILS:
- Trade ID: {trade_id}
- Symbol: {trade_data.get("symbol")}
- Side: {trade_data.get("side")} (Long/Short)
- Entry Price: ${trade_data.get("entry_price", 0):.4f}
- Exit Price: ${trade_data.get("exit_price", 0):.4f}
- Quantity: {trade_data.get("quantity", 0)}
- Leverage: {trade_data.get("leverage", 1)}x
- PnL: ${pnl:.2f} ({pnl_pct:.2f}%)
- Duration: {trade_data.get("duration_seconds", 0)} seconds
- Close Reason: {trade_data.get("close_reason", "unknown")}
- Open Time: {trade_data.get("open_time", "N/A")}
- Close Time: {trade_data.get("close_time", "N/A")}
- Strategy: {trade_data.get("strategy_name", "N/A")}
- AI Confidence: {trade_data.get("ai_confidence", "N/A")}

## CURRENT MARKET CONDITIONS:
- 24h Price Change: {market_data.get("priceChangePercent", "N/A")}%
- 24h Volume: {market_data.get("volume", "N/A")}
- Current Price: ${market_data.get("lastPrice", "N/A")}

## YOUR ANALYSIS TASKS:
1. **Entry Analysis**: Was the entry timing good? Were signals valid?
2. **Exit Analysis**: Was the exit optimal? Too early? Too late?
3. **What Went {"Right" if is_winning else "Wrong"}**: Key factors that led to this {"profit" if is_winning else "loss"}
4. **Lessons Learned**: What can be improved for future trades?
5. **Actionable Recommendations**: Specific parameter changes if needed

## OUTPUT FORMAT (MUST BE VALID JSON):
{{
  "trade_verdict": "{trade_type}",
  "entry_analysis": {{
    "quality": "good" | "acceptable" | "poor",
    "reasoning": "Why entry was good/bad",
    "signals_valid": true/false
  }},
  "exit_analysis": {{
    "quality": "optimal" | "acceptable" | "suboptimal",
    "reasoning": "Why exit was good/bad",
    "better_exit_point": "Description of better exit if applicable"
  }},
  "key_factors": ["factor1", "factor2", "factor3"],
  "lessons_learned": ["lesson1", "lesson2"],
  "recommendations": {{
    "config_changes": {{"param": "suggested_value", ...}} or null,
    "strategy_improvements": ["improvement1", "improvement2"],
    "risk_management": "Any risk management advice"
  }},
  "confidence": 0.0-1.0,
  "summary": "One paragraph summary of the analysis"
}}
"""

    # Call xAI Grok
    logger.info(f"Calling xAI Grok to analyze trade {trade_id}...")
    response = client.chat.completions.create(
        model=AI_MODEL,
        messages=[
            {
                "role": "system",
                "content": "You are a professional trading analyst. Provide detailed, actionable analysis of trades. Always respond with valid JSON.",
            },
            {
                "role": "user",
                "content": prompt,
            },
        ],
        temperature=0.3,
        max_completion_tokens=1500,
    )

    gpt4_response = response.choices[0].message.content

    # Strip markdown code blocks if present
    if gpt4_response.startswith("```"):
        lines = gpt4_response.split("\n")
        lines = [l for l in lines if not l.strip().startswith("```")]
        gpt4_response = "\n".join(lines)

    # Parse JSON response
    analysis = json.loads(gpt4_response)

    logger.info(f"xAI Trade Analysis Complete for {trade_id}:")
    logger.info(f"  Verdict: {analysis.get('trade_verdict', 'N/A')}")
    logger.info(
        f"  Entry Quality: {analysis.get('entry_analysis', {}).get('quality', 'N/A')}"
    )
    logger.info(
        f"  Exit Quality: {analysis.get('exit_analysis', {}).get('quality', 'N/A')}"
    )
    logger.info(f"  Summary: {analysis.get('summary', 'N/A')[:100]}...")

    # Store analysis in MongoDB
    sync_storage.store_trade_analysis(trade_id, trade_data, analysis)

    return {
        "status": "success",
        "trade_id": trade_id,
        "is_winning": is_winning,
        "pnl_usdt": pnl,
        "analysis": analysis,
    }


@app.post("/ai/analyze-trade")
async def analyze_trade_endpoint(request: Request, trade_request: TradeAnalysisRequest):
    """
    Analyze a closed trade using xAI Grok AI.
    Called automatically by Rust engine when a losing trade is closed.
    Runs analysis in background and returns immediately.
    """
    trade_data = trade_request.model_dump()

    # Run analysis in background so endpoint returns immediately
    async def _run_analysis():
        try:
            await perform_trade_analysis(trade_data)
        except Exception as e:
            logger.error(
                f"Background trade analysis failed for {trade_data.get('trade_id')}: {e}"
            )

    asyncio.create_task(_run_analysis())

    return {
        "status": "accepted",
        "trade_id": trade_request.trade_id,
        "message": "Trade analysis queued",
    }


# === PROJECT CHATBOT ENDPOINTS ===

# Global chatbot instance (initialized lazily)
_project_chatbot: Optional[ProjectChatbot] = None


@app.post("/api/chat/project", response_model=ProjectChatResponse)
async def chat_with_project(request: ProjectChatRequest):
    """
    Chat with the project documentation using RAG (Retrieval Augmented Generation).

    This endpoint allows users to ask questions about the BotCore project.
    It uses Grok with context from indexed project documentation (specs, docs, READMEs).

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
            _project_chatbot = await get_chatbot(grok_client)
        elif grok_client and _project_chatbot.grok_client is None:
            _project_chatbot.grok_client = grok_client

        # Process the message
        result = await _project_chatbot.chat(
            message=request.message, include_history=request.include_history
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
            message="Xin lỗi, có lỗi xảy ra. Vui lòng thử lại sau.",
            sources=[],
            confidence=0.0,
            type="error",
        )


@app.get("/api/chat/project/suggestions")
async def get_chat_suggestions():
    """Get suggested questions for the project chatbot."""
    global _project_chatbot

    if _project_chatbot is None:
        _project_chatbot = await get_chatbot(grok_client)

    return {
        "suggestions": _project_chatbot.get_suggested_questions(),
        "timestamp": datetime.now(timezone.utc).isoformat(),
    }


@app.post("/api/chat/project/clear")
async def clear_chat_history():
    """Clear conversation history for the project chatbot."""
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
        "service": "Grok AI Cryptocurrency Trading Service",
        "version": "3.0.0",
        "description": (
            "Advanced AI-powered trading signal generation using xAI Grok "
            "with MongoDB storage and real-time WebSocket broadcasting"
        ),
        "endpoints": {
            "analyze": "POST /ai/analyze - Generate trading signals with AI (stored in MongoDB)",
            "strategy_recommendations": "POST /ai/strategy-recommendations - Get strategy recommendations",
            "market_condition": "POST /ai/market-condition - Analyze market conditions",
            "feedback": "POST /ai/feedback - Send performance feedback",
            "health": "GET /health - Health check with MongoDB status",
            "storage_stats": "GET /ai/storage/stats - View storage statistics",
            "cost_stats": "GET /ai/cost/statistics - View Grok API cost statistics",
            "clear_storage": "POST /ai/storage/clear - Clear analysis storage",
            "websocket": "WS /ws - Real-time AI signal broadcasting",
            "project_chat": "POST /api/chat/project - RAG chatbot for project questions",
            "chat_suggestions": "GET /api/chat/project/suggestions - Get suggested questions",
            "clear_chat": "POST /api/chat/project/clear - Clear chat history",
        },
        "documentation": "/docs",
        "features": {
            "grok_enabled": grok_client is not None,
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

    logger.info("🚀 Starting Grok AI Trading Service...")

    uvicorn.run("main:app", host="0.0.0.0", port=8000, reload=False, log_level="info")
