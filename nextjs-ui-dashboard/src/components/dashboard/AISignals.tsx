import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useAIAnalysis } from "@/hooks/useAIAnalysis";
import { useWebSocket } from "@/hooks/useWebSocket";
import { AIMarketAnalysis, AIRiskAssessment } from "@/services/api";
import {
  AlertCircle,
  RefreshCw,
  Zap,
  TrendingUp,
  Wifi,
  WifiOff,
  BarChart3,
  Target,
  Activity,
  Shield,
  ArrowUp,
  ArrowDown,
  Info,
} from "lucide-react";

// Strategy Information Database
const STRATEGY_INFO = {
  "RSI Strategy": {
    name: "RSI Strategy",
    description: "Relative Strength Index - Xác định điều kiện quá mua/quá bán",
    how_it_works:
      "RSI dao động từ 0-100. Trên 70 = quá mua (có thể bán), dưới 30 = quá bán (có thể mua)",
    signals: {
      buy: "RSI < 30 và bắt đầu tăng",
      sell: "RSI > 70 và bắt đầu giảm",
    },
    advantages: ["Dễ hiểu", "Hiệu quả trong sideway", "Tín hiệu rõ ràng"],
    disadvantages: [
      "Lag signal",
      "False signal trong trending",
      "Cần kết hợp indicator khác",
    ],
    best_timeframe: "1h, 4h, 1d",
    chart_description:
      "Đường RSI dao động với vùng quá mua (70+) và quá bán (30-)",
  },
  "MACD Strategy": {
    name: "MACD Strategy",
    description:
      "Moving Average Convergence Divergence - Phân tích xu hướng và momentum",
    how_it_works:
      "Sử dụng 2 đường EMA (12, 26) và đường signal (9). Khi MACD cắt lên signal = mua, cắt xuống = bán",
    signals: {
      buy: "MACD line cắt lên trên Signal line",
      sell: "MACD line cắt xuống dưới Signal line",
    },
    advantages: ["Bắt trend tốt", "Tín hiệu momentum", "Phù hợp swing trading"],
    disadvantages: [
      "Lag signal",
      "Nhiều false signal",
      "Không tốt trong sideway",
    ],
    best_timeframe: "4h, 1d, 1w",
    chart_description:
      "Histogram, MACD line và Signal line với crossover signals",
  },
  "Volume Strategy": {
    name: "Volume Strategy",
    description: "Phân tích khối lượng giao dịch - Xác định sức mạnh của trend",
    how_it_works:
      "Volume tăng = trend mạnh, volume giảm = trend yếu. Breakout với volume cao = tin cậy",
    signals: {
      buy: "Giá tăng + Volume tăng mạnh",
      sell: "Giá giảm + Volume tăng mạnh",
    },
    advantages: [
      "Xác nhận trend",
      "Phát hiện breakout",
      "Đánh giá sức mạnh move",
    ],
    disadvantages: [
      "Cần kết hợp price action",
      "Volume fake",
      "Khó đọc trong crypto",
    ],
    best_timeframe: "1h, 4h, 1d",
    chart_description:
      "Volume bars với price action, tìm sự tương quan tăng/giảm",
  },
  "Bollinger Bands Strategy": {
    name: "Bollinger Bands Strategy",
    description:
      "Volatility bands - Đo lường độ biến động và tìm levels support/resistance",
    how_it_works:
      "3 bands: Middle (SMA20), Upper (+2σ), Lower (-2σ). Giá chạm upper = quá mua, chạm lower = quá bán",
    signals: {
      buy: "Giá chạm Lower Band và bounce back",
      sell: "Giá chạm Upper Band và reject",
    },
    advantages: [
      "Dynamic S/R levels",
      "Đo volatility",
      "Mean reversion signals",
    ],
    disadvantages: [
      "Không tốt trong trending",
      "False breakouts",
      "Cần confirm khác",
    ],
    best_timeframe: "1h, 4h, 1d",
    chart_description:
      "3 đường bands tạo kênh price, squeeze/expansion patterns",
  },
};

// Strategy Explanation Dialog Component
function StrategyExplanationDialog({ strategyName }: { strategyName: string }) {
  const strategy = STRATEGY_INFO[strategyName as keyof typeof STRATEGY_INFO];

  if (!strategy) {
    return null;
  }

  return (
    <DialogContent className="max-w-4xl max-h-[85vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle className="flex items-center gap-2">
          <Activity className="h-5 w-5" />
          Giải thích Strategy: {strategy.name}
        </DialogTitle>
      </DialogHeader>

      <div className="space-y-6">
        {/* Strategy Overview */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">📖 Mô tả Strategy</CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm leading-relaxed mb-4">
              {strategy.description}
            </p>
            <div className="p-4 bg-muted/50 rounded-lg">
              <h4 className="font-medium mb-2">🔧 Cách hoạt động:</h4>
              <p className="text-sm">{strategy.how_it_works}</p>
            </div>
          </CardContent>
        </Card>

        {/* Trading Signals */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg text-profit">
                🟢 Tín hiệu MUA
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm">{strategy.signals.buy}</p>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-lg text-loss">
                🔴 Tín hiệu BÁN
              </CardTitle>
            </CardHeader>
            <CardContent>
              <p className="text-sm">{strategy.signals.sell}</p>
            </CardContent>
          </Card>
        </div>

        {/* Pros and Cons */}
        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
          <Card>
            <CardHeader>
              <CardTitle className="text-lg text-profit">✅ Ưu điểm</CardTitle>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2">
                {strategy.advantages.map((advantage, index) => (
                  <li key={index} className="flex items-center gap-2 text-sm">
                    <div className="w-2 h-2 bg-profit rounded-full"></div>
                    {advantage}
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>

          <Card>
            <CardHeader>
              <CardTitle className="text-lg text-loss">⚠️ Nhược điểm</CardTitle>
            </CardHeader>
            <CardContent>
              <ul className="space-y-2">
                {strategy.disadvantages.map((disadvantage, index) => (
                  <li key={index} className="flex items-center gap-2 text-sm">
                    <div className="w-2 h-2 bg-loss rounded-full"></div>
                    {disadvantage}
                  </li>
                ))}
              </ul>
            </CardContent>
          </Card>
        </div>

        {/* Usage Info */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">⚙️ Thông tin sử dụng</CardTitle>
          </CardHeader>
          <CardContent className="space-y-4">
            <div>
              <h4 className="font-medium mb-2">🕐 Timeframe tốt nhất:</h4>
              <p className="text-sm text-muted-foreground">
                {strategy.best_timeframe}
              </p>
            </div>

            <div>
              <h4 className="font-medium mb-2">📊 Mô tả biểu đồ:</h4>
              <p className="text-sm text-muted-foreground">
                {strategy.chart_description}
              </p>
            </div>
          </CardContent>
        </Card>

        {/* Chart Illustration */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">📈 Hình minh họa</CardTitle>
          </CardHeader>
          <CardContent>
            {/* RSI Strategy Chart */}
            {strategyName === "RSI Strategy" && (
              <div className="space-y-4">
                <div className="w-full h-64 bg-muted/10 rounded-lg p-4">
                  <svg
                    viewBox="0 0 400 200"
                    className="w-full h-full"
                    role="img"
                    aria-label="RSI Strategy visualization showing overbought zone above 70 and oversold zone below 30"
                  >
                    <defs>
                      <pattern
                        id="grid"
                        width="20"
                        height="20"
                        patternUnits="userSpaceOnUse"
                      >
                        <path
                          d="M 20 0 L 0 0 0 20"
                          fill="none"
                          stroke="currentColor"
                          strokeWidth="0.5"
                          opacity="0.2"
                        />
                      </pattern>
                    </defs>
                    <rect width="400" height="200" fill="url(#grid)" />

                    {/* RSI Zone Lines */}
                    <line
                      x1="50"
                      y1="40"
                      x2="350"
                      y2="40"
                      stroke="#ef4444"
                      strokeWidth="2"
                      strokeDasharray="5,5"
                    />
                    <text x="355" y="45" fontSize="10" fill="#ef4444">
                      70 - Overbought
                    </text>

                    <line
                      x1="50"
                      y1="100"
                      x2="350"
                      y2="100"
                      stroke="#6b7280"
                      strokeWidth="1"
                      strokeDasharray="3,3"
                    />
                    <text x="355" y="105" fontSize="10" fill="#6b7280">
                      50 - Midline
                    </text>

                    <line
                      x1="50"
                      y1="160"
                      x2="350"
                      y2="160"
                      stroke="#22c55e"
                      strokeWidth="2"
                      strokeDasharray="5,5"
                    />
                    <text x="355" y="165" fontSize="10" fill="#22c55e">
                      30 - Oversold
                    </text>

                    {/* RSI Line */}
                    <path
                      d="M 50 120 Q 80 140 100 160 Q 120 170 140 165 Q 160 155 180 130 Q 200 90 220 70 Q 240 50 260 45 Q 280 50 300 80 Q 320 120 340 140 Q 350 145 350 145"
                      fill="none"
                      stroke="#3b82f6"
                      strokeWidth="3"
                    />

                    {/* Buy/Sell Signals */}
                    <circle cx="140" cy="165" r="4" fill="#22c55e" />
                    <text x="125" y="185" fontSize="9" fill="#22c55e">
                      BUY
                    </text>

                    <circle cx="260" cy="45" r="4" fill="#ef4444" />
                    <text x="245" y="25" fontSize="9" fill="#ef4444">
                      SELL
                    </text>

                    {/* Y-axis labels */}
                    <text x="40" y="45" fontSize="10" fill="currentColor">
                      70
                    </text>
                    <text x="40" y="105" fontSize="10" fill="currentColor">
                      50
                    </text>
                    <text x="40" y="165" fontSize="10" fill="currentColor">
                      30
                    </text>
                    <text x="45" y="185" fontSize="10" fill="currentColor">
                      0
                    </text>

                    {/* Title */}
                    <text
                      x="200"
                      y="15"
                      fontSize="12"
                      fill="currentColor"
                      textAnchor="middle"
                      fontWeight="bold"
                    >
                      RSI Indicator (14)
                    </text>
                  </svg>
                </div>
                <div className="text-sm text-muted-foreground">
                  <p>
                    🟢 <strong>Signal mua:</strong> RSI dưới 30 (oversold) và
                    bắt đầu tăng
                  </p>
                  <p>
                    🔴 <strong>Signal bán:</strong> RSI trên 70 (overbought) và
                    bắt đầu giảm
                  </p>
                </div>
              </div>
            )}

            {/* MACD Strategy Chart */}
            {strategyName === "MACD Strategy" && (
              <div className="space-y-4">
                <div className="w-full h-64 bg-muted/10 rounded-lg p-4">
                  <svg
                    viewBox="0 0 400 200"
                    className="w-full h-full"
                    role="img"
                    aria-label="MACD Strategy visualization showing MACD line crossing signal line for buy and sell signals"
                  >
                    {/* Grid */}
                    <rect width="400" height="200" fill="url(#grid)" />

                    {/* Zero line */}
                    <line
                      x1="50"
                      y1="100"
                      x2="350"
                      y2="100"
                      stroke="#6b7280"
                      strokeWidth="1"
                      strokeDasharray="3,3"
                    />
                    <text x="355" y="105" fontSize="10" fill="#6b7280">
                      0
                    </text>

                    {/* MACD Line */}
                    <path
                      d="M 50 110 Q 70 120 90 115 Q 110 105 130 95 Q 150 85 170 75 Q 190 70 210 75 Q 230 85 250 95 Q 270 110 290 125 Q 310 135 330 140 Q 350 142 350 142"
                      fill="none"
                      stroke="#3b82f6"
                      strokeWidth="2"
                    />

                    {/* Signal Line */}
                    <path
                      d="M 50 115 Q 70 118 90 112 Q 110 108 130 98 Q 150 88 170 80 Q 190 75 210 80 Q 230 90 250 100 Q 270 115 290 128 Q 310 138 330 142 Q 350 144 350 144"
                      fill="none"
                      stroke="#f59e0b"
                      strokeWidth="2"
                      strokeDasharray="4,2"
                    />

                    {/* Histogram bars */}
                    {[
                      60, 80, 100, 120, 140, 160, 180, 200, 220, 240, 260, 280,
                      300, 320, 340,
                    ].map((x, i) => {
                      const height = Math.sin(i * 0.5) * 15 + 10;
                      const isPositive = height > 5;
                      return (
                        <rect
                          key={x}
                          x={x - 2}
                          y={isPositive ? 100 - height : 100}
                          width="4"
                          height={Math.abs(height)}
                          fill={isPositive ? "#22c55e" : "#ef4444"}
                          opacity="0.7"
                        />
                      );
                    })}

                    {/* Crossover signals */}
                    <circle cx="130" cy="95" r="4" fill="#22c55e" />
                    <text x="115" y="85" fontSize="9" fill="#22c55e">
                      BUY
                    </text>

                    <circle cx="250" cy="95" r="4" fill="#ef4444" />
                    <text x="235" y="85" fontSize="9" fill="#ef4444">
                      SELL
                    </text>

                    {/* Legend */}
                    <line
                      x1="60"
                      y1="20"
                      x2="80"
                      y2="20"
                      stroke="#3b82f6"
                      strokeWidth="2"
                    />
                    <text x="85" y="25" fontSize="10" fill="currentColor">
                      MACD Line
                    </text>

                    <line
                      x1="150"
                      y1="20"
                      x2="170"
                      y2="20"
                      stroke="#f59e0b"
                      strokeWidth="2"
                      strokeDasharray="4,2"
                    />
                    <text x="175" y="25" fontSize="10" fill="currentColor">
                      Signal Line
                    </text>

                    <rect
                      x="250"
                      y="17"
                      width="8"
                      height="6"
                      fill="#22c55e"
                      opacity="0.7"
                    />
                    <text x="265" y="25" fontSize="10" fill="currentColor">
                      Histogram
                    </text>

                    {/* Title */}
                    <text
                      x="200"
                      y="15"
                      fontSize="12"
                      fill="currentColor"
                      textAnchor="middle"
                      fontWeight="bold"
                    >
                      MACD (12,26,9)
                    </text>
                  </svg>
                </div>
                <div className="text-sm text-muted-foreground">
                  <p>
                    🟢 <strong>Signal mua:</strong> MACD line cắt lên trên
                    Signal line
                  </p>
                  <p>
                    🔴 <strong>Signal bán:</strong> MACD line cắt xuống dưới
                    Signal line
                  </p>
                </div>
              </div>
            )}

            {/* Volume Strategy Chart */}
            {strategyName === "Volume Strategy" && (
              <div className="space-y-4">
                <div className="w-full h-64 bg-muted/10 rounded-lg p-4">
                  <svg
                    viewBox="0 0 400 200"
                    className="w-full h-full"
                    role="img"
                    aria-label="Volume Strategy visualization showing price movement with volume bars indicating high and normal trading volume"
                  >
                    <rect width="400" height="200" fill="url(#grid)" />

                    {/* Price line (top half) */}
                    <path
                      d="M 50 60 Q 70 55 90 50 Q 110 45 130 40 Q 150 38 170 42 Q 190 48 210 45 Q 230 42 250 38 Q 270 35 290 40 Q 310 45 330 50 Q 350 55 350 55"
                      fill="none"
                      stroke="#3b82f6"
                      strokeWidth="2"
                    />

                    {/* Volume bars (bottom half) */}
                    {[
                      60, 75, 90, 105, 120, 135, 150, 165, 180, 195, 210, 225,
                      240, 255, 270, 285, 300, 315, 330, 345,
                    ].map((x, i) => {
                      const height = Math.random() * 40 + 10;
                      const isHighVolume = height > 30;
                      return (
                        <rect
                          key={x}
                          x={x - 3}
                          y={190 - height}
                          width="6"
                          height={height}
                          fill={isHighVolume ? "#f59e0b" : "#6b7280"}
                          opacity="0.8"
                        />
                      );
                    })}

                    {/* Volume average line */}
                    <line
                      x1="50"
                      y1="170"
                      x2="350"
                      y2="170"
                      stroke="#ef4444"
                      strokeWidth="1"
                      strokeDasharray="3,3"
                    />
                    <text x="355" y="175" fontSize="10" fill="#ef4444">
                      Avg Volume
                    </text>

                    {/* Breakout signals */}
                    <circle cx="170" cy="42" r="4" fill="#22c55e" />
                    <text x="155" y="30" fontSize="9" fill="#22c55e">
                      BREAKOUT
                    </text>

                    <circle cx="250" cy="38" r="4" fill="#ef4444" />
                    <text x="235" y="30" fontSize="9" fill="#ef4444">
                      BREAKDOWN
                    </text>

                    {/* Split line */}
                    <line
                      x1="50"
                      y1="120"
                      x2="350"
                      y2="120"
                      stroke="#6b7280"
                      strokeWidth="1"
                      opacity="0.5"
                    />

                    {/* Labels */}
                    <text x="25" y="60" fontSize="10" fill="currentColor">
                      Price
                    </text>
                    <text x="25" y="180" fontSize="10" fill="currentColor">
                      Volume
                    </text>

                    {/* Title */}
                    <text
                      x="200"
                      y="15"
                      fontSize="12"
                      fill="currentColor"
                      textAnchor="middle"
                      fontWeight="bold"
                    >
                      Price & Volume Analysis
                    </text>
                  </svg>
                </div>
                <div className="text-sm text-muted-foreground">
                  <p>
                    🟢 <strong>Signal mua:</strong> Giá tăng + Volume cao (trên
                    average)
                  </p>
                  <p>
                    🔴 <strong>Signal bán:</strong> Giá giảm + Volume cao
                    (confirmation)
                  </p>
                </div>
              </div>
            )}

            {/* Bollinger Bands Strategy Chart */}
            {strategyName === "Bollinger Bands Strategy" && (
              <div className="space-y-4">
                <div className="w-full h-64 bg-muted/10 rounded-lg p-4">
                  <svg
                    viewBox="0 0 400 200"
                    className="w-full h-full"
                    role="img"
                    aria-label="Bollinger Bands Strategy visualization showing upper band, middle band, and lower band with price movements"
                  >
                    <rect width="400" height="200" fill="url(#grid)" />

                    {/* Upper Band */}
                    <path
                      d="M 50 40 Q 70 35 90 32 Q 110 30 130 28 Q 150 26 170 30 Q 190 35 210 32 Q 230 28 250 25 Q 270 22 290 25 Q 310 30 330 35 Q 350 40 350 40"
                      fill="none"
                      stroke="#ef4444"
                      strokeWidth="2"
                      opacity="0.8"
                    />

                    {/* Middle Band (SMA) */}
                    <path
                      d="M 50 70 Q 70 65 90 62 Q 110 60 130 58 Q 150 56 170 60 Q 190 65 210 62 Q 230 58 250 55 Q 270 52 290 55 Q 310 60 330 65 Q 350 70 350 70"
                      fill="none"
                      stroke="#3b82f6"
                      strokeWidth="2"
                    />

                    {/* Lower Band */}
                    <path
                      d="M 50 100 Q 70 95 90 92 Q 110 90 130 88 Q 150 86 170 90 Q 190 95 210 92 Q 230 88 250 85 Q 270 82 290 85 Q 310 90 330 95 Q 350 100 350 100"
                      fill="none"
                      stroke="#22c55e"
                      strokeWidth="2"
                      opacity="0.8"
                    />

                    {/* Price action */}
                    <path
                      d="M 50 75 Q 70 80 90 85 Q 110 88 130 85 Q 150 80 170 75 Q 190 70 210 65 Q 230 60 250 55 Q 270 50 290 45 Q 310 50 330 60 Q 350 70 350 70"
                      fill="none"
                      stroke="#f59e0b"
                      strokeWidth="3"
                    />

                    {/* Band fill areas */}
                    <path
                      d="M 50 40 Q 70 35 90 32 Q 110 30 130 28 Q 150 26 170 30 Q 190 35 210 32 Q 230 28 250 25 Q 270 22 290 25 Q 310 30 330 35 Q 350 40 350 40 L 350 100 Q 330 95 310 90 Q 290 85 270 82 Q 250 85 230 88 Q 210 92 190 95 Q 170 90 150 86 Q 130 88 110 90 Q 90 92 70 95 Q 50 100 50 100 Z"
                      fill="#3b82f6"
                      opacity="0.1"
                    />

                    {/* Buy/Sell signals */}
                    <circle cx="130" cy="85" r="4" fill="#22c55e" />
                    <text x="115" y="75" fontSize="9" fill="#22c55e">
                      BUY
                    </text>

                    <circle cx="290" cy="45" r="4" fill="#ef4444" />
                    <text x="275" y="35" fontSize="9" fill="#ef4444">
                      SELL
                    </text>

                    {/* Legend */}
                    <line
                      x1="60"
                      y1="20"
                      x2="80"
                      y2="20"
                      stroke="#ef4444"
                      strokeWidth="2"
                    />
                    <text x="85" y="25" fontSize="10" fill="currentColor">
                      Upper Band (+2σ)
                    </text>

                    <line
                      x1="180"
                      y1="20"
                      x2="200"
                      y2="20"
                      stroke="#3b82f6"
                      strokeWidth="2"
                    />
                    <text x="205" y="25" fontSize="10" fill="currentColor">
                      SMA 20
                    </text>

                    <line
                      x1="270"
                      y1="20"
                      x2="290"
                      y2="20"
                      stroke="#22c55e"
                      strokeWidth="2"
                    />
                    <text x="295" y="25" fontSize="10" fill="currentColor">
                      Lower Band (-2σ)
                    </text>

                    {/* Title */}
                    <text
                      x="200"
                      y="15"
                      fontSize="12"
                      fill="currentColor"
                      textAnchor="middle"
                      fontWeight="bold"
                    >
                      Bollinger Bands (20,2)
                    </text>
                  </svg>
                </div>
                <div className="text-sm text-muted-foreground">
                  <p>
                    🟢 <strong>Signal mua:</strong> Giá chạm Lower Band và
                    bounce back
                  </p>
                  <p>
                    🔴 <strong>Signal bán:</strong> Giá chạm Upper Band và bị
                    reject
                  </p>
                </div>
              </div>
            )}

            {/* Additional explanations */}
            <div className="mt-4 p-4 bg-muted/20 rounded-lg">
              <h4 className="font-medium mb-2">💡 Giải thích biểu đồ:</h4>
              <div className="text-sm text-muted-foreground space-y-1">
                {strategyName === "RSI Strategy" && (
                  <>
                    <p>
                      • <strong>Đường xanh:</strong> RSI line dao động từ 0-100
                    </p>
                    <p>
                      • <strong>Vùng đỏ (70+):</strong> Overbought - có thể bán
                    </p>
                    <p>
                      • <strong>Vùng xanh (30-):</strong> Oversold - có thể mua
                    </p>
                    <p>
                      • <strong>Chấm tròn:</strong> Entry/Exit signals
                    </p>
                  </>
                )}

                {strategyName === "MACD Strategy" && (
                  <>
                    <p>
                      • <strong>Đường xanh đậm:</strong> MACD line (EMA12 -
                      EMA26)
                    </p>
                    <p>
                      • <strong>Đường vàng chấm:</strong> Signal line (EMA9 của
                      MACD)
                    </p>
                    <p>
                      • <strong>Histogram:</strong> MACD - Signal (xanh =
                      bullish, đỏ = bearish)
                    </p>
                    <p>
                      • <strong>Crossover:</strong> Điểm cắt nhau tạo signal
                    </p>
                  </>
                )}

                {strategyName === "Volume Strategy" && (
                  <>
                    <p>
                      • <strong>Đường xanh trên:</strong> Price movement
                    </p>
                    <p>
                      • <strong>Cột xám/vàng dưới:</strong> Volume bars
                    </p>
                    <p>
                      • <strong>Đường đỏ chấm:</strong> Average volume
                    </p>
                    <p>
                      • <strong>Volume cao + price breakout:</strong> Signal
                      mạnh
                    </p>
                  </>
                )}

                {strategyName === "Bollinger Bands Strategy" && (
                  <>
                    <p>
                      • <strong>Đường xanh giữa:</strong> SMA 20 (Middle Band)
                    </p>
                    <p>
                      • <strong>Đường đỏ trên:</strong> Upper Band (+2 standard
                      deviation)
                    </p>
                    <p>
                      • <strong>Đường xanh dưới:</strong> Lower Band (-2
                      standard deviation)
                    </p>
                    <p>
                      • <strong>Đường vàng:</strong> Price action trong kênh
                    </p>
                  </>
                )}
              </div>
            </div>
          </CardContent>
        </Card>
      </div>
    </DialogContent>
  );
}

// Types for signals
interface CombinedSignal {
  signal: string;
  confidence: number;
  timestamp: string | number;
  symbol?: string;
  reasoning?: string;
  strategy_scores?: Record<string, number>;
  market_analysis?: AIMarketAnalysis;
  risk_assessment?: AIRiskAssessment;
  source: string;
  model_type?: string;
}

interface FormattedSignal {
  id: string;
  signal: "LONG" | "SHORT" | "NEUTRAL";
  confidence: number;
  timestamp: string;
  pair: string;
  reason: string;
  active: boolean;
  marketAnalysis?: CombinedSignal["market_analysis"];
  riskAssessment?: CombinedSignal["risk_assessment"];
  strategyScores?: Record<string, number>;
  source: string;
  isWebSocket: boolean;
}

// Component for detailed signal analysis popup
function DetailedSignalDialog({ signal }: { signal: FormattedSignal }) {
  return (
    <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle className="flex items-center gap-2">
          <BarChart3 className="h-5 w-5" />
          Detailed AI Analysis: {signal.pair}
          <Badge
            className={
              signal.signal === "LONG"
                ? "bg-profit"
                : signal.signal === "SHORT"
                ? "bg-loss"
                : "bg-warning"
            }
          >
            {signal.signal}
          </Badge>
        </DialogTitle>
      </DialogHeader>

      <div className="space-y-6">
        {/* Signal Overview */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-2">
                <Target className="h-4 w-4 text-blue-500" />
                <span className="font-medium">Signal Strength</span>
              </div>
              <div className="text-2xl font-bold text-blue-500">
                {(signal.confidence * 100).toFixed(1)}%
              </div>
              <div className="text-sm text-muted-foreground">
                Confidence Level
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-2">
                {signal.signal === "LONG" ? (
                  <ArrowUp className="h-4 w-4 text-profit" />
                ) : signal.signal === "SHORT" ? (
                  <ArrowDown className="h-4 w-4 text-loss" />
                ) : (
                  <Activity className="h-4 w-4 text-warning" />
                )}
                <span className="font-medium">Recommendation</span>
              </div>
              <div
                className={`text-2xl font-bold ${
                  signal.signal === "LONG"
                    ? "text-profit"
                    : signal.signal === "SHORT"
                    ? "text-loss"
                    : "text-warning"
                }`}
              >
                {signal.signal === "LONG"
                  ? "BUY (LONG)"
                  : signal.signal === "SHORT"
                  ? "SELL (SHORT)"
                  : "HOLD"}
              </div>
              <div className="text-sm text-muted-foreground">
                {signal.signal === "LONG"
                  ? "🟢 Go Long - Buy Position"
                  : signal.signal === "SHORT"
                  ? "🔴 Go Short - Sell Position"
                  : "🟡 Wait - No Action"}
              </div>
              <div className="mt-2 p-2 rounded-md bg-muted/50">
                <div className="text-xs font-medium text-muted-foreground mb-1">
                  Action:
                </div>
                <div className="text-sm">
                  {signal.signal === "LONG"
                    ? "📈 Mua vào - Giá có thể tăng"
                    : signal.signal === "SHORT"
                    ? "📉 Bán ra - Giá có thể giảm"
                    : "⏸️ Chờ đợi - Thị trường chưa rõ"}
                </div>
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-2">
                <Shield className="h-4 w-4 text-orange-500" />
                <span className="font-medium">Risk Level</span>
              </div>
              <div className="text-2xl font-bold text-orange-500">
                {signal.riskAssessment?.overall_risk || "Medium"}
              </div>
              <div className="text-sm text-muted-foreground">Overall Risk</div>
            </CardContent>
          </Card>
        </div>

        {/* Market Analysis */}
        {signal.marketAnalysis && (
          <Card>
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <TrendingUp className="h-5 w-5" />
                Market Analysis
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Trend Direction:
                  </span>
                  <p className="text-lg font-semibold">
                    {signal.marketAnalysis.trend_direction || "Sideways"}
                  </p>
                </div>
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Trend Strength:
                  </span>
                  <p className="text-lg font-semibold">
                    {(
                      (signal.marketAnalysis.trend_strength || 0.5) * 100
                    ).toFixed(1)}
                    %
                  </p>
                </div>
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Volatility:
                  </span>
                  <p className="text-lg font-semibold">
                    {signal.marketAnalysis.volatility_level || "Medium"}
                  </p>
                </div>
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Volume Analysis:
                  </span>
                  <p className="text-lg font-semibold">
                    {signal.marketAnalysis.volume_analysis || "Normal"}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Strategy Scores - Now Clickable */}
        {signal.strategyScores &&
          Object.keys(signal.strategyScores).length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-lg flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  Strategy Analysis
                  <Badge variant="outline" className="text-xs">
                    Click để xem chi tiết
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {Object.entries(signal.strategyScores).map(
                    ([strategy, score]) => (
                      <Dialog key={strategy}>
                        <DialogTrigger asChild>
                          <div className="space-y-2 p-3 rounded-lg hover:bg-muted/50 cursor-pointer transition-colors border border-transparent hover:border-muted-foreground/20">
                            <div className="flex justify-between items-center">
                              <div className="flex items-center gap-2">
                                <span className="text-sm font-medium">
                                  {strategy}
                                </span>
                                <Info className="h-3 w-3 text-muted-foreground" />
                              </div>
                              <span className="text-sm font-bold">
                                {((score as number) * 100).toFixed(1)}%
                              </span>
                            </div>
                            <div className="w-full bg-muted rounded-full h-2">
                              <div
                                className={`h-2 rounded-full transition-all duration-500 ${
                                  (score as number) >= 0.7
                                    ? "bg-profit"
                                    : (score as number) >= 0.5
                                    ? "bg-warning"
                                    : "bg-loss"
                                }`}
                                style={{ width: `${(score as number) * 100}%` }}
                              />
                            </div>
                            <div className="text-xs text-muted-foreground">
                              Click để xem giải thích chi tiết về {strategy}
                            </div>
                          </div>
                        </DialogTrigger>
                        <StrategyExplanationDialog strategyName={strategy} />
                      </Dialog>
                    )
                  )}
                </div>
              </CardContent>
            </Card>
          )}

        {/* Risk Assessment Details */}
        {signal.riskAssessment && (
          <Card>
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <Shield className="h-5 w-5" />
                Risk Assessment
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-4">
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Technical Risk:
                    </span>
                    <p className="text-lg font-semibold">
                      {(
                        (signal.riskAssessment.technical_risk || 0.5) * 100
                      ).toFixed(1)}
                      %
                    </p>
                  </div>
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Position Size:
                    </span>
                    <p className="text-lg font-semibold">
                      {(
                        (signal.riskAssessment.recommended_position_size ||
                          0.02) * 100
                      ).toFixed(1)}
                      %
                    </p>
                  </div>
                </div>
                <div className="space-y-4">
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Market Risk:
                    </span>
                    <p className="text-lg font-semibold">
                      {(
                        (signal.riskAssessment.market_risk || 0.5) * 100
                      ).toFixed(1)}
                      %
                    </p>
                  </div>
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Source:
                    </span>
                    <p className="text-lg font-semibold capitalize">
                      {signal.source}
                    </p>
                  </div>
                </div>
              </div>

              {(signal.riskAssessment.stop_loss_suggestion ||
                signal.riskAssessment.take_profit_suggestion) && (
                <div className="pt-4 border-t">
                  <h4 className="font-medium mb-2">Trading Levels:</h4>
                  <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                    {signal.riskAssessment.stop_loss_suggestion && (
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">
                          Stop Loss:
                        </span>
                        <p className="text-lg font-semibold text-loss">
                          $
                          {signal.riskAssessment.stop_loss_suggestion.toFixed(
                            2
                          )}
                        </p>
                      </div>
                    )}
                    {signal.riskAssessment.take_profit_suggestion && (
                      <div>
                        <span className="text-sm font-medium text-muted-foreground">
                          Take Profit:
                        </span>
                        <p className="text-lg font-semibold text-profit">
                          $
                          {signal.riskAssessment.take_profit_suggestion.toFixed(
                            2
                          )}
                        </p>
                      </div>
                    )}
                  </div>
                </div>
              )}
            </CardContent>
          </Card>
        )}

        {/* Analysis Reasoning */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <Info className="h-5 w-5" />
              Analysis Reasoning
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm leading-relaxed">{signal.reason}</p>
            <div className="mt-4 pt-4 border-t text-xs text-muted-foreground">
              <p>Generated: {signal.timestamp}</p>
              <p>Status: {signal.active ? "Active" : "Expired"}</p>
            </div>
          </CardContent>
        </Card>
      </div>
    </DialogContent>
  );
}

export function AISignals() {
  const { state: aiState, analyzeSymbol, clearError } = useAIAnalysis();
  const { state: wsState } = useWebSocket();

  // Combine signals from both AI analysis and WebSocket
  const allSignalsRaw = [
    ...aiState.signals.map((s) => ({ ...s, source: "api" })),
    ...wsState.aiSignals.map((s) => ({
      ...s,
      source: "websocket",
      reasoning: `WebSocket signal from ${s.model_type}`,
      strategy_scores: {},
      market_analysis: {
        trend_direction:
          s.signal === "long"
            ? "Bullish"
            : s.signal === "short"
            ? "Bearish"
            : "Sideways",
        trend_strength: s.confidence,
        support_levels: [],
        resistance_levels: [],
        volatility_level: "Medium",
        volume_analysis: "Real-time analysis",
      },
      risk_assessment: {
        overall_risk: "Medium",
        technical_risk: 0.5,
        market_risk: 0.5,
        recommended_position_size: 0.02,
        stop_loss_suggestion: null,
        take_profit_suggestion: null,
      },
    })),
  ];

  // Normalize and sort signals by timestamp (newest first)
  const normalizedSignals = allSignalsRaw
    .map((signal) => {
      // Safely convert timestamp to Date
      const dateObj = new Date(signal.timestamp);
      const isValidDate = !isNaN(dateObj.getTime());

      return {
        ...signal,
        symbol: (signal.symbol || "unknown").toUpperCase(), // Normalize symbol names
        timestamp:
          typeof signal.timestamp === "string" && isValidDate
            ? signal.timestamp
            : isValidDate
            ? dateObj.toISOString()
            : new Date().toISOString(), // Fallback to current time if invalid
        timestampMs: isValidDate ? dateObj.getTime() : Date.now(),
      };
    })
    .sort((a, b) => b.timestampMs - a.timestampMs); // Sort by timestamp descending (newest first)

  // Filter to show only the most recent signal per token pair
  const uniqueSignalsMap = new Map<string, typeof normalizedSignals[0]>();
  normalizedSignals.forEach((signal) => {
    const symbol = signal.symbol;
    // Only keep the signal if this symbol hasn't been seen yet (since we're sorted by newest first)
    if (!uniqueSignalsMap.has(symbol)) {
      uniqueSignalsMap.set(symbol, signal);
    }
  });

  // Convert back to array and sort again to maintain order
  const allSignals = Array.from(uniqueSignalsMap.values()).sort(
    (a, b) => b.timestampMs - a.timestampMs
  );

  interface CombinedSignal {
    signal: string;
    confidence: number;
    timestamp: string | number;
    symbol?: string;
    reasoning?: string;
    strategy_scores?: Record<string, number>;
    market_analysis?: {
      trend_direction: string;
      trend_strength: number;
      support_levels: number[];
      resistance_levels: number[];
      volatility_level: string;
      volume_analysis: string;
    };
    risk_assessment?: {
      overall_risk: string;
      technical_risk: number;
      market_risk: number;
      recommended_position_size: number;
      stop_loss_suggestion: number | null;
      take_profit_suggestion: number | null;
    };
    source: string;
    model_type?: string;
  }

  const formatSignalForDisplay = (signal: CombinedSignal) => ({
    id: `${signal.symbol}-${signal.timestamp}-${signal.source}`,
    signal: (signal.signal || "NEUTRAL").toUpperCase() as "LONG" | "SHORT" | "NEUTRAL",
    confidence: signal.confidence || 0,
    timestamp: new Date(signal.timestamp).toLocaleString(),
    pair: signal.symbol ? signal.symbol.replace("USDT", "/USDT") : "N/A",
    reason: signal.reasoning || `${signal.source} signal`,
    active: Date.now() - new Date(signal.timestamp).getTime() < 30 * 60 * 1000, // Active if less than 30 minutes old
    marketAnalysis: signal.market_analysis,
    riskAssessment: signal.risk_assessment,
    strategyScores: signal.strategy_scores,
    source: signal.source,
    isWebSocket: signal.source === "websocket",
  });

  const getSignalColor = (signal: string) => {
    switch (signal) {
      case "LONG":
        return "bg-profit text-profit-foreground";
      case "SHORT":
        return "bg-loss text-loss-foreground";
      default:
        return "bg-warning text-warning-foreground";
    }
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return "text-profit";
    if (confidence >= 0.6) return "text-warning";
    return "text-loss";
  };

  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          AI Trading Signals
          <Badge
            variant="outline"
            className="bg-info/10 text-info border-info/20"
          >
            <div className="w-2 h-2 bg-info rounded-full mr-2 animate-pulse"></div>
            Live Analysis
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4 flex-1 overflow-auto">
        {/* WebSocket Connection Status */}
        <div className="flex items-center gap-4 mb-4">
          <div className="flex items-center gap-2">
            {wsState.isConnected ? (
              <Wifi className="h-4 w-4 text-success" />
            ) : (
              <WifiOff className="h-4 w-4 text-destructive" />
            )}
            <span className="text-xs text-muted-foreground">
              WebSocket: {wsState.isConnected ? "Connected" : "Disconnected"}
            </span>
          </div>
        </div>

        {/* Error Display */}
        {(aiState.error || wsState.error) && (
          <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 flex items-center gap-2">
            <AlertCircle className="h-4 w-4 text-destructive" />
            <span className="text-sm text-destructive">
              {aiState.error || wsState.error}
            </span>
            <Button
              variant="outline"
              size="sm"
              onClick={clearError}
              className="ml-auto"
            >
              Dismiss
            </Button>
          </div>
        )}

        {/* Loading State */}
        {aiState.isLoading && (
          <div className="p-4 rounded-lg bg-muted/20 border border-muted/40 flex items-center gap-2">
            <RefreshCw className="h-4 w-4 animate-spin" />
            <span className="text-sm text-muted-foreground">
              Analyzing market signals...
            </span>
          </div>
        )}

        {/* AI Service Info */}
        {aiState.serviceInfo && (
          <div className="p-3 rounded-lg bg-info/10 border border-info/20 flex items-center gap-2">
            <Zap className="h-4 w-4 text-info" />
            <span className="text-sm text-info">
              {aiState.serviceInfo.service_name} v{aiState.serviceInfo.version}{" "}
              • Model: {aiState.serviceInfo.model_version}
            </span>
            {aiState.lastUpdate && (
              <span className="text-xs text-muted-foreground ml-auto">
                Last updated:{" "}
                {new Date(aiState.lastUpdate).toLocaleTimeString()}
              </span>
            )}
          </div>
        )}

        {/* No Signals Message */}
        {!aiState.isLoading && allSignals.length === 0 && (
          <div className="p-8 text-center text-muted-foreground">
            <TrendingUp className="h-8 w-8 mx-auto mb-2 opacity-50" />
            <p>No AI signals available yet</p>
            <p className="text-sm">Analysis will start automatically</p>
          </div>
        )}

        {/* Signals List */}
        {allSignals.map((signalData) => {
          const signal = formatSignalForDisplay(signalData);
          return (
            <Dialog key={signal.id}>
              <DialogTrigger asChild>
                <div
                  className={`p-4 rounded-lg border transition-all duration-200 hover:shadow-lg cursor-pointer ${
                    signal.active
                      ? "bg-secondary/50 border-primary/20 shadow-primary/5 hover:bg-secondary/70"
                      : "bg-muted/20 border-muted/40 hover:bg-muted/30"
                  }`}
                >
                  <div className="flex justify-between items-start mb-3">
                    <div className="flex items-center gap-3">
                      <Badge className={getSignalColor(signal.signal)}>
                        {signal.signal}
                      </Badge>
                      <span className="font-semibold">{signal.pair}</span>
                      {signal.active && (
                        <Badge
                          variant="outline"
                          className="bg-profit/10 text-profit border-profit/20 text-xs"
                        >
                          ACTIVE
                        </Badge>
                      )}
                    </div>
                    <div className="text-right">
                      <div
                        className={`font-bold text-lg ${getConfidenceColor(
                          signal.confidence
                        )}`}
                      >
                        {(signal.confidence * 100).toFixed(0)}%
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Confidence
                      </div>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <p className="text-sm text-foreground">{signal.reason}</p>
                    <div className="flex justify-between items-center text-xs text-muted-foreground">
                      <span>{signal.timestamp}</span>
                      <div className="flex items-center gap-1">
                        <div
                          className={`w-2 h-2 rounded-full ${
                            signal.confidence >= 0.8
                              ? "bg-profit"
                              : signal.confidence >= 0.6
                              ? "bg-warning"
                              : "bg-loss"
                          }`}
                        ></div>
                        <span>AI Confidence</span>
                      </div>
                    </div>
                  </div>

                  {/* Confidence Bar */}
                  <div className="mt-3">
                    <div className="w-full bg-muted rounded-full h-1.5">
                      <div
                        className={`h-1.5 rounded-full transition-all duration-500 ${
                          signal.confidence >= 0.8
                            ? "bg-profit"
                            : signal.confidence >= 0.6
                            ? "bg-warning"
                            : "bg-loss"
                        }`}
                        style={{ width: `${signal.confidence * 100}%` }}
                      ></div>
                    </div>
                  </div>
                </div>
              </DialogTrigger>
              <DetailedSignalDialog signal={signal} />
            </Dialog>
          );
        })}
      </CardContent>
    </Card>
  );
}
