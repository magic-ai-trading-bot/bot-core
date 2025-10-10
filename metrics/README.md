# Quality Metrics

This directory contains quality metrics reports and historical data for the bot-core project.

## Quick Start

### Run Quality Analysis

```bash
# From project root
make quality-metrics

# Or directly
./scripts/quality-metrics.sh
```

### View Reports

**Latest Summary:**
```bash
cat docs/reports/QUALITY_METRICS_SUMMARY.md
```

**Latest JSON Report:**
```bash
cat metrics/quality-report-*.json | jq
```

**View Historical Trends:**
```bash
tail -10 metrics/quality-history.jsonl | jq '.overall_score'
```

## Current Scores

**Overall Quality Score: 94/100 (Grade A)**

| Category | Score | Grade | Status |
|----------|-------|-------|--------|
| Code Quality | 96/100 | A+ | ⭐ Excellent |
| Security Score | 98/100 | A+ | ⭐ Excellent |
| Test Quality | 89/100 | B+ | Good |
| Documentation | 96/100 | A+ | ⭐ Excellent |
| Performance | 95/100 | A+ | ⭐ Excellent |

## Files

- `quality-report-YYYYMMDD_HHMMSS.json` - Detailed metrics report (timestamped)
- `quality-history.jsonl` - Historical metrics data (one JSON object per line)
- `README.md` - This file

## Report Format

Each JSON report contains:
- `timestamp` - Report generation time (UTC)
- `version` - Report format version
- `overall_score` - Overall quality score (0-100)
- `category_scores` - Scores for each category
- `detailed_metrics` - Granular metric breakdown
- `deployment_readiness` - Deployment readiness percentage

## Documentation

For detailed information about the quality metrics system:

- **Quality Metrics Guide:** `/docs/QUALITY_METRICS.md`
- **Latest Summary Report:** `/docs/reports/QUALITY_METRICS_SUMMARY.md`
- **Testing Guide:** `/docs/TESTING_GUIDE.md`

## Tracking Trends

The quality metrics are tracked over time in `quality-history.jsonl`. Each run appends a new entry.

### Example: View Last 7 Days

```bash
tail -7 quality-history.jsonl | jq '{date: .timestamp, score: .overall_score}'
```

### Example: Calculate Average Score

```bash
cat quality-history.jsonl | jq '.overall_score' | awk '{sum+=$1; n++} END {print sum/n}'
```

### Example: Plot Trends (requires gnuplot)

```bash
cat quality-history.jsonl | jq -r '[.timestamp, .overall_score] | @csv' > /tmp/metrics.csv
gnuplot -e "set terminal png; set output 'quality-trend.png'; plot '/tmp/metrics.csv' using 2 with lines"
```

## CI/CD Integration

Add to your CI pipeline to enforce quality standards:

```yaml
name: Quality Gate
on: [push, pull_request]

jobs:
  quality:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Run Quality Metrics
        run: ./scripts/quality-metrics.sh
      - name: Check Quality Threshold
        run: |
          score=$(jq '.overall_score' metrics/quality-report-*.json | head -1)
          if [ $score -lt 90 ]; then
            echo "Quality score $score below threshold 90"
            exit 1
          fi
```

## Quality Gates

Recommended minimum scores for different environments:

| Environment | Min Score | Rationale |
|-------------|-----------|-----------|
| Development | 70+ | Allow experimentation |
| Staging | 85+ | Ensure stability |
| Production | 90+ | World-class quality |

**Current Status:** ✅ **94/100 - Production Ready**

## Maintenance

### Cleanup Old Reports

```bash
# Keep only last 30 reports
ls -t metrics/quality-report-*.json | tail -n +31 | xargs rm -f
```

### Archive Historical Data

```bash
# Archive data older than 90 days
grep -v "$(date -d '90 days ago' +%Y-%m)" quality-history.jsonl > quality-history-archive.jsonl
```

## Support

For questions or issues with quality metrics:
1. Check `/docs/QUALITY_METRICS.md`
2. Review the latest summary report
3. Open an issue on GitHub

---

**Last Updated:** 2025-10-10
**Current Version:** 1.0.0
