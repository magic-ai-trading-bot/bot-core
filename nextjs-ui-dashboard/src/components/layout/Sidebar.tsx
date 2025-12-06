/**
 * Sidebar Component
 *
 * Collapsible sidebar with glassmorphism effect, logo animation, and mobile drawer support.
 * Transitions smoothly between expanded (256px) and collapsed (64px) states.
 */

import { motion, AnimatePresence } from 'framer-motion';
import { ChevronLeft, ChevronRight, X } from 'lucide-react';
import { cn } from '@/lib/utils';
import { SidebarNav } from './SidebarNav';
import { duration, easing } from '@/styles/tokens/animations';
import { luxuryColors } from '@/styles/luxury-design-system';
import { useSidebar } from '@/hooks/useSidebar';

interface SidebarProps {
  className?: string;
}

export function Sidebar({ className }: SidebarProps) {
  const {
    isExpanded,
    isMobile,
    isMobileOpen,
    toggle,
    closeMobile,
  } = useSidebar();

  // Desktop sidebar
  const sidebarContent = (
    <motion.aside
      initial={false}
      animate={{
        width: isExpanded ? 256 : 64,
      }}
      transition={{
        duration: duration.normal,
        ease: easing.easeOut,
      }}
      className={cn(
        'flex flex-col',
        'backdrop-blur-xl',
        className
      )}
      style={{
        backgroundColor: luxuryColors.bgPrimary,
        borderRight: `1px solid ${luxuryColors.borderSubtle}`,
        boxShadow: '0 8px 32px rgba(0, 0, 0, 0.8)',
      }}
    >
      {/* Header with Logo and Toggle */}
      <div
        className="flex h-16 items-center justify-between px-4"
        style={{ borderBottom: `1px solid ${luxuryColors.borderSubtle}` }}
      >
        <motion.div
          initial={false}
          animate={{
            opacity: isExpanded ? 1 : 0,
            scale: isExpanded ? 1 : 0.8,
          }}
          transition={{
            duration: duration.fast,
            ease: easing.easeOut,
          }}
          className="flex items-center gap-3 overflow-hidden whitespace-nowrap"
        >
          <div
            className="flex h-9 w-9 items-center justify-center rounded-xl font-black text-sm text-white"
            style={{
              background: luxuryColors.gradientPremium,
              boxShadow: luxuryColors.glowCyan,
            }}
          >
            BT
          </div>
          <span
            className="text-lg font-black tracking-tight"
            style={{ color: luxuryColors.textPrimary }}
          >
            BotCore
          </span>
        </motion.div>

        {/* Toggle button - only show on desktop */}
        {!isMobile && (
          <motion.button
            onClick={toggle}
            whileHover={{ scale: 1.1, backgroundColor: luxuryColors.bgHover }}
            whileTap={{ scale: 0.9 }}
            className={cn(
              'flex h-8 w-8 items-center justify-center rounded-lg',
              'transition-all duration-200'
            )}
            style={{
              backgroundColor: luxuryColors.bgSecondary,
              color: luxuryColors.textMuted,
              border: `1px solid ${luxuryColors.borderSubtle}`,
            }}
            aria-label={isExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
          >
            {isExpanded ? (
              <ChevronLeft className="h-4 w-4" />
            ) : (
              <ChevronRight className="h-4 w-4" />
            )}
          </motion.button>
        )}
      </div>

      {/* Navigation */}
      <SidebarNav
        isExpanded={isExpanded}
        onNavigate={isMobile ? closeMobile : undefined}
      />

      {/* Footer - User info section (placeholder for now) */}
      <div
        className="p-4"
        style={{ borderTop: `1px solid ${luxuryColors.borderSubtle}` }}
      >
        <motion.div
          initial={false}
          animate={{
            opacity: isExpanded ? 1 : 0,
          }}
          transition={{
            duration: duration.fast,
            ease: easing.easeOut,
          }}
          className="text-xs overflow-hidden whitespace-nowrap"
          style={{ color: luxuryColors.textMuted }}
        >
          v1.0.0
        </motion.div>
      </div>
    </motion.aside>
  );

  // Mobile drawer overlay
  if (isMobile) {
    return (
      <>
        <AnimatePresence>
          {isMobileOpen && (
            <>
              {/* Backdrop */}
              <motion.div
                initial={{ opacity: 0 }}
                animate={{ opacity: 1 }}
                exit={{ opacity: 0 }}
                transition={{ duration: duration.fast }}
                className="fixed inset-0 z-40 bg-black/60 backdrop-blur-sm"
                onClick={closeMobile}
              />

              {/* Drawer */}
              <motion.div
                initial={{ x: -256 }}
                animate={{ x: 0 }}
                exit={{ x: -256 }}
                transition={{
                  duration: duration.normal,
                  ease: easing.easeOut,
                }}
                className="fixed inset-y-0 left-0 z-50 w-64"
              >
                {/* Close button */}
                <button
                  onClick={closeMobile}
                  className="absolute right-4 top-4 z-10 flex h-8 w-8 items-center justify-center rounded-lg bg-white/10 text-white hover:bg-white/20 transition-colors"
                  aria-label="Close menu"
                >
                  <X className="h-5 w-5" />
                </button>
                {sidebarContent}
              </motion.div>
            </>
          )}
        </AnimatePresence>
      </>
    );
  }

  // Desktop sidebar
  return sidebarContent;
}
