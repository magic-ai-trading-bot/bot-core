import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { TrendingUp, TrendingDown } from "lucide-react";
import { usePaperTrading } from "@/hooks/usePaperTrading";
import { useMarketData } from "@/hooks/useMarketData";

export function BotStatus() {
  const { portfolio, positions, isLoading } = usePaperTrading();
  const { data: marketData } = useMarketData("BTCUSDT", "1h", 5000);

  if (isLoading) {
    return (
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6">
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Skeleton className="h-4 w-24" />
                <Skeleton className="h-8 w-32" />
              </div>
              <div className="space-y-2">
                <Skeleton className="h-4 w-24" />
                <Skeleton className="h-8 w-32" />
              </div>
            </div>
            <div className="pt-2 border-t">
              <Skeleton className="h-6 w-full" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent className="space-y-3">
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-20 w-full" />
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6">
      {/* Balance Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base lg:text-lg">Account Balance</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <p className="text-xs lg:text-sm text-muted-foreground">Total Balance</p>
              <p className="text-xl lg:text-2xl font-bold">${(portfolio?.current_balance || 0).toLocaleString()}</p>
            </div>
            <div>
              <p className="text-xs lg:text-sm text-muted-foreground">Available Funds</p>
              <p className="text-xl lg:text-2xl font-bold text-profit">${(portfolio?.available_balance || 0).toLocaleString()}</p>
            </div>
          </div>
          <div className="pt-2 border-t">
            <div className="flex justify-between items-center">
              <span className="text-xs lg:text-sm text-muted-foreground">BTC/USDT</span>
              <span className="font-mono text-base lg:text-lg">${(marketData?.price || 0).toLocaleString()}</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Open Positions */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base lg:text-lg">Open Positions</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 lg:space-y-4">
          {positions.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <div className="rounded-full bg-muted p-3 mb-3">
                <TrendingUp className="h-6 w-6 text-muted-foreground" />
              </div>
              <p className="text-muted-foreground text-sm">
                Không có vị thế đang mở
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                Bot sẽ tự động mở vị thế khi phát hiện tín hiệu
              </p>
            </div>
          ) : (
            positions.map((position) => (
              <div key={position.id} className="p-3 rounded-lg bg-secondary/50 border">
                <div className="flex flex-col sm:flex-row sm:justify-between sm:items-start gap-2 mb-2">
                  <div className="flex items-center gap-2 flex-wrap">
                    <Badge
                      variant={position.side === "BUY" ? "default" : "secondary"}
                      className={position.side === "BUY" ? "bg-profit text-profit-foreground" : "bg-loss text-loss-foreground"}
                    >
                      {position.side === "BUY" ? "LONG" : "SHORT"}
                    </Badge>
                    <span className="font-semibold text-sm lg:text-base">{position.symbol}</span>
                    <span className="text-xs lg:text-sm text-muted-foreground">{position.leverage}x</span>
                  </div>
                  <div className="text-left sm:text-right">
                    <div className={`font-semibold text-base lg:text-lg flex items-center gap-1 ${position.unrealized_pnl >= 0 ? 'text-profit' : 'text-loss'}`}>
                      {position.unrealized_pnl >= 0 ? (
                        <TrendingUp className="h-4 w-4" aria-hidden="true" />
                      ) : (
                        <TrendingDown className="h-4 w-4" aria-hidden="true" />
                      )}
                      <span>{position.unrealized_pnl >= 0 ? '+' : ''}${position.unrealized_pnl.toFixed(2)}</span>
                      <span className="sr-only">{position.unrealized_pnl >= 0 ? 'Profit' : 'Loss'}</span>
                    </div>
                    <div className={`text-xs lg:text-sm flex items-center gap-1 ${position.unrealized_pnl_percent >= 0 ? 'text-profit' : 'text-loss'}`}>
                      <span>{position.unrealized_pnl_percent >= 0 ? '+' : ''}{position.unrealized_pnl_percent.toFixed(2)}%</span>
                    </div>
                  </div>
                </div>
                <div className="flex flex-col sm:flex-row sm:justify-between gap-1 text-xs lg:text-sm text-muted-foreground">
                  <span>Entry: ${(position?.entry_price || 0).toLocaleString()}</span>
                  <span>Size: {position?.quantity || 0}</span>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}