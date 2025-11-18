# GPT-4 API Cost Optimization Guide

## 📊 Overview

This document describes the cost optimizations implemented for GPT-4o-mini API usage in the Bot-Core AI service.

**Optimization Date:** 2024-11-19
**Status:** ✅ IMPLEMENTED
**Estimated Savings:** 63% reduction in monthly costs

---

## 💰 Cost Comparison

### Before Optimization

| Metric | Value |
|--------|-------|
| Cache Duration | 5 minutes |
| Analysis Interval | 5 minutes |
| Max Tokens | 2,000 |
| Prompt Size | ~1,200 tokens |
| Estimated Monthly Cost | **$96.90** (~2,228,700 VNĐ) |

### After Optimization

| Metric | Value |
|--------|-------|
| Cache Duration | **15 minutes** ⬆️ |
| Analysis Interval | **10 minutes** ⬆️ |
| Max Tokens | **1,200** ⬇️ |
| Prompt Size | **~300 tokens** ⬇️ |
| Estimated Monthly Cost | **$36.00** (~828,000 VNĐ) |

**💵 Monthly Savings: $60.90 (1,400,700 VNĐ) - 63% reduction**

---

## 🔧 Optimizations Implemented

### 1. Cache Duration Extension

**File:** `config.yaml`
**Change:** Increased from 5 → 15 minutes

```yaml
ai_cache:
  duration_minutes: 15  # Optimized from 5
```

**Impact:** Reduces redundant API calls by 66%

### 2. Analysis Interval Optimization

**File:** `main.py:65`
**Change:** Increased from 5 → 10 minutes

```python
ANALYSIS_INTERVAL_MINUTES = 10  # Optimized from 5
```

**Impact:** Reduces periodic analysis frequency by 50%

### 3. Prompt Token Reduction

**File:** `main.py:1350-1371`
**Changes:**
- Compressed system prompt (400 → 100 tokens)
- Optimized market context (800 → 150 tokens)
- Simplified analysis prompt (400 → 50 tokens)

**Total Reduction:** ~1,200 → ~300 tokens (75% reduction)

**Example Before:**
```
MARKET DATA ANALYSIS:
Symbol: BTCUSDT
Current Price: $50,000.00
24h Volume: 1,234,567
...extensive formatting...
```

**Example After:**
```
BTCUSDT $50000
1H: RSI:65.2 MACD:0.45 BB:0.75 Vol:1.2x
```

### 4. Max Tokens Reduction

**File:** `main.py:1186, 945`
**Change:** Reduced from 2,000 → 1,200 tokens

```python
max_tokens=1200  # Reduced from 2000
```

**Impact:** Reduces output cost by 40%

### 5. Cost Monitoring System

**File:** `main.py:63-69, 1201-1223, 1952-2000`

**Features:**
- Real-time token tracking
- Per-request cost calculation
- Session statistics
- Daily/monthly projections
- Cost breakdown by input/output tokens

**New Endpoint:** `GET /ai/cost/statistics`

---

## 📈 Monitoring Cost Usage

### Check Real-Time Statistics

```bash
# Via API
curl http://localhost:8000/ai/cost/statistics

# Via browser
open http://localhost:8000/ai/cost/statistics
```

### Example Response

```json
{
  "session_statistics": {
    "total_requests": 150,
    "total_input_tokens": 45000,
    "total_output_tokens": 120000,
    "total_tokens": 165000,
    "total_cost_usd": 0.0792,
    "total_cost_vnd": 1822,
    "average_cost_per_request_usd": 0.00053,
    "average_tokens_per_request": 1100
  },
  "projections": {
    "estimated_daily_cost_usd": 1.2,
    "estimated_daily_cost_vnd": 27600,
    "estimated_monthly_cost_usd": 36.0,
    "estimated_monthly_cost_vnd": 828000
  },
  "configuration": {
    "model": "gpt-4o-mini",
    "analysis_interval_minutes": 10,
    "symbols_tracked": 8,
    "cache_duration_minutes": 15,
    "max_tokens": 1200
  },
  "optimization_status": {
    "cache_optimized": true,
    "interval_optimized": true,
    "prompt_optimized": true,
    "max_tokens_optimized": true,
    "estimated_savings_percent": 63
  }
}
```

### Monitor Logs

Cost information is logged with each GPT-4 API call:

```
💰 Cost: $0.00053 | Tokens: 280 in + 820 out = 1100 | Total today: $0.079 (150 requests)
```

---

## 🎯 Recommended Monitoring

### Daily Checks

1. **Morning Check:** Review overnight cost accumulation
   ```bash
   curl http://localhost:8000/ai/cost/statistics | jq '.session_statistics.total_cost_usd'
   ```

2. **Check Logs:** Look for cost spikes
   ```bash
   docker logs python-ai-service | grep "💰 Cost"
   ```

### Weekly Review

1. Calculate actual vs. projected costs
2. Adjust `ANALYSIS_INTERVAL_MINUTES` if needed
3. Review cache hit rate in MongoDB

### Monthly Audit

1. Compare with OpenAI billing dashboard
2. Analyze cost trends
3. Consider further optimizations if needed

---

## ⚙️ Fine-Tuning Options

### If Cost is Still Too High

#### Option 1: Increase Cache Duration
```yaml
# config.yaml
ai_cache:
  duration_minutes: 20  # Increase from 15
```

#### Option 2: Reduce Symbol Count
```python
# main.py
ANALYSIS_SYMBOLS = [
    "BTCUSDT",
    "ETHUSDT",
    # Remove less important symbols
]
```

#### Option 3: Increase Analysis Interval
```python
# main.py
ANALYSIS_INTERVAL_MINUTES = 15  # Increase from 10
```

### If Performance is Degraded

#### Option 1: Reduce Cache Duration
```yaml
ai_cache:
  duration_minutes: 10  # Decrease from 15
```

#### Option 2: Increase Max Tokens
```python
# main.py (in chat_completions_create)
max_tokens=1500  # Increase from 1200
```

---

## 🚨 Cost Alerts

### Set Up Alerts

Monitor the `/ai/cost/statistics` endpoint and alert if:

1. **Daily cost exceeds $2.00**
   ```bash
   DAILY_COST=$(curl -s http://localhost:8000/ai/cost/statistics | jq '.projections.estimated_daily_cost_usd')
   if (( $(echo "$DAILY_COST > 2.0" | bc -l) )); then
     echo "⚠️ ALERT: Daily cost exceeded $2.00!"
   fi
   ```

2. **Token usage spike** (>2000 tokens per request)
3. **Request rate spike** (>200 requests/hour)

---

## 📊 ROI Analysis

### Cost-Benefit of GPT-4 Analysis

**Monthly Cost:** $36 (~828k VNĐ)

**Value Provided:**
- 8 symbols × 144 analyses/day = **1,152 AI signals/day**
- 34,560 AI signals/month
- **Cost per signal:** $0.001 (23 VNĐ)

**Comparison with Alternatives:**
- Manual analysis: Impossible to scale
- Traditional indicators only: Limited accuracy
- GPT-4: Comprehensive multi-factor analysis at scale

**Conclusion:** Excellent ROI for production trading system

---

## 🔍 Troubleshooting

### High Cost Issues

**Problem:** Monthly projection shows >$50

**Solutions:**
1. Check for cache misses: `GET /ai/storage/stats`
2. Verify analysis interval: Should be 10 minutes
3. Check for manual request spikes in logs
4. Increase cache duration to 20 minutes

### Quality Degradation

**Problem:** Signals seem less accurate after optimization

**Solutions:**
1. Review GPT-4 responses in logs
2. Consider increasing max_tokens to 1500
3. Add more context to prompts (increase token budget slightly)
4. A/B test with longer prompts on sample data

### Rate Limiting

**Problem:** Getting 429 errors from OpenAI

**Solutions:**
1. Verify `OPENAI_REQUEST_DELAY = 20` seconds
2. Add backup API keys via `OPENAI_BACKUP_API_KEYS`
3. Increase delay between symbols (currently 10s)

---

## 📝 Version History

| Date | Version | Changes |
|------|---------|---------|
| 2024-11-19 | 1.0 | Initial optimization implementation |
|  |  | - Cache: 5→15min |
|  |  | - Interval: 5→10min |
|  |  | - Tokens: 2000→1200 |
|  |  | - Prompt optimization |
|  |  | - Cost monitoring |

---

## 🔗 Related Documentation

- [Python AI Service README](../README.md)
- [API Documentation](../../docs/API_DEPLOYMENT.md)
- [OpenAI Pricing](https://openai.com/pricing)
- [Cost Analysis Report](../../docs/reports/GPT4_COST_ANALYSIS.md)

---

## 📧 Support

For questions about cost optimization:
1. Check logs: `docker logs python-ai-service`
2. Review metrics: `GET /ai/cost/statistics`
3. Consult main documentation: `docs/TROUBLESHOOTING.md`

**Last Updated:** 2024-11-19
