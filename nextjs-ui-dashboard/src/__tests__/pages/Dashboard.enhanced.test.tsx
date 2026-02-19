import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
import Dashboard from '../../pages/Dashboard'

// Create configurable mock factories
const createPaperTradingMock = (overrides = {}) => ({
  portfolio: {
    current_balance: 10000,
    available_balance: 9000,
    equity: 10000,
    total_pnl: 150,
    total_pnl_percentage: 1.5,
    total_trades: 10,
    margin_used: 1000,
    free_margin: 9000,
    win_rate: 60,
    average_win: 25,
    average_loss: -15,
    profit_factor: 1.67,
    max_drawdown: 500,
    max_drawdown_percentage: 5,
    sharpe_ratio: 1.2,
    win_streak: 3,
    loss_streak: 1,
    best_trade: 100,
    worst_trade: -50,
    ...overrides.portfolio,
  },
  positions: overrides.positions || [],
  openTrades: overrides.openTrades || [],
  closedTrades: overrides.closedTrades || [],
  recentSignals: overrides.recentSignals || [],
  isActive: overrides.isActive ?? true,
  isLoading: overrides.isLoading ?? false,
  error: overrides.error ?? null,
  lastUpdated: new Date(),
  startTrading: vi.fn(),
  stopTrading: vi.fn(),
  updateSettings: vi.fn(),
  resetPortfolio: vi.fn(),
  closeTrade: vi.fn(),
  refreshAISignals: vi.fn(),
  refreshSettings: vi.fn(),
  refreshData: overrides.refreshData || vi.fn(),
})

// Mock fetch globally
global.fetch = vi.fn() as any

// Mock API
vi.mock('@/services/api', () => {
  const mockAuthClient = {
    login: vi.fn(),
    register: vi.fn(),
    getProfile: vi.fn(() => Promise.resolve({
      id: 'user123',
      email: 'test@example.com',
      full_name: 'Test User',
      roles: ['user'],
      created_at: '2024-01-01T00:00:00Z',
    })),
    verifyToken: vi.fn(),
    setAuthToken: vi.fn(),
    removeAuthToken: vi.fn(),
    getAuthToken: vi.fn(() => null),
    isTokenExpired: vi.fn(() => true),
  }

  return {
    BotCoreApiClient: vi.fn(function() {
      this.auth = mockAuthClient
      this.rust = {
        getChartData: vi.fn(() => Promise.resolve({
          latest_price: 50000,
          price_change_24h: 1500,
          price_change_percent_24h: 3.0,
        })),
      }
      this.python = {}
    }),
    apiClient: {
      rust: {
        getChartData: vi.fn(() => Promise.resolve({
          latest_price: 50000,
          price_change_24h: 1500,
          price_change_percent_24h: 3.0,
        })),
      },
    },
  }
})

// Mock hooks with default values
let paperTradingMockValue = createPaperTradingMock()

vi.mock('../../hooks/usePaperTrading', () => ({
  usePaperTrading: () => paperTradingMockValue,
}))

vi.mock('../../hooks/useWebSocket', () => ({
  useWebSocket: () => ({
    state: {
      isConnected: true,
      isConnecting: false,
      error: null,
      lastMessage: null,
      positions: [],
      trades: [],
      aiSignals: [],
      botStatus: null,
    },
    connect: vi.fn(),
    disconnect: vi.fn(),
    sendMessage: vi.fn(),
  }),
}))

vi.mock('../../contexts/TradingModeContext', () => ({
  useTradingModeContext: () => ({
    mode: 'paper',
    setMode: vi.fn(),
    isPaperMode: true,
    isLiveMode: false,
  }),
  TradingModeProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

vi.mock('../../contexts/AuthContext', () => ({
  useAuth: () => ({
    user: {
      id: 'user123',
      email: 'test@example.com',
      full_name: 'Test User',
    },
    isAuthenticated: true,
    isLoading: false,
    login: vi.fn(),
    logout: vi.fn(),
    register: vi.fn(),
  }),
  AuthProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

describe('Dashboard - Enhanced Coverage', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    paperTradingMockValue = createPaperTradingMock()
  })

  describe('Error States', () => {
    it('displays error banner when API fails', async () => {
      paperTradingMockValue = createPaperTradingMock({
        error: 'Connection failed',
        refreshData: vi.fn()
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/Connection failed/i)).toBeInTheDocument()
        expect(screen.getByText(/Thử lại/i)).toBeInTheDocument()
      })
    })

    it('retry button calls refreshData', async () => {
      const mockRefresh = vi.fn()
      paperTradingMockValue = createPaperTradingMock({
        error: 'Connection failed',
        refreshData: mockRefresh
      })

      const user = userEvent.setup()
      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/Thử lại/i)).toBeInTheDocument()
      })

      const retryBtn = screen.getByText(/Thử lại/i)
      await user.click(retryBtn)

      expect(mockRefresh).toHaveBeenCalled()
    })
  })

  describe('Trade Display', () => {
    it('shows open trade with animated badge', async () => {
      const openTrade = {
        id: 'open-1',
        symbol: 'BTCUSDT',
        side: 'BUY' as const,
        quantity: 0.5,
        entry_price: 50000,
        open_time: new Date().toISOString(),
        status: 'open' as const,
      }

      paperTradingMockValue = createPaperTradingMock({
        closedTrades: [openTrade]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/BTC/i)).toBeInTheDocument()
        expect(screen.getByText(/Đang mở/i)).toBeInTheDocument()
      })
    })

    it('displays closed trade with PnL', async () => {
      const closedTrade = {
        id: 'closed-1',
        symbol: 'ETHUSDT',
        side: 'SELL' as const,
        quantity: 1.0,
        entry_price: 3000,
        exit_price: 3100,
        pnl: 100,
        open_time: new Date(Date.now() - 7200000).toISOString(),
        close_time: new Date(Date.now() - 3600000).toISOString(),
        status: 'closed' as const,
      }

      paperTradingMockValue = createPaperTradingMock({
        closedTrades: [closedTrade]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/ETH/i)).toBeInTheDocument()
        expect(screen.getByText(/\+\$?100\.00/i)).toBeInTheDocument()
      })
    })

    it('displays negative PnL trade with loss color', async () => {
      const lossTrade = {
        id: 'loss-1',
        symbol: 'BNBUSDT',
        side: 'BUY' as const,
        quantity: 5.0,
        entry_price: 400,
        exit_price: 380,
        pnl: -100,
        open_time: new Date(Date.now() - 3600000).toISOString(),
        close_time: new Date(Date.now() - 1800000).toISOString(),
        status: 'closed' as const,
      }

      paperTradingMockValue = createPaperTradingMock({
        closedTrades: [lossTrade]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/BNB/i)).toBeInTheDocument()
        expect(screen.getByText(/-\$?100\.00/i)).toBeInTheDocument()
      })
    })
  })

  describe('AI Signals', () => {
    it('displays long signal with confidence', async () => {
      const longSignal = {
        signal: 'long' as const,
        confidence: 0.85,
        symbol: 'BTCUSDT',
        timestamp: new Date(Date.now() - 300000).toISOString(),
        model_type: 'LSTM',
        timeframe: '1h',
      }

      paperTradingMockValue = createPaperTradingMock({
        recentSignals: [longSignal]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/BTC/i)).toBeInTheDocument()
        expect(screen.getByText(/LONG/i)).toBeInTheDocument()
        expect(screen.getByText(/85%/i)).toBeInTheDocument()
      })
    })

    it('displays short signal', async () => {
      const shortSignal = {
        signal: 'short' as const,
        confidence: 0.75,
        symbol: 'ETHUSDT',
        timestamp: new Date(Date.now() - 180000).toISOString(),
        model_type: 'GRU',
        timeframe: '4h',
      }

      paperTradingMockValue = createPaperTradingMock({
        recentSignals: [shortSignal]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/ETH/i)).toBeInTheDocument()
        expect(screen.getByText(/SHORT/i)).toBeInTheDocument()
        expect(screen.getByText(/75%/i)).toBeInTheDocument()
      })
    })

    it('displays neutral signal', async () => {
      const neutralSignal = {
        signal: 'neutral' as const,
        confidence: 0.65,
        symbol: 'BNBUSDT',
        timestamp: new Date(Date.now() - 120000).toISOString(),
        model_type: 'Transformer',
        timeframe: '15m',
      }

      paperTradingMockValue = createPaperTradingMock({
        recentSignals: [neutralSignal]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/BNB/i)).toBeInTheDocument()
        expect(screen.getByText(/NEUTRAL/i)).toBeInTheDocument()
        expect(screen.getByText(/65%/i)).toBeInTheDocument()
      })
    })

    it('formats signal timestamp as just now', async () => {
      const recentSignal = {
        signal: 'long' as const,
        confidence: 0.9,
        symbol: 'BTCUSDT',
        timestamp: new Date(Date.now() - 30000).toISOString(),
        model_type: 'LSTM',
        timeframe: '5m',
      }

      paperTradingMockValue = createPaperTradingMock({
        recentSignals: [recentSignal]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/Vừa xong/i)).toBeInTheDocument()
      })
    })
  })

  describe('Portfolio Display', () => {
    it('shows negative PnL with loss styling', async () => {
      paperTradingMockValue = createPaperTradingMock({
        portfolio: {
          current_balance: 9500,
          equity: 9500,
          total_pnl: -500,
          total_pnl_percentage: -5.0,
          win_rate: 40,
          profit_factor: 0.8,
        }
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/9,500/)).toBeInTheDocument()
        expect(screen.getByText(/-5\.00%/i)).toBeInTheDocument()
      })
    })

    it('calculates avgProfit from profit_factor', async () => {
      paperTradingMockValue = createPaperTradingMock({
        portfolio: {
          current_balance: 12000,
          total_pnl: 2000,
          total_pnl_percentage: 20.0,
          total_trades: 25,
          win_rate: 75,
          profit_factor: 2.5, // avgProfit = (2.5 - 1) * 100 = 150%
        }
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/\+150\.00%/i)).toBeInTheDocument()
        expect(screen.getByText(/75\.0%/i)).toBeInTheDocument()
      })
    })
  })

  describe('Loading States', () => {
    it('displays loading skeletons', async () => {
      paperTradingMockValue = createPaperTradingMock({
        isLoading: true,
        portfolio: {
          current_balance: 0,
        }
      })

      render(<Dashboard />)

      const skeletons = document.querySelectorAll('.animate-pulse')
      expect(skeletons.length).toBeGreaterThan(0)
    })
  })

  describe('Time Formatting', () => {
    it('formats trade time in hours when recent', async () => {
      const recentTrade = {
        id: 'recent-1',
        symbol: 'BTCUSDT',
        side: 'BUY' as const,
        quantity: 0.1,
        entry_price: 50000,
        exit_price: 51000,
        pnl: 100,
        open_time: new Date(Date.now() - 7200000).toISOString(),
        close_time: new Date(Date.now() - 3600000).toISOString(),
        status: 'closed' as const,
      }

      paperTradingMockValue = createPaperTradingMock({
        closedTrades: [recentTrade]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/giờ trước/i)).toBeInTheDocument()
      })
    })
  })

  describe('Component Lifecycle', () => {
    it('calls refreshData on mount', async () => {
      const mockRefresh = vi.fn()
      paperTradingMockValue = createPaperTradingMock({
        refreshData: mockRefresh
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(mockRefresh).toHaveBeenCalledTimes(1)
      })
    })

    it('completes initial load after timeout', async () => {
      paperTradingMockValue = createPaperTradingMock({
        isLoading: false,
        portfolio: {
          current_balance: 10000,
        }
      })

      render(<Dashboard />)

      // Wait for initial load (component has 500ms timeout internally)
      await waitFor(() => {
        expect(screen.getByText(/10,000/)).toBeInTheDocument()
      }, { timeout: 1000 })
    })
  })

  describe('Advanced Time Formatting', () => {
    it('formats signal timestamp in hours when > 60 minutes', async () => {
      const oldSignal = {
        signal: 'long' as const,
        confidence: 0.85,
        symbol: 'BTCUSDT',
        timestamp: new Date(Date.now() - 7200000).toISOString(), // 2 hours ago
        model_type: 'LSTM',
        timeframe: '1h',
      }

      paperTradingMockValue = createPaperTradingMock({
        recentSignals: [oldSignal]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/2 giờ trước/i)).toBeInTheDocument()
      })
    })

    it('formats signal timestamp in minutes when < 60 minutes', async () => {
      const recentSignal = {
        signal: 'long' as const,
        confidence: 0.85,
        symbol: 'BTCUSDT',
        timestamp: new Date(Date.now() - 1800000).toISOString(), // 30 minutes ago
        model_type: 'LSTM',
        timeframe: '1h',
      }

      paperTradingMockValue = createPaperTradingMock({
        recentSignals: [recentSignal]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/30 phút trước/i)).toBeInTheDocument()
      })
    })

    it('formats trade timestamp in days when > 24 hours', async () => {
      const oldTrade = {
        id: 'old-1',
        symbol: 'BTCUSDT',
        side: 'BUY' as const,
        quantity: 0.1,
        entry_price: 50000,
        exit_price: 51000,
        pnl: 100,
        open_time: new Date(Date.now() - 172800000).toISOString(), // 2 days ago
        close_time: new Date(Date.now() - 86400000).toISOString(), // 1 day ago
        status: 'closed' as const,
      }

      paperTradingMockValue = createPaperTradingMock({
        closedTrades: [oldTrade]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/1 ngày trước/i)).toBeInTheDocument()
      })
    })

    it('formats trade timestamp in minutes when < 60 minutes', async () => {
      const veryRecentTrade = {
        id: 'very-recent-1',
        symbol: 'ETHUSDT',
        side: 'SELL' as const,
        quantity: 1.0,
        entry_price: 3000,
        exit_price: 3050,
        pnl: 50,
        open_time: new Date(Date.now() - 3600000).toISOString(),
        close_time: new Date(Date.now() - 1800000).toISOString(), // 30 min ago
        status: 'closed' as const,
      }

      paperTradingMockValue = createPaperTradingMock({
        closedTrades: [veryRecentTrade]
      })

      render(<Dashboard />)

      await waitFor(() => {
        expect(screen.getByText(/30 phút trước/i)).toBeInTheDocument()
      })
    })
  })

  describe('Chart Data Generation', () => {
    it('renders performance chart with trades', async () => {
      const trades = [
        {
          id: 'trade-1',
          symbol: 'BTCUSDT',
          side: 'BUY' as const,
          quantity: 0.1,
          entry_price: 50000,
          exit_price: 51000,
          pnl: 100,
          open_time: new Date(Date.now() - 604800000).toISOString(),
          close_time: new Date(Date.now() - 518400000).toISOString(),
          status: 'closed' as const,
        },
      ]

      paperTradingMockValue = createPaperTradingMock({
        closedTrades: trades,
        portfolio: {
          current_balance: 10000,
        }
      })

      render(<Dashboard />)

      // Just check that Dashboard renders with data
      await waitFor(() => {
        expect(screen.getByText(/Chào mừng trở lại/i)).toBeInTheDocument()
      })
    })
  })

  describe('Price Ticker Interactions', () => {
    it('renders price ticker with crypto symbols', async () => {
      render(<Dashboard />)

      // Check that ticker displays multiple crypto symbols
      await waitFor(() => {
        expect(screen.getByText(/BTC/i)).toBeInTheDocument()
        expect(screen.getByText(/ETH/i)).toBeInTheDocument()
      })
    })
  })
})
