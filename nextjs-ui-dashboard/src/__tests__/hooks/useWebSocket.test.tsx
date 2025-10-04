import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useWebSocket } from '../../hooks/useWebSocket'

// Mock environment to disable auto-connect
vi.stubEnv('VITE_ENABLE_REALTIME', 'false')
vi.stubEnv('VITE_WS_URL', 'ws://localhost:8080/ws')

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

  // Test helper methods
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

// Mock the global WebSocket
global.WebSocket = vi.fn(() => {
  mockWs = new MockWebSocket()
  return mockWs as unknown as WebSocket
}) as unknown as typeof WebSocket

describe('useWebSocket', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Reset mockWs
    mockWs = undefined as unknown as MockWebSocket
  })

  // Helper function to connect and wait for WebSocket
  const connectAndWait = async (result: any) => {
    act(() => {
      result.current.connect()
    })
    await waitFor(() => expect(mockWs).toBeDefined())
    act(() => {
      mockWs.triggerOpen()
    })
    await waitFor(() => expect(result.current.state.isConnected).toBe(true))
  }

  it('initializes with disconnected state', () => {
    const { result } = renderHook(() => useWebSocket())

    expect(result.current.state.isConnected).toBe(false)
    expect(result.current.state.isConnecting).toBe(false)
    expect(result.current.state.error).toBe(null)
  })

  it('connects to WebSocket server', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    // Wait for mockWs to be created
    await waitFor(() => {
      expect(mockWs).toBeDefined()
    })

    expect(result.current.state.isConnecting).toBe(true)
    expect(global.WebSocket).toHaveBeenCalledWith('ws://localhost:8080/ws')

    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => {
      expect(result.current.state.isConnected).toBe(true)
      expect(result.current.state.isConnecting).toBe(false)
    })
  })

  it('disconnects from WebSocket server', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => {
      expect(result.current.state.isConnected).toBe(true)
    })

    act(() => {
      result.current.disconnect()
    })

    expect(result.current.state.isConnected).toBe(false)
  })

  it('sends messages when connected', async () => {
    const { result } = renderHook(() => useWebSocket())
    await connectAndWait(result)

    const message = { type: 'test', data: 'hello' }

    act(() => {
      result.current.sendMessage(message)
    })

    expect(mockWs.sent).toHaveLength(1)
    expect(JSON.parse(mockWs.sent[0])).toEqual(message)
  })

  it('handles incoming messages', async () => {
    const { result } = renderHook(() => useWebSocket())
    await connectAndWait(result)

    const messageData = {
      type: 'AISignalReceived',
      data: {
        symbol: 'BTCUSDT',
        signal: 'long',
        confidence: 0.85,
        timestamp: Date.now(),
        model_type: 'LSTM',
        timeframe: '1h'
      },
      timestamp: new Date().toISOString()
    }

    act(() => {
      mockWs.triggerMessage(messageData)
    })

    await waitFor(() => {
      expect(result.current.state.lastMessage).toEqual(messageData)
      expect(result.current.state.aiSignals).toHaveLength(1)
      expect(result.current.state.aiSignals[0].signal).toBe('long')
    })
  })

  it('handles connection errors', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerError()
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('WebSocket connection error')
      expect(result.current.state.isConnecting).toBe(false)
    })
  })

  it('handles position updates', async () => {
    const { result } = renderHook(() => useWebSocket())
    await connectAndWait(result)

    // First add a position by triggering a position update
    const positionMessage = {
      type: 'PositionUpdate',
      data: {
        symbol: 'BTCUSDT',
        side: 'LONG',
        pnl: 100,
        current_price: 45000,
        unrealized_pnl: 50,
        timestamp: Date.now()
      },
      timestamp: new Date().toISOString()
    }

    act(() => {
      mockWs.triggerMessage(positionMessage)
    })

    await waitFor(() => {
      expect(result.current.state.lastMessage?.type).toBe('PositionUpdate')
    })
  })

  it('handles trade execution messages', async () => {
    const { result } = renderHook(() => useWebSocket())
    await connectAndWait(result)

    const tradeMessage = {
      type: 'TradeExecuted',
      data: {
        symbol: 'ETHUSDT',
        side: 'BUY',
        quantity: 1.0,
        price: 2500,
        timestamp: Date.now()
      },
      timestamp: new Date().toISOString()
    }

    act(() => {
      mockWs.triggerMessage(tradeMessage)
    })

    await waitFor(() => {
      expect(result.current.state.recentTrades).toHaveLength(1)
      expect(result.current.state.recentTrades[0].symbol).toBe('ETHUSDT')
    })
  })

  it('handles bot status updates', async () => {
    const { result } = renderHook(() => useWebSocket())
    await connectAndWait(result)

    const statusMessage = {
      type: 'BotStatusUpdate',
      data: {
        status: 'running',
        active_positions: 3,
        total_pnl: 250.50,
        total_trades: 15,
        uptime: 3600
      },
      timestamp: new Date().toISOString()
    }

    act(() => {
      mockWs.triggerMessage(statusMessage)
    })

    await waitFor(() => {
      expect(result.current.state.botStatus).toBeTruthy()
      expect(result.current.state.botStatus?.status).toBe('running')
      expect(result.current.state.botStatus?.active_positions).toBe(3)
    })
  })

  it('handles malformed JSON messages', async () => {
    const { result } = renderHook(() => useWebSocket())
    await connectAndWait(result)

    // Trigger a message event with invalid JSON
    act(() => {
      if (mockWs.onmessage) {
        mockWs.onmessage(new MessageEvent('message', { data: 'invalid json{' }))
      }
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('Failed to parse WebSocket message')
    })
  })

  it('cleans up on unmount', async () => {
    const { result, unmount } = renderHook(() => useWebSocket())
    await connectAndWait(result)

    const closeSpy = vi.spyOn(mockWs, 'close')

    unmount()

    expect(closeSpy).toHaveBeenCalled()
  })
})
