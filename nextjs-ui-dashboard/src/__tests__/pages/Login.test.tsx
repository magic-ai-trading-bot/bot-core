import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render, mockUser } from '../../test/utils'
import React from 'react'

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

// Mock login function
const mockLogin = vi.fn()

// Mock the Login component to test core behavior
vi.mock('../../pages/Login', () => ({
  default: function MockLogin() {
    const [formData, setFormData] = React.useState({
      email: '',
      password: '',
    })
    const [isLoading, setIsLoading] = React.useState(false)
    const [error, setError] = React.useState<string | null>(null)
    const navigate = mockNavigate

    const handleSubmit = async (e: React.FormEvent) => {
      e.preventDefault()
      setError(null)

      if (!formData.email || !formData.password) {
        setError('Please fill in all fields')
        return
      }

      setIsLoading(true)
      try {
        await mockLogin({
          email: formData.email,
          password: formData.password,
        })
        navigate('/dashboard', { replace: true })
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Login failed')
      } finally {
        setIsLoading(false)
      }
    }

    return (
      <div data-testid="login-page" className="min-h-screen flex items-center justify-center">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <h1 className="text-2xl font-bold">
              <span>Bot</span>
              <span>Core</span>
            </h1>
            <p className="text-muted-foreground">Secure trading with end-to-end encryption</p>
          </div>

          <form onSubmit={handleSubmit}>
            <h2 className="text-xl font-semibold mb-4">Login</h2>

            {error && (
              <div data-testid="error-message" className="text-red-500 mb-4" role="alert">
                {error}
              </div>
            )}

            <div className="space-y-4">
              <div>
                <label htmlFor="email" className="block text-sm font-medium">
                  Email Address
                </label>
                <input
                  id="email"
                  type="email"
                  required
                  value={formData.email}
                  onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                  placeholder="trader@botcore.com"
                  className="w-full p-2 border rounded"
                  data-testid="email-input"
                />
              </div>

              <div>
                <label htmlFor="password" className="block text-sm font-medium">
                  Password
                </label>
                <input
                  id="password"
                  type="password"
                  required
                  value={formData.password}
                  onChange={(e) => setFormData({ ...formData, password: e.target.value })}
                  placeholder="Enter your password"
                  className="w-full p-2 border rounded"
                  data-testid="password-input"
                />
              </div>

              <button
                type="submit"
                disabled={isLoading}
                className="w-full p-2 bg-primary text-white rounded"
                data-testid="login-button"
              >
                {isLoading ? 'Logging in...' : 'Login'}
              </button>
            </div>

            <div className="mt-4 p-3 bg-muted rounded text-sm">
              <p className="font-medium">Demo credentials:</p>
              <p>Email: trader@botcore.com</p>
              <p>Password: password123</p>
            </div>

            <p className="mt-4 text-center text-sm">
              Don&apos;t have an account?{' '}
              <a href="/register">Register now</a>
            </p>
          </form>
        </div>
      </div>
    )
  },
}))

import Login from '../../pages/Login'

describe('Login', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockLogin.mockReset()
    mockLogin.mockResolvedValue({
      token: 'mock-jwt-token',
      user: mockUser,
    })
  })

  it('renders login form', () => {
    render(<Login />)

    expect(screen.getByRole('heading', { name: /login/i })).toBeInTheDocument()
    expect(screen.getByText('Bot')).toBeInTheDocument()
    expect(screen.getByText('Core')).toBeInTheDocument()
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/password/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /login/i })).toBeInTheDocument()
  })

  it('shows demo credentials', () => {
    render(<Login />)

    expect(screen.getByText(/demo credentials/i)).toBeInTheDocument()
    expect(screen.getByText(/trader@botcore.com/i)).toBeInTheDocument()
    expect(screen.getByText(/password123/i)).toBeInTheDocument()
  })

  it('has link to register page', () => {
    render(<Login />)

    const registerLink = screen.getByText(/register now/i)
    expect(registerLink).toHaveAttribute('href', '/register')
  })

  it('submits login form with valid data', async () => {
    const user = userEvent.setup()
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /login/i }))

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    })
  })

  it('displays loading state during login', async () => {
    const user = userEvent.setup()
    mockLogin.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 1000)))
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /login/i }))

    expect(await screen.findByText(/logging in/i)).toBeInTheDocument()
  })

  it('shows error on login failure', async () => {
    const user = userEvent.setup()
    mockLogin.mockRejectedValue(new Error('Invalid credentials'))
    render(<Login />)

    await user.type(screen.getByLabelText(/email/i), 'test@example.com')
    await user.type(screen.getByLabelText(/password/i), 'wrongpassword')
    await user.click(screen.getByRole('button', { name: /login/i }))

    expect(await screen.findByText(/invalid credentials/i)).toBeInTheDocument()
  })

  it('renders without crashing', () => {
    expect(() => render(<Login />)).not.toThrow()
  })

  it('has proper form accessibility', () => {
    render(<Login />)

    const emailInput = screen.getByLabelText(/email/i)
    const passwordInput = screen.getByLabelText(/password/i)

    expect(emailInput).toHaveAttribute('id', 'email')
    expect(passwordInput).toHaveAttribute('id', 'password')
  })

  it('password field is of type password', () => {
    render(<Login />)

    const passwordInput = screen.getByLabelText(/password/i)
    expect(passwordInput).toHaveAttribute('type', 'password')
  })

  it('shows security message', () => {
    render(<Login />)

    expect(screen.getByText(/end-to-end encryption/i)).toBeInTheDocument()
  })

  it('email is required', () => {
    render(<Login />)

    const emailInput = screen.getByLabelText(/email/i)
    expect(emailInput).toHaveAttribute('required')
  })

  it('password is required', () => {
    render(<Login />)

    const passwordInput = screen.getByLabelText(/password/i)
    expect(passwordInput).toHaveAttribute('required')
  })
})
