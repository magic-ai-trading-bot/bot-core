# Scout Report: Telegram Notification Tool Pattern Analysis

**Date**: 2026-02-17  
**Task**: Find MCP server tool registration patterns and setup for adding `send_telegram_notification` tool  
**Status**: COMPLETE  
**Token Usage**: Optimized parallel search + minimal reads  

---

## FINDINGS SUMMARY

Found all key files needed to implement `send_telegram_notification` tool. Pattern is consistent and well-established across 8 existing tool files.

---

## 1. MCP SERVER ARCHITECTURE

### Core Files Location
```
/Users/dungngo97/Documents/bot-core/mcp-server/src/
├── index.ts              # Server entrypoint, port 8090
├── server.ts             # Tool registration orchestrator
├── client.ts             # API request wrapper
├── types.ts              # Response types, helpers
├── auth.ts               # JWT auth
└── tools/                # Tool implementations (11 files)
    ├── monitoring.ts     # 4 tools (example for pattern)
    ├── health.ts         # 4 tools
    ├── market.ts         # 8 tools
    ├── paper-trading.ts  # 28 tools
    ├── settings.ts       # 10 tools (has notification prefs)
    ├── ai.ts             # 12 tools
    ├── auth-tools.ts     # 4 tools
    ├── trading.ts        # 4 tools
    ├── real-trading.ts   # 14 tools
    ├── tasks.ts          # 7 tools
    └── tuning.ts         # 8 tools
```

---

## 2. TOOL REGISTRATION PATTERN

### Standard Pattern (All Tools Follow This)

**File**: `/Users/dungngo97/Documents/bot-core/mcp-server/src/tools/monitoring.ts` (Example)

```typescript
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";  // For input schema validation
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerMonitoringTools(server: McpServer): void {
  server.registerTool(
    "tool_name_slug",  // kebab-case, unique identifier
    {
      title: "Human Readable Title",
      description: "Description of what the tool does",
      inputSchema: {
        // Zod schema for inputs (empty {} if no inputs)
        param1: z.string().describe("Parameter description"),
        param2: z.optional(z.number()),
      },
      annotations: {
        readOnlyHint: true,  // or false for write operations
        openWorldHint: false, // Usually false
      },
    },
    async ({ param1, param2 } = {}) => {
      // Handler: async function that processes inputs
      const res = await apiRequest("rust", "/api/path", {
        method: "POST",
        body: { param1, param2 },
        timeoutMs: 10_000,
      });
      return res.success 
        ? toolSuccess(res.data) 
        : toolError(res.error || "Failed to do thing");
    }
  );

  log("info", "Tools registered (N tools)");
}
```

### Key Points
1. **One registration function per tool file** (`registerXXXTools`)
2. **Zod schema** for input validation (imported from `zod`)
3. **apiRequest()** for HTTP calls (handles auth, timeouts, retries)
4. **toolSuccess()/toolError()** for responses (MCP format)
5. **log()** utility for consistency
6. **@spec tags** at top of file

---

## 3. SERVER REGISTRATION ORCHESTRATOR

**File**: `/Users/dungngo97/Documents/bot-core/mcp-server/src/server.ts`

### Current Registration Pattern
```typescript
export function createMcpServer(): McpServer {
  const server = new McpServer(...);
  
  // Register all tool categories
  registerHealthTools(server);          // 4 tools
  registerMarketTools(server);          // 8 tools
  registerTradingTools(server);         // 4 tools
  registerPaperTradingTools(server);    // 28 tools
  registerRealTradingTools(server);     // 14 tools
  registerAiTools(server);              // 12 tools
  registerTaskTools(server);            // 7 tools
  registerMonitoringTools(server);      // 5 tools
  registerSettingsTools(server);        // 10 tools
  registerAuthTools(server);            // 4 tools
  registerTuningTools(server);          // 8 tools
  
  return server;
}
```

**Total Current Tools**: 104 tools across 11 categories

**To Add Telegram Notifications**:
- Option A: Add to `monitoring.ts` (most logical - monitoring + alerting)
- Option B: Create new `notifications.ts` file (if building complex notification system)

---

## 4. API REQUEST WRAPPER

**File**: `/Users/dungngo97/Documents/bot-core/mcp-server/src/client.ts`

### apiRequest() Signature
```typescript
export async function apiRequest<T = unknown>(
  service: ServiceTarget,  // "rust" | "python"
  path: string,
  options: RequestOptions = {}
): Promise<BotCoreResponse<T>>

interface RequestOptions {
  method?: "GET" | "POST" | "PUT" | "DELETE";  // Default: GET
  body?: unknown;
  timeoutMs?: number;                            // Default: 30_000
  skipAuth?: boolean;                            // Default: false
}
```

### Response Type
```typescript
interface BotCoreResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp?: string;
}
```

### Features
- ✅ Automatic JWT authentication (unless skipAuth=true)
- ✅ Configurable timeouts
- ✅ Automatic retry on 5xx errors (GET only)
- ✅ Response normalization
- ✅ Error logging

---

## 5. RESPONSE HELPERS

**File**: `/Users/dungngo97/Documents/bot-core/mcp-server/src/types.ts`

```typescript
// MCP success response (CallToolResult format)
export function toolSuccess(data: unknown): CallToolResult {
  const text = typeof data === "string" 
    ? data 
    : JSON.stringify(data, null, 2);
  return { content: [{ type: "text", text }] };
}

// MCP error response
export function toolError(message: string): CallToolResult {
  return { 
    content: [{ type: "text", text: message }], 
    isError: true 
  };
}

// MCP confirmation request (for sensitive operations)
export function toolConfirm(
  action: string,
  details: string,
  token: string
): CallToolResult {
  return {
    content: [{
      type: "text",
      text: `CONFIRM REQUIRED: ${action}\n${details}\nReply with confirm_token: ${token}`
    }],
  };
}

// Logging utility
export function log(
  level: "info" | "warn" | "error" | "debug",
  message: string,
  data?: Record<string, unknown>
): void {
  // Logs as JSON to stdout
}
```

---

## 6. MCP SERVER STARTUP

**File**: `/Users/dungngo97/Documents/bot-core/mcp-server/src/index.ts`

```typescript
const PORT = parseInt(process.env.MCP_PORT || "8090", 10);
const app = express();

app.use(express.json({ limit: "1mb" }));

app.get("/health", (_req, res) => {
  res.json({ 
    status: "ok", 
    service: "botcore-mcp-server", 
    timestamp: new Date().toISOString() 
  });
});

// Main MCP endpoint
app.all("/mcp", async (req, res) => {
  // 1. Auth validation (Bearer token)
  if (!validateBearerToken(req.headers.authorization)) {
    res.status(401).json({ error: "Unauthorized" });
    return;
  }
  
  // 2. Session management (new McpServer per session)
  const sessionId = req.headers["mcp-session-id"] as string | undefined;
  
  if (sessionId && transports.has(sessionId)) {
    // Existing session - reuse transport
    const transport = transports.get(sessionId)!;
    await transport.handleRequest(req, res, req.body);
    return;
  }
  
  // 3. Create new session (if not exists)
  const mcpServer = createMcpServer();
  const transport = new StreamableHTTPServerTransport({
    sessionIdGenerator: () => randomUUID(),
  });
  
  await mcpServer.connect(transport);
  await transport.handleRequest(req, res, req.body);
  
  // Store by session ID
  if (transport.sessionId) {
    transports.set(transport.sessionId, transport);
  }
});

app.listen(PORT, "0.0.0.0", () => {
  log("info", `BotCore MCP Server listening on port ${PORT}`);
});
```

**Key Details**:
- Port: **8090** (configured via MCP_PORT env var)
- Transport: **StreamableHTTPServerTransport** (HTTP + SSE)
- Auth: **Bearer token** (env var: MCP_AUTH_TOKEN)
- Endpoint: **/mcp** (POST for requests, GET for SSE, DELETE for cleanup)

---

## 7. ENVIRONMENT VARIABLES

**File**: `/Users/dungngo97/Documents/bot-core/docker-compose-vps.yml` (Lines 224-258)

### Current MCP Server Env Vars
```yaml
mcp-server:
  environment:
    - MCP_AUTH_TOKEN=${MCP_AUTH_TOKEN:-}
    - RUST_API_URL=http://rust-core-engine:8080
    - PYTHON_API_URL=http://python-ai-service:8000
    - BOTCORE_EMAIL=${BOTCORE_EMAIL:-}
    - BOTCORE_PASSWORD=${BOTCORE_PASSWORD:-}
    - MCP_PORT=8090
    - NODE_ENV=production
```

### Available for Telegram Notifications
```
TELEGRAM_BOT_TOKEN       # Already in docker-compose for OpenClaw
TELEGRAM_USER_ID         # Already in docker-compose for OpenClaw
```

**Both env vars are already in compose file** (OpenClaw service, lines 197-198). Can be reused by MCP server.

---

## 8. EXAMPLE: SETTINGS NOTIFICATION TOOLS

**File**: `/Users/dungngo97/Documents/bot-core/mcp-server/src/tools/settings.ts` (Lines 82-150)

Already has **notification preference management** tools:
- `get_notification_preferences` - Read preferences
- `update_notification_preferences` - Update enabled channels
- `subscribe_push_notifications` - Browser push subs
- `unsubscribe_push_notifications` - Remove push subs

These manage **preferences**, but don't **send notifications**. Perfect place to add `send_telegram_notification` tool.

---

## 9. RECOMMENDED IMPLEMENTATION APPROACH

### Step 1: Create/Update Tool File
**Location**: `/Users/dungngo97/Documents/bot-core/mcp-server/src/tools/monitoring.ts` (add to existing)

```typescript
server.registerTool(
  "send_telegram_notification",
  {
    title: "Send Telegram Notification",
    description: "Send a notification to configured Telegram channel or user",
    inputSchema: {
      message: z.string().describe("Notification message"),
      title: z.string().optional().describe("Message title/header"),
      severity: z.enum(["info", "warning", "error"]).optional().describe("Severity level"),
    },
    annotations: {
      readOnlyHint: false,  // Write operation
      openWorldHint: false,
    },
  },
  async ({ message, title, severity }) => {
    const res = await apiRequest("rust", "/api/notifications/telegram", {
      method: "POST",
      body: { message, title, severity },
      timeoutMs: 10_000,
    });
    return res.success
      ? toolSuccess({ success: true, message: "Telegram notification sent" })
      : toolError(res.error || "Failed to send Telegram notification");
  }
);
```

### Step 2: Update server.ts Registration
No changes needed if adding to monitoring.ts (already registered).

If creating new file:
```typescript
import { registerNotificationTools } from "./tools/notifications.js";

// In createMcpServer():
registerNotificationTools(server);  // Add this line
```

### Step 3: Backend API Requirement
Rust backend needs endpoint:
- `POST /api/notifications/telegram` - Send Telegram notification
  - Input: `{ message, title?, severity? }`
  - Output: `{ success: bool, message_id?: string }`

### Step 4: Environment Setup
Add to docker-compose-vps.yml mcp-server section:
```yaml
- TELEGRAM_BOT_TOKEN=${TELEGRAM_BOT_TOKEN:-}
- TELEGRAM_USER_ID=${TELEGRAM_USER_ID:-}
```

(Already exist in OpenClaw service - can reference or duplicate)

---

## 10. KEY INSIGHTS

| Aspect | Pattern | Details |
|--------|---------|---------|
| **Tool Count** | 104 current | 11 categories across 11 files |
| **File Pattern** | One function per file | `registerXXXTools(server: McpServer)` |
| **Schema Validation** | Zod types | `z.string()`, `z.enum()`, `z.optional()` |
| **HTTP Requests** | Via apiRequest() | Handles auth, timeouts, retries automatically |
| **Response Format** | MCP CallToolResult | Via `toolSuccess()`/`toolError()` helpers |
| **Server Port** | 8090 | HTTP + SSE StreamableHTTPServerTransport |
| **Auth Method** | Bearer token | From MCP_AUTH_TOKEN env var |
| **Session Model** | Per-connection | McpServer created per session |
| **Service Targets** | "rust" \| "python" | Two backend services |
| **Logging** | JSON stdout | Via `log(level, msg, data)` |
| **Timeout Default** | 30 seconds | Configurable per request |

---

## 11. FILE CHECKLIST FOR TELEGRAM NOTIFICATION TOOL

**Files to read/modify**:
- ✅ `/Users/dungngo97/Documents/bot-core/mcp-server/src/tools/monitoring.ts` - Add tool here
- ⚠️  `/Users/dungngo97/Documents/bot-core/mcp-server/src/server.ts` - No change (monitoring already registered)
- ⚠️  `/Users/dungngo97/Documents/bot-core/docker-compose-vps.yml` - Verify/add TELEGRAM env vars
- ❌ `/Users/dungngo97/Documents/bot-core/mcp-server/src/client.ts` - No change needed
- ❌ `/Users/dungngo97/Documents/bot-core/mcp-server/src/types.ts` - No change needed
- ❌ `/Users/dungngo97/Documents/bot-core/mcp-server/src/index.ts` - No change needed

**Backend requirement**:
- 🔧 Rust backend needs: `POST /api/notifications/telegram` endpoint

---

## REFERENCE CODE LOCATIONS

### Core Pattern Examples
| Feature | File | Lines |
|---------|------|-------|
| Tool registration | monitoring.ts | 8-70 |
| Zod input schema | market.ts | 9, 48-52 |
| apiRequest usage | monitoring.ts | 19 |
| Error handling | market.ts | 60 |
| Log statement | monitoring.ts | 69 |
| Response helpers | types.ts | 32-55 |
| Server setup | index.ts | 26-81 |
| Server creation | server.ts | 21-54 |

---

## UNRESOLVED QUESTIONS

1. **Telegram backend implementation**: Does Rust service already have `/api/notifications/telegram` endpoint? Or needs to be added?
2. **Message formatting**: Should Telegram messages support markdown/HTML formatting, or plain text only?
3. **Fallback handling**: What happens if Telegram token is not configured?
4. **Rate limiting**: Should there be rate limits on sending notifications?
5. **Notification logging**: Should sent notifications be persisted to MongoDB for history?

---

**Scout Report Complete** ✅
