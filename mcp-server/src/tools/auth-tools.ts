// @spec:FR-MCP-006 - Authentication Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-auth-tools.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerAuthTools(server: McpServer): void {
  // POST /api/auth/login
  server.registerTool(
    "login",
    {
      title: "Login",
      description: "Authenticates user with email and password, returns JWT token",
      inputSchema: {
        email: z.string().email().describe("User email address"),
        password: z.string().describe("User password"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ email, password }: { email: string; password: string }) => {
      const res = await apiRequest("rust", "/api/auth/login", {
        method: "POST",
        body: { email, password },
        skipAuth: true,
        timeoutMs: 10_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Login failed");
    }
  );

  // POST /api/auth/register
  server.registerTool(
    "register_user",
    {
      title: "Register User",
      description: "Creates a new user account with email, password, and name",
      inputSchema: {
        email: z.string().email().describe("User email address"),
        password: z.string().min(8).describe("User password (min 8 characters)"),
        name: z.string().describe("User full name"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ email, password, name }: { email: string; password: string; name: string }) => {
      const res = await apiRequest("rust", "/api/auth/register", {
        method: "POST",
        body: { email, password, name },
        skipAuth: true,
        timeoutMs: 10_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Registration failed");
    }
  );

  // GET /api/auth/me
  server.registerTool(
    "get_profile",
    {
      title: "Get Profile",
      description: "Retrieves current authenticated user profile information",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/auth/me", { timeoutMs: 5_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch profile");
    }
  );

  // POST /api/auth/refresh
  server.registerTool(
    "refresh_token",
    {
      title: "Refresh Token",
      description: "Refreshes the current JWT authentication token",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/auth/refresh", {
        method: "POST",
        timeoutMs: 5_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Token refresh failed");
    }
  );

  log("info", "Auth tools registered (4 tools)");
}
