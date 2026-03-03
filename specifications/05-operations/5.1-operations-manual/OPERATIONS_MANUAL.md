# Operations Manual - Bot Core

**Version:** 2.0.0
**Last Updated:** 2025-11-18
**Audience:** Operations Team, DevOps, SRE

---

## Table of Contents

1. [Daily Operations](#daily-operations)
2. [Monitoring and Alerting](#monitoring-and-alerting)
3. [Log Management](#log-management)
4. [Backup Procedures](#backup-procedures)
5. [Disaster Recovery](#disaster-recovery)
6. [Scaling Procedures](#scaling-procedures)
7. [Performance Tuning](#performance-tuning)
8. [Security Incident Response](#security-incident-response)
9. [Maintenance Windows](#maintenance-windows)
10. [Troubleshooting Playbooks](#troubleshooting-playbooks)

---

## Daily Operations

### Morning Checklist (Every Day, 9:00 AM)

```bash
#!/bin/bash
# daily-health-check.sh

echo "=== Daily Health Check - $(date) ==="

# 1. Check service health
echo "1. Checking service health..."
docker-compose ps
curl -s http://localhost:8080/api/health | jq '.status'
curl -s http://localhost:8000/health | jq '.status'

# 2. Check resource usage
echo "2. Checking resource usage..."
docker stats --no-stream

# 3. Check disk space
echo "3. Checking disk space..."
df -h | grep -E '/$|/opt'

# 4. Check error logs
echo "4. Checking for errors..."
docker-compose logs --since=24h | grep -i error | tail -20

# 5. Verify backups
echo "5. Verifying latest backup..."
ls -lht /opt/bot-core/backups/ | head -5

# 6. Check external API connectivity
echo "6. Testing external APIs..."
curl -s http://localhost:8080/api/v1/binance/ping
curl -s http://localhost:8000/api/ai/status

echo "=== Health Check Complete ==="
```

**Schedule:** Run daily at 9:00 AM via cron
```bash
0 9 * * * /opt/bot-core/scripts/daily-health-check.sh >> /var/log/bot-core-health.log 2>&1
```

### Weekly Checklist (Every Monday, 10:00 AM)

- [ ] Review system logs for anomalies
- [ ] Check SSL certificate expiration (renew if < 30 days)
- [ ] Review and rotate logs older than 30 days
- [ ] Update system packages
- [ ] Run security vulnerability scan
- [ ] Review performance metrics trends
- [ ] Check database indexes and performance
- [ ] Review and update documentation
- [ ] Verify backup restoration capability
- [ ] Review monitoring alerts and tune thresholds

### Monthly Checklist (First Monday of Month)

- [ ] Full security audit
- [ ] Dependency updates (Rust, Python, Node)
- [ ] Database optimization and reindexing
- [ ] Review and archive old backups
- [ ] Capacity planning review
- [ ] Disaster recovery drill
- [ ] Review and update runbooks
- [ ] Performance benchmarking
- [ ] Cost optimization review
- [ ] Team training on new procedures

---

## Monitoring and Alerting

### Grafana Dashboards

**Access:** http://your-domain:3001

**Key Dashboards:**
1. **Bot Core Overview** - System-wide health
2. **Rust Core Engine** - Trading engine metrics
3. **Python AI Service** - ML model performance
4. **MongoDB Performance** - Database metrics
5. **System Resources** - CPU, Memory, Disk

### Critical Alerts

**Immediate Response (P0):**
- Service down > 2 minutes
- Database connection lost
- Disk usage > 90%
- Memory usage > 95%
- Error rate > 10%
- Trading execution failures

**Urgent Response (P1 - 15 min):**
- API latency > 500ms p95
- Memory usage > 85%
- Disk usage > 80%
- Error rate > 5%
- Backup failure

**Warning (P2 - 1 hour):**
- API latency > 200ms p95
- Memory usage > 70%
- Disk usage > 70%
- Error rate > 1%

### Alert Response Procedures

```bash
# P0 Alert: Service Down
1. Check service status: docker-compose ps
2. Check logs: docker-compose logs --tail=100 <service>
3. Attempt restart: docker-compose restart <service>
4. If restart fails: docker-compose up -d --force-recreate <service>
5. Escalate if not resolved in 5 minutes

# P1 Alert: High Resource Usage
1. Identify cause: docker stats
2. Check for memory leaks: docker logs <service> | grep -i memory
3. Consider scaling: docker-compose up --scale <service>=2
4. Review recent changes in deployment log

# P2 Alert: Performance Degradation
1. Check database slow queries
2. Review recent traffic patterns
3. Check for long-running processes
4. Consider caching improvements
```

---

## Log Management

### Log Locations

```bash
# Application logs
/opt/bot-core/rust-core-engine/logs/*.log
/opt/bot-core/python-ai-service/logs/*.log
/opt/bot-core/nextjs-ui-dashboard/logs/*.log

# Docker logs
docker-compose logs <service>

# System logs
/var/log/syslog
/var/log/auth.log
```

### Log Rotation

**Automatic rotation configured:**
```bash
# /etc/logrotate.d/bot-core
/opt/bot-core/*/logs/*.log {
    daily
    rotate 30
    compress
    delaycompress
    notifempty
    create 0644 bot-core bot-core
    postrotate
        docker-compose restart
    endscript
}
```

### Log Analysis

```bash
# Search for errors in last 24 hours
docker-compose logs --since=24h | grep -i error

# Find slow API requests
grep -r "duration.*ms" /opt/bot-core/rust-core-engine/logs/ | \
  awk -F'duration:' '{print $2}' | sort -n | tail -20

# Find failed trading attempts
grep -i "trade.*failed" /opt/bot-core/rust-core-engine/logs/*.log

# Aggregate error types
docker-compose logs | grep ERROR | \
  awk '{print $NF}' | sort | uniq -c | sort -rn
```

### Centralized Logging (Optional)

If using ELK Stack or similar:
```bash
# Configure Filebeat
sudo vi /etc/filebeat/filebeat.yml

# Add Bot Core logs
filebeat.inputs:
- type: log
  enabled: true
  paths:
    - /opt/bot-core/*/logs/*.log
  fields:
    app: bot-core
```

---

## Backup Procedures

### Automated Daily Backups

**Schedule:** Daily at 2:00 AM UTC

```bash
#!/bin/bash
# /opt/bot-core/scripts/backup.sh

BACKUP_DIR=/opt/bot-core/backups
DATE=$(date +%Y%m%d-%H%M%S)
BACKUP_PATH=$BACKUP_DIR/$DATE

mkdir -p $BACKUP_PATH

# 1. MongoDB backup
echo "Backing up MongoDB..."
mongodump --uri="$DATABASE_URL" \
  --archive=$BACKUP_PATH/mongodb-$DATE.tar.gz \
  --gzip

# 2. Configuration backup
echo "Backing up configuration..."
tar -czf $BACKUP_PATH/config-$DATE.tar.gz \
  /opt/bot-core/.env \
  /opt/bot-core/rust-core-engine/config.toml \
  /opt/bot-core/python-ai-service/config.yaml

# 3. Upload to cloud storage (S3, GCS, etc.)
echo "Uploading to cloud storage..."
aws s3 cp $BACKUP_PATH/ s3://bot-core-backups/$DATE/ --recursive

# 4. Verify backup
echo "Verifying backup..."
ls -lh $BACKUP_PATH/

# 5. Clean old local backups (keep last 7 days)
find $BACKUP_DIR -type d -mtime +7 -exec rm -rf {} +

# 6. Log backup completion
echo "Backup completed: $DATE" >> $BACKUP_DIR/backup.log

# 7. Send notification (optional)
# curl -X POST https://hooks.slack.com/... -d '{"text":"Backup completed: '$DATE'"}'
```

**Cron configuration:**
```bash
0 2 * * * /opt/bot-core/scripts/backup.sh >> /var/log/bot-core-backup.log 2>&1
```

### Backup Verification (Weekly)

```bash
# Test backup restoration
LATEST_BACKUP=$(ls -t /opt/bot-core/backups/ | head -1)

# Restore to test database
mongorestore --uri="mongodb://localhost:27017/bot_core_test" \
  --archive=/opt/bot-core/backups/$LATEST_BACKUP/mongodb-*.tar.gz \
  --gzip

# Verify data
mongosh "mongodb://localhost:27017/bot_core_test" --eval "db.users.count()"

# Clean up test database
mongosh "mongodb://localhost:27017/bot_core_test" --eval "db.dropDatabase()"
```

### Backup Retention Policy

- **Daily backups:** 30 days
- **Weekly backups:** 12 weeks
- **Monthly backups:** 12 months
- **Yearly backups:** 7 years (compliance)

---

## Disaster Recovery

### RTO & RPO Targets

- **Recovery Time Objective (RTO):** < 2 hours
- **Recovery Point Objective (RPO):** < 1 hour

### DR Scenarios

#### Scenario 1: Complete Server Failure

**Recovery Steps:**
```bash
# 1. Provision new server
# 2. Install Docker and dependencies
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# 3. Clone repository
git clone https://github.com/your-org/bot-core.git /opt/bot-core
cd /opt/bot-core

# 4. Restore configuration
aws s3 cp s3://bot-core-backups/latest/config-*.tar.gz .
tar -xzf config-*.tar.gz -C /opt/bot-core

# 5. Restore database
LATEST_BACKUP=$(aws s3 ls s3://bot-core-backups/ | sort | tail -1 | awk '{print $2}')
aws s3 cp s3://bot-core-backups/$LATEST_BACKUP/mongodb-*.tar.gz .
mongorestore --uri="$DATABASE_URL" --archive=mongodb-*.tar.gz --gzip

# 6. Start services
docker-compose up -d

# 7. Verify health
./scripts/bot.sh status
```

**Estimated Time:** 90 minutes

#### Scenario 2: Database Corruption

```bash
# 1. Stop application
docker-compose down

# 2. Backup corrupted database
mongodump --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/corrupted-$(date +%Y%m%d).tar.gz --gzip

# 3. Restore from latest backup
LATEST_BACKUP=$(ls -t /opt/bot-core/backups/ | head -1)
mongorestore --uri="$DATABASE_URL" \
  --archive=/opt/bot-core/backups/$LATEST_BACKUP/mongodb-*.tar.gz \
  --gzip --drop

# 4. Restart application
docker-compose up -d

# 5. Verify data integrity
mongosh "$DATABASE_URL" --eval "db.users.count(); db.trades.count();"
```

**Estimated Time:** 30 minutes

#### Scenario 3: Ransomware Attack

```bash
# 1. IMMEDIATELY isolate infected systems
# 2. DO NOT pay ransom
# 3. Contact security team
# 4. Follow Scenario 1 (Complete Server Failure) on clean infrastructure
# 5. Restore from offline/immutable backups only
# 6. Conduct security audit before resuming operations
```

### DR Drill Schedule

- **Quarterly:** Full DR drill
- **Monthly:** Database restore test
- **Weekly:** Backup verification

---

## Scaling Procedures

### Vertical Scaling (Increase Resources)

```bash
# 1. Update resource limits in .env
RUST_MEMORY_LIMIT=8G
RUST_CPU_LIMIT=4

# 2. Restart services with new limits
docker-compose down
docker-compose up -d

# 3. Verify new limits
docker stats
```

### Horizontal Scaling (Add Instances)

```bash
# Using Docker Compose
docker-compose up --scale rust-core-engine=3 -d

# Using Kubernetes
kubectl scale deployment rust-core-engine --replicas=3
```

### Database Scaling

**Read Replicas:**
```bash
# Add MongoDB read replicas
# Update connection string to use replica set
DATABASE_URL=mongodb://primary:27017,replica1:27017,replica2:27017/bot_core?replicaSet=rs0&readPreference=secondaryPreferred
```

### Load Balancer Configuration

**Nginx example:**
```nginx
upstream rust-backend {
    least_conn;
    server rust-1:8080 max_fails=3 fail_timeout=30s;
    server rust-2:8080 max_fails=3 fail_timeout=30s;
    server rust-3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    location / {
        proxy_pass http://rust-backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

---

## Performance Tuning

### Database Optimization

```bash
# Create indexes for common queries
mongosh "$DATABASE_URL" <<EOF
use bot_core

// Trades collection
db.trades.createIndex({ user_id: 1, timestamp: -1 })
db.trades.createIndex({ symbol: 1, timestamp: -1 })
db.trades.createIndex({ status: 1, timestamp: -1 })

// Market data collection
db.market_data.createIndex({ symbol: 1, timestamp: -1 })
db.market_data.createIndex({ timestamp: -1 }, { expireAfterSeconds: 2592000 })

// Users collection
db.users.createIndex({ email: 1 }, { unique: true })
db.users.createIndex({ created_at: -1 })

// Enable profiling
db.setProfilingLevel(1, { slowms: 100 })
EOF
```

### Application Tuning

**Rust Core Engine (config.toml):**
```toml
[server]
workers = 8  # 2x CPU cores
max_connections = 2000
keep_alive = 60

[database]
pool_size = 50
connection_timeout = 30
idle_timeout = 300

[cache]
enabled = true
ttl = 300
max_size = "1GB"
```

**Python AI Service (config.yaml):**
```yaml
server:
  workers: 4
  worker_class: "uvicorn.workers.UvicornWorker"
  timeout: 60

cache:
  enabled: true
  backend: "redis"
  ttl: 600
```

### Network Optimization

```bash
# Increase TCP connection limits
sudo sysctl -w net.core.somaxconn=4096
sudo sysctl -w net.ipv4.tcp_max_syn_backlog=8192

# Enable TCP BBR congestion control
sudo sysctl -w net.core.default_qdisc=fq
sudo sysctl -w net.ipv4.tcp_congestion_control=bbr
```

---

## Security Incident Response

### Incident Severity Levels

**Critical (P0):** Data breach, unauthorized access
**High (P1):** DDoS attack, service compromise attempt
**Medium (P2):** Suspicious activity, failed login attempts
**Low (P3):** Minor security alerts

### Incident Response Procedure

**Phase 1: Detection & Analysis (0-15 min)**
```bash
# 1. Identify incident type and severity
# 2. Document initial findings
# 3. Isolate affected systems if necessary
docker-compose stop <compromised-service>

# 4. Preserve evidence
docker-compose logs <service> > incident-$(date +%Y%m%d-%H%M%S).log
```

**Phase 2: Containment (15-30 min)**
```bash
# 1. Block malicious IPs
sudo ufw deny from <malicious-ip>

# 2. Rotate compromised credentials
./scripts/generate-secrets.sh
# Update .env with new secrets

# 3. Enable additional logging
LOG_LEVEL=debug
```

**Phase 3: Eradication (30-60 min)**
```bash
# 1. Remove malicious code/files
# 2. Patch vulnerabilities
# 3. Update dependencies
# 4. Run security scan
make security-check
```

**Phase 4: Recovery (1-2 hours)**
```bash
# 1. Restore from clean backup if necessary
# 2. Verify system integrity
# 3. Gradually restore services
# 4. Monitor closely
```

**Phase 5: Post-Incident (Within 24 hours)**
- Document incident timeline
- Conduct root cause analysis
- Update security procedures
- Notify affected parties if required
- Implement preventive measures

### Security Contact Information

- **Security Team:** security@your-org.com
- **On-Call Security:** +1-555-SECURITY
- **External Security Consultant:** +1-555-INFOSEC

---

## Maintenance Windows

### Scheduled Maintenance

**Standard Window:** Every 2nd Saturday, 2:00 AM - 6:00 AM UTC

**Procedure:**
```bash
# 1. Notify users 7 days in advance
# 2. Deploy maintenance page
# 3. Create backup
# 4. Stop services
docker-compose down

# 5. Perform maintenance (updates, migrations, etc.)
# 6. Test in staging first
# 7. Deploy to production
docker-compose up -d

# 8. Verify health
./scripts/bot.sh status

# 9. Remove maintenance page
# 10. Notify users of completion
```

### Emergency Maintenance

**Trigger conditions:**
- Critical security vulnerability
- Data corruption risk
- Complete service outage

**Approval:** Requires CTO or Engineering Manager approval

---

## Troubleshooting Playbooks

### Service Won't Start

```bash
# 1. Check logs
docker-compose logs <service>

# 2. Verify configuration
./scripts/validate-env.sh

# 3. Check dependencies
docker-compose ps

# 4. Verify database connectivity
mongosh "$DATABASE_URL" --eval "db.adminCommand('ping')"

# 5. Check resource availability
free -h
df -h

# 6. Recreate container
docker-compose up -d --force-recreate <service>
```

### High Memory Usage

```bash
# 1. Identify process
docker stats

# 2. Check for memory leaks
docker logs <service> | grep -i memory

# 3. Restart service
docker-compose restart <service>

# 4. If persistent, investigate code
# 5. Consider scaling vertically
```

### Database Performance Issues

```bash
# 1. Check slow queries
mongosh "$DATABASE_URL" --eval "db.system.profile.find().limit(10).sort({ts:-1})"

# 2. Verify indexes
mongosh "$DATABASE_URL" --eval "db.trades.getIndexes()"

# 3. Check connection pool
docker-compose logs rust-core-engine | grep -i "pool\|connection"

# 4. Consider adding read replicas
```

---

## Appendix

### Useful Commands Reference

```bash
# Service management
docker-compose ps                    # List services
docker-compose logs -f <service>     # Follow logs
docker-compose restart <service>     # Restart service
docker-compose up -d --scale <service>=3  # Scale service

# Monitoring
docker stats                         # Resource usage
docker-compose top                   # Process list
htop                                 # System resources

# Database
mongosh "$DATABASE_URL"              # Connect to MongoDB
mongodump --uri="$DATABASE_URL"      # Backup database
mongorestore --uri="$DATABASE_URL"   # Restore database

# Logs
journalctl -u docker -f              # Docker daemon logs
tail -f /var/log/syslog             # System logs
```

### Contact Directory

| Role | Name | Email | Phone |
|------|------|-------|-------|
| On-Call Engineer | TBD | oncall@your-org.com | +1-555-0101 |
| Tech Lead | TBD | techlead@your-org.com | +1-555-0103 |
| DevOps Lead | TBD | devops@your-org.com | +1-555-0105 |
| Security Lead | TBD | security@your-org.com | +1-555-0107 |

---

**Document Version:** 2.0.0
**Next Review:** 2025-12-18
**Owner:** Operations Team
