# 🚀 Bot.sh Script - Hướng Dẫn Đầy Đủ

## Tổng Quan

Script `scripts/bot.sh` là công cụ chính để quản lý toàn bộ hệ thống Crypto Trading Bot.

**Syntax**:
```bash
./scripts/bot.sh [COMMAND] [OPTIONS]
```

---

## 📋 Danh Sách Commands (11 Commands)

### 1. `start` - Khởi động hệ thống (Production Mode)

**Chức năng**:
- Khởi động tất cả services ở chế độ production
- Sử dụng Dockerfile production
- Tự động seed MongoDB data (lần đầu tiên)
- Hiển thị service URLs sau khi start

**Cú pháp**:
```bash
./scripts/bot.sh start [OPTIONS]
```

**Ví dụ**:
```bash
# Start core services only (MongoDB, Redis, Rust, Python, Frontend)
./scripts/bot.sh start

# Start với async jobs (RabbitMQ, Celery, Flower)
./scripts/bot.sh start --with-rabbitmq

# Start tất cả enterprise features
./scripts/bot.sh start --with-enterprise

# Start với memory optimization
./scripts/bot.sh start --memory-optimized

# Start với multiple features
./scripts/bot.sh start --with-rabbitmq --with-monitoring --memory-optimized
```

**Services được start**:
- **Core** (luôn start):
  - MongoDB (database)
  - Redis (cache)
  - Rust Core Engine (port 8080)
  - Python AI Service (port 8000)
  - Frontend Dashboard (port 3000)

- **Với --with-rabbitmq**:
  - RabbitMQ (ports 5672, 15672)
  - Celery Worker
  - Celery Beat
  - Flower (port 5555)

- **Với --with-enterprise** (tất cả):
  - All core services
  - RabbitMQ + Celery + Flower
  - Kong API Gateway (ports 8100, 8001)
  - Prometheus (port 9090)
  - Grafana (port 3001)

---

### 2. `dev` - Khởi động ở Development Mode

**Chức năng**:
- Khởi động services với hot reload
- Sử dụng Dockerfile.dev
- Log level = DEBUG
- Node.js memory tăng lên 768MB

**Cú pháp**:
```bash
./scripts/bot.sh dev [OPTIONS]
```

**Ví dụ**:
```bash
# Dev mode cơ bản
./scripts/bot.sh dev

# Dev mode với async jobs
./scripts/bot.sh dev --with-rabbitmq

# Dev mode với memory optimization
./scripts/bot.sh dev --memory-optimized
```

**Khác biệt với `start`**:
| Feature | start | dev |
|---------|-------|-----|
| Dockerfile | Dockerfile | Dockerfile.dev |
| Log Level | INFO | DEBUG |
| Hot Reload | ❌ | ✅ |
| Node Memory | 512MB | 768MB |
| Rust Log | info | debug |

---

### 3. `stop` - Dừng tất cả services

**Chức năng**:
- Dừng tất cả containers đang chạy
- Xóa orphan containers
- Giữ nguyên volumes (data không bị mất)

**Cú pháp**:
```bash
./scripts/bot.sh stop
```

**Ví dụ**:
```bash
./scripts/bot.sh stop
```

**Lưu ý**:
- Data trong MongoDB, Redis vẫn được giữ
- Có thể start lại bất cứ lúc nào

---

### 4. `restart` - Khởi động lại hệ thống

**Chức năng**:
- Dừng tất cả services
- Sau đó khởi động lại với cùng cấu hình

**Cú pháp**:
```bash
./scripts/bot.sh restart [OPTIONS]
```

**Ví dụ**:
```bash
# Restart basic
./scripts/bot.sh restart

# Restart với async jobs
./scripts/bot.sh restart --with-rabbitmq

# Restart với enterprise features
./scripts/bot.sh restart --with-enterprise
```

**Khi nào dùng**:
- Sau khi thay đổi .env
- Sau khi update code
- Khi services bị lỗi

---

### 5. `build` - Build lại Docker images

**Chức năng**:
- Build lại Docker images cho services
- Có thể build toàn bộ hoặc từng service riêng lẻ
- Sử dụng cache để tăng tốc

**Cú pháp**:
```bash
./scripts/bot.sh build [OPTIONS]
```

**Ví dụ**:
```bash
# Build tất cả services
./scripts/bot.sh build

# Build specific service
./scripts/bot.sh build --service python-ai-service

# Build trong dev mode
./scripts/bot.sh dev build

# Build và bypass cache (clean build)
docker compose build --no-cache
```

**Khi nào cần build**:
- Sau khi thay đổi Dockerfile
- Sau khi thay đổi dependencies (requirements.txt, Cargo.toml, package.json)
- Khi muốn update base images

---

### 6. `test` - Chạy Test Suite ✨ (NEW)

**Chức năng**:
- Chạy automated tests cho async tasks
- Hỗ trợ coverage report
- Có thể chạy simplified hoặc comprehensive tests

**Cú pháp**:
```bash
./scripts/bot.sh test [OPTIONS]
```

**Ví dụ**:
```bash
# Test đơn giản (24 tests, 100% pass) - RECOMMENDED
./scripts/bot.sh test

# Test với coverage report
./scripts/bot.sh test --coverage

# Test tất cả test files (138 tests)
./scripts/bot.sh test --all

# Test tất cả + coverage
./scripts/bot.sh test --all --coverage
```

**Output**:
```
============================== 24 passed in 2.50s ==============================
✅ All tests passed!
```

**Coverage report location**:
```
python-ai-service/htmlcov/index.html
```

**Test files**:
- `test_async_tasks_simple.py` (24 tests) - Default
- `test_monitoring_tasks.py` (21 tests) - Với --all
- `test_ai_improvement_tasks.py` (23 tests) - Với --all
- `test_notifications.py` (24 tests) - Với --all
- `test_data_storage.py` (24 tests) - Với --all
- `test_celery_integration.py` (22 tests) - Với --all

**Requirements**:
- Celery-worker container phải đang chạy
- Start services trước: `./scripts/bot.sh start --with-rabbitmq`

---

### 7. `status` - Hiển thị trạng thái hệ thống

**Chức năng**:
- Hiển thị status của tất cả containers
- Hiển thị resource usage (CPU, Memory)
- Kiểm tra service health

**Cú pháp**:
```bash
./scripts/bot.sh status
```

**Output**:
```
Service status:
NAME              STATUS
mongodb           Up 10 minutes (healthy)
redis             Up 10 minutes (healthy)
rabbitmq          Up 10 minutes (healthy)
celery-worker     Up 5 minutes (unhealthy)
celery-beat       Up 5 minutes (unhealthy)
flower            Up 5 minutes (unhealthy)
rust-core-engine  Up 10 minutes (healthy)
python-ai-service Up 10 minutes (healthy)
frontend          Up 10 minutes (healthy)

Resource usage:
NAME              MEM USAGE / LIMIT    MEM %    CPU %
mongodb           245.2MiB / 512MiB    47.89%   1.23%
redis             15.4MiB / 256MiB     6.01%    0.45%
rabbitmq          156.8MiB / 512MiB    30.63%   2.34%
...
```

---

### 8. `logs` - Xem logs của services

**Chức năng**:
- Hiển thị logs của tất cả services hoặc specific service
- Theo dõi logs real-time (tail -f)
- Hỗ trợ filtering

**Cú pháp**:
```bash
./scripts/bot.sh logs [OPTIONS]
```

**Ví dụ**:
```bash
# Xem logs tất cả services
./scripts/bot.sh logs

# Xem logs specific service
./scripts/bot.sh logs --service python-ai-service

# Xem logs của Rust Core Engine
./scripts/bot.sh logs --service rust-core-engine

# Xem logs của Celery Worker
./scripts/bot.sh logs --service celery-worker

# Xem logs của RabbitMQ
./scripts/bot.sh logs --service rabbitmq
```

**Tips**:
- Press `Ctrl+C` để thoát
- Logs sẽ scroll real-time
- Có thể grep logs: `./scripts/bot.sh logs --service celery-worker | grep ERROR`

---

### 9. `clean` - Dọn dẹp hệ thống

**Chức năng**:
- Dừng và xóa tất cả containers
- Xóa tất cả volumes (⚠️ DATA SẼ MẤT)
- Xóa unused images
- Giải phóng disk space

**Cú pháp**:
```bash
./scripts/bot.sh clean
```

**Interactive confirmation**:
```
⚠️  This will remove all containers, images, and volumes. Are you sure? (y/N)
```

**⚠️ CẢNH BÁO**:
- Sẽ XÓA TẤT CẢ DATA trong MongoDB, Redis
- Không thể khôi phục
- Chỉ dùng khi muốn reset hoàn toàn

**Khi nào dùng**:
- Muốn reset hệ thống về trạng thái ban đầu
- Troubleshooting các vấn đề nghiêm trọng
- Giải phóng disk space

---

### 10. `verify` - Kiểm tra cấu hình hệ thống

**Chức năng**:
- Verify các prerequisites (Docker, Docker Compose)
- Kiểm tra .env configuration
- Verify secrets và API keys
- Test connectivity

**Cú pháp**:
```bash
./scripts/bot.sh verify
```

**Script được chạy**:
```bash
./scripts/verify-setup.sh
```

**Checks performed**:
- ✅ Docker installed
- ✅ Docker Compose installed
- ✅ .env file exists
- ✅ Required environment variables set
- ✅ MongoDB connection
- ✅ Redis connection
- ✅ Network connectivity

---

### 11. `help` - Hiển thị help message

**Chức năng**:
- Hiển thị usage instructions
- List tất cả commands và options
- Hiển thị examples

**Cú pháp**:
```bash
./scripts/bot.sh help
```

**Hoặc**:
```bash
./scripts/bot.sh
```

---

## ⚙️ Options (9 Options)

### 1. `--memory-optimized`

**Chức năng**: Sử dụng memory-optimized settings

**Resource limits**:
```bash
PYTHON_MEMORY_LIMIT="1.5G"    # Default: 2G
PYTHON_CPU_LIMIT="1.5"        # Default: 2
RUST_MEMORY_LIMIT="1G"        # Default: 2G
RUST_CPU_LIMIT="1"            # Default: 2
FRONTEND_MEMORY_LIMIT="512M"  # Default: 1G
FRONTEND_CPU_LIMIT="0.5"      # Default: 1
NODE_MEMORY="512"             # Default: 768
```

**Khi nào dùng**:
- RAM < 16GB
- Muốn chạy nhiều services khác
- Cloud instances với limited resources

**Ví dụ**:
```bash
./scripts/bot.sh start --memory-optimized
./scripts/bot.sh dev --memory-optimized --with-rabbitmq
```

---

### 2. `--with-enterprise`

**Chức năng**: Start TẤT CẢ enterprise features

**Bao gồm**:
- Redis cache
- RabbitMQ + Celery + Flower (messaging)
- Kong API Gateway
- Prometheus + Grafana (monitoring)

**Equivalent to**:
```bash
--with-redis --with-rabbitmq --with-kong --with-monitoring
```

**Ví dụ**:
```bash
./scripts/bot.sh start --with-enterprise
```

---

### 3. `--with-redis`

**Chức năng**: Start Redis cache

**Services**:
- Redis (port 6379)

**Ví dụ**:
```bash
./scripts/bot.sh start --with-redis
```

---

### 4. `--with-rabbitmq`

**Chức năng**: Start async job processing system

**Services**:
- RabbitMQ (ports 5672, 15672)
- Celery Worker
- Celery Beat
- Flower (port 5555)

**Ví dụ**:
```bash
./scripts/bot.sh start --with-rabbitmq
```

**⭐ RECOMMENDED cho async tasks**

---

### 5. `--with-kong`

**Chức năng**: Start Kong API Gateway

**Services**:
- Kong Database (PostgreSQL)
- Kong (ports 8100, 8001)

**Ví dụ**:
```bash
./scripts/bot.sh start --with-kong
```

---

### 6. `--with-monitoring`

**Chức năng**: Start monitoring stack

**Services**:
- Prometheus (port 9090)
- Grafana (port 3001)

**Ví dụ**:
```bash
./scripts/bot.sh start --with-monitoring
```

---

### 7. `--service SERVICE`

**Chức năng**: Target specific service

**Áp dụng cho**: `build`, `logs`

**Ví dụ**:
```bash
# Build specific service
./scripts/bot.sh build --service python-ai-service

# View logs của specific service
./scripts/bot.sh logs --service celery-worker
```

**Available services**:
- mongodb
- redis
- rabbitmq
- celery-worker
- celery-beat
- flower
- rust-core-engine
- python-ai-service
- frontend
- kong
- prometheus
- grafana

---

### 8. `--coverage`

**Chức năng**: Generate test coverage report

**Áp dụng cho**: `test` command only

**Output**:
- Terminal: Coverage summary
- HTML: `python-ai-service/htmlcov/index.html`

**Ví dụ**:
```bash
./scripts/bot.sh test --coverage
```

---

### 9. `--all`

**Chức năng**: Run ALL test files (138 tests)

**Áp dụng cho**: `test` command only

**Test files**:
- test_async_tasks_simple.py (24 tests)
- test_monitoring_tasks.py (21 tests)
- test_ai_improvement_tasks.py (23 tests)
- test_notifications.py (24 tests)
- test_data_storage.py (24 tests)
- test_celery_integration.py (22 tests)

**Ví dụ**:
```bash
./scripts/bot.sh test --all
./scripts/bot.sh test --all --coverage
```

---

## 🎯 Common Workflows

### 1. First Time Setup

```bash
# 1. Verify prerequisites
./scripts/bot.sh verify

# 2. Start core services
./scripts/bot.sh start --memory-optimized

# 3. Check status
./scripts/bot.sh status
```

### 2. Development Workflow

```bash
# 1. Start dev mode với async jobs
./scripts/bot.sh dev --with-rabbitmq

# 2. Make code changes...

# 3. View logs
./scripts/bot.sh logs --service python-ai-service

# 4. Run tests
./scripts/bot.sh test

# 5. Restart if needed
./scripts/bot.sh restart --with-rabbitmq
```

### 3. Production Deployment

```bash
# 1. Build production images
./scripts/bot.sh build

# 2. Start with all features
./scripts/bot.sh start --with-enterprise --memory-optimized

# 3. Verify all services healthy
./scripts/bot.sh status

# 4. Run comprehensive tests
./scripts/bot.sh test --all --coverage

# 5. Monitor logs
./scripts/bot.sh logs
```

### 4. Testing Async Jobs

```bash
# 1. Start services with messaging
./scripts/bot.sh start --with-rabbitmq

# 2. Run tests
./scripts/bot.sh test

# 3. Check Flower dashboard
open http://localhost:5555

# 4. Check RabbitMQ management
open http://localhost:15672
```

### 5. Troubleshooting

```bash
# 1. Check status
./scripts/bot.sh status

# 2. View logs
./scripts/bot.sh logs --service <problematic-service>

# 3. Restart
./scripts/bot.sh restart --with-rabbitmq

# 4. If still issues, clean rebuild
./scripts/bot.sh clean
./scripts/bot.sh build
./scripts/bot.sh start --with-rabbitmq
```

---

## 📊 Service URLs Reference

### Core Services
- **Frontend Dashboard**: http://localhost:3000
- **Rust Core API**: http://localhost:8080/api/health
- **Python AI API**: http://localhost:8000/health
- **MongoDB**: mongodb://localhost:27017

### Enterprise Features
- **RabbitMQ Management**: http://localhost:15672 (admin/admin)
- **Flower (Celery)**: http://localhost:5555 (admin/admin)
- **Kong Admin API**: http://localhost:8001
- **Kong Proxy**: http://localhost:8100
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3001 (admin/admin)

---

## 🔒 Security Notes

### Default Credentials

**⚠️ CHANGE THESE IN PRODUCTION**

```bash
# RabbitMQ
Username: admin
Password: rabbitmq_default_password

# MongoDB
Username: bot_core_app
Password: secure_mongo_password_change_me

# Redis
Password: redis_default_password

# Flower
Username: admin
Password: admin

# Grafana
Username: admin
Password: admin
```

### How to Change

Edit `.env` file:
```bash
RABBITMQ_USER=your_user
RABBITMQ_PASSWORD=your_strong_password
MONGO_ROOT_PASSWORD=your_mongo_password
REDIS_PASSWORD=your_redis_password
FLOWER_USER=your_flower_user
FLOWER_PASSWORD=your_flower_password
```

Then restart:
```bash
./scripts/bot.sh restart --with-enterprise
```

---

## 🚨 Important Notes

### Prerequisites
- Docker 20.10+
- Docker Compose 2.0+
- 16GB RAM (minimum 8GB với --memory-optimized)
- 20GB free disk space

### Data Persistence
- MongoDB data: `mongodb_data` volume
- Redis data: `redis_data` volume
- RabbitMQ data: `rabbitmq_data` volume

**⚠️ Only `clean` command deletes volumes**

### Performance Tips
1. Use `--memory-optimized` on limited resources
2. Use `dev` mode only for development
3. Use `--with-rabbitmq` instead of `--with-enterprise` if you don't need monitoring
4. Monitor resource usage with `./scripts/bot.sh status`

---

## 📖 Quick Reference

| Command | Purpose | Example |
|---------|---------|---------|
| `start` | Start services (production) | `./scripts/bot.sh start --with-rabbitmq` |
| `dev` | Start services (development) | `./scripts/bot.sh dev --memory-optimized` |
| `stop` | Stop all services | `./scripts/bot.sh stop` |
| `restart` | Restart services | `./scripts/bot.sh restart --with-enterprise` |
| `build` | Build Docker images | `./scripts/bot.sh build --service python-ai-service` |
| `test` | Run tests | `./scripts/bot.sh test --coverage` |
| `status` | Check service status | `./scripts/bot.sh status` |
| `logs` | View logs | `./scripts/bot.sh logs --service celery-worker` |
| `clean` | Clean everything | `./scripts/bot.sh clean` |
| `verify` | Verify setup | `./scripts/bot.sh verify` |
| `help` | Show help | `./scripts/bot.sh help` |

---

**Last Updated**: 2025-11-22
**Version**: 2.0 (với test command)
