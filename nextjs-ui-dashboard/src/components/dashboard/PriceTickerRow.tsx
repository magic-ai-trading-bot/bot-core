/**
 * PriceTickerRow Component
 *
 * Horizontal scrolling price ticker with flash animations.
 * Real-time price updates via WebSocket.
 */

import { useState, useEffect } from 'react';
import { PriceFlash } from '@/components/ui/PriceFlash';
import { AnimatedPercentage } from '@/components/ui/AnimatedNumber';
import { cn } from '@/lib/utils';
import { colors } from '@/styles';
import { useNavigate } from 'react-router-dom';

interface CoinPrice {
  symbol: string;
  price: number;
  change24h: number;
  changePercent24h: number;
}

interface PriceTickerRowProps {
  coins?: CoinPrice[];
  isLoading?: boolean;
  className?: string;
}

// Default coins with zero values - real prices fetched from API
const DEFAULT_COINS: CoinPrice[] = [
  { symbol: 'BTCUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'ETHUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'BNBUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'SOLUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'ADAUSDT', price: 0, change24h: 0, changePercent24h: 0 },
];

export function PriceTickerRow({
  coins = DEFAULT_COINS,
  isLoading = false,
  className,
}: PriceTickerRowProps) {
  const navigate = useNavigate();

  const handleCoinClick = (symbol: string) => {
    navigate(`/trading/${symbol}`);
  };

  if (isLoading) {
    return (
      <div className={cn('py-4 bg-slate-900/50 border-b border-slate-700', className)}>
        <div className="flex gap-6 overflow-x-auto scrollbar-hide px-6 animate-pulse">
          {[...Array(5)].map((_, i) => (
            <div key={i} className="flex-shrink-0 w-32 h-12 bg-slate-700/20 rounded"></div>
          ))}
        </div>
      </div>
    );
  }

  return (
    <div className={cn('py-4 bg-slate-900/50 border-b border-slate-700', className)}>
      <div className="flex gap-6 overflow-x-auto scrollbar-hide px-6">
        {coins.map((coin) => (
          <button
            key={coin.symbol}
            onClick={() => handleCoinClick(coin.symbol)}
            className="flex-shrink-0 flex flex-col items-start gap-1 py-2 px-4 rounded-lg transition-colors hover:bg-slate-800/50 cursor-pointer group"
          >
            {/* Symbol */}
            <div className="text-xs font-semibold text-gray-400 uppercase group-hover:text-gray-300 transition-colors">
              {coin.symbol.replace('USDT', '')}
            </div>

            {/* Price with flash animation */}
            <PriceFlash value={coin.price} colorMode="up-down" flashDuration={500}>
              <div className="text-lg font-bold font-mono text-gray-100">
                ${coin.price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
              </div>
            </PriceFlash>

            {/* 24h change */}
            <div className="flex items-center gap-1">
              <span
                className="text-xs font-semibold"
                style={{
                  color: coin.changePercent24h >= 0 ? colors.profit : colors.loss,
                }}
              >
                {coin.changePercent24h >= 0 ? '▲' : '▼'}
              </span>
              <AnimatedPercentage
                value={coin.changePercent24h}
                decimals={2}
                showSign={false}
                className="text-xs"
              />
            </div>
          </button>
        ))}
      </div>
    </div>
  );
}
