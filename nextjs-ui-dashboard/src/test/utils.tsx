import React, { ReactElement } from 'react'
import { render, RenderOptions } from '@testing-library/react'
import { BrowserRouter } from 'react-router-dom'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { I18nextProvider } from 'react-i18next'
import i18n from '../i18n/config'
import { AuthProvider } from '../contexts/AuthContext'
import { WebSocketProvider } from '../contexts/WebSocketContext'
import { ThemeProvider } from '../contexts/ThemeContext'

// Initialize i18n for tests with Vietnamese as default (to match existing test assertions)
i18n.changeLanguage('vi')

// NOTE: AIAnalysisProvider NOT included by default - tests that need it should mock useAIAnalysis hook
// This prevents conflicts with test-specific API mocks

// Create a test query client
const createTestQueryClient = () =>
  new QueryClient({
    defaultOptions: {
      queries: {
        retry: false,
      },
      mutations: {
        retry: false,
      },
    },
  })

// Custom render with providers
interface CustomRenderOptions extends Omit<RenderOptions, 'wrapper'> {
  initialEntries?: string[]
  queryClient?: QueryClient
}

const AllTheProviders: React.FC<{ children: React.ReactNode; queryClient: QueryClient }> = ({
  children,
  queryClient
}) => {
  return (
    <BrowserRouter>
      <QueryClientProvider client={queryClient}>
        <I18nextProvider i18n={i18n}>
          <ThemeProvider>
            <AuthProvider>
              <WebSocketProvider>
                {children}
              </WebSocketProvider>
            </AuthProvider>
          </ThemeProvider>
        </I18nextProvider>
      </QueryClientProvider>
    </BrowserRouter>
  )
}

const customRender = (
  ui: ReactElement,
  options: CustomRenderOptions = {}
) => {
  const { queryClient = createTestQueryClient(), ...renderOptions } = options

  return render(ui, {
    wrapper: ({ children }) => (
      <AllTheProviders queryClient={queryClient}>
        {children}
      </AllTheProviders>
    ),
    ...renderOptions,
  })
}

// Mock data generators
export const mockUser = {
  id: 'user123',
  email: 'test@example.com',
  full_name: 'Test User',
  roles: ['user'],
  created_at: '2024-01-01T00:00:00Z',
}

export const mockPosition = {
  symbol: 'BTCUSDT',
  side: 'LONG' as const,
  quantity: 0.1,
  entry_price: 45000,
  current_price: 45500,
  unrealized_pnl: 50,
  percentage: 1.11,
}

export const mockTrade = {
  id: 'trade1',
  symbol: 'BTCUSDT',
  side: 'BUY' as const,
  quantity: 0.1,
  price: 45000,
  timestamp: '2024-01-01T00:00:00Z',
  status: 'executed' as const,
  pnl: 50,
}

export const mockCandle = {
  open: 45000,
  high: 45500,
  low: 44800,
  close: 45200,
  volume: 1000,
  timestamp: Date.now(),
}

export const mockStrategy = {
  id: 'rsi_strategy',
  name: 'RSI Strategy',
  type: 'RSI' as const,
  enabled: true,
  parameters: {
    period: 14,
    overbought: 70,
    oversold: 30,
  },
  performance: {
    total_trades: 25,
    win_rate: 0.68,
    total_pnl: 350,
  },
}

// Helper functions
export const waitForLoadingToFinish = () =>
  new Promise((resolve) => setTimeout(resolve, 0))

export const mockLocalStorage = () => {
  const storage: Record<string, string> = {}
  
  return {
    getItem: (key: string) => storage[key] || null,
    setItem: (key: string, value: string) => {
      storage[key] = value
    },
    removeItem: (key: string) => {
      delete storage[key]
    },
    clear: () => {
      Object.keys(storage).forEach(key => delete storage[key])
    },
  }
}

// Mock WebSocket
export const mockWebSocket = () => {
  const listeners: Record<string, ((data?: unknown) => void)[]> = {}
  
  return {
    send: vi.fn(),
    close: vi.fn(),
    addEventListener: vi.fn((event: string, handler: (data?: unknown) => void) => {
      if (!listeners[event]) listeners[event] = []
      listeners[event].push(handler)
    }),
    removeEventListener: vi.fn(),
    readyState: 1,
    trigger: (event: string, data?: unknown) => {
      if (listeners[event]) {
        listeners[event].forEach(handler => handler(data))
      }
    },
  }
}

// Re-export everything
export * from '@testing-library/react'
export { customRender as render }