# 🚀 Crypto Trading Bot - Multi-Service Architecture

A comprehensive cryptocurrency trading bot with AI-powered market analysis, built with microservices architecture using Python (AI), Rust (Core Engine), and Next.js (Frontend).

## 🎯 Quick Start

### Prerequisites

- Docker & Docker Compose
- Git

### 🔥 One-Command Start

```bash
# Clone the repository
git clone <repository-url>
cd bot-core

# Start all services with memory optimization
./scripts/bot.sh start --memory-optimized

# Or start in development mode
./scripts/bot.sh dev

# Or regular production mode
./scripts/bot.sh start
```

### 🎯 Access URLs

- **📊 Frontend Dashboard**: http://localhost:3000
- **🦀 Rust Core Engine**: http://localhost:8080/api/health
- **🐍 Python AI Service**: http://localhost:8000/health

## 📋 Available Commands

The bot uses a single control script that handles all operations:

```bash
# Service Management
./scripts/bot.sh start              # Start all services (production)
./scripts/bot.sh dev                # Start in development mode
./scripts/bot.sh stop               # Stop all services
./scripts/bot.sh restart            # Restart all services

# Monitoring & Logs
./scripts/bot.sh status             # Show service status & resource usage
./scripts/bot.sh logs               # Show logs for all services
./scripts/bot.sh logs --service python-ai-service  # Show logs for specific service

# Build & Maintenance
./scripts/bot.sh build              # Build all services
./scripts/bot.sh build --service rust-core-engine  # Build specific service
./scripts/bot.sh clean              # Clean up containers and volumes

# Help
./scripts/bot.sh help               # Show all available commands
```

## 🎛️ Configuration Modes

### Memory Optimized (Recommended for Low-Memory Systems)

```bash
./scripts/bot.sh start --memory-optimized
```

- Python AI Service: 1.5GB limit
- Rust Core Engine: 1GB limit
- Frontend Dashboard: 512MB limit

### Development Mode

```bash
./scripts/bot.sh dev
```

- Hot reload for all services
- Debug logging enabled
- Source code mounted for live editing

### Production Mode

```bash
./scripts/bot.sh start
```

- Optimized builds
- Full resource allocation
- Production logging

## 🔧 Environment Configuration

The bot uses environment variables for configuration. Copy `config.env` to `.env` and customize:

```bash
cp config.env .env
# Edit .env with your preferred settings
```

### Key Configuration Options:

```env
# Resource Limits
PYTHON_MEMORY_LIMIT=2G
RUST_MEMORY_LIMIT=2G
FRONTEND_MEMORY_LIMIT=1G

# API Keys
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
BINANCE_TESTNET=true

# Security
TRADING_ENABLED=false  # Set to true only when ready for live trading
```

## 🏗️ Architecture

### Services:

1. **🐍 Python AI Service** (Port 8000)

   - Machine learning models for market analysis
   - Technical indicators calculation
   - Real-time predictions

2. **🦀 Rust Core Engine** (Port 8080)

   - High-performance trading engine
   - Binance WebSocket connections
   - Risk management & position control

3. **⚛️ Next.js Frontend** (Port 3000)
   - Real-time trading dashboard
   - Interactive charts and analytics
   - User-friendly controls

### Key Features:

- 🔄 **Real-time Market Data**: Live WebSocket connections
- 🤖 **AI-Powered Analysis**: Machine learning market predictions
- 📊 **Advanced Charts**: Technical analysis visualization
- 🛡️ **Risk Management**: Built-in safety mechanisms
- 📱 **Responsive UI**: Modern, mobile-friendly interface

## 🐳 Docker Structure

The project uses a single `docker-compose.yml` file with profiles for different modes:

```yaml
# Production services (default)
docker compose up -d

# Development services with hot reload
docker compose --profile dev up -d
```

## 🚀 Performance Optimization

### Memory Usage (with --memory-optimized):

- **Frontend**: ~3MB (0.58% of 512MB limit)
- **Rust Core**: ~8MB (0.75% of 1GB limit)
- **Python AI**: ~237MB (15.44% of 1.5GB limit)
- **Total**: ~248MB RAM usage

### CPU Usage:

- All services optimized for low CPU consumption
- Multi-threaded processing where beneficial
- Efficient resource allocation

## 🔍 Monitoring & Debugging

### Check Service Status:

```bash
./scripts/bot.sh status
```

### View Resource Usage:

```bash
docker stats --no-stream
```

### Debug Specific Service:

```bash
./scripts/bot.sh logs --service <service-name>
```

### Available Services:

- `python-ai-service`
- `rust-core-engine`
- `nextjs-ui-dashboard`

## 🛠️ Development

### Hot Reload Development:

```bash
./scripts/bot.sh dev
```

This enables:

- Live code reloading for all services
- Debug logging
- Source code mounting

### Build Individual Services:

```bash
./scripts/bot.sh build --service python-ai-service
./scripts/bot.sh build --service rust-core-engine
./scripts/bot.sh build --service nextjs-ui-dashboard
```

## 🔒 Security

- All services communicate through internal Docker network
- API keys stored in environment variables
- Testnet mode enabled by default
- Trading disabled by default (safety first)

## 🚨 Safety Features

- **Testnet Mode**: All trading operations use Binance Testnet by default
- **Trading Disabled**: Manual activation required for live trading
- **Risk Management**: Built-in position size limits and stop-loss mechanisms
- **Health Checks**: Automatic service monitoring and restart

## 📝 Troubleshooting

### Common Issues:

1. **Out of Memory**: Use `--memory-optimized` flag
2. **Service Unhealthy**: Check logs with `./scripts/bot.sh logs --service <name>`
3. **Port Conflicts**: Ensure ports 3000, 8000, 8080 are available
4. **Permission Errors**: Ensure Docker is running and user has permissions

### Clean Reset:

```bash
./scripts/bot.sh stop
./scripts/bot.sh clean
./scripts/bot.sh start --memory-optimized
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Test with `./scripts/bot.sh dev`
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License.

## ⚠️ Disclaimer

This software is for educational and testing purposes only. Cryptocurrency trading involves significant risks. Never risk more than you can afford to lose. Always test thoroughly with testnet before considering live trading.

---

**🎯 Happy Trading!** 🚀
