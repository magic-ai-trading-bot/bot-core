/**
 * AI Signals Page
 *
 * Premium AI-powered trading signals dashboard with real-time updates,
 * confidence visualizations, and comprehensive signal history.
 *
 * @spec:FR-AI-001 - AI Trading Signals Display
 * @ref:docs/features/ai-integration.md
 */

import { useState, useMemo, useEffect, useRef } from "react";
import { useTranslation } from "react-i18next";
import { motion, AnimatePresence } from "framer-motion";
import {
  Brain,
  Zap,
  Target,
  TrendingUp,
  TrendingDown,
  Minus,
  Clock,
  CheckCircle2,
  XCircle,
  ChevronDown,
  ChevronUp,
  RefreshCw,
  BarChart3,
  Eye,
  ArrowUpRight,
  ArrowDownRight,
  Timer,
  Shield,
} from "lucide-react";
import { cn } from "@/lib/utils";
import { useWebSocket } from "@/hooks/useWebSocket";
import { AISignal } from "@/services/api";
import { useThemeColors } from "@/hooks/useThemeColors";
import {
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  SectionHeader,
  PageWrapper,
} from "@/styles/luxury-design-system";

// ============================================================================
// TYPES & INTERFACES
// ============================================================================

interface SignalWithMeta extends AISignal {
  id: string;
  entry_price?: number;
  target_price?: number;
  stop_loss?: number;
  outcome?: "win" | "loss" | "pending";
  actual_pnl?: number;
  pnl_percentage?: number;
  exit_price?: number;
  close_reason?: string;
  closed_at?: string;
}

// API Response types for signals history
interface SignalHistoryStats {
  total: number;
  wins: number;
  losses: number;
  pending: number;
  win_rate: number;
  total_pnl: number;
}

interface SignalsHistoryResponse {
  signals: SignalWithMeta[];
  stats: SignalHistoryStats;
}

// WebSocket event for signal outcome updates
interface SignalOutcomeEvent {
  signal_id: string;
  outcome: "win" | "loss";
  actual_pnl: number;
  pnl_percentage: number;
  exit_price: number;
  close_reason: string;
  trade_id: string;
}

// API Base URL
const RUST_API_URL = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";


// ============================================================================
// NO MOCK DATA - Only real data from WebSocket/API
// ============================================================================

// ============================================================================
// UTILITY FUNCTIONS
// ============================================================================

function formatTimeAgo(timestamp: string): string {
  const now = new Date();
  const date = new Date(timestamp);
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);

  if (diffMins < 1) return "Just now";
  if (diffMins < 60) return `${diffMins}m ago`;
  const diffHours = Math.floor(diffMins / 60);
  if (diffHours < 24) return `${diffHours}h ago`;
  return `${Math.floor(diffHours / 24)}d ago`;
}

function formatPrice(price: number, symbol: string): string {
  if (symbol.includes("BTC")) return `$${price.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  if (symbol.includes("ETH")) return `$${price.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
  if (price < 1) return `$${price.toFixed(4)}`;
  if (price < 10) return `$${price.toFixed(3)}`;
  return `$${price.toLocaleString("en-US", { minimumFractionDigits: 2, maximumFractionDigits: 2 })}`;
}

// ============================================================================
// COMPONENTS
// ============================================================================

// Neural Network Background Animation
const NeuralNetworkBg = () => {
  const themeColors = useThemeColors();
  return (
  <div className="absolute inset-0 overflow-hidden pointer-events-none">
    {/* Gradient orbs */}
    <div
      className="absolute top-0 left-1/4 w-[600px] h-[600px] blur-[120px] animate-pulse-slow"
      style={{
        background: `radial-gradient(circle, ${themeColors.purple}20, ${themeColors.cyan}10, transparent)`,
      }}
    />
    <div
      className="absolute bottom-0 right-1/4 w-[400px] h-[400px] blur-[100px] animate-pulse-slow"
      style={{
        background: `radial-gradient(circle, ${themeColors.cyan}15, ${themeColors.purple}10, transparent)`,
        animationDelay: "1s",
      }}
    />

    {/* Neural network SVG pattern */}
    <svg className="absolute inset-0 w-full h-full opacity-[0.03]" viewBox="0 0 800 600">
      <defs>
        <linearGradient id="nodeGrad" x1="0%" y1="0%" x2="100%" y2="100%">
          <stop offset="0%" stopColor={themeColors.purple} />
          <stop offset="100%" stopColor={themeColors.cyan} />
        </linearGradient>
      </defs>
      {/* Neural connections */}
      {[...Array(20)].map((_, i) => (
        <line
          key={`line-${i}`}
          x1={Math.random() * 800}
          y1={Math.random() * 600}
          x2={Math.random() * 800}
          y2={Math.random() * 600}
          stroke="url(#nodeGrad)"
          strokeWidth="0.5"
          opacity="0.3"
        />
      ))}
      {/* Neural nodes */}
      {[...Array(15)].map((_, i) => (
        <circle
          key={`node-${i}`}
          cx={100 + (i % 5) * 150}
          cy={100 + Math.floor(i / 5) * 150}
          r="4"
          fill="url(#nodeGrad)"
          opacity="0.5"
        />
      ))}
    </svg>
  </div>
  );
};

// Pulse Animation Ring
const PulseRing = ({ color, size = 80 }: { color?: string; size?: number }) => {
  const themeColors = useThemeColors();
  const ringColor = color || themeColors.purple;
  return (
    <div className="relative" style={{ width: size, height: size }}>
      <motion.div
        className="absolute inset-0 rounded-full"
        style={{ backgroundColor: ringColor, opacity: 0.4 }}
        animate={{ scale: [1, 1.5, 1], opacity: [0.4, 0, 0.4] }}
        transition={{ duration: 2, repeat: Infinity, ease: "easeInOut" }}
      />
      <motion.div
        className="absolute inset-0 rounded-full"
        style={{ backgroundColor: ringColor, opacity: 0.3 }}
        animate={{ scale: [1, 1.8, 1], opacity: [0.3, 0, 0.3] }}
        transition={{ duration: 2, repeat: Infinity, ease: "easeInOut", delay: 0.5 }}
      />
    </div>
  );
};

// Confidence Meter Component
const ConfidenceMeter = ({ value, size = "md" }: { value: number; size?: "sm" | "md" | "lg" }) => {
  const themeColors = useThemeColors();
  const percentage = Math.round(value * 100);
  const getColor = () => {
    if (percentage >= 85) return { main: themeColors.emerald, glow: `${themeColors.emerald}80` };
    if (percentage >= 70) return { main: themeColors.cyan, glow: `${themeColors.cyan}80` };
    if (percentage >= 55) return { main: themeColors.amber, glow: `${themeColors.amber}80` };
    return { main: themeColors.loss, glow: `${themeColors.loss}80` };
  };

  const colors = getColor();
  const dimensions = {
    sm: { width: 60, height: 6, text: "text-xs" },
    md: { width: 100, height: 8, text: "text-sm" },
    lg: { width: 140, height: 10, text: "text-base" },
  };
  const dim = dimensions[size];

  return (
    <div className="flex items-center gap-2">
      <div
        className="relative rounded-full overflow-hidden"
        style={{
          width: dim.width,
          height: dim.height,
          backgroundColor: themeColors.bgTertiary,
        }}
      >
        <motion.div
          className="absolute inset-y-0 left-0 rounded-full"
          style={{
            backgroundColor: colors.main,
            boxShadow: `0 0 12px ${colors.glow}`,
          }}
          initial={{ width: 0 }}
          animate={{ width: `${percentage}%` }}
          transition={{ duration: 0.8, ease: "easeOut" }}
        />
      </div>
      <span className={cn("font-bold", dim.text)} style={{ color: colors.main }}>
        {percentage}%
      </span>
    </div>
  );
};

// Signal Direction Badge
const SignalBadge = ({ signal, size = "md" }: { signal: "long" | "short" | "neutral"; size?: "sm" | "md" | "lg" }) => {
  const config = {
    long: {
      label: "BUY",
      icon: TrendingUp,
      variant: "success" as const,
    },
    short: {
      label: "SELL",
      icon: TrendingDown,
      variant: "error" as const,
    },
    neutral: {
      label: "HOLD",
      icon: Minus,
      variant: "default" as const,
    },
  };

  const { label, icon: Icon, variant } = config[signal];
  const iconSize = { sm: 12, md: 14, lg: 16 };

  return (
    <Badge variant={variant} size={size === "lg" ? "md" : "sm"} glow>
      <Icon size={iconSize[size]} className="mr-1" />
      {label}
    </Badge>
  );
};

// AI Status Hero Section
const AIStatusHero = ({ isActive, accuracy, lastUpdate }: { isActive: boolean; accuracy: number; lastUpdate: string }) => {
  const { t } = useTranslation('dashboard');
  const themeColors = useThemeColors();
  const statusColor = isActive ? themeColors.emerald : themeColors.loss;

  return (
    <GlassCard hoverable glowColor={`0 8px 32px ${themeColors.purple}40`}>
      {/* Background effects */}
      <div className="absolute inset-0 overflow-hidden pointer-events-none">
        <div
          className="absolute top-0 right-0 w-[300px] h-[300px] blur-[80px]"
          style={{
            background: `radial-gradient(circle, ${themeColors.purple}20, ${themeColors.cyan}10, transparent)`,
          }}
        />
        <div
          className="absolute bottom-0 left-0 w-[200px] h-[200px] blur-[60px]"
          style={{
            background: `radial-gradient(circle, ${themeColors.cyan}15, transparent)`,
          }}
        />
      </div>

      <div className="relative flex flex-col md:flex-row items-center justify-between gap-6">
        {/* Left: Status indicator */}
        <div className="flex items-center gap-6">
          <div className="relative">
            <PulseRing color={statusColor} size={80} />
            <div
              className="absolute inset-0 flex items-center justify-center rounded-full"
              style={{
                background: `linear-gradient(135deg, ${statusColor}20, ${statusColor}05)`,
                border: `2px solid ${statusColor}40`,
              }}
            >
              <Brain className="w-8 h-8" style={{ color: statusColor }} />
            </div>
          </div>
          <div>
            <div className="flex items-center gap-3 mb-2">
              <GradientText className="text-2xl md:text-3xl font-bold">{t('trading:aiEngine.title')}</GradientText>
              <Badge variant={isActive ? "success" : "error"} glow>
                {isActive ? t('aiSignalsPage.engine.active') : t('aiSignalsPage.engine.offline')}
              </Badge>
            </div>
            <p className="text-sm md:text-base" style={{ color: themeColors.textMuted }}>
              {t('aiSignalsPage.engine.description')}
            </p>
          </div>
        </div>

        {/* Right: Stats */}
        <div className="flex items-center gap-6 md:gap-8">
          <div className="text-center">
            <GradientText className="text-3xl md:text-4xl font-bold">{accuracy}%</GradientText>
            <div className="text-xs mt-1" style={{ color: themeColors.textMuted }}>
              {t('aiSignalsPage.accuracy.label')}
            </div>
          </div>
          <div
            className="w-px h-12"
            style={{
              background: `linear-gradient(to bottom, transparent, ${themeColors.purple}30, transparent)`,
            }}
          />
          <div className="text-center">
            <div className="flex items-center gap-1" style={{ color: themeColors.cyan }}>
              <Clock className="w-4 h-4" />
              <span className="text-lg font-semibold">{lastUpdate}</span>
            </div>
            <div className="text-xs mt-1" style={{ color: themeColors.textMuted }}>
              {t('aiSignalsPage.lastUpdate')}
            </div>
          </div>
        </div>
      </div>
    </GlassCard>
  );
};

// Signal Card Component
const SignalCard = ({ signal, index, onExpand }: { signal: SignalWithMeta; index: number; onExpand: () => void }) => {
  const themeColors = useThemeColors();
  const [isExpanded, setIsExpanded] = useState(false);

  const signalColor =
    signal.signal === "long"
      ? themeColors.emerald
      : signal.signal === "short"
        ? themeColors.loss
        : themeColors.textMuted;

  return (
    <motion.div
      initial={{ opacity: 0, x: -20 }}
      animate={{ opacity: 1, x: 0 }}
      transition={{ delay: index * 0.1 }}
    >
      <GlassCard
        noPadding
        hoverable
        glowColor={`0 8px 32px ${signalColor}40`}
        className="relative border"
        style={{ borderColor: `${signalColor}20` }}
      >
        {/* Top accent line */}
        <div
          className="absolute top-0 left-0 right-0 h-0.5"
          style={{
            background: `linear-gradient(90deg, transparent, ${signalColor}, transparent)`,
          }}
        />

      <div className="p-4 md:p-5">
        {/* Header Row */}
        <div className="flex items-center justify-between mb-4">
          <div className="flex items-center gap-3">
            <div className="relative">
              <GlowIcon
                icon={
                  signal.signal === "long"
                    ? ArrowUpRight
                    : signal.signal === "short"
                      ? ArrowDownRight
                      : Minus
                }
                color={signalColor}
                size="lg"
              />
              {/* Live indicator */}
              <div
                className="absolute -top-1 -right-1 w-3 h-3 rounded-full animate-pulse"
                style={{ backgroundColor: themeColors.emerald }}
              />
            </div>
            <div>
              <div className="flex items-center gap-2">
                <span className="font-bold text-lg" style={{ color: themeColors.textPrimary }}>
                  {signal.symbol.replace("USDT", "")}
                </span>
                <Badge variant="default" size="sm">
                  {signal.timeframe}
                </Badge>
              </div>
              <div className="flex items-center gap-2 mt-0.5">
                <span className="text-xs" style={{ color: themeColors.textMuted }}>
                  {signal.model_type.toUpperCase()}
                </span>
                <span className="text-xs" style={{ color: themeColors.textMuted }}>
                  |
                </span>
                <span className="text-xs" style={{ color: themeColors.textMuted }}>
                  {formatTimeAgo(signal.timestamp)}
                </span>
              </div>
            </div>
          </div>
          <SignalBadge signal={signal.signal} />
        </div>

        {/* Confidence Row */}
        <div className="flex items-center justify-between mb-4">
          <div>
            <span className="text-xs block mb-1" style={{ color: themeColors.textMuted }}>
              Confidence
            </span>
            <ConfidenceMeter value={signal.confidence} size="md" />
          </div>
          {signal.entry_price && (
            <div className="text-right">
              <span className="text-xs block mb-1" style={{ color: themeColors.textMuted }}>
                Entry Price
              </span>
              <span className="font-mono font-semibold" style={{ color: themeColors.textPrimary }}>
                {formatPrice(signal.entry_price, signal.symbol)}
              </span>
            </div>
          )}
        </div>

        {/* Price Targets */}
        {(signal.target_price || signal.stop_loss) && (
          <div className="flex items-center gap-4 mb-4">
            {signal.target_price && (
              <div
                className="flex-1 p-2 rounded-lg border"
                style={{
                  backgroundColor: `${themeColors.emerald}10`,
                  borderColor: `${themeColors.emerald}20`,
                }}
              >
                <div className="flex items-center gap-1 text-xs mb-1" style={{ color: themeColors.emerald }}>
                  <Target className="w-3 h-3" />
                  Target
                </div>
                <span className="font-mono font-semibold" style={{ color: themeColors.emerald }}>
                  {formatPrice(signal.target_price, signal.symbol)}
                </span>
              </div>
            )}
            {signal.stop_loss && (
              <div
                className="flex-1 p-2 rounded-lg border"
                style={{
                  backgroundColor: `${themeColors.loss}10`,
                  borderColor: `${themeColors.loss}20`,
                }}
              >
                <div className="flex items-center gap-1 text-xs mb-1" style={{ color: themeColors.loss }}>
                  <Shield className="w-3 h-3" />
                  Stop Loss
                </div>
                <span className="font-mono font-semibold" style={{ color: themeColors.loss }}>
                  {formatPrice(signal.stop_loss, signal.symbol)}
                </span>
              </div>
            )}
          </div>
        )}

        {/* Expandable Analysis */}
        <button
          onClick={() => setIsExpanded(!isExpanded)}
          className="w-full flex items-center justify-between p-2 rounded-lg transition-colors text-sm"
          style={{
            backgroundColor: themeColors.bgSecondary,
            color: themeColors.textMuted,
          }}
        >
          <span className="flex items-center gap-2">
            <Eye className="w-4 h-4" />
            View Analysis
          </span>
          {isExpanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
        </button>

        <AnimatePresence>
          {isExpanded && (
            <motion.div
              initial={{ height: 0, opacity: 0 }}
              animate={{ height: "auto", opacity: 1 }}
              exit={{ height: 0, opacity: 0 }}
              transition={{ duration: 0.2 }}
              className="overflow-hidden"
            >
              <div className="pt-4 space-y-3">
                {signal.reasoning && (
                  <div
                    className="p-3 rounded-lg"
                    style={{ backgroundColor: themeColors.bgSecondary }}
                  >
                    <div className="text-xs font-semibold mb-1" style={{ color: themeColors.purple }}>
                      AI Reasoning
                    </div>
                    <p className="text-sm leading-relaxed" style={{ color: themeColors.textSecondary }}>
                      {signal.reasoning}
                    </p>
                  </div>
                )}
                {signal.strategy_scores && (
                  <div
                    className="p-3 rounded-lg"
                    style={{ backgroundColor: themeColors.bgSecondary }}
                  >
                    <div className="text-xs font-semibold mb-2" style={{ color: themeColors.cyan }}>
                      Strategy Scores
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      {Object.entries(signal.strategy_scores).map(([strategy, score]) => (
                        <div key={strategy} className="flex items-center justify-between">
                          <span className="text-xs capitalize" style={{ color: themeColors.textMuted }}>
                            {strategy}
                          </span>
                          <ConfidenceMeter value={score} size="sm" />
                        </div>
                      ))}
                    </div>
                  </div>
                )}
              </div>
            </motion.div>
          )}
        </AnimatePresence>
      </div>
      </GlassCard>
    </motion.div>
  );
};

// Signal History Row
const HistoryRow = ({ signal, index }: { signal: SignalWithMeta; index: number }) => {
  const themeColors = useThemeColors();
  const outcomeColor =
    signal.outcome === "win"
      ? themeColors.emerald
      : signal.outcome === "loss"
        ? themeColors.loss
        : themeColors.amber;

  return (
    <motion.div
      initial={{ opacity: 0, y: 10 }}
      animate={{ opacity: 1, y: 0 }}
      transition={{ delay: index * 0.05 }}
      className="p-3 rounded-lg transition-colors"
      style={{
        backgroundColor: themeColors.bgSecondary,
      }}
    >
      <div className="flex items-center justify-between">
        <div className="flex items-center gap-3">
          <GlowIcon
            icon={
              signal.outcome === "win"
                ? CheckCircle2
                : signal.outcome === "loss"
                  ? XCircle
                  : Timer
            }
            color={outcomeColor}
            size="md"
          />
          <div>
            <div className="flex items-center gap-2">
              <span className="font-semibold" style={{ color: themeColors.textPrimary }}>
                {signal.symbol.replace("USDT", "")}
              </span>
              <SignalBadge signal={signal.signal} size="sm" />
            </div>
            <span className="text-xs" style={{ color: themeColors.textMuted }}>
              {formatTimeAgo(signal.timestamp)} | {signal.model_type.toUpperCase()}
            </span>
          </div>
        </div>
        <div className="text-right">
          <div
            className="font-mono font-bold"
            style={{
              color:
                signal.actual_pnl && signal.actual_pnl > 0
                  ? themeColors.emerald
                  : signal.actual_pnl && signal.actual_pnl < 0
                    ? themeColors.loss
                    : themeColors.amber,
            }}
          >
            {signal.actual_pnl
              ? `${signal.actual_pnl > 0 ? "+" : ""}${signal.actual_pnl.toFixed(2)}%`
              : "Pending"}
          </div>
          <span className="text-xs" style={{ color: themeColors.textMuted }}>
            Conf: {Math.round(signal.confidence * 100)}%
          </span>
        </div>
      </div>
    </motion.div>
  );
};

// ============================================================================
// MAIN COMPONENT
// ============================================================================

const AISignals = () => {
  const { t } = useTranslation('dashboard');
  const { state: wsState } = useWebSocket();
  const themeColors = useThemeColors();
  const [selectedFilter, setSelectedFilter] = useState<"all" | "win" | "loss">("all");
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [historySignals, setHistorySignals] = useState<SignalWithMeta[]>([]);
  const [apiStats, setApiStats] = useState<SignalHistoryStats | null>(null);
  const [isLoadingHistory, setIsLoadingHistory] = useState(true);

  // Track previous signals to detect changes and move old ones to history
  const prevSignalsRef = useRef<Map<string, SignalWithMeta>>(new Map());

  // Fetch signals history from backend API
  // @spec:FR-AI-012 - Signal Outcome Tracking
  const fetchSignalsHistory = async () => {
    try {
      const response = await fetch(`${RUST_API_URL}/api/paper-trading/signals-history?limit=100`);
      if (!response.ok) {
        throw new Error(`HTTP error: ${response.status}`);
      }
      const responseData = await response.json();

      // Handle both wrapped and unwrapped API responses
      const data = responseData.data || responseData;

      // Transform API signals to SignalWithMeta format
      // API returns: signal_type, signal_id, confidence, reasoning, etc.
      // Frontend expects: signal, id, model_type, timeframe, etc.
      const transformedSignals: SignalWithMeta[] = (data.signals || []).map((s: {
        signal_id?: string;
        signal_type?: string;
        signal?: string;
        symbol: string;
        confidence: number;
        reasoning?: string;
        entry_price?: number;
        timestamp?: string;
        created_at?: string;
        outcome?: string;
        actual_pnl?: number;
        pnl_percentage?: number;
        exit_price?: number;
        close_reason?: string;
        trend_direction?: string;
        trend_strength?: number;
      }) => {
        // Map signal_type to signal (API uses "Long"/"Short"/"Neutral", frontend uses "long"/"short"/"neutral")
        const signalType = (s.signal_type || s.signal || "neutral").toLowerCase() as "long" | "short" | "neutral";

        return {
          id: s.signal_id || `api-${s.symbol}-${s.timestamp || s.created_at}`,
          symbol: s.symbol,
          signal: signalType,
          confidence: s.confidence,
          probability: s.confidence,
          reasoning: s.reasoning,
          entry_price: s.entry_price,
          timestamp: s.timestamp || s.created_at || new Date().toISOString(),
          // API doesn't provide model_type/timeframe, derive from reasoning or use defaults
          model_type: s.reasoning?.includes("4H") ? "multi-tf" : "gpt-4",
          timeframe: s.reasoning?.includes("4H") ? "4H" : "1H",
          outcome: (s.outcome as "win" | "loss" | "pending") || "pending",
          actual_pnl: s.actual_pnl || s.pnl_percentage,
          pnl_percentage: s.pnl_percentage,
          exit_price: s.exit_price,
          close_reason: s.close_reason,
          // Map additional fields
          strategy_scores: s.trend_direction ? {
            trend: s.trend_strength || 0.5,
          } : undefined,
        };
      });

      setHistorySignals(transformedSignals);
      setApiStats(data.stats || { total: 0, wins: 0, losses: 0, pending: 0, win_rate: 0, total_pnl: 0 });
    } catch (error) {
      console.error("Failed to fetch signals history:", error);
    } finally {
      setIsLoadingHistory(false);
    }
  };

  // Fetch signals history on mount
  useEffect(() => {
    fetchSignalsHistory();
  }, []);

  // Listen for signal outcome updates via WebSocket
  // @spec:FR-AI-012 - Signal Outcome Tracking
  useEffect(() => {
    const lastMessage = wsState.lastMessage;
    if (!lastMessage) return;

    // Check for signal_outcome_updated event from paper trading WebSocket
    // The event comes from PaperTradingEvent which uses JSON data wrapper
    if (lastMessage.type === "PaperTradingEvent" ||
        (lastMessage as unknown as { event_type?: string }).event_type === "signal_outcome_updated") {
      const eventData = (lastMessage as unknown as { data?: SignalOutcomeEvent }).data;
      if (eventData?.signal_id) {
        // Update the signal in history with the outcome
        setHistorySignals(prev => prev.map(signal => {
          // Match by signal_id (checking both id field and embedded signal_id)
          const signalId = signal.id || (signal as unknown as { signal_id?: string }).signal_id;
          if (signalId === eventData.signal_id || signal.id?.includes(eventData.signal_id)) {
            return {
              ...signal,
              outcome: eventData.outcome,
              actual_pnl: eventData.pnl_percentage, // Use percentage for display
              pnl_percentage: eventData.pnl_percentage,
              exit_price: eventData.exit_price,
              close_reason: eventData.close_reason,
            };
          }
          return signal;
        }));

        // Update stats (increment win/loss counter)
        setApiStats(prev => {
          if (!prev) return prev;
          const newStats = { ...prev };
          if (eventData.outcome === "win") {
            newStats.wins += 1;
            newStats.pending = Math.max(0, newStats.pending - 1);
          } else if (eventData.outcome === "loss") {
            newStats.losses += 1;
            newStats.pending = Math.max(0, newStats.pending - 1);
          }
          newStats.total_pnl += eventData.actual_pnl;
          const totalDecided = newStats.wins + newStats.losses;
          newStats.win_rate = totalDecided > 0 ? (newStats.wins / totalDecided) * 100 : 0;
          return newStats;
        });
      }
    }
  }, [wsState.lastMessage]);

  // Process WebSocket signals: dedupe by symbol (keep latest) and move old to history
  const liveSignals = useMemo(() => {
    const signalMap = new Map<string, SignalWithMeta>();

    // Process signals - keep only the latest for each symbol
    wsState.aiSignals.forEach((s, _i) => {
      const signal: SignalWithMeta = {
        ...s,
        id: `ws-${s.symbol}-${s.timestamp}`,
        entry_price: s.entry_price,
        target_price: s.target_price,
        stop_loss: s.stop_loss,
        outcome: "pending" as const,
      };

      // If we already have a signal for this symbol, check which is newer
      const existing = signalMap.get(s.symbol);
      if (!existing || new Date(s.timestamp) > new Date(existing.timestamp)) {
        signalMap.set(s.symbol, signal);
      }
    });

    return Array.from(signalMap.values());
  }, [wsState.aiSignals]);

  // Move old signals to history when new ones arrive for the same symbol
  useEffect(() => {
    const currentSignalMap = new Map<string, SignalWithMeta>();
    liveSignals.forEach(s => currentSignalMap.set(s.symbol, s));

    // Find signals that were replaced (same symbol but different timestamp)
    const signalsToMoveToHistory: SignalWithMeta[] = [];

    prevSignalsRef.current.forEach((prevSignal, symbol) => {
      const currentSignal = currentSignalMap.get(symbol);
      if (currentSignal && currentSignal.timestamp !== prevSignal.timestamp) {
        // Signal was replaced - move old one to history
        signalsToMoveToHistory.push({
          ...prevSignal,
          outcome: "pending" as const, // Will be determined by backend later
        });
      }
    });

    if (signalsToMoveToHistory.length > 0) {
      setHistorySignals(prev => {
        // Avoid duplicates by checking id
        const existingIds = new Set(prev.map(s => s.id));
        const newSignals = signalsToMoveToHistory.filter(s => !existingIds.has(s.id));
        return [...newSignals, ...prev].slice(0, 50); // Keep last 50
      });
    }

    // Update ref with current signals
    prevSignalsRef.current = currentSignalMap;
  }, [liveSignals]);

  const filteredHistory = useMemo(() => {
    if (selectedFilter === "all") return historySignals;
    return historySignals.filter((s) => s.outcome === selectedFilter);
  }, [selectedFilter, historySignals]);

  // Calculate stats from API response or fallback to local calculation
  // @spec:FR-AI-012 - Signal Outcome Tracking
  const stats = useMemo(() => {
    // Use API stats if available (more accurate, includes all historical data)
    if (apiStats) {
      return {
        wins: apiStats.wins,
        losses: apiStats.losses,
        winRate: Math.round(apiStats.win_rate),
        totalPnl: apiStats.total_pnl,
        pending: apiStats.pending,
      };
    }

    // Fallback to local calculation from history signals
    const wins = historySignals.filter((s) => s.outcome === "win").length;
    const losses = historySignals.filter((s) => s.outcome === "loss").length;
    const total = wins + losses;
    const winRate = total > 0 ? Math.round((wins / total) * 100) : 0;
    const totalPnl = historySignals.reduce((acc, s) => acc + (s.actual_pnl || 0), 0);
    return { wins, losses, winRate, totalPnl, pending: historySignals.length - total };
  }, [apiStats, historySignals]);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await fetchSignalsHistory();
    setIsRefreshing(false);
  };

  return (
    <PageWrapper>
      {/* Background */}
      <NeuralNetworkBg />

      <div className="relative space-y-6">
        {/* Page Header */}
        <div className="flex items-center justify-between">
          <div>
            <GradientText className="text-3xl font-bold tracking-tight">{t('aiSignalsPage.title')}</GradientText>
            <p className="mt-1" style={{ color: themeColors.textMuted }}>
              {t('aiSignalsPage.subtitle')}
            </p>
          </div>
          <PremiumButton
            variant="secondary"
            onClick={handleRefresh}
            disabled={isRefreshing}
            loading={isRefreshing}
          >
            <RefreshCw className="w-4 h-4" />
            {t('tradeAnalyses.refresh')}
          </PremiumButton>
        </div>

        {/* AI Status Hero */}
        <AIStatusHero
          isActive={wsState.isConnected || true}
          accuracy={78}
          lastUpdate={t('aiSignalsPage.justNow')}
        />

        {/* Main Content Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
          {/* Live Signals - 2 columns */}
          <div className="lg:col-span-2 space-y-4">
            <SectionHeader
              title={t('aiSignalsPage.signals.live')}
              icon={Zap}
              action={
                <Badge variant="success" glow>
                  {liveSignals.length} {t('aiSignalsPage.signals.active')}
                </Badge>
              }
            />

            {liveSignals.length > 0 ? (
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {liveSignals.map((signal, index) => (
                  <SignalCard
                    key={signal.id}
                    signal={signal}
                    index={index}
                    onExpand={() => {}}
                  />
                ))}
              </div>
            ) : (
              <GlassCard className="text-center py-12">
                <GlowIcon icon={Zap} color={themeColors.cyan} size="lg" className="mx-auto mb-4" />
                <GradientText className="text-lg font-semibold mb-2">
                  {t('aiSignalsPage.signals.waiting')}
                </GradientText>
                <p className="text-sm" style={{ color: themeColors.textMuted }}>
                  {t('aiSignalsPage.signals.waitingDesc')}
                </p>
              </GlassCard>
            )}
          </div>

          {/* Signal History - 1 column */}
          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <SectionHeader title={t('aiSignalsPage.history.title')} icon={BarChart3} />
              <div
                className="flex items-center gap-1 p-1 rounded-lg"
                style={{ backgroundColor: themeColors.bgSecondary }}
              >
                {(["all", "win", "loss"] as const).map((filter) => (
                  <button
                    key={filter}
                    onClick={() => setSelectedFilter(filter)}
                    className="px-3 py-1 text-xs rounded-md transition-colors"
                    style={{
                      backgroundColor:
                        selectedFilter === filter ? themeColors.purple : "transparent",
                      color:
                        selectedFilter === filter
                          ? themeColors.textPrimary
                          : themeColors.textMuted,
                    }}
                  >
                    {t(`aiSignalsPage.history.${filter}`)}
                  </button>
                ))}
              </div>
            </div>

            {/* Stats Summary */}
            <div className="grid grid-cols-2 gap-2">
              <GlassCard noPadding className="text-center p-3">
                <div className="text-lg font-bold" style={{ color: themeColors.emerald }}>
                  {isLoadingHistory ? "-" : stats.wins}
                </div>
                <div className="text-xs" style={{ color: themeColors.textMuted }}>
                  {t('aiSignalsPage.stats.wins')}
                </div>
              </GlassCard>
              <GlassCard noPadding className="text-center p-3">
                <div className="text-lg font-bold" style={{ color: themeColors.loss }}>
                  {isLoadingHistory ? "-" : stats.losses}
                </div>
                <div className="text-xs" style={{ color: themeColors.textMuted }}>
                  {t('aiSignalsPage.stats.losses')}
                </div>
              </GlassCard>
              <GlassCard noPadding className="text-center p-3">
                <GradientText className="text-lg font-bold">
                  {isLoadingHistory ? "-" : `${stats.winRate}%`}
                </GradientText>
                <div className="text-xs" style={{ color: themeColors.textMuted }}>
                  {t('aiSignalsPage.stats.winRate')}
                </div>
              </GlassCard>
              <GlassCard noPadding className="text-center p-3">
                <div
                  className="text-lg font-bold font-mono"
                  style={{
                    color: isLoadingHistory
                      ? themeColors.textMuted
                      : stats.totalPnl >= 0
                        ? themeColors.emerald
                        : themeColors.loss,
                  }}
                >
                  {isLoadingHistory
                    ? "-"
                    : `${stats.totalPnl >= 0 ? "+" : ""}${stats.totalPnl.toFixed(2)}`}
                </div>
                <div className="text-xs" style={{ color: themeColors.textMuted }}>
                  {t('aiSignalsPage.stats.totalPnl')} (USDT)
                </div>
              </GlassCard>
            </div>

            {/* History List */}
            <div className="space-y-2 max-h-[500px] overflow-y-auto custom-scrollbar">
              {filteredHistory.length > 0 ? (
                filteredHistory.map((signal, index) => (
                  <HistoryRow key={signal.id} signal={signal} index={index} />
                ))
              ) : (
                <div
                  className="text-center py-8 rounded-lg"
                  style={{ backgroundColor: themeColors.bgSecondary }}
                >
                  <BarChart3 className="w-8 h-8 mx-auto mb-2" style={{ color: themeColors.textMuted }} />
                  <p className="text-sm" style={{ color: themeColors.textMuted }}>
                    {t('aiSignalsPage.history.noHistory')}
                  </p>
                </div>
              )}
            </div>
          </div>
        </div>

      </div>

      {/* Custom scrollbar styles */}
      <style>{`
        .custom-scrollbar::-webkit-scrollbar {
          width: 6px;
        }
        .custom-scrollbar::-webkit-scrollbar-track {
          background: ${themeColors.bgSecondary};
          border-radius: 3px;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb {
          background: ${themeColors.purple}50;
          border-radius: 3px;
        }
        .custom-scrollbar::-webkit-scrollbar-thumb:hover {
          background: ${themeColors.purple}80;
        }
      `}</style>
    </PageWrapper>
  );
};

export default AISignals;
