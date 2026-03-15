/**
 * PerformanceWidget Component
 *
 * Portfolio performance chart with time range selector.
 * Shows value over time with mode-aware styling.
 */

import { useState } from 'react';
import { GlassCardWithHeader } from '@/components/ui/GlassCard';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import ReactECharts from 'echarts-for-react';
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
        <ReactECharts
          option={{
            backgroundColor: 'transparent',
            tooltip: {
              trigger: 'axis',
              backgroundColor: 'rgba(15,23,42,0.95)',
              borderColor: colors.border,
              borderWidth: 1,
              textStyle: { color: accentColor, fontSize: 12 },
              formatter: (params: Array<{ axisValue: number; value: number }>) => {
                const p = params[0];
                const date = new Date(p.axisValue);
                const dateStr = timeRange === '24h'
                  ? date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })
                  : date.toLocaleString('en-US', { month: 'short', day: 'numeric', hour: '2-digit', minute: '2-digit' });
                return `<span style="color:${colors.text.secondary}">${dateStr}</span><br/><span style="color:${accentColor}">$${p.value.toFixed(2)}</span>`;
              },
            },
            grid: { left: 55, right: 16, top: 8, bottom: 28, containLabel: false },
            xAxis: {
              type: 'value',
              min: chartData.length > 0 ? chartData[0].timestamp : undefined,
              max: chartData.length > 0 ? chartData[chartData.length - 1].timestamp : undefined,
              axisLine: { show: false },
              axisTick: { show: false },
              axisLabel: {
                color: colors.text.muted,
                fontSize: 12,
                formatter: (value: number) => {
                  const date = new Date(value);
                  return timeRange === '24h'
                    ? date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })
                    : date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
                },
              },
              splitLine: { lineStyle: { color: colors.grid, opacity: 0.2, type: 'dashed' } },
            },
            yAxis: {
              type: 'value',
              axisLine: { show: false },
              axisTick: { show: false },
              axisLabel: {
                color: colors.text.muted,
                fontSize: 12,
                formatter: (value: number) => `$${(value / 1000).toFixed(1)}k`,
              },
              splitLine: { lineStyle: { color: colors.grid, opacity: 0.2, type: 'dashed' } },
            },
            series: [
              {
                type: 'line',
                data: chartData.map((d) => [d.timestamp, d.value]),
                smooth: true,
                symbol: 'none',
                lineStyle: { width: 2, color: accentColor },
                areaStyle: {
                  color: {
                    type: 'linear', x: 0, y: 0, x2: 0, y2: 1,
                    colorStops: [
                      { offset: 0, color: accentColor + 'cc' },
                      { offset: 1, color: accentColor + '1a' },
                    ],
                  },
                },
                animationDuration: 500,
              },
            ],
          }}
          notMerge={true}
          style={{ height: '100%', width: '100%' }}
        />
      </div>
    </GlassCardWithHeader>
  );
}
