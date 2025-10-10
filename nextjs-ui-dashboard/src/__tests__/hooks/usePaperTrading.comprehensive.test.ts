import { describe, it, expect, beforeEach, vi } from 'vitest';
import { renderHook, act, waitFor } from '@testing-library/react';
import { usePaperTrading } from '@/hooks/usePaperTrading';

// Mock the API
vi.mock('@/services/api', () => ({
  placePaperOrder: vi.fn(),
  getPaperOrders: vi.fn(),
  getPaperPositions: vi.fn(),
  getPaperBalance: vi.fn(),
  startPaperTrading: vi.fn(),
  stopPaperTrading: vi.fn(),
}));

describe('usePaperTrading - Comprehensive Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  describe('Order Placement', () => {
    it('should place buy order successfully', async () => {
      const { placePaperOrder } = await import('@/services/api');
      const mockOrder = { id: '123', symbol: 'BTCUSDT', side: 'BUY', quantity: 0.1, price: 45000 };
      (placePaperOrder as any).mockResolvedValueOnce(mockOrder);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.placeOrder({
          symbol: 'BTCUSDT',
          side: 'BUY',
          quantity: 0.1,
          price: 45000,
        });
      });

      expect(placePaperOrder).toHaveBeenCalled();
    });

    it('should place sell order successfully', async () => {
      const { placePaperOrder } = await import('@/services/api');
      const mockOrder = { id: '124', symbol: 'ETHUSDT', side: 'SELL', quantity: 1.0, price: 3000 };
      (placePaperOrder as any).mockResolvedValueOnce(mockOrder);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.placeOrder({
          symbol: 'ETHUSDT',
          side: 'SELL',
          quantity: 1.0,
          price: 3000,
        });
      });

      expect(placePaperOrder).toHaveBeenCalled();
    });

    it('should handle order placement errors', async () => {
      const { placePaperOrder } = await import('@/services/api');
      (placePaperOrder as any).mockRejectedValueOnce(new Error('Order failed'));

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        try {
          await result.current.placeOrder({
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0.1,
            price: 45000,
          });
        } catch (error) {
          expect(error).toBeDefined();
        }
      });
    });
  });

  describe('Position Management', () => {
    it('should fetch positions', async () => {
      const { getPaperPositions } = await import('@/services/api');
      const mockPositions = [
        {
          symbol: 'BTCUSDT',
          quantity: 0.5,
          entry_price: 44000,
          current_price: 45000,
          unrealized_pnl: 500,
        },
      ];
      (getPaperPositions as any).mockResolvedValueOnce(mockPositions);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.fetchPositions();
      });

      await waitFor(() => {
        expect(result.current.positions).toEqual(mockPositions);
      });
    });

    it('should calculate total PnL from positions', async () => {
      const { getPaperPositions } = await import('@/services/api');
      const mockPositions = [
        { symbol: 'BTCUSDT', unrealized_pnl: 500 },
        { symbol: 'ETHUSDT', unrealized_pnl: 300 },
        { symbol: 'ADAUSDT', unrealized_pnl: -100 },
      ];
      (getPaperPositions as any).mockResolvedValueOnce(mockPositions);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.fetchPositions();
      });

      await waitFor(() => {
        const totalPnl = result.current.positions.reduce(
          (sum: number, pos: any) => sum + (pos.unrealized_pnl || 0),
          0
        );
        expect(totalPnl).toBe(700);
      });
    });
  });

  describe('Balance Management', () => {
    it('should fetch balance', async () => {
      const { getPaperBalance } = await import('@/services/api');
      const mockBalance = { total: 10000, available: 8000, in_orders: 2000 };
      (getPaperBalance as any).mockResolvedValueOnce(mockBalance);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.fetchBalance();
      });

      await waitFor(() => {
        expect(result.current.balance).toEqual(mockBalance);
      });
    });

    it('should update balance after order execution', async () => {
      const { getPaperBalance, placePaperOrder } = await import('@/services/api');
      const initialBalance = { total: 10000, available: 8000, in_orders: 2000 };
      const updatedBalance = { total: 10000, available: 7500, in_orders: 2500 };

      (getPaperBalance as any)
        .mockResolvedValueOnce(initialBalance)
        .mockResolvedValueOnce(updatedBalance);

      (placePaperOrder as any).mockResolvedValueOnce({
        id: '123',
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: 0.01,
        price: 45000,
      });

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.fetchBalance();
      });

      await act(async () => {
        await result.current.placeOrder({
          symbol: 'BTCUSDT',
          side: 'BUY',
          quantity: 0.01,
          price: 45000,
        });
        await result.current.fetchBalance();
      });

      await waitFor(() => {
        expect(result.current.balance?.available).toBe(7500);
      });
    });
  });

  describe('Trading Control', () => {
    it('should start paper trading', async () => {
      const { startPaperTrading } = await import('@/services/api');
      (startPaperTrading as any).mockResolvedValueOnce({ status: 'started' });

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.startTrading();
      });

      expect(startPaperTrading).toHaveBeenCalled();
    });

    it('should stop paper trading', async () => {
      const { stopPaperTrading } = await import('@/services/api');
      (stopPaperTrading as any).mockResolvedValueOnce({ status: 'stopped' });

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.stopTrading();
      });

      expect(stopPaperTrading).toHaveBeenCalled();
    });

    it('should prevent multiple simultaneous starts', async () => {
      const { startPaperTrading } = await import('@/services/api');
      (startPaperTrading as any).mockImplementation(
        () => new Promise(resolve => setTimeout(() => resolve({ status: 'started' }), 100))
      );

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        const promise1 = result.current.startTrading();
        const promise2 = result.current.startTrading();
        await Promise.all([promise1, promise2]);
      });

      // Should only call API once
      expect(startPaperTrading).toHaveBeenCalledTimes(1);
    });
  });

  describe('Order History', () => {
    it('should fetch order history', async () => {
      const { getPaperOrders } = await import('@/services/api');
      const mockOrders = [
        { id: '1', symbol: 'BTCUSDT', side: 'BUY', quantity: 0.1, price: 45000, status: 'filled' },
        { id: '2', symbol: 'ETHUSDT', side: 'SELL', quantity: 1.0, price: 3000, status: 'filled' },
      ];
      (getPaperOrders as any).mockResolvedValueOnce(mockOrders);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.fetchOrders();
      });

      await waitFor(() => {
        expect(result.current.orders).toEqual(mockOrders);
      });
    });

    it('should filter orders by symbol', async () => {
      const { getPaperOrders } = await import('@/services/api');
      const mockOrders = [
        { id: '1', symbol: 'BTCUSDT', side: 'BUY', quantity: 0.1, price: 45000 },
        { id: '2', symbol: 'ETHUSDT', side: 'SELL', quantity: 1.0, price: 3000 },
        { id: '3', symbol: 'BTCUSDT', side: 'SELL', quantity: 0.05, price: 46000 },
      ];
      (getPaperOrders as any).mockResolvedValueOnce(mockOrders);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.fetchOrders();
      });

      await waitFor(() => {
        const btcOrders = result.current.orders.filter((o: any) => o.symbol === 'BTCUSDT');
        expect(btcOrders).toHaveLength(2);
      });
    });
  });

  describe('Performance Metrics', () => {
    it('should calculate win rate', async () => {
      const { getPaperOrders } = await import('@/services/api');
      const mockOrders = [
        { id: '1', pnl: 100, status: 'closed' },
        { id: '2', pnl: 50, status: 'closed' },
        { id: '3', pnl: -30, status: 'closed' },
        { id: '4', pnl: 80, status: 'closed' },
        { id: '5', pnl: -20, status: 'closed' },
      ];
      (getPaperOrders as any).mockResolvedValueOnce(mockOrders);

      const { result } = renderHook(() => usePaperTrading());

      await act(async () => {
        await result.current.fetchOrders();
      });

      await waitFor(() => {
        const closedOrders = result.current.orders.filter((o: any) => o.status === 'closed');
        const winningOrders = closedOrders.filter((o: any) => (o.pnl || 0) > 0);
        const winRate = (winningOrders.length / closedOrders.length) * 100;
        expect(winRate).toBe(60); // 3 wins out of 5 trades
      });
    });
  });
});
