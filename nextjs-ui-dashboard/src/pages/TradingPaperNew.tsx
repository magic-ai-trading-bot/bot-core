import ErrorBoundary from "@/components/ErrorBoundary";
import { DashboardHeader } from "@/components/dashboard/DashboardHeader";
import logger from "@/utils/logger";
import { PerformanceChart } from "@/components/dashboard/PerformanceChart";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { PremiumButton, PremiumInput } from "@/styles/luxury-design-system";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { usePaperTradingContext, PaperTrade } from "@/contexts/PaperTradingContext";
import { useState, useEffect } from "react";
import { toast } from "sonner";
import {
  Target,
  AlertCircle,
  RefreshCw,
  Play,
  Pause,
  X,
  Wifi,
  WifiOff,
  Clock,
} from "lucide-react";
import ChatBot from "@/components/ChatBot";
import { PortfolioStats } from "@/components/trading/PortfolioStats";
import { RiskMetrics } from "@/components/trading/RiskMetrics";
import { OpenPositionsTable } from "@/components/trading/OpenPositionsTable";
import { ClosedTradesTable } from "@/components/trading/ClosedTradesTable";
import { TradingChartPanel } from "@/components/trading/TradingChartPanel";
import { TradingSettingsPanel } from "@/components/trading/TradingSettingsPanel";
import { SymbolConfig } from "@/components/trading/types";
import { Label } from "@/components/ui/label";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

// API Base URL - using environment variable with fallback
const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

const TradingPaper = () => {
  const {
    portfolio,
    openTrades,
    closedTrades,
    settings,
    recentSignals,
    isActive,
    isLoading,
    error,
    lastUpdated,
    startTrading,
    stopTrading,
    updateSettings,
    resetPortfolio,
    closeTrade,
    refreshAISignals,
    refreshSettings,
  } = usePaperTradingContext();

  const [wsConnected, setWsConnected] = useState(true);
  const [currentTime, setCurrentTime] = useState(new Date());
  const [lastUpdateCount, setLastUpdateCount] = useState(0);
  const [selectedTradeId, setSelectedTradeId] = useState<string | null>(null);
  const [isTradeDetailOpen, setIsTradeDetailOpen] = useState(false);
  const [settingsForm, setSettingsForm] = useState(settings);
  const [showReset, setShowReset] = useState(false);
  const [showSymbolDialog, setShowSymbolDialog] = useState(false);
  const [symbolSettings, setSymbolSettings] = useState<{
    [key: string]: SymbolConfig;
  }>({});
  const [isLoadingSymbols, setIsLoadingSymbols] = useState(false);
  const [lastSelectedTradePnl, setLastSelectedTradePnl] = useState<number | null>(null);

  const trades = [...openTrades, ...closedTrades];
  const selectedTrade = selectedTradeId
    ? trades.find((trade) => trade.id === selectedTradeId) || null
    : null;

  useEffect(() => {
    if (lastUpdated && wsConnected) {
      setLastUpdateCount((prev) => prev + 1);
    }
  }, [lastUpdated, wsConnected]);

  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(new Date());
      if (lastUpdated) {
        const timeSinceUpdate = Date.now() - lastUpdated.getTime();
        setWsConnected(timeSinceUpdate < 30000);
      }
    }, 1000);
    return () => clearInterval(interval);
  }, [lastUpdated, wsConnected]);

  useEffect(() => {
    setSettingsForm(settings);
  }, [settings]);

  useEffect(() => {
    if (selectedTradeId && !selectedTrade && isTradeDetailOpen) {
      setIsTradeDetailOpen(false);
      setSelectedTradeId(null);
      toast.info("Giao dịch đã được đóng");
    }
  }, [selectedTradeId, selectedTrade, isTradeDetailOpen]);

  useEffect(() => {
    if (selectedTrade && isTradeDetailOpen) {
      const currentPnl = selectedTrade.pnl || 0;
      if (
        lastSelectedTradePnl !== null &&
        Math.abs(currentPnl - lastSelectedTradePnl) > 1
      ) {
        const change = currentPnl - lastSelectedTradePnl;
        toast.info(`${selectedTrade.symbol} P&L Updated`, {
          description: `${change > 0 ? "↗️" : "↘️"} ${formatCurrency(change)}`,
          duration: 2000,
        });
      }
      setLastSelectedTradePnl(currentPnl);
    } else if (!isTradeDetailOpen) {
      setLastSelectedTradePnl(null);
    }
  }, [selectedTrade, isTradeDetailOpen, lastSelectedTradePnl]);

  const togglePaperTrading = async (active: boolean) => {
    try {
      if (active) {
        await startTrading();
        toast.success("Bot trading đã được khởi động!");
      } else {
        await stopTrading();
        toast.success("Bot trading đã được dừng!");
      }
    } catch (error) {
      logger.error("Failed to toggle paper trading:", error);
      toast.error(`Lỗi khi ${active ? "khởi động" : "dừng"} bot`);
    }
  };

  const openTradeDetails = (trade: PaperTrade) => {
    setSelectedTradeId(trade.id);
    setIsTradeDetailOpen(true);
  };

  const calculatePositionValue = (trade: PaperTrade) => trade.quantity * trade.entry_price;
  const calculatePositionSize = (trade: PaperTrade) => calculatePositionValue(trade);
  const calculateMarginRequired = (trade: PaperTrade) => calculatePositionValue(trade) / trade.leverage;

  const fetchAISignals = async () => {
    try {
      await refreshAISignals();
    } catch (error) {
      logger.error("Failed to refresh AI signals:", error);
      toast.error("Lỗi khi cập nhật tín hiệu AI");
    }
  };

  const loadSymbolSettings = async () => {
    try {
      setIsLoadingSymbols(true);
      const response = await fetch(`${API_BASE}/api/paper-trading/symbols`);
      const data = await response.json();
      if (data.success && data.data) {
        setSymbolSettings(data.data);
      } else {
        throw new Error(data.error || "Failed to load symbol settings");
      }
    } catch (error) {
      logger.error("Failed to load symbol settings:", error);
      toast.error("Lỗi khi tải cài đặt symbols");
    } finally {
      setIsLoadingSymbols(false);
    }
  };

  const updateSymbolSettings = async () => {
    try {
      setIsLoadingSymbols(true);
      const response = await fetch(`${API_BASE}/api/paper-trading/symbols`, {
        method: "PUT",
        headers: { "Content-Type": "application/json" },
        body: JSON.stringify({ symbols: symbolSettings }),
      });
      const data = await response.json();
      if (data.success) {
        toast.success("Cài đặt symbols đã được lưu thành công!");
      } else {
        throw new Error(data.error || "Failed to update symbol settings");
      }
    } catch (error) {
      logger.error("Failed to update symbol settings:", error);
      toast.error("Lỗi khi lưu cài đặt symbols");
    } finally {
      setIsLoadingSymbols(false);
    }
  };

  const handleTabChange = async (value: string) => {
    if (value === "settings") {
      try {
        await refreshSettings();
        await loadSymbolSettings();
      } catch (error) {
        logger.error("Failed to refresh settings:", error);
        toast.error("Lỗi khi tải cài đặt");
      }
    }
  };

  const handleSettingsSubmit = async () => {
    try {
      await updateSettings(settingsForm);
      toast.success("Cài đặt đã được lưu thành công!");
    } catch (error) {
      logger.error("Failed to update settings:", error);
      toast.error("Lỗi khi lưu cài đặt");
    }
  };

  const handleReset = async () => {
    try {
      await resetPortfolio();
      setShowReset(false);
      toast.success("Portfolio đã được reset thành công!");
    } catch (error) {
      logger.error("Failed to reset portfolio:", error);
      toast.error("Lỗi khi reset portfolio");
      setShowReset(false);
    }
  };

  const formatCurrency = (value: number) =>
    new Intl.NumberFormat("vi-VN", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);

  const formatPercentage = (value: number | undefined) => {
    if (value === undefined || value === null || isNaN(value)) return "0.00%";
    return `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
  };

  const formatDate = (date: Date | string | number) => {
    try {
      const dateObj = date instanceof Date ? date : new Date(date);
      if (isNaN(dateObj.getTime())) return "N/A";
      return new Intl.DateTimeFormat("vi-VN", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      }).format(dateObj);
    } catch (error) {
      logger.error("Invalid date:", date, error);
      return "N/A";
    }
  };

  return (
    <ErrorBoundary>
      <div className="min-h-screen bg-background">
        <DashboardHeader />
        <div className="p-4 lg:p-6">
          <div className="mb-4 lg:mb-6">
            <div className="flex items-center justify-between">
              <div>
                <h1 className="text-2xl lg:text-3xl font-bold flex items-center gap-2">
                  <Target className="h-6 w-6 text-primary" />
                  Trading Paper
                </h1>
                <p className="text-muted-foreground text-sm lg:text-base">
                  Mô phỏng giao dịch với AI Bot - Kiểm thử chiến lược không rủi ro
                </p>
              </div>
              <div className="flex flex-col items-end gap-2">
                <div className="flex items-center gap-3 text-xs">
                  <div className="flex items-center gap-1">
                    {wsConnected ? (
                      <Wifi className="h-3 w-3 text-green-500" />
                    ) : (
                      <WifiOff className="h-3 w-3 text-red-500" />
                    )}
                    <span className={wsConnected ? "text-green-600" : "text-red-600"}>
                      {wsConnected ? "WebSocket Connected" : "WebSocket Disconnected"}
                    </span>
                  </div>
                  <div className="flex items-center gap-1 text-muted-foreground">
                    <Clock className="h-3 w-3" />
                    <span>{currentTime.toLocaleTimeString("vi-VN")}</span>
                  </div>
                  {lastUpdated && (
                    <div className="text-muted-foreground">
                      Last update: {Math.floor((currentTime.getTime() - lastUpdated.getTime()) / 1000)}s ago
                    </div>
                  )}
                </div>
                <div className="flex items-center gap-4">
                  <Badge variant={isActive ? "default" : "secondary"} className={`text-sm flex items-center gap-1 ${isActive ? "animate-pulse" : ""}`}>
                    <div className={`w-2 h-2 rounded-full ${isActive ? "bg-green-500" : "bg-gray-400"}`}></div>
                    {isActive ? "Đang hoạt động" : "Tạm dừng"}
                  </Badge>
                  <PremiumButton onClick={() => togglePaperTrading(!isActive)} variant={isActive ? "danger" : "primary"} size="sm" disabled={isLoading}>
                    {isLoading ? <RefreshCw className="h-4 w-4 mr-2 animate-spin" /> : isActive ? <Pause className="h-4 w-4 mr-2" /> : <Play className="h-4 w-4 mr-2" />}
                    {isActive ? "Dừng Bot" : "Khởi động Bot"}
                  </PremiumButton>
                  <PremiumButton onClick={fetchAISignals} variant="secondary" size="sm" disabled={isLoading}>
                    <RefreshCw className={`h-4 w-4 mr-2 ${isLoading ? "animate-spin" : ""}`} />
                    Cập nhật
                  </PremiumButton>
                </div>
              </div>
            </div>
          </div>

          {error && (
            <Alert className="mb-4 border-destructive">
              <AlertCircle className="h-4 w-4" />
              <AlertDescription className="flex items-center justify-between">
                <span>{error}</span>
                <PremiumButton variant="ghost" size="sm"><X className="h-4 w-4" /></PremiumButton>
              </AlertDescription>
            </Alert>
          )}

          <Tabs defaultValue="overview" className="space-y-4 lg:space-y-6" onValueChange={handleTabChange}>
            <TabsList className="grid w-full grid-cols-2 lg:grid-cols-4 gap-1">
              <TabsTrigger value="overview">Tổng quan</TabsTrigger>
              <TabsTrigger value="signals">Tín hiệu AI</TabsTrigger>
              <TabsTrigger value="trades">Lịch sử giao dịch</TabsTrigger>
              <TabsTrigger value="settings">Cài đặt</TabsTrigger>
            </TabsList>

            <TabsContent value="overview" className="space-y-4 lg:space-y-6">
              <PortfolioStats
                portfolio={portfolio}
                openTrades={openTrades}
                closedTrades={closedTrades}
                wsConnected={wsConnected}
                calculatePositionSize={calculatePositionSize}
                calculateMarginRequired={calculateMarginRequired}
                formatCurrency={formatCurrency}
                formatPercentage={formatPercentage}
              />
              <RiskMetrics
                portfolio={portfolio}
                openTrades={openTrades}
                closedTrades={closedTrades}
                calculateMarginRequired={calculateMarginRequired}
                formatCurrency={formatCurrency}
                formatPercentage={formatPercentage}
              />
              <PerformanceChart />
            </TabsContent>

            <TabsContent value="signals" className="space-y-4 lg:space-y-6">
              <TradingChartPanel
                recentSignals={recentSignals}
                isLoading={isLoading}
                formatDate={formatDate}
                refreshAISignals={fetchAISignals}
              />
            </TabsContent>

            <TabsContent value="trades" className="space-y-4 lg:space-y-6">
              <OpenPositionsTable
                openTrades={openTrades}
                wsConnected={wsConnected}
                calculatePositionSize={calculatePositionSize}
                calculateMarginRequired={calculateMarginRequired}
                formatCurrency={formatCurrency}
                formatDate={formatDate}
                openTradeDetails={openTradeDetails}
                closeTrade={closeTrade}
              />
              <ClosedTradesTable
                closedTrades={closedTrades}
                wsConnected={wsConnected}
                formatCurrency={formatCurrency}
                formatPercentage={formatPercentage}
                openTradeDetails={openTradeDetails}
              />
            </TabsContent>

            <TabsContent value="settings" className="space-y-4 lg:space-y-6">
              <TradingSettingsPanel
                settingsForm={settingsForm}
                setSettingsForm={setSettingsForm}
                handleSettingsSubmit={handleSettingsSubmit}
                handleReset={handleReset}
                showReset={showReset}
                setShowReset={setShowReset}
                isLoading={isLoading}
                symbolSettings={symbolSettings}
                setShowSymbolDialog={setShowSymbolDialog}
              />
            </TabsContent>
          </Tabs>

          {wsConnected && (
            <div className="mt-6 p-3 bg-green-50 border border-green-200 rounded-lg">
              <div className="flex items-center justify-between text-sm">
                <div className="flex items-center gap-4">
                  <div className="flex items-center gap-2">
                    <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                    <span className="text-green-700 font-medium">WebSocket Active</span>
                  </div>
                  <div className="text-green-600">Real-time updates: {lastUpdateCount}</div>
                  <div className="text-green-600">Last sync: {lastUpdated?.toLocaleTimeString("vi-VN") || "Never"}</div>
                </div>
                <div className="text-green-600">Data refreshes automatically every second 🚀</div>
              </div>
            </div>
          )}
        </div>

        {/* Symbol Configuration Dialog - Keeping inline for now due to complexity */}
        <Dialog open={showSymbolDialog} onOpenChange={setShowSymbolDialog}>
          <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
            <DialogHeader>
              <DialogTitle className="flex items-center gap-2">
                <Target className="h-5 w-5" />
                Cài đặt Symbols
                <Badge variant="secondary" className="text-xs">{Object.keys(symbolSettings).length} symbols</Badge>
              </DialogTitle>
            </DialogHeader>
            <div className="space-y-6 py-4">
              {isLoadingSymbols ? (
                <div className="flex items-center justify-center p-8">
                  <RefreshCw className="h-6 w-6 animate-spin mr-2" />
                  <span>Đang tải cài đặt symbols...</span>
                </div>
              ) : Object.keys(symbolSettings).length > 0 ? (
                <>
                  {Object.entries(symbolSettings).map(([symbol, config]) => (
                    <Card key={symbol} className="border border-muted">
                      <CardHeader className="pb-3">
                        <div className="flex items-center justify-between">
                          <CardTitle className="text-lg font-semibold">{symbol.replace("USDT", "/USDT")}</CardTitle>
                          <div className="flex items-center gap-2">
                            <Badge variant={config.enabled ? "default" : "secondary"}>{config.enabled ? "Bật" : "Tắt"}</Badge>
                            <Label htmlFor={`enabled-${symbol}`} className="text-sm">Kích hoạt</Label>
                            <input
                              type="checkbox"
                              id={`enabled-${symbol}`}
                              checked={config.enabled}
                              onChange={(e) => setSymbolSettings((prev) => ({ ...prev, [symbol]: { ...prev[symbol], enabled: e.target.checked } }))}
                              className="h-4 w-4"
                            />
                          </div>
                        </div>
                      </CardHeader>
                      <CardContent className="space-y-4">
                        <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                          <div className="space-y-2">
                            <Label htmlFor={`leverage-${symbol}`}>Đòn bẩy</Label>
                            <PremiumInput
                              id={`leverage-${symbol}`}
                              type="number"
                              min="1"
                              max="50"
                              value={config.leverage}
                              onChange={(e) => setSymbolSettings((prev) => ({ ...prev, [symbol]: { ...prev[symbol], leverage: parseInt(e.target.value) || 1 } }))}
                            />
                          </div>
                          <div className="space-y-2">
                            <Label htmlFor={`position-size-${symbol}`}>Kích thước vị thế (%)</Label>
                            <PremiumInput
                              id={`position-size-${symbol}`}
                              type="number"
                              min="0.1"
                              max="100"
                              step="0.1"
                              value={config.position_size_pct}
                              onChange={(e) => setSymbolSettings((prev) => ({ ...prev, [symbol]: { ...prev[symbol], position_size_pct: parseFloat(e.target.value) || 0 } }))}
                            />
                          </div>
                          <div className="space-y-2">
                            <Label htmlFor={`max-positions-${symbol}`}>Số vị thế tối đa</Label>
                            <PremiumInput
                              id={`max-positions-${symbol}`}
                              type="number"
                              min="1"
                              max="10"
                              value={config.max_positions}
                              onChange={(e) => setSymbolSettings((prev) => ({ ...prev, [symbol]: { ...prev[symbol], max_positions: parseInt(e.target.value) || 1 } }))}
                            />
                          </div>
                        </div>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                          <div className="space-y-2">
                            <Label htmlFor={`stop-loss-${symbol}`}>Stop Loss (%)</Label>
                            <PremiumInput
                              id={`stop-loss-${symbol}`}
                              type="number"
                              min="0.1"
                              max="50"
                              step="0.1"
                              value={config.stop_loss_pct}
                              onChange={(e) => setSymbolSettings((prev) => ({ ...prev, [symbol]: { ...prev[symbol], stop_loss_pct: parseFloat(e.target.value) || 0 } }))}
                            />
                          </div>
                          <div className="space-y-2">
                            <Label htmlFor={`take-profit-${symbol}`}>Take Profit (%)</Label>
                            <PremiumInput
                              id={`take-profit-${symbol}`}
                              type="number"
                              min="0.1"
                              max="100"
                              step="0.1"
                              value={config.take_profit_pct}
                              onChange={(e) => setSymbolSettings((prev) => ({ ...prev, [symbol]: { ...prev[symbol], take_profit_pct: parseFloat(e.target.value) || 0 } }))}
                            />
                          </div>
                        </div>
                      </CardContent>
                    </Card>
                  ))}
                  <div className="flex gap-4 pt-4">
                    <PremiumButton onClick={updateSymbolSettings} className="flex-1" disabled={isLoadingSymbols}>
                      {isLoadingSymbols ? <><RefreshCw className="h-4 w-4 mr-2 animate-spin" />Đang lưu...</> : "Lưu cài đặt Symbols"}
                    </PremiumButton>
                    <PremiumButton variant="secondary" onClick={loadSymbolSettings} disabled={isLoadingSymbols}>
                      <RefreshCw className="h-4 w-4 mr-2" />Tải lại
                    </PremiumButton>
                  </div>
                </>
              ) : (
                <div className="flex items-center justify-center p-8">
                  <div className="text-center">
                    <Target className="h-8 w-8 mx-auto mb-2 opacity-50" />
                    <p className="text-muted-foreground">Chưa có cài đặt symbols</p>
                    <PremiumButton variant="secondary" onClick={loadSymbolSettings} className="mt-2">Tải cài đặt</PremiumButton>
                  </div>
                </div>
              )}
            </div>
          </DialogContent>
        </Dialog>

        <ChatBot />
      </div>
    </ErrorBoundary>
  );
};

export default TradingPaper;
