# Bot Core Monorepo Makefile
# Common development and deployment tasks

.PHONY: help setup build start stop restart clean logs test lint docker-build docker-push deploy

# Default target
.DEFAULT_GOAL := help

# Variables
COMPOSE_FILE := infrastructure/docker/docker-compose.yml
SERVICES := rust-core-engine python-ai-service nextjs-ui-dashboard
DOCKER_REGISTRY := your-registry.com
DOCKER_TAG := latest

# Help target
help: ## Display this help message
	@echo "Bot Core Monorepo - Available commands:"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2}'
	@echo ""

# Setup and Configuration
setup: ## Initial setup - create directories and copy config files
	@echo "Setting up monorepo..."
	@mkdir -p rust-core-engine/data rust-core-engine/logs
	@mkdir -p python-ai-service/models python-ai-service/logs python-ai-service/data
	@mkdir -p nextjs-ui-dashboard/logs
	@if [ ! -f .env ]; then \
		echo "Creating .env file from example..."; \
		cp .env.example .env 2>/dev/null || echo "Please create .env file manually"; \
	fi
	@if [ ! -f rust-core-engine/config.toml ]; then \
		cp rust-core-engine/config.example.toml rust-core-engine/config.toml; \
	fi
	@echo "Setup complete!"

setup-dev: ## Setup development environment with hot reload
	@echo "Setting up development environment..."
	@chmod +x scripts/setup-dev.sh
	@./scripts/setup-dev.sh

# Docker Operations
build: ## Build all Docker images with optimized strategy
	@echo "Building Docker images with optimized strategy..."
	@chmod +x scripts/build-services.sh
	@./scripts/build-services.sh --build-only

build-fast: ## Build all Docker images sequentially (memory optimized)
	@echo "Building Docker images with memory optimization..."
	@chmod +x scripts/build-services.sh
	@./scripts/build-services.sh

build-clean: ## Clean build with cache reset
	@echo "Clean building all Docker images..."
	@chmod +x scripts/build-services.sh
	@./scripts/build-services.sh --clean-cache

build-rust: ## Build only Rust service
	@echo "Building Rust service..."
	@docker-compose build rust-core-engine

build-python: ## Build only Python service
	@echo "Building Python service..."
	@docker-compose build python-ai-service

build-frontend: ## Build only Frontend service
	@echo "Building Frontend service..."
	@docker-compose build nextjs-ui-dashboard

# Service Management
start: ## Start all services
	@echo "Starting all services..."
	@docker-compose up -d

start-memory: ## Start all services with memory optimization
	@echo "Starting all services with memory optimization..."
	@docker-compose -f docker-compose.memory-optimized.yml up -d

start-core: ## Start core services only
	@echo "Starting core services..."
	@docker-compose up -d rust-core-engine python-ai-service nextjs-ui-dashboard

# PostgreSQL removed - MongoDB only

start-with-monitoring: ## Start services with monitoring
	@echo "Starting services with monitoring..."
	@docker-compose --profile monitoring up -d

start-all: ## Start all services including optional ones
	@echo "Starting all services..."
	@docker-compose --profile redis --profile monitoring up -d

stop: ## Stop all services
	@echo "Stopping all services..."
	@docker-compose down

restart: ## Restart all services
	@echo "Restarting all services..."
	@docker-compose restart

# Logs and Monitoring
logs: ## Show logs for all services
	@docker-compose logs -f

logs-rust: ## Show logs for Rust service
	@docker-compose logs -f rust-core-engine

logs-python: ## Show logs for Python service
	@docker-compose logs -f python-ai-service

logs-frontend: ## Show logs for Frontend service
	@docker-compose logs -f nextjs-ui-dashboard

# Development
dev: ## Start all services in development mode with hot reload
	@echo "Starting all services in development mode with hot reload..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml up --build

dev-detach: ## Start all services in development mode (detached)
	@echo "Starting all services in development mode (detached)..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml up -d --build

dev-rust: ## Start Rust service in development mode with hot reload
	@echo "Starting Rust service in development mode with hot reload..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml up rust-core-engine --build

dev-python: ## Start Python service in development mode with hot reload
	@echo "Starting Python service in development mode with hot reload..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml up python-ai-service --build

dev-frontend: ## Start Frontend service in development mode with hot reload
	@echo "Starting Frontend service in development mode with hot reload..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml up nextjs-ui-dashboard --build

dev-local-rust: ## Start Rust service locally (without Docker)
	@echo "Starting Rust service locally..."
	@cd rust-core-engine && cargo run -- --config config.toml

dev-local-python: ## Start Python service locally (without Docker)
	@echo "Starting Python service locally..."
	@cd python-ai-service && python main.py

dev-local-frontend: ## Start Frontend service locally (without Docker)
	@echo "Starting Frontend service locally..."
	@cd nextjs-ui-dashboard && npm run dev

dev-logs: ## Show development logs
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml logs -f

dev-stop: ## Stop development services
	@echo "Stopping development services..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml down

dev-rebuild: ## Rebuild and restart development services
	@echo "Rebuilding development services..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml down
	@docker-compose -f infrastructure/docker/docker-compose.yml -f docker-compose.dev.yml up --build -d

# Testing
test: ## Run all tests with coverage
	@echo "🧪 Running all tests with coverage..."
	@chmod +x run_all_tests.sh
	@./run_all_tests.sh

test-integration: ## Run integration tests for all services
	@echo "Running integration tests..."
	@$(MAKE) test-rust-python
	@$(MAKE) test-dashboard-rust
	@$(MAKE) test-dashboard-python
	@$(MAKE) test-websocket

test-rust-python: ## Test Rust → Python AI communication
	@echo "Testing Rust → Python AI integration..."
	@cd rust-core-engine && cargo test test_ai_service_integration -- --test-threads=1

test-dashboard-rust: ## Test Dashboard → Rust API communication
	@echo "Testing Dashboard → Rust API integration..."
	@cd nextjs-ui-dashboard && npm run test:integration:rust

test-dashboard-python: ## Test Dashboard → Python AI communication
	@echo "Testing Dashboard → Python AI integration..."
	@cd nextjs-ui-dashboard && npm run test:integration:python

test-websocket: ## Test WebSocket communication
	@echo "Testing WebSocket integration..."
	@cd nextjs-ui-dashboard && npm run test:websocket

test-rust: ## Run Rust tests with coverage
	@echo "🦀 Running Rust tests..."
	@cd rust-core-engine && chmod +x run_tests.sh && ./run_tests.sh

test-python: ## Run Python tests with coverage
	@echo "🐍 Running Python tests..."
	@cd python-ai-service && chmod +x run_tests.sh && ./run_tests.sh

test-frontend: ## Run Frontend tests with coverage
	@echo "⚛️  Running Next.js tests..."
	@cd nextjs-ui-dashboard && chmod +x run_tests.sh && ./run_tests.sh

# Linting and Code Quality
lint: ## Run linting for all services
	@echo "Running linting for all services..."
	@$(MAKE) lint-rust
	@$(MAKE) lint-python
	@$(MAKE) lint-frontend

lint-rust: ## Run Rust linting
	@echo "Running Rust linting..."
	@cd rust-core-engine && cargo clippy -- -D warnings

lint-python: ## Run Python linting
	@echo "Running Python linting..."
	@cd python-ai-service && python -m flake8 . --count --select=E9,F63,F7,F82 --show-source --statistics

lint-frontend: ## Run Frontend linting
	@echo "Running Frontend linting..."
	@cd nextjs-ui-dashboard && npm run lint

# Cleanup
clean: ## Clean up Docker resources
	@echo "Cleaning up Docker resources..."
	@docker-compose down -v --rmi all --remove-orphans

clean-data: ## Clean up data volumes
	@echo "Cleaning up data volumes..."
	@docker-compose down -v
	@docker volume prune -f

clean-images: ## Clean up Docker images
	@echo "Cleaning up Docker images..."
	@docker image prune -f

clean-all: ## Clean up everything
	@echo "Cleaning up everything..."
	@docker-compose down -v --rmi all --remove-orphans
	@docker system prune -f

# Health Checks
health: ## Check health of all services
	@echo "Checking service health..."
	@echo "Frontend: $$(curl -s -o /dev/null -w '%{http_code}' http://localhost:3000/health || echo 'DOWN')"
	@echo "Python AI: $$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8000/health || echo 'DOWN')"
	@echo "Rust Core: $$(curl -s -o /dev/null -w '%{http_code}' http://localhost:8080/health || echo 'DOWN')"

# MongoDB Operations
db-backup: ## Backup MongoDB
	@echo "Backing up MongoDB..."
	@docker exec mongodb-primary mongodump --uri="${DATABASE_URL}" --out=/backup/dump_$$(date +%Y%m%d_%H%M%S)

db-restore: ## Restore MongoDB from backup (specify BACKUP_DIR)
	@echo "Restoring MongoDB from $(BACKUP_DIR)..."
	@docker exec mongodb-primary mongorestore --uri="${DATABASE_URL}" --dir=$(BACKUP_DIR)

# Production Deployment
docker-build: ## Build Docker images for production
	@echo "Building production Docker images..."
	@docker build -t $(DOCKER_REGISTRY)/rust-core-engine:$(DOCKER_TAG) ./rust-core-engine
	@docker build -t $(DOCKER_REGISTRY)/python-ai-service:$(DOCKER_TAG) ./python-ai-service
	@docker build -t $(DOCKER_REGISTRY)/nextjs-ui-dashboard:$(DOCKER_TAG) ./nextjs-ui-dashboard

docker-push: ## Push Docker images to registry
	@echo "Pushing Docker images to registry..."
	@docker push $(DOCKER_REGISTRY)/rust-core-engine:$(DOCKER_TAG)
	@docker push $(DOCKER_REGISTRY)/python-ai-service:$(DOCKER_TAG)
	@docker push $(DOCKER_REGISTRY)/nextjs-ui-dashboard:$(DOCKER_TAG)

deploy: ## Deploy to production
	@echo "Deploying to production..."
	@docker-compose -f infrastructure/docker/docker-compose.yml -f infrastructure/docker/docker-compose.prod.yml up -d

# Quick commands
up: start ## Alias for start
down: stop ## Alias for stop
ps: ## Show running containers
	@docker-compose ps

exec-rust: ## Execute shell in Rust container
	@docker-compose exec rust-core-engine sh

exec-python: ## Execute shell in Python container
	@docker-compose exec python-ai-service bash

exec-frontend: ## Execute shell in Frontend container
	@docker-compose exec nextjs-ui-dashboard sh

# Installation checks
check-deps: ## Check if required dependencies are installed
	@echo "Checking dependencies..."
	@which docker > /dev/null || (echo "Docker not found. Please install Docker." && exit 1)
	@which docker-compose > /dev/null || (echo "Docker Compose not found. Please install Docker Compose." && exit 1)
	@echo "All dependencies are installed!"

# Security and Environment Validation
validate-env: ## Validate that all required environment variables are set
	@echo "Validating environment variables..."
	@chmod +x scripts/validate-env.sh
	@./scripts/validate-env.sh || (echo "Environment validation failed. Please check your .env file." && exit 1)

check-secrets: ## Check for weak or default secrets in environment files
	@echo "Checking for weak or default secrets..."
	@if [ -f .env ]; then \
		if grep -q "your_api_key_here\|your_secret_key_here\|changeme\|password123\|admin123" .env; then \
			echo "ERROR: Weak or default secrets found in .env file!"; \
			echo "Please replace all placeholder values with secure secrets."; \
			exit 1; \
		fi; \
		echo "No obvious weak secrets detected."; \
	else \
		echo "WARNING: .env file not found. Run 'make setup' first."; \
	fi

validate-secrets: ## Validate that secrets meet minimum security requirements
	@echo "Validating secret strength..."
	@if [ -f .env ]; then \
		source .env 2>/dev/null; \
		ERRORS=0; \
		if [ ! -z "$$BINANCE_API_KEY" ] && [ $${#BINANCE_API_KEY} -lt 32 ]; then \
			echo "ERROR: BINANCE_API_KEY is too short (min 32 chars)"; \
			ERRORS=$$((ERRORS + 1)); \
		fi; \
		if [ ! -z "$$INTER_SERVICE_TOKEN" ] && [ $${#INTER_SERVICE_TOKEN} -lt 32 ]; then \
			echo "ERROR: INTER_SERVICE_TOKEN is too short (min 32 chars)"; \
			ERRORS=$$((ERRORS + 1)); \
		fi; \
		if [ ! -z "$$RUST_API_KEY" ] && [ $${#RUST_API_KEY} -lt 32 ]; then \
			echo "ERROR: RUST_API_KEY is too short (min 32 chars)"; \
			ERRORS=$$((ERRORS + 1)); \
		fi; \
		if [ $$ERRORS -gt 0 ]; then \
			echo "$$ERRORS secret validation errors found."; \
			exit 1; \
		fi; \
		echo "All secrets meet minimum requirements."; \
	else \
		echo "ERROR: .env file not found."; \
		exit 1; \
	fi

generate-secrets: ## Generate secure random secrets for all required variables
	@echo "Generating secure secrets..."
	@echo "Copy these values to your .env file:"
	@echo ""
	@echo "# Inter-service Authentication"
	@echo "INTER_SERVICE_TOKEN=$$(openssl rand -hex 32)"
	@echo "RUST_API_KEY=$$(openssl rand -hex 32)"
	@echo "PYTHON_API_KEY=$$(openssl rand -hex 32)"
	@echo ""
	@echo "# Dashboard"
	@echo "DASHBOARD_SESSION_SECRET=$$(openssl rand -hex 32)"
	@echo ""
	@echo "# Infrastructure (if using optional services)"
	@echo "REDIS_PASSWORD=$$(openssl rand -hex 16)"
	@echo "RABBITMQ_PASSWORD=$$(openssl rand -hex 16)"
	@echo "KONG_DB_PASSWORD=$$(openssl rand -hex 16)"
	@echo ""
	@echo "Note: You still need to provide BINANCE_API_KEY, BINANCE_SECRET_KEY, and DATABASE_URL manually."

security-check: check-secrets validate-secrets ## Run all security checks on environment configuration
	@echo "All security checks passed!"

# Show service URLs
urls: ## Show service URLs
	@echo "Service URLs:"
	@echo "  Frontend Dashboard: http://localhost:3000"
	@echo "  Rust Trading API:   http://localhost:8080"
	@echo "  Python AI API:      http://localhost:8000"
	@echo "  Grafana (optional): http://localhost:3001"
	@echo "  Prometheus (opt.):  http://localhost:9090"

# Quality Metrics
quality-metrics: ## Run comprehensive quality metrics analysis
	@echo "Running quality metrics analysis..."
	@chmod +x scripts/quality-metrics.sh
	@./scripts/quality-metrics.sh

quality-report: ## Generate quality metrics report
	@echo "Generating quality metrics report..."
	@chmod +x scripts/quality-metrics.sh
	@./scripts/quality-metrics.sh
	@echo ""
	@echo "Report available at: metrics/quality-report-*.json"
	@echo "See docs/reports/QUALITY_METRICS_SUMMARY.md for detailed analysis"

# Development help
dev-help: ## Show development commands
	@echo "Development Commands:"
	@echo ""
	@echo "Setup:"
	@echo "  make setup-dev    - Setup development environment with hot reload"
	@echo ""
	@echo "Development:"
	@echo "  make dev          - Start all services with hot reload"
	@echo "  make dev-detach   - Start all services in background"
	@echo "  make dev-rust     - Start only Rust service with hot reload"
	@echo "  make dev-python   - Start only Python service with hot reload"
	@echo "  make dev-frontend - Start only Frontend service with hot reload"
	@echo ""
	@echo "Development Utilities:"
	@echo "  make dev-logs     - Show development logs"
	@echo "  make dev-stop     - Stop development services"
	@echo "  make dev-rebuild  - Rebuild and restart development services"
	@echo ""
	@echo "Local Development (without Docker):"
	@echo "  make dev-local-rust     - Start Rust locally"
	@echo "  make dev-local-python   - Start Python locally"
	@echo "  make dev-local-frontend - Start Frontend locally" 