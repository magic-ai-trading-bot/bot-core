// @spec:FR-MCP-005 - AI Task Management Tools

import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { log } from "../types.js";

export function registerTaskTools(_server: McpServer): void {
  log("info", "Task tools registered (0 tools)");
}
