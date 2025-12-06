/**
 * MarketOverviewWidget Component
 *
 * Market sentiment indicators and trending coins.
 * Shows Fear & Greed index and market overview.
 */

import { GlassCardWithHeader } from '@/components/ui/GlassCard';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { colors, getModeColor } from '@/styles';
import { Activity, TrendingUp, Flame } from 'lucide-react';
import { cn } from '@/lib/utils';

type SentimentLevel = 'fear' | 'neutral' | 'greed';

interface MarketData {
  sentiment: SentimentLevel;
  sentimentScore: number; // 0-100
  trendingCoins: Array<{
    symbol: string;
    change24h: number;
  }>;
  volumeChange: number;
}

interface MarketOverviewWidgetProps {
  data?: MarketData;
  isLoading?: boolean;
}

// Default empty data
const DEFAULT_DATA: MarketData = {
  sentiment: 'neutral',
  sentimentScore: 50,
  trendingCoins: [],
  volumeChange: 0,
};

export function MarketOverviewWidget({ data = DEFAULT_DATA, isLoading = false }: MarketOverviewWidgetProps) {
  const { mode } = useTradingModeContext();
  const accentColor = getModeColor(mode, 'accent');

  const getSentimentColor = (sentiment: SentimentLevel) => {
    switch (sentiment) {
      case 'fear':
        return colors.loss;
      case 'neutral':
        return colors.neutral;
      case 'greed':
        return colors.profit;
    }
  };

  const getSentimentLabel = (sentiment: SentimentLevel) => {
    switch (sentiment) {
      case 'fear':
        return 'Extreme Fear';
      case 'neutral':
        return 'Neutral';
      case 'greed':
        return 'Extreme Greed';
    }
  };

  if (isLoading) {
    return (
      <GlassCardWithHeader
        title="Market Overview"
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
      title="Market Overview"
      mode={mode}
      className="h-full"
      action={
        <Activity className="w-4 h-4 text-gray-400" />
      }
    >
      <div className="space-y-6">
        {/* Fear & Greed Index */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <span className="text-sm text-gray-400">Market Sentiment</span>
            <span
              className="text-sm font-bold"
              style={{ color: getSentimentColor(data.sentiment) }}
            >
              {getSentimentLabel(data.sentiment)}
            </span>
          </div>

          {/* Sentiment gauge */}
          <div className="relative h-2 bg-slate-800 rounded-full overflow-hidden">
            {/* Gradient background */}
            <div
              className="absolute top-0 left-0 h-full w-full"
              style={{
                background: `linear-gradient(to right, ${colors.loss}, ${colors.neutral}, ${colors.profit})`,
                opacity: 0.3,
              }}
            />
            {/* Indicator */}
            <div
              className="absolute top-0 h-full w-1 bg-white shadow-lg transition-all duration-500"
              style={{
                left: `${data.sentimentScore}%`,
                transform: 'translateX(-50%)',
              }}
            />
          </div>

          <div className="flex justify-between mt-1">
            <span className="text-xs text-gray-500">Fear</span>
            <span className="text-xs font-semibold" style={{ color: getSentimentColor(data.sentiment) }}>
              {data.sentimentScore}
            </span>
            <span className="text-xs text-gray-500">Greed</span>
          </div>
        </div>

        {/* Trending Coins */}
        <div>
          <div className="flex items-center gap-2 mb-3">
            <Flame className="w-4 h-4 text-orange-500" />
            <span className="text-sm text-gray-400">Trending</span>
          </div>

          <div className="space-y-2">
            {data.trendingCoins.map((coin, index) => (
              <div
                key={coin.symbol}
                className="flex items-center justify-between p-2 rounded-lg bg-slate-800/50 hover:bg-slate-800/70 transition-colors"
              >
                <div className="flex items-center gap-2">
                  <span className="text-xs text-gray-500 font-mono w-4">
                    #{index + 1}
                  </span>
                  <span className="text-sm font-semibold text-gray-100">
                    {coin.symbol}
                  </span>
                </div>

                <span
                  className="text-sm font-bold"
                  style={{
                    color: coin.change24h >= 0 ? colors.profit : colors.loss,
                  }}
                >
                  {coin.change24h >= 0 ? '+' : ''}
                  {coin.change24h.toFixed(1)}%
                </span>
              </div>
            ))}
          </div>
        </div>

        {/* Volume Change */}
        <div>
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <TrendingUp className="w-4 h-4 text-gray-400" />
              <span className="text-sm text-gray-400">24h Volume</span>
            </div>
            <span
              className="text-sm font-bold"
              style={{
                color: data.volumeChange >= 0 ? colors.profit : colors.loss,
              }}
            >
              {data.volumeChange >= 0 ? '+' : ''}
              {data.volumeChange.toFixed(1)}%
            </span>
          </div>

          <p className="text-xs text-gray-500">
            {data.volumeChange >= 0 ? 'Higher' : 'Lower'} than yesterday
          </p>
        </div>
      </div>
    </GlassCardWithHeader>
  );
}
