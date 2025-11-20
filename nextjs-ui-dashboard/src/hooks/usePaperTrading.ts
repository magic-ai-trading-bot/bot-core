import { useState, useEffect, useCallback, useRef } from "react";
import logger from "@/utils/logger";
import { useToast } from "@/hooks/use-toast";

// Types for Rust backend integration


// @spec:FR-PAPER-001 (Frontend) - `nextjs-ui-dashboard/src/hooks/usePaperTrading.ts:56-245`
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#paper-trading
// @test:--- ## Design to Test Mapping

interface RustPaperTradingResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

interface SimpleApiResponse {
  message: string;
}

// Paper Trading Types - matching Rust backend
export interface PaperTradingSettings {
  basic: {
    initial_balance: number;
    max_positions: number;
    default_position_size_pct: number;
    default_leverage: number;
    trading_fee_rate: number;
    funding_fee_rate: number;
    slippage_pct: number;
    enabled: boolean;
    auto_restart: boolean;
  };
  risk: {
    max_risk_per_trade_pct: number;
    max_portfolio_risk_pct: number;
    default_stop_loss_pct: number;
    default_take_profit_pct: number;
    max_leverage: number;
    min_margin_level: number;
    max_drawdown_pct: number;
    daily_loss_limit_pct: number;
    max_consecutive_losses: number;
    cool_down_minutes: number;
  };
}

export interface PaperTrade {
  id: string;
  symbol: string;
  trade_type: "Long" | "Short";
  status: "Open" | "Closed" | "Cancelled";
  entry_price: number;
  exit_price?: number;
  quantity: number;
  leverage: number;
  stop_loss?: number;
  take_profit?: number;
  pnl?: number;
  pnl_percentage: number;
  duration_ms?: number;
  open_time: string;
  close_time?: string;
}

export interface PortfolioMetrics {
  total_trades: number;
  win_rate: number;
  total_pnl: number;
  total_pnl_percentage: number;
  max_drawdown: number;
  max_drawdown_percentage: number;
  sharpe_ratio: number;
  profit_factor: number;
  average_win: number;
  average_loss: number;
  largest_win: number;
  largest_loss: number;
  current_balance: number;
  equity: number;
  margin_used: number;
  free_margin: number;
}

export interface AISignal {
  id: string;
  signal: string;
  symbol: string;
  confidence: number;
  timestamp: Date;
  reasoning: string;
  strategy_scores: Record<string, number>;
  market_analysis: {
    trend_direction: string;
    trend_strength: number;
    support_levels: number[];
    resistance_levels: number[];
    volatility_level: string;
    volume_analysis: string;
  };
  risk_assessment: {
    overall_risk: string;
    technical_risk: number;
    market_risk: number;
    recommended_position_size: number;
    stop_loss_suggestion: number | null;
    take_profit_suggestion: number | null;
  };
}

export interface PaperTradingState {
  isActive: boolean;
  portfolio: PortfolioMetrics;
  openTrades: PaperTrade[];
  closedTrades: PaperTrade[];
  settings: PaperTradingSettings;
  recentSignals: AISignal[];
  isLoading: boolean;
  error: string | null;
  lastUpdated: Date | null;
  updateCounter: number;
}

const defaultSettings: PaperTradingSettings = {
  basic: {
    initial_balance: 10000,
    max_positions: 10,
    default_position_size_pct: 5.0,
    default_leverage: 10,
    trading_fee_rate: 0.0004,
    funding_fee_rate: 0.0001,
    slippage_pct: 0.01,
    enabled: true,
    auto_restart: false,
  },
  risk: {
    max_risk_per_trade_pct: 2.0,
    max_portfolio_risk_pct: 20.0,
    default_stop_loss_pct: 2.0,
    default_take_profit_pct: 4.0,
    max_leverage: 50,
    min_margin_level: 200.0,
    max_drawdown_pct: 15.0,
    daily_loss_limit_pct: 5.0,
    max_consecutive_losses: 5,
    cool_down_minutes: 60,
  },
};

const defaultPortfolio: PortfolioMetrics = {
  total_trades: 0,
  win_rate: 0,
  total_pnl: 0,
  total_pnl_percentage: 0,
  max_drawdown: 0,
  max_drawdown_percentage: 0,
  sharpe_ratio: 0,
  profit_factor: 0,
  average_win: 0,
  average_loss: 0,
  largest_win: 0,
  largest_loss: 0,
  current_balance: 10000,
  equity: 10000,
  margin_used: 0,
  free_margin: 10000,
};

export const usePaperTrading = () => {
  const { toast } = useToast();
  const [state, setState] = useState<PaperTradingState>({
    isActive: false,
    portfolio: defaultPortfolio,
    openTrades: [],
    closedTrades: [],
    settings: defaultSettings,
    recentSignals: [],
    isLoading: false,
    error: null,
    lastUpdated: null,
    updateCounter: 0,
  });

  // API base URL - should match your Rust backend
  const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

  // Fetch bot status to check if it's running
  const fetchBotStatus = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/api/paper-trading/status`);
      const data: RustPaperTradingResponse<{
        is_running: boolean;
        portfolio: PortfolioMetrics;
        last_updated: string;
      }> = await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          isActive: data.data!.is_running,
          portfolio: data.data!.portfolio,
          lastUpdated: new Date(),
        }));
      }
    } catch (error) {
      logger.error("Failed to fetch bot status:", error);
    }
  }, [API_BASE]);

  // Fetch portfolio status from Rust backend
  const fetchPortfolioStatus = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      const response = await fetch(`${API_BASE}/api/paper-trading/portfolio`);
      const data: RustPaperTradingResponse<PortfolioMetrics> =
        await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          portfolio: data.data!,
          isLoading: false,
          lastUpdated: new Date(),
        }));
      } else {
        throw new Error(data.error || "Failed to fetch portfolio status");
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : "Unknown error",
      }));
    }
  }, [API_BASE]);

  // Fetch open trades
  const fetchOpenTrades = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/api/paper-trading/trades/open`);
      const data: RustPaperTradingResponse<PaperTrade[]> =
        await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          openTrades: data.data!,
        }));
      }
    } catch (error) {
      logger.error("Failed to fetch open trades:", error);
    }
  }, [API_BASE]);

  // Helper: Fetch with retry logic
  const fetchWithRetry = useCallback(async (url: string, retries = 3) => {
    for (let i = 0; i < retries; i++) {
      try {
        const response = await fetch(url);

        if (!response.ok) {
          throw new Error(`HTTP ${response.status}: ${response.statusText}`);
        }

        return await response.json();
      } catch (error) {
        if (i === retries - 1) throw error; // Last attempt failed
        await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1))); // Exponential backoff
      }
    }
  }, []);

  // Fetch closed trades
  const fetchClosedTrades = useCallback(async () => {
    try {
      const data: RustPaperTradingResponse<PaperTrade[]> = await fetchWithRetry(
        `${API_BASE}/api/paper-trading/trades/closed`
      );

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          closedTrades: data.data!,
        }));
      } else {
        logger.warn("Empty or failed response for closed trades:", data.error);
        toast({
          title: "Warning",
          description: `Failed to fetch trades: ${data.error || "Unknown error"}`,
          variant: "destructive",
        });
      }
    } catch (error) {
      logger.error("Failed to fetch closed trades after retries:", error);
      toast({
        title: "Error",
        description: "Unable to connect to trading service. Please try again.",
        variant: "destructive",
      });
    }
  }, [API_BASE, fetchWithRetry, toast]);

  // Fetch current settings from backend
  const fetchCurrentSettings = useCallback(async () => {
    try {
      const response = await fetch(
        `${API_BASE}/api/paper-trading/basic-settings`,
        {
          method: "GET",
          headers: {
            "Content-Type": "application/json",
          },
        }
      );

      const data: RustPaperTradingResponse<PaperTradingSettings> =
        await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          settings: data.data!,
          lastUpdated: new Date(),
        }));
      }
    } catch (error) {
      logger.error("Failed to fetch current settings:", error);
      // Don't set error for settings fetch failure to avoid disrupting the app
    }
  }, [API_BASE]);

  // Helper function to deduplicate signals by symbol and recency
  const deduplicateSignals = useCallback((signals: AISignal[]): AISignal[] => {
    const signalMap = new Map<string, AISignal>();

    // Sort signals by timestamp (newest first)
    const sortedSignals = [...signals].sort(
      (a, b) =>
        new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );

    // Keep only the most recent signal for each symbol
    for (const signal of sortedSignals) {
      if (!signalMap.has(signal.symbol)) {
        // Only keep signals from the last 30 minutes
        const signalAge = Date.now() - new Date(signal.timestamp).getTime();
        if (signalAge < 30 * 60 * 1000) {
          // 30 minutes
          signalMap.set(signal.symbol, signal);
        }
      }
    }

    return Array.from(signalMap.values()).slice(0, 8); // Limit to 8 signals max
  }, []);

  // Manual fetch AI signals (for refresh button only)
  const fetchAISignals = useCallback(async () => {
    try {
      const symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];
      const signalsPromises = symbols.map(async (symbol) => {
        // Get recent candle data for AI analysis
        const now = Date.now();
        const sampleData = {
          symbol,
          timeframe_data: {
            "1h": [
              {
                open_time: now - 3600000,
                close_time: now,
                open: 50000.0,
                high: 51000.0,
                low: 49500.0,
                close: 50500.0,
                volume: 1000.0,
                quote_volume: 50000000.0,
                trades: 10000,
                is_closed: true,
              },
            ],
          },
          current_price: 50500.0,
          volume_24h: 75000.0,
          timestamp: now,
          strategy_context: {
            selected_strategies: ["RSI Strategy", "MACD Strategy"],
            market_condition: "Trending",
            risk_level: "Moderate",
            user_preferences: {},
            technical_indicators: {},
          },
        };

        const response = await fetch(`${API_BASE}/api/ai/analyze`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(sampleData),
        });

        const result = await response.json();
        if (result.success && result.data) {
          return {
            id: `${symbol}-${now}`,
            signal: result.data.signal,
            symbol,
            confidence: result.data.confidence,
            timestamp: new Date(now),
            reasoning: result.data.reasoning,
            strategy_scores: result.data.strategy_scores,
            market_analysis: result.data.market_analysis,
            risk_assessment: result.data.risk_assessment,
          } as AISignal;
        }
        return null;
      });

      const signals = await Promise.all(signalsPromises);
      const validSignals = signals.filter(
        (signal): signal is AISignal => signal !== null
      );

      setState((prev) => {
        // Merge with existing signals and deduplicate
        const allSignals = [...validSignals, ...prev.recentSignals];
        const deduplicatedSignals = deduplicateSignals(allSignals);

        return {
          ...prev,
          recentSignals: deduplicatedSignals,
        };
      });
    } catch (error) {
      logger.error("Failed to fetch AI signals:", error);
    }
  }, [API_BASE, deduplicateSignals]);

  // Start paper trading
  const startTrading = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      const response = await fetch(`${API_BASE}/api/paper-trading/start`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const data: RustPaperTradingResponse<SimpleApiResponse> =
        await response.json();

      if (data.success) {
        setState((prev) => ({
          ...prev,
          isActive: true,
          isLoading: false,
        }));
        // Refresh data after starting
        await fetchPortfolioStatus();
        await fetchOpenTrades();
      } else {
        throw new Error(data.error || "Failed to start trading");
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : "Unknown error",
      }));
    }
  }, [API_BASE, fetchPortfolioStatus, fetchOpenTrades]);

  // Stop paper trading
  const stopTrading = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      const response = await fetch(`${API_BASE}/api/paper-trading/stop`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const data: RustPaperTradingResponse<SimpleApiResponse> =
        await response.json();

      if (data.success) {
        setState((prev) => ({
          ...prev,
          isActive: false,
          isLoading: false,
        }));
      } else {
        throw new Error(data.error || "Failed to stop trading");
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : "Unknown error",
      }));
    }
  }, [API_BASE]);

  // Close a specific trade
  const closeTrade = useCallback(
    async (tradeId: string) => {
      try {
        const response = await fetch(
          `${API_BASE}/api/paper-trading/trades/${tradeId}/close`,
          {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              trade_id: tradeId,
              reason: "manual",
            }),
          }
        );

        const data: RustPaperTradingResponse<SimpleApiResponse> =
          await response.json();

        if (data.success) {
          // Refresh trades and portfolio
          await fetchOpenTrades();
          await fetchClosedTrades();
          await fetchPortfolioStatus();
        } else {
          throw new Error(data.error || "Failed to close trade");
        }
      } catch (error) {
        setState((prev) => ({
          ...prev,
          error: error instanceof Error ? error.message : "Unknown error",
        }));
      }
    },
    [API_BASE, fetchOpenTrades, fetchClosedTrades, fetchPortfolioStatus]
  );

  // Update settings (simplified - only basic settings)
  const updateSettings = useCallback(
    async (newSettings: PaperTradingSettings) => {
      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        // Extract only the basic and risk settings that the simplified API expects
        const basicSettingsRequest = {
          initial_balance: newSettings.basic.initial_balance,
          max_positions: newSettings.basic.max_positions,
          default_position_size_pct:
            newSettings.basic.default_position_size_pct,
          default_leverage: newSettings.basic.default_leverage,
          trading_fee_rate: newSettings.basic.trading_fee_rate,
          funding_fee_rate: newSettings.basic.funding_fee_rate,
          slippage_pct: newSettings.basic.slippage_pct,
          max_risk_per_trade_pct: newSettings.risk.max_risk_per_trade_pct,
          max_portfolio_risk_pct: newSettings.risk.max_portfolio_risk_pct,
          default_stop_loss_pct: newSettings.risk.default_stop_loss_pct,
          default_take_profit_pct: newSettings.risk.default_take_profit_pct,
          max_leverage: newSettings.risk.max_leverage,
          enabled: newSettings.basic.enabled,
        };

        const response = await fetch(
          `${API_BASE}/api/paper-trading/basic-settings`,
          {
            method: "PUT",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify(basicSettingsRequest),
          }
        );

        const data: RustPaperTradingResponse<SimpleApiResponse> =
          await response.json();

        if (data.success) {
          setState((prev) => ({
            ...prev,
            settings: newSettings,
            isLoading: false,
          }));

          // Refresh portfolio data after settings update
          await fetchPortfolioStatus();
        } else {
          throw new Error(data.error || "Failed to update settings");
        }
      } catch (error) {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: error instanceof Error ? error.message : "Unknown error",
        }));
      }
    },
    [API_BASE, fetchPortfolioStatus]
  );

  // Reset portfolio
  const resetPortfolio = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      const response = await fetch(`${API_BASE}/api/paper-trading/reset`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const data: RustPaperTradingResponse<SimpleApiResponse> =
        await response.json();

      if (data.success) {
        // Refresh all data after reset
        await fetchPortfolioStatus();
        await fetchOpenTrades();
        await fetchClosedTrades();
        setState((prev) => ({ ...prev, isLoading: false }));
      } else {
        throw new Error(data.error || "Failed to reset portfolio");
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : "Unknown error",
      }));
    }
  }, [API_BASE, fetchPortfolioStatus, fetchOpenTrades, fetchClosedTrades]);

  // Note: AI signals are now handled via WebSocket in real-time
  // The fetchAISignals function above provides manual refresh through the Rust API

  // Initial data fetch on mount (no polling - use WebSocket for real-time updates)
  useEffect(() => {
    // Only fetch initial data once on mount
    fetchBotStatus();
    fetchOpenTrades();
    fetchClosedTrades();
    fetchCurrentSettings();

    // 🎯 FIXED: Fetch initial AI signals on load
    fetchAISignals();

    // Note: AI signals are now handled via WebSocket in real-time
    // Manual refresh available via fetchAISignals function
  }, [
    fetchBotStatus,
    fetchOpenTrades,
    fetchClosedTrades,
    fetchCurrentSettings,
    fetchAISignals,
  ]);

  // Set up WebSocket connection for real-time updates
  useEffect(() => {
    const wsUrl = (
      import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws"
    ).replace("http", "ws");
    const ws = new WebSocket(wsUrl);
    let heartbeatInterval: NodeJS.Timeout | null = null;

    ws.onopen = () => {
      // Start heartbeat to keep connection alive
      heartbeatInterval = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: "ping" }));
        }
      }, 30000); // Send heartbeat every 30 seconds
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);

        // Handle new format from Rust backend
        const eventType = message.event_type || message.type;
        const data = message.data;

        switch (eventType) {
          case "MarketData":
            // Handle real-time price updates from WebSocket (same as Dashboard)
            if (data && data.symbol && data.price) {

              // Update portfolio with new price information
              setState((prev) => {
                // Guard against undefined portfolio (during initialization)
                if (!prev.portfolio || !prev.portfolio.current_balance) {
                  return { ...prev, lastUpdated: new Date() };
                }

                // Calculate approximate unrealized P&L for open trades
                let totalUnrealizedPnl = 0;

                const updatedOpenTrades = prev.openTrades.map((trade) => {
                  if (trade.symbol === data.symbol) {
                    // Calculate unrealized P&L based on trade type
                    const priceDiff = data.price - trade.entry_price;
                    const newUnrealizedPnl =
                      trade.trade_type === "Long"
                        ? priceDiff * trade.quantity
                        : -priceDiff * trade.quantity;

                    totalUnrealizedPnl += newUnrealizedPnl;

                    // Update trade with approximate P&L (actual update from backend will override)
                    return {
                      ...trade,
                      pnl: newUnrealizedPnl,
                    };
                  } else {
                    totalUnrealizedPnl += trade.pnl || 0;
                  }
                  return trade;
                });

                // Update portfolio equity with real-time price changes
                const newEquity = prev.portfolio.current_balance + totalUnrealizedPnl;
                const equityChanged = Math.abs(newEquity - prev.portfolio.equity) > 0.01;

                const updatedPortfolio = {
                  ...prev.portfolio,
                  equity: newEquity,
                  total_pnl: totalUnrealizedPnl,
                };

                return {
                  ...prev,
                  openTrades: updatedOpenTrades,
                  portfolio: updatedPortfolio,
                  // Only update timestamp if equity changed significantly to reduce re-renders
                  lastUpdated: equityChanged ? new Date() : prev.lastUpdated,
                };
              });

              // Periodically fetch fresh data to ensure accuracy (less frequent now)
              if (Math.random() < 0.05) {
                // 5% chance to fetch fresh data
                fetchPortfolioStatus();
              }
            }
            break;

          case "price_update":
            // Legacy price update format
            // Trigger refresh of portfolio data to update P&L
            if (data && Object.keys(data).length > 0) {
              fetchPortfolioStatus();
            }
            break;

          case "performance_update":
            // Real-time portfolio metrics update
            if (data) {
              setState((prev) => ({
                ...prev,
                portfolio: { ...prev.portfolio, ...data },
                lastUpdated: new Date(),
              }));
            }
            break;

          case "trade_executed":
            // Refresh both portfolio and trades
            fetchPortfolioStatus();
            fetchOpenTrades();
            break;

          case "trade_closed":
            // Refresh everything when a trade is closed
            fetchPortfolioStatus();
            fetchOpenTrades();
            fetchClosedTrades();
            break;

          case "AISignalReceived":
            // Real-time AI signals
            if (data) {
              setState((prev) => {
                // Add new signal and deduplicate all signals
                const allSignals = [data, ...prev.recentSignals];
                const deduplicatedSignals = deduplicateSignals(allSignals);

                return {
                  ...prev,
                  recentSignals: deduplicatedSignals,
                  lastUpdated: new Date(),
                };
              });
            }
            break;

          case "Connected":
            setState((prev) => ({
              ...prev,
              lastUpdated: new Date(),
            }));
            break;

          case "Pong":
            // Keep-alive response, no action needed
            break;

          default:
            // Silently ignore unknown message types
            break;
        }
      } catch (error) {
        logger.error("Failed to parse WebSocket message:", error);
      }
    };

    ws.onclose = () => {
      if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
      }
    };

    ws.onerror = (error) => {
      logger.error("📡 Paper Trading WebSocket error:", error);
      if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
      }
    };

    return () => {
      if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
      }
      if (ws.readyState === WebSocket.OPEN) {
        ws.close();
      }
    };
  }, [
    fetchPortfolioStatus,
    fetchOpenTrades,
    fetchClosedTrades,
    deduplicateSignals,
  ]);

  return {
    // State
    ...state,

    // Actions
    startTrading,
    stopTrading,
    closeTrade,
    updateSettings,
    resetPortfolio,

    // Manual refresh functions
    refreshData: fetchPortfolioStatus,
    refreshStatus: fetchBotStatus,
    refreshSettings: fetchCurrentSettings,
    refreshAISignals: fetchAISignals,
    // fetchAISignalsFromAPI removed - use WebSocket for real-time signals
    refreshTrades: useCallback(async () => {
      await fetchOpenTrades();
      await fetchClosedTrades();
    }, [fetchOpenTrades, fetchClosedTrades]),
  };
};
