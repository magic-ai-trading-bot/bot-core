// @spec:FR-MCP-001 - MCP Server Types
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-01-mcp-server-setup.md

import type { CallToolResult } from "@modelcontextprotocol/sdk/types.js";

// Security tiers for MCP tools
export enum SecurityTier {
  PUBLIC = 1,
  AUTHENTICATED = 2,
  SENSITIVE = 3,
  CRITICAL = 4,
}

// Tool configuration
export interface ToolConfig {
  name: string;
  tier: SecurityTier;
  requiresConfirmation: boolean;
  rateLimit: { max: number; windowMs: number };
  service: "rust" | "python";
}

// BotCore API response format
export interface BotCoreResponse<T = unknown> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp?: string;
}

// Standard MCP tool result builders
export function toolSuccess(data: unknown): CallToolResult {
  const text =
    typeof data === "string" ? data : JSON.stringify(data, null, 2);
  return { content: [{ type: "text", text }] };
}

export function toolError(message: string): CallToolResult {
  return { content: [{ type: "text", text: message }], isError: true };
}

export function toolConfirm(
  action: string,
  details: string,
  token: string
): CallToolResult {
  return {
    content: [
      {
        type: "text",
        text: `CONFIRM REQUIRED: ${action}\n${details}\nReply with confirm_token: ${token}`,
      },
    ],
  };
}

// Logger utility
export function log(
  level: "info" | "warn" | "error" | "debug",
  message: string,
  data?: Record<string, unknown>
): void {
  const entry = {
    timestamp: new Date().toISOString(),
    level,
    message,
    ...data,
  };
  if (level === "error") {
    console.error(JSON.stringify(entry));
  } else {
    console.log(JSON.stringify(entry));
  }
}
