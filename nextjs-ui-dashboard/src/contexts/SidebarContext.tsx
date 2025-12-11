/**
 * SidebarContext
 *
 * Shared state for sidebar (expanded/collapsed/mobile) across Header and Sidebar components.
 */

import React, { createContext, useContext, useState, useEffect, useCallback, ReactNode } from 'react';

type SidebarState = 'expanded' | 'collapsed';

interface SidebarContextType {
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

const SidebarContext = createContext<SidebarContextType | undefined>(undefined);

const STORAGE_KEY = 'sidebar-state';
const MOBILE_BREAKPOINT = 768; // md breakpoint

interface SidebarProviderProps {
  children: ReactNode;
}

export function SidebarProvider({ children }: SidebarProviderProps) {
  // Load initial state from localStorage or default to 'expanded'
  const [state, setState] = useState<SidebarState>(() => {
    if (typeof window === 'undefined') return 'expanded';

    const saved = localStorage.getItem(STORAGE_KEY);
    return (saved === 'collapsed' || saved === 'expanded') ? saved : 'expanded';
  });

  const [isMobile, setIsMobile] = useState(() => {
    if (typeof window === 'undefined') return false;
    return window.innerWidth < MOBILE_BREAKPOINT;
  });
  const [isMobileOpen, setIsMobileOpen] = useState(false);

  // Detect mobile viewport
  useEffect(() => {
    const checkMobile = () => {
      const mobile = window.innerWidth < MOBILE_BREAKPOINT;
      setIsMobile(mobile);

      // Auto-close mobile menu when switching to desktop
      if (!mobile && isMobileOpen) {
        setIsMobileOpen(false);
      }
    };

    checkMobile();
    window.addEventListener('resize', checkMobile);
    return () => window.removeEventListener('resize', checkMobile);
  }, [isMobileOpen]);

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

  const value: SidebarContextType = {
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

  return (
    <SidebarContext.Provider value={value}>
      {children}
    </SidebarContext.Provider>
  );
}

export function useSidebar(): SidebarContextType {
  const context = useContext(SidebarContext);
  if (context === undefined) {
    throw new Error('useSidebar must be used within a SidebarProvider');
  }
  return context;
}
