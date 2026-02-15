// @spec:FR-MCP-009 - Trading Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-mcp-tool-implementation.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

/**
 * Register live/spot trading tools (read positions, account, performance).
 */
export function registerTradingTools(server: McpServer): void {
  server.registerTool(
    "get_trading_positions",
    {
      title: "Get Trading Positions",
      description:
        "Get all current open trading positions with entry price, current P&L, and position size.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/trading/positions");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to get positions");
    }
  );

  server.registerTool(
    "get_trading_account",
    {
      title: "Get Trading Account",
      description:
        "Get trading account balance, equity, margin usage, and available funds.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/trading/account");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to get account");
    }
  );

  server.registerTool(
    "close_trading_position",
    {
      title: "Close Trading Position",
      description:
        "Close an open trading position by symbol. This is a SENSITIVE operation that executes a market sell/buy to close the position.",
      inputSchema: {
        symbol: z.string().describe("Trading pair symbol (e.g. BTCUSDT)"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ symbol }: { symbol: string }) => {
      const res = await apiRequest("rust", `/api/trading/positions/${symbol}/close`, {
        method: "POST",
        body: {},
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to close position");
    }
  );

  server.registerTool(
    "get_trading_performance",
    {
      title: "Get Trading Performance",
      description:
        "Get trading performance metrics: win rate, total P&L, Sharpe ratio, max drawdown, and trade history statistics.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/trading/performance");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to get performance");
    }
  );

  log("info", "Trading tools registered (4 tools)");
}
