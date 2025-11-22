#!/bin/bash

# =============================================================================
# CRYPTO TRADING BOT - MAIN CONTROL SCRIPT
# =============================================================================

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

print_header() {
    echo -e "${PURPLE}========================================${NC}"
    echo -e "${PURPLE}🚀 CRYPTO TRADING BOT CONTROL${NC}"
    echo -e "${PURPLE}========================================${NC}"
    echo ""
}

print_usage() {
    echo -e "${CYAN}Usage: $0 [COMMAND] [OPTIONS]${NC}"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo "  start     - Start all services (production mode)"
    echo "  dev       - Start in development mode with hot reload"
    echo "  stop      - Stop all services"
    echo "  restart   - Restart all services"
    echo "  build     - Build all services"
    echo "  test      - Run test suite"
    echo "  status    - Show service status"
    echo "  logs      - Show logs for all services"
    echo "  clean     - Clean up containers and volumes"
    echo "  verify    - Verify setup and configuration"
    echo "  help      - Show this help message"
    echo ""
    echo -e "${YELLOW}Options:${NC}"
    echo "  --memory-optimized  - Use memory optimized settings"
    echo "  --with-enterprise   - Explicitly enable all enterprise features (Redis, RabbitMQ, Kong, Monitoring)"
    echo "  --service SERVICE   - Target specific service"
    echo "  --coverage          - Run tests with coverage report (test command only)"
    echo "  --all               - Run all test files including comprehensive tests (test command only)"
    echo ""
    echo -e "${GREEN}⭐ DEFAULT SERVICES (Always Started):${NC}"
    echo "  ✅ Core Services: MongoDB, Rust Engine, Python AI, Frontend"
    echo "  ✅ Redis Cache"
    echo "  ✅ RabbitMQ + Celery Worker + Celery Beat + Flower (Async Jobs)"
    echo "  ✅ Kong API Gateway"
    echo "  ✅ Prometheus + Grafana (Monitoring)"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo "  $0 start                      # Start ALL services (full stack)"
    echo "  $0 start --memory-optimized   # Start all services with optimized memory"
    echo "  $0 dev                        # Start in development mode (all features)"
    echo "  $0 test                       # Run simplified test suite (24 tests)"
    echo "  $0 test --coverage            # Run tests with coverage report"
    echo "  $0 test --all                 # Run all test files (138 tests)"
    echo "  $0 logs --service celery-worker  # Show logs for Celery worker"
    echo "  $0 status                     # Check all services status"
}

print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

setup_environment() {
    if [[ "$MEMORY_OPTIMIZED" == "true" ]]; then
        export PYTHON_MEMORY_LIMIT="1.5G"
        export PYTHON_CPU_LIMIT="1.5"
        export RUST_MEMORY_LIMIT="1G"
        export RUST_CPU_LIMIT="1"
        export FRONTEND_MEMORY_LIMIT="512M"
        export FRONTEND_CPU_LIMIT="0.5"
        export NODE_MEMORY="512"
        print_status "Using memory optimized settings"
    fi

    if [[ "$DEV_MODE" == "true" ]]; then
        export DOCKERFILE="Dockerfile.dev"
        export LOG_LEVEL="DEBUG"
        export RUST_LOG="debug"
        export NODE_ENV="development"
        export NODE_MEMORY="768"
        print_status "Development mode enabled"
    fi

    # Copy config if .env doesn't exist
    if [[ ! -f ".env" ]]; then
        if [[ -f "config.env" ]]; then
            cp config.env .env
            print_status "Created .env file from config.env"
        fi
    fi
}

check_prerequisites() {
    if ! command -v docker &> /dev/null; then
        print_error "Docker is not installed or not in PATH"
        exit 1
    fi

    if ! command -v docker-compose &> /dev/null && ! docker compose version &> /dev/null; then
        print_error "Docker Compose is not installed"
        exit 1
    fi
}

start_services() {
    print_status "Starting services..."
    
    # Build profiles string
    PROFILES=""
    if [[ "$DEV_MODE" == "true" ]]; then
        PROFILES="--profile dev"
    else
        PROFILES="--profile prod"
    fi
    
    # Add enterprise features if requested
    if [[ "$WITH_ENTERPRISE" == "true" ]]; then
        PROFILES="$PROFILES --profile redis --profile messaging --profile monitoring --profile api-gateway"
        print_status "Including all enterprise features"
    else
        # Add individual features
        if [[ "$WITH_REDIS" == "true" ]]; then
            PROFILES="$PROFILES --profile redis"
            print_status "Including Redis cache"
        fi
        if [[ "$WITH_RABBITMQ" == "true" ]]; then
            PROFILES="$PROFILES --profile messaging"
            print_status "Including RabbitMQ message queue"
        fi
        if [[ "$WITH_KONG" == "true" ]]; then
            PROFILES="$PROFILES --profile api-gateway"
            print_status "Including Kong API Gateway"
        fi
        if [[ "$WITH_MONITORING" == "true" ]]; then
            PROFILES="$PROFILES --profile monitoring"
            print_status "Including Prometheus & Grafana monitoring"
        fi
    fi
    
    # Start services with selected profiles
    docker compose $PROFILES up -d
    
    if [[ "$DEV_MODE" == "true" ]]; then
        print_success "Development services started"
    else
        print_success "Production services started"
    fi
    
    # Wait for services to be ready
    print_status "Waiting for services to be ready..."
    sleep 5

    # Auto-seed MongoDB with sample data (only on first run)
    if [[ -f "scripts/init-mongodb-seed.sh" ]]; then
        print_status "Checking MongoDB seed data..."
        bash scripts/init-mongodb-seed.sh || print_warning "Seed script failed (may already exist)"
    fi

    show_status
    show_urls
}

stop_services() {
    print_status "Stopping services..."
    docker compose down --remove-orphans
    print_success "All services stopped"
}

restart_services() {
    print_status "Restarting services..."
    stop_services
    start_services
}

build_services() {
    print_status "Building services..."
    if [[ -n "$SERVICE" ]]; then
        if [[ "$DEV_MODE" == "true" ]]; then
            docker compose --profile dev build $SERVICE
        else
            docker compose --profile prod build $SERVICE
        fi
        print_success "Service $SERVICE built successfully"
    else
        if [[ "$DEV_MODE" == "true" ]]; then
            docker compose --profile dev build
        else
            docker compose --profile prod build
        fi
        print_success "All services built successfully"
    fi
}

show_status() {
    print_status "Service status:"
    docker compose ps
    echo ""
    
    print_status "Resource usage:"
    docker stats --no-stream --format "table {{.Name}}\t{{.MemUsage}}\t{{.MemPerc}}\t{{.CPUPerc}}" || true
}

show_logs() {
    if [[ -n "$SERVICE" ]]; then
        print_status "Showing logs for $SERVICE..."
        docker compose logs -f $SERVICE
    else
        print_status "Showing logs for all services..."
        docker compose logs -f
    fi
}

clean_up() {
    print_warning "This will remove all containers, images, and volumes. Are you sure? (y/N)"
    read -r response
    if [[ "$response" =~ ^[Yy]$ ]]; then
        print_status "Cleaning up..."
        docker compose down --volumes --remove-orphans
        docker system prune -af
        print_success "Cleanup completed"
    else
        print_status "Cleanup cancelled"
    fi
}

show_urls() {
    echo ""
    print_success "🎯 Service URLs:"
    echo -e "${GREEN}Core Services:${NC}"
    echo -e "  📊 Frontend Dashboard: ${CYAN}http://localhost:3000${NC}"
    echo -e "  🦀 Rust Core Engine: ${CYAN}http://localhost:8080/api/health${NC}"
    echo -e "  🐍 Python AI Service: ${CYAN}http://localhost:8000/health${NC}"

    # Check if enterprise features are running
    if docker ps --format '{{.Names}}' | grep -q "rabbitmq"; then
        echo -e "\n${GREEN}Enterprise Features:${NC}"
        if docker ps --format '{{.Names}}' | grep -q "rabbitmq"; then
            echo -e "  🐰 RabbitMQ Management: ${CYAN}http://localhost:15672${NC} (admin/admin)"
        fi
        if docker ps --format '{{.Names}}' | grep -q "kong"; then
            echo -e "  👑 Kong Admin API: ${CYAN}http://localhost:8001${NC}"
            echo -e "  🔀 Kong Proxy: ${CYAN}http://localhost:8100${NC}"
        fi
        if docker ps --format '{{.Names}}' | grep -q "prometheus"; then
            echo -e "  📈 Prometheus: ${CYAN}http://localhost:9090${NC}"
        fi
        if docker ps --format '{{.Names}}' | grep -q "grafana"; then
            echo -e "  📊 Grafana: ${CYAN}http://localhost:3001${NC} (admin/admin)"
        fi
    fi
    echo ""
}

run_tests() {
    print_status "Running test suite..."

    # Check if celery-worker is running
    if ! docker ps --format '{{.Names}}' | grep -q "celery-worker"; then
        print_error "celery-worker container is not running"
        print_status "Start services first with: ./scripts/bot.sh start --with-rabbitmq"
        exit 1
    fi

    if [[ "$RUN_ALL_TESTS" == "true" ]]; then
        print_status "Running ALL test files (138 tests)..."
        if [[ "$WITH_COVERAGE" == "true" ]]; then
            docker exec celery-worker pytest tests/ -v \
                --cov=tasks --cov=utils --cov=celery_app \
                --cov-report=html --cov-report=term-missing
        else
            docker exec celery-worker pytest tests/ -v
        fi
    else
        print_status "Running simplified test suite (24 tests)..."
        if [[ "$WITH_COVERAGE" == "true" ]]; then
            docker exec celery-worker pytest tests/test_async_tasks_simple.py -v \
                --cov=tasks --cov=utils --cov=celery_app \
                --cov-report=html --cov-report=term-missing
            print_success "Coverage report generated at: python-ai-service/htmlcov/index.html"
        else
            docker exec celery-worker pytest tests/test_async_tasks_simple.py -v
        fi
    fi

    if [[ $? -eq 0 ]]; then
        print_success "All tests passed! ✅"
    else
        print_error "Some tests failed! ❌"
        exit 1
    fi
}

# Parse arguments
COMMAND=""
MEMORY_OPTIMIZED="false"
DEV_MODE="false"
SERVICE=""
WITH_ENTERPRISE="false"
WITH_REDIS="true"           # ⭐ Default: true (always start Redis)
WITH_RABBITMQ="true"        # ⭐ Default: true (always start async jobs)
WITH_KONG="true"            # ⭐ Default: true (always start API Gateway)
WITH_MONITORING="true"      # ⭐ Default: true (always start monitoring)
WITH_COVERAGE="false"
RUN_ALL_TESTS="false"

while [[ $# -gt 0 ]]; do
    case $1 in
        start|dev|stop|restart|build|test|status|logs|clean|verify|help)
            COMMAND="$1"
            if [[ "$1" == "dev" ]]; then
                DEV_MODE="true"
            fi
            shift
            ;;
        --memory-optimized)
            MEMORY_OPTIMIZED="true"
            shift
            ;;
        --with-enterprise)
            WITH_ENTERPRISE="true"
            shift
            ;;
        --with-redis)
            WITH_REDIS="true"
            shift
            ;;
        --with-rabbitmq)
            WITH_RABBITMQ="true"
            shift
            ;;
        --with-kong)
            WITH_KONG="true"
            shift
            ;;
        --with-monitoring)
            WITH_MONITORING="true"
            shift
            ;;
        --coverage)
            WITH_COVERAGE="true"
            shift
            ;;
        --all)
            RUN_ALL_TESTS="true"
            shift
            ;;
        --service)
            SERVICE="$2"
            shift 2
            ;;
        *)
            print_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Main execution
print_header

if [[ -z "$COMMAND" ]]; then
    print_usage
    exit 1
fi

check_prerequisites
setup_environment

case $COMMAND in
    start)
        start_services
        ;;
    dev)
        start_services
        ;;
    stop)
        stop_services
        ;;
    restart)
        restart_services
        ;;
    build)
        build_services
        ;;
    test)
        run_tests
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    clean)
        clean_up
        ;;
    verify)
        if [[ -f "./scripts/verify-setup.sh" ]]; then
            ./scripts/verify-setup.sh
        else
            print_error "Verify script not found"
        fi
        ;;
    help)
        print_usage
        ;;
    *)
        print_error "Unknown command: $COMMAND"
        print_usage
        exit 1
        ;;
esac 