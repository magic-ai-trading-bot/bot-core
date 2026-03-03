# Operations Manual

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** Operations Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Daily Operations](#2-daily-operations)
- [3. Service Management](#3-service-management)
- [4. Database Operations](#4-database-operations)
- [5. Monitoring Operations](#5-monitoring-operations)
- [6. Security Operations](#6-security-operations)
- [7. Runbooks](#7-runbooks)
- [8. On-Call Procedures](#8-on-call-procedures)

---

## 1. Overview

### 1.1 Purpose

This manual provides operational procedures, runbooks, and best practices for managing the Bot Core trading platform.

### 1.2 Related Documents

- `TROUBLESHOOTING.md` - Issue resolution guide
- `DR-PLAN.md` - Disaster recovery procedures
- `INFRA-REQUIREMENTS.md` - Infrastructure specifications
- `scripts/bot.sh` - Main control script
- `Makefile` - Development and deployment commands

### 1.3 Contact Information

| Role | Contact | Availability |
|------|---------|--------------|
| Operations Team | ops@botcore.app | 24/7 |
| Dev Team Lead | dev-lead@botcore.app | Business hours |
| Security Team | security@botcore.app | 24/7 |
| On-Call Engineer | See PagerDuty rotation | 24/7 |

---

## 2. Daily Operations

### 2.1 Daily Checklist

**Morning Routine (9:00 AM):**

```bash
# 1. Check service status
./scripts/bot.sh status

# 2. Review overnight alerts
# - Check Slack #bot-core-alerts
# - Review PagerDuty incidents
# - Check Grafana dashboards

# 3. Check resource usage
docker stats --no-stream

# 4. Review logs for errors
./scripts/bot.sh logs --service rust-core-engine | grep ERROR
./scripts/bot.sh logs --service python-ai-service | grep ERROR

# 5. Verify backups completed
aws s3 ls s3://bot-core-backups/daily/ | tail -1

# 6. Check trading performance
# - Review Grafana trading dashboard
# - Verify trade execution rate
# - Check portfolio values
```

**Midday Routine (2:00 PM):**

```bash
# 1. Check system health
make health

# 2. Review performance metrics
# - Check API latency (should be < 500ms p95)
# - Verify WebSocket connections stable
# - Review AI analysis times

# 3. Check for security alerts
# - Review security logs
# - Check for failed login attempts
# - Verify no unauthorized access

# 4. Verify database health
# MongoDB connection pool
# Query performance
# Replication lag (should be < 1s)
```

**Evening Routine (6:00 PM):**

```bash
# 1. Review day's trading activity
# - Total trades executed
# - Success rate (target: >95%)
# - P&L summary

# 2. Check for pending deployments
git fetch origin
git log --oneline origin/main ^HEAD

# 3. Verify monitoring is working
# - Prometheus targets up
# - Grafana dashboards loading
# - Alerts firing correctly

# 4. Plan for next day
# - Review scheduled maintenance
# - Check for upcoming releases
# - Coordinate with team
```

### 2.2 Weekly Tasks

**Monday:**
- Review and merge Dependabot PRs
- Plan week's deployments
- Review capacity planning

**Wednesday:**
- Mid-week health check
- Review and update documentation
- Security scan results review

**Friday:**
- Weekly metrics report
- Backup verification test
- Update on-call rotation

**Sunday:**
- Performance testing on staging
- Database maintenance window
- Log rotation and cleanup

---

## 3. Service Management

### 3.1 Starting Services

**Development Mode:**
```bash
# Start all services with hot reload
./scripts/bot.sh dev

# Or use Makefile
make dev

# Start specific service
make dev-rust
make dev-python
make dev-frontend
```

**Production Mode:**
```bash
# Start core services only
./scripts/bot.sh start

# Start with memory optimization (recommended)
./scripts/bot.sh start --memory-optimized

# Start with all enterprise features
./scripts/bot.sh start --with-enterprise

# Start with specific features
./scripts/bot.sh start --with-redis --with-monitoring
```

### 3.2 Stopping Services

```bash
# Stop all services gracefully
./scripts/bot.sh stop

# Or use docker compose
docker compose down

# Force stop (use with caution)
docker compose down --timeout 5
```

### 3.3 Restarting Services

**Restart All Services:**
```bash
./scripts/bot.sh restart
```

**Restart Individual Service:**
```bash
# Using docker compose
docker compose restart rust-core-engine
docker compose restart python-ai-service
docker compose restart nextjs-ui-dashboard

# Or using Makefile
make restart
```

### 3.4 Service Status

**Check Status:**
```bash
# Overall status
./scripts/bot.sh status

# Detailed status with resources
docker compose ps
docker stats --no-stream

# Check health endpoints
curl -f http://localhost:3000/health       # Frontend
curl -f http://localhost:8080/api/health   # Rust
curl -f http://localhost:8000/health       # Python
```

**Service URLs:**
```bash
# Show all service URLs
make urls

# Manual check
echo "Frontend: http://localhost:3000"
echo "Rust API: http://localhost:8080"
echo "Python AI: http://localhost:8000"
echo "Prometheus: http://localhost:9090"
echo "Grafana: http://localhost:3001"
```

### 3.5 Viewing Logs

**All Services:**
```bash
./scripts/bot.sh logs
# Or
make logs
```

**Specific Service:**
```bash
./scripts/bot.sh logs --service rust-core-engine
./scripts/bot.sh logs --service python-ai-service
./scripts/bot.sh logs --service nextjs-ui-dashboard

# Or using Makefile
make logs-rust
make logs-python
make logs-frontend
```

**Follow Logs (Real-time):**
```bash
docker compose logs -f rust-core-engine
docker compose logs -f --tail=100 python-ai-service
```

**Export Logs:**
```bash
# Export to file
docker compose logs rust-core-engine > rust-logs-$(date +%Y%m%d).log

# Search logs
docker compose logs rust-core-engine | grep "ERROR"
docker compose logs python-ai-service | grep "trade"
```

### 3.6 Scaling Services

**Docker Compose:**
```bash
# Scale up (not supported in basic compose)
# Use Kubernetes or Docker Swarm for scaling
```

**Kubernetes:**
```bash
# Scale deployment
kubectl scale deployment rust-core-engine --replicas=5 -n bot-core-production

# Check scaling status
kubectl get pods -n bot-core-production -l app=rust-core-engine

# Auto-scaling is handled by HPA
kubectl get hpa -n bot-core-production
```

---

## 4. Database Operations

### 4.1 Database Backups

**Manual Backup:**
```bash
# Using Makefile
make db-backup

# Manual mongodump
docker exec mongodb-primary mongodump \
  --uri="mongodb://botuser:PASSWORD@localhost:27017/trading_bot?authSource=admin" \
  --out=/backup/dump_$(date +%Y%m%d_%H%M%S)

# Copy backup to host
docker cp mongodb-primary:/backup/ ./backups/
```

**Automated Backups:**
- Daily full backup at 2:00 AM UTC
- Incremental backups every 6 hours
- Backups stored in S3 with 30-day retention

**Verify Backup:**
```bash
# Check backup exists
aws s3 ls s3://bot-core-backups/daily/ | tail -1

# Test restore (on staging)
make db-restore BACKUP_DIR=/backup/dump_20251011_020000
```

### 4.2 Database Restore

**From Latest Backup:**
```bash
# Get latest backup
LATEST_BACKUP=$(aws s3 ls s3://bot-core-backups/daily/ | tail -1 | awk '{print $4}')

# Download backup
aws s3 cp s3://bot-core-backups/daily/$LATEST_BACKUP ./backup.tar.gz
tar -xzf backup.tar.gz

# Restore
docker exec mongodb-primary mongorestore \
  --uri="mongodb://botuser:PASSWORD@localhost:27017/trading_bot?authSource=admin" \
  --dir=/backup/dump_20251011_020000
```

### 4.3 Database Maintenance

**Index Management:**
```bash
# Connect to MongoDB
docker exec -it mongodb-primary mongosh -u botuser -p PASSWORD --authenticationDatabase admin

# List indexes
use trading_bot
db.trades.getIndexes()

# Create index
db.trades.createIndex({ user_id: 1, timestamp: -1 })

# Analyze index usage
db.trades.aggregate([{ $indexStats: {} }])
```

**Optimize Performance:**
```bash
# Compact collection
db.runCommand({ compact: 'trades' })

# Rebuild indexes
db.trades.reIndex()

# Check collection stats
db.trades.stats()
```

**Monitor Connections:**
```bash
# Current connections
db.serverStatus().connections

# Active operations
db.currentOp()

# Kill long-running operation
db.killOp(opid)
```

### 4.4 Database Migrations

**Run Migration:**
```bash
# Validate migration first
python scripts/migrate.py \
  --uri "$DATABASE_URL" \
  --direction up \
  --dry-run

# Run migration
python scripts/migrate.py \
  --uri "$DATABASE_URL" \
  --direction up

# Rollback if needed
python scripts/migrate.py \
  --uri "$DATABASE_URL" \
  --direction down
```

---

## 5. Monitoring Operations

### 5.1 Check Dashboards

**Grafana Dashboards:**
```
http://localhost:3001

Dashboards:
1. System Overview - CPU, memory, disk, network
2. Service Health - API latency, error rates, uptime
3. Trading Performance - Trades, success rate, P&L
4. AI Analysis - Analysis time, confidence, predictions
5. Database Performance - Query time, connections
```

**Prometheus:**
```
http://localhost:9090

Useful queries:
- up{job="rust-core-engine"}
- rate(http_requests_total[5m])
- histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))
```

### 5.2 Check Alerts

**Active Alerts:**
```bash
# Check Prometheus alerts
curl http://localhost:9090/api/v1/alerts

# Check Alertmanager
curl http://localhost:9093/api/v2/alerts

# View in Grafana
# Navigate to Alerting > Alert rules
```

**Alert History:**
```bash
# Check Slack #bot-core-alerts channel
# Check PagerDuty incidents
# Review Alertmanager logs
docker compose logs alertmanager | grep "alert"
```

---

## 6. Security Operations

### 6.1 Secret Rotation

**Generate New Secrets:**
```bash
# Generate all secrets
make generate-secrets

# Manual generation
openssl rand -hex 32  # For tokens (64 chars)
openssl rand -hex 16  # For passwords (32 chars)
```

**Update Secrets:**
```bash
# Update .env file
vi .env

# For Kubernetes
kubectl create secret generic bot-core-secrets \
  --from-literal=inter-service-token='NEW_TOKEN' \
  --from-literal=rust-api-key='NEW_KEY' \
  --dry-run=client -o yaml | kubectl apply -f -

# Restart services to pick up new secrets
kubectl rollout restart deployment/rust-core-engine -n bot-core-production
```

**Secret Rotation Schedule:**
- Inter-service tokens: Every 90 days
- API keys: Every 180 days
- Database passwords: Every 90 days
- SSL certificates: Auto-renewed by cert-manager

### 6.2 Security Scanning

**Run Security Scans:**
```bash
# Trivy scan
docker run --rm -v /var/run/docker.sock:/var/run/docker.sock \
  aquasec/trivy image rust-core-engine:latest

# Dependency audit
cd rust-core-engine && cargo audit
cd python-ai-service && pip-audit
cd nextjs-ui-dashboard && npm audit

# Fix vulnerabilities
npm audit fix
pip install --upgrade package-name
cargo update
```

### 6.3 Access Management

**Add New User:**
```bash
# Create user in authentication system
# Grant appropriate roles
# Add to Kubernetes RBAC (if applicable)

kubectl create rolebinding user-access \
  --clusterrole=view \
  --user=newuser@botcore.app \
  --namespace=bot-core-production
```

**Revoke Access:**
```bash
# Remove user from authentication system
# Revoke Kubernetes access
kubectl delete rolebinding user-access -n bot-core-production

# Rotate any shared secrets
```

### 6.4 Audit Logs

**Review Audit Logs:**
```bash
# API access logs
docker compose logs rust-core-engine | grep "401\|403"

# Database access logs
# Check MongoDB audit log

# Kubernetes audit logs
kubectl logs -n kube-system kube-apiserver-* --tail=100 | grep audit
```

---

## 7. Runbooks

### 7.1 Add New Trading Symbol

**Steps:**
1. Update configuration
2. Restart services
3. Verify data collection
4. Monitor performance

**Commands:**
```bash
# 1. Update config
vi rust-core-engine/config.toml
# Add symbol to market_data.symbols array
# e.g., symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT", "ADAUSDT"]

# 2. Restart Rust service
docker compose restart rust-core-engine

# 3. Verify
curl http://localhost:8080/api/market-data/ADAUSDT

# 4. Monitor logs
docker compose logs -f rust-core-engine | grep "ADAUSDT"
```

### 7.2 Update AI Model

**Steps:**
1. Train new model
2. Upload to storage
3. Update configuration
4. Restart AI service
5. Verify accuracy

**Commands:**
```bash
# 1. Train model (on ML workstation)
cd python-ai-service
python train_model.py --model lstm --epochs 100

# 2. Upload model
aws s3 cp models/saved/lstm-v2.4.pkl s3://bot-core-models/production/

# 3. Update config
vi python-ai-service/config.yaml
# Update model version

# 4. Restart service
docker compose restart python-ai-service

# 5. Verify
curl http://localhost:8000/api/model/info
curl -X POST http://localhost:8000/api/analyze \
  -H "Content-Type: application/json" \
  -d '{"symbol": "BTCUSDT", "timeframe": "1h"}'
```

### 7.3 Scale Services

**Kubernetes Scaling:**
```bash
# Check current replicas
kubectl get deployment rust-core-engine -n bot-core-production

# Scale up
kubectl scale deployment rust-core-engine --replicas=5 -n bot-core-production

# Verify scaling
kubectl get pods -n bot-core-production -l app=rust-core-engine

# Check HPA status
kubectl get hpa rust-core-engine-hpa -n bot-core-production

# Adjust HPA if needed
kubectl patch hpa rust-core-engine-hpa -n bot-core-production \
  --patch '{"spec":{"minReplicas":3,"maxReplicas":15}}'
```

### 7.4 Rotate API Keys

**Binance API Keys:**
```bash
# 1. Generate new keys in Binance
# Go to Binance account > API Management > Create API Key

# 2. Update secrets
kubectl create secret generic binance-secrets \
  --from-literal=api-key='NEW_KEY' \
  --from-literal=secret-key='NEW_SECRET' \
  --dry-run=client -o yaml | kubectl apply -f - -n bot-core-production

# Or for Docker Compose
vi .env
# Update BINANCE_API_KEY and BINANCE_SECRET_KEY

# 3. Restart services
kubectl rollout restart deployment/rust-core-engine -n bot-core-production
# Or
docker compose restart rust-core-engine

# 4. Verify
# Check logs for successful API connection
docker compose logs rust-core-engine | grep "Binance API"
```

### 7.5 Enable/Disable Trading

**Enable Trading:**
```bash
# WARNING: Only enable in production after thorough testing

# 1. Update config
vi rust-core-engine/config.toml
# Set trading.enabled = true

# 2. Update environment variable
export TRADING_ENABLED=true

# 3. Restart service
docker compose restart rust-core-engine

# 4. Monitor closely
docker compose logs -f rust-core-engine | grep "trade"
# Watch Grafana trading dashboard
```

**Disable Trading (Emergency):**
```bash
# Quick disable
export TRADING_ENABLED=false
docker compose restart rust-core-engine

# Or via API (if implemented)
curl -X POST http://localhost:8080/api/admin/trading/disable \
  -H "Authorization: Bearer $ADMIN_TOKEN"
```

### 7.6 Maintenance Mode

**Enter Maintenance Mode:**
```bash
# 1. Notify users (if applicable)
# Post maintenance notice

# 2. Stop accepting new requests
# Update load balancer to show maintenance page

# 3. Wait for in-flight requests to complete
sleep 60

# 4. Stop services
./scripts/bot.sh stop

# 5. Perform maintenance
# - Update configurations
# - Run database migrations
# - Apply security patches

# 6. Restart services
./scripts/bot.sh start

# 7. Run smoke tests
curl -f http://localhost:3000/health
curl -f http://localhost:8080/api/health
curl -f http://localhost:8000/health

# 8. Resume normal operations
# Update load balancer to route to services
```

---

## 8. On-Call Procedures

### 8.1 On-Call Rotation

**Schedule:**
- Weekly rotation (Monday 9:00 AM to Monday 9:00 AM)
- Check PagerDuty for current on-call engineer
- Handoff meeting every Monday at 9:00 AM

**On-Call Responsibilities:**
- Respond to PagerDuty alerts within 15 minutes
- Resolve P0/P1 incidents within SLA
- Escalate to team lead if unable to resolve
- Document all incidents in post-mortem

### 8.2 Incident Response

**Severity Levels:**

| Severity | Description | Response Time | Examples |
|----------|-------------|---------------|----------|
| P0 (Critical) | Complete outage | 15 minutes | Service down, data loss |
| P1 (High) | Major degradation | 30 minutes | High error rate, slow response |
| P2 (Medium) | Partial degradation | 2 hours | Single feature broken |
| P3 (Low) | Minor issue | Next business day | UI bug, logging issue |

**Response Steps:**

**P0 - Critical:**
```bash
# 1. Acknowledge alert (PagerDuty)

# 2. Assess impact
./scripts/bot.sh status
make health

# 3. Check recent changes
git log --oneline --since="2 hours ago"
# Check recent deployments

# 4. Rollback if needed (see DR-PLAN.md)
kubectl rollout undo deployment/rust-core-engine -n bot-core-production

# 5. Notify stakeholders
# Post in #incidents Slack channel
# Update status page

# 6. Restore service
# Follow troubleshooting guide (TROUBLESHOOTING.md)

# 7. Document incident
# Create post-mortem in incidents/ directory
```

**P1 - High:**
```bash
# 1. Acknowledge alert

# 2. Gather diagnostics
docker compose logs --tail=500 rust-core-engine > diagnostics.log
kubectl describe pod <pod-name> -n bot-core-production >> diagnostics.log

# 3. Identify root cause
# Check metrics in Grafana
# Review logs in Kibana

# 4. Apply fix
# Deploy hotfix if needed
# Restart affected services

# 5. Monitor recovery
# Watch metrics return to normal
# Verify error rate decreases

# 6. Document resolution
```

### 8.3 Escalation Path

**Level 1: On-Call Engineer**
- Initial response and triage
- Resolve using runbooks
- Escalate if unable to resolve in 30 minutes

**Level 2: Team Lead**
- Complex issues requiring architectural knowledge
- Coordinate with multiple teams
- Make decisions on rollbacks/hotfixes

**Level 3: Engineering Manager**
- Critical incidents requiring executive decision
- Resource allocation
- External communication

**Level 4: CTO**
- Major incidents with business impact
- Regulatory/compliance issues

### 8.4 Post-Incident Review

**Post-Mortem Template:**
```markdown
# Incident Post-Mortem

## Incident Summary
- Date: YYYY-MM-DD
- Time: HH:MM UTC
- Duration: X hours
- Severity: P0/P1/P2/P3
- Services Affected: List

## Timeline
- HH:MM - Incident detected
- HH:MM - On-call acknowledged
- HH:MM - Root cause identified
- HH:MM - Fix applied
- HH:MM - Service restored
- HH:MM - Incident resolved

## Root Cause
Detailed explanation of what caused the incident.

## Impact
- Users affected: X
- Requests failed: X
- Revenue impact: $X

## Resolution
Steps taken to resolve the incident.

## Action Items
1. [ ] Action item 1 - Owner - Due date
2. [ ] Action item 2 - Owner - Due date
3. [ ] Action item 3 - Owner - Due date

## Lessons Learned
What went well:
- Item 1
- Item 2

What could be improved:
- Item 1
- Item 2
```

---

## Appendix A: Quick Reference Commands

### Service Management
```bash
# Start/Stop
./scripts/bot.sh start
./scripts/bot.sh stop
./scripts/bot.sh restart
./scripts/bot.sh status

# Logs
./scripts/bot.sh logs
./scripts/bot.sh logs --service <name>

# Health
make health
curl http://localhost:8080/api/health
```

### Database
```bash
# Backup
make db-backup

# Restore
make db-restore BACKUP_DIR=/path/to/backup

# Connect
docker exec -it mongodb-primary mongosh -u botuser -p PASSWORD
```

### Monitoring
```bash
# Dashboards
open http://localhost:3001  # Grafana
open http://localhost:9090  # Prometheus

# Resource usage
docker stats
kubectl top pods -n bot-core-production
```

### Deployment
```bash
# Build
make build
make build-fast

# Deploy
kubectl apply -k infrastructure/kubernetes/overlays/production/

# Rollback
kubectl rollout undo deployment/<name> -n bot-core-production
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | Operations Team | Initial version |

---

**Document End**
