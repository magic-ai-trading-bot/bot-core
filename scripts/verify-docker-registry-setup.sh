#!/bin/bash
# Docker Registry Setup Verification Script
# Verifies that all Docker Registry infrastructure is properly configured

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

# Counters
PASSED=0
FAILED=0
WARNINGS=0

# Function to print colored output
print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

print_success() {
    echo -e "${GREEN}✅ $1${NC}"
    ((PASSED++))
}

print_error() {
    echo -e "${RED}❌ $1${NC}"
    ((FAILED++))
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
    ((WARNINGS++))
}

print_header() {
    echo -e "${CYAN}$1${NC}"
}

# Verification functions
check_file_exists() {
    local file=$1
    local description=$2

    if [ -f "$file" ]; then
        print_success "$description exists: $file"
        return 0
    else
        print_error "$description missing: $file"
        return 1
    fi
}

check_file_executable() {
    local file=$1
    local description=$2

    if [ -x "$file" ]; then
        print_success "$description is executable"
        return 0
    else
        print_error "$description is not executable: $file"
        return 1
    fi
}

check_env_variable() {
    local var=$1
    local description=$2

    if grep -q "^$var=" "$PROJECT_ROOT/.env.example"; then
        print_success "$description configured in .env.example"
        return 0
    else
        print_error "$description missing in .env.example"
        return 1
    fi
}

check_docker_running() {
    if docker info > /dev/null 2>&1; then
        print_success "Docker is running"
        return 0
    else
        print_warning "Docker is not running (optional for verification)"
        return 1
    fi
}

# Main verification
main() {
    echo -e "${CYAN}╔════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║   Docker Registry Setup Verification              ║${NC}"
    echo -e "${CYAN}╚════════════════════════════════════════════════════╝${NC}"
    echo ""

    # Check scripts
    print_header "=== Checking Scripts ==="
    check_file_exists "$PROJECT_ROOT/scripts/docker-registry-setup.sh" "Registry setup script"
    check_file_executable "$PROJECT_ROOT/scripts/docker-registry-setup.sh" "Registry setup script"

    check_file_exists "$PROJECT_ROOT/scripts/build-and-push.sh" "Build and push script"
    check_file_executable "$PROJECT_ROOT/scripts/build-and-push.sh" "Build and push script"

    check_file_exists "$PROJECT_ROOT/scripts/pull-images.sh" "Pull images script"
    check_file_executable "$PROJECT_ROOT/scripts/pull-images.sh" "Pull images script"

    echo ""

    # Check CI/CD workflow
    print_header "=== Checking CI/CD Workflow ==="
    check_file_exists "$PROJECT_ROOT/.github/workflows/docker-build-push.yml" "Docker build and push workflow"

    echo ""

    # Check configuration files
    print_header "=== Checking Configuration ==="
    check_file_exists "$PROJECT_ROOT/.env.example" "Environment template"
    check_env_variable "DOCKER_REGISTRY" "Docker registry URL"
    check_env_variable "DOCKER_USERNAME" "Docker username"
    check_env_variable "DOCKER_PASSWORD" "Docker password"
    check_env_variable "VERSION" "Image version"
    check_env_variable "IMAGE_PULL_POLICY" "Image pull policy"

    check_file_exists "$PROJECT_ROOT/docker-compose.prod.yml" "Production docker-compose"

    # Check if pull_policy is set in docker-compose.prod.yml
    if grep -q "pull_policy:" "$PROJECT_ROOT/docker-compose.prod.yml"; then
        print_success "Pull policy configured in docker-compose.prod.yml"
    else
        print_error "Pull policy missing in docker-compose.prod.yml"
    fi

    echo ""

    # Check documentation
    print_header "=== Checking Documentation ==="
    check_file_exists "$PROJECT_ROOT/docs/DOCKER_REGISTRY_SETUP.md" "Registry setup documentation"
    check_file_exists "$PROJECT_ROOT/docs/plans/251118-docker-registry-implementation-plan.md" "Implementation plan"

    echo ""

    # Check Docker (optional)
    print_header "=== Checking Docker Environment ==="
    check_docker_running

    if command -v trivy &> /dev/null; then
        print_success "Trivy (vulnerability scanner) is installed"
    else
        print_warning "Trivy not installed (optional, for vulnerability scanning)"
    fi

    if command -v cosign &> /dev/null; then
        print_success "Cosign (image signing) is installed"
    else
        print_warning "Cosign not installed (optional, for image signing)"
    fi

    echo ""

    # Check .env file
    print_header "=== Checking Environment Configuration ==="
    if [ -f "$PROJECT_ROOT/.env" ]; then
        print_success ".env file exists"

        # Check if variables are configured (not default values)
        if grep -q "your-username" "$PROJECT_ROOT/.env" 2>/dev/null; then
            print_warning ".env contains placeholder values - please configure real values"
        else
            print_success ".env appears to be configured"
        fi
    else
        print_warning ".env file not found (create from .env.example)"
        print_info "Run: cp .env.example .env && nano .env"
    fi

    echo ""

    # Summary
    print_header "=== Verification Summary ==="
    echo -e "${GREEN}Passed:   $PASSED${NC}"
    echo -e "${YELLOW}Warnings: $WARNINGS${NC}"
    echo -e "${RED}Failed:   $FAILED${NC}"
    echo ""

    if [ $FAILED -eq 0 ]; then
        print_success "All critical checks passed! ✨"
        echo ""
        print_info "Next steps:"
        echo "  1. Configure .env: cp .env.example .env && nano .env"
        echo "  2. Setup registry: ./scripts/docker-registry-setup.sh"
        echo "  3. Build images: ./scripts/build-and-push.sh"
        echo "  4. Pull images: ./scripts/pull-images.sh"
        echo "  5. Deploy: VERSION=latest docker-compose -f docker-compose.prod.yml up -d"
        echo ""
        print_info "Documentation: docs/DOCKER_REGISTRY_SETUP.md"
        return 0
    else
        print_error "Some checks failed. Please review and fix issues."
        return 1
    fi
}

# Run main function
main "$@"
