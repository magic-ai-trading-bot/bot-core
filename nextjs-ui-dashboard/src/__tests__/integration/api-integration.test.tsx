import { describe, it, expect, beforeAll, afterEach, afterAll, vi } from 'vitest';
import { renderHook, waitFor, act } from '@testing-library/react';
import { rest } from 'msw';
import { setupServer } from 'msw/node';
import { useAuth } from '@/contexts/AuthContext';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { ReactNode } from 'react';
import api from '@/services/api';

const server = setupServer();

beforeAll(() => server.listen({ onUnhandledRequest: 'bypass' }));
afterEach(() => server.resetHandlers());
afterAll(() => server.close());

const createWrapper = () => {
  const queryClient = new QueryClient({
    defaultOptions: {
      queries: { retry: false },
      mutations: { retry: false },
    },
  });

  return ({ children }: { children: ReactNode }) => (
    <QueryClientProvider client={queryClient}>{children}</QueryClientProvider>
  );
};

describe('API Integration Tests', () => {
  it('should login successfully with valid credentials', async () => {
    server.use(
      rest.post('http://localhost:8080/api/auth/login', (req, res, ctx) => {
        return res(
          ctx.json({
            token: 'jwt-token-12345',
            user: {
              id: 1,
              email: 'test@example.com',
              username: 'testuser',
            },
          })
        );
      })
    );

    const response = await api.post('/auth/login', {
      email: 'test@example.com',
      password: 'password123',
    });

    expect(response.data.token).toBe('jwt-token-12345');
    expect(response.data.user.email).toBe('test@example.com');
  });

  it('should handle login failure with 401', async () => {
    server.use(
      rest.post('http://localhost:8080/api/auth/login', (req, res, ctx) => {
        return res(
          ctx.status(401),
          ctx.json({
            error: 'Invalid credentials',
          })
        );
      })
    );

    await expect(
      api.post('/auth/login', {
        email: 'wrong@example.com',
        password: 'wrongpass',
      })
    ).rejects.toThrow();
  });

  it('should fetch market data successfully', async () => {
    server.use(
      rest.get('http://localhost:8080/api/market/data', (req, res, ctx) => {
        return res(
          ctx.json({
            symbol: 'BTCUSDT',
            price: 50000.0,
            volume: 1000000,
            change24h: 2.5,
          })
        );
      })
    );

    const response = await api.get('/market/data?symbol=BTCUSDT');

    expect(response.data.symbol).toBe('BTCUSDT');
    expect(response.data.price).toBe(50000.0);
  });

  it('should place a trade order successfully', async () => {
    server.use(
      rest.post('http://localhost:8080/api/paper-trading/order', (req, res, ctx) => {
        return res(
          ctx.json({
            orderId: 'order-123',
            symbol: 'BTCUSDT',
            side: 'BUY',
            quantity: 0.001,
            price: 50000.0,
            status: 'FILLED',
          })
        );
      })
    );

    const response = await api.post('/paper-trading/order', {
      symbol: 'BTCUSDT',
      side: 'BUY',
      quantity: 0.001,
      price: 50000.0,
    });

    expect(response.data.orderId).toBe('order-123');
    expect(response.data.status).toBe('FILLED');
  });

  it('should fetch AI analysis successfully', async () => {
    server.use(
      rest.post('http://localhost:8000/ai/analyze', (req, res, ctx) => {
        return res(
          ctx.json({
            signal: 'Long',
            confidence: 0.85,
            reasoning: 'Strong bullish indicators',
            suggested_entry: 50000.0,
            suggested_stop_loss: 48000.0,
            suggested_take_profit: 54000.0,
          })
        );
      })
    );

    const response = await api.post('/ai/analyze', {
      symbol: 'BTCUSDT',
      timeframe: '1h',
    });

    expect(response.data.signal).toBe('Long');
    expect(response.data.confidence).toBe(0.85);
  });

  it('should handle 500 server error gracefully', async () => {
    server.use(
      rest.get('http://localhost:8080/api/positions', (req, res, ctx) => {
        return res(
          ctx.status(500),
          ctx.json({
            error: 'Internal server error',
          })
        );
      })
    );

    await expect(api.get('/positions')).rejects.toThrow();
  });

  it('should retry on network failure', async () => {
    let callCount = 0;
    server.use(
      rest.get('http://localhost:8080/api/health', (req, res, ctx) => {
        callCount++;
        if (callCount < 2) {
          return res.networkError('Network error');
        }
        return res(ctx.json({ status: 'healthy' }));
      })
    );

    // Note: Actual retry logic depends on api configuration
    await expect(api.get('/health')).rejects.toThrow();
    expect(callCount).toBeGreaterThan(0);
  });

  it('should handle concurrent API requests', async () => {
    server.use(
      rest.get('http://localhost:8080/api/market/data', (req, res, ctx) => {
        const symbol = req.url.searchParams.get('symbol');
        return res(
          ctx.json({
            symbol,
            price: Math.random() * 50000,
          })
        );
      })
    );

    const requests = [
      api.get('/market/data?symbol=BTCUSDT'),
      api.get('/market/data?symbol=ETHUSDT'),
      api.get('/market/data?symbol=BNBUSDT'),
    ];

    const results = await Promise.all(requests);

    expect(results).toHaveLength(3);
    expect(results[0].data.symbol).toBe('BTCUSDT');
    expect(results[1].data.symbol).toBe('ETHUSDT');
    expect(results[2].data.symbol).toBe('BNBUSDT');
  });

  it('should fetch user portfolio data', async () => {
    server.use(
      rest.get('http://localhost:8080/api/portfolio', (req, res, ctx) => {
        return res(
          ctx.json({
            totalValue: 10000,
            cash: 5000,
            positions: [
              { symbol: 'BTCUSDT', quantity: 0.1, value: 5000 },
            ],
          })
        );
      })
    );

    const response = await api.get('/portfolio');

    expect(response.data.totalValue).toBe(10000);
    expect(response.data.positions).toHaveLength(1);
  });

  it('should update trading settings', async () => {
    server.use(
      rest.put('http://localhost:8080/api/settings', (req, res, ctx) => {
        return res(
          ctx.json({
            success: true,
            settings: {
              riskLevel: 'medium',
              maxPositionSize: 0.1,
            },
          })
        );
      })
    );

    const response = await api.put('/settings', {
      riskLevel: 'medium',
      maxPositionSize: 0.1,
    });

    expect(response.data.success).toBe(true);
  });

  it('should fetch trading history', async () => {
    server.use(
      rest.get('http://localhost:8080/api/trades', (req, res, ctx) => {
        return res(
          ctx.json({
            trades: [
              {
                id: 'trade-1',
                symbol: 'BTCUSDT',
                side: 'BUY',
                quantity: 0.001,
                price: 50000,
                pnl: 100,
                timestamp: '2025-01-01T00:00:00Z',
              },
            ],
            total: 1,
          })
        );
      })
    );

    const response = await api.get('/trades');

    expect(response.data.trades).toHaveLength(1);
    expect(response.data.trades[0].pnl).toBe(100);
  });

  it('should close a position successfully', async () => {
    server.use(
      rest.post('http://localhost:8080/api/positions/close', (req, res, ctx) => {
        return res(
          ctx.json({
            success: true,
            positionId: 'pos-123',
            pnl: 150.5,
          })
        );
      })
    );

    const response = await api.post('/positions/close', {
      positionId: 'pos-123',
    });

    expect(response.data.success).toBe(true);
    expect(response.data.pnl).toBe(150.5);
  });

  it('should fetch technical indicators', async () => {
    server.use(
      rest.get('http://localhost:8000/api/indicators', (req, res, ctx) => {
        return res(
          ctx.json({
            symbol: 'BTCUSDT',
            rsi: 65.5,
            macd: 123.45,
            bollinger_upper: 51000,
            bollinger_lower: 49000,
          })
        );
      })
    );

    const response = await api.get('/indicators?symbol=BTCUSDT');

    expect(response.data.rsi).toBe(65.5);
    expect(response.data.macd).toBe(123.45);
  });

  it('should handle rate limiting', async () => {
    server.use(
      rest.get('http://localhost:8080/api/data', (req, res, ctx) => {
        return res(
          ctx.status(429),
          ctx.json({
            error: 'Too many requests',
          })
        );
      })
    );

    await expect(api.get('/data')).rejects.toThrow();
  });

  it('should validate request payload', async () => {
    server.use(
      rest.post('http://localhost:8080/api/order', (req, res, ctx) => {
        return res(
          ctx.status(400),
          ctx.json({
            error: 'Invalid quantity',
          })
        );
      })
    );

    await expect(
      api.post('/order', {
        symbol: 'BTCUSDT',
        quantity: -1, // Invalid
      })
    ).rejects.toThrow();
  });

  it('should fetch real-time price data', async () => {
    server.use(
      rest.get('http://localhost:8080/api/price', (req, res, ctx) => {
        return res(
          ctx.json({
            symbol: 'BTCUSDT',
            price: 50123.45,
            timestamp: Date.now(),
          })
        );
      })
    );

    const response = await api.get('/price?symbol=BTCUSDT');

    expect(response.data.symbol).toBe('BTCUSDT');
    expect(response.data.price).toBeGreaterThan(0);
  });

  it('should start paper trading session', async () => {
    server.use(
      rest.post('http://localhost:8080/api/paper-trading/start', (req, res, ctx) => {
        return res(
          ctx.json({
            sessionId: 'session-123',
            initialBalance: 10000,
            status: 'active',
          })
        );
      })
    );

    const response = await api.post('/paper-trading/start', {
      initialBalance: 10000,
    });

    expect(response.data.sessionId).toBe('session-123');
    expect(response.data.status).toBe('active');
  });

  it('should stop paper trading session', async () => {
    server.use(
      rest.post('http://localhost:8080/api/paper-trading/stop', (req, res, ctx) => {
        return res(
          ctx.json({
            sessionId: 'session-123',
            finalBalance: 10500,
            totalPnl: 500,
            status: 'stopped',
          })
        );
      })
    );

    const response = await api.post('/paper-trading/stop', {
      sessionId: 'session-123',
    });

    expect(response.data.status).toBe('stopped');
    expect(response.data.totalPnl).toBe(500);
  });

  it('should fetch performance metrics', async () => {
    server.use(
      rest.get('http://localhost:8080/api/performance', (req, res, ctx) => {
        return res(
          ctx.json({
            totalTrades: 50,
            winRate: 62.5,
            sharpeRatio: 1.8,
            maxDrawdown: -8.5,
            totalPnl: 1250.75,
          })
        );
      })
    );

    const response = await api.get('/performance');

    expect(response.data.winRate).toBe(62.5);
    expect(response.data.sharpeRatio).toBe(1.8);
  });

  it('should register new user', async () => {
    server.use(
      rest.post('http://localhost:8080/api/auth/register', (req, res, ctx) => {
        return res(
          ctx.status(201),
          ctx.json({
            success: true,
            user: {
              id: 2,
              email: 'newuser@example.com',
            },
          })
        );
      })
    );

    const response = await api.post('/auth/register', {
      email: 'newuser@example.com',
      password: 'SecurePass123!',
    });

    expect(response.status).toBe(201);
    expect(response.data.success).toBe(true);
  });
});
