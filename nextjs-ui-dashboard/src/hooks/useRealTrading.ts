/**
 * Real Trading Hook
 *
 * Mirror of usePaperTrading API structure for real trading.
 * Connects to real trading endpoints with safety checks.
 *
 * @spec:FR-TRADING-016 (Real Trading Mode)
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-TRADING.md
 */

import { useState, useEffect, useCallback, useRef } from "react";
import logger from "@/utils/logger";
import { useToast } from "@/hooks/use-toast";
import { useTradingMode } from "@/hooks/useTradingMode";

// Re-use types from usePaperTrading (same structure)
import {
  PaperTradingSettings as RealTradingSettings,
  PaperTrade as RealTrade,
  PortfolioMetrics as RealPortfolioMetrics,
  AISignal,
} from "@/hooks/usePaperTrading";

interface RustRealTradingResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

interface SimpleApiResponse {
  message: string;
}

export interface RealTradingState {
  isActive: boolean;
  portfolio: RealPortfolioMetrics;
  openTrades: RealTrade[];
  closedTrades: RealTrade[];
  activeOrders: RealOrder[];
  settings: RealTradingSettings;
  recentSignals: AISignal[];
  isLoading: boolean;
  error: string | null;
  lastUpdated: Date | null;
  updateCounter: number;
  pendingConfirmation: PendingOrderConfirmation | null;
}

export interface RealOrder {
  id: string;
  exchange_order_id: number;
  symbol: string;
  side: string;
  order_type: string;
  quantity: number;
  executed_quantity: number;
  price: number | null;
  avg_fill_price: number;
  status: string;
  is_entry: boolean;
  created_at: string;
  updated_at: string;
}

export interface PlaceOrderRequest {
  symbol: string;
  side: string;
  order_type: string;
  quantity: number;
  price?: number;
  stop_loss?: number;
  take_profit?: number;
  confirmation_token?: string;
}

export interface PendingOrderConfirmation {
  token: string;
  expires_at: string;
  summary: string;
  order_details: PlaceOrderRequest;
}

const defaultSettings: RealTradingSettings = {
  basic: {
    initial_balance: 0, // Real mode has no initial balance
    max_positions: 5,
    default_position_size_pct: 2.0, // More conservative for real money
    default_leverage: 5, // Lower leverage for real trading
    trading_fee_rate: 0.0004,
    funding_fee_rate: 0.0001,
    slippage_pct: 0.02, // Higher slippage for real market
    enabled: false, // Disabled by default
    auto_restart: false,
  },
  risk: {
    max_risk_per_trade_pct: 1.0, // More conservative
    max_portfolio_risk_pct: 10.0, // More conservative
    default_stop_loss_pct: 1.5, // Tighter stop loss
    default_take_profit_pct: 3.0, // More conservative take profit
    max_leverage: 10, // Lower max leverage
    min_margin_level: 300.0, // Higher margin requirement
    max_drawdown_pct: 10.0, // Lower max drawdown
    daily_loss_limit_pct: 3.0, // Stricter daily loss limit
    max_consecutive_losses: 3, // Fewer consecutive losses allowed
    cool_down_minutes: 120, // Longer cool-down period
  },
};

const defaultPortfolio: RealPortfolioMetrics = {
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
  current_balance: 0, // Fetched from exchange
  equity: 0,
  margin_used: 0,
  free_margin: 0,
};

export const useRealTrading = () => {
  const { toast } = useToast();
  const { mode } = useTradingMode();
  const [state, setState] = useState<RealTradingState>({
    isActive: false,
    portfolio: defaultPortfolio,
    openTrades: [],
    closedTrades: [],
    activeOrders: [],
    settings: defaultSettings,
    recentSignals: [],
    isLoading: false,
    error: null,
    lastUpdated: null,
    updateCounter: 0,
    pendingConfirmation: null,
  });

  // API base URL
  const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

  // Guard to prevent duplicate AI signals fetch
  const aiSignalsFetchedRef = useRef(false);

  // Refs for WebSocket message handlers
  const fetchPortfolioStatusRef = useRef<() => Promise<void>>(() => Promise.resolve());
  const fetchOpenTradesRef = useRef<() => Promise<void>>(() => Promise.resolve());
  const fetchClosedTradesRef = useRef<() => Promise<void>>(() => Promise.resolve());
  const fetchOrdersRef = useRef<() => Promise<void>>(() => Promise.resolve());
  const deduplicateSignalsRef = useRef<(signals: AISignal[]) => AISignal[]>((s) => s);

  // Safety check: Only fetch data when in real mode
  const isRealMode = mode === 'real';

  // Fetch bot status
  const fetchBotStatus = useCallback(async () => {
    if (!isRealMode) return;

    try {
      // TODO: Update endpoint when real trading API is ready
      const response = await fetch(`${API_BASE}/api/real-trading/status`);
      const data: RustRealTradingResponse<{
        is_running: boolean;
        portfolio: RealPortfolioMetrics;
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
      logger.error("Failed to fetch real trading status:", error);
      // Don't show error toast for status checks (too noisy)
    }
  }, [API_BASE, isRealMode]);

  // Fetch portfolio status
  const fetchPortfolioStatus = useCallback(async () => {
    if (!isRealMode) return;

    try {
      // TODO: Update endpoint when real trading API is ready
      const response = await fetch(`${API_BASE}/api/real-trading/portfolio`);
      const data = await response.json();

      if (data.success && data.data) {
        // Map API response to frontend type
        // API returns: total_balance, available_balance, locked_balance
        // Frontend expects: current_balance, equity, margin_used, free_margin
        const apiData = data.data;
        const mappedPortfolio: RealPortfolioMetrics = {
          ...defaultPortfolio,
          current_balance: apiData.total_balance || 0,
          equity: apiData.total_balance || 0,
          margin_used: apiData.locked_balance || 0,
          free_margin: apiData.available_balance || 0,
          total_pnl: apiData.realized_pnl || 0,
          total_pnl_percentage: apiData.total_balance > 0
            ? ((apiData.realized_pnl || 0) / apiData.total_balance) * 100
            : 0,
        };

        setState((prev) => ({
          ...prev,
          portfolio: mappedPortfolio,
          lastUpdated: new Date(),
        }));
      } else {
        logger.error("Failed to fetch real portfolio status:", data.error);
      }
    } catch (error) {
      logger.error("Failed to fetch real portfolio status:", error);
    }
  }, [API_BASE, isRealMode]);

  // Fetch open trades
  const fetchOpenTrades = useCallback(async () => {
    if (!isRealMode) return;

    try {
      // TODO: Update endpoint when real trading API is ready
      const response = await fetch(`${API_BASE}/api/real-trading/trades/open`);
      const data: RustRealTradingResponse<RealTrade[]> = await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          openTrades: data.data!,
        }));
      }
    } catch (error) {
      logger.error("Failed to fetch real open trades:", error);
    }
  }, [API_BASE, isRealMode]);

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
        if (i === retries - 1) throw error;
        await new Promise(resolve => setTimeout(resolve, 1000 * (i + 1)));
      }
    }
  }, []);

  // Fetch closed trades
  const fetchClosedTrades = useCallback(async () => {
    if (!isRealMode) return;

    try {
      // TODO: Update endpoint when real trading API is ready
      const data: RustRealTradingResponse<RealTrade[]> = await fetchWithRetry(
        `${API_BASE}/api/real-trading/trades/closed`
      );

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          closedTrades: data.data!,
        }));
      } else {
        logger.warn("Empty or failed response for real closed trades:", data.error);
        toast({
          title: "Warning",
          description: `Failed to fetch real trades: ${data.error || "Unknown error"}`,
          variant: "destructive",
        });
      }
    } catch (error) {
      logger.error("Failed to fetch real closed trades after retries:", error);
      toast({
        title: "Error",
        description: "Unable to connect to real trading service. Please try again.",
        variant: "destructive",
      });
    }
     
  }, [API_BASE, fetchWithRetry, isRealMode]);

  // Fetch current settings
  const fetchCurrentSettings = useCallback(async () => {
    if (!isRealMode) return;

    try {
      // TODO: Update endpoint when real trading API is ready
      const response = await fetch(`${API_BASE}/api/real-trading/settings`, {
        method: "GET",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const data: RustRealTradingResponse<RealTradingSettings> = await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          settings: data.data!,
          lastUpdated: new Date(),
        }));
      }
    } catch (error) {
      logger.error("Failed to fetch real trading settings:", error);
    }
  }, [API_BASE, isRealMode]);

  // Deduplicate signals
  const deduplicateSignals = useCallback((signals: AISignal[]): AISignal[] => {
    const signalMap = new Map<string, AISignal>();

    const sortedSignals = [...signals].sort(
      (a, b) => new Date(b.timestamp).getTime() - new Date(a.timestamp).getTime()
    );

    for (const signal of sortedSignals) {
      if (!signalMap.has(signal.symbol)) {
        const signalAge = Date.now() - new Date(signal.timestamp).getTime();
        if (signalAge < 30 * 60 * 1000) {
          signalMap.set(signal.symbol, signal);
        }
      }
    }

    return Array.from(signalMap.values()).slice(0, 8);
  }, []);

  // Update refs (will be set after fetchOrders is defined)
  fetchPortfolioStatusRef.current = fetchPortfolioStatus;
  fetchOpenTradesRef.current = fetchOpenTrades;
  fetchClosedTradesRef.current = fetchClosedTrades;
  deduplicateSignalsRef.current = deduplicateSignals;
  // Note: fetchOrdersRef.current is set after fetchOrders is defined below

  // Fetch AI signals (re-use paper trading signals for now)
  const fetchAISignals = useCallback(async () => {
    // Real trading uses same AI signals as paper trading
    // Signals are market analysis, not mode-specific
    try {
      let symbols: string[] = [];
      try {
        const symbolsResponse = await fetch(`${API_BASE}/api/market/symbols`);
        const symbolsData = await symbolsResponse.json();
        if (symbolsData.success && symbolsData.data && symbolsData.data.symbols) {
          symbols = symbolsData.data.symbols;
          logger.info(`Fetched ${symbols.length} symbols from API:`, symbols);
        }
      } catch (e) {
        logger.warn("Failed to fetch symbols from API, using fallback:", e);
      }

      if (symbols.length === 0) {
        logger.warn("No symbols from API, falling back to default 4 symbols");
        symbols = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];
      }

      const signalsPromises = symbols.map(async (symbol) => {
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
            risk_level: "Conservative", // More conservative for real mode
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
        const allSignals = [...validSignals, ...prev.recentSignals];
        const deduplicatedSignals = deduplicateSignals(allSignals);

        return {
          ...prev,
          recentSignals: deduplicatedSignals,
        };
      });
    } catch (error) {
      logger.error("Failed to fetch AI signals for real trading:", error);
    }
  }, [API_BASE, deduplicateSignals]);

  // Start real trading (with safety check)
  const startTrading = useCallback(async () => {
    if (!isRealMode) {
      toast({
        title: "Error",
        description: "Cannot start real trading - switch to real mode first",
        variant: "destructive",
      });
      return;
    }

    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      // TODO: Update endpoint when real trading API is ready
      const response = await fetch(`${API_BASE}/api/real-trading/start`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const data: RustRealTradingResponse<SimpleApiResponse> = await response.json();

      if (data.success) {
        setState((prev) => ({
          ...prev,
          isActive: true,
          isLoading: false,
        }));
        await fetchPortfolioStatus();
        await fetchOpenTrades();

        toast({
          title: "⚠️ Real Trading Started",
          description: "All trades will now execute with real funds",
          variant: "destructive",
        });
      } else {
        throw new Error(data.error || "Failed to start real trading");
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : "Unknown error",
      }));
      toast({
        title: "Error",
        description: error instanceof Error ? error.message : "Failed to start real trading",
        variant: "destructive",
      });
    }
  }, [API_BASE, fetchPortfolioStatus, fetchOpenTrades, isRealMode, toast]);

  // Stop real trading
  const stopTrading = useCallback(async () => {
    try {
      setState((prev) => ({ ...prev, isLoading: true, error: null }));

      // TODO: Update endpoint when real trading API is ready
      const response = await fetch(`${API_BASE}/api/real-trading/stop`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
      });

      const data: RustRealTradingResponse<SimpleApiResponse> = await response.json();

      if (data.success) {
        setState((prev) => ({
          ...prev,
          isActive: false,
          isLoading: false,
        }));

        toast({
          title: "Real Trading Stopped",
          description: "No new trades will be executed",
        });
      } else {
        throw new Error(data.error || "Failed to stop real trading");
      }
    } catch (error) {
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: error instanceof Error ? error.message : "Unknown error",
      }));
    }
  }, [API_BASE, toast]);

  // Close trade
  const closeTrade = useCallback(
    async (tradeId: string) => {
      if (!isRealMode) {
        toast({
          title: "Error",
          description: "Cannot close real trade - not in real mode",
          variant: "destructive",
        });
        return;
      }

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        // TODO: Update endpoint when real trading API is ready
        const response = await fetch(`${API_BASE}/api/real-trading/trades/${tradeId}/close`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            trade_id: tradeId,
            reason: "manual",
          }),
        });

        const data: RustRealTradingResponse<SimpleApiResponse> = await response.json();

        if (data.success) {
          await fetchOpenTrades();
          await fetchClosedTrades();
          await fetchPortfolioStatus();
          setState((prev) => ({ ...prev, isLoading: false }));

          toast({
            title: "Trade Closed",
            description: "Real trade has been closed",
          });
        } else {
          throw new Error(data.error || "Failed to close real trade");
        }
      } catch (error) {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: error instanceof Error ? error.message : "Unknown error",
        }));
        toast({
          title: "Error",
          description: error instanceof Error ? error.message : "Failed to close trade",
          variant: "destructive",
        });
      }
    },
    [API_BASE, fetchOpenTrades, fetchClosedTrades, fetchPortfolioStatus, isRealMode, toast]
  );

  // Update settings
  const updateSettings = useCallback(
    async (newSettings: RealTradingSettings) => {
      if (!isRealMode) {
        toast({
          title: "Error",
          description: "Cannot update real trading settings - not in real mode",
          variant: "destructive",
        });
        return;
      }

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        // TODO: Update endpoint when real trading API is ready
        const response = await fetch(`${API_BASE}/api/real-trading/settings`, {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(newSettings),
        });

        const data: RustRealTradingResponse<SimpleApiResponse> = await response.json();

        if (data.success) {
          setState((prev) => ({
            ...prev,
            settings: newSettings,
            isLoading: false,
          }));

          await fetchPortfolioStatus();

          toast({
            title: "Settings Updated",
            description: "Real trading settings have been updated",
          });
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
    [API_BASE, fetchPortfolioStatus, isRealMode, toast]
  );

  // Reset portfolio (disabled for real trading)
  const resetPortfolio = useCallback(async () => {
    toast({
      title: "Error",
      description: "Cannot reset real trading portfolio - this feature is only available in paper mode",
      variant: "destructive",
    });
  }, [toast]);

  // ========================================
  // Phase 5: Order Management Methods
  // ========================================

  // Fetch active orders
  const fetchOrders = useCallback(async () => {
    if (!isRealMode) return;

    try {
      const response = await fetch(`${API_BASE}/api/real-trading/orders`);
      const data: RustRealTradingResponse<RealOrder[]> = await response.json();

      if (data.success && data.data) {
        setState((prev) => ({
          ...prev,
          activeOrders: data.data!,
          lastUpdated: new Date(),
        }));
      }
    } catch (error) {
      logger.error("Failed to fetch orders:", error);
    }
  }, [API_BASE, isRealMode]);

  // Update fetchOrdersRef after fetchOrders is defined
  fetchOrdersRef.current = fetchOrders;

  // Place order with 2-step confirmation
  // Step 1: Call without token → returns confirmation token + summary
  // Step 2: Call with token → executes order
  const placeOrder = useCallback(
    async (request: PlaceOrderRequest): Promise<boolean> => {
      if (!isRealMode) {
        toast({
          title: "Error",
          description: "Cannot place order - not in real mode",
          variant: "destructive",
        });
        return false;
      }

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        const response = await fetch(`${API_BASE}/api/real-trading/orders`, {
          method: "POST",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(request),
        });

        const data = await response.json();

        if (!data.success) {
          throw new Error(data.error || "Failed to place order");
        }

        // Check if this is a confirmation request (no token provided)
        if (data.data?.requires_confirmation) {
          // Store pending confirmation
          setState((prev) => ({
            ...prev,
            isLoading: false,
            pendingConfirmation: {
              token: data.data.token,
              expires_at: data.data.expires_at,
              summary: data.data.summary,
              order_details: request,
            },
          }));

          toast({
            title: "⚠️ Confirmation Required",
            description: data.data.summary,
          });

          return false; // Needs confirmation
        }

        // Order was executed (had confirmation token)
        setState((prev) => ({
          ...prev,
          isLoading: false,
          pendingConfirmation: null,
        }));

        await fetchOrders();
        await fetchOpenTrades();
        await fetchPortfolioStatus();

        toast({
          title: "✅ Order Placed",
          description: `${request.side} ${request.quantity} ${request.symbol}`,
        });

        return true;
      } catch (error) {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: error instanceof Error ? error.message : "Unknown error",
        }));
        toast({
          title: "Error",
          description: error instanceof Error ? error.message : "Failed to place order",
          variant: "destructive",
        });
        return false;
      }
    },
    [API_BASE, fetchOrders, fetchOpenTrades, fetchPortfolioStatus, isRealMode, toast]
  );

  // Confirm pending order
  const confirmOrder = useCallback(async (): Promise<boolean> => {
    if (!state.pendingConfirmation) {
      toast({
        title: "Error",
        description: "No pending order to confirm",
        variant: "destructive",
      });
      return false;
    }

    // Check if token expired
    if (new Date(state.pendingConfirmation.expires_at) < new Date()) {
      setState((prev) => ({ ...prev, pendingConfirmation: null }));
      toast({
        title: "Error",
        description: "Confirmation token expired. Please place order again.",
        variant: "destructive",
      });
      return false;
    }

    // Place order with confirmation token
    const orderWithToken: PlaceOrderRequest = {
      ...state.pendingConfirmation.order_details,
      confirmation_token: state.pendingConfirmation.token,
    };

    return placeOrder(orderWithToken);
  }, [placeOrder, state.pendingConfirmation, toast]);

  // Clear pending confirmation
  const clearPendingConfirmation = useCallback(() => {
    setState((prev) => ({ ...prev, pendingConfirmation: null }));
  }, []);

  // Cancel specific order
  const cancelOrder = useCallback(
    async (orderId: string): Promise<boolean> => {
      if (!isRealMode) {
        toast({
          title: "Error",
          description: "Cannot cancel order - not in real mode",
          variant: "destructive",
        });
        return false;
      }

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        const response = await fetch(`${API_BASE}/api/real-trading/orders/${orderId}`, {
          method: "DELETE",
        });

        const data: RustRealTradingResponse<SimpleApiResponse> = await response.json();

        if (data.success) {
          setState((prev) => ({ ...prev, isLoading: false }));
          await fetchOrders();

          toast({
            title: "Order Cancelled",
            description: `Order ${orderId} has been cancelled`,
          });

          return true;
        } else {
          throw new Error(data.error || "Failed to cancel order");
        }
      } catch (error) {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: error instanceof Error ? error.message : "Unknown error",
        }));
        toast({
          title: "Error",
          description: error instanceof Error ? error.message : "Failed to cancel order",
          variant: "destructive",
        });
        return false;
      }
    },
    [API_BASE, fetchOrders, isRealMode, toast]
  );

  // Cancel all orders
  const cancelAllOrders = useCallback(
    async (symbol?: string): Promise<boolean> => {
      if (!isRealMode) {
        toast({
          title: "Error",
          description: "Cannot cancel orders - not in real mode",
          variant: "destructive",
        });
        return false;
      }

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        const url = symbol
          ? `${API_BASE}/api/real-trading/orders/all?symbol=${symbol}`
          : `${API_BASE}/api/real-trading/orders/all`;

        const response = await fetch(url, {
          method: "DELETE",
        });

        const data: RustRealTradingResponse<SimpleApiResponse> = await response.json();

        if (data.success) {
          setState((prev) => ({
            ...prev,
            isLoading: false,
            activeOrders: [],
          }));

          toast({
            title: "All Orders Cancelled",
            description: symbol ? `All ${symbol} orders cancelled` : "All orders cancelled",
          });

          return true;
        } else {
          throw new Error(data.error || "Failed to cancel orders");
        }
      } catch (error) {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: error instanceof Error ? error.message : "Unknown error",
        }));
        toast({
          title: "Error",
          description: error instanceof Error ? error.message : "Failed to cancel orders",
          variant: "destructive",
        });
        return false;
      }
    },
    [API_BASE, isRealMode, toast]
  );

  // Modify position SL/TP
  const modifySlTp = useCallback(
    async (symbol: string, stopLoss?: number, takeProfit?: number): Promise<boolean> => {
      if (!isRealMode) {
        toast({
          title: "Error",
          description: "Cannot modify position - not in real mode",
          variant: "destructive",
        });
        return false;
      }

      try {
        setState((prev) => ({ ...prev, isLoading: true, error: null }));

        const response = await fetch(`${API_BASE}/api/real-trading/positions/${symbol}/sltp`, {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            stop_loss: stopLoss,
            take_profit: takeProfit,
          }),
        });

        const data: RustRealTradingResponse<SimpleApiResponse> = await response.json();

        if (data.success) {
          setState((prev) => ({ ...prev, isLoading: false }));
          await fetchOpenTrades();
          await fetchOrders();

          toast({
            title: "Position Updated",
            description: `SL/TP updated for ${symbol}`,
          });

          return true;
        } else {
          throw new Error(data.error || "Failed to modify SL/TP");
        }
      } catch (error) {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: error instanceof Error ? error.message : "Unknown error",
        }));
        toast({
          title: "Error",
          description: error instanceof Error ? error.message : "Failed to modify SL/TP",
          variant: "destructive",
        });
        return false;
      }
    },
    [API_BASE, fetchOpenTrades, fetchOrders, isRealMode, toast]
  );

  // Initial data fetch (only when in real mode)
  useEffect(() => {
    if (!isRealMode) {
      // Clear state when switching away from real mode
      setState({
        isActive: false,
        portfolio: defaultPortfolio,
        openTrades: [],
        closedTrades: [],
        activeOrders: [],
        settings: defaultSettings,
        recentSignals: [],
        isLoading: false,
        error: null,
        lastUpdated: null,
        updateCounter: 0,
        pendingConfirmation: null,
      });
      return;
    }

    fetchBotStatus();
    fetchPortfolioStatus(); // Fetch real balance from Binance
    fetchOpenTrades();
    fetchClosedTrades();
    fetchOrders();
    fetchCurrentSettings();

    if (!aiSignalsFetchedRef.current) {
      aiSignalsFetchedRef.current = true;
      fetchAISignals();
    }

  }, [isRealMode]);

  // WebSocket connection (same as paper trading but with real endpoints)
  useEffect(() => {
    if (!isRealMode) return;

    const wsUrl = (import.meta.env.VITE_WS_URL || "ws://localhost:8080/ws").replace("http", "ws");
    const ws = new WebSocket(wsUrl);
    let heartbeatInterval: NodeJS.Timeout | null = null;

    ws.onopen = () => {
      logger.info("🔴 Real Trading WebSocket connected");
      heartbeatInterval = setInterval(() => {
        if (ws.readyState === WebSocket.OPEN) {
          ws.send(JSON.stringify({ type: "ping" }));
        }
      }, 30000);
    };

    ws.onmessage = (event) => {
      try {
        const message = JSON.parse(event.data);
        const eventType = message.event_type || message.type;
        const data = message.data;

        switch (eventType) {
          case "MarketData":
            if (data && data.symbol && data.price) {
              setState((prev) => {
                if (!prev.portfolio || !prev.portfolio.current_balance) {
                  return { ...prev, lastUpdated: new Date() };
                }

                let totalUnrealizedPnl = 0;

                const updatedOpenTrades = prev.openTrades.map((trade) => {
                  if (trade.symbol === data.symbol) {
                    const priceDiff = data.price - trade.entry_price;
                    const newUnrealizedPnl =
                      trade.trade_type === "Long"
                        ? priceDiff * trade.quantity
                        : -priceDiff * trade.quantity;

                    totalUnrealizedPnl += newUnrealizedPnl;

                    return {
                      ...trade,
                      pnl: newUnrealizedPnl,
                    };
                  } else {
                    totalUnrealizedPnl += trade.pnl || 0;
                  }
                  return trade;
                });

                const updatedPortfolio = {
                  ...prev.portfolio,
                };

                return {
                  ...prev,
                  openTrades: updatedOpenTrades,
                  portfolio: updatedPortfolio,
                  lastUpdated: new Date(),
                };
              });

              if (Math.random() < 0.05) {
                fetchPortfolioStatusRef.current();
              }
            }
            break;

          case "trade_executed":
            fetchPortfolioStatusRef.current();
            fetchOpenTradesRef.current();
            break;

          case "trade_closed":
            fetchPortfolioStatusRef.current();
            fetchOpenTradesRef.current();
            fetchClosedTradesRef.current();
            break;

          case "AISignalReceived":
            if (data) {
              setState((prev) => {
                const allSignals = [data, ...prev.recentSignals];
                const deduplicatedSignals = deduplicateSignalsRef.current(allSignals);

                return {
                  ...prev,
                  recentSignals: deduplicatedSignals,
                  lastUpdated: new Date(),
                };
              });
            }
            break;

          // Order events (Phase 5)
          case "order_placed":
          case "order_filled":
          case "order_partially_filled":
            fetchOrdersRef.current();
            fetchPortfolioStatusRef.current();
            fetchOpenTradesRef.current();
            break;

          case "order_cancelled":
            fetchOrdersRef.current();
            break;

          default:
            break;
        }
      } catch (error) {
        logger.error("Failed to parse Real Trading WebSocket message:", error);
      }
    };

    ws.onclose = () => {
      logger.info("🔴 Real Trading WebSocket disconnected");
      if (heartbeatInterval) {
        clearInterval(heartbeatInterval);
      }
    };

    ws.onerror = (error) => {
      logger.error("🔴 Real Trading WebSocket error:", error);
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
  }, [isRealMode]);

  return {
    // State
    ...state,

    // Actions
    startTrading,
    stopTrading,
    closeTrade,
    updateSettings,
    resetPortfolio,

    // Order management (Phase 5)
    placeOrder,
    confirmOrder,
    cancelOrder,
    cancelAllOrders,
    modifySlTp,
    clearPendingConfirmation,

    // Manual refresh functions
    refreshData: fetchPortfolioStatus,
    refreshStatus: fetchBotStatus,
    refreshSettings: fetchCurrentSettings,
    refreshAISignals: fetchAISignals,
    refreshOrders: fetchOrders,
    refreshTrades: useCallback(async () => {
      await fetchOpenTrades();
      await fetchClosedTrades();
    }, [fetchOpenTrades, fetchClosedTrades]),
  };
};
