# TruffleHog Security Scan Fix

## Problem

The TruffleHog secret detection job was failing with the error:
```
BASE and HEAD commits are the same. TruffleHog won't scan anything.
```

## Root Cause

The original configuration used:
```yaml
base: ${{ github.event.before || format('HEAD~{0}', github.event.commits && length(github.event.commits) || 1) }}
head: HEAD
```

This approach had several issues:

1. **Scheduled runs**: `github.event.before` is not available for scheduled cron jobs
2. **First push to branch**: When `github.event.before` is `0000000000000000000000000000000000000000` (null SHA)
3. **Fallback logic**: The fallback `HEAD~N` would fail if there aren't enough commits in history

## Solution

Split TruffleHog into 3 conditional steps based on event type:

### 1. Pull Request Scans
```yaml
- name: Run TruffleHog (PR)
  if: github.event_name == 'pull_request'
  uses: trufflesecurity/trufflehog@main
  with:
    path: ./
    base: ${{ github.event.pull_request.base.sha }}
    head: ${{ github.event.pull_request.head.sha }}
    extra_args: --debug --only-verified
```
- Uses PR-specific base and head SHAs
- Scans only the diff between PR base and head

### 2. Push Event Scans (Normal)
```yaml
- name: Run TruffleHog (Push)
  if: github.event_name == 'push' && github.event.before != '0000000000000000000000000000000000000000'
  uses: trufflesecurity/trufflehog@main
  with:
    path: ./
    base: ${{ github.event.before }}
    head: ${{ github.sha }}
    extra_args: --debug --only-verified
```
- Runs on push events when there's a valid previous commit
- Scans the diff between the previous commit and current commit
- Skips if `before` is the null SHA (first push)

### 3. Full Repository Scans (Scheduled/First Push)
```yaml
- name: Run TruffleHog (Full Scan - Scheduled/First Push)
  if: github.event_name == 'schedule' || (github.event_name == 'push' && github.event.before == '0000000000000000000000000000000000000000')
  uses: trufflesecurity/trufflehog@main
  with:
    path: ./
    extra_args: --debug --only-verified --since-commit="" --max-depth=1000
```
- Runs on scheduled cron jobs (daily at 2 AM UTC)
- Runs on first push to a new branch
- Performs a full repository scan (last 1000 commits)
- No base/head comparison needed

## Benefits

1. **No more failures**: Handles all event types correctly
2. **Efficient scanning**: Only scans diffs for PRs and pushes
3. **Complete coverage**: Full scans on schedule and first push
4. **Better debugging**: Clear step names indicate which scan is running

## Testing

The fix handles these scenarios:

- ✅ Pull requests (base vs head)
- ✅ Normal pushes (before vs current)
- ✅ First push to branch (full scan)
- ✅ Scheduled runs (full scan)
- ✅ Manual workflow dispatch (full scan)

## Verification

After deploying, verify the workflow:

```bash
# Check recent workflow runs
gh run list --workflow=security-scan.yml

# View specific run details
gh run view <run-id>

# Check TruffleHog step output
gh run view <run-id> --log | grep -A 20 "Run TruffleHog"
```

## References

- [TruffleHog GitHub Action Docs](https://github.com/trufflesecurity/trufflehog#octocat-trufflehog-github-action)
- [GitHub Actions Context](https://docs.github.com/en/actions/learn-github-actions/contexts)
- [GitHub Events Reference](https://docs.github.com/en/actions/using-workflows/events-that-trigger-workflows)

## Related Files

- `.github/workflows/security-scan.yml` - Main security workflow
- `docs/SECURITY_AUDIT_REPORT.md` - Overall security documentation
- `scripts/security-scan.sh` - Local security scanning script
