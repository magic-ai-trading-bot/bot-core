/**
 * Design System Exports
 *
 * Central export file for all design tokens and themes.
 */

// Tokens
export * from './tokens/colors';
export * from './tokens/typography';
export * from './tokens/spacing';
export * from './tokens/animations';

// Themes
export * from './themes/paper-theme';
export * from './themes/real-theme';

// Theme selector utility
import { paperTheme, PaperTheme } from './themes/paper-theme';
import { realTheme, RealTheme } from './themes/real-theme';

export type TradingMode = 'paper' | 'real';
export type Theme = PaperTheme | RealTheme;

/**
 * Get theme configuration based on trading mode
 */
export function getTheme(mode: TradingMode): Theme {
  return mode === 'paper' ? paperTheme : realTheme;
}

/**
 * Check if current mode is paper trading
 */
export function isPaperMode(mode: TradingMode): mode is 'paper' {
  return mode === 'paper';
}

/**
 * Check if current mode is real trading
 */
export function isRealMode(mode: TradingMode): mode is 'real' {
  return mode === 'real';
}
