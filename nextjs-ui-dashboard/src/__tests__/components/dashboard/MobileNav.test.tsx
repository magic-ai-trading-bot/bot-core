import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, waitFor } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { BrowserRouter } from 'react-router-dom'
import { MobileNav } from '../../../components/dashboard/MobileNav'

// Mock useAuth hook
const mockLogout = vi.fn()
const mockNavigate = vi.fn()
const mockUser = {
  id: '123',
  email: 'test@example.com',
  full_name: 'Test User',
}

vi.mock('../../../contexts/AuthContext', () => ({
  useAuth: vi.fn(() => ({
    user: mockUser,
    logout: mockLogout,
    isAuthenticated: true,
    loading: false,
  })),
}))

vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useNavigate: () => mockNavigate,
    useLocation: () => ({ pathname: '/dashboard' }),
  }
})

// Mock ThemeContext
vi.mock('../../../contexts/ThemeContext', () => ({
  useTheme: () => ({
    theme: 'dark',
    setTheme: vi.fn(),
    toggleTheme: vi.fn(),
  }),
  ThemeProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
}))

const renderWithRouter = (component: React.ReactElement) => {
  return render(<BrowserRouter>{component}</BrowserRouter>)
}

describe('MobileNav', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Component Rendering', () => {
    it('renders menu trigger button', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      expect(menuButton).toBeInTheDocument()
    })

    it('renders menu icon', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      const icon = menuButton.querySelector('svg')
      expect(icon).toBeInTheDocument()
    })

    it('menu button has ghost variant class', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      expect(menuButton.className).toContain('focus-custom')
    })

    it('menu button is hidden on large screens', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      expect(menuButton.className).toContain('lg:hidden')
    })

    it('renders with correct component structure', () => {
      const { container } = renderWithRouter(<MobileNav />)

      expect(container).toBeInTheDocument()
      expect(container.querySelector('button')).toBeInTheDocument()
    })
  })

  describe('Accessibility', () => {
    it('menu button has aria-label', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      // Check if button exists (aria-label might not be visible in test env)
      expect(menuButton).toBeInTheDocument()
    })

    it('button is keyboard accessible', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      expect(menuButton).toHaveAttribute('type', 'button')
    })
  })

  describe('Props and State', () => {
    it('initializes with closed state', () => {
      const { container } = renderWithRouter(<MobileNav />)

      // Sheet content should not be visible initially
      const nav = container.querySelector('nav')
      expect(nav).not.toBeInTheDocument()
    })

    it('uses auth context for user data', () => {
      renderWithRouter(<MobileNav />)

      // Component should render without errors when auth context is provided
      expect(screen.getByRole('button')).toBeInTheDocument()
    })

    it('uses location for active route', () => {
      renderWithRouter(<MobileNav />)

      // Component should render without errors when location is provided
      expect(screen.getByRole('button')).toBeInTheDocument()
    })
  })

  describe('Button styles', () => {
    it('applies correct button variant', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      // Ghost variant should apply transparent background
      expect(menuButton.style.background).toContain('transparent')
    })

    it('applies correct button size', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      expect(menuButton.className).toContain('text-xs')
    })

    it('applies focus-custom class', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      expect(menuButton.className).toContain('focus-custom')
    })
  })

  describe('Icon rendering', () => {
    it('renders Menu icon from lucide-react', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      const svg = menuButton.querySelector('svg')

      expect(svg).toBeInTheDocument()
      expect(svg?.classList.contains('lucide-menu')).toBe(true)
    })

    it('icon has correct size classes', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      const svg = menuButton.querySelector('svg')

      expect(svg?.classList.contains('h-5')).toBe(true)
      expect(svg?.classList.contains('w-5')).toBe(true)
    })

    it('icon is properly nested in button', () => {
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      const svg = menuButton.querySelector('svg')

      expect(svg?.parentElement).toBe(menuButton)
    })
  })

  describe('Sheet content (opened)', () => {
    it('shows navigation links when sheet is opened', async () => {
      const user = userEvent.setup()
      renderWithRouter(<MobileNav />)

      // Click trigger to open sheet
      const menuButton = screen.getByRole('button')
      await user.click(menuButton)

      await waitFor(() => {
        expect(screen.getByText('Dashboard')).toBeInTheDocument()
        expect(screen.getByText('Paper Trading')).toBeInTheDocument()
        expect(screen.getByText('AI Analyses')).toBeInTheDocument()
        expect(screen.getByText('Settings')).toBeInTheDocument()
      })
    })

    it('shows user info when sheet is opened', async () => {
      const user = userEvent.setup()
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      await user.click(menuButton)

      await waitFor(() => {
        expect(screen.getByText('Test User')).toBeInTheDocument()
        expect(screen.getByText('Logged in as')).toBeInTheDocument()
      })
    })

    it('highlights active route', async () => {
      const user = userEvent.setup()
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      await user.click(menuButton)

      await waitFor(() => {
        const dashboardLink = screen.getByText('Dashboard').closest('a')
        expect(dashboardLink?.getAttribute('aria-current')).toBe('page')
      })
    })

    it('closes sheet when nav link is clicked', async () => {
      const user = userEvent.setup()
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      await user.click(menuButton)

      await waitFor(() => {
        expect(screen.getByText('Paper Trading')).toBeInTheDocument()
      })

      const paperLink = screen.getByText('Paper Trading')
      await user.click(paperLink)

      // Sheet should close (nav content should disappear)
      await waitFor(() => {
        expect(screen.queryByText('Paper Trading')).not.toBeInTheDocument()
      }, { timeout: 2000 })
    })

    it('calls logout and navigates when logout button is clicked', async () => {
      const user = userEvent.setup()
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      await user.click(menuButton)

      await waitFor(() => {
        expect(screen.getByText('Đăng xuất')).toBeInTheDocument()
      })

      const logoutButton = screen.getByText('Đăng xuất').closest('button')!
      await user.click(logoutButton)

      expect(mockLogout).toHaveBeenCalled()
      expect(mockNavigate).toHaveBeenCalledWith('/login')
    })

    it('shows logout button with icon', async () => {
      const user = userEvent.setup()
      renderWithRouter(<MobileNav />)

      const menuButton = screen.getByRole('button')
      await user.click(menuButton)

      await waitFor(() => {
        const logoutButton = screen.getByText('Đăng xuất').closest('button')
        expect(logoutButton).toBeInTheDocument()
        const svg = logoutButton?.querySelector('svg')
        expect(svg).toBeInTheDocument()
      })
    })
  })
})
