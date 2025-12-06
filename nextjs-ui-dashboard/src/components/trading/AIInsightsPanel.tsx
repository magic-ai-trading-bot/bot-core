/**
 * AI Insights Panel
 *
 * Display latest AI trading signals with confidence scores
 * and recommended actions.
 */

import { Card, CardContent, CardHeader, CardTitle } from '@/components/ui/card';
import { Badge } from '@/components/ui/badge';
import { Button } from '@/components/ui/button';
import { ScrollArea } from '@/components/ui/scroll-area';
import { formatDistanceToNow } from 'date-fns';
import type { AISignal } from '@/hooks/usePaperTrading';

interface AIInsightsPanelProps {
  signals: AISignal[];
  isLoading?: boolean;
  onRefresh?: () => void;
}

export function AIInsightsPanel({
  signals,
  isLoading = false,
  onRefresh,
}: AIInsightsPanelProps) {
  const getSignalColor = (signal: string) => {
    switch (signal.toLowerCase()) {
      case 'strong_buy':
        return 'bg-green-600 text-white';
      case 'buy':
        return 'bg-green-500 text-white';
      case 'strong_sell':
        return 'bg-red-600 text-white';
      case 'sell':
        return 'bg-red-500 text-white';
      case 'hold':
      default:
        return 'bg-gray-500 text-white';
    }
  };

  const getConfidenceColor = (confidence: number) => {
    if (confidence >= 0.8) return 'text-green-500';
    if (confidence >= 0.6) return 'text-yellow-500';
    return 'text-red-500';
  };

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <CardTitle className="text-sm">AI Insights</CardTitle>
          <Button
            size="sm"
            variant="outline"
            onClick={onRefresh}
            disabled={isLoading}
          >
            Refresh
          </Button>
        </div>
      </CardHeader>

      <CardContent>
        {isLoading ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            Loading AI insights...
          </div>
        ) : signals.length === 0 ? (
          <div className="py-8 text-center text-sm text-muted-foreground">
            No signals available
          </div>
        ) : (
          <ScrollArea className="h-[400px]">
            <div className="space-y-3">
              {signals.map((signal) => (
                <div
                  key={signal.id}
                  className="rounded-lg border bg-card p-3 text-sm"
                >
                  {/* Header */}
                  <div className="mb-2 flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="font-semibold">{signal.symbol}</span>
                      <Badge className={getSignalColor(signal.signal)}>
                        {signal.signal.replace('_', ' ')}
                      </Badge>
                    </div>
                    <span className="text-xs text-muted-foreground">
                      {formatDistanceToNow(new Date(signal.timestamp), {
                        addSuffix: true,
                      })}
                    </span>
                  </div>

                  {/* Confidence */}
                  <div className="mb-2 flex items-center justify-between text-xs">
                    <span className="text-muted-foreground">Confidence</span>
                    <span
                      className={`font-bold ${getConfidenceColor(signal.confidence)}`}
                    >
                      {(signal.confidence * 100).toFixed(1)}%
                    </span>
                  </div>

                  {/* Reasoning */}
                  <p className="mb-2 text-xs text-muted-foreground">
                    {signal.reasoning}
                  </p>

                  {/* Market Analysis */}
                  {signal.market_analysis && (
                    <div className="mb-2 rounded-md bg-muted/50 p-2 text-xs">
                      <p className="mb-1 font-semibold">Market Analysis</p>
                      <div className="grid grid-cols-2 gap-1">
                        <div>
                          <span className="text-muted-foreground">Trend:</span>{' '}
                          <span>{signal.market_analysis.trend_direction}</span>
                        </div>
                        <div>
                          <span className="text-muted-foreground">Strength:</span>{' '}
                          <span>
                            {(signal.market_analysis.trend_strength * 100).toFixed(0)}
                            %
                          </span>
                        </div>
                        <div className="col-span-2">
                          <span className="text-muted-foreground">Volatility:</span>{' '}
                          <span>{signal.market_analysis.volatility_level}</span>
                        </div>
                      </div>
                    </div>
                  )}

                  {/* Risk Assessment */}
                  {signal.risk_assessment && (
                    <div className="rounded-md bg-muted/50 p-2 text-xs">
                      <p className="mb-1 font-semibold">Risk Assessment</p>
                      <div className="space-y-1">
                        <div>
                          <span className="text-muted-foreground">Overall Risk:</span>{' '}
                          <Badge variant="outline" className="ml-1 text-xs">
                            {signal.risk_assessment.overall_risk}
                          </Badge>
                        </div>
                        {signal.risk_assessment.stop_loss_suggestion && (
                          <div>
                            <span className="text-muted-foreground">Stop Loss:</span>{' '}
                            <span className="text-red-500">
                              $
                              {signal.risk_assessment.stop_loss_suggestion.toFixed(2)}
                            </span>
                          </div>
                        )}
                        {signal.risk_assessment.take_profit_suggestion && (
                          <div>
                            <span className="text-muted-foreground">
                              Take Profit:
                            </span>{' '}
                            <span className="text-green-500">
                              $
                              {signal.risk_assessment.take_profit_suggestion.toFixed(
                                2
                              )}
                            </span>
                          </div>
                        )}
                      </div>
                    </div>
                  )}

                  {/* Strategy Scores (if available) */}
                  {signal.strategy_scores &&
                    Object.keys(signal.strategy_scores).length > 0 && (
                      <div className="mt-2 border-t pt-2">
                        <p className="mb-1 text-xs font-semibold">Strategy Scores</p>
                        <div className="grid grid-cols-2 gap-1 text-xs">
                          {Object.entries(signal.strategy_scores).map(
                            ([strategy, score]) => (
                              <div key={strategy}>
                                <span className="text-muted-foreground">
                                  {strategy}:
                                </span>{' '}
                                <span>{(score as number).toFixed(2)}</span>
                              </div>
                            )
                          )}
                        </div>
                      </div>
                    )}
                </div>
              ))}
            </div>
          </ScrollArea>
        )}
      </CardContent>
    </Card>
  );
}
