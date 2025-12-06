/**
 * Paper Trading Mode Theme
 *
 * Blue accent theme for safe sandbox environment.
 * Optimized for visual distinction from real trading mode.
 */

import { colors } from '../tokens/colors';

export const paperTheme = {
  // Mode identifier
  mode: 'paper' as const,

  // Primary colors - Blue accent
  primary: colors.paper.accent,
  primaryHover: colors.paper.hover,
  primaryBorder: colors.paper.border,
  primaryBackground: colors.paper.background,

  // Badge styling
  badge: {
    background: colors.paper.badge,
    text: colors.text.inverse,
    label: 'PAPER',
    pulse: false, // No pulsing for paper mode
  },

  // Banner styling (top of dashboard)
  banner: {
    background: colors.paper.background, // Use with 0.1 opacity
    border: colors.paper.border, // Use with 0.2 opacity
    text: colors.paper.accent,
    icon: '🧪', // Test tube emoji
    message: 'Paper Trading Mode - Simulated Environment',
  },

  // Card styling overrides
  card: {
    borderColor: colors.paper.border, // Use with 0.2 opacity
    highlightColor: colors.paper.accent, // Use with 0.1 opacity
  },

  // Button styling overrides
  button: {
    primary: {
      background: colors.paper.accent,
      text: colors.text.inverse,
      hover: colors.paper.hover,
    },
  },

  // Status colors remain the same (profit/loss)
  profit: colors.profit,
  loss: colors.loss,
  neutral: colors.neutral,
} as const;

export type PaperTheme = typeof paperTheme;
