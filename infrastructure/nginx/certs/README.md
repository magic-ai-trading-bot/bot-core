# SSL/TLS Certificates Directory

## Purpose

This directory contains SSL/TLS certificates for bot-core nginx reverse proxy.

## Directory Contents

```
certs/
├── dev/
│   ├── cert.pem          # Self-signed certificate (development)
│   └── key.pem           # Private key (development)
├── prod/
│   ├── fullchain.pem     # Let's Encrypt certificate chain
│   ├── privkey.pem       # Let's Encrypt private key
│   └── dhparam.pem       # Diffie-Hellman parameters (2048-bit)
└── .gitkeep
```

## Security Warnings

**NEVER commit certificates or private keys to git!**

All `.pem` files are ignored by `.gitignore`.

## Certificate Types

### Development (Self-Signed)

- **Location**: `dev/`
- **Validity**: 365 days
- **Algorithm**: RSA 2048-bit
- **Subject**: `CN=bot-core.local`
- **Generation**: `scripts/generate-ssl-certs.sh --dev`

### Production (Let's Encrypt)

- **Location**: `prod/`
- **Validity**: 90 days (auto-renewed)
- **Algorithm**: ECDSA P-256 or RSA 2048-bit
- **Subject**: `CN=api.botcore.com` (or your domain)
- **Generation**: `scripts/setup-letsencrypt.sh`

## File Permissions

Ensure strict permissions:

```bash
# Certificates (public) - readable by all
chmod 644 certs/dev/cert.pem
chmod 644 certs/prod/fullchain.pem

# Private keys - readable only by owner
chmod 600 certs/dev/key.pem
chmod 600 certs/prod/privkey.pem
chmod 600 certs/prod/dhparam.pem
```

## Certificate Validation

Test certificates:

```bash
# Check certificate details
openssl x509 -in certs/dev/cert.pem -text -noout

# Verify certificate matches private key
openssl x509 -noout -modulus -in certs/dev/cert.pem | openssl md5
openssl rsa -noout -modulus -in certs/dev/key.pem | openssl md5
# Hashes should match

# Test HTTPS connection
curl -k https://localhost:443
```

## Certificate Renewal

### Development

Self-signed certificates expire after 365 days.

Regenerate:
```bash
./scripts/generate-ssl-certs.sh --dev --force
```

### Production

Let's Encrypt certificates auto-renew via cron job:

```bash
# Check renewal status
certbot certificates

# Manual renewal
certbot renew

# Test renewal (dry-run)
certbot renew --dry-run
```

## Troubleshooting

### Error: "SSL certificate problem: self signed certificate"

**Cause**: Using self-signed certificate in development

**Solution**: Use `-k` flag with curl or add certificate to system trust store

```bash
# macOS
sudo security add-trusted-cert -d -r trustRoot -k /Library/Keychains/System.keychain certs/dev/cert.pem

# Linux
sudo cp certs/dev/cert.pem /usr/local/share/ca-certificates/bot-core.crt
sudo update-ca-certificates
```

### Error: "Permission denied" when nginx starts

**Cause**: Incorrect file permissions

**Solution**: Fix permissions

```bash
chmod 600 certs/*/key.pem certs/*/privkey.pem
chmod 644 certs/*/cert.pem certs/*/fullchain.pem
```

### Error: "Certificate and private key do not match"

**Cause**: Mismatched certificate/key pair

**Solution**: Verify modulus match (see Certificate Validation section)

## References

- [Let's Encrypt](https://letsencrypt.org/)
- [OpenSSL Documentation](https://www.openssl.org/docs/)
- [Mozilla SSL Configuration Generator](https://ssl-config.mozilla.org/)
