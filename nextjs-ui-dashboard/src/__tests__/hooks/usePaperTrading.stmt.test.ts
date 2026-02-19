/**
 * Statement Coverage Tests for usePaperTrading.ts
 *
 * Target uncovered lines: 240-244, 283, 286, 497-498, 565, 764, 907-908,
 *                          934-968, 988-991
 *
 * Goal: Cover ~15-20 of these statements with targeted tests
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';

// Create a stable toast function
const mockToastFn = vi.fn();

// Mock dependencies FIRST (hoisted to top)
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: mockToastFn,
  }),
}));

vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
  },
}));

// Import AFTER mocks
import { renderHook, act, waitFor } from '@testing-library/react';
import { usePaperTrading } from '@/hooks/usePaperTrading';
import logger from '@/utils/logger';

const mockLogger = logger;

// Mock WebSocket
let wsInstances: any[] = [];

class MockWebSocket {
  static OPEN = 1;
  static CLOSED = 3;
  readyState = MockWebSocket.OPEN;
  onopen: ((ev: any) => void) | null = null;
  onclose: ((ev: any) => void) | null = null;
  onmessage: ((ev: any) => void) | null = null;
  onerror: ((ev: any) => void) | null = null;
  url: string;

  constructor(url: string) {
    this.url = url;
    wsInstances.push(this);
    // Simulate connection after microtask
    setTimeout(() => {
      if (this.onopen) this.onopen({} as any);
    }, 10);
  }

  send = vi.fn();
  close = vi.fn();
}

global.WebSocket = MockWebSocket as any;

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

const delay = (ms: number) => new Promise(resolve => setTimeout(resolve, ms));

describe('usePaperTrading - Statement Coverage', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    wsInstances = [];

    // Default mock responses for initialization
    mockFetch.mockImplementation((url: string) => {
      if (url.includes('/status')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: {
              is_running: true,
              portfolio: {
                total_trades: 0,
                win_rate: 0,
                total_pnl: 0,
                total_pnl_percentage: 0,
                max_drawdown: 0,
                max_drawdown_percentage: 0,
                sharpe_ratio: 0,
                profit_factor: 0,
                average_win: 0,
                average_loss: 0,
                largest_win: 0,
                largest_loss: 0,
                current_balance: 10000,
                equity: 10000,
                margin_used: 0,
                free_margin: 10000,
              },
              last_updated: new Date().toISOString(),
            },
            timestamp: new Date().toISOString(),
          }),
        });
      }

      if (url.includes('/trades/open')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: [],
            timestamp: new Date().toISOString(),
          }),
        });
      }

      if (url.includes('/trades/closed')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: [],
            timestamp: new Date().toISOString(),
          }),
        });
      }

      if (url.includes('/pending-orders')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: [],
            timestamp: new Date().toISOString(),
          }),
        });
      }

      if (url.includes('/basic-settings')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: {
              basic: {
                initial_balance: 10000,
                max_positions: 10,
                default_position_size_pct: 5.0,
                default_leverage: 10,
                trading_fee_rate: 0.0004,
                funding_fee_rate: 0.0001,
                slippage_pct: 0.01,
                enabled: true,
                auto_restart: false,
              },
              risk: {
                max_risk_per_trade_pct: 2.0,
                max_portfolio_risk_pct: 20.0,
                default_stop_loss_pct: 2.0,
                default_take_profit_pct: 4.0,
                max_leverage: 50,
                min_margin_level: 200.0,
                max_drawdown_pct: 15.0,
                daily_loss_limit_pct: 5.0,
                max_consecutive_losses: 5,
                cool_down_minutes: 60,
              },
            },
            timestamp: new Date().toISOString(),
          }),
        });
      }

      if (url.includes('/market/symbols')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: { symbols: ['BTCUSDT', 'ETHUSDT'] },
            timestamp: new Date().toISOString(),
          }),
        });
      }

      if (url.includes('/ai/analyze')) {
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({
            success: true,
            data: {
              signal: 'buy',
              confidence: 0.75,
              reasoning: 'Test signal',
              strategy_scores: { rsi: 0.8 },
              market_analysis: {
                trend_direction: 'up',
                trend_strength: 0.7,
                support_levels: [50000],
                resistance_levels: [52000],
                volatility_level: 'medium',
                volume_analysis: 'increasing',
              },
              risk_assessment: {
                overall_risk: 'medium',
                technical_risk: 0.5,
                market_risk: 0.4,
                recommended_position_size: 0.05,
                stop_loss_suggestion: 49000,
                take_profit_suggestion: 53000,
              },
            },
            timestamp: new Date().toISOString(),
          }),
        });
      }

      return Promise.resolve({
        ok: true,
        json: () => Promise.resolve({ success: true, data: null }),
      });
    });
  });

  afterEach(() => {
    // Clean up any WebSocket instances
    wsInstances.forEach((ws) => {
      if (ws.readyState === MockWebSocket.OPEN) {
        ws.close();
      }
    });
    wsInstances = [];
  });

  describe('fetchPortfolioStatus error paths', () => {
    it('should handle failed portfolio fetch (line 283)', async () => {
      // Mock portfolio endpoint to return error response
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/portfolio')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: false,
              error: 'Database connection failed',
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/status')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: { is_running: false, portfolio: {}, last_updated: new Date().toISOString() },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => usePaperTrading());

      // Wait a bit for initialization
      await delay(100);

      // Call refreshData which triggers fetchPortfolioStatus
      await act(async () => {
        await result.current.refreshData();
      });

      // Verify logger.error was called with the error (line 283)
      await waitFor(() => {
        expect(mockLogger.error).toHaveBeenCalledWith(
          'Failed to fetch portfolio status:',
          'Database connection failed'
        );
      });
    });

    it('should handle network error in portfolio fetch (line 286)', async () => {
      // Mock portfolio endpoint to throw network error
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/portfolio')) {
          return Promise.reject(new Error('Network error'));
        }

        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => usePaperTrading());

      await delay(100);

      // Call refreshData which triggers fetchPortfolioStatus
      await act(async () => {
        await result.current.refreshData();
      });

      // Verify logger.error was called in catch block (line 286)
      await waitFor(() => {
        expect(mockLogger.error).toHaveBeenCalledWith(
          'Failed to fetch portfolio status:',
          expect.any(Error)
        );
      });
    });
  });

  describe('fetchAISignals with symbols (lines 497-498)', () => {
    it('should fetch symbols from API and log count (lines 497-498)', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/market/symbols')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: { symbols: ['BTCUSDT', 'ETHUSDT', 'BNBUSDT'] },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/ai/analyze')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: {
                signal: 'buy',
                confidence: 0.75,
                reasoning: 'Test signal',
                strategy_scores: { rsi: 0.8 },
                market_analysis: {
                  trend_direction: 'up',
                  trend_strength: 0.7,
                  support_levels: [50000],
                  resistance_levels: [52000],
                  volatility_level: 'medium',
                  volume_analysis: 'increasing',
                },
                risk_assessment: {
                  overall_risk: 'medium',
                  technical_risk: 0.5,
                  market_risk: 0.4,
                  recommended_position_size: 0.05,
                  stop_loss_suggestion: 49000,
                  take_profit_suggestion: 53000,
                },
              },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => usePaperTrading());

      await delay(100);

      // Call refreshAISignals which triggers fetchAISignals
      await act(async () => {
        await result.current.refreshAISignals();
      });

      // Verify logger.info was called with symbol count (line 498)
      await waitFor(() => {
        expect(mockLogger.info).toHaveBeenCalledWith(
          expect.stringContaining('Fetched 3 symbols from API:'),
          ['BTCUSDT', 'ETHUSDT', 'BNBUSDT']
        );
      });
    });
  });

  describe('AI signal processing null return (line 565)', () => {
    it('should return null for invalid AI signal response (line 565)', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/market/symbols')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: { symbols: ['BTCUSDT'] },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/ai/analyze')) {
          // Return failed response
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: false,
              error: 'Analysis failed',
              timestamp: new Date().toISOString(),
            }),
          });
        }

        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => usePaperTrading());

      await delay(100);

      const initialSignalsCount = result.current.recentSignals.length;

      // Call refreshAISignals which triggers fetchAISignals
      await act(async () => {
        await result.current.refreshAISignals();
      });

      await delay(100);

      // The null signal should be filtered out, count should not increase
      expect(result.current.recentSignals.length).toBe(initialSignalsCount);
    });
  });

  describe('updateSettings database warning (line 764)', () => {
    it('should show toast warning when database save fails (line 764)', async () => {
      mockFetch.mockImplementation((url: string, options?: any) => {
        // Check for PUT method in options
        if (url.includes('/basic-settings') && options?.method === 'PUT') {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: {
                message: 'Settings updated in memory',
                database_saved: false,
                warning: 'MongoDB not available - settings saved in memory only',
              },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/portfolio')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: {
                total_trades: 0,
                win_rate: 0,
                total_pnl: 0,
                total_pnl_percentage: 0,
                max_drawdown: 0,
                max_drawdown_percentage: 0,
                sharpe_ratio: 0,
                profit_factor: 0,
                average_win: 0,
                average_loss: 0,
                largest_win: 0,
                largest_loss: 0,
                current_balance: 10000,
                equity: 10000,
                margin_used: 0,
                free_margin: 10000,
              },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => usePaperTrading());

      await delay(100);

      // Call updateSettings
      await act(async () => {
        await result.current.updateSettings({
          basic: {
            initial_balance: 20000,
            max_positions: 10,
            default_position_size_pct: 5.0,
            default_leverage: 10,
            trading_fee_rate: 0.0004,
            funding_fee_rate: 0.0001,
            slippage_pct: 0.01,
            enabled: true,
            auto_restart: false,
          },
          risk: {
            max_risk_per_trade_pct: 2.0,
            max_portfolio_risk_pct: 20.0,
            default_stop_loss_pct: 2.0,
            default_take_profit_pct: 4.0,
            max_leverage: 50,
            min_margin_level: 200.0,
            max_drawdown_pct: 15.0,
            daily_loss_limit_pct: 5.0,
            max_consecutive_losses: 5,
            cool_down_minutes: 60,
          },
        });
      });

      // Verify toast was called with warning (line 764)
      await waitFor(() => {
        expect(mockToastFn).toHaveBeenCalledWith({
          title: '⚠️ Warning',
          description: 'MongoDB not available - settings saved in memory only',
          variant: 'destructive',
        });
      });
    });
  });

  describe('WebSocket heartbeat (lines 907-908)', () => {
    it('should send heartbeat when WebSocket is open (lines 907-908)', async () => {
      vi.useFakeTimers();

      const { result } = renderHook(() => usePaperTrading());

      // Wait for WebSocket connection
      await act(async () => {
        await vi.advanceTimersByTimeAsync(100);
      });

      // Get the WebSocket instance
      expect(wsInstances.length).toBeGreaterThan(0);
      const ws = wsInstances[0];

      // Clear previous send calls
      ws.send.mockClear();

      // Advance timer to trigger heartbeat (30 seconds)
      await act(async () => {
        await vi.advanceTimersByTimeAsync(30000);
      });

      // Verify heartbeat was sent (line 908)
      expect(ws.send).toHaveBeenCalledWith(
        JSON.stringify({ type: 'ping' })
      );

      vi.useRealTimers();
    });
  });

  describe('WebSocket MarketData handler with open trades (lines 934-968)', () => {
    it('should calculate unrealized PnL for Long trades (lines 936-955)', async () => {
      // Mock open trades endpoint to return trades
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/trades/open')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: [
                {
                  id: 'trade-1',
                  symbol: 'BTCUSDT',
                  trade_type: 'Long',
                  status: 'Open',
                  entry_price: 50000,
                  quantity: 0.1,
                  leverage: 10,
                  pnl: 0,
                  pnl_percentage: 0,
                  open_time: new Date().toISOString(),
                },
              ],
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/status')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: {
                is_running: true,
                portfolio: {
                  total_trades: 1,
                  win_rate: 0,
                  total_pnl: 0,
                  total_pnl_percentage: 0,
                  max_drawdown: 0,
                  max_drawdown_percentage: 0,
                  sharpe_ratio: 0,
                  profit_factor: 0,
                  average_win: 0,
                  average_loss: 0,
                  largest_win: 0,
                  largest_loss: 0,
                  current_balance: 10000,
                  equity: 10000,
                  margin_used: 500,
                  free_margin: 9500,
                },
                last_updated: new Date().toISOString(),
              },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/portfolio')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: {
                total_trades: 1,
                win_rate: 0,
                total_pnl: 0,
                total_pnl_percentage: 0,
                max_drawdown: 0,
                max_drawdown_percentage: 0,
                sharpe_ratio: 0,
                profit_factor: 0,
                average_win: 0,
                average_loss: 0,
                largest_win: 0,
                largest_loss: 0,
                current_balance: 10000,
                equity: 10000,
                margin_used: 500,
                free_margin: 9500,
              },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => usePaperTrading());

      // Wait for initial data fetch and portfolio to be set
      await waitFor(() => {
        expect(result.current.openTrades.length).toBeGreaterThan(0);
        expect(result.current.portfolio.current_balance).toBe(10000);
      }, { timeout: 2000 });

      // Get the WebSocket instance
      expect(wsInstances.length).toBeGreaterThan(0);
      const ws = wsInstances[0];

      // Send MarketData message with matching symbol
      await act(async () => {
        if (ws.onmessage) {
          ws.onmessage({
            data: JSON.stringify({
              type: 'MarketData',
              data: {
                symbol: 'BTCUSDT',
                price: 51000,
              },
              timestamp: new Date().toISOString(),
            }),
          } as MessageEvent);
        }
      });

      // Verify trade PnL was updated (lines 936-955)
      // Long trade: (51000 - 50000) * 0.1 = 100
      await waitFor(() => {
        expect(result.current.openTrades[0].pnl).toBe(100);
      });
    });

    it('should calculate unrealized PnL for Short trades (lines 941-943)', async () => {
      // Mock open trades endpoint with Short trade
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/trades/open')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: [
                {
                  id: 'trade-2',
                  symbol: 'ETHUSDT',
                  trade_type: 'Short',
                  status: 'Open',
                  entry_price: 3000,
                  quantity: 1.0,
                  leverage: 5,
                  pnl: 0,
                  pnl_percentage: 0,
                  open_time: new Date().toISOString(),
                },
              ],
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/status')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: {
                is_running: true,
                portfolio: {
                  total_trades: 1,
                  win_rate: 0,
                  total_pnl: 0,
                  total_pnl_percentage: 0,
                  max_drawdown: 0,
                  max_drawdown_percentage: 0,
                  sharpe_ratio: 0,
                  profit_factor: 0,
                  average_win: 0,
                  average_loss: 0,
                  largest_win: 0,
                  largest_loss: 0,
                  current_balance: 10000,
                  equity: 10000,
                  margin_used: 600,
                  free_margin: 9400,
                },
                last_updated: new Date().toISOString(),
              },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        if (url.includes('/portfolio')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({
              success: true,
              data: {
                total_trades: 1,
                win_rate: 0,
                total_pnl: 0,
                total_pnl_percentage: 0,
                max_drawdown: 0,
                max_drawdown_percentage: 0,
                sharpe_ratio: 0,
                profit_factor: 0,
                average_win: 0,
                average_loss: 0,
                largest_win: 0,
                largest_loss: 0,
                current_balance: 10000,
                equity: 10000,
                margin_used: 600,
                free_margin: 9400,
              },
              timestamp: new Date().toISOString(),
            }),
          });
        }

        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => usePaperTrading());

      // Wait for initial data fetch and portfolio to be set
      await waitFor(() => {
        expect(result.current.openTrades.length).toBeGreaterThan(0);
        expect(result.current.portfolio.current_balance).toBe(10000);
      }, { timeout: 2000 });

      // Get the WebSocket instance
      expect(wsInstances.length).toBeGreaterThan(0);
      const ws = wsInstances[0];

      // Send MarketData message with matching symbol
      await act(async () => {
        if (ws.onmessage) {
          ws.onmessage({
            data: JSON.stringify({
              type: 'MarketData',
              data: {
                symbol: 'ETHUSDT',
                price: 2900,
              },
              timestamp: new Date().toISOString(),
            }),
          } as MessageEvent);
        }
      });

      // Verify Short trade PnL (lines 941-943)
      // Short trade: -(2900 - 3000) * 1.0 = 100
      await waitFor(() => {
        expect(result.current.openTrades[0].pnl).toBe(100);
      });
    });
  });

  describe('WebSocket price_update handler (lines 988-991)', () => {
    it('should trigger portfolio refresh on price_update with data (lines 988-991)', async () => {
      const { result } = renderHook(() => usePaperTrading());

      // Wait for initialization
      await delay(100);

      // Get the WebSocket instance
      expect(wsInstances.length).toBeGreaterThan(0);
      const ws = wsInstances[0];

      // Clear previous fetch calls
      mockFetch.mockClear();

      // Send price_update message with data
      await act(async () => {
        if (ws.onmessage) {
          ws.onmessage({
            data: JSON.stringify({
              type: 'price_update',
              data: {
                BTCUSDT: 51000,
                ETHUSDT: 3100,
              },
              timestamp: new Date().toISOString(),
            }),
          } as MessageEvent);
        }
      });

      // Wait a bit for async operations
      await delay(50);

      // Verify portfolio fetch was called (line 989)
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/portfolio')
      );
    });

    it('should not trigger refresh when price_update data is empty', async () => {
      const { result } = renderHook(() => usePaperTrading());

      await delay(100);

      // Get the WebSocket instance
      expect(wsInstances.length).toBeGreaterThan(0);
      const ws = wsInstances[0];

      // Clear previous fetch calls
      mockFetch.mockClear();

      // Send price_update message with empty data
      await act(async () => {
        if (ws.onmessage) {
          ws.onmessage({
            data: JSON.stringify({
              type: 'price_update',
              data: {},
              timestamp: new Date().toISOString(),
            }),
          } as MessageEvent);
        }
      });

      // Wait a bit
      await delay(50);

      // Verify portfolio fetch was NOT called (condition failed)
      expect(mockFetch).not.toHaveBeenCalledWith(
        expect.stringContaining('/portfolio')
      );
    });
  });
});
