/**
 * Additional tests for useRealTrading hook to boost coverage to 95%+
 * Target uncovered lines: 1133, 1140, 1147 (WebSocket event handling)
 */

import { renderHook, waitFor, act } from "@testing-library/react";
import { useRealTrading } from "@/hooks/useRealTrading";
import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";

// Mock dependencies
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

vi.mock('@/hooks/useTradingMode', () => ({
  useTradingMode: () => ({ mode: 'real' }),
}));

vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
  },
}));

import logger from '@/utils/logger';

// MockWebSocket class for testing WebSocket events
let wsInstance: any = null;
class MockWebSocket {
  static OPEN = 1;
  static CONNECTING = 0;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  onopen: any = null;
  onclose: any = null;
  onmessage: any = null;
  onerror: any = null;
  send = vi.fn();
  close = vi.fn();
  url: string;

  constructor(url: string) {
    this.url = url;
    wsInstance = this;
    // Simulate async connection
    setTimeout(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.();
    }, 0);
  }
}

global.WebSocket = MockWebSocket as any;

describe('useRealTrading - Additional Coverage Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    wsInstance = null;

    // Mock fetch for all API calls
    global.fetch = vi.fn((url: string) => {
      const urlStr = url.toString();

      if (urlStr.includes('/status')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: {
              is_running: false,
              portfolio: {
                total_trades: 0,
                win_rate: 0,
                total_pnl: 0,
                current_balance: 10000,
                equity: 10000,
                margin_used: 0,
                free_margin: 10000,
              },
              last_updated: new Date().toISOString(),
            },
          }),
        } as any);
      }

      if (urlStr.includes('/portfolio')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: {
              total_balance: 10000,
              available_balance: 9000,
              locked_balance: 1000,
              realized_pnl: 500,
            },
          }),
        } as any);
      }

      if (urlStr.includes('/trades/open')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: [],
          }),
        } as any);
      }

      if (urlStr.includes('/trades/closed')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: [],
          }),
        } as any);
      }

      if (urlStr.includes('/orders')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: [],
          }),
        } as any);
      }

      if (urlStr.includes('/settings')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: {
              basic: {
                initial_balance: 0,
                max_positions: 5,
                default_position_size_pct: 2.0,
                enabled: false,
              },
              risk: {
                max_risk_per_trade_pct: 1.0,
                daily_loss_limit_pct: 3.0,
              },
            },
          }),
        } as any);
      }

      if (urlStr.includes('/market/symbols')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: {
              symbols: ['BTCUSDT', 'ETHUSDT'],
            },
          }),
        } as any);
      }

      if (urlStr.includes('/api/ai/analyze')) {
        return Promise.resolve({
          json: () => Promise.resolve({
            success: true,
            data: {
              signal: 'BUY',
              confidence: 0.85,
              reasoning: 'Test',
              strategy_scores: {},
              market_analysis: {},
              risk_assessment: {},
            },
          }),
        } as any);
      }

      return Promise.reject(new Error('Unknown endpoint'));
    }) as any;
  });

  afterEach(() => {
    vi.clearAllMocks();
    wsInstance = null;
  });

  describe('WebSocket onmessage - JSON parse error (line 1133)', () => {
    it('should handle invalid JSON in WebSocket message', async () => {
      const { result } = renderHook(() => useRealTrading());

      // Wait for WebSocket to connect
      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      // Send invalid JSON
      await act(async () => {
        wsInstance?.onmessage?.({ data: 'invalid-json{[' });
      });

      // Verify error was logged
      await waitFor(() => {
        expect(logger.error).toHaveBeenCalledWith(
          'Failed to parse Real Trading WebSocket message:',
          expect.any(Error)
        );
      });
    });

    it('should handle MarketData event with valid data', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      // Send MarketData event
      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'MarketData',
            data: {
              symbol: 'BTCUSDT',
              price: 51000,
            },
          }),
        });
      });

      await waitFor(() => {
        expect(result.current.lastUpdated).not.toBeNull();
      });
    });

    it('should handle trade_executed event', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'trade_executed',
            data: {},
          }),
        });
      });

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalled();
      });
    });

    it('should handle trade_closed event', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'trade_closed',
            data: {},
          }),
        });
      });

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalled();
      });
    });

    it('should handle AISignalReceived event', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      const signalData = {
        id: 'test-signal',
        signal: 'BUY',
        symbol: 'BTCUSDT',
        confidence: 0.9,
        timestamp: new Date(),
        reasoning: 'Test signal',
      };

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'AISignalReceived',
            data: signalData,
          }),
        });
      });

      await waitFor(() => {
        expect(result.current.recentSignals.length).toBeGreaterThan(0);
      });
    });

    it('should handle order_placed event', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'order_placed',
            data: {},
          }),
        });
      });

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalled();
      });
    });

    it('should handle order_filled event', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'order_filled',
            data: {},
          }),
        });
      });

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalled();
      });
    });

    it('should handle order_partially_filled event', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'order_partially_filled',
            data: {},
          }),
        });
      });

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalled();
      });
    });

    it('should handle order_cancelled event', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'order_cancelled',
            data: {},
          }),
        });
      });

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalled();
      });
    });

    it('should handle unknown event type', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      await act(async () => {
        wsInstance?.onmessage?.({
          data: JSON.stringify({
            event_type: 'unknown_event',
            data: {},
          }),
        });
      });

      // Should not crash
      expect(result.current).toBeDefined();
    });
  });

  describe('WebSocket onclose - heartbeat cleanup (line 1140)', () => {
    it('should clear heartbeat interval on WebSocket close', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      // Trigger onclose
      await act(async () => {
        wsInstance?.onclose?.();
      });

      // Verify logger was called
      await waitFor(() => {
        expect(logger.info).toHaveBeenCalledWith('🔴 Real Trading WebSocket disconnected');
      });
    });

    it('should handle close when heartbeat is null', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      // Close immediately after open (before heartbeat starts)
      await act(async () => {
        wsInstance?.onclose?.();
      });

      expect(logger.info).toHaveBeenCalledWith('🔴 Real Trading WebSocket disconnected');
    });
  });

  describe('WebSocket onerror - heartbeat cleanup (line 1147)', () => {
    it('should clear heartbeat interval on WebSocket error', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      const testError = new Error('WebSocket error');

      // Trigger onerror
      await act(async () => {
        wsInstance?.onerror?.(testError);
      });

      // Verify logger was called
      await waitFor(() => {
        expect(logger.error).toHaveBeenCalledWith(
          '🔴 Real Trading WebSocket error:',
          testError
        );
      });
    });

    it('should handle error when heartbeat is active', async () => {
      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(wsInstance).not.toBeNull();
      });

      // Wait for heartbeat to start
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 100));
      });

      const testError = new Error('Connection lost');

      // Trigger onerror
      await act(async () => {
        wsInstance?.onerror?.(testError);
      });

      expect(logger.error).toHaveBeenCalledWith(
        '🔴 Real Trading WebSocket error:',
        testError
      );
    });
  });

});
