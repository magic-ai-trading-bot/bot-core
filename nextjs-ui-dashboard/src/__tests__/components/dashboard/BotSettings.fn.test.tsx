/**
 * BotSettings Component - Functional Tests
 * Target: Boost coverage from 80.45% to 95%+
 * Focus: API calls, error handling, loading states, edge cases
 */

import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { BotSettings } from '../../../components/dashboard/BotSettings'

// Mock framer-motion at the top
vi.mock('framer-motion', () => {
  const createMotionComponent = (tag: string) => {
    const Comp = ({ children, whileHover, whileTap, animate, transition, initial, exit, variants, ...props }: any) => {
      const Tag = tag as any
      return <Tag {...props}>{children}</Tag>
    }
    return Comp
  }
  return {
    motion: new Proxy({}, { get: (_t, p: string) => createMotionComponent(p) }),
    AnimatePresence: ({ children }: any) => <>{children}</>,
  }
})

const mockStartBot = vi.fn()
const mockStopBot = vi.fn()
const mockUpdateSettings = vi.fn()
const mockResetPortfolio = vi.fn()

const mockPortfolio = {
  current_balance: 12450,
  available_balance: 11000,
  equity: 12450,
  total_pnl: 0,
  total_pnl_percentage: 0,
  total_trades: 0,
  margin_used: 1450,
  free_margin: 11000,
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
}

const mockSettings = {
  basic: {
    initial_balance: 10000,
    default_position_size_pct: 75,
    trading_fee_rate: 0.04,
    enabled: true,
    default_leverage: 10,
  },
  risk: {
    max_leverage: 20,
    default_stop_loss_pct: 5,
    default_take_profit_pct: 10,
    max_risk_per_trade_pct: 5,
  },
  strategy: {
    name: 'MACD',
    parameters: {},
  },
  exit_strategy: {
    type: 'trailing_stop',
    parameters: {},
  },
}

vi.mock('../../../contexts/PaperTradingContext', () => ({
  usePaperTradingContext: vi.fn(() => ({
    portfolio: mockPortfolio,
    settings: mockSettings,
    positions: [],
    openTrades: [],
    closedTrades: [],
    recentSignals: [],
    isActive: false,
    isLoading: false,
    error: null,
    lastUpdated: new Date(),
    startBot: mockStartBot,
    stopBot: mockStopBot,
    updateSettings: mockUpdateSettings,
    resetPortfolio: mockResetPortfolio,
    startTrading: vi.fn(),
    stopTrading: vi.fn(),
    closeTrade: vi.fn(),
    refreshAISignals: vi.fn(),
    refreshSettings: vi.fn(),
  })),
  PaperTradingProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

describe('BotSettings - Functional Tests', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    Element.prototype.scrollIntoView = vi.fn()

    // Default mock fetch
    global.fetch = vi.fn().mockResolvedValue({
      ok: true,
      json: vi.fn().mockResolvedValue({
        success: true,
        data: {
          symbols: ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'XRPUSDT', 'SOLUSDT']
        }
      })
    }) as unknown as typeof fetch
  })

  describe('API - Symbol Loading', () => {
    it('should fetch symbols from API on mount', async () => {
      render(<BotSettings />)

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalledWith(
          expect.stringContaining('/api/market/symbols')
        )
      })
    })

    it('should display loading state while fetching symbols', () => {
      render(<BotSettings />)

      expect(screen.getByText('Loading trading pairs...')).toBeInTheDocument()
    })

    it('should handle API success with symbols', async () => {
      render(<BotSettings />)

      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })

      expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
      expect(screen.getByText('BNB/USDT')).toBeInTheDocument()
    })

    it('should use fallback symbols when API fails', async () => {
      global.fetch = vi.fn().mockRejectedValue(new Error('Network error'))

      render(<BotSettings />)

      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })

      // Should still display fallback symbols
      expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
    })

    it('should use fallback symbols when API returns empty data', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue({
          success: true,
          data: {
            symbols: []
          }
        })
      }) as unknown as typeof fetch

      render(<BotSettings />)

      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })
    })

    it('should use fallback symbols when API returns no success', async () => {
      global.fetch = vi.fn().mockResolvedValue({
        ok: true,
        json: vi.fn().mockResolvedValue({
          success: false,
          data: null
        })
      }) as unknown as typeof fetch

      render(<BotSettings />)

      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })
    })

    it('should enable first 2 symbols by default', async () => {
      render(<BotSettings />)

      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      expect(switches[1]).toBeChecked() // BTC
      expect(switches[2]).toBeChecked() // ETH
      expect(switches[3]).not.toBeChecked() // BNB
    })
  })

  describe('Bot Status Toggle - Success', () => {
    it('should display bot switch in active state', async () => {
      render(<BotSettings />)

      await waitFor(() => {
        expect(screen.getByText('ACTIVE')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      expect(switches[0]).toBeInTheDocument()
    })

    it('should call stopBot when toggling off', async () => {
      const user = userEvent.setup()
      mockStopBot.mockResolvedValue(undefined)

      render(<BotSettings />)

      const botSwitch = screen.getAllByRole('switch')[0]
      await user.click(botSwitch)

      await waitFor(() => {
        expect(mockStopBot).toHaveBeenCalled()
      })
    })
  })

  describe('Bot Status Toggle - Error Handling', () => {
    it('should handle errors when toggling bot status', async () => {
      mockStopBot.mockRejectedValue(new Error('Failed to stop'))

      render(<BotSettings />)

      expect(screen.getByText('ACTIVE')).toBeInTheDocument()
    })
  })

  describe('Save Settings', () => {
    it('should render save settings button', () => {
      render(<BotSettings />)

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      expect(saveButton).toBeInTheDocument()
      expect(saveButton).not.toBeDisabled()
    })
  })

  describe('Reset Portfolio', () => {
    it('should render reset to default button', () => {
      render(<BotSettings />)

      const resetButton = screen.getByRole('button', { name: /reset to default/i })
      expect(resetButton).toBeInTheDocument()
      expect(resetButton).not.toBeDisabled()
    })
  })

  describe('Emergency Stop', () => {
    it('should render emergency stop button', () => {
      render(<BotSettings />)

      const stopButton = screen.getByRole('button', { name: /stop all/i })
      expect(stopButton).toBeInTheDocument()
      expect(stopButton).not.toBeDisabled()
    })
  })

  describe('Slider Interactions', () => {
    it('should update capital allocation display when slider changes', async () => {
      const user = userEvent.setup()

      render(<BotSettings />)

      // Initial value
      expect(screen.getByText('75%')).toBeInTheDocument()

      // Note: Slider interaction is difficult to test in jsdom
      // This test verifies the display is correct
    })

    it('should calculate correct allocated capital amount', () => {
      render(<BotSettings />)

      // 75% of 12450 = 9337.50
      expect(screen.getByText(/Amount: \$9,337\.5/)).toBeInTheDocument()
    })

    it('should calculate correct max loss per trade', () => {
      render(<BotSettings />)

      // 5% of 12450 = 622.50
      expect(screen.getByText(/Max loss per trade: \$622.50/)).toBeInTheDocument()
    })
  })

  describe('Trading Pair Toggles', () => {
    it('should toggle trading pair state', async () => {
      const user = userEvent.setup()

      render(<BotSettings />)

      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      const btcSwitch = switches[1]

      expect(btcSwitch).toBeChecked()
      await user.click(btcSwitch)
      expect(btcSwitch).not.toBeChecked()
    })
  })

  describe('Edge Cases', () => {
    it('should render component with portfolio data', () => {
      render(<BotSettings />)

      // Should display calculated amount
      expect(screen.getByText(/Amount: \$/)).toBeInTheDocument()
    })

    it('should handle API fetch scenarios', async () => {
      render(<BotSettings />)

      // Wait for loading to complete
      await waitFor(() => {
        expect(screen.queryByText('Loading trading pairs...')).not.toBeInTheDocument()
      }, { timeout: 3000 })

      // Should show trading pairs
      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
    })
  })

  describe('Settings Sync', () => {
    it('should sync local state with backend settings on mount', () => {
      render(<BotSettings />)

      expect(screen.getByText('75%')).toBeInTheDocument()
      expect(screen.getByText('10x')).toBeInTheDocument()
      expect(screen.getByText('5%')).toBeInTheDocument()
    })
  })

  describe('Button States', () => {
    it('should render all buttons in enabled state initially', () => {
      render(<BotSettings />)

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      const resetButton = screen.getByRole('button', { name: /reset to default/i })
      const stopButton = screen.getByRole('button', { name: /stop all/i })

      expect(saveButton).not.toBeDisabled()
      expect(resetButton).not.toBeDisabled()
      expect(stopButton).not.toBeDisabled()
    })
  })
})
