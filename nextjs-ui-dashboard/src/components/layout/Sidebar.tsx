/**
 * Sidebar Component
 *
 * Collapsible sidebar with glassmorphism effect, logo animation, and mobile drawer support.
 * Desktop: Transitions smoothly between expanded (256px) and collapsed (64px) states.
 * Mobile: Full-page slide-in drawer from left.
 */

import { motion, AnimatePresence } from 'framer-motion';
import { ChevronLeft, ChevronRight, X } from 'lucide-react';
import { cn } from '@/lib/utils';
import { SidebarNav } from './SidebarNav';
import { duration, easing } from '@/styles/tokens/animations';
import { useSidebar } from '@/hooks/useSidebar';
import { useThemeColors } from '@/hooks/useThemeColors';
import { BotCoreLogo } from '@/components/BotCoreLogo';

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
  const colors = useThemeColors();

  // Mobile full-page drawer
  if (isMobile) {
    return (
      <AnimatePresence>
        {isMobileOpen && (
          <>
            {/* Backdrop - dark overlay */}
            <motion.div
              initial={{ opacity: 0 }}
              animate={{ opacity: 1 }}
              exit={{ opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="fixed inset-0 z-40 bg-black/70"
              onClick={closeMobile}
            />

            {/* Full-page drawer from left */}
            <motion.div
              initial={{ x: '-100%' }}
              animate={{ x: 0 }}
              exit={{ x: '-100%' }}
              transition={{
                type: 'spring',
                damping: 30,
                stiffness: 300,
              }}
              className="fixed inset-y-0 left-0 z-50 w-[85vw] max-w-[320px] flex flex-col"
              style={{
                backgroundColor: colors.bgPrimary,
                boxShadow: '4px 0 24px rgba(0, 0, 0, 0.5)',
              }}
            >
              {/* Header with Logo and Close button */}
              <div
                className="flex h-16 items-center justify-between px-5 flex-shrink-0"
                style={{ borderBottom: `1px solid ${colors.borderSubtle}` }}
              >
                <BotCoreLogo size="sm" />

                <button
                  onClick={closeMobile}
                  className={cn(
                    'flex h-10 w-10 items-center justify-center rounded-xl',
                    'transition-all duration-200 active:scale-95'
                  )}
                  style={{
                    backgroundColor: colors.bgSecondary,
                    color: colors.textMuted,
                  }}
                  aria-label="Close menu"
                >
                  <X className="h-5 w-5" />
                </button>
              </div>

              {/* Navigation - scrollable */}
              <div className="flex-1 overflow-y-auto">
                <SidebarNav
                  isExpanded={true}
                  onNavigate={closeMobile}
                />
              </div>

              {/* Footer */}
              <div
                className="p-5 flex-shrink-0"
                style={{ borderTop: `1px solid ${colors.borderSubtle}` }}
              >
                <p className="text-xs" style={{ color: colors.textMuted }}>
                  v1.0.0
                </p>
              </div>
            </motion.div>
          </>
        )}
      </AnimatePresence>
    );
  }

  // Desktop sidebar
  return (
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
        backgroundColor: colors.bgPrimary,
        borderRight: `1px solid ${colors.borderSubtle}`,
        boxShadow: colors.bgPrimary === '#000000' ? '0 8px 32px rgba(0, 0, 0, 0.8)' : '0 8px 32px rgba(0, 0, 0, 0.1)',
      }}
    >
      {/* Header with Logo and Toggle */}
      <div
        className="flex h-16 items-center justify-between px-4"
        style={{ borderBottom: `1px solid ${colors.borderSubtle}` }}
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
          className="flex items-center overflow-hidden whitespace-nowrap"
        >
          <BotCoreLogo size="sm" />
        </motion.div>

        {/* Toggle button */}
        <motion.button
          onClick={toggle}
          whileHover={{ scale: 1.1, backgroundColor: colors.bgHover }}
          whileTap={{ scale: 0.9 }}
          className={cn(
            'flex h-8 w-8 items-center justify-center rounded-lg',
            'transition-all duration-200'
          )}
          style={{
            backgroundColor: colors.bgSecondary,
            color: colors.textMuted,
            border: `1px solid ${colors.borderSubtle}`,
          }}
          aria-label={isExpanded ? 'Collapse sidebar' : 'Expand sidebar'}
        >
          {isExpanded ? (
            <ChevronLeft className="h-4 w-4" />
          ) : (
            <ChevronRight className="h-4 w-4" />
          )}
        </motion.button>
      </div>

      {/* Navigation */}
      <SidebarNav isExpanded={isExpanded} />

      {/* Footer */}
      <div
        className="p-4"
        style={{ borderTop: `1px solid ${colors.borderSubtle}` }}
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
          style={{ color: colors.textMuted }}
        >
          v1.0.0
        </motion.div>
      </div>
    </motion.aside>
  );
}
