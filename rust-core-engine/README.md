# Binance Trading Bot

A comprehensive, high-performance Rust-based trading bot for Binance Futures with real-time market data processing, multi-timeframe analysis, and AI-powered trading signals.

## Features

### 🚀 Real-time Market Data Processing

- **WebSocket Connections**: Stable, auto-reconnecting WebSocket connections to Binance Futures
- **Multi-timeframe Support**: Simultaneous tracking of 1m, 5m, 15m, 1h, 4h, and 1d charts
- **High-performance Caching**: In-memory caching with configurable size limits
- **Order Book Updates**: Real-time order book depth updates

### 🤖 AI-Powered Trading

- **Multi-timeframe Analysis**: Combines signals across different timeframes for better accuracy
- **Python AI Integration**: Sends market data to external Python AI service for analysis
- **Signal Confidence**: Weighted confidence scores for trading decisions
- **Risk-Reward Optimization**: Automatic calculation of optimal entry, stop-loss, and take-profit levels

### ⚡ High-Performance Architecture

- **Async/Await**: Built on Tokio for maximum concurrency
- **Event-driven**: Instant reaction to market movements with minimal latency
- **Thread-safe**: Lock-free data structures where possible
- **Scalable**: Configurable for multiple symbols and timeframes

### 🛡️ Risk Management

- **Position Sizing**: Intelligent position sizing based on account balance and risk parameters
- **Stop Loss/Take Profit**: Automatic order management with configurable levels
- **Maximum Positions**: Configurable limits on concurrent positions
- **Drawdown Protection**: Risk reduction during losing streaks

### 📊 Monitoring & Logging

- **Comprehensive Logging**: Detailed tracing with configurable log levels
- **Performance Metrics**: Real-time tracking of PnL, win rate, and other statistics
- **Connection Monitoring**: Health checks for WebSocket and API connections
- **System Metrics**: Memory usage, uptime, and cache statistics

### 🌐 HTTP API

- **RESTful Endpoints**: Complete API for external dashboard integration
- **Real-time Data**: Access to live market data and trading metrics
- **Position Management**: View and manage active positions
- **Performance Analytics**: Historical trading performance data

### 💾 Data Persistence

- **MongoDB Database**: Cloud-ready database for storing trade history and analysis
- **Trade Records**: Complete audit trail of all trading activities
- **Analysis History**: Historical market analysis results
- **Performance Tracking**: Long-term performance statistics

## Installation

### Prerequisites

- Rust 1.70+
- MongoDB (for database features)
- Python AI service (for market analysis)

### Build from Source

```bash
git clone <repository-url>
cd binance-trading-bot
cargo build --release
```

### Configuration

1. Copy the example configuration:

```bash
cp config.example.toml config.toml
```

2. Edit `config.toml` with your settings:

```toml
[binance]
api_key = "your_binance_api_key"
secret_key = "your_binance_secret_key"
testnet = true  # Set to false for live trading

[market_data]
symbols = ["BTCUSDT", "ETHUSDT"]
timeframes = ["1m", "5m", "15m", "1h", "4h", "1d"]
python_ai_service_url = "http://localhost:8000"

[trading]
enabled = false  # Set to true to enable live trading
max_positions = 5
risk_percentage = 2.0
```

## Usage

### Basic Usage

```bash
# Run with default configuration
./target/release/binance-trading-bot

# Run with custom configuration
./target/release/binance-trading-bot -c my-config.toml

# Enable verbose logging
./target/release/binance-trading-bot -vv
```

### Command Line Options

- `-c, --config <FILE>`: Configuration file path (default: config.toml)
- `-v, --verbose`: Increase log verbosity (use -vv for debug, -vvv for trace)

## API Endpoints

### Market Data

- `GET /api/market/prices` - Latest prices for all symbols
- `GET /api/market/overview` - Complete market overview with analysis
- `GET /api/market/candles/{symbol}/{timeframe}?limit=100` - Historical candle data

### Trading

- `GET /api/trading/positions` - Current open positions
- `GET /api/trading/account` - Account information
- `POST /api/trading/positions/{symbol}/close` - Force close position
- `GET /api/trading/performance` - Trading performance statistics

### Monitoring

- `GET /api/monitoring/system` - System metrics and health
- `GET /api/monitoring/trading` - Trading-specific metrics
- `GET /api/monitoring/connection` - Connection status

### Health Check

- `GET /api/health` - Simple health check

## Configuration Reference

### Binance Configuration

```toml
[binance]
api_key = "your_api_key"           # Binance API key
secret_key = "your_secret_key"     # Binance secret key
testnet = true                     # Use testnet (recommended for testing)
base_url = "https://testnet.binance.vision"
futures_base_url = "https://testnet.binancefuture.com"
```

### Market Data Configuration

```toml
[market_data]
symbols = ["BTCUSDT", "ETHUSDT"]   # Symbols to monitor
timeframes = ["1m", "5m", "1h"]    # Timeframes to track
kline_limit = 500                  # Historical candles to fetch
update_interval_ms = 1000          # Data refresh interval
cache_size = 1000                  # Max candles per timeframe
python_ai_service_url = "http://localhost:8000"  # AI service endpoint
```

### Trading Configuration

```toml
[trading]
enabled = false                    # Enable/disable live trading
max_positions = 5                  # Maximum concurrent positions
default_quantity = 0.01            # Default position size
risk_percentage = 2.0              # Risk per trade (%)
stop_loss_percentage = 2.0         # Default stop loss (%)
take_profit_percentage = 4.0       # Default take profit (%)
leverage = 1                       # Trading leverage
margin_type = "CROSSED"            # Margin type
```

### Database Configuration

```toml
[database]
url = "mongodb://botuser:defaultpassword@localhost:27017/trading_bot?authSource=admin"   # Database URL
max_connections = 10               # Connection pool size
enable_logging = false             # Enable SQL logging
```

### API Configuration

```toml
[api]
host = "0.0.0.0"                   # Bind address
port = 8080                        # HTTP port
cors_origins = ["*"]               # CORS origins
enable_metrics = true              # Enable metrics collection
```

## Python AI Service Integration

The bot expects a Python service running on the configured URL that accepts POST requests to `/api/analyze` with the following format:

### Request Format

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
      "volume": 1250.5
    }
  ],
  "analysis_type": "trend_analysis",
  "parameters": {}
}
```

### Expected Response

```json
{
  "symbol": "BTCUSDT",
  "timeframe": "1h",
  "timestamp": 1640995200000,
  "signal": "BUY",
  "confidence": 0.85,
  "indicators": {
    "rsi": 65.2,
    "macd": 0.5,
    "bb_position": 0.7
  },
  "analysis_details": {}
}
```

## Safety Features

- **Testnet Support**: Full support for Binance testnet for safe testing
- **Trading Disable**: Easy on/off switch for trading functionality
- **Position Limits**: Configurable maximum number of positions
- **Risk Controls**: Multiple layers of risk management
- **Graceful Shutdown**: Clean shutdown on SIGINT/SIGTERM

## Performance Considerations

- **Memory Usage**: Approximately 50-100MB for typical configurations
- **CPU Usage**: Low CPU usage during normal operation
- **Network**: Minimal bandwidth usage with efficient WebSocket connections
- **Latency**: Sub-millisecond reaction times to market events

## Troubleshooting

### Common Issues

1. **WebSocket Connection Failed**

   - Check internet connection
   - Verify Binance API endpoints are accessible
   - Check firewall settings

2. **Authentication Errors**

   - Verify API key and secret are correct
   - Ensure API key has futures trading permissions
   - Check if IP is whitelisted (if required)

3. **Database Errors**

   - Ensure MongoDB is running and accessible
   - Check database file permissions
   - Verify disk space availability

4. **Python AI Service Connection**
   - Ensure AI service is running
   - Check service URL configuration
   - Verify network connectivity

### Logging

Enable debug logging for troubleshooting:

```bash
./binance-trading-bot -vv
```

Log files are written to stdout/stderr and can be redirected:

```bash
./binance-trading-bot 2>&1 | tee bot.log
```

## Development

### Project Structure

```
src/
├── main.rs                 # Application entry point
├── config.rs              # Configuration management
├── binance/               # Binance API integration
│   ├── client.rs         # REST API client
│   ├── websocket.rs      # WebSocket client
│   └── types.rs          # Data structures
├── market_data/          # Market data processing
│   ├── processor.rs      # Main data processor
│   ├── cache.rs          # Data caching
│   └── analyzer.rs       # AI integration
├── trading/              # Trading engine
│   ├── engine.rs         # Main trading logic
│   ├── position_manager.rs  # Position tracking
│   └── risk_manager.rs   # Risk management
├── storage/              # Data persistence
│   └── mod.rs           # Database operations
├── monitoring/           # System monitoring
│   └── mod.rs           # Metrics and health
└── api/                 # HTTP API server
    └── mod.rs           # REST endpoints
```

### Building

```bash
# Debug build
cargo build

# Release build
cargo build --release

# Run tests
cargo test

# Check code
cargo clippy
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.

## Disclaimer

This software is for educational and research purposes only. Trading cryptocurrencies involves substantial risk of loss and is not suitable for all investors. The authors are not responsible for any financial losses incurred through the use of this software. Always test thoroughly with testnet before using with real funds.
