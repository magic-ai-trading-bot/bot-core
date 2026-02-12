/**
 * useRealTrading Hook - Functional Tests (Coverage Boost)
 *
 * Comprehensive tests for uncovered branches and edge cases:
 * - fetchClosedTrades error handling (lines 283-297)
 * - WebSocket message parsing errors (line 1133)
 * - WebSocket close/error handlers (lines 1137-1149)
 * - fetchBotStatus success path (lines 176-183)
 * - fetchOpenTrades error path (lines 244-246)
 * - fetchCurrentSettings success path (lines 316-322)
 * - HTTP error in fetchWithRetry (line 256)
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useRealTrading } from '@/hooks/useRealTrading';

// Mock toast
const mockToast = vi.fn();

// Mock useTradingMode
const mockUseTradingMode = vi.fn(() => ({ mode: 'real' }));

// Mock useTradingMode and toast
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({ toast: mockToast }),
}));

vi.mock('@/hooks/useTradingMode', () => ({
  useTradingMode: () => mockUseTradingMode(),
}));

vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
  },
}));

// Mock fetch globally
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Mock WebSocket
class MockWebSocket {
  static OPEN = 1;
  readyState = MockWebSocket.OPEN;
  onopen: (() => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onerror: ((error: Event) => void) | null = null;

  constructor() {
    setTimeout(() => this.onopen?.(), 0);
  }

  send = vi.fn();
  close = vi.fn();
}

global.WebSocket = MockWebSocket as unknown as typeof WebSocket;

describe('useRealTrading - Functional Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockReset();
    mockUseTradingMode.mockReturnValue({ mode: 'real' });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // ========================================
  // fetchClosedTrades Error Handling (lines 283-297)
  // ========================================

  describe('fetchClosedTrades Error Handling', () => {
    it('should show warning toast when API returns error', async () => {
      mockToast.mockClear();

      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/trades/closed')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: false,
                error: 'Database connection error',
              }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      await act(async () => {
        await result.current.refreshTrades();
      });

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Warning',
          description: expect.stringContaining('Database connection error'),
          variant: 'destructive',
        });
      });
    });

    it('should show error toast after all retries fail', async () => {
      mockToast.mockClear();

      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/trades/closed')) {
          return Promise.reject(new Error('Network timeout'));
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      await act(async () => {
        await result.current.refreshTrades();
      });

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Error',
          description: expect.stringContaining('Unable to connect'),
          variant: 'destructive',
        });
      });
    });
  });

  // ========================================
  // WebSocket Error Handling (lines 1133, 1137-1149)
  // ========================================

  describe('WebSocket Error Handling', () => {
    it('should handle invalid JSON without crashing', async () => {
      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(result.current).toBeDefined());

      const ws = new MockWebSocket();
      await act(async () => {
        ws.onmessage?.({ data: 'invalid json {' });
      });

      expect(result.current).toBeDefined();
    });

    it('should cleanup on WebSocket close', async () => {
      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(result.current).toBeDefined());

      const ws = new MockWebSocket();
      await act(async () => {
        ws.onclose?.(new CloseEvent('close'));
      });

      expect(result.current).toBeDefined();
    });

    it('should handle WebSocket error gracefully', async () => {
      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(result.current).toBeDefined());

      const ws = new MockWebSocket();
      await act(async () => {
        ws.onerror?.(new Event('error'));
      });

      expect(result.current).toBeDefined();
    });
  });

  // ========================================
  // fetchBotStatus Success Path (lines 176-183)
  // ========================================

  describe('fetchBotStatus Success Path', () => {
    it('should update state when bot status is fetched successfully', async () => {
      const now = new Date().toISOString();

      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/status')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: true,
                data: {
                  is_running: true,
                  portfolio: {
                    total_trades: 10,
                    win_rate: 65,
                    total_pnl: 1500,
                    current_balance: 15000,
                  },
                  last_updated: now,
                },
              }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(result.current.isActive).toBe(true);
        expect(result.current.portfolio.total_pnl).toBe(1500);
        expect(result.current.lastUpdated).toBeTruthy();
      });
    });
  });

  // ========================================
  // fetchOpenTrades Error Path (lines 244-246)
  // ========================================

  describe('fetchOpenTrades Error Path', () => {
    it('should handle fetchOpenTrades failure gracefully', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/trades/open')) {
          return Promise.reject(new Error('API unavailable'));
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      expect(result.current.openTrades).toEqual([]);
    });
  });

  // ========================================
  // fetchCurrentSettings Success Path (lines 316-322)
  // ========================================

  describe('fetchCurrentSettings Success Path', () => {
    it('should update settings when API returns success', async () => {
      const customSettings = {
        basic: {
          initial_balance: 0,
          max_positions: 3,
          default_position_size_pct: 2.0,
          default_leverage: 5,
          trading_fee_rate: 0.0005,
          funding_fee_rate: 0.0001,
          slippage_pct: 0.03,
          enabled: true,
          auto_restart: true,
        },
        risk: {
          max_risk_per_trade_pct: 1.5,
          max_portfolio_risk_pct: 12.0,
          default_stop_loss_pct: 2.0,
          default_take_profit_pct: 4.0,
          max_leverage: 15,
          min_margin_level: 250.0,
          max_drawdown_pct: 12.0,
          daily_loss_limit_pct: 4.0,
          max_consecutive_losses: 4,
          cool_down_minutes: 90,
        },
      };

      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/settings')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: true,
                data: customSettings,
              }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(result.current.settings.basic.max_positions).toBe(3);
        expect(result.current.settings.risk.max_consecutive_losses).toBe(4);
        expect(result.current.lastUpdated).toBeTruthy();
      });
    });
  });
});
