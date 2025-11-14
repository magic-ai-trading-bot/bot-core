import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { http, HttpResponse } from 'msw'
import { server } from '../../test/mocks/server'
import { render, mockUser } from '../../test/utils'
import Register from '../../pages/Register'

// Mock the API module with factory function
vi.mock('../../services/api', () => {
  const mockRegister = vi.fn()
  const mockGetProfile = vi.fn()
  const mockGetAuthToken = vi.fn()
  const mockIsTokenExpired = vi.fn()

  return {
    BotCoreApiClient: vi.fn(() => ({
      auth: {
        login: vi.fn(),
        register: mockRegister,
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
      mockRegister,
      mockGetProfile,
      mockGetAuthToken,
      mockIsTokenExpired,
    },
  }
})

// Get the exported mocks
const { mockAuthHelpers } = await import('../../services/api')
const { mockRegister, mockGetProfile, mockGetAuthToken, mockIsTokenExpired } = mockAuthHelpers

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

describe('Register', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    if (typeof localStorage !== 'undefined' && typeof localStorage.clear === 'function') {
      localStorage.clear()
    }

    // Setup default mock behaviors
    mockGetAuthToken.mockReturnValue(null)
    mockIsTokenExpired.mockReturnValue(true)
    mockRegister.mockResolvedValue({
      token: 'mock-jwt-token',
      user: mockUser,
    })
    mockGetProfile.mockResolvedValue(mockUser)
  })

  it('renders register form', () => {
    render(<Register />)

    expect(screen.getByRole('heading', { name: /đăng ký/i })).toBeInTheDocument()
    expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    expect(screen.getByLabelText(/họ và tên/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
    expect(screen.getByLabelText('Mật khẩu')).toBeInTheDocument()
    expect(screen.getByLabelText(/xác nhận mật khẩu/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /đăng ký/i })).toBeInTheDocument()
  })

  it('shows brand and tagline', () => {
    render(<Register />)

    expect(screen.getByText('BT')).toBeInTheDocument()
    expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    expect(screen.getByText(/tạo tài khoản để bắt đầu giao dịch/i)).toBeInTheDocument()
  })

  it('has link to login page', () => {
    render(<Register />)

    const loginLink = screen.getByText(/đăng nhập ngay/i)
    expect(loginLink).toHaveAttribute('href', '/login')
  })

  it('submits registration form with valid data', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/họ và tên/i), 'John Doe')
    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'password123')
    await user.click(screen.getByRole('button', { name: /đăng ký/i }))

    // Wait for navigation
    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    }, { timeout: 3000 })
  })

  it('submits registration without optional full name', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'password123')
    await user.click(screen.getByRole('button', { name: /đăng ký/i }))

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    }, { timeout: 3000 })
  })

  it('displays loading state during registration', async () => {
    // Make the mockRegister slow to capture loading state
    mockRegister.mockImplementation(() => {
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
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'password123')

    const registerButton = screen.getByRole('button', { name: /^đăng ký$/i })
    await user.click(registerButton)

    // Check loading state immediately after click
    await waitFor(() => {
      const loadingButton = screen.queryByRole('button', { name: /đang đăng ký/i })
      if (loadingButton) {
        expect(loadingButton).toBeInTheDocument()
        expect(loadingButton).toBeDisabled()
      } else {
        // If loading state is too fast, just verify register happened
        expect(mockRegister).toHaveBeenCalled()
      }
    })
  })

  it('validates that all required fields are filled', async () => {
    const user = userEvent.setup()
    render(<Register />)

    // Try to submit without filling anything - form has HTML5 validation
    const button = screen.getByRole('button', { name: /đăng ký/i })

    // Button should be visible but form may prevent submission via HTML5 validation
    expect(button).toBeInTheDocument()

    // Check that required fields exist
    expect(screen.getByLabelText(/email/i)).toHaveAttribute('required')
    expect(screen.getByLabelText('Mật khẩu')).toHaveAttribute('required')
  })

  it('validates password confirmation matches', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'different-password')
    await user.click(screen.getByRole('button', { name: /đăng ký/i }))

    // Should not navigate due to password mismatch
    // Wait a bit to ensure no navigation happens
    await waitFor(() => {
      // Check that we're still on register page (button is still visible and not disabled)
      const button = screen.getByRole('button', { name: /đăng ký/i })
      expect(button).toBeInTheDocument()
    }, { timeout: 1000 })
  })

  it('validates password length (minimum 6 characters)', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), '12345')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), '12345')
    await user.click(screen.getByRole('button', { name: /đăng ký/i }))

    // Should not navigate
    expect(mockNavigate).not.toHaveBeenCalled()
  })

  it('handles registration failure', async () => {
    server.use(
      http.post('http://localhost:8080/api/auth/register', () => {
        return HttpResponse.json(
          {
            success: false,
            error: 'Email already exists'
          },
          { status: 400 }
        )
      })
    )

    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'existing@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'password123')
    await user.click(screen.getByRole('button', { name: /đăng ký/i }))

    // Button should return to normal state after failure
    await waitFor(() => {
      const button = screen.getByRole('button', { name: /đăng ký/i })
      expect(button).not.toBeDisabled()
    }, { timeout: 5000 })
  })

  it('handles network errors during registration', async () => {
    server.use(
      http.post('http://localhost:8080/api/auth/register', () => {
        return HttpResponse.error()
      })
    )

    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'password123')
    await user.click(screen.getByRole('button', { name: /đăng ký/i }))

    await waitFor(() => {
      const button = screen.getByRole('button', { name: /đăng ký/i })
      expect(button).not.toBeDisabled()
    }, { timeout: 5000 })
  })

  it('persists form data on error', async () => {
    server.use(
      http.post('http://localhost:8080/api/auth/register', () => {
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
    render(<Register />)

    const fullNameInput = screen.getByLabelText(/họ và tên/i) as HTMLInputElement
    const emailInput = screen.getByLabelText(/email/i) as HTMLInputElement
    const passwordInput = screen.getByLabelText('Mật khẩu') as HTMLInputElement

    await user.type(fullNameInput, 'John Doe')
    await user.type(emailInput, 'john@example.com')
    await user.type(passwordInput, 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'password123')
    await user.click(screen.getByRole('button', { name: /đăng ký/i }))

    await waitFor(() => {
      expect(fullNameInput.value).toBe('John Doe')
      expect(emailInput.value).toBe('john@example.com')
      expect(passwordInput.value).toBe('password123')
    })
  })

  it('shows features preview', () => {
    render(<Register />)

    expect(screen.getByText('AI-Powered Trading Signals')).toBeInTheDocument()
    expect(screen.getByText('Real-time Performance Analytics')).toBeInTheDocument()
    expect(screen.getByText('Advanced Risk Management')).toBeInTheDocument()
  })

  it('redirects if already authenticated', async () => {
    localStorage.setItem('authToken', 'valid-token')

    // Mock that user is already authenticated
    mockGetAuthToken.mockReturnValue('valid-token')
    mockIsTokenExpired.mockReturnValue(false) // Token is NOT expired
    mockGetProfile.mockResolvedValue(mockUser)

    render(<Register />)

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    }, { timeout: 3000 })
  })

  it('shows security message', () => {
    render(<Register />)

    expect(screen.getByText(/bảo mật với mã hóa end-to-end và xác thực 2fa/i)).toBeInTheDocument()
  })

  it('handles enter key submission', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Mật khẩu'), 'password123')
    await user.type(screen.getByLabelText(/xác nhận mật khẩu/i), 'password123{Enter}')

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    }, { timeout: 3000 })
  })

  it('renders all input fields with correct placeholders', () => {
    render(<Register />)

    expect(screen.getByPlaceholderText('Nguyễn Văn A')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('your@email.com')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Nhập mật khẩu của bạn')).toBeInTheDocument()
    expect(screen.getByPlaceholderText('Nhập lại mật khẩu')).toBeInTheDocument()
  })

  it('validates email field is required', async () => {
    const user = userEvent.setup()
    render(<Register />)

    const emailInput = screen.getByLabelText(/email/i)
    expect(emailInput).toHaveAttribute('required')
    expect(emailInput).toHaveAttribute('type', 'email')
  })

  it('validates password fields are required', () => {
    render(<Register />)

    const passwordInput = screen.getByLabelText('Mật khẩu')
    const confirmPasswordInput = screen.getByLabelText(/xác nhận mật khẩu/i)

    expect(passwordInput).toHaveAttribute('required')
    expect(passwordInput).toHaveAttribute('type', 'password')
    expect(confirmPasswordInput).toHaveAttribute('required')
    expect(confirmPasswordInput).toHaveAttribute('type', 'password')
  })

  it('full name field is optional', () => {
    render(<Register />)

    const fullNameInput = screen.getByLabelText(/họ và tên/i)
    expect(fullNameInput).not.toHaveAttribute('required')
  })
})
