import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { PremiumButton } from "@/styles/luxury-design-system";
import { Skeleton } from "@/components/ui/skeleton";
import { TrendingUp } from "lucide-react";
import { usePaperTradingContext } from "@/contexts/PaperTradingContext";
import { useNavigate } from "react-router-dom";
import { formatTimestamp } from "@/utils/formatters";
import { memo, useMemo, useCallback } from "react";

// Memoized trade row component to prevent unnecessary re-renders
interface TradeRowProps {
  trade: {
    id: string;
    side: string;
    symbol: string;
    open_time: string;
    close_time?: string;
    leverage: number;
    realized_pnl: number;
    realized_pnl_percent: number;
    entry_price: number;
    exit_price?: number;
    quantity: number;
  };
}

const TradeRow = memo(({ trade }: TradeRowProps) => {
  const getTypeColor = useCallback((side: string) => {
    return side === "BUY" ? "bg-profit text-profit-foreground" : "bg-loss text-loss-foreground";
  }, []);

  const getPnLColor = useCallback((pnl: number) => {
    return pnl >= 0 ? "text-profit" : "text-loss";
  }, []);

  const progressWidth = useMemo(() => {
    return Math.min(Math.abs(trade.realized_pnl_percent || 0) * 10, 100);
  }, [trade.realized_pnl_percent]);

  return (
    <div className="p-4 rounded-lg border bg-secondary/20 hover:bg-secondary/40 transition-colors">
      <div className="flex items-start justify-between mb-3">
        <div className="flex items-center gap-3">
          <Badge className={getTypeColor(trade.side)}>
            {trade.side === "BUY" ? "LONG" : "SHORT"}
          </Badge>
          <div>
            <div className="font-semibold">{trade.symbol}</div>
            <div className="text-xs text-muted-foreground">
              {formatTimestamp(trade.close_time || trade.open_time, 'datetime', 'vi-VN')}
            </div>
          </div>
          <Badge variant="outline" className="text-xs">
            {trade.leverage || 1}x
          </Badge>
        </div>

        <div className="text-right">
          <div className={`font-bold text-lg ${getPnLColor(trade.realized_pnl || 0)}`}>
            {(trade.realized_pnl || 0) >= 0 ? '+' : ''}${(trade.realized_pnl || 0).toFixed(2)}
          </div>
          <div className={`text-sm ${getPnLColor(trade.realized_pnl || 0)}`}>
            {(trade.realized_pnl_percent || 0) >= 0 ? '+' : ''}{(trade.realized_pnl_percent || 0).toFixed(2)}%
          </div>
        </div>
      </div>

      <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
        <div>
          <span className="text-muted-foreground">Entry: </span>
          <span className="font-mono">${(trade?.entry_price || 0).toLocaleString()}</span>
        </div>
        <div>
          <span className="text-muted-foreground">Exit: </span>
          <span className="font-mono">${(trade.exit_price || 0).toLocaleString()}</span>
        </div>
        <div>
          <span className="text-muted-foreground">Size: </span>
          <span className="font-mono">{trade.quantity || 0}</span>
        </div>
        <div>
          <Badge
            variant="outline"
            className="bg-profit/10 text-profit border-profit/20 text-xs"
          >
            Closed
          </Badge>
        </div>
      </div>

      {/* P&L Progress Bar */}
      <div className="mt-3">
        <div className="w-full bg-muted rounded-full h-1">
          <div
            className={`h-1 rounded-full transition-all duration-500 ${
              (trade.realized_pnl || 0) >= 0 ? 'bg-profit' : 'bg-loss'
            }`}
            style={{ width: `${progressWidth}%` }}
          ></div>
        </div>
      </div>
    </div>
  );
});

TradeRow.displayName = 'TradeRow';

export function TransactionHistory() {
  const { closedTrades, isLoading } = usePaperTradingContext();
  const navigate = useNavigate();

  // Memoize navigation callback
  const handleNavigateToTrading = useCallback(() => {
    navigate('/trading');
  }, [navigate]);

  // Memoize sorted and sliced trades list
  const displayedTrades = useMemo(() => {
    return closedTrades.slice(0, 10);
  }, [closedTrades]);

  // Memoize remaining trades count
  const remainingTradesCount = useMemo(() => {
    return closedTrades && closedTrades.length > 10 ? closedTrades.length - 10 : 0;
  }, [closedTrades]);

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <div className="flex items-center justify-between">
            <Skeleton className="h-6 w-48" />
            <Skeleton className="h-9 w-32" />
          </div>
        </CardHeader>
        <CardContent className="space-y-4">
          {[1, 2, 3, 4, 5].map((i) => (
            <div key={i} className="p-4 rounded-lg border bg-secondary/20 space-y-3">
              <div className="flex items-start justify-between">
                <div className="flex items-center gap-3">
                  <Skeleton className="h-6 w-16" />
                  <div className="space-y-2">
                    <Skeleton className="h-5 w-24" />
                    <Skeleton className="h-4 w-32" />
                  </div>
                  <Skeleton className="h-6 w-12" />
                </div>
                <div className="space-y-2 text-right">
                  <Skeleton className="h-6 w-20" />
                  <Skeleton className="h-4 w-16" />
                </div>
              </div>
              <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-full" />
                <Skeleton className="h-4 w-full" />
              </div>
              <Skeleton className="h-1 w-full" />
            </div>
          ))}
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">Recent Transactions</CardTitle>
          <PremiumButton variant="secondary" size="sm">
            Export History
          </PremiumButton>
        </div>
      </CardHeader>
      <CardContent>
        {!closedTrades || closedTrades.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <div className="rounded-full bg-muted p-4 mb-4">
              <TrendingUp className="h-8 w-8 text-muted-foreground" />
            </div>
            <h3 className="font-semibold text-lg mb-2">Chưa có giao dịch nào</h3>
            <p className="text-muted-foreground mb-4 max-w-sm">
              Bắt đầu trade để xem lịch sử giao dịch của bạn. Bot sẽ tự động ghi lại tất cả các giao dịch.
            </p>
            <PremiumButton onClick={handleNavigateToTrading}>
              Bắt đầu trading
            </PremiumButton>
          </div>
        ) : (
          <>
            <div className="space-y-4">
              {displayedTrades.map((trade) => (
                <TradeRow key={trade.id} trade={trade} />
              ))}
            </div>

            {/* Load More */}
            {remainingTradesCount > 0 && (
              <div className="text-center pt-4">
                <PremiumButton variant="secondary" className="w-full">
                  Load More Transactions ({remainingTradesCount} more)
                </PremiumButton>
              </div>
            )}
          </>
        )}
      </CardContent>
    </Card>
  );
}