import { describe, it, expect, vi } from 'vitest'
import { render } from '@testing-library/react'
import { Logo, LogoIcon } from '../../../components/ui/Logo'

// Mock BotCoreLogo components
vi.mock('../../../components/BotCoreLogo', () => ({
  BotCoreLogo: ({ size, showText, className }: any) => (
    <div data-testid="bot-core-logo" data-size={size} data-show-text={showText} className={className}>
      BotCoreLogo Mock
    </div>
  ),
  BotCoreIcon: ({ size, className }: any) => (
    <div data-testid="bot-core-icon" data-size={size} className={className}>
      BotCoreIcon Mock
    </div>
  ),
}))

describe('Logo', () => {
  describe('Logo component', () => {
    it('renders BotCoreLogo with default props', () => {
      const { getByTestId } = render(<Logo />)

      const logo = getByTestId('bot-core-logo')
      expect(logo).toBeInTheDocument()
      expect(logo.getAttribute('data-size')).toBe('md')
      expect(logo.getAttribute('data-show-text')).toBe('true')
    })

    it('renders with sm size', () => {
      const { getByTestId } = render(<Logo size="sm" />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-size')).toBe('sm')
    })

    it('renders with md size', () => {
      const { getByTestId } = render(<Logo size="md" />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-size')).toBe('md')
    })

    it('renders with lg size', () => {
      const { getByTestId } = render(<Logo size="lg" />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-size')).toBe('lg')
    })

    it('renders with xl size', () => {
      const { getByTestId } = render(<Logo size="xl" />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-size')).toBe('xl')
    })

    it('renders with text shown by default', () => {
      const { getByTestId } = render(<Logo />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-show-text')).toBe('true')
    })

    it('renders with text hidden when showText is false', () => {
      const { getByTestId } = render(<Logo showText={false} />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-show-text')).toBe('false')
    })

    it('renders with text shown when showText is true', () => {
      const { getByTestId } = render(<Logo showText={true} />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-show-text')).toBe('true')
    })

    it('passes custom className', () => {
      const { getByTestId } = render(<Logo className="custom-class" />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.className).toContain('custom-class')
    })

    it('passes multiple props together', () => {
      const { getByTestId } = render(<Logo size="lg" showText={false} className="test-logo" />)

      const logo = getByTestId('bot-core-logo')
      expect(logo.getAttribute('data-size')).toBe('lg')
      expect(logo.getAttribute('data-show-text')).toBe('false')
      expect(logo.className).toContain('test-logo')
    })
  })

  describe('LogoIcon component', () => {
    it('renders BotCoreIcon with default props', () => {
      const { getByTestId } = render(<LogoIcon />)

      const icon = getByTestId('bot-core-icon')
      expect(icon).toBeInTheDocument()
      expect(icon.getAttribute('data-size')).toBe('32')
    })

    it('renders with sm size (24px)', () => {
      const { getByTestId } = render(<LogoIcon size="sm" />)

      const icon = getByTestId('bot-core-icon')
      expect(icon.getAttribute('data-size')).toBe('24')
    })

    it('renders with md size (32px)', () => {
      const { getByTestId } = render(<LogoIcon size="md" />)

      const icon = getByTestId('bot-core-icon')
      expect(icon.getAttribute('data-size')).toBe('32')
    })

    it('renders with lg size (40px)', () => {
      const { getByTestId } = render(<LogoIcon size="lg" />)

      const icon = getByTestId('bot-core-icon')
      expect(icon.getAttribute('data-size')).toBe('40')
    })

    it('renders with xl size (56px)', () => {
      const { getByTestId } = render(<LogoIcon size="xl" />)

      const icon = getByTestId('bot-core-icon')
      expect(icon.getAttribute('data-size')).toBe('56')
    })

    it('passes custom className', () => {
      const { getByTestId } = render(<LogoIcon className="icon-custom" />)

      const icon = getByTestId('bot-core-icon')
      expect(icon.className).toContain('icon-custom')
    })

    it('passes size and className together', () => {
      const { getByTestId } = render(<LogoIcon size="lg" className="large-icon" />)

      const icon = getByTestId('bot-core-icon')
      expect(icon.getAttribute('data-size')).toBe('40')
      expect(icon.className).toContain('large-icon')
    })

    it('does not accept showText prop', () => {
      // LogoIcon omits showText prop - verify it only accepts size and className
      const { getByTestId } = render(<LogoIcon size="md" className="test" />)

      const icon = getByTestId('bot-core-icon')
      expect(icon.getAttribute('data-show-text')).toBeNull()
    })
  })

  describe('Backward compatibility', () => {
    it('Logo component acts as wrapper for BotCoreLogo', () => {
      const { getByTestId } = render(<Logo size="sm" showText={false} />)

      expect(getByTestId('bot-core-logo')).toBeInTheDocument()
    })

    it('LogoIcon acts as wrapper for BotCoreIcon', () => {
      const { getByTestId } = render(<LogoIcon size="sm" />)

      expect(getByTestId('bot-core-icon')).toBeInTheDocument()
    })
  })

  describe('Size mapping', () => {
    it('maps correct pixel size for each Logo size', () => {
      const sizes = ['sm', 'md', 'lg', 'xl'] as const
      const expectedSizes = { sm: '24', md: '32', lg: '40', xl: '56' }

      sizes.forEach((size) => {
        const { getByTestId, unmount } = render(<LogoIcon size={size} />)
        const icon = getByTestId('bot-core-icon')
        expect(icon.getAttribute('data-size')).toBe(expectedSizes[size])
        unmount() // Clean up between iterations
      })
    })
  })
})
