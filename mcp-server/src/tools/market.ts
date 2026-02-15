// @spec:FR-MCP-004 - Market Data Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-market-tools.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

const timeframeSchema = z.enum(["1m", "5m", "15m", "30m", "1h", "4h", "1d"]);

export function registerMarketTools(server: McpServer): void {
  // GET /api/market/prices
  server.registerTool(
    "get_market_prices",
    {
      title: "Get Market Prices",
      description: "Retrieves current market prices for all tracked symbols",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/market/prices", { timeoutMs: 10_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch market prices");
    }
  );

  // GET /api/market/overview
  server.registerTool(
    "get_market_overview",
    {
      title: "Get Market Overview",
      description: "Retrieves market overview with statistics and trends",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/market/overview", { timeoutMs: 10_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch market overview");
    }
  );

  // GET /api/market/candles/:symbol/:timeframe
  server.registerTool(
    "get_candles",
    {
      title: "Get Candles",
      description: "Retrieves candlestick data for a specific symbol and timeframe",
      inputSchema: {
        symbol: z.string().describe("Trading symbol (e.g., BTCUSDT)"),
        timeframe: timeframeSchema.describe("Timeframe for candles"),
        limit: z.number().optional().describe("Number of candles to retrieve (default: 100)"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ symbol, timeframe, limit }: { symbol: string; timeframe: string; limit?: number }) => {
      const query = limit ? `?limit=${limit}` : "";
      const res = await apiRequest("rust", `/api/market/candles/${symbol}/${timeframe}${query}`, {
        timeoutMs: 15_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch candles");
    }
  );

  // GET /api/market/chart/:symbol/:timeframe
  server.registerTool(
    "get_chart",
    {
      title: "Get Chart Data",
      description: "Retrieves chart data with indicators for a specific symbol and timeframe",
      inputSchema: {
        symbol: z.string().describe("Trading symbol (e.g., BTCUSDT)"),
        timeframe: timeframeSchema.describe("Timeframe for chart"),
        limit: z.number().optional().describe("Number of data points (default: 100)"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ symbol, timeframe, limit }: { symbol: string; timeframe: string; limit?: number }) => {
      const query = limit ? `?limit=${limit}` : "";
      const res = await apiRequest("rust", `/api/market/chart/${symbol}/${timeframe}${query}`, {
        timeoutMs: 15_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch chart data");
    }
  );

  // GET /api/market/charts
  server.registerTool(
    "get_multi_charts",
    {
      title: "Get Multiple Charts",
      description: "Retrieves chart data for multiple symbols simultaneously",
      inputSchema: {
        symbols: z.array(z.string()).optional().describe("Array of trading symbols (optional)"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ symbols }: { symbols?: string[] }) => {
      const query = symbols?.length ? `?symbols=${symbols.join(",")}` : "";
      const res = await apiRequest("rust", `/api/market/charts${query}`, { timeoutMs: 20_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch charts");
    }
  );

  // GET /api/market/symbols
  server.registerTool(
    "get_symbols",
    {
      title: "Get Tracked Symbols",
      description: "Retrieves list of all tracked trading symbols",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/market/symbols", { timeoutMs: 5_000 });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch symbols");
    }
  );

  // POST /api/market/symbols
  server.registerTool(
    "add_symbol",
    {
      title: "Add Symbol",
      description: "Adds a new trading symbol to track",
      inputSchema: {
        symbol: z.string().describe("Trading symbol to add (e.g., ETHUSDT)"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ symbol }: { symbol: string }) => {
      const res = await apiRequest("rust", "/api/market/symbols", {
        method: "POST",
        body: { symbol },
        timeoutMs: 10_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to add symbol");
    }
  );

  // DELETE /api/market/symbols/:symbol
  server.registerTool(
    "remove_symbol",
    {
      title: "Remove Symbol",
      description: "Removes a trading symbol from tracking",
      inputSchema: {
        symbol: z.string().describe("Trading symbol to remove"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ symbol }: { symbol: string }) => {
      const res = await apiRequest("rust", `/api/market/symbols/${symbol}`, {
        method: "DELETE",
        timeoutMs: 10_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to remove symbol");
    }
  );

  log("info", "Market tools registered (8 tools)");
}
