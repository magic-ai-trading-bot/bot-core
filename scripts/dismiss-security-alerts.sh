#!/bin/bash
# =============================================================================
# Dismiss GitHub Security Alerts
# =============================================================================
# This script helps dismiss security alerts for known test/revoked credentials
#
# Prerequisites:
# - GitHub CLI (gh) installed and authenticated
# - Admin access to the repository
#
# Usage:
#   ./scripts/dismiss-security-alerts.sh
# =============================================================================

set -e

REPO="magic-ai-trading-bot/bot-core"

echo "=============================================="
echo "GitHub Security Alerts Dismissal Script"
echo "=============================================="
echo ""

# Check if gh is installed
if ! command -v gh &> /dev/null; then
    echo "Error: GitHub CLI (gh) is not installed."
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "Error: Not authenticated with GitHub CLI."
    echo "Run: gh auth login"
    exit 1
fi

echo "Repository: $REPO"
echo ""

# =============================================================================
# SECRET SCANNING ALERTS (20 alerts)
# =============================================================================
echo "=== Secret Scanning Alerts ==="
echo ""
echo "The following secrets were found in git history:"
echo ""
echo "1. Binance API Key: iiZAQULhnkkfDiueUWavpVXzePSi1WjKlJwiB3k72EZTif2k4BcWuCC8FNqo1R1F"
echo "   - Status: TESTNET key (safe, but should be revoked)"
echo "   - Location: config.env (commit f328684, now deleted)"
echo ""
echo "2. Binance Secret Key: oJNiTwYTh3oc2iPz5oXg2Phqoa7MhhV2IO9llyezVkh3pHtCYiC2v4Uym1kcAriK"
echo "   - Status: TESTNET key (safe, but should be revoked)"
echo "   - Location: config.env (commit f328684, now deleted)"
echo ""
echo "ACTION REQUIRED:"
echo "1. Go to Binance Testnet: https://testnet.binance.vision/"
echo "2. Revoke the old API keys"
echo "3. Generate new API keys and store them in .env (not committed to git)"
echo ""

# Try to list secret scanning alerts
echo "Listing secret scanning alerts..."
gh api repos/$REPO/secret-scanning/alerts --jq '.[] | {number, state, secret_type}' 2>/dev/null || echo "Unable to fetch secret scanning alerts (may require admin access)"
echo ""

# =============================================================================
# CODE SCANNING ALERTS (175 alerts)
# =============================================================================
echo "=== Code Scanning Alerts ==="
echo ""
echo "Code scanning alerts are typically from:"
echo "- CodeQL analysis"
echo "- Semgrep security scans"
echo "- Trivy vulnerability scans"
echo ""
echo "Common patterns detected:"
echo "- console.log statements (development/debug code)"
echo "- Unsafe Rust .unwrap() calls"
echo "- Test credentials in test files"
echo ""
echo "Listing code scanning alerts..."
gh api repos/$REPO/code-scanning/alerts --jq '.[] | select(.state=="open") | {number, rule: .rule.id, severity: .rule.security_severity_level, file: .most_recent_instance.location.path}' 2>/dev/null | head -50 || echo "Unable to fetch code scanning alerts (may require admin access)"
echo ""

# =============================================================================
# DEPENDABOT ALERTS (1 alert)
# =============================================================================
echo "=== Dependabot Alerts ==="
echo ""
echo "Listing dependabot alerts..."
gh api repos/$REPO/dependabot/alerts --jq '.[] | {number, state, package: .security_vulnerability.package.name, severity: .security_vulnerability.severity}' 2>/dev/null || echo "Unable to fetch dependabot alerts (may require admin access)"
echo ""

# =============================================================================
# MANUAL STEPS
# =============================================================================
echo "=============================================="
echo "MANUAL STEPS REQUIRED"
echo "=============================================="
echo ""
echo "1. SECRET SCANNING ALERTS:"
echo "   - Go to: https://github.com/$REPO/security/secret-scanning"
echo "   - For each alert, click 'Close as' -> 'Revoked' or 'Test'"
echo ""
echo "2. CODE SCANNING ALERTS:"
echo "   - Go to: https://github.com/$REPO/security/code-scanning"
echo "   - Review each alert and dismiss false positives"
echo "   - For test file alerts: dismiss as 'Test'"
echo "   - For development code: dismiss as 'Won't fix'"
echo ""
echo "3. DEPENDABOT ALERTS:"
echo "   - Go to: https://github.com/$REPO/security/dependabot"
echo "   - Review and apply recommended fixes"
echo ""
echo "4. REVOKE COMPROMISED KEYS:"
echo "   - Binance Testnet: https://testnet.binance.vision/"
echo "   - Generate new keys and update .env file"
echo ""
echo "=============================================="
echo "Script completed. Please complete manual steps."
echo "=============================================="
