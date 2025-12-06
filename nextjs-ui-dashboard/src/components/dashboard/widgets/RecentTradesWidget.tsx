/**
 * RecentTradesWidget Component
 *
 * Latest executed trades with PnL indicators.
 * Real-time updates via WebSocket.
 */

import { GlassCardWithHeader } from '@/components/ui/GlassCard';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { colors, getModeColor } from '@/styles';
import { ArrowUpRight, ArrowDownRight, History } from 'lucide-react';
import { cn } from '@/lib/utils';
import { TradeHistory } from '@/services/api';
import { AnimatedCurrency } from '@/components/ui/AnimatedNumber';

interface RecentTradesWidgetProps {
  trades?: TradeHistory[];
  isLoading?: boolean;
}

export function RecentTradesWidget({ trades = [], isLoading = false }: RecentTradesWidgetProps) {
  const { mode } = useTradingModeContext();
  const accentColor = getModeColor(mode, 'accent');

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
        title="Recent Trades"
        mode={mode}
        className="h-full"
      >
        <div className="space-y-3">
          {[...Array(5)].map((_, i) => (
            <div key={i} className="h-16 bg-slate-700/20 rounded animate-pulse"></div>
          ))}
        </div>
      </GlassCardWithHeader>
    );
  }

  return (
    <GlassCardWithHeader
      title="Recent Trades"
      mode={mode}
      className="h-full"
      action={
        <div className="flex items-center gap-2">
          <History className="w-4 h-4 text-gray-400" />
          <span className="text-xs font-medium text-gray-400">
            {trades.length}
          </span>
        </div>
      }
    >
      <div className="space-y-2">
        {trades.length === 0 ? (
          <div className="text-center py-8 text-gray-400">
            <History className="w-8 h-8 mx-auto mb-2 opacity-50" />
            <p className="text-sm">No trades yet</p>
          </div>
        ) : (
          trades.slice(0, 5).map((trade) => {
            const isProfitable = (trade.pnl || 0) >= 0;
            const isOpen = trade.status === 'open';

            return (
              <div
                key={trade.id}
                className="flex items-center justify-between p-3 rounded-lg bg-slate-800/50 hover:bg-slate-800/70 transition-colors"
              >
                {/* Left: Trade info */}
                <div className="flex items-center gap-3">
                  {/* Side icon */}
                  <div
                    className={cn(
                      'p-2 rounded-lg',
                      trade.side === 'BUY' ? 'bg-emerald-500/10' : 'bg-red-500/10'
                    )}
                  >
                    {trade.side === 'BUY' ? (
                      <ArrowUpRight
                        className="w-4 h-4"
                        style={{ color: colors.profit }}
                      />
                    ) : (
                      <ArrowDownRight
                        className="w-4 h-4"
                        style={{ color: colors.loss }}
                      />
                    )}
                  </div>

                  {/* Symbol and details */}
                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-semibold text-gray-100">
                        {trade.symbol.replace('USDT', '')}
                      </span>
                      {isOpen && (
                        <span
                          className="text-xs px-2 py-0.5 rounded-full font-medium"
                          style={{
                            backgroundColor: mode === 'paper' ? 'rgba(14, 165, 233, 0.1)' : 'rgba(239, 68, 68, 0.1)',
                            color: accentColor,
                          }}
                        >
                          Open
                        </span>
                      )}
                    </div>
                    <div className="flex items-center gap-2 mt-1">
                      <span className="text-xs text-gray-400">
                        {trade.quantity} @ ${trade.entry_price.toFixed(2)}
                      </span>
                      {!isOpen && (
                        <>
                          <span className="text-xs text-gray-500">•</span>
                          <span className="text-xs text-gray-500">
                            {formatTimestamp(trade.exit_time || trade.entry_time)}
                          </span>
                        </>
                      )}
                    </div>
                  </div>
                </div>

                {/* Right: PnL */}
                {trade.pnl !== undefined && (
                  <div className="text-right">
                    <AnimatedCurrency
                      value={trade.pnl}
                      showSign
                      className="text-sm font-bold"
                    />
                  </div>
                )}
              </div>
            );
          })
        )}
      </div>
    </GlassCardWithHeader>
  );
}
