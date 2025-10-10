#!/bin/bash
set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Get the script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  Security Scan - Trading Bot System  ${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# Track overall status
TOTAL_VULNS=0
SCAN_FAILED=0

# Function to print section header
print_header() {
    echo -e "\n${BLUE}>>> $1${NC}\n"
}

# Function to print success
print_success() {
    echo -e "${GREEN}✓ $1${NC}"
}

# Function to print warning
print_warning() {
    echo -e "${YELLOW}⚠ $1${NC}"
}

# Function to print error
print_error() {
    echo -e "${RED}✗ $1${NC}"
}

# ====================================
# 1. Python Security Audit
# ====================================
print_header "1. Python AI Service Security Audit"

cd "$PROJECT_ROOT/python-ai-service"

# Check if pip-audit is available
if ! command -v pip-audit &> /dev/null && ! python3 -m pip_audit --version &> /dev/null; then
    print_error "pip-audit not found. Installing..."
    python3 -m pip install pip-audit
fi

# Run pip-audit on all requirements files
echo "Scanning requirements.txt..."
if python3 -m pip_audit --requirement requirements.txt --desc; then
    print_success "requirements.txt: No vulnerabilities found"
else
    VULNS=$(python3 -m pip_audit --requirement requirements.txt 2>&1 | grep "Found" | grep -oE '[0-9]+' | head -1)
    if [ -n "$VULNS" ]; then
        TOTAL_VULNS=$((TOTAL_VULNS + VULNS))
        print_error "requirements.txt: Found $VULNS vulnerabilities"
    else
        print_warning "requirements.txt: Scan completed with warnings"
    fi
fi

echo ""
echo "Scanning requirements.dev.txt..."
if python3 -m pip_audit --requirement requirements.dev.txt --desc 2>/dev/null; then
    print_success "requirements.dev.txt: No vulnerabilities found"
else
    print_warning "requirements.dev.txt: Check completed (dev dependencies may have warnings)"
fi

echo ""
echo "Scanning requirements.test.txt..."
if python3 -m pip_audit --requirement requirements.test.txt --desc 2>/dev/null; then
    print_success "requirements.test.txt: No vulnerabilities found"
else
    print_warning "requirements.test.txt: Check completed (test dependencies may have warnings)"
fi

# ====================================
# 2. Rust Security Audit
# ====================================
print_header "2. Rust Core Engine Security Audit"

cd "$PROJECT_ROOT/rust-core-engine"

# Check if cargo-deny is available
if ! command -v cargo-deny &> /dev/null; then
    if [ -f "$HOME/.cargo/bin/cargo-deny" ]; then
        CARGO_DENY="$HOME/.cargo/bin/cargo-deny"
    else
        print_error "cargo-deny not found. Install with: cargo install cargo-deny"
        SCAN_FAILED=1
        CARGO_DENY=""
    fi
else
    CARGO_DENY="cargo-deny"
fi

if [ -n "$CARGO_DENY" ]; then
    echo "Checking for security advisories..."
    if $CARGO_DENY check advisories; then
        print_success "No security advisories found"
    else
        print_error "Security advisories found! Please review."
        SCAN_FAILED=1
    fi

    echo ""
    echo "Checking licenses..."
    if $CARGO_DENY check licenses 2>&1 | grep -q "error"; then
        print_warning "License check has warnings (review but not critical for security)"
    else
        print_success "License check passed"
    fi

    echo ""
    echo "Checking for banned/duplicated crates..."
    if $CARGO_DENY check bans 2>/dev/null; then
        print_success "No banned or problematic crates found"
    else
        print_warning "Some crate version conflicts detected (review recommended)"
    fi
fi

# ====================================
# 3. Node/Frontend Security Audit
# ====================================
print_header "3. Next.js UI Dashboard Security Audit"

cd "$PROJECT_ROOT/nextjs-ui-dashboard"

if ! command -v npm &> /dev/null; then
    print_error "npm not found. Please install Node.js and npm."
    SCAN_FAILED=1
else
    echo "Running npm audit..."
    AUDIT_OUTPUT=$(npm audit --json 2>&1)

    # Parse JSON output
    if echo "$AUDIT_OUTPUT" | jq -e '.metadata.vulnerabilities.total == 0' > /dev/null 2>&1; then
        print_success "No vulnerabilities found in npm packages"
    else
        # Extract vulnerability counts
        CRITICAL=$(echo "$AUDIT_OUTPUT" | jq -r '.metadata.vulnerabilities.critical // 0' 2>/dev/null || echo "0")
        HIGH=$(echo "$AUDIT_OUTPUT" | jq -r '.metadata.vulnerabilities.high // 0' 2>/dev/null || echo "0")
        MODERATE=$(echo "$AUDIT_OUTPUT" | jq -r '.metadata.vulnerabilities.moderate // 0' 2>/dev/null || echo "0")
        LOW=$(echo "$AUDIT_OUTPUT" | jq -r '.metadata.vulnerabilities.low // 0' 2>/dev/null || echo "0")

        TOTAL_NPM=$((CRITICAL + HIGH + MODERATE + LOW))
        TOTAL_VULNS=$((TOTAL_VULNS + TOTAL_NPM))

        if [ "$TOTAL_NPM" -gt 0 ]; then
            print_error "Found $TOTAL_NPM npm vulnerabilities:"
            [ "$CRITICAL" -gt 0 ] && print_error "  - Critical: $CRITICAL"
            [ "$HIGH" -gt 0 ] && print_error "  - High: $HIGH"
            [ "$MODERATE" -gt 0 ] && print_warning "  - Moderate: $MODERATE"
            [ "$LOW" -gt 0 ] && print_warning "  - Low: $LOW"
            echo ""
            echo "Run 'npm audit fix' to attempt automatic fixes"
        else
            print_success "No vulnerabilities found"
        fi
    fi
fi

# ====================================
# 4. Docker Security (if applicable)
# ====================================
print_header "4. Docker Security Check"

cd "$PROJECT_ROOT"

if command -v docker &> /dev/null; then
    # Check for running containers
    if docker ps -q > /dev/null 2>&1; then
        print_success "Docker is available"

        # Check if docker-compose files exist
        if [ -f "docker-compose.yml" ]; then
            echo "Found docker-compose.yml"

            # Check for hardcoded secrets (basic check)
            if grep -qiE "(password|secret|token|key).*=.*['\"].*['\"]" docker-compose.yml; then
                print_warning "Potential hardcoded secrets found in docker-compose.yml"
                print_warning "Review and use environment variables instead"
            else
                print_success "No obvious hardcoded secrets in docker-compose.yml"
            fi
        fi
    else
        print_warning "Docker is installed but not running"
    fi
else
    print_warning "Docker not found (optional)"
fi

# ====================================
# 5. Environment File Check
# ====================================
print_header "5. Environment Configuration Security"

cd "$PROJECT_ROOT"

# Check for .env files
if [ -f ".env" ]; then
    print_warning ".env file exists - ensure it's in .gitignore"

    # Check if .env is in .gitignore
    if [ -f ".gitignore" ] && grep -q "\.env" .gitignore; then
        print_success ".env is properly ignored by git"
    else
        print_error ".env is NOT in .gitignore! This is a security risk!"
        SCAN_FAILED=1
    fi
else
    print_warning "No .env file found (may be expected if using env vars)"
fi

# Check for example env files
if [ -f ".env.example" ] || [ -f "config.env" ]; then
    print_success "Found example environment file"
fi

# ====================================
# Summary
# ====================================
print_header "Security Scan Summary"

echo ""
echo "Total vulnerabilities found: $TOTAL_VULNS"
echo ""

if [ $TOTAL_VULNS -eq 0 ] && [ $SCAN_FAILED -eq 0 ]; then
    print_success "All security scans passed!"
    echo ""
    echo -e "${GREEN}Security Status: EXCELLENT (10/10)${NC}"
    exit 0
elif [ $TOTAL_VULNS -lt 5 ] && [ $SCAN_FAILED -eq 0 ]; then
    print_warning "Minor issues found - review recommended"
    echo ""
    echo -e "${YELLOW}Security Status: GOOD (8/10)${NC}"
    exit 0
else
    print_error "Security issues detected - immediate action required!"
    echo ""
    echo -e "${RED}Security Status: NEEDS ATTENTION (6/10)${NC}"
    echo ""
    echo "Next steps:"
    echo "1. Review vulnerabilities listed above"
    echo "2. Update dependencies to patched versions"
    echo "3. Run security scan again to verify fixes"
    exit 1
fi
