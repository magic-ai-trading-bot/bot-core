#!/usr/bin/env bash

# @spec:ARCH-SECURITY-004 - TLS/SSL Configuration
# @ref:specs/02-design/2.1-architecture/ARCH-SECURITY.md
# SSL Certificate Generation Script for Bot-Core

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
CERTS_DIR="${PROJECT_ROOT}/infrastructure/nginx/certs"

# Default values
MODE="dev"
FORCE=false
DOMAIN="bot-core.local"
KEY_SIZE=2048
DAYS=365

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
Usage: $0 [OPTIONS]

Generate SSL/TLS certificates for Bot-Core

OPTIONS:
    --dev                 Generate self-signed certificates for development (default)
    --prod                Generate CSR for production Let's Encrypt
    --domain DOMAIN       Domain name (default: bot-core.local for dev, required for prod)
    --key-size SIZE       RSA key size: 2048 or 4096 (default: 2048)
    --days DAYS           Certificate validity in days (default: 365)
    --force               Overwrite existing certificates
    --ecdsa               Use ECDSA P-256 instead of RSA
    -h, --help            Show this help message

EXAMPLES:
    # Development: Generate self-signed certificate
    $0 --dev

    # Development: Custom domain and validity
    $0 --dev --domain localhost --days 730

    # Production: Generate CSR for Let's Encrypt
    $0 --prod --domain api.botcore.com

    # Use ECDSA (recommended for production)
    $0 --dev --ecdsa

    # Force regeneration
    $0 --dev --force

NOTES:
    - Development certificates are self-signed (not trusted by browsers)
    - Production certificates require Let's Encrypt setup (use setup-letsencrypt.sh)
    - Private keys are stored with 600 permissions
    - Certificates are stored with 644 permissions

EOF
}

check_openssl() {
    if ! command -v openssl &> /dev/null; then
        print_error "openssl is not installed"
        print_info "Install: brew install openssl (macOS) or apt-get install openssl (Linux)"
        exit 1
    fi
    print_info "OpenSSL version: $(openssl version)"
}

generate_dev_rsa() {
    local cert_dir="${CERTS_DIR}/dev"
    local key_file="${cert_dir}/key.pem"
    local cert_file="${cert_dir}/cert.pem"

    print_info "Generating RSA ${KEY_SIZE}-bit self-signed certificate for development..."

    # Create directory
    mkdir -p "${cert_dir}"

    # Check if certificates already exist
    if [[ -f "${cert_file}" && "${FORCE}" == "false" ]]; then
        print_warning "Certificate already exists: ${cert_file}"
        print_info "Use --force to overwrite"
        exit 0
    fi

    # Generate private key
    print_info "Step 1/2: Generating private key..."
    openssl genrsa -out "${key_file}" "${KEY_SIZE}" 2>/dev/null

    # Generate self-signed certificate
    print_info "Step 2/2: Generating self-signed certificate..."
    openssl req -new -x509 -key "${key_file}" -out "${cert_file}" -days "${DAYS}" \
        -subj "/C=US/ST=Development/L=Local/O=Bot-Core/OU=Development/CN=${DOMAIN}" \
        -addext "subjectAltName=DNS:${DOMAIN},DNS:localhost,IP:127.0.0.1" \
        2>/dev/null

    # Set permissions
    chmod 600 "${key_file}"
    chmod 644 "${cert_file}"

    print_success "Development certificates generated successfully!"
    print_info "Certificate: ${cert_file}"
    print_info "Private Key: ${key_file}"
    print_info "Valid for: ${DAYS} days"
    print_info "Domain: ${DOMAIN}"

    # Display certificate info
    echo ""
    print_info "Certificate Details:"
    openssl x509 -in "${cert_file}" -text -noout | grep -A 3 "Subject:"
    openssl x509 -in "${cert_file}" -text -noout | grep -A 1 "Validity"

    # Trust instructions
    echo ""
    print_warning "Browser Trust Setup:"
    print_info "macOS: sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain ${cert_file}"
    print_info "Linux: sudo cp ${cert_file} /usr/local/share/ca-certificates/bot-core.crt && sudo update-ca-certificates"
}

generate_dev_ecdsa() {
    local cert_dir="${CERTS_DIR}/dev"
    local key_file="${cert_dir}/key.pem"
    local cert_file="${cert_dir}/cert.pem"

    print_info "Generating ECDSA P-256 self-signed certificate for development..."

    # Create directory
    mkdir -p "${cert_dir}"

    # Check if certificates already exist
    if [[ -f "${cert_file}" && "${FORCE}" == "false" ]]; then
        print_warning "Certificate already exists: ${cert_file}"
        print_info "Use --force to overwrite"
        exit 0
    fi

    # Generate private key (ECDSA P-256)
    print_info "Step 1/2: Generating ECDSA private key..."
    openssl ecparam -genkey -name prime256v1 -out "${key_file}"

    # Generate self-signed certificate
    print_info "Step 2/2: Generating self-signed certificate..."
    openssl req -new -x509 -key "${key_file}" -out "${cert_file}" -days "${DAYS}" \
        -subj "/C=US/ST=Development/L=Local/O=Bot-Core/OU=Development/CN=${DOMAIN}" \
        -addext "subjectAltName=DNS:${DOMAIN},DNS:localhost,IP:127.0.0.1" \
        2>/dev/null

    # Set permissions
    chmod 600 "${key_file}"
    chmod 644 "${cert_file}"

    print_success "Development certificates (ECDSA) generated successfully!"
    print_info "Certificate: ${cert_file}"
    print_info "Private Key: ${key_file}"
    print_info "Algorithm: ECDSA P-256"
    print_info "Valid for: ${DAYS} days"
    print_info "Domain: ${DOMAIN}"
}

generate_prod_csr() {
    local cert_dir="${CERTS_DIR}/prod"
    local key_file="${cert_dir}/privkey.pem"
    local csr_file="${cert_dir}/csr.pem"
    local config_file="${cert_dir}/openssl.cnf"

    if [[ "${DOMAIN}" == "bot-core.local" ]]; then
        print_error "Production mode requires --domain option"
        print_info "Example: $0 --prod --domain api.botcore.com"
        exit 1
    fi

    print_info "Generating CSR for production Let's Encrypt..."
    print_warning "This generates a Certificate Signing Request (CSR)"
    print_info "Use setup-letsencrypt.sh to obtain actual certificates"

    # Create directory
    mkdir -p "${cert_dir}"

    # Check if key already exists
    if [[ -f "${key_file}" && "${FORCE}" == "false" ]]; then
        print_warning "Private key already exists: ${key_file}"
        print_info "Use --force to overwrite"
        exit 0
    fi

    # Create OpenSSL config
    cat > "${config_file}" << EOF
[req]
default_bits = ${KEY_SIZE}
prompt = no
default_md = sha256
distinguished_name = dn
req_extensions = v3_req

[dn]
C = US
ST = Production
L = Cloud
O = Bot-Core
OU = Trading Platform
CN = ${DOMAIN}

[v3_req]
subjectAltName = @alt_names

[alt_names]
DNS.1 = ${DOMAIN}
DNS.2 = www.${DOMAIN}
EOF

    # Generate private key
    print_info "Step 1/2: Generating private key..."
    openssl genrsa -out "${key_file}" "${KEY_SIZE}" 2>/dev/null

    # Generate CSR
    print_info "Step 2/2: Generating Certificate Signing Request (CSR)..."
    openssl req -new -key "${key_file}" -out "${csr_file}" -config "${config_file}"

    # Set permissions
    chmod 600 "${key_file}"
    chmod 644 "${csr_file}"

    print_success "CSR generated successfully!"
    print_info "Private Key: ${key_file}"
    print_info "CSR: ${csr_file}"
    print_info "Domain: ${DOMAIN}"
    echo ""
    print_info "Next Steps:"
    print_info "1. Run: ./scripts/setup-letsencrypt.sh --domain ${DOMAIN}"
    print_info "2. Let's Encrypt will use this key to issue certificates"
}

generate_dhparam() {
    local dhparam_file="${CERTS_DIR}/prod/dhparam.pem"

    if [[ -f "${dhparam_file}" && "${FORCE}" == "false" ]]; then
        print_info "DH parameters already exist: ${dhparam_file}"
        return 0
    fi

    print_info "Generating Diffie-Hellman parameters (2048-bit)..."
    print_warning "This may take several minutes..."

    mkdir -p "${CERTS_DIR}/prod"
    openssl dhparam -out "${dhparam_file}" 2048 2>/dev/null

    chmod 600 "${dhparam_file}"
    print_success "DH parameters generated: ${dhparam_file}"
}

# Parse arguments
USE_ECDSA=false

while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            MODE="dev"
            shift
            ;;
        --prod)
            MODE="prod"
            shift
            ;;
        --domain)
            DOMAIN="$2"
            shift 2
            ;;
        --key-size)
            KEY_SIZE="$2"
            shift 2
            ;;
        --days)
            DAYS="$2"
            shift 2
            ;;
        --ecdsa)
            USE_ECDSA=true
            shift
            ;;
        --force)
            FORCE=true
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
print_info "Bot-Core SSL Certificate Generator"
print_info "Mode: ${MODE}"
echo ""

check_openssl

case "${MODE}" in
    dev)
        if [[ "${USE_ECDSA}" == "true" ]]; then
            generate_dev_ecdsa
        else
            generate_dev_rsa
        fi
        ;;
    prod)
        generate_prod_csr
        generate_dhparam
        ;;
    *)
        print_error "Invalid mode: ${MODE}"
        exit 1
        ;;
esac

print_success "Done!"
