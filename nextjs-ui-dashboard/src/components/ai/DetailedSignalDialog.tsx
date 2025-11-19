import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Dialog, DialogTrigger } from "@/components/ui/dialog";
import { DialogContent, DialogHeader, DialogTitle } from "@/components/ui/dialog";
import {
  BarChart3,
  Target,
  Activity,
  Shield,
  ArrowUp,
  ArrowDown,
  Info,
  TrendingUp,
} from "lucide-react";
import { FormattedSignal } from "./types";
import { StrategyExplanationDialog } from "./StrategyExplanation";

interface DetailedSignalDialogProps {
  signal: FormattedSignal;
}

export function DetailedSignalDialog({ signal }: DetailedSignalDialogProps) {
  return (
    <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
      <DialogHeader>
        <DialogTitle className="flex items-center gap-2">
          <BarChart3 className="h-5 w-5" />
          Detailed AI Analysis: {signal.pair}
          <Badge
            className={
              signal.signal === "LONG"
                ? "bg-profit"
                : signal.signal === "SHORT"
                ? "bg-loss"
                : "bg-warning"
            }
          >
            {signal.signal}
          </Badge>
        </DialogTitle>
      </DialogHeader>

      <div className="space-y-6">
        {/* Signal Overview */}
        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-2">
                <Target className="h-4 w-4 text-blue-500" />
                <span className="font-medium">Signal Strength</span>
              </div>
              <div className="text-2xl font-bold text-blue-500">
                {(signal.confidence * 100).toFixed(1)}%
              </div>
              <div className="text-sm text-muted-foreground">
                Confidence Level
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-2">
                {signal.signal === "LONG" ? (
                  <ArrowUp className="h-4 w-4 text-profit" />
                ) : signal.signal === "SHORT" ? (
                  <ArrowDown className="h-4 w-4 text-loss" />
                ) : (
                  <Activity className="h-4 w-4 text-warning" />
                )}
                <span className="font-medium">Recommendation</span>
              </div>
              <div
                className={`text-2xl font-bold ${
                  signal.signal === "LONG"
                    ? "text-profit"
                    : signal.signal === "SHORT"
                    ? "text-loss"
                    : "text-warning"
                }`}
              >
                {signal.signal === "LONG"
                  ? "BUY (LONG)"
                  : signal.signal === "SHORT"
                  ? "SELL (SHORT)"
                  : "HOLD"}
              </div>
              <div className="text-sm text-muted-foreground">
                {signal.signal === "LONG"
                  ? "🟢 Go Long - Buy Position"
                  : signal.signal === "SHORT"
                  ? "🔴 Go Short - Sell Position"
                  : "🟡 Wait - No Action"}
              </div>
            </CardContent>
          </Card>

          <Card>
            <CardContent className="p-4">
              <div className="flex items-center gap-2 mb-2">
                <Shield className="h-4 w-4 text-orange-500" />
                <span className="font-medium">Risk Level</span>
              </div>
              <div className="text-2xl font-bold text-orange-500">
                {signal.riskAssessment?.overall_risk || "Medium"}
              </div>
              <div className="text-sm text-muted-foreground">Overall Risk</div>
            </CardContent>
          </Card>
        </div>

        {/* Market Analysis */}
        {signal.marketAnalysis && (
          <Card>
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <TrendingUp className="h-5 w-5" />
                Market Analysis
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Trend Direction:
                  </span>
                  <p className="text-lg font-semibold">
                    {signal.marketAnalysis.trend_direction || "Sideways"}
                  </p>
                </div>
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Trend Strength:
                  </span>
                  <p className="text-lg font-semibold">
                    {(
                      (signal.marketAnalysis.trend_strength || 0.5) * 100
                    ).toFixed(1)}
                    %
                  </p>
                </div>
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Volatility:
                  </span>
                  <p className="text-lg font-semibold">
                    {signal.marketAnalysis.volatility_level || "Medium"}
                  </p>
                </div>
                <div>
                  <span className="text-sm font-medium text-muted-foreground">
                    Volume Analysis:
                  </span>
                  <p className="text-lg font-semibold">
                    {signal.marketAnalysis.volume_analysis || "Normal"}
                  </p>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Strategy Scores */}
        {signal.strategyScores &&
          Object.keys(signal.strategyScores).length > 0 && (
            <Card>
              <CardHeader>
                <CardTitle className="text-lg flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  Strategy Analysis
                  <Badge variant="outline" className="text-xs">
                    Click để xem chi tiết
                  </Badge>
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3">
                  {Object.entries(signal.strategyScores).map(
                    ([strategy, score]) => (
                      <Dialog key={strategy}>
                        <DialogTrigger asChild>
                          <div className="space-y-2 p-3 rounded-lg hover:bg-muted/50 cursor-pointer transition-colors border border-transparent hover:border-muted-foreground/20">
                            <div className="flex justify-between items-center">
                              <div className="flex items-center gap-2">
                                <span className="text-sm font-medium">
                                  {strategy}
                                </span>
                                <Info className="h-3 w-3 text-muted-foreground" />
                              </div>
                              <span className="text-sm font-bold">
                                {((score as number) * 100).toFixed(1)}%
                              </span>
                            </div>
                            <div className="w-full bg-muted rounded-full h-2">
                              <div
                                className={`h-2 rounded-full transition-all duration-500 ${
                                  (score as number) >= 0.7
                                    ? "bg-profit"
                                    : (score as number) >= 0.5
                                    ? "bg-warning"
                                    : "bg-loss"
                                }`}
                                style={{ width: `${(score as number) * 100}%` }}
                              />
                            </div>
                          </div>
                        </DialogTrigger>
                        <StrategyExplanationDialog strategyName={strategy} />
                      </Dialog>
                    )
                  )}
                </div>
              </CardContent>
            </Card>
          )}

        {/* Risk Assessment */}
        {signal.riskAssessment && (
          <Card>
            <CardHeader>
              <CardTitle className="text-lg flex items-center gap-2">
                <Shield className="h-5 w-5" />
                Risk Assessment
              </CardTitle>
            </CardHeader>
            <CardContent className="space-y-4">
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-4">
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Technical Risk:
                    </span>
                    <p className="text-lg font-semibold">
                      {(
                        (signal.riskAssessment.technical_risk || 0.5) * 100
                      ).toFixed(1)}
                      %
                    </p>
                  </div>
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Position Size:
                    </span>
                    <p className="text-lg font-semibold">
                      {(
                        (signal.riskAssessment.recommended_position_size ||
                          0.02) * 100
                      ).toFixed(1)}
                      %
                    </p>
                  </div>
                </div>
                <div className="space-y-4">
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Market Risk:
                    </span>
                    <p className="text-lg font-semibold">
                      {(
                        (signal.riskAssessment.market_risk || 0.5) * 100
                      ).toFixed(1)}
                      %
                    </p>
                  </div>
                  <div>
                    <span className="text-sm font-medium text-muted-foreground">
                      Source:
                    </span>
                    <p className="text-lg font-semibold capitalize">
                      {signal.source}
                    </p>
                  </div>
                </div>
              </div>
            </CardContent>
          </Card>
        )}

        {/* Analysis Reasoning */}
        <Card>
          <CardHeader>
            <CardTitle className="text-lg flex items-center gap-2">
              <Info className="h-5 w-5" />
              Analysis Reasoning
            </CardTitle>
          </CardHeader>
          <CardContent>
            <p className="text-sm leading-relaxed">{signal.reason}</p>
            <div className="mt-4 pt-4 border-t text-xs text-muted-foreground">
              <p>Generated: {signal.timestamp}</p>
              <p>Status: {signal.active ? "Active" : "Expired"}</p>
            </div>
          </CardContent>
        </Card>
      </div>
    </DialogContent>
  );
}
