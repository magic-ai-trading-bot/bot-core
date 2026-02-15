// @spec:FR-MCP-002 - MCP Authentication Layer
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-01-mcp-server-setup.md

import { log } from "./types.js";

// Bearer token for incoming requests from OpenClaw
const MCP_AUTH_TOKEN = process.env.MCP_AUTH_TOKEN || "";

// JWT management for outgoing BotCore API calls
let jwtToken: string | null = null;
let jwtExpiry = 0;

const BOTCORE_EMAIL = process.env.BOTCORE_EMAIL || "";
const BOTCORE_PASSWORD = process.env.BOTCORE_PASSWORD || "";
const RUST_API_URL = process.env.RUST_API_URL || "http://localhost:8080";

/**
 * Validate incoming bearer token from OpenClaw/Claude.
 */
export function validateBearerToken(authHeader: string | undefined): boolean {
  if (!MCP_AUTH_TOKEN) {
    log("warn", "MCP_AUTH_TOKEN not set, allowing all requests");
    return true;
  }
  if (!authHeader) return false;
  const token = authHeader.startsWith("Bearer ")
    ? authHeader.slice(7)
    : authHeader;
  return token === MCP_AUTH_TOKEN;
}

/**
 * Get a valid JWT token for BotCore API calls.
 * Logs in via /api/auth/login if no token or expired.
 */
export async function getJwtToken(): Promise<string> {
  const now = Date.now();
  // Refresh if token expires within 1 hour
  if (jwtToken && jwtExpiry > now + 3600_000) {
    return jwtToken;
  }

  if (!BOTCORE_EMAIL || !BOTCORE_PASSWORD) {
    log("warn", "BOTCORE credentials not set, API calls may fail");
    return "";
  }

  try {
    log("info", "Authenticating with BotCore API");
    const res = await fetch(`${RUST_API_URL}/api/auth/login`, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        email: BOTCORE_EMAIL,
        password: BOTCORE_PASSWORD,
      }),
    });

    if (!res.ok) {
      const text = await res.text();
      log("error", "BotCore login failed", {
        status: res.status,
        body: text,
      });
      throw new Error(`Login failed: ${res.status}`);
    }

    const data = (await res.json()) as {
      success: boolean;
      data?: { token: string };
    };
    if (data.success && data.data?.token) {
      jwtToken = data.data.token;
      // JWT expires in 7 days, refresh at 6 days
      jwtExpiry = now + 6 * 24 * 3600_000;
      log("info", "BotCore JWT obtained successfully");
      return jwtToken;
    }

    throw new Error("Unexpected login response format");
  } catch (err) {
    log("error", "JWT authentication error", {
      error: err instanceof Error ? err.message : String(err),
    });
    return "";
  }
}
