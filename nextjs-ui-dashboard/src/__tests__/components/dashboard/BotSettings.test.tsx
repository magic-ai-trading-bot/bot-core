import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { BotSettings } from '../../../components/dashboard/BotSettings'

// Mock usePaperTradingContext hook (component uses context, not hook directly)
const mockStartBot = vi.fn()
const mockStopBot = vi.fn()
const mockUpdateSettings = vi.fn()
const mockResetPortfolio = vi.fn()

// Create stable mock data outside the mock to prevent infinite loops
// These objects will maintain the same reference across renders
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

const mockPositions: never[] = []
const mockOpenTrades: never[] = []
const mockClosedTrades: never[] = []
const mockRecentSignals: never[] = []
const mockLastUpdated = new Date()

vi.mock('../../../contexts/PaperTradingContext', () => ({
  usePaperTradingContext: vi.fn(() => ({
    portfolio: mockPortfolio,
    settings: mockSettings,
    positions: mockPositions,
    openTrades: mockOpenTrades,
    closedTrades: mockClosedTrades,
    recentSignals: mockRecentSignals,
    isActive: false,
    isLoading: false,
    error: null,
    lastUpdated: mockLastUpdated,
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

describe('BotSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    // Mock fetch for trading pairs API
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

  describe('Component Rendering', () => {
    it('renders the bot settings card', () => {
      render(<BotSettings />)

      expect(screen.getByText('Bot Configuration')).toBeInTheDocument()
    })

    it('displays ACTIVE badge when bot is active by default', () => {
      render(<BotSettings />)

      expect(screen.getByText('ACTIVE')).toBeInTheDocument()
    })

    it('displays all configuration sections', () => {
      render(<BotSettings />)

      expect(screen.getByText('Bot Status')).toBeInTheDocument()
      expect(screen.getByText('Capital Allocation')).toBeInTheDocument()
      expect(screen.getByText('Maximum Leverage')).toBeInTheDocument()
      expect(screen.getByText('Risk Threshold')).toBeInTheDocument()
      expect(screen.getByText('Active Trading Pairs')).toBeInTheDocument()
    })

    it('displays all trading pairs', async () => {
      render(<BotSettings />)

      // Wait for trading pairs to load from API mock
      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })
      expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
      expect(screen.getByText('BNB/USDT')).toBeInTheDocument()
      expect(screen.getByText('SOL/USDT')).toBeInTheDocument()
    })

    it('displays action buttons', () => {
      render(<BotSettings />)

      expect(screen.getByText('Reset to Default')).toBeInTheDocument()
      expect(screen.getByText('Save Settings')).toBeInTheDocument()
    })

    it('displays emergency stop section', () => {
      render(<BotSettings />)

      expect(screen.getByText('Emergency Stop')).toBeInTheDocument()
      expect(screen.getByText('Immediately close all positions and stop trading')).toBeInTheDocument()
      expect(screen.getByText('STOP ALL')).toBeInTheDocument()
    })
  })

  describe('Bot Status Toggle', () => {
    it('toggles bot status from active to inactive', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      // Initially active
      expect(screen.getByText('ACTIVE')).toBeInTheDocument()
      expect(screen.getByText('Bot is actively trading')).toBeInTheDocument()

      // Find and click the switch
      const botSwitch = screen.getAllByRole('switch')[0]
      await user.click(botSwitch)

      // Should be inactive
      await waitFor(() => {
        expect(screen.getByText('INACTIVE')).toBeInTheDocument()
        expect(screen.getByText('Bot is stopped')).toBeInTheDocument()
      })
    })

    it('toggles bot status from inactive to active', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      // Toggle to inactive first
      const botSwitch = screen.getAllByRole('switch')[0]
      await user.click(botSwitch)

      await waitFor(() => {
        expect(screen.getByText('INACTIVE')).toBeInTheDocument()
      })

      // Toggle back to active
      await user.click(botSwitch)

      await waitFor(() => {
        expect(screen.getByText('ACTIVE')).toBeInTheDocument()
      })
    })
  })

  describe('Capital Allocation Slider', () => {
    it('displays initial capital allocation value (75%)', () => {
      render(<BotSettings />)

      expect(screen.getByText('75%')).toBeInTheDocument()
    })

    it('displays calculated amount based on capital allocation', () => {
      render(<BotSettings />)

      // 75% of 12450 = 9337.5 (displayed as 9,337.5)
      expect(screen.getByText(/Amount: \$9,337\.5/)).toBeInTheDocument()
    })

    it('shows conservative and aggressive labels', () => {
      render(<BotSettings />)

      expect(screen.getByText('Conservative (10%)')).toBeInTheDocument()
      expect(screen.getByText('Aggressive (100%)')).toBeInTheDocument()
    })
  })

  describe('Leverage Slider', () => {
    it('displays initial leverage value (10x)', () => {
      render(<BotSettings />)

      expect(screen.getByText('10x')).toBeInTheDocument()
    })

    it('shows safe and high risk labels', () => {
      render(<BotSettings />)

      expect(screen.getByText('Safe (1x)')).toBeInTheDocument()
      expect(screen.getByText('High Risk (20x)')).toBeInTheDocument()
    })
  })

  describe('Risk Threshold Slider', () => {
    it('displays initial risk threshold value (5%)', () => {
      render(<BotSettings />)

      expect(screen.getByText('5%')).toBeInTheDocument()
    })

    it('displays calculated max loss per trade', () => {
      render(<BotSettings />)

      // 5% of 12450 = 622.50
      expect(screen.getByText(/Max loss per trade: \$622.50/)).toBeInTheDocument()
    })

    it('shows conservative and aggressive labels for risk', () => {
      render(<BotSettings />)

      const conservativeLabels = screen.getAllByText(/Conservative/)
      const aggressiveLabels = screen.getAllByText(/Aggressive/)

      // Should have at least 2 of each (for different sliders)
      expect(conservativeLabels.length).toBeGreaterThanOrEqual(2)
      expect(aggressiveLabels.length).toBeGreaterThanOrEqual(2)
    })
  })

  describe('Trading Pairs', () => {
    it('has BTC/USDT switch checked by default', async () => {
      render(<BotSettings />)

      // Wait for trading pairs to load
      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      // BTC/USDT is the second switch (index 1)
      expect(switches[1]).toBeChecked()
    })

    it('has ETH/USDT switch checked by default', async () => {
      render(<BotSettings />)

      // Wait for trading pairs to load
      await waitFor(() => {
        expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      // ETH/USDT is the third switch (index 2)
      expect(switches[2]).toBeChecked()
    })

    it('toggles trading pair switch', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      // Wait for trading pairs to load
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

  describe('Action Buttons', () => {
    it('renders reset to default button', () => {
      render(<BotSettings />)

      const resetButton = screen.getByRole('button', { name: /reset to default/i })
      expect(resetButton).toBeInTheDocument()
    })

    it('renders save settings button', () => {
      render(<BotSettings />)

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      expect(saveButton).toBeInTheDocument()
    })

    it('clicks reset to default button', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      const resetButton = screen.getByRole('button', { name: /reset to default/i })
      await user.click(resetButton)

      // Button should still be in the document after click
      expect(resetButton).toBeInTheDocument()
    })

    it('clicks save settings button', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      // Button should still be in the document after click
      expect(saveButton).toBeInTheDocument()
    })
  })

  describe('Emergency Stop', () => {
    it('renders emergency stop button', () => {
      render(<BotSettings />)

      const stopButton = screen.getByRole('button', { name: /stop all/i })
      expect(stopButton).toBeInTheDocument()
    })

    it('clicks emergency stop button', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      const stopButton = screen.getByRole('button', { name: /stop all/i })
      await user.click(stopButton)

      // Button should still be in the document after click
      expect(stopButton).toBeInTheDocument()
    })

    it('displays emergency stop warning text', () => {
      render(<BotSettings />)

      expect(screen.getByText('Immediately close all positions and stop trading')).toBeInTheDocument()
    })
  })

  describe('Visual States', () => {
    it('applies profit color class to active badge', () => {
      render(<BotSettings />)

      const activeBadge = screen.getByText('ACTIVE')
      expect(activeBadge.className).toContain('bg-profit')
    })

    it('applies correct styling to bot status section', () => {
      render(<BotSettings />)

      const statusSection = screen.getByText('Bot Status').closest('div')
      expect(statusSection).toBeInTheDocument()
    })

    it('applies loss color to emergency stop section', () => {
      render(<BotSettings />)

      const emergencyTitle = screen.getByText('Emergency Stop')
      expect(emergencyTitle.className).toContain('text-loss')
    })
  })

  describe('Accessibility', () => {
    it('has accessible switch roles', async () => {
      render(<BotSettings />)

      // Wait for trading pairs to load from API
      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      // Should have 6 switches: bot status + 5 trading pairs (from API mock)
      expect(switches).toHaveLength(6)
    })

    it('has accessible button roles', () => {
      render(<BotSettings />)

      const buttons = screen.getAllByRole('button')
      // Should have at least 3 buttons: reset, save, stop all
      expect(buttons.length).toBeGreaterThanOrEqual(3)
    })
  })

  describe('Data Display', () => {
    it('displays correct initial capital allocation percentage', () => {
      render(<BotSettings />)

      // Initial value is 75%
      const percentageText = screen.getByText('75%')
      expect(percentageText).toBeInTheDocument()
    })

    it('displays correct initial leverage multiplier', () => {
      render(<BotSettings />)

      // Initial value is 10x
      const leverageText = screen.getByText('10x')
      expect(leverageText).toBeInTheDocument()
    })

    it('calculates and displays correct capital amount', () => {
      render(<BotSettings />)

      // 75% of 12450 = 9337.5
      expect(screen.getByText(/\$9,337\.5/)).toBeInTheDocument()
    })

    it('calculates and displays correct max loss', () => {
      render(<BotSettings />)

      // 5% of 12450 = 622.50
      expect(screen.getByText(/\$622.50/)).toBeInTheDocument()
    })
  })

  describe('Component Structure', () => {
    it('renders within a Card component', () => {
      const { container } = render(<BotSettings />)

      // Card component uses specific class patterns
      const card = container.querySelector('[class*="border"]')
      expect(card).toBeInTheDocument()
    })

    it('has proper spacing between sections', () => {
      render(<BotSettings />)

      // All main sections should be present
      expect(screen.getByText('Bot Status')).toBeInTheDocument()
      expect(screen.getByText('Capital Allocation')).toBeInTheDocument()
      expect(screen.getByText('Maximum Leverage')).toBeInTheDocument()
      expect(screen.getByText('Risk Threshold')).toBeInTheDocument()
    })
  })

  describe('Edge Cases', () => {
    it('handles rapid toggle clicks', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      const botSwitch = screen.getAllByRole('switch')[0]

      // Rapidly click multiple times
      await user.click(botSwitch)
      await user.click(botSwitch)
      await user.click(botSwitch)

      // Component should still be functional
      expect(screen.getByText('Bot Configuration')).toBeInTheDocument()
    })

    it('handles multiple trading pair toggles', async () => {
      const user = userEvent.setup()
      render(<BotSettings />)

      // Wait for trading pairs to load from API
      await waitFor(() => {
        expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')

      // Toggle multiple pairs
      await user.click(switches[1]) // BTC
      await user.click(switches[2]) // ETH
      await user.click(switches[3]) // BNB

      // All switches should be functional - 6 switches: bot status + 5 trading pairs
      expect(switches).toHaveLength(6)
    })
  })
})
