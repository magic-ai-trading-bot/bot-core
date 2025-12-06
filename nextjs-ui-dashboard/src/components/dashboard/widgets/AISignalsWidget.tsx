/**
 * AISignalsWidget Component
 *
 * Latest AI trading signals with confidence scores.
 * Real-time updates via WebSocket.
 */

import { GlassCardWithHeader } from '@/components/ui/GlassCard';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { colors, getModeColor } from '@/styles';
import { TrendingUp, TrendingDown, Minus, Brain } from 'lucide-react';
import { cn } from '@/lib/utils';
import { AISignal } from '@/services/api';

interface AISignalsWidgetProps {
  signals?: AISignal[];
  isLoading?: boolean;
}

export function AISignalsWidget({ signals = [], isLoading = false }: AISignalsWidgetProps) {
  const { mode } = useTradingModeContext();
  const accentColor = getModeColor(mode, 'accent');

  const getSignalIcon = (signal: 'long' | 'short' | 'neutral') => {
    switch (signal) {
      case 'long':
        return <TrendingUp className="w-4 h-4" />;
      case 'short':
        return <TrendingDown className="w-4 h-4" />;
      case 'neutral':
        return <Minus className="w-4 h-4" />;
    }
  };

  const getSignalColor = (signal: 'long' | 'short' | 'neutral') => {
    switch (signal) {
      case 'long':
        return colors.profit;
      case 'short':
        return colors.loss;
      case 'neutral':
        return colors.neutral;
    }
  };

  const formatTimestamp = (timestamp: string) => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / 60000);

    if (diffMins < 1) return 'Just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    const diffHours = Math.floor(diffMins / 60);
    if (diffHours < 24) return `${diffHours}h ago`;
    return `${Math.floor(diffHours / 24)}d ago`;
  };

  if (isLoading) {
    return (
      <GlassCardWithHeader
        title="AI Signals"
        mode={mode}
        className="h-full"
      >
        <div className="space-y-3">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-16 bg-slate-700/20 rounded animate-pulse"></div>
          ))}
        </div>
      </GlassCardWithHeader>
    );
  }

  return (
    <GlassCardWithHeader
      title="AI Signals"
      mode={mode}
      className="h-full"
      action={
        <div className="flex items-center gap-2">
          <Brain className="w-4 h-4" style={{ color: accentColor }} />
          <span className="text-xs font-medium" style={{ color: accentColor }}>
            Live
          </span>
        </div>
      }
    >
      <div className="space-y-3">
        {signals.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            <Brain className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No signals available</p>
          </div>
        ) : (
          signals.slice(0, 5).map((signal, index) => (
            <div
              key={`${signal.symbol}-${signal.timestamp}-${index}`}
              className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 hover:bg-slate-800/70 transition-colors"
            >
              {/* Left: Signal info */}
              <div className="flex items-center gap-3">
                {/* Signal icon */}
                <div
                  className="p-2 rounded-lg"
                  style={{
                    backgroundColor: `${getSignalColor(signal.signal)}20`,
                    color: getSignalColor(signal.signal),
                  }}
                >
                  {getSignalIcon(signal.signal)}
                </div>

                {/* Symbol and time */}
                <div>
                  <div className="flex items-center gap-2">
                    <span className="font-semibold text-gray-100">
                      {signal.symbol.replace('USDT', '')}
                    </span>
                    <span className="text-xs text-gray-500">
                      {signal.timeframe}
                    </span>
                  </div>
                  <div className="flex items-center gap-2 mt-1">
                    <span className="text-xs text-gray-400 capitalize">
                      {signal.signal}
                    </span>
                    <span className="text-xs text-gray-500">•</span>
                    <span className="text-xs text-gray-500">
                      {formatTimestamp(signal.timestamp)}
                    </span>
                  </div>
                </div>
              </div>

              {/* Right: Confidence */}
              <div className="text-right">
                <div
                  className="text-sm font-bold"
                  style={{ color: accentColor }}
                >
                  {(signal.confidence * 100).toFixed(0)}%
                </div>
                <div className="text-xs text-gray-400">
                  confidence
                </div>
              </div>
            </div>
          ))
        )}
      </div>
    </GlassCardWithHeader>
  );
}
