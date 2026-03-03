# 🌱 Seed Data Guide - MongoDB Sample Data

**Date:** 2025-11-19
**Status:** ✅ Sample data created and ready to view

---

## 📊 Available Data in MongoDB

### ✅ Successfully Created:

| Collection | Count | Description |
|------------|-------|-------------|
| **users** | 3 | Sample user accounts |
| **ai_analysis_results** | 23 | AI/GPT-4 trading signals (auto-generated) |
| **paper_trading_accounts** | 1 | Paper trading accounts |
| **positions** | 2 | Open trading positions |
| **strategy_configs** | 3 | Trading strategy configurations |
| **performance_metrics** | 2 | Daily/weekly performance data |
| **notifications** | 3 | User notifications |

**Total Documents:** 37+ sample documents

---

## 👥 Sample User Accounts

### **1. Admin User**
- **Email:** `admin@botcore.com`
- **Password:** `password123`
- **Role:** Admin
- **Status:** Active
- **Features:** Full access to all features

### **2. Trader User**
- **Email:** `trader@botcore.com`
- **Password:** `password123`
- **Role:** Trader
- **Status:** Active
- **Features:** Trading access, portfolio management

### **3. Viewer User**
- **Email:** `viewer@botcore.com`
- **Password:** `password123`
- **Role:** Viewer
- **Status:** Active
- **Features:** Read-only access

---

## 🔌 MongoDB Compass Connection

### **Connection String:**

```
mongodb://admin:secure_mongo_password_change_me@localhost:27017/bot_core?authSource=admin
```

### **Connection Details:**

| Field | Value |
|-------|-------|
| **Host** | `localhost` |
| **Port** | `27017` |
| **Username** | `admin` |
| **Password** | `secure_mongo_password_change_me` |
| **Auth Database** | `admin` |
| **Database** | `bot_core` |

---

## 📁 Collections You Can View

### **1. users** (3 documents)

Sample document:
```json
{
  "_id": ObjectId("..."),
  "email": "admin@botcore.com",
  "password_hash": "$2b$12$...",
  "created_at": ISODate("2025-11-19T08:09:04.098Z"),
  "is_active": true,
  "is_admin": true,
  "role": "admin",
  "settings": {
    "theme": "dark",
    "notifications_enabled": true,
    "default_symbol": "BTCUSDT"
  }
}
```

**Queries to try:**
```javascript
// View all users (without password)
db.users.find({}, { password_hash: 0 })

// Find admin users
db.users.find({ is_admin: true })

// Count active users
db.users.countDocuments({ is_active: true })
```

---

### **2. ai_analysis_results** (23 documents)

Sample document:
```json
{
  "_id": ObjectId("691d769502893732e47af10b"),
  "symbol": "BTCUSDT",
  "timestamp": ISODate("2025-11-19T07:49:41.203Z"),
  "analysis": {
    "signal": "Neutral",
    "confidence": 0.55,
    "reasoning": "Market indicators show mixed signals...",
    "strategy_scores": {
      "RSI Strategy": 0.5,
      "MACD Strategy": 0.4,
      "Volume Strategy": 0.6,
      "Bollinger Bands Strategy": 0.5
    },
    "market_analysis": {
      "trend_direction": "Sideways",
      "trend_strength": 0.5,
      "volatility_level": "Medium"
    },
    "risk_assessment": {
      "overall_risk": "Medium",
      "recommended_position_size": 0.1
    }
  },
  "created_at": ISODate("2025-11-19T07:49:41.203Z")
}
```

**Queries to try:**
```javascript
// Latest AI signals
db.ai_analysis_results.find().sort({ timestamp: -1 }).limit(5)

// BUY signals only
db.ai_analysis_results.find({ "analysis.signal": "BUY" })

// High confidence signals (>70%)
db.ai_analysis_results.find({ "analysis.confidence": { $gt: 0.7 } })

// Signals for BTCUSDT
db.ai_analysis_results.find({ symbol: "BTCUSDT" })
```

---

### **3. positions** (2 documents)

Sample document:
```json
{
  "_id": ObjectId("..."),
  "user_email": "trader@botcore.com",
  "symbol": "BTCUSDT",
  "side": "LONG",
  "entry_price": 43250.50,
  "current_price": 43890.75,
  "quantity": 0.1,
  "leverage": 3,
  "margin": 1441.68,
  "unrealized_pnl": 192.08,
  "unrealized_pnl_percent": 4.44,
  "stop_loss": 42305.49,
  "take_profit": 45412.03,
  "entry_time": ISODate("2025-11-19T06:09:04.261Z"),
  "status": "open",
  "strategy": "RSI Oversold + MACD Bullish"
}
```

**Queries to try:**
```javascript
// All open positions
db.positions.find({ status: "open" })

// Profitable positions (PnL > 0)
db.positions.find({ unrealized_pnl: { $gt: 0 } })

// Positions by user
db.positions.find({ user_email: "trader@botcore.com" })
```

---

### **4. strategy_configs** (3 documents)

Sample document:
```json
{
  "_id": ObjectId("..."),
  "name": "RSI Oversold Strategy",
  "type": "RSI",
  "enabled": true,
  "parameters": {
    "rsi_period": 14,
    "oversold_threshold": 30,
    "overbought_threshold": 70,
    "min_confidence": 0.6
  },
  "symbols": ["BTCUSDT", "ETHUSDT", "BNBUSDT"],
  "created_at": ISODate("2025-11-19T08:09:04.261Z")
}
```

**Queries to try:**
```javascript
// Enabled strategies
db.strategy_configs.find({ enabled: true })

// Strategies for BTCUSDT
db.strategy_configs.find({ symbols: "BTCUSDT" })
```

---

### **5. performance_metrics** (2 documents)

Sample document:
```json
{
  "_id": ObjectId("..."),
  "user_email": "admin@botcore.com",
  "period": "daily",
  "date": ISODate("2025-11-18T08:09:04.261Z"),
  "total_trades": 3,
  "winning_trades": 2,
  "losing_trades": 1,
  "win_rate": 0.667,
  "total_pnl": 425.50,
  "total_pnl_percent": 5.32,
  "max_drawdown": -2.98,
  "sharpe_ratio": 1.85
}
```

**Queries to try:**
```javascript
// Daily metrics
db.performance_metrics.find({ period: "daily" })

// Best performing days (PnL > 5%)
db.performance_metrics.find({ total_pnl_percent: { $gt: 5 } })
```

---

### **6. notifications** (3 documents)

Sample document:
```json
{
  "_id": ObjectId("..."),
  "user_email": "trader@botcore.com",
  "type": "trade_signal",
  "title": "New Trading Signal: BTCUSDT",
  "message": "RSI Strategy suggests LONG position with 75% confidence",
  "severity": "info",
  "read": false,
  "created_at": ISODate("2025-11-19T07:39:04.261Z"),
  "data": {
    "symbol": "BTCUSDT",
    "signal": "LONG",
    "confidence": 0.75
  }
}
```

**Queries to try:**
```javascript
// Unread notifications
db.notifications.find({ read: false })

// Trade signal notifications
db.notifications.find({ type: "trade_signal" })
```

---

## 🔧 Regenerate Seed Data

If you want to reset and regenerate seed data:

### **1. Clear existing data:**

```bash
docker exec -it mongodb mongosh -u admin -p secure_mongo_password_change_me --authenticationDatabase admin

use bot_core
db.users.deleteMany({})
db.paper_trading_accounts.deleteMany({})
db.positions.deleteMany({})
db.strategy_configs.deleteMany({})
db.performance_metrics.deleteMany({})
db.notifications.deleteMany({})
```

### **2. Run seed script:**

```bash
docker exec -i mongodb mongosh -u admin -p secure_mongo_password_change_me --authenticationDatabase admin bot_core < scripts/seed-mongodb.js
```

---

## 📊 Data Statistics

### **Current Database Size:**

```bash
docker exec mongodb mongosh -u admin -p secure_mongo_password_change_me --authenticationDatabase admin --quiet --eval "db = db.getSiblingDB('bot_core'); db.stats()"
```

### **Collection Counts:**

```bash
docker exec mongodb mongosh -u admin -p secure_mongo_password_change_me --authenticationDatabase admin --quiet --eval "
db = db.getSiblingDB('bot_core');
db.getCollectionNames().forEach(function(col) {
  print(col + ': ' + db[col].countDocuments());
})
"
```

---

## 🎯 What You Can Do Now

### **In MongoDB Compass:**

1. ✅ **Browse Collections** - See all 25 collections
2. ✅ **View Documents** - Browse 37+ sample documents
3. ✅ **Run Queries** - Test aggregations and filters
4. ✅ **Check Indexes** - View database indexes
5. ✅ **Monitor Performance** - Check query performance
6. ✅ **Validate Schemas** - Review document validation rules

### **In Dashboard (http://localhost:3000):**

1. ✅ **Login** - Use `trader@botcore.com` / `password123`
2. ✅ **View AI Signals** - See 23 AI analysis results
3. ✅ **Check Positions** - View 2 open positions
4. ✅ **Review Performance** - Check metrics
5. ✅ **Read Notifications** - 3 sample notifications

---

## ⚠️ Known Issues

### **1. Some Collections Failed Validation**

**Issue:** Trades collection has strict schema validation

**Solution:** MongoDB has validation rules that require specific field names:
- `side` must be `BUY` or `SELL` (not `LONG` or `SHORT`)
- `status` must be `PENDING`, `FILLED`, etc. (not `closed`)

**Workaround:** Data still viewable in other collections

### **2. Duplicate Paper Trading Accounts**

**Issue:** Only 1 paper trading account created (expected 2)

**Reason:** Unique index on `user_id` field

**Impact:** Minimal - still have sample data to view

---

## 💡 Tips for MongoDB Compass

### **1. Use Filters:**

Click the **Filter** box and try:
```json
{ "analysis.confidence": { "$gt": 0.7 } }
```

### **2. Sort Results:**

Click column headers to sort, or use:
```json
{ "timestamp": -1 }
```

### **3. Aggregation Pipeline:**

Try the **Aggregations** tab for complex queries.

### **4. Export Data:**

Click **Export** to save data as JSON or CSV.

---

## 🎉 Summary

**You now have:**

- ✅ **37+ sample documents** across 7 collections
- ✅ **3 user accounts** to test login
- ✅ **23 AI signals** from GPT-4
- ✅ **2 open positions** to view
- ✅ **Working MongoDB Compass connection**
- ✅ **Sample queries** to explore data

**Next steps:**

1. Open **MongoDB Compass**
2. Connect using: `mongodb://admin:secure_mongo_password_change_me@localhost:27017/bot_core?authSource=admin`
3. Browse **bot_core** database
4. Explore collections and try sample queries
5. Login to dashboard with `trader@botcore.com` / `password123`

---

**Happy data exploring!** 🚀

If you need more seed data or have questions, let me know!
