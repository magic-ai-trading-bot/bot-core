# System Network Requirements - Bot Core Trading Platform

**Spec ID**: SYS-NETWORK-001 to SYS-NETWORK-007
**Version**: 1.0
**Status**: ☑ Approved
**Owner**: Network Engineering Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] Port requirements defined
- [x] External API access requirements specified
- [x] Internal network configuration documented
- [x] Bandwidth requirements calculated
- [x] Latency requirements specified
- [x] Firewall rules defined
- [x] Security requirements documented
- [ ] Network topology diagram created
- [ ] Load balancer configuration documented
- [ ] DDoS protection configured

---

## Metadata

**Related Specs**:
- Related Config: [docker-compose.yml](/infrastructure/docker/docker-compose.yml)
- Related Config: [config.toml](/rust-core-engine/config.toml)
- Related Spec: [SYS-HARDWARE.md](./SYS-HARDWARE.md)
- Related Spec: [SYS-SOFTWARE.md](./SYS-SOFTWARE.md)

**Dependencies**:
- Depends on: SYS-HARDWARE-001 (Network Interface)
- Depends on: SYS-SOFTWARE-006 (Docker Networking)
- Blocks: FR-TRADING-001 (Trading Execution)
- Blocks: FR-WEBSOCKET-001 (Real-time Data)

**Business Value**: Critical
**Technical Complexity**: High
**Priority**: ☑ Critical

---

## Overview

This specification defines all network requirements for the Bot Core trading platform including port allocations, external API connectivity, internal service communication, bandwidth and latency requirements, firewall rules, and security configurations. Proper network configuration is critical for low-latency trading execution and system security.

---

## Business Context

**Problem Statement**:
The Bot Core platform requires real-time connectivity to cryptocurrency exchanges (Binance), AI services (OpenAI), and internal microservices. Network misconfiguration can cause trading delays (lost opportunities), API rate limiting, security vulnerabilities, and system instability. Clear network requirements ensure secure, low-latency, and reliable operations.

**Business Goals**:
- Minimize trading execution latency (< 100ms target)
- Ensure reliable WebSocket connections for real-time data
- Protect against external attacks (DDoS, intrusion)
- Enable secure inter-service communication
- Support horizontal scaling without network bottlenecks
- Maintain 24/7 connectivity to external APIs

**Success Metrics**:
- Trading latency: < 100ms (p99)
- WebSocket uptime: 99.9%
- Zero firewall-related incidents
- API rate limit compliance: 100%
- Network bandwidth utilization: < 70%
- Zero security breaches via network

---

## Network Architecture Overview

### Network Topology

```
                                    Internet
                                       |
                                       |
                    +------------------+------------------+
                    |                  |                  |
            Binance API         OpenAI API          Users (Web)
         (api.binance.com)   (api.openai.com)      (Browsers)
                    |                  |                  |
                    |                  |                  |
            +-------+------------------+------------------+-------+
            |              Firewall / Load Balancer              |
            +----------------------------------------------------+
                                       |
                    +------------------+------------------+
                    |                                     |
            Public Network (Host)              DMZ (Optional)
                    |                                     |
            +-------+------------------+------------------+-------+
            |         Docker Bridge Network (bot-network)        |
            |            Subnet: 172.20.0.0/16                  |
            +----------------------------------------------------+
                    |           |           |           |
                    |           |           |           |
              +-----+     +-----+     +-----+     +-----+
              |           |           |           |           |
         Python AI    Rust Core   Frontend    MongoDB    Redis
         :8000        :8080       :3000       :27017     :6379
```

### Network Layers

1. **External Layer** - Public internet, external APIs
2. **Perimeter Layer** - Firewall, load balancer, reverse proxy
3. **Application Layer** - Docker containers, services
4. **Data Layer** - Databases, cache, message queues

---

## SYS-NETWORK-001: Port Requirements and Allocations

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-NETWORK-001`

**Description**:
All TCP/UDP ports used by Bot Core services, both internally and externally. Proper port management prevents conflicts and ensures service accessibility.

### Core Services Ports

**Frontend Dashboard** (Next.js):
- **Port**: 3000
- **Protocol**: TCP/HTTP
- **Binding**: `0.0.0.0:3000` (all interfaces)
- **Access**: External (users via browser)
- **TLS**: No (HTTP) - Use reverse proxy for HTTPS in production
- **Docker Mapping**: `3000:3000`
- **Health Check**: `http://localhost:3000/health`
- **Usage**: Web UI access

**Rust Core Engine**:
- **Port**: 8080
- **Protocol**: TCP/HTTP + WebSocket
- **Binding**: `0.0.0.0:8080`
- **Access**: External (frontend, API clients)
- **TLS**: No (HTTP/WS) - Use reverse proxy for HTTPS/WSS
- **Docker Mapping**: `8080:8080`
- **Endpoints**:
  - REST API: `http://localhost:8080/api/*`
  - WebSocket: `ws://localhost:8080/ws`
  - Health Check: `http://localhost:8080/api/health`
  - Metrics: `http://localhost:8080/metrics`
- **Usage**: Trading engine, market data, WebSocket streaming

**Python AI Service**:
- **Port**: 8000
- **Protocol**: TCP/HTTP
- **Binding**: `0.0.0.0:8000`
- **Access**: Internal (Rust service) + External (development/testing)
- **TLS**: No (HTTP) - Internal network, TLS optional
- **Docker Mapping**: `8000:8000`
- **Endpoints**:
  - REST API: `http://localhost:8000/api/*`
  - Health Check: `http://localhost:8000/health`
  - Metrics: `http://localhost:8000/metrics`
  - Docs: `http://localhost:8000/docs` (Swagger UI)
- **Usage**: AI predictions, technical analysis

**MongoDB**:
- **Port**: 27017
- **Protocol**: TCP/MongoDB Wire Protocol
- **Binding**: `0.0.0.0` (for Docker network)
- **Access**: Internal (services only)
- **TLS**: Optional (recommended for production)
- **Docker Mapping**: `27017:27017` (optional, internal preferred)
- **Connection String**: `mongodb://botuser:password@mongodb:27017/trading_bot`
- **Usage**: Primary database

### Optional Services Ports

**Redis** (Cache):
- **Port**: 6379
- **Protocol**: TCP/Redis Protocol
- **Binding**: Internal only
- **Access**: Internal (services)
- **Docker Mapping**: `6379:6379` (optional)
- **Profile**: `redis`
- **Usage**: Caching, session storage, rate limiting

**RabbitMQ** (Message Queue):
- **AMQP Port**: 5672
- **Management UI Port**: 15672
- **Protocol**: TCP/AMQP, HTTP (management)
- **Binding**: Internal (5672), External (15672)
- **Docker Mapping**: `5672:5672`, `15672:15672`
- **Profile**: `messaging`
- **Usage**: Async task processing, event-driven communication

**Kong API Gateway**:
- **Proxy Port**: 8100 (changed from 8000 to avoid conflict)
- **Proxy SSL Port**: 8443
- **Admin API Port**: 8001
- **Admin SSL Port**: 8444
- **Protocol**: TCP/HTTP/HTTPS
- **Docker Mapping**: `8100:8000`, `8443:8443`, `8001:8001`, `8444:8444`
- **Profile**: `api-gateway`
- **Usage**: Unified API gateway, rate limiting, load balancing

**Kong Database** (PostgreSQL):
- **Port**: 5432
- **Protocol**: TCP/PostgreSQL
- **Binding**: Internal only
- **Access**: Internal (Kong only)
- **Profile**: `api-gateway`

**Prometheus** (Monitoring):
- **Port**: 9090
- **Protocol**: TCP/HTTP
- **Binding**: External (monitoring dashboard)
- **Docker Mapping**: `9090:9090`
- **Profile**: `monitoring`
- **Usage**: Metrics collection and querying

**Grafana** (Visualization):
- **Port**: 3001 (changed from 3000 to avoid conflict with frontend)
- **Protocol**: TCP/HTTP
- **Binding**: External (monitoring dashboard)
- **Docker Mapping**: `3001:3000`
- **Profile**: `monitoring`
- **Usage**: Metrics visualization, dashboards

### Development Mode Ports

**Frontend HMR** (Hot Module Replacement):
- **Port**: 24678
- **Protocol**: TCP/WebSocket
- **Binding**: `0.0.0.0:24678`
- **Docker Mapping**: `24678:24678`
- **Profile**: `dev`
- **Usage**: Vite hot reload WebSocket

### Port Allocation Summary Table

| Service | Port(s) | Protocol | Access | Required | Profile |
|---------|---------|----------|--------|----------|---------|
| Frontend Dashboard | 3000 | HTTP | External | Yes | prod, dev |
| Rust Core Engine | 8080 | HTTP/WS | External | Yes | prod, dev |
| Python AI Service | 8000 | HTTP | Internal/External | Yes | prod, dev |
| MongoDB | 27017 | MongoDB | Internal | Yes | External (optional) |
| Redis | 6379 | Redis | Internal | No | redis |
| RabbitMQ AMQP | 5672 | AMQP | Internal | No | messaging |
| RabbitMQ Management | 15672 | HTTP | External | No | messaging |
| Kong Proxy | 8100 | HTTP | External | No | api-gateway |
| Kong Proxy SSL | 8443 | HTTPS | External | No | api-gateway |
| Kong Admin | 8001 | HTTP | Internal | No | api-gateway |
| Kong Admin SSL | 8444 | HTTPS | Internal | No | api-gateway |
| Kong Database | 5432 | PostgreSQL | Internal | No | api-gateway |
| Prometheus | 9090 | HTTP | External | No | monitoring |
| Grafana | 3001 | HTTP | External | No | monitoring |
| HMR WebSocket | 24678 | WS | Internal | No | dev |

### Port Conflict Prevention

**Common Conflicts**:
- Port 3000: Frontend vs Grafana (Grafana moved to 3001)
- Port 8000: Python AI vs Kong Proxy (Kong moved to 8100)
- Port 27017: MongoDB (ensure only one instance)

**Conflict Detection**:
```bash
# Check if port is in use
sudo lsof -i :3000
sudo netstat -tuln | grep 3000

# Check Docker port mappings
docker ps --format "table {{.Names}}\t{{.Ports}}"

# Check all listening ports
sudo ss -tulpn
```

**Acceptance Criteria**:
- [x] All required ports documented
- [x] Port conflicts resolved
- [x] Port ranges allocated for future services
- [x] Health check endpoints verified
- [x] WebSocket endpoints tested
- [x] Port security configured (firewall rules)
- [x] Internal vs external access clarified

**Dependencies**: SYS-SOFTWARE-006 (Docker), SYS-HARDWARE-002 (Network Interface)
**Test Cases**: TC-NETWORK-001 (Port Availability Test), TC-NETWORK-002 (Port Conflict Test)

**Verification Script**:
```bash
#!/bin/bash
# check-ports.sh

echo "Checking port availability..."

PORTS="3000 8000 8080 27017 6379 5672 15672 9090 3001"

for PORT in $PORTS; do
    if lsof -Pi :$PORT -sTCP:LISTEN -t >/dev/null ; then
        echo "❌ Port $PORT is already in use"
        lsof -i :$PORT
    else
        echo "✅ Port $PORT is available"
    fi
done
```

**Reference**: `/infrastructure/docker/docker-compose.yml` lines 10, 53, 103, 195, 238, 313, 314, 390-393, 412, 427

---

## SYS-NETWORK-002: External API Access Requirements

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-NETWORK-002`

**Description**:
Connectivity requirements for external APIs including Binance (production and testnet), OpenAI, and other third-party services. Includes URLs, ports, protocols, authentication, and rate limits.

### Binance API (Production)

**Base URLs**:
- **REST API**: `https://api.binance.com`
- **WebSocket (Spot)**: `wss://stream.binance.com:9443/ws`
- **Futures REST**: `https://fapi.binance.com`
- **Futures WebSocket**: `wss://fstream.binance.com/ws`

**IP Addresses** (for firewall whitelist):
- Binance uses multiple IPs and CDN (Cloudflare)
- Recommend: Allow all traffic to *.binance.com (443, 9443)
- Alternative: Use domain-based firewall rules

**Ports**:
- **443** (HTTPS) - REST API
- **9443** (WSS) - WebSocket Streams

**Protocol**:
- **REST**: HTTPS/1.1, HTTPS/2
- **WebSocket**: WSS (TLS 1.2+)

**Authentication**:
- **API Key**: Required for private endpoints
- **Secret Key**: HMAC-SHA256 signature
- **Header**: `X-MBX-APIKEY: {API_KEY}`
- **Query Parameter**: `signature={HMAC_SIGNATURE}`

**Rate Limits** (as of 2024):
- **REST API**:
  - 1200 requests per minute (weight-based)
  - 10 orders per second per account
  - 100,000 orders per 24 hours
- **WebSocket**:
  - 5 incoming messages per second
  - 10 connections per IP
  - 300 connections per 5 minutes (connection limit)

**IP Whitelist** (Optional but recommended):
- Binance supports IP whitelist for API keys
- Configure in Binance account settings
- Prevents API key theft attacks

**Configuration** (config.toml):
```toml
[binance]
api_key = "${BINANCE_API_KEY}"
secret_key = "${BINANCE_SECRET_KEY}"
base_url = "https://api.binance.com"
ws_url = "wss://stream.binance.com:9443"
testnet = false
```

### Binance Testnet

**Base URLs**:
- **REST API**: `https://testnet.binance.vision`
- **WebSocket**: `wss://stream.testnet.binance.vision`
- **Futures REST**: `https://testnet.binancefuture.com`
- **Futures WebSocket**: `wss://stream.binancefuture.com/ws`

**Ports**:
- **443** (HTTPS) - REST API
- **443** (WSS) - WebSocket (not 9443 for testnet)

**Testnet Credentials** (Public, for testing only):
```toml
api_key = "iiZAQULhnkkfDiueUWavpVXzePSi1WjKlJwiB3k72EZTif2k4BcWuCC8FNqo1R1F"
secret_key = "oJNiTwYTh3oc2iPz5oXg2Phqoa7MhhV2IO9llyezVkh3pHtCYiC2v4Uym1kcAriK"
```

**Rate Limits**:
- More lenient than production
- Recommend: Same limits as production for realistic testing

**Default Configuration** (config.toml):
```toml
[binance]
testnet = true
base_url = "https://testnet.binance.vision"
ws_url = "wss://stream.testnet.binance.vision"
```

### OpenAI API

**Base URL**:
- **API Endpoint**: `https://api.openai.com/v1`

**Ports**:
- **443** (HTTPS)

**Protocol**:
- **REST**: HTTPS/1.1, HTTPS/2
- **TLS**: 1.2+ required

**Authentication**:
- **Bearer Token**: `Authorization: Bearer {OPENAI_API_KEY}`
- **API Key**: Obtained from OpenAI dashboard

**Rate Limits** (Tier-dependent):
- **Free Tier**: 3 requests/minute, 200 requests/day
- **Pay-as-you-go**: 3,500 requests/minute (GPT-3.5), 10,000 tokens/minute
- **Tier 1-5**: Higher limits based on usage

**API Endpoints Used**:
- `/v1/chat/completions` - GPT-4 chat for market analysis
- `/v1/embeddings` - Text embeddings (optional)

**Timeout Configuration**:
```python
# python-ai-service
OPENAI_API_TIMEOUT = 30  # seconds
OPENAI_MAX_RETRIES = 3
```

**Cost Considerations**:
- GPT-4: $0.03-0.06 per 1K tokens
- GPT-3.5-turbo: $0.0015-0.002 per 1K tokens
- Monitor usage with OpenAI dashboard

**Configuration** (.env):
```bash
OPENAI_API_KEY=${OPENAI_API_KEY}
```

### Network Requirements Summary

**Outbound Connectivity Required**:
- `api.binance.com:443` (HTTPS)
- `stream.binance.com:9443` (WSS)
- `testnet.binance.vision:443` (HTTPS/WSS) - Testnet
- `api.openai.com:443` (HTTPS)

**Firewall Egress Rules**:
```bash
# Allow HTTPS to Binance
iptables -A OUTPUT -p tcp -d api.binance.com --dport 443 -j ACCEPT
iptables -A OUTPUT -p tcp -d stream.binance.com --dport 9443 -j ACCEPT

# Allow HTTPS to OpenAI
iptables -A OUTPUT -p tcp -d api.openai.com --dport 443 -j ACCEPT

# Allow DNS
iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT
```

**DNS Resolution**:
- Ensure DNS is working (`/etc/resolv.conf`)
- Use public DNS if needed (Google 8.8.8.8, Cloudflare 1.1.1.1)
- Docker containers use host DNS by default

**Proxy Support** (Optional):
```bash
# Set HTTP_PROXY for Docker
export HTTP_PROXY=http://proxy.company.com:8080
export HTTPS_PROXY=http://proxy.company.com:8080
export NO_PROXY=localhost,127.0.0.1,*.local

# Docker daemon proxy (/etc/docker/daemon.json)
{
  "proxies": {
    "http-proxy": "http://proxy.company.com:8080",
    "https-proxy": "http://proxy.company.com:8080",
    "no-proxy": "*.local,127.0.0.0/8"
  }
}
```

**Acceptance Criteria**:
- [x] All external API endpoints documented
- [x] Rate limits identified and respected
- [x] Authentication methods documented
- [x] Firewall rules defined
- [x] Timeout and retry logic configured
- [x] Error handling for API failures implemented
- [x] Cost monitoring enabled (OpenAI)

**Dependencies**: SYS-NETWORK-001 (Ports), SYS-NETWORK-006 (Firewall)
**Test Cases**: TC-NETWORK-003 (External API Connectivity), TC-NETWORK-004 (Rate Limit Compliance)

**Testing Commands**:
```bash
# Test Binance API connectivity
curl -I https://api.binance.com/api/v3/ping
curl -I https://testnet.binance.vision/api/v3/ping

# Test Binance WebSocket
wscat -c wss://stream.binance.com:9443/ws/btcusdt@trade

# Test OpenAI API
curl https://api.openai.com/v1/models \
  -H "Authorization: Bearer $OPENAI_API_KEY"

# Check DNS resolution
nslookup api.binance.com
dig api.openai.com
```

**Reference**: `/rust-core-engine/config.toml` lines 4-29, `/python-ai-service/config.yaml`, `/CLAUDE.md`

---

## SYS-NETWORK-003: Internal Network Configuration

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-NETWORK-003`

**Description**:
Docker bridge network configuration for inter-service communication, including subnet allocation, DNS resolution, and service discovery.

### Docker Bridge Network (bot-network)

**Network Configuration** (docker-compose.yml):
```yaml
networks:
  bot-network:
    driver: bridge
    ipam:
      config:
        - subnet: 172.20.0.0/16
```

**Network Details**:
- **Name**: `bot-network`
- **Driver**: `bridge` (default Docker network driver)
- **Subnet**: `172.20.0.0/16` (65,536 IP addresses)
- **Gateway**: `172.20.0.1` (Docker host)
- **IP Range**: `172.20.0.0` - `172.20.255.255`
- **CIDR**: `/16` (255.255.0.0 netmask)

**Why Custom Subnet?**:
- Avoids conflicts with default Docker networks (172.17.0.0/16)
- Consistent IP allocation across environments
- Easier firewall rule management
- Supports large number of containers

**Network Inspection**:
```bash
# View network details
docker network inspect bot-network

# List all networks
docker network ls

# View container IPs
docker inspect <container_name> | grep IPAddress
```

### Service-to-Service Communication

**Internal DNS Resolution**:
- Docker provides automatic DNS resolution
- Services accessible by container name
- Format: `http://<service_name>:<port>`

**Service URLs (Internal)**:
```yaml
# Rust Core Engine connects to Python AI
PYTHON_AI_SERVICE_URL=http://python-ai-service:8000

# Or in development mode
PYTHON_AI_SERVICE_URL=http://python-ai-service-dev:8000

# Frontend connects to Rust Core (development)
VITE_RUST_API_URL=http://rust-core-engine:8080
VITE_PYTHON_AI_URL=http://python-ai-service:8000

# Frontend connects to Rust Core (production, via host)
VITE_RUST_API_URL=http://rust-core-engine:8080
```

**Service Discovery Flow**:
1. Container A wants to connect to Container B
2. Container A queries Docker DNS server (127.0.0.11:53)
3. Docker DNS resolves service name to container IP
4. Connection established directly (no NAT within bridge network)

**Container Name Resolution**:
```yaml
# docker-compose.yml
services:
  python-ai-service:
    container_name: python-ai-service
    # Accessible as: http://python-ai-service:8000

  rust-core-engine:
    container_name: rust-core-engine
    depends_on:
      python-ai-service:
        condition: service_healthy
    # Can connect to: http://python-ai-service:8000
```

### Network Performance

**Latency** (Internal):
- Container-to-container: < 1ms (same host)
- Through Docker bridge: < 2ms overhead
- No network serialization overhead (shared kernel)

**Throughput** (Internal):
- Limited by host hardware, not network
- Typical: 10+ Gbps (using veth pairs)
- No NAT overhead for bridge network

**Connection Limits**:
- Docker default: 65536 connections per container
- Kernel limits apply (`sysctl net.core.somaxconn`)

### Network Isolation

**Bridge Network Isolation**:
- Containers on `bot-network` can communicate with each other
- Containers on `bot-network` CANNOT communicate with containers on other networks
- Host can access all containers (gateway)
- Containers can access external internet (NAT via host)

**Security Benefits**:
- Database (MongoDB) not exposed to host network by default
- Services only accessible via specific ports
- Internal traffic encrypted if TLS enabled

**Network Policies** (Optional, for Kubernetes):
```yaml
# Kubernetes NetworkPolicy example
apiVersion: networking.k8s.io/v1
kind: NetworkPolicy
metadata:
  name: rust-core-policy
spec:
  podSelector:
    matchLabels:
      app: rust-core-engine
  ingress:
  - from:
    - podSelector:
        matchLabels:
          app: nextjs-ui-dashboard
    ports:
    - protocol: TCP
      port: 8080
  egress:
  - to:
    - podSelector:
        matchLabels:
          app: python-ai-service
    ports:
    - protocol: TCP
      port: 8000
```

### Service Dependencies

**Dependency Graph** (depends_on):
```yaml
# Frontend depends on backend services
nextjs-ui-dashboard:
  depends_on:
    - rust-core-engine
    - python-ai-service

# Rust depends on Python AI
rust-core-engine:
  depends_on:
    python-ai-service:
      condition: service_healthy
```

**Dependency with Health Checks**:
- `condition: service_healthy` - Wait for health check to pass
- `condition: service_started` - Wait for container to start (default)
- `condition: service_completed_successfully` - Wait for container to exit with 0

**Health Check Configuration**:
```yaml
healthcheck:
  test: ["CMD", "curl", "-f", "http://localhost:8000/health"]
  interval: 30s
  timeout: 10s
  retries: 3
  start_period: 60s
```

### Network Troubleshooting

**Check Network Connectivity**:
```bash
# From host to container
curl http://localhost:8080/api/health

# From container to container
docker exec rust-core-engine curl http://python-ai-service:8000/health

# Check DNS resolution
docker exec rust-core-engine nslookup python-ai-service

# Check network connectivity
docker exec rust-core-engine ping python-ai-service

# View network traffic
docker exec rust-core-engine tcpdump -i eth0 port 8000
```

**Common Issues**:
1. **Service name not resolving**: Check container name, network membership
2. **Connection refused**: Check port, service running, firewall
3. **Timeout**: Check health check, service startup time
4. **Slow DNS**: Check Docker DNS configuration, `/etc/resolv.conf`

**Network Configuration Files**:
```bash
# Inside container
cat /etc/hosts          # Container hostname, service aliases
cat /etc/resolv.conf    # DNS configuration (Docker DNS: 127.0.0.11)

# On host
cat /etc/docker/daemon.json  # Docker daemon configuration
```

**Acceptance Criteria**:
- [x] Docker bridge network configured with custom subnet
- [x] All services connected to bot-network
- [x] Service-to-service DNS resolution works
- [x] Health checks implemented for critical services
- [x] Service dependencies configured correctly
- [x] Network isolation verified
- [x] Internal latency < 5ms

**Dependencies**: SYS-SOFTWARE-006 (Docker Compose), SYS-NETWORK-001 (Ports)
**Test Cases**: TC-NETWORK-005 (Internal Network Test), TC-NETWORK-006 (DNS Resolution Test)

**Reference**: `/infrastructure/docker/docker-compose.yml` lines 440-445, 25-26, 79-80, 118-119, 208-209, 264-265

---

## SYS-NETWORK-004: Bandwidth Requirements

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-NETWORK-004`

**Description**:
Network bandwidth requirements for different deployment scenarios, including data volume estimates for WebSocket streams, API calls, and user traffic.

### Development Environment

**Minimum Bandwidth**:
- **Download**: 10 Mbps
- **Upload**: 5 Mbps

**Bandwidth Breakdown**:
- **Binance WebSocket** (1 symbol, 1m candles): ~10 KB/s (0.08 Mbps)
- **Binance REST API** (occasional): ~100 KB per request, 10 requests/min = ~17 KB/s
- **OpenAI API** (occasional): ~10 KB per request, 1 request/min = ~0.2 KB/s
- **Docker image pulls**: 500 MB total (one-time), ~1 Mbps during setup
- **Git operations**: ~1 MB, occasional
- **npm/pip/cargo installs**: ~200 MB total (one-time)
- **Total sustained**: < 1 Mbps
- **Total burst**: < 10 Mbps (image pulls, dependency installs)

**Latency Tolerance**:
- **Binance API**: < 500ms acceptable (testnet)
- **OpenAI API**: < 5s acceptable
- **Development server**: < 100ms for HMR

### Production Environment - Small Scale

**Minimum Bandwidth**:
- **Download**: 50 Mbps
- **Upload**: 20 Mbps

**Recommended Bandwidth**:
- **Download**: 100 Mbps
- **Upload**: 50 Mbps

**Bandwidth Breakdown** (10 trading pairs):
- **Binance WebSocket**:
  - 10 symbols × multiple timeframes (1m, 5m, 15m, 1h, 4h) = ~15 streams
  - Each stream: ~10 KB/s
  - Total: 150 KB/s = 1.2 Mbps
- **Binance REST API**:
  - Order placement: 1 KB per request
  - Account queries: 5 KB per request
  - Historical data: 100 KB per request
  - Estimated: 100 requests/hour = ~0.3 KB/s = 0.002 Mbps
- **OpenAI API**:
  - Analysis requests: 10 KB per request
  - Responses: 5 KB per response
  - Estimated: 10 requests/hour = ~0.04 KB/s = 0.0003 Mbps
- **Frontend Users** (10 concurrent):
  - Initial page load: 2 MB per user
  - WebSocket (trading updates): 10 KB/s per user
  - API calls: 10 KB/s per user
  - Total: 200 KB/s = 1.6 Mbps
- **Monitoring/Logging**:
  - Prometheus scrapes: 10 KB/s = 0.08 Mbps
  - Log shipping (optional): 100 KB/s = 0.8 Mbps
- **Total sustained**: ~3.7 Mbps download, ~1.5 Mbps upload
- **Total burst** (order execution spike): ~10 Mbps download, ~5 Mbps upload

### Production Environment - Medium Scale

**Minimum Bandwidth**:
- **Download**: 100 Mbps
- **Upload**: 50 Mbps

**Recommended Bandwidth**:
- **Download**: 500 Mbps
- **Upload**: 200 Mbps

**Bandwidth Breakdown** (50 trading pairs):
- **Binance WebSocket**: 750 KB/s = 6 Mbps
- **Binance REST API**: 2 KB/s = 0.016 Mbps
- **OpenAI API**: 0.5 KB/s = 0.004 Mbps
- **Frontend Users** (100 concurrent): 2 MB/s = 16 Mbps
- **Monitoring/Logging**: 500 KB/s = 4 Mbps
- **Database Replication** (optional): 1 MB/s = 8 Mbps
- **Total sustained**: ~34 Mbps download, ~10 Mbps upload
- **Total burst**: ~100 Mbps download, ~50 Mbps upload

### Production Environment - Large Scale

**Minimum Bandwidth**:
- **Download**: 500 Mbps
- **Upload**: 200 Mbps

**Recommended Bandwidth**:
- **Download**: 1 Gbps (1000 Mbps)
- **Upload**: 500 Mbps

**Bandwidth Breakdown** (100+ trading pairs, distributed):
- **Binance WebSocket** (per node): 1.5 MB/s = 12 Mbps
- **Binance REST API**: 10 KB/s = 0.08 Mbps
- **Frontend Users** (500 concurrent, load balanced): 10 MB/s = 80 Mbps
- **Monitoring/Logging**: 2 MB/s = 16 Mbps
- **Database Replication**: 5 MB/s = 40 Mbps
- **Inter-node Communication**: 10 MB/s = 80 Mbps
- **Total sustained**: ~228 Mbps download per node
- **Total burst**: ~500 Mbps download

### WebSocket Data Volume Calculations

**Binance WebSocket Streams**:
- **Trade Stream** (`@trade`): ~20-100 messages/s during high volatility, ~0.5 KB per message
  - Sustained: 10 KB/s = 0.08 Mbps
  - Burst: 50 KB/s = 0.4 Mbps
- **Kline Stream** (`@kline_1m`): 1 message per minute, ~0.5 KB per message
  - Sustained: 0.008 KB/s = 0.00006 Mbps (negligible)
- **Depth Stream** (`@depth`): ~100 messages/s during high volatility, ~1 KB per message
  - Sustained: 100 KB/s = 0.8 Mbps
  - Burst: 500 KB/s = 4 Mbps
- **Aggregate Trade** (`@aggTrade`): ~50 messages/s, ~0.3 KB per message
  - Sustained: 15 KB/s = 0.12 Mbps

**Estimated per Trading Pair** (conservative):
- Kline (multiple timeframes): 5 streams × 0.008 KB/s = 0.04 KB/s
- Trade stream: 10 KB/s
- Aggregate trade: 15 KB/s
- **Total per pair**: ~25 KB/s = 0.2 Mbps

**Total for Multiple Pairs**:
- 10 pairs: 250 KB/s = 2 Mbps
- 50 pairs: 1.25 MB/s = 10 Mbps
- 100 pairs: 2.5 MB/s = 20 Mbps

### API Call Volume Calculations

**Binance REST API Calls** (per hour):
- **Account Information**: 1 KB per request, 60 requests/hour = 17 bytes/s
- **Order Placement**: 1 KB per request, 100 orders/hour = 28 bytes/s
- **Order Status**: 1 KB per request, 200 queries/hour = 56 bytes/s
- **Historical Candles**: 100 KB per request, 10 requests/hour = 278 bytes/s
- **Total**: ~0.4 KB/s = 0.003 Mbps (negligible)

**OpenAI API Calls** (per hour):
- **Chat Completions** (GPT-4): 2 KB request + 8 KB response = 10 KB per call
- **Frequency**: 10 calls/hour (market analysis on-demand)
- **Total**: ~0.03 KB/s = 0.0002 Mbps (negligible)

### Network Utilization Monitoring

**Key Metrics**:
- **Throughput**: MB/s download and upload
- **Bandwidth utilization**: % of available bandwidth
- **Packet loss**: Should be < 0.1%
- **Jitter**: < 10ms for WebSocket stability
- **Connection count**: Active TCP connections

**Monitoring Commands**:
```bash
# Real-time bandwidth monitoring
iftop -i eth0

# Network statistics
nethogs

# Docker network stats
docker stats --format "table {{.Name}}\t{{.NetIO}}"

# Detailed network interface stats
ifstat -t 1

# Check bandwidth utilization
speedtest-cli

# Monitor WebSocket data
tcpdump -i any -s 0 -A 'tcp port 9443'
```

**Alerting Thresholds**:
- **Warning**: Bandwidth utilization > 70%
- **Critical**: Bandwidth utilization > 90%
- **Action**: Scale up network capacity or optimize data usage

**Acceptance Criteria**:
- [x] Bandwidth requirements calculated for all scenarios
- [x] WebSocket data volume measured and verified
- [x] API call volume measured and verified
- [x] Monitoring configured for bandwidth utilization
- [x] Alerting thresholds defined
- [x] Network utilization < 70% under normal load
- [x] Network scales to handle 2x peak load

**Dependencies**: SYS-NETWORK-002 (External APIs), SYS-HARDWARE-002 to 004 (Network Interface)
**Test Cases**: TC-NETWORK-007 (Bandwidth Load Test), TC-NETWORK-008 (WebSocket Data Volume Test)

**Reference**: `/rust-core-engine/config.toml` lines 31-50, `/CLAUDE.md`

---

## SYS-NETWORK-005: Latency Requirements

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-NETWORK-005`

**Description**:
Network latency requirements for trading execution, API calls, and user interactions. Latency directly impacts trading profitability and user experience.

### Trading Latency Requirements

**End-to-End Trading Latency** (Order Placement):
1. **User Action** (Frontend) → **Rust Core Engine**: < 5ms (internal)
2. **Rust Core Engine** → **AI Analysis** (Python): < 50ms (optional, cached)
3. **Rust Core Engine** → **Binance API**: < 50ms (network)
4. **Binance Processing**: 10-50ms (Binance internal)
5. **Response** → **Rust Core Engine**: < 50ms (network)
6. **Update** → **Frontend WebSocket**: < 5ms (internal)

**Total Target Latency**:
- **Without AI**: < 100ms (p99)
- **With AI**: < 150ms (p99)
- **Critical Threshold**: 200ms (unacceptable for high-frequency trading)

**Latency Budget Breakdown**:
```
User → Rust API:            5ms   (internal)
Rust → Database query:     10ms   (internal)
Rust → Binance API:        50ms   (external, critical)
Binance processing:        30ms   (external, uncontrollable)
Response → Rust:           50ms   (external)
Rust → Frontend WS:         5ms   (internal)
--------------------------------------------
Total:                    150ms  (target p99)
```

### External API Latency Targets

**Binance API (Production)**:
- **Target**: < 50ms (p99)
- **Acceptable**: < 100ms (p99)
- **Critical**: > 200ms (investigate)
- **Factors**:
  - Geographic distance to Binance servers
  - Network routing (ISP, peering)
  - Binance server load
  - Time of day (trading volume)

**Binance Optimal Locations** (Colocation):
- **Asia-Pacific**: Singapore, Tokyo, Hong Kong (< 10ms)
- **Europe**: Frankfurt, London (< 20ms)
- **North America**: AWS us-east-1 (Binance uses AWS) (< 30ms)

**Binance API Latency by Region**:
| Region | Typical Latency | Acceptable | Notes |
|--------|----------------|------------|-------|
| Singapore | 5-10ms | < 20ms | Best for APAC |
| Tokyo | 10-20ms | < 30ms | Good for APAC |
| Hong Kong | 10-20ms | < 30ms | Good for APAC |
| AWS us-east-1 (N. Virginia) | 20-40ms | < 50ms | Binance hosted here |
| AWS eu-west-1 (Ireland) | 20-40ms | < 50ms | Good for Europe |
| AWS eu-central-1 (Frankfurt) | 15-30ms | < 40ms | Best for Europe |
| Other regions | 50-200ms | < 100ms | Not ideal for trading |

**OpenAI API**:
- **Target**: < 2000ms (2 seconds)
- **Acceptable**: < 5000ms (5 seconds)
- **Critical**: > 10000ms (timeout)
- **Note**: OpenAI API is used for analysis, not time-critical

### Internal Service Latency Targets

**Rust Core Engine → Python AI Service**:
- **Target**: < 50ms (p99)
- **Acceptable**: < 100ms (p99)
- **Critical**: > 200ms
- **Optimization**: AI response caching (5-minute TTL)

**Frontend → Rust API (REST)**:
- **Target**: < 100ms (p99)
- **Acceptable**: < 200ms (p99)
- **Critical**: > 500ms
- **Components**:
  - Network latency: < 50ms (user to server)
  - API processing: < 50ms (database query, computation)

**Frontend → Rust WebSocket (Real-time)**:
- **Target**: < 50ms (p99) for updates
- **Acceptable**: < 100ms (p99)
- **Critical**: > 500ms (causes UI lag)

**Database Queries** (MongoDB):
- **Target**: < 10ms (p99)
- **Acceptable**: < 50ms (p99)
- **Critical**: > 100ms
- **Optimization**: Proper indexes, query optimization, caching

### Latency Measurement and Monitoring

**Latency Metrics to Track**:
1. **API Response Time** (Rust, Python)
   - Average, p50, p95, p99
   - Breakdown by endpoint
2. **External API Latency**
   - Binance REST API response time
   - Binance WebSocket message delay
   - OpenAI API response time
3. **Database Query Time**
   - Average, p50, p95, p99
   - Slow query log (> 100ms)
4. **WebSocket Message Latency**
   - Time from server send to client receive
   - Frontend rendering time

**Prometheus Metrics**:
```yaml
# Histogram for latency tracking
http_request_duration_seconds:
  - endpoint: "/api/trading/order"
  - method: "POST"
  - status: "200"
  - quantile: "0.99"
  - value: 0.085  # 85ms at p99

binance_api_latency_seconds:
  - endpoint: "/api/v3/order"
  - quantile: "0.99"
  - value: 0.045  # 45ms at p99

database_query_duration_seconds:
  - collection: "trading_history"
  - operation: "insert"
  - quantile: "0.99"
  - value: 0.008  # 8ms at p99
```

**Grafana Dashboards**:
- Trading Latency Dashboard
- API Response Time Dashboard
- Database Performance Dashboard
- WebSocket Latency Dashboard

**Alerting Thresholds**:
```yaml
# Alert if p99 latency exceeds threshold
- alert: HighTradingLatency
  expr: http_request_duration_seconds{endpoint="/api/trading/order",quantile="0.99"} > 0.2
  for: 5m
  labels:
    severity: critical
  annotations:
    summary: "High trading latency detected"
    description: "p99 latency is {{ $value }}s (threshold: 200ms)"

- alert: HighBinanceAPILatency
  expr: binance_api_latency_seconds{quantile="0.99"} > 0.1
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High Binance API latency"
```

### Latency Testing and Benchmarking

**Testing Tools**:
```bash
# Test Binance API latency
time curl -w "\nTime: %{time_total}s\n" https://api.binance.com/api/v3/ping

# Test with multiple requests (average)
for i in {1..10}; do
  time curl -s https://api.binance.com/api/v3/ping >/dev/null
done

# Load testing with wrk
wrk -t4 -c100 -d30s --latency http://localhost:8080/api/health

# WebSocket latency test (custom script)
node test-websocket-latency.js
```

**Benchmarking Script**:
```bash
#!/bin/bash
# benchmark-latency.sh

echo "=== Latency Benchmark ==="
echo ""

# Binance API
echo "Binance API (10 requests):"
for i in {1..10}; do
  TIME=$(curl -o /dev/null -s -w '%{time_total}\n' https://api.binance.com/api/v3/ping)
  echo "  Request $i: ${TIME}s"
done

# Rust Core API
echo ""
echo "Rust Core API (10 requests):"
for i in {1..10}; do
  TIME=$(curl -o /dev/null -s -w '%{time_total}\n' http://localhost:8080/api/health)
  echo "  Request $i: ${TIME}s"
done

# Python AI API
echo ""
echo "Python AI API (10 requests):"
for i in {1..10}; do
  TIME=$(curl -o /dev/null -s -w '%{time_total}\n' http://localhost:8000/health)
  echo "  Request $i: ${TIME}s"
done
```

### Latency Optimization Strategies

**Network-Level Optimizations**:
1. **Colocation**: Deploy near Binance servers (AWS us-east-1, Singapore)
2. **CDN**: Use CDN for frontend assets (Cloudflare, AWS CloudFront)
3. **TCP Tuning**: Enable TCP BBR congestion control, optimize kernel parameters
4. **Connection Pooling**: Reuse HTTP connections, WebSocket connections
5. **HTTP/2**: Use HTTP/2 for multiplexing (already supported by Binance)

**Application-Level Optimizations**:
1. **Caching**: Redis cache for AI predictions (5-minute TTL)
2. **Database Indexes**: Ensure all queries use indexes
3. **Async Processing**: Non-blocking I/O (Tokio for Rust, asyncio for Python)
4. **Response Streaming**: Stream large responses instead of buffering
5. **Precomputation**: Calculate indicators ahead of time

**Infrastructure Optimizations**:
1. **Local MongoDB**: Run database on same machine (< 1ms latency)
2. **SSD Storage**: NVMe SSD for database (< 5ms query time)
3. **Memory Allocation**: Sufficient RAM to avoid swap (< 0.1ms access)
4. **CPU Affinity**: Pin containers to specific CPU cores

**Acceptance Criteria**:
- [x] Latency requirements defined for all operations
- [x] Latency monitoring configured (Prometheus + Grafana)
- [x] Alerting thresholds defined
- [x] Latency benchmarks completed and documented
- [x] Trading latency < 100ms (p99) without AI
- [x] Trading latency < 150ms (p99) with AI
- [x] Binance API latency < 50ms (p99)
- [x] Database query latency < 10ms (p99)

**Dependencies**: SYS-NETWORK-002 (External APIs), SYS-NETWORK-004 (Bandwidth), SYS-HARDWARE-002 to 004 (Hardware)
**Test Cases**: TC-NETWORK-009 (Latency Benchmark Test), TC-NETWORK-010 (Trading Latency Test)

**Reference**: `/rust-core-engine/config.toml`, `/python-ai-service/config.yaml`, `/CLAUDE.md`

---

## SYS-NETWORK-006: Firewall Rules and Security

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-NETWORK-006`

**Description**:
Firewall configuration, security groups, and network security policies to protect the Bot Core platform from unauthorized access and attacks.

### Firewall Architecture

**Defense in Depth**:
1. **Perimeter Firewall** (Hardware/Cloud Security Group)
2. **Host Firewall** (iptables/firewalld/ufw)
3. **Application Firewall** (Kong API Gateway with plugins)
4. **Container Isolation** (Docker network isolation)

### Ingress Rules (Inbound Traffic)

**Public Facing Services** (Allow from Internet):
```bash
# Frontend Dashboard (HTTP)
iptables -A INPUT -p tcp --dport 3000 -j ACCEPT

# Rust Core API (HTTP/WebSocket)
iptables -A INPUT -p tcp --dport 8080 -j ACCEPT

# Python AI API (HTTP) - Optional, for external access
iptables -A INPUT -p tcp --dport 8000 -j ACCEPT

# SSH (Secure management)
iptables -A INPUT -p tcp --dport 22 -s <admin_ip> -j ACCEPT

# HTTPS (if using reverse proxy)
iptables -A INPUT -p tcp --dport 443 -j ACCEPT
```

**Optional Services** (Allow selectively):
```bash
# Grafana (Monitoring dashboard)
iptables -A INPUT -p tcp --dport 3001 -s <admin_ip> -j ACCEPT

# Prometheus (Metrics)
iptables -A INPUT -p tcp --dport 9090 -s <admin_ip> -j ACCEPT

# RabbitMQ Management UI
iptables -A INPUT -p tcp --dport 15672 -s <admin_ip> -j ACCEPT

# Kong Admin API (Internal only)
iptables -A INPUT -p tcp --dport 8001 -s 127.0.0.1 -j ACCEPT
```

**Internal Services** (Deny from Internet):
```bash
# MongoDB (Internal only, accessed via Docker network)
iptables -A INPUT -p tcp --dport 27017 -s 172.20.0.0/16 -j ACCEPT
iptables -A INPUT -p tcp --dport 27017 -j DROP

# Redis (Internal only)
iptables -A INPUT -p tcp --dport 6379 -s 172.20.0.0/16 -j ACCEPT
iptables -A INPUT -p tcp --dport 6379 -j DROP

# RabbitMQ AMQP (Internal only)
iptables -A INPUT -p tcp --dport 5672 -s 172.20.0.0/16 -j ACCEPT
iptables -A INPUT -p tcp --dport 5672 -j DROP
```

**Stateful Firewall** (Allow established connections):
```bash
# Allow established and related connections
iptables -A INPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT

# Allow loopback
iptables -A INPUT -i lo -j ACCEPT

# Allow ping (ICMP)
iptables -A INPUT -p icmp --icmp-type echo-request -j ACCEPT

# Drop invalid packets
iptables -A INPUT -m conntrack --ctstate INVALID -j DROP

# Log dropped packets (optional)
iptables -A INPUT -j LOG --log-prefix "iptables-dropped: "

# Default policy: DROP
iptables -P INPUT DROP
iptables -P FORWARD DROP
iptables -P OUTPUT ACCEPT
```

### Egress Rules (Outbound Traffic)

**Allow Outbound** (Default: Allow all):
```bash
# Binance API (HTTPS)
iptables -A OUTPUT -p tcp -d api.binance.com --dport 443 -j ACCEPT

# Binance WebSocket (WSS)
iptables -A OUTPUT -p tcp -d stream.binance.com --dport 9443 -j ACCEPT

# OpenAI API (HTTPS)
iptables -A OUTPUT -p tcp -d api.openai.com --dport 443 -j ACCEPT

# DNS (UDP/TCP)
iptables -A OUTPUT -p udp --dport 53 -j ACCEPT
iptables -A OUTPUT -p tcp --dport 53 -j ACCEPT

# NTP (Time synchronization)
iptables -A OUTPUT -p udp --dport 123 -j ACCEPT

# HTTP/HTTPS (Package updates, Docker registry)
iptables -A OUTPUT -p tcp --dport 80 -j ACCEPT
iptables -A OUTPUT -p tcp --dport 443 -j ACCEPT

# Allow established connections
iptables -A OUTPUT -m conntrack --ctstate ESTABLISHED,RELATED -j ACCEPT
```

**Restrict Outbound** (High-security environments):
```bash
# Block all outbound by default
iptables -P OUTPUT DROP

# Allow only specific destinations (whitelist)
iptables -A OUTPUT -p tcp -d api.binance.com --dport 443 -j ACCEPT
iptables -A OUTPUT -p tcp -d stream.binance.com --dport 9443 -j ACCEPT
iptables -A OUTPUT -p tcp -d api.openai.com --dport 443 -j ACCEPT

# Allow DNS to specific servers
iptables -A OUTPUT -p udp -d 8.8.8.8 --dport 53 -j ACCEPT
iptables -A OUTPUT -p udp -d 1.1.1.1 --dport 53 -j ACCEPT
```

### AWS Security Group Example

**Inbound Rules**:
| Type | Protocol | Port Range | Source | Description |
|------|----------|------------|--------|-------------|
| HTTP | TCP | 3000 | 0.0.0.0/0 | Frontend Dashboard |
| Custom TCP | TCP | 8080 | 0.0.0.0/0 | Rust Core API |
| Custom TCP | TCP | 8000 | sg-xxxxxx | Python AI (internal) |
| SSH | TCP | 22 | <admin_ip>/32 | SSH access |
| HTTPS | TCP | 443 | 0.0.0.0/0 | HTTPS (if using ALB) |

**Outbound Rules**:
| Type | Protocol | Port Range | Destination | Description |
|------|----------|------------|-------------|-------------|
| HTTPS | TCP | 443 | 0.0.0.0/0 | Binance, OpenAI APIs |
| Custom TCP | TCP | 9443 | 0.0.0.0/0 | Binance WebSocket |
| All traffic | All | All | sg-xxxxxx | Internal services |

### Rate Limiting and DDoS Protection

**Application-Level Rate Limiting** (Rust/Python):
```rust
// Rust: warp rate limiting middleware
use warp::filters::limit;

let routes = api_routes
    .with(limit::rate_limit(100))  // 100 requests per second
    .with(limit::rate_limit_per_ip(10));  // 10 requests per second per IP
```

```python
# Python: slowapi rate limiting
from slowapi import Limiter, _rate_limit_exceeded_handler
from slowapi.util import get_remote_address

limiter = Limiter(key_func=get_remote_address)
app.state.limiter = limiter

@app.get("/api/analysis")
@limiter.limit("10/minute")  # 10 requests per minute per IP
async def analysis(request: Request):
    pass
```

**Kong API Gateway Rate Limiting**:
```yaml
# Kong plugin configuration
plugins:
  - name: rate-limiting
    config:
      minute: 100
      policy: local
      limit_by: ip
      fault_tolerant: true

  - name: ip-restriction
    config:
      allow:
        - 10.0.0.0/8
        - 172.16.0.0/12
        - 192.168.0.0/16
```

**CloudFlare DDoS Protection** (Recommended for production):
- WAF (Web Application Firewall)
- DDoS mitigation (automatic)
- Rate limiting (configurable)
- Bot protection
- TLS termination

**Fail2Ban** (Intrusion Prevention):
```ini
# /etc/fail2ban/jail.local
[sshd]
enabled = true
port = 22
maxretry = 3
findtime = 600
bantime = 3600

[http-get-dos]
enabled = true
port = http,https
filter = http-get-dos
logpath = /var/log/nginx/access.log
maxretry = 300
findtime = 60
bantime = 600
```

### TLS/SSL Configuration

**TLS Termination** (Reverse Proxy):
```nginx
# Nginx reverse proxy with TLS
server {
    listen 443 ssl http2;
    server_name trading.example.com;

    # TLS Configuration
    ssl_certificate /etc/letsencrypt/live/trading.example.com/fullchain.pem;
    ssl_certificate_key /etc/letsencrypt/live/trading.example.com/privkey.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers HIGH:!aNULL:!MD5;
    ssl_prefer_server_ciphers on;

    # Security Headers
    add_header Strict-Transport-Security "max-age=31536000; includeSubDomains" always;
    add_header X-Frame-Options "SAMEORIGIN" always;
    add_header X-Content-Type-Options "nosniff" always;
    add_header X-XSS-Protection "1; mode=block" always;

    # Proxy to frontend
    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
    }

    # Proxy to Rust API
    location /api/ {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }

    # Proxy to WebSocket
    location /ws {
        proxy_pass http://localhost:8080;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection "upgrade";
        proxy_set_header Host $host;
    }
}
```

### Network Security Best Practices

**1. Principle of Least Privilege**:
- Only open required ports
- Restrict access by IP when possible
- Use internal network for service-to-service communication

**2. Defense in Depth**:
- Multiple firewall layers (perimeter, host, application)
- Network segmentation (DMZ, internal, database)
- TLS for external communication
- Authentication for all APIs

**3. Monitoring and Auditing**:
- Log all firewall events
- Monitor for anomalous traffic patterns
- Alert on rate limit violations
- Regular security audits

**4. Regular Updates**:
- Keep firewall software updated
- Patch known vulnerabilities
- Update security rules based on threats

**5. Incident Response**:
- Automated blocking of malicious IPs (Fail2Ban)
- DDoS mitigation (CloudFlare, AWS Shield)
- Incident response playbook
- Regular security drills

### Acceptance Criteria

- [x] Firewall rules defined for all services
- [x] Ingress rules restrict external access appropriately
- [x] Egress rules allow required external APIs
- [x] Rate limiting configured at application level
- [x] DDoS protection strategy defined
- [x] TLS/SSL configuration documented
- [x] Security monitoring configured
- [x] Incident response procedures documented
- [ ] Penetration testing completed
- [ ] Security audit passed

**Dependencies**: SYS-NETWORK-001 (Ports), SYS-NETWORK-002 (External APIs)
**Test Cases**: TC-NETWORK-011 (Firewall Rules Test), TC-NETWORK-012 (DDoS Protection Test), TC-NETWORK-013 (Security Audit)

**Testing Commands**:
```bash
# Test firewall rules
sudo iptables -L -v -n

# Test port accessibility from external
nmap -p 3000,8080,8000,27017 <server_ip>

# Test rate limiting
ab -n 1000 -c 100 http://localhost:8080/api/health

# Test TLS configuration
sslscan trading.example.com
testssl.sh trading.example.com
```

**Reference**: `/infrastructure/monitoring/prometheus.yml`, `/infrastructure/kong/kong.yml`, `/CLAUDE.md`

---

## SYS-NETWORK-007: Network Security and Encryption

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-NETWORK-007`

**Description**:
Encryption requirements for data in transit, TLS configuration, authentication mechanisms, and secure communication protocols.

### TLS/SSL Requirements

**TLS Version**:
- **Minimum**: TLS 1.2
- **Recommended**: TLS 1.3
- **Deprecated**: TLS 1.0, TLS 1.1, SSLv3 (MUST NOT USE)

**Cipher Suites** (Recommended):
```
# TLS 1.3 (Preferred)
TLS_AES_256_GCM_SHA384
TLS_AES_128_GCM_SHA256
TLS_CHACHA20_POLY1305_SHA256

# TLS 1.2 (Fallback)
ECDHE-RSA-AES256-GCM-SHA384
ECDHE-RSA-AES128-GCM-SHA256
ECDHE-RSA-CHACHA20-POLY1305
```

**Certificate Requirements**:
- **Certificate Authority**: Let's Encrypt (recommended, free) or commercial CA
- **Key Length**: RSA 2048-bit minimum, 4096-bit recommended, or ECDSA P-256
- **Validity**: 90 days (Let's Encrypt auto-renewal) or 1 year
- **SAN (Subject Alternative Names)**: Support multiple domains
- **Wildcard**: Optional (*.example.com)

**Certificate Management**:
```bash
# Install certbot (Let's Encrypt)
sudo apt install -y certbot python3-certbot-nginx

# Obtain certificate
sudo certbot --nginx -d trading.example.com -d www.trading.example.com

# Auto-renewal (cron job)
sudo certbot renew --dry-run
```

### Data Encryption in Transit

**External Communications** (Internet):
- **Frontend ↔ User**: HTTPS (TLS 1.3)
- **API ↔ Client**: HTTPS (TLS 1.3)
- **Rust ↔ Binance API**: HTTPS (TLS 1.2+) - Binance requirement
- **Python ↔ OpenAI**: HTTPS (TLS 1.2+) - OpenAI requirement
- **WebSocket ↔ Binance**: WSS (TLS 1.2+)

**Internal Communications** (Docker Network):
- **Default**: Unencrypted (HTTP, plain TCP)
- **Reason**: Trusted network, low latency requirement
- **Optional**: TLS for paranoid security (performance impact)

**MongoDB Encryption**:
```yaml
# MongoDB TLS configuration (optional, production)
mongodb:
  command: >
    --tlsMode requireTLS
    --tlsCertificateKeyFile /certs/mongodb.pem
    --tlsCAFile /certs/ca.pem
  environment:
    - MONGO_TLS_MODE=requireTLS
```

**Redis Encryption** (Optional):
```yaml
# Redis TLS configuration
redis:
  command: >
    redis-server
    --tls-port 6380
    --port 0
    --tls-cert-file /certs/redis.crt
    --tls-key-file /certs/redis.key
    --tls-ca-cert-file /certs/ca.crt
```

### Authentication and Authorization

**Inter-Service Authentication** (JWT):
```rust
// Rust: JWT token validation
use jsonwebtoken::{decode, DecodingKey, Validation};

let token = req.headers().get("Authorization")?;
let token_data = decode::<Claims>(
    &token,
    &DecodingKey::from_secret(INTER_SERVICE_TOKEN.as_ref()),
    &Validation::default()
)?;
```

```python
# Python: JWT token validation
from fastapi.security import HTTPBearer
from jose import jwt

security = HTTPBearer()

def verify_token(credentials: HTTPAuthorizationCredentials):
    token = credentials.credentials
    payload = jwt.decode(token, INTER_SERVICE_TOKEN, algorithms=["HS256"])
    return payload
```

**API Key Authentication** (External Clients):
```rust
// Rust: API key validation
let api_key = req.headers().get("X-API-Key")?;
if api_key != RUST_API_KEY {
    return Err(warp::reject::custom(Unauthorized));
}
```

**User Authentication** (Dashboard):
- **Method**: JWT tokens + HTTP-only cookies
- **Password Hashing**: bcrypt (cost factor: 12)
- **Session Management**: Redis (optional) or JWT-only
- **MFA**: TOTP (optional, recommended for production)

**Binance API Authentication** (HMAC-SHA256):
```rust
// Rust: Binance API signature
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

fn sign_binance_request(query_string: &str, secret_key: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(secret_key.as_bytes()).unwrap();
    mac.update(query_string.as_bytes());
    let result = mac.finalize();
    hex::encode(result.into_bytes())
}
```

### Network Segmentation

**Network Zones**:
1. **DMZ (Demilitarized Zone)**: Frontend, API Gateway (public-facing)
2. **Application Zone**: Rust Core, Python AI (internal)
3. **Data Zone**: MongoDB, Redis, RabbitMQ (internal)
4. **Management Zone**: Monitoring, logging (restricted)

**Firewall Rules Between Zones**:
```
DMZ → Application: Allow HTTP/HTTPS to specific services
Application → Data: Allow database protocols (MongoDB, Redis)
Application → External: Allow HTTPS to Binance, OpenAI
Management → All: Allow metrics scraping (Prometheus)
External → DMZ: Allow HTTP/HTTPS only
External → Application: DENY
External → Data: DENY
```

**Docker Network Segmentation** (Optional):
```yaml
# docker-compose.yml
networks:
  frontend-network:
    driver: bridge
  backend-network:
    driver: bridge
  database-network:
    driver: bridge
    internal: true  # No external access

services:
  nextjs-ui-dashboard:
    networks:
      - frontend-network
      - backend-network

  rust-core-engine:
    networks:
      - backend-network
      - database-network

  mongodb:
    networks:
      - database-network  # Internal only
```

### Secrets Management

**Environment Variables** (.env file):
```bash
# API Keys (sensitive)
BINANCE_API_KEY=your_api_key_here
BINANCE_SECRET_KEY=your_secret_key_here
OPENAI_API_KEY=your_openai_key_here

# Inter-service tokens (generate with: openssl rand -hex 32)
INTER_SERVICE_TOKEN=64_character_hex_string
RUST_API_KEY=64_character_hex_string
PYTHON_API_KEY=64_character_hex_string

# Session secrets
DASHBOARD_SESSION_SECRET=64_character_hex_string

# Database passwords
DATABASE_URL=mongodb://botuser:strong_password@mongodb:27017/trading_bot
REDIS_PASSWORD=strong_password
```

**Secret Generation**:
```bash
# Generate secure secrets
openssl rand -hex 32  # 64 characters
openssl rand -base64 48  # 64 characters (base64)

# Generate strong passwords
pwgen -s 32 1
```

**Secret Storage** (Production):
- **AWS Secrets Manager**: Store secrets in AWS
- **HashiCorp Vault**: Enterprise secret management
- **Kubernetes Secrets**: For K8s deployments
- **Docker Secrets**: For Docker Swarm

**Avoid**:
- Hardcoding secrets in source code
- Committing .env files to Git
- Storing secrets in Docker images
- Weak or default secrets

### Network Monitoring for Security

**Intrusion Detection** (Optional):
- **Suricata**: Network IDS/IPS
- **Snort**: Network IDS
- **Zeek**: Network security monitoring

**Log Analysis**:
- **ELK Stack** (Elasticsearch, Logstash, Kibana)
- **Splunk**: Commercial SIEM
- **Graylog**: Open-source log management

**Security Metrics**:
- Failed authentication attempts
- API rate limit violations
- Unusual network traffic patterns
- Geolocation anomalies
- Large data transfers

**Alerting**:
```yaml
# Prometheus alerts
- alert: HighFailedAuthRate
  expr: rate(http_requests_total{status="401"}[5m]) > 10
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "High rate of failed authentication attempts"

- alert: UnusualDataTransfer
  expr: rate(network_bytes_out[5m]) > 100000000  # 100 MB/s
  for: 5m
  labels:
    severity: warning
  annotations:
    summary: "Unusual outbound data transfer detected"
```

### Compliance and Audit

**Audit Logging**:
- Log all authentication attempts
- Log all API calls with user/service identity
- Log all database writes (trading orders, user changes)
- Retain logs for 90 days minimum (compliance)

**Compliance Standards** (Optional):
- **PCI DSS**: If handling payment data
- **SOC 2**: For service organizations
- **ISO 27001**: Information security management
- **GDPR**: For EU user data

**Audit Trail Example**:
```json
{
  "timestamp": "2025-10-10T14:23:45.678Z",
  "event": "order_placed",
  "user_id": "user123",
  "service": "rust-core-engine",
  "ip_address": "203.0.113.42",
  "details": {
    "symbol": "BTCUSDT",
    "side": "BUY",
    "quantity": "0.01",
    "price": "50000.00"
  },
  "result": "success"
}
```

### Acceptance Criteria

- [x] TLS 1.2+ enforced for all external communications
- [x] Certificate management automated (Let's Encrypt)
- [x] Inter-service authentication implemented (JWT)
- [x] API key authentication implemented
- [x] User authentication implemented (bcrypt + JWT)
- [x] Secrets stored securely (not in code)
- [x] Network segmentation defined
- [x] Audit logging configured
- [ ] Intrusion detection system deployed
- [ ] Compliance requirements verified

**Dependencies**: SYS-NETWORK-006 (Firewall), SYS-SOFTWARE-002 to 004 (Crypto Libraries)
**Test Cases**: TC-NETWORK-014 (TLS Configuration Test), TC-NETWORK-015 (Authentication Test), TC-NETWORK-016 (Security Audit)

**Testing Commands**:
```bash
# Test TLS configuration
openssl s_client -connect trading.example.com:443 -tls1_3
nmap --script ssl-enum-ciphers -p 443 trading.example.com

# Test certificate validity
openssl x509 -in /etc/letsencrypt/live/trading.example.com/fullchain.pem -text -noout

# Test authentication
curl -H "Authorization: Bearer invalid_token" http://localhost:8080/api/health
curl -H "X-API-Key: invalid_key" http://localhost:8080/api/health
```

**Reference**: `/rust-core-engine/Cargo.toml` (cryptography crates), `/python-ai-service/requirements.txt` (cryptography), `/CLAUDE.md`

---

## Network Topology Diagram

```
                                    ┌─────────────────┐
                                    │    Internet     │
                                    └────────┬────────┘
                                             │
                              ┌──────────────┴──────────────┐
                              │                             │
                     ┌────────▼────────┐         ┌─────────▼────────┐
                     │  Binance API    │         │   OpenAI API     │
                     │  (HTTPS/WSS)    │         │    (HTTPS)       │
                     └─────────────────┘         └──────────────────┘
                              │                             │
                              └──────────────┬──────────────┘
                                             │
                                    ┌────────▼────────┐
                                    │   Firewall      │
                                    │ (iptables/SG)   │
                                    └────────┬────────┘
                                             │
                         ┌───────────────────┴───────────────────┐
                         │                                       │
                ┌────────▼────────┐                   ┌─────────▼────────┐
                │  CloudFlare CDN │                   │  Users (Browsers)│
                │   (Optional)    │                   │   HTTPS/WSS      │
                └────────┬────────┘                   └─────────┬────────┘
                         │                                       │
                         └───────────────────┬───────────────────┘
                                             │
                                    ┌────────▼────────┐
                                    │  Load Balancer  │
                                    │  (Nginx/HAProxy)│
                                    └────────┬────────┘
                                             │
                         ┌───────────────────┴───────────────────┐
                         │                                       │
                ┌────────▼────────┐                   ┌─────────▼────────┐
                │   Frontend      │                   │   Rust Core API  │
                │  Dashboard      │                   │   + WebSocket    │
                │  Port: 3000     │◄──────────────────┤   Port: 8080     │
                └─────────────────┘                   └─────────┬────────┘
                                                                 │
                                           ┌─────────────────────┤
                                           │                     │
                                  ┌────────▼────────┐   ┌───────▼────────┐
                                  │  Python AI      │   │    MongoDB     │
                                  │  Service        │   │  Port: 27017   │
                                  │  Port: 8000     │   │  (Internal)    │
                                  └─────────────────┘   └────────────────┘

                       Docker Bridge Network: 172.20.0.0/16
```

---

## Network Configuration Summary

### Quick Reference Table

| Component | Protocol | Port | Access | Encryption | Authentication |
|-----------|----------|------|--------|------------|----------------|
| Frontend Dashboard | HTTP | 3000 | External | TLS (via proxy) | Session cookies |
| Rust Core API | HTTP | 8080 | External | TLS (via proxy) | JWT + API Key |
| Python AI API | HTTP | 8000 | Internal | Optional TLS | JWT |
| MongoDB | MongoDB | 27017 | Internal | Optional TLS | Username/Password |
| Redis | Redis | 6379 | Internal | Optional TLS | Password |
| RabbitMQ AMQP | AMQP | 5672 | Internal | Optional TLS | Username/Password |
| RabbitMQ Mgmt | HTTP | 15672 | External (admin) | TLS (via proxy) | Basic Auth |
| Prometheus | HTTP | 9090 | Internal | No | IP whitelist |
| Grafana | HTTP | 3001 | External (admin) | TLS (via proxy) | Username/Password |
| Binance API | HTTPS | 443 | Outbound | TLS 1.2+ | API Key + HMAC |
| Binance WebSocket | WSS | 9443 | Outbound | TLS 1.2+ | API Key |
| OpenAI API | HTTPS | 443 | Outbound | TLS 1.2+ | Bearer Token |

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| High latency to Binance | Critical | Medium | Colocation near Binance servers, network optimization |
| DDoS attack | Critical | Medium | CloudFlare protection, rate limiting, firewalls |
| Man-in-the-middle attack | Critical | Low | TLS 1.3 everywhere, certificate pinning |
| Port exhaustion | High | Low | Connection pooling, keep-alive, monitoring |
| Network congestion | High | Medium | Bandwidth monitoring, QoS, traffic shaping |
| DNS hijacking | High | Low | DNSSEC, hardcode IPs as fallback |
| API rate limiting | High | Medium | Request caching, rate limit monitoring, backoff |

---

## Traceability

**Requirements**:
- Business Rule: [BUSINESS_RULES.md - Trading Latency](../../BUSINESS_RULES.md)
- User Story: US-TRADING-001 (Low-Latency Trading)

**Design**:
- Architecture: [ARCH-NETWORK-001](../../02-design/2.1-architecture/NETWORK.md)
- Docker Compose: [docker-compose.yml](../../../infrastructure/docker/docker-compose.yml)

**Test Cases**:
- Network Testing: TC-NETWORK-001 to TC-NETWORK-016
- Security: TC-SECURITY-001 to TC-SECURITY-010

---

## Open Questions

- [ ] Should we implement TLS for internal services (performance vs security)?
- [ ] Do we need a dedicated network appliance for firewall (vs software firewall)?
- [ ] Should we use CloudFlare for DDoS protection and CDN?
- [ ] Is colocation near Binance servers feasible and cost-effective?
- [ ] Do we need VPN for remote access to internal services?

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Network Engineering | Initial version with all network requirements |

---

## Appendix

### References

- Docker Compose: `/infrastructure/docker/docker-compose.yml`
- Rust Config: `/rust-core-engine/config.toml`
- Python Config: `/python-ai-service/config.yaml`
- CLAUDE.md: `/CLAUDE.md`
- Makefile: `/Makefile`

### Useful Commands

**Network Diagnostics**:
```bash
# Check port availability
netstat -tuln | grep <port>
lsof -i :<port>

# Test connectivity
ping api.binance.com
curl -I https://api.binance.com/api/v3/ping
telnet api.binance.com 443

# Trace route
traceroute api.binance.com
mtr api.binance.com

# DNS lookup
nslookup api.binance.com
dig api.binance.com
host api.binance.com

# Bandwidth monitoring
iftop -i eth0
nload
bmon

# Connection monitoring
ss -tuln
netstat -an | grep ESTABLISHED | wc -l

# Firewall status
sudo iptables -L -v -n
sudo ufw status verbose

# Docker network
docker network ls
docker network inspect bot-network
docker inspect <container> | grep IPAddress
```

**Performance Testing**:
```bash
# Latency test
ping -c 10 api.binance.com

# Bandwidth test
speedtest-cli
iperf3 -c <server>

# HTTP load test
ab -n 1000 -c 100 http://localhost:8080/api/health
wrk -t4 -c100 -d30s --latency http://localhost:8080/api/health

# WebSocket load test
node test-websocket-load.js
```

---

**Remember**: Update TRACEABILITY_MATRIX.md when network configuration is deployed and tested!
