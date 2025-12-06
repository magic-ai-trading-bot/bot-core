/**
 * Notification Preferences Hook
 *
 * Manages notification preferences with backend API integration.
 * Syncs with Rust Core Engine's notification preferences endpoints.
 *
 * @spec:FR-NOTIFICATION-001 - Notification Preferences API Integration
 * @ref:specs/02-design/2.3-api/API-RUST-CORE.md
 */

import { useState, useEffect, useCallback } from 'react';
import logger from '@/utils/logger';

const API_BASE = import.meta.env.VITE_RUST_API_URL || 'http://localhost:8080';

// Types matching the Rust API
export interface NotificationPreferencesAPI {
  enabled: boolean;
  channels: ChannelSettings;
  alerts: AlertSettings;
  price_alert_threshold: number;
  created_at: string;
  updated_at: string;
}

export interface ChannelSettings {
  email: boolean;
  push: PushSettings;
  telegram: TelegramSettings;
  discord: DiscordSettings;
  sound: boolean;
}

export interface PushSettings {
  enabled: boolean;
  vapid_public_key: string | null;
  vapid_private_key: string | null;
}

export interface TelegramSettings {
  enabled: boolean;
  bot_token: string | null;
  chat_id: string | null;
}

export interface DiscordSettings {
  enabled: boolean;
  webhook_url: string | null;
}

export interface AlertSettings {
  price_alerts: boolean;
  trade_alerts: boolean;
  system_alerts: boolean;
  signal_alerts: boolean;
  risk_alerts: boolean;
}

// Update request types
export interface UpdateNotificationPreferencesRequest {
  enabled?: boolean;
  channels?: Partial<ChannelSettingsUpdate>;
  alerts?: Partial<AlertSettings>;
  price_alert_threshold?: number;
}

export interface ChannelSettingsUpdate {
  email?: boolean;
  push?: Partial<PushSettings>;
  telegram?: Partial<TelegramSettings>;
  discord?: Partial<DiscordSettings>;
  sound?: boolean;
}

// API response type
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
}

// Default preferences (mirrors Rust defaults)
const DEFAULT_PREFERENCES: NotificationPreferencesAPI = {
  enabled: true,
  channels: {
    email: false,
    push: { enabled: false, vapid_public_key: null, vapid_private_key: null },
    telegram: { enabled: false, bot_token: null, chat_id: null },
    discord: { enabled: false, webhook_url: null },
    sound: true,
  },
  alerts: {
    price_alerts: true,
    trade_alerts: true,
    system_alerts: true,
    signal_alerts: true,
    risk_alerts: true,
  },
  price_alert_threshold: 5.0,
  created_at: new Date().toISOString(),
  updated_at: new Date().toISOString(),
};

export interface UseNotificationPreferencesResult {
  preferences: NotificationPreferencesAPI;
  isLoading: boolean;
  isSaving: boolean;
  error: string | null;
  loadPreferences: () => Promise<void>;
  savePreferences: (updates: UpdateNotificationPreferencesRequest) => Promise<boolean>;
  testNotification: (channel: 'email' | 'telegram' | 'discord' | 'push') => Promise<boolean>;
}

export function useNotificationPreferences(): UseNotificationPreferencesResult {
  const [preferences, setPreferences] = useState<NotificationPreferencesAPI>(DEFAULT_PREFERENCES);
  const [isLoading, setIsLoading] = useState(true);
  const [isSaving, setIsSaving] = useState(false);
  const [error, setError] = useState<string | null>(null);

  // Load preferences from API
  const loadPreferences = useCallback(async () => {
    setIsLoading(true);
    setError(null);

    try {
      const response = await fetch(`${API_BASE}/api/notifications/preferences`, {
        method: 'GET',
        headers: {
          'Content-Type': 'application/json',
        },
      });

      if (!response.ok) {
        throw new Error(`Failed to load preferences: ${response.status}`);
      }

      const data: ApiResponse<NotificationPreferencesAPI> = await response.json();

      if (data.success && data.data) {
        setPreferences(data.data);
        logger.info('📬 Notification preferences loaded from API');
      } else {
        // Use defaults if no data
        setPreferences(DEFAULT_PREFERENCES);
        logger.info('📬 Using default notification preferences');
      }
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      setError(errorMessage);
      logger.error('Failed to load notification preferences:', errorMessage);
      // Fallback to defaults
      setPreferences(DEFAULT_PREFERENCES);
    } finally {
      setIsLoading(false);
    }
  }, []);

  // Save preferences to API
  const savePreferences = useCallback(
    async (updates: UpdateNotificationPreferencesRequest): Promise<boolean> => {
      setIsSaving(true);
      setError(null);

      try {
        const response = await fetch(`${API_BASE}/api/notifications/preferences`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify(updates),
        });

        if (!response.ok) {
          const errorData = await response.json();
          throw new Error(errorData.error || `Failed to save preferences: ${response.status}`);
        }

        const data: ApiResponse<NotificationPreferencesAPI> = await response.json();

        if (data.success && data.data) {
          setPreferences(data.data);
          logger.info('✅ Notification preferences saved to API');
          return true;
        } else {
          throw new Error(data.error || 'Failed to save preferences');
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        setError(errorMessage);
        logger.error('Failed to save notification preferences:', errorMessage);
        return false;
      } finally {
        setIsSaving(false);
      }
    },
    []
  );

  // Test notification
  const testNotification = useCallback(
    async (channel: 'email' | 'telegram' | 'discord' | 'push'): Promise<boolean> => {
      try {
        const response = await fetch(`${API_BASE}/api/notifications/test`, {
          method: 'POST',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({ channel }),
        });

        if (!response.ok) {
          const errorData = await response.json();
          throw new Error(errorData.error || `Test failed: ${response.status}`);
        }

        const data: ApiResponse<{ message: string; channel: string }> = await response.json();

        if (data.success) {
          logger.info(`🔔 Test notification sent to ${channel}`);
          return true;
        } else {
          throw new Error(data.error || 'Test notification failed');
        }
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        logger.error(`Failed to send test notification to ${channel}:`, errorMessage);
        return false;
      }
    },
    []
  );

  // Load preferences on mount
  useEffect(() => {
    loadPreferences();
  }, [loadPreferences]);

  return {
    preferences,
    isLoading,
    isSaving,
    error,
    loadPreferences,
    savePreferences,
    testNotification,
  };
}

// Helper to convert between API format and local state format
export function apiToLocalFormat(api: NotificationPreferencesAPI) {
  return {
    email: api.channels.email,
    push: api.channels.push.enabled,
    telegram: api.channels.telegram.enabled,
    discord: api.channels.discord.enabled,
    sound: api.channels.sound,
    priceAlerts: api.alerts.price_alerts,
    tradeAlerts: api.alerts.trade_alerts,
    systemAlerts: api.alerts.system_alerts,
    signalAlerts: api.alerts.signal_alerts,
    riskAlerts: api.alerts.risk_alerts,
  };
}

export interface NotificationCredentials {
  telegramToken?: string;
  telegramChatId?: string;
  discordWebhookUrl?: string;
  vapidPublicKey?: string;
  vapidPrivateKey?: string;
  alertThreshold?: number;
}

export function localToApiFormat(
  local: {
    email: boolean;
    push: boolean;
    telegram: boolean;
    discord: boolean;
    sound: boolean;
    priceAlerts: boolean;
    tradeAlerts: boolean;
    systemAlerts: boolean;
  },
  credentials?: NotificationCredentials
): UpdateNotificationPreferencesRequest {
  return {
    channels: {
      email: local.email,
      push: {
        enabled: local.push,
        vapid_public_key: credentials?.vapidPublicKey || null,
        vapid_private_key: credentials?.vapidPrivateKey || null,
      },
      telegram: {
        enabled: local.telegram,
        bot_token: credentials?.telegramToken || null,
        chat_id: credentials?.telegramChatId || null,
      },
      discord: {
        enabled: local.discord,
        webhook_url: credentials?.discordWebhookUrl || null,
      },
      sound: local.sound,
    },
    alerts: {
      price_alerts: local.priceAlerts,
      trade_alerts: local.tradeAlerts,
      system_alerts: local.systemAlerts,
      signal_alerts: true,
      risk_alerts: true,
    },
    price_alert_threshold: credentials?.alertThreshold,
  };
}
