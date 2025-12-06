/**
 * ModeBadge Component
 *
 * Visual indicator for Paper/Real trading mode.
 * Paper: Blue badge with "PAPER" label
 * Real: Red pulsing badge with "REAL MONEY" label
 */

import { motion } from 'framer-motion';
import { cn } from '@/lib/utils';
import { TradingMode, getTheme } from '@/styles';

interface ModeBadgeProps {
  mode: TradingMode;
  className?: string;
  size?: 'sm' | 'md' | 'lg';
  showIcon?: boolean;
}

export function ModeBadge({
  mode,
  className,
  size = 'md',
  showIcon = true,
}: ModeBadgeProps) {
  const theme = getTheme(mode);

  const sizeClasses = {
    sm: 'px-2 py-0.5 text-xs',
    md: 'px-3 py-1 text-sm',
    lg: 'px-4 py-1.5 text-base',
  };

  return (
    <motion.div
      animate={theme.badge.pulse ? { scale: [1, 1.05, 1], opacity: [1, 0.8, 1] } : {}}
      transition={
        theme.badge.pulse
          ? { duration: 2, repeat: Infinity, ease: 'easeInOut' }
          : undefined
      }
      className={cn(
        'inline-flex items-center gap-1.5 rounded-full font-semibold uppercase tracking-wide',
        sizeClasses[size],
        mode === 'paper'
          ? 'bg-[#0284C7] text-white'
          : 'bg-[#DC2626] text-white',
        className
      )}
      role="status"
      aria-label={`Trading mode: ${mode}`}
    >
      {showIcon && <span aria-hidden="true">{theme.banner.icon}</span>}
      <span>{theme.badge.label}</span>
    </motion.div>
  );
}

// Variant: Banner mode (full width)
interface ModeBannerProps {
  mode: TradingMode;
  className?: string;
  onDismiss?: () => void;
}

export function ModeBanner({ mode, className, onDismiss }: ModeBannerProps) {
  const theme = getTheme(mode);

  return (
    <motion.div
      initial={{ opacity: 0, y: -20 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ duration: 0.3 }}
      className={cn(
        'flex items-center justify-between px-4 py-2 border-b',
        mode === 'paper'
          ? 'bg-[#0EA5E9]/10 border-[#0EA5E9]/20 text-[#0EA5E9]'
          : 'bg-[#EF4444]/10 border-[#EF4444]/30 text-[#EF4444]',
        className
      )}
      role="banner"
      aria-live="polite"
    >
      <div className="flex items-center gap-2">
        <span className="text-lg" aria-hidden="true">
          {theme.banner.icon}
        </span>
        <span className="text-sm font-medium">{theme.banner.message}</span>
      </div>

      {onDismiss && (
        <button
          onClick={onDismiss}
          className="text-current hover:opacity-70 transition-opacity"
          aria-label="Dismiss banner"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            className="h-5 w-5"
            viewBox="0 0 20 20"
            fill="currentColor"
          >
            <path
              fillRule="evenodd"
              d="M4.293 4.293a1 1 0 011.414 0L10 8.586l4.293-4.293a1 1 0 111.414 1.414L11.414 10l4.293 4.293a1 1 0 01-1.414 1.414L10 11.414l-4.293 4.293a1 1 0 01-1.414-1.414L8.586 10 4.293 5.707a1 1 0 010-1.414z"
              clipRule="evenodd"
            />
          </svg>
        </button>
      )}
    </motion.div>
  );
}
