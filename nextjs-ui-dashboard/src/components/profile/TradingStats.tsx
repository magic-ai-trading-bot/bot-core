/**
 * TradingStats Component
 *
 * Display trading statistics with animated counters and time period selector.
 */

import { useState } from 'react';
import { GlassCard } from '@/components/ui/GlassCard';
import { AnimatedNumber, AnimatedPercentage, AnimatedCurrency } from '@/components/ui/AnimatedNumber';
import { Button } from '@/components/ui/button';
import { TrendingUp, TrendingDown, Activity, Target } from 'lucide-react';
import { cn } from '@/lib/utils';

interface TradingStatsProps {
  stats?: {
    totalTrades: number;
    winRate: number;
    totalPnL: number;
    bestTrade: number;
    avgProfit: number;
    avgLoss: number;
    profitFactor: number;
    sharpeRatio: number;
  };
}

const DEFAULT_STATS = {
  totalTrades: 342,
  winRate: 64.5,
  totalPnL: 15847.32,
  bestTrade: 2847.50,
  avgProfit: 185.23,
  avgLoss: -92.15,
  profitFactor: 2.01,
  sharpeRatio: 1.85,
};

type TimePeriod = '7d' | '30d' | '90d' | 'all';

export function TradingStats({ stats = DEFAULT_STATS }: TradingStatsProps) {
  const [period, setPeriod] = useState<TimePeriod>('30d');

  const periods: { value: TimePeriod; label: string }[] = [
    { value: '7d', label: '7 Days' },
    { value: '30d', label: '30 Days' },
    { value: '90d', label: '90 Days' },
    { value: 'all', label: 'All Time' },
  ];

  return (
    <div className="space-y-6">
      {/* Time Period Selector */}
      <div className="flex items-center justify-between">
        <h3 className="text-xl font-semibold text-gray-100">Trading Performance</h3>
        <div className="flex gap-2">
          {periods.map((p) => (
            <Button
              key={p.value}
              size="sm"
              variant={period === p.value ? 'default' : 'outline'}
              onClick={() => setPeriod(p.value)}
              className={cn(
                period === p.value
                  ? 'bg-sky-600 hover:bg-sky-700'
                  : 'border-slate-700 text-gray-400 hover:text-gray-100'
              )}
            >
              {p.label}
            </Button>
          ))}
        </div>
      </div>

      {/* Main Stats Grid */}
      <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
        {/* Total Trades */}
        <GlassCard className="text-center">
          <div className="flex items-center justify-center gap-2 mb-2">
            <Activity className="w-5 h-5 text-sky-500" />
            <span className="text-sm text-gray-400">Total Trades</span>
          </div>
          <AnimatedNumber
            value={stats.totalTrades}
            decimals={0}
            className="text-4xl"
            customColor="#0EA5E9"
          />
        </GlassCard>

        {/* Win Rate */}
        <GlassCard className="text-center">
          <div className="flex items-center justify-center gap-2 mb-2">
            <Target className="w-5 h-5 text-green-500" />
            <span className="text-sm text-gray-400">Win Rate</span>
          </div>
          <AnimatedPercentage
            value={stats.winRate}
            decimals={1}
            showSign={false}
            className="text-4xl"
          />
        </GlassCard>

        {/* Total P&L */}
        <GlassCard className="text-center">
          <div className="flex items-center justify-center gap-2 mb-2">
            {stats.totalPnL >= 0 ? (
              <TrendingUp className="w-5 h-5 text-green-500" />
            ) : (
              <TrendingDown className="w-5 h-5 text-red-500" />
            )}
            <span className="text-sm text-gray-400">Total P&L</span>
          </div>
          <AnimatedCurrency
            value={stats.totalPnL}
            showSign={true}
            className="text-4xl"
          />
        </GlassCard>

        {/* Best Trade */}
        <GlassCard className="text-center">
          <div className="flex items-center justify-center gap-2 mb-2">
            <TrendingUp className="w-5 h-5 text-green-500" />
            <span className="text-sm text-gray-400">Best Trade</span>
          </div>
          <AnimatedCurrency
            value={stats.bestTrade}
            showSign={false}
            className="text-4xl"
          />
        </GlassCard>
      </div>

      {/* Advanced Metrics */}
      <GlassCard>
        <h4 className="text-lg font-semibold text-gray-100 mb-4">Advanced Metrics</h4>
        <div className="grid grid-cols-2 md:grid-cols-4 gap-6">
          <div>
            <p className="text-xs text-gray-400 mb-1">Avg Profit</p>
            <AnimatedCurrency
              value={stats.avgProfit}
              showSign={false}
              className="text-2xl"
            />
          </div>
          <div>
            <p className="text-xs text-gray-400 mb-1">Avg Loss</p>
            <AnimatedCurrency
              value={stats.avgLoss}
              showSign={false}
              className="text-2xl"
            />
          </div>
          <div>
            <p className="text-xs text-gray-400 mb-1">Profit Factor</p>
            <AnimatedNumber
              value={stats.profitFactor}
              decimals={2}
              className="text-2xl"
              colorMode="profit-loss"
            />
          </div>
          <div>
            <p className="text-xs text-gray-400 mb-1">Sharpe Ratio</p>
            <AnimatedNumber
              value={stats.sharpeRatio}
              decimals={2}
              className="text-2xl"
              colorMode="profit-loss"
            />
          </div>
        </div>
      </GlassCard>
    </div>
  );
}
