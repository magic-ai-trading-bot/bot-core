"""
Pydantic models and schemas for API requests/responses.
"""

from typing import Dict, Any, List, Optional
from pydantic import BaseModel, Field
from datetime import datetime


class AnalyzeRequest(BaseModel):
    """Request model for market analysis endpoint."""

    symbol: str = Field(..., description="Trading pair symbol (e.g., BTCUSDT)")
    timeframe: str = Field(default="1h", description="Chart timeframe")
    limit: int = Field(default=100, description="Number of candles to analyze")


class TechnicalIndicators(BaseModel):
    """Technical indicators for market analysis."""

    rsi: Optional[float] = Field(None, description="RSI indicator value")
    macd: Optional[float] = Field(None, description="MACD value")
    macd_signal: Optional[float] = Field(None, description="MACD signal line")
    bb_upper: Optional[float] = Field(None, description="Bollinger Band upper")
    bb_middle: Optional[float] = Field(None, description="Bollinger Band middle")
    bb_lower: Optional[float] = Field(None, description="Bollinger Band lower")
    volume_ma: Optional[float] = Field(None, description="Volume moving average")
    price_ma_20: Optional[float] = Field(None, description="20-period MA")
    price_ma_50: Optional[float] = Field(None, description="50-period MA")


class MarketAnalysis(BaseModel):
    """Market analysis result model."""

    symbol: str
    timestamp: datetime
    signal: str = Field(..., description="Trading signal: BUY, SELL, or HOLD")
    confidence: float = Field(..., ge=0.0, le=1.0, description="Confidence score")
    reasoning: str = Field(..., description="Analysis reasoning")
    technical_indicators: Optional[TechnicalIndicators] = None
    support_level: Optional[float] = None
    resistance_level: Optional[float] = None
    predicted_direction: Optional[str] = None


class GPT4AnalysisRequest(BaseModel):
    """Request model for GPT-4 analysis endpoint."""

    symbol: str
    price: float = Field(..., gt=0, description="Current price")
    indicators: Dict[str, Any] = Field(..., description="Technical indicators")
    market_context: Optional[str] = Field(None, description="Additional context")


class HealthResponse(BaseModel):
    """Health check response model."""

    status: str
    timestamp: datetime
    service: str
    version: str
    gpt4_available: bool
    api_key_configured: bool
    mongodb_connected: bool
    analysis_interval_minutes: int
    supported_symbols: List[str]


class MetricsResponse(BaseModel):
    """Metrics response model."""

    total_requests: int
    total_input_tokens: int
    total_output_tokens: int
    total_cost_usd: float
    uptime_seconds: float
    active_websocket_connections: int


class CostSummaryResponse(BaseModel):
    """Cost summary response model."""

    total_cost_usd: float
    total_requests: int
    avg_cost_per_request: float
    total_input_tokens: int
    total_output_tokens: int
    input_cost_per_1m: float
    output_cost_per_1m: float
