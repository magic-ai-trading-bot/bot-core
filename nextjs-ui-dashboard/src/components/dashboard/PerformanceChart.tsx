import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  AreaChart,
  Area,
} from "recharts";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { TrendingUp, TrendingDown, Activity } from "lucide-react";
import { usePaperTrading } from "@/hooks/usePaperTrading";

interface PerformanceDataPoint {
  date: string;
  equity: number;
  pnl: number;
  dailyPnL: number;
  balance: number;
}

export function PerformanceChart() {
  const { portfolio, openTrades, closedTrades } = usePaperTrading();

  // Generate mock performance data based on current portfolio status
  const generatePerformanceData = (): PerformanceDataPoint[] => {
    const data: PerformanceDataPoint[] = [];
    const currentDate = new Date();
    const initialBalance = 10000;

    // Generate last 30 days of mock data based on current portfolio
    for (let i = 29; i >= 0; i--) {
      const date = new Date(currentDate);
      date.setDate(date.getDate() - i);

      // Create realistic performance curve
      const progress = (29 - i) / 29;
      const totalPnLProgress = portfolio.total_pnl * progress;
      const equity = initialBalance + totalPnLProgress;

      // Add some daily volatility
      const dailyVariation = Math.sin(i * 0.5) * 20 + Math.random() * 40 - 20;
      const adjustedEquity = equity + dailyVariation;

      const dailyPnL =
        i > 0
          ? adjustedEquity - (data[data.length - 1]?.equity || initialBalance)
          : 0;

      data.push({
        date: date.toISOString().split("T")[0],
        equity: Math.max(adjustedEquity, initialBalance - 1000), // Don't go too negative
        pnl: adjustedEquity - initialBalance,
        dailyPnL,
        balance: initialBalance,
      });
    }

    // Ensure last point matches current portfolio
    if (data.length > 0) {
      data[data.length - 1].equity = portfolio.equity;
      data[data.length - 1].pnl = portfolio.total_pnl;
    }

    return data;
  };

  const performanceData = generatePerformanceData();
  const currentPnL = portfolio.total_pnl;
  const currentPnLPercentage = portfolio.total_pnl_percentage;

  // Calculate performance metrics
  const isProfit = currentPnL >= 0;
  const trend =
    performanceData.length >= 2
      ? performanceData[performanceData.length - 1].equity >
        performanceData[performanceData.length - 2].equity
      : false;

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat("vi-VN", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 2,
    }).format(value);
  };

  const formatDate = (dateStr: string) => {
    return new Date(dateStr).toLocaleDateString("vi-VN", {
      month: "short",
      day: "numeric",
    });
  };

  const CustomTooltip = ({
    active,
    payload,
    label,
  }: {
    active?: boolean;
    payload?: Array<{ payload: PerformanceDataPoint }>;
    label?: string;
  }) => {
    if (active && payload && payload.length) {
      const data = payload[0].payload;
      return (
        <div className="bg-background p-3 border rounded-lg shadow-lg">
          <p className="font-medium">{formatDate(label)}</p>
          <p className="text-sm">
            <span className="text-muted-foreground">Equity: </span>
            <span className="font-medium">{formatCurrency(data.equity)}</span>
          </p>
          <p className="text-sm">
            <span className="text-muted-foreground">P&L: </span>
            <span
              className={`font-medium ${
                data.pnl >= 0 ? "text-profit" : "text-loss"
              }`}
            >
              {formatCurrency(data.pnl)}
            </span>
          </p>
          <p className="text-sm">
            <span className="text-muted-foreground">Daily: </span>
            <span
              className={`font-medium ${
                data.dailyPnL >= 0 ? "text-profit" : "text-loss"
              }`}
            >
              {formatCurrency(data.dailyPnL)}
            </span>
          </p>
        </div>
      );
    }
    return null;
  };

  return (
    <Card className="w-full">
      <CardHeader className="pb-4">
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg font-semibold">
            Biểu đồ hiệu suất
          </CardTitle>
          <div className="flex items-center gap-2">
            <Badge
              variant={isProfit ? "default" : "destructive"}
              className={`${
                isProfit
                  ? "bg-profit text-profit-foreground"
                  : "bg-loss text-loss-foreground"
              }`}
            >
              {trend ? (
                <TrendingUp className="w-3 h-3 mr-1" />
              ) : (
                <TrendingDown className="w-3 h-3 mr-1" />
              )}
              {currentPnLPercentage >= 0 ? "+" : ""}
              {currentPnLPercentage.toFixed(2)}%
            </Badge>
            <Badge variant="outline" className="gap-1">
              <Activity className="w-3 h-3" />
              {portfolio.total_trades} trades
            </Badge>
          </div>
        </div>

        <div className="grid grid-cols-3 gap-4 pt-2">
          <div className="text-center">
            <p className="text-sm text-muted-foreground">Tổng P&L</p>
            <p
              className={`font-bold text-lg ${
                isProfit ? "text-profit" : "text-loss"
              }`}
            >
              {formatCurrency(currentPnL)}
            </p>
          </div>
          <div className="text-center">
            <p className="text-sm text-muted-foreground">Equity hiện tại</p>
            <p className="font-bold text-lg">
              {formatCurrency(portfolio.equity)}
            </p>
          </div>
          <div className="text-center">
            <p className="text-sm text-muted-foreground">Win Rate</p>
            <p className="font-bold text-lg text-primary">
              {portfolio.win_rate.toFixed(1)}%
            </p>
          </div>
        </div>
      </CardHeader>

      <CardContent>
        <div className="h-[300px] w-full">
          <ResponsiveContainer width="100%" height="100%">
            <AreaChart
              data={performanceData}
              margin={{ top: 5, right: 30, left: 20, bottom: 5 }}
            >
              <defs>
                <linearGradient id="equityGradient" x1="0" y1="0" x2="0" y2="1">
                  <stop
                    offset="5%"
                    stopColor={
                      isProfit ? "hsl(var(--profit))" : "hsl(var(--loss))"
                    }
                    stopOpacity={0.3}
                  />
                  <stop
                    offset="95%"
                    stopColor={
                      isProfit ? "hsl(var(--profit))" : "hsl(var(--loss))"
                    }
                    stopOpacity={0.0}
                  />
                </linearGradient>
              </defs>
              <CartesianGrid strokeDasharray="3 3" className="stroke-muted" />
              <XAxis
                dataKey="date"
                className="text-xs"
                tickFormatter={formatDate}
                interval="preserveStartEnd"
              />
              <YAxis
                className="text-xs"
                tickFormatter={(value) => formatCurrency(value)}
                domain={["dataMin - 100", "dataMax + 100"]}
              />
              <Tooltip content={<CustomTooltip />} />

              {/* Balance baseline */}
              <Line
                type="monotone"
                dataKey="balance"
                stroke="hsl(var(--muted-foreground))"
                strokeDasharray="2 2"
                strokeWidth={1}
                dot={false}
                strokeOpacity={0.5}
              />

              {/* Equity line */}
              <Area
                type="monotone"
                dataKey="equity"
                stroke={isProfit ? "hsl(var(--profit))" : "hsl(var(--loss))"}
                strokeWidth={2.5}
                fill="url(#equityGradient)"
                dot={false}
              />
            </AreaChart>
          </ResponsiveContainer>
        </div>

        {/* Performance summary */}
        <div className="flex justify-between items-center mt-4 p-4 bg-muted/30 rounded-lg">
          <div className="grid grid-cols-2 gap-4 text-sm">
            <div>
              <span className="text-muted-foreground">Max Drawdown: </span>
              <span className="font-medium text-loss">
                {formatCurrency(portfolio.max_drawdown)}
              </span>
            </div>
            <div>
              <span className="text-muted-foreground">Sharpe Ratio: </span>
              <span className="font-medium">
                {portfolio.sharpe_ratio.toFixed(2)}
              </span>
            </div>
          </div>

          <div className="text-right">
            <p className="text-xs text-muted-foreground">
              Lịch sử 30 ngày • {openTrades.length} lệnh mở •{" "}
              {closedTrades.length} lệnh đóng
            </p>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
