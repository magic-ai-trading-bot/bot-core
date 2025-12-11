import { setupServer } from 'msw/node'
import { http, HttpResponse } from 'msw'

/**
 * MSW Handlers - Mock API responses
 * NOTE: Server instance is created lazily to avoid localStorage errors
 */
export const handlers = [
  // Auth endpoints
  http.post('http://localhost:8080/api/auth/login', () => {
    return HttpResponse.json({
      success: true,
      data: {
        token: 'mock-jwt-token',
        user: {
          id: 'user123',
          email: 'test@example.com',
          full_name: 'Test User',
        },
      },
    })
  }),

  http.post('http://localhost:8080/api/auth/register', () => {
    return HttpResponse.json({
      success: true,
      data: {
        token: 'mock-jwt-token',
        user: {
          id: 'user123',
          email: 'test@example.com',
          full_name: 'Test User',
        },
      },
    })
  }),

  http.get('http://localhost:8080/api/auth/profile', () => {
    return HttpResponse.json({
      success: true,
      data: {
        id: 'user123',
        email: 'test@example.com',
        full_name: 'Test User',
        created_at: '2024-01-01T00:00:00Z',
        roles: ['user'],
      },
    })
  }),

  http.put('http://localhost:8080/api/auth/profile', () => {
    return HttpResponse.json({
      success: true,
      data: {
        id: 'user123',
        email: 'test@example.com',
        full_name: 'Updated User',
        created_at: '2024-01-01T00:00:00Z',
        roles: ['user'],
      },
    })
  }),

  http.post('http://localhost:8080/api/auth/change-password', () => {
    return HttpResponse.json({
      success: true,
      data: { message: 'Password updated' }
    })
  }),

  // Trading endpoints
  http.get('http://localhost:8080/api/positions', () => {
    return HttpResponse.json({
      positions: [
        {
          symbol: 'BTCUSDT',
          side: 'LONG',
          quantity: 0.1,
          entry_price: 45000,
          current_price: 45500,
          unrealized_pnl: 50,
          percentage: 1.11,
        },
        {
          symbol: 'ETHUSDT',
          side: 'LONG',
          quantity: 1.0,
          entry_price: 3000,
          current_price: 3100,
          unrealized_pnl: 100,
          percentage: 3.33,
        },
      ],
    })
  }),

  http.get('http://localhost:8080/api/trades/history', () => {
    return HttpResponse.json({
      trades: [
        {
          id: 'trade1',
          symbol: 'BTCUSDT',
          side: 'BUY',
          quantity: 0.1,
          price: 45000,
          timestamp: '2024-01-01T00:00:00Z',
          status: 'executed',
          pnl: 50,
        },
        {
          id: 'trade2',
          symbol: 'ETHUSDT',
          side: 'SELL',
          quantity: 1.0,
          price: 3100,
          timestamp: '2024-01-01T01:00:00Z',
          status: 'executed',
          pnl: 100,
        },
      ],
      pagination: {
        page: 1,
        limit: 10,
        total: 2,
        pages: 1,
      },
    })
  }),

  http.get('http://localhost:8080/api/account', () => {
    return HttpResponse.json({
      balance: {
        USDT: 10000,
        BTC: 0.1,
        ETH: 1.0,
      },
      total_balance_usdt: 10000,
      total_pnl: 150,
      daily_pnl: 25,
    })
  }),

  http.post('http://localhost:8080/api/trades/execute', () => {
    return HttpResponse.json({
      trade_id: 'trade123',
      status: 'executed',
      symbol: 'BTCUSDT',
      side: 'BUY',
      quantity: 0.001,
      price: 45000,
      timestamp: new Date().toISOString(),
    })
  }),

  // AI Service endpoints
  http.get('http://localhost:8000/ai/status', () => {
    return HttpResponse.json({
      status: 'healthy',
      models_loaded: true,
      last_analysis: '2024-01-01T00:00:00Z',
    })
  }),

  http.post('http://localhost:8000/ai/analyze', () => {
    return HttpResponse.json({
      signal: 'Long',
      confidence: 0.85,
      reasoning: 'Strong bullish indicators detected',
      metadata: {
        rsi: 35,
        macd: 0.15,
        moving_averages: {
          sma_20: 44800,
          sma_50: 44500,
        },
      },
    })
  }),

  // WebSocket mock (for testing purposes)
  http.get('http://localhost:8080/ws', () => {
    return new Response('WebSocket endpoint', { status: 101 })
  }),

  // Market data
  http.get('http://localhost:8080/api/market/candles', () => {
    return HttpResponse.json({
      candles: Array.from({ length: 50 }, (_, i) => ({
        open: 45000 + i * 10,
        high: 45100 + i * 10,
        low: 44900 + i * 10,
        close: 45050 + i * 10,
        volume: 1000 + i * 5,
        timestamp: Date.now() - (50 - i) * 60000,
      })),
    })
  }),

  // Strategy endpoints
  http.get('http://localhost:8080/api/strategies', () => {
    return HttpResponse.json({
      strategies: [
        {
          id: 'rsi_strategy',
          name: 'RSI Strategy',
          type: 'RSI',
          enabled: true,
          parameters: {
            period: 14,
            overbought: 70,
            oversold: 30,
          },
          performance: {
            total_trades: 25,
            win_rate: 0.68,
            total_pnl: 350,
          },
        },
        {
          id: 'macd_strategy',
          name: 'MACD Strategy',
          type: 'MACD',
          enabled: false,
          parameters: {
            fast_period: 12,
            slow_period: 26,
            signal_period: 9,
          },
          performance: {
            total_trades: 18,
            win_rate: 0.61,
            total_pnl: 180,
          },
        },
      ],
    })
  }),
]

// Create server instance lazily (after jsdom localStorage is available)
let serverInstance: ReturnType<typeof setupServer> | null = null

export const getServer = () => {
  if (!serverInstance) {
    serverInstance = setupServer(...handlers)
  }
  return serverInstance
}

// For backward compatibility
export const server = new Proxy({} as ReturnType<typeof setupServer>, {
  get(target, prop) {
     
    return (getServer() as any)[prop]
  }
})