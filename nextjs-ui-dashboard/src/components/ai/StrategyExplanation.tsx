import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Activity } from "lucide-react";
import { DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";

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

interface StrategyExplanationDialogProps {
  strategyName: string;
}

export function StrategyExplanationDialog({ strategyName }: StrategyExplanationDialogProps) {
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
      </div>
    </DialogContent>
  );
}
