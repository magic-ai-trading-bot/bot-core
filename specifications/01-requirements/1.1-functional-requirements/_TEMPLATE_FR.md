# [Feature Name] - Functional Requirements

**Spec ID**: FR-[MODULE]-[NUMBER]  
**Version**: 1.0  
**Status**: ☐ Draft  
**Owner**: [Team]  
**Last Updated**: YYYY-MM-DD

## Quick Start

Use this template for all functional requirement specifications. Copy this file and replace bracketed placeholders with actual content.

## Template Sections

1. **Metadata** - Spec ID, version, status, ownership
2. **Tasks Checklist** - Track implementation progress
3. **Overview** - Brief description
4. **Requirements** - Detailed functional requirements with acceptance criteria
5. **Use Cases** - Actor-system interactions
6. **Data Requirements** - Input/output/validation
7. **Interface Requirements** - APIs, UI, external systems
8. **Testing Strategy** - How to test this feature
9. **Traceability** - Links to related specs
10. **Changelog** - Version history

## Example: FR-AUTH-001

See `_SPEC_TEMPLATE.md` in the specs root directory for the complete template structure.

## Code Tagging

When implementing, add tags to your code:

```rust
// @spec:FR-AUTH-001 - JWT token generation
// @ref:API_SPEC.md#authentication
fn generate_jwt_token(user_id: &str) -> Result<String> {
    // Implementation
}
```

## Checklist Before Marking as "Approved"

- [ ] All acceptance criteria are clear and testable
- [ ] Dependencies are identified
- [ ] Test cases are planned
- [ ] Design doc is referenced
- [ ] Non-functional requirements considered
- [ ] Peer review completed
- [ ] TRACEABILITY_MATRIX.md updated
