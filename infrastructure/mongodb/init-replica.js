// MongoDB Replica Set Initialization Script
rs.initiate({
  _id: "rs0",
  members: [
    { _id: 0, host: "mongodb-primary:27017", priority: 2 },
    { _id: 1, host: "mongodb-secondary1:27017", priority: 1 },
    { _id: 2, host: "mongodb-secondary2:27017", priority: 1 },
    { _id: 3, host: "mongodb-arbiter:27017", arbiterOnly: true }
  ]
});

// Wait for replica set to initialize
sleep(5000);

// Create application user
db = db.getSiblingDB('admin');
db.createUser({
  user: 'bot-app',
  pwd: process.env.MONGO_APP_PASSWORD || 'app-password',
  roles: [
    { role: 'readWrite', db: 'bot_core' },
    { role: 'dbAdmin', db: 'bot_core' }
  ]
});

// Create read-only user for analytics
db.createUser({
  user: 'bot-analytics',
  pwd: process.env.MONGO_ANALYTICS_PASSWORD || 'analytics-password',
  roles: [
    { role: 'read', db: 'bot_core' }
  ]
});

// Create database and collections
db = db.getSiblingDB('bot_core');

// Create collections with validation
db.createCollection('trades', {
  validator: {
    $jsonSchema: {
      bsonType: 'object',
      required: ['symbol', 'side', 'price', 'quantity', 'timestamp'],
      properties: {
        symbol: { bsonType: 'string' },
        side: { enum: ['BUY', 'SELL'] },
        price: { bsonType: 'number', minimum: 0 },
        quantity: { bsonType: 'number', minimum: 0 },
        timestamp: { bsonType: 'date' }
      }
    }
  }
});

db.createCollection('market_data', {
  timeseries: {
    timeField: 'timestamp',
    metaField: 'symbol',
    granularity: 'minutes'
  }
});

// Create indexes
db.trades.createIndex({ symbol: 1, timestamp: -1 });
db.trades.createIndex({ timestamp: -1 });
db.market_data.createIndex({ symbol: 1, timestamp: -1 });

print('Replica set initialized successfully');