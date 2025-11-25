import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { History } from "lucide-react";
import { Trade } from "./types";

interface ClosedTradesTableProps {
  closedTrades: Trade[];
  wsConnected: boolean;
  formatCurrency: (value: number) => string;
  formatPercentage: (value: number | undefined) => string;
  openTradeDetails: (trade: Trade) => void;
}

export function ClosedTradesTable({
  closedTrades,
  wsConnected,
  formatCurrency,
  formatPercentage,
  openTradeDetails,
}: ClosedTradesTableProps) {
  return (
    <Card className={wsConnected ? "ring-1 ring-green-500/20" : ""}>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Lịch sử giao dịch ({closedTrades.length})
          {wsConnected && (
            <div className="flex items-center gap-1">
              <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
              <span className="text-xs text-green-600">Live</span>
            </div>
          )}
        </CardTitle>
      </CardHeader>
      <CardContent>
        {closedTrades.length > 0 ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Symbol</TableHead>
                <TableHead>Type</TableHead>
                <TableHead>Entry</TableHead>
                <TableHead>Exit</TableHead>
                <TableHead>Quantity</TableHead>
                <TableHead>P&L</TableHead>
                <TableHead>P&L %</TableHead>
                <TableHead>Duration</TableHead>
                <TableHead>Reason</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {closedTrades
                .slice()
                .reverse()
                .map((trade) => (
                  <TableRow
                    key={trade.id}
                    className="cursor-pointer hover:bg-muted/50 transition-colors"
                    onClick={() => openTradeDetails(trade)}
                  >
                    <TableCell className="font-medium">
                      {trade.symbol}
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={
                          trade.trade_type === "Long"
                            ? "default"
                            : "destructive"
                        }
                      >
                        {trade.trade_type}
                      </Badge>
                    </TableCell>
                    <TableCell>
                      {formatCurrency(trade.entry_price)}
                    </TableCell>
                    <TableCell>
                      {trade.exit_price
                        ? formatCurrency(trade.exit_price)
                        : "N/A"}
                    </TableCell>
                    <TableCell>{trade.quantity.toFixed(6)}</TableCell>
                    <TableCell
                      className={
                        trade.pnl && trade.pnl >= 0
                          ? "text-profit"
                          : "text-loss"
                      }
                    >
                      {trade.pnl ? formatCurrency(trade.pnl) : "N/A"}
                    </TableCell>
                    <TableCell
                      className={
                        trade.pnl_percentage &&
                        trade.pnl_percentage >= 0
                          ? "text-profit"
                          : "text-loss"
                      }
                    >
                      {trade.pnl_percentage
                        ? formatPercentage(trade.pnl_percentage)
                        : "N/A"}
                    </TableCell>
                    <TableCell>
                      {trade.close_time
                        ? Math.round(
                            (new Date(trade.close_time).getTime() -
                              new Date(trade.open_time).getTime()) /
                              (1000 * 60)
                          ) + "m"
                        : "N/A"}
                    </TableCell>
                    <TableCell>
                      <Badge
                        variant={
                          trade.close_reason === "StopLoss" || trade.close_reason === "MarginCall"
                            ? "destructive"
                            : trade.close_reason === "TakeProfit"
                            ? "default"
                            : "outline"
                        }
                      >
                        {trade.close_reason || "Unknown"}
                      </Badge>
                    </TableCell>
                  </TableRow>
                ))}
            </TableBody>
          </Table>
        ) : (
          <div className="flex items-center justify-center h-32 text-muted-foreground">
            <div className="text-center">
              <History className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>Chưa có giao dịch nào</p>
              <p className="text-sm">
                Giao dịch sẽ hiển thị tại đây khi bot hoạt động
              </p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
