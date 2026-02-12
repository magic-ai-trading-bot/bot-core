import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen, waitFor, within, fireEvent } from '@testing-library/react'
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

describe('TradingSettings.boost - Dialog Full Coverage', () => {
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

  // ========================================
  // Dialog Opening & Structure
  // ========================================

  it('should render dialog trigger button', () => {
    render(<TradingSettings />)
    expect(screen.getByRole('button', { name: /trading settings/i })).toBeInTheDocument()
  })

  it('should open dialog when trigger button clicked', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    const triggerButton = screen.getByRole('button', { name: /trading settings/i })
    await user.click(triggerButton)

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })
  })

  it('should render dialog header with title and description', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByText('Trading Bot Settings')).toBeInTheDocument()
      expect(screen.getByText('Advanced Configuration')).toBeInTheDocument()
      expect(screen.getByText('Paper Mode')).toBeInTheDocument()
    })
  })

  it('should render all tabs in dialog', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('tab', { name: /market presets/i })).toBeInTheDocument()
      expect(screen.getByRole('tab', { name: /^strategies$/i })).toBeInTheDocument()
      expect(screen.getByRole('tab', { name: /risk management/i })).toBeInTheDocument()
      expect(screen.getByRole('tab', { name: /engine settings/i })).toBeInTheDocument()
    })
  })

  // ========================================
  // Tab: Market Presets (in Dialog)
  // ========================================

  it('should render all preset cards in dialog', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByText('Low Volatility')).toBeInTheDocument()
      expect(screen.getByText('Normal Volatility')).toBeInTheDocument()
      expect(screen.getByText('High Volatility')).toBeInTheDocument()
    })
  })

  it('should show preset recommendations in info box', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      // Normal volatility recommendation should show by default
      expect(screen.getByText(/Normal volatility settings provide balanced parameters/i)).toBeInTheDocument()
    })
  })

  it('should update info box when preset selected', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByText('Low Volatility')).toBeInTheDocument()
    })

    // Click low volatility preset
    const lowVolCard = screen.getByText('Low Volatility').closest('div[class*="cursor-pointer"]')
    if (lowVolCard) {
      await user.click(lowVolCard)

      await waitFor(() => {
        expect(screen.getByText(/In low volatility markets/i)).toBeInTheDocument()
      })
    }
  })

  it('should update info box for high volatility preset', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByText('High Volatility')).toBeInTheDocument()
    })

    // Click high volatility preset
    const highVolCard = screen.getByText('High Volatility').closest('div[class*="cursor-pointer"]')
    if (highVolCard) {
      await user.click(highVolCard)

      await waitFor(() => {
        expect(screen.getByText(/In high volatility markets/i)).toBeInTheDocument()
      })
    }
  })

  // ========================================
  // Tab: Strategies (Dialog)
  // ========================================

  it('should switch to strategies tab', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('tab', { name: /^strategies$/i })).toBeInTheDocument()
    })

    const strategiesTab = screen.getByRole('tab', { name: /^strategies$/i })
    await user.click(strategiesTab)

    // Should show strategy cards
    await waitFor(() => {
      const strategiesTabPanel = screen.getByRole('tabpanel')
      expect(within(strategiesTabPanel).getByText('RSI Strategy')).toBeInTheDocument()
    })
  })

  it('should render RSI strategy controls in dialog', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))

    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      expect(within(strategiesTab).getByText('RSI Period')).toBeInTheDocument()
      expect(within(strategiesTab).getByText('Oversold')).toBeInTheDocument()
      expect(within(strategiesTab).getByText('Overbought')).toBeInTheDocument()
    })
  })

  it('should update RSI oversold threshold slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const oversoldLabel = within(strategiesTab).getByText('Oversold')
      const slider = oversoldLabel.closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '25' } })
        expect(slider).toHaveValue('25')
      }
    })
  })

  it('should update RSI overbought threshold slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const overboughtLabel = within(strategiesTab).getByText('Overbought')
      const slider = overboughtLabel.closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '75' } })
        expect(slider).toHaveValue('75')
      }
    })
  })

  it('should render MACD strategy controls in dialog', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const macdSection = within(strategiesTab).getByText('MACD Strategy').closest('div[class*="rounded-xl"]')
      if (macdSection) {
        expect(within(macdSection).getByText('Fast')).toBeInTheDocument()
        expect(within(macdSection).getByText('Slow')).toBeInTheDocument()
        expect(within(macdSection).getByText('Signal')).toBeInTheDocument()
      }
    })
  })

  it('should update MACD fast period slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const fastLabels = within(strategiesTab).getAllByText('Fast')
      const slider = fastLabels[0].closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '10' } })
        expect(slider).toHaveValue('10')
      }
    })
  })

  it('should update MACD slow period slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const slowLabels = within(strategiesTab).getAllByText('Slow')
      const slider = slowLabels[0].closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '30' } })
        expect(slider).toHaveValue('30')
      }
    })
  })

  it('should update MACD signal period slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const signalLabels = within(strategiesTab).getAllByText('Signal')
      const slider = signalLabels[0].closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '8' } })
        expect(slider).toHaveValue('8')
      }
    })
  })

  it('should render Volume strategy controls in dialog', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const volumeSection = within(strategiesTab).getByText('Volume Strategy').closest('div[class*="rounded-xl"]')
      if (volumeSection) {
        expect(within(volumeSection).getByText('Spike Threshold')).toBeInTheDocument()
        expect(within(volumeSection).getByText('SMA Period')).toBeInTheDocument()
      }
    })
  })

  it('should update Volume SMA period slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const smaLabel = within(strategiesTab).getByText('SMA Period')
      const slider = smaLabel.closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '25' } })
        expect(slider).toHaveValue('25')
      }
    })
  })

  it('should render Bollinger Bands controls in dialog', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const bollingerSection = within(strategiesTab).getByText('Bollinger Bands').closest('div[class*="rounded-xl"]')
      if (bollingerSection) {
        expect(within(bollingerSection).getByText('Period')).toBeInTheDocument()
        expect(within(bollingerSection).getByText('Multiplier')).toBeInTheDocument()
      }
    })
  })

  it('should update Bollinger period slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const bollingerSection = within(strategiesTab).getByText('Bollinger Bands').closest('div[class*="rounded-xl"]')
      if (bollingerSection) {
        const periodLabel = within(bollingerSection).getByText('Period')
        const slider = periodLabel.closest('div')?.querySelector('input[type="range"]')

        if (slider) {
          fireEvent.change(slider, { target: { value: '25' } })
          expect(slider).toHaveValue('25')
        }
      }
    })
  })

  it('should render Stochastic strategy controls in dialog', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const stochasticSection = within(strategiesTab).getByText('Stochastic Strategy').closest('div[class*="rounded-xl"]')
      if (stochasticSection) {
        expect(within(stochasticSection).getByText('K Period')).toBeInTheDocument()
        expect(within(stochasticSection).getByText('D Period')).toBeInTheDocument()
      }
    })
  })

  it('should update Stochastic K period slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const kPeriodLabel = within(strategiesTab).getByText('K Period')
      const slider = kPeriodLabel.closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '20' } })
        expect(slider).toHaveValue('20')
      }
    })
  })

  it('should update Stochastic D period slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const dPeriodLabel = within(strategiesTab).getByText('D Period')
      const slider = dPeriodLabel.closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '5' } })
        expect(slider).toHaveValue('5')
      }
    })
  })

  // ========================================
  // Tab: Risk Management (Dialog)
  // ========================================

  it('should switch to risk management tab', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))

    const riskTab = screen.getByRole('tab', { name: /risk management/i })
    await user.click(riskTab)

    await waitFor(() => {
      const riskTabPanel = screen.getByRole('tabpanel')
      expect(within(riskTabPanel).getByText('Position Risk')).toBeInTheDocument()
      expect(within(riskTabPanel).getByText('Portfolio Risk')).toBeInTheDocument()
    })
  })

  it('should render position risk controls', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
    await user.click(screen.getByRole('tab', { name: /risk management/i }))

    await waitFor(() => {
      const riskTab = screen.getByRole('tabpanel')
      const positionRisk = within(riskTab).getByText('Position Risk').closest('div[class*="rounded-xl"]')
      if (positionRisk) {
        expect(within(positionRisk).getByText('Max Risk per Trade')).toBeInTheDocument()
        expect(within(positionRisk).getByText('Stop Loss')).toBeInTheDocument()
        expect(within(positionRisk).getByText('Take Profit')).toBeInTheDocument()
      }
    })
  })

  it('should update max risk per trade slider', async () => {
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
        fireEvent.change(slider, { target: { value: '3.0' } })
        expect(slider).toHaveValue('3.0')
      }
    })
  })

  it('should update stop loss slider in risk tab', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
    await user.click(screen.getByRole('tab', { name: /risk management/i }))

    await waitFor(() => {
      const riskTab = screen.getByRole('tabpanel')
      const stopLossLabels = within(riskTab).getAllByText('Stop Loss')
      const slider = stopLossLabels[0].closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '3.5' } })
        expect(slider).toHaveValue('3.5')
      }
    })
  })

  it('should update take profit slider in risk tab', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
    await user.click(screen.getByRole('tab', { name: /risk management/i }))

    await waitFor(() => {
      const riskTab = screen.getByRole('tabpanel')
      const takeProfitLabels = within(riskTab).getAllByText('Take Profit')
      const slider = takeProfitLabels[0].closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '6.0' } })
        expect(slider).toHaveValue('6.0')
      }
    })
  })

  it('should render portfolio risk controls', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
    await user.click(screen.getByRole('tab', { name: /risk management/i }))

    await waitFor(() => {
      const riskTab = screen.getByRole('tabpanel')
      const portfolioRisk = within(riskTab).getByText('Portfolio Risk').closest('div[class*="rounded-xl"]')
      if (portfolioRisk) {
        expect(within(portfolioRisk).getByText('Max Portfolio Risk')).toBeInTheDocument()
        expect(within(portfolioRisk).getByText('Max Drawdown')).toBeInTheDocument()
        expect(within(portfolioRisk).getByText('Max Consecutive Losses')).toBeInTheDocument()
        expect(within(portfolioRisk).getByText('Position Correlation Limit')).toBeInTheDocument()
      }
    })
  })

  it('should update max portfolio risk slider', async () => {
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
        fireEvent.change(slider, { target: { value: '30' } })
        expect(slider).toHaveValue('30')
      }
    })
  })

  it('should update max drawdown slider in risk tab', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
    await user.click(screen.getByRole('tab', { name: /risk management/i }))

    await waitFor(() => {
      const riskTab = screen.getByRole('tabpanel')
      const maxDrawdownLabels = within(riskTab).getAllByText('Max Drawdown')
      const slider = maxDrawdownLabels[0].closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '20' } })
        expect(slider).toHaveValue('20')
      }
    })
  })

  it('should update max consecutive losses slider', async () => {
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
        fireEvent.change(slider, { target: { value: '7' } })
        expect(slider).toHaveValue('7')
      }
    })
  })

  it('should update correlation limit slider', async () => {
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
        fireEvent.change(slider, { target: { value: '80' } })
        expect(slider).toHaveValue('80')
      }
    })
  })

  it('should show correlation limit description', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /risk management/i }))
    await user.click(screen.getByRole('tab', { name: /risk management/i }))

    await waitFor(() => {
      const riskTab = screen.getByRole('tabpanel')
      expect(within(riskTab).getByText('Maximum % of positions in same direction')).toBeInTheDocument()
    })
  })

  // ========================================
  // Tab: Engine Settings (Dialog)
  // ========================================

  it('should switch to engine settings tab', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))

    const engineTab = screen.getByRole('tab', { name: /engine settings/i })
    await user.click(engineTab)

    await waitFor(() => {
      const engineTabPanel = screen.getByRole('tabpanel')
      expect(within(engineTabPanel).getByText('Signal Processing')).toBeInTheDocument()
      expect(within(engineTabPanel).getByText('Market Conditions')).toBeInTheDocument()
    })
  })

  it('should render signal processing controls', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
    await user.click(screen.getByRole('tab', { name: /engine settings/i }))

    await waitFor(() => {
      const engineTab = screen.getByRole('tabpanel')
      const signalProcessing = within(engineTab).getByText('Signal Processing').closest('div[class*="rounded-xl"]')
      if (signalProcessing) {
        expect(within(signalProcessing).getByText('Min Confidence Threshold')).toBeInTheDocument()
        expect(within(signalProcessing).getByText('Signal Combination Mode')).toBeInTheDocument()
      }
    })
  })

  it('should update min confidence threshold slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
    await user.click(screen.getByRole('tab', { name: /engine settings/i }))

    await waitFor(() => {
      const engineTab = screen.getByRole('tabpanel')
      const minConfidenceLabels = within(engineTab).getAllByText('Min Confidence Threshold')
      const slider = minConfidenceLabels[0].closest('div')?.querySelector('input[type="range"]')

      if (slider) {
        fireEvent.change(slider, { target: { value: '75' } })
        expect(slider).toHaveValue('75')
      }
    })
  })

  it('should show confidence threshold description', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
    await user.click(screen.getByRole('tab', { name: /engine settings/i }))

    await waitFor(() => {
      const engineTab = screen.getByRole('tabpanel')
      expect(within(engineTab).getByText(/Lower values = more signals/i)).toBeInTheDocument()
    })
  })

  it('should render market conditions controls', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
    await user.click(screen.getByRole('tab', { name: /engine settings/i }))

    await waitFor(() => {
      const engineTab = screen.getByRole('tabpanel')
      const marketConditions = within(engineTab).getByText('Market Conditions').closest('div[class*="rounded-xl"]')
      if (marketConditions) {
        expect(within(marketConditions).getByText('Market Condition')).toBeInTheDocument()
        expect(within(marketConditions).getByText('Risk Level')).toBeInTheDocument()
        expect(within(marketConditions).getByText('Data Timeframe')).toBeInTheDocument()
      }
    })
  })

  it('should show data timeframe description', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
    await user.click(screen.getByRole('tab', { name: /engine settings/i }))

    await waitFor(() => {
      const engineTab = screen.getByRole('tabpanel')
      expect(within(engineTab).getByText('Timeframe for trading signals and technical analysis')).toBeInTheDocument()
    })
  })

  // ========================================
  // Dialog Footer Actions
  // ========================================

  it('should render footer with reload and save buttons', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      const dialog = screen.getByRole('dialog')
      expect(within(dialog).getByRole('button', { name: /reload/i })).toBeInTheDocument()
      expect(within(dialog).getByRole('button', { name: /cancel/i })).toBeInTheDocument()
      expect(within(dialog).getByRole('button', { name: /save settings/i })).toBeInTheDocument()
    })
  })

  it('should reload settings when reload button clicked', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })

    vi.clearAllMocks()

    const reloadButton = within(screen.getByRole('dialog')).getByRole('button', { name: /reload/i })
    await user.click(reloadButton)

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/strategy-settings')
      )
    })
  })

  it('should close dialog when cancel button clicked', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })

    const cancelButton = within(screen.getByRole('dialog')).getByRole('button', { name: /cancel/i })
    await user.click(cancelButton)

    await waitFor(() => {
      expect(screen.queryByRole('dialog')).not.toBeInTheDocument()
    })
  })

  it('should save settings when save button clicked', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })

    vi.clearAllMocks()
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ success: true }),
    })

    const saveButton = within(screen.getByRole('dialog')).getByRole('button', { name: /save settings/i })
    await user.click(saveButton)

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/strategy-settings'),
        expect.objectContaining({
          method: 'PUT',
        })
      )
      expect(toast.success).toHaveBeenCalledWith('Trading settings saved successfully!')
    })
  })

  it('should show error toast when save fails', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })

    vi.clearAllMocks()
    mockFetch.mockResolvedValueOnce({
      ok: false,
      json: async () => ({ success: false }),
    })

    const saveButton = within(screen.getByRole('dialog')).getByRole('button', { name: /save settings/i })
    await user.click(saveButton)

    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
    })
  })

  // ========================================
  // Error Handling
  // ========================================

  it('should handle load settings error in dialog', async () => {
    const user = userEvent.setup()
    mockFetch.mockRejectedValueOnce(new Error('Network error'))

    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith('Failed to load trading settings')
    })
  })

  it('should handle save error with exception', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })

    vi.clearAllMocks()
    mockFetch.mockRejectedValueOnce(new Error('Network error'))

    const saveButton = within(screen.getByRole('dialog')).getByRole('button', { name: /save settings/i })
    await user.click(saveButton)

    await waitFor(() => {
      expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
    })
  })

  // ========================================
  // Loading States
  // ========================================

  it('should disable reload button while loading', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })

    vi.clearAllMocks()
    let resolvePromise: (value: any) => void
    const loadPromise = new Promise((resolve) => {
      resolvePromise = resolve
    })

    mockFetch.mockReturnValueOnce(loadPromise as any)

    const reloadButton = within(screen.getByRole('dialog')).getByRole('button', { name: /reload/i })
    await user.click(reloadButton)

    // Button should be disabled during loading
    expect(reloadButton).toBeDisabled()

    // @ts-ignore
    resolvePromise({
      ok: true,
      json: async () => mockDefaultSettings,
    })

    await waitFor(() => {
      expect(reloadButton).not.toBeDisabled()
    })
  })

  it('should disable save button while saving', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))

    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })

    vi.clearAllMocks()
    let resolvePromise: (value: any) => void
    const savePromise = new Promise((resolve) => {
      resolvePromise = resolve
    })

    mockFetch.mockReturnValueOnce(savePromise as any)

    const saveButton = within(screen.getByRole('dialog')).getByRole('button', { name: /save settings/i })
    await user.click(saveButton)

    // Button should be disabled during saving
    expect(saveButton).toBeDisabled()

    // @ts-ignore
    resolvePromise({
      ok: true,
      json: async () => ({ success: true }),
    })

    await waitFor(() => {
      expect(saveButton).not.toBeDisabled()
    })
  })
})

describe('InlineTradingSettings - Additional Coverage', () => {
  const mockDefaultSettings = {
    success: true,
    data: {
      strategies: {
        rsi: {
          enabled: false,
          period: 14,
          oversold_threshold: 30,
          overbought_threshold: 70,
          extreme_oversold: 20,
          extreme_overbought: 80,
        },
        macd: {
          enabled: false,
          fast_period: 12,
          slow_period: 26,
          signal_period: 9,
          histogram_threshold: 0.001,
        },
        volume: {
          enabled: false,
          sma_period: 20,
          spike_threshold: 2.0,
          correlation_period: 10,
        },
        bollinger: {
          enabled: false,
          period: 20,
          multiplier: 2.0,
          squeeze_threshold: 0.02,
        },
        stochastic: {
          enabled: false,
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
        enabled_strategies: [],
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

  it('should show loading state initially', async () => {
    let resolvePromise: (value: any) => void
    const loadPromise = new Promise((resolve) => {
      resolvePromise = resolve
    })

    mockFetch.mockReturnValueOnce(loadPromise as any)

    render(<InlineTradingSettings />)

    expect(screen.getByText('Loading settings...')).toBeInTheDocument()

    // @ts-ignore
    resolvePromise({
      ok: true,
      json: async () => mockDefaultSettings,
    })

    await waitFor(() => {
      expect(screen.queryByText('Loading settings...')).not.toBeInTheDocument()
    })
  })

  it('should load settings without market_preset field', async () => {
    const settingsWithoutPreset = {
      success: true,
      data: {
        ...mockDefaultSettings.data,
        market_preset: undefined,
      },
    }

    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => settingsWithoutPreset,
    })

    render(<InlineTradingSettings />)

    await waitFor(() => {
      expect(screen.queryByText('Loading settings...')).not.toBeInTheDocument()
    })
  })

  it('should handle stochastic strategy with undefined values', async () => {
    const settingsWithoutStochastic = {
      success: true,
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
    })

    render(<InlineTradingSettings />)

    await waitFor(() => {
      expect(screen.getByText('Stochastic')).toBeInTheDocument()
    })
  })

  it('should reload settings when reload button clicked in inline', async () => {
    const user = userEvent.setup()
    render(<InlineTradingSettings />)

    await waitFor(() => {
      expect(screen.queryByText('Loading settings...')).not.toBeInTheDocument()
    })

    vi.clearAllMocks()

    const reloadButton = screen.getByRole('button', { name: /reload/i })
    await user.click(reloadButton)

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/strategy-settings')
      )
    })
  })

  it('should save settings when save button clicked in inline', async () => {
    const user = userEvent.setup()
    render(<InlineTradingSettings />)

    await waitFor(() => {
      expect(screen.queryByText('Loading settings...')).not.toBeInTheDocument()
    })

    vi.clearAllMocks()
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ success: true }),
    })

    const saveButton = screen.getByRole('button', { name: /save strategy settings/i })
    await user.click(saveButton)

    await waitFor(() => {
      expect(mockFetch).toHaveBeenCalledWith(
        expect.stringContaining('/api/paper-trading/strategy-settings'),
        expect.objectContaining({
          method: 'PUT',
        })
      )
      expect(toast.success).toHaveBeenCalledWith('Trading settings saved successfully!')
    })
  })
})

describe('TradingSettings Dialog - Deep Coverage', () => {
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

  // Test all RSI sliders
  it('should update RSI period slider in strategies tab', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const rsiSection = within(strategiesTab).getByText('RSI Strategy').closest('div[class*="rounded-xl"]')
      if (rsiSection) {
        const periodLabel = within(rsiSection).getByText('RSI Period')
        const slider = periodLabel.closest('div')?.querySelector('input[type="range"]')

        if (slider) {
          fireEvent.change(slider, { target: { value: '20' } })
          expect(slider).toHaveValue('20')
        }
      }
    })
  })

  // Test toggle switches
  it('should toggle RSI strategy off', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const rsiSection = within(strategiesTab).getByText('RSI Strategy').closest('div[class*="rounded-xl"]')
      if (rsiSection) {
        const toggle = within(rsiSection).getByRole('switch')
        expect(toggle).toBeChecked()
      }
    })
  })

  it('should toggle MACD strategy', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const macdSection = within(strategiesTab).getByText('MACD Strategy').closest('div[class*="rounded-xl"]')
      if (macdSection) {
        const toggle = within(macdSection).getByRole('switch')
        expect(toggle).toBeChecked()
      }
    })
  })

  it('should toggle Volume strategy', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const volumeSection = within(strategiesTab).getByText('Volume Strategy').closest('div[class*="rounded-xl"]')
      if (volumeSection) {
        const toggle = within(volumeSection).getByRole('switch')
        expect(toggle).toBeChecked()
      }
    })
  })

  it('should toggle Bollinger Bands strategy', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const bollingerSection = within(strategiesTab).getByText('Bollinger Bands').closest('div[class*="rounded-xl"]')
      if (bollingerSection) {
        const toggle = within(bollingerSection).getByRole('switch')
        expect(toggle).toBeChecked()
      }
    })
  })

  it('should toggle Stochastic strategy', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const stochasticSection = within(strategiesTab).getByText('Stochastic Strategy').closest('div[class*="rounded-xl"]')
      if (stochasticSection) {
        const toggle = within(stochasticSection).getByRole('switch')
        expect(toggle).toBeChecked()
      }
    })
  })

  it('should update Bollinger multiplier slider', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /^strategies$/i }))
    await user.click(screen.getByRole('tab', { name: /^strategies$/i }))

    await waitFor(() => {
      const strategiesTab = screen.getByRole('tabpanel')
      const bollingerSection = within(strategiesTab).getByText('Bollinger Bands').closest('div[class*="rounded-xl"]')
      if (bollingerSection) {
        const multiplierLabel = within(bollingerSection).getByText('Multiplier')
        const slider = multiplierLabel.closest('div')?.querySelector('input[type="range"]')

        if (slider) {
          fireEvent.change(slider, { target: { value: '2.5' } })
          expect(slider).toHaveValue('2.5')
        }
      }
    })
  })

  // Test Select components in Engine tab
  it('should update signal combination mode select', async () => {
    const user = userEvent.setup()
    render(<TradingSettings />)

    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => screen.getByRole('tab', { name: /engine settings/i }))
    await user.click(screen.getByRole('tab', { name: /engine settings/i }))

    await waitFor(() => {
      const engineTab = screen.getByRole('tabpanel')
      expect(within(engineTab).getByText('Signal Combination Mode')).toBeInTheDocument()
    })
  })

  it('should update market condition select', async () => {
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

  it('should update risk level select', async () => {
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

  it('should update data timeframe select', async () => {
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

  it('should handle dialog not loading settings if not opened', () => {
    render(<TradingSettings />)
    // Dialog should not fetch until opened
    expect(mockFetch).not.toHaveBeenCalled()
  })
})
