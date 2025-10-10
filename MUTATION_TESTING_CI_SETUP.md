# Mutation Testing CI/CD Integration Guide

**Complete setup guide for integrating mutation testing into continuous integration**

---

## Quick Start

### 1. Add GitHub Actions Workflow

Create `.github/workflows/mutation-testing.yml`:

```yaml
name: Mutation Testing

on:
  # Run on PR to main
  pull_request:
    branches: [main]
    paths:
      - 'rust-core-engine/src/**/*.rs'
      - 'python-ai-service/services/**/*.py'
      - 'python-ai-service/models/**/*.py'
      - 'nextjs-ui-dashboard/src/**/*.{ts,tsx}'

  # Run weekly to catch test degradation
  schedule:
    - cron: '0 2 * * 0'  # Sunday 2 AM UTC

  # Manual trigger
  workflow_dispatch:
    inputs:
      service:
        description: 'Service to test (rust|python|frontend|all)'
        required: true
        default: 'all'

jobs:
  rust-mutation-testing:
    name: Rust Mutation Testing
    runs-on: ubuntu-latest
    timeout-minutes: 120
    if: ${{ github.event.inputs.service == 'rust' || github.event.inputs.service == 'all' || github.event_name != 'workflow_dispatch' }}

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1
        with:
          toolchain: stable

      - name: Cache cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache cargo build
        uses: actions/cache@v3
        with:
          path: rust-core-engine/target
          key: ${{ runner.os }}-cargo-build-${{ hashFiles('**/Cargo.lock') }}

      - name: Install cargo-mutants
        run: cargo install cargo-mutants --locked

      - name: Run mutation testing (critical modules only)
        working-directory: rust-core-engine
        run: |
          cargo mutants \
            --file 'src/trading/*.rs' \
            --file 'src/strategies/*.rs' \
            --file 'src/paper_trading/*.rs' \
            --timeout 300 \
            --jobs 4 \
            --output mutants.out \
            --no-shuffle \
            2>&1 | tee mutation-log.txt

      - name: Parse mutation results
        id: parse-results
        working-directory: rust-core-engine
        run: |
          if [ -f mutants.out/outcomes.json ]; then
            TOTAL=$(jq '.total_mutants // 0' mutants.out/outcomes.json)
            CAUGHT=$(jq '.caught // 0' mutants.out/outcomes.json)
            MISSED=$(jq '.missed // 0' mutants.out/outcomes.json)
            TIMEOUT=$(jq '.timeout // 0' mutants.out/outcomes.json)
            UNVIABLE=$(jq '.unviable // 0' mutants.out/outcomes.json)

            if [ "$TOTAL" -gt 0 ]; then
              SCORE=$(echo "scale=2; $CAUGHT * 100 / $TOTAL" | bc)
            else
              SCORE=0
            fi

            echo "total=$TOTAL" >> $GITHUB_OUTPUT
            echo "caught=$CAUGHT" >> $GITHUB_OUTPUT
            echo "missed=$MISSED" >> $GITHUB_OUTPUT
            echo "score=$SCORE" >> $GITHUB_OUTPUT
            echo "timeout=$TIMEOUT" >> $GITHUB_OUTPUT

            echo "### Rust Mutation Testing Results" >> $GITHUB_STEP_SUMMARY
            echo "" >> $GITHUB_STEP_SUMMARY
            echo "| Metric | Value |" >> $GITHUB_STEP_SUMMARY
            echo "|--------|-------|" >> $GITHUB_STEP_SUMMARY
            echo "| **Mutation Score** | **${SCORE}%** |" >> $GITHUB_STEP_SUMMARY
            echo "| Total Mutants | $TOTAL |" >> $GITHUB_STEP_SUMMARY
            echo "| Caught | $CAUGHT |" >> $GITHUB_STEP_SUMMARY
            echo "| Missed | $MISSED |" >> $GITHUB_STEP_SUMMARY
            echo "| Timeout | $TIMEOUT |" >> $GITHUB_STEP_SUMMARY
            echo "| Unviable | $UNVIABLE |" >> $GITHUB_STEP_SUMMARY
          else
            echo "No mutation results found"
            echo "score=0" >> $GITHUB_OUTPUT
          fi

      - name: Check mutation score threshold
        run: |
          SCORE="${{ steps.parse-results.outputs.score }}"
          THRESHOLD=75

          echo "Mutation Score: ${SCORE}%"
          echo "Threshold: ${THRESHOLD}%"

          if (( $(echo "$SCORE < $THRESHOLD" | bc -l) )); then
            echo "❌ Mutation score ${SCORE}% is below ${THRESHOLD}% threshold"
            echo "::warning::Rust mutation score ${SCORE}% below target ${THRESHOLD}%"
            # Don't fail on low score, just warn
            # exit 1
          else
            echo "✅ Mutation score ${SCORE}% meets ${THRESHOLD}% threshold"
          fi

      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: rust-mutation-report
          path: |
            rust-core-engine/mutants.out/
            rust-core-engine/mutation-log.txt
          retention-days: 30

      - name: Comment PR with results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const score = '${{ steps.parse-results.outputs.score }}';
            const caught = '${{ steps.parse-results.outputs.caught }}';
            const missed = '${{ steps.parse-results.outputs.missed }}';
            const total = '${{ steps.parse-results.outputs.total }}';

            const emoji = parseFloat(score) >= 75 ? '✅' : '⚠️';

            const body = `## ${emoji} Rust Mutation Testing Results

            **Mutation Score:** ${score}%
            - ✅ Caught: ${caught} mutants
            - ❌ Missed: ${missed} mutants
            - 📊 Total: ${total} mutants

            ${parseFloat(score) < 75 ? '**Warning:** Mutation score below 75% target' : '**Success:** Mutation score meets target!'}

            [View detailed report](https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }})
            `;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });

  python-mutation-testing:
    name: Python Mutation Testing
    runs-on: ubuntu-latest
    timeout-minutes: 60
    if: ${{ github.event.inputs.service == 'python' || github.event.inputs.service == 'all' || github.event_name != 'workflow_dispatch' }}

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
          cache: 'pip'

      - name: Install dependencies
        working-directory: python-ai-service
        run: |
          pip install -r requirements.txt
          pip install -r requirements.test.txt
          pip install mutmut

      - name: Run mutation testing
        working-directory: python-ai-service
        run: |
          # Clear previous cache
          rm -rf .mutmut-cache

          # Run mutmut
          mutmut run \
            --paths-to-mutate=services/,models/,utils/ \
            --tests-dir=tests/ \
            --runner='pytest tests/ -x --tb=short' \
            || true  # Don't fail on mutations

          # Generate results
          mutmut results > mutation-results.txt
          mutmut html || true

      - name: Parse mutation results
        id: parse-results
        working-directory: python-ai-service
        run: |
          if [ -f mutation-results.txt ]; then
            TOTAL=$(grep -oP 'Total: \K\d+' mutation-results.txt || echo "0")
            KILLED=$(grep -oP 'Killed: \K\d+' mutation-results.txt || echo "0")
            SURVIVED=$(grep -oP 'Survived: \K\d+' mutation-results.txt || echo "0")
            TIMEOUT=$(grep -oP 'Timeout: \K\d+' mutation-results.txt || echo "0")

            if [ "$TOTAL" -gt 0 ]; then
              SCORE=$(echo "scale=2; $KILLED * 100 / $TOTAL" | bc)
            else
              SCORE=0
            fi

            echo "total=$TOTAL" >> $GITHUB_OUTPUT
            echo "killed=$KILLED" >> $GITHUB_OUTPUT
            echo "survived=$SURVIVED" >> $GITHUB_OUTPUT
            echo "score=$SCORE" >> $GITHUB_OUTPUT

            echo "### Python Mutation Testing Results" >> $GITHUB_STEP_SUMMARY
            echo "" >> $GITHUB_STEP_SUMMARY
            echo "| Metric | Value |" >> $GITHUB_STEP_SUMMARY
            echo "|--------|-------|" >> $GITHUB_STEP_SUMMARY
            echo "| **Mutation Score** | **${SCORE}%** |" >> $GITHUB_STEP_SUMMARY
            echo "| Total Mutants | $TOTAL |" >> $GITHUB_STEP_SUMMARY
            echo "| Killed | $KILLED |" >> $GITHUB_STEP_SUMMARY
            echo "| Survived | $SURVIVED |" >> $GITHUB_STEP_SUMMARY
            echo "| Timeout | $TIMEOUT |" >> $GITHUB_STEP_SUMMARY
          else
            echo "No mutation results found"
            echo "score=0" >> $GITHUB_OUTPUT
          fi

      - name: Check mutation score threshold
        run: |
          SCORE="${{ steps.parse-results.outputs.score }}"
          THRESHOLD=75

          echo "Mutation Score: ${SCORE}%"
          echo "Threshold: ${THRESHOLD}%"

          if (( $(echo "$SCORE < $THRESHOLD" | bc -l) )); then
            echo "❌ Mutation score ${SCORE}% is below ${THRESHOLD}% threshold"
            echo "::warning::Python mutation score ${SCORE}% below target ${THRESHOLD}%"
          else
            echo "✅ Mutation score ${SCORE}% meets ${THRESHOLD}% threshold"
          fi

      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: python-mutation-report
          path: |
            python-ai-service/html/
            python-ai-service/mutation-results.txt
          retention-days: 30

  frontend-mutation-testing:
    name: Frontend Mutation Testing
    runs-on: ubuntu-latest
    timeout-minutes: 60
    if: ${{ github.event.inputs.service == 'frontend' || github.event.inputs.service == 'all' || github.event_name != 'workflow_dispatch' }}

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Setup Node.js
        uses: actions/setup-node@v4
        with:
          node-version: '20'
          cache: 'npm'
          cache-dependency-path: nextjs-ui-dashboard/package-lock.json

      - name: Install dependencies
        working-directory: nextjs-ui-dashboard
        run: npm ci

      - name: Run Stryker mutation testing
        working-directory: nextjs-ui-dashboard
        run: |
          npx stryker run \
            --mutationScoreThreshold 75 \
            --reporters html,json,clear-text \
            || true  # Don't fail on low score

      - name: Parse mutation results
        id: parse-results
        working-directory: nextjs-ui-dashboard
        run: |
          if [ -f reports/mutation/mutation-report.json ]; then
            SCORE=$(jq '.mutationScore // 0' reports/mutation/mutation-report.json)
            TOTAL=$(jq '.totalMutants // 0' reports/mutation/mutation-report.json)
            KILLED=$(jq '.killed // 0' reports/mutation/mutation-report.json)
            SURVIVED=$(jq '.survived // 0' reports/mutation/mutation-report.json)

            echo "total=$TOTAL" >> $GITHUB_OUTPUT
            echo "killed=$KILLED" >> $GITHUB_OUTPUT
            echo "survived=$SURVIVED" >> $GITHUB_OUTPUT
            echo "score=$SCORE" >> $GITHUB_OUTPUT

            echo "### Frontend Mutation Testing Results" >> $GITHUB_STEP_SUMMARY
            echo "" >> $GITHUB_STEP_SUMMARY
            echo "| Metric | Value |" >> $GITHUB_STEP_SUMMARY
            echo "|--------|-------|" >> $GITHUB_STEP_SUMMARY
            echo "| **Mutation Score** | **${SCORE}%** |" >> $GITHUB_STEP_SUMMARY
            echo "| Total Mutants | $TOTAL |" >> $GITHUB_STEP_SUMMARY
            echo "| Killed | $KILLED |" >> $GITHUB_STEP_SUMMARY
            echo "| Survived | $SURVIVED |" >> $GITHUB_STEP_SUMMARY
          else
            echo "No mutation results found"
            echo "score=0" >> $GITHUB_OUTPUT
          fi

      - name: Upload mutation report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: frontend-mutation-report
          path: nextjs-ui-dashboard/reports/mutation/
          retention-days: 30

  mutation-summary:
    name: Mutation Testing Summary
    runs-on: ubuntu-latest
    needs: [rust-mutation-testing, python-mutation-testing, frontend-mutation-testing]
    if: always()

    steps:
      - name: Create summary
        run: |
          echo "# 🧬 Mutation Testing Summary" >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "All mutation testing jobs completed. Check individual job results for details." >> $GITHUB_STEP_SUMMARY
          echo "" >> $GITHUB_STEP_SUMMARY
          echo "**Target:** ≥75% mutation score across all services" >> $GITHUB_STEP_SUMMARY
```

---

## 2. Local Development Workflow

### Run mutation testing locally before pushing:

```bash
#!/bin/bash
# scripts/run-mutation-tests.sh

set -e

echo "🧬 Running Mutation Testing Locally"
echo "==================================="

# Function to run Rust mutation testing
run_rust_mutations() {
    echo ""
    echo "📦 Rust Core Engine"
    echo "-------------------"
    cd rust-core-engine

    # Run on changed files only
    CHANGED_FILES=$(git diff --name-only main...HEAD | grep '\.rs$' || true)

    if [ -z "$CHANGED_FILES" ]; then
        echo "No Rust files changed, skipping mutation testing"
    else
        echo "Testing changed files: $CHANGED_FILES"

        for file in $CHANGED_FILES; do
            echo "Mutating: $file"
            cargo mutants --file "$file" --timeout 120 --jobs 2
        done
    fi

    cd ..
}

# Function to run Python mutation testing
run_python_mutations() {
    echo ""
    echo "🐍 Python AI Service"
    echo "--------------------"
    cd python-ai-service

    # Run on changed files only
    CHANGED_FILES=$(git diff --name-only main...HEAD | grep '\.py$' | grep -E '^(services|models|utils)/' || true)

    if [ -z "$CHANGED_FILES" ]; then
        echo "No Python files changed, skipping mutation testing"
    else
        echo "Testing changed files: $CHANGED_FILES"

        # Clear cache
        rm -rf .mutmut-cache

        # Run mutmut on specific files
        for file in $CHANGED_FILES; do
            echo "Mutating: $file"
            mutmut run --paths-to-mutate="$file" --tests-dir=tests/ || true
        done

        # Show results
        mutmut results
    fi

    cd ..
}

# Function to run Frontend mutation testing
run_frontend_mutations() {
    echo ""
    echo "⚛️  Frontend Dashboard"
    echo "---------------------"
    cd nextjs-ui-dashboard

    # Check for changed files
    CHANGED_FILES=$(git diff --name-only main...HEAD | grep -E '\.(ts|tsx)$' | grep '^src/' || true)

    if [ -z "$CHANGED_FILES" ]; then
        echo "No frontend files changed, skipping mutation testing"
    else
        echo "Testing changed files: $CHANGED_FILES"

        # Run Stryker (it automatically detects changed files in CI mode)
        npx stryker run --concurrency 2
    fi

    cd ..
}

# Main execution
case "${1:-all}" in
    rust)
        run_rust_mutations
        ;;
    python)
        run_python_mutations
        ;;
    frontend)
        run_frontend_mutations
        ;;
    all)
        run_rust_mutations
        run_python_mutations
        run_frontend_mutations
        ;;
    *)
        echo "Usage: $0 {rust|python|frontend|all}"
        exit 1
        ;;
esac

echo ""
echo "✅ Mutation testing complete!"
```

Make executable:
```bash
chmod +x scripts/run-mutation-tests.sh
```

---

## 3. Pre-commit Hook

Add mutation testing to pre-commit (optional, for critical files only):

```bash
# .git/hooks/pre-commit
#!/bin/bash

echo "Running pre-commit mutation testing on critical files..."

# Get staged files
STAGED_RS=$(git diff --cached --name-only --diff-filter=ACM | grep '\.rs$' | grep -E '(trading|strategies)/' || true)
STAGED_PY=$(git diff --cached --name-only --diff-filter=ACM | grep '\.py$' | grep -E '(services|models)/' || true)

EXIT_CODE=0

# Test critical Rust files
if [ -n "$STAGED_RS" ]; then
    echo "Testing critical Rust files: $STAGED_RS"
    cd rust-core-engine
    for file in $STAGED_RS; do
        cargo mutants --file "../$file" --timeout 60 --jobs 1 || EXIT_CODE=1
    done
    cd ..
fi

# Test critical Python files
if [ -n "$STAGED_PY" ]; then
    echo "Testing critical Python files: $STAGED_PY"
    cd python-ai-service
    for file in $STAGED_PY; do
        mutmut run --paths-to-mutate="../$file" --tests-dir=tests/ || true
    done
    mutmut results || EXIT_CODE=1
    cd ..
fi

if [ $EXIT_CODE -ne 0 ]; then
    echo "❌ Mutation testing failed. Commit aborted."
    echo "Run 'scripts/run-mutation-tests.sh' to see details"
    exit 1
fi

exit 0
```

---

## 4. Makefile Integration

Add to `Makefile`:

```makefile
# Mutation testing targets

.PHONY: mutation-test
mutation-test: mutation-test-rust mutation-test-python mutation-test-frontend

.PHONY: mutation-test-rust
mutation-test-rust:
	@echo "Running Rust mutation testing..."
	cd rust-core-engine && \
	cargo mutants \
		--file 'src/strategies/*.rs' \
		--file 'src/trading/*.rs' \
		--timeout 300 \
		--jobs 4 \
		--output mutants.out

.PHONY: mutation-test-python
mutation-test-python:
	@echo "Running Python mutation testing..."
	cd python-ai-service && \
	rm -rf .mutmut-cache && \
	mutmut run --paths-to-mutate=services/,models/,utils/ && \
	mutmut results && \
	mutmut html

.PHONY: mutation-test-frontend
mutation-test-frontend:
	@echo "Running Frontend mutation testing..."
	cd nextjs-ui-dashboard && \
	npx stryker run

.PHONY: mutation-report
mutation-report:
	@echo "Generating mutation testing reports..."
	@echo "Rust: file://$(PWD)/rust-core-engine/mutants.out/index.html"
	@echo "Python: file://$(PWD)/python-ai-service/html/index.html"
	@echo "Frontend: file://$(PWD)/nextjs-ui-dashboard/reports/mutation/html/index.html"
```

Usage:
```bash
make mutation-test           # Run all mutation tests
make mutation-test-rust      # Rust only
make mutation-test-python    # Python only
make mutation-test-frontend  # Frontend only
make mutation-report         # Show report URLs
```

---

## 5. Badge Setup

### Add to README.md:

```markdown
## Test Quality Metrics

![Tests](https://github.com/yourusername/bot-core/workflows/Tests/badge.svg)
![Mutation Score - Rust](https://img.shields.io/badge/mutation%20score%20(rust)-dynamic-brightgreen?url=https://yourusername.github.io/bot-core/badges/rust-mutation.json&query=$.score&suffix=%25)
![Mutation Score - Python](https://img.shields.io/badge/mutation%20score%20(python)-dynamic-brightgreen?url=https://yourusername.github.io/bot-core/badges/python-mutation.json&query=$.score&suffix=%25)
![Mutation Score - Frontend](https://img.shields.io/badge/mutation%20score%20(frontend)-dynamic-brightgreen?url=https://yourusername.github.io/bot-core/badges/frontend-mutation.json&query=$.score&suffix=%25)

**Mutation Testing:** Verifies that tests actually catch bugs, not just achieve coverage.
- Target: ≥75% mutation score
- See [Mutation Testing Report](./MUTATION_TESTING_REPORT.md) for details
```

---

## 6. Troubleshooting

### Common Issues

**Problem: Mutation testing times out**
```bash
# Solution: Increase timeout and reduce parallelism
cargo mutants --timeout 600 --jobs 1
```

**Problem: Too many mutants to test**
```bash
# Solution: Test critical files only
cargo mutants --file 'src/trading/*.rs'  # Just trading module
```

**Problem: Baseline tests fail**
```bash
# Solution: Check tests pass first
cargo test
pytest tests/
npm test
```

**Problem: Out of memory**
```bash
# Solution: Reduce parallelism
cargo mutants --jobs 1  # Sequential execution
```

**Problem: False positives (equivalent mutants)**
```bash
# Solution: Mark equivalent mutants
# In Rust: Add #[cfg_attr(test, mutants::skip)]
# In Python: Add # pragma: no mutate
```

---

## 7. Monitoring & Alerts

### Set up Slack notifications:

```yaml
# Add to .github/workflows/mutation-testing.yml

- name: Notify Slack on failure
  if: failure()
  uses: 8398a7/action-slack@v3
  with:
    status: ${{ job.status }}
    text: |
      Mutation testing failed!
      Service: ${{ matrix.service }}
      Score: ${{ steps.parse-results.outputs.score }}%
      Threshold: 75%
    webhook_url: ${{ secrets.SLACK_WEBHOOK }}
```

### Set up email notifications:

```yaml
- name: Email results
  if: always()
  uses: dawidd6/action-send-mail@v3
  with:
    server_address: smtp.gmail.com
    server_port: 465
    username: ${{ secrets.EMAIL_USERNAME }}
    password: ${{ secrets.EMAIL_PASSWORD }}
    subject: Mutation Testing Results - ${{ github.repository }}
    body: |
      Mutation Score: ${{ steps.parse-results.outputs.score }}%
      Total Mutants: ${{ steps.parse-results.outputs.total }}
      Caught: ${{ steps.parse-results.outputs.caught }}
      See: https://github.com/${{ github.repository }}/actions/runs/${{ github.run_id }}
    to: team@yourcompany.com
```

---

## 8. Next Steps

1. ✅ Copy `.github/workflows/mutation-testing.yml` to your repo
2. ✅ Copy `scripts/run-mutation-tests.sh` and make executable
3. ✅ Update `Makefile` with mutation testing targets
4. ✅ Add badges to `README.md`
5. ✅ Configure Slack/email notifications
6. ✅ Run first mutation test: `make mutation-test`
7. ✅ Review results and fix weak tests
8. ✅ Enable required checks in GitHub branch protection

**Questions or issues?** See `MUTATION_TESTING_REPORT.md` for detailed analysis.
