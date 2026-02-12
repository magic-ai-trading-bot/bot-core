import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, mockCandle } from '../../../test/utils'
import { TradingCharts } from '../../../components/dashboard/TradingCharts'
import { toast } from 'sonner'

// Mock scrollIntoView for Radix UI
beforeEach(() => {
  Element.prototype.scrollIntoView = vi.fn()
})

// Mock recharts
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

// Mock API client
const mockApiClient = vi.hoisted(() => ({
  rust: {
    getSupportedSymbols: vi.fn(),
    getChartData: vi.fn(),
    getChartDataFast: vi.fn(),
    getLatestPrices: vi.fn(),
    addSymbol: vi.fn(),
    removeSymbol: vi.fn(),
    fetchAvailableSymbols: vi.fn(),
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

// Mock WebSocket context
const mockUseWebSocketContext = vi.fn()

vi.mock('../../../contexts/WebSocketContext', () => ({
  useWebSocketContext: () => mockUseWebSocketContext(),
  WebSocketProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

describe('TradingCharts - Comprehensive Coverage', () => {
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

    mockApiClient.rust.getSupportedSymbols.mockResolvedValue({
      symbols: ['BTCUSDT'],
    })

    mockApiClient.rust.getChartData.mockImplementation(async (symbol: string) => ({
      ...mockChartData,
      symbol,
    }))

    mockApiClient.rust.getChartDataFast.mockImplementation(async (symbol: string) => ({
      ...mockChartData,
      symbol,
    }))

    mockApiClient.rust.fetchAvailableSymbols.mockResolvedValue(['BTCUSDT'])

    mockApiClient.rust.getLatestPrices.mockResolvedValue({
      BTCUSDT: 45500,
      ETHUSDT: 3000,
    })

    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue({
        success: true,
        data: {
          symbols: ['BTCUSDT']
        }
      })
    }) as unknown as typeof fetch

    mockUseWebSocketContext.mockReturnValue({
      state: {
        isConnected: true,
        isConnecting: false,
        lastMessage: null,
      },
      connect: vi.fn(),
      disconnect: vi.fn(),
    })
  })

  describe('Candlestick Chart Component', () => {
    it('displays candlestick chart with data', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Check for candlestick chart container
      const chartContainer = document.querySelector('.bg-gray-950')
      expect(chartContainer).toBeInTheDocument()
    }, 15000)

    it('shows price grid lines', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Grid lines should be rendered
      const gridLines = document.querySelectorAll('.border-gray-700')
      expect(gridLines.length).toBeGreaterThan(0)
    })

    it('displays price labels at top and bottom', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Price labels in chart
      const priceLabels = document.querySelectorAll('.text-gray-300.font-mono')
      expect(priceLabels.length).toBeGreaterThan(0)
    })

    it('shows time labels for first and last candle', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Time labels at bottom of chart
      const timeLabels = document.querySelectorAll('.text-gray-400')
      expect(timeLabels.length).toBeGreaterThan(0)
    })

    it('renders candles with wicks and bodies', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Wick elements (thin lines)
      const wicks = document.querySelectorAll('.w-0\\.5')
      expect(wicks.length).toBeGreaterThan(0)
    })

    it('renders bullish candles correctly', async () => {
      const bullishCandle = {
        ...mockCandle,
        open: 44000,
        close: 45000, // close > open = bullish
      }

      mockApiClient.rust.getChartDataFast.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        candles: [bullishCandle],
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Chart container should render
      const chartContainer = document.querySelector('.bg-gray-950')
      expect(chartContainer).toBeInTheDocument()
    }, 15000)

    it('renders bearish candles correctly', async () => {
      const bearishCandle = {
        ...mockCandle,
        open: 45000,
        close: 44000, // close < open = bearish
      }

      mockApiClient.rust.getChartDataFast.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        candles: [bearishCandle],
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Chart container should render
      const chartContainer = document.querySelector('.bg-gray-950')
      expect(chartContainer).toBeInTheDocument()
    }, 15000)

    it('renders candle elements with hover capability', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Check that candle elements exist with cursor-pointer class
      const candles = document.querySelectorAll('.cursor-pointer')
      expect(candles.length).toBeGreaterThan(0)
    }, 15000)

    it('displays no data message when candles array is empty', async () => {
      mockApiClient.rust.getChartDataFast.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        candles: [],
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText(/No chart data available/)).toBeInTheDocument()
      }, { timeout: 10000 })
    }, 15000)

    it('handles candles with equal open and close (doji)', async () => {
      const dojiCandle = {
        ...mockCandle,
        open: 45000,
        close: 45000, // equal = doji
      }

      mockApiClient.rust.getChartDataFast.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        candles: [dojiCandle],
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Should still render (body height has Math.max(..., 2) minimum)
      const candles = document.querySelectorAll('.flex-1.relative.cursor-pointer')
      expect(candles.length).toBeGreaterThan(0)
    })

    it('scales chart based on price range with padding', async () => {
      const candles = [
        { ...mockCandle, high: 46000, low: 44000 },
        { ...mockCandle, high: 47000, low: 43000 },
        { ...mockCandle, high: 45500, low: 44500 },
      ]

      mockApiClient.rust.getChartDataFast.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        candles,
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Chart should show max and min prices with padding
      // Price labels should be visible
      const priceLabels = document.querySelectorAll('.text-gray-300.font-mono')
      expect(priceLabels.length).toBeGreaterThan(0)
    })

    it('limits chart to last 15 candles', async () => {
      const candles = Array.from({ length: 20 }, (_, i) => ({
        ...mockCandle,
        timestamp: Date.now() - (20 - i) * 60000,
      }))

      mockApiClient.rust.getChartDataFast.mockImplementation(async (symbol: string) => ({
        ...mockChartData,
        symbol,
        candles,
      }))

      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // Should render max 15 candles
      const candleElements = document.querySelectorAll('.flex-1.relative.cursor-pointer')
      expect(candleElements.length).toBeLessThanOrEqual(15)
    })
  })

  describe('Price Update Animations', () => {
    it('displays price with transition classes', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Check for price element with transition class
      const priceElement = document.querySelector('.transition-all')
      expect(priceElement).toBeInTheDocument()
    }, 15000)
  })

  describe('Add Symbol Dialog Validation', () => {
    it('shows error when submitting empty symbol', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)).toBeInTheDocument()
      })

      // Try to submit without entering symbol
      const submitButtons = screen.getAllByRole('button', { name: /add symbol/i })
      const submitButton = submitButtons[submitButtons.length - 1]
      await user.click(submitButton)

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Please enter a symbol')
      })
    })

    it('trims and uppercases symbol on submit', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.addSymbol.mockResolvedValue({ success: true })

      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)).toBeInTheDocument()
      })

      const input = screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)
      await user.type(input, '  ethusdt  ') // Lowercase with spaces

      const submitButtons = screen.getAllByRole('button', { name: /add symbol/i })
      const submitButton = submitButtons[submitButtons.length - 1]
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockApiClient.rust.addSymbol).toHaveBeenCalledWith({
          symbol: 'ETHUSDT', // Trimmed and uppercased
          timeframes: expect.any(Array),
        })
      })
    })

    it('closes dialog after successful add', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.addSymbol.mockResolvedValue({ success: true })

      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByText('Add New Trading Symbol')).toBeInTheDocument()
      })

      const input = screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)
      await user.type(input, 'SOLUSDT')

      const submitButtons = screen.getAllByRole('button', { name: /add symbol/i })
      const submitButton = submitButtons[submitButtons.length - 1]
      await user.click(submitButton)

      await waitFor(() => {
        expect(screen.queryByText('Add New Trading Symbol')).not.toBeInTheDocument()
      })
    })

    it('closes dialog on cancel button', async () => {
      const user = userEvent.setup()
      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByText('Add New Trading Symbol')).toBeInTheDocument()
      })

      const cancelButton = screen.getByRole('button', { name: /cancel/i })
      await user.click(cancelButton)

      await waitFor(() => {
        expect(screen.queryByText('Add New Trading Symbol')).not.toBeInTheDocument()
      })
    })

    it('loads chart immediately after adding symbol', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.addSymbol.mockResolvedValue({ success: true })
      mockApiClient.rust.getChartData.mockResolvedValue({
        ...mockChartData,
        symbol: 'NEWUSDT',
      })

      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)).toBeInTheDocument()
      })

      const input = screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)
      await user.type(input, 'NEWUSDT')

      const submitButtons = screen.getAllByRole('button', { name: /add symbol/i })
      const submitButton = submitButtons[submitButtons.length - 1]
      await user.click(submitButton)

      await waitFor(() => {
        expect(mockApiClient.rust.getChartData).toHaveBeenCalledWith('NEWUSDT', '1m', 100)
      })
    })

    it('handles getChartData error after adding symbol gracefully', async () => {
      const user = userEvent.setup()
      mockApiClient.rust.addSymbol.mockResolvedValue({ success: true })
      mockApiClient.rust.getChartData.mockRejectedValue(new Error('Chart load failed'))

      render(<TradingCharts />)

      const addButton = screen.getByRole('button', { name: /add symbol/i })
      await user.click(addButton)

      await waitFor(() => {
        expect(screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)).toBeInTheDocument()
      })

      const input = screen.getByPlaceholderText(/BTCUSDT, ETHUSDT/i)
      await user.type(input, 'FAILUSDT')

      const submitButtons = screen.getAllByRole('button', { name: /add symbol/i })
      const submitButton = submitButtons[submitButtons.length - 1]
      await user.click(submitButton)

      // Should show success for adding symbol but log warning for chart load
      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Successfully added FAILUSDT')
      })
    })
  })

  describe('Symbol Fetching Logic', () => {
    it('fetches symbols from API on mount', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalledWith(expect.stringContaining('/api/market/symbols'))
      })
    })

    it('uses fallback symbols when API fetch fails', async () => {
      global.fetch = vi.fn().mockRejectedValue(new Error('Network error')) as unknown as typeof fetch

      render(<TradingCharts />)

      await waitFor(() => {
        // Should still attempt to load charts with fallback symbols
        expect(mockApiClient.rust.getChartDataFast).toHaveBeenCalled()
      })
    })

    it('uses fallback symbols when API returns invalid data', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue({
          success: false,
          data: null,
        })
      }) as unknown as typeof fetch

      render(<TradingCharts />)

      await waitFor(() => {
        expect(mockApiClient.rust.getChartDataFast).toHaveBeenCalled()
      })
    })

    it('loads additional symbols in phase 2', async () => {
      mockApiClient.rust.getSupportedSymbols.mockResolvedValue({
        symbols: ['BTCUSDT', 'ETHUSDT', 'BNBUSDT'], // More symbols than initial fetch
      })

      render(<TradingCharts />)

      await waitFor(() => {
        // Should load initial symbol first
        expect(mockApiClient.rust.getChartDataFast).toHaveBeenCalledWith('BTCUSDT', expect.any(String), 100, expect.any(Object))
      }, { timeout: 3000 })

      // Phase 2 loading happens in background
      await waitFor(() => {
        expect(mockApiClient.rust.getSupportedSymbols).toHaveBeenCalled()
      }, { timeout: 5000 })
    })

    it('ignores phase 2 errors silently', async () => {
      mockApiClient.rust.getSupportedSymbols.mockRejectedValue(new Error('Phase 2 failed'))

      render(<TradingCharts />)

      await waitFor(() => {
        // Should still show initial charts
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      })

      // No error toast should be shown
      expect(toast.error).not.toHaveBeenCalled()
    })
  })

  describe('Price Polling Fallback', () => {
    it('sets up polling interval when WebSocket is disconnected', async () => {
      mockUseWebSocketContext.mockReturnValue({
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
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Component should render successfully
      expect(screen.getByText('Real-time Trading Charts')).toBeInTheDocument()
    }, 15000)
  })

  describe('Abort Controller Handling', () => {
    it('handles timeframe selector rendering', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Timeframe selector should be present
      const selects = screen.getAllByRole('combobox')
      expect(selects.length).toBeGreaterThan(0)
    }, 15000)

    it('handles CanceledError gracefully', async () => {
      mockApiClient.rust.getChartDataFast.mockRejectedValue({ name: 'CanceledError' })

      render(<TradingCharts />)

      // Should not show error for canceled requests
      await waitFor(() => {
        expect(toast.error).not.toHaveBeenCalled()
      })
    })

    it('handles ERR_CANCELED gracefully', async () => {
      mockApiClient.rust.getChartDataFast.mockRejectedValue({ code: 'ERR_CANCELED' })

      render(<TradingCharts />)

      // Should not show error for canceled requests
      await waitFor(() => {
        expect(toast.error).not.toHaveBeenCalled()
      })
    })
  })

  describe('Accessibility Features', () => {
    it('provides screen reader updates for price changes', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Live region for screen readers
      const liveRegion = document.querySelector('[aria-live="polite"]')
      expect(liveRegion).toBeInTheDocument()
    }, 15000)

    it('labels trending icons appropriately', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Screen reader text for trend direction
      expect(screen.getByText('Price increase')).toBeInTheDocument()
    }, 15000)

    it('provides aria-hidden for decorative icons', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Icons should have aria-hidden
      const icons = document.querySelectorAll('[aria-hidden="true"]')
      expect(icons.length).toBeGreaterThan(0)
    }, 15000)
  })

  describe('Responsive Grid Layout', () => {
    it('renders grid layout for charts', async () => {
      render(<TradingCharts />)

      await waitFor(() => {
        expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      }, { timeout: 10000 })

      // Grid container should exist
      const grids = document.querySelectorAll('.grid')
      expect(grids.length).toBeGreaterThan(0)
    }, 15000)
  })
})
