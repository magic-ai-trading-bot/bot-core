// @spec:FR-MCP-010 - Self-Tuning Types
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-03-self-tuning-engine.md

export type TuningTier = "GREEN" | "YELLOW" | "RED";

export interface ParameterBound {
  name: string;
  tier: TuningTier;
  min?: number;
  max?: number;
  step?: number;
  type: "number" | "boolean" | "enum";
  enumValues?: string[];
  apiEndpoint: string;
  apiField: string;
  description: string;
  defaultValue: number | boolean | string;
  cooldownMs: number; // Minimum time between adjustments
}

export interface AuditEntry {
  id: string;
  timestamp: string;
  parameter: string;
  tier: TuningTier;
  oldValue: unknown;
  newValue: unknown;
  reasoning: string;
  source: "auto" | "confirmed" | "approved";
  approvedBy?: string;
  snapshotId: string;
}

export interface ParameterSnapshot {
  id: string;
  timestamp: string;
  parameters: Record<string, unknown>;
  performance?: {
    winRate?: number;
    totalPnl?: number;
    sharpeRatio?: number;
    maxDrawdown?: number;
  };
}

export interface ValidationResult {
  valid: boolean;
  error?: string;
  clampedValue?: number;
}
