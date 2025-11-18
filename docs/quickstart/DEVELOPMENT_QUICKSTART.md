# Development Quick Start - Bot Core

**Time Required:** 5-10 minutes
**Prerequisites:** Docker, Docker Compose, Git

---

## Quick Setup (5 Steps)

### 1. Clone Repository
```bash
git clone https://github.com/your-org/bot-core.git
cd bot-core
```

### 2. Configure Environment
```bash
# Copy environment template
cp .env.example .env

# Generate secure secrets
./scripts/generate-secrets.sh

# Edit .env with your API keys
nano .env
# Required: BINANCE_API_KEY, BINANCE_SECRET_KEY, OPENAI_API_KEY
# Set: BINANCE_TESTNET=true, TRADING_ENABLED=false
```

### 3. Start Development Environment
```bash
# Start with hot reload (recommended)
./scripts/bot.sh dev

# Or start with memory optimization
./scripts/bot.sh start --memory-optimized
```

### 4. Verify Services
```bash
# Check service status
./scripts/bot.sh status

# Test endpoints
curl http://localhost:8080/api/health
curl http://localhost:8000/health
curl http://localhost:3000/
```

### 5. Access Dashboard
- **Dashboard:** http://localhost:3000
- **Rust API:** http://localhost:8080/api/health
- **Python AI:** http://localhost:8000/health

---

## Development Workflow

### Making Changes

**Rust (rust-core-engine):**
```bash
cd rust-core-engine
cargo watch -x "run"  # Auto-reload on changes
cargo test            # Run tests
cargo clippy          # Lint code
```

**Python (python-ai-service):**
```bash
cd python-ai-service
uvicorn main:app --reload  # Auto-reload
pytest                      # Run tests
black .                     # Format code
```

**Frontend (nextjs-ui-dashboard):**
```bash
cd nextjs-ui-dashboard
npm run dev      # Development with HMR
npm run test     # Run tests
npm run lint     # Lint code
```

### Running Tests

```bash
# All tests
make test

# Specific service
make test-rust
make test-python
make test-frontend
```

### Viewing Logs

```bash
# All services
./scripts/bot.sh logs

# Specific service
./scripts/bot.sh logs --service rust-core-engine
```

---

## Common Issues

**Port Already in Use:**
```bash
# Check what's using the port
lsof -i :3000
# Kill the process or change port in .env
```

**Out of Memory:**
```bash
# Use memory-optimized mode
./scripts/bot.sh start --memory-optimized
```

**Services Not Starting:**
```bash
# Check logs
docker-compose logs

# Restart services
./scripts/bot.sh restart
```

---

## Next Steps

- Read [CONTRIBUTING.md](../CONTRIBUTING.md) for coding standards
- Review [TESTING_GUIDE.md](../TESTING_GUIDE.md) for testing practices
- Check [TROUBLESHOOTING.md](../TROUBLESHOOTING.md) for common issues

---

**Happy Coding!**
