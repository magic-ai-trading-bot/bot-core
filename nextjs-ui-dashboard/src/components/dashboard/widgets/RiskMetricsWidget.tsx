/**
 * RiskMetricsWidget Component
 *
 * Risk exposure metrics with visual gauges.
 * Shows daily loss limit, exposure, and open positions.
 */

import { GlassCardWithHeader } from '@/components/ui/GlassCard';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { colors, getModeColor } from '@/styles';
import { Shield, AlertTriangle, TrendingUp } from 'lucide-react';
import { cn } from '@/lib/utils';

interface RiskMetrics {
  exposurePercent: number; // 0-100
  dailyLossPercent: number; // 0-100 (% of daily loss limit used)
  openPositions: number;
  maxPositions?: number;
}

interface RiskMetricsWidgetProps {
  metrics?: RiskMetrics;
  isLoading?: boolean;
}

// Default empty metrics
const DEFAULT_METRICS: RiskMetrics = {
  exposurePercent: 0,
  dailyLossPercent: 0,
  openPositions: 0,
  maxPositions: 10,
};

export function RiskMetricsWidget({ metrics = DEFAULT_METRICS, isLoading = false }: RiskMetricsWidgetProps) {
  const { mode } = useTradingModeContext();
  const accentColor = getModeColor(mode, 'accent');

  const getRiskLevel = (percent: number): 'low' | 'medium' | 'high' => {
    if (percent < 50) return 'low';
    if (percent < 80) return 'medium';
    return 'high';
  };

  const getRiskColor = (level: 'low' | 'medium' | 'high') => {
    switch (level) {
      case 'low':
        return colors.profit;
      case 'medium':
        return colors.status.warning;
      case 'high':
        return colors.loss;
    }
  };

  const exposureLevel = getRiskLevel(metrics.exposurePercent);
  const lossLevel = getRiskLevel(metrics.dailyLossPercent);

  if (isLoading) {
    return (
      <GlassCardWithHeader
        title="Risk Metrics"
        mode={mode}
        className="h-full"
      >
        <div className="space-y-4">
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-12 bg-slate-700/20 rounded animate-pulse"></div>
          ))}
        </div>
      </GlassCardWithHeader>
    );
  }

  return (
    <GlassCardWithHeader
      title="Risk Metrics"
      mode={mode}
      className="h-full"
      action={
        <Shield className="w-4 h-4 text-gray-400" />
      }
    >
      <div className="space-y-6">
        {/* Exposure Gauge */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-400">Exposure</span>
            <span
              className="text-sm font-bold"
              style={{ color: getRiskColor(exposureLevel) }}
            >
              {metrics.exposurePercent.toFixed(0)}%
            </span>
          </div>
          <div className="relative h-2 bg-slate-800 rounded-full overflow-hidden">
            <div
              className="absolute top-0 left-0 h-full rounded-full transition-all duration-500"
              style={{
                width: `${metrics.exposurePercent}%`,
                backgroundColor: getRiskColor(exposureLevel),
              }}
            />
          </div>
        </div>

        {/* Daily Loss Limit */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <AlertTriangle className="w-4 h-4 text-gray-400" />
              <span className="text-sm text-gray-400">Daily Loss</span>
            </div>
            <span
              className="text-sm font-bold"
              style={{ color: getRiskColor(lossLevel) }}
            >
              {metrics.dailyLossPercent.toFixed(0)}%
            </span>
          </div>
          <div className="relative h-2 bg-slate-800 rounded-full overflow-hidden">
            <div
              className="absolute top-0 left-0 h-full rounded-full transition-all duration-500"
              style={{
                width: `${metrics.dailyLossPercent}%`,
                backgroundColor: getRiskColor(lossLevel),
              }}
            />
          </div>
          <p className="text-xs text-gray-500 mt-1">
            of 5% daily limit used
          </p>
        </div>

        {/* Open Positions */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-4 h-4 text-gray-400" />
              <span className="text-sm text-gray-400">Open Positions</span>
            </div>
            <span className="text-sm font-bold text-gray-100">
              {metrics.openPositions}
              {metrics.maxPositions && (
                <span className="text-gray-500 font-normal">
                  {' '}/ {metrics.maxPositions}
                </span>
              )}
            </span>
          </div>
          {metrics.maxPositions && (
            <div className="relative h-2 bg-slate-800 rounded-full overflow-hidden">
              <div
                className="absolute top-0 left-0 h-full rounded-full transition-all duration-500"
                style={{
                  width: `${(metrics.openPositions / metrics.maxPositions) * 100}%`,
                  backgroundColor: accentColor,
                }}
              />
            </div>
          )}
        </div>

        {/* Risk Summary */}
        <div
          className={cn(
            'p-3 rounded-lg border',
            exposureLevel === 'high' || lossLevel === 'high'
              ? 'bg-red-500/5 border-red-500/20'
              : 'bg-slate-800/50 border-slate-700/50'
          )}
        >
          <div className="flex items-start gap-2">
            <Shield
              className="w-4 h-4 mt-0.5 flex-shrink-0"
              style={{
                color:
                  exposureLevel === 'high' || lossLevel === 'high'
                    ? colors.loss
                    : colors.profit,
              }}
            />
            <div>
              <p className="text-xs font-medium text-gray-300">
                {exposureLevel === 'high' || lossLevel === 'high'
                  ? 'High Risk Alert'
                  : 'Risk Under Control'}
              </p>
              <p className="text-xs text-gray-500 mt-1">
                {exposureLevel === 'high' || lossLevel === 'high'
                  ? 'Consider reducing exposure or waiting for better opportunities.'
                  : 'Your portfolio is within safe risk limits.'}
              </p>
            </div>
          </div>
        </div>
      </div>
    </GlassCardWithHeader>
  );
}
