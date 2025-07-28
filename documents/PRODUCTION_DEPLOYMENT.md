# Production Deployment Guide

## 🚀 Prerequisites

- Docker 20.10+
- Docker Compose 2.0+
- 8GB RAM minimum
- 50GB disk space
- Domain with SSL certificate
- MongoDB Atlas account (recommended)
- Monitoring infrastructure

## 📋 Pre-Deployment Checklist

### 1. Environment Setup
```bash
# Clone repository
git clone https://github.com/your-org/bot-core.git
cd bot-core

# Generate secure secrets
./scripts/generate-secrets.sh

# Edit .env with production values
nano .env
```

### 2. Required Environment Variables
```env
# Production settings
NODE_ENV=production
LOG_LEVEL=info
BINANCE_TESTNET=false  # CAREFUL: Real trading!
TRADING_ENABLED=false  # Enable manually after testing

# Database (use MongoDB Atlas)
DATABASE_URL=mongodb+srv://...

# API Keys (from providers)
BINANCE_API_KEY=...
BINANCE_SECRET_KEY=...
OPENAI_API_KEY=...

# Generated secrets (from script)
INTER_SERVICE_TOKEN=...
JWT_SECRET=...
```

## 🏗️ Deployment Steps

### 1. Initial Setup
```bash
# Create required directories
make setup

# Build images with production optimizations
make build-clean

# Run database migrations (if any)
docker-compose run --rm rust-core-engine migrate
```

### 2. SSL Certificate Setup
```bash
# Using Let's Encrypt
mkdir -p nginx/ssl
docker run -it --rm \
  -v $(pwd)/nginx/ssl:/etc/letsencrypt \
  certbot/certbot certonly \
  --standalone \
  -d your-domain.com \
  -d www.your-domain.com
```

### 3. Start Services
```bash
# Start with production config
docker-compose -f docker-compose.yml -f docker-compose.prod.yml up -d

# Verify health
make health

# Check logs
make logs
```

## 🔍 Post-Deployment Verification

### 1. Health Checks
```bash
# Service health
curl https://your-domain.com/health
curl https://your-domain.com/api/health
curl https://your-domain.com/ai/health

# Metrics
curl https://your-domain.com/metrics
```

### 2. Functionality Tests
```bash
# Test authentication
curl -X POST https://your-domain.com/api/register \
  -H "Content-Type: application/json" \
  -d '{"email":"test@example.com","password":"..."}'

# Test WebSocket
wscat -c wss://your-domain.com/ws
```

## 📊 Monitoring Setup

### 1. Prometheus & Grafana
```bash
# Start monitoring stack
docker-compose --profile monitoring up -d

# Access dashboards
# Prometheus: http://your-domain.com:9090
# Grafana: http://your-domain.com:3001 (admin/admin)
```

### 2. Import Dashboards
1. Login to Grafana
2. Import dashboard IDs:
   - 1860 (Node Exporter)
   - 893 (Docker Monitoring)
   - Custom bot-core dashboard

### 3. Configure Alerts
- Edit `monitoring/alerts/alerts.yml`
- Set notification channels in Grafana
- Test alert delivery

## 🔄 Update Procedures

### 1. Rolling Update
```bash
# Pull latest changes
git pull origin main

# Build new images
make build

# Update services one by one
docker-compose up -d --no-deps rust-core-engine
# Wait and verify
docker-compose up -d --no-deps python-ai-service
# Wait and verify
docker-compose up -d --no-deps nextjs-ui-dashboard
```

### 2. Database Backup
```bash
# Before any major update
make db-backup

# Verify backup
ls -la backup_*.sql
```

## 🚨 Troubleshooting

### Common Issues

1. **Service Won't Start**
   ```bash
   # Check logs
   docker-compose logs service-name
   
   # Verify resources
   docker stats
   
   # Check disk space
   df -h
   ```

2. **High Memory Usage**
   ```bash
   # Apply memory limits
   docker-compose -f docker-compose.memory-optimized.yml up -d
   ```

3. **Connection Issues**
   ```bash
   # Check network
   docker network ls
   docker network inspect bot-network
   
   # Restart network
   docker-compose down
   docker network prune -f
   docker-compose up -d
   ```

## 🔐 Security Hardening

### 1. Firewall Rules
```bash
# Allow only required ports
ufw allow 22/tcp   # SSH
ufw allow 80/tcp   # HTTP (redirect)
ufw allow 443/tcp  # HTTPS
ufw enable
```

### 2. Fail2ban Setup
```bash
# Install fail2ban
apt-get install fail2ban

# Configure for Docker
# /etc/fail2ban/jail.local
[DEFAULT]
bantime = 1h
maxretry = 5

[docker-nginx]
enabled = true
filter = docker-nginx
logpath = /var/lib/docker/containers/*/*.log
```

## 📈 Performance Optimization

### 1. Resource Limits
```yaml
# docker-compose.prod.yml adjustments
services:
  rust-core-engine:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: "2"
```

### 2. Database Optimization
- Enable connection pooling
- Set appropriate indexes
- Regular VACUUM (PostgreSQL)
- Optimize queries

### 3. Caching Strategy
- Redis for session storage
- Nginx proxy cache for static assets
- CDN for frontend assets

## 🔄 Backup & Recovery

### 1. Automated Backups
```bash
# Add to crontab
0 2 * * * /opt/bot-core/scripts/backup.sh
```

### 2. Recovery Procedure
```bash
# Stop services
docker-compose down

# Restore database
make db-restore BACKUP_FILE=backup_20240101.sql

# Start services
docker-compose up -d
```

## 📞 Support & Maintenance

### Regular Maintenance
- Weekly: Review logs, check metrics
- Monthly: Update dependencies, security patches
- Quarterly: Performance review, capacity planning
- Annually: Disaster recovery drill

### Monitoring Checklist
- [ ] All services running
- [ ] CPU usage < 80%
- [ ] Memory usage < 90%
- [ ] Disk usage < 80%
- [ ] No critical alerts
- [ ] API response times < 500ms
- [ ] WebSocket connections stable
- [ ] Trading operations normal