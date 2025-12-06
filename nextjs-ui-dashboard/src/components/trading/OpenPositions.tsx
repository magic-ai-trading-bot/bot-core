/**
 * Open Positions
 *
 * List of current open positions with live P&L updates.
 * Includes close position action.
 */

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Button } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { ScrollArea } from '@/components/ui/scroll-area';
import { useTradingMode } from '@/hooks/useTradingMode';
import type { PaperTrade } from '@/hooks/usePaperTrading';

interface OpenPositionsProps {
  trades: PaperTrade[];
  isLoading?: boolean;
  onCloseTrade?: (tradeId: string) => void;
}

export function OpenPositions({
  trades,
  isLoading = false,
  onCloseTrade,
}: OpenPositionsProps) {
  const { mode } = useTradingMode();

  const handleClose = (tradeId: string) => {
    if (onCloseTrade) {
      onCloseTrade(tradeId);
    }
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">Open Positions</CardTitle>
          <Badge variant="outline">{trades.length} positions</Badge>
        </div>
      </CardHeader>

      <CardContent>
        {isLoading ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            Loading positions...
          </div>
        ) : trades.length === 0 ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            No open positions
          </div>
        ) : (
          <ScrollArea className="h-[300px]">
            <div className="space-y-3">
              {trades.map((trade) => {
                const isProfitable = (trade.pnl || 0) >= 0;
                const pnlColor = isProfitable ? 'text-green-500' : 'text-red-500';

                return (
                  <div
                    key={trade.id}
                    className="rounded-lg border bg-card p-3 text-sm"
                  >
                    {/* Header */}
                    <div className="mb-2 flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold">{trade.symbol}</span>
                        <Badge
                          variant={trade.trade_type === 'Long' ? 'default' : 'destructive'}
                        >
                          {trade.trade_type}
                        </Badge>
                      </div>
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => handleClose(trade.id)}
                        disabled={isLoading}
                      >
                        Close
                      </Button>
                    </div>

                    {/* Details Grid */}
                    <div className="grid grid-cols-2 gap-2 text-xs">
                      <div>
                        <p className="text-muted-foreground">Entry Price</p>
                        <p className="font-semibold">${trade.entry_price.toFixed(2)}</p>
                      </div>
                      <div>
                        <p className="text-muted-foreground">Quantity</p>
                        <p className="font-semibold">{trade.quantity.toFixed(4)}</p>
                      </div>
                      <div>
                        <p className="text-muted-foreground">Leverage</p>
                        <p className="font-semibold">{trade.leverage}x</p>
                      </div>
                      <div>
                        <p className="text-muted-foreground">P&L</p>
                        <p className={`font-bold ${pnlColor}`}>
                          {isProfitable ? '+' : ''}
                          ${(trade.pnl || 0).toFixed(2)} ({trade.pnl_percentage.toFixed(2)}
                          %)
                        </p>
                      </div>
                    </div>

                    {/* Stop Loss / Take Profit */}
                    {(trade.stop_loss || trade.take_profit) && (
                      <div className="mt-2 grid grid-cols-2 gap-2 border-t pt-2 text-xs">
                        {trade.stop_loss && (
                          <div>
                            <p className="text-muted-foreground">Stop Loss</p>
                            <p className="text-red-500">${trade.stop_loss.toFixed(2)}</p>
                          </div>
                        )}
                        {trade.take_profit && (
                          <div>
                            <p className="text-muted-foreground">Take Profit</p>
                            <p className="text-green-500">
                              ${trade.take_profit.toFixed(2)}
                            </p>
                          </div>
                        )}
                      </div>
                    )}
                  </div>
                );
              })}
            </div>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}
