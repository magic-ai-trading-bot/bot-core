import { useState, useEffect, useCallback, useRef } from "react";
import {
  AISignalResponse,
  AIStrategyContext,
  StrategyRecommendation,
  MarketConditionAnalysis,
  AIServiceInfo,
  CandleDataAI,
} from "@/services/api";
import { BotCoreApiClient } from "@/services/api";

export interface AIAnalysisState {
  signals: AISignalResponse[];
  strategies: StrategyRecommendation[];
  marketCondition: MarketConditionAnalysis | null;
  serviceInfo: AIServiceInfo | null;
  supportedStrategies: string[];
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
  clearError: () => void;
}

const REFRESH_INTERVAL = 300000; // 5 minutes
const DEFAULT_SYMBOLS = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];
const DEFAULT_STRATEGIES = [
  "RSI Strategy",
  "MACD Strategy",
  "Volume Strategy",
  "Bollinger Bands Strategy",
];

export const useAIAnalysis = (): AIAnalysisHook => {
  const [state, setState] = useState<AIAnalysisState>({
    signals: [],
    strategies: [],
    marketCondition: null,
    serviceInfo: null,
    supportedStrategies: [],
    isLoading: false,
    error: null,
    lastUpdate: null,
  });

  const apiClient = useRef(new BotCoreApiClient());
  const refreshIntervalRef = useRef<NodeJS.Timeout | null>(null);

  const setLoading = useCallback((loading: boolean) => {
    setState((prev) => ({ ...prev, isLoading: loading }));
  }, []);

  const setError = useCallback((error: string | null) => {
    setState((prev) => ({ ...prev, error }));
  }, []);

  const clearError = useCallback(() => {
    setError(null);
  }, [setError]);

  // Generate sample candle data for testing (replace with real data from chart)
  const generateSampleCandles = useCallback(
    (symbol: string): Record<string, CandleDataAI[]> => {
      const now = Date.now();
      const basePrice =
        symbol === "BTCUSDT" ? 95000 : symbol === "ETHUSDT" ? 3500 : 600;

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

        const timeframeData = generateSampleCandles(symbol);
        const latestCandle =
          timeframeData["1h"][timeframeData["1h"].length - 1];

        const request = {
          symbol,
          timeframe_data: timeframeData,
          current_price: latestCandle.close,
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

        const signal = await apiClient.current.rust.analyzeAI(request);

        // Add symbol to the response for display purposes
        const enhancedSignal = { ...signal, symbol };

        setState((prev) => ({
          ...prev,
          signals: [enhancedSignal, ...prev.signals.slice(0, 19)], // Keep last 20 signals
          lastUpdate: new Date().toISOString(),
        }));
      } catch (error) {
        console.error("AI Analysis error:", error);
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
        const timeframeData = generateSampleCandles(symbol);
        const latestCandle =
          timeframeData["1h"][timeframeData["1h"].length - 1];

        const data = {
          symbol,
          timeframe_data: timeframeData,
          current_price: latestCandle.close,
          available_strategies: DEFAULT_STRATEGIES,
          timestamp: Date.now(),
        };

        const recommendations =
          await apiClient.current.rust.getStrategyRecommendations(data);

        setState((prev) => ({
          ...prev,
          strategies: recommendations,
        }));
      } catch (error) {
        console.error("Strategy recommendations error:", error);
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
        const timeframeData = generateSampleCandles(symbol);
        const latestCandle =
          timeframeData["1h"][timeframeData["1h"].length - 1];

        const data = {
          symbol,
          timeframe_data: timeframeData,
          current_price: latestCandle.close,
          volume_24h: timeframeData["1h"].reduce(
            (sum, candle) => sum + candle.volume,
            0
          ),
          timestamp: Date.now(),
        };

        const condition = await apiClient.current.rust.analyzeMarketCondition(
          data
        );

        setState((prev) => ({
          ...prev,
          marketCondition: condition,
        }));
      } catch (error) {
        console.error("Market condition analysis error:", error);
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
        apiClient.current.rust.getAIServiceInfo(),
        apiClient.current.rust.getSupportedStrategies(),
      ]);

      setState((prev) => ({
        ...prev,
        serviceInfo,
        supportedStrategies: supportedStrategies.strategies,
      }));
    } catch (error) {
      console.error("Service info error:", error);
      // Don't show error for service info as it's not critical
    }
  }, []);

  // Auto-refresh signals periodically
  const startAutoRefresh = useCallback(() => {
    if (refreshIntervalRef.current) {
      clearInterval(refreshIntervalRef.current);
    }

    refreshIntervalRef.current = setInterval(() => {
      // Analyze default symbols in rotation
      const symbolIndex =
        Math.floor(Date.now() / REFRESH_INTERVAL) % DEFAULT_SYMBOLS.length;
      const symbol = DEFAULT_SYMBOLS[symbolIndex];
      analyzeSymbol(symbol);
    }, REFRESH_INTERVAL);
  }, [analyzeSymbol]);

  const stopAutoRefresh = useCallback(() => {
    if (refreshIntervalRef.current) {
      clearInterval(refreshIntervalRef.current);
      refreshIntervalRef.current = null;
    }
  }, []);

  // Initialize on mount
  useEffect(() => {
    refreshServiceInfo();

    // Analyze initial symbols
    DEFAULT_SYMBOLS.forEach((symbol, index) => {
      setTimeout(() => {
        analyzeSymbol(symbol);
      }, index * 1000); // Stagger initial requests
    });

    startAutoRefresh();

    return () => {
      stopAutoRefresh();
    };
  }, [refreshServiceInfo, analyzeSymbol, startAutoRefresh, stopAutoRefresh]);

  return {
    state,
    analyzeSymbol,
    getStrategyRecommendations,
    analyzeMarketCondition,
    refreshServiceInfo,
    clearError,
  };
};
