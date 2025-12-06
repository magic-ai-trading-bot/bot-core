/**
 * Notification Types
 *
 * Types for the notification system including trading events,
 * signals, and system alerts.
 * @spec:FR-NOTIFICATION-001 - Notification System Types
 */

export type NotificationType =
  | 'trade_executed'      // When a trade is opened
  | 'trade_closed'        // When a trade is closed
  | 'stop_loss_hit'       // Stop loss triggered
  | 'take_profit_hit'     // Take profit triggered
  | 'order_filled'        // Pending order executed
  | 'order_cancelled'     // Order cancelled
  | 'signal_generated'    // New AI trading signal
  | 'price_alert'         // Price target reached
  | 'risk_warning'        // Risk threshold warning
  | 'system_alert'        // System notifications
  | 'mode_switch';        // Trading mode changed

export type NotificationPriority = 'low' | 'medium' | 'high' | 'critical';

// Named AppNotification to avoid conflict with browser's global Notification API
export interface AppNotification {
  id: string;
  type: NotificationType;
  title: string;
  message: string;
  priority: NotificationPriority;
  read: boolean;
  createdAt: string;
  data?: NotificationData;
}

export interface NotificationData {
  // Trade-related
  tradeId?: string;
  symbol?: string;
  side?: 'long' | 'short';
  entryPrice?: number;
  exitPrice?: number;
  quantity?: number;
  pnl?: number;
  pnlPercentage?: number;

  // Order-related
  orderId?: string;
  orderType?: 'market' | 'limit' | 'stop_limit';
  orderStatus?: string;

  // Signal-related
  signalId?: string;
  confidence?: number;
  strategy?: string;

  // Price alert
  targetPrice?: number;
  currentPrice?: number;

  // Risk warning
  riskLevel?: number;
  riskMessage?: string;

  // Mode switch
  fromMode?: 'paper' | 'real';
  toMode?: 'paper' | 'real';

  // Generic link for more details
  actionUrl?: string;
}

export interface NotificationGroup {
  date: string;
  notifications: AppNotification[];
}

export interface NotificationStats {
  total: number;
  unread: number;
  byType: Record<NotificationType, number>;
}

// API Response types
export interface NotificationListResponse {
  notifications: AppNotification[];
  total: number;
  unread: number;
  page: number;
  limit: number;
  hasMore: boolean;
}

export interface NotificationMarkReadResponse {
  success: boolean;
  notificationId: string;
}

export interface NotificationMarkAllReadResponse {
  success: boolean;
  count: number;
}

export interface NotificationDeleteResponse {
  success: boolean;
  notificationId: string;
}

// WebSocket notification event
export interface NotificationWebSocketEvent {
  type: 'notification';
  action: 'new' | 'read' | 'deleted';
  notification?: AppNotification;
  notificationId?: string;
}

// Notification preferences
export interface NotificationPreferences {
  enabled: boolean;
  sound: boolean;
  desktop: boolean;
  email: boolean;
  types: {
    trade_executed: boolean;
    trade_closed: boolean;
    stop_loss_hit: boolean;
    take_profit_hit: boolean;
    order_filled: boolean;
    order_cancelled: boolean;
    signal_generated: boolean;
    price_alert: boolean;
    risk_warning: boolean;
    system_alert: boolean;
    mode_switch: boolean;
  };
}

// Helper functions for notification display
export function getNotificationIcon(type: NotificationType): string {
  switch (type) {
    case 'trade_executed':
      return 'play-circle';
    case 'trade_closed':
      return 'check-circle';
    case 'stop_loss_hit':
      return 'alert-triangle';
    case 'take_profit_hit':
      return 'trending-up';
    case 'order_filled':
      return 'check-square';
    case 'order_cancelled':
      return 'x-circle';
    case 'signal_generated':
      return 'zap';
    case 'price_alert':
      return 'bell';
    case 'risk_warning':
      return 'alert-octagon';
    case 'system_alert':
      return 'info';
    case 'mode_switch':
      return 'refresh-cw';
    default:
      return 'bell';
  }
}

export function getNotificationColor(type: NotificationType): string {
  switch (type) {
    case 'trade_executed':
      return '#3B82F6'; // Blue
    case 'trade_closed':
      return '#10B981'; // Green
    case 'stop_loss_hit':
      return '#EF4444'; // Red
    case 'take_profit_hit':
      return '#10B981'; // Green
    case 'order_filled':
      return '#8B5CF6'; // Purple
    case 'order_cancelled':
      return '#6B7280'; // Gray
    case 'signal_generated':
      return '#F59E0B'; // Amber
    case 'price_alert':
      return '#06B6D4'; // Cyan
    case 'risk_warning':
      return '#EF4444'; // Red
    case 'system_alert':
      return '#6B7280'; // Gray
    case 'mode_switch':
      return '#3B82F6'; // Blue
    default:
      return '#6B7280';
  }
}

export function getPriorityColor(priority: NotificationPriority): string {
  switch (priority) {
    case 'critical':
      return '#EF4444'; // Red
    case 'high':
      return '#F59E0B'; // Amber
    case 'medium':
      return '#3B82F6'; // Blue
    case 'low':
      return '#6B7280'; // Gray
    default:
      return '#6B7280';
  }
}
