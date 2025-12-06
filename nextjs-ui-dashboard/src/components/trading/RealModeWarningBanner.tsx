/**
 * Real Mode Warning Banner
 *
 * Persistent warning banner displayed in real trading mode.
 * Cannot be dismissed - always visible at top of viewport.
 */

import { motion } from 'framer-motion';
import { colors } from '@/styles/tokens/colors';
import { slideDown } from '@/styles/tokens/animations';
import { useTradingMode } from '@/hooks/useTradingMode';

export function RealModeWarningBanner() {
  const { mode } = useTradingMode();

  if (mode !== 'real') {
    return null;
  }

  return (
    <motion.div
      variants={slideDown}
      initial="initial"
      animate="animate"
      exit="exit"
      className="sticky top-0 z-40 w-full px-4 py-3 text-center"
      style={{
        backgroundColor: colors.real.banner,
        borderBottom: `2px solid ${colors.real.warning}`,
      }}
    >
      <div className="flex items-center justify-center gap-2">
        <span className="text-lg">⚠️</span>
        <span
          className="font-bold text-white"
          style={{
            textShadow: '0 1px 2px rgba(0, 0, 0, 0.5)',
          }}
        >
          REAL MONEY MODE
        </span>
        <span className="text-lg">⚠️</span>
      </div>
      <p className="mt-1 text-sm text-white/90">
        All trades execute with real funds on live exchange
      </p>
    </motion.div>
  );
}
