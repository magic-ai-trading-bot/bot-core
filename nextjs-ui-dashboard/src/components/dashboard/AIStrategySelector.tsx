import { useState, useEffect, useCallback } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { PremiumButton } from "@/styles/luxury-design-system";
import { Checkbox } from "@/components/ui/checkbox";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { useAIAnalysisContext } from "@/contexts/AIAnalysisContext";
import {
  Settings,
  TrendingUp,
  BarChart3,
  Activity,
  Target,
  Info,
  Loader2,
} from "lucide-react";
import logger from "@/utils/logger";

// API Base URL - using environment variable with fallback
const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

// Fallback symbols only used if ALL API calls fail
const FALLBACK_SYMBOLS = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];

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
  "Stochastic Strategy": {
    name: "Stochastic Strategy",
    description:
      "Stochastic Oscillator - Xác định momentum và điều kiện quá mua/quá bán",
    how_it_works:
      "So sánh giá đóng cửa với khoảng giá trong N kỳ. %K (fast line) và %D (slow line) dao động 0-100. Crossover tạo tín hiệu mua/bán",
    signals: {
      buy: "%K cắt lên %D trong vùng oversold (<20)",
      sell: "%K cắt xuống %D trong vùng overbought (>80)",
    },
    advantages: [
      "Tín hiệu sớm hơn RSI",
      "Phát hiện divergence tốt",
      "Hiệu quả trong ranging market",
    ],
    disadvantages: [
      "Nhiều false signals trong strong trend",
      "Whipsaw trong choppy market",
      "Cần kết hợp trend filter",
    ],
    best_timeframe: "1h, 4h, 1d",
    chart_description:
      "%K (fast line) và %D (signal line) với vùng oversold/overbought",
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

        {/* Real Chart Illustrations */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg">📈 Biểu đồ minh họa</CardTitle>
          </CardHeader>
          <CardContent>
            {/* RSI Strategy Chart */}
            {strategyName === "RSI Strategy" && (
              <div className="space-y-4">
                <div className="w-full h-64 bg-muted/10 rounded-lg p-4">
                  <svg viewBox="0 0 400 200" className="w-full h-full">
                    {/* Grid lines */}
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
                  <svg viewBox="0 0 400 200" className="w-full h-full">
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
                    ].map((x, _i) => {
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
                  <svg viewBox="0 0 400 200" className="w-full h-full">
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
                    ].map((x, _i) => {
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
                  <svg viewBox="0 0 400 200" className="w-full h-full">
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

const STRATEGY_CONFIGS = {
  "RSI Strategy": {
    icon: TrendingUp,
    description:
      "Relative Strength Index - identifies overbought/oversold conditions",
    color: "bg-blue-500",
    defaultParams: { period: 14, oversold: 30, overbought: 70 },
  },
  "MACD Strategy": {
    icon: BarChart3,
    description:
      "Moving Average Convergence Divergence - trend following momentum",
    color: "bg-green-500",
    defaultParams: { fast: 12, slow: 26, signal: 9 },
  },
  "Volume Strategy": {
    icon: Activity,
    description: "Volume analysis - detects accumulation/distribution patterns",
    color: "bg-purple-500",
    defaultParams: { multiplier: 2, period: 20 },
  },
  "Bollinger Bands Strategy": {
    icon: Target,
    description: "Volatility bands - mean reversion and breakout detection",
    color: "bg-orange-500",
    defaultParams: { period: 20, stdDev: 2 },
  },
  "Stochastic Strategy": {
    icon: Target,
    description: "Stochastic Oscillator - momentum and overbought/oversold detection",
    color: "bg-pink-500",
    defaultParams: { kPeriod: 14, dPeriod: 3, oversold: 20, overbought: 80 },
  },
};

const RISK_LEVELS = [
  {
    value: "Conservative",
    label: "Conservative",
    description: "Lower risk, more stable signals",
  },
  {
    value: "Moderate",
    label: "Moderate",
    description: "Balanced risk-reward approach",
  },
  {
    value: "Aggressive",
    label: "Aggressive",
    description: "Higher risk, more frequent signals",
  },
];

export function AIStrategySelector() {
  const { state, analyzeSymbol } = useAIAnalysisContext();
  const [selectedStrategies, setSelectedStrategies] = useState<string[]>([
    "RSI Strategy",
    "MACD Strategy",
  ]);
  const [riskLevel, setRiskLevel] = useState("Moderate");
  // Dynamic symbols - initialized with empty, will be set after API call
  const [availableSymbols, setAvailableSymbols] = useState<string[]>([]);
  const [selectedSymbol, setSelectedSymbol] = useState("");
  const [isLoadingSymbols, setIsLoadingSymbols] = useState(true);
  const [openDialog, setOpenDialog] = useState<string | null>(null);

  // Fetch symbols dynamically from API
  const fetchSymbols = useCallback(async () => {
    setIsLoadingSymbols(true);
    try {
      const response = await fetch(`${API_BASE}/api/market/symbols`);
      const data = await response.json();
      // FIX: API returns {success: true, data: {symbols: [...]}} - access data.data.symbols
      if (data.success && data.data && data.data.symbols && data.data.symbols.length > 0) {
        const symbols = data.data.symbols;
        setAvailableSymbols(symbols);
        setSelectedSymbol(symbols[0]); // Set first symbol as default
        logger.info(`Loaded ${symbols.length} symbols for AI strategy selector`);
      } else {
        initializeFallbackSymbols();
      }
    } catch (error) {
      logger.error("Failed to fetch symbols:", error);
      initializeFallbackSymbols();
    } finally {
      setIsLoadingSymbols(false);
    }
  }, []);

  // Initialize with fallback symbols
  const initializeFallbackSymbols = useCallback(() => {
    setAvailableSymbols(FALLBACK_SYMBOLS);
    setSelectedSymbol(FALLBACK_SYMBOLS[0]);
    logger.warn("Using fallback symbols for AI strategy selector");
  }, []);

  // Load symbols on mount
  useEffect(() => {
    fetchSymbols();
  }, [fetchSymbols]);

  const handleStrategyToggle = (strategy: string) => {
    setSelectedStrategies((prev) =>
      prev.includes(strategy)
        ? prev.filter((s) => s !== strategy)
        : [...prev, strategy]
    );
  };

  const handleAnalyze = () => {
    if (selectedStrategies.length > 0) {
      analyzeSymbol(selectedSymbol, selectedStrategies);
    }
  };

  return (
    <Card className="h-full flex flex-col">
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          <Settings className="h-5 w-5" />
          AI Strategy Configuration
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6 flex-1">
        {/* Symbol Selection */}
        <div className="space-y-2">
          <label className="text-sm font-medium">Trading Symbol</label>
          <Select value={selectedSymbol} onValueChange={setSelectedSymbol}>
            <SelectTrigger>
              <SelectValue placeholder="Select trading pair" />
            </SelectTrigger>
            <SelectContent>
              {isLoadingSymbols ? (
                <div className="flex items-center justify-center p-2">
                  <Loader2 className="h-4 w-4 animate-spin mr-2" />
                  <span className="text-sm text-muted-foreground">Loading...</span>
                </div>
              ) : availableSymbols.length === 0 ? (
                <div className="text-sm text-muted-foreground text-center p-2">
                  No symbols available
                </div>
              ) : (
                availableSymbols.map((symbol) => (
                  <SelectItem key={symbol} value={symbol}>
                    {symbol.replace('USDT', '/USDT')}
                  </SelectItem>
                ))
              )}
            </SelectContent>
          </Select>
        </div>

        {/* Risk Level Selection */}
        <div className="space-y-2">
          <label className="text-sm font-medium">Risk Level</label>
          <Select value={riskLevel} onValueChange={setRiskLevel}>
            <SelectTrigger>
              <SelectValue placeholder="Select risk level" />
            </SelectTrigger>
            <SelectContent>
              {RISK_LEVELS.map((level) => (
                <SelectItem key={level.value} value={level.value}>
                  <div className="flex flex-col">
                    <span>{level.label}</span>
                    <span className="text-xs text-muted-foreground">
                      {level.description}
                    </span>
                  </div>
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
        </div>

        {/* Strategy Selection */}
        <div className="space-y-3">
          <div className="flex items-center justify-between">
            <label className="text-sm font-medium">Active Strategies</label>
            <Badge variant="outline" className="text-xs">
              Click tên strategy để xem chi tiết
            </Badge>
          </div>
          <div className="grid grid-cols-1 gap-3">
            {Object.entries(STRATEGY_CONFIGS).map(([strategy, config]) => {
              const Icon = config.icon;
              const isSelected = selectedStrategies.includes(strategy);

              return (
                <div
                  key={strategy}
                  className={`p-3 rounded-lg border transition-all ${
                    isSelected
                      ? "border-primary bg-primary/5"
                      : "border-muted hover:border-muted-foreground/50"
                  }`}
                >
                  <div className="flex items-start gap-3">
                    {/* Checkbox - separate click area */}
                    <div
                      className="flex items-center cursor-pointer"
                      onClick={(e) => {
                        e.stopPropagation();
                        handleStrategyToggle(strategy);
                      }}
                    >
                      <Checkbox
                        checked={isSelected}
                        onChange={() => handleStrategyToggle(strategy)}
                      />
                    </div>

                    {/* Strategy Info - clickable for dialog */}
                    <div className="flex-1 min-w-0">
                      <Dialog
                        open={openDialog === strategy}
                        onOpenChange={(open) =>
                          setOpenDialog(open ? strategy : null)
                        }
                      >
                        <DialogTrigger asChild>
                          <div className="cursor-pointer hover:bg-muted/30 p-2 -m-2 rounded transition-colors">
                            <div className="flex items-center gap-2 mb-1">
                              <div
                                className={`w-3 h-3 rounded-full ${config.color}`}
                              />
                              <Icon className="h-4 w-4" />
                              <span className="font-medium text-sm">
                                {strategy}
                              </span>
                              <Info className="h-3 w-3 text-muted-foreground ml-auto" />
                            </div>
                            <p className="text-xs text-muted-foreground">
                              {config.description}
                            </p>
                            <p className="text-xs text-blue-500 mt-1">
                              Click để xem giải thích chi tiết
                            </p>
                          </div>
                        </DialogTrigger>
                        <StrategyExplanationDialog strategyName={strategy} />
                      </Dialog>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        </div>

        {/* Strategy Recommendations */}
        {state.strategies.length > 0 && (
          <div className="space-y-2">
            <label className="text-sm font-medium">AI Recommendations</label>
            <div className="space-y-2">
              {state.strategies.slice(0, 3).map((rec, index) => (
                <div
                  key={index}
                  className="p-2 rounded-lg bg-muted/20 border border-muted/40"
                >
                  <div className="flex items-center justify-between">
                    <span className="text-sm font-medium">
                      {rec.strategy_name}
                    </span>
                    <Badge variant="outline" className="text-xs">
                      {(rec.suitability_score * 100).toFixed(0)}% match
                    </Badge>
                  </div>
                  <p className="text-xs text-muted-foreground mt-1">
                    {rec.reasoning}
                  </p>
                </div>
              ))}
            </div>
          </div>
        )}

        {/* Analysis Button */}
        <PremiumButton
          onClick={handleAnalyze}
          disabled={selectedStrategies.length === 0 || state.isLoading}
          className="w-full"
        >
          {state.isLoading
            ? "Analyzing..."
            : "Analyze with Selected Strategies"}
        </PremiumButton>

        {/* Selected Strategies Summary */}
        {selectedStrategies.length > 0 && (
          <div className="pt-2 border-t">
            <div className="flex flex-wrap gap-1">
              {selectedStrategies.map((strategy) => (
                <Badge key={strategy} variant="secondary" className="text-xs">
                  {strategy}
                </Badge>
              ))}
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
