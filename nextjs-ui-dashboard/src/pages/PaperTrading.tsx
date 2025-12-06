/**
 * Paper Trading Page - Professional Trading Terminal
 *
 * Designed following Binance/Bybit/OKX design patterns:
 * - Dark theme optimized for OLED (#0D1117 background)
 * - 60-40 layout split (chart vs order panels)
 * - Monospace fonts for numeric data
 * - Subtle micro-interactions (150ms transitions)
 * - Green/Red for profit/loss (#3FB950/#F85149)
 *
 * @spec:FR-TRADING-015 - Paper Trading Interface
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
 */

import { useState, useMemo, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { usePaperTrading } from '@/hooks/usePaperTrading';
import { TradingViewChart } from '@/components/trading/TradingViewChart';
import { type OrderFormData } from '@/components/trading/OrderForm';
import { useToast } from '@/hooks/use-toast';
import { formatDistanceToNow } from 'date-fns';
import { fetchBinancePrice } from '@/utils/binancePrice';
import logger from '@/utils/logger';
import {
  TrendingUp,
  TrendingDown,
  Activity,
  Wallet,
  BarChart3,
  Clock,
  Target,
  Shield,
  Zap,
  X,
  RefreshCw,
  ChevronDown,
  AlertTriangle,
  Percent,
  DollarSign,
  LineChart,
} from 'lucide-react';
import type { PaperTrade } from '@/hooks/usePaperTrading';

// ============================================================================
// DESIGN TOKENS - Premium Dark OLED Luxury (Matching Dashboard)
// ============================================================================

const luxuryColors = {
  // Backgrounds - Pure black for OLED
  bgPrimary: '#000000',
  bgSecondary: 'rgba(255, 255, 255, 0.03)',
  bgTertiary: 'rgba(255, 255, 255, 0.05)',
  bgHover: 'rgba(255, 255, 255, 0.08)',

  // Accents
  emerald: '#22c55e',
  cyan: '#00D9FF',
  profit: '#22c55e',
  loss: '#ef4444',
  warning: '#f59e0b',

  // Text
  textPrimary: '#ffffff',
  textSecondary: 'rgba(255, 255, 255, 0.7)',
  textMuted: 'rgba(255, 255, 255, 0.4)',

  // Borders
  borderSubtle: 'rgba(255, 255, 255, 0.08)',
  borderLight: 'rgba(255, 255, 255, 0.12)',
  borderActive: '#00D9FF',

  // Gradients
  gradientProfit: 'linear-gradient(135deg, #22c55e, #00D9FF)',
  gradientLoss: 'linear-gradient(135deg, #ef4444, #f97316)',
  gradientPremium: 'linear-gradient(135deg, #00D9FF, #22c55e)',
};

// Animation variants for consistency
const containerVariants = {
  hidden: { opacity: 0 },
  visible: {
    opacity: 1,
    transition: { staggerChildren: 0.08, delayChildren: 0.02 },
  },
};

const itemVariants = {
  hidden: { opacity: 0, y: 15 },
  visible: {
    opacity: 1,
    y: 0,
    transition: { type: 'spring', stiffness: 100, damping: 15 },
  },
};

// ============================================================================
// UTILITY COMPONENTS - Premium Glassmorphism
// ============================================================================

/**
 * GlassCard - Premium glassmorphism card component (matching Dashboard)
 */
function GlassCard({
  children,
  className = '',
  noPadding = false,
  hoverable = false,
  glowColor,
}: {
  children: React.ReactNode;
  className?: string;
  noPadding?: boolean;
  hoverable?: boolean;
  glowColor?: string;
}) {
  return (
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
      className={`relative overflow-hidden rounded-2xl bg-white/[0.03] backdrop-blur-xl border border-white/[0.08] transition-all duration-300 ${hoverable ? 'cursor-pointer' : ''} ${className}`}
    >
      <div className={noPadding ? '' : 'p-4'}>{children}</div>
    </motion.div>
  );
}

/**
 * PanelHeader - Premium header with icon glow
 */
function PanelHeader({
  title,
  icon: Icon,
  action,
}: {
  title: string;
  icon?: React.ElementType;
  action?: React.ReactNode;
}) {
  return (
    <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.08]">
      <div className="flex items-center gap-3">
        {Icon && (
          <div
            className="p-2 rounded-xl"
            style={{
              background: 'rgba(0, 217, 255, 0.1)',
              border: '1px solid rgba(0, 217, 255, 0.2)',
            }}
          >
            <Icon className="w-4 h-4" style={{ color: luxuryColors.cyan }} />
          </div>
        )}
        <h3 className="text-sm font-bold text-white">{title}</h3>
      </div>
      {action}
    </div>
  );
}

/**
 * Badge - Premium status badge with glow effect
 */
function Badge({
  children,
  variant = 'default',
}: {
  children: React.ReactNode;
  variant?: 'default' | 'buy' | 'sell' | 'info' | 'warning';
}) {
  const variants = {
    default: { bg: 'rgba(255, 255, 255, 0.1)', color: luxuryColors.textSecondary, border: 'rgba(255, 255, 255, 0.15)' },
    buy: { bg: 'rgba(34, 197, 94, 0.15)', color: luxuryColors.profit, border: 'rgba(34, 197, 94, 0.3)' },
    sell: { bg: 'rgba(239, 68, 68, 0.15)', color: luxuryColors.loss, border: 'rgba(239, 68, 68, 0.3)' },
    info: { bg: 'rgba(0, 217, 255, 0.15)', color: luxuryColors.cyan, border: 'rgba(0, 217, 255, 0.3)' },
    warning: { bg: 'rgba(245, 158, 11, 0.15)', color: luxuryColors.warning, border: 'rgba(245, 158, 11, 0.3)' },
  };

  const style = variants[variant];

  return (
    <span
      className="inline-flex items-center px-2.5 py-1 rounded-lg text-[10px] font-bold uppercase tracking-wider"
      style={{ backgroundColor: style.bg, color: style.color, border: `1px solid ${style.border}` }}
    >
      {children}
    </span>
  );
}

/**
 * GradientText - Text with gradient color
 */
function GradientText({
  children,
  className = '',
  gradient = luxuryColors.gradientPremium,
}: {
  children: React.ReactNode;
  className?: string;
  gradient?: string;
}) {
  return (
    <span
      className={`bg-clip-text text-transparent ${className}`}
      style={{ backgroundImage: gradient }}
    >
      {children}
    </span>
  );
}

/**
 * MonoText - Monospace text for numbers with premium styling
 */
function MonoText({
  children,
  className = '',
  positive,
  negative,
}: {
  children: React.ReactNode;
  className?: string;
  positive?: boolean;
  negative?: boolean;
}) {
  let color = luxuryColors.textPrimary;
  if (positive) color = luxuryColors.profit;
  if (negative) color = luxuryColors.loss;

  return (
    <span className={`font-mono ${className}`} style={{ color }}>
      {children}
    </span>
  );
}

/**
 * InputField - Premium form input with glass styling
 */
function InputField({
  label,
  value,
  onChange,
  placeholder,
  suffix,
}: {
  label: string;
  value: string;
  onChange: (v: string) => void;
  placeholder: string;
  suffix?: string;
}) {
  return (
    <div>
      <label className="block text-[10px] uppercase tracking-wider mb-1.5" style={{ color: luxuryColors.textMuted }}>
        {label}
      </label>
      <div
        className="relative flex items-center rounded-xl border transition-all duration-300 focus-within:border-cyan-500/50 focus-within:shadow-[0_0_20px_rgba(0,217,255,0.15)]"
        style={{
          backgroundColor: 'rgba(255, 255, 255, 0.03)',
          borderColor: luxuryColors.borderSubtle,
        }}
      >
        <input
          type="number"
          step="any"
          value={value}
          onChange={(e) => onChange(e.target.value)}
          placeholder={placeholder}
          className="w-full px-3 py-2.5 text-sm font-mono bg-transparent outline-none text-white placeholder:text-white/30"
        />
        {suffix && (
          <span className="px-3 text-xs font-medium" style={{ color: luxuryColors.textMuted }}>
            {suffix}
          </span>
        )}
      </div>
    </div>
  );
}

// ============================================================================
// PORTFOLIO STATS BAR - Premium Hero Stats
// ============================================================================

function PortfolioStatsBar({
  balance,
  equity,
  totalPnl,
  totalPnlPercent,
  winRate,
  totalTrades,
}: {
  balance: number;
  equity: number;
  totalPnl: number;
  totalPnlPercent: number;
  winRate: number;
  totalTrades: number;
}) {
  const isProfitable = totalPnl >= 0;

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      className="relative overflow-hidden"
      style={{ backgroundColor: luxuryColors.bgPrimary }}
    >
      {/* Background glow effect */}
      <div
        className="absolute top-0 right-0 w-64 h-64 rounded-full blur-3xl opacity-20 pointer-events-none"
        style={{
          background: isProfitable
            ? 'radial-gradient(circle, rgba(34, 197, 94, 0.4), transparent 70%)'
            : 'radial-gradient(circle, rgba(239, 68, 68, 0.4), transparent 70%)',
        }}
      />

      <div className="relative z-10 flex items-center gap-6 px-6 py-4 border-b border-white/[0.08]">
        {/* Balance - Hero stat */}
        <div className="flex items-center gap-3">
          <div
            className="p-2.5 rounded-xl"
            style={{
              background: 'rgba(0, 217, 255, 0.1)',
              border: '1px solid rgba(0, 217, 255, 0.2)',
            }}
          >
            <Wallet className="w-5 h-5" style={{ color: luxuryColors.cyan }} />
          </div>
          <div>
            <p className="text-[10px] uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>Balance</p>
            <GradientText className="text-xl font-black">
              ${balance.toLocaleString('en-US', { minimumFractionDigits: 2 })}
            </GradientText>
          </div>
        </div>

        <div className="h-10 w-px bg-white/[0.08]" />

        {/* Equity */}
        <div className="flex items-center gap-3">
          <div
            className="p-2 rounded-xl"
            style={{
              background: 'rgba(255, 255, 255, 0.05)',
              border: '1px solid rgba(255, 255, 255, 0.1)',
            }}
          >
            <Activity className="w-4 h-4" style={{ color: luxuryColors.textSecondary }} />
          </div>
          <div>
            <p className="text-[10px] uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>Equity</p>
            <MonoText className="text-lg font-bold">${equity.toLocaleString('en-US', { minimumFractionDigits: 2 })}</MonoText>
          </div>
        </div>

        <div className="h-10 w-px bg-white/[0.08]" />

        {/* PnL with glow */}
        <motion.div
          className="flex items-center gap-3 px-4 py-2 rounded-xl"
          style={{
            background: isProfitable ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)',
            border: isProfitable ? '1px solid rgba(34, 197, 94, 0.2)' : '1px solid rgba(239, 68, 68, 0.2)',
            boxShadow: isProfitable ? '0 0 20px rgba(34, 197, 94, 0.15)' : '0 0 20px rgba(239, 68, 68, 0.15)',
          }}
          animate={{
            boxShadow: isProfitable
              ? ['0 0 20px rgba(34, 197, 94, 0.15)', '0 0 30px rgba(34, 197, 94, 0.25)', '0 0 20px rgba(34, 197, 94, 0.15)']
              : ['0 0 20px rgba(239, 68, 68, 0.15)', '0 0 30px rgba(239, 68, 68, 0.25)', '0 0 20px rgba(239, 68, 68, 0.15)'],
          }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          {isProfitable ? (
            <TrendingUp className="w-5 h-5" style={{ color: luxuryColors.profit }} />
          ) : (
            <TrendingDown className="w-5 h-5" style={{ color: luxuryColors.loss }} />
          )}
          <div>
            <p className="text-[10px] uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>P&L</p>
            <MonoText className="text-lg font-bold" positive={isProfitable} negative={!isProfitable}>
              {isProfitable ? '+' : ''}${Math.abs(totalPnl).toFixed(2)} ({isProfitable ? '+' : ''}{totalPnlPercent.toFixed(2)}%)
            </MonoText>
          </div>
        </motion.div>

        {/* Stats */}
        <div className="flex items-center gap-4 ml-auto">
          <div className="text-center px-3">
            <p className="text-[10px] uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>Win Rate</p>
            <MonoText className="text-base font-bold" style={{ color: luxuryColors.cyan }}>{winRate.toFixed(1)}%</MonoText>
          </div>
          <div className="text-center px-3">
            <p className="text-[10px] uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>Trades</p>
            <MonoText className="text-base font-bold">{totalTrades}</MonoText>
          </div>

          {/* Paper Mode Badge */}
          <motion.div
            className="flex items-center gap-2 px-4 py-2 rounded-full"
            style={{
              background: 'rgba(0, 217, 255, 0.15)',
              border: '1px solid rgba(0, 217, 255, 0.3)',
            }}
            animate={{ scale: [1, 1.02, 1] }}
            transition={{ duration: 2, repeat: Infinity }}
          >
            <motion.div
              className="w-2 h-2 rounded-full"
              style={{ backgroundColor: luxuryColors.cyan }}
              animate={{ opacity: [1, 0.5, 1] }}
              transition={{ duration: 1.5, repeat: Infinity }}
            />
            <span className="text-xs font-bold uppercase tracking-wider" style={{ color: luxuryColors.cyan }}>
              Paper Mode
            </span>
          </motion.div>
        </div>
      </div>
    </motion.div>
  );
}

// ============================================================================
// ORDER BOOK COMPONENT - Premium Glass Design
// ============================================================================

interface OrderBookLevel {
  price: number;
  quantity: number;
  total: number;
}

function OrderBook({
  symbol = 'BTCUSDT',
  onPriceClick,
}: {
  symbol?: string;
  onPriceClick?: (price: number) => void;
}) {
  const [asks, setAsks] = useState<OrderBookLevel[]>([]);
  const [bids, setBids] = useState<OrderBookLevel[]>([]);
  const [spread, setSpread] = useState(0);
  const [spreadPercent, setSpreadPercent] = useState(0);
  const [midPrice, setMidPrice] = useState(0);
  const [isLoading, setIsLoading] = useState(true);

  // Fetch real price from Binance and generate realistic order book
  useEffect(() => {
    let cancelled = false;

    const loadOrderBook = async () => {
      if (cancelled) return;
      try {
        const realPrice = await fetchBinancePrice(symbol);
        if (!cancelled && realPrice > 0) {
          // Generate realistic order book around real Binance price
          const tickSize = realPrice > 10000 ? 0.1 : 0.01;
          const spreadTicks = realPrice > 10000 ? 5 : 10;
          const spread = tickSize * spreadTicks;

          const newAsks: OrderBookLevel[] = [];
          const newBids: OrderBookLevel[] = [];

          let askTotal = 0;
          let bidTotal = 0;

          for (let i = 0; i < 8; i++) {
            // Asks (sell orders) above mid price
            const askPrice = realPrice + spread / 2 + i * tickSize;
            const askQuantity = Math.random() * 2 + 0.1;
            askTotal += askQuantity;
            newAsks.push({
              price: askPrice,
              quantity: askQuantity,
              total: askTotal,
            });

            // Bids (buy orders) below mid price
            const bidPrice = realPrice - spread / 2 - i * tickSize;
            const bidQuantity = Math.random() * 2 + 0.1;
            bidTotal += bidQuantity;
            newBids.push({
              price: bidPrice,
              quantity: bidQuantity,
              total: bidTotal,
            });
          }

          setAsks(newAsks);
          setBids(newBids);
          setMidPrice(realPrice);

          // Calculate spread
          if (newAsks.length > 0 && newBids.length > 0) {
            const calcSpread = newAsks[0].price - newBids[0].price;
            setSpread(calcSpread);
            setSpreadPercent((calcSpread / realPrice) * 100);
          }

          setIsLoading(false);
        }
      } catch (error) {
        logger.error('Failed to fetch order book price:', error);
      }
    };

    // Initial load
    loadOrderBook();

    // Refresh order book every 2 seconds for realistic feel
    const interval = setInterval(loadOrderBook, 2000);

    return () => {
      cancelled = true;
      clearInterval(interval);
    };
  }, [symbol]);

  const maxAskTotal = Math.max(...asks.map((a) => a.total), 1);
  const maxBidTotal = Math.max(...bids.map((b) => b.total), 1);

  const OrderRow = ({
    level,
    type,
    maxTotal,
  }: {
    level: OrderBookLevel;
    type: 'ask' | 'bid';
    maxTotal: number;
  }) => {
    const isAsk = type === 'ask';
    const depthWidth = (level.total / maxTotal) * 100;
    const priceColor = isAsk ? luxuryColors.loss : luxuryColors.profit;
    const depthColor = isAsk ? 'rgba(239, 68, 68, 0.15)' : 'rgba(34, 197, 94, 0.15)';

    return (
      <motion.div
        className="relative grid grid-cols-3 gap-2 px-4 py-1.5 text-[11px] cursor-pointer transition-all duration-150 hover:bg-white/[0.05]"
        onClick={() => onPriceClick?.(level.price)}
        whileHover={{ x: isAsk ? -2 : 2 }}
      >
        <motion.div
          className="absolute inset-0 pointer-events-none"
          initial={{ width: 0 }}
          animate={{ width: `${depthWidth}%` }}
          transition={{ duration: 0.3 }}
          style={{
            background: isAsk
              ? `linear-gradient(to left, ${depthColor}, transparent)`
              : `linear-gradient(to right, ${depthColor}, transparent)`,
            [isAsk ? 'right' : 'left']: 0,
          }}
        />
        <div className="relative z-10 font-mono font-semibold" style={{ color: priceColor }}>
          {level.price.toFixed(2)}
        </div>
        <div className="relative z-10 text-right font-mono" style={{ color: luxuryColors.textPrimary }}>
          {level.quantity.toFixed(4)}
        </div>
        <div className="relative z-10 text-right font-mono" style={{ color: luxuryColors.textSecondary }}>
          {level.total.toFixed(4)}
        </div>
      </motion.div>
    );
  };

  return (
    <GlassCard noPadding>
      <PanelHeader title="Order Book" icon={BarChart3} />

      {/* Column Headers */}
      <div
        className="grid grid-cols-3 gap-2 px-4 py-2 text-[10px] uppercase tracking-wider border-b border-white/[0.08]"
        style={{ color: luxuryColors.textMuted }}
      >
        <div>Price (USDT)</div>
        <div className="text-right">Size</div>
        <div className="text-right">Total</div>
      </div>

      {/* Asks (reversed) */}
      <div className="flex flex-col-reverse">
        {asks.slice(0, 8).map((ask, i) => (
          <OrderRow key={`ask-${i}`} level={ask} type="ask" maxTotal={maxAskTotal} />
        ))}
      </div>

      {/* Spread / Mid Price - Premium styling */}
      <motion.div
        className="px-4 py-3 border-y border-white/[0.08] flex items-center justify-between"
        style={{
          background: 'linear-gradient(135deg, rgba(0, 217, 255, 0.1), rgba(34, 197, 94, 0.1))',
        }}
        animate={{
          boxShadow: ['0 0 0 rgba(0, 217, 255, 0)', '0 0 20px rgba(0, 217, 255, 0.1)', '0 0 0 rgba(0, 217, 255, 0)'],
        }}
        transition={{ duration: 2, repeat: Infinity }}
      >
        <GradientText className="text-lg font-black">
          {midPrice > 0 ? `$${midPrice.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}` : 'Loading...'}
        </GradientText>
        <span className="text-[10px]" style={{ color: luxuryColors.textMuted }}>
          Spread: <MonoText className="text-[10px]" style={{ color: luxuryColors.cyan }}>{spread.toFixed(2)}</MonoText> (
          <MonoText className="text-[10px]" style={{ color: luxuryColors.cyan }}>{spreadPercent.toFixed(4)}%</MonoText>)
        </span>
      </motion.div>

      {/* Bids */}
      <div>
        {bids.slice(0, 8).map((bid, i) => (
          <OrderRow key={`bid-${i}`} level={bid} type="bid" maxTotal={maxBidTotal} />
        ))}
      </div>
    </GlassCard>
  );
}

// ============================================================================
// ORDER FORM COMPONENT - Premium Glass Design
// ============================================================================

function OrderForm({
  symbol = 'BTCUSDT',
  onSubmit,
  selectedPrice,
}: {
  symbol?: string;
  onSubmit?: (order: OrderFormData) => void;
  selectedPrice?: number;
}) {
  const { toast } = useToast();
  const [side, setSide] = useState<'buy' | 'sell'>('buy');
  const [orderType, setOrderType] = useState<'market' | 'limit' | 'stop-limit'>('market');
  const [quantity, setQuantity] = useState('');
  const [stopPrice, setStopPrice] = useState('');
  const [leverage, setLeverage] = useState(10);

  // Price state - controlled input
  const [price, setPrice] = useState('');

  // Update price when selectedPrice changes from order book click (valid props-to-state sync)
  useEffect(() => {
    if (selectedPrice && orderType !== 'market') {
      // eslint-disable-next-line react-hooks/set-state-in-effect -- Valid sync pattern for controlled input
      setPrice(selectedPrice.toFixed(2));
    }
  }, [selectedPrice, orderType]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!quantity || parseFloat(quantity) <= 0) {
      toast({
        title: 'Invalid Quantity',
        description: 'Please enter a valid quantity',
        variant: 'destructive',
      });
      return;
    }

    if (orderType !== 'market' && (!price || parseFloat(price) <= 0)) {
      toast({
        title: 'Invalid Price',
        description: 'Please enter a valid limit price',
        variant: 'destructive',
      });
      return;
    }

    const orderData: OrderFormData = {
      symbol,
      orderType,
      side,
      quantity: parseFloat(quantity),
      leverage,
      ...(orderType !== 'market' && { price: parseFloat(price) }),
      ...(orderType === 'stop-limit' && { stopPrice: parseFloat(stopPrice) }),
    };

    logger.info('Paper trading order submitted:', orderData);
    onSubmit?.(orderData);
  };

  const orderValue = quantity && price ? parseFloat(quantity) * parseFloat(price) : 0;
  const isBuy = side === 'buy';

  return (
    <GlassCard noPadding>
      <PanelHeader title="Place Order" icon={Target} />

      {/* Buy/Sell Toggle - Premium styling */}
      <div className="p-4 border-b border-white/[0.08]">
        <div className="relative flex rounded-xl overflow-hidden bg-white/[0.03]">
          <motion.div
            className="absolute top-0 bottom-0 w-1/2 rounded-xl"
            style={{
              background: isBuy ? luxuryColors.gradientProfit : luxuryColors.gradientLoss,
              boxShadow: isBuy
                ? '0 4px 20px rgba(34, 197, 94, 0.4)'
                : '0 4px 20px rgba(239, 68, 68, 0.4)',
            }}
            animate={{ x: isBuy ? 0 : '100%' }}
            transition={{ type: 'spring', stiffness: 400, damping: 35 }}
          />
          <button
            type="button"
            onClick={() => setSide('buy')}
            className="relative z-10 flex-1 py-3 text-xs font-bold transition-colors flex items-center justify-center gap-2"
            style={{ color: isBuy ? '#fff' : luxuryColors.textSecondary }}
          >
            <TrendingUp className="w-4 h-4" />
            Buy / Long
          </button>
          <button
            type="button"
            onClick={() => setSide('sell')}
            className="relative z-10 flex-1 py-3 text-xs font-bold transition-colors flex items-center justify-center gap-2"
            style={{ color: !isBuy ? '#fff' : luxuryColors.textSecondary }}
          >
            <TrendingDown className="w-4 h-4" />
            Sell / Short
          </button>
        </div>
      </div>

      <form onSubmit={handleSubmit} className="p-4 space-y-4">
        {/* Order Type Tabs */}
        <div className="flex gap-1 p-1 rounded-xl bg-white/[0.03] border border-white/[0.08]">
          {(['market', 'limit', 'stop-limit'] as const).map((type) => (
            <button
              key={type}
              type="button"
              onClick={() => setOrderType(type)}
              className="flex-1 py-2 text-[10px] font-bold uppercase tracking-wider rounded-lg transition-all duration-300"
              style={{
                background: orderType === type
                  ? 'linear-gradient(135deg, rgba(0, 217, 255, 0.2), rgba(34, 197, 94, 0.2))'
                  : 'transparent',
                border: orderType === type ? '1px solid rgba(0, 217, 255, 0.3)' : '1px solid transparent',
                color: orderType === type ? luxuryColors.cyan : luxuryColors.textMuted,
              }}
            >
              {type}
            </button>
          ))}
        </div>

        {/* Price Inputs */}
        {orderType !== 'market' && (
          <InputField
            label="Price"
            value={price}
            onChange={setPrice}
            placeholder="0.00"
            suffix="USDT"
          />
        )}

        {orderType === 'stop-limit' && (
          <InputField
            label="Stop Price"
            value={stopPrice}
            onChange={setStopPrice}
            placeholder="0.00"
            suffix="USDT"
          />
        )}

        {/* Quantity */}
        <InputField
          label="Quantity"
          value={quantity}
          onChange={setQuantity}
          placeholder="0.0000"
          suffix="BTC"
        />

        {/* Leverage - Premium styling */}
        <div>
          <label className="block text-[10px] uppercase tracking-wider mb-2" style={{ color: luxuryColors.textMuted }}>
            Leverage: <GradientText className="text-xs font-bold">{leverage}x</GradientText>
          </label>
          <div className="flex flex-wrap gap-1.5">
            {[1, 2, 5, 10, 20, 50, 100].map((lev) => (
              <motion.button
                key={lev}
                type="button"
                onClick={() => setLeverage(lev)}
                whileHover={{ scale: 1.05 }}
                whileTap={{ scale: 0.95 }}
                className="px-3 py-1.5 text-[10px] font-bold rounded-lg transition-all duration-300"
                style={{
                  background: leverage === lev
                    ? 'linear-gradient(135deg, rgba(0, 217, 255, 0.2), rgba(34, 197, 94, 0.2))'
                    : 'rgba(255, 255, 255, 0.03)',
                  border: leverage === lev
                    ? '1px solid rgba(0, 217, 255, 0.3)'
                    : '1px solid rgba(255, 255, 255, 0.08)',
                  color: leverage === lev ? luxuryColors.cyan : luxuryColors.textMuted,
                }}
              >
                {lev}x
              </motion.button>
            ))}
          </div>
        </div>

        {/* Order Summary - Premium glass panel */}
        <div
          className="p-4 rounded-xl space-y-2 text-xs"
          style={{
            background: 'rgba(255, 255, 255, 0.03)',
            border: '1px solid rgba(255, 255, 255, 0.08)',
          }}
        >
          <div className="flex justify-between items-center">
            <span style={{ color: luxuryColors.textMuted }}>Order Value</span>
            <MonoText className="font-semibold">{orderValue > 0 ? `$${orderValue.toFixed(2)}` : '--'}</MonoText>
          </div>
          <div className="flex justify-between items-center">
            <span style={{ color: luxuryColors.textMuted }}>With Leverage ({leverage}x)</span>
            <GradientText className="font-bold">
              {orderValue > 0 ? `$${(orderValue * leverage).toFixed(2)}` : '--'}
            </GradientText>
          </div>
        </div>

        {/* Submit Button - Premium with glow */}
        <motion.button
          type="submit"
          whileHover={{ scale: 1.02, y: -2 }}
          whileTap={{ scale: 0.98 }}
          className="w-full py-3.5 rounded-xl font-bold text-sm text-white transition-all duration-300 flex items-center justify-center gap-2"
          style={{
            background: isBuy ? luxuryColors.gradientProfit : luxuryColors.gradientLoss,
            boxShadow: isBuy
              ? '0 8px 32px rgba(34, 197, 94, 0.4)'
              : '0 8px 32px rgba(239, 68, 68, 0.4)',
          }}
        >
          {isBuy ? (
            <>
              <TrendingUp className="w-5 h-5" />
              Buy / Long {symbol.replace('USDT', '')}
            </>
          ) : (
            <>
              <TrendingDown className="w-5 h-5" />
              Sell / Short {symbol.replace('USDT', '')}
            </>
          )}
        </motion.button>
      </form>
    </GlassCard>
  );
}

// ============================================================================
// POSITIONS TABLE - Premium Glass Design
// ============================================================================

function PositionsTable({
  trades,
  isLoading,
  onCloseTrade,
}: {
  trades: PaperTrade[];
  isLoading: boolean;
  onCloseTrade?: (id: string) => void;
}) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
        >
          <RefreshCw className="w-6 h-6" style={{ color: luxuryColors.cyan }} />
        </motion.div>
      </div>
    );
  }

  if (trades.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12" style={{ color: luxuryColors.textMuted }}>
        <div
          className="p-4 rounded-2xl mb-3"
          style={{
            background: 'rgba(255, 255, 255, 0.03)',
            border: '1px solid rgba(255, 255, 255, 0.08)',
          }}
        >
          <Activity className="w-8 h-8 opacity-50" />
        </div>
        <p className="text-sm font-medium">No open positions</p>
        <p className="text-xs mt-1" style={{ color: luxuryColors.textMuted }}>
          Place an order to start trading
        </p>
      </div>
    );
  }

  return (
    <div
      className="overflow-x-auto overflow-y-auto custom-scrollbar h-full"
      style={{
        scrollbarWidth: 'thin',
        scrollbarColor: 'rgba(0, 217, 255, 0.3) rgba(255, 255, 255, 0.05)',
      }}
    >
      <table className="w-full text-xs">
        <thead className="sticky top-0 z-10" style={{ backgroundColor: luxuryColors.bgPrimary }}>
          <tr style={{ color: luxuryColors.textMuted }}>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">Symbol</th>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">Side</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">Entry</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">Size</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">P&L</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">Action</th>
          </tr>
        </thead>
        <tbody>
          {trades.map((trade, index) => {
            const isProfitable = (trade.pnl || 0) >= 0;
            return (
              <motion.tr
                key={trade.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.05 }}
                whileHover={{ backgroundColor: 'rgba(255, 255, 255, 0.05)' }}
                className="border-t border-white/[0.06] transition-all duration-300"
              >
                <td className="py-3 px-4">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-white">{trade.symbol.replace('USDT', '')}</span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'rgba(0, 217, 255, 0.1)', color: luxuryColors.cyan }}>
                      {trade.leverage}x
                    </span>
                  </div>
                </td>
                <td className="py-3 px-4">
                  <Badge variant={trade.trade_type === 'Long' ? 'buy' : 'sell'}>
                    {trade.trade_type === 'Long' ? (
                      <TrendingUp className="w-3 h-3 mr-1" />
                    ) : (
                      <TrendingDown className="w-3 h-3 mr-1" />
                    )}
                    {trade.trade_type}
                  </Badge>
                </td>
                <td className="py-3 px-4 text-right">
                  <MonoText className="font-semibold">${trade.entry_price.toFixed(2)}</MonoText>
                </td>
                <td className="py-3 px-4 text-right">
                  <MonoText>{trade.quantity.toFixed(4)}</MonoText>
                </td>
                <td className="py-3 px-4 text-right">
                  <div
                    className="inline-flex flex-col items-end px-2 py-1 rounded-lg"
                    style={{
                      background: isProfitable ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)',
                    }}
                  >
                    <MonoText positive={isProfitable} negative={!isProfitable} className="font-bold">
                      {isProfitable ? '+' : ''}${(trade.pnl || 0).toFixed(2)}
                    </MonoText>
                    <MonoText positive={isProfitable} negative={!isProfitable} className="text-[10px]">
                      ({isProfitable ? '+' : ''}{trade.pnl_percentage.toFixed(2)}%)
                    </MonoText>
                  </div>
                </td>
                <td className="py-3 px-4 text-right">
                  <motion.button
                    onClick={() => onCloseTrade?.(trade.id)}
                    whileHover={{ scale: 1.05 }}
                    whileTap={{ scale: 0.95 }}
                    className="px-3 py-1.5 text-[10px] font-bold rounded-lg transition-all duration-300 flex items-center gap-1"
                    style={{
                      background: 'rgba(239, 68, 68, 0.15)',
                      border: '1px solid rgba(239, 68, 68, 0.3)',
                      color: luxuryColors.loss,
                    }}
                  >
                    <X className="w-3 h-3" />
                    Close
                  </motion.button>
                </td>
              </motion.tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

// ============================================================================
// TRADE HISTORY TABLE - Premium Glass Design
// ============================================================================

function TradeHistoryTable({
  trades,
  isLoading,
}: {
  trades: PaperTrade[];
  isLoading: boolean;
}) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
        >
          <RefreshCw className="w-6 h-6" style={{ color: luxuryColors.cyan }} />
        </motion.div>
      </div>
    );
  }

  if (trades.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12" style={{ color: luxuryColors.textMuted }}>
        <div
          className="p-4 rounded-2xl mb-3"
          style={{
            background: 'rgba(255, 255, 255, 0.03)',
            border: '1px solid rgba(255, 255, 255, 0.08)',
          }}
        >
          <Clock className="w-8 h-8 opacity-50" />
        </div>
        <p className="text-sm font-medium">No trade history</p>
        <p className="text-xs mt-1" style={{ color: luxuryColors.textMuted }}>
          Closed trades will appear here
        </p>
      </div>
    );
  }

  return (
    <div
      className="overflow-x-auto overflow-y-auto custom-scrollbar h-full"
      style={{
        scrollbarWidth: 'thin',
        scrollbarColor: 'rgba(0, 217, 255, 0.3) rgba(255, 255, 255, 0.05)',
      }}
    >
      <table className="w-full text-xs">
        <thead className="sticky top-0 z-10" style={{ backgroundColor: luxuryColors.bgPrimary }}>
          <tr style={{ color: luxuryColors.textMuted }}>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">Symbol</th>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">Side</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">Entry</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">Exit</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">P&L</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">Time</th>
          </tr>
        </thead>
        <tbody>
          {trades.map((trade, index) => {
            const isProfitable = (trade.pnl || 0) >= 0;
            const closeTime = trade.close_time ? new Date(trade.close_time) : new Date();
            return (
              <motion.tr
                key={trade.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.05 }}
                whileHover={{ backgroundColor: 'rgba(255, 255, 255, 0.05)' }}
                className="border-t border-white/[0.06] transition-all duration-300"
              >
                <td className="py-3 px-4">
                  <span className="font-bold text-white">{trade.symbol.replace('USDT', '')}</span>
                </td>
                <td className="py-3 px-4">
                  <Badge variant={trade.trade_type === 'Long' ? 'buy' : 'sell'}>
                    {trade.trade_type === 'Long' ? (
                      <TrendingUp className="w-3 h-3 mr-1" />
                    ) : (
                      <TrendingDown className="w-3 h-3 mr-1" />
                    )}
                    {trade.trade_type}
                  </Badge>
                </td>
                <td className="py-3 px-4 text-right">
                  <MonoText className="font-semibold">${trade.entry_price.toFixed(2)}</MonoText>
                </td>
                <td className="py-3 px-4 text-right">
                  <MonoText className="font-semibold">${trade.exit_price?.toFixed(2) || '--'}</MonoText>
                </td>
                <td className="py-3 px-4 text-right">
                  <div
                    className="inline-flex flex-col items-end px-2 py-1 rounded-lg"
                    style={{
                      background: isProfitable ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)',
                    }}
                  >
                    <MonoText positive={isProfitable} negative={!isProfitable} className="font-bold">
                      {isProfitable ? '+' : ''}${(trade.pnl || 0).toFixed(2)}
                    </MonoText>
                  </div>
                </td>
                <td className="py-3 px-4 text-right" style={{ color: luxuryColors.textSecondary }}>
                  <div className="flex items-center justify-end gap-1.5">
                    <Clock className="w-3 h-3" style={{ color: luxuryColors.textMuted }} />
                    {formatDistanceToNow(closeTime, { addSuffix: true })}
                  </div>
                </td>
              </motion.tr>
            );
          })}
        </tbody>
      </table>
    </div>
  );
}

// ============================================================================
// MAIN PAPER TRADING PAGE - Premium Luxury Design
// ============================================================================

export default function PaperTrading() {
  const { toast } = useToast();
  const paperTrading = usePaperTrading();
  const [selectedSymbol, setSelectedSymbol] = useState('BTCUSDT');
  const availableSymbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT', 'XRPUSDT', 'ADAUSDT'];
  const [showSymbolDropdown, setShowSymbolDropdown] = useState(false);
  const [selectedPrice, setSelectedPrice] = useState<number | undefined>();
  const [activeTab, setActiveTab] = useState<'positions' | 'history'>('positions');
  const [selectedTimeframe, setSelectedTimeframe] = useState('5m');

  const timeframes = ['1m', '5m', '15m', '1h', '4h', '1D'];

  // @spec:FR-PAPER-003 - Manual Order Placement
  const handleOrderSubmit = async (order: OrderFormData) => {
    logger.info('Paper trading order submitted:', order);

    // Map OrderFormData to PlaceOrderRequest format
    const result = await paperTrading.placeOrder({
      symbol: order.symbol,
      side: order.side,
      order_type: order.orderType === 'stop-limit' ? 'limit' : order.orderType,
      quantity: order.quantity,
      price: order.price,
      stop_price: order.stopPrice,
      leverage: order.leverage,
    });

    if (result) {
      toast({
        title: 'Order Executed',
        description: `${order.side.toUpperCase()} ${order.quantity} ${order.symbol} @ $${result.entry_price.toFixed(2)}`,
      });
    } else {
      toast({
        title: 'Order Failed',
        description: paperTrading.error || 'Failed to place order. Please try again.',
        variant: 'destructive',
      });
    }
  };

  const handlePriceClick = (price: number) => {
    setSelectedPrice(price);
  };

  return (
    <motion.div
      className="h-full flex flex-col"
      style={{ backgroundColor: luxuryColors.bgPrimary }}
      initial="hidden"
      animate="visible"
      variants={containerVariants}
    >
      {/* Portfolio Stats Bar */}
      <PortfolioStatsBar
        balance={paperTrading.portfolio.current_balance}
        equity={paperTrading.portfolio.equity}
        totalPnl={paperTrading.portfolio.total_pnl}
        totalPnlPercent={paperTrading.portfolio.total_pnl_percentage}
        winRate={paperTrading.portfolio.win_rate}
        totalTrades={paperTrading.portfolio.total_trades}
      />

      {/* Main Trading Grid */}
      <div className="flex-1 grid grid-cols-12 gap-[1px] min-h-0 overflow-hidden" style={{ backgroundColor: luxuryColors.borderSubtle }}>
        {/* Left Column: Chart (60%) */}
        <div
          className="col-span-7 flex flex-col overflow-y-auto custom-scrollbar"
          style={{ backgroundColor: luxuryColors.bgPrimary }}
        >
          {/* Chart Header - Premium styling */}
          <motion.div
            className="flex items-center justify-between px-4 py-3 border-b border-white/[0.08]"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
          >
            <div className="flex items-center gap-4">
              <div className="relative flex items-center gap-3">
                <div
                  className="p-2 rounded-xl"
                  style={{
                    background: 'rgba(0, 217, 255, 0.1)',
                    border: '1px solid rgba(0, 217, 255, 0.2)',
                  }}
                >
                  <LineChart className="w-4 h-4" style={{ color: luxuryColors.cyan }} />
                </div>
                {/* Symbol Selector Dropdown */}
                <motion.button
                  onClick={() => setShowSymbolDropdown(!showSymbolDropdown)}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="flex items-center gap-2 px-3 py-1.5 rounded-xl cursor-pointer transition-all duration-200"
                  style={{
                    background: showSymbolDropdown ? 'rgba(0, 217, 255, 0.15)' : 'rgba(255, 255, 255, 0.05)',
                    border: `1px solid ${showSymbolDropdown ? 'rgba(0, 217, 255, 0.3)' : 'rgba(255, 255, 255, 0.1)'}`,
                  }}
                >
                  <div className="flex items-center">
                    <GradientText className="text-lg font-black">
                      {selectedSymbol.replace('USDT', '')}
                    </GradientText>
                    <span className="text-xs font-medium" style={{ color: luxuryColors.textMuted }}>/USDT</span>
                  </div>
                  <ChevronDown
                    className={`w-4 h-4 transition-transform duration-200 ${showSymbolDropdown ? 'rotate-180' : ''}`}
                    style={{ color: luxuryColors.textMuted }}
                  />
                </motion.button>

                {/* Dropdown Menu */}
                <AnimatePresence>
                  {showSymbolDropdown && (
                    <>
                      {/* Backdrop */}
                      <div
                        className="fixed inset-0 z-40"
                        onClick={() => setShowSymbolDropdown(false)}
                      />
                      {/* Dropdown */}
                      <motion.div
                        initial={{ opacity: 0, y: -10, scale: 0.95 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: -10, scale: 0.95 }}
                        transition={{ duration: 0.15 }}
                        className="absolute top-full left-0 mt-2 z-50 min-w-[180px] rounded-xl overflow-hidden"
                        style={{
                          backgroundColor: luxuryColors.bgPrimary,
                          border: `1px solid rgba(255, 255, 255, 0.1)`,
                          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.6)',
                        }}
                      >
                        {availableSymbols.map((symbol) => (
                          <motion.button
                            key={symbol}
                            onClick={() => {
                              setSelectedSymbol(symbol);
                              setShowSymbolDropdown(false);
                            }}
                            whileHover={{ backgroundColor: 'rgba(255, 255, 255, 0.08)' }}
                            className="w-full flex items-center justify-between px-4 py-3 text-sm font-medium transition-colors"
                            style={{
                              color: selectedSymbol === symbol ? luxuryColors.cyan : luxuryColors.textSecondary,
                              backgroundColor: selectedSymbol === symbol ? 'rgba(0, 217, 255, 0.1)' : 'transparent',
                            }}
                          >
                            <span className="font-bold">{symbol.replace('USDT', '')}</span>
                            <span className="text-xs" style={{ color: luxuryColors.textMuted }}>/USDT</span>
                          </motion.button>
                        ))}
                      </motion.div>
                    </>
                  )}
                </AnimatePresence>
              </div>
              <Badge variant="info">Perpetual</Badge>
            </div>

            {/* Timeframe buttons - Premium styling */}
            <div className="flex items-center gap-1 p-1 rounded-xl bg-white/[0.03] border border-white/[0.08]">
              {timeframes.map((tf) => (
                <motion.button
                  key={tf}
                  onClick={() => setSelectedTimeframe(tf)}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  className="px-3 py-1.5 text-[10px] font-bold uppercase tracking-wider rounded-lg transition-all duration-300"
                  style={{
                    background: selectedTimeframe === tf
                      ? 'linear-gradient(135deg, rgba(0, 217, 255, 0.2), rgba(34, 197, 94, 0.2))'
                      : 'transparent',
                    border: selectedTimeframe === tf ? '1px solid rgba(0, 217, 255, 0.3)' : '1px solid transparent',
                    color: selectedTimeframe === tf ? luxuryColors.cyan : luxuryColors.textMuted,
                  }}
                >
                  {tf}
                </motion.button>
              ))}
            </div>
          </motion.div>

          {/* Chart */}
          <div className="flex-1 min-h-0">
            <TradingViewChart
              symbol={selectedSymbol}
              timeframe={selectedTimeframe}
              showControls={false}
            />
          </div>

          {/* Positions/History Tabs - Premium styling with proper scroll */}
          <div
            className="rounded-t-2xl flex flex-col"
            style={{
              backgroundColor: 'rgba(255, 255, 255, 0.02)',
              minHeight: '250px',  // Minimum height
              maxHeight: '40vh',   // Max 40% of viewport for positions
              flex: '0 0 auto',    // Don't grow, don't shrink from initial
            }}
          >
            <div className="flex border-b border-white/[0.08]">
              {[
                { id: 'positions', label: 'Positions', icon: Activity, count: paperTrading.openTrades.length },
                { id: 'history', label: 'Trade History', icon: Clock },
              ].map((tab) => (
                <motion.button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as 'positions' | 'history')}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="relative px-5 py-3 text-xs font-bold transition-all duration-300 flex items-center gap-2"
                  style={{
                    color: activeTab === tab.id ? luxuryColors.cyan : luxuryColors.textMuted,
                  }}
                >
                  <tab.icon className="w-4 h-4" />
                  {tab.label}
                  {tab.count !== undefined && (
                    <span
                      className="px-2 py-0.5 text-[10px] rounded-full font-bold"
                      style={{
                        background: activeTab === tab.id ? 'rgba(0, 217, 255, 0.2)' : 'rgba(255, 255, 255, 0.1)',
                        color: activeTab === tab.id ? luxuryColors.cyan : luxuryColors.textSecondary,
                      }}
                    >
                      {tab.count}
                    </span>
                  )}

                  {/* Active indicator */}
                  {activeTab === tab.id && (
                    <motion.div
                      layoutId="activeTabIndicator"
                      className="absolute bottom-0 left-0 right-0 h-[2px]"
                      style={{
                        background: luxuryColors.gradientPremium,
                        boxShadow: '0 0 10px rgba(0, 217, 255, 0.5)',
                      }}
                    />
                  )}
                </motion.button>
              ))}
            </div>

            <div className="flex-1 min-h-0 overflow-hidden">
              <AnimatePresence mode="wait">
                {activeTab === 'positions' ? (
                  <motion.div
                    key="positions"
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -10 }}
                    transition={{ duration: 0.2 }}
                    className="h-full overflow-auto"
                    style={{
                      scrollbarWidth: 'thin',
                      scrollbarColor: 'rgba(0, 217, 255, 0.3) rgba(255, 255, 255, 0.05)',
                    }}
                  >
                    <PositionsTable
                      trades={paperTrading.openTrades}
                      isLoading={paperTrading.isLoading}
                      onCloseTrade={paperTrading.closeTrade}
                    />
                  </motion.div>
                ) : (
                  <motion.div
                    key="history"
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    exit={{ opacity: 0, y: -10 }}
                    transition={{ duration: 0.2 }}
                    className="h-full overflow-auto"
                    style={{
                      scrollbarWidth: 'thin',
                      scrollbarColor: 'rgba(0, 217, 255, 0.3) rgba(255, 255, 255, 0.05)',
                    }}
                  >
                    <TradeHistoryTable
                      trades={paperTrading.closedTrades}
                      isLoading={paperTrading.isLoading}
                    />
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </div>
        </div>

        {/* Right Column: Order Book + Form (40%) */}
        <div
          className="col-span-5 flex flex-col overflow-y-auto"
          style={{ backgroundColor: luxuryColors.bgPrimary }}
        >
          <div className="grid grid-cols-2 gap-[1px] h-full" style={{ backgroundColor: luxuryColors.borderSubtle }}>
            {/* Order Book */}
            <div style={{ backgroundColor: luxuryColors.bgPrimary }}>
              <OrderBook symbol={selectedSymbol} onPriceClick={handlePriceClick} />
            </div>

            {/* Order Form */}
            <div style={{ backgroundColor: luxuryColors.bgPrimary }}>
              <OrderForm
                symbol={selectedSymbol}
                onSubmit={handleOrderSubmit}
                selectedPrice={selectedPrice}
              />

              {/* Risk Warning - Premium styling */}
              <div className="p-4">
                <motion.div
                  className="p-4 rounded-xl"
                  style={{
                    background: 'linear-gradient(135deg, rgba(245, 158, 11, 0.1), rgba(251, 133, 0, 0.05))',
                    border: '1px solid rgba(245, 158, 11, 0.2)',
                  }}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.3 }}
                >
                  <div className="flex items-start gap-3">
                    <div
                      className="p-2 rounded-lg flex-shrink-0"
                      style={{
                        background: 'rgba(245, 158, 11, 0.15)',
                        border: '1px solid rgba(245, 158, 11, 0.3)',
                      }}
                    >
                      <Shield className="w-4 h-4" style={{ color: luxuryColors.warning }} />
                    </div>
                    <div>
                      <p className="text-xs font-bold" style={{ color: luxuryColors.warning }}>
                        Paper Trading Mode
                      </p>
                      <p className="text-[10px] mt-1.5 leading-relaxed" style={{ color: luxuryColors.textSecondary }}>
                        This is a simulation environment. No real funds are at risk. Use this mode to practice strategies and test your skills.
                      </p>
                    </div>
                  </div>
                </motion.div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </motion.div>
  );
}
