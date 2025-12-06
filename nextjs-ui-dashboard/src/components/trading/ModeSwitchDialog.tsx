/**
 * Mode Switch Dialog
 *
 * Confirmation dialog for switching to real trading mode.
 * Requires explicit checkbox confirmation before enabling.
 */

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTradingMode } from '@/hooks/useTradingMode';
import { colors } from '@/styles/tokens/colors';
import { scaleIn } from '@/styles/tokens/animations';
import logger from '@/utils/logger';

export function ModeSwitchDialog() {
  const { isModeSwitchOpen, closeModeSwitchDialog, confirmModeSwitch, pendingMode } = useTradingMode();
  const [isConfirmed, setIsConfirmed] = useState(false);

  // Reset checkbox when dialog opens
  useEffect(() => {
    if (isModeSwitchOpen) {
      setIsConfirmed(false);
    }
  }, [isModeSwitchOpen]);

  if (!isModeSwitchOpen || pendingMode !== 'real') {
    return null;
  }

  const handleConfirm = () => {
    if (!isConfirmed) {
      logger.warn('Cannot confirm mode switch: checkbox not checked');
      return;
    }

    confirmModeSwitch();
  };

  const handleCancel = () => {
    closeModeSwitchDialog();
  };

  return (
    <AnimatePresence>
      {isModeSwitchOpen && (
        <>
          {/* Backdrop */}
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            transition={{ duration: 0.2 }}
            className="fixed inset-0 z-50 bg-black/80"
            onClick={handleCancel}
          />

          {/* Dialog */}
          <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            <motion.div
              variants={scaleIn}
              initial="initial"
              animate="animate"
              exit="exit"
              className="relative w-full max-w-md rounded-lg p-6"
              style={{
                backgroundColor: colors.bg.secondary,
                border: `1px solid ${colors.border}`,
              }}
              onClick={(e) => e.stopPropagation()}
            >
              {/* Title */}
              <div className="mb-4">
                <h2
                  className="text-2xl font-bold"
                  style={{ color: colors.real.warning }}
                >
                  ⚠️ Switch to Real Trading?
                </h2>
              </div>

              {/* Warning text */}
              <div className="mb-6 space-y-3">
                <p style={{ color: colors.text.primary }}>
                  You are about to switch to <strong>Real Trading Mode</strong>.
                </p>
                <p style={{ color: colors.text.secondary }}>
                  All trades will be executed with <strong>real money</strong> on the live exchange.
                  This is not a simulation.
                </p>
                <div
                  className="rounded-md p-3"
                  style={{
                    backgroundColor: `${colors.real.warning}20`,
                    border: `1px solid ${colors.real.warning}40`,
                  }}
                >
                  <p
                    className="text-sm font-semibold"
                    style={{ color: colors.real.warning }}
                  >
                    ⚠️ Warning: You can lose real money
                  </p>
                  <p
                    className="mt-1 text-sm"
                    style={{ color: colors.text.secondary }}
                  >
                    Only enable this if you understand the risks and have tested your strategies thoroughly.
                  </p>
                </div>
              </div>

              {/* Confirmation checkbox */}
              <label
                className="mb-6 flex cursor-pointer items-start gap-3"
                onClick={(e) => e.stopPropagation()}
              >
                <input
                  type="checkbox"
                  checked={isConfirmed}
                  onChange={(e) => setIsConfirmed(e.target.checked)}
                  className="mt-1 h-4 w-4 cursor-pointer rounded"
                  style={{
                    accentColor: colors.real.warning,
                  }}
                />
                <span style={{ color: colors.text.primary }}>
                  I understand this involves real money and accept the risks
                </span>
              </label>

              {/* Action buttons */}
              <div className="flex gap-3">
                <button
                  onClick={handleCancel}
                  className="flex-1 rounded-md px-4 py-2 font-medium transition-colors"
                  style={{
                    backgroundColor: colors.bg.tertiary,
                    color: colors.text.primary,
                  }}
                  onMouseEnter={(e) => {
                    e.currentTarget.style.backgroundColor = colors.border;
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.backgroundColor = colors.bg.tertiary;
                  }}
                >
                  Cancel
                </button>
                <button
                  onClick={handleConfirm}
                  disabled={!isConfirmed}
                  className="flex-1 rounded-md px-4 py-2 font-medium transition-colors disabled:cursor-not-allowed disabled:opacity-50"
                  style={{
                    backgroundColor: isConfirmed ? colors.real.warning : colors.bg.tertiary,
                    color: isConfirmed ? '#FFFFFF' : colors.text.muted,
                  }}
                  onMouseEnter={(e) => {
                    if (isConfirmed) {
                      e.currentTarget.style.backgroundColor = colors.real.hover;
                    }
                  }}
                  onMouseLeave={(e) => {
                    if (isConfirmed) {
                      e.currentTarget.style.backgroundColor = colors.real.warning;
                    }
                  }}
                >
                  Confirm Switch
                </button>
              </div>

              {/* Additional info */}
              <p
                className="mt-4 text-center text-xs"
                style={{ color: colors.text.muted }}
              >
                You can switch back to Paper Mode at any time without confirmation.
              </p>
            </motion.div>
          </div>
        </>
      )}
    </AnimatePresence>
  );
}
