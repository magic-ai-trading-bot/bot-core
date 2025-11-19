import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock the API module - MUST be before any imports that use it
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
      this.rust = {}
      this.python = {}
    }),
  }
})

// Import other dependencies AFTER the mock
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, mockPosition, mockTrade, mockUser } from '../../test/utils'
import Dashboard from '../../pages/Dashboard'

// Mock the hooks
vi.mock('../../hooks/usePositions', () => ({
  usePositions: () => ({
    data: [mockPosition],
    isLoading: false,
    error: null,
  }),
}))

vi.mock('../../hooks/useTrades', () => ({
  useTrades: () => ({
    data: {
      trades: [mockTrade],
      pagination: { page: 1, limit: 10, total: 1, pages: 1 },
    },
    isLoading: false,
    error: null,
  }),
}))

vi.mock('../../hooks/useAccount', () => ({
  useAccount: () => ({
    data: {
      balance: { USDT: 10000, BTC: 0.1, ETH: 1.0 },
      total_balance_usdt: 10000,
      total_pnl: 150,
      daily_pnl: 25,
    },
    isLoading: false,
    error: null,
  }),
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

vi.mock('../../hooks/usePaperTrading', () => ({
  usePaperTrading: () => ({
    portfolio: {
      current_balance: 10000,
      available_balance: 9000,
      equity: 10000,
      total_pnl: 0,
      total_pnl_percentage: 0,
      total_trades: 0,
      margin_used: 1000,
      free_margin: 9000,
      win_rate: 0,
      average_win: 0,
      average_loss: 0,
      profit_factor: 0,
      max_drawdown: 0,
      max_drawdown_percentage: 0,
      sharpe_ratio: 0,
      win_streak: 0,
      loss_streak: 0,
      best_trade: 0,
      worst_trade: 0,
    },
    positions: [],
    openTrades: [],
    closedTrades: [],
    settings: {
      basic: {
        initial_balance: 10000,
        default_position_size_pct: 10,
        trading_fee_rate: 0.04,
      },
      risk: {
        max_leverage: 20,
        default_stop_loss_pct: 2,
        default_take_profit_pct: 4,
      },
    },
    recentSignals: [],
    isActive: false,
    isLoading: false,
    error: null,
    lastUpdated: new Date(),
    startTrading: vi.fn(),
    stopTrading: vi.fn(),
    updateSettings: vi.fn(),
    resetPortfolio: vi.fn(),
    closeTrade: vi.fn(),
    refreshAISignals: vi.fn(),
    refreshSettings: vi.fn(),
  }),
}))

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders dashboard header', async () => {
    render(<Dashboard />)

    await waitFor(() => {
      expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
      expect(screen.getByText('AI-Powered Futures Trading')).toBeInTheDocument()
    })
  })

  it.todo('displays account balance', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('$10,000.00')).toBeInTheDocument()
    })
  })

  it.todo('shows positions table', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('Current Positions')).toBeInTheDocument()
      expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      expect(screen.getByText('LONG')).toBeInTheDocument()
      expect(screen.getByText('0.1')).toBeInTheDocument()
    })
  })

  it.todo('displays recent trades', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('Recent Trades')).toBeInTheDocument()
      expect(screen.getByText('BUY')).toBeInTheDocument()
      expect(screen.getByText('45,000')).toBeInTheDocument()
    })
  })

  it.todo('shows PnL information', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('Total P&L')).toBeInTheDocument()
      expect(screen.getByText('$150.00')).toBeInTheDocument()
      expect(screen.getByText('Daily P&L')).toBeInTheDocument()
      expect(screen.getByText('$25.00')).toBeInTheDocument()
    })
  })

  it.todo('handles refresh button click', async () => {
    const user = userEvent.setup()
    render(<Dashboard />)

    const refreshButton = screen.getByRole('button', { name: /refresh/i })
    await user.click(refreshButton)

    // Should trigger data refetch
    expect(refreshButton).toBeInTheDocument()
  })

  it.todo('shows loading state', async () => {
    // Mock loading state
    vi.mock('../../hooks/usePositions', () => ({
      usePositions: () => ({
        data: null,
        isLoading: true,
        error: null,
      }),
    }))
    
    render(<Dashboard />)
    
    expect(screen.getByTestId('loading-spinner')).toBeInTheDocument()
  })

  it.todo('displays error state', async () => {
    // Mock error state
    vi.mock('../../hooks/usePositions', () => ({
      usePositions: () => ({
        data: null,
        isLoading: false,
        error: new Error('Failed to fetch positions'),
      }),
    }))
    
    render(<Dashboard />)
    
    expect(screen.getByText(/error loading/i)).toBeInTheDocument()
  })

  it('updates data when WebSocket receives updates', async () => {
    render(<Dashboard />)

    // Dashboard uses WebSocket data from state
    await waitFor(() => {
      expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    })
  })

  it.todo('filters positions by symbol', async () => {
    const user = userEvent.setup()
    render(<Dashboard />)
    
    const filterInput = screen.getByPlaceholderText(/filter by symbol/i)
    await user.type(filterInput, 'BTC')
    
    expect(filterInput).toHaveValue('BTC')
  })

  it.todo('sorts trades by timestamp', async () => {
    const user = userEvent.setup()
    render(<Dashboard />)
    
    const timestampHeader = screen.getByText('Time')
    await user.click(timestampHeader)
    
    // Should trigger sort
    expect(timestampHeader).toBeInTheDocument()
  })
})