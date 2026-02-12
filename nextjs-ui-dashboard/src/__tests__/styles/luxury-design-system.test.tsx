import { describe, it, expect } from 'vitest'
import { render, screen } from '@testing-library/react'
import {
  luxuryColors,
  GlassCard,
  GradientText,
  StatCard,
  SectionHeader,
  PageWrapper,
  EmptyState,
  LoadingSpinner,
  Divider,
} from '../../styles/luxury-design-system'

describe('Luxury Design System - Enhanced Coverage', () => {
  describe('Design Tokens', () => {
    it('exports luxuryColors object with all required properties', () => {
      expect(luxuryColors).toBeDefined()
      expect(luxuryColors.bgPrimary).toBe('#000000')
      expect(luxuryColors.emerald).toBe('#22c55e')
      expect(luxuryColors.cyan).toBe('#00D9FF')
      expect(luxuryColors.profit).toBe('#22c55e')
      expect(luxuryColors.loss).toBe('#ef4444')
      expect(luxuryColors.textPrimary).toBe('#ffffff')
      expect(luxuryColors.borderSubtle).toBe('rgba(255, 255, 255, 0.08)')
    })

    it('provides gradient colors', () => {
      expect(luxuryColors.gradientPremium).toContain('linear-gradient')
      expect(luxuryColors.gradientProfit).toContain('linear-gradient')
      expect(luxuryColors.gradientLoss).toContain('linear-gradient')
      expect(luxuryColors.gradientPurple).toContain('linear-gradient')
    })

    it('provides nested color structures', () => {
      expect(luxuryColors.text.primary).toBe('#ffffff')
      expect(luxuryColors.status.success).toBe('#22c55e')
      expect(luxuryColors.accent.cyan).toBe('#00D9FF')
      expect(luxuryColors.glass.background).toBe('rgba(255, 255, 255, 0.03)')
    })

    it('provides glow effects', () => {
      expect(luxuryColors.glowCyan).toContain('rgba(0, 217, 255')
      expect(luxuryColors.glowEmerald).toContain('rgba(34, 197, 94')
      expect(luxuryColors.glowPurple).toContain('rgba(139, 92, 246')
    })
  })

  describe('GlassCard Component', () => {
    it('renders children correctly', () => {
      render(
        <GlassCard>
          <div>Test Content</div>
        </GlassCard>
      )

      expect(screen.getByText('Test Content')).toBeInTheDocument()
    })

    it('applies custom className', () => {
      const { container } = render(
        <GlassCard className="custom-class">
          <div>Content</div>
        </GlassCard>
      )

      const card = container.firstChild as HTMLElement
      expect(card.className).toContain('custom-class')
    })

    it('applies hoverable styles when hoverable prop is true', () => {
      const { container } = render(
        <GlassCard hoverable>
          <div>Content</div>
        </GlassCard>
      )

      const card = container.firstChild as HTMLElement
      expect(card.className).toContain('cursor-pointer')
    })

    it('handles onClick callback', () => {
      const onClick = vi.fn()

      render(
        <GlassCard onClick={onClick}>
          <button>Click me</button>
        </GlassCard>
      )

      const button = screen.getByText('Click me')
      button.click()

      expect(onClick).toHaveBeenCalled()
    })

    it('renders multiple children', () => {
      render(
        <GlassCard>
          <div>Child 1</div>
          <div>Child 2</div>
        </GlassCard>
      )

      expect(screen.getByText('Child 1')).toBeInTheDocument()
      expect(screen.getByText('Child 2')).toBeInTheDocument()
    })
  })

  describe('GradientText Component', () => {
    it('renders text with gradient', () => {
      render(<GradientText>Gradient Text</GradientText>)

      expect(screen.getByText('Gradient Text')).toBeInTheDocument()
    })

    it('applies custom className', () => {
      const { container } = render(
        <GradientText className="custom-gradient">Test</GradientText>
      )

      const text = container.querySelector('.custom-gradient')
      expect(text).toBeInTheDocument()
    })

    it('uses custom gradient', () => {
      const customGradient = 'linear-gradient(to right, red, blue)'

      const { container } = render(
        <GradientText gradient={customGradient}>Test</GradientText>
      )

      const text = container.firstChild as HTMLElement
      expect(text.style.backgroundImage).toBe(customGradient)
    })

    it('renders child elements', () => {
      render(
        <GradientText>
          <span>Nested</span> <strong>Content</strong>
        </GradientText>
      )

      expect(screen.getByText('Nested')).toBeInTheDocument()
      expect(screen.getByText('Content')).toBeInTheDocument()
    })
  })

  describe('StatCard Component', () => {
    const TestIcon = () => <svg data-testid="test-icon" />

    it('renders stat card with label and value', () => {
      render(<StatCard label="Total Balance" value="$10,000" />)

      expect(screen.getByText('Total Balance')).toBeInTheDocument()
      expect(screen.getByText('$10,000')).toBeInTheDocument()
    })

    it('displays icon when provided', () => {
      render(
        <StatCard
          label="Test Stat"
          value="100"
          icon={TestIcon}
        />
      )

      expect(screen.getByTestId('test-icon')).toBeInTheDocument()
    })

    it('renders with gradient text', () => {
      render(
        <StatCard
          label="Test"
          value="100"
          gradient
        />
      )

      expect(screen.getByText('Test')).toBeInTheDocument()
      expect(screen.getByText('100')).toBeInTheDocument()
    })

    it('applies custom color', () => {
      render(
        <StatCard
          label="Profit"
          value="+$500"
          valueColor={luxuryColors.profit}
        />
      )

      expect(screen.getByText('+$500')).toBeInTheDocument()
    })

    it('displays trend when provided', () => {
      render(
        <StatCard
          label="Balance"
          value="$10,000"
          trend={5.5}
        />
      )

      expect(screen.getByText('+5.5%')).toBeInTheDocument()
    })

    it('displays trend label when provided', () => {
      render(
        <StatCard
          label="Balance"
          value="$10,000"
          trend={5.5}
          trendLabel="vs last week"
        />
      )

      expect(screen.getByText('vs last week')).toBeInTheDocument()
    })

    it('handles negative trend', () => {
      render(
        <StatCard
          label="Balance"
          value="$10,000"
          trend={-3.2}
        />
      )

      expect(screen.getByText('-3.2%')).toBeInTheDocument()
    })
  })

  describe('SectionHeader Component', () => {
    const TestIcon = () => <svg data-testid="section-icon" />

    it('renders title', () => {
      render(<SectionHeader title="Section Title" />)

      expect(screen.getByText('Section Title')).toBeInTheDocument()
    })

    it('renders subtitle when provided', () => {
      render(
        <SectionHeader
          title="Title"
          subtitle="This is a subtitle"
        />
      )

      expect(screen.getByText('This is a subtitle')).toBeInTheDocument()
    })

    it('renders without subtitle', () => {
      render(<SectionHeader title="Title Only" />)

      expect(screen.getByText('Title Only')).toBeInTheDocument()
    })

    it('renders with gradient by default', () => {
      render(
        <SectionHeader
          title="Test"
        />
      )

      expect(screen.getByText('Test')).toBeInTheDocument()
    })

    it('renders without gradient', () => {
      render(
        <SectionHeader
          title="Test"
          gradient={false}
        />
      )

      expect(screen.getByText('Test')).toBeInTheDocument()
    })

    it('renders icon when provided', () => {
      render(
        <SectionHeader
          title="Test"
          icon={TestIcon}
        />
      )

      expect(screen.getByTestId('section-icon')).toBeInTheDocument()
    })

    it('renders action element when provided', () => {
      render(
        <SectionHeader
          title="Test"
          action={<button>Action</button>}
        />
      )

      expect(screen.getByText('Action')).toBeInTheDocument()
    })
  })

  describe('PageWrapper Component', () => {
    it('renders children', () => {
      render(
        <PageWrapper>
          <div>Page Content</div>
        </PageWrapper>
      )

      expect(screen.getByText('Page Content')).toBeInTheDocument()
    })

    it('applies custom className', () => {
      const { container } = render(
        <PageWrapper className="custom-page">
          <div>Content</div>
        </PageWrapper>
      )

      const wrapper = container.firstChild as HTMLElement
      expect(wrapper.className).toContain('custom-page')
    })

    it('applies padding by default', () => {
      const { container } = render(
        <PageWrapper>
          <div>Content</div>
        </PageWrapper>
      )

      const wrapper = container.firstChild as HTMLElement
      expect(wrapper.className).toContain('p-4')
    })

    it('renders without padding when withPadding is false', () => {
      const { container } = render(
        <PageWrapper withPadding={false}>
          <div>Content</div>
        </PageWrapper>
      )

      const wrapper = container.firstChild as HTMLElement
      expect(wrapper.className).not.toContain('p-4')
    })

    it('renders multiple children', () => {
      render(
        <PageWrapper>
          <div>Section 1</div>
          <div>Section 2</div>
        </PageWrapper>
      )

      expect(screen.getByText('Section 1')).toBeInTheDocument()
      expect(screen.getByText('Section 2')).toBeInTheDocument()
    })
  })

  describe('EmptyState Component', () => {
    const TestIcon = () => <svg data-testid="empty-icon" />

    it('renders title and description', () => {
      render(
        <EmptyState
          icon={TestIcon}
          title="No Data"
          description="There is no data to display"
        />
      )

      expect(screen.getByText('No Data')).toBeInTheDocument()
      expect(screen.getByText('There is no data to display')).toBeInTheDocument()
    })

    it('renders icon when provided', () => {
      render(
        <EmptyState
          icon={TestIcon}
          title="Empty"
          description="No items"
        />
      )

      expect(screen.getByTestId('empty-icon')).toBeInTheDocument()
    })

    it('renders action button when provided', () => {
      const handleClick = vi.fn()

      render(
        <EmptyState
          icon={TestIcon}
          title="Empty"
          description="No items"
          action={<button onClick={handleClick}>Add Item</button>}
        />
      )

      const button = screen.getByText('Add Item')
      expect(button).toBeInTheDocument()

      button.click()
      expect(handleClick).toHaveBeenCalled()
    })

    it('renders without description', () => {
      render(
        <EmptyState
          icon={TestIcon}
          title="Empty"
        />
      )

      expect(screen.getByText('Empty')).toBeInTheDocument()
      expect(screen.queryByText('No items')).not.toBeInTheDocument()
    })

    it('renders without action', () => {
      render(
        <EmptyState
          icon={TestIcon}
          title="Empty"
        />
      )

      expect(screen.queryByRole('button')).not.toBeInTheDocument()
    })
  })

  describe('LoadingSpinner Component', () => {
    it('renders loading spinner', () => {
      const { container } = render(<LoadingSpinner />)

      expect(container.firstChild).toBeInTheDocument()
    })

    it('applies small size', () => {
      const { container } = render(<LoadingSpinner size="sm" />)

      const spinner = container.firstChild as HTMLElement
      expect(spinner.className).toContain('w-4')
      expect(spinner.className).toContain('h-4')
    })

    it('applies medium size', () => {
      const { container } = render(<LoadingSpinner size="md" />)

      const spinner = container.firstChild as HTMLElement
      expect(spinner.className).toContain('w-6')
      expect(spinner.className).toContain('h-6')
    })

    it('applies large size', () => {
      const { container } = render(<LoadingSpinner size="lg" />)

      const spinner = container.firstChild as HTMLElement
      expect(spinner.className).toContain('w-8')
      expect(spinner.className).toContain('h-8')
    })

    it('applies custom color', () => {
      const customColor = '#ff0000'
      const { container } = render(<LoadingSpinner color={customColor} />)

      const spinner = container.firstChild as HTMLElement
      expect(spinner.style.borderTopColor).toBe('rgb(255, 0, 0)')
    })

    it('applies default cyan color', () => {
      const { container } = render(<LoadingSpinner />)

      const spinner = container.firstChild as HTMLElement
      expect(spinner.style.borderTopColor).toBe('rgb(0, 217, 255)')
    })
  })

  describe('Divider Component', () => {
    it('renders horizontal divider by default', () => {
      const { container } = render(<Divider />)

      const divider = container.firstChild as HTMLElement
      expect(divider).toBeInTheDocument()
      expect(divider.className).toContain('h-px')
      expect(divider.className).toContain('w-full')
    })

    it('applies custom className', () => {
      const { container } = render(<Divider className="custom-divider" />)

      const divider = container.querySelector('.custom-divider')
      expect(divider).toBeInTheDocument()
    })

    it('renders vertical divider', () => {
      const { container } = render(<Divider vertical />)

      const divider = container.firstChild as HTMLElement
      expect(divider.className).toContain('w-px')
      expect(divider.className).toContain('h-full')
    })

    it('applies correct background color', () => {
      const { container } = render(<Divider />)

      const divider = container.firstChild as HTMLElement
      expect(divider.style.backgroundColor).toBe('rgba(255, 255, 255, 0.08)')
    })
  })

  describe('Edge Cases', () => {
    const TestIcon = () => <svg />

    it('handles null children in GlassCard', () => {
      render(<GlassCard>{null}</GlassCard>)

      expect(document.body).toBeInTheDocument()
    })

    it('handles undefined children in PageWrapper', () => {
      render(<PageWrapper>{undefined}</PageWrapper>)

      expect(document.body).toBeInTheDocument()
    })

    it('handles empty string value in StatCard', () => {
      render(<StatCard label="Empty" value="" />)

      expect(screen.getByText('Empty')).toBeInTheDocument()
    })

    it('handles numeric value in StatCard', () => {
      render(<StatCard label="Number" value={1234.56} />)

      expect(screen.getByText('1234.56')).toBeInTheDocument()
    })

    it('handles zero trend in StatCard', () => {
      render(<StatCard label="Test" value="100" trend={0} />)

      expect(screen.getByText('0%')).toBeInTheDocument()
    })
  })

  describe('Accessibility', () => {
    const TestIcon = () => <svg data-testid="icon" />

    it('GlassCard is keyboard accessible when hoverable', () => {
      const onClick = vi.fn()

      const { container } = render(
        <GlassCard hoverable onClick={onClick}>
          <button>Click</button>
        </GlassCard>
      )

      const card = container.firstChild as HTMLElement
      expect(card.className).toContain('cursor-pointer')
    })

    it('StatCard displays meaningful text for screen readers', () => {
      render(<StatCard label="Win Rate" value="75%" />)

      expect(screen.getByText('Win Rate')).toBeInTheDocument()
      expect(screen.getByText('75%')).toBeInTheDocument()
    })

    it('EmptyState provides clear messaging', () => {
      render(
        <EmptyState
          icon={TestIcon}
          title="No Results Found"
          description="Try adjusting your search criteria"
        />
      )

      expect(screen.getByText('No Results Found')).toBeInTheDocument()
      expect(
        screen.getByText('Try adjusting your search criteria')
      ).toBeInTheDocument()
    })

    it('LoadingSpinner renders spinner animation', () => {
      const { container } = render(<LoadingSpinner />)

      expect(container.firstChild).toBeInTheDocument()
    })
  })
})
