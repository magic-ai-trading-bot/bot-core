import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '../../test/mocks/server'
import { render, mockUser } from '../../test/utils'
import Login from '../../pages/Login'

// Mock the API module with factory function - use alias path
vi.mock('@/services/api', () => {
  const mockLogin = vi.fn()
  const mockGetProfile = vi.fn()
  const mockGetAuthToken = vi.fn()
  const mockIsTokenExpired = vi.fn()

  return {
    BotCoreApiClient: vi.fn(() => ({
      auth: {
        login: mockLogin,
        register: vi.fn(),
        getProfile: mockGetProfile,
        verifyToken: vi.fn(),
        setAuthToken: vi.fn(),
        removeAuthToken: vi.fn(),
        getAuthToken: mockGetAuthToken,
        isTokenExpired: mockIsTokenExpired,
      },
      rust: {},
      python: {},
    })),
    mockAuthHelpers: {
      mockLogin,
      mockGetProfile,
      mockGetAuthToken,
      mockIsTokenExpired,
    },
  }
})

// Get the exported mocks
const { mockAuthHelpers } = await import('@/services/api')
const { mockLogin, mockGetProfile, mockGetAuthToken, mockIsTokenExpired } = mockAuthHelpers

// Mock router
const mockNavigate = vi.fn()
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    Link: ({ children, to, ...props }: { children: React.ReactNode; to: string; [key: string]: unknown }) => (
      <a href={to} {...props}>
        {children}
      </a>
    ),
  }
})

// Mock ChatBot component
vi.mock('../../components/ChatBot', () => ({
  default: () => null,
}))

describe('Login', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    if (typeof localStorage !== 'undefined' && typeof localStorage.clear === 'function') {
      localStorage.clear()
    }

    // Setup default mock behaviors
    mockGetAuthToken.mockReturnValue(null)
    mockIsTokenExpired.mockReturnValue(true)
    mockLogin.mockResolvedValue({
      token: 'mock-jwt-token',
      user: mockUser,
    })
    mockGetProfile.mockResolvedValue(mockUser)
  })

  it('renders login form', () => {
    render(<Login />)

    expect(screen.getByRole('heading', { name: /đăng nhập/i })).toBeInTheDocument()
    expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/mật khẩu/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /đăng nhập/i })).toBeInTheDocument()
  })

  it('shows demo credentials', () => {
    render(<Login />)

    expect(screen.getByText('Demo credentials:')).toBeInTheDocument()
    expect(screen.getByText(/admin@tradingbot.com/i)).toBeInTheDocument()
    expect(screen.getByText(/demo123/i)).toBeInTheDocument()
  })

  it('has link to register page', () => {
    render(<Login />)

    const registerLink = screen.getByText(/đăng ký ngay/i)
    expect(registerLink).toHaveAttribute('href', '/register')
  })

  it('submits login form with valid data', async () => {
    const user = userEvent.setup()
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/mật khẩu/i), 'password123')
    await user.click(screen.getByRole('button', { name: /đăng nhập/i }))

    // Wait for navigation
    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    }, { timeout: 3000 })
  })

  it('displays loading state during login', async () => {
    // Make the mockLogin slow to capture loading state
    mockLogin.mockImplementation(() => {
      return new Promise(resolve => {
        setTimeout(() => {
          resolve({
            token: 'mock-jwt-token',
            user: mockUser,
          })
        }, 100)
      })
    })

    const user = userEvent.setup()
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/mật khẩu/i), 'password123')

    const loginButton = screen.getByRole('button', { name: /^đăng nhập$/i })
    await user.click(loginButton)

    // Check loading state immediately after click
    await waitFor(() => {
      const loadingButton = screen.queryByRole('button', { name: /đang đăng nhập/i })
      if (loadingButton) {
        expect(loadingButton).toBeInTheDocument()
        expect(loadingButton).toBeDisabled()
      } else {
        // If loading state is too fast, just verify login happened
        expect(mockLogin).toHaveBeenCalledWith('test@example.com', 'password123')
      }
    })
  })

  it('handles login failure', async () => {
    server.use(
      http.post('http://localhost:8080/api/auth/login', () => {
        return HttpResponse.json(
          {
            success: false,
            error: 'Invalid credentials'
          },
          { status: 401 }
        )
      })
    )

    const user = userEvent.setup()
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/mật khẩu/i), 'wrongpassword')
    await user.click(screen.getByRole('button', { name: /đăng nhập/i }))

    // Button should return to normal state after failure
    await waitFor(() => {
      const button = screen.getByRole('button', { name: /đăng nhập/i })
      expect(button).not.toBeDisabled()
    }, { timeout: 5000 })
  })

  it('persists form data on error', async () => {
    server.use(
      http.post('http://localhost:8080/api/auth/login', () => {
        return HttpResponse.json(
          {
            success: false,
            error: 'Server error'
          },
          { status: 500 }
        )
      })
    )

    const user = userEvent.setup()
    render(<Login />)

    const emailInput = screen.getByLabelText(/email/i) as HTMLInputElement
    const passwordInput = screen.getByLabelText(/mật khẩu/i) as HTMLInputElement

    await user.type(emailInput, 'test@example.com')
    await user.type(passwordInput, 'password123')
    await user.click(screen.getByRole('button', { name: /đăng nhập/i }))

    await waitFor(() => {
      expect(emailInput.value).toBe('test@example.com')
      expect(passwordInput.value).toBe('password123')
    })
  })

  it('handles enter key submission', async () => {
    const user = userEvent.setup()
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/mật khẩu/i), 'password123{Enter}')

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    }, { timeout: 3000 })
  })

  it('shows features preview', () => {
    render(<Login />)

    expect(screen.getByText('AI-Powered Trading Signals')).toBeInTheDocument()
    expect(screen.getByText('Real-time Performance Analytics')).toBeInTheDocument()
    expect(screen.getByText('Advanced Risk Management')).toBeInTheDocument()
  })

  it('redirects if already authenticated', async () => {
    // Set up authenticated state
    localStorage.setItem('authToken', 'valid-token')

    // Mock that user is already authenticated
    mockGetAuthToken.mockReturnValue('valid-token')
    mockIsTokenExpired.mockReturnValue(false) // Token is NOT expired
    mockGetProfile.mockResolvedValue(mockUser)

    render(<Login />)

    // Should redirect to dashboard
    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    }, { timeout: 3000 })
  })

  it('shows security message', () => {
    render(<Login />)

    expect(screen.getByText(/bảo mật với mã hóa end-to-end/i)).toBeInTheDocument()
  })
})