// @spec:FR-MCP-011 - Parameter Bounds Registry Tests
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-03-self-tuning-engine.md

import { validateAdjustment, getParametersByTier, PARAMETER_BOUNDS } from "../src/tuning/bounds.js";

describe("validateAdjustment", () => {
  it("rejects unknown parameters", () => {
    const result = validateAdjustment("unknown_param", 42);
    expect(result.valid).toBe(false);
    expect(result.error).toBe("Unknown parameter: unknown_param");
  });

  it("accepts valid GREEN parameter values", () => {
    const result = validateAdjustment("rsi_oversold", 30);
    expect(result.valid).toBe(true);
    expect(result.error).toBeUndefined();
  });

  it("rejects values below min bound", () => {
    const result = validateAdjustment("rsi_oversold", 15);
    expect(result.valid).toBe(false);
    expect(result.error).toContain("must be >= 20");
    expect(result.error).toContain("got 15");
  });

  it("rejects values above max bound", () => {
    const result = validateAdjustment("rsi_overbought", 85);
    expect(result.valid).toBe(false);
    expect(result.error).toContain("must be <= 80");
    expect(result.error).toContain("got 85");
  });

  it("rounds to nearest step", () => {
    // confidence_threshold has step: 0.05, min: 0.50, max: 0.90
    const result = validateAdjustment("confidence_threshold", 0.73);
    expect(result.valid).toBe(true);
    expect(result.clampedValue).toBe(0.75); // Rounds 0.73 to nearest 0.05 step
  });

  it("handles boolean parameters (engine_running)", () => {
    const resultTrue = validateAdjustment("engine_running", true);
    expect(resultTrue.valid).toBe(true);

    const resultFalse = validateAdjustment("engine_running", false);
    expect(resultFalse.valid).toBe(true);
  });

  it("rejects non-number for number params", () => {
    const result = validateAdjustment("rsi_oversold", "not a number");
    expect(result.valid).toBe(false);
    expect(result.error).toContain("must be a number");
  });

  it("rejects non-boolean for boolean params", () => {
    const result = validateAdjustment("engine_running", "yes");
    expect(result.valid).toBe(false);
    expect(result.error).toContain("must be a boolean");
  });

  describe("fuzz test: out-of-bounds values", () => {
    const testValues = [
      -1, 0, 999, NaN, Infinity, -Infinity,
      null, undefined, "string", [], {}
    ];

    testValues.forEach((value) => {
      it(`handles ${JSON.stringify(value)} for rsi_oversold`, () => {
        const result = validateAdjustment("rsi_oversold", value);

        // All invalid values should be rejected
        if (typeof value === "string") {
          expect(result.valid).toBe(false);
          expect(result.error).toContain("must be a number");
        } else if (value === null || value === undefined || typeof value === "object") {
          // Number(null) = 0, Number([]) = 0, Number({}) = NaN
          // These will be checked after Number() conversion
          expect(result.valid).toBe(false);
          // May fail bounds check rather than type check
        } else if (Number.isNaN(value)) {
          expect(result.valid).toBe(false);
          expect(result.error).toContain("must be a number");
        } else if (!isFinite(value as number)) {
          // Infinity and -Infinity are technically numbers but fail bounds
          expect(result.valid).toBe(false);
        } else if ((value as number) < 20) {
          expect(result.valid).toBe(false);
          expect(result.error).toContain("must be >= 20");
        } else if ((value as number) > 40) {
          expect(result.valid).toBe(false);
          expect(result.error).toContain("must be <= 40");
        }
      });

      it(`handles ${JSON.stringify(value)} for engine_running (boolean)`, () => {
        const result = validateAdjustment("engine_running", value);

        // Only true/false are valid for boolean params
        if (typeof value === "boolean") {
          expect(result.valid).toBe(true);
        } else {
          expect(result.valid).toBe(false);
          expect(result.error).toContain("must be a boolean");
        }
      });
    });
  });
});

describe("getParametersByTier", () => {
  it("returns correct grouping", () => {
    const grouped = getParametersByTier();

    // Verify structure
    expect(grouped).toHaveProperty("GREEN");
    expect(grouped).toHaveProperty("YELLOW");
    expect(grouped).toHaveProperty("RED");

    // Verify all are arrays
    expect(Array.isArray(grouped.GREEN)).toBe(true);
    expect(Array.isArray(grouped.YELLOW)).toBe(true);
    expect(Array.isArray(grouped.RED)).toBe(true);

    // Verify counts (from bounds.ts)
    // GREEN: all tunable params except RED (12 total)
    expect(grouped.GREEN.length).toBe(12);

    // YELLOW: none (all promoted to GREEN for autonomous tuning)
    expect(grouped.YELLOW.length).toBe(0);

    // RED: max_daily_loss_percent, engine_running (2)
    expect(grouped.RED.length).toBe(2);

    // Verify specific parameters are in correct tiers
    const greenNames = grouped.GREEN.map(p => p.name);
    expect(greenNames).toContain("RSI Oversold Threshold");
    expect(greenNames).toContain("Signal Confidence Threshold");
    expect(greenNames).toContain("Stop Loss %");
    expect(greenNames).toContain("Leverage");

    const redNames = grouped.RED.map(p => p.name);
    expect(redNames).toContain("Max Daily Loss %");
    expect(redNames).toContain("Paper Trading Engine On/Off");
  });

  it("all parameters have required fields", () => {
    const grouped = getParametersByTier();
    const allParams = [...grouped.GREEN, ...grouped.YELLOW, ...grouped.RED];

    allParams.forEach(param => {
      expect(param).toHaveProperty("name");
      expect(param).toHaveProperty("tier");
      expect(param).toHaveProperty("type");
      expect(param).toHaveProperty("apiEndpoint");
      expect(param).toHaveProperty("apiField");
      expect(param).toHaveProperty("description");
      expect(param).toHaveProperty("defaultValue");
      expect(param).toHaveProperty("cooldownMs");

      // Type-specific validations
      if (param.type === "number") {
        expect(param).toHaveProperty("min");
        expect(param).toHaveProperty("max");
        expect(param).toHaveProperty("step");
      }
    });
  });
});

describe("PARAMETER_BOUNDS integrity", () => {
  it("all bounds have consistent min/max/step for number types", () => {
    Object.entries(PARAMETER_BOUNDS).forEach(([key, bound]) => {
      if (bound.type === "number") {
        expect(bound.min).toBeDefined();
        expect(bound.max).toBeDefined();
        expect(bound.step).toBeDefined();

        // Min should be less than max
        expect(bound.min!).toBeLessThan(bound.max!);

        // Default value should be within range
        if (typeof bound.defaultValue === "number") {
          expect(bound.defaultValue).toBeGreaterThanOrEqual(bound.min!);
          expect(bound.defaultValue).toBeLessThanOrEqual(bound.max!);
        }
      }
    });
  });

  it("all cooldown values are reasonable", () => {
    Object.values(PARAMETER_BOUNDS).forEach(bound => {
      // Cooldown should be at least 1 minute
      expect(bound.cooldownMs).toBeGreaterThanOrEqual(60_000);

      // Cooldown should not exceed 24 hours
      expect(bound.cooldownMs).toBeLessThanOrEqual(24 * 60 * 60 * 1000);
    });
  });

  it("step values divide range evenly", () => {
    Object.entries(PARAMETER_BOUNDS).forEach(([key, bound]) => {
      if (bound.type === "number" && bound.step !== undefined) {
        const range = bound.max! - bound.min!;
        const steps = range / bound.step;

        // Range should be divisible by step (with small floating point tolerance)
        expect(Math.abs(steps - Math.round(steps))).toBeLessThan(0.01);
      }
    });
  });
});
