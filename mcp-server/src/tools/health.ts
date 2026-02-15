// @spec:FR-MCP-005 - System Health Check Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-01-mcp-server-setup.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";

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

      // Check Prometheus (monitoring)
      try {
        const prom = await apiRequest(
          "prometheus",
          "/api/v1/query?query=up",
          { skipAuth: true, timeoutMs: 5_000 }
        );
        results["prometheus"] = {
          status: prom.success ? "healthy" : "unhealthy",
          port: 9090,
        };
      } catch {
        results["prometheus"] = { status: "unreachable", port: 9090 };
      }

      return toolSuccess({
        overall_status: allHealthy ? "healthy" : "degraded",
        timestamp: new Date().toISOString(),
        services: results,
      });
    }
  );

  // Tool: get_system_metrics - Get container/host metrics from Prometheus
  server.registerTool(
    "get_system_metrics",
    {
      title: "Get System Metrics",
      description:
        "Get CPU, memory, and container metrics for all BotCore services from Prometheus. Shows resource usage per container.",
      inputSchema: {
        metric: z.enum(["memory", "cpu", "all"]).describe("Which metrics to fetch").default("all"),
      },
      annotations: {
        readOnlyHint: true,
        openWorldHint: false,
      },
    },
    async ({ metric }: { metric: string }) => {
      const queries: Record<string, string> = {};

      if (metric === "memory" || metric === "all") {
        queries["container_memory_usage"] =
          "container_memory_usage_bytes{name=~'.+'}";
        queries["container_memory_limit"] =
          "container_spec_memory_limit_bytes{name=~'.+'}";
      }
      if (metric === "cpu" || metric === "all") {
        queries["container_cpu_usage"] =
          "rate(container_cpu_usage_seconds_total{name=~'.+'}[5m])";
      }

      const results: Record<string, unknown> = {};

      for (const [key, query] of Object.entries(queries)) {
        try {
          const res = await apiRequest(
            "prometheus",
            `/api/v1/query?query=${encodeURIComponent(query)}`,
            { skipAuth: true, timeoutMs: 10_000 }
          );
          if (res.success) {
            results[key] = res.data;
          } else {
            results[key] = { error: res.error };
          }
        } catch {
          results[key] = { error: "prometheus_unreachable" };
        }
      }

      return toolSuccess({
        timestamp: new Date().toISOString(),
        metrics: results,
      });
    }
  );

  // Tool: get_docker_status - Get container status overview
  server.registerTool(
    "get_docker_status",
    {
      title: "Get Docker Container Status",
      description:
        "Get status of all Docker containers including health, uptime, restart count. Covers all 15 services: MongoDB, Redis, RabbitMQ, Celery Worker/Beat, Flower, Kong, Prometheus, Grafana, Rust Engine, Python AI, Dashboard, MCP Server, OpenClaw.",
      annotations: {
        readOnlyHint: true,
        openWorldHint: false,
      },
    },
    async () => {
      const upQuery = "up";
      try {
        const res = await apiRequest(
          "prometheus",
          `/api/v1/query?query=${encodeURIComponent(upQuery)}`,
          { skipAuth: true, timeoutMs: 10_000 }
        );

        if (res.success) {
          return toolSuccess({
            timestamp: new Date().toISOString(),
            containers: res.data,
          });
        }
        return toolError(`Failed to query Prometheus: ${res.error}`);
      } catch {
        return toolError(
          "Prometheus is unreachable. Cannot fetch container status."
        );
      }
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

  log("info", "Health & monitoring tools registered (4 tools)");
}
