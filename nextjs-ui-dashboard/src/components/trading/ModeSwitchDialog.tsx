/**
 * Mode Switch Dialog
 *
 * Confirmation dialog for switching to real trading mode.
 * Requires explicit checkbox confirmation before enabling.
 * Supports light/dark theme and i18n translations.
 */

import { useState, useEffect } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import { AlertTriangle } from 'lucide-react';
import { useTradingMode } from '@/hooks/useTradingMode';
import { useThemeColors } from '@/hooks/useThemeColors';
import { scaleIn } from '@/styles/tokens/animations';
import logger from '@/utils/logger';

export function ModeSwitchDialog() {
  const { isModeSwitchOpen, closeModeSwitchDialog, confirmModeSwitch, pendingMode } = useTradingMode();
  const [isConfirmed, setIsConfirmed] = useState(false);
  const { t } = useTranslation('trading');
  const colors = useThemeColors();

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
            className="fixed inset-0 z-50 bg-black/70 backdrop-blur-sm"
            onClick={handleCancel}
          />

          {/* Dialog */}
          <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
            <motion.div
              variants={scaleIn}
              initial="initial"
              animate="animate"
              exit="exit"
              className="relative w-full max-w-md rounded-2xl p-6 shadow-2xl"
              style={{
                backgroundColor: colors.bgPrimary,
                border: `1px solid ${colors.borderSubtle}`,
              }}
              onClick={(e) => e.stopPropagation()}
            >
              {/* Title with icon */}
              <div className="mb-5 flex items-center gap-3">
                <div
                  className="flex h-10 w-10 items-center justify-center rounded-full"
                  style={{ backgroundColor: 'rgba(239, 68, 68, 0.15)' }}
                >
                  <AlertTriangle className="h-5 w-5" style={{ color: colors.loss }} />
                </div>
                <h2
                  className="text-xl font-bold"
                  style={{ color: colors.loss }}
                >
                  {t('modeSwitchDialog.title')}
                </h2>
              </div>

              {/* Warning text */}
              <div className="mb-6 space-y-3">
                <p style={{ color: colors.textPrimary }}>
                  {t('modeSwitchDialog.description')}{' '}
                  <strong>{t('modeSwitchDialog.realTradingMode')}</strong>.
                </p>
                <p style={{ color: colors.textSecondary }}>
                  {t('modeSwitchDialog.warningText')}{' '}
                  <strong style={{ color: colors.textPrimary }}>{t('modeSwitchDialog.realMoney')}</strong>{' '}
                  {t('modeSwitchDialog.warningContinue')}
                </p>

                {/* Warning box */}
                <div
                  className="rounded-xl p-4"
                  style={{
                    backgroundColor: 'rgba(239, 68, 68, 0.08)',
                    border: '1px solid rgba(239, 68, 68, 0.2)',
                  }}
                >
                  <div className="flex items-start gap-2">
                    <AlertTriangle className="h-4 w-4 mt-0.5 flex-shrink-0" style={{ color: colors.loss }} />
                    <div>
                      <p
                        className="text-sm font-semibold"
                        style={{ color: colors.loss }}
                      >
                        {t('modeSwitchDialog.warningTitle')}
                      </p>
                      <p
                        className="mt-1 text-sm"
                        style={{ color: colors.textSecondary }}
                      >
                        {t('modeSwitchDialog.warningDescription')}
                      </p>
                    </div>
                  </div>
                </div>
              </div>

              {/* Confirmation checkbox */}
              <label
                className="mb-6 flex cursor-pointer items-start gap-3 rounded-lg p-3 transition-colors"
                style={{
                  backgroundColor: isConfirmed ? `${colors.success}10` : colors.bgSecondary,
                  border: `1px solid ${isConfirmed ? colors.success : colors.borderSubtle}`,
                }}
                onClick={(e) => e.stopPropagation()}
              >
                <input
                  type="checkbox"
                  checked={isConfirmed}
                  onChange={(e) => setIsConfirmed(e.target.checked)}
                  className="mt-0.5 h-5 w-5 cursor-pointer rounded"
                  style={{
                    accentColor: colors.success,
                  }}
                />
                <span
                  className="text-sm"
                  style={{ color: isConfirmed ? colors.textPrimary : colors.textSecondary }}
                >
                  {t('modeSwitchDialog.confirmCheckbox')}
                </span>
              </label>

              {/* Action buttons */}
              <div className="flex gap-3">
                <button
                  onClick={handleCancel}
                  className="flex-1 rounded-xl px-4 py-3 font-medium transition-all hover:scale-[1.02]"
                  style={{
                    backgroundColor: colors.bgSecondary,
                    color: colors.textPrimary,
                    border: `1px solid ${colors.borderSubtle}`,
                  }}
                >
                  {t('modeSwitchDialog.cancel')}
                </button>
                <button
                  onClick={handleConfirm}
                  disabled={!isConfirmed}
                  className="flex-1 rounded-xl px-4 py-3 font-medium transition-all disabled:cursor-not-allowed disabled:opacity-50"
                  style={{
                    backgroundColor: isConfirmed ? colors.loss : colors.bgTertiary,
                    color: isConfirmed ? '#FFFFFF' : colors.textMuted,
                    border: isConfirmed ? 'none' : `1px solid ${colors.borderSubtle}`,
                    transform: isConfirmed ? 'scale(1)' : 'scale(1)',
                  }}
                  onMouseEnter={(e) => {
                    if (isConfirmed) {
                      e.currentTarget.style.transform = 'scale(1.02)';
                      e.currentTarget.style.boxShadow = `0 0 20px ${colors.loss}40`;
                    }
                  }}
                  onMouseLeave={(e) => {
                    e.currentTarget.style.transform = 'scale(1)';
                    e.currentTarget.style.boxShadow = 'none';
                  }}
                >
                  {t('modeSwitchDialog.confirmSwitch')}
                </button>
              </div>

              {/* Additional info */}
              <p
                className="mt-4 text-center text-xs"
                style={{ color: colors.textMuted }}
              >
                {t('modeSwitchDialog.switchBackNote')}
              </p>
            </motion.div>
          </div>
        </>
      )}
    </AnimatePresence>
  );
}
