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
import { cn } from '@/lib/utils';
import { ModeIndicatorBadge } from './ModeIndicatorBadge';
import { Breadcrumbs } from './Breadcrumbs';
import { NotificationDropdown } from './NotificationDropdown';
import { useAuth } from '@/contexts/AuthContext';
import { useSidebar } from '@/hooks/useSidebar';
import { useTradingMode } from '@/hooks/useTradingMode';
import { luxuryColors } from '@/styles/luxury-design-system';
import { duration, easing } from '@/styles/tokens/animations';

interface HeaderProps {
  className?: string;
}

export function Header({ className }: HeaderProps) {
  const navigate = useNavigate();
  const { user, logout } = useAuth();
  const { isMobile, openMobile } = useSidebar();
  const { mode: tradingMode, requestModeSwitch } = useTradingMode();
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
        'sticky top-0 z-30 flex h-16 items-center justify-between gap-4 px-6',
        'backdrop-blur-xl',
        className
      )}
      style={{
        backgroundColor: luxuryColors.bgPrimary,
        borderBottom: `1px solid ${luxuryColors.borderSubtle}`,
        boxShadow: '0 4px 20px rgba(0, 0, 0, 0.5)',
      }}
    >
      {/* Left section - Mobile menu button + Breadcrumbs */}
      <div className="flex items-center gap-4">
        {/* Mobile menu button */}
        {isMobile && (
          <motion.button
            onClick={openMobile}
            whileHover={{ scale: 1.05, backgroundColor: luxuryColors.bgHover }}
            whileTap={{ scale: 0.95 }}
            className={cn(
              'flex h-10 w-10 items-center justify-center rounded-xl',
              'transition-all duration-200'
            )}
            style={{
              backgroundColor: luxuryColors.bgSecondary,
              color: luxuryColors.textMuted,
              border: `1px solid ${luxuryColors.borderSubtle}`,
            }}
            aria-label="Open menu"
          >
            <Menu className="h-5 w-5" />
          </motion.button>
        )}

        {/* Breadcrumbs */}
        <Breadcrumbs />
      </div>

      {/* Right section - Mode indicator + Notifications + User menu */}
      <div className="flex items-center gap-3">
        {/* Mode indicator badge - clickable to switch modes */}
        <button
          onClick={handleModeToggle}
          className="cursor-pointer transition-transform hover:scale-105 active:scale-95"
          aria-label={`Switch to ${tradingMode === 'paper' ? 'real' : 'paper'} trading mode`}
        >
          <ModeIndicatorBadge mode={tradingMode} />
        </button>

        {/* Notification dropdown */}
        <NotificationDropdown />

        {/* User menu */}
        <div className="relative">
          <motion.button
            onClick={() => setShowUserMenu(!showUserMenu)}
            whileHover={{ scale: 1.05, backgroundColor: luxuryColors.bgHover }}
            whileTap={{ scale: 0.95 }}
            className={cn(
              'flex h-10 w-10 items-center justify-center rounded-xl',
              'transition-all duration-200'
            )}
            style={{
              backgroundColor: luxuryColors.bgSecondary,
              color: luxuryColors.textMuted,
              border: `1px solid ${luxuryColors.borderSubtle}`,
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
                    backgroundColor: luxuryColors.bgPrimary,
                    border: `1px solid ${luxuryColors.borderSubtle}`,
                    boxShadow: '0 8px 32px rgba(0, 0, 0, 0.6)',
                  }}
                >
                  {/* User info */}
                  <div
                    className="px-4 py-3"
                    style={{ borderBottom: `1px solid ${luxuryColors.borderSubtle}` }}
                  >
                    <p className="text-sm font-bold" style={{ color: luxuryColors.textPrimary }}>
                      {user?.email || 'User'}
                    </p>
                    <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
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
                      style={{ color: luxuryColors.textSecondary }}
                    >
                      <SettingsIcon className="h-4 w-4" />
                      Settings
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
                      style={{ color: luxuryColors.loss }}
                    >
                      <LogOut className="h-4 w-4" />
                      Logout
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
