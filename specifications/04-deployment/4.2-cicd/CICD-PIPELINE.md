# CI/CD Pipeline Specification

**Document Version:** 1.0.0
**Last Updated:** 2025-10-11
**Status:** Active
**Owner:** DevOps Team

---

## Table of Contents

- [1. Overview](#1-overview)
- [2. Pipeline Architecture](#2-pipeline-architecture)
- [3. GitHub Actions Workflows](#3-github-actions-workflows)
- [4. Build Pipeline](#4-build-pipeline)
- [5. Test Pipeline](#5-test-pipeline)
- [6. Deployment Pipeline](#6-deployment-pipeline)
- [7. Security Scanning](#7-security-scanning)
- [8. Deployment Strategies](#8-deployment-strategies)
- [9. Rollback Procedures](#9-rollback-procedures)

---

## 1. Overview

### 1.1 Purpose

This document defines the CI/CD pipeline design for automated building, testing, and deployment of the Bot Core platform.

### 1.2 Pipeline Goals

- Automated testing on every pull request
- Automated deployment to staging on merge to main
- Manual approval for production deployment
- Zero-downtime deployments
- Automatic rollback on failure

---

## 2. Pipeline Architecture

```mermaid
graph LR
    A[Git Push] --> B[GitHub Actions]
    B --> C{Branch?}
    C -->|feature| D[CI Pipeline]
    C -->|main| E[CI + CD Pipeline]

    D --> F[Lint & Format]
    F --> G[Unit Tests]
    G --> H[Integration Tests]
    H --> I[Build Docker Images]
    I --> J[Security Scan]
    J --> K{Pass?}
    K -->|Yes| L[Ready for Review]
    K -->|No| M[Notify Developer]

    E --> F
    J --> N{Pass?}
    N -->|Yes| O[Push to Registry]
    O --> P[Deploy to Staging]
    P --> Q[E2E Tests]
    Q --> R{Tests Pass?}
    R -->|Yes| S{Manual Approval}
    R -->|No| T[Rollback Staging]
    S -->|Approved| U[Deploy to Production]
    S -->|Rejected| V[Stop]
    U --> W[Health Checks]
    W --> X{Healthy?}
    X -->|Yes| Y[Complete]
    X -->|No| Z[Rollback Production]
```

---

## 3. GitHub Actions Workflows

### 3.1 CI Workflow

**File:** `.github/workflows/ci.yml`

```yaml
name: Continuous Integration

on:
  pull_request:
    branches: [main, develop]
  push:
    branches: [main, develop]

env:
  DOCKER_BUILDKIT: 1
  COMPOSE_DOCKER_CLI_BUILD: 1

jobs:
  # Rust Service CI
  rust-ci:
    name: Rust CI
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ./rust-core-engine

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          components: rustfmt, clippy

      - name: Cache Cargo registry
        uses: actions/cache@v3
        with:
          path: ~/.cargo/registry
          key: ${{ runner.os }}-cargo-registry-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache Cargo index
        uses: actions/cache@v3
        with:
          path: ~/.cargo/git
          key: ${{ runner.os }}-cargo-git-${{ hashFiles('**/Cargo.lock') }}

      - name: Cache build artifacts
        uses: actions/cache@v3
        with:
          path: rust-core-engine/target
          key: ${{ runner.os }}-cargo-target-${{ hashFiles('**/Cargo.lock') }}

      - name: Check formatting
        run: cargo fmt -- --check

      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

      - name: Run tests
        run: cargo test --all-features --verbose

      - name: Build release
        run: cargo build --release --verbose

      - name: Upload artifacts
        uses: actions/upload-artifact@v3
        with:
          name: rust-binary
          path: rust-core-engine/target/release/rust-core-engine
          retention-days: 7

  # Python Service CI
  python-ci:
    name: Python CI
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ./python-ai-service

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Python 3.11
        uses: actions/setup-python@v4
        with:
          python-version: '3.11'
          cache: 'pip'

      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -r requirements.txt
          pip install -r requirements.test.txt

      - name: Check code formatting with black
        run: black --check .

      - name: Run flake8 linter
        run: flake8 . --count --select=E9,F63,F7,F82 --show-source --statistics

      - name: Run pylint
        run: pylint **/*.py --fail-under=8.0

      - name: Run mypy type checker
        run: mypy . --ignore-missing-imports

      - name: Run pytest
        run: pytest tests/ --cov=. --cov-report=xml --cov-report=html --verbose

      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          files: ./python-ai-service/coverage.xml
          flags: python
          name: python-coverage

      - name: Generate coverage report
        run: coverage report

  # Frontend CI
  frontend-ci:
    name: Frontend CI
    runs-on: ubuntu-latest
    defaults:
      run:
        working-directory: ./nextjs-ui-dashboard

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Node.js 18
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
          cache-dependency-path: nextjs-ui-dashboard/package-lock.json

      - name: Install dependencies
        run: npm ci

      - name: Run ESLint
        run: npm run lint

      - name: Check TypeScript types
        run: npm run type-check

      - name: Run tests
        run: npm test -- --coverage --watchAll=false

      - name: Build application
        run: npm run build
        env:
          NODE_ENV: production

      - name: Upload build artifacts
        uses: actions/upload-artifact@v3
        with:
          name: frontend-dist
          path: nextjs-ui-dashboard/dist
          retention-days: 7

      - name: Upload coverage
        uses: codecov/codecov-action@v3
        with:
          files: ./nextjs-ui-dashboard/coverage/lcov.info
          flags: frontend
          name: frontend-coverage

  # Integration Tests
  integration-tests:
    name: Integration Tests
    runs-on: ubuntu-latest
    needs: [rust-ci, python-ci, frontend-ci]

    services:
      mongodb:
        image: mongo:7.0
        ports:
          - 27017:27017
        env:
          MONGO_INITDB_ROOT_USERNAME: testuser
          MONGO_INITDB_ROOT_PASSWORD: testpass
        options: >-
          --health-cmd "mongosh --eval 'db.adminCommand({ping: 1})'"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup environment
        run: |
          cp config.env .env
          echo "DATABASE_URL=mongodb://testuser:testpass@localhost:27017/test_db?authSource=admin" >> .env
          echo "TRADING_ENABLED=false" >> .env
          echo "BINANCE_TESTNET=true" >> .env

      - name: Build services
        run: docker compose --profile dev build

      - name: Start services
        run: docker compose --profile dev up -d

      - name: Wait for services
        run: |
          timeout 120 bash -c 'until curl -sf http://localhost:8080/api/health; do sleep 2; done'
          timeout 120 bash -c 'until curl -sf http://localhost:8000/health; do sleep 2; done'
          timeout 120 bash -c 'until curl -sf http://localhost:3000/; do sleep 2; done'

      - name: Run integration tests
        run: |
          cd rust-core-engine && cargo test test_ai_service_integration -- --test-threads=1
          cd ../nextjs-ui-dashboard && npm run test:integration

      - name: Collect logs on failure
        if: failure()
        run: docker compose logs > integration-tests.log

      - name: Upload logs
        if: failure()
        uses: actions/upload-artifact@v3
        with:
          name: integration-test-logs
          path: integration-tests.log

      - name: Stop services
        if: always()
        run: docker compose down -v

  # Docker Build
  docker-build:
    name: Build Docker Images
    runs-on: ubuntu-latest
    needs: [rust-ci, python-ci, frontend-ci]
    if: github.event_name == 'push' && github.ref == 'refs/heads/main'

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v2

      - name: Login to Docker Registry
        uses: docker/login-action@v2
        with:
          registry: ${{ secrets.DOCKER_REGISTRY }}
          username: ${{ secrets.DOCKER_USERNAME }}
          password: ${{ secrets.DOCKER_PASSWORD }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v4
        with:
          images: |
            ${{ secrets.DOCKER_REGISTRY }}/rust-core-engine
            ${{ secrets.DOCKER_REGISTRY }}/python-ai-service
            ${{ secrets.DOCKER_REGISTRY }}/nextjs-ui-dashboard
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=semver,pattern={{version}}
            type=semver,pattern={{major}}.{{minor}}
            type=sha

      - name: Build and push Rust image
        uses: docker/build-push-action@v4
        with:
          context: ./rust-core-engine
          push: true
          tags: ${{ secrets.DOCKER_REGISTRY }}/rust-core-engine:${{ github.sha }}
          cache-from: type=registry,ref=${{ secrets.DOCKER_REGISTRY }}/rust-core-engine:buildcache
          cache-to: type=registry,ref=${{ secrets.DOCKER_REGISTRY }}/rust-core-engine:buildcache,mode=max

      - name: Build and push Python image
        uses: docker/build-push-action@v4
        with:
          context: ./python-ai-service
          push: true
          tags: ${{ secrets.DOCKER_REGISTRY }}/python-ai-service:${{ github.sha }}
          cache-from: type=registry,ref=${{ secrets.DOCKER_REGISTRY }}/python-ai-service:buildcache
          cache-to: type=registry,ref=${{ secrets.DOCKER_REGISTRY }}/python-ai-service:buildcache,mode=max

      - name: Build and push Frontend image
        uses: docker/build-push-action@v4
        with:
          context: ./nextjs-ui-dashboard
          push: true
          tags: ${{ secrets.DOCKER_REGISTRY }}/nextjs-ui-dashboard:${{ github.sha }}
          cache-from: type=registry,ref=${{ secrets.DOCKER_REGISTRY }}/nextjs-ui-dashboard:buildcache
          cache-to: type=registry,ref=${{ secrets.DOCKER_REGISTRY }}/nextjs-ui-dashboard:buildcache,mode=max

  # Security Scanning
  security-scan:
    name: Security Scanning
    runs-on: ubuntu-latest
    needs: [docker-build]

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Run Trivy vulnerability scanner - Rust
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ secrets.DOCKER_REGISTRY }}/rust-core-engine:${{ github.sha }}
          format: 'sarif'
          output: 'rust-trivy-results.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Run Trivy vulnerability scanner - Python
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ secrets.DOCKER_REGISTRY }}/python-ai-service:${{ github.sha }}
          format: 'sarif'
          output: 'python-trivy-results.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Run Trivy vulnerability scanner - Frontend
        uses: aquasecurity/trivy-action@master
        with:
          image-ref: ${{ secrets.DOCKER_REGISTRY }}/nextjs-ui-dashboard:${{ github.sha }}
          format: 'sarif'
          output: 'frontend-trivy-results.sarif'
          severity: 'CRITICAL,HIGH'

      - name: Upload Trivy results to GitHub Security
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: '.'

      - name: Run Snyk security scan
        uses: snyk/actions/docker@master
        env:
          SNYK_TOKEN: ${{ secrets.SNYK_TOKEN }}
        with:
          image: ${{ secrets.DOCKER_REGISTRY }}/rust-core-engine:${{ github.sha }}
          args: --severity-threshold=high
```

### 3.2 CD Workflow

**File:** `.github/workflows/cd.yml`

```yaml
name: Continuous Deployment

on:
  workflow_run:
    workflows: ["Continuous Integration"]
    types:
      - completed
    branches: [main]

env:
  KUBECONFIG_STAGING: ${{ secrets.KUBECONFIG_STAGING }}
  KUBECONFIG_PROD: ${{ secrets.KUBECONFIG_PROD }}

jobs:
  deploy-staging:
    name: Deploy to Staging
    runs-on: ubuntu-latest
    if: ${{ github.event.workflow_run.conclusion == 'success' }}
    environment:
      name: staging
      url: https://staging.botcore.app

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup kubectl
        uses: azure/setup-kubectl@v3
        with:
          version: 'v1.27.0'

      - name: Configure kubectl for staging
        run: |
          mkdir -p $HOME/.kube
          echo "${{ secrets.KUBECONFIG_STAGING }}" | base64 -d > $HOME/.kube/config
          chmod 600 $HOME/.kube/config

      - name: Update image tags
        run: |
          cd infrastructure/kubernetes/overlays/staging
          kustomize edit set image \
            rust-core-engine=${{ secrets.DOCKER_REGISTRY }}/rust-core-engine:${{ github.sha }} \
            python-ai-service=${{ secrets.DOCKER_REGISTRY }}/python-ai-service:${{ github.sha }} \
            nextjs-ui-dashboard=${{ secrets.DOCKER_REGISTRY }}/nextjs-ui-dashboard:${{ github.sha }}

      - name: Deploy to staging
        run: |
          kubectl apply -k infrastructure/kubernetes/overlays/staging/
          kubectl rollout status deployment/rust-core-engine -n bot-core-staging --timeout=5m
          kubectl rollout status deployment/python-ai-service -n bot-core-staging --timeout=5m
          kubectl rollout status deployment/nextjs-ui-dashboard -n bot-core-staging --timeout=5m

      - name: Run smoke tests
        run: |
          STAGING_URL="https://staging.botcore.app"
          curl -f $STAGING_URL/health || exit 1
          curl -f $STAGING_URL/api/health || exit 1

      - name: Notify Slack on success
        if: success()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "Staging deployment successful for commit ${{ github.sha }}",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": ":white_check_mark: *Staging Deployment Successful*\nCommit: `${{ github.sha }}`\nEnvironment: `staging`"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}

      - name: Rollback on failure
        if: failure()
        run: |
          kubectl rollout undo deployment/rust-core-engine -n bot-core-staging
          kubectl rollout undo deployment/python-ai-service -n bot-core-staging
          kubectl rollout undo deployment/nextjs-ui-dashboard -n bot-core-staging

      - name: Notify Slack on failure
        if: failure()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "Staging deployment failed for commit ${{ github.sha }}",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": ":x: *Staging Deployment Failed*\nCommit: `${{ github.sha }}`\nCheck logs: ${{ github.server_url }}/${{ github.repository }}/actions/runs/${{ github.run_id }}"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}

  e2e-tests:
    name: E2E Tests on Staging
    runs-on: ubuntu-latest
    needs: [deploy-staging]

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'

      - name: Install Playwright
        run: |
          cd nextjs-ui-dashboard
          npm ci
          npx playwright install --with-deps

      - name: Run E2E tests
        run: |
          cd nextjs-ui-dashboard
          npm run test:e2e
        env:
          BASE_URL: https://staging.botcore.app

      - name: Upload test results
        if: always()
        uses: actions/upload-artifact@v3
        with:
          name: e2e-test-results
          path: nextjs-ui-dashboard/test-results/
          retention-days: 7

  deploy-production:
    name: Deploy to Production
    runs-on: ubuntu-latest
    needs: [e2e-tests]
    environment:
      name: production
      url: https://botcore.app

    steps:
      - name: Checkout code
        uses: actions/checkout@v3

      - name: Setup kubectl
        uses: azure/setup-kubectl@v3
        with:
          version: 'v1.27.0'

      - name: Configure kubectl for production
        run: |
          mkdir -p $HOME/.kube
          echo "${{ secrets.KUBECONFIG_PROD }}" | base64 -d > $HOME/.kube/config
          chmod 600 $HOME/.kube/config

      - name: Create deployment backup
        run: |
          kubectl get deployment rust-core-engine -n bot-core-production -o yaml > rust-backup.yaml
          kubectl get deployment python-ai-service -n bot-core-production -o yaml > python-backup.yaml
          kubectl get deployment nextjs-ui-dashboard -n bot-core-production -o yaml > frontend-backup.yaml

      - name: Update image tags
        run: |
          cd infrastructure/kubernetes/overlays/production
          kustomize edit set image \
            rust-core-engine=${{ secrets.DOCKER_REGISTRY }}/rust-core-engine:${{ github.sha }} \
            python-ai-service=${{ secrets.DOCKER_REGISTRY }}/python-ai-service:${{ github.sha }} \
            nextjs-ui-dashboard=${{ secrets.DOCKER_REGISTRY }}/nextjs-ui-dashboard:${{ github.sha }}

      - name: Deploy to production
        run: |
          kubectl apply -k infrastructure/kubernetes/overlays/production/
          kubectl rollout status deployment/rust-core-engine -n bot-core-production --timeout=10m
          kubectl rollout status deployment/python-ai-service -n bot-core-production --timeout=10m
          kubectl rollout status deployment/nextjs-ui-dashboard -n bot-core-production --timeout=10m

      - name: Health check
        run: |
          PROD_URL="https://botcore.app"
          for i in {1..10}; do
            if curl -f $PROD_URL/health && curl -f $PROD_URL/api/health; then
              echo "Health check passed"
              exit 0
            fi
            echo "Health check attempt $i failed, retrying..."
            sleep 10
          done
          echo "Health check failed after 10 attempts"
          exit 1

      - name: Monitor error rate
        run: |
          # Query Prometheus for error rate
          ERROR_RATE=$(curl -s "http://prometheus:9090/api/v1/query?query=rate(http_requests_total{status=~\"5..\"}[5m])" | jq '.data.result[0].value[1]')
          if (( $(echo "$ERROR_RATE > 0.05" | bc -l) )); then
            echo "Error rate too high: $ERROR_RATE"
            exit 1
          fi

      - name: Notify Slack on success
        if: success()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "Production deployment successful!",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": ":rocket: *Production Deployment Successful*\nCommit: `${{ github.sha }}`\nDeployed by: ${{ github.actor }}"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}

      - name: Rollback on failure
        if: failure()
        run: |
          echo "Rolling back to previous version..."
          kubectl apply -f rust-backup.yaml
          kubectl apply -f python-backup.yaml
          kubectl apply -f frontend-backup.yaml
          kubectl rollout status deployment/rust-core-engine -n bot-core-production
          kubectl rollout status deployment/python-ai-service -n bot-core-production
          kubectl rollout status deployment/nextjs-ui-dashboard -n bot-core-production

      - name: Notify Slack on failure
        if: failure()
        uses: slackapi/slack-github-action@v1
        with:
          payload: |
            {
              "text": "Production deployment failed and rolled back",
              "blocks": [
                {
                  "type": "section",
                  "text": {
                    "type": "mrkdwn",
                    "text": ":warning: *Production Deployment Failed*\nCommit: `${{ github.sha }}`\nRolled back to previous version\n@here Please investigate"
                  }
                }
              ]
            }
        env:
          SLACK_WEBHOOK_URL: ${{ secrets.SLACK_WEBHOOK_URL }}
```

---

## 4. Build Pipeline

### 4.1 Build Stages

1. **Checkout**: Clone repository
2. **Cache**: Restore dependency cache
3. **Lint**: Code style checking
4. **Test**: Unit and integration tests
5. **Build**: Compile/build artifacts
6. **Security Scan**: Vulnerability scanning
7. **Push**: Upload to registry

### 4.2 Build Optimization

**Caching Strategy:**
- Cargo registry and build artifacts (Rust)
- pip packages (Python)
- npm modules (Frontend)
- Docker layer cache

**Parallel Execution:**
- All three services build in parallel
- Reduces total pipeline time from 30min to 10min

---

## 5. Test Pipeline

### 5.1 Test Levels

1. **Unit Tests**: Test individual functions
2. **Integration Tests**: Test service interactions
3. **E2E Tests**: Test complete user flows
4. **Performance Tests**: Load and stress testing
5. **Security Tests**: Vulnerability scanning

### 5.2 Test Coverage Requirements

- Rust: 80% coverage
- Python: 85% coverage
- Frontend: 75% coverage

---

## 6. Deployment Pipeline

### 6.1 Deployment Flow

1. **Staging Deployment** (automatic on merge to main)
2. **Smoke Tests** on staging
3. **E2E Tests** on staging
4. **Manual Approval** (via GitHub Environment)
5. **Production Deployment** (canary or blue-green)
6. **Health Checks**
7. **Monitoring** (error rates, latency)
8. **Rollback** if unhealthy

### 6.2 Deployment Gates

**Staging Gates:**
- All CI tests pass
- Security scans clean
- No critical vulnerabilities

**Production Gates:**
- Staging deployment successful
- E2E tests pass
- Manual approval granted
- Business hours only (optional)

---

## 7. Security Scanning

### 7.1 Static Analysis

- **Rust**: Clippy, cargo-audit
- **Python**: Bandit, safety
- **Frontend**: ESLint security rules, npm audit

### 7.2 Container Scanning

- **Trivy**: Vulnerability scanning
- **Snyk**: Dependency scanning
- **Grype**: Additional CVE detection

### 7.3 SAST (Static Application Security Testing)

- **SonarQube**: Code quality and security
- **CodeQL**: Advanced code analysis

---

## 8. Deployment Strategies

### 8.1 Rolling Update (Default)

```yaml
strategy:
  type: RollingUpdate
  rollingUpdate:
    maxSurge: 1
    maxUnavailable: 1
```

**Pros:** Simple, gradual rollout
**Cons:** Mixed versions running

### 8.2 Blue-Green Deployment

```bash
# Deploy green environment
kubectl apply -f green-deployment.yaml

# Switch traffic
kubectl patch service rust-core-engine -p '{"spec":{"selector":{"version":"green"}}}'

# Verify and cleanup blue
kubectl delete -f blue-deployment.yaml
```

**Pros:** Zero downtime, easy rollback
**Cons:** Requires 2x resources

### 8.3 Canary Deployment

```yaml
# Deploy canary (10% traffic)
apiVersion: networking.istio.io/v1beta1
kind: VirtualService
metadata:
  name: rust-core-engine
spec:
  hosts:
  - rust-core-engine
  http:
  - match:
    - headers:
        canary:
          exact: "true"
    route:
    - destination:
        host: rust-core-engine
        subset: canary
  - route:
    - destination:
        host: rust-core-engine
        subset: stable
      weight: 90
    - destination:
        host: rust-core-engine
        subset: canary
      weight: 10
```

**Pros:** Gradual rollout, risk mitigation
**Cons:** Complex, requires service mesh

---

## 9. Rollback Procedures

### 9.1 Automatic Rollback

Triggered by:
- Health check failures (3 consecutive)
- Error rate > 5%
- Latency p99 > 2s

```bash
kubectl rollout undo deployment/rust-core-engine -n bot-core-production
```

### 9.2 Manual Rollback

```bash
# List revisions
kubectl rollout history deployment/rust-core-engine -n bot-core-production

# Rollback to specific revision
kubectl rollout undo deployment/rust-core-engine --to-revision=3 -n bot-core-production

# Verify rollback
kubectl rollout status deployment/rust-core-engine -n bot-core-production
```

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0.0 | 2025-10-11 | DevOps Team | Initial version |

---

**Document End**
