# CI/CD Workflow Automation Specification

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** DevOps Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Dependency Management](#2-dependency-management)
- [3. Security Scanning](#3-security-scanning)
- [4. Performance Testing](#4-performance-testing)
- [5. Database Migrations](#5-database-migrations)
- [6. Documentation Generation](#6-documentation-generation)
- [7. Release Management](#7-release-management)

---

## 1. Overview

This document defines automated workflows for dependency updates, security scanning, testing, migrations, and documentation generation.

---

## 2. Dependency Management

### 2.1 Dependabot Configuration

**File:** `.github/dependabot.yml`

```yaml
version: 2
updates:
  # Rust dependencies
  - package-ecosystem: "cargo"
    directory: "/rust-core-engine"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
    open-pull-requests-limit: 10
    reviewers:
      - "rust-team"
    labels:
      - "dependencies"
      - "rust"
    commit-message:
      prefix: "chore(rust)"
    ignore:
      - dependency-name: "*"
        update-types: ["version-update:semver-major"]

  # Python dependencies
  - package-ecosystem: "pip"
    directory: "/python-ai-service"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
    open-pull-requests-limit: 10
    reviewers:
      - "python-team"
    labels:
      - "dependencies"
      - "python"
    commit-message:
      prefix: "chore(python)"

  # npm dependencies
  - package-ecosystem: "npm"
    directory: "/nextjs-ui-dashboard"
    schedule:
      interval: "weekly"
      day: "monday"
      time: "09:00"
    open-pull-requests-limit: 10
    reviewers:
      - "frontend-team"
    labels:
      - "dependencies"
      - "frontend"
    commit-message:
      prefix: "chore(frontend)"
    ignore:
      - dependency-name: "*"
        update-types: ["version-update:semver-major"]

  # Docker base images
  - package-ecosystem: "docker"
    directory: "/rust-core-engine"
    schedule:
      interval: "weekly"
      day: "monday"
    labels:
      - "dependencies"
      - "docker"

  # GitHub Actions
  - package-ecosystem: "github-actions"
    directory: "/"
    schedule:
      interval: "monthly"
    labels:
      - "dependencies"
      - "ci"
```

### 2.2 Automated Dependency Updates

**File:** `.github/workflows/dependency-update.yml`

```yaml
name: Automated Dependency Updates

on:
  schedule:
    - cron: '0 2 * * 1'  # Every Monday at 2 AM
  workflow_dispatch:

jobs:
  update-rust-deps:
    name: Update Rust Dependencies
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Update dependencies
        run: |
          cd rust-core-engine
          cargo update
          cargo check

      - name: Create Pull Request
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: "chore(rust): update dependencies"
          title: "chore(rust): automated dependency updates"
          body: "Automated update of Rust dependencies"
          branch: "deps/rust-auto-update"
          labels: dependencies, rust

  update-python-deps:
    name: Update Python Dependencies
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Update dependencies
        run: |
          cd python-ai-service
          pip install pip-tools
          pip-compile --upgrade requirements.in
          pip install -r requirements.txt
          pytest tests/

      - name: Create Pull Request
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: "chore(python): update dependencies"
          title: "chore(python): automated dependency updates"
          body: "Automated update of Python dependencies"
          branch: "deps/python-auto-update"
          labels: dependencies, python

  update-npm-deps:
    name: Update npm Dependencies
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Update dependencies
        run: |
          cd nextjs-ui-dashboard
          npm update
          npm audit fix
          npm test

      - name: Create Pull Request
        uses: peter-evans/create-pull-request@v5
        with:
          token: ${{ secrets.GITHUB_TOKEN }}
          commit-message: "chore(frontend): update dependencies"
          title: "chore(frontend): automated dependency updates"
          body: "Automated update of npm dependencies"
          branch: "deps/npm-auto-update"
          labels: dependencies, frontend
```

---

## 3. Security Scanning

### 3.1 Scheduled Security Scans

**File:** `.github/workflows/security-scan.yml`

```yaml
name: Security Scanning

on:
  schedule:
    - cron: '0 3 * * *'  # Daily at 3 AM
  workflow_dispatch:

jobs:
  trivy-scan:
    name: Trivy Container Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run Trivy for Rust
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: './rust-core-engine'
          format: 'sarif'
          output: 'rust-trivy.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Run Trivy for Python
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: './python-ai-service'
          format: 'sarif'
          output: 'python-trivy.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Run Trivy for Frontend
        uses: aquasecurity/trivy-action@master
        with:
          scan-type: 'fs'
          scan-ref: './nextjs-ui-dashboard'
          format: 'sarif'
          output: 'frontend-trivy.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Upload results
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: '.'

  snyk-scan:
    name: Snyk Vulnerability Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run Snyk for Rust
        uses: snyk/actions/rust@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        with:
          args: --severity-threshold=high

      - name: Run Snyk for Python
        uses: snyk/actions/python@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        with:
          args: --severity-threshold=high

      - name: Run Snyk for npm
        uses: snyk/actions/node@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        with:
          args: --severity-threshold=high

  code-scanning:
    name: CodeQL Analysis
    runs-on: ubuntu-latest
    permissions:
      security-events: write
    strategy:
      matrix:
        language: ['javascript', 'python']
    steps:
      - uses: actions/checkout@v3

      - name: Initialize CodeQL
        uses: github/codeql-action/init@v2
        with:
          languages: ${{ matrix.language }}

      - name: Autobuild
        uses: github/codeql-action/autobuild@v2

      - name: Perform CodeQL Analysis
        uses: github/codeql-action/analyze@v2
```

### 3.2 Secrets Scanning

**File:** `.github/workflows/secrets-scan.yml`

```yaml
name: Secrets Scanning

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  gitleaks:
    name: Gitleaks Secret Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      - name: Run Gitleaks
        uses: gitleaks/gitleaks-action@v2
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
          GITLEAKS_LICENSE: ${{ secrets.GITLEAKS_LICENSE }}

  trufflehog:
    name: TruffleHog Secret Scan
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      - name: Run TruffleHog
        uses: trufflesecurity/trufflehog@main
        with:
          path: ./
          base: ${{ github.event.repository.default_branch }}
          head: HEAD
```

---

## 4. Performance Testing

### 4.1 Load Testing Workflow

**File:** `.github/workflows/performance-test.yml`

```yaml
name: Performance Testing

on:
  schedule:
    - cron: '0 4 * * 0'  # Weekly on Sunday at 4 AM
  workflow_dispatch:

jobs:
  load-test:
    name: Load Testing
    runs-on: ubuntu-latest
    environment: staging

    steps:
      - uses: actions/checkout@v3

      - name: Setup k6
        run: |
          sudo apt-key adv --keyserver hkp://keyserver.ubuntu.com:80 --recv-keys C5AD17C747E3415A3642D57D77C6C491D6AC1D69
          echo "deb https://dl.k6.io/deb stable main" | sudo tee /etc/apt/sources.list.d/k6.list
          sudo apt-get update
          sudo apt-get install k6

      - name: Run load test - Rust API
        run: |
          k6 run --out json=rust-results.json tests/load/rust-api-load-test.js
        env:
          BASE_URL: https://staging.botcore.app

      - name: Run load test - Python AI
        run: |
          k6 run --out json=python-results.json tests/load/python-ai-load-test.js
        env:
          BASE_URL: https://staging.botcore.app

      - name: Run load test - Frontend
        run: |
          k6 run --out json=frontend-results.json tests/load/frontend-load-test.js
        env:
          BASE_URL: https://staging.botcore.app

      - name: Analyze results
        run: |
          python tests/load/analyze-results.py \
            rust-results.json \
            python-results.json \
            frontend-results.json

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: load-test-results
          path: '*.json'

      - name: Comment on PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const results = JSON.parse(fs.readFileSync('summary.json', 'utf8'));
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: `## Performance Test Results\n\n${results.summary}`
            });

  stress-test:
    name: Stress Testing
    runs-on: ubuntu-latest
    environment: staging

    steps:
      - uses: actions/checkout@v3

      - name: Setup Artillery
        run: npm install -g artillery

      - name: Run stress test
        run: |
          artillery run tests/stress/stress-test.yml \
            --output stress-report.json

      - name: Generate HTML report
        run: |
          artillery report stress-report.json --output stress-report.html

      - name: Upload report
        uses: actions/upload-artifact@v3
        with:
          name: stress-test-report
          path: stress-report.html
```

---

## 5. Database Migrations

### 5.1 Automated Migration Workflow

**File:** `.github/workflows/db-migration.yml`

```yaml
name: Database Migration

on:
  push:
    paths:
      - 'migrations/**'
    branches: [main]
  workflow_dispatch:

jobs:
  validate-migration:
    name: Validate Migration
    runs-on: ubuntu-latest

    services:
      mongodb:
        image: mongo:7.0
        ports:
          - 27017:27017
        env:
          MONGO_INITDB_ROOT_USERNAME: testuser
          MONGO_INITDB_ROOT_PASSWORD: testpass

    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install migration tools
        run: pip install pymongo migrate-mongo

      - name: Run migration on test database
        run: |
          python scripts/migrate.py \
            --uri "mongodb://testuser:testpass@localhost:27017/test_db" \
            --direction up \
            --dry-run

      - name: Validate data integrity
        run: |
          python scripts/validate-migration.py \
            --uri "mongodb://testuser:testpass@localhost:27017/test_db"

  migrate-staging:
    name: Migrate Staging Database
    runs-on: ubuntu-latest
    needs: [validate-migration]
    environment: staging

    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install migration tools
        run: pip install pymongo migrate-mongo

      - name: Backup database
        run: |
          mongodump --uri="${{ secrets.STAGING_DATABASE_URL }}" \
            --out=/tmp/backup/staging_$(date +%Y%m%d_%H%M%S)

      - name: Upload backup to S3
        uses: jakejarvis/s3-sync-action@master
        with:
          args: --follow-symlinks
        env:
          AWS_S3_BUCKET: ${{ secrets.BACKUP_BUCKET }}
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          AWS_REGION: 'us-east-1'
          SOURCE_DIR: '/tmp/backup'

      - name: Run migration
        run: |
          python scripts/migrate.py \
            --uri "${{ secrets.STAGING_DATABASE_URL }}" \
            --direction up

      - name: Validate migration
        run: |
          python scripts/validate-migration.py \
            --uri "${{ secrets.STAGING_DATABASE_URL }}"

      - name: Notify on failure
        if: failure()
        run: |
          echo "Migration failed! Backup available at S3"
          # Send alert to Slack/PagerDuty

  migrate-production:
    name: Migrate Production Database
    runs-on: ubuntu-latest
    needs: [migrate-staging]
    environment: production

    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install migration tools
        run: pip install pymongo migrate-mongo

      - name: Backup database
        run: |
          mongodump --uri="${{ secrets.PRODUCTION_DATABASE_URL }}" \
            --out=/tmp/backup/production_$(date +%Y%m%d_%H%M%S)

      - name: Upload backup to S3
        uses: jakejarvis/s3-sync-action@master
        with:
          args: --follow-symlinks
        env:
          AWS_S3_BUCKET: ${{ secrets.BACKUP_BUCKET }}
          AWS_ACCESS_KEY_ID: ${{ secrets.AWS_ACCESS_KEY_ID }}
          AWS_SECRET_ACCESS_KEY: ${{ secrets.AWS_SECRET_ACCESS_KEY }}
          AWS_REGION: 'us-east-1'
          SOURCE_DIR: '/tmp/backup'

      - name: Create maintenance window
        run: |
          curl -X POST https://api.statuspage.io/v1/pages/${{ secrets.STATUSPAGE_ID }}/incidents \
            -H "Authorization: OAuth ${{ secrets.STATUSPAGE_TOKEN }}" \
            -d '{"incident":{"name":"Database Maintenance","status":"investigating","impact_override":"maintenance"}}'

      - name: Run migration with rollback capability
        id: migration
        run: |
          python scripts/migrate.py \
            --uri "${{ secrets.PRODUCTION_DATABASE_URL }}" \
            --direction up \
            --with-rollback

      - name: Validate migration
        run: |
          python scripts/validate-migration.py \
            --uri "${{ secrets.PRODUCTION_DATABASE_URL }}"

      - name: Close maintenance window
        if: always()
        run: |
          curl -X PATCH https://api.statuspage.io/v1/pages/${{ secrets.STATUSPAGE_ID }}/incidents/latest \
            -H "Authorization: OAuth ${{ secrets.STATUSPAGE_TOKEN }}" \
            -d '{"incident":{"status":"resolved"}}'

      - name: Rollback on failure
        if: failure()
        run: |
          python scripts/migrate.py \
            --uri "${{ secrets.PRODUCTION_DATABASE_URL }}" \
            --direction down \
            --to-version ${{ steps.migration.outputs.previous_version }}
```

---

## 6. Documentation Generation

### 6.1 API Documentation

**File:** `.github/workflows/docs-generation.yml`

```yaml
name: Documentation Generation

on:
  push:
    branches: [main]
    paths:
      - 'rust-core-engine/src/**'
      - 'python-ai-service/**/*.py'
      - 'nextjs-ui-dashboard/src/**'
      - 'docs/**'
  workflow_dispatch:

jobs:
  generate-rust-docs:
    name: Generate Rust Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Generate docs
        run: |
          cd rust-core-engine
          cargo doc --no-deps --document-private-items

      - name: Upload docs
        uses: actions/upload-artifact@v3
        with:
          name: rust-docs
          path: rust-core-engine/target/doc

  generate-python-docs:
    name: Generate Python Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Setup Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'

      - name: Install dependencies
        run: |
          cd python-ai-service
          pip install sphinx sphinx-rtd-theme

      - name: Generate docs
        run: |
          cd python-ai-service
          sphinx-apidoc -o docs/source .
          sphinx-build -b html docs/source docs/build

      - name: Upload docs
        uses: actions/upload-artifact@v3
        with:
          name: python-docs
          path: python-ai-service/docs/build

  generate-api-specs:
    name: Generate API Specifications
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Generate OpenAPI spec
        run: |
          docker compose up -d rust-core-engine python-ai-service
          sleep 30
          curl http://localhost:8080/api/openapi.json > rust-api-spec.json
          curl http://localhost:8000/openapi.json > python-api-spec.json

      - name: Validate OpenAPI specs
        run: |
          npx @apidevtools/swagger-cli validate rust-api-spec.json
          npx @apidevtools/swagger-cli validate python-api-spec.json

      - name: Generate API docs
        run: |
          npx @redocly/cli build-docs rust-api-spec.json -o rust-api-docs.html
          npx @redocly/cli build-docs python-api-spec.json -o python-api-docs.html

      - name: Upload specs
        uses: actions/upload-artifact@v3
        with:
          name: api-specs
          path: '*.json'

      - name: Upload docs
        uses: actions/upload-artifact@v3
        with:
          name: api-docs
          path: '*.html'

  publish-docs:
    name: Publish Documentation
    runs-on: ubuntu-latest
    needs: [generate-rust-docs, generate-python-docs, generate-api-specs]
    steps:
      - uses: actions/checkout@v3

      - name: Download all artifacts
        uses: actions/download-artifact@v3

      - name: Organize documentation
        run: |
          mkdir -p public/rust
          mkdir -p public/python
          mkdir -p public/api
          cp -r rust-docs/* public/rust/
          cp -r python-docs/* public/python/
          cp api-docs/*.html public/api/
          cp api-specs/*.json public/api/

      - name: Deploy to GitHub Pages
        uses: peaceiris/actions-gh-pages@v3
        with:
          github_token: ${{ secrets.GITHUB_TOKEN }}
          publish_dir: ./public
          cname: docs.botcore.app
```

---

## 7. Release Management

### 7.1 Semantic Release

**File:** `.github/workflows/release.yml`

```yaml
name: Release Management

on:
  push:
    branches: [main]

jobs:
  release:
    name: Create Release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
        with:
          fetch-depth: 0

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install semantic-release
        run: |
          npm install -g semantic-release \
            @semantic-release/changelog \
            @semantic-release/git \
            @semantic-release/github

      - name: Create release
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
        run: npx semantic-release

      - name: Get version
        id: version
        run: echo "version=$(cat package.json | jq -r .version)" >> $GITHUB_OUTPUT

      - name: Tag Docker images
        run: |
          docker tag rust-core-engine:latest rust-core-engine:${{ steps.version.outputs.version }}
          docker tag python-ai-service:latest python-ai-service:${{ steps.version.outputs.version }}
          docker tag nextjs-ui-dashboard:latest nextjs-ui-dashboard:${{ steps.version.outputs.version }}

      - name: Push tagged images
        run: |
          docker push rust-core-engine:${{ steps.version.outputs.version }}
          docker push python-ai-service:${{ steps.version.outputs.version }}
          docker push nextjs-ui-dashboard:${{ steps.version.outputs.version }}

      - name: Create changelog
        run: |
          git log $(git describe --tags --abbrev=0 HEAD^)..HEAD --oneline > CHANGELOG.md

      - name: Notify Slack
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "New release: v${{ steps.version.outputs.version }}",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": ":rocket: *New Release: v${{ steps.version.outputs.version }}*\n\nCheck the changelog for details."
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | DevOps Team | Initial version |

---

**Document End**
