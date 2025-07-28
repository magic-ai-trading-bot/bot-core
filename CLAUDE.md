# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## 🚀 Quick Start Commands

```bash
# Start all services (production mode with memory optimization)
./scripts/bot.sh start --memory-optimized

# Start in development mode with hot reload
./scripts/bot.sh dev

# View logs for specific service
./scripts/bot.sh logs --service python-ai-service
./scripts/bot.sh logs --service rust-core-engine
./scripts/bot.sh logs --service nextjs-ui-dashboard

# Check service status
./scripts/bot.sh status

# Clean restart
./scripts/bot.sh stop
./scripts/bot.sh clean
./scripts/bot.sh start --memory-optimized
```

## 🏗️ Architecture Overview

This is a **microservices cryptocurrency trading bot** with three core components:

1. **Rust Core Engine** (`rust-core-engine/`) - Port 8080
   - High-performance trading execution
   - Binance WebSocket integration
   - JWT authentication
   - Risk management
   - MongoDB persistence

2. **Python AI Service** (`python-ai-service/`) - Port 8000
   - Machine learning models (LSTM, GRU, Transformer)
   - Technical indicators calculation
   - FastAPI REST endpoints
   - OpenAI integration
   - Real-time market predictions

3. **Next.js Dashboard** (`nextjs-ui-dashboard/`) - Port 3000
   - React + TypeScript + Vite
   - Shadcn/UI components
   - Real-time WebSocket updates
   - 3D visualizations
   - i18n support

## 🔧 Development Commands

### Building Services
```bash
# Build all services with optimized strategy
make build

# Build individual services
make build-rust
make build-python
make build-frontend

# Memory-optimized sequential build
make build-fast
```

### Testing (Note: Test files need implementation)
```bash
# Run all tests
make test

# Service-specific tests
make test-rust
make test-python
make test-frontend

# Integration tests
make test-rust-python
make test-dashboard-rust
make test-websocket
```

### Linting & Code Quality
```bash
# Lint all services
make lint

# Service-specific linting
make lint-rust      # cargo clippy
make lint-python    # flake8
make lint-frontend  # ESLint
```

### Local Development (without Docker)
```bash
# Rust service
cd rust-core-engine && cargo run -- --config config.toml

# Python service
cd python-ai-service && python main.py

# Frontend
cd nextjs-ui-dashboard && npm run dev
```

## 📝 Key Configuration Files

- **Environment**: `config.env` → `.env` (copy on first run)
- **Rust config**: `rust-core-engine/config.toml`
- **Python config**: `python-ai-service/config.yaml`
- **Frontend build**: `nextjs-ui-dashboard/vite.config.ts`
- **Docker orchestration**: `docker-compose.yml`

## 🔄 Service Communication Flow

```
Dashboard (3000) → Rust Engine (8080) → Python AI (8000)
                         ↓
                    MongoDB (27017)
                         ↓
                   Binance WebSocket
```

All services communicate through internal Docker network (`bot-network`).

## ⚡ Performance Optimization

### Memory Limits (--memory-optimized)
- Python AI: 1.5GB
- Rust Core: 1GB
- Frontend: 512MB

### Resource Monitoring
```bash
# Check resource usage
docker stats --no-stream

# View service health
make health
```

## 🛡️ Security Considerations

1. **Testnet by default**: `BINANCE_TESTNET=true`
2. **Trading disabled**: `TRADING_ENABLED=false` (manual activation required)
3. **JWT tokens**: Inter-service authentication
4. **Internal networking**: Services isolated from host
5. **Environment secrets**: Use `.env` file, never commit

## 🚨 Common Issues & Solutions

1. **Out of Memory**: Use `--memory-optimized` flag
2. **Port Conflicts**: Ensure 3000, 8000, 8080 are free
3. **Service Unhealthy**: Check logs with `./scripts/bot.sh logs --service <name>`
4. **Build Failures**: Try `make build-fast` for sequential builds

## 📂 Project Structure

```
bot-core/
├── rust-core-engine/       # Trading engine
│   ├── src/
│   ├── Cargo.toml
│   └── config.toml
├── python-ai-service/      # AI/ML service
│   ├── models/
│   ├── services/
│   └── config.yaml
├── nextjs-ui-dashboard/    # Frontend
│   ├── src/
│   ├── package.json
│   └── vite.config.ts
├── scripts/               # Utility scripts
│   └── bot.sh            # Main control script
├── docker-compose.yml    # Service orchestration
├── Makefile             # Development tasks
└── config.env          # Environment template
```

## 🔑 Important Notes

- **No test implementations**: Test commands exist but test files are missing
- **Hardcoded API keys**: Some keys in docker-compose.yml should use env vars
- **Paper trading**: Always test with testnet before live trading
- **Resource limits**: Essential for stability on constrained systems
- **Hot reload**: Available in dev mode for all services