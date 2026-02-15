// @spec:FR-MCP-005 - Monitoring Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-monitoring-tools.md

import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerMonitoringTools(server: McpServer): void {
  // GET /api/monitoring/system
  server.registerTool(
    "get_system_monitoring",
    {
      title: "Get System Monitoring",
      description: "Retrieves system-level monitoring data (CPU, memory, disk, network)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/monitoring/system", { timeoutMs: 10_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch system monitoring");
    }
  );

  // GET /api/monitoring/trading
  server.registerTool(
    "get_trading_metrics",
    {
      title: "Get Trading Metrics",
      description: "Retrieves trading-specific metrics (win rate, PnL, positions, signals)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/monitoring/trading", { timeoutMs: 10_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch trading metrics");
    }
  );

  // GET /api/monitoring/connection
  server.registerTool(
    "get_connection_status",
    {
      title: "Get Connection Status",
      description: "Retrieves connection status for all external services (Binance, MongoDB, WebSocket)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/monitoring/connection", { timeoutMs: 5_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch connection status");
    }
  );

  // GET /health (Python AI service)
  server.registerTool(
    "get_python_health",
    {
      title: "Get Python AI Health",
      description: "Retrieves health status of the Python AI service",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/health", { skipAuth: true, timeoutMs: 5_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch Python health");
    }
  );

  log("info", "Monitoring tools registered (4 tools)");
}
