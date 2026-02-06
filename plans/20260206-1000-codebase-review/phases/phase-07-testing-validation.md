# Phase 07: Testing & Final Validation

**Parent Plan**: [plan.md](../plan.md)
**Dependencies**: All previous phases (01-06)
**Blocks**: Production deployment

---

## Overview

| Field | Value |
|-------|-------|
| Date | 2026-02-06 |
| Priority | P3-LOW |
| Status | Pending |
| Effort | Small (2-3 days) |
| Risk | LOW - Validation only, no code changes |

---

## Key Insights (From All Reports)

This phase consolidates testing requirements from all review reports to ensure fixes are properly validated before production deployment.

**Current Test Status**:
- Rust: 1,336 tests (passing)
- Python: 409 tests (passing)
- Frontend: 601 tests (passing)
- Total: 2,346+ tests

**Coverage**: 90.4% average

---

## Requirements

### VAL-01: Run All Test Suites After Fixes
- **Commands**:
  - `cargo test` (Rust)
  - `pytest` (Python)
  - `npm test` (Frontend)
- **Target**: 100% pass rate

### VAL-02: Run Security Scans
- **Tools**: Trivy, GitLeaks, CodeQL
- **Target**: 0 critical/high findings

### VAL-03: Run Spec Validation
- **Script**: `python3 scripts/validate-specs.py`
- **Target**: <5 orphan tags, PASS status

### VAL-04: Run Clippy (Rust Linting)
- **Command**: `cargo clippy --all-targets -- -D warnings`
- **Target**: 0 warnings

### VAL-05: Run MyPy (Python Type Checking)
- **Command**: `mypy python-ai-service --ignore-missing-imports`
- **Target**: 0 errors

### VAL-06: Run TypeScript Check (Frontend)
- **Command**: `npm run type-check`
- **Target**: 0 errors

### VAL-07: Integration Testing
- **Scope**: Cross-service communication
- **Tests**: WebSocket events, API contracts, Auth flow

### VAL-08: Performance Baseline
- **Tool**: Benchmark before/after refactoring
- **Target**: No regression (or document improvements)

### VAL-09: Documentation Update
- **Files**: README.md, CONTRIBUTING.md, setup guides
- **Target**: Reflect all changes from Phases 01-06

### VAL-10: Create Changelog
- **File**: CHANGELOG.md
- **Target**: Document all fixes with issue references

---

## Validation Checklist

### Security Validation (Phase 01)
- [ ] No hardcoded secrets in codebase (`grep -r "api_key"`)
- [ ] config.toml not in git (`git ls-files config.toml`)
- [ ] Docker containers run as non-root (`docker inspect --format '{{.Config.User}}'`)
- [ ] No sshpass in workflows (`grep -r "sshpass" .github/`)
- [ ] No default passwords in docker-compose (`grep -r ":-default"`)
- [ ] CORS restricted (`grep "cors_origins" config.toml`)
- [ ] Trivy scan passes (`trivy image`)
- [ ] GitLeaks scan passes (`gitleaks detect`)

### Rust Validation (Phase 02)
- [ ] `cargo test` passes (1,336+ tests)
- [ ] `cargo clippy --all-targets -- -D warnings` passes
- [ ] `grep -r "unwrap()" --include="*.rs"` < 10 production occurrences
- [ ] `grep -r "panic!" --include="*.rs"` only in #[cfg(test)] modules
- [ ] No files >3000 lines (`wc -l *.rs`)
- [ ] Key functions documented with examples

### Python Validation (Phase 03)
- [ ] `pytest` passes (409+ tests)
- [ ] `mypy` passes with 95%+ coverage
- [ ] `grep -r "except:" --include="*.py"` = 0 (bare except)
- [ ] `grep -r "global " main.py` = 0 (global variables)
- [ ] Background tasks have error callbacks
- [ ] ML models have hash verification

### Frontend Validation (Phase 04)
- [ ] `npm test` passes (601+ tests)
- [ ] `npm run type-check` passes
- [ ] `grep -r "console.log" src/` = 0 (use logger)
- [ ] AbortController in all fetch hooks
- [ ] useMemo in performance-critical components
- [ ] React DevTools profiler shows <50% re-renders

### Spec Validation (Phase 05)
- [ ] `python3 scripts/validate-specs.py` passes
- [ ] FR-REAL-TRADING.md exists with 14+ requirements
- [ ] FR-SETTINGS.md exists
- [ ] FR-PAPER-003 documented in FR-PAPER-TRADING.md
- [ ] TRACEABILITY_MATRIX.md has 0 broken refs
- [ ] @spec tag accuracy >= 95%

### Integration Validation (Phase 06)
- [ ] API-PYTHON-AI.md documents all endpoints
- [ ] DB-SCHEMA.md matches code (v2.1.0+)
- [ ] PositionUpdateData matches spec (15+ fields)
- [ ] Error format consistent (Rust == Python)
- [ ] Token refresh endpoint works
- [ ] WebSocket integration tests pass

---

## Test Commands

### Full Test Suite
```bash
# Run all tests in parallel
make test

# Or individually:
cd rust-core-engine && cargo test
cd python-ai-service && pytest -v
cd nextjs-ui-dashboard && npm test
```

### Security Scans
```bash
# Docker image scan
trivy image rust-core-engine:latest
trivy image python-ai-service:latest
trivy image nextjs-ui-dashboard:latest

# Secret scan
gitleaks detect --source . --verbose

# Dependency scan
cargo audit
pip-audit -r python-ai-service/requirements.txt
npm audit --prefix nextjs-ui-dashboard
```

### Static Analysis
```bash
# Rust
cargo clippy --all-targets --all-features -- -D warnings

# Python
black --check python-ai-service/
flake8 python-ai-service/
mypy python-ai-service/

# Frontend
npm run lint --prefix nextjs-ui-dashboard
npm run type-check --prefix nextjs-ui-dashboard
```

### Spec Validation
```bash
python3 scripts/validate-specs.py
python3 scripts/validate-spec-tags.py
```

---

## Performance Benchmarks

### Before Fixes (Baseline)
| Operation | Latency | Throughput |
|-----------|---------|------------|
| Trade execution | ~10ms | - |
| API response | ~100ms | - |
| WebSocket event | ~50ms | - |
| ML inference | ~500ms | - |

### After Fixes (Target)
| Operation | Latency | Throughput | Status |
|-----------|---------|------------|--------|
| Trade execution | <15ms | - | Verify |
| API response | <150ms | - | Verify |
| WebSocket event | <75ms | - | Verify |
| ML inference | <750ms | - | Verify |

---

## Documentation Updates

### Files to Update
- [ ] README.md - Setup instructions, quick start
- [ ] CONTRIBUTING.md - New code standards
- [ ] docs/TROUBLESHOOTING.md - New issues/solutions
- [ ] docs/features/*.md - Feature changes
- [ ] CLAUDE.md - Agent instructions if needed

### Changelog Entry Template
```markdown
## [Unreleased] - 2026-02-XX

### Security
- Removed hardcoded API keys from config.toml (#01-1)
- Added non-root user to Docker containers (#01-2)
- Replaced sshpass with SSH key authentication (#01-3)

### Fixed
- Mutex poisoning in WebSocket handler (#02-1)
- Bare except clauses in Python monitoring (#03-1)
- Missing AbortController in useAIAnalysis (#04-1)

### Changed
- Refactored paper_trading/engine.rs into modules (#02-4)
- Migrated global state to AppState class (#03-4)
- Updated PositionUpdateData interface (#06-3)

### Added
- FR-REAL-TRADING.md specification (#05-1)
- Token refresh endpoint (#06-4)
- Rate limiting to API routes (#02-9)

### Documentation
- Fixed 76 broken refs in TRACEABILITY_MATRIX.md (#05-5)
- Documented 8 undocumented Python endpoints (#06-1)
- Updated DB-SCHEMA.md to v2.1.0 (#06-2)
```

---

## Todo List

### Pre-Validation
- [ ] Ensure all Phase 01-06 fixes are complete
- [ ] Commit all changes to feature branch
- [ ] Create comprehensive PR description

### Test Execution
- [ ] Run Rust tests (`cargo test`)
- [ ] Run Python tests (`pytest`)
- [ ] Run Frontend tests (`npm test`)
- [ ] Run integration tests
- [ ] Run security scans (Trivy, GitLeaks)
- [ ] Run static analysis (Clippy, MyPy, ESLint)
- [ ] Run spec validation

### Documentation
- [ ] Update README.md
- [ ] Update CONTRIBUTING.md
- [ ] Create CHANGELOG.md entry
- [ ] Update docs/features/ as needed

### Final Review
- [ ] Code review by team
- [ ] Security review for Phase 01 changes
- [ ] Performance comparison
- [ ] Sign-off from stakeholders

---

## Success Criteria

| Criteria | Target | Status |
|----------|--------|--------|
| All tests pass | 100% | Pending |
| Security scans | 0 critical | Pending |
| Clippy warnings | 0 | Pending |
| MyPy errors | 0 | Pending |
| TypeScript errors | 0 | Pending |
| Spec validation | PASS | Pending |
| Documentation | Complete | Pending |
| Changelog | Created | Pending |

---

## Risk Assessment

| Risk | Likelihood | Impact | Mitigation |
|------|------------|--------|------------|
| Tests fail after refactor | Medium | Medium | Fix before merge |
| Performance regression | Low | Medium | Benchmark comparison |
| Security scan finds new issues | Low | High | Address before deploy |

---

## Estimated Completion

- **Test execution**: 0.5 day
- **Security scans**: 0.5 day
- **Static analysis**: 0.5 day
- **Documentation updates**: 0.5 day
- **Review and sign-off**: 1 day

**Total**: 2-3 days

---

## Final Deployment Checklist

Before merging to main and deploying:

- [ ] All validation checks pass
- [ ] PR approved by 2+ reviewers
- [ ] Security review completed for Phase 01
- [ ] FR-REAL-TRADING.md reviewed (if live trading enabled)
- [ ] Staging deployment tested
- [ ] Rollback plan documented
- [ ] Team notified of deployment
- [ ] Monitoring alerts configured
