#!/usr/bin/env bash

# @spec:ARCH-SECURITY-004 - TLS/SSL Configuration
# @ref:specs/02-design/2.1-architecture/ARCH-SECURITY.md
# SSL Security Verification Script for Bot-Core

set -e  # Exit on error
set -u  # Exit on undefined variable

# Color output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
HOST="${1:-localhost}"
PORT="${2:-443}"
TIMEOUT=10

# Counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0

# Functions
print_header() {
    echo ""
    echo -e "${BLUE}========================================${NC}"
    echo -e "${BLUE}$1${NC}"
    echo -e "${BLUE}========================================${NC}"
}

print_test() {
    echo -e "${BLUE}[TEST]${NC} $1"
    ((TOTAL_TESTS++))
}

print_pass() {
    echo -e "${GREEN}  ✓ PASS${NC} $1"
    ((PASSED_TESTS++))
}

print_fail() {
    echo -e "${RED}  ✗ FAIL${NC} $1"
    ((FAILED_TESTS++))
}

print_info() {
    echo -e "${BLUE}  ℹ INFO${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}  ⚠ WARNING${NC} $1"
}

check_requirements() {
    local missing=0

    # Check openssl
    if ! command -v openssl &> /dev/null; then
        echo -e "${RED}ERROR: openssl is not installed${NC}"
        missing=1
    fi

    # Check curl
    if ! command -v curl &> /dev/null; then
        echo -e "${RED}ERROR: curl is not installed${NC}"
        missing=1
    fi

    if [[ $missing -eq 1 ]]; then
        exit 1
    fi
}

# Test SSL/TLS Connection
test_ssl_connection() {
    print_test "SSL/TLS Connection"

    if timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -servername ${HOST} < /dev/null 2>&1 | grep -q "Verify return code: 0"; then
        print_pass "Successfully connected to ${HOST}:${PORT}"
    else
        local error=$(timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -servername ${HOST} < /dev/null 2>&1 | grep "Verify return code" || echo "Connection failed")
        print_fail "Connection to ${HOST}:${PORT} failed: ${error}"
    fi
}

# Test TLS Version
test_tls_version() {
    print_test "TLS Protocol Version"

    local tls_version=$(timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -servername ${HOST} < /dev/null 2>&1 | grep "Protocol" | awk '{print $3}')

    if [[ "${tls_version}" == "TLSv1.3" ]]; then
        print_pass "TLS 1.3 detected (optimal)"
    elif [[ "${tls_version}" == "TLSv1.2" ]]; then
        print_pass "TLS 1.2 detected (acceptable)"
    else
        print_fail "TLS version ${tls_version} is outdated (require TLS 1.2+)"
    fi

    # Check for weak protocols
    if timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -ssl3 < /dev/null 2>&1 | grep -q "SSL-Session"; then
        print_fail "SSLv3 is enabled (VULNERABLE)"
    else
        print_pass "SSLv3 is disabled"
    fi

    if timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -tls1 < /dev/null 2>&1 | grep -q "SSL-Session"; then
        print_fail "TLS 1.0 is enabled (deprecated)"
    else
        print_pass "TLS 1.0 is disabled"
    fi

    if timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -tls1_1 < /dev/null 2>&1 | grep -q "SSL-Session"; then
        print_fail "TLS 1.1 is enabled (deprecated)"
    else
        print_pass "TLS 1.1 is disabled"
    fi
}

# Test Cipher Suites
test_cipher_suites() {
    print_test "Cipher Suites"

    local cipher=$(timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -servername ${HOST} -cipher 'ECDHE' < /dev/null 2>&1 | grep "Cipher" | awk '{print $3}')

    if [[ -n "${cipher}" ]]; then
        print_pass "Strong cipher suite: ${cipher}"
    else
        print_warning "Could not determine cipher suite"
    fi

    # Check for weak ciphers
    if timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -cipher 'RC4' < /dev/null 2>&1 | grep -q "Cipher"; then
        print_fail "RC4 cipher is enabled (INSECURE)"
    else
        print_pass "RC4 cipher is disabled"
    fi

    if timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -cipher 'DES' < /dev/null 2>&1 | grep -q "Cipher"; then
        print_fail "DES cipher is enabled (INSECURE)"
    else
        print_pass "DES cipher is disabled"
    fi
}

# Test Certificate
test_certificate() {
    print_test "SSL Certificate"

    local cert_info=$(timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -servername ${HOST} < /dev/null 2>&1)

    # Check expiry
    local expiry_date=$(echo "${cert_info}" | openssl x509 -noout -enddate 2>/dev/null | cut -d= -f2)
    if [[ -n "${expiry_date}" ]]; then
        print_info "Certificate expires: ${expiry_date}"

        local expiry_epoch=$(date -d "${expiry_date}" +%s 2>/dev/null || date -j -f "%b %d %H:%M:%S %Y %Z" "${expiry_date}" +%s 2>/dev/null)
        local current_epoch=$(date +%s)
        local days_until_expiry=$(( (expiry_epoch - current_epoch) / 86400 ))

        if [[ $days_until_expiry -lt 0 ]]; then
            print_fail "Certificate has EXPIRED"
        elif [[ $days_until_expiry -lt 14 ]]; then
            print_warning "Certificate expiring in ${days_until_expiry} days (renew soon)"
        else
            print_pass "Certificate valid for ${days_until_expiry} days"
        fi
    fi

    # Check subject
    local subject=$(echo "${cert_info}" | openssl x509 -noout -subject 2>/dev/null | sed 's/subject=//')
    if [[ -n "${subject}" ]]; then
        print_info "Subject: ${subject}"
    fi

    # Check issuer
    local issuer=$(echo "${cert_info}" | openssl x509 -noout -issuer 2>/dev/null | sed 's/issuer=//')
    if [[ -n "${issuer}" ]]; then
        print_info "Issuer: ${issuer}"

        if echo "${issuer}" | grep -q "Let's Encrypt"; then
            print_pass "Certificate issued by Let's Encrypt"
        elif echo "${subject}" | grep -q "CN=${HOST}"; then
            print_info "Self-signed certificate (development)"
        fi
    fi
}

# Test Security Headers
test_security_headers() {
    print_test "Security Headers"

    local url="https://${HOST}"
    if [[ "${PORT}" != "443" ]]; then
        url="https://${HOST}:${PORT}"
    fi

    # HSTS
    if curl -s -I -k --max-time $TIMEOUT "${url}" | grep -qi "Strict-Transport-Security"; then
        local hsts=$(curl -s -I -k --max-time $TIMEOUT "${url}" | grep -i "Strict-Transport-Security" | cut -d: -f2-)
        print_pass "HSTS enabled:${hsts}"
    else
        print_fail "HSTS header missing"
    fi

    # X-Frame-Options
    if curl -s -I -k --max-time $TIMEOUT "${url}" | grep -qi "X-Frame-Options"; then
        local xfo=$(curl -s -I -k --max-time $TIMEOUT "${url}" | grep -i "X-Frame-Options" | cut -d: -f2-)
        print_pass "X-Frame-Options:${xfo}"
    else
        print_fail "X-Frame-Options header missing"
    fi

    # X-Content-Type-Options
    if curl -s -I -k --max-time $TIMEOUT "${url}" | grep -qi "X-Content-Type-Options"; then
        print_pass "X-Content-Type-Options enabled"
    else
        print_fail "X-Content-Type-Options header missing"
    fi

    # Content-Security-Policy
    if curl -s -I -k --max-time $TIMEOUT "${url}" | grep -qi "Content-Security-Policy"; then
        print_pass "Content-Security-Policy enabled"
    else
        print_warning "Content-Security-Policy header missing"
    fi

    # Referrer-Policy
    if curl -s -I -k --max-time $TIMEOUT "${url}" | grep -qi "Referrer-Policy"; then
        print_pass "Referrer-Policy enabled"
    else
        print_warning "Referrer-Policy header missing"
    fi
}

# Test HTTP to HTTPS Redirect
test_http_redirect() {
    print_test "HTTP to HTTPS Redirect"

    local http_response=$(curl -s -o /dev/null -w "%{http_code}" --max-time $TIMEOUT "http://${HOST}")

    if [[ "${http_response}" == "301" ]] || [[ "${http_response}" == "302" ]]; then
        print_pass "HTTP redirects to HTTPS (${http_response})"
    else
        print_fail "HTTP does not redirect to HTTPS (got ${http_response})"
    fi
}

# Test OCSP Stapling
test_ocsp_stapling() {
    print_test "OCSP Stapling"

    if timeout $TIMEOUT openssl s_client -connect ${HOST}:${PORT} -servername ${HOST} -status < /dev/null 2>&1 | grep -q "OCSP Response Status: successful"; then
        print_pass "OCSP stapling enabled"
    else
        print_info "OCSP stapling not detected (optional)"
    fi
}

# Generate Summary Report
generate_summary() {
    print_header "Test Summary"

    echo ""
    echo "Total Tests:  ${TOTAL_TESTS}"
    echo -e "Passed:       ${GREEN}${PASSED_TESTS}${NC}"
    echo -e "Failed:       ${RED}${FAILED_TESTS}${NC}"
    echo ""

    local pass_rate=$(( PASSED_TESTS * 100 / TOTAL_TESTS ))

    if [[ $FAILED_TESTS -eq 0 ]]; then
        echo -e "${GREEN}✓ All tests passed! SSL/TLS configuration is secure.${NC}"
        return 0
    elif [[ $pass_rate -ge 80 ]]; then
        echo -e "${YELLOW}⚠ Some tests failed (${pass_rate}% pass rate). Review warnings above.${NC}"
        return 1
    else
        echo -e "${RED}✗ Multiple tests failed (${pass_rate}% pass rate). SSL/TLS configuration needs improvement.${NC}"
        return 1
    fi
}

# Main execution
main() {
    print_header "Bot-Core SSL/TLS Security Verification"
    echo "Target: ${HOST}:${PORT}"
    echo ""

    check_requirements

    test_ssl_connection
    test_tls_version
    test_cipher_suites
    test_certificate
    test_security_headers
    test_http_redirect
    test_ocsp_stapling

    echo ""
    generate_summary

    exit $?
}

# Run
main
