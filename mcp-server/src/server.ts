// @spec:FR-MCP-004 - MCP Server Core
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-01-mcp-server-setup.md

import { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { registerHealthTools } from "./tools/health.js";
import { registerMarketTools } from "./tools/market.js";
import { registerTradingTools } from "./tools/trading.js";
import { registerPaperTradingTools } from "./tools/paper-trading.js";
import { registerRealTradingTools } from "./tools/real-trading.js";
import { registerAiTools } from "./tools/ai.js";
import { registerTaskTools } from "./tools/tasks.js";
import { registerMonitoringTools } from "./tools/monitoring.js";
import { registerSettingsTools } from "./tools/settings.js";
import { registerAuthTools } from "./tools/auth-tools.js";
import { registerTuningTools } from "./tools/tuning.js";
import { registerNotificationTools } from "./tools/notification.js";
import { log } from "./types.js";

/**
 * Create and configure the MCP server with all tools registered.
 */
export function createMcpServer(): McpServer {
  const server = new McpServer(
    {
      name: "botcore-mcp-server",
      version: "1.0.0",
    },
    {
      capabilities: {
        tools: {},
        resources: {},
      },
      instructions: `BotCore Trading Bot MCP Server.
Provides tools to monitor, control, and tune a cryptocurrency trading system.
Services: Rust Core Engine (trading), Python AI Service (ML/GPT-4), and 13+ infrastructure services.
Safety: 4-tier security (PUBLIC, AUTHENTICATED, SENSITIVE, CRITICAL). Write operations require confirmation.`,
    }
  );

  // Register all tool categories
  registerHealthTools(server);     // 4 tools - system health & Docker monitoring
  registerMarketTools(server);     // 8 tools - market data & symbols
  registerTradingTools(server);    // 4 tools - live trading positions
  registerPaperTradingTools(server); // 28 tools - paper trading engine
  registerRealTradingTools(server);  // 14 tools - real trading (CAUTION)
  registerAiTools(server);         // 12 tools - AI analysis & predictions
  registerTaskTools(server);       // 7 tools - AI tasks & chat
  registerMonitoringTools(server); // 5 tools - system & trading metrics
  registerSettingsTools(server);   // 10 tools - API keys & notifications
  registerAuthTools(server);       // 4 tools - authentication
  registerTuningTools(server);     // 8 tools - self-tuning engine
  registerNotificationTools(server); // 1 tool - Telegram notifications

  log("info", "MCP server created with all tools registered");
  return server;
}
