// @spec:FR-MCP-007 - Security Tier Enforcement
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-mcp-tool-implementation.md

import { createHash, randomBytes } from "node:crypto";
import { SecurityTier, toolConfirm, toolError, log } from "./types.js";

const CONFIRM_SECRET = process.env.MCP_CONFIRM_SECRET || randomBytes(32).toString("hex");
const TOKEN_EXPIRY_MS = 5 * 60 * 1000; // 5 minutes

// Store used tokens to prevent replay
const usedTokens = new Set<string>();

// Clean up expired tracking data every 10 minutes
setInterval(() => {
  usedTokens.clear();
}, 10 * 60 * 1000);

/**
 * Generate a confirmation token for a tool call.
 */
export function generateConfirmToken(toolName: string, paramsHash: string): string {
  const timestamp = Date.now().toString();
  const data = `${toolName}:${paramsHash}:${timestamp}:${CONFIRM_SECRET}`;
  const token = createHash("sha256").update(data).digest("hex").slice(0, 32);
  return `${token}:${timestamp}`;
}

/**
 * Validate a confirmation token.
 */
export function validateConfirmToken(
  token: string,
  toolName: string,
  paramsHash: string
): boolean {
  if (usedTokens.has(token)) {
    return false; // Replay prevention
  }

  const parts = token.split(":");
  if (parts.length !== 2) return false;

  const [hash, timestamp] = parts;
  const ts = parseInt(timestamp, 10);
  if (isNaN(ts)) return false;

  // Check expiry
  if (Date.now() - ts > TOKEN_EXPIRY_MS) return false;

  // Verify hash
  const data = `${toolName}:${paramsHash}:${timestamp}:${CONFIRM_SECRET}`;
  const expected = createHash("sha256").update(data).digest("hex").slice(0, 32);
  if (hash !== expected) return false;

  // Mark as used
  usedTokens.add(token);
  return true;
}

/**
 * Hash parameters for confirmation token generation.
 */
export function hashParams(params: Record<string, unknown>): string {
  return createHash("sha256")
    .update(JSON.stringify(params))
    .digest("hex")
    .slice(0, 16);
}

/**
 * Middleware: check if a write tool needs confirmation.
 * Returns null if the tool can proceed, or a confirmation result if it needs approval.
 */
export function checkConfirmation(
  toolName: string,
  tier: SecurityTier,
  params: Record<string, unknown>,
  confirmToken?: string
): ReturnType<typeof toolConfirm> | ReturnType<typeof toolError> | null {
  // Tier 1 and read-only tools don't need confirmation
  if (tier <= SecurityTier.PUBLIC) return null;

  // If a confirm token is provided, validate it
  if (confirmToken) {
    const paramsHash = hashParams(params);
    if (validateConfirmToken(confirmToken, toolName, paramsHash)) {
      log("info", `Confirmation accepted for ${toolName}`, { tier });
      return null; // Proceed
    }
    return toolError(`Invalid or expired confirmation token for ${toolName}`);
  }

  // For SENSITIVE and CRITICAL, always require confirmation on write
  if (tier >= SecurityTier.SENSITIVE) {
    const paramsHash = hashParams(params);
    const token = generateConfirmToken(toolName, paramsHash);
    return toolConfirm(
      toolName,
      `This is a ${tier >= SecurityTier.CRITICAL ? "CRITICAL" : "SENSITIVE"} operation. Parameters: ${JSON.stringify(params)}`,
      token
    );
  }

  return null;
}
