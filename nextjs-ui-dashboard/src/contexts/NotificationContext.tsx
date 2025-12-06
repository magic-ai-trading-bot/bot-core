/**
 * Notification Context
 *
 * Manages notification state, fetching, and real-time updates via WebSocket.
 * Supports localStorage persistence and mock data for development.
 * @spec:FR-NOTIFICATION-001 - Notification System Context
 */

import React, {
  createContext,
  useContext,
  useState,
  useEffect,
  useCallback,
  ReactNode,
  useRef,
} from 'react';
import {
  AppNotification,
  NotificationType,
  NotificationPriority,
  NotificationPreferences,
} from '@/types/notification';
import { useWebSocket } from '@/hooks/useWebSocket';
import logger from '@/utils/logger';

// Storage keys
const NOTIFICATIONS_KEY = 'bot-core-notifications';
const PREFERENCES_KEY = 'bot-core-notification-preferences';

// Default preferences
const DEFAULT_PREFERENCES: NotificationPreferences = {
  enabled: true,
  sound: true,
  desktop: false,
  email: false,
  types: {
    trade_executed: true,
    trade_closed: true,
    stop_loss_hit: true,
    take_profit_hit: true,
    order_filled: true,
    order_cancelled: true,
    signal_generated: true,
    price_alert: true,
    risk_warning: true,
    system_alert: true,
    mode_switch: true,
  },
};

export interface NotificationContextType {
  notifications: AppNotification[];
  unreadCount: number;
  isLoading: boolean;
  preferences: NotificationPreferences;
  addNotification: (notification: Omit<AppNotification, 'id' | 'createdAt' | 'read'>) => void;
  markAsRead: (id: string) => void;
  markAllAsRead: () => void;
  deleteNotification: (id: string) => void;
  clearAllNotifications: () => void;
  updatePreferences: (prefs: Partial<NotificationPreferences>) => void;
  refreshNotifications: () => void;
}

const NotificationContext = createContext<NotificationContextType | undefined>(undefined);

interface NotificationProviderProps {
  children: ReactNode;
}

export function NotificationProvider({ children }: NotificationProviderProps) {
  const [notifications, setNotifications] = useState<AppNotification[]>([]);
  const [preferences, setPreferences] = useState<NotificationPreferences>(DEFAULT_PREFERENCES);
  const [isLoading, setIsLoading] = useState(true);
  const audioRef = useRef<HTMLAudioElement | null>(null);

  // Get WebSocket for real-time notifications
  const { lastMessage } = useWebSocket();

  // Calculate unread count
  const unreadCount = notifications.filter((n) => !n.read).length;

  // Load notifications and preferences from localStorage on mount
  useEffect(() => {
    try {
      const savedNotifications = localStorage.getItem(NOTIFICATIONS_KEY);
      if (savedNotifications) {
        const parsed = JSON.parse(savedNotifications);
        setNotifications(parsed);
        logger.info(`📬 Loaded ${parsed.length} notifications from localStorage`);
      }

      const savedPreferences = localStorage.getItem(PREFERENCES_KEY);
      if (savedPreferences) {
        setPreferences({ ...DEFAULT_PREFERENCES, ...JSON.parse(savedPreferences) });
      }
    } catch (error) {
      logger.error('Failed to load notifications from localStorage:', error);
    }
    setIsLoading(false);
  }, []);

  // Save notifications to localStorage when they change
  useEffect(() => {
    if (!isLoading) {
      try {
        localStorage.setItem(NOTIFICATIONS_KEY, JSON.stringify(notifications));
      } catch (error) {
        logger.error('Failed to save notifications to localStorage:', error);
      }
    }
  }, [notifications, isLoading]);

  // Save preferences to localStorage when they change
  useEffect(() => {
    try {
      localStorage.setItem(PREFERENCES_KEY, JSON.stringify(preferences));
    } catch (error) {
      logger.error('Failed to save preferences to localStorage:', error);
    }
  }, [preferences]);

  // Handle real-time notifications from WebSocket
  useEffect(() => {
    if (!lastMessage) return;

    try {
      const message = typeof lastMessage === 'string' ? JSON.parse(lastMessage) : lastMessage;

      // Check if this is a notification event
      if (message.type === 'notification' && message.notification) {
        const newNotification: AppNotification = {
          ...message.notification,
          id: message.notification.id || generateId(),
          read: false,
          createdAt: message.notification.createdAt || new Date().toISOString(),
        };

        // Check if notification type is enabled in preferences
        if (preferences.enabled && preferences.types[newNotification.type as keyof typeof preferences.types]) {
          setNotifications((prev) => [newNotification, ...prev]);
          playNotificationSound();
          showDesktopNotification(newNotification);
          logger.info(`🔔 New notification: ${newNotification.title}`);
        }
      }

      // Handle trading events as notifications
      if (message.type === 'trade_executed' || message.type === 'trade_closed') {
        const notification = createTradeNotification(message);
        if (notification && preferences.enabled && preferences.types[notification.type]) {
          setNotifications((prev) => [notification, ...prev]);
          playNotificationSound();
          showDesktopNotification(notification);
        }
      }

      if (message.type === 'signal') {
        const notification = createSignalNotification(message);
        if (notification && preferences.enabled && preferences.types.signal_generated) {
          setNotifications((prev) => [notification, ...prev]);
          playNotificationSound();
          showDesktopNotification(notification);
        }
      }
    } catch (error) {
      logger.error('Failed to process WebSocket notification:', error);
    }
  }, [lastMessage, preferences]);

  // Play notification sound
  const playNotificationSound = useCallback(() => {
    if (!preferences.sound) return;

    try {
      if (!audioRef.current) {
        audioRef.current = new Audio('/sounds/notification.mp3');
        audioRef.current.volume = 0.5;
      }
      audioRef.current.play().catch(() => {
        // Audio play failed - browser might block autoplay
      });
    } catch (error) {
      // Ignore audio errors
    }
  }, [preferences.sound]);

  // Show desktop notification (uses browser's global Notification API)
  const showDesktopNotification = useCallback(
    (appNotification: AppNotification) => {
      if (!preferences.desktop || !('Notification' in window)) return;

      if (Notification.permission === 'granted') {
        new Notification(appNotification.title, {
          body: appNotification.message,
          icon: '/logo.png',
          tag: appNotification.id,
        });
      } else if (Notification.permission !== 'denied') {
        Notification.requestPermission().then((permission) => {
          if (permission === 'granted') {
            new Notification(appNotification.title, {
              body: appNotification.message,
              icon: '/logo.png',
              tag: appNotification.id,
            });
          }
        });
      }
    },
    [preferences.desktop]
  );

  // Add a new notification
  const addNotification = useCallback(
    (notification: Omit<AppNotification, 'id' | 'createdAt' | 'read'>) => {
      const newNotification: AppNotification = {
        ...notification,
        id: generateId(),
        read: false,
        createdAt: new Date().toISOString(),
      };

      if (preferences.enabled && preferences.types[notification.type as keyof typeof preferences.types]) {
        setNotifications((prev) => [newNotification, ...prev]);
        playNotificationSound();
        showDesktopNotification(newNotification);
        logger.info(`🔔 Notification added: ${newNotification.title}`);
      }
    },
    [preferences, playNotificationSound, showDesktopNotification]
  );

  // Mark single notification as read
  const markAsRead = useCallback((id: string) => {
    setNotifications((prev) =>
      prev.map((n) => (n.id === id ? { ...n, read: true } : n))
    );
    logger.info(`✓ Notification ${id} marked as read`);
  }, []);

  // Mark all notifications as read
  const markAllAsRead = useCallback(() => {
    setNotifications((prev) => prev.map((n) => ({ ...n, read: true })));
    logger.info('✓ All notifications marked as read');
  }, []);

  // Delete a notification
  const deleteNotification = useCallback((id: string) => {
    setNotifications((prev) => prev.filter((n) => n.id !== id));
    logger.info(`🗑 Notification ${id} deleted`);
  }, []);

  // Clear all notifications
  const clearAllNotifications = useCallback(() => {
    setNotifications([]);
    logger.info('🗑 All notifications cleared');
  }, []);

  // Update preferences
  const updatePreferences = useCallback((prefs: Partial<NotificationPreferences>) => {
    setPreferences((prev) => ({ ...prev, ...prefs }));
    logger.info('⚙️ Notification preferences updated');
  }, []);

  // Refresh notifications (placeholder for API call)
  const refreshNotifications = useCallback(() => {
    logger.info('🔄 Refreshing notifications...');
    // TODO: Implement API call to fetch notifications
    // For now, just log that we would fetch
  }, []);

  const value: NotificationContextType = {
    notifications,
    unreadCount,
    isLoading,
    preferences,
    addNotification,
    markAsRead,
    markAllAsRead,
    deleteNotification,
    clearAllNotifications,
    updatePreferences,
    refreshNotifications,
  };

  return (
    <NotificationContext.Provider value={value}>
      {children}
    </NotificationContext.Provider>
  );
}

// Hook to use notification context
export function useNotificationContext(): NotificationContextType {
  const context = useContext(NotificationContext);
  if (context === undefined) {
    throw new Error('useNotificationContext must be used within NotificationProvider');
  }
  return context;
}

// Helper functions
function generateId(): string {
  return `notif_${Date.now()}_${Math.random().toString(36).substr(2, 9)}`;
}

function createTradeNotification(message: Record<string, unknown>): AppNotification | null {
  try {
    const isOpen = message.type === 'trade_executed';
    const data = message.data as Record<string, unknown> | undefined;

    return {
      id: generateId(),
      type: isOpen ? 'trade_executed' : 'trade_closed',
      title: isOpen ? 'Trade Opened' : 'Trade Closed',
      message: `${data?.side || 'Long'} ${data?.symbol || 'BTCUSDT'} @ $${data?.price || '0'}`,
      priority: 'medium' as NotificationPriority,
      read: false,
      createdAt: new Date().toISOString(),
      data: {
        tradeId: data?.trade_id as string,
        symbol: data?.symbol as string,
        side: data?.side as 'long' | 'short',
        entryPrice: data?.price as number,
        quantity: data?.quantity as number,
        pnl: data?.pnl as number,
      },
    };
  } catch {
    return null;
  }
}

function createSignalNotification(message: Record<string, unknown>): AppNotification | null {
  try {
    const data = message.data as Record<string, unknown> | undefined;

    return {
      id: generateId(),
      type: 'signal_generated',
      title: 'New AI Signal',
      message: `${data?.signal || 'Long'} signal for ${data?.symbol || 'BTCUSDT'} (${Math.round((data?.confidence as number || 0.7) * 100)}% confidence)`,
      priority: 'high' as NotificationPriority,
      read: false,
      createdAt: new Date().toISOString(),
      data: {
        signalId: data?.signal_id as string,
        symbol: data?.symbol as string,
        confidence: data?.confidence as number,
        strategy: data?.strategy as string,
      },
    };
  } catch {
    return null;
  }
}
