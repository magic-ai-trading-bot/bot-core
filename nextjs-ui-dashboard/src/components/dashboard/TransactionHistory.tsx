import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";

export function TransactionHistory() {
  const mockTransactions = [
    {
      id: 1,
      timestamp: "2024-01-15 14:23:15",
      pair: "BTC/USDT",
      type: "LONG",
      entryPrice: 42800.50,
      exitPrice: 43567.89,
      size: 0.1,
      leverage: "10x",
      pnl: 767.39,
      pnlPercent: 1.79,
      status: "Closed"
    },
    {
      id: 2,
      timestamp: "2024-01-15 13:45:22",
      pair: "ETH/USDT",
      type: "SHORT",
      entryPrice: 2650.25,
      exitPrice: 2625.80,
      size: 2.5,
      leverage: "5x",
      pnl: 305.60,
      pnlPercent: 1.15,
      status: "Closed"
    },
    {
      id: 3,
      timestamp: "2024-01-15 12:10:08",
      pair: "BNB/USDT",
      type: "LONG",
      entryPrice: 315.40,
      exitPrice: 312.85,
      size: 10,
      leverage: "3x",
      pnl: -76.50,
      pnlPercent: -0.81,
      status: "Closed"
    },
    {
      id: 4,
      timestamp: "2024-01-15 11:30:45",
      pair: "SOL/USDT",
      type: "SHORT",
      entryPrice: 98.75,
      exitPrice: 95.20,
      size: 50,
      leverage: "5x",
      pnl: 887.50,
      pnlPercent: 3.59,
      status: "Closed"
    },
    {
      id: 5,
      timestamp: "2024-01-15 10:15:30",
      pair: "BTC/USDT",
      type: "LONG",
      entryPrice: 42150.00,
      exitPrice: 41890.25,
      size: 0.05,
      leverage: "15x",
      pnl: -194.63,
      pnlPercent: -0.62,
      status: "Closed"
    }
  ];

  const getTypeColor = (type: string) => {
    return type === "LONG" ? "bg-profit text-profit-foreground" : "bg-loss text-loss-foreground";
  };

  const getPnLColor = (pnl: number) => {
    return pnl >= 0 ? "text-profit" : "text-loss";
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-lg">Recent Transactions</CardTitle>
          <Button variant="outline" size="sm">
            Export History
          </Button>
        </div>
      </CardHeader>
      <CardContent>
        <div className="space-y-4">
          {mockTransactions.map((transaction) => (
            <div 
              key={transaction.id}
              className="p-4 rounded-lg border bg-secondary/20 hover:bg-secondary/40 transition-colors"
            >
              <div className="flex items-start justify-between mb-3">
                <div className="flex items-center gap-3">
                  <Badge className={getTypeColor(transaction.type)}>
                    {transaction.type}
                  </Badge>
                  <div>
                    <div className="font-semibold">{transaction.pair}</div>
                    <div className="text-xs text-muted-foreground">
                      {transaction.timestamp}
                    </div>
                  </div>
                  <Badge variant="outline" className="text-xs">
                    {transaction.leverage}
                  </Badge>
                </div>
                
                <div className="text-right">
                  <div className={`font-bold text-lg ${getPnLColor(transaction.pnl)}`}>
                    {transaction.pnl >= 0 ? '+' : ''}${transaction.pnl.toFixed(2)}
                  </div>
                  <div className={`text-sm ${getPnLColor(transaction.pnl)}`}>
                    {transaction.pnl >= 0 ? '+' : ''}{transaction.pnlPercent.toFixed(2)}%
                  </div>
                </div>
              </div>

              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                <div>
                  <span className="text-muted-foreground">Entry: </span>
                  <span className="font-mono">${transaction.entryPrice.toLocaleString()}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Exit: </span>
                  <span className="font-mono">${transaction.exitPrice.toLocaleString()}</span>
                </div>
                <div>
                  <span className="text-muted-foreground">Size: </span>
                  <span className="font-mono">{transaction.size}</span>
                </div>
                <div>
                  <Badge 
                    variant="outline" 
                    className="bg-profit/10 text-profit border-profit/20 text-xs"
                  >
                    {transaction.status}
                  </Badge>
                </div>
              </div>

              {/* P&L Progress Bar */}
              <div className="mt-3">
                <div className="w-full bg-muted rounded-full h-1">
                  <div 
                    className={`h-1 rounded-full transition-all duration-500 ${
                      transaction.pnl >= 0 ? 'bg-profit' : 'bg-loss'
                    }`}
                    style={{ 
                      width: `${Math.min(Math.abs(transaction.pnlPercent) * 10, 100)}%` 
                    }}
                  ></div>
                </div>
              </div>
            </div>
          ))}
        </div>

        {/* Load More */}
        <div className="text-center pt-4">
          <Button variant="outline" className="w-full">
            Load More Transactions
          </Button>
        </div>
      </CardContent>
    </Card>
  );
}