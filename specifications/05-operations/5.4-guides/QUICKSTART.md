# Quick Start Guide - Bot Core

Three environment setups: Development, Staging, Production.

---

## Development

**Time Required:** 5-10 minutes
**Prerequisites:** Docker, Docker Compose, Git

### 1. Clone Repository
```bash
git clone https://github.com/your-org/bot-core.git
cd bot-core
```

### 2. Configure Environment
```bash
cp .env.example .env
./scripts/generate-secrets.sh
nano .env
# Required: BINANCE_API_KEY, BINANCE_SECRET_KEY, OPENAI_API_KEY
# Set: BINANCE_TESTNET=true, TRADING_ENABLED=false
```

### 3. Start Services
```bash
./scripts/bot.sh dev                         # Hot reload (recommended)
./scripts/bot.sh start --memory-optimized    # Memory-constrained systems
```

### 4. Verify
```bash
./scripts/bot.sh status
curl http://localhost:8080/api/health
curl http://localhost:8000/health
curl http://localhost:3000/
```

### 5. Access Dashboard
- **Dashboard:** http://localhost:3000
- **Rust API:** http://localhost:8080/api/health
- **Python AI:** http://localhost:8000/health

### Development Workflow

**Rust:**
```bash
cd rust-core-engine
cargo watch -x "run"   # Auto-reload
cargo test             # Run tests
cargo clippy           # Lint
```

**Python:**
```bash
cd python-ai-service
uvicorn main:app --reload
pytest
black .
```

**Frontend:**
```bash
cd nextjs-ui-dashboard
npm run dev
npm run test
npm run lint
```

**All tests:**
```bash
make test          # All services
make test-rust
make test-python
make test-frontend
```

**Logs:**
```bash
./scripts/bot.sh logs
./scripts/bot.sh logs --service rust-core-engine
```

### Common Dev Issues

```bash
# Port in use
lsof -i :3000

# Out of memory
./scripts/bot.sh start --memory-optimized

# Services not starting
docker-compose logs
./scripts/bot.sh restart
```

---

## Staging

**Time Required:** 15-20 minutes

### Prerequisites
- [ ] Staging server access (SSH)
- [ ] Docker & Docker Compose installed
- [ ] Git access to repository
- [ ] Staging database credentials
- [ ] API keys for staging environment

### 1. Server Preparation
```bash
ssh user@staging-server
sudo apt update && sudo apt upgrade -y
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
```

### 2. Clone Repository
```bash
sudo mkdir -p /opt/bot-core
sudo chown $USER:$USER /opt/bot-core
cd /opt/bot-core
git clone https://github.com/your-org/bot-core.git .
```

### 3. Configure Environment
```bash
cp .env.example .env
./scripts/generate-secrets.sh
nano .env
```

**Staging `.env` values:**
```bash
NODE_ENV=staging
LOG_LEVEL=debug
BINANCE_TESTNET=true
TRADING_ENABLED=false
DATABASE_URL=mongodb://staging-db:27017/bot_core_staging
```

### 4. Build & Start
```bash
make build-fast
docker-compose --profile prod up -d
docker-compose ps
```

### 5. Health Checks
```bash
watch -n 2 'docker-compose ps'
curl http://localhost:8080/api/health
curl http://localhost:8000/health
```

### 6. Verify & Monitor
```bash
./scripts/verify-setup.sh
make test-integration
docker-compose --profile monitoring up -d
# Grafana: http://staging-server:3001
```

### Staging URLs
- **Dashboard:** http://staging-server:3000
- **Rust API:** http://staging-server:8080
- **Python AI:** http://staging-server:8000
- **Grafana:** http://staging-server:3001

### Testing in Staging
```bash
# Smoke tests
curl http://localhost:8080/api/v1/binance/ping
curl -X POST http://localhost:8000/api/ai/predict

# Load test
ab -n 1000 -c 50 http://localhost:8080/api/health

# Security scan
make security-check
```

### Promoting to Production
1. Tag release: `git tag -a v2.0.0 -m "Release 2.0.0"`
2. Push tag: `git push origin v2.0.0`
3. Follow `specifications/04-deployment/PRODUCTION_DEPLOYMENT_GUIDE.md`

---

## Production

**Time Required:** 30-40 minutes
**WARNING:** Follow `PRODUCTION_DEPLOYMENT_GUIDE.md` for comprehensive instructions.

### Pre-Deployment Checklist
- [ ] All tests passing (2,411+ tests)
- [ ] Security scan completed (0 HIGH/CRITICAL vulnerabilities)
- [ ] Staging environment tested successfully
- [ ] Database backup completed
- [ ] Rollback plan documented
- [ ] Team notified
- [ ] Monitoring configured

### 1. Access Production Server
```bash
ssh -i ~/.ssh/production-key.pem user@production-server
hostname  # Expected: production-server-01
```

### 2. Backup Current State
```bash
BACKUP_DATE=$(date +%Y%m%d-%H%M%S)
mkdir -p /opt/bot-core/backups/$BACKUP_DATE
mongodump --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/backups/$BACKUP_DATE/mongodb-backup.tar.gz \
  --gzip
ls -lh /opt/bot-core/backups/$BACKUP_DATE/
```

### 3. Stop Services
```bash
cd /opt/bot-core
docker-compose down
docker ps
```

### 4. Update Codebase
```bash
git fetch origin
git checkout v2.0.0
git rev-parse HEAD
```

### 5. Validate Configuration
```bash
./scripts/validate-env.sh
source .env
echo $NODE_ENV          # Expected: production
echo $BINANCE_TESTNET   # Expected: false (or true for testnet)
echo $TRADING_ENABLED   # Expected: false (until verified)
```

### 6. Build & Deploy
```bash
make build
docker images | grep bot-core
docker-compose --profile prod up -d
docker-compose logs -f
```

### 7. Health Verification (CRITICAL)
```bash
sleep 180  # Wait 2-3 minutes
curl http://localhost:8080/api/health | jq
curl http://localhost:8000/health | jq
curl -I http://localhost:3000/
```

### 8. Functional Testing
```bash
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}'
curl http://localhost:8080/api/v1/binance/ping
wscat -c ws://localhost:8080/ws
```

### 9. Enable Monitoring
```bash
docker-compose --profile monitoring up -d
# Grafana: http://production-server:3001
# Prometheus: http://production-server:9090
```

### Post-Deployment Checks
```bash
# Performance
ab -n 100 -c 10 http://localhost:8080/api/health
# Expected: < 50ms mean, 0 failed requests

# Resources
docker stats --no-stream
# rust-core-engine: < 10% CPU, < 1GB RAM
# python-ai-service: < 15% CPU, < 2GB RAM

# Security
docker-compose logs | grep -i 'api.*key\|secret\|password'  # No secrets
```

### Rollback Procedure
```bash
docker-compose down
git checkout <previous-tag>
mongorestore --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/backups/$BACKUP_DATE/mongodb-backup.tar.gz \
  --gzip --drop
docker-compose up -d
./scripts/bot.sh status
```

### Enabling Live Trading (CRITICAL)

**ONLY after 24+ hours of stable operation (0 crashes, 0 critical errors):**

```bash
nano .env
# Set: TRADING_ENABLED=true
# Set: BINANCE_TESTNET=false (if production)

docker-compose restart rust-core-engine
docker-compose logs -f rust-core-engine | grep -i trading
# Expected: "Trading engine: ENABLED"

# Start with VERY small position sizes
# config.toml: max_position_size = 0.001
```

### Monitoring (First 24 Hours)
- **Hour 1:** Every 5 minutes — logs, resources, critical flows
- **Hours 2-24:** Every 30 minutes — error rates, performance, memory leaks

---

## Support Resources

- Full deployment: `specifications/04-deployment/PRODUCTION_DEPLOYMENT_GUIDE.md`
- Operations manual: `specifications/05-operations/5.1-operations-manual/`
- Troubleshooting: `specifications/05-operations/5.2-troubleshooting/TROUBLESHOOTING-GUIDE.md`
- Contributing: `specifications/05-operations/5.4-guides/CONTRIBUTING.md`
