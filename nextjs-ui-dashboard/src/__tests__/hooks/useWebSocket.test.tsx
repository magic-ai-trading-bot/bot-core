import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useWebSocket } from '../../hooks/useWebSocket'

// Environment is mocked in vitest.config.ts to disable auto-connect

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

// Create WebSocket mock class that extends the mock implementation
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

describe('useWebSocket', () => {
  let originalWebSocket: typeof WebSocket

  beforeEach(() => {
    // Reset mockWs
    mockWs = undefined as unknown as MockWebSocket
    // Save original WebSocket
    originalWebSocket = global.WebSocket
    // Mock global WebSocket with our mock class
    Object.defineProperty(global, 'WebSocket', {
      writable: true,
      configurable: true,
      value: WebSocketMockClass
    })
  })

  afterEach(() => {
    // Restore original WebSocket
    Object.defineProperty(global, 'WebSocket', {
      writable: true,
      configurable: true,
      value: originalWebSocket
    })
    // Reset mockWs
    mockWs = undefined as unknown as MockWebSocket
    vi.clearAllMocks()
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

    expect(result.current.state.isConnecting).toBe(false)

    act(() => {
      result.current.connect()
    })

    expect(result.current.state.isConnecting).toBe(true)

    // Wait for mockWs to be created
    await waitFor(() => {
      expect(mockWs).toBeDefined()
    }, { timeout: 1000 })

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

    // Clear the initial Ping message sent by heartbeat
    mockWs.sent = []

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

  describe('Infinite Loop Prevention', () => {
    it('should not reconnect infinitely on mount', async () => {
      // This test verifies the fix for the infinite loop issue
      let connectionAttempts = 0
      const originalWebSocket = global.WebSocket

      // Create a custom mock that counts connection attempts
      class CountingWebSocketMock {
        static CONNECTING = 0
        static OPEN = 1
        static CLOSING = 2
        static CLOSED = 3

        constructor(url: string) {
          connectionAttempts++
          mockWs = new MockWebSocket()
          return mockWs as any
        }
      }

      Object.defineProperty(global, 'WebSocket', {
        writable: true,
        configurable: true,
        value: CountingWebSocketMock
      })

      const { result } = renderHook(() => useWebSocket())

      // Manually trigger connect since auto-connect is disabled in test env
      act(() => {
        result.current.connect()
      })

      // Wait for initial connection attempt
      await waitFor(() => expect(connectionAttempts).toBe(1), { timeout: 1000 })

      // Trigger open
      act(() => {
        mockWs?.triggerOpen()
      })

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 1000 })

      // Wait a bit to ensure no additional connection attempts
      await new Promise(resolve => setTimeout(resolve, 100))

      // Should only have 1 connection attempt (not infinite)
      expect(connectionAttempts).toBe(1)

      Object.defineProperty(global, 'WebSocket', {
        writable: true,
        configurable: true,
        value: originalWebSocket
      })
    })

    it('should not re-render infinitely when connect function changes', async () => {
      let renderCount = 0
      const CustomHook = () => {
        renderCount++
        return useWebSocket()
      }

      const { result } = renderHook(() => CustomHook())

      // Initial render
      expect(renderCount).toBe(1)

      // Connect
      act(() => {
        result.current.connect()
      })

      await waitFor(() => expect(mockWs).toBeDefined())

      act(() => {
        mockWs.triggerOpen()
      })

      await waitFor(() => expect(result.current.state.isConnected).toBe(true))

      // Should not have excessive re-renders (allow some for state updates)
      expect(renderCount).toBeLessThan(10)
    })

    it('should have stable connect function reference', () => {
      const { result, rerender } = renderHook(() => useWebSocket())

      const firstConnect = result.current.connect
      rerender()
      const secondConnect = result.current.connect

      // connect function should be stable across re-renders
      expect(firstConnect).toBe(secondConnect)
    })

    it('should have stable disconnect function reference', () => {
      const { result, rerender } = renderHook(() => useWebSocket())

      const firstDisconnect = result.current.disconnect
      rerender()
      const secondDisconnect = result.current.disconnect

      // disconnect function should be stable across re-renders
      expect(firstDisconnect).toBe(secondDisconnect)
    })

    it('should have stable sendMessage function reference', () => {
      const { result, rerender } = renderHook(() => useWebSocket())

      const firstSendMessage = result.current.sendMessage
      rerender()
      const secondSendMessage = result.current.sendMessage

      // sendMessage function should be stable across re-renders
      expect(firstSendMessage).toBe(secondSendMessage)
    })

    it('should only auto-connect once on mount', async () => {
      let connectionAttempts = 0

      class CountingWebSocketMock2 {
        static CONNECTING = 0
        static OPEN = 1
        static CLOSING = 2
        static CLOSED = 3

        constructor(url: string) {
          connectionAttempts++
          mockWs = new MockWebSocket()
          return mockWs as any
        }
      }

      Object.defineProperty(global, 'WebSocket', {
        writable: true,
        configurable: true,
        value: CountingWebSocketMock2
      })

      // Mount the hook
      const { result } = renderHook(() => useWebSocket())

      // Manually trigger connect since auto-connect is disabled in test env
      act(() => {
        result.current.connect()
      })

      // Wait for connect to happen
      await waitFor(() => expect(connectionAttempts).toBeGreaterThan(0), { timeout: 1000 })

      const initialAttempts = connectionAttempts

      // Trigger connection open
      act(() => {
        mockWs?.triggerOpen()
      })

      // Wait for connection to be established
      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 1000 })

      // Wait a bit to ensure no additional connections
      await new Promise(resolve => setTimeout(resolve, 100))

      // Should not have additional connection attempts
      expect(connectionAttempts).toBe(initialAttempts)
    })
  })
})
