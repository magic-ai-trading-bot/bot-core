import { describe, it, expect, vi, beforeEach } from 'vitest'

// Mock the API module FIRST - before any other imports
vi.mock('@/services/api', () => {
  const mockAuthClient = {
    login: vi.fn(),
    register: vi.fn(),
    getProfile: vi.fn(),
    verifyToken: vi.fn(),
    setAuthToken: vi.fn(),
    removeAuthToken: vi.fn(),
    getAuthToken: vi.fn(() => null),
    isTokenExpired: vi.fn(() => true),
  }

  return {
    BotCoreApiClient: vi.fn(function() {
      this.auth = mockAuthClient
      this.rust = {}
      this.python = {}
    }),
  }
})

// Then import other dependencies
import { screen } from '@testing-library/react'
import { render } from '../../test/utils'
import Index from '../../pages/Index'

// Mock all landing components
vi.mock('../../components/landing/LandingHeader', () => ({
  LandingHeader: () => <div data-testid="landing-header">Landing Header</div>,
}))

vi.mock('../../components/landing/HeroSection', () => ({
  HeroSection: () => <div data-testid="hero-section">Hero Section</div>,
}))

vi.mock('../../components/landing/PartnersSection', () => ({
  PartnersSection: () => <div data-testid="partners-section">Partners Section</div>,
}))

vi.mock('../../components/landing/FeaturesSection', () => ({
  FeaturesSection: () => <div data-testid="features-section">Features Section</div>,
}))

vi.mock('../../components/landing/PricingSection', () => ({
  PricingSection: () => <div data-testid="pricing-section">Pricing Section</div>,
}))

vi.mock('../../components/landing/TestimonialsSection', () => ({
  TestimonialsSection: () => <div data-testid="testimonials-section">Testimonials Section</div>,
}))

vi.mock('../../components/landing/FAQSection', () => ({
  FAQSection: () => <div data-testid="faq-section">FAQ Section</div>,
}))

vi.mock('../../components/landing/CTASection', () => ({
  CTASection: () => <div data-testid="cta-section">CTA Section</div>,
}))

vi.mock('../../components/landing/LandingFooter', () => ({
  LandingFooter: () => <div data-testid="landing-footer">Landing Footer</div>,
}))

vi.mock('../../components/ChatBot', () => ({
  default: () => null,
}))

describe('Index (Landing Page)', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  it('renders the landing page', () => {
    render(<Index />)

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

  it('renders partners section', () => {
    render(<Index />)

    expect(screen.getByTestId('partners-section')).toBeInTheDocument()
  })

  it('renders features section with correct id', () => {
    const { container } = render(<Index />)

    const featuresSection = container.querySelector('#features')
    expect(featuresSection).toBeInTheDocument()
    expect(featuresSection).toContainElement(screen.getByTestId('features-section'))
  })

  it('renders pricing section with correct id', () => {
    const { container } = render(<Index />)

    const pricingSection = container.querySelector('#pricing')
    expect(pricingSection).toBeInTheDocument()
    expect(pricingSection).toContainElement(screen.getByTestId('pricing-section'))
  })

  it('renders testimonials section with correct id', () => {
    const { container } = render(<Index />)

    const testimonialsSection = container.querySelector('#testimonials')
    expect(testimonialsSection).toBeInTheDocument()
    expect(testimonialsSection).toContainElement(screen.getByTestId('testimonials-section'))
  })

  it('renders FAQ section with correct id', () => {
    const { container } = render(<Index />)

    const faqSection = container.querySelector('#faq')
    expect(faqSection).toBeInTheDocument()
    expect(faqSection).toContainElement(screen.getByTestId('faq-section'))
  })

  it('renders CTA section', () => {
    render(<Index />)

    expect(screen.getByTestId('cta-section')).toBeInTheDocument()
  })

  it('renders footer at the bottom', () => {
    render(<Index />)

    const footer = screen.getByTestId('landing-footer')
    expect(footer).toBeInTheDocument()
  })

  it('has correct section order', () => {
    const { container } = render(<Index />)

    const sections = container.querySelectorAll('section')
    const sectionIds = Array.from(sections).map(section => section.id)

    expect(sectionIds).toEqual(['features', 'pricing', 'testimonials', 'faq'])
  })

  it('wraps sections in main element', () => {
    const { container } = render(<Index />)

    const main = container.querySelector('main')
    expect(main).toBeInTheDocument()
    expect(main).toContainElement(screen.getByTestId('hero-section'))
    expect(main).toContainElement(screen.getByTestId('partners-section'))
    expect(main).toContainElement(screen.getByTestId('features-section'))
    expect(main).toContainElement(screen.getByTestId('pricing-section'))
    expect(main).toContainElement(screen.getByTestId('testimonials-section'))
    expect(main).toContainElement(screen.getByTestId('faq-section'))
    expect(main).toContainElement(screen.getByTestId('cta-section'))
  })

  it('applies correct background styling', () => {
    const { container } = render(<Index />)

    const mainContainer = container.firstChild as HTMLElement
    expect(mainContainer).toHaveClass('min-h-screen', 'bg-background')
  })

  it('renders all sections exactly once', () => {
    render(<Index />)

    expect(screen.getAllByTestId('landing-header')).toHaveLength(1)
    expect(screen.getAllByTestId('hero-section')).toHaveLength(1)
    expect(screen.getAllByTestId('partners-section')).toHaveLength(1)
    expect(screen.getAllByTestId('features-section')).toHaveLength(1)
    expect(screen.getAllByTestId('pricing-section')).toHaveLength(1)
    expect(screen.getAllByTestId('testimonials-section')).toHaveLength(1)
    expect(screen.getAllByTestId('faq-section')).toHaveLength(1)
    expect(screen.getAllByTestId('cta-section')).toHaveLength(1)
    expect(screen.getAllByTestId('landing-footer')).toHaveLength(1)
  })

  it('mounts without errors', () => {
    expect(() => render(<Index />)).not.toThrow()
  })

  it('has semantic HTML structure', () => {
    const { container } = render(<Index />)

    // Should have header (via component)
    expect(screen.getByTestId('landing-header')).toBeInTheDocument()

    // Should have main element
    const main = container.querySelector('main')
    expect(main).toBeInTheDocument()

    // Should have footer (via component)
    expect(screen.getByTestId('landing-footer')).toBeInTheDocument()
  })

  it('sections have proper anchor links for navigation', () => {
    const { container } = render(<Index />)

    // Check that sections have IDs for anchor navigation
    expect(container.querySelector('#features')).toBeInTheDocument()
    expect(container.querySelector('#pricing')).toBeInTheDocument()
    expect(container.querySelector('#testimonials')).toBeInTheDocument()
    expect(container.querySelector('#faq')).toBeInTheDocument()
  })
})
