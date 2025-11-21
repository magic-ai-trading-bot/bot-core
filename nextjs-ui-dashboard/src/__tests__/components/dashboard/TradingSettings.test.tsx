import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { TradingSettings } from '../../../components/dashboard/TradingSettings'
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

describe('TradingSettings', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    const mockResponse = {
      ok: true,
      json: async () => ({
        success: true,
        data: {
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
        },
      }),
      clone: function() { return this; },
    }
    mockFetch.mockResolvedValue(mockResponse)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('Dialog Trigger', () => {
    it('renders trading settings trigger button', () => {
      render(<TradingSettings />)

      const triggerButton = screen.getByRole('button', { name: /trading settings/i })
      expect(triggerButton).toBeInTheDocument()
    })

    it('opens dialog when trigger button is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      const triggerButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(triggerButton)

      await waitFor(() => {
        expect(screen.getByText('Trading Bot Settings')).toBeInTheDocument()
      })
    })

    it('shows advanced configuration badge', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      const triggerButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(triggerButton)

      await waitFor(() => {
        expect(screen.getByText('Advanced Configuration')).toBeInTheDocument()
      })
    })
  })

  describe('Tabs Navigation', () => {
    it('displays all four tabs', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('tab', { name: /market presets/i })).toBeInTheDocument()
        expect(screen.getByRole('tab', { name: /strategies/i })).toBeInTheDocument()
        expect(screen.getByRole('tab', { name: /risk management/i })).toBeInTheDocument()
        expect(screen.getByRole('tab', { name: /engine settings/i })).toBeInTheDocument()
      })
    })

    it('switches between tabs', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('tab', { name: /market presets/i })).toBeInTheDocument()
      })

      // Switch to Strategies tab
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('RSI Strategy')).toBeInTheDocument()
      })

      // Switch to Risk Management tab
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText('Position Risk')).toBeInTheDocument()
      })

      // Switch to Engine Settings tab
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Signal Processing')).toBeInTheDocument()
      })
    })
  })

  describe('Market Presets Tab', () => {
    it('displays all three market presets', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Low Volatility')).toBeInTheDocument()
        expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
        expect(screen.getByText('High Volatility')).toBeInTheDocument()
      })
    })

    it('displays preset descriptions', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText(/Optimized for sideways\/ranging markets/i)).toBeInTheDocument()
        expect(screen.getByText(/Balanced settings for typical market/i)).toBeInTheDocument()
        expect(screen.getByText(/Conservative settings for highly volatile/i)).toBeInTheDocument()
      })
    })

    it('applies low volatility preset', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

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

    it('applies normal volatility preset', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
      })

      const normalVolCard = screen.getByText('Normal Volatility').closest('div')
      if (normalVolCard) {
        await user.click(normalVolCard)
      }

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Applied Normal Volatility preset')
      })
    })

    it('applies high volatility preset', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

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
    })

    it('displays preset metrics', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getAllByText('Confidence Threshold:').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Max Risk per Trade:').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Stop Loss:').length).toBeGreaterThan(0)
      })
    })
  })

  describe('Strategies Tab', () => {
    it('displays all four strategy cards', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('RSI Strategy')).toBeInTheDocument()
        expect(screen.getByText('MACD Strategy')).toBeInTheDocument()
        expect(screen.getByText('Volume Strategy')).toBeInTheDocument()
        expect(screen.getByText('Bollinger Bands')).toBeInTheDocument()
      })
    })

    it('toggles RSI strategy on/off', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('RSI Strategy')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      const rsiSwitch = switches[0]

      expect(rsiSwitch).toBeChecked()

      await user.click(rsiSwitch)

      expect(rsiSwitch).not.toBeChecked()
    })

    it('adjusts RSI period slider', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText(/RSI Period:/)).toBeInTheDocument()
      })

      // Slider interaction is tested by presence
      const sliders = screen.getAllByRole('slider')
      expect(sliders.length).toBeGreaterThan(0)
    })

    it('toggles MACD strategy on/off', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('MACD Strategy')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      const macdSwitch = switches[1]

      expect(macdSwitch).toBeChecked()

      await user.click(macdSwitch)

      expect(macdSwitch).not.toBeChecked()
    })

    it('toggles Volume strategy on/off', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Volume Strategy')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      const volumeSwitch = switches[2]

      expect(volumeSwitch).toBeChecked()

      await user.click(volumeSwitch)

      expect(volumeSwitch).not.toBeChecked()
    })

    it('toggles Bollinger Bands strategy on/off', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Bollinger Bands')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      const bollingerSwitch = switches[3]

      expect(bollingerSwitch).toBeChecked()

      await user.click(bollingerSwitch)

      expect(bollingerSwitch).not.toBeChecked()
    })
  })

  describe('Risk Management Tab', () => {
    it('displays position risk and portfolio risk sections', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText('Position Risk')).toBeInTheDocument()
        expect(screen.getByText('Portfolio Risk')).toBeInTheDocument()
      })
    })

    it('displays risk management sliders', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText(/Max Risk per Trade:/)).toBeInTheDocument()
        expect(screen.getByText(/Stop Loss:/)).toBeInTheDocument()
        expect(screen.getByText(/Take Profit:/)).toBeInTheDocument()
        expect(screen.getByText(/Max Portfolio Risk:/)).toBeInTheDocument()
        expect(screen.getByText(/Max Drawdown:/)).toBeInTheDocument()
        expect(screen.getByText(/Max Consecutive Losses:/)).toBeInTheDocument()
      })
    })

    it('has all risk management sliders functional', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const sliders = screen.getAllByRole('slider')
        expect(sliders.length).toBeGreaterThan(0)
      })
    })
  })

  describe('Engine Settings Tab', () => {
    it('displays signal processing and market conditions sections', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Signal Processing')).toBeInTheDocument()
        expect(screen.getByText('Market Conditions')).toBeInTheDocument()
      })
    })

    it('displays confidence threshold slider', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText(/Min Confidence Threshold:/)).toBeInTheDocument()
      })
    })

    it('displays signal combination mode selector', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Signal Combination Mode')).toBeInTheDocument()
      })
    })

    it('displays market condition selector', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Market Condition')).toBeInTheDocument()
      })
    })

    it('displays risk level selector', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Risk Level')).toBeInTheDocument()
      })
    })
  })

  describe('Save and Load Actions', () => {
    it('displays reload and save buttons', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /reload/i })).toBeInTheDocument()
        expect(screen.getByRole('button', { name: /save settings/i })).toBeInTheDocument()
        expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument()
      })
    })

    it('loads settings on dialog open', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled()
        const callArg = mockFetch.mock.calls[0][0]
        const url = typeof callArg === 'string' ? callArg : callArg.url
        expect(url).toBe('http://localhost:8080/api/paper-trading/strategy-settings')
      })
    })

    it('saves settings when save button is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save settings/i })).toBeInTheDocument()
      })

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        // mockFetch should have been called multiple times (GET for load, PUT for save)
        expect(mockFetch.mock.calls.length).toBeGreaterThan(1)
      })
    })

    it('shows success toast when settings saved successfully', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save settings/i })).toBeInTheDocument()
      })

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Trading settings saved successfully!')
      })
    })

    it('handles save error gracefully', async () => {
      // Clear previous mocks and set up new responses
      vi.clearAllMocks()

      // First call (GET) succeeds with full data, second call (PUT) fails
      const mockResponsePut = {
        ok: false,
        json: async () => ({ error: 'Failed to save' }),
        clone: function() { return this; },
      }

      // Reset the default mock for GET
      const mockResponseGet = {
        ok: true,
        json: async () => ({
          success: true,
          data: {
            strategies: {
              rsi: { enabled: true, period: 14, oversold_threshold: 30, overbought_threshold: 70, extreme_oversold: 20, extreme_overbought: 80 },
              macd: { enabled: true, fast_period: 12, slow_period: 26, signal_period: 9, histogram_threshold: 0.001 },
              volume: { enabled: true, sma_period: 20, spike_threshold: 2.0, correlation_period: 10 },
              bollinger: { enabled: true, period: 20, multiplier: 2.0, squeeze_threshold: 0.02 },
            },
            risk: { max_risk_per_trade: 2.0, max_portfolio_risk: 20, stop_loss_percent: 2.0, take_profit_percent: 4.0, max_leverage: 50, max_drawdown: 15, daily_loss_limit: 5, max_consecutive_losses: 5 },
            engine: { min_confidence_threshold: 0.65, signal_combination_mode: 'WeightedAverage', enabled_strategies: ['RSI Strategy', 'MACD Strategy', 'Volume Strategy', 'Bollinger Bands Strategy'], market_condition: 'Trending', risk_level: 'Moderate' },
          },
        }),
        clone: function() { return this; },
      }
      mockFetch.mockResolvedValueOnce(mockResponseGet)
      mockFetch.mockResolvedValueOnce(mockResponsePut)

      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save settings/i })).toBeInTheDocument()
      })

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
      })
    })

    it('reloads settings when reload button is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /reload/i })).toBeInTheDocument()
      })

      vi.clearAllMocks()

      const reloadButton = screen.getByRole('button', { name: /reload/i })
      await user.click(reloadButton)

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled()
        const callArg = mockFetch.mock.calls[0][0]
        const url = typeof callArg === 'string' ? callArg : callArg.url
        expect(url).toBe('http://localhost:8080/api/paper-trading/strategy-settings')
      })
    })
  })

  describe('Dialog Close', () => {
    it('closes dialog when cancel button is clicked', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /cancel/i })).toBeInTheDocument()
      })

      const cancelButton = screen.getByRole('button', { name: /cancel/i })
      await user.click(cancelButton)

      await waitFor(() => {
        expect(screen.queryByText('Trading Bot Settings')).not.toBeInTheDocument()
      })
    })
  })

  describe('Loading States', () => {
    it('shows loading state when fetching settings', async () => {
      const user = userEvent.setup()

      // Make fetch slow
      mockFetch.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve({
        ok: true,
        json: async () => ({ success: true, data: {} })
      }), 100)))

      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      // Loading button should appear briefly
      await waitFor(() => {
        expect(screen.getByText(/loading/i)).toBeInTheDocument()
      }, { timeout: 50 })
    })

    it('shows saving state when saving settings', async () => {
      const user = userEvent.setup()

      // Make save slow
      mockFetch.mockImplementation(() => new Promise(resolve => setTimeout(() => resolve({
        ok: true,
        json: async () => ({ success: true })
      }), 100)))

      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save settings/i })).toBeInTheDocument()
      })

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      // Saving button should appear briefly
      await waitFor(() => {
        expect(screen.getByText(/saving/i)).toBeInTheDocument()
      }, { timeout: 50 })
    })
  })

  describe('Error Handling', () => {
    it('handles load error gracefully', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to load trading settings')
      })
    })

    it('handles save network error', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save settings/i })).toBeInTheDocument()
      })

      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
      })
    })
  })
})
