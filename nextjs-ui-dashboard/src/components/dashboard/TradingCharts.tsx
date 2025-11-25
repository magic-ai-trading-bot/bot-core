import React, { useState, useEffect, useCallback, useRef } from "react";
import logger from "@/utils/logger";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import {


// @spec:FR-DASHBOARD-001 - Real-time Trading Charts
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-INTEGRATION-035, TC-INTEGRATION-036

  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogTrigger,
} from "@/components/ui/dialog";
import { Label } from "@/components/ui/label";
import { Plus, TrendingUp, TrendingDown, Activity, X } from "lucide-react";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
  Area,
  AreaChart,
  ComposedChart,
  Bar,
  Cell,
} from "recharts";
import { apiClient } from "@/services/api";
import type { ChartData, AddSymbolRequest } from "@/services/api";
import { toast } from "sonner";
import {
  useWebSocket,
  type ChartUpdateData,
  type MarketDataUpdateData,
} from "@/hooks/useWebSocket";

interface TradingChartsProps {
  className?: string;
}

interface ChartCardProps {
  chartData: ChartData;
  onRemove: (symbol: string) => void;
}

const AVAILABLE_TIMEFRAMES = ["1m", "5m", "15m", "1h", "4h", "1d"];

function formatPrice(price: number): string {
  if (price >= 1000) {
    return price.toLocaleString("en-US", { maximumFractionDigits: 2 });
  }
  return price.toFixed(6);
}

function formatTime(timestamp: number): string {
  const date = new Date(timestamp);
  return date.toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
}

// Simple Candlestick Chart using DOM elements
const CandlestickChart: React.FC<{
  candles: ChartData["candles"];
  symbol: string;
}> = ({ candles, symbol }) => {
  const [hoveredIndex, setHoveredIndex] = useState<number | null>(null);

  // Prepare data for candlestick chart
  const chartData = candles.slice(-15).map((candle, index) => {
    const isBullish = candle.close >= candle.open;

    return {
      index,
      timestamp: candle.timestamp,
      open: candle.open,
      high: candle.high,
      low: candle.low,
      close: candle.close,
      volume: candle.volume,
      isBullish,
      bodyTop: Math.max(candle.open, candle.close),
      bodyBottom: Math.min(candle.open, candle.close),
      color: isBullish ? "#00ff88" : "#ff4444",
    };
  });

  if (!chartData || chartData.length === 0) {
    return (
      <div className="flex items-center justify-center h-full text-muted-foreground">
        📊 No chart data available
      </div>
    );
  }

  // Calculate price range for scaling
  const allPrices = chartData.flatMap((d) => [d.high, d.low]);
  const minPrice = Math.min(...allPrices);
  const maxPrice = Math.max(...allPrices);
  const priceRange = maxPrice - minPrice;
  const padding = priceRange * 0.1;
  const scaledMin = minPrice - padding;
  const scaledMax = maxPrice + padding;
  const scaledRange = scaledMax - scaledMin;

  // Helper function to convert price to percentage position
  const priceToPercent = (price: number) => {
    return ((scaledMax - price) / scaledRange) * 100;
  };

  return (
    <div className="relative w-full h-full bg-gray-950 rounded overflow-hidden">
      {/* Price grid lines */}
      <div className="absolute inset-0">
        {[0, 25, 50, 75, 100].map((percent) => (
          <div
            key={percent}
            className="absolute w-full border-t border-gray-700 opacity-30"
            style={{ top: `${percent}%` }}
          />
        ))}
      </div>

      {/* Price labels */}
      <div className="absolute left-2 top-2 text-xs text-gray-300 font-mono bg-gray-800 px-1 rounded">
        {formatPrice(scaledMax)}
      </div>
      <div className="absolute left-2 bottom-2 text-xs text-gray-300 font-mono bg-gray-800 px-1 rounded">
        {formatPrice(scaledMin)}
      </div>

      {/* Candlesticks */}
      <div className="absolute inset-4 flex">
        {chartData.map((candle, index) => {
          const wickTopPercent = priceToPercent(candle.high);
          const wickBottomPercent = priceToPercent(candle.low);
          const bodyTopPercent = priceToPercent(candle.bodyTop);
          const bodyBottomPercent = priceToPercent(candle.bodyBottom);

          const wickHeight = wickBottomPercent - wickTopPercent;
          const bodyHeight = Math.max(bodyBottomPercent - bodyTopPercent, 2);

          return (
            <div
              key={index}
              className="flex-1 relative cursor-pointer mx-1"
              onMouseEnter={() => setHoveredIndex(index)}
              onMouseLeave={() => setHoveredIndex(null)}
            >
              {/* Wick (thin line) - centered */}
              <div
                className="absolute left-1/2 transform -translate-x-1/2 w-0.5"
                style={{
                  backgroundColor: candle.color,
                  top: `${wickTopPercent}%`,
                  height: `${wickHeight}%`,
                }}
              />

              {/* Body (thick rectangle) - centered and wider */}
              <div
                className="absolute left-1/2 transform -translate-x-1/2 border"
                style={{
                  top: `${bodyTopPercent}%`,
                  height: `${bodyHeight}%`,
                  width: "12px",
                  backgroundColor: candle.isBullish
                    ? "transparent"
                    : candle.color,
                  borderColor: candle.color,
                  borderWidth: "2px",
                }}
              />

              {/* Hover effect */}
              {hoveredIndex === index && (
                <div className="absolute inset-0 bg-white bg-opacity-20 rounded" />
              )}
            </div>
          );
        })}
      </div>

      {/* Time labels */}
      <div className="absolute bottom-0 left-0 right-0 flex justify-between px-2 py-1 text-xs text-gray-400">
        {chartData && chartData.length > 0 && (
          <>
            <span>{formatTime(chartData[0].timestamp)}</span>
            <span>{formatTime(chartData[chartData.length - 1].timestamp)}</span>
          </>
        )}
      </div>

      {/* Hover tooltip */}
      {hoveredIndex !== null && (
        <div className="absolute top-2 left-2 bg-black border border-gray-600 rounded-lg p-2 shadow-lg z-40 text-xs">
          <div className="text-gray-400 mb-1">
            {new Date(chartData[hoveredIndex].timestamp).toLocaleString()}
          </div>
          <div className="grid grid-cols-2 gap-x-3 gap-y-1">
            <div>
              <span className="text-gray-400">O:</span>{" "}
              <span className="text-orange-400 font-mono">
                {formatPrice(chartData[hoveredIndex].open)}
              </span>
            </div>
            <div>
              <span className="text-gray-400">H:</span>{" "}
              <span className="text-green-400 font-mono">
                {formatPrice(chartData[hoveredIndex].high)}
              </span>
            </div>
            <div>
              <span className="text-gray-400">L:</span>{" "}
              <span className="text-red-400 font-mono">
                {formatPrice(chartData[hoveredIndex].low)}
              </span>
            </div>
            <div>
              <span className="text-gray-400">C:</span>{" "}
              <span
                className={`font-mono ${
                  chartData[hoveredIndex].isBullish
                    ? "text-green-400"
                    : "text-red-400"
                }`}
              >
                {formatPrice(chartData[hoveredIndex].close)}
              </span>
            </div>
          </div>
          <div className="mt-1 text-center">
            <span
              className={`text-xs px-1 py-0.5 rounded ${
                chartData[hoveredIndex].isBullish
                  ? "bg-green-900 text-green-300"
                  : "bg-red-900 text-red-300"
              }`}
            >
              {chartData[hoveredIndex].isBullish ? "🟢 BULL" : "🔴 BEAR"}
            </span>
          </div>
        </div>
      )}
    </div>
  );
};

function formatPercent(percent: number): string {
  return `${percent >= 0 ? "+" : ""}${percent.toFixed(2)}%`;
}

const ChartCard: React.FC<ChartCardProps> = React.memo(
  ({ chartData, onRemove }) => {
    const isPositive = chartData.price_change_percent_24h >= 0;
    const [isPriceUpdating, setIsPriceUpdating] = useState(false);
    const prevPriceRef = useRef(chartData.latest_price);

    // Detect price change and show pulse effect
    useEffect(() => {
      if (prevPriceRef.current !== chartData.latest_price) {
        setIsPriceUpdating(true);
        const timer = setTimeout(() => setIsPriceUpdating(false), 1000);
        prevPriceRef.current = chartData.latest_price;
        return () => clearTimeout(timer);
      }
    }, [chartData.latest_price]);

    return (
      <Card className="relative">
        <Button
          variant="ghost"
          size="sm"
          className="absolute top-2 right-2 h-6 w-6 p-0 opacity-60 hover:opacity-100"
          onClick={() => onRemove(chartData.symbol)}
        >
          <X className="h-3 w-3" />
        </Button>

        <CardHeader className="pb-3">
          <div className="flex items-center justify-between">
            <div>
              <CardTitle className="text-lg">{chartData.symbol}</CardTitle>
              <div className="flex items-center gap-2">
                <Badge variant="outline" className="text-xs">
                  {chartData.timeframe}
                </Badge>
                <Badge variant="outline" className="text-xs">
                  🗂️ MONGODB
                </Badge>
              </div>
            </div>
            <div className="text-right">
              <div
                className={`text-2xl font-bold font-mono transition-all ${
                  isPriceUpdating
                    ? "text-yellow-400 scale-110"
                    : isPositive
                    ? "text-profit"
                    : "text-destructive"
                }`}
              >
                ${formatPrice(chartData.latest_price)}
              </div>
              <div
                className={`text-sm flex items-center ${
                  isPositive ? "text-profit" : "text-destructive"
                }`}
              >
                {isPositive ? (
                  <TrendingUp className="h-3 w-3 mr-1" aria-hidden="true" />
                ) : (
                  <TrendingDown className="h-3 w-3 mr-1" aria-hidden="true" />
                )}
                <span>{formatPercent(chartData.price_change_percent_24h)}</span>
                <span className="sr-only">
                  {isPositive ? "Price increase" : "Price decrease"}
                </span>
              </div>
              {/* Live region for price updates - announced to screen readers */}
              <div
                aria-live="polite"
                aria-atomic="true"
                className="sr-only"
              >
                {chartData.symbol} price updated to ${formatPrice(chartData.latest_price)}
              </div>
            </div>
          </div>
        </CardHeader>

        <CardContent>
          <div className="space-y-3">
            {/* Price Change */}
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">24h Change:</span>
              <span
                className={`font-mono flex items-center gap-1 ${
                  isPositive ? "text-profit" : "text-destructive"
                }`}
              >
                {isPositive ? (
                  <TrendingUp className="h-3 w-3" aria-hidden="true" />
                ) : (
                  <TrendingDown className="h-3 w-3" aria-hidden="true" />
                )}
                <span>${chartData.price_change_24h.toFixed(2)}</span>
              </span>
            </div>

            {/* Volume */}
            <div className="flex justify-between text-sm">
              <span className="text-muted-foreground">24h Volume:</span>
              <span className="font-mono">
                ${(chartData.volume_24h || 0).toLocaleString()}
              </span>
            </div>

            {/* Candlestick Chart */}
            <div className="mt-4 h-40 bg-gray-900 rounded border">
              {chartData.candles && chartData.candles.length > 0 ? (
                <CandlestickChart
                  candles={chartData.candles}
                  symbol={chartData.symbol}
                />
              ) : (
                <div className="h-full bg-muted/20 rounded-md flex items-center justify-center">
                  <div className="flex items-center text-muted-foreground text-sm">
                    <Activity className="h-4 w-4 mr-2" />
                    No chart data available
                  </div>
                </div>
              )}
            </div>

            {/* Latest Candle Info */}
            {chartData.candles && chartData.candles.length > 0 && (
              <div className="text-xs text-muted-foreground space-y-1 p-2 bg-muted/20 rounded">
                <div className="font-semibold text-foreground mb-1">
                  Latest Candle:
                </div>
                <div className="grid grid-cols-2 gap-2">
                  <div className="flex justify-between">
                    <span>Open:</span>
                    <span className="font-mono text-orange-400">
                      $
                      {formatPrice(
                        chartData.candles?.[chartData.candles.length - 1]?.open ||
                          0
                      )}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>High:</span>
                    <span className="font-mono text-green-400">
                      $
                      {formatPrice(
                        chartData.candles?.[chartData.candles.length - 1]?.high ||
                          0
                      )}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Low:</span>
                    <span className="font-mono text-red-400">
                      $
                      {formatPrice(
                        chartData.candles?.[chartData.candles.length - 1]?.low ||
                          0
                      )}
                    </span>
                  </div>
                  <div className="flex justify-between">
                    <span>Close:</span>
                    <span
                      className={`font-mono ${
                        ((chartData.candles && chartData.candles.length > 0
                          ? chartData.candles[chartData.candles.length - 1]?.close
                          : 0) || 0) >=
                        ((chartData.candles && chartData.candles.length > 0
                          ? chartData.candles[chartData.candles.length - 1]?.open
                          : 0) || 0)
                          ? "text-green-400"
                          : "text-red-400"
                      }`}
                    >
                      $
                      {formatPrice(
                        (chartData.candles && chartData.candles.length > 0
                          ? chartData.candles[chartData.candles.length - 1]?.close
                          : 0) || 0
                      )}
                    </span>
                  </div>
                </div>
              </div>
            )}
          </div>
        </CardContent>
      </Card>
    );
  }
);

ChartCard.displayName = "ChartCard";

const AddSymbolDialog: React.FC<{
  onAddSymbol: (request: AddSymbolRequest) => void;
}> = ({ onAddSymbol }) => {
  const [open, setOpen] = useState(false);
  const [symbol, setSymbol] = useState("");
  const [selectedTimeframes, setSelectedTimeframes] = useState<string[]>([
    "1h",
  ]);

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault();

    if (!symbol.trim()) {
      toast.error("Please enter a symbol");
      return;
    }

    if (selectedTimeframes.length === 0) {
      toast.error("Please select at least one timeframe");
      return;
    }

    const request: AddSymbolRequest = {
      symbol: symbol.toUpperCase().trim(),
      timeframes: selectedTimeframes,
    };

    onAddSymbol(request);
    setOpen(false);
    setSymbol("");
    setSelectedTimeframes(["1h"]);
  };

  const toggleTimeframe = (timeframe: string) => {
    setSelectedTimeframes((prev) =>
      prev.includes(timeframe)
        ? prev.filter((tf) => tf !== timeframe)
        : [...prev, timeframe]
    );
  };

  return (
    <Dialog open={open} onOpenChange={setOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" size="sm">
          <Plus className="h-4 w-4 mr-2" />
          Add Symbol
        </Button>
      </DialogTrigger>
      <DialogContent>
        <DialogHeader>
          <DialogTitle>Add New Trading Symbol</DialogTitle>
        </DialogHeader>
        <form onSubmit={handleSubmit} className="space-y-4">
          <div>
            <Label htmlFor="symbol">Symbol (e.g., BTCUSDT)</Label>
            <Input
              id="symbol"
              value={symbol}
              onChange={(e) => setSymbol(e.target.value)}
              placeholder="Enter symbol like BTCUSDT, ETHUSDT..."
              className="mt-1"
            />
          </div>

          <div>
            <Label>Select Timeframes</Label>
            <div className="grid grid-cols-3 gap-2 mt-2">
              {AVAILABLE_TIMEFRAMES.map((timeframe) => (
                <Button
                  key={timeframe}
                  type="button"
                  variant={
                    selectedTimeframes.includes(timeframe)
                      ? "default"
                      : "outline"
                  }
                  size="sm"
                  onClick={() => toggleTimeframe(timeframe)}
                  className="text-xs"
                >
                  {timeframe}
                </Button>
              ))}
            </div>
          </div>

          <div className="flex justify-end space-x-2">
            <Button
              type="button"
              variant="outline"
              onClick={() => setOpen(false)}
            >
              Cancel
            </Button>
            <Button type="submit">Add Symbol</Button>
          </div>
        </form>
      </DialogContent>
    </Dialog>
  );
};

// Default symbols to load immediately without waiting for API
const DEFAULT_SYMBOLS = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];

export const TradingCharts: React.FC<TradingChartsProps> = React.memo(
  ({ className }) => {
    const [charts, setCharts] = useState<ChartData[]>([]);
    const [loading, setLoading] = useState(true);
    const [selectedTimeframe, setSelectedTimeframe] = useState("1m");
    const { state: wsState, connect: connectWs } = useWebSocket();
    const lastPriceUpdateRef = useRef<Record<string, number>>({});
    const abortControllerRef = useRef<AbortController | null>(null);

    const loadChartData = useCallback(async () => {
      // Cancel any pending requests from previous loads
      if (abortControllerRef.current) {
        abortControllerRef.current.abort();
      }

      // Create new AbortController for this load
      const abortController = new AbortController();
      abortControllerRef.current = abortController;

      try {
        setLoading(true);

        // PHASE 1: Load default symbols immediately for instant display
        const chartPromises = DEFAULT_SYMBOLS.map((symbol) =>
          apiClient.rust.getChartDataFast(
            symbol,
            selectedTimeframe,
            100,
            abortController.signal
          ).catch(() => null)
        );

        const chartResults = await Promise.all(chartPromises);
        const successfulCharts = chartResults.filter(
          (chart): chart is ChartData => chart !== null
        );

        if (!abortController.signal.aborted) {
          setCharts(successfulCharts);
          setLoading(false); // End loading after default symbols
        }

        // PHASE 2: Load any additional symbols from backend (user-added symbols)
        // This runs in background after initial display
        try {
          const supportedSymbols = await apiClient.rust.getSupportedSymbols(abortController.signal);
          const allSymbols = supportedSymbols.symbols || [];

          // Find symbols not in DEFAULT_SYMBOLS
          const additionalSymbols = allSymbols.filter(
            (symbol: string) => !DEFAULT_SYMBOLS.includes(symbol)
          );

          if (additionalSymbols.length > 0 && !abortController.signal.aborted) {
            // Load additional symbols in parallel
            const additionalPromises = additionalSymbols.map((symbol: string) =>
              apiClient.rust.getChartDataFast(
                symbol,
                selectedTimeframe,
                100,
                abortController.signal
              ).catch(() => null)
            );

            const additionalResults = await Promise.all(additionalPromises);
            const additionalCharts = additionalResults.filter(
              (chart): chart is ChartData => chart !== null
            );

            if (additionalCharts.length > 0 && !abortController.signal.aborted) {
              setCharts((prev) => [...prev, ...additionalCharts]);
            }
          }
        } catch {
          // Silently ignore errors in phase 2 - user already has default charts
        }
      } catch (error) {
        // Ignore abort errors (expected when component unmounts or new load starts)
        if (error instanceof Error && error.name === "CanceledError") {
          return;
        }
        if ((error as { code?: string })?.code === "ERR_CANCELED") {
          return;
        }

        logger.error("Failed to load chart data:", error);
      } finally {
        // Only update loading state if not aborted
        if (!abortController.signal.aborted) {
          setLoading(false);
        }
      }
    }, [selectedTimeframe]);

    const updatePricesOnly = useCallback(async () => {
      try {
        // Only update prices without reloading entire charts
        const pricesData = await apiClient.rust.getLatestPrices();

        setCharts((prevCharts) =>
          prevCharts.map((chart) => {
            const newPrice = pricesData[chart.symbol];
            if (newPrice && newPrice !== chart.latest_price) {
              const priceChange = newPrice - chart.latest_price;
              const priceChangePercent =
                (priceChange / chart.latest_price) * 100;

              // Track when this price was updated
              lastPriceUpdateRef.current[chart.symbol] = Date.now();

              return {
                ...chart,
                latest_price: newPrice,
                price_change_24h: priceChange,
                price_change_percent_24h: priceChangePercent,
              };
            }
            return chart;
          })
        );
      } catch (error) {
        logger.error("Failed to update prices:", error);
        // Silently fail for price updates to avoid spam
      }
    }, []);

    const handleAddSymbol = async (request: AddSymbolRequest) => {
      try {
        await apiClient.rust.addSymbol(request);
        toast.success(`Successfully added ${request.symbol}`);

        // Load chart data for the new symbol
        try {
          const newChart = await apiClient.rust.getChartData(
            request.symbol,
            selectedTimeframe,
            100
          );
          setCharts((prev) => [...prev, newChart]);
        } catch (error) {
          logger.warn(
            "Failed to load chart for new symbol immediately:",
            error
          );
          // Will be loaded on next refresh
        }
      } catch (error) {
        logger.error("Failed to add symbol:", error);
        toast.error("Failed to add symbol");
      }
    };

    const handleRemoveSymbol = async (symbol: string) => {
      try {
        await apiClient.rust.removeSymbol(symbol);
        setCharts((prev) => prev.filter((chart) => chart.symbol !== symbol));
        toast.success(`Removed ${symbol}`);
      } catch (error) {
        logger.error("Failed to remove symbol:", error);
        toast.error("Failed to remove symbol");
      }
    };

    useEffect(() => {
      loadChartData();
      // Connect WebSocket for real-time updates
      if (!wsState.isConnected && !wsState.isConnecting) {
        connectWs();
      }

      // Cleanup: abort pending requests on unmount or timeframe change
      return () => {
        if (abortControllerRef.current) {
          abortControllerRef.current.abort();
        }
      };
      // eslint-disable-next-line react-hooks/exhaustive-deps
    }, [selectedTimeframe]); // Only depend on selectedTimeframe to avoid infinite loops

    // Real-time price polling every 2 seconds as backup (in case WebSocket is delayed)
    useEffect(() => {
      const interval = setInterval(() => {
        updatePricesOnly();
      }, 2000); // Poll every 2 seconds for smooth updates
      return () => clearInterval(interval);
    }, [updatePricesOnly]);

    // Handle WebSocket updates (both ChartUpdate and MarketData)
    useEffect(() => {
      if (!wsState.lastMessage) return;

      // Handle real-time price updates via MarketData messages
      if (wsState.lastMessage.type === "MarketData") {
        const marketData = wsState.lastMessage.data as MarketDataUpdateData;

        setCharts((prev) =>
          prev.map((chart) =>
            chart.symbol === marketData.symbol
              ? {
                  ...chart,
                  latest_price: marketData.price,
                  // Keep existing 24h data unless provided
                  price_change_24h:
                    marketData.price_change_24h || chart.price_change_24h,
                  price_change_percent_24h:
                    marketData.price_change_percent_24h ||
                    chart.price_change_percent_24h,
                  volume_24h: marketData.volume_24h || chart.volume_24h,
                }
              : chart
          )
        );
      }

      // Handle detailed chart updates (when candles close)
      if (wsState.lastMessage.type === "ChartUpdate") {
        const updateData = wsState.lastMessage.data as ChartUpdateData;

        setCharts((prev) =>
          prev.map((chart) =>
            chart.symbol === updateData.symbol
              ? {
                  ...chart,
                  latest_price: updateData.latest_price,
                  price_change_24h: updateData.price_change_24h,
                  price_change_percent_24h: updateData.price_change_percent_24h,
                  volume_24h: updateData.volume_24h,
                  candles: updateData.candle
                    ? [...chart.candles.slice(-99), updateData.candle]
                    : chart.candles,
                }
              : chart
          )
        );
      }
    }, [wsState.lastMessage]);

    return (
      <div className={className}>
        <Card>
          <CardHeader>
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4">
              <div>
                <CardTitle className="flex items-center gap-2">
                  <Activity className="h-5 w-5" />
                  Real-time Trading Charts
                  <Badge
                    variant="outline"
                    className={
                      wsState.isConnected
                        ? "text-green-600 border-green-600"
                        : "text-red-600 border-red-600"
                    }
                  >
                    {wsState.isConnected ? "🟢 LIVE" : "🔴 DISCONNECTED"}
                  </Badge>
                </CardTitle>
                <div className="flex flex-wrap items-center gap-1 mt-2">
                  <Badge
                    variant="outline"
                    className="text-xs text-green-600 border-green-600"
                  >
                    🔥 HOT RELOAD
                  </Badge>
                  <Badge
                    variant="outline"
                    className="text-xs text-blue-600 border-blue-600"
                  >
                    ⚡ MAINNET
                  </Badge>
                  <Badge
                    variant="outline"
                    className="text-xs text-purple-600 border-purple-600"
                  >
                    🚀 {selectedTimeframe.toUpperCase()}
                  </Badge>
                  <Badge
                    variant="outline"
                    className="text-xs text-orange-600 border-orange-600"
                  >
                    📡 WEBSOCKET
                  </Badge>
                </div>
              </div>
            </div>
          </CardHeader>
          <CardContent>
            <div className="flex flex-col sm:flex-row sm:items-center sm:justify-between gap-4 mb-6">
              <div className="flex flex-wrap items-center gap-2">
                <Select
                  value={selectedTimeframe}
                  onValueChange={setSelectedTimeframe}
                >
                  <SelectTrigger className="w-24">
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    {AVAILABLE_TIMEFRAMES.map((tf) => (
                      <SelectItem key={tf} value={tf}>
                        {tf}
                      </SelectItem>
                    ))}
                  </SelectContent>
                </Select>
                <AddSymbolDialog onAddSymbol={handleAddSymbol} />
              </div>
            </div>
            {loading ? (
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {[...Array(4)].map((_, i) => (
                  <div
                    key={i}
                    className="h-64 bg-muted/20 rounded-lg animate-pulse"
                  />
                ))}
              </div>
            ) : charts && charts.length > 0 ? (
              <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4 gap-4">
                {charts.map((chart) => (
                  <ChartCard
                    key={`${chart.symbol}-${chart.timeframe}`}
                    chartData={chart}
                    onRemove={handleRemoveSymbol}
                  />
                ))}
              </div>
            ) : (
              <div className="flex flex-col items-center justify-center py-12 text-center">
                <Activity className="h-12 w-12 text-muted-foreground mb-4" />
                <h3 className="text-lg font-semibold mb-2">
                  No Charts Available
                </h3>
                <p className="text-muted-foreground mb-4 max-w-md">
                  Add trading symbols to start monitoring real-time market data
                  and price movements.
                </p>
                <AddSymbolDialog onAddSymbol={handleAddSymbol} />
              </div>
            )}
          </CardContent>
        </Card>
      </div>
    );
  }
);

TradingCharts.displayName = "TradingCharts";
