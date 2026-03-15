# Phase 2: Docker & Infrastructure Cleanup

**Priority**: P0 | **Effort**: 1h | **Status**: Pending

## Files to Modify

| File | Action |
|------|--------|
| `docker-compose-vps.yml` | Remove python-ai-service definition + depends_on + env vars |
| `infrastructure/docker/docker-compose.yml` | Remove python-ai-service + python-ai-service-dev definitions |
| `scripts/bot.sh` | Remove Python memory optimization + service references |
| `Makefile` | Remove build-python, dev-python, logs-python targets |
| `.env.example` | Remove XAI_API_KEY, PYTHON_AI_SERVICE_URL |
| `rust-core-engine/fly.toml` | Remove PYTHON_AI_SERVICE_URL |

## Implementation Steps

### 2a. docker-compose-vps.yml
- Delete entire `python-ai-service:` service block (lines 38-77)
- Remove `PYTHON_AI_SERVICE_URL` from rust-core-engine environment
- Remove `depends_on: python-ai-service` from rust-core-engine
- Remove `VITE_PYTHON_AI_URL` from nextjs-ui-dashboard
- Remove `PYTHON_API_URL` from mcp-server environment
- Remove `depends_on: python-ai-service` from mcp-server

### 2b. infrastructure/docker/docker-compose.yml
- Delete both python-ai-service blocks (prod + dev)
- Remove all depends_on and env var references

### 2c. Scripts & Makefile
- Remove Python-related make targets
- Remove Python memory optimization in bot.sh

## Todo

- [x] Remove Python from docker-compose-vps.yml
- [x] Remove Python from infrastructure docker-compose.yml
- [x] Clean Makefile
- [x] Clean bot.sh
- [x] Clean .env.example
- [x] Clean fly.toml
