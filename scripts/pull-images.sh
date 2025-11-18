#!/bin/bash
# Docker Image Pull Script
# Pulls pre-built images from registry for production deployment

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

# Services to pull
SERVICES=("rust-core-engine" "python-ai-service" "nextjs-ui-dashboard")

# Pull options
VERIFY_SIGNATURES=${VERIFY_SIGNATURES:-false}
LIST_VERSIONS=${LIST_VERSIONS:-false}

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

# Function to list available versions for a service
list_service_versions() {
    local service=$1

    print_step "Available versions for $service:"

    # Try to list tags using different methods based on registry
    if [[ "$DOCKER_REGISTRY" == *"ghcr.io"* ]]; then
        list_github_tags "$service"
    elif [[ "$DOCKER_REGISTRY" == *"docker.io"* ]] || [[ "$DOCKER_REGISTRY" != *"."* ]]; then
        list_dockerhub_tags "$service"
    else
        print_info "Listing tags not supported for this registry type"
        print_info "Available locally:"
        docker images "${DOCKER_REGISTRY}/${service}" --format "  • {{.Tag}}" | head -n 10
    fi
}

# Function to list GitHub Container Registry tags
list_github_tags() {
    local service=$1

    # Extract owner and repo from registry URL
    local owner=$(echo "$DOCKER_REGISTRY" | cut -d'/' -f2)
    local repo=$(echo "$DOCKER_REGISTRY" | cut -d'/' -f3)

    print_info "Fetching tags from GitHub..."

    # Use GitHub API to list tags
    if command -v curl &> /dev/null && command -v jq &> /dev/null; then
        local url="https://api.github.com/users/${owner}/packages/container/${repo}%2F${service}/versions"

        curl -s "$url" | jq -r '.[].metadata.container.tags[]' | head -n 10 | while read -r tag; do
            echo -e "  ${GREEN}•${NC} $tag"
        done
    else
        print_warning "curl or jq not installed. Cannot fetch remote tags."
        print_info "Install with: brew install jq (macOS) or apt-get install jq (Linux)"
    fi
}

# Function to list Docker Hub tags
list_dockerhub_tags() {
    local service=$1

    # Extract repository name
    local repo=$(echo "$DOCKER_REGISTRY" | sed 's|docker.io/||' | sed 's|/|%2F|g')

    print_info "Fetching tags from Docker Hub..."

    if command -v curl &> /dev/null && command -v jq &> /dev/null; then
        local url="https://hub.docker.com/v2/repositories/${repo}/${service}/tags?page_size=10"

        curl -s "$url" | jq -r '.results[].name' | while read -r tag; do
            echo -e "  ${GREEN}•${NC} $tag"
        done
    else
        print_warning "curl or jq not installed. Cannot fetch remote tags."
    fi
}

# Function to pull a single service
pull_service() {
    local service=$1

    print_step "Pulling $service..."

    # Image name
    local image="${DOCKER_REGISTRY}/${service}:${VERSION}"

    print_info "Pulling image: $image"

    docker pull "$image"

    if [ $? -eq 0 ]; then
        print_success "Pulled $service successfully"

        # Display image details
        local size=$(docker images "$image" --format "{{.Size}}" | head -n 1)
        local created=$(docker images "$image" --format "{{.CreatedSince}}" | head -n 1)

        print_info "Image size: $size"
        print_info "Created: $created"

        return 0
    else
        print_error "Failed to pull $service"
        return 1
    fi
}

# Function to verify image signatures (if Docker Content Trust is enabled)
verify_image_signature() {
    local service=$1

    if [ "$VERIFY_SIGNATURES" != "true" ]; then
        return 0
    fi

    print_step "Verifying signature for $service..."

    local image="${DOCKER_REGISTRY}/${service}:${VERSION}"

    # Enable Docker Content Trust
    export DOCKER_CONTENT_TRUST=1

    # Try to pull with signature verification
    docker pull "$image" > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        print_success "Signature verified for $service"
        return 0
    else
        print_warning "Signature verification failed for $service"
        print_info "Image may not be signed or Content Trust is not configured"
        return 1
    fi

    unset DOCKER_CONTENT_TRUST
}

# Function to inspect pulled images
inspect_images() {
    print_step "Inspecting pulled images..."
    echo ""

    for service in "${SERVICES[@]}"; do
        local image="${DOCKER_REGISTRY}/${service}:${VERSION}"

        if docker images "$image" | grep -q "${VERSION}"; then
            print_info "$service:"

            # Get image metadata
            local sha=$(docker images "$image" --format "{{.ID}}" | head -n 1)
            local size=$(docker images "$image" --format "{{.Size}}" | head -n 1)
            local created=$(docker images "$image" --format "{{.CreatedSince}}" | head -n 1)

            echo "  ID: $sha"
            echo "  Size: $size"
            echo "  Created: $created"

            # Get build args if available
            local build_version=$(docker inspect "$image" --format '{{index .Config.Labels "version"}}' 2>/dev/null || echo "N/A")
            local build_sha=$(docker inspect "$image" --format '{{index .Config.Labels "git-sha"}}' 2>/dev/null || echo "N/A")

            if [ "$build_version" != "N/A" ]; then
                echo "  Build Version: $build_version"
            fi

            if [ "$build_sha" != "N/A" ]; then
                echo "  Build SHA: $build_sha"
            fi

            echo ""
        fi
    done
}

# Function to clean up old local images
cleanup_old_versions() {
    print_step "Cleaning up old versions..."

    for service in "${SERVICES[@]}"; do
        # Get all images for this service except the current version
        local old_images=$(docker images "${DOCKER_REGISTRY}/${service}" --format "{{.Repository}}:{{.Tag}}" | \
            grep -v ":${VERSION}$" || true)

        if [ -n "$old_images" ]; then
            print_info "Removing old versions of $service..."

            echo "$old_images" | while read -r image; do
                print_info "  Removing: $image"
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
    echo -e "${CYAN}║              Pull Summary                          ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
    echo ""

    print_info "Registry: $DOCKER_REGISTRY"
    print_info "Version: $VERSION"
    echo ""

    print_info "Pulled Images:"
    for service in "${SERVICES[@]}"; do
        local image="${DOCKER_REGISTRY}/${service}:${VERSION}"
        if docker images "$image" | grep -q "${VERSION}"; then
            echo -e "  ${GREEN}✓${NC} $image"
        else
            echo -e "  ${RED}✗${NC} $image"
        fi
    done

    echo ""
    print_success "All images pulled successfully"
    echo ""
    print_info "Deploy to production with:"
    print_info "  VERSION=$VERSION docker-compose -f docker-compose.prod.yml up -d"
    echo ""
    print_info "Or use the deployment script:"
    print_info "  ./scripts/deploy.sh"
}

# Parse command line arguments
parse_args() {
    while [[ $# -gt 0 ]]; do
        case $1 in
            --list)
                LIST_VERSIONS=true
                shift
                ;;
            --verify)
                VERIFY_SIGNATURES=true
                shift
                ;;
            --cleanup)
                CLEANUP_OLD=true
                shift
                ;;
            --service)
                SERVICES=("$2")
                shift 2
                ;;
            --version)
                VERSION="$2"
                shift 2
                ;;
            --help)
                echo "Usage: $0 [OPTIONS]"
                echo ""
                echo "Options:"
                echo "  --list            List available versions for each service"
                echo "  --verify          Verify image signatures (requires Docker Content Trust)"
                echo "  --cleanup         Remove old versions after pulling"
                echo "  --service NAME    Pull only specific service"
                echo "  --version TAG     Pull specific version (default: latest)"
                echo "  --help            Show this help message"
                echo ""
                echo "Examples:"
                echo "  $0                              # Pull latest versions of all services"
                echo "  $0 --version v1.0.0             # Pull specific version"
                echo "  $0 --service rust-core-engine   # Pull only Rust service"
                echo "  $0 --list                       # List available versions"
                exit 0
                ;;
            *)
                print_error "Unknown option: $1"
                exit 1
                ;;
        esac
    done
}

# Main pull flow
main() {
    parse_args "$@"

    echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║       Docker Image Pull - Bot Core                ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"
    echo ""

    # Pre-flight checks
    check_docker
    check_registry_auth

    echo ""

    # List versions if requested
    if [ "$LIST_VERSIONS" = "true" ]; then
        for service in "${SERVICES[@]}"; do
            list_service_versions "$service"
            echo ""
        done
        exit 0
    fi

    # Pull all services
    local pull_failed=false

    for service in "${SERVICES[@]}"; do
        pull_service "$service" || pull_failed=true

        # Verify signature if requested
        if [ "$VERIFY_SIGNATURES" = "true" ]; then
            verify_image_signature "$service"
        fi

        echo ""
    done

    if [ "$pull_failed" = "true" ]; then
        print_error "One or more pulls failed"
        exit 1
    fi

    # Inspect images
    inspect_images

    # Cleanup old versions if requested
    if [ "${CLEANUP_OLD:-false}" = "true" ]; then
        cleanup_old_versions
        echo ""
    fi

    # Display summary
    display_summary
}

# Run main function
main "$@"
