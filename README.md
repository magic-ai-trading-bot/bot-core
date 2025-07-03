# Bot Core Monorepo

A comprehensive cryptocurrency trading bot monorepo with AI-powered market analysis, real-time trading capabilities, and a modern web dashboard.

## 🏗️ Architecture Overview

This monorepo contains three main services:

1. **Rust Core Engine** (`rust-core-engine/`) - High-performance trading engine
2. **Python AI Service** (`python-ai-service/`) - Machine learning powered market analysis
3. **Next.js UI Dashboard** (`nextjs-ui-dashboard/`) - Modern web interface

## 🚀 Quick Start

### Prerequisites

- Docker and Docker Compose
- Git
- (Optional) Node.js 18+, Rust 1.74+, Python 3.9+ for local development

### 1. Clone and Setup

```bash
git clone <repository-url>
cd bot-core
```

### 2. Configure Environment Variables

Create a `.env` file in the root directory with the following variables:

```env
# Binance API Configuration
BINANCE_API_KEY=your_binance_api_key_here
BINANCE_SECRET_KEY=your_binance_secret_key_here
BINANCE_TESTNET=true
TRADING_ENABLED=false

# Database Configuration
POSTGRES_PASSWORD=secure_postgres_password_here
REDIS_PASSWORD=secure_redis_password_here

# Monitoring
GRAFANA_PASSWORD=secure_grafana_password_here

# Application Settings
LOG_LEVEL=INFO
NODE_ENV=development
REACT_APP_API_URL=http://localhost:8080
REACT_APP_AI_API_URL=http://localhost:8000
```

### 3. Run with Docker Compose

```bash
# For PRODUCTION - Start core services
docker-compose up -d

# For DEVELOPMENT with hot reload - Setup first
make setup-dev

# Then start development environment
make dev

# Or start with optional services (production)
docker-compose --profile postgres --profile redis --profile monitoring up -d
```

### 4. Access the Services

- **Web Dashboard**: http://localhost:3000
- **Rust Trading API**: http://localhost:8080
- **Python AI API**: http://localhost:8000
- **Grafana Monitoring**: http://localhost:3001 (if monitoring profile enabled)

## 📋 Services Overview

### Rust Core Engine (Port 8080)

High-performance trading engine built with Rust featuring:

- **Real-time Market Data**: WebSocket connections to Binance
- **Trading Execution**: Automated order placement and management
- **Risk Management**: Position sizing and stop-loss management
- **Data Storage**: SQLite/PostgreSQL for trade history
- **REST API**: HTTP endpoints for dashboard integration

**Key Features:**

- Multi-symbol trading support
- Real-time price monitoring
- Technical analysis integration
- Risk management controls
- Performance metrics

### Python AI Service (Port 8000)

Machine learning service for market analysis:

- **AI Models**: LSTM, GRU, and Transformer models
- **Technical Analysis**: 15+ technical indicators
- **Signal Generation**: Long/Short/Neutral trading signals
- **Model Training**: Custom model training with historical data
- **FastAPI**: REST API for real-time predictions

**Key Features:**

- Multiple AI model types
- Real-time signal generation
- Model retraining capabilities
- Technical indicator calculation
- Confidence scoring

### Next.js UI Dashboard (Port 3000)

Modern web dashboard built with React and Next.js:

- **Real-time Updates**: Live trading data and charts
- **Performance Analytics**: Trading performance metrics
- **AI Insights**: Market analysis and signal visualization
- **Settings Management**: Trading parameters and risk controls
- **Responsive Design**: Mobile-friendly interface

**Key Features:**

- Real-time charting
- Trading history
- AI signal visualization
- Bot configuration
- Performance analytics

## 🛠️ Development Setup

### Local Development

Each service can be run locally for development:

#### Rust Core Engine

```bash
cd rust-core-engine
cargo run -- --config config.toml
```

#### Python AI Service

```bash
cd python-ai-service
pip install -r requirements.txt
python main.py
```

#### Next.js Dashboard

```bash
cd nextjs-ui-dashboard
npm install
npm run dev
```

### Docker Profiles and Development Mode

The docker-compose.yml supports multiple profiles:

```bash
# Core services only (Production)
docker-compose up -d

# Development mode with hot reload
make dev
# or
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up --build

# With PostgreSQL database
docker-compose --profile postgres up -d

# With Redis caching
docker-compose --profile redis up -d

# With monitoring (Prometheus + Grafana)
docker-compose --profile monitoring up -d

# All services
docker-compose --profile postgres --profile redis --profile monitoring up -d
```

### Development with Hot Reload

For development with automatic code reloading:

```bash
# Start all services in development mode
make dev

# Start individual services in development mode
make dev-rust      # Rust with cargo-watch
make dev-python    # Python with uvicorn --reload
make dev-frontend  # Vite with hot module replacement

# View development logs
make dev-logs

# Stop development services
make dev-stop

# Rebuild development containers
make dev-rebuild
```

**Development Features:**

- **Hot Reload**: All services automatically restart on code changes
- **Volume Mounting**: Source code is mounted for instant updates
- **Debug Logging**: Enhanced logging for development
- **Development Dependencies**: Additional dev tools and debuggers

## 🔧 Configuration

### Environment Variables

| Variable             | Description            | Default           |
| -------------------- | ---------------------- | ----------------- |
| `BINANCE_API_KEY`    | Binance API key        | Required          |
| `BINANCE_SECRET_KEY` | Binance secret key     | Required          |
| `BINANCE_TESTNET`    | Use testnet            | `true`            |
| `TRADING_ENABLED`    | Enable live trading    | `false`           |
| `LOG_LEVEL`          | Logging level          | `INFO`            |
| `POSTGRES_PASSWORD`  | PostgreSQL password    | `defaultpassword` |
| `REDIS_PASSWORD`     | Redis password         | `defaultpassword` |
| `GRAFANA_PASSWORD`   | Grafana admin password | `admin`           |

### Service Configuration

Each service has its own configuration file:

- **Rust**: `rust-core-engine/config.toml`
- **Python**: `python-ai-service/config.yaml`
- **Next.js**: `nextjs-ui-dashboard/vite.config.ts`

## 🔒 Security Considerations

### API Keys

- Store Binance API keys securely
- Use testnet for development
- Never commit API keys to version control

### Network Security

- Services communicate over internal Docker network
- Use proper firewall rules in production
- Consider using SSL/TLS certificates

### Database Security

- Use strong passwords for database services
- Backup trading data regularly
- Consider encryption for sensitive data

## 📊 Monitoring

### Health Checks

All services include health check endpoints:

- **Rust**: `GET /health`
- **Python**: `GET /health`
- **Next.js**: `GET /health`

### Metrics (with monitoring profile)

- **Prometheus**: Metrics collection (port 9090)
- **Grafana**: Dashboards and visualization (port 3001)

## 🚢 Production Deployment

### Docker Images

Build production images:

```bash
# Build all services
docker-compose build

# Build specific service
docker-compose build rust-core-engine
```

### Kubernetes Deployment

For Kubernetes deployment, consider:

1. Converting docker-compose to Kubernetes manifests
2. Using Helm charts for easier management
3. Implementing proper secrets management
4. Setting up ingress controllers

## 📝 API Documentation

### Rust Core Engine API

- **GET /health** - Health check
- **GET /api/status** - Trading bot status
- **GET /api/positions** - Current positions
- **GET /api/history** - Trading history
- **POST /api/trade** - Manual trade execution

### Python AI Service API

- **GET /health** - Health check
- **POST /analyze** - Analyze market data
- **POST /train** - Train AI model
- **GET /model/info** - Model information
- **GET /config** - Service configuration

Full API documentation available at:

- Rust API: http://localhost:8080/docs
- Python API: http://localhost:8000/docs

## 🔧 Troubleshooting

### Common Issues

1. **Port conflicts**: Ensure ports 3000, 8000, 8080 are available
2. **Docker permission errors**: Add user to docker group
3. **Build failures**: Check Docker resources and internet connection
4. **Service startup failures**: Check logs with `docker-compose logs <service>`

### Debugging

```bash
# View logs for all services
docker-compose logs -f

# View logs for specific service
docker-compose logs -f rust-core-engine

# Access container shell
docker-compose exec rust-core-engine sh
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests if applicable
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## ⚠️ Disclaimer

This software is for educational and development purposes only. Cryptocurrency trading involves significant financial risk. The authors are not responsible for any financial losses incurred through the use of this software.

Always test with small amounts and use testnet environments before any live trading.
