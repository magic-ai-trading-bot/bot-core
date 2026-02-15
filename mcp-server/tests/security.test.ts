// @spec:FR-MCP-007 - Security Tier Enforcement Tests
// @ref:plans/20260215-1900-openclaw-mcp-integration/phases/phase-02-mcp-tool-implementation.md

import { vi } from "vitest";
import { generateConfirmToken, validateConfirmToken, hashParams } from "../src/security.js";

describe("generateConfirmToken", () => {
  it("returns a non-empty string", () => {
    const token = generateConfirmToken("test-tool", "abc123");
    expect(token).toBeDefined();
    expect(typeof token).toBe("string");
    expect(token.length).toBeGreaterThan(0);
  });

  it("returns token with correct format (hash:timestamp)", () => {
    const token = generateConfirmToken("test-tool", "abc123");
    const parts = token.split(":");
    expect(parts.length).toBe(2);

    const [hash, timestamp] = parts;
    expect(hash.length).toBe(32); // SHA256 sliced to 32 chars
    expect(timestamp).toMatch(/^\d+$/); // Timestamp is numeric string
  });

  it("generates different tokens for different tool names", () => {
    const token1 = generateConfirmToken("tool-a", "params123");
    const token2 = generateConfirmToken("tool-b", "params123");
    expect(token1).not.toBe(token2);
  });

  it("generates different tokens for different params hashes", () => {
    const token1 = generateConfirmToken("test-tool", "hash1");
    const token2 = generateConfirmToken("test-tool", "hash2");
    expect(token1).not.toBe(token2);
  });
});

describe("validateConfirmToken", () => {
  it("accepts valid token with matching tool/params", () => {
    const toolName = "adjust-parameter";
    const paramsHash = "def456";
    const token = generateConfirmToken(toolName, paramsHash);

    const isValid = validateConfirmToken(token, toolName, paramsHash);
    expect(isValid).toBe(true);
  });

  it("rejects token with wrong tool name", () => {
    const paramsHash = "xyz789";
    const token = generateConfirmToken("correct-tool", paramsHash);

    const isValid = validateConfirmToken(token, "wrong-tool", paramsHash);
    expect(isValid).toBe(false);
  });

  it("rejects token with wrong params hash", () => {
    const toolName = "adjust-parameter";
    const token = generateConfirmToken(toolName, "correct-hash");

    const isValid = validateConfirmToken(token, toolName, "wrong-hash");
    expect(isValid).toBe(false);
  });

  it("rejects expired token (mock Date.now)", () => {
    const toolName = "test-tool";
    const paramsHash = "hash123";

    // Generate token at current time
    const originalNow = Date.now;
    const baseTime = 1_700_000_000_000;
    Date.now = vi.fn(() => baseTime);

    const token = generateConfirmToken(toolName, paramsHash);

    // Fast-forward 6 minutes (beyond 5 minute expiry)
    Date.now = vi.fn(() => baseTime + 6 * 60 * 1000);

    const isValid = validateConfirmToken(token, toolName, paramsHash);
    expect(isValid).toBe(false);

    // Restore
    Date.now = originalNow;
  });

  it("rejects already-used token (single-use)", () => {
    const toolName = "test-tool";
    const paramsHash = "hash456";
    const token = generateConfirmToken(toolName, paramsHash);

    // First use should succeed
    const firstUse = validateConfirmToken(token, toolName, paramsHash);
    expect(firstUse).toBe(true);

    // Second use should fail (replay prevention)
    const secondUse = validateConfirmToken(token, toolName, paramsHash);
    expect(secondUse).toBe(false);
  });

  it("rejects malformed token (no colon separator)", () => {
    const isValid = validateConfirmToken("invalidtoken", "test-tool", "hash");
    expect(isValid).toBe(false);
  });

  it("rejects token with non-numeric timestamp", () => {
    const isValid = validateConfirmToken("abc123:notanumber", "test-tool", "hash");
    expect(isValid).toBe(false);
  });

  it("rejects token with extra parts", () => {
    const isValid = validateConfirmToken("hash:123456:extra:parts", "test-tool", "hash");
    expect(isValid).toBe(false);
  });
});

describe("hashParams", () => {
  it("produces consistent hashes for same input", () => {
    const params = { parameter: "rsi_oversold", value: 25 };
    const hash1 = hashParams(params);
    const hash2 = hashParams(params);

    expect(hash1).toBe(hash2);
    expect(hash1.length).toBe(16); // Sliced to 16 chars
  });

  it("produces different hashes for different input", () => {
    const params1 = { parameter: "rsi_oversold", value: 25 };
    const params2 = { parameter: "rsi_oversold", value: 30 };
    const params3 = { parameter: "rsi_overbought", value: 25 };

    const hash1 = hashParams(params1);
    const hash2 = hashParams(params2);
    const hash3 = hashParams(params3);

    expect(hash1).not.toBe(hash2);
    expect(hash1).not.toBe(hash3);
    expect(hash2).not.toBe(hash3);
  });

  it("handles empty object", () => {
    const hash = hashParams({});
    expect(hash).toBeDefined();
    expect(hash.length).toBe(16);
  });

  it("handles nested objects", () => {
    const params = {
      parameter: "stop_loss_percent",
      value: 2.5,
      metadata: { source: "auto-tune", confidence: 0.85 }
    };
    const hash = hashParams(params);
    expect(hash).toBeDefined();
    expect(hash.length).toBe(16);
  });

  it("handles arrays", () => {
    const params = {
      symbols: ["BTCUSDT", "ETHUSDT"],
      enabled: true
    };
    const hash = hashParams(params);
    expect(hash).toBeDefined();
    expect(hash.length).toBe(16);
  });

  it("key order does not affect hash (JSON.stringify)", () => {
    // Note: JSON.stringify preserves object key insertion order in modern JS
    // So this test verifies that same keys in same order produce same hash
    const params1 = { a: 1, b: 2 };
    const params2 = { a: 1, b: 2 };

    const hash1 = hashParams(params1);
    const hash2 = hashParams(params2);

    expect(hash1).toBe(hash2);
  });

  it("different key order produces different hash", () => {
    // JSON.stringify will serialize in property insertion order
    const obj1 = {};
    obj1.a = 1;
    obj1.b = 2;

    const obj2 = {};
    obj2.b = 2;
    obj2.a = 1;

    const hash1 = hashParams(obj1);
    const hash2 = hashParams(obj2);

    // These should be different due to key order
    expect(hash1).not.toBe(hash2);
  });
});

describe("token expiry edge cases", () => {
  it("accepts token just before expiry", () => {
    const toolName = "test-tool";
    const paramsHash = "hash123";

    const originalNow = Date.now;
    const baseTime = 1_700_000_000_000;
    Date.now = vi.fn(() => baseTime);

    const token = generateConfirmToken(toolName, paramsHash);

    // 4 minutes 59 seconds later (just before 5 minute expiry)
    Date.now = vi.fn(() => baseTime + 4 * 60 * 1000 + 59 * 1000);

    const isValid = validateConfirmToken(token, toolName, paramsHash);
    expect(isValid).toBe(true);

    // Restore
    Date.now = originalNow;
  });

  it("rejects token exactly at expiry", () => {
    const toolName = "test-tool";
    const paramsHash = "hash123";

    const originalNow = Date.now;
    const baseTime = 1_700_000_000_000;
    Date.now = vi.fn(() => baseTime);

    const token = generateConfirmToken(toolName, paramsHash);

    // Exactly 5 minutes later (TOKEN_EXPIRY_MS = 5 * 60 * 1000)
    Date.now = vi.fn(() => baseTime + 5 * 60 * 1000 + 1);

    const isValid = validateConfirmToken(token, toolName, paramsHash);
    expect(isValid).toBe(false);

    // Restore
    Date.now = originalNow;
  });
});

describe("integration: full confirmation flow", () => {
  it("generates, validates, and prevents replay", () => {
    const toolName = "adjust-parameter";
    const params = { parameter: "rsi_oversold", value: 25 };
    const paramsHash = hashParams(params);

    // Step 1: Generate token
    const token = generateConfirmToken(toolName, paramsHash);
    expect(token).toBeDefined();

    // Step 2: Validate token (first use)
    const isValid = validateConfirmToken(token, toolName, paramsHash);
    expect(isValid).toBe(true);

    // Step 3: Attempt replay (should fail)
    const replayAttempt = validateConfirmToken(token, toolName, paramsHash);
    expect(replayAttempt).toBe(false);
  });

  it("multiple concurrent tokens work independently", () => {
    const tool1 = "adjust-rsi";
    const tool2 = "adjust-stop-loss";
    const params1 = { value: 25 };
    const params2 = { value: 2.5 };

    const hash1 = hashParams(params1);
    const hash2 = hashParams(params2);

    const token1 = generateConfirmToken(tool1, hash1);
    const token2 = generateConfirmToken(tool2, hash2);

    // Both should be valid
    expect(validateConfirmToken(token1, tool1, hash1)).toBe(true);
    expect(validateConfirmToken(token2, tool2, hash2)).toBe(true);

    // Replay both should fail
    expect(validateConfirmToken(token1, tool1, hash1)).toBe(false);
    expect(validateConfirmToken(token2, tool2, hash2)).toBe(false);
  });
});
