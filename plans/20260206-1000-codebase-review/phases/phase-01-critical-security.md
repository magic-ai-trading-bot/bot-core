# Phase 01: Critical Security Fixes

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: None (First Priority)
**Blocks**: All other phases

---

## Overview

| Field | Value |
|-------|-------|
| Date | 2026-02-06 |
| Priority | P0-CRITICAL |
| Status | Pending |
| Effort | Medium (2-3 days) |
| Risk | CRITICAL - Finance project, secrets exposed |

---

## Key Insights (From Reports)

**Source**: `reports/05-infrastructure-review.md`

1. **Hardcoded API keys** in `rust-core-engine/config.toml` - Binance credentials committed to git
2. **Docker containers running as root** - All 3 production Dockerfiles missing USER directive
3. **sshpass usage** in CI/CD - Password-based SSH deployment vulnerable to MITM
4. **Default passwords** in docker-compose - 8 instances of weak defaults
5. **CORS wildcard** in production config - API accepts all origins
6. **MongoDB connection string** with hardcoded password in default value

---

## Requirements

### CRITICAL-01: Remove Hardcoded API Keys
- **File**: `rust-core-engine/config.toml:6-7`
- **Issue**: Binance API keys committed to git
- **Fix**: Move to environment variables, add config.toml to .gitignore
- **Ref**: Infrastructure Review Issue #1

### CRITICAL-02: Add Non-Root User to Dockerfiles
- **Files**:
  - `rust-core-engine/Dockerfile.production:31-57`
  - `python-ai-service/Dockerfile.production:1-43`
  - `nextjs-ui-dashboard/Dockerfile.production:19-35`
- **Issue**: Containers run as UID 0
- **Fix**: Add USER directive with non-root user (UID 1001)
- **Ref**: Infrastructure Review Issue #2

### CRITICAL-03: Replace sshpass with SSH Keys
- **File**: `.github/workflows/deploy-vps.yml:51-52, 369`
- **Issue**: Password-based SSH authentication
- **Fix**: Use SSH key authentication with known_hosts verification
- **Ref**: Infrastructure Review Issue #3

### HIGH-04: Remove Default Passwords
- **File**: `infrastructure/docker/docker-compose.yml`
- **Lines**: 16, 19, 113, 116, 120, 161, 306, 352
- **Issue**: Weak defaults used if env not set
- **Fix**: Use fail-safe mode (require explicit setting)
- **Ref**: Infrastructure Review Issue #4

### HIGH-05: Fix CORS Wildcard
- **File**: `rust-core-engine/config.toml:97`
- **Issue**: `cors_origins = ["*"]`
- **Fix**: Configure specific allowed origins
- **Ref**: Infrastructure Review Issue #6

### HIGH-06: Secure MongoDB Connection String
- **File**: `infrastructure/docker/docker-compose.yml:19,113,161,430,483,530`
- **Issue**: Password in default connection string
- **Fix**: Use separate env vars, build URL at runtime
- **Ref**: Infrastructure Review Issue #5

---

## Related Code Files

```
rust-core-engine/
├── config.toml                     # Hardcoded keys (REMOVE)
├── config.example.toml             # Create with placeholders
├── Dockerfile.production           # Add USER directive
├── .gitignore                      # Add config.toml

python-ai-service/
├── Dockerfile.production           # Add USER directive

nextjs-ui-dashboard/
├── Dockerfile.production           # Add USER directive

infrastructure/docker/
├── docker-compose.yml              # Fix defaults, MongoDB URL

.github/workflows/
├── deploy-vps.yml                  # Replace sshpass with SSH keys
```

---

## Implementation Steps

### Step 1: Rotate Compromised Credentials (IMMEDIATE)
```bash
# 1. Generate new Binance API keys in Binance dashboard
# 2. Revoke old keys immediately
# 3. Update .env with new keys
# 4. Never commit keys to git again
```

### Step 2: Fix config.toml
```bash
# Create example config
cp rust-core-engine/config.toml rust-core-engine/config.example.toml
# Edit example to use placeholders
sed -i '' 's/api_key = ".*"/api_key = "${BINANCE_API_KEY}"/' rust-core-engine/config.example.toml
sed -i '' 's/secret_key = ".*"/secret_key = "${BINANCE_SECRET_KEY}"/' rust-core-engine/config.example.toml
# Add to gitignore
echo "config.toml" >> rust-core-engine/.gitignore
# Remove from git tracking
git rm --cached rust-core-engine/config.toml
```

### Step 3: Add USER to Dockerfiles
```dockerfile
# Add to each Dockerfile.production after base image setup:
RUN addgroup -g 1001 -S appuser && \
    adduser -u 1001 -S appuser -G appuser
# Before CMD:
USER appuser
```

### Step 4: Replace sshpass in CI/CD
```yaml
# In deploy-vps.yml, replace:
- name: Setup SSH key
  run: |
    mkdir -p ~/.ssh
    echo "${{ secrets.VPS_SSH_PRIVATE_KEY }}" > ~/.ssh/deploy_key
    chmod 600 ~/.ssh/deploy_key
    ssh-keyscan -H ${{ secrets.VPS_HOST }} >> ~/.ssh/known_hosts

- name: Deploy
  run: |
    ssh -i ~/.ssh/deploy_key ${{ secrets.VPS_USER }}@${{ secrets.VPS_HOST }} 'bash -s' < deploy.sh
```

### Step 5: Fix Default Passwords
```yaml
# In docker-compose.yml, change:
# FROM: INTER_SERVICE_TOKEN=${INTER_SERVICE_TOKEN:-default_token}
# TO:   INTER_SERVICE_TOKEN=${INTER_SERVICE_TOKEN:?INTER_SERVICE_TOKEN not set}
```

### Step 6: Fix CORS Configuration
```toml
# In config.toml:
[api]
cors_origins = ["https://dashboard.example.com"]
# Or load from CORS_ALLOWED_ORIGINS env var
```

---

## Todo List

- [ ] Rotate Binance API keys immediately
- [ ] Create config.example.toml with placeholders
- [ ] Add config.toml to .gitignore
- [ ] Remove config.toml from git history (git filter-branch or BFG)
- [ ] Add USER directive to rust-core-engine/Dockerfile.production
- [ ] Add USER directive to python-ai-service/Dockerfile.production
- [ ] Add USER directive to nextjs-ui-dashboard/Dockerfile.production
- [ ] Generate SSH key pair for CI/CD deployment
- [ ] Replace sshpass with SSH key auth in deploy-vps.yml
- [ ] Change all default passwords to fail-safe mode
- [ ] Fix CORS to use specific origins
- [ ] Update MongoDB connection string pattern
- [ ] Run security scan to verify fixes
- [ ] Update documentation with new setup process

---

## Success Criteria

| Criteria | Metric | Target |
|----------|--------|--------|
| No hardcoded secrets | grep for API keys | 0 matches |
| Non-root containers | docker inspect USER | UID 1001 |
| SSH key deployment | workflow logs | No sshpass |
| No default passwords | grep ":-default" | 0 matches |
| CORS restricted | config check | Specific origins |
| Security scan | Trivy + GitLeaks | 0 critical findings |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Key rotation breaks production | Medium | High | Test on staging first |
| Non-root breaks container | Low | Medium | Test all services |
| SSH key setup fails CI | Low | Medium | Keep password fallback temporarily |

---

## Security Considerations

- **Git History**: Consider using BFG Repo-Cleaner to remove secrets from history
- **Key Rotation**: New keys should be generated BEFORE pushing fixes
- **Access Audit**: Review who has access to current secrets
- **Documentation**: Update setup docs to reflect new secret management

---

## Estimated Completion

- **Critical fixes (keys, Docker)**: 4 hours
- **CI/CD changes**: 2 hours
- **Testing & validation**: 2 hours
- **Documentation updates**: 1 hour

**Total**: 9-10 hours (1.5 days)
