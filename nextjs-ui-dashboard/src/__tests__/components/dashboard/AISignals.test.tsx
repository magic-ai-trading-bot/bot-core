import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { AISignals } from '../../../components/dashboard/AISignals'

// Mock hooks and contexts
const mockUseAIAnalysisContext = vi.fn()
const mockUseWebSocket = vi.fn()

vi.mock('../../../contexts/AIAnalysisContext', () => ({
  useAIAnalysisContext: () => mockUseAIAnalysisContext(),
  AIAnalysisProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

vi.mock('../../../hooks/useWebSocket', () => ({
  useWebSocket: () => mockUseWebSocket(),
}))

vi.mock('../../../contexts/WebSocketContext', () => ({
  useWebSocketContext: () => mockUseWebSocket(),
  WebSocketProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

describe('AISignals', () => {
  const mockSignal = {
    signal: 'long',
    confidence: 0.85,
    timestamp: new Date('2024-01-01T12:00:00Z').toISOString(),
    symbol: 'BTCUSDT',
    reasoning: 'Strong bullish momentum with RSI oversold',
    strategy_scores: {
      'RSI Strategy': 0.9,
      'MACD Strategy': 0.8,
      'Volume Strategy': 0.75,
      'Bollinger Bands Strategy': 0.85,
    },
    market_analysis: {
      trend_direction: 'Bullish',
      trend_strength: 0.85,
      support_levels: [45000, 44000],
      resistance_levels: [46000, 47000],
      volatility_level: 'Medium',
      volume_analysis: 'High volume on uptrend',
    },
    risk_assessment: {
      overall_risk: 'Medium',
      technical_risk: 0.4,
      market_risk: 0.5,
      recommended_position_size: 0.02,
      stop_loss_suggestion: 44500,
      take_profit_suggestion: 46500,
    },
    source: 'api',
  }

  beforeEach(() => {
    vi.clearAllMocks()

    // Default mock implementations
    mockUseAIAnalysisContext.mockReturnValue({
      state: {
        signals: [],
        isLoading: false,
        error: null,
        serviceInfo: {
          service_name: 'AI Trading Service',
          version: '1.0.0',
          model_version: 'v2.3',
        },
        lastUpdate: new Date('2024-01-01T12:00:00Z').toISOString(),
      },
      analyzeSymbol: vi.fn(),
      clearError: vi.fn(),
    })

    mockUseWebSocket.mockReturnValue({
      state: {
        isConnected: true,
        aiSignals: [],
        lastMessage: null,
        error: null,
      },
    })
  })

  describe('Component Rendering', () => {
    it('renders AI signals card', () => {
      render(<AISignals />)

      expect(screen.getByText('AI Trading Signals')).toBeInTheDocument()
    })

    it('displays live analysis badge', () => {
      render(<AISignals />)

      expect(screen.getByText('Live Analysis')).toBeInTheDocument()
    })

    it('shows WebSocket connection status', () => {
      render(<AISignals />)

      expect(screen.getByText(/WebSocket:/)).toBeInTheDocument()
      expect(screen.getByText(/Connected/)).toBeInTheDocument()
    })

    it('shows disconnected status when WebSocket is disconnected', () => {
      mockUseWebSocket.mockReturnValue({
        state: {
          isConnected: false,
          aiSignals: [],
          lastMessage: null,
          error: null,
        },
      })

      render(<AISignals />)

      expect(screen.getByText(/Disconnected/)).toBeInTheDocument()
    })
  })

  describe('Service Information', () => {
    it('displays AI service info', () => {
      render(<AISignals />)

      expect(screen.getByText(/AI Trading Service v1.0.0/)).toBeInTheDocument()
      expect(screen.getByText(/Model: v2.3/)).toBeInTheDocument()
    })

    it('displays last update time', () => {
      render(<AISignals />)

      expect(screen.getByText(/Last updated:/)).toBeInTheDocument()
    })

    it('does not show service info when not available', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.queryByText(/AI Trading Service/)).not.toBeInTheDocument()
    })
  })

  describe('Loading State', () => {
    it('displays loading indicator when analyzing', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [],
          isLoading: true,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('Analyzing market signals...')).toBeInTheDocument()
    })
  })

  describe('Error Handling', () => {
    it('displays error message when AI analysis fails', () => {
      const clearError = vi.fn()
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [],
          isLoading: false,
          error: 'Failed to fetch AI signals',
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError,
      })

      render(<AISignals />)

      expect(screen.getByText('Failed to fetch AI signals')).toBeInTheDocument()
    })

    it('displays error message when WebSocket fails', () => {
      mockUseWebSocket.mockReturnValue({
        state: {
          isConnected: false,
          aiSignals: [],
          lastMessage: null,
          error: 'WebSocket connection failed',
        },
      })

      render(<AISignals />)

      expect(screen.getByText('WebSocket connection failed')).toBeInTheDocument()
    })

    it('clears error when dismiss button is clicked', async () => {
      const user = userEvent.setup()
      const clearError = vi.fn()

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [],
          isLoading: false,
          error: 'Test error',
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError,
      })

      render(<AISignals />)

      const dismissButton = screen.getByRole('button', { name: /dismiss/i })
      await user.click(dismissButton)

      expect(clearError).toHaveBeenCalled()
    })
  })

  describe('Empty State', () => {
    it('displays message when no signals available', () => {
      render(<AISignals />)

      expect(screen.getByText('No AI signals available yet')).toBeInTheDocument()
      expect(screen.getByText('Analysis will start automatically')).toBeInTheDocument()
    })
  })

  describe('Signal Display', () => {
    it('displays signal from API', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('LONG')).toBeInTheDocument()
      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      expect(screen.getByText('85%')).toBeInTheDocument()
    })

    it('displays signal from WebSocket', () => {
      const wsSignal = {
        signal: 'short',
        confidence: 0.75,
        timestamp: Date.now(),
        symbol: 'ETHUSDT',
        model_type: 'transformer',
      }

      mockUseWebSocket.mockReturnValue({
        state: {
          isConnected: true,
          aiSignals: [wsSignal],
          lastMessage: null,
          error: null,
        },
      })

      render(<AISignals />)

      expect(screen.getByText('SHORT')).toBeInTheDocument()
      expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
      expect(screen.getByText('75%')).toBeInTheDocument()
    })

    it('marks active signals with ACTIVE badge', () => {
      const recentSignal = {
        ...mockSignal,
        timestamp: new Date(Date.now() - 5 * 60 * 1000).toISOString(), // 5 minutes ago
      }

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [recentSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('ACTIVE')).toBeInTheDocument()
    })

    it('does not mark expired signals as active', () => {
      const oldSignal = {
        ...mockSignal,
        timestamp: new Date(Date.now() - 60 * 60 * 1000).toISOString(), // 1 hour ago
      }

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [oldSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.queryByText('ACTIVE')).not.toBeInTheDocument()
    })

    it('displays confidence bar with correct color for high confidence', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      const { container } = render(<AISignals />)

      // High confidence (>= 0.8) should have profit color
      const confidenceBars = container.querySelectorAll('.bg-profit')
      expect(confidenceBars.length).toBeGreaterThan(0)
    })

    it('displays signal reasoning', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('Strong bullish momentum with RSI oversold')).toBeInTheDocument()
    })
  })

  describe('Signal Details Dialog', () => {
    it('opens detailed dialog when signal is clicked', async () => {
      const user = userEvent.setup()

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      const signalCard = screen.getByText('BTC/USDT').closest('div')
      if (signalCard) {
        await user.click(signalCard)
      }

      await waitFor(() => {
        expect(screen.getByText(/Detailed AI Analysis:/)).toBeInTheDocument()
      })
    })

    it('displays market analysis in dialog', async () => {
      const user = userEvent.setup()

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      const signalCard = screen.getByText('BTC/USDT').closest('div')
      if (signalCard) {
        await user.click(signalCard)
      }

      await waitFor(() => {
        expect(screen.getByText('Market Analysis')).toBeInTheDocument()
        expect(screen.getByText('Bullish')).toBeInTheDocument()
        // Use getAllByText since "Medium" appears multiple times (risk level and volatility)
        expect(screen.getAllByText('Medium').length).toBeGreaterThan(0)
      })
    })

    it('displays strategy scores in dialog', async () => {
      const user = userEvent.setup()

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      const signalCard = screen.getByText('BTC/USDT').closest('div')
      if (signalCard) {
        await user.click(signalCard)
      }

      await waitFor(() => {
        expect(screen.getByText('Strategy Analysis')).toBeInTheDocument()
        expect(screen.getByText('RSI Strategy')).toBeInTheDocument()
        expect(screen.getByText('90.0%')).toBeInTheDocument()
      })
    })

    it('displays risk assessment in dialog', async () => {
      const user = userEvent.setup()

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      const signalCard = screen.getByText('BTC/USDT').closest('div')
      if (signalCard) {
        await user.click(signalCard)
      }

      await waitFor(() => {
        expect(screen.getByText('Risk Assessment')).toBeInTheDocument()
        expect(screen.getByText(/Technical Risk:/)).toBeInTheDocument()
        expect(screen.getByText(/Market Risk:/)).toBeInTheDocument()
      })
    })

    it('displays stop loss and take profit suggestions', async () => {
      const user = userEvent.setup()

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      const signalCard = screen.getByText('BTC/USDT').closest('div')
      if (signalCard) {
        await user.click(signalCard)
      }

      await waitFor(() => {
        expect(screen.getByText(/Stop Loss:/)).toBeInTheDocument()
        expect(screen.getByText(/Take Profit:/)).toBeInTheDocument()
        // Check for values using regex to be flexible with formatting
        expect(screen.getByText(/44,?500/)).toBeInTheDocument()
        expect(screen.getByText(/46,?500/)).toBeInTheDocument()
      })
    })

    it('opens strategy explanation dialog when strategy is clicked', async () => {
      const user = userEvent.setup()

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      // Open signal details
      const signalCard = screen.getByText('BTC/USDT').closest('div')
      if (signalCard) {
        await user.click(signalCard)
      }

      await waitFor(() => {
        expect(screen.getByText('Strategy Analysis')).toBeInTheDocument()
      })

      // Click on RSI Strategy
      const rsiStrategy = screen.getByText('RSI Strategy').closest('div')
      if (rsiStrategy) {
        await user.click(rsiStrategy)
      }

      await waitFor(() => {
        expect(screen.getByText(/Giải thích Strategy:/)).toBeInTheDocument()
      })
    })
  })

  describe('Multiple Signals', () => {
    it('displays multiple signals correctly', () => {
      const signals = [
        { ...mockSignal, symbol: 'BTCUSDT' },
        { ...mockSignal, symbol: 'ETHUSDT', signal: 'short', confidence: 0.7 },
        { ...mockSignal, symbol: 'BNBUSDT', signal: 'neutral', confidence: 0.5 },
      ]

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals,
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
      expect(screen.getByText('ETH/USDT')).toBeInTheDocument()
      expect(screen.getByText('BNB/USDT')).toBeInTheDocument()
    })

    it('shows only most recent signal per symbol', () => {
      const signals = [
        { ...mockSignal, symbol: 'BTCUSDT', timestamp: new Date('2024-01-01T12:00:00Z').toISOString() },
        { ...mockSignal, symbol: 'BTCUSDT', timestamp: new Date('2024-01-01T11:00:00Z').toISOString() },
      ]

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals,
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      // Should only show one BTC/USDT signal (the most recent)
      const btcSignals = screen.getAllByText('BTC/USDT')
      expect(btcSignals).toHaveLength(1)
    })
  })

  describe('Signal Colors', () => {
    it('displays LONG signal with profit color', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [{ ...mockSignal, signal: 'long' }],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      const { container } = render(<AISignals />)

      const longBadge = screen.getByText('LONG')
      expect(longBadge.className).toContain('bg-profit')
    })

    it('displays SHORT signal with loss color', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [{ ...mockSignal, signal: 'short' }],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      const shortBadge = screen.getByText('SHORT')
      expect(shortBadge.className).toContain('bg-loss')
    })

    it('displays NEUTRAL signal with warning color', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [{ ...mockSignal, signal: 'neutral' }],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      const neutralBadge = screen.getByText('NEUTRAL')
      expect(neutralBadge.className).toContain('bg-warning')
    })
  })

  describe('Timestamp Formatting', () => {
    it('formats timestamp correctly', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      // Should display a formatted timestamp
      const timestampRegex = /\d{1,2}\/\d{1,2}\/\d{4}/
      const timestamps = screen.getAllByText(timestampRegex)
      expect(timestamps.length).toBeGreaterThan(0)
    })
  })

  describe('Accessibility', () => {
    it('has clickable signal cards', () => {
      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [mockSignal],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      const { container } = render(<AISignals />)

      const clickableCards = container.querySelectorAll('[class*="cursor-pointer"]')
      expect(clickableCards.length).toBeGreaterThan(0)
    })
  })

  describe('Edge Cases', () => {
    it('handles signals without strategy scores', () => {
      const signalWithoutScores = {
        ...mockSignal,
        strategy_scores: undefined,
      }

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [signalWithoutScores],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
    })

    it('handles signals without market analysis', () => {
      const signalWithoutAnalysis = {
        ...mockSignal,
        market_analysis: undefined,
      }

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [signalWithoutAnalysis],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
    })

    it('handles signals without risk assessment', () => {
      const signalWithoutRisk = {
        ...mockSignal,
        risk_assessment: undefined,
      }

      mockUseAIAnalysisContext.mockReturnValue({
        state: {
          signals: [signalWithoutRisk],
          isLoading: false,
          error: null,
          serviceInfo: null,
          lastUpdate: null,
        },
        analyzeSymbol: vi.fn(),
        clearError: vi.fn(),
      })

      render(<AISignals />)

      expect(screen.getByText('BTC/USDT')).toBeInTheDocument()
    })
  })
})
