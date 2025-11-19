import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { DollarSign } from "lucide-react";
import { Portfolio, Trade } from "./types";

interface RiskMetricsProps {
  portfolio: Portfolio;
  openTrades: Trade[];
  closedTrades: Trade[];
  calculateMarginRequired: (trade: Trade) => number;
  formatCurrency: (value: number) => string;
  formatPercentage: (value: number | undefined) => string;
}

export function RiskMetrics({
  portfolio,
  openTrades,
  closedTrades,
  calculateMarginRequired,
  formatCurrency,
  formatPercentage,
}: RiskMetricsProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">
            Margin sử dụng
          </CardTitle>
          <DollarSign className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-lg font-bold text-warning">
            {formatCurrency(portfolio.margin_used)}
          </div>
          <div className="text-xs text-muted-foreground space-y-1">
            <div>Free: {formatCurrency(portfolio.free_margin)}</div>
            <div>
              Usage:{" "}
              {(
                (portfolio.margin_used / portfolio.equity) *
                100
              ).toFixed(1)}
              %
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">
            {closedTrades.length > 0 ? "Lợi nhuận TB" : "Avg Margin"}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {closedTrades.length > 0 ? (
            <div className="text-lg font-bold text-profit">
              {formatCurrency(portfolio.average_win)}
            </div>
          ) : (
            <div className="text-lg font-bold text-primary">
              {openTrades.length > 0
                ? formatCurrency(
                    openTrades.reduce(
                      (total, trade) =>
                        total + calculateMarginRequired(trade),
                      0
                    ) / openTrades.length
                  )
                : "$0.00"}
            </div>
          )}
          <p className="text-xs text-muted-foreground">
            {closedTrades.length > 0
              ? "Trung bình thắng"
              : "Margin trung bình"}
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">
            {closedTrades.length > 0 ? "Profit Factor" : "Daily P&L"}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {closedTrades.length > 0 ? (
            <div className="text-lg font-bold">
              {portfolio.profit_factor.toFixed(2)}
            </div>
          ) : (
            <div
              className={`text-lg font-bold ${
                portfolio.total_pnl >= 0 ? "text-profit" : "text-loss"
              }`}
            >
              {portfolio.total_pnl >= 0 ? "+" : ""}
              {formatCurrency(portfolio.total_pnl)}
            </div>
          )}
          <p className="text-xs text-muted-foreground">
            {closedTrades.length > 0
              ? "Tỷ lệ lời/lỗ"
              : "Unrealized P&L"}
          </p>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">
            {closedTrades.length > 0
              ? "Max Drawdown"
              : "Trading Status"}
          </CardTitle>
        </CardHeader>
        <CardContent>
          {closedTrades.length > 0 ? (
            <div className="text-lg font-bold text-loss">
              {formatCurrency(portfolio.max_drawdown)}
            </div>
          ) : (
            <div className="text-lg font-bold text-info">
              {openTrades.length > 0
                ? `${openTrades.length} Active`
                : "No Trades"}
            </div>
          )}
          <p className="text-xs text-muted-foreground">
            {closedTrades.length > 0
              ? formatPercentage(portfolio.max_drawdown_percentage)
              : openTrades.length > 0
              ? "Positions running"
              : "Waiting for signals"}
          </p>
        </CardContent>
      </Card>
    </div>
  );
}
