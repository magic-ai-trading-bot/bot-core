// @spec:FR-MCP-006 - Notification Tools
// @ref:specs/01-requirements/1.1-functional-requirements/FR-MCP.md

import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { z } from "zod";
import { toolSuccess, toolError, log } from "../types.js";

const TELEGRAM_BOT_TOKEN = process.env.TELEGRAM_BOT_TOKEN || "";
const TELEGRAM_CHAT_ID = process.env.TELEGRAM_CHAT_ID || "";

async function sendTelegramMessage(
  chatId: string,
  text: string,
  parseMode: string = "HTML"
): Promise<{ ok: boolean; description?: string }> {
  const url = `https://api.telegram.org/bot${TELEGRAM_BOT_TOKEN}/sendMessage`;
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      chat_id: chatId,
      text,
      parse_mode: parseMode,
    }),
  });
  return res.json() as Promise<{ ok: boolean; description?: string }>;
}

export function registerNotificationTools(server: McpServer): void {
  if (!TELEGRAM_BOT_TOKEN) {
    log("warn", "TELEGRAM_BOT_TOKEN not set, notification tools disabled");
    return;
  }

  server.registerTool(
    "send_telegram_notification",
    {
      title: "Send Telegram Notification",
      description:
        "Send a notification message to the user's Telegram. Use for alerts, reports, portfolio updates, and scheduled notifications.",
      inputSchema: {
        message: z.string().describe("The notification message to send"),
        parse_mode: z
          .enum(["HTML", "Markdown", "MarkdownV2"])
          .optional()
          .describe("Message format (default: HTML)"),
      },
      annotations: { readOnlyHint: false, openWorldHint: true },
    },
    async ({ message, parse_mode }) => {
      const chatId = TELEGRAM_CHAT_ID;
      if (!chatId) {
        return toolError(
          "TELEGRAM_CHAT_ID not configured. Set it in MCP server environment."
        );
      }

      try {
        const result = await sendTelegramMessage(
          chatId,
          message,
          parse_mode || "HTML"
        );
        if (result.ok) {
          return toolSuccess({
            sent: true,
            chat_id: chatId,
            message_preview: message.substring(0, 100),
          });
        }
        return toolError(
          `Telegram API error: ${result.description || "Unknown error"}`
        );
      } catch (err) {
        const errMsg = err instanceof Error ? err.message : String(err);
        log("error", "Failed to send Telegram notification", { error: errMsg });
        return toolError(`Failed to send: ${errMsg}`);
      }
    }
  );

  log("info", "Notification tools registered (1 tool)");
}
