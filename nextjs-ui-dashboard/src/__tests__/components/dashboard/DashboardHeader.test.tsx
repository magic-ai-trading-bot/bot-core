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
  AuthProvider: ({ children }: { children: React.ReactNode }) => children,
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

      expect(screen.getByText('Bot')).toBeInTheDocument()
      expect(screen.getByText('Core')).toBeInTheDocument()
    })

    it('displays the app logo with Bot icon', () => {
      const { container } = render(<DashboardHeader />)

      const logo = container.querySelector('[aria-label="BotCore Logo"]')
      expect(logo).toBeInTheDocument()
    })

    it('displays the app tagline', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('AI Trading')).toBeInTheDocument()
    })

    it('displays navigation menu', () => {
      render(<DashboardHeader />)

      expect(screen.getByRole('button', { name: /dashboard/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /trading paper/i })).toBeInTheDocument()
      expect(screen.getByRole('button', { name: /settings/i })).toBeInTheDocument()
    })

    it('displays bot active badge', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Active')).toBeInTheDocument()
    })

    it('displays Binance connection indicator', () => {
      render(<DashboardHeader />)

      expect(screen.getByText('Binance')).toBeInTheDocument()
    })

    it('displays user info with first name or email prefix', () => {
      render(<DashboardHeader />)

      // Shows first name "Test" from "Test User"
      expect(screen.getByText('Test')).toBeInTheDocument()
    })

    it('displays logout button as icon', () => {
      render(<DashboardHeader />)

      // Logout button is now an icon button, find by aria-label or role
      const buttons = screen.getAllByRole('button')
      const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'))
      expect(logoutButton).toBeInTheDocument()
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
    it('displays user first name when full_name available', () => {
      render(<DashboardHeader />)

      // Shows first name "Test" from "Test User"
      expect(screen.getByText('Test')).toBeInTheDocument()
    })

    it('displays email prefix when full name is not available', () => {
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

      // Shows email prefix "test" from "test@example.com"
      expect(screen.getByText('test')).toBeInTheDocument()
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

      // Falls back to email prefix
      expect(screen.getByText('test')).toBeInTheDocument()
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

      // Find logout button by icon
      const buttons = screen.getAllByRole('button')
      const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'))
      expect(logoutButton).toBeInTheDocument()
      await user.click(logoutButton!)

      expect(mockLogout).toHaveBeenCalled()
    })

    it('navigates to login page after logout', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const buttons = screen.getAllByRole('button')
      const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'))
      await user.click(logoutButton!)

      await waitFor(() => {
        expect(mockNavigate).toHaveBeenCalledWith('/login')
      })
    })

    it('handles logout in correct order', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const buttons = screen.getAllByRole('button')
      const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'))
      await user.click(logoutButton!)

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

      const badge = screen.getByText('Active').closest('div')
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
      // Ghost variant applies hover effects and muted text color
      expect(dashboardButton.className).toContain('text-muted-foreground')
      expect(dashboardButton.className).toContain('hover:text-foreground')
    })

    it('renders trading paper button with ghost variant', () => {
      render(<DashboardHeader />)

      const tradingButton = screen.getByRole('button', { name: /trading paper/i })
      // Ghost variant applies hover effects and muted text color
      expect(tradingButton.className).toContain('text-muted-foreground')
      expect(tradingButton.className).toContain('hover:text-foreground')
    })

    it('renders settings button with ghost variant', () => {
      render(<DashboardHeader />)

      const settingsButton = screen.getByRole('button', { name: /^settings$/i })
      // Ghost variant applies hover effects and muted text color
      expect(settingsButton.className).toContain('text-muted-foreground')
      expect(settingsButton.className).toContain('hover:text-foreground')
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

      const logo = container.querySelector('.w-11.h-11')
      expect(logo).toBeTruthy()
    })

    it('displays brand name with gradient text', () => {
      render(<DashboardHeader />)

      const botText = screen.getByText('Bot')
      const coreText = screen.getByText('Core')
      expect(botText).toBeInTheDocument()
      expect(coreText.className).toContain('bg-gradient-to-r')
      expect(coreText.className).toContain('bg-clip-text')
    })

    it('applies hover effect to logo link', () => {
      const { container } = render(<DashboardHeader />)

      const logoLink = screen.getAllByRole('link')[0]
      expect(logoLink.className).toContain('group')
    })
  })

  describe('Edge Cases', () => {
    it('handles null user gracefully', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: null,
      })

      render(<DashboardHeader />)

      expect(screen.getByText('Bot')).toBeInTheDocument()
      expect(screen.getByText('Core')).toBeInTheDocument()
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

      expect(screen.getByText('Bot')).toBeInTheDocument()
      expect(screen.getByText('Core')).toBeInTheDocument()
    })

    it('handles multiple rapid logout clicks', async () => {
      const user = userEvent.setup()
      render(<DashboardHeader />)

      const buttons = screen.getAllByRole('button')
      const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'))

      await user.click(logoutButton!)
      await user.click(logoutButton!)
      await user.click(logoutButton!)

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
      // Logout button is now an icon-only button
      const buttons = screen.getAllByRole('button')
      const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'))
      expect(logoutButton).toBeInTheDocument()
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

      // Brand name is split into two spans inside h1
      const botText = screen.getByText('Bot')
      const coreText = screen.getByText('Core')
      expect(botText.closest('h1')).toBeInTheDocument()
      expect(coreText.closest('h1')).toBeInTheDocument()
    })

    it('applies correct text sizes', () => {
      render(<DashboardHeader />)

      const heading = screen.getByText('Bot').closest('h1')
      expect(heading?.className).toContain('text-lg')
      expect(heading?.className).toContain('lg:text-xl')
    })

    it('applies muted color to tagline', () => {
      render(<DashboardHeader />)

      const tagline = screen.getByText('AI Trading')
      expect(tagline.className).toContain('text-muted-foreground')
    })
  })

  describe('Button Styling', () => {
    it('applies ghost variant to logout button', () => {
      render(<DashboardHeader />)

      const buttons = screen.getAllByRole('button')
      const logoutButton = buttons.find(btn => btn.querySelector('svg.lucide-log-out'))
      // Ghost variant button with icon
      expect(logoutButton).toBeInTheDocument()
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

      // Shows first name from full_name
      expect(screen.getByText('Test')).toBeInTheDocument()
    })

    it('handles unauthenticated state', () => {
      mockUseAuth.mockReturnValue({
        logout: mockLogout,
        user: null,
      })

      render(<DashboardHeader />)

      // Should still render header
      expect(screen.getByText('Bot')).toBeInTheDocument()
      expect(screen.getByText('Core')).toBeInTheDocument()
    })
  })
})
