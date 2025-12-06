/**
 * Notification Hook
 *
 * Wrapper hook for easy access to NotificationContext.
 * @spec:FR-NOTIFICATION-001 - Notification System Hook
 */

import { useNotificationContext } from '@/contexts/NotificationContext';

export function useNotification() {
  return useNotificationContext();
}
