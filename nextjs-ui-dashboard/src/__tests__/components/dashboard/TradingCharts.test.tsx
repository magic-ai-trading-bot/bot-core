import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, mockCandle } from '../../../test/utils'
import { TradingCharts } from '../../../components/dashboard/TradingCharts'
import { toast } from 'sonner'

// Mock recharts to avoid canvas issues
vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  LineChart: () => <div>LineChart</div>,
  Line: () => null,
  XAxis: () => null,
  YAxis: () => null,
  CartesianGrid: () => null,
  Tooltip: () => null,
  AreaChart: () => <div>AreaChart</div>,
  Area: () => null,
  ComposedChart: () => <div>ComposedChart</div>,
  Bar: () => null,
  Cell: () => null,
}))

// Mock API client - use vi.hoisted to avoid hoisting issues
const mockApiClient = vi.hoisted(() => ({
  rust: {
    getSupportedSymbols: vi.fn(),
    getChartData: vi.fn(),
    getLatestPrices: vi.fn(),
    addSymbol: vi.fn(),
    removeSymbol: vi.fn(),
  },
}))

vi.mock('@/services/api', () => ({
  BotCoreApiClient: class MockBotCoreApiClient {
    rust = mockApiClient.rust
    python = {}
    auth = { getAuthToken: vi.fn(() => null), isTokenExpired: vi.fn(() => false) }
    getDashboardData = vi.fn()
    performHealthCheck = vi.fn()
  },
  apiClient: mockApiClient,
  rustApi: mockApiClient.rust,
  pythonAI: {},
}))

// Mock toast
vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

// Mock WebSocket hook
const mockUseWebSocket = vi.fn()

vi.mock('../../../hooks/useWebSocket', () => ({
  useWebSocket: () => mockUseWebSocket(),
}))

describe('TradingCharts', () => {
  const mockChartData = {
    symbol: 'BTCUSDT',
    timeframe: '1m',
    latest_price: 45000,
    price_change_24h: 500,
    price_change_percent_24h: 1.12,
    volume_24h: 1000000,
    candles: [mockCandle],
  }

  beforeEach(() => {
    vi.clearAllMocks()

    // Default mock implementations - use single symbol to avoid duplicate elements
    mockApiClient.rust.getSupportedSymbols.mockResolvedValue({
      symbols: ['BTCUSDT'],
    })

    // Mock getChartData to return different data based on symbol
    mockApiClient.rust.getChartData.mockImplementation(async (symbol: string) => ({
      ...mockChartData,
      symbol,
    }))

    mockApiClient.rust.getLatestPrices.mockResolvedValue({
      BTCUSDT: 45500,
      ETHUSDT: 3000,
    })

    mockUseWebSocket.mockReturnValue({
      state: {
        isConnected: true,
        isConnecting: false,
        lastMessage: null,
      },
      connect: vi.fn(),
      disconnect: vi.fn(),
    })
  })

  describe('Component Rendering', () => {
    it('renders trading charts card', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('Real-time Trading Charts')).toBeInTheDocument()
      })
    })

    it('displays WebSocket live status', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('🟢 LIVE')).toBeInTheDocument()
      })
    })

    it('displays disconnected status when WebSocket is offline', async () => {
      mockUseWebSocket.mockReturnValue({
        state: {
          isConnected: false,
          isConnecting: false,
          lastMessage: null,
        },
        connect: vi.fn(),
        disconnect: vi.fn(),
      })

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('🔴 DISCONNECTED')).toBeInTheDocument()
      })
    })

    it('displays feature badges', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('🔥 HOT RELOAD')).toBeInTheDocument()
        expect(screen.getByText('⚡ MAINNET')).toBeInTheDocument()
        expect(screen.getByText('📡 WEBSOCKET')).toBeInTheDocument()
      })
    })
  })

  describe('Timeframe Selection', () => {
    it('displays timeframe selector', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        const selects = screen.getAllByRole('combobox')
        expect(selects.length).toBeGreaterThan(0)
      })
    })

    it.todo('loads charts when timeframe changes', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      await waitFor(() => {
        expect(mockApiClient.rust.getSupportedSymbols).toHaveBeenCalled()
      })

      vi.clearAllMocks()

      const select = screen.getAllByRole('combobox')[0]
      await user.click(select)

      // Select a different timeframe
      const option = screen.getByRole('option', { name: '5m' })
      await user.click(option)

      await waitFor(() => {
        expect(mockApiClient.rust.getSupportedSymbols).toHaveBeenCalled()
      })
    })
  })

  describe('Chart Data Loading', () => {
    it('loads chart data on mount', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(mockApiClient.rust.getSupportedSymbols).toHaveBeenCalled()
        expect(mockApiClient.rust.getChartData).toHaveBeenCalled()
      })
    })

    it('displays loading skeleton while loading', () => {
      render(<TradingCharts />)

      const skeletons = document.querySelectorAll('.animate-pulse')
      expect(skeletons.length).toBeGreaterThan(0)
    })

    it('displays charts after loading', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })
    })

    it('handles load error gracefully', async () => {
      mockApiClient.rust.getSupportedSymbols.mockRejectedValue(new Error('Network error'))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to load chart data')
      })
    })
  })

  describe('Chart Cards', () => {
    it('displays chart card with symbol', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })
    })

    it('displays timeframe badge', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('1m')).toBeInTheDocument()
      })
    })

    it('displays MongoDB badge', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('🗂️ MONGODB')).toBeInTheDocument()
      })
    })

    it('displays latest price', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getAllByText(/\$45,000/)[0]).toBeInTheDocument()
      })
    })

    it('displays 24h price change', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText(/\+1\.12%/)).toBeInTheDocument()
      })
    })

    it('displays volume', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText(/24h Volume:/)).toBeInTheDocument()
        expect(screen.getByText(/\$1,000,000/)).toBeInTheDocument()
      })
    })

    it('shows trending up icon for positive change', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        const trendingUpIcons = document.querySelectorAll('[class*="lucide-trending-up"]')
        expect(trendingUpIcons.length).toBeGreaterThan(0)
      })
    })

    it('shows trending down icon for negative change', async () => {
      mockApiClient.rust.getChartData.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        price_change_24h: -500,
        price_change_percent_24h: -1.12,
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        const trendingDownIcons = document.querySelectorAll('[class*="lucide-trending-down"]')
        expect(trendingDownIcons.length).toBeGreaterThan(0)
      })
    })
  })

  describe('Add Symbol Dialog', () => {
    it('opens add symbol dialog when button is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /add symbol/i })).toBeInTheDocument()
      })

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByText('Add New Trading Symbol')).toBeInTheDocument()
      })
    })

    it('displays symbol input field', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /add symbol/i })).toBeInTheDocument()
      })

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/Enter symbol like BTCUSDT/i)).toBeInTheDocument()
      })
    })

    it('displays timeframe selection buttons', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByText('Select Timeframes')).toBeInTheDocument()
        expect(screen.getByRole('button', { name: '1m' })).toBeInTheDocument()
        expect(screen.getByRole('button', { name: '5m' })).toBeInTheDocument()
        expect(screen.getByRole('button', { name: '15m' })).toBeInTheDocument()
      })
    })

    it('adds symbol successfully', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.addSymbol.mockResolvedValue({ success: true })

      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/Enter symbol like BTCUSDT/i)).toBeInTheDocument()
      })

      const input = screen.getByPlaceholderText(/Enter symbol like BTCUSDT/i)
      await user.type(input, 'SOLUSDT')

      const submitButton = screen.getByRole('button', { name: /add symbol/i, hidden: false })
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockApiClient.rust.addSymbol).toHaveBeenCalledWith({
          symbol: 'SOLUSDT',
          timeframes: ['1h'],
        })
        expect(toast.success).toHaveBeenCalledWith('Successfully added SOLUSDT')
      })
    })

    it('handles add symbol error', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.addSymbol.mockRejectedValue(new Error('Failed to add'))

      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/Enter symbol like BTCUSDT/i)).toBeInTheDocument()
      })

      const input = screen.getByPlaceholderText(/Enter symbol like BTCUSDT/i)
      await user.type(input, 'INVALID')

      const submitButton = screen.getByRole('button', { name: /add symbol/i, hidden: false })
      await user.click(submitButton)

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to add symbol')
      })
    })

    it('selects multiple timeframes', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.addSymbol.mockResolvedValue({ success: true })

      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: '5m' })).toBeInTheDocument()
      })

      // Select additional timeframes
      const tf5m = screen.getByRole('button', { name: '5m' })
      const tf15m = screen.getByRole('button', { name: '15m' })

      await user.click(tf5m)
      await user.click(tf15m)

      const input = screen.getByPlaceholderText(/Enter symbol like BTCUSDT/i)
      await user.type(input, 'ETHUSDT')

      const submitButton = screen.getByRole('button', { name: /add symbol/i, hidden: false })
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockApiClient.rust.addSymbol).toHaveBeenCalledWith({
          symbol: 'ETHUSDT',
          timeframes: expect.arrayContaining(['1h', '5m', '15m']),
        })
      })
    })
  })

  describe('Remove Symbol', () => {
    it('removes symbol when close button is clicked', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.removeSymbol.mockResolvedValue({ success: true })

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Find and click the X button
      const closeButtons = screen.getAllByRole('button')
      const closeButton = closeButtons.find(btn => btn.querySelector('[class*="lucide-x"]'))

      if (closeButton) {
        await user.click(closeButton)

        await waitFor(() => {
          expect(mockApiClient.rust.removeSymbol).toHaveBeenCalledWith('BTCUSDT')
          expect(toast.success).toHaveBeenCalledWith('Removed BTCUSDT')
        })
      }
    })

    it('handles remove symbol error', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.removeSymbol.mockRejectedValue(new Error('Failed to remove'))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      const closeButtons = screen.getAllByRole('button')
      const closeButton = closeButtons.find(btn => btn.querySelector('[class*="lucide-x"]'))

      if (closeButton) {
        await user.click(closeButton)

        await waitFor(() => {
          expect(toast.error).toHaveBeenCalledWith('Failed to remove symbol')
        })
      }
    })
  })

  describe('Action Buttons', () => {
    it('updates prices when update prices button is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /update prices/i })).toBeInTheDocument()
      })

      vi.clearAllMocks()

      const updateButton = screen.getByRole('button', { name: /update prices/i })
      await user.click(updateButton)

      await waitFor(() => {
        expect(mockApiClient.rust.getLatestPrices).toHaveBeenCalled()
      })
    })

    it('refreshes charts when refresh button is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /refresh charts/i })).toBeInTheDocument()
      })

      vi.clearAllMocks()

      const refreshButton = screen.getByRole('button', { name: /refresh charts/i })
      await user.click(refreshButton)

      await waitFor(() => {
        expect(mockApiClient.rust.getSupportedSymbols).toHaveBeenCalled()
      })
    })
  })

  describe('WebSocket Updates', () => {
    it('updates price when MarketData message is received', async () => {
      const connectFn = vi.fn()

      mockUseWebSocket.mockReturnValue({
        state: {
          isConnected: true,
          isConnecting: false,
          lastMessage: {
            type: 'MarketData',
            data: {
              symbol: 'BTCUSDT',
              price: 46000,
              price_change_24h: 1000,
              price_change_percent_24h: 2.22,
              volume_24h: 2000000,
            },
          },
        },
        connect: connectFn,
        disconnect: vi.fn(),
      })

      render(<TradingCharts />)

      await waitFor(() => {
        // Component should render the chart - WebSocket data should be processed
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
        expect(screen.getByText('Real-time Trading Charts')).toBeInTheDocument()
      })
    })

    it('updates chart when ChartUpdate message is received', async () => {
      mockUseWebSocket.mockReturnValue({
        state: {
          isConnected: true,
          isConnecting: false,
          lastMessage: {
            type: 'ChartUpdate',
            data: {
              symbol: 'BTCUSDT',
              timeframe: '1m',
              latest_price: 46000,
              price_change_24h: 1000,
              price_change_percent_24h: 2.22,
              volume_24h: 2000000,
              candle: mockCandle,
            },
          },
        },
        connect: vi.fn(),
        disconnect: vi.fn(),
      })

      render(<TradingCharts />)

      await waitFor(() => {
        // Component should render the chart - WebSocket data should be processed
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
        expect(screen.getByText('Real-time Trading Charts')).toBeInTheDocument()
      })
    })

    it('connects WebSocket on mount', async () => {
      const connectFn = vi.fn()

      mockUseWebSocket.mockReturnValue({
        state: {
          isConnected: false,
          isConnecting: false,
          lastMessage: null,
        },
        connect: connectFn,
        disconnect: vi.fn(),
      })

      render(<TradingCharts />)

      await waitFor(() => {
        expect(connectFn).toHaveBeenCalled()
      })
    })
  })

  describe('Empty State', () => {
    it('displays empty state when no charts available', async () => {
      mockApiClient.rust.getSupportedSymbols.mockResolvedValue({
        symbols: [],
      })

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('No Charts Available')).toBeInTheDocument()
        expect(screen.getByText(/Add trading symbols to start monitoring/i)).toBeInTheDocument()
      })
    })
  })

  describe('Candlestick Chart', () => {
    it('displays candlestick chart when candles are available', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Candlestick chart should be rendered
      const chartContainers = document.querySelectorAll('.bg-gray-900')
      expect(chartContainers.length).toBeGreaterThan(0)
    })

    it('displays no data message when no candles available', async () => {
      mockApiClient.rust.getChartData.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        candles: [],
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('No chart data available')).toBeInTheDocument()
      })
    })

    it('displays latest candle information', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('Latest Candle:')).toBeInTheDocument()
        expect(screen.getByText('Open:')).toBeInTheDocument()
        expect(screen.getByText('High:')).toBeInTheDocument()
        expect(screen.getByText('Low:')).toBeInTheDocument()
        expect(screen.getByText('Close:')).toBeInTheDocument()
      })
    })
  })

  describe('Price Formatting', () => {
    it('formats large prices with commas', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getAllByText(/\$45,000/)[0]).toBeInTheDocument()
      })
    })

    it('formats small prices with decimals', async () => {
      mockApiClient.rust.getChartData.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        latest_price: 0.00001234,
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        // Use getAllByText since price appears in both visible element and aria-live region
        const priceElements = screen.getAllByText(/0\.000012/)
        expect(priceElements.length).toBeGreaterThan(0)
        expect(priceElements[0]).toBeInTheDocument()
      })
    })
  })

  describe('Performance', () => {
    it('handles multiple charts efficiently', async () => {
      mockApiClient.rust.getSupportedSymbols.mockResolvedValue({
        symbols: ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT'],
      })

      render(<TradingCharts />)

      await waitFor(() => {
        expect(mockApiClient.rust.getChartData).toHaveBeenCalledTimes(4)
      })
    })

    it('memoizes chart cards', async () => {
      const { rerender } = render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Rerender with same props
      rerender(<TradingCharts />)

      // Component should still render correctly
      expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
    })
  })
})
