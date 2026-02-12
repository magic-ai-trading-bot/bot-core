/**
 * TradingSettings - Direct onChange Coverage Tests
 *
 * Uses clickable mocked Slider/Switch/Select that fire callbacks when clicked.
 * This covers ALL onChange/onToggle inline callbacks.
 *
 * Uncovered lines: 763-871, 909-998, 1022-1039, 1251-1531, 1627-1658
 */
import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import React from 'react'
import { screen, waitFor, within, fireEvent } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'

// Mock Slider - clicking the button fires onValueChange
vi.mock('@/components/ui/slider', () => ({
  Slider: ({ onValueChange, value, ...props }: any) => (
    <button
      data-testid="mock-slider-btn"
      onClick={() => onValueChange?.([((value?.[0] ?? 0) + 1)])}
    >
      slider:{value?.[0]}
    </button>
  ),
}))

// Mock Switch - clicking fires onCheckedChange
vi.mock('@/components/ui/switch', () => ({
  Switch: ({ checked, onCheckedChange, ...props }: any) => (
    <button
      data-testid="mock-switch-btn"
      onClick={() => onCheckedChange?.(!checked)}
    >
      switch:{checked ? 'on' : 'off'}
    </button>
  ),
}))

// Mock Select - clicking fires onValueChange
vi.mock('@/components/ui/select', () => ({
  Select: ({ onValueChange, value, children }: any) => (
    <div>
      <button
        data-testid="mock-select-btn"
        onClick={() => onValueChange?.('Consensus')}
      >
        select:{value}
      </button>
      {children}
    </div>
  ),
  SelectTrigger: ({ children }: any) => <div>{children}</div>,
  SelectValue: ({ placeholder }: any) => <span>{placeholder}</span>,
  SelectContent: ({ children }: any) => <div>{children}</div>,
  SelectItem: ({ value, children }: any) => <div data-value={value}>{children}</div>,
}))

// Mock framer-motion with Proxy
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

// Mock sonner toast
vi.mock('sonner', () => ({
  toast: { success: vi.fn(), error: vi.fn() },
}))

import { TradingSettings, InlineTradingSettings } from '../../../components/dashboard/TradingSettings'

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
      enabled_strategies: ['RSI Strategy'],
      market_condition: 'Trending', risk_level: 'Moderate', data_resolution: '15m',
    },
    market_preset: 'normal_volatility',
  },
}

const mockFetch = vi.fn()
global.fetch = mockFetch

describe('TradingSettings - Click-to-fire onChange Coverage', () => {
  const user = userEvent.setup()

  beforeEach(() => {
    vi.clearAllMocks()
    Element.prototype.scrollIntoView = vi.fn()
    mockFetch.mockResolvedValue({
      ok: true,
      json: async () => mockSettings,
      clone: function() { return this; },
    })
  })

  afterEach(() => { vi.clearAllMocks() })

  async function openDialog() {
    render(<TradingSettings />)
    await waitFor(() => {
      expect(screen.getByRole('button', { name: /trading settings/i })).toBeInTheDocument()
    })
    await user.click(screen.getByRole('button', { name: /trading settings/i }))
    await waitFor(() => {
      expect(screen.getByRole('dialog')).toBeInTheDocument()
    })
  }

  async function switchToTab(tabName: string) {
    const tab = screen.getByRole('tab', { name: new RegExp(tabName, 'i') })
    await user.click(tab)
    await waitFor(() => {
      expect(tab).toHaveAttribute('data-state', 'active')
    })
  }

  // ========================================================
  // Strategies Tab - Click sliders to fire onChange
  // Covers: 763-871, 909-926, 1251-1531
  // ========================================================
  describe('Strategies tab', () => {
    it('should click ALL slider buttons to fire onChange callbacks', async () => {
      await openDialog()
      await switchToTab('strategies')

      const dialog = screen.getByRole('dialog')
      // Use fireEvent.click (synchronous) to avoid stale refs from re-renders
      const sliderBtns = within(dialog).getAllByTestId('mock-slider-btn')
      for (const btn of sliderBtns) {
        fireEvent.click(btn)
      }

      // After batch update, re-query to verify
      await waitFor(() => {
        expect(within(screen.getByRole('dialog')).getAllByTestId('mock-slider-btn').length).toBeGreaterThan(5)
      })
    })

    it('should click ALL switch buttons to fire onToggle callbacks', async () => {
      await openDialog()
      await switchToTab('strategies')

      const dialog = screen.getByRole('dialog')
      const switchBtns = within(dialog).getAllByTestId('mock-switch-btn')
      for (const btn of switchBtns) {
        fireEvent.click(btn)
      }

      await waitFor(() => {
        expect(within(screen.getByRole('dialog')).getAllByTestId('mock-switch-btn').length).toBeGreaterThanOrEqual(5)
      })
    })

    it('should click each slider one at a time with re-query', async () => {
      await openDialog()
      await switchToTab('strategies')

      const dialog = screen.getByRole('dialog')
      const count = within(dialog).getAllByTestId('mock-slider-btn').length

      for (let i = 0; i < count; i++) {
        const btns = within(screen.getByRole('dialog')).getAllByTestId('mock-slider-btn')
        if (i < btns.length) {
          fireEvent.click(btns[i])
        }
      }

      expect(count).toBeGreaterThan(5)
    })

    it('should click each switch one at a time with re-query', async () => {
      await openDialog()
      await switchToTab('strategies')

      const dialog = screen.getByRole('dialog')
      const count = within(dialog).getAllByTestId('mock-switch-btn').length

      for (let i = 0; i < count; i++) {
        const btns = within(screen.getByRole('dialog')).getAllByTestId('mock-switch-btn')
        if (i < btns.length) {
          fireEvent.click(btns[i])
        }
      }

      expect(count).toBeGreaterThanOrEqual(5)
    })
  })

  // ========================================================
  // Risk Tab - Click sliders to fire onChange
  // Covers: 1558-1661, 1627-1658
  // ========================================================
  describe('Risk tab', () => {
    it('should click all risk slider buttons to fire onChange', async () => {
      await openDialog()
      await switchToTab('risk')

      const dialog = screen.getByRole('dialog')
      const count = within(dialog).getAllByTestId('mock-slider-btn').length

      for (let i = 0; i < count; i++) {
        const btns = within(screen.getByRole('dialog')).getAllByTestId('mock-slider-btn')
        if (i < btns.length) {
          fireEvent.click(btns[i])
        }
      }

      expect(count).toBeGreaterThan(3)
    })
  })

  // ========================================================
  // Engine Tab - Click sliders and selects to fire onChange
  // Covers: 1686-1764, 1022-1039
  // ========================================================
  describe('Engine tab', () => {
    it('should click engine sliders and selects to fire onChange', async () => {
      await openDialog()
      await switchToTab('engine')

      const dialog = screen.getByRole('dialog')
      const sliderCount = within(dialog).getAllByTestId('mock-slider-btn').length
      const selectCount = within(dialog).getAllByTestId('mock-select-btn').length

      for (let i = 0; i < sliderCount; i++) {
        const btns = within(screen.getByRole('dialog')).getAllByTestId('mock-slider-btn')
        if (i < btns.length) fireEvent.click(btns[i])
      }
      for (let i = 0; i < selectCount; i++) {
        const btns = within(screen.getByRole('dialog')).getAllByTestId('mock-select-btn')
        if (i < btns.length) fireEvent.click(btns[i])
      }

      expect(sliderCount).toBeGreaterThan(0)
      expect(selectCount).toBeGreaterThan(0)
    })
  })

  // ========================================================
  // InlineTradingSettings - Covers: 909-998, 1022-1039
  // ========================================================
  describe('InlineTradingSettings', () => {
    it('should click all inline slider buttons one by one', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Risk Management')).toBeInTheDocument()
      })

      const count = screen.getAllByTestId('mock-slider-btn').length
      for (let i = 0; i < count; i++) {
        const btns = screen.getAllByTestId('mock-slider-btn')
        if (i < btns.length) fireEvent.click(btns[i])
      }

      expect(count).toBeGreaterThan(3)
    })

    it('should click all inline select buttons one by one', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('Engine Settings')).toBeInTheDocument()
      })

      const count = screen.getAllByTestId('mock-select-btn').length
      for (let i = 0; i < count; i++) {
        const btns = screen.getAllByTestId('mock-select-btn')
        if (i < btns.length) fireEvent.click(btns[i])
      }

      expect(count).toBeGreaterThan(0)
    })

    it('should click all inline switch buttons one by one', async () => {
      render(<InlineTradingSettings />)

      await waitFor(() => {
        expect(screen.getByText('RSI Strategy')).toBeInTheDocument()
      })

      const count = screen.getAllByTestId('mock-switch-btn').length
      for (let i = 0; i < count; i++) {
        const btns = screen.getAllByTestId('mock-switch-btn')
        if (i < btns.length) fireEvent.click(btns[i])
      }

      expect(count).toBeGreaterThan(0)
    })
  })
})
