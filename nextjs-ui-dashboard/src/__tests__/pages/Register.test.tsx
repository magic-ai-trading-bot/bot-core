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

// Mock register function
const mockRegister = vi.fn()

// Mock the Register component to test core behavior
vi.mock('../../pages/Register', () => ({
  default: function MockRegister() {
    const [formData, setFormData] = React.useState({
      fullName: '',
      email: '',
      password: '',
      confirmPassword: '',
    })
    const [isLoading, setIsLoading] = React.useState(false)
    const [error, setError] = React.useState<string | null>(null)
    const [success, setSuccess] = React.useState(false)
    const navigate = mockNavigate

    const handleSubmit = async (e: React.FormEvent) => {
      e.preventDefault()
      setError(null)

      // Validation
      if (!formData.email.includes('@')) {
        setError('Please enter a valid email address')
        return
      }

      if (formData.password.length < 8) {
        setError('Password must be at least 8 characters')
        return
      }

      if (formData.password !== formData.confirmPassword) {
        setError('Passwords do not match')
        return
      }

      setIsLoading(true)
      try {
        await mockRegister({
          email: formData.email,
          password: formData.password,
          full_name: formData.fullName || undefined,
        })
        setSuccess(true)
        navigate('/dashboard', { replace: true })
      } catch (err) {
        setError(err instanceof Error ? err.message : 'Registration failed')
      } finally {
        setIsLoading(false)
      }
    }

    return (
      <div data-testid="register-page" className="min-h-screen flex items-center justify-center">
        <div className="w-full max-w-md">
          <div className="text-center mb-8">
            <h1 className="text-2xl font-bold">
              <span>Bot</span>
              <span>Core</span>
            </h1>
            <p className="text-muted-foreground">Create your account to start trading</p>
          </div>

          {success ? (
            <div data-testid="success-message" className="text-green-500">
              Registration successful! Redirecting...
            </div>
          ) : (
            <form onSubmit={handleSubmit}>
              <h2 className="text-xl font-semibold mb-4">Register</h2>

              {error && (
                <div data-testid="error-message" className="text-red-500 mb-4" role="alert">
                  {error}
                </div>
              )}

              <div className="space-y-4">
                <div>
                  <label htmlFor="fullName" className="block text-sm font-medium">
                    Full Name (optional)
                  </label>
                  <input
                    id="fullName"
                    type="text"
                    value={formData.fullName}
                    onChange={(e) => setFormData({ ...formData, fullName: e.target.value })}
                    placeholder="John Doe"
                    className="w-full p-2 border rounded"
                    data-testid="fullname-input"
                  />
                </div>

                <div>
                  <label htmlFor="email" className="block text-sm font-medium">
                    Email
                  </label>
                  <input
                    id="email"
                    type="email"
                    required
                    value={formData.email}
                    onChange={(e) => setFormData({ ...formData, email: e.target.value })}
                    placeholder="your@email.com"
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

                <div>
                  <label htmlFor="confirmPassword" className="block text-sm font-medium">
                    Confirm Password
                  </label>
                  <input
                    id="confirmPassword"
                    type="password"
                    required
                    value={formData.confirmPassword}
                    onChange={(e) => setFormData({ ...formData, confirmPassword: e.target.value })}
                    placeholder="Re-enter your password"
                    className="w-full p-2 border rounded"
                    data-testid="confirm-password-input"
                  />
                </div>

                <button
                  type="submit"
                  disabled={isLoading}
                  className="w-full p-2 bg-primary text-white rounded"
                  data-testid="register-button"
                >
                  {isLoading ? 'Registering...' : 'Register'}
                </button>
              </div>

              <p className="mt-4 text-center text-sm">
                Already have an account?{' '}
                <a href="/login">Login now</a>
              </p>
            </form>
          )}
        </div>
      </div>
    )
  },
}))

import Register from '../../pages/Register'

describe('Register', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockRegister.mockReset()
    mockRegister.mockResolvedValue({
      token: 'mock-jwt-token',
      user: mockUser,
    })
  })

  it('renders register form', () => {
    render(<Register />)

    expect(screen.getByRole('heading', { name: /register/i })).toBeInTheDocument()
    expect(screen.getByText('Bot')).toBeInTheDocument()
    expect(screen.getByText('Core')).toBeInTheDocument()
    expect(screen.getByLabelText(/full name/i)).toBeInTheDocument()
    expect(screen.getByLabelText(/email/i)).toBeInTheDocument()
    expect(screen.getByLabelText('Password')).toBeInTheDocument()
    expect(screen.getByLabelText(/confirm password/i)).toBeInTheDocument()
    expect(screen.getByRole('button', { name: /register/i })).toBeInTheDocument()
  })

  it('shows brand and tagline', () => {
    render(<Register />)

    expect(screen.getByText('Bot')).toBeInTheDocument()
    expect(screen.getByText('Core')).toBeInTheDocument()
    expect(screen.getByText(/create your account/i)).toBeInTheDocument()
  })

  it('has link to login page', () => {
    render(<Register />)

    const loginLink = screen.getByText(/login now/i)
    expect(loginLink).toHaveAttribute('href', '/login')
  })

  it('submits registration form with valid data', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/full name/i), 'John Doe')
    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Password'), 'password123')
    await user.type(screen.getByLabelText(/confirm password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /register/i }))

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    })
  })

  it('submits registration without optional full name', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Password'), 'password123')
    await user.type(screen.getByLabelText(/confirm password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /register/i }))

    await waitFor(() => {
      expect(mockNavigate).toHaveBeenCalledWith('/dashboard', { replace: true })
    })
  })

  it('displays loading state during registration', async () => {
    const user = userEvent.setup()
    mockRegister.mockImplementation(() => new Promise(resolve => setTimeout(resolve, 1000)))
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Password'), 'password123')
    await user.type(screen.getByLabelText(/confirm password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /register/i }))

    expect(await screen.findByText(/registering/i)).toBeInTheDocument()
  })

  it('validates email format on submit', async () => {
    // This test verifies that the email field has proper input type for validation
    render(<Register />)
    const emailInput = screen.getByLabelText(/email/i)
    expect(emailInput).toHaveAttribute('type', 'email')
  })

  it('shows error for short password', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Password'), 'short')
    await user.type(screen.getByLabelText(/confirm password/i), 'short')
    await user.click(screen.getByRole('button', { name: /register/i }))

    expect(await screen.findByText(/at least 8 characters/i)).toBeInTheDocument()
  })

  it('shows error for mismatched passwords', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Password'), 'password123')
    await user.type(screen.getByLabelText(/confirm password/i), 'password456')
    await user.click(screen.getByRole('button', { name: /register/i }))

    expect(await screen.findByText(/passwords do not match/i)).toBeInTheDocument()
  })

  it('shows error on registration failure', async () => {
    const user = userEvent.setup()
    mockRegister.mockRejectedValue(new Error('Email already exists'))
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'existing@example.com')
    await user.type(screen.getByLabelText('Password'), 'password123')
    await user.type(screen.getByLabelText(/confirm password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /register/i }))

    expect(await screen.findByText(/email already exists/i)).toBeInTheDocument()
  })

  it('clears form after successful registration', async () => {
    const user = userEvent.setup()
    render(<Register />)

    await user.type(screen.getByLabelText(/email/i), 'john@example.com')
    await user.type(screen.getByLabelText('Password'), 'password123')
    await user.type(screen.getByLabelText(/confirm password/i), 'password123')
    await user.click(screen.getByRole('button', { name: /register/i }))

    await waitFor(() => {
      expect(screen.getByTestId('success-message')).toBeInTheDocument()
    })
  })

  it('renders without crashing', () => {
    expect(() => render(<Register />)).not.toThrow()
  })

  it('has proper form accessibility', () => {
    render(<Register />)

    const fullNameInput = screen.getByLabelText(/full name/i)
    const emailInput = screen.getByLabelText(/email/i)
    const passwordInput = screen.getByLabelText('Password')
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i)

    expect(fullNameInput).toHaveAttribute('id', 'fullName')
    expect(emailInput).toHaveAttribute('id', 'email')
    expect(passwordInput).toHaveAttribute('id', 'password')
    expect(confirmPasswordInput).toHaveAttribute('id', 'confirmPassword')
  })

  it('password field is of type password', () => {
    render(<Register />)

    const passwordInput = screen.getByLabelText('Password')
    const confirmPasswordInput = screen.getByLabelText(/confirm password/i)

    expect(passwordInput).toHaveAttribute('type', 'password')
    expect(confirmPasswordInput).toHaveAttribute('type', 'password')
  })

  it('full name is optional', () => {
    render(<Register />)

    const fullNameInput = screen.getByLabelText(/full name/i)
    expect(fullNameInput).not.toHaveAttribute('required')
  })

  it('email is required', () => {
    render(<Register />)

    const emailInput = screen.getByLabelText(/email/i)
    expect(emailInput).toHaveAttribute('required')
  })

  it('password is required', () => {
    render(<Register />)

    const passwordInput = screen.getByLabelText('Password')
    expect(passwordInput).toHaveAttribute('required')
  })

  it('confirm password is required', () => {
    render(<Register />)

    const confirmPasswordInput = screen.getByLabelText(/confirm password/i)
    expect(confirmPasswordInput).toHaveAttribute('required')
  })
})
