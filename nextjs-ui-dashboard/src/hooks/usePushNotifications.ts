/**
 * Push Notifications Hook
 *
 * Manages Service Worker registration and push notification subscriptions.
 * Integrates with Rust Core Engine's push subscription endpoints.
 *
 * @spec:FR-NOTIFICATION-002 - Push Notifications via Service Worker
 * @ref:specs/02-design/2.3-api/API-RUST-CORE.md
 */

import { useState, useEffect, useCallback } from 'react';
import logger from '@/utils/logger';

const API_BASE = import.meta.env.VITE_RUST_API_URL || 'http://localhost:8080';

export interface PushNotificationState {
  isSupported: boolean;
  isSubscribed: boolean;
  isLoading: boolean;
  permission: NotificationPermission | 'unsupported';
  error: string | null;
  registration: ServiceWorkerRegistration | null;
  subscription: PushSubscription | null;
}

export interface UsePushNotificationsResult extends PushNotificationState {
  subscribe: (vapidPublicKey: string) => Promise<boolean>;
  unsubscribe: () => Promise<boolean>;
  requestPermission: () => Promise<NotificationPermission | 'unsupported'>;
  showLocalNotification: (title: string, options?: NotificationOptions) => Promise<void>;
}

// Check if push notifications are supported
function isPushSupported(): boolean {
  return (
    'serviceWorker' in navigator &&
    'PushManager' in window &&
    'Notification' in window
  );
}

// Convert base64 VAPID key to Uint8Array
function urlBase64ToUint8Array(base64String: string): Uint8Array {
  const padding = '='.repeat((4 - (base64String.length % 4)) % 4);
  const base64 = (base64String + padding)
    .replace(/-/g, '+')
    .replace(/_/g, '/');

  const rawData = window.atob(base64);
  const outputArray = new Uint8Array(rawData.length);

  for (let i = 0; i < rawData.length; ++i) {
    outputArray[i] = rawData.charCodeAt(i);
  }

  return outputArray;
}

export function usePushNotifications(): UsePushNotificationsResult {
  const [state, setState] = useState<PushNotificationState>({
    isSupported: false,
    isSubscribed: false,
    isLoading: true,
    permission: 'unsupported',
    error: null,
    registration: null,
    subscription: null,
  });

  // Initialize - check support and register service worker
  useEffect(() => {
    const initialize = async () => {
      if (!isPushSupported()) {
        setState((prev) => ({
          ...prev,
          isSupported: false,
          isLoading: false,
          permission: 'unsupported',
        }));
        return;
      }

      try {
        // Register service worker
        const registration = await navigator.serviceWorker.register('/sw.js', {
          scope: '/',
        });

        logger.info('📱 Service Worker registered:', registration.scope);

        // Wait for the service worker to be ready
        await navigator.serviceWorker.ready;

        // Check current subscription
        const subscription = await registration.pushManager.getSubscription();

        setState({
          isSupported: true,
          isSubscribed: !!subscription,
          isLoading: false,
          permission: Notification.permission,
          error: null,
          registration,
          subscription,
        });

        logger.info('📱 Push notifications initialized', {
          subscribed: !!subscription,
          permission: Notification.permission,
        });
      } catch (err) {
        const errorMessage = err instanceof Error ? err.message : 'Unknown error';
        logger.error('Failed to initialize push notifications:', errorMessage);
        setState((prev) => ({
          ...prev,
          isSupported: true,
          isLoading: false,
          error: errorMessage,
        }));
      }
    };

    initialize();
  }, []);

  // Request notification permission
  const requestPermission = useCallback(async (): Promise<NotificationPermission | 'unsupported'> => {
    if (!isPushSupported()) {
      return 'unsupported';
    }

    try {
      const permission = await Notification.requestPermission();
      setState((prev) => ({ ...prev, permission }));
      logger.info('📱 Notification permission:', permission);
      return permission;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      logger.error('Failed to request notification permission:', errorMessage);
      return 'denied';
    }
  }, []);

  // Subscribe to push notifications
  const subscribe = useCallback(async (vapidPublicKey: string): Promise<boolean> => {
    if (!state.registration) {
      setState((prev) => ({ ...prev, error: 'Service worker not registered' }));
      return false;
    }

    if (!vapidPublicKey) {
      setState((prev) => ({ ...prev, error: 'VAPID public key not configured. Please enter it in Settings.' }));
      logger.warn('📱 VAPID public key not configured - push notifications disabled');
      return false;
    }

    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      // Request permission if not granted
      const permission = await requestPermission();
      if (permission !== 'granted') {
        setState((prev) => ({
          ...prev,
          isLoading: false,
          error: 'Notification permission denied',
        }));
        return false;
      }

      // Subscribe to push manager
      const subscription = await state.registration.pushManager.subscribe({
        userVisibleOnly: true,
        applicationServerKey: urlBase64ToUint8Array(vapidPublicKey),
      });

      // Send subscription to backend
      const response = await fetch(`${API_BASE}/api/notifications/push/subscribe`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          endpoint: subscription.endpoint,
          keys: {
            p256dh: arrayBufferToBase64(subscription.getKey('p256dh')),
            auth: arrayBufferToBase64(subscription.getKey('auth')),
          },
        }),
      });

      if (!response.ok) {
        throw new Error(`Failed to register subscription: ${response.status}`);
      }

      setState((prev) => ({
        ...prev,
        isSubscribed: true,
        isLoading: false,
        subscription,
      }));

      logger.info('📱 Push notification subscription successful');
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      logger.error('Failed to subscribe to push notifications:', errorMessage);
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      return false;
    }
  }, [state.registration, requestPermission]);

  // Unsubscribe from push notifications
  const unsubscribe = useCallback(async (): Promise<boolean> => {
    if (!state.subscription) {
      return true; // Already unsubscribed
    }

    setState((prev) => ({ ...prev, isLoading: true, error: null }));

    try {
      // Unsubscribe from push manager
      await state.subscription.unsubscribe();

      // Notify backend
      await fetch(`${API_BASE}/api/notifications/push/unsubscribe`, {
        method: 'POST',
        headers: {
          'Content-Type': 'application/json',
        },
        body: JSON.stringify({
          endpoint: state.subscription.endpoint,
        }),
      });

      setState((prev) => ({
        ...prev,
        isSubscribed: false,
        isLoading: false,
        subscription: null,
      }));

      logger.info('📱 Push notification unsubscribed');
      return true;
    } catch (err) {
      const errorMessage = err instanceof Error ? err.message : 'Unknown error';
      logger.error('Failed to unsubscribe from push notifications:', errorMessage);
      setState((prev) => ({
        ...prev,
        isLoading: false,
        error: errorMessage,
      }));
      return false;
    }
  }, [state.subscription]);

  // Show a local notification (without push server)
  const showLocalNotification = useCallback(
    async (title: string, options?: NotificationOptions): Promise<void> => {
      if (!state.registration) {
        logger.warn('📱 Cannot show notification - service worker not registered');
        return;
      }

      if (Notification.permission !== 'granted') {
        const permission = await requestPermission();
        if (permission !== 'granted') {
          logger.warn('📱 Cannot show notification - permission denied');
          return;
        }
      }

      try {
        await state.registration.showNotification(title, {
          icon: '/android-chrome-192x192.png',
          badge: '/android-chrome-192x192.png',
          tag: `local-${Date.now()}`,
          ...options,
        });
      } catch (err) {
        logger.error('Failed to show local notification:', err);
      }
    },
    [state.registration, requestPermission]
  );

  return {
    ...state,
    subscribe,
    unsubscribe,
    requestPermission,
    showLocalNotification,
  };
}

// Helper: Convert ArrayBuffer to base64
function arrayBufferToBase64(buffer: ArrayBuffer | null): string {
  if (!buffer) return '';
  const bytes = new Uint8Array(buffer);
  let binary = '';
  for (let i = 0; i < bytes.byteLength; i++) {
    binary += String.fromCharCode(bytes[i]);
  }
  return window.btoa(binary);
}

export default usePushNotifications;
