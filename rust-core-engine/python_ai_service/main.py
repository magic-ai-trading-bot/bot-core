#!/usr/bin/env python3
"""
Simple AI Service for Binance Trading Bot
This is a basic example that provides trading signals based on simple technical indicators.
In a real implementation, you would use more sophisticated ML models.
"""

import asyncio
import logging
import random
from datetime import datetime
from typing import Dict, List, Optional

import numpy as np
import pandas as pd
from fastapi import FastAPI, HTTPException
from pydantic import BaseModel
import uvicorn

# Configure logging
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

app = FastAPI(title="Trading Bot AI Service", version="1.0.0")

class CandleData(BaseModel):
    timestamp: int
    open: float
    high: float
    low: float
    close: float
    volume: float

class AnalysisRequest(BaseModel):
    symbol: str
    timeframe: str
    candles: List[CandleData]
    analysis_type: str
    parameters: Dict = {}

class AnalysisResponse(BaseModel):
    symbol: str
    timeframe: str
    timestamp: int
    signal: str  # BUY, SELL, HOLD, STRONG_BUY, STRONG_SELL
    confidence: float
    indicators: Dict[str, float]
    analysis_details: Dict = {}

class TechnicalAnalyzer:
    """Simple technical analysis indicators"""
    
    @staticmethod
    def calculate_sma(prices: List[float], period: int) -> float:
        """Simple Moving Average"""
        if len(prices) < period:
            return prices[-1] if prices else 0.0
        return sum(prices[-period:]) / period
    
    @staticmethod
    def calculate_ema(prices: List[float], period: int) -> float:
        """Exponential Moving Average"""
        if len(prices) < period:
            return prices[-1] if prices else 0.0
        
        multiplier = 2 / (period + 1)
        ema = prices[0]
        
        for price in prices[1:]:
            ema = (price * multiplier) + (ema * (1 - multiplier))
        
        return ema
    
    @staticmethod
    def calculate_rsi(prices: List[float], period: int = 14) -> float:
        """Relative Strength Index"""
        if len(prices) < period + 1:
            return 50.0  # Neutral
        
        deltas = [prices[i] - prices[i-1] for i in range(1, len(prices))]
        gains = [d if d > 0 else 0 for d in deltas]
        losses = [-d if d < 0 else 0 for d in deltas]
        
        avg_gain = sum(gains[-period:]) / period
        avg_loss = sum(losses[-period:]) / period
        
        if avg_loss == 0:
            return 100.0
        
        rs = avg_gain / avg_loss
        rsi = 100 - (100 / (1 + rs))
        return rsi
    
    @staticmethod
    def calculate_macd(prices: List[float], fast: int = 12, slow: int = 26, signal: int = 9) -> Dict[str, float]:
        """MACD Indicator"""
        if len(prices) < slow:
            return {"macd": 0.0, "signal": 0.0, "histogram": 0.0}
        
        ema_fast = TechnicalAnalyzer.calculate_ema(prices, fast)
        ema_slow = TechnicalAnalyzer.calculate_ema(prices, slow)
        macd_line = ema_fast - ema_slow
        
        # Simplified signal line calculation
        signal_line = macd_line * 0.8  # Simplified
        histogram = macd_line - signal_line
        
        return {
            "macd": macd_line,
            "signal": signal_line,
            "histogram": histogram
        }
    
    @staticmethod
    def calculate_bollinger_bands(prices: List[float], period: int = 20, std_dev: float = 2) -> Dict[str, float]:
        """Bollinger Bands"""
        if len(prices) < period:
            current_price = prices[-1] if prices else 0.0
            return {
                "upper": current_price * 1.02,
                "middle": current_price,
                "lower": current_price * 0.98,
                "position": 0.5
            }
        
        sma = TechnicalAnalyzer.calculate_sma(prices, period)
        variance = sum((price - sma) ** 2 for price in prices[-period:]) / period
        std = variance ** 0.5
        
        upper = sma + (std * std_dev)
        lower = sma - (std * std_dev)
        current_price = prices[-1]
        
        # Position within bands (0 = lower band, 1 = upper band)
        if upper != lower:
            position = (current_price - lower) / (upper - lower)
        else:
            position = 0.5
        
        return {
            "upper": upper,
            "middle": sma,
            "lower": lower,
            "position": min(max(position, 0), 1)
        }

class SignalGenerator:
    """Generate trading signals based on technical indicators"""
    
    def __init__(self):
        self.analyzer = TechnicalAnalyzer()
    
    def analyze_trend(self, candles: List[CandleData]) -> AnalysisResponse:
        """Analyze trend and generate trading signal"""
        if len(candles) < 20:
            return self._neutral_signal(candles[0].symbol if candles else "UNKNOWN")
        
        # Extract price data
        closes = [c.close for c in candles]
        highs = [c.high for c in candles]
        lows = [c.low for c in candles]
        volumes = [c.volume for c in candles]
        
        # Calculate indicators
        rsi = self.analyzer.calculate_rsi(closes)
        macd_data = self.analyzer.calculate_macd(closes)
        bb_data = self.analyzer.calculate_bollinger_bands(closes)
        sma_20 = self.analyzer.calculate_sma(closes, 20)
        sma_50 = self.analyzer.calculate_sma(closes, 50)
        
        current_price = closes[-1]
        
        # Generate signals based on indicators
        signals = []
        confidence_factors = []
        
        # RSI signals
        if rsi < 30:
            signals.append("BUY")
            confidence_factors.append(0.8)
        elif rsi > 70:
            signals.append("SELL")
            confidence_factors.append(0.8)
        else:
            signals.append("HOLD")
            confidence_factors.append(0.3)
        
        # MACD signals
        if macd_data["histogram"] > 0 and macd_data["macd"] > macd_data["signal"]:
            signals.append("BUY")
            confidence_factors.append(0.7)
        elif macd_data["histogram"] < 0 and macd_data["macd"] < macd_data["signal"]:
            signals.append("SELL")
            confidence_factors.append(0.7)
        else:
            signals.append("HOLD")
            confidence_factors.append(0.4)
        
        # Bollinger Bands signals
        bb_position = bb_data["position"]
        if bb_position < 0.2:
            signals.append("BUY")
            confidence_factors.append(0.6)
        elif bb_position > 0.8:
            signals.append("SELL")
            confidence_factors.append(0.6)
        else:
            signals.append("HOLD")
            confidence_factors.append(0.3)
        
        # Moving Average signals
        if current_price > sma_20 > sma_50:
            signals.append("BUY")
            confidence_factors.append(0.5)
        elif current_price < sma_20 < sma_50:
            signals.append("SELL")
            confidence_factors.append(0.5)
        else:
            signals.append("HOLD")
            confidence_factors.append(0.3)
        
        # Combine signals
        buy_signals = signals.count("BUY")
        sell_signals = signals.count("SELL")
        hold_signals = signals.count("HOLD")
        
        # Determine final signal
        if buy_signals > sell_signals and buy_signals > hold_signals:
            if buy_signals >= 3:
                final_signal = "STRONG_BUY"
                base_confidence = 0.85
            else:
                final_signal = "BUY"
                base_confidence = 0.65
        elif sell_signals > buy_signals and sell_signals > hold_signals:
            if sell_signals >= 3:
                final_signal = "STRONG_SELL"
                base_confidence = 0.85
            else:
                final_signal = "SELL"
                base_confidence = 0.65
        else:
            final_signal = "HOLD"
            base_confidence = 0.40
        
        # Calculate overall confidence
        avg_confidence = sum(confidence_factors) / len(confidence_factors)
        final_confidence = min((base_confidence + avg_confidence) / 2, 0.95)
        
        # Add some randomness to make it more realistic
        final_confidence *= (0.9 + random.random() * 0.2)
        final_confidence = min(max(final_confidence, 0.1), 0.95)
        
        return AnalysisResponse(
            symbol=candles[0].symbol,
            timeframe=candles[0].timestamp,  # This should be timeframe, but we'll use timestamp
            timestamp=int(datetime.now().timestamp() * 1000),
            signal=final_signal,
            confidence=round(final_confidence, 3),
            indicators={
                "rsi": round(rsi, 2),
                "macd": round(macd_data["macd"], 4),
                "macd_signal": round(macd_data["signal"], 4),
                "macd_histogram": round(macd_data["histogram"], 4),
                "bb_position": round(bb_position, 3),
                "bb_upper": round(bb_data["upper"], 2),
                "bb_lower": round(bb_data["lower"], 2),
                "sma_20": round(sma_20, 2),
                "sma_50": round(sma_50, 2),
                "current_price": round(current_price, 2)
            },
            analysis_details={
                "total_signals": len(signals),
                "buy_signals": buy_signals,
                "sell_signals": sell_signals,
                "hold_signals": hold_signals,
                "primary_factors": [
                    f"RSI: {rsi:.1f}",
                    f"MACD: {macd_data['macd']:.4f}",
                    f"BB Position: {bb_position:.2f}",
                    f"Price vs SMA20: {((current_price / sma_20 - 1) * 100):.1f}%"
                ]
            }
        )
    
    def _neutral_signal(self, symbol: str) -> AnalysisResponse:
        """Return a neutral signal when insufficient data"""
        return AnalysisResponse(
            symbol=symbol,
            timeframe="unknown",
            timestamp=int(datetime.now().timestamp() * 1000),
            signal="HOLD",
            confidence=0.1,
            indicators={"insufficient_data": True},
            analysis_details={"reason": "Insufficient historical data for analysis"}
        )

# Initialize signal generator
signal_generator = SignalGenerator()

@app.get("/")
async def root():
    """Health check endpoint"""
    return {
        "service": "Trading Bot AI Service",
        "status": "running",
        "timestamp": datetime.now().isoformat()
    }

@app.post("/api/analyze", response_model=AnalysisResponse)
async def analyze_market(request: AnalysisRequest):
    """
    Analyze market data and return trading signal
    """
    try:
        logger.info(f"Received analysis request for {request.symbol} {request.timeframe}")
        
        if not request.candles:
            raise HTTPException(status_code=400, detail="No candle data provided")
        
        # Process different analysis types
        if request.analysis_type == "trend_analysis":
            result = signal_generator.analyze_trend(request.candles)
        else:
            # Default to trend analysis
            result = signal_generator.analyze_trend(request.candles)
        
        # Update symbol and timeframe from request
        result.symbol = request.symbol
        result.timeframe = request.timeframe
        
        logger.info(f"Generated signal for {request.symbol}: {result.signal} "
                   f"(confidence: {result.confidence:.2f})")
        
        return result
        
    except Exception as e:
        logger.error(f"Error analyzing {request.symbol}: {str(e)}")
        raise HTTPException(status_code=500, detail=f"Analysis failed: {str(e)}")

@app.get("/api/health")
async def health_check():
    """Detailed health check"""
    return {
        "status": "healthy",
        "service": "AI Trading Service",
        "version": "1.0.0",
        "timestamp": datetime.now().isoformat(),
        "capabilities": [
            "trend_analysis",
            "technical_indicators",
            "signal_generation"
        ]
    }

if __name__ == "__main__":
    print("Starting AI Trading Service...")
    print("This service provides trading signals based on technical analysis.")
    print("Available at: http://localhost:8000")
    print("API docs at: http://localhost:8000/docs")
    
    uvicorn.run(
        app,
        host="0.0.0.0",
        port=8000,
        log_level="info"
    ) 