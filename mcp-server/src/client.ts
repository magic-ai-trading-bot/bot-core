// @spec:FR-MCP-003 - BotCore API Client
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-01-mcp-server-setup.md

import { getJwtToken } from "./auth.js";
import { log, type BotCoreResponse } from "./types.js";

const RUST_API_URL = process.env.RUST_API_URL || "http://localhost:8080";
const PYTHON_API_URL = process.env.PYTHON_API_URL || "http://localhost:8000";
const PROMETHEUS_URL = process.env.PROMETHEUS_URL || "http://prometheus:9090";

export type ServiceTarget = "rust" | "python" | "prometheus";

function getBaseUrl(service: ServiceTarget): string {
  switch (service) {
    case "rust":
      return RUST_API_URL;
    case "python":
      return PYTHON_API_URL;
    case "prometheus":
      return PROMETHEUS_URL;
  }
}

interface RequestOptions {
  method?: "GET" | "POST" | "PUT" | "DELETE";
  body?: unknown;
  timeoutMs?: number;
  skipAuth?: boolean;
}

/**
 * Make an authenticated HTTP request to a BotCore service.
 * Handles JWT auth, timeouts, retries, and error normalization.
 */
export async function apiRequest<T = unknown>(
  service: ServiceTarget,
  path: string,
  options: RequestOptions = {}
): Promise<BotCoreResponse<T>> {
  const { method = "GET", body, timeoutMs = 30_000, skipAuth = false } = options;
  const baseUrl = getBaseUrl(service);
  const url = `${baseUrl}${path}`;

  const headers: Record<string, string> = {
    "Content-Type": "application/json",
  };

  if (!skipAuth && service !== "prometheus") {
    const jwt = await getJwtToken();
    if (jwt) {
      headers["Authorization"] = `Bearer ${jwt}`;
    }
  }

  const controller = new AbortController();
  const timeout = setTimeout(() => controller.abort(), timeoutMs);

  try {
    const res = await fetch(url, {
      method,
      headers,
      body: body ? JSON.stringify(body) : undefined,
      signal: controller.signal,
    });

    clearTimeout(timeout);

    const text = await res.text();
    let data: unknown;
    try {
      data = JSON.parse(text);
    } catch {
      data = { message: text };
    }

    if (!res.ok) {
      // Retry once on 5xx
      if (res.status >= 500 && method === "GET") {
        log("warn", `Retrying ${method} ${path} after ${res.status}`);
        await new Promise((r) => setTimeout(r, 2000));
        return apiRequest(service, path, { ...options, timeoutMs: timeoutMs - 2000 });
      }

      const errMsg =
        (data as Record<string, unknown>)?.error ||
        (data as Record<string, unknown>)?.detail ||
        (data as Record<string, unknown>)?.message ||
        `HTTP ${res.status}`;
      return { success: false, error: String(errMsg) };
    }

    // Normalize response: Rust returns {success, data}, Python may return directly
    if (
      typeof data === "object" &&
      data !== null &&
      "success" in (data as Record<string, unknown>)
    ) {
      return data as BotCoreResponse<T>;
    }

    return { success: true, data: data as T };
  } catch (err) {
    clearTimeout(timeout);
    const message =
      err instanceof Error && err.name === "AbortError"
        ? `Request timeout after ${timeoutMs}ms`
        : err instanceof Error
          ? err.message
          : String(err);
    log("error", `API request failed: ${method} ${path}`, { error: message });
    return { success: false, error: message };
  }
}
