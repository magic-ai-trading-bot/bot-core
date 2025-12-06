/**
 * Mode Toggle
 *
 * Toggle switch for Paper/Real trading mode.
 * Paper = blue, Real = red.
 * Triggers confirmation dialog when switching to real.
 */

import { motion } from 'framer-motion';
import { useTradingMode } from '@/hooks/useTradingMode';
import { colors } from '@/styles/tokens/colors';
import logger from '@/utils/logger';

export function ModeToggle() {
  const { mode, requestModeSwitch } = useTradingMode();

  const handleToggle = () => {
    const targetMode = mode === 'paper' ? 'real' : 'paper';
    logger.info(`Mode toggle clicked: ${mode} → ${targetMode}`);
    requestModeSwitch(targetMode);
  };

  const isPaperMode = mode === 'paper';

  return (
    <div className="flex items-center gap-3">
      {/* Mode label */}
      <span
        className="text-sm font-medium"
        style={{ color: colors.text.secondary }}
      >
        Trading Mode:
      </span>

      {/* Toggle switch */}
      <button
        onClick={handleToggle}
        className="relative h-8 w-16 rounded-full transition-colors focus:outline-none focus:ring-2 focus:ring-offset-2"
        style={{
          backgroundColor: isPaperMode ? colors.paper.accent : colors.real.warning,
          focusRingColor: isPaperMode ? colors.paper.accent : colors.real.warning,
        }}
        aria-label={`Switch to ${isPaperMode ? 'real' : 'paper'} trading mode`}
      >
        {/* Toggle slider */}
        <motion.div
          layout
          transition={{
            type: 'spring',
            stiffness: 500,
            damping: 30,
          }}
          className="absolute top-1 h-6 w-6 rounded-full bg-white shadow-md"
          style={{
            left: isPaperMode ? '4px' : 'calc(100% - 28px)',
          }}
        />
      </button>

      {/* Mode text */}
      <div className="flex items-center gap-2">
        <span
          className="text-sm font-semibold"
          style={{
            color: isPaperMode ? colors.paper.accent : colors.real.warning,
          }}
        >
          {isPaperMode ? '📊 Paper' : '🔴 Real'}
        </span>
        {!isPaperMode && (
          <motion.span
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            className="rounded px-2 py-0.5 text-xs font-bold text-white"
            style={{
              backgroundColor: colors.real.warning,
            }}
          >
            LIVE
          </motion.span>
        )}
      </div>
    </div>
  );
}
