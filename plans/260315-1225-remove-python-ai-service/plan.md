---
title: "Remove Python AI Service - Strategy-Only Architecture"
description: "Remove Python AI service and Grok API dependency. Rust runs purely on built-in strategies and config."
status: pending
priority: P0
effort: 8h
tags: [architecture, simplification, python-removal, performance]
created: 2026-03-15
---

# Remove Python AI Service - Strategy-Only Architecture

## Problem

- Python AI service consumes 1.5-2GB RAM but adds minimal value
- Grok/xAI API dependency = external cost + latency + single point of failure
- Rust already has complete strategy engine (RSI, MACD, Bollinger, Volume, Indicators)
- Trading should run on deterministic strategies + config, not external AI predictions

## Solution

Remove Python AI service entirely. Remove Grok API calls. Rust engine runs purely on:
- Built-in strategies (RSI, MACD, Bollinger, Volume)
- Strategy engine with configurable parameters
- YAML-based settings (already implemented)
- Signal pipeline with weighted timeframe analysis

## Impact

| Metric | Before | After |
|--------|--------|-------|
| Services | 7 | 6 |
| RAM usage | ~6GB | ~4GB |
| Languages | Rust + Python + TS | Rust + TS |
| External API deps | Grok/xAI | None |
| Test suites | 3 | 2 |
| CI/CD time | ~15min | ~10min |

## Phases

| Phase | Name | Status | Link |
|-------|------|--------|------|
| 1 | Rust: Remove AIService dependency | Pending | [phase-01](./phase-01-rust-remove-ai-service.md) |
| 2 | Docker & Infrastructure cleanup | Pending | [phase-02](./phase-02-docker-cleanup.md) |
| 3 | Frontend: Remove AI endpoints | Pending | [phase-03](./phase-03-frontend-cleanup.md) |
| 4 | MCP Server: Remove AI tools | Pending | [phase-04](./phase-04-mcp-cleanup.md) |
| 5 | CI/CD & Docs update | Pending | [phase-05](./phase-05-cicd-docs.md) |
| 6 | Delete python-ai-service/ & verify | Pending | [phase-06](./phase-06-delete-verify.md) |

## Key Decision

**Strategy engine is the source of trading signals.** No AI, no ML, no external API.
The signal pipeline (`signal_pipeline` config in YAML) already handles:
- Multi-timeframe analysis (15m, 30m, 1h)
- RSI bull/bear thresholds
- Bollinger band positioning
- Stochastic overbought/oversold
- Volume confirmation
- Counter-trend blocking
- Weighted confidence scoring

## Risk Assessment

| Risk | Impact | Mitigation |
|------|--------|------------|
| Lose AI signal quality | Low — Grok signals weren't significantly better than strategy ensemble | Strategy engine already has 65% win rate |
| Breaking existing trades | Medium | Do during market downtime, verify with paper trading first |
| Frontend pages empty | Low | Remove AI-specific pages, keep strategy signals display |

## Success Criteria

- [ ] Rust engine starts and trades without Python/Grok dependency
- [ ] All existing strategy-based tests pass
- [ ] Docker compose starts with 6 services (no Python)
- [ ] VPS RAM usage < 4GB
- [ ] CI/CD pipelines pass without Python steps
- [ ] Frontend shows strategy signals (not AI signals)
