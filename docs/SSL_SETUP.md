# SSL/TLS Setup Guide for Bot-Core

**Document Version:** 1.0.0
**Last Updated:** 2025-11-18
**Status:** Active
**Owner:** Security Team

---

## Table of Contents

- [Overview](#overview)
- [Prerequisites](#prerequisites)
- [Development Setup (Self-Signed)](#development-setup-self-signed)
- [Production Setup (Let's Encrypt)](#production-setup-lets-encrypt)
- [Certificate Renewal](#certificate-renewal)
- [Security Headers](#security-headers)
- [Troubleshooting](#troubleshooting)
- [Security Best Practices](#security-best-practices)

---

## Overview

Bot-Core uses nginx as a reverse proxy with SSL/TLS termination for secure HTTPS communication.

**Architecture:**
```
Internet → nginx (Port 443, TLS 1.3) → Internal Services
          ↓
      Frontend :3000
      Rust API :8080
      Python AI :8000
```

**TLS Configuration:**
- **Protocol**: TLS 1.3 (TLS 1.2 minimum)
- **Cipher Suites**: Modern, strong ciphers only
- **Certificates**:
  - Development: Self-signed (RSA 2048-bit or ECDSA P-256)
  - Production: Let's Encrypt (free, automated renewal)

---

## Prerequisites

### Required Software

**Development:**
- OpenSSL 1.1.1+ (check: `openssl version`)
- Docker 24.0+ and Docker Compose 2.20+

**Production:**
- certbot (Let's Encrypt client)
- DNS A record pointing to your server
- Port 80 and 443 accessible from internet

### Installation

**macOS:**
```bash
brew install openssl certbot
```

**Ubuntu/Debian:**
```bash
sudo apt-get update
sudo apt-get install openssl certbot
```

**CentOS/RHEL:**
```bash
sudo yum install openssl certbot
```

---

## Development Setup (Self-Signed)

### Quick Start

Generate self-signed certificate for development:

```bash
cd /path/to/bot-core
./scripts/generate-ssl-certs.sh --dev
```

**Output:**
```
[INFO] Bot-Core SSL Certificate Generator
[INFO] Mode: dev
[INFO] Generating RSA 2048-bit self-signed certificate...
[SUCCESS] Development certificates generated successfully!
[INFO] Certificate: infrastructure/nginx/certs/dev/cert.pem
[INFO] Private Key: infrastructure/nginx/certs/dev/key.pem
[INFO] Valid for: 365 days
[INFO] Domain: bot-core.local
```

### Start Services

```bash
# Start with docker-compose
docker-compose -f infrastructure/docker/docker-compose.yml \
               -f infrastructure/docker/docker-compose.prod.yml \
               up -d

# Or use bot.sh script
./scripts/bot.sh start --memory-optimized
```

### Verify HTTPS

```bash
# Test HTTPS connection (ignore self-signed warning)
curl -k https://localhost/health

# Check certificate details
openssl s_client -connect localhost:443 -showcerts < /dev/null
```

### Trust Self-Signed Certificate

**macOS:**
```bash
sudo security add-trusted-cert -d -r trustRoot \
  -k /Library/Keychains/System.keychain \
  infrastructure/nginx/certs/dev/cert.pem
```

**Linux:**
```bash
sudo cp infrastructure/nginx/certs/dev/cert.pem \
  /usr/local/share/ca-certificates/bot-core.crt
sudo update-ca-certificates
```

**Windows:**
```powershell
# Import certificate to Trusted Root Certification Authorities
certutil -addstore "Root" infrastructure\nginx\certs\dev\cert.pem
```

### Advanced Options

**Generate ECDSA Certificate (faster, smaller):**
```bash
./scripts/generate-ssl-certs.sh --dev --ecdsa
```

**Custom Domain:**
```bash
./scripts/generate-ssl-certs.sh --dev --domain localhost
```

**Longer Validity (2 years):**
```bash
./scripts/generate-ssl-certs.sh --dev --days 730
```

**Force Regeneration:**
```bash
./scripts/generate-ssl-certs.sh --dev --force
```

---

## Production Setup (Let's Encrypt)

### Prerequisites

1. **Domain Name**: Own a domain (e.g., api.botcore.com)
2. **DNS Configuration**: A record pointing to your server IP
3. **Firewall**: Port 80 and 443 open
4. **Server**: Public IP address accessible from internet

### DNS Setup

Create A record:
```
Type: A
Name: api (or @)
Value: YOUR_SERVER_IP
TTL: 3600
```

Verify:
```bash
nslookup api.botcore.com
# Should return your server IP
```

### Certificate Request

**1. Generate CSR (Optional):**
```bash
./scripts/generate-ssl-certs.sh --prod --domain api.botcore.com
```

**2. Request Let's Encrypt Certificate:**
```bash
./scripts/setup-letsencrypt.sh \
  --domain api.botcore.com \
  --email admin@botcore.com
```

**Output:**
```
[INFO] Bot-Core Let's Encrypt Setup
[INFO] Domain: api.botcore.com
[INFO] Email: admin@botcore.com

[INFO] Checking DNS resolution...
[SUCCESS] DNS resolved: api.botcore.com → 203.0.113.42

[INFO] Setting up webroot for ACME challenge...
[SUCCESS] Webroot ready: /var/www/certbot

[INFO] Requesting Let's Encrypt certificate...
[INFO] Running certbot...
[SUCCESS] Certificate obtained successfully!

[INFO] Copying certificates to project directory...
[SUCCESS] Certificates copied to: infrastructure/nginx/certs/prod

[INFO] Setting up auto-renewal...
[SUCCESS] Cron job added: Daily renewal check at 2 AM

[INFO] Reloading nginx configuration...
[SUCCESS] Nginx reloaded successfully

[SUCCESS] Let's Encrypt setup complete!
[INFO] Certificates will auto-renew every 60 days
```

### Test Before Production

Use Let's Encrypt **staging** server to avoid rate limits:

```bash
./scripts/setup-letsencrypt.sh \
  --domain api.botcore.com \
  --email admin@botcore.com \
  --staging
```

Certificates from staging are NOT trusted by browsers (for testing only).

### Dry Run

Validate configuration without saving certificates:

```bash
./scripts/setup-letsencrypt.sh \
  --domain api.botcore.com \
  --email admin@botcore.com \
  --dry-run
```

### Update docker-compose

Edit `infrastructure/docker/docker-compose.prod.yml`:

```yaml
nginx:
  volumes:
    # Comment out dev certificates
    # - ../nginx/certs/dev:/etc/nginx/ssl:ro

    # Enable production certificates
    - ../nginx/certs/prod:/etc/nginx/ssl:ro
```

Restart nginx:
```bash
docker-compose restart nginx
```

---

## Certificate Renewal

### Auto-Renewal (Recommended)

Let's Encrypt certificates expire after **90 days**.

Auto-renewal is setup via cron job during `setup-letsencrypt.sh`.

**Cron Schedule:**
```
0 2 * * * /path/to/scripts/renew-ssl.sh >> /var/log/certbot-renew.log 2>&1
```

### Manual Renewal

**Check Certificate Status:**
```bash
sudo certbot certificates
```

**Output:**
```
Found the following certs:
  Certificate Name: api.botcore.com
    Domains: api.botcore.com
    Expiry Date: 2025-02-15 12:00:00+00:00 (VALID: 89 days)
    Certificate Path: /etc/letsencrypt/live/api.botcore.com/fullchain.pem
    Private Key Path: /etc/letsencrypt/live/api.botcore.com/privkey.pem
```

**Renew Manually:**
```bash
sudo certbot renew
```

**Test Renewal (Dry Run):**
```bash
sudo certbot renew --dry-run
```

### Renewal Script

The renewal script (`scripts/renew-ssl.sh`) performs:
1. Check certificate expiry
2. Attempt renewal (if within 30 days of expiry)
3. Copy renewed certificates to project directory
4. Reload nginx
5. Send alerts (if configured)
6. Cleanup old logs

**Run Manually:**
```bash
sudo ./scripts/renew-ssl.sh
```

### Troubleshooting Renewal

**Check Logs:**
```bash
sudo cat /var/log/letsencrypt/letsencrypt.log
sudo cat /var/log/certbot-renew.log
```

**Common Issues:**

**1. Port 80 not accessible**
```
Solution: Ensure firewall allows port 80
sudo ufw allow 80/tcp
```

**2. Webroot not writable**
```
Solution: Fix permissions
sudo chown -R www-data:www-data /var/www/certbot
```

**3. Rate limit exceeded**
```
Solution: Wait 1 week or use staging server
./scripts/setup-letsencrypt.sh --staging ...
```

---

## Security Headers

Bot-Core nginx is configured with strict security headers.

### Enabled Headers

**HSTS (HTTP Strict Transport Security):**
```
Strict-Transport-Security: max-age=31536000; includeSubDomains; preload
```
Forces HTTPS for 1 year. Submitted to browser preload list.

**X-Frame-Options:**
```
X-Frame-Options: DENY
```
Prevents clickjacking attacks.

**X-Content-Type-Options:**
```
X-Content-Type-Options: nosniff
```
Prevents MIME sniffing.

**X-XSS-Protection:**
```
X-XSS-Protection: 1; mode=block
```
Enables browser XSS filter (legacy browsers).

**Referrer-Policy:**
```
Referrer-Policy: strict-origin-when-cross-origin
```
Only sends origin when navigating cross-origin.

**Permissions-Policy:**
```
Permissions-Policy: geolocation=(), microphone=(), camera=(), payment=()
```
Disables unnecessary browser features.

**Content-Security-Policy (CSP):**
```
Content-Security-Policy: default-src 'self'; script-src 'self' 'unsafe-inline' https://cdn.jsdelivr.net; ...
```
Defines allowed content sources.

### Verify Security Headers

**Using curl:**
```bash
curl -I https://api.botcore.com | grep -E "(Strict|Frame|Content|XSS|Referrer|Permissions)"
```

**Using online tools:**
- [SecurityHeaders.com](https://securityheaders.com/)
- [Mozilla Observatory](https://observatory.mozilla.org/)

### Test Results

Expected grade: **A+** on SecurityHeaders.com

---

## Troubleshooting

### Common Issues

#### 1. "SSL certificate problem: self signed certificate"

**Cause:** Using self-signed certificate in development

**Solution:**
```bash
# Ignore warning (development only)
curl -k https://localhost/health

# Or trust certificate (see Trust Self-Signed Certificate section)
```

#### 2. "nginx: [emerg] cannot load certificate"

**Cause:** Certificate file not found or wrong path

**Solution:**
```bash
# Check file exists
ls -l infrastructure/nginx/certs/dev/cert.pem

# Check docker volume mount
docker exec bot-nginx ls -l /etc/nginx/ssl/

# Fix path in docker-compose.prod.yml
```

#### 3. "Certificate and private key do not match"

**Cause:** Mismatched certificate/key pair

**Solution:**
```bash
# Verify modulus match
openssl x509 -noout -modulus -in cert.pem | openssl md5
openssl rsa -noout -modulus -in key.pem | openssl md5
# Hashes should be identical

# Regenerate if needed
./scripts/generate-ssl-certs.sh --dev --force
```

#### 4. "Port 443 already in use"

**Cause:** Another process using port 443

**Solution:**
```bash
# Find process
sudo lsof -i :443

# Stop conflicting service
sudo systemctl stop apache2  # or other service
```

#### 5. "Let's Encrypt rate limit exceeded"

**Cause:** Too many certificate requests

**Limits:**
- 50 certificates per domain per week
- 5 duplicate certificates per week

**Solution:**
```bash
# Use staging server for testing
./scripts/setup-letsencrypt.sh --staging ...

# Wait 1 week for rate limit reset
```

### Debug Mode

**Test nginx configuration:**
```bash
# Outside Docker
nginx -t

# Inside Docker
docker exec bot-nginx nginx -t
```

**View nginx error logs:**
```bash
docker logs bot-nginx
docker logs bot-nginx --tail 100 -f  # Follow last 100 lines
```

**Check certificate expiry:**
```bash
echo | openssl s_client -connect localhost:443 2>/dev/null | \
  openssl x509 -noout -dates
```

---

## Security Best Practices

### DO

✅ **Use TLS 1.3** (or minimum TLS 1.2)
✅ **Auto-renew certificates** (before 30 days expiry)
✅ **Use HSTS** with preload
✅ **Disable weak ciphers** (no RC4, 3DES, MD5)
✅ **Enable OCSP stapling** (faster certificate validation)
✅ **Use strong DH parameters** (2048-bit minimum)
✅ **Monitor certificate expiry**
✅ **Test configurations** before deploying

### DON'T

❌ **Don't commit certificates** to git (.gitignore is configured)
❌ **Don't use SHA-1** certificates (deprecated)
❌ **Don't allow SSLv3 or TLS 1.0/1.1** (vulnerable)
❌ **Don't use weak ciphers** (RC4, DES, export ciphers)
❌ **Don't share private keys** (keep 600 permissions)
❌ **Don't forget to renew** (auto-renewal recommended)

### Monitoring

**Certificate Expiry Alerts:**

Add to monitoring system (Prometheus, Datadog, etc.):

```yaml
# prometheus alert
- alert: SSLCertificateExpiringSoon
  expr: (ssl_certificate_expiry_seconds - time()) / 86400 < 14
  annotations:
    summary: "SSL certificate expiring in {{ $value }} days"
```

**Check Manually:**
```bash
# Days until expiry
echo | openssl s_client -connect api.botcore.com:443 2>/dev/null | \
  openssl x509 -noout -checkend 1209600
# Exit code 0 = valid for 14+ days
```

### Security Audits

**Test TLS Configuration:**
- [SSL Labs](https://www.ssllabs.com/ssltest/) - Comprehensive TLS test
- [testssl.sh](https://testssl.sh/) - Command-line TLS testing

**Expected Grade:** A or A+ on SSL Labs

**Run testssl.sh:**
```bash
# Install
git clone --depth 1 https://github.com/drwetter/testssl.sh.git
cd testssl.sh

# Test
./testssl.sh https://api.botcore.com
```

---

## Quick Reference

### File Locations

```
infrastructure/nginx/
├── nginx.conf                    # Main nginx config
├── conf.d/
│   ├── default.conf              # Server blocks
│   ├── ssl.conf                  # SSL/TLS settings
│   ├── security-headers.conf     # Security headers
│   └── upstream.conf             # Backend services
├── certs/
│   ├── dev/                      # Development certificates
│   │   ├── cert.pem
│   │   └── key.pem
│   └── prod/                     # Production certificates
│       ├── fullchain.pem
│       ├── privkey.pem
│       ├── chain.pem
│       └── dhparam.pem
└── html/
    └── 50x.html                  # Error page

scripts/
├── generate-ssl-certs.sh         # Generate self-signed certs
├── setup-letsencrypt.sh          # Setup Let's Encrypt
└── renew-ssl.sh                  # Renew certificates (cron)
```

### Essential Commands

```bash
# Development
./scripts/generate-ssl-certs.sh --dev
docker-compose up -d nginx

# Production
./scripts/setup-letsencrypt.sh --domain api.botcore.com --email admin@botcore.com
docker-compose restart nginx

# Renewal
sudo certbot renew --dry-run     # Test
sudo certbot renew                # Actual renewal

# Testing
curl -k https://localhost/health
openssl s_client -connect localhost:443 -showcerts
nginx -t                          # Test config
```

### Support

**Documentation:**
- [nginx SSL Module](https://nginx.org/en/docs/http/ngx_http_ssl_module.html)
- [Let's Encrypt Documentation](https://letsencrypt.org/docs/)
- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)

**Security:**
- Report: security@botcore.com
- Incident Response: See `docs/SECURITY_AUDIT_REPORT.md`

---

**Last Updated:** 2025-11-18
**Document End**
