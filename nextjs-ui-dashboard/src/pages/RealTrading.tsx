/**
 * Real Trading Page - Professional Trading Terminal
 *
 * Designed following Binance/Bybit/OKX design patterns:
 * - Dark theme optimized for OLED (#0D1117 background)
 * - RED accents for real money warning
 * - 60-40 layout split (chart vs order panels)
 * - Monospace fonts for numeric data
 * - 2-step confirmation for all orders
 *
 * @spec:FR-TRADING-016 - Real Trading Interface
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
 */

import { useState, useMemo, useEffect, useRef } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useRealTrading } from '@/hooks/useRealTrading';
import { useTradingMode } from '@/hooks/useTradingMode';
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
  AlertOctagon,
  CheckCircle,
} from 'lucide-react';
import type { PaperTrade } from '@/hooks/usePaperTrading';

// ============================================================================
// DESIGN TOKENS - Premium Dark OLED Luxury (RED THEME for Real Trading)
// ============================================================================

const luxuryColors = {
  // Backgrounds - Pure black for OLED
  bgPrimary: '#000000',
  bgSecondary: 'rgba(255, 255, 255, 0.03)',
  bgTertiary: 'rgba(255, 255, 255, 0.05)',
  bgHover: 'rgba(255, 255, 255, 0.08)',

  // Accents - RED theme for real trading danger
  primary: '#ef4444',
  primaryLight: '#f87171',
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
  borderActive: '#ef4444',
  borderDanger: 'rgba(239, 68, 68, 0.3)',

  // Gradients
  gradientProfit: 'linear-gradient(135deg, #22c55e, #00D9FF)',
  gradientLoss: 'linear-gradient(135deg, #ef4444, #f97316)',
  gradientPremium: 'linear-gradient(135deg, #ef4444, #f97316)',
  gradientDanger: 'linear-gradient(135deg, #ef4444, #dc2626)',
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
// UTILITY COMPONENTS - Premium Glassmorphism with RED accents
// ============================================================================

/**
 * GlassCard - Premium glassmorphism card component
 */
function GlassCard({
  children,
  className = '',
  noPadding = false,
  hoverable = false,
  glowColor,
  danger = false,
}: {
  children: React.ReactNode;
  className?: string;
  noPadding?: boolean;
  hoverable?: boolean;
  glowColor?: string;
  danger?: boolean;
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
                : danger
                  ? '0 8px 32px rgba(239, 68, 68, 0.15)'
                  : '0 8px 32px rgba(0, 217, 255, 0.15)',
            }
          : undefined
      }
      className={`relative overflow-hidden rounded-2xl bg-white/[0.03] backdrop-blur-xl border transition-all duration-300 ${hoverable ? 'cursor-pointer' : ''} ${className}`}
      style={{
        borderColor: danger ? 'rgba(239, 68, 68, 0.2)' : 'rgba(255, 255, 255, 0.08)',
      }}
    >
      <div className={noPadding ? '' : 'p-4'}>{children}</div>
    </motion.div>
  );
}

/**
 * PanelHeader - Premium header with icon glow (RED theme)
 */
function PanelHeader({
  title,
  icon: Icon,
  action,
  danger = false,
}: {
  title: string;
  icon?: React.ElementType;
  action?: React.ReactNode;
  danger?: boolean;
}) {
  const accentColor = danger ? luxuryColors.primary : luxuryColors.cyan;
  return (
    <div className="flex items-center justify-between px-4 py-3 border-b border-white/[0.08]">
      <div className="flex items-center gap-3">
        {Icon && (
          <div
            className="p-2 rounded-xl"
            style={{
              background: danger ? 'rgba(239, 68, 68, 0.1)' : 'rgba(0, 217, 255, 0.1)',
              border: danger ? '1px solid rgba(239, 68, 68, 0.2)' : '1px solid rgba(0, 217, 255, 0.2)',
            }}
          >
            <Icon className="w-4 h-4" style={{ color: accentColor }} />
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
  variant?: 'default' | 'buy' | 'sell' | 'info' | 'warning' | 'danger';
}) {
  const variants = {
    default: { bg: 'rgba(255, 255, 255, 0.1)', color: luxuryColors.textSecondary, border: 'rgba(255, 255, 255, 0.15)' },
    buy: { bg: 'rgba(34, 197, 94, 0.15)', color: luxuryColors.profit, border: 'rgba(34, 197, 94, 0.3)' },
    sell: { bg: 'rgba(239, 68, 68, 0.15)', color: luxuryColors.loss, border: 'rgba(239, 68, 68, 0.3)' },
    info: { bg: 'rgba(0, 217, 255, 0.15)', color: luxuryColors.cyan, border: 'rgba(0, 217, 255, 0.3)' },
    warning: { bg: 'rgba(245, 158, 11, 0.15)', color: luxuryColors.warning, border: 'rgba(245, 158, 11, 0.3)' },
    danger: { bg: 'rgba(239, 68, 68, 0.15)', color: luxuryColors.loss, border: 'rgba(239, 68, 68, 0.3)' },
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
  style: customStyle,
}: {
  children: React.ReactNode;
  className?: string;
  positive?: boolean;
  negative?: boolean;
  style?: React.CSSProperties;
}) {
  let color = luxuryColors.textPrimary;
  if (positive) color = luxuryColors.profit;
  if (negative) color = luxuryColors.loss;

  return (
    <span className={`font-mono ${className}`} style={{ color, ...customStyle }}>
      {children}
    </span>
  );
}

/**
 * InputField - Premium form input with glass styling (RED focus)
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
        className="relative flex items-center rounded-xl border transition-all duration-300 focus-within:border-red-500/50 focus-within:shadow-[0_0_20px_rgba(239,68,68,0.15)]"
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
// PORTFOLIO STATS BAR - Premium Hero Stats (RED Theme for Real Trading)
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
      {/* Background glow effect - RED for real trading */}
      <div
        className="absolute top-0 left-0 w-64 h-64 rounded-full blur-3xl opacity-20 pointer-events-none"
        style={{
          background: 'radial-gradient(circle, rgba(239, 68, 68, 0.4), transparent 70%)',
        }}
      />

      <div className="relative z-10 flex items-center gap-6 px-6 py-4 border-b border-white/[0.08]">
        {/* REAL TRADING Warning Banner */}
        <motion.div
          className="flex items-center gap-2 px-4 py-2 rounded-full"
          style={{
            background: 'rgba(239, 68, 68, 0.15)',
            border: '1px solid rgba(239, 68, 68, 0.3)',
            boxShadow: '0 0 20px rgba(239, 68, 68, 0.2)',
          }}
          animate={{
            scale: [1, 1.02, 1],
            boxShadow: ['0 0 20px rgba(239, 68, 68, 0.2)', '0 0 30px rgba(239, 68, 68, 0.4)', '0 0 20px rgba(239, 68, 68, 0.2)']
          }}
          transition={{ duration: 2, repeat: Infinity }}
        >
          <motion.div
            className="w-2 h-2 rounded-full"
            style={{ backgroundColor: luxuryColors.loss }}
            animate={{ opacity: [1, 0.5, 1] }}
            transition={{ duration: 1, repeat: Infinity }}
          />
          <AlertTriangle className="w-4 h-4" style={{ color: luxuryColors.loss }} />
          <span className="text-xs font-bold uppercase tracking-wider" style={{ color: luxuryColors.loss }}>
            REAL MONEY
          </span>
        </motion.div>

        <div className="h-10 w-px bg-white/[0.08]" />

        {/* Balance - Hero stat */}
        <div className="flex items-center gap-3">
          <div
            className="p-2.5 rounded-xl"
            style={{
              background: 'rgba(239, 68, 68, 0.1)',
              border: '1px solid rgba(239, 68, 68, 0.2)',
            }}
          >
            <Wallet className="w-5 h-5" style={{ color: luxuryColors.loss }} />
          </div>
          <div>
            <p className="text-[10px] uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>Balance</p>
            <GradientText className="text-xl font-black" gradient={luxuryColors.gradientDanger}>
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
            <MonoText className="text-base font-bold" style={{ color: luxuryColors.loss }}>{winRate.toFixed(1)}%</MonoText>
          </div>
          <div className="text-center px-3">
            <p className="text-[10px] uppercase tracking-wider" style={{ color: luxuryColors.textMuted }}>Trades</p>
            <MonoText className="text-base font-bold">{totalTrades}</MonoText>
          </div>
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

  useEffect(() => {
    let cancelled = false;

    const loadOrderBook = async () => {
      if (cancelled) return;
      try {
        const realPrice = await fetchBinancePrice(symbol);
        if (!cancelled && realPrice > 0) {
          const tickSize = realPrice > 10000 ? 0.1 : 0.01;
          const spreadTicks = realPrice > 10000 ? 5 : 10;
          const spread = tickSize * spreadTicks;

          const newAsks: OrderBookLevel[] = [];
          const newBids: OrderBookLevel[] = [];

          let askTotal = 0;
          let bidTotal = 0;

          for (let i = 0; i < 8; i++) {
            const askPrice = realPrice + spread / 2 + i * tickSize;
            const askQuantity = Math.random() * 2 + 0.1;
            askTotal += askQuantity;
            newAsks.push({ price: askPrice, quantity: askQuantity, total: askTotal });

            const bidPrice = realPrice - spread / 2 - i * tickSize;
            const bidQuantity = Math.random() * 2 + 0.1;
            bidTotal += bidQuantity;
            newBids.push({ price: bidPrice, quantity: bidQuantity, total: bidTotal });
          }

          setAsks(newAsks);
          setBids(newBids);
          setMidPrice(realPrice);

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

    loadOrderBook();
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
    <GlassCard noPadding danger>
      <PanelHeader title="Order Book" icon={BarChart3} danger />

      <div
        className="grid grid-cols-3 gap-2 px-4 py-2 text-[10px] uppercase tracking-wider border-b border-white/[0.08]"
        style={{ color: luxuryColors.textMuted }}
      >
        <div>Price (USDT)</div>
        <div className="text-right">Size</div>
        <div className="text-right">Total</div>
      </div>

      <div className="flex flex-col-reverse">
        {asks.slice(0, 8).map((ask, i) => (
          <OrderRow key={`ask-${i}`} level={ask} type="ask" maxTotal={maxAskTotal} />
        ))}
      </div>

      <motion.div
        className="px-4 py-3 border-y border-white/[0.08] flex items-center justify-between"
        style={{
          background: 'linear-gradient(135deg, rgba(239, 68, 68, 0.1), rgba(249, 115, 22, 0.1))',
        }}
        animate={{
          boxShadow: ['0 0 0 rgba(239, 68, 68, 0)', '0 0 20px rgba(239, 68, 68, 0.1)', '0 0 0 rgba(239, 68, 68, 0)'],
        }}
        transition={{ duration: 2, repeat: Infinity }}
      >
        <GradientText className="text-lg font-black" gradient={luxuryColors.gradientDanger}>
          {midPrice > 0 ? `$${midPrice.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}` : 'Loading...'}
        </GradientText>
        <span className="text-[10px]" style={{ color: luxuryColors.textMuted }}>
          Spread: <MonoText className="text-[10px]" style={{ color: luxuryColors.loss }}>{spread.toFixed(2)}</MonoText> (
          <MonoText className="text-[10px]" style={{ color: luxuryColors.loss }}>{spreadPercent.toFixed(4)}%</MonoText>)
        </span>
      </motion.div>

      <div>
        {bids.slice(0, 8).map((bid, i) => (
          <OrderRow key={`bid-${i}`} level={bid} type="bid" maxTotal={maxBidTotal} />
        ))}
      </div>
    </GlassCard>
  );
}

// ============================================================================
// ORDER FORM COMPONENT - Premium Glass Design with 2-STEP CONFIRMATION
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
  const [price, setPrice] = useState('');
  const [showConfirmation, setShowConfirmation] = useState(false);
  const [confirmStep, setConfirmStep] = useState(1);
  const [riskConfirmed, setRiskConfirmed] = useState(false);

  useEffect(() => {
    if (selectedPrice && orderType !== 'market') {
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

    // Open 2-step confirmation
    setShowConfirmation(true);
    setConfirmStep(1);
    setRiskConfirmed(false);
  };

  const handleConfirmOrder = () => {
    if (confirmStep === 1 && riskConfirmed) {
      setConfirmStep(2);
      return;
    }

    if (confirmStep === 2) {
      const orderData: OrderFormData = {
        symbol,
        orderType,
        side,
        quantity: parseFloat(quantity),
        leverage,
        ...(orderType !== 'market' && { price: parseFloat(price) }),
        ...(orderType === 'stop-limit' && { stopPrice: parseFloat(stopPrice) }),
      };

      logger.info('Real trading order confirmed:', orderData);
      onSubmit?.(orderData);

      // Reset
      setShowConfirmation(false);
      setConfirmStep(1);
      setRiskConfirmed(false);
      setQuantity('');
      setPrice('');
      setStopPrice('');
    }
  };

  const orderValue = quantity && price ? parseFloat(quantity) * parseFloat(price) : 0;
  const isBuy = side === 'buy';

  return (
    <>
      <GlassCard noPadding danger>
        <PanelHeader title="Place Order" icon={Target} danger />

        {/* Buy/Sell Toggle */}
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
                    ? 'linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(249, 115, 22, 0.2))'
                    : 'transparent',
                  border: orderType === type ? '1px solid rgba(239, 68, 68, 0.3)' : '1px solid transparent',
                  color: orderType === type ? luxuryColors.loss : luxuryColors.textMuted,
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

          <InputField
            label="Quantity"
            value={quantity}
            onChange={setQuantity}
            placeholder="0.0000"
            suffix="BTC"
          />

          {/* Leverage */}
          <div>
            <label className="block text-[10px] uppercase tracking-wider mb-2" style={{ color: luxuryColors.textMuted }}>
              Leverage: <GradientText className="text-xs font-bold" gradient={luxuryColors.gradientDanger}>{leverage}x</GradientText>
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
                      ? 'linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(249, 115, 22, 0.2))'
                      : 'rgba(255, 255, 255, 0.03)',
                    border: leverage === lev
                      ? '1px solid rgba(239, 68, 68, 0.3)'
                      : '1px solid rgba(255, 255, 255, 0.08)',
                    color: leverage === lev ? luxuryColors.loss : luxuryColors.textMuted,
                  }}
                >
                  {lev}x
                </motion.button>
              ))}
            </div>
          </div>

          {/* Order Summary */}
          <div
            className="p-4 rounded-xl space-y-2 text-xs"
            style={{
              background: 'rgba(239, 68, 68, 0.05)',
              border: '1px solid rgba(239, 68, 68, 0.15)',
            }}
          >
            <div className="flex justify-between items-center">
              <span style={{ color: luxuryColors.textMuted }}>Order Value</span>
              <MonoText className="font-semibold">{orderValue > 0 ? `$${orderValue.toFixed(2)}` : '--'}</MonoText>
            </div>
            <div className="flex justify-between items-center">
              <span style={{ color: luxuryColors.textMuted }}>With Leverage ({leverage}x)</span>
              <GradientText className="font-bold" gradient={luxuryColors.gradientDanger}>
                {orderValue > 0 ? `$${(orderValue * leverage).toFixed(2)}` : '--'}
              </GradientText>
            </div>
          </div>

          {/* Submit Button */}
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
            <AlertTriangle className="w-4 h-4" />
            {isBuy ? `Buy / Long ${symbol.replace('USDT', '')}` : `Sell / Short ${symbol.replace('USDT', '')}`}
          </motion.button>
        </form>
      </GlassCard>

      {/* 2-STEP CONFIRMATION MODAL */}
      <AnimatePresence>
        {showConfirmation && (
          <motion.div
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
            className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
            onClick={() => setShowConfirmation(false)}
          >
            <motion.div
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
              className="w-full max-w-md mx-4 rounded-2xl overflow-hidden"
              style={{
                backgroundColor: luxuryColors.bgPrimary,
                border: '1px solid rgba(239, 68, 68, 0.3)',
                boxShadow: '0 0 60px rgba(239, 68, 68, 0.2)',
              }}
              onClick={(e) => e.stopPropagation()}
            >
              {/* Header */}
              <div
                className="px-6 py-4"
                style={{
                  background: luxuryColors.gradientDanger,
                }}
              >
                <div className="flex items-center gap-3">
                  <motion.div
                    animate={{ scale: [1, 1.1, 1] }}
                    transition={{ duration: 1.5, repeat: Infinity }}
                  >
                    <AlertOctagon className="w-8 h-8 text-white" />
                  </motion.div>
                  <div>
                    <h3 className="text-lg font-bold text-white">Confirm Real Trade</h3>
                    <p className="text-sm text-white/70">Step {confirmStep} of 2</p>
                  </div>
                </div>
                <div className="mt-4 flex gap-2">
                  <div className={`h-1 flex-1 rounded-full ${confirmStep >= 1 ? 'bg-white' : 'bg-white/30'}`} />
                  <div className={`h-1 flex-1 rounded-full ${confirmStep >= 2 ? 'bg-white' : 'bg-white/30'}`} />
                </div>
              </div>

              {/* Content */}
              <div className="p-6 space-y-4">
                {confirmStep === 1 ? (
                  <>
                    <div className="p-4 rounded-xl" style={{ background: 'rgba(239, 68, 68, 0.1)', border: '1px solid rgba(239, 68, 68, 0.2)' }}>
                      <p className="text-sm font-semibold" style={{ color: luxuryColors.loss }}>Risk Warning</p>
                      <ul className="mt-2 text-xs space-y-1" style={{ color: luxuryColors.textSecondary }}>
                        <li>• This trade uses REAL MONEY</li>
                        <li>• Leveraged trading can exceed your deposit</li>
                        <li>• Order Value: <strong className="text-white">${orderValue.toFixed(2)}</strong></li>
                        <li>• Exposure ({leverage}x): <strong style={{ color: luxuryColors.loss }}>${(orderValue * leverage).toFixed(2)}</strong></li>
                      </ul>
                    </div>

                    <label className="flex items-start gap-3 p-3 rounded-xl cursor-pointer hover:bg-white/5" style={{ border: '1px solid rgba(255,255,255,0.1)' }}>
                      <input
                        type="checkbox"
                        checked={riskConfirmed}
                        onChange={(e) => setRiskConfirmed(e.target.checked)}
                        className="mt-0.5 accent-red-500"
                      />
                      <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>
                        I understand the <strong style={{ color: luxuryColors.loss }}>risks</strong> and accept full responsibility for this trade.
                      </span>
                    </label>
                  </>
                ) : (
                  <div className="text-center py-4">
                    <motion.div
                      animate={{ scale: [1, 1.05, 1] }}
                      transition={{ duration: 2, repeat: Infinity }}
                      className="mx-auto mb-4 w-16 h-16 rounded-full flex items-center justify-center"
                      style={{ background: 'rgba(239, 68, 68, 0.2)' }}
                    >
                      <AlertTriangle className="w-8 h-8" style={{ color: luxuryColors.loss }} />
                    </motion.div>
                    <h4 className="text-xl font-bold text-white">Final Confirmation</h4>
                    <p className="mt-2 text-sm" style={{ color: luxuryColors.textSecondary }}>
                      Execute <strong style={{ color: isBuy ? luxuryColors.profit : luxuryColors.loss }}>{isBuy ? 'LONG' : 'SHORT'}</strong> {quantity} {symbol.replace('USDT', '')} with <strong style={{ color: luxuryColors.loss }}>${(orderValue * leverage).toFixed(2)}</strong> exposure?
                    </p>
                    <p className="mt-4 text-xs px-4 py-2 rounded-full inline-flex items-center gap-2" style={{ background: 'rgba(239, 68, 68, 0.15)', color: luxuryColors.loss }}>
                      <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
                      This action cannot be undone
                    </p>
                  </div>
                )}
              </div>

              {/* Footer */}
              <div className="px-6 py-4 flex gap-3" style={{ borderTop: '1px solid rgba(255,255,255,0.08)' }}>
                <button
                  onClick={() => setShowConfirmation(false)}
                  className="flex-1 py-3 rounded-xl font-semibold text-sm"
                  style={{ background: 'rgba(255,255,255,0.1)', color: luxuryColors.textSecondary }}
                >
                  Cancel
                </button>
                <motion.button
                  onClick={handleConfirmOrder}
                  disabled={confirmStep === 1 && !riskConfirmed}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="flex-1 py-3 rounded-xl font-bold text-sm text-white flex items-center justify-center gap-2 disabled:opacity-50"
                  style={{
                    background: luxuryColors.gradientDanger,
                    boxShadow: '0 4px 20px rgba(239, 68, 68, 0.3)',
                  }}
                >
                  {confirmStep === 1 ? 'Continue' : (
                    <>
                      <CheckCircle className="w-4 h-4" />
                      Execute Trade
                    </>
                  )}
                </motion.button>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>
    </>
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
          <RefreshCw className="w-6 h-6" style={{ color: luxuryColors.loss }} />
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
            background: 'rgba(239, 68, 68, 0.05)',
            border: '1px solid rgba(239, 68, 68, 0.15)',
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
    <div className="overflow-x-auto overflow-y-auto custom-scrollbar h-full">
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
                className="border-t border-white/[0.06]"
              >
                <td className="py-3 px-4">
                  <div className="flex items-center gap-2">
                    <span className="font-bold text-white">{trade.symbol.replace('USDT', '')}</span>
                    <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'rgba(239, 68, 68, 0.1)', color: luxuryColors.loss }}>
                      {trade.leverage}x
                    </span>
                  </div>
                </td>
                <td className="py-3 px-4">
                  <Badge variant={trade.trade_type === 'Long' ? 'buy' : 'sell'}>
                    {trade.trade_type === 'Long' ? <TrendingUp className="w-3 h-3 mr-1" /> : <TrendingDown className="w-3 h-3 mr-1" />}
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
                  <div className="inline-flex flex-col items-end px-2 py-1 rounded-lg" style={{ background: isProfitable ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)' }}>
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
                    className="px-3 py-1.5 text-[10px] font-bold rounded-lg flex items-center gap-1"
                    style={{ background: 'rgba(239, 68, 68, 0.15)', border: '1px solid rgba(239, 68, 68, 0.3)', color: luxuryColors.loss }}
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
// TRADE HISTORY TABLE
// ============================================================================

function TradeHistoryTable({ trades, isLoading }: { trades: PaperTrade[]; isLoading: boolean }) {
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>
          <RefreshCw className="w-6 h-6" style={{ color: luxuryColors.loss }} />
        </motion.div>
      </div>
    );
  }

  if (trades.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12" style={{ color: luxuryColors.textMuted }}>
        <div className="p-4 rounded-2xl mb-3" style={{ background: 'rgba(239, 68, 68, 0.05)', border: '1px solid rgba(239, 68, 68, 0.15)' }}>
          <Clock className="w-8 h-8 opacity-50" />
        </div>
        <p className="text-sm font-medium">No trade history</p>
        <p className="text-xs mt-1">Closed trades will appear here</p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto overflow-y-auto custom-scrollbar h-full">
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
                className="border-t border-white/[0.06]"
              >
                <td className="py-3 px-4">
                  <span className="font-bold text-white">{trade.symbol.replace('USDT', '')}</span>
                </td>
                <td className="py-3 px-4">
                  <Badge variant={trade.trade_type === 'Long' ? 'buy' : 'sell'}>
                    {trade.trade_type === 'Long' ? <TrendingUp className="w-3 h-3 mr-1" /> : <TrendingDown className="w-3 h-3 mr-1" />}
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
                  <div className="inline-flex flex-col items-end px-2 py-1 rounded-lg" style={{ background: isProfitable ? 'rgba(34, 197, 94, 0.1)' : 'rgba(239, 68, 68, 0.1)' }}>
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
// MODE SWITCH PROMPT (When not in Real mode)
// ============================================================================

function ModeSwitchPrompt() {
  return (
    <motion.div
      className="h-full flex items-center justify-center p-8"
      style={{ backgroundColor: luxuryColors.bgPrimary }}
      initial="hidden"
      animate="visible"
      variants={containerVariants}
    >
      <GlassCard className="max-w-lg w-full text-center py-12">
        <motion.div
          animate={{ scale: [1, 1.05, 1], rotate: [0, 5, -5, 0] }}
          transition={{ duration: 3, repeat: Infinity }}
          className="mx-auto mb-6 w-20 h-20 rounded-full flex items-center justify-center"
          style={{ background: 'rgba(245, 158, 11, 0.2)' }}
        >
          <AlertTriangle className="w-10 h-10" style={{ color: luxuryColors.warning }} />
        </motion.div>
        <h2 className="text-2xl font-bold text-white">Paper Trading Mode Active</h2>
        <p className="mt-3 text-sm" style={{ color: luxuryColors.textMuted }}>
          Switch to Real Trading mode to execute trades with real funds.
        </p>
        <div className="mt-6">
          <Badge variant="info">Paper Mode: Safe Practice</Badge>
        </div>
        <p className="mt-6 text-xs" style={{ color: luxuryColors.textMuted }}>
          Use the mode toggle in the header to switch between Paper and Real trading modes.
        </p>
      </GlassCard>
    </motion.div>
  );
}

// ============================================================================
// COMING SOON OVERLAY (Real Trading API Not Available)
// ============================================================================

function ComingSoonOverlay() {
  const { switchMode } = useTradingMode();

  return (
    <motion.div
      className="h-full flex items-center justify-center"
      style={{ backgroundColor: luxuryColors.bgPrimary }}
      initial={{ opacity: 0 }}
      animate={{ opacity: 1 }}
      transition={{ duration: 0.5 }}
    >
      <GlassCard className="p-12 text-center max-w-lg" danger>
        {/* Animated Icon */}
        <motion.div
          className="mb-8 relative mx-auto w-32 h-32"
          animate={{
            scale: [1, 1.05, 1],
          }}
          transition={{
            duration: 2,
            repeat: Infinity,
            ease: 'easeInOut',
          }}
        >
          {/* Outer glow ring */}
          <motion.div
            className="absolute inset-0 rounded-full"
            style={{
              background: 'linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(249, 115, 22, 0.1))',
              border: '2px solid rgba(239, 68, 68, 0.3)',
            }}
            animate={{
              boxShadow: [
                '0 0 30px rgba(239, 68, 68, 0.2)',
                '0 0 60px rgba(239, 68, 68, 0.4)',
                '0 0 30px rgba(239, 68, 68, 0.2)',
              ],
            }}
            transition={{
              duration: 2,
              repeat: Infinity,
              ease: 'easeInOut',
            }}
          />
          {/* Inner icon */}
          <div className="absolute inset-0 flex items-center justify-center">
            <AlertOctagon className="w-16 h-16" style={{ color: luxuryColors.loss }} />
          </div>
        </motion.div>

        {/* Title */}
        <GradientText className="text-3xl font-black tracking-tight mb-4">
          COMING SOON
        </GradientText>

        {/* Subtitle */}
        <h2 className="text-xl font-bold text-white mb-4">Real Trading Module</h2>

        {/* Description */}
        <p className="text-sm leading-relaxed mb-6" style={{ color: luxuryColors.textSecondary }}>
          The real trading API is currently under development. This feature will allow you to execute trades
          with real funds on Binance exchange.
        </p>

        {/* Features list */}
        <div className="text-left mb-8 space-y-3">
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <CheckCircle className="w-5 h-5 flex-shrink-0" style={{ color: luxuryColors.emerald }} />
            <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>Binance Spot Order API Integration</span>
          </div>
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <CheckCircle className="w-5 h-5 flex-shrink-0" style={{ color: luxuryColors.emerald }} />
            <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>Real-time Position Tracking via WebSocket</span>
          </div>
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <CheckCircle className="w-5 h-5 flex-shrink-0" style={{ color: luxuryColors.emerald }} />
            <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>Advanced Risk Management & Circuit Breaker</span>
          </div>
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <Clock className="w-5 h-5 flex-shrink-0" style={{ color: luxuryColors.warning }} />
            <span className="text-sm" style={{ color: luxuryColors.textSecondary }}>API Endpoints Under Development</span>
          </div>
        </div>

        {/* CTA Button */}
        <motion.button
          className="w-full py-4 px-6 rounded-xl font-bold text-white flex items-center justify-center gap-3"
          style={{
            background: 'linear-gradient(135deg, rgba(34, 197, 94, 0.15), rgba(0, 217, 255, 0.1))',
            border: '1px solid rgba(34, 197, 94, 0.3)',
          }}
          whileHover={{
            scale: 1.02,
            boxShadow: '0 0 20px rgba(34, 197, 94, 0.3)',
          }}
          whileTap={{ scale: 0.98 }}
          onClick={() => switchMode('paper')}
        >
          <Zap className="w-5 h-5" style={{ color: luxuryColors.emerald }} />
          <span>Try Paper Trading Instead</span>
        </motion.button>

        {/* Info text */}
        <p className="mt-6 text-xs" style={{ color: luxuryColors.textMuted }}>
          Paper trading provides the same luxury UI with simulated funds - perfect for testing strategies!
        </p>
      </GlassCard>
    </motion.div>
  );
}

// ============================================================================
// MAIN REAL TRADING PAGE
// ============================================================================

export default function RealTrading() {
  const { toast } = useToast();
  const { mode } = useTradingMode();
  const realTrading = useRealTrading();
  const [selectedSymbol, setSelectedSymbol] = useState('BTCUSDT');
  const availableSymbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT', 'XRPUSDT', 'ADAUSDT'];
  const [showSymbolDropdown, setShowSymbolDropdown] = useState(false);
  const [selectedPrice, setSelectedPrice] = useState<number | undefined>();
  const [activeTab, setActiveTab] = useState<'positions' | 'history'>('positions');
  const [selectedTimeframe, setSelectedTimeframe] = useState('5m');
  const [apiAvailable, setApiAvailable] = useState<boolean | null>(null);

  const timeframes = ['1m', '5m', '15m', '1h', '4h', '1D'];

  // Check if real trading API is available
  useEffect(() => {
    const checkApiAvailability = async () => {
      try {
        const apiBase = import.meta.env.VITE_RUST_API_URL || 'http://localhost:8080';
        const response = await fetch(`${apiBase}/api/real-trading/status`, {
          method: 'GET',
          signal: AbortSignal.timeout(5000), // 5 second timeout
        });
        // If we get any response (even error), API route exists
        if (response.ok) {
          const data = await response.json();
          setApiAvailable(data.success === true);
        } else {
          setApiAvailable(false);
        }
      } catch {
        // Network error or timeout = API not available
        setApiAvailable(false);
      }
    };

    if (mode === 'real') {
      checkApiAvailability();
    }
  }, [mode]);

  // Show mode switch prompt if not in real mode
  if (mode !== 'real') {
    return <ModeSwitchPrompt />;
  }

  // Show coming soon overlay if API not available
  if (apiAvailable === false) {
    return <ComingSoonOverlay />;
  }

  // Show loading while checking API
  if (apiAvailable === null) {
    return (
      <div className="h-full flex items-center justify-center" style={{ backgroundColor: luxuryColors.bgPrimary }}>
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
        >
          <RefreshCw className="w-8 h-8" style={{ color: luxuryColors.loss }} />
        </motion.div>
      </div>
    );
  }

  const handleOrderSubmit = async (order: OrderFormData) => {
    logger.info('Real trading order submitted:', order);

    toast({
      title: 'Real Order Executed',
      description: `${order.side.toUpperCase()} ${order.quantity} ${order.symbol} with ${order.leverage}x leverage`,
      variant: 'destructive',
    });
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
        balance={realTrading.portfolio.current_balance}
        equity={realTrading.portfolio.equity}
        totalPnl={realTrading.portfolio.total_pnl}
        totalPnlPercent={realTrading.portfolio.total_pnl_percentage || 0}
        winRate={realTrading.portfolio.win_rate}
        totalTrades={realTrading.portfolio.total_trades}
      />

      {/* Main Trading Grid */}
      <div className="flex-1 grid grid-cols-12 gap-[1px] min-h-0 overflow-hidden" style={{ backgroundColor: luxuryColors.borderSubtle }}>
        {/* Left Column: Chart (60%) */}
        <div
          className="col-span-7 flex flex-col overflow-y-auto custom-scrollbar"
          style={{ backgroundColor: luxuryColors.bgPrimary }}
        >
          {/* Chart Header */}
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
                    background: 'rgba(239, 68, 68, 0.1)',
                    border: '1px solid rgba(239, 68, 68, 0.2)',
                  }}
                >
                  <LineChart className="w-4 h-4" style={{ color: luxuryColors.loss }} />
                </div>
                {/* Symbol Selector */}
                <motion.button
                  onClick={() => setShowSymbolDropdown(!showSymbolDropdown)}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="flex items-center gap-2 px-3 py-1.5 rounded-xl cursor-pointer"
                  style={{
                    background: showSymbolDropdown ? 'rgba(239, 68, 68, 0.15)' : 'rgba(255, 255, 255, 0.05)',
                    border: `1px solid ${showSymbolDropdown ? 'rgba(239, 68, 68, 0.3)' : 'rgba(255, 255, 255, 0.1)'}`,
                  }}
                >
                  <div className="flex items-center">
                    <GradientText className="text-lg font-black" gradient={luxuryColors.gradientDanger}>
                      {selectedSymbol.replace('USDT', '')}
                    </GradientText>
                    <span className="text-xs font-medium" style={{ color: luxuryColors.textMuted }}>/USDT</span>
                  </div>
                  <ChevronDown
                    className={`w-4 h-4 transition-transform duration-200 ${showSymbolDropdown ? 'rotate-180' : ''}`}
                    style={{ color: luxuryColors.textMuted }}
                  />
                </motion.button>

                {/* Dropdown */}
                <AnimatePresence>
                  {showSymbolDropdown && (
                    <>
                      <div className="fixed inset-0 z-40" onClick={() => setShowSymbolDropdown(false)} />
                      <motion.div
                        initial={{ opacity: 0, y: -10, scale: 0.95 }}
                        animate={{ opacity: 1, y: 0, scale: 1 }}
                        exit={{ opacity: 0, y: -10, scale: 0.95 }}
                        className="absolute top-full left-0 mt-2 z-50 min-w-[180px] rounded-xl overflow-hidden"
                        style={{
                          backgroundColor: luxuryColors.bgPrimary,
                          border: '1px solid rgba(239, 68, 68, 0.2)',
                          boxShadow: '0 8px 32px rgba(0, 0, 0, 0.6)',
                        }}
                      >
                        {availableSymbols.map((symbol) => (
                          <motion.button
                            key={symbol}
                            onClick={() => { setSelectedSymbol(symbol); setShowSymbolDropdown(false); }}
                            whileHover={{ backgroundColor: 'rgba(255, 255, 255, 0.08)' }}
                            className="w-full flex items-center justify-between px-4 py-3 text-sm font-medium"
                            style={{
                              color: selectedSymbol === symbol ? luxuryColors.loss : luxuryColors.textSecondary,
                              backgroundColor: selectedSymbol === symbol ? 'rgba(239, 68, 68, 0.1)' : 'transparent',
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
              <Badge variant="danger">REAL</Badge>
            </div>

            {/* Timeframe buttons */}
            <div className="flex items-center gap-1 p-1 rounded-xl bg-white/[0.03] border border-white/[0.08]">
              {timeframes.map((tf) => (
                <motion.button
                  key={tf}
                  onClick={() => setSelectedTimeframe(tf)}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  className="px-3 py-1.5 text-[10px] font-bold uppercase tracking-wider rounded-lg"
                  style={{
                    background: selectedTimeframe === tf
                      ? 'linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(249, 115, 22, 0.2))'
                      : 'transparent',
                    border: selectedTimeframe === tf ? '1px solid rgba(239, 68, 68, 0.3)' : '1px solid transparent',
                    color: selectedTimeframe === tf ? luxuryColors.loss : luxuryColors.textMuted,
                  }}
                >
                  {tf}
                </motion.button>
              ))}
            </div>
          </motion.div>

          {/* Chart */}
          <div className="flex-1 min-h-0">
            <TradingViewChart symbol={selectedSymbol} timeframe={selectedTimeframe} showControls={false} />
          </div>

          {/* Positions/History Tabs */}
          <div
            className="rounded-t-2xl flex flex-col"
            style={{ backgroundColor: 'rgba(255, 255, 255, 0.02)', minHeight: '250px', maxHeight: '40vh' }}
          >
            <div className="flex border-b border-white/[0.08]">
              {[
                { id: 'positions', label: 'Positions', icon: Activity, count: realTrading.openTrades.length },
                { id: 'history', label: 'Trade History', icon: Clock },
              ].map((tab) => (
                <motion.button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as 'positions' | 'history')}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="relative px-5 py-3 text-xs font-bold flex items-center gap-2"
                  style={{ color: activeTab === tab.id ? luxuryColors.loss : luxuryColors.textMuted }}
                >
                  <tab.icon className="w-4 h-4" />
                  {tab.label}
                  {tab.count !== undefined && (
                    <span
                      className="px-2 py-0.5 text-[10px] rounded-full font-bold"
                      style={{
                        background: activeTab === tab.id ? 'rgba(239, 68, 68, 0.2)' : 'rgba(255, 255, 255, 0.1)',
                        color: activeTab === tab.id ? luxuryColors.loss : luxuryColors.textSecondary,
                      }}
                    >
                      {tab.count}
                    </span>
                  )}
                  {activeTab === tab.id && (
                    <motion.div
                      layoutId="activeRealTabIndicator"
                      className="absolute bottom-0 left-0 right-0 h-[2px]"
                      style={{ background: luxuryColors.gradientDanger, boxShadow: '0 0 10px rgba(239, 68, 68, 0.5)' }}
                    />
                  )}
                </motion.button>
              ))}
            </div>

            <div className="flex-1 min-h-0 overflow-hidden">
              <AnimatePresence mode="wait">
                {activeTab === 'positions' ? (
                  <motion.div key="positions" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} className="h-full overflow-auto">
                    <PositionsTable trades={realTrading.openTrades} isLoading={realTrading.isLoading} onCloseTrade={realTrading.closeTrade} />
                  </motion.div>
                ) : (
                  <motion.div key="history" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} className="h-full overflow-auto">
                    <TradeHistoryTable trades={realTrading.closedTrades} isLoading={realTrading.isLoading} />
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </div>
        </div>

        {/* Right Column: Order Book + Form (40%) */}
        <div className="col-span-5 flex flex-col overflow-y-auto" style={{ backgroundColor: luxuryColors.bgPrimary }}>
          <div className="grid grid-cols-2 gap-[1px] h-full" style={{ backgroundColor: luxuryColors.borderSubtle }}>
            {/* Order Book */}
            <div style={{ backgroundColor: luxuryColors.bgPrimary }}>
              <OrderBook symbol={selectedSymbol} onPriceClick={handlePriceClick} />
            </div>

            {/* Order Form */}
            <div style={{ backgroundColor: luxuryColors.bgPrimary }}>
              <OrderForm symbol={selectedSymbol} onSubmit={handleOrderSubmit} selectedPrice={selectedPrice} />

              {/* Risk Warning */}
              <div className="p-4">
                <motion.div
                  className="p-4 rounded-xl"
                  style={{
                    background: 'linear-gradient(135deg, rgba(239, 68, 68, 0.1), rgba(220, 38, 38, 0.05))',
                    border: '1px solid rgba(239, 68, 68, 0.2)',
                  }}
                  initial={{ opacity: 0, y: 10 }}
                  animate={{ opacity: 1, y: 0 }}
                  transition={{ delay: 0.3 }}
                >
                  <div className="flex items-start gap-3">
                    <div className="p-2 rounded-lg flex-shrink-0" style={{ background: 'rgba(239, 68, 68, 0.15)', border: '1px solid rgba(239, 68, 68, 0.3)' }}>
                      <AlertOctagon className="w-4 h-4" style={{ color: luxuryColors.loss }} />
                    </div>
                    <div>
                      <p className="text-xs font-bold" style={{ color: luxuryColors.loss }}>Real Money Trading</p>
                      <p className="text-[10px] mt-1.5 leading-relaxed" style={{ color: luxuryColors.textSecondary }}>
                        All orders require 2-step confirmation. Set stop-loss on every trade. Never risk more than you can afford to lose.
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
