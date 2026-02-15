// @spec:FR-MCP-005 - AI Task Management Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-tool-implementation.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerTaskTools(server: McpServer): void {
  // All tools target Python service (port 8000)

  server.registerTool(
    "trigger_config_analysis",
    {
      title: "Trigger Configuration Analysis",
      description: "Trigger AI-powered configuration analysis to optimize trading parameters. This is a long-running task (up to 2 minutes).",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/config-analysis/trigger", {
        method: "POST",
        body: {},
        timeoutMs: 120_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to trigger config analysis");
    }
  );

  server.registerTool(
    "predict_trend",
    {
      title: "Predict Price Trend",
      description: "Predict future price trend for a symbol using ML models. Returns predicted direction and confidence score.",
      inputSchema: {
        symbol: z.string().describe("Trading pair symbol (e.g., BTCUSDT)"),
        timeframe: z.string().optional().describe("Timeframe for prediction (e.g., 1h, 4h, 1d)"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ symbol, timeframe }: { symbol: string; timeframe?: string }) => {
      const res = await apiRequest("python", "/predict-trend", {
        method: "POST",
        body: { symbol, timeframe },
        timeoutMs: 60_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Trend prediction failed");
    }
  );

  server.registerTool(
    "get_ai_config_suggestions_python",
    {
      title: "Get AI Config Suggestions (Python)",
      description: "Get AI-generated configuration suggestions directly from Python service for optimal trading parameters.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/ai/config-suggestions");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch config suggestions");
    }
  );

  server.registerTool(
    "chat_with_project",
    {
      title: "Chat with Project Assistant",
      description: "Ask questions about the BotCore project, get help with configuration, or request explanations.",
      inputSchema: {
        message: z.string().describe("Your question or message to the project assistant"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ message }: { message: string }) => {
      const res = await apiRequest("python", "/api/chat/project", {
        method: "POST",
        body: { message },
        timeoutMs: 60_000,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Chat request failed");
    }
  );

  server.registerTool(
    "get_chat_suggestions",
    {
      title: "Get Chat Suggestions",
      description: "Get suggested questions or topics you can ask the project assistant.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/api/chat/project/suggestions");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch chat suggestions");
    }
  );

  server.registerTool(
    "clear_chat_history",
    {
      title: "Clear Chat History",
      description: "Clear the conversation history with the project assistant.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/api/chat/project/clear", {
        method: "POST",
        body: {},
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to clear chat history");
    }
  );

  server.registerTool(
    "get_ai_debug_info",
    {
      title: "Get AI Debug Information",
      description: "Get detailed debug information about GPT-4 service status, API connectivity, and recent errors.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("python", "/debug/gpt4");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch AI debug info");
    }
  );

  log("info", "Registered 7 AI task management tools (all Python)");
}
