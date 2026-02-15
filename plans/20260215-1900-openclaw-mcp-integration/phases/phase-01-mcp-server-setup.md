# Phase 01: MCP TypeScript Server Setup

**Status**: Pending | **Est.**: 2 days | **Priority**: P0 (foundation)

## Context Links

- [MCP TypeScript SDK](https://github.com/modelcontextprotocol/typescript-sdk)
- [Research: MCP Server Patterns](../research/researcher-02-mcp-server-patterns.md)
- [Research: OpenClaw MCP](../research/researcher-01-openclaw-mcp.md)
- [BotCore API Spec (Rust)](../../../../specs/02-design/2.3-api/API-RUST-CORE.md)
- [BotCore API Spec (Python)](../../../../specs/02-design/2.3-api/API-PYTHON-AI.md)

## Overview

Bootstrap the MCP TypeScript server project that acts as a proxy between OpenClaw/Claude and BotCore REST APIs. This phase sets up project structure, SDK integration, transport layer, authentication, and a minimal set of tools to validate the architecture end-to-end.

## Key Insights

1. **Streamable HTTP transport** is the recommended approach for 2025+ MCP servers (stateless, proxy-friendly, multi-client)
2. MCP server lives on internal Docker network only -- never exposed to internet
3. Auth flow: OpenClaw sends bearer token to MCP server; MCP server holds JWT for BotCore APIs
4. Error strategy: return `isError: true` in tool results (not protocol errors) so Claude can handle gracefully
5. Zod is mandatory peer dependency for input schema validation

## Requirements

- Node.js 18+ TypeScript project with ESM modules
- `@modelcontextprotocol/sdk` for MCP protocol handling
- `zod` for schema validation
- HTTP client (native `fetch` or `undici`) for upstream API calls
- Streamable HTTP transport on port 8090
- Bearer token auth for incoming requests
- JWT token management for outgoing BotCore API calls
- Structured logging (JSON format for Docker)
- Health check endpoint at `/health`
- Docker container with `node:18-alpine` base

## Architecture

```
                    +-----------------------+
                    |   MCP Server (:8090)  |
                    |   TypeScript / Node   |
                    +-----------+-----------+
                                |
        +-----------------------+-----------------------+
        |                       |                       |
   Streamable HTTP        Auth Layer              API Client
   Transport (in)     (bearer token)          (JWT -> BotCore)
        |                       |                       |
   OpenClaw/Claude        Token Store            +------+------+
                        (env vars)               |             |
                                           Rust :8080    Python :8000
```

## Related Code Files

| File | Purpose |
|------|---------|
| `mcp-server/` (new) | Root directory for MCP server |
| `mcp-server/src/index.ts` | Server entrypoint, transport setup |
| `mcp-server/src/server.ts` | MCP server instance, tool/resource registration |
| `mcp-server/src/transport.ts` | Streamable HTTP transport config |
| `mcp-server/src/auth.ts` | Bearer token validation, JWT management |
| `mcp-server/src/client.ts` | HTTP client for BotCore APIs |
| `mcp-server/src/types.ts` | Shared TypeScript types |
| `mcp-server/src/tools/` | Tool definitions directory |
| `mcp-server/src/tools/health.ts` | Minimal health check tool (validation) |
| `mcp-server/Dockerfile` | Docker build |
| `mcp-server/tsconfig.json` | TypeScript config |
| `mcp-server/package.json` | Dependencies |

## Implementation Steps

### 1. Project Scaffolding (~2h)
```bash
mkdir -p mcp-server/src/tools
cd mcp-server
npm init -y
npm install @modelcontextprotocol/sdk zod undici
npm install -D typescript @types/node tsx
```

- Configure `tsconfig.json` with ESM, strict mode, Node18 target
- Configure `package.json` with `"type": "module"`, build/dev scripts
- Add `.dockerignore` (node_modules, dist, .env)

### 2. Core Server Setup (~3h)
- `src/index.ts`: Entrypoint that creates MCP server and starts Streamable HTTP transport on port 8090
- `src/server.ts`: MCP Server class wrapping `@modelcontextprotocol/sdk`
  - Register `ListToolsRequestSchema` handler
  - Register `CallToolRequestSchema` handler with tool dispatch
  - Register `ListResourcesRequestSchema` handler (for status resources)
- `src/transport.ts`: Streamable HTTP transport configuration
  - Bind to `0.0.0.0:8090`
  - Add CORS headers for internal network
  - Add request logging middleware

### 3. Authentication Layer (~2h)
- `src/auth.ts`:
  - **Incoming**: Validate bearer token from OpenClaw (env: `MCP_AUTH_TOKEN`)
  - **Outgoing**: Manage JWT for BotCore API calls
    - Login via `POST /api/auth/login` on startup
    - Cache JWT token, track expiry
    - Auto-refresh before expiry (7-day tokens, refresh at 6 days)
  - Token store: in-memory (single instance, no persistence needed)

### 4. BotCore API Client (~3h)
- `src/client.ts`:
  - `BotCoreClient` class with methods for both Rust (8080) and Python (8000) services
  - Base URL from env: `RUST_API_URL`, `PYTHON_API_URL`
  - Auto-attach JWT to all requests
  - Timeout: 30s default, 120s for AI analysis
  - Error handling: map HTTP errors to MCP `isError: true` responses
  - Retry logic: 1 retry on 5xx with 2s backoff
  - Response normalization (Rust returns `{success, data, error}`, Python returns `{detail}`)

### 5. Validation Tool (~1h)
- `src/tools/health.ts`: Minimal tool to validate end-to-end flow
  - Tool: `check_system_health`
  - Calls `GET /api/health` (Rust) and `GET /health` (Python)
  - Returns combined health status
  - Validates: MCP protocol, auth, API client, response formatting

### 6. Docker Setup (~1h)
- `Dockerfile`:
  ```dockerfile
  FROM node:18-alpine AS builder
  WORKDIR /app
  COPY package*.json ./
  RUN npm ci --production=false
  COPY . .
  RUN npm run build

  FROM node:18-alpine
  WORKDIR /app
  COPY --from=builder /app/dist ./dist
  COPY --from=builder /app/node_modules ./node_modules
  COPY --from=builder /app/package.json ./
  ENV NODE_OPTIONS="--max-old-space-size=384"
  EXPOSE 8090
  HEALTHCHECK CMD wget -q --spider http://localhost:8090/health || exit 1
  CMD ["node", "dist/index.js"]
  ```

### 7. Docker Compose Integration (~1h)
- Add `mcp-server` service to `docker-compose-vps.yml`:
  ```yaml
  mcp-server:
    build:
      context: ./mcp-server
      dockerfile: Dockerfile
    container_name: mcp-server
    restart: unless-stopped
    environment:
      - MCP_AUTH_TOKEN=${MCP_AUTH_TOKEN}
      - RUST_API_URL=http://rust-core-engine:8080
      - PYTHON_API_URL=http://python-ai-service:8000
      - BOTCORE_EMAIL=${BOTCORE_EMAIL}
      - BOTCORE_PASSWORD=${BOTCORE_PASSWORD}
      - NODE_ENV=production
    networks:
      - bot-network
    depends_on:
      rust-core-engine:
        condition: service_healthy
      python-ai-service:
        condition: service_healthy
    deploy:
      resources:
        limits:
          memory: 512M
          cpus: "0.5"
  ```
  - Note: No `ports` exposed -- internal network only. OpenClaw accesses via `http://mcp-server:8090`.

## Todo List

- [ ] Create `mcp-server/` directory and scaffold project
- [ ] Install dependencies: `@modelcontextprotocol/sdk`, `zod`, `undici`
- [ ] Configure TypeScript (ESM, strict, Node18 target)
- [ ] Implement Streamable HTTP transport (`src/transport.ts`)
- [ ] Implement MCP server with tool dispatch (`src/server.ts`)
- [ ] Implement auth layer -- bearer validation + JWT management (`src/auth.ts`)
- [ ] Implement BotCore API client with error normalization (`src/client.ts`)
- [ ] Implement `check_system_health` validation tool (`src/tools/health.ts`)
- [ ] Write Dockerfile (multi-stage, alpine, 384MB heap)
- [ ] Add mcp-server to `docker-compose-vps.yml` (internal network only)
- [ ] Test: MCP server starts and responds to health check
- [ ] Test: `check_system_health` tool returns combined status from both APIs
- [ ] Test: Bearer token rejection for unauthorized requests

## Success Criteria

1. MCP server starts on port 8090 inside Docker network
2. `check_system_health` tool callable via MCP protocol and returns correct status
3. Unauthorized requests (wrong/missing bearer token) are rejected with 401
4. BotCore API client successfully authenticates via JWT and makes API calls
5. Container memory stays under 512MB during normal operation
6. Health check endpoint at `/health` returns 200

## Risk Assessment

| Risk | Probability | Impact | Mitigation |
|------|-------------|--------|------------|
| SDK version incompatibility | Low | High | Pin exact SDK version, test locally first |
| JWT token refresh race condition | Low | Medium | Single-instance server, mutex on refresh |
| Memory pressure on VPS | Medium | High | 384MB heap limit, monitor via `docker stats` |

## Security Considerations

- MCP server is NEVER exposed outside Docker network (no port mapping in compose)
- Bearer token for OpenClaw->MCP stored as env var, rotatable
- BotCore credentials (email/password) stored as env vars, used only for initial JWT login
- All internal communication over Docker bridge network (unencrypted but isolated)
- No sensitive data in logs (redact JWT tokens, passwords)

## Next Steps

After this phase: proceed to Phase 02 to implement the full set of 80 MCP tools mapped to BotCore endpoints.
