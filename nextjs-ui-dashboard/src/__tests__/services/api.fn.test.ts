import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'

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
    patch: vi.fn(),
    delete: vi.fn(),
    interceptors: {
      request: {
        use: vi.fn((callback: any) => {
          // Store the request interceptor for testing
          mockInstance._requestInterceptor = callback
          return 1
        }),
      },
      response: {
        use: vi.fn((successCallback: any, errorCallback: any) => {
          // Store the response interceptors for testing
          mockInstance._responseSuccessInterceptor = successCallback
          mockInstance._responseErrorInterceptor = errorCallback
          return 1
        }),
      },
    },
    _requestInterceptor: null as any,
    _responseSuccessInterceptor: null as any,
    _responseErrorInterceptor: null as any,
  }

  return {
    default: {
      create: vi.fn(() => mockInstance),
    },
  }
})

// Import after mocking
import axios from 'axios'
import { BotCoreApiClient, apiClient } from '../../services/api'

const mockAxiosInstance = (axios.create as any)()

describe('API Service - Additional Function Tests', () => {
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
    mockAxiosInstance.patch.mockReset()
    mockAxiosInstance.delete.mockReset()
  })

  afterEach(() => {
    vi.clearAllMocks()
  })

  describe('AuthApiClient - Change Password', () => {
    it('should change password successfully', async () => {
      const mockResponse = {
        data: {
          success: true,
          message: 'Password changed successfully',
        },
      }

      mockAxiosInstance.post.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.changePassword({
        current_password: 'oldpass123',
        new_password: 'newpass456',
      })

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/auth/change-password', {
        current_password: 'oldpass123',
        new_password: 'newpass456',
      })
      expect(result).toEqual({
        success: true,
        message: 'Password changed successfully',
      })
    })

    it('should handle change password failure', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: 'Current password is incorrect',
        },
      }

      // Mock all retry attempts to fail
      mockAxiosInstance.post.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(
        client.auth.changePassword({
          current_password: 'wrongpass',
          new_password: 'newpass456',
        })
      ).rejects.toThrow('Current password is incorrect')
    })

    it('should handle network error on change password', async () => {
      // Mock all retry attempts to fail
      mockAxiosInstance.post.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(
        client.auth.changePassword({
          current_password: 'oldpass',
          new_password: 'newpass',
        })
      ).rejects.toThrow('Network error')
    })

    it('should retry on failure and succeed', async () => {
      const mockResponse = {
        data: {
          success: true,
          message: 'Password changed successfully',
        },
      }

      // First call fails, second succeeds
      mockAxiosInstance.post
        .mockRejectedValueOnce(new Error('Temporary error'))
        .mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.changePassword({
        current_password: 'oldpass',
        new_password: 'newpass',
      })

      expect(mockAxiosInstance.post).toHaveBeenCalledTimes(2)
      expect(result.success).toBe(true)
    })
  })

  describe('AuthApiClient - Update Profile', () => {
    it('should update profile successfully', async () => {
      const mockProfile = {
        id: 'user-123',
        email: 'user@example.com',
        display_name: 'New Display Name',
        is_active: true,
        is_admin: false,
        two_factor_enabled: false,
        created_at: '2024-01-01T00:00:00Z',
        settings: {
          trading_enabled: true,
          risk_level: 'Medium' as const,
          max_positions: 5,
          default_quantity: 100,
          notifications: {
            email_alerts: true,
            trade_notifications: true,
            system_alerts: true,
          },
        },
      }

      const mockResponse = {
        data: {
          success: true,
          data: mockProfile,
        },
      }

      mockAxiosInstance.patch.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.updateProfile({
        display_name: 'New Display Name',
      })

      expect(mockAxiosInstance.patch).toHaveBeenCalledWith('/api/auth/profile', {
        display_name: 'New Display Name',
      })
      expect(result).toEqual(mockProfile)
    })

    it('should handle update profile failure', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: 'Invalid display name',
        },
      }

      mockAxiosInstance.patch.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(
        client.auth.updateProfile({
          display_name: '',
        })
      ).rejects.toThrow('Invalid display name')
    })

    it('should handle network error on update profile', async () => {
      mockAxiosInstance.patch.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(
        client.auth.updateProfile({
          display_name: 'Test',
        })
      ).rejects.toThrow('Network error')
    })
  })

  describe('AuthApiClient - 2FA Setup', () => {
    it('should setup 2FA successfully', async () => {
      const mock2FAResponse = {
        secret: 'JBSWY3DPEHPK3PXP',
        qr_code: 'data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAA...',
        otpauth_url: 'otpauth://totp/BotCore:user@example.com?secret=JBSWY3DPEHPK3PXP',
      }

      const mockResponse = {
        data: {
          success: true,
          data: mock2FAResponse,
        },
      }

      mockAxiosInstance.post.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.setup2FA()

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/auth/2fa/setup')
      expect(result).toEqual(mock2FAResponse)
      expect(result.secret).toBe('JBSWY3DPEHPK3PXP')
    })

    it('should handle 2FA setup failure', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: '2FA already enabled',
        },
      }

      mockAxiosInstance.post.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(client.auth.setup2FA()).rejects.toThrow('2FA already enabled')
    })

    it('should handle network error on 2FA setup', async () => {
      mockAxiosInstance.post.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(client.auth.setup2FA()).rejects.toThrow('Network error')
    })
  })

  describe('AuthApiClient - 2FA Verify', () => {
    it('should verify 2FA code successfully', async () => {
      const mockResponse = {
        data: {
          success: true,
          message: '2FA enabled successfully',
        },
      }

      mockAxiosInstance.post.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.verify2FA({ code: '123456' })

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/auth/2fa/verify', {
        code: '123456',
      })
      expect(result).toEqual({
        success: true,
        message: '2FA enabled successfully',
      })
    })

    it('should handle invalid 2FA code', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: 'Invalid 2FA code',
        },
      }

      mockAxiosInstance.post.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(client.auth.verify2FA({ code: '000000' })).rejects.toThrow(
        'Invalid 2FA code'
      )
    })

    it('should handle network error on 2FA verify', async () => {
      mockAxiosInstance.post.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(client.auth.verify2FA({ code: '123456' })).rejects.toThrow(
        'Network error'
      )
    })
  })

  describe('AuthApiClient - 2FA Disable', () => {
    it('should disable 2FA successfully', async () => {
      const mockResponse = {
        data: {
          success: true,
          message: '2FA disabled successfully',
        },
      }

      mockAxiosInstance.post.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.disable2FA({ code: '123456' })

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/auth/2fa/disable', {
        code: '123456',
      })
      expect(result).toEqual({
        success: true,
        message: '2FA disabled successfully',
      })
    })

    it('should handle invalid 2FA code on disable', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: 'Invalid 2FA code',
        },
      }

      mockAxiosInstance.post.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(client.auth.disable2FA({ code: '000000' })).rejects.toThrow(
        'Invalid 2FA code'
      )
    })

    it('should handle network error on 2FA disable', async () => {
      mockAxiosInstance.post.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(client.auth.disable2FA({ code: '123456' })).rejects.toThrow(
        'Network error'
      )
    })
  })

  describe('AuthApiClient - Session Management', () => {
    it('should get sessions successfully', async () => {
      const mockSessions = [
        {
          session_id: 'session-1',
          device: 'Chrome on Windows',
          browser: 'Chrome 120',
          os: 'Windows 11',
          ip_address: '192.168.1.1',
          location: 'New York, US',
          created_at: '2024-01-01T00:00:00Z',
          last_active: '2024-01-15T12:00:00Z',
          is_current: true,
        },
        {
          session_id: 'session-2',
          device: 'Safari on iPhone',
          browser: 'Safari 17',
          os: 'iOS 17',
          ip_address: '192.168.1.2',
          location: 'San Francisco, US',
          created_at: '2024-01-10T00:00:00Z',
          last_active: '2024-01-14T10:00:00Z',
          is_current: false,
        },
      ]

      const mockResponse = {
        data: {
          success: true,
          data: {
            sessions: mockSessions,
          },
        },
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.getSessions()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/auth/sessions')
      expect(result).toEqual(mockSessions)
      expect(result).toHaveLength(2)
    })

    it('should handle get sessions failure', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: 'Unauthorized',
        },
      }

      mockAxiosInstance.get.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(client.auth.getSessions()).rejects.toThrow('Unauthorized')
    })

    it('should handle network error on get sessions', async () => {
      mockAxiosInstance.get.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(client.auth.getSessions()).rejects.toThrow('Network error')
    })

    it('should revoke session successfully', async () => {
      const mockResponse = {
        data: {
          success: true,
          message: 'Session revoked',
        },
      }

      mockAxiosInstance.delete.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.revokeSession('session-123')

      expect(mockAxiosInstance.delete).toHaveBeenCalledWith('/api/auth/sessions/session-123')
      expect(result).toEqual({
        success: true,
        message: 'Session revoked',
      })
    })

    it('should handle revoke session failure', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: 'Session not found',
        },
      }

      mockAxiosInstance.delete.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(client.auth.revokeSession('invalid-session')).rejects.toThrow(
        'Session not found'
      )
    })

    it('should handle network error on revoke session', async () => {
      mockAxiosInstance.delete.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(client.auth.revokeSession('session-123')).rejects.toThrow(
        'Network error'
      )
    })

    it('should revoke all sessions successfully', async () => {
      const mockResponse = {
        data: {
          success: true,
          message: 'All sessions revoked',
          data: {
            revoked_count: 5,
          },
        },
      }

      mockAxiosInstance.post.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.revokeAllSessions()

      expect(mockAxiosInstance.post).toHaveBeenCalledWith('/api/auth/sessions/revoke-all')
      expect(result).toEqual({
        success: true,
        message: 'All sessions revoked',
        revoked_count: 5,
      })
    })

    it('should handle revoke all sessions with no data', async () => {
      const mockResponse = {
        data: {
          success: true,
          message: 'All sessions revoked',
        },
      }

      mockAxiosInstance.post.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.auth.revokeAllSessions()

      expect(result.revoked_count).toBe(0)
    })

    it('should handle revoke all sessions failure', async () => {
      const mockResponse = {
        data: {
          success: false,
          error: 'Unauthorized',
        },
      }

      mockAxiosInstance.post.mockResolvedValue(mockResponse)

      const client = new BotCoreApiClient()
      await expect(client.auth.revokeAllSessions()).rejects.toThrow('Unauthorized')
    })

    it('should handle network error on revoke all sessions', async () => {
      mockAxiosInstance.post.mockRejectedValue(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(client.auth.revokeAllSessions()).rejects.toThrow('Network error')
    })
  })

  describe('RustTradingApiClient - getChartDataFast', () => {
    it('should get chart data fast without retry', async () => {
      const mockChartData = {
        symbol: 'BTCUSDT',
        timeframe: '1h',
        candles: [
          {
            timestamp: 1704067200000,
            open: 45000,
            high: 45500,
            low: 44800,
            close: 45200,
            volume: 1000,
          },
        ],
        latest_price: 45200,
        volume_24h: 50000,
        price_change_24h: 200,
        price_change_percent_24h: 0.44,
      }

      const mockResponse = {
        data: {
          data: mockChartData,
        },
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.rust.getChartDataFast('BTCUSDT', '1h')

      expect(mockAxiosInstance.get).toHaveBeenCalledWith(
        '/api/market/chart/BTCUSDT/1h',
        { signal: undefined }
      )
      expect(result).toEqual(mockChartData)
    })

    it('should get chart data fast with limit', async () => {
      const mockChartData = {
        symbol: 'ETHUSDT',
        timeframe: '15m',
        candles: [],
        latest_price: 3000,
        volume_24h: 30000,
        price_change_24h: 50,
        price_change_percent_24h: 1.67,
      }

      const mockResponse = {
        data: {
          data: mockChartData,
        },
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.rust.getChartDataFast('ETHUSDT', '15m', 50)

      expect(mockAxiosInstance.get).toHaveBeenCalledWith(
        '/api/market/chart/ETHUSDT/15m?limit=50',
        { signal: undefined }
      )
      expect(result).toEqual(mockChartData)
    })

    it('should get chart data fast with abort signal', async () => {
      const mockChartData = {
        symbol: 'BNBUSDT',
        timeframe: '5m',
        candles: [],
        latest_price: 400,
        volume_24h: 10000,
        price_change_24h: 10,
        price_change_percent_24h: 2.5,
      }

      const mockResponse = {
        data: mockChartData,
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const abortController = new AbortController()
      const client = new BotCoreApiClient()
      const result = await client.rust.getChartDataFast(
        'BNBUSDT',
        '5m',
        undefined,
        abortController.signal
      )

      expect(mockAxiosInstance.get).toHaveBeenCalledWith(
        '/api/market/chart/BNBUSDT/5m',
        { signal: abortController.signal }
      )
      expect(result).toEqual(mockChartData)
    })

    it('should handle error on chart data fast without retry', async () => {
      mockAxiosInstance.get.mockRejectedValueOnce(new Error('Network error'))

      const client = new BotCoreApiClient()
      await expect(
        client.rust.getChartDataFast('BTCUSDT', '1h')
      ).rejects.toThrow('Network error')

      // Should NOT retry - called only once
      expect(mockAxiosInstance.get).toHaveBeenCalledTimes(1)
    })

    it('should extract data from wrapper response', async () => {
      const mockChartData = {
        symbol: 'BTCUSDT',
        timeframe: '1h',
        candles: [],
        latest_price: 45000,
        volume_24h: 50000,
        price_change_24h: 100,
        price_change_percent_24h: 0.22,
      }

      const mockResponse = {
        data: {
          success: true,
          data: mockChartData,
        },
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.rust.getChartDataFast('BTCUSDT', '1h')

      expect(result).toEqual(mockChartData)
    })

    it('should use direct response if no wrapper', async () => {
      const mockChartData = {
        symbol: 'BTCUSDT',
        timeframe: '1h',
        candles: [],
        latest_price: 45000,
        volume_24h: 50000,
        price_change_24h: 100,
        price_change_percent_24h: 0.22,
      }

      const mockResponse = {
        data: mockChartData,
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.rust.getChartDataFast('BTCUSDT', '1h')

      expect(result).toEqual(mockChartData)
    })
  })

  describe('AuthApiClient - Token Utility Methods', () => {
    it('should set auth token in localStorage', () => {
      const client = new BotCoreApiClient()
      client.auth.setAuthToken('test-token-123')

      expect(localStorage.getItem('authToken')).toBe('test-token-123')
    })

    it('should remove auth token from localStorage', () => {
      localStorage.setItem('authToken', 'test-token-123')

      const client = new BotCoreApiClient()
      client.auth.removeAuthToken()

      expect(localStorage.getItem('authToken')).toBeNull()
    })

    it('should get auth token from localStorage', () => {
      localStorage.setItem('authToken', 'test-token-123')

      const client = new BotCoreApiClient()
      const token = client.auth.getAuthToken()

      expect(token).toBe('test-token-123')
    })

    it('should return null when no token in localStorage', () => {
      const client = new BotCoreApiClient()
      const token = client.auth.getAuthToken()

      expect(token).toBeNull()
    })

    it('should check if token is not expired', () => {
      // Create token that expires in 1 hour
      const futureExp = Math.floor(Date.now() / 1000) + 3600
      const payload = btoa(JSON.stringify({ exp: futureExp }))
      const token = `header.${payload}.signature`

      const client = new BotCoreApiClient()
      const isExpired = client.auth.isTokenExpired(token)

      expect(isExpired).toBe(false)
    })

    it('should check if token is expired', () => {
      // Create token that expired 1 hour ago
      const pastExp = Math.floor(Date.now() / 1000) - 3600
      const payload = btoa(JSON.stringify({ exp: pastExp }))
      const token = `header.${payload}.signature`

      const client = new BotCoreApiClient()
      const isExpired = client.auth.isTokenExpired(token)

      expect(isExpired).toBe(true)
    })

    it('should return true for invalid token format', () => {
      const client = new BotCoreApiClient()
      const isExpired = client.auth.isTokenExpired('invalid-token')

      expect(isExpired).toBe(true)
    })

    it('should return true for null token', () => {
      const client = new BotCoreApiClient()
      const isExpired = client.auth.isTokenExpired()

      expect(isExpired).toBe(true)
    })

    it('should check token from localStorage if not provided', () => {
      // Create expired token in localStorage
      const pastExp = Math.floor(Date.now() / 1000) - 3600
      const payload = btoa(JSON.stringify({ exp: pastExp }))
      const token = `header.${payload}.signature`
      localStorage.setItem('authToken', token)

      const client = new BotCoreApiClient()
      const isExpired = client.auth.isTokenExpired()

      expect(isExpired).toBe(true)
    })
  })

  describe('Request Interceptor Tests', () => {
    it('should create axios instance with interceptors', () => {
      const client = new BotCoreApiClient()

      // Verify interceptors were registered
      expect(mockAxiosInstance.interceptors.request.use).toHaveBeenCalled()
      expect(mockAxiosInstance.interceptors.response.use).toHaveBeenCalled()
      expect(client.auth).toBeDefined()
      expect(client.rust).toBeDefined()
      expect(client.python).toBeDefined()
    })

    it('should handle token in request via auth methods', async () => {
      localStorage.setItem('authToken', 'test-token-123')

      const mockResponse = {
        data: {
          success: true,
          data: {
            user_id: 'user-123',
            email: 'test@example.com',
            is_admin: false,
            exp: Math.floor(Date.now() / 1000) + 3600,
          },
        },
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      await client.auth.verifyToken()

      // Verify the call was made
      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/auth/verify')
    })

    it('should work without token in localStorage', async () => {
      localStorage.clear()

      const mockResponse = {
        data: {
          status: 'ok',
        },
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      await client.rust.healthCheck()

      expect(mockAxiosInstance.get).toHaveBeenCalledWith('/api/health')
    })
  })

  describe('Response Interceptor Tests', () => {
    it('should pass through successful responses', () => {
      const mockResponse = {
        data: { test: 'data' },
        status: 200,
      }

      // Create client to trigger interceptor setup
      new BotCoreApiClient()

      // Get the success interceptor
      const interceptor = mockAxiosInstance._responseSuccessInterceptor
      const result = interceptor(mockResponse)

      expect(result).toEqual(mockResponse)
    })

    it('should reject errors through error interceptor', async () => {
      const mockError = {
        response: {
          status: 404,
          data: { error: 'Not found' },
        },
        message: 'Request failed',
      }

      // Create client to trigger interceptor setup
      new BotCoreApiClient()

      // Get the error interceptor
      const interceptor = mockAxiosInstance._responseErrorInterceptor

      await expect(interceptor(mockError)).rejects.toEqual(mockError)
    })

    it('should handle network errors without response', async () => {
      const mockError = {
        message: 'Network Error',
      }

      // Create client to trigger interceptor setup
      new BotCoreApiClient()

      // Get the error interceptor
      const interceptor = mockAxiosInstance._responseErrorInterceptor

      await expect(interceptor(mockError)).rejects.toEqual(mockError)
    })
  })

  describe('Edge Cases - Empty Responses', () => {
    it('should handle empty response data', async () => {
      const mockResponse = {
        data: {},
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.rust.getLatestPrices()

      expect(result).toEqual({})
    })

    it('should handle undefined response data', async () => {
      const mockResponse = {
        data: {
          data: undefined,
        },
      }

      mockAxiosInstance.get.mockResolvedValueOnce(mockResponse)

      const client = new BotCoreApiClient()
      const result = await client.rust.getLatestPrices()

      expect(result).toEqual({ data: undefined })
    })
  })

  describe('Edge Cases - Non-OK Status Codes', () => {
    it('should handle 400 Bad Request', async () => {
      const mockError = {
        response: {
          status: 400,
          data: { error: 'Bad request' },
        },
        message: 'Request failed with status code 400',
      }

      mockAxiosInstance.post.mockRejectedValue(mockError)

      const client = new BotCoreApiClient()
      await expect(
        client.auth.changePassword({
          current_password: 'test',
          new_password: 'test',
        })
      ).rejects.toMatchObject({
        response: {
          status: 400,
        },
      })
    })

    it('should handle 401 Unauthorized', async () => {
      const mockError = {
        response: {
          status: 401,
          data: { error: 'Unauthorized' },
        },
        message: 'Request failed with status code 401',
      }

      mockAxiosInstance.get.mockRejectedValue(mockError)

      const client = new BotCoreApiClient()
      await expect(client.auth.getSessions()).rejects.toMatchObject({
        response: {
          status: 401,
        },
      })
    })

    it('should handle 403 Forbidden', async () => {
      const mockError = {
        response: {
          status: 403,
          data: { error: 'Forbidden' },
        },
        message: 'Request failed with status code 403',
      }

      mockAxiosInstance.delete.mockRejectedValue(mockError)

      const client = new BotCoreApiClient()
      await expect(client.auth.revokeSession('session-123')).rejects.toMatchObject({
        response: {
          status: 403,
        },
      })
    })

    it('should handle 404 Not Found', async () => {
      const mockError = {
        response: {
          status: 404,
          data: { error: 'Not found' },
        },
        message: 'Request failed with status code 404',
      }

      mockAxiosInstance.get.mockRejectedValue(mockError)

      const client = new BotCoreApiClient()
      await expect(client.rust.getChartDataFast('INVALID', '1h')).rejects.toMatchObject({
        response: {
          status: 404,
        },
      })
    })

    it('should handle 500 Internal Server Error', async () => {
      const mockError = {
        response: {
          status: 500,
          data: { error: 'Internal server error' },
        },
        message: 'Request failed with status code 500',
      }

      mockAxiosInstance.post.mockRejectedValue(mockError)

      const client = new BotCoreApiClient()
      await expect(client.auth.setup2FA()).rejects.toMatchObject({
        response: {
          status: 500,
        },
      })
    })

    it('should handle 503 Service Unavailable', async () => {
      const mockError = {
        response: {
          status: 503,
          data: { error: 'Service unavailable' },
        },
        message: 'Request failed with status code 503',
      }

      mockAxiosInstance.get.mockRejectedValue(mockError)

      const client = new BotCoreApiClient()
      await expect(client.auth.getProfile()).rejects.toMatchObject({
        response: {
          status: 503,
        },
      })
    })
  })

  describe('Retry Logic Edge Cases', () => {
    it('should fail immediately after max retries (2) for changePassword', async () => {
      mockAxiosInstance.post
        .mockRejectedValueOnce(new Error('Error 1'))
        .mockRejectedValueOnce(new Error('Error 2'))

      const client = new BotCoreApiClient()
      await expect(
        client.auth.changePassword({
          current_password: 'test',
          new_password: 'test',
        })
      ).rejects.toThrow('Error 2')

      expect(mockAxiosInstance.post).toHaveBeenCalledTimes(2)
    })

    it('should succeed on second retry for updateProfile', async () => {
      const mockProfile = {
        id: 'user-123',
        email: 'user@example.com',
        display_name: 'Test User',
        is_active: true,
        is_admin: false,
        two_factor_enabled: false,
        created_at: '2024-01-01T00:00:00Z',
        settings: {
          trading_enabled: true,
          risk_level: 'Medium' as const,
          max_positions: 5,
          default_quantity: 100,
          notifications: {
            email_alerts: true,
            trade_notifications: true,
            system_alerts: true,
          },
        },
      }

      mockAxiosInstance.patch
        .mockRejectedValueOnce(new Error('Temporary error'))
        .mockResolvedValueOnce({
          data: {
            success: true,
            data: mockProfile,
          },
        })

      const client = new BotCoreApiClient()
      const result = await client.auth.updateProfile({ display_name: 'Test User' })

      expect(mockAxiosInstance.patch).toHaveBeenCalledTimes(2)
      expect(result).toEqual(mockProfile)
    })

    it('should handle timeout with retry for getSessions', async () => {
      const mockSessions = [
        {
          session_id: 'session-1',
          device: 'Chrome',
          browser: 'Chrome',
          os: 'Windows',
          ip_address: '1.1.1.1',
          location: 'US',
          created_at: '2024-01-01T00:00:00Z',
          last_active: '2024-01-01T00:00:00Z',
          is_current: true,
        },
      ]

      mockAxiosInstance.get
        .mockRejectedValueOnce(new Error('Timeout'))
        .mockResolvedValueOnce({
          data: {
            success: true,
            data: { sessions: mockSessions },
          },
        })

      const client = new BotCoreApiClient()
      const result = await client.auth.getSessions()

      expect(mockAxiosInstance.get).toHaveBeenCalledTimes(2)
      expect(result).toEqual(mockSessions)
    })
  })

  describe('Combined API Client Integration', () => {
    it('should have rust, python, and auth clients', () => {
      const client = new BotCoreApiClient()

      expect(client.rust).toBeDefined()
      expect(client.python).toBeDefined()
      expect(client.auth).toBeDefined()
    })

    it('should export singleton apiClient', () => {
      expect(apiClient).toBeDefined()
      expect(apiClient.rust).toBeDefined()
      expect(apiClient.python).toBeDefined()
      expect(apiClient.auth).toBeDefined()
    })
  })
})
