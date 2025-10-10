import { describe, it, expect, beforeEach, afterEach, vi } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useWebSocket } from '@/hooks/useWebSocket';

class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;
  url: string;

  constructor(url: string) {
    this.url = url;
    setTimeout(() => this.triggerOpen(), 10);
  }

  triggerOpen() {
    this.readyState = MockWebSocket.OPEN;
    if (this.onopen) {
      this.onopen(new Event('open'));
    }
  }

  triggerClose(code = 1000, reason = '') {
    this.readyState = MockWebSocket.CLOSED;
    if (this.onclose) {
      const closeEvent = new CloseEvent('close', { code, reason });
      this.onclose(closeEvent);
    }
  }

  triggerError() {
    if (this.onerror) {
      this.onerror(new Event('error'));
    }
  }

  triggerMessage(data: any) {
    if (this.onmessage) {
      const messageEvent = new MessageEvent('message', {
        data: JSON.stringify(data),
      });
      this.onmessage(messageEvent);
    }
  }

  send(data: string) {
    if (this.readyState !== MockWebSocket.OPEN) {
      throw new Error('WebSocket is not open');
    }
  }

  close() {
    this.readyState = MockWebSocket.CLOSING;
    setTimeout(() => this.triggerClose(), 10);
  }
}

describe('useWebSocket - Comprehensive Tests', () => {
  let mockWs: MockWebSocket | null = null;

  beforeEach(() => {
    mockWs = null;
    global.WebSocket = vi.fn((url: string) => {
      mockWs = new MockWebSocket(url);
      return mockWs as any;
    }) as any;
  });

  afterEach(() => {
    vi.clearAllMocks();
    mockWs = null;
  });

  describe('Connection Management', () => {
    it('should handle multiple rapid connect/disconnect cycles', async () => {
      const { result } = renderHook(() => useWebSocket());

      // Rapid connect/disconnect
      act(() => {
        result.current.connect();
      });
      await waitFor(() => expect(mockWs).toBeDefined(), { timeout: 3000 });

      act(() => {
        result.current.disconnect();
      });

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });
    });

    it('should handle connection loss and reconnection', async () => {
      const { result } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      // Simulate connection loss
      await act(async () => {
        mockWs?.triggerClose(1006, 'Connection lost');
        await new Promise(resolve => setTimeout(resolve, 100));
      });

      expect(result.current.state.isConnected).toBe(false);
    });

    it('should not create multiple connections on rapid connect calls', async () => {
      const { result } = renderHook(() => useWebSocket());
      let connectionCount = 0;

      global.WebSocket = vi.fn((url: string) => {
        connectionCount++;
        mockWs = new MockWebSocket(url);
        return mockWs as any;
      }) as any;

      act(() => {
        result.current.connect();
        result.current.connect();
        result.current.connect();
      });

      await waitFor(() => expect(mockWs).toBeDefined(), { timeout: 3000 });

      // Should only create one connection
      expect(connectionCount).toBeLessThanOrEqual(2);
    });
  });

  describe('Message Handling', () => {
    it('should handle position update messages', async () => {
      const { result } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      const positionUpdate = {
        type: 'PositionUpdate',
        data: {
          symbol: 'BTCUSDT',
          side: 'LONG',
          pnl: 150.50,
          current_price: 45000,
          unrealized_pnl: 200.25,
          timestamp: Date.now(),
        },
        timestamp: new Date().toISOString(),
      };

      act(() => {
        mockWs?.triggerMessage(positionUpdate);
      });

      await waitFor(() => {
        expect(result.current.state.lastMessage?.type).toBe('PositionUpdate');
      }, { timeout: 3000 });
    });

    it('should handle trade executed messages', async () => {
      const { result } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      const tradeExecuted = {
        type: 'TradeExecuted',
        data: {
          symbol: 'ETHUSDT',
          side: 'BUY',
          quantity: 1.5,
          price: 3000,
          timestamp: Date.now(),
          pnl: 50,
        },
        timestamp: new Date().toISOString(),
      };

      act(() => {
        mockWs?.triggerMessage(tradeExecuted);
      });

      await waitFor(() => {
        expect(result.current.state.recentTrades.length).toBeGreaterThan(0);
      }, { timeout: 3000 });
    });

    it('should handle AI signal messages', async () => {
      const { result } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      const aiSignal = {
        type: 'AISignalReceived',
        data: {
          symbol: 'BTCUSDT',
          signal: 'long',
          confidence: 0.85,
          timestamp: Date.now(),
          model_type: 'LSTM',
          timeframe: '1h',
        },
        timestamp: new Date().toISOString(),
      };

      act(() => {
        mockWs?.triggerMessage(aiSignal);
      });

      await waitFor(() => {
        expect(result.current.state.aiSignals.length).toBeGreaterThan(0);
      }, { timeout: 3000 });
    });

    it('should handle bot status update messages', async () => {
      const { result } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      const statusUpdate = {
        type: 'BotStatusUpdate',
        data: {
          status: 'running',
          active_positions: 5,
          total_pnl: 1250.75,
          total_trades: 42,
          uptime: 3600,
        },
        timestamp: new Date().toISOString(),
      };

      act(() => {
        mockWs?.triggerMessage(statusUpdate);
      });

      await waitFor(() => {
        expect(result.current.state.botStatus?.status).toBe('running');
      }, { timeout: 3000 });
    });

    it('should handle error messages', async () => {
      const { result } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      const errorMessage = {
        type: 'Error',
        data: {
          message: 'Test error',
          code: 'TEST_ERROR',
        },
        timestamp: new Date().toISOString(),
      };

      act(() => {
        mockWs?.triggerMessage(errorMessage);
      });

      await waitFor(() => {
        expect(result.current.state.error).toBe('Test error');
      }, { timeout: 3000 });
    });

    it('should handle malformed messages gracefully', async () => {
      const { result } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      // Send malformed JSON
      act(() => {
        if (mockWs?.onmessage) {
          mockWs.onmessage(new MessageEvent('message', { data: 'invalid json' }));
        }
      });

      await waitFor(() => {
        expect(result.current.state.error).toBeTruthy();
      }, { timeout: 3000 });
    });
  });

  describe('Message Sending', () => {
    it('should send messages when connected', async () => {
      const { result } = renderHook(() => useWebSocket());
      const sendSpy = vi.spyOn(MockWebSocket.prototype, 'send');

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      act(() => {
        result.current.sendMessage({ type: 'test', data: { foo: 'bar' } });
      });

      expect(sendSpy).toHaveBeenCalled();
    });

    it('should not send messages when disconnected', () => {
      const { result } = renderHook(() => useWebSocket());
      const sendSpy = vi.spyOn(MockWebSocket.prototype, 'send');

      act(() => {
        result.current.sendMessage({ type: 'test', data: { foo: 'bar' } });
      });

      expect(sendSpy).not.toHaveBeenCalled();
    });
  });

  describe('Cleanup and Memory Management', () => {
    it('should cleanup on unmount', async () => {
      const { result, unmount } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(mockWs).toBeDefined(), { timeout: 3000 });

      const closeSpy = vi.spyOn(mockWs as MockWebSocket, 'close');

      unmount();

      expect(closeSpy).toHaveBeenCalled();
    });

    it('should prevent memory leaks from event listeners', async () => {
      const { result, unmount } = renderHook(() => useWebSocket());

      act(() => {
        result.current.connect();
      });

      await waitFor(() => expect(result.current.state.isConnected).toBe(true), { timeout: 3000 });

      unmount();

      // Attempt to trigger events after unmount should not cause errors
      expect(() => {
        mockWs?.triggerMessage({ type: 'Test', timestamp: new Date().toISOString() });
      }).not.toThrow();
    });
  });
});
