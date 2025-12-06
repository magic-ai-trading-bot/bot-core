/**
 * useSidebar Hook
 *
 * Manages sidebar state (expanded/collapsed/mobile) with localStorage persistence.
 * Auto-collapses on mobile viewports for better UX.
 */

import { useState, useEffect, useCallback } from 'react';

type SidebarState = 'expanded' | 'collapsed';

interface UseSidebarReturn {
  isExpanded: boolean;
  isCollapsed: boolean;
  isMobile: boolean;
  isMobileOpen: boolean;
  toggle: () => void;
  expand: () => void;
  collapse: () => void;
  openMobile: () => void;
  closeMobile: () => void;
}

const STORAGE_KEY = 'sidebar-state';
const MOBILE_BREAKPOINT = 768; // md breakpoint

export function useSidebar(): UseSidebarReturn {
  // Load initial state from localStorage or default to 'expanded'
  const [state, setState] = useState<SidebarState>(() => {
    if (typeof window === 'undefined') return 'expanded';

    const saved = localStorage.getItem(STORAGE_KEY);
    return (saved === 'collapsed' || saved === 'expanded') ? saved : 'expanded';
  });

  const [isMobile, setIsMobile] = useState(false);
  const [isMobileOpen, setIsMobileOpen] = useState(false);

  // Detect mobile viewport
  useEffect(() => {
    const checkMobile = () => {
      const mobile = window.innerWidth < MOBILE_BREAKPOINT;
      setIsMobile(mobile);

      // Auto-collapse on mobile
      if (mobile && state === 'expanded') {
        setState('collapsed');
      }
    };

    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, [state]);

  // Persist state to localStorage
  useEffect(() => {
    localStorage.setItem(STORAGE_KEY, state);
  }, [state]);

  const toggle = useCallback(() => {
    setState(prev => prev === 'expanded' ? 'collapsed' : 'expanded');
  }, []);

  const expand = useCallback(() => {
    setState('expanded');
  }, []);

  const collapse = useCallback(() => {
    setState('collapsed');
  }, []);

  const openMobile = useCallback(() => {
    setIsMobileOpen(true);
  }, []);

  const closeMobile = useCallback(() => {
    setIsMobileOpen(false);
  }, []);

  return {
    isExpanded: state === 'expanded',
    isCollapsed: state === 'collapsed',
    isMobile,
    isMobileOpen,
    toggle,
    expand,
    collapse,
    openMobile,
    closeMobile,
  };
}
