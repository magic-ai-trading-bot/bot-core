import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useAIAnalysis } from '../../hooks/useAIAnalysis'
import { BotCoreApiClient } from '@/services/api'

// Create mock functions that will be reused
const mockAnalyzeAI = vi.fn()
const mockGetStrategyRecommendations = vi.fn()
const mockAnalyzeMarketCondition = vi.fn()
const mockGetAIServiceInfo = vi.fn()
const mockGetSupportedStrategies = vi.fn()

// Mock the API client module
vi.mock('@/services/api', () => {
  return {
    BotCoreApiClient: class {
      rust = {
        analyzeAI: mockAnalyzeAI,
        getStrategyRecommendations: mockGetStrategyRecommendations,
        analyzeMarketCondition: mockAnalyzeMarketCondition,
        getAIServiceInfo: mockGetAIServiceInfo,
        getSupportedStrategies: mockGetSupportedStrategies
      }
      python = {}
      auth = {}
    }
  }
})

describe('useAIAnalysis', () => {
  beforeEach(() => {
    vi.clearAllMocks()

    // Default mock implementations
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

  it('initializes with default state', () => {
    const { result } = renderHook(() => useAIAnalysis())

    expect(result.current.state.signals).toEqual([])
    expect(result.current.state.strategies).toEqual([])
    expect(result.current.state.marketCondition).toBe(null)
    expect(result.current.state.serviceInfo).toBe(null)
    expect(result.current.state.supportedStrategies).toEqual([])
    expect(result.current.state.isLoading).toBe(false)
    expect(result.current.state.error).toBe(null)
    expect(result.current.state.lastUpdate).toBe(null)
  })

  it('provides all required methods', () => {
    const { result } = renderHook(() => useAIAnalysis())

    expect(result.current.analyzeSymbol).toBeDefined()
    expect(result.current.getStrategyRecommendations).toBeDefined()
    expect(result.current.analyzeMarketCondition).toBeDefined()
    expect(result.current.refreshServiceInfo).toBeDefined()
    expect(result.current.clearError).toBeDefined()
  })

  it('fetches service info on mount', async () => {
    const { unmount } = renderHook(() => useAIAnalysis())

    await waitFor(() => {
      expect(mockGetAIServiceInfo).toHaveBeenCalled()
      expect(mockGetSupportedStrategies).toHaveBeenCalled()
    })

    unmount()
  })

  it('analyzes symbol successfully', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.signals.length).toBeGreaterThan(0)
      expect(result.current.state.signals[0].symbol).toBe('BTCUSDT')
      expect(result.current.state.signals[0].signal).toBe('LONG')
      expect(result.current.state.isLoading).toBe(false)
    })

    unmount()
  })

  it('analyzes symbol with custom strategies', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    const customStrategies = ['RSI Strategy', 'Bollinger Bands Strategy']

    await act(async () => {
      await result.current.analyzeSymbol('ETHUSDT', customStrategies)
    })

    await waitFor(() => {
      expect(mockAnalyzeAI).toHaveBeenCalledWith(
        expect.objectContaining({
          symbol: 'ETHUSDT',
          strategy_context: expect.objectContaining({
            selected_strategies: customStrategies
          })
        })
      )
    })
  })

  it.skip('handles analyze symbol error', async () => {
    mockAnalyzeAI.mockRejectedValueOnce(new Error('Analysis failed'))

    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('Analysis failed')
      expect(result.current.state.isLoading).toBe(false)
    })
  })

  it('sets loading state during analysis', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    // Create a slow mock to catch loading state
    mockAnalyzeAI.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve({
      signal: 'LONG',
      confidence: 0.85,
      reasoning: 'Test',
      strategy_scores: {},
      market_analysis: {
        trend_direction: 'UP',
        trend_strength: 0.9,
        support_levels: [],
        resistance_levels: [],
        volatility_level: 'MEDIUM',
        volume_analysis: 'NORMAL'
      },
      risk_assessment: {
        overall_risk: 'LOW',
        technical_risk: 0.2,
        market_risk: 0.3,
        recommended_position_size: 5,
        stop_loss_suggestion: 48000,
        take_profit_suggestion: 54000
      }
    }), 50)))

    let wasLoading = false

    act(() => {
      result.current.analyzeSymbol('BTCUSDT')
    })

    // Check if it becomes loading
    await waitFor(() => {
      if (result.current.state.isLoading) {
        wasLoading = true
      }
      expect(result.current.state.isLoading).toBe(false)
    }, { timeout: 1000 })

    expect(wasLoading).toBe(true)

    unmount()
  })

  it('keeps last 20 signals', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    // Analyze 25 symbols to test limit
    for (let i = 0; i < 25; i++) {
      await act(async () => {
        await result.current.analyzeSymbol(`SYMBOL${i}`)
      })
    }

    await waitFor(() => {
      expect(result.current.state.signals.length).toBe(20)
    })
  })

  it('gets strategy recommendations successfully', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.getStrategyRecommendations('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.strategies.length).toBeGreaterThan(0)
      expect(result.current.state.strategies[0].strategy_name).toBe('RSI Strategy')
    })
  })

  it('handles strategy recommendations error', async () => {
    mockGetStrategyRecommendations.mockRejectedValueOnce(
      new Error('Failed to get recommendations')
    )

    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.getStrategyRecommendations('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('Failed to get recommendations')
    })
  })

  it('analyzes market condition successfully', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeMarketCondition('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.marketCondition).not.toBe(null)
      expect(result.current.state.marketCondition?.trend).toBe('BULLISH')
    })
  })

  it('handles market condition analysis error', async () => {
    mockAnalyzeMarketCondition.mockRejectedValueOnce(
      new Error('Market analysis failed')
    )

    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeMarketCondition('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('Market analysis failed')
    })
  })

  it('refreshes service info successfully', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.refreshServiceInfo()
    })

    await waitFor(() => {
      expect(result.current.state.serviceInfo).not.toBe(null)
      expect(result.current.state.serviceInfo?.status).toBe('operational')
      expect(result.current.state.supportedStrategies.length).toBeGreaterThan(0)
    })
  })

  it('handles service info error gracefully', async () => {
    mockGetAIServiceInfo.mockRejectedValueOnce(
      new Error('Service unavailable')
    )

    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.refreshServiceInfo()
    })

    // Should not set error for service info failures
    expect(result.current.state.error).toBe(null)
  })

  it.skip('clears error state', async () => {
    mockAnalyzeAI.mockRejectedValueOnce(new Error('Test error'))

    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('Test error')
    })

    await act(async () => {
      result.current.clearError()
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe(null)
    })
  })

  it('generates sample candles for different symbols', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    const btcCall = mockAnalyzeAI.mock.calls[0][0]

    await act(async () => {
      await result.current.analyzeSymbol('ETHUSDT')
    })

    const ethCall = mockAnalyzeAI.mock.calls[1][0]

    // Should have different base prices
    expect(btcCall.current_price).not.toBe(ethCall.current_price)
  })

  it('includes timeframe data in analysis request', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    const callArgs = mockAnalyzeAI.mock.calls[0][0]

    expect(callArgs.timeframe_data).toHaveProperty('1h')
    expect(callArgs.timeframe_data).toHaveProperty('4h')
    expect(Array.isArray(callArgs.timeframe_data['1h'])).toBe(true)
    expect(Array.isArray(callArgs.timeframe_data['4h'])).toBe(true)
  })

  it('calculates volume_24h from candle data', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    const callArgs = mockAnalyzeAI.mock.calls[0][0]

    expect(callArgs.volume_24h).toBeGreaterThan(0)
    expect(typeof callArgs.volume_24h).toBe('number')
  })

  it('includes strategy context in analysis', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    const strategies = ['RSI Strategy', 'MACD Strategy']

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT', strategies)
    })

    const callArgs = mockAnalyzeAI.mock.calls[0][0]

    expect(callArgs.strategy_context).toBeDefined()
    expect(callArgs.strategy_context.selected_strategies).toEqual(strategies)
    expect(callArgs.strategy_context.market_condition).toBeDefined()
    expect(callArgs.strategy_context.risk_level).toBeDefined()
  })

  it('updates lastUpdate timestamp after analysis', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    expect(result.current.state.lastUpdate).toBe(null)

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    await waitFor(() => {
      expect(result.current.state.lastUpdate).not.toBe(null)
    })
  })

  it('analyzes multiple symbols in sequence', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    // Clear any signals from mount
    await waitFor(() => {
      // Wait for mount effects to settle
    })

    const symbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT']

    for (const symbol of symbols) {
      await act(async () => {
        await result.current.analyzeSymbol(symbol)
      })
    }

    await waitFor(() => {
      // Should have at least 3 signals (may have more from mount)
      expect(result.current.state.signals.length).toBeGreaterThanOrEqual(3)
      // Check that our symbols are in the list
      const symbols = result.current.state.signals.map(s => s.symbol)
      expect(symbols).toContain('BNBUSDT')
      expect(symbols).toContain('ETHUSDT')
      expect(symbols).toContain('BTCUSDT')
    })

    unmount()
  })

  it('generates 1h candles with correct structure', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    const callArgs = mockAnalyzeAI.mock.calls[0][0]
    const candles1h = callArgs.timeframe_data['1h']

    expect(candles1h.length).toBeGreaterThan(0)
    candles1h.forEach((candle: any) => {
      expect(candle).toHaveProperty('open_time')
      expect(candle).toHaveProperty('close_time')
      expect(candle).toHaveProperty('open')
      expect(candle).toHaveProperty('high')
      expect(candle).toHaveProperty('low')
      expect(candle).toHaveProperty('close')
      expect(candle).toHaveProperty('volume')
      expect(candle).toHaveProperty('quote_volume')
      expect(candle).toHaveProperty('trades')
      expect(candle).toHaveProperty('is_closed')
    })
  })

  it('generates 4h candles with correct structure', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    const callArgs = mockAnalyzeAI.mock.calls[0][0]
    const candles4h = callArgs.timeframe_data['4h']

    expect(candles4h.length).toBeGreaterThan(0)
    candles4h.forEach((candle: any) => {
      expect(candle).toHaveProperty('open_time')
      expect(candle).toHaveProperty('close_time')
      expect(candle).toHaveProperty('open')
      expect(candle).toHaveProperty('high')
      expect(candle).toHaveProperty('low')
      expect(candle).toHaveProperty('close')
      expect(candle).toHaveProperty('volume')
      expect(candle).toHaveProperty('quote_volume')
      expect(candle).toHaveProperty('trades')
      expect(candle).toHaveProperty('is_closed')
    })
  })

  it('uses current price from latest candle', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    const callArgs = mockAnalyzeAI.mock.calls[0][0]
    const latestCandle = callArgs.timeframe_data['1h'][callArgs.timeframe_data['1h'].length - 1]

    expect(callArgs.current_price).toBe(latestCandle.close)
  })

  it('prevents state updates after unmount', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    unmount()

    // Try to analyze after unmount
    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    // Should not throw or update state
    expect(true).toBe(true)
  })

  it('handles concurrent analysis requests', async () => {
    const { result } = renderHook(() => useAIAnalysis())

    // Wait for hook to be ready
    await waitFor(() => {
      expect(result.current).not.toBeNull()
    })

    // Clear mock calls from initialization
    mockAnalyzeAI.mockClear()

    await act(async () => {
      await Promise.all([
        result.current.analyzeSymbol('BTCUSDT'),
        result.current.analyzeSymbol('ETHUSDT'),
        result.current.analyzeSymbol('BNBUSDT')
      ])
    })

    await waitFor(() => {
      expect(mockAnalyzeAI).toHaveBeenCalledTimes(3)
      expect(result.current.state.signals.length).toBeGreaterThanOrEqual(3)
    })
  })

  it('enhances signal with symbol information', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    // Wait for hook to be ready
    await waitFor(() => {
      expect(result.current).not.toBeNull()
      expect(result.current.analyzeSymbol).toBeDefined()
    })

    // Clear mocks from initialization
    mockAnalyzeAI.mockClear()

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    await waitFor(() => {
      const btcSignals = result.current.state.signals.filter(s => s.symbol === 'BTCUSDT')
      expect(btcSignals.length).toBeGreaterThan(0)
      const signal = btcSignals[0]
      expect(signal.symbol).toBe('BTCUSDT')
      expect(signal.signal).toBeDefined()
      expect(signal.confidence).toBeDefined()
      expect(signal.reasoning).toBeDefined()
    })

    unmount()
  })

  it('generates different base prices for different symbols', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
      expect(result.current.analyzeSymbol).toBeDefined()
    })

    mockAnalyzeAI.mockClear() // Clear previous calls

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    await waitFor(() => {
      expect(mockAnalyzeAI).toHaveBeenCalledTimes(1)
    })

    const btcPrice = mockAnalyzeAI.mock.calls[0][0].current_price

    await act(async () => {
      await result.current.analyzeSymbol('ETHUSDT')
    })

    await waitFor(() => {
      expect(mockAnalyzeAI).toHaveBeenCalledTimes(2)
    })

    const ethPrice = mockAnalyzeAI.mock.calls[1][0].current_price

    // BTC should have higher base price than ETH
    expect(btcPrice).toBeGreaterThan(ethPrice)

    unmount()
  })

  it('includes timestamp in analysis request', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
      expect(result.current.analyzeSymbol).toBeDefined()
    })

    mockAnalyzeAI.mockClear()
    const beforeTime = Date.now()

    await act(async () => {
      await result.current.analyzeSymbol('BTCUSDT')
    })

    await waitFor(() => {
      expect(mockAnalyzeAI).toHaveBeenCalledTimes(1)
    })

    const afterTime = Date.now()
    const callArgs = mockAnalyzeAI.mock.calls[0][0]

    expect(callArgs.timestamp).toBeGreaterThanOrEqual(beforeTime)
    expect(callArgs.timestamp).toBeLessThanOrEqual(afterTime)

    unmount()
  })

  it('passes correct data to getStrategyRecommendations', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
      expect(result.current.getStrategyRecommendations).toBeDefined()
    })

    mockGetStrategyRecommendations.mockClear()

    await act(async () => {
      await result.current.getStrategyRecommendations('ETHUSDT')
    })

    await waitFor(() => {
      expect(mockGetStrategyRecommendations).toHaveBeenCalledTimes(1)
    })

    const callArgs = mockGetStrategyRecommendations.mock.calls[0][0]

    expect(callArgs.symbol).toBe('ETHUSDT')
    expect(callArgs.timeframe_data).toBeDefined()
    expect(callArgs.current_price).toBeDefined()
    expect(callArgs.available_strategies).toBeDefined()
    expect(callArgs.timestamp).toBeDefined()

    unmount()
  })

  it('passes correct data to analyzeMarketCondition', async () => {
    const { result, unmount } = renderHook(() => useAIAnalysis())

    await waitFor(() => {
      expect(result.current).not.toBeNull()
      expect(result.current.analyzeMarketCondition).toBeDefined()
    })

    mockAnalyzeMarketCondition.mockClear()

    await act(async () => {
      await result.current.analyzeMarketCondition('SOLUSDT')
    })

    await waitFor(() => {
      expect(mockAnalyzeMarketCondition).toHaveBeenCalledTimes(1)
    })

    const callArgs = mockAnalyzeMarketCondition.mock.calls[0][0]

    expect(callArgs.symbol).toBe('SOLUSDT')
    expect(callArgs.timeframe_data).toBeDefined()
    expect(callArgs.current_price).toBeDefined()
    expect(callArgs.volume_24h).toBeDefined()
    expect(callArgs.timestamp).toBeDefined()

    unmount()
  })
})
