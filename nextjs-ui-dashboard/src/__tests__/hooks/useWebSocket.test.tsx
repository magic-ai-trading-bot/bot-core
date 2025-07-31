import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useWebSocket } from '../../hooks/useWebSocket'
import { mockWebSocket } from '../../test/utils'

// Mock WebSocket
const mockWs = mockWebSocket()

global.WebSocket = vi.fn(() => mockWs) as unknown as typeof WebSocket

describe('useWebSocket', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockWs.readyState = 1 // OPEN
  })

  it('initializes WebSocket connection', () => {
    const { result } = renderHook(() => useWebSocket())
    
    expect(result.current.connected).toBe(true)
    expect(global.WebSocket).toHaveBeenCalledWith('ws://localhost:8080/ws')
  })

  it('subscribes to market data', () => {
    const { result } = renderHook(() => useWebSocket())
    
    act(() => {
      result.current.subscribe('prices')
    })
    
    expect(mockWs.send).toHaveBeenCalledWith(
      JSON.stringify({
        type: 'subscribe',
        channel: 'prices'
      })
    )
  })

  it('unsubscribes from market data', () => {
    const { result } = renderHook(() => useWebSocket())
    
    act(() => {
      result.current.unsubscribe('prices')
    })
    
    expect(mockWs.send).toHaveBeenCalledWith(
      JSON.stringify({
        type: 'unsubscribe',
        channel: 'prices'
      })
    )
  })

  it('handles incoming messages', () => {
    const onMessage = vi.fn()
    const { result } = renderHook(() => useWebSocket({ onMessage }))
    
    const messageData = {
      type: 'price_update',
      data: {
        symbol: 'BTCUSDT',
        price: 45000
      }
    }
    
    act(() => {
      mockWs.trigger('message', {
        data: JSON.stringify(messageData)
      })
    })
    
    expect(onMessage).toHaveBeenCalledWith(messageData)
  })

  it('handles connection open', () => {
    const onOpen = vi.fn()
    renderHook(() => useWebSocket({ onOpen }))
    
    act(() => {
      mockWs.trigger('open')
    })
    
    expect(onOpen).toHaveBeenCalled()
  })

  it('handles connection close', () => {
    const onClose = vi.fn()
    const { result } = renderHook(() => useWebSocket({ onClose }))
    
    act(() => {
      mockWs.readyState = 3 // CLOSED
      mockWs.trigger('close')
    })
    
    expect(result.current.connected).toBe(false)
    expect(onClose).toHaveBeenCalled()
  })

  it('handles connection error', () => {
    const onError = vi.fn()
    renderHook(() => useWebSocket({ onError }))
    
    const error = new Error('Connection failed')
    act(() => {
      mockWs.trigger('error', error)
    })
    
    expect(onError).toHaveBeenCalledWith(error)
  })

  it('reconnects on connection loss', async () => {
    const { result } = renderHook(() => useWebSocket({ reconnect: true }))
    
    // Simulate connection loss
    act(() => {
      mockWs.readyState = 3 // CLOSED
      mockWs.trigger('close')
    })
    
    expect(result.current.connected).toBe(false)
    
    // Wait for reconnection attempt
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 1100)) // Wait for reconnect delay
    })
    
    expect(global.WebSocket).toHaveBeenCalledTimes(2)
  })

  it('maintains subscription state across reconnections', async () => {
    const { result } = renderHook(() => useWebSocket({ reconnect: true }))
    
    // Subscribe to channels
    act(() => {
      result.current.subscribe('prices')
      result.current.subscribe('trades')
    })
    
    // Simulate reconnection
    act(() => {
      mockWs.readyState = 3
      mockWs.trigger('close')
    })
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 1100))
      mockWs.readyState = 1
      mockWs.trigger('open')
    })
    
    // Should re-subscribe to all channels
    expect(mockWs.send).toHaveBeenCalledWith(
      JSON.stringify({ type: 'subscribe', channel: 'prices' })
    )
    expect(mockWs.send).toHaveBeenCalledWith(
      JSON.stringify({ type: 'subscribe', channel: 'trades' })
    )
  })

  it('sends custom messages', () => {
    const { result } = renderHook(() => useWebSocket())
    
    const message = { type: 'ping', timestamp: Date.now() }
    
    act(() => {
      result.current.send(message)
    })
    
    expect(mockWs.send).toHaveBeenCalledWith(JSON.stringify(message))
  })

  it('handles malformed JSON messages', () => {
    const onError = vi.fn()
    renderHook(() => useWebSocket({ onError }))
    
    act(() => {
      mockWs.trigger('message', {
        data: 'invalid json{'
      })
    })
    
    expect(onError).toHaveBeenCalledWith(
      expect.objectContaining({
        message: expect.stringContaining('JSON')
      })
    )
  })

  it('queues messages when disconnected', () => {
    const { result } = renderHook(() => useWebSocket())
    
    // Simulate disconnection
    act(() => {
      mockWs.readyState = 3 // CLOSED
    })
    
    const message = { type: 'test', data: 'queued' }
    
    act(() => {
      result.current.send(message)
    })
    
    // Message should not be sent immediately
    expect(mockWs.send).not.toHaveBeenCalledWith(JSON.stringify(message))
  })

  it('sends queued messages on reconnection', async () => {
    const { result } = renderHook(() => useWebSocket({ reconnect: true }))
    
    // Disconnect and queue messages
    act(() => {
      mockWs.readyState = 3
    })
    
    const message1 = { type: 'test1' }
    const message2 = { type: 'test2' }
    
    act(() => {
      result.current.send(message1)
      result.current.send(message2)
    })
    
    // Reconnect
    await act(async () => {
      mockWs.readyState = 1
      mockWs.trigger('open')
    })
    
    expect(mockWs.send).toHaveBeenCalledWith(JSON.stringify(message1))
    expect(mockWs.send).toHaveBeenCalledWith(JSON.stringify(message2))
  })

  it('closes connection on unmount', () => {
    const { unmount } = renderHook(() => useWebSocket())
    
    unmount()
    
    expect(mockWs.close).toHaveBeenCalled()
  })

  it('handles heartbeat/ping-pong', () => {
    renderHook(() => useWebSocket({ heartbeat: true }))
    
    act(() => {
      mockWs.trigger('message', {
        data: JSON.stringify({ type: 'ping' })
      })
    })
    
    expect(mockWs.send).toHaveBeenCalledWith(
      JSON.stringify({ type: 'pong' })
    )
  })
})