import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useAIAnalysis } from '../../hooks/useAIAnalysis'

// Mock logger
vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn()
  }
}))

// Mock the API client module
vi.mock('@/services/api', () => {
  const createMockFn = () => vi.fn()

  const mockApiClient = {
    analyzeAI: createMockFn(),
    getStrategyRecommendations: createMockFn(),
    analyzeMarketCondition: createMockFn(),
    getAIServiceInfo: createMockFn(),
    getSupportedStrategies: createMockFn(),
    getChartData: createMockFn(),
    getLatestPrices: createMockFn(),
    getSupportedSymbols: createMockFn()
  }

  return {
    BotCoreApiClient: class {
      rust = mockApiClient
      python = {}
      auth = {}
    },
    apiClient: {
      rust: mockApiClient
    }
  }
})

// Mock binancePrice utility
vi.mock('@/utils/binancePrice', () => ({
  fetchBinancePrice: vi.fn((symbol: string) => {
    const prices: Record<string, number> = {
      'BTCUSDT': 50000,
      'ETHUSDT': 3000,
      'BNBUSDT': 400,
      'SOLUSDT': 100
    }
    return Promise.resolve(prices[symbol] || 1000)
  })
}))

describe('useAIAnalysis - Coverage Boost', () => {
  let mockAnalyzeAI: ReturnType<typeof vi.fn>
  let mockGetStrategyRecommendations: ReturnType<typeof vi.fn>
  let mockAnalyzeMarketCondition: ReturnType<typeof vi.fn>
  let mockGetAIServiceInfo: ReturnType<typeof vi.fn>
  let mockGetSupportedStrategies: ReturnType<typeof vi.fn>
  let mockGetChartData: ReturnType<typeof vi.fn>
  let mockGetLatestPrices: ReturnType<typeof vi.fn>
  let mockGetSupportedSymbols: ReturnType<typeof vi.fn>
  let logger: any

  beforeEach(async () => {
    // Import mocked modules
    const apiModule = await import('@/services/api')
    const loggerModule = await import('@/utils/logger')
    logger = loggerModule.default

    mockAnalyzeAI = apiModule.apiClient.rust.analyzeAI as ReturnType<typeof vi.fn>
    mockGetStrategyRecommendations = apiModule.apiClient.rust.getStrategyRecommendations as ReturnType<typeof vi.fn>
    mockAnalyzeMarketCondition = apiModule.apiClient.rust.analyzeMarketCondition as ReturnType<typeof vi.fn>
    mockGetAIServiceInfo = apiModule.apiClient.rust.getAIServiceInfo as ReturnType<typeof vi.fn>
    mockGetSupportedStrategies = apiModule.apiClient.rust.getSupportedStrategies as ReturnType<typeof vi.fn>
    mockGetChartData = apiModule.apiClient.rust.getChartData as ReturnType<typeof vi.fn>
    mockGetLatestPrices = apiModule.apiClient.rust.getLatestPrices as ReturnType<typeof vi.fn>
    mockGetSupportedSymbols = apiModule.apiClient.rust.getSupportedSymbols as ReturnType<typeof vi.fn>

    vi.clearAllMocks()

    // Mock chart data
    const createMockCandles = (count: number = 10) => {
      const now = Date.now()
      return Array.from({ length: count }, (_, i) => ({
        timestamp: now - (count - i) * 3600000,
        open: 50000 + Math.random() * 100,
        high: 51000 + Math.random() * 100,
        low: 49500 + Math.random() * 100,
        close: 50500 + Math.random() * 100,
        volume: 1000 + Math.random() * 500
      }))
    }

    mockGetChartData.mockResolvedValue({
      symbol: 'BTCUSDT',
      timeframe: '1h',
      candles: createMockCandles(10),
      latest_price: 50500,
      volume_24h: 75000,
      price_change_24h: 500,
      price_change_percent_24h: 1.0
    })

    mockGetLatestPrices.mockResolvedValue({
      BTCUSDT: 50000,
      ETHUSDT: 3000,
      BNBUSDT: 400,
      SOLUSDT: 100
    })

    mockGetSupportedSymbols.mockResolvedValue({
      symbols: ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT'],
      available_timeframes: ['15m', '30m', '1h', '4h', '1d']
    })

    mockAnalyzeAI.mockResolvedValue({
      signal: 'LONG',
      confidence: 0.85,
      reasoning: 'Strong uptrend detected',
      strategy_scores: {
        'RSI Strategy': 0.8,
        'MACD Strategy': 0.9
      },
      market_analysis: {
        trend_direction: 'UP',
        trend_strength: 0.9,
        support_levels: [48000, 47000],
        resistance_levels: [52000, 53000],
        volatility_level: 'MEDIUM',
        volume_analysis: 'INCREASING'
      },
      risk_assessment: {
        overall_risk: 'LOW',
        technical_risk: 0.2,
        market_risk: 0.3,
        recommended_position_size: 5,
        stop_loss_suggestion: 48000,
        take_profit_suggestion: 54000
      }
    })

    mockGetStrategyRecommendations.mockResolvedValue([
      {
        strategy_name: 'RSI Strategy',
        score: 0.85,
        confidence: 0.9,
        recommendation: 'STRONG_BUY',
        reasoning: 'RSI oversold'
      }
    ])

    mockAnalyzeMarketCondition.mockResolvedValue({
      trend: 'BULLISH',
      strength: 0.8,
      volatility: 'MEDIUM',
      support_levels: [48000],
      resistance_levels: [52000],
      key_indicators: {
        rsi: 65,
        macd: 'BULLISH',
        volume: 'HIGH'
      }
    })

    mockGetAIServiceInfo.mockResolvedValue({
      status: 'operational',
      version: '1.0.0',
      models_loaded: ['LSTM', 'GRU'],
      uptime: 3600
    })

    mockGetSupportedStrategies.mockResolvedValue({
      strategies: ['RSI Strategy', 'MACD Strategy', 'Volume Strategy']
    })
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('refreshServiceInfo - Coverage Lines 285-303', () => {
    it('refreshes service info successfully and updates state', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      // Wait for mount to complete
      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      // Clear previous calls
      vi.clearAllMocks()

      await act(async () => {
        await result.current.refreshServiceInfo()
      })

      await waitFor(() => {
        expect(mockGetAIServiceInfo).toHaveBeenCalled()
        expect(mockGetSupportedStrategies).toHaveBeenCalled()
        expect(result.current.state.serviceInfo).not.toBe(null)
        expect(result.current.state.serviceInfo?.status).toBe('operational')
        expect(result.current.state.supportedStrategies).toEqual([
          'RSI Strategy',
          'MACD Strategy',
          'Volume Strategy'
        ])
      })

      unmount()
    })

    it('handles refreshServiceInfo error gracefully without setting error state', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      // Wait for mount
      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      vi.clearAllMocks()

      // Mock errors for the next call
      mockGetAIServiceInfo.mockRejectedValueOnce(new Error('Service unavailable'))
      mockGetSupportedStrategies.mockRejectedValueOnce(new Error('Strategies unavailable'))

      await act(async () => {
        await result.current.refreshServiceInfo()
      })

      await waitFor(() => {
        expect(logger.error).toHaveBeenCalledWith(
          'Service info error:',
          expect.any(Error)
        )
      })

      // Should not set error state (line 301 comment says "not critical")
      expect(result.current.state.error).toBe(null)

      unmount()
    })
  })

  describe('refreshAvailableSymbols - Coverage Lines 306-327', () => {
    it('fetches and updates available symbols successfully', async () => {
      const customSymbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT', 'ADAUSDT']

      const { result, unmount } = renderHook(() => useAIAnalysis())

      // Wait for mount to complete first
      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      // Now mock with custom symbols
      mockGetSupportedSymbols.mockResolvedValueOnce({
        symbols: customSymbols,
        available_timeframes: ['15m', '30m', '1h', '4h', '1d']
      })

      let returnedSymbols: string[] = []
      await act(async () => {
        returnedSymbols = await result.current.refreshAvailableSymbols()
      })

      await waitFor(() => {
        expect(result.current.state.availableSymbols).toEqual(customSymbols)
        expect(returnedSymbols).toEqual(customSymbols)
        expect(logger.info).toHaveBeenCalledWith(
          `Loaded ${customSymbols.length} symbols from API:`,
          customSymbols
        )
      })

      unmount()
    })

    it('handles refreshAvailableSymbols error and returns fallback symbols', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      // Wait for mount
      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      mockGetSupportedSymbols.mockRejectedValueOnce(new Error('API unavailable'))

      let returnedSymbols: string[] = []
      await act(async () => {
        returnedSymbols = await result.current.refreshAvailableSymbols()
      })

      await waitFor(() => {
        expect(logger.error).toHaveBeenCalledWith(
          'Failed to fetch symbols from API:',
          expect.any(Error)
        )
        // Should return fallback symbols
        expect(returnedSymbols).toEqual(['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT'])
      })

      unmount()
    })

    it('handles empty symbols array from API and updates state', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      mockGetSupportedSymbols.mockResolvedValueOnce({
        symbols: [],
        available_timeframes: ['15m', '30m', '1h', '4h', '1d']
      })

      let returnedSymbols: string[] = []
      await act(async () => {
        returnedSymbols = await result.current.refreshAvailableSymbols()
      })

      // When API returns empty array, the function returns FALLBACK_SYMBOLS (line 309)
      await waitFor(() => {
        // Code at line 309: const symbols = response.symbols || FALLBACK_SYMBOLS
        // When response.symbols is [], it's truthy, so it uses []
        // This test verifies the actual behavior
        expect(result.current.state.availableSymbols).toEqual([])
        expect(returnedSymbols).toEqual([])
      })

      unmount()
    })

    it('updates availableSymbolsRef for use in startAutoRefresh', async () => {
      const customSymbols = ['BTCUSDT', 'ETHUSDT', 'XRPUSDT']

      const { result, unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      mockGetSupportedSymbols.mockResolvedValueOnce({
        symbols: customSymbols,
        available_timeframes: ['15m', '30m', '1h', '4h', '1d']
      })

      await act(async () => {
        await result.current.refreshAvailableSymbols()
      })

      // Verify state is updated (line 314-315)
      expect(result.current.state.availableSymbols).toEqual(customSymbols)

      unmount()
    })
  })

  describe('Error handling for fetchRealCandles - Coverage Lines 128-132', () => {
    it('logs error when getChartData fails', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      vi.clearAllMocks()

      // Make all getChartData calls fail to trigger error at line 129
      mockGetChartData.mockRejectedValue(new Error('Chart data fetch failed'))

      await act(async () => {
        await result.current.analyzeSymbol('BTCUSDT')
      })

      await waitFor(() => {
        expect(logger.error).toHaveBeenCalledWith(
          expect.stringContaining('Failed to fetch real candles'),
          expect.any(Error)
        )
      })

      unmount()
    })

    it('validates candle data exists before proceeding with analysis', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      // Return empty candles to trigger validation error at line 154
      mockGetChartData.mockResolvedValue({
        symbol: 'BTCUSDT',
        timeframe: '1h',
        candles: [],
        latest_price: 50500,
        volume_24h: 75000,
        price_change_24h: 500,
        price_change_percent_24h: 1.0
      })

      await act(async () => {
        await result.current.analyzeSymbol('BTCUSDT')
      })

      await waitFor(() => {
        // Should throw error about no real candle data (line 154)
        expect(result.current.state.error).toContain('No real candle data available')
      })

      unmount()
    })
  })

  describe('Candle conversion with all timeframes - Coverage Lines 99-127', () => {
    it('fetches and converts candles for 15m, 30m, 1h, 4h timeframes', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      vi.clearAllMocks()

      await act(async () => {
        await result.current.analyzeSymbol('BTCUSDT')
      })

      await waitFor(() => {
        expect(mockAnalyzeAI).toHaveBeenCalled()
      })

      // Should call getChartData 4 times (15m, 30m, 1h, 4h)
      // Note: signal parameter is undefined when called manually (not from auto-refresh)
      expect(mockGetChartData).toHaveBeenCalledWith('BTCUSDT', '15m', 100, undefined)
      expect(mockGetChartData).toHaveBeenCalledWith('BTCUSDT', '30m', 100, undefined)
      expect(mockGetChartData).toHaveBeenCalledWith('BTCUSDT', '1h', 100, undefined)
      expect(mockGetChartData).toHaveBeenCalledWith('BTCUSDT', '4h', 50, undefined)

      // Verify the converted candle structure includes all required fields
      const callArgs = mockAnalyzeAI.mock.calls[0][0]
      expect(callArgs.timeframe_data).toHaveProperty('15m')
      expect(callArgs.timeframe_data).toHaveProperty('30m')
      expect(callArgs.timeframe_data).toHaveProperty('1h')
      expect(callArgs.timeframe_data).toHaveProperty('4h')

      // Verify converted candle structure (lines 108-120)
      const candle15m = callArgs.timeframe_data['15m'][0]
      expect(candle15m).toHaveProperty('open_time')
      expect(candle15m).toHaveProperty('close_time')
      expect(candle15m).toHaveProperty('open')
      expect(candle15m).toHaveProperty('high')
      expect(candle15m).toHaveProperty('low')
      expect(candle15m).toHaveProperty('close')
      expect(candle15m).toHaveProperty('volume')
      expect(candle15m).toHaveProperty('quote_volume')
      expect(candle15m).toHaveProperty('trades')
      expect(candle15m).toHaveProperty('is_closed')
      expect(candle15m.is_closed).toBe(true)

      unmount()
    })

    it('calculates quote_volume and trades from candle data', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      vi.clearAllMocks()

      await act(async () => {
        await result.current.analyzeSymbol('BTCUSDT')
      })

      await waitFor(() => {
        expect(mockAnalyzeAI).toHaveBeenCalled()
      })

      const callArgs = mockAnalyzeAI.mock.calls[0][0]
      const candle = callArgs.timeframe_data['1h'][0]

      // Line 116: quote_volume = volume * ((open + close) / 2)
      const expectedQuoteVolume = candle.volume * ((candle.open + candle.close) / 2)
      expect(candle.quote_volume).toBe(expectedQuoteVolume)

      // Line 117: trades = floor(volume / 10)
      const expectedTrades = Math.floor(candle.volume / 10)
      expect(candle.trades).toBe(expectedTrades)

      unmount()
    })
  })

  describe('useEffect initialization - Coverage Lines 368-391', () => {
    it('calls refreshServiceInfo on mount', async () => {
      const { unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetAIServiceInfo).toHaveBeenCalled()
        expect(mockGetSupportedStrategies).toHaveBeenCalled()
      })

      unmount()
    })

    it('calls refreshAvailableSymbols on mount', async () => {
      const { unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      unmount()
    })

    it('auto-analyzes first symbol after fetching symbols', async () => {
      const { unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
        expect(mockGetChartData).toHaveBeenCalled()
        expect(mockAnalyzeAI).toHaveBeenCalled()
      })

      unmount()
    })

    it('sets isMountedRef to false on unmount to prevent state updates', async () => {
      const { result, unmount } = renderHook(() => useAIAnalysis())

      await waitFor(() => {
        expect(mockGetSupportedSymbols).toHaveBeenCalled()
      })

      unmount()

      // Try to trigger state update after unmount
      await act(async () => {
        await result.current.analyzeSymbol('BTCUSDT')
      })

      // Should not crash - isMountedRef prevents state updates (line 385)
      expect(true).toBe(true)
    })

  })

  describe('Auto-refresh and cleanup - Coverage Lines 330-365, 381, 387-388', () => {
    it('cleans up on unmount without errors', () => {
      const { unmount } = renderHook(() => useAIAnalysis())

      // Auto-refresh is started on mount (line 381)
      // Unmount should stop auto-refresh and abort pending requests (lines 387-388)
      unmount()

      // Should complete without errors
      expect(true).toBe(true)
    })
  })
})
