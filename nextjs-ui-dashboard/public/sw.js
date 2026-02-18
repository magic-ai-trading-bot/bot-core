/**
 * Service Worker for Push Notifications
 *
 * @spec:FR-NOTIFICATION-002 - Push Notifications via Service Worker
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
 */

const CACHE_NAME = 'bot-core-v1';
const NOTIFICATION_ICON = '/android-chrome-192x192.png';
const NOTIFICATION_BADGE = '/android-chrome-192x192.png';

// Install event - cache essential assets
self.addEventListener('install', (event) => {
  console.log('[SW] Installing service worker...');
  self.skipWaiting();
});

// Activate event - clean up old caches
self.addEventListener('activate', (event) => {
  console.log('[SW] Activating service worker...');
  event.waitUntil(
    caches.keys().then((cacheNames) => {
      return Promise.all(
        cacheNames
          .filter((name) => name !== CACHE_NAME)
          .map((name) => caches.delete(name))
      );
    })
  );
  self.clients.claim();
});

// Push event - handle incoming push notifications
self.addEventListener('push', (event) => {
  console.log('[SW] Push notification received:', event);

  let data = {
    title: 'Trading Bot Notification',
    body: 'You have a new notification',
    icon: NOTIFICATION_ICON,
    badge: NOTIFICATION_BADGE,
    tag: 'bot-core-notification',
    requireInteraction: false,
    data: {}
  };

  if (event.data) {
    try {
      const payload = event.data.json();
      data = {
        title: payload.title || data.title,
        body: payload.body || payload.message || data.body,
        icon: payload.icon || data.icon,
        badge: payload.badge || data.badge,
        tag: payload.tag || `notification-${Date.now()}`,
        requireInteraction: payload.level === 'critical',
        data: {
          url: payload.url || '/',
          level: payload.level || 'info',
          type: payload.type || 'general',
          ...payload.data
        }
      };
    } catch (e) {
      // If not JSON, treat as plain text
      data.body = event.data.text();
    }
  }

  const options = {
    body: data.body,
    icon: data.icon,
    badge: data.badge,
    tag: data.tag,
    requireInteraction: data.requireInteraction,
    vibrate: [100, 50, 100], // Vibration pattern
    data: data.data,
    actions: getActionsForLevel(data.data?.level)
  };

  event.waitUntil(
    self.registration.showNotification(data.title, options)
  );
});

// Notification click handler
self.addEventListener('notificationclick', (event) => {
  console.log('[SW] Notification clicked:', event.notification.tag);

  event.notification.close();

  const urlToOpen = event.notification.data?.url || '/';
  const action = event.action;

  // Handle action buttons
  if (action === 'view') {
    event.waitUntil(openUrl(urlToOpen));
  } else if (action === 'dismiss') {
    // Just close the notification
    return;
  } else if (action === 'trade') {
    event.waitUntil(openUrl('/paper-trading'));
  } else {
    // Default click behavior - open the app
    event.waitUntil(openUrl(urlToOpen));
  }
});

// Notification close handler
self.addEventListener('notificationclose', (event) => {
  console.log('[SW] Notification closed:', event.notification.tag);
  // Could track analytics here
});

// Helper: Get action buttons based on notification level
function getActionsForLevel(level) {
  switch (level) {
    case 'critical':
    case 'error':
      return [
        { action: 'view', title: 'View Details', icon: '/icons/view.png' },
        { action: 'dismiss', title: 'Dismiss', icon: '/icons/dismiss.png' }
      ];
    case 'trade':
    case 'signal':
      return [
        { action: 'trade', title: 'Go to Trading', icon: '/icons/trade.png' },
        { action: 'view', title: 'View Signal', icon: '/icons/view.png' }
      ];
    default:
      return [
        { action: 'view', title: 'View', icon: '/icons/view.png' }
      ];
  }
}

// Helper: Open URL in appropriate window
async function openUrl(url) {
  const clientList = await self.clients.matchAll({
    type: 'window',
    includeUncontrolled: true
  });

  // Try to find an existing window to focus
  for (const client of clientList) {
    if (client.url.includes(self.location.origin)) {
      await client.focus();
      client.navigate(url);
      return;
    }
  }

  // No existing window, open a new one
  return self.clients.openWindow(url);
}

// Message handler - for communication with main thread
self.addEventListener('message', (event) => {
  console.log('[SW] Message received:', event.data);

  if (event.data?.type === 'SKIP_WAITING') {
    self.skipWaiting();
  }
});
