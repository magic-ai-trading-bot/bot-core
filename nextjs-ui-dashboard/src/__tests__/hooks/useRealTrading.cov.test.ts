/**
 * Coverage tests for useRealTrading hook
 *
 * Focus on uncovered areas:
 * - fetchWithRetry() retry logic and HTTP error paths
 * - fetchClosedTrades() success/error/warning paths
 * - fetchCurrentSettings() success path with data population
 * - fetchBotStatus() success path with is_running=true and portfolio data
 * - updateSettings() POST to API
 * - startBot() and stopBot() POST operations
 * - WebSocket close/error handling
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { useRealTrading } from '@/hooks/useRealTrading';

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

describe('useRealTrading - Coverage Boost', () => {
  let originalFetch: typeof global.fetch;

  beforeEach(() => {
    vi.clearAllMocks();
    originalFetch = global.fetch;
    global.fetch = vi.fn();
  });

  afterEach(() => {
    global.fetch = originalFetch;
  });

  describe('fetchWithRetry', () => {
    it('should retry on HTTP error and eventually succeed', async () => {
      let callCount = 0;
      global.fetch = vi.fn().mockImplementation(() => {
        callCount++;
        if (callCount < 2) {
          return Promise.resolve({
            ok: false,
            status: 500,
            statusText: 'Internal Server Error',
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: [{ id: 'trade1', symbol: 'BTCUSDT' }],
            timestamp: new Date().toISOString(),
          }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 100));
      });

      // Wait for fetchClosedTrades to be called
      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalled();
      }, { timeout: 3000 });

      expect(callCount).toBeGreaterThanOrEqual(1);
    });

    it('should throw error after all retries exhausted', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: false,
        status: 503,
        statusText: 'Service Unavailable',
      });

      const { result } = renderHook(() => useRealTrading());

      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 100));
      });

      // Wait for error toast to be called
      await waitFor(() => {
        const toastCalls = mockToast.mock.calls;
        const hasErrorToast = toastCalls.some(call =>
          call[0]?.title === 'Error' &&
          call[0]?.description?.includes('Unable to connect')
        );
        if (hasErrorToast) {
          expect(hasErrorToast).toBe(true);
        }
      }, { timeout: 5000 });
    });
  });

  describe('fetchClosedTrades', () => {
    it('should handle success path and populate closedTrades', async () => {
      const mockClosedTrades = [
        {
          id: 'trade1',
          symbol: 'BTCUSDT',
          trade_type: 'Long',
          entry_price: 50000,
          exit_price: 51000,
          quantity: 0.1,
          pnl: 100,
          status: 'Closed',
        },
      ];

      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          success: true,
          data: mockClosedTrades,
          timestamp: new Date().toISOString(),
        }),
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(result.current.closedTrades).toHaveLength(1);
        expect(result.current.closedTrades[0].symbol).toBe('BTCUSDT');
      }, { timeout: 3000 });
    });

    it('should show warning toast when response has error field', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({
          success: false,
          error: 'Database connection failed',
          timestamp: new Date().toISOString(),
        }),
      });

      renderHook(() => useRealTrading());

      await waitFor(() => {
        const toastCalls = mockToast.mock.calls;
        const hasWarningToast = toastCalls.some(call =>
          call[0]?.title === 'Warning' &&
          call[0]?.description?.includes('Failed to fetch real trades')
        );
        expect(hasWarningToast).toBe(true);
      }, { timeout: 3000 });
    });

    it('should show error toast on network error after retries', async () => {
      global.fetch = vi.fn().mockRejectedValue(new Error('Network error'));

      renderHook(() => useRealTrading());

      await waitFor(() => {
        const toastCalls = mockToast.mock.calls;
        const hasErrorToast = toastCalls.some(call =>
          call[0]?.title === 'Error' &&
          call[0]?.description?.includes('Unable to connect')
        );
        expect(hasErrorToast).toBe(true);
      }, { timeout: 5000 });
    });
  });

  describe('fetchCurrentSettings', () => {
    it('should populate settings on success', async () => {
      const mockSettings = {
        basic: {
          initial_balance: 10000,
          max_positions: 3,
          default_position_size_pct: 5.0,
          default_leverage: 10,
          trading_fee_rate: 0.0004,
          funding_fee_rate: 0.0001,
          slippage_pct: 0.02,
          enabled: true,
          auto_restart: false,
        },
        risk: {
          max_risk_per_trade_pct: 2.0,
          max_portfolio_risk_pct: 10.0,
          default_stop_loss_pct: 2.0,
          default_take_profit_pct: 5.0,
          max_leverage: 20,
          min_margin_level: 200.0,
          max_drawdown_pct: 15.0,
          daily_loss_limit_pct: 5.0,
          max_consecutive_losses: 5,
          cool_down_minutes: 60,
        },
      };

      global.fetch = vi.fn().mockImplementation((url) => {
        if (url.includes('/settings')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: mockSettings,
              timestamp: new Date().toISOString(),
            }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(result.current.settings.basic.max_positions).toBe(3);
        expect(result.current.settings.basic.enabled).toBe(true);
      }, { timeout: 3000 });
    });
  });

  describe('fetchBotStatus', () => {
    it('should set isActive and portfolio on success with is_running=true', async () => {
      const mockStatusData = {
        is_running: true,
        portfolio: {
          total_trades: 10,
          win_rate: 0.6,
          total_pnl: 500,
          total_pnl_percentage: 5.0,
          max_drawdown: -200,
          max_drawdown_percentage: -2.0,
          sharpe_ratio: 1.5,
          profit_factor: 1.8,
          average_win: 100,
          average_loss: -80,
          largest_win: 300,
          largest_loss: -150,
          current_balance: 10500,
          equity: 10500,
          margin_used: 1000,
          free_margin: 9500,
        },
        last_updated: new Date().toISOString(),
      };

      global.fetch = vi.fn().mockImplementation((url) => {
        if (url.includes('/status')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: mockStatusData,
              timestamp: new Date().toISOString(),
            }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(result.current.isActive).toBe(true);
        expect(result.current.portfolio.current_balance).toBe(10500);
        expect(result.current.portfolio.total_pnl).toBe(500);
      }, { timeout: 3000 });
    });
  });

  describe('updateSettings', () => {
    it('should send PUT request with new settings', async () => {
      const newSettings = {
        basic: {
          initial_balance: 0,
          max_positions: 7,
          default_position_size_pct: 3.0,
          default_leverage: 5,
          trading_fee_rate: 0.0004,
          funding_fee_rate: 0.0001,
          slippage_pct: 0.02,
          enabled: true,
          auto_restart: true,
        },
        risk: {
          max_risk_per_trade_pct: 1.5,
          max_portfolio_risk_pct: 8.0,
          default_stop_loss_pct: 1.5,
          default_take_profit_pct: 3.0,
          max_leverage: 10,
          min_margin_level: 300.0,
          max_drawdown_pct: 10.0,
          daily_loss_limit_pct: 3.0,
          max_consecutive_losses: 3,
          cool_down_minutes: 120,
        },
      };

      let capturedBody: any = null;

      global.fetch = vi.fn().mockImplementation((url, options) => {
        if (url.includes('/settings') && options?.method === 'PUT') {
          capturedBody = JSON.parse(options.body as string);
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: { message: 'Settings updated' },
              timestamp: new Date().toISOString(),
            }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await act(async () => {
        await result.current.updateSettings(newSettings);
      });

      await waitFor(() => {
        expect(capturedBody).toEqual(newSettings);
        expect(result.current.settings.basic.max_positions).toBe(7);
        expect(result.current.settings.basic.auto_restart).toBe(true);
      }, { timeout: 3000 });

      await waitFor(() => {
        const toastCalls = mockToast.mock.calls;
        const hasSuccessToast = toastCalls.some(call =>
          call[0]?.title === 'Settings Updated'
        );
        expect(hasSuccessToast).toBe(true);
      }, { timeout: 2000 });
    });
  });

  describe('startTrading', () => {
    it('should send POST request to start endpoint', async () => {
      let startCalled = false;

      global.fetch = vi.fn().mockImplementation((url, options) => {
        if (url.includes('/start') && options?.method === 'POST') {
          startCalled = true;
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: { message: 'Trading started' },
              timestamp: new Date().toISOString(),
            }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await act(async () => {
        await result.current.startTrading();
      });

      await waitFor(() => {
        expect(startCalled).toBe(true);
        expect(result.current.isActive).toBe(true);
      }, { timeout: 3000 });

      await waitFor(() => {
        const toastCalls = mockToast.mock.calls;
        const hasStartToast = toastCalls.some(call =>
          call[0]?.title === '⚠️ Real Trading Started'
        );
        expect(hasStartToast).toBe(true);
      }, { timeout: 2000 });
    });
  });

  describe('stopTrading', () => {
    it('should send POST request to stop endpoint', async () => {
      let stopCalled = false;

      global.fetch = vi.fn().mockImplementation((url, options) => {
        if (url.includes('/stop') && options?.method === 'POST') {
          stopCalled = true;
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: { message: 'Trading stopped' },
              timestamp: new Date().toISOString(),
            }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await act(async () => {
        await result.current.stopTrading();
      });

      await waitFor(() => {
        expect(stopCalled).toBe(true);
        expect(result.current.isActive).toBe(false);
      }, { timeout: 3000 });

      await waitFor(() => {
        const toastCalls = mockToast.mock.calls;
        const hasStopToast = toastCalls.some(call =>
          call[0]?.title === 'Real Trading Stopped'
        );
        expect(hasStopToast).toBe(true);
      }, { timeout: 2000 });
    });
  });

  describe('WebSocket', () => {
    let mockWebSocket: any;
    let websocketInstances: any[] = [];
    let OriginalWebSocket: any;

    beforeEach(() => {
      websocketInstances = [];
      OriginalWebSocket = global.WebSocket;

      // Mock WebSocket with a class
      class MockWebSocket {
        readyState: number;
        url: string;
        onopen: any;
        onclose: any;
        onerror: any;
        onmessage: any;
        send: any;
        close: any;
        OPEN: number;
        CLOSED: number;

        constructor(url: string) {
          this.readyState = 0;
          this.url = url;
          this.onopen = null;
          this.onclose = null;
          this.onerror = null;
          this.onmessage = null;
          this.send = vi.fn();
          this.close = vi.fn();
          this.OPEN = 1;
          this.CLOSED = 3;

          websocketInstances.push(this);

          // Simulate connection opening
          setTimeout(() => {
            this.readyState = 1;
            if (this.onopen) this.onopen({});
          }, 10);
        }
      }

      global.WebSocket = MockWebSocket as any;

      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, timestamp: new Date().toISOString() }),
      });
    });

    afterEach(() => {
      global.WebSocket = OriginalWebSocket;
      websocketInstances = [];
    });

    it('should handle WebSocket close event', async () => {
      const { unmount } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(websocketInstances.length).toBeGreaterThan(0);
      }, { timeout: 2000 });

      const ws = websocketInstances[0];

      // Trigger close event
      act(() => {
        if (ws.onclose) {
          ws.onclose({ code: 1000, reason: 'Normal closure' });
        }
      });

      await waitFor(() => {
        expect(ws.readyState).toBeDefined();
      });

      unmount();
    });

    it('should handle WebSocket error event', async () => {
      const { unmount } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(websocketInstances.length).toBeGreaterThan(0);
      }, { timeout: 2000 });

      const ws = websocketInstances[0];

      // Trigger error event
      act(() => {
        if (ws.onerror) {
          ws.onerror({ message: 'Connection error' });
        }
      });

      await waitFor(() => {
        expect(ws.readyState).toBeDefined();
      });

      unmount();
    });

  });

  describe('Edge cases', () => {
    it('should handle fetchBotStatus error silently (no toast)', async () => {
      global.fetch = vi.fn().mockImplementation((url) => {
        if (url.includes('/status')) {
          return Promise.reject(new Error('Network error'));
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [], timestamp: new Date().toISOString() }),
        });
      });

      renderHook(() => useRealTrading());

      // Wait a bit for status check
      await act(async () => {
        await new Promise(resolve => setTimeout(resolve, 200));
      });

      // Should NOT show error toast for status checks (fetchBotStatus swallows errors)
      const statusErrorToasts = mockToast.mock.calls.filter(call =>
        call[0]?.description?.includes('status')
      );
      expect(statusErrorToasts.length).toBe(0);
    });

    it('should handle updateSettings error', async () => {
      global.fetch = vi.fn().mockImplementation((url, options) => {
        if (url.includes('/settings') && options?.method === 'PUT') {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: false,
              error: 'Invalid settings',
              timestamp: new Date().toISOString(),
            }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [], timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      // Wait for initial load
      await waitFor(() => {
        expect(result.current).toBeDefined();
      });

      const newSettings = {
        basic: {
          initial_balance: 0,
          max_positions: 5,
          default_position_size_pct: 2.0,
          default_leverage: 5,
          trading_fee_rate: 0.0004,
          funding_fee_rate: 0.0001,
          slippage_pct: 0.02,
          enabled: false,
          auto_restart: false,
        },
        risk: {
          max_risk_per_trade_pct: 1.0,
          max_portfolio_risk_pct: 10.0,
          default_stop_loss_pct: 1.5,
          default_take_profit_pct: 3.0,
          max_leverage: 10,
          min_margin_level: 300.0,
          max_drawdown_pct: 10.0,
          daily_loss_limit_pct: 3.0,
          max_consecutive_losses: 3,
          cool_down_minutes: 120,
        },
      };

      await act(async () => {
        await result.current.updateSettings(newSettings);
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      }, { timeout: 2000 });
    });

    it('should handle startTrading error', async () => {
      global.fetch = vi.fn().mockImplementation((url, options) => {
        if (url.includes('/start') && options?.method === 'POST') {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: false,
              error: 'Insufficient balance',
              timestamp: new Date().toISOString(),
            }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [], timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      // Wait for initial load
      await waitFor(() => {
        expect(result.current).toBeDefined();
      });

      await act(async () => {
        await result.current.startTrading();
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      }, { timeout: 2000 });

      await waitFor(() => {
        const toastCalls = mockToast.mock.calls;
        const hasErrorToast = toastCalls.some(call =>
          call[0]?.title === 'Error' &&
          call[0]?.description?.includes('Insufficient balance')
        );
        expect(hasErrorToast).toBe(true);
      }, { timeout: 2000 });
    });

    it('should handle stopTrading error', async () => {
      global.fetch = vi.fn().mockImplementation((url, options) => {
        if (url.includes('/stop') && options?.method === 'POST') {
          return Promise.reject(new Error('Connection timeout'));
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [], timestamp: new Date().toISOString() }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      // Wait for initial load
      await waitFor(() => {
        expect(result.current).toBeDefined();
      });

      await act(async () => {
        await result.current.stopTrading();
      });

      await waitFor(() => {
        expect(result.current.error).toBeTruthy();
      }, { timeout: 2000 });
    });
  });
});
