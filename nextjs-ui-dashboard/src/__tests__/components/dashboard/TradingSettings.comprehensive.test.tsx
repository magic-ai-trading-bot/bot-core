import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { InlineTradingSettings, TradingSettings } from '../../../components/dashboard/TradingSettings'
import { toast } from 'sonner'

// Mock scrollIntoView for Radix UI
beforeEach(() => {
  Element.prototype.scrollIntoView = vi.fn()
})

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

describe('TradingSettings - Comprehensive Coverage', () => {
  const mockDefaultSettings = {
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
      market_preset: 'normal_volatility',
    },
  }

  beforeEach(() => {
    vi.clearAllMocks()
    const mockResponse = {
      ok: true,
      json: async () => mockDefaultSettings,
      clone: function() { return this; },
    }
    mockFetch.mockResolvedValue(mockResponse)
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('InlineTradingSettings Component', () => {
    it('renders inline trading settings without dialog', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })
    })

    it('displays loading spinner during initial load', () => {
      render(<InlineTradingSettings />)

      const spinner = document.querySelector('.animate-spin')
      expect(spinner).toBeInTheDocument()
    })

    it('shows loading text during fetch', () => {
      render(<InlineTradingSettings />)

      expect(screen.getByText('Loading settings...')).toBeInTheDocument()
    })

    it('hides loading state after settings loaded', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.queryByText('Loading settings...')).not.toBeInTheDocument()
      })
    })

    it('renders all preset cards inline', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Low Volatility')).toBeInTheDocument()
        expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
        expect(screen.getByText('High Volatility')).toBeInTheDocument()
      })
    })

    it('renders all strategy cards inline', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('RSI Strategy')).toBeInTheDocument()
        expect(screen.getByText('MACD Strategy')).toBeInTheDocument()
        expect(screen.getByText('Volume Strategy')).toBeInTheDocument()
        expect(screen.getByText('Bollinger Bands')).toBeInTheDocument()
        expect(screen.getByText('Stochastic')).toBeInTheDocument()
      })
    })

    it('renders risk management section inline', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Risk Management')).toBeInTheDocument()
      })
    })

    it('renders engine settings section inline', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Engine Settings')).toBeInTheDocument()
      })
    })

    it('displays reload button in inline view', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /reload/i })).toBeInTheDocument()
      })
    })

    it('displays save button in inline view', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save strategy settings/i })).toBeInTheDocument()
      })
    })
  })

  describe('Preset Card Interactions', () => {
    it('shows preset metrics for low volatility', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Low Volatility')).toBeInTheDocument()
        // Check for metrics labels (getAllByText because they appear multiple times)
        expect(screen.getAllByText('Confidence Threshold:').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Max Risk per Trade:').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Stop Loss:').length).toBeGreaterThan(0)
      })
    })

    it('shows preset metrics for high volatility', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('High Volatility')).toBeInTheDocument()
        // Check for metrics labels
        expect(screen.getAllByText('Confidence Threshold:').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Max Risk per Trade:').length).toBeGreaterThan(0)
        expect(screen.getAllByText('Stop Loss:').length).toBeGreaterThan(0)
      })
    })

    it('shows check icon on selected preset', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        const normalVolCard = screen.getByText('Normal Volatility').closest('div')
        const checkIcon = normalVolCard!.querySelector('.lucide-check')
        expect(checkIcon).toBeInTheDocument()
      })
    })

    it('displays preset icons', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('📊')).toBeInTheDocument() // Low vol icon
        expect(screen.getByText('⚖️')).toBeInTheDocument() // Normal vol icon
        expect(screen.getByText('🚀')).toBeInTheDocument() // High vol icon
      })
    })

    it('shows preset recommendations based on selection', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText(/Normal volatility settings provide balanced parameters/i)).toBeInTheDocument()
      })
    })

    it('updates recommendations when preset changes', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        const lowVolCard = screen.getByText('Low Volatility').closest('div')
        if (lowVolCard) {
          user.click(lowVolCard)
        }
      })

      await waitFor(() => {
        expect(screen.getByText(/In low volatility markets.*more sensitive parameters/i)).toBeInTheDocument()
      })
    })
  })

  describe('Strategy Slider Interactions', () => {
    it('displays RSI oversold and overbought sliders', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Oversold')).toBeInTheDocument()
        expect(screen.getByText('Overbought')).toBeInTheDocument()
      })
    })

    it('displays MACD fast, slow, and signal sliders', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Fast')).toBeInTheDocument()
        expect(screen.getByText('Slow')).toBeInTheDocument()
        expect(screen.getByText('Signal')).toBeInTheDocument()
      })
    })

    it('displays Volume SMA period slider', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('SMA Period')).toBeInTheDocument()
      })
    })

    it('displays Bollinger period slider', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Period')).toBeInTheDocument()
      })
    })

    it('displays Stochastic K and D period sliders', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('K Period')).toBeInTheDocument()
        expect(screen.getByText('D Period')).toBeInTheDocument()
      })
    })

    it('toggles Stochastic strategy when initially undefined', async () => {
      const settingsWithoutStochastic = {
        ...mockDefaultSettings,
        data: {
          ...mockDefaultSettings.data,
          strategies: {
            ...mockDefaultSettings.data.strategies,
            stochastic: undefined as any,
          },
        },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => settingsWithoutStochastic,
        clone: function() { return this; },
      })

      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        const switches = screen.getAllByRole('switch')
        const stochasticSwitch = switches[4] // 5th switch
        expect(stochasticSwitch).not.toBeChecked()
      })
    })
  })

  describe('Risk Management Sliders', () => {
    it('displays all position risk sliders', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText(/Max Risk per Trade/)).toBeInTheDocument()
        expect(screen.getByText(/Stop Loss/)).toBeInTheDocument()
        expect(screen.getByText(/Take Profit/)).toBeInTheDocument()
      })
    })

    it('displays all portfolio risk sliders', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText(/Max Portfolio Risk/)).toBeInTheDocument()
        expect(screen.getByText(/Max Drawdown/)).toBeInTheDocument()
        expect(screen.getByText(/Max Consecutive Losses/)).toBeInTheDocument()
      })
    })

    it('displays position correlation limit slider', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText(/Position Correlation Limit/)).toBeInTheDocument()
        expect(screen.getByText(/Maximum % of positions in same direction/)).toBeInTheDocument()
      })
    })

    it('shows correct slider descriptions', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText(/Maximum % of positions in same direction/)).toBeInTheDocument()
      })
    })
  })

  describe('Engine Settings Selectors', () => {
    it('displays data timeframe selector', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Data Timeframe')).toBeInTheDocument()
      })
    })

    it('shows timeframe description', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText(/Timeframe for trading signals and technical analysis/)).toBeInTheDocument()
      })
    })

    it('shows confidence threshold description', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getByText(/Lower values = more signals/)).toBeInTheDocument()
      })
    })

    it('displays all signal combination mode options', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const select = screen.getByText('Signal Combination Mode').parentElement?.querySelector('[role="combobox"]')
        if (select) {
          user.click(select)
        }
      })

      await waitFor(() => {
        expect(screen.getByRole('option', { name: /weighted average/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /consensus/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /best confidence/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /conservative/i })).toBeInTheDocument()
      })
    })

    it('displays all market condition options', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const select = screen.getByText('Market Condition').parentElement?.querySelector('[role="combobox"]')
        if (select) {
          user.click(select)
        }
      })

      await waitFor(() => {
        expect(screen.getByRole('option', { name: /trending/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /ranging/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /volatile/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /low volume/i })).toBeInTheDocument()
      })
    })

    it('displays all risk level options', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const select = screen.getByText('Risk Level').parentElement?.querySelector('[role="combobox"]')
        if (select) {
          user.click(select)
        }
      })

      await waitFor(() => {
        expect(screen.getByRole('option', { name: /conservative/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /moderate/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /aggressive/i })).toBeInTheDocument()
      })
    })

    it('displays all timeframe options', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const select = screen.getByText('Data Timeframe').parentElement?.querySelector('[role="combobox"]')
        if (select) {
          user.click(select)
        }
      })

      await waitFor(() => {
        expect(screen.getByRole('option', { name: /1 minute/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /15 minutes \(recommended\)/i })).toBeInTheDocument()
        expect(screen.getByRole('option', { name: /1 day/i })).toBeInTheDocument()
      })
    })
  })

  describe('Settings Persistence', () => {
    it('loads settings with market_preset from backend', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        const normalVolCard = screen.getByText('Normal Volatility').closest('div')
        const checkIcon = normalVolCard!.querySelector('.lucide-check')
        expect(checkIcon).toBeInTheDocument()
      })
    })

    it('handles settings without market_preset field', async () => {
      const settingsWithoutPreset = {
        ...mockDefaultSettings,
        data: {
          ...mockDefaultSettings.data,
          market_preset: undefined,
        },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => settingsWithoutPreset,
        clone: function() { return this; },
      })

      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })
    })

    it('sends market_preset in save request', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /save settings/i })).toBeInTheDocument()
      })

      const lowVolCard = screen.getByText('Low Volatility').closest('div')
      if (lowVolCard) {
        await user.click(lowVolCard)
      }

      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      await waitFor(() => {
        const putCall = mockFetch.mock.calls.find(call => {
          const request = call[1] as RequestInit
          return request?.method === 'PUT'
        })
        expect(putCall).toBeDefined()

        if (putCall && putCall[1]) {
          const body = JSON.parse((putCall[1] as RequestInit).body as string)
          expect(body.settings.market_preset).toBe('low_volatility')
        }
      })
    })
  })

  describe('UI States and Animations', () => {
    it('shows Paper Mode badge in dialog', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Paper Mode')).toBeInTheDocument()
      })
    })

    it('shows Quick Setup badge on presets', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Quick Setup')).toBeInTheDocument()
      })
    })

    it('displays dialog title correctly', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Trading Bot Settings')).toBeInTheDocument()
        expect(screen.getByText('Advanced Configuration')).toBeInTheDocument()
      })
    })

    it('shows glow icons in sections', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        const icons = document.querySelectorAll('[class*="lucide"]')
        expect(icons.length).toBeGreaterThan(0)
      })
    })

    it('shows settings icon in dialog', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        const settingsIcons = document.querySelectorAll('.lucide-settings')
        expect(settingsIcons.length).toBeGreaterThan(0)
      })
    })

    it('displays section headings with icons', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
        expect(screen.getByText('Active Strategies')).toBeInTheDocument()
        expect(screen.getByText('Risk Management')).toBeInTheDocument()
        expect(screen.getByText('Engine Settings')).toBeInTheDocument()
      })
    })
  })

  describe('Edge Cases', () => {
    it('handles failed load gracefully', async () => {
      mockFetch.mockRejectedValueOnce(new Error('Network error'))

      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to load trading settings')
      })
    })

    it('handles non-ok response for load', async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ error: 'Server error' }),
        clone: function() { return this; },
      })

      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        // Should not show error toast for non-ok GET response (fails silently)
        expect(screen.getByRole('dialog')).toBeInTheDocument()
      })
    })

    it('handles missing stochastic settings gracefully', async () => {
      const settingsWithoutStochastic = {
        ...mockDefaultSettings,
        data: {
          ...mockDefaultSettings.data,
          strategies: {
            ...mockDefaultSettings.data.strategies,
            stochastic: undefined as any,
          },
        },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => settingsWithoutStochastic,
        clone: function() { return this; },
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Stochastic')).toBeInTheDocument()
      })
    })

    it('initializes stochastic with defaults when toggling on from undefined', async () => {
      const settingsWithoutStochastic = {
        ...mockDefaultSettings,
        data: {
          ...mockDefaultSettings.data,
          strategies: {
            ...mockDefaultSettings.data.strategies,
            stochastic: undefined as any,
          },
        },
      }

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => settingsWithoutStochastic,
        clone: function() { return this; },
      })

      const user = userEvent.setup()
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Stochastic')).toBeInTheDocument()
      })

      const switches = screen.getAllByRole('switch')
      const stochasticSwitch = switches[4]

      await user.click(stochasticSwitch)

      await waitFor(() => {
        expect(stochasticSwitch).toBeChecked()
      })
    })
  })

  describe('Inline vs Dialog Consistency', () => {
    it('inline and dialog show same preset cards', async () => {
      const { unmount } = render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Low Volatility')).toBeInTheDocument()
        expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
        expect(screen.getByText('High Volatility')).toBeInTheDocument()
      })

      unmount()

      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(screen.getByText('Low Volatility')).toBeInTheDocument()
        expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
        expect(screen.getByText('High Volatility')).toBeInTheDocument()
      })
    })

    it('both versions load settings on mount/open', async () => {
      vi.clearAllMocks()

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled()
      })

      vi.clearAllMocks()

      const user = userEvent.setup()
      render(<TradingSettings />)

      // Dialog doesn't load until opened
      expect(mockFetch).not.toHaveBeenCalled()

      await user.click(screen.getByRole('button', { name: /trading settings/i }))

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalled()
      })
    })
  })
})
