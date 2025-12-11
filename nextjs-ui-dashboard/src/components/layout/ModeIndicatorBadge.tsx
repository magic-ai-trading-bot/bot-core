/**
 * ModeIndicatorBadge Component
 *
 * Always-visible trading mode indicator in header.
 * Paper mode = blue badge, Real mode = red badge with pulse animation.
 * This is a simplified wrapper around Phase 1's design tokens.
 */

import { motion } from 'framer-motion';
import { TestTube, CircleDollarSign } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { cn } from '@/lib/utils';
import { colors } from '@/styles/tokens/colors';
import { duration } from '@/styles/tokens/animations';

interface ModeIndicatorBadgeProps {
  mode: 'paper' | 'real';
  className?: string;
  /** Hide text label on mobile - show icon only */
  compact?: boolean;
}

export function ModeIndicatorBadge({ mode, className, compact = false }: ModeIndicatorBadgeProps) {
  const { t } = useTranslation('common');
  const isPaper = mode === 'paper';

  return (
    <motion.div
      initial={{ scale: 0.9, opacity: 0 }}
      animate={{ scale: 1, opacity: 1 }}
      transition={{ duration: duration.normal }}
      className={cn(
        'inline-flex items-center rounded-full text-sm font-medium',
        'border transition-all',
        // Compact mode: smaller padding, no gap (icon only)
        compact ? 'p-2' : 'gap-2 px-3 py-1.5',
        className
      )}
      style={{
        backgroundColor: isPaper
          ? `${colors.paper.background}20`
          : `${colors.real.background}20`,
        borderColor: isPaper
          ? `${colors.paper.border}40`
          : `${colors.real.border}40`,
        color: isPaper ? colors.paper.accent : colors.real.warning,
      }}
    >
      {/* Icon */}
      <motion.div
        animate={
          isPaper
            ? {}
            : {
                scale: [1, 1.1, 1],
                rotate: [0, -5, 5, 0],
              }
        }
        transition={
          isPaper
            ? {}
            : {
                duration: 2,
                repeat: Infinity,
                ease: 'easeInOut',
              }
        }
      >
        {isPaper ? (
          <TestTube className="h-4 w-4" />
        ) : (
          <CircleDollarSign className="h-4 w-4" />
        )}
      </motion.div>

      {/* Label - hidden in compact mode */}
      {!compact && (
        <span className="font-semibold">
          {isPaper ? t('mode.paper') : t('mode.real')}
        </span>
      )}

      {/* Pulse dot for real mode - hidden in compact mode */}
      {!isPaper && !compact && (
        <motion.div
          animate={{
            scale: [1, 1.3, 1],
            opacity: [1, 0.5, 1],
          }}
          transition={{
            duration: 1.5,
            repeat: Infinity,
            ease: 'easeInOut',
          }}
          className="h-2 w-2 rounded-full"
          style={{ backgroundColor: colors.real.warning }}
        />
      )}
    </motion.div>
  );
}
