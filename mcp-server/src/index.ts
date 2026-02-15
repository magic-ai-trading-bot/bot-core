// @spec:FR-MCP-006 - MCP Server Entrypoint
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-01-mcp-server-setup.md

import express from "express";
import { randomUUID } from "node:crypto";
import { StreamableHTTPServerTransport } from "@modelcontextprotocol/sdk/server/streamableHttp.js";
import { createMcpServer } from "./server.js";
import { validateBearerToken } from "./auth.js";
import { log } from "./types.js";

const PORT = parseInt(process.env.MCP_PORT || "8090", 10);
const app = express();

// Parse JSON bodies
app.use(express.json({ limit: "1mb" }));

// Health check endpoint (no auth)
app.get("/health", (_req, res) => {
  res.json({ status: "ok", service: "botcore-mcp-server", timestamp: new Date().toISOString() });
});

// Map of session ID to transport (McpServer is created per-session per SDK requirement)
const transports = new Map<string, StreamableHTTPServerTransport>();

// MCP endpoint - handles POST (requests), GET (SSE streams), DELETE (cleanup)
app.all("/mcp", async (req, res) => {
  // Auth check
  if (!validateBearerToken(req.headers.authorization)) {
    res.status(401).json({ error: "Unauthorized" });
    return;
  }

  // Check for existing session
  const sessionId = req.headers["mcp-session-id"] as string | undefined;

  if (sessionId && transports.has(sessionId)) {
    const transport = transports.get(sessionId)!;
    await transport.handleRequest(req, res, req.body);
    return;
  }

  // Handle DELETE for session cleanup
  if (req.method === "DELETE") {
    if (sessionId && transports.has(sessionId)) {
      const transport = transports.get(sessionId)!;
      await transport.close();
      transports.delete(sessionId);
      res.status(200).json({ message: "Session closed" });
    } else {
      res.status(404).json({ error: "Session not found" });
    }
    return;
  }

  // Create new McpServer + transport per session (SDK requires one transport per server)
  const mcpServer = createMcpServer();
  const transport = new StreamableHTTPServerTransport({
    sessionIdGenerator: () => randomUUID(),
  });

  await mcpServer.connect(transport);

  // Clean up on close
  transport.onclose = () => {
    const sid = transport.sessionId;
    if (sid) {
      transports.delete(sid);
      log("info", "MCP session closed", { sessionId: sid });
    }
  };

  // Handle the initial request (sessionId is set during this call)
  await transport.handleRequest(req, res, req.body);

  // Store transport by session ID AFTER handleRequest (sessionId is set during initialize)
  const newSessionId = transport.sessionId;
  if (newSessionId) {
    transports.set(newSessionId, transport);
    log("info", "New MCP session created", { sessionId: newSessionId });
  }
});

// Start server
app.listen(PORT, "0.0.0.0", () => {
  log("info", `BotCore MCP Server listening on port ${PORT}`, {
    transport: "Streamable HTTP",
    endpoint: `/mcp`,
    health: `/health`,
  });
});
