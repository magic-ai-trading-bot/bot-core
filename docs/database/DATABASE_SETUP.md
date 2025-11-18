# Database Setup Guide

**Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Production-Ready

---

## Table of Contents

1. [Overview](#overview)
2. [MongoDB Architecture](#mongodb-architecture)
3. [Quick Start](#quick-start)
4. [Database Schema](#database-schema)
5. [Migrations](#migrations)
6. [Connection Strings](#connection-strings)
7. [Backup and Restore](#backup-and-restore)
8. [Performance Tuning](#performance-tuning)
9. [Troubleshooting](#troubleshooting)
10. [Security Best Practices](#security-best-practices)

---

## Overview

Bot Core uses **MongoDB 7.0+** as its primary database with a replica set configuration for high availability and data consistency. The database supports:

- High-frequency trading operations (< 10ms latency)
- AI/ML model storage and predictions
- Paper trading simulations
- Real-time portfolio tracking
- Comprehensive audit logging

### Key Features

- **21 Collections** - Organized by domain (trading, AI, user management)
- **50+ Indexes** - Optimized for query performance
- **Time-Series Data** - Efficient market data storage with TTL
- **Schema Validation** - Runtime data validation with JSON Schema
- **Replica Set** - High availability with automatic failover
- **Automated Migrations** - Version-controlled schema changes

---

## MongoDB Architecture

### Database Structure

```
bot_core (database)
│
├── Trading Domain (7 collections)
│   ├── trades                    # Live trading records
│   ├── positions                 # Active trading positions
│   ├── portfolio_snapshots       # Historical portfolio states
│   ├── risk_metrics              # Risk calculations
│   ├── strategy_configs          # User strategy settings
│   ├── performance_metrics       # Aggregated performance data
│   └── ai_signals                # AI-generated signals
│
├── Paper Trading Domain (4 collections)
│   ├── paper_trading_accounts    # Virtual account balances
│   ├── paper_trading_trades      # Simulated trades
│   ├── paper_trading_settings    # Paper trading config
│   └── portfolio_history         # Historical portfolio states
│
├── AI/ML Domain (6 collections)
│   ├── ml_models                 # Model metadata and performance
│   ├── predictions               # AI predictions and verification
│   ├── training_jobs             # Model training job tracking
│   ├── model_performance_history # Historical model performance
│   ├── market_indicators         # Technical indicators (time-series)
│   └── ai_analysis_results       # AI-generated analysis
│
├── User Management (4 collections)
│   ├── users                     # User accounts and profiles
│   ├── sessions                  # Active user sessions (TTL)
│   ├── api_keys                  # Exchange API keys (encrypted)
│   └── notifications             # User notifications (TTL)
│
└── System (3 collections)
    ├── market_data               # OHLCV candlestick data (time-series)
    ├── audit_logs                # System-wide audit trail (capped)
    └── system_config             # Global configuration settings
```

### Replica Set Configuration

The database runs as a single-node replica set (`rs0`) to enable:
- Change streams for real-time updates
- Oplog for point-in-time recovery
- Transaction support for multi-document operations
- Easy horizontal scaling (add replicas later)

---

## Quick Start

### 1. Environment Setup

Copy the environment template:

```bash
cp .env.example .env
```

Edit `.env` and set MongoDB credentials:

```bash
# Development (Docker)
DATABASE_URL=mongodb://admin:password@mongodb:27017/bot_core?authSource=admin&replicaSet=rs0

# MongoDB Credentials
MONGO_ROOT_USER=admin
MONGO_ROOT_PASSWORD=your-secure-password-here

# Resource Limits
MONGO_MEMORY_LIMIT=2G
MONGO_CPU_LIMIT=2
```

### 2. Start MongoDB Container

```bash
# Start all services (includes MongoDB)
./scripts/bot.sh start

# Or start only MongoDB
docker-compose -f infrastructure/docker/docker-compose.yml up -d mongodb
```

### 3. Initialize Database

Run the initialization script to create collections, indexes, and seed data:

```bash
./scripts/init-db.sh
```

The script will:
1. Wait for MongoDB to be ready
2. Initialize replica set
3. Run Rust service migrations
4. Run Python service migrations
5. Verify database setup
6. Display connection information

### 4. Verify Setup

Check database status:

```bash
# Using mongosh
mongosh "mongodb://admin:password@localhost:27017/bot_core?authSource=admin"

# List collections
db.getCollectionNames()

# Check replica set status
rs.status()

# View system configuration
db.system_config.find().pretty()
```

Or use MongoDB Express (web interface):

```bash
# Start with mongo-admin profile
docker-compose --profile mongo-admin up -d mongo-express

# Access at http://localhost:8081
# Username: admin
# Password: admin
```

---

## Database Schema

### Critical Collections

#### 1. Users Collection

**Purpose:** User authentication and profile management

**Key Fields:**
- `email` (unique, indexed) - User email address
- `password_hash` - bcrypt hashed password (60+ chars)
- `is_active` - Account status
- `is_admin` - Admin privileges
- `settings` - Trading preferences (embedded document)

**Indexes:**
- `{ email: 1 }` (unique)
- `{ created_at: -1 }`
- `{ is_active: 1 }`

#### 2. Trades Collection

**Purpose:** Live trading execution records

**Key Fields:**
- `user_id` (ObjectId) - Reference to users
- `symbol` - Trading pair (e.g., BTCUSDT)
- `side` - BUY | SELL
- `order_type` - MARKET | LIMIT | STOP_LOSS | TAKE_PROFIT
- `status` - PENDING | FILLED | PARTIALLY_FILLED | CANCELLED | FAILED
- `pnl` - Realized profit/loss
- `binance_order_id` - Binance order reference

**Indexes:**
- `{ user_id: 1, created_at: -1 }`
- `{ symbol: 1, created_at: -1 }`
- `{ status: 1 }`
- `{ binance_order_id: 1 }` (sparse)
- `{ created_at: 1 }` (TTL: 1 year)

#### 3. Positions Collection

**Purpose:** Active trading positions

**Key Fields:**
- `user_id` (ObjectId)
- `symbol` - Trading pair
- `side` - LONG | SHORT
- `quantity` - Position size
- `entry_price` - Average entry price
- `unrealized_pnl` - Current P&L
- `is_open` - Position status
- `stop_loss`, `take_profit` - Risk management levels

**Indexes:**
- `{ user_id: 1, symbol: 1 }` (unique)
- `{ is_open: 1, user_id: 1 }`
- `{ closed_at: 1 }` (TTL: 90 days for closed positions)

#### 4. ML Models Collection

**Purpose:** AI/ML model metadata and performance tracking

**Key Fields:**
- `model_name` - LSTM | GRU | TRANSFORMER | etc.
- `model_type` - CLASSIFICATION | REGRESSION | TIME_SERIES
- `version` - Semantic version (e.g., 1.0.0)
- `architecture` - Model structure details
- `performance_metrics` - Accuracy, precision, recall, F1, etc.
- `is_active`, `is_production` - Deployment status

**Indexes:**
- `{ model_name: 1, version: 1 }` (unique)
- `{ is_active: 1, is_production: 1 }`
- `{ 'performance_metrics.accuracy': -1 }` (sparse)

#### 5. Market Data Collection (Time-Series)

**Purpose:** OHLCV candlestick data storage

**Configuration:**
- Time-series collection optimized for timestamps
- `timeField`: timestamp
- `metaField`: symbol
- `granularity`: minutes
- TTL: 1 year (automatic cleanup)

**Indexes:**
- `{ symbol: 1, timestamp: -1 }`
- `{ interval: 1, timestamp: -1 }`

### Complete Schema Reference

For detailed schema specifications, see:
- **Comprehensive Schema:** `/Users/dungngo97/Documents/bot-core/specs/02-design/2.2-database/DB-SCHEMA.md`

---

## Migrations

### Migration System

Migrations are JavaScript files executed via `mongosh` to apply schema changes incrementally.

**Location:**
- Rust service: `/Users/dungngo97/Documents/bot-core/rust-core-engine/migrations/`
- Python service: `/Users/dungngo97/Documents/bot-core/python-ai-service/migrations/`

### Rust Service Migrations

#### 001_initial_schema.js
- Creates/updates core trading collections
- Applies schema validation rules
- Configures users, trades, positions, portfolios, etc.

#### 002_indexes.js
- Creates performance-optimized indexes
- Compound indexes for complex queries
- TTL indexes for automatic data cleanup
- Sparse indexes for optional fields

#### 003_seed_data.js
- Inserts system configuration
- Default strategy templates
- Risk thresholds and limits
- Feature flags
- Exchange configuration

### Python Service Migrations

#### 001_ml_models.js
- ML models collection schema
- Predictions collection with verification tracking
- Training jobs collection
- Model performance history

#### 002_market_data.js
- Market indicators (time-series)
- Technical indicator configurations
- AI service settings
- Indicator presets and templates

### Running Migrations

**Automated (Recommended):**
```bash
./scripts/init-db.sh
```

**Manual Execution:**
```bash
# Connect to database
MONGO_URL="mongodb://admin:password@localhost:27017/bot_core?authSource=admin"

# Run Rust migrations
mongosh "$MONGO_URL" < rust-core-engine/migrations/001_initial_schema.js
mongosh "$MONGO_URL" < rust-core-engine/migrations/002_indexes.js
mongosh "$MONGO_URL" < rust-core-engine/migrations/003_seed_data.js

# Run Python migrations
mongosh "$MONGO_URL" < python-ai-service/migrations/001_ml_models.js
mongosh "$MONGO_URL" < python-ai-service/migrations/002_market_data.js
```

### Creating New Migrations

1. Create numbered file (e.g., `004_new_feature.js`)
2. Use MongoDB commands to modify schema
3. Test in development environment
4. Document changes in migration file header
5. Add to migration sequence in `init-db.sh`

**Example Migration Template:**
```javascript
// @spec:FR-DB-XXX - Migration Description
// Migration XXX: Feature name
// Created: YYYY-MM-DD
// Purpose: What this migration does

print('Migration XXX: Feature Name');
db = db.getSiblingDB('bot_core');

// Your migration code here
db.createCollection('new_collection');
db.new_collection.createIndex({ field: 1 });

print('Migration XXX Completed');
```

---

## Connection Strings

### Development (Docker)

```bash
# Application connection (read/write)
mongodb://bot_core_app:password@localhost:27017/bot_core

# Admin connection (full access)
mongodb://bot_core_admin:password@localhost:27017/bot_core

# Read-only connection (analytics)
mongodb://bot_core_readonly:password@localhost:27017/bot_core

# With replica set (recommended)
mongodb://bot_core_app:password@localhost:27017/bot_core?authSource=admin&replicaSet=rs0
```

### Production (MongoDB Atlas)

```bash
# Standard connection
mongodb+srv://username:password@cluster.mongodb.net/bot_core?retryWrites=true&w=majority

# With additional options
mongodb+srv://username:password@cluster.mongodb.net/bot_core?retryWrites=true&w=majority&maxPoolSize=50&minPoolSize=10
```

### Connection Parameters

| Parameter | Description | Recommended Value |
|-----------|-------------|-------------------|
| `authSource` | Authentication database | `admin` |
| `replicaSet` | Replica set name | `rs0` |
| `retryWrites` | Retry failed writes | `true` |
| `w` | Write concern | `majority` |
| `maxPoolSize` | Max connections | `50` (production) |
| `minPoolSize` | Min connections | `10` (production) |
| `serverSelectionTimeoutMS` | Connection timeout | `5000` (5 seconds) |

---

## Backup and Restore

### Creating Backups

**Automated Backup (init-db.sh):**
```bash
CREATE_BACKUP=yes ./scripts/init-db.sh
```

**Manual Backup:**
```bash
# Full database backup
mongodump --uri="mongodb://admin:password@localhost:27017/bot_core?authSource=admin" \
  --out=/path/to/backup/bot_core_$(date +%Y%m%d_%H%M%S)

# Compress backup
tar -czf bot_core_backup.tar.gz /path/to/backup/

# Specific collection backup
mongodump --uri="mongodb://admin:password@localhost:27017/bot_core?authSource=admin" \
  --collection=trades \
  --out=/path/to/backup/
```

**Backup to S3/Cloud Storage:**
```bash
# Backup and upload to S3
mongodump --uri="$DATABASE_URL" --archive | aws s3 cp - s3://bucket/backups/bot_core_$(date +%Y%m%d).archive
```

### Restoring from Backup

```bash
# Restore full database
mongorestore --uri="mongodb://admin:password@localhost:27017/bot_core?authSource=admin" \
  /path/to/backup/

# Restore specific collection
mongorestore --uri="mongodb://admin:password@localhost:27017/bot_core?authSource=admin" \
  --collection=trades \
  /path/to/backup/bot_core/trades.bson

# Restore from archive
mongorestore --uri="$DATABASE_URL" --archive=/path/to/backup.archive
```

### Backup Schedule Recommendations

- **Development:** Daily backups, 7-day retention
- **Production:**
  - Hourly snapshots (24-hour retention)
  - Daily backups (30-day retention)
  - Weekly archives (1-year retention)
  - Monthly archives (permanent)

---

## Performance Tuning

### Index Optimization

**View Index Usage:**
```javascript
// Check index usage statistics
db.trades.aggregate([
  { $indexStats: {} }
])

// Find slow queries
db.setProfilingLevel(1, { slowms: 100 })
db.system.profile.find().limit(10).sort({ ts: -1 }).pretty()
```

**Analyze Query Performance:**
```javascript
// Use explain() to analyze queries
db.trades.find({ user_id: ObjectId("..."), status: "FILLED" }).explain("executionStats")

// Compound index for common query pattern
db.trades.createIndex({ user_id: 1, status: 1, created_at: -1 })
```

### Memory Configuration

**Docker Resource Limits (docker-compose.yml):**
```yaml
MONGO_MEMORY_LIMIT=2G      # Maximum memory
MONGO_CPU_LIMIT=2          # Maximum CPU cores
MONGO_MEMORY_RESERVE=512M  # Reserved memory
MONGO_CPU_RESERVE=0.5      # Reserved CPU
```

**MongoDB WiredTiger Cache:**
```javascript
// Default: 50% of RAM minus 1GB
// For 4GB system: ~1.5GB cache
// For 8GB system: ~3.5GB cache

// Check current cache size
db.serverStatus().wiredTiger.cache["maximum bytes configured"]
```

### Connection Pooling

**Application Configuration:**
```javascript
// Rust (MongoDB driver)
maxPoolSize: 50
minPoolSize: 10

// Python (Motor)
maxPoolSize=50
minPoolSize=10
```

### Time-Series Optimization

**Market Data Collection:**
- Granularity: `minutes` (optimal for trading data)
- Automatic bucketing by symbol
- TTL for automatic old data removal
- Efficient compression (40-60% storage savings)

**Query Optimization:**
```javascript
// Efficient time-range query
db.market_data.find({
  symbol: "BTCUSDT",
  timestamp: {
    $gte: ISODate("2025-11-01"),
    $lt: ISODate("2025-11-18")
  }
}).sort({ timestamp: -1 })
```

---

## Troubleshooting

### Common Issues

#### 1. Connection Refused

**Symptoms:** Services can't connect to MongoDB

**Solutions:**
```bash
# Check if MongoDB is running
docker ps | grep mongodb

# Check MongoDB logs
docker logs mongodb

# Verify replica set initialization
mongosh "mongodb://admin:password@localhost:27017/?authSource=admin" --eval "rs.status()"

# Restart MongoDB
docker-compose restart mongodb
```

#### 2. Replica Set Not Initialized

**Symptoms:** Error: "not master and slaveOk=false"

**Solution:**
```bash
# Initialize replica set manually
mongosh "mongodb://admin:password@localhost:27017/?authSource=admin" --eval "
  rs.initiate({
    _id: 'rs0',
    members: [{ _id: 0, host: 'localhost:27017' }]
  })
"

# Wait 10-15 seconds for initialization
# Verify status
mongosh "mongodb://admin:password@localhost:27017/?authSource=admin" --eval "rs.status()"
```

#### 3. Slow Query Performance

**Diagnosis:**
```javascript
// Enable profiling
db.setProfilingLevel(1, { slowms: 100 })

// Find slow queries
db.system.profile.find({ millis: { $gt: 100 } }).sort({ ts: -1 }).limit(10).pretty()
```

**Solutions:**
- Add missing indexes
- Use compound indexes for multi-field queries
- Limit result sets with pagination
- Use projection to return only needed fields

#### 4. High Memory Usage

**Check Memory:**
```bash
# Container memory usage
docker stats mongodb

# MongoDB server status
mongosh "mongodb://admin:password@localhost:27017/?authSource=admin" --eval "
  db.serverStatus().mem
"
```

**Solutions:**
- Increase Docker memory limit
- Add indexes to reduce collection scans
- Enable TTL indexes for automatic cleanup
- Archive old data

#### 5. Authentication Failed

**Solutions:**
```bash
# Verify credentials in .env
grep MONGO .env

# Test connection
mongosh "mongodb://admin:password@localhost:27017/bot_core?authSource=admin"

# Reset password (if needed)
docker exec -it mongodb mongosh --eval "
  use admin
  db.changeUserPassword('admin', 'new_password')
"
```

### Health Checks

**MongoDB Health:**
```bash
# Quick health check
mongosh "mongodb://admin:password@localhost:27017/?authSource=admin" --eval "db.adminCommand('ping')"

# Detailed status
mongosh "mongodb://admin:password@localhost:27017/?authSource=admin" --eval "
  printjson(db.serverStatus())
"

# Replica set health
mongosh "mongodb://admin:password@localhost:27017/?authSource=admin" --eval "
  printjson(rs.status())
"
```

**Database Statistics:**
```javascript
// Database size and stats
db.stats()

// Collection stats
db.trades.stats()

// Index sizes
db.trades.aggregate([
  { $collStats: { storageStats: {} } }
])
```

---

## Security Best Practices

### 1. Authentication

**Always Use Authentication:**
```bash
# NEVER use MongoDB without authentication in production
# BAD: mongodb://localhost:27017/
# GOOD: mongodb://username:password@localhost:27017/?authSource=admin
```

**User Roles:**
- `bot_core_admin` - Full database access (migrations, admin tasks)
- `bot_core_app` - Read/write access (application use)
- `bot_core_readonly` - Read-only access (analytics, reporting)

### 2. Encryption

**Transport Encryption (TLS/SSL):**
```bash
# Production: Always use TLS
mongodb+srv://...  # Enforces TLS

# Docker: Enable TLS with certificates
--tlsMode requireTLS \
--tlsCertificateKeyFile /etc/ssl/mongodb.pem
```

**At-Rest Encryption:**
- MongoDB Atlas: Enabled by default
- Self-hosted: Use WiredTiger encryption
- Docker volumes: Use encrypted volumes (LUKS, dm-crypt)

### 3. Network Security

**Firewall Rules:**
```bash
# Allow only trusted IPs
# Production: Use VPC/private network
# Development: Bind to localhost only

# Docker: Use internal network
networks:
  - bot-network  # Not exposed to host
```

**MongoDB Configuration:**
```yaml
# Bind to specific interface
--bind_ip localhost,192.168.1.100

# Disable HTTP status interface
--nohttpinterface
```

### 4. Secret Management

**Environment Variables:**
```bash
# NEVER commit .env file
# Use secret management: AWS Secrets Manager, HashiCorp Vault

# Rotate credentials regularly
MONGO_ROOT_PASSWORD=$(openssl rand -base64 32)
```

**API Key Encryption:**
```javascript
// Store encrypted API keys
{
  key_hash: "sha256_hash",
  encrypted_secret: "AES_encrypted_value"
}
```

### 5. Audit Logging

**Enable Audit Log:**
```javascript
// Track all database operations
db.adminCommand({
  auditLog: {
    destination: 'file',
    format: 'JSON',
    path: '/var/log/mongodb/audit.json'
  }
})
```

**Monitor Audit Logs:**
- Failed authentication attempts
- Privilege escalations
- Schema modifications
- Data exports

### 6. Backup Security

**Encrypt Backups:**
```bash
# Encrypt backup files
mongodump ... | gpg --encrypt --recipient backup@bot-core.com > backup.gpg

# Secure backup storage
# - Use encrypted S3 buckets
# - Enable versioning
# - Restrict access with IAM policies
```

### 7. Regular Security Audits

**Monthly Checks:**
- [ ] Review user permissions
- [ ] Audit database access logs
- [ ] Update MongoDB version (security patches)
- [ ] Rotate credentials
- [ ] Test backup restoration
- [ ] Review firewall rules

---

## Additional Resources

### MongoDB Documentation
- [MongoDB Manual](https://docs.mongodb.com/manual/)
- [Replica Sets](https://docs.mongodb.com/manual/replication/)
- [Time-Series Collections](https://docs.mongodb.com/manual/core/timeseries-collections/)
- [Security Checklist](https://docs.mongodb.com/manual/administration/security-checklist/)

### Bot Core Documentation
- **Schema Specification:** `/Users/dungngo97/Documents/bot-core/specs/02-design/2.2-database/DB-SCHEMA.md`
- **Integration Spec:** `/Users/dungngo97/Documents/bot-core/specs/INTEGRATION_SPEC.md`
- **Troubleshooting Guide:** `/Users/dungngo97/Documents/bot-core/docs/TROUBLESHOOTING.md`

### Support

For database issues:
1. Check MongoDB logs: `docker logs mongodb`
2. Run init script: `./scripts/init-db.sh`
3. Verify setup: See [Troubleshooting](#troubleshooting) section
4. Review audit logs: `db.audit_logs.find().sort({ created_at: -1 }).limit(10)`

---

**Last Updated:** 2025-11-18
**Maintained By:** Database Administrator Agent
**Status:** Production-Ready ✓
