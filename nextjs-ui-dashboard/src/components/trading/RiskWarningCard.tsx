/**
 * Risk Warning Card
 *
 * Displays current portfolio risk metrics:
 * - Current exposure
 * - Daily loss status
 * - Margin usage
 */

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Progress } from '@/components/ui/progress';
import { Badge } from '@/components/ui/badge';
import type { PortfolioMetrics } from '@/hooks/usePaperTrading';

interface RiskWarningCardProps {
  portfolio: PortfolioMetrics;
  dailyLossLimit: number; // Percentage
  maxDrawdown: number; // Percentage
}

export function RiskWarningCard({
  portfolio,
  dailyLossLimit = 5,
  maxDrawdown = 15,
}: RiskWarningCardProps) {
  // Calculate metrics
  const marginUsagePercent =
    portfolio.equity > 0 ? (portfolio.margin_used / portfolio.equity) * 100 : 0;

  const currentDrawdownPercent = Math.abs(portfolio.max_drawdown_percentage);
  const drawdownStatus =
    currentDrawdownPercent < maxDrawdown * 0.5
      ? 'safe'
      : currentDrawdownPercent < maxDrawdown * 0.8
      ? 'warning'
      : 'danger';

  // Daily P&L percentage (assuming we track this)
  const dailyPnlPercent = portfolio.total_pnl_percentage; // Simplified
  const dailyLossStatus =
    dailyPnlPercent > 0
      ? 'profit'
      : Math.abs(dailyPnlPercent) < dailyLossLimit * 0.5
      ? 'safe'
      : Math.abs(dailyPnlPercent) < dailyLossLimit * 0.8
      ? 'warning'
      : 'danger';

  const getStatusColor = (status: string) => {
    switch (status) {
      case 'profit':
        return 'text-green-500';
      case 'safe':
        return 'text-blue-500';
      case 'warning':
        return 'text-yellow-500';
      case 'danger':
        return 'text-red-500';
      default:
        return 'text-muted-foreground';
    }
  };

  const getProgressColor = (status: string) => {
    switch (status) {
      case 'profit':
        return 'bg-green-500';
      case 'safe':
        return 'bg-blue-500';
      case 'warning':
        return 'bg-yellow-500';
      case 'danger':
        return 'bg-red-500';
      default:
        return 'bg-primary';
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">Risk Monitor</CardTitle>
          <Badge
            variant={
              drawdownStatus === 'danger' || dailyLossStatus === 'danger'
                ? 'destructive'
                : 'outline'
            }
          >
            {drawdownStatus === 'danger' || dailyLossStatus === 'danger'
              ? 'High Risk'
              : 'Normal'}
          </Badge>
        </div>
      </CardHeader>

      <CardContent className="space-y-4">
        {/* Margin Usage */}
        <div>
          <div className="mb-2 flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Margin Usage</span>
            <span className="font-semibold">{marginUsagePercent.toFixed(1)}%</span>
          </div>
          <Progress value={marginUsagePercent} className="h-2" />
          <div className="mt-1 text-xs text-muted-foreground">
            ${portfolio.margin_used.toFixed(2)} / ${portfolio.equity.toFixed(2)}
          </div>
        </div>

        {/* Daily P&L */}
        <div>
          <div className="mb-2 flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Daily P&L</span>
            <span className={`font-semibold ${getStatusColor(dailyLossStatus)}`}>
              {dailyPnlPercent >= 0 ? '+' : ''}
              {dailyPnlPercent.toFixed(2)}%
            </span>
          </div>
          <Progress
            value={Math.min(Math.abs(dailyPnlPercent), dailyLossLimit)}
            max={dailyLossLimit}
            className="h-2"
          />
          <div className="mt-1 text-xs text-muted-foreground">
            Limit: {dailyLossLimit}% daily loss
          </div>
        </div>

        {/* Max Drawdown */}
        <div>
          <div className="mb-2 flex items-center justify-between text-sm">
            <span className="text-muted-foreground">Max Drawdown</span>
            <span className={`font-semibold ${getStatusColor(drawdownStatus)}`}>
              {currentDrawdownPercent.toFixed(2)}%
            </span>
          </div>
          <Progress
            value={Math.min(currentDrawdownPercent, maxDrawdown)}
            max={maxDrawdown}
            className="h-2"
          />
          <div className="mt-1 text-xs text-muted-foreground">
            Limit: {maxDrawdown}% max drawdown
          </div>
        </div>

        {/* Current Exposure */}
        <div className="rounded-md border bg-muted/50 p-3">
          <p className="mb-2 text-xs font-semibold">Current Exposure</p>
          <div className="grid grid-cols-2 gap-2 text-xs">
            <div>
              <p className="text-muted-foreground">Equity</p>
              <p className="font-semibold">${portfolio.equity.toFixed(2)}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Free Margin</p>
              <p className="font-semibold">${portfolio.free_margin.toFixed(2)}</p>
            </div>
            <div>
              <p className="text-muted-foreground">Total P&L</p>
              <p
                className={`font-semibold ${
                  portfolio.total_pnl >= 0 ? 'text-green-500' : 'text-red-500'
                }`}
              >
                ${portfolio.total_pnl.toFixed(2)}
              </p>
            </div>
            <div>
              <p className="text-muted-foreground">Win Rate</p>
              <p className="font-semibold">{portfolio.win_rate.toFixed(1)}%</p>
            </div>
          </div>
        </div>

        {/* Warning Messages */}
        {(drawdownStatus === 'danger' || dailyLossStatus === 'danger') && (
          <div className="rounded-md bg-destructive/10 p-3 text-xs">
            <p className="font-semibold text-destructive">⚠️ Risk Alert</p>
            {drawdownStatus === 'danger' && (
              <p className="mt-1 text-muted-foreground">
                Drawdown limit approaching. Consider reducing exposure.
              </p>
            )}
            {dailyLossStatus === 'danger' && (
              <p className="mt-1 text-muted-foreground">
                Daily loss limit approaching. Trading may be paused.
              </p>
            )}
          </div>
        )}
      </CardContent>
    </Card>
  );
}
