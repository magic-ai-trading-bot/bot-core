import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { screen, waitFor, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { TradingSettings } from '../../../components/dashboard/TradingSettings'

// Mock scrollIntoView for Radix UI
beforeEach(() => {
  Element.prototype.scrollIntoView = vi.fn()
})

// Mock fetch API
const mockFetch = vi.fn()
global.fetch = mockFetch

// Mock sonner toast
vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

// Mock framer-motion
vi.mock('framer-motion', () => {
  const createMotionComponent = (tag: string) => {
    const Component = ({ children, whileHover, whileTap, animate, transition, initial, exit, variants, ...props }: any) => {
      const Tag = tag as any
      return <Tag {...props}>{children}</Tag>
    }
    Component.displayName = `motion.${tag}`
    return Component
  }
  return {
    motion: new Proxy({}, {
      get: (_target, prop: string) => createMotionComponent(prop),
    }),
    AnimatePresence: ({ children }: any) => <>{children}</>,
  }
})

const mockSettings = {
  success: true,
  data: {
    strategies: {
      rsi: { enabled: true, period: 14, oversold_threshold: 30, overbought_threshold: 70, extreme_oversold: 20, extreme_overbought: 80 },
      macd: { enabled: true, fast_period: 12, slow_period: 26, signal_period: 9, histogram_threshold: 0.001 },
      volume: { enabled: true, sma_period: 20, spike_threshold: 2.0, correlation_period: 10 },
      bollinger: { enabled: true, period: 20, multiplier: 2.0, squeeze_threshold: 0.02 },
      stochastic: { enabled: true, k_period: 14, d_period: 3, oversold_threshold: 20.0, overbought_threshold: 80.0, extreme_oversold: 10.0, extreme_overbought: 90.0 },
    },
    risk: {
      max_risk_per_trade: 2.0, max_portfolio_risk: 20, stop_loss_percent: 2.0,
      take_profit_percent: 4.0, max_leverage: 50, max_drawdown: 15,
      daily_loss_limit: 5, max_consecutive_losses: 5, correlation_limit: 0.7,
    },
    engine: {
      min_confidence_threshold: 0.65, signal_combination_mode: 'WeightedAverage',
      enabled_strategies: ['RSI Strategy', 'MACD Strategy', 'Volume Strategy', 'Bollinger Bands Strategy', 'Stochastic Strategy'],
      market_condition: 'Trending', risk_level: 'Moderate', data_resolution: '15m',
    },
    market_preset: 'normal_volatility',
  },
}

async function openDialogAndSwitchTab(user: ReturnType<typeof userEvent.setup>, tabName: string) {
  render(<TradingSettings />)
  await waitFor(() => {
    expect(screen.getByRole('button', { name: /trading settings/i })).toBeInTheDocument()
  })
  await user.click(screen.getByRole('button', { name: /trading settings/i }))
  await waitFor(() => {
    expect(screen.getByRole('dialog')).toBeInTheDocument()
  })
  // Wait for settings to load
  await waitFor(() => {
    expect(screen.getByRole('tab', { name: new RegExp(tabName, 'i') })).toBeInTheDocument()
  })
  const tab = screen.getByRole('tab', { name: new RegExp(tabName, 'i') })
  await user.click(tab)
  // Wait for tab content to render
  await new Promise(r => setTimeout(r, 100))
}

describe('TradingSettings - Risk Management Tab', () => {
  const user = userEvent.setup()

  beforeEach(() => {
    vi.clearAllMocks()
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockSettings,
      clone: function() { return this; },
    })
  })

  afterEach(() => { vi.clearAllMocks() })

  it('should render risk management tab content', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Position Risk')).toBeInTheDocument()
    })
  })

  it('should render portfolio risk section', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Portfolio Risk')).toBeInTheDocument()
    })
  })

  it('should display max risk per trade slider', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Max Risk per Trade')).toBeInTheDocument()
    })
  })

  it('should display stop loss slider', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Stop Loss')).toBeInTheDocument()
    })
  })

  it('should display take profit slider', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Take Profit')).toBeInTheDocument()
    })
  })

  it('should display max portfolio risk slider', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Max Portfolio Risk')).toBeInTheDocument()
    })
  })

  it('should display max drawdown slider', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Max Drawdown')).toBeInTheDocument()
    })
  })

  it('should display max consecutive losses slider', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Max Consecutive Losses')).toBeInTheDocument()
    })
  })

  it('should display position correlation limit slider', async () => {
    await openDialogAndSwitchTab(user, 'risk')
    await waitFor(() => {
      expect(screen.getByText('Position Correlation Limit')).toBeInTheDocument()
    })
  })
})

describe('TradingSettings - Engine Settings Tab', () => {
  const user = userEvent.setup()

  beforeEach(() => {
    vi.clearAllMocks()
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockSettings,
      clone: function() { return this; },
    })
  })

  afterEach(() => { vi.clearAllMocks() })

  it('should render engine settings tab content', async () => {
    await openDialogAndSwitchTab(user, 'engine')
    await waitFor(() => {
      expect(screen.getByText('Signal Processing')).toBeInTheDocument()
    })
  })

  it('should render market conditions section', async () => {
    await openDialogAndSwitchTab(user, 'engine')
    await waitFor(() => {
      expect(screen.getByText('Market Conditions')).toBeInTheDocument()
    })
  })

  it('should display min confidence threshold slider', async () => {
    await openDialogAndSwitchTab(user, 'engine')
    await waitFor(() => {
      expect(screen.getByText('Min Confidence Threshold')).toBeInTheDocument()
    })
  })

  it('should display signal combination mode select', async () => {
    await openDialogAndSwitchTab(user, 'engine')
    await waitFor(() => {
      expect(screen.getByText('Signal Combination Mode')).toBeInTheDocument()
    })
  })

  it('should display market condition select', async () => {
    await openDialogAndSwitchTab(user, 'engine')
    await waitFor(() => {
      expect(screen.getByText('Market Condition')).toBeInTheDocument()
    })
  })

  it('should display risk level select', async () => {
    await openDialogAndSwitchTab(user, 'engine')
    await waitFor(() => {
      expect(screen.getByText('Risk Level')).toBeInTheDocument()
    })
  })

  it('should display data timeframe select', async () => {
    await openDialogAndSwitchTab(user, 'engine')
    await waitFor(() => {
      expect(screen.getByText('Data Timeframe')).toBeInTheDocument()
    })
  })
})

describe('TradingSettings - Stochastic Strategy Tab', () => {
  const user = userEvent.setup()

  beforeEach(() => {
    vi.clearAllMocks()
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockSettings,
      clone: function() { return this; },
    })
  })

  afterEach(() => { vi.clearAllMocks() })

  it('should render stochastic strategy card in strategies tab', async () => {
    await openDialogAndSwitchTab(user, 'strategies')
    await waitFor(() => {
      expect(screen.getByText('Stochastic Strategy')).toBeInTheDocument()
    })
  })

  it('should display stochastic K Period slider', async () => {
    await openDialogAndSwitchTab(user, 'strategies')
    await waitFor(() => {
      expect(screen.getByText('K Period')).toBeInTheDocument()
    })
  })

  it('should display stochastic D Period slider', async () => {
    await openDialogAndSwitchTab(user, 'strategies')
    await waitFor(() => {
      expect(screen.getByText('D Period')).toBeInTheDocument()
    })
  })
})
