import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useThemeColors, getThemeColors, darkColors, lightColors, accentColors } from '../../hooks/useThemeColors'

// Mock ThemeContext
const mockResolvedTheme = vi.fn(() => 'dark')

vi.mock('../../contexts/ThemeContext', () => ({
  useTheme: vi.fn(() => ({
    resolvedTheme: mockResolvedTheme(),
    theme: 'dark',
    setTheme: vi.fn(),
  })),
}))

describe('useThemeColors', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  describe('Dark mode colors', () => {
    beforeEach(() => {
      mockResolvedTheme.mockReturnValue('dark')
    })

    it('returns dark theme colors when theme is dark', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.bgPrimary).toBe(darkColors.bgPrimary)
      expect(result.current.textPrimary).toBe(darkColors.textPrimary)
    })

    it('returns correct background colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.bgPrimary).toBe('#000000')
      expect(result.current.bgSecondary).toBe('rgba(255, 255, 255, 0.03)')
      expect(result.current.bgTertiary).toBe('rgba(255, 255, 255, 0.05)')
      expect(result.current.bgHover).toBe('rgba(255, 255, 255, 0.08)')
    })

    it('returns correct text colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.textPrimary).toBe('#ffffff')
      expect(result.current.textSecondary).toBe('rgba(255, 255, 255, 0.7)')
      expect(result.current.textMuted).toBe('rgba(255, 255, 255, 0.4)')
      expect(result.current.textDisabled).toBe('rgba(255, 255, 255, 0.25)')
    })

    it('returns correct border colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.borderSubtle).toBe('rgba(255, 255, 255, 0.08)')
      expect(result.current.borderLight).toBe('rgba(255, 255, 255, 0.12)')
      expect(result.current.borderHover).toBe('rgba(255, 255, 255, 0.15)')
    })

    it('returns dark hero gradients', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.heroGradient1).toContain('#00D9FF')
      expect(result.current.heroGradient2).toContain('#22c55e')
      expect(result.current.heroOrbOpacity).toBe('0.20')
    })
  })

  describe('Light mode colors', () => {
    beforeEach(() => {
      mockResolvedTheme.mockReturnValue('light')
    })

    it('returns light theme colors when theme is light', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.bgPrimary).toBe(lightColors.bgPrimary)
      expect(result.current.textPrimary).toBe(lightColors.textPrimary)
    })

    it('returns correct background colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.bgPrimary).toBe('#faf8f5')
      expect(result.current.bgSecondary).toBe('rgba(0, 0, 0, 0.02)')
      expect(result.current.bgTertiary).toBe('rgba(0, 0, 0, 0.04)')
      expect(result.current.bgHover).toBe('rgba(0, 0, 0, 0.06)')
    })

    it('returns correct text colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.textPrimary).toBe('#1a1a1a')
      expect(result.current.textSecondary).toBe('rgba(0, 0, 0, 0.65)')
      expect(result.current.textMuted).toBe('rgba(0, 0, 0, 0.45)')
      expect(result.current.textDisabled).toBe('rgba(0, 0, 0, 0.25)')
    })

    it('returns correct border colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.borderSubtle).toBe('rgba(0, 0, 0, 0.06)')
      expect(result.current.borderLight).toBe('rgba(0, 0, 0, 0.1)')
      expect(result.current.borderHover).toBe('rgba(0, 0, 0, 0.15)')
    })

    it('returns light hero gradients', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.heroGradient1).toContain('rgba')
      expect(result.current.heroGradient2).toContain('rgba')
      expect(result.current.heroOrbOpacity).toBe('0.08')
    })
  })

  describe('Accent colors (theme-independent)', () => {
    it('returns same accent colors for dark mode', () => {
      mockResolvedTheme.mockReturnValue('dark')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.emerald).toBe('#22c55e')
      expect(result.current.cyan).toBe('#00D9FF')
      expect(result.current.purple).toBe('#8b5cf6')
      expect(result.current.amber).toBe('#f59e0b')
      expect(result.current.rose).toBe('#f43f5e')
    })

    it('returns same accent colors for light mode', () => {
      mockResolvedTheme.mockReturnValue('light')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.emerald).toBe('#22c55e')
      expect(result.current.cyan).toBe('#00D9FF')
      expect(result.current.purple).toBe('#8b5cf6')
      expect(result.current.amber).toBe('#f59e0b')
      expect(result.current.rose).toBe('#f43f5e')
    })

    it('returns semantic colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.profit).toBe('#22c55e')
      expect(result.current.loss).toBe('#ef4444')
      expect(result.current.warning).toBe('#f59e0b')
      expect(result.current.info).toBe('#00D9FF')
      expect(result.current.success).toBe('#22c55e')
    })
  })

  describe('Nested structures', () => {
    it('returns nested text colors for dark mode', () => {
      mockResolvedTheme.mockReturnValue('dark')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.text.primary).toBe('#ffffff')
      expect(result.current.text.secondary).toBe('rgba(255, 255, 255, 0.7)')
      expect(result.current.text.muted).toBe('rgba(255, 255, 255, 0.4)')
      expect(result.current.text.disabled).toBe('rgba(255, 255, 255, 0.25)')
    })

    it('returns nested text colors for light mode', () => {
      mockResolvedTheme.mockReturnValue('light')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.text.primary).toBe('#1a1a1a')
      expect(result.current.text.secondary).toBe('rgba(0, 0, 0, 0.65)')
      expect(result.current.text.muted).toBe('rgba(0, 0, 0, 0.45)')
      expect(result.current.text.disabled).toBe('rgba(0, 0, 0, 0.25)')
    })

    it('returns nested glass colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.glass.background).toBeDefined()
      expect(result.current.glass.blur).toBe('blur(20px)')
      expect(result.current.glass.border).toBeDefined()
    })

    it('returns nested border colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.border.subtle).toBeDefined()
      expect(result.current.border.light).toBeDefined()
      expect(result.current.border.active).toBe('#00D9FF')
      expect(result.current.border.hover).toBeDefined()
    })

    it('returns nested status colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.status.success).toBe('#22c55e')
      expect(result.current.status.error).toBe('#ef4444')
      expect(result.current.status.warning).toBe('#f59e0b')
      expect(result.current.status.info).toBe('#00D9FF')
    })

    it('returns nested accent colors', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.accent.cyan).toBe('#00D9FF')
      expect(result.current.accent.emerald).toBe('#22c55e')
      expect(result.current.accent.purple).toBe('#8b5cf6')
      expect(result.current.accent.amber).toBe('#f59e0b')
      expect(result.current.accent.gold).toBe('#f59e0b')
      expect(result.current.accent.rose).toBe('#f43f5e')
    })
  })

  describe('Gradients', () => {
    it('returns gradient definitions', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.gradientPremium).toContain('linear-gradient')
      expect(result.current.gradientProfit).toContain('linear-gradient')
      expect(result.current.gradientLoss).toContain('linear-gradient')
      expect(result.current.gradientPurple).toContain('linear-gradient')
      expect(result.current.gradientGold).toContain('linear-gradient')
      expect(result.current.gradientCyan).toContain('linear-gradient')
    })

    it('returns nested gradient definitions', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.gradient.premium).toContain('linear-gradient')
      expect(result.current.gradient.profit).toContain('linear-gradient')
      expect(result.current.gradient.loss).toContain('linear-gradient')
      expect(result.current.gradient.purple).toContain('linear-gradient')
      expect(result.current.gradient.gold).toContain('linear-gradient')
      expect(result.current.gradient.cyan).toContain('linear-gradient')
    })
  })

  describe('Glow effects', () => {
    it('returns glow effects', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.glowCyan).toContain('rgba(0, 217, 255')
      expect(result.current.glowEmerald).toContain('rgba(34, 197, 94')
      expect(result.current.glowPurple).toContain('rgba(139, 92, 246')
      expect(result.current.glowRed).toContain('rgba(239, 68, 68')
    })

    it('returns nested glow effects', () => {
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.glow.cyan).toContain('rgba(0, 217, 255')
      expect(result.current.glow.emerald).toContain('rgba(34, 197, 94')
      expect(result.current.glow.purple).toContain('rgba(139, 92, 246')
      expect(result.current.glow.red).toContain('rgba(239, 68, 68')
    })
  })

  describe('getThemeColors function', () => {
    it('returns dark colors when theme is dark', () => {
      const colors = getThemeColors('dark')

      expect(colors.bgPrimary).toBe('#000000')
      expect(colors.textPrimary).toBe('#ffffff')
    })

    it('returns light colors when theme is light', () => {
      const colors = getThemeColors('light')

      expect(colors.bgPrimary).toBe('#faf8f5')
      expect(colors.textPrimary).toBe('#1a1a1a')
    })

    it('includes accent colors', () => {
      const colors = getThemeColors('dark')

      expect(colors.cyan).toBe('#00D9FF')
      expect(colors.emerald).toBe('#22c55e')
    })

    it('includes nested structures', () => {
      const colors = getThemeColors('dark')

      expect(colors.text.primary).toBe('#ffffff')
      expect(colors.glass.blur).toBe('blur(20px)')
      expect(colors.border.active).toBe('#00D9FF')
    })
  })

  describe('Exported constants', () => {
    it('exports darkColors', () => {
      expect(darkColors).toBeDefined()
      expect(darkColors.bgPrimary).toBe('#000000')
    })

    it('exports lightColors', () => {
      expect(lightColors).toBeDefined()
      expect(lightColors.bgPrimary).toBe('#faf8f5')
    })

    it('exports accentColors', () => {
      expect(accentColors).toBeDefined()
      expect(accentColors.cyan).toBe('#00D9FF')
    })
  })

  describe('Glass effect', () => {
    it('returns glass background for dark mode', () => {
      mockResolvedTheme.mockReturnValue('dark')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.glassBackground).toBe('rgba(255, 255, 255, 0.03)')
      expect(result.current.glassBorder).toBe('rgba(255, 255, 255, 0.08)')
    })

    it('returns glass background for light mode', () => {
      mockResolvedTheme.mockReturnValue('light')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.glassBackground).toBe('rgba(255, 255, 255, 0.7)')
      expect(result.current.glassBorder).toBe('rgba(0, 0, 0, 0.08)')
    })
  })

  describe('Card backgrounds', () => {
    it('returns card backgrounds for dark mode', () => {
      mockResolvedTheme.mockReturnValue('dark')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.bgCard).toBe('rgba(255, 255, 255, 0.03)')
      expect(result.current.bgHeader).toBe('rgba(0, 0, 0, 0.8)')
      expect(result.current.bgMobileMenu).toBe('rgba(0, 0, 0, 0.95)')
    })

    it('returns card backgrounds for light mode', () => {
      mockResolvedTheme.mockReturnValue('light')
      const { result } = renderHook(() => useThemeColors())

      expect(result.current.bgCard).toBe('rgba(255, 255, 255, 0.8)')
      expect(result.current.bgHeader).toBe('rgba(250, 248, 245, 0.9)')
      expect(result.current.bgMobileMenu).toBe('rgba(250, 248, 245, 0.98)')
    })
  })

  describe('Border active color', () => {
    it('returns same border active color for all themes', () => {
      mockResolvedTheme.mockReturnValue('dark')
      const darkResult = renderHook(() => useThemeColors())

      mockResolvedTheme.mockReturnValue('light')
      const lightResult = renderHook(() => useThemeColors())

      expect(darkResult.result.current.borderActive).toBe('#00D9FF')
      expect(lightResult.result.current.borderActive).toBe('#00D9FF')
    })
  })
})
