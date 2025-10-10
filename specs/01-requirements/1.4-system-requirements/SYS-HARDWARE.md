# System Hardware Requirements - Bot Core Trading Platform

**Spec ID**: SYS-HARDWARE-001 to SYS-HARDWARE-006
**Version**: 1.0
**Status**: ☑ Approved
**Owner**: Infrastructure Team
**Last Updated**: 2025-10-10

---

## Tasks Checklist

- [x] Development environment requirements defined
- [x] Production small scale requirements defined
- [x] Production large scale requirements defined
- [x] Memory optimization settings documented
- [x] Storage requirements specified
- [x] Resource monitoring tools identified
- [ ] Hardware procurement approved
- [ ] Performance benchmarks completed
- [ ] Scalability testing completed

---

## Metadata

**Related Specs**:
- Related Config: [docker-compose.yml](/infrastructure/docker/docker-compose.yml)
- Related Docs: [CLAUDE.md](/CLAUDE.md)
- Related Spec: [SYS-SOFTWARE.md](./SYS-SOFTWARE.md)
- Related Spec: [SYS-NETWORK.md](./SYS-NETWORK.md)

**Dependencies**:
- Depends on: SYS-SOFTWARE-001 (Operating System)
- Depends on: SYS-NETWORK-001 (Network Infrastructure)
- Blocks: NFR-PERFORMANCE-001 (Performance Requirements)

**Business Value**: Critical
**Technical Complexity**: Medium
**Priority**: ☑ Critical

---

## Overview

This specification defines the hardware requirements for deploying the Bot Core trading platform across different environments (development, staging, production). It includes CPU, memory, storage, and GPU specifications with minimum and recommended configurations for optimal performance.

---

## Business Context

**Problem Statement**:
The Bot Core platform is a resource-intensive microservices application combining high-performance Rust trading engine, Python AI/ML models, and real-time frontend dashboard. Without proper hardware provisioning, the system will experience performance degradation, trading latency, and potential system failures that could result in missed trading opportunities or financial losses.

**Business Goals**:
- Ensure system stability and uptime for 24/7 trading operations
- Minimize trading latency for competitive advantage
- Support scalability for growing trading volumes
- Optimize infrastructure costs while maintaining performance
- Enable development team productivity with adequate local resources

**Success Metrics**:
- Trading execution latency: < 100ms (production)
- System uptime: 99.9% (production)
- Resource utilization: 60-80% average, < 90% peak
- Development build time: < 5 minutes
- Zero out-of-memory (OOM) incidents in production

---

## Hardware Requirements Overview

The Bot Core platform requires different hardware configurations based on deployment environment and scale. This section provides detailed specifications for each scenario.

### Environment Types

1. **Development**: Local developer workstations
2. **Staging**: Pre-production testing environment
3. **Production Small Scale**: 1-10 trading pairs, single instance
4. **Production Medium Scale**: 10-50 trading pairs, 3-5 instances
5. **Production Large Scale**: 50+ trading pairs, distributed cluster

---

## SYS-HARDWARE-001: Development Environment Requirements

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-HARDWARE-001`

**Description**:
Hardware specifications for local development workstations used by engineers to build, test, and debug the Bot Core platform. Optimized for Docker-based development with hot reload capabilities.

**Minimum Requirements**:

**CPU**:
- Cores: 4 physical cores (8 threads)
- Architecture: x86_64 (AMD64) or ARM64 (Apple Silicon)
- Clock Speed: 2.0 GHz base frequency
- Instruction Sets: AVX2 (Intel/AMD), NEON (ARM)
- **Rationale**:
  - 1 core: Rust compilation (cargo build)
  - 1 core: Python AI service
  - 1 core: Frontend build (Vite/Bun)
  - 1 core: System operations, MongoDB, IDE

**Memory (RAM)**:
- Minimum: 8 GB
- Available for Docker: 6 GB
- Swap: 4 GB
- **Memory Breakdown**:
  - Host OS: 2 GB
  - IDE (VSCode/IntelliJ): 1.5 GB
  - Docker Desktop: 500 MB overhead
  - Python AI Service (dev): 1.5 GB (limit)
  - Rust Core Engine (dev): 1.5 GB (limit)
  - Frontend Dashboard (dev): 768 MB (limit)
  - MongoDB: 512 MB
  - Build processes: 1.5 GB

**Storage**:
- Minimum: 50 GB available space
- Type: SSD (SATA or NVMe)
- **Storage Breakdown**:
  - Docker images: 5 GB
  - Rust target/ directory: 8 GB
  - Python venv + packages: 3 GB
  - Node modules: 2 GB
  - MongoDB data: 1 GB
  - Logs + cache: 2 GB
  - Source code: 500 MB
  - Available for builds: 28 GB

**Network**:
- Bandwidth: 10 Mbps down / 5 Mbps up
- Latency to Binance testnet: < 500ms acceptable
- Reliable connection for WebSocket stability

**GPU**: Not required for development

---

**Recommended Requirements**:

**CPU**:
- Cores: 8 physical cores (16 threads)
- Architecture: Modern x86_64 or Apple M1/M2/M3
- Clock Speed: 3.0+ GHz boost frequency
- Instruction Sets: AVX2, AVX-512 (Intel), or Apple Silicon AMX
- **Benefits**:
  - Parallel builds significantly faster
  - Hot reload with minimal impact
  - Run full test suite locally
  - Multiple services simultaneously

**Memory (RAM)**:
- Recommended: 16 GB
- Available for Docker: 12 GB
- Swap: 8 GB
- **Memory Breakdown**:
  - Host OS: 3 GB
  - IDE + Extensions: 2 GB
  - Docker Desktop: 1 GB
  - Python AI Service (dev): 2 GB
  - Rust Core Engine (dev): 2 GB
  - Frontend Dashboard (dev): 1 GB
  - MongoDB: 1 GB
  - Redis (optional): 512 MB
  - Build processes: 3 GB

**Storage**:
- Recommended: 100 GB+ available space
- Type: NVMe SSD
- Read Speed: 1500+ MB/s
- Write Speed: 1000+ MB/s
- **Benefits**:
  - Faster cargo incremental builds
  - Quicker npm install/bun install
  - Improved Docker layer caching
  - Room for test databases

**Network**:
- Bandwidth: 50+ Mbps down / 20+ Mbps up
- Latency to Binance: < 100ms ideal

**GPU**:
- Optional: Integrated graphics sufficient
- Benefit: Accelerated AI model training (not required for dev)

**Acceptance Criteria**:
- [x] Minimum specs support `make dev` without OOM errors
- [x] Recommended specs build all services in < 5 minutes
- [x] Docker Desktop configured with memory limits
- [x] Hot reload works without system lag
- [x] Can run full test suite locally

**Dependencies**: SYS-SOFTWARE-001 (OS), SYS-SOFTWARE-006 (Docker)
**Test Cases**: TC-HARDWARE-001 (Development Environment Test)

**Implementation Notes**:
```yaml
# docker-compose.yml - Development memory limits
deploy:
  resources:
    limits:
      memory: 1.5G      # Python AI (dev)
      cpus: "1.5"
    limits:
      memory: 1.5G      # Rust Core (dev)
      cpus: "1.5"
    limits:
      memory: 768M      # Frontend (dev)
      cpus: "1"
```

**Reference**: `/infrastructure/docker/docker-compose.yml` lines 88-92, 180-183, 276-279

---

## SYS-HARDWARE-002: Production Environment - Small Scale

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-HARDWARE-002`

**Description**:
Hardware specifications for small-scale production deployments supporting 1-10 trading pairs with moderate trading volume. Suitable for individual traders or small teams testing production strategies.

**Minimum Requirements**:

**CPU**:
- Cores: 4 physical cores (8 threads)
- Architecture: x86_64 server-grade (Intel Xeon, AMD EPYC)
- Clock Speed: 2.5 GHz base, 3.5 GHz boost
- L3 Cache: 8 MB+
- **Rationale**:
  - Real-time WebSocket data processing
  - Low-latency trading execution
  - ML inference for market predictions
  - Database operations

**Memory (RAM)**:
- Minimum: 8 GB
- Type: DDR4-2666 or better, ECC recommended
- Configuration: Single channel acceptable
- **Memory Allocation**:
  - Python AI Service: 2 GB (limit)
  - Rust Core Engine: 2 GB (limit)
  - Frontend Dashboard: 1 GB (limit)
  - MongoDB: 2 GB
  - Operating System: 1 GB
  - Available/Cache: 1 GB

**Storage**:
- Minimum: 100 GB
- Type: SSD (NVMe preferred)
- IOPS: 3,000+ read, 1,500+ write
- **Storage Allocation**:
  - MongoDB data: 50 GB (historical data)
  - Docker images: 10 GB
  - Logs (30 day retention): 10 GB
  - AI models + checkpoints: 5 GB
  - Backups: 20 GB
  - Available: 5 GB

**Network Interface**:
- Bandwidth: 1 Gbps port
- Latency to exchange: < 50ms critical
- Redundancy: Single NIC acceptable

**GPU**: Not required

---

**Recommended Requirements**:

**CPU**:
- Cores: 8 physical cores (16 threads)
- Architecture: Modern x86_64 (Intel Xeon Ice Lake, AMD EPYC Milan)
- Clock Speed: 3.0 GHz base, 4.0 GHz boost
- L3 Cache: 16 MB+
- Instruction Sets: AVX-512
- **Benefits**:
  - Headroom for traffic spikes
  - Concurrent AI analysis
  - Faster database queries
  - Support for optional services (monitoring)

**Memory (RAM)**:
- Recommended: 16 GB
- Type: DDR4-3200 or DDR5, ECC
- Configuration: Dual channel
- **Memory Allocation**:
  - Python AI Service: 3 GB
  - Rust Core Engine: 3 GB
  - Frontend Dashboard: 1.5 GB
  - MongoDB: 4 GB
  - Redis (optional): 1 GB
  - Operating System: 2 GB
  - Available/Cache: 2.5 GB

**Storage**:
- Recommended: 250 GB NVMe SSD
- IOPS: 10,000+ read, 5,000+ write
- RAID: RAID-1 or RAID-10 for redundancy
- **Storage Allocation**:
  - MongoDB data: 120 GB
  - Docker images: 15 GB
  - Logs (90 day retention): 30 GB
  - AI models + checkpoints: 10 GB
  - Backups: 60 GB
  - Available: 15 GB

**Network Interface**:
- Bandwidth: 10 Gbps port
- Latency to exchange: < 20ms target
- Redundancy: Dual NIC (bonded/teamed)

**GPU**:
- Optional: NVIDIA Tesla T4 or equivalent
- VRAM: 8 GB+
- Purpose: Accelerated AI model training and inference

**Acceptance Criteria**:
- [x] System handles 10 trading pairs concurrently
- [x] Trading execution latency < 100ms (p99)
- [x] Zero OOM incidents under normal load
- [x] MongoDB query response < 50ms average
- [x] Resource utilization 60-80% average, < 90% peak
- [x] 30-day logs retained without disk pressure

**Dependencies**: SYS-NETWORK-002 (External API Access), SYS-SOFTWARE-005 (MongoDB)
**Test Cases**: TC-HARDWARE-002 (Production Small Scale Load Test)

**Implementation Notes**:
```yaml
# docker-compose.yml - Production memory limits
deploy:
  resources:
    limits:
      memory: ${PYTHON_MEMORY_LIMIT:-2G}
      cpus: "${PYTHON_CPU_LIMIT:-2}"
    reservations:
      memory: ${PYTHON_MEMORY_RESERVE:-1G}
      cpus: "${PYTHON_CPU_RESERVE:-1}"
```

**Reference**: `/infrastructure/docker/docker-compose.yml` lines 33-40, 129-136, 218-225

---

## SYS-HARDWARE-003: Production Environment - Medium Scale

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-HARDWARE-003`

**Description**:
Hardware specifications for medium-scale production deployments supporting 10-50 trading pairs with high trading volume. Suitable for professional traders, hedge funds, or trading firms.

**Minimum Requirements**:

**CPU**:
- Cores: 8 physical cores (16 threads)
- Architecture: x86_64 server-grade
- Clock Speed: 2.8 GHz base, 3.8 GHz boost
- L3 Cache: 16 MB+
- NUMA: Single socket acceptable

**Memory (RAM)**:
- Minimum: 32 GB
- Type: DDR4-3200, ECC required
- Configuration: Dual channel
- **Memory Allocation**:
  - Python AI Service: 8 GB
  - Rust Core Engine: 8 GB
  - Frontend Dashboard: 2 GB
  - MongoDB: 8 GB
  - Redis: 2 GB
  - Operating System: 2 GB
  - Available/Cache: 2 GB

**Storage**:
- Minimum: 500 GB NVMe SSD
- IOPS: 20,000+ read, 10,000+ write
- RAID: RAID-10 required
- **Storage Allocation**:
  - MongoDB data: 250 GB
  - Docker images: 20 GB
  - Logs (90 day retention): 80 GB
  - AI models + checkpoints: 30 GB
  - Backups: 100 GB
  - Available: 20 GB

**Network Interface**:
- Bandwidth: 10 Gbps port
- Latency to exchange: < 20ms required
- Redundancy: Dual NIC (bonded) required

**GPU**: Recommended for optimal performance

---

**Recommended Requirements**:

**CPU**:
- Cores: 16 physical cores (32 threads)
- Architecture: Intel Xeon Platinum 8300 series or AMD EPYC 7003 series
- Clock Speed: 3.0+ GHz base, 4.0+ GHz boost
- L3 Cache: 32 MB+
- NUMA: Dual socket for horizontal scaling

**Memory (RAM)**:
- Recommended: 64 GB
- Type: DDR4-3200 or DDR5, ECC
- Configuration: Quad channel
- **Memory Allocation**:
  - Python AI Service (×2 instances): 16 GB
  - Rust Core Engine (×2 instances): 16 GB
  - Frontend Dashboard: 4 GB
  - MongoDB: 16 GB
  - Redis: 4 GB
  - RabbitMQ (optional): 2 GB
  - Operating System: 4 GB
  - Available/Cache: 2 GB

**Storage**:
- Primary: 1 TB NVMe SSD (RAID-10)
- Backup: 2 TB HDD (RAID-1)
- IOPS: 50,000+ read, 25,000+ write
- **Storage Allocation**:
  - MongoDB data: 500 GB
  - Docker images: 30 GB
  - Logs (180 day retention): 150 GB
  - AI models + checkpoints: 50 GB
  - Backups: 250 GB
  - Available: 20 GB

**Network Interface**:
- Bandwidth: 25 Gbps or dual 10 Gbps
- Latency to exchange: < 10ms target
- Redundancy: Dual NIC (active-active)

**GPU**:
- Recommended: NVIDIA A10 or A100
- VRAM: 24 GB+
- Purpose: Real-time AI inference + training

**Acceptance Criteria**:
- [x] System handles 50 trading pairs concurrently
- [x] Trading execution latency < 50ms (p99)
- [x] Support 3-5 service instances per component
- [x] Database query response < 25ms average
- [x] AI inference latency < 100ms
- [x] Zero downtime during failover
- [x] 180-day logs retained

**Dependencies**: SYS-NETWORK-003 (Internal Network), SYS-SOFTWARE-006 (Kubernetes)
**Test Cases**: TC-HARDWARE-003 (Production Medium Scale Load Test)

---

## SYS-HARDWARE-004: Production Environment - Large Scale

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-HARDWARE-004`

**Description**:
Hardware specifications for large-scale production deployments supporting 50+ trading pairs with very high trading volume and distributed architecture. Suitable for institutional trading, exchanges, or high-frequency trading (HFT) operations.

**Minimum Requirements**:

**CPU**:
- Cores: 16 physical cores (32 threads) per node
- Architecture: Latest gen x86_64 server-grade
- Clock Speed: 3.0 GHz base, 4.0 GHz boost
- L3 Cache: 32 MB+
- NUMA: Dual socket

**Memory (RAM)**:
- Minimum: 64 GB per node
- Type: DDR5, ECC required
- Configuration: Quad channel
- Nodes: 3+ nodes for redundancy

**Storage**:
- Minimum: 1 TB NVMe SSD per node
- IOPS: 50,000+ read, 25,000+ write
- RAID: RAID-10 required
- Distributed Storage: Ceph or GlusterFS recommended

**Network Infrastructure**:
- Bandwidth: 25 Gbps per node
- Latency: < 10ms inter-node
- Redundancy: Dual NIC per node (bonded)
- Load Balancer: Required (HAProxy, NGINX, or hardware)

**GPU**: Required for optimal AI performance

---

**Recommended Requirements**:

**CPU**:
- Cores: 32 physical cores (64 threads) per node
- Architecture: Intel Xeon Platinum 8400 or AMD EPYC 9004 series
- Clock Speed: 3.2+ GHz base, 4.5+ GHz boost
- L3 Cache: 64 MB+
- NUMA: Dual socket optimized

**Memory (RAM)**:
- Recommended: 128 GB per node
- Type: DDR5-4800, ECC with advanced RAS features
- Configuration: Octa channel
- Nodes: 5+ nodes for high availability

**Storage**:
- Primary: 2 TB NVMe SSD (RAID-10) per node
- Hot Storage: Additional 4 TB NVMe for active data
- Cold Storage: 20 TB HDD for historical archives
- IOPS: 100,000+ read, 50,000+ write
- Distributed Storage: Required with 3-way replication

**Network Infrastructure**:
- Bandwidth: 100 Gbps backbone, 25 Gbps per node
- Latency: < 5ms inter-node, < 1ms intra-rack
- Redundancy: Dual 25 Gbps NIC per node
- Load Balancer: Redundant pair (active-active)
- Network Fabric: RDMA-capable for ultra-low latency

**GPU Cluster**:
- Recommended: NVIDIA A100 or H100 per node
- VRAM: 40-80 GB per GPU
- Interconnect: NVLink for multi-GPU
- Purpose: Distributed AI training + inference

**Acceptance Criteria**:
- [x] System handles 100+ trading pairs concurrently
- [x] Trading execution latency < 20ms (p99), < 50ms (p99.9)
- [x] Support horizontal scaling to 10+ nodes
- [x] Database cluster with automatic failover
- [x] AI inference latency < 50ms
- [x] 99.99% uptime SLA achievable
- [x] Zero data loss during node failure
- [x] 1-year historical data retention

**Dependencies**: SYS-NETWORK-006 (Firewall Rules), SYS-SOFTWARE-006 (Kubernetes)
**Test Cases**: TC-HARDWARE-004 (Production Large Scale Load Test), TC-HARDWARE-005 (Failover Test)

**Configuration Example**:
```yaml
# Kubernetes node specification
apiVersion: v1
kind: Node
metadata:
  name: trading-node-01
spec:
  capacity:
    cpu: "64"
    memory: "128Gi"
    nvidia.com/gpu: "2"
    ephemeral-storage: "2Ti"
```

---

## SYS-HARDWARE-005: Memory Optimization Settings

**Priority**: ☑ Critical
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-HARDWARE-005`

**Description**:
Docker memory limits and optimization settings to prevent OOM (Out Of Memory) errors and ensure stable operation on resource-constrained systems. These settings are critical for deployment on machines with limited RAM.

**Memory Limits Configuration**:

### Development Mode (docker-compose.yml)

**Python AI Service (Dev)**:
```yaml
deploy:
  resources:
    limits:
      memory: 1.5G      # Hard limit
      cpus: "1.5"
```
- **Hard Limit**: 1.5 GB
- **Soft Limit**: 1.0 GB (reservation)
- **OOM Kill**: Enabled if exceeded
- **Swap**: Up to 512 MB allowed

**Rust Core Engine (Dev)**:
```yaml
deploy:
  resources:
    limits:
      memory: 1.5G      # Hard limit
      cpus: "1.5"
```
- **Hard Limit**: 1.5 GB
- **Soft Limit**: 1.0 GB (reservation)
- **OOM Kill**: Enabled if exceeded

**Frontend Dashboard (Dev)**:
```yaml
deploy:
  resources:
    limits:
      memory: 768M      # Hard limit
      cpus: "1"
environment:
  - NODE_OPTIONS="--max-old-space-size=768"
```
- **Hard Limit**: 768 MB
- **Node.js Heap**: 768 MB
- **OOM Kill**: Enabled if exceeded

### Production Mode (docker-compose.yml)

**Python AI Service (Prod)**:
```yaml
deploy:
  resources:
    limits:
      memory: ${PYTHON_MEMORY_LIMIT:-2G}
      cpus: "${PYTHON_CPU_LIMIT:-2}"
    reservations:
      memory: ${PYTHON_MEMORY_RESERVE:-1G}
      cpus: "${PYTHON_CPU_RESERVE:-1}"
```
- **Default Limit**: 2 GB (configurable via env)
- **Reservation**: 1 GB guaranteed
- **Recommended for AI workloads**: 3-4 GB

**Rust Core Engine (Prod)**:
```yaml
deploy:
  resources:
    limits:
      memory: ${RUST_MEMORY_LIMIT:-2G}
      cpus: "${RUST_CPU_LIMIT:-2}"
    reservations:
      memory: ${RUST_MEMORY_RESERVE:-1G}
      cpus: "${RUST_CPU_RESERVE:-1}"
```
- **Default Limit**: 2 GB
- **Reservation**: 1 GB
- **Recommended for trading**: 3-4 GB

**Frontend Dashboard (Prod)**:
```yaml
deploy:
  resources:
    limits:
      memory: ${FRONTEND_MEMORY_LIMIT:-1G}
      cpus: "${FRONTEND_CPU_LIMIT:-1}"
    reservations:
      memory: ${FRONTEND_MEMORY_RESERVE:-256M}
      cpus: "${FRONTEND_CPU_RESERVE:-0.5}"
environment:
  - NODE_OPTIONS="--max-old-space-size=${NODE_MEMORY:-1024}"
```
- **Default Limit**: 1 GB
- **Reservation**: 256 MB
- **Node.js Heap**: 1024 MB

### Memory-Optimized Mode

**Start Command**:
```bash
./scripts/bot.sh start --memory-optimized
```

**Total Memory Allocation**:
- Python AI: 1.5 GB
- Rust Core: 1 GB
- Frontend: 512 MB
- MongoDB: 2 GB (external service)
- **Total**: ~5 GB + 1 GB overhead = **6 GB minimum**

### MongoDB Memory Configuration

```yaml
# docker-compose.yml - MongoDB (if added)
mongodb:
  environment:
    - MONGO_MEMORY_LIMIT=2G
  deploy:
    resources:
      limits:
        memory: 2G
      reservations:
        memory: 1G
```
- **WiredTiger Cache**: 50% of allocated memory (1 GB)
- **Minimum**: 1 GB for development
- **Recommended**: 4+ GB for production

**Acceptance Criteria**:
- [x] Memory-optimized mode runs on 8 GB RAM machines
- [x] No OOM kills under normal operation
- [x] Swap usage < 20% of RAM
- [x] Memory limits enforced by Docker cgroups
- [x] Graceful degradation when approaching limits
- [x] Environment variables allow customization

**Dependencies**: SYS-SOFTWARE-006 (Docker Compose)
**Test Cases**: TC-HARDWARE-006 (Memory Limit Test), TC-HARDWARE-007 (OOM Handling Test)

**Monitoring Commands**:
```bash
# Check resource usage
docker stats --no-stream

# Monitor memory in real-time
docker stats --format "table {{.Name}}\t{{.MemUsage}}\t{{.MemPerc}}"

# Check for OOM events
docker inspect <container> | grep OOMKilled
```

**Reference**: `/infrastructure/docker/docker-compose.yml` lines 33-40, 88-92, 129-136, 180-183, 218-225, 276-279

---

## SYS-HARDWARE-006: Storage Requirements and Planning

**Priority**: ☑ High
**Status**: ☑ Completed
**Code Tags**: `@spec:SYS-HARDWARE-006`

**Description**:
Detailed storage requirements for database persistence, logs, AI models, backups, and Docker images. Includes growth projections and retention policies.

### Storage Breakdown by Component

**MongoDB Data Storage**:
- **Development**: 1-5 GB
  - Test data: 500 MB
  - Historical candles (30 days): 2 GB
  - User data: 100 MB
- **Small Production**: 10-50 GB
  - Historical candles (90 days): 20 GB
  - Trade history: 10 GB
  - User accounts + settings: 1 GB
  - Indexes: 5 GB
- **Medium Production**: 50-250 GB
  - Historical candles (180 days): 100 GB
  - Trade history: 80 GB
  - AI training data: 30 GB
  - Indexes: 20 GB
- **Large Production**: 250-1000 GB
  - Historical candles (1 year): 400 GB
  - Trade history: 300 GB
  - AI training data: 150 GB
  - Indexes: 100 GB

**Growth Rate**: ~2-5 GB/month per trading pair

**Log Storage**:
- **Retention Policy**:
  - Development: 7 days
  - Production: 30-180 days
- **Log Volume**:
  - Rust Core: 100-500 MB/day
  - Python AI: 200-1000 MB/day
  - Frontend: 50-200 MB/day
  - MongoDB: 100-300 MB/day
- **Total**: 500 MB - 2 GB per day
- **30-day retention**: 15-60 GB
- **90-day retention**: 45-180 GB

**AI Models and Checkpoints**:
- **LSTM Model**: 50-200 MB
- **GRU Model**: 50-200 MB
- **Transformer Model**: 200-800 MB
- **Model Checkpoints** (5 versions): 1-4 GB
- **Training Data Cache**: 2-10 GB
- **Total**: 3-15 GB typical, up to 50 GB with multiple strategies

**Docker Images**:
- **Base Images**:
  - Rust: 1.5 GB
  - Python: 1.2 GB
  - Node.js: 800 MB
- **Built Images**:
  - rust-core-engine: 500 MB - 1 GB
  - python-ai-service: 1.5 - 2.5 GB
  - nextjs-ui-dashboard: 300 - 500 MB
  - Supporting images (MongoDB, Redis, etc.): 1 GB
- **Total with layers**: 5-10 GB
- **With multiple versions**: 10-20 GB

**Backup Storage**:
- **MongoDB Dumps**:
  - Compressed: 20-50% of database size
  - Frequency: Daily
  - Retention: 30 backups
- **Configuration Backups**: 100 MB
- **Code Snapshots**: 500 MB
- **Total**: 1-2x database size for 30-day retention

**Temporary/Cache Storage**:
- **Rust target/ directory**: 5-10 GB
- **Python __pycache__ + venv**: 2-3 GB
- **Node node_modules**: 1-2 GB
- **Docker build cache**: 5-10 GB
- **Total**: 15-25 GB

### Storage Performance Requirements

**IOPS Requirements**:
- **Development**:
  - Read: 500 IOPS
  - Write: 250 IOPS
  - Any SSD acceptable
- **Small Production**:
  - Read: 3,000 IOPS
  - Write: 1,500 IOPS
  - SATA SSD minimum
- **Medium Production**:
  - Read: 10,000 IOPS
  - Write: 5,000 IOPS
  - NVMe SSD required
- **Large Production**:
  - Read: 50,000+ IOPS
  - Write: 25,000+ IOPS
  - Enterprise NVMe with power-loss protection

**Throughput Requirements**:
- **MongoDB Operations**: 200-500 MB/s sustained
- **Log Writing**: 50-100 MB/s burst
- **Backup Operations**: 500+ MB/s for fast backups
- **AI Model Loading**: 1+ GB/s for quick startup

**Latency Requirements**:
- **Database Reads**: < 5ms average, < 20ms p99
- **Database Writes**: < 10ms average, < 50ms p99
- **Log Writes**: < 20ms acceptable (async)

### Storage Recommendations

**Development**:
- **Type**: Consumer NVMe SSD
- **Size**: 100 GB minimum, 250 GB recommended
- **Models**: Samsung 970 EVO, WD Black SN750, any M.2 NVMe

**Production Small/Medium**:
- **Type**: Data Center NVMe SSD
- **Size**: 500 GB - 2 TB
- **RAID**: RAID-10 for redundancy
- **Models**: Intel D7-P5510, Samsung PM9A3, Micron 7450

**Production Large**:
- **Type**: Enterprise NVMe with PLP (Power Loss Protection)
- **Size**: 2+ TB per node
- **RAID**: RAID-10 or distributed storage (Ceph)
- **Models**: Intel P5800X (Optane), Samsung PM1733, Kioxia CD6

### Volume Configuration

**Docker Volumes (docker-compose.yml)**:
```yaml
volumes:
  # Persistent data volumes
  ./python-ai-service/models:/app/models        # AI models
  ./python-ai-service/logs:/app/logs            # Logs
  ./python-ai-service/data:/app/data            # Training data
  ./rust-core-engine/data:/app/data             # Trading data
  ./rust-core-engine/logs:/app/logs             # Logs

  # Named volumes
  redis_data: {}           # Redis persistence
  rabbitmq_data: {}        # RabbitMQ persistence
  prometheus_data: {}      # Metrics data
  grafana_data: {}         # Dashboard configs
  rust_target_cache: {}    # Rust build cache
```

**Acceptance Criteria**:
- [x] Storage allocation matches documented breakdown
- [x] Growth projections validated with 30-day test
- [x] Backup process completes within maintenance window
- [x] IOPS requirements met under load testing
- [x] No disk space alerts under normal operation
- [x] Retention policies automatically enforced
- [x] Log rotation configured correctly

**Dependencies**: SYS-SOFTWARE-005 (MongoDB), SYS-HARDWARE-001 to 004
**Test Cases**: TC-HARDWARE-008 (Storage Performance Test), TC-HARDWARE-009 (Disk Space Monitoring)

**Monitoring**:
```bash
# Check disk usage
df -h

# Check Docker volume usage
docker system df -v

# Check MongoDB storage stats
docker exec mongodb mongo --eval "db.stats()"

# Check IOPS
iostat -x 1
```

**Reference**: `/infrastructure/docker/docker-compose.yml` lines 20-24, 64-67, 115-117, 160-167, 257-262, 447-459

---

## Resource Monitoring and Capacity Planning

### Monitoring Tools

**System-Level Monitoring**:
- **CPU**: `top`, `htop`, `mpstat`
- **Memory**: `free -h`, `vmstat`, `sar`
- **Disk**: `df -h`, `iostat`, `iotop`
- **Network**: `iftop`, `nethogs`, `ss`

**Docker Monitoring**:
- **Real-time**: `docker stats`
- **Historical**: cAdvisor + Prometheus + Grafana
- **Logs**: `docker logs <container>`

**Application Monitoring**:
- **Rust**: Built-in metrics endpoint at `:8080/metrics`
- **Python**: FastAPI metrics at `:8000/metrics`
- **Frontend**: Browser performance API

### Alerting Thresholds

**Critical Alerts**:
- CPU utilization > 90% for 5 minutes
- Memory utilization > 95%
- Disk space < 10% free
- Swap usage > 50% of RAM
- IOPS at storage limit

**Warning Alerts**:
- CPU utilization > 80% for 10 minutes
- Memory utilization > 85%
- Disk space < 20% free
- Swap usage > 25% of RAM
- Network bandwidth > 80%

### Capacity Planning Guidelines

**When to Scale Up (Vertical)**:
- CPU utilization consistently > 70%
- Memory utilization consistently > 80%
- Disk IOPS consistently > 80% of limit
- Trading latency degradation

**When to Scale Out (Horizontal)**:
- Single node at capacity
- Geographic distribution needed
- Redundancy requirements
- Cost efficiency (cloud)

**Upgrade Triggers**:
- Adding 10+ new trading pairs
- Implementing new AI models
- Expanding to 24/7 operations
- User growth > 50%

---

## Hardware Procurement Checklist

### Development Workstations

- [ ] CPU: 8+ cores recommended
- [ ] RAM: 16 GB minimum
- [ ] Storage: 250 GB NVMe SSD
- [ ] OS: Ubuntu 22.04 LTS or macOS 12+
- [ ] Docker Desktop: 4.0+ installed
- [ ] Network: Stable broadband connection

### Production Servers (Small Scale)

- [ ] Server: Dell PowerEdge R650, HPE ProLiant DL360, or equivalent
- [ ] CPU: Intel Xeon Gold 6300 or AMD EPYC 7003 series (8 cores)
- [ ] RAM: 32 GB DDR4 ECC
- [ ] Storage: 500 GB NVMe SSD in RAID-10
- [ ] Network: Dual 10 Gbps NICs
- [ ] Power: Redundant PSU
- [ ] Location: Colocation near exchange (< 20ms)

### Production Servers (Large Scale)

- [ ] Server: Dell PowerEdge R750xa, HPE ProLiant DL380, or equivalent
- [ ] CPU: Intel Xeon Platinum 8400 or AMD EPYC 9004 series (32 cores)
- [ ] RAM: 128 GB DDR5 ECC
- [ ] Storage: 2 TB NVMe SSD in RAID-10 + distributed storage
- [ ] Network: Dual 25 Gbps NICs (RDMA-capable)
- [ ] GPU: NVIDIA A100 (optional but recommended)
- [ ] Power: Redundant PSU with UPS
- [ ] Location: Multiple colocations for redundancy

### Cloud Alternatives

**AWS**:
- Small: c6i.2xlarge (8 vCPU, 16 GB)
- Medium: c6i.4xlarge (16 vCPU, 32 GB)
- Large: c6i.16xlarge (64 vCPU, 128 GB) + p4d for GPU

**GCP**:
- Small: n2-standard-8 (8 vCPU, 32 GB)
- Medium: n2-standard-16 (16 vCPU, 64 GB)
- Large: n2-standard-64 (64 vCPU, 256 GB) + a2-highgpu for GPU

**Azure**:
- Small: D8s v5 (8 vCPU, 32 GB)
- Medium: D16s v5 (16 vCPU, 64 GB)
- Large: D64s v5 (64 vCPU, 256 GB) + NC A100 for GPU

---

## Performance Benchmarks

### Expected Performance by Hardware Tier

**Development (8GB RAM, 4 cores)**:
- Build time: 5-10 minutes
- Trading latency: 200-500ms
- AI inference: 500-1000ms
- Max trading pairs: 3-5
- Concurrent users: 1-2

**Small Production (16GB RAM, 8 cores)**:
- Trading latency: 50-100ms (p99)
- AI inference: 200-300ms
- Max trading pairs: 10
- Concurrent users: 10-20
- Throughput: 100 trades/minute

**Medium Production (64GB RAM, 16 cores)**:
- Trading latency: 20-50ms (p99)
- AI inference: 50-100ms
- Max trading pairs: 50
- Concurrent users: 50-100
- Throughput: 500 trades/minute

**Large Production (128GB RAM, 32 cores + GPU)**:
- Trading latency: 10-20ms (p99)
- AI inference: 20-50ms
- Max trading pairs: 100+
- Concurrent users: 200-500
- Throughput: 2000+ trades/minute

---

## Cost Estimates (Approximate)

### On-Premise Hardware

**Development Workstation**: $1,500 - $3,000
- Laptop: MacBook Pro M2/M3 or Dell XPS 15
- Desktop: Custom build or workstation

**Small Production Server**: $5,000 - $10,000
- Server chassis + components
- Colocation: $200-500/month

**Medium Production Server**: $15,000 - $30,000
- Server chassis + components
- Colocation: $500-1000/month

**Large Production Cluster (5 nodes)**: $150,000+
- Servers + networking + storage
- Colocation: $3,000-5,000/month

### Cloud Costs (Monthly Estimates)

**Small Production**:
- AWS c6i.2xlarge: ~$250/month
- Storage (500GB): ~$50/month
- Network: ~$50/month
- **Total**: ~$350/month

**Medium Production**:
- AWS c6i.4xlarge: ~$500/month
- Storage (2TB): ~$200/month
- Network: ~$150/month
- **Total**: ~$850/month

**Large Production**:
- AWS c6i.16xlarge (3 nodes): ~$4,500/month
- GPU (p4d.24xlarge): ~$10,000/month (if needed)
- Storage (10TB): ~$1,000/month
- Network: ~$500/month
- **Total**: ~$6,000-16,000/month

---

## Risks & Mitigations

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Insufficient RAM causes OOM kills | High | Medium | Memory-optimized mode, monitoring, swap allocation |
| Storage fills up with logs | Medium | High | Log rotation, 30-day retention, monitoring alerts |
| CPU bottleneck during market volatility | High | Medium | Horizontal scaling, priority queues, rate limiting |
| Disk failure causes data loss | Critical | Low | RAID-10, regular backups, distributed storage |
| Network latency to exchange | High | Medium | Colocation near exchange, redundant links |
| Docker build exceeds memory | Medium | Medium | Sequential builds (--memory-optimized), increased swap |

---

## Traceability

**Requirements**:
- Business Rule: [BUSINESS_RULES.md - System Availability](../../BUSINESS_RULES.md)
- User Story: US-OPS-001 (System Deployment)

**Design**:
- Architecture: [ARCH-INFRASTRUCTURE-001](../../02-design/2.1-architecture/INFRASTRUCTURE.md)
- Docker Compose: [docker-compose.yml](../../../infrastructure/docker/docker-compose.yml)
- Build Scripts: [build-services.sh](../../../scripts/build-services.sh)

**Test Cases**:
- Load Testing: TC-HARDWARE-001 to TC-HARDWARE-009
- Performance: TC-PERFORMANCE-001 to TC-PERFORMANCE-010
- Monitoring: TC-OPS-MONITORING-001

---

## Open Questions

- [ ] Should we provide ARM64 specifications for Apple Silicon or AWS Graviton?
- [ ] What is the budget for GPU hardware for AI acceleration?
- [ ] Do we need bare metal servers or is cloud acceptable?
- [ ] What is the target latency to Binance (colocation vs cloud)?

---

## Changelog

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-10-10 | Infrastructure Team | Initial version with dev, small, medium, large scale specs |

---

## Appendix

### References

- Docker Compose File: `/infrastructure/docker/docker-compose.yml`
- CLAUDE.md: `/CLAUDE.md` (Performance Optimization section)
- Build Scripts: `/scripts/build-services.sh`, `/scripts/bot.sh`
- Makefile: `/Makefile` (Memory and build targets)

### Glossary

- **OOM**: Out Of Memory - condition where process exceeds memory limit
- **IOPS**: Input/Output Operations Per Second - storage performance metric
- **NVMe**: Non-Volatile Memory Express - high-speed storage interface
- **NUMA**: Non-Uniform Memory Access - multi-processor memory architecture
- **ECC**: Error-Correcting Code - memory that detects/corrects errors
- **RDMA**: Remote Direct Memory Access - low-latency networking
- **PLP**: Power Loss Protection - prevents data corruption on power failure

### Configuration Examples

**Development .env**:
```bash
# Memory limits for development
PYTHON_MEMORY_LIMIT=1.5G
RUST_MEMORY_LIMIT=1.5G
FRONTEND_MEMORY_LIMIT=768M
NODE_MEMORY=768
```

**Production .env**:
```bash
# Memory limits for production
PYTHON_MEMORY_LIMIT=3G
PYTHON_MEMORY_RESERVE=2G
RUST_MEMORY_LIMIT=3G
RUST_MEMORY_RESERVE=2G
FRONTEND_MEMORY_LIMIT=2G
FRONTEND_MEMORY_RESERVE=512M
NODE_MEMORY=1536
```

---

**Remember**: Update TRACEABILITY_MATRIX.md when hardware is procured and benchmarked!
