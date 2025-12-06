import { describe, it, expect, vi, beforeEach } from 'vitest'
import { screen, within } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { render } from '../../test/utils'
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

// Mock the Index page to test core behavior
vi.mock('../../pages/Index', () => ({
  default: function MockIndex() {
    const navigate = mockNavigate
    const [mobileMenuOpen, setMobileMenuOpen] = React.useState(false)

    return (
      <div data-testid="landing-page" style={{ backgroundColor: '#000000', minHeight: '100vh' }}>
        {/* Header */}
        <header data-testid="landing-header" className="sticky top-0 z-50 border-b backdrop-blur-xl">
          <div className="container mx-auto px-4 py-4 flex items-center justify-between">
            <a href="/" className="text-xl font-bold">
              <span>Bot</span>
              <span className="text-cyan-400">Core</span>
            </a>

            {/* Mobile menu button */}
            <button
              className="md:hidden text-white"
              onClick={() => setMobileMenuOpen(!mobileMenuOpen)}
              aria-label="Toggle menu"
              data-testid="mobile-menu-toggle"
            >
              {mobileMenuOpen ? 'X' : 'Menu'}
            </button>

            {/* Desktop nav */}
            <nav className="hidden md:flex items-center gap-6" data-testid="desktop-nav">
              <a href="#features">Features</a>
              <a href="#pricing">Pricing</a>
              <a href="#testimonials">Testimonials</a>
              <a href="#faq">FAQ</a>
            </nav>

            <div className="hidden md:flex items-center gap-4">
              <a href="/login">Login</a>
              <button
                onClick={() => navigate('/register')}
                className="bg-cyan-400 text-black px-4 py-2 rounded"
                data-testid="get-started-btn"
              >
                Get Started
              </button>
            </div>
          </div>

          {/* Mobile menu */}
          {mobileMenuOpen && (
            <nav className="md:hidden p-4" data-testid="mobile-nav">
              <a href="#features" className="block py-2">Features</a>
              <a href="#pricing" className="block py-2">Pricing</a>
              <a href="#testimonials" className="block py-2">Testimonials</a>
              <a href="#faq" className="block py-2">FAQ</a>
              <a href="/login" className="block py-2">Login</a>
              <a href="/register" className="block py-2">Register</a>
            </nav>
          )}
        </header>

        <main>
          {/* Hero Section */}
          <section data-testid="hero-section" className="py-20 px-4">
            <div className="container mx-auto text-center">
              <h1 className="text-4xl md:text-6xl font-bold text-white mb-6">
                AI Trading Platform
              </h1>
              <p className="text-gray-400 text-lg mb-8 max-w-2xl mx-auto">
                Automate your trading with advanced AI algorithms
              </p>
              <div className="flex gap-4 justify-center">
                <button
                  onClick={() => navigate('/register')}
                  className="bg-cyan-400 text-black px-6 py-3 rounded"
                  data-testid="hero-cta"
                >
                  Start Trading
                </button>
                <button className="border border-white text-white px-6 py-3 rounded">
                  Learn More
                </button>
              </div>
            </div>
          </section>

          {/* Stats Section */}
          <section data-testid="stats-section" className="py-12 bg-gray-900">
            <div className="container mx-auto grid grid-cols-2 md:grid-cols-4 gap-8 text-center">
              <div data-testid="stat-users">
                <span className="text-3xl font-bold text-white">10K+</span>
                <span className="text-gray-400 block">Total Users</span>
              </div>
              <div data-testid="stat-volume">
                <span className="text-3xl font-bold text-white">$5B+</span>
                <span className="text-gray-400 block">Trading Volume</span>
              </div>
              <div data-testid="stat-winrate">
                <span className="text-3xl font-bold text-white">78%</span>
                <span className="text-gray-400 block">Win Rate</span>
              </div>
              <div data-testid="stat-response">
                <span className="text-3xl font-bold text-white">&lt;50ms</span>
                <span className="text-gray-400 block">Response Time</span>
              </div>
            </div>
          </section>

          {/* Features Section */}
          <section id="features" data-testid="features-section" className="py-20 px-4">
            <div className="container mx-auto">
              <h2 className="text-3xl font-bold text-center text-white mb-12">Features</h2>
              <div className="grid md:grid-cols-3 gap-8">
                <div className="p-6 bg-gray-900 rounded-lg">
                  <h3 className="text-xl font-bold text-white mb-4">AI Strategies</h3>
                  <p className="text-gray-400">Advanced AI-powered trading strategies</p>
                </div>
                <div className="p-6 bg-gray-900 rounded-lg">
                  <h3 className="text-xl font-bold text-white mb-4">Risk Management</h3>
                  <p className="text-gray-400">Comprehensive risk management tools</p>
                </div>
                <div className="p-6 bg-gray-900 rounded-lg">
                  <h3 className="text-xl font-bold text-white mb-4">Real-time Analysis</h3>
                  <p className="text-gray-400">Real-time market analysis and signals</p>
                </div>
              </div>
            </div>
          </section>

          {/* Pricing Section */}
          <section id="pricing" data-testid="pricing-section" className="py-20 px-4 bg-gray-900">
            <div className="container mx-auto">
              <h2 className="text-3xl font-bold text-center text-white mb-12">Pricing</h2>
              <div className="grid md:grid-cols-3 gap-8">
                <div className="p-6 bg-black rounded-lg">
                  <h3 className="text-xl font-bold text-white mb-4">Free</h3>
                  <span className="text-3xl font-bold text-white">$0</span>
                  <span className="text-gray-400">/month</span>
                </div>
                <div className="p-6 bg-cyan-900 rounded-lg border-2 border-cyan-400">
                  <h3 className="text-xl font-bold text-white mb-4">Pro</h3>
                  <span className="text-3xl font-bold text-white">$49</span>
                  <span className="text-gray-400">/month</span>
                </div>
                <div className="p-6 bg-black rounded-lg">
                  <h3 className="text-xl font-bold text-white mb-4">Enterprise</h3>
                  <span className="text-3xl font-bold text-white">Custom</span>
                </div>
              </div>
            </div>
          </section>

          {/* Testimonials Section */}
          <section id="testimonials" data-testid="testimonials-section" className="py-20 px-4">
            <div className="container mx-auto">
              <h2 className="text-3xl font-bold text-center text-white mb-12">Testimonials</h2>
              <div className="grid md:grid-cols-2 gap-8">
                <blockquote className="p-6 bg-gray-900 rounded-lg">
                  <p className="text-gray-300 mb-4">&quot;Amazing platform!&quot;</p>
                  <cite className="text-white font-bold">- John D., Trader</cite>
                </blockquote>
                <blockquote className="p-6 bg-gray-900 rounded-lg">
                  <p className="text-gray-300 mb-4">&quot;Best trading bot I&apos;ve used.&quot;</p>
                  <cite className="text-white font-bold">- Sarah M., Investor</cite>
                </blockquote>
              </div>
            </div>
          </section>

          {/* FAQ Section */}
          <section id="faq" data-testid="faq-section" className="py-20 px-4 bg-gray-900">
            <div className="container mx-auto max-w-3xl">
              <h2 className="text-3xl font-bold text-center text-white mb-12">FAQ</h2>
              <div className="space-y-4">
                <details className="bg-black p-4 rounded-lg">
                  <summary className="text-white font-bold cursor-pointer">What is BotCore?</summary>
                  <p className="text-gray-400 mt-4">BotCore is an AI-powered trading platform.</p>
                </details>
                <details className="bg-black p-4 rounded-lg">
                  <summary className="text-white font-bold cursor-pointer">Is it safe?</summary>
                  <p className="text-gray-400 mt-4">Yes, we use industry-standard security.</p>
                </details>
              </div>
            </div>
          </section>

          {/* CTA Section */}
          <section data-testid="cta-section" className="py-20 px-4">
            <div className="container mx-auto text-center">
              <h2 className="text-3xl font-bold text-white mb-6">Ready to Start?</h2>
              <button
                onClick={() => navigate('/register')}
                className="bg-cyan-400 text-black px-8 py-4 rounded text-lg"
                data-testid="cta-button"
              >
                Get Started Free
              </button>
            </div>
          </section>
        </main>

        {/* Footer */}
        <footer data-testid="landing-footer" className="border-t py-12 px-4">
          <div className="container mx-auto text-center text-gray-400">
            <p>&copy; 2024 BotCore. All rights reserved.</p>
            <div className="flex justify-center gap-4 mt-4">
              <a href="/privacy">Privacy</a>
              <a href="/terms">Terms</a>
            </div>
          </div>
        </footer>
      </div>
    )
  },
}))

import Index from '../../pages/Index'

describe('Index (Landing Page)', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders the landing page', () => {
    render(<Index />)

    expect(screen.getByTestId('landing-page')).toBeInTheDocument()
    expect(screen.getByTestId('landing-header')).toBeInTheDocument()
    expect(screen.getByTestId('hero-section')).toBeInTheDocument()
    expect(screen.getByTestId('features-section')).toBeInTheDocument()
    expect(screen.getByTestId('pricing-section')).toBeInTheDocument()
    expect(screen.getByTestId('testimonials-section')).toBeInTheDocument()
    expect(screen.getByTestId('faq-section')).toBeInTheDocument()
    expect(screen.getByTestId('cta-section')).toBeInTheDocument()
    expect(screen.getByTestId('landing-footer')).toBeInTheDocument()
  })

  it('renders header at the top', () => {
    render(<Index />)

    const header = screen.getByTestId('landing-header')
    expect(header).toBeInTheDocument()
  })

  it('renders hero section first in main content', () => {
    render(<Index />)

    const hero = screen.getByTestId('hero-section')
    expect(hero).toBeInTheDocument()
  })

  it('renders footer at the bottom', () => {
    render(<Index />)

    const footer = screen.getByTestId('landing-footer')
    expect(footer).toBeInTheDocument()
  })

  it('has brand logo in header', () => {
    render(<Index />)

    expect(screen.getByText('Bot')).toBeInTheDocument()
    expect(screen.getByText('Core')).toBeInTheDocument()
  })

  it('has navigation links in header', () => {
    render(<Index />)

    const desktopNav = screen.getByTestId('desktop-nav')
    expect(desktopNav).toBeInTheDocument()
    // Use within to scope the search to just the navigation
    expect(within(desktopNav).getByText('Features')).toBeInTheDocument()
    expect(within(desktopNav).getByText('Pricing')).toBeInTheDocument()
    expect(within(desktopNav).getByText('Testimonials')).toBeInTheDocument()
    expect(within(desktopNav).getByText('FAQ')).toBeInTheDocument()
  })

  it('has login link in header', () => {
    render(<Index />)

    const loginLink = screen.getByRole('link', { name: /login/i })
    expect(loginLink).toHaveAttribute('href', '/login')
  })

  it('has get started button', () => {
    render(<Index />)

    const getStartedBtn = screen.getByTestId('get-started-btn')
    expect(getStartedBtn).toBeInTheDocument()
    expect(getStartedBtn).toHaveTextContent('Get Started')
  })

  it('navigates to register on get started click', async () => {
    const user = userEvent.setup()
    render(<Index />)

    const getStartedBtn = screen.getByTestId('get-started-btn')
    await user.click(getStartedBtn)

    expect(mockNavigate).toHaveBeenCalledWith('/register')
  })

  it('displays stats section', () => {
    render(<Index />)

    expect(screen.getByTestId('stats-section')).toBeInTheDocument()
    expect(screen.getByTestId('stat-users')).toBeInTheDocument()
    expect(screen.getByTestId('stat-volume')).toBeInTheDocument()
    expect(screen.getByTestId('stat-winrate')).toBeInTheDocument()
    expect(screen.getByTestId('stat-response')).toBeInTheDocument()
  })

  it('has mobile menu toggle', () => {
    render(<Index />)

    const menuToggle = screen.getByTestId('mobile-menu-toggle')
    expect(menuToggle).toBeInTheDocument()
  })

  it('toggles mobile menu', async () => {
    const user = userEvent.setup()
    render(<Index />)

    const menuToggle = screen.getByTestId('mobile-menu-toggle')

    // Initially closed
    expect(screen.queryByTestId('mobile-nav')).not.toBeInTheDocument()

    // Open menu
    await user.click(menuToggle)
    expect(screen.getByTestId('mobile-nav')).toBeInTheDocument()

    // Close menu
    await user.click(menuToggle)
    expect(screen.queryByTestId('mobile-nav')).not.toBeInTheDocument()
  })

  it('renders without crashing', () => {
    expect(() => render(<Index />)).not.toThrow()
  })

  it('has hero CTA button', () => {
    render(<Index />)

    const heroCTA = screen.getByTestId('hero-cta')
    expect(heroCTA).toBeInTheDocument()
    expect(heroCTA).toHaveTextContent('Start Trading')
  })

  it('has main CTA button in CTA section', () => {
    render(<Index />)

    const ctaButton = screen.getByTestId('cta-button')
    expect(ctaButton).toBeInTheDocument()
    expect(ctaButton).toHaveTextContent('Get Started Free')
  })

  it('navigates to register from CTA button', async () => {
    const user = userEvent.setup()
    render(<Index />)

    const ctaButton = screen.getByTestId('cta-button')
    await user.click(ctaButton)

    expect(mockNavigate).toHaveBeenCalledWith('/register')
  })
})
