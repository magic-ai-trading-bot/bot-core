# 🚀 Hướng Dẫn Khởi Động Với OpenAI API Key Mới

**Ngày tạo:** 2024-11-19
**Tối ưu hóa:** Đã áp dụng (tiết kiệm 63% chi phí)

---

## ⚠️ BẢO MẬT API KEY

### ❌ **ĐỪNG BAO GIỜ:**
- ❌ Paste API key vào chat/message
- ❌ Share API key qua email/Slack
- ❌ Commit API key vào Git
- ❌ Screenshot chứa API key

### ✅ **NÊN:**
- ✅ Lưu API key trong file `.env` (đã được gitignore)
- ✅ Sử dụng script an toàn để setup
- ✅ Revoke key ngay nếu bị lộ
- ✅ Rotate key định kỳ (3-6 tháng)

---

## 🔐 BƯỚC 1: THU HỒI KEY CŨ (QUAN TRỌNG!)

**Key bị lộ:** `sk-proj-hCQBYSka1lf...` (trong chat trước)

### Thu hồi ngay:
1. Truy cập: https://platform.openai.com/api-keys
2. Đăng nhập vào tài khoản OpenAI
3. Tìm key: `sk-proj-hCQBYSka1lf...`
4. Click **"Revoke"** hoặc **"Delete"**
5. Confirm deletion

**⏰ Làm ngay bây giờ để tránh bị charge!**

---

## 🆕 BƯỚC 2: TẠO API KEY MỚI

### Tạo key:
1. Vẫn ở: https://platform.openai.com/api-keys
2. Click **"Create new secret key"**
3. Đặt tên: `bot-core-production` (hoặc tên bạn muốn)
4. **QUAN TRỌNG:** Copy key ngay (chỉ hiện 1 lần!)
   - Key sẽ có dạng: `sk-proj-...` (khoảng 100-150 ký tự)

### Kiểm tra billing:
5. Vào: https://platform.openai.com/settings/organization/billing
6. Đảm bảo có credit hoặc payment method
7. Set spending limit (khuyến nghị: $50/month)

---

## 🔧 BƯỚC 3: CẤU HÌNH API KEY

### Option 1: Sử dụng Script An Toàn (Khuyến Nghị)

```bash
cd /Users/dungngo97/Documents/bot-core
./scripts/setup-openai-key.sh
```

**Script sẽ:**
- ✅ Backup file `.env` hiện tại
- ✅ Validate format của API key
- ✅ Tự động thêm/update `OPENAI_API_KEY`
- ✅ Không hiển thị key ra console

**Khi chạy:**
1. Paste API key khi được hỏi (input sẽ ẩn)
2. Nhấn Enter
3. Đợi thông báo "✅ Success!"

### Option 2: Thủ Công

```bash
# Mở .env
nano .env

# Tìm dòng (khoảng dòng 20):
# API Keys - NEVER commit real keys

# Thêm vào ngay sau dòng BINANCE_SECRET_KEY:
OPENAI_API_KEY=sk-proj-[PASTE_KEY_CỦA_BẠN_VÀO_ĐÂY]

# Lưu: Ctrl+O, Enter, Ctrl+X
```

### Verify:

```bash
# Check key đã được set (ẩn key)
grep "OPENAI_API_KEY" .env | sed 's/\(OPENAI_API_KEY=sk-proj-.\{10\}\).*$/\1.../'
```

**Expected:** `OPENAI_API_KEY=sk-proj-hC...`

---

## 🚀 BƯỚC 4: KHỞI ĐỘNG SERVICES

### Stop services cũ (nếu đang chạy):
```bash
./scripts/bot.sh stop
```

### Start với optimization:
```bash
./scripts/bot.sh start --memory-optimized
```

### Đợi services khởi động (~1-2 phút):
```bash
# Watch logs
docker logs -f python-ai-service

# Đợi thấy:
# ✅ Direct OpenAI HTTP client initialized successfully
# ✅ OpenAI GPT-4 client ready for analysis
# 🔄 Started periodic analysis task (every 10 minutes)

# Nhấn Ctrl+C khi thấy messages trên
```

---

## ✅ BƯỚC 5: XÁC NHẬN HOẠT ĐỘNG

### Test 1: Health Check
```bash
curl http://localhost:8000/health | jq
```

**Expected:**
```json
{
  "status": "healthy",
  "gpt4_available": true,         // ← Phải là true
  "api_key_configured": true,     // ← Phải là true
  "analysis_interval_minutes": 10 // ← Optimized (was 5)
}
```

### Test 2: Debug GPT-4
```bash
curl http://localhost:8000/debug/gpt4 | jq
```

**Expected:**
```json
{
  "status": "success",
  "test_response": "SUCCESS",
  "model_used": "gpt-4o-mini"
}
```

**If error:** Check API key và billing status

### Test 3: Cost Statistics (Initial)
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
    "analysis_interval_minutes": 10,
    "cache_duration_minutes": 15,
    "max_tokens": 1200
  },
  "optimization_status": {
    "estimated_savings_percent": 63
  }
}
```

---

## 📊 BƯỚC 6: MONITOR CHI PHÍ

### Watch First Request (~10 phút):
```bash
# Watch logs cho first analysis
docker logs -f python-ai-service | grep "💰 Cost"

# Expected (sau ~10 phút):
# 💰 Cost: $0.00053 | Tokens: 280 in + 820 out = 1100 |
#    Total today: $0.00053 (1 requests)
```

### Kiểm tra sau 1 giờ:
```bash
curl http://localhost:8000/ai/cost/statistics | jq '.session_statistics'
```

**Expected (sau ~1 giờ, ~48 requests):**
```json
{
  "total_requests": 48,
  "total_input_tokens": 13440,      // ~280 per request
  "total_output_tokens": 39360,     // ~820 per request
  "total_tokens": 52800,
  "total_cost_usd": 0.0252,         // ~$0.025 for 48 requests
  "average_cost_per_request_usd": 0.000525
}
```

### Dashboard Real-time:
```bash
# Run monitor script
cat > monitor-costs.sh << 'SCRIPT'
#!/bin/bash
while true; do
  clear
  echo "=== Bot-Core GPT-4 Cost Monitor ==="
  echo "Time: $(date)"
  echo ""

  curl -s http://localhost:8000/ai/cost/statistics | jq '{
    requests: .session_statistics.total_requests,
    cost_usd: .session_statistics.total_cost_usd,
    cost_vnd: .session_statistics.total_cost_vnd,
    daily_projection_usd: .projections.estimated_daily_cost_usd,
    monthly_projection_usd: .projections.estimated_monthly_cost_usd,
    savings: .optimization_status.estimated_savings_percent
  }'

  echo ""
  echo "Recent cost logs:"
  docker logs python-ai-service 2>&1 | grep "💰 Cost" | tail -5

  echo ""
  echo "Press Ctrl+C to exit. Refreshing in 30s..."
  sleep 30
done
SCRIPT

chmod +x monitor-costs.sh
./monitor-costs.sh
```

---

## 📈 KỲ VỌNG CHI PHÍ

### Với Optimization (Hiện tại):

| Thời gian | Requests | Chi phí (USD) | Chi phí (VNĐ) |
|-----------|----------|---------------|---------------|
| **1 giờ** | ~48 | $0.025 | ~575 VNĐ |
| **1 ngày** | ~1,152 | $0.60 - $1.20 | 14k - 28k VNĐ |
| **1 tuần** | ~8,064 | $4.20 - $8.40 | 97k - 193k VNĐ |
| **1 tháng** | ~34,560 | $18.60 - $36.00 | 428k - 828k VNĐ |

### So sánh trước Optimization:

| Metric | Trước | Sau | Tiết kiệm |
|--------|-------|-----|-----------|
| Requests/ngày | 2,304 | 1,152 | -50% |
| Chi phí/ngày | $3.23 | $1.20 | -63% |
| Chi phí/tháng | $96.90 | $36.00 | **-$60.90** |

**💰 Tiết kiệm: 1.4 triệu VNĐ/tháng, 16.8 triệu VNĐ/năm**

---

## 🎯 CHECKLIST THÀNH CÔNG

Sau 24 giờ chạy, verify:

- [ ] Service running: `docker ps | grep python-ai`
- [ ] Health check: `gpt4_available: true`
- [ ] Debug test: `status: success`
- [ ] First request logged: Thấy "💰 Cost" trong logs
- [ ] Input tokens: < 500 per request (target: ~280)
- [ ] Output tokens: < 1200 per request (target: ~820)
- [ ] Cost per request: < $0.001 (target: ~$0.0005)
- [ ] Daily cost: $0.62 - $1.20 ✅
- [ ] Monthly projection: $18.60 - $36.00 ✅
- [ ] Cache working: Requests trong 15min dùng cache
- [ ] Signal quality: Confidence > 0.5 ✅

**If all ✅: Optimization thành công!** 🎉

---

## 🚨 TROUBLESHOOTING

### Problem: `gpt4_available: false`

**Nguyên nhân:** API key không đúng hoặc không set

**Giải pháp:**
```bash
# Check key trong .env
grep OPENAI_API_KEY .env

# Nếu empty hoặc "your-openai-api-key":
./scripts/setup-openai-key.sh

# Restart services
./scripts/bot.sh stop
./scripts/bot.sh start --memory-optimized
```

### Problem: Debug test failed với 401

**Nguyên nhân:** API key invalid hoặc expired

**Giải pháp:**
1. Verify key trên: https://platform.openai.com/api-keys
2. Tạo key mới nếu cần
3. Update `.env` với key mới
4. Restart services

### Problem: Debug test failed với 429

**Nguyên nhân:** Rate limit exceeded

**Giải pháp:**
```bash
# Check logs
docker logs python-ai-service | grep "rate limit"

# Wait 1 phút và thử lại
sleep 60
curl http://localhost:8000/debug/gpt4 | jq
```

### Problem: High cost (>$0.002 per request)

**Nguyên nhân:** Optimization chưa apply

**Giải pháp:**
```bash
# Check optimization
curl http://localhost:8000/ai/cost/statistics | jq '.configuration'

# Expected:
# {
#   "analysis_interval_minutes": 10,  // Not 5
#   "cache_duration_minutes": 15,     // Not 5
#   "max_tokens": 1200                // Not 2000
# }

# Nếu values sai:
git pull origin main  # Get latest optimizations
./scripts/bot.sh stop
./scripts/bot.sh start --memory-optimized
```

### Problem: No cost logs sau 15 phút

**Nguyên nhân:** Periodic analysis chưa chạy hoặc GPT-4 không được call

**Giải pháp:**
```bash
# Check periodic analysis
docker logs python-ai-service | grep "periodic analysis"

# Should see:
# 🔄 Started periodic analysis task (every 10 minutes)
# 🎯 Completed AI analysis cycle for 8 symbols

# Nếu không thấy:
docker restart python-ai-service
```

---

## 📊 MONITORING DÀI HẠN

### Hàng ngày (9 AM):
```bash
# Check overnight costs
curl -s http://localhost:8000/ai/cost/statistics | \
  jq '{daily_cost: .projections.estimated_daily_cost_usd, session_total: .session_statistics.total_cost_usd}'
```

### Hàng tuần:
```bash
# Export weekly stats
curl -s http://localhost:8000/ai/cost/statistics > cost-week-$(date +%Y%m%d).json

# Review
jq '.session_statistics' cost-week-*.json
```

### Alerts (Optional):
```bash
# Add to crontab: crontab -e
0 */6 * * * /Users/dungngo97/Documents/bot-core/scripts/cost-alert.sh
```

**Create alert script:**
```bash
cat > scripts/cost-alert.sh << 'SCRIPT'
#!/bin/bash
DAILY_COST=$(curl -s http://localhost:8000/ai/cost/statistics | jq -r '.projections.estimated_daily_cost_usd')
if (( $(echo "$DAILY_COST > 2.0" | bc -l) )); then
  echo "⚠️ ALERT: Daily cost exceeded $2.00! Current: $${DAILY_COST}"
  # Send notification (email, Slack, etc.)
fi
SCRIPT
chmod +x scripts/cost-alert.sh
```

---

## 🎓 BEST PRACTICES

### Security:
- ✅ Rotate API keys mỗi 3-6 tháng
- ✅ Set spending limits trên OpenAI dashboard
- ✅ Monitor daily costs
- ✅ Revoke unused keys

### Cost Management:
- ✅ Review weekly cost statistics
- ✅ Adjust cache/interval nếu cần
- ✅ Monitor token usage trends
- ✅ Set up cost alerts

### Quality Assurance:
- ✅ Review signal confidence scores
- ✅ Compare GPT-4 vs fallback performance
- ✅ Monitor for API errors
- ✅ Check response quality in logs

---

## 📞 SUPPORT

### Kiểm tra logs:
```bash
docker logs -f python-ai-service
```

### Kiểm tra services:
```bash
docker ps
```

### Documentation:
- Optimization guide: `python-ai-service/docs/GPT4_COST_OPTIMIZATION.md`
- Quick test: `QUICK_TEST_GUIDE.md`
- Summary: `OPTIMIZATION_SUMMARY.md`

---

## ✅ SUMMARY

**Sau khi hoàn thành guide này, bạn sẽ có:**

✅ API key mới an toàn (key cũ đã revoke)
✅ Service đang chạy với optimization
✅ Cost monitoring active
✅ Chi phí giảm 63% (từ $96.90 → $36.00/month)
✅ Real-time cost tracking
✅ Documentation đầy đủ

**Expected monthly cost: $18.60 - $36.00 (~428k - 828k VNĐ)**
**Tiết kiệm: $60.90/month (~1.4 triệu VNĐ/tháng)**

🎉 **Chúc mừng! Hệ thống đã sẵn sàng production!**
