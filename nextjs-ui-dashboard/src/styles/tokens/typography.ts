/**
 * Design Tokens - Typography
 *
 * Typography scale optimized for trading data density and readability.
 * Uses Inter font family with tabular numbers for consistent number alignment.
 */

export const typography = {
  // Font families
  fontFamily: {
    sans: "'Inter', -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', sans-serif",
    mono: "'JetBrains Mono', 'Fira Code', 'Courier New', monospace",
  },

  // Font sizes - Optimized for trading dashboard
  fontSize: {
    xs: '0.75rem',      // 12px - Compact data labels
    sm: '0.875rem',     // 14px - Secondary text, table data
    base: '1rem',       // 16px - Body text, default
    lg: '1.125rem',     // 18px - Emphasized text
    xl: '1.25rem',      // 20px - Section headings
    '2xl': '1.5rem',    // 24px - Card headings
    '3xl': '1.875rem',  // 30px - Large numbers (P&L)
    '4xl': '2.25rem',   // 36px - Hero numbers
    '5xl': '3rem',      // 48px - Dashboard totals
  },

  // Font weights
  fontWeight: {
    normal: '400',
    medium: '500',
    semibold: '600',
    bold: '700',
  },

  // Line heights - Compact for data density
  lineHeight: {
    tight: '1.25',      // For large numbers
    normal: '1.5',      // For body text
    relaxed: '1.75',    // For longer content
  },

  // Letter spacing
  letterSpacing: {
    tighter: '-0.05em',
    tight: '-0.025em',
    normal: '0',
    wide: '0.025em',
  },

  // Text transforms
  textTransform: {
    uppercase: 'uppercase',
    lowercase: 'lowercase',
    capitalize: 'capitalize',
    none: 'none',
  },
} as const;

// Type exports
export type FontFamilyKey = keyof typeof typography.fontFamily;
export type FontSizeKey = keyof typeof typography.fontSize;
export type FontWeightKey = keyof typeof typography.fontWeight;
export type LineHeightKey = keyof typeof typography.lineHeight;
export type LetterSpacingKey = keyof typeof typography.letterSpacing;

// Preset combinations for common use cases
export const typographyPresets = {
  // Trading numbers - Tabular numbers, tight spacing
  tradingNumber: {
    fontFamily: typography.fontFamily.mono,
    fontWeight: typography.fontWeight.semibold,
    letterSpacing: typography.letterSpacing.tight,
    lineHeight: typography.lineHeight.tight,
    fontFeatureSettings: "'tnum', 'zero'", // Tabular numbers with slashed zero
  },

  // Card heading
  cardHeading: {
    fontFamily: typography.fontFamily.sans,
    fontSize: typography.fontSize['2xl'],
    fontWeight: typography.fontWeight.semibold,
    lineHeight: typography.lineHeight.tight,
  },

  // Body text
  body: {
    fontFamily: typography.fontFamily.sans,
    fontSize: typography.fontSize.base,
    fontWeight: typography.fontWeight.normal,
    lineHeight: typography.lineHeight.normal,
  },

  // Label text
  label: {
    fontFamily: typography.fontFamily.sans,
    fontSize: typography.fontSize.sm,
    fontWeight: typography.fontWeight.medium,
    lineHeight: typography.lineHeight.normal,
    textTransform: typography.textTransform.uppercase,
    letterSpacing: typography.letterSpacing.wide,
  },

  // Compact data (for tables)
  compact: {
    fontFamily: typography.fontFamily.mono,
    fontSize: typography.fontSize.sm,
    fontWeight: typography.fontWeight.normal,
    lineHeight: typography.lineHeight.tight,
  },
} as const;

export type TypographyPresetKey = keyof typeof typographyPresets;
