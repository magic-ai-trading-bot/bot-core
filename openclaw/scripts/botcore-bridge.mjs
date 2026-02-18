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

const tool = process.argv[2];
const argsRaw = process.argv[3];

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
  MCP_AUTH_TOKEN   Bearer token for MCP server auth`);
  process.exit(0);
}

/** Parse SSE response to extract JSON-RPC result */
function parseSSE(text) {
  const dataLine = text.split("\n").find((l) => l.startsWith("data: "));
  if (!dataLine) return null;
  return JSON.parse(dataLine.replace("data: ", ""));
}

/** Make an MCP JSON-RPC request */
async function mcpRequest(method, params, sessionId) {
  const headers = {
    "Content-Type": "application/json",
    Accept: "application/json, text/event-stream",
  };
  if (MCP_TOKEN) headers["Authorization"] = `Bearer ${MCP_TOKEN}`;
  if (sessionId) headers["mcp-session-id"] = sessionId;

  const res = await fetch(`${MCP_URL}/mcp`, {
    method: "POST",
    headers,
    body: JSON.stringify({
      jsonrpc: "2.0",
      id: Date.now(),
      method,
      params,
    }),
  });

  const text = await res.text();
  const sid = res.headers.get("mcp-session-id");
  const parsed = parseSSE(text);

  return { data: parsed, sessionId: sid || sessionId };
}

async function main() {
  try {
    // Step 1: Initialize MCP session
    const init = await mcpRequest("initialize", {
      protocolVersion: "2025-03-26",
      capabilities: {},
      clientInfo: { name: "botcore-bridge", version: "1.0.0" },
    });

    if (!init.sessionId) {
      console.error("ERROR: Failed to establish MCP session");
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
        console.error(JSON.stringify(listRes.data, null, 2));
      }
      process.exit(0);
    }

    // Step 2: Call the requested tool
    let toolArgs = {};
    // Collect all args after tool name (handles shell-split multi-word text)
    const allArgs = process.argv.slice(3).join(" ").trim();
    if (allArgs) {
      try {
        toolArgs = JSON.parse(allArgs);
      } catch {
        // Try to parse flag-style args: --key value --flag
        const flagArgs = {};
        const parts = allArgs.split(/\s+/);
        for (let i = 0; i < parts.length; i++) {
          if (parts[i].startsWith("--")) {
            const key = parts[i].replace(/^--/, "");
            const val = parts[i + 1] && !parts[i + 1].startsWith("--") ? parts[++i] : true;
            flagArgs[key] = isNaN(val) ? val : Number(val);
          }
        }
        if (Object.keys(flagArgs).length > 0) {
          toolArgs = flagArgs;
        } else if (tool === "send_telegram_notification") {
          // Auto-wrap plain text as message for notification tool
          toolArgs = { message: allArgs };
        } else {
          console.error(`ERROR: Invalid arguments: ${allArgs}. Use JSON: '{"key":"value"}'`);
          process.exit(1);
        }
      }
    }

    // Normalize send_telegram_notification args — AI models may use
    // different field names (text, content, body) instead of "message"
    if (tool === "send_telegram_notification" && !toolArgs.message) {
      const msg = toolArgs.text || toolArgs.content || toolArgs.body || "";
      if (msg) {
        toolArgs = { message: msg, ...(toolArgs.parse_mode && { parse_mode: toolArgs.parse_mode }) };
      }
    }

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
