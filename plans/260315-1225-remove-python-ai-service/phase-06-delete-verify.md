# Phase 6: Delete python-ai-service/ & Final Verification

**Priority**: P0 | **Effort**: 30min | **Status**: Pending

## Steps

### 6a. Delete the directory
```bash
rm -rf python-ai-service/
```

### 6b. Full verification
```bash
# Rust
cd rust-core-engine && cargo check && cargo test && cargo clippy -- -D warnings

# Frontend
cd nextjs-ui-dashboard && npm run type-check && npm test

# MCP
cd mcp-server && npx tsc --noEmit

# Docker
docker compose -f docker-compose-vps.yml config --quiet

# Search for orphan references
grep -r "python-ai-service\|python_ai_service\|port 8000\|:8000\|PYTHON_AI" . \
  --include="*.rs" --include="*.ts" --include="*.tsx" --include="*.yml" --include="*.yaml" \
  --include="*.toml" --include="*.sh" --include="*.md" \
  --exclude-dir=plans --exclude-dir=.git --exclude-dir=node_modules
```

### 6c. Verify VPS resource estimate
- Expected: 6 services (MongoDB, Rust, Frontend, MCP, OpenClaw, Redis)
- Expected RAM: ~3.5-4GB (fits 8GB VPS comfortably)

## Todo

- [ ] Delete python-ai-service/ directory
- [ ] Rust compiles and tests pass
- [ ] Frontend compiles and tests pass
- [ ] MCP compiles
- [ ] Docker compose validates
- [ ] No orphan Python references
- [ ] Commit and push
