# Phase 4: Create Missing Specs

**Priority**: High | **Status**: Pending | **Effort**: Medium

## Overview

Create formal specification documents for features that exist in code but lack specs.

## Missing Specs

### 1. FR-MCP.md (MCP Server)

**Source**: `mcp-server/` codebase, `CLAUDE.md` MCP section
**Location**: `specifications/01-requirements/1.1-functional-requirements/FR-MCP.md`

Requirements to document:
- MCP protocol v2024-11-05 compliance
- Streamable HTTP transport
- 103 tools across 12 categories
- Per-session server architecture
- Health check endpoint
- Authentication & session management
- Tool registration patterns (Zod schemas)
- Error handling & response formats

### 2. FR-OPENCLAW.md (OpenClaw/Telegram Gateway)

**Source**: `openclaw/` codebase, `CLAUDE.md` OpenClaw section
**Location**: `specifications/01-requirements/1.1-functional-requirements/FR-OPENCLAW.md`

Requirements to document:
- Telegram bot integration
- WebSocket gateway (port 18789)
- Bridge script (botcore-bridge.mjs) as MCP client
- Cron job scheduling
- Skill injection via SKILL.md
- Authentication (gateway token, session keys)
- Model configuration
- Production deployment (Docker)

### 3. FR-SELF-TUNING.md (Self-Tuning Engine)

**Source**: `mcp-server/src/tools/self-tuning-tools.ts`, `CLAUDE.md`
**Location**: `specifications/01-requirements/1.1-functional-requirements/FR-SELF-TUNING.md`

Requirements to document:
- 3-tier system (GREEN/YELLOW/RED)
- 11 tunable parameters
- Auto-adjustment for GREEN tier
- Confirmation workflow for YELLOW tier
- Approval workflow for RED tier
- Rollback capability
- Dashboard/monitoring

### 4. Feature docs for specifications/06-features/

Create/update feature guides:
- `mcp-server.md` — MCP Server feature guide
- `openclaw.md` — OpenClaw feature guide
- Review & update existing 5 feature docs against actual code

## Steps

1. Read relevant source code for each missing spec
2. Write FR-MCP.md following _SPEC_TEMPLATE.md format
3. Write FR-OPENCLAW.md following _SPEC_TEMPLATE.md format
4. Write FR-SELF-TUNING.md following _SPEC_TEMPLATE.md format
5. Create mcp-server.md and openclaw.md feature guides
6. Update TRACEABILITY_MATRIX.md with new entries

## Todo

- [ ] Create FR-MCP.md
- [ ] Create FR-OPENCLAW.md
- [ ] Create FR-SELF-TUNING.md
- [ ] Create mcp-server.md feature guide
- [ ] Create openclaw.md feature guide
- [ ] Update TRACEABILITY_MATRIX.md

## Success Criteria

- All features in code have corresponding specs
- New specs follow existing template format
- TRACEABILITY_MATRIX updated with new mappings
