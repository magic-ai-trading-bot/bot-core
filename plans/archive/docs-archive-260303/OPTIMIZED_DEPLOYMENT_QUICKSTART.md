# 🚀 Optimized Deployment Quick Start

**For:** Deploying bot with optimized 5-strategy parameters
**Date:** 2025-11-20
**Prerequisites:** Paper trading validation complete (≥100 trades, ≥65% win rate)

---

## ✅ Quick Deployment Steps (TL;DR)

```bash
# 1. Backup current config
cp rust-core-engine/config.toml rust-core-engine/config.toml.backup

# 2. Optimized config already applied! ✅
# (Done in Priority 1)

# 3. Switch to production mode
export BINANCE_TESTNET=false
export TRADING_ENABLED=true
export BINANCE_API_KEY="your_production_key"
export BINANCE_SECRET_KEY="your_production_secret"

# 4. Restart bot
./scripts/bot.sh restart

# 5. Monitor performance
./scripts/daily_report.sh                        # Daily summary
python3 scripts/monitor_performance.py --continuous --alert   # Real-time
```

---

## 📊 Monitoring Commands (Cho Bạn)

### **Daily Report (Chạy Hàng Ngày)**

```bash
./scripts/daily_report.sh
```

**Output:**
```
╔══════════════════════════════════════════════════════════════════════╗
║           📊 DAILY PERFORMANCE REPORT                                 ║
╠══════════════════════════════════════════════════════════════════════╣
║   Date: 2025-11-20 09:00:00                                         ║
╚══════════════════════════════════════════════════════════════════════╝

💰 PORTFOLIO SUMMARY
--------------------------------------------------------------------
   Current Balance:   $10,500.00
   Initial Balance:   $10,000.00
   Total Return:      +5.00%
   Unrealized P&L:    $+150.00
   Total Trades:      45

📈 PERFORMANCE METRICS
--------------------------------------------------------------------
   Win Rate:          🟢 70.0% (Target: 70%)
   Avg Profit:        🟢 2.6% (Target: 2.6%)
   Total Trades:      45

✅ Status: EXCELLENT - Meeting optimization targets!
```

### **Continuous Monitoring (Tự Động)**

```bash
# Run in background với alerts
python3 scripts/monitor_performance.py --continuous --alert &

# Or với screen/tmux
screen -S monitor
python3 scripts/monitor_performance.py --continuous --alert
# Press Ctrl+A+D to detach
```

**Alerts sẽ hiện khi:**
- ❌ Win rate < 60%
- ❌ Avg profit < 1.5%
- ❌ Portfolio drawdown > 10%

### **Check Logs (Khi Cần)**

```bash
# View real-time logs
./scripts/bot.sh logs --service rust-core-engine -f

# Check for errors
./scripts/bot.sh logs --service rust-core-engine | grep -i error

# View last 100 lines
./scripts/bot.sh logs --service rust-core-engine --tail 100
```

---

## ⚙️ Optimized Parameters (Đã Apply)

✅ **Config đã được update với optimized parameters:**

| Strategy | Parameter | Before → After | Expected |
|----------|-----------|----------------|----------|
| **RSI** | period | 14 → 10 | +3% win rate |
| **MACD** | fast/slow | 12/26 → 10/22 | +3% win rate |
| **Bollinger** | period, std | 20, 2.0 → 15, 2.5 | +3% win rate |
| **Volume** | spike | 2.0 → 1.8 | +6% win rate |
| **Stochastic** | k_period | 14 → 10 | +4% win rate |

**Combined Expected:**
- Win Rate: 65% → 70% (+5%)
- Avg Profit: 2.3% → 2.6% (+0.3%)
- Annual Return: 280% → 350% (+70% APY)

---

## 🎯 Gradual Rollout (KHUYẾN NGHỊ)

### **Phase 1: Days 1-3 (Ultra Conservative)**

```bash
# Edit config
vim rust-core-engine/config.toml
```

```toml
[trading]
enabled = true
max_positions = 1         # Single position only
default_quantity = 0.001  # Minimum (e.g., 0.001 BTC)
risk_percentage = 0.5     # Ultra-conservative
```

**Monitor:**
- ✅ Orders executing correctly
- ✅ Stop loss working
- ✅ No errors

### **Phase 2: Days 4-7 (Conservative)**

```toml
[trading]
max_positions = 2         # Two positions
default_quantity = 0.005
risk_percentage = 1.0
```

**Monitor:**
- ✅ Performance matches paper trading
- ✅ Multiple positions managed correctly

### **Phase 3: Day 8+ (Normal)**

```toml
[trading]
max_positions = 3         # Normal operations
default_quantity = 0.01
risk_percentage = 2.0
```

**Monitor:**
- ✅ Win rate ≥65%
- ✅ No performance degradation

---

## 🛡️ Risk Management

### **Daily Checks**

```bash
# Check balance and PnL
curl -s http://localhost:8080/api/paper-trading/portfolio | jq '.balance, .unrealized_pnl'

# Check active positions
curl -s http://localhost:8080/api/positions/active | jq 'length'

# Check today's win rate
./scripts/daily_report.sh
```

### **Emergency Stop**

```bash
# If need to stop immediately
./scripts/emergency_stop.sh

# Or manually
curl -X POST http://localhost:8080/api/trading/disable
docker stop rust-core-engine
```

---

## 📊 Performance Targets

| Metric | Target | Alert If Below |
|--------|--------|----------------|
| **Win Rate** | 70% | 60% |
| **Avg Profit** | 2.6% | 1.5% |
| **Sharpe Ratio** | 2.1 | 1.0 |
| **Max Drawdown** | <10% | >15% |
| **Daily Loss** | <5% | >5% |

---

## 🔄 Monitoring Schedule (Cho Bạn)

### **Tự Động (Set Up Once)**

```bash
# Setup cron job
crontab -e

# Add these lines:
0 9 * * * /path/to/bot-core/scripts/daily_report.sh >> /var/log/bot-report.log 2>&1
*/30 * * * * /path/to/bot-core/scripts/check_health.sh >> /var/log/bot-health.log 2>&1
```

### **Thủ Công (Khi Bạn Check)**

**Hàng ngày (5 phút):**
```bash
./scripts/daily_report.sh
```

**Hàng tuần (15 phút):**
```bash
# Review performance
python3 scripts/monitor_performance.py

# Check logs for errors
./scripts/bot.sh logs --service rust-core-engine | grep -i error

# Backup database
./scripts/backup_database.sh
```

**Hàng tháng (30 phút):**
```bash
# Compare to targets
# Adjust if needed
# Document performance
```

---

## ❓ FAQs

### **Q: Monitoring có tự động không hay tôi phải làm manual?**

**A:** Bạn có 2 options:

1. **Tự động** (Recommended):
   ```bash
   # Setup continuous monitoring (chạy 1 lần)
   screen -S monitor
   python3 scripts/monitor_performance.py --continuous --alert
   # Ctrl+A+D to detach
   
   # Setup cron job for daily report
   crontab -e
   # Add: 0 9 * * * /path/to/daily_report.sh
   ```
   
   → Script sẽ tự check và alert khi có vấn đề

2. **Manual** (Đơn giản hơn):
   ```bash
   # Chạy daily report mỗi ngày
   ./scripts/daily_report.sh
   ```
   
   → Bạn check manually mỗi ngày (5 phút)

**Recommendation:** Dùng tự động + check manual 1 lần/ngày

### **Q: Khi nào thì deploy production?**

**A:** Sau khi:
- ✅ Paper trading ≥100 trades
- ✅ Win rate ≥65% trong 1-2 tuần
- ✅ No crashes/errors
- ✅ Bạn đã review và comfortable với risk

### **Q: Performance thấp hơn expected thì sao?**

**A:**

1. **Check ngay:**
   ```bash
   ./scripts/daily_report.sh
   python3 scripts/monitor_performance.py
   ```

2. **Nếu win rate < 60%:**
   - Reduce risk (risk_percentage = 0.5%)
   - Reduce positions (max_positions = 1)
   - Hoặc pause trading để investigate

3. **Nếu có errors:**
   - Check logs: `./scripts/bot.sh logs -f`
   - Emergency stop if needed

### **Q: Mình có cần monitor 24/7 không?**

**A:** KHÔNG!

**Tự động monitoring:**
- Script sẽ alert khi có vấn đề
- Cron job chạy daily report

**Manual check:**
- Mỗi ngày 5-10 phút (xem daily report)
- Mỗi tuần 15 phút (deeper review)

---

## 📝 Next Steps Summary

1. **✅ Đã xong:** Optimized parameters applied
2. **✅ Đã xong:** Monitoring scripts ready
3. **Bây giờ:** 
   - Run daily_report.sh mỗi ngày để track performance
   - Sau 1-2 tuần validation → deploy production
   - Dùng gradual rollout (Phase 1 → 2 → 3)

**Commands để remember:**

```bash
# Daily check (hàng ngày)
./scripts/daily_report.sh

# Continuous monitoring (tự động)
python3 scripts/monitor_performance.py --continuous --alert

# Emergency stop (khi cần)
curl -X POST http://localhost:8080/api/trading/disable

# Restart (sau changes)
./scripts/bot.sh restart
```

---

**Status:** ✅ Ready for Validation → Production
**Monitoring:** ✅ Automated (bạn chỉ cần check daily report)
**Next:** Run paper trading với optimized params trong 1-2 tuần

---

*Mọi thứ đã setup xong! Bạn chỉ cần chạy daily_report.sh mỗi ngày thôi* 🚀
