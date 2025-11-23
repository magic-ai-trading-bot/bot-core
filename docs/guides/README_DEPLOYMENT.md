# Bot Core - Deployment Guides

Comprehensive deployment documentation for Bot Core trading platform.

---

## 📚 Available Guides

### **🚀 Quick Start (Recommended for Beginners)**
- **[VPS_QUICK_START.md](./VPS_QUICK_START.md)** - 30-minute quick deployment
  - For: First-time deployment to Viettel VPS
  - Time: ~30 minutes
  - Difficulty: Beginner
  - Automated setup script included

### **📖 Comprehensive Guide**
- **[VIETTEL_VPS_DEPLOYMENT_GUIDE.md](./VIETTEL_VPS_DEPLOYMENT_GUIDE.md)** - Complete deployment manual
  - For: Detailed step-by-step deployment
  - Time: 2-3 hours
  - Difficulty: Intermediate
  - Covers all aspects: security, monitoring, backup, troubleshooting

### **🏢 Production Deployment**
- **[PRODUCTION_DEPLOYMENT_GUIDE.md](../PRODUCTION_DEPLOYMENT_GUIDE.md)** - Enterprise-grade deployment
  - For: Large-scale production deployment
  - Covers: Cloud platforms, Kubernetes, high availability
  - Difficulty: Advanced

---

## 🎯 Which Guide Should I Use?

### **I'm deploying to Viettel VPS for the first time**
→ Start with **[VPS_QUICK_START.md](./VPS_QUICK_START.md)**
- Fastest way to get started
- Automated setup script
- Beginner-friendly

### **I want complete control and understanding**
→ Use **[VIETTEL_VPS_DEPLOYMENT_GUIDE.md](./VIETTEL_VPS_DEPLOYMENT_GUIDE.md)**
- Manual step-by-step process
- Learn exactly what's happening
- Better for customization

### **I'm deploying to AWS/GCP/Azure or Kubernetes**
→ Use **[PRODUCTION_DEPLOYMENT_GUIDE.md](../PRODUCTION_DEPLOYMENT_GUIDE.md)**
- Cloud-native deployment
- High availability setup
- Enterprise features

---

## 🛠️ Deployment Scripts

### **Automated Setup (VPS)**
```bash
# Download and run auto-setup script
curl -fsSL https://raw.githubusercontent.com/YOUR_REPO/bot-core/main/scripts/vps-auto-setup.sh | sudo bash
```

### **Manual Deployment (Local to VPS)**
```bash
# Use deployment script
./scripts/deploy-to-viettel-vps.sh
```

---

## 📋 Pre-Deployment Checklist

Before deploying, ensure you have:

### **Required Accounts & Keys**
- [ ] Viettel VPS purchased (T2.GEN 03 recommended)
- [ ] SSH access to VPS (IP, username, password/key)
- [ ] Binance account with API keys (testnet for testing)
- [ ] OpenAI account with API key (GPT-4 access)
- [ ] Domain name (optional, for custom URL)

### **Required Software (Local Machine)**
- [ ] Git installed
- [ ] SSH client installed
- [ ] Text editor (for configuration files)

### **Recommended Knowledge**
- [ ] Basic Linux command line
- [ ] Basic Docker concepts (helpful but not required)
- [ ] Understanding of API keys and environment variables

---

## 🌟 Deployment Comparison

| Feature | Quick Start | VPS Guide | Production Guide |
|---------|-------------|-----------|------------------|
| **Time** | 30 min | 2-3 hours | 1+ days |
| **Automation** | High | Medium | Low |
| **Customization** | Low | High | Very High |
| **Learning** | Minimal | Good | Extensive |
| **Difficulty** | ⭐ Easy | ⭐⭐ Medium | ⭐⭐⭐ Hard |
| **Best For** | Testing | Production VPS | Enterprise |

---

## 📖 Deployment Flow

### **Step-by-Step Process**

```
1. Choose Your Guide
   ↓
2. Purchase Viettel VPS (T2.GEN 03)
   ↓
3. Prepare API Keys
   ├─ Binance Testnet API
   ├─ OpenAI API Key
   └─ MongoDB (auto-installed or cloud)
   ↓
4. Run Deployment
   ├─ Auto: vps-auto-setup.sh
   └─ Manual: Follow guide step-by-step
   ↓
5. Configure Environment
   ├─ Edit .env file
   ├─ Set API keys
   └─ Configure trading settings
   ↓
6. Deploy Services
   └─ docker compose up -d
   ↓
7. Verify Deployment
   ├─ Check health endpoints
   ├─ Test API calls
   └─ Access dashboard
   ↓
8. Monitor & Test
   ├─ Run on testnet (1 week)
   ├─ Monitor logs
   └─ Fix any issues
   ↓
9. Production Ready! 🎉
```

---

## 🔐 Security Best Practices

### **CRITICAL Security Steps**

1. **Always Start with Testnet**
   ```bash
   TRADING_ENABLED=false
   BINANCE_TESTNET=true
   ```

2. **Generate Strong Secrets**
   ```bash
   openssl rand -base64 64  # JWT_SECRET
   openssl rand -base64 32  # SESSION_SECRET
   ```

3. **Configure Firewall**
   ```bash
   sudo ufw enable
   sudo ufw allow 22,80,443,3000,8000,8080/tcp
   ```

4. **Use SSH Keys** (not passwords)
   ```bash
   ssh-keygen -t ed25519
   ssh-copy-id user@vps-ip
   ```

5. **Regular Backups**
   ```bash
   # Automated backup script included
   ./backup-bot-core.sh
   ```

---

## 📊 Post-Deployment

### **After Successful Deployment**

1. **Monitor for 24-48 hours**
   ```bash
   docker compose logs -f
   docker stats
   ```

2. **Test with Small Amounts**
   - Start with testnet (virtual money)
   - Test all features thoroughly
   - Monitor for 1 week minimum

3. **Setup Monitoring**
   - Grafana dashboard (http://VPS_IP:3001)
   - Email/Telegram alerts
   - Log aggregation

4. **Regular Maintenance**
   - Daily log review
   - Weekly backups verification
   - Monthly security updates

---

## 🆘 Support & Troubleshooting

### **Common Issues**

**Services won't start?**
```bash
docker compose logs
docker compose restart
```

**Out of memory?**
```bash
docker stats
# Adjust limits in .env
```

**Can't connect to dashboard?**
```bash
sudo ufw status
sudo ufw allow 3000/tcp
```

**MongoDB connection failed?**
```bash
sudo systemctl status mongod
sudo systemctl restart mongod
```

### **Get Help**

- 📖 **Troubleshooting Guide**: [TROUBLESHOOTING.md](../TROUBLESHOOTING.md)
- 🐛 **GitHub Issues**: https://github.com/magic-ai-trading-bot/bot-core/issues
- 📧 **Email Support**: support@botcore.io (if available)

---

## 🎓 Learning Resources

### **Recommended Reading**

1. **Before Deployment**
   - Docker basics
   - Linux command line basics
   - API authentication concepts

2. **During Deployment**
   - Follow guide step-by-step
   - Don't skip security steps
   - Test each component

3. **After Deployment**
   - Monitor logs regularly
   - Understand trading strategies
   - Learn risk management

### **Video Tutorials** (Coming Soon)
- Quick Start Deployment
- Security Hardening
- Trading Strategy Configuration
- Monitoring & Alerts Setup

---

## 📞 Quick Links

| Resource | Link |
|----------|------|
| **Quick Start** | [VPS_QUICK_START.md](./VPS_QUICK_START.md) |
| **Full VPS Guide** | [VIETTEL_VPS_DEPLOYMENT_GUIDE.md](./VIETTEL_VPS_DEPLOYMENT_GUIDE.md) |
| **Production Guide** | [PRODUCTION_DEPLOYMENT_GUIDE.md](../PRODUCTION_DEPLOYMENT_GUIDE.md) |
| **Troubleshooting** | [TROUBLESHOOTING.md](../TROUBLESHOOTING.md) |
| **Main README** | [README.md](../../README.md) |

---

## ⚡ TL;DR - Fastest Deployment

```bash
# 1. SSH to VPS
ssh root@YOUR_VPS_IP

# 2. Run auto-setup
curl -fsSL https://raw.githubusercontent.com/YOUR_REPO/bot-core/main/scripts/vps-auto-setup.sh | sudo bash

# 3. Switch to bot user
su - botadmin

# 4. Edit .env (add API keys)
cd ~/projects/bot-core
nano .env

# 5. Deploy
docker compose up -d

# 6. Access dashboard
# http://YOUR_VPS_IP:3000
```

**Done in 30 minutes! 🚀**

---

**Remember**: Always start with `TRADING_ENABLED=false` and `BINANCE_TESTNET=true` until fully tested!
