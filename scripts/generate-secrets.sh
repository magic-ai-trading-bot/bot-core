#!/bin/bash

# Generate secure secrets for Bot Core services
# Usage: ./scripts/generate-secrets.sh

echo "🔐 Generating secure secrets for Bot Core..."

# Function to generate secure random string
generate_secret() {
    openssl rand -base64 32 | tr -d "=+/" | cut -c1-$1
}

# Function to generate JWT secret
generate_jwt_secret() {
    openssl rand -base64 64 | tr -d "\n"
}

# Check if .env exists
if [ -f .env ]; then
    echo "⚠️  .env file already exists. Do you want to update it? (y/N)"
    read -r response
    if [[ ! "$response" =~ ^[Yy]$ ]]; then
        echo "Aborted."
        exit 1
    fi
fi

# Copy .env.example if .env doesn't exist
if [ ! -f .env ]; then
    cp .env.example .env
fi

# Generate and update secrets
echo "📝 Generating Inter-Service Token..."
INTER_SERVICE_TOKEN=$(generate_secret 32)
sed -i.bak "s/^INTER_SERVICE_TOKEN=.*/INTER_SERVICE_TOKEN=$INTER_SERVICE_TOKEN/" .env

echo "📝 Generating Rust API Key..."
RUST_API_KEY=$(generate_secret 32)
sed -i.bak "s/^RUST_API_KEY=.*/RUST_API_KEY=$RUST_API_KEY/" .env

echo "📝 Generating Python API Key..."
PYTHON_API_KEY=$(generate_secret 32)
sed -i.bak "s/^PYTHON_API_KEY=.*/PYTHON_API_KEY=$PYTHON_API_KEY/" .env

echo "📝 Generating JWT Secret..."
JWT_SECRET=$(generate_jwt_secret)
sed -i.bak "s/^JWT_SECRET=.*/JWT_SECRET=$JWT_SECRET/" .env

echo "📝 Generating Dashboard Session Secret..."
DASHBOARD_SESSION_SECRET=$(generate_secret 32)
sed -i.bak "s/^DASHBOARD_SESSION_SECRET=.*/DASHBOARD_SESSION_SECRET=$DASHBOARD_SESSION_SECRET/" .env

echo "📝 Generating PostgreSQL Password..."
POSTGRES_PASSWORD=$(generate_secret 24)
sed -i.bak "s/^POSTGRES_PASSWORD=.*/POSTGRES_PASSWORD=$POSTGRES_PASSWORD/" .env

echo "📝 Generating Redis Password..."
REDIS_PASSWORD=$(generate_secret 24)
sed -i.bak "s/^REDIS_PASSWORD=.*/REDIS_PASSWORD=$REDIS_PASSWORD/" .env

# Clean up backup files
rm -f .env.bak*

echo "✅ Secrets generated successfully!"
echo ""
echo "⚠️  IMPORTANT: Please update the following manually:"
echo "  - DATABASE_URL: Your MongoDB connection string"
echo "  - BINANCE_API_KEY & BINANCE_SECRET_KEY: Your Binance API credentials"
echo "  - OPENAI_API_KEY: Your OpenAI API key"
echo "  - Any other service-specific credentials"
echo ""
echo "🔒 Remember: NEVER commit .env to version control!"