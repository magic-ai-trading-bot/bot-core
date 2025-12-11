/**
 * Dashboard Page - Premium Dark OLED Luxury Edition
 *
 * Award-winning design with glassmorphism, gradient accents, and premium animations.
 * Features: Hero stats, price ticker, performance chart, AI signals, and recent trades.
 *
 * Design Specifications:
 * - Pure black background (#000000) for OLED displays
 * - Glassmorphism cards with backdrop-blur-xl
 * - Emerald (#22c55e) for profit, Cyan (#00D9FF) for highlights
 * - Premium typography with gradient text
 * - Micro-interactions and pulse animations
 */

import { useEffect, useState, useMemo, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { motion } from 'framer-motion';
import ErrorBoundary from '@/components/ErrorBoundary';
import { useWebSocket } from '@/hooks/useWebSocket';
import { useTradingModeContext } from '@/contexts/TradingModeContext';
import { usePaperTrading } from '@/hooks/usePaperTrading';
import { useNavigate } from 'react-router-dom';
import { apiClient } from '@/services/api';
import { cn } from '@/lib/utils';
import logger from '@/utils/logger';
import { useThemeColors } from '@/hooks/useThemeColors';
import {
  TrendingUp,
  TrendingDown,
  Brain,
  Activity,
  ArrowUpRight,
  ArrowDownRight,
  Sparkles,
  Zap,
  Shield,
  Flame,
  Clock,
  ChevronRight,
  BarChart3,
} from 'lucide-react';
import {
  AreaChart,
  Area,
  XAxis,
  YAxis,
  Tooltip,
  ResponsiveContainer,
  CartesianGrid,
} from 'recharts';

// =============================================================================
// DESIGN TOKENS - Now using useThemeColors() hook for theme-aware colors
// =============================================================================

// =============================================================================
// ANIMATION VARIANTS
// =============================================================================

const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: {
      staggerChildren: 0.1,
      delayChildren: 0.05,
    },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 20 },
  visible: {
    opacity: 1,
    y: 0,
    transition: {
      type: 'spring',
      stiffness: 100,
      damping: 15,
    },
  },
};

const pulseVariants = {
  pulse: {
    scale: [1, 1.02, 1],
    transition: {
      duration: 2,
      repeat: Infinity,
      ease: 'easeInOut',
    },
  },
};

const glowVariants = {
  glow: {
    boxShadow: [
      '0 0 20px rgba(34, 197, 94, 0.2)',
      '0 0 40px rgba(34, 197, 94, 0.3)',
      '0 0 20px rgba(34, 197, 94, 0.2)',
    ],
    transition: {
      duration: 2,
      repeat: Infinity,
      ease: 'easeInOut',
    },
  },
};

// =============================================================================
// UTILITY COMPONENTS
// =============================================================================

interface GlassCardProps {
  children: React.ReactNode;
  className?: string;
  hoverable?: boolean;
  glowColor?: string;
  onClick?: () => void;
}

const GlassCard = ({
  children,
  className,
  hoverable = false,
  glowColor,
  onClick,
}: GlassCardProps) => (
  <motion.div
    variants={itemVariants}
    whileHover={
      hoverable
        ? {
            y: -2,
            boxShadow: glowColor
              ? `0 8px 32px ${glowColor}`
              : '0 8px 32px rgba(0, 217, 255, 0.15)',
          }
        : undefined
    }
    onClick={onClick}
    className={cn(
      'relative overflow-hidden rounded-2xl',
      'bg-white/[0.03] backdrop-blur-xl',
      'border border-white/[0.08]',
      'transition-all duration-300',
      hoverable && 'cursor-pointer',
      className
    )}
  >
    {children}
  </motion.div>
);

interface GradientTextProps {
  children: React.ReactNode;
  className?: string;
  gradient?: string;
}

const GradientText = ({
  children,
  className,
  gradient,
}: GradientTextProps) => {
  const colors = useThemeColors();
  return (
    <span
      className={cn('bg-clip-text text-transparent', className)}
      style={{ backgroundImage: gradient || colors.gradientPremium }}
    >
      {children}
    </span>
  );
};

interface AnimatedValueProps {
  value: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  className?: string;
  showSign?: boolean;
  colorize?: boolean;
}

const AnimatedValue = ({
  value,
  prefix = '',
  suffix = '',
  decimals = 2,
  className,
  showSign = false,
  colorize = false,
}: AnimatedValueProps) => {
  const colors = useThemeColors();
  const formattedValue = useMemo(() => {
    const sign = showSign && value > 0 ? '+' : '';
    return `${prefix}${sign}${value.toLocaleString('en-US', {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    })}${suffix}`;
  }, [value, prefix, suffix, decimals, showSign]);

  const color = colorize
    ? value >= 0
      ? colors.profit
      : colors.loss
    : undefined;

  return (
    <motion.span
      key={value} // Re-animate on value change
      className={className}
      style={{ color }}
      initial={{ scale: 1 }}
      animate={{ scale: [1, 1.02, 1] }}
      transition={{ duration: 0.3 }}
    >
      {formattedValue}
    </motion.span>
  );
};

// =============================================================================
// HERO STATS SECTION
// =============================================================================

interface HeroStatsProps {
  balance: number;
  pnl: number;
  pnlPercentage: number;
  isLoading: boolean;
  colors: ReturnType<typeof useThemeColors>;
}

const HeroStats = ({ balance, pnl, pnlPercentage, isLoading, colors }: HeroStatsProps) => {
  const { t } = useTranslation('dashboard');
  const { mode } = useTradingModeContext();
  const isProfitable = pnl >= 0;

  if (isLoading) {
    return (
      <GlassCard className="p-8">
        <div className="animate-pulse space-y-4">
          <div className="h-4 w-24 bg-white/10 rounded" />
          <div className="h-16 w-64 bg-white/10 rounded" />
          <div className="h-8 w-48 bg-white/10 rounded" />
        </div>
      </GlassCard>
    );
  }

  return (
    <GlassCard className="relative p-8">
      {/* Background glow effect */}
      <div
        className="absolute top-0 right-0 w-96 h-96 rounded-full blur-3xl opacity-20 pointer-events-none"
        style={{
          background: isProfitable
            ? 'radial-gradient(circle, rgba(34, 197, 94, 0.4), transparent 70%)'
            : 'radial-gradient(circle, rgba(239, 68, 68, 0.4), transparent 70%)',
        }}
      />

      {/* Animated particles effect */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        {[...Array(5)].map((_, i) => (
          <motion.div
            key={i}
            className="absolute w-1 h-1 rounded-full bg-cyan-400/30"
            initial={{ x: `${Math.random() * 100}%`, y: '100%', opacity: 0 }}
            animate={{
              y: '-20%',
              opacity: [0, 1, 0],
            }}
            transition={{
              duration: 3 + Math.random() * 2,
              repeat: Infinity,
              delay: i * 0.5,
              ease: 'linear',
            }}
          />
        ))}
      </div>

      <div className="relative z-10">
        {/* Header row */}
        <div className="flex items-center justify-between mb-6">
          <div className="flex items-center gap-3">
            <div
              className="p-2 rounded-xl"
              style={{
                background: 'rgba(0, 217, 255, 0.1)',
                border: '1px solid rgba(0, 217, 255, 0.2)',
              }}
            >
              <BarChart3 className="w-5 h-5 text-cyan-400" />
            </div>
            <div>
              <p className="text-sm uppercase tracking-wider font-medium" style={{ color: colors.textMuted }}>
                {t('hero.totalBalance')}
              </p>
              <p className="text-xs" style={{ color: colors.textMuted }}>
                {mode === 'paper' ? t('hero.paperTrading') : t('hero.liveTrading')}
              </p>
            </div>
          </div>

          {/* Mode badge */}
          <motion.div
            className="px-4 py-2 rounded-full text-xs font-bold uppercase tracking-wider"
            style={{
              background:
                mode === 'paper'
                  ? 'rgba(0, 217, 255, 0.15)'
                  : 'rgba(239, 68, 68, 0.15)',
              border:
                mode === 'paper'
                  ? '1px solid rgba(0, 217, 255, 0.3)'
                  : '1px solid rgba(239, 68, 68, 0.3)',
              color: mode === 'paper' ? '#00D9FF' : '#ef4444',
            }}
            animate={{ scale: [1, 1.02, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
          >
            {mode === 'paper' ? t('hero.paperMode') : t('hero.liveMode')}
          </motion.div>
        </div>

        {/* Main balance with gradient text */}
        <motion.div
          className="mb-6"
          variants={pulseVariants}
          animate={isProfitable ? 'pulse' : undefined}
        >
          <GradientText
            className="text-6xl md:text-7xl font-black tracking-tight"
            gradient={isProfitable ? colors.gradientProfit : colors.gradientLoss}
          >
            <AnimatedValue value={balance} prefix="$" decimals={2} />
          </GradientText>
        </motion.div>

        {/* PnL section with glow */}
        <motion.div
          className="inline-flex items-center gap-4 p-4 rounded-xl"
          style={{
            background: isProfitable
              ? 'rgba(34, 197, 94, 0.1)'
              : 'rgba(239, 68, 68, 0.1)',
            border: isProfitable
              ? '1px solid rgba(34, 197, 94, 0.2)'
              : '1px solid rgba(239, 68, 68, 0.2)',
            boxShadow: isProfitable
              ? '0 0 30px rgba(34, 197, 94, 0.2)'
              : '0 0 30px rgba(239, 68, 68, 0.2)',
          }}
          variants={glowVariants}
          animate="glow"
        >
          {/* Trend icon */}
          <div
            className="p-3 rounded-xl"
            style={{
              background: isProfitable
                ? 'rgba(34, 197, 94, 0.2)'
                : 'rgba(239, 68, 68, 0.2)',
            }}
          >
            {isProfitable ? (
              <TrendingUp className="w-6 h-6" style={{ color: colors.profit }} />
            ) : (
              <TrendingDown className="w-6 h-6" style={{ color: colors.loss }} />
            )}
          </div>

          {/* PnL values */}
          <div>
            <div className="flex items-center gap-2">
              <AnimatedValue
                value={pnl}
                prefix="$"
                showSign
                colorize
                className="text-2xl font-bold"
              />
              <span style={{ color: colors.textMuted }}>|</span>
              <AnimatedValue
                value={pnlPercentage}
                suffix="%"
                showSign
                colorize
                className="text-xl font-semibold"
              />
            </div>
            <p className="text-sm mt-1" style={{ color: colors.textMuted }}>{t('hero.change24h')}</p>
          </div>
        </motion.div>
      </div>
    </GlassCard>
  );
};

// =============================================================================
// PRICE TICKER ROW
// =============================================================================

interface CoinPrice {
  symbol: string;
  price: number;
  change24h: number;
  changePercent24h: number;
}

// Fallback data when API is unavailable
const FALLBACK_COINS: CoinPrice[] = [
  { symbol: 'BTCUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'ETHUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'BNBUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'SOLUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'ADAUSDT', price: 0, change24h: 0, changePercent24h: 0 },
  { symbol: 'XRPUSDT', price: 0, change24h: 0, changePercent24h: 0 },
];

// List of symbols to fetch
const TICKER_SYMBOLS = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT', 'ADAUSDT', 'XRPUSDT'];

interface PriceTickerProps {
  isLoading: boolean;
  colors: ReturnType<typeof useThemeColors>;
}

const PriceTicker = ({ isLoading: parentLoading, colors }: PriceTickerProps) => {
  const navigate = useNavigate();
  const [coins, setCoins] = useState<CoinPrice[]>(FALLBACK_COINS);
  const [isTickerLoading, setIsTickerLoading] = useState(true);

  // Fetch prices for all symbols
  const fetchPrices = useCallback(async () => {
    try {
      const pricePromises = TICKER_SYMBOLS.map(async (symbol) => {
        try {
          const chartData = await apiClient.rust.getChartData(symbol, '1h', 10);
          return {
            symbol,
            price: chartData.latest_price,
            change24h: chartData.price_change_24h,
            changePercent24h: chartData.price_change_percent_24h,
          };
        } catch {
          // Return current data for this symbol on error
          return coins.find((c) => c.symbol === symbol) || FALLBACK_COINS.find((c) => c.symbol === symbol)!;
        }
      });

      const results = await Promise.all(pricePromises);
      // Only update if we have valid price data
      const validResults = results.filter((r) => r.price > 0);
      if (validResults.length > 0) {
        setCoins(results);
      }
    } catch (error) {
      logger.debug('Failed to fetch ticker prices:', error);
      // Keep previous data on error
    } finally {
      setIsTickerLoading(false);
    }
  }, [coins]);

  // Initial fetch and auto-refresh every 10 seconds
  useEffect(() => {
    fetchPrices();
    const interval = setInterval(fetchPrices, 10000);
    return () => clearInterval(interval);
  }, []);  

  const isLoading = parentLoading || isTickerLoading;

  if (isLoading) {
    return (
      <div className="flex gap-4 overflow-x-auto scrollbar-hide py-2">
        {[...Array(6)].map((_, i) => (
          <div
            key={i}
            className="flex-shrink-0 w-40 h-24 bg-white/[0.03] rounded-xl animate-pulse"
          />
        ))}
      </div>
    );
  }

  return (
    <motion.div
      className="flex gap-4 overflow-x-auto scrollbar-hide py-2 -mx-2 px-2"
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      {coins.map((coin, index) => {
        const isPositive = coin.changePercent24h >= 0;
        const hasData = coin.price > 0;
        return (
          <motion.button
            key={coin.symbol}
            variants={itemVariants}
            whileHover={{ y: -4, scale: 1.02 }}
            whileTap={{ scale: 0.98 }}
            onClick={() => navigate(`/trading/${coin.symbol}`)}
            className={cn(
              'flex-shrink-0 w-40 p-4 rounded-xl',
              'bg-white/[0.03] backdrop-blur-xl',
              'border border-white/[0.08]',
              'hover:border-white/[0.15]',
              'transition-all duration-300 text-left group'
            )}
            style={{
              animationDelay: `${index * 100}ms`,
            }}
          >
            {/* Symbol */}
            <div className="flex items-center gap-2 mb-2">
              <span className="text-sm font-bold group-hover:opacity-100" style={{ color: colors.textPrimary }}>
                {coin.symbol.replace('USDT', '')}
              </span>
              <span className="text-xs" style={{ color: colors.textMuted }}>/USDT</span>
            </div>

            {/* Price */}
            <div className="text-lg font-bold mb-2 font-mono" style={{ color: colors.textPrimary }}>
              {hasData
                ? `$${coin.price.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: coin.price < 1 ? 4 : 2 })}`
                : '--'}
            </div>

            {/* Change indicator */}
            <div
              className="flex items-center gap-1 px-2 py-1 rounded-md w-fit"
              style={{
                background: hasData
                  ? isPositive
                    ? 'rgba(34, 197, 94, 0.15)'
                    : 'rgba(239, 68, 68, 0.15)'
                  : 'rgba(255, 255, 255, 0.05)',
              }}
            >
              {hasData ? (
                <>
                  {isPositive ? (
                    <TrendingUp className="w-3 h-3" style={{ color: colors.profit }} />
                  ) : (
                    <TrendingDown className="w-3 h-3" style={{ color: colors.loss }} />
                  )}
                  <span
                    className="text-xs font-bold"
                    style={{ color: isPositive ? colors.profit : colors.loss }}
                  >
                    {isPositive ? '+' : ''}
                    {coin.changePercent24h.toFixed(2)}%
                  </span>
                </>
              ) : (
                <span className="text-xs" style={{ color: colors.textMuted }}>--</span>
              )}
            </div>
          </motion.button>
        );
      })}
    </motion.div>
  );
};

// =============================================================================
// SHARED TRADE INTERFACE
// =============================================================================

interface Trade {
  id: string;
  symbol: string;
  side: 'BUY' | 'SELL';
  quantity: number;
  entry_price: number;
  exit_price?: number;
  pnl?: number;
  entry_time: string;
  exit_time?: string;
  status: 'open' | 'closed';
}

// =============================================================================
// PERFORMANCE CHART
// =============================================================================

type TimeRange = '24h' | '7d' | '30d' | '90d';

interface PerformanceChartProps {
  isLoading: boolean;
  closedTrades?: Trade[];
  currentBalance?: number;
  colors: ReturnType<typeof useThemeColors>;
}

// Generate chart data from real closed trades
const generateChartDataFromTrades = (
  trades: Trade[],
  range: TimeRange,
  currentBalance: number
) => {
  const now = Date.now();
  const rangeDays = { '24h': 1, '7d': 7, '30d': 30, '90d': 90 };
  const rangeMs = rangeDays[range] * 24 * 60 * 60 * 1000;
  const cutoffTime = now - rangeMs;
  const initialBalance = 10000;

  // Filter trades within the time range and sort by exit_time
  const filteredTrades = trades
    .filter((trade) => {
      const exitTime = trade.exit_time ? new Date(trade.exit_time).getTime() : 0;
      return exitTime >= cutoffTime && trade.status === 'closed';
    })
    .sort((a, b) => {
      const timeA = a.exit_time ? new Date(a.exit_time).getTime() : 0;
      const timeB = b.exit_time ? new Date(b.exit_time).getTime() : 0;
      return timeA - timeB;
    });

  // If no trades, show flat line from initial balance to current balance
  if (filteredTrades.length === 0) {
    const dataPoints = range === '24h' ? 24 : rangeDays[range];
    const step = rangeMs / dataPoints;
    return Array.from({ length: dataPoints }, (_, i) => ({
      timestamp: cutoffTime + i * step,
      value: i === dataPoints - 1 ? currentBalance : initialBalance,
    }));
  }

  // Build cumulative balance from trades
  const dataPoints: { timestamp: number; value: number }[] = [];

  // Start with initial balance at the beginning of the range
  let runningBalance = initialBalance;

  // Calculate balance before the first filtered trade
  // Sum all PnL from trades before the cutoff time
  const tradesBeforeCutoff = trades
    .filter((trade) => {
      const exitTime = trade.exit_time ? new Date(trade.exit_time).getTime() : 0;
      return exitTime < cutoffTime && trade.status === 'closed';
    });

  tradesBeforeCutoff.forEach((trade) => {
    runningBalance += trade.pnl || 0;
  });

  // Add starting point
  dataPoints.push({
    timestamp: cutoffTime,
    value: runningBalance,
  });

  // Add data points for each trade
  filteredTrades.forEach((trade) => {
    const exitTime = trade.exit_time ? new Date(trade.exit_time).getTime() : now;
    runningBalance += trade.pnl || 0;
    dataPoints.push({
      timestamp: exitTime,
      value: runningBalance,
    });
  });

  // Add current balance as final point if different from last trade time
  const lastPoint = dataPoints[dataPoints.length - 1];
  if (lastPoint.timestamp < now - 60000) {
    dataPoints.push({
      timestamp: now,
      value: currentBalance || runningBalance,
    });
  }

  return dataPoints;
};

const PerformanceChart = ({ isLoading, closedTrades = [], currentBalance = 10000, colors }: PerformanceChartProps) => {
  const { t } = useTranslation('dashboard');
  const [timeRange, setTimeRange] = useState<TimeRange>('7d');
  const chartData = useMemo(
    () => generateChartDataFromTrades(closedTrades, timeRange, currentBalance),
    [closedTrades, timeRange, currentBalance]
  );

  const timeRanges: TimeRange[] = ['24h', '7d', '30d', '90d'];

  if (isLoading) {
    return (
      <GlassCard className="p-6 h-full">
        <div className="animate-pulse space-y-4">
          <div className="h-6 w-48 bg-white/10 rounded" />
          <div className="h-64 bg-white/5 rounded-xl" />
        </div>
      </GlassCard>
    );
  }

  return (
    <GlassCard className="p-6 h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <div
            className="p-2 rounded-xl"
            style={{
              background: 'rgba(0, 217, 255, 0.1)',
              border: '1px solid rgba(0, 217, 255, 0.2)',
            }}
          >
            <Activity className="w-5 h-5 text-cyan-400" />
          </div>
          <div>
            <h3 className="text-lg font-bold" style={{ color: colors.textPrimary }}>{t('chart.portfolioPerformance')}</h3>
            <p className="text-sm" style={{ color: colors.textMuted }}>{t('chart.trackGains')}</p>
          </div>
        </div>

        {/* Time range selector */}
        <div className="flex gap-1 p-1 rounded-xl" style={{ backgroundColor: colors.bgSecondary, border: `1px solid ${colors.borderSubtle}` }}>
          {timeRanges.map((range) => (
            <button
              key={range}
              onClick={() => setTimeRange(range)}
              className={cn(
                'px-3 py-1.5 rounded-lg text-xs font-bold uppercase tracking-wider',
                'transition-all duration-300',
                timeRange === range
                  ? 'bg-gradient-to-r from-emerald-500/20 to-cyan-500/20 text-cyan-400 border border-cyan-500/20'
                  : ''
              )}
              style={timeRange !== range ? { color: colors.textMuted } : undefined}
            >
              {range}
            </button>
          ))}
        </div>
      </div>

      {/* Chart */}
      <div className="h-64">
        <ResponsiveContainer width="100%" height="100%">
          <AreaChart data={chartData}>
            <defs>
              <linearGradient id="performanceGradient" x1="0" y1="0" x2="0" y2="1">
                <stop offset="0%" stopColor="#22c55e" stopOpacity={0.4} />
                <stop offset="50%" stopColor="#00D9FF" stopOpacity={0.2} />
                <stop offset="100%" stopColor="#00D9FF" stopOpacity={0} />
              </linearGradient>
              <linearGradient id="lineGradient" x1="0" y1="0" x2="1" y2="0">
                <stop offset="0%" stopColor="#22c55e" />
                <stop offset="100%" stopColor="#00D9FF" />
              </linearGradient>
            </defs>
            <CartesianGrid
              strokeDasharray="3 3"
              stroke="rgba(255,255,255,0.05)"
              vertical={false}
            />
            <XAxis
              dataKey="timestamp"
              stroke="rgba(255,255,255,0.2)"
              fontSize={11}
              tickFormatter={(timestamp) => {
                const date = new Date(timestamp);
                return timeRange === '24h'
                  ? date.toLocaleTimeString('en-US', { hour: '2-digit', minute: '2-digit' })
                  : date.toLocaleDateString('en-US', { month: 'short', day: 'numeric' });
              }}
              axisLine={false}
              tickLine={false}
            />
            <YAxis
              stroke="rgba(255,255,255,0.2)"
              fontSize={11}
              tickFormatter={(value) => `$${(value / 1000).toFixed(1)}k`}
              axisLine={false}
              tickLine={false}
              width={60}
            />
            <Tooltip
              contentStyle={{
                backgroundColor: 'rgba(0, 0, 0, 0.9)',
                border: '1px solid rgba(255, 255, 255, 0.1)',
                borderRadius: '12px',
                padding: '12px 16px',
                boxShadow: '0 8px 32px rgba(0, 0, 0, 0.5)',
              }}
              labelStyle={{ color: 'rgba(255,255,255,0.5)', marginBottom: '4px' }}
              itemStyle={{ color: '#00D9FF', fontWeight: 'bold' }}
              formatter={(value: number) => [`$${value.toFixed(2)}`, 'Value']}
              labelFormatter={(timestamp) =>
                new Date(timestamp).toLocaleString('en-US', {
                  month: 'short',
                  day: 'numeric',
                  hour: '2-digit',
                  minute: '2-digit',
                })
              }
            />
            <Area
              type="monotone"
              dataKey="value"
              stroke="url(#lineGradient)"
              strokeWidth={3}
              fill="url(#performanceGradient)"
              animationDuration={1000}
            />
          </AreaChart>
        </ResponsiveContainer>
      </div>
    </GlassCard>
  );
};

// =============================================================================
// AI SIGNALS PANEL
// =============================================================================

interface AISignal {
  signal: 'long' | 'short' | 'neutral';
  confidence: number;
  symbol: string;
  timestamp: string;
  model_type: string;
  timeframe: string;
}

interface AISignalsPanelProps {
  signals?: AISignal[];
  isLoading: boolean;
  colors: ReturnType<typeof useThemeColors>;
}

const AISignalsPanel = ({ signals = [], isLoading, colors }: AISignalsPanelProps) => {
  const { t } = useTranslation('dashboard');
  const formatTime = (timestamp: string) => {
    const diff = Math.floor((Date.now() - new Date(timestamp).getTime()) / 60000);
    if (diff < 1) return t('aiSignals.justNow');
    if (diff < 60) return t('aiSignals.minutesAgo', { minutes: diff });
    return t('aiSignals.hoursAgo', { hours: Math.floor(diff / 60) });
  };

  if (isLoading) {
    return (
      <GlassCard className="p-6 h-full">
        <div className="animate-pulse space-y-4">
          <div className="h-6 w-32 bg-white/10 rounded" />
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-20 bg-white/5 rounded-xl" />
          ))}
        </div>
      </GlassCard>
    );
  }

  return (
    <GlassCard className="p-6 h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <motion.div
            className="p-2 rounded-xl"
            style={{
              background: 'linear-gradient(135deg, rgba(34, 197, 94, 0.2), rgba(0, 217, 255, 0.2))',
              border: '1px solid rgba(0, 217, 255, 0.3)',
            }}
            animate={{
              boxShadow: [
                '0 0 10px rgba(0, 217, 255, 0.3)',
                '0 0 20px rgba(0, 217, 255, 0.5)',
                '0 0 10px rgba(0, 217, 255, 0.3)',
              ],
            }}
            transition={{ duration: 2, repeat: Infinity }}
          >
            <Brain className="w-5 h-5 text-cyan-400" />
          </motion.div>
          <div>
            <h3 className="text-lg font-bold" style={{ color: colors.textPrimary }}>{t('aiSignals.title')}</h3>
            <p className="text-sm" style={{ color: colors.textMuted }}>{t('aiSignals.subtitle')}</p>
          </div>
        </div>

        {/* Live indicator */}
        <div className="flex items-center gap-2 px-3 py-1.5 rounded-full bg-emerald-500/10 border border-emerald-500/20">
          <motion.div
            className="w-2 h-2 rounded-full bg-emerald-500"
            animate={{ opacity: [1, 0.3, 1] }}
            transition={{ duration: 1.5, repeat: Infinity }}
          />
          <span className="text-xs font-bold text-emerald-400 uppercase tracking-wider">{t('aiSignals.live')}</span>
        </div>
      </div>

      {/* Signals list */}
      <div className="space-y-3">
        {signals.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-8 text-center">
            <Brain className="w-10 h-10 mb-3" style={{ color: colors.textMuted }} />
            <p className="font-medium" style={{ color: colors.textSecondary }}>{t('aiSignals.noSignals')}</p>
            <p className="text-sm mt-1" style={{ color: colors.textMuted }}>{t('aiSignals.noSignalsDesc')}</p>
          </div>
        ) : signals.map((signal, index) => {
          const isLong = signal.signal === 'long';
          const isShort = signal.signal === 'short';
          const signalColor = isLong ? colors.profit : isShort ? colors.loss : colors.warning;
          const confidencePercent = Math.round(signal.confidence * 100);

          return (
            <motion.div
              key={`${signal.symbol}-${index}`}
              initial={{ opacity: 0, x: -20 }}
              animate={{ opacity: 1, x: 0 }}
              transition={{ delay: index * 0.1 }}
              className="p-4 rounded-xl bg-white/[0.03] border border-white/[0.06] hover:border-white/[0.12] transition-all duration-300 group"
            >
              <div className="flex items-center justify-between">
                {/* Signal info */}
                <div className="flex items-center gap-3">
                  <div
                    className="p-2.5 rounded-xl"
                    style={{
                      background: `${signalColor}15`,
                      border: `1px solid ${signalColor}30`,
                    }}
                  >
                    {isLong ? (
                      <TrendingUp className="w-4 h-4" style={{ color: signalColor }} />
                    ) : isShort ? (
                      <TrendingDown className="w-4 h-4" style={{ color: signalColor }} />
                    ) : (
                      <Activity className="w-4 h-4" style={{ color: signalColor }} />
                    )}
                  </div>

                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-bold" style={{ color: colors.textPrimary }}>
                        {signal.symbol.replace('USDT', '')}
                      </span>
                      <span
                        className="text-xs font-bold uppercase px-2 py-0.5 rounded"
                        style={{
                          background: `${signalColor}20`,
                          color: signalColor,
                        }}
                      >
                        {signal.signal}
                      </span>
                    </div>
                    <div className="flex items-center gap-2 mt-1 text-xs" style={{ color: colors.textMuted }}>
                      <span>{signal.model_type}</span>
                      <span>.</span>
                      <span>{signal.timeframe}</span>
                      <span>.</span>
                      <span>{formatTime(signal.timestamp)}</span>
                    </div>
                  </div>
                </div>

                {/* Confidence meter */}
                <div className="text-right">
                  <div className="text-lg font-bold" style={{ color: colors.cyan }}>
                    {confidencePercent}%
                  </div>
                  <div className="w-20 h-1.5 mt-1 rounded-full bg-white/10 overflow-hidden">
                    <motion.div
                      className="h-full rounded-full"
                      style={{
                        background: colors.gradientPremium,
                      }}
                      initial={{ width: 0 }}
                      animate={{ width: `${confidencePercent}%` }}
                      transition={{ duration: 1, delay: index * 0.1 }}
                    />
                  </div>
                </div>
              </div>
            </motion.div>
          );
        })}
      </div>
    </GlassCard>
  );
};

// =============================================================================
// RECENT TRADES TABLE
// =============================================================================

interface RecentTradesProps {
  trades?: Trade[];
  isLoading: boolean;
  colors: ReturnType<typeof useThemeColors>;
}

const RecentTrades = ({ trades = [], isLoading, colors }: RecentTradesProps) => {
  const { t } = useTranslation('dashboard');
  const formatTime = (timestamp: string) => {
    const diff = Math.floor((Date.now() - new Date(timestamp).getTime()) / 60000);
    if (diff < 60) return t('aiSignals.minutesAgo', { minutes: diff });
    if (diff < 1440) return t('aiSignals.hoursAgo', { hours: Math.floor(diff / 60) });
    return t('trades.daysAgo', { days: Math.floor(diff / 1440) });
  };

  if (isLoading) {
    return (
      <GlassCard className="p-6 h-full">
        <div className="animate-pulse space-y-4">
          <div className="h-6 w-36 bg-white/10 rounded" />
          {[...Array(3)].map((_, i) => (
            <div key={i} className="h-16 bg-white/5 rounded-xl" />
          ))}
        </div>
      </GlassCard>
    );
  }

  return (
    <GlassCard className="p-6 h-full">
      {/* Header */}
      <div className="flex items-center justify-between mb-6">
        <div className="flex items-center gap-3">
          <div
            className="p-2 rounded-xl"
            style={{
              background: 'rgba(245, 158, 11, 0.1)',
              border: '1px solid rgba(245, 158, 11, 0.2)',
            }}
          >
            <Clock className="w-5 h-5 text-amber-400" />
          </div>
          <div>
            <h3 className="text-lg font-bold" style={{ color: colors.textPrimary }}>{t('trades.title')}</h3>
            <p className="text-sm" style={{ color: colors.textMuted }}>{t('trades.subtitle')}</p>
          </div>
        </div>

        <button className="flex items-center gap-1 px-3 py-1.5 rounded-lg text-xs font-bold hover:bg-white/5 transition-all" style={{ color: colors.textMuted }}>
          {t('trades.viewAll')}
          <ChevronRight className="w-4 h-4" />
        </button>
      </div>

      {/* Trades list */}
      <div className="space-y-3">
        {trades.length === 0 ? (
          <div className="flex flex-col items-center justify-center py-12 text-center">
            <Clock className="w-12 h-12 mb-4" style={{ color: colors.textMuted }} />
            <p className="font-medium" style={{ color: colors.textSecondary }}>{t('trades.noTrades')}</p>
            <p className="text-sm mt-1" style={{ color: colors.textMuted }}>{t('trades.noTradesDesc')}</p>
          </div>
        ) : trades.map((trade, index) => {
          const isBuy = trade.side === 'BUY';
          const isProfitable = (trade.pnl || 0) >= 0;
          const isOpen = trade.status === 'open';

          return (
            <motion.div
              key={trade.id}
              initial={{ opacity: 0, y: 10 }}
              animate={{ opacity: 1, y: 0 }}
              transition={{ delay: index * 0.1 }}
              whileHover={{ scale: 1.01, backgroundColor: 'rgba(255,255,255,0.05)' }}
              className="p-4 rounded-xl bg-white/[0.02] border border-white/[0.06] transition-all duration-300 cursor-pointer"
            >
              <div className="flex items-center justify-between">
                {/* Trade info */}
                <div className="flex items-center gap-3">
                  <div
                    className="p-2.5 rounded-xl"
                    style={{
                      background: isBuy ? 'rgba(34, 197, 94, 0.15)' : 'rgba(239, 68, 68, 0.15)',
                    }}
                  >
                    {isBuy ? (
                      <ArrowUpRight className="w-4 h-4" style={{ color: colors.profit }} />
                    ) : (
                      <ArrowDownRight className="w-4 h-4" style={{ color: colors.loss }} />
                    )}
                  </div>

                  <div>
                    <div className="flex items-center gap-2">
                      <span className="font-bold" style={{ color: colors.textPrimary }}>
                        {trade.symbol.replace('USDT', '')}
                      </span>
                      {isOpen && (
                        <motion.span
                          className="text-xs font-bold uppercase px-2 py-0.5 rounded-full"
                          style={{
                            background: 'rgba(0, 217, 255, 0.15)',
                            color: colors.cyan,
                            border: '1px solid rgba(0, 217, 255, 0.3)',
                          }}
                          animate={{ opacity: [1, 0.6, 1] }}
                          transition={{ duration: 2, repeat: Infinity }}
                        >
                          {t('trades.open')}
                        </motion.span>
                      )}
                    </div>
                    <div className="text-xs mt-1" style={{ color: colors.textMuted }}>
                      {trade.quantity} @ ${trade.entry_price.toFixed(2)}
                      {!isOpen && ` . ${formatTime(trade.exit_time || trade.entry_time)}`}
                    </div>
                  </div>
                </div>

                {/* PnL */}
                {trade.pnl !== undefined && (
                  <div className="text-right">
                    <div
                      className="text-lg font-bold"
                      style={{ color: isProfitable ? colors.profit : colors.loss }}
                    >
                      {isProfitable ? '+' : ''}${trade.pnl.toFixed(2)}
                    </div>
                    <div className="text-xs" style={{ color: colors.textMuted }}>{trade.side}</div>
                  </div>
                )}
              </div>
            </motion.div>
          );
        })}
      </div>
    </GlassCard>
  );
};

// =============================================================================
// QUICK STATS ROW
// =============================================================================

interface QuickStatsProps {
  winRate: number;
  totalTrades: number;
  avgProfit: number;
  isLoading: boolean;
  colors: ReturnType<typeof useThemeColors>;
}

const QuickStats = ({ winRate = 0, totalTrades = 0, avgProfit = 0, isLoading, colors }: QuickStatsProps) => {
  const { t } = useTranslation('dashboard');
  const stats = [
    {
      label: t('stats.winRate'),
      value: `${winRate}%`,
      icon: Sparkles,
      color: colors.profit,
      bgColor: 'rgba(34, 197, 94, 0.1)',
      borderColor: 'rgba(34, 197, 94, 0.2)',
    },
    {
      label: t('stats.totalTrades'),
      value: totalTrades.toString(),
      icon: Zap,
      color: colors.cyan,
      bgColor: 'rgba(0, 217, 255, 0.1)',
      borderColor: 'rgba(0, 217, 255, 0.2)',
    },
    {
      label: t('stats.avgProfit'),
      value: `+${avgProfit}%`,
      icon: TrendingUp,
      color: colors.profit,
      bgColor: 'rgba(34, 197, 94, 0.1)',
      borderColor: 'rgba(34, 197, 94, 0.2)',
    },
    {
      label: t('stats.riskScore'),
      value: t('stats.riskLow'),
      icon: Shield,
      color: colors.profit,
      bgColor: 'rgba(34, 197, 94, 0.1)',
      borderColor: 'rgba(34, 197, 94, 0.2)',
    },
  ];

  if (isLoading) {
    return (
      <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
        {[...Array(4)].map((_, i) => (
          <div
            key={i}
            className="h-24 bg-white/[0.03] rounded-xl animate-pulse"
          />
        ))}
      </div>
    );
  }

  return (
    <motion.div
      className="grid grid-cols-2 md:grid-cols-4 gap-4"
      variants={containerVariants}
      initial="hidden"
      animate="visible"
    >
      {stats.map((stat) => (
        <motion.div
          key={stat.label}
          variants={itemVariants}
          whileHover={{ y: -2, scale: 1.02 }}
          className="p-4 rounded-xl bg-white/[0.03] backdrop-blur-xl border border-white/[0.08] hover:border-white/[0.15] transition-all duration-300"
        >
          <div className="flex items-center gap-3 mb-3">
            <div
              className="p-2 rounded-lg"
              style={{
                background: stat.bgColor,
                border: `1px solid ${stat.borderColor}`,
              }}
            >
              <stat.icon className="w-4 h-4" style={{ color: stat.color }} />
            </div>
            <span className="text-sm font-medium" style={{ color: colors.textMuted }}>{stat.label}</span>
          </div>
          <div className="text-2xl font-bold" style={{ color: stat.color }}>
            {stat.value}
          </div>
        </motion.div>
      ))}
    </motion.div>
  );
};

// =============================================================================
// MAIN DASHBOARD COMPONENT
// =============================================================================

const Dashboard = () => {
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  const { state: wsState } = useWebSocket();
  const { portfolio, closedTrades, recentSignals, isLoading: isPaperTradingLoading, error: paperTradingError, refreshData } = usePaperTrading();
  const [initialLoadComplete, setInitialLoadComplete] = useState(false);

  // Fetch data on mount
  useEffect(() => {
    refreshData();
  }, [refreshData]);

  // Use real portfolio data from usePaperTrading hook
  const portfolioData = useMemo(() => {
    // Prefer usePaperTrading data, fallback to WebSocket for real-time updates
    const balance = portfolio.current_balance || portfolio.equity || 10000;
    const totalPnl = portfolio.total_pnl || wsState.botStatus?.total_pnl || 0;
    const pnlPercentage = portfolio.total_pnl_percentage || (totalPnl / 10000) * 100;

    logger.debug('Portfolio data from API:', {
      balance,
      pnl: totalPnl,
      pnlPercentage,
      source: portfolio.current_balance ? 'usePaperTrading' : 'WebSocket',
    });

    return {
      balance,
      pnl: totalPnl,
      pnlPercentage,
    };
  }, [portfolio, wsState.botStatus?.total_pnl]);

  // Calculate stats from real data
  const statsData = useMemo(() => {
    return {
      winRate: portfolio.win_rate || 0,
      totalTrades: portfolio.total_trades || 0,
      avgProfit: portfolio.profit_factor ? (portfolio.profit_factor - 1) * 100 : 0,
      maxDrawdown: portfolio.max_drawdown_percentage || 0,
    };
  }, [portfolio]);

  // Simulate initial load complete after 500ms
  useEffect(() => {
    const timer = setTimeout(() => {
      setInitialLoadComplete(true);
    }, 500);

    return () => clearTimeout(timer);
  }, []);

  // Loading is false when either API data arrives or initial timeout completes
  const isLoading = isPaperTradingLoading || (!initialLoadComplete && !wsState.botStatus && !portfolio.current_balance);

  const { balance, pnl, pnlPercentage } = portfolioData;

  return (
    <ErrorBoundary>
      <motion.div
        className="min-h-screen p-6 space-y-8"
        style={{ backgroundColor: colors.bgPrimary }}
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* Error Banner */}
        {paperTradingError && (
          <motion.div
            initial={{ opacity: 0, y: -20 }}
            animate={{ opacity: 1, y: 0 }}
            className="p-4 rounded-xl bg-red-500/10 border border-red-500/20"
          >
            <div className="flex items-center gap-3">
              <div className="p-2 rounded-lg bg-red-500/20">
                <Shield className="w-5 h-5 text-red-400" />
              </div>
              <div>
                <p className="text-sm font-medium text-red-400">{t('error.apiConnection')}</p>
                <p className="text-xs text-red-400/60">{paperTradingError}</p>
              </div>
              <button
                onClick={() => refreshData()}
                className="ml-auto px-3 py-1.5 text-xs font-medium text-red-400 bg-red-500/20 rounded-lg hover:bg-red-500/30 transition-colors"
              >
                {t('error.retry')}
              </button>
            </div>
          </motion.div>
        )}

        {/* Header */}
        <motion.div variants={itemVariants} className="flex items-center justify-between">
          <div>
            <h1 className="text-3xl font-black tracking-tight" style={{ color: colors.textPrimary }}>
              {t('welcomeBack')}
            </h1>
            <p className="mt-1" style={{ color: colors.textMuted }}>
              {t('subtitle')}
            </p>
          </div>

          {/* Live status indicator */}
          <div
            className="flex items-center gap-3 px-4 py-2 rounded-full"
            style={{ backgroundColor: colors.bgSecondary, border: `1px solid ${colors.borderSubtle}` }}
          >
            <motion.div
              className="w-2 h-2 rounded-full bg-emerald-500"
              animate={{ opacity: [1, 0.3, 1] }}
              transition={{ duration: 1.5, repeat: Infinity }}
            />
            <span className="text-sm font-medium" style={{ color: colors.textSecondary }}>{t('marketsOpen')}</span>
          </div>
        </motion.div>

        {/* Hero Stats */}
        <motion.div variants={itemVariants}>
          <HeroStats
            balance={balance}
            pnl={pnl}
            pnlPercentage={pnlPercentage}
            isLoading={isLoading}
            colors={colors}
          />
        </motion.div>

        {/* Quick Stats Row */}
        <motion.div variants={itemVariants}>
          <QuickStats
            winRate={statsData.winRate}
            totalTrades={statsData.totalTrades}
            avgProfit={statsData.avgProfit}
            isLoading={isLoading}
            colors={colors}
          />
        </motion.div>

        {/* Price Ticker */}
        <motion.div variants={itemVariants}>
          <div className="flex items-center gap-3 mb-4">
            <Flame className="w-5 h-5" style={{ color: colors.amber }} />
            <h2 className="text-lg font-bold" style={{ color: colors.textPrimary }}>{t('ticker.trendingMarkets')}</h2>
          </div>
          <PriceTicker isLoading={isLoading} colors={colors} />
        </motion.div>

        {/* Main Grid: Performance Chart + AI Signals + Recent Trades */}
        <div className="grid grid-cols-1 lg:grid-cols-2 xl:grid-cols-3 gap-6">
          {/* Performance Chart - Takes 2 columns on XL */}
          <motion.div variants={itemVariants} className="lg:col-span-2 xl:col-span-2">
            <PerformanceChart
              isLoading={isLoading}
              closedTrades={closedTrades}
              currentBalance={portfolio.current_balance || portfolio.equity || 10000}
              colors={colors}
            />
          </motion.div>

          {/* AI Signals - prefer usePaperTrading signals, fallback to WebSocket */}
          <motion.div variants={itemVariants} className="lg:col-span-1">
            <AISignalsPanel
              signals={recentSignals?.length ? recentSignals : (wsState.aiSignals?.length ? wsState.aiSignals : undefined)}
              isLoading={isLoading}
              colors={colors}
            />
          </motion.div>

          {/* Recent Trades - use real closed trades from API */}
          <motion.div variants={itemVariants} className="lg:col-span-2 xl:col-span-3">
            <RecentTrades
              trades={closedTrades?.length ? closedTrades.slice(0, 10) : (wsState.recentTrades?.length ? wsState.recentTrades : undefined)}
              isLoading={isLoading}
              colors={colors}
            />
          </motion.div>
        </div>

        {/* Footer spacing */}
        <div className="h-8" />
      </motion.div>
    </ErrorBoundary>
  );
};

export default Dashboard;
