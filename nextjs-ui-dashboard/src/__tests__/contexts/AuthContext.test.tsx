import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock the entire API module - MUST be before any imports that use it
vi.mock('@/services/api', () => {
  const mockAuthClient = {
    login: vi.fn(),
    register: vi.fn(),
    getProfile: vi.fn(),
    verifyToken: vi.fn(),
    setAuthToken: vi.fn(),
    removeAuthToken: vi.fn(),
    getAuthToken: vi.fn(() => null),
    isTokenExpired: vi.fn(() => true),
  }

  return {
    BotCoreApiClient: vi.fn(function() {
      this.auth = mockAuthClient
      this.rust = {}
      this.python = {}
    }),
    mockAuthClient, // Export for test access
  }
})

// Mock localStorage
Object.defineProperty(window, 'localStorage', {
  value: mockLocalStorage(),
  writable: true,
})

// Import other dependencies AFTER the mock
import { renderHook, act, waitFor } from '@testing-library/react'
import { AuthProvider, useAuth } from '../../contexts/AuthContext'
import { mockUser, mockLocalStorage } from '../../test/utils'
import React from 'react'

// Get mock auth client for test assertions
const { mockAuthClient } = await import('@/services/api')

describe('AuthContext', () => {
  const wrapper = ({ children }: { children: React.ReactNode }) => (
    <AuthProvider>{children}</AuthProvider>
  )

  beforeEach(() => {
    vi.clearAllMocks()
    window.localStorage.clear()

    // Reset mock implementations
    mockAuthClient.getAuthToken.mockReturnValue(null)
    mockAuthClient.isTokenExpired.mockReturnValue(true)
    mockAuthClient.login.mockResolvedValue({
      token: 'mock-jwt-token',
      user: mockUser,
    })
    mockAuthClient.register.mockResolvedValue({
      token: 'mock-jwt-token',
      user: mockUser,
    })
    mockAuthClient.getProfile.mockResolvedValue(mockUser)
  })

  it('initializes with no user', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper })

    // Wait for initialization to complete
    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.user).toBeNull()
    expect(result.current.isAuthenticated).toBe(false)
  })

  it.todo('loads user from localStorage on init', async () => {
    window.localStorage.setItem('auth_token', 'mock-token')
    window.localStorage.setItem('user', JSON.stringify(mockUser))

    const { result } = renderHook(() => useAuth(), { wrapper })

    // Wait for async useEffect to complete
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 100))
    })

    expect(result.current.user).toEqual(mockUser)
    expect(result.current.isAuthenticated).toBe(true)
  })

  it('logs in user successfully', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper })

    let loginResult: boolean = false
    await act(async () => {
      loginResult = await result.current.login('test@example.com', 'password123')
    })

    expect(loginResult).toBe(true)
    expect(result.current.user).toBeDefined()
    expect(result.current.isAuthenticated).toBe(true)
  })

  it('handles login failure', async () => {
    // Mock login to reject with error
    mockAuthClient.login.mockRejectedValueOnce(new Error('Invalid credentials'))

    const { result } = renderHook(() => useAuth(), { wrapper })

    let loginResult: boolean = true
    await act(async () => {
      loginResult = await result.current.login('test@example.com', 'wrongpassword')
    })

    expect(loginResult).toBe(false)
    expect(result.current.user).toBeNull()
    expect(result.current.isAuthenticated).toBe(false)
    expect(result.current.error).toBeTruthy()
  })

  it.todo('registers user successfully', async () => {
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({
        token: 'new-token',
        user: mockUser,
      }),
    })
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    await act(async () => {
      await result.current.register({
        email: 'test@example.com',
        password: 'password123',
        full_name: 'Test User',
      })
    })
    
    expect(result.current.user).toEqual(mockUser)
    expect(result.current.isAuthenticated).toBe(true)
    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:8080/api/auth/register',
      expect.objectContaining({
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          email: 'test@example.com',
          password: 'password123',
          full_name: 'Test User',
        }),
      })
    )
  })

  it.todo('logs out user', () => {
    window.localStorage.setItem('auth_token', 'mock-token')
    window.localStorage.setItem('user', JSON.stringify(mockUser))
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    act(() => {
      result.current.logout()
    })
    
    expect(result.current.user).toBeNull()
    expect(result.current.isAuthenticated).toBe(false)
    expect(window.localStorage.getItem('auth_token')).toBeNull()
    expect(window.localStorage.getItem('user')).toBeNull()
  })

  it.todo('validates existing token on mount', async () => {
    window.localStorage.setItem('auth_token', 'existing-token')
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => mockUser,
    })
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    // Should be loading initially
    expect(result.current.loading).toBe(true)
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 0))
    })
    
    expect(result.current.user).toEqual(mockUser)
    expect(result.current.loading).toBe(false)
    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:8080/api/auth/me',
      expect.objectContaining({
        headers: {
          Authorization: 'Bearer existing-token',
        },
      })
    )
  })

  it.todo('clears invalid token on validation failure', async () => {
    window.localStorage.setItem('auth_token', 'invalid-token')
    mockFetch.mockResolvedValueOnce({
      ok: false,
      status: 401,
    })
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    await act(async () => {
      await new Promise(resolve => setTimeout(resolve, 0))
    })
    
    expect(result.current.user).toBeNull()
    expect(result.current.isAuthenticated).toBe(false)
    expect(window.localStorage.getItem('auth_token')).toBeNull()
  })

  it.todo('updates user profile', async () => {
    window.localStorage.setItem('auth_token', 'mock-token')
    window.localStorage.setItem('user', JSON.stringify(mockUser))
    
    const updatedUser = { ...mockUser, full_name: 'Updated Name' }
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => updatedUser,
    })
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    await act(async () => {
      await result.current.updateProfile({
        full_name: 'Updated Name',
      })
    })
    
    expect(result.current.user).toEqual(updatedUser)
    expect(JSON.parse(window.localStorage.getItem('user')!)).toEqual(updatedUser)
  })

  it.todo('changes password successfully', async () => {
    window.localStorage.setItem('auth_token', 'mock-token')
    mockFetch.mockResolvedValueOnce({
      ok: true,
      json: async () => ({ message: 'Password updated' }),
    })
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    await act(async () => {
      await result.current.changePassword({
        currentPassword: 'oldpassword',
        newPassword: 'newpassword',
      })
    })
    
    expect(mockFetch).toHaveBeenCalledWith(
      'http://localhost:8080/api/auth/change-password',
      expect.objectContaining({
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
          Authorization: 'Bearer mock-token',
        },
        body: JSON.stringify({
          currentPassword: 'oldpassword',
          newPassword: 'newpassword',
        }),
      })
    )
  })

  it('handles network errors gracefully', async () => {
    // Simulate network error
    mockAuthClient.login.mockRejectedValueOnce(new Error('Network error'))

    const { result } = renderHook(() => useAuth(), { wrapper })

    let loginResult: boolean = true
    await act(async () => {
      loginResult = await result.current.login('test@example.com', 'password123')
    })

    expect(loginResult).toBe(false)
    expect(result.current.error).toBeTruthy()
    expect(result.current.isAuthenticated).toBe(false)
  })

  it('registers user successfully', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper })

    let registerResult: boolean = false
    await act(async () => {
      registerResult = await result.current.register('test@example.com', 'password123', 'Test User')
    })

    expect(registerResult).toBe(true)
    expect(result.current.user).toBeDefined()
    expect(result.current.user?.email).toBe('test@example.com')
    expect(result.current.isAuthenticated).toBe(true)
  })

  it('handles register failure', async () => {
    mockAuthClient.register.mockRejectedValueOnce(new Error('Registration failed'))

    const { result } = renderHook(() => useAuth(), { wrapper })

    let registerResult: boolean = true
    await act(async () => {
      registerResult = await result.current.register('test@example.com', 'password123')
    })

    expect(registerResult).toBe(false)
    expect(result.current.user).toBeNull()
    expect(result.current.isAuthenticated).toBe(false)
    expect(result.current.error).toBeTruthy()
  })

  it('logs out user successfully', async () => {
    const { result } = renderHook(() => useAuth(), { wrapper })

    // First login
    await act(async () => {
      await result.current.login('test@example.com', 'password123')
    })

    expect(result.current.isAuthenticated).toBe(true)

    // Then logout
    act(() => {
      result.current.logout()
    })

    expect(result.current.user).toBeNull()
    expect(result.current.isAuthenticated).toBe(false)
    expect(result.current.error).toBeNull()
  })

  it('refreshes user profile successfully', async () => {
    const updatedUser = { ...mockUser, full_name: 'Updated Name' }
    mockAuthClient.getProfile.mockResolvedValueOnce(updatedUser)

    const { result } = renderHook(() => useAuth(), { wrapper })

    // First login
    await act(async () => {
      await result.current.login('test@example.com', 'password123')
    })

    // Then refresh
    await act(async () => {
      await result.current.refreshUser()
    })

    expect(result.current.user?.full_name).toBe('Updated Name')
  })

  it('handles refreshUser error', async () => {
    mockAuthClient.getProfile.mockRejectedValueOnce(new Error('Refresh failed'))

    const { result } = renderHook(() => useAuth(), { wrapper })

    // First login
    await act(async () => {
      await result.current.login('test@example.com', 'password123')
    })

    // Then refresh (should throw)
    await expect(async () => {
      await act(async () => {
        await result.current.refreshUser()
      })
    }).rejects.toThrow()
  })

  it('initializes auth with valid token', async () => {
    // Reset mocks
    vi.clearAllMocks()

    // Set up valid token scenario
    mockAuthClient.getAuthToken.mockReturnValue('valid-token')
    mockAuthClient.isTokenExpired.mockReturnValue(false)
    mockAuthClient.getProfile.mockResolvedValue(mockUser)

    const { result } = renderHook(() => useAuth(), { wrapper })

    // Wait for initialization
    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.user).toBeDefined()
    expect(result.current.isAuthenticated).toBe(true)
  })

  it('clears invalid token on initialization', async () => {
    // Reset mocks
    vi.clearAllMocks()

    // Set up invalid token scenario
    mockAuthClient.getAuthToken.mockReturnValue('invalid-token')
    mockAuthClient.isTokenExpired.mockReturnValue(false)
    mockAuthClient.getProfile.mockRejectedValue(new Error('Unauthorized'))

    const { result } = renderHook(() => useAuth(), { wrapper })

    // Wait for initialization
    await waitFor(() => {
      expect(result.current.loading).toBe(false)
    })

    expect(result.current.user).toBeNull()
    expect(result.current.isAuthenticated).toBe(false)
    expect(mockAuthClient.removeAuthToken).toHaveBeenCalled()
  })

  it('throws error when useAuth is used outside provider', () => {
    // Mock console.error to suppress error output
    const consoleErrorSpy = vi.spyOn(console, 'error').mockImplementation(() => {});

    expect(() => {
      renderHook(() => useAuth())
    }).toThrow('useAuth must be used within an AuthProvider')

    consoleErrorSpy.mockRestore()
  })

  it.todo('sets loading state correctly during operations', async () => {
    let resolveLogin: (value: unknown) => void
    const loginPromise = new Promise(resolve => {
      resolveLogin = resolve
    })
    
    mockFetch.mockReturnValueOnce(loginPromise)
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    // Start login
    act(() => {
      result.current.login({
        email: 'test@example.com',
        password: 'password123',
      })
    })
    
    expect(result.current.loading).toBe(true)
    
    // Resolve login
    await act(async () => {
      resolveLogin!({
        ok: true,
        json: async () => ({ token: 'token', user: mockUser }),
      })
      await loginPromise
    })
    
    expect(result.current.loading).toBe(false)
  })

  it.todo('provides auth token for API requests', () => {
    window.localStorage.setItem('auth_token', 'test-token')
    
    const { result } = renderHook(() => useAuth(), { wrapper })
    
    expect(result.current.token).toBe('test-token')
  })

  it.todo('throws error when used outside provider', () => {
    expect(() => {
      renderHook(() => useAuth())
    }).toThrow('useAuth must be used within an AuthProvider')
  })
})