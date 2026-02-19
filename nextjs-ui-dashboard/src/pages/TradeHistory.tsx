/**
 * Trade History Page
 *
 * Displays ALL closed trades with filtering, summary stats, and detailed info.
 * Linked from Dashboard's "Xem tất cả" button.
 */

import { useState, useMemo } from "react";
import { useTranslation } from "react-i18next";
import { useThemeColors } from "@/hooks/useThemeColors";
import { usePaperTradingContext } from "@/contexts/PaperTradingContext";
import type { PaperTrade } from "@/contexts/PaperTradingContext";
import { motion } from "framer-motion";
import {
  History,
  TrendingUp,
  Trophy,
  BarChart2,
  ArrowUpRight,
  ArrowDownRight,
  Clock,
  DollarSign,
} from "lucide-react";

import {
  GlassCard,
  SectionHeader,
  PageWrapper,
  StatCard,
  EmptyState,
  Badge,
  containerVariants,
  itemVariants,
} from "@/styles/luxury-design-system";

type FilterTab = "all" | "winning" | "losing";

const formatDuration = (ms?: number): string => {
  if (!ms || ms <= 0) return "-";
  const totalMinutes = Math.floor(ms / 60000);
  if (totalMinutes < 60) return `${totalMinutes}m`;
  const hours = Math.floor(totalMinutes / 60);
  const minutes = totalMinutes % 60;
  if (hours < 24) return minutes > 0 ? `${hours}h ${minutes}m` : `${hours}h`;
  const days = Math.floor(hours / 24);
  const remainingHours = hours % 24;
  return remainingHours > 0 ? `${days}d ${remainingHours}h` : `${days}d`;
};

const formatTimeAgo = (timestamp?: string): string => {
  if (!timestamp) return "-";
  const diff = Math.floor(
    (Date.now() - new Date(timestamp).getTime()) / 60000
  );
  if (diff < 1) return "Just now";
  if (diff < 60) return `${diff}m ago`;
  const hours = Math.floor(diff / 60);
  if (hours < 24) return `${hours}h ago`;
  const days = Math.floor(hours / 24);
  return `${days}d ago`;
};

export default function TradeHistory() {
  const { t } = useTranslation("dashboard");
  const colors = useThemeColors();
  const { closedTrades } = usePaperTradingContext();
  const [activeFilter, setActiveFilter] = useState<FilterTab>("all");

  const filteredTrades = useMemo(() => {
    const trades = closedTrades || [];
    switch (activeFilter) {
      case "winning":
        return trades.filter((tr) => (tr.pnl || 0) >= 0);
      case "losing":
        return trades.filter((tr) => (tr.pnl || 0) < 0);
      default:
        return trades;
    }
  }, [closedTrades, activeFilter]);

  const stats = useMemo(() => {
    const trades = closedTrades || [];
    const total = trades.length;
    const winners = trades.filter((tr) => (tr.pnl || 0) >= 0);
    const winRate = total > 0 ? (winners.length / total) * 100 : 0;
    const totalPnl = trades.reduce((sum, tr) => sum + (tr.pnl || 0), 0);
    const bestTrade =
      trades.length > 0
        ? trades.reduce((best, tr) =>
            (tr.pnl || 0) > (best.pnl || 0) ? tr : best
          )
        : null;
    return { total, winRate, totalPnl, bestTradePnl: bestTrade?.pnl || 0 };
  }, [closedTrades]);

  const filterTabs: { key: FilterTab; label: string }[] = [
    { key: "all", label: t("tradeHistory.filterAll", "All") },
    { key: "winning", label: t("tradeHistory.filterWinning", "Winning") },
    { key: "losing", label: t("tradeHistory.filterLosing", "Losing") },
  ];

  return (
    <PageWrapper>
      <motion.main
        className="container mx-auto px-4 py-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* Header */}
        <motion.div variants={itemVariants} className="mb-8">
          <SectionHeader
            icon={History}
            title={t("tradeHistory.title", "Trade History")}
            subtitle={t(
              "tradeHistory.subtitle",
              "All closed trades and performance"
            )}
          />
        </motion.div>

        {/* Summary Stats */}
        <motion.div
          variants={itemVariants}
          className="grid grid-cols-2 md:grid-cols-4 gap-4 mb-8"
        >
          <StatCard
            icon={BarChart2}
            iconColor={colors.cyan}
            label={t("tradeHistory.totalTrades", "Total Trades")}
            value={stats.total.toString()}
          />
          <StatCard
            icon={TrendingUp}
            iconColor={colors.emerald}
            label={t("tradeHistory.winRate", "Win Rate")}
            value={`${stats.winRate.toFixed(1)}%`}
            valueColor={stats.winRate >= 50 ? colors.emerald : colors.loss}
          />
          <StatCard
            icon={DollarSign}
            iconColor={stats.totalPnl >= 0 ? colors.emerald : colors.loss}
            label={t("tradeHistory.totalPnl", "Total PnL")}
            value={`${stats.totalPnl >= 0 ? "+" : ""}$${stats.totalPnl.toFixed(2)}`}
            valueColor={stats.totalPnl >= 0 ? colors.emerald : colors.loss}
          />
          <StatCard
            icon={Trophy}
            iconColor={colors.gold}
            label={t("tradeHistory.bestTrade", "Best Trade")}
            value={`+$${stats.bestTradePnl.toFixed(2)}`}
            valueColor={colors.emerald}
          />
        </motion.div>

        {/* Filter Tabs */}
        <motion.div variants={itemVariants} className="flex gap-2 mb-6">
          {filterTabs.map((tab) => (
            <button
              key={tab.key}
              onClick={() => setActiveFilter(tab.key)}
              className="px-4 py-2 rounded-xl text-sm font-medium transition-all duration-200"
              style={{
                backgroundColor:
                  activeFilter === tab.key
                    ? colors.bgTertiary
                    : "transparent",
                color:
                  activeFilter === tab.key
                    ? colors.textPrimary
                    : colors.textMuted,
                border:
                  activeFilter === tab.key
                    ? `1px solid ${colors.borderSubtle}`
                    : "1px solid transparent",
              }}
            >
              {tab.label}
            </button>
          ))}
        </motion.div>

        {/* Trade List */}
        <motion.div variants={itemVariants}>
          {filteredTrades.length === 0 ? (
            <EmptyState
              icon={History}
              title={t("tradeHistory.empty", "No trades yet")}
              description={t(
                "tradeHistory.emptyDesc",
                "Closed trades will appear here"
              )}
            />
          ) : (
            <div className="space-y-3">
              {filteredTrades.map((trade: PaperTrade, index: number) => {
                const isLong = trade.trade_type === "Long";
                const isProfitable = (trade.pnl || 0) >= 0;

                return (
                  <motion.div
                    key={trade.id}
                    initial={{ opacity: 0, y: 10 }}
                    animate={{ opacity: 1, y: 0 }}
                    transition={{ delay: Math.min(index * 0.03, 0.5) }}
                  >
                    <GlassCard className="p-4">
                      <div className="flex items-center justify-between">
                        {/* Left: Icon + Trade info */}
                        <div className="flex items-center gap-3 min-w-0">
                          <div
                            className="p-2.5 rounded-xl flex-shrink-0"
                            style={{
                              background: isLong
                                ? "rgba(34, 197, 94, 0.15)"
                                : "rgba(239, 68, 68, 0.15)",
                            }}
                          >
                            {isLong ? (
                              <ArrowUpRight
                                className="w-4 h-4"
                                style={{ color: colors.profit }}
                              />
                            ) : (
                              <ArrowDownRight
                                className="w-4 h-4"
                                style={{ color: colors.loss }}
                              />
                            )}
                          </div>

                          <div className="min-w-0">
                            <div className="flex items-center gap-2 flex-wrap">
                              <span
                                className="font-bold"
                                style={{ color: colors.textPrimary }}
                              >
                                {trade.symbol.replace("USDT", "")}
                              </span>
                              <Badge
                                variant={isLong ? "success" : "error"}
                              >
                                {trade.trade_type}
                              </Badge>
                              {trade.leverage > 1 && (
                                <span
                                  className="text-xs font-bold px-1.5 py-0.5 rounded"
                                  style={{
                                    background: "rgba(0, 217, 255, 0.15)",
                                    color: colors.cyan,
                                    border: "1px solid rgba(0, 217, 255, 0.3)",
                                  }}
                                >
                                  {trade.leverage}x
                                </span>
                              )}
                            </div>
                            <div
                              className="text-xs mt-1 flex items-center gap-1 flex-wrap"
                              style={{ color: colors.textMuted }}
                            >
                              <span>
                                {trade.quantity} @ $
                                {trade.entry_price.toFixed(2)}
                              </span>
                              <span>→</span>
                              <span>
                                ${trade.exit_price?.toFixed(2) || "-"}
                              </span>
                              <span className="mx-1">·</span>
                              <Clock className="w-3 h-3 inline" />
                              <span>
                                {formatDuration(trade.duration_ms)}
                              </span>
                              <span className="mx-1">·</span>
                              <span>
                                {formatTimeAgo(trade.close_time)}
                              </span>
                            </div>
                          </div>
                        </div>

                        {/* Right: PnL */}
                        <div className="text-right flex-shrink-0 ml-3">
                          <div
                            className="text-lg font-bold"
                            style={{
                              color: isProfitable
                                ? colors.profit
                                : colors.loss,
                            }}
                          >
                            {isProfitable ? "+" : ""}$
                            {(trade.pnl || 0).toFixed(2)}
                          </div>
                          <div
                            className="text-xs"
                            style={{
                              color: isProfitable
                                ? colors.profit
                                : colors.loss,
                            }}
                          >
                            {isProfitable ? "+" : ""}
                            {trade.pnl_percentage?.toFixed(2) || "0.00"}%
                          </div>
                        </div>
                      </div>
                    </GlassCard>
                  </motion.div>
                );
              })}
            </div>
          )}
        </motion.div>
      </motion.main>
    </PageWrapper>
  );
}
