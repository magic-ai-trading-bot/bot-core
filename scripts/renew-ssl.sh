#!/usr/bin/env bash

# @spec:ARCH-SECURITY-004 - TLS/SSL Configuration
# @ref:specs/02-design/2.1-architecture/ARCH-SECURITY.md
# SSL Certificate Renewal Script for Bot-Core
# Cron: 0 2 * * * /path/to/renew-ssl.sh

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
LOG_FILE="/var/log/certbot-renew.log"

# Functions
log_info() {
    echo -e "${BLUE}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} [INFO] $1" | tee -a "${LOG_FILE}"
}

log_success() {
    echo -e "${GREEN}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} [SUCCESS] $1" | tee -a "${LOG_FILE}"
}

log_warning() {
    echo -e "${YELLOW}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} [WARNING] $1" | tee -a "${LOG_FILE}"
}

log_error() {
    echo -e "${RED}[$(date '+%Y-%m-%d %H:%M:%S')]${NC} [ERROR] $1" | tee -a "${LOG_FILE}"
}

check_certificate_expiry() {
    log_info "Checking certificate expiry..."

    # Find all certificates
    local certs=$(sudo certbot certificates 2>/dev/null | grep "Certificate Path" | awk '{print $3}')

    if [[ -z "${certs}" ]]; then
        log_warning "No certificates found"
        return 1
    fi

    for cert in ${certs}; do
        local domain=$(sudo openssl x509 -in "${cert}" -noout -subject | sed 's/.*CN=\([^,]*\).*/\1/')
        local expiry=$(sudo openssl x509 -in "${cert}" -noout -enddate | cut -d= -f2)
        local expiry_epoch=$(date -d "${expiry}" +%s 2>/dev/null || date -j -f "%b %d %H:%M:%S %Y %Z" "${expiry}" +%s)
        local current_epoch=$(date +%s)
        local days_until_expiry=$(( (expiry_epoch - current_epoch) / 86400 ))

        log_info "Certificate: ${domain}"
        log_info "  Expires: ${expiry}"
        log_info "  Days until expiry: ${days_until_expiry}"

        if [[ ${days_until_expiry} -lt 7 ]]; then
            log_warning "  Certificate expiring soon!"
        fi
    done
}

renew_certificates() {
    log_info "Attempting to renew certificates..."

    # Renew certificates (only if within 30 days of expiry)
    if sudo certbot renew --quiet --no-self-upgrade; then
        log_success "Certificate renewal successful"
        return 0
    else
        log_error "Certificate renewal failed"
        return 1
    fi
}

copy_renewed_certificates() {
    log_info "Copying renewed certificates..."

    # Find all renewed certificates
    local le_dirs=$(sudo ls -d /etc/letsencrypt/live/*/ 2>/dev/null | grep -v "README")

    for le_dir in ${le_dirs}; do
        local domain=$(basename "${le_dir}")

        # Check if certificate was recently renewed (modified in last 24 hours)
        local cert_file="${le_dir}/fullchain.pem"
        if [[ -f "${cert_file}" ]]; then
            local mod_time=$(stat -c %Y "${cert_file}" 2>/dev/null || stat -f %m "${cert_file}")
            local current_time=$(date +%s)
            local age=$(( current_time - mod_time ))

            if [[ ${age} -lt 86400 ]]; then
                log_info "Certificate recently renewed: ${domain}"

                # Copy to project directory
                if [[ -d "${CERTS_DIR}" ]]; then
                    sudo cp "${cert_file}" "${CERTS_DIR}/fullchain.pem"
                    sudo cp "${le_dir}/privkey.pem" "${CERTS_DIR}/privkey.pem"
                    sudo cp "${le_dir}/chain.pem" "${CERTS_DIR}/chain.pem" 2>/dev/null || true

                    sudo chown $(whoami):$(whoami) "${CERTS_DIR}"/*.pem
                    chmod 644 "${CERTS_DIR}/fullchain.pem"
                    chmod 644 "${CERTS_DIR}/chain.pem" 2>/dev/null || true
                    chmod 600 "${CERTS_DIR}/privkey.pem"

                    log_success "Certificates copied to project directory"
                fi
            fi
        fi
    done
}

reload_services() {
    log_info "Reloading services..."

    # Reload nginx
    if command -v nginx &> /dev/null; then
        if sudo nginx -t &> /dev/null; then
            sudo nginx -s reload
            log_success "Nginx reloaded"
        else
            log_error "Nginx configuration test failed"
            return 1
        fi
    elif command -v docker &> /dev/null; then
        if docker ps | grep -q nginx; then
            if docker exec nginx nginx -t &> /dev/null; then
                docker exec nginx nginx -s reload
                log_success "Nginx (Docker) reloaded"
            else
                log_error "Nginx (Docker) configuration test failed"
                return 1
            fi
        fi
    fi

    # Reload other services if needed (e.g., HAProxy, Caddy)
    # Add your service reload commands here
}

send_alert() {
    local status="$1"
    local message="$2"

    # Send email alert (if configured)
    if command -v mail &> /dev/null && [[ -n "${ALERT_EMAIL:-}" ]]; then
        echo "${message}" | mail -s "Bot-Core SSL Renewal: ${status}" "${ALERT_EMAIL}"
    fi

    # Send Slack notification (if configured)
    if [[ -n "${SLACK_WEBHOOK_URL:-}" ]]; then
        curl -X POST "${SLACK_WEBHOOK_URL}" \
            -H 'Content-Type: application/json' \
            -d "{\"text\": \"SSL Renewal ${status}: ${message}\"}" \
            &> /dev/null
    fi

    # Send to monitoring system (if configured)
    # Add your monitoring integration here
}

cleanup_old_logs() {
    log_info "Cleaning up old logs..."

    # Keep last 30 days of logs
    if [[ -f "${LOG_FILE}" ]]; then
        local log_size=$(du -h "${LOG_FILE}" | awk '{print $1}')
        log_info "Log file size: ${log_size}"

        # If log file > 10MB, rotate it
        if [[ $(stat -c %s "${LOG_FILE}" 2>/dev/null || stat -f %z "${LOG_FILE}") -gt 10485760 ]]; then
            sudo mv "${LOG_FILE}" "${LOG_FILE}.$(date +%Y%m%d)"
            sudo gzip "${LOG_FILE}.$(date +%Y%m%d)" &
            log_info "Log file rotated"
        fi
    fi

    # Delete logs older than 30 days
    find "$(dirname ${LOG_FILE})" -name "certbot-renew.log.*" -mtime +30 -delete 2>/dev/null || true
}

# Main execution
log_info "===== SSL Certificate Renewal Check ====="

# Check if running as root or with sudo
if [[ $EUID -ne 0 && -z "${SUDO_USER:-}" ]]; then
    log_warning "This script should be run with sudo for certificate renewal"
fi

# Check certificate expiry
check_certificate_expiry

# Attempt renewal
if renew_certificates; then
    copy_renewed_certificates

    if reload_services; then
        log_success "Services reloaded successfully"
        send_alert "SUCCESS" "SSL certificates renewed and services reloaded"
    else
        log_error "Failed to reload services"
        send_alert "WARNING" "SSL certificates renewed but service reload failed"
        exit 1
    fi
else
    # Renewal failed (or not needed)
    # certbot renew exits with 0 if no renewal needed, 1 if renewal failed
    if sudo certbot certificates | grep -q "INVALID"; then
        log_error "Certificate renewal failed - certificates may be invalid"
        send_alert "CRITICAL" "SSL certificate renewal FAILED"
        exit 1
    else
        log_info "No certificates need renewal at this time"
    fi
fi

# Cleanup
cleanup_old_logs

log_info "===== Renewal Check Complete ====="
