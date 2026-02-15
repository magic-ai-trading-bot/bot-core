// @spec:FR-MCP-012 - Tuning Audit Logger
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-03-self-tuning-engine.md

import { randomUUID } from "node:crypto";
import { appendFileSync, readFileSync, mkdirSync, existsSync } from "node:fs";
import type { AuditEntry, TuningTier } from "./types.js";
import { log } from "../types.js";

const AUDIT_DIR = process.env.TUNING_AUDIT_DIR || "/data/audit";
const AUDIT_FILE = `${AUDIT_DIR}/tuning-audit.jsonl`;

// Ensure audit directory exists
try {
  if (!existsSync(AUDIT_DIR)) {
    mkdirSync(AUDIT_DIR, { recursive: true });
  }
} catch {
  log("warn", `Cannot create audit directory ${AUDIT_DIR}, using /tmp`);
}

const effectiveFile = existsSync(AUDIT_DIR)
  ? AUDIT_FILE
  : "/tmp/tuning-audit.jsonl";

// Track last adjustment time per parameter (for cooldown enforcement)
const lastAdjustmentTime = new Map<string, number>();

/**
 * Log a parameter adjustment to the audit trail.
 */
export function logAdjustment(entry: Omit<AuditEntry, "id" | "timestamp">): AuditEntry {
  const full: AuditEntry = {
    id: randomUUID(),
    timestamp: new Date().toISOString(),
    ...entry,
  };

  try {
    appendFileSync(effectiveFile, JSON.stringify(full) + "\n");
  } catch (err) {
    log("error", "Failed to write audit log", { error: String(err) });
  }

  lastAdjustmentTime.set(entry.parameter, Date.now());
  log("info", "Tuning adjustment logged", {
    parameter: entry.parameter,
    tier: entry.tier,
    oldValue: entry.oldValue as string,
    newValue: entry.newValue as string,
  });

  return full;
}

/**
 * Get recent audit entries (most recent first).
 */
export function getAuditHistory(options?: {
  limit?: number;
  tier?: TuningTier;
  parameter?: string;
}): AuditEntry[] {
  try {
    const content = readFileSync(effectiveFile, "utf-8");
    let entries: AuditEntry[] = content
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line));

    if (options?.tier) {
      entries = entries.filter((e) => e.tier === options.tier);
    }
    if (options?.parameter) {
      entries = entries.filter((e) => e.parameter === options.parameter);
    }

    entries.reverse(); // Most recent first
    if (options?.limit) {
      entries = entries.slice(0, options.limit);
    }

    return entries;
  } catch {
    return [];
  }
}

/**
 * Check if a parameter is within its cooldown period.
 */
export function isInCooldown(paramKey: string, cooldownMs: number): boolean {
  const lastTime = lastAdjustmentTime.get(paramKey);
  if (!lastTime) return false;
  return Date.now() - lastTime < cooldownMs;
}

/**
 * Get time remaining in cooldown (in seconds).
 */
export function getCooldownRemaining(paramKey: string, cooldownMs: number): number {
  const lastTime = lastAdjustmentTime.get(paramKey);
  if (!lastTime) return 0;
  const remaining = cooldownMs - (Date.now() - lastTime);
  return remaining > 0 ? Math.ceil(remaining / 1000) : 0;
}
