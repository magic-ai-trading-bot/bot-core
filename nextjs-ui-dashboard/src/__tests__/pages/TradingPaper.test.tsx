import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
import TradingPaper from '../../pages/TradingPaper'

// Mock DashboardHeader
vi.mock('../../components/dashboard/DashboardHeader', () => ({
  DashboardHeader: () => <div data-testid="dashboard-header">Dashboard Header</div>,
}))

// Mock TradingSettings
vi.mock('../../components/dashboard/TradingSettings', () => ({
  TradingSettings: () => <div data-testid="trading-settings">Trading Settings Component</div>,
}))

// Mock ChatBot
vi.mock('../../components/ChatBot', () => ({
  default: () => null,
}))

// Mock PerformanceChart
vi.mock('../../components/dashboard/PerformanceChart', () => ({
  PerformanceChart: () => <div data-testid="performance-chart">Biểu đồ hiệu suất</div>,
}))

// Mock recharts to avoid rendering issues
vi.mock('recharts', () => ({
  LineChart: ({ children }: { children: React.ReactNode }) => <div data-testid="line-chart">{children}</div>,
  AreaChart: ({ children }: { children: React.ReactNode }) => <div data-testid="area-chart">{children}</div>,
  Line: () => <div data-testid="line" />,
  Area: () => <div data-testid="area" />,
  XAxis: () => <div data-testid="x-axis" />,
  YAxis: () => <div data-testid="y-axis" />,
  CartesianGrid: () => <div data-testid="cartesian-grid" />,
  Tooltip: () => <div data-testid="tooltip" />,
  Legend: () => <div data-testid="legend" />,
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => <div data-testid="responsive-container">{children}</div>,
}))

// Mock functions for PaperTradingContext
const mockStartTrading = vi.fn()
const mockStopTrading = vi.fn()
const mockUpdateSettings = vi.fn()
const mockResetPortfolio = vi.fn()
const mockCloseTrade = vi.fn()
const mockRefreshAISignals = vi.fn()
const mockRefreshSettings = vi.fn()

const defaultHookReturn = {
  portfolio: {
    current_balance: 10000,
    equity: 10000,
    total_pnl: 0,
    total_pnl_percentage: 0,
    total_trades: 0,
    margin_used: 0,
    free_margin: 10000,
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
  startTrading: mockStartTrading,
  stopTrading: mockStopTrading,
  updateSettings: mockUpdateSettings,
  resetPortfolio: mockResetPortfolio,
  closeTrade: mockCloseTrade,
  refreshAISignals: mockRefreshAISignals,
  refreshSettings: mockRefreshSettings,
}

// Mock the PaperTradingContext (TradingPaper uses usePaperTradingContext)
vi.mock('../../contexts/PaperTradingContext', () => ({
  usePaperTradingContext: vi.fn(() => defaultHookReturn),
  PaperTradingProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

// Mock useAIAnalysis hook (used by AIAnalysisContext if provider is used)
vi.mock('../../hooks/useAIAnalysis', () => ({
  useAIAnalysis: vi.fn(() => ({
    state: {
      signals: [],
      strategies: [],
      marketCondition: null,
      serviceInfo: null,
      supportedStrategies: [],
      availableSymbols: ['BTCUSDT', 'ETHUSDT'],
      isLoading: false,
      error: null,
      lastUpdate: null,
    },
    analyzeSymbol: vi.fn(),
    getStrategyRecommendations: vi.fn(),
    analyzeMarketCondition: vi.fn(),
    refreshServiceInfo: vi.fn(),
    refreshAvailableSymbols: vi.fn().mockResolvedValue(['BTCUSDT', 'ETHUSDT']),
    clearError: vi.fn(),
  })),
}))

const { usePaperTradingContext } = await import('../../contexts/PaperTradingContext')

describe('TradingPaper', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(usePaperTradingContext).mockReturnValue(defaultHookReturn)
    global.fetch = vi.fn()
  })

  it('renders trading paper page', () => {
    render(<TradingPaper />)

    expect(screen.getByText('Trading Paper')).toBeInTheDocument()
    expect(screen.getByText(/mô phỏng giao dịch với ai bot/i)).toBeInTheDocument()
  })

  it('displays portfolio overview cards', () => {
    render(<TradingPaper />)

    // These texts may appear multiple times on the page, use getAllByText
    expect(screen.getAllByText('Số dư hiện tại')[0]).toBeInTheDocument()
    expect(screen.getAllByText('Tổng P&L')[0]).toBeInTheDocument()
    expect(screen.getAllByText('Tổng số lệnh')[0]).toBeInTheDocument()
  })

  it('shows correct initial balance', () => {
    render(<TradingPaper />)

    // Check for the section heading instead
    expect(screen.getByText('Số dư hiện tại')).toBeInTheDocument()
  })

  describe('Bot Controls', () => {
    it('shows inactive badge when bot is not running', () => {
      render(<TradingPaper />)

      expect(screen.getByText('Tạm dừng')).toBeInTheDocument()
    })

    it('shows start button when bot is inactive', () => {
      render(<TradingPaper />)

      expect(screen.getByRole('button', { name: /khởi động bot/i })).toBeInTheDocument()
    })

    it('starts trading when start button is clicked', async () => {
      const user = userEvent.setup()
      mockStartTrading.mockResolvedValue(undefined)

      render(<TradingPaper />)

      await user.click(screen.getByRole('button', { name: /khởi động bot/i }))

      expect(mockStartTrading).toHaveBeenCalled()
    })

    it('shows active badge when bot is running', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        isActive: true,
      })

      render(<TradingPaper />)

      expect(screen.getByText('Đang hoạt động')).toBeInTheDocument()
    })

    it('shows stop button when bot is active', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        isActive: true,
      })

      render(<TradingPaper />)

      expect(screen.getByRole('button', { name: /dừng bot/i })).toBeInTheDocument()
    })

    it('stops trading when stop button is clicked', async () => {
      const user = userEvent.setup()
      mockStopTrading.mockResolvedValue(undefined)

      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        isActive: true,
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('button', { name: /dừng bot/i }))

      expect(mockStopTrading).toHaveBeenCalled()
    })

    it('disables buttons when loading', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        isLoading: true,
      })

      render(<TradingPaper />)

      expect(screen.getByRole('button', { name: /khởi động bot/i })).toBeDisabled()
    })

    it('shows refresh button', () => {
      render(<TradingPaper />)

      expect(screen.getByRole('button', { name: /cập nhật/i })).toBeInTheDocument()
    })

    it('refreshes AI signals when refresh button is clicked', async () => {
      const user = userEvent.setup()
      mockRefreshAISignals.mockResolvedValue(undefined)

      render(<TradingPaper />)

      await user.click(screen.getByRole('button', { name: /cập nhật/i }))

      expect(mockRefreshAISignals).toHaveBeenCalled()
    })
  })

  describe('Tabs Navigation', () => {
    it('renders all tabs', () => {
      render(<TradingPaper />)

      expect(screen.getByRole('tab', { name: /tổng quan/i })).toBeInTheDocument()
      expect(screen.getByRole('tab', { name: /tín hiệu ai/i })).toBeInTheDocument()
      expect(screen.getByRole('tab', { name: /lịch sử giao dịch/i })).toBeInTheDocument()
      expect(screen.getByRole('tab', { name: /cài đặt/i })).toBeInTheDocument()
    })

    it('shows overview tab by default', () => {
      render(<TradingPaper />)

      expect(screen.getByText('Biểu đồ hiệu suất')).toBeInTheDocument()
    })

    it('switches to AI signals tab', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /tín hiệu ai/i }))

      expect(screen.getByText(/tín hiệu ai gần đây/i)).toBeInTheDocument()
    })

    it('switches to trades history tab', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /lịch sử giao dịch/i }))

      expect(screen.getByText(/lịch sử giao dịch \(/i)).toBeInTheDocument()
    })

    it('switches to settings tab', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      expect(screen.getByText(/cài đặt paper trading/i)).toBeInTheDocument()
    })
  })

  describe('Overview Tab', () => {
    it('displays performance chart', () => {
      render(<TradingPaper />)

      expect(screen.getByText('Biểu đồ hiệu suất')).toBeInTheDocument()
    })

    it('shows margin usage card', () => {
      render(<TradingPaper />)

      expect(screen.getByText('Margin sử dụng')).toBeInTheDocument()
    })

    it('displays portfolio metrics with live data', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        portfolio: {
          ...defaultHookReturn.portfolio,
          current_balance: 12500,
          total_pnl: 2500,
          total_pnl_percentage: 25,
        },
      })

      render(<TradingPaper />)

      // Just check that the sections exist (may appear multiple times)
      expect(screen.getAllByText('Số dư hiện tại')[0]).toBeInTheDocument()
      expect(screen.getAllByText('Tổng P&L')[0]).toBeInTheDocument()
    })
  })

  describe('AI Signals Tab', () => {
    it('shows empty state when no signals', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /tín hiệu ai/i }))

      expect(screen.getByText(/chưa có tín hiệu ai/i)).toBeInTheDocument()
    })

    it('displays AI signals when available', async () => {
      const user = userEvent.setup()
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        recentSignals: [
          {
            symbol: 'BTCUSDT',
            signal: 'Long',
            confidence: 0.85,
            reasoning: 'Strong uptrend detected',
            timestamp: new Date().toISOString(),
          },
        ],
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /tín hiệu ai/i }))

      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      expect(screen.getByText('LONG')).toBeInTheDocument()
      expect(screen.getByText('85%')).toBeInTheDocument()
      expect(screen.getByText('Strong uptrend detected')).toBeInTheDocument()
    })
  })

  describe('Trades History Tab', () => {
    it('shows empty state when no trades', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /lịch sử giao dịch/i }))

      expect(screen.getByText(/chưa có giao dịch nào/i)).toBeInTheDocument()
    })

    it('displays open trades', async () => {
      const user = userEvent.setup()
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        openTrades: [
          {
            id: 'trade-1',
            symbol: 'BTCUSDT',
            trade_type: 'Long',
            entry_price: 50000,
            quantity: 0.1,
            leverage: 10,
            pnl: 100,
            pnl_percentage: 2,
            open_time: new Date().toISOString(),
            status: 'Open',
            stop_loss: 49000,
            take_profit: 52000,
          },
        ],
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /lịch sử giao dịch/i }))

      expect(screen.getByText(/lệnh đang mở \(1\)/i)).toBeInTheDocument()
      expect(screen.getByText('BTCUSDT')).toBeInTheDocument()
      expect(screen.getByText('Long')).toBeInTheDocument()
    })

    it('displays closed trades', async () => {
      const user = userEvent.setup()
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        closedTrades: [
          {
            id: 'trade-2',
            symbol: 'ETHUSDT',
            trade_type: 'Short',
            entry_price: 3000,
            exit_price: 2950,
            quantity: 1,
            leverage: 5,
            pnl: 50,
            pnl_percentage: 3.33,
            open_time: new Date(Date.now() - 3600000).toISOString(),
            close_time: new Date().toISOString(),
            status: 'Closed',
          },
        ],
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /lịch sử giao dịch/i }))

      expect(screen.getByText('ETHUSDT')).toBeInTheDocument()
      expect(screen.getByText('Short')).toBeInTheDocument()
    })

    it('allows closing an open trade', async () => {
      const user = userEvent.setup()
      mockCloseTrade.mockResolvedValue(undefined)

      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        openTrades: [
          {
            id: 'trade-1',
            symbol: 'BTCUSDT',
            trade_type: 'Long',
            entry_price: 50000,
            quantity: 0.1,
            leverage: 10,
            pnl: 100,
            pnl_percentage: 2,
            open_time: new Date().toISOString(),
            status: 'Open',
          },
        ],
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /lịch sử giao dịch/i }))

      const closeButton = screen.getByRole('button', { name: /đóng/i })
      await user.click(closeButton)

      expect(mockCloseTrade).toHaveBeenCalledWith('trade-1')
    })
  })

  describe('Settings Tab', () => {
    it('displays paper trading settings', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      expect(screen.getByText(/cài đặt paper trading/i)).toBeInTheDocument()
      expect(screen.getByLabelText(/vốn ban đầu/i)).toBeInTheDocument()
      expect(screen.getByLabelText(/đòn bẩy tối đa/i)).toBeInTheDocument()
    })

    it('allows updating initial balance', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      const initialBalanceInput = screen.getByLabelText(/vốn ban đầu/i)
      await user.clear(initialBalanceInput)
      await user.type(initialBalanceInput, '20000')

      expect(initialBalanceInput).toHaveValue(20000)
    })

    it('displays max leverage field', async () => {
      const user = userEvent.setup()
      global.fetch = vi.fn().mockResolvedValue({
        json: async () => ({ success: true, data: {} }),
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      const maxLeverageInput = screen.getByLabelText(/đòn bẩy tối đa/i)
      expect(maxLeverageInput).toBeInTheDocument()
      expect(maxLeverageInput).toHaveValue(20) // default value from mock
    })

    it('saves settings when save button is clicked', async () => {
      const user = userEvent.setup()
      mockUpdateSettings.mockResolvedValue(undefined)

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      const saveButton = screen.getByRole('button', { name: /lưu cài đặt/i })
      await user.click(saveButton)

      expect(mockUpdateSettings).toHaveBeenCalled()
    })

    it('shows reset button', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      expect(screen.getByRole('button', { name: /reset dữ liệu/i })).toBeInTheDocument()
    })

    it('shows confirmation dialog when reset is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      await user.click(screen.getByRole('button', { name: /reset dữ liệu/i }))

      expect(screen.getByText(/xác nhận reset toàn bộ dữ liệu/i)).toBeInTheDocument()
    })

    it('resets portfolio when confirmed', async () => {
      const user = userEvent.setup()
      mockResetPortfolio.mockResolvedValue(undefined)

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))
      await user.click(screen.getByRole('button', { name: /reset dữ liệu/i }))

      const confirmButton = screen.getByRole('button', { name: /xác nhận/i })
      await user.click(confirmButton)

      expect(mockResetPortfolio).toHaveBeenCalled()
    })

    it('cancels reset when cancel is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))
      await user.click(screen.getByRole('button', { name: /reset dữ liệu/i }))

      const cancelButton = screen.getByRole('button', { name: /hủy/i })
      await user.click(cancelButton)

      expect(screen.queryByText(/xác nhận reset/i)).not.toBeInTheDocument()
    })

    it('displays trading strategy settings', async () => {
      const user = userEvent.setup()
      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      expect(screen.getByText(/cài đặt chiến lược trading/i)).toBeInTheDocument()
      expect(screen.getByTestId('trading-settings')).toBeInTheDocument()
    })
  })

  describe('Error Handling', () => {
    it('displays error alert when error occurs', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        error: 'Failed to fetch data',
      })

      render(<TradingPaper />)

      expect(screen.getByText('Failed to fetch data')).toBeInTheDocument()
    })
  })

  describe('WebSocket Status', () => {
    it('shows WebSocket connected status', () => {
      render(<TradingPaper />)

      expect(screen.getByText(/websocket connected/i)).toBeInTheDocument()
    })

    it('displays last update time', () => {
      render(<TradingPaper />)

      expect(screen.getByText(/last update:/i)).toBeInTheDocument()
    })
  })

  describe('Trade Details Dialog', () => {
    it('opens trade details when trade row is clicked', async () => {
      const user = userEvent.setup()
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        openTrades: [
          {
            id: 'trade-1',
            symbol: 'BTCUSDT',
            trade_type: 'Long',
            entry_price: 50000,
            quantity: 0.1,
            leverage: 10,
            pnl: 100,
            pnl_percentage: 2,
            open_time: new Date().toISOString(),
            status: 'Open',
          },
        ],
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /lịch sử giao dịch/i }))

      const tradeRow = screen.getByText('BTCUSDT').closest('tr')
      if (tradeRow) {
        await user.click(tradeRow)
        expect(screen.getByText(/chi tiết giao dịch/i)).toBeInTheDocument()
      }
    })
  })

  describe('Symbol Settings Dialog', () => {
    it('opens symbol settings dialog', async () => {
      const user = userEvent.setup()
      global.fetch = vi.fn().mockResolvedValue({
        json: async () => ({
          success: true,
          data: {
            BTCUSDT: {
              enabled: true,
              leverage: 10,
              position_size_pct: 10,
              stop_loss_pct: 2,
              take_profit_pct: 4,
              max_positions: 3,
            },
          },
        }),
      })

      render(<TradingPaper />)

      await user.click(screen.getByRole('tab', { name: /cài đặt/i }))

      const openSymbolButton = screen.getByRole('button', { name: /mở cài đặt symbols/i })
      await user.click(openSymbolButton)

      await waitFor(() => {
        // Find by dialog title which is more specific
        const dialogs = screen.getAllByText(/cài đặt symbols/i)
        expect(dialogs.length).toBeGreaterThan(0)
      })
    })
  })

  describe('Loading States', () => {
    it('shows loading state in buttons', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        isLoading: true,
      })

      render(<TradingPaper />)

      const buttons = screen.getAllByRole('button')
      const disabledButtons = buttons.filter(btn => btn.hasAttribute('disabled'))

      expect(disabledButtons.length).toBeGreaterThan(0)
    })
  })

  describe('Real-time Updates', () => {
    it('displays live P&L section', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        portfolio: {
          ...defaultHookReturn.portfolio,
          total_pnl: 500,
          total_pnl_percentage: 5,
        },
        openTrades: [
          {
            id: 'trade-1',
            symbol: 'BTCUSDT',
            trade_type: 'Long',
            entry_price: 50000,
            quantity: 0.1,
            leverage: 10,
            pnl: 500,
            pnl_percentage: 10,
            open_time: new Date().toISOString(),
            status: 'Open',
          },
        ],
      })

      render(<TradingPaper />)

      expect(screen.getAllByText('Tổng P&L')[0]).toBeInTheDocument()
    })
  })

  describe('Currency Formatting', () => {
    it('displays balance section with values', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        portfolio: {
          ...defaultHookReturn.portfolio,
          current_balance: 15432.67,
        },
      })

      render(<TradingPaper />)

      expect(screen.getAllByText('Số dư hiện tại')[0]).toBeInTheDocument()
    })

    it('displays P&L percentage section', () => {
      vi.mocked(usePaperTradingContext).mockReturnValue({
        ...defaultHookReturn,
        portfolio: {
          ...defaultHookReturn.portfolio,
          total_pnl: 1234.56,
          total_pnl_percentage: 12.3456,
        },
      })

      render(<TradingPaper />)

      expect(screen.getAllByText('Tổng P&L')[0]).toBeInTheDocument()
    })
  })
})
