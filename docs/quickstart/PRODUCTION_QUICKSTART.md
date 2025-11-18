# Production Quick Start - Bot Core

**Time Required:** 30-40 minutes
**Environment:** Production
**WARNING:** Follow [PRODUCTION_DEPLOYMENT_GUIDE.md](../PRODUCTION_DEPLOYMENT_GUIDE.md) for comprehensive instructions

---

## Critical Pre-Deployment Checklist

- [ ] All tests passing (2,411+ tests)
- [ ] Security scan completed (0 HIGH/CRITICAL vulnerabilities)
- [ ] Staging environment tested successfully
- [ ] Database backup completed
- [ ] Rollback plan documented
- [ ] Team notified
- [ ] Monitoring configured

---

## Production Deployment (10 Steps)

### 1. Access Production Server
```bash
# SSH with key-based authentication
ssh -i ~/.ssh/production-key.pem user@production-server

# Verify server identity
hostname
# Expected: production-server-01
```

### 2. Backup Current State
```bash
# Create backup
BACKUP_DATE=$(date +%Y%m%d-%H%M%S)
mkdir -p /opt/bot-core/backups/$BACKUP_DATE

# Backup database
mongodump --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/backups/$BACKUP_DATE/mongodb-backup.tar.gz \
  --gzip

# Verify backup
ls -lh /opt/bot-core/backups/$BACKUP_DATE/
```

### 3. Stop Current Services
```bash
cd /opt/bot-core
docker-compose down

# Verify stopped
docker ps
```

### 4. Update Codebase
```bash
# Fetch latest
git fetch origin

# Checkout production tag
git checkout v2.0.0

# Verify version
git rev-parse HEAD
```

### 5. Validate Configuration
```bash
# Verify .env
./scripts/validate-env.sh

# Check critical variables
source .env
echo $NODE_ENV          # Expected: production
echo $BINANCE_TESTNET   # Expected: false or true (based on your setup)
echo $TRADING_ENABLED   # Expected: false (until verified)
```

### 6. Build Images
```bash
# Build with no cache
make build

# Verify images
docker images | grep bot-core
```

### 7. Deploy Services
```bash
# Start production services
docker-compose --profile prod up -d

# Monitor startup
docker-compose logs -f
```

### 8. Health Verification (CRITICAL)
```bash
# Wait 2-3 minutes for health checks
sleep 180

# Verify health
curl http://localhost:8080/api/health | jq
curl http://localhost:8000/health | jq
curl -I http://localhost:3000/

# Expected: All return healthy status
```

### 9. Functional Testing
```bash
# Test authentication
curl -X POST http://localhost:8080/api/v1/auth/login \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"test123"}'

# Test Binance connection
curl http://localhost:8080/api/v1/binance/ping

# Test WebSocket
wscat -c ws://localhost:8080/ws
```

### 10. Enable Monitoring
```bash
# Start monitoring
docker-compose --profile monitoring up -d

# Access dashboards
# Grafana: http://production-server:3001
# Prometheus: http://production-server:9090
```

---

## Post-Deployment Verification

### Performance Check
```bash
# Load test (light)
ab -n 100 -c 10 http://localhost:8080/api/health

# Expected:
# - Time per request: < 50ms (mean)
# - 95%ile: < 100ms
# - 0 failed requests
```

### Resource Usage
```bash
# Check resource consumption
docker stats --no-stream

# Expected:
# - rust-core-engine: < 10% CPU, < 1GB RAM
# - python-ai-service: < 15% CPU, < 2GB RAM
# - nextjs-ui-dashboard: < 5% CPU, < 500MB RAM
```

### Security Check
```bash
# Verify no secrets in logs
docker-compose logs | grep -i 'api.*key\|secret\|password'
# Expected: No actual secrets visible

# Test rate limiting
for i in {1..150}; do curl -s http://localhost:8080/api/health; done
# Expected: 429 Too Many Requests after ~100 requests
```

---

## Monitoring (First 24 Hours)

### Hour 1
- Monitor every 5 minutes
- Check logs for errors
- Verify resource usage stable
- Test critical user flows

### Hours 2-24
- Monitor every 30 minutes
- Review error rates
- Check performance metrics
- Verify no memory leaks

---

## Rollback Procedure (If Needed)

```bash
# 1. Stop current deployment
docker-compose down

# 2. Restore previous version
git checkout <previous-tag>

# 3. Restore database (if needed)
mongorestore --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/backups/$BACKUP_DATE/mongodb-backup.tar.gz \
  --gzip --drop

# 4. Start previous version
docker-compose up -d

# 5. Verify health
./scripts/bot.sh status
```

---

## Enabling Live Trading (CRITICAL)

**ONLY after 24+ hours of stable operation:**

```bash
# 1. Verify system stability
# - 0 crashes
# - 0 critical errors
# - Performance within targets
# - All monitoring working

# 2. Update .env
nano .env
# Set: TRADING_ENABLED=true
# Set: BINANCE_TESTNET=false (if using production)

# 3. Restart services
docker-compose restart rust-core-engine

# 4. Verify trading engine
docker-compose logs -f rust-core-engine | grep -i trading
# Expected: "Trading engine: ENABLED"

# 5. START WITH SMALL POSITION SIZES
# Configure in rust-core-engine/config.toml:
# max_position_size = 0.001  # Very small initially

# 6. Monitor first trades closely
# Watch for 1-2 hours before increasing limits
```

---

## Emergency Contacts

- **On-Call Engineer:** +1-555-0101
- **Tech Lead:** +1-555-0103
- **DevOps:** +1-555-0105
- **Emergency Hotline:** +1-555-9999

---

## Support Resources

- **Full Guide:** [PRODUCTION_DEPLOYMENT_GUIDE.md](../PRODUCTION_DEPLOYMENT_GUIDE.md)
- **Checklist:** [PRODUCTION_CHECKLIST.md](../PRODUCTION_CHECKLIST.md)
- **Runbook:** [runbooks/DEPLOYMENT_RUNBOOK.md](../runbooks/DEPLOYMENT_RUNBOOK.md)
- **Troubleshooting:** [TROUBLESHOOTING.md](../TROUBLESHOOTING.md)

---

**CRITICAL REMINDERS:**
- ⚠️ Never enable trading without 24+ hours of stable operation
- ⚠️ Always backup before deployment
- ⚠️ Test rollback procedure before deploying
- ⚠️ Monitor closely for first 24 hours
- ⚠️ Start with small position sizes
