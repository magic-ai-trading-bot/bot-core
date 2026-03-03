# [Category] - Non-Functional Requirements

**Spec ID**: NFR-[CATEGORY]-[NUMBER]  
**Version**: 1.0  
**Status**: ☐ Draft  
**Owner**: [Team]  
**Last Updated**: YYYY-MM-DD

## Categories

- **Performance** (NFR-PERF-XXX)
- **Security** (NFR-SEC-XXX)
- **Scalability** (NFR-SCALE-XXX)
- **Reliability** (NFR-REL-XXX)
- **Maintainability** (NFR-MAINT-XXX)

## Template

Use `_SPEC_TEMPLATE.md` and focus on the "Non-Functional Requirements" section.

## Example Metrics

**Performance**:
- API response time: < 200ms (p95)
- Database query time: < 50ms (p95)
- Page load time: < 2s

**Security**:
- Authentication: JWT with 24h expiry
- Authorization: Role-based access control (RBAC)
- Encryption: TLS 1.3 for transit, AES-256 for storage

**Scalability**:
- Concurrent users: 10,000
- Horizontal scaling: Auto-scaling enabled
- Database: Read replicas for load distribution

**Reliability**:
- Uptime: 99.9% (8.76 hours downtime/year max)
- RTO: 15 minutes
- RPO: 5 minutes

**Maintainability**:
- Code coverage: > 80%
- Technical debt ratio: < 5%
- Documentation: Complete API docs, architecture diagrams
