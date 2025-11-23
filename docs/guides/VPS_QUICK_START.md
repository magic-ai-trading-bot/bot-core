# Viettel VPS Quick Start Guide

**Deployment Time:** 30 minutes
**Difficulty:** Beginner-Friendly
**For:** Viettel IDC T2.GEN 03 (8 vCPU / 8 GB RAM / 100 GB SSD)

---

## 🚀 Quick Deployment (30 Minutes)

### **Prerequisites**
- ✅ VPS Viettel đã mua và nhận thông tin SSH
- ✅ Binance testnet API keys
- ✅ OpenAI API key

---

## Step 1: Connect to VPS (2 minutes)

```bash
# From your local machine
ssh root@<VPS_IP>

# Enter password when prompted
```

---

## Step 2: Run Auto-Setup Script (20 minutes)

```bash
# Download and run setup script
curl -fsSL https://raw.githubusercontent.com/YOUR_REPO/bot-core/main/scripts/vps-auto-setup.sh | bash
```

**Script will automatically:**
1. ✅ Update system packages
2. ✅ Install Docker & Docker Compose
3. ✅ Install MongoDB
4. ✅ Setup firewall (UFW)
5. ✅ Clone bot-core repository
6. ✅ Generate secrets
7. ✅ Create .env file template

---

## Step 3: Configure Environment (5 minutes)

```bash
cd ~/projects/bot-core

# Edit .env file
nano .env
```

**Update these CRITICAL values:**
```bash
BINANCE_API_KEY=your_testnet_api_key
BINANCE_SECRET_KEY=your_testnet_secret
OPENAI_API_KEY=sk-your_openai_key
MONGODB_URI=mongodb://botcore_user:PASSWORD@localhost:27017/botcore_production

# ⚠️ KEEP THESE AS IS:
TRADING_ENABLED=false
BINANCE_TESTNET=true
```

**Save:** `Ctrl + X`, then `Y`, then `Enter`

---

## Step 4: Deploy Services (3 minutes)

```bash
# Build and start all services
docker compose up -d

# Check status
docker compose ps
```

**Expected output:**
```
NAME                          STATUS
bot-core-rust-engine          Up
bot-core-python-ai           Up
bot-core-frontend            Up
```

---

## Step 5: Verify Deployment (1 minute)

```bash
# Test health endpoints
curl http://localhost:8080/api/health
curl http://localhost:8000/health

# Should see: {"status":"ok"}
```

**Access Dashboard:**
```
http://<VPS_IP>:3000
```

---

## ✅ Done! What's Next?

### **Monitor for 24-48 hours:**
```bash
# Watch logs
docker compose logs -f

# Check resource usage
docker stats
```

### **After testing successfully:**
1. Keep running on testnet for 1 week
2. Monitor all trades and performance
3. Only then consider production (with caution)

---

## 🆘 Need Help?

**Quick Fixes:**

**Services won't start?**
```bash
docker compose logs rust-core-engine
docker compose restart
```

**Out of memory?**
```bash
free -h
docker stats
# Reduce limits in .env and restart
```

**Connection refused?**
```bash
sudo ufw status
sudo ufw allow 3000/tcp
```

---

## 📊 Monitoring Commands

```bash
# View logs
docker compose logs -f

# Check health
curl http://localhost:8080/api/health

# Resource usage
docker stats --no-stream

# Restart service
docker compose restart rust-core-engine

# Full restart
docker compose down && docker compose up -d
```

---

## 🛑 Emergency Stop

```bash
# Stop all services
docker compose down

# Stop and remove all data
docker compose down -v
```

---

**For detailed guide, see:** [VIETTEL_VPS_DEPLOYMENT_GUIDE.md](./VIETTEL_VPS_DEPLOYMENT_GUIDE.md)
