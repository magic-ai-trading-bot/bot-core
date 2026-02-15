// @spec:FR-MCP-008 - Per-Tool Rate Limiting
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-mcp-tool-implementation.md

import { toolError } from "./types.js";

interface RateLimitEntry {
  timestamps: number[];
}

const buckets = new Map<string, RateLimitEntry>();

// Default limits per tool category
const CATEGORY_LIMITS: Record<string, { max: number; windowMs: number }> = {
  market: { max: 1000, windowMs: 60_000 },
  "paper-trading": { max: 300, windowMs: 60_000 },
  "real-trading": { max: 30, windowMs: 60_000 },
  ai: { max: 60, windowMs: 60_000 },
  monitoring: { max: 300, windowMs: 60_000 },
  tasks: { max: 60, windowMs: 60_000 },
  backtests: { max: 30, windowMs: 60_000 },
  auth: { max: 30, windowMs: 60_000 },
  settings: { max: 60, windowMs: 60_000 },
  notifications: { max: 60, windowMs: 60_000 },
  health: { max: 300, windowMs: 60_000 },
};

// Clean up old entries every 5 minutes
setInterval(() => {
  const now = Date.now();
  for (const [key, entry] of buckets) {
    entry.timestamps = entry.timestamps.filter((t) => now - t < 120_000);
    if (entry.timestamps.length === 0) buckets.delete(key);
  }
}, 5 * 60_000);

/**
 * Check rate limit for a tool call. Returns null if allowed, or error result if limited.
 */
export function checkRateLimit(
  category: string
): ReturnType<typeof toolError> | null {
  const limit = CATEGORY_LIMITS[category] || { max: 100, windowMs: 60_000 };
  const now = Date.now();

  let entry = buckets.get(category);
  if (!entry) {
    entry = { timestamps: [] };
    buckets.set(category, entry);
  }

  // Remove timestamps outside window
  entry.timestamps = entry.timestamps.filter(
    (t) => now - t < limit.windowMs
  );

  if (entry.timestamps.length >= limit.max) {
    const retryAfter = Math.ceil(
      (entry.timestamps[0] + limit.windowMs - now) / 1000
    );
    return toolError(
      `Rate limit exceeded for ${category} tools (${limit.max}/${limit.windowMs / 1000}s). Retry after ${retryAfter}s.`
    );
  }

  entry.timestamps.push(now);
  return null;
}
