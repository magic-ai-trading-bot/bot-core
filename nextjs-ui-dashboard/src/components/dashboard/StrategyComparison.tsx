import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { TrendingUp, TrendingDown, Activity } from "lucide-react";

interface StrategyPerformance {
  name: string;
  winRate: number;
  avgProfit: number;
  totalTrades: number;
  sharpeRatio: number;
  maxDrawdown: number;
  status: "excellent" | "good" | "neutral" | "poor";
}

const strategyData: StrategyPerformance[] = [
  {
    name: "RSI Strategy",
    winRate: 68.5,
    avgProfit: 2.3,
    totalTrades: 145,
    sharpeRatio: 1.8,
    maxDrawdown: -12.4,
    status: "excellent",
  },
  {
    name: "MACD Strategy",
    winRate: 62.3,
    avgProfit: 1.9,
    totalTrades: 198,
    sharpeRatio: 1.5,
    maxDrawdown: -15.2,
    status: "good",
  },
  {
    name: "Bollinger Bands",
    winRate: 58.7,
    avgProfit: 1.6,
    totalTrades: 167,
    sharpeRatio: 1.2,
    maxDrawdown: -18.5,
    status: "good",
  },
  {
    name: "Volume Strategy",
    winRate: 52.1,
    avgProfit: 1.1,
    totalTrades: 134,
    sharpeRatio: 0.9,
    maxDrawdown: -22.3,
    status: "neutral",
  },
];

export function StrategyComparison() {
  const getStatusColor = (status: string) => {
    switch (status) {
      case "excellent":
        return "bg-profit/20 text-profit border-profit/30";
      case "good":
        return "bg-info/20 text-info border-info/30";
      case "neutral":
        return "bg-warning/20 text-warning border-warning/30";
      default:
        return "bg-loss/20 text-loss border-loss/30";
    }
  };

  const getStatusIcon = (status: string) => {
    switch (status) {
      case "excellent":
        return <TrendingUp className="h-3 w-3" aria-hidden="true" />;
      case "good":
        return <TrendingUp className="h-3 w-3" aria-hidden="true" />;
      case "neutral":
        return <Activity className="h-3 w-3" aria-hidden="true" />;
      default:
        return <TrendingDown className="h-3 w-3" aria-hidden="true" />;
    }
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">📊 Strategy Performance Comparison</CardTitle>
        <p className="text-sm text-muted-foreground">
          Compare trading strategies based on backtesting results
        </p>
      </CardHeader>
      <CardContent>
        <div className="overflow-x-auto">
          <table className="w-full text-sm">
            <thead>
              <tr className="border-b border-border">
                <th className="text-left py-3 px-2 font-semibold">Strategy</th>
                <th className="text-right py-3 px-2 font-semibold">Win Rate</th>
                <th className="text-right py-3 px-2 font-semibold">Avg Profit</th>
                <th className="text-right py-3 px-2 font-semibold">Trades</th>
                <th className="text-right py-3 px-2 font-semibold">Sharpe</th>
                <th className="text-right py-3 px-2 font-semibold">Max DD</th>
                <th className="text-right py-3 px-2 font-semibold">Status</th>
              </tr>
            </thead>
            <tbody>
              {strategyData.map((strategy, index) => (
                <tr
                  key={strategy.name}
                  className={`border-b border-border/50 hover:bg-secondary/30 transition-colors ${
                    index === 0 ? "bg-profit/5" : ""
                  }`}
                >
                  <td className="py-3 px-2">
                    <div className="flex items-center gap-2">
                      {index === 0 && (
                        <span className="text-xs">🏆</span>
                      )}
                      <span className="font-medium">{strategy.name}</span>
                    </div>
                  </td>
                  <td className="text-right py-3 px-2">
                    <div className="flex items-center justify-end gap-1">
                      {strategy.winRate >= 60 ? (
                        <TrendingUp className="h-3 w-3 text-profit" aria-hidden="true" />
                      ) : strategy.winRate >= 55 ? (
                        <Activity className="h-3 w-3 text-warning" aria-hidden="true" />
                      ) : (
                        <TrendingDown className="h-3 w-3 text-loss" aria-hidden="true" />
                      )}
                      <span className={strategy.winRate >= 60 ? "text-profit" : strategy.winRate >= 55 ? "text-warning" : "text-loss"}>
                        {strategy.winRate}%
                      </span>
                    </div>
                  </td>
                  <td className="text-right py-3 px-2">
                    <span className={strategy.avgProfit >= 2 ? "text-profit" : strategy.avgProfit >= 1.5 ? "text-warning" : "text-loss"}>
                      {strategy.avgProfit >= 0 ? "+" : ""}{strategy.avgProfit}%
                    </span>
                  </td>
                  <td className="text-right py-3 px-2 text-muted-foreground">
                    {strategy.totalTrades}
                  </td>
                  <td className="text-right py-3 px-2">
                    <span className={strategy.sharpeRatio >= 1.5 ? "text-profit" : strategy.sharpeRatio >= 1 ? "text-warning" : "text-loss"}>
                      {strategy.sharpeRatio.toFixed(1)}
                    </span>
                  </td>
                  <td className="text-right py-3 px-2">
                    <span className={Math.abs(strategy.maxDrawdown) <= 15 ? "text-profit" : Math.abs(strategy.maxDrawdown) <= 20 ? "text-warning" : "text-loss"}>
                      {strategy.maxDrawdown}%
                    </span>
                  </td>
                  <td className="text-right py-3 px-2">
                    <Badge
                      variant="outline"
                      className={`${getStatusColor(strategy.status)} flex items-center gap-1 w-fit ml-auto`}
                    >
                      {getStatusIcon(strategy.status)}
                      <span className="capitalize text-xs">
                        {strategy.status}
                      </span>
                    </Badge>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>

        {/* Legend */}
        <div className="mt-4 pt-4 border-t border-border">
          <p className="text-xs text-muted-foreground mb-2">
            <strong>Metrics Explained:</strong>
          </p>
          <div className="grid grid-cols-2 gap-2 text-xs text-muted-foreground">
            <div>
              <strong>Win Rate:</strong> % of profitable trades
            </div>
            <div>
              <strong>Avg Profit:</strong> Average profit per trade
            </div>
            <div>
              <strong>Sharpe Ratio:</strong> Risk-adjusted returns (&gt;1.5 = excellent)
            </div>
            <div>
              <strong>Max DD:</strong> Maximum drawdown from peak
            </div>
          </div>
        </div>

        {/* Best Strategy Recommendation */}
        <div className="mt-4 p-3 bg-profit/10 border border-profit/20 rounded-md">
          <div className="flex items-start gap-2">
            <span className="text-lg">🏆</span>
            <div className="text-xs">
              <strong className="text-profit">Recommended:</strong>{" "}
              <span className="text-foreground">
                RSI Strategy has the highest win rate (68.5%) and best Sharpe ratio (1.8).
                Consider combining it with MACD for even better results!
              </span>
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
