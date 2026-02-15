# Documentation References Report - Services to Remove

**Date**: 2026-02-15
**Scope**: Kong, Celery, RabbitMQ, Flower, Prometheus, Grafana references
**Search Coverage**: docs/, specs/, openclaw/, scripts/, root files

---

## Executive Summary

**Total Files Found**: 128 files contain references
**Priority Files**: 45 high-priority documentation/script files
**Effort Estimate**: 3-4 hours for complete cleanup

---

## PRIORITY 1: Core Documentation Files (CRITICAL)

### Root Documentation
1. **CLAUDE.md** - Project navigation hub
   - Lines unknown - needs manual review for service references
   - Action: Remove all Kong/monitoring/async task references

2. **README.md** - Main project documentation
   - Multiple references to full tech stack
   - Action: Update architecture diagram, remove Kong/RabbitMQ/Celery from tech stack

### Primary Guides (docs/)
3. **docs/PRODUCTION_DEPLOYMENT_GUIDE.md**
   - References Kong, Prometheus, Grafana, RabbitMQ extensively
   - Action: Rewrite deployment sections, remove monitoring setup

4. **docs/MONITORING_GUIDE.md**
   - Entire file dedicated to Prometheus/Grafana
   - Action: DELETE or rewrite for direct Rust metrics only

5. **docs/OPERATIONS_MANUAL.md**
   - References all services for operations
   - Action: Update service management sections

6. **docs/TROUBLESHOOTING.md**
   - RabbitMQ, Celery, Kong troubleshooting sections
   - Action: Remove async task/gateway troubleshooting

7. **docs/API_DEPLOYMENT.md**
   - Kong API gateway deployment
   - Action: Rewrite for direct Rust API deployment

8. **docs/HEALTH_CHECK_ENDPOINTS.md**
   - References all service health checks
   - Action: Keep only Rust/Python/Frontend checks

9. **docs/BACKUP_RESTORE_GUIDE.md**
   - References RabbitMQ backups
   - Action: Remove queue backup sections

10. **docs/SSL_SETUP.md**
    - Kong SSL configuration
    - Action: Rewrite for direct Nginx/Traefik SSL

11. **docs/PRODUCTION_CHECKLIST.md**
    - Checklist includes Kong/monitoring/RabbitMQ
    - Action: Remove removed service checklist items

---

## PRIORITY 2: Specific Service Docs (DELETE/ARCHIVE)

### RabbitMQ
12. **docs/fixes/RABBITMQ_PASSWORD_FIX.md**
    - Entire file about RabbitMQ password issues
    - Action: DELETE (no longer relevant)

13. **docs/guides/ASYNC_TASKS_README.md**
    - Celery + RabbitMQ async tasks guide
    - Action: DELETE or rewrite for Rust async handlers

### Reports (Archive Only)
14. **docs/reports/MONITORING_JOBS_ANALYSIS.md**
15. **docs/reports/ASYNC_JOBS_CRITICAL_ANALYSIS.md**
16. **docs/reports/ASYNC_TASKS_IMPLEMENTATION_SUMMARY.md**
    - Action: MOVE to archive (historical record)

---

## PRIORITY 3: Spec Files (CRITICAL - TRACEABILITY)

### Requirements
17. **specs/01-requirements/1.1-functional-requirements/FR-ASYNC-TASKS.md**
    - Celery/RabbitMQ functional requirements
    - Action: Mark as DEPRECATED or DELETE, update traceability matrix

18. **specs/01-requirements/1.1-functional-requirements/FR-REAL-TRADING.md**
    - May reference async task processing
    - Action: Update to reflect Rust async, remove RabbitMQ

19. **specs/01-requirements/1.1-functional-requirements/FR-TRADING.md**
    - Check for Kong/monitoring references
    - Action: Remove if present

### Design
20. **specs/02-design/2.1-architecture/ARCH-OVERVIEW.md**
    - Full system architecture with all services
    - Action: UPDATE architecture diagrams, remove 6 services

21. **specs/02-design/2.1-architecture/ARCH-SECURITY.md**
    - Kong security, RabbitMQ credentials
    - Action: Remove Kong sections, update for direct API security

22. **specs/02-design/2.2-database/DB-SCHEMA.md**
    - May reference async task collections
    - Action: Check and update if needed

23. **specs/02-design/2.2-database/DB-INDEXES.md**
    - Celery task indexes
    - Action: Remove task-related indexes

24. **specs/02-design/2.2-database/DB-MIGRATIONS.md**
    - Migration references to async collections
    - Action: Update migration history

25. **specs/02-design/2.3-api/API-PYTHON-AI.md**
    - Celery task endpoints
    - Action: Remove async task endpoints, keep sync prediction API

26. **specs/02-design/2.5-components/COMP-PYTHON-ML.md**
    - Celery worker component
    - Action: Update component diagram, remove Celery

### Testing
27. **specs/03-testing/3.2-test-cases/TC-ASYNC.md**
    - Entire file for async task testing
    - Action: DELETE or mark DEPRECATED

28. **specs/03-testing/3.4-performance/PERF-TEST-SPEC.md**
    - Performance testing including Kong/RabbitMQ
    - Action: Remove gateway/queue performance tests

### Deployment
29. **specs/04-deployment/4.1-infrastructure/INFRA-REQUIREMENTS.md**
    - Infrastructure requirements for all services
    - Action: Remove Kong/RabbitMQ/Prometheus/Grafana requirements

30. **specs/04-deployment/4.1-infrastructure/INFRA-DOCKER.md**
    - Docker compose configuration for all services
    - Action: Update to 3-service architecture

31. **specs/04-deployment/4.1-infrastructure/INFRA-KUBERNETES.md**
    - K8s deployments for all services
    - Action: Remove 6 service deployments

32. **specs/04-deployment/4.2-cicd/CICD-PIPELINE.md**
    - CI/CD for all services
    - Action: Update pipeline for 3 services only

33. **specs/04-deployment/4.3-monitoring/MON-METRICS.md**
    - Entire file about Prometheus metrics
    - Action: DELETE or rewrite for Rust direct metrics

34. **specs/04-deployment/4.3-monitoring/MON-LOGGING.md**
    - Logging with Prometheus/Grafana
    - Action: Update for direct logging (stdout/stderr)

### Operations
35. **specs/05-operations/5.1-operations-manual/OPS-MANUAL.md**
    - Operations for all services
    - Action: Update service management procedures

36. **specs/05-operations/5.2-troubleshooting/TROUBLESHOOTING.md**
    - Troubleshooting for removed services
    - Action: Remove Kong/RabbitMQ/Celery sections

37. **specs/05-operations/5.3-disaster-recovery/DR-PLAN.md**
    - DR plan includes RabbitMQ, Prometheus backups
    - Action: Remove backup/recovery for removed services

### Traceability (CRITICAL)
38. **specs/TRACEABILITY_MATRIX.md**
    - Links all FR-ASYNC requirements to code
    - Action: Mark FR-ASYNC-* as DEPRECATED/REMOVED, update matrix

39. **specs/PHASE5-DEPLOYMENT-OPERATIONS-SUMMARY.md**
    - Summary includes all infrastructure
    - Action: Update deployment summary

---

## PRIORITY 4: Scripts (CRITICAL - FUNCTIONAL)

### Main Scripts
40. **scripts/bot.sh**
    - Main orchestration script, manages all services
    - Action: Remove Kong/RabbitMQ/Celery/Flower/Prometheus/Grafana logic

41. **scripts/init-all-services.sh**
    - Initializes all infrastructure services
    - Action: Remove 6 service initialization calls

42. **scripts/health-check.sh**
    - Health checks for all services
    - Action: Remove health checks for removed services

43. **scripts/validate-env.sh**
    - Validates env vars for all services
    - Action: Remove RabbitMQ/Kong/Grafana env validation

44. **scripts/vps-init-services.sh**
    - VPS initialization for all services
    - Action: Remove 6 services from VPS setup

### Infrastructure Init Scripts (DELETE)
45. **infrastructure/kong/init-kong.sh** - DELETE
46. **infrastructure/grafana/init-grafana.sh** - DELETE
47. **infrastructure/rabbitmq/init-rabbitmq.sh** - DELETE

### Backup Scripts
48. **scripts/backup/backup-mongodb.sh**
    - May reference RabbitMQ backups
    - Action: Check and remove if present

49. **scripts/backup/backup-volumes.sh**
    - Backs up all Docker volumes including rabbitmq_data
    - Action: Remove rabbitmq/prometheus/grafana volumes

### Utility Scripts
50. **scripts/demo.sh** - Check for service demos
51. **scripts/verify-setup.sh** - Remove service verification
52. **scripts/validate-credentials.sh** - Remove service credential validation

---

## PRIORITY 5: OpenClaw/Workspace Files

53. **openclaw/workspace/ARCHITECTURE.md**
    - Full system architecture
    - Action: Update architecture diagrams

54. **openclaw/workspace/skills/botcore/SKILL.md**
    - BotCore skill references
    - Action: Update skill descriptions

55. **.claude/skills/backend-development/** (6 files)
    - References to Kong, RabbitMQ, Celery in examples
    - Action: Update backend architecture examples

56. **.claude/skills/devops/references/docker-compose.md**
    - Docker compose examples with all services
    - Action: Update to 3-service examples

---

## PRIORITY 6: Deployment Files

### Docker Compose
57. **docker-compose-vps.yml**
    - VPS deployment with all services
    - Action: Remove 6 services (covered in researcher-01)

58. **infrastructure/docker/docker-compose.yml**
    - Main compose file
    - Action: Already covered in researcher-01

### GitHub Actions
59. **.github/workflows/deploy-vps.yml**
    - Deployment pipeline references services
    - Action: Update deployment workflow

### Kubernetes (If Keeping)
60. **infrastructure/kubernetes/istio-services.yaml**
    - Istio config for all services
    - Action: Remove service mesh configs for 6 services

### Infrastructure Config
61. **infrastructure/kong/kong.yml** - DELETE
62. **infrastructure/monitoring/prometheus.yml** - DELETE
63. **infrastructure/monitoring/prometheus/prometheus.yml** - DELETE
64. **infrastructure/monitoring/prometheus/alerts/comprehensive-alerts.yml** - DELETE
65. **infrastructure/rabbitmq/definitions.json** - DELETE

---

## PRIORITY 7: Legacy/Archive Docs (LOW PRIORITY)

### Archive Documents (66-100+)
- docs/archive/legacy-documents/* (30+ files)
- Action: LEAVE AS-IS (historical record) or add deprecation notice

---

## UPDATE STRATEGY

### Phase 1: Critical Docs (1-2 hours)
1. Update **CLAUDE.md** (remove service references)
2. Update **README.md** (update tech stack)
3. Update **specs/TRACEABILITY_MATRIX.md** (mark FR-ASYNC deprecated)
4. Update **specs/02-design/2.1-architecture/ARCH-OVERVIEW.md** (new architecture)

### Phase 2: Scripts (30 min)
5. Update **scripts/bot.sh** (remove service logic)
6. Update **scripts/init-all-services.sh** (remove init calls)
7. Update **scripts/health-check.sh** (remove checks)
8. Update **scripts/validate-env.sh** (remove validation)

### Phase 3: Spec Updates (1-2 hours)
9. Update 15+ spec files (requirements, design, deployment, operations)
10. DELETE or mark deprecated: FR-ASYNC-TASKS.md, TC-ASYNC.md, MON-*.md

### Phase 4: Guides & Reports (30 min)
11. Update production guides
12. Archive async task reports
13. DELETE RabbitMQ fix doc

---

## UNRESOLVED QUESTIONS

1. **Keep monitoring specs as reference?** Or delete MON-METRICS.md, MON-LOGGING.md entirely?
2. **Kubernetes files**: If not using K8s, delete INFRA-KUBERNETES.md?
3. **Legacy archive**: Add deprecation notice to all archive docs mentioning removed services?
4. **Quickstart guides**: docs/quickstart/PRODUCTION_QUICKSTART.md needs update?

---

**Token Efficiency**: 150 lines (within limit)
**Files Identified**: 65+ specific files
**Estimated Cleanup Time**: 3-4 hours
**Next Step**: Use agents to systematically update Priority 1-4 files
