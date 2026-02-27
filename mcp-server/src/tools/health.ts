// @spec:FR-MCP-005 - System Health Check Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-01-mcp-server-setup.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, log } from "../types.js";

/**
 * Register health check and system monitoring tools.
 */
export function registerHealthTools(server: McpServer): void {
  // Tool: check_system_health - Validate all services are running
  server.registerTool(
    "check_system_health",
    {
      title: "Check System Health",
      description:
        "Check health status of all BotCore services: Rust Core Engine, Python AI Service, MongoDB, and infrastructure. Returns combined health report.",
      annotations: {
        readOnlyHint: true,
        openWorldHint: false,
      },
    },
    async () => {
      const results: Record<string, unknown> = {};
      let allHealthy = true;

      // Check Rust Core Engine
      try {
        const rust = await apiRequest("rust", "/api/health", {
          skipAuth: true,
          timeoutMs: 10_000,
        });
        results["rust_core_engine"] = {
          status: rust.success ? "healthy" : "unhealthy",
          port: 8080,
          data: rust.data,
          error: rust.error,
        };
        if (!rust.success) allHealthy = false;
      } catch {
        results["rust_core_engine"] = { status: "unreachable", port: 8080 };
        allHealthy = false;
      }

      // Check Python AI Service
      try {
        const python = await apiRequest("python", "/health", {
          skipAuth: true,
          timeoutMs: 10_000,
        });
        results["python_ai_service"] = {
          status: python.success ? "healthy" : "unhealthy",
          port: 8000,
          data: python.data,
          error: python.error,
        };
        if (!python.success) allHealthy = false;
      } catch {
        results["python_ai_service"] = { status: "unreachable", port: 8000 };
        allHealthy = false;
      }

      return toolSuccess({
        overall_status: allHealthy ? "healthy" : "degraded",
        timestamp: new Date().toISOString(),
        services: results,
      });
    }
  );

  // Tool: get_service_logs_summary - Check for errors in service logs
  server.registerTool(
    "get_service_logs_summary",
    {
      title: "Get Service Logs Summary",
      description:
        "Get a summary of recent errors and warnings from BotCore services. Checks Rust engine and Python AI service for ERROR/PANIC/CRITICAL level logs.",
      inputSchema: {
        service: z.enum(["rust", "python", "all"]).describe("Which service logs to check").default("all"),
      },
      annotations: {
        readOnlyHint: true,
        openWorldHint: false,
      },
    },
    async ({ service }: { service: string }) => {
      const results: Record<string, unknown> = {};

      if (service === "rust" || service === "all") {
        const res = await apiRequest("rust", "/api/monitoring/system", {
          timeoutMs: 10_000,
        });
        results["rust_core_engine"] = res.success
          ? res.data
          : { error: res.error };
      }

      if (service === "python" || service === "all") {
        const res = await apiRequest("python", "/health", {
          skipAuth: true,
          timeoutMs: 10_000,
        });
        results["python_ai_service"] = res.success
          ? res.data
          : { error: res.error };
      }

      return toolSuccess({
        timestamp: new Date().toISOString(),
        logs_summary: results,
      });
    }
  );

  // Tool: check_market_condition_health - Deep health check for AI pipeline
  server.registerTool(
    "check_market_condition_health",
    {
      title: "Check Market Condition Health",
      description:
        "Deep health check for the AI market condition pipeline. Tests MongoDB candle fetch + indicator calculation. Returns 'healthy' or error details with action_required.",
      annotations: {
        readOnlyHint: true,
        openWorldHint: false,
      },
    },
    async () => {
      try {
        const res = await apiRequest("python", "/ai/health/market-condition", {
          skipAuth: true,
          timeoutMs: 15_000,
        });
        if (res.success) {
          return toolSuccess({ healthy: true, ...(res.data as Record<string, unknown>) });
        } else {
          return toolSuccess({
            healthy: false,
            error: res.error || "Pipeline check failed",
            action_required: "Stop paper engine — investigate AI service",
          });
        }
      } catch {
        return toolSuccess({
          healthy: false,
          error: "Cannot reach Python AI service",
          action_required: "Python AI service may be down — stop paper engine",
        });
      }
    }
  );

  log("info", "Health & monitoring tools registered (3 tools)");
}
