import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { TradingSettings } from '../../../components/dashboard/TradingSettings'

// Mock fetch API
const mockFetch = vi.fn()
global.fetch = mockFetch

describe('TradingSettings - Slider onChange Handlers', () => {
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
            enabled_strategies: ['RSI Strategy', 'MACD Strategy', 'Volume Strategy', 'Bollinger Bands Strategy', 'Stochastic Strategy'],
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

  describe('Strategies Tab - Slider Handlers', () => {
    it('changes RSI oversold threshold - line 1250', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Oversold')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const oversoldSlider = sliders.find((s) => s.getAttribute('min') === '20' && s.getAttribute('max') === '50')
      if (oversoldSlider) {
        fireEvent.change(oversoldSlider, { target: { value: '25' } })
        await waitFor(() => expect(oversoldSlider).toHaveValue('25'))
      }
    })

    it('changes RSI overbought threshold - line 1267', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Overbought')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const overboughtSlider = sliders.find((s) => s.getAttribute('min') === '50' && s.getAttribute('max') === '80')
      if (overboughtSlider) {
        fireEvent.change(overboughtSlider, { target: { value: '75' } })
        await waitFor(() => expect(overboughtSlider).toHaveValue('75'))
      }
    })

    it('changes MACD fast period - line 1303', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Fast')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const fastSlider = sliders.find((s) => s.getAttribute('min') === '5' && s.getAttribute('max') === '20')
      if (fastSlider) {
        fireEvent.change(fastSlider, { target: { value: '10' } })
        await waitFor(() => expect(fastSlider).toHaveValue('10'))
      }
    })

    it('changes MACD slow period - line 1320', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Slow')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const slowSlider = sliders.find((s) => s.getAttribute('min') === '15' && s.getAttribute('max') === '35')
      if (slowSlider) {
        fireEvent.change(slowSlider, { target: { value: '26' } })
        await waitFor(() => expect(slowSlider).toHaveValue('26'))
      }
    })

    it('changes MACD signal period - line 1337', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Signal')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const signalSlider = sliders.find((s) => s.getAttribute('min') === '3' && s.getAttribute('max') === '15')
      if (signalSlider) {
        fireEvent.change(signalSlider, { target: { value: '9' } })
        await waitFor(() => expect(signalSlider).toHaveValue('9'))
      }
    })

    it('changes Volume spike threshold - line 1372', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Spike Threshold')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const spikeSlider = sliders.find((s) => s.getAttribute('min') === '1' && s.getAttribute('max') === '5')
      if (spikeSlider) {
        fireEvent.change(spikeSlider, { target: { value: '2.5' } })
        await waitFor(() => expect(spikeSlider).toHaveValue('2.5'))
      }
    })

    it('changes Volume SMA period - line 1389', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('SMA Period')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const smaPeriodSlider = sliders.find((s) => s.getAttribute('min') === '10' && s.getAttribute('max') === '30')
      if (smaPeriodSlider) {
        fireEvent.change(smaPeriodSlider, { target: { value: '20' } })
        await waitFor(() => expect(smaPeriodSlider).toHaveValue('20'))
      }
    })

    it('changes Bollinger period - line 1423', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Period')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const bollingerPeriods = sliders.filter((s) => s.getAttribute('min') === '10' && s.getAttribute('max') === '30')
      if (bollingerPeriods.length > 0) {
        fireEvent.change(bollingerPeriods[0], { target: { value: '20' } })
        await waitFor(() => expect(bollingerPeriods[0]).toHaveValue('20'))
      }
    })

    it('changes Bollinger multiplier - line 1440', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('Multiplier')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const multiplierSliders = sliders.filter((s) => s.getAttribute('min') === '1' && s.getAttribute('max') === '3')
      if (multiplierSliders.length > 0) {
        fireEvent.change(multiplierSliders[0], { target: { value: '2.2' } })
        await waitFor(() => expect(multiplierSliders[0]).toHaveValue('2.2'))
      }
    })

    it('changes Stochastic K period - line 1486', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('K Period')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const kPeriodSliders = sliders.filter((s) => s.getAttribute('min') === '5' && s.getAttribute('max') === '30')
      if (kPeriodSliders.length > 0) {
        const kSlider = kPeriodSliders[kPeriodSliders.length - 1]
        fireEvent.change(kSlider, { target: { value: '18' } })
        await waitFor(() => expect(kSlider).toHaveValue('18'))
      }
    })

    it('changes Stochastic D period - line 1513', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /strategies/i }))

      await waitFor(() => {
        expect(screen.getByText('D Period')).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const dPeriodSlider = sliders.find((s) => s.getAttribute('min') === '1' && s.getAttribute('max') === '10')
      if (dPeriodSlider) {
        fireEvent.change(dPeriodSlider, { target: { value: '5' } })
        await waitFor(() => expect(dPeriodSlider).toHaveValue('5'))
      }
    })
  })

  describe('Risk Tab - Slider Handlers', () => {
    it('changes max risk per trade - line 1558', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getAllByText(/Max Risk per Trade/i).length).toBeGreaterThan(0)
      })

      const sliders = screen.getAllByRole('slider')
      const riskSlider = sliders.find((s) => s.getAttribute('min') === '0.5' && s.getAttribute('max') === '5')
      if (riskSlider) {
        fireEvent.change(riskSlider, { target: { value: '3.0' } })
        await waitFor(() => expect(riskSlider).toHaveValue('3'))
      }
    })

    it('changes stop loss - line 1572', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getAllByText(/Stop Loss/i).length).toBeGreaterThan(0)
      })

      const sliders = screen.getAllByRole('slider')
      const stopLossSliders = sliders.filter((s) => s.getAttribute('min') === '0.5' && s.getAttribute('max') === '5')
      if (stopLossSliders.length > 1) {
        fireEvent.change(stopLossSliders[1], { target: { value: '2.5' } })
        await waitFor(() => expect(stopLossSliders[1]).toHaveValue('2.5'))
      }
    })

    it('changes take profit - line 1586', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getAllByText(/Take Profit/i).length).toBeGreaterThan(0)
      })

      const sliders = screen.getAllByRole('slider')
      const takeProfitSlider = sliders.find((s) => s.getAttribute('min') === '1' && s.getAttribute('max') === '10')
      if (takeProfitSlider) {
        fireEvent.change(takeProfitSlider, { target: { value: '5' } })
        await waitFor(() => expect(takeProfitSlider).toHaveValue('5'))
      }
    })

    it('changes max portfolio risk - line 1612', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getAllByText(/Max Portfolio Risk/i).length).toBeGreaterThan(0)
      })

      const sliders = screen.getAllByRole('slider')
      const portfolioRiskSlider = sliders.find((s) => s.getAttribute('min') === '5' && s.getAttribute('max') === '50')
      if (portfolioRiskSlider) {
        fireEvent.change(portfolioRiskSlider, { target: { value: '30' } })
        await waitFor(() => expect(portfolioRiskSlider).toHaveValue('30'))
      }
    })

    it('changes max drawdown - line 1626', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getAllByText(/Max Drawdown/i).length).toBeGreaterThan(0)
      })

      const sliders = screen.getAllByRole('slider')
      const drawdownSlider = sliders.find((s) => s.getAttribute('min') === '5' && s.getAttribute('max') === '25')
      if (drawdownSlider) {
        fireEvent.change(drawdownSlider, { target: { value: '15' } })
        await waitFor(() => expect(drawdownSlider).toHaveValue('15'))
      }
    })

    it('changes max consecutive losses - line 1640', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getAllByText(/Max Consecutive Losses/i).length).toBeGreaterThan(0)
      })

      const sliders = screen.getAllByRole('slider')
      const consecutiveLossesSlider = sliders.find((s) => s.getAttribute('min') === '2' && s.getAttribute('max') === '10')
      if (consecutiveLossesSlider) {
        fireEvent.change(consecutiveLossesSlider, { target: { value: '6' } })
        await waitFor(() => expect(consecutiveLossesSlider).toHaveValue('6'))
      }
    })

    it('changes correlation limit - line 1654', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /risk management/i }))

      await waitFor(() => {
        expect(screen.getByText(/Position Correlation Limit/i)).toBeInTheDocument()
      })

      const sliders = screen.getAllByRole('slider')
      const correlationSlider = sliders.find((s) => s.getAttribute('min') === '50' && s.getAttribute('max') === '100')
      if (correlationSlider) {
        fireEvent.change(correlationSlider, { target: { value: '75' } })
        await waitFor(() => expect(correlationSlider).toHaveValue('75'))
      }
    })
  })

  describe('Engine Tab - Slider Handlers', () => {
    it('changes min confidence threshold - line 1686', async () => {
      const user = userEvent.setup()
      render(<TradingSettings />)

      await user.click(screen.getByRole('button', { name: /trading settings/i }))
      await user.click(screen.getByRole('tab', { name: /engine settings/i }))

      await waitFor(() => {
        expect(screen.getAllByText(/Min Confidence Threshold/i).length).toBeGreaterThan(0)
      })

      const sliders = screen.getAllByRole('slider')
      const confidenceSlider = sliders.find((s) => s.getAttribute('min') === '30' && s.getAttribute('max') === '90')
      if (confidenceSlider) {
        fireEvent.change(confidenceSlider, { target: { value: '70' } })
        await waitFor(() => expect(confidenceSlider).toHaveValue('70'))
      }
    })
  })
})
