import asyncio
import pandas as pd
from datetime import datetime
from typing import Dict, Any, List, Optional
from fastapi import FastAPI, HTTPException, BackgroundTasks
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel, Field, validator
from contextlib import asynccontextmanager

# Local imports
from config.config import config
from utils.logger import setup_logger, get_logger
from utils.helpers import validate_ohlcv_data, create_dataframe_from_ohlcv, format_confidence_score
from models.model_manager import ModelManager

# Initialize logger
setup_logger()
logger = get_logger("TradingAI_API")

# Global model manager instance
model_manager = None

@asynccontextmanager
async def lifespan(app: FastAPI):
    """Application lifespan manager."""
    global model_manager
    
    # Startup
    logger.info("Starting AI Trading Service")
    model_manager = ModelManager()
    
    # Try to load existing model
    if model_manager.load_model():
        logger.info("Existing model loaded successfully")
    else:
        logger.info("No existing model found - will need training data")
    
    yield
    
    # Shutdown
    logger.info("Shutting down AI Trading Service")

# Create FastAPI app
app = FastAPI(
    title="Cryptocurrency AI Trading Service",
    description="AI-powered trading signal generation for cryptocurrency markets",
    version="1.0.0",
    lifespan=lifespan
)

# CORS middleware
app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],  # Configure appropriately for production
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# Pydantic models for request/response
class CandleData(BaseModel):
    """Individual candle data model."""
    timestamp: int = Field(..., description="Unix timestamp in milliseconds")
    open: float = Field(..., gt=0, description="Opening price")
    high: float = Field(..., gt=0, description="High price")
    low: float = Field(..., gt=0, description="Low price") 
    close: float = Field(..., gt=0, description="Closing price")
    volume: float = Field(..., ge=0, description="Trading volume")
    
    @validator('high')
    def validate_high(cls, v, values):
        if 'low' in values and v < values['low']:
            raise ValueError('High price must be >= low price')
        return v
    
    @validator('open')
    def validate_open(cls, v, values):
        if 'high' in values and 'low' in values:
            if not (values['low'] <= v <= values['high']):
                raise ValueError('Open price must be between low and high')
        return v
    
    @validator('close')
    def validate_close(cls, v, values):
        if 'high' in values and 'low' in values:
            if not (values['low'] <= v <= values['high']):
                raise ValueError('Close price must be between low and high')
        return v

class AnalysisRequest(BaseModel):
    """Request model for trading signal analysis."""
    symbol: str = Field(..., description="Trading pair symbol (e.g., BTCUSDT)")
    timeframe: str = Field(..., description="Timeframe (1m, 5m, 15m, 1h, 4h, 1d)")
    candles: List[CandleData] = Field(..., min_items=1, description="OHLCV candle data")
    
    @validator('timeframe')
    def validate_timeframe(cls, v):
        if not config.is_valid_timeframe(v):
            supported = config.get_supported_timeframes()
            raise ValueError(f'Unsupported timeframe. Supported: {supported}')
        return v

class TradingSignal(BaseModel):
    """Response model for trading signals."""
    signal: str = Field(..., description="Trading signal: long, short, or neutral")
    confidence: float = Field(..., ge=0, le=100, description="Confidence percentage")
    probability: float = Field(..., ge=0, le=1, description="Raw probability score")
    timestamp: str = Field(..., description="Analysis timestamp")
    model_type: str = Field(..., description="AI model type used")
    symbol: str = Field(..., description="Trading pair symbol")
    timeframe: str = Field(..., description="Timeframe analyzed")

class TrainingRequest(BaseModel):
    """Request model for model training."""
    symbol: str = Field(..., description="Trading pair symbol")
    candles: List[CandleData] = Field(..., min_items=100, description="Historical OHLCV data")
    model_type: Optional[str] = Field("lstm", description="Model type: lstm, gru, or transformer")
    retrain: bool = Field(False, description="Force retrain existing model")

class ModelInfo(BaseModel):
    """Response model for model information."""
    model_type: str
    model_loaded: bool
    training_samples: Optional[int] = None
    validation_samples: Optional[int] = None
    feature_count: Optional[int] = None
    trained_timestamp: Optional[str] = None
    training_accuracy: Optional[float] = None

class HealthResponse(BaseModel):
    """Health check response model."""
    status: str
    timestamp: str
    model_loaded: bool
    version: str

# API Endpoints

@app.get("/health", response_model=HealthResponse)
async def health_check():
    """Health check endpoint."""
    return HealthResponse(
        status="healthy",
        timestamp=datetime.utcnow().isoformat(),
        model_loaded=model_manager.current_model is not None,
        version="1.0.0"
    )

@app.post("/analyze", response_model=TradingSignal)
async def analyze_market_data(request: AnalysisRequest):
    """
    Analyze cryptocurrency market data and return trading signals.
    
    This endpoint receives OHLCV candlestick data and returns AI-generated
    trading signals (Long, Short, Neutral) with confidence scores.
    """
    try:
        logger.info(f"Received analysis request for {request.symbol} on {request.timeframe}")
        
        # Validate minimum data requirements
        min_candles = config.get('data', 'min_candles_required', 100)
        if len(request.candles) < min_candles:
            raise HTTPException(
                status_code=400,
                detail=f"Insufficient data. Minimum {min_candles} candles required, got {len(request.candles)}"
            )
        
        # Convert to DataFrame
        candle_dicts = [candle.dict() for candle in request.candles]
        data_dict = {
            'candles': candle_dicts,
            'symbol': request.symbol,
            'timeframe': request.timeframe
        }
        
        df = create_dataframe_from_ohlcv(data_dict)
        if df is None or df.empty:
            raise HTTPException(
                status_code=400,
                detail="Failed to process candle data"
            )
        
        # Check if model is available
        if model_manager.current_model is None:
            raise HTTPException(
                status_code=503,
                detail="AI model not available. Please train a model first."
            )
        
        # Make prediction
        prediction_result = model_manager.predict(df)
        
        # Check for prediction errors
        if 'error' in prediction_result:
            raise HTTPException(
                status_code=500,
                detail=f"Prediction failed: {prediction_result['error']}"
            )
        
        # Format response
        response = TradingSignal(
            signal=prediction_result['signal'],
            confidence=prediction_result['confidence'],
            probability=prediction_result['probability'],
            timestamp=prediction_result['timestamp'],
            model_type=prediction_result['model_type'],
            symbol=request.symbol,
            timeframe=request.timeframe
        )
        
        logger.info(f"Analysis completed: {response.signal} with {response.confidence}% confidence")
        return response
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error in analysis: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/train")
async def train_model(request: TrainingRequest, background_tasks: BackgroundTasks):
    """
    Train the AI model with historical data.
    
    This endpoint accepts historical OHLCV data and trains the specified
    AI model type (LSTM, GRU, or Transformer) in the background.
    """
    try:
        logger.info(f"Received training request for {request.symbol} with {len(request.candles)} candles")
        
        # Validate model type
        valid_types = ['lstm', 'gru', 'transformer']
        if request.model_type.lower() not in valid_types:
            raise HTTPException(
                status_code=400,
                detail=f"Invalid model type. Supported types: {valid_types}"
            )
        
        # Convert to DataFrame
        candle_dicts = [candle.dict() for candle in request.candles]
        data_dict = {
            'candles': candle_dicts,
            'symbol': request.symbol,
            'timeframe': 'training_data'
        }
        
        df = create_dataframe_from_ohlcv(data_dict)
        if df is None or df.empty:
            raise HTTPException(
                status_code=400,
                detail="Failed to process training data"
            )
        
        # Start training in background
        background_tasks.add_task(
            _train_model_background,
            df,
            request.model_type.lower(),
            request.retrain
        )
        
        return {
            "message": "Model training started",
            "model_type": request.model_type,
            "training_samples": len(request.candles),
            "status": "training_in_progress"
        }
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error starting training: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/model/info", response_model=ModelInfo)
async def get_model_info():
    """Get information about the current AI model."""
    try:
        info = model_manager.get_model_info()
        metadata = info.get('metadata', {})
        
        return ModelInfo(
            model_type=info['model_type'],
            model_loaded=info['model_loaded'],
            training_samples=metadata.get('training_samples'),
            validation_samples=metadata.get('validation_samples'),
            feature_count=metadata.get('feature_count'),
            trained_timestamp=metadata.get('trained_timestamp'),
            training_accuracy=metadata.get('training_results', {}).get('final_accuracy')
        )
        
    except Exception as e:
        logger.error(f"Error getting model info: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/model/load")
async def load_model(model_path: Optional[str] = None):
    """Load a saved AI model."""
    try:
        success = model_manager.load_model(model_path)
        
        if success:
            return {
                "message": "Model loaded successfully",
                "model_type": model_manager.model_type,
                "status": "ready"
            }
        else:
            raise HTTPException(
                status_code=404,
                detail="Failed to load model. Check if model file exists."
            )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error loading model: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.post("/model/save")
async def save_model(model_name: Optional[str] = None):
    """Save the current AI model."""
    try:
        if model_manager.current_model is None:
            raise HTTPException(
                status_code=400,
                detail="No model to save. Train a model first."
            )
        
        success = model_manager.save_model(model_name)
        
        if success:
            return {
                "message": "Model saved successfully",
                "model_type": model_manager.model_type,
                "timestamp": datetime.utcnow().isoformat()
            }
        else:
            raise HTTPException(
                status_code=500,
                detail="Failed to save model"
            )
        
    except HTTPException:
        raise
    except Exception as e:
        logger.error(f"Error saving model: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.delete("/model/cleanup")
async def cleanup_old_models(keep_count: int = 5):
    """Clean up old model files."""
    try:
        deleted_count = model_manager.cleanup_old_models(keep_count)
        
        return {
            "message": f"Cleanup completed",
            "deleted_models": deleted_count,
            "kept_models": keep_count
        }
        
    except Exception as e:
        logger.error(f"Error cleaning up models: {e}")
        raise HTTPException(status_code=500, detail=str(e))

@app.get("/config")
async def get_configuration():
    """Get current service configuration."""
    try:
        return {
            "supported_timeframes": config.get_supported_timeframes(),
            "model_config": config.get_model_config(),
            "trading_config": config.get_trading_config(),
            "data_config": config.get_data_config()
        }
        
    except Exception as e:
        logger.error(f"Error getting configuration: {e}")
        raise HTTPException(status_code=500, detail=str(e))

# Background task functions
async def _train_model_background(df: pd.DataFrame, model_type: str, retrain: bool):
    """Background task for model training."""
    try:
        logger.info(f"Starting background training of {model_type} model")
        
        # Set model type in config
        original_type = model_manager.model_type
        model_manager.model_type = model_type
        
        # Train model
        results = model_manager.train_model(df, retrain=retrain)
        
        logger.info(f"Background training completed successfully: {results}")
        
        # Cleanup old models
        model_manager.cleanup_old_models()
        
    except Exception as e:
        logger.error(f"Background training failed: {e}")
        # Restore original model type
        model_manager.model_type = original_type

# Root endpoint
@app.get("/")
async def root():
    """Root endpoint with API information."""
    return {
        "service": "Cryptocurrency AI Trading Service",
        "version": "1.0.0",
        "description": "AI-powered trading signal generation for cryptocurrency markets",
        "endpoints": {
            "analyze": "POST /analyze - Generate trading signals from OHLCV data",
            "train": "POST /train - Train AI model with historical data",
            "model_info": "GET /model/info - Get current model information",
            "health": "GET /health - Health check",
            "config": "GET /config - Get service configuration"
        },
        "documentation": "/docs",
        "model_loaded": model_manager.current_model is not None if model_manager else False
    }

# Run the application
if __name__ == "__main__":
    import uvicorn
    
    server_config = config.get_server_config()
    
    uvicorn.run(
        "main:app",
        host=server_config.get('host', '0.0.0.0'),
        port=server_config.get('port', 8000),
        reload=server_config.get('reload', False),
        log_level="info"
    ) 