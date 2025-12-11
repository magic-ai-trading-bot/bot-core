/**
 * Header Component
 *
 * Top navigation bar with mode indicator, breadcrumbs, user menu, and mobile menu button.
 * @spec:FR-DASHBOARD-001 - Header with trading mode switch
 */

import { useState } from 'react';
import { Menu, User, LogOut, Settings as SettingsIcon } from 'lucide-react';
import { motion, AnimatePresence } from 'framer-motion';
import { useNavigate } from 'react-router-dom';
import { useTranslation } from 'react-i18next';
import { cn } from '@/lib/utils';
import { ModeIndicatorBadge } from './ModeIndicatorBadge';
import { Breadcrumbs } from './Breadcrumbs';
import { NotificationDropdown } from './NotificationDropdown';
import { ThemeToggle } from '@/components/ThemeToggle';
import { LanguageSelector } from '@/components/LanguageSelector';
import { useAuth } from '@/contexts/AuthContext';
import { useSidebar } from '@/hooks/useSidebar';
import { useTradingMode } from '@/hooks/useTradingMode';
import { useThemeColors } from '@/hooks/useThemeColors';
import { duration, easing } from '@/styles/tokens/animations';

interface HeaderProps {
  className?: string;
}

export function Header({ className }: HeaderProps) {
  const navigate = useNavigate();
  const { t } = useTranslation('common');
  const { user, logout } = useAuth();
  const { isMobile, openMobile } = useSidebar();
  const { mode: tradingMode, requestModeSwitch } = useTradingMode();
  const colors = useThemeColors();
  const [showUserMenu, setShowUserMenu] = useState(false);

  const handleLogout = () => {
    logout();
    navigate('/login');
  };

  // Toggle trading mode when badge is clicked
  const handleModeToggle = () => {
    const targetMode = tradingMode === 'paper' ? 'real' : 'paper';
    requestModeSwitch(targetMode);
  };

  return (
    <header
      className={cn(
        // Responsive height: h-14 on mobile, h-16 on tablet+
        'sticky top-0 z-30 flex h-14 sm:h-16 items-center justify-between',
        // Responsive padding: tighter on mobile
        'gap-2 sm:gap-4 px-3 sm:px-4 md:px-6',
        'backdrop-blur-xl',
        className
      )}
      style={{
        backgroundColor: colors.bgPrimary,
        borderBottom: `1px solid ${colors.borderSubtle}`,
        boxShadow: '0 4px 20px rgba(0, 0, 0, 0.5)',
      }}
    >
      {/* Left section - Mobile menu button + Breadcrumbs */}
      <div className="flex items-center gap-2 sm:gap-4">
        {/* Mobile menu button - 44px touch target (WCAG 2.5.5) */}
        {isMobile && (
          <motion.button
            onClick={openMobile}
            whileHover={{ scale: 1.05, backgroundColor: colors.bgHover }}
            whileTap={{ scale: 0.95 }}
            className={cn(
              'flex h-11 w-11 items-center justify-center rounded-xl',
              'transition-all duration-200'
            )}
            style={{
              backgroundColor: colors.bgSecondary,
              color: colors.textMuted,
              border: `1px solid ${colors.borderSubtle}`,
            }}
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" />
          </motion.button>
        )}

        {/* Breadcrumbs */}
        <Breadcrumbs />
      </div>

      {/* Right section - Mode indicator + Theme + Language + Notifications + User menu */}
      <div className="flex items-center gap-1.5 sm:gap-2 md:gap-3">
        {/* Mode indicator badge - clickable to switch modes */}
        {/* Hide text on very small screens, show icon only */}
        <button
          onClick={handleModeToggle}
          className="cursor-pointer transition-transform hover:scale-105 active:scale-95"
          aria-label={`Switch to ${tradingMode === 'paper' ? 'real' : 'paper'} trading mode`}
        >
          <ModeIndicatorBadge mode={tradingMode} compact={isMobile} />
        </button>

        {/* Theme toggle */}
        <ThemeToggle />

        {/* Language selector - hidden on mobile to save space */}
        <div className="hidden sm:block">
          <LanguageSelector />
        </div>

        {/* Notification dropdown */}
        <NotificationDropdown />

        {/* User menu - 44px touch target (WCAG 2.5.5) */}
        <div className="relative">
          <motion.button
            onClick={() => setShowUserMenu(!showUserMenu)}
            whileHover={{ scale: 1.05, backgroundColor: colors.bgHover }}
            whileTap={{ scale: 0.95 }}
            className={cn(
              'flex h-11 w-11 items-center justify-center rounded-xl',
              'transition-all duration-200'
            )}
            style={{
              backgroundColor: colors.bgSecondary,
              color: colors.textMuted,
              border: `1px solid ${colors.borderSubtle}`,
            }}
            aria-label="User menu"
            aria-expanded={showUserMenu}
          >
            <User className="h-5 w-5" />
          </motion.button>

          {/* Dropdown menu */}
          <AnimatePresence>
            {showUserMenu && (
              <>
                {/* Backdrop */}
                <div
                  className="fixed inset-0 z-40"
                  onClick={() => setShowUserMenu(false)}
                />

                {/* Menu */}
                <motion.div
                  initial={{ opacity: 0, scale: 0.95, y: -10 }}
                  animate={{ opacity: 1, scale: 1, y: 0 }}
                  exit={{ opacity: 0, scale: 0.95, y: -10 }}
                  transition={{
                    duration: duration.fast,
                    ease: easing.easeOut,
                  }}
                  className="absolute right-0 top-12 z-50 w-56 rounded-xl overflow-hidden"
                  style={{
                    backgroundColor: colors.bgPrimary,
                    border: `1px solid ${colors.borderSubtle}`,
                    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.6)',
                  }}
                >
                  {/* User info */}
                  <div
                    className="px-4 py-3"
                    style={{ borderBottom: `1px solid ${colors.borderSubtle}` }}
                  >
                    <p className="text-sm font-bold" style={{ color: colors.textPrimary }}>
                      {user?.email || 'User'}
                    </p>
                    <p className="text-xs" style={{ color: colors.textMuted }}>
                      Free Plan
                    </p>
                  </div>

                  {/* Menu items */}
                  <div className="py-1">
                    <button
                      onClick={() => {
                        setShowUserMenu(false);
                        navigate('/settings');
                      }}
                      className={cn(
                        'flex w-full items-center gap-3 px-4 py-2.5 text-sm',
                        'hover:bg-white/5 transition-colors'
                      )}
                      style={{ color: colors.textSecondary }}
                    >
                      <SettingsIcon className="h-4 w-4" />
                      {t('label.settings')}
                    </button>

                    <button
                      onClick={() => {
                        setShowUserMenu(false);
                        handleLogout();
                      }}
                      className={cn(
                        'flex w-full items-center gap-3 px-4 py-2.5 text-sm',
                        'hover:bg-white/5 transition-colors'
                      )}
                      style={{ color: colors.loss }}
                    >
                      <LogOut className="h-4 w-4" />
                      {t('label.logout')}
                    </button>
                  </div>
                </motion.div>
              </>
            )}
          </AnimatePresence>
        </div>
      </div>
    </header>
  );
}
