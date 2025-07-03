# Cryptocurrency AI Trading Service

A comprehensive Python-based AI service that analyzes cryptocurrency market data and provides intelligent trading signals (Long, Short, Neutral) via REST API. Built with FastAPI, TensorFlow/Keras, and advanced technical analysis.

## Features

### 🧠 AI-Powered Analysis

- **Multiple Model Types**: LSTM, GRU, and Transformer models for time series prediction
- **Advanced Feature Engineering**: 20+ technical indicators and price patterns
- **Real-time Inference**: Optimized for low-latency trading signal generation
- **Configurable Thresholds**: Customizable confidence levels for trading signals

### 📊 Technical Indicators

- **Momentum**: RSI, MACD, Stochastic, Williams %R, ROC
- **Trend**: EMA (9, 21, 50), SMA, CCI
- **Volatility**: Bollinger Bands, ATR, Volatility Ratios
- **Volume**: OBV, VWAP, Volume SMA, Volume ROC
- **Price Patterns**: Support/Resistance, Breakouts, Doji, Hammer

### 🚀 REST API

- **FastAPI**: Modern, fast, auto-documented API
- **Comprehensive Endpoints**: Analysis, training, model management
- **Data Validation**: Pydantic models with extensive validation
- **Error Handling**: Detailed error responses and logging

### 🔧 Model Management

- **Auto-save/Load**: Automatic model persistence with metadata
- **Versioning**: Keep multiple model versions with cleanup
- **Retraining**: Scheduled and manual model retraining
- **Monitoring**: Model performance tracking and health checks

## Installation

### Prerequisites

- Python 3.8+
- 8GB+ RAM (for model training)
- GPU support (optional, for faster training)

### Install Dependencies

```bash
pip install -r requirements.txt
```

### Environment Setup

Create a `.env` file (optional):

```bash
SERVER_HOST=0.0.0.0
SERVER_PORT=8000
MODEL_TYPE=lstm
LOG_LEVEL=INFO
```

## Configuration

The service is configured via `config.yaml`. Key sections:

### Model Configuration

```yaml
model:
  type: "lstm" # lstm, gru, or transformer
  sequence_length: 60
  hidden_size: 64
  num_layers: 2
  dropout: 0.2
  learning_rate: 0.001
  batch_size: 32
  epochs: 100
```

### Trading Signals

```yaml
trading:
  long_threshold: 0.6 # Probability threshold for long signals
  short_threshold: 0.4 # Probability threshold for short signals
  confidence_threshold: 0.55
```

### Data Processing

```yaml
data:
  supported_timeframes: ["1m", "5m", "15m", "1h", "4h", "1d"]
  min_candles_required: 100
  max_candles_per_request: 1000
```

## Usage

### Start the Service

```bash
python main.py
```

The service will start on `http://localhost:8000` with:

- **API Documentation**: http://localhost:8000/docs
- **Health Check**: http://localhost:8000/health
- **Configuration**: http://localhost:8000/config

### API Endpoints

#### 1. Analyze Market Data

**POST** `/analyze`

Generate trading signals from OHLCV data.

```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "candles": [
    {
      "timestamp": 1640995200000,
      "open": 47000.0,
      "high": 47500.0,
      "low": 46800.0,
      "close": 47200.0,
      "volume": 1234.56
    }
  ]
}
```

**Response:**

```json
{
  "signal": "long",
  "confidence": 75.8,
  "probability": 0.758,
  "timestamp": "2024-01-15T10:30:00Z",
  "model_type": "lstm",
  "symbol": "BTCUSDT",
  "timeframe": "1h"
}
```

#### 2. Train Model

**POST** `/train`

Train AI model with historical data.

```json
{
  "symbol": "BTCUSDT",
  "model_type": "lstm",
  "retrain": false,
  "candles": [...]  // Array of 500+ historical candles
}
```

#### 3. Model Information

**GET** `/model/info`

Get current model status and performance metrics.

#### 4. Load/Save Model

- **POST** `/model/load` - Load saved model
- **POST** `/model/save` - Save current model
- **DELETE** `/model/cleanup` - Clean old model files

### Python Client Example

```python
import requests
import json

# Example OHLCV data
candles = [
    {
        "timestamp": 1640995200000,
        "open": 47000.0,
        "high": 47500.0,
        "low": 46800.0,
        "close": 47200.0,
        "volume": 1234.56
    }
    # ... more candles
]

# Analyze market data
response = requests.post("http://localhost:8000/analyze", json={
    "symbol": "BTCUSDT",
    "timeframe": "1h",
    "candles": candles
})

if response.status_code == 200:
    signal = response.json()
    print(f"Signal: {signal['signal']}")
    print(f"Confidence: {signal['confidence']}%")
else:
    print(f"Error: {response.text}")
```

### Rust Integration Example

```rust
use reqwest;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    let payload = json!({
        "symbol": "BTCUSDT",
        "timeframe": "1h",
        "candles": [
            {
                "timestamp": 1640995200000,
                "open": 47000.0,
                "high": 47500.0,
                "low": 46800.0,
                "close": 47200.0,
                "volume": 1234.56
            }
        ]
    });

    let response = client
        .post("http://localhost:8000/analyze")
        .json(&payload)
        .send()
        .await?;

    let signal: serde_json::Value = response.json().await?;
    println!("Trading Signal: {:?}", signal);

    Ok(())
}
```

## Model Training

### Training Process

1. **Data Preparation**: Technical indicators calculation and feature engineering
2. **Sequence Creation**: Time series sequences for model input
3. **Feature Scaling**: Normalization for neural network training
4. **Model Architecture**: Dynamic model creation based on configuration
5. **Training**: With early stopping and learning rate scheduling
6. **Validation**: Performance evaluation and metrics calculation
7. **Persistence**: Model saving with metadata and feature scalers

### Training Data Requirements

- **Minimum**: 100 candles (configurable)
- **Recommended**: 1000+ candles for better performance
- **Format**: OHLCV with timestamps
- **Quality**: Clean data without gaps or anomalies

### Model Performance

The service tracks various metrics:

- **Accuracy**: Prediction accuracy on validation set
- **Precision/Recall**: Signal quality metrics
- **Loss**: Model training loss
- **Confidence Distribution**: Signal confidence statistics

## Architecture

### Project Structure

```
python-ai/
├── config/
│   ├── __init__.py
│   ├── config.py          # Configuration management
│   └── config.yaml        # Configuration file
├── features/
│   ├── __init__.py
│   ├── technical_indicators.py  # Technical analysis
│   └── feature_engineering.py  # Feature preparation
├── models/
│   ├── __init__.py
│   ├── lstm_model.py      # LSTM implementation
│   ├── gru_model.py       # GRU implementation
│   ├── transformer_model.py  # Transformer implementation
│   └── model_manager.py   # Model lifecycle management
├── utils/
│   ├── __init__.py
│   ├── logger.py          # Logging utilities
│   └── helpers.py         # Helper functions
├── main.py                # FastAPI application
├── requirements.txt       # Dependencies
└── README.md
```

### Data Flow

1. **Input**: OHLCV data from Rust trading bot
2. **Processing**: Technical indicators calculation
3. **Feature Engineering**: Sequence creation and scaling
4. **Prediction**: AI model inference
5. **Output**: Trading signal with confidence

## Performance Optimization

### Inference Performance

- **Model Optimization**: Efficient architectures for real-time inference
- **Feature Caching**: Reuse calculated indicators
- **Batch Processing**: Process multiple requests efficiently
- **Memory Management**: Optimized memory usage

### Training Performance

- **GPU Support**: CUDA acceleration for model training
- **Parallel Processing**: Multi-threaded feature calculation
- **Early Stopping**: Prevent overfitting and reduce training time
- **Model Checkpointing**: Save best models during training

## Monitoring & Logging

### Logging Features

- **Structured Logging**: JSON format with timestamps
- **Log Levels**: DEBUG, INFO, WARNING, ERROR
- **Log Rotation**: Automatic log file rotation
- **Performance Metrics**: Request timing and model performance

### Health Monitoring

- **Health Endpoint**: Service status and model availability
- **Model Metrics**: Training performance and prediction statistics
- **Resource Usage**: Memory and CPU monitoring
- **Error Tracking**: Detailed error logging and reporting

## Deployment

### Production Considerations

- **Environment Variables**: Use environment variables for sensitive configuration
- **CORS**: Configure appropriate CORS settings
- **Rate Limiting**: Implement rate limiting for API endpoints
- **Load Balancing**: Use multiple instances for high availability
- **Model Versioning**: Implement model versioning strategy

### Docker Deployment

```dockerfile
FROM python:3.9-slim

WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt

COPY . .
EXPOSE 8000

CMD ["python", "main.py"]
```

### Kubernetes Deployment

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: ai-trading-service
spec:
  replicas: 3
  selector:
    matchLabels:
      app: ai-trading-service
  template:
    metadata:
      labels:
        app: ai-trading-service
    spec:
      containers:
        - name: ai-trading-service
          image: ai-trading-service:latest
          ports:
            - containerPort: 8000
          env:
            - name: SERVER_HOST
              value: "0.0.0.0"
            - name: SERVER_PORT
              value: "8000"
```

## Troubleshooting

### Common Issues

**1. Model Training Fails**

- Check minimum data requirements (100+ candles)
- Verify data quality and format
- Ensure sufficient memory/GPU resources

**2. Prediction Errors**

- Verify model is loaded (`/model/info`)
- Check input data format and validation
- Review feature engineering pipeline

**3. Performance Issues**

- Monitor memory usage during training
- Use GPU acceleration if available
- Optimize batch sizes and sequence lengths

**4. API Errors**

- Check API documentation at `/docs`
- Verify request format and validation
- Review error logs for detailed messages

## Contributing

1. Fork the repository
2. Create a feature branch
3. Implement changes with tests
4. Submit pull request with description

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Support

For issues and questions:

- Create GitHub issue with detailed description
- Include logs and configuration details
- Provide reproducible examples

---

**Note**: This is a sophisticated AI trading system. Always test thoroughly in a safe environment before using with real trading capital. Past performance does not guarantee future results.
