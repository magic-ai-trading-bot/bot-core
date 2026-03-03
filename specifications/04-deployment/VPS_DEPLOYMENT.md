# Viettel VPS Deployment Guide - Bot Core Trading Platform

**Version:** 1.0.0
**Last Updated:** 2024-11-24
**Target VPS:** Viettel IDC T2.GEN 03 (8 vCPU / 8 GB RAM / 100 GB SSD)
**Estimated Setup Time:** 2-3 hours
**Difficulty:** Intermediate

---

## 📋 Table of Contents

1. [Pre-Deployment Checklist](#pre-deployment-checklist)
2. [Server Access & Initial Setup](#server-access--initial-setup)
3. [Security Hardening](#security-hardening)
4. [Install Required Software](#install-required-software)
5. [Clone & Configure Bot-Core](#clone--configure-bot-core)
6. [Environment Configuration](#environment-configuration)
7. [Deploy Services](#deploy-services)
8. [Verify Deployment](#verify-deployment)
9. [Setup Monitoring](#setup-monitoring)
10. [Backup & Disaster Recovery](#backup--disaster-recovery)
11. [Troubleshooting](#troubleshooting)

---

## 🎯 Pre-Deployment Checklist

### **Before You Start**

- [ ] VPS Viettel T2.GEN 03 đã được provisioned
- [ ] Nhận được thông tin SSH (IP, username, password/key)
- [ ] Domain name (nếu có) đã trỏ về VPS IP
- [ ] Binance API keys (testnet hoặc production)
- [ ] OpenAI API key (cho GPT-4 analysis)
- [ ] MongoDB connection string (hoặc sẽ cài local)
- [ ] Git repository access

### **Required Information**

```bash
# VPS Access
VPS_IP=<your-vps-ip>                    # e.g., 123.456.789.10
VPS_USER=root                           # hoặc username được cung cấp
VPS_PASSWORD=<your-password>            # hoặc SSH key path

# Bot Configuration
BINANCE_API_KEY=<your-binance-api-key>
BINANCE_SECRET_KEY=<your-binance-secret>
BINANCE_TESTNET=true                    # QUAN TRỌNG: Luôn bắt đầu với testnet
OPENAI_API_KEY=<your-openai-key>

# Security
JWT_SECRET=<generate-strong-secret>     # Sẽ generate trong guide
```

---

## 🔐 Server Access & Initial Setup

### **Step 1: First SSH Connection**

```bash
# Connect to VPS (from your local machine)
ssh root@<VPS_IP>

# Hoặc nếu dùng password:
ssh root@<VPS_IP>
# Enter password when prompted
```

**Expected output:**
```
Welcome to Ubuntu 22.04.3 LTS
Last login: ...
root@viettel-vps:~#
```

### **Step 2: Update System**

```bash
# Update package lists
sudo apt update

# Upgrade all packages
sudo apt upgrade -y

# Install essential utilities
sudo apt install -y \
    curl \
    wget \
    git \
    vim \
    htop \
    net-tools \
    ufw \
    fail2ban \
    unzip \
    build-essential \
    software-properties-common
```

**Time:** ~5-10 minutes

### **Step 3: Create Non-Root User** (Security Best Practice)

```bash
# Create new user for deployment
adduser botadmin

# Add to sudo group
usermod -aG sudo botadmin

# Switch to new user
su - botadmin

# Verify sudo access
sudo whoami
# Should output: root
```

**From now on, use `botadmin` user (not root)**

---

## 🛡️ Security Hardening

### **Step 1: Configure Firewall (UFW)**

```bash
# Enable UFW
sudo ufw --force enable

# Default policies
sudo ufw default deny incoming
sudo ufw default allow outgoing

# Allow SSH (IMPORTANT: Do this first!)
sudo ufw allow 22/tcp

# Allow application ports
sudo ufw allow 80/tcp      # HTTP
sudo ufw allow 443/tcp     # HTTPS
sudo ufw allow 8080/tcp    # Rust Core Engine API
sudo ufw allow 8000/tcp    # Python AI Service API
sudo ufw allow 3000/tcp    # Next.js Dashboard

# Check status
sudo ufw status verbose
```

**Expected output:**
```
Status: active

To                         Action      From
--                         ------      ----
22/tcp                     ALLOW       Anywhere
80/tcp                     ALLOW       Anywhere
443/tcp                    ALLOW       Anywhere
8080/tcp                   ALLOW       Anywhere
8000/tcp                   ALLOW       Anywhere
3000/tcp                   ALLOW       Anywhere
```

### **Step 2: Configure Fail2Ban** (Prevent Brute Force)

```bash
# Install and enable fail2ban
sudo apt install -y fail2ban

# Create local config
sudo cp /etc/fail2ban/jail.conf /etc/fail2ban/jail.local

# Edit config
sudo vim /etc/fail2ban/jail.local
```

**Add these settings:**
```ini
[sshd]
enabled = true
port = 22
filter = sshd
logpath = /var/log/auth.log
maxretry = 3
bantime = 3600
findtime = 600
```

```bash
# Restart fail2ban
sudo systemctl restart fail2ban

# Check status
sudo fail2ban-client status sshd
```

### **Step 3: SSH Key Authentication** (More Secure)

**On your local machine:**
```bash
# Generate SSH key pair (if you don't have one)
ssh-keygen -t ed25519 -C "your_email@example.com"

# Copy public key to VPS
ssh-copy-id botadmin@<VPS_IP>
```

**On VPS:**
```bash
# Disable password authentication (optional but recommended)
sudo vim /etc/ssh/sshd_config
```

**Change these lines:**
```
PasswordAuthentication no
PubkeyAuthentication yes
PermitRootLogin no
```

```bash
# Restart SSH
sudo systemctl restart sshd
```

---

## 📦 Install Required Software

### **Step 1: Install Docker**

```bash
# Add Docker's official GPG key
curl -fsSL https://download.docker.com/linux/ubuntu/gpg | sudo gpg --dearmor -o /usr/share/keyrings/docker-archive-keyring.gpg

# Add Docker repository
echo \
  "deb [arch=$(dpkg --print-architecture) signed-by=/usr/share/keyrings/docker-archive-keyring.gpg] https://download.docker.com/linux/ubuntu \
  $(lsb_release -cs) stable" | sudo tee /etc/apt/sources.list.d/docker.list > /dev/null

# Update and install Docker
sudo apt update
sudo apt install -y docker-ce docker-ce-cli containerd.io docker-buildx-plugin docker-compose-plugin

# Add user to docker group
sudo usermod -aG docker $USER

# Apply group changes (logout and login, or run:)
newgrp docker

# Verify installation
docker --version
docker compose version
```

**Expected output:**
```
Docker version 24.0.7, build afdd53b
Docker Compose version v2.23.0
```

### **Step 2: Install MongoDB** (Local Installation)

```bash
# Import MongoDB public GPG key
curl -fsSL https://www.mongodb.org/static/pgp/server-7.0.asc | \
   sudo gpg -o /usr/share/keyrings/mongodb-server-7.0.gpg \
   --dearmor

# Add MongoDB repository
echo "deb [ arch=amd64,arm64 signed-by=/usr/share/keyrings/mongodb-server-7.0.gpg ] https://repo.mongodb.org/apt/ubuntu jammy/mongodb-org/7.0 multiverse" | \
   sudo tee /etc/apt/sources.list.d/mongodb-org-7.0.list

# Update and install MongoDB
sudo apt update
sudo apt install -y mongodb-org

# Start MongoDB
sudo systemctl start mongod
sudo systemctl enable mongod

# Verify MongoDB is running
sudo systemctl status mongod
```

**Expected output:**
```
● mongod.service - MongoDB Database Server
     Loaded: loaded (/lib/systemd/system/mongod.service; enabled; vendor preset: enabled)
     Active: active (running) since ...
```

### **Step 3: Configure MongoDB**

```bash
# Create admin user
mongosh
```

**In MongoDB shell:**
```javascript
use admin

db.createUser({
  user: "admin",
  pwd: "CHANGE_THIS_PASSWORD_123",
  roles: [ { role: "userAdminAnyDatabase", db: "admin" }, "readWriteAnyDatabase" ]
})

// Create database for bot-core
use botcore_production

db.createUser({
  user: "botcore_user",
  pwd: "CHANGE_THIS_PASSWORD_456",
  roles: [ { role: "readWrite", db: "botcore_production" } ]
})

exit
```

**Enable authentication:**
```bash
sudo vim /etc/mongod.conf
```

**Add security settings:**
```yaml
security:
  authorization: enabled

net:
  bindIp: 127.0.0.1
  port: 27017
```

```bash
# Restart MongoDB
sudo systemctl restart mongod

# Test authentication
mongosh -u admin -p --authenticationDatabase admin
```

**MongoDB connection string:**
```
mongodb://botcore_user:CHANGE_THIS_PASSWORD_456@localhost:27017/botcore_production
```

### **Step 4: Install Node.js** (For frontend builds)

```bash
# Install Node.js 18.x LTS
curl -fsSL https://deb.nodesource.com/setup_18.x | sudo -E bash -
sudo apt install -y nodejs

# Verify installation
node --version
npm --version
```

**Expected output:**
```
v18.19.0
10.2.3
```

---

## 🚀 Clone & Configure Bot-Core

### **Step 1: Clone Repository**

```bash
# Create projects directory
mkdir -p ~/projects
cd ~/projects

# Clone repository (replace with your actual repo)
git clone https://github.com/magic-ai-trading-bot/bot-core.git

# Or if using SSH:
git clone git@github.com:magic-ai-trading-bot/bot-core.git

cd bot-core

# Verify files
ls -la
```

**Expected output:**
```
drwxr-xr-x  rust-core-engine/
drwxr-xr-x  python-ai-service/
drwxr-xr-x  nextjs-ui-dashboard/
-rw-r--r--  docker-compose.yml
-rw-r--r--  .env.example
-rw-r--r--  Makefile
...
```

### **Step 2: Generate Secrets**

```bash
# Generate JWT secret
export JWT_SECRET=$(openssl rand -base64 64)
echo "JWT Secret: $JWT_SECRET"

# Generate session secret
export SESSION_SECRET=$(openssl rand -base64 32)
echo "Session Secret: $SESSION_SECRET"

# Generate MongoDB password (if not done above)
export MONGO_PASSWORD=$(openssl rand -base64 32)
echo "MongoDB Password: $MONGO_PASSWORD"

# Save these securely!
cat > ~/bot-secrets.txt <<EOF
JWT_SECRET=$JWT_SECRET
SESSION_SECRET=$SESSION_SECRET
MONGO_PASSWORD=$MONGO_PASSWORD
Generated on: $(date)
EOF

chmod 600 ~/bot-secrets.txt
```

---

## ⚙️ Environment Configuration

### **Step 1: Create Production .env File**

```bash
cd ~/projects/bot-core

# Copy example env file
cp .env.example .env

# Edit .env file
vim .env
```

### **Step 2: Configure .env for Production**

**Complete .env configuration:**

```bash
# ============================================
# PRODUCTION ENVIRONMENT - VIETTEL VPS
# ============================================

# Environment
NODE_ENV=production
RUST_ENV=production
PYTHON_ENV=production

# Trading Configuration
TRADING_ENABLED=false                    # ⚠️ CRITICAL: Keep false until fully tested
BINANCE_TESTNET=true                     # ⚠️ CRITICAL: Start with testnet
BINANCE_API_KEY=your_binance_api_key_here
BINANCE_SECRET_KEY=your_binance_secret_key_here
BINANCE_API_URL=https://testnet.binance.vision
BINANCE_WS_URL=wss://testnet.binance.vision/ws

# AI/ML Configuration
OPENAI_API_KEY=your_openai_api_key_here
OPENAI_MODEL=gpt-4
OPENAI_MAX_TOKENS=1000

# Database - MongoDB
MONGODB_URI=mongodb://botcore_user:CHANGE_THIS_PASSWORD_456@localhost:27017/botcore_production
MONGODB_DATABASE=botcore_production

# Redis (if using)
REDIS_URL=redis://localhost:6379
REDIS_PASSWORD=

# Authentication
JWT_SECRET=<paste_jwt_secret_from_above>
JWT_EXPIRATION=24h
JWT_REFRESH_EXPIRATION=7d
SESSION_SECRET=<paste_session_secret_from_above>

# Server Ports
RUST_PORT=8080
PYTHON_PORT=8000
FRONTEND_PORT=3000

# Logging
LOG_LEVEL=info
LOG_FORMAT=json
LOG_FILE=/app/logs/app.log

# Memory Limits (for 8GB VPS)
PYTHON_MEMORY_LIMIT=2G
PYTHON_MEMORY_RESERVE=1G
RUST_MEMORY_LIMIT=2G
RUST_MEMORY_RESERVE=1G
FRONTEND_MEMORY_LIMIT=1G
FRONTEND_MEMORY_RESERVE=512M
NODE_MEMORY=1024

# CPU Limits
PYTHON_CPU_LIMIT=2
RUST_CPU_LIMIT=2
FRONTEND_CPU_LIMIT=1

# CORS (adjust for your domain)
CORS_ORIGINS=http://localhost:3000,http://<VPS_IP>:3000
ALLOWED_HOSTS=localhost,<VPS_IP>

# Security
RATE_LIMIT_MAX=100
RATE_LIMIT_WINDOW=60000

# Monitoring
ENABLE_METRICS=true
METRICS_PORT=9090

# Paper Trading Risk Management
PAPER_TRADING_ENABLED=true
DAILY_LOSS_LIMIT_PERCENT=5.0
CONSECUTIVE_LOSS_LIMIT=5
COOL_DOWN_DURATION_MINUTES=60
MAX_POSITION_CORRELATION=0.7
SIMULATE_SLIPPAGE=true
SIMULATE_MARKET_IMPACT=true
SIMULATE_PARTIAL_FILLS=true
SIMULATE_EXECUTION_LATENCY=true
```

**Save and exit** (`:wq` in vim)

### **Step 3: Validate Environment**

```bash
# Create validation script
cat > ~/projects/bot-core/scripts/validate-env-vps.sh <<'EOF'
#!/bin/bash

echo "🔍 Validating environment configuration..."

# Required variables
REQUIRED_VARS=(
    "BINANCE_API_KEY"
    "BINANCE_SECRET_KEY"
    "OPENAI_API_KEY"
    "MONGODB_URI"
    "JWT_SECRET"
)

MISSING_VARS=()

for var in "${REQUIRED_VARS[@]}"; do
    if [ -z "${!var}" ]; then
        MISSING_VARS+=("$var")
    fi
done

if [ ${#MISSING_VARS[@]} -eq 0 ]; then
    echo "✅ All required environment variables are set"
else
    echo "❌ Missing required variables:"
    printf '%s\n' "${MISSING_VARS[@]}"
    exit 1
fi

# Check critical settings
if [ "$TRADING_ENABLED" = "true" ]; then
    echo "⚠️  WARNING: TRADING_ENABLED is true"
    echo "   Make sure you're ready for production trading!"
fi

if [ "$BINANCE_TESTNET" = "false" ]; then
    echo "⚠️  WARNING: BINANCE_TESTNET is false"
    echo "   You are using PRODUCTION Binance!"
fi

echo "✅ Environment validation complete"
EOF

chmod +x ~/projects/bot-core/scripts/validate-env-vps.sh

# Run validation
cd ~/projects/bot-core
source .env
./scripts/validate-env-vps.sh
```

---

## 🐳 Deploy Services

### **Step 1: Build Docker Images**

```bash
cd ~/projects/bot-core

# Build all services (this will take 10-15 minutes)
docker compose build --no-cache

# Or build sequentially to avoid OOM on 8GB VPS:
docker compose build rust-core-engine
docker compose build python-ai-service
docker compose build nextjs-ui-dashboard
```

**Expected output:**
```
[+] Building 542.3s (45/45) FINISHED
 => [rust-core-engine internal] load build definition
 => => transferring dockerfile: 1.23kB
 => [python-ai-service internal] load build definition
...
 => exporting to image
 => => writing image sha256:abc123...
```

### **Step 2: Start Services**

```bash
# Start all services in background
docker compose up -d

# Check running containers
docker ps
```

**Expected output:**
```
CONTAINER ID   IMAGE                        STATUS         PORTS
abc123def456   bot-core-rust-engine        Up 2 minutes   0.0.0.0:8080->8080/tcp
def456ghi789   bot-core-python-ai          Up 2 minutes   0.0.0.0:8000->8000/tcp
ghi789jkl012   bot-core-nextjs-dashboard   Up 2 minutes   0.0.0.0:3000->3000/tcp
```

### **Step 3: Monitor Service Startup**

```bash
# Watch logs from all services
docker compose logs -f

# Or watch specific service:
docker compose logs -f rust-core-engine
docker compose logs -f python-ai-service
docker compose logs -f nextjs-ui-dashboard

# Check service health
docker compose ps
```

**Wait for services to be healthy (2-5 minutes)**

---

## ✅ Verify Deployment

### **Step 1: Check Service Health**

```bash
# Rust Core Engine
curl http://localhost:8080/api/health

# Expected: {"status":"ok","timestamp":"..."}

# Python AI Service
curl http://localhost:8000/health

# Expected: {"status":"healthy","version":"1.0.0"}

# Frontend Dashboard
curl http://localhost:3000

# Expected: HTML content
```

### **Step 2: Test API Endpoints**

```bash
# Test Rust API
curl -X GET http://localhost:8080/api/strategies/active

# Test Python AI API
curl -X POST http://localhost:8000/predict \
  -H "Content-Type: application/json" \
  -d '{"symbol":"BTCUSDT","interval":"1h"}'
```

### **Step 3: Access Dashboard**

**From your local browser:**
```
http://<VPS_IP>:3000
```

**You should see the Bot Core trading dashboard.**

---

## 📊 Setup Monitoring

### **Step 1: Install Monitoring Stack**

```bash
cd ~/projects/bot-core

# Create monitoring docker-compose file
cat > docker-compose.monitoring.yml <<'EOF'
version: '3.8'

services:
  prometheus:
    image: prom/prometheus:latest
    container_name: prometheus
    ports:
      - "9090:9090"
    volumes:
      - ./monitoring/prometheus.yml:/etc/prometheus/prometheus.yml
      - prometheus_data:/prometheus
    command:
      - '--config.file=/etc/prometheus/prometheus.yml'
      - '--storage.tsdb.path=/prometheus'
    restart: unless-stopped

  grafana:
    image: grafana/grafana:latest
    container_name: grafana
    ports:
      - "3001:3000"
    volumes:
      - grafana_data:/var/lib/grafana
    environment:
      - GF_SECURITY_ADMIN_PASSWORD=admin123
    restart: unless-stopped

volumes:
  prometheus_data:
  grafana_data:
EOF

# Create Prometheus config
mkdir -p monitoring
cat > monitoring/prometheus.yml <<'EOF'
global:
  scrape_interval: 15s

scrape_configs:
  - job_name: 'rust-core-engine'
    static_configs:
      - targets: ['rust-core-engine:8080']

  - job_name: 'python-ai-service'
    static_configs:
      - targets: ['python-ai-service:8000']
EOF

# Start monitoring stack
docker compose -f docker-compose.monitoring.yml up -d
```

### **Step 2: Access Grafana**

**URL:** `http://<VPS_IP>:3001`
**Username:** admin
**Password:** admin123

---

## 💾 Backup & Disaster Recovery

### **Step 1: Automated Backup Script**

```bash
# Create backup script
cat > ~/backup-bot-core.sh <<'EOF'
#!/bin/bash

BACKUP_DIR="/var/backups/bot-core"
DATE=$(date +%Y%m%d_%H%M%S)

mkdir -p $BACKUP_DIR

# Backup MongoDB
mongodump --uri="mongodb://botcore_user:CHANGE_THIS_PASSWORD_456@localhost:27017/botcore_production" \
  --out="$BACKUP_DIR/mongodb_$DATE"

# Backup .env file
cp ~/projects/bot-core/.env "$BACKUP_DIR/env_$DATE"

# Backup logs
tar -czf "$BACKUP_DIR/logs_$DATE.tar.gz" ~/projects/bot-core/*/logs/

# Keep only last 7 days of backups
find $BACKUP_DIR -type f -mtime +7 -delete
find $BACKUP_DIR -type d -mtime +7 -delete

echo "✅ Backup completed: $DATE"
EOF

chmod +x ~/backup-bot-core.sh

# Test backup
./backup-bot-core.sh
```

### **Step 2: Setup Cron Job**

```bash
# Add daily backup at 2 AM
crontab -e
```

**Add this line:**
```
0 2 * * * /home/botadmin/backup-bot-core.sh >> /var/log/bot-backup.log 2>&1
```

---

## 🔧 Troubleshooting

### **Common Issues**

#### **1. Service Won't Start**

```bash
# Check logs
docker compose logs rust-core-engine

# Check resource usage
docker stats

# Restart service
docker compose restart rust-core-engine
```

#### **2. Out of Memory**

```bash
# Check memory usage
free -h
docker stats --no-stream

# Reduce memory limits in .env
PYTHON_MEMORY_LIMIT=1.5G
RUST_MEMORY_LIMIT=1.5G

# Restart services
docker compose down
docker compose up -d
```

#### **3. MongoDB Connection Failed**

```bash
# Check MongoDB status
sudo systemctl status mongod

# Check MongoDB logs
sudo journalctl -u mongod -n 50

# Restart MongoDB
sudo systemctl restart mongod
```

#### **4. Port Already in Use**

```bash
# Check what's using port 8080
sudo lsof -i :8080

# Kill process if needed
sudo kill -9 <PID>
```

---

## 📚 Next Steps

After successful deployment:

1. ✅ **Test với Binance Testnet** (2-3 days)
2. ✅ **Monitor performance & logs** (1 week)
3. ✅ **Optimize resource usage** (adjust limits)
4. ✅ **Setup SSL/HTTPS** (Let's Encrypt)
5. ✅ **Configure domain name** (if applicable)
6. ⚠️ **Switch to production** (only when confident)

---

## 🆘 Support

**If you need help:**
- Check logs: `docker compose logs -f`
- Check resources: `docker stats`
- Check troubleshooting guide: `/docs/TROUBLESHOOTING.md`
- GitHub Issues: https://github.com/magic-ai-trading-bot/bot-core/issues

---

**🎉 Congratulations! Bot Core is now deployed on Viettel VPS!**

Remember to keep `TRADING_ENABLED=false` and `BINANCE_TESTNET=true` until you're confident the system is working correctly.
