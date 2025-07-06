import { useState, useEffect, useCallback, useRef } from "react";
import { useToast } from "@/hooks/use-toast";

// Types for Rust backend integration
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
      console.error("Failed to fetch bot status:", error);
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
      console.error("Failed to fetch open trades:", error);
    }
  }, [API_BASE]);

  // Fetch closed trades
  const fetchClosedTrades = useCallback(async () => {
    try {
      const response = await fetch(
        `${API_BASE}/api/paper-trading/trades/closed`
      );
      const data: RustPaperTradingResponse<PaperTrade[]> =
        await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          closedTrades: data.data!,
        }));
      }
    } catch (error) {
      console.error("Failed to fetch closed trades:", error);
    }
  }, [API_BASE]);

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
      console.error("Failed to fetch current settings:", error);
      // Don't set error for settings fetch failure to avoid disrupting the app
    }
  }, [API_BASE]);

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
        // Merge with existing signals, removing duplicates by symbol
        const existingSignals = prev.recentSignals.filter(
          (existing) =>
            !validSignals.some(
              (newSignal) => newSignal.symbol === existing.symbol
            )
        );

        return {
          ...prev,
          recentSignals: [...validSignals, ...existingSignals].slice(0, 10), // Keep latest 10
        };
      });
    } catch (error) {
      console.error("Failed to fetch AI signals:", error);
    }
  }, [API_BASE]);

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

    ws.onopen = () => {
      console.log("📡 Paper Trading WebSocket connected");
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        console.log(
          "📡 WebSocket message received:",
          message.event_type || message.type
        );

        // Handle new format from Rust backend
        const eventType = message.event_type || message.type;
        const data = message.data;

        switch (eventType) {
          case "engine_started":
            setState((prev) => ({
              ...prev,
              isActive: true,
              lastUpdated: new Date(),
            }));
            break;

          case "engine_stopped":
            setState((prev) => ({
              ...prev,
              isActive: false,
              lastUpdated: new Date(),
            }));
            break;

          case "price_update":
            // Real-time price updates (every second)
            console.log("💰 Real-time price update:", data);
            // Trigger refresh of open trades to update unrealized P&L
            if (data && Object.keys(data).length > 0) {
              fetchOpenTrades();
            }
            break;

          case "performance_update":
            // Real-time portfolio metrics update
            if (data) {
              console.log("📊 Performance update:", data);
              setState((prev) => ({
                ...prev,
                portfolio: { ...prev.portfolio, ...data },
                lastUpdated: new Date(),
              }));
            }
            break;

          case "trade_executed":
            console.log("🎯 New trade executed:", data);
            // Refresh both portfolio and trades
            fetchPortfolioStatus();
            fetchOpenTrades();
            break;

          case "trade_closed":
            console.log("🔒 Trade closed:", data);
            // Refresh everything when a trade is closed
            fetchPortfolioStatus();
            fetchOpenTrades();
            fetchClosedTrades();
            break;

          case "AISignalReceived":
            // Real-time AI signals
            if (data) {
              console.log("🤖 New AI Signal:", data);
              setState((prev) => ({
                ...prev,
                recentSignals: [data, ...prev.recentSignals.slice(0, 19)],
                lastUpdated: new Date(),
              }));
            }
            break;

          case "portfolio_reset":
            console.log("🔄 Portfolio reset");
            // Refresh everything after reset
            fetchPortfolioStatus();
            fetchOpenTrades();
            fetchClosedTrades();
            break;

          case "settings_updated":
            console.log("⚙️ Settings updated");
            fetchCurrentSettings();
            break;

          case "MarketData":
            // Handle market data updates (price updates, volume, etc.)
            // This is sent frequently by the backend but we don't need to process it
            // Just acknowledge it to avoid "unknown message type" errors
            break;

          case "ChartUpdate":
            // Handle chart data updates
            // Similar to MarketData, just acknowledge to avoid errors
            break;

          case "Connected":
            console.log(
              "📡 Paper Trading WebSocket connected:",
              message.message
            );
            break;

          case "Pong":
            // Keep-alive response, no action needed
            break;

          case "Error":
            console.error("📡 Paper Trading WebSocket error:", message.data);
            if (message.data) {
              setState((prev) => ({
                ...prev,
                error: message.data.message || "WebSocket error",
                lastUpdated: new Date(),
              }));
            }
            break;

          default:
            // Log unknown message types for debugging
            console.log("📡 Unknown WebSocket message type:", message.type);
            break;
        }
      } catch (error) {
        console.error("Failed to parse WebSocket message:", error);
      }
    };

    ws.onclose = () => {
      console.log("📡 Paper Trading WebSocket disconnected");
    };

    ws.onerror = (error) => {
      console.error("📡 Paper Trading WebSocket error:", error);
    };

    return () => {
      ws.close();
    };
  }, [
    fetchOpenTrades,
    fetchClosedTrades,
    fetchPortfolioStatus,
    fetchCurrentSettings,
  ]); // Dependencies for WebSocket callbacks

  // WebSocket connection management
  const wsRef = useRef<WebSocket | null>(null);
  const reconnectTimeoutRef = useRef<NodeJS.Timeout | null>(null);
  const heartbeatIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const reconnectAttempts = useRef(0);
  const maxReconnectAttempts = 10;
  const reconnectDelay = useRef(1000); // Start with 1 second

  // Enhanced WebSocket connection with retry logic
  const connectWebSocket = useCallback(() => {
    if (wsRef.current?.readyState === WebSocket.OPEN) {
      return; // Already connected
    }

    try {
      const ws = new WebSocket("ws://localhost:8080/ws");
      wsRef.current = ws;

      ws.onopen = () => {
        console.log("✅ WebSocket connected");
        setState((prev) => ({
          ...prev,
          isActive: true,
          lastUpdated: new Date(),
        }));
        setState((prev) => ({ ...prev, error: null }));
        reconnectAttempts.current = 0;
        reconnectDelay.current = 1000; // Reset delay

        // Start heartbeat
        startHeartbeat();

        // Show connection success toast
        toast({
          title: "🔗 Real-time connection established",
          description: "WebSocket connected successfully",
        });
      };

      ws.onmessage = (event) => {
        try {
          const data = JSON.parse(event.data);
          console.log("📡 WebSocket message:", data);

          // Handle different event types from Rust backend
          switch (data.event_type) {
            case "price_update":
              console.log("💰 Price update:", data.data);
              setState((prev) => ({
                ...prev,
                portfolio: { ...prev.portfolio, ...data.data },
                lastUpdated: new Date(),
              }));
              break;

            case "trade_executed":
              console.log("🎯 Trade executed:", data.data);
              fetchPortfolioStatus();
              fetchOpenTrades();
              setState((prev) => ({
                ...prev,
                portfolio: { ...prev.portfolio, ...data.data },
                lastUpdated: new Date(),
              }));
              break;

            case "trade_closed":
              console.log("🔒 Trade closed:", data.data);
              fetchPortfolioStatus();
              fetchOpenTrades();
              fetchClosedTrades();
              setState((prev) => ({
                ...prev,
                portfolio: { ...prev.portfolio, ...data.data },
                lastUpdated: new Date(),
              }));
              break;

            case "performance_update":
              console.log("📊 Performance update:", data.data);
              fetchPortfolioStatus();
              setState((prev) => ({
                ...prev,
                portfolio: { ...prev.portfolio, ...data.data },
                lastUpdated: new Date(),
              }));
              break;

            case "AISignalReceived":
              console.log("🤖 AI Signal received:", data.data);
              setState((prev) => ({
                ...prev,
                recentSignals: [data.data, ...prev.recentSignals.slice(0, 19)],
                lastUpdated: new Date(),
              }));

              // Show AI signal notification
              toast({
                title: "🤖 AI Signal Received",
                description: `${data.data.signal?.toUpperCase()} ${
                  data.data.symbol
                } (${Math.round(data.data.confidence * 100)}%)`,
              });
              break;

            case "pong":
              // Heartbeat response
              console.log("💓 Heartbeat response received");
              break;

            default:
              console.log("📨 Unknown event type:", data.event_type);
          }

          // Update counter (removed spam sync notification)
          setState((prev) => {
            const newUpdateCounter = prev.updateCounter + 1;
            return { ...prev, updateCounter: newUpdateCounter };
          });
        } catch (error) {
          console.error("❌ Error parsing WebSocket message:", error);
        }
      };

      ws.onclose = (event) => {
        console.log("❌ WebSocket disconnected:", event.code, event.reason);
        setState((prev) => ({
          ...prev,
          isActive: false,
          lastUpdated: new Date(),
        }));
        stopHeartbeat();

        // Only attempt reconnection if it wasn't a normal closure
        if (event.code !== 1000) {
          scheduleReconnect();
        }
      };

      ws.onerror = (error) => {
        console.error("❌ WebSocket error:", error);
        setState((prev) => ({
          ...prev,
          error: "WebSocket connection failed",
          isActive: false,
          lastUpdated: new Date(),
        }));
        stopHeartbeat();
      };
    } catch (error) {
      console.error("❌ Failed to create WebSocket:", error);
      setState((prev) => ({
        ...prev,
        error: "Failed to establish WebSocket connection",
        isActive: false,
        lastUpdated: new Date(),
      }));
      scheduleReconnect();
    }
  }, [fetchPortfolioStatus, fetchOpenTrades, fetchClosedTrades, setState]);

  // Schedule reconnection with exponential backoff
  const scheduleReconnect = useCallback(() => {
    if (reconnectAttempts.current >= maxReconnectAttempts) {
      console.error("❌ Max reconnection attempts reached");
      toast({
        variant: "destructive",
        title: "🔌 Connection lost",
        description: "Please refresh the page to reconnect.",
      });
      return;
    }

    reconnectAttempts.current++;
    const delay = Math.min(
      reconnectDelay.current * Math.pow(2, reconnectAttempts.current - 1),
      30000
    ); // Max 30 seconds

    console.log(
      `⏰ Scheduling reconnection attempt ${reconnectAttempts.current} in ${delay}ms`
    );

    // Show reconnection toast
    toast({
      title: "🔄 Reconnecting...",
      description: `Attempt ${reconnectAttempts.current}/${maxReconnectAttempts}`,
    });

    reconnectTimeoutRef.current = setTimeout(() => {
      connectWebSocket();
    }, delay);
  }, [connectWebSocket]);

  // Start heartbeat to keep connection alive
  const startHeartbeat = useCallback(() => {
    stopHeartbeat(); // Clear any existing heartbeat

    heartbeatIntervalRef.current = setInterval(() => {
      if (wsRef.current?.readyState === WebSocket.OPEN) {
        wsRef.current.send(JSON.stringify({ type: "ping" }));
        console.log("💓 Heartbeat sent");
      }
    }, 30000); // Send heartbeat every 30 seconds
  }, []);

  // Stop heartbeat
  const stopHeartbeat = useCallback(() => {
    if (heartbeatIntervalRef.current) {
      clearInterval(heartbeatIntervalRef.current);
      heartbeatIntervalRef.current = null;
    }
  }, []);

  // Enhanced cleanup function
  const cleanup = useCallback(() => {
    // Clear reconnection timeout
    if (reconnectTimeoutRef.current) {
      clearTimeout(reconnectTimeoutRef.current);
      reconnectTimeoutRef.current = null;
    }

    // Stop heartbeat
    stopHeartbeat();

    // Close WebSocket connection
    if (wsRef.current) {
      wsRef.current.close(1000, "Component unmounting");
      wsRef.current = null;
    }
  }, [stopHeartbeat]);

  // Enhanced useEffect for WebSocket management
  useEffect(() => {
    connectWebSocket();

    // Handle page visibility changes
    const handleVisibilityChange = () => {
      if (document.visibilityState === "visible") {
        // Page became visible, ensure connection is active
        if (!wsRef.current || wsRef.current.readyState !== WebSocket.OPEN) {
          console.log("📱 Page visible, reconnecting WebSocket...");
          connectWebSocket();
        }
      }
    };

    // Handle online/offline events
    const handleOnline = () => {
      console.log("🌐 Network online, reconnecting WebSocket...");
      connectWebSocket();
    };

    const handleOffline = () => {
      console.log("📡 Network offline, cleaning up WebSocket...");
      cleanup();
    };

    document.addEventListener("visibilitychange", handleVisibilityChange);
    window.addEventListener("online", handleOnline);
    window.addEventListener("offline", handleOffline);

    return () => {
      document.removeEventListener("visibilitychange", handleVisibilityChange);
      window.removeEventListener("online", handleOnline);
      window.removeEventListener("offline", handleOffline);
      cleanup();
    };
  }, [connectWebSocket, cleanup]);

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
