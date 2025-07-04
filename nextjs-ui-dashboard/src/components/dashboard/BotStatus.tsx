import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function BotStatus() {
  const mockData = {
    balance: 12450.32,
    availableFunds: 8200.15,
    currentPrice: 43567.89,
    openPositions: [
      {
        id: 1,
        pair: "BTC/USDT",
        type: "LONG",
        entryPrice: 42800.50,
        leverage: "10x",
        size: 0.1,
        pnl: 767.39,
        pnlPercent: 1.79
      },
      {
        id: 2,
        pair: "ETH/USDT",
        type: "SHORT",
        entryPrice: 2650.25,
        leverage: "5x",
        size: 2.5,
        pnl: -125.60,
        pnlPercent: -0.47
      }
    ]
  };

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
              <p className="text-xl lg:text-2xl font-bold">${mockData.balance.toLocaleString()}</p>
            </div>
            <div>
              <p className="text-xs lg:text-sm text-muted-foreground">Available Funds</p>
              <p className="text-xl lg:text-2xl font-bold text-profit">${mockData.availableFunds.toLocaleString()}</p>
            </div>
          </div>
          <div className="pt-2 border-t">
            <div className="flex justify-between items-center">
              <span className="text-xs lg:text-sm text-muted-foreground">BTC/USDT</span>
              <span className="font-mono text-base lg:text-lg">${mockData.currentPrice.toLocaleString()}</span>
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
          {mockData.openPositions.map((position) => (
            <div key={position.id} className="p-3 rounded-lg bg-secondary/50 border">
              <div className="flex flex-col sm:flex-row sm:justify-between sm:items-start gap-2 mb-2">
                <div className="flex items-center gap-2 flex-wrap">
                  <Badge 
                    variant={position.type === "LONG" ? "default" : "secondary"}
                    className={position.type === "LONG" ? "bg-profit text-profit-foreground" : "bg-loss text-loss-foreground"}
                  >
                    {position.type}
                  </Badge>
                  <span className="font-semibold text-sm lg:text-base">{position.pair}</span>
                  <span className="text-xs lg:text-sm text-muted-foreground">{position.leverage}</span>
                </div>
                <div className="text-left sm:text-right">
                  <div className={`font-semibold text-base lg:text-lg ${position.pnl >= 0 ? 'text-profit' : 'text-loss'}`}>
                    {position.pnl >= 0 ? '+' : ''}${position.pnl.toFixed(2)}
                  </div>
                  <div className={`text-xs lg:text-sm ${position.pnl >= 0 ? 'text-profit' : 'text-loss'}`}>
                    {position.pnl >= 0 ? '+' : ''}{position.pnlPercent.toFixed(2)}%
                  </div>
                </div>
              </div>
              <div className="flex flex-col sm:flex-row sm:justify-between gap-1 text-xs lg:text-sm text-muted-foreground">
                <span>Entry: ${position.entryPrice.toLocaleString()}</span>
                <span>Size: {position.size} BTC</span>
              </div>
            </div>
          ))}
        </CardContent>
      </Card>
    </div>
  );
}