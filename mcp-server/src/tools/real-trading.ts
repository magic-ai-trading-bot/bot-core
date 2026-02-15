// @spec:FR-MCP-006 - Real Trading Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-tool-implementation.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerRealTradingTools(server: McpServer): void {
  // All tools target Rust service (port 8080)

  // ====== Read-Only Tools ======

  server.registerTool(
    "get_real_trading_status",
    {
      title: "Get Real Trading Status",
      description: "Get current status of the real trading engine including whether it's running, account balance, and active positions.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/status");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch real trading status");
    }
  );

  server.registerTool(
    "get_real_portfolio",
    {
      title: "Get Real Portfolio",
      description: "Get current real trading portfolio including account balance, open positions, and performance metrics.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/portfolio");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch real portfolio");
    }
  );

  server.registerTool(
    "get_real_open_trades",
    {
      title: "Get Real Open Trades",
      description: "Get all currently open real trades with details including entry price, current P&L, and stop-loss/take-profit levels.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/trades/open");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch open trades");
    }
  );

  server.registerTool(
    "get_real_closed_trades",
    {
      title: "Get Real Closed Trades",
      description: "Get history of closed real trades with P&L, duration, and exit reasons.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/trades/closed");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch closed trades");
    }
  );

  server.registerTool(
    "get_real_trading_settings",
    {
      title: "Get Real Trading Settings",
      description: "Get current real trading settings including risk parameters, position sizing, and strategy configurations.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/settings");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch real trading settings");
    }
  );

  server.registerTool(
    "get_real_orders",
    {
      title: "Get Real Orders",
      description: "Get all active orders (pending, filled, cancelled) on the real trading account.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/orders");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch real orders");
    }
  );

  // ====== Write Tools (CAUTION: Real Money) ======

  server.registerTool(
    "start_real_engine",
    {
      title: "Start Real Trading Engine",
      description: "⚠️ CAUTION: Start the real trading engine. This will begin executing real trades with real money. Ensure settings are correct before starting.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/start", {
        method: "POST",
        body: {},
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to start real trading engine");
    }
  );

  server.registerTool(
    "stop_real_engine",
    {
      title: "Stop Real Trading Engine",
      description: "Stop the real trading engine. This will halt all new trade execution but leave existing positions open.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/stop", {
        method: "POST",
        body: {},
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to stop real trading engine");
    }
  );

  server.registerTool(
    "close_real_trade",
    {
      title: "Close Real Trade",
      description: "⚠️ CAUTION: Close a specific real trade immediately at market price. This action cannot be undone.",
      inputSchema: {
        trade_id: z.string().describe("ID of the trade to close"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ trade_id }: { trade_id: string }) => {
      const res = await apiRequest("rust", `/api/real-trading/trades/${trade_id}/close`, {
        method: "POST",
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to close real trade");
    }
  );

  server.registerTool(
    "update_real_trading_settings",
    {
      title: "Update Real Trading Settings",
      description: "⚠️ CAUTION: Update real trading settings. Changes will affect future trades and may impact risk management.",
      inputSchema: {
        settings: z.record(z.unknown()).describe("Settings object with fields to update"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest("rust", "/api/real-trading/settings", {
        method: "PUT",
        body: settings,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to update real trading settings");
    }
  );

  server.registerTool(
    "create_real_order",
    {
      title: "Create Real Order",
      description: "⚠️ CAUTION: Create a new real order on the exchange. This will execute with real money.",
      inputSchema: {
        symbol: z.string().describe("Trading pair symbol (e.g., BTCUSDT)"),
        side: z.enum(["buy", "sell"]).describe("Order side"),
        order_type: z.string().describe("Order type (market, limit, stop_loss, etc.)"),
        quantity: z.number().optional().describe("Order quantity"),
        price: z.number().optional().describe("Order price (for limit orders)"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ symbol, side, order_type, quantity, price }: { symbol: string; side: "buy" | "sell"; order_type: string; quantity?: number; price?: number }) => {
      const res = await apiRequest("rust", "/api/real-trading/orders", {
        method: "POST",
        body: { symbol, side, order_type, quantity, price },
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to create real order");
    }
  );

  server.registerTool(
    "cancel_real_order",
    {
      title: "Cancel Real Order",
      description: "Cancel a specific pending order on the real exchange.",
      inputSchema: {
        id: z.string().describe("Order ID to cancel"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ id }: { id: string }) => {
      const res = await apiRequest("rust", `/api/real-trading/orders/${id}`, {
        method: "DELETE",
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to cancel real order");
    }
  );

  server.registerTool(
    "cancel_all_real_orders",
    {
      title: "Cancel All Real Orders",
      description: "⚠️ CAUTION: Cancel ALL pending orders on the real exchange. This action affects all symbols.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/real-trading/orders/all", {
        method: "DELETE",
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to cancel all real orders");
    }
  );

  server.registerTool(
    "update_real_position_sltp",
    {
      title: "Update Position Stop-Loss/Take-Profit",
      description: "Update stop-loss and/or take-profit levels for an open real position.",
      inputSchema: {
        symbol: z.string().describe("Trading pair symbol (e.g., BTCUSDT)"),
        stop_loss: z.number().optional().describe("New stop-loss price"),
        take_profit: z.number().optional().describe("New take-profit price"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ symbol, stop_loss, take_profit }: { symbol: string; stop_loss?: number; take_profit?: number }) => {
      const res = await apiRequest("rust", `/api/real-trading/positions/${symbol}/sltp`, {
        method: "PUT",
        body: { stop_loss, take_profit },
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to update position SL/TP");
    }
  );

  log("info", "Registered 14 real trading tools (6 read-only + 8 write operations)");
}
