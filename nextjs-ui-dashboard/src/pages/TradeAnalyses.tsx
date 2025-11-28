/**
 * Trade Analyses Page
 * @spec:FR-ASYNC-011 - GPT-4 Individual Trade Analysis
 *
 * Displays GPT-4 analyses for paper trading trades.
 * Each failed/losing trade is analyzed by GPT-4 to provide insights.
 */

import { DashboardHeader } from "@/components/dashboard/DashboardHeader";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { Alert, AlertDescription, AlertTitle } from "@/components/ui/alert";
import { Separator } from "@/components/ui/separator";
import React, { useState, useEffect, useCallback } from "react";
import { toast } from "sonner";
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
    return <p className="text-gray-400 text-sm">{detail}</p>;
  }

  // If it's an object, render structured content
  return (
    <div className="space-y-2">
      {detail.quality && (
        <div className="flex items-center gap-2">
          <Badge variant={detail.quality === 'good' ? 'default' : detail.quality === 'acceptable' ? 'secondary' : 'destructive'}>
            {detail.quality}
          </Badge>
          {detail.signals_valid !== undefined && (
            <span className="text-xs text-gray-500">
              Signals: {detail.signals_valid ? '✓ Valid' : '✗ Invalid'}
            </span>
          )}
        </div>
      )}
      {detail.reasoning && (
        <p className="text-gray-400 text-sm">{detail.reasoning}</p>
      )}
      {detail.better_exit_point && (
        <p className="text-yellow-400 text-sm mt-1">
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
      console.error("Error fetching trade analyses:", err);
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
    } catch (err) {
      console.error("Error fetching config suggestions:", err);
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
    } catch (err) {
      console.error("Error fetching latest suggestion:", err);
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
    <div className="min-h-screen bg-background">
      <DashboardHeader />

      <main className="container mx-auto px-4 py-8">
        {/* Page Header */}
        <div className="flex justify-between items-center mb-8">
          <div>
            <h1 className="text-3xl font-bold text-white flex items-center gap-3">
              <Brain className="h-8 w-8 text-purple-500" />
              GPT-4 Trade Analyses
            </h1>
            <p className="text-gray-400 mt-2">
              AI-powered analysis of your trades to help you understand what went wrong and improve your strategy.
            </p>
          </div>
          <Button
            onClick={handleRefresh}
            disabled={refreshing}
            variant="outline"
            className="flex items-center gap-2"
          >
            <RefreshCw className={`h-4 w-4 ${refreshing ? "animate-spin" : ""}`} />
            Refresh
          </Button>
        </div>

        {/* Error Alert */}
        {error && (
          <Alert variant="destructive" className="mb-6">
            <AlertCircle className="h-4 w-4" />
            <AlertTitle>Error</AlertTitle>
            <AlertDescription>{error}</AlertDescription>
          </Alert>
        )}

        {/* Stats Cards */}
        <div className="grid grid-cols-1 md:grid-cols-4 gap-4 mb-8">
          <Card className="bg-gray-800/50 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm text-gray-400 flex items-center gap-2">
                <BarChart2 className="h-4 w-4" />
                Total Analyses
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-white">{stats.totalAnalyses}</div>
            </CardContent>
          </Card>

          <Card className="bg-gray-800/50 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm text-gray-400 flex items-center gap-2">
                <XCircle className="h-4 w-4 text-red-500" />
                Losing Trades
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-red-400">{stats.losingTrades}</div>
            </CardContent>
          </Card>

          <Card className="bg-gray-800/50 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm text-gray-400 flex items-center gap-2">
                <CheckCircle className="h-4 w-4 text-green-500" />
                Winning Trades
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-2xl font-bold text-green-400">{stats.winningTrades}</div>
            </CardContent>
          </Card>

          <Card className="bg-gray-800/50 border-gray-700">
            <CardHeader className="pb-2">
              <CardTitle className="text-sm text-gray-400 flex items-center gap-2">
                <DollarSign className="h-4 w-4" />
                Average P&L
              </CardTitle>
            </CardHeader>
            <CardContent>
              <div className={`text-2xl font-bold ${stats.averagePnL >= 0 ? "text-green-400" : "text-red-400"}`}>
                {formatCurrency(stats.averagePnL)}
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Latest Config Suggestion Alert */}
        {latestSuggestion && latestSuggestion.suggestions?.analysis && (
          <Alert className="mb-6 bg-purple-900/30 border-purple-700">
            <Lightbulb className="h-4 w-4 text-purple-400" />
            <AlertTitle className="text-purple-300">Latest GPT-4 Config Suggestion</AlertTitle>
            <AlertDescription className="text-gray-300">
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
            </AlertDescription>
          </Alert>
        )}

        {/* Tabs */}
        <Tabs value={selectedTab} onValueChange={setSelectedTab} className="space-y-4">
          <TabsList className="bg-gray-800">
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
            <Card className="bg-gray-800/50 border-gray-700">
              <CardHeader>
                <div className="flex justify-between items-center">
                  <CardTitle className="text-white flex items-center gap-2">
                    <History className="h-5 w-5" />
                    GPT-4 Trade Analyses
                  </CardTitle>
                  <Button
                    variant="outline"
                    size="sm"
                    onClick={handleToggleFilter}
                    className={showOnlyLosing ? "bg-red-900/30" : ""}
                  >
                    {showOnlyLosing ? "Showing: Losing Only" : "Showing: All Trades"}
                  </Button>
                </div>
              </CardHeader>
              <CardContent>
                {loading ? (
                  <div className="flex items-center justify-center py-12">
                    <RefreshCw className="h-8 w-8 animate-spin text-purple-500" />
                  </div>
                ) : tradeAnalyses.length === 0 ? (
                  <div className="text-center py-12 text-gray-400">
                    <Brain className="h-16 w-16 mx-auto mb-4 opacity-50" />
                    <p className="text-lg">No trade analyses yet</p>
                    <p className="text-sm mt-2">
                      Analyses are generated automatically for closed trades by the Python AI service.
                    </p>
                  </div>
                ) : (
                  <Accordion type="single" collapsible className="space-y-2">
                    {tradeAnalyses.map((analysis, index) => (
                      <AccordionItem
                        key={analysis.trade_id || index}
                        value={analysis.trade_id || String(index)}
                        className="bg-gray-900/50 border border-gray-700 rounded-lg px-4"
                      >
                        <AccordionTrigger className="hover:no-underline">
                          <div className="flex items-center justify-between w-full pr-4">
                            <div className="flex items-center gap-4">
                              {analysis.is_winning ? (
                                <TrendingUp className="h-5 w-5 text-green-500" />
                              ) : (
                                <TrendingDown className="h-5 w-5 text-red-500" />
                              )}
                              <div className="text-left">
                                <div className="font-medium text-white">
                                  {analysis.symbol || "Unknown"}{" "}
                                  <Badge variant={analysis.side === "Long" ? "default" : "destructive"} className="ml-2">
                                    {analysis.side || "N/A"}
                                  </Badge>
                                </div>
                                <div className="text-sm text-gray-400">
                                  {analysis.close_reason || "Closed"} • {formatDate(analysis.created_at)}
                                </div>
                              </div>
                            </div>
                            <div className="flex items-center gap-4">
                              <div className={`text-right ${analysis.pnl_usdt >= 0 ? "text-green-400" : "text-red-400"}`}>
                                <div className="font-bold">{formatCurrency(analysis.pnl_usdt)}</div>
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
                                <span className="text-gray-400">Entry Price:</span>
                                <span className="ml-2 text-white">${analysis.entry_price?.toFixed(2) || "N/A"}</span>
                              </div>
                              <div>
                                <span className="text-gray-400">Exit Price:</span>
                                <span className="ml-2 text-white">${analysis.exit_price?.toFixed(2) || "N/A"}</span>
                              </div>
                              <div>
                                <span className="text-gray-400">Trade ID:</span>
                                <span className="ml-2 text-gray-300 font-mono text-xs">{analysis.trade_id}</span>
                              </div>
                              <div>
                                <span className="text-gray-400">Close Reason:</span>
                                <span className="ml-2 text-white">{analysis.close_reason || "Manual"}</span>
                              </div>
                            </div>

                            <Separator className="bg-gray-700" />

                            {/* GPT-4 Analysis */}
                            <div className="space-y-4">
                              {/* New format: trade_verdict + summary OR old format: overall_assessment */}
                              {(analysis.analysis?.trade_verdict || analysis.analysis?.summary || analysis.analysis?.overall_assessment) && (
                                <div className="bg-gray-800/50 p-4 rounded-lg">
                                  <h4 className="text-purple-400 font-medium flex items-center gap-2 mb-2">
                                    <Brain className="h-4 w-4" />
                                    {analysis.analysis?.trade_verdict || 'Overall Assessment'}
                                  </h4>
                                  <p className="text-gray-300">
                                    {analysis.analysis?.summary || analysis.analysis?.overall_assessment}
                                  </p>
                                </div>
                              )}

                              {analysis.analysis?.what_went_wrong && (
                                <div className="bg-red-900/20 p-4 rounded-lg border border-red-900/50">
                                  <h4 className="text-red-400 font-medium flex items-center gap-2 mb-2">
                                    <AlertCircle className="h-4 w-4" />
                                    What Went Wrong
                                  </h4>
                                  <p className="text-gray-300">{analysis.analysis.what_went_wrong}</p>
                                </div>
                              )}

                              {/* New format: key_factors OR old format: key_mistakes */}
                              {((analysis.analysis?.key_factors && analysis.analysis.key_factors.length > 0) ||
                                (analysis.analysis?.key_mistakes && analysis.analysis.key_mistakes.length > 0)) && (
                                <div className="bg-orange-900/20 p-4 rounded-lg border border-orange-900/50">
                                  <h4 className="text-orange-400 font-medium flex items-center gap-2 mb-2">
                                    <Target className="h-4 w-4" />
                                    {analysis.analysis?.key_factors ? 'Key Factors' : 'Key Mistakes'}
                                  </h4>
                                  <ul className="list-disc ml-5 space-y-1 text-gray-300">
                                    {(analysis.analysis?.key_factors || analysis.analysis?.key_mistakes || []).map((item, i) => (
                                      <li key={i}>{item}</li>
                                    ))}
                                  </ul>
                                </div>
                              )}

                              {analysis.analysis?.lessons_learned && analysis.analysis.lessons_learned.length > 0 && (
                                <div className="bg-green-900/20 p-4 rounded-lg border border-green-900/50">
                                  <h4 className="text-green-400 font-medium flex items-center gap-2 mb-2">
                                    <Lightbulb className="h-4 w-4" />
                                    Lessons Learned
                                  </h4>
                                  <ul className="list-disc ml-5 space-y-1 text-gray-300">
                                    {analysis.analysis.lessons_learned.map((lesson, i) => (
                                      <li key={i}>{lesson}</li>
                                    ))}
                                  </ul>
                                </div>
                              )}

                              {/* New format: recommendations OR old format: suggested_improvements */}
                              {(analysis.analysis?.recommendations ||
                                (analysis.analysis?.suggested_improvements && analysis.analysis.suggested_improvements.length > 0)) && (
                                <div className="bg-blue-900/20 p-4 rounded-lg border border-blue-900/50">
                                  <h4 className="text-blue-400 font-medium flex items-center gap-2 mb-2">
                                    <TrendingUp className="h-4 w-4" />
                                    Recommendations
                                  </h4>
                                  {/* Old format */}
                                  {analysis.analysis?.suggested_improvements && (
                                    <ul className="list-disc ml-5 space-y-1 text-gray-300">
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
                                          <h5 className="text-gray-400 text-sm font-medium">Strategy Improvements:</h5>
                                          <ul className="list-disc ml-5 space-y-1 text-gray-300 text-sm">
                                            {analysis.analysis.recommendations.strategy_improvements.map((item, i) => (
                                              <li key={i}>{item}</li>
                                            ))}
                                          </ul>
                                        </div>
                                      )}
                                      {analysis.analysis.recommendations.risk_management && (
                                        <div>
                                          <h5 className="text-gray-400 text-sm font-medium">Risk Management:</h5>
                                          <p className="text-gray-300 text-sm ml-2">{analysis.analysis.recommendations.risk_management}</p>
                                        </div>
                                      )}
                                      {analysis.analysis.recommendations.config_changes && (
                                        <div>
                                          <h5 className="text-gray-400 text-sm font-medium">Config Changes:</h5>
                                          <ul className="list-disc ml-5 space-y-1 text-gray-300 text-sm">
                                            {Object.entries(analysis.analysis.recommendations.config_changes).map(([key, value], i) => (
                                              <li key={i}><strong>{key}:</strong> {value}</li>
                                            ))}
                                          </ul>
                                        </div>
                                      )}
                                    </div>
                                  )}
                                </div>
                              )}

                              {/* Additional analysis sections */}
                              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                                {analysis.analysis?.entry_analysis && (
                                  <div className="bg-gray-800/50 p-4 rounded-lg">
                                    <h4 className="text-gray-300 font-medium mb-2">Entry Analysis</h4>
                                    {renderAnalysisDetail(analysis.analysis.entry_analysis)}
                                  </div>
                                )}
                                {analysis.analysis?.exit_analysis && (
                                  <div className="bg-gray-800/50 p-4 rounded-lg">
                                    <h4 className="text-gray-300 font-medium mb-2">Exit Analysis</h4>
                                    {renderAnalysisDetail(analysis.analysis.exit_analysis)}
                                  </div>
                                )}
                              </div>

                              {analysis.analysis?.risk_management_review && (
                                <div className="bg-gray-800/50 p-4 rounded-lg">
                                  <h4 className="text-gray-300 font-medium mb-2">Risk Management Review</h4>
                                  <p className="text-gray-400 text-sm">{analysis.analysis.risk_management_review}</p>
                                </div>
                              )}

                              {analysis.analysis?.market_context && (
                                <div className="bg-gray-800/50 p-4 rounded-lg">
                                  <h4 className="text-gray-300 font-medium mb-2">Market Context</h4>
                                  <p className="text-gray-400 text-sm">{analysis.analysis.market_context}</p>
                                </div>
                              )}
                            </div>
                          </div>
                        </AccordionContent>
                      </AccordionItem>
                    ))}
                  </Accordion>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          {/* Config Suggestions Tab */}
          <TabsContent value="suggestions">
            <Card className="bg-gray-800/50 border-gray-700">
              <CardHeader>
                <CardTitle className="text-white flex items-center gap-2">
                  <Settings2 className="h-5 w-5" />
                  GPT-4 Config Improvement Suggestions
                </CardTitle>
              </CardHeader>
              <CardContent>
                {loading ? (
                  <div className="flex items-center justify-center py-12">
                    <RefreshCw className="h-8 w-8 animate-spin text-purple-500" />
                  </div>
                ) : configSuggestions.length === 0 ? (
                  <div className="text-center py-12 text-gray-400">
                    <Settings2 className="h-16 w-16 mx-auto mb-4 opacity-50" />
                    <p className="text-lg">No config suggestions yet</p>
                    <p className="text-sm mt-2">
                      GPT-4 analyzes your trading performance and suggests config improvements periodically.
                    </p>
                  </div>
                ) : (
                  <div className="space-y-4">
                    {configSuggestions.map((suggestion, index) => (
                      <Card key={suggestion.task_id || index} className="bg-gray-900/50 border-gray-700">
                        <CardHeader className="pb-2">
                          <div className="flex justify-between items-start">
                            <div>
                              <CardTitle className="text-lg text-white flex items-center gap-2">
                                <Clock className="h-4 w-4 text-gray-400" />
                                {formatDate(suggestion.created_at)}
                              </CardTitle>
                              <Badge variant={suggestion.status === "success" ? "default" : "destructive"} className="mt-1">
                                {suggestion.status}
                              </Badge>
                            </div>
                            {suggestion.suggestions?.confidence && (
                              <Badge variant="outline" className="text-purple-400 border-purple-400">
                                Confidence: {(suggestion.suggestions.confidence * 100).toFixed(0)}%
                              </Badge>
                            )}
                          </div>
                        </CardHeader>
                        <CardContent className="space-y-4">
                          {/* Trade Stats */}
                          {suggestion.trade_stats && (
                            <div className="grid grid-cols-2 md:grid-cols-4 gap-4 text-sm bg-gray-800/50 p-3 rounded-lg">
                              <div>
                                <span className="text-gray-400">Total Trades:</span>
                                <span className="ml-2 text-white">{suggestion.trade_stats.total_trades}</span>
                              </div>
                              <div>
                                <span className="text-gray-400">Win Rate:</span>
                                <span className={`ml-2 ${suggestion.trade_stats.win_rate >= 50 ? "text-green-400" : "text-red-400"}`}>
                                  {suggestion.trade_stats.win_rate.toFixed(1)}%
                                </span>
                              </div>
                              <div>
                                <span className="text-gray-400">Total P&L:</span>
                                <span className={`ml-2 ${suggestion.trade_stats.total_pnl >= 0 ? "text-green-400" : "text-red-400"}`}>
                                  {formatCurrency(suggestion.trade_stats.total_pnl)}
                                </span>
                              </div>
                              {suggestion.trade_stats.average_pnl !== undefined && (
                                <div>
                                  <span className="text-gray-400">Avg P&L:</span>
                                  <span className={`ml-2 ${suggestion.trade_stats.average_pnl >= 0 ? "text-green-400" : "text-red-400"}`}>
                                    {formatCurrency(suggestion.trade_stats.average_pnl)}
                                  </span>
                                </div>
                              )}
                            </div>
                          )}

                          {/* Analysis */}
                          {suggestion.suggestions?.analysis && (
                            <div className="bg-purple-900/20 p-4 rounded-lg border border-purple-900/50">
                              <h4 className="text-purple-400 font-medium mb-2">Root Cause Analysis</h4>
                              <p className="text-gray-300">{suggestion.suggestions.analysis.root_cause}</p>
                              {suggestion.suggestions.analysis.market_condition && (
                                <p className="text-gray-400 text-sm mt-2">
                                  Market Condition: {suggestion.suggestions.analysis.market_condition}
                                </p>
                              )}
                              {suggestion.suggestions.analysis.data_quality && (
                                <p className="text-gray-400 text-sm mt-2">
                                  <Badge variant={
                                    suggestion.suggestions.analysis.data_quality === "good" ? "default" :
                                    suggestion.suggestions.analysis.data_quality === "limited" ? "secondary" : "destructive"
                                  } className="mr-2">
                                    {suggestion.suggestions.analysis.data_quality}
                                  </Badge>
                                  Data Quality
                                </p>
                              )}
                              {suggestion.suggestions.analysis.key_issues && suggestion.suggestions.analysis.key_issues.length > 0 && (
                                <div className="mt-3">
                                  <p className="text-gray-400 text-sm font-medium mb-1">Key Issues:</p>
                                  <ul className="list-disc ml-5 space-y-1 text-gray-300 text-sm">
                                    {suggestion.suggestions.analysis.key_issues.map((issue, i) => (
                                      <li key={i}>{issue}</li>
                                    ))}
                                  </ul>
                                </div>
                              )}
                            </div>
                          )}

                          {/* Summary */}
                          {suggestion.suggestions?.summary && (
                            <div className="bg-gray-800/50 p-4 rounded-lg">
                              <h4 className="text-gray-300 font-medium mb-2">Summary</h4>
                              <p className="text-gray-400">{suggestion.suggestions.summary}</p>
                            </div>
                          )}

                          {/* Applied Changes */}
                          {suggestion.applied_changes && suggestion.applied_changes.length > 0 && (
                            <div className="bg-green-900/20 p-4 rounded-lg border border-green-900/50">
                              <h4 className="text-green-400 font-medium flex items-center gap-2 mb-2">
                                <CheckCircle className="h-4 w-4" />
                                Auto-Applied Changes
                              </h4>
                              <ul className="list-disc ml-5 space-y-1 text-gray-300">
                                {suggestion.applied_changes.map((change, i) => (
                                  <li key={i}>{change}</li>
                                ))}
                              </ul>
                            </div>
                          )}

                          {/* Indicator Suggestions */}
                          {suggestion.suggestions?.indicator_suggestions && Object.keys(suggestion.suggestions.indicator_suggestions).length > 0 && (
                            <div className="bg-blue-900/20 p-4 rounded-lg border border-blue-900/50">
                              <h4 className="text-blue-400 font-medium mb-2">Indicator Suggestions</h4>
                              <pre className="text-xs text-gray-400 overflow-auto">
                                {JSON.stringify(suggestion.suggestions.indicator_suggestions, null, 2)}
                              </pre>
                            </div>
                          )}

                          {/* Signal Suggestions */}
                          {suggestion.suggestions?.signal_suggestions && Object.keys(suggestion.suggestions.signal_suggestions).length > 0 && (
                            <div className="bg-orange-900/20 p-4 rounded-lg border border-orange-900/50">
                              <h4 className="text-orange-400 font-medium mb-2">Signal Suggestions</h4>
                              <pre className="text-xs text-gray-400 overflow-auto">
                                {JSON.stringify(suggestion.suggestions.signal_suggestions, null, 2)}
                              </pre>
                            </div>
                          )}
                        </CardContent>
                      </Card>
                    ))}
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>
      </main>
    </div>
  );
}
