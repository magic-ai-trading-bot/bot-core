import React, { useState, useEffect, useMemo } from "react";
import { motion } from "framer-motion";
import { useTranslation } from "react-i18next";
import {
  PieChart,
  Pie,
  Cell,
  ResponsiveContainer,
  AreaChart,
  Area,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
} from "recharts";
import {
  TrendingUp,
  TrendingDown,
  Wallet,
  ChevronUp,
  ChevronDown,
  Sparkles,
  Trophy,
  AlertTriangle,
  BarChart3,
  Layers,
  Clock,
  Target,
  Shield,
  Activity,
  RefreshCw,
} from "lucide-react";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  GlassCard,
  GradientText,
  Badge,
  GlowIcon,
  PageWrapper,
  containerVariants,
  itemVariants,
} from "@/styles/luxury-design-system";
import { usePaperTrading } from "@/hooks/usePaperTrading";
import { useThemeColors } from "@/hooks/useThemeColors";

// ============================================================================
// INTERFACES
// ============================================================================

interface PerformanceDataPoint {
  date: string;
  value: number;
  pnl: number;
}

// ============================================================================
// ANIMATED COUNTER COMPONENT
// ============================================================================

interface AnimatedCounterProps {
  value: number;
  prefix?: string;
  suffix?: string;
  decimals?: number;
  duration?: number;
  className?: string;
}

const AnimatedCounter = ({
  value,
  prefix = "",
  suffix = "",
  decimals = 2,
  duration = 2000,
  className = "",
}: AnimatedCounterProps) => {
  const [displayValue, setDisplayValue] = useState(0);
  const startValueRef = React.useRef(0);

  useEffect(() => {
    let startTime: number;
    let animationFrame: number;
    startValueRef.current = displayValue;

    const animate = (timestamp: number) => {
      if (!startTime) startTime = timestamp;
      const progress = Math.min((timestamp - startTime) / duration, 1);

      // Easing function for smooth animation
      const easeOutQuart = 1 - Math.pow(1 - progress, 4);
      const current = startValueRef.current + (value - startValueRef.current) * easeOutQuart;

      setDisplayValue(current);

      if (progress < 1) {
        animationFrame = requestAnimationFrame(animate);
      }
    };

    animationFrame = requestAnimationFrame(animate);
    return () => cancelAnimationFrame(animationFrame);
     
  }, [value, duration]);

  const formatNumber = (num: number) => {
    return new Intl.NumberFormat("en-US", {
      minimumFractionDigits: decimals,
      maximumFractionDigits: decimals,
    }).format(num);
  };

  return (
    <span className={className}>
      {prefix}
      {formatNumber(displayValue)}
      {suffix}
    </span>
  );
};

// ============================================================================
// CUSTOM TOOLTIP COMPONENTS
// ============================================================================

interface ChartTooltipProps {
  active?: boolean;
  payload?: Array<{ value: number; payload: PerformanceDataPoint }>;
  label?: string;
}

const PerformanceTooltip = ({ active, payload, label }: ChartTooltipProps) => {
  if (!active || !payload || !payload.length) return null;

  const data = payload[0].payload;
  const isPositive = data.pnl >= 0;

  return (
    <div
      className="backdrop-blur-xl rounded-xl p-4 shadow-2xl"
      style={{
        backgroundColor: 'rgba(0, 0, 0, 0.95)',
        border: `1px solid ${themeColors.borderSubtle}`,
      }}
    >
      <p className="text-sm mb-2" style={{ color: themeColors.textMuted }}>
        {new Date(label || "").toLocaleDateString("en-US", {
          month: "short",
          day: "numeric",
          year: "numeric",
        })}
      </p>
      <p className="font-semibold text-lg" style={{ color: themeColors.textPrimary }}>
        ${data.value.toLocaleString("en-US", { minimumFractionDigits: 2 })}
      </p>
      <p className="text-sm mt-1" style={{ color: isPositive ? themeColors.profit : themeColors.loss }}>
        {isPositive ? "+" : ""}${data.pnl.toLocaleString("en-US", { minimumFractionDigits: 2 })}
      </p>
    </div>
  );
};

// ============================================================================
// SORT UTILITIES
// ============================================================================

type SortKey = "pnl" | "symbol" | "date";
type SortDirection = "asc" | "desc";
type TimePeriod = "24H" | "7D" | "30D" | "ALL";

interface SortIconProps {
  columnKey: SortKey;
  sortKey: SortKey;
  sortDirection: SortDirection;
}

const SortIcon = ({ columnKey, sortKey, sortDirection }: SortIconProps) => {
  if (sortKey !== columnKey) {
    return <ChevronUp className="w-4 h-4 ml-1" style={{ color: themeColors.textMuted }} />;
  }
  return sortDirection === "asc" ? (
    <ChevronUp className="w-4 h-4 ml-1" style={{ color: themeColors.cyan }} />
  ) : (
    <ChevronDown className="w-4 h-4 ml-1" style={{ color: themeColors.cyan }} />
  );
};

// ============================================================================
// MAIN PORTFOLIO COMPONENT - REAL DATA FROM PAPER TRADING
// ============================================================================

const Portfolio = () => {
  const { t } = useTranslation('dashboard');
  const [sortKey, setSortKey] = useState<SortKey>("date");
  const [sortDirection, setSortDirection] = useState<SortDirection>("desc");
  const [timePeriod, setTimePeriod] = useState<TimePeriod>("30D");
  const [isLoaded, setIsLoaded] = useState(false);
  const themeColors = useThemeColors();

  // Get real data from paper trading hook
  const {
    portfolio,
    openTrades,
    closedTrades,
    isLoading,
    isActive,
    refreshAll,
  } = usePaperTrading();

  useEffect(() => {
    // Trigger animations after mount
    const timer = setTimeout(() => setIsLoaded(true), 100);
    return () => clearTimeout(timer);
  }, []);

  // Calculate real portfolio metrics from paper trading data
  const portfolioMetrics = useMemo(() => {
    const initialBalance = 10000; // Default initial balance
    const currentBalance = portfolio.current_balance;
    const equity = portfolio.equity;
    const totalPnL = portfolio.total_pnl;
    const totalPnLPercentage = portfolio.total_pnl_percentage;
    const winRate = portfolio.win_rate;
    const totalTrades = portfolio.total_trades;

    return {
      currentBalance,
      equity,
      totalPnL,
      totalPnLPercentage,
      winRate,
      totalTrades,
      initialBalance,
      marginUsed: portfolio.margin_used,
      freeMargin: portfolio.free_margin,
      maxDrawdown: portfolio.max_drawdown,
      maxDrawdownPct: portfolio.max_drawdown_percentage,
      sharpeRatio: portfolio.sharpe_ratio,
      profitFactor: portfolio.profit_factor,
      averageWin: portfolio.average_win,
      averageLoss: portfolio.average_loss,
      largestWin: portfolio.largest_win,
      largestLoss: portfolio.largest_loss,
    };
  }, [portfolio]);

  // Calculate symbol allocation from open trades
  const allocationData = useMemo(() => {
    const symbolMap = new Map<string, { value: number; trades: number; color: string }>();
    const colors = ["#F7931A", "#627EEA", "#00FFA3", "#F3BA2F", "#23292F", "#0033AD", "#E84142", "#8247E5"];

    openTrades.forEach((trade) => {
      const positionValue = trade.quantity * trade.entry_price;
      const existing = symbolMap.get(trade.symbol) || { value: 0, trades: 0, color: colors[symbolMap.size % colors.length] };
      symbolMap.set(trade.symbol, {
        value: existing.value + positionValue,
        trades: existing.trades + 1,
        color: existing.color,
      });
    });

    const totalValue = Array.from(symbolMap.values()).reduce((sum, item) => sum + item.value, 0);

    return Array.from(symbolMap.entries()).map(([symbol, data]) => ({
      name: symbol,
      symbol,
      value: data.value,
      trades: data.trades,
      percentage: totalValue > 0 ? (data.value / totalValue) * 100 : 0,
      color: data.color,
    })).sort((a, b) => b.value - a.value);
  }, [openTrades]);

  // Generate performance data from closed trades
  const performanceData = useMemo(() => {
    const now = new Date();
    const days = timePeriod === "24H" ? 1 : timePeriod === "7D" ? 7 : timePeriod === "30D" ? 30 : 90;
    const cutoffDate = new Date(now.getTime() - days * 24 * 60 * 60 * 1000);

    // Group trades by date
    const tradesByDate = new Map<string, number>();
    let runningBalance = portfolioMetrics.initialBalance;

    // Sort trades by date
    const sortedTrades = [...closedTrades]
      .filter(trade => new Date(trade.close_time || trade.open_time) >= cutoffDate)
      .sort((a, b) => new Date(a.close_time || a.open_time).getTime() - new Date(b.close_time || b.open_time).getTime());

    sortedTrades.forEach(trade => {
      const date = new Date(trade.close_time || trade.open_time).toISOString().split("T")[0];
      const existingPnL = tradesByDate.get(date) || 0;
      tradesByDate.set(date, existingPnL + (trade.pnl || 0));
    });

    // Generate data points for each day
    const data: PerformanceDataPoint[] = [];
    for (let i = days; i >= 0; i--) {
      const date = new Date(now);
      date.setDate(date.getDate() - i);
      const dateStr = date.toISOString().split("T")[0];

      const dayPnL = tradesByDate.get(dateStr) || 0;
      runningBalance += dayPnL;

      data.push({
        date: dateStr,
        value: runningBalance,
        pnl: runningBalance - portfolioMetrics.initialBalance,
      });
    }

    // Ensure last value matches current balance
    if (data.length > 0) {
      data[data.length - 1].value = portfolioMetrics.currentBalance;
      data[data.length - 1].pnl = portfolioMetrics.totalPnL;
    }

    return data;
  }, [closedTrades, timePeriod, portfolioMetrics]);

  // Sorted closed trades
  const sortedTrades = useMemo(() => {
    return [...closedTrades].sort((a, b) => {
      let aVal: number | string;
      let bVal: number | string;

      switch (sortKey) {
        case "pnl":
          aVal = a.pnl || 0;
          bVal = b.pnl || 0;
          break;
        case "symbol":
          aVal = a.symbol;
          bVal = b.symbol;
          break;
        case "date":
          aVal = new Date(a.close_time || a.open_time).getTime();
          bVal = new Date(b.close_time || b.open_time).getTime();
          break;
        default:
          return 0;
      }

      if (typeof aVal === "string" && typeof bVal === "string") {
        return sortDirection === "asc"
          ? aVal.localeCompare(bVal)
          : bVal.localeCompare(aVal);
      }

      return sortDirection === "asc"
        ? (aVal as number) - (bVal as number)
        : (bVal as number) - (aVal as number);
    });
  }, [closedTrades, sortKey, sortDirection]);

  // Find best/worst trades
  const bestTrade = useMemo(() => {
    if (closedTrades.length === 0) return null;
    return [...closedTrades].sort((a, b) => (b.pnl || 0) - (a.pnl || 0))[0];
  }, [closedTrades]);

  const worstTrade = useMemo(() => {
    if (closedTrades.length === 0) return null;
    return [...closedTrades].sort((a, b) => (a.pnl || 0) - (b.pnl || 0))[0];
  }, [closedTrades]);

  const handleSort = (key: SortKey) => {
    if (sortKey === key) {
      setSortDirection(sortDirection === "asc" ? "desc" : "asc");
    } else {
      setSortKey(key);
      setSortDirection("desc");
    }
  };

  return (
    <PageWrapper>
      {/* Animated Background */}
      <div className="fixed inset-0 overflow-hidden pointer-events-none">
        <div
          className="absolute top-0 left-1/4 w-[600px] h-[600px] blur-[120px] animate-pulse-slow"
          style={{
            background: `radial-gradient(circle, ${themeColors.cyan}15 0%, transparent 70%)`,
          }}
        />
        <div
          className="absolute bottom-1/4 right-1/4 w-[500px] h-[500px] blur-[100px] animate-pulse-slow"
          style={{
            background: `radial-gradient(circle, ${themeColors.purple}15 0%, ${themeColors.emerald}08 50%, transparent 70%)`,
            animationDelay: "1.5s",
          }}
        />
      </div>

      <motion.div className="relative z-10 space-y-6" variants={containerVariants}>
        {/* ================================================================
            SECTION 1: PORTFOLIO SUMMARY HERO
            ================================================================ */}
        <motion.div variants={itemVariants}>
          <GlassCard className="overflow-hidden">
            {/* Decorative glow */}
            <div
              className="absolute top-0 left-1/2 -translate-x-1/2 w-[400px] h-[200px] blur-[60px]"
              style={{
                background: `linear-gradient(to bottom, ${themeColors.cyan}30, transparent)`,
              }}
            />

            <div className="relative p-8">
              {/* Header */}
              <div className="flex items-center justify-between mb-8">
                <div className="flex items-center gap-3">
                  <GlowIcon icon={Wallet} color={themeColors.cyan} size="lg" />
                  <div>
                    <h1 className="text-2xl font-bold" style={{ color: themeColors.textPrimary }}>
                      {t('portfolio.title')}
                    </h1>
                    <p className="text-sm" style={{ color: themeColors.textMuted }}>
                      {t('portfolio.subtitle')}
                    </p>
                  </div>
                </div>
                <div className="flex items-center gap-3">
                  <button
                    onClick={refreshAll}
                    className="p-2 rounded-lg transition-colors"
                    style={{ backgroundColor: themeColors.bgTertiary }}
                    disabled={isLoading}
                  >
                    <RefreshCw className={`w-5 h-5 ${isLoading ? 'animate-spin' : ''}`} style={{ color: themeColors.textMuted }} />
                  </button>
                  <Badge variant={isActive ? "success" : "warning"} glow>
                    <Sparkles className="w-3 h-3 mr-1" />
                    {isActive ? t('portfolio.status.live') : t('portfolio.status.inactive')}
                  </Badge>
                </div>
              </div>

              {/* Main Value Display */}
              <div className="text-center mb-8">
                <div className="inline-block">
                  <p
                    className="text-sm mb-2 uppercase tracking-wider"
                    style={{ color: themeColors.textMuted }}
                  >
                    {t('portfolio.currentBalance')}
                  </p>
                  <h2 className="text-5xl md:text-6xl lg:text-7xl font-bold">
                    <GradientText>
                      $<AnimatedCounter value={portfolioMetrics.currentBalance} decimals={2} duration={2500} />
                    </GradientText>
                  </h2>
                </div>
              </div>

              {/* Change Indicators */}
              <div className="flex flex-wrap justify-center gap-4 md:gap-8">
                {/* Win Rate */}
                <div
                  className="flex flex-col items-center p-4 rounded-xl backdrop-blur-sm min-w-[140px]"
                  style={{
                    backgroundColor: themeColors.bgTertiary,
                    border: `1px solid ${themeColors.borderSubtle}`,
                  }}
                >
                  <span className="text-xs uppercase tracking-wider mb-1" style={{ color: themeColors.textMuted }}>
                    {t('portfolio.stats.winRate')}
                  </span>
                  <div
                    className="flex items-center gap-1 text-lg font-semibold"
                    style={{
                      color: portfolioMetrics.winRate >= 50 ? themeColors.profit : themeColors.loss,
                    }}
                  >
                    <Target className="w-5 h-5" />
                    {portfolioMetrics.winRate.toFixed(1)}%
                  </div>
                </div>

                {/* Total Trades */}
                <div
                  className="flex flex-col items-center p-4 rounded-xl backdrop-blur-sm min-w-[140px]"
                  style={{
                    backgroundColor: themeColors.bgTertiary,
                    border: `1px solid ${themeColors.borderSubtle}`,
                  }}
                >
                  <span className="text-xs uppercase tracking-wider mb-1" style={{ color: themeColors.textMuted }}>
                    {t('portfolio.stats.totalTrades')}
                  </span>
                  <div className="flex items-center gap-1 text-lg font-semibold" style={{ color: themeColors.textPrimary }}>
                    <Activity className="w-5 h-5" />
                    {portfolioMetrics.totalTrades}
                  </div>
                </div>

                {/* Profit Factor */}
                <div
                  className="flex flex-col items-center p-4 rounded-xl backdrop-blur-sm min-w-[140px]"
                  style={{
                    backgroundColor: themeColors.bgTertiary,
                    border: `1px solid ${themeColors.borderSubtle}`,
                  }}
                >
                  <span className="text-xs uppercase tracking-wider mb-1" style={{ color: themeColors.textMuted }}>
                    {t('portfolio.stats.profitFactor')}
                  </span>
                  <div className="flex items-center gap-1 text-lg font-semibold" style={{ color: portfolioMetrics.profitFactor >= 1 ? themeColors.profit : themeColors.loss }}>
                    <BarChart3 className="w-5 h-5" />
                    {portfolioMetrics.profitFactor.toFixed(2)}
                  </div>
                </div>

                {/* Total P&L */}
                <div
                  className="flex flex-col items-center p-4 rounded-xl backdrop-blur-sm min-w-[160px]"
                  style={{
                    background: `linear-gradient(135deg, ${portfolioMetrics.totalPnL >= 0 ? themeColors.profit : themeColors.loss}15, transparent)`,
                    border: `1px solid ${portfolioMetrics.totalPnL >= 0 ? themeColors.profit : themeColors.loss}30`,
                  }}
                >
                  <span className="text-xs uppercase tracking-wider mb-1" style={{ color: themeColors.textMuted }}>
                    {t('portfolio.stats.totalPnl')}
                  </span>
                  <div className="flex items-center gap-1 text-lg font-semibold" style={{ color: portfolioMetrics.totalPnL >= 0 ? themeColors.profit : themeColors.loss }}>
                    {portfolioMetrics.totalPnL >= 0 ? <TrendingUp className="w-5 h-5" /> : <TrendingDown className="w-5 h-5" />}
                    {portfolioMetrics.totalPnL >= 0 ? "+" : ""}$<AnimatedCounter value={portfolioMetrics.totalPnL} decimals={2} duration={2000} />
                  </div>
                </div>
              </div>
            </div>
          </GlassCard>
        </motion.div>

        {/* ================================================================
            SECTION 2 & 3: POSITION ALLOCATION + OPEN TRADES
            ================================================================ */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Position Allocation Donut Chart */}
          <motion.div className="lg:col-span-1" variants={itemVariants}>
            <GlassCard noPadding>
              <div className="p-6 pb-2">
                <div className="flex items-center gap-2 mb-4">
                  <GlowIcon icon={Layers} color={themeColors.cyan} size="md" />
                  <h3 className="text-lg font-semibold" style={{ color: themeColors.textPrimary }}>
                    {t('portfolio.sections.allocation')}
                  </h3>
                </div>
              </div>
              <div className="px-6 pb-6">
                {openTrades.length > 0 ? (
                  <>
                    <div className="h-[280px] relative">
                      <ResponsiveContainer width="100%" height="100%">
                        <PieChart>
                          <Pie
                            data={allocationData}
                            cx="50%"
                            cy="50%"
                            innerRadius={70}
                            outerRadius={100}
                            paddingAngle={2}
                            dataKey="value"
                            stroke="none"
                          >
                            {allocationData.map((entry, index) => (
                              <Cell
                                key={`cell-${index}`}
                                fill={entry.color}
                                className="transition-all duration-300 hover:opacity-80"
                                style={{
                                  filter: "drop-shadow(0 0 8px rgba(0,0,0,0.3))",
                                }}
                              />
                            ))}
                          </Pie>
                          <Tooltip />
                        </PieChart>
                      </ResponsiveContainer>

                      {/* Center text */}
                      <div className="absolute top-1/2 left-1/2 -translate-x-1/2 -translate-y-1/2 text-center">
                        <p className="text-xs uppercase tracking-wider" style={{ color: themeColors.textMuted }}>
                          {t('portfolio.stats.openPositions')}
                        </p>
                        <p className="text-2xl font-bold" style={{ color: themeColors.textPrimary }}>
                          {openTrades.length}
                        </p>
                      </div>
                    </div>

                    {/* Legend */}
                    <div className="grid grid-cols-2 gap-2 mt-4">
                      {allocationData.slice(0, 6).map((asset) => (
                        <div
                          key={asset.symbol}
                          className="flex items-center gap-2 p-2 rounded-lg transition-colors"
                          style={{ backgroundColor: 'transparent' }}
                        >
                          <div
                            className="w-3 h-3 rounded-full"
                            style={{ backgroundColor: asset.color }}
                          />
                          <span className="text-sm" style={{ color: themeColors.textSecondary }}>
                            {asset.symbol}
                          </span>
                          <span className="text-sm font-medium ml-auto" style={{ color: themeColors.textPrimary }}>
                            {asset.percentage.toFixed(1)}%
                          </span>
                        </div>
                      ))}
                    </div>
                  </>
                ) : (
                  <div className="h-[280px] flex items-center justify-center">
                    <div className="text-center">
                      <Layers className="w-12 h-12 mx-auto mb-3" style={{ color: themeColors.textMuted }} />
                      <p style={{ color: themeColors.textMuted }}>{t('portfolio.empty.noPositions')}</p>
                      <p className="text-sm" style={{ color: themeColors.textMuted }}>{t('portfolio.empty.startTrading')}</p>
                    </div>
                  </div>
                )}
              </div>
            </GlassCard>
          </motion.div>

          {/* Recent Closed Trades Table */}
          <motion.div className="lg:col-span-2" variants={itemVariants}>
            <GlassCard noPadding>
              <div className="p-6 pb-2">
                <div className="flex items-center gap-2">
                  <GlowIcon icon={BarChart3} color={themeColors.cyan} size="md" />
                  <h3 className="text-lg font-semibold" style={{ color: themeColors.textPrimary }}>
                    {t('portfolio.sections.recentTrades')} ({closedTrades.length})
                  </h3>
                </div>
              </div>
              <div className="px-6 pb-6">
                {closedTrades.length > 0 ? (
                  <div className="overflow-x-auto max-h-[400px] overflow-y-auto">
                    <table className="w-full">
                      <thead className="sticky top-0" style={{ backgroundColor: themeColors.bgSecondary }}>
                        <tr style={{ borderBottom: `1px solid ${themeColors.borderSubtle}` }}>
                          <th
                            className="text-left py-3 px-2 text-sm font-medium cursor-pointer"
                            style={{ color: themeColors.textMuted }}
                            onClick={() => handleSort("symbol")}
                          >
                            <div className="flex items-center">
                              {t('portfolio.table.symbol')}
                              <SortIcon columnKey="symbol" sortKey={sortKey} sortDirection={sortDirection} />
                            </div>
                          </th>
                          <th className="text-right py-3 px-2 text-sm font-medium" style={{ color: themeColors.textMuted }}>
                            {t('portfolio.table.type')}
                          </th>
                          <th className="text-right py-3 px-2 text-sm font-medium" style={{ color: themeColors.textMuted }}>
                            {t('portfolio.table.entryExit')}
                          </th>
                          <th
                            className="text-right py-3 px-2 text-sm font-medium cursor-pointer"
                            style={{ color: themeColors.textMuted }}
                            onClick={() => handleSort("pnl")}
                          >
                            <div className="flex items-center justify-end">
                              {t('portfolio.table.pnl')}
                              <SortIcon columnKey="pnl" sortKey={sortKey} sortDirection={sortDirection} />
                            </div>
                          </th>
                          <th
                            className="text-right py-3 px-2 text-sm font-medium cursor-pointer"
                            style={{ color: themeColors.textMuted }}
                            onClick={() => handleSort("date")}
                          >
                            <div className="flex items-center justify-end">
                              {t('portfolio.table.date')}
                              <SortIcon columnKey="date" sortKey={sortKey} sortDirection={sortDirection} />
                            </div>
                          </th>
                        </tr>
                      </thead>
                      <tbody>
                        {sortedTrades.slice(0, 20).map((trade) => {
                          const pnl = trade.pnl || 0;
                          const isProfit = pnl >= 0;

                          return (
                            <tr
                              key={trade.id}
                              className="transition-colors"
                              style={{ borderBottom: `1px solid ${themeColors.borderSubtle}` }}
                            >
                              <td className="py-3 px-2">
                                <div className="flex items-center gap-2">
                                  <div
                                    className="w-8 h-8 rounded-full flex items-center justify-center text-xs font-bold"
                                    style={{ backgroundColor: trade.trade_type === "Long" ? `${themeColors.profit}20` : `${themeColors.loss}20` }}
                                  >
                                    {trade.symbol.slice(0, 3)}
                                  </div>
                                  <span className="font-medium" style={{ color: themeColors.textPrimary }}>
                                    {trade.symbol}
                                  </span>
                                </div>
                              </td>
                              <td className="py-3 px-2 text-right">
                                <Badge variant={trade.trade_type === "Long" ? "success" : "error"}>
                                  {trade.trade_type}
                                </Badge>
                              </td>
                              <td className="py-3 px-2 text-right">
                                <p style={{ color: themeColors.textPrimary }}>${trade.entry_price.toFixed(2)}</p>
                                <p className="text-sm" style={{ color: themeColors.textMuted }}>
                                  ${trade.exit_price?.toFixed(2) || "-"}
                                </p>
                              </td>
                              <td className="py-3 px-2 text-right">
                                <p className="font-medium" style={{ color: isProfit ? themeColors.profit : themeColors.loss }}>
                                  {isProfit ? "+" : ""}${pnl.toFixed(2)}
                                </p>
                                <p className="text-sm" style={{ color: isProfit ? `${themeColors.profit}80` : `${themeColors.loss}80` }}>
                                  {trade.pnl_percentage >= 0 ? "+" : ""}{trade.pnl_percentage.toFixed(2)}%
                                </p>
                              </td>
                              <td className="py-3 px-2 text-right text-sm" style={{ color: themeColors.textMuted }}>
                                {new Date(trade.close_time || trade.open_time).toLocaleDateString()}
                              </td>
                            </tr>
                          );
                        })}
                      </tbody>
                    </table>
                  </div>
                ) : (
                  <div className="h-[200px] flex items-center justify-center">
                    <div className="text-center">
                      <BarChart3 className="w-12 h-12 mx-auto mb-3" style={{ color: themeColors.textMuted }} />
                      <p style={{ color: themeColors.textMuted }}>{t('portfolio.empty.noClosedTrades')}</p>
                    </div>
                  </div>
                )}
              </div>
            </GlassCard>
          </motion.div>
        </div>

        {/* ================================================================
            SECTION 4: PERFORMANCE CHART
            ================================================================ */}
        <motion.div variants={itemVariants}>
          <GlassCard noPadding>
            <div className="p-6 pb-2">
              <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
                <div className="flex items-center gap-2">
                  <GlowIcon icon={Clock} color={themeColors.cyan} size="md" />
                  <h3 className="text-lg font-semibold" style={{ color: themeColors.textPrimary }}>
                    {t('portfolio.sections.performance')}
                  </h3>
                </div>
                <Tabs value={timePeriod} onValueChange={(v) => setTimePeriod(v as TimePeriod)}>
                  <TabsList
                    style={{
                      backgroundColor: themeColors.bgTertiary,
                      border: `1px solid ${themeColors.borderSubtle}`,
                    }}
                  >
                    {["24H", "7D", "30D", "ALL"].map((period) => (
                      <TabsTrigger
                        key={period}
                        value={period}
                        style={{
                          color: timePeriod === period ? themeColors.textPrimary : themeColors.textMuted,
                        }}
                      >
                        {period}
                      </TabsTrigger>
                    ))}
                  </TabsList>
                </Tabs>
              </div>
            </div>
            <div className="px-6 pb-6">
              <div className="h-[350px]">
                <ResponsiveContainer width="100%" height="100%">
                  <AreaChart
                    data={performanceData}
                    margin={{ top: 10, right: 30, left: 0, bottom: 0 }}
                  >
                    <defs>
                      <linearGradient id="portfolioGradient" x1="0" y1="0" x2="0" y2="1">
                        <stop offset="0%" stopColor={themeColors.cyan} stopOpacity={0.4} />
                        <stop offset="50%" stopColor={themeColors.purple} stopOpacity={0.2} />
                        <stop offset="100%" stopColor={themeColors.purple} stopOpacity={0} />
                      </linearGradient>
                      <linearGradient id="lineGradient" x1="0" y1="0" x2="1" y2="0">
                        <stop offset="0%" stopColor={themeColors.purple} />
                        <stop offset="50%" stopColor={themeColors.cyan} />
                        <stop offset="100%" stopColor={themeColors.emerald} />
                      </linearGradient>
                    </defs>
                    <CartesianGrid
                      strokeDasharray="3 3"
                      stroke={themeColors.borderSubtle}
                      vertical={false}
                    />
                    <XAxis
                      dataKey="date"
                      axisLine={false}
                      tickLine={false}
                      tick={{ fill: themeColors.textMuted, fontSize: 12 }}
                      tickFormatter={(value) =>
                        new Date(value).toLocaleDateString("en-US", {
                          month: "short",
                          day: "numeric",
                        })
                      }
                      interval="preserveStartEnd"
                    />
                    <YAxis
                      axisLine={false}
                      tickLine={false}
                      tick={{ fill: themeColors.textMuted, fontSize: 12 }}
                      tickFormatter={(value) =>
                        `$${(value / 1000).toFixed(0)}k`
                      }
                      domain={["dataMin - 500", "dataMax + 500"]}
                    />
                    <Tooltip content={<PerformanceTooltip />} />
                    <Area
                      type="monotone"
                      dataKey="value"
                      stroke="url(#lineGradient)"
                      strokeWidth={3}
                      fill="url(#portfolioGradient)"
                      dot={false}
                      activeDot={{
                        r: 6,
                        fill: themeColors.cyan,
                        stroke: themeColors.bgPrimary,
                        strokeWidth: 3,
                      }}
                    />
                  </AreaChart>
                </ResponsiveContainer>
              </div>
            </div>
          </GlassCard>
        </motion.div>

        {/* ================================================================
            SECTION 5: QUICK STATS CARDS
            ================================================================ */}
        <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-4">
          {/* Best Trade */}
          <motion.div variants={itemVariants}>
            <GlassCard className="overflow-hidden" noPadding>
              <div className="p-5 relative">
                <div
                  className="absolute top-0 right-0 w-24 h-24 blur-2xl"
                  style={{
                    background: `linear-gradient(to bottom left, ${themeColors.profit}30, transparent)`,
                  }}
                />
                <div className="relative">
                  <div className="flex items-center justify-between mb-3">
                    <GlowIcon icon={Trophy} color={themeColors.profit} size="md" />
                    {bestTrade && (
                      <Badge variant="success">
                        +{bestTrade.pnl_percentage.toFixed(2)}%
                      </Badge>
                    )}
                  </div>
                  <p className="text-sm mb-1" style={{ color: themeColors.textMuted }}>{t('portfolio.cards.bestTrade')}</p>
                  {bestTrade ? (
                    <div>
                      <p className="font-semibold" style={{ color: themeColors.textPrimary }}>
                        {bestTrade.symbol}
                      </p>
                      <p className="text-sm font-medium" style={{ color: themeColors.profit }}>
                        +${(bestTrade.pnl || 0).toFixed(2)}
                      </p>
                    </div>
                  ) : (
                    <p style={{ color: themeColors.textMuted }}>{t('portfolio.cards.noTradesYet')}</p>
                  )}
                </div>
              </div>
            </GlassCard>
          </motion.div>

          {/* Worst Trade */}
          <motion.div variants={itemVariants}>
            <GlassCard className="overflow-hidden" noPadding>
              <div className="p-5 relative">
                <div
                  className="absolute top-0 right-0 w-24 h-24 blur-2xl"
                  style={{
                    background: `linear-gradient(to bottom left, ${themeColors.loss}30, transparent)`,
                  }}
                />
                <div className="relative">
                  <div className="flex items-center justify-between mb-3">
                    <GlowIcon icon={AlertTriangle} color={themeColors.loss} size="md" />
                    {worstTrade && (
                      <Badge variant="error">
                        {worstTrade.pnl_percentage.toFixed(2)}%
                      </Badge>
                    )}
                  </div>
                  <p className="text-sm mb-1" style={{ color: themeColors.textMuted }}>{t('portfolio.cards.worstTrade')}</p>
                  {worstTrade ? (
                    <div>
                      <p className="font-semibold" style={{ color: themeColors.textPrimary }}>
                        {worstTrade.symbol}
                      </p>
                      <p className="text-sm font-medium" style={{ color: themeColors.loss }}>
                        ${(worstTrade.pnl || 0).toFixed(2)}
                      </p>
                    </div>
                  ) : (
                    <p style={{ color: themeColors.textMuted }}>{t('portfolio.cards.noTradesYet')}</p>
                  )}
                </div>
              </div>
            </GlassCard>
          </motion.div>

          {/* Max Drawdown */}
          <motion.div variants={itemVariants}>
            <GlassCard className="overflow-hidden" noPadding>
              <div className="p-5 relative">
                <div
                  className="absolute top-0 right-0 w-24 h-24 blur-2xl"
                  style={{
                    background: `linear-gradient(to bottom left, ${themeColors.cyan}30, transparent)`,
                  }}
                />
                <div className="relative">
                  <div className="flex items-center justify-between mb-3">
                    <GlowIcon icon={Shield} color={themeColors.cyan} size="md" />
                    <Badge variant={portfolioMetrics.maxDrawdownPct < 10 ? "success" : portfolioMetrics.maxDrawdownPct < 20 ? "warning" : "error"}>
                      {portfolioMetrics.maxDrawdownPct < 10 ? t('portfolio.status.safe') : portfolioMetrics.maxDrawdownPct < 20 ? t('portfolio.status.moderate') : t('portfolio.status.high')}
                    </Badge>
                  </div>
                  <p className="text-sm mb-1" style={{ color: themeColors.textMuted }}>{t('portfolio.cards.maxDrawdown')}</p>
                  <p className="text-2xl font-bold" style={{ color: themeColors.loss }}>
                    -{portfolioMetrics.maxDrawdownPct.toFixed(2)}%
                  </p>
                  <p className="text-sm" style={{ color: themeColors.textMuted }}>
                    ${portfolioMetrics.maxDrawdown.toFixed(2)}
                  </p>
                </div>
              </div>
            </GlassCard>
          </motion.div>

          {/* Sharpe Ratio */}
          <motion.div variants={itemVariants}>
            <GlassCard className="overflow-hidden" noPadding>
              <div className="p-5 relative">
                <div
                  className="absolute top-0 right-0 w-24 h-24 blur-2xl"
                  style={{
                    background: `linear-gradient(to bottom left, ${themeColors.purple}30, transparent)`,
                  }}
                />
                <div className="relative">
                  <div className="flex items-center justify-between mb-3">
                    <GlowIcon icon={TrendingUp} color={themeColors.purple} size="md" />
                    <Badge variant={portfolioMetrics.sharpeRatio >= 1.5 ? "success" : portfolioMetrics.sharpeRatio >= 1 ? "warning" : "error"}>
                      {portfolioMetrics.sharpeRatio >= 1.5 ? t('portfolio.status.excellent') : portfolioMetrics.sharpeRatio >= 1 ? t('portfolio.status.good') : t('portfolio.status.low')}
                    </Badge>
                  </div>
                  <p className="text-sm mb-1" style={{ color: themeColors.textMuted }}>{t('portfolio.cards.sharpeRatio')}</p>
                  <p className="text-2xl font-bold" style={{ color: themeColors.textPrimary }}>
                    {portfolioMetrics.sharpeRatio.toFixed(2)}
                  </p>
                  <p className="text-sm" style={{ color: themeColors.textMuted }}>
                    {t('portfolio.cards.riskAdjusted')}
                  </p>
                </div>
              </div>
            </GlassCard>
          </motion.div>
        </div>
      </motion.div>
    </PageWrapper>
  );
};

export default Portfolio;
