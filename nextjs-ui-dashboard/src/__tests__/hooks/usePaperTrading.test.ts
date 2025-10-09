import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { usePaperTrading } from '../../hooks/usePaperTrading'

// Mock the toast hook
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: vi.fn()
  })
}))

// Mock WebSocket
class MockWebSocket {
  public readyState = WebSocket.CONNECTING
  public onopen: ((ev: Event) => void) | null = null
  public onclose: ((ev: CloseEvent) => void) | null = null
  public onerror: ((ev: Event) => void) | null = null
  public onmessage: ((ev: MessageEvent) => void) | null = null
  public sent: string[] = []

  send(data: string) {
    this.sent.push(data)
  }

  close() {
    this.readyState = WebSocket.CLOSED
    if (this.onclose) {
      this.onclose(new CloseEvent('close'))
    }
  }

  triggerOpen() {
    this.readyState = WebSocket.OPEN
    if (this.onopen) {
      this.onopen(new Event('open'))
    }
  }

  triggerMessage(data: unknown) {
    if (this.onmessage) {
      this.onmessage(new MessageEvent('message', { data: JSON.stringify(data) }))
    }
  }

  triggerError() {
    if (this.onerror) {
      this.onerror(new Event('error'))
    }
  }

  triggerClose() {
    this.readyState = WebSocket.CLOSED
    if (this.onclose) {
      this.onclose(new CloseEvent('close'))
    }
  }
}

let mockWs: MockWebSocket

class WebSocketMockClass {
  static CONNECTING = 0
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3

  constructor(url: string) {
    mockWs = new MockWebSocket()
    return mockWs as any
  }
}

describe('usePaperTrading', () => {
  beforeEach(() => {
    mockWs = undefined as unknown as MockWebSocket
    vi.stubGlobal('WebSocket', WebSocketMockClass)

    // Setup default fetch mock that returns empty successful responses
    const defaultMockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', defaultMockFetch)
  })

  afterEach(() => {
    vi.unstubAllGlobals()
    vi.restoreAllMocks()
  })

  it('initializes with default state', () => {
    const { result } = renderHook(() => usePaperTrading())

    expect(result.current.isActive).toBe(false)
    expect(result.current.isLoading).toBe(false)
    expect(result.current.error).toBe(null)
    expect(result.current.openTrades).toEqual([])
    expect(result.current.closedTrades).toEqual([])
    expect(result.current.recentSignals).toEqual([])
  })

  it('has default portfolio metrics', () => {
    const { result } = renderHook(() => usePaperTrading())

    expect(result.current.portfolio).toMatchObject({
      total_trades: 0,
      win_rate: 0,
      total_pnl: 0,
      current_balance: 10000,
      equity: 10000,
      margin_used: 0,
      free_margin: 10000
    })
  })

  it('has default settings', () => {
    const { result } = renderHook(() => usePaperTrading())

    expect(result.current.settings.basic).toMatchObject({
      initial_balance: 10000,
      max_positions: 10,
      default_position_size_pct: 5.0,
      default_leverage: 10,
      enabled: true
    })

    expect(result.current.settings.risk).toMatchObject({
      max_risk_per_trade_pct: 2.0,
      max_portfolio_risk_pct: 20.0,
      default_stop_loss_pct: 2.0,
      default_take_profit_pct: 4.0
    })
  })

  it('fetches bot status on mount', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({
        success: true,
        data: {
          is_running: true,
          portfolio: {
            current_balance: 15000,
            total_pnl: 5000
          },
          last_updated: new Date().toISOString()
        }
      })
    })
    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/status')
      )
    })
  })

  it('fetches open trades on mount', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({
        success: true,
        data: []
      })
    })
    vi.stubGlobal('fetch', mockFetch)

    renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/trades/open')
      )
    })
  })

  it('fetches closed trades on mount', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({
        success: true,
        data: []
      })
    })
    vi.stubGlobal('fetch', mockFetch)

    renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/trades/closed')
      )
    })
  })

  it('starts trading successfully', async () => {
    const mockFetch = vi.fn()
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { is_running: false } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({
          success: true,
          data: { message: 'Trading started' }
        })
      })
      .mockResolvedValue({
        json: async () => ({ success: true, data: {} })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await act(async () => {
      await result.current.startTrading()
    })

    await waitFor(() => {
      expect(result.current.isActive).toBe(true)
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('stops trading successfully', async () => {
    const mockFetch = vi.fn()
      // Initial mount calls
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { is_running: true, portfolio: {}, last_updated: new Date().toISOString() } })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      // fetchAISignals makes 4 calls (one per symbol)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      // The actual test call - stop trading
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: { message: 'Trading stopped' }
        })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(result.current.isActive).toBe(true)
    })

    await act(async () => {
      await result.current.stopTrading()
    })

    await waitFor(() => {
      expect(result.current.isActive).toBe(false)
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('handles start trading error', async () => {
    const mockFetch = vi.fn()
      // Initial mount calls
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { is_running: false, portfolio: {}, last_updated: new Date().toISOString() } })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      // fetchAISignals makes 4 calls (one per symbol)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      // The actual test call - start trading
      .mockResolvedValueOnce({
        ok: false,
        json: async () => ({
          success: false,
          error: 'Failed to start'
        })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(result.current.isActive).toBe(false)
    })

    await act(async () => {
      await result.current.startTrading()
    })

    await waitFor(() => {
      expect(result.current.error).toBe('Failed to start')
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('handles stop trading error', async () => {
    const mockFetch = vi.fn()
      // Initial mount calls
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { is_running: true, portfolio: {}, last_updated: new Date().toISOString() } })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      // fetchAISignals makes 4 calls (one per symbol)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      // The actual test call - stop trading
      .mockResolvedValueOnce({
        ok: false,
        json: async () => ({
          success: false,
          error: 'Failed to stop'
        })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(result.current.isActive).toBe(true)
    })

    await act(async () => {
      await result.current.stopTrading()
    })

    await waitFor(() => {
      expect(result.current.error).toBe('Failed to stop')
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('closes a trade successfully', async () => {
    const mockFetch = vi.fn()
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { is_running: false } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { message: 'Trade closed' } })
      })
      .mockResolvedValue({
        json: async () => ({ success: true, data: [] })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await act(async () => {
      await result.current.closeTrade('trade123')
    })

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/trades/trade123/close'),
        expect.any(Object)
      )
    })
  })

  it('handles close trade error', async () => {
    const mockFetch = vi.fn()
      // Initial mount calls
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { is_running: false, portfolio: {}, last_updated: new Date().toISOString() } })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      // fetchAISignals makes 4 calls (one per symbol)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      // The actual test call - close trade
      .mockResolvedValueOnce({
        ok: false,
        json: async () => ({
          success: false,
          error: 'Trade not found'
        })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(result.current.isActive).toBe(false)
    })

    await act(async () => {
      await result.current.closeTrade('invalid-trade')
    })

    await waitFor(() => {
      expect(result.current.error).toBe('Trade not found')
    })
  })

  it('updates settings successfully', async () => {
    const mockFetch = vi.fn()
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { is_running: false } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { message: 'Settings updated' } })
      })
      .mockResolvedValue({
        json: async () => ({ success: true, data: {} })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    const newSettings = {
      basic: {
        initial_balance: 20000,
        max_positions: 5,
        default_position_size_pct: 10,
        default_leverage: 5,
        trading_fee_rate: 0.001,
        funding_fee_rate: 0.0001,
        slippage_pct: 0.01,
        enabled: true,
        auto_restart: false
      },
      risk: {
        max_risk_per_trade_pct: 1.0,
        max_portfolio_risk_pct: 10.0,
        default_stop_loss_pct: 1.0,
        default_take_profit_pct: 2.0,
        max_leverage: 25,
        min_margin_level: 150.0,
        max_drawdown_pct: 10.0,
        daily_loss_limit_pct: 3.0,
        max_consecutive_losses: 3,
        cool_down_minutes: 30
      }
    }

    await act(async () => {
      await result.current.updateSettings(newSettings)
    })

    await waitFor(() => {
      expect(result.current.settings).toEqual(newSettings)
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('handles update settings error', async () => {
    const mockFetch = vi.fn()
      // Initial mount calls
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { is_running: false, portfolio: {}, last_updated: new Date().toISOString() } })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      // fetchAISignals makes 4 calls (one per symbol)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      // The actual test call - update settings
      .mockResolvedValueOnce({
        ok: false,
        json: async () => ({
          success: false,
          error: 'Invalid settings'
        })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(result.current.isActive).toBe(false)
    })

    const newSettings = {
      basic: {
        initial_balance: -1000,
        max_positions: 5,
        default_position_size_pct: 10,
        default_leverage: 5,
        trading_fee_rate: 0.001,
        funding_fee_rate: 0.0001,
        slippage_pct: 0.01,
        enabled: true,
        auto_restart: false
      },
      risk: {
        max_risk_per_trade_pct: 1.0,
        max_portfolio_risk_pct: 10.0,
        default_stop_loss_pct: 1.0,
        default_take_profit_pct: 2.0,
        max_leverage: 25,
        min_margin_level: 150.0,
        max_drawdown_pct: 10.0,
        daily_loss_limit_pct: 3.0,
        max_consecutive_losses: 3,
        cool_down_minutes: 30
      }
    }

    await act(async () => {
      await result.current.updateSettings(newSettings)
    })

    await waitFor(() => {
      expect(result.current.error).toBe('Invalid settings')
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('resets portfolio successfully', async () => {
    const mockFetch = vi.fn()
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { is_running: false } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        json: async () => ({ success: true, data: { message: 'Portfolio reset' } })
      })
      .mockResolvedValue({
        json: async () => ({ success: true, data: [] })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await act(async () => {
      await result.current.resetPortfolio()
    })

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/reset'),
        expect.any(Object)
      )
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('handles reset portfolio error', async () => {
    const mockFetch = vi.fn()
      // Initial mount calls
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { is_running: false, portfolio: {}, last_updated: new Date().toISOString() } })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: [] })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: { basic: {}, risk: {} } })
      })
      // fetchAISignals makes 4 calls (one per symbol)
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      .mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: false, data: null })
      })
      // The actual test call - reset portfolio
      .mockResolvedValueOnce({
        ok: false,
        json: async () => ({
          success: false,
          error: 'Reset failed'
        })
      })

    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(result.current.isActive).toBe(false)
    })

    await act(async () => {
      await result.current.resetPortfolio()
    })

    await waitFor(() => {
      expect(result.current.error).toBe('Reset failed')
      expect(result.current.isLoading).toBe(false)
    })
  })

  it('handles WebSocket connection', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })
  })

  it('handles WebSocket market data message', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
      // Portfolio is initialized with defaults, not from fetch
      expect(result.current.portfolio.current_balance).toBe(10000)
    })

    act(() => {
      mockWs.triggerOpen()
    })

    // Set initial open trades via state
    act(() => {
      result.current.openTrades.push({
        id: 'trade1',
        symbol: 'BTCUSDT',
        trade_type: 'Long',
        entry_price: 48000,
        quantity: 1,
        leverage: 10,
        status: 'Open',
        pnl: 0,
        pnl_percentage: 0,
        open_time: new Date().toISOString()
      })
    })

    act(() => {
      mockWs.triggerMessage({
        event_type: 'MarketData',
        data: {
          symbol: 'BTCUSDT',
          price: 50000
        }
      })
    })

    await waitFor(() => {
      expect(result.current.lastUpdated).not.toBe(null)
    })
  })

  it('handles WebSocket AI signal message', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    act(() => {
      mockWs.triggerOpen()
    })

    const aiSignal = {
      id: 'signal1',
      symbol: 'BTCUSDT',
      signal: 'LONG',
      confidence: 0.85,
      timestamp: new Date(),
      reasoning: 'Strong uptrend',
      strategy_scores: { RSI: 0.8 },
      market_analysis: {
        trend_direction: 'UP',
        trend_strength: 0.9,
        support_levels: [48000],
        resistance_levels: [52000],
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
    }

    act(() => {
      mockWs.triggerMessage({
        event_type: 'AISignalReceived',
        data: aiSignal
      })
    })

    await waitFor(() => {
      expect(result.current.recentSignals.length).toBeGreaterThan(0)
    })
  })

  it('handles WebSocket trade executed message', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        event_type: 'trade_executed',
        data: {
          id: 'trade1',
          symbol: 'BTCUSDT',
          side: 'LONG'
        }
      })
    })

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalled()
    })
  })

  it('handles WebSocket trade closed message', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        event_type: 'trade_closed',
        data: {
          id: 'trade1',
          pnl: 100
        }
      })
    })

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalled()
    })
  })

  it('handles WebSocket pong message', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        event_type: 'Pong'
      })
    })

    // Should not cause any errors
    expect(mockWs).toBeDefined()
  })

  it('handles WebSocket connected message', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        event_type: 'Connected',
        message: 'WebSocket connected'
      })
    })

    await waitFor(() => {
      expect(result.current.lastUpdated).not.toBe(null)
    })
  })

  it('handles WebSocket performance update message', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        event_type: 'performance_update',
        data: {
          total_pnl: 1000,
          win_rate: 0.65
        }
      })
    })

    await waitFor(() => {
      expect(result.current.portfolio.total_pnl).toBe(1000)
      expect(result.current.portfolio.win_rate).toBe(0.65)
    })
  })

  it('deduplicates AI signals by symbol', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    act(() => {
      mockWs.triggerOpen()
    })

    // Send multiple signals for the same symbol
    const signal1 = {
      id: 'signal1',
      symbol: 'BTCUSDT',
      signal: 'LONG',
      confidence: 0.85,
      timestamp: new Date(Date.now() - 1000),
      reasoning: 'Old signal',
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
        stop_loss_suggestion: null,
        take_profit_suggestion: null
      }
    }

    const signal2 = {
      ...signal1,
      id: 'signal2',
      timestamp: new Date(),
      reasoning: 'New signal'
    }

    act(() => {
      mockWs.triggerMessage({
        event_type: 'AISignalReceived',
        data: signal1
      })
    })

    // Wait for the first signal to be added
    await waitFor(() => {
      expect(result.current.recentSignals.length).toBeGreaterThan(0)
    })

    act(() => {
      mockWs.triggerMessage({
        event_type: 'AISignalReceived',
        data: signal2
      })
    })

    // Wait for deduplication to occur
    await waitFor(() => {
      // Should only keep the most recent signal for BTCUSDT
      const btcSignals = result.current.recentSignals.filter(s => s.symbol === 'BTCUSDT')
      expect(btcSignals.length).toBe(1)
      expect(btcSignals[0].id).toBe('signal2')
    }, { timeout: 3000 })
  })

  it('cleans up WebSocket on unmount', async () => {
    const mockFetch = vi.fn().mockResolvedValue({
      ok: true,
      json: async () => ({ success: true, data: [] })
    })
    vi.stubGlobal('fetch', mockFetch)

    const { unmount } = renderHook(() => usePaperTrading())

    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    // Open the WebSocket connection first
    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => {
      expect(mockWs.readyState).toBe(WebSocket.OPEN)
    })

    const closeSpy = vi.spyOn(mockWs, 'close')

    unmount()

    // Wait for cleanup to occur
    await waitFor(() => {
      expect(closeSpy).toHaveBeenCalled()
    })
  })

  it('provides refresh functions', () => {
    const { result } = renderHook(() => usePaperTrading())

    expect(result.current.refreshData).toBeDefined()
    expect(result.current.refreshStatus).toBeDefined()
    expect(result.current.refreshSettings).toBeDefined()
    expect(result.current.refreshAISignals).toBeDefined()
    expect(result.current.refreshTrades).toBeDefined()
  })

  it('handles network errors gracefully', async () => {
    const mockFetch = vi.fn().mockRejectedValue(new Error('Network error'))
    vi.stubGlobal('fetch', mockFetch)

    const { result } = renderHook(() => usePaperTrading())

    await act(async () => {
      await result.current.startTrading()
    })

    await waitFor(() => {
      expect(result.current.error).toBe('Network error')
      expect(result.current.isLoading).toBe(false)
    })
  })
})
