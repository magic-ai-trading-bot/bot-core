#!/bin/bash

# =============================================================================
# Bot Core - Viettel VPS Deployment Script
# =============================================================================
# This script automates the deployment of Bot Core to Viettel VPS
# Version: 1.0.0
# =============================================================================

set -e  # Exit on error

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
VPS_IP=""
VPS_USER="botadmin"
VPS_SSH_KEY=""
DEPLOY_DIR="/home/botadmin/projects/bot-core"

# =============================================================================
# Helper Functions
# =============================================================================

print_header() {
    echo -e "\n${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}\n"
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

print_info() {
    echo -e "${BLUE}ℹ️  $1${NC}"
}

check_prerequisites() {
    print_header "Checking Prerequisites"

    # Check if VPS_IP is set
    if [ -z "$VPS_IP" ]; then
        print_error "VPS_IP is not set. Please edit this script and set VPS_IP variable."
        exit 1
    fi

    # Check SSH connection
    if ! ssh -o ConnectTimeout=5 $VPS_USER@$VPS_IP "echo 'SSH connection successful'" &>/dev/null; then
        print_error "Cannot connect to VPS. Please check VPS_IP, VPS_USER, and SSH configuration."
        exit 1
    fi

    print_success "SSH connection successful"

    # Check if docker is installed on VPS
    if ! ssh $VPS_USER@$VPS_IP "command -v docker" &>/dev/null; then
        print_warning "Docker not found on VPS. Will need to install it."
        INSTALL_DOCKER=true
    else
        print_success "Docker is installed on VPS"
        INSTALL_DOCKER=false
    fi
}

# =============================================================================
# Main Deployment Functions
# =============================================================================

sync_code() {
    print_header "Syncing Code to VPS"

    # Create directory on VPS
    ssh $VPS_USER@$VPS_IP "mkdir -p $DEPLOY_DIR"

    # Sync files (excluding node_modules, target, etc.)
    rsync -avz --progress \
        --exclude 'node_modules' \
        --exclude 'target' \
        --exclude '.git' \
        --exclude '__pycache__' \
        --exclude '*.pyc' \
        --exclude '.DS_Store' \
        --exclude '.env' \
        ./ $VPS_USER@$VPS_IP:$DEPLOY_DIR/

    print_success "Code synced successfully"
}

setup_environment() {
    print_header "Setting Up Environment"

    # Check if .env exists on VPS
    if ssh $VPS_USER@$VPS_IP "[ -f $DEPLOY_DIR/.env ]"; then
        print_warning ".env file already exists on VPS"
        read -p "Do you want to overwrite it? (y/N): " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            scp .env.example $VPS_USER@$VPS_IP:$DEPLOY_DIR/.env
            print_info "Please SSH to VPS and edit .env file with production values"
        fi
    else
        scp .env.example $VPS_USER@$VPS_IP:$DEPLOY_DIR/.env
        print_warning ".env.example copied to VPS as .env"
        print_info "Please SSH to VPS and edit .env file with production values"
    fi
}

build_services() {
    print_header "Building Docker Images on VPS"

    ssh $VPS_USER@$VPS_IP "cd $DEPLOY_DIR && docker compose build --no-cache"

    print_success "Docker images built successfully"
}

deploy_services() {
    print_header "Deploying Services"

    # Stop existing services
    print_info "Stopping existing services..."
    ssh $VPS_USER@$VPS_IP "cd $DEPLOY_DIR && docker compose down || true"

    # Start services
    print_info "Starting services..."
    ssh $VPS_USER@$VPS_IP "cd $DEPLOY_DIR && docker compose up -d"

    print_success "Services deployed successfully"
}

verify_deployment() {
    print_header "Verifying Deployment"

    sleep 10  # Wait for services to start

    # Check if containers are running
    print_info "Checking running containers..."
    ssh $VPS_USER@$VPS_IP "cd $DEPLOY_DIR && docker compose ps"

    # Test health endpoints
    print_info "Testing health endpoints..."

    # Rust Core Engine
    if ssh $VPS_USER@$VPS_IP "curl -s http://localhost:8080/api/health" &>/dev/null; then
        print_success "Rust Core Engine is healthy"
    else
        print_error "Rust Core Engine health check failed"
    fi

    # Python AI Service
    if ssh $VPS_USER@$VPS_IP "curl -s http://localhost:8000/health" &>/dev/null; then
        print_success "Python AI Service is healthy"
    else
        print_error "Python AI Service health check failed"
    fi

    # Frontend Dashboard
    if ssh $VPS_USER@$VPS_IP "curl -s http://localhost:3000" &>/dev/null; then
        print_success "Frontend Dashboard is accessible"
    else
        print_error "Frontend Dashboard health check failed"
    fi
}

show_logs() {
    print_header "Recent Logs"

    print_info "Showing last 20 lines of logs from each service..."
    ssh $VPS_USER@$VPS_IP "cd $DEPLOY_DIR && docker compose logs --tail=20"
}

# =============================================================================
# Interactive Menu
# =============================================================================

show_menu() {
    echo -e "\n${BLUE}╔════════════════════════════════════════╗${NC}"
    echo -e "${BLUE}║   Bot Core - Viettel VPS Deployment   ║${NC}"
    echo -e "${BLUE}╚════════════════════════════════════════╝${NC}\n"

    echo "1) Full Deployment (sync + build + deploy)"
    echo "2) Sync Code Only"
    echo "3) Build Images Only"
    echo "4) Deploy Services Only"
    echo "5) Verify Deployment"
    echo "6) Show Logs"
    echo "7) Restart Services"
    echo "8) Stop Services"
    echo "9) Exit"
    echo
    read -p "Select option [1-9]: " choice

    case $choice in
        1) full_deployment ;;
        2) sync_code ;;
        3) build_services ;;
        4) deploy_services ;;
        5) verify_deployment ;;
        6) show_logs ;;
        7) restart_services ;;
        8) stop_services ;;
        9) exit 0 ;;
        *) print_error "Invalid option"; show_menu ;;
    esac
}

full_deployment() {
    print_header "Starting Full Deployment"

    check_prerequisites
    sync_code
    setup_environment
    build_services
    deploy_services
    verify_deployment

    print_header "Deployment Complete"
    print_success "Bot Core has been deployed to Viettel VPS!"
    print_info "Access dashboard at: http://$VPS_IP:3000"
    print_info "Rust API at: http://$VPS_IP:8080"
    print_info "Python API at: http://$VPS_IP:8000"

    show_menu
}

restart_services() {
    print_header "Restarting Services"
    ssh $VPS_USER@$VPS_IP "cd $DEPLOY_DIR && docker compose restart"
    print_success "Services restarted"
    show_menu
}

stop_services() {
    print_header "Stopping Services"
    ssh $VPS_USER@$VPS_IP "cd $DEPLOY_DIR && docker compose down"
    print_success "Services stopped"
    show_menu
}

# =============================================================================
# Main Entry Point
# =============================================================================

main() {
    # Check if VPS_IP is set
    if [ -z "$VPS_IP" ]; then
        print_error "VPS_IP is not configured!"
        echo
        read -p "Enter VPS IP address: " VPS_IP

        # Update script with VPS_IP
        sed -i.bak "s/^VPS_IP=\"\"/VPS_IP=\"$VPS_IP\"/" "$0"
        print_success "VPS_IP saved to script"
    fi

    show_menu
}

# Run main function
main
