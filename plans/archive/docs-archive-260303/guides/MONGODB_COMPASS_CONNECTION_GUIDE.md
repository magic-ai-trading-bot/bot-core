# 🔌 Kết Nối MongoDB Compass

**Date:** 2025-11-19
**MongoDB Version:** 7.0.25
**Status:** ✅ Running in Docker

---

## 📊 Connection Information

### **MongoDB Container Details:**

- **Container Name:** `mongodb`
- **Port:** `27017` (exposed to localhost)
- **Image:** `mongo:7.0`
- **Status:** ✅ Healthy

### **Databases Created:**

1. **bot_core** - Main application database
2. **trading_bot** - Trading data database
3. **admin** - Administrative database

### **Users Created:**

MongoDB đã tạo 3 users với quyền khác nhau:

1. **Admin User** (Full access)
   - Username: `bot_core_admin`
   - Database: `bot_core`
   - Quyền: Read/Write/Admin

2. **App User** (Application access)
   - Username: `bot_core_app`
   - Database: `bot_core`
   - Quyền: Read/Write

3. **Read-only User** (View only)
   - Username: `bot_core_readonly`
   - Database: `bot_core`
   - Quyền: Read only

---

## 🔑 Connection Strings

### **Option 1: Admin User (Recommended for development)**

```
mongodb://bot_core_admin:<password>@localhost:27017/bot_core?authSource=admin
```

**Trong MongoDB Compass:**
- **Host:** `localhost`
- **Port:** `27017`
- **Authentication:** Username/Password
- **Username:** `bot_core_admin`
- **Password:** Xem bên dưới cách lấy password
- **Authentication Database:** `admin`
- **Database:** `bot_core`

---

### **Option 2: Read-only User (An toàn cho production)**

```
mongodb://bot_core_readonly:<password>@localhost:27017/bot_core?authSource=admin
```

**Trong MongoDB Compass:**
- **Host:** `localhost`
- **Port:** `27017`
- **Username:** `bot_core_readonly`
- **Password:** Xem bên dưới
- **Authentication Database:** `admin`
- **Database:** `bot_core`

---

## 🔐 Lấy Password

### **Cách 1: Từ Docker Logs**

```bash
docker logs mongodb 2>&1 | grep "password" | head -5
```

Hoặc kiểm tra file init script:

```bash
cat infrastructure/docker/mongodb/init/01-init-users.sh
```

### **Cách 2: Từ Environment Variables**

```bash
grep MONGO .env 2>/dev/null || grep MONGO .env.example
```

### **Cách 3: Reset Password (Nếu quên)**

```bash
# Stop MongoDB
docker stop mongodb

# Start without auth
docker run --rm -v bot-core_mongodb_data:/data/db -p 27017:27017 mongo:7.0

# Connect và reset password
docker exec -it mongodb mongosh
> use admin
> db.changeUserPassword("bot_core_admin", "new_password")
```

---

## 📱 Hướng Dẫn Kết Nối MongoDB Compass

### **Bước 1: Mở MongoDB Compass**

Mở ứng dụng MongoDB Compass trên máy của bạn.

### **Bước 2: Chọn "New Connection"**

Click vào nút **"New Connection"** ở góc trên bên trái.

### **Bước 3: Nhập Connection String**

**Option A: Dùng Connection String (Nhanh nhất)**

Paste connection string này vào:

```
mongodb://bot_core_admin:YOUR_PASSWORD@localhost:27017/bot_core?authSource=admin
```

Thay `YOUR_PASSWORD` bằng password thực tế.

**Option B: Điền Form Thủ Công**

Hoặc điền form:

| Field | Value |
|-------|-------|
| **Connection Name** | Bot Core Trading (tùy ý) |
| **Host** | `localhost` |
| **Port** | `27017` |
| **Authentication** | Username/Password |
| **Username** | `bot_core_admin` |
| **Password** | (password từ logs) |
| **Authentication Database** | `admin` |
| **Default Database** | `bot_core` |

### **Bước 4: Test Connection**

Click **"Connect"** để test kết nối.

Nếu thành công, bạn sẽ thấy:
- ✅ Databases: `bot_core`, `trading_bot`, `admin`
- ✅ Collections trong `bot_core`: `users`, `ai_analysis_results`, etc.

---

## 🗄️ Collections Có Thể Xem

Sau khi kết nối, bạn có thể xem các collections sau:

### **Database: bot_core**

1. **ai_analysis_results** - Kết quả phân tích AI/GPT-4
   - Chứa trading signals
   - Market analysis
   - Timestamps

2. **users** - User accounts
   - Email (unique index)
   - Hashed passwords
   - Created timestamps

3. **positions** - Trading positions
   - Open/closed positions
   - Entry/exit prices
   - PnL data

4. **trades** - Trade history
   - Trade details
   - Performance metrics

### **Database: trading_bot**

1. **users** - Alternative user storage
2. **market_data** - Historical market data
3. **strategies** - Strategy configurations

---

## 🔍 Example Queries

Sau khi kết nối, bạn có thể chạy các queries này trong MongoDB Compass:

### **1. Xem AI Analysis gần nhất:**

```javascript
db.ai_analysis_results.find().sort({ timestamp: -1 }).limit(10)
```

### **2. Xem Users:**

```javascript
db.users.find({}, { password: 0 }) // Exclude password field
```

### **3. Xem Positions đang mở:**

```javascript
db.positions.find({ status: "open" })
```

### **4. Xem Trade history:**

```javascript
db.trades.find().sort({ timestamp: -1 }).limit(20)
```

---

## ⚙️ Troubleshooting

### **Issue 1: Connection Refused**

```
Error: connect ECONNREFUSED 127.0.0.1:27017
```

**Solution:**

```bash
# Check if MongoDB container is running
docker ps | grep mongodb

# If not running, start it
docker-compose up -d mongodb
```

---

### **Issue 2: Authentication Failed**

```
Error: Authentication failed
```

**Solutions:**

1. **Check password đúng chưa:**

```bash
docker logs mongodb 2>&1 | grep "password"
```

2. **Check authentication database:**

Đảm bảo set `authSource=admin` trong connection string.

3. **Thử user khác:**

Thử với `bot_core_readonly` (read-only access).

---

### **Issue 3: Cannot see databases**

**Solution:**

1. Check authentication database is `admin`
2. Check user có quyền trên database chưa
3. Try với admin user thay vì read-only

---

## 🛡️ Security Best Practices

### **Development:**
- ✅ Dùng `bot_core_admin` để test
- ✅ Keep password trong `.env` (không commit)

### **Production:**
- ⚠️ KHÔNG expose port 27017 ra internet
- ⚠️ Dùng `bot_core_readonly` để view data
- ⚠️ Rotate passwords định kỳ
- ⚠️ Enable MongoDB authentication
- ⚠️ Use MongoDB Atlas cho production

---

## 📊 Expected Data

Sau khi services chạy một lúc, bạn sẽ thấy data:

### **ai_analysis_results Collection:**

```json
{
    "_id": ObjectId("..."),
    "symbol": "BTCUSDT",
    "timestamp": ISODate("2025-11-19T08:00:00.000Z"),
    "signal": "BUY",
    "confidence": 0.85,
    "reasoning": "Strong bullish indicators...",
    "technical_indicators": {
        "rsi": 45.2,
        "macd": 125.3,
        "bb_upper": 45000,
        "bb_lower": 43000
    },
    "created_at": ISODate("2025-11-19T08:00:00.000Z")
}
```

### **users Collection:**

```json
{
    "_id": ObjectId("..."),
    "email": "user@example.com",
    "password_hash": "$2b$12$...",
    "created_at": ISODate("2025-11-19T07:49:42.000Z"),
    "role": "trader"
}
```

---

## ✅ Quick Start Checklist

- [ ] MongoDB container đang chạy (`docker ps | grep mongodb`)
- [ ] Lấy password từ logs hoặc init script
- [ ] Mở MongoDB Compass
- [ ] Paste connection string: `mongodb://bot_core_admin:<pass>@localhost:27017/bot_core?authSource=admin`
- [ ] Click "Connect"
- [ ] Browse databases: `bot_core`, `trading_bot`
- [ ] Xem collections: `ai_analysis_results`, `users`, etc.

---

## 🔗 Connection String Template

**Copy và thay `<password>`:**

```
mongodb://bot_core_admin:<password>@localhost:27017/bot_core?authSource=admin
```

**Example với fake password:**

```
mongodb://bot_core_admin:mySecurePass123@localhost:27017/bot_core?authSource=admin
```

---

## 📝 Notes

1. **Port 27017** đã được expose ra `localhost:27017` trong docker-compose
2. **MongoDB 7.0.25** đang chạy
3. **3 databases** được tạo: `bot_core`, `trading_bot`, `admin`
4. **3 users** với quyền khác nhau
5. **Collections** sẽ tự động tạo khi có data

---

## 🎯 Recommended Connection

**Cho Development:**

```
mongodb://bot_core_admin:<password>@localhost:27017/bot_core?authSource=admin
```

**Quyền:** Full Read/Write/Admin

**Use case:**
- ✅ Development và testing
- ✅ Insert/Update/Delete data
- ✅ Create indexes
- ✅ Debug issues

---

**Happy MongoDB browsing!** 🎉

Nếu có vấn đề, check `docker logs mongodb` để xem chi tiết errors.
