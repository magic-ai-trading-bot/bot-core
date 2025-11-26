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

  // FIXED: Fetch REAL candle data from Rust API instead of generating random fake data
  // This is CRITICAL for accurate AI analysis - fake data leads to wrong trading decisions!
  // Now includes 15m AND 30m timeframes for comprehensive short-term trend detection
  const fetchRealCandles = useCallback(
    async (symbol: string): Promise<Record<string, CandleDataAI[]>> => {
      try {
        // Fetch real candle data from Rust API for ALL timeframes in parallel
        // Including 15m & 30m for short-term trend detection (fixes issue where short-term downtrend
        // was ignored because AI only looked at 1H/4H which showed bullish)
        const [chartData15m, chartData30m, chartData1h, chartData4h] = await Promise.all([
          apiClient.rust.getChartData(symbol, "15m", 100), // Very short-term trend
          apiClient.rust.getChartData(symbol, "30m", 100), // Short-term trend
          apiClient.rust.getChartData(symbol, "1h", 100),  // Medium-term trend
          apiClient.rust.getChartData(symbol, "4h", 50),   // Long-term trend
        ]);

        // Convert CandleData to CandleDataAI format
        const convertCandles = (candles: { timestamp: number; open: number; high: number; low: number; close: number; volume: number }[], intervalMs: number): CandleDataAI[] => {
          return candles.map((candle) => ({
            open_time: candle.timestamp,
            close_time: candle.timestamp + intervalMs,
            open: candle.open,
            high: candle.high,
            low: candle.low,
            close: candle.close,
            volume: candle.volume,
            quote_volume: candle.volume * ((candle.open + candle.close) / 2),
            trades: Math.floor(candle.volume / 10), // Estimate trades from volume
            is_closed: true,
          }));
        };

        return {
          "15m": convertCandles(chartData15m.candles || [], 15 * 60 * 1000),
          "30m": convertCandles(chartData30m.candles || [], 30 * 60 * 1000),
          "1h": convertCandles(chartData1h.candles || [], 60 * 60 * 1000),
          "4h": convertCandles(chartData4h.candles || [], 4 * 60 * 60 * 1000),
        };
      } catch (error) {
        logger.error(`Failed to fetch real candles for ${symbol}:`, error);
        // Return empty arrays on error - DO NOT generate fake data!
        return { "15m": [], "30m": [], "1h": [], "4h": [] };
      }
    },
    []
  );

  const analyzeSymbol = useCallback(
    async (symbol: string, strategies: string[] = DEFAULT_STRATEGIES) => {
      try {
        setLoading(true);
        setError(null);

        // FIXED: Fetch REAL candle data from Rust API (not fake random data!)
        const [timeframeData, currentPrice] = await Promise.all([
          fetchRealCandles(symbol),
          fetchBinancePrice(symbol, async () => {
            const prices = await apiClient.rust.getLatestPrices();
            return prices[symbol] || 0;
          }),
        ]);

        // Validate we have real data before proceeding
        if (!timeframeData["1h"]?.length || !timeframeData["4h"]?.length) {
          throw new Error(`No real candle data available for ${symbol}`);
        }

        const request = {
          symbol,
          timeframe_data: timeframeData,
          current_price: currentPrice,
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
    [fetchRealCandles, setLoading, setError]
  );

  const getStrategyRecommendations = useCallback(
    async (symbol: string) => {
      try {
        // FIXED: Fetch REAL candle data from Rust API (not fake random data!)
        const [timeframeData, currentPrice] = await Promise.all([
          fetchRealCandles(symbol),
          fetchBinancePrice(symbol, async () => {
            const prices = await apiClient.rust.getLatestPrices();
            return prices[symbol] || 0;
          }),
        ]);

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
    [fetchRealCandles, setError]
  );

  const analyzeMarketCondition = useCallback(
    async (symbol: string) => {
      try {
        // FIXED: Fetch REAL candle data from Rust API (not fake random data!)
        const [timeframeData, currentPrice] = await Promise.all([
          fetchRealCandles(symbol),
          fetchBinancePrice(symbol, async () => {
            const prices = await apiClient.rust.getLatestPrices();
            return prices[symbol] || 0;
          }),
        ]);

        const data = {
          symbol,
          timeframe_data: timeframeData,
          current_price: currentPrice,
          volume_24h: timeframeData["1h"]?.reduce(
            (sum, candle) => sum + candle.volume,
            0
          ) || 0,
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
    [fetchRealCandles, setError]
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
