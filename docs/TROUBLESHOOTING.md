# Troubleshooting Guide - Bot-Core

Comprehensive troubleshooting guide for **Bot-Core** cryptocurrency trading platform. This guide covers common issues, diagnostic procedures, and solutions for all services.

---

## Table of Contents

1. [General Troubleshooting Approach](#general-troubleshooting-approach)
2. [Service Startup Issues](#service-startup-issues)
3. [Port Conflicts](#port-conflicts)
4. [Memory Issues](#memory-issues)
5. [Docker Issues](#docker-issues)
6. [Database Connection Failures](#database-connection-failures)
7. [Test Failures](#test-failures)
8. [Build Failures](#build-failures)
9. [Performance Issues](#performance-issues)
10. [Security Scan Issues](#security-scan-issues)
11. [WebSocket Connection Issues](#websocket-connection-issues)
12. [Trading Execution Issues](#trading-execution-issues)
13. [CI/CD Pipeline Failures](#cicd-pipeline-failures)
14. [Diagnostic Commands](#diagnostic-commands)

---

## General Troubleshooting Approach

### Six-Step Methodology

1. **Identify** - Determine what's failing
2. **Isolate** - Narrow down the root cause
3. **Diagnose** - Understand why it's failing
4. **Resolve** - Apply fix
5. **Verify** - Confirm resolution
6. **Document** - Record for future reference

### Quick Health Check

```bash
# Check all services status
./scripts/bot.sh status

# Check service health endpoints
curl http://localhost:8080/health    # Rust API
curl http://localhost:8000/health    # Python AI
curl http://localhost:3000/          # Dashboard

# Check resource usage
docker stats --no-stream

# Check logs
./scripts/bot.sh logs
```

---

## Service Startup Issues

### Symptom: Service Won't Start

**Indicators:**
- Container exits immediately with code 1
- Service shows as "Restarting" in `docker compose ps`
- Health checks never pass

**Diagnosis:**
```bash
# Check service status
docker compose ps

# Check exit code
docker inspect rust-core-engine --format='{{.State.ExitCode}}'
docker inspect python-ai-service --format='{{.State.ExitCode}}'
docker inspect nextjs-ui-dashboard --format='{{.State.ExitCode}}'

# Check logs for errors
docker compose logs rust-core-engine
docker compose logs python-ai-service
docker compose logs nextjs-ui-dashboard

# Check environment variables
docker inspect rust-core-engine --format='{{range .Config.Env}}{{println .}}{{end}}'
```

### Solution 1: Missing Environment Variables

**Cause:** Required environment variables not set in `.env`

**Fix:**
```bash
# Verify .env file exists
ls -la .env

# Validate environment variables
./scripts/validate-env.sh

# Check for missing secrets
cat .env | grep -E "^[A-Z_]+=\s*$"

# Generate missing secrets
./scripts/generate-secrets.sh

# Required variables:
# - DATABASE_URL
# - BINANCE_API_KEY
# - BINANCE_API_SECRET
# - BINANCE_TESTNET=true
# - TRADING_ENABLED=false
# - JWT_SECRET
# - INTER_SERVICE_TOKEN
```

### Solution 2: Configuration Errors

**Cause:** Invalid syntax in configuration files

**Fix:**
```bash
# Validate Rust config
cd rust-core-engine
cat config.toml | python3 -c "import sys, toml; toml.load(sys.stdin)" 2>&1

# Validate Python config
cd python-ai-service
cat config.yaml | python3 -c "import sys, yaml; yaml.safe_load(sys.stdin)" 2>&1

# Fix syntax errors in config files
nano rust-core-engine/config.toml
nano python-ai-service/config.yaml
```

### Solution 3: Dependency Issues

**Cause:** Required services not running

**Fix:**
```bash
# Check if MongoDB is running
docker compose ps mongodb

# Start dependencies first
docker compose up -d mongodb
sleep 10

# Then start application services
docker compose up -d rust-core-engine python-ai-service nextjs-ui-dashboard

# Check all services are running
docker compose ps
```

---

## Port Conflicts

### Symptom: Port Already in Use

**Error Messages:**
```
Error starting userland proxy: listen tcp 0.0.0.0:8080: bind: address already in use
Error starting userland proxy: listen tcp 0.0.0.0:3000: bind: address already in use
```

**Diagnosis:**
```bash
# Check which process is using ports
lsof -i :3000    # Dashboard
lsof -i :8080    # Rust API
lsof -i :8000    # Python AI
lsof -i :27017   # MongoDB

# Alternative (Linux)
netstat -tulpn | grep :3000
netstat -tulpn | grep :8080
netstat -tulpn | grep :8000
```

### Solution 1: Kill Conflicting Process

```bash
# Kill process using port 8080
kill -9 $(lsof -t -i:8080)

# Kill process using port 3000
kill -9 $(lsof -t -i:3000)

# Kill process using port 8000
kill -9 $(lsof -t -i:8000)

# Restart services
./scripts/bot.sh restart
```

### Solution 2: Change Port Mapping

```bash
# Edit docker-compose.yml
nano docker-compose.yml

# Change port mapping (example for Rust API)
# From:
ports:
  - "8080:8080"

# To:
ports:
  - "8081:8080"  # Map host 8081 to container 8080

# Restart services
docker compose down
docker compose up -d
```

---

## Memory Issues

### Symptom: Out of Memory (OOM) Errors

**Indicators:**
- Container restarts frequently
- `docker inspect` shows `OOMKilled: true`
- Services become unresponsive

**Diagnosis:**
```bash
# Check if OOM killed
docker inspect rust-core-engine --format='{{.State.OOMKilled}}'
docker inspect python-ai-service --format='{{.State.OOMKilled}}'

# Check memory usage
docker stats --no-stream

# Check system memory
free -h  # Linux
vm_stat  # macOS

# Check restart count
docker inspect rust-core-engine --format='{{.RestartCount}}'
```

### Solution 1: Use Memory-Optimized Mode

```bash
# Stop services
./scripts/bot.sh stop

# Start with memory optimization
./scripts/bot.sh start --memory-optimized

# Verify memory limits applied
docker inspect rust-core-engine --format='{{.HostConfig.Memory}}'
docker inspect python-ai-service --format='{{.HostConfig.Memory}}'
```

**Memory limits in optimized mode:**
- Rust Core Engine: 1GB
- Python AI Service: 1.5GB
- Next.js Dashboard: 512MB
- MongoDB: 2GB

### Solution 2: Increase Docker Memory

```bash
# macOS: Docker Desktop → Preferences → Resources
# Set memory to at least 6GB (8GB recommended)

# Linux: Edit /etc/docker/daemon.json
{
  "default-shm-size": "2G",
  "default-ulimits": {
    "memlock": {
      "Hard": -1,
      "Soft": -1
    }
  }
}

# Restart Docker
sudo systemctl restart docker
```

### Solution 3: Reduce Python ML Model Memory

```bash
# Edit python-ai-service/config.yaml
cd python-ai-service
nano config.yaml

# Reduce batch size
ml:
  batch_size: 16  # Reduce from 32
  max_sequence_length: 60  # Reduce from 100

# Restart Python service
docker compose restart python-ai-service
```

---

## Docker Issues

### Symptom: Docker Build Failures

**Diagnosis:**
```bash
# Check Docker version
docker --version
docker compose --version

# Check disk space
df -h

# Check Docker disk usage
docker system df
```

### Solution 1: Clean Docker Cache

```bash
# Clean build cache
docker builder prune -a

# Remove unused containers
docker container prune

# Remove unused images
docker image prune -a

# Remove unused volumes
docker volume prune

# Clean everything (CAREFUL: removes all unused data)
docker system prune -a --volumes

# Verify cleanup
docker system df
```

### Solution 2: Fix Dockerfile Issues

```bash
# Rust build issues
cd rust-core-engine
docker build -t rust-core-engine:test .

# Python build issues
cd python-ai-service
docker build -t python-ai-service:test .

# Frontend build issues
cd nextjs-ui-dashboard
docker build -t nextjs-ui-dashboard:test .
```

---

## Database Connection Failures

### Symptom: Cannot Connect to MongoDB

**Error Messages:**
```
Error: MongoNetworkError: failed to connect to server
Error: MongoTimeoutError: Server selection timed out after 30000 ms
```

**Diagnosis:**
```bash
# Check MongoDB is running
docker compose ps mongodb

# Check MongoDB logs
docker compose logs mongodb

# Test MongoDB connection
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "db.adminCommand({ping: 1})"

# Check network connectivity
docker network ls
docker network inspect bot-network
```

### Solution 1: Start MongoDB

```bash
# Start MongoDB
docker compose up -d mongodb

# Wait for MongoDB to be ready
sleep 10

# Verify MongoDB is healthy
docker compose ps mongodb

# Check MongoDB logs
docker compose logs mongodb | tail -20
```

### Solution 2: Fix Database URL

```bash
# Check DATABASE_URL in .env
cat .env | grep DATABASE_URL

# Correct format:
# DATABASE_URL=mongodb://admin:password123@mongodb:27017/bot_core?authSource=admin

# Update .env
nano .env

# Restart services
docker compose restart rust-core-engine python-ai-service
```

### Solution 3: Reset MongoDB

```bash
# Stop all services
docker compose down

# Remove MongoDB data (CAREFUL: deletes all data)
docker volume rm bot-core_mongodb_data

# Start MongoDB fresh
docker compose up -d mongodb
sleep 10

# Initialize database
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "
    use bot_core;
    db.createCollection('users');
    db.createCollection('orders');
  "

# Start application services
docker compose up -d
```

---

## Test Failures

### Symptom: Tests Failing Locally

**Diagnosis:**
```bash
# Run tests with verbose output
cd rust-core-engine && cargo test -- --nocapture
cd python-ai-service && pytest -v
cd nextjs-ui-dashboard && npm run test -- --reporter=verbose

# Check for environment issues
printenv | grep -E "(DATABASE_URL|BINANCE|JWT)"

# Check for port conflicts
lsof -i :8080 -i :8000 -i :3000
```

### Solution 1: Rust Test Failures

```bash
# Clean and rebuild
cd rust-core-engine
cargo clean
cargo build

# Run specific failing test
cargo test test_name -- --nocapture

# Check for compilation warnings
cargo clippy -- -D warnings

# Run with backtrace
RUST_BACKTRACE=1 cargo test test_name
```

### Solution 2: Python Test Failures

```bash
# Recreate virtual environment
cd python-ai-service
rm -rf venv
python3 -m venv venv
source venv/bin/activate

# Reinstall dependencies
pip install -r requirements.txt
pip install -r requirements.dev.txt

# Run specific test
pytest tests/test_file.py::test_function_name -v

# Run with debugger
pytest --pdb tests/test_file.py
```

### Solution 3: Frontend Test Failures

```bash
# Clear cache and reinstall
cd nextjs-ui-dashboard
rm -rf node_modules .next coverage
npm install

# Run tests in watch mode
npm run test:watch

# Run specific test
npm run test -- src/components/Component.test.tsx

# Update snapshots (if needed)
npm run test -- -u
```

### Solution 4: Integration Test Failures

```bash
# Ensure all services are running
./scripts/bot.sh status

# Check service health
curl http://localhost:8080/health
curl http://localhost:8000/health
curl http://localhost:3000/

# Reset test database
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "db.getSiblingDB('bot_core_test').dropDatabase()"

# Run integration tests
make test-integration
```

---

## Build Failures

### Symptom: Rust Build Fails

**Error Messages:**
```
error: could not compile `rust-core-engine` due to previous error
error: linking with `cc` failed: exit status: 1
```

**Solution:**
```bash
# Update Rust toolchain
rustup update stable

# Clean build artifacts
cd rust-core-engine
cargo clean

# Build with verbose output
cargo build --verbose

# Check for missing dependencies (macOS)
xcode-select --install

# Check for missing dependencies (Linux)
sudo apt-get update
sudo apt-get install -y build-essential pkg-config libssl-dev

# Try incremental build
cargo build --release
```

### Symptom: Python Build Fails

**Error Messages:**
```
ERROR: Could not build wheels for package
ModuleNotFoundError: No module named 'module_name'
```

**Solution:**
```bash
# Update pip and setuptools
cd python-ai-service
pip install --upgrade pip setuptools wheel

# Install system dependencies (Ubuntu/Debian)
sudo apt-get install -y python3-dev python3-pip \
  build-essential libssl-dev libffi-dev

# Install system dependencies (macOS)
brew install python@3.11

# Reinstall dependencies
pip install -r requirements.txt --force-reinstall

# Fix specific package issues
pip install --no-cache-dir package_name
```

### Symptom: Frontend Build Fails

**Error Messages:**
```
Module not found: Can't resolve 'module'
Type error: Cannot find module
```

**Solution:**
```bash
# Clear npm cache
cd nextjs-ui-dashboard
npm cache clean --force

# Remove and reinstall
rm -rf node_modules package-lock.json
npm install

# Fix peer dependencies
npm install --legacy-peer-deps

# Build with verbose output
npm run build --verbose

# Check TypeScript errors
npm run type-check
```

---

## Performance Issues

### Symptom: Slow Response Times

**Diagnosis:**
```bash
# Check API response times
time curl http://localhost:8080/api/v1/health

# Check resource usage
docker stats --no-stream

# Check database performance
docker exec -it mongodb mongosh \
  --eval "db.currentOp()" \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin

# Run performance benchmarks
cd rust-core-engine && cargo bench
cd tests/load && k6 run trading_load_test.js
```

### Solution 1: Enable Redis Caching

```bash
# Start Redis
docker compose up -d redis

# Update Python config to use Redis
cd python-ai-service
nano config.yaml

# Set:
# redis:
#   enabled: true
#   url: redis://redis:6379

# Restart Python service
docker compose restart python-ai-service
```

### Solution 2: Optimize Database

```bash
# Create indexes
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "
    use bot_core;
    db.orders.createIndex({symbol: 1, created_at: -1});
    db.trades.createIndex({user_id: 1, created_at: -1});
    db.users.createIndex({email: 1}, {unique: true});
  "

# Check slow queries
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "db.setProfilingLevel(1, {slowms: 100})"
```

### Solution 3: Scale Services

```bash
# Edit docker-compose.yml to add replicas
services:
  rust-core-engine:
    deploy:
      replicas: 2

# Restart with scaling
docker compose up -d --scale rust-core-engine=2
```

---

## Security Scan Issues

### Symptom: Security Scan Failures

**Diagnosis:**
```bash
# Run security scan
./scripts/security-scan.sh

# Check for vulnerabilities
cd rust-core-engine && cargo audit
cd python-ai-service && safety check
cd nextjs-ui-dashboard && npm audit

# Check for secrets
brew install trufflesecurity/trufflehog/trufflehog
trufflehog git file://. --only-verified
```

### Solution 1: Update Dependencies

```bash
# Update Rust dependencies
cd rust-core-engine
cargo update
cargo audit fix

# Update Python dependencies
cd python-ai-service
pip install --upgrade -r requirements.txt
safety check

# Update Frontend dependencies
cd nextjs-ui-dashboard
npm update
npm audit fix
```

### Solution 2: Fix Secret Leaks

```bash
# Remove secrets from .env
nano .env

# Ensure .env is in .gitignore
echo ".env" >> .gitignore

# Remove secrets from git history (if committed)
git filter-branch --force --index-filter \
  "git rm --cached --ignore-unmatch .env" \
  --prune-empty --tag-name-filter cat -- --all

# Regenerate secrets
./scripts/generate-secrets.sh
```

---

## WebSocket Connection Issues

### Symptom: WebSocket Not Connecting

**Error Messages:**
```
WebSocket connection failed
WebSocket closed with code 1006
```

**Diagnosis:**
```bash
# Test WebSocket connection
wscat -c ws://localhost:8080/ws

# Check nginx/proxy configuration
curl -I http://localhost:8080/

# Check browser console for errors
# Open DevTools → Network → WS tab
```

### Solution 1: Fix WebSocket Configuration

```bash
# Update Rust config for WebSocket
cd rust-core-engine
nano config.toml

# Ensure WebSocket is enabled:
# [server]
# enable_websocket = true
# websocket_path = "/ws"

# Restart Rust service
docker compose restart rust-core-engine
```

### Solution 2: Fix Proxy Configuration

If using nginx/proxy:
```nginx
# Add WebSocket upgrade headers
location /ws {
    proxy_pass http://rust-core-engine:8080;
    proxy_http_version 1.1;
    proxy_set_header Upgrade $http_upgrade;
    proxy_set_header Connection "upgrade";
    proxy_set_header Host $host;
}
```

---

## Trading Execution Issues

### Symptom: Orders Not Executing

**Diagnosis:**
```bash
# Check trading enabled
cat .env | grep TRADING_ENABLED

# Check Binance testnet
cat .env | grep BINANCE_TESTNET

# Check API keys
cat .env | grep BINANCE_API

# Check order logs
docker compose logs rust-core-engine | grep "order"
```

### Solution 1: Enable Trading (CAREFUL!)

```bash
# IMPORTANT: Only enable for testnet!
nano .env

# Verify testnet is enabled:
# BINANCE_TESTNET=true

# Then enable trading:
# TRADING_ENABLED=true

# Restart services
docker compose restart
```

### Solution 2: Fix Binance API Keys

```bash
# Generate testnet API keys at:
# https://testnet.binance.vision/

# Update .env
nano .env
# BINANCE_API_KEY=your_testnet_key
# BINANCE_API_SECRET=your_testnet_secret
# BINANCE_TESTNET=true

# Restart Rust service
docker compose restart rust-core-engine
```

---

## CI/CD Pipeline Failures

### Symptom: GitHub Actions Failing

**Diagnosis:**
```bash
# Check workflow runs
# Go to: https://github.com/YOUR_ORG/bot-core/actions

# View specific job logs
# Click on failed job → View logs

# Run workflow locally with act
brew install act
act -j rust-build-test
```

### Solution 1: Fix FlyCI Wingman

```bash
# FlyCI analyzes failures automatically
# Check PR comments for FlyCI suggestions

# If FlyCI not installed:
# 1. Go to https://www.flyci.net/
# 2. Install FlyCI GitHub App
# 3. Select repository: YOUR_ORG/bot-core
# 4. Grant permissions
# 5. FlyCI will automatically analyze future failures
```

### Solution 2: Fix Specific Job Failures

**Rust job failure:**
```bash
# Run locally
cd rust-core-engine
cargo fmt --check
cargo clippy -- -D warnings
cargo test
cargo build --release
```

**Python job failure:**
```bash
# Run locally
cd python-ai-service
flake8 .
black --check .
pytest --cov
```

**Frontend job failure:**
```bash
# Run locally
cd nextjs-ui-dashboard
npm run lint
npm run type-check
npm run test
npm run build
```

---

## Diagnostic Commands

### Service Health Checks

```bash
# Check all services
./scripts/bot.sh status

# Check specific service
docker compose ps rust-core-engine
docker compose ps python-ai-service
docker compose ps nextjs-ui-dashboard
docker compose ps mongodb

# Check service health endpoints
curl http://localhost:8080/health
curl http://localhost:8000/health
curl http://localhost:3000/
```

### Log Analysis

```bash
# View all logs
./scripts/bot.sh logs

# View specific service logs
./scripts/bot.sh logs --service rust-core-engine
./scripts/bot.sh logs --service python-ai-service
./scripts/bot.sh logs --service nextjs-ui-dashboard

# Follow logs
docker compose logs -f rust-core-engine

# View last N lines
docker compose logs --tail=100 rust-core-engine

# Search logs
docker compose logs rust-core-engine | grep ERROR
docker compose logs python-ai-service | grep "Exception"
```

### Resource Monitoring

```bash
# Monitor resources in real-time
docker stats

# Check disk usage
docker system df
df -h

# Check memory usage
free -h  # Linux
vm_stat  # macOS

# Check CPU usage
top  # Linux/macOS
htop # Linux (if installed)
```

### Network Diagnostics

```bash
# Check Docker networks
docker network ls
docker network inspect bot-network

# Check port bindings
docker compose ps --format json | jq '.[].Ports'

# Test connectivity between services
docker exec -it rust-core-engine ping mongodb
docker exec -it rust-core-engine ping python-ai-service

# Test external connectivity
docker exec -it rust-core-engine curl https://api.binance.com/api/v3/ping
```

### Database Diagnostics

```bash
# Check MongoDB status
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "rs.status()"

# List databases
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "show dbs"

# List collections
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "use bot_core; show collections"

# Count documents
docker exec -it mongodb mongosh \
  --username admin \
  --password YOUR_PASSWORD \
  --authenticationDatabase admin \
  --eval "use bot_core; db.users.count()"
```

---

## Quick Fixes

### Complete System Reset

```bash
# CAREFUL: This deletes all data!

# 1. Stop all services
./scripts/bot.sh stop

# 2. Clean Docker
docker compose down -v
docker system prune -a --volumes -f

# 3. Clean build artifacts
cd rust-core-engine && cargo clean
cd ../python-ai-service && rm -rf venv __pycache__
cd ../nextjs-ui-dashboard && rm -rf node_modules .next

# 4. Regenerate secrets
./scripts/generate-secrets.sh

# 5. Start fresh
./scripts/bot.sh start --memory-optimized
```

### Reset Individual Service

```bash
# Rust
docker compose restart rust-core-engine

# Python
docker compose restart python-ai-service

# Frontend
docker compose restart nextjs-ui-dashboard

# MongoDB
docker compose restart mongodb
```

---

## Getting Help

### Resources

- **Documentation:** `/Users/dungngo97/Documents/bot-core/docs/`
- **Specifications:** `/Users/dungngo97/Documents/bot-core/specs/`
- **Testing Guide:** `/Users/dungngo97/Documents/bot-core/docs/TESTING_GUIDE.md`
- **Contributing:** `/Users/dungngo97/Documents/bot-core/docs/CONTRIBUTING.md`
- **FlyCI Setup:** `/Users/dungngo97/Documents/bot-core/docs/FLYCI_SETUP.md`

### Support Channels

- **GitHub Issues:** Report bugs and issues
- **GitHub Discussions:** Ask questions
- **Pull Requests:** Propose fixes

### Debugging Checklist

Before asking for help:
- [ ] Checked service logs
- [ ] Verified environment variables
- [ ] Ran health checks
- [ ] Checked resource usage
- [ ] Tried clean restart
- [ ] Searched existing issues
- [ ] Reviewed documentation

---

**Last Updated:** 2025-11-14
**Version:** 1.0.0
**Maintainers:** Bot-Core Development Team
