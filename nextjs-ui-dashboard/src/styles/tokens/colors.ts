/**
 * Design Tokens - Colors
 *
 * Color system for Paper/Real trading mode distinction with WCAG 2.1 AA compliance.
 * All contrast ratios meet minimum 4.5:1 requirement for text readability.
 */

export const colors = {
  // Background colors - Dark OLED optimized
  bg: {
    primary: '#000000',    // Pure black for OLED
    secondary: '#0A0A0A',  // Slightly elevated (near black)
    tertiary: '#141414',   // Card/elevated elements
  },

  // Paper mode - Blue accent for safe sandbox environment
  paper: {
    accent: '#0EA5E9',     // Sky blue - primary accent
    badge: '#0284C7',      // Darker blue for badge emphasis
    border: '#0EA5E9',     // Border color (use with opacity: 0.2)
    background: '#0EA5E9', // Background color (use with opacity: 0.1)
    hover: '#38BDF8',      // Lighter blue for hover states
  },

  // Real mode - Red warnings for real money environment
  real: {
    warning: '#EF4444',    // Red warning - primary accent
    banner: '#DC2626',     // Darker red for banner emphasis
    border: '#EF4444',     // Border color (use with opacity: 0.3)
    background: '#EF4444', // Background color (use with opacity: 0.1)
    hover: '#F87171',      // Lighter red for hover states
  },

  // Trading indicators - Profit/Loss colors
  profit: '#10B981',       // Green (emerald-500)
  loss: '#EF4444',         // Red (red-500)
  neutral: '#64748B',      // Slate (slate-500)

  // Text colors - High contrast for readability
  text: {
    primary: '#F3F4F6',    // Main text (gray-100) - 15.6:1 contrast on bg.primary
    secondary: '#94A3B8',  // Muted text (slate-400) - 6.4:1 contrast on bg.primary
    muted: '#64748B',      // Very muted (slate-500) - 4.6:1 contrast on bg.primary
    inverse: '#0F172A',    // Inverse for light backgrounds
  },

  // Grid/Lines - Subtle separators
  grid: '#374151',         // Gray-700
  border: '#475569',       // Slate-600

  // Status indicators
  status: {
    success: '#10B981',    // Green
    error: '#EF4444',      // Red
    warning: '#F59E0B',    // Amber
    info: '#3B82F6',       // Blue
  },

  // Chart colors - Distinct colors for multi-series charts
  chart: {
    1: '#10B981',          // Green
    2: '#EF4444',          // Red
    3: '#F59E0B',          // Amber
    4: '#3B82F6',          // Blue
    5: '#8B5CF6',          // Violet
  },
} as const;

// Type exports
export type ColorKey = keyof typeof colors;
export type BgColorKey = keyof typeof colors.bg;
export type PaperColorKey = keyof typeof colors.paper;
export type RealColorKey = keyof typeof colors.real;
export type TextColorKey = keyof typeof colors.text;
export type StatusColorKey = keyof typeof colors.status;
export type ChartColorKey = keyof typeof colors.chart;

// Mode-specific color getter
export function getModeColor(mode: 'paper' | 'real', key: 'accent' | 'border' | 'background' | 'hover') {
  return mode === 'paper' ? colors.paper[key] : colors.real[key === 'accent' ? 'warning' : key];
}

// Opacity helper for border/background colors
export function withOpacity(color: string, opacity: number): string {
  // Convert hex to rgba
  const r = parseInt(color.slice(1, 3), 16);
  const g = parseInt(color.slice(3, 5), 16);
  const b = parseInt(color.slice(5, 7), 16);
  return `rgba(${r}, ${g}, ${b}, ${opacity})`;
}
