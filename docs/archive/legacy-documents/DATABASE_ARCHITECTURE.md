# Database Architecture - MongoDB Only

## 🎯 Quyết Định Kiến Trúc

Bot Core sử dụng **MongoDB làm database duy nhất** cho tất cả data storage needs.

## 📊 Tại Sao Chỉ MongoDB?

### 1. **Phù Hợp Với Trading Data**
- Time-series data cho market candles
- Flexible schema cho different order types
- High write performance cho real-time updates
- Native support cho geospatial queries (multi-region)

### 2. **Đơn Giản Hóa Architecture**
- Một database system = ít complexity
- Không cần sync giữa 2 databases
- Giảm chi phí infrastructure
- Dễ backup và restore

### 3. **MongoDB Đủ Mạnh**
- ACID transactions từ version 4.0+
- Change streams cho real-time events
- Aggregation pipeline cho analytics
- Time-series collections cho market data

## 🗄️ Data Organization

### Collections
```javascript
// Market Data (Time-series)
db.market_data = {
  symbol: "BTCUSDT",
  timestamp: ISODate(),
  open: 50000,
  high: 51000,
  low: 49500,
  close: 50500,
  volume: 1000
}

// Trading History
db.trades = {
  _id: ObjectId(),
  user_id: "user123",
  symbol: "BTCUSDT",
  side: "BUY",
  price: 50000,
  quantity: 0.1,
  status: "FILLED",
  timestamp: ISODate()
}

// User Portfolios
db.portfolios = {
  user_id: "user123",
  balances: {
    USDT: 10000,
    BTC: 0.5,
    ETH: 10
  },
  positions: [...],
  updated_at: ISODate()
}

// AI Predictions
db.ai_predictions = {
  symbol: "BTCUSDT",
  model: "LSTM",
  prediction: "BUY",
  confidence: 0.85,
  features: {...},
  timestamp: ISODate()
}
```

### Indexes
```javascript
// Performance optimization
db.market_data.createIndex({ symbol: 1, timestamp: -1 })
db.trades.createIndex({ user_id: 1, timestamp: -1 })
db.trades.createIndex({ symbol: 1, status: 1 })
db.ai_predictions.createIndex({ symbol: 1, timestamp: -1 })
```

## 🔄 MongoDB Replica Set (Optional)

Nếu cần high availability:
```yaml
# docker-compose.replicas.yml (simplified)
mongodb-primary:
  image: mongo:7.0
  command: mongod --replSet rs0

mongodb-secondary:
  image: mongo:7.0
  command: mongod --replSet rs0

mongodb-arbiter:
  image: mongo:7.0
  command: mongod --replSet rs0 --arbiter
```

## 💡 Best Practices

### 1. **Connection String**
```javascript
// Use connection pooling
mongodb+srv://username:password@cluster.mongodb.net/trading_bot?
  retryWrites=true&
  w=majority&
  maxPoolSize=100
```

### 2. **Backup Strategy**
```bash
# Daily backups
mongodump --uri=$DATABASE_URL --gzip --archive=backup.gz

# Point-in-time recovery
mongorestore --uri=$DATABASE_URL --gzip --archive=backup.gz
```

### 3. **Monitoring**
- Use MongoDB Atlas for managed hosting
- Enable performance monitoring
- Set up alerts for slow queries

## ❌ Removed PostgreSQL

PostgreSQL đã được remove vì:
- Over-engineering cho trading bot
- MongoDB đủ cho financial data
- Giảm complexity và cost
- Không có use case thực sự cần SQL

## 📈 Scalability Path

1. **Vertical Scaling**: Upgrade MongoDB instance
2. **Horizontal Scaling**: Add read replicas
3. **Sharding**: Shard by user_id hoặc symbol
4. **Atlas**: Migrate to MongoDB Atlas for auto-scaling

## 🎯 Conclusion

MongoDB-only architecture cung cấp:
- ✅ Simplicity
- ✅ Performance
- ✅ Flexibility
- ✅ Cost-effectiveness
- ✅ Sufficient features cho trading bot

Không cần PostgreSQL cho use case này!