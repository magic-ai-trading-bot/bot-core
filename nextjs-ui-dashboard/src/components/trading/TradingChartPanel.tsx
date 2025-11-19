import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { RefreshCw, TrendingUp } from "lucide-react";

interface Signal {
  signal?: string;
  symbol?: string;
  confidence: number;
  timestamp: string | number;
  reasoning?: string;
}

interface TradingChartPanelProps {
  recentSignals: Signal[];
  isLoading: boolean;
  formatDate: (date: Date | string | number) => string;
  refreshAISignals: () => void;
}

export function TradingChartPanel({
  recentSignals,
  isLoading,
  formatDate,
  refreshAISignals,
}: TradingChartPanelProps) {
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          Tín hiệu AI gần đây
          <Badge
            variant="outline"
            className="bg-info/10 text-info border-info/20"
          >
            <div className="w-2 h-2 bg-info rounded-full mr-2 animate-pulse"></div>
            Live Analysis
          </Badge>
        </CardTitle>
        <div className="text-sm text-muted-foreground">
          GPT-4 Trading AI v2.0.0 • Model: gpt-3.5-turbo
          <span className="ml-2">• WebSocket real-time signals</span>
        </div>
      </CardHeader>
      <CardContent>
        {isLoading && (
          <div className="p-4 rounded-lg bg-muted/20 border border-muted/40 flex items-center gap-2 mb-4">
            <RefreshCw className="h-4 w-4 animate-spin" />
            <span className="text-sm text-muted-foreground">
              Đang phân tích tín hiệu thị trường...
            </span>
          </div>
        )}

        {recentSignals && recentSignals.length > 0 ? (
          <div className="space-y-4">
            {recentSignals.map((signal, index) => {
              const isActive =
                Date.now() - new Date(signal.timestamp).getTime() <
                30 * 60 * 1000;
              const isLong = signal.signal?.toLowerCase() === "long";
              const isShort = signal.signal?.toLowerCase() === "short";

              return (
                <div
                  key={`${signal.symbol}-${signal.timestamp}-websocket`}
                  className={`p-4 rounded-lg border transition-all duration-200 ${
                    isActive
                      ? "bg-secondary/50 border-primary/20 shadow-primary/5"
                      : "bg-muted/20 border-muted/40"
                  }`}
                >
                  <div className="flex justify-between items-start mb-3">
                    <div className="flex items-center gap-3">
                      <Badge
                        className={
                          isLong
                            ? "bg-profit text-profit-foreground"
                            : isShort
                            ? "bg-loss text-loss-foreground"
                            : "bg-warning text-warning-foreground"
                        }
                      >
                        {signal.signal?.toUpperCase() || "NEUTRAL"}
                      </Badge>
                      <span className="font-semibold">
                        {signal.symbol?.replace("USDT", "/USDT") ||
                          "N/A"}
                      </span>
                      {isActive && (
                        <Badge
                          variant="outline"
                          className="bg-profit/10 text-profit border-profit/20 text-xs"
                        >
                          ACTIVE
                        </Badge>
                      )}
                      <Badge
                        variant="outline"
                        className="text-xs capitalize"
                      >
                        websocket
                      </Badge>
                    </div>
                    <div className="text-right">
                      <div
                        className={`font-bold text-lg ${
                          signal.confidence >= 0.8
                            ? "text-profit"
                            : signal.confidence >= 0.6
                            ? "text-warning"
                            : "text-loss"
                        }`}
                      >
                        {((signal.confidence || 0) * 100).toFixed(0)}%
                      </div>
                      <div className="text-xs text-muted-foreground">
                        Confidence
                      </div>
                    </div>
                  </div>

                  <div className="space-y-2">
                    <p className="text-sm text-foreground">
                      {signal.reasoning || "WebSocket real-time signal"}
                    </p>
                    <div className="flex justify-between items-center text-xs text-muted-foreground">
                      <span>{formatDate(signal.timestamp)}</span>
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
                        style={{
                          width: `${(signal.confidence || 0) * 100}%`,
                        }}
                      ></div>
                    </div>
                  </div>
                </div>
              );
            })}
          </div>
        ) : (
          <div className="flex items-center justify-center h-32 text-muted-foreground">
            <div className="text-center">
              <TrendingUp className="h-8 w-8 mx-auto mb-2 opacity-50" />
              <p>Chưa có tín hiệu AI</p>
              <p className="text-sm">
                Tín hiệu sẽ xuất hiện tự động khi có phân tích mới
              </p>
            </div>
          </div>
        )}
      </CardContent>
    </Card>
  );
}
