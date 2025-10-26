import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
import NotFound from '../../pages/NotFound'

// Mock useLocation
const mockLocation = { pathname: '/non-existent-page' }
vi.mock('react-router-dom', async () => {
  const actual = await vi.importActual('react-router-dom')
  return {
    ...actual,
    useLocation: () => mockLocation,
  }
})

// Mock ChatBot
vi.mock('../../components/ChatBot', () => ({
  default: () => null,
}))

// Mock logger.error
const loggerErrorSpy = vi.fn()
vi.mock('@/utils/logger', () => ({
  default: {
    error: (...args: any[]) => loggerErrorSpy(...args),
    info: vi.fn(),
    warn: vi.fn(),
    debug: vi.fn(),
  },
}))

describe('NotFound', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    loggerErrorSpy.mockClear()
  })

  it('renders 404 page', () => {
    render(<NotFound />)

    expect(screen.getByText('404')).toBeInTheDocument()
    expect(screen.getByText('Oops! Page not found')).toBeInTheDocument()
  })

  it('displays 404 heading', () => {
    render(<NotFound />)

    const heading = screen.getByRole('heading', { level: 1 })
    expect(heading).toHaveTextContent('404')
  })

  it('shows error message', () => {
    render(<NotFound />)

    expect(screen.getByText('Oops! Page not found')).toBeInTheDocument()
  })

  it('has link to return home', () => {
    render(<NotFound />)

    const homeLink = screen.getByRole('link', { name: /return to home/i })
    expect(homeLink).toBeInTheDocument()
    expect(homeLink).toHaveAttribute('href', '/')
  })

  it('logs error to console with pathname', () => {
    render(<NotFound />)

    expect(loggerErrorSpy).toHaveBeenCalledWith(
      '404 Error: User attempted to access non-existent route:',
      '/non-existent-page'
    )
  })

  it('logs error only once on mount', () => {
    render(<NotFound />)

    expect(loggerErrorSpy).toHaveBeenCalledTimes(1)
  })

  it('uses correct pathname from location', () => {
    mockLocation.pathname = '/some-other-page'
    render(<NotFound />)

    expect(loggerErrorSpy).toHaveBeenCalledWith(
      '404 Error: User attempted to access non-existent route:',
      '/some-other-page'
    )
  })

  it('applies correct styling to container', () => {
    const { container } = render(<NotFound />)

    const mainDiv = container.firstChild as HTMLElement
    expect(mainDiv).toHaveClass('min-h-screen', 'flex', 'items-center', 'justify-center', 'bg-gray-100')
  })

  it('centers content properly', () => {
    const { container } = render(<NotFound />)

    const contentDiv = container.querySelector('.text-center')
    expect(contentDiv).toBeInTheDocument()
  })

  it('styles 404 heading correctly', () => {
    render(<NotFound />)

    const heading = screen.getByText('404')
    expect(heading).toHaveClass('text-4xl', 'font-bold', 'mb-4')
  })

  it('styles error message correctly', () => {
    render(<NotFound />)

    const message = screen.getByText('Oops! Page not found')
    expect(message).toHaveClass('text-xl', 'text-gray-600', 'mb-4')
  })

  it('styles home link correctly', () => {
    render(<NotFound />)

    const homeLink = screen.getByRole('link', { name: /return to home/i })
    expect(homeLink).toHaveClass('text-blue-500', 'hover:text-blue-700', 'underline')
  })

  it('home link is clickable', async () => {
    const user = userEvent.setup()
    render(<NotFound />)

    const homeLink = screen.getByRole('link', { name: /return to home/i })

    // Should not throw error when clicked
    await expect(user.click(homeLink)).resolves.not.toThrow()
  })

  it('renders without crashing', () => {
    expect(() => render(<NotFound />)).not.toThrow()
  })

  it('handles different pathname formats', () => {
    const testPaths = [
      '/invalid-route',
      '/user/123/edit',
      '/deeply/nested/path/that/does/not/exist',
      '/路径/with/unicode',
      '/%20spaces%20in%20path'
    ]

    testPaths.forEach(path => {
      loggerErrorSpy.mockClear()
      mockLocation.pathname = path
      const { unmount } = render(<NotFound />)

      expect(loggerErrorSpy).toHaveBeenCalledWith(
        '404 Error: User attempted to access non-existent route:',
        path
      )

      unmount()
    })
  })

  it('logs error once per mount', () => {
    const { rerender } = render(<NotFound />)

    expect(loggerErrorSpy).toHaveBeenCalledTimes(1)

    // Re-rendering should not call again since pathname hasn't changed
    rerender(<NotFound />)

    // Should still be 1 since the dependency (pathname) hasn't changed
    expect(loggerErrorSpy).toHaveBeenCalledTimes(1)
  })

  it('displays all elements in correct hierarchy', () => {
    const { container } = render(<NotFound />)

    const mainDiv = container.firstChild as HTMLElement
    const centerDiv = mainDiv.querySelector('.text-center')

    expect(centerDiv).toContainElement(screen.getByText('404'))
    expect(centerDiv).toContainElement(screen.getByText('Oops! Page not found'))
    expect(centerDiv).toContainElement(screen.getByRole('link', { name: /return to home/i }))
  })

  it('has accessible structure', () => {
    render(<NotFound />)

    // Should have heading for screen readers
    expect(screen.getByRole('heading', { level: 1 })).toBeInTheDocument()

    // Should have link for navigation
    expect(screen.getByRole('link')).toBeInTheDocument()
  })
})
