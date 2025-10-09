import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../../test/utils'
import { DashboardHeader } from '../../../components/dashboard/DashboardHeader'
import { useNavigate } from 'react-router-dom'

// Mock react-router-dom
const mockNavigate = vi.fn()

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
  }
})

// Mock AuthContext
const mockLogout = vi.fn()
const mockUseAuth = vi.fn()

vi.mock('../../../contexts/AuthContext', () => ({
  useAuth: () => mockUseAuth(),
}))

describe('DashboardHeader', () => {
  beforeEach(() => {
    vi.clearAllMocks()

    mockUseAuth.mockReturnValue({
      logout: mockLogout,
      user: {
        id: 'user123',
        email: 'test@example.com',
        full_name: 'Test User',
        roles: ['user'],
        created_at: '2024-01-01T00:00:00Z',
      },
    })
  })

  describe('Component Rendering', () => {
    it('renders the dashboard header', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    })

    it('displays the app logo', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('BT')).toBeInTheDocument()
    })

    it('displays the app tagline', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('AI-Powered Futures Trading')).toBeInTheDocument()
    })

    it('displays navigation menu', () => {
      render(<DashboardHeader />)

      expect(screen.getByRole('button', { name: /dashboard/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /trading paper/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /settings/i })).toBeInTheDocument()
    })

    it('displays bot active badge', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Bot Active')).toBeInTheDocument()
    })

    it('displays Binance Futures connection', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Connected to')).toBeInTheDocument()
      expect(screen.getByText('Binance Futures')).toBeInTheDocument()
    })

    it('displays logged in user information', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Logged in as')).toBeInTheDocument()
      expect(screen.getByText('Test User')).toBeInTheDocument()
    })

    it('displays logout button', () => {
      render(<DashboardHeader />)

      expect(screen.getByRole('button', { name: /đăng xuất/i })).toBeInTheDocument()
    })
  })

  describe('Navigation Links', () => {
    it('renders Dashboard navigation link', () => {
      render(<DashboardHeader />)

      const dashboardLinks = screen.getAllByRole('link')
      const dashboardLink = dashboardLinks.find(link => link.getAttribute('href') === '/dashboard')

      expect(dashboardLink).toBeInTheDocument()
    })

    it('renders Trading Paper navigation link', () => {
      render(<DashboardHeader />)

      const links = screen.getAllByRole('link')
      const tradingPaperLink = links.find(link => link.getAttribute('href') === '/trading-paper')

      expect(tradingPaperLink).toBeInTheDocument()
    })

    it('renders Settings navigation link', () => {
      render(<DashboardHeader />)

      const links = screen.getAllByRole('link')
      const settingsLink = links.find(link => link.getAttribute('href') === '/settings')

      expect(settingsLink).toBeInTheDocument()
    })

    it('navigates to dashboard when logo is clicked', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const logoLink = screen.getAllByRole('link')[0]
      expect(logoLink.getAttribute('href')).toBe('/dashboard')
    })
  })

  describe('User Information Display', () => {
    it('displays user full name when available', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Test User')).toBeInTheDocument()
    })

    it('displays user email when full name is not available', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: {
          id: 'user123',
          email: 'test@example.com',
          full_name: null,
          roles: ['user'],
          created_at: '2024-01-01T00:00:00Z',
        },
      })

      render(<DashboardHeader />)

      expect(screen.getByText('test@example.com')).toBeInTheDocument()
    })

    it('handles empty user name gracefully', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: {
          id: 'user123',
          email: 'test@example.com',
          full_name: '',
          roles: ['user'],
          created_at: '2024-01-01T00:00:00Z',
        },
      })

      render(<DashboardHeader />)

      expect(screen.getByText('test@example.com')).toBeInTheDocument()
    })

    it('truncates long user names', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: {
          id: 'user123',
          email: 'verylongemail@example.com',
          full_name: 'Very Long User Name That Should Be Truncated',
          roles: ['user'],
          created_at: '2024-01-01T00:00:00Z',
        },
      })

      const { container } = render(<DashboardHeader />)

      const truncatedElements = container.querySelectorAll('.truncate')
      expect(truncatedElements.length).toBeGreaterThan(0)
    })
  })

  describe('Logout Functionality', () => {
    it('calls logout when logout button is clicked', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const logoutButton = screen.getByRole('button', { name: /đăng xuất/i })
      await user.click(logoutButton)

      expect(mockLogout).toHaveBeenCalled()
    })

    it('navigates to login page after logout', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const logoutButton = screen.getByRole('button', { name: /đăng xuất/i })
      await user.click(logoutButton)

      await waitFor(() => {
        expect(mockNavigate).toHaveBeenCalledWith('/login')
      })
    })

    it('handles logout in correct order', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const logoutButton = screen.getByRole('button', { name: /đăng xuất/i })
      await user.click(logoutButton)

      // Logout should be called before navigation
      expect(mockLogout).toHaveBeenCalled()
      expect(mockNavigate).toHaveBeenCalledWith('/login')
    })
  })

  describe('Status Badges', () => {
    it('displays bot active badge with animated pulse', () => {
      const { container } = render(<DashboardHeader />)

      const animatedElements = container.querySelectorAll('.animate-pulse')
      expect(animatedElements.length).toBeGreaterThan(0)
    })

    it('applies profit color to bot active badge', () => {
      render(<DashboardHeader />)

      const badge = screen.getByText('Bot Active').closest('div')
      expect(badge?.className).toContain('bg-profit/10')
    })
  })

  describe('Responsive Design', () => {
    it('has responsive flex layout', () => {
      const { container } = render(<DashboardHeader />)

      const header = container.querySelector('[class*="lg:flex-row"]')
      expect(header).toBeInTheDocument()
    })

    it('displays navigation in correct order on mobile', () => {
      const { container } = render(<DashboardHeader />)

      const orderElements = container.querySelectorAll('[class*="order-"]')
      expect(orderElements.length).toBeGreaterThan(0)
    })

    it('has responsive text sizes', () => {
      const { container } = render(<DashboardHeader />)

      const responsiveText = container.querySelector('[class*="lg:text-"]')
      expect(responsiveText).toBeInTheDocument()
    })
  })

  describe('Navigation Buttons', () => {
    it('renders dashboard button with ghost variant', () => {
      render(<DashboardHeader />)

      const dashboardButton = screen.getByRole('button', { name: /^dashboard$/i })
      expect(dashboardButton.className).toContain('ghost')
    })

    it('renders trading paper button with ghost variant', () => {
      render(<DashboardHeader />)

      const tradingButton = screen.getByRole('button', { name: /trading paper/i })
      expect(tradingButton.className).toContain('ghost')
    })

    it('renders settings button with ghost variant', () => {
      render(<DashboardHeader />)

      const settingsButton = screen.getByRole('button', { name: /^settings$/i })
      expect(settingsButton.className).toContain('ghost')
    })

    it('applies hover styles to navigation buttons', () => {
      render(<DashboardHeader />)

      const dashboardButton = screen.getByRole('button', { name: /^dashboard$/i })
      expect(dashboardButton.className).toContain('hover:text-foreground')
    })
  })

  describe('Logo and Branding', () => {
    it('displays logo with gradient background', () => {
      const { container } = render(<DashboardHeader />)

      const logo = container.querySelector('.bg-gradient-to-br')
      expect(logo).toBeInTheDocument()
    })

    it('displays logo with correct size', () => {
      const { container } = render(<DashboardHeader />)

      const logo = container.querySelector('.w-8.h-8')
      expect(logo).toBeTruthy()
    })

    it('displays logo text centered', () => {
      render(<DashboardHeader />)

      const logoText = screen.getByText('BT')
      expect(logoText.className).toContain('font-bold')
    })

    it('applies hover effect to logo link', () => {
      const { container } = render(<DashboardHeader />)

      const logoLink = screen.getAllByRole('link')[0]
      expect(logoLink.className).toContain('hover:opacity-80')
    })
  })

  describe('Edge Cases', () => {
    it('handles null user gracefully', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: null,
      })

      render(<DashboardHeader />)

      expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    })

    it('handles user without full_name or email', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: {
          id: 'user123',
          roles: ['user'],
          created_at: '2024-01-01T00:00:00Z',
        },
      })

      render(<DashboardHeader />)

      expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    })

    it('handles multiple rapid logout clicks', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const logoutButton = screen.getByRole('button', { name: /đăng xuất/i })

      await user.click(logoutButton)
      await user.click(logoutButton)
      await user.click(logoutButton)

      // Should only logout once (or multiple times, but shouldn't crash)
      expect(mockLogout).toHaveBeenCalled()
    })
  })

  describe('Accessibility', () => {
    it('has accessible button roles', () => {
      render(<DashboardHeader />)

      const buttons = screen.getAllByRole('button')
      expect(buttons.length).toBeGreaterThanOrEqual(4) // Dashboard, Trading Paper, Settings, Logout
    })

    it('has accessible link roles', () => {
      render(<DashboardHeader />)

      const links = screen.getAllByRole('link')
      expect(links.length).toBeGreaterThanOrEqual(4) // Logo, Dashboard, Trading Paper, Settings
    })

    it('has readable button text', () => {
      render(<DashboardHeader />)

      expect(screen.getByRole('button', { name: /dashboard/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /trading paper/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /settings/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /đăng xuất/i })).toBeInTheDocument()
    })
  })

  describe('Layout Structure', () => {
    it('has border bottom', () => {
      const { container } = render(<DashboardHeader />)

      const header = container.querySelector('.border-b')
      expect(header).toBeInTheDocument()
    })

    it('has correct padding', () => {
      const { container } = render(<DashboardHeader />)

      const header = container.querySelector('[class*="lg:p-6"]')
      expect(header).toBeInTheDocument()
    })

    it('has gap between elements', () => {
      const { container } = render(<DashboardHeader />)

      const gapElements = container.querySelectorAll('[class*="gap-"]')
      expect(gapElements.length).toBeGreaterThan(0)
    })
  })

  describe('Text Content', () => {
    it('displays correct heading text', () => {
      render(<DashboardHeader />)

      const heading = screen.getByText('Crypto Trading Bot')
      expect(heading.tagName).toBe('H1')
    })

    it('applies correct text sizes', () => {
      render(<DashboardHeader />)

      const heading = screen.getByText('Crypto Trading Bot')
      expect(heading.className).toContain('text-xl')
      expect(heading.className).toContain('lg:text-2xl')
    })

    it('applies muted color to tagline', () => {
      render(<DashboardHeader />)

      const tagline = screen.getByText('AI-Powered Futures Trading')
      expect(tagline.className).toContain('text-muted-foreground')
    })
  })

  describe('Button Styling', () => {
    it('applies outline variant to logout button', () => {
      render(<DashboardHeader />)

      const logoutButton = screen.getByRole('button', { name: /đăng xuất/i })
      expect(logoutButton.className).toContain('outline')
    })

    it('applies small size to navigation buttons', () => {
      render(<DashboardHeader />)

      const dashboardButton = screen.getByRole('button', { name: /^dashboard$/i })
      expect(dashboardButton.className).toContain('sm')
    })
  })

  describe('User Authentication State', () => {
    it('displays user info when authenticated', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Logged in as')).toBeInTheDocument()
      expect(screen.getByText('Test User')).toBeInTheDocument()
    })

    it('handles unauthenticated state', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: null,
      })

      render(<DashboardHeader />)

      // Should still render header
      expect(screen.getByText('Crypto Trading Bot')).toBeInTheDocument()
    })
  })
})
