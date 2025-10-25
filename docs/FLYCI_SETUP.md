# FlyCI Wingman Setup Guide

## Overview

FlyCI Wingman is an AI-powered CI/CD tool that automatically analyzes failing builds and provides intelligent code suggestions directly in your pull requests. This guide explains how FlyCI Wingman is integrated into the bot-core project.

**Official Documentation:** https://www.flyci.net/

**Key Benefits:**
- 🤖 AI-powered failure analysis
- 💡 Intelligent code suggestions
- 🔍 Automatic root cause detection
- 📝 PR comments with fixes
- ⚡ Faster debugging and resolution

---

## Table of Contents

1. [Installation](#installation)
2. [Configuration](#configuration)
3. [How It Works](#how-it-works)
4. [Usage](#usage)
5. [Supported Runners](#supported-runners)
6. [Best Practices](#best-practices)
7. [Troubleshooting](#troubleshooting)
8. [Cost & Limits](#cost--limits)

---

## Installation

### Step 1: Install FlyCI GitHub App

1. Go to the FlyCI Wingman GitHub App page: https://www.flyci.net/
2. Click "Install FlyCI Wingman" or "Add to GitHub"
3. Select your organization/account: `magic-ai-trading-bot`
4. Choose repository access:
   - **Recommended:** Select "Only select repositories" → Choose `bot-core`
   - **Or:** Select "All repositories" (if you want FlyCI on all repos)
5. Review and accept the permissions:
   - ✅ Read access to checks and metadata
   - ✅ Read and write access to actions, pull requests
6. Click "Install & Authorize"

### Step 2: Grant Permissions

FlyCI Wingman requires the following permissions to function:

| Permission | Access Level | Purpose |
|-----------|--------------|---------|
| **Contents** | Read | Access repository code for analysis |
| **Pull Requests** | Write | Post AI suggestions as comments |
| **Checks** | Read | Read build status and failure logs |
| **Actions** | Read | Access GitHub Actions workflow runs |

**Note:** These permissions are already configured in the workflow file (`.github/workflows/flyci-wingman.yml:102-106`).

### Step 3: Verify Installation

After installation, verify FlyCI is working:

```bash
# Create a test branch with a failing build
git checkout -b test/flyci-integration
# Make a change that will fail linting
echo "console.log('test')" >> nextjs-ui-dashboard/src/test-flyci.ts
git add .
git commit -m "test: Verify FlyCI Wingman integration"
git push origin test/flyci-integration
```

Create a PR and wait for FlyCI Wingman to analyze the failure and post suggestions.

---

## Configuration

### Workflow Configuration

The FlyCI Wingman workflow is located at: `.github/workflows/flyci-wingman.yml`

**Key Configuration Options:**

```yaml
# FlyCI Wingman step configuration
- name: Run FlyCI Wingman
  uses: flyci-io/wingman-action@v1
  with:
    # Required
    github-token: ${{ secrets.GITHUB_TOKEN }}

    # Optional: Analyze specific paths
    analyze-paths: |
      rust-core-engine/src
      python-ai-service
      nextjs-ui-dashboard/src

    # Exclude paths from analysis
    exclude-paths: |
      **/node_modules
      **/target
      **/__pycache__
      **/dist
      **/build
      **/.next

    # Analysis options
    failure-analysis: true      # Analyze build failures
    code-suggestions: true      # Provide code fixes
    security-scan: true         # Security vulnerability analysis

    # Comment behavior
    comment-on-success: false   # Don't comment on success
    comment-on-failure: true    # Comment on failures

    # Severity threshold
    min-severity: medium        # Only comment on medium+ issues
```

### Customization Options

#### 1. **Analyze Specific Services**

To analyze only specific services, modify `analyze-paths`:

```yaml
analyze-paths: |
  rust-core-engine/src     # Only Rust service
```

#### 2. **Adjust Comment Threshold**

Control when FlyCI posts comments:

```yaml
min-severity: low       # Comment on all issues
min-severity: medium    # Comment on medium/high issues only (default)
min-severity: high      # Comment only on critical issues
```

#### 3. **Disable Certain Features**

```yaml
failure-analysis: false    # Disable failure analysis
code-suggestions: false    # Disable code suggestions
security-scan: false       # Disable security scanning
```

#### 4. **Always Comment (Even on Success)**

```yaml
comment-on-success: true   # Post a success message
comment-on-failure: true
```

---

## How It Works

### Workflow Flow

```
┌─────────────────────────────────────────────────────────────────┐
│                    1. Build & Test Jobs                         │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐         │
│  │ Rust Build   │  │ Python Build │  │ Frontend     │         │
│  │ & Test       │  │ & Test       │  │ Build & Test │         │
│  └──────┬───────┘  └──────┬───────┘  └──────┬───────┘         │
│         │                  │                  │                  │
│         └──────────────────┴──────────────────┘                 │
│                            │                                     │
└────────────────────────────┼─────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              2. FlyCI Wingman Analysis (if failure)             │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │ 1. Download failure artifacts                            │  │
│  │ 2. Analyze failure logs with AI                          │  │
│  │ 3. Identify root cause                                   │  │
│  │ 4. Generate code suggestions                             │  │
│  │ 5. Post PR comment with fixes                            │  │
│  └──────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│     3. Integration Tests (only if all builds pass)              │
└─────────────────────────────────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│              4. Security Scan & Quality Metrics                 │
└─────────────────────────────────────────────────────────────────┘
                             ▼
┌─────────────────────────────────────────────────────────────────┐
│                    5. Final Status Check                        │
└─────────────────────────────────────────────────────────────────┘
```

### Build Jobs

**1. Rust Core Engine (rust-build-test)**
   - Format check with `cargo fmt`
   - Linting with `cargo clippy`
   - Run all tests with `cargo test`
   - Build release binary
   - Upload failure artifacts if job fails

**2. Python AI Service (python-build-test)**
   - Lint with `flake8`
   - Format check with `black`
   - Type check with `mypy`
   - Run tests with `pytest` + coverage
   - Upload failure artifacts if job fails

**3. Next.js Dashboard (frontend-build-test)**
   - Lint with ESLint
   - Type check with TypeScript
   - Run tests with Vitest
   - Build production bundle
   - Upload failure artifacts if job fails

### FlyCI Wingman Analysis

When any build job fails, FlyCI Wingman automatically:

1. **Downloads failure artifacts** (logs, test results)
2. **Analyzes failures** using AI
3. **Identifies root cause** (syntax errors, type errors, test failures, etc.)
4. **Generates suggestions** with code examples
5. **Posts PR comment** with actionable fixes

Example FlyCI comment:

```markdown
## 🤖 FlyCI Wingman Analysis

### Build Failure Detected
**Job:** Rust Core Engine
**Status:** Failed ❌

### Root Cause
Type mismatch in `rust-core-engine/src/trading/engine.rs:145`

### Suggested Fix
```rust
// Current code (line 145)
let price: f64 = order.price;

// Suggested fix
let price: f64 = order.price.parse().unwrap_or(0.0);
```

### Why This Fixes It
The `order.price` is a String, but the variable expects f64.
Use `.parse()` to convert String → f64.
```

---

## Usage

### Running FlyCI Wingman

FlyCI Wingman runs automatically on:

✅ **Every push** to `main`, `develop`, `feature/*`, `bugfix/*`, `hotfix/*`
✅ **Every pull request** to `main` or `develop`
✅ **Manual trigger** via GitHub Actions UI

### Manual Trigger

1. Go to **Actions** tab in GitHub
2. Select **FlyCI Wingman - AI Code Review** workflow
3. Click **Run workflow**
4. Select branch
5. Click **Run workflow** button

### Viewing Results

**In GitHub Actions:**
- Go to the **Actions** tab
- Click on a workflow run
- View the **FlyCI Wingman Analysis** job
- Check job summary for failure analysis

**In Pull Requests:**
- FlyCI Wingman posts comments directly on the PR
- Comments include:
  - Root cause analysis
  - Suggested fixes with code examples
  - Links to relevant documentation

**Artifacts:**
- Build failure logs are uploaded as artifacts
- Download from the workflow run page

---

## Supported Runners

FlyCI Wingman currently supports these runners:

| Runner | Supported | Notes |
|--------|-----------|-------|
| **ubuntu-latest** | ✅ Yes | Recommended (used in this project) |
| **ubuntu-22.04** | ✅ Yes | Fully supported |
| **ubuntu-20.04** | ✅ Yes | Fully supported |
| **windows-latest** | ⚠️ Partial | Limited AI analysis |
| **macos-latest** | ⚠️ Partial | Limited AI analysis |
| **Self-hosted** | ❌ No | Not currently supported |

**Note:** This project uses `ubuntu-latest` for all jobs, which is fully supported.

---

## Best Practices

### 1. **Use Descriptive Job Names**

```yaml
# ✅ Good - Clear job name
- name: 🦀 Rust Core Engine

# ❌ Bad - Generic name
- name: Build
```

FlyCI uses job names to provide context-aware suggestions.

### 2. **Upload Failure Artifacts**

```yaml
- name: Upload Rust artifacts on failure
  if: failure()
  uses: actions/upload-artifact@v4
  with:
    name: rust-failure-logs
    path: rust-core-engine/target/debug/
```

This allows FlyCI to analyze detailed failure logs.

### 3. **Set Appropriate Timeouts**

```yaml
jobs:
  rust-build-test:
    timeout-minutes: 30  # Prevent hanging builds
```

### 4. **Use `continue-on-error: false`**

```yaml
- name: Format check
  run: cargo fmt -- --check
  continue-on-error: false  # Fail fast for critical checks
```

This ensures FlyCI analyzes real failures, not ignored errors.

### 5. **Keep Build Logs Clean**

- Remove debug print statements
- Use proper logging levels
- Avoid excessive output

Clean logs help FlyCI provide more accurate analysis.

### 6. **Run FlyCI After All Build Jobs**

```yaml
flyci-wingman:
  needs: [rust-build-test, python-build-test, frontend-build-test]
  if: always()  # Run even if builds fail
```

This ensures FlyCI can analyze failures from any service.

### 7. **Set Severity Thresholds**

```yaml
min-severity: medium  # Balance between noise and usefulness
```

- `low`: More comments (may be noisy)
- `medium`: Balanced (recommended)
- `high`: Only critical issues

---

## Troubleshooting

### Issue 1: FlyCI Not Posting Comments

**Symptoms:**
- FlyCI job runs successfully
- No PR comments appear

**Possible Causes:**
1. `comment-on-failure: false` is set
2. Severity threshold too high (`min-severity: high`)
3. No failures detected
4. Insufficient permissions

**Solutions:**
```yaml
# Check configuration
comment-on-failure: true
min-severity: medium

# Verify permissions
permissions:
  pull-requests: write  # Required for comments
```

### Issue 2: FlyCI Job Fails

**Symptoms:**
- FlyCI Wingman job shows red X
- Error in job logs

**Common Errors:**

#### Error: "No artifacts found"
```
Error: Unable to download artifact: Artifact not found
```

**Solution:** This is normal if no builds failed. FlyCI only downloads artifacts when failures occur.

```yaml
# Use continue-on-error for artifact downloads
- name: Download failure artifacts
  uses: actions/download-artifact@v4
  continue-on-error: true  # ← Add this
```

#### Error: "Permission denied"
```
Error: Resource not accessible by integration
```

**Solution:** Check workflow permissions:

```yaml
permissions:
  contents: read
  pull-requests: write
  checks: read
  actions: read
```

### Issue 3: FlyCI Analysis Incorrect

**Symptoms:**
- FlyCI suggests wrong fixes
- Analysis doesn't match actual error

**Solutions:**
1. **Improve error messages** in your code
2. **Upload more detailed artifacts**
3. **Use descriptive commit messages**
4. **Add code comments** explaining complex logic

### Issue 4: Workflow Runs on Wrong Branches

**Symptoms:**
- FlyCI runs on branches you don't want

**Solution:** Adjust trigger configuration:

```yaml
on:
  push:
    branches: [ main, develop ]  # Only these branches
  pull_request:
    branches: [ main ]  # Only PRs to main
```

### Issue 5: FlyCI Timeout

**Symptoms:**
- FlyCI job times out after 6 hours (GitHub default)

**Solution:** Set a reasonable timeout:

```yaml
jobs:
  flyci-wingman:
    timeout-minutes: 10  # FlyCI analysis is usually fast
```

---

## Cost & Limits

### Free Tier

FlyCI Wingman is **FREE** for public repositories (like bot-core):

✅ **Unlimited builds**
✅ **Unlimited PR comments**
✅ **All features enabled**

### Private Repositories (Beta)

For private repositories:

- 🆓 **Free during beta** (current status)
- ⏰ **Future pricing:** TBD (check https://www.flyci.net/pricing)

### Rate Limits

**GitHub Actions Limits:**
- Public repos: Unlimited minutes ✅
- Private repos: 2,000 minutes/month (GitHub Free)

**FlyCI Limits:**
- No additional limits for public repos
- Analysis typically takes < 2 minutes

---

## Integration with Existing CI/CD

### Workflow Order

FlyCI Wingman is integrated into the existing CI/CD pipeline:

```
1. Existing Workflows (run in parallel):
   - ci-cd.yml (Main CI/CD)
   - rust-tests.yml (Rust-specific tests)
   - python-tests.yml (Python-specific tests)
   - nextjs-tests.yml (Frontend-specific tests)
   - integration-tests.yml (Cross-service tests)
   - security-scan.yml (Security checks)

2. New Workflow:
   - flyci-wingman.yml (AI failure analysis)
     └─ Runs AFTER build jobs
     └─ Analyzes failures
     └─ Posts suggestions
```

### Combining with Other Tools

FlyCI Wingman complements existing tools:

| Tool | Purpose | Works With FlyCI? |
|------|---------|-------------------|
| **Codecov** | Code coverage | ✅ Yes - Coverage data in comments |
| **Trivy** | Security scanning | ✅ Yes - Security insights |
| **TruffleHog** | Secret detection | ✅ Yes - Secret leak alerts |
| **Dependabot** | Dependency updates | ✅ Yes - Upgrade suggestions |

---

## Advanced Configuration

### Custom Analysis Rules

Create a `.flyci.yml` config file (optional):

```yaml
# .flyci.yml
version: 1

# Analysis configuration
analysis:
  # Languages to analyze
  languages:
    - rust
    - python
    - typescript

  # Custom rules
  rules:
    - id: custom-rust-rule
      language: rust
      pattern: |
        fn $FUNC() -> Result<$TYPE> {
          $BODY.unwrap()
        }
      message: "Avoid .unwrap() in production code. Use ? operator."
      severity: high

    - id: custom-python-rule
      language: python
      pattern: |
        except:
          pass
      message: "Avoid bare except. Specify exception types."
      severity: medium

# Comment template
comment:
  template: |
    ## 🤖 FlyCI Wingman Analysis

    **Build Status:** {{ status }}
    **Job:** {{ job_name }}

    {{ analysis }}

    {{ suggestions }}
```

### Conditional FlyCI Runs

Run FlyCI only on specific conditions:

```yaml
flyci-wingman:
  if: |
    always() &&
    github.event_name == 'pull_request' &&
    (needs.rust-build-test.result == 'failure' ||
     needs.python-build-test.result == 'failure' ||
     needs.frontend-build-test.result == 'failure')
```

This only runs FlyCI on PRs when builds actually fail.

---

## Monitoring & Metrics

### View FlyCI Statistics

1. Go to **Insights** → **Actions**
2. Select **FlyCI Wingman - AI Code Review**
3. View:
   - Success rate
   - Average run time
   - Failure trends

### Custom Metrics

Add custom metrics to your workflow:

```yaml
- name: FlyCI Metrics
  run: |
    echo "flyci_runtime=${{ steps.flyci.outputs.runtime }}" >> $GITHUB_ENV
    echo "flyci_suggestions=${{ steps.flyci.outputs.suggestion_count }}" >> $GITHUB_ENV
```

---

## Next Steps

1. ✅ **Verify Installation** - Push a test PR and confirm FlyCI comments
2. ✅ **Customize Configuration** - Adjust `min-severity` and `analyze-paths`
3. ✅ **Monitor Results** - Check PR comments for AI suggestions
4. ✅ **Iterate** - Fine-tune based on comment quality

---

## Resources

**Official Links:**
- 🌐 Website: https://www.flyci.net/
- 📚 Documentation: https://www.flyci.net/docs
- 💬 Supported Runners: https://www.flyci.net/docs/supported-runners

**Bot-Core Specific:**
- 📄 Workflow: `.github/workflows/flyci-wingman.yml`
- 📋 CI/CD Spec: `specs/04-deployment/4.2-cicd/CICD-PIPELINE.md`
- 🏷️ Spec Tag: `@spec:FR-CICD-001`

---

## Support

**Questions or Issues?**

1. Check this guide first
2. Review official FlyCI docs: https://www.flyci.net/docs
3. Check GitHub Actions logs
4. Open an issue in bot-core repo with:
   - Workflow run URL
   - Expected behavior
   - Actual behavior
   - Screenshots (if applicable)

---

**Last Updated:** 2025-10-26
**FlyCI Wingman Version:** v1
**Status:** ✅ Active & Integrated
