# Deployment Runbook - Bot Core

**Version:** 2.0.0
**Last Updated:** 2025-11-18
**Purpose:** Step-by-step deployment procedures with exact commands

---

## Table of Contents

1. [Overview](#overview)
2. [Pre-Deployment Steps](#pre-deployment-steps)
3. [Production Deployment](#production-deployment)
4. [Rollback Procedures](#rollback-procedures)
5. [Emergency Procedures](#emergency-procedures)
6. [Verification Steps](#verification-steps)
7. [Contact Information](#contact-information)

---

## Overview

This runbook provides exact commands and expected outputs for deploying Bot Core to production. Follow each step sequentially and verify outputs before proceeding.

### Deployment Timeline

| Phase | Duration | Actions |
|-------|----------|---------|
| Pre-checks | 15 min | Verify prerequisites |
| Build | 20 min | Build and push images |
| Deploy | 10 min | Start services |
| Verification | 15 min | Health checks and testing |
| **Total** | **60 min** | Complete deployment |

### Required Access

- SSH access to production servers
- Docker registry credentials
- MongoDB access credentials
- GitHub repository access
- Monitoring dashboard access

---

## Pre-Deployment Steps

### Step 1: Verify Prerequisites (5 minutes)

```bash
# 1.1 Verify you're on the correct server
hostname
# Expected: production-server-01 or your server name

# 1.2 Check Docker version
docker --version
# Expected: Docker version 20.10.0 or higher

# 1.3 Check Docker Compose version
docker-compose --version
# Expected: Docker Compose version 2.0.0 or higher

# 1.4 Verify disk space
df -h
# Expected: At least 50GB available on /

# 1.5 Verify memory
free -h
# Expected: At least 16GB total memory

# 1.6 Check current running containers
docker ps
# Expected: List of currently running containers (if any)
```

**Decision Point:** If any check fails, STOP and resolve issues before continuing.

### Step 2: Backup Current State (5 minutes)

```bash
# 2.1 Create backup directory
BACKUP_DATE=$(date +%Y%m%d-%H%M%S)
mkdir -p /opt/bot-core/backups/$BACKUP_DATE

# 2.2 Backup database
mongodump --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/backups/$BACKUP_DATE/mongodb-backup.tar.gz \
  --gzip

# Expected output:
# 2025-11-18T10:00:00.000+0000  writing bot_core.users to archive
# 2025-11-18T10:00:01.000+0000  writing bot_core.trades to archive
# 2025-11-18T10:00:02.000+0000  done dumping bot_core.users (1234 documents)
# 2025-11-18T10:00:03.000+0000  done dumping bot_core.trades (5678 documents)

# 2.3 Verify backup file exists
ls -lh /opt/bot-core/backups/$BACKUP_DATE/
# Expected: mongodb-backup.tar.gz with size > 0 bytes

# 2.4 Record current version
cd /opt/bot-core
git rev-parse HEAD > /opt/bot-core/backups/$BACKUP_DATE/previous-commit.txt
docker-compose ps > /opt/bot-core/backups/$BACKUP_DATE/previous-services.txt

# 2.5 Create backup marker
echo "Backup created at $BACKUP_DATE" > /opt/bot-core/backups/$BACKUP_DATE/BACKUP_INFO.txt
echo "Previous commit: $(cat /opt/bot-core/backups/$BACKUP_DATE/previous-commit.txt)" >> /opt/bot-core/backups/$BACKUP_DATE/BACKUP_INFO.txt
```

**Decision Point:** Verify backup completed successfully before proceeding.

### Step 3: Stop Current Services (2 minutes)

```bash
# 3.1 Navigate to application directory
cd /opt/bot-core

# 3.2 View current service status
docker-compose ps
# Expected: List of running services with "Up" status

# 3.3 Stop all services gracefully
docker-compose down

# Expected output:
# Stopping nextjs-ui-dashboard ... done
# Stopping rust-core-engine    ... done
# Stopping python-ai-service   ... done
# Removing nextjs-ui-dashboard ... done
# Removing rust-core-engine    ... done
# Removing python-ai-service   ... done
# Removing network bot-network

# 3.4 Verify all containers stopped
docker ps -a | grep bot-core
# Expected: No running containers

# 3.5 Record stop time
echo "Services stopped at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

**Timing:** This step should take less than 2 minutes.

### Step 4: Update Codebase (3 minutes)

```bash
# 4.1 Fetch latest changes
git fetch origin

# Expected output:
# From github.com:your-org/bot-core
#  * branch            main       -> FETCH_HEAD

# 4.2 Check which version to deploy
git log --oneline -5
# Expected: List of recent commits

# 4.3 Checkout target version (replace v2.0.0 with actual version)
export DEPLOY_VERSION="v2.0.0"
git checkout $DEPLOY_VERSION

# Expected output:
# Note: switching to 'v2.0.0'
# HEAD is now at abc1234 Release v2.0.0

# 4.4 Verify checkout
git rev-parse HEAD
# Expected: Commit hash of target version

# 4.5 Record deployed version
echo "Deploying version: $DEPLOY_VERSION" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
echo "Commit hash: $(git rev-parse HEAD)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

**Decision Point:** Confirm you're deploying the correct version.

---

## Production Deployment

### Step 5: Build Docker Images (20 minutes)

```bash
# 5.1 Set build environment variables
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1

# 5.2 Build all images (this will take 15-20 minutes)
echo "Starting build at $(date)"
time docker-compose build --no-cache

# Expected output (abbreviated):
# Building rust-core-engine
# Step 1/15 : FROM rust:1.86 as builder
# ...
# Successfully built abc123def456
# Successfully tagged bot-core/rust-core-engine:latest
#
# Building python-ai-service
# Step 1/12 : FROM python:3.11-slim
# ...
# Successfully built def456abc789
# Successfully tagged bot-core/python-ai-service:latest
#
# Building nextjs-ui-dashboard
# Step 1/10 : FROM node:18-alpine as builder
# ...
# Successfully built 789abc123def
# Successfully tagged bot-core/nextjs-ui-dashboard:latest
#
# real    18m32.456s
# user    2m15.123s
# sys     0m45.678s

# 5.3 Verify images built
docker images | grep bot-core

# Expected output:
# bot-core/nextjs-ui-dashboard  latest  789abc123def  2 minutes ago  150MB
# bot-core/python-ai-service    latest  def456abc789  5 minutes ago  1.2GB
# bot-core/rust-core-engine     latest  abc123def456  10 minutes ago 85MB

# 5.4 Tag images with version
docker tag bot-core/rust-core-engine:latest bot-core/rust-core-engine:$DEPLOY_VERSION
docker tag bot-core/python-ai-service:latest bot-core/python-ai-service:$DEPLOY_VERSION
docker tag bot-core/nextjs-ui-dashboard:latest bot-core/nextjs-ui-dashboard:$DEPLOY_VERSION

# 5.5 Record build completion
echo "Build completed at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

**Timing:** Build should complete in 15-20 minutes depending on server resources.

### Step 6: Validate Environment Configuration (3 minutes)

```bash
# 6.1 Verify .env file exists
ls -la /opt/bot-core/.env
# Expected: .env file with recent modification date

# 6.2 Check critical environment variables
source /opt/bot-core/.env

# 6.3 Verify required variables are set
echo "Checking environment variables..."
for var in DATABASE_URL BINANCE_API_KEY BINANCE_SECRET_KEY OPENAI_API_KEY JWT_SECRET; do
  if [ -z "${!var}" ]; then
    echo "ERROR: $var is not set!"
    exit 1
  else
    echo "✓ $var is set"
  fi
done

# Expected output:
# Checking environment variables...
# ✓ DATABASE_URL is set
# ✓ BINANCE_API_KEY is set
# ✓ BINANCE_SECRET_KEY is set
# ✓ OPENAI_API_KEY is set
# ✓ JWT_SECRET is set

# 6.4 Validate secrets are not defaults
if [[ "$JWT_SECRET" == "default"* ]]; then
  echo "ERROR: JWT_SECRET is using default value!"
  exit 1
fi

# 6.5 Run environment validation script
./scripts/validate-env.sh

# Expected output:
# ✓ All required environment variables are set
# ✓ No default values detected
# ✓ Secret lengths are adequate
# ✓ API keys format is valid
# Environment validation: PASSED
```

**Decision Point:** All validations must pass before proceeding.

### Step 7: Start Services (5 minutes)

```bash
# 7.1 Start services with production profile
echo "Starting services at $(date)"
docker-compose --profile prod up -d

# Expected output:
# Creating network "bot-network" with driver "bridge"
# Creating python-ai-service ... done
# Creating rust-core-engine  ... done
# Creating nextjs-ui-dashboard ... done

# 7.2 Monitor startup logs
docker-compose logs -f &
LOGS_PID=$!

# Wait 30 seconds for initial startup
sleep 30

# 7.3 Check container status
docker-compose ps

# Expected output:
# Name                  Command               State           Ports
# ---------------------------------------------------------------------------
# python-ai-service    python main.py               Up      0.0.0.0:8000->8000/tcp
# rust-core-engine     /app/bot-core                Up      0.0.0.0:8080->8080/tcp
# nextjs-ui-dashboard  node server.js               Up      0.0.0.0:3000->3000/tcp

# 7.4 Stop following logs
kill $LOGS_PID

# 7.5 Record startup time
echo "Services started at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

**Timing:** Services should start within 1 minute, but health checks may take 2-3 minutes.

### Step 8: Wait for Health Checks (2 minutes)

```bash
# 8.1 Wait for services to become healthy
echo "Waiting for health checks..."

# Function to check health
check_health() {
  local service=$1
  local url=$2
  local max_attempts=30
  local attempt=0

  while [ $attempt -lt $max_attempts ]; do
    if curl -f -s $url > /dev/null; then
      echo "✓ $service is healthy"
      return 0
    fi
    attempt=$((attempt + 1))
    sleep 5
    echo "  Waiting for $service... (attempt $attempt/$max_attempts)"
  done

  echo "✗ $service failed health check"
  return 1
}

# 8.2 Check Python AI Service
check_health "Python AI Service" "http://localhost:8000/health"

# Expected output:
# Waiting for Python AI Service... (attempt 1/30)
# Waiting for Python AI Service... (attempt 2/30)
# ✓ Python AI Service is healthy

# 8.3 Check Rust Core Engine
check_health "Rust Core Engine" "http://localhost:8080/api/health"

# Expected output:
# Waiting for Rust Core Engine... (attempt 1/30)
# ✓ Rust Core Engine is healthy

# 8.4 Check Frontend Dashboard
check_health "Frontend Dashboard" "http://localhost:3000/"

# Expected output:
# Waiting for Frontend Dashboard... (attempt 1/30)
# ✓ Frontend Dashboard is healthy

# 8.5 All checks passed
echo "All health checks passed at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

**Decision Point:** If any health check fails after 2 minutes, proceed to rollback.

---

## Verification Steps

### Step 9: Functional Testing (10 minutes)

```bash
# 9.1 Test Rust API health endpoint
curl -s http://localhost:8080/api/health | jq

# Expected output:
# {
#   "status": "healthy",
#   "version": "2.0.0",
#   "timestamp": "2025-11-18T10:30:00Z",
#   "services": {
#     "database": "connected",
#     "binance": "connected",
#     "python_ai": "connected"
#   }
# }

# 9.2 Test Python AI health endpoint
curl -s http://localhost:8000/health | jq

# Expected output:
# {
#   "status": "healthy",
#   "version": "2.0.0",
#   "openai": "connected",
#   "models_loaded": 3
# }

# 9.3 Test authentication
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}' | jq

# Expected output (may vary):
# {
#   "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
#   "expires_in": 86400
# }

# 9.4 Test Binance connectivity
curl -s http://localhost:8080/api/v1/binance/ping | jq

# Expected output:
# {
#   "status": "ok",
#   "timestamp": 1700307000000
# }

# 9.5 Test WebSocket connection
npm install -g wscat  # Install if not already installed
wscat -c ws://localhost:8080/ws

# Expected: Connection established
# Type: {"type":"ping"}
# Expected response: {"type":"pong","timestamp":...}
# Ctrl+C to exit

# 9.6 Test frontend access
curl -s http://localhost:3000/ | grep -o '<title>.*</title>'

# Expected output:
# <title>Bot Core - Trading Dashboard</title>

# 9.7 Record test results
echo "Functional tests passed at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

**Decision Point:** All tests should pass. If any fail, investigate or rollback.

### Step 10: Performance Verification (5 minutes)

```bash
# 10.1 Install Apache Bench if needed
sudo apt install apache2-utils -y

# 10.2 Test API response time
ab -n 100 -c 10 http://localhost:8080/api/health

# Expected output (key metrics):
# Concurrency Level:      10
# Time taken for tests:   0.500 seconds
# Complete requests:      100
# Failed requests:        0
# Requests per second:    200.00 [#/sec] (mean)
# Time per request:       50.000 [ms] (mean)
# Percentage of the requests served within a certain time (ms)
#   50%     25
#   95%     45
#   99%     60

# 10.3 Check resource usage
docker stats --no-stream

# Expected output:
# CONTAINER             CPU %   MEM USAGE / LIMIT     MEM %   NET I/O
# rust-core-engine      5.2%    350MiB / 4GiB        8.5%    1.2kB / 850B
# python-ai-service     8.1%    1.5GiB / 4GiB        37.5%   850B / 1.2kB
# nextjs-ui-dashboard   2.3%    180MiB / 2GiB        9.0%    1.5kB / 2.1kB

# 10.4 Verify no memory leaks (check after 5 minutes)
sleep 300
docker stats --no-stream

# Expected: Memory usage should be stable, not continuously increasing

# 10.5 Record performance metrics
echo "Performance verified at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

### Step 11: Security Verification (3 minutes)

```bash
# 11.1 Verify SSL/TLS (if using SSL)
curl -I https://your-domain.com | grep -i 'HTTP\|strict-transport'

# Expected output:
# HTTP/2 200
# strict-transport-security: max-age=31536000

# 11.2 Check security headers
curl -I http://localhost:3000 | grep -i 'x-frame\|x-content\|x-xss'

# Expected output:
# x-frame-options: SAMEORIGIN
# x-content-type-options: nosniff
# x-xss-protection: 1; mode=block

# 11.3 Verify no secrets in logs
docker-compose logs | grep -i 'api.*key\|secret\|password' | head -5

# Expected: No actual secrets visible (only masked/redacted)

# 11.4 Test rate limiting
for i in {1..150}; do curl -s http://localhost:8080/api/health; done

# Expected: After ~100 requests, should receive 429 Too Many Requests

# 11.5 Record security checks
echo "Security verified at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

---

## Rollback Procedures

### Emergency Rollback (5 minutes)

**Use when:** Critical issues detected, service unavailable, data corruption

```bash
# ROLLBACK STEP 1: Stop current deployment
echo "ROLLBACK INITIATED at $(date)" | tee -a /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
docker-compose down

# Expected: All containers stopped

# ROLLBACK STEP 2: Restore previous version
PREVIOUS_COMMIT=$(cat /opt/bot-core/backups/$BACKUP_DATE/previous-commit.txt)
git checkout $PREVIOUS_COMMIT

# Expected output:
# Previous HEAD position was abc1234
# HEAD is now at def5678

# ROLLBACK STEP 3: Rebuild images (if needed) or use previous images
# Option A: Use existing images from registry
docker-compose pull

# Option B: Rebuild from previous commit
# docker-compose build

# ROLLBACK STEP 4: Restore database (ONLY if data corruption detected)
# WARNING: This will overwrite current data
mongorestore --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/backups/$BACKUP_DATE/mongodb-backup.tar.gz \
  --gzip \
  --drop

# Expected output:
# preparing collections to restore from
# reading metadata for bot_core.users from archive
# restoring bot_core.users from archive
# finished restoring bot_core.users (1234 documents, 0 failures)

# ROLLBACK STEP 5: Start services
docker-compose --profile prod up -d

# ROLLBACK STEP 6: Verify services
sleep 60
./scripts/bot.sh status

# ROLLBACK STEP 7: Document rollback
echo "ROLLBACK COMPLETED at $(date)" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
echo "Reason: <FILL IN REASON>" >> /opt/bot-core/backups/$BACKUP_DATE/deployment-log.txt
```

**Post-Rollback:** Schedule post-mortem meeting to analyze failure.

---

## Emergency Procedures

### Service Crashed

```bash
# 1. Identify crashed service
docker-compose ps

# 2. Check logs
docker-compose logs --tail=100 <service-name>

# 3. Restart specific service
docker-compose restart <service-name>

# 4. If restart fails, recreate
docker-compose up -d --force-recreate <service-name>
```

### Database Connection Lost

```bash
# 1. Test database connectivity
mongosh "$DATABASE_URL" --eval "db.adminCommand('ping')"

# 2. Check service logs
docker-compose logs rust-core-engine | grep -i mongo

# 3. Restart affected services
docker-compose restart rust-core-engine python-ai-service
```

### Out of Memory

```bash
# 1. Check memory usage
docker stats
free -h

# 2. Increase memory limits in .env
# Edit: RUST_MEMORY_LIMIT=8G
# Edit: PYTHON_MEMORY_LIMIT=8G

# 3. Restart with new limits
docker-compose down
docker-compose up -d
```

### Disk Space Full

```bash
# 1. Check disk usage
df -h
du -sh /opt/bot-core/*

# 2. Clean Docker resources
docker system prune -a --volumes

# 3. Clean old logs
find /opt/bot-core/*/logs -name "*.log" -mtime +7 -delete

# 4. Archive old backups
tar -czf old-backups.tar.gz /opt/bot-core/backups/*
```

---

## Contact Information

### On-Call Rotation

| Role | Primary | Backup |
|------|---------|--------|
| DevOps Engineer | +1-555-0101 | +1-555-0102 |
| Backend Engineer | +1-555-0103 | +1-555-0104 |
| Database Admin | +1-555-0105 | +1-555-0106 |
| Security Engineer | +1-555-0107 | +1-555-0108 |

### Escalation Path

1. On-Call Engineer (immediate response)
2. Tech Lead (15 minutes)
3. Engineering Manager (30 minutes)
4. CTO (critical issues only)

### External Vendors

- **Cloud Provider Support:** 1-800-XXX-XXXX
- **MongoDB Atlas Support:** support.mongodb.com
- **Binance API Support:** support@binance.com

---

## Post-Deployment Checklist

- [ ] All services healthy and running
- [ ] Health checks passing
- [ ] Functional tests passed
- [ ] Performance metrics acceptable
- [ ] Security checks passed
- [ ] Monitoring dashboards updated
- [ ] Logs being collected
- [ ] Alerts configured
- [ ] Backup verified
- [ ] Deployment documented
- [ ] Team notified
- [ ] Customer communication sent (if applicable)

---

## Deployment Log Template

```
=== DEPLOYMENT LOG ===
Date: YYYY-MM-DD
Time: HH:MM:SS
Operator: <YOUR NAME>
Version: v2.0.0
Commit: abc123def456

Pre-Deployment:
- Backup completed: YES/NO
- Previous version: v1.9.0
- Database backup size: XXX MB

Deployment:
- Build start: HH:MM:SS
- Build end: HH:MM:SS
- Deploy start: HH:MM:SS
- Services healthy: HH:MM:SS

Verification:
- Health checks: PASS/FAIL
- Functional tests: PASS/FAIL
- Performance tests: PASS/FAIL
- Security tests: PASS/FAIL

Issues:
- Issue 1: <DESCRIPTION>
- Resolution: <RESOLUTION>

Status: SUCCESS/ROLLED_BACK/PARTIAL

Notes:
- <Any additional notes>
```

---

**End of Deployment Runbook**

**Version Control:**
- Version 2.0.0
- Last Updated: 2025-11-18
- Next Review: 2025-12-18
