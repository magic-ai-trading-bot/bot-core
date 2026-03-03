# 🛑 Hướng Dẫn Stop Services - Bot Core

**Quick Reference:** Các cách để stop services hoàn toàn

---

## ⚡ CÁCH NHANH NHẤT (Khuyến nghị)

### Stop tất cả services:
```bash
cd /Users/dungngo97/Documents/bot-core

# Method 1: Script (recommended)
./scripts/bot.sh stop

# Method 2: Docker Compose
docker-compose down

# Method 3: Force stop all
docker stop $(docker ps -q)
```

---

## 🔍 STOP TỪNG BƯỚC (Chi tiết)

### Bước 1: Stop containers

```bash
# List running containers
docker ps

# Stop all bot-core containers
docker stop rust-core-engine-dev python-ai-service-dev nextjs-ui-dashboard-dev
```

### Bước 2: Cleanup networks

```bash
# Remove docker compose networks
docker-compose down --remove-orphans
```

### Bước 3: Verify

```bash
# Should show empty
docker ps

# Check no bot-core networks
docker network ls | grep bot-core
```

---

## 🎯 STOP CỤ THỂ TỪNG SERVICE

### Stop chỉ Python AI Service:
```bash
docker stop python-ai-service-dev
```

### Stop chỉ Rust Core Engine:
```bash
docker stop rust-core-engine-dev
```

### Stop chỉ Frontend Dashboard:
```bash
docker stop nextjs-ui-dashboard-dev
```

---

## 🧹 CLEANUP HOÀN TOÀN (Nếu cần)

### Stop + Remove containers + Cleanup volumes:
```bash
docker-compose down -v --remove-orphans
```

**⚠️ Warning:** `-v` sẽ xóa volumes (MongoDB data)!

### Remove tất cả images (nếu muốn rebuild từ đầu):
```bash
# List bot-core images
docker images | grep bot-core

# Remove all bot-core images
docker rmi $(docker images | grep bot-core | awk '{print $3}')
```

---

## ✅ VERIFICATION CHECKLIST

Sau khi stop, verify:

- [ ] No running containers: `docker ps` (should be empty)
- [ ] No bot-core networks: `docker network ls | grep bot-core`
- [ ] Ports released:
  - [ ] Port 3000 free: `lsof -i :3000`
  - [ ] Port 8000 free: `lsof -i :8000`
  - [ ] Port 8080 free: `lsof -i :8080`

---

## 🚀 RESTART SAU KHI STOP

### Khởi động lại với optimization:
```bash
./scripts/bot.sh start --memory-optimized
```

### Rebuild nếu cần:
```bash
docker-compose up -d --build
```

---

## 🚨 TROUBLESHOOTING

### Problem: Container không stop

**Giải pháp:**
```bash
# Force kill
docker kill rust-core-engine-dev python-ai-service-dev nextjs-ui-dashboard-dev
```

### Problem: Port vẫn bị chiếm

**Kiểm tra:**
```bash
lsof -i :3000
lsof -i :8000
lsof -i :8080
```

**Kill process:**
```bash
# Replace PID with actual process ID
kill -9 <PID>
```

### Problem: Network không xóa được

**Giải pháp:**
```bash
# Force disconnect all containers from network
docker network disconnect -f bot-core_bot-network <container_id>

# Then remove
docker network rm bot-core_bot-network
```

---

## 📋 QUICK COMMANDS SUMMARY

```bash
# STOP
./scripts/bot.sh stop                          # Recommended
docker-compose down                            # Alternative
docker stop $(docker ps -q)                    # Force stop all

# VERIFY
docker ps                                      # Should be empty

# CLEANUP
docker-compose down -v --remove-orphans        # Full cleanup (removes volumes!)

# RESTART
./scripts/bot.sh start --memory-optimized      # With optimization
```

---

## 💡 TIPS

1. **Always use `./scripts/bot.sh stop` first** - Safest method
2. **Check `docker ps` before starting** - Ensure clean state
3. **Use `docker logs <container>` before stopping** - Debug if needed
4. **Backup MongoDB data** before cleanup with `-v` flag

---

**Last Updated:** 2024-11-19
