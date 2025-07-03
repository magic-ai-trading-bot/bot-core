# 🐳 Docker Desktop Optimization Guide

> **Giải pháp cho vấn đề Docker Desktop crash do thiếu RAM**

## ⚠️ Vấn đề thường gặp

**Triệu chứng:**

- Docker Desktop crash với lỗi "Internal Virtualization error"
- Lỗi "Virtual machine stopped unexpectedly"
- Build process bị dừng đột ngột
- Máy treo khi build nhiều service cùng lúc

**Nguyên nhân:**

- Docker Desktop không đủ RAM được cấp phát
- Build 3 services (Rust + Python ML + Node.js) cùng lúc tốn nhiều memory
- Cấu hình Docker Desktop không phù hợp với project

## 🔧 Cấu hình Docker Desktop

### **1. Tăng RAM cho Docker Desktop**

#### **macOS:**

1. Mở **Docker Desktop**
2. Vào **Settings** (⚙️) → **Resources** → **Advanced**
3. Cấu hình khuyến nghị:
   ```
   Memory: 8GB (tối thiểu) → 12GB+ (khuyến nghị)
   CPUs: 4 cores (tối thiểu) → 6-8 cores (khuyến nghị)
   Disk: 50GB (tối thiểu) → 100GB+ (khuyến nghị)
   Swap: 2GB
   ```

#### **Windows:**

1. Mở **Docker Desktop**
2. Vào **Settings** → **Resources** → **Advanced**
3. Cấu hình tương tự như macOS

#### **Linux:**

Docker trên Linux không có giới hạn memory như Desktop version, nhưng vẫn nên monitor:

```bash
# Check Docker resource usage
docker system df
docker stats
```

### **2. Restart Docker Desktop**

Sau khi thay đổi cấu hình:

1. **Apply & Restart** Docker Desktop
2. Đợi Docker khởi động hoàn toàn (icon chuyển xanh)
3. Kiểm tra với: `docker info`

## 🚀 Sử dụng Build Strategy Tối Ưu

### **Option 1: Sử dụng Build Script (Khuyến nghị)**

```bash
# Build từng service một, tránh overload memory
make build-fast

# Hoặc với clean cache
make build-clean

# Build và start luôn
chmod +x scripts/build-services.sh
./scripts/build-services.sh
```

### **Option 2: Sử dụng Memory-Optimized Compose**

```bash
# Start với resource limits
make start-memory

# Hoặc manual
docker-compose -f docker-compose.memory-optimized.yml up -d
```

### **Option 3: Build từng service riêng biệt**

```bash
# Build theo thứ tự từ nặng đến nhẹ
make build-python    # Python ML (nặng nhất)
make build-rust      # Rust (trung bình)
make build-frontend  # Node.js (nhẹ nhất)
```

## 📊 Monitoring & Troubleshooting

### **1. Kiểm tra Docker Resource Usage**

```bash
# Check Docker system info
docker system info

# Check memory usage
docker system df

# Monitor real-time resource usage
docker stats

# Check running containers
docker ps -a
```

### **2. Cleanup Commands**

```bash
# Clean up unused containers
docker container prune -f

# Clean up unused images
docker image prune -f

# Clean up build cache
docker builder prune -f

# Clean up everything (careful!)
docker system prune -a -f
```

### **3. Debug Build Issues**

```bash
# Build with verbose output
docker-compose build --no-cache --progress=plain

# Check build logs
docker-compose logs python-ai-service
docker-compose logs rust-core-engine
docker-compose logs nextjs-ui-dashboard
```

## 🛠️ Resource Limits trong Docker Compose

Project này đã được tối ưu với resource limits:

```yaml
# docker-compose.memory-optimized.yml
services:
  python-ai-service:
    deploy:
      resources:
        limits:
          memory: 2G # Tối đa 2GB RAM
          cpus: "2" # Tối đa 2 CPU cores
        reservations:
          memory: 1G # Đặt trước 1GB RAM
          cpus: "1" # Đặt trước 1 CPU core

  rust-core-engine:
    deploy:
      resources:
        limits:
          memory: 1G # Tối đa 1GB RAM
          cpus: "2" # Tối đa 2 CPU cores
        reservations:
          memory: 512M # Đặt trước 512MB RAM
          cpus: "1" # Đặt trước 1 CPU core

  nextjs-ui-dashboard:
    deploy:
      resources:
        limits:
          memory: 512M # Tối đa 512MB RAM
          cpus: "1" # Tối đa 1 CPU core
        reservations:
          memory: 256M # Đặt trước 256MB RAM
          cpus: "0.5" # Đặt trước 0.5 CPU core
```

## 🎯 Các Tình huống Cụ thể

### **Tình huống 1: Máy có RAM < 16GB**

```bash
# Sử dụng memory-optimized build
make build-clean
make start-memory

# Hoặc build từng service một
make build-python
docker container prune -f
make build-rust
docker container prune -f
make build-frontend
```

### **Tình huống 2: Docker Desktop bị crash**

```bash
# 1. Restart Docker Desktop
# 2. Tăng RAM allocation trong Settings
# 3. Clean up tất cả
make clean-all

# 4. Build lại với strategy tối ưu
make build-fast
```

### **Tình huống 3: Build bị stuck**

```bash
# Stop tất cả containers
docker-compose down

# Kill tất cả processes
docker kill $(docker ps -q)

# Clean up
docker container prune -f
docker image prune -f

# Build lại
make build-fast
```

## 🔍 Monitoring Commands

### **Check Docker Health**

```bash
# Check Docker daemon status
docker info

# Check allocated memory
docker system info | grep -i memory

# Check Docker Desktop resource usage
docker system df -v

# Monitor container resource usage
docker stats --no-stream
```

### **Check Build Progress**

```bash
# Follow build logs
docker-compose build --progress=plain

# Check individual service build
docker-compose build python-ai-service --progress=plain
```

## ⚡ Quick Commands

```bash
# Emergency cleanup
make clean-all

# Quick build và start
make build-fast

# Build với memory optimization
make start-memory

# Check health
make health

# Show service URLs
make urls
```

## 🆘 Troubleshooting Checklist

- [ ] Docker Desktop có đủ RAM? (8GB+ khuyến nghị)
- [ ] Docker Desktop Settings đã Apply & Restart?
- [ ] Đã clean up containers cũ? (`make clean-all`)
- [ ] Có dùng memory-optimized compose file?
- [ ] Có build từng service riêng biệt?
- [ ] Đã check Docker daemon status? (`docker info`)
- [ ] Có monitor resource usage? (`docker stats`)

## 📞 Support

Nếu vẫn gặp vấn đề:

1. Chụp screenshot lỗi
2. Chạy `docker system info > docker-info.txt`
3. Chạy `docker stats --no-stream > docker-stats.txt`
4. Gửi kèm 2 file trên để debug

---

**💡 Tip:** Luôn dùng `make build-fast` thay vì `docker-compose build` trực tiếp để tránh memory issues!
