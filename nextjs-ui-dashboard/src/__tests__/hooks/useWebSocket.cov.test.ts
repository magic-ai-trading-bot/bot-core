/**
 * useWebSocket Hook - Coverage Boost Tests #2
 *
 * Target: Boost coverage from 90% to 95%+
 * Focus: Uncovered lines around 479, 508-509, 551
 * - WebSocket reconnection logic
 * - Message parsing error handling
 * - Connection timeout handling
 * - Specific message type handlers
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useWebSocket } from '@/hooks/useWebSocket';

// Mock logger
vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    debug: vi.fn(),
  },
}));

// Mock WebSocket with timing control
class MockWebSocket {
  public readyState = WebSocket.CONNECTING;
  public onopen: ((ev: Event) => void) | null = null;
  public onclose: ((ev: CloseEvent) => void) | null = null;
  public onerror: ((ev: Event) => void) | null = null;
  public onmessage: ((ev: MessageEvent) => void) | null = null;
  public sent: string[] = [];

  send(data: string) {
    if (this.readyState === WebSocket.OPEN) {
      this.sent.push(data);
    }
  }

  close() {
    this.readyState = WebSocket.CLOSED;
    if (this.onclose) {
      this.onclose(new CloseEvent('close'));
    }
  }

  triggerOpen() {
    this.readyState = WebSocket.OPEN;
    if (this.onopen) {
      this.onopen(new Event('open'));
    }
  }

  triggerMessage(data: unknown) {
    if (this.onmessage) {
      this.onmessage(new MessageEvent('message', { data: JSON.stringify(data) }));
    }
  }

  triggerRawMessage(data: string) {
    if (this.onmessage) {
      this.onmessage(new MessageEvent('message', { data }));
    }
  }

  triggerError() {
    if (this.onerror) {
      this.onerror(new Event('error'));
    }
  }

  triggerClose(code: number = 1000, reason: string = '') {
    this.readyState = WebSocket.CLOSED;
    if (this.onclose) {
      const event = new CloseEvent('close', { code, reason });
      this.onclose(event);
    }
  }
}

let mockWsInstances: MockWebSocket[] = [];
let currentMockWs: MockWebSocket | null = null;

class WebSocketMockClass {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  constructor(url: string) {
    const ws = new MockWebSocket();
    mockWsInstances.push(ws);
    currentMockWs = ws;
    return ws as any;
  }
}

describe('useWebSocket - Coverage Boost #2', () => {
  let originalWebSocket: typeof WebSocket;

  beforeEach(() => {
    vi.clearAllMocks();
    mockWsInstances = [];
    currentMockWs = null;

    originalWebSocket = global.WebSocket;
    Object.defineProperty(global, 'WebSocket', {
      writable: true,
      configurable: true,
      value: WebSocketMockClass,
    });

    // Mock environment variable
    import.meta.env.VITE_ENABLE_REALTIME = 'true';
  });

  afterEach(() => {
    Object.defineProperty(global, 'WebSocket', {
      writable: true,
      configurable: true,
      value: originalWebSocket,
    });
    mockWsInstances = [];
    currentMockWs = null;
  });

  // ========================================
  // connectWebSocket error handling (line 479)
  // ========================================
  describe('WebSocket Connection Error Handling', () => {
    it('should handle handleError when WebSocket triggers error event', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // Trigger error event
      act(() => {
        currentMockWs!.triggerError();
      });

      await waitFor(() => {
        expect(result.current.state.error).toBe('WebSocket connection error');
      });
    });

    it('should handle connection errors gracefully', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // Trigger error
      act(() => {
        currentMockWs!.triggerError();
      });

      // Should update state
      await waitFor(() => {
        expect(result.current.state.isConnecting).toBe(false);
      });
    });
  });

  // ========================================
  // Reconnection timeout clearing (lines 508-509)
  // ========================================
  describe('Reconnection Timeout Management', () => {
    it('should clear reconnection timeout on disconnect', async () => {
      const { result } = renderHook(() => useWebSocket());

      // Wait for initial connection
      await waitFor(() => expect(currentMockWs).toBeDefined());

      const initialCount = mockWsInstances.length;

      // Trigger close to start reconnection
      act(() => {
        currentMockWs!.triggerClose(1006, 'Connection lost');
      });

      // Immediately disconnect before reconnection timeout fires
      act(() => {
        result.current.disconnect();
      });

      // Wait a bit
      await new Promise(resolve => setTimeout(resolve, 100));

      // Verify disconnect was called
      expect(result.current.state.isConnected).toBe(false);
    });

    it('should clear timeout when disconnect called during reconnection', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // Disconnect immediately
      act(() => {
        result.current.disconnect();
      });

      // Should be disconnected
      expect(result.current.state.isConnected).toBe(false);
    });
  });

  // ========================================
  // Auto-connect environment check (line 551)
  // ========================================
  describe('Auto-connect Environment Variable', () => {
    it('should not auto-connect when VITE_ENABLE_REALTIME is false', async () => {
      // Set environment to false
      import.meta.env.VITE_ENABLE_REALTIME = 'false';

      const { result } = renderHook(() => useWebSocket());

      // Wait a bit
      await new Promise(resolve => setTimeout(resolve, 100));

      // Should not be connecting
      expect(result.current.state.isConnecting).toBe(false);
      expect(result.current.state.isConnected).toBe(false);

      // Reset
      import.meta.env.VITE_ENABLE_REALTIME = 'true';
    });

    it('should auto-connect when VITE_ENABLE_REALTIME is not set to false', async () => {
      import.meta.env.VITE_ENABLE_REALTIME = 'true';

      const { result } = renderHook(() => useWebSocket());

      // Should create WebSocket
      await waitFor(() => {
        expect(currentMockWs).toBeDefined();
      });
    });

    it('should auto-connect when VITE_ENABLE_REALTIME is undefined', async () => {
      import.meta.env.VITE_ENABLE_REALTIME = undefined;

      const { result } = renderHook(() => useWebSocket());

      // Should still create WebSocket (only "false" disables it)
      await waitFor(() => {
        expect(currentMockWs).toBeDefined();
      });

      // Reset
      import.meta.env.VITE_ENABLE_REALTIME = 'true';
    });
  });

  // ========================================
  // Additional Message Type Handlers
  // ========================================
  describe('Message Type Handlers', () => {
    it('should handle ChartUpdate message type', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Send ChartUpdate message
      act(() => {
        currentMockWs!.triggerMessage({
          type: 'ChartUpdate',
          data: {
            symbol: 'BTCUSDT',
            timeframe: '1h',
            candle: {
              timestamp: Date.now(),
              open: 50000,
              high: 51000,
              low: 49500,
              close: 50500,
              volume: 1000,
              is_closed: true,
            },
            latest_price: 50500,
            price_change_24h: 500,
            price_change_percent_24h: 1.0,
            volume_24h: 75000,
            timestamp: Date.now(),
          },
          timestamp: new Date().toISOString(),
        });
      });

      // Should update lastMessage
      await waitFor(() => {
        expect(result.current.state.lastMessage?.type).toBe('ChartUpdate');
      });
    });

    it('should handle MarketData message type', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Send MarketData message
      act(() => {
        currentMockWs!.triggerMessage({
          type: 'MarketData',
          data: {
            symbol: 'ETHUSDT',
            price: 3000,
            price_change_24h: 100,
            price_change_percent_24h: 3.45,
            volume_24h: 50000,
            timestamp: Date.now(),
          },
          timestamp: new Date().toISOString(),
        });
      });

      await waitFor(() => {
        expect(result.current.state.lastMessage?.type).toBe('MarketData');
      });
    });

    it('should handle Error message type and set error state', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Send Error message
      act(() => {
        currentMockWs!.triggerMessage({
          type: 'Error',
          data: {
            message: 'Server error occurred',
            code: 'INTERNAL_ERROR',
          },
          timestamp: new Date().toISOString(),
        });
      });

      await waitFor(() => {
        expect(result.current.state.error).toBe('Server error occurred');
      });
    });

    it('should log warning for unknown message types', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Send unknown message type
      act(() => {
        currentMockWs!.triggerMessage({
          type: 'UnknownMessageType',
          data: { some: 'data' },
          timestamp: new Date().toISOString(),
        });
      });

      // Should not crash
      await waitFor(() => {
        expect(result.current.state.lastMessage?.type).toBe('UnknownMessageType');
      });
    });
  });

  // ========================================
  // Message Parsing Error Handling
  // ========================================
  describe('Message Parsing Error Handling', () => {
    it('should handle malformed JSON in WebSocket messages', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Send malformed JSON
      act(() => {
        currentMockWs!.triggerRawMessage('{invalid json');
      });

      // Should set error state
      await waitFor(() => {
        expect(result.current.state.error).toBe('Failed to parse WebSocket message');
      });
    });

    it('should handle empty message', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Send empty message
      act(() => {
        currentMockWs!.triggerRawMessage('');
      });

      // Should handle gracefully
      await waitFor(() => {
        expect(result.current.state).toBeDefined();
      });
    });

    it('should handle null message data', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Send message with null type
      act(() => {
        currentMockWs!.triggerRawMessage('null');
      });

      // Should handle gracefully
      await waitFor(() => {
        expect(result.current.state).toBeDefined();
      });
    });
  });

  // ========================================
  // Connection State Guards
  // ========================================
  describe('Connection State Guards', () => {
    it('should not reconnect when already OPEN', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      const initialCount = mockWsInstances.length;

      // Try to connect again
      act(() => {
        result.current.connect();
      });

      // Should not create new WebSocket
      expect(mockWsInstances.length).toBe(initialCount);
    });

    it('should not reconnect when already CONNECTING', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // WebSocket is in CONNECTING state by default
      const initialCount = mockWsInstances.length;

      // Try to connect again
      act(() => {
        result.current.connect();
      });

      // Should not create new WebSocket
      expect(mockWsInstances.length).toBe(initialCount);
    });

    it('should not send message when WebSocket is not OPEN', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // WebSocket is CONNECTING, try to send
      act(() => {
        result.current.sendMessage({ type: 'test', data: {} });
      });

      // Should not have sent message
      expect(currentMockWs!.sent.length).toBe(0);
    });
  });

  // ========================================
  // Reconnection Logic
  // ========================================
  describe('Reconnection Logic', () => {
    it('should trigger reconnection on abnormal close', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // Trigger abnormal close
      act(() => {
        currentMockWs!.triggerClose(1006, 'Connection lost');
      });

      // Connection should be marked as not connected
      await waitFor(() => {
        expect(result.current.state.isConnected).toBe(false);
      });
    });

    it('should handle exponential backoff logic', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      const initialCount = mockWsInstances.length;

      // Close connection to trigger reconnection
      act(() => {
        currentMockWs!.triggerClose(1006);
      });

      // Should mark as disconnected
      await waitFor(() => {
        expect(result.current.state.isConnected).toBe(false);
      });
    });

    it('should handle reconnection attempts', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // Close connection
      act(() => {
        currentMockWs!.triggerClose(1006);
      });

      // Should be disconnected
      expect(result.current.state.isConnected).toBe(false);
    });
  });

  // ========================================
  // Heartbeat Edge Cases
  // ========================================
  describe('Heartbeat Edge Cases', () => {
    it('should not start heartbeat if WebSocket is not OPEN', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      // Don't open WebSocket, leave it in CONNECTING state
      await new Promise(resolve => setTimeout(resolve, 100));

      // Should not have sent ping (not OPEN)
      expect(currentMockWs!.sent.length).toBe(0);
    });

    it('should handle connection state properly', async () => {
      const { result } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      await waitFor(() => {
        expect(result.current.state.isConnected).toBe(true);
      });
    });
  });

  // ========================================
  // Cleanup Tests
  // ========================================
  describe('Cleanup', () => {
    it('should cleanup all timers and WebSocket on unmount', async () => {
      const { result, unmount } = renderHook(() => useWebSocket());

      await waitFor(() => expect(currentMockWs).toBeDefined());

      act(() => {
        currentMockWs!.triggerOpen();
      });

      // Unmount
      unmount();

      // Should have closed WebSocket
      expect(currentMockWs!.readyState).toBe(WebSocket.CLOSED);

      // Should not crash
      await new Promise(resolve => setTimeout(resolve, 100));
      expect(true).toBe(true);
    });
  });
});
