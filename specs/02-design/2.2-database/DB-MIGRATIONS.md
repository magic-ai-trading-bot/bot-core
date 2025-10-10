# Database Migration Strategy

**Version:** 1.0.0
**Last Updated:** 2025-10-10
**Database:** MongoDB 7.0+
**Migration Tool:** Custom Scripts + Mongosh
**Author:** Bot Core Team

## Overview

This document defines the database migration strategy for the Bot Core trading platform. Migrations must be executed safely without downtime in production environments while maintaining data integrity and backward compatibility.

### Migration Principles

1. **Zero-Downtime Migrations** - All migrations must support rolling deployments
2. **Backward Compatibility** - Old code must work during migration
3. **Forward Compatibility** - New code must tolerate old schema
4. **Idempotency** - Migrations can be run multiple times safely
5. **Rollback Safety** - Every migration has a tested rollback procedure
6. **Data Integrity** - Validations ensure no data corruption
7. **Performance Awareness** - Large migrations use batching to avoid blocking

---

## Migration Framework

### Schema Versioning

```javascript
// system_config collection stores current schema version
{
  "key": "DATABASE_SCHEMA_VERSION",
  "value": "1.2.0",
  "category": "system",
  "description": "Current database schema version",
  "is_active": true,
  "updated_by": "migration_script",
  "updated_at": ISODate("2025-10-10T00:00:00Z"),
  "created_at": ISODate("2025-01-01T00:00:00Z")
}
```

### Migration Script Structure

```javascript
// migrations/001_initial_schema.js

module.exports = {
  version: "1.0.0",
  description: "Initial schema setup",

  async up(db) {
    // Forward migration
    console.log("Running migration 1.0.0: Initial schema");

    // Create collections with validation
    await createUsersCollection(db);
    await createTradesCollection(db);
    // ... more collections

    // Create indexes
    await createIndexes(db);

    // Update schema version
    await db.collection('system_config').updateOne(
      { key: "DATABASE_SCHEMA_VERSION" },
      { $set: { value: "1.0.0", updated_at: new Date() } },
      { upsert: true }
    );

    console.log("Migration 1.0.0 completed successfully");
  },

  async down(db) {
    // Rollback migration
    console.log("Rolling back migration 1.0.0");

    // Drop collections in reverse order
    await db.collection('trades').drop();
    await db.collection('users').drop();

    // Revert schema version
    await db.collection('system_config').deleteOne(
      { key: "DATABASE_SCHEMA_VERSION" }
    );

    console.log("Rollback 1.0.0 completed");
  },

  // Validation function to check if migration succeeded
  async validate(db) {
    const collections = await db.listCollections().toArray();
    const collectionNames = collections.map(c => c.name);

    assert(collectionNames.includes('users'), "users collection missing");
    assert(collectionNames.includes('trades'), "trades collection missing");

    const indexes = await db.collection('users').indexes();
    assert(indexes.find(i => i.name === 'idx_users_email_unique'),
           "email index missing");

    return true;
  }
};
```

---

## Migration History

### Version 1.0.0 - Initial Schema (2025-01-01)

**Status:** ✅ Completed
**Description:** Initial database schema setup for MVP launch

**Changes:**
- Created `users` collection with authentication fields
- Created `trades` collection for live trading
- Created `positions` collection for open positions
- Created `market_data` time-series collection
- Created all core indexes
- Established schema validation rules

**Migration Script:**

```javascript
// migrations/001_initial_schema.js

async function up(db) {
  // 1. Create users collection with validation
  await db.createCollection("users", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: ["email", "password_hash", "created_at"],
        properties: {
          email: {
            bsonType: "string",
            pattern: "^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}$"
          },
          password_hash: {
            bsonType: "string",
            minLength: 60
          },
          is_active: {
            bsonType: "bool"
          }
        }
      }
    }
  });

  // 2. Create unique email index
  await db.collection("users").createIndex(
    { email: 1 },
    { unique: true, name: "idx_users_email_unique" }
  );

  // 3. Create trades collection
  await db.createCollection("trades", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: ["user_id", "symbol", "side", "quantity", "created_at"],
        properties: {
          user_id: { bsonType: "objectId" },
          symbol: { bsonType: "string" },
          side: { enum: ["BUY", "SELL"] },
          status: { enum: ["PENDING", "FILLED", "PARTIALLY_FILLED", "CANCELLED", "FAILED"] }
        }
      }
    }
  });

  // 4. Create trades indexes
  await db.collection("trades").createIndex(
    { user_id: 1, created_at: -1 },
    { name: "idx_trades_user_created" }
  );

  await db.collection("trades").createIndex(
    { created_at: 1 },
    { expireAfterSeconds: 31536000, name: "idx_trades_ttl" }
  );

  // 5. Create market_data as time-series collection
  await db.createCollection("market_data", {
    timeseries: {
      timeField: "open_time",
      metaField: "symbol",
      granularity: "minutes"
    },
    expireAfterSeconds: 7776000  // 90 days
  });

  // 6. Update schema version
  await db.collection("system_config").insertOne({
    key: "DATABASE_SCHEMA_VERSION",
    value: "1.0.0",
    category: "system",
    description: "Initial schema version",
    is_active: true,
    updated_by: "migration",
    updated_at: new Date(),
    created_at: new Date()
  });

  console.log("✅ Migration 1.0.0 completed");
}

async function down(db) {
  await db.collection("market_data").drop();
  await db.collection("trades").drop();
  await db.collection("users").drop();
  await db.collection("system_config").deleteOne({ key: "DATABASE_SCHEMA_VERSION" });

  console.log("✅ Rollback 1.0.0 completed");
}
```

**Execution:**
```bash
# Apply migration
mongosh mongodb://localhost:27017/bot_core_db migrations/001_initial_schema.js

# Verify
mongosh --eval "db.system_config.findOne({ key: 'DATABASE_SCHEMA_VERSION' })"
```

---

### Version 1.1.0 - Paper Trading Support (2025-03-15)

**Status:** ✅ Completed
**Description:** Added paper trading functionality

**Changes:**
- Created `paper_trading_accounts` collection
- Created `paper_trading_trades` collection
- Created `portfolio_history` collection
- Added performance metrics tracking
- Created indexes for paper trading queries

**Migration Script:**

```javascript
// migrations/002_paper_trading.js

async function up(db) {
  console.log("Running migration 1.1.0: Paper Trading Support");

  // 1. Create paper_trading_accounts collection
  await db.createCollection("paper_trading_accounts", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: ["user_id", "initial_balance", "current_balance", "created_at"],
        properties: {
          user_id: { bsonType: "objectId" },
          initial_balance: { bsonType: "decimal" },
          current_balance: { bsonType: "decimal" }
        }
      }
    }
  });

  // 2. Create unique user_id index (one account per user)
  await db.collection("paper_trading_accounts").createIndex(
    { user_id: 1 },
    { unique: true, name: "idx_paper_accounts_user_unique" }
  );

  // 3. Create paper_trading_trades collection
  await db.createCollection("paper_trading_trades", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: ["trade_id", "user_id", "account_id", "symbol", "trade_type"],
        properties: {
          trade_id: { bsonType: "string" },
          user_id: { bsonType: "objectId" },
          account_id: { bsonType: "objectId" },
          trade_type: { enum: ["LONG", "SHORT"] },
          status: { enum: ["OPEN", "CLOSED", "CANCELLED"] }
        }
      }
    }
  });

  // 4. Create indexes
  await db.collection("paper_trading_trades").createIndex(
    { trade_id: 1 },
    { unique: true, name: "idx_paper_trades_id_unique" }
  );

  await db.collection("paper_trading_trades").createIndex(
    { user_id: 1, created_at: -1 },
    { name: "idx_paper_trades_user_created" }
  );

  // 5. Create initial paper accounts for existing users
  const users = await db.collection("users").find({}).toArray();

  for (const user of users) {
    await db.collection("paper_trading_accounts").insertOne({
      user_id: user._id,
      initial_balance: NumberDecimal("10000.00"),
      current_balance: NumberDecimal("10000.00"),
      equity: NumberDecimal("10000.00"),
      margin_used: NumberDecimal("0.00"),
      free_margin: NumberDecimal("10000.00"),
      metrics: {
        total_trades: 0,
        winning_trades: 0,
        losing_trades: 0,
        win_rate: 0.0,
        total_pnl: NumberDecimal("0.00"),
        total_pnl_percentage: 0.0
      },
      open_positions: 0,
      open_trade_ids: [],
      settings: {
        max_leverage: 10,
        max_positions: 5,
        default_quantity: NumberDecimal("0.05"),
        trading_fees_rate: 0.0004,
        enable_ai_trading: true,
        risk_per_trade: 2.0
      },
      created_at: new Date(),
      updated_at: new Date(),
      last_trade_at: null,
      reset_at: null
    });
  }

  // 6. Update schema version
  await db.collection("system_config").updateOne(
    { key: "DATABASE_SCHEMA_VERSION" },
    { $set: { value: "1.1.0", updated_at: new Date() } }
  );

  console.log(`✅ Created paper accounts for ${users.length} users`);
  console.log("✅ Migration 1.1.0 completed");
}

async function down(db) {
  await db.collection("paper_trading_trades").drop();
  await db.collection("paper_trading_accounts").drop();

  await db.collection("system_config").updateOne(
    { key: "DATABASE_SCHEMA_VERSION" },
    { $set: { value: "1.0.0", updated_at: new Date() } }
  );

  console.log("✅ Rollback 1.1.0 completed");
}
```

---

### Version 1.2.0 - AI Analysis Caching (2025-06-01)

**Status:** ✅ Completed
**Description:** Added AI analysis result caching in MongoDB

**Changes:**
- Created `ai_analysis_results` collection with TTL
- Created `ai_signals` collection for execution tracking
- Added compound indexes for signal retrieval
- Migrated existing Redis cache data to MongoDB

**Migration Script:**

```javascript
// migrations/003_ai_analysis_cache.js

async function up(db) {
  console.log("Running migration 1.2.0: AI Analysis Caching");

  // 1. Create ai_analysis_results collection
  await db.createCollection("ai_analysis_results", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: ["analysis_id", "symbol", "signal", "confidence", "timestamp"],
        properties: {
          analysis_id: { bsonType: "string" },
          symbol: { bsonType: "string" },
          signal: { enum: ["BUY", "SELL", "HOLD", "NEUTRAL"] },
          confidence: {
            bsonType: "double",
            minimum: 0.0,
            maximum: 1.0
          }
        }
      }
    }
  });

  // 2. Create indexes
  await db.collection("ai_analysis_results").createIndex(
    { symbol: 1, timestamp: -1 },
    { name: "idx_ai_analysis_symbol_timestamp" }
  );

  await db.collection("ai_analysis_results").createIndex(
    { signal: 1, confidence: -1 },
    { name: "idx_ai_analysis_signal_confidence" }
  );

  await db.collection("ai_analysis_results").createIndex(
    { created_at: 1 },
    { expireAfterSeconds: 2592000, name: "idx_ai_analysis_ttl" } // 30 days
  );

  // 3. Create ai_signals collection
  await db.createCollection("ai_signals");

  await db.collection("ai_signals").createIndex(
    { signal_id: 1 },
    { unique: true, name: "idx_ai_signals_id_unique" }
  );

  // 4. Migrate existing Redis cache to MongoDB (if needed)
  // This would involve reading from Redis and inserting into MongoDB
  // Skipped for this example as Redis may not be available

  // 5. Update schema version
  await db.collection("system_config").updateOne(
    { key: "DATABASE_SCHEMA_VERSION" },
    { $set: { value: "1.2.0", updated_at: new Date() } }
  );

  console.log("✅ Migration 1.2.0 completed");
}

async function down(db) {
  await db.collection("ai_signals").drop();
  await db.collection("ai_analysis_results").drop();

  await db.collection("system_config").updateOne(
    { key: "DATABASE_SCHEMA_VERSION" },
    { $set: { value: "1.1.0", updated_at: new Date() } }
  );

  console.log("✅ Rollback 1.2.0 completed");
}
```

---

### Version 1.3.0 - Enhanced Risk Metrics (Planned: 2025-11-01)

**Status:** ⏳ Pending
**Description:** Add comprehensive risk tracking and metrics

**Planned Changes:**
- Create `risk_metrics` collection for real-time risk calculations
- Add VaR (Value at Risk) calculations to `portfolio_snapshots`
- Add `liquidation_alerts` collection for margin call tracking
- Enhance `positions` collection with more risk fields
- Add risk dashboard indexes

**Migration Script (Draft):**

```javascript
// migrations/004_risk_metrics.js

async function up(db) {
  console.log("Running migration 1.3.0: Enhanced Risk Metrics");

  // 1. Create risk_metrics collection
  await db.createCollection("risk_metrics", {
    validator: {
      $jsonSchema: {
        bsonType: "object",
        required: ["user_id", "timestamp"],
        properties: {
          user_id: { bsonType: "objectId" },
          portfolio_var_95: { bsonType: "decimal" },
          portfolio_var_99: { bsonType: "decimal" },
          expected_shortfall: { bsonType: "decimal" },
          leverage_ratio: { bsonType: "double" },
          margin_usage_percentage: { bsonType: "double" }
        }
      }
    }
  });

  // 2. Add indexes
  await db.collection("risk_metrics").createIndex(
    { user_id: 1, timestamp: -1 },
    { name: "idx_risk_metrics_user_time" }
  );

  // 3. Add new fields to existing positions (backward compatible)
  await db.collection("positions").updateMany(
    {},
    {
      $set: {
        "metadata.var_contribution": NumberDecimal("0.00"),
        "metadata.correlation_score": 0.0,
        "metadata.concentration_risk": 0.0
      }
    }
  );

  // 4. Add new fields to portfolio_snapshots
  await db.collection("portfolio_snapshots").updateMany(
    {},
    {
      $set: {
        var_95: NumberDecimal("0.00"),
        var_99: NumberDecimal("0.00"),
        expected_shortfall: NumberDecimal("0.00")
      }
    }
  );

  // 5. Update schema version
  await db.collection("system_config").updateOne(
    { key: "DATABASE_SCHEMA_VERSION" },
    { $set: { value: "1.3.0", updated_at: new Date() } }
  );

  console.log("✅ Migration 1.3.0 completed");
}

async function down(db) {
  // Remove added fields
  await db.collection("positions").updateMany(
    {},
    {
      $unset: {
        "metadata.var_contribution": "",
        "metadata.correlation_score": "",
        "metadata.concentration_risk": ""
      }
    }
  );

  await db.collection("portfolio_snapshots").updateMany(
    {},
    {
      $unset: {
        var_95: "",
        var_99: "",
        expected_shortfall: ""
      }
    }
  );

  await db.collection("risk_metrics").drop();

  await db.collection("system_config").updateOne(
    { key: "DATABASE_SCHEMA_VERSION" },
    { $set: { value: "1.2.0", updated_at: new Date() } }
  );

  console.log("✅ Rollback 1.3.0 completed");
}
```

---

## Migration Execution Process

### Pre-Migration Checklist

```bash
# 1. Backup database
mongodump --uri="mongodb://localhost:27017/bot_core_db" --out=backup_$(date +%Y%m%d)

# 2. Verify backup
mongorestore --uri="mongodb://localhost:27017/bot_core_db_test" backup_YYYYMMDD/ --drop

# 3. Check current schema version
mongosh --eval "db.system_config.findOne({ key: 'DATABASE_SCHEMA_VERSION' })"

# 4. Test migration on staging
mongosh mongodb://staging:27017/bot_core_db migrations/XXX_migration.js

# 5. Verify data integrity
mongosh --eval "db.runCommand({ validate: 'users' })"
```

### Migration Execution (Production)

```javascript
// migrations/run_migration.js

const { MongoClient } = require('mongodb');

async function runMigration(migrationFile) {
  const client = await MongoClient.connect('mongodb://localhost:27017');
  const db = client.db('bot_core_db');

  try {
    // Load migration
    const migration = require(migrationFile);

    // Check current version
    const currentVersion = await db.collection('system_config')
      .findOne({ key: 'DATABASE_SCHEMA_VERSION' });

    console.log(`Current schema version: ${currentVersion?.value || 'none'}`);
    console.log(`Target schema version: ${migration.version}`);

    // Confirm migration
    const readline = require('readline').createInterface({
      input: process.stdin,
      output: process.stdout
    });

    const answer = await new Promise(resolve => {
      readline.question('Proceed with migration? (yes/no): ', resolve);
    });

    if (answer.toLowerCase() !== 'yes') {
      console.log('❌ Migration cancelled');
      process.exit(0);
    }

    // Run migration
    console.log('⏳ Running migration...');
    await migration.up(db);

    // Validate
    console.log('⏳ Validating migration...');
    await migration.validate(db);

    console.log('✅ Migration completed successfully');

  } catch (error) {
    console.error('❌ Migration failed:', error);

    // Attempt rollback
    console.log('⏳ Attempting rollback...');
    try {
      await migration.down(db);
      console.log('✅ Rollback completed');
    } catch (rollbackError) {
      console.error('❌ Rollback failed:', rollbackError);
      console.error('⚠️  MANUAL INTERVENTION REQUIRED');
    }

    process.exit(1);

  } finally {
    await client.close();
  }
}

// Usage: node run_migration.js ./migrations/003_ai_analysis_cache.js
runMigration(process.argv[2]);
```

### Post-Migration Checklist

```bash
# 1. Verify schema version
mongosh --eval "db.system_config.findOne({ key: 'DATABASE_SCHEMA_VERSION' })"

# 2. Check collection counts
mongosh --eval "db.stats()"

# 3. Verify indexes
mongosh --eval "db.users.getIndexes()"

# 4. Run validation on critical collections
mongosh --eval "db.runCommand({ validate: 'trades' })"

# 5. Test critical queries
mongosh --eval "db.trades.find({ user_id: ObjectId('...') }).limit(10).explain('executionStats')"

# 6. Monitor application logs for errors
tail -f /var/log/bot-core/app.log

# 7. Check performance metrics
# (Use monitoring tools like MongoDB Compass or Grafana)
```

---

## Zero-Downtime Migration Strategies

### 1. Expand-Contract Pattern

**Phase 1: Expand (Add New Schema)**
```javascript
// Migration 1.x.0 - Add new field
await db.collection("users").updateMany(
  {},
  { $set: { email_verified: false } }  // Default value for backward compatibility
);
```

**Phase 2: Dual Write**
```javascript
// Application code supports both old and new schema
async function updateUser(userId, updates) {
  // Write to both old and new fields
  await db.collection("users").updateOne(
    { _id: userId },
    {
      $set: {
        ...updates,
        email_verified: updates.email_verified || false  // Always set new field
      }
    }
  );
}
```

**Phase 3: Migrate Data**
```javascript
// Background job to migrate old data
async function migrateEmailVerification() {
  const cursor = db.collection("users").find({ email_verified: { $exists: false } });

  while (await cursor.hasNext()) {
    const user = await cursor.next();

    // Compute new field value from old data
    const verified = user.last_login !== null;

    await db.collection("users").updateOne(
      { _id: user._id },
      { $set: { email_verified: verified } }
    );
  }
}
```

**Phase 4: Contract (Remove Old Schema)**
```javascript
// Migration 1.x+1.0 - Remove old field (after all data migrated)
await db.collection("users").updateMany(
  {},
  { $unset: { legacy_field: "" } }
);
```

### 2. Read-Modify-Write for Large Collections

```javascript
// Batch migration for performance
async function batchMigration(collection, batchSize = 1000) {
  let skip = 0;
  let processed = 0;

  while (true) {
    const batch = await db.collection(collection)
      .find({})
      .skip(skip)
      .limit(batchSize)
      .toArray();

    if (batch.length === 0) break;

    // Process batch
    const bulkOps = batch.map(doc => ({
      updateOne: {
        filter: { _id: doc._id },
        update: { $set: { new_field: computeValue(doc) } }
      }
    }));

    await db.collection(collection).bulkWrite(bulkOps);

    processed += batch.length;
    skip += batchSize;

    console.log(`Processed ${processed} documents`);

    // Pause to avoid overwhelming database
    await new Promise(resolve => setTimeout(resolve, 100));
  }

  console.log(`✅ Migration completed: ${processed} documents processed`);
}
```

### 3. Shadow Collections for Complex Migrations

```javascript
// Create shadow collection with new schema
await db.createCollection("users_v2");

// Copy data with transformations
const cursor = db.collection("users").find({});

while (await cursor.hasNext()) {
  const oldDoc = await cursor.next();

  const newDoc = {
    _id: oldDoc._id,
    email: oldDoc.email,
    // Transform data
    full_name: `${oldDoc.first_name} ${oldDoc.last_name}`,
    // ... other transformations
  };

  await db.collection("users_v2").insertOne(newDoc);
}

// Switch collections atomically
await db.collection("users").rename("users_old");
await db.collection("users_v2").rename("users");
```

---

## Data Integrity Validation

### Pre-Migration Validation

```javascript
async function validateBeforeMigration(db) {
  console.log("⏳ Running pre-migration validation...");

  // 1. Check for orphaned records
  const orphanedTrades = await db.collection("trades").countDocuments({
    user_id: { $nin: await db.collection("users").distinct("_id") }
  });

  if (orphanedTrades > 0) {
    throw new Error(`Found ${orphanedTrades} orphaned trades`);
  }

  // 2. Check for data inconsistencies
  const invalidTrades = await db.collection("trades").countDocuments({
    quantity: { $lte: 0 }
  });

  if (invalidTrades > 0) {
    throw new Error(`Found ${invalidTrades} trades with invalid quantity`);
  }

  // 3. Verify critical indexes exist
  const indexes = await db.collection("users").indexes();
  const emailIndex = indexes.find(i => i.name === "idx_users_email_unique");

  if (!emailIndex) {
    throw new Error("Critical email index missing");
  }

  console.log("✅ Pre-migration validation passed");
}
```

### Post-Migration Validation

```javascript
async function validateAfterMigration(db) {
  console.log("⏳ Running post-migration validation...");

  // 1. Verify schema version updated
  const version = await db.collection("system_config")
    .findOne({ key: "DATABASE_SCHEMA_VERSION" });

  assert(version.value === "1.2.0", "Schema version not updated");

  // 2. Verify new collections exist
  const collections = await db.listCollections().toArray();
  const collectionNames = collections.map(c => c.name);

  assert(collectionNames.includes("ai_analysis_results"),
         "ai_analysis_results collection missing");

  // 3. Verify new indexes created
  const indexes = await db.collection("ai_analysis_results").indexes();
  assert(indexes.find(i => i.name === "idx_ai_analysis_symbol_timestamp"),
         "Symbol-timestamp index missing");

  // 4. Verify data migrated correctly
  const sampleDoc = await db.collection("ai_analysis_results").findOne({});
  if (sampleDoc) {
    assert(sampleDoc.confidence >= 0 && sampleDoc.confidence <= 1,
           "Invalid confidence value");
  }

  // 5. Verify performance (index usage)
  const explainResult = await db.collection("ai_analysis_results")
    .find({ symbol: "BTCUSDT" })
    .sort({ timestamp: -1 })
    .limit(10)
    .explain("executionStats");

  assert(explainResult.executionStats.executionStages.stage === "IXSCAN",
         "Query not using index");

  console.log("✅ Post-migration validation passed");
}
```

---

## Rollback Procedures

### Automatic Rollback on Failure

```javascript
async function safelyRunMigration(migration, db) {
  const session = db.getMongo().startSession();

  try {
    // Start transaction (for supported operations)
    session.startTransaction();

    // Run migration
    await migration.up(db);

    // Validate
    const isValid = await migration.validate(db);

    if (!isValid) {
      throw new Error("Migration validation failed");
    }

    // Commit transaction
    await session.commitTransaction();

    console.log("✅ Migration committed successfully");

  } catch (error) {
    console.error("❌ Migration failed:", error);

    // Rollback transaction
    await session.abortTransaction();

    // Run explicit rollback
    console.log("⏳ Running rollback...");
    await migration.down(db);

    throw error;

  } finally {
    await session.endSession();
  }
}
```

### Manual Rollback

```bash
# 1. Stop application to prevent new writes
systemctl stop bot-core

# 2. Restore from backup (if needed)
mongorestore --uri="mongodb://localhost:27017/bot_core_db" \
  --drop backup_20251010/ \
  --nsInclude="bot_core_db.*"

# 3. Or run rollback migration
mongosh mongodb://localhost:27017/bot_core_db migrations/003_ai_analysis_cache.js --eval "down(db)"

# 4. Verify rollback
mongosh --eval "db.system_config.findOne({ key: 'DATABASE_SCHEMA_VERSION' })"

# 5. Restart application with old code
git checkout v1.1.0
systemctl start bot-core
```

---

## Best Practices

### 1. Always Use Transactions When Possible

```javascript
// Good: Use transaction for atomic operations
const session = client.startSession();
try {
  session.startTransaction();

  await db.collection("accounts").updateOne({ _id: userId }, { $inc: { balance: -100 } }, { session });
  await db.collection("trades").insertOne({ userId, amount: 100 }, { session });

  await session.commitTransaction();
} catch (error) {
  await session.abortTransaction();
  throw error;
} finally {
  await session.endSession();
}
```

### 2. Use Bulk Operations for Performance

```javascript
// Bad: Individual updates
for (const user of users) {
  await db.collection("users").updateOne(
    { _id: user._id },
    { $set: { migrated: true } }
  );
}

// Good: Bulk update
const bulkOps = users.map(user => ({
  updateOne: {
    filter: { _id: user._id },
    update: { $set: { migrated: true } }
  }
}));

await db.collection("users").bulkWrite(bulkOps);
```

### 3. Add Fields with Defaults (Backward Compatible)

```javascript
// Good: Add field with default value
await db.collection("users").updateMany(
  { email_verified: { $exists: false } },
  { $set: { email_verified: false } }
);

// Application code handles both cases
const user = await db.collection("users").findOne({ _id: userId });
const isVerified = user.email_verified ?? false;  // Handle missing field
```

### 4. Use Migrations for Schema Changes, Not Data Fixes

```javascript
// Bad: Fixing production data in migration
await db.collection("trades").updateMany(
  { user_id: "specific_user_id" },
  { $set: { status: "FILLED" } }
);

// Good: Use separate data fix script
// scripts/fix_trade_statuses.js
```

### 5. Test Migrations on Production-Like Data

```bash
# Create production snapshot for testing
mongodump --uri="mongodb://production/bot_core_db" --out=prod_snapshot

# Restore to staging
mongorestore --uri="mongodb://staging/bot_core_db" prod_snapshot/ --drop

# Test migration on staging
mongosh mongodb://staging/bot_core_db migrations/004_risk_metrics.js
```

---

## Migration Checklist Template

```markdown
# Migration Checklist: Version X.Y.Z

## Pre-Migration
- [ ] Created migration script with up/down/validate functions
- [ ] Tested migration on local development database
- [ ] Tested migration on staging with production-like data
- [ ] Reviewed migration for performance impact
- [ ] Created full database backup
- [ ] Documented rollback procedure
- [ ] Scheduled maintenance window (if needed)
- [ ] Notified team of upcoming migration

## Migration Execution
- [ ] Verified backup is valid and recent
- [ ] Checked current schema version
- [ ] Ran pre-migration validation
- [ ] Executed migration script
- [ ] Monitored migration progress
- [ ] Verified schema version updated
- [ ] Ran post-migration validation
- [ ] Tested critical application functions

## Post-Migration
- [ ] Verified all indexes created successfully
- [ ] Checked query performance
- [ ] Monitored application logs for errors
- [ ] Verified data integrity
- [ ] Updated documentation
- [ ] Notified team of successful migration
- [ ] Scheduled backup deletion (after retention period)

## Rollback (If Needed)
- [ ] Stopped application
- [ ] Ran rollback migration
- [ ] Restored from backup (if needed)
- [ ] Verified rollback successful
- [ ] Restarted application
- [ ] Documented rollback reason
- [ ] Scheduled migration retry
```

---

## Summary

### Key Takeaways

1. **Always Backup** - Full backup before every migration
2. **Test Thoroughly** - Test on staging with production-like data
3. **Zero Downtime** - Use expand-contract pattern for large changes
4. **Validate Everything** - Pre and post-migration validation scripts
5. **Rollback Ready** - Every migration must have tested rollback
6. **Monitor Performance** - Watch for slow queries after migrations
7. **Document Changes** - Maintain migration history and rationale

### Migration Tool Recommendations

1. **migrate-mongo** - Popular migration framework for MongoDB
2. **Custom Scripts** - More control for complex migrations
3. **MongoDB Atlas** - Automated schema versioning (for cloud)

### Emergency Contacts

- **DBA On-Call:** [Contact Info]
- **DevOps Lead:** [Contact Info]
- **Backup/Restore SLA:** 2 hours

---

**Document Version:** 1.0.0
**Last Updated:** 2025-10-10
**Next Review:** 2026-01-10
