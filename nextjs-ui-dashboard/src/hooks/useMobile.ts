/**
 * useMobile Hook
 *
 * Simple boolean hook for mobile detection.
 * Mobile = viewport width < 768px (md breakpoint)
 */

import { useMediaQuery, mediaQueries } from './useMediaQuery';

export function useMobile(): boolean {
  return useMediaQuery(mediaQueries.mobile);
}

export function useTablet(): boolean {
  return useMediaQuery(mediaQueries.tablet);
}

export function useDesktop(): boolean {
  return useMediaQuery(mediaQueries.desktop);
}

/**
 * Check if device supports touch
 */
export function useIsTouchDevice(): boolean {
  return useMediaQuery(mediaQueries.coarsePointer);
}

/**
 * Check if user prefers reduced motion
 */
export function usePrefersReducedMotion(): boolean {
  return useMediaQuery(mediaQueries.prefersReducedMotion);
}

export default useMobile;
