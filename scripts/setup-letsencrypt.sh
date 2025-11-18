#!/usr/bin/env bash

# @spec:ARCH-SECURITY-004 - TLS/SSL Configuration
# @ref:specs/02-design/2.1-architecture/ARCH-SECURITY.md
# Let's Encrypt Setup Script for Bot-Core

set -e  # Exit on error
set -u  # Exit on undefined variable

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
CERTS_DIR="${PROJECT_ROOT}/infrastructure/nginx/certs/prod"

# Default values
DOMAIN=""
EMAIL=""
STAGING=false
WEBROOT="/var/www/certbot"
DRY_RUN=false

# Functions
print_info() {
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

show_usage() {
    cat << EOF
Usage: $0 --domain DOMAIN --email EMAIL [OPTIONS]

Setup Let's Encrypt SSL certificates for production

REQUIRED OPTIONS:
    --domain DOMAIN       Domain name (e.g., api.botcore.com)
    --email EMAIL         Email for Let's Encrypt notifications

OPTIONAL:
    --staging             Use Let's Encrypt staging server (for testing)
    --webroot PATH        Webroot path for verification (default: /var/www/certbot)
    --dry-run             Simulate certificate request without saving
    -h, --help            Show this help message

EXAMPLES:
    # Production: Request real certificate
    $0 --domain api.botcore.com --email admin@botcore.com

    # Staging: Test setup without rate limits
    $0 --domain api.botcore.com --email admin@botcore.com --staging

    # Dry run: Validate configuration
    $0 --domain api.botcore.com --email admin@botcore.com --dry-run

PREREQUISITES:
    1. DNS A record pointing to your server IP
    2. Port 80 accessible (for HTTP-01 challenge)
    3. certbot installed (brew install certbot OR apt-get install certbot)
    4. nginx running and configured for ACME challenge

NOTES:
    - Let's Encrypt certificates are valid for 90 days
    - Auto-renewal is setup via cron (scripts/renew-ssl.sh)
    - Rate limit: 50 certificates per domain per week
    - Use --staging for testing to avoid rate limits

EOF
}

check_requirements() {
    # Check certbot
    if ! command -v certbot &> /dev/null; then
        print_error "certbot is not installed"
        print_info "Install:"
        print_info "  macOS: brew install certbot"
        print_info "  Ubuntu/Debian: apt-get install certbot"
        print_info "  CentOS/RHEL: yum install certbot"
        exit 1
    fi
    print_info "certbot version: $(certbot --version 2>&1 | head -n1)"

    # Check domain
    if [[ -z "${DOMAIN}" ]]; then
        print_error "--domain is required"
        show_usage
        exit 1
    fi

    # Check email
    if [[ -z "${EMAIL}" ]]; then
        print_error "--email is required"
        show_usage
        exit 1
    fi

    # Check DNS resolution
    print_info "Checking DNS resolution for ${DOMAIN}..."
    if ! host "${DOMAIN}" &> /dev/null; then
        print_warning "DNS resolution failed for ${DOMAIN}"
        print_info "Make sure DNS A record points to this server"
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    else
        local resolved_ip=$(host "${DOMAIN}" | grep "has address" | awk '{print $4}' | head -n1)
        print_success "DNS resolved: ${DOMAIN} → ${resolved_ip}"
    fi
}

setup_webroot() {
    print_info "Setting up webroot for ACME challenge..."

    # Create webroot directory
    sudo mkdir -p "${WEBROOT}/.well-known/acme-challenge"
    sudo chmod -R 755 "${WEBROOT}"

    # Create test file
    echo "acme-challenge-test" | sudo tee "${WEBROOT}/.well-known/acme-challenge/test.txt" > /dev/null

    print_success "Webroot ready: ${WEBROOT}"
    print_info "Test URL: http://${DOMAIN}/.well-known/acme-challenge/test.txt"
}

request_certificate() {
    print_info "Requesting Let's Encrypt certificate..."

    local certbot_args=(
        "certonly"
        "--webroot"
        "-w" "${WEBROOT}"
        "-d" "${DOMAIN}"
        "--email" "${EMAIL}"
        "--agree-tos"
        "--no-eff-email"
        "--non-interactive"
    )

    # Add staging flag if requested
    if [[ "${STAGING}" == "true" ]]; then
        certbot_args+=("--staging")
        print_warning "Using Let's Encrypt STAGING server (certificates not trusted)"
    fi

    # Add dry-run flag if requested
    if [[ "${DRY_RUN}" == "true" ]]; then
        certbot_args+=("--dry-run")
        print_info "DRY RUN mode: No certificates will be saved"
    fi

    # Request certificate
    print_info "Running certbot..."
    if sudo certbot "${certbot_args[@]}"; then
        if [[ "${DRY_RUN}" == "true" ]]; then
            print_success "Dry run successful! Configuration is valid."
        else
            print_success "Certificate obtained successfully!"
            copy_certificates
        fi
    else
        print_error "Certificate request failed"
        print_info "Check certbot logs: sudo cat /var/log/letsencrypt/letsencrypt.log"
        exit 1
    fi
}

copy_certificates() {
    print_info "Copying certificates to project directory..."

    # Let's Encrypt certificate location
    local le_dir="/etc/letsencrypt/live/${DOMAIN}"

    # Create project cert directory
    mkdir -p "${CERTS_DIR}"

    # Copy certificates
    sudo cp "${le_dir}/fullchain.pem" "${CERTS_DIR}/fullchain.pem"
    sudo cp "${le_dir}/privkey.pem" "${CERTS_DIR}/privkey.pem"
    sudo cp "${le_dir}/chain.pem" "${CERTS_DIR}/chain.pem" 2>/dev/null || true

    # Set ownership and permissions
    sudo chown $(whoami):$(whoami) "${CERTS_DIR}"/*.pem
    chmod 644 "${CERTS_DIR}/fullchain.pem"
    chmod 644 "${CERTS_DIR}/chain.pem" 2>/dev/null || true
    chmod 600 "${CERTS_DIR}/privkey.pem"

    print_success "Certificates copied to: ${CERTS_DIR}"
    print_info "fullchain.pem: $(ls -lh ${CERTS_DIR}/fullchain.pem | awk '{print $5}')"
    print_info "privkey.pem: $(ls -lh ${CERTS_DIR}/privkey.pem | awk '{print $5}')"
}

show_certificate_info() {
    local cert_file="${CERTS_DIR}/fullchain.pem"

    if [[ ! -f "${cert_file}" ]]; then
        return
    fi

    print_info "Certificate Information:"
    sudo openssl x509 -in "${cert_file}" -text -noout | grep -A 2 "Subject:"
    sudo openssl x509 -in "${cert_file}" -text -noout | grep -A 2 "Validity"
    sudo openssl x509 -in "${cert_file}" -text -noout | grep -A 1 "Subject Alternative Name"
}

setup_auto_renewal() {
    print_info "Setting up auto-renewal..."

    # Create renewal script
    local renewal_script="${SCRIPT_DIR}/renew-ssl.sh"

    if [[ ! -f "${renewal_script}" ]]; then
        print_warning "Renewal script not found: ${renewal_script}"
        print_info "Create it manually or run generate-ssl-certs.sh again"
        return
    fi

    # Setup cron job
    print_info "Adding cron job for auto-renewal..."

    # Check if cron job already exists
    if crontab -l 2>/dev/null | grep -q "renew-ssl.sh"; then
        print_info "Cron job already exists"
    else
        # Add cron job (daily at 2 AM)
        (crontab -l 2>/dev/null; echo "0 2 * * * ${renewal_script} >> /var/log/certbot-renew.log 2>&1") | crontab -
        print_success "Cron job added: Daily renewal check at 2 AM"
    fi

    print_info "Test renewal: sudo certbot renew --dry-run"
}

reload_nginx() {
    print_info "Reloading nginx configuration..."

    if command -v nginx &> /dev/null; then
        # Test configuration first
        if sudo nginx -t; then
            sudo nginx -s reload
            print_success "Nginx reloaded successfully"
        else
            print_error "Nginx configuration test failed"
            return 1
        fi
    elif command -v docker &> /dev/null; then
        # Reload nginx in Docker
        if docker ps | grep -q nginx; then
            docker exec nginx nginx -t && docker exec nginx nginx -s reload
            print_success "Nginx (Docker) reloaded successfully"
        else
            print_warning "Nginx container not running"
        fi
    else
        print_warning "Nginx not found. Please reload manually."
    fi
}

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --domain)
            DOMAIN="$2"
            shift 2
            ;;
        --email)
            EMAIL="$2"
            shift 2
            ;;
        --staging)
            STAGING=true
            shift
            ;;
        --webroot)
            WEBROOT="$2"
            shift 2
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        -h|--help)
            show_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            show_usage
            exit 1
            ;;
    esac
done

# Main execution
print_info "Bot-Core Let's Encrypt Setup"
print_info "Domain: ${DOMAIN}"
print_info "Email: ${EMAIL}"
echo ""

check_requirements
setup_webroot
request_certificate

if [[ "${DRY_RUN}" == "false" && "${STAGING}" == "false" ]]; then
    show_certificate_info
    setup_auto_renewal
    reload_nginx

    echo ""
    print_success "Let's Encrypt setup complete!"
    print_info "Certificates will auto-renew every 60 days"
    print_info "Check renewal status: sudo certbot certificates"
    print_info "Manual renewal: sudo certbot renew"
fi

print_success "Done!"
