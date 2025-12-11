/**
 * useThemeColors Hook
 *
 * Dynamic theme-aware color system that responds to light/dark mode changes.
 * Replaces the static luxuryColors object for theme-responsive styling.
 */

import { useTheme } from '@/contexts/ThemeContext';

// Dark mode colors (OLED luxury)
const darkColors = {
  // Backgrounds - Pure black for OLED
  bgPrimary: '#000000',
  bgSecondary: 'rgba(255, 255, 255, 0.03)',
  bgTertiary: 'rgba(255, 255, 255, 0.05)',
  bgHover: 'rgba(255, 255, 255, 0.08)',
  bgCard: 'rgba(255, 255, 255, 0.03)',
  bgHeader: 'rgba(0, 0, 0, 0.8)',
  bgMobileMenu: 'rgba(0, 0, 0, 0.95)',

  // Text
  textPrimary: '#ffffff',
  textSecondary: 'rgba(255, 255, 255, 0.7)',
  textMuted: 'rgba(255, 255, 255, 0.4)',
  textDisabled: 'rgba(255, 255, 255, 0.25)',

  // Borders
  borderSubtle: 'rgba(255, 255, 255, 0.08)',
  borderLight: 'rgba(255, 255, 255, 0.12)',
  borderHover: 'rgba(255, 255, 255, 0.15)',

  // Glass effect
  glassBackground: 'rgba(255, 255, 255, 0.03)',
  glassBorder: 'rgba(255, 255, 255, 0.08)',
} as const;

// Light mode colors (Luxury cream/warm)
const lightColors = {
  // Backgrounds - Warm luxury cream
  bgPrimary: '#faf8f5',
  bgSecondary: 'rgba(0, 0, 0, 0.02)',
  bgTertiary: 'rgba(0, 0, 0, 0.04)',
  bgHover: 'rgba(0, 0, 0, 0.06)',
  bgCard: 'rgba(255, 255, 255, 0.8)',
  bgHeader: 'rgba(250, 248, 245, 0.9)',
  bgMobileMenu: 'rgba(250, 248, 245, 0.98)',

  // Text
  textPrimary: '#1a1a1a',
  textSecondary: 'rgba(0, 0, 0, 0.65)',
  textMuted: 'rgba(0, 0, 0, 0.45)',
  textDisabled: 'rgba(0, 0, 0, 0.25)',

  // Borders
  borderSubtle: 'rgba(0, 0, 0, 0.06)',
  borderLight: 'rgba(0, 0, 0, 0.1)',
  borderHover: 'rgba(0, 0, 0, 0.15)',

  // Glass effect
  glassBackground: 'rgba(255, 255, 255, 0.7)',
  glassBorder: 'rgba(0, 0, 0, 0.08)',

  // Light mode specific - Hero section decorative gradients (subtle, light)
  heroGradient1: 'linear-gradient(135deg, rgba(0, 217, 255, 0.1), rgba(34, 197, 94, 0.1))',
  heroGradient2: 'linear-gradient(135deg, rgba(34, 197, 94, 0.08), rgba(0, 217, 255, 0.08))',
  heroOrbOpacity: '0.08',
} as const;

// Dark mode hero gradients
const darkHeroColors = {
  heroGradient1: 'linear-gradient(135deg, #00D9FF, #06b6d4)',
  heroGradient2: 'linear-gradient(135deg, #22c55e, #00D9FF)',
  heroOrbOpacity: '0.20',
} as const;

// Shared accent colors (same for both themes)
const accentColors = {
  // Primary Accents
  emerald: '#22c55e',
  cyan: '#00D9FF',
  purple: '#8b5cf6',
  amber: '#f59e0b',
  rose: '#f43f5e',

  // Semantic Colors
  profit: '#22c55e',
  loss: '#ef4444',
  warning: '#f59e0b',
  info: '#00D9FF',
  success: '#22c55e',

  // Border active
  borderActive: '#00D9FF',

  // Gradients
  gradientPremium: 'linear-gradient(135deg, #00D9FF, #22c55e)',
  gradientProfit: 'linear-gradient(135deg, #22c55e, #00D9FF)',
  gradientLoss: 'linear-gradient(135deg, #ef4444, #f97316)',
  gradientPurple: 'linear-gradient(135deg, #8b5cf6, #ec4899)',
  gradientGold: 'linear-gradient(135deg, #f59e0b, #fbbf24)',
  gradientCyan: 'linear-gradient(135deg, #00D9FF, #06b6d4)',

  // Glow Effects
  glowCyan: '0 0 20px rgba(0, 217, 255, 0.3)',
  glowEmerald: '0 0 20px rgba(34, 197, 94, 0.3)',
  glowPurple: '0 0 20px rgba(139, 92, 246, 0.3)',
  glowRed: '0 0 20px rgba(239, 68, 68, 0.3)',

  // Nested structures for backward compatibility
  text: {
    primary: '', // Will be set dynamically
    secondary: '',
    muted: '',
    disabled: '',
  },
  status: {
    success: '#22c55e',
    error: '#ef4444',
    warning: '#f59e0b',
    info: '#00D9FF',
  },
  accent: {
    cyan: '#00D9FF',
    emerald: '#22c55e',
    purple: '#8b5cf6',
    amber: '#f59e0b',
    gold: '#f59e0b',
    rose: '#f43f5e',
  },
  glass: {
    background: '',
    blur: 'blur(20px)',
    border: '',
  },
  border: {
    subtle: '',
    light: '',
    active: '#00D9FF',
    hover: '',
  },
  gradient: {
    premium: 'linear-gradient(135deg, #00D9FF, #22c55e)',
    profit: 'linear-gradient(135deg, #22c55e, #00D9FF)',
    loss: 'linear-gradient(135deg, #ef4444, #f97316)',
    purple: 'linear-gradient(135deg, #8b5cf6, #ec4899)',
    gold: 'linear-gradient(135deg, #f59e0b, #fbbf24)',
    cyan: 'linear-gradient(135deg, #00D9FF, #06b6d4)',
  },
  glow: {
    cyan: '0 0 20px rgba(0, 217, 255, 0.3)',
    emerald: '0 0 20px rgba(34, 197, 94, 0.3)',
    purple: '0 0 20px rgba(139, 92, 246, 0.3)',
    red: '0 0 20px rgba(239, 68, 68, 0.3)',
  },
} as const;

export type ThemeColors = typeof darkColors & typeof accentColors;

/**
 * Hook to get theme-aware colors
 * Returns colors that automatically update when theme changes
 */
export function useThemeColors() {
  const { resolvedTheme } = useTheme();
  const isDark = resolvedTheme === 'dark';

  const themeColors = isDark ? darkColors : lightColors;
  const heroColors = isDark ? darkHeroColors : {
    heroGradient1: lightColors.heroGradient1,
    heroGradient2: lightColors.heroGradient2,
    heroOrbOpacity: lightColors.heroOrbOpacity,
  };

  // Merge theme-specific colors with accent colors and update nested structures
  return {
    ...themeColors,
    ...accentColors,
    ...heroColors,
    // Update nested text colors based on theme
    text: {
      primary: themeColors.textPrimary,
      secondary: themeColors.textSecondary,
      muted: themeColors.textMuted,
      disabled: themeColors.textDisabled,
    },
    // Update nested glass colors based on theme
    glass: {
      background: themeColors.glassBackground,
      blur: 'blur(20px)',
      border: themeColors.glassBorder,
    },
    // Update nested border colors based on theme
    border: {
      subtle: themeColors.borderSubtle,
      light: themeColors.borderLight,
      active: '#00D9FF',
      hover: themeColors.borderHover,
    },
  };
}

/**
 * Get static colors for a specific theme (for use outside React components)
 */
export function getThemeColors(theme: 'light' | 'dark'): ThemeColors {
  const themeColors = theme === 'dark' ? darkColors : lightColors;

  return {
    ...themeColors,
    ...accentColors,
    text: {
      primary: themeColors.textPrimary,
      secondary: themeColors.textSecondary,
      muted: themeColors.textMuted,
      disabled: themeColors.textDisabled,
    },
    glass: {
      background: themeColors.glassBackground,
      blur: 'blur(20px)',
      border: themeColors.glassBorder,
    },
    border: {
      subtle: themeColors.borderSubtle,
      light: themeColors.borderLight,
      active: '#00D9FF',
      hover: themeColors.borderHover,
    },
  };
}

export { darkColors, lightColors, accentColors };
