# Docker Registry Setup Guide

Complete guide for setting up and managing Docker Registry for Bot-Core production deployments.

---

## Table of Contents

- [Overview](#overview)
- [Quick Start](#quick-start)
- [Registry Options](#registry-options)
  - [GitHub Container Registry (Recommended)](#1-github-container-registry-recommended)
  - [Docker Hub](#2-docker-hub)
  - [AWS Elastic Container Registry (ECR)](#3-aws-elastic-container-registry-ecr)
  - [Google Container Registry (GCR)](#4-google-container-registry-gcr)
  - [Azure Container Registry (ACR)](#5-azure-container-registry-acr)
  - [Private Registry](#6-private-registry)
- [Image Versioning Strategy](#image-versioning-strategy)
- [CI/CD Integration](#cicd-integration)
- [Production Deployment](#production-deployment)
- [Troubleshooting](#troubleshooting)
- [Security Best Practices](#security-best-practices)

---

## Overview

Bot-Core uses Docker images for production deployments. Images are:
- Built automatically via GitHub Actions
- Tagged with version, git SHA, and 'latest'
- Scanned for vulnerabilities with Trivy
- Signed with Cosign for security
- Stored in container registry (GitHub, Docker Hub, AWS, etc.)

**Services:**
- `rust-core-engine` - Rust trading engine
- `python-ai-service` - Python AI/ML service
- `nextjs-ui-dashboard` - Next.js frontend

---

## Quick Start

### 1. Configure Registry

```bash
# Copy environment template
cp .env.example .env

# Edit .env and set:
# DOCKER_REGISTRY=ghcr.io/your-username/bot-core
# DOCKER_USERNAME=your-github-username
# DOCKER_PASSWORD=your-github-token
nano .env
```

### 2. Setup Authentication

```bash
# Run setup script
./scripts/docker-registry-setup.sh

# Script will:
# - Detect registry type
# - Authenticate to registry
# - Test push/pull permissions
```

### 3. Build and Push Images

```bash
# Build all services and push to registry
./scripts/build-and-push.sh

# Build specific service only
./scripts/build-and-push.sh --service rust-core-engine

# Build without pushing (local testing)
./scripts/build-and-push.sh --no-push

# Build with vulnerability scanning
./scripts/build-and-push.sh --scan
```

### 4. Pull Images for Production

```bash
# Pull latest images
./scripts/pull-images.sh

# Pull specific version
./scripts/pull-images.sh --version v1.0.0

# List available versions
./scripts/pull-images.sh --list

# Pull and cleanup old versions
./scripts/pull-images.sh --cleanup
```

### 5. Deploy to Production

```bash
# Deploy with pulled images
VERSION=v1.0.0 docker-compose -f docker-compose.prod.yml up -d

# Or use deployment script
./scripts/deploy.sh
```

---

## Registry Options

### 1. GitHub Container Registry (Recommended)

**Benefits:**
- Free for public repositories
- Free for private repositories with GitHub Free
- Unlimited bandwidth
- Integrated with GitHub Actions
- Automatic vulnerability scanning

**Setup:**

1. **Create Personal Access Token:**
   - Go to: https://github.com/settings/tokens
   - Click "Generate new token (classic)"
   - Select scopes:
     - `write:packages`
     - `read:packages`
     - `delete:packages`
   - Copy token (you won't see it again!)

2. **Configure .env:**

```bash
DOCKER_REGISTRY=ghcr.io/your-username/bot-core
DOCKER_USERNAME=your-github-username
DOCKER_PASSWORD=ghp_xxxxxxxxxxxxxxxxxxxxxxxxxxxx
VERSION=latest
IMAGE_PULL_POLICY=Always
```

3. **Authenticate:**

```bash
./scripts/docker-registry-setup.sh
```

4. **Make Repository Public (Optional):**
   - Go to: https://github.com/users/your-username/packages/container/bot-core
   - Click "Package settings"
   - Change visibility to "Public" (for free unlimited pulls)

**Image URLs:**
```
ghcr.io/your-username/bot-core/rust-core-engine:latest
ghcr.io/your-username/bot-core/python-ai-service:v1.0.0
ghcr.io/your-username/bot-core/nextjs-ui-dashboard:main-abc123
```

---

### 2. Docker Hub

**Benefits:**
- Most popular registry
- Free tier: 1 private repository, unlimited public
- Well-established ecosystem

**Limits:**
- Free tier: 200 pulls/6 hours (unauthenticated)
- 5,000 pulls/day (authenticated free tier)

**Setup:**

1. **Create Docker Hub Account:**
   - Go to: https://hub.docker.com
   - Sign up for free account

2. **Create Access Token:**
   - Go to: https://hub.docker.com/settings/security
   - Click "New Access Token"
   - Name: "bot-core-ci"
   - Permissions: Read, Write, Delete
   - Copy token

3. **Configure .env:**

```bash
DOCKER_REGISTRY=docker.io/your-dockerhub-username/bot-core
# Or just: your-dockerhub-username/bot-core
DOCKER_USERNAME=your-dockerhub-username
DOCKER_PASSWORD=dckr_pat_xxxxxxxxxxxxxxxxxxxx
VERSION=latest
```

4. **Authenticate:**

```bash
./scripts/docker-registry-setup.sh
```

**Image URLs:**
```
your-username/bot-core/rust-core-engine:latest
docker.io/your-username/bot-core/python-ai-service:v1.0.0
```

---

### 3. AWS Elastic Container Registry (ECR)

**Benefits:**
- Integrated with AWS ecosystem
- High security and compliance
- Private by default
- Automatic vulnerability scanning

**Pricing:**
- $0.10 per GB/month storage
- $0.09 per GB data transfer

**Setup:**

1. **Install AWS CLI:**

```bash
# macOS
brew install awscli

# Linux
pip install awscli

# Configure credentials
aws configure
```

2. **Create ECR Repositories:**

```bash
# Set region
AWS_REGION=us-east-1

# Create repositories
aws ecr create-repository --repository-name bot-core/rust-core-engine --region $AWS_REGION
aws ecr create-repository --repository-name bot-core/python-ai-service --region $AWS_REGION
aws ecr create-repository --repository-name bot-core/nextjs-ui-dashboard --region $AWS_REGION

# Enable scan on push
aws ecr put-image-scanning-configuration \
  --repository-name bot-core/rust-core-engine \
  --image-scanning-configuration scanOnPush=true \
  --region $AWS_REGION
```

3. **Configure .env:**

```bash
DOCKER_REGISTRY=123456789012.dkr.ecr.us-east-1.amazonaws.com/bot-core
DOCKER_USERNAME=AWS
DOCKER_PASSWORD=  # Not needed, uses AWS credentials
VERSION=latest
```

4. **Authenticate:**

```bash
./scripts/docker-registry-setup.sh
```

**Note:** ECR authentication expires after 12 hours. Re-run setup script or use:

```bash
aws ecr get-login-password --region us-east-1 | \
  docker login --username AWS --password-stdin \
  123456789012.dkr.ecr.us-east-1.amazonaws.com
```

**Image URLs:**
```
123456789012.dkr.ecr.us-east-1.amazonaws.com/bot-core/rust-core-engine:latest
```

---

### 4. Google Container Registry (GCR)

**Benefits:**
- Integrated with Google Cloud
- Fast in GCP regions
- Automatic vulnerability scanning

**Pricing:**
- Storage: $0.026 per GB/month
- Network egress: varies by region

**Setup:**

1. **Install Google Cloud SDK:**

```bash
# macOS
brew install google-cloud-sdk

# Authenticate
gcloud auth login
gcloud auth configure-docker
```

2. **Configure .env:**

```bash
DOCKER_REGISTRY=gcr.io/your-project-id/bot-core
DOCKER_USERNAME=  # Not needed
DOCKER_PASSWORD=  # Not needed, uses gcloud credentials
VERSION=latest
```

3. **Authenticate:**

```bash
./scripts/docker-registry-setup.sh
```

**Image URLs:**
```
gcr.io/your-project-id/bot-core/rust-core-engine:latest
```

---

### 5. Azure Container Registry (ACR)

**Benefits:**
- Integrated with Azure
- Geo-replication
- Security scanning

**Pricing:**
- Basic: $5/month (10 GB storage)
- Standard: $20/month (100 GB storage)
- Premium: $50/month (500 GB storage)

**Setup:**

1. **Install Azure CLI:**

```bash
# macOS
brew install azure-cli

# Authenticate
az login
```

2. **Create ACR:**

```bash
# Create resource group
az group create --name bot-core-rg --location eastus

# Create registry
az acr create --resource-group bot-core-rg \
  --name botcoreregistry --sku Basic

# Enable admin user
az acr update -n botcoreregistry --admin-enabled true
```

3. **Configure .env:**

```bash
DOCKER_REGISTRY=botcoreregistry.azurecr.io/bot-core
DOCKER_USERNAME=botcoreregistry
DOCKER_PASSWORD=  # Get from: az acr credential show -n botcoreregistry
VERSION=latest
```

4. **Authenticate:**

```bash
./scripts/docker-registry-setup.sh
```

**Image URLs:**
```
botcoreregistry.azurecr.io/bot-core/rust-core-engine:latest
```

---

### 6. Private Registry

For self-hosted registries (Harbor, Nexus, etc.)

**Setup:**

1. **Configure .env:**

```bash
DOCKER_REGISTRY=registry.yourcompany.com/bot-core
DOCKER_USERNAME=your-username
DOCKER_PASSWORD=your-password
VERSION=latest
```

2. **Authenticate:**

```bash
./scripts/docker-registry-setup.sh
```

**Image URLs:**
```
registry.yourcompany.com/bot-core/rust-core-engine:latest
```

---

## Image Versioning Strategy

### Tagging Scheme

Each image is tagged with multiple tags for flexibility:

1. **`latest`** - Always points to most recent build from main branch
2. **`v1.0.0`** - Semantic version (from git tags)
3. **`main-abc123`** - Branch name + short git SHA
4. **`abc123`** - Short git SHA only

### Examples

```bash
# Build from main branch, commit abc123, tagged v1.0.0
ghcr.io/username/bot-core/rust-core-engine:latest
ghcr.io/username/bot-core/rust-core-engine:v1.0.0
ghcr.io/username/bot-core/rust-core-engine:main-abc123
ghcr.io/username/bot-core/rust-core-engine:abc123

# Build from develop branch, commit def456
ghcr.io/username/bot-core/rust-core-engine:develop
ghcr.io/username/bot-core/rust-core-engine:develop-def456
ghcr.io/username/bot-core/rust-core-engine:def456
```

### Best Practices

**Production:**
```bash
# Use semantic version tags
VERSION=v1.0.0 docker-compose -f docker-compose.prod.yml up -d
```

**Staging:**
```bash
# Use branch tags
VERSION=develop docker-compose -f docker-compose.prod.yml up -d
```

**Testing:**
```bash
# Use git SHA for specific commits
VERSION=abc123 docker-compose -f docker-compose.prod.yml up -d
```

**Never in production:**
```bash
# DON'T use 'latest' in production (unpredictable)
VERSION=latest docker-compose -f docker-compose.prod.yml up -d
```

---

## CI/CD Integration

### GitHub Actions

The `.github/workflows/docker-build-push.yml` workflow automatically:
- Builds images on push to main/develop
- Tags images appropriately
- Pushes to GitHub Container Registry
- Scans for vulnerabilities with Trivy
- Generates SBOM (Software Bill of Materials)
- Signs images with Cosign

**Triggers:**
- Push to main/develop branches
- Pull requests (builds but doesn't push)
- Git tags matching `v*` pattern
- Manual workflow dispatch

**Configuration:**

Set these GitHub Secrets:
- `GITHUB_TOKEN` - Automatically provided
- `DOCKER_USERNAME` - Optional, for other registries
- `DOCKER_PASSWORD` - Optional, for other registries

**Usage:**

```bash
# Tag a release
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions will:
# 1. Build all images
# 2. Tag with: v1.0.0, v1.0, v1, latest
# 3. Push to registry
# 4. Scan for vulnerabilities
# 5. Comment on PR/commit with results
```

### Manual Workflow Trigger

```bash
# Via GitHub CLI
gh workflow run docker-build-push.yml

# Via GitHub web interface
# Go to Actions → Docker Build and Push → Run workflow
```

---

## Production Deployment

### Step-by-Step Deployment

1. **Pull Latest Images:**

```bash
# Authenticate to registry
./scripts/docker-registry-setup.sh

# Pull production images
./scripts/pull-images.sh --version v1.0.0
```

2. **Verify Images:**

```bash
# List pulled images
docker images | grep bot-core

# Inspect image details
docker inspect ghcr.io/username/bot-core/rust-core-engine:v1.0.0
```

3. **Deploy:**

```bash
# Deploy all services
VERSION=v1.0.0 docker-compose -f docker-compose.prod.yml up -d

# Check status
docker-compose -f docker-compose.prod.yml ps

# View logs
docker-compose -f docker-compose.prod.yml logs -f
```

### Rolling Updates

```bash
# Update to new version
VERSION=v1.1.0 ./scripts/pull-images.sh
VERSION=v1.1.0 docker-compose -f docker-compose.prod.yml up -d

# Docker Compose will:
# 1. Start new containers with new version
# 2. Wait for health checks
# 3. Stop old containers
# 4. Remove old containers
```

### Rollback

```bash
# Rollback to previous version
VERSION=v1.0.0 docker-compose -f docker-compose.prod.yml up -d

# Or use specific git SHA
VERSION=abc123 docker-compose -f docker-compose.prod.yml up -d
```

---

## Troubleshooting

### Common Issues

#### 1. Authentication Failed

**Error:**
```
Error response from daemon: unauthorized: authentication required
```

**Solution:**
```bash
# Re-run setup script
./scripts/docker-registry-setup.sh

# Or manually login
echo $DOCKER_PASSWORD | docker login ghcr.io -u $DOCKER_USERNAME --password-stdin
```

#### 2. Rate Limit Exceeded (Docker Hub)

**Error:**
```
You have reached your pull rate limit
```

**Solution:**
```bash
# Authenticate to increase limit
docker login

# Or switch to GitHub Container Registry (unlimited)
DOCKER_REGISTRY=ghcr.io/username/bot-core
```

#### 3. Image Not Found

**Error:**
```
Error response from daemon: manifest for image:tag not found
```

**Solution:**
```bash
# List available versions
./scripts/pull-images.sh --list

# Check registry web interface
# GitHub: https://github.com/users/USERNAME/packages
# Docker Hub: https://hub.docker.com/r/USERNAME/REPO
```

#### 4. Build Failed

**Error:**
```
ERROR: failed to solve: process "/bin/sh -c cargo build --release" did not complete successfully
```

**Solution:**
```bash
# Build locally to see full error
cd rust-core-engine
docker build -t test .

# Check Dockerfile and dependencies
# Increase Docker memory/CPU limits in Docker Desktop
```

#### 5. Push Failed

**Error:**
```
denied: requested access to the resource is denied
```

**Solution:**
```bash
# Check repository exists
# For GitHub: create package at github.com/users/USERNAME/packages

# Check permissions
# For GitHub: token needs write:packages scope

# Check registry URL matches .env
echo $DOCKER_REGISTRY
```

---

## Security Best Practices

### 1. Credential Management

**Never commit credentials:**
```bash
# Add to .gitignore
echo ".env" >> .gitignore

# Use environment variables
export DOCKER_PASSWORD="secret-token"
```

**Use tokens, not passwords:**
- GitHub: Personal Access Tokens
- Docker Hub: Access Tokens
- AWS: IAM roles/temporary credentials

### 2. Image Scanning

**Scan before deploying:**
```bash
# Scan with Trivy (installed automatically in CI)
./scripts/build-and-push.sh --scan

# Or manually
trivy image ghcr.io/username/bot-core/rust-core-engine:v1.0.0
```

**Address vulnerabilities:**
```bash
# Update base images in Dockerfiles
FROM rust:1.86-alpine  # Use specific versions

# Update dependencies
cargo update
pip install --upgrade package
npm update
```

### 3. Image Signing

**Verify signatures:**
```bash
# Pull with signature verification
./scripts/pull-images.sh --verify

# Or manually with Cosign
cosign verify ghcr.io/username/bot-core/rust-core-engine:v1.0.0
```

### 4. Least Privilege

**Use minimal permissions:**
```bash
# Run containers as non-root
USER 1000:1000

# Read-only root filesystem
docker run --read-only

# Drop capabilities
docker run --cap-drop=ALL
```

### 5. Network Security

**Private registries:**
```bash
# Use VPN for private registry access
# Use registry mirrors in same cloud region
# Enable HTTPS only
```

### 6. Registry Access Control

**Limit access:**
- Use private repositories
- Enable IP whitelisting
- Use service accounts for CI/CD
- Rotate credentials regularly

---

## Additional Resources

### Official Documentation

- **GitHub Container Registry:** https://docs.github.com/en/packages/working-with-a-github-packages-registry/working-with-the-container-registry
- **Docker Hub:** https://docs.docker.com/docker-hub/
- **AWS ECR:** https://docs.aws.amazon.com/ecr/
- **Google GCR:** https://cloud.google.com/container-registry/docs
- **Azure ACR:** https://docs.microsoft.com/en-us/azure/container-registry/

### Tools

- **Trivy (Vulnerability Scanner):** https://aquasecurity.github.io/trivy/
- **Cosign (Image Signing):** https://docs.sigstore.dev/cosign/overview/
- **Docker Buildx:** https://docs.docker.com/buildx/working-with-buildx/
- **SBOM Generator:** https://github.com/anchore/syft

### Scripts Reference

- `./scripts/docker-registry-setup.sh` - Setup authentication
- `./scripts/build-and-push.sh` - Build and push images
- `./scripts/pull-images.sh` - Pull images from registry
- `./scripts/deploy.sh` - Deploy to production

---

**Last Updated:** 2025-11-18
**Maintained By:** Bot-Core Team
**Support:** See CONTRIBUTING.md for help
