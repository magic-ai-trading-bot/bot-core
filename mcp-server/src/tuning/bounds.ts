// @spec:FR-MCP-011 - Parameter Bounds Registry
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-03-self-tuning-engine.md

import type { ParameterBound, ValidationResult } from "./types.js";

const SIX_HOURS = 6 * 60 * 60 * 1000;
const ONE_HOUR = 60 * 60 * 1000;

// All tunable parameters with hard bounds
export const PARAMETER_BOUNDS: Record<string, ParameterBound> = {
  // ── GREEN: Auto-adjust + notify ──
  rsi_oversold: {
    name: "RSI Oversold Threshold",
    tier: "GREEN",
    min: 20, max: 40, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "rsi_oversold",
    description: "RSI level below which a symbol is considered oversold (buy signal)",
    defaultValue: 30,
    cooldownMs: SIX_HOURS,
  },
  rsi_overbought: {
    name: "RSI Overbought Threshold",
    tier: "GREEN",
    min: 60, max: 80, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "rsi_overbought",
    description: "RSI level above which a symbol is considered overbought (sell signal)",
    defaultValue: 70,
    cooldownMs: SIX_HOURS,
  },
  signal_interval_minutes: {
    name: "Signal Generation Interval",
    tier: "GREEN",
    min: 3, max: 30, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/signal-interval",
    apiField: "interval_seconds",
    description: "Minutes between signal generation cycles",
    defaultValue: 5,
    cooldownMs: ONE_HOUR,
  },
  confidence_threshold: {
    name: "Signal Confidence Threshold",
    tier: "GREEN",
    min: 0.50, max: 0.90, step: 0.05, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "confidence_threshold",
    description: "Minimum confidence score required to act on a trading signal",
    defaultValue: 0.65,
    cooldownMs: SIX_HOURS,
  },

  // ── YELLOW: Require confirmation ──
  stop_loss_percent: {
    name: "Stop Loss %",
    tier: "YELLOW",
    min: 0.5, max: 5.0, step: 0.5, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "stop_loss_percent",
    description: "Percentage below entry price to trigger stop loss",
    defaultValue: 2.0,
    cooldownMs: SIX_HOURS,
  },
  take_profit_percent: {
    name: "Take Profit %",
    tier: "YELLOW",
    min: 1.0, max: 10.0, step: 0.5, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "take_profit_percent",
    description: "Percentage above entry price to trigger take profit",
    defaultValue: 4.0,
    cooldownMs: SIX_HOURS,
  },
  position_size_percent: {
    name: "Position Size %",
    tier: "YELLOW",
    min: 1.0, max: 10.0, step: 0.5, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "position_size_percent",
    description: "Percentage of portfolio allocated per trade",
    defaultValue: 5.0,
    cooldownMs: SIX_HOURS,
  },
  max_positions: {
    name: "Max Concurrent Positions",
    tier: "YELLOW",
    min: 1, max: 8, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "max_positions",
    description: "Maximum number of simultaneous open positions",
    defaultValue: 4,
    cooldownMs: SIX_HOURS,
  },
  leverage: {
    name: "Leverage",
    tier: "YELLOW",
    min: 1, max: 20, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "leverage",
    description: "Trading leverage multiplier",
    defaultValue: 10,
    cooldownMs: SIX_HOURS,
  },

  // ── RED: Require explicit approval ──
  max_daily_loss_percent: {
    name: "Max Daily Loss %",
    tier: "RED",
    min: 3.0, max: 15.0, step: 1.0, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "max_daily_loss_percent",
    description: "Maximum daily portfolio loss before trading is paused",
    defaultValue: 10.0,
    cooldownMs: SIX_HOURS,
  },
  engine_running: {
    name: "Paper Trading Engine On/Off",
    tier: "RED",
    type: "boolean",
    apiEndpoint: "/api/paper-trading/start",
    apiField: "_action",
    description: "Start or stop the paper trading engine",
    defaultValue: false,
    cooldownMs: ONE_HOUR,
  },
};

/**
 * Validate a proposed parameter adjustment against hard bounds.
 */
export function validateAdjustment(
  paramKey: string,
  newValue: unknown
): ValidationResult {
  const bound = PARAMETER_BOUNDS[paramKey];
  if (!bound) {
    return { valid: false, error: `Unknown parameter: ${paramKey}` };
  }

  if (bound.type === "boolean") {
    if (typeof newValue !== "boolean") {
      return { valid: false, error: `${paramKey} must be a boolean` };
    }
    return { valid: true };
  }

  if (bound.type === "number") {
    const num = Number(newValue);
    if (isNaN(num)) {
      return { valid: false, error: `${paramKey} must be a number` };
    }
    if (bound.min !== undefined && num < bound.min) {
      return { valid: false, error: `${paramKey} must be >= ${bound.min} (got ${num})` };
    }
    if (bound.max !== undefined && num > bound.max) {
      return { valid: false, error: `${paramKey} must be <= ${bound.max} (got ${num})` };
    }
    if (bound.step !== undefined) {
      // Round to nearest step
      const rounded = Math.round(num / bound.step) * bound.step;
      const roundedFixed = parseFloat(rounded.toFixed(4));
      return { valid: true, clampedValue: roundedFixed };
    }
    return { valid: true };
  }

  if (bound.type === "enum") {
    if (!bound.enumValues?.includes(String(newValue))) {
      return { valid: false, error: `${paramKey} must be one of: ${bound.enumValues?.join(", ")}` };
    }
    return { valid: true };
  }

  return { valid: false, error: `Unsupported parameter type: ${bound.type}` };
}

/**
 * Get all parameter bounds grouped by tier.
 */
export function getParametersByTier(): Record<string, ParameterBound[]> {
  const grouped: Record<string, ParameterBound[]> = { GREEN: [], YELLOW: [], RED: [] };
  for (const bound of Object.values(PARAMETER_BOUNDS)) {
    grouped[bound.tier].push(bound);
  }
  return grouped;
}
