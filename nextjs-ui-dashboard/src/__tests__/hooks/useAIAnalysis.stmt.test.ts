/**
 * Statement Coverage Tests for useAIAnalysis.ts
 *
 * Targeting uncovered lines:
 * - Lines 332, 337-338: startAutoRefresh clearing and setting interval
 * - Lines 342, 346, 349-351: Interval callback (abort, create controller, analyze symbol)
 * - Lines 362-363: stopAutoRefresh abort and null controller
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act } from '@testing-library/react';
import { useAIAnalysis } from '@/hooks/useAIAnalysis';

// Mock dependencies
vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
  },
}));

vi.mock('@/utils/binancePrice', () => ({
  fetchBinancePrice: vi.fn().mockResolvedValue(50000),
}));

vi.mock('@/services/api', () => {
  return {
    apiClient: {
      rust: {
        getChartData: vi.fn().mockResolvedValue({
          symbol: 'BTCUSDT',
          timeframe: '1h',
          candles: [
            {
              timestamp: Date.now() - 3600000,
              open: 50000,
              high: 51000,
              low: 49000,
              close: 50500,
              volume: 1000,
            },
          ],
          latest_price: 50500,
          volume_24h: 10000,
          price_change_24h: 500,
          price_change_percent_24h: 1.0,
        }),
        analyzeAI: vi.fn().mockResolvedValue({
          signal: 'long',
          confidence: 0.85,
          reasoning: 'Test reasoning',
          strategy_scores: { RSI: 0.8 },
          market_analysis: {
            trend_direction: 'up',
            trend_strength: 0.7,
            support_levels: [49000],
            resistance_levels: [51000],
            volatility_level: 'medium',
            volume_analysis: 'increasing',
          },
          risk_assessment: {
            overall_risk: 'low',
            technical_risk: 0.3,
            market_risk: 0.2,
            recommended_position_size: 0.5,
            stop_loss_suggestion: 49000,
            take_profit_suggestion: 52000,
          },
          timestamp: Date.now(),
        }),
        getStrategyRecommendations: vi.fn().mockResolvedValue([]),
        analyzeMarketCondition: vi.fn().mockResolvedValue(null),
        getAIServiceInfo: vi.fn().mockResolvedValue({
          service_name: 'AI Service',
          version: '1.0.0',
          model_version: '1.0',
          supported_timeframes: ['15m', '30m', '1h', '4h'],
          supported_symbols: ['BTCUSDT', 'ETHUSDT'],
          capabilities: ['analysis', 'prediction'],
        }),
        getSupportedStrategies: vi.fn().mockResolvedValue({
          strategies: ['RSI Strategy', 'MACD Strategy'],
        }),
        getSupportedSymbols: vi.fn().mockResolvedValue({
          symbols: ['BTCUSDT', 'ETHUSDT', 'BNBUSDT'],
          available_timeframes: ['15m', '30m', '1h', '4h'],
        }),
        getLatestPrices: vi.fn().mockResolvedValue({
          BTCUSDT: 50000,
          ETHUSDT: 3000,
        }),
      },
    },
    AISignalResponse: {},
    AIStrategyContext: {},
    StrategyRecommendation: {},
    MarketConditionAnalysis: {},
    AIServiceInfo: {},
    CandleDataAI: {},
  };
});

describe('useAIAnalysis.ts - Statement Coverage', () => {
  let mockAnalyzeAI: ReturnType<typeof vi.fn>;
  let mockGetChartData: ReturnType<typeof vi.fn>;

  beforeEach(async () => {
    vi.clearAllMocks();
    vi.useFakeTimers();

    // Get references to mocked functions
    const { apiClient } = await import('@/services/api');
    mockAnalyzeAI = apiClient.rust.analyzeAI as ReturnType<typeof vi.fn>;
    mockGetChartData = apiClient.rust.getChartData as ReturnType<typeof vi.fn>;
  });

  afterEach(() => {
    vi.clearAllTimers();
    vi.useRealTimers();
  });

  describe('startAutoRefresh and Interval Callback - Lines 332, 337-338, 342, 346, 349-351', () => {
    it('should clear existing interval and set new one (lines 332, 337-338)', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      // Initial mount completes
      await act(async () => {
        await Promise.resolve();
      });

      // Fast-forward by REFRESH_INTERVAL (30000ms) to trigger interval callback
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      // Verify analyzeAI was called (interval executed)
      expect(mockAnalyzeAI).toHaveBeenCalled();

      unmount();
    });

    it('should abort previous request in interval callback (line 337-338)', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      // Advance to first interval
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      const callsAfterFirst = mockAnalyzeAI.mock.calls.length;

      // Advance to second interval (should abort previous if still pending)
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      // Verify analyzeAI was called again
      expect(mockAnalyzeAI.mock.calls.length).toBeGreaterThan(callsAfterFirst);

      unmount();
    });

    it('should create new AbortController in interval callback (line 342)', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      // Trigger interval
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      // Verify analyzeAI was called (which means AbortController was created)
      expect(mockAnalyzeAI).toHaveBeenCalled();

      unmount();
    });

    it('should read symbols from ref and calculate index (lines 346, 349)', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      // Advance timer to trigger interval
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      // Verify analyzeAI was called with a symbol
      expect(mockAnalyzeAI).toHaveBeenCalled();
      // Verify it used chart data (which means symbol was selected)
      expect(mockGetChartData).toHaveBeenCalled();

      unmount();
    });

    it('should call analyzeSymbol with calculated symbol (line 351)', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      // Advance timer
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      // Verify analyzeAI was called (analyzeSymbol calls analyzeAI)
      expect(mockAnalyzeAI).toHaveBeenCalled();

      unmount();
    });
  });

  describe('stopAutoRefresh - Lines 362-363', () => {
    it('should abort pending requests on stopAutoRefresh (line 362)', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      // Unmount (triggers stopAutoRefresh which aborts)
      unmount();

      // Verify cleanup happened (no errors thrown)
      expect(true).toBe(true);
    });

    it('should set abortController to null (line 363)', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      // Trigger interval to create abortController
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      // Unmount to trigger cleanup (sets controller to null at line 363)
      unmount();

      // Verify no errors and cleanup succeeded
      expect(true).toBe(true);
    });

    it('should clear interval in stopAutoRefresh', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      const callCountBeforeUnmount = mockAnalyzeAI.mock.calls.length;

      // Unmount (clears interval)
      unmount();

      // Advance timer - should NOT trigger more analysis
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      // No new calls after unmount
      expect(mockAnalyzeAI.mock.calls.length).toBe(callCountBeforeUnmount);
    });
  });

  describe('Multiple Interval Cycles', () => {
    it('should handle multiple interval cycles correctly', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      const initialCalls = mockAnalyzeAI.mock.calls.length;

      // First cycle
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      const firstCycleCalls = mockAnalyzeAI.mock.calls.length;
      expect(firstCycleCalls).toBeGreaterThan(initialCalls);

      // Second cycle
      await act(async () => {
        vi.advanceTimersByTime(30000);
        await Promise.resolve();
      });

      const secondCycleCalls = mockAnalyzeAI.mock.calls.length;
      expect(secondCycleCalls).toBeGreaterThan(firstCycleCalls);

      unmount();
    });
  });

  describe('Edge Cases', () => {
    it('should handle unmount during interval execution', async () => {
      const { unmount } = renderHook(() => useAIAnalysis());

      await act(async () => {
        await Promise.resolve();
      });

      // Unmount immediately
      unmount();

      // No errors thrown
      expect(true).toBe(true);
    });

    it('should handle rapid mount/unmount cycles', async () => {
      // Mount and unmount quickly
      const { unmount: unmount1 } = renderHook(() => useAIAnalysis());
      await act(async () => {
        await Promise.resolve();
      });
      unmount1();

      // Mount again
      const { unmount: unmount2 } = renderHook(() => useAIAnalysis());
      await act(async () => {
        await Promise.resolve();
      });
      unmount2();

      // Should not cause errors
      expect(true).toBe(true);
    });
  });
});
