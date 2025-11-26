import { useState, useEffect, useCallback, useRef } from "react";
import logger from "@/utils/logger";
import { fetchBinancePrice } from "@/utils/binancePrice";
import {


// @spec:FR-AI-005 (Frontend) - `nextjs-ui-dashboard/src/hooks/useAIAnalysis.ts:45-189`
// @ref:specs/02-design/2.5-components/COMP-PYTHON-ML.md
// @test:

  AISignalResponse,
  AIStrategyContext,
  StrategyRecommendation,
  MarketConditionAnalysis,
  AIServiceInfo,
  CandleDataAI,
} from "@/services/api";
import { apiClient } from "@/services/api";

export interface AIAnalysisState {
  signals: AISignalResponse[];
  strategies: StrategyRecommendation[];
  marketCondition: MarketConditionAnalysis | null;
  serviceInfo: AIServiceInfo | null;
  supportedStrategies: string[];
  availableSymbols: string[]; // Dynamic symbols from API (includes user-added)
  isLoading: boolean;
  error: string | null;
  lastUpdate: string | null;
}

export interface AIAnalysisHook {
  state: AIAnalysisState;
  analyzeSymbol: (symbol: string, strategies?: string[]) => Promise<void>;
  getStrategyRecommendations: (symbol: string) => Promise<void>;
  analyzeMarketCondition: (symbol: string) => Promise<void>;
  refreshServiceInfo: () => Promise<void>;
  refreshAvailableSymbols: () => Promise<string[]>; // Fetch dynamic symbols from API
  clearError: () => void;
}

const REFRESH_INTERVAL = 600000; // 10 minutes (increased to avoid rate limiting)
// FALLBACK symbols - actual symbols are fetched dynamically from /api/market/symbols
const FALLBACK_SYMBOLS = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];
const DEFAULT_STRATEGIES = [
  "RSI Strategy",
  "MACD Strategy",
  "Volume Strategy",
  "Bollinger Bands Strategy",
  "Stochastic Strategy",
];

export const useAIAnalysis = (): AIAnalysisHook => {
  const [state, setState] = useState<AIAnalysisState>({
    signals: [],
    strategies: [],
    marketCondition: null,
    serviceInfo: null,
    supportedStrategies: [],
    availableSymbols: FALLBACK_SYMBOLS, // Will be updated from API
    isLoading: false,
    error: null,
    lastUpdate: null,
  });

  // Use singleton apiClient from api.ts (no need for useRef)
  const refreshIntervalRef = useRef<NodeJS.Timeout | null>(null);
  const isMountedRef = useRef(true);
  // Use ref to track availableSymbols to avoid infinite loop in startAutoRefresh
  const availableSymbolsRef = useRef<string[]>(FALLBACK_SYMBOLS);

  const setLoading = useCallback((loading: boolean) => {
    if (isMountedRef.current) {
      setState((prev) => ({ ...prev, isLoading: loading }));
    }
  }, []);

  const setError = useCallback((error: string | null) => {
    if (isMountedRef.current) {
      setState((prev) => ({ ...prev, error }));
    }
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, [setError]);

  // Generate sample candle data for testing (replace with real data from chart)
  // Now accepts basePrice parameter to use real prices from API instead of hardcoded values
  const generateSampleCandles = useCallback(
    (symbol: string, basePrice: number): Record<string, CandleDataAI[]> => {
      const now = Date.now();
      // Use provided basePrice (from real API) instead of hardcoded values

      const generate1hCandles = (): CandleDataAI[] => {
        const candles: CandleDataAI[] = [];
        for (let i = 5; i >= 0; i--) {
          const open_time = now - i * 60 * 60 * 1000; // 1h intervals
          const close_time = open_time + 60 * 60 * 1000; // 1h later
          const randomChange = (Math.random() - 0.5) * 0.02; // ±1% random change
          const open = basePrice * (1 + randomChange);
          const close = open * (1 + (Math.random() - 0.5) * 0.01); // ±0.5% from open
          const high = Math.max(open, close) * (1 + Math.random() * 0.005); // Up to 0.5% higher
          const low = Math.min(open, close) * (1 - Math.random() * 0.005); // Up to 0.5% lower
          const volume = 1000 + Math.random() * 500;
          const quote_volume = volume * ((open + close) / 2); // Estimated quote volume
          const trades = Math.floor(100 + Math.random() * 500); // Random trade count
          const is_closed = true;

          candles.push({
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume,
            trades,
            is_closed,
          });
        }
        return candles;
      };

      const generate4hCandles = (): CandleDataAI[] => {
        const candles: CandleDataAI[] = [];
        for (let i = 2; i >= 0; i--) {
          const open_time = now - i * 4 * 60 * 60 * 1000; // 4h intervals
          const close_time = open_time + 4 * 60 * 60 * 1000; // 4h later
          const randomChange = (Math.random() - 0.5) * 0.03; // ±1.5% random change
          const open = basePrice * (1 + randomChange);
          const close = open * (1 + (Math.random() - 0.5) * 0.02); // ±1% from open
          const high = Math.max(open, close) * (1 + Math.random() * 0.01); // Up to 1% higher
          const low = Math.min(open, close) * (1 - Math.random() * 0.01); // Up to 1% lower
          const volume = 4000 + Math.random() * 2000;
          const quote_volume = volume * ((open + close) / 2); // Estimated quote volume
          const trades = Math.floor(200 + Math.random() * 1000); // Random trade count
          const is_closed = true;

          candles.push({
            open_time,
            close_time,
            open,
            high,
            low,
            close,
            volume,
            quote_volume,
            trades,
            is_closed,
          });
        }
        return candles;
      };

      return {
        "1h": generate1hCandles(),
        "4h": generate4hCandles(),
      };
    },
    []
  );

  const analyzeSymbol = useCallback(
    async (symbol: string, strategies: string[] = DEFAULT_STRATEGIES) => {
      try {
        setLoading(true);
        setError(null);

        // Fetch price from Binance (with caching) - fallback to our API
        const currentPrice = await fetchBinancePrice(symbol, async () => {
          const prices = await apiClient.rust.getLatestPrices();
          return prices[symbol] || 0;
        });

        const timeframeData = generateSampleCandles(symbol, currentPrice);
        const latestCandle =
          timeframeData["1h"] && timeframeData["1h"].length > 0
            ? timeframeData["1h"][timeframeData["1h"].length - 1]
            : null;

        const request = {
          symbol,
          timeframe_data: timeframeData,
          current_price: currentPrice, // FIXED: Use real price from Binance API!
          volume_24h: timeframeData["1h"].reduce(
            (sum, candle) => sum + candle.volume,
            0
          ),
          timestamp: Date.now(),
          strategy_context: {
            selected_strategies: strategies,
            market_condition: "Bullish",
            risk_level: "Moderate",
            user_preferences: {},
            technical_indicators: {},
          } as AIStrategyContext,
        };

        const signal = await apiClient.rust.analyzeAI(request);

        // Add symbol to the response for display purposes
        const enhancedSignal = { ...signal, symbol };

        if (isMountedRef.current) {
          setState((prev) => ({
            ...prev,
            signals: [enhancedSignal, ...prev.signals.slice(0, 19)], // Keep last 20 signals
            lastUpdate: new Date().toISOString(),
          }));
        }
      } catch (error) {
        logger.error("AI Analysis error:", error);
        setError(
          error instanceof Error ? error.message : "Failed to analyze symbol"
        );
      } finally {
        setLoading(false);
      }
    },
    [generateSampleCandles, setLoading, setError]
  );

  const getStrategyRecommendations = useCallback(
    async (symbol: string) => {
      try {
        // Fetch price from Binance (with caching) - fallback to our API
        const currentPrice = await fetchBinancePrice(symbol, async () => {
          const prices = await apiClient.rust.getLatestPrices();
          return prices[symbol] || 0;
        });

        const timeframeData = generateSampleCandles(symbol, currentPrice);

        const data = {
          symbol,
          timeframe_data: timeframeData,
          current_price: currentPrice,
          available_strategies: DEFAULT_STRATEGIES,
          timestamp: Date.now(),
        };

        const recommendations =
          await apiClient.rust.getStrategyRecommendations(data);

        if (isMountedRef.current) {
          setState((prev) => ({
            ...prev,
            strategies: recommendations,
          }));
        }
      } catch (error) {
        logger.error("Strategy recommendations error:", error);
        setError(
          error instanceof Error
            ? error.message
            : "Failed to get strategy recommendations"
        );
      }
    },
    [generateSampleCandles, setError]
  );

  const analyzeMarketCondition = useCallback(
    async (symbol: string) => {
      try {
        // Fetch price from Binance (with caching) - fallback to our API
        const currentPrice = await fetchBinancePrice(symbol, async () => {
          const prices = await apiClient.rust.getLatestPrices();
          return prices[symbol] || 0;
        });

        const timeframeData = generateSampleCandles(symbol, currentPrice);

        const data = {
          symbol,
          timeframe_data: timeframeData,
          current_price: currentPrice,
          volume_24h: timeframeData["1h"].reduce(
            (sum, candle) => sum + candle.volume,
            0
          ),
          timestamp: Date.now(),
        };

        const condition = await apiClient.rust.analyzeMarketCondition(
          data
        );

        if (isMountedRef.current) {
          setState((prev) => ({
            ...prev,
            marketCondition: condition,
          }));
        }
      } catch (error) {
        logger.error("Market condition analysis error:", error);
        setError(
          error instanceof Error
            ? error.message
            : "Failed to analyze market condition"
        );
      }
    },
    [generateSampleCandles, setError]
  );

  const refreshServiceInfo = useCallback(async () => {
    try {
      const [serviceInfo, supportedStrategies] = await Promise.all([
        apiClient.rust.getAIServiceInfo(),
        apiClient.rust.getSupportedStrategies(),
      ]);

      if (isMountedRef.current) {
        setState((prev) => ({
          ...prev,
          serviceInfo,
          supportedStrategies: supportedStrategies.strategies,
        }));
      }
    } catch (error) {
      logger.error("Service info error:", error);
      // Don't show error for service info as it's not critical
    }
  }, []);

  // Fetch available symbols dynamically from API (includes user-added symbols from database)
  const refreshAvailableSymbols = useCallback(async (): Promise<string[]> => {
    try {
      const response = await apiClient.rust.getSupportedSymbols();
      const symbols = response.symbols || FALLBACK_SYMBOLS;

      if (isMountedRef.current) {
        setState((prev) => ({
          ...prev,
          availableSymbols: symbols,
        }));
        // Update ref for use in startAutoRefresh (avoids stale closure)
        availableSymbolsRef.current = symbols;
      }

      logger.info(`Loaded ${symbols.length} symbols from API:`, symbols);
      return symbols;
    } catch (error) {
      logger.error("Failed to fetch symbols from API:", error);
      // Return fallback symbols on error
      return FALLBACK_SYMBOLS;
    }
  }, []);

  // Auto-refresh signals periodically using dynamic symbols from API
  const startAutoRefresh = useCallback(() => {
    if (refreshIntervalRef.current) {
      clearInterval(refreshIntervalRef.current);
    }

    refreshIntervalRef.current = setInterval(() => {
      // Use ref to access current symbols without causing infinite loop
      // (avoids stale closure by reading ref.current instead of state)
      const symbols = availableSymbolsRef.current.length > 0
        ? availableSymbolsRef.current
        : FALLBACK_SYMBOLS;
      const symbolIndex = Math.floor(Date.now() / REFRESH_INTERVAL) % symbols.length;
      const symbol = symbols[symbolIndex];
      analyzeSymbol(symbol);
    }, REFRESH_INTERVAL);
  }, [analyzeSymbol]); // Removed state.availableSymbols - use ref instead

  const stopAutoRefresh = useCallback(() => {
    if (refreshIntervalRef.current) {
      clearInterval(refreshIntervalRef.current);
      refreshIntervalRef.current = null;
    }
  }, []);

  // Initialize on mount
  useEffect(() => {
    // Fetch service info and available symbols from API (includes user-added symbols)
    refreshServiceInfo();
    refreshAvailableSymbols().then((symbols) => {
      // Auto-analyze first symbol on mount to show initial data
      if (symbols.length > 0) {
        analyzeSymbol(symbols[0]);
      }
    });

    // Start auto-refresh to periodically analyze symbols (every 10 minutes)
    startAutoRefresh();

    return () => {
      // Mark as unmounted to prevent state updates
      isMountedRef.current = false;
      stopAutoRefresh();
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []); // Only run once on mount

  return {
    state,
    analyzeSymbol,
    getStrategyRecommendations,
    analyzeMarketCondition,
    refreshServiceInfo,
    refreshAvailableSymbols,
    clearError,
  };
};
