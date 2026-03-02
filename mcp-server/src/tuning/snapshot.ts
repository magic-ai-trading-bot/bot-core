// @spec:FR-MCP-013 - Parameter Snapshots
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-03-self-tuning-engine.md

import { randomUUID } from "node:crypto";
import type { ParameterSnapshot } from "./types.js";
import { apiRequest } from "../client.js";
import { log } from "../types.js";

const MAX_SNAPSHOTS = 48;
const snapshots: ParameterSnapshot[] = [];

/**
 * Take a snapshot of current parameter state by reading from BotCore APIs.
 */
export async function takeSnapshot(): Promise<ParameterSnapshot> {
  // Fetch current settings, indicator/pipeline settings, and performance in parallel
  const [settingsRes, indicatorRes, performanceRes] = await Promise.all([
    apiRequest("rust", "/api/paper-trading/basic-settings", { timeoutMs: 10_000 }),
    apiRequest("rust", "/api/paper-trading/indicator-settings", { timeoutMs: 10_000 }),
    apiRequest("rust", "/api/trading/performance", { timeoutMs: 10_000 }),
  ]);

  // Merge basic settings with flattened signal_pipeline.* keys for snapshot lookup
  const params: Record<string, unknown> = settingsRes.success
    ? (settingsRes.data as Record<string, unknown>)
    : {};

  if (indicatorRes.success) {
    const indicatorData = indicatorRes.data as Record<string, unknown>;
    // Flatten signal_pipeline into dotted keys for audit trail lookup
    const pipeline = indicatorData?.signal_pipeline as Record<string, unknown> | undefined;
    if (pipeline) {
      for (const [key, val] of Object.entries(pipeline)) {
        params[`signal_pipeline.${key}`] = val;
      }
    }
    // Also store indicator/signal sections for completeness
    if (indicatorData?.indicators) params["_indicators"] = indicatorData.indicators;
    if (indicatorData?.signal) params["_signal"] = indicatorData.signal;
  }

  const snapshot: ParameterSnapshot = {
    id: randomUUID(),
    timestamp: new Date().toISOString(),
    parameters: params,
    performance: performanceRes.success
      ? (performanceRes.data as ParameterSnapshot["performance"])
      : undefined,
  };

  snapshots.push(snapshot);

  // Prune oldest if over limit
  while (snapshots.length > MAX_SNAPSHOTS) {
    snapshots.shift();
  }

  log("info", "Parameter snapshot taken", { snapshotId: snapshot.id });
  return snapshot;
}

/**
 * Get a snapshot by ID.
 */
export function getSnapshot(snapshotId: string): ParameterSnapshot | undefined {
  return snapshots.find((s) => s.id === snapshotId);
}

/**
 * Get the most recent snapshot.
 */
export function getLatestSnapshot(): ParameterSnapshot | undefined {
  return snapshots.length > 0 ? snapshots[snapshots.length - 1] : undefined;
}

/**
 * Get all snapshots (most recent first).
 */
export function getSnapshots(limit?: number): ParameterSnapshot[] {
  const result = [...snapshots].reverse();
  return limit ? result.slice(0, limit) : result;
}

/**
 * Restore parameters from a snapshot by applying them via BotCore API.
 */
export async function restoreFromSnapshot(
  snapshotId: string
): Promise<{ success: boolean; error?: string }> {
  const snapshot = getSnapshot(snapshotId);
  if (!snapshot) {
    return { success: false, error: `Snapshot ${snapshotId} not found` };
  }

  const res = await apiRequest("rust", "/api/paper-trading/basic-settings", {
    method: "PUT",
    body: snapshot.parameters,
    timeoutMs: 10_000,
  });

  if (res.success) {
    log("info", "Parameters restored from snapshot", { snapshotId });
    return { success: true };
  }

  return { success: false, error: res.error || "Failed to restore snapshot" };
}
