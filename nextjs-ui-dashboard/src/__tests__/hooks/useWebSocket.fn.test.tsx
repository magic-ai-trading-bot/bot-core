/**
 * useWebSocket Hook - Function Coverage Boost Tests (Simplified)
 *
 * Tests uncovered code paths to increase coverage from 90% to 95%+
 * Focus: Simple, focused tests that target specific uncovered lines
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useWebSocket } from '@/hooks/useWebSocket'

// Mock WebSocket
class MockWebSocket {
  public readyState = 0
  public onopen: ((ev: Event) => void) | null = null
  public onclose: ((ev: CloseEvent) => void) | null = null
  public onerror: ((ev: Event) => void) | null = null
  public onmessage: ((ev: MessageEvent) => void) | null = null
  public sent: string[] = []

  send(data: string) {
    if (this.readyState === 1) {
      this.sent.push(data)
    }
  }

  close() {
    this.readyState = 3
    if (this.onclose) {
      this.onclose(new CloseEvent('close'))
    }
  }

  triggerOpen() {
    this.readyState = 1
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
}

let mockWs: MockWebSocket

class WebSocketMockClass {
  static CONNECTING = 0
  static OPEN = 1
  static CLOSING = 2
  static CLOSED = 3

  constructor() {
    mockWs = new MockWebSocket()
    return mockWs as any
  }
}

describe('useWebSocket - Function Coverage Boost (Simplified)', () => {
  beforeEach(() => {
    mockWs = undefined as unknown as MockWebSocket
    Object.defineProperty(global, 'WebSocket', {
      writable: true,
      configurable: true,
      value: WebSocketMockClass,
    })
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  // Test message parsing error
  it('should handle malformed JSON gracefully', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => expect(result.current.state.isConnected).toBe(true))

    // Send malformed JSON
    act(() => {
      if (mockWs.onmessage) {
        mockWs.onmessage(new MessageEvent('message', { data: 'not valid json {' }))
      }
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('Failed to parse WebSocket message')
    })
  })

  // Test PositionUpdate without data
  it('should handle PositionUpdate without data', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        type: 'PositionUpdate',
        timestamp: new Date().toISOString()
      })
    })

    // Should not crash
    expect(result.current.state).toBeDefined()
  })

  // Test Error without data
  it('should handle Error message without data', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        type: 'Error',
        timestamp: new Date().toISOString()
      })
    })

    // Should not set error when data is missing
    expect(result.current.state.error).toBeNull()
  })

  // Test Error with data
  it('should extract error message from ErrorData', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        type: 'Error',
        data: {
          message: 'Custom error',
          code: 'ERR_TEST'
        },
        timestamp: new Date().toISOString()
      })
    })

    await waitFor(() => {
      expect(result.current.state.error).toBe('Custom error')
    })
  })

  // Test server Ping response
  it('should respond to server Ping with Pong', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    mockWs.sent = []

    act(() => {
      mockWs.triggerMessage({
        type: 'Ping',
        timestamp: new Date().toISOString()
      })
    })

    await waitFor(() => {
      const pongs = mockWs.sent.filter(msg => {
        try {
          return JSON.parse(msg).type === 'Pong'
        } catch {
          return false
        }
      })
      expect(pongs.length).toBeGreaterThan(0)
    })
  })

  // Test BotStatusUpdate
  it('should update bot status from BotStatusUpdate', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        type: 'BotStatusUpdate',
        data: {
          status: 'running',
          active_positions: 3,
          total_pnl: 1500,
          total_trades: 25,
          uptime: 3600000
        },
        timestamp: new Date().toISOString()
      })
    })

    await waitFor(() => {
      expect(result.current.state.botStatus).toBeDefined()
      expect(result.current.state.botStatus?.status).toBe('running')
      expect(result.current.state.botStatus?.active_positions).toBe(3)
    })
  })

  // Test disconnect clears WebSocket
  it('should clear WebSocket on disconnect', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => expect(result.current.state.isConnected).toBe(true))

    const closeSpy = vi.spyOn(mockWs, 'close')

    act(() => {
      result.current.disconnect()
    })

    await waitFor(() => {
      expect(closeSpy).toHaveBeenCalled()
      expect(result.current.state.isConnected).toBe(false)
    })
  })

  // Test sendMessage when connected
  it('should send message when connected', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => expect(result.current.state.isConnected).toBe(true))

    mockWs.sent = []

    act(() => {
      result.current.sendMessage({
        type: 'test',
        data: { hello: 'world' }
      })
    })

    expect(mockWs.sent.length).toBeGreaterThan(0)
    const parsed = JSON.parse(mockWs.sent[0])
    expect(parsed.type).toBe('test')
  })

  // Test cleanup on unmount
  it('should cleanup on unmount', async () => {
    const { result, unmount } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    const closeSpy = vi.spyOn(mockWs, 'close')

    act(() => {
      mockWs.triggerOpen()
    })

    unmount()

    expect(closeSpy).toHaveBeenCalled()
  })

  // Test no duplicate WebSocket when already connecting
  it('should not create duplicate WebSocket when connecting', () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    const ws1 = mockWs

    // Try connect again immediately
    act(() => {
      result.current.connect()
    })

    expect(mockWs).toBe(ws1)
  })

  // Test no duplicate WebSocket when already connected
  it('should not create duplicate WebSocket when connected', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => expect(result.current.state.isConnected).toBe(true))

    const ws1 = mockWs

    act(() => {
      result.current.connect()
    })

    expect(mockWs).toBe(ws1)
  })

  // Test handleClose sets isConnected to false
  it('should set isConnected to false on close', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    await waitFor(() => expect(result.current.state.isConnected).toBe(true))

    act(() => {
      mockWs.close()
    })

    await waitFor(() => {
      expect(result.current.state.isConnected).toBe(false)
    })
  })

  // Test TradeExecuted without data
  it('should handle TradeExecuted without data', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        type: 'TradeExecuted',
        timestamp: new Date().toISOString()
      })
    })

    // Should not crash
    expect(result.current.state).toBeDefined()
  })

  // Test AISignalReceived without data
  it('should handle AISignalReceived without data', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        type: 'AISignalReceived',
        timestamp: new Date().toISOString()
      })
    })

    // Should not crash
    expect(result.current.state).toBeDefined()
  })

  // Test BotStatusUpdate without data
  it('should handle BotStatusUpdate without data', async () => {
    const { result } = renderHook(() => useWebSocket())

    act(() => {
      result.current.connect()
    })

    await waitFor(() => expect(mockWs).toBeDefined())

    act(() => {
      mockWs.triggerOpen()
    })

    act(() => {
      mockWs.triggerMessage({
        type: 'BotStatusUpdate',
        timestamp: new Date().toISOString()
      })
    })

    // Should not crash
    expect(result.current.state).toBeDefined()
  })
})
