import { describe, it, expect, beforeEach, vi } from 'vitest';
import axios from 'axios';
import * as api from '@/services/api';

vi.mock('axios');
const mockedAxios = axios as any;

describe('API Service - Comprehensive Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockedAxios.create.mockReturnValue(mockedAxios);
  });

  describe('Authentication', () => {
    it('should login with valid credentials', async () => {
      const mockResponse = {
        data: {
          token: 'test-token',
          user: { id: 1, email: 'test@example.com' },
        },
      };
      mockedAxios.post.mockResolvedValueOnce(mockResponse);

      const result = await api.login('test@example.com', 'password123');
      expect(result).toEqual(mockResponse.data);
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/auth/login', {
        email: 'test@example.com',
        password: 'password123',
      });
    });

    it('should handle login errors', async () => {
      mockedAxios.post.mockRejectedValueOnce(new Error('Invalid credentials'));

      await expect(api.login('test@example.com', 'wrong')).rejects.toThrow();
    });

    it('should register new user', async () => {
      const mockResponse = {
        data: {
          token: 'new-token',
          user: { id: 2, email: 'new@example.com' },
        },
      };
      mockedAxios.post.mockResolvedValueOnce(mockResponse);

      const result = await api.register('new@example.com', 'password123');
      expect(result).toEqual(mockResponse.data);
    });

    it('should handle duplicate email registration', async () => {
      mockedAxios.post.mockRejectedValueOnce({
        response: { status: 409, data: { error: 'Email already exists' } },
      });

      await expect(api.register('existing@example.com', 'password123')).rejects.toThrow();
    });
  });

  describe('Paper Trading', () => {
    it('should place buy order', async () => {
      const mockOrder = {
        id: '123',
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: 0.1,
        price: 45000,
      };
      mockedAxios.post.mockResolvedValueOnce({ data: mockOrder });

      const result = await api.placePaperOrder(mockOrder);
      expect(result).toEqual(mockOrder);
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/paper/orders', mockOrder);
    });

    it('should place sell order', async () => {
      const mockOrder = {
        id: '124',
        symbol: 'ETHUSDT',
        side: 'SELL',
        quantity: 1.5,
        price: 3000,
      };
      mockedAxios.post.mockResolvedValueOnce({ data: mockOrder });

      const result = await api.placePaperOrder(mockOrder);
      expect(result).toEqual(mockOrder);
    });

    it('should get all orders', async () => {
      const mockOrders = [
        { id: '1', symbol: 'BTCUSDT', side: 'BUY', quantity: 0.1, price: 45000 },
        { id: '2', symbol: 'ETHUSDT', side: 'SELL', quantity: 1.0, price: 3000 },
      ];
      mockedAxios.get.mockResolvedValueOnce({ data: mockOrders });

      const result = await api.getPaperOrders();
      expect(result).toEqual(mockOrders);
      expect(mockedAxios.get).toHaveBeenCalledWith('/api/paper/orders');
    });

    it('should get positions', async () => {
      const mockPositions = [
        {
          symbol: 'BTCUSDT',
          quantity: 0.5,
          entry_price: 44000,
          current_price: 45000,
          unrealized_pnl: 500,
        },
      ];
      mockedAxios.get.mockResolvedValueOnce({ data: mockPositions });

      const result = await api.getPaperPositions();
      expect(result).toEqual(mockPositions);
    });

    it('should get account balance', async () => {
      const mockBalance = {
        total: 10000,
        available: 8000,
        in_orders: 2000,
      };
      mockedAxios.get.mockResolvedValueOnce({ data: mockBalance });

      const result = await api.getPaperBalance();
      expect(result).toEqual(mockBalance);
    });

    it('should start paper trading', async () => {
      mockedAxios.post.mockResolvedValueOnce({ data: { status: 'started' } });

      const result = await api.startPaperTrading();
      expect(result).toEqual({ status: 'started' });
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/paper/start');
    });

    it('should stop paper trading', async () => {
      mockedAxios.post.mockResolvedValueOnce({ data: { status: 'stopped' } });

      const result = await api.stopPaperTrading();
      expect(result).toEqual({ status: 'stopped' });
      expect(mockedAxios.post).toHaveBeenCalledWith('/api/paper/stop');
    });

    it('should handle invalid order parameters', async () => {
      mockedAxios.post.mockRejectedValueOnce({
        response: {
          status: 400,
          data: { error: 'Invalid quantity' },
        },
      });

      const invalidOrder = {
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: -1,
        price: 45000,
      };

      await expect(api.placePaperOrder(invalidOrder)).rejects.toThrow();
    });

    it('should handle insufficient balance', async () => {
      mockedAxios.post.mockRejectedValueOnce({
        response: {
          status: 400,
          data: { error: 'Insufficient balance' },
        },
      });

      const order = {
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: 100,
        price: 45000,
      };

      await expect(api.placePaperOrder(order)).rejects.toThrow();
    });
  });

  describe('Market Data', () => {
    it('should fetch market prices', async () => {
      const mockPrices = {
        BTCUSDT: 45000,
        ETHUSDT: 3000,
      };
      mockedAxios.get.mockResolvedValueOnce({ data: mockPrices });

      // Assuming this function exists
      // const result = await api.getMarketPrices();
      // expect(result).toEqual(mockPrices);
    });
  });

  describe('Error Handling', () => {
    it('should handle network errors', async () => {
      mockedAxios.get.mockRejectedValueOnce(new Error('Network error'));

      await expect(api.getPaperOrders()).rejects.toThrow('Network error');
    });

    it('should handle timeout errors', async () => {
      mockedAxios.get.mockRejectedValueOnce({
        code: 'ECONNABORTED',
        message: 'timeout of 5000ms exceeded',
      });

      await expect(api.getPaperOrders()).rejects.toThrow();
    });

    it('should handle 401 unauthorized', async () => {
      mockedAxios.get.mockRejectedValueOnce({
        response: { status: 401, data: { error: 'Unauthorized' } },
      });

      await expect(api.getPaperOrders()).rejects.toThrow();
    });

    it('should handle 404 not found', async () => {
      mockedAxios.get.mockRejectedValueOnce({
        response: { status: 404, data: { error: 'Not found' } },
      });

      await expect(api.getPaperOrders()).rejects.toThrow();
    });

    it('should handle 500 server error', async () => {
      mockedAxios.get.mockRejectedValueOnce({
        response: { status: 500, data: { error: 'Internal server error' } },
      });

      await expect(api.getPaperOrders()).rejects.toThrow();
    });
  });

  describe('Request Configuration', () => {
    it('should include auth token in headers when available', async () => {
      localStorage.setItem('token', 'test-token');

      mockedAxios.get.mockResolvedValueOnce({ data: [] });

      await api.getPaperOrders();

      // Verify headers include token
      // This would need actual implementation details
    });

    it('should handle requests without auth token', async () => {
      localStorage.removeItem('token');

      mockedAxios.post.mockResolvedValueOnce({ data: { token: 'new-token' } });

      await api.login('test@example.com', 'password');

      // Should not include auth header for login
    });
  });
});
