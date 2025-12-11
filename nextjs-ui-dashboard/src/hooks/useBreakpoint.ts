/**
 * useBreakpoint Hook
 *
 * Returns current Tailwind breakpoint name based on window width.
 * Debounced for performance, SSR-safe.
 */

import { useState, useEffect, useCallback } from 'react';

export type Breakpoint = 'xs' | 'sm' | 'md' | 'lg' | 'xl' | '2xl';

// Tailwind CSS default breakpoints
const breakpoints = {
  xs: 0,     // Extra small phones
  sm: 640,   // Small phones & larger
  md: 768,   // Tablets
  lg: 1024,  // Laptops
  xl: 1280,  // Desktops
  '2xl': 1536, // Large desktops
};

function getBreakpoint(width: number): Breakpoint {
  if (width >= breakpoints['2xl']) return '2xl';
  if (width >= breakpoints.xl) return 'xl';
  if (width >= breakpoints.lg) return 'lg';
  if (width >= breakpoints.md) return 'md';
  if (width >= breakpoints.sm) return 'sm';
  return 'xs';
}

export function useBreakpoint(): Breakpoint {
  // SSR-safe: default to 'md' on server
  const [breakpoint, setBreakpoint] = useState<Breakpoint>(() => {
    if (typeof window === 'undefined') return 'md';
    return getBreakpoint(window.innerWidth);
  });

  const handleResize = useCallback(() => {
    setBreakpoint(getBreakpoint(window.innerWidth));
  }, []);

  useEffect(() => {
    // Set initial breakpoint after hydration
    handleResize();

    // Debounced resize listener
    let timeoutId: ReturnType<typeof setTimeout>;
    const debouncedResize = () => {
      clearTimeout(timeoutId);
      timeoutId = setTimeout(handleResize, 100);
    };

    window.addEventListener('resize', debouncedResize);
    return () => {
      window.removeEventListener('resize', debouncedResize);
      clearTimeout(timeoutId);
    };
  }, [handleResize]);

  return breakpoint;
}

/**
 * Utility functions for breakpoint comparisons
 */
export function useIsBreakpoint(target: Breakpoint): boolean {
  const current = useBreakpoint();
  return current === target;
}

export function useIsBreakpointUp(target: Breakpoint): boolean {
  const current = useBreakpoint();
  const order: Breakpoint[] = ['xs', 'sm', 'md', 'lg', 'xl', '2xl'];
  return order.indexOf(current) >= order.indexOf(target);
}

export function useIsBreakpointDown(target: Breakpoint): boolean {
  const current = useBreakpoint();
  const order: Breakpoint[] = ['xs', 'sm', 'md', 'lg', 'xl', '2xl'];
  return order.indexOf(current) <= order.indexOf(target);
}

export default useBreakpoint;
