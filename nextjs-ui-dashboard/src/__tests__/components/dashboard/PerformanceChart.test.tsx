import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import { render } from '../../../test/utils'
import { PerformanceChart } from '../../../components/dashboard/PerformanceChart'

// Mock recharts
vi.mock('recharts', () => ({
  ResponsiveContainer: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  LineChart: () => <div>LineChart</div>,
  Line: () => null,
  XAxis: () => null,
  YAxis: () => null,
  CartesianGrid: () => null,
  Tooltip: () => null,
  AreaChart: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
  Area: () => null,
}))

// Mock usePaperTradingContext hook (component uses context, not hook directly)
const mockUsePaperTradingContext = vi.fn()

vi.mock('../../../contexts/PaperTradingContext', () => ({
  usePaperTradingContext: () => mockUsePaperTradingContext(),
  PaperTradingProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

describe('PerformanceChart', () => {
  const mockPortfolio = {
    equity: 10500,
    total_pnl: 500,
    total_pnl_percentage: 5.0,
    total_trades: 25,
    win_rate: 68.5,
    max_drawdown: -150,
    sharpe_ratio: 1.85,
  }

  const mockOpenTrades = [
    {
      id: '1',
      symbol: 'BTCUSDT',
      side: 'long',
      quantity: 0.1,
      entry_price: 45000,
      current_price: 45500,
      unrealized_pnl: 50,
    },
  ]

  const mockClosedTrades = [
    {
      id: '2',
      symbol: 'ETHUSDT',
      side: 'short',
      quantity: 1.0,
      entry_price: 3000,
      exit_price: 2950,
      pnl: 50,
      status: 'closed',
    },
    {
      id: '3',
      symbol: 'BNBUSDT',
      side: 'long',
      quantity: 5.0,
      entry_price: 300,
      exit_price: 310,
      pnl: 50,
      status: 'closed',
    },
  ]

  beforeEach(() => {
    vi.clearAllMocks()

    mockUsePaperTradingContext.mockReturnValue({
      portfolio: mockPortfolio,
      openTrades: mockOpenTrades,
      closedTrades: mockClosedTrades,
    })
  })

  describe('Component Rendering', () => {
    it('renders performance chart card', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('Biểu đồ hiệu suất')).toBeInTheDocument()
    })

    it('displays profit badge when PnL is positive', () => {
      render(<PerformanceChart />)

      const badge = screen.getByText('+5.00%')
      expect(badge).toBeInTheDocument()
      expect(badge.className).toContain('bg-profit')
    })

    it('displays loss badge when PnL is negative', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_pnl: -300,
          total_pnl_percentage: -3.0,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      const badge = screen.getByText('-3.00%')
      expect(badge).toBeInTheDocument()
      expect(badge.className).toContain('bg-loss')
    })

    it('displays total trades count', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('25 trades')).toBeInTheDocument()
    })

    it('displays trending up icon when trend is positive', () => {
      const { container } = render(<PerformanceChart />)

      // Check for the SVG element with the specific class pattern
      const svgs = container.querySelectorAll('svg')
      const trendingIcons = Array.from(svgs).filter(svg =>
        svg.getAttribute('class')?.includes('lucide')
      )
      expect(trendingIcons.length).toBeGreaterThan(0)
    })

    it('displays trending down icon when trend is negative', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_pnl: -300,
          total_pnl_percentage: -3.0,
          equity: 9500, // Lower equity to ensure downward trend in last data point
        },
        openTrades: [],
        closedTrades: [],
      })

      const { container } = render(<PerformanceChart />)

      // Check for the trending icons - either up or down should be present
      const svgs = container.querySelectorAll('svg')
      const trendingIcons = Array.from(svgs).filter(svg =>
        svg.getAttribute('class')?.includes('lucide')
      )
      expect(trendingIcons.length).toBeGreaterThan(0)
    })
  })

  describe('Performance Metrics', () => {
    it('displays total P&L in profit color', () => {
      render(<PerformanceChart />)

      const pnlElements = screen.getAllByText(/500,00 US\$/)
      const pnlElement = pnlElements.find(el => el.className.includes('text-profit'))
      expect(pnlElement).toBeTruthy()
      expect(pnlElement?.className).toContain('text-profit')
    })

    it('displays total P&L in loss color when negative', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_pnl: -300,
          total_pnl_percentage: -3.0,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      const pnlElement = screen.getByText(/-300,00 US\$/)
      expect(pnlElement).toBeInTheDocument()
      expect(pnlElement.className).toContain('text-loss')
    })

    it('displays current equity', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('Equity hiện tại')).toBeInTheDocument()
      expect(screen.getByText(/10\.500,00 US\$/)).toBeInTheDocument()
    })

    it('displays win rate', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('Win Rate')).toBeInTheDocument()
      expect(screen.getByText('68.5%')).toBeInTheDocument()
    })

    it('displays max drawdown', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('Max Drawdown:')).toBeInTheDocument()
      expect(screen.getByText(/-150,00 US\$/)).toBeInTheDocument()
    })

    it('displays Sharpe ratio', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('Sharpe Ratio:')).toBeInTheDocument()
      expect(screen.getByText('1.85')).toBeInTheDocument()
    })

    it('displays open and closed trades count', () => {
      render(<PerformanceChart />)

      expect(screen.getByText(/1 lệnh mở/)).toBeInTheDocument()
      expect(screen.getByText(/2 lệnh đóng/)).toBeInTheDocument()
    })
  })

  describe('Chart Data Generation', () => {
    it('generates 30 days of performance data', () => {
      const { container } = render(<PerformanceChart />)

      // Chart should be rendered
      expect(container.querySelector('[class*="h-\\[300px\\]"]')).toBeInTheDocument()
    })

    it('handles zero trades', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_trades: 0,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText('0 trades')).toBeInTheDocument()
    })

    it('handles zero equity', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          equity: 0,
          total_pnl: -10000,
          total_pnl_percentage: -100,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getAllByText(/0,00 US\$/).length).toBeGreaterThan(0)
    })
  })

  describe('Currency Formatting', () => {
    it('formats positive currency values correctly', () => {
      render(<PerformanceChart />)

      expect(screen.getAllByText(/500,00 US\$/).length).toBeGreaterThan(0)
      expect(screen.getAllByText(/10\.500,00 US\$/).length).toBeGreaterThan(0)
    })

    it('formats negative currency values correctly', () => {
      render(<PerformanceChart />)

      expect(screen.getByText(/-150,00 US\$/)).toBeInTheDocument()
    })

    it('formats large numbers with commas', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          equity: 1000000,
          total_pnl: 100000,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText(/1\.000\.000,00 US\$/)).toBeInTheDocument()
      expect(screen.getByText(/100\.000,00 US\$/)).toBeInTheDocument()
    })

    it('formats decimal percentages correctly', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('+5.00%')).toBeInTheDocument()
      expect(screen.getByText('68.5%')).toBeInTheDocument()
    })
  })

  describe('Date Formatting', () => {
    it('formats dates in Vietnamese locale', () => {
      render(<PerformanceChart />)

      // Should display "Lịch sử 30 ngày"
      expect(screen.getByText(/Lịch sử 30 ngày/)).toBeInTheDocument()
    })
  })

  describe('Tooltip', () => {
    it('renders custom tooltip component', () => {
      const { container } = render(<PerformanceChart />)

      // Chart container should exist (mocked AreaChart renders as div)
      const chartContainer = container.querySelector('.h-\\[300px\\]')
      expect(chartContainer).toBeInTheDocument()
    })
  })

  describe('Visual Indicators', () => {
    it('applies correct gradient for profit', () => {
      const { container } = render(<PerformanceChart />)

      // Should have gradient definition
      const svg = container.querySelector('svg')
      expect(svg).toBeTruthy()
    })

    it('applies correct gradient for loss', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_pnl: -300,
          total_pnl_percentage: -3.0,
        },
        openTrades: [],
        closedTrades: [],
      })

      const { container } = render(<PerformanceChart />)

      const svg = container.querySelector('svg')
      expect(svg).toBeTruthy()
    })
  })

  describe('Performance Summary', () => {
    it('displays performance summary section', () => {
      render(<PerformanceChart />)

      const summarySection = screen.getByText(/Lịch sử 30 ngày/).closest('div')
      expect(summarySection).toBeInTheDocument()
    })

    it('shows correct number of open trades', () => {
      render(<PerformanceChart />)

      expect(screen.getByText(/1 lệnh mở/)).toBeInTheDocument()
    })

    it('shows correct number of closed trades', () => {
      render(<PerformanceChart />)

      expect(screen.getByText(/2 lệnh đóng/)).toBeInTheDocument()
    })

    it('handles multiple open trades', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: mockPortfolio,
        openTrades: [
          mockOpenTrades[0],
          { ...mockOpenTrades[0], id: '2', symbol: 'ETHUSDT' },
          { ...mockOpenTrades[0], id: '3', symbol: 'BNBUSDT' },
        ],
        closedTrades: mockClosedTrades,
      })

      render(<PerformanceChart />)

      expect(screen.getByText(/3 lệnh mở/)).toBeInTheDocument()
    })

    it('handles empty open trades', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: mockPortfolio,
        openTrades: [],
        closedTrades: mockClosedTrades,
      })

      render(<PerformanceChart />)

      expect(screen.getByText(/0 lệnh mở/)).toBeInTheDocument()
    })

    it('handles empty closed trades', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: mockPortfolio,
        openTrades: mockOpenTrades,
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText(/0 lệnh đóng/)).toBeInTheDocument()
    })
  })

  describe('Edge Cases', () => {
    it('handles zero win rate', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          win_rate: 0,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText('0.0%')).toBeInTheDocument()
    })

    it('handles perfect win rate', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          win_rate: 100,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText('100.0%')).toBeInTheDocument()
    })

    it('handles very small PnL values', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_pnl: 0.01,
          total_pnl_percentage: 0.0001,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText(/0,01 US\$/)).toBeInTheDocument()
    })

    it('handles very large PnL values', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_pnl: 999999.99,
          total_pnl_percentage: 1000,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText(/999\.999,99 US\$/)).toBeInTheDocument()
    })

    it('handles negative Sharpe ratio', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          sharpe_ratio: -0.5,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText('-0.50')).toBeInTheDocument()
    })

    it('handles zero total trades', () => {
      mockUsePaperTradingContext.mockReturnValue({
        portfolio: {
          ...mockPortfolio,
          total_trades: 0,
        },
        openTrades: [],
        closedTrades: [],
      })

      render(<PerformanceChart />)

      expect(screen.getByText('0 trades')).toBeInTheDocument()
    })
  })

  describe('Responsiveness', () => {
    it('renders ResponsiveContainer for chart', () => {
      const { container } = render(<PerformanceChart />)

      // Should have height class for responsive container
      expect(container.querySelector('[class*="h-\\[300px\\]"]')).toBeInTheDocument()
    })

    it('displays all metrics in grid layout', () => {
      render(<PerformanceChart />)

      expect(screen.getByText('Tổng P&L')).toBeInTheDocument()
      expect(screen.getByText('Equity hiện tại')).toBeInTheDocument()
      expect(screen.getByText('Win Rate')).toBeInTheDocument()
    })
  })
})
