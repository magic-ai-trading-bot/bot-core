#!/bin/bash
# Docker Image Build and Push Script
# Builds all service images, tags them, and pushes to registry

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Load environment variables
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
else
    echo -e "${YELLOW}⚠️  .env file not found. Using defaults.${NC}"
fi

# Configuration
DOCKER_REGISTRY="${DOCKER_REGISTRY:-ghcr.io/your-username/bot-core}"
VERSION="${VERSION:-latest}"

# Auto-detect GIT SHA if not set
if [ -z "$GIT_SHA" ] || [ "$GIT_SHA" = "auto-detect" ]; then
    if command -v git &> /dev/null && [ -d "$PROJECT_ROOT/.git" ]; then
        GIT_SHA=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD)
        echo -e "${BLUE}ℹ️  Auto-detected GIT_SHA: $GIT_SHA${NC}"
    else
        GIT_SHA="unknown"
    fi
fi

# Services to build
SERVICES=("rust-core-engine" "python-ai-service" "nextjs-ui-dashboard")

# Build options
BUILD_PARALLEL=${BUILD_PARALLEL:-false}
PUSH_IMAGES=${PUSH_IMAGES:-true}
BUILD_CACHE=${BUILD_CACHE:-true}
PLATFORM=${PLATFORM:-linux/amd64}

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

print_step() {
    echo -e "${CYAN}▶️  $1${NC}"
}

# Function to check if Docker is running
check_docker() {
    if ! docker info > /dev/null 2>&1; then
        print_error "Docker is not running. Please start Docker and try again."
        exit 1
    fi
    print_success "Docker is running"
}

# Function to check registry authentication
check_registry_auth() {
    print_info "Checking registry authentication..."

    # Extract registry URL
    REGISTRY_URL=$(echo "$DOCKER_REGISTRY" | sed 's|/.*||')

    # Check if already authenticated
    if grep -q "$REGISTRY_URL" ~/.docker/config.json 2>/dev/null; then
        print_success "Already authenticated to $REGISTRY_URL"
        return 0
    else
        print_warning "Not authenticated to $REGISTRY_URL"
        print_info "Running registry setup..."
        "$SCRIPT_DIR/docker-registry-setup.sh"
        return $?
    fi
}

# Function to build a single service
build_service() {
    local service=$1
    local service_dir="$PROJECT_ROOT/$service"

    print_step "Building $service..."

    # Check if Dockerfile exists
    if [ ! -f "$service_dir/Dockerfile" ]; then
        print_error "Dockerfile not found for $service at $service_dir/Dockerfile"
        return 1
    fi

    # Image names
    local image_latest="${DOCKER_REGISTRY}/${service}:latest"
    local image_version="${DOCKER_REGISTRY}/${service}:${VERSION}"
    local image_sha="${DOCKER_REGISTRY}/${service}:${GIT_SHA}"

    # Build arguments
    local build_args=""
    if [ "$BUILD_CACHE" = "false" ]; then
        build_args="--no-cache"
    fi

    # Build the image
    print_info "Building image: $image_version"

    docker build \
        $build_args \
        --platform "$PLATFORM" \
        --build-arg VERSION="$VERSION" \
        --build-arg GIT_SHA="$GIT_SHA" \
        --build-arg BUILD_DATE="$(date -u +'%Y-%m-%dT%H:%M:%SZ')" \
        -t "$image_latest" \
        -t "$image_version" \
        -t "$image_sha" \
        -f "$service_dir/Dockerfile" \
        "$service_dir"

    if [ $? -eq 0 ]; then
        print_success "Built $service successfully"

        # Display image size
        local size=$(docker images "$image_latest" --format "{{.Size}}" | head -n 1)
        print_info "Image size: $size"

        return 0
    else
        print_error "Failed to build $service"
        return 1
    fi
}

# Function to push a single service
push_service() {
    local service=$1

    print_step "Pushing $service..."

    # Image names
    local image_latest="${DOCKER_REGISTRY}/${service}:latest"
    local image_version="${DOCKER_REGISTRY}/${service}:${VERSION}"
    local image_sha="${DOCKER_REGISTRY}/${service}:${GIT_SHA}"

    # Push all tags
    local push_failed=false

    print_info "Pushing $image_latest"
    docker push "$image_latest" || push_failed=true

    if [ "$VERSION" != "latest" ]; then
        print_info "Pushing $image_version"
        docker push "$image_version" || push_failed=true
    fi

    if [ "$GIT_SHA" != "unknown" ] && [ "$GIT_SHA" != "$VERSION" ]; then
        print_info "Pushing $image_sha"
        docker push "$image_sha" || push_failed=true
    fi

    if [ "$push_failed" = "false" ]; then
        print_success "Pushed $service successfully"
        return 0
    else
        print_error "Failed to push $service"
        return 1
    fi
}

# Function to clean up old local images
cleanup_old_images() {
    print_step "Cleaning up old local images..."

    # Remove dangling images
    local dangling=$(docker images -f "dangling=true" -q)
    if [ -n "$dangling" ]; then
        print_info "Removing dangling images..."
        docker rmi $dangling > /dev/null 2>&1 || true
        print_success "Cleaned up dangling images"
    else
        print_info "No dangling images to clean up"
    fi

    # Remove old tagged images (keep latest, version, and git sha)
    for service in "${SERVICES[@]}"; do
        print_info "Cleaning up old images for $service..."

        # Get all images for this service except the ones we just built
        local old_images=$(docker images "${DOCKER_REGISTRY}/${service}" --format "{{.Repository}}:{{.Tag}}" | \
            grep -v ":latest$" | \
            grep -v ":${VERSION}$" | \
            grep -v ":${GIT_SHA}$" || true)

        if [ -n "$old_images" ]; then
            echo "$old_images" | while read -r image; do
                print_info "Removing old image: $image"
                docker rmi "$image" > /dev/null 2>&1 || true
            done
        fi
    done

    print_success "Cleanup complete"
}

# Function to display summary
display_summary() {
    echo ""
    echo -e "${CYAN}╔════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║            Build & Push Summary                    ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
    echo ""

    print_info "Registry: $DOCKER_REGISTRY"
    print_info "Version: $VERSION"
    print_info "Git SHA: $GIT_SHA"
    print_info "Platform: $PLATFORM"
    echo ""

    print_info "Built Images:"
    for service in "${SERVICES[@]}"; do
        echo -e "  ${GREEN}•${NC} ${DOCKER_REGISTRY}/${service}:${VERSION}"
    done
    echo ""

    if [ "$PUSH_IMAGES" = "true" ]; then
        print_success "All images pushed to registry"
        echo ""
        print_info "Pull images with:"
        print_info "  ./scripts/pull-images.sh"
        echo ""
        print_info "Deploy to production with:"
        print_info "  VERSION=$VERSION docker-compose -f docker-compose.prod.yml up -d"
    else
        print_info "Images built locally (not pushed)"
    fi
}

# Function to scan images for vulnerabilities (optional)
scan_images() {
    if ! command -v trivy &> /dev/null; then
        print_warning "Trivy not installed. Skipping vulnerability scanning."
        print_info "Install Trivy: https://aquasecurity.github.io/trivy/latest/getting-started/installation/"
        return
    fi

    print_step "Scanning images for vulnerabilities..."

    for service in "${SERVICES[@]}"; do
        local image="${DOCKER_REGISTRY}/${service}:${VERSION}"

        print_info "Scanning $service..."
        trivy image --severity HIGH,CRITICAL --exit-code 0 "$image"

        if [ $? -eq 0 ]; then
            print_success "$service scan complete"
        else
            print_warning "$service has vulnerabilities"
        fi
    done
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --no-push)
                PUSH_IMAGES=false
                shift
                ;;
            --no-cache)
                BUILD_CACHE=false
                shift
                ;;
            --parallel)
                BUILD_PARALLEL=true
                shift
                ;;
            --scan)
                SCAN_IMAGES=true
                shift
                ;;
            --platform)
                PLATFORM="$2"
                shift 2
                ;;
            --service)
                SERVICES=("$2")
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --no-push         Build images but don't push to registry"
                echo "  --no-cache        Build without using cache"
                echo "  --parallel        Build images in parallel (experimental)"
                echo "  --scan            Scan images with Trivy after build"
                echo "  --platform ARCH   Build for specific platform (default: linux/amd64)"
                echo "  --service NAME    Build only specific service"
                echo "  --help            Show this help message"
                echo ""
                echo "Examples:"
                echo "  $0                              # Build and push all services"
                echo "  $0 --no-push                    # Build only, don't push"
                echo "  $0 --service rust-core-engine   # Build only Rust service"
                echo "  $0 --scan                       # Build, push, and scan"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
}

# Main build flow
main() {
    parse_args "$@"

    echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║     Docker Build & Push - Bot Core                ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"
    echo ""

    # Pre-flight checks
    check_docker

    if [ "$PUSH_IMAGES" = "true" ]; then
        check_registry_auth
    fi

    echo ""

    # Build all services
    local build_failed=false

    if [ "$BUILD_PARALLEL" = "true" ]; then
        print_info "Building services in parallel..."
        for service in "${SERVICES[@]}"; do
            build_service "$service" &
        done
        wait

        # Check if any builds failed
        for service in "${SERVICES[@]}"; do
            if ! docker images "${DOCKER_REGISTRY}/${service}:${VERSION}" | grep -q "${VERSION}"; then
                build_failed=true
                break
            fi
        done
    else
        for service in "${SERVICES[@]}"; do
            build_service "$service" || build_failed=true
        done
    fi

    if [ "$build_failed" = "true" ]; then
        print_error "One or more builds failed"
        exit 1
    fi

    echo ""

    # Scan images if requested
    if [ "${SCAN_IMAGES:-false}" = "true" ]; then
        scan_images
        echo ""
    fi

    # Push images if requested
    if [ "$PUSH_IMAGES" = "true" ]; then
        local push_failed=false

        for service in "${SERVICES[@]}"; do
            push_service "$service" || push_failed=true
        done

        if [ "$push_failed" = "true" ]; then
            print_error "One or more pushes failed"
            exit 1
        fi

        echo ""

        # Cleanup old images
        cleanup_old_images
    fi

    # Display summary
    display_summary
}

# Run main function
main "$@"
