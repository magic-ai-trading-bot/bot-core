// @spec:FR-MCP-004 - AI Analysis Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-tool-implementation.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerAiTools(server: McpServer): void {
  // ====== Rust API Tools (port 8080) ======

  server.registerTool(
    "analyze_market",
    {
      title: "Analyze Market with GPT-4",
      description: "Use GPT-4 to analyze market conditions for a symbol. Returns comprehensive AI-powered market analysis including sentiment, trend, and recommendations.",
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
      title: "Get AI Strategy Recommendations",
      description: "Get AI-powered trading strategy recommendations for a specific symbol based on current market conditions.",
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
      description: "Get AI assessment of current market condition (bullish/bearish/neutral) with confidence score.",
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
      title: "Send AI Signal Feedback",
      description: "Provide feedback on AI-generated signals to improve model performance.",
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
      title: "Get AI Service Information",
      description: "Get information about available AI models and capabilities. No authentication required.",
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
      title: "Get Available AI Strategies",
      description: "List all available AI-powered trading strategies and their configurations.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/ai/strategies");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch AI strategies");
    }
  );

  // ====== Python API Tools (port 8000) ======

  server.registerTool(
    "get_ai_performance",
    {
      title: "Get AI Model Performance Metrics",
      description: "Get performance metrics for all AI/ML models including accuracy, precision, recall, and recent predictions.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/performance");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch AI performance");
    }
  );

  server.registerTool(
    "get_ai_storage_stats",
    {
      title: "Get AI Storage Statistics",
      description: "Get statistics about AI model storage usage, cache size, and stored predictions.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/storage/stats");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch AI storage stats");
    }
  );

  server.registerTool(
    "clear_ai_storage",
    {
      title: "Clear AI Storage Cache",
      description: "Clear AI model cache and stored predictions. Use with caution as this may impact performance temporarily.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/storage/clear", {
        method: "POST",
        body: {},
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to clear AI storage");
    }
  );

  server.registerTool(
    "get_ai_cost_statistics",
    {
      title: "Get AI API Cost Statistics",
      description: "Get cost statistics for AI API usage (OpenAI, other providers) including total costs and breakdown by model.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/cost/statistics");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch AI cost statistics");
    }
  );

  server.registerTool(
    "get_ai_config_suggestions",
    {
      title: "Get AI Configuration Suggestions",
      description: "Get AI-powered suggestions for optimal trading configuration based on historical performance.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/config-suggestions");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch config suggestions");
    }
  );

  server.registerTool(
    "get_ai_analysis_history",
    {
      title: "Get GPT-4 Analysis History",
      description: "Get history of all GPT-4 market analyses with timestamps and recommendations.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/gpt4-analysis-history");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch analysis history");
    }
  );

  log("info", "Registered 12 AI tools (6 Rust + 6 Python)");
}
