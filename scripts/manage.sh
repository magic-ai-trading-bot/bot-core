#!/bin/bash

# 🛠️ Trading Bot Management Script for Fly.io
# Utility script to manage deployed services

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

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

# App names
APPS=("trading-bot-database" "trading-bot-ai-service" "trading-bot-rust-engine" "trading-bot-dashboard")

show_help() {
    echo "🛠️  Trading Bot Management Script"
    echo ""
    echo "Usage: $0 <command> [app_name]"
    echo ""
    echo "Commands:"
    echo "  status          - Show status of all services"
    echo "  logs <app>      - Show logs for specific app"
    echo "  restart <app>   - Restart specific app"
    echo "  scale <app>     - Scale app resources"
    echo "  ssh <app>       - SSH into app container"
    echo "  secrets <app>   - List secrets for app"
    echo "  update <app>    - Update specific app"
    echo "  destroy         - Destroy all apps (careful!)"
    echo "  help            - Show this help"
    echo ""
    echo "Available apps:"
    for app in "${APPS[@]}"; do
        echo "  - $app"
    done
    echo ""
    echo "Examples:"
    echo "  $0 status"
    echo "  $0 logs trading-bot-rust-engine"
    echo "  $0 restart trading-bot-ai-service"
    echo "  $0 ssh trading-bot-rust-engine"
}

check_app_exists() {
    local app_name=$1
    if [[ ! " ${APPS[@]} " =~ " ${app_name} " ]]; then
        print_error "App '$app_name' not found. Available apps: ${APPS[*]}"
        exit 1
    fi
}

show_status() {
    print_step "Checking status of all services..."
    echo ""
    
    for app in "${APPS[@]}"; do
        echo "📊 $app:"
        if flyctl status --app $app 2>/dev/null; then
            print_success "✅ Running: https://$app.fly.dev"
        else
            print_error "❌ Not running or not found"
        fi
        echo ""
    done
}

show_logs() {
    local app_name=$1
    check_app_exists $app_name
    
    print_step "Showing logs for $app_name..."
    flyctl logs --app $app_name
}

restart_app() {
    local app_name=$1
    check_app_exists $app_name
    
    print_step "Restarting $app_name..."
    flyctl restart --app $app_name
    print_success "Restarted $app_name"
}

scale_app() {
    local app_name=$1
    check_app_exists $app_name
    
    print_step "Current scale for $app_name:"
    flyctl scale show --app $app_name
    
    echo ""
    echo "Scale options:"
    echo "1. Shared CPU 1x, 512MB RAM"
    echo "2. Shared CPU 1x, 1GB RAM"  
    echo "3. Shared CPU 1x, 2GB RAM"
    echo "4. Shared CPU 2x, 2GB RAM"
    echo "5. Custom"
    
    read -p "Choose option (1-5): " choice
    
    case $choice in
        1) flyctl scale vm shared-cpu-1x --memory 512 --app $app_name ;;
        2) flyctl scale vm shared-cpu-1x --memory 1024 --app $app_name ;;
        3) flyctl scale vm shared-cpu-1x --memory 2048 --app $app_name ;;
        4) flyctl scale vm shared-cpu-2x --memory 2048 --app $app_name ;;
        5) 
            read -p "Enter VM size (e.g., shared-cpu-1x): " vm_size
            read -p "Enter memory in MB: " memory
            flyctl scale vm $vm_size --memory $memory --app $app_name
            ;;
        *) print_error "Invalid option" ;;
    esac
}

ssh_app() {
    local app_name=$1
    check_app_exists $app_name
    
    print_step "Connecting to $app_name..."
    flyctl ssh console --app $app_name
}

list_secrets() {
    local app_name=$1
    check_app_exists $app_name
    
    print_step "Secrets for $app_name:"
    flyctl secrets list --app $app_name
}

update_app() {
    local app_name=$1
    check_app_exists $app_name
    
    print_step "Updating $app_name..."
    
    case $app_name in
        "trading-bot-database")
            flyctl deploy --config fly-database.toml --remote-only
            ;;
        "trading-bot-ai-service")
            cd python-ai-service && flyctl deploy --remote-only && cd ..
            ;;
        "trading-bot-rust-engine")
            cd rust-core-engine && flyctl deploy --remote-only && cd ..
            ;;
        "trading-bot-dashboard")
            cd nextjs-ui-dashboard && flyctl deploy --remote-only && cd ..
            ;;
    esac
    
    print_success "Updated $app_name"
}

destroy_all() {
    print_warning "⚠️  This will DESTROY all apps and data!"
    read -p "Are you sure? Type 'YES' to confirm: " confirm
    
    if [ "$confirm" = "YES" ]; then
        for app in "${APPS[@]}"; do
            print_step "Destroying $app..."
            flyctl apps destroy $app --yes || true
        done
        print_success "All apps destroyed"
    else
        print_step "Destruction cancelled"
    fi
}

# Main script logic
case $1 in
    "status")
        show_status
        ;;
    "logs")
        if [ -z "$2" ]; then
            print_error "Please specify an app name"
            show_help
            exit 1
        fi
        show_logs $2
        ;;
    "restart")
        if [ -z "$2" ]; then
            print_error "Please specify an app name"
            show_help
            exit 1
        fi
        restart_app $2
        ;;
    "scale")
        if [ -z "$2" ]; then
            print_error "Please specify an app name"
            show_help
            exit 1
        fi
        scale_app $2
        ;;
    "ssh")
        if [ -z "$2" ]; then
            print_error "Please specify an app name"
            show_help
            exit 1
        fi
        ssh_app $2
        ;;
    "secrets")
        if [ -z "$2" ]; then
            print_error "Please specify an app name"
            show_help
            exit 1
        fi
        list_secrets $2
        ;;
    "update")
        if [ -z "$2" ]; then
            print_error "Please specify an app name"
            show_help
            exit 1
        fi
        update_app $2
        ;;
    "destroy")
        destroy_all
        ;;
    "help"|*)
        show_help
        ;;
esac 