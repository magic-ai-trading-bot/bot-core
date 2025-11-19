import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { DollarSign, TrendingUp, Activity, Target } from "lucide-react";
import { Portfolio, Trade } from "./types";

interface PortfolioStatsProps {
  portfolio: Portfolio;
  openTrades: Trade[];
  closedTrades: Trade[];
  wsConnected: boolean;
  calculatePositionSize: (trade: Trade) => number;
  calculateMarginRequired: (trade: Trade) => number;
  formatCurrency: (value: number) => string;
  formatPercentage: (value: number | undefined) => string;
}

export function PortfolioStats({
  portfolio,
  openTrades,
  closedTrades,
  wsConnected,
  calculatePositionSize,
  calculateMarginRequired,
  formatCurrency,
  formatPercentage,
}: PortfolioStatsProps) {
  return (
    <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">
            Số dư hiện tại
          </CardTitle>
          <DollarSign className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {formatCurrency(portfolio.current_balance)}
          </div>
          <p className="text-xs text-muted-foreground">
            Equity: {formatCurrency(portfolio.equity)}
          </p>
        </CardContent>
      </Card>

      <Card
        className={
          wsConnected && portfolio.total_pnl !== 0
            ? "animate-pulse"
            : ""
        }
      >
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-1">
            Tổng P&L
            {wsConnected && (
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
            )}
          </CardTitle>
          <TrendingUp
            className={`h-4 w-4 ${
              portfolio.total_pnl >= 0 ? "text-profit" : "text-loss"
            }`}
          />
        </CardHeader>
        <CardContent>
          <div
            className={`text-2xl font-bold ${
              portfolio.total_pnl >= 0 ? "text-profit" : "text-loss"
            }`}
          >
            {portfolio.total_pnl >= 0 ? "+" : ""}
            {formatCurrency(portfolio.total_pnl)}
          </div>
          <p className="text-xs text-muted-foreground">
            {formatPercentage(portfolio.total_pnl_percentage)}
            {wsConnected && " • Live"}
          </p>
        </CardContent>
      </Card>

      <Card
        className={
          wsConnected && openTrades.length > 0 ? "border-green-200" : ""
        }
      >
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium flex items-center gap-1">
            Tổng số lệnh
            {wsConnected && openTrades.length > 0 && (
              <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
            )}
          </CardTitle>
          <Activity
            className={`h-4 w-4 ${
              wsConnected && openTrades.length > 0
                ? "text-blue-500 animate-pulse"
                : "text-muted-foreground"
            }`}
          />
        </CardHeader>
        <CardContent>
          <div className="text-2xl font-bold">
            {portfolio.total_trades}
          </div>
          <div className="space-y-1">
            <p className="text-xs text-muted-foreground">
              Đang mở: {openTrades.length} • Đã đóng:{" "}
              {closedTrades.length}
            </p>
            <div className="text-xs space-y-1">
              <div>
                <span className="text-muted-foreground">
                  Position Size:{" "}
                </span>
                <span className="font-medium text-primary">
                  {formatCurrency(
                    openTrades.reduce(
                      (total, trade) =>
                        total + calculatePositionSize(trade),
                      0
                    )
                  )}
                </span>
              </div>
              <div>
                <span className="text-muted-foreground">
                  Margin Used:{" "}
                </span>
                <span className="font-medium text-warning">
                  {formatCurrency(
                    openTrades.reduce(
                      (total, trade) =>
                        total + calculateMarginRequired(trade),
                      0
                    )
                  )}
                </span>
              </div>
            </div>
          </div>
        </CardContent>
      </Card>

      <Card>
        <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
          <CardTitle className="text-sm font-medium">
            {closedTrades.length > 0 ? "Tỷ lệ thắng" : "Margin Usage"}
          </CardTitle>
          <Target className="h-4 w-4 text-muted-foreground" />
        </CardHeader>
        <CardContent>
          {closedTrades.length > 0 ? (
            <>
              <div className="text-2xl font-bold">
                {portfolio.win_rate.toFixed(1)}%
              </div>
              <p className="text-xs text-muted-foreground">
                {Math.round(
                  (portfolio.win_rate * portfolio.total_trades) / 100
                )}
                /{portfolio.total_trades}
              </p>
            </>
          ) : (
            <>
              <div className="text-2xl font-bold text-warning">
                {(
                  (portfolio.margin_used / portfolio.equity) *
                  100
                ).toFixed(1)}
                %
              </div>
              <p className="text-xs text-muted-foreground">
                {formatCurrency(portfolio.margin_used)} /{" "}
                {formatCurrency(portfolio.equity)}
              </p>
            </>
          )}
        </CardContent>
      </Card>
    </div>
  );
}
