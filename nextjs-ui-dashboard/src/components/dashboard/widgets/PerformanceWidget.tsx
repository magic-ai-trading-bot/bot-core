/**
 * PerformanceWidget Component
 *
 * Portfolio performance chart with time range selector.
 * Shows value over time with mode-aware styling.
 */

import { useState } from 'react';
import { GlassCardWithHeader } from '@/components/ui/GlassCard';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { LineChart, Line, XAxis, YAxis, Tooltip, ResponsiveContainer, CartesianGrid } from 'recharts';
import { colors, getModeColor } from '@/styles';
import { cn } from '@/lib/utils';

type TimeRange = '24h' | '7d' | '30d' | '90d';

interface DataPoint {
  timestamp: number;
  value: number;
}

interface PerformanceWidgetProps {
  data?: DataPoint[];
  isLoading?: boolean;
}

export function PerformanceWidget({ data = [], isLoading = false }: PerformanceWidgetProps) {
  const { mode } = useTradingModeContext();
  const [timeRange, setTimeRange] = useState<TimeRange>('7d');

  const chartData = data;
  const accentColor = getModeColor(mode, 'accent');

  const timeRangeButtons: TimeRange[] = ['24h', '7d', '30d', '90d'];

  if (isLoading) {
    return (
      <GlassCardWithHeader
        title="Performance"
        mode={mode}
        className="h-full"
      >
        <div className="h-64 bg-slate-700/20 rounded animate-pulse"></div>
      </GlassCardWithHeader>
    );
  }

  return (
    <GlassCardWithHeader
      title="Portfolio Performance"
      mode={mode}
      className="h-full"
      action={
        <div className="flex gap-1 bg-slate-800/50 rounded-lg p-1">
          {timeRangeButtons.map((range) => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              className={cn(
                'px-3 py-1 rounded text-xs font-medium transition-all',
                timeRange === range
                  ? 'text-white'
                  : 'text-gray-400 hover:text-gray-300'
              )}
              style={
                timeRange === range
                  ? {
                      backgroundColor: mode === 'paper' ? 'rgba(14, 165, 233, 0.2)' : 'rgba(239, 68, 68, 0.2)',
                      color: accentColor,
                    }
                  : undefined
              }
            >
              {range.toUpperCase()}
            </button>
          ))}
        </div>
      }
    >
      <div className="h-64 mt-4">
        <ResponsiveContainer width="100%" height="100%">
          <LineChart data={chartData}>
            <defs>
              <linearGradient id="lineGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor={accentColor} stopOpacity={0.8} />
                <stop offset="100%" stopColor={accentColor} stopOpacity={0.1} />
              </linearGradient>
            </defs>
            <CartesianGrid
              strokeDasharray="3 3"
              stroke={colors.grid}
              opacity={0.2}
            />
            <XAxis
              dataKey="timestamp"
              stroke={colors.text.muted}
              fontSize={12}
              tickFormatter={(timestamp) => {
                const date = new Date(timestamp);
                if (timeRange === '24h') {
                  return date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' });
                }
                return date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
              }}
            />
            <YAxis
              stroke={colors.text.muted}
              fontSize={12}
              tickFormatter={(value) => `$${(value / 1000).toFixed(1)}k`}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'rgba(15, 23, 42, 0.95)',
                border: `1px solid ${colors.border}`,
                borderRadius: '8px',
                padding: '8px 12px',
              }}
              labelStyle={{ color: colors.text.secondary }}
              itemStyle={{ color: accentColor }}
              formatter={(value: number) => [`$${value.toFixed(2)}`, 'Value']}
              labelFormatter={(timestamp) => {
                const date = new Date(timestamp);
                return date.toLocaleString('en-US', {
                  month: 'short',
                  day: 'numeric',
                  hour: '2-digit',
                  minute: '2-digit',
                });
              }}
            />
            <Line
              type="monotone"
              dataKey="value"
              stroke={accentColor}
              strokeWidth={2}
              dot={false}
              fill="url(#lineGradient)"
              animationDuration={500}
            />
          </LineChart>
        </ResponsiveContainer>
      </div>
    </GlassCardWithHeader>
  );
}
