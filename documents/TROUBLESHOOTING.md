# Troubleshooting Guide

This comprehensive guide helps you diagnose and resolve common issues in the Bot Core trading platform.

## Table of Contents

- [Quick Diagnostics](#quick-diagnostics)
- [Common Issues](#common-issues)
- [Service-Specific Issues](#service-specific-issues)
- [Error Messages Reference](#error-messages-reference)
- [Performance Issues](#performance-issues)
- [Network & Connectivity](#network--connectivity)
- [Database Issues](#database-issues)
- [Trading Issues](#trading-issues)
- [Debugging Tools](#debugging-tools)
- [Getting Help](#getting-help)

## Quick Diagnostics

### Health Check Commands

```bash
# Check all services status
./scripts/bot.sh status

# Check individual service health
curl http://localhost:8080/api/health  # Rust Core
curl http://localhost:8000/health      # Python AI
curl http://localhost:3000/api/health  # Next.js

# Check Docker containers
docker ps
docker stats --no-stream

# Check logs
./scripts/bot.sh logs                          # All services
./scripts/bot.sh logs --service rust-core-engine
./scripts/bot.sh logs --service python-ai-service
```

### System Requirements Check

```bash
# Check Docker version
docker --version  # Should be 24.0+
docker-compose --version  # Should be 2.0+

# Check available resources
docker system df
free -h  # Check RAM (need 8GB minimum)
df -h    # Check disk space (need 50GB minimum)

# Check ports availability
netstat -tuln | grep -E '3000|8000|8080|27017|6379|5672'
```

## Common Issues

### 1. Services Won't Start

**Symptom**: `./scripts/bot.sh start` fails or containers exit immediately

**Diagnosis**:
```bash
# Check Docker logs
docker-compose logs

# Check for port conflicts
lsof -i :3000  # Frontend
lsof -i :8000  # Python
lsof -i :8080  # Rust
```

**Solutions**:

**A. Port Already in Use**
```bash
# Kill process using the port
kill -9 $(lsof -ti:8080)

# Or change port in docker-compose.yml
# Edit ports: "8081:8080" instead of "8080:8080"
```

**B. Insufficient Memory**
```bash
# Start with memory optimization
./scripts/bot.sh start --memory-optimized

# Or increase Docker memory limit
# Docker Desktop > Settings > Resources > Memory: 8GB
```

**C. Missing Environment Variables**
```bash
# Check .env file exists
ls -la .env

# Copy from example if missing
cp .env.example .env

# Edit required variables
nano .env
```

### 2. Out of Memory Errors

**Symptom**: Services crash with "OOMKilled" or "exit code 137"

**Diagnosis**:
```bash
# Check memory usage
docker stats --no-stream

# Check container restart count
docker ps -a | grep -E 'rust-core|python-ai|nextjs'

# Check logs for OOM
docker logs rust-core-engine 2>&1 | grep -i "memory\|oom"
```

**Solutions**:

**A. Use Memory-Optimized Mode**
```bash
# Stop all services
./scripts/bot.sh stop

# Start with memory limits
./scripts/bot.sh start --memory-optimized
```

**B. Increase Memory Limits**
```yaml
# Edit docker-compose.yml
services:
  python-ai-service:
    deploy:
      resources:
        limits:
          memory: 2G  # Increase from 1.5G
```

**C. Sequential Build**
```bash
# Build one service at a time
make build-fast
```

### 3. Connection Refused / Service Unreachable

**Symptom**: "Connection refused" or "ECONNREFUSED" errors

**Diagnosis**:
```bash
# Check if service is running
docker ps | grep rust-core-engine

# Check service logs
docker logs rust-core-engine --tail 50

# Test connectivity
curl -v http://localhost:8080/api/health
telnet localhost 8080
```

**Solutions**:

**A. Wait for Services to Start**
```bash
# Services need 30-60 seconds to fully start
# Check health endpoints
while ! curl -f http://localhost:8080/api/health; do
  echo "Waiting for Rust service..."
  sleep 5
done
```

**B. Check Docker Network**
```bash
# Inspect network
docker network inspect bot-network

# Recreate network
docker-compose down
docker network rm bot-network
docker-compose up -d
```

**C. Firewall Issues**
```bash
# Check if firewall is blocking
sudo ufw status
sudo ufw allow 8080/tcp

# macOS
sudo /usr/libexec/ApplicationFirewall/socketfilterfw --add /usr/local/bin/docker
```

### 4. Database Connection Issues

**Symptom**: "MongoNetworkError" or "Connection timeout"

**Diagnosis**:
```bash
# Check MongoDB is running
docker ps | grep mongo

# Check MongoDB logs
docker logs mongodb

# Test connection
docker exec -it mongodb mongosh --eval "db.adminCommand('ping')"
```

**Solutions**:

**A. MongoDB Not Started**
```bash
# Start MongoDB
docker-compose up -d mongodb

# Wait for ready
docker logs mongodb 2>&1 | grep "Waiting for connections"
```

**B. Wrong Connection String**
```bash
# Check .env file
cat .env | grep DATABASE_URL

# Should be (for Docker):
# DATABASE_URL=mongodb://mongodb:27017/trading_bot

# For external MongoDB Atlas:
# DATABASE_URL=mongodb+srv://user:pass@cluster.mongodb.net/trading_bot
```

**C. MongoDB Authentication Failed**
```bash
# Reset MongoDB
docker-compose down -v  # WARNING: Deletes data
docker-compose up -d mongodb

# Or check credentials
docker exec -it mongodb mongosh -u admin -p password
```

### 5. Trading Disabled / Orders Rejected

**Symptom**: "Trading disabled" or "Orders rejected" errors

**Diagnosis**:
```bash
# Check trading status
curl http://localhost:8080/api/health | jq '.system_status.trading_enabled'

# Check user settings
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/account | jq '.can_trade'

# Check .env file
grep TRADING_ENABLED .env
```

**Solutions**:

**A. Trading Disabled in Config**
```bash
# Enable trading in .env
echo "TRADING_ENABLED=true" >> .env

# Restart services
./scripts/bot.sh restart
```

**B. Using Testnet**
```bash
# Check testnet setting
grep BINANCE_TESTNET .env

# Should be:
# BINANCE_TESTNET=true  (for testing)
# BINANCE_TESTNET=false (for live trading - USE WITH CAUTION)
```

**C. Risk Limits Exceeded**
```bash
# Check account status
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/account

# Check risk metrics
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/risk/metrics
```

## Service-Specific Issues

### Rust Core Engine

**Issue: WebSocket Connection Fails**
```bash
# Check WebSocket endpoint
curl -i -N \
  -H "Connection: Upgrade" \
  -H "Upgrade: websocket" \
  -H "Sec-WebSocket-Version: 13" \
  -H "Sec-WebSocket-Key: SGVsbG8sIHdvcmxkIQ==" \
  http://localhost:8080/ws

# Check logs for WebSocket errors
docker logs rust-core-engine 2>&1 | grep -i websocket
```

**Issue: Compilation Errors**
```bash
# Clean build
cd rust-core-engine
cargo clean
cargo build

# Check Rust version
rustc --version  # Should be 1.75+

# Update Rust
rustup update stable
```

**Issue: High CPU Usage**
```bash
# Check profiling
docker stats rust-core-engine

# Reduce WebSocket connections
# Edit config.toml
[websocket]
max_connections = 1000  # Reduce from 10000

# Restart
docker-compose restart rust-core-engine
```

### Python AI Service

**Issue: OpenAI API Errors**
```bash
# Check API key
grep OPENAI_API_KEY .env

# Test API key
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

# Check rate limits
docker logs python-ai-service | grep -i "rate limit"
```

**Issue: ML Model Loading Fails**
```bash
# Check model files
docker exec python-ai-service ls -la /app/models

# Check memory available for models
docker stats python-ai-service

# Reduce model size or disable heavy models
# Edit python-ai-service/config.yaml
ml_models:
  enabled: false  # Temporarily disable
```

**Issue: Import Errors**
```bash
# Rebuild with dependencies
cd python-ai-service
pip install -r requirements.txt

# Or rebuild container
docker-compose build --no-cache python-ai-service
docker-compose up -d python-ai-service
```

### Next.js Dashboard

**Issue: Build Fails**
```bash
# Check Node version
node --version  # Should be 18+

# Clear cache and rebuild
cd nextjs-ui-dashboard
rm -rf node_modules .next
npm install
npm run build

# Check for TypeScript errors
npm run type-check
```

**Issue: Hot Reload Not Working**
```bash
# Check if running in dev mode
docker-compose -f docker-compose.yml -f docker-compose.dev.yml up

# Or use dev script
./scripts/bot.sh dev

# Check watchman (for macOS)
brew install watchman
```

**Issue: API Connection Errors**
```bash
# Check VITE env variables
cat nextjs-ui-dashboard/.env

# Should have:
# VITE_RUST_API_URL=http://localhost:8080
# VITE_PYTHON_API_URL=http://localhost:8000

# Rebuild with new env
docker-compose build nextjs-ui-dashboard
docker-compose up -d nextjs-ui-dashboard
```

## Error Messages Reference

### HTTP Error Codes

| Code | Error | Cause | Solution |
|------|-------|-------|----------|
| 400 | Bad Request | Invalid parameters | Check request body against API spec |
| 401 | Unauthorized | Missing/invalid token | Re-authenticate to get new token |
| 403 | Forbidden | Insufficient permissions | Check user role and permissions |
| 404 | Not Found | Resource doesn't exist | Verify resource ID/endpoint |
| 429 | Rate Limited | Too many requests | Wait for rate limit reset |
| 500 | Internal Server Error | Server error | Check service logs |
| 502 | Bad Gateway | Service unavailable | Check if backend services are running |
| 503 | Service Unavailable | Service down/overloaded | Wait or check service health |

### Application Error Codes

**INSUFFICIENT_BALANCE**
```json
{
  "code": "INSUFFICIENT_BALANCE",
  "message": "Insufficient balance for this trade"
}
```
**Solution**: Check account balance, reduce order size, or add funds

**MAX_POSITIONS_EXCEEDED**
```json
{
  "code": "MAX_POSITIONS_EXCEEDED",
  "message": "Maximum concurrent positions reached"
}
```
**Solution**: Close existing positions or increase max_positions in settings

**INVALID_SYMBOL**
```json
{
  "code": "INVALID_SYMBOL",
  "message": "Trading pair not supported"
}
```
**Solution**: Check supported symbols list, use valid pairs like BTCUSDT

**SIGNAL_EXPIRED**
```json
{
  "code": "SIGNAL_EXPIRED",
  "message": "AI signal has expired"
}
```
**Solution**: Request new AI analysis, signals expire after 5-30 minutes

**DAILY_LOSS_LIMIT_EXCEEDED**
```json
{
  "code": "DAILY_LOSS_LIMIT_EXCEEDED",
  "message": "Daily loss limit reached"
}
```
**Solution**: Trading disabled for 24 hours, review strategy

## Performance Issues

### Slow API Responses

**Diagnosis**:
```bash
# Measure response time
time curl http://localhost:8080/api/health

# Check service resource usage
docker stats

# Check database performance
docker exec mongodb mongosh --eval "db.currentOp()"
```

**Solutions**:

**A. Enable Redis Caching**
```bash
# Start with Redis
./scripts/bot.sh start --with-redis

# Verify caching is working
docker logs python-ai-service | grep -i cache
```

**B. Add Database Indexes**
```javascript
// Connect to MongoDB
docker exec -it mongodb mongosh trading_bot

// Add indexes
db.trades.createIndex({ user_id: 1, created_at: -1 })
db.positions.createIndex({ user_id: 1, closed_at: 1 })
db.ai_analysis.createIndex({ symbol: 1, created_at: -1 })
```

**C. Optimize Queries**
```bash
# Enable MongoDB profiling
docker exec mongodb mongosh --eval "db.setProfilingLevel(2)"

# Check slow queries
docker exec mongodb mongosh --eval "db.system.profile.find().sort({ts:-1}).limit(10)"
```

### High Memory Usage

**Diagnosis**:
```bash
# Check memory per service
docker stats --format "table {{.Name}}\t{{.MemUsage}}"

# Check system memory
free -h

# Check for memory leaks
docker logs rust-core-engine | grep -i "memory\|allocation"
```

**Solutions**:

**A. Implement Memory Limits**
```yaml
# docker-compose.yml
services:
  rust-core-engine:
    deploy:
      resources:
        limits:
          memory: 1G
        reservations:
          memory: 512M
```

**B. Clear Caches**
```bash
# Clear Redis cache
docker exec redis redis-cli FLUSHALL

# Clear application caches
docker-compose restart
```

## Network & Connectivity

### DNS Resolution Issues

```bash
# Check DNS
nslookup mongodb
ping mongodb

# Use IP instead of hostname in .env
docker inspect mongodb | grep IPAddress

# Update DATABASE_URL with IP
DATABASE_URL=mongodb://172.18.0.2:27017/trading_bot
```

### Proxy/VPN Issues

```bash
# Check if behind proxy
env | grep -i proxy

# Configure Docker proxy
mkdir -p ~/.docker
cat > ~/.docker/config.json <<EOF
{
  "proxies": {
    "default": {
      "httpProxy": "http://proxy:port",
      "httpsProxy": "http://proxy:port"
    }
  }
}
EOF
```

## Database Issues

### MongoDB Disk Space Full

```bash
# Check MongoDB disk usage
docker exec mongodb df -h /data/db

# Compact collections
docker exec mongodb mongosh --eval "db.runCommand({compact: 'trades'})"

# Drop old data
docker exec mongodb mongosh trading_bot --eval "
  db.trades.deleteMany({
    created_at: { \$lt: new Date(Date.now() - 90*24*60*60*1000) }
  })
"
```

### MongoDB Replication Lag

```bash
# Check replica set status
docker exec mongodb mongosh --eval "rs.status()"

# Check replication lag
docker exec mongodb mongosh --eval "rs.printSecondaryReplicationInfo()"

# Force resync
docker exec mongodb mongosh --eval "rs.syncFrom('primary_host')"
```

## Trading Issues

### Orders Not Executing

**Checklist**:
- [ ] Trading enabled in .env (`TRADING_ENABLED=true`)
- [ ] Binance API keys configured
- [ ] Sufficient balance
- [ ] Valid order parameters (price, quantity)
- [ ] Network connectivity to Binance
- [ ] Not in paper trading mode (unless intended)

**Debug**:
```bash
# Check Binance API connectivity
curl https://api.binance.com/api/v3/ping

# Check order validation
docker logs rust-core-engine | grep -i "order\|trade"

# Check Binance API errors
docker logs rust-core-engine | grep -i "binance"
```

### Positions Not Updating

```bash
# Force position refresh
curl -X POST -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/positions/refresh

# Check WebSocket connection
docker logs rust-core-engine | grep -i "websocket.*binance"

# Restart WebSocket connection
docker-compose restart rust-core-engine
```

## Debugging Tools

### Enable Debug Logging

**Rust**:
```bash
# Set environment variable
export RUST_LOG=debug

# Or in docker-compose.yml
environment:
  RUST_LOG: debug
```

**Python**:
```bash
# Edit config.yaml
logging:
  level: DEBUG

# Or environment variable
export LOG_LEVEL=DEBUG
```

### Using Docker Logs

```bash
# Follow logs in real-time
docker logs -f rust-core-engine

# Show last 100 lines
docker logs --tail 100 rust-core-engine

# Show logs since specific time
docker logs --since 2025-10-10T12:00:00 rust-core-engine

# Filter logs
docker logs rust-core-engine 2>&1 | grep ERROR
```

### Database Queries

```bash
# Connect to MongoDB
docker exec -it mongodb mongosh trading_bot

# Check recent trades
db.trades.find().sort({created_at: -1}).limit(10)

# Check active positions
db.positions.find({closed_at: null})

# Check AI analysis
db.ai_analysis.find().sort({created_at: -1}).limit(5)
```

### Network Debugging

```bash
# Check DNS from container
docker exec rust-core-engine nslookup mongodb

# Check connectivity
docker exec rust-core-engine ping mongodb

# Inspect network
docker network inspect bot-network

# Check routing
docker exec rust-core-engine traceroute api.binance.com
```

## Getting Help

### Before Asking for Help

1. **Check Logs**:
   ```bash
   ./scripts/bot.sh logs --service <service-name>
   ```

2. **Check Health**:
   ```bash
   ./scripts/bot.sh status
   curl http://localhost:8080/api/health
   ```

3. **Search Documentation**:
   - [README.md](../README.md)
   - [API Specification](../specs/API_SPEC.md)
   - [Architecture Docs](./architecture/)

4. **Search Issues**:
   - Check GitHub Issues for similar problems
   - Search Discord/Slack channels

### Reporting Issues

Include this information:

```bash
# System info
uname -a
docker --version
docker-compose --version

# Service status
docker ps -a

# Recent logs
docker logs --tail 100 rust-core-engine > rust-logs.txt
docker logs --tail 100 python-ai-service > python-logs.txt

# Config (REMOVE SENSITIVE DATA)
cat .env | sed 's/=.*/=***REDACTED***/'

# Error message (exact copy)
```

### Support Channels

- **GitHub Issues**: Bug reports and feature requests
- **Discord**: Community support
- **Email**: support@botcore.com
- **Documentation**: https://docs.botcore.com

## Emergency Procedures

### System Unresponsive

```bash
# Force stop all containers
docker-compose kill

# Remove all containers
docker-compose down

# Clean restart
docker system prune -a --volumes  # WARNING: Removes all data
./scripts/bot.sh start --memory-optimized
```

### Data Corruption

```bash
# Stop services
./scripts/bot.sh stop

# Restore from backup (if available)
./scripts/restore-backup.sh --date 2025-10-09

# Or reset database
docker volume rm bot-core_mongodb-data  # WARNING: Deletes all data
./scripts/bot.sh start
```

### Trading Emergency Stop

```bash
# Disable trading immediately
curl -X POST -H "Authorization: Bearer $ADMIN_TOKEN" \
  http://localhost:8080/api/admin/disable-trading

# Close all positions
curl -X POST -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/positions/close-all

# Stop services
./scripts/bot.sh stop
```

## Preventive Measures

### Regular Maintenance

```bash
# Weekly: Clean Docker
docker system prune

# Monthly: Backup database
./scripts/backup-database.sh

# Monthly: Update dependencies
git pull
make build
```

### Monitoring Setup

```bash
# Start with monitoring
./scripts/bot.sh start --with-monitoring

# Access Grafana
open http://localhost:3001

# Access Prometheus
open http://localhost:9090
```

### Health Checks

Add to crontab:
```bash
# Check health every 5 minutes
*/5 * * * * curl -f http://localhost:8080/api/health || /path/to/alert.sh
```

---

**Last Updated**: 2025-10-10

For the latest troubleshooting tips, check the [online documentation](https://docs.botcore.com/troubleshooting).
