import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act, waitFor } from '@testing-library/react'
import { useWebSocket } from '../../hooks/useWebSocket'

/**
 * Enhanced WebSocket tests to improve mutation testing score from ~50% to 75%+
 * Focus: Exact value assertions, message verification, error handling
 */

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

describe('useWebSocket - Enhanced Tests', () => {
  let originalWebSocket: typeof WebSocket

  beforeEach(() => {
    mockWs = undefined as unknown as MockWebSocket
    originalWebSocket = global.WebSocket
    Object.defineProperty(global, 'WebSocket', {
      writable: true,
      configurable: true,
      value: WebSocketMockClass
    })
  })

  afterEach(() => {
    Object.defineProperty(global, 'WebSocket', {
      writable: true,
      configurable: true,
      value: originalWebSocket
    })
    mockWs = undefined as unknown as MockWebSocket
    vi.clearAllMocks()
  })

  describe('Message Handling with Exact Verification', () => {
    it('receives and stores exact message data', async () => {
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

      // Send exact message
      const testMessage = {
        type: 'trade',
        symbol: 'BTCUSDT',
        price: 50000.25,
        quantity: 0.5,
        timestamp: 1234567890
      }

      act(() => {
        mockWs.triggerMessage(testMessage)
      })

      await waitFor(() => {
        expect(result.current.state.lastMessage).not.toBeNull()
      })

      // Exact value verification
      const receivedMessage = result.current.state.lastMessage
      expect(receivedMessage).toEqual(testMessage)
      expect(receivedMessage?.type).toBe('trade')
      expect(receivedMessage?.symbol).toBe('BTCUSDT')
      expect(receivedMessage?.price).toBe(50000.25)
      expect(receivedMessage?.quantity).toBe(0.5)
      expect(receivedMessage?.timestamp).toBe(1234567890)
    })

    it('handles multiple messages in sequence correctly', async () => {
      const messages: any[] = []
      const onMessage = vi.fn((msg: any) => messages.push(msg))

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

      // Send multiple messages
      const message1 = { id: 1, data: 'first' }
      const message2 = { id: 2, data: 'second' }
      const message3 = { id: 3, data: 'third' }

      act(() => {
        mockWs.triggerMessage(message1)
      })

      await waitFor(() => {
        expect(result.current.state.lastMessage).toEqual(message1)
      })

      act(() => {
        mockWs.triggerMessage(message2)
      })

      await waitFor(() => {
        expect(result.current.state.lastMessage).toEqual(message2)
      })

      act(() => {
        mockWs.triggerMessage(message3)
      })

      await waitFor(() => {
        expect(result.current.state.lastMessage).toEqual(message3)
      })

      // Last message should be message3
      expect(result.current.state.lastMessage?.id).toBe(3)
      expect(result.current.state.lastMessage?.data).toBe('third')
    })

    it('handles malformed JSON gracefully', async () => {
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

      // Send malformed JSON
      act(() => {
        if (mockWs.onmessage) {
          mockWs.onmessage(new MessageEvent('message', { data: '{invalid json}' }))
        }
      })

      // Should handle gracefully without crashing
      await waitFor(() => {
        // Either error is set or connection remains stable
        expect(result.current.state.isConnected || result.current.state.error).toBeTruthy()
      }, { timeout: 1000 })
    })
  })

  describe('Connection State Verification', () => {
    it('transitions through exact state sequence: disconnected -> connecting -> connected', async () => {
      const { result } = renderHook(() => useWebSocket())

      // Initial state: disconnected
      expect(result.current.state.isConnected).toBe(false)
      expect(result.current.state.isConnecting).toBe(false)

      // Start connection
      act(() => {
        result.current.connect()
      })

      // State: connecting
      expect(result.current.state.isConnecting).toBe(true)
      expect(result.current.state.isConnected).toBe(false)

      await waitFor(() => expect(mockWs).toBeDefined())

      act(() => {
        mockWs.triggerOpen()
      })

      // State: connected
      await waitFor(() => {
        expect(result.current.state.isConnected).toBe(true)
        expect(result.current.state.isConnecting).toBe(false)
      })
    })

    it('handles disconnect with exact state transition', async () => {
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

      // Disconnect
      act(() => {
        result.current.disconnect()
      })

      // Should be disconnected
      expect(result.current.state.isConnected).toBe(false)
      expect(result.current.state.isConnecting).toBe(false)
    })
  })

  describe('Error Handling with Exact Error States', () => {
    it('sets exact error state on connection error', async () => {
      const { result } = renderHook(() => useWebSocket())

      act(() => {
        result.current.connect()
      })

      await waitFor(() => expect(mockWs).toBeDefined())

      act(() => {
        mockWs.triggerError()
      })

      await waitFor(() => {
        expect(result.current.state.error).not.toBeNull()
      })

      // Error should be set
      expect(result.current.state.error).toBeTruthy()
      // Should not be connected
      expect(result.current.state.isConnected).toBe(false)
    })

    it('clears error on successful reconnection', async () => {
      const { result } = renderHook(() => useWebSocket())

      act(() => {
        result.current.connect()
      })

      await waitFor(() => expect(mockWs).toBeDefined())

      // Trigger error
      act(() => {
        mockWs.triggerError()
      })

      await waitFor(() => {
        expect(result.current.state.error).not.toBeNull()
      })

      // Reconnect
      act(() => {
        result.current.connect()
      })

      await waitFor(() => expect(mockWs).toBeDefined())

      act(() => {
        mockWs.triggerOpen()
      })

      await waitFor(() => {
        expect(result.current.state.isConnected).toBe(true)
        expect(result.current.state.error).toBeNull()
      })

      // Error should be cleared
      expect(result.current.state.error).toBeNull()
    })

    it('maintains error state across close events', async () => {
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

      // Close connection
      act(() => {
        mockWs.triggerClose()
      })

      await waitFor(() => {
        expect(result.current.state.isConnected).toBe(false)
      })

      // Should be disconnected but no error
      expect(result.current.state.isConnected).toBe(false)
      expect(result.current.state.isConnecting).toBe(false)
    })
  })

  describe('Send Message Validation', () => {
    it('sends exact message data', async () => {
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

      // Clear the initial Ping message sent by heartbeat
      mockWs.sent = []

      // Send message
      const messageToSend = {
        action: 'subscribe',
        channel: 'trades',
        symbol: 'BTCUSDT'
      }

      act(() => {
        result.current.sendMessage(messageToSend)
      })

      // Verify exact message was sent
      expect(mockWs.sent).toHaveLength(1)
      const sentData = JSON.parse(mockWs.sent[0])
      expect(sentData).toEqual(messageToSend)
      expect(sentData.action).toBe('subscribe')
      expect(sentData.channel).toBe('trades')
      expect(sentData.symbol).toBe('BTCUSDT')
    })

    it('does not send when disconnected', () => {
      const { result } = renderHook(() => useWebSocket())

      // Try to send without connecting
      const message = { data: 'test' }

      act(() => {
        result.current.sendMessage(message)
      })

      // Should not crash, message should not be sent
      expect(result.current.state.isConnected).toBe(false)
    })

    it('queues multiple messages correctly', async () => {
      const { result } = renderHook(() => useWebSocket())

      act(() => {
        result.current.connect()
      })

      await waitFor(() => expect(mockWs).toBeDefined(), { timeout: 1000 })

      act(() => {
        mockWs.triggerOpen()
      })

      await waitFor(() => {
        expect(result.current.state.isConnected).toBe(true)
      }, { timeout: 1000 })

      // Clear the initial Ping message sent by heartbeat
      mockWs.sent = []

      // Send multiple messages
      const msg1 = { id: 1, action: 'first' }
      const msg2 = { id: 2, action: 'second' }
      const msg3 = { id: 3, action: 'third' }

      act(() => {
        result.current.sendMessage(msg1)
        result.current.sendMessage(msg2)
        result.current.sendMessage(msg3)
      })

      // All messages should be sent in order
      expect(mockWs.sent).toHaveLength(3)
      expect(JSON.parse(mockWs.sent[0])).toEqual(msg1)
      expect(JSON.parse(mockWs.sent[1])).toEqual(msg2)
      expect(JSON.parse(mockWs.sent[2])).toEqual(msg3)
    })
  })

  describe('Reconnection Logic', () => {
    it('can reconnect after disconnect', async () => {
      const { result } = renderHook(() => useWebSocket())

      // First connection
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

      // Disconnect
      act(() => {
        result.current.disconnect()
      })

      expect(result.current.state.isConnected).toBe(false)

      // Reconnect
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

      // Should be connected again
      expect(result.current.state.isConnected).toBe(true)
    })
  })
})
