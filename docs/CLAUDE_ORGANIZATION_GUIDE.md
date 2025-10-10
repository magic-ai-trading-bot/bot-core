# Claude Code - Project Organization Notes

## File Organization Rules

### Root Directory - ONLY Essential Files
Keep ONLY these files in root:
- ✅ README.md - Project overview (GitHub standard, must be in root)
- ✅ CLAUDE.md - Project instructions for Claude Code (must be in root)
- ✅ Makefile - Build automation
- ✅ docker-compose*.yml - Docker configuration
- ✅ config.env - Environment template

All other .md files go to docs/:
- 📁 docs/CONTRIBUTING.md - Contribution guidelines
- 📁 docs/SECURITY_CREDENTIALS.md - Security documentation

### Documentation Files - Always use docs/
- ✅ Reports → `docs/reports/`
- ✅ Certificates → `docs/certificates/`
- ✅ Quality metrics → `docs/`
- ✅ Testing docs → `docs/testing/`
- ✅ Architecture → `docs/architecture/`

### Service-Specific Docs
- ✅ Python AI Service → `python-ai-service/docs/`
- ✅ Rust Core Engine → `rust-core-engine/docs/`
- ✅ Frontend Dashboard → `nextjs-ui-dashboard/docs/`

### Temporary Files - Never Commit
- ❌ security_audit*.json
- ❌ *_VERSIONS*.txt
- ❌ requirements.*.updated.txt
- ❌ compare_versions.sh
- ❌ verify_versions.py

## Auto-Organization Checklist

When creating new documentation:
1. ✅ Determine if it's project-wide or service-specific
2. ✅ Put service docs in `{service}/docs/`
3. ✅ Put project docs in `docs/{category}/`
4. ✅ Never leave .md files in root (except essential 4)
5. ✅ Update .gitignore for temporary files
6. ✅ Use descriptive folder names (reports, testing, certificates)

## Commit Organization

Always commit in this order:
1. Create/update files in correct locations
2. Move misplaced files to correct locations
3. Remove temporary files
4. Update .gitignore
5. Single commit with clear organization message

---

**Last Updated:** 2025-10-10
**By:** Claude Code
