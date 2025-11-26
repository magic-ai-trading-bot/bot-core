import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Dialog, DialogTrigger } from "@/components/ui/dialog";
import { useAIAnalysisContext } from "@/contexts/AIAnalysisContext";
import { useWebSocketContext } from "@/contexts/WebSocketContext";
import {
  AlertCircle,
  RefreshCw,
  Zap,
  TrendingUp,
  Wifi,
  WifiOff,
} from "lucide-react";
import { CombinedSignal, FormattedSignal } from "./types";
import { SignalCard } from "./SignalCard";
import { DetailedSignalDialog } from "./DetailedSignalDialog";

export function AISignalsDashboard() {
  const { state: aiState, clearError } = useAIAnalysisContext();
  const { state: wsState } = useWebSocketContext();

  // Combine signals from both AI analysis and WebSocket
  const allSignalsRaw = [
    ...aiState.signals.map((s) => ({ ...s, source: "api" })),
    ...wsState.aiSignals.map((s) => ({
      ...s,
      source: "websocket",
      // Use reasoning from WebSocket if available, otherwise generate a meaningful default
      reasoning: s.reasoning ||
        `Real-time ${s.model_type} analysis for ${s.symbol} on ${s.timeframe} timeframe`,
      // Use strategy_scores from WebSocket if available
      strategy_scores: s.strategy_scores || {},
      market_analysis: {
        trend_direction:
          s.signal === "long"
            ? "Bullish"
            : s.signal === "short"
            ? "Bearish"
            : "Sideways",
        trend_strength: s.confidence,
        support_levels: [],
        resistance_levels: [],
        volatility_level: "Medium",
        volume_analysis: "Real-time analysis",
      },
      risk_assessment: {
        overall_risk: "Medium",
        technical_risk: 0.5,
        market_risk: 0.5,
        recommended_position_size: 0.02,
        stop_loss_suggestion: null,
        take_profit_suggestion: null,
      },
    })),
  ];

  // Normalize and sort signals by timestamp (newest first)
  const normalizedSignals = allSignalsRaw
    .map((signal) => {
      const dateObj = new Date(signal.timestamp);
      const isValidDate = !isNaN(dateObj.getTime());

      return {
        ...signal,
        symbol: (signal.symbol || "unknown").toUpperCase(),
        timestamp:
          typeof signal.timestamp === "string" && isValidDate
            ? signal.timestamp
            : isValidDate
            ? dateObj.toISOString()
            : new Date().toISOString(),
        timestampMs: isValidDate ? dateObj.getTime() : Date.now(),
      };
    })
    .sort((a, b) => b.timestampMs - a.timestampMs);

  // Filter to show only the most recent signal per token pair
  const uniqueSignalsMap = new Map<string, typeof normalizedSignals[0]>();
  normalizedSignals.forEach((signal) => {
    const symbol = signal.symbol;
    if (!uniqueSignalsMap.has(symbol)) {
      uniqueSignalsMap.set(symbol, signal);
    }
  });

  const allSignals = Array.from(uniqueSignalsMap.values()).sort(
    (a, b) => b.timestampMs - a.timestampMs
  );

  const formatSignalForDisplay = (signal: CombinedSignal): FormattedSignal => ({
    id: `${signal.symbol}-${signal.timestamp}-${signal.source}`,
    signal: (signal.signal || "NEUTRAL").toUpperCase() as "LONG" | "SHORT" | "NEUTRAL",
    confidence: signal.confidence || 0,
    timestamp: new Date(signal.timestamp).toLocaleString(),
    pair: signal.symbol ? signal.symbol.replace("USDT", "/USDT") : "N/A",
    reason: signal.reasoning || `${signal.source} signal`,
    active: Date.now() - new Date(signal.timestamp).getTime() < 30 * 60 * 1000,
    marketAnalysis: signal.market_analysis,
    riskAssessment: signal.risk_assessment,
    strategyScores: signal.strategy_scores,
    source: signal.source,
    isWebSocket: signal.source === "websocket",
  });

  return (
    <Card className="h-full flex flex-col w-full">
      <CardHeader className="flex-shrink-0">
        <CardTitle className="text-lg flex items-center gap-2">
          AI Trading Signals
          <Badge
            variant="outline"
            className="bg-info/10 text-info border-info/20"
          >
            <div className="w-2 h-2 bg-info rounded-full mr-2 animate-pulse"></div>
            Live Analysis
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-4 flex-1 overflow-y-auto min-h-0">
        {/* WebSocket Connection Status */}
        <div className="flex items-center gap-4 mb-4">
          <div className="flex items-center gap-2">
            {wsState.isConnected ? (
              <Wifi className="h-4 w-4 text-success" />
            ) : (
              <WifiOff className="h-4 w-4 text-destructive" />
            )}
            <span className="text-xs text-muted-foreground">
              WebSocket: {wsState.isConnected ? "Connected" : "Disconnected"}
            </span>
          </div>
        </div>

        {/* Error Display */}
        {(aiState.error || wsState.error) && (
          <div className="p-4 rounded-lg bg-destructive/10 border border-destructive/20 flex items-center gap-2">
            <AlertCircle className="h-4 w-4 text-destructive" />
            <span className="text-sm text-destructive">
              {aiState.error || wsState.error}
            </span>
            <Button
              variant="outline"
              size="sm"
              onClick={clearError}
              className="ml-auto"
            >
              Dismiss
            </Button>
          </div>
        )}

        {/* Loading State */}
        {aiState.isLoading && (
          <div className="p-4 rounded-lg bg-muted/20 border border-muted/40 flex items-center gap-2">
            <RefreshCw className="h-4 w-4 animate-spin" />
            <span className="text-sm text-muted-foreground">
              Analyzing market signals...
            </span>
          </div>
        )}

        {/* AI Service Info */}
        {aiState.serviceInfo && (
          <div className="p-3 rounded-lg bg-info/10 border border-info/20 flex items-center gap-2">
            <Zap className="h-4 w-4 text-info" />
            <span className="text-sm text-info">
              {aiState.serviceInfo.service_name} v{aiState.serviceInfo.version}{" "}
              • Model: {aiState.serviceInfo.model_version}
            </span>
            {aiState.lastUpdate && (
              <span className="text-xs text-muted-foreground ml-auto">
                Last updated:{" "}
                {new Date(aiState.lastUpdate).toLocaleTimeString()}
              </span>
            )}
          </div>
        )}

        {/* No Signals Message */}
        {!aiState.isLoading && allSignals.length === 0 && (
          <div className="p-8 text-center text-muted-foreground">
            <TrendingUp className="h-8 w-8 mx-auto mb-2 opacity-50" />
            <p>No AI signals available yet</p>
            <p className="text-sm">Analysis will start automatically</p>
          </div>
        )}

        {/* Signals List */}
        {allSignals.map((signalData) => {
          const signal = formatSignalForDisplay(signalData);
          return (
            <Dialog key={signal.id}>
              <DialogTrigger asChild>
                <div>
                  <SignalCard signal={signal} onClick={() => {}} />
                </div>
              </DialogTrigger>
              <DetailedSignalDialog signal={signal} />
            </Dialog>
          );
        })}
      </CardContent>
    </Card>
  );
}
