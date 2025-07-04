import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";

export function AISignals() {
  const mockSignals = [
    {
      id: 1,
      signal: "LONG",
      confidence: 0.87,
      timestamp: "2024-01-15 14:23:15",
      pair: "BTC/USDT",
      reason: "Strong bullish momentum with RSI oversold recovery",
      active: true
    },
    {
      id: 2,
      signal: "SHORT",
      confidence: 0.73,
      timestamp: "2024-01-15 13:45:22",
      pair: "ETH/USDT", 
      reason: "Bearish divergence detected on 4H timeframe",
      active: true
    },
    {
      id: 3,
      signal: "NEUTRAL",
      confidence: 0.65,
      timestamp: "2024-01-15 12:10:08",
      pair: "BNB/USDT",
      reason: "Consolidation phase, waiting for breakout",
      active: false
    },
    {
      id: 4,
      signal: "LONG",
      confidence: 0.91,
      timestamp: "2024-01-15 11:30:45",
      pair: "SOL/USDT",
      reason: "Volume surge with price action confirmation",
      active: false
    }
  ];

  const getSignalColor = (signal: string) => {
    switch (signal) {
      case "LONG":
        return "bg-profit text-profit-foreground";
      case "SHORT":
        return "bg-loss text-loss-foreground";
      default:
        return "bg-warning text-warning-foreground";
    }
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return "text-profit";
    if (confidence >= 0.6) return "text-warning";
    return "text-loss";
  };

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg flex items-center gap-2">
          AI Trading Signals
          <Badge variant="outline" className="bg-info/10 text-info border-info/20">
            <div className="w-2 h-2 bg-info rounded-full mr-2 animate-pulse"></div>
            Live Analysis
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4">
        {mockSignals.map((signal) => (
          <div 
            key={signal.id} 
            className={`p-4 rounded-lg border transition-all duration-200 hover:shadow-lg ${
              signal.active 
                ? 'bg-secondary/50 border-primary/20 shadow-primary/5' 
                : 'bg-muted/20 border-muted/40'
            }`}
          >
            <div className="flex justify-between items-start mb-3">
              <div className="flex items-center gap-3">
                <Badge className={getSignalColor(signal.signal)}>
                  {signal.signal}
                </Badge>
                <span className="font-semibold">{signal.pair}</span>
                {signal.active && (
                  <Badge variant="outline" className="bg-profit/10 text-profit border-profit/20 text-xs">
                    ACTIVE
                  </Badge>
                )}
              </div>
              <div className="text-right">
                <div className={`font-bold text-lg ${getConfidenceColor(signal.confidence)}`}>
                  {(signal.confidence * 100).toFixed(0)}%
                </div>
                <div className="text-xs text-muted-foreground">Confidence</div>
              </div>
            </div>
            
            <div className="space-y-2">
              <p className="text-sm text-foreground">{signal.reason}</p>
              <div className="flex justify-between items-center text-xs text-muted-foreground">
                <span>{signal.timestamp}</span>
                <div className="flex items-center gap-1">
                  <div className={`w-2 h-2 rounded-full ${
                    signal.confidence >= 0.8 ? 'bg-profit' : 
                    signal.confidence >= 0.6 ? 'bg-warning' : 'bg-loss'
                  }`}></div>
                  <span>AI Confidence</span>
                </div>
              </div>
            </div>

            {/* Confidence Bar */}
            <div className="mt-3">
              <div className="w-full bg-muted rounded-full h-1.5">
                <div 
                  className={`h-1.5 rounded-full transition-all duration-500 ${
                    signal.confidence >= 0.8 ? 'bg-profit' : 
                    signal.confidence >= 0.6 ? 'bg-warning' : 'bg-loss'
                  }`}
                  style={{ width: `${signal.confidence * 100}%` }}
                ></div>
              </div>
            </div>
          </div>
        ))}
      </CardContent>
    </Card>
  );
}