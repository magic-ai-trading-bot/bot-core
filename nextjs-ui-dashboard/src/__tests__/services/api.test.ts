import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import type {
  BotStatus,
  Position,
  TradeHistory,
  AISignal,
  AISignalResponse,
  PerformanceStats,
  AccountInfo,
  TradingConfig,
  CandleData,
  ChartData,
  SupportedSymbols,
  MarketOverview,
  AIModelInfo,
  LoginRequest,
  RegisterRequest,
  LoginResponse,
  UserProfile,
  StrategyRecommendation,
  MarketConditionAnalysis,
  AIServiceInfo,
} from '../../services/api'

// Mock localStorage for this test file
const localStorageMock = (() => {
  let store: Record<string, string> = {}
  return {
    getItem: (key: string) => store[key] || null,
    setItem: (key: string, value: string) => { store[key] = value },
    removeItem: (key: string) => { delete store[key] },
    clear: () => { store = {} },
    get length() { return Object.keys(store).length },
    key: (index: number) => Object.keys(store)[index] || null,
  }
})()

Object.defineProperty(global, 'localStorage', {
  value: localStorageMock,
  writable: true,
})

// Mock axios before importing the service
vi.mock('axios', () => {
  const mockInstance = {
    get: vi.fn(),
    post: vi.fn(),
    put: vi.fn(),
    delete: vi.fn(),
    interceptors: {
      request: {
        use: vi.fn(),
      },
      response: {
        use: vi.fn(),
      },
    },
  }

  return {
    default: {
      create: vi.fn(() => mockInstance),
    },
  }
})

// Import after mocking
import axios from 'axios'
import { BotCoreApiClient, apiClient, rustApi, pythonAI } from '../../services/api'

const mockAxiosInstance = (axios.create as any)()

describe('API Service Tests', () => {
  beforeEach(async () => {
    // Clear localStorage safely
    try {
      if (typeof localStorage !== 'undefined' && typeof localStorage.clear === 'function') {
        localStorage.clear()
      }
    } catch (error) {
      // Handle SecurityError in test environments
    }

    // Reset all mocks
    vi.clearAllMocks()

    // Reset mock implementations
    mockAxiosInstance.get.mockReset()
    mockAxiosInstance.post.mockReset()
    mockAxiosInstance.put.mockReset()
    mockAxiosInstance.delete.mockReset()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('BaseApiClient', () => {
    it('should create axios instance', () => {
      const client = new BotCoreApiClient()

      expect(client).toBeDefined()
      expect(client.rust).toBeDefined()
      expect(client.python).toBeDefined()
      expect(client.auth).toBeDefined()
    })

    it('should set up request and response interceptors', () => {
      const client = new BotCoreApiClient()

      // Verify interceptors were set up
      expect(mockAxiosInstance.interceptors.request.use).toHaveBeenCalled()
      expect(mockAxiosInstance.interceptors.response.use).toHaveBeenCalled()
    })
  })

  describe('RustTradingApiClient - Bot Status and Control', () => {
    it('should get bot status', async () => {
      const mockStatus: BotStatus = {
        status: 'running',
        uptime: 3600,
        active_positions: 2,
        total_trades: 50,
        total_pnl: 1234.56,
        last_update: '2024-01-01T00:00:00Z',
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockStatus })

      const client = new BotCoreApiClient()
      const result = await client.rust.getBotStatus()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/status')
      expect(result).toEqual(mockStatus)
    })

    it('should retry on failure and succeed', async () => {
      const mockStatus: BotStatus = {
        status: 'running',
        uptime: 3600,
        active_positions: 2,
        total_trades: 50,
        total_pnl: 1234.56,
        last_update: '2024-01-01T00:00:00Z',
      }

      mockAxiosInstance.get
        .mockRejectedValueOnce(new Error('Network error'))
        .mockResolvedValueOnce({ data: mockStatus })

      const client = new BotCoreApiClient()
      const result = await client.rust.getBotStatus()

      expect(mockAxiosInstance.get).toHaveBeenCalledTimes(2)
      expect(result).toEqual(mockStatus)
    })

    it('should fail after max retries', async () => {
      mockAxiosInstance.get.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toThrow('Network error')
      expect(mockAxiosInstance.get).toHaveBeenCalledTimes(3)
    })

    it('should start bot', async () => {
      const mockResponse = { message: 'Bot started successfully' }
      mockAxiosInstance.post.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.rust.startBot()

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/bot/start')
      expect(result).toEqual(mockResponse)
    })

    it('should stop bot', async () => {
      const mockResponse = { message: 'Bot stopped successfully' }
      mockAxiosInstance.post.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.rust.stopBot()

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/bot/stop')
      expect(result).toEqual(mockResponse)
    })
  })

  describe('RustTradingApiClient - Position Management', () => {
    it('should get positions', async () => {
      const mockPositions: Position[] = [
        {
          symbol: 'BTCUSDT',
          side: 'BUY',
          size: 1.5,
          entry_price: 50000,
          current_price: 51000,
          unrealized_pnl: 1500,
          timestamp: '2024-01-01T00:00:00Z',
        },
      ]

      mockAxiosInstance.get.mockResolvedValue({ data: mockPositions })

      const client = new BotCoreApiClient()
      const result = await client.rust.getPositions()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/positions')
      expect(result).toEqual(mockPositions)
    })

    it('should close position', async () => {
      const mockResponse = {
        message: 'Position closed',
        exit_price: 51000,
        pnl: 1500,
      }

      mockAxiosInstance.post.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.rust.closePosition('BTCUSDT')

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/positions/BTCUSDT/close')
      expect(result).toEqual(mockResponse)
    })

    it('should close all positions', async () => {
      const mockResponse = {
        message: 'All positions closed',
        closed_positions: 3,
      }

      mockAxiosInstance.post.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.rust.closeAllPositions()

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/positions/close-all')
      expect(result).toEqual(mockResponse)
    })
  })

  describe('RustTradingApiClient - Trading History', () => {
    it('should get trade history with default limit', async () => {
      const mockHistory: TradeHistory[] = [
        {
          id: '1',
          symbol: 'BTCUSDT',
          side: 'BUY',
          quantity: 1.5,
          entry_price: 50000,
          exit_price: 51000,
          pnl: 1500,
          entry_time: '2024-01-01T00:00:00Z',
          exit_time: '2024-01-01T01:00:00Z',
          status: 'closed',
        },
      ]

      mockAxiosInstance.get.mockResolvedValue({ data: mockHistory })

      const client = new BotCoreApiClient()
      const result = await client.rust.getTradeHistory()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/trades/history?limit=100')
      expect(result).toEqual(mockHistory)
    })

    it('should get trade history with custom limit', async () => {
      const mockHistory: TradeHistory[] = []
      mockAxiosInstance.get.mockResolvedValue({ data: mockHistory })

      const client = new BotCoreApiClient()
      await client.rust.getTradeHistory(50)

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/trades/history?limit=50')
    })
  })

  describe('RustTradingApiClient - Performance and Account', () => {
    it('should get performance stats', async () => {
      const mockStats: PerformanceStats = {
        total_pnl: 5000,
        win_rate: 0.75,
        total_trades: 100,
        avg_trade_duration: 3600,
        max_drawdown: 500,
        sharpe_ratio: 2.5,
        best_trade: 1000,
        worst_trade: -200,
        profit_factor: 3.0,
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockStats })

      const client = new BotCoreApiClient()
      const result = await client.rust.getPerformanceStats()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/performance/stats')
      expect(result).toEqual(mockStats)
    })

    it('should get account info', async () => {
      const mockAccount: AccountInfo = {
        account_id: 'acc123',
        balance: 10000,
        available_balance: 8000,
        currency: 'USDT',
        created_at: '2024-01-01T00:00:00Z',
        updated_at: '2024-01-01T00:00:00Z',
        account_type: 'standard',
        status: 'active',
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockAccount })

      const client = new BotCoreApiClient()
      const result = await client.rust.getAccountInfo()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/account')
      expect(result).toEqual(mockAccount)
    })
  })

  describe('RustTradingApiClient - Configuration', () => {
    it('should update trading config', async () => {
      const mockResponse = { message: 'Config updated' }
      const config = {
        enabled: true,
        max_positions: 5,
        risk_percentage: 2,
      }

      mockAxiosInstance.put.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.rust.updateTradingConfig(config)

      expect(mockAxiosInstance.put).toHaveBeenCalledWith('/api/config/trading', config)
      expect(result).toEqual(mockResponse)
    })

    it('should get trading config', async () => {
      const mockConfig: TradingConfig = {
        enabled: true,
        max_positions: 5,
        risk_percentage: 2,
        stop_loss_percentage: 3,
        take_profit_percentage: 6,
        max_daily_trades: 10,
        trading_hours: {
          start: '00:00',
          end: '23:59',
        },
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockConfig })

      const client = new BotCoreApiClient()
      const result = await client.rust.getTradingConfig()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/config/trading')
      expect(result).toEqual(mockConfig)
    })
  })

  describe('RustTradingApiClient - Market Data', () => {
    it('should get market data with default limit', async () => {
      const mockCandles: CandleData[] = [
        {
          timestamp: 1234567890,
          open: 50000,
          high: 51000,
          low: 49000,
          close: 50500,
          volume: 1000,
        },
      ]

      mockAxiosInstance.get.mockResolvedValue({ data: mockCandles })

      const client = new BotCoreApiClient()
      const result = await client.rust.getMarketData('BTCUSDT', '1h')

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market-data/BTCUSDT/1h?limit=100')
      expect(result).toEqual(mockCandles)
    })

    it('should get market data with custom limit', async () => {
      const mockCandles: CandleData[] = []
      mockAxiosInstance.get.mockResolvedValue({ data: mockCandles })

      const client = new BotCoreApiClient()
      await client.rust.getMarketData('BTCUSDT', '1h', 50)

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market-data/BTCUSDT/1h?limit=50')
    })

    it('should get chart data', async () => {
      const mockChartData: ChartData = {
        symbol: 'BTCUSDT',
        timeframe: '1h',
        candles: [],
        latest_price: 50000,
        volume_24h: 1000000,
        price_change_24h: 500,
        price_change_percent_24h: 1.0,
      }

      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockChartData } })

      const client = new BotCoreApiClient()
      const result = await client.rust.getChartData('BTCUSDT', '1h')

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market/chart/BTCUSDT/1h')
      expect(result).toEqual(mockChartData)
    })

    it('should get chart data with limit', async () => {
      const mockChartData: ChartData = {
        symbol: 'BTCUSDT',
        timeframe: '1h',
        candles: [],
        latest_price: 50000,
        volume_24h: 1000000,
        price_change_24h: 500,
        price_change_percent_24h: 1.0,
      }

      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockChartData } })

      const client = new BotCoreApiClient()
      await client.rust.getChartData('BTCUSDT', '1h', 200)

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market/chart/BTCUSDT/1h?limit=200')
    })

    it('should get multi chart data', async () => {
      const mockChartData: ChartData[] = []
      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockChartData } })

      const client = new BotCoreApiClient()
      const result = await client.rust.getMultiChartData(['BTCUSDT', 'ETHUSDT'], ['1h', '4h'])

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market/charts?symbols=BTCUSDT,ETHUSDT&timeframes=1h,4h')
      expect(result).toEqual(mockChartData)
    })

    it('should get multi chart data with limit', async () => {
      const mockChartData: ChartData[] = []
      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockChartData } })

      const client = new BotCoreApiClient()
      await client.rust.getMultiChartData(['BTCUSDT'], ['1h'], 50)

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market/charts?symbols=BTCUSDT&timeframes=1h&limit=50')
    })

    it('should get supported symbols', async () => {
      const mockSymbols: SupportedSymbols = {
        symbols: ['BTCUSDT', 'ETHUSDT'],
        available_timeframes: ['1h', '4h', '1d'],
      }

      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockSymbols } })

      const client = new BotCoreApiClient()
      const result = await client.rust.getSupportedSymbols()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market/symbols')
      expect(result).toEqual(mockSymbols)
    })

    it('should add symbol', async () => {
      const mockResponse = { message: 'Symbol added' }
      const request = {
        symbol: 'SOLUSDT',
        timeframes: ['1h', '4h'],
      }

      mockAxiosInstance.post.mockResolvedValue({ data: { data: mockResponse } })

      const client = new BotCoreApiClient()
      const result = await client.rust.addSymbol(request)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/market/symbols', request)
      expect(result).toEqual(mockResponse)
    })

    it('should remove symbol', async () => {
      const mockResponse = { message: 'Symbol removed' }
      mockAxiosInstance.delete.mockResolvedValue({ data: { data: mockResponse } })

      const client = new BotCoreApiClient()
      const result = await client.rust.removeSymbol('SOLUSDT')

      expect(mockAxiosInstance.delete).toHaveBeenCalledWith('/api/market/symbols/SOLUSDT')
      expect(result).toEqual(mockResponse)
    })

    it('should get latest prices', async () => {
      const mockPrices = {
        BTCUSDT: 50000,
        ETHUSDT: 3000,
      }

      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockPrices } })

      const client = new BotCoreApiClient()
      const result = await client.rust.getLatestPrices()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market/prices')
      expect(result).toEqual(mockPrices)
    })

    it('should get market overview', async () => {
      const mockOverview: MarketOverview = {
        total_symbols: 10,
        active_symbols: 8,
        market_status: 'active',
        last_update: '2024-01-01T00:00:00Z',
        top_performers: [],
        market_stats: {
          total_volume: 1000000,
          total_trades: 5000,
          avg_price_change: 1.5,
        },
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockOverview })

      const client = new BotCoreApiClient()
      const result = await client.rust.getMarketOverview()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/market/overview')
      expect(result).toEqual(mockOverview)
    })
  })

  describe('RustTradingApiClient - AI Integration', () => {
    it('should analyze AI', async () => {
      const mockResponse: AISignalResponse = {
        signal: 'long',
        confidence: 0.85,
        reasoning: 'Strong uptrend',
        strategy_scores: { rsi: 0.8 },
        market_analysis: {
          trend_direction: 'up',
          trend_strength: 0.8,
          support_levels: [49000],
          resistance_levels: [51000],
          volatility_level: 'medium',
          volume_analysis: 'increasing',
        },
        risk_assessment: {
          overall_risk: 'medium',
          technical_risk: 0.3,
          market_risk: 0.4,
          recommended_position_size: 1.5,
        },
        timestamp: 1234567890,
      }

      const request = {
        symbol: 'BTCUSDT',
        timeframe_data: {},
        current_price: 50000,
        volume_24h: 1000000,
        timestamp: 1234567890,
        strategy_context: {
          selected_strategies: ['rsi'],
          market_condition: 'trending',
          risk_level: 'medium',
          user_preferences: {},
          technical_indicators: {},
        },
      }

      mockAxiosInstance.post.mockResolvedValue({ data: { data: mockResponse } })

      const client = new BotCoreApiClient()
      const result = await client.rust.analyzeAI(request)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/ai/analyze', request)
      expect(result).toEqual(mockResponse)
    })

    it('should get strategy recommendations', async () => {
      const mockRecommendations: StrategyRecommendation[] = [
        {
          strategy_name: 'RSI',
          suitability_score: 0.9,
          reasoning: 'Good for current conditions',
          recommended_config: {},
        },
      ]

      const data = {
        symbol: 'BTCUSDT',
        timeframe_data: {},
        current_price: 50000,
        available_strategies: ['rsi', 'macd'],
        timestamp: 1234567890,
      }

      mockAxiosInstance.post.mockResolvedValue({ data: { data: mockRecommendations } })

      const client = new BotCoreApiClient()
      const result = await client.rust.getStrategyRecommendations(data)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/ai/strategy-recommendations', data)
      expect(result).toEqual(mockRecommendations)
    })

    it('should analyze market condition', async () => {
      const mockAnalysis: MarketConditionAnalysis = {
        condition_type: 'trending',
        confidence: 0.85,
        characteristics: ['high volume'],
        recommended_strategies: ['rsi'],
        market_phase: 'accumulation',
      }

      const data = {
        symbol: 'BTCUSDT',
        timeframe_data: {},
        current_price: 50000,
        volume_24h: 1000000,
        timestamp: 1234567890,
      }

      mockAxiosInstance.post.mockResolvedValue({ data: { data: mockAnalysis } })

      const client = new BotCoreApiClient()
      const result = await client.rust.analyzeMarketCondition(data)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/ai/market-condition', data)
      expect(result).toEqual(mockAnalysis)
    })

    it('should send AI feedback', async () => {
      const mockResponse = { message: 'Feedback received' }
      const feedback = {
        signal_id: 'sig123',
        symbol: 'BTCUSDT',
        predicted_signal: 'long',
        actual_outcome: 'profit',
        profit_loss: 500,
        confidence_was_accurate: true,
        timestamp: 1234567890,
      }

      mockAxiosInstance.post.mockResolvedValue({ data: { data: mockResponse } })

      const client = new BotCoreApiClient()
      const result = await client.rust.sendAIFeedback(feedback)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/ai/feedback', feedback)
      expect(result).toEqual(mockResponse)
    })

    it('should get AI service info', async () => {
      const mockInfo: AIServiceInfo = {
        service_name: 'AI Service',
        version: '1.0.0',
        model_version: '1.0',
        supported_timeframes: ['1h', '4h'],
        supported_symbols: ['BTCUSDT'],
        capabilities: ['prediction'],
      }

      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockInfo } })

      const client = new BotCoreApiClient()
      const result = await client.rust.getAIServiceInfo()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/ai/info')
      expect(result).toEqual(mockInfo)
    })

    it('should get supported strategies', async () => {
      const mockStrategies = { strategies: ['rsi', 'macd', 'bollinger'] }
      mockAxiosInstance.get.mockResolvedValue({ data: { data: mockStrategies } })

      const client = new BotCoreApiClient()
      const result = await client.rust.getSupportedStrategies()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/ai/strategies')
      expect(result).toEqual(mockStrategies)
    })
  })

  describe('RustTradingApiClient - Health Check', () => {
    it('should perform health check', async () => {
      const mockHealth = { status: 'healthy' }
      mockAxiosInstance.get.mockResolvedValue({ data: mockHealth })

      const client = new BotCoreApiClient()
      const result = await client.rust.healthCheck()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/health')
      expect(result).toEqual(mockHealth)
    })
  })

  describe('AuthApiClient', () => {
    it('should login successfully', async () => {
      const mockResponse: LoginResponse = {
        token: 'jwt-token-123',
        user: {
          id: 'user123',
          email: 'test@example.com',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          settings: {
            trading_enabled: true,
            risk_level: 'Medium',
            max_positions: 5,
            default_quantity: 1,
            notifications: {
              email_alerts: true,
              trade_notifications: true,
              system_alerts: true,
            },
          },
        },
      }

      const request: LoginRequest = {
        email: 'test@example.com',
        password: 'password123',
      }

      mockAxiosInstance.post.mockResolvedValue({
        data: { success: true, data: mockResponse },
      })

      const client = new BotCoreApiClient()
      const result = await client.auth.login(request)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/auth/login', request)
      expect(result).toEqual(mockResponse)
    })

    it('should handle login failure', async () => {
      const request: LoginRequest = {
        email: 'test@example.com',
        password: 'wrong-password',
      }

      mockAxiosInstance.post.mockResolvedValue({
        data: { success: false, error: 'Invalid credentials' },
      })

      const client = new BotCoreApiClient()

      await expect(client.auth.login(request)).rejects.toThrow('Invalid credentials')
    })

    it('should register successfully', async () => {
      const mockResponse: LoginResponse = {
        token: 'jwt-token-123',
        user: {
          id: 'user123',
          email: 'test@example.com',
          full_name: 'Test User',
          is_active: true,
          is_admin: false,
          created_at: '2024-01-01T00:00:00Z',
          settings: {
            trading_enabled: true,
            risk_level: 'Medium',
            max_positions: 5,
            default_quantity: 1,
            notifications: {
              email_alerts: true,
              trade_notifications: true,
              system_alerts: true,
            },
          },
        },
      }

      const request: RegisterRequest = {
        email: 'test@example.com',
        password: 'password123',
        full_name: 'Test User',
      }

      mockAxiosInstance.post.mockResolvedValue({
        data: { success: true, data: mockResponse },
      })

      const client = new BotCoreApiClient()
      const result = await client.auth.register(request)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/auth/register', request)
      expect(result).toEqual(mockResponse)
    })

    it('should handle registration failure', async () => {
      const request: RegisterRequest = {
        email: 'test@example.com',
        password: 'password123',
      }

      mockAxiosInstance.post.mockResolvedValue({
        data: { success: false, error: 'Email already exists' },
      })

      const client = new BotCoreApiClient()

      await expect(client.auth.register(request)).rejects.toThrow('Email already exists')
    })

    it('should verify token', async () => {
      const mockVerify = {
        user_id: 'user123',
        email: 'test@example.com',
        is_admin: false,
        exp: 1234567890,
      }

      mockAxiosInstance.get.mockResolvedValue({
        data: { success: true, data: mockVerify },
      })

      const client = new BotCoreApiClient()
      const result = await client.auth.verifyToken()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/auth/verify')
      expect(result).toEqual(mockVerify)
    })

    it('should handle token verification failure', async () => {
      mockAxiosInstance.get.mockResolvedValue({
        data: { success: false, error: 'Invalid token' },
      })

      const client = new BotCoreApiClient()

      await expect(client.auth.verifyToken()).rejects.toThrow('Invalid token')
    })

    it('should get user profile', async () => {
      const mockProfile: UserProfile = {
        id: 'user123',
        email: 'test@example.com',
        full_name: 'Test User',
        is_active: true,
        is_admin: false,
        created_at: '2024-01-01T00:00:00Z',
        settings: {
          trading_enabled: true,
          risk_level: 'Medium',
          max_positions: 5,
          default_quantity: 1,
          notifications: {
            email_alerts: true,
            trade_notifications: true,
            system_alerts: true,
          },
        },
      }

      mockAxiosInstance.get.mockResolvedValue({
        data: { success: true, data: mockProfile },
      })

      const client = new BotCoreApiClient()
      const result = await client.auth.getProfile()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/auth/profile')
      expect(result).toEqual(mockProfile)
    })

    it('should set auth token', () => {
      const client = new BotCoreApiClient()
      client.auth.setAuthToken('test-token')

      expect(localStorage.getItem('authToken')).toBe('test-token')
    })

    it('should remove auth token', () => {
      localStorage.setItem('authToken', 'test-token')

      const client = new BotCoreApiClient()
      client.auth.removeAuthToken()

      expect(localStorage.getItem('authToken')).toBeNull()
    })

    it('should get auth token', () => {
      localStorage.setItem('authToken', 'test-token')

      const client = new BotCoreApiClient()
      const token = client.auth.getAuthToken()

      expect(token).toBe('test-token')
    })

    it('should check if token is expired', () => {
      const client = new BotCoreApiClient()

      // Create a valid JWT token with future expiry
      const futureExp = Math.floor(Date.now() / 1000) + 3600
      const payload = btoa(JSON.stringify({ exp: futureExp }))
      const token = `header.${payload}.signature`

      expect(client.auth.isTokenExpired(token)).toBe(false)
    })

    it('should detect expired token', () => {
      const client = new BotCoreApiClient()

      // Create a JWT token with past expiry
      const pastExp = Math.floor(Date.now() / 1000) - 3600
      const payload = btoa(JSON.stringify({ exp: pastExp }))
      const token = `header.${payload}.signature`

      expect(client.auth.isTokenExpired(token)).toBe(true)
    })

    it('should return true for invalid token', () => {
      const client = new BotCoreApiClient()

      expect(client.auth.isTokenExpired('invalid-token')).toBe(true)
    })

    it('should return true for null token', () => {
      const client = new BotCoreApiClient()

      expect(client.auth.isTokenExpired()).toBe(true)
    })
  })

  describe('PythonAIApiClient', () => {
    it('should get model info', async () => {
      const mockInfo: AIModelInfo = {
        model_type: 'LSTM',
        model_loaded: true,
        training_samples: 10000,
        validation_samples: 2000,
        feature_count: 50,
        training_accuracy: 0.85,
        trained_timestamp: '2024-01-01T00:00:00Z',
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockInfo })

      const client = new BotCoreApiClient()
      const result = await client.python.getModelInfo()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/model/info')
      expect(result).toEqual(mockInfo)
    })

    it('should train model', async () => {
      const mockResponse = {
        message: 'Training completed',
        model_type: 'LSTM',
        training_samples: 10000,
        status: 'success',
      }

      const data = {
        symbol: 'BTCUSDT',
        model_type: 'LSTM',
        retrain: true,
        candles: [],
      }

      mockAxiosInstance.post.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.python.trainModel(data)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/train', data)
      expect(result).toEqual(mockResponse)
    })

    it('should load model', async () => {
      const mockResponse = {
        message: 'Model loaded',
        model_type: 'LSTM',
        status: 'success',
      }

      mockAxiosInstance.post.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.python.loadModel('model-path')

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/model/load', {
        model_path: 'model-path',
      })
      expect(result).toEqual(mockResponse)
    })

    it('should save model', async () => {
      const mockResponse = {
        message: 'Model saved',
        model_type: 'LSTM',
        timestamp: '2024-01-01T00:00:00Z',
      }

      mockAxiosInstance.post.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.python.saveModel('my-model')

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/model/save', {
        model_name: 'my-model',
      })
      expect(result).toEqual(mockResponse)
    })

    it('should analyze market', async () => {
      const mockSignal: AISignal = {
        signal: 'long',
        confidence: 0.85,
        probability: 0.75,
        timestamp: '2024-01-01T00:00:00Z',
        model_type: 'LSTM',
        symbol: 'BTCUSDT',
        timeframe: '1h',
      }

      const data = {
        symbol: 'BTCUSDT',
        timeframe: '1h',
        candles: [],
      }

      mockAxiosInstance.post.mockResolvedValue({ data: mockSignal })

      const client = new BotCoreApiClient()
      const result = await client.python.analyzeMarket(data)

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/analyze', data)
      expect(result).toEqual(mockSignal)
    })

    it('should get config', async () => {
      const mockConfig = {
        supported_timeframes: ['1h', '4h'],
        model_config: {
          model_type: 'LSTM',
          sequence_length: 60,
          batch_size: 32,
          learning_rate: 0.001,
          epochs: 100,
          validation_split: 0.2,
          features: ['close', 'volume'],
        },
        trading_config: {
          enabled: true,
          model_confidence_threshold: 0.7,
          max_signal_age_minutes: 15,
          supported_timeframes: ['1h'],
          risk_management: {
            max_position_size: 1000,
            stop_loss_percentage: 3,
            take_profit_percentage: 6,
          },
        },
        data_config: {
          data_source: 'binance',
          update_interval_seconds: 60,
          historical_data_days: 30,
          required_indicators: ['rsi', 'macd'],
          cache_enabled: true,
          cache_ttl_minutes: 5,
        },
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockConfig })

      const client = new BotCoreApiClient()
      const result = await client.python.getConfig()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/config')
      expect(result).toEqual(mockConfig)
    })

    it('should cleanup old models', async () => {
      const mockResponse = {
        message: 'Cleanup completed',
        deleted_models: 3,
        kept_models: 5,
      }

      mockAxiosInstance.delete.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      const result = await client.python.cleanupOldModels(5)

      expect(mockAxiosInstance.delete).toHaveBeenCalledWith('/model/cleanup?keep_count=5')
      expect(result).toEqual(mockResponse)
    })

    it('should cleanup old models with default count', async () => {
      const mockResponse = {
        message: 'Cleanup completed',
        deleted_models: 10,
        kept_models: 5,
      }

      mockAxiosInstance.delete.mockResolvedValue({ data: mockResponse })

      const client = new BotCoreApiClient()
      await client.python.cleanupOldModels()

      expect(mockAxiosInstance.delete).toHaveBeenCalledWith('/model/cleanup?keep_count=5')
    })

    it('should perform health check', async () => {
      const mockHealth = {
        status: 'healthy',
        timestamp: '2024-01-01T00:00:00Z',
        model_loaded: true,
        version: '1.0.0',
      }

      mockAxiosInstance.get.mockResolvedValue({ data: mockHealth })

      const client = new BotCoreApiClient()
      const result = await client.python.healthCheck()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/health')
      expect(result).toEqual(mockHealth)
    })
  })

  describe('BotCoreApiClient - Combined Health Check', () => {
    it('should perform combined health check when both services are healthy', async () => {
      mockAxiosInstance.get
        .mockResolvedValueOnce({ data: { status: 'healthy' } })
        .mockResolvedValueOnce({
          data: {
            status: 'healthy',
            timestamp: '2024-01-01T00:00:00Z',
            model_loaded: true,
            version: '1.0.0',
          },
        })

      const client = new BotCoreApiClient()
      const result = await client.healthCheck()

      expect(result).toEqual({
        rust: { status: 'healthy', healthy: true },
        python: { status: 'healthy', healthy: true, model_loaded: true },
        overall: true,
      })
    })

    it('should handle rust service failure', async () => {
      mockAxiosInstance.get
        .mockRejectedValueOnce(new Error('Rust service down'))
        .mockResolvedValueOnce({
          data: {
            status: 'healthy',
            timestamp: '2024-01-01T00:00:00Z',
            model_loaded: true,
            version: '1.0.0',
          },
        })

      const client = new BotCoreApiClient()
      const result = await client.healthCheck()

      expect(result).toEqual({
        rust: { status: 'error', healthy: false },
        python: { status: 'healthy', healthy: true, model_loaded: true },
        overall: false,
      })
    })

    it('should handle python service failure', async () => {
      mockAxiosInstance.get
        .mockResolvedValueOnce({ data: { status: 'healthy' } })
        .mockRejectedValueOnce(new Error('Python service down'))

      const client = new BotCoreApiClient()
      const result = await client.healthCheck()

      expect(result).toEqual({
        rust: { status: 'healthy', healthy: true },
        python: { status: 'error', healthy: false, model_loaded: false },
        overall: false,
      })
    })

    it('should handle both services failure', async () => {
      mockAxiosInstance.get
        .mockRejectedValueOnce(new Error('Rust service down'))
        .mockRejectedValueOnce(new Error('Python service down'))

      const client = new BotCoreApiClient()
      const result = await client.healthCheck()

      expect(result).toEqual({
        rust: { status: 'error', healthy: false },
        python: { status: 'error', healthy: false, model_loaded: false },
        overall: false,
      })
    })
  })

  describe('BotCoreApiClient - Get Dashboard Data', () => {
    it('should get complete dashboard data', async () => {
      const mockBotStatus: BotStatus = {
        status: 'running',
        uptime: 3600,
        active_positions: 2,
        total_trades: 50,
        total_pnl: 1234.56,
        last_update: '2024-01-01T00:00:00Z',
      }

      const mockPositions: Position[] = []

      const mockModelInfo: AIModelInfo = {
        model_type: 'LSTM',
        model_loaded: true,
      }

      const mockPerformanceStats: PerformanceStats = {
        total_pnl: 5000,
        win_rate: 0.75,
        total_trades: 100,
        avg_trade_duration: 3600,
        max_drawdown: 500,
        best_trade: 1000,
        worst_trade: -200,
        profit_factor: 3.0,
      }

      const mockRecentTrades: TradeHistory[] = []

      mockAxiosInstance.get
        .mockResolvedValueOnce({ data: mockBotStatus })
        .mockResolvedValueOnce({ data: mockPositions })
        .mockResolvedValueOnce({ data: mockModelInfo })
        .mockResolvedValueOnce({ data: mockPerformanceStats })
        .mockResolvedValueOnce({ data: mockRecentTrades })

      const client = new BotCoreApiClient()
      const result = await client.getDashboardData()

      expect(result).toEqual({
        botStatus: mockBotStatus,
        positions: mockPositions,
        aiModelInfo: mockModelInfo,
        performanceStats: mockPerformanceStats,
        recentTrades: mockRecentTrades,
      })
    })

    it('should handle dashboard data fetch failure', async () => {
      mockAxiosInstance.get.mockRejectedValue(new Error('API Error'))

      const client = new BotCoreApiClient()

      await expect(client.getDashboardData()).rejects.toThrow('API Error')
    })
  })

  describe('Singleton Instances', () => {
    it('should export singleton apiClient', () => {
      expect(apiClient).toBeInstanceOf(BotCoreApiClient)
    })

    it('should export rustApi singleton', () => {
      expect(rustApi).toBeDefined()
    })

    it('should export pythonAI singleton', () => {
      expect(pythonAI).toBeDefined()
    })
  })

  describe('Error Handling', () => {
    it('should handle network errors', async () => {
      mockAxiosInstance.get.mockRejectedValue(new Error('Network Error'))

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toThrow('Network Error')
    }, 10000) // 10s timeout for retry logic

    it('should handle 400 errors', async () => {
      const error = {
        response: {
          status: 400,
          data: { error: 'Bad Request' },
        },
      }

      mockAxiosInstance.get.mockRejectedValue(error)

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toEqual(error)
    }, 10000)

    it('should handle 401 errors', async () => {
      const error = {
        response: {
          status: 401,
          data: { error: 'Unauthorized' },
        },
      }

      mockAxiosInstance.get.mockRejectedValue(error)

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toEqual(error)
    }, 10000)

    it('should handle 403 errors', async () => {
      const error = {
        response: {
          status: 403,
          data: { error: 'Forbidden' },
        },
      }

      mockAxiosInstance.get.mockRejectedValue(error)

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toEqual(error)
    }, 10000)

    it('should handle 404 errors', async () => {
      const error = {
        response: {
          status: 404,
          data: { error: 'Not Found' },
        },
      }

      mockAxiosInstance.get.mockRejectedValue(error)

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toEqual(error)
    }, 10000)

    it('should handle 500 errors', async () => {
      const error = {
        response: {
          status: 500,
          data: { error: 'Internal Server Error' },
        },
      }

      mockAxiosInstance.get.mockRejectedValue(error)

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toEqual(error)
    }, 10000)

    it('should handle timeout errors', async () => {
      const error = {
        code: 'ECONNABORTED',
        message: 'timeout of 10000ms exceeded',
      }

      mockAxiosInstance.get.mockRejectedValue(error)

      const client = new BotCoreApiClient()

      await expect(client.rust.getBotStatus()).rejects.toEqual(error)
    }, 10000)
  })

  describe('Response Data Extraction', () => {
    it('should extract data from wrapped response', async () => {
      const mockData = { symbol: 'BTCUSDT' }
      mockAxiosInstance.get.mockResolvedValue({
        data: { data: mockData },
      })

      const client = new BotCoreApiClient()
      const result = await client.rust.getSupportedSymbols()

      expect(result).toEqual(mockData)
    })

    it('should use direct response if no wrapper', async () => {
      const mockData = { symbol: 'BTCUSDT' }
      mockAxiosInstance.get.mockResolvedValue({
        data: mockData,
      })

      const client = new BotCoreApiClient()
      const result = await client.rust.getSupportedSymbols()

      expect(result).toEqual(mockData)
    })
  })
})
