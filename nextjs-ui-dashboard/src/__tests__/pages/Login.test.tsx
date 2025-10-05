import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
import Login from '../../pages/Login'
import { http, HttpResponse } from 'msw'
import { server } from '../../test/mocks/server'

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
    localStorage.clear()
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
    // Make the API slow
    server.use(
      http.post('http://localhost:8080/api/auth/login', async () => {
        await new Promise(resolve => setTimeout(resolve, 100))
        return HttpResponse.json({
          success: true,
          data: {
            token: 'mock-jwt-token',
            user: { id: 'user123', email: 'test@example.com', full_name: 'Test User' },
          },
        })
      })
    )

    const user = userEvent.setup()
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/mật khẩu/i), 'password123')
    await user.click(screen.getByRole('button', { name: /đăng nhập/i }))

    // Check loading state
    expect(screen.getByRole('button', { name: /đang đăng nhập/i })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /đang đăng nhập/i })).toBeDisabled()
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

    server.use(
      http.get('http://localhost:8080/api/auth/profile', () => {
        return HttpResponse.json({
          success: true,
          data: {
            id: 'user123',
            email: 'test@example.com',
            full_name: 'Test User',
            created_at: '2024-01-01T00:00:00Z',
            roles: ['user'],
          },
        })
      })
    )

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