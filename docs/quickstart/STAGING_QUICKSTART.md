# Staging Quick Start - Bot Core

**Time Required:** 15-20 minutes
**Environment:** Staging/Pre-Production

---

## Prerequisites

- [ ] Staging server access (SSH)
- [ ] Docker & Docker Compose installed
- [ ] Git access to repository
- [ ] Staging database credentials
- [ ] API keys for staging environment

---

## Staging Deployment (8 Steps)

### 1. Server Preparation
```bash
# SSH into staging server
ssh user@staging-server

# Update system
sudo apt update && sudo apt upgrade -y

# Install Docker
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh
```

### 2. Clone Repository
```bash
# Create app directory
sudo mkdir -p /opt/bot-core
sudo chown $USER:$USER /opt/bot-core
cd /opt/bot-core

# Clone repository
git clone https://github.com/your-org/bot-core.git .
```

### 3. Configure Environment
```bash
# Copy staging environment
cp .env.example .env

# Generate secrets
./scripts/generate-secrets.sh

# Edit environment for staging
nano .env
```

**Staging Configuration:**
```bash
NODE_ENV=staging
LOG_LEVEL=debug
BINANCE_TESTNET=true      # Use testnet
TRADING_ENABLED=false     # Disable trading
DATABASE_URL=mongodb://staging-db:27017/bot_core_staging
```

### 4. Build Images
```bash
# Build all services
make build-fast

# Verify images
docker images | grep bot-core
```

### 5. Start Services
```bash
# Start with production profile
docker-compose --profile prod up -d

# Check status
docker-compose ps
```

### 6. Wait for Health Checks
```bash
# Monitor startup
watch -n 2 'docker-compose ps'

# Check health (wait 2-3 minutes)
curl http://localhost:8080/api/health
curl http://localhost:8000/health
```

### 7. Verify Functionality
```bash
# Test all endpoints
./scripts/verify-setup.sh

# Run smoke tests
make test-integration
```

### 8. Configure Monitoring
```bash
# Start monitoring stack
docker-compose --profile monitoring up -d

# Access Grafana
# URL: http://staging-server:3001
```

---

## Staging URLs

- **Dashboard:** http://staging-server:3000
- **Rust API:** http://staging-server:8080
- **Python AI:** http://staging-server:8000
- **Grafana:** http://staging-server:3001

---

## Testing in Staging

### Smoke Tests
```bash
# Run automated tests
make test-integration

# Manual smoke tests
curl http://localhost:8080/api/v1/binance/ping
curl -X POST http://localhost:8000/api/ai/predict
```

### Load Testing
```bash
# Install Apache Bench
sudo apt install apache2-utils

# Test API performance
ab -n 1000 -c 50 http://localhost:8080/api/health
```

### Security Testing
```bash
# Run security scan
make security-check

# Check for vulnerabilities
docker scan bot-core/rust-core-engine:latest
```

---

## Maintenance

### Viewing Logs
```bash
# Real-time logs
docker-compose logs -f

# Last 100 lines
docker-compose logs --tail=100
```

### Updating Staging
```bash
# Pull latest changes
git pull origin main

# Rebuild and restart
docker-compose down
make build-fast
docker-compose up -d
```

### Cleanup
```bash
# Remove old images
docker image prune -a

# Remove old logs
find /opt/bot-core/*/logs -name "*.log" -mtime +7 -delete
```

---

## Promoting to Production

After successful staging verification:

1. Tag release: `git tag -a v2.0.0 -m "Release 2.0.0"`
2. Push tag: `git push origin v2.0.0`
3. Follow [Production Deployment Guide](../PRODUCTION_DEPLOYMENT_GUIDE.md)

---

**Support:** See [TROUBLESHOOTING.md](../TROUBLESHOOTING.md)
