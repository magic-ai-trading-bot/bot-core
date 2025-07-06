#!/bin/bash

# 🚀 Trading Bot Deployment Script for Fly.io
# This script deploys all services in the correct order

set -e  # Exit on any error

echo "🚀 Starting Trading Bot deployment to Fly.io..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_step() {
    echo -e "${BLUE}==>${NC} $1"
}

print_success() {
    echo -e "${GREEN}✅${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}⚠️${NC} $1"
}

print_error() {
    echo -e "${RED}❌${NC} $1"
}

# Check if flyctl is installed
if ! command -v flyctl &> /dev/null; then
    print_error "flyctl is not installed. Please install it first:"
    echo "curl -L https://fly.io/install.sh | sh"
    exit 1
fi

# Check if user is logged in
if ! flyctl auth whoami &> /dev/null; then
    print_error "You are not logged in to Fly.io. Please run:"
    echo "flyctl auth login"
    exit 1
fi

print_success "Fly.io CLI is ready!"

# Step 1: Deploy MongoDB Database
print_step "Deploying MongoDB Database..."
if flyctl apps list | grep -q "trading-bot-database"; then
    print_warning "Database app already exists, recreating for clean deployment..."
    flyctl apps destroy trading-bot-database --yes || true
    sleep 5
fi
print_step "Creating and deploying database app..."
flyctl launch --config ../fly-database.toml --name trading-bot-database --org personal --remote-only --no-deploy
flyctl deploy --config ../fly-database.toml --app trading-bot-database --remote-only

# Wait for database to be ready
print_step "Waiting for database to be ready..."
sleep 30

# Step 2: Deploy Python AI Service
print_step "Deploying Python AI Service..."
cd ../python-ai-service
if flyctl apps list | grep -q "trading-bot-ai-service"; then
    print_warning "AI service app already exists, recreating for clean deployment..."
    flyctl apps destroy trading-bot-ai-service --yes || true
    sleep 5
fi
print_step "Creating and deploying AI service app..."
flyctl launch --name trading-bot-ai-service --org personal --remote-only --no-deploy
flyctl deploy --app trading-bot-ai-service --remote-only
cd ../scripts

# Wait for AI service to be ready
print_step "Waiting for AI service to be ready..."
sleep 20

# Step 3: Deploy Rust Core Engine
print_step "Deploying Rust Core Engine..."
cd ../rust-core-engine
if flyctl apps list | grep -q "trading-bot-rust-engine"; then
    print_warning "Rust engine app already exists, recreating for clean deployment..."
    flyctl apps destroy trading-bot-rust-engine --yes || true
    sleep 5
fi
print_step "Creating and deploying Rust engine app..."
flyctl launch --name trading-bot-rust-engine --org personal --remote-only --no-deploy

# Set secrets for Rust engine
print_step "Setting up secrets for Rust engine..."
echo "Please set the following secrets:"
echo "BINANCE_API_KEY, BINANCE_SECRET_KEY, JWT_SECRET, MONGO_PASSWORD"
echo "Example:"
echo "flyctl secrets set BINANCE_API_KEY=your_api_key BINANCE_SECRET_KEY=your_secret JWT_SECRET=your_jwt_secret MONGO_PASSWORD=your_mongo_password --app trading-bot-rust-engine"

flyctl deploy --app trading-bot-rust-engine --remote-only
cd ../scripts

# Wait for Rust engine to be ready
print_step "Waiting for Rust engine to be ready..."
sleep 30

# Step 4: Deploy React Dashboard
print_step "Deploying React Dashboard..."
cd ../nextjs-ui-dashboard

# Update environment variables for production
print_step "Setting up production environment variables..."
cat > .env.production << EOF
VITE_API_BASE_URL=https://trading-bot-rust-engine.fly.dev
VITE_WS_URL=wss://trading-bot-rust-engine.fly.dev
NODE_ENV=production
EOF

if flyctl apps list | grep -q "trading-bot-dashboard"; then
    print_warning "Dashboard app already exists, recreating for clean deployment..."
    flyctl apps destroy trading-bot-dashboard --yes || true
    sleep 5
fi
print_step "Creating and deploying dashboard app..."
flyctl launch --name trading-bot-dashboard --org personal --remote-only --no-deploy
flyctl deploy --app trading-bot-dashboard --remote-only
cd ../scripts

# Step 5: Check all deployments
print_step "Checking deployment status..."
echo ""
echo "📊 Deployment Summary:"
echo "======================"

check_app() {
    local app_name=$1
    local display_name=$2
    
    if flyctl status --app $app_name &> /dev/null; then
        print_success "$display_name: https://$app_name.fly.dev"
    else
        print_error "$display_name: Deployment failed"
    fi
}

check_app "trading-bot-database" "MongoDB Database"
check_app "trading-bot-ai-service" "Python AI Service"  
check_app "trading-bot-rust-engine" "Rust Core Engine"
check_app "trading-bot-dashboard" "React Dashboard"

echo ""
print_success "🎉 Deployment completed!"
echo ""
echo "🔗 Access your trading bot at: https://trading-bot-dashboard.fly.dev"
echo ""
echo "📝 Post-deployment steps:"
echo "1. Set up your Binance API keys in the Rust engine secrets"
echo "2. Configure your OpenAI API keys for the AI service"  
echo "3. Test all functionality through the dashboard"
echo ""
echo "🔧 Useful commands:"
echo "- Monitor logs: flyctl logs --app trading-bot-rust-engine"
echo "- Scale resources: flyctl scale vm shared-cpu-1x --memory 1024 --app trading-bot-rust-engine"
echo "- Check status: flyctl status --app trading-bot-rust-engine" 