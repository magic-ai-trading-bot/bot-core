/**
 * useMediaQuery Hook
 *
 * Generic media query hook for custom responsive logic.
 * SSR-safe, debounced for performance.
 */

import { useState, useEffect } from 'react';

export function useMediaQuery(query: string): boolean {
  // SSR-safe: default to false on server
  const [matches, setMatches] = useState(() => {
    if (typeof window === 'undefined') return false;
    return window.matchMedia(query).matches;
  });

  useEffect(() => {
    if (typeof window === 'undefined') return;

    const mediaQuery = window.matchMedia(query);

    // Set initial value after hydration
    setMatches(mediaQuery.matches);

    // Modern listener with MediaQueryListEvent
    const handler = (event: MediaQueryListEvent) => {
      setMatches(event.matches);
    };

    // Use addEventListener for modern browsers
    mediaQuery.addEventListener('change', handler);

    return () => {
      mediaQuery.removeEventListener('change', handler);
    };
  }, [query]);

  return matches;
}

/**
 * Common media query presets
 */
export const mediaQueries = {
  // Breakpoint-based
  xs: '(max-width: 639px)',
  sm: '(min-width: 640px)',
  md: '(min-width: 768px)',
  lg: '(min-width: 1024px)',
  xl: '(min-width: 1280px)',
  '2xl': '(min-width: 1536px)',

  // Device-based
  mobile: '(max-width: 767px)',
  tablet: '(min-width: 768px) and (max-width: 1023px)',
  desktop: '(min-width: 1024px)',

  // Orientation
  portrait: '(orientation: portrait)',
  landscape: '(orientation: landscape)',

  // Preferences
  prefersReducedMotion: '(prefers-reduced-motion: reduce)',
  prefersDark: '(prefers-color-scheme: dark)',
  prefersLight: '(prefers-color-scheme: light)',

  // Touch
  coarsePointer: '(pointer: coarse)', // Touch device
  finePointer: '(pointer: fine)', // Mouse/trackpad

  // Hover
  canHover: '(hover: hover)',
  cannotHover: '(hover: none)',
};

export default useMediaQuery;
