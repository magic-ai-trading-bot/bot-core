// @spec:FR-MCP-007 - Settings Management Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-tool-implementation.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

export function registerSettingsTools(server: McpServer): void {
  // All tools target Rust service (port 8080)

  // ====== API Keys Management ======

  server.registerTool(
    "get_api_keys",
    {
      title: "Get API Keys Status",
      description: "Get status of configured API keys (masked for security). Shows which exchanges are configured.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/settings/api-keys");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch API keys status");
    }
  );

  server.registerTool(
    "save_api_keys",
    {
      title: "Save API Keys",
      description: "⚠️ SENSITIVE: Save or update API keys for an exchange. Keys are encrypted at rest.",
      inputSchema: {
        exchange: z.string().describe("Exchange name (e.g., binance)"),
        api_key: z.string().describe("API key from the exchange"),
        secret_key: z.string().describe("Secret key from the exchange"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ exchange, api_key, secret_key }: { exchange: string; api_key: string; secret_key: string }) => {
      const res = await apiRequest("rust", "/api/settings/api-keys", {
        method: "POST",
        body: { exchange, api_key, secret_key },
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to save API keys");
    }
  );

  server.registerTool(
    "delete_api_keys",
    {
      title: "Delete API Keys",
      description: "Delete all stored API keys. This will disable real trading until new keys are configured.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/settings/api-keys", {
        method: "DELETE",
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to delete API keys");
    }
  );

  server.registerTool(
    "test_api_keys",
    {
      title: "Test API Keys",
      description: "Test configured API keys by making a test request to the exchange. Verifies connectivity and permissions.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/settings/api-keys/test", {
        method: "POST",
        body: {},
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "API keys test failed");
    }
  );

  // ====== Notification Settings ======

  server.registerTool(
    "get_notification_preferences",
    {
      title: "Get Notification Preferences",
      description: "Get current notification preferences including enabled channels and alert types.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/notifications/preferences");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch notification preferences");
    }
  );

  server.registerTool(
    "update_notification_preferences",
    {
      title: "Update Notification Preferences",
      description: "Update notification preferences including which events trigger notifications and delivery channels.",
      inputSchema: {
        preferences: z.record(z.unknown()).describe("Notification preferences object"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ preferences }: { preferences: Record<string, unknown> }) => {
      const res = await apiRequest("rust", "/api/notifications/preferences", {
        method: "PUT",
        body: preferences,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to update notification preferences");
    }
  );

  server.registerTool(
    "subscribe_push_notifications",
    {
      title: "Subscribe to Push Notifications",
      description: "Subscribe to browser push notifications. Requires user permission in browser.",
      inputSchema: {
        subscription: z.record(z.unknown()).describe("Push notification subscription object from browser"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ subscription }: { subscription: Record<string, unknown> }) => {
      const res = await apiRequest("rust", "/api/notifications/push/subscribe", {
        method: "POST",
        body: subscription,
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to subscribe to push notifications");
    }
  );

  server.registerTool(
    "unsubscribe_push_notifications",
    {
      title: "Unsubscribe from Push Notifications",
      description: "Unsubscribe from browser push notifications.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/notifications/push/subscribe", {
        method: "DELETE",
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to unsubscribe from push notifications");
    }
  );

  server.registerTool(
    "test_notification",
    {
      title: "Test Notification",
      description: "Send a test notification through all enabled channels to verify configuration.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/notifications/test", {
        method: "POST",
        body: {},
      });
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to send test notification");
    }
  );

  server.registerTool(
    "get_vapid_key",
    {
      title: "Get VAPID Public Key",
      description: "Get the VAPID public key required for browser push notifications setup.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const res = await apiRequest("rust", "/api/notifications/vapid-key");
      return res.success ? toolSuccess(res.data) : toolError(res.error || "Failed to fetch VAPID key");
    }
  );

  log("info", "Registered 10 settings management tools (4 API keys + 6 notifications)");
}
