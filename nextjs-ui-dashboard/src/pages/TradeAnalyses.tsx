/**
 * Trade Analyses Page
 * @spec:FR-ASYNC-011 - GPT-4 Individual Trade Analysis
 *
 * Displays GPT-4 analyses for paper trading trades.
 * Each failed/losing trade is analyzed by GPT-4 to provide insights.
 */

import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import React, { useState, useEffect, useCallback } from "react";
import { toast } from "sonner";
import { motion } from "framer-motion";
import {
  TrendingUp,
  TrendingDown,
  AlertCircle,
  RefreshCw,
  Brain,
  Target,
  Lightbulb,
  History,
  CheckCircle,
  XCircle,
  Clock,
  DollarSign,
  BarChart2,
  Settings2,
} from "lucide-react";

// Luxury OLED Design System
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
  SectionHeader,
  PageWrapper,
  StatCard,
  LoadingSpinner,
  EmptyState,
  Divider,
  containerVariants,
  itemVariants,
} from '@/styles/luxury-design-system';

// API Base URLs
const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";
const PYTHON_API_BASE = import.meta.env.VITE_PYTHON_AI_URL || "http://localhost:8000";

// Types for API responses
interface ApiResponse<T> {
  success: boolean;
  data?: T;
  error?: string;
  timestamp: string;
}

// BSON Date format from MongoDB
interface BsonDate {
  $date: {
    $numberLong: string;
  };
}

// Trade Analysis from GPT-4
interface TradeAnalysis {
  id?: string;
  _id?: { $oid: string };
  trade_id: string;
  created_at: string | BsonDate;
  is_winning: boolean;
  pnl_usdt: number;
  pnl_percentage: number;
  symbol?: string;
  side?: string;
  entry_price?: number;
  exit_price?: number;
  close_reason?: string;
  analysis: GPT4Analysis;
  trade_data?: Record<string, unknown>;
}

// Entry/Exit analysis can be string (old format) or object (new GPT-4 format)
interface AnalysisDetail {
  quality?: string;
  reasoning?: string;
  signals_valid?: boolean;
  better_exit_point?: string;
}

interface GPT4Recommendations {
  config_changes?: Record<string, string>;
  strategy_improvements?: string[];
  risk_management?: string;
}

interface GPT4Analysis {
  // New GPT-4 format
  trade_verdict?: string;
  entry_analysis?: string | AnalysisDetail;
  exit_analysis?: string | AnalysisDetail;
  key_factors?: string[];
  lessons_learned?: string[];
  recommendations?: GPT4Recommendations;
  confidence?: number;
  summary?: string;
  // Old format (backwards compatibility)
  overall_assessment?: string;
  risk_management_review?: string;
  market_context?: string;
  key_mistakes?: string[];
  what_went_wrong?: string;
  what_could_improve?: string[];
  confidence_score?: number;
  suggested_improvements?: string[];
}

// Config Suggestion from GPT-4
interface ConfigSuggestion {
  id?: string;
  _id?: { $oid: string };
  created_at: string | BsonDate;
  status: string;
  timestamp?: string;
  current_config?: Record<string, unknown>;
  trade_stats?: TradeStats;
  suggestions: GPT4ConfigSuggestions;
  applied_changes: string[];
  task_id?: string;
}

interface TradeStats {
  total_trades: number;
  winning_trades?: number;
  losing_trades?: number;
  win_rate: number;
  total_pnl: number;
  average_pnl?: number;
}

interface GPT4ConfigSuggestions {
  analysis?: {
    root_cause: string;
    market_condition?: string;
    key_issues?: string[];
    data_quality?: string;
  };
  indicator_suggestions?: Record<string, unknown>;
  signal_suggestions?: Record<string, unknown>;
  summary?: string;
  confidence?: number;
  auto_apply_safe?: boolean;
  risk_assessment?: string;
}

// Utility functions
const formatCurrency = (value: number): string => {
  const sign = value >= 0 ? "+" : "";
  return `${sign}$${value.toFixed(2)}`;
};

const formatPercentage = (value: number): string => {
  const sign = value >= 0 ? "+" : "";
  return `${sign}${value.toFixed(2)}%`;
};

const formatDate = (dateValue: string | BsonDate): string => {
  let date: Date;

  // Handle BSON date format from MongoDB
  if (typeof dateValue === 'object' && dateValue.$date) {
    const timestamp = parseInt(dateValue.$date.$numberLong, 10);
    date = new Date(timestamp);
  } else {
    date = new Date(dateValue as string);
  }

  return date.toLocaleString();
};

// Helper to render analysis detail (handles both string and object format)
const renderAnalysisDetail = (detail: string | AnalysisDetail | undefined): React.ReactNode => {
  if (!detail) return null;

  // If it's a string, render directly
  if (typeof detail === 'string') {
    return <p style={{ color: luxuryColors.text.secondary }} className="text-sm">{detail}</p>;
  }

  // If it's an object, render structured content
  return (
    <div className="space-y-2">
      {detail.quality && (
        <div className="flex items-center gap-2">
          <Badge variant={detail.quality === 'good' ? 'success' : detail.quality === 'acceptable' ? 'warning' : 'error'}>
            {detail.quality}
          </Badge>
          {detail.signals_valid !== undefined && (
            <span className="text-xs" style={{ color: luxuryColors.text.muted }}>
              Signals: {detail.signals_valid ? '✓ Valid' : '✗ Invalid'}
            </span>
          )}
        </div>
      )}
      {detail.reasoning && (
        <p style={{ color: luxuryColors.text.secondary }} className="text-sm">{detail.reasoning}</p>
      )}
      {detail.better_exit_point && (
        <p className="text-sm mt-1" style={{ color: luxuryColors.status.warning }}>
          <strong>Better exit:</strong> {detail.better_exit_point}
        </p>
      )}
    </div>
  );
};

export default function TradeAnalyses() {
  // State
  const [tradeAnalyses, setTradeAnalyses] = useState<TradeAnalysis[]>([]);
  const [configSuggestions, setConfigSuggestions] = useState<ConfigSuggestion[]>([]);
  const [latestSuggestion, setLatestSuggestion] = useState<ConfigSuggestion | null>(null);
  const [loading, setLoading] = useState(true);
  const [refreshing, setRefreshing] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [selectedTab, setSelectedTab] = useState("analyses");
  const [showOnlyLosing, setShowOnlyLosing] = useState(true);

  // Fetch trade analyses
  const fetchTradeAnalyses = useCallback(async (onlyLosing: boolean = true) => {
    try {
      const response = await fetch(
        `${API_BASE}/api/paper-trading/trade-analyses?only_losing=${onlyLosing}&limit=50`
      );
      const result: ApiResponse<TradeAnalysis[]> = await response.json();

      if (result.success && result.data) {
        setTradeAnalyses(result.data);
      } else {
        throw new Error(result.error || "Failed to fetch trade analyses");
      }
    } catch (err) {
      throw err;
    }
  }, []);

  // Fetch config suggestions from Python AI service
  const fetchConfigSuggestions = useCallback(async () => {
    try {
      const response = await fetch(
        `${PYTHON_API_BASE}/ai/config-suggestions?limit=20`
      );
      const result = await response.json();

      if (result.success && result.suggestions) {
        // Map Python API response format to our interface
        const suggestions = result.suggestions.map((s: Record<string, unknown>) => ({
          ...s,
          id: s._id,
        }));
        setConfigSuggestions(suggestions);
      } else {
        // Config suggestions may not exist yet, don't throw
        setConfigSuggestions([]);
      }
    } catch {
      setConfigSuggestions([]);
    }
  }, []);

  // Fetch latest config suggestion from Python AI service (get first from list)
  const fetchLatestSuggestion = useCallback(async () => {
    try {
      const response = await fetch(
        `${PYTHON_API_BASE}/ai/config-suggestions?limit=1`
      );
      const result = await response.json();

      if (result.success && result.suggestions && result.suggestions.length > 0) {
        const latest = result.suggestions[0];
        setLatestSuggestion({
          ...latest,
          id: latest._id,
        });
      } else {
        setLatestSuggestion(null);
      }
    } catch {
      setLatestSuggestion(null);
    }
  }, []);

  // Load all data
  const loadData = useCallback(async () => {
    setLoading(true);
    setError(null);

    try {
      await Promise.all([
        fetchTradeAnalyses(showOnlyLosing),
        fetchConfigSuggestions(),
        fetchLatestSuggestion(),
      ]);
    } catch (err) {
      setError(err instanceof Error ? err.message : "Failed to load data");
      toast.error("Failed to load trade analyses");
    } finally {
      setLoading(false);
    }
  }, [fetchTradeAnalyses, fetchConfigSuggestions, fetchLatestSuggestion, showOnlyLosing]);

  // Refresh data
  const handleRefresh = async () => {
    setRefreshing(true);
    await loadData();
    setRefreshing(false);
    toast.success("Data refreshed");
  };

  // Toggle filter
  const handleToggleFilter = async () => {
    const newValue = !showOnlyLosing;
    setShowOnlyLosing(newValue);
    setRefreshing(true);
    await fetchTradeAnalyses(newValue);
    setRefreshing(false);
  };

  // Initial load
  useEffect(() => {
    loadData();
  }, [loadData]);

  // Calculate stats
  const stats = {
    totalAnalyses: tradeAnalyses.length,
    losingTrades: tradeAnalyses.filter(a => !a.is_winning).length,
    winningTrades: tradeAnalyses.filter(a => a.is_winning).length,
    totalPnL: tradeAnalyses.reduce((sum, a) => sum + a.pnl_usdt, 0),
    averagePnL: tradeAnalyses.length > 0
      ? tradeAnalyses.reduce((sum, a) => sum + a.pnl_usdt, 0) / tradeAnalyses.length
      : 0,
  };

  return (
    <PageWrapper>
      <motion.main
        className="container mx-auto px-4 py-8"
        variants={containerVariants}
        initial="hidden"
        animate="visible"
      >
        {/* Page Header */}
        <motion.div variants={itemVariants} className="flex justify-between items-center mb-8">
          <div>
            <SectionHeader
              icon={Brain}
              title="GPT-4 Trade Analyses"
              subtitle="AI-powered analysis of your trades to help you understand what went wrong and improve your strategy."
            />
          </div>
          <PremiumButton
            onClick={handleRefresh}
            disabled={refreshing}
            variant="secondary"
            className="flex items-center gap-2"
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
            Refresh
          </PremiumButton>
        </motion.div>

        {/* Error Alert */}
        {error && (
          <motion.div variants={itemVariants}>
            <GlassCard className="mb-6 p-4 flex items-start gap-3" style={{
              backgroundColor: `${luxuryColors.status.error}15`,
              borderColor: luxuryColors.status.error
            }}>
              <AlertCircle className="h-5 w-5 flex-shrink-0" style={{ color: luxuryColors.status.error }} />
              <div>
                <h4 className="font-medium" style={{ color: luxuryColors.status.error }}>Error</h4>
                <p className="text-sm mt-1" style={{ color: luxuryColors.text.primary }}>{error}</p>
              </div>
            </GlassCard>
          </motion.div>
        )}

        {/* Stats Cards */}
        <motion.div
          variants={itemVariants}
          className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8"
        >
          <StatCard
            icon={BarChart2}
            iconColor={luxuryColors.accent.cyan}
            label="Total Analyses"
            value={stats.totalAnalyses.toString()}
          />

          <StatCard
            icon={XCircle}
            iconColor={luxuryColors.status.error}
            label="Losing Trades"
            value={stats.losingTrades.toString()}
            valueColor={luxuryColors.status.error}
          />

          <StatCard
            icon={CheckCircle}
            iconColor={luxuryColors.status.success}
            label="Winning Trades"
            value={stats.winningTrades.toString()}
            valueColor={luxuryColors.status.success}
          />

          <StatCard
            icon={DollarSign}
            iconColor={luxuryColors.accent.gold}
            label="Average P&L"
            value={formatCurrency(stats.averagePnL)}
            valueColor={stats.averagePnL >= 0 ? luxuryColors.status.success : luxuryColors.status.error}
          />
        </motion.div>

        {/* Latest Config Suggestion Alert */}
        {latestSuggestion && latestSuggestion.suggestions?.analysis && (
          <motion.div variants={itemVariants}>
            <GlassCard className="mb-6 p-4" style={{
              borderColor: luxuryColors.accent.purple,
              background: `linear-gradient(135deg, ${luxuryColors.accent.purple}10 0%, transparent 100%)`
            }}>
              <div className="flex items-start gap-3">
                <GlowIcon icon={Lightbulb} color={luxuryColors.accent.purple} size="sm" />
                <div>
                  <h4 className="font-medium mb-2" style={{ color: luxuryColors.accent.purple }}>
                    Latest GPT-4 Config Suggestion
                  </h4>
                  <div style={{ color: luxuryColors.text.primary }}>
                    <p className="mb-2">
                      <strong>Root Cause:</strong> {latestSuggestion.suggestions.analysis.root_cause}
                    </p>
                    {latestSuggestion.suggestions.summary && (
                      <p>{latestSuggestion.suggestions.summary}</p>
                    )}
                    {latestSuggestion.applied_changes && latestSuggestion.applied_changes.length > 0 && (
                      <div className="mt-2">
                        <strong>Auto-applied changes:</strong>
                        <ul className="list-disc ml-5 mt-1">
                          {latestSuggestion.applied_changes.map((change, idx) => (
                            <li key={idx}>{change}</li>
                          ))}
                        </ul>
                      </div>
                    )}
                  </div>
                </div>
              </div>
            </GlassCard>
          </motion.div>
        )}

        {/* Tabs */}
        <motion.div variants={itemVariants}>
          <Tabs value={selectedTab} onValueChange={setSelectedTab} className="space-y-4">
            <TabsList style={{
              backgroundColor: luxuryColors.glass.background,
              backdropFilter: luxuryColors.glass.blur,
              border: `1px solid ${luxuryColors.glass.border}`
            }}>
              <TabsTrigger value="analyses" className="flex items-center gap-2">
                <Brain className="h-4 w-4" />
                Trade Analyses
              </TabsTrigger>
              <TabsTrigger value="suggestions" className="flex items-center gap-2">
                <Settings2 className="h-4 w-4" />
                Config Suggestions
              </TabsTrigger>
            </TabsList>

            {/* Trade Analyses Tab */}
            <TabsContent value="analyses">
              <GlassCard>
                <div className="p-6 pb-2">
                  <div className="flex justify-between items-center">
                    <h3 className="text-lg font-semibold flex items-center gap-2" style={{ color: luxuryColors.text.primary }}>
                      <GlowIcon icon={History} color={luxuryColors.accent.cyan} />
                      GPT-4 Trade Analyses
                    </h3>
                    <PremiumButton
                      variant={showOnlyLosing ? "danger" : "secondary"}
                      size="sm"
                      onClick={handleToggleFilter}
                    >
                      {showOnlyLosing ? "Showing: Losing Only" : "Showing: All Trades"}
                    </PremiumButton>
                  </div>
                </div>
                <div className="p-6 pt-2">
                  {loading ? (
                    <LoadingSpinner message="Loading trade analyses..." />
                  ) : tradeAnalyses.length === 0 ? (
                    <EmptyState
                      icon={Brain}
                      title="No trade analyses yet"
                      description="Analyses are generated automatically for closed trades by the Python AI service."
                    />
                  ) : (
                  <Accordion type="single" collapsible className="space-y-2">
                    {tradeAnalyses.map((analysis, index) => (
                      <AccordionItem
                        key={analysis.trade_id || index}
                        value={analysis.trade_id || String(index)}
                        style={{
                          backgroundColor: luxuryColors.glass.background,
                          backdropFilter: luxuryColors.glass.blur,
                          border: `1px solid ${luxuryColors.glass.border}`,
                          borderRadius: '12px',
                        }}
                        className="px-4"
                      >
                        <AccordionTrigger className="hover:no-underline">
                          <div className="flex items-center justify-between w-full pr-4">
                            <div className="flex items-center gap-4">
                              {analysis.is_winning ? (
                                <GlowIcon icon={TrendingUp} color={luxuryColors.status.success} />
                              ) : (
                                <GlowIcon icon={TrendingDown} color={luxuryColors.status.error} />
                              )}
                              <div className="text-left">
                                <div className="font-medium" style={{ color: luxuryColors.text.primary }}>
                                  {analysis.symbol || "Unknown"}{" "}
                                  <Badge
                                    variant={analysis.side === "Long" ? "success" : "error"}
                                    className="ml-2"
                                  >
                                    {analysis.side || "N/A"}
                                  </Badge>
                                </div>
                                <div className="text-sm" style={{ color: luxuryColors.text.secondary }}>
                                  {analysis.close_reason || "Closed"} • {formatDate(analysis.created_at)}
                                </div>
                              </div>
                            </div>
                            <div className="flex items-center gap-4">
                              <div className="text-right" style={{
                                color: analysis.pnl_usdt >= 0 ? luxuryColors.status.success : luxuryColors.status.error
                              }}>
                                <GradientText className="font-bold text-xl">
                                  {formatCurrency(analysis.pnl_usdt)}
                                </GradientText>
                                <div className="text-sm">{formatPercentage(analysis.pnl_percentage)}</div>
                              </div>
                            </div>
                          </div>
                        </AccordionTrigger>
                        <AccordionContent>
                          <div className="pt-4 space-y-4">
                            {/* Trade Details */}
                            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                              <div>
                                <span style={{ color: luxuryColors.text.secondary }}>Entry Price:</span>
                                <span className="ml-2" style={{ color: luxuryColors.text.primary }}>
                                  ${analysis.entry_price?.toFixed(2) || "N/A"}
                                </span>
                              </div>
                              <div>
                                <span style={{ color: luxuryColors.text.secondary }}>Exit Price:</span>
                                <span className="ml-2" style={{ color: luxuryColors.text.primary }}>
                                  ${analysis.exit_price?.toFixed(2) || "N/A"}
                                </span>
                              </div>
                              <div>
                                <span style={{ color: luxuryColors.text.secondary }}>Trade ID:</span>
                                <span className="ml-2 font-mono text-xs" style={{ color: luxuryColors.text.muted }}>
                                  {analysis.trade_id}
                                </span>
                              </div>
                              <div>
                                <span style={{ color: luxuryColors.text.secondary }}>Close Reason:</span>
                                <span className="ml-2" style={{ color: luxuryColors.text.primary }}>
                                  {analysis.close_reason || "Manual"}
                                </span>
                              </div>
                            </div>

                            <Divider />

                            {/* GPT-4 Analysis */}
                            <div className="space-y-4">
                              {/* New format: trade_verdict + summary OR old format: overall_assessment */}
                              {(analysis.analysis?.trade_verdict || analysis.analysis?.summary || analysis.analysis?.overall_assessment) && (
                                <GlassCard className="p-4" style={{
                                  borderColor: luxuryColors.accent.purple,
                                  background: `linear-gradient(135deg, ${luxuryColors.accent.purple}10 0%, transparent 100%)`
                                }}>
                                  <h4 className="font-medium flex items-center gap-2 mb-2" style={{ color: luxuryColors.accent.purple }}>
                                    <GlowIcon icon={Brain} color={luxuryColors.accent.purple} size="sm" />
                                    {analysis.analysis?.trade_verdict || 'Overall Assessment'}
                                  </h4>
                                  <p style={{ color: luxuryColors.text.primary }}>
                                    {analysis.analysis?.summary || analysis.analysis?.overall_assessment}
                                  </p>
                                </GlassCard>
                              )}

                              {analysis.analysis?.what_went_wrong && (
                                <GlassCard className="p-4" style={{
                                  borderColor: luxuryColors.status.error,
                                  background: `linear-gradient(135deg, ${luxuryColors.status.error}15 0%, transparent 100%)`
                                }}>
                                  <h4 className="font-medium flex items-center gap-2 mb-2" style={{ color: luxuryColors.status.error }}>
                                    <GlowIcon icon={AlertCircle} color={luxuryColors.status.error} size="sm" />
                                    What Went Wrong
                                  </h4>
                                  <p style={{ color: luxuryColors.text.primary }}>{analysis.analysis.what_went_wrong}</p>
                                </GlassCard>
                              )}

                              {/* New format: key_factors OR old format: key_mistakes */}
                              {((analysis.analysis?.key_factors && analysis.analysis.key_factors.length > 0) ||
                                (analysis.analysis?.key_mistakes && analysis.analysis.key_mistakes.length > 0)) && (
                                <GlassCard className="p-4" style={{
                                  borderColor: luxuryColors.status.warning,
                                  background: `linear-gradient(135deg, ${luxuryColors.status.warning}15 0%, transparent 100%)`
                                }}>
                                  <h4 className="font-medium flex items-center gap-2 mb-2" style={{ color: luxuryColors.status.warning }}>
                                    <GlowIcon icon={Target} color={luxuryColors.status.warning} size="sm" />
                                    {analysis.analysis?.key_factors ? 'Key Factors' : 'Key Mistakes'}
                                  </h4>
                                  <ul className="list-disc ml-5 space-y-1" style={{ color: luxuryColors.text.primary }}>
                                    {(analysis.analysis?.key_factors || analysis.analysis?.key_mistakes || []).map((item, i) => (
                                      <li key={i}>{item}</li>
                                    ))}
                                  </ul>
                                </GlassCard>
                              )}

                              {analysis.analysis?.lessons_learned && analysis.analysis.lessons_learned.length > 0 && (
                                <GlassCard className="p-4" style={{
                                  borderColor: luxuryColors.status.success,
                                  background: `linear-gradient(135deg, ${luxuryColors.status.success}15 0%, transparent 100%)`
                                }}>
                                  <h4 className="font-medium flex items-center gap-2 mb-2" style={{ color: luxuryColors.status.success }}>
                                    <GlowIcon icon={Lightbulb} color={luxuryColors.status.success} size="sm" />
                                    Lessons Learned
                                  </h4>
                                  <ul className="list-disc ml-5 space-y-1" style={{ color: luxuryColors.text.primary }}>
                                    {analysis.analysis.lessons_learned.map((lesson, i) => (
                                      <li key={i}>{lesson}</li>
                                    ))}
                                  </ul>
                                </GlassCard>
                              )}

                              {/* New format: recommendations OR old format: suggested_improvements */}
                              {(analysis.analysis?.recommendations ||
                                (analysis.analysis?.suggested_improvements && analysis.analysis.suggested_improvements.length > 0)) && (
                                <GlassCard className="p-4" style={{
                                  borderColor: luxuryColors.accent.cyan,
                                  background: `linear-gradient(135deg, ${luxuryColors.accent.cyan}15 0%, transparent 100%)`
                                }}>
                                  <h4 className="font-medium flex items-center gap-2 mb-2" style={{ color: luxuryColors.accent.cyan }}>
                                    <GlowIcon icon={TrendingUp} color={luxuryColors.accent.cyan} size="sm" />
                                    Recommendations
                                  </h4>
                                  {/* Old format */}
                                  {analysis.analysis?.suggested_improvements && (
                                    <ul className="list-disc ml-5 space-y-1" style={{ color: luxuryColors.text.primary }}>
                                      {analysis.analysis.suggested_improvements.map((improvement, i) => (
                                        <li key={i}>{improvement}</li>
                                      ))}
                                    </ul>
                                  )}
                                  {/* New format */}
                                  {analysis.analysis?.recommendations && (
                                    <div className="space-y-3">
                                      {analysis.analysis.recommendations.strategy_improvements && (
                                        <div>
                                          <h5 className="text-sm font-medium" style={{ color: luxuryColors.text.secondary }}>
                                            Strategy Improvements:
                                          </h5>
                                          <ul className="list-disc ml-5 space-y-1 text-sm" style={{ color: luxuryColors.text.primary }}>
                                            {analysis.analysis.recommendations.strategy_improvements.map((item, i) => (
                                              <li key={i}>{item}</li>
                                            ))}
                                          </ul>
                                        </div>
                                      )}
                                      {analysis.analysis.recommendations.risk_management && (
                                        <div>
                                          <h5 className="text-sm font-medium" style={{ color: luxuryColors.text.secondary }}>
                                            Risk Management:
                                          </h5>
                                          <p className="text-sm ml-2" style={{ color: luxuryColors.text.primary }}>
                                            {analysis.analysis.recommendations.risk_management}
                                          </p>
                                        </div>
                                      )}
                                      {analysis.analysis.recommendations.config_changes && (
                                        <div>
                                          <h5 className="text-sm font-medium" style={{ color: luxuryColors.text.secondary }}>
                                            Config Changes:
                                          </h5>
                                          <ul className="list-disc ml-5 space-y-1 text-sm" style={{ color: luxuryColors.text.primary }}>
                                            {Object.entries(analysis.analysis.recommendations.config_changes).map(([key, value], i) => (
                                              <li key={i}><strong>{key}:</strong> {value}</li>
                                            ))}
                                          </ul>
                                        </div>
                                      )}
                                    </div>
                                  )}
                                </GlassCard>
                              )}

                              {/* Additional analysis sections */}
                              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                {analysis.analysis?.entry_analysis && (
                                  <GlassCard className="p-4">
                                    <h4 className="font-medium mb-2" style={{ color: luxuryColors.text.primary }}>
                                      Entry Analysis
                                    </h4>
                                    {renderAnalysisDetail(analysis.analysis.entry_analysis)}
                                  </GlassCard>
                                )}
                                {analysis.analysis?.exit_analysis && (
                                  <GlassCard className="p-4">
                                    <h4 className="font-medium mb-2" style={{ color: luxuryColors.text.primary }}>
                                      Exit Analysis
                                    </h4>
                                    {renderAnalysisDetail(analysis.analysis.exit_analysis)}
                                  </GlassCard>
                                )}
                              </div>

                              {analysis.analysis?.risk_management_review && (
                                <GlassCard className="p-4">
                                  <h4 className="font-medium mb-2" style={{ color: luxuryColors.text.primary }}>
                                    Risk Management Review
                                  </h4>
                                  <p className="text-sm" style={{ color: luxuryColors.text.secondary }}>
                                    {analysis.analysis.risk_management_review}
                                  </p>
                                </GlassCard>
                              )}

                              {analysis.analysis?.market_context && (
                                <GlassCard className="p-4">
                                  <h4 className="font-medium mb-2" style={{ color: luxuryColors.text.primary }}>
                                    Market Context
                                  </h4>
                                  <p className="text-sm" style={{ color: luxuryColors.text.secondary }}>
                                    {analysis.analysis.market_context}
                                  </p>
                                </GlassCard>
                              )}
                            </div>
                          </div>
                        </AccordionContent>
                      </AccordionItem>
                    ))}
                  </Accordion>
                )}
              </div>
            </GlassCard>
          </TabsContent>

          {/* Config Suggestions Tab */}
          <TabsContent value="suggestions">
            <GlassCard>
              <div className="p-6 pb-2">
                <h3 className="text-lg font-semibold flex items-center gap-2" style={{ color: luxuryColors.text.primary }}>
                  <GlowIcon icon={Settings2} color={luxuryColors.accent.purple} />
                  GPT-4 Config Improvement Suggestions
                </h3>
              </div>
              <div className="p-6 pt-2">
                {loading ? (
                  <LoadingSpinner message="Loading config suggestions..." />
                ) : configSuggestions.length === 0 ? (
                  <EmptyState
                    icon={Settings2}
                    title="No config suggestions yet"
                    description="GPT-4 analyzes your trading performance and suggests config improvements periodically."
                  />
                ) : (
                  <div className="space-y-4">
                    {configSuggestions.map((suggestion, index) => (
                      <GlassCard key={suggestion.task_id || index}>
                        <div className="p-4 pb-2">
                          <div className="flex justify-between items-start">
                            <div>
                              <h4 className="text-lg font-semibold flex items-center gap-2" style={{ color: luxuryColors.text.primary }}>
                                <GlowIcon icon={Clock} color={luxuryColors.text.secondary} size="sm" />
                                {formatDate(suggestion.created_at)}
                              </h4>
                              <Badge
                                variant={suggestion.status === "success" ? "success" : "error"}
                                className="mt-1"
                              >
                                {suggestion.status}
                              </Badge>
                            </div>
                            {suggestion.suggestions?.confidence && (
                              <Badge variant="info" style={{
                                borderColor: luxuryColors.accent.purple,
                                color: luxuryColors.accent.purple
                              }}>
                                Confidence: {(suggestion.suggestions.confidence * 100).toFixed(0)}%
                              </Badge>
                            )}
                          </div>
                        </div>
                        <div className="p-4 pt-0 space-y-4">
                          {/* Trade Stats */}
                          {suggestion.trade_stats && (
                            <GlassCard className="p-3">
                              <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm">
                                <div>
                                  <span style={{ color: luxuryColors.text.secondary }}>Total Trades:</span>
                                  <span className="ml-2" style={{ color: luxuryColors.text.primary }}>
                                    {suggestion.trade_stats.total_trades}
                                  </span>
                                </div>
                                <div>
                                  <span style={{ color: luxuryColors.text.secondary }}>Win Rate:</span>
                                  <span className="ml-2" style={{
                                    color: suggestion.trade_stats.win_rate >= 50
                                      ? luxuryColors.status.success
                                      : luxuryColors.status.error
                                  }}>
                                    {suggestion.trade_stats.win_rate.toFixed(1)}%
                                  </span>
                                </div>
                                <div>
                                  <span style={{ color: luxuryColors.text.secondary }}>Total P&L:</span>
                                  <span className="ml-2" style={{
                                    color: suggestion.trade_stats.total_pnl >= 0
                                      ? luxuryColors.status.success
                                      : luxuryColors.status.error
                                  }}>
                                    {formatCurrency(suggestion.trade_stats.total_pnl)}
                                  </span>
                                </div>
                                {suggestion.trade_stats.average_pnl !== undefined && (
                                  <div>
                                    <span style={{ color: luxuryColors.text.secondary }}>Avg P&L:</span>
                                    <span className="ml-2" style={{
                                      color: suggestion.trade_stats.average_pnl >= 0
                                        ? luxuryColors.status.success
                                        : luxuryColors.status.error
                                    }}>
                                      {formatCurrency(suggestion.trade_stats.average_pnl)}
                                    </span>
                                  </div>
                                )}
                              </div>
                            </GlassCard>
                          )}

                          {/* Analysis */}
                          {suggestion.suggestions?.analysis && (
                            <GlassCard className="p-4" style={{
                              borderColor: luxuryColors.accent.purple,
                              background: `linear-gradient(135deg, ${luxuryColors.accent.purple}10 0%, transparent 100%)`
                            }}>
                              <h4 className="font-medium mb-2" style={{ color: luxuryColors.accent.purple }}>
                                Root Cause Analysis
                              </h4>
                              <p style={{ color: luxuryColors.text.primary }}>
                                {suggestion.suggestions.analysis.root_cause}
                              </p>
                              {suggestion.suggestions.analysis.market_condition && (
                                <p className="text-sm mt-2" style={{ color: luxuryColors.text.secondary }}>
                                  Market Condition: {suggestion.suggestions.analysis.market_condition}
                                </p>
                              )}
                              {suggestion.suggestions.analysis.data_quality && (
                                <p className="text-sm mt-2" style={{ color: luxuryColors.text.secondary }}>
                                  <Badge
                                    variant={
                                      suggestion.suggestions.analysis.data_quality === "good" ? "success" :
                                      suggestion.suggestions.analysis.data_quality === "limited" ? "warning" : "error"
                                    }
                                    className="mr-2"
                                  >
                                    {suggestion.suggestions.analysis.data_quality}
                                  </Badge>
                                  Data Quality
                                </p>
                              )}
                              {suggestion.suggestions.analysis.key_issues && suggestion.suggestions.analysis.key_issues.length > 0 && (
                                <div className="mt-3">
                                  <p className="text-sm font-medium mb-1" style={{ color: luxuryColors.text.secondary }}>
                                    Key Issues:
                                  </p>
                                  <ul className="list-disc ml-5 space-y-1 text-sm" style={{ color: luxuryColors.text.primary }}>
                                    {suggestion.suggestions.analysis.key_issues.map((issue, i) => (
                                      <li key={i}>{issue}</li>
                                    ))}
                                  </ul>
                                </div>
                              )}
                            </GlassCard>
                          )}

                          {/* Summary */}
                          {suggestion.suggestions?.summary && (
                            <GlassCard className="p-4">
                              <h4 className="font-medium mb-2" style={{ color: luxuryColors.text.primary }}>
                                Summary
                              </h4>
                              <p style={{ color: luxuryColors.text.secondary }}>{suggestion.suggestions.summary}</p>
                            </GlassCard>
                          )}

                          {/* Applied Changes */}
                          {suggestion.applied_changes && suggestion.applied_changes.length > 0 && (
                            <GlassCard className="p-4" style={{
                              borderColor: luxuryColors.status.success,
                              background: `linear-gradient(135deg, ${luxuryColors.status.success}15 0%, transparent 100%)`
                            }}>
                              <h4 className="font-medium flex items-center gap-2 mb-2" style={{ color: luxuryColors.status.success }}>
                                <GlowIcon icon={CheckCircle} color={luxuryColors.status.success} size="sm" />
                                Auto-Applied Changes
                              </h4>
                              <ul className="list-disc ml-5 space-y-1" style={{ color: luxuryColors.text.primary }}>
                                {suggestion.applied_changes.map((change, i) => (
                                  <li key={i}>{change}</li>
                                ))}
                              </ul>
                            </GlassCard>
                          )}

                          {/* Indicator Suggestions */}
                          {suggestion.suggestions?.indicator_suggestions && Object.keys(suggestion.suggestions.indicator_suggestions).length > 0 && (
                            <GlassCard className="p-4" style={{
                              borderColor: luxuryColors.accent.cyan,
                              background: `linear-gradient(135deg, ${luxuryColors.accent.cyan}10 0%, transparent 100%)`
                            }}>
                              <h4 className="font-medium mb-2" style={{ color: luxuryColors.accent.cyan }}>
                                Indicator Suggestions
                              </h4>
                              <pre className="text-xs overflow-auto" style={{
                                color: luxuryColors.text.secondary,
                                backgroundColor: `${luxuryColors.glass.background}80`,
                                padding: '12px',
                                borderRadius: '8px'
                              }}>
                                {JSON.stringify(suggestion.suggestions.indicator_suggestions, null, 2)}
                              </pre>
                            </GlassCard>
                          )}

                          {/* Signal Suggestions */}
                          {suggestion.suggestions?.signal_suggestions && Object.keys(suggestion.suggestions.signal_suggestions).length > 0 && (
                            <GlassCard className="p-4" style={{
                              borderColor: luxuryColors.status.warning,
                              background: `linear-gradient(135deg, ${luxuryColors.status.warning}10 0%, transparent 100%)`
                            }}>
                              <h4 className="font-medium mb-2" style={{ color: luxuryColors.status.warning }}>
                                Signal Suggestions
                              </h4>
                              <pre className="text-xs overflow-auto" style={{
                                color: luxuryColors.text.secondary,
                                backgroundColor: `${luxuryColors.glass.background}80`,
                                padding: '12px',
                                borderRadius: '8px'
                              }}>
                                {JSON.stringify(suggestion.suggestions.signal_suggestions, null, 2)}
                              </pre>
                            </GlassCard>
                          )}
                        </div>
                      </GlassCard>
                    ))}
                  </div>
                )}
              </div>
            </GlassCard>
          </TabsContent>
        </Tabs>
        </motion.div>
      </motion.main>
    </PageWrapper>
  );
}
