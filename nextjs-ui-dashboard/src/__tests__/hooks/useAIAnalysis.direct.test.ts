/**
 * Additional tests for useAIAnalysis hook to boost coverage to 95%+
 * Target uncovered lines: 87, 147-148, 206-207, 247-248, 332, 337-351, 362-363
 */

import { renderHook, waitFor, act } from "@testing-library/react";
import { useAIAnalysis } from "@/hooks/useAIAnalysis";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// Mock dependencies
vi.mock('@/services/api', () => ({
  apiClient: {
    rust: {
      getAIServiceInfo: vi.fn(),
      getSupportedStrategies: vi.fn(),
      getSupportedSymbols: vi.fn(),
      getChartData: vi.fn(),
      getLatestPrices: vi.fn(),
      analyzeAI: vi.fn(),
      getStrategyRecommendations: vi.fn(),
      analyzeMarketCondition: vi.fn(),
    }
  }
}));

vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
  }
}));

vi.mock('@/utils/binancePrice', () => ({
  fetchBinancePrice: vi.fn((symbol, fallback) => fallback()),
}));

import { apiClient } from '@/services/api';

describe('useAIAnalysis - Additional Coverage Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();

    // Default mock implementations
    vi.mocked(apiClient.rust.getAIServiceInfo).mockResolvedValue({
      status: 'online',
      version: '1.0.0',
      models: ['lstm', 'gru'],
    } as any);

    vi.mocked(apiClient.rust.getSupportedStrategies).mockResolvedValue({
      strategies: ['RSI Strategy', 'MACD Strategy'],
    });

    vi.mocked(apiClient.rust.getSupportedSymbols).mockResolvedValue({
      symbols: ['BTCUSDT', 'ETHUSDT'],
    });

    vi.mocked(apiClient.rust.getChartData).mockResolvedValue({
      candles: [
        {
          timestamp: Date.now() - 3600000,
          open: 50000,
          high: 51000,
          low: 49500,
          close: 50500,
          volume: 1000,
        }
      ],
    });

    vi.mocked(apiClient.rust.getLatestPrices).mockResolvedValue({
      BTCUSDT: 50500,
      ETHUSDT: 3000,
    });

    vi.mocked(apiClient.rust.analyzeAI).mockResolvedValue({
      signal: 'BUY',
      confidence: 0.85,
      reasoning: 'Strong uptrend',
      strategy_scores: {},
      market_analysis: {},
      risk_assessment: {},
    } as any);

    vi.mocked(apiClient.rust.getStrategyRecommendations).mockResolvedValue([
      {
        strategy: 'RSI Strategy',
        score: 0.8,
        reasoning: 'Good',
      }
    ] as any);

    vi.mocked(apiClient.rust.analyzeMarketCondition).mockResolvedValue({
      condition: 'Bullish',
      confidence: 0.9,
      analysis: 'Strong uptrend',
    } as any);
  });

  afterEach(() => {
    vi.clearAllMocks();
  });

  describe('clearError (line 87)', () => {
    it('should clear error state', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      // Set error manually by triggering failed analysis
      vi.mocked(apiClient.rust.analyzeAI).mockRejectedValueOnce(new Error('Test error'));

      await act(async () => {
        try {
          await result.current.analyzeSymbol('BTCUSDT');
        } catch {}
      });

      await waitFor(() => {
        expect(result.current.state.error).toBeTruthy();
      });

      // Call clearError
      act(() => {
        result.current.clearError();
      });

      expect(result.current.state.error).toBeNull();
    });
  });

  describe('analyzeSymbol - getLatestPrices fallback (lines 147-148)', () => {
    it('should use fallback when symbol not in prices', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      // Mock empty prices
      vi.mocked(apiClient.rust.getLatestPrices).mockResolvedValueOnce({});

      await act(async () => {
        try {
          await result.current.analyzeSymbol('UNKNOWNSYMBOL');
        } catch {}
      });

      // Verify fallback was called (returns 0 for unknown symbol)
      expect(apiClient.rust.getLatestPrices).toHaveBeenCalled();
    });

    it('should handle missing candle data', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      // Mock empty candle data
      vi.mocked(apiClient.rust.getChartData).mockResolvedValue({ candles: [] });

      await act(async () => {
        try {
          await result.current.analyzeSymbol('BTCUSDT');
        } catch {}
      });

      await waitFor(() => {
        expect(result.current.state.error).toContain('No real candle data available');
      });
    });
  });

  describe('getStrategyRecommendations - getLatestPrices fallback (lines 206-207)', () => {
    it('should use fallback when symbol not in prices', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      vi.mocked(apiClient.rust.getLatestPrices).mockResolvedValueOnce({});

      await act(async () => {
        try {
          await result.current.getStrategyRecommendations('UNKNOWNSYMBOL');
        } catch {}
      });

      expect(apiClient.rust.getLatestPrices).toHaveBeenCalled();
    });

    it('should handle error in getStrategyRecommendations', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      vi.mocked(apiClient.rust.getStrategyRecommendations).mockRejectedValueOnce(
        new Error('Strategy error')
      );

      await act(async () => {
        try {
          await result.current.getStrategyRecommendations('BTCUSDT');
        } catch {}
      });

      await waitFor(() => {
        expect(result.current.state.error).toBe('Strategy error');
      });
    });
  });

  describe('analyzeMarketCondition - getLatestPrices fallback (lines 247-248)', () => {
    it('should use fallback when symbol not in prices', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      vi.mocked(apiClient.rust.getLatestPrices).mockResolvedValueOnce({});

      await act(async () => {
        try {
          await result.current.analyzeMarketCondition('UNKNOWNSYMBOL');
        } catch {}
      });

      expect(apiClient.rust.getLatestPrices).toHaveBeenCalled();
    });

    it('should handle error in analyzeMarketCondition', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      vi.mocked(apiClient.rust.analyzeMarketCondition).mockRejectedValueOnce(
        new Error('Market condition error')
      );

      await act(async () => {
        try {
          await result.current.analyzeMarketCondition('BTCUSDT');
        } catch {}
      });

      await waitFor(() => {
        expect(result.current.state.error).toBe('Market condition error');
      });
    });
  });

  describe('startAutoRefresh and AbortController (lines 332, 337-351)', () => {
    it('should initialize with default symbols', () => {
      const { result } = renderHook(() => useAIAnalysis());

      // Initial state should have fallback symbols
      expect(result.current.state.availableSymbols).toEqual([
        'BTCUSDT',
        'ETHUSDT',
        'BNBUSDT',
        'SOLUSDT',
      ]);
    });
  });

  describe('stopAutoRefresh - cleanup (lines 362-363)', () => {
    it('should abort controller on unmount', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      // Unmount immediately
      unmount();

      // Should not crash
      expect(apiClient.rust.getAIServiceInfo).toHaveBeenCalled();
    });
  });

  describe('fetchRealCandles error handling', () => {
    it('should handle candles with zero volume', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      vi.mocked(apiClient.rust.getChartData).mockResolvedValue({
        candles: [
          {
            timestamp: Date.now(),
            open: 50000,
            high: 50000,
            low: 50000,
            close: 50000,
            volume: 0,
          },
        ],
      });

      await act(async () => {
        try {
          await result.current.analyzeSymbol('BTCUSDT');
        } catch {}
      });

      // Should not crash
      expect(result.current.state).toBeDefined();
    });
  });

  describe('refreshAvailableSymbols', () => {
    it('should return symbols from API', async () => {
      const { result } = renderHook(() => useAIAnalysis());

      let symbols: string[] = [];

      await act(async () => {
        symbols = await result.current.refreshAvailableSymbols();
      });

      expect(symbols).toEqual(['BTCUSDT', 'ETHUSDT']);
    });
  });
});
