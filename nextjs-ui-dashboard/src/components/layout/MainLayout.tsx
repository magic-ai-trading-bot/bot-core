/**
 * MainLayout Component
 *
 * Main application layout wrapper combining Sidebar + Header + Content area.
 * Responsive grid layout with proper spacing and overflow handling.
 */

import { ReactNode } from 'react';
import { motion } from 'framer-motion';
import { Sidebar } from './Sidebar';
import { Header } from './Header';
import { ModeSwitchDialog } from '@/components/trading/ModeSwitchDialog';
import { useSidebar } from '@/hooks/useSidebar';
import { useThemeColors } from '@/hooks/useThemeColors';
import { duration, easing } from '@/styles/tokens/animations';

interface MainLayoutProps {
  children: ReactNode;
}

export function MainLayout({ children }: MainLayoutProps) {
  const { isExpanded, isMobile } = useSidebar();
  const colors = useThemeColors();

  return (
    <div className="flex h-screen overflow-hidden" style={{ backgroundColor: colors.bgPrimary }}>
      {/* Sidebar */}
      <Sidebar />

      {/* Mode switch confirmation dialog */}
      <ModeSwitchDialog />

      {/* Main content area */}
      <motion.div
        initial={false}
        animate={{
          marginLeft: isMobile ? 0 : isExpanded ? 0 : 0,
        }}
        transition={{
          duration: duration.normal,
          ease: easing.easeOut,
        }}
        className="flex flex-1 flex-col overflow-hidden"
      >
        {/* Header */}
        <Header />

        {/* Page content */}
        <main className="flex-1 overflow-y-auto overflow-x-hidden">
          <motion.div
            initial={{ opacity: 0, y: 20 }}
            animate={{ opacity: 1, y: 0 }}
            exit={{ opacity: 0, y: -20 }}
            transition={{
              duration: duration.normal,
              ease: easing.easeOut,
            }}
            className="h-full"
          >
            {children}
          </motion.div>
        </main>
      </motion.div>
    </div>
  );
}
