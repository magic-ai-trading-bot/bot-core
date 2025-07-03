#!/bin/bash

# Bot Core - Optimized Build Script
# Builds services sequentially to avoid memory issues

set -e

echo "🚀 Bot Core - Sequential Build Script"
echo "======================================="

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

print_step() {
    echo -e "${BLUE}[STEP]${NC} $1"
}

# Check Docker daemon
check_docker() {
    print_step "Checking Docker daemon..."
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker daemon is not running. Please start Docker Desktop."
        exit 1
    fi
    print_status "Docker daemon is running"
}

# Check available memory
check_resources() {
    print_step "Checking system resources..."
    
    # Get Docker info
    docker_memory=$(docker system info --format '{{.MemTotal}}' 2>/dev/null || echo "0")
    if [ "$docker_memory" != "0" ]; then
        docker_memory_gb=$((docker_memory / 1024 / 1024 / 1024))
        print_status "Docker allocated memory: ${docker_memory_gb}GB"
        
        if [ $docker_memory_gb -lt 6 ]; then
            print_warning "Docker has less than 6GB RAM. Consider increasing Docker memory allocation."
            print_warning "Go to Docker Desktop Settings > Resources > Advanced > Memory"
        fi
    fi
}

# Clean up existing containers and images
cleanup() {
    print_step "Cleaning up existing containers..."
    
    # Stop and remove existing containers
    docker compose down 2>/dev/null || true
    
    # Remove unused images to free space
    print_status "Removing unused Docker images..."
    docker image prune -f || true
    
    # Remove build cache if needed
    if [ "$1" = "--clean-cache" ]; then
        print_status "Removing Docker build cache..."
        docker builder prune -f || true
    fi
}

# Build service with resource monitoring
build_service() {
    local service_name=$1
    local max_retries=3
    local retry_count=0
    
    print_step "Building $service_name..."
    
    while [ $retry_count -lt $max_retries ]; do
        if docker compose build $service_name; then
            print_status "$service_name built successfully"
            return 0
        else
            retry_count=$((retry_count + 1))
            print_warning "$service_name build failed. Attempt $retry_count/$max_retries"
            
            if [ $retry_count -lt $max_retries ]; then
                print_status "Cleaning up and retrying in 10 seconds..."
                docker image prune -f || true
                sleep 10
            fi
        fi
    done
    
    print_error "Failed to build $service_name after $max_retries attempts"
    return 1
}

# Start service and check health
start_and_check() {
    local service_name=$1
    local health_url=$2
    local max_wait=60
    
    print_step "Starting $service_name..."
    docker compose up -d $service_name
    
    if [ -n "$health_url" ]; then
        print_status "Waiting for $service_name to be healthy..."
        local wait_time=0
        while [ $wait_time -lt $max_wait ]; do
            if curl -f "$health_url" > /dev/null 2>&1; then
                print_status "$service_name is healthy"
                return 0
            fi
            sleep 5
            wait_time=$((wait_time + 5))
            echo -n "."
        done
        echo ""
        print_warning "$service_name health check timeout, but continuing..."
    fi
}

# Main build process
main() {
    echo "Starting optimized build process..."
    
    # Parse arguments
    CLEAN_CACHE=false
    BUILD_ONLY=false
    
    while [[ $# -gt 0 ]]; do
        case $1 in
            --clean-cache)
                CLEAN_CACHE=true
                shift
                ;;
            --build-only)
                BUILD_ONLY=true
                shift
                ;;
            *)
                echo "Unknown option: $1"
                echo "Usage: $0 [--clean-cache] [--build-only]"
                exit 1
                ;;
        esac
    done
    
    # Initial checks
    check_docker
    check_resources
    
    # Cleanup
    if [ "$CLEAN_CACHE" = true ]; then
        cleanup --clean-cache
    else
        cleanup
    fi
    
    # Build services sequentially (lightest to heaviest)
    print_step "Building services in optimal order..."
    
    # Use memory-optimized docker-compose if available
    COMPOSE_FILE="docker-compose.yml"
    if [ -f "docker-compose.memory-optimized.yml" ]; then
        COMPOSE_FILE="docker-compose.memory-optimized.yml"
        print_status "Using memory-optimized compose file"
    fi
    
    # 1. Python AI Service (heaviest - build first when memory is fresh)
    if ! docker compose -f "$COMPOSE_FILE" build python-ai-service; then
        print_error "Failed to build python-ai-service"
        exit 1
    fi
    print_status "python-ai-service built successfully"
    
    # Clear intermediate containers to free memory
    docker container prune -f || true
    
    # 2. Rust Core Engine (medium weight)
    if ! docker compose -f "$COMPOSE_FILE" build rust-core-engine; then
        print_error "Failed to build rust-core-engine"
        exit 1
    fi
    print_status "rust-core-engine built successfully"
    
    # Clear intermediate containers again
    docker container prune -f || true
    
    # 3. Next.js Dashboard (lightest)
    if ! docker compose -f "$COMPOSE_FILE" build nextjs-ui-dashboard; then
        print_error "Failed to build nextjs-ui-dashboard"
        exit 1
    fi
    print_status "nextjs-ui-dashboard built successfully"
    
    print_status "All services built successfully!"
    
    # Start services if not build-only
    if [ "$BUILD_ONLY" = false ]; then
        print_step "Starting services in dependency order..."
        
        # Start Python AI first
        start_and_check "python-ai-service" "http://localhost:8000/health"
        
        # Start Rust Engine (depends on Python AI)
        start_and_check "rust-core-engine" "http://localhost:8080/health"
        
        # Start Dashboard (depends on both)
        start_and_check "nextjs-ui-dashboard" "http://localhost:3000/health"
        
        print_status "All services are running!"
        
        echo ""
        echo "🎉 Build and deployment complete!"
        echo ""
        echo "Service URLs:"
        echo "  📊 Dashboard:    http://localhost:3000"
        echo "  🦀 Rust API:     http://localhost:8080"
        echo "  🐍 Python AI:    http://localhost:8000"
        echo ""
        echo "Check status with: docker compose ps"
        echo "View logs with:    docker compose logs -f"
    else
        print_status "Build-only mode completed. Use 'docker compose up -d' to start services."
    fi
}

# Trap to cleanup on exit
trap 'echo ""; print_status "Build script interrupted"' INT TERM

# Run main function
main "$@" 