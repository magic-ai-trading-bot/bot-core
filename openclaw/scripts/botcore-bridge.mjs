#!/usr/bin/env node
// @spec:FR-MCP-016 - BotCore MCP Bridge CLI
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-04-openclaw-deployment.md
//
// CLI bridge between OpenClaw (exec tool) and BotCore MCP Server.
// Usage: botcore <tool-name> [json-args]
// Example: botcore get_system_health
// Example: botcore get_tuning_dashboard
// Example: botcore apply_green_adjustment '{"parameter":"rsi_oversold","new_value":25,"reasoning":"Bearish trend"}'

const MCP_URL = process.env.MCP_URL || "http://mcp-server:8090";
const MCP_TOKEN = process.env.MCP_AUTH_TOKEN || "";
const REQUEST_TIMEOUT_MS = Number(process.env.MCP_TIMEOUT_MS) || 30000;
const MAX_RETRIES = 2;
const RETRY_DELAY_MS = 2000;

const tool = process.argv[2];

if (!tool || tool === "--help" || tool === "-h") {
  console.log(`BotCore MCP Bridge - CLI tool for interacting with the trading bot

Usage: botcore <tool-name> [json-args]
       botcore --list              List all available tools
       botcore --help              Show this help

Examples:
  botcore get_system_health
  botcore get_tuning_dashboard
  botcore get_market_prices '{"symbols":["BTCUSDT","ETHUSDT"]}'
  botcore apply_green_adjustment '{"parameter":"rsi_oversold","new_value":25,"reasoning":"Bearish market"}'

Environment:
  MCP_URL          MCP server URL (default: http://mcp-server:8090)
  MCP_AUTH_TOKEN   Bearer token for MCP server auth
  MCP_TIMEOUT_MS   Request timeout in ms (default: 30000)`);
  process.exit(0);
}

/** Parse SSE response to extract JSON-RPC result */
function parseSSE(text) {
  const dataLine = text.split("\n").find((l) => l.startsWith("data: "));
  if (!dataLine) return null;
  try {
    return JSON.parse(dataLine.replace("data: ", ""));
  } catch {
    return null;
  }
}

/** Sleep helper */
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Make an MCP JSON-RPC request with timeout and retry */
async function mcpRequest(method, params, sessionId, retries = MAX_RETRIES) {
  const headers = {
    "Content-Type": "application/json",
    Accept: "application/json, text/event-stream",
  };
  if (MCP_TOKEN) headers["Authorization"] = `Bearer ${MCP_TOKEN}`;
  if (sessionId) headers["mcp-session-id"] = sessionId;

  for (let attempt = 0; attempt <= retries; attempt++) {
    try {
      const controller = new AbortController();
      const timer = setTimeout(() => controller.abort(), REQUEST_TIMEOUT_MS);

      const res = await fetch(`${MCP_URL}/mcp`, {
        method: "POST",
        headers,
        body: JSON.stringify({
          jsonrpc: "2.0",
          id: Date.now(),
          method,
          params,
        }),
        signal: controller.signal,
      });
      clearTimeout(timer);

      if (!res.ok) {
        throw new Error(`HTTP ${res.status}: ${res.statusText}`);
      }

      const text = await res.text();
      const sid = res.headers.get("mcp-session-id");
      const parsed = parseSSE(text);

      if (!parsed && method !== "notifications/initialized") {
        throw new Error(`Invalid MCP response (no JSON-RPC data in SSE)`);
      }

      return { data: parsed, sessionId: sid || sessionId };
    } catch (err) {
      const isLast = attempt >= retries;
      const isTimeout = err.name === "AbortError";
      const errMsg = isTimeout ? `Timeout after ${REQUEST_TIMEOUT_MS}ms` : err.message;

      if (isLast) {
        throw new Error(`MCP ${method} failed after ${attempt + 1} attempt(s): ${errMsg}`);
      }
      const delay = RETRY_DELAY_MS * (attempt + 1);
      console.error(`WARN: ${method} attempt ${attempt + 1} failed (${errMsg}), retrying in ${delay}ms...`);
      await sleep(delay);
    }
  }
}

/** Parse tool arguments from CLI */
function parseToolArgs(tool) {
  const allArgs = process.argv.slice(3).join(" ").trim();
  if (!allArgs) return {};

  // Try JSON first
  try {
    return JSON.parse(allArgs);
  } catch {
    // JSON parse failed — maybe literal newlines in string values.
    // Try escaping unescaped newlines inside JSON string values.
    try {
      const fixed = allArgs.replace(/\n/g, "\\n").replace(/\r/g, "\\r").replace(/\t/g, "\\t");
      return JSON.parse(fixed);
    } catch {
      // noop
    }
  }

  // Try flag-style: --key value --flag
  const flagArgs = {};
  const parts = allArgs.split(/\s+/);
  for (let i = 0; i < parts.length; i++) {
    if (parts[i].startsWith("--")) {
      const key = parts[i].replace(/^--/, "");
      const val =
        parts[i + 1] && !parts[i + 1].startsWith("--") ? parts[++i] : true;
      flagArgs[key] = isNaN(val) ? val : Number(val);
    }
  }
  if (Object.keys(flagArgs).length > 0) return flagArgs;

  // For send_telegram_notification, auto-wrap plain text
  if (tool === "send_telegram_notification") {
    return { message: allArgs };
  }

  console.error(
    `ERROR: Invalid arguments: ${allArgs}. Use JSON: '{"key":"value"}'`
  );
  process.exit(1);
}

/** Normalize notification args (AI may use text/content/body instead of message) */
function normalizeNotificationArgs(tool, args) {
  if (tool !== "send_telegram_notification" || args.message) return args;
  const msg = args.text || args.content || args.body || "";
  if (!msg) return args;
  return {
    message: msg,
    ...(args.parse_mode && { parse_mode: args.parse_mode }),
  };
}

async function main() {
  try {
    // Step 1: Initialize MCP session
    const init = await mcpRequest("initialize", {
      protocolVersion: "2025-03-26",
      capabilities: {},
      clientInfo: { name: "botcore-bridge", version: "1.0.0" },
    });

    if (init.data?.error) {
      console.error(`ERROR: MCP init error: ${init.data.error.message}`);
      process.exit(1);
    }

    if (!init.sessionId) {
      console.error("ERROR: Failed to establish MCP session (no session ID)");
      process.exit(1);
    }

    // Handle --list: list all available tools
    if (tool === "--list") {
      const listRes = await mcpRequest("tools/list", {}, init.sessionId);
      if (listRes.data?.result?.tools) {
        const tools = listRes.data.result.tools;
        console.log(`Available tools (${tools.length}):\n`);
        for (const t of tools) {
          console.log(`  ${t.name}`);
          if (t.description) {
            const desc = t.description.split("\n")[0].substring(0, 80);
            console.log(`    ${desc}`);
          }
        }
      } else {
        console.error("ERROR: Failed to list tools");
        if (listRes.data) console.error(JSON.stringify(listRes.data, null, 2));
      }
      process.exit(0);
    }

    // Step 2: Parse and normalize arguments
    let toolArgs = parseToolArgs(tool);
    toolArgs = normalizeNotificationArgs(tool, toolArgs);

    // Step 3: Call the requested tool
    const result = await mcpRequest(
      "tools/call",
      { name: tool, arguments: toolArgs },
      init.sessionId
    );

    if (result.data?.error) {
      console.error(`ERROR: ${result.data.error.message}`);
      process.exit(1);
    }

    if (result.data?.result) {
      const content = result.data.result.content;
      if (Array.isArray(content)) {
        for (const item of content) {
          if (item.type === "text") {
            // Try to pretty-print JSON, fall back to raw text
            try {
              const parsed = JSON.parse(item.text);
              console.log(JSON.stringify(parsed, null, 2));
            } catch {
              console.log(item.text);
            }
          }
        }
      } else {
        console.log(JSON.stringify(result.data.result, null, 2));
      }
    }
  } catch (err) {
    console.error(`ERROR: ${err.message}`);
    if (err.cause) console.error(`Cause: ${err.cause.message || err.cause}`);
    process.exit(1);
  }
}

main();
