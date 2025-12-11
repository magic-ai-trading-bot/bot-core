/**
 * Notification Dropdown Component
 *
 * Dropdown menu showing notifications with read/unread status,
 * mark as read, and clear functionality.
 * @spec:FR-NOTIFICATION-001 - Notification UI Component
 */

import { useState } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useTranslation } from 'react-i18next';
import {
  Bell,
  Check,
  CheckCheck,
  Trash2,
  X,
  TrendingUp,
  AlertTriangle,
  Zap,
  Info,
  RefreshCw,
  PlayCircle,
  CheckCircle,
  XCircle,
} from 'lucide-react';
import { cn } from '@/lib/utils';
import { useNotification } from '@/hooks/useNotification';
import { AppNotification, NotificationType, getNotificationColor } from '@/types/notification';
import { useThemeColors } from '@/hooks/useThemeColors';
import { duration, easing } from '@/styles/tokens/animations';

export function NotificationDropdown() {
  const [isOpen, setIsOpen] = useState(false);
  const { t } = useTranslation('common');
  const colors = useThemeColors();
  const {
    notifications,
    unreadCount,
    markAsRead,
    markAllAsRead,
    deleteNotification,
    clearAllNotifications,
  } = useNotification();

  const handleToggle = () => {
    setIsOpen(!isOpen);
  };

  const handleNotificationClick = (notification: AppNotification) => {
    if (!notification.read) {
      markAsRead(notification.id);
    }
    // Could navigate to relevant page based on notification type
  };

  const formatTime = (dateString: string) => {
    const date = new Date(dateString);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);
    const diffHours = Math.floor(diffMs / 3600000);
    const diffDays = Math.floor(diffMs / 86400000);

    if (diffMins < 1) return t('time.now');
    if (diffMins < 60) return t('time.minutesAgo', { count: diffMins });
    if (diffHours < 24) return t('time.hoursAgo', { count: diffHours });
    if (diffDays < 7) return t('time.daysAgo', { count: diffDays });
    return date.toLocaleDateString();
  };

  const getIcon = (type: NotificationType) => {
    const color = getNotificationColor(type);
    const iconProps = { className: 'h-4 w-4', style: { color } };

    switch (type) {
      case 'trade_executed':
        return <PlayCircle {...iconProps} />;
      case 'trade_closed':
        return <CheckCircle {...iconProps} />;
      case 'stop_loss_hit':
        return <AlertTriangle {...iconProps} />;
      case 'take_profit_hit':
        return <TrendingUp {...iconProps} />;
      case 'order_filled':
        return <Check {...iconProps} />;
      case 'order_cancelled':
        return <XCircle {...iconProps} />;
      case 'signal_generated':
        return <Zap {...iconProps} />;
      case 'price_alert':
        return <Bell {...iconProps} />;
      case 'risk_warning':
        return <AlertTriangle {...iconProps} />;
      case 'system_alert':
        return <Info {...iconProps} />;
      case 'mode_switch':
        return <RefreshCw {...iconProps} />;
      default:
        return <Bell {...iconProps} />;
    }
  };

  return (
    <div className="relative">
      {/* Bell button */}
      <motion.button
        onClick={handleToggle}
        whileHover={{ scale: 1.05, backgroundColor: colors.bgHover }}
        whileTap={{ scale: 0.95 }}
        className={cn(
          'relative flex h-10 w-10 items-center justify-center rounded-xl',
          'transition-all duration-200'
        )}
        style={{
          backgroundColor: isOpen ? colors.bgHover : colors.bgSecondary,
          color: colors.textMuted,
          border: `1px solid ${colors.borderSubtle}`,
        }}
        aria-label="Notifications"
        aria-expanded={isOpen}
      >
        <Bell className="h-5 w-5" />

        {/* Unread badge */}
        {unreadCount > 0 && (
          <motion.span
            initial={{ scale: 0 }}
            animate={{ scale: 1 }}
            className="absolute -right-1 -top-1 flex h-5 min-w-[20px] items-center justify-center rounded-full px-1 text-xs font-bold text-white"
            style={{
              backgroundColor: colors.amber,
              boxShadow: '0 0 8px rgba(245, 158, 11, 0.5)',
            }}
          >
            {unreadCount > 99 ? '99+' : unreadCount}
          </motion.span>
        )}
      </motion.button>

      {/* Dropdown */}
      <AnimatePresence>
        {isOpen && (
          <>
            {/* Backdrop */}
            <div
              className="fixed inset-0 z-40"
              onClick={() => setIsOpen(false)}
            />

            {/* Dropdown panel */}
            <motion.div
              initial={{ opacity: 0, scale: 0.95, y: -10 }}
              animate={{ opacity: 1, scale: 1, y: 0 }}
              exit={{ opacity: 0, scale: 0.95, y: -10 }}
              transition={{
                duration: duration.fast,
                ease: easing.easeOut,
              }}
              className="absolute right-0 top-12 z-50 w-80 md:w-96 rounded-xl overflow-hidden"
              style={{
                backgroundColor: colors.bgPrimary,
                border: `1px solid ${colors.borderSubtle}`,
                boxShadow: '0 8px 32px rgba(0, 0, 0, 0.6)',
              }}
            >
              {/* Header */}
              <div
                className="flex items-center justify-between px-4 py-3"
                style={{ borderBottom: `1px solid ${colors.borderSubtle}` }}
              >
                <div className="flex items-center gap-2">
                  <h3
                    className="text-sm font-bold"
                    style={{ color: colors.textPrimary }}
                  >
                    {t('notification.title')}
                  </h3>
                  {unreadCount > 0 && (
                    <span
                      className="rounded-full px-2 py-0.5 text-xs font-medium"
                      style={{
                        backgroundColor: `${colors.amber}20`,
                        color: colors.amber,
                      }}
                    >
                      {t('notification.new', { count: unreadCount })}
                    </span>
                  )}
                </div>

                <div className="flex items-center gap-2">
                  {unreadCount > 0 && (
                    <button
                      onClick={markAllAsRead}
                      className="flex items-center gap-1 text-xs transition-colors hover:text-white"
                      style={{ color: colors.textMuted }}
                      title="Mark all as read"
                    >
                      <CheckCheck className="h-4 w-4" />
                    </button>
                  )}
                  {notifications.length > 0 && (
                    <button
                      onClick={clearAllNotifications}
                      className="flex items-center gap-1 text-xs transition-colors hover:text-red-400"
                      style={{ color: colors.textMuted }}
                      title="Clear all"
                    >
                      <Trash2 className="h-4 w-4" />
                    </button>
                  )}
                </div>
              </div>

              {/* Notification list */}
              <div className="max-h-96 overflow-y-auto">
                {notifications.length === 0 ? (
                  <div className="flex flex-col items-center justify-center py-12">
                    <Bell
                      className="h-12 w-12 mb-3"
                      style={{ color: colors.textMuted }}
                    />
                    <p
                      className="text-sm"
                      style={{ color: colors.textMuted }}
                    >
                      {t('notification.empty')}
                    </p>
                    <p
                      className="text-xs mt-1"
                      style={{ color: colors.textMuted }}
                    >
                      {t('notification.emptyDesc')}
                    </p>
                  </div>
                ) : (
                  notifications.slice(0, 50).map((notification) => (
                    <motion.div
                      key={notification.id}
                      initial={{ opacity: 0, x: -20 }}
                      animate={{ opacity: 1, x: 0 }}
                      exit={{ opacity: 0, x: 20 }}
                      className={cn(
                        'group flex items-start gap-3 px-4 py-3 cursor-pointer transition-colors',
                        'hover:bg-white/5'
                      )}
                      style={{
                        borderBottom: `1px solid ${colors.borderSubtle}`,
                        backgroundColor: notification.read
                          ? 'transparent'
                          : `${colors.amber}05`,
                      }}
                      onClick={() => handleNotificationClick(notification)}
                    >
                      {/* Icon */}
                      <div
                        className="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg"
                        style={{
                          backgroundColor: `${getNotificationColor(notification.type)}15`,
                        }}
                      >
                        {getIcon(notification.type)}
                      </div>

                      {/* Content */}
                      <div className="flex-1 min-w-0">
                        <div className="flex items-start justify-between gap-2">
                          <p
                            className={cn(
                              'text-sm truncate',
                              notification.read ? 'font-normal' : 'font-semibold'
                            )}
                            style={{ color: colors.textPrimary }}
                          >
                            {notification.title}
                          </p>
                          {!notification.read && (
                            <span
                              className="flex-shrink-0 h-2 w-2 rounded-full mt-1"
                              style={{ backgroundColor: colors.amber }}
                            />
                          )}
                        </div>
                        <p
                          className="text-xs mt-0.5 line-clamp-2"
                          style={{ color: colors.textSecondary }}
                        >
                          {notification.message}
                        </p>
                        <p
                          className="text-xs mt-1"
                          style={{ color: colors.textMuted }}
                        >
                          {formatTime(notification.createdAt)}
                        </p>
                      </div>

                      {/* Delete button (on hover) */}
                      <button
                        onClick={(e) => {
                          e.stopPropagation();
                          deleteNotification(notification.id);
                        }}
                        className="flex-shrink-0 opacity-0 group-hover:opacity-100 transition-opacity"
                        style={{ color: colors.textMuted }}
                        title="Delete"
                      >
                        <X className="h-4 w-4 hover:text-red-400" />
                      </button>
                    </motion.div>
                  ))
                )}
              </div>

              {/* Footer */}
              {notifications.length > 0 && (
                <div
                  className="px-4 py-2"
                  style={{ borderTop: `1px solid ${colors.borderSubtle}` }}
                >
                  <p
                    className="text-xs text-center"
                    style={{ color: colors.textMuted }}
                  >
                    {t('notification.showing', { current: Math.min(notifications.length, 50), total: notifications.length })}
                  </p>
                </div>
              )}
            </motion.div>
          </>
        )}
      </AnimatePresence>
    </div>
  );
}
