# Docker Configuration Specification

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** DevOps Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Docker Architecture](#2-docker-architecture)
- [3. Service Configurations](#3-service-configurations)
- [4. Multi-Stage Builds](#4-multi-stage-builds)
- [5. Image Optimization](#5-image-optimization)
- [6. Docker Compose Profiles](#6-docker-compose-profiles)
- [7. Volume Management](#7-volume-management)
- [8. Network Configuration](#8-network-configuration)
- [9. Health Checks](#9-health-checks)
- [10. Resource Limits](#10-resource-limits)
- [11. Security Configuration](#11-security-configuration)
- [12. Build Strategies](#12-build-strategies)
- [13. Troubleshooting](#13-troubleshooting)

---

## 1. Overview

### 1.1 Purpose

This document defines the Docker configuration for all services in the Bot Core platform, including container specifications, networking, volumes, and deployment strategies.

### 1.2 Related Documents

- `INFRA-REQUIREMENTS.md` - Infrastructure requirements
- `docker-compose.yml` - Main compose file
- `Dockerfile` files - Service-specific images
- `scripts/bot.sh` - Control script

### 1.3 Docker Version Requirements

**Minimum Versions:**
- Docker Engine: 24.0+
- Docker Compose: 2.20+
- Docker BuildKit: Enabled

**Check Versions:**
```bash
docker --version
docker compose version
docker buildx version
```

---

## 2. Docker Architecture

### 2.1 System Architecture

```
┌────────────────────────────────────────────────────────────┐
│                    Docker Host                              │
├────────────────────────────────────────────────────────────┤
│                                                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │         Docker Network: bot-network (bridge)           │ │
│  │                  172.20.0.0/16                         │ │
│  │                                                         │ │
│  │  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐  │ │
│  │  │  Frontend   │  │  Rust Core  │  │  Python AI  │  │ │
│  │  │  Container  │  │  Container  │  │  Container  │  │ │
│  │  │             │  │             │  │             │  │ │
│  │  └─────┬───────┘  └─────┬───────┘  └─────┬───────┘  │ │
│  │        │                 │                 │          │ │
│  │        └─────────────────┼─────────────────┘          │ │
│  │                          │                             │ │
│  │                  ┌───────▼────────┐                   │ │
│  │                  │   MongoDB      │                   │ │
│  │                  │   Container    │                   │ │
│  │                  │   :27017       │                   │ │
│  │                  └────────────────┘                   │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                              │
│  ┌───────────────────────────────────────────────────────┐ │
│  │            Named Volumes (Persistent)                  │ │
│  │  • redis_data         • kong_data                      │ │
│  │  • rabbitmq_data      • prometheus_data                │ │
│  │  • grafana_data       • rust_target_cache              │ │
│  └───────────────────────────────────────────────────────┘ │
│                                                              │
└────────────────────────────────────────────────────────────┘
```

### 2.2 Service Dependencies

```mermaid
graph TD
    A[nextjs-ui-dashboard] --> B[rust-core-engine]
    B --> D[MongoDB]
    B --> D
    B --> E[Redis]
    B --> F[RabbitMQ]
```

---

## 3. Service Configurations

### 3.1 Frontend Service (React/Vite Dashboard)

#### Production Configuration

**Source:** `docker-compose.yml` (lines 188-227)

```yaml
nextjs-ui-dashboard:
  build:
    context: ./nextjs-ui-dashboard
    dockerfile: ${DOCKERFILE:-Dockerfile}
  container_name: nextjs-ui-dashboard
  restart: unless-stopped
  ports:
    - "3000:3000"
  environment:
    - NODE_ENV=${NODE_ENV:-production}
    - VITE_RUST_API_URL=http://rust-core-engine:8080
    - VITE_WS_URL=ws://rust-core-engine:8080/ws
    - VITE_API_TIMEOUT=10000
    - VITE_REFRESH_INTERVAL=5000
    - VITE_ENABLE_REALTIME=true
    - DASHBOARD_SESSION_SECRET=${DASHBOARD_SESSION_SECRET:?Error: DASHBOARD_SESSION_SECRET not set}
    - NODE_OPTIONS="--max-old-space-size=${NODE_MEMORY:-1024}"
  networks:
    - bot-network
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
    interval: 30s
    timeout: 10s
    retries: 3
    start_period: 30s
  depends_on:
    - rust-core-engine
  deploy:
    resources:
      limits:
        memory: ${FRONTEND_MEMORY_LIMIT:-1G}
        cpus: "${FRONTEND_CPU_LIMIT:-1}"
      reservations:
        memory: ${FRONTEND_MEMORY_RESERVE:-256M}
        cpus: "${FRONTEND_CPU_RESERVE:-0.5}"
  profiles:
    - prod
```

**Key Features:**
- Static build served by Vite preview
- Memory limit: 1GB (configurable)
- Health check on `/health` endpoint
- Depends on backend services

#### Development Configuration

**Source:** `docker-compose.yml` (lines 230-281)

```yaml
nextjs-ui-dashboard-dev:
  build:
    context: ./nextjs-ui-dashboard
    dockerfile: Dockerfile.dev
    target: development
  container_name: nextjs-ui-dashboard-dev
  restart: unless-stopped
  ports:
    - "3000:3000"
    - "24678:24678" # HMR WebSocket port
  environment:
    - NODE_ENV=development
    - VITE_RUST_API_URL=http://localhost:8080
    - VITE_WS_URL=ws://localhost:8080/ws
    - VITE_API_TIMEOUT=10000
    - VITE_REFRESH_INTERVAL=5000
    - VITE_ENABLE_REALTIME=true
    - CHOKIDAR_USEPOLLING=true
    - NODE_OPTIONS="--max-old-space-size=768"
    - HMR_PORT=24678
    - BUN_RUNTIME_TRANSPILER_CACHE_PATH=/tmp/bun-cache
    - BUN_ENABLE_JEMALLOC=true
    - BUN_ENABLE_SMOL=false
  volumes:
    - ./nextjs-ui-dashboard/src:/app/src:delegated
    - ./nextjs-ui-dashboard/public:/app/public:delegated
    - ./nextjs-ui-dashboard/index.html:/app/index.html:ro
    - ./nextjs-ui-dashboard/vite.config.ts:/app/vite.config.ts:ro
    - ./nextjs-ui-dashboard/tailwind.config.ts:/app/tailwind.config.ts:ro
  command: ["bun", "run", "dev", "--", "--host", "0.0.0.0", "--port", "3000"]
  networks:
    - bot-network
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:3000/"]
    interval: 10s
    timeout: 5s
    retries: 3
    start_period: 30s
  depends_on:
    - rust-core-engine-dev
  deploy:
    resources:
      limits:
        memory: 768M
        cpus: "1"
  profiles:
    - dev
```

**Key Features:**
- Hot module replacement (HMR) enabled
- Source code mounted as volumes
- Bun runtime for fast development
- Port 24678 for HMR WebSocket

### 3.2 Rust Core Engine Service

#### Production Configuration

**Source:** `docker-compose.yml` (lines 96-138)

```yaml
rust-core-engine:
  build:
    context: ./rust-core-engine
    dockerfile: ${DOCKERFILE:-Dockerfile}
  container_name: rust-core-engine
  restart: unless-stopped
  ports:
    - "8080:8080"
  environment:
    - RUST_LOG=${RUST_LOG:-info}
    - DATABASE_URL=${DATABASE_URL:?Error: DATABASE_URL not set}
    - BINANCE_API_KEY=${BINANCE_API_KEY:?Error: BINANCE_API_KEY not set}
    - BINANCE_SECRET_KEY=${BINANCE_SECRET_KEY:?Error: BINANCE_SECRET_KEY not set}
    - BINANCE_TESTNET=${BINANCE_TESTNET:-true}
    - TRADING_ENABLED=${TRADING_ENABLED:-false}
    - INTER_SERVICE_TOKEN=${INTER_SERVICE_TOKEN:?Error: INTER_SERVICE_TOKEN not set}
    - RUST_API_KEY=${RUST_API_KEY:?Error: RUST_API_KEY not set}
  volumes:
    - ./rust-core-engine/data:/app/data
    - ./rust-core-engine/logs:/app/logs
    - ./rust-core-engine/config.toml:/app/config.toml
  networks:
    - bot-network
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
    interval: 30s
    timeout: 10s
    retries: 3
    start_period: 60s
  depends_on:
      condition: service_healthy
  deploy:
    resources:
      limits:
        memory: ${RUST_MEMORY_LIMIT:-2G}
        cpus: "${RUST_CPU_LIMIT:-2}"
      reservations:
        memory: ${RUST_MEMORY_RESERVE:-1G}
        cpus: "${RUST_CPU_RESERVE:-1}"
  profiles:
    - prod
```

**Key Features:**
- Release build for production
- Health check on `/api/health`
- Waits for Python AI service to be healthy
- Memory limit: 2GB (configurable)

#### Development Configuration

**Source:** `docker-compose.yml` (lines 140-185)

```yaml
rust-core-engine-dev:
  build:
    context: ./rust-core-engine
    dockerfile: Dockerfile.dev
  container_name: rust-core-engine-dev
  restart: unless-stopped
  ports:
    - "8080:8080"
  environment:
    - RUST_LOG=debug
    - RUST_BACKTRACE=1
    - DATABASE_URL=${DATABASE_URL:?Error: DATABASE_URL not set}
    - BINANCE_API_KEY=${BINANCE_API_KEY:?Error: BINANCE_API_KEY not set}
    - BINANCE_SECRET_KEY=${BINANCE_SECRET_KEY:?Error: BINANCE_SECRET_KEY not set}
    - BINANCE_TESTNET=${BINANCE_TESTNET:-true}
    - TRADING_ENABLED=${TRADING_ENABLED:-false}
    - INTER_SERVICE_TOKEN=${INTER_SERVICE_TOKEN:?Error: INTER_SERVICE_TOKEN not set}
    - RUST_API_KEY=${RUST_API_KEY:?Error: RUST_API_KEY not set}
  volumes:
    - ./rust-core-engine/src:/app/src
    - ./rust-core-engine/Cargo.toml:/app/Cargo.toml
    - ./rust-core-engine/Cargo.lock:/app/Cargo.lock
    - ./rust-core-engine/config.toml:/app/config.toml
    - ./rust-core-engine/data:/app/data
    - ./rust-core-engine/logs:/app/logs
    - rust_target_cache:/app/target
  networks:
    - bot-network
  healthcheck:
    test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
    interval: 10s
    timeout: 5s
    retries: 3
    start_period: 60s
  depends_on:
      condition: service_healthy
  deploy:
    resources:
      limits:
        memory: 1.5G
        cpus: "1.5"
  profiles:
    - dev
```

**Key Features:**
- Debug build with backtrace
- Source code mounted for hot reload
- Target directory cached in named volume
- Faster health checks (10s interval)

#### Production Configuration

**Source:** `docker-compose.yml` (lines 3-42)

```yaml
  build:
    dockerfile: ${DOCKERFILE:-Dockerfile}
  restart: unless-stopped
  ports:
    - "27017:27017" # MongoDB (if needed)
"6379:6379"   # Redis (optional)
"5672:5672"   # RabbitMQ AMQP (optional)
"15672:15672" # RabbitMQ UI (optional)
"9090:9090"   # Prometheus (optional)
"3001:3000"   # Grafana (optional, port changed to avoid conflict)
```

**Port Conflict Resolution:**
```bash
# Check if port is in use
lsof -i :3000
netstat -tulpn | grep :3000

# Kill process on port
kill -9 $(lsof -t -i :3000)

# Use different host port
ports:
  - "3001:3000"  # Map host 3001 to container 3000
```

### 8.4 Network Troubleshooting

**Inspect network:**
```bash
# List networks
docker network ls

# Inspect network
docker network inspect bot-network

# List containers in network
docker network inspect bot-network --format '{{range .Containers}}{{.Name}} {{end}}'
```

**Test connectivity:**
```bash
# From host to container
curl http://localhost:8080/api/health

# From container to container
docker exec rust-core-engine 
# DNS resolution
```

---

## 9. Health Checks

### 9.1 Health Check Configuration

**Frontend Health Check:**
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:3000/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 30s
```

**Rust Engine Health Check:**
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8080/api/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 60s
```

healthcheck:
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 60s
```

### 9.2 Health Check Parameters

| Parameter | Description | Recommended Value |
|-----------|-------------|-------------------|
| `interval` | Time between checks | 30s (production), 10s (dev) |
| `timeout` | Max time for check to complete | 10s |
| `retries` | Failed attempts before unhealthy | 3 |
| `start_period` | Grace period after start | 30-60s |

### 9.3 Health Check Commands

**Check service health:**
```bash
# View health status
docker compose ps

# Inspect container health
docker inspect --format='{{.State.Health.Status}}' rust-core-engine

# View health check logs
docker inspect --format='{{range .State.Health.Log}}{{.Output}}{{end}}' rust-core-engine
```

**Manual health checks:**
```bash
# Frontend
curl -f http://localhost:3000/health

# Rust Engine
curl -f http://localhost:8080/api/health

# Python AI

# All services
make health
```

### 9.4 Dependency Health Checks

**Conditional dependencies:**
```yaml
depends_on:
    condition: service_healthy  # Wait until healthy
```

**Startup order:**
1. Python AI service starts
2. Health check runs every 30s
3. After 3 successful checks, marked healthy
4. Rust Engine starts
5. Frontend starts (depends on both)

---

## 10. Resource Limits

### 10.1 Memory Limits

**Production Limits:**
```yaml
  deploy:
    resources:
      limits:
        memory: 2G
      reservations:
        memory: 1G

rust-core-engine:
  deploy:
    resources:
      limits:
        memory: 2G
      reservations:
        memory: 1G

nextjs-ui-dashboard:
  deploy:
    resources:
      limits:
        memory: 1G
      reservations:
        memory: 256M
```

**Memory Optimized Settings (from bot.sh):**
```bash
export PYTHON_MEMORY_LIMIT="1.5G"
export RUST_MEMORY_LIMIT="1G"
export FRONTEND_MEMORY_LIMIT="512M"
```

### 10.2 CPU Limits

**Production Limits:**
```yaml
  deploy:
    resources:
      limits:
        cpus: "2"
      reservations:
        cpus: "1"

rust-core-engine:
  deploy:
    resources:
      limits:
        cpus: "2"
      reservations:
        cpus: "1"

nextjs-ui-dashboard:
  deploy:
    resources:
      limits:
        cpus: "1"
      reservations:
        cpus: "0.5"
```

### 10.3 Resource Monitoring

**Monitor resource usage:**
```bash
# Real-time stats
docker stats

# Specific service
docker stats rust-core-engine

# Formatted output
docker stats --format "table {{.Name}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.CPUPerc}}"

# From bot.sh
./scripts/bot.sh status
```

**Resource alerts:**
- Memory usage > 90%: Warning
- CPU usage > 80% sustained: Warning
- OOM kill detected: Critical

---

## 11. Security Configuration

### 11.1 Secret Management

**Environment variable validation:**
```yaml
environment:
  # Required secrets (will fail if not set)
  - INTER_SERVICE_TOKEN=${INTER_SERVICE_TOKEN:?Error: INTER_SERVICE_TOKEN not set}
  - BINANCE_API_KEY=${BINANCE_API_KEY:?Error: BINANCE_API_KEY not set}
  - DATABASE_URL=${DATABASE_URL:?Error: DATABASE_URL not set}

  # Optional secrets
  - XAI_API_KEY=${XAI_API_KEY}
```

**Generate secrets:**
```bash
# From Makefile
make generate-secrets

# Manual generation
openssl rand -hex 32  # For tokens
openssl rand -hex 16  # For passwords
```

### 11.2 Container Security

**Security best practices:**
1. Run as non-root user
2. Use read-only root filesystem where possible
3. Drop unnecessary capabilities
4. Limit network access

**Example secure configuration:**
```yaml
  security_opt:
    - no-new-privileges:true
  read_only: true
  tmpfs:
    - /tmp
  cap_drop:
    - ALL
  cap_add:
    - NET_BIND_SERVICE
```

### 11.3 Network Security

**Firewall rules:**
- Only expose necessary ports
- Use internal DNS names for service-to-service
- Implement mTLS for production

**Security groups (production):**
```yaml
# ALB → Services
- Source: ALB security group
  Ports: 3000, 8080, 8000

# Services → Database
- Source: Service security group
  Port: 27017

# Services → External APIs
- Destination: 0.0.0.0/0
  Port: 443
```

---

## 12. Build Strategies

### 12.1 Sequential Build (Memory Optimized)

**From Makefile:**
```bash
make build-fast
# Builds services one at a time to avoid OOM
```

**Build script (`scripts/build-services.sh`):**
```bash
#!/bin/bash

echo "Building services sequentially (memory optimized)..."

echo "1/3 Building Python AI Service..."
if [ $? -ne 0 ]; then
  echo "Failed to build Python AI Service"
  exit 1
fi

echo "2/3 Building Rust Core Engine..."
docker compose build rust-core-engine
if [ $? -ne 0 ]; then
  echo "Failed to build Rust Core Engine"
  exit 1
fi

echo "3/3 Building Frontend..."
docker compose build nextjs-ui-dashboard
if [ $? -ne 0 ]; then
  echo "Failed to build Frontend"
  exit 1
fi

echo "All services built successfully!"
```

### 12.2 Parallel Build

**From Makefile:**
```bash
make build
# Builds all services in parallel (requires more memory)
```

**Parallel build command:**
```bash
docker compose build --parallel
```

### 12.3 Clean Build

**From Makefile:**
```bash
make build-clean
# Rebuild without cache
```

**Clean build commands:**
```bash
# Remove all images
docker compose down --rmi all

# Build without cache
docker compose build --no-cache

# Full clean build
make clean-all && make build
```

### 12.4 BuildKit Features

**Enable BuildKit:**
```bash
export DOCKER_BUILDKIT=1
export COMPOSE_DOCKER_CLI_BUILD=1
```

**BuildKit benefits:**
- Parallel stage execution
- Efficient layer caching
- Build-time secrets
- SSH agent forwarding

**Example with secrets:**
```dockerfile
# syntax=docker/dockerfile:1.4
RUN --mount=type=secret,id=api_key \
    API_KEY=$(cat /run/secrets/api_key) && \
    ./configure --api-key=$API_KEY
```

---

## 13. Troubleshooting

### 13.1 Common Issues

#### Issue: Service won't start

**Symptoms:**
- Container exits immediately
- Health check never passes

**Diagnosis:**
```bash
# View logs

# Inspect container

# Check exit code
docker ps -a
```

**Solutions:**
- Check environment variables
- Verify configuration files
- Check port conflicts
- Review resource limits

#### Issue: Out of Memory (OOM)

**Symptoms:**
- Container killed unexpectedly
- `OOMKilled` status

**Diagnosis:**
```bash
# Check OOM status
docker inspect --format='{{.State.OOMKilled}}' rust-core-engine

# View memory usage
docker stats rust-core-engine
```

**Solutions:**
- Increase memory limit
- Use memory-optimized settings: `./scripts/bot.sh start --memory-optimized`
- Build services sequentially: `make build-fast`

#### Issue: Build failures

**Symptoms:**
- Build process hangs
- Out of disk space
- Download errors

**Diagnosis:**
```bash
# Check disk space
df -h

# View build logs
```

**Solutions:**
```bash
# Clean up
docker system prune -a

# Remove unused images
docker image prune -a

# Remove build cache
docker builder prune

# Full cleanup
make clean-all
```

#### Issue: Network connectivity

**Symptoms:**
- Service can't reach another service
- DNS resolution fails

**Diagnosis:**
```bash
# Test DNS

# Test connectivity
docker exec rust-core-engine 
# Inspect network
docker network inspect bot-network
```

**Solutions:**
```bash
# Recreate network
docker compose down
docker network rm bot-network
docker compose up -d

# Check firewall
# (platform-specific commands)
```

### 13.2 Debugging Commands

**Container inspection:**
```bash
# View container details
docker inspect rust-core-engine

# Enter container shell
docker exec -it rust-core-engine sh

# View container processes
docker top rust-core-engine

# View resource usage
docker stats rust-core-engine
```

**Log analysis:**
```bash
# Follow logs
docker compose logs -f rust-core-engine

# Last 100 lines
docker compose logs --tail=100 rust-core-engine

# Logs since timestamp
docker compose logs --since=2025-10-11T10:00:00 rust-core-engine

# All service logs
docker compose logs -f
```

### 13.3 Performance Tuning

**Docker daemon configuration (`/etc/docker/daemon.json`):**
```json
{
  "log-driver": "json-file",
  "log-opts": {
    "max-size": "10m",
    "max-file": "3"
  },
  "storage-driver": "overlay2",
  "default-ulimits": {
    "nofile": {
      "Name": "nofile",
      "Hard": 64000,
      "Soft": 64000
    }
  }
}
```

**Compose performance:**
```yaml
# Use delegated mounts (macOS)
volumes:
  - ./src:/app/src:delegated

# Reduce health check frequency
healthcheck:
  interval: 60s  # Instead of 10s

# Disable logging for high-volume services
logging:
  driver: none
```

---

## Appendix A: Complete docker-compose.yml

**Location:** `/Users/dungngo97/Documents/bot-core/docker-compose.yml`

See full file for complete configuration.

---

## Appendix B: Docker Commands Reference

### Basic Commands

```bash
# Start services
docker compose up -d

# Stop services
docker compose down

# Restart service
docker compose restart rust-core-engine

# View logs
docker compose logs -f

# Execute command
docker compose exec rust-core-engine sh

# Build services
docker compose build

# Pull images
docker compose pull

# List services
docker compose ps

# View config
docker compose config
```

### Makefile Commands

```bash
# Setup
make setup

# Build
make build         # Parallel build
make build-fast    # Sequential build
make build-clean   # Clean build

# Start/Stop
make start         # Start production
make dev           # Start development
make stop          # Stop all
make restart       # Restart all

# Logs
make logs          # All services
make logs-rust     # Rust only
make logs-python   # Python only
make logs-frontend # Frontend only

# Health
make health        # Check all services

# Clean
make clean         # Remove containers
make clean-all     # Remove everything
```

### bot.sh Commands

```bash
# Start services
./scripts/bot.sh start
./scripts/bot.sh start --memory-optimized
./scripts/bot.sh start --with-enterprise
./scripts/bot.sh start --with-monitoring

# Development
./scripts/bot.sh dev

# Management
./scripts/bot.sh stop
./scripts/bot.sh restart
./scripts/bot.sh status
./scripts/bot.sh logs --service rust-core-engine

# Cleanup
./scripts/bot.sh clean
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | DevOps Team | Initial version |
| 1.1.0 | 2026-03-03 | System | Added service matrix tables for all compose files |

---

## 14. Service Matrix

### 14.1 docker-compose.yml (Dev + Prod profiles)

| Service | Container | Port(s) | Image | Profile | Memory Limit | Healthcheck |
|---------|-----------|---------|-------|---------|-------------|------------|
| `rust-core-engine` | `rust-core-engine` | 8080 | Build: `rust-core-engine/Dockerfile` | `prod` | 2G | `GET /api/health` |
| `rust-core-engine-dev` | `rust-core-engine-dev` | 8080 | Build: `rust-core-engine/Dockerfile.dev` | `dev` | 1.5G | `GET /api/health` |
| `nextjs-ui-dashboard` | `nextjs-ui-dashboard` | 3000 | Build: `nextjs-ui-dashboard/Dockerfile` | `prod` | 1G | `GET /health` |
| `nextjs-ui-dashboard-dev` | `nextjs-ui-dashboard-dev` | 3000, 24678 | Build: `nextjs-ui-dashboard/Dockerfile.dev` | `dev` | 768M | `GET /` |
| `mongodb` | `mongodb` | 27017 | `mongo:7.0` | (no profile — always) | 2G | `mongosh ping` |
| `redis` | `redis-cache` | (internal) | `redis:7-alpine` | `redis` | 256M | `redis-cli ping` |
| `mcp-server` | `mcp-server` | 8090 | Build: `mcp-server/Dockerfile` | `prod` | 512M | `GET /health` |
| `mcp-server-dev` | `mcp-server` | 8090 | Build: `mcp-server/Dockerfile` | `dev` | 512M | `GET /health` |
| `openclaw` | `openclaw` | (none/internal) | Build: `openclaw/Dockerfile` | `prod` | 2048M | `curl canvas` |
| `openclaw-dev` | `openclaw` | 18789 | Build: `openclaw/Dockerfile` | `dev` | 2048M | `curl canvas` |

**Key env vars (docker-compose.yml):**

| Service | Critical Env Vars |
|---------|------------------|
| `rust-core-engine` | `BINANCE_API_KEY`, `BINANCE_SECRET_KEY`, `BINANCE_TESTNET`, `TRADING_ENABLED`, `DATABASE_URL`, `JWT_SECRET` |
| `mcp-server` | `MCP_AUTH_TOKEN`, `RUST_API_URL`, `PYTHON_API_URL`, `BOTCORE_EMAIL`, `BOTCORE_PASSWORD` |
| `openclaw` | `TELEGRAM_BOT_TOKEN`, `TELEGRAM_USER_ID`, `MCP_URL`, `MCP_AUTH_TOKEN`, `ANTHROPIC_API_KEY`, `XAI_API_KEY` |

---

### 14.2 docker-compose-vps.yml (VPS Production — 7 core services)

| Service | Container | Port(s) | Image | Memory Limit | Healthcheck | Depends On |
|---------|-----------|---------|-------|-------------|------------|-----------|
| `mongodb` | `mongodb` | 27017 | `mongo:7.0` | 2G | `mongosh ping` | — |
| `nextjs-ui-dashboard` | `nextjs-ui-dashboard` | 3000 | Build: `nextjs-ui-dashboard/Dockerfile` | 512M | `GET /` | rust-core-engine |
| `redis` | `redis-cache` | (internal) | `redis:7-alpine` | 256M | `redis-cli ping` | — |
| `openclaw` | `openclaw` | 18789 | Build: `openclaw/Dockerfile` | 768M | `curl canvas` (120s start) | mcp-server |

**VPS-specific env vars:**

| Variable | VPS Value |
|----------|----------|
| `BINANCE_TESTNET` | `true` (keep safe) |
| `TRADING_ENABLED` | `true` (paper trading active) |
| `BINANCE_BASE_URL` | `https://api.binance.com` (mainnet for market data) |
| `TZ` | `Asia/Ho_Chi_Minh` |
| `REDIS_HOST` | `redis-cache` |

**VPS**: 16GB RAM / Viettel IDC. SSH: `root@180.93.2.247`

---

### 14.3 docker-compose.prod.yml

Extends `docker-compose.yml` with production overrides. Used alongside base file:
```bash
docker compose -f docker-compose.yml -f docker-compose.prod.yml up
```

Typical overrides: stricter resource limits, `NODE_ENV=production`, no dev volumes.

---

### 14.4 Startup Order (VPS)

```
mongodb (healthy)
            └─> rust-core-engine (healthy, 900s start — Rust compile + init)
                    ├─> nextjs-ui-dashboard
                    └─> mcp-server (healthy)
                            └─> openclaw (healthy, 120s start)
```

**Note**: `rust-core-engine` has a 900-second `start_period` because Cargo compilation can take 10–15 minutes on first build. Subsequent starts are fast (pre-compiled binary).

---

**Document End**
