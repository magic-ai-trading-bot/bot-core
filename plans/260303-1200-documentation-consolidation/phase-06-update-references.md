# Phase 6: Update References & Clean Up

**Priority**: High | **Status**: Pending | **Effort**: Medium

## Overview

Update all references across the project to point to new `specifications/` paths. Remove old directories.

## Reference Updates

### 1. CLAUDE.md (Root)

Update all paths:
- `specs/01-requirements/` → `specifications/01-requirements/`
- `specs/02-design/` → `specifications/02-design/`
- `specs/03-testing/` → `specifications/03-testing/`
- `specs/TRACEABILITY_MATRIX.md` → `specifications/TRACEABILITY_MATRIX.md`
- `specs/_SPEC_TEMPLATE.md` → `specifications/_SPEC_TEMPLATE.md`
- `docs/features/` → `specifications/06-features/`
- All `docs/*.md` references → new `specifications/` paths
- Update "Documentation Structure" section
- Update "Quick Feature Location Map" section

### 2. Code @spec Tags

Search and update:
```bash
grep -r "@ref:specs/" rust-core-engine/
grep -r "@ref:docs/" rust-core-engine/
```
Update all `@ref` paths to `specifications/` paths.

### 3. .claude/BOT_CORE_INSTRUCTIONS.md

Update any docs/ or specs/ references.

### 4. README.md (if exists at root)

Update documentation links.

### 5. Internal Spec Cross-References

Within specification files, update:
- Links between spec files that reference old paths
- TRACEABILITY_MATRIX.md path references

### 6. CI/CD & Scripts

Check for hardcoded paths:
```bash
grep -r "specs/" .github/
grep -r "docs/" .github/
grep -r "specs/" scripts/
grep -r "docs/" scripts/
```

## Clean Up

### Delete Old Directories

1. `rm -rf specs/` (after verifying all content migrated)
2. `rm -rf docs/` (after verifying all content migrated/archived)

### Verify No Broken References

```bash
# Should return 0 results after update
grep -r "specs/" --include="*.md" --include="*.rs" --include="*.py" --include="*.ts" --include="*.tsx" . | grep -v specifications/ | grep -v node_modules | grep -v target
grep -r "docs/" --include="*.md" --include="*.rs" . | grep -v specifications/ | grep -v node_modules | grep -v target | grep -v "// docs" | grep -v "# docs"
```

## Todo

- [ ] Update CLAUDE.md (all path references)
- [ ] Update code @spec/@ref tags in rust-core-engine/
- [ ] Update .claude/BOT_CORE_INSTRUCTIONS.md
- [ ] Update internal cross-references in spec files
- [ ] Check CI/CD and scripts for hardcoded paths
- [ ] Delete old specs/ directory
- [ ] Delete old docs/ directory
- [ ] Run broken reference check
- [ ] Final git commit

## Success Criteria

- Zero references to old `specs/` or `docs/` paths
- All internal links within specifications work
- CLAUDE.md accurately reflects new structure
- Clean git status after final commit
