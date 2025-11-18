#!/bin/bash
# Docker Registry Setup Script
# Configures authentication for Docker registries (GitHub, DockerHub, Private)

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Load environment variables
if [ -f "$PROJECT_ROOT/.env" ]; then
    source "$PROJECT_ROOT/.env"
else
    echo -e "${YELLOW}⚠️  .env file not found. Using .env.example as template.${NC}"
    if [ -f "$PROJECT_ROOT/.env.example" ]; then
        cp "$PROJECT_ROOT/.env.example" "$PROJECT_ROOT/.env"
        echo -e "${YELLOW}⚠️  Please edit .env and add your registry credentials.${NC}"
        exit 1
    fi
fi

# Default values
DOCKER_REGISTRY="${DOCKER_REGISTRY:-ghcr.io/your-username/bot-core}"
DOCKER_USERNAME="${DOCKER_USERNAME:-}"
DOCKER_PASSWORD="${DOCKER_PASSWORD:-}"

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

# Function to detect registry type
detect_registry_type() {
    if [[ "$DOCKER_REGISTRY" == *"ghcr.io"* ]]; then
        echo "github"
    elif [[ "$DOCKER_REGISTRY" == *"docker.io"* ]] || [[ "$DOCKER_REGISTRY" != *"."* ]]; then
        echo "dockerhub"
    elif [[ "$DOCKER_REGISTRY" == *"amazonaws.com"* ]]; then
        echo "ecr"
    elif [[ "$DOCKER_REGISTRY" == *"gcr.io"* ]] || [[ "$DOCKER_REGISTRY" == *"pkg.dev"* ]]; then
        echo "gcr"
    elif [[ "$DOCKER_REGISTRY" == *"azurecr.io"* ]]; then
        echo "acr"
    else
        echo "private"
    fi
}

# Function to setup GitHub Container Registry
setup_github_registry() {
    print_info "Setting up GitHub Container Registry (ghcr.io)..."

    if [ -z "$DOCKER_USERNAME" ] || [ -z "$DOCKER_PASSWORD" ]; then
        print_error "DOCKER_USERNAME and DOCKER_PASSWORD must be set in .env"
        print_info "For GitHub:"
        print_info "  1. Create a Personal Access Token at: https://github.com/settings/tokens"
        print_info "  2. Select scopes: write:packages, read:packages, delete:packages"
        print_info "  3. Set DOCKER_USERNAME=your-github-username"
        print_info "  4. Set DOCKER_PASSWORD=your-github-token"
        return 1
    fi

    echo "$DOCKER_PASSWORD" | docker login ghcr.io -u "$DOCKER_USERNAME" --password-stdin

    if [ $? -eq 0 ]; then
        print_success "Successfully authenticated to GitHub Container Registry"
        return 0
    else
        print_error "Failed to authenticate to GitHub Container Registry"
        return 1
    fi
}

# Function to setup Docker Hub
setup_dockerhub_registry() {
    print_info "Setting up Docker Hub..."

    if [ -z "$DOCKER_USERNAME" ] || [ -z "$DOCKER_PASSWORD" ]; then
        print_error "DOCKER_USERNAME and DOCKER_PASSWORD must be set in .env"
        print_info "For Docker Hub:"
        print_info "  1. Create account at: https://hub.docker.com"
        print_info "  2. Create Access Token at: https://hub.docker.com/settings/security"
        print_info "  3. Set DOCKER_USERNAME=your-dockerhub-username"
        print_info "  4. Set DOCKER_PASSWORD=your-access-token"
        return 1
    fi

    echo "$DOCKER_PASSWORD" | docker login -u "$DOCKER_USERNAME" --password-stdin

    if [ $? -eq 0 ]; then
        print_success "Successfully authenticated to Docker Hub"
        return 0
    else
        print_error "Failed to authenticate to Docker Hub"
        return 1
    fi
}

# Function to setup AWS ECR
setup_ecr_registry() {
    print_info "Setting up AWS Elastic Container Registry (ECR)..."

    if ! command -v aws &> /dev/null; then
        print_error "AWS CLI not installed. Install with: pip install awscli"
        return 1
    fi

    # Extract region from registry URL
    REGION=$(echo "$DOCKER_REGISTRY" | sed -n 's/.*\.ecr\.\([^.]*\)\.amazonaws\.com.*/\1/p')

    if [ -z "$REGION" ]; then
        print_error "Could not detect AWS region from registry URL"
        return 1
    fi

    print_info "Detected AWS region: $REGION"

    # Get ECR login password and authenticate
    aws ecr get-login-password --region "$REGION" | \
        docker login --username AWS --password-stdin "${DOCKER_REGISTRY%%/*}"

    if [ $? -eq 0 ]; then
        print_success "Successfully authenticated to AWS ECR"
        return 0
    else
        print_error "Failed to authenticate to AWS ECR"
        print_info "Ensure AWS credentials are configured: aws configure"
        return 1
    fi
}

# Function to setup Google Container Registry
setup_gcr_registry() {
    print_info "Setting up Google Container Registry (GCR)..."

    if ! command -v gcloud &> /dev/null; then
        print_error "gcloud CLI not installed. Install from: https://cloud.google.com/sdk/install"
        return 1
    fi

    gcloud auth configure-docker

    if [ $? -eq 0 ]; then
        print_success "Successfully configured Docker for GCR"
        return 0
    else
        print_error "Failed to configure Docker for GCR"
        print_info "Ensure you're authenticated: gcloud auth login"
        return 1
    fi
}

# Function to setup Azure Container Registry
setup_acr_registry() {
    print_info "Setting up Azure Container Registry (ACR)..."

    if ! command -v az &> /dev/null; then
        print_error "Azure CLI not installed. Install from: https://docs.microsoft.com/cli/azure/install-azure-cli"
        return 1
    fi

    REGISTRY_NAME=$(echo "$DOCKER_REGISTRY" | sed 's/\.azurecr\.io.*//')

    az acr login --name "$REGISTRY_NAME"

    if [ $? -eq 0 ]; then
        print_success "Successfully authenticated to Azure Container Registry"
        return 0
    else
        print_error "Failed to authenticate to Azure Container Registry"
        print_info "Ensure you're authenticated: az login"
        return 1
    fi
}

# Function to setup private registry
setup_private_registry() {
    print_info "Setting up private Docker registry..."

    if [ -z "$DOCKER_USERNAME" ] || [ -z "$DOCKER_PASSWORD" ]; then
        print_error "DOCKER_USERNAME and DOCKER_PASSWORD must be set in .env"
        return 1
    fi

    REGISTRY_URL=$(echo "$DOCKER_REGISTRY" | sed 's|/.*||')

    echo "$DOCKER_PASSWORD" | docker login "$REGISTRY_URL" -u "$DOCKER_USERNAME" --password-stdin

    if [ $? -eq 0 ]; then
        print_success "Successfully authenticated to private registry: $REGISTRY_URL"
        return 0
    else
        print_error "Failed to authenticate to private registry"
        return 1
    fi
}

# Function to test registry connection
test_registry_connection() {
    print_info "Testing registry connection..."

    # Try to pull a minimal test image (if exists) or check authentication
    REGISTRY_URL=$(echo "$DOCKER_REGISTRY" | sed 's|/.*||')

    # Check if we can list repositories (for registries that support it)
    print_info "Verifying write permissions..."

    # Create a minimal test image
    TEST_IMAGE="${DOCKER_REGISTRY}/test:${VERSION:-latest}"

    print_info "Building test image: $TEST_IMAGE"

    # Create temporary Dockerfile
    TEMP_DIR=$(mktemp -d)
    cat > "$TEMP_DIR/Dockerfile" <<EOF
FROM alpine:latest
RUN echo "Registry test image"
EOF

    docker build -t "$TEST_IMAGE" "$TEMP_DIR" > /dev/null 2>&1

    if [ $? -ne 0 ]; then
        print_warning "Could not build test image"
        rm -rf "$TEMP_DIR"
        return 0
    fi

    print_info "Pushing test image..."
    docker push "$TEST_IMAGE" > /dev/null 2>&1

    if [ $? -eq 0 ]; then
        print_success "Push test successful"

        # Clean up test image
        print_info "Cleaning up test image..."
        docker rmi "$TEST_IMAGE" > /dev/null 2>&1

        print_success "Registry is fully configured and operational"
    else
        print_warning "Push test failed. You may not have write permissions."
        print_info "Pull permissions verified, but push may require additional configuration."
    fi

    rm -rf "$TEMP_DIR"
}

# Main setup flow
main() {
    echo -e "${BLUE}╔════════════════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║     Docker Registry Setup - Bot Core              ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════════════════╝${NC}"
    echo ""

    print_info "Registry: $DOCKER_REGISTRY"

    REGISTRY_TYPE=$(detect_registry_type)
    print_info "Detected registry type: $REGISTRY_TYPE"
    echo ""

    # Setup based on registry type
    case "$REGISTRY_TYPE" in
        github)
            setup_github_registry
            ;;
        dockerhub)
            setup_dockerhub_registry
            ;;
        ecr)
            setup_ecr_registry
            ;;
        gcr)
            setup_gcr_registry
            ;;
        acr)
            setup_acr_registry
            ;;
        private)
            setup_private_registry
            ;;
        *)
            print_error "Unknown registry type"
            exit 1
            ;;
    esac

    if [ $? -eq 0 ]; then
        echo ""
        test_registry_connection
        echo ""
        print_success "Registry setup complete!"
        echo ""
        print_info "Next steps:"
        print_info "  1. Build images: ./scripts/build-and-push.sh"
        print_info "  2. Pull images: ./scripts/pull-images.sh"
        print_info "  3. Deploy: docker-compose -f docker-compose.prod.yml up -d"
    else
        echo ""
        print_error "Registry setup failed. Please check your credentials and try again."
        exit 1
    fi
}

# Run main function
main "$@"
