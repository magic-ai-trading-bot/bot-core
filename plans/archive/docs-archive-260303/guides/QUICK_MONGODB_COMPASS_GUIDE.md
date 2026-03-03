# 🚀 Quick MongoDB Compass Connection Guide

**TL;DR:** Copy connection string này vào MongoDB Compass và bấm Connect!

---

## 🔑 Connection String (Copy & Paste)

```
mongodb://admin:secure_mongo_password_change_me@localhost:27017/bot_core?authSource=admin
```

---

## 📱 3 Bước Kết Nối Nhanh

### **Bước 1:** Mở MongoDB Compass

### **Bước 2:** Click "New Connection"

### **Bước 3:** Paste connection string trên và click "Connect"

**DONE!** ✅

---

## 📊 Những Gì Bạn Sẽ Thấy

### **Databases:**
- `bot_core` ← Main database (chọn cái này)
- `trading_bot`
- `admin`

### **Collections trong bot_core (25 collections):**

| Collection | Số Documents | Mô Tả |
|------------|--------------|-------|
| 🤖 **ai_analysis_results** | **23** | AI/GPT-4 trading signals |
| 👥 **users** | **3** | User accounts (admin, trader, viewer) |
| 📊 **positions** | **2** | Open trading positions |
| ⚙️ **strategy_configs** | **3** | Trading strategies (RSI, MACD, BB) |
| 📈 **performance_metrics** | **2** | Daily/weekly performance |
| 🔔 **notifications** | **3** | User notifications |
| 💰 **paper_trading_accounts** | **1** | Paper trading account |

**Total:** 37+ sample documents ✅

---

## 🔍 Sample Queries to Try

### **1. View Latest AI Signals:**

Click vào `ai_analysis_results` collection → Bạn sẽ thấy 23 documents

Filter box (phía trên):
```json
{ "analysis.confidence": { "$gt": 0.6 } }
```

### **2. View Users:**

Click vào `users` collection → 3 users

Filter để ẩn password:
```json
{}
```

Chọn **Project** (bên phải):
```json
{ "password_hash": 0 }
```

### **3. View Open Positions:**

Click vào `positions` collection → 2 positions

Filter:
```json
{ "status": "open" }
```

Sort by PnL:
```json
{ "unrealized_pnl": -1 }
```

### **4. Latest AI Analysis by Symbol:**

Collection: `ai_analysis_results`

Filter:
```json
{ "symbol": "BTCUSDT" }
```

Sort:
```json
{ "timestamp": -1 }
```

---

## 👥 Sample Login Credentials

Dùng để login vào dashboard (http://localhost:3000):

### **Admin:**
- Email: `admin@botcore.com`
- Password: `password123`

### **Trader:**
- Email: `trader@botcore.com`
- Password: `password123`

### **Viewer:**
- Email: `viewer@botcore.com`
- Password: `password123`

---

## 📊 Top Collections to Explore

### **1. ai_analysis_results (23 docs)** ⭐ MUST SEE!

**What:** GPT-4 AI trading signals tự động phân tích

**Sample data:**
- Symbol: BTCUSDT, ETHUSDT, etc.
- Signal: BUY/SELL/Neutral
- Confidence: 0.0-1.0
- Strategy scores: RSI, MACD, Volume, Bollinger Bands
- Risk assessment

**Try this query:**
```javascript
db.ai_analysis_results.find().sort({ timestamp: -1 }).limit(5)
```

---

### **2. users (3 docs)**

**What:** User accounts cho testing

**Fields:**
- email (unique)
- role (admin/trader/viewer)
- settings (theme, notifications, default_symbol)

**Try this query:**
```javascript
db.users.find({}, { password_hash: 0 })
```

---

### **3. positions (2 docs)**

**What:** Open trading positions

**Fields:**
- symbol, side (LONG/SHORT)
- entry_price, current_price
- unrealized_pnl, unrealized_pnl_percent
- stop_loss, take_profit
- strategy

**Try this query:**
```javascript
db.positions.find({ unrealized_pnl: { $gt: 0 } })
```

---

### **4. strategy_configs (3 docs)**

**What:** Trading strategy configurations

**Strategies:**
1. RSI Oversold Strategy (enabled)
2. MACD Crossover Strategy (enabled)
3. Bollinger Bands Strategy (disabled)

**Try this query:**
```javascript
db.strategy_configs.find({ enabled: true })
```

---

## 💡 Pro Tips

### **Tip 1: Use Aggregation Builder**

Click **Aggregations** tab để build complex queries visually.

### **Tip 2: Export Data**

Click **Export Collection** (top right) → Save as JSON or CSV

### **Tip 3: Schema Tab**

Click **Schema** tab để xem data structure và types

### **Tip 4: Explain Plan**

Click **Explain Plan** để xem query performance

---

## ⚠️ Troubleshooting

### **Issue: "Connection Refused"**

**Solution:**
```bash
# Check if MongoDB container is running
docker ps | grep mongodb

# If not, start it
docker-compose up -d mongodb
```

---

### **Issue: "Authentication Failed"**

**Solution:** Đảm bảo password là `secure_mongo_password_change_me`

Connection string đúng:
```
mongodb://admin:secure_mongo_password_change_me@localhost:27017/bot_core?authSource=admin
```

---

### **Issue: "Cannot see collections"**

**Solution:**
1. Click vào database `bot_core` (không phải `admin`)
2. Đợi 2-3 giây cho collections load
3. Scroll down để xem tất cả 25 collections

---

## 🎯 Quick Actions Checklist

- [ ] Open MongoDB Compass
- [ ] Paste connection string
- [ ] Click "Connect"
- [ ] Select `bot_core` database
- [ ] Click `ai_analysis_results` collection
- [ ] Sort by `timestamp` descending
- [ ] See 23 AI signals! 🎉

---

## 📚 Full Documentation

**Detailed guides:**
- `MONGODB_COMPASS_CONNECTION_GUIDE.md` - Comprehensive connection guide
- `SEED_DATA_GUIDE.md` - Complete seed data documentation

---

**That's it! Bây giờ bạn có thể xem toàn bộ data trong MongoDB!** 🚀

Có gì thắc mắc cứ hỏi nhé! 😊
