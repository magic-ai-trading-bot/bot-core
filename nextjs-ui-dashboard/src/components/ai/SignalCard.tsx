import { Badge } from "@/components/ui/badge";
import { FormattedSignal } from "./types";

interface SignalCardProps {
  signal: FormattedSignal;
  onClick: () => void;
}

export function SignalCard({ signal, onClick }: SignalCardProps) {
  const getSignalColor = (signalType: string) => {
    switch (signalType) {
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
    <div
      className={`p-4 rounded-lg border transition-all duration-200 hover:shadow-lg cursor-pointer ${
        signal.active
          ? "bg-secondary/50 border-primary/20 shadow-primary/5 hover:bg-secondary/70"
          : "bg-muted/20 border-muted/40 hover:bg-muted/30"
      }`}
      onClick={onClick}
    >
      <div className="flex justify-between items-start mb-3">
        <div className="flex items-center gap-3">
          <Badge className={getSignalColor(signal.signal)}>
            {signal.signal}
          </Badge>
          <span className="font-semibold">{signal.pair}</span>
          {signal.active && (
            <Badge
              variant="outline"
              className="bg-profit/10 text-profit border-profit/20 text-xs"
            >
              ACTIVE
            </Badge>
          )}
        </div>
        <div className="text-right">
          <div
            className={`font-bold text-lg ${getConfidenceColor(
              signal.confidence
            )}`}
          >
            {(signal.confidence * 100).toFixed(0)}%
          </div>
          <div className="text-xs text-muted-foreground">
            Confidence
          </div>
        </div>
      </div>

      <div className="space-y-2">
        <p className="text-sm text-foreground">{signal.reason}</p>
        <div className="flex justify-between items-center text-xs text-muted-foreground">
          <span>{signal.timestamp}</span>
          <div className="flex items-center gap-1">
            <div
              className={`w-2 h-2 rounded-full ${
                signal.confidence >= 0.8
                  ? "bg-profit"
                  : signal.confidence >= 0.6
                  ? "bg-warning"
                  : "bg-loss"
              }`}
            ></div>
            <span>AI Confidence</span>
          </div>
        </div>
      </div>

      {/* Confidence Bar */}
      <div className="mt-3">
        <div className="w-full bg-muted rounded-full h-1.5">
          <div
            className={`h-1.5 rounded-full transition-all duration-500 ${
              signal.confidence >= 0.8
                ? "bg-profit"
                : signal.confidence >= 0.6
                ? "bg-warning"
                : "bg-loss"
            }`}
            style={{ width: `${signal.confidence * 100}%` }}
          ></div>
        </div>
      </div>
    </div>
  );
}
