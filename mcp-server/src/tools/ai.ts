// @spec:FR-MCP-004 - AI Analysis Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-tool-implementation.md
// All tools proxy exclusively to Rust API.

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerAiTools(server: McpServer): void {
  // ====== Rust API Tools (port 8080) ======

  server.registerTool(
    "analyze_market",
    {
      title: "Analyze Market",
      description: "Analyze market conditions for a symbol using the Rust strategy engine. Returns market analysis including trend, indicators, and strategy-based recommendations.",
      inputSchema: {
        symbol: z.string().describe("Trading pair symbol (e.g., BTCUSDT)"),
        timeframe: z.string().optional().describe("Timeframe for analysis (e.g., 1h, 4h, 1d)"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ symbol, timeframe }: { symbol: string; timeframe?: string }) => {
      const res = await apiRequest("rust", "/api/ai/analyze", {
        method: "POST",
        body: { symbol, timeframe },
        timeoutMs: 120_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Market analysis failed");
    }
  );

  server.registerTool(
    "get_strategy_recommendations",
    {
      title: "Get Strategy Recommendations",
      description: "Get trading strategy recommendations for a specific symbol based on current market conditions from the Rust strategy engine.",
      inputSchema: {
        symbol: z.string().describe("Trading pair symbol (e.g., BTCUSDT)"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ symbol }: { symbol: string }) => {
      const res = await apiRequest("rust", "/api/ai/strategy-recommendations", {
        method: "POST",
        body: { symbol },
        timeoutMs: 60_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Strategy recommendations failed");
    }
  );

  server.registerTool(
    "get_market_condition",
    {
      title: "Get Market Condition Analysis",
      description: "Get strategy engine assessment of current market condition (bullish/bearish/neutral) with confidence score.",
      inputSchema: {
        symbol: z.string().describe("Trading pair symbol (e.g., BTCUSDT)"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ symbol }: { symbol: string }) => {
      const res = await apiRequest("rust", "/api/ai/market-condition", {
        method: "POST",
        body: { symbol },
        timeoutMs: 60_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Market condition analysis failed");
    }
  );

  server.registerTool(
    "send_ai_feedback",
    {
      title: "Send Signal Feedback",
      description: "Provide feedback on strategy-generated signals to improve configuration performance.",
      inputSchema: {
        signal_id: z.string().describe("Signal ID to provide feedback for"),
        feedback: z.enum(["positive", "negative"]).describe("Feedback type"),
        comment: z.string().optional().describe("Optional comment explaining the feedback"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ signal_id, feedback, comment }: { signal_id: string; feedback: "positive" | "negative"; comment?: string }) => {
      const res = await apiRequest("rust", "/api/ai/feedback", {
        method: "POST",
        body: { signal_id, feedback, comment },
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Feedback submission failed");
    }
  );

  server.registerTool(
    "get_ai_info",
    {
      title: "Get Strategy Service Information",
      description: "Get information about available trading strategies and capabilities. No authentication required.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/ai/info", { skipAuth: true });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch AI info");
    }
  );

  server.registerTool(
    "get_ai_strategies",
    {
      title: "Get Available Strategies",
      description: "List all available trading strategies and their configurations (RSI, MACD, Bollinger, Volume, etc.).",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/ai/strategies");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch AI strategies");
    }
  );

  log("info", "Registered 6 strategy analysis tools (Rust API)");
}
