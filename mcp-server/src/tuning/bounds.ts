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

  data_resolution: {
    name: "Data Resolution / Timeframe",
    tier: "GREEN",
    type: "enum",
    enumValues: ["1m", "3m", "5m", "15m", "30m", "1h", "4h", "1d"],
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "data_resolution",
    description: "Timeframe for trading signal analysis and kline data (e.g., 15m, 1h, 4h)",
    defaultValue: "15m",
    cooldownMs: ONE_HOUR,
  },

  // ── GREEN: Previously YELLOW, promoted to auto-adjust ──
  stop_loss_percent: {
    name: "Stop Loss % (PnL-based)",
    tier: "GREEN",
    min: 1.0, max: 20.0, step: 0.5, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "stop_loss_percent",
    description: "PnL percentage to trigger stop loss. Actual price move = this / leverage. Always check current leverage before adjusting.",
    defaultValue: 10.0,
    cooldownMs: SIX_HOURS,
  },
  take_profit_percent: {
    name: "Take Profit % (PnL-based)",
    tier: "GREEN",
    min: 2.0, max: 40.0, step: 1.0, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "take_profit_percent",
    description: "PnL percentage to trigger take profit. Actual price move = this / leverage. Always check current leverage before adjusting.",
    defaultValue: 20.0,
    cooldownMs: SIX_HOURS,
  },
  // ── YELLOW: Require confirmation (capital risk params) ──
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
  min_required_indicators: {
    name: "Min Required Indicators",
    tier: "GREEN",
    min: 2, max: 5, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "min_required_indicators",
    description: "Minimum indicators that must agree per timeframe before trading (MACD, RSI, Bollinger, Stochastic, Volume). 2=aggressive, 4=balanced, 5=conservative",
    defaultValue: 4,
    cooldownMs: SIX_HOURS,
  },
  min_required_timeframes: {
    name: "Min Required Timeframes",
    tier: "GREEN",
    min: 1, max: 4, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/basic-settings",
    apiField: "min_required_timeframes",
    description: "Minimum timeframes that must agree before trading (5M, 15M, 1H, 4H). 1=aggressive, 3=balanced, 4=conservative",
    defaultValue: 3,
    cooldownMs: SIX_HOURS,
  },

  // ── GREEN: Signal pipeline parameters (auto-adjust + notify) ──
  // @spec:FR-SETTINGS-003 - Signal pipeline tunable parameters
  sp_min_weighted_threshold: {
    name: "Min Weighted Threshold",
    tier: "GREEN",
    min: 30, max: 70, step: 5, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.min_weighted_threshold",
    description: "Minimum weighted agreement % to generate directional signal. Lower = more signals, higher = fewer but higher quality",
    defaultValue: 60,
    cooldownMs: SIX_HOURS,
  },
  sp_rsi_bull_threshold: {
    name: "RSI Bull Threshold",
    tier: "GREEN",
    min: 50, max: 65, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.rsi_bull_threshold",
    description: "RSI value above which indicator counts as bullish. Lower = more bullish signals",
    defaultValue: 55,
    cooldownMs: SIX_HOURS,
  },
  sp_rsi_bear_threshold: {
    name: "RSI Bear Threshold",
    tier: "GREEN",
    min: 35, max: 50, step: 1, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.rsi_bear_threshold",
    description: "RSI value below which indicator counts as bearish. Higher = more bearish signals",
    defaultValue: 45,
    cooldownMs: SIX_HOURS,
  },
  sp_volume_confirm_multiplier: {
    name: "Volume Confirm Multiplier",
    tier: "GREEN",
    min: 1.0, max: 2.0, step: 0.1, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.volume_confirm_multiplier",
    description: "Volume ratio above which volume confirms trend direction. Lower = easier confirmation",
    defaultValue: 1.2,
    cooldownMs: SIX_HOURS,
  },
  sp_confidence_max: {
    name: "Confidence Max Cap",
    tier: "GREEN",
    min: 0.70, max: 0.95, step: 0.05, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.confidence_max",
    description: "Maximum confidence cap for directional signals",
    defaultValue: 0.85,
    cooldownMs: SIX_HOURS,
  },
  sp_neutral_confidence: {
    name: "Neutral Signal Confidence",
    tier: "GREEN",
    min: 0.30, max: 0.50, step: 0.05, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.neutral_confidence",
    description: "Confidence value assigned to neutral signals",
    defaultValue: 0.40,
    cooldownMs: SIX_HOURS,
  },
  sp_counter_trend_block_offset: {
    name: "Counter-Trend Block Offset",
    tier: "GREEN",
    min: 0.0, max: 0.15, step: 0.01, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.counter_trend_block_offset",
    description: "Confidence offset added to neutral_confidence for counter-trend blocked signals",
    defaultValue: 0.05,
    cooldownMs: ONE_HOUR,
  },
  sp_counter_trend_mode: {
    name: "Counter-Trend Mode",
    tier: "GREEN",
    type: "enum",
    enumValues: ["block", "reduce"],
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.counter_trend_mode",
    description: "How to handle counter-trend signals: 'block' forces Neutral, 'reduce' allows with lower confidence",
    defaultValue: "block",
    cooldownMs: ONE_HOUR,
  },

  // ── YELLOW: Signal pipeline parameters (require confirmation) ──
  sp_bb_bull_threshold: {
    name: "BB Bull Threshold",
    tier: "YELLOW",
    min: 0.1, max: 0.4, step: 0.05, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.bb_bull_threshold",
    description: "BB position below which indicator counts as bullish (oversold). Lower = fewer bullish signals",
    defaultValue: 0.3,
    cooldownMs: SIX_HOURS,
  },
  sp_bb_bear_threshold: {
    name: "BB Bear Threshold",
    tier: "YELLOW",
    min: 0.6, max: 0.9, step: 0.05, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.bb_bear_threshold",
    description: "BB position above which indicator counts as bearish (overbought). Higher = fewer bearish signals",
    defaultValue: 0.7,
    cooldownMs: SIX_HOURS,
  },
  sp_stoch_overbought: {
    name: "Stochastic Overbought",
    tier: "YELLOW",
    min: 70, max: 90, step: 5, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.stoch_overbought",
    description: "Stochastic K level above which bullish signal is suppressed",
    defaultValue: 80,
    cooldownMs: SIX_HOURS,
  },
  sp_stoch_oversold: {
    name: "Stochastic Oversold",
    tier: "YELLOW",
    min: 10, max: 30, step: 5, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.stoch_oversold",
    description: "Stochastic K level below which bearish signal is suppressed",
    defaultValue: 20,
    cooldownMs: SIX_HOURS,
  },
  sp_weight_15m: {
    name: "15M Timeframe Weight",
    tier: "YELLOW",
    min: 0.0, max: 3.0, step: 0.5, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.weight_15m",
    description: "Weight for 15-minute timeframe in voting. 0 = ignore, higher = more important",
    defaultValue: 0.5,
    cooldownMs: SIX_HOURS,
  },
  sp_weight_30m: {
    name: "30M Timeframe Weight",
    tier: "YELLOW",
    min: 0.0, max: 3.0, step: 0.5, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.weight_30m",
    description: "Weight for 30-minute timeframe in voting. 0 = ignore, higher = more important",
    defaultValue: 1.0,
    cooldownMs: SIX_HOURS,
  },
  sp_weight_1h: {
    name: "1H Timeframe Weight",
    tier: "YELLOW",
    min: 0.0, max: 3.0, step: 0.5, type: "number",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.weight_1h",
    description: "Weight for 1-hour timeframe in voting. This is the main trend - highest weight recommended",
    defaultValue: 2.0,
    cooldownMs: SIX_HOURS,
  },
  sp_counter_trend_enabled: {
    name: "Counter-Trend Protection",
    tier: "YELLOW",
    type: "boolean",
    apiEndpoint: "/api/paper-trading/indicator-settings",
    apiField: "signal_pipeline.counter_trend_enabled",
    description: "Enable/disable counter-trend protection. When enabled, prevents trading against the 1H trend",
    defaultValue: true,
    cooldownMs: ONE_HOUR,
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
