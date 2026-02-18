import { describe, it, expect, vi } from 'vitest'
import { render } from '@testing-library/react'
import { BotCoreLogo, BotCoreIcon } from '../../components/BotCoreLogo'

// Mock useThemeColors hook
const mockColors = {
  cyan: '#00D9FF',
  bgPrimary: '#000000',
  textPrimary: '#ffffff',
}

vi.mock('../../hooks/useThemeColors', () => ({
  useThemeColors: vi.fn(() => mockColors),
}))

describe('BotCoreLogo', () => {
  describe('BotCoreLogo component', () => {
    it('renders with default props', () => {
      const { container } = render(<BotCoreLogo />)

      const img = container.querySelector('img')
      expect(img).toBeInTheDocument()
      expect(img?.getAttribute('width')).toBe('40')
      expect(img?.getAttribute('height')).toBe('40')
    })

    it('renders with sm size', () => {
      const { container } = render(<BotCoreLogo size="sm" />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('32')
      expect(img?.getAttribute('height')).toBe('32')
    })

    it('renders with md size', () => {
      const { container } = render(<BotCoreLogo size="md" />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('40')
      expect(img?.getAttribute('height')).toBe('40')
    })

    it('renders with lg size', () => {
      const { container } = render(<BotCoreLogo size="lg" />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('48')
      expect(img?.getAttribute('height')).toBe('48')
    })

    it('renders with xl size', () => {
      const { container } = render(<BotCoreLogo size="xl" />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('64')
      expect(img?.getAttribute('height')).toBe('64')
    })

    it('renders text by default', () => {
      const { getByText } = render(<BotCoreLogo />)

      expect(getByText('Bot Core')).toBeInTheDocument()
    })

    it('hides text when showText is false', () => {
      const { queryByText } = render(<BotCoreLogo showText={false} />)

      expect(queryByText('Bot Core')).not.toBeInTheDocument()
    })

    it('shows text when showText is true', () => {
      const { getByText } = render(<BotCoreLogo showText={true} />)

      expect(getByText('Bot Core')).toBeInTheDocument()
    })

    it('applies custom className', () => {
      const { container } = render(<BotCoreLogo className="custom-logo" />)

      const wrapper = container.firstChild as HTMLElement
      expect(wrapper.className).toContain('custom-logo')
    })

    it('applies flex layout classes', () => {
      const { container } = render(<BotCoreLogo />)

      const wrapper = container.firstChild as HTMLElement
      expect(wrapper.className).toContain('flex')
      expect(wrapper.className).toContain('items-center')
      expect(wrapper.className).toContain('gap-3')
    })

    it('uses correct avatar image source', () => {
      const { container } = render(<BotCoreLogo />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('src')).toBe('/brand/botcore-avatar-512.png')
    })

    it('has alt text', () => {
      const { container } = render(<BotCoreLogo />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('alt')).toBe('BotCore')
    })

    it('applies rounded-lg class to image', () => {
      const { container } = render(<BotCoreLogo />)

      const img = container.querySelector('img')
      expect(img?.className).toContain('rounded-lg')
    })

    it('applies drop shadow filter', () => {
      const { container } = render(<BotCoreLogo />)

      const img = container.querySelector('img')
      const style = img?.getAttribute('style')
      expect(style).toContain('drop-shadow')
      expect(style).toContain(mockColors.cyan)
    })

    it('applies gradient to text', () => {
      const { getByText } = render(<BotCoreLogo />)

      const text = getByText('Bot Core')
      const style = text.getAttribute('style')
      expect(style).toContain('linear-gradient')
      // Style may convert hex to rgb, so check for either format
      expect(style).toMatch(/(?:#00D9FF|rgb\(0, 217, 255\))/)
    })

    it('applies correct text size for sm', () => {
      const { getByText } = render(<BotCoreLogo size="sm" />)

      const text = getByText('Bot Core')
      expect(text.className).toContain('text-base')
    })

    it('applies correct text size for md', () => {
      const { getByText } = render(<BotCoreLogo size="md" />)

      const text = getByText('Bot Core')
      expect(text.className).toContain('text-xl')
    })

    it('applies correct text size for lg', () => {
      const { getByText } = render(<BotCoreLogo size="lg" />)

      const text = getByText('Bot Core')
      expect(text.className).toContain('text-2xl')
    })

    it('applies correct text size for xl', () => {
      const { getByText } = render(<BotCoreLogo size="xl" />)

      const text = getByText('Bot Core')
      expect(text.className).toContain('text-3xl')
    })

    it('applies font-black and tracking-tight to text', () => {
      const { getByText } = render(<BotCoreLogo />)

      const text = getByText('Bot Core')
      expect(text.className).toContain('font-black')
      expect(text.className).toContain('tracking-tight')
    })
  })

  describe('BotCoreIcon component', () => {
    it('renders with default size', () => {
      const { container } = render(<BotCoreIcon />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('40')
      expect(img?.getAttribute('height')).toBe('40')
    })

    it('renders with custom size', () => {
      const { container } = render(<BotCoreIcon size={64} />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('64')
      expect(img?.getAttribute('height')).toBe('64')
    })

    it('renders with small size', () => {
      const { container } = render(<BotCoreIcon size={24} />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('24')
      expect(img?.getAttribute('height')).toBe('24')
    })

    it('renders with large size', () => {
      const { container } = render(<BotCoreIcon size={128} />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('width')).toBe('128')
      expect(img?.getAttribute('height')).toBe('128')
    })

    it('applies custom className', () => {
      const { container } = render(<BotCoreIcon className="icon-class" />)

      const img = container.querySelector('img')
      expect(img?.className).toContain('icon-class')
    })

    it('does not render text', () => {
      const { queryByText } = render(<BotCoreIcon />)

      expect(queryByText('Bot Core')).not.toBeInTheDocument()
    })

    it('uses correct avatar image source', () => {
      const { container } = render(<BotCoreIcon />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('src')).toBe('/brand/botcore-avatar-512.png')
    })

    it('has alt text', () => {
      const { container } = render(<BotCoreIcon />)

      const img = container.querySelector('img')
      expect(img?.getAttribute('alt')).toBe('BotCore')
    })

    it('applies rounded-lg class', () => {
      const { container } = render(<BotCoreIcon />)

      const img = container.querySelector('img')
      expect(img?.className).toContain('rounded-lg')
    })

    it('applies drop shadow filter', () => {
      const { container } = render(<BotCoreIcon />)

      const img = container.querySelector('img')
      const style = img?.getAttribute('style')
      expect(style).toContain('drop-shadow')
      expect(style).toContain(mockColors.cyan)
    })
  })

  describe('Theme integration', () => {
    it('uses theme colors for drop shadow', () => {
      const { container } = render(<BotCoreLogo />)

      const img = container.querySelector('img')
      const style = img?.getAttribute('style')
      expect(style).toContain(mockColors.cyan)
    })

    it('uses theme colors for text gradient', () => {
      const { getByText } = render(<BotCoreLogo />)

      const text = getByText('Bot Core')
      const style = text.getAttribute('style')
      // Style may convert hex to rgb, so check for either format
      expect(style).toMatch(/(?:#00D9FF|rgb\(0, 217, 255\))/)
    })

    it('BotCoreIcon uses theme colors', () => {
      const { container } = render(<BotCoreIcon />)

      const img = container.querySelector('img')
      const style = img?.getAttribute('style')
      expect(style).toContain(mockColors.cyan)
    })
  })
})
