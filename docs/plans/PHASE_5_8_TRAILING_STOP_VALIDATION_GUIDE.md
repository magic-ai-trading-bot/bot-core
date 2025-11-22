# Phase 5.8: Trailing Stop Validation Guide

**Date**: November 20, 2025
**Status**: ⏳ **READY FOR VALIDATION**
**Prerequisites**: ✅ Code complete, ✅ Tests passing (17/17)

---

## 🎯 **VALIDATION OBJECTIVE**

Verify that trailing stops work correctly in **real paper trading** environment:
- ✅ Trailing activates after 5% profit
- ✅ Stop moves only in favorable direction
- ✅ Profit improvement of 15-25% vs fixed SL/TP
- ✅ Zero unexpected behaviors or bugs

---

## ⚠️ **CURRENT STATUS**

**System State**:
- ✅ Paper trading engine: RUNNING
- ✅ Open trades: 2 active (SOLUSDT Long)
- ⚠️ **Trailing stop code: NOT DEPLOYED** (Docker running old binary)

**Problem**: Current trades were opened BEFORE trailing stop implementation.
**Solution**: Need to rebuild and redeploy to test trailing stops.

---

## 🚀 **DEPLOYMENT STEPS**

### **Option A: Safe Rebuild (Recommended)**

**Step 1: Rebuild Rust Engine**
```bash
cd /Users/dungngo97/Documents/bot-core

# Rebuild with new trailing stop code
docker-compose build rust-core-engine-dev

# Should see compilation of new code (2-3 minutes)
```

**Step 2: Restart Service**
```bash
# Restart only rust-core-engine (preserves other services)
docker-compose restart rust-core-engine-dev

# Verify healthy
docker ps | grep rust-core-engine
```

**Step 3: Verify Deployment**
```bash
# Check logs for successful start
docker logs --tail 50 rust-core-engine-dev

# Should see: "🚀 Paper trading engine started"
```

---

### **Option B: Full Clean Restart (If Issues)**

```bash
cd /Users/dungngo97/Documents/bot-core

# Stop all services
./scripts/bot.sh stop

# Rebuild rust-core-engine
docker-compose build rust-core-engine-dev

# Start all services
./scripts/bot.sh start

# Wait for healthy (2-3 minutes)
./scripts/bot.sh status
```

---

## 📊 **MONITORING TRAILING STOPS**

### **Real-Time Log Monitoring**

**Follow logs for trailing stop events**:
```bash
# Terminal 1: Watch for trailing stop activations
docker logs -f rust-core-engine-dev 2>&1 | grep "🎯"

# Terminal 2: Watch for trailing stop updates
docker logs -f rust-core-engine-dev 2>&1 | grep "📈"

# Terminal 3: Watch for trades and exits
docker logs -f rust-core-engine-dev 2>&1 | grep -E "(💸|Open|Closed)"
```

### **Expected Log Messages**

**Trailing Activation** (at +5% profit):
```
🎯 Trailing stop ACTIVATED for BTCUSDT at $55000.00 (+5.23%)
```

**Trailing Update** (stop moves up):
```
📈 Trailing SL updated: BTCUSDT $53350.00 → $55110.00 (best: $56700.00)
```

**Trade Exit** (trailing stop hit):
```
💸 Trade closed: BTCUSDT Long @ $55110.00 (SL hit) | Profit: +$2335.50 (+4.24%)
```

---

## 🧪 **VALIDATION CHECKLIST**

### **1. Verify Trailing Activation**
- [ ] Trade reaches +5% profit
- [ ] Log shows: `🎯 Trailing stop ACTIVATED`
- [ ] API shows `trailing_stop_active: true`
- [ ] `highest_price_achieved` is set to current price

**Test**:
```bash
# Check open trades for trailing fields
curl -s http://localhost:8080/api/paper-trading/trades/open | \
  python3 -c "import sys,json; trades=json.load(sys.stdin)['data']; \
  print(f'Trailing fields: {[(t[\"symbol\"], t.get(\"trailing_stop_active\", \"N/A\"), t.get(\"highest_price_achieved\", \"N/A\")) for t in trades]}')"
```

### **2. Verify Stop Movement (Long)**
- [ ] Price moves from $100 → $110 (+10%)
- [ ] Stop moves from $95 → $106.70 (3% below $110)
- [ ] Log shows: `📈 Trailing SL updated`
- [ ] Price drops to $108, stop STAYS at $106.70
- [ ] Price drops to $106, stop HITS at $106.70

**Monitor**:
```bash
# Watch price and stop loss changes
watch -n 5 'curl -s http://localhost:8080/api/paper-trading/trades/open | \
  python3 -m json.tool | grep -A 3 -E "(symbol|entry_price|stop_loss|pnl_percentage)"'
```

### **3. Verify Stop Movement (Short)**
- [ ] Price moves from $100 → $90 (-10%)
- [ ] Stop moves from $105 → $92.70 (3% above $90)
- [ ] Price rises to $92, stop STAYS at $92.70
- [ ] Price rises to $93, stop HITS at $92.70

### **4. Verify One-Way Movement**
- [ ] **Long**: Stop only moves UP, never DOWN
- [ ] **Short**: Stop only moves DOWN, never UP
- [ ] Price retracements DON'T move stop backwards

### **5. Verify Profit Improvement**
Compare trades WITH vs WITHOUT trailing:

**Without Trailing**:
```
Trade: BTCUSDT Long
Entry: $50,000
Exit: $55,000 (TP hit)
Profit: +$5,000 (+10%)
```

**With Trailing**:
```
Trade: BTCUSDT Long
Entry: $50,000
Peak: $57,000 (+14%)
Exit: $55,290 (trailing stop at 3% below $57k)
Profit: +$5,290 (+10.58%)
Improvement: +$290 (+5.8% more profit)
```

---

## 📈 **API VALIDATION**

### **Check Trailing Stop Status**

**Get Open Trades**:
```bash
curl -s http://localhost:8080/api/paper-trading/trades/open | python3 -m json.tool
```

**Expected Fields** (after rebuild):
```json
{
  "id": "...",
  "symbol": "BTCUSDT",
  "entry_price": 50000.0,
  "stop_loss": 53350.0,
  "highest_price_achieved": 54950.0,    // ← NEW FIELD
  "trailing_stop_active": true,         // ← NEW FIELD
  "pnl_percentage": 9.9
}
```

**Check Settings**:
```bash
curl -s http://localhost:8080/api/paper-trading/settings/risk | python3 -m json.tool
```

**Expected Settings**:
```json
{
  "trailing_stop_enabled": true,
  "trailing_stop_pct": 3.0,
  "trailing_activation_pct": 5.0
}
```

---

## 🎯 **SUCCESS CRITERIA**

**For Phase 5.8 Completion**:
1. ✅ At least 1 trade with trailing stop activated
2. ✅ Stop moves correctly (one-way movement verified)
3. ✅ Profit improvement measured (>0% vs fixed SL/TP)
4. ✅ Zero unexpected behaviors or crashes
5. ✅ Logs show activation and update messages
6. ✅ API returns trailing stop fields correctly

**Validation Duration**: 2-4 hours (depends on trade frequency)

---

## 📊 **VALIDATION SCENARIOS**

### **Scenario 1: Quick Win (Best Case)**
```
Timeline: 30 minutes
- New trade opens at $100
- Price moves to $105 (+5%) → Trailing activates
- Price moves to $110 (+10%) → Stop at $106.70
- Price drops to $107 → Stop hits at $106.70
- Result: +6.7% profit vs +5% without trailing
```

### **Scenario 2: Extended Move (Ideal Case)**
```
Timeline: 2-3 hours
- New trade opens at $100
- Price moves to $115 (+15%) → Trailing active, stop at $111.55
- Price moves to $120 (+20%) → Stop at $116.40
- Price drops to $116.50 → Stop hits at $116.40
- Result: +16.4% profit vs +10% TP
- Improvement: +64% more profit!
```

### **Scenario 3: Failed Activation (Expected)**
```
Timeline: 1 hour
- New trade opens at $100
- Price moves to $103 (+3%) → Below 5% threshold
- Price drops to $95 → Fixed SL hits at $95
- Result: -5% loss (trailing never activated)
- Behavior: CORRECT (needs 5% to activate)
```

---

## 🔍 **TROUBLESHOOTING**

### **Problem: No Trailing Fields in API**
**Cause**: Docker running old binary
**Fix**: Rebuild with `docker-compose build rust-core-engine-dev`

### **Problem: Trailing Never Activates**
**Cause**: Trades not reaching +5% profit
**Fix**: Either wait longer OR reduce `trailing_activation_pct` to 3%

### **Problem: Stop Moves in Wrong Direction**
**Cause**: Logic bug (should not happen, tests passed)
**Fix**: Check logs, file bug report

### **Problem: No Activation Logs**
**Cause**: Logging level too high
**Fix**: Set `RUST_LOG=info` in docker-compose.yml

---

## 📝 **VALIDATION REPORT TEMPLATE**

After validation, create `PHASE_5_8_VALIDATION_RESULTS.md`:

```markdown
# Phase 5.8 Validation Results

**Date**: [Date]
**Duration**: [Hours] hours monitoring
**Trades Monitored**: [N] trades

## Results

### Trailing Activation
- Trades with +5% profit: [N]
- Trailing activated: [N] / [N] (100%)
- Activation logs verified: ✅ / ❌

### Stop Movement
- Long positions: [N] trades
  - Stop moved up correctly: ✅ / ❌
  - Stop stayed on dips: ✅ / ❌
- Short positions: [N] trades
  - Stop moved down correctly: ✅ / ❌
  - Stop stayed on rises: ✅ / ❌

### Profit Improvement
- Average profit without trailing: +[X]%
- Average profit with trailing: +[Y]%
- Improvement: +[Z]% ([Z/X * 100]% more)

### Issues Found
- None / [List issues]

## Conclusion
✅ PASSED - Trailing stops working as designed
❌ FAILED - [Reasons]
```

---

## 🚀 **NEXT STEPS AFTER VALIDATION**

1. **If PASSED** ✅:
   - Mark Phase 5.8 complete
   - Move to Phase 6 (Reduce Signal Frequency)
   - Keep trailing stops enabled for all trading

2. **If FAILED** ❌:
   - Document bugs found
   - Fix issues
   - Re-run tests
   - Repeat validation

3. **If INCONCLUSIVE** ⏳:
   - Extend monitoring period
   - Trigger more test trades
   - Collect more data

---

## 📊 **ESTIMATED TIMELINE**

**Immediate** (5 minutes):
- ✅ Rebuild docker image
- ✅ Restart service
- ✅ Verify deployment

**Short-term** (30-60 minutes):
- ⏳ Wait for first new trade
- ⏳ Monitor for trailing activation
- ⏳ Verify first trailing update

**Complete** (2-4 hours):
- ⏳ Monitor 3-5 trades
- ⏳ Measure profit improvement
- ⏳ Create validation report
- ⏳ Mark Phase 5.8 complete

---

**Status**: ⏳ **READY TO START VALIDATION**
**Action**: Run deployment steps above to begin
**Expected Completion**: 2-4 hours from deployment

---

🤖 **Generated with [Claude Code](https://claude.com/claude-code)**

**Co-Authored-By**: Claude <noreply@anthropic.com>
