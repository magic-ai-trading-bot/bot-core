# Runbook: Backup Failure

**Document Version:** 1.0.0
**Last Updated:** 2025-11-18
**Severity:** High
**Response Time:** < 4 hours

---

## Symptoms

- Backup cron job fails
- No new backups in last 24-48 hours
- Backup verification alerts
- Monitoring alerts: "Backup failed" or "Backup too old"

---

## Initial Response (First 15 Minutes)

### 1. Verify the Issue

```bash
# Check last backup time
ls -lth /backups/mongodb/*/mongodb_full_*.tar.gz* | head -1

# Check backup logs
tail -100 /var/log/backups/mongodb-backup-$(date +%Y%m%d).log

# Check cron execution
grep backup /var/log/syslog | tail -20
```

### 2. Identify Failure Type

**Check for common issues:**

```bash
# A. Disk space
df -h /backups

# B. MongoDB container status
docker ps | grep mongodb

# C. Permissions
ls -la /backups/
ls -la /var/log/backups/

# D. Recent errors
grep -i error /var/log/backups/*.log | tail -20
```

### 3. Quick Assessment

**Determine severity:**

- ✅ **Low:** Single backup failed, previous backups OK
- ⚠️ **Medium:** Multiple consecutive failures (2-3)
- 🔴 **High:** No backups for > 24 hours
- 🚨 **Critical:** No backups for > 48 hours OR all backups corrupted

---

## Investigation (15-30 Minutes)

### Scenario A: Disk Space Full

**Symptoms:**
```
ERROR: No space left on device
df shows > 95% usage
```

**Resolution:**
```bash
# 1. Immediate: Free up space
./scripts/cleanup-old-backups.sh

# 2. Check disk usage
du -sh /backups/*

# 3. Remove oldest backups manually
find /backups -name "*.tar.gz*" -mtime +30 -delete

# 4. Retry backup
./scripts/backup/backup-mongodb.sh full

# 5. Long-term: Increase disk size or add retention cleanup to cron
```

### Scenario B: MongoDB Container Not Running

**Symptoms:**
```
ERROR: MongoDB container 'mongodb-primary' is not running
docker ps shows no mongodb container
```

**Resolution:**
```bash
# 1. Start MongoDB
docker-compose up -d mongodb

# 2. Wait for healthy
docker-compose ps mongodb

# 3. Check logs
docker logs mongodb-primary --tail 50

# 4. Retry backup
./scripts/backup/backup-mongodb.sh full
```

### Scenario C: Permissions Issue

**Symptoms:**
```
ERROR: Permission denied
Cannot write to /backups/
```

**Resolution:**
```bash
# 1. Fix ownership
sudo chown -R $(whoami):$(whoami) /backups
sudo chown -R $(whoami):$(whoami) /var/log/backups

# 2. Fix permissions
chmod 700 /backups
chmod 700 /var/log/backups

# 3. Retry backup
./scripts/backup/backup-mongodb.sh full
```

### Scenario D: Encryption Failure

**Symptoms:**
```
ERROR: gpg: encryption failed
ERROR: No encryption method configured
```

**Resolution:**
```bash
# 1. Check GPG passphrase file
ls -la /etc/backup-secrets/gpg-passphrase

# 2. If missing, create new one
sudo mkdir -p /etc/backup-secrets
sudo chmod 700 /etc/backup-secrets
sudo openssl rand -base64 32 | sudo tee /etc/backup-secrets/gpg-passphrase
sudo chmod 400 /etc/backup-secrets/gpg-passphrase

# 3. Or disable encryption temporarily
export ENCRYPTION_ENABLED=false
./scripts/backup/backup-mongodb.sh full

# 4. Re-enable encryption after fixing
```

### Scenario E: Cloud Upload Failure

**Symptoms:**
```
ERROR: Upload failed
ERROR: AWS credentials not found
ERROR: S3 upload failed
```

**Resolution:**
```bash
# 1. Check AWS credentials
aws sts get-caller-identity

# 2. If fails, reconfigure
aws configure

# 3. Test S3 access
aws s3 ls s3://bot-core-backups/

# 4. Check bucket exists
aws s3 mb s3://bot-core-backups --region us-east-1

# 5. Retry backup
./scripts/backup/backup-mongodb.sh full

# 6. If still fails, disable cloud upload temporarily
export ENABLE_S3_BACKUP=false
./scripts/backup/backup-mongodb.sh full
```

### Scenario F: MongoDB Dump Failure

**Symptoms:**
```
ERROR: mongodump failed
ERROR: Failed to connect to MongoDB
```

**Resolution:**
```bash
# 1. Test MongoDB connection
docker exec mongodb-primary mongosh --eval "db.version()"

# 2. Check DATABASE_URL
echo $DATABASE_URL

# 3. Test mongodump manually
docker exec mongodb-primary mongodump \
    --uri="$DATABASE_URL" \
    --out=/tmp/test-dump \
    --gzip

# 4. If fails, check MongoDB logs
docker logs mongodb-primary --tail 100

# 5. If authentication issue, update credentials
# Edit .env and update DATABASE_URL
```

---

## Resolution Steps

### Step 1: Manual Backup (Immediate)

**While investigating, take manual backup:**

```bash
# Quick manual backup
docker exec mongodb-primary mongodump \
    --uri="$DATABASE_URL" \
    --out=/tmp/emergency-backup-$(date +%Y%m%d_%H%M%S) \
    --gzip

# Copy to backup location
docker cp mongodb-primary:/tmp/emergency-backup-* /backups/manual/

# Verify backup
ls -lh /backups/manual/
```

### Step 2: Fix Root Cause

**Apply appropriate fix from investigation scenarios above**

### Step 3: Test Backup

```bash
# Run backup manually
./scripts/backup/backup-mongodb.sh full

# Verify success
ls -lth /backups/mongodb/*/mongodb_full_*.tar.gz* | head -1

# Check logs
tail -50 /var/log/backups/mongodb-backup-$(date +%Y%m%d).log
```

### Step 4: Verify Backup Integrity

```bash
# Run verification
./scripts/verify-backups.sh

# Check report
cat /backups/verification-report-$(date +%Y%m%d).txt
```

### Step 5: Test Restore (Optional but Recommended)

```bash
# Dry-run restore to verify backup is usable
./scripts/restore/restore-mongodb.sh --dry-run \
    /backups/mongodb/*/mongodb_full_$(date +%Y%m%d)*.tar.gz*
```

---

## Prevention

### 1. Increase Monitoring

```bash
# Add healthcheck URL
# Sign up at: https://healthchecks.io
export HEALTHCHECK_URL=https://hc-ping.com/your-uuid

# Test healthcheck
curl -fsS -m 10 --retry 3 $HEALTHCHECK_URL
```

### 2. Set Up Alerts

```bash
# Slack webhook
export SLACK_WEBHOOK=https://hooks.slack.com/services/YOUR/WEBHOOK

# Email alerts
export EMAIL_RECIPIENT=ops@botcore.app
export ENABLE_NOTIFICATIONS=true
```

### 3. Regular Testing

```bash
# Add to cron (weekly test)
0 4 * * 0 /path/to/bot-core/scripts/verify-backups.sh --test-restore

# Monthly DR drill
0 5 1 * * /path/to/bot-core/scripts/test-dr.sh --automated
```

### 4. Documentation Update

```bash
# Update runbook with new issues found
vim docs/runbooks/BACKUP_FAILURE_RUNBOOK.md

# Commit changes
git add docs/runbooks/BACKUP_FAILURE_RUNBOOK.md
git commit -m "docs: update backup failure runbook with new scenario"
```

---

## Escalation

### Level 1: Self-Service (0-30 minutes)

- Follow this runbook
- Check common issues
- Attempt manual backup

### Level 2: Team Lead (30-60 minutes)

**Escalate if:**
- Unable to identify root cause
- Multiple failures after fixes
- System-wide issues

**Contact:**
- Operations Lead: ops-lead@botcore.app
- On-call: +1-xxx-xxx-xxxx

### Level 3: Management (> 60 minutes)

**Escalate if:**
- No backups possible
- Data loss risk
- Infrastructure failure

**Contact:**
- CTO: cto@botcore.app
- Incident Commander: ic@botcore.app

---

## Post-Incident

### 1. Create Incident Report

```markdown
## Incident Report: Backup Failure

**Date:** YYYY-MM-DD
**Duration:** X hours
**Severity:** High/Medium/Low

**Summary:**
[Brief description of what happened]

**Root Cause:**
[What caused the failure]

**Resolution:**
[How it was fixed]

**Impact:**
[How many backups missed, any data loss]

**Prevention:**
[What will prevent this in future]
```

### 2. Update Monitoring

- Add check for new failure scenario
- Improve alerting thresholds
- Add metric to dashboard

### 3. Review & Improve

- Update runbook with lessons learned
- Add automated check if possible
- Review retention policy

---

## Quick Reference

**Key Commands:**
```bash
# Check last backup
ls -lth /backups/mongodb/*/mongodb_full_*.tar.gz* | head -1

# Manual backup
./scripts/backup/backup-mongodb.sh full

# Check logs
tail -100 /var/log/backups/mongodb-backup-$(date +%Y%m%d).log

# Verify backups
./scripts/verify-backups.sh

# Test restore
./scripts/restore/restore-mongodb.sh --dry-run <backup-file>
```

**Key Files:**
- Backup scripts: `scripts/backup/`
- Logs: `/var/log/backups/`
- Backups: `/backups/`
- Cron: `/etc/cron.d/` or `crontab -l`
- Config: `.env`

**Emergency Contacts:**
- Operations: ops@botcore.app
- On-call: +1-xxx-xxx-xxxx
- Slack: #incidents

---

**Document End**
