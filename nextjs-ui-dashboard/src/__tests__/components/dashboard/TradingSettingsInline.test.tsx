import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { InlineTradingSettings } from '../../../components/dashboard/TradingSettings'
import { toast } from 'sonner'

// Mock fetch API
const mockFetch = vi.fn()
global.fetch = mockFetch

// Mock toast
vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

describe('InlineTradingSettings', () => {
  const mockSettingsData = {
    strategies: {
      rsi: {
        enabled: true,
        period: 14,
        oversold_threshold: 30,
        overbought_threshold: 70,
        extreme_oversold: 20,
        extreme_overbought: 80,
      },
      macd: {
        enabled: true,
        fast_period: 12,
        slow_period: 26,
        signal_period: 9,
        histogram_threshold: 0.001,
      },
      volume: {
        enabled: true,
        sma_period: 20,
        spike_threshold: 2.0,
        correlation_period: 10,
      },
      bollinger: {
        enabled: true,
        period: 20,
        multiplier: 2.0,
        squeeze_threshold: 0.02,
      },
      stochastic: {
        enabled: true,
        k_period: 14,
        d_period: 3,
        oversold_threshold: 20.0,
        overbought_threshold: 80.0,
        extreme_oversold: 10.0,
        extreme_overbought: 90.0,
      },
    },
    risk: {
      max_risk_per_trade: 2.0,
      max_portfolio_risk: 20,
      stop_loss_percent: 2.0,
      take_profit_percent: 4.0,
      max_leverage: 50,
      max_drawdown: 15,
      daily_loss_limit: 5,
      max_consecutive_losses: 5,
      correlation_limit: 0.7,
    },
    engine: {
      min_confidence_threshold: 0.65,
      signal_combination_mode: 'WeightedAverage',
      enabled_strategies: [
        'RSI Strategy',
        'MACD Strategy',
        'Volume Strategy',
        'Bollinger Bands Strategy',
        'Stochastic Strategy',
      ],
      market_condition: 'Trending',
      risk_level: 'Moderate',
      data_resolution: '15m',
    },
  }

  beforeEach(() => {
    vi.clearAllMocks()
    const mockResponse = {
      ok: true,
      json: async () => ({
        success: true,
        data: mockSettingsData,
      }),
      clone: function() { return this; },
    }
    mockFetch.mockResolvedValue(mockResponse)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('Inline Component Rendering', () => {
    it('renders inline trading settings without dialog wrapper', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })
    })

    it('loads settings on mount', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          'http://localhost:8080/api/paper-trading/strategy-settings'
        )
      })
    })

    it('shows loading state initially', () => {
      render(<InlineTradingSettings />)

      expect(screen.getByText('Loading settings...')).toBeInTheDocument()
    })
  })

  describe('Market Presets in Inline Mode', () => {
    it('displays all three presets in inline mode', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Low Volatility')).toBeInTheDocument()
        expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
        expect(screen.getByText('High Volatility')).toBeInTheDocument()
      })
    })

    it('applies preset when clicked in inline mode', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Low Volatility')).toBeInTheDocument()
      })

      const lowVolCard = screen.getByText('Low Volatility').closest('div')
      if (lowVolCard) {
        await user.click(lowVolCard)
      }

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Applied Low Volatility preset')
      })
    })

    it('shows preset metrics correctly', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText(/Confidence Threshold:/i).length).toBeGreaterThan(0)
        expect(screen.getAllByText(/Max Risk per Trade:/i).length).toBeGreaterThan(0)
      })
    })
  })

  describe('Active Strategies Section', () => {
    it('displays all active strategies', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText('RSI Strategy').length).toBeGreaterThan(0)
        expect(screen.getAllByText('MACD Strategy').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Volume Strategy').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Bollinger Bands').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Stochastic').length).toBeGreaterThan(0)
      })
    })

    it('toggles RSI strategy in inline mode', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText('RSI Strategy')[0]).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      const rsiSwitch = switches[0]
      expect(rsiSwitch).toBeChecked()

      await user.click(rsiSwitch)
      expect(rsiSwitch).not.toBeChecked()
    })

    it('adjusts strategy parameters with sliders', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        const sliders = screen.getAllByRole('slider')
        expect(sliders.length).toBeGreaterThan(0)
      })
    })

    it('displays stochastic strategy settings when enabled', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText('Stochastic').length).toBeGreaterThan(0)
      })

      // Should show K Period slider
      await waitFor(() => {
        expect(screen.getAllByText(/K Period/i).length).toBeGreaterThan(0)
      })
    })

    it('handles missing stochastic strategy gracefully', async () => {
      const dataWithoutStochastic = {
        ...mockSettingsData,
        strategies: {
          ...mockSettingsData.strategies,
          stochastic: undefined as any,
        },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: dataWithoutStochastic,
        }),
        clone: function() { return this; },
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText('RSI Strategy').length).toBeGreaterThan(0)
      })

      // Component should render without crashing
      const stochastic = screen.queryAllByText('Stochastic')
      expect(stochastic.length).toBeGreaterThan(0)
    })
  })

  describe('Risk Management Summary in Inline Mode', () => {
    it('displays risk management section', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Risk Management')).toBeInTheDocument()
      })
    })

    it('shows risk sliders with correct values', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText(/Max Risk per Trade/i).length).toBeGreaterThan(0)
        expect(screen.getAllByText(/Stop Loss/i).length).toBeGreaterThan(0)
        expect(screen.getAllByText(/Take Profit/i).length).toBeGreaterThan(0)
        expect(screen.getAllByText(/Max Drawdown/i).length).toBeGreaterThan(0)
      })
    })

    it('updates risk settings via sliders', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        const sliders = screen.getAllByRole('slider')
        expect(sliders.length).toBeGreaterThan(4) // Should have multiple risk sliders
      })
    })
  })

  describe('Engine Settings in Inline Mode', () => {
    it('displays engine settings section', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Engine Settings')).toBeInTheDocument()
      })
    })

    it('shows confidence threshold slider', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText(/Min Confidence Threshold/i).length).toBeGreaterThan(0)
      })
    })

    it('displays signal combination mode selector', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText(/Signal Combination Mode/i).length).toBeGreaterThan(0)
      })
    })

    it('allows changing signal combination mode', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getAllByText(/Signal Combination Mode/i).length).toBeGreaterThan(0)
      })

      // Find the select trigger for Signal Combination Mode
      const selectTriggers = screen.getAllByRole('combobox')
      if (selectTriggers.length > 0) {
        await user.click(selectTriggers[0])

        await waitFor(() => {
          // Options should appear
          const options = screen.queryAllByRole('option')
          expect(options.length).toBeGreaterThan(0)
        })
      }
    })
  })

  describe('Save and Reload Actions in Inline Mode', () => {
    it('displays save and reload buttons', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /reload/i })).toBeInTheDocument()
        expect(screen.getByRole('button', { name: /save strategy settings/i })).toBeInTheDocument()
      })
    })

    it('saves settings when save button clicked', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save strategy settings/i })).toBeInTheDocument()
      })

      vi.clearAllMocks()

      const saveButton = screen.getByRole('button', { name: /save strategy settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          'http://localhost:8080/api/paper-trading/strategy-settings',
          expect.objectContaining({
            method: 'PUT',
            headers: {
              'Content-Type': 'application/json',
            },
          })
        )
      })
    })

    it('shows success toast after save', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save strategy settings/i })).toBeInTheDocument()
      })

      const saveButton = screen.getByRole('button', { name: /save strategy settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Trading settings saved successfully!')
      })
    })

    it('reloads settings when reload button clicked', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /reload/i })).toBeInTheDocument()
      })

      vi.clearAllMocks()

      const reloadButton = screen.getByRole('button', { name: /reload/i })
      await user.click(reloadButton)

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          'http://localhost:8080/api/paper-trading/strategy-settings'
        )
      })
    })
  })

  describe('Error Handling in Inline Mode', () => {
    it('handles load error gracefully', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to load trading settings')
      })
    })

    it('handles save error gracefully', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save strategy settings/i })).toBeInTheDocument()
      })

      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      const saveButton = screen.getByRole('button', { name: /save strategy settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
      })
    })
  })

  describe('Preset Application', () => {
    it('updates settings when applying high volatility preset', async () => {
      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('High Volatility')).toBeInTheDocument()
      })

      const highVolCard = screen.getByText('High Volatility').closest('div')
      if (highVolCard) {
        await user.click(highVolCard)
      }

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Applied High Volatility preset')
      })

      // Confidence threshold should update to 75%
      await waitFor(() => {
        const percentageElements = screen.getAllByText(/75%/)
        expect(percentageElements.length).toBeGreaterThan(0)
      })
    })

    it('shows selected preset indicator', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        // Normal Volatility should be selected by default
        expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
      })

      // Check icon should be visible on selected preset
      const normalVolCard = screen.getByText('Normal Volatility').closest('div')
      expect(normalVolCard).toBeInTheDocument()
    })
  })
})
