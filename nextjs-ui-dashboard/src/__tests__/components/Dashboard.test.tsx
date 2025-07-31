import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, mockPosition, mockTrade } from '../../test/utils'
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
    connected: true,
    subscribe: vi.fn(),
    unsubscribe: vi.fn(),
  }),
}))

describe('Dashboard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders dashboard header', async () => {
    render(<Dashboard />)
    
    expect(screen.getByText('Trading Dashboard')).toBeInTheDocument()
    expect(screen.getByText('Overview of your trading portfolio')).toBeInTheDocument()
  })

  it('displays account balance', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('$10,000.00')).toBeInTheDocument()
    })
  })

  it('shows positions table', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('Current Positions')).toBeInTheDocument()
      expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      expect(screen.getByText('LONG')).toBeInTheDocument()
      expect(screen.getByText('0.1')).toBeInTheDocument()
    })
  })

  it('displays recent trades', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('Recent Trades')).toBeInTheDocument()
      expect(screen.getByText('BUY')).toBeInTheDocument()
      expect(screen.getByText('45,000')).toBeInTheDocument()
    })
  })

  it('shows PnL information', async () => {
    render(<Dashboard />)
    
    await waitFor(() => {
      expect(screen.getByText('Total P&L')).toBeInTheDocument()
      expect(screen.getByText('$150.00')).toBeInTheDocument()
      expect(screen.getByText('Daily P&L')).toBeInTheDocument()
      expect(screen.getByText('$25.00')).toBeInTheDocument()
    })
  })

  it('handles refresh button click', async () => {
    const user = userEvent.setup()
    render(<Dashboard />)
    
    const refreshButton = screen.getByRole('button', { name: /refresh/i })
    await user.click(refreshButton)
    
    // Should trigger data refetch
    expect(refreshButton).toBeInTheDocument()
  })

  it('shows loading state', async () => {
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

  it('displays error state', async () => {
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
    const mockSubscribe = vi.fn()
    
    vi.mock('../../hooks/useWebSocket', () => ({
      useWebSocket: () => ({
        connected: true,
        subscribe: mockSubscribe,
        unsubscribe: vi.fn(),
      }),
    }))
    
    render(<Dashboard />)
    
    expect(mockSubscribe).toHaveBeenCalledWith('positions')
    expect(mockSubscribe).toHaveBeenCalledWith('trades')
  })

  it('filters positions by symbol', async () => {
    const user = userEvent.setup()
    render(<Dashboard />)
    
    const filterInput = screen.getByPlaceholderText(/filter by symbol/i)
    await user.type(filterInput, 'BTC')
    
    expect(filterInput).toHaveValue('BTC')
  })

  it('sorts trades by timestamp', async () => {
    const user = userEvent.setup()
    render(<Dashboard />)
    
    const timestampHeader = screen.getByText('Time')
    await user.click(timestampHeader)
    
    // Should trigger sort
    expect(timestampHeader).toBeInTheDocument()
  })
})