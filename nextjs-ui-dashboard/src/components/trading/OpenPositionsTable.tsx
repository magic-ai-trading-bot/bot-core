import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Trade } from "./types";

interface OpenPositionsTableProps {
  openTrades: Trade[];
  wsConnected: boolean;
  calculatePositionSize: (trade: Trade) => number;
  calculateMarginRequired: (trade: Trade) => number;
  formatCurrency: (value: number) => string;
  formatDate: (date: Date | string | number) => string;
  openTradeDetails: (trade: Trade) => void;
  closeTrade: (tradeId: string) => void;
}

export function OpenPositionsTable({
  openTrades,
  wsConnected,
  calculatePositionSize,
  calculateMarginRequired,
  formatCurrency,
  formatDate,
  openTradeDetails,
  closeTrade,
}: OpenPositionsTableProps) {
  if (openTrades.length === 0) {
    return null;
  }

  return (
    <Card className={wsConnected ? "ring-1 ring-green-500/20" : ""}>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="flex items-center gap-2">
            Lệnh đang mở ({openTrades.length})
            {wsConnected && (
              <div className="flex items-center gap-1">
                <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                <span className="text-xs text-green-600">Live</span>
              </div>
            )}
          </CardTitle>
          <div className="text-right space-y-1">
            <div>
              <div className="text-sm text-muted-foreground">
                Tổng Position Size
              </div>
              <div className="font-bold text-primary">
                {formatCurrency(
                  openTrades.reduce(
                    (total, trade) =>
                      total + calculatePositionSize(trade),
                    0
                  )
                )}
              </div>
            </div>
            <div>
              <div className="text-sm text-muted-foreground">
                Tổng Margin Required
              </div>
              <div className="font-bold text-warning">
                {formatCurrency(
                  openTrades.reduce(
                    (total, trade) =>
                      total + calculateMarginRequired(trade),
                    0
                  )
                )}
              </div>
            </div>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Symbol</TableHead>
              <TableHead>Type</TableHead>
              <TableHead>Entry Price</TableHead>
              <TableHead>Quantity</TableHead>
              <TableHead>Position Size</TableHead>
              <TableHead>Margin Required</TableHead>
              <TableHead>Leverage</TableHead>
              <TableHead>Unrealized P&L</TableHead>
              <TableHead>Stop Loss</TableHead>
              <TableHead>Take Profit</TableHead>
              <TableHead>Open Time</TableHead>
              <TableHead>Action</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {openTrades.map((trade) => (
              <TableRow
                key={trade.id}
                className="cursor-pointer hover:bg-muted/50 transition-colors"
                onClick={() => openTradeDetails(trade)}
              >
                <TableCell className="font-medium">
                  <div className="flex items-center gap-2">
                    {trade.symbol}
                    <span className="text-xs text-muted-foreground">
                      ({trade.leverage}x)
                    </span>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge
                    variant={
                      trade.trade_type === "Long"
                        ? "default"
                        : "destructive"
                    }
                    className={
                      trade.trade_type === "Long"
                        ? "bg-profit text-profit-foreground"
                        : "bg-loss text-loss-foreground"
                    }
                  >
                    {trade.trade_type}
                  </Badge>
                </TableCell>
                <TableCell>
                  {formatCurrency(trade.entry_price)}
                </TableCell>
                <TableCell>
                  <div className="text-right">
                    <div className="font-medium">
                      {trade.quantity.toFixed(6)}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      {trade.symbol.replace("USDT", "")}
                    </div>
                  </div>
                </TableCell>
                <TableCell>
                  <div className="text-right">
                    <div className="font-medium text-primary">
                      {formatCurrency(calculatePositionSize(trade))}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      Notional Value
                    </div>
                  </div>
                </TableCell>
                <TableCell>
                  <div className="text-right">
                    <div className="font-medium text-warning">
                      {formatCurrency(calculateMarginRequired(trade))}
                    </div>
                    <div className="text-xs text-muted-foreground">
                      với {trade.leverage}x leverage
                    </div>
                  </div>
                </TableCell>
                <TableCell>
                  <Badge variant="outline" className="font-mono">
                    {trade.leverage}x
                  </Badge>
                </TableCell>
                <TableCell>
                  <div className="text-right">
                    <div
                      className={`font-medium ${
                        (trade.pnl || 0) >= 0
                          ? "text-profit"
                          : "text-loss"
                      }`}
                    >
                      {formatCurrency(trade.pnl || 0)}
                    </div>
                    <div
                      className={`text-xs ${
                        trade.pnl_percentage >= 0
                          ? "text-profit"
                          : "text-loss"
                      }`}
                    >
                      ({trade.pnl_percentage >= 0 ? "+" : ""}
                      {trade.pnl_percentage.toFixed(2)}%)
                    </div>
                  </div>
                </TableCell>
                <TableCell>
                  <div className="text-center">
                    {trade.stop_loss ? (
                      <div className="text-loss font-medium">
                        {formatCurrency(trade.stop_loss)}
                      </div>
                    ) : (
                      <Badge variant="secondary" className="text-xs">
                        Chưa đặt
                      </Badge>
                    )}
                  </div>
                </TableCell>
                <TableCell>
                  <div className="text-center">
                    {trade.take_profit ? (
                      <div className="text-profit font-medium">
                        {formatCurrency(trade.take_profit)}
                      </div>
                    ) : (
                      <Badge variant="secondary" className="text-xs">
                        Chưa đặt
                      </Badge>
                    )}
                  </div>
                </TableCell>
                <TableCell>
                  <div className="text-sm">
                    {formatDate(new Date(trade.open_time))}
                  </div>
                </TableCell>
                <TableCell>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={(e) => {
                      e.stopPropagation();
                      closeTrade(trade.id);
                    }}
                    className="hover:bg-destructive hover:text-destructive-foreground"
                  >
                    Đóng
                  </Button>
                </TableCell>
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </CardContent>
    </Card>
  );
}
