#!/bin/bash

# Bot Core Monorepo - Development Setup Script
# This script sets up the development environment with hot reload

set -e

echo "🚀 Setting up Bot Core Development Environment..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if running from project root
if [ ! -f "docker-compose.yml" ]; then
    print_error "Please run this script from the project root directory"
    exit 1
fi

print_status "Creating development directories..."

# Create necessary directories
mkdir -p rust-core-engine/data rust-core-engine/logs
mkdir -p python-ai-service/models python-ai-service/logs python-ai-service/data
mkdir -p nextjs-ui-dashboard/logs
mkdir -p scripts

print_status "Copying configuration files..."

# Copy configuration files if they don't exist
if [ ! -f rust-core-engine/config.toml ]; then
    cp rust-core-engine/config.example.toml rust-core-engine/config.toml
    print_status "Created rust-core-engine/config.toml"
fi

# Create development environment file
if [ ! -f .env.dev ]; then
    cat > .env.dev << 'EOF'
# Development Environment Variables
NODE_ENV=development
RUST_LOG=debug
LOG_LEVEL=DEBUG
BINANCE_TESTNET=true
TRADING_ENABLED=false
ENABLE_HOT_RELOAD=true
CHOKIDAR_USEPOLLING=true
WATCHPACK_POLLING=true
CARGO_INCREMENTAL=1
RUST_BACKTRACE=1
POSTGRES_PASSWORD=devpassword
REDIS_PASSWORD=devpassword
GRAFANA_PASSWORD=admin
REACT_APP_API_URL=http://localhost:8080
REACT_APP_AI_API_URL=http://localhost:8000
VITE_API_URL=http://localhost:8080
VITE_AI_API_URL=http://localhost:8000
PYTHONUNBUFFERED=1
PYTHONDONTWRITEBYTECODE=1
DATABASE_URL=sqlite:./data/trading_dev.db
EOF
    print_status "Created .env.dev file"
fi

# Create main .env file if it doesn't exist
if [ ! -f .env ]; then
    print_warning ".env file not found. Creating from .env.dev..."
    cp .env.dev .env
    print_status "Created .env file (copy from .env.dev)"
fi

print_status "Setting up Git hooks..."

# Create git hooks directory if it doesn't exist
mkdir -p .git/hooks

# Create pre-commit hook for linting
cat > .git/hooks/pre-commit << 'EOF'
#!/bin/bash
echo "Running pre-commit linting..."

# Run Rust linting
if [ -d "rust-core-engine" ]; then
    cd rust-core-engine
    cargo clippy -- -D warnings || exit 1
    cd ..
fi

# Run Python linting
if [ -d "python-ai-service" ]; then
    cd python-ai-service
    python -m flake8 . --count --select=E9,F63,F7,F82 --show-source --statistics || exit 1
    cd ..
fi

# Run Frontend linting
if [ -d "nextjs-ui-dashboard" ]; then
    cd nextjs-ui-dashboard
    npm run lint || exit 1
    cd ..
fi

echo "Pre-commit checks passed!"
EOF

chmod +x .git/hooks/pre-commit

print_status "Checking dependencies..."

# Check Docker
if ! command -v docker &> /dev/null; then
    print_error "Docker is not installed. Please install Docker first."
    exit 1
fi

# Check Docker Compose
if ! command -v docker-compose &> /dev/null; then
    print_error "Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

print_status "Dependencies check passed!"

print_status "Building development images..."

# Build development images
docker-compose -f docker-compose.yml -f docker-compose.dev.yml build --no-cache

print_status "Development environment setup complete!"

echo ""
echo -e "${BLUE}🎉 Development Environment Ready!${NC}"
echo ""
echo "Available commands:"
echo "  make dev          - Start all services with hot reload"
echo "  make dev-detach   - Start all services in background"
echo "  make dev-logs     - View development logs"
echo "  make dev-stop     - Stop development services"
echo "  make dev-rebuild  - Rebuild and restart services"
echo ""
echo "Individual services:"
echo "  make dev-rust     - Start only Rust service"
echo "  make dev-python   - Start only Python service"
echo "  make dev-frontend - Start only Frontend service"
echo ""
echo "Service URLs:"
echo "  Frontend:    http://localhost:3000"
echo "  Rust API:    http://localhost:8080"
echo "  Python API:  http://localhost:8000"
echo ""
echo -e "${GREEN}Happy coding! 🚀${NC}" 