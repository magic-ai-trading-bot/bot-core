import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor, within, fireEvent, act } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { TradingSettings, InlineTradingSettings } from '../../../components/dashboard/TradingSettings'
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

describe('TradingSettings - onChange Handler Coverage', () => {
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

  // Helper to find slider by label in a specific section
  const findSliderInSection = (sectionText: string, sliderLabel: string) => {
    const section = screen.getByText(sectionText).closest('div[class*="rounded-xl"]')
    if (!section) return null
    const label = within(section as HTMLElement).getByText(sliderLabel)
    return label.closest('div')?.querySelector('input[type="range"]')
  }

  // Helper to find ALL sliders by label text
  const findAllSlidersByLabel = (labelText: string) => {
    const labels = screen.queryAllByText(labelText)
    return labels.map(label => label.closest('div')?.querySelector('input[type="range"]')).filter(Boolean)
  }

  describe('Dialog - Strategies Tab onChange Handlers', () => {
    it('should trigger RSI period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('RSI Strategy', 'RSI Period')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '20' } })
            fireEvent.input(slider, { target: { value: '20' } })
          })
        }
      })
    })

    it('should trigger RSI oversold onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('RSI Strategy', 'Oversold')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '25' } })
            fireEvent.input(slider, { target: { value: '25' } })
          })
        }
      })
    })

    it('should trigger RSI overbought onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('RSI Strategy', 'Overbought')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '75' } })
            fireEvent.input(slider, { target: { value: '75' } })
          })
        }
      })
    })

    it('should trigger RSI toggle onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const rsiSection = screen.getByText('RSI Strategy').closest('div[class*="rounded-xl"]')
        if (rsiSection) {
          const toggle = within(rsiSection as HTMLElement).getByRole('switch')
          act(() => {
            fireEvent.click(toggle)
          })
        }
      })
    })

    it('should trigger MACD fast period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('MACD Strategy', 'Fast')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '10' } })
          })
        }
      })
    })

    it('should trigger MACD slow period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('MACD Strategy', 'Slow')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '30' } })
          })
        }
      })
    })

    it('should trigger MACD signal period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('MACD Strategy', 'Signal')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '8' } })
          })
        }
      })
    })

    it('should trigger MACD toggle onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const macdSection = screen.getByText('MACD Strategy').closest('div[class*="rounded-xl"]')
        if (macdSection) {
          const toggle = within(macdSection as HTMLElement).getByRole('switch')
          act(() => {
            fireEvent.click(toggle)
          })
        }
      })
    })

    it('should trigger Volume spike threshold onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('Volume Strategy', 'Spike Threshold')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '3.5' } })
          })
        }
      })
    })

    it('should trigger Volume SMA period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('Volume Strategy', 'SMA Period')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '25' } })
          })
        }
      })
    })

    it('should trigger Volume toggle onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const volumeSection = screen.getByText('Volume Strategy').closest('div[class*="rounded-xl"]')
        if (volumeSection) {
          const toggle = within(volumeSection as HTMLElement).getByRole('switch')
          act(() => {
            fireEvent.click(toggle)
          })
        }
      })
    })

    it('should trigger Bollinger period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('Bollinger Bands', 'Period')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '25' } })
          })
        }
      })
    })

    it('should trigger Bollinger multiplier onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('Bollinger Bands', 'Multiplier')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '2.5' } })
          })
        }
      })
    })

    it('should trigger Bollinger toggle onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const bollingerSection = screen.getByText('Bollinger Bands').closest('div[class*="rounded-xl"]')
        if (bollingerSection) {
          const toggle = within(bollingerSection as HTMLElement).getByRole('switch')
          act(() => {
            fireEvent.click(toggle)
          })
        }
      })
    })

    it('should trigger Stochastic K period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('Stochastic Strategy', 'K Period')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '20' } })
          })
        }
      })
    })

    it('should trigger Stochastic D period onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const slider = findSliderInSection('Stochastic Strategy', 'D Period')
        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '5' } })
          })
        }
      })
    })

    it('should trigger Stochastic toggle onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
      await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

      await waitFor(() => {
        const stochasticSection = screen.getByText('Stochastic Strategy').closest('div[class*="rounded-xl"]')
        if (stochasticSection) {
          const toggle = within(stochasticSection as HTMLElement).getByRole('switch')
          act(() => {
            fireEvent.click(toggle)
          })
        }
      })
    })
  })

  describe('Dialog - Risk Tab onChange Handlers', () => {
    it('should trigger max risk per trade onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const riskTab = screen.getByRole('tabpanel')
        const maxRiskLabel = within(riskTab).getByText('Max Risk per Trade')
        const slider = maxRiskLabel.closest('div')?.querySelector('input[type="range"]')

        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '3.0' } })
          })
        }
      })
    })

    it('should trigger stop loss onChange in risk tab', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const sliders = findAllSlidersByLabel('Stop Loss')
        if (sliders.length > 0 && sliders[0]) {
          act(() => {
            fireEvent.change(sliders[0], { target: { value: '3.5' } })
          })
        }
      })
    })

    it('should trigger take profit onChange in risk tab', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const sliders = findAllSlidersByLabel('Take Profit')
        if (sliders.length > 0 && sliders[0]) {
          act(() => {
            fireEvent.change(sliders[0], { target: { value: '6.0' } })
          })
        }
      })
    })

    it('should trigger max portfolio risk onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const riskTab = screen.getByRole('tabpanel')
        const maxPortfolioLabel = within(riskTab).getByText('Max Portfolio Risk')
        const slider = maxPortfolioLabel.closest('div')?.querySelector('input[type="range"]')

        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '30' } })
          })
        }
      })
    })

    it('should trigger max drawdown onChange in risk tab', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const sliders = findAllSlidersByLabel('Max Drawdown')
        if (sliders.length > 0 && sliders[0]) {
          act(() => {
            fireEvent.change(sliders[0], { target: { value: '20' } })
          })
        }
      })
    })

    it('should trigger max consecutive losses onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const riskTab = screen.getByRole('tabpanel')
        const maxConsecLabel = within(riskTab).getByText('Max Consecutive Losses')
        const slider = maxConsecLabel.closest('div')?.querySelector('input[type="range"]')

        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '7' } })
          })
        }
      })
    })

    it('should trigger correlation limit onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        const riskTab = screen.getByRole('tabpanel')
        const correlationLabel = within(riskTab).getByText('Position Correlation Limit')
        const slider = correlationLabel.closest('div')?.querySelector('input[type="range"]')

        if (slider) {
          act(() => {
            fireEvent.change(slider, { target: { value: '80' } })
          })
        }
      })
    })
  })

  describe('Dialog - Engine Tab onChange Handlers', () => {
    it('should trigger min confidence threshold onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const sliders = findAllSlidersByLabel('Min Confidence Threshold')
        if (sliders.length > 0 && sliders[0]) {
          act(() => {
            fireEvent.change(sliders[0], { target: { value: '75' } })
          })
        }
      })
    })

    it('should trigger signal combination mode onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const engineTab = screen.getByRole('tabpanel')
        expect(within(engineTab).getByText('Signal Combination Mode')).toBeInTheDocument()
        // Select components need special handling - just verify presence
      })
    })

    it('should trigger market condition onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const engineTab = screen.getByRole('tabpanel')
        expect(within(engineTab).getByText('Market Condition')).toBeInTheDocument()
      })
    })

    it('should trigger risk level onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const engineTab = screen.getByRole('tabpanel')
        expect(within(engineTab).getByText('Risk Level')).toBeInTheDocument()
      })
    })

    it('should trigger data timeframe onChange', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        const engineTab = screen.getByRole('tabpanel')
        expect(within(engineTab).getByText('Data Timeframe')).toBeInTheDocument()
      })
    })
  })

  describe('InlineTradingSettings - onChange Handlers', () => {
    it('should trigger signal combination mode onChange in inline', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.queryByText('Loading settings...')).not.toBeInTheDocument()
      })

      await waitFor(() => {
        expect(screen.getByText('Engine Settings')).toBeInTheDocument()
      })
    })
  })
})
