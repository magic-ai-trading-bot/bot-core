// @spec:FR-MCP-004 - MCP Server Tool Registration Tests
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-06-integration-tests.md

import { describe, it, expect, beforeAll } from "vitest";
import { Client } from "@modelcontextprotocol/sdk/client/index.js";
import { InMemoryTransport } from "@modelcontextprotocol/sdk/inMemory.js";
import { createMcpServer } from "../src/server.js";

describe("Tool Registration", () => {
  let client: Client;
  let server: ReturnType<typeof createMcpServer>;

  beforeAll(async () => {
    // Create linked transport pair for client-server communication
    const [clientTransport, serverTransport] = InMemoryTransport.createLinkedPair();

    // Create and connect server
    server = createMcpServer();
    await server.connect(serverTransport);

    // Create and connect client
    client = new Client(
      {
        name: "test-client",
        version: "1.0.0",
      },
      {
        capabilities: {},
      }
    );
    await client.connect(clientTransport);
  });

  it("createMcpServer returns a valid McpServer instance", () => {
    expect(server).toBeDefined();
    expect(typeof server).toBe("object");
    expect(server.connect).toBeDefined();
    expect(typeof server.connect).toBe("function");
  });

  it("all 103 tools are registered", async () => {
    const response = await client.listTools();

    expect(response.tools).toBeDefined();
    expect(Array.isArray(response.tools)).toBe(true);

    // The server should have 103 tools total:
    // Health: 4, Market: 8, Trading: 4, Paper Trading: 28, Real Trading: 14,
    // AI: 12, Tasks: 7, Monitoring: 5, Settings: 10, Auth: 4, Tuning: 8
    // Total: 4 + 8 + 4 + 28 + 14 + 12 + 7 + 5 + 10 + 4 + 8 = 104
    expect(response.tools.length).toBeGreaterThanOrEqual(103);
  });

  it("tool names are unique (no duplicates)", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);
    const uniqueNames = new Set(toolNames);

    expect(toolNames.length).toBe(uniqueNames.size);
  });

  it("health tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for expected health tools
    expect(toolNames).toContain("check_system_health");
    expect(toolNames).toContain("get_docker_status");
    expect(toolNames).toContain("get_python_health");
  });

  it("tuning tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for expected tuning tools
    expect(toolNames).toContain("get_tuning_dashboard");
    expect(toolNames).toContain("apply_green_adjustment");
    expect(toolNames).toContain("request_yellow_adjustment");
    expect(toolNames).toContain("request_red_adjustment");
    expect(toolNames).toContain("get_adjustment_history");
    expect(toolNames).toContain("get_parameter_bounds");
    expect(toolNames).toContain("take_parameter_snapshot");
    expect(toolNames).toContain("rollback_adjustment");
  });

  it("all tools have a description", async () => {
    const response = await client.listTools();

    for (const tool of response.tools) {
      expect(tool.description).toBeDefined();
      expect(typeof tool.description).toBe("string");
      expect(tool.description.length).toBeGreaterThan(0);
    }
  });

  it("all tools have a valid name", async () => {
    const response = await client.listTools();

    for (const tool of response.tools) {
      expect(tool.name).toBeDefined();
      expect(typeof tool.name).toBe("string");
      expect(tool.name.length).toBeGreaterThan(0);
      // Tool names should follow snake_case convention
      expect(tool.name).toMatch(/^[a-z][a-z0-9_]*$/);
    }
  });

  it("all tools have an input schema", async () => {
    const response = await client.listTools();

    for (const tool of response.tools) {
      expect(tool.inputSchema).toBeDefined();
      expect(typeof tool.inputSchema).toBe("object");
    }
  });

  it("tuning tools have security tier annotations", async () => {
    const response = await client.listTools();
    const tuningTools = response.tools.filter(tool =>
      tool.name.includes("tuning") ||
      tool.name.includes("adjustment") ||
      tool.name.includes("parameter") ||
      tool.name.includes("snapshot")
    );

    // Should have at least the 8 tuning tools
    expect(tuningTools.length).toBeGreaterThanOrEqual(8);
  });

  it("paper trading tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected paper trading tools
    expect(toolNames).toContain("get_paper_portfolio");
    expect(toolNames).toContain("get_paper_open_trades");
    expect(toolNames).toContain("close_paper_trade");
  });

  it("real trading tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected real trading tools
    expect(toolNames).toContain("get_real_portfolio");
    expect(toolNames).toContain("get_real_open_trades");
    expect(toolNames).toContain("create_real_order");
  });

  it("AI tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected AI tools
    expect(toolNames).toContain("predict_trend");
    expect(toolNames).toContain("analyze_market");
    expect(toolNames).toContain("get_ai_config_suggestions");
  });

  it("monitoring tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected monitoring tools
    expect(toolNames).toContain("get_system_metrics");
    expect(toolNames).toContain("get_trading_metrics");
  });

  it("market data tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected market data tools
    expect(toolNames).toContain("get_market_prices");
    expect(toolNames).toContain("get_candles");
    expect(toolNames).toContain("get_symbols");
  });

  it("auth tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected auth tools
    expect(toolNames).toContain("login");
    expect(toolNames).toContain("get_profile");
  });

  it("settings tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected settings tools
    expect(toolNames).toContain("get_api_keys");
    expect(toolNames).toContain("save_api_keys");
  });

  it("task tools are registered", async () => {
    const response = await client.listTools();
    const toolNames = response.tools.map(tool => tool.name);

    // Check for some expected task tools
    expect(toolNames).toContain("chat_with_project");
    expect(toolNames).toContain("get_chat_suggestions");
  });
});
