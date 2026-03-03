# 🎯 Bot.sh Default Services Update

## ⚡ TL;DR

**Before**: Chỉ start core services, phải thêm flags để start async jobs
**After**: Start TẤT CẢ services by default (full stack)

---

## 📋 What Changed

### Before (Old Behavior)

```bash
# Chỉ start: MongoDB, Rust, Python, Frontend
./scripts/bot.sh start

# Phải thêm flags để start async jobs
./scripts/bot.sh start --with-rabbitmq --with-redis --with-monitoring
```

**Default values**:
- `WITH_REDIS="false"`
- `WITH_RABBITMQ="false"`
- `WITH_KONG="false"`
- `WITH_MONITORING="false"`

### After (New Behavior) ✨

```bash
# Start TẤT CẢ: MongoDB, Redis, RabbitMQ, Celery, Flower, Kong, Prometheus, Grafana
./scripts/bot.sh start
```

**New default values**:
- `WITH_REDIS="true"` ✅
- `WITH_RABBITMQ="true"` ✅
- `WITH_KONG="true"` ✅
- `WITH_MONITORING="true"` ✅

---

## 🚀 Services Started By Default

### 1. Core Services (Always)
- ✅ MongoDB (port 27017)
- ✅ Rust Core Engine (port 8080)
- ✅ Python AI Service (port 8000)
- ✅ Frontend Dashboard (port 3000)

### 2. Cache (NEW Default)
- ✅ Redis (port 6379)

### 3. Async Jobs System (NEW Default)
- ✅ RabbitMQ (ports 5672, 15672)
- ✅ Celery Worker
- ✅ Celery Beat (scheduler)
- ✅ Flower (monitoring - port 5555)

### 4. API Gateway (NEW Default)
- ✅ Kong Database (PostgreSQL)
- ✅ Kong (ports 8100, 8001)

### 5. Monitoring Stack (NEW Default)
- ✅ Prometheus (port 9090)
- ✅ Grafana (port 3001)

---

## 🎯 Impact

### For Users

**Easier Usage** 👍
```bash
# Old way (phải nhớ nhiều flags)
./scripts/bot.sh start --with-redis --with-rabbitmq --with-kong --with-monitoring

# New way (chỉ cần 1 lệnh)
./scripts/bot.sh start
```

**Full Features By Default** 🎉
- Async jobs hoạt động ngay
- Monitoring sẵn sàng
- API Gateway enabled
- Redis caching available

**Single Command** ⚡
```bash
./scripts/bot.sh start --memory-optimized
```
→ Start toàn bộ hệ thống với memory optimization

### For System Resources

**Before**:
```
Core services only: ~3GB RAM
```

**After**:
```
Full stack: ~6-8GB RAM
With --memory-optimized: ~4-5GB RAM
```

**Recommendation**:
- **Minimum**: 8GB RAM
- **Optimal**: 16GB RAM
- **Always use**: `--memory-optimized` on limited resources

---

## 📊 Service URLs (All Available Now)

### Core Services
- Frontend: http://localhost:3000
- Rust API: http://localhost:8080/api/health
- Python AI: http://localhost:8000/health

### Async Jobs
- RabbitMQ Management: http://localhost:15672 (admin/admin)
- Flower (Celery Monitor): http://localhost:5555 (admin/admin)

### API Gateway
- Kong Admin: http://localhost:8001
- Kong Proxy: http://localhost:8100

### Monitoring
- Prometheus: http://localhost:9090
- Grafana: http://localhost:3001 (admin/admin)

---

## 🔄 Migration Guide

### If You Were Using Flags Before

**Old command**:
```bash
./scripts/bot.sh start --with-rabbitmq --with-monitoring
```

**New equivalent**:
```bash
./scripts/bot.sh start
```

**Nothing to change!** 🎉

### If You Want Minimal Services

Hiện tại không có cách để disable services riêng lẻ. Nếu cần minimal services:

**Option 1**: Sử dụng docker compose trực tiếp
```bash
docker compose up -d mongodb redis rust-core-engine python-ai-service frontend
```

**Option 2**: Stop services không cần
```bash
./scripts/bot.sh start
docker stop kong prometheus grafana  # Stop services không cần
```

---

## 🧪 Testing Impact

### Test Command Unchanged

```bash
# Still works the same
./scripts/bot.sh test
./scripts/bot.sh test --coverage
./scripts/bot.sh test --all
```

**Requirements**:
- Services must be running (celery-worker needed)
- Just run `./scripts/bot.sh start` first

---

## 💡 Best Practices

### Development
```bash
# Start full stack in dev mode
./scripts/bot.sh dev --memory-optimized

# Run tests
./scripts/bot.sh test --coverage

# View specific service logs
./scripts/bot.sh logs --service celery-worker
```

### Production
```bash
# Start full stack with optimization
./scripts/bot.sh start --memory-optimized

# Monitor all services
./scripts/bot.sh status

# Check logs
./scripts/bot.sh logs
```

### Troubleshooting
```bash
# Check status
./scripts/bot.sh status

# View logs for problematic service
./scripts/bot.sh logs --service <service-name>

# Restart
./scripts/bot.sh restart --memory-optimized
```

---

## 🎯 Common Commands Now

### 1. Start Everything (Simplest)
```bash
./scripts/bot.sh start
```

### 2. Start with Memory Optimization (Recommended)
```bash
./scripts/bot.sh start --memory-optimized
```

### 3. Development Mode
```bash
./scripts/bot.sh dev --memory-optimized
```

### 4. Run Tests
```bash
./scripts/bot.sh test
```

### 5. Check Status
```bash
./scripts/bot.sh status
```

### 6. View Logs
```bash
./scripts/bot.sh logs --service celery-worker
```

### 7. Restart
```bash
./scripts/bot.sh restart --memory-optimized
```

---

## ⚠️ Important Notes

### Memory Requirements

**Minimum System Requirements**:
- **RAM**: 8GB minimum (16GB recommended)
- **Disk**: 20GB free space
- **CPU**: 4 cores (2 cores minimum)

**Memory Usage**:
| Configuration | RAM Usage |
|--------------|-----------|
| Full stack (no optimization) | ~6-8GB |
| Full stack (--memory-optimized) | ~4-5GB |
| Core only (old default) | ~3GB |

**Always use `--memory-optimized`** on systems with <16GB RAM

### First Start

First start will take longer because:
1. Building Docker images (if not built)
2. Pulling base images
3. Initializing databases
4. Setting up networks

**Subsequent starts**: Much faster (~30 seconds)

### Default Credentials

**⚠️ CHANGE THESE IN PRODUCTION**

Services with default credentials:
- RabbitMQ: admin/rabbitmq_default_password
- MongoDB: bot_core_app/secure_mongo_password_change_me
- Redis: redis_default_password
- Flower: admin/admin
- Grafana: admin/admin

**How to change**: Edit `.env` file and restart

---

## 📈 Benefits

### 1. Simplified Workflow ✅
- One command to start everything
- No need to remember flags
- Consistent behavior

### 2. Full Features Available ✅
- Async jobs ready to use
- Monitoring enabled
- API gateway configured
- Redis caching active

### 3. Better Development Experience ✅
- All services available immediately
- Easy to test integrations
- Quick access to dashboards

### 4. Production-Ready ✅
- Same command for dev and prod
- All enterprise features enabled
- Monitoring out of the box

---

## 🔄 Rollback (If Needed)

Nếu cần quay lại old behavior, edit `scripts/bot.sh`:

```bash
# Change these lines back to "false"
WITH_REDIS="false"
WITH_RABBITMQ="false"
WITH_KONG="false"
WITH_MONITORING="false"
```

Nhưng **KHÔNG KHUYẾN KHÍCH** vì full stack là best practice.

---

## 📝 Summary

| Aspect | Before | After |
|--------|--------|-------|
| Default command | `./scripts/bot.sh start` | `./scripts/bot.sh start` |
| Services started | 5 core services | 15+ services (full stack) |
| Flags needed | Multiple (--with-*) | None (optional --memory-optimized) |
| Async jobs | ❌ Manual enable | ✅ Always available |
| Monitoring | ❌ Manual enable | ✅ Always available |
| API Gateway | ❌ Manual enable | ✅ Always available |
| RAM usage | ~3GB | ~4-5GB (with optimization) |
| User experience | Complex | Simple ✅ |

---

**Updated**: 2025-11-22
**Version**: 2.0 (Full Stack by Default)
**Impact**: HIGH (Simplifies workflow, ensures all features available)
