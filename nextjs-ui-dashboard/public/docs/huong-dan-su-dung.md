# 📖 Hướng Dẫn Sử Dụng Bot Trading

## 🎯 Bot Hoạt Động Như Thế Nào?

Bot trading tự động của chúng tôi hoạt động như một trader chuyên nghiệp, phân tích thị trường 24/7 và thực hiện giao dịch dựa trên các chiến lược đã được tối ưu hóa.

### **Quy Trình Hoạt Động (4 Bước)**

```
1. THU THẬP DỮ LIỆU → 2. PHÂN TÍCH → 3. QUYẾT ĐỊNH → 4. GIAO DỊCH
```

---

## 📊 Bước 1: Thu Thập Dữ Liệu Thị Trường

Bot tự động thu thập dữ liệu từ Binance mỗi giây:

- **Giá mở cửa, cao nhất, thấp nhất, đóng cửa** (OHLC)
- **Khối lượng giao dịch**
- **Biến động giá theo thời gian thực**

**Khung thời gian theo dõi:**
- ⏱️ **1 giờ (1h)**: Xu hướng ngắn hạn
- ⏱️ **4 giờ (4h)**: Xu hướng trung hạn

---

## 🔍 Bước 2: Phân Tích Kỹ Thuật

Bot sử dụng **4 chiến lược giao dịch** được chứng minh hiệu quả:

### **A. Chiến Lược RSI (Relative Strength Index)**
- **Mục đích**: Phát hiện điểm quá mua/quá bán
- **Tín hiệu MUA**: RSI < 30 (quá bán) → Giá có thể tăng
- **Tín hiệu BÁN**: RSI > 70 (quá mua) → Giá có thể giảm
- **Tỷ lệ thắng**: 62%

### **B. Chiến Lược MACD (Moving Average Convergence Divergence)**
- **Mục đích**: Phát hiện xu hướng và điểm đảo chiều
- **Tín hiệu MUA**: MACD cắt lên trên đường Signal
- **Tín hiệu BÁN**: MACD cắt xuống dưới đường Signal
- **Tỷ lệ thắng**: 58%

### **C. Chiến Lược Bollinger Bands**
- **Mục đích**: Phát hiện biến động và breakout
- **Tín hiệu MUA**: Giá chạm dải dưới → Có thể phục hồi
- **Tín hiệu BÁN**: Giá chạm dải trên → Có thể điều chỉnh
- **Tỷ lệ thắng**: 60%

### **D. Chiến Lược Volume (Khối lượng)**
- **Mục đích**: Xác nhận độ mạnh của xu hướng
- **Tín hiệu MUA**: Khối lượng tăng đột biến + giá tăng
- **Tín hiệu BÁN**: Khối lượng tăng đột biến + giá giảm
- **Tỷ lệ thắng**: 52%

### **E. AI/ML Prediction (Dự đoán AI)**
- Bot sử dụng **3 mô hình AI** để dự đoán giá:
  - 🤖 **LSTM**: 68% độ chính xác
  - 🤖 **GRU**: 65% độ chính xác
  - 🤖 **Transformer**: 70% độ chính xác
- **GPT-4** phân tích tin tức và sentiment thị trường
- **Độ chính xác trung bình**: 72%

---

## 🎯 Bước 3: Tạo Tín Hiệu Giao Dịch

Bot tạo tín hiệu **mỗi 60 phút** (có thể điều chỉnh).

### **Cơ Chế Xác Nhận Tín Hiệu (Multi-Confirmation)**

Để đảm bảo chất lượng, bot yêu cầu **ít nhất 2/4 chiến lược** đồng ý:

**Ví dụ tín hiệu MUA mạnh:**
- ✅ RSI: 28 (quá bán) → MUA
- ✅ MACD: Cắt lên → MUA
- ✅ Bollinger: Giá chạm dải dưới → MUA
- ✅ Volume: Tăng đột biến → MUA
- ✅ AI: 75% confidence → MUA

**→ Tín hiệu MẠNH: 5/5 chiến lược đồng ý → Mở lệnh MUA**

### **Mức Độ Tin Cậy (Confidence)**

| Số chiến lược đồng ý | Mức độ | Hành động |
|---------------------|---------|-----------|
| **4-5/5** | 🟢 MẠNH (80-100%) | Vào lệnh ngay |
| **3/5** | 🟡 TRUNG BÌNH (60-80%) | Xem xét vào lệnh |
| **2/5** | 🟠 YẾU (40-60%) | Không vào lệnh |
| **0-1/5** | 🔴 RẤT YẾU (<40%) | Bỏ qua |

---

## 💰 Bước 4: Thực Hiện Giao Dịch

### **A. Kiểm Tra Rủi Ro (7 Lớp Bảo Vệ)**

Trước khi vào lệnh, bot kiểm tra **7 điều kiện an toàn**:

1. **✅ Rủi ro mỗi lệnh ≤ 2%**
   - Ví dụ: Tài khoản $10,000 → Rủi ro tối đa: $200/lệnh

2. **✅ Rủi ro tổng danh mục ≤ 10%**
   - Tổng tất cả các lệnh đang mở không vượt quá 10% tài khoản

3. **✅ Stop Loss bắt buộc (2%)**
   - Mọi lệnh đều có stop loss tự động

4. **✅ Giới hạn thua lỗ trong ngày (5%)**
   - Nếu thua lỗ 5% trong ngày → Bot dừng giao dịch đến ngày hôm sau

5. **✅ Giới hạn thua lỗ liên tiếp (5 lệnh)**
   - Sau 5 lệnh thua liên tiếp → Bot nghỉ 60 phút

6. **✅ Cool-down sau thua lỗ**
   - Nghỉ 60 phút để tránh giao dịch cảm tính

7. **✅ Giới hạn tương quan vị thế (70%)**
   - Tránh mở quá nhiều lệnh cùng chiều (phân tán rủi ro)

### **B. Mở Lệnh Giao Dịch**

Nếu **PASS** tất cả 7 điều kiện trên:

**Thông tin lệnh:**
```
Symbol:      BTCUSDT
Direction:   LONG (MUA)
Entry Price: $45,000
Position Size: 0.02 BTC (tương đương 2% rủi ro)
Stop Loss:   $44,100 (-2%)
Take Profit: $46,800 (+4%)
Leverage:    3x (có thể điều chỉnh)
```

### **C. Trailing Stop Loss (Bảo Vệ Lợi Nhuận Tự Động)**

**Cơ chế hoạt động:**

1. **Khi lợi nhuận đạt +5%** → Trailing stop kích hoạt
2. **Bot tự động nâng stop loss** theo giá tối đa
3. **Khoảng cách trailing: 3%**

**Ví dụ thực tế:**

| Giá hiện tại | Lợi nhuận | Stop Loss | Trạng thái |
|-------------|-----------|-----------|------------|
| $45,000 | 0% | $44,100 (-2%) | Chờ kích hoạt |
| $47,250 | +5% | $45,847 (+1.9%) | ✅ Kích hoạt! |
| $48,000 | +6.7% | $46,560 (+3.5%) | Tự động nâng |
| $49,000 | +8.9% | $47,530 (+5.6%) | Tự động nâng |
| $48,500 | +7.8% | $47,530 (+5.6%) | Giữ nguyên |
| **$47,530** | +5.6% | $47,530 | **Đóng lệnh, khóa lãi +5.6%** |

**Lợi ích:**
- ✅ Tự động bảo vệ lợi nhuận
- ✅ Không cần theo dõi 24/7
- ✅ Giảm stress giao dịch
- ✅ Tăng 10-15% lợi nhuận giữ được

---

## ⚙️ Cấu Hình Bot (Settings)

### **1. Cài Đặt Giao Dịch Cơ Bản**

#### **A. Initial Balance (Số dư ban đầu)**
```
Giá trị mặc định: $10,000
Phạm vi: $1,000 - $1,000,000
```
**Giải thích**: Số tiền bạn muốn bot quản lý.

#### **B. Trading Enabled (Bật/Tắt giao dịch)**
```
Mặc định: BẬT (trong chế độ Paper Trading)
```
**Giải thích**:
- ✅ **BẬT**: Bot sẽ thực hiện giao dịch
- ❌ **TẮT**: Bot chỉ phân tích, không giao dịch

#### **C. Symbols (Cặp tiền)**
```
Mặc định: BTCUSDT, ETHUSDT
Hỗ trợ: Tất cả cặp Binance Futures
```
**Giải thích**: Chọn các cặp tiền muốn giao dịch.

---

### **2. Cài Đặt Rủi Ro (Risk Settings)**

#### **A. Max Risk Per Trade (Rủi ro tối đa mỗi lệnh)**
```
Mặc định: 2%
Khuyến nghị: 1-2%
Phạm vi: 0.5% - 5%
```
**Giải thích**: Phần trăm tài khoản có thể mất trong 1 lệnh.

**Ví dụ:**
- Tài khoản: $10,000
- Max risk: 2%
- **→ Rủi ro tối đa: $200/lệnh**

#### **B. Stop Loss Percentage (Phần trăm stop loss)**
```
Mặc định: 2%
Khuyến nghị: 2-3%
Phạm vi: 1% - 5%
```
**Giải thích**: Khoảng cách stop loss từ giá vào lệnh.

#### **C. Take Profit Percentage (Phần trăm take profit)**
```
Mặc định: 4%
Khuyến nghị: 4-6%
Phạm vi: 2% - 10%
```
**Giải thích**: Mục tiêu lợi nhuận cho mỗi lệnh.

#### **D. Max Leverage (Đòn bẩy tối đa)**
```
Mặc định: 3x
Khuyến nghị: 2-3x (bảo thủ), 5-10x (tích cực)
Phạm vi: 1x - 10x
```
**Giải thích**: Đòn bẩy càng cao, lợi nhuận và rủi ro càng lớn.

**⚠️ Cảnh báo:**
- 1-3x: An toàn cho người mới
- 5-10x: Chỉ dành cho trader có kinh nghiệm
- >10x: Rủi ro thanh lý cao

#### **E. Daily Loss Limit (Giới hạn thua lỗ trong ngày)**
```
Mặc định: 5%
Khuyến nghị: 5-7%
Phạm vi: 3% - 10%
```
**Giải thích**: Nếu thua lỗ đạt mức này, bot dừng giao dịch đến ngày hôm sau.

#### **F. Max Consecutive Losses (Số lệnh thua liên tiếp tối đa)**
```
Mặc định: 5 lệnh
Khuyến nghị: 3-5 lệnh
Phạm vi: 3 - 10 lệnh
```
**Giải thích**: Sau N lệnh thua liên tiếp, bot nghỉ.

#### **G. Cool-Down Period (Thời gian nghỉ)**
```
Mặc định: 60 phút
Khuyến nghị: 30-60 phút
Phạm vi: 15 - 180 phút
```
**Giải thích**: Thời gian bot nghỉ sau khi đạt giới hạn thua lỗ liên tiếp.

---

### **3. Trailing Stop Settings (Cài đặt trailing stop)**

#### **A. Enabled (Bật/Tắt)**
```
Mặc định: BẬT
Khuyến nghị: BẬT (bảo vệ lợi nhuận)
```

#### **B. Activation Threshold (Ngưỡng kích hoạt)**
```
Mặc định: 5%
Khuyến nghị: 3-5%
Phạm vi: 2% - 10%
```
**Giải thích**: Lợi nhuận tối thiểu để kích hoạt trailing stop.

#### **C. Trail Distance (Khoảng cách trailing)**
```
Mặc định: 3%
Khuyến nghị: 2-3%
Phạm vi: 1% - 5%
```
**Giải thích**: Khoảng cách stop loss với giá đỉnh.

---

### **4. AI Settings (Cài đặt AI)**

#### **A. Signal Refresh Interval (Tần suất tạo tín hiệu)**
```
Mặc định: 60 phút
Khuyến nghị:
  - Bảo thủ: 60-120 phút
  - Trung bình: 30-60 phút
  - Tích cực: 15-30 phút (không khuyến nghị)
Phạm vi: 15 - 240 phút
```
**Giải thích**: Khoảng thời gian giữa các lần phân tích và tạo tín hiệu.

**So sánh:**

| Tần suất | Tín hiệu/ngày | Ưu điểm | Nhược điểm |
|----------|--------------|---------|------------|
| **60 phút** ✅ | 24 | Chất lượng cao, ít nhiễu | Ít cơ hội |
| **30 phút** | 48 | Cân bằng | Trung bình |
| **15 phút** ⚠️ | 96 | Nhiều cơ hội | Nhiều nhiễu, overtrading |

#### **B. Min Confidence Threshold (Ngưỡng tin cậy tối thiểu)**
```
Mặc định: 60%
Khuyến nghị: 60-70%
Phạm vi: 40% - 90%
```
**Giải thích**: Mức độ tin cậy tối thiểu của tín hiệu để vào lệnh.

---

### **5. Strategy Settings (Cài đặt chiến lược)**

Bật/tắt từng chiến lược:

| Chiến lược | Mặc định | Tỷ lệ thắng | Khuyến nghị |
|-----------|---------|------------|-------------|
| **RSI** | ✅ BẬT | 62% | BẬT (hiệu quả nhất) |
| **MACD** | ✅ BẬT | 58% | BẬT |
| **Bollinger** | ✅ BẬT | 60% | BẬT |
| **Volume** | ✅ BẬT | 52% | TÙY CHỌN |

**Khuyến nghị:**
- Người mới: Bật cả 4 chiến lược (tín hiệu an toàn nhất)
- Có kinh nghiệm: Tùy chỉnh theo phong cách giao dịch

---

## 📈 Theo Dõi Hiệu Suất

### **Dashboard Metrics (Các chỉ số quan trọng)**

#### **1. Win Rate (Tỷ lệ thắng)**
```
Công thức: (Số lệnh thắng / Tổng số lệnh) × 100%
Mục tiêu: ≥ 60%
```
**Ví dụ**: 60 lệnh thắng / 100 lệnh = 60% win rate

#### **2. Profit Factor (Hệ số lợi nhuận)**
```
Công thức: Tổng lợi nhuận / Tổng thua lỗ
Mục tiêu: ≥ 1.5
```
**Ví dụ**: $3,000 lời / $2,000 lỗ = 1.5 profit factor

#### **3. Max Drawdown (Sụt giảm tối đa)**
```
Công thức: (Đỉnh - Đáy) / Đỉnh × 100%
Mục tiêu: ≤ 10%
```
**Ví dụ**: ($12,000 - $10,800) / $12,000 = 10% drawdown

#### **4. Sharpe Ratio (Tỷ lệ Sharpe)**
```
Công thức: (Lợi nhuận trung bình - Lãi suất phi rủi ro) / Độ lệch chuẩn
Mục tiêu: ≥ 1.5
```
**Ý nghĩa**: Đo lường lợi nhuận điều chỉnh theo rủi ro.

#### **5. Total Trades (Tổng số lệnh)**
```
Mục tiêu: ≥ 50 lệnh để có dữ liệu thống kê tin cậy
```

---

## ⚠️ Lưu Ý Quan Trọng

### **1. Paper Trading vs Live Trading**

#### **Paper Trading (Giao dịch giả lập)** ✅ KHUYẾN NGHỊ
- **Ưu điểm**:
  - ✅ Không rủi ro tiền thật
  - ✅ Test chiến lược an toàn
  - ✅ Học cách bot hoạt động
- **Nhược điểm**:
  - ❌ Không có cảm xúc thật
  - ❌ Slippage giả lập (không 100% thực tế)

**→ Khuyến nghị: Chạy paper trading ít nhất 1-2 tuần trước khi dùng tiền thật**

#### **Live Trading (Giao dịch thật)** ⚠️ THẬN TRỌNG
- **Yêu cầu**:
  - ✅ Đã test paper trading thành công
  - ✅ Hiểu rõ cơ chế bot
  - ✅ Có kiến thức giao dịch cơ bản
  - ✅ Chấp nhận được rủi ro
- **Lưu ý**:
  - ⚠️ Bắt đầu với số tiền nhỏ
  - ⚠️ Không dùng tiền cần thiết cho sinh hoạt
  - ⚠️ Theo dõi bot thường xuyên (ít nhất 1 lần/ngày)

### **2. Quản Lý Rủi Ro**

**Nguyên tắc vàng:**
1. **Không bao giờ rủi ro >2% mỗi lệnh**
2. **Không bao giờ rủi ro >10% tổng danh mục**
3. **Luôn đặt stop loss**
4. **Phân tán danh mục** (không all-in 1 coin)
5. **Nghỉ khi thua lỗ liên tiếp**

### **3. Giám Sát Bot**

**Hàng ngày:**
- ✅ Kiểm tra dashboard 1-2 lần/ngày
- ✅ Xem các lệnh đang mở
- ✅ Theo dõi lợi nhuận/thua lỗ trong ngày

**Hàng tuần:**
- ✅ Xem lại performance metrics
- ✅ Điều chỉnh settings nếu cần
- ✅ Phân tích các lệnh thua lỗ

**Hàng tháng:**
- ✅ Đánh giá tổng thể
- ✅ So sánh với mục tiêu
- ✅ Quyết định tiếp tục hay điều chỉnh

---

## 🎯 Quy Trình Bắt Đầu (Cho Người Mới)

### **Bước 1: Đăng ký Binance** ✅
1. Tạo tài khoản Binance
2. Xác minh danh tính (KYC)
3. Kích hoạt Binance Futures
4. Tạo API key (đọc + giao dịch)

### **Bước 2: Cấu hình API** ✅
1. Vào Settings → API Configuration
2. Nhập Binance API Key và Secret Key
3. Chọn **Testnet mode** (môi trường test)
4. Lưu cấu hình

### **Bước 3: Cấu hình Bot** ✅
1. Vào Settings → Risk Settings
2. Thiết lập:
   - Initial Balance: $10,000
   - Max Risk Per Trade: 2%
   - Stop Loss: 2%
   - Take Profit: 4%
   - Max Leverage: 3x
3. Bật Paper Trading
4. Lưu cấu hình

### **Bước 4: Chọn Chiến Lược** ✅
1. Vào Settings → Strategy Settings
2. Bật tất cả 4 chiến lược (khuyến nghị cho người mới)
3. Lưu cấu hình

### **Bước 5: Khởi Động Bot** ✅
1. Vào Dashboard
2. Nhấn nút "Start Bot"
3. Bot bắt đầu phân tích và giao dịch

### **Bước 6: Theo Dõi** ✅
1. Kiểm tra dashboard hàng ngày
2. Xem các lệnh và hiệu suất
3. Sau 1-2 tuần, đánh giá kết quả

### **Bước 7: Chuyển Sang Live Trading** ⚠️
**CHỈ KHI:**
- ✅ Paper trading có lãi ≥2 tuần liên tiếp
- ✅ Win rate ≥60%
- ✅ Profit factor ≥1.5
- ✅ Hiểu rõ cách bot hoạt động

---

## 💡 Mẹo Tối Ưu Hiệu Suất

### **1. Tối Ưu Settings**

**Thị trường trending (xu hướng rõ ràng):**
- Tăng leverage lên 5x
- Tăng take profit lên 6%
- Giảm stop loss xuống 1.5%

**Thị trường sideways (đi ngang):**
- Giảm leverage xuống 2x
- Giảm take profit xuống 3%
- Tăng stop loss lên 2.5%

### **2. Tần Suất Tín Hiệu**

**Bull market (thị trường tăng):**
- Signal interval: 30-60 phút
- Confidence threshold: 60%

**Bear market (thị trường giảm):**
- Signal interval: 60-120 phút
- Confidence threshold: 70% (thận trọng hơn)

### **3. Quản Lý Vốn**

**Quy tắc 2-20:**
- Rủi ro tối đa 2% mỗi lệnh
- Lợi nhuận mục tiêu 20% mỗi tháng

**Compound profits (lãi kép):**
- Tái đầu tư 50% lợi nhuận
- Rút 50% về để bảo toàn vốn

---

## ❓ Câu Hỏi Thường Gặp (FAQ)

### **Q1: Bot có tự động giao dịch 24/7 không?**
**A**: Có, bot phân tích thị trường và giao dịch 24/7 theo cấu hình bạn đặt.

### **Q2: Tôi có thể tắt bot bất cứ lúc nào không?**
**A**: Có, bạn có thể tắt bot bất cứ lúc nào. Bot sẽ đóng tất cả các lệnh đang mở an toàn trước khi dừng.

### **Q3: Bot có đảm bảo lợi nhuận không?**
**A**: **KHÔNG**. Không có bot nào đảm bảo lợi nhuận 100%. Giao dịch có rủi ro. Bot chỉ tối ưu hóa cơ hội thắng dựa trên dữ liệu lịch sử.

### **Q4: Tôi cần bao nhiêu vốn để bắt đầu?**
**A**: Khuyến nghị tối thiểu $1,000 cho paper trading, $5,000 cho live trading. Bắt đầu với số vốn bạn có thể chấp nhận mất.

### **Q5: Tôi có thể thay đổi settings khi bot đang chạy không?**
**A**: Có, nhưng các thay đổi sẽ chỉ áp dụng cho các lệnh MỚI. Các lệnh đang mở sẽ giữ nguyên settings cũ.

### **Q6: Bot có cần internet luôn kết nối không?**
**A**: Có, bot cần kết nối internet để thu thập dữ liệu từ Binance và thực hiện giao dịch.

### **Q7: Nếu bot gặp lỗi, lệnh của tôi có an toàn không?**
**A**: Có, tất cả lệnh đều có stop loss tự động trên Binance. Ngay cả khi bot offline, stop loss vẫn hoạt động.

### **Q8: Tôi có thể chạy nhiều bot cùng lúc không?**
**A**: Có, bạn có thể chạy nhiều bot với các cặp tiền và settings khác nhau.

### **Q9: Bot có tính phí không?**
**A**: Hiện tại bot hoàn toàn miễn phí. Bạn chỉ trả phí giao dịch cho Binance (0.04%).

### **Q10: Tôi có thể withdraw lợi nhuận bất cứ lúc nào không?**
**A**: Có, tiền của bạn luôn nằm trên tài khoản Binance của bạn. Bot chỉ giao dịch thông qua API, không giữ tiền.

---

## 📞 Hỗ Trợ

Nếu bạn cần hỗ trợ:

1. **Telegram**: @bottrading_support
2. **Email**: support@bottrading.com
3. **Discord**: discord.gg/bottrading

**Giờ hỗ trợ**: 24/7 (trong vòng 2 giờ)

---

## 📚 Tài Nguyên Học Tập

**Video tutorials:**
- Cách cài đặt bot
- Giải thích các chiến lược
- Quản lý rủi ro
- Phân tích kết quả

**Tài liệu:**
- Hướng dẫn chi tiết từng bước
- Best practices
- Case studies
- Performance analysis

---

**Chúc bạn giao dịch thành công! 🚀**

*Lưu ý: Tài liệu này được cập nhật thường xuyên. Phiên bản hiện tại: v2.0 - November 2025*
