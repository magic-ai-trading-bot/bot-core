import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { WebSocketProvider, useWebSocketContext } from '../../contexts/WebSocketContext'
import React from 'react'

// Mock useWebSocket hook
const mockWebSocketHook = {
  state: {
    isConnected: false,
    isConnecting: false,
    error: null,
    lastMessage: null,
    aiSignals: [],
    recentTrades: [],
    botStatus: null,
    connectionAttempts: 0,
  },
  connect: vi.fn(),
  disconnect: vi.fn(),
  sendMessage: vi.fn(),
}

vi.mock('../../hooks/useWebSocket', () => ({
  useWebSocket: vi.fn(() => mockWebSocketHook),
}))

describe('WebSocketContext', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('WebSocketProvider', () => {
    it('renders children successfully', () => {
      const TestChild = () => <div>Test Child</div>
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>
          <TestChild />
          {children}
        </WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      // If the hook works, the provider is rendering correctly
      expect(result.current).toBeDefined()
    })

    it('provides WebSocket context to children', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      expect(result.current).toBeDefined()
      expect(result.current.state).toBeDefined()
      expect(result.current.connect).toBeDefined()
      expect(result.current.disconnect).toBeDefined()
      expect(result.current.sendMessage).toBeDefined()
    })

    it('shares single WebSocket instance across multiple consumers', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result: result1 } = renderHook(() => useWebSocketContext(), { wrapper })
      const { result: result2 } = renderHook(() => useWebSocketContext(), { wrapper })

      // Both consumers should get the same WebSocket instance (same functions)
      expect(result1.current.connect).toBe(result2.current.connect)
      expect(result1.current.disconnect).toBe(result2.current.disconnect)
      expect(result1.current.sendMessage).toBe(result2.current.sendMessage)
    })

    it('has displayName set for debugging', () => {
      expect(WebSocketProvider.displayName).toBe('WebSocketProvider')
    })
  })

  describe('useWebSocketContext', () => {
    it('returns WebSocket hook when used within provider', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      expect(result.current).toEqual(mockWebSocketHook)
    })

    it('throws error when used outside provider', () => {
      // Suppress console.error for this test
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      expect(() => {
        renderHook(() => useWebSocketContext())
      }).toThrow('useWebSocketContext must be used within WebSocketProvider')

      consoleErrorSpy.mockRestore()
    })

    it('provides access to WebSocket state', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      expect(result.current.state).toEqual(mockWebSocketHook.state)
      expect(result.current.state.isConnected).toBe(false)
      expect(result.current.state.isConnecting).toBe(false)
      expect(result.current.state.error).toBe(null)
    })

    it('provides access to connect function', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      expect(result.current.connect).toBe(mockWebSocketHook.connect)
      expect(typeof result.current.connect).toBe('function')
    })

    it('provides access to disconnect function', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      expect(result.current.disconnect).toBe(mockWebSocketHook.disconnect)
      expect(typeof result.current.disconnect).toBe('function')
    })

    it('provides access to sendMessage function', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      expect(result.current.sendMessage).toBe(mockWebSocketHook.sendMessage)
      expect(typeof result.current.sendMessage).toBe('function')
    })
  })

  describe('Integration scenarios', () => {
    it('allows multiple components to use same WebSocket connection', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      // Simulate multiple components using the context
      const { result: componentA } = renderHook(() => useWebSocketContext(), { wrapper })
      const { result: componentB } = renderHook(() => useWebSocketContext(), { wrapper })
      const { result: componentC } = renderHook(() => useWebSocketContext(), { wrapper })

      // All should have access to the same WebSocket functions
      expect(componentA.current.connect).toBe(componentB.current.connect)
      expect(componentB.current.connect).toBe(componentC.current.connect)
      expect(componentA.current.state).toEqual(componentB.current.state)
      expect(componentB.current.state).toEqual(componentC.current.state)
    })

    it('provides all WebSocket state properties', () => {
      const wrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>{children}</WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper })

      // Verify all state properties are accessible
      expect(result.current.state).toHaveProperty('isConnected')
      expect(result.current.state).toHaveProperty('isConnecting')
      expect(result.current.state).toHaveProperty('error')
      expect(result.current.state).toHaveProperty('lastMessage')
      expect(result.current.state).toHaveProperty('aiSignals')
      expect(result.current.state).toHaveProperty('recentTrades')
      expect(result.current.state).toHaveProperty('botStatus')
      expect(result.current.state).toHaveProperty('connectionAttempts')
    })

    it('handles nested provider correctly', () => {
      // Nested providers should work (though not recommended in practice)
      const OuterWrapper = ({ children }: { children: React.ReactNode }) => (
        <WebSocketProvider>
          <WebSocketProvider>{children}</WebSocketProvider>
        </WebSocketProvider>
      )

      const { result } = renderHook(() => useWebSocketContext(), { wrapper: OuterWrapper })

      // Should still provide valid context from inner provider
      expect(result.current).toBeDefined()
      expect(result.current.state).toBeDefined()
    })
  })

  describe('Error handling', () => {
    it('throws descriptive error message when not wrapped', () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      expect(() => {
        renderHook(() => useWebSocketContext())
      }).toThrow('useWebSocketContext must be used within WebSocketProvider')

      consoleErrorSpy.mockRestore()
    })

    it('error message guides users to correct usage', () => {
      const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

      try {
        renderHook(() => useWebSocketContext())
      } catch (error) {
        expect((error as Error).message).toContain('WebSocketProvider')
      }

      consoleErrorSpy.mockRestore()
    })
  })
})
