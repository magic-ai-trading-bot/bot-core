#!/bin/bash

# MongoDB Seed Data Initialization Script
# This script automatically seeds MongoDB with sample data on first startup

set -e

echo "🌱 Checking if MongoDB seed data is needed..."

# Wait for MongoDB to be ready
echo "⏳ Waiting for MongoDB to be ready..."
sleep 5

# Check if users collection already has data in bot_core database
# Both Rust and Python now use 'bot_core' database
USER_COUNT=$(docker exec mongodb mongosh -u admin -p secure_mongo_password_change_me --authenticationDatabase admin --quiet --eval "db.getSiblingDB('bot_core').users.countDocuments()" 2>/dev/null || echo "0")

if [ "$USER_COUNT" -gt 0 ]; then
    echo "✅ Seed data already exists ($USER_COUNT users in bot_core). Skipping seed."
    exit 0
fi

echo "📝 No seed data found. Creating sample data in bot_core database..."

# Run seed script (seeds all data into bot_core database)
docker exec -i mongodb mongosh -u admin -p secure_mongo_password_change_me --authenticationDatabase admin < scripts/seed-mongodb.js

echo "✅ MongoDB seed data created successfully!"
echo ""
echo "📊 You can now:"
echo "   - Login to dashboard: http://localhost:3000"
echo "   - Email: trader@botcore.com"
echo "   - Password: password123"
echo ""
echo "   - Or use admin: admin@botcore.com / password123"
echo ""
echo "   - Connect MongoDB Compass:"
echo "     mongodb://admin:secure_mongo_password_change_me@localhost:27017/bot_core?authSource=admin"
echo ""
