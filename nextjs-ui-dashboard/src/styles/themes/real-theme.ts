/**
 * Real Trading Mode Theme
 *
 * Red warning theme for real money environment.
 * High-visibility warnings to prevent accidental actions.
 */

import { colors } from '../tokens/colors';

export const realTheme = {
  // Mode identifier
  mode: 'real' as const,

  // Primary colors - Red warnings
  primary: colors.real.warning,
  primaryHover: colors.real.hover,
  primaryBorder: colors.real.border,
  primaryBackground: colors.real.background,

  // Badge styling - Pulsing red to draw attention
  badge: {
    background: colors.real.banner,
    text: colors.text.primary,
    label: 'REAL MONEY',
    pulse: true, // Pulsing animation for visibility
  },

  // Banner styling (top of dashboard) - High visibility
  banner: {
    background: colors.real.background, // Use with 0.1 opacity
    border: colors.real.border, // Use with 0.3 opacity
    text: colors.real.warning,
    icon: '⚠️', // Warning emoji
    message: 'Real Trading Mode - Live Money at Risk',
  },

  // Card styling overrides
  card: {
    borderColor: colors.real.border, // Use with 0.3 opacity
    highlightColor: colors.real.warning, // Use with 0.1 opacity
  },

  // Button styling overrides - Dangerous actions
  button: {
    primary: {
      background: colors.real.warning,
      text: colors.text.primary,
      hover: colors.real.hover,
    },
  },

  // Status colors remain the same (profit/loss)
  profit: colors.profit,
  loss: colors.loss,
  neutral: colors.neutral,
} as const;

export type RealTheme = typeof realTheme;
