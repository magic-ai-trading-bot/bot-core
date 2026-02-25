import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerPaperTradingTools(server: McpServer): void {
  // ============================================================================
  // READ-ONLY TOOLS
  // ============================================================================

  server.registerTool(
    "get_paper_trading_status",
    {
      title: "Get Paper Trading Status",
      description:
        "Get current paper trading engine status (running/stopped, active positions count, P&L, daily stats)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/paper-trading/status", {
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get status");
    }
  );

  server.registerTool(
    "get_paper_portfolio",
    {
      title: "Get Paper Portfolio",
      description:
        "Get paper trading portfolio details (balance, equity, margin, positions, performance metrics)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/paper-trading/portfolio", {
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get portfolio");
    }
  );

  server.registerTool(
    "get_paper_open_trades",
    {
      title: "Get Paper Open Trades",
      description:
        "Get all currently open paper trading positions with P&L, entry price, current price, size",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/trades/open",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get open trades");
    }
  );

  server.registerTool(
    "get_paper_closed_trades",
    {
      title: "Get Paper Closed Trades",
      description:
        "Get all closed paper trading positions with realized P&L, entry/exit prices, duration, win/loss status",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/trades/closed",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get closed trades");
    }
  );

  server.registerTool(
    "get_paper_strategy_settings",
    {
      title: "Get Paper Strategy Settings",
      description:
        "Get current paper trading strategy settings (RSI, MACD, Bollinger Bands, Volume thresholds, enable/disable flags)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/strategy-settings",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get strategy settings");
    }
  );

  server.registerTool(
    "get_paper_basic_settings",
    {
      title: "Get Paper Basic Settings",
      description:
        "Get paper trading basic settings (initial balance, max positions, position size, leverage, timeframe)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/basic-settings",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get basic settings");
    }
  );

  server.registerTool(
    "get_paper_symbols",
    {
      title: "Get Paper Trading Symbol Settings",
      description:
        "Get per-symbol settings (leverage, stop_loss_pct, take_profit_pct, position_size_pct, max_positions, enabled). " +
        "IMPORTANT: These per-symbol values OVERRIDE global defaults from basic-settings. Always check these to know the ACTUAL settings used for each symbol.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/paper-trading/symbols", {
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get symbols");
    }
  );

  server.registerTool(
    "get_paper_indicator_settings",
    {
      title: "Get Paper Indicator Settings",
      description:
        "Get technical indicator settings (RSI period/oversold/overbought, MACD fast/slow/signal, BB period/std dev, volume lookback)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/indicator-settings",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get indicator settings");
    }
  );

  server.registerTool(
    "get_paper_trade_analyses",
    {
      title: "Get Paper Trade Analyses",
      description:
        "Get GPT-4 analyses for all closed paper trades (AI commentary on strategy performance, mistakes, suggestions)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/trade-analyses",
        { timeoutMs: 15_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get trade analyses");
    }
  );

  server.registerTool(
    "get_paper_trade_analysis",
    {
      title: "Get Paper Trade Analysis",
      description:
        "Get GPT-4 analysis for a specific closed trade by trade ID (detailed AI commentary on entry/exit, strategy effectiveness)",
      inputSchema: {
        trade_id: z
          .string()
          .describe("The trade ID to get analysis for (e.g., 'trade_123')"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ trade_id }: { trade_id: string }) => {
      const res = await apiRequest(
        "rust",
        `/api/paper-trading/trade-analyses/${trade_id}`,
        { timeoutMs: 15_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get trade analysis");
    }
  );

  server.registerTool(
    "get_paper_config_suggestions",
    {
      title: "Get Paper Config Suggestions",
      description:
        "Get all GPT-4 configuration suggestions based on paper trading performance (AI recommendations to improve strategy)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/config-suggestions",
        { timeoutMs: 15_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get config suggestions");
    }
  );

  server.registerTool(
    "get_paper_latest_config_suggestions",
    {
      title: "Get Latest Paper Config Suggestions",
      description:
        "Get the most recent GPT-4 configuration suggestions (latest AI recommendations for strategy improvement)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/config-suggestions/latest",
        { timeoutMs: 15_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get latest config suggestions");
    }
  );

  server.registerTool(
    "get_paper_signals_history",
    {
      title: "Get Paper Signals History",
      description:
        "Get all paper trading signals history (buy/sell signals generated by strategies, with timestamp and reasoning)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/signals-history",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get signals history");
    }
  );

  server.registerTool(
    "get_paper_latest_signals",
    {
      title: "Get Latest Paper Signals",
      description:
        "Get the most recent paper trading signals (latest buy/sell signals from all active strategies)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/latest-signals",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get latest signals");
    }
  );

  server.registerTool(
    "get_paper_pending_orders",
    {
      title: "Get Paper Pending Orders",
      description:
        "Get all pending paper trading orders (limit orders, stop-loss, take-profit not yet executed)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/pending-orders",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get pending orders");
    }
  );

  // ============================================================================
  // WRITE TOOLS
  // ============================================================================

  server.registerTool(
    "close_paper_trade",
    {
      title: "Close Paper Trade",
      description:
        "Close a specific paper trading position by trade ID (manually exit position at current market price)",
      inputSchema: {
        trade_id: z
          .string()
          .describe("The trade ID to close (e.g., 'trade_123')"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ trade_id }: { trade_id: string }) => {
      const res = await apiRequest(
        "rust",
        `/api/paper-trading/trades/${trade_id}/close`,
        { method: "POST", body: {}, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to close trade");
    }
  );

  server.registerTool(
    "close_paper_trade_by_symbol",
    {
      title: "Close Paper Trade by Symbol",
      description:
        "Close an open paper trading position by symbol (e.g., ETHUSDT). Automatically finds the trade ID and closes it. Use this when you know the symbol but not the trade ID.",
      inputSchema: {
        symbol: z
          .string()
          .describe("The trading pair symbol to close (e.g., 'ETHUSDT', 'BTCUSDT')"),
        reason: z
          .string()
          .optional()
          .describe("Optional reason for closing (e.g., 'take profit', 'manual close')"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ symbol, reason }: { symbol: string; reason?: string }) => {
      // Step 1: Get open trades
      const tradesRes = await apiRequest(
        "rust",
        "/api/paper-trading/trades/open",
        { timeoutMs: 10_000 }
      );
      if (!tradesRes.success) {
        return toolError(tradesRes.error || "Failed to get open trades");
      }

      // Step 2: Find trade by symbol
      const rawData = tradesRes.data as Record<string, unknown> | unknown[];
      const trades = Array.isArray(rawData)
        ? rawData
        : Array.isArray((rawData as Record<string, unknown>)?.trades)
          ? (rawData as Record<string, unknown>).trades as unknown[]
          : [];
      const upperSymbol = symbol.toUpperCase();
      const trade = (trades as Record<string, unknown>[]).find(
        (t) => (t.symbol as string)?.toUpperCase() === upperSymbol
      );

      if (!trade) {
        return toolError(
          `No open position found for ${upperSymbol}. Use get_paper_open_trades to see all open positions.`
        );
      }

      const tradeId = (trade.id || trade.trade_id) as string;
      if (!tradeId) {
        return toolError(`Found position for ${upperSymbol} but could not determine trade ID.`);
      }

      // Step 3: Close the trade
      const closeRes = await apiRequest(
        "rust",
        `/api/paper-trading/trades/${tradeId}/close`,
        { method: "POST", body: { reason: reason || `Manual close ${upperSymbol}` }, timeoutMs: 10_000 }
      );
      const closeData = (closeRes.data ?? {}) as Record<string, unknown>;
      return closeRes.success
        ? toolSuccess({ ...closeData, closed_trade_id: tradeId, symbol: upperSymbol })
        : toolError(closeRes.error || `Failed to close ${upperSymbol} trade (ID: ${tradeId})`);
    }
  );

  server.registerTool(
    "update_paper_strategy_settings",
    {
      title: "Update Paper Strategy Settings",
      description:
        "Update paper trading strategy settings (enable/disable strategies like RSI/MACD/BB/Volume/Stochastic, adjust strategy-specific thresholds). Note: For stop loss, take profit, trailing stop, use update_paper_basic_settings instead.",
      inputSchema: {
        settings: z
          .record(z.unknown())
          .describe(
            "Strategy settings object (e.g., {rsi_enabled: true, macd_threshold: 0.5, stochastic_enabled: true})"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/strategy-settings",
        { method: "PUT", body: settings, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update strategy settings");
    }
  );

  server.registerTool(
    "update_paper_basic_settings",
    {
      title: "Update Paper Basic & Risk Settings",
      description:
        "Update paper trading basic and risk settings. Supports ALL fields: " +
        "Basic: initial_balance, max_positions, default_position_size_pct, default_leverage, trading_fee_rate, funding_fee_rate, slippage_pct, enabled, auto_restart. " +
        "Risk: max_risk_per_trade_pct, max_portfolio_risk_pct, default_stop_loss_pct, default_take_profit_pct, max_leverage, min_margin_level, max_drawdown_pct, daily_loss_limit_pct, max_consecutive_losses, cool_down_minutes, " +
        "trailing_stop_enabled, trailing_stop_pct, trailing_activation_pct, " +
        "position_sizing_method (FixedPercentage|RiskBased|VolatilityAdjusted|ConfidenceWeighted|Composite), min_risk_reward_ratio, correlation_limit, dynamic_sizing, volatility_lookback_hours, " +
        "enable_signal_reversal, ai_auto_enable_reversal, reversal_min_confidence, reversal_max_pnl_pct, reversal_allowed_regimes (array of strings). " +
        "MARKET REGIME: short_only_mode (true=block ALL Long signals, bearish market), long_only_mode (true=block ALL Short signals, bullish market). " +
        "IMPORTANT: These modes are PERSISTED to DB and survive restarts. Toggle based on market conditions.",
      inputSchema: {
        settings: z
          .record(z.unknown())
          .describe(
            "Settings object. Examples: {default_stop_loss_pct: 3.0} or {trailing_stop_enabled: true, trailing_stop_pct: 2.5} or {max_positions: 3, default_leverage: 5}"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/basic-settings",
        { method: "PUT", body: settings, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update basic settings");
    }
  );

  server.registerTool(
    "update_paper_symbols",
    {
      title: "Update Paper Trading Symbol Settings",
      description:
        "Update per-symbol trading settings (leverage, stop_loss_pct, take_profit_pct, position_size_pct, max_positions, enabled). " +
        "IMPORTANT: Per-symbol settings OVERRIDE global defaults. " +
        "If you change global stop_loss via basic-settings or green-adjustment, you MUST also update per-symbol stop_loss_pct here. " +
        "Values: stop_loss_pct and take_profit_pct are PnL-based (not price). With 10x leverage, stop_loss_pct=15 means 1.5% price move.",
      inputSchema: {
        symbols: z
          .record(
            z.object({
              enabled: z.boolean().describe("Whether trading is enabled for this symbol"),
              leverage: z.number().optional().describe("Leverage for this symbol (e.g., 10)"),
              position_size_pct: z.number().optional().describe("Position size as % of equity (e.g., 5.0)"),
              stop_loss_pct: z.number().optional().describe("Stop loss PnL% (e.g., 15.0 = 1.5% price with 10x leverage)"),
              take_profit_pct: z.number().optional().describe("Take profit PnL% (e.g., 20.0 = 2.0% price with 10x leverage)"),
              max_positions: z.number().optional().describe("Max concurrent positions for this symbol"),
            })
          )
          .describe(
            'Map of symbol → config. Example: {"BTCUSDT": {"enabled": true, "leverage": 10, "stop_loss_pct": 15.0, "take_profit_pct": 20.0, "position_size_pct": 5.0, "max_positions": 1}}'
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ symbols }: { symbols: Record<string, { enabled: boolean; leverage?: number; position_size_pct?: number; stop_loss_pct?: number; take_profit_pct?: number; max_positions?: number }> }) => {
      const res = await apiRequest("rust", "/api/paper-trading/symbols", {
        method: "PUT",
        body: { symbols },
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update symbol settings");
    }
  );

  server.registerTool(
    "update_paper_indicator_settings",
    {
      title: "Update Paper Indicator Settings",
      description:
        "Update technical indicator settings (RSI period/thresholds, MACD parameters, Bollinger Bands period/std dev, volume lookback)",
      inputSchema: {
        settings: z
          .record(z.unknown())
          .describe(
            "Indicator settings object (e.g., {rsi_period: 14, rsi_oversold: 30, macd_fast: 12})"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/indicator-settings",
        { method: "PUT", body: settings, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update indicator settings");
    }
  );

  server.registerTool(
    "reset_paper_account",
    {
      title: "Reset Paper Trading Account",
      description:
        "Reset the paper trading account to initial balance, closing all open positions and clearing trade history",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/paper-trading/reset", {
        method: "POST",
        body: {},
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to reset account");
    }
  );

  server.registerTool(
    "start_paper_engine",
    {
      title: "Start Paper Trading Engine",
      description:
        "Start the paper trading engine to begin automated trading based on configured strategies",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/paper-trading/start", {
        method: "POST",
        body: {},
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to start engine");
    }
  );

  server.registerTool(
    "stop_paper_engine",
    {
      title: "Stop Paper Trading Engine",
      description:
        "Stop the paper trading engine, pausing automated trading (open positions remain active)",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/paper-trading/stop", {
        method: "POST",
        body: {},
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to stop engine");
    }
  );

  server.registerTool(
    "create_paper_order",
    {
      title: "Create Paper Order",
      description:
        "Create a manual paper trading order (market, limit, stop-loss, take-profit)",
      inputSchema: {
        symbol: z.string().describe("Trading symbol (e.g., 'BTCUSDT')"),
        side: z.enum(["buy", "sell"]).describe("Order side (buy or sell)"),
        order_type: z
          .string()
          .describe("Order type (e.g., 'market', 'limit', 'stop_loss')"),
        quantity: z.number().optional().describe("Order quantity (optional)"),
        price: z
          .number()
          .optional()
          .describe("Limit price for limit orders (optional)"),
        stop_price: z
          .number()
          .optional()
          .describe("Stop price for stop orders (optional)"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({
      symbol,
      side,
      order_type,
      quantity,
      price,
      stop_price,
    }: {
      symbol: string;
      side: "buy" | "sell";
      order_type: string;
      quantity?: number;
      price?: number;
      stop_price?: number;
    }) => {
      const res = await apiRequest("rust", "/api/paper-trading/orders", {
        method: "POST",
        body: { symbol, side, order_type, quantity, price, stop_price },
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to create order");
    }
  );

  server.registerTool(
    "cancel_paper_order",
    {
      title: "Cancel Paper Order",
      description:
        "Cancel a pending paper trading order by order ID (remove limit/stop order before execution)",
      inputSchema: {
        order_id: z
          .string()
          .describe("The order ID to cancel (e.g., 'order_123')"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ order_id }: { order_id: string }) => {
      const res = await apiRequest(
        "rust",
        `/api/paper-trading/pending-orders/${order_id}`,
        { method: "DELETE", timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to cancel order");
    }
  );

  server.registerTool(
    "trigger_paper_analysis",
    {
      title: "Trigger Paper Trade Analysis",
      description:
        "Manually trigger GPT-4 analysis of recent closed trades to get AI insights and strategy suggestions",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/trigger-analysis",
        { method: "POST", body: {}, timeoutMs: 30_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to trigger analysis");
    }
  );

  server.registerTool(
    "update_paper_signal_interval",
    {
      title: "Update Paper Signal Interval",
      description:
        "Update the interval (in seconds) for generating new trading signals from strategies",
      inputSchema: {
        interval_seconds: z
          .number()
          .describe(
            "Signal generation interval in seconds (e.g., 60 for 1 minute)"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ interval_seconds }: { interval_seconds: number }) => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/signal-interval",
        { method: "PUT", body: { interval_seconds }, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update signal interval");
    }
  );

  server.registerTool(
    "update_paper_settings",
    {
      title: "Update Paper Settings (Generic)",
      description:
        "Update generic paper trading settings (catch-all for settings not covered by specific endpoints)",
      inputSchema: {
        settings: z
          .record(z.unknown())
          .describe(
            "Generic settings object with any paper trading configuration"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest("rust", "/api/paper-trading/settings", {
        method: "PUT",
        body: settings,
        timeoutMs: 10_000,
      });
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update settings");
    }
  );

  // ============================================================================
  // EXECUTION SETTINGS TOOLS
  // ============================================================================

  server.registerTool(
    "get_paper_execution_settings",
    {
      title: "Get Paper Execution Settings",
      description:
        "Get paper trading execution settings (auto_execution, slippage simulation, partial fills, market impact, execution delay, order expiration)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/execution-settings",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get execution settings");
    }
  );

  server.registerTool(
    "update_paper_execution_settings",
    {
      title: "Update Paper Execution Settings",
      description:
        "Update paper trading execution settings. Supports: auto_execution, execution_delay_ms, simulate_partial_fills, partial_fill_probability, order_expiration_minutes, simulate_slippage, max_slippage_pct, simulate_market_impact, market_impact_factor, price_update_frequency_seconds",
      inputSchema: {
        settings: z
          .record(z.unknown())
          .describe(
            "Execution settings object. Examples: {simulate_slippage: true, max_slippage_pct: 0.05} or {auto_execution: false}"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/execution-settings",
        { method: "PUT", body: settings, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update execution settings");
    }
  );

  // ============================================================================
  // AI SETTINGS TOOLS
  // ============================================================================

  server.registerTool(
    "get_paper_ai_settings",
    {
      title: "Get Paper AI Settings",
      description:
        "Get paper trading AI integration settings (service URL, timeouts, signal refresh interval, realtime signals, feedback learning, strategy recommendations, model tracking, confidence_thresholds per market regime)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/ai-settings",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get AI settings");
    }
  );

  server.registerTool(
    "update_paper_ai_settings",
    {
      title: "Update Paper AI Settings",
      description:
        "Update paper trading AI settings. Supports: service_url, request_timeout_seconds, signal_refresh_interval_minutes, enable_realtime_signals, enable_feedback_learning, feedback_delay_hours, enable_strategy_recommendations, track_model_performance, confidence_thresholds (object: {regime: threshold}, e.g. {\"trending\": 0.65, \"ranging\": 0.75})",
      inputSchema: {
        settings: z
          .record(z.unknown())
          .describe(
            "AI settings object. Examples: {signal_refresh_interval_minutes: 10} or {enable_feedback_learning: true, feedback_delay_hours: 2}"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/ai-settings",
        { method: "PUT", body: settings, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update AI settings");
    }
  );

  // ============================================================================
  // NOTIFICATION SETTINGS TOOLS
  // ============================================================================

  server.registerTool(
    "get_paper_notification_settings",
    {
      title: "Get Paper Notification Settings",
      description:
        "Get paper trading notification settings (trade notifications, performance notifications, risk warnings, daily summary, weekly report, min P&L threshold, max notifications per hour)",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/notification-settings",
        { timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to get notification settings");
    }
  );

  server.registerTool(
    "update_paper_notification_settings",
    {
      title: "Update Paper Notification Settings",
      description:
        "Update paper trading notification settings. Supports: enable_trade_notifications, enable_performance_notifications, enable_risk_warnings, daily_summary, weekly_report, min_pnl_notification, max_notifications_per_hour",
      inputSchema: {
        settings: z
          .record(z.unknown())
          .describe(
            "Notification settings object. Examples: {enable_risk_warnings: true} or {daily_summary: false, max_notifications_per_hour: 10}"
          ),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ settings }: { settings: Record<string, unknown> }) => {
      const res = await apiRequest(
        "rust",
        "/api/paper-trading/notification-settings",
        { method: "PUT", body: settings, timeoutMs: 10_000 }
      );
      return res.success
        ? toolSuccess(res.data)
        : toolError(res.error || "Failed to update notification settings");
    }
  );

  log("info", "Paper trading tools registered (34 tools)");
}
