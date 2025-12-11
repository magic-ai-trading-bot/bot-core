/**
 * Responsive Typography System
 *
 * Provides consistent, responsive text sizing across the application.
 * Based on a modular scale with fluid typography using CSS clamp().
 *
 * Usage:
 * import { responsiveText, fluidText } from '@/styles/typography';
 * <h1 className={responsiveText.display1}>Large Display</h1>
 * <p className={responsiveText.body}>Body text</p>
 */

/**
 * Responsive text classes using Tailwind breakpoints
 * Mobile-first approach: base size → sm → md → lg → xl
 */
export const responsiveText = {
  // Display text - Hero sections, large headings
  display1: 'text-3xl sm:text-4xl md:text-5xl lg:text-6xl xl:text-7xl font-bold tracking-tight',
  display2: 'text-2xl sm:text-3xl md:text-4xl lg:text-5xl xl:text-6xl font-bold tracking-tight',
  display3: 'text-xl sm:text-2xl md:text-3xl lg:text-4xl xl:text-5xl font-bold tracking-tight',

  // Headings - Section titles, card headers
  h1: 'text-2xl sm:text-3xl md:text-4xl font-bold',
  h2: 'text-xl sm:text-2xl md:text-3xl font-semibold',
  h3: 'text-lg sm:text-xl md:text-2xl font-semibold',
  h4: 'text-base sm:text-lg md:text-xl font-medium',
  h5: 'text-sm sm:text-base md:text-lg font-medium',
  h6: 'text-xs sm:text-sm md:text-base font-medium',

  // Body text
  bodyLarge: 'text-base sm:text-lg md:text-xl',
  body: 'text-sm sm:text-base',
  bodySmall: 'text-xs sm:text-sm',

  // Labels and captions
  label: 'text-xs sm:text-sm font-medium uppercase tracking-wider',
  caption: 'text-[10px] sm:text-xs',
  overline: 'text-[10px] sm:text-xs uppercase tracking-widest font-medium',

  // Stats and numbers - Dashboard values
  statLarge: 'text-2xl sm:text-3xl md:text-4xl lg:text-5xl font-bold tabular-nums',
  statMedium: 'text-xl sm:text-2xl md:text-3xl font-bold tabular-nums',
  statSmall: 'text-lg sm:text-xl md:text-2xl font-semibold tabular-nums',
  statValue: 'text-base sm:text-lg md:text-xl font-semibold tabular-nums',

  // Monospace - Code, prices, data
  mono: 'font-mono text-sm sm:text-base',
  monoSmall: 'font-mono text-xs sm:text-sm',
  monoLarge: 'font-mono text-base sm:text-lg md:text-xl',

  // Button text
  buttonLarge: 'text-base sm:text-lg font-semibold',
  button: 'text-sm sm:text-base font-medium',
  buttonSmall: 'text-xs sm:text-sm font-medium',

  // Navigation
  navItem: 'text-sm sm:text-base font-medium',
  navItemSmall: 'text-xs sm:text-sm font-medium',

  // Badge/Tag text
  badge: 'text-[10px] sm:text-xs font-semibold uppercase tracking-wide',
  tag: 'text-xs sm:text-sm font-medium',
} as const;

/**
 * Fluid typography using CSS clamp()
 * These use viewport-relative sizing for smooth scaling
 *
 * Formula: clamp(min, preferred, max)
 * preferred: base + (vw * scale)
 */
export const fluidText = {
  // Display - scales from 1.75rem to 3.5rem
  display: 'text-[clamp(1.75rem,4vw+1rem,3.5rem)]',

  // Heading - scales from 1.25rem to 2rem
  heading: 'text-[clamp(1.25rem,2vw+0.75rem,2rem)]',

  // Subheading - scales from 1rem to 1.5rem
  subheading: 'text-[clamp(1rem,1.5vw+0.5rem,1.5rem)]',

  // Body - scales from 0.875rem to 1rem
  body: 'text-[clamp(0.875rem,0.5vw+0.75rem,1rem)]',

  // Small - scales from 0.75rem to 0.875rem
  small: 'text-[clamp(0.75rem,0.25vw+0.7rem,0.875rem)]',
} as const;

/**
 * Line height utilities for better readability
 */
export const lineHeight = {
  tight: 'leading-tight',      // 1.25
  snug: 'leading-snug',        // 1.375
  normal: 'leading-normal',    // 1.5
  relaxed: 'leading-relaxed',  // 1.625
  loose: 'leading-loose',      // 2
} as const;

/**
 * Letter spacing utilities
 */
export const tracking = {
  tighter: 'tracking-tighter',  // -0.05em
  tight: 'tracking-tight',      // -0.025em
  normal: 'tracking-normal',    // 0
  wide: 'tracking-wide',        // 0.025em
  wider: 'tracking-wider',      // 0.05em
  widest: 'tracking-widest',    // 0.1em
} as const;

/**
 * Combine typography styles
 * @example combineTypography(responsiveText.h1, lineHeight.tight, tracking.tight)
 */
export function combineTypography(...classes: string[]): string {
  return classes.filter(Boolean).join(' ');
}

/**
 * Preset combinations for common use cases
 */
export const typography = {
  // Hero section
  heroTitle: combineTypography(responsiveText.display1, lineHeight.tight, tracking.tight),
  heroSubtitle: combineTypography(responsiveText.bodyLarge, lineHeight.relaxed),

  // Page headers
  pageTitle: combineTypography(responsiveText.h1, lineHeight.tight),
  pageSubtitle: combineTypography(responsiveText.body, lineHeight.normal),

  // Section headers
  sectionTitle: combineTypography(responsiveText.h2, lineHeight.snug),
  sectionSubtitle: combineTypography(responsiveText.bodySmall, lineHeight.normal),

  // Card content
  cardTitle: combineTypography(responsiveText.h4, lineHeight.snug),
  cardBody: combineTypography(responsiveText.body, lineHeight.relaxed),

  // Dashboard stats
  dashboardValue: combineTypography(responsiveText.statLarge, lineHeight.tight),
  dashboardLabel: combineTypography(responsiveText.label, lineHeight.normal),

  // Form labels
  formLabel: combineTypography(responsiveText.label, lineHeight.normal),
  formHelper: combineTypography(responsiveText.caption, lineHeight.normal),

  // Navigation
  navLink: combineTypography(responsiveText.navItem, lineHeight.normal),

  // Prices and data
  price: combineTypography(responsiveText.monoLarge, lineHeight.tight),
  priceSmall: combineTypography(responsiveText.mono, lineHeight.tight),
} as const;

export default typography;
