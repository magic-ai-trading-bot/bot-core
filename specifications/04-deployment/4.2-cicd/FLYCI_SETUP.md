# FlyCI Wingman Setup Guide

Complete setup and usage guide for **FlyCI Wingman** - AI-powered CI/CD failure analysis and intelligent code suggestions for Bot-Core.

---

## Table of Contents

1. [What is FlyCI Wingman?](#what-is-flyci-wingman)
2. [How FlyCI Works](#how-flyci-works)
3. [Installation](#installation)
4. [Configuration](#configuration)
5. [Workflow Integration](#workflow-integration)
6. [Viewing Results](#viewing-results)
7. [Manual Trigger](#manual-trigger)
8. [Configuration Options](#configuration-options)
9. [Best Practices](#best-practices)
10. [Troubleshooting](#troubleshooting)
11. [Examples](#examples)

---

## What is FlyCI Wingman?

**FlyCI Wingman** is an AI-powered GitHub App that automatically analyzes CI/CD build failures and provides intelligent code suggestions via pull request comments.

### Key Features

🔍 **Intelligent Failure Analysis**
- Automatically analyzes build failures using AI
- Identifies root causes of errors
- Detects patterns across multiple failures

💡 **Smart Code Suggestions**
- Provides actionable code fixes
- Suggests improvements with code examples
- Links to relevant documentation

🤖 **Automated PR Comments**
- Posts AI-generated suggestions on failed builds
- Includes severity levels and priority
- Provides step-by-step resolution guides

⚡ **Fast Debugging**
- Reduces debugging time by 50%+
- Accelerates issue resolution
- Improves developer productivity

### Benefits for Bot-Core

- **Faster Issue Resolution** - AI identifies problems before manual debugging
- **Learning Tool** - Helps developers understand failure patterns
- **Quality Assurance** - Catches issues early in CI/CD pipeline
- **Time Savings** - Reduces time spent analyzing logs

---

## How FlyCI Works

### Workflow Process

```
┌─────────────────────────────────────────────────────────────┐
│                    FlyCI Wingman Workflow                   │
├─────────────────────────────────────────────────────────────┤
│                                                             │
│  1. Developer pushes code to GitHub                         │
│                    │                                        │
│                    ▼                                        │
│  2. GitHub Actions CI/CD pipeline runs                      │
│     ├─ Rust Build & Test                                   │
│     ├─ Python Build & Test                                 │
│     └─ Frontend Build & Test                               │
│                    │                                        │
│                    ▼                                        │
│  3. Build Failure Detected                                  │
│     └─ Failure artifacts uploaded                          │
│                    │                                        │
│                    ▼                                        │
│  4. FlyCI Wingman Analyzes Failure                         │
│     ├─ Downloads failure artifacts                         │
│     ├─ Analyzes logs with AI                               │
│     ├─ Identifies root cause                               │
│     └─ Generates code suggestions                          │
│                    │                                        │
│                    ▼                                        │
│  5. FlyCI Posts PR Comment                                  │
│     ├─ Root cause analysis                                 │
│     ├─ Suggested fixes with code examples                  │
│     ├─ Links to relevant documentation                     │
│     └─ Priority and severity levels                        │
│                    │                                        │
│                    ▼                                        │
│  6. Developer Reviews & Applies Fix                         │
│                                                             │
└─────────────────────────────────────────────────────────────┘
```

### What FlyCI Analyzes

**Build Failures:**
- Compilation errors (Rust, Python, TypeScript)
- Linting errors (clippy, flake8, ESLint)
- Type errors (TypeScript, mypy)
- Formatting errors (cargo fmt, black, prettier)

**Test Failures:**
- Unit test failures
- Integration test failures
- E2E test failures
- Coverage threshold failures

**Security Issues:**
- Vulnerability scan failures
- Secret detection failures
- Dependency audit failures

**Performance Issues:**
- Build timeout failures
- Memory limit exceeded
- Performance regression detected

---

## Installation

### Prerequisites

- GitHub repository (Bot-Core)
- Admin access to repository
- GitHub Actions enabled

### Step 1: Install FlyCI GitHub App

1. **Go to FlyCI website:**
   ```
   https://www.flyci.net/
   ```

2. **Click "Install FlyCI Wingman"**

3. **Select Repository:**
   - Choose: `magic-ai-trading-bot/bot-core`
   - Or: Select all repositories (if preferred)

4. **Grant Permissions:**
   - ✅ Read access to code
   - ✅ Write access to pull requests (for comments)
   - ✅ Read access to actions (for workflow runs)
   - ✅ Read access to checks

5. **Confirm Installation**
   - Click "Install & Authorize"
   - Authenticate with GitHub

### Step 2: Verify Installation

```bash
# Check GitHub Apps installed on repository
# Go to: https://github.com/magic-ai-trading-bot/bot-core/settings/installations

# Verify FlyCI Wingman appears in list
```

**Installation is complete!** FlyCI will now automatically analyze all future build failures.

---

## Configuration

### Workflow File

FlyCI integration is configured in:
```
.github/workflows/flyci-wingman.yml
```

### Current Configuration

The workflow is already configured and includes:

**1. Build & Test Jobs:**
- ✅ Rust Core Engine (format, clippy, test, build)
- ✅ Python AI Service (flake8, black, pytest, coverage)
- ✅ Next.js Dashboard (lint, type-check, test, build)

**2. Failure Artifact Collection:**
```yaml
- name: Upload Rust artifacts on failure
  if: failure()
  uses: actions/upload-artifact@v4
  with:
    name: rust-failure-logs
    path: |
      rust-core-engine/target/debug/
      rust-core-engine/Cargo.lock
```

**3. FlyCI Wingman Job:**
```yaml
flyci-wingman:
  name: 🤖 FlyCI Wingman Analysis
  runs-on: ubuntu-latest
  needs: [rust-build-test, python-build-test, frontend-build-test]
  if: always()  # Run even if previous jobs fail
```

**4. Artifact Download:**
```yaml
- name: Download failure artifacts (Rust)
  if: needs.rust-build-test.result == 'failure'
  uses: actions/download-artifact@v4
  with:
    name: rust-failure-logs
    path: ./failure-artifacts/rust/
```

---

## Workflow Integration

### Triggers

FlyCI Wingman runs on:

**Push Events:**
```yaml
push:
  branches: [ main, develop, feature/**, bugfix/**, hotfix/** ]
```

**Pull Request Events:**
```yaml
pull_request:
  branches: [ main, develop ]
```

**Manual Trigger:**
```yaml
workflow_dispatch:
```

### Concurrency Control

```yaml
concurrency:
  group: ${{ github.workflow }}-${{ github.ref }}
  cancel-in-progress: true
```

This ensures only one workflow runs per branch, canceling in-progress runs when new commits are pushed.

### Job Dependencies

```yaml
needs: [rust-build-test, python-build-test, frontend-build-test]
if: always()  # Run even if previous jobs fail
```

FlyCI Wingman runs **after** all build jobs complete, regardless of success or failure.

---

## Viewing Results

### GitHub Actions UI

**1. Go to Actions Tab:**
```
https://github.com/magic-ai-trading-bot/bot-core/actions
```

**2. Click on Workflow Run:**
- Select the failed workflow run
- View individual job logs

**3. View FlyCI Wingman Job:**
- Click on "🤖 FlyCI Wingman Analysis" job
- Review AI analysis in job summary

### Pull Request Comments

**FlyCI posts comments on PRs with:**

**Example Comment:**
```markdown
## 🤖 FlyCI Wingman Analysis

### Build Failure Summary

| Service | Status |
|---------|--------|
| Rust Core Engine | ❌ Failed |
| Python AI Service | ✅ Passed |
| Next.js Dashboard | ✅ Passed |

---

### Root Cause Analysis

**Issue:** Compilation error in `src/trading/engine.rs`

**Error Message:**
```
error[E0425]: cannot find value `order_id` in this scope
  --> src/trading/engine.rs:145:20
   |
145|         let id = order_id;
   |                    ^^^^^^^^ not found in this scope
```

**Root Cause:** Variable `order_id` is not defined in the current scope. It appears you're trying to use a variable that hasn't been declared or is out of scope.

---

### Suggested Fix

**Option 1: Define the variable**
```rust
let order_id = order.id.clone();
let id = order_id;
```

**Option 2: Use order.id directly**
```rust
let id = order.id.clone();
```

---

### Severity: Medium
### Priority: High

**Estimated Time to Fix:** 5 minutes

**Related Documentation:**
- [Rust Error Index - E0425](https://doc.rust-lang.org/error-index.html#E0425)
- [Rust Scope and Shadowing](https://doc.rust-lang.org/book/ch03-01-variables-and-mutability.html)

---

**FlyCI Wingman** - Powered by AI
```

### Job Summary

FlyCI also creates a job summary viewable in GitHub Actions:

```markdown
## 🤖 FlyCI Wingman

FlyCI Wingman is a GitHub App that automatically analyzes failures.

**Installation Status:**
- Install from: https://www.flyci.net/
- Select repository: magic-ai-trading-bot/bot-core
- Once installed, FlyCI will automatically comment on failed builds

**Current Status:** ❌ Builds failed - FlyCI will analyze once installed
```

---

## Manual Trigger

### Via GitHub UI

**1. Go to Actions Tab:**
```
https://github.com/magic-ai-trading-bot/bot-core/actions
```

**2. Select "FlyCI Wingman - AI Code Review" Workflow**

**3. Click "Run workflow" Button**

**4. Select Branch:**
- Choose branch to analyze
- Click "Run workflow"

### Via GitHub CLI

```bash
# Install GitHub CLI
brew install gh

# Authenticate
gh auth login

# Trigger workflow manually
gh workflow run flyci-wingman.yml --ref main

# Trigger with specific inputs (if configured)
gh workflow run flyci-wingman.yml \
  --ref feature/my-feature \
  --field analyze-paths="rust-core-engine/src"
```

---

## Configuration Options

### Customize Analysis Paths

Edit `.github/workflows/flyci-wingman.yml`:

```yaml
env:
  # Analyze specific paths
  ANALYZE_PATHS: |
    rust-core-engine/src
    python-ai-service
    nextjs-ui-dashboard/src

  # Exclude paths
  EXCLUDE_PATHS: |
    **/node_modules
    **/target
    **/__pycache__
    **/coverage
```

### Customize Analysis Options

```yaml
env:
  # Enable/disable features
  FAILURE_ANALYSIS: true        # AI failure analysis
  CODE_SUGGESTIONS: true        # Code fix suggestions
  SECURITY_SCAN: true           # Security checks
  PERFORMANCE_ANALYSIS: true    # Performance checks

  # Comment settings
  COMMENT_ON_SUCCESS: false     # No comment on success
  COMMENT_ON_FAILURE: true      # Comment on failures
  MIN_SEVERITY: medium          # Severity threshold
```

### Customize Artifact Retention

```yaml
- name: Upload failure artifacts
  uses: actions/upload-artifact@v4
  with:
    name: failure-logs
    path: ./logs
    retention-days: 30  # Keep artifacts for 30 days
```

### Timeout Settings

```yaml
jobs:
  rust-build-test:
    timeout-minutes: 30  # Job timeout

  flyci-wingman:
    timeout-minutes: 15  # FlyCI analysis timeout
```

---

## Best Practices

### ✅ DO

**1. Upload Comprehensive Failure Artifacts:**
```yaml
- name: Upload artifacts on failure
  if: failure()
  uses: actions/upload-artifact@v4
  with:
    name: failure-logs
    path: |
      **/target/debug/
      **/.pytest_cache/
      **/coverage/
      **/*.log
```

**2. Use Descriptive Commit Messages:**
```bash
# Good - helps AI understand context
feat(rust): add WebSocket authentication middleware

# Bad - AI has no context
fixed stuff
```

**3. Set Appropriate Timeouts:**
```yaml
timeout-minutes: 30  # Prevent hanging jobs
```

**4. Review FlyCI Suggestions:**
- Read AI suggestions before manually debugging
- Consider multiple suggested options
- Apply fix and verify

**5. Keep Artifacts Small:**
```yaml
path: |
  **/target/debug/deps  # Only debug artifacts
  !**/target/release    # Exclude release builds
```

### ❌ DON'T

**1. Don't Ignore FlyCI Suggestions:**
- AI analysis is usually accurate
- Review suggestions before dismissing

**2. Don't Disable FlyCI Without Understanding:**
```yaml
# Bad - disables valuable analysis
if: false  # Don't do this
```

**3. Don't Set Severity Too High:**
```yaml
# Bad - misses useful suggestions
MIN_SEVERITY: critical  # Only shows critical issues
```

**4. Don't Skip Artifact Uploads:**
```yaml
# Bad - reduces analysis quality
if: success()  # Only uploads on success
```

**5. Don't Upload Secrets:**
```yaml
# Bad - exposes secrets
path: |
  .env  # Don't upload secrets!
  credentials.json
```

---

## Troubleshooting

### Issue: FlyCI Not Posting Comments

**Possible Causes:**
1. FlyCI app not installed
2. Insufficient permissions
3. No PR associated with commit

**Solution:**
```bash
# 1. Verify FlyCI is installed
# Go to: https://github.com/magic-ai-trading-bot/bot-core/settings/installations
# Check if FlyCI Wingman appears

# 2. Check permissions
# FlyCI needs:
# - Read access to code
# - Write access to pull requests
# - Read access to actions

# 3. Ensure commit is on a PR
# Push to a branch and create PR
git checkout -b fix/test-flyci
git push origin fix/test-flyci
# Create PR on GitHub
```

### Issue: Workflow Not Triggering

**Possible Causes:**
1. Workflow file syntax error
2. Branch not in trigger list
3. Workflow disabled

**Solution:**
```bash
# 1. Validate workflow syntax
yamllint .github/workflows/flyci-wingman.yml

# 2. Check trigger branches
cat .github/workflows/flyci-wingman.yml | grep -A 5 "on:"

# 3. Enable workflow
# Go to: Actions → Select workflow → Enable workflow
```

### Issue: Artifacts Not Uploading

**Possible Causes:**
1. Path not matching files
2. Artifacts too large
3. Retention limit exceeded

**Solution:**
```bash
# 1. Verify paths exist
ls -la rust-core-engine/target/debug/

# 2. Check artifact size
du -sh rust-core-engine/target/

# 3. Reduce artifact size
path: |
  rust-core-engine/target/debug/*.log  # Only logs
  !rust-core-engine/target/debug/deps  # Exclude deps
```

### Issue: FlyCI Analysis Incomplete

**Possible Causes:**
1. Timeout reached
2. Insufficient context
3. AI model limitations

**Solution:**
```yaml
# 1. Increase timeout
flyci-wingman:
  timeout-minutes: 20  # Increase from 15

# 2. Provide more context
env:
  ANALYZE_PATHS: |
    rust-core-engine/src
    rust-core-engine/tests  # Include tests for context

# 3. Upload comprehensive logs
path: |
  **/logs/
  **/*.log
  **/target/debug/
```

---

## Examples

### Example 1: Rust Compilation Error

**Build Failure:**
```
error[E0308]: mismatched types
  --> src/trading/engine.rs:42:5
   |
42 |     order_id
   |     ^^^^^^^^ expected struct `String`, found `u64`
```

**FlyCI Comment:**
```markdown
## 🤖 FlyCI Wingman Analysis

**Issue:** Type mismatch in `src/trading/engine.rs:42`

**Root Cause:** Function expects `String` return type but returns `u64`.

**Suggested Fix:**
```rust
// Option 1: Convert u64 to String
order_id.to_string()

// Option 2: Change return type to u64
pub fn get_order_id(&self) -> u64 {
    order_id
}
```

**Severity:** Medium | **Priority:** High
```

### Example 2: Python Test Failure

**Test Failure:**
```
FAILED tests/test_indicators.py::test_calculate_rsi - AssertionError: assert None is not None
```

**FlyCI Comment:**
```markdown
## 🤖 FlyCI Wingman Analysis

**Issue:** RSI calculation returning `None` when value expected

**Root Cause:** Insufficient price data for RSI calculation (need 14+ periods).

**Suggested Fix:**
```python
def test_calculate_rsi():
    # Add more price data
    prices = [Decimal(str(100 + i)) for i in range(20)]  # 20 periods
    period = 14

    rsi = calculate_rsi(prices, period)

    assert rsi is not None
    assert 0 <= rsi <= 100
```

**Severity:** Low | **Priority:** Medium
```

### Example 3: Frontend Lint Error

**Lint Failure:**
```
error  'useState' is not defined  no-undef
```

**FlyCI Comment:**
```markdown
## 🤖 FlyCI Wingman Analysis

**Issue:** `useState` hook not imported in React component

**Root Cause:** Missing React import statement.

**Suggested Fix:**
```typescript
// Add import at top of file
import { useState } from 'react';

// Or use React namespace
import React from 'react';
const [state, setState] = React.useState();
```

**Severity:** High | **Priority:** High
```

---

## Summary

**FlyCI Wingman provides:**
- 🔍 Automated failure analysis
- 💡 Intelligent code suggestions
- 🤖 AI-powered debugging assistance
- ⚡ Faster issue resolution

**Setup is simple:**
1. Install FlyCI GitHub App
2. Grant repository permissions
3. FlyCI automatically analyzes failures
4. Review AI suggestions in PR comments

**Configuration files:**
- Workflow: `.github/workflows/flyci-wingman.yml`
- Documentation: `docs/FLYCI_SETUP.md` (this file)
- CI/CD Spec: `specs/04-deployment/4.2-cicd/CICD-PIPELINE.md`

**Resources:**
- FlyCI Website: https://www.flyci.net/
- FlyCI Docs: https://www.flyci.net/docs
- GitHub Actions: https://github.com/magic-ai-trading-bot/bot-core/actions

---

**Last Updated:** 2025-11-14
**Version:** 1.0.0
**Maintainers:** Bot-Core Development Team
