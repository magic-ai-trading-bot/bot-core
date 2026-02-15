// @spec:FR-MCP-014 - Self-Tuning MCP Tools
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-03-self-tuning-engine.md

import { z } from "zod";
import type { McpServer } from "@modelcontextprotocol/sdk/server/mcp.js";
import { apiRequest } from "../client.js";
import { toolSuccess, toolError, log } from "../types.js";
import { PARAMETER_BOUNDS, validateAdjustment, getParametersByTier } from "../tuning/bounds.js";
import { logAdjustment, getAuditHistory, isInCooldown, getCooldownRemaining } from "../tuning/audit.js";
import { takeSnapshot, getSnapshots, restoreFromSnapshot, getLatestSnapshot } from "../tuning/snapshot.js";
import { generateConfirmToken, validateConfirmToken, hashParams } from "../security.js";

export function registerTuningTools(server: McpServer): void {
  // ── 1. Tuning Dashboard ──
  server.registerTool(
    "get_tuning_dashboard",
    {
      title: "Get Tuning Dashboard",
      description:
        "Aggregated view of trading performance, current settings, AI suggestions, and open positions. Use this to assess if parameter adjustments are needed.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const [settings, performance, suggestions, openTrades, strategySettings] =
        await Promise.all([
          apiRequest("rust", "/api/paper-trading/basic-settings", { timeoutMs: 10_000 }),
          apiRequest("rust", "/api/trading/performance", { timeoutMs: 10_000 }),
          apiRequest("rust", "/api/paper-trading/config-suggestions/latest", { timeoutMs: 10_000 }),
          apiRequest("rust", "/api/paper-trading/trades/open", { timeoutMs: 10_000 }),
          apiRequest("rust", "/api/paper-trading/strategy-settings", { timeoutMs: 10_000 }),
        ]);

      const recentAdjustments = getAuditHistory({ limit: 5 });
      const latestSnapshot = getLatestSnapshot();

      return toolSuccess({
        current_settings: settings.success ? settings.data : { error: settings.error },
        strategy_settings: strategySettings.success ? strategySettings.data : { error: strategySettings.error },
        performance: performance.success ? performance.data : { error: performance.error },
        ai_suggestions: suggestions.success ? suggestions.data : { error: suggestions.error },
        open_positions_count: openTrades.success && Array.isArray(openTrades.data)
          ? (openTrades.data as unknown[]).length
          : "unknown",
        recent_adjustments: recentAdjustments,
        last_snapshot: latestSnapshot
          ? { id: latestSnapshot.id, timestamp: latestSnapshot.timestamp }
          : null,
      });
    }
  );

  // ── 2. Parameter Bounds ──
  server.registerTool(
    "get_parameter_bounds",
    {
      title: "Get Tunable Parameter Bounds",
      description:
        "Returns all tunable parameters grouped by tier (GREEN/YELLOW/RED) with current allowed ranges, step sizes, and cooldown status.",
      inputSchema: {},
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async () => {
      const grouped = getParametersByTier();
      const result: Record<string, unknown[]> = {};

      for (const [tier, params] of Object.entries(grouped)) {
        result[tier] = params.map((p) => ({
          key: Object.entries(PARAMETER_BOUNDS).find(([, v]) => v === p)?.[0],
          name: p.name,
          type: p.type,
          min: p.min,
          max: p.max,
          step: p.step,
          default: p.defaultValue,
          description: p.description,
          inCooldown: isInCooldown(
            Object.entries(PARAMETER_BOUNDS).find(([, v]) => v === p)?.[0] || "",
            p.cooldownMs
          ),
          cooldownRemainingSeconds: getCooldownRemaining(
            Object.entries(PARAMETER_BOUNDS).find(([, v]) => v === p)?.[0] || "",
            p.cooldownMs
          ),
        }));
      }

      return toolSuccess(result);
    }
  );

  // ── 3. Apply GREEN Adjustment ──
  server.registerTool(
    "apply_green_adjustment",
    {
      title: "Apply GREEN Adjustment (Auto)",
      description:
        "Auto-apply a GREEN tier parameter adjustment. The change is applied immediately and the user is notified. Only works for GREEN tier parameters (RSI thresholds, signal interval, confidence). Validates bounds and cooldown.",
      inputSchema: {
        parameter: z.string().describe("Parameter key (e.g., 'rsi_oversold', 'signal_interval_minutes')"),
        new_value: z.number().describe("New value to set (must be within allowed bounds)"),
        reasoning: z.string().describe("Explanation of why this adjustment is being made"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ parameter, new_value, reasoning }: { parameter: string; new_value: number; reasoning: string }) => {
      const bound = PARAMETER_BOUNDS[parameter];
      if (!bound) return toolError(`Unknown parameter: ${parameter}`);
      if (bound.tier !== "GREEN") return toolError(`${parameter} is ${bound.tier} tier, not GREEN. Use request_yellow_adjustment or request_red_adjustment.`);

      // Check cooldown
      if (isInCooldown(parameter, bound.cooldownMs)) {
        const remaining = getCooldownRemaining(parameter, bound.cooldownMs);
        return toolError(`${parameter} is in cooldown. ${remaining}s remaining before next adjustment.`);
      }

      // Validate bounds
      const validation = validateAdjustment(parameter, new_value);
      if (!validation.valid) return toolError(validation.error!);

      const effectiveValue = validation.clampedValue ?? new_value;

      // Take snapshot before change
      const snapshot = await takeSnapshot();

      // Apply via BotCore API
      const body = parameter === "signal_interval_minutes"
        ? { interval_seconds: effectiveValue * 60 }
        : { [bound.apiField]: effectiveValue };

      const method = bound.apiEndpoint.includes("signal-interval") ? "PUT" : "PUT";
      const res = await apiRequest("rust", bound.apiEndpoint, {
        method,
        body,
        timeoutMs: 10_000,
      });

      if (!res.success) return toolError(res.error || "Failed to apply adjustment");

      // Log to audit
      const oldValue = snapshot.parameters[bound.apiField];
      logAdjustment({
        parameter,
        tier: "GREEN",
        oldValue,
        newValue: effectiveValue,
        reasoning,
        source: "auto",
        snapshotId: snapshot.id,
      });

      return toolSuccess({
        applied: true,
        parameter: bound.name,
        oldValue,
        newValue: effectiveValue,
        reasoning,
        notification: `[AUTO] ${bound.name} changed: ${oldValue} → ${effectiveValue}. Reason: ${reasoning}`,
        snapshotId: snapshot.id,
      });
    }
  );

  // ── 4. Request YELLOW Adjustment ──
  server.registerTool(
    "request_yellow_adjustment",
    {
      title: "Request YELLOW Adjustment (Needs Confirmation)",
      description:
        "Request a YELLOW tier parameter adjustment. Returns a confirmation token that the user must approve. Use for stop loss, take profit, position size, leverage changes.",
      inputSchema: {
        parameter: z.string().describe("Parameter key"),
        new_value: z.number().describe("Proposed new value"),
        reasoning: z.string().describe("Why this change is recommended"),
        confirm_token: z.string().optional().describe("Confirmation token from user (if confirming)"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ parameter, new_value, reasoning, confirm_token }: {
      parameter: string; new_value: number; reasoning: string; confirm_token?: string;
    }) => {
      const bound = PARAMETER_BOUNDS[parameter];
      if (!bound) return toolError(`Unknown parameter: ${parameter}`);
      if (bound.tier !== "YELLOW") return toolError(`${parameter} is ${bound.tier} tier, not YELLOW.`);

      // Validate bounds
      const validation = validateAdjustment(parameter, new_value);
      if (!validation.valid) return toolError(validation.error!);

      const effectiveValue = validation.clampedValue ?? new_value;
      const paramsHash = hashParams({ parameter, new_value: effectiveValue });

      // If confirmation token provided, validate and apply
      if (confirm_token) {
        if (!validateConfirmToken(confirm_token, `tune_${parameter}`, paramsHash)) {
          return toolError("Invalid or expired confirmation token.");
        }

        // Check cooldown
        if (isInCooldown(parameter, bound.cooldownMs)) {
          const remaining = getCooldownRemaining(parameter, bound.cooldownMs);
          return toolError(`${parameter} is in cooldown. ${remaining}s remaining.`);
        }

        const snapshot = await takeSnapshot();
        const res = await apiRequest("rust", bound.apiEndpoint, {
          method: "PUT",
          body: { [bound.apiField]: effectiveValue },
          timeoutMs: 10_000,
        });

        if (!res.success) return toolError(res.error || "Failed to apply adjustment");

        const oldValue = snapshot.parameters[bound.apiField];
        logAdjustment({
          parameter,
          tier: "YELLOW",
          oldValue,
          newValue: effectiveValue,
          reasoning,
          source: "confirmed",
          snapshotId: snapshot.id,
        });

        return toolSuccess({
          applied: true,
          parameter: bound.name,
          oldValue,
          newValue: effectiveValue,
          reasoning,
        });
      }

      // No token - return confirmation request
      const token = generateConfirmToken(`tune_${parameter}`, paramsHash);
      return toolSuccess({
        pending: true,
        message: `CONFIRM: Change ${bound.name} to ${effectiveValue}?\nReason: ${reasoning}\nCurrent bounds: [${bound.min} - ${bound.max}]`,
        confirm_token: token,
        parameter: bound.name,
        newValue: effectiveValue,
      });
    }
  );

  // ── 5. Request RED Adjustment ──
  server.registerTool(
    "request_red_adjustment",
    {
      title: "Request RED Adjustment (Needs Explicit Approval)",
      description:
        "Request a RED tier parameter adjustment. Returns a warning and requires the user to type explicit approval text. Use for max daily loss, engine start/stop.",
      inputSchema: {
        parameter: z.string().describe("Parameter key"),
        new_value: z.union([z.number(), z.boolean()]).describe("Proposed new value"),
        reasoning: z.string().describe("Why this change is recommended"),
        risk_assessment: z.string().describe("Assessment of risks involved in this change"),
        approval_text: z.string().optional().describe("User's explicit approval (e.g., 'APPROVE CHANGE MAX DAILY LOSS')"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ parameter, new_value, reasoning, risk_assessment, approval_text }: {
      parameter: string; new_value: number | boolean; reasoning: string;
      risk_assessment: string; approval_text?: string;
    }) => {
      const bound = PARAMETER_BOUNDS[parameter];
      if (!bound) return toolError(`Unknown parameter: ${parameter}`);
      if (bound.tier !== "RED") return toolError(`${parameter} is ${bound.tier} tier, not RED.`);

      const validation = validateAdjustment(parameter, new_value);
      if (!validation.valid) return toolError(validation.error!);

      const effectiveValue = validation.clampedValue ?? new_value;
      const expectedApproval = `APPROVE CHANGE ${bound.name.toUpperCase()}`;

      // If approval provided, validate and apply
      if (approval_text) {
        if (approval_text.toUpperCase() !== expectedApproval.toUpperCase()) {
          return toolError(
            `Invalid approval. Expected: "${expectedApproval}"\nGot: "${approval_text}"`
          );
        }

        if (isInCooldown(parameter, bound.cooldownMs)) {
          const remaining = getCooldownRemaining(parameter, bound.cooldownMs);
          return toolError(`${parameter} is in cooldown. ${remaining}s remaining.`);
        }

        const snapshot = await takeSnapshot();

        // Handle engine start/stop specially
        let res;
        if (parameter === "engine_running") {
          const endpoint = effectiveValue
            ? "/api/paper-trading/start"
            : "/api/paper-trading/stop";
          res = await apiRequest("rust", endpoint, { method: "POST", body: {}, timeoutMs: 10_000 });
        } else {
          res = await apiRequest("rust", bound.apiEndpoint, {
            method: "PUT",
            body: { [bound.apiField]: effectiveValue },
            timeoutMs: 10_000,
          });
        }

        if (!res.success) return toolError(res.error || "Failed to apply RED adjustment");

        const oldValue = snapshot.parameters[bound.apiField];
        logAdjustment({
          parameter,
          tier: "RED",
          oldValue,
          newValue: effectiveValue,
          reasoning: `${reasoning} | Risk: ${risk_assessment}`,
          source: "approved",
          approvedBy: "user",
          snapshotId: snapshot.id,
        });

        return toolSuccess({
          applied: true,
          parameter: bound.name,
          oldValue,
          newValue: effectiveValue,
          warning: "RED tier change applied. Monitor closely.",
        });
      }

      // No approval - return warning
      return toolSuccess({
        pending: true,
        warning: `⚠️ RED TIER CHANGE: ${bound.name}`,
        proposed_value: effectiveValue,
        reasoning,
        risk_assessment,
        required_approval: expectedApproval,
        message: `To approve, the user must type exactly: "${expectedApproval}"`,
      });
    }
  );

  // ── 6. Adjustment History ──
  server.registerTool(
    "get_adjustment_history",
    {
      title: "Get Adjustment History",
      description:
        "View past parameter adjustments from the audit trail. Filter by tier, parameter, or get the last N entries.",
      inputSchema: {
        limit: z.number().optional().describe("Max entries to return (default: 20)"),
        tier: z.enum(["GREEN", "YELLOW", "RED"]).optional().describe("Filter by tier"),
        parameter: z.string().optional().describe("Filter by parameter key"),
      },
      annotations: { readOnlyHint: true, openWorldHint: false },
    },
    async ({ limit, tier, parameter }: { limit?: number; tier?: "GREEN" | "YELLOW" | "RED"; parameter?: string }) => {
      const entries = getAuditHistory({ limit: limit || 20, tier, parameter });
      return toolSuccess({
        count: entries.length,
        entries,
      });
    }
  );

  // ── 7. Rollback ──
  server.registerTool(
    "rollback_adjustment",
    {
      title: "Rollback to Previous State",
      description:
        "Revert all parameters to a previous snapshot state. Use when a recent adjustment caused poor performance.",
      inputSchema: {
        snapshot_id: z.string().optional().describe("Specific snapshot ID to restore (default: most recent)"),
      },
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async ({ snapshot_id }: { snapshot_id?: string }) => {
      const snapList = getSnapshots(2);
      if (snapList.length < 2 && !snapshot_id) {
        return toolError("No previous snapshot available for rollback.");
      }

      const targetId = snapshot_id || snapList[1]?.id;
      if (!targetId) return toolError("No snapshot ID available.");

      // Take current snapshot before rollback
      const preRollback = await takeSnapshot();

      const result = await restoreFromSnapshot(targetId);
      if (!result.success) return toolError(result.error || "Rollback failed");

      logAdjustment({
        parameter: "_rollback",
        tier: "RED",
        oldValue: preRollback.id,
        newValue: targetId,
        reasoning: "Manual rollback to previous parameter state",
        source: "approved",
        snapshotId: preRollback.id,
      });

      return toolSuccess({
        rolled_back: true,
        restored_snapshot_id: targetId,
        pre_rollback_snapshot_id: preRollback.id,
        message: "Parameters restored to previous snapshot state.",
      });
    }
  );

  // ── 8. Take Snapshot ──
  server.registerTool(
    "take_parameter_snapshot",
    {
      title: "Take Parameter Snapshot",
      description:
        "Manually take a snapshot of current parameters and performance. Useful before making manual changes outside the tuning system.",
      inputSchema: {},
      annotations: { readOnlyHint: false, openWorldHint: false },
    },
    async () => {
      const snapshot = await takeSnapshot();
      return toolSuccess({
        snapshot_id: snapshot.id,
        timestamp: snapshot.timestamp,
        parameters: snapshot.parameters,
        performance: snapshot.performance,
      });
    }
  );

  log("info", "Self-tuning tools registered (8 tools)");
}
