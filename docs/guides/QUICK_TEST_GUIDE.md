# Quick Testing Guide - GPT-4 Cost Optimization

**Date:** 2024-11-19
**Purpose:** Verify cost optimizations work correctly

---

## ⚡ Quick Start Test (5 minutes)

### 1. Start Services
```bash
cd /Users/dungngo97/Documents/bot-core
./scripts/bot.sh stop
./scripts/bot.sh start --memory-optimized
```

### 2. Wait for Startup (30 seconds)
```bash
# Watch logs
docker logs -f python-ai-service
# Wait for: "✅ OpenAI GPT-4 client ready for analysis"
# Press Ctrl+C when ready
```

### 3. Check Health
```bash
curl http://localhost:8000/health | jq
```

**Expected:**
```json
{
  "status": "healthy",
  "gpt4_available": true,
  "analysis_interval_minutes": 10,  // ← Should be 10 (was 5)
  ...
}
```

### 4. View Cost Statistics (Initial)
```bash
curl http://localhost:8000/ai/cost/statistics | jq
```

**Expected:**
```json
{
  "session_statistics": {
    "total_requests": 0,
    "total_cost_usd": 0
  },
  "configuration": {
    "analysis_interval_minutes": 10,    // ← Optimized
    "cache_duration_minutes": 15,       // ← Optimized
    "max_tokens": 1200                  // ← Optimized
  },
  "optimization_status": {
    "cache_optimized": true,
    "interval_optimized": true,
    "prompt_optimized": true,
    "estimated_savings_percent": 63
  }
}
```

### 5. Wait for First Analysis (10 minutes max)
```bash
# Watch for cost logs
docker logs -f python-ai-service | grep "💰 Cost"
```

**Expected log:**
```
💰 Cost: $0.00053 | Tokens: 280 in + 820 out = 1100 | Total today: $0.00053 (1 requests)
```

### 6. Check Cost After First Request
```bash
curl http://localhost:8000/ai/cost/statistics | jq '.session_statistics'
```

**Expected:**
```json
{
  "total_requests": 1,
  "total_input_tokens": 280,      // ← Should be ~200-400 (was ~1200)
  "total_output_tokens": 820,     // ← Should be ~600-1000 (was ~1500-2000)
  "total_cost_usd": 0.0005,       // ← Should be ~$0.0005 (was ~$0.0014)
  "average_tokens_per_request": 1100
}
```

✅ **SUCCESS:** If input tokens < 500 and cost < $0.001, optimization working!

---

## 🔍 Detailed Testing (15 minutes)

### Test 1: Verify Optimized Prompts

```bash
# Trigger manual analysis
curl -X POST http://localhost:8000/ai/analyze \
  -H "Content-Type: application/json" \
  -d '{
    "symbol": "BTCUSDT",
    "timeframe_data": {
      "1h": [
        {"timestamp": 1700000000000, "open": 50000, "high": 50100, "low": 49900, "close": 50050, "volume": 1000}
      ]
    },
    "current_price": 50000,
    "volume_24h": 1000000,
    "timestamp": 1700000000000,
    "strategy_context": {
      "selected_strategies": ["RSI Strategy"],
      "market_condition": "Trending",
      "risk_level": "Moderate"
    }
  }'

# Watch logs for compact prompt
docker logs python-ai-service | tail -50 | grep "BTCUSDT"
```

**Expected in logs:**
```
BTCUSDT $50000
1H: RSI:50.0 MACD:0.00 BB:0.50 Vol:1.0x
```

### Test 2: Verify Cache (15-minute duration)

```bash
# First request (should hit GPT-4)
TIME1=$(date +%s)
curl -X POST http://localhost:8000/ai/analyze [...same payload...]
echo "First request at: $TIME1"

# Wait 5 seconds
sleep 5

# Second request (should use cache)
TIME2=$(date +%s)
curl -X POST http://localhost:8000/ai/analyze [...same payload...]
echo "Second request at: $TIME2"

# Check logs
docker logs python-ai-service | grep "Using recent MongoDB analysis"
```

**Expected:**
```
📊 Using recent MongoDB analysis for BTCUSDT (age: 0.1min)
```

**Verify:** No new "💰 Cost" log = cache hit ✅

### Test 3: Verify 10-Minute Interval

```bash
# Check when next analysis will run
docker logs python-ai-service | grep "Started periodic analysis"
```

**Expected:**
```
🔄 Started periodic analysis task (every 10 minutes)
```

**Wait 10 minutes and check:**
```bash
docker logs -f python-ai-service | grep "Completed AI analysis cycle"
```

**Expected (every 10 minutes):**
```
🎯 Completed AI analysis cycle for 8 symbols
```

### Test 4: Cost Accumulation

```bash
# Wait for 3-4 periodic cycles (30-40 minutes)
# Then check total cost

curl http://localhost:8000/ai/cost/statistics | jq '.session_statistics.total_cost_usd'
```

**Expected (after 30 minutes, ~24 requests):**
```
0.0126  // ~$0.013 for 24 requests
        // Old: would be ~$0.033
        // Savings: ~60%+ ✅
```

---

## 📊 Monitoring Dashboard

### Create Real-Time Monitor Script

```bash
# Save as monitor-costs.sh
cat > monitor-costs.sh << 'SCRIPT'
#!/bin/bash
while true; do
  clear
  echo "=== GPT-4 Cost Monitor ==="
  echo "Time: $(date)"
  echo ""

  STATS=$(curl -s http://localhost:8000/ai/cost/statistics)

  echo "📊 Session Statistics:"
  echo "$STATS" | jq '.session_statistics'

  echo ""
  echo "💰 Projections:"
  echo "$STATS" | jq '.projections'

  echo ""
  echo "⚙️ Configuration:"
  echo "$STATS" | jq '.configuration'

  echo ""
  echo "Press Ctrl+C to exit"
  sleep 10
done
SCRIPT

chmod +x monitor-costs.sh
./monitor-costs.sh
```

---

## ✅ Success Criteria Checklist

After testing, verify:

- [ ] Service starts without errors
- [ ] Health endpoint shows optimized config
- [ ] First GPT-4 request logs cost
- [ ] Input tokens < 500 per request
- [ ] Cost per request < $0.001
- [ ] Cache works (second request uses cached result)
- [ ] Analysis runs every 10 minutes (not 5)
- [ ] `/ai/cost/statistics` endpoint works
- [ ] Cost projections show ~$36/month
- [ ] Signal quality remains good (confidence > 0.5)

---

## 🚨 Troubleshooting

### Problem: Service won't start

**Check:**
```bash
docker logs python-ai-service
```

**Common issues:**
- Syntax error in main.py (run: `python3 -m py_compile main.py`)
- YAML error in config.yaml (run: `cat config.yaml | head -50`)

**Solution:**
```bash
# Rebuild
docker-compose up -d --build python-ai-service
```

### Problem: No cost logs appearing

**Check if GPT-4 is being called:**
```bash
docker logs python-ai-service | grep "Calling GPT-4"
```

**If no API calls:**
- Check `OPENAI_API_KEY` in `.env`
- Verify: `curl http://localhost:8000/debug/gpt4`

**Solution:**
```bash
# Check API key
grep OPENAI_API_KEY .env
# Should NOT be empty or "your-openai-api-key"
```

### Problem: High token usage (>1000 input tokens)

**Check prompt in logs:**
```bash
docker logs python-ai-service | grep "Market context prepared"
```

**Should see compact format:**
```
BTCUSDT $50000
1H: RSI:50.0...
```

**If still verbose:**
- Verify main.py changes applied: `grep "Crypto trading analyst" python-ai-service/main.py`

### Problem: Cache not working

**Check cache duration:**
```bash
grep "duration_minutes" python-ai-service/config.yaml
```

**Expected:** `duration_minutes: 15`

**Check MongoDB:**
```bash
docker exec -it mongodb mongosh --eval 'db.ai_analysis_results.find().limit(1)'
```

---

## 📈 Long-Term Monitoring (1 week)

### Daily Checks

**Morning (9 AM):**
```bash
# Check overnight costs
curl http://localhost:8000/ai/cost/statistics | \
  jq '{daily_cost: .projections.estimated_daily_cost_usd, total_today: .session_statistics.total_cost_usd}'
```

**Evening (9 PM):**
```bash
# Check full day costs
docker logs python-ai-service | grep "💰 Cost" | tail -20
```

### Weekly Review

**After 7 days:**
```bash
# Get full week statistics
curl http://localhost:8000/ai/cost/statistics > cost-week1.json

# Calculate actual weekly cost
WEEKLY_COST=$(jq '.session_statistics.total_cost_usd' cost-week1.json)
echo "Weekly cost: $${WEEKLY_COST}"
echo "Projected monthly: $(echo "$WEEKLY_COST * 4.3" | bc)"
```

**Expected:** ~$8-9 per week (~$36/month) ✅

---

## 🎯 Final Validation

**After 24 hours of running:**

1. **Cost Check:**
   - Daily cost: $0.62 - $1.20 ✅
   - Monthly projection: $18.60 - $36.00 ✅

2. **Quality Check:**
   - Review signals: `curl http://localhost:8000/ai/storage/stats`
   - Average confidence > 0.5 ✅

3. **Performance Check:**
   - Analysis interval: 10 minutes ✅
   - Cache duration: 15 minutes ✅
   - No errors in logs ✅

**If all checks pass: Optimization successful! 🎉**

---

## 📝 Test Results Log

Record your test results:

```markdown
# Test Results - [Date]

## Environment
- Service: python-ai-service
- Commit: [git rev-parse --short HEAD]
- OpenAI API Key: Configured ✅

## Test Results
- [ ] Service started: Yes/No
- [ ] Health check: Pass/Fail
- [ ] First request cost: $_____ (target: <$0.001)
- [ ] Input tokens: ____ (target: <500)
- [ ] Output tokens: ____ (target: <1000)
- [ ] Cache working: Yes/No
- [ ] Analysis interval: ____ min (target: 10)
- [ ] 24h cost: $_____ (target: $0.62-$1.20)

## Issues Found
- None / [List issues]

## Conclusion
- Success ✅ / Needs adjustment ⚠️
```

---

**Quick Reference Commands:**

```bash
# Start
./scripts/bot.sh start --memory-optimized

# Health
curl http://localhost:8000/health | jq

# Cost stats
curl http://localhost:8000/ai/cost/statistics | jq

# Watch logs
docker logs -f python-ai-service | grep "💰"

# Monitor
./monitor-costs.sh
```

**Expected daily cost: $0.62 - $1.20**
**Expected monthly cost: $18.60 - $36.00**
**Savings: 63% ✅**
