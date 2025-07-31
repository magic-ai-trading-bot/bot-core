import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
import TradingInterface from '../../components/TradingInterface'

// Mock the trade execution hook
const mockExecuteTrade = vi.fn()
vi.mock('../../hooks/useTradingApi', () => ({
  useTradingApi: () => ({
    executeTrade: mockExecuteTrade,
    isLoading: false,
  }),
}))

vi.mock('../../hooks/useMarketData', () => ({
  useMarketData: () => ({
    data: {
      price: 45000,
      change24h: 2.5,
      volume: 1234567,
    },
    isLoading: false,
  }),
}))

describe('TradingInterface', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockExecuteTrade.mockResolvedValue({
      trade_id: 'trade123',
      status: 'executed',
    })
  })

  it('renders trading form', () => {
    render(<TradingInterface />)
    
    expect(screen.getByText('Execute Trade')).toBeInTheDocument()
    expect(screen.getByLabelText(/symbol/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/side/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/quantity/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/price/i)).toBeInTheDocument()
  })

  it('displays current market price', () => {
    render(<TradingInterface />)
    
    expect(screen.getByText('$45,000.00')).toBeInTheDocument()
    expect(screen.getByText('+2.5%')).toBeInTheDocument()
  })

  it('validates form inputs', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    const submitButton = screen.getByRole('button', { name: /execute trade/i })
    await user.click(submitButton)
    
    expect(screen.getByText(/symbol is required/i)).toBeInTheDocument()
    expect(screen.getByText(/quantity must be greater than 0/i)).toBeInTheDocument()
  })

  it('executes buy trade', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    // Fill form
    await user.selectOptions(screen.getByLabelText(/symbol/i), 'BTCUSDT')
    await user.selectOptions(screen.getByLabelText(/side/i), 'BUY')
    await user.type(screen.getByLabelText(/quantity/i), '0.001')
    await user.type(screen.getByLabelText(/price/i), '45000')
    
    // Submit
    await user.click(screen.getByRole('button', { name: /execute trade/i }))
    
    await waitFor(() => {
      expect(mockExecuteTrade).toHaveBeenCalledWith({
        symbol: 'BTCUSDT',
        side: 'BUY',
        quantity: 0.001,
        price: 45000,
        type: 'LIMIT',
      })
    })
  })

  it('executes sell trade', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    await user.selectOptions(screen.getByLabelText(/symbol/i), 'BTCUSDT')
    await user.selectOptions(screen.getByLabelText(/side/i), 'SELL')
    await user.type(screen.getByLabelText(/quantity/i), '0.001')
    await user.type(screen.getByLabelText(/price/i), '45000')
    
    await user.click(screen.getByRole('button', { name: /execute trade/i }))
    
    await waitFor(() => {
      expect(mockExecuteTrade).toHaveBeenCalledWith({
        symbol: 'BTCUSDT',
        side: 'SELL',
        quantity: 0.001,
        price: 45000,
        type: 'LIMIT',
      })
    })
  })

  it('switches between order types', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    // Switch to market order
    await user.click(screen.getByLabelText(/market/i))
    
    // Price field should be disabled for market orders
    expect(screen.getByLabelText(/price/i)).toBeDisabled()
  })

  it('calculates order value', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    await user.type(screen.getByLabelText(/quantity/i), '0.1')
    await user.type(screen.getByLabelText(/price/i), '45000')
    
    expect(screen.getByText('Order Value: $4,500.00')).toBeInTheDocument()
  })

  it('shows loading state during execution', async () => {
    // Mock loading state
    vi.mock('../../hooks/useTradingApi', () => ({
      useTradingApi: () => ({
        executeTrade: mockExecuteTrade,
        isLoading: true,
      }),
    }))
    
    render(<TradingInterface />)
    
    const submitButton = screen.getByRole('button', { name: /executing/i })
    expect(submitButton).toBeDisabled()
  })

  it('handles trade execution error', async () => {
    mockExecuteTrade.mockRejectedValue(new Error('Insufficient balance'))
    
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    await user.selectOptions(screen.getByLabelText(/symbol/i), 'BTCUSDT')
    await user.selectOptions(screen.getByLabelText(/side/i), 'BUY')
    await user.type(screen.getByLabelText(/quantity/i), '0.001')
    await user.type(screen.getByLabelText(/price/i), '45000')
    await user.click(screen.getByRole('button', { name: /execute trade/i }))
    
    await waitFor(() => {
      expect(screen.getByText(/insufficient balance/i)).toBeInTheDocument()
    })
  })

  it('resets form after successful trade', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    await user.selectOptions(screen.getByLabelText(/symbol/i), 'BTCUSDT')
    await user.type(screen.getByLabelText(/quantity/i), '0.001')
    await user.type(screen.getByLabelText(/price/i), '45000')
    await user.click(screen.getByRole('button', { name: /execute trade/i }))
    
    await waitFor(() => {
      expect(screen.getByLabelText(/quantity/i)).toHaveValue('')
      expect(screen.getByLabelText(/price/i)).toHaveValue('')
    })
  })

  it('shows confirmation dialog for large trades', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    await user.selectOptions(screen.getByLabelText(/symbol/i), 'BTCUSDT')
    await user.type(screen.getByLabelText(/quantity/i), '10') // Large quantity
    await user.type(screen.getByLabelText(/price/i), '45000')
    await user.click(screen.getByRole('button', { name: /execute trade/i }))
    
    expect(screen.getByText(/confirm large trade/i)).toBeInTheDocument()
    expect(screen.getByText(/order value: \\$450,000/i)).toBeInTheDocument()
  })

  it('applies trading fees to order value', async () => {
    const user = userEvent.setup()
    render(<TradingInterface />)
    
    await user.type(screen.getByLabelText(/quantity/i), '0.1')
    await user.type(screen.getByLabelText(/price/i), '45000')
    
    expect(screen.getByText('Trading Fee: $4.50')).toBeInTheDocument()
    expect(screen.getByText('Total Cost: $4,504.50')).toBeInTheDocument()
  })
})
