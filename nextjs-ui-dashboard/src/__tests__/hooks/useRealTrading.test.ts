/**
 * useRealTrading Hook Tests
 *
 * Tests for real trading hook functionality including:
 * - Order placement with 2-step confirmation
 * - Order cancellation
 * - Portfolio fetching
 * - State management
 *
 * @spec:FR-TRADING-016 - Real Trading System
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-TRADING.md
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { useRealTrading, RealOrder, PlaceOrderRequest, PendingOrderConfirmation } from '@/hooks/useRealTrading';

// Mock dependencies
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: vi.fn(),
  }),
}));

vi.mock('@/hooks/useTradingMode', () => ({
  useTradingMode: () => ({
    mode: 'real',
  }),
}));

vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
  },
}));

// Mock fetch
const mockFetch = vi.fn();
global.fetch = mockFetch;

// Mock WebSocket
class MockWebSocket {
  static OPEN = 1;
  readyState = MockWebSocket.OPEN;
  onopen: (() => void) | null = null;
  onclose: (() => void) | null = null;
  onmessage: ((event: { data: string }) => void) | null = null;
  onerror: ((error: unknown) => void) | null = null;

  constructor() {
    setTimeout(() => this.onopen?.(), 0);
  }

  send = vi.fn();
  close = vi.fn();
}

global.WebSocket = MockWebSocket as unknown as typeof WebSocket;

describe('useRealTrading', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockFetch.mockReset();
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // ========================================
  // Type Tests
  // ========================================

  describe('Type Definitions', () => {
    it('should have correct RealOrder interface', () => {
      const order: RealOrder = {
        id: 'order-123',
        exchange_order_id: 12345678,
        symbol: 'BTCUSDT',
        side: 'BUY',
        order_type: 'LIMIT',
        quantity: 0.01,
        executed_quantity: 0.005,
        price: 50000,
        avg_fill_price: 49950,
        status: 'PARTIALLY_FILLED',
        is_entry: true,
        created_at: '2025-01-01T00:00:00Z',
        updated_at: '2025-01-01T00:01:00Z',
      };

      expect(order.id).toBe('order-123');
      expect(order.exchange_order_id).toBe(12345678);
      expect(order.symbol).toBe('BTCUSDT');
    });

    it('should have correct PlaceOrderRequest interface', () => {
      const request: PlaceOrderRequest = {
        symbol: 'ETHUSDT',
        side: 'SELL',
        order_type: 'MARKET',
        quantity: 1.5,
      };

      expect(request.symbol).toBe('ETHUSDT');
      expect(request.price).toBeUndefined();
    });

    it('should have correct PendingOrderConfirmation interface', () => {
      const confirmation: PendingOrderConfirmation = {
        token: 'abc-123',
        expires_at: '2025-01-01T00:01:00Z',
        summary: 'BUY 0.01 BTCUSDT',
        order_details: {
          symbol: 'BTCUSDT',
          side: 'BUY',
          order_type: 'MARKET',
          quantity: 0.01,
        },
      };

      expect(confirmation.token).toBe('abc-123');
      expect(confirmation.order_details.symbol).toBe('BTCUSDT');
    });
  });

  // ========================================
  // Initial State Tests
  // ========================================

  describe('Initial State', () => {
    it('should initialize with default state', async () => {
      // Mock API responses for initial fetch
      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      expect(result.current.isActive).toBe(false);
      expect(result.current.isLoading).toBe(false);
      expect(result.current.error).toBeNull();
      expect(result.current.activeOrders).toEqual([]);
      expect(result.current.pendingConfirmation).toBeNull();
    });

    it('should have order management methods', async () => {
      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      expect(typeof result.current.placeOrder).toBe('function');
      expect(typeof result.current.confirmOrder).toBe('function');
      expect(typeof result.current.cancelOrder).toBe('function');
      expect(typeof result.current.cancelAllOrders).toBe('function');
      expect(typeof result.current.modifySlTp).toBe('function');
      expect(typeof result.current.clearPendingConfirmation).toBe('function');
      expect(typeof result.current.refreshOrders).toBe('function');
    });
  });

  // ========================================
  // Order Placement Tests
  // ========================================

  describe('placeOrder', () => {
    it('should return confirmation token on first call (no token provided)', async () => {
      const confirmationResponse = {
        success: true,
        data: {
          requires_confirmation: true,
          token: 'confirm-token-123',
          expires_at: '2025-01-01T00:01:00Z',
          summary: 'BUY 0.01 BTCUSDT MARKET',
        },
      };

      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/orders') && !url.includes('/all')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve(confirmationResponse),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      // Wait for initial fetches to complete
      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.placeOrder({
          symbol: 'BTCUSDT',
          side: 'BUY',
          order_type: 'MARKET',
          quantity: 0.01,
        });
      });

      // Should return false (needs confirmation)
      expect(success).toBe(false);
      // Should store pending confirmation
      expect(result.current.pendingConfirmation).toBeDefined();
      expect(result.current.pendingConfirmation?.token).toBe('confirm-token-123');
    });

    it('should execute order when confirmation token is provided', async () => {
      const orderResponse = {
        success: true,
        data: {
          id: 'order-123',
          exchange_order_id: 12345678,
          symbol: 'BTCUSDT',
          side: 'BUY',
          order_type: 'MARKET',
          quantity: 0.01,
          executed_quantity: 0.01,
          price: null,
          avg_fill_price: 50000,
          status: 'FILLED',
          is_entry: true,
          created_at: '2025-01-01T00:00:00Z',
          updated_at: '2025-01-01T00:00:01Z',
        },
      };

      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve(orderResponse),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.placeOrder({
          symbol: 'BTCUSDT',
          side: 'BUY',
          order_type: 'MARKET',
          quantity: 0.01,
          confirmation_token: 'valid-token',
        });
      });

      expect(success).toBe(true);
      expect(result.current.pendingConfirmation).toBeNull();
    });

    it('should handle API errors gracefully', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/orders') && !url.includes('/all')) {
          return Promise.resolve({
            ok: false,
            json: () =>
              Promise.resolve({
                success: false,
                error: 'Insufficient balance',
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

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.placeOrder({
          symbol: 'BTCUSDT',
          side: 'BUY',
          order_type: 'MARKET',
          quantity: 100000, // Very large quantity
        });
      });

      expect(success).toBe(false);
      expect(result.current.error).toBeDefined();
    });
  });

  // ========================================
  // Order Cancellation Tests
  // ========================================

  describe('cancelOrder', () => {
    it('should cancel order successfully', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/orders/order-123')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: true,
                data: { message: 'Order cancelled' },
              }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.cancelOrder('order-123');
      });

      expect(success).toBe(true);
    });

    it('should handle cancel failure', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/orders/') && !url.includes('/all')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: false,
                error: 'Order not found',
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

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.cancelOrder('nonexistent-order');
      });

      expect(success).toBe(false);
    });
  });

  // ========================================
  // Cancel All Orders Tests
  // ========================================

  describe('cancelAllOrders', () => {
    it('should cancel all orders successfully', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/orders/all')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: true,
                data: { message: 'All orders cancelled', count: 3 },
              }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.cancelAllOrders();
      });

      expect(success).toBe(true);
      expect(result.current.activeOrders).toEqual([]);
    });

    it('should cancel all orders for specific symbol', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/orders/all?symbol=BTCUSDT')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: true,
                data: { message: 'BTCUSDT orders cancelled' },
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

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.cancelAllOrders('BTCUSDT');
      });

      expect(success).toBe(true);
    });
  });

  // ========================================
  // Modify SL/TP Tests
  // ========================================

  describe('modifySlTp', () => {
    it('should modify stop loss and take profit', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/positions/BTCUSDT/sltp')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: true,
                data: { message: 'SL/TP updated' },
              }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: [] }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.modifySlTp('BTCUSDT', 48000, 55000);
      });

      expect(success).toBe(true);
    });

    it('should modify only stop loss', async () => {
      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/positions/ETHUSDT/sltp')) {
          return Promise.resolve({
            ok: true,
            json: () =>
              Promise.resolve({
                success: true,
                data: { message: 'SL updated' },
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

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.modifySlTp('ETHUSDT', 2800, undefined);
      });

      expect(success).toBe(true);
    });
  });

  // ========================================
  // Confirmation Flow Tests
  // ========================================

  describe('Confirmation Flow', () => {
    it('should clear pending confirmation', async () => {
      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () =>
            Promise.resolve({
              success: true,
              data: {
                requires_confirmation: true,
                token: 'token-123',
                expires_at: '2025-01-01T00:01:00Z',
                summary: 'BUY 0.01 BTCUSDT',
              },
            }),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      // Place order to get pending confirmation
      await act(async () => {
        await result.current.placeOrder({
          symbol: 'BTCUSDT',
          side: 'BUY',
          order_type: 'MARKET',
          quantity: 0.01,
        });
      });

      expect(result.current.pendingConfirmation).toBeDefined();

      // Clear it
      act(() => {
        result.current.clearPendingConfirmation();
      });

      expect(result.current.pendingConfirmation).toBeNull();
    });

    it('should not confirm if no pending order', async () => {
      mockFetch.mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        })
      );

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => expect(mockFetch).toHaveBeenCalled());

      let success: boolean | undefined;
      await act(async () => {
        success = await result.current.confirmOrder();
      });

      expect(success).toBe(false);
    });
  });

  // ========================================
  // Fetch Orders Tests
  // ========================================

  describe('fetchOrders', () => {
    it('should fetch active orders', async () => {
      const orders: RealOrder[] = [
        {
          id: 'order-1',
          exchange_order_id: 11111,
          symbol: 'BTCUSDT',
          side: 'BUY',
          order_type: 'LIMIT',
          quantity: 0.01,
          executed_quantity: 0,
          price: 40000,
          avg_fill_price: 0,
          status: 'NEW',
          is_entry: true,
          created_at: '2025-01-01T00:00:00Z',
          updated_at: '2025-01-01T00:00:00Z',
        },
        {
          id: 'order-2',
          exchange_order_id: 22222,
          symbol: 'ETHUSDT',
          side: 'SELL',
          order_type: 'LIMIT',
          quantity: 1.0,
          executed_quantity: 0.5,
          price: 3500,
          avg_fill_price: 3500,
          status: 'PARTIALLY_FILLED',
          is_entry: false,
          created_at: '2025-01-01T00:00:00Z',
          updated_at: '2025-01-01T00:01:00Z',
        },
      ];

      mockFetch.mockImplementation((url: string) => {
        if (url.includes('/orders') && !url.includes('/all')) {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({ success: true, data: orders }),
          });
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: null }),
        });
      });

      const { result } = renderHook(() => useRealTrading());

      await waitFor(() => {
        expect(result.current.activeOrders.length).toBe(2);
      });

      expect(result.current.activeOrders[0].symbol).toBe('BTCUSDT');
      expect(result.current.activeOrders[1].symbol).toBe('ETHUSDT');
    });
  });
});
