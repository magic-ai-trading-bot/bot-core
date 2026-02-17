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

import { useState, useEffect } from 'react';
import { useTranslation } from 'react-i18next';
import { motion, AnimatePresence } from 'framer-motion';
import { useRealTrading, type RealOrder } from '@/hooks/useRealTrading';
import { useTradingMode } from '@/hooks/useTradingMode';
import { TradingViewChart } from '@/components/trading/TradingViewChart';
import { type OrderFormData } from '@/components/trading/OrderForm';
import { useToast } from '@/hooks/use-toast';
import { formatDistanceToNow } from 'date-fns';
import { fetchBinancePrice } from '@/utils/binancePrice';
import logger from '@/utils/logger';
import { useThemeColors } from '@/hooks/useThemeColors';
import {
  TrendingUp,
  TrendingDown,
  Activity,
  Wallet,
  BarChart3,
  Clock,
  Target,
  Zap,
  X,
  RefreshCw,
  ChevronDown,
  AlertTriangle,
  LineChart,
  AlertOctagon,
  CheckCircle,
} from 'lucide-react';
import type { PaperTrade } from '@/hooks/usePaperTrading';

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
  // Check if className contains flex layout classes
  const hasFlexLayout = className.includes('flex');

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
      {hasFlexLayout ? (
        // When using flex layout, render children directly without wrapper
        children
      ) : (
        <div className={noPadding ? '' : 'p-4'}>{children}</div>
      )}
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
  const colors = useThemeColors();
  const accentColor = danger ? colors.primary : colors.cyan;
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
  const colors = useThemeColors();
  const variants = {
    default: { bg: 'rgba(255, 255, 255, 0.1)', color: colors.textSecondary, border: 'rgba(255, 255, 255, 0.15)' },
    buy: { bg: 'rgba(34, 197, 94, 0.15)', color: colors.profit, border: 'rgba(34, 197, 94, 0.3)' },
    sell: { bg: 'rgba(239, 68, 68, 0.15)', color: colors.loss, border: 'rgba(239, 68, 68, 0.3)' },
    info: { bg: 'rgba(0, 217, 255, 0.15)', color: colors.cyan, border: 'rgba(0, 217, 255, 0.3)' },
    warning: { bg: 'rgba(245, 158, 11, 0.15)', color: colors.warning, border: 'rgba(245, 158, 11, 0.3)' },
    danger: { bg: 'rgba(239, 68, 68, 0.15)', color: colors.loss, border: 'rgba(239, 68, 68, 0.3)' },
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
  gradient,
}: {
  children: React.ReactNode;
  className?: string;
  gradient?: string;
}) {
  const colors = useThemeColors();
  return (
    <span
      className={`bg-clip-text text-transparent ${className}`}
      style={{ backgroundImage: gradient || colors.gradientPremium }}
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
  const colors = useThemeColors();
  let color = colors.textPrimary;
  if (positive) color = colors.profit;
  if (negative) color = colors.loss;

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
  const colors = useThemeColors();
  return (
    <div>
      <label className="block text-[10px] uppercase tracking-wider mb-1.5" style={{ color: colors.textMuted }}>
        {label}
      </label>
      <div
        className="relative flex items-center rounded-xl border transition-all duration-300 focus-within:border-red-500/50 focus-within:shadow-[0_0_20px_rgba(239,68,68,0.15)]"
        style={{
          backgroundColor: 'rgba(255, 255, 255, 0.03)',
          borderColor: colors.borderSubtle,
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
          <span className="px-3 text-xs font-medium" style={{ color: colors.textMuted }}>
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
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  const isProfitable = totalPnl >= 0;

  return (
    <motion.div
      initial={{ opacity: 0, y: -10 }}
      animate={{ opacity: 1, y: 0 }}
      className="relative overflow-hidden"
      style={{ backgroundColor: colors.bgPrimary }}
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
            style={{ backgroundColor: colors.loss }}
            animate={{ opacity: [1, 0.5, 1] }}
            transition={{ duration: 1, repeat: Infinity }}
          />
          <AlertTriangle className="w-4 h-4" style={{ color: colors.loss }} />
          <span className="text-xs font-bold uppercase tracking-wider" style={{ color: colors.loss }}>
            {t('realTrading.warning.title')}
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
            <Wallet className="w-5 h-5" style={{ color: colors.loss }} />
          </div>
          <div>
            <p className="text-[10px] uppercase tracking-wider" style={{ color: colors.textMuted }}>{t('realTrading.orderBook.total')}</p>
            <GradientText className="text-xl font-black" gradient={colors.gradientDanger}>
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
            <Activity className="w-4 h-4" style={{ color: colors.textSecondary }} />
          </div>
          <div>
            <p className="text-[10px] uppercase tracking-wider" style={{ color: colors.textMuted }}>{t('stats.totalProfit')}</p>
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
            <TrendingUp className="w-5 h-5" style={{ color: colors.profit }} />
          ) : (
            <TrendingDown className="w-5 h-5" style={{ color: colors.loss }} />
          )}
          <div>
            <p className="text-[10px] uppercase tracking-wider" style={{ color: colors.textMuted }}>{t('table.pnl')}</p>
            <MonoText className="text-lg font-bold" positive={isProfitable} negative={!isProfitable}>
              {isProfitable ? '+' : ''}${Math.abs(totalPnl).toFixed(2)} ({isProfitable ? '+' : ''}{totalPnlPercent.toFixed(2)}%)
            </MonoText>
          </div>
        </motion.div>

        {/* Stats */}
        <div className="flex items-center gap-4 ml-auto">
          <div className="text-center px-3">
            <p className="text-[10px] uppercase tracking-wider" style={{ color: colors.textMuted }}>{t('stats.winRate')}</p>
            <MonoText className="text-base font-bold" style={{ color: colors.loss }}>{winRate.toFixed(1)}%</MonoText>
          </div>
          <div className="text-center px-3">
            <p className="text-[10px] uppercase tracking-wider" style={{ color: colors.textMuted }}>{t('stats.totalTrades')}</p>
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
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
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

          for (let i = 0; i < 12; i++) {
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
    const priceColor = isAsk ? colors.loss : colors.profit;
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
        <div className="relative z-10 text-right font-mono" style={{ color: colors.textPrimary }}>
          {level.quantity.toFixed(4)}
        </div>
        <div className="relative z-10 text-right font-mono" style={{ color: colors.textSecondary }}>
          {level.total.toFixed(4)}
        </div>
      </motion.div>
    );
  };

  return (
    <GlassCard noPadding danger className="h-full flex flex-col">
      <PanelHeader title={t('realTrading.orderBook.title')} icon={BarChart3} danger />

      <div
        className="grid grid-cols-3 gap-2 px-4 py-2 text-[10px] uppercase tracking-wider border-b border-white/[0.08]"
        style={{ color: colors.textMuted }}
      >
        <div>{t('realTrading.orderBook.price')} (USDT)</div>
        <div className="text-right">{t('realTrading.orderBook.amount')}</div>
        <div className="text-right">{t('realTrading.orderBook.total')}</div>
      </div>

      {/* Asks (reversed) - flex-1 to fill available space */}
      <div className="flex flex-col-reverse flex-1 justify-end">
        {asks.slice(0, 12).map((ask, i) => (
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
        <GradientText className="text-lg font-black" gradient={colors.gradientDanger}>
          {midPrice > 0 ? `$${midPrice.toLocaleString('en-US', { minimumFractionDigits: 2, maximumFractionDigits: 2 })}` : 'Loading...'}
        </GradientText>
        <span className="text-[10px]" style={{ color: colors.textMuted }}>
          Spread: <MonoText className="text-[10px]" style={{ color: colors.loss }}>{spread.toFixed(2)}</MonoText> (
          <MonoText className="text-[10px]" style={{ color: colors.loss }}>{spreadPercent.toFixed(4)}%</MonoText>)
        </span>
      </motion.div>

      {/* Bids - flex-1 to fill available space */}
      <div className="flex-1">
        {bids.slice(0, 12).map((bid, i) => (
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
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
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
      <GlassCard noPadding danger className="h-full flex flex-col">
        <PanelHeader title={t('realTrading.orderForm.submit')} icon={Target} danger />

        {/* Buy/Sell Toggle */}
        <div className="p-4 border-b border-white/[0.08]">
          <div className="relative flex rounded-xl overflow-hidden bg-white/[0.03]">
            <motion.div
              className="absolute top-0 bottom-0 w-1/2 rounded-xl"
              style={{
                background: isBuy ? colors.gradientProfit : colors.gradientLoss,
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
              style={{ color: isBuy ? '#fff' : colors.textSecondary }}
            >
              <TrendingUp className="w-4 h-4" />
              {t('realTrading.orderForm.buy')} / Long
            </button>
            <button
              type="button"
              onClick={() => setSide('sell')}
              className="relative z-10 flex-1 py-3 text-xs font-bold transition-colors flex items-center justify-center gap-2"
              style={{ color: !isBuy ? '#fff' : colors.textSecondary }}
            >
              <TrendingDown className="w-4 h-4" />
              {t('realTrading.orderForm.sell')} / Short
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
                  color: orderType === type ? colors.loss : colors.textMuted,
                }}
              >
                {type === 'market' ? t('realTrading.orderForm.market') : type === 'limit' ? t('realTrading.orderForm.limit') : t('realTrading.orderForm.stopLimit')}
              </button>
            ))}
          </div>

          {/* Price Inputs */}
          {orderType !== 'market' && (
            <InputField
              label={t('realTrading.orderForm.price')}
              value={price}
              onChange={setPrice}
              placeholder="0.00"
              suffix="USDT"
            />
          )}

          {orderType === 'stop-limit' && (
            <InputField
              label={t('realTrading.orderForm.stopPrice')}
              value={stopPrice}
              onChange={setStopPrice}
              placeholder="0.00"
              suffix="USDT"
            />
          )}

          <InputField
            label={t('realTrading.orderForm.amount')}
            value={quantity}
            onChange={setQuantity}
            placeholder="0.0000"
            suffix="BTC"
          />

          {/* Leverage */}
          <div>
            <label className="block text-[10px] uppercase tracking-wider mb-2" style={{ color: colors.textMuted }}>
              Leverage: <GradientText className="text-xs font-bold" gradient={colors.gradientDanger}>{leverage}x</GradientText>
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
                    color: leverage === lev ? colors.loss : colors.textMuted,
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
              <span style={{ color: colors.textMuted }}>{t('realTrading.orderForm.total')}</span>
              <MonoText className="font-semibold">{orderValue > 0 ? `$${orderValue.toFixed(2)}` : '--'}</MonoText>
            </div>
            <div className="flex justify-between items-center">
              <span style={{ color: colors.textMuted }}>{t('realTrading.orderForm.available')} ({leverage}x)</span>
              <GradientText className="font-bold" gradient={colors.gradientDanger}>
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
              background: isBuy ? colors.gradientProfit : colors.gradientLoss,
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
                backgroundColor: colors.bgPrimary,
                border: '1px solid rgba(239, 68, 68, 0.3)',
                boxShadow: '0 0 60px rgba(239, 68, 68, 0.2)',
              }}
              onClick={(e) => e.stopPropagation()}
            >
              {/* Header */}
              <div
                className="px-6 py-4"
                style={{
                  background: colors.gradientDanger,
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
                    <h3 className="text-lg font-bold text-white">{t('realTrading.confirm.title')}</h3>
                    <p className="text-sm text-white/70">{t('realTrading.confirm.step', { current: confirmStep, total: 2 })}</p>
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
                      <p className="text-sm font-semibold" style={{ color: colors.loss }}>{t('realTrading.confirm.riskWarning')}</p>
                      <ul className="mt-2 text-xs space-y-1" style={{ color: colors.textSecondary }}>
                        <li>• {t('realTrading.confirm.realMoney')}</li>
                        <li>• {t('realTrading.confirm.leverageWarning')}</li>
                        <li>• {t('realTrading.confirm.orderValue')}: <strong className="text-white">${orderValue.toFixed(2)}</strong></li>
                        <li>• {t('realTrading.confirm.exposure', { leverage })}: <strong style={{ color: colors.loss }}>${(orderValue * leverage).toFixed(2)}</strong></li>
                      </ul>
                    </div>

                    <label className="flex items-start gap-3 p-3 rounded-xl cursor-pointer hover:bg-white/5" style={{ border: '1px solid rgba(255,255,255,0.1)' }}>
                      <input
                        type="checkbox"
                        checked={riskConfirmed}
                        onChange={(e) => setRiskConfirmed(e.target.checked)}
                        className="mt-0.5 accent-red-500"
                      />
                      <span className="text-sm" style={{ color: colors.textSecondary }}>
                        {t('realTrading.confirm.acceptRisk')}
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
                      <AlertTriangle className="w-8 h-8" style={{ color: colors.loss }} />
                    </motion.div>
                    <h4 className="text-xl font-bold text-white">{t('realTrading.confirm.finalTitle')}</h4>
                    <p className="mt-2 text-sm" style={{ color: colors.textSecondary }}>
                      {t('realTrading.confirm.executeQuestion', {
                        direction: isBuy ? 'LONG' : 'SHORT',
                        quantity,
                        symbol: symbol.replace('USDT', ''),
                        exposure: (orderValue * leverage).toFixed(2)
                      })}
                    </p>
                    <p className="mt-4 text-xs px-4 py-2 rounded-full inline-flex items-center gap-2" style={{ background: 'rgba(239, 68, 68, 0.15)', color: colors.loss }}>
                      <span className="w-2 h-2 rounded-full bg-red-500 animate-pulse" />
                      {t('realTrading.confirm.cannotUndo')}
                    </p>
                  </div>
                )}
              </div>

              {/* Footer */}
              <div className="px-6 py-4 flex gap-3" style={{ borderTop: '1px solid rgba(255,255,255,0.08)' }}>
                <button
                  onClick={() => setShowConfirmation(false)}
                  className="flex-1 py-3 rounded-xl font-semibold text-sm"
                  style={{ background: 'rgba(255,255,255,0.1)', color: colors.textSecondary }}
                >
                  {t('realTrading.confirm.cancel')}
                </button>
                <motion.button
                  onClick={handleConfirmOrder}
                  disabled={confirmStep === 1 && !riskConfirmed}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="flex-1 py-3 rounded-xl font-bold text-sm text-white flex items-center justify-center gap-2 disabled:opacity-50"
                  style={{
                    background: colors.gradientDanger,
                    boxShadow: '0 4px 20px rgba(239, 68, 68, 0.3)',
                  }}
                >
                  {confirmStep === 1 ? t('realTrading.confirm.continue') : (
                    <>
                      <CheckCircle className="w-4 h-4" />
                      {t('realTrading.confirm.executeTrade')}
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
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
        >
          <RefreshCw className="w-6 h-6" style={{ color: colors.loss }} />
        </motion.div>
      </div>
    );
  }

  if (trades.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12" style={{ color: colors.textMuted }}>
        <div
          className="p-4 rounded-2xl mb-3"
          style={{
            background: 'rgba(239, 68, 68, 0.05)',
            border: '1px solid rgba(239, 68, 68, 0.15)',
          }}
        >
          <Activity className="w-8 h-8 opacity-50" />
        </div>
        <p className="text-sm font-medium">{t('realTrading.positions.noPositions')}</p>
        <p className="text-xs mt-1" style={{ color: colors.textMuted }}>
          {t('realTrading.orderForm.submit')}
        </p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto overflow-y-auto custom-scrollbar h-full">
      <table className="w-full text-xs">
        <thead className="sticky top-0 z-10" style={{ backgroundColor: colors.bgPrimary }}>
          <tr style={{ color: colors.textMuted }}>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.symbol')}</th>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.side')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.entry')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.size')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.pnl')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('table.action')}</th>
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
                    <span className="text-[10px] px-1.5 py-0.5 rounded" style={{ background: 'rgba(239, 68, 68, 0.1)', color: colors.loss }}>
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
                    style={{ background: 'rgba(239, 68, 68, 0.15)', border: '1px solid rgba(239, 68, 68, 0.3)', color: colors.loss }}
                  >
                    <X className="w-3 h-3" />
                    {t('realTrading.positions.close')}
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
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>
          <RefreshCw className="w-6 h-6" style={{ color: colors.loss }} />
        </motion.div>
      </div>
    );
  }

  if (trades.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12" style={{ color: colors.textMuted }}>
        <div className="p-4 rounded-2xl mb-3" style={{ background: 'rgba(239, 68, 68, 0.05)', border: '1px solid rgba(239, 68, 68, 0.15)' }}>
          <Clock className="w-8 h-8 opacity-50" />
        </div>
        <p className="text-sm font-medium">{t('realTrading.history.title')}</p>
        <p className="text-xs mt-1">{t('realTrading.orders.noOrders')}</p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto overflow-y-auto custom-scrollbar h-full">
      <table className="w-full text-xs">
        <thead className="sticky top-0 z-10" style={{ backgroundColor: colors.bgPrimary }}>
          <tr style={{ color: colors.textMuted }}>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.symbol')}</th>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.side')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.entry')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('table.exit')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.positions.pnl')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.history.time')}</th>
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
                <td className="py-3 px-4 text-right" style={{ color: colors.textSecondary }}>
                  <div className="flex items-center justify-end gap-1.5">
                    <Clock className="w-3 h-3" style={{ color: colors.textMuted }} />
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
// ACTIVE ORDERS TABLE (Phase 5)
// ============================================================================

function OrdersTable({
  orders,
  isLoading,
  onCancelOrder,
}: {
  orders: RealOrder[];
  isLoading: boolean;
  onCancelOrder?: (id: string) => void;
}) {
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();

  if (isLoading) {
    return (
      <div className="flex items-center justify-center py-12">
        <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>
          <RefreshCw className="w-6 h-6" style={{ color: colors.loss }} />
        </motion.div>
      </div>
    );
  }

  if (orders.length === 0) {
    return (
      <div className="flex flex-col items-center justify-center py-12" style={{ color: colors.textMuted }}>
        <div className="p-4 rounded-2xl mb-3" style={{ background: 'rgba(239, 68, 68, 0.05)', border: '1px solid rgba(239, 68, 68, 0.15)' }}>
          <Target className="w-8 h-8 opacity-50" />
        </div>
        <p className="text-sm font-medium">{t('realTrading.orders.noOrders')}</p>
        <p className="text-xs mt-1" style={{ color: colors.textMuted }}>
          {t('realTrading.orders.placeOrder')}
        </p>
      </div>
    );
  }

  return (
    <div className="overflow-x-auto overflow-y-auto custom-scrollbar h-full">
      <table className="w-full text-xs">
        <thead className="sticky top-0 z-10" style={{ backgroundColor: colors.bgPrimary }}>
          <tr style={{ color: colors.textMuted }}>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.orders.symbol')}</th>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.orders.type')}</th>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.orders.side')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.orders.price')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.orders.quantity')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.orders.filled')}</th>
            <th className="text-left py-3 px-4 font-bold uppercase tracking-wider">{t('realTrading.orders.status')}</th>
            <th className="text-right py-3 px-4 font-bold uppercase tracking-wider">{t('table.action')}</th>
          </tr>
        </thead>
        <tbody>
          {orders.map((order, index) => {
            const isBuy = order.side.toUpperCase() === 'BUY';
            const fillPercent = order.quantity > 0 ? (order.executed_quantity / order.quantity) * 100 : 0;
            return (
              <motion.tr
                key={order.id}
                initial={{ opacity: 0, x: -20 }}
                animate={{ opacity: 1, x: 0 }}
                transition={{ delay: index * 0.05 }}
                whileHover={{ backgroundColor: 'rgba(255, 255, 255, 0.05)' }}
                className="border-t border-white/[0.06]"
              >
                <td className="py-3 px-4">
                  <span className="font-bold text-white">{order.symbol.replace('USDT', '')}</span>
                  <span className="text-[10px] ml-1" style={{ color: colors.textMuted }}>USDT</span>
                </td>
                <td className="py-3 px-4">
                  <span className="text-[10px] px-2 py-1 rounded" style={{ background: 'rgba(255, 255, 255, 0.1)' }}>
                    {order.order_type}
                  </span>
                </td>
                <td className="py-3 px-4">
                  <Badge variant={isBuy ? 'buy' : 'sell'}>
                    {isBuy ? <TrendingUp className="w-3 h-3 mr-1" /> : <TrendingDown className="w-3 h-3 mr-1" />}
                    {order.side}
                  </Badge>
                </td>
                <td className="py-3 px-4 text-right">
                  <MonoText className="font-semibold">
                    {order.price ? `$${order.price.toFixed(2)}` : 'Market'}
                  </MonoText>
                </td>
                <td className="py-3 px-4 text-right">
                  <MonoText>{order.quantity.toFixed(4)}</MonoText>
                </td>
                <td className="py-3 px-4 text-right">
                  <div className="flex flex-col items-end">
                    <MonoText>{order.executed_quantity.toFixed(4)}</MonoText>
                    <span className="text-[10px]" style={{ color: colors.textMuted }}>
                      ({fillPercent.toFixed(0)}%)
                    </span>
                  </div>
                </td>
                <td className="py-3 px-4">
                  <span
                    className="text-[10px] px-2 py-1 rounded font-medium"
                    style={{
                      background: order.status === 'FILLED' ? 'rgba(34, 197, 94, 0.1)' : 'rgba(245, 158, 11, 0.1)',
                      color: order.status === 'FILLED' ? colors.profit : colors.warning,
                    }}
                  >
                    {order.status}
                  </span>
                </td>
                <td className="py-3 px-4 text-right">
                  {order.status !== 'FILLED' && order.status !== 'CANCELED' && (
                    <motion.button
                      onClick={() => onCancelOrder?.(order.id)}
                      whileHover={{ scale: 1.05 }}
                      whileTap={{ scale: 0.95 }}
                      className="px-3 py-1.5 text-[10px] font-bold rounded-lg flex items-center gap-1"
                      style={{ background: 'rgba(239, 68, 68, 0.15)', border: '1px solid rgba(239, 68, 68, 0.3)', color: colors.loss }}
                    >
                      <X className="w-3 h-3" />
                      Cancel
                    </motion.button>
                  )}
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
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  return (
    <motion.div
      className="h-full flex items-center justify-center p-8"
      style={{ backgroundColor: colors.bgPrimary }}
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
          <AlertTriangle className="w-10 h-10" style={{ color: colors.warning }} />
        </motion.div>
        <h2 className="text-2xl font-bold text-black dark:text-white">{t('realTrading.modeSwitch.title')}</h2>
        <p className="mt-3 text-sm" style={{ color: colors.textMuted }}>
          {t('realTrading.modeSwitch.description')}
        </p>
        <div className="mt-6">
          <Badge variant="info">{t('realTrading.modeSwitch.badge')}</Badge>
        </div>
        <p className="mt-6 text-xs" style={{ color: colors.textMuted }}>
          {t('realTrading.modeSwitch.hint')}
        </p>
      </GlassCard>
    </motion.div>
  );
}

// ============================================================================
// COMING SOON OVERLAY (Real Trading API Not Available)
// ============================================================================

function ComingSoonOverlay() {
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  const { switchMode } = useTradingMode();

  return (
    <motion.div
      className="h-full flex items-center justify-center"
      style={{ backgroundColor: colors.bgPrimary }}
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
            <AlertOctagon className="w-16 h-16" style={{ color: colors.loss }} />
          </div>
        </motion.div>

        {/* Title */}
        <GradientText className="text-3xl font-black tracking-tight mb-4">
          {t('realTrading.comingSoon.title')}
        </GradientText>

        {/* Subtitle */}
        <h2 className="text-xl font-bold text-white mb-4">{t('realTrading.comingSoon.subtitle')}</h2>

        {/* Description */}
        <p className="text-sm leading-relaxed mb-6" style={{ color: colors.textSecondary }}>
          {t('realTrading.comingSoon.description')}
        </p>

        {/* Features list */}
        <div className="text-left mb-8 space-y-3">
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <CheckCircle className="w-5 h-5 flex-shrink-0" style={{ color: colors.emerald }} />
            <span className="text-sm" style={{ color: colors.textSecondary }}>{t('realTrading.comingSoon.features.spotApi')}</span>
          </div>
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <CheckCircle className="w-5 h-5 flex-shrink-0" style={{ color: colors.emerald }} />
            <span className="text-sm" style={{ color: colors.textSecondary }}>{t('realTrading.comingSoon.features.websocket')}</span>
          </div>
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <CheckCircle className="w-5 h-5 flex-shrink-0" style={{ color: colors.emerald }} />
            <span className="text-sm" style={{ color: colors.textSecondary }}>{t('realTrading.comingSoon.features.riskManagement')}</span>
          </div>
          <div className="flex items-center gap-3 p-3 rounded-lg" style={{ background: 'rgba(255, 255, 255, 0.03)' }}>
            <Clock className="w-5 h-5 flex-shrink-0" style={{ color: colors.warning }} />
            <span className="text-sm" style={{ color: colors.textSecondary }}>{t('realTrading.comingSoon.features.underDevelopment')}</span>
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
          <Zap className="w-5 h-5" style={{ color: colors.emerald }} />
          <span>{t('realTrading.comingSoon.tryPaperTrading')}</span>
        </motion.button>

        {/* Info text */}
        <p className="mt-6 text-xs" style={{ color: colors.textMuted }}>
          {t('realTrading.comingSoon.hint')}
        </p>
      </GlassCard>
    </motion.div>
  );
}

// ============================================================================
// PENDING ORDER CONFIRMATION DIALOG (Phase 5 - API 2-Step Confirmation)
// ============================================================================

function PendingConfirmationDialog({
  confirmation,
  onConfirm,
  onCancel,
  isLoading,
}: {
  confirmation: { token: string; expires_at: string; summary: string } | null;
  onConfirm: () => void;
  onCancel: () => void;
  isLoading: boolean;
}) {
  const { t } = useTranslation('dashboard');
  const colors = useThemeColors();
  const [timeLeft, setTimeLeft] = useState(60);

  useEffect(() => {
    if (!confirmation) return;

    const expiresAt = new Date(confirmation.expires_at).getTime();
    const updateTimer = () => {
      const now = Date.now();
      const remaining = Math.max(0, Math.floor((expiresAt - now) / 1000));
      setTimeLeft(remaining);
      if (remaining <= 0) {
        onCancel();
      }
    };

    updateTimer();
    const interval = setInterval(updateTimer, 1000);
    return () => clearInterval(interval);
  }, [confirmation, onCancel]);

  if (!confirmation) return null;

  return (
    <AnimatePresence>
      <motion.div
        initial={{ opacity: 0 }}
        animate={{ opacity: 1 }}
        exit={{ opacity: 0 }}
        className="fixed inset-0 z-50 flex items-center justify-center bg-black/80 backdrop-blur-sm"
        onClick={onCancel}
      >
        <motion.div
          initial={{ scale: 0.9, opacity: 0 }}
          animate={{ scale: 1, opacity: 1 }}
          exit={{ scale: 0.9, opacity: 0 }}
          className="relative max-w-md w-full mx-4 p-6 rounded-2xl"
          style={{
            background: 'rgba(13, 17, 23, 0.95)',
            border: '2px solid rgba(239, 68, 68, 0.5)',
            boxShadow: '0 0 40px rgba(239, 68, 68, 0.3)',
          }}
          onClick={(e) => e.stopPropagation()}
        >
          {/* Header */}
          <div className="flex items-center gap-3 mb-4">
            <div className="p-2 rounded-xl" style={{ background: 'rgba(239, 68, 68, 0.2)' }}>
              <AlertOctagon className="w-6 h-6" style={{ color: colors.loss }} />
            </div>
            <div>
              <h3 className="text-lg font-bold text-white">{t('realTrading.confirm.title')}</h3>
              <p className="text-xs" style={{ color: colors.textMuted }}>
                {t('realTrading.confirm.subtitle')}
              </p>
            </div>
          </div>

          {/* Summary */}
          <div
            className="p-4 rounded-xl mb-4"
            style={{ background: 'rgba(239, 68, 68, 0.1)', border: '1px solid rgba(239, 68, 68, 0.2)' }}
          >
            <p className="text-sm text-white font-medium">{confirmation.summary}</p>
          </div>

          {/* Timer */}
          <div className="flex items-center justify-center gap-2 mb-4">
            <Clock className="w-4 h-4" style={{ color: timeLeft <= 10 ? colors.loss : colors.warning }} />
            <span
              className="text-sm font-mono font-bold"
              style={{ color: timeLeft <= 10 ? colors.loss : colors.warning }}
            >
              {timeLeft}s
            </span>
            <span className="text-xs" style={{ color: colors.textMuted }}>
              {t('realTrading.confirm.expires')}
            </span>
          </div>

          {/* Warning */}
          <p className="text-[10px] text-center mb-4" style={{ color: colors.textMuted }}>
            ⚠️ {t('realTrading.confirm.warning')}
          </p>

          {/* Actions */}
          <div className="flex gap-3">
            <motion.button
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              onClick={onCancel}
              disabled={isLoading}
              className="flex-1 py-3 px-4 rounded-xl text-sm font-bold"
              style={{
                background: 'rgba(255, 255, 255, 0.1)',
                border: '1px solid rgba(255, 255, 255, 0.2)',
                color: colors.textSecondary,
              }}
            >
              {t('common.cancel')}
            </motion.button>
            <motion.button
              whileHover={{ scale: 1.02 }}
              whileTap={{ scale: 0.98 }}
              onClick={onConfirm}
              disabled={isLoading}
              className="flex-1 py-3 px-4 rounded-xl text-sm font-bold flex items-center justify-center gap-2"
              style={{
                background: colors.gradientDanger,
                boxShadow: '0 4px 20px rgba(239, 68, 68, 0.4)',
                color: '#fff',
              }}
            >
              {isLoading ? (
                <motion.div animate={{ rotate: 360 }} transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}>
                  <RefreshCw className="w-4 h-4" />
                </motion.div>
              ) : (
                <>
                  <CheckCircle className="w-4 h-4" />
                  {t('realTrading.confirm.execute')}
                </>
              )}
            </motion.button>
          </div>
        </motion.div>
      </motion.div>
    </AnimatePresence>
  );
}

// ============================================================================
// MAIN REAL TRADING PAGE
// ============================================================================

export default function RealTrading() {
  const { t } = useTranslation('dashboard');
  const { toast } = useToast();
  const { mode } = useTradingMode();
  const realTrading = useRealTrading();
  const colors = useThemeColors();
  const [selectedSymbol, setSelectedSymbol] = useState('BTCUSDT');
  const availableSymbols = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT', 'XRPUSDT', 'ADAUSDT'];
  const [showSymbolDropdown, setShowSymbolDropdown] = useState(false);
  const [selectedPrice, setSelectedPrice] = useState<number | undefined>();
  const [activeTab, setActiveTab] = useState<'positions' | 'orders' | 'history'>('positions');
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
      <div className="h-full flex items-center justify-center" style={{ backgroundColor: colors.bgPrimary }}>
        <motion.div
          animate={{ rotate: 360 }}
          transition={{ duration: 1, repeat: Infinity, ease: 'linear' }}
        >
          <RefreshCw className="w-8 h-8" style={{ color: colors.loss }} />
        </motion.div>
      </div>
    );
  }

  const handleOrderSubmit = async (order: OrderFormData) => {
    logger.info('Real trading order submitted:', order);

    // Map frontend order type to API order type
    const orderTypeMap: Record<string, string> = {
      'market': 'MARKET',
      'limit': 'LIMIT',
      'stop-limit': 'STOP_LOSS_LIMIT',
    };

    const success = await realTrading.placeOrder({
      symbol: order.symbol,
      side: order.side.toUpperCase(), // 'BUY' or 'SELL'
      order_type: orderTypeMap[order.orderType] || 'MARKET',
      quantity: order.quantity,
      price: order.price,
      stop_loss: order.stopLoss,
      take_profit: order.takeProfit,
    });

    if (success) {
      toast({
        title: '✅ Order Placed',
        description: `${order.side.toUpperCase()} ${order.quantity} ${order.symbol}`,
      });
    }
    // Error toast is handled by placeOrder
  };

  const handlePriceClick = (price: number) => {
    setSelectedPrice(price);
  };

  return (
    <motion.div
      className="min-h-full lg:h-full flex flex-col overflow-x-hidden overflow-y-auto lg:overflow-y-hidden w-full max-w-full"
      style={{ backgroundColor: colors.bgPrimary }}
      initial="hidden"
      animate="visible"
      variants={containerVariants}
    >
      {/* API Order Confirmation Dialog (Phase 5) */}
      <PendingConfirmationDialog
        confirmation={realTrading.pendingConfirmation}
        onConfirm={realTrading.confirmOrder}
        onCancel={realTrading.clearPendingConfirmation}
        isLoading={realTrading.isLoading}
      />

      {/* Portfolio Stats Bar */}
      <PortfolioStatsBar
        balance={realTrading.portfolio?.current_balance || 0}
        equity={realTrading.portfolio?.equity || 0}
        totalPnl={realTrading.portfolio?.total_pnl || 0}
        totalPnlPercent={realTrading.portfolio?.total_pnl_percentage || 0}
        winRate={realTrading.portfolio?.win_rate || 0}
        totalTrades={realTrading.portfolio?.total_trades || 0}
      />

      {/* Main Trading Grid - Responsive: stacked on mobile, side-by-side on lg+ */}
      {/* Mobile: overflow-visible for natural page scroll, Desktop: contained layout */}
      <div className="flex-1 grid grid-cols-1 lg:grid-cols-12 gap-[1px] min-h-0 lg:overflow-hidden" style={{ backgroundColor: colors.borderSubtle }}>
        {/* Left Column: Chart (full width on mobile, 60% on desktop) */}
        {/* Mobile: no scroll constraints, Desktop: scrollable column */}
        <div
          className="col-span-1 lg:col-span-7 flex flex-col lg:overflow-y-auto overflow-x-hidden custom-scrollbar min-h-[400px] lg:min-h-0 w-full max-w-full"
          style={{ backgroundColor: colors.bgPrimary }}
        >
          {/* Chart Header - Responsive */}
          <motion.div
            className="flex flex-wrap items-center justify-between gap-2 px-2 md:px-4 py-2 md:py-3 border-b border-white/[0.08]"
            initial={{ opacity: 0, y: -10 }}
            animate={{ opacity: 1, y: 0 }}
            transition={{ delay: 0.1 }}
          >
            <div className="flex items-center gap-2 md:gap-4">
              <div className="relative flex items-center gap-2 md:gap-3">
                <div
                  className="p-1.5 md:p-2 rounded-xl"
                  style={{
                    background: 'rgba(239, 68, 68, 0.1)',
                    border: '1px solid rgba(239, 68, 68, 0.2)',
                  }}
                >
                  <LineChart className="w-3.5 h-3.5 md:w-4 md:h-4" style={{ color: colors.loss }} />
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
                    <GradientText className="text-lg font-black" gradient={colors.gradientDanger}>
                      {selectedSymbol.replace('USDT', '')}
                    </GradientText>
                    <span className="text-xs font-medium" style={{ color: colors.textMuted }}>/USDT</span>
                  </div>
                  <ChevronDown
                    className={`w-4 h-4 transition-transform duration-200 ${showSymbolDropdown ? 'rotate-180' : ''}`}
                    style={{ color: colors.textMuted }}
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
                          backgroundColor: colors.bgPrimary,
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
                              color: selectedSymbol === symbol ? colors.loss : colors.textSecondary,
                              backgroundColor: selectedSymbol === symbol ? 'rgba(239, 68, 68, 0.1)' : 'transparent',
                            }}
                          >
                            <span className="font-bold">{symbol.replace('USDT', '')}</span>
                            <span className="text-xs" style={{ color: colors.textMuted }}>/USDT</span>
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
            <div className="flex items-center gap-0.5 md:gap-1 p-0.5 md:p-1 rounded-xl bg-white/[0.03] border border-white/[0.08]">
              {timeframes.map((tf) => (
                <motion.button
                  key={tf}
                  onClick={() => setSelectedTimeframe(tf)}
                  whileHover={{ scale: 1.05 }}
                  whileTap={{ scale: 0.95 }}
                  className="px-2 md:px-3 py-1 md:py-1.5 text-[9px] md:text-[10px] font-bold uppercase tracking-wider rounded-lg transition-all duration-300"
                  style={{
                    background: selectedTimeframe === tf
                      ? 'linear-gradient(135deg, rgba(239, 68, 68, 0.2), rgba(249, 115, 22, 0.2))'
                      : 'transparent',
                    border: selectedTimeframe === tf ? '1px solid rgba(239, 68, 68, 0.3)' : '1px solid transparent',
                    color: selectedTimeframe === tf ? colors.loss : colors.textMuted,
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
          {/* Mobile: no max-height for natural flow, Desktop: capped height */}
          <div
            className="rounded-t-2xl flex flex-col min-h-[200px] lg:min-h-[250px] lg:max-h-[40vh]"
            style={{ backgroundColor: 'rgba(255, 255, 255, 0.02)' }}
          >
            <div className="flex border-b border-white/[0.08]">
              {[
                { id: 'positions', label: t('realTrading.positions.title'), icon: Activity, count: realTrading.openTrades.length },
                { id: 'orders', label: t('realTrading.orders.title'), icon: Target, count: realTrading.activeOrders.length },
                { id: 'history', label: t('realTrading.history.title'), icon: Clock },
              ].map((tab) => (
                <motion.button
                  key={tab.id}
                  onClick={() => setActiveTab(tab.id as 'positions' | 'orders' | 'history')}
                  whileHover={{ scale: 1.02 }}
                  whileTap={{ scale: 0.98 }}
                  className="relative px-3 md:px-5 py-2 md:py-3 text-[10px] md:text-xs font-bold flex items-center gap-1.5 md:gap-2"
                  style={{ color: activeTab === tab.id ? colors.loss : colors.textMuted }}
                >
                  <tab.icon className="w-3.5 h-3.5 md:w-4 md:h-4" />
                  {tab.label}
                  {tab.count !== undefined && (
                    <span
                      className="px-2 py-0.5 text-[10px] rounded-full font-bold"
                      style={{
                        background: activeTab === tab.id ? 'rgba(239, 68, 68, 0.2)' : 'rgba(255, 255, 255, 0.1)',
                        color: activeTab === tab.id ? colors.loss : colors.textSecondary,
                      }}
                    >
                      {tab.count}
                    </span>
                  )}
                  {activeTab === tab.id && (
                    <motion.div
                      layoutId="activeRealTabIndicator"
                      className="absolute bottom-0 left-0 right-0 h-[2px]"
                      style={{ background: colors.gradientDanger, boxShadow: '0 0 10px rgba(239, 68, 68, 0.5)' }}
                    />
                  )}
                </motion.button>
              ))}
            </div>

            <div className="flex-1 min-h-0 overflow-hidden">
              <AnimatePresence mode="wait">
                {activeTab === 'positions' && (
                  <motion.div key="positions" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} className="h-full overflow-auto">
                    <PositionsTable trades={realTrading.openTrades} isLoading={realTrading.isLoading} onCloseTrade={realTrading.closeTrade} />
                  </motion.div>
                )}
                {activeTab === 'orders' && (
                  <motion.div key="orders" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} className="h-full overflow-auto">
                    <OrdersTable orders={realTrading.activeOrders} isLoading={realTrading.isLoading} onCancelOrder={realTrading.cancelOrder} />
                  </motion.div>
                )}
                {activeTab === 'history' && (
                  <motion.div key="history" initial={{ opacity: 0, y: 10 }} animate={{ opacity: 1, y: 0 }} exit={{ opacity: 0, y: -10 }} className="h-full overflow-auto">
                    <TradeHistoryTable trades={realTrading.closedTrades} isLoading={realTrading.isLoading} />
                  </motion.div>
                )}
              </AnimatePresence>
            </div>
          </div>
        </div>

        {/* Right Column: Order Book + Form (full width on mobile, 40% on desktop) */}
        {/* Desktop: no scroll - content should fit naturally */}
        <div className="col-span-1 lg:col-span-5 flex flex-col overflow-hidden w-full max-w-full" style={{ backgroundColor: colors.bgPrimary }}>
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-[1px] h-full" style={{ backgroundColor: colors.borderSubtle }}>
            {/* Order Book */}
            <div className="h-full flex flex-col overflow-hidden" style={{ backgroundColor: colors.bgPrimary }}>
              <OrderBook symbol={selectedSymbol} onPriceClick={handlePriceClick} />
            </div>

            {/* Order Form */}
            <div className="h-full flex flex-col min-h-0 overflow-hidden" style={{ backgroundColor: colors.bgPrimary }}>
              <div className="flex-1 min-h-0">
                <OrderForm symbol={selectedSymbol} onSubmit={handleOrderSubmit} selectedPrice={selectedPrice} />
              </div>

              {/* Risk Warning */}
              <div className="p-4 flex-shrink-0">
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
                      <AlertOctagon className="w-4 h-4" style={{ color: colors.loss }} />
                    </div>
                    <div>
                      <p className="text-xs font-bold" style={{ color: colors.loss }}>{t('realTrading.warning.title')}</p>
                      <p className="text-[10px] mt-1.5 leading-relaxed" style={{ color: colors.textSecondary }}>
                        {t('realTrading.riskWarning.message')}
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
