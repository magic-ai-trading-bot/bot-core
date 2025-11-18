# Backup & Restore Guide

**Document Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Production-Ready

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Backup System](#2-backup-system)
- [3. Restore Procedures](#3-restore-procedures)
- [4. Configuration](#4-configuration)
- [5. Monitoring](#5-monitoring)
- [6. Troubleshooting](#6-troubleshooting)

---

## 1. Overview

### 1.1 Purpose

Comprehensive guide for backing up and restoring the Bot-Core trading platform.

### 1.2 Backup Components

| Component | Frequency | Retention | Storage |
|-----------|-----------|-----------|---------|
| MongoDB Database | Hourly (incremental), Daily (full) | 7 daily, 4 weekly, 12 monthly | Local + S3 |
| Docker Volumes | Daily | 7 days | Local |
| Configuration Files | Daily | 90 days | Local + S3 |
| AI Models | On change | All versions | S3 with versioning |

### 1.3 Key Features

- ✅ Automated backups (hourly/daily)
- ✅ Compression (gzip/bzip2/xz)
- ✅ Encryption (GPG AES256)
- ✅ Cloud upload (S3/GCS/SFTP)
- ✅ Verification & integrity checks
- ✅ Point-in-time recovery
- ✅ Automated retention policy
- ✅ Health monitoring & alerts

---

## 2. Backup System

### 2.1 Quick Start

**Manual Full Backup:**
```bash
# Complete system backup
./scripts/backup/backup-all.sh

# MongoDB only
./scripts/backup/backup-mongodb.sh full

# Volumes only
./scripts/backup/backup-volumes.sh

# Configuration only
./scripts/backup/backup-config.sh
```

**View Backup Status:**
```bash
# List all backups
find /backups -name "*.tar.gz*" -type f | sort -r | head -20

# Check latest backup
ls -lth /backups/mongodb/*/mongodb_full_*.tar.gz* | head -1

# Verify backups
./scripts/verify-backups.sh
```

### 2.2 Automated Backups

**Setup Cron Jobs:**
```bash
# Edit crontab
crontab -e

# Add backup schedule (example)
0 * * * * /path/to/bot-core/scripts/backup/backup-mongodb.sh incremental
0 2 * * * /path/to/bot-core/scripts/backup/backup-all.sh
0 4 * * * /path/to/bot-core/scripts/cleanup-old-backups.sh
```

**Or use provided crontab:**
```bash
# Install provided cron schedule
cat infrastructure/cron/backup-crontab >> /tmp/mycron
crontab /tmp/mycron

# Verify cron is running
crontab -l
```

**Schedule Overview:**
- **Hourly:** Incremental MongoDB backup
- **Daily (2 AM):** Full MongoDB backup
- **Daily (3 AM):** Complete system backup
- **Daily (4 AM):** Cleanup old backups
- **Weekly (Sunday 3 AM):** Backup verification
- **Weekly (Sunday 4 AM):** Test restore
- **Monthly:** DR drill

### 2.3 Backup Configuration

**Environment Variables (.env):**
```bash
# Backup directories
BACKUP_DIR=/backups
LOG_DIR=/var/log/backups

# MongoDB
MONGO_CONTAINER=mongodb-primary
DATABASE_URL=mongodb://admin:password@localhost:27017/bot_core

# Compression
COMPRESSION=gzip  # gzip, bzip2, xz

# Encryption
ENCRYPTION_ENABLED=true
GPG_PASSPHRASE_FILE=/etc/backup-secrets/gpg-passphrase

# Retention (days)
RETENTION_DAILY=7
RETENTION_WEEKLY=28
RETENTION_MONTHLY=365

# Cloud storage
ENABLE_S3_BACKUP=true
S3_BACKUP_BUCKET=s3://bot-core-backups
AWS_REGION=us-east-1

ENABLE_GCS_BACKUP=false
GCS_BACKUP_BUCKET=gs://bot-core-backups

ENABLE_SFTP_BACKUP=false
SFTP_HOST=backup.example.com
SFTP_USER=bot-core-backup
SFTP_DIR=/backups/bot-core

# Notifications
SLACK_WEBHOOK=https://hooks.slack.com/services/YOUR/WEBHOOK/URL
EMAIL_RECIPIENT=ops@botcore.app
ENABLE_NOTIFICATIONS=true

# Monitoring
HEALTHCHECK_URL=https://hc-ping.com/your-uuid
```

### 2.4 Encryption Setup

**Generate GPG Key:**
```bash
# Generate new GPG key
gpg --gen-key

# Or use passphrase file
mkdir -p /etc/backup-secrets
chmod 700 /etc/backup-secrets

# Create passphrase (minimum 32 characters)
openssl rand -base64 32 > /etc/backup-secrets/gpg-passphrase
chmod 400 /etc/backup-secrets/gpg-passphrase
```

### 2.5 Cloud Storage Setup

**AWS S3:**
```bash
# Install AWS CLI
pip install awscli

# Configure credentials
aws configure

# Create bucket
aws s3 mb s3://bot-core-backups --region us-east-1

# Enable versioning
aws s3api put-bucket-versioning \
    --bucket bot-core-backups \
    --versioning-configuration Status=Enabled

# Enable encryption
aws s3api put-bucket-encryption \
    --bucket bot-core-backups \
    --server-side-encryption-configuration '{
        "Rules": [{
            "ApplyServerSideEncryptionByDefault": {
                "SSEAlgorithm": "AES256"
            }
        }]
    }'
```

**Google Cloud Storage:**
```bash
# Install gcloud SDK
# See: https://cloud.google.com/sdk/docs/install

# Authenticate
gcloud auth login

# Create bucket
gsutil mb -l us-east1 gs://bot-core-backups

# Enable versioning
gsutil versioning set on gs://bot-core-backups
```

**SFTP:**
```bash
# Generate SSH key
ssh-keygen -t ed25519 -f ~/.ssh/backup_key

# Copy public key to backup server
ssh-copy-id -i ~/.ssh/backup_key.pub backup-user@backup-server

# Test connection
sftp -i ~/.ssh/backup_key backup-user@backup-server
```

### 2.6 Backup Locations

**Local Backups:**
```
/backups/
├── mongodb/
│   ├── 2025/
│   │   └── 11/
│   │       ├── mongodb_full_20251118_020000.tar.gz.gpg
│   │       ├── mongodb_full_20251118_020000.tar.gz.gpg.sha256
│   │       └── mongodb_incremental_20251118_080000.tar.gz.gpg
│   └── verified/
├── volumes/
│   ├── rust_target_cache_20251118_030000.tar.gz
│   ├── redis_data_20251118_030000.tar.gz
│   └── prometheus_data_20251118_030000.tar.gz
├── config/
│   └── config_20251118_030000.tar.gz
└── pre-restore-snapshots/
    └── pre_restore_20251118_100000.tar.gz
```

**Cloud Backups:**
```
s3://bot-core-backups/
├── mongodb_full/2025/11/
├── mongodb_incremental/2025/11/
├── volumes/2025/11/
└── config/2025/11/
```

---

## 3. Restore Procedures

### 3.1 Quick Restore

**Interactive MongoDB Restore:**
```bash
# Restore MongoDB (interactive - select from available backups)
./scripts/restore/restore-mongodb.sh

# Follow prompts to select backup
# Script will:
#   1. List available backups
#   2. Verify backup integrity
#   3. Create pre-restore snapshot
#   4. Perform restore
#   5. Verify restored data
```

**Restore Specific Backup:**
```bash
# Restore from specific backup file
./scripts/restore/restore-mongodb.sh /backups/mongodb/2025/11/mongodb_full_20251118_020000.tar.gz.gpg

# Dry-run (test without restoring)
./scripts/restore/restore-mongodb.sh --dry-run /backups/mongodb/2025/11/mongodb_full_20251118_020000.tar.gz.gpg
```

**Full System Restore:**
```bash
# Restore everything (MongoDB + volumes + config)
./scripts/restore/restore-full.sh

# Follow interactive prompts
```

### 3.2 Restore from Cloud

**Download from S3:**
```bash
# List available backups
aws s3 ls s3://bot-core-backups/mongodb_full/2025/11/

# Download specific backup
aws s3 cp s3://bot-core-backups/mongodb_full/2025/11/mongodb_full_20251118_020000.tar.gz.gpg /tmp/

# Restore
./scripts/restore/restore-mongodb.sh /tmp/mongodb_full_20251118_020000.tar.gz.gpg
```

**Download from GCS:**
```bash
# List backups
gsutil ls gs://bot-core-backups/mongodb_full/2025/11/

# Download
gsutil cp gs://bot-core-backups/mongodb_full/2025/11/mongodb_full_20251118_020000.tar.gz.gpg /tmp/

# Restore
./scripts/restore/restore-mongodb.sh /tmp/mongodb_full_20251118_020000.tar.gz.gpg
```

### 3.3 Point-in-Time Recovery

**Restore to Specific Time:**
```bash
# 1. Find backup closest to desired time
find /backups/mongodb -name "mongodb_full_*.tar.gz*" | grep "20251118"

# 2. Restore that backup
./scripts/restore/restore-mongodb.sh /backups/mongodb/2025/11/mongodb_full_20251118_020000.tar.gz.gpg

# 3. If using incremental backups, replay oplog
# (Advanced - requires MongoDB oplog)
```

### 3.4 Rollback After Failed Restore

**If restore fails:**
```bash
# Pre-restore snapshot is created automatically
# Location: /backups/pre-restore-snapshots/pre_restore_TIMESTAMP.tar.gz

# List snapshots
ls -lth /backups/pre-restore-snapshots/

# Restore from snapshot
./scripts/restore/restore-mongodb.sh /backups/pre-restore-snapshots/pre_restore_20251118_100000.tar.gz
```

### 3.5 Restore Individual Components

**Restore Specific Volume:**
```bash
./scripts/restore/restore-volumes.sh /backups/volumes/redis_data_20251118_030000.tar.gz
```

**Restore Configuration:**
```bash
./scripts/restore/restore-config.sh /backups/config/config_20251118_030000.tar.gz
```

### 3.6 Emergency Recovery

**Complete system failure:**
```bash
# 1. Deploy fresh infrastructure
docker-compose up -d

# 2. Stop services
docker-compose stop

# 3. Restore configuration
./scripts/restore/restore-config.sh <latest-config-backup>

# 4. Restore MongoDB
./scripts/restore/restore-mongodb.sh <latest-mongodb-backup>

# 5. Restore volumes (optional)
for backup in /backups/volumes/*.tar.gz; do
    ./scripts/restore/restore-volumes.sh "$backup"
done

# 6. Start services
docker-compose start

# 7. Verify
docker-compose ps
curl -f http://localhost:8080/api/health
```

---

## 4. Configuration

### 4.1 Retention Policy

**Default Retention:**
- Daily backups: 7 days
- Weekly backups: 4 weeks
- Monthly backups: 12 months

**Modify Retention:**
```bash
# Edit .env
RETENTION_DAILY=14        # Keep 14 days
RETENTION_WEEKLY=56       # Keep 8 weeks
RETENTION_MONTHLY=730     # Keep 2 years
```

**Manual Cleanup:**
```bash
# Cleanup old backups
./scripts/cleanup-old-backups.sh

# Dry-run (see what would be deleted)
./scripts/cleanup-old-backups.sh --dry-run
```

### 4.2 Compression Options

**Available Formats:**
```bash
# gzip (fastest, default)
COMPRESSION=gzip

# bzip2 (better compression)
COMPRESSION=bzip2

# xz (best compression, slowest)
COMPRESSION=xz
```

**Compression Comparison:**
| Format | Speed | Compression Ratio | CPU Usage |
|--------|-------|-------------------|-----------|
| gzip | Fast | Good | Low |
| bzip2 | Medium | Better | Medium |
| xz | Slow | Best | High |

### 4.3 Notifications

**Slack Integration:**
```bash
# Create Slack webhook
# 1. Go to https://api.slack.com/apps
# 2. Create new app
# 3. Add Incoming Webhook
# 4. Copy webhook URL

# Configure in .env
SLACK_WEBHOOK=https://hooks.slack.com/services/YOUR/WEBHOOK/URL
ENABLE_NOTIFICATIONS=true
```

**Email Alerts:**
```bash
# Install mail utility
sudo apt-get install mailutils  # Ubuntu/Debian
sudo yum install mailx          # RHEL/CentOS

# Configure in .env
EMAIL_RECIPIENT=ops@botcore.app
ENABLE_NOTIFICATIONS=true
```

---

## 5. Monitoring

### 5.1 Backup Verification

**Verify All Backups:**
```bash
# Run verification
./scripts/verify-backups.sh

# With test restore
./scripts/verify-backups.sh --test-restore

# View verification report
cat /backups/verification-report-$(date +%Y%m%d).txt
```

**Verification Report Example:**
```
================================================================================
BACKUP VERIFICATION REPORT
Generated: 2025-11-18 10:00:00
================================================================================

MONGODB BACKUPS:
  [OK] /backups/mongodb/2025/11/mongodb_full_20251118_020000.tar.gz.gpg
  [OK] /backups/mongodb/2025/11/mongodb_full_20251117_020000.tar.gz.gpg

VOLUME BACKUPS:
  [OK] /backups/volumes/redis_data_20251118_030000.tar.gz
  [WARN] No checksum: /backups/volumes/prometheus_data_20251118_030000.tar.gz

BACKUP FRESHNESS:
  Latest full backup: mongodb_full_20251118_020000.tar.gz.gpg
  Age: 8 hours
  Status: ✓ FRESH

DISK SPACE:
  Backup directory size: 15G
  Available space: 85G
  Disk usage: 15%
  Status: ✓ OK

================================================================================
SUMMARY
================================================================================
Total backups checked:     25
Verified OK:               24
Verification failed:       0
Missing checksums:         1

Overall Status: ✓ ALL BACKUPS VERIFIED
```

### 5.2 Health Checks

**Check Backup Health:**
```bash
# Manual health check
./scripts/check-backup-health.sh

# Checks:
#   - Last backup age (< 25 hours)
#   - Disk space (> 10GB)
#   - Backup count (>= 7 in last 7 days)
#   - Failed backup logs
```

**Daily Status Report:**
```bash
# Generate daily report
./scripts/backup-status-report.sh

# Sent to: ops@botcore.app
# Logged to: /var/log/backups/daily-reports-YYYYMM.log
```

### 5.3 Metrics & Monitoring

**Prometheus Metrics:**
```bash
# Metrics file location
cat /var/lib/prometheus/node_exporter/backup_metrics.prom

# Available metrics:
#   - mongodb_backup_last_success_timestamp
#   - mongodb_backup_success (1=success, 0=failure)
#   - mongodb_backup_size_bytes
#   - mongodb_backup_duration_seconds
```

**Grafana Dashboard:**
- Import dashboard: `infrastructure/monitoring/grafana/dashboards/backup-metrics.json`
- Panels:
  - Backup success rate
  - Backup size trend
  - Backup duration
  - Last successful backup time
  - Disk space usage

### 5.4 Logs

**Log Locations:**
```bash
# Backup logs
/var/log/backups/mongodb-backup-YYYYMMDD.log
/var/log/backups/volumes-backup-YYYYMMDD.log
/var/log/backups/config-backup-YYYYMMDD.log

# Restore logs
/var/log/backups/restore-YYYYMMDD_HHMMSS.log

# Cron logs
/var/log/backups/cron-*.log

# Health check logs
/var/log/backups/health-check-YYYYMMDD.log
```

**View Logs:**
```bash
# Today's MongoDB backup log
tail -f /var/log/backups/mongodb-backup-$(date +%Y%m%d).log

# All errors today
grep ERROR /var/log/backups/*.log

# Cron execution
grep backup /var/log/syslog
```

---

## 6. Troubleshooting

### 6.1 Common Issues

**Issue: Backup fails with "No space left on device"**
```bash
# Solution 1: Free up space
./scripts/cleanup-old-backups.sh

# Solution 2: Check disk usage
df -h /backups

# Solution 3: Mount larger volume
# Increase disk size or mount external storage
```

**Issue: Encryption fails - GPG key not found**
```bash
# Solution: Generate GPG passphrase
mkdir -p /etc/backup-secrets
chmod 700 /etc/backup-secrets
openssl rand -base64 32 > /etc/backup-secrets/gpg-passphrase
chmod 400 /etc/backup-secrets/gpg-passphrase
```

**Issue: Cloud upload fails - AWS credentials not found**
```bash
# Solution: Configure AWS CLI
aws configure

# Or set credentials in .env
export AWS_ACCESS_KEY_ID=your_key
export AWS_SECRET_ACCESS_KEY=your_secret
export AWS_DEFAULT_REGION=us-east-1
```

**Issue: Restore fails - checksum mismatch**
```bash
# Solution 1: Try different backup
./scripts/restore/restore-mongodb.sh

# Select an older verified backup

# Solution 2: Restore from cloud
# Cloud backups have replication and integrity checks
aws s3 cp s3://bot-core-backups/mongodb_full/... /tmp/
./scripts/restore/restore-mongodb.sh /tmp/mongodb_full_...
```

**Issue: MongoDB container not running**
```bash
# Solution: Start MongoDB
docker-compose up -d mongodb

# Or check logs
docker logs mongodb-primary
```

### 6.2 Manual Backup

**If automated backup fails:**
```bash
# Manual MongoDB backup
docker exec mongodb-primary mongodump \
    --uri="$DATABASE_URL" \
    --out=/tmp/manual-backup-$(date +%Y%m%d) \
    --gzip

# Copy out of container
docker cp mongodb-primary:/tmp/manual-backup-$(date +%Y%m%d) /backups/

# Compress
tar czf /backups/manual-backup-$(date +%Y%m%d).tar.gz \
    /backups/manual-backup-$(date +%Y%m%d)
```

### 6.3 Recovery from Complete Failure

**All backups lost:**
```bash
# 1. Check cloud storage
aws s3 ls s3://bot-core-backups/mongodb_full/ --recursive

# 2. Download all available backups
aws s3 sync s3://bot-core-backups/mongodb_full/ /backups/mongodb/

# 3. Verify and restore latest
./scripts/verify-backups.sh
./scripts/restore/restore-mongodb.sh
```

### 6.4 Test Restore

**Regular testing (recommended monthly):**
```bash
# Full DR test
./scripts/test-dr.sh

# Manual test restore to staging
./scripts/restore/restore-mongodb.sh --dry-run <backup-file>

# Or restore to test database
DATABASE_URL=mongodb://localhost:27017/test_restore \
    ./scripts/restore/restore-mongodb.sh <backup-file>
```

---

## Appendix A: Backup Script Reference

| Script | Purpose | Usage |
|--------|---------|-------|
| backup-mongodb.sh | MongoDB backup | `./backup-mongodb.sh [full\|incremental]` |
| backup-volumes.sh | Docker volumes backup | `./backup-volumes.sh` |
| backup-config.sh | Configuration backup | `./backup-config.sh` |
| backup-all.sh | Complete system backup | `./backup-all.sh` |
| restore-mongodb.sh | MongoDB restore | `./restore-mongodb.sh [backup-file]` |
| restore-volumes.sh | Volume restore | `./restore-volumes.sh <backup-file>` |
| restore-config.sh | Configuration restore | `./restore-config.sh <backup-file>` |
| restore-full.sh | Full system restore | `./restore-full.sh` |
| verify-backups.sh | Verify all backups | `./verify-backups.sh [--test-restore]` |
| cleanup-old-backups.sh | Apply retention policy | `./cleanup-old-backups.sh [--dry-run]` |
| check-backup-health.sh | Health monitoring | `./check-backup-health.sh` |
| test-dr.sh | DR drill | `./test-dr.sh [--automated]` |

---

## Appendix B: Recovery Time Objectives

| Scenario | RTO Target | Actual | RPO Target | Actual |
|----------|------------|--------|------------|--------|
| Database corruption | 1 hour | ~30 min | 5 min | ~5 min |
| Complete system failure | 2 hours | ~60 min | 5 min | ~5 min |
| Single service failure | 30 min | ~15 min | 5 min | ~5 min |
| Regional outage | 4 hours | ~2 hours | 15 min | ~15 min |

---

**Document End**
