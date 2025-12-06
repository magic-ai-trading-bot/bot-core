/**
 * PortfolioSummaryCard Component
 *
 * Portfolio summary with balance, PnL, and mode-aware styling.
 * Real-time updates via WebSocket.
 */

import { GlassCard } from '@/components/ui/GlassCard';
import { AnimatedCurrency, AnimatedPercentage } from '@/components/ui/AnimatedNumber';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { colors, getModeColor } from '@/styles';
import { TrendingUp, TrendingDown } from 'lucide-react';
import { cn } from '@/lib/utils';

interface PortfolioSummaryCardProps {
  balance: number;
  pnl: number;
  pnlPercentage: number;
  currency?: string;
  isLoading?: boolean;
}

export function PortfolioSummaryCard({
  balance,
  pnl,
  pnlPercentage,
  currency = 'USD',
  isLoading = false,
}: PortfolioSummaryCardProps) {
  const { mode } = useTradingModeContext();

  const isProfitable = pnl >= 0;
  const accentColor = getModeColor(mode, 'accent');

  if (isLoading) {
    return (
      <GlassCard mode={mode} className="animate-pulse">
        <div className="h-32 bg-slate-700/20 rounded"></div>
      </GlassCard>
    );
  }

  return (
    <GlassCard mode={mode} className="relative overflow-hidden">
      {/* Background accent glow */}
      <div
        className="absolute top-0 right-0 w-32 h-32 rounded-full blur-3xl opacity-10 pointer-events-none"
        style={{ backgroundColor: accentColor }}
      />

      <div className="relative z-10">
        {/* Title */}
        <div className="flex items-center justify-between mb-4">
          <h3 className="text-sm font-medium text-gray-400 uppercase tracking-wide">
            Total Balance
          </h3>
          <div
            className="px-2 py-1 rounded-full text-xs font-semibold"
            style={{
              backgroundColor: mode === 'paper' ? 'rgba(14, 165, 233, 0.1)' : 'rgba(239, 68, 68, 0.1)',
              color: accentColor,
            }}
          >
            {mode === 'paper' ? 'Paper' : 'Real'}
          </div>
        </div>

        {/* Balance */}
        <div className="mb-4">
          <div className="text-4xl font-bold text-gray-100">
            <AnimatedCurrency
              value={balance}
              currency={currency}
              showSign={false}
            />
          </div>
        </div>

        {/* PnL */}
        <div className="flex items-center gap-3">
          {/* Icon */}
          <div
            className={cn(
              'p-2 rounded-lg',
              isProfitable ? 'bg-emerald-500/10' : 'bg-red-500/10'
            )}
          >
            {isProfitable ? (
              <TrendingUp className="w-5 h-5" style={{ color: colors.profit }} />
            ) : (
              <TrendingDown className="w-5 h-5" style={{ color: colors.loss }} />
            )}
          </div>

          {/* PnL values */}
          <div>
            <div className="text-lg font-semibold">
              <AnimatedCurrency value={pnl} showSign />
            </div>
            <div className="text-sm">
              <AnimatedPercentage value={pnlPercentage} showSign />
            </div>
          </div>
        </div>
      </div>
    </GlassCard>
  );
}
