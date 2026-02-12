import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { render, screen, waitFor } from '../../../test/utils'
import { TradingSettings, InlineTradingSettings } from '@/components/dashboard/TradingSettings'
import userEvent from '@testing-library/user-event'
import { toast } from 'sonner'

// Mock framer-motion with Proxy pattern
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
    motion: new Proxy({}, { get: (_target, prop: string) => createMotionComponent(prop) }),
    AnimatePresence: ({ children }: any) => <>{children}</>,
  }
})

// Mock sonner
vi.mock('sonner', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
  },
}))

describe('TradingSettings - Save Flow', () => {
  const mockSettings = {
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
  }

  beforeEach(() => {
    Element.prototype.scrollIntoView = vi.fn()
    global.fetch = vi.fn()
    vi.clearAllMocks()
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  describe('TradingSettings Dialog', () => {
    it('should save settings successfully when Save Settings button is clicked', async () => {
      const user = userEvent.setup()

      // Mock successful GET and PUT requests
      ;(global.fetch as any).mockImplementation((url: string, options?: any) => {
        if (options?.method === 'PUT') {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({ success: true }),
          })
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      })

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      // Wait for settings to load
      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click Save Settings button
      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      // Verify PUT request was made
      await waitFor(() => {
        const putCalls = (global.fetch as any).mock.calls.filter(
          (call: any[]) => call[1]?.method === 'PUT'
        )
        expect(putCalls.length).toBeGreaterThan(0)
        expect(putCalls[0][0]).toContain('/api/paper-trading/strategy-settings')
      })

      // Verify success toast
      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Trading settings saved successfully!')
      })
    })

    it('should show error toast when save fails', async () => {
      const user = userEvent.setup()

      // Mock GET success but PUT failure
      ;(global.fetch as any).mockImplementation((url: string, options?: any) => {
        if (options?.method === 'PUT') {
          return Promise.resolve({
            ok: false,
            json: () => Promise.resolve({ error: 'Failed to save' }),
          })
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      })

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click Save Settings button
      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      // Verify error toast
      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
      })
    })

    it('should show error toast when save throws exception', async () => {
      const user = userEvent.setup()

      // Mock GET success but PUT throws
      ;(global.fetch as any).mockImplementation((url: string, options?: any) => {
        if (options?.method === 'PUT') {
          return Promise.reject(new Error('Network error'))
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      })

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click Save Settings button
      const saveButton = screen.getByRole('button', { name: /save settings/i })
      await user.click(saveButton)

      // Verify error toast
      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
      })
    })

    it('should reload settings when Reload button is clicked', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Clear previous fetch calls
      vi.clearAllMocks()

      // Click Reload button
      const reloadButton = screen.getByRole('button', { name: /reload/i })
      await user.click(reloadButton)

      // Verify GET request was made again
      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalledWith(
          expect.stringContaining('/api/paper-trading/strategy-settings')
        )
      })
    })

    it('should close dialog when Cancel button is clicked', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click Cancel button
      const cancelButton = screen.getByRole('button', { name: /cancel/i })
      await user.click(cancelButton)

      // Verify dialog is closed (title should not be visible)
      await waitFor(() => {
        expect(screen.queryByText(/trading bot settings/i)).not.toBeInTheDocument()
      })
    })

    it('should apply market preset when preset card is clicked', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click on Market Presets tab
      const presetsTab = screen.getByRole('tab', { name: /market presets/i })
      await user.click(presetsTab)

      // Click on High Volatility preset
      const highVolatilityPreset = screen.getByText('High Volatility')
      await user.click(highVolatilityPreset)

      // Verify success toast
      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Applied High Volatility preset')
      })
    })

    it('should render RSI strategy switch on Strategies tab', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click on Strategies tab
      const strategiesTab = screen.getByRole('tab', { name: /^strategies$/i })
      await user.click(strategiesTab)

      // Verify RSI Strategy card exists
      await waitFor(() => {
        const text = screen.getByText('RSI Strategy')
        expect(text).toBeInTheDocument()
      })

      // Verify at least one switch is rendered (the strategies have switches)
      const switches = screen.getAllByRole('switch')
      expect(switches.length).toBeGreaterThan(0)
    })

    it('should render MACD strategy switch on Strategies tab', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click on Strategies tab
      const strategiesTab = screen.getByRole('tab', { name: /^strategies$/i })
      await user.click(strategiesTab)

      // Verify MACD Strategy card exists
      await waitFor(() => {
        const text = screen.getByText('MACD Strategy')
        expect(text).toBeInTheDocument()
      })
    })

    it('should render Volume strategy switch on Strategies tab', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click on Strategies tab
      const strategiesTab = screen.getByRole('tab', { name: /^strategies$/i })
      await user.click(strategiesTab)

      // Verify Volume Strategy card exists
      await waitFor(() => {
        const text = screen.getByText('Volume Strategy')
        expect(text).toBeInTheDocument()
      })
    })

    it('should render Bollinger Bands strategy switch on Strategies tab', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click on Strategies tab
      const strategiesTab = screen.getByRole('tab', { name: /^strategies$/i })
      await user.click(strategiesTab)

      // Verify Bollinger Bands card exists
      await waitFor(() => {
        const text = screen.getByText('Bollinger Bands')
        expect(text).toBeInTheDocument()
      })
    })

    it('should render Stochastic strategy switch on Strategies tab', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation(() =>
        Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      )

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })

      // Click on Strategies tab
      const strategiesTab = screen.getByRole('tab', { name: /^strategies$/i })
      await user.click(strategiesTab)

      // Verify Stochastic Strategy card exists
      await waitFor(() => {
        const text = screen.getByText('Stochastic Strategy')
        expect(text).toBeInTheDocument()
      })
    })

    it('should handle loadSettings error when dialog opens', async () => {
      const user = userEvent.setup()

      // Mock fetch to fail
      ;(global.fetch as any).mockRejectedValue(new Error('Network error'))

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      // Verify error toast
      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to load trading settings')
      })
    })

    it('should handle loadSettings with non-ok response', async () => {
      const user = userEvent.setup()

      // Mock fetch to return non-ok response
      ;(global.fetch as any).mockResolvedValue({
        ok: false,
        json: () => Promise.resolve({ error: 'Server error' }),
      })

      render(<TradingSettings />)

      // Open dialog
      const openButton = screen.getByRole('button', { name: /trading settings/i })
      await user.click(openButton)

      // Component should still render (just with default settings, no error toast)
      await waitFor(() => {
        expect(screen.getByText(/trading bot settings/i)).toBeInTheDocument()
      })
    })
  })

  describe('InlineTradingSettings', () => {
    it('should render loading state initially', () => {
      ;(global.fetch as any).mockImplementation(
        () =>
          new Promise(() => {
            // Never resolve to keep loading state
          })
      )

      render(<InlineTradingSettings />)

      expect(screen.getByText(/loading settings/i)).toBeInTheDocument()
    })

    it('should load settings on mount', async () => {
      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalledWith(
          expect.stringContaining('/api/paper-trading/strategy-settings')
        )
      })

      // Should render Market Presets section
      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })
    })

    it('should save settings when Save Strategy Settings button is clicked', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation((url: string, options?: any) => {
        if (options?.method === 'PUT') {
          return Promise.resolve({
            ok: true,
            json: () => Promise.resolve({ success: true }),
          })
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Click Save Strategy Settings button
      const saveButton = screen.getByRole('button', { name: /save strategy settings/i })
      await user.click(saveButton)

      // Verify PUT request was made
      await waitFor(() => {
        const putCalls = (global.fetch as any).mock.calls.filter(
          (call: any[]) => call[1]?.method === 'PUT'
        )
        expect(putCalls.length).toBeGreaterThan(0)
      })

      // Verify success toast
      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Trading settings saved successfully!')
      })
    })

    it('should reload settings when Reload button is clicked', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Clear previous fetch calls
      vi.clearAllMocks()

      // Click Reload button
      const reloadButton = screen.getByRole('button', { name: /reload/i })
      await user.click(reloadButton)

      // Verify GET request was made again
      await waitFor(() => {
        expect(global.fetch).toHaveBeenCalledWith(
          expect.stringContaining('/api/paper-trading/strategy-settings')
        )
      })
    })

    it('should apply market preset when preset card is clicked', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Click on Low Volatility preset
      const lowVolatilityPreset = screen.getByText('Low Volatility')
      await user.click(lowVolatilityPreset)

      // Verify success toast
      await waitFor(() => {
        expect(toast.success).toHaveBeenCalledWith('Applied Low Volatility preset')
      })
    })

    it('should render RSI strategy with switches', async () => {
      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Verify RSI Strategy section exists
      await waitFor(() => {
        const text = screen.getByText('RSI Strategy')
        expect(text).toBeInTheDocument()
      })

      // Verify switches are rendered
      const switches = screen.getAllByRole('switch')
      expect(switches.length).toBeGreaterThan(0)
    })

    it('should render MACD strategy with switches', async () => {
      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Verify MACD Strategy section exists
      await waitFor(() => {
        const text = screen.getByText('MACD Strategy')
        expect(text).toBeInTheDocument()
      })
    })

    it('should render Volume strategy with switches', async () => {
      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Verify Volume Strategy section exists
      await waitFor(() => {
        const text = screen.getByText('Volume Strategy')
        expect(text).toBeInTheDocument()
      })
    })

    it('should render Bollinger Bands strategy with switches', async () => {
      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Verify Bollinger Bands section exists
      await waitFor(() => {
        const text = screen.getByText('Bollinger Bands')
        expect(text).toBeInTheDocument()
      })
    })

    it('should render Stochastic strategy with switches', async () => {
      ;(global.fetch as any).mockResolvedValue({
        ok: true,
        json: () => Promise.resolve({ success: true, data: mockSettings }),
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Verify Stochastic section exists
      await waitFor(() => {
        const text = screen.getByText('Stochastic')
        expect(text).toBeInTheDocument()
      })
    })

    it('should handle loadSettings error on mount', async () => {
      // Mock fetch to fail
      ;(global.fetch as any).mockRejectedValue(new Error('Network error'))

      render(<InlineTradingSettings />)

      // Verify error toast
      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to load trading settings')
      })
    })

    it('should show error toast when save fails', async () => {
      const user = userEvent.setup()

      ;(global.fetch as any).mockImplementation((url: string, options?: any) => {
        if (options?.method === 'PUT') {
          return Promise.resolve({
            ok: false,
            json: () => Promise.resolve({ error: 'Failed to save' }),
          })
        }
        return Promise.resolve({
          ok: true,
          json: () => Promise.resolve({ success: true, data: mockSettings }),
        })
      })

      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Market Presets')).toBeInTheDocument()
      })

      // Click Save Strategy Settings button
      const saveButton = screen.getByRole('button', { name: /save strategy settings/i })
      await user.click(saveButton)

      // Verify error toast
      await waitFor(() => {
        expect(toast.error).toHaveBeenCalledWith('Failed to save trading settings')
      })
    })
  })
})
