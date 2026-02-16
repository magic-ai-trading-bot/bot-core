# DEPLOYMENT.md - Deployment & Operations Knowledge

## VPS Production Environment

| Item | Value |
|------|-------|
| **IP** | 180.93.2.247 |
| **OS** | Ubuntu (Docker host) |
| **RAM** | ~6 GB |
| **Compose File** | docker-compose-vps.yml |
| **Deploy Script** | .github/workflows/deploy-vps.yml |

## Services (7 total)

| Service | Container | Port | Health Check |
|---------|-----------|------|-------------|
| MongoDB | mongodb | 27017 | `mongosh --eval "db.adminCommand('ping')"` |
| Redis | redis-cache | 6379 | `redis-cli ping` |
| Python AI | python-ai-service | 8000 | `curl http://localhost:8000/health` |
| Rust Backend | rust-core-engine | 8080 | `curl http://localhost:8080/api/health` |
| Frontend | nextjs-ui-dashboard | 3000 | `curl http://localhost:3000` |
| MCP Server | mcp-server | 8090 | `curl http://localhost:8090/health` |
| OpenClaw | openclaw | 18789 | `pgrep -f openclaw` (no HTTP health endpoint) |

## Access URLs

| Service | URL |
|---------|-----|
| Frontend Dashboard | http://180.93.2.247:3000 |
| Rust API | http://180.93.2.247:8080/api/health |
| Python AI API | http://180.93.2.247:8000/health |
| OpenClaw UI | http://180.93.2.247:18789 |
| MCP Server | http://180.93.2.247:8090/health |

## Deployment Process

1. Code pushed to `main` branch on GitHub
2. GitHub Actions workflow triggers (deploy-vps.yml)
3. SSH into VPS → `git pull` → selective rebuild (only changed services)
4. `docker compose up -d` → rolling restart (2-5s downtime per service)
5. Health checks verify all services

## Common Operations

**Check all services**: `docker ps --format "table {{.Names}}\t{{.Status}}"`
**View logs**: `docker logs <container-name> --tail 50`
**Restart service**: `docker compose -f docker-compose-vps.yml restart <service>`
**Full rebuild**: `docker compose -f docker-compose-vps.yml up -d --build`

## Known Issues & Troubleshooting

### OpenClaw config overwrite
OpenClaw modifies `openclaw.json` on startup. When pulling code on VPS, use:
```bash
git fetch origin && git reset --hard origin/main
```

### Telegram bot conflict
If you see "getUpdates conflict: terminated by other getUpdates request", another bot instance with the same token is running somewhere. Only 1 instance can use a Telegram bot token at a time.

### Rust engine startup time
Rust backend takes up to 15 minutes to become healthy (start_period: 900s) due to loading 500 candles per symbol per timeframe from Binance API.

### Binance rate limiting
You may see "Rate limited (403 Forbidden)" warnings during market data refresh. This is normal - the client retries automatically (3 attempts with backoff).

### Signals marked executed but no trade opened
Signals can be marked `executed: true` with `trade_id: null` when:
1. **Neutral signal** (50% confidence) - Cannot execute neutral signals
2. **Correlation limit** (70%) - Too many positions in same direction (only checked with 3+ open positions)
3. **Daily loss limit** (3%) - Hit daily loss cap
4. **Cool-down** - In cooldown after consecutive losses
5. **Max positions** reached - Already at position limit

This is **normal risk management behavior**, not a bug.

## Data Volumes

| Volume | Purpose |
|--------|---------|
| mongodb_data | MongoDB data files |
| mongodb_config | MongoDB config |
| redis_data | Redis persistence |
| openclaw_data | OpenClaw persistent state |

**Reset all data**: Stop services → `docker volume rm` all volumes → restart
