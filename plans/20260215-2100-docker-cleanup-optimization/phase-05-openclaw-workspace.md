# Phase 5: OpenClaw Workspace Updates

**Parent Plan**: [plan.md](plan.md)
**Dependencies**: Phase 4 (Core Documentation)
**Date**: 2026-02-15
**Effort**: 20 minutes
**Priority**: MEDIUM
**Status**: 🔲 Pending

---

## Overview

Update OpenClaw workspace documentation and skill references to remove Kong, RabbitMQ, Celery, Prometheus, Grafana.

**Files to Update**: ARCHITECTURE.md, FEATURES.md (auto-generated), backend skills (6 files), docker-compose skill

---

## Key Insights

- OpenClaw workspace has auto-generated docs from sync script
- SOUL.md and STRATEGIES.md are auto-generated (don't edit manually)
- ARCHITECTURE.md may be auto-generated - check sync script first
- .claude/skills/ contains backend development examples with removed services
- If sync script regenerates docs, update the sync script itself

---

## Requirements

### Must Update
1. **openclaw/workspace/ARCHITECTURE.md** - System architecture
2. **.claude/skills/backend-development/** - 6 skill files with service examples
3. **.claude/skills/devops/references/docker-compose.md** - Docker compose examples

### Must Check (Auto-Generated)
1. **Sync script**: Check if it auto-generates ARCHITECTURE.md, FEATURES.md
2. If yes, update sync script to exclude removed services

---

## Related Code Files

```
/Users/dungngo97/Documents/bot-core/openclaw/workspace/ARCHITECTURE.md
/Users/dungngo97/Documents/bot-core/openclaw/workspace/FEATURES.md
/Users/dungngo97/Documents/bot-core/.claude/skills/backend-development/
/Users/dungngo97/Documents/bot-core/.claude/skills/devops/references/docker-compose.md
/Users/dungngo97/Documents/bot-core/openclaw/scripts/sync-*.sh (if exists)
```

---

## Implementation Steps

### Step 1: Check for Sync Script

```bash
# Find sync scripts
find openclaw/scripts -name "sync*.sh" 2>/dev/null

# If found, read to check what it auto-generates
# If ARCHITECTURE.md or FEATURES.md auto-generated, update script first
```

**If sync script exists**:
- Update script to exclude removed services
- Run script to regenerate docs
- Skip manual updates to auto-generated files

**If no sync script**:
- Manually update all files

### Step 2: Update openclaw/workspace/ARCHITECTURE.md

**File**: `/Users/dungngo97/Documents/bot-core/openclaw/workspace/ARCHITECTURE.md`

**Changes**:
1. Update system architecture diagram
2. Remove Kong API Gateway section
3. Remove RabbitMQ/Celery section
4. Remove Prometheus/Grafana section
5. Update service count (20 → 6-7)
6. Update component interactions

**Example**:
```markdown
# OLD
## Services
- Kong API Gateway (8100, 8001)
- RabbitMQ (5672, 15672)
- Celery Worker
- Prometheus (9090)
- Grafana (3001)

# NEW
## Services
- Rust Core Engine (8080)
- Python AI Service (8000)
- Next.js Frontend (3000)
- MongoDB (27017)
- MCP Server (8090)
- OpenClaw (Telegram/WhatsApp)
- Redis (6379) - optional
```

### Step 3: Update Backend Development Skills

**Directory**: `/Users/dungngo97/Documents/bot-core/.claude/skills/backend-development/`

**Files to Update** (6 files):
- API integration examples
- Microservices examples
- Message queue examples
- Monitoring examples

**Changes**:
1. Remove Kong API gateway examples
2. Remove RabbitMQ/Celery async task examples
3. Remove Prometheus metrics examples
4. Update to direct API calls (Rust → Python)
5. Update architecture examples

**Example**:
```markdown
# OLD (in skill examples)
- Use Kong for API routing
- Use RabbitMQ for async tasks
- Add Prometheus metrics

# NEW
- Use direct HTTP calls (Rust ↔ Python)
- Use Rust async for background tasks
- Use built-in logging
```

### Step 4: Update DevOps Docker Compose Skill

**File**: `/Users/dungngo97/Documents/bot-core/.claude/skills/devops/references/docker-compose.md`

**Changes**:
1. Remove Kong, RabbitMQ, Celery, Prometheus, Grafana service examples
2. Update service count examples
3. Update profile examples (remove api-gateway, messaging, monitoring)
4. Update docker-compose commands

**Example**:
```yaml
# OLD
services:
  kong:
    image: kong:3.8
    profiles: [api-gateway]

  rabbitmq:
    image: rabbitmq:3.12
    profiles: [messaging]

# NEW
services:
  mongodb:
    image: mongo:7.0

  rust-core-engine:
    build: ./rust-core-engine
    profiles: [prod]
```

### Step 5: Verify OpenClaw Workspace

```bash
# Verify no removed service references
grep -ri "kong\|rabbitmq\|celery\|prometheus\|grafana" openclaw/workspace/
# Expected: ZERO matches (or only historical context)

# Verify skills updated
grep -ri "kong\|rabbitmq\|celery\|prometheus\|grafana" .claude/skills/
# Expected: ZERO matches (or only as deprecated examples)
```

---

## Todo List

### Sync Script Check
- [ ] Find sync scripts in openclaw/scripts/
- [ ] Check if ARCHITECTURE.md auto-generated
- [ ] Check if FEATURES.md auto-generated
- [ ] If yes, update sync script first
- [ ] If yes, run sync script to regenerate

### Manual Updates (if no sync script)
- [ ] Update openclaw/workspace/ARCHITECTURE.md
- [ ] Update openclaw/workspace/FEATURES.md (if exists)
- [ ] Update backend skill: API integration
- [ ] Update backend skill: Microservices
- [ ] Update backend skill: Message queue (remove entirely)
- [ ] Update backend skill: Monitoring (remove Prometheus)
- [ ] Update backend skill: Error handling
- [ ] Update backend skill: Testing
- [ ] Update devops skill: docker-compose.md

### Verification
- [ ] grep openclaw/workspace/ for removed services
- [ ] grep .claude/skills/ for removed services
- [ ] Verify ARCHITECTURE.md shows 6-7 core services
- [ ] Verify no Kong/RabbitMQ/Celery examples in skills

---

## Success Criteria

- ✅ Sync script updated (if exists)
- ✅ openclaw/workspace/ARCHITECTURE.md updated
- ✅ 6 backend skill files updated
- ✅ docker-compose skill updated
- ✅ Zero references to removed services in workspace
- ✅ Zero removed service examples in skills

---

## Risk Assessment

**Risk Level**: 🟢 LOW

**Risks**:
- Breaking OpenClaw documentation
- Outdated skill examples

**Mitigation**:
- Check for sync script first
- Update sync script if auto-generated
- Test OpenClaw functionality after updates

---

## Output Artifacts

- Updated openclaw/workspace/ARCHITECTURE.md
- Updated backend skills (6 files)
- Updated docker-compose skill
- Sync script updated (if exists)
- Verification report
