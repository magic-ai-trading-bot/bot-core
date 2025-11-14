# GitHub Dependabot Automation Guide

**Status**: ✅ CONFIGURED AND ACTIVE
**Last Updated**: 2025-11-15
**Configuration**: `.github/dependabot.yml`

---

## Overview

GitHub Dependabot is **now fully configured** for the Bot-Core project to automatically check and update dependencies across all three services:

- **Rust Core Engine** - Cargo dependencies
- **Python AI Service** - pip dependencies
- **Next.js Dashboard** - npm dependencies
- **GitHub Actions** - workflow dependencies

---

## How It Works

### Automatic Scanning

Dependabot automatically:
- 🔍 **Scans dependencies weekly** (every Monday at 9:00 AM)
- 🚨 **Detects vulnerabilities** in real-time
- 📝 **Creates pull requests** for updates
- ✅ **Groups minor/patch updates** to reduce PR noise
- 🔐 **Prioritizes security updates** for critical vulnerabilities

### Update Strategy

**Security Updates** (Immediate):
- Critical vulnerabilities: PRs created immediately
- High vulnerabilities: PRs created within 24 hours
- Medium/Low vulnerabilities: PRs created on weekly schedule

**Dependency Updates** (Weekly):
- Minor version updates: Grouped together
- Patch version updates: Grouped together
- Major version updates: Individual PRs for review

---

## Current Vulnerabilities

**As of 2025-11-15 (after commit 1d472e9):**

GitHub Dependabot detected **6 vulnerabilities**:
- 🔴 **1 Critical** - Requires immediate attention
- 🟠 **2 High** - High priority fixes
- 🟡 **1 Moderate** - Medium priority
- 🟢 **2 Low** - Low priority

**View Details:**
```bash
# Open GitHub Security tab
open https://github.com/magic-ai-trading-bot/bot-core/security/dependabot

# Or via GitHub CLI
gh browse --repo magic-ai-trading-bot/bot-core --settings security
```

---

## Configuration Details

### Rust Core Engine

```yaml
package-ecosystem: "cargo"
directory: "/rust-core-engine"
schedule:
  interval: "weekly"
  day: "monday"
  time: "09:00"
open-pull-requests-limit: 10
```

**Features:**
- Weekly Cargo.toml scans
- Automatic security updates
- Groups minor/patch updates
- Commits prefixed with `chore(rust):`

### Python AI Service

```yaml
package-ecosystem: "pip"
directory: "/python-ai-service"
schedule:
  interval: "weekly"
  day: "monday"
  time: "09:00"
open-pull-requests-limit: 10
```

**Features:**
- Weekly requirements.txt scans
- ML library updates (TensorFlow, PyTorch, NumPy)
- Groups minor/patch updates
- Commits prefixed with `chore(python):`

### Next.js Dashboard

```yaml
package-ecosystem: "npm"
directory: "/nextjs-ui-dashboard"
schedule:
  interval: "weekly"
  day: "monday"
  time: "09:00"
open-pull-requests-limit: 10
```

**Features:**
- Weekly package.json scans
- React ecosystem updates
- Groups minor/patch updates
- Commits prefixed with `chore(frontend):`

### GitHub Actions

```yaml
package-ecosystem: "github-actions"
directory: "/"
schedule:
  interval: "weekly"
  day: "monday"
  time: "09:00"
open-pull-requests-limit: 5
```

**Features:**
- Weekly workflow action scans
- CI/CD dependency updates
- Commits prefixed with `chore(ci):`

---

## Using Dependabot

### 1. Viewing Dependabot Alerts

**GitHub Web Interface:**
```bash
# Navigate to repository
https://github.com/magic-ai-trading-bot/bot-core

# Click "Security" tab
# Select "Dependabot alerts"
```

**GitHub CLI:**
```bash
# List all Dependabot alerts
gh api repos/magic-ai-trading-bot/bot-core/dependabot/alerts

# List vulnerable dependencies
gh api repos/magic-ai-trading-bot/bot-core/vulnerable-dependencies
```

### 2. Reviewing Dependabot PRs

**When Dependabot creates a PR:**

1. **Check PR Title** - Shows dependency name and version
   ```
   chore(rust): Bump tokio from 1.35.0 to 1.35.1
   ```

2. **Review Changes** - Click "Files changed" tab
   - Rust: Check `Cargo.toml` and `Cargo.lock`
   - Python: Check `requirements.txt`
   - Node.js: Check `package.json` and `package-lock.json`

3. **Check CI/CD** - Verify all tests pass
   ```bash
   # CI/CD runs automatically on Dependabot PRs
   # Check "Actions" tab for test results
   ```

4. **Review Changelog** - Check dependency release notes
   - Dependabot includes changelog links in PR description
   - Review breaking changes (especially for major updates)

5. **Approve and Merge** - If all checks pass
   ```bash
   # Via GitHub CLI
   gh pr review <PR-NUMBER> --approve
   gh pr merge <PR-NUMBER> --auto --squash
   ```

### 3. Manual Dependency Checks

**Force Dependabot to check now:**

```bash
# Trigger manual check (GitHub CLI)
gh api -X POST repos/magic-ai-trading-bot/bot-core/dependabot/secrets

# Or via GitHub Web UI:
# Settings → Security → Dependabot → "Check for updates"
```

**Check specific dependency:**

```bash
# Rust - Check outdated crates
cd rust-core-engine
cargo outdated

# Python - Check outdated packages
cd python-ai-service
pip list --outdated

# Node.js - Check outdated packages
cd nextjs-ui-dashboard
npm outdated
```

---

## Handling Dependabot PRs

### ✅ Safe to Auto-Merge

**Criteria:**
- ✅ Patch version updates (1.2.3 → 1.2.4)
- ✅ Minor version updates (1.2.0 → 1.3.0) for non-critical dependencies
- ✅ All CI/CD tests passing
- ✅ No breaking changes in changelog
- ✅ Security updates with no API changes

**Auto-merge command:**
```bash
gh pr merge <PR-NUMBER> --auto --squash
```

### ⚠️ Requires Manual Review

**Criteria:**
- ⚠️ Major version updates (1.x.x → 2.0.0)
- ⚠️ Updates to critical dependencies (React, Rust core libraries, ML libraries)
- ⚠️ Breaking changes mentioned in changelog
- ⚠️ CI/CD tests failing
- ⚠️ Dependencies with known compatibility issues

**Manual review process:**
1. Read changelog and release notes
2. Check for breaking changes
3. Test locally before merging
4. Update code if needed

### ❌ Reject and Close

**When to reject:**
- ❌ Update introduces breaking changes we can't support
- ❌ Update conflicts with project requirements
- ❌ Update has known bugs or issues
- ❌ Update significantly increases bundle size (frontend)

**Close PR:**
```bash
gh pr close <PR-NUMBER> --comment "Reason for rejection"
```

---

## Security Best Practices

### 1. Critical Vulnerability Response

**Within 24 hours:**
```bash
# 1. Review Dependabot alert
gh api repos/magic-ai-trading-bot/bot-core/dependabot/alerts

# 2. Check if PR already exists
gh pr list --label "dependencies" --label "security"

# 3. If no PR, manually update
cd <service-directory>
# Update dependency to patched version
# Run tests
# Create PR

# 4. Fast-track review and merge
gh pr review <PR-NUMBER> --approve
gh pr merge <PR-NUMBER> --squash
```

### 2. Weekly Dependency Review

**Every Monday (after Dependabot scan):**

```bash
# 1. List all Dependabot PRs
gh pr list --author "app/dependabot"

# 2. Review each PR
# - Check CI/CD status
# - Review changelog
# - Approve or request changes

# 3. Merge approved PRs
gh pr merge <PR-NUMBER> --auto --squash
```

### 3. Quarterly Major Updates

**Every 3 months:**

```bash
# 1. Check for major version updates
cd rust-core-engine && cargo outdated
cd python-ai-service && pip list --outdated
cd nextjs-ui-dashboard && npm outdated

# 2. Plan major updates
# - Create migration plan
# - Test in development environment
# - Update code for breaking changes

# 3. Manual PR for major updates
git checkout -b chore/quarterly-dependency-updates
# Update dependencies
# Test thoroughly
# Create PR for review
```

---

## Customizing Dependabot

### Ignore Specific Dependencies

**Edit `.github/dependabot.yml`:**

```yaml
# Example: Ignore React major updates
- package-ecosystem: "npm"
  directory: "/nextjs-ui-dashboard"
  ignore:
    - dependency-name: "react"
      update-types: ["version-update:semver-major"]
```

### Change Update Frequency

**Options:**
- `daily` - Every day
- `weekly` - Every week (current: Monday)
- `monthly` - First day of month

**Example:**
```yaml
schedule:
  interval: "daily"
  time: "09:00"
```

### Limit Pull Requests

**Adjust `open-pull-requests-limit`:**

```yaml
# Reduce PR noise
open-pull-requests-limit: 5  # Default: 10
```

---

## Troubleshooting

### Issue: Too Many Dependabot PRs

**Solution 1: Group Updates**
```yaml
groups:
  all-dependencies:
    patterns:
      - "*"
    update-types:
      - "minor"
      - "patch"
```

**Solution 2: Reduce Frequency**
```yaml
schedule:
  interval: "monthly"  # Instead of weekly
```

### Issue: Dependabot PR Failing Tests

**Steps:**
1. Review test failure logs in GitHub Actions
2. Check if dependency update introduced breaking changes
3. Either:
   - Fix code to accommodate changes, OR
   - Close PR and ignore that update

**Example:**
```bash
# Review failing tests
gh run view <RUN-ID> --log-failed

# Close PR if incompatible
gh pr close <PR-NUMBER> --comment "Breaking changes, needs manual migration"
```

### Issue: Dependabot Not Creating PRs

**Check:**
1. Verify `.github/dependabot.yml` syntax
2. Check repository permissions
3. Verify GitHub Actions enabled
4. Check Dependabot logs in Settings

**Debug:**
```bash
# Validate YAML syntax
yamllint .github/dependabot.yml

# Check Dependabot status
gh api repos/magic-ai-trading-bot/bot-core/dependabot/secrets
```

---

## Monitoring and Metrics

### Dependabot Dashboard

**GitHub Web UI:**
- Navigate to: `https://github.com/magic-ai-trading-bot/bot-core/security/dependabot`
- View all alerts, PRs, and update history

### Metrics to Track

**Weekly:**
- Number of open Dependabot PRs
- Number of security vulnerabilities
- Average PR merge time

**Monthly:**
- Total dependencies updated
- Security vulnerabilities fixed
- Major version upgrades completed

**Quarterly:**
- Dependency update success rate
- Time to fix critical vulnerabilities
- Breaking change incidents

---

## Integration with CI/CD

Dependabot PRs automatically trigger:

1. **Rust Tests** (`.github/workflows/rust-tests.yml`)
   - `cargo fmt --check`
   - `cargo clippy -- -D warnings`
   - `cargo test --all-targets`
   - `cargo build --release`

2. **Python Tests** (`.github/workflows/python-tests.yml`)
   - `flake8 .`
   - `black --check .`
   - `pytest --cov --cov-report=html`
   - `mypy .`

3. **Frontend Tests** (`.github/workflows/nextjs-tests.yml`)
   - `npm run lint`
   - `npm run type-check`
   - `npm run test:coverage`
   - `npm run build`

4. **Integration Tests** (`.github/workflows/integration-tests.yml`)
   - Cross-service communication tests
   - WebSocket integration tests
   - API endpoint tests

**All tests must pass before merging Dependabot PRs.**

---

## Best Practices

### ✅ DO:
- Review Dependabot PRs promptly (within 1 week)
- Merge security updates quickly (within 24 hours for critical)
- Group minor/patch updates to reduce noise
- Test major updates locally before merging
- Keep dependencies up-to-date regularly
- Monitor Dependabot alerts weekly

### ❌ DON'T:
- Auto-merge major version updates without review
- Ignore security alerts
- Disable Dependabot without good reason
- Merge PRs with failing tests
- Accumulate technical debt by delaying updates
- Skip changelog review for critical dependencies

---

## Resources

**Official Documentation:**
- [GitHub Dependabot Docs](https://docs.github.com/en/code-security/dependabot)
- [Dependabot Configuration Options](https://docs.github.com/en/code-security/dependabot/dependabot-version-updates/configuration-options-for-the-dependabot.yml-file)
- [Dependabot Security Updates](https://docs.github.com/en/code-security/dependabot/dependabot-security-updates)

**Project Files:**
- Configuration: `.github/dependabot.yml`
- Security Policy: `docs/SECURITY_CREDENTIALS.md`
- Contributing Guide: `docs/CONTRIBUTING.md`

**GitHub CLI:**
```bash
# Install GitHub CLI
brew install gh

# Authenticate
gh auth login

# View Dependabot commands
gh dependabot --help
```

---

## Summary

✅ **Dependabot is now fully configured for Bot-Core**

**Active Monitoring:**
- 🦀 Rust Core Engine (Cargo)
- 🐍 Python AI Service (pip)
- ⚛️ Next.js Dashboard (npm)
- 🔧 GitHub Actions (workflows)

**Automatic Actions:**
- Weekly dependency scans (Mondays at 9:00 AM)
- Immediate security vulnerability alerts
- Automatic PRs for updates
- Grouped minor/patch updates
- CI/CD integration

**Current Status:**
- 6 vulnerabilities detected (1 critical, 2 high, 1 moderate, 2 low)
- Dependabot will create PRs to fix these automatically
- All PRs will run comprehensive test suite before merge

**Next Steps:**
1. Monitor Dependabot PRs starting next Monday
2. Review and merge security PRs within 24 hours
3. Plan quarterly major version updates
4. Keep dependencies current to maintain Perfect 10/10 quality

---

**Last Updated**: 2025-11-15
**Status**: ✅ PRODUCTION-READY
**Maintainer**: Bot-Core Development Team
