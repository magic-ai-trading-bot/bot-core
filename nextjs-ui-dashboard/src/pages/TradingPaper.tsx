import { DashboardHeader } from "@/components/dashboard/DashboardHeader";
import { TradingSettings } from "@/components/dashboard/TradingSettings";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Separator } from "@/components/ui/separator";
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from "@/components/ui/table";
import { Alert, AlertDescription } from "@/components/ui/alert";
import {
  Dialog,
  DialogContent,
  DialogHeader,
  DialogTitle,
  DialogDescription,
} from "@/components/ui/dialog";
import { usePaperTrading, PaperTrade } from "@/hooks/usePaperTrading";
import { useState, useEffect } from "react";
import { toast } from "sonner";
import {
  TrendingUp,
  TrendingDown,
  Activity,
  DollarSign,
  Target,
  History,
  AlertCircle,
  RefreshCw,
  Play,
  Pause,
  RotateCcw,
  X,
  Settings,
  Wifi,
  WifiOff,
  Clock,
} from "lucide-react";

// Symbol configuration interface
interface SymbolConfig {
  enabled: boolean;
  leverage: number;
  position_size_pct: number;
  stop_loss_pct: number;
  take_profit_pct: number;
  max_positions: number;
}
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

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
  } = usePaperTrading();

  // WebSocket status - simulated based on last update time
  const [wsConnected, setWsConnected] = useState(true);
  const [currentTime, setCurrentTime] = useState(new Date());
  const [lastUpdateCount, setLastUpdateCount] = useState(0);

  // Show real-time update notifications
  useEffect(() => {
    if (lastUpdated && wsConnected) {
      setLastUpdateCount((prev) => {
        const newCount = prev + 1;
        // Removed spam sync notification
        return newCount;
      });
    }
  }, [lastUpdated, wsConnected]);

  // Update current time every second for real-time feel
  useEffect(() => {
    const interval = setInterval(() => {
      setCurrentTime(new Date());

      // Consider WebSocket disconnected if no updates for >30 seconds
      if (lastUpdated) {
        const timeSinceUpdate = Date.now() - lastUpdated.getTime();
        const newWsConnected = timeSinceUpdate < 30000;

        // Show toast when connection status changes
        if (wsConnected !== newWsConnected) {
          if (newWsConnected) {
            toast.success("WebSocket reconnected!", {
              description: "Real-time updates resumed",
              duration: 2000,
            });
          } else {
            toast.warning("WebSocket disconnected", {
              description: "Real-time updates paused",
              duration: 3000,
            });
          }
        }

        setWsConnected(newWsConnected);
      }
    }, 1000);

    return () => clearInterval(interval);
  }, [lastUpdated, wsConnected]);

  // State for trade details popup
  const [selectedTrade, setSelectedTrade] = useState<PaperTrade | null>(null);
  const [isTradeDetailOpen, setIsTradeDetailOpen] = useState(false);

  // Additional properties and functions
  const trades = [...openTrades, ...closedTrades];

  const togglePaperTrading = async (active: boolean) => {
    try {
      if (active) {
        await startTrading();
        toast.success("Bot trading đã được khởi động!", {
          description:
            "Paper trading bot hiện đang hoạt động và sẵn sàng thực hiện giao dịch",
          duration: 3000,
        });
      } else {
        await stopTrading();
        toast.success("Bot trading đã được dừng!", {
          description: "Paper trading bot đã dừng hoạt động",
          duration: 3000,
        });
      }
    } catch (error) {
      console.error("Failed to toggle paper trading:", error);
      toast.error(`Lỗi khi ${active ? "khởi động" : "dừng"} bot`, {
        description: "Có lỗi xảy ra. Vui lòng thử lại.",
        duration: 4000,
      });
    }
  };

  const resetPaperTrading = () => resetPortfolio();
  const clearError = () => {
    // Only paper trading errors now
  };

  // Open trade details popup
  const openTradeDetails = (trade: PaperTrade) => {
    setSelectedTrade(trade);
    setIsTradeDetailOpen(true);
  };

  // Calculate position value and P&L
  const calculatePositionValue = (trade: PaperTrade) => {
    return trade.quantity * trade.entry_price;
  };

  const calculatePositionSize = (trade: PaperTrade) => {
    return calculatePositionValue(trade); // Position Size = Entry Price × Quantity (notional value)
  };

  const calculateMarginRequired = (trade: PaperTrade) => {
    return calculatePositionValue(trade) / trade.leverage; // Margin = Position Size / Leverage
  };

  const fetchAISignals = async () => {
    try {
      await refreshAISignals();
      toast.success("Tín hiệu AI đã được cập nhật!", {
        description: "Dữ liệu tín hiệu mới nhất đã được tải về",
        duration: 2000,
      });
    } catch (error) {
      console.error("Failed to refresh AI signals:", error);
      toast.error("Lỗi khi cập nhật tín hiệu AI", {
        description: "Không thể tải tín hiệu mới. Vui lòng thử lại.",
        duration: 3000,
      });
    }
  };

  const [settingsForm, setSettingsForm] = useState(settings);
  const [showReset, setShowReset] = useState(false);
  const [showSymbolDialog, setShowSymbolDialog] = useState(false);

  // Add symbol settings state
  const [symbolSettings, setSymbolSettings] = useState<{
    [key: string]: SymbolConfig;
  }>({});
  const [isLoadingSymbols, setIsLoadingSymbols] = useState(false);

  // Update settings form when settings change
  useEffect(() => {
    setSettingsForm(settings);
  }, [settings]);

  // Load symbol settings
  const loadSymbolSettings = async () => {
    try {
      setIsLoadingSymbols(true);
      const response = await fetch(
        "http://localhost:8080/api/paper-trading/symbols"
      );
      const data = await response.json();

      if (data.success && data.data) {
        setSymbolSettings(data.data);
        toast.success("Cài đặt symbols đã được tải!", {
          description: "Hiển thị cài đặt hiện tại cho từng symbol",
          duration: 2000,
        });
      } else {
        throw new Error(data.error || "Failed to load symbol settings");
      }
    } catch (error) {
      console.error("Failed to load symbol settings:", error);
      toast.error("Lỗi khi tải cài đặt symbols", {
        description: "Không thể tải cài đặt symbols. Sử dụng giá trị mặc định.",
        duration: 3000,
      });
    } finally {
      setIsLoadingSymbols(false);
    }
  };

  // Update symbol settings
  const updateSymbolSettings = async () => {
    try {
      setIsLoadingSymbols(true);
      const response = await fetch(
        "http://localhost:8080/api/paper-trading/symbols",
        {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            symbols: symbolSettings,
          }),
        }
      );

      const data = await response.json();

      if (data.success) {
        toast.success("Cài đặt symbols đã được lưu thành công!", {
          description: `Đã cập nhật ${data.data.updated_symbols.join(", ")}`,
          duration: 4000,
        });
      } else {
        throw new Error(data.error || "Failed to update symbol settings");
      }
    } catch (error) {
      console.error("Failed to update symbol settings:", error);
      toast.error("Lỗi khi lưu cài đặt symbols", {
        description:
          error instanceof Error
            ? error.message
            : "Có lỗi xảy ra khi lưu cài đặt symbols.",
        duration: 5000,
      });
    } finally {
      setIsLoadingSymbols(false);
    }
  };

  // Load fresh settings when opening settings tab
  const handleTabChange = async (value: string) => {
    if (value === "settings") {
      try {
        await refreshSettings(); // Load latest settings from backend
        await loadSymbolSettings(); // Load symbol settings
        toast.success("Cài đặt đã được tải về!", {
          description: "Hiển thị cài đặt hiện tại từ hệ thống",
          duration: 2000,
        });
      } catch (error) {
        console.error("Failed to refresh settings:", error);
        toast.error("Lỗi khi tải cài đặt", {
          description: "Không thể tải cài đặt hiện tại. Hiển thị dữ liệu cũ.",
          duration: 3000,
        });
      }
    }
  };

  const handleSettingsSubmit = async () => {
    try {
      await updateSettings(settingsForm);
      // Settings will be automatically refreshed via the hook
      toast.success("Cài đặt đã được lưu thành công!", {
        description:
          "Tất cả thay đổi đã được áp dụng và portfolio đã được cập nhật",
        duration: 4000,
      });
    } catch (error) {
      console.error("Failed to update settings:", error);
      toast.error("Lỗi khi lưu cài đặt", {
        description:
          error instanceof Error
            ? error.message
            : "Có lỗi xảy ra khi lưu cài đặt. Vui lòng thử lại.",
        duration: 5000,
      });
    }
  };

  const handleReset = async () => {
    try {
      await resetPaperTrading();
      setShowReset(false);
      toast.success("Portfolio đã được reset thành công!", {
        description:
          "Tất cả dữ liệu giao dịch đã được xóa và portfolio được khôi phục về trạng thái ban đầu",
        duration: 4000,
      });
    } catch (error) {
      console.error("Failed to reset portfolio:", error);
      toast.error("Lỗi khi reset portfolio", {
        description: "Có lỗi xảy ra khi reset dữ liệu. Vui lòng thử lại.",
        duration: 5000,
      });
      setShowReset(false);
    }
  };

  const formatCurrency = (value: number) => {
    return new Intl.NumberFormat("vi-VN", {
      style: "currency",
      currency: "USD",
      minimumFractionDigits: 2,
      maximumFractionDigits: 2,
    }).format(value);
  };

  const formatPercentage = (value: number | undefined) => {
    if (value === undefined || value === null || isNaN(value)) {
      return "0.00%";
    }
    return `${value >= 0 ? "+" : ""}${value.toFixed(2)}%`;
  };

  const formatDate = (date: Date | string | number) => {
    try {
      const dateObj = date instanceof Date ? date : new Date(date);

      // Check if date is valid
      if (isNaN(dateObj.getTime())) {
        return "N/A";
      }

      return new Intl.DateTimeFormat("vi-VN", {
        year: "numeric",
        month: "2-digit",
        day: "2-digit",
        hour: "2-digit",
        minute: "2-digit",
      }).format(dateObj);
    } catch (error) {
      console.error("Invalid date:", date, error);
      return "N/A";
    }
  };

  // Generate performance chart data with more detailed timeline
  const generateChartData = () => {
    // Create timeline for the last 24 hours
    const data = [];
    const now = new Date();
    const startDate = new Date(now.getTime() - 24 * 60 * 60 * 1000); // 24 hours ago

    // Add initial balance (ensure reasonable values)
    const initialBalance = settings.basic.initial_balance || 10000;
    data.push({
      date: "24h trước",
      balance: initialBalance,
      pnl: 0,
      equity: initialBalance,
      marginUsed: 0,
      totalPositions: 0,
      timestamp: startDate.toISOString(),
    });

    // Generate 8 data points over 24 hours (every 3 hours)
    for (let i = 1; i <= 8; i++) {
      const hoursAgo = 24 - i * 3;
      const currentTime = new Date(now.getTime() - hoursAgo * 60 * 60 * 1000);
      const progressRatio = Math.min(i / 8, 1);

      // Simulate gradual portfolio changes
      let simulatedBalance = initialBalance;
      let simulatedPnl = 0;

      // Add some realistic market movement simulation
      if (openTrades.length > 0) {
        // Create a realistic P&L progression
        simulatedPnl = (portfolio.total_pnl || 0) * progressRatio;
        simulatedBalance = initialBalance + simulatedPnl;

        // Add small market volatility (max ±0.5% of balance)
        const volatility = Math.sin(i * 0.5) * 0.005; // 0.5% max volatility
        const volatilityAmount = initialBalance * volatility;
        simulatedBalance += volatilityAmount;
        simulatedPnl += volatilityAmount;
      } else {
        // If no trades, add small random movement around initial balance
        const smallMovement = Math.sin(i * 0.3) * 25; // ±$25 movement
        simulatedBalance = initialBalance + smallMovement;
        simulatedPnl = smallMovement;
      }

      // Create simple time labels
      let timeLabel;
      if (hoursAgo <= 0) {
        timeLabel = "Hiện tại";
      } else {
        timeLabel = `${hoursAgo}h trước`;
      }

      data.push({
        date: timeLabel,
        balance: simulatedBalance,
        pnl: simulatedPnl,
        equity: simulatedBalance,
        marginUsed: portfolio.margin_used * progressRatio,
        totalPositions: Math.floor(openTrades.length * progressRatio),
        timestamp: currentTime.toISOString(),
      });
    }

    // Add closed trades if any
    if (closedTrades.length > 0) {
      const sortedTrades = [...closedTrades].sort(
        (a, b) =>
          new Date(a.close_time!).getTime() - new Date(b.close_time!).getTime()
      );

      let runningBalance = initialBalance;

      sortedTrades.forEach((trade, index) => {
        runningBalance += trade.pnl || 0;
        const tradeDate = new Date(trade.close_time!);

        data.push({
          date: `T${index + 1}`,
          balance: runningBalance,
          pnl: trade.pnl || 0,
          equity: runningBalance,
          marginUsed: 0,
          totalPositions: 0,
          timestamp: tradeDate.toISOString(),
          tradeEvent: true,
        });
      });
    }

    // Always add current state (ensure values are reasonable)
    data.push({
      date: "Hiện tại",
      balance: Math.max(portfolio.equity || initialBalance, 0),
      pnl: portfolio.total_pnl || 0,
      equity: Math.max(portfolio.equity || initialBalance, 0),
      marginUsed: Math.max(portfolio.margin_used || 0, 0),
      totalPositions: openTrades.length || 0,
      timestamp: now.toISOString(),
      isCurrent: true,
    });

    // Sort by timestamp for correct order
    return data.sort(
      (a, b) =>
        new Date(a.timestamp).getTime() - new Date(b.timestamp).getTime()
    );
  };

  const chartData = generateChartData();

  return (
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
              {/* Real-time Status Indicators */}
              <div className="flex items-center gap-3 text-xs">
                <div className="flex items-center gap-1">
                  {wsConnected ? (
                    <Wifi className="h-3 w-3 text-green-500" />
                  ) : (
                    <WifiOff className="h-3 w-3 text-red-500" />
                  )}
                  <span
                    className={wsConnected ? "text-green-600" : "text-red-600"}
                  >
                    {wsConnected
                      ? "WebSocket Connected"
                      : "WebSocket Disconnected"}
                  </span>
                </div>
                <div className="flex items-center gap-1 text-muted-foreground">
                  <Clock className="h-3 w-3" />
                  <span>
                    {currentTime.toLocaleTimeString("vi-VN", {
                      hour: "2-digit",
                      minute: "2-digit",
                      second: "2-digit",
                    })}
                  </span>
                </div>
                {lastUpdated && (
                  <div className="text-muted-foreground">
                    Last update:{" "}
                    {Math.floor(
                      (currentTime.getTime() - lastUpdated.getTime()) / 1000
                    )}
                    s ago
                  </div>
                )}
              </div>

              {/* Control Buttons */}
              <div className="flex items-center gap-4">
                <Badge
                  variant={isActive ? "default" : "secondary"}
                  className={`text-sm flex items-center gap-1 ${
                    isActive ? "animate-pulse" : ""
                  }`}
                >
                  <div
                    className={`w-2 h-2 rounded-full ${
                      isActive ? "bg-green-500" : "bg-gray-400"
                    }`}
                  ></div>
                  {isActive ? "Đang hoạt động" : "Tạm dừng"}
                </Badge>
                <Button
                  onClick={() => togglePaperTrading(!isActive)}
                  variant={isActive ? "destructive" : "default"}
                  size="sm"
                  disabled={isLoading}
                >
                  {isLoading ? (
                    <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                  ) : isActive ? (
                    <Pause className="h-4 w-4 mr-2" />
                  ) : (
                    <Play className="h-4 w-4 mr-2" />
                  )}
                  {isActive ? "Dừng Bot" : "Khởi động Bot"}
                </Button>
                <Button
                  onClick={fetchAISignals}
                  variant="outline"
                  size="sm"
                  disabled={isLoading}
                >
                  <RefreshCw
                    className={`h-4 w-4 mr-2 ${
                      isLoading ? "animate-spin" : ""
                    }`}
                  />
                  Cập nhật
                </Button>
              </div>
            </div>
          </div>
        </div>

        {error && (
          <Alert className="mb-4 border-destructive">
            <AlertCircle className="h-4 w-4" />
            <AlertDescription className="flex items-center justify-between">
              <span>{error}</span>
              <Button variant="ghost" size="sm" onClick={clearError}>
                <X className="h-4 w-4" />
              </Button>
            </AlertDescription>
          </Alert>
        )}

        <Tabs
          defaultValue="overview"
          className="space-y-4 lg:space-y-6"
          onValueChange={handleTabChange}
        >
          <TabsList className="grid w-full grid-cols-2 lg:grid-cols-4 gap-1">
            <TabsTrigger value="overview" className="text-xs lg:text-sm">
              Tổng quan
            </TabsTrigger>
            <TabsTrigger value="signals" className="text-xs lg:text-sm">
              Tín hiệu AI
            </TabsTrigger>
            <TabsTrigger value="trades" className="text-xs lg:text-sm">
              Lịch sử giao dịch
            </TabsTrigger>
            <TabsTrigger value="settings" className="text-xs lg:text-sm">
              Cài đặt
            </TabsTrigger>
          </TabsList>

          {/* Overview Tab */}
          <TabsContent value="overview" className="space-y-4 lg:space-y-6">
            {/* Portfolio Overview */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Số dư hiện tại
                  </CardTitle>
                  <DollarSign className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">
                    {formatCurrency(portfolio.current_balance)}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Equity: {formatCurrency(portfolio.equity)}
                  </p>
                </CardContent>
              </Card>

              <Card
                className={
                  wsConnected && portfolio.total_pnl !== 0
                    ? "animate-pulse"
                    : ""
                }
              >
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium flex items-center gap-1">
                    Tổng P&L
                    {wsConnected && (
                      <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                    )}
                  </CardTitle>
                  <TrendingUp
                    className={`h-4 w-4 ${
                      portfolio.total_pnl >= 0 ? "text-profit" : "text-loss"
                    }`}
                  />
                </CardHeader>
                <CardContent>
                  <div
                    className={`text-2xl font-bold ${
                      portfolio.total_pnl >= 0 ? "text-profit" : "text-loss"
                    }`}
                  >
                    {portfolio.total_pnl >= 0 ? "+" : ""}
                    {formatCurrency(portfolio.total_pnl)}
                  </div>
                  <p className="text-xs text-muted-foreground">
                    {formatPercentage(portfolio.total_pnl_percentage)}
                    {wsConnected && " • Live"}
                  </p>
                </CardContent>
              </Card>

              <Card
                className={
                  wsConnected && openTrades.length > 0 ? "border-green-200" : ""
                }
              >
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium flex items-center gap-1">
                    Tổng số lệnh
                    {wsConnected && openTrades.length > 0 && (
                      <div className="w-2 h-2 bg-blue-500 rounded-full animate-pulse"></div>
                    )}
                  </CardTitle>
                  <Activity
                    className={`h-4 w-4 ${
                      wsConnected && openTrades.length > 0
                        ? "text-blue-500 animate-pulse"
                        : "text-muted-foreground"
                    }`}
                  />
                </CardHeader>
                <CardContent>
                  <div className="text-2xl font-bold">
                    {portfolio.total_trades}
                  </div>
                  <div className="space-y-1">
                    <p className="text-xs text-muted-foreground">
                      Đang mở: {openTrades.length} • Đã đóng:{" "}
                      {closedTrades.length}
                    </p>
                    <div className="text-xs space-y-1">
                      <div>
                        <span className="text-muted-foreground">
                          Position Size:{" "}
                        </span>
                        <span className="font-medium text-primary">
                          {formatCurrency(
                            openTrades.reduce(
                              (total, trade) =>
                                total + calculatePositionSize(trade),
                              0
                            )
                          )}
                        </span>
                      </div>
                      <div>
                        <span className="text-muted-foreground">
                          Margin Used:{" "}
                        </span>
                        <span className="font-medium text-warning">
                          {formatCurrency(
                            openTrades.reduce(
                              (total, trade) =>
                                total + calculateMarginRequired(trade),
                              0
                            )
                          )}
                        </span>
                      </div>
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    {closedTrades.length > 0 ? "Tỷ lệ thắng" : "Margin Usage"}
                  </CardTitle>
                  <Target className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  {closedTrades.length > 0 ? (
                    <>
                      <div className="text-2xl font-bold">
                        {portfolio.win_rate.toFixed(1)}%
                      </div>
                      <p className="text-xs text-muted-foreground">
                        {Math.round(
                          (portfolio.win_rate * portfolio.total_trades) / 100
                        )}
                        /{portfolio.total_trades}
                      </p>
                    </>
                  ) : (
                    <>
                      <div className="text-2xl font-bold text-warning">
                        {(
                          (portfolio.margin_used / portfolio.equity) *
                          100
                        ).toFixed(1)}
                        %
                      </div>
                      <p className="text-xs text-muted-foreground">
                        {formatCurrency(portfolio.margin_used)} /{" "}
                        {formatCurrency(portfolio.equity)}
                      </p>
                    </>
                  )}
                </CardContent>
              </Card>
            </div>

            {/* Performance & Risk Metrics */}
            <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4">
              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    Margin sử dụng
                  </CardTitle>
                  <DollarSign className="h-4 w-4 text-muted-foreground" />
                </CardHeader>
                <CardContent>
                  <div className="text-lg font-bold text-warning">
                    {formatCurrency(portfolio.margin_used)}
                  </div>
                  <div className="text-xs text-muted-foreground space-y-1">
                    <div>Free: {formatCurrency(portfolio.free_margin)}</div>
                    <div>
                      Usage:{" "}
                      {(
                        (portfolio.margin_used / portfolio.equity) *
                        100
                      ).toFixed(1)}
                      %
                    </div>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    {closedTrades.length > 0 ? "Lợi nhuận TB" : "Avg Margin"}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {closedTrades.length > 0 ? (
                    <div className="text-lg font-bold text-profit">
                      {formatCurrency(portfolio.average_win)}
                    </div>
                  ) : (
                    <div className="text-lg font-bold text-primary">
                      {openTrades.length > 0
                        ? formatCurrency(
                            openTrades.reduce(
                              (total, trade) =>
                                total + calculateMarginRequired(trade),
                              0
                            ) / openTrades.length
                          )
                        : "$0.00"}
                    </div>
                  )}
                  <p className="text-xs text-muted-foreground">
                    {closedTrades.length > 0
                      ? "Trung bình thắng"
                      : "Margin trung bình"}
                  </p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    {closedTrades.length > 0 ? "Profit Factor" : "Daily P&L"}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {closedTrades.length > 0 ? (
                    <div className="text-lg font-bold">
                      {portfolio.profit_factor.toFixed(2)}
                    </div>
                  ) : (
                    <div
                      className={`text-lg font-bold ${
                        portfolio.total_pnl >= 0 ? "text-profit" : "text-loss"
                      }`}
                    >
                      {portfolio.total_pnl >= 0 ? "+" : ""}
                      {formatCurrency(portfolio.total_pnl)}
                    </div>
                  )}
                  <p className="text-xs text-muted-foreground">
                    {closedTrades.length > 0
                      ? "Tỷ lệ lời/lỗ"
                      : "Unrealized P&L"}
                  </p>
                </CardContent>
              </Card>

              <Card>
                <CardHeader className="flex flex-row items-center justify-between space-y-0 pb-2">
                  <CardTitle className="text-sm font-medium">
                    {closedTrades.length > 0
                      ? "Max Drawdown"
                      : "Trading Status"}
                  </CardTitle>
                </CardHeader>
                <CardContent>
                  {closedTrades.length > 0 ? (
                    <div className="text-lg font-bold text-loss">
                      {formatCurrency(portfolio.max_drawdown)}
                    </div>
                  ) : (
                    <div className="text-lg font-bold text-info">
                      {openTrades.length > 0
                        ? `${openTrades.length} Active`
                        : "No Trades"}
                    </div>
                  )}
                  <p className="text-xs text-muted-foreground">
                    {closedTrades.length > 0
                      ? formatPercentage(portfolio.max_drawdown_percentage)
                      : openTrades.length > 0
                      ? "Positions running"
                      : "Waiting for signals"}
                  </p>
                </CardContent>
              </Card>
            </div>

            {/* Performance Chart */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  Biểu đồ hiệu suất
                  <Badge
                    variant="outline"
                    className={`text-xs ${
                      wsConnected
                        ? "border-green-500 text-green-600"
                        : "border-gray-400"
                    }`}
                  >
                    {wsConnected && (
                      <div className="w-2 h-2 bg-green-500 rounded-full mr-1 animate-pulse"></div>
                    )}
                    {wsConnected ? "Live Data" : "Static Data"}
                  </Badge>
                  {wsConnected && (
                    <Badge
                      variant="secondary"
                      className="text-xs bg-blue-100 text-blue-700"
                    >
                      Updates #{lastUpdateCount}
                    </Badge>
                  )}
                </CardTitle>
                <div className="text-sm text-muted-foreground">
                  {openTrades.length > 0
                    ? `${
                        openTrades.length
                      } lệnh đang hoạt động • P&L: ${formatCurrency(
                        portfolio.total_pnl
                      )}`
                    : "Chưa có lệnh giao dịch nào"}
                </div>
              </CardHeader>
              <CardContent>
                {chartData.length > 1 ? (
                  <div className="h-96">
                    <ResponsiveContainer width="100%" height="100%">
                      <LineChart data={chartData}>
                        <CartesianGrid
                          strokeDasharray="3 3"
                          stroke="hsl(var(--border))"
                        />
                        <XAxis
                          dataKey="date"
                          stroke="hsl(var(--muted-foreground))"
                          fontSize={12}
                          angle={0}
                          textAnchor="middle"
                          height={50}
                          interval={0}
                        />
                        <YAxis
                          stroke="hsl(var(--muted-foreground))"
                          fontSize={11}
                          domain={[
                            (dataMin: number) => Math.max(dataMin * 0.998, 0),
                            (dataMax: number) => dataMax * 1.002,
                          ]}
                          tickFormatter={(value) => {
                            if (value >= 1000000) {
                              return `$${(value / 1000000).toFixed(1)}M`;
                            } else if (value >= 1000) {
                              return `$${(value / 1000).toFixed(1)}K`;
                            } else {
                              return `$${value.toFixed(0)}`;
                            }
                          }}
                        />
                        <Tooltip
                          content={({ active, payload, label }) => {
                            if (active && payload && payload.length) {
                              const data = payload[0]?.payload;
                              return (
                                <div className="bg-card border border-border rounded-lg p-4 shadow-lg min-w-[200px]">
                                  <p className="font-medium mb-2">{label}</p>
                                  <div className="space-y-1">
                                    <p className="text-sm flex justify-between">
                                      <span>Balance:</span>
                                      <span className="font-medium text-primary">
                                        {formatCurrency(data?.balance || 0)}
                                      </span>
                                    </p>
                                    <p className="text-sm flex justify-between">
                                      <span>P&L:</span>
                                      <span
                                        className={`font-medium ${
                                          (data?.pnl || 0) >= 0
                                            ? "text-profit"
                                            : "text-loss"
                                        }`}
                                      >
                                        {(data?.pnl || 0) >= 0 ? "+" : ""}
                                        {formatCurrency(data?.pnl || 0)}
                                      </span>
                                    </p>
                                    <p className="text-sm flex justify-between">
                                      <span>Margin Used:</span>
                                      <span className="font-medium text-warning">
                                        {formatCurrency(data?.marginUsed || 0)}
                                      </span>
                                    </p>
                                    <p className="text-sm flex justify-between">
                                      <span>Positions:</span>
                                      <span className="font-medium text-info">
                                        {data?.totalPositions || 0}
                                      </span>
                                    </p>
                                    {data?.tradeEvent && (
                                      <p className="text-xs text-muted-foreground mt-2">
                                        📊 Trade Event
                                      </p>
                                    )}
                                    {data?.isCurrent && (
                                      <p className="text-xs text-muted-foreground mt-2">
                                        🔴 Current Status
                                      </p>
                                    )}
                                  </div>
                                </div>
                              );
                            }
                            return null;
                          }}
                        />
                        <Line
                          type="monotone"
                          dataKey="balance"
                          stroke="hsl(var(--primary))"
                          strokeWidth={2}
                          dot={(props) => {
                            const { payload } = props;
                            if (payload?.tradeEvent) {
                              return (
                                <circle
                                  cx={props.cx}
                                  cy={props.cy}
                                  r={6}
                                  fill="hsl(var(--chart-1))"
                                  stroke="hsl(var(--primary))"
                                  strokeWidth={2}
                                />
                              );
                            }
                            if (payload?.isCurrent) {
                              return (
                                <circle
                                  cx={props.cx}
                                  cy={props.cy}
                                  r={8}
                                  fill="hsl(var(--chart-5))"
                                  stroke="hsl(var(--primary))"
                                  strokeWidth={3}
                                />
                              );
                            }
                            return null;
                          }}
                          activeDot={{
                            r: 6,
                            stroke: "hsl(var(--primary))",
                            strokeWidth: 2,
                          }}
                        />
                      </LineChart>
                    </ResponsiveContainer>
                  </div>
                ) : (
                  <div className="h-64 flex items-center justify-center text-muted-foreground">
                    <div className="text-center">
                      <Activity className="h-12 w-12 mx-auto mb-2 opacity-50" />
                      <p>Chưa có dữ liệu hiệu suất</p>
                      <p className="text-sm">
                        Khởi động bot để bắt đầu thu thập dữ liệu
                      </p>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          {/* AI Signals Tab */}
          <TabsContent value="signals" className="space-y-4 lg:space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  Tín hiệu AI gần đây
                  <Badge
                    variant="outline"
                    className="bg-info/10 text-info border-info/20"
                  >
                    <div className="w-2 h-2 bg-info rounded-full mr-2 animate-pulse"></div>
                    Live Analysis
                  </Badge>
                </CardTitle>
                <div className="text-sm text-muted-foreground">
                  GPT-4 Trading AI v2.0.0 • Model: gpt-3.5-turbo
                  <span className="ml-2">• WebSocket real-time signals</span>
                </div>
              </CardHeader>
              <CardContent>
                {isLoading && (
                  <div className="p-4 rounded-lg bg-muted/20 border border-muted/40 flex items-center gap-2 mb-4">
                    <RefreshCw className="h-4 w-4 animate-spin" />
                    <span className="text-sm text-muted-foreground">
                      Đang phân tích tín hiệu thị trường...
                    </span>
                  </div>
                )}

                {recentSignals && recentSignals.length > 0 ? (
                  <div className="space-y-4">
                    {recentSignals.map((signal, index) => {
                      const isActive =
                        Date.now() - new Date(signal.timestamp).getTime() <
                        30 * 60 * 1000; // Active if less than 30 minutes old
                      const isLong = signal.signal?.toLowerCase() === "long";
                      const isShort = signal.signal?.toLowerCase() === "short";

                      return (
                        <div
                          key={`${signal.symbol}-${signal.timestamp}-websocket`}
                          className={`p-4 rounded-lg border transition-all duration-200 ${
                            isActive
                              ? "bg-secondary/50 border-primary/20 shadow-primary/5"
                              : "bg-muted/20 border-muted/40"
                          }`}
                        >
                          <div className="flex justify-between items-start mb-3">
                            <div className="flex items-center gap-3">
                              <Badge
                                className={
                                  isLong
                                    ? "bg-profit text-profit-foreground"
                                    : isShort
                                    ? "bg-loss text-loss-foreground"
                                    : "bg-warning text-warning-foreground"
                                }
                              >
                                {signal.signal?.toUpperCase() || "NEUTRAL"}
                              </Badge>
                              <span className="font-semibold">
                                {signal.symbol?.replace("USDT", "/USDT") ||
                                  "N/A"}
                              </span>
                              {isActive && (
                                <Badge
                                  variant="outline"
                                  className="bg-profit/10 text-profit border-profit/20 text-xs"
                                >
                                  ACTIVE
                                </Badge>
                              )}
                              <Badge
                                variant="outline"
                                className="text-xs capitalize"
                              >
                                websocket
                              </Badge>
                            </div>
                            <div className="text-right">
                              <div
                                className={`font-bold text-lg ${
                                  signal.confidence >= 0.8
                                    ? "text-profit"
                                    : signal.confidence >= 0.6
                                    ? "text-warning"
                                    : "text-loss"
                                }`}
                              >
                                {((signal.confidence || 0) * 100).toFixed(0)}%
                              </div>
                              <div className="text-xs text-muted-foreground">
                                Confidence
                              </div>
                            </div>
                          </div>

                          <div className="space-y-2">
                            <p className="text-sm text-foreground">
                              {signal.reasoning || "WebSocket real-time signal"}
                            </p>
                            <div className="flex justify-between items-center text-xs text-muted-foreground">
                              <span>{formatDate(signal.timestamp)}</span>
                              <div className="flex items-center gap-1">
                                <div
                                  className={`w-2 h-2 rounded-full ${
                                    signal.confidence >= 0.8
                                      ? "bg-profit"
                                      : signal.confidence >= 0.6
                                      ? "bg-warning"
                                      : "bg-loss"
                                  }`}
                                ></div>
                                <span>AI Confidence</span>
                              </div>
                            </div>
                          </div>

                          {/* Confidence Bar */}
                          <div className="mt-3">
                            <div className="w-full bg-muted rounded-full h-1.5">
                              <div
                                className={`h-1.5 rounded-full transition-all duration-500 ${
                                  signal.confidence >= 0.8
                                    ? "bg-profit"
                                    : signal.confidence >= 0.6
                                    ? "bg-warning"
                                    : "bg-loss"
                                }`}
                                style={{
                                  width: `${(signal.confidence || 0) * 100}%`,
                                }}
                              ></div>
                            </div>
                          </div>
                        </div>
                      );
                    })}
                  </div>
                ) : (
                  <div className="flex items-center justify-center h-32 text-muted-foreground">
                    <div className="text-center">
                      <TrendingUp className="h-8 w-8 mx-auto mb-2 opacity-50" />
                      <p>Chưa có tín hiệu AI</p>
                      <p className="text-sm">
                        Tín hiệu sẽ xuất hiện tự động khi có phân tích mới
                      </p>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          {/* Trading History Tab */}
          <TabsContent value="trades" className="space-y-4 lg:space-y-6">
            {/* Open Trades */}
            {openTrades.length > 0 && (
              <Card>
                <CardHeader>
                  <div className="flex items-center justify-between">
                    <CardTitle>Lệnh đang mở ({openTrades.length})</CardTitle>
                    <div className="text-right space-y-1">
                      <div>
                        <div className="text-sm text-muted-foreground">
                          Tổng Position Size
                        </div>
                        <div className="font-bold text-primary">
                          {formatCurrency(
                            openTrades.reduce(
                              (total, trade) =>
                                total + calculatePositionSize(trade),
                              0
                            )
                          )}
                        </div>
                      </div>
                      <div>
                        <div className="text-sm text-muted-foreground">
                          Tổng Margin Required
                        </div>
                        <div className="font-bold text-warning">
                          {formatCurrency(
                            openTrades.reduce(
                              (total, trade) =>
                                total + calculateMarginRequired(trade),
                              0
                            )
                          )}
                        </div>
                      </div>
                    </div>
                  </div>
                </CardHeader>
                <CardContent>
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Symbol</TableHead>
                        <TableHead>Type</TableHead>
                        <TableHead>Entry Price</TableHead>
                        <TableHead>Quantity</TableHead>
                        <TableHead>Position Size</TableHead>
                        <TableHead>Margin Required</TableHead>
                        <TableHead>Leverage</TableHead>
                        <TableHead>Unrealized P&L</TableHead>
                        <TableHead>Stop Loss</TableHead>
                        <TableHead>Take Profit</TableHead>
                        <TableHead>Open Time</TableHead>
                        <TableHead>Action</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {openTrades.map((trade) => (
                        <TableRow
                          key={trade.id}
                          className="cursor-pointer hover:bg-muted/50 transition-colors"
                          onClick={() => openTradeDetails(trade)}
                        >
                          <TableCell className="font-medium">
                            <div className="flex items-center gap-2">
                              {trade.symbol}
                              <span className="text-xs text-muted-foreground">
                                ({trade.leverage}x)
                              </span>
                            </div>
                          </TableCell>
                          <TableCell>
                            <Badge
                              variant={
                                trade.trade_type === "Long"
                                  ? "default"
                                  : "destructive"
                              }
                              className={
                                trade.trade_type === "Long"
                                  ? "bg-profit text-profit-foreground"
                                  : "bg-loss text-loss-foreground"
                              }
                            >
                              {trade.trade_type}
                            </Badge>
                          </TableCell>
                          <TableCell>
                            {formatCurrency(trade.entry_price)}
                          </TableCell>
                          <TableCell>
                            <div className="text-right">
                              <div className="font-medium">
                                {trade.quantity.toFixed(6)}
                              </div>
                              <div className="text-xs text-muted-foreground">
                                {trade.symbol.replace("USDT", "")}
                              </div>
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="text-right">
                              <div className="font-medium text-primary">
                                {formatCurrency(calculatePositionSize(trade))}
                              </div>
                              <div className="text-xs text-muted-foreground">
                                Notional Value
                              </div>
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="text-right">
                              <div className="font-medium text-warning">
                                {formatCurrency(calculateMarginRequired(trade))}
                              </div>
                              <div className="text-xs text-muted-foreground">
                                với {trade.leverage}x leverage
                              </div>
                            </div>
                          </TableCell>
                          <TableCell>
                            <Badge variant="outline" className="font-mono">
                              {trade.leverage}x
                            </Badge>
                          </TableCell>
                          <TableCell>
                            <div className="text-right">
                              <div
                                className={`font-medium ${
                                  (trade.pnl || 0) >= 0
                                    ? "text-profit"
                                    : "text-loss"
                                }`}
                              >
                                {formatCurrency(trade.pnl || 0)}
                              </div>
                              <div
                                className={`text-xs ${
                                  trade.pnl_percentage >= 0
                                    ? "text-profit"
                                    : "text-loss"
                                }`}
                              >
                                ({trade.pnl_percentage >= 0 ? "+" : ""}
                                {trade.pnl_percentage.toFixed(2)}%)
                              </div>
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="text-center">
                              {trade.stop_loss ? (
                                <div className="text-loss font-medium">
                                  {formatCurrency(trade.stop_loss)}
                                </div>
                              ) : (
                                <Badge variant="secondary" className="text-xs">
                                  Chưa đặt
                                </Badge>
                              )}
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="text-center">
                              {trade.take_profit ? (
                                <div className="text-profit font-medium">
                                  {formatCurrency(trade.take_profit)}
                                </div>
                              ) : (
                                <Badge variant="secondary" className="text-xs">
                                  Chưa đặt
                                </Badge>
                              )}
                            </div>
                          </TableCell>
                          <TableCell>
                            <div className="text-sm">
                              {formatDate(new Date(trade.open_time))}
                            </div>
                          </TableCell>
                          <TableCell>
                            <Button
                              variant="outline"
                              size="sm"
                              onClick={(e) => {
                                e.stopPropagation();
                                closeTrade(trade.id);
                              }}
                              className="hover:bg-destructive hover:text-destructive-foreground"
                            >
                              Đóng
                            </Button>
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                </CardContent>
              </Card>
            )}

            {/* Closed Trades */}
            <Card>
              <CardHeader>
                <CardTitle>Lịch sử giao dịch ({closedTrades.length})</CardTitle>
              </CardHeader>
              <CardContent>
                {closedTrades.length > 0 ? (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead>Symbol</TableHead>
                        <TableHead>Type</TableHead>
                        <TableHead>Entry</TableHead>
                        <TableHead>Exit</TableHead>
                        <TableHead>Quantity</TableHead>
                        <TableHead>P&L</TableHead>
                        <TableHead>P&L %</TableHead>
                        <TableHead>Duration</TableHead>
                        <TableHead>Reason</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {closedTrades
                        .slice()
                        .reverse()
                        .map((trade) => (
                          <TableRow
                            key={trade.id}
                            className="cursor-pointer hover:bg-muted/50 transition-colors"
                            onClick={() => openTradeDetails(trade)}
                          >
                            <TableCell className="font-medium">
                              {trade.symbol}
                            </TableCell>
                            <TableCell>
                              <Badge
                                variant={
                                  trade.trade_type === "Long"
                                    ? "default"
                                    : "destructive"
                                }
                              >
                                {trade.trade_type}
                              </Badge>
                            </TableCell>
                            <TableCell>
                              {formatCurrency(trade.entry_price)}
                            </TableCell>
                            <TableCell>
                              {trade.exit_price
                                ? formatCurrency(trade.exit_price)
                                : "N/A"}
                            </TableCell>
                            <TableCell>{trade.quantity.toFixed(6)}</TableCell>
                            <TableCell
                              className={
                                trade.pnl && trade.pnl >= 0
                                  ? "text-profit"
                                  : "text-loss"
                              }
                            >
                              {trade.pnl ? formatCurrency(trade.pnl) : "N/A"}
                            </TableCell>
                            <TableCell
                              className={
                                trade.pnl_percentage &&
                                trade.pnl_percentage >= 0
                                  ? "text-profit"
                                  : "text-loss"
                              }
                            >
                              {trade.pnl_percentage
                                ? formatPercentage(trade.pnl_percentage)
                                : "N/A"}
                            </TableCell>
                            <TableCell>
                              {trade.close_time
                                ? Math.round(
                                    (new Date(trade.close_time).getTime() -
                                      new Date(trade.open_time).getTime()) /
                                      (1000 * 60)
                                  ) + "m"
                                : "N/A"}
                            </TableCell>
                            <TableCell>
                              <Badge variant="outline">{"Manual"}</Badge>
                            </TableCell>
                          </TableRow>
                        ))}
                    </TableBody>
                  </Table>
                ) : (
                  <div className="flex items-center justify-center h-32 text-muted-foreground">
                    <div className="text-center">
                      <History className="h-8 w-8 mx-auto mb-2 opacity-50" />
                      <p>Chưa có giao dịch nào</p>
                      <p className="text-sm">
                        Giao dịch sẽ hiển thị tại đây khi bot hoạt động
                      </p>
                    </div>
                  </div>
                )}
              </CardContent>
            </Card>
          </TabsContent>

          {/* Settings Tab */}
          <TabsContent value="settings" className="space-y-4 lg:space-y-6">
            {/* Paper Trading Basic Settings */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <DollarSign className="h-5 w-5" />
                  Cài đặt Paper Trading
                </CardTitle>
              </CardHeader>
              <CardContent className="space-y-4">
                <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                  <div className="space-y-2">
                    <Label htmlFor="initial-balance">Vốn ban đầu (USDT)</Label>
                    <Input
                      id="initial-balance"
                      type="number"
                      value={settingsForm.basic.initial_balance}
                      onChange={(e) =>
                        setSettingsForm((prev) => ({
                          ...prev,
                          basic: {
                            ...prev.basic,
                            initial_balance: parseFloat(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="max-leverage">Đòn bẩy tối đa</Label>
                    <Input
                      id="max-leverage"
                      type="number"
                      value={settingsForm.risk.max_leverage}
                      onChange={(e) =>
                        setSettingsForm((prev) => ({
                          ...prev,
                          risk: {
                            ...prev.risk,
                            max_leverage: parseFloat(e.target.value) || 1,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="position-size">Kích thước vị thế (%)</Label>
                    <Input
                      id="position-size"
                      type="number"
                      value={settingsForm.basic.default_position_size_pct}
                      onChange={(e) =>
                        setSettingsForm((prev) => ({
                          ...prev,
                          basic: {
                            ...prev.basic,
                            default_position_size_pct:
                              parseFloat(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="stop-loss">Stop Loss mặc định (%)</Label>
                    <Input
                      id="stop-loss"
                      type="number"
                      value={settingsForm.risk.default_stop_loss_pct}
                      onChange={(e) =>
                        setSettingsForm((prev) => ({
                          ...prev,
                          risk: {
                            ...prev.risk,
                            default_stop_loss_pct:
                              parseFloat(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="take-profit">
                      Take Profit mặc định (%)
                    </Label>
                    <Input
                      id="take-profit"
                      type="number"
                      value={settingsForm.risk.default_take_profit_pct}
                      onChange={(e) =>
                        setSettingsForm((prev) => ({
                          ...prev,
                          risk: {
                            ...prev.risk,
                            default_take_profit_pct:
                              parseFloat(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                  <div className="space-y-2">
                    <Label htmlFor="trading-fee">Phí giao dịch (%)</Label>
                    <Input
                      id="trading-fee"
                      type="number"
                      value={settingsForm.basic.trading_fee_rate}
                      onChange={(e) =>
                        setSettingsForm((prev) => ({
                          ...prev,
                          basic: {
                            ...prev.basic,
                            trading_fee_rate: parseFloat(e.target.value) || 0,
                          },
                        }))
                      }
                    />
                  </div>
                </div>
                <div className="flex gap-4 pt-4">
                  <Button
                    onClick={handleSettingsSubmit}
                    className="flex-1"
                    disabled={isLoading}
                  >
                    {isLoading ? "Đang lưu..." : "Lưu cài đặt"}
                  </Button>
                  <Button
                    variant="outline"
                    onClick={() => setShowReset(true)}
                    className="flex-1"
                  >
                    <RotateCcw className="h-4 w-4 mr-2" />
                    Reset dữ liệu
                  </Button>
                </div>
                {showReset && (
                  <Alert>
                    <AlertCircle className="h-4 w-4" />
                    <AlertDescription>
                      <div className="flex items-center justify-between">
                        <span>
                          Xác nhận reset toàn bộ dữ liệu paper trading?
                        </span>
                        <div className="flex gap-2">
                          <Button
                            variant="destructive"
                            size="sm"
                            onClick={handleReset}
                          >
                            Xác nhận
                          </Button>
                          <Button
                            variant="outline"
                            size="sm"
                            onClick={() => setShowReset(false)}
                          >
                            Hủy
                          </Button>
                        </div>
                      </div>
                    </AlertDescription>
                  </Alert>
                )}
              </CardContent>
            </Card>

            {/* Separator */}
            <div className="flex items-center gap-4">
              <Separator className="flex-1" />
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <Settings className="h-4 w-4" />
                Cài đặt nâng cao
              </div>
              <Separator className="flex-1" />
            </div>

            {/* Symbol Configuration Settings - Simple Button */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <Target className="h-5 w-5" />
                  Cài đặt Symbols
                  <Badge variant="secondary" className="text-xs">
                    {Object.keys(symbolSettings).length} symbols
                  </Badge>
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  Cấu hình riêng cho từng symbol: leverage, kích thước vị thế,
                  stop loss/take profit.
                </p>
              </CardHeader>
              <CardContent>
                <Button
                  onClick={() => setShowSymbolDialog(true)}
                  variant="outline"
                  className="w-full"
                >
                  <Settings className="h-4 w-4 mr-2" />
                  Mở cài đặt Symbols
                </Button>
              </CardContent>
            </Card>

            {/* Advanced Trading Strategy Settings */}
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center gap-2">
                  <TrendingUp className="h-5 w-5" />
                  Cài đặt Chiến lược Trading
                  <Badge variant="secondary" className="text-xs">
                    Thích hợp cho thị trường ít biến động
                  </Badge>
                </CardTitle>
                <p className="text-sm text-muted-foreground">
                  Điều chỉnh các tham số chiến lược để tối ưu cho điều kiện thị
                  trường hiện tại. Sử dụng preset "Low Volatility" cho thị
                  trường ít biến động.
                </p>
              </CardHeader>
              <CardContent>
                <TradingSettings />
              </CardContent>
            </Card>
          </TabsContent>
        </Tabs>

        {/* Real-time Footer Status */}
        {wsConnected && (
          <div className="mt-6 p-3 bg-green-50 border border-green-200 rounded-lg">
            <div className="flex items-center justify-between text-sm">
              <div className="flex items-center gap-4">
                <div className="flex items-center gap-2">
                  <div className="w-2 h-2 bg-green-500 rounded-full animate-pulse"></div>
                  <span className="text-green-700 font-medium">
                    WebSocket Active
                  </span>
                </div>
                <div className="text-green-600">
                  Real-time updates: {lastUpdateCount}
                </div>
                <div className="text-green-600">
                  Last sync:{" "}
                  {lastUpdated?.toLocaleTimeString("vi-VN") || "Never"}
                </div>
              </div>
              <div className="text-green-600">
                Data refreshes automatically every second 🚀
              </div>
            </div>
          </div>
        )}
      </div>

      {/* Trade Details Popup */}
      <Dialog open={isTradeDetailOpen} onOpenChange={setIsTradeDetailOpen}>
        <DialogContent className="max-w-2xl max-h-[90vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2 text-xl">
              <Badge
                variant={
                  selectedTrade?.trade_type === "Long"
                    ? "default"
                    : "destructive"
                }
                className={
                  selectedTrade?.trade_type === "Long"
                    ? "bg-profit text-profit-foreground"
                    : "bg-loss text-loss-foreground"
                }
              >
                {selectedTrade?.trade_type}
              </Badge>
              {selectedTrade?.symbol}
              <span className="text-sm font-normal text-muted-foreground">
                Chi tiết giao dịch
              </span>
            </DialogTitle>
            <DialogDescription>
              Thông tin chi tiết về vị thế đang mở
            </DialogDescription>
          </DialogHeader>

          {selectedTrade && (
            <div className="space-y-6">
              {/* Key Metrics */}
              <div className="grid grid-cols-2 gap-4 p-4 bg-muted/30 rounded-lg">
                <div className="text-center">
                  <div className="text-sm text-muted-foreground">
                    Unrealized P&L
                  </div>
                  <div
                    className={`text-2xl font-bold ${
                      (selectedTrade.pnl || 0) >= 0
                        ? "text-profit"
                        : "text-loss"
                    }`}
                  >
                    {formatCurrency(selectedTrade.pnl || 0)}
                  </div>
                  <div
                    className={`text-sm ${
                      selectedTrade.pnl_percentage >= 0
                        ? "text-profit"
                        : "text-loss"
                    }`}
                  >
                    ({selectedTrade.pnl_percentage >= 0 ? "+" : ""}
                    {selectedTrade.pnl_percentage.toFixed(2)}%)
                  </div>
                </div>
                <div className="text-center">
                  <div className="text-sm text-muted-foreground">
                    Position Size
                  </div>
                  <div className="text-2xl font-bold text-primary">
                    {formatCurrency(calculatePositionSize(selectedTrade))}
                  </div>
                  <div className="text-sm text-muted-foreground">
                    với {selectedTrade.leverage}x leverage
                  </div>
                </div>
              </div>

              {/* Trade Details */}
              <div className="grid grid-cols-2 gap-6">
                <div className="space-y-4">
                  <h3 className="font-semibold text-lg">Thông tin giao dịch</h3>

                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Symbol:</span>
                      <span className="font-medium">
                        {selectedTrade.symbol}
                      </span>
                    </div>

                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Type:</span>
                      <Badge
                        variant={
                          selectedTrade.trade_type === "Long"
                            ? "default"
                            : "destructive"
                        }
                        className={
                          selectedTrade.trade_type === "Long"
                            ? "bg-profit text-profit-foreground"
                            : "bg-loss text-loss-foreground"
                        }
                      >
                        {selectedTrade.trade_type}
                      </Badge>
                    </div>

                    <div className="flex justify-between">
                      <span className="text-muted-foreground">
                        Entry Price:
                      </span>
                      <span className="font-medium">
                        {formatCurrency(selectedTrade.entry_price)}
                      </span>
                    </div>

                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Quantity:</span>
                      <span className="font-medium">
                        {selectedTrade.quantity.toFixed(6)}{" "}
                        {selectedTrade.symbol.replace("USDT", "")}
                      </span>
                    </div>

                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Leverage:</span>
                      <Badge variant="outline" className="font-mono">
                        {selectedTrade.leverage}x
                      </Badge>
                    </div>

                    <div className="flex justify-between">
                      <span className="text-muted-foreground">
                        Position Value:
                      </span>
                      <span className="font-medium">
                        {formatCurrency(calculatePositionValue(selectedTrade))}
                      </span>
                    </div>
                  </div>
                </div>

                <div className="space-y-4">
                  <h3 className="font-semibold text-lg">Risk Management</h3>

                  <div className="space-y-3">
                    <div className="flex justify-between items-center">
                      <span className="text-muted-foreground">Stop Loss:</span>
                      {selectedTrade.stop_loss ? (
                        <div className="text-right">
                          <div className="text-loss font-medium">
                            {formatCurrency(selectedTrade.stop_loss)}
                          </div>
                          <div className="text-xs text-muted-foreground">
                            {(
                              ((selectedTrade.stop_loss -
                                selectedTrade.entry_price) /
                                selectedTrade.entry_price) *
                              100
                            ).toFixed(2)}
                            %
                          </div>
                        </div>
                      ) : (
                        <Badge variant="secondary" className="text-xs">
                          Chưa đặt
                        </Badge>
                      )}
                    </div>

                    <div className="flex justify-between items-center">
                      <span className="text-muted-foreground">
                        Take Profit:
                      </span>
                      {selectedTrade.take_profit ? (
                        <div className="text-right">
                          <div className="text-profit font-medium">
                            {formatCurrency(selectedTrade.take_profit)}
                          </div>
                          <div className="text-xs text-muted-foreground">
                            +
                            {(
                              ((selectedTrade.take_profit -
                                selectedTrade.entry_price) /
                                selectedTrade.entry_price) *
                              100
                            ).toFixed(2)}
                            %
                          </div>
                        </div>
                      ) : (
                        <Badge variant="secondary" className="text-xs">
                          Chưa đặt
                        </Badge>
                      )}
                    </div>

                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Open Time:</span>
                      <div className="text-right">
                        <div className="font-medium">
                          {formatDate(new Date(selectedTrade.open_time))}
                        </div>
                        <div className="text-xs text-muted-foreground">
                          {new Date(selectedTrade.open_time).toLocaleTimeString(
                            "vi-VN"
                          )}
                        </div>
                      </div>
                    </div>

                    <div className="flex justify-between">
                      <span className="text-muted-foreground">Duration:</span>
                      <span className="font-medium">
                        {Math.floor(
                          (Date.now() -
                            new Date(selectedTrade.open_time).getTime()) /
                            (1000 * 60)
                        )}{" "}
                        phút
                      </span>
                    </div>
                  </div>
                </div>
              </div>

              {/* Action Buttons */}
              <div className="flex gap-3 pt-4 border-t">
                <Button
                  variant="destructive"
                  className="flex-1"
                  onClick={() => {
                    closeTrade(selectedTrade.id);
                    setIsTradeDetailOpen(false);
                    toast.success(`Đã đóng vị thế ${selectedTrade.symbol}`);
                  }}
                >
                  <X className="w-4 h-4 mr-2" />
                  Đóng vị thế
                </Button>
                <Button
                  variant="outline"
                  className="flex-1"
                  onClick={() => setIsTradeDetailOpen(false)}
                >
                  Đóng popup
                </Button>
              </div>
            </div>
          )}
        </DialogContent>
      </Dialog>

      {/* Symbol Configuration Dialog */}
      <Dialog open={showSymbolDialog} onOpenChange={setShowSymbolDialog}>
        <DialogContent className="max-w-4xl max-h-[80vh] overflow-y-auto">
          <DialogHeader>
            <DialogTitle className="flex items-center gap-2">
              <Target className="h-5 w-5" />
              Cài đặt Symbols
              <Badge variant="secondary" className="text-xs">
                {Object.keys(symbolSettings).length} symbols
              </Badge>
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
                        <CardTitle className="text-lg font-semibold">
                          {symbol.replace("USDT", "/USDT")}
                        </CardTitle>
                        <div className="flex items-center gap-2">
                          <Badge
                            variant={config.enabled ? "default" : "secondary"}
                          >
                            {config.enabled ? "Bật" : "Tắt"}
                          </Badge>
                          <Label
                            htmlFor={`enabled-${symbol}`}
                            className="text-sm"
                          >
                            Kích hoạt
                          </Label>
                          <input
                            type="checkbox"
                            id={`enabled-${symbol}`}
                            checked={config.enabled}
                            onChange={(e) =>
                              setSymbolSettings((prev) => ({
                                ...prev,
                                [symbol]: {
                                  ...prev[symbol],
                                  enabled: e.target.checked,
                                },
                              }))
                            }
                            className="h-4 w-4"
                          />
                        </div>
                      </div>
                    </CardHeader>
                    <CardContent className="space-y-4">
                      <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                        <div className="space-y-2">
                          <Label htmlFor={`leverage-${symbol}`}>Đòn bẩy</Label>
                          <Input
                            id={`leverage-${symbol}`}
                            type="number"
                            min="1"
                            max="50"
                            value={config.leverage}
                            onChange={(e) =>
                              setSymbolSettings((prev) => ({
                                ...prev,
                                [symbol]: {
                                  ...prev[symbol],
                                  leverage: parseInt(e.target.value) || 1,
                                },
                              }))
                            }
                          />
                        </div>
                        <div className="space-y-2">
                          <Label htmlFor={`position-size-${symbol}`}>
                            Kích thước vị thế (%)
                          </Label>
                          <Input
                            id={`position-size-${symbol}`}
                            type="number"
                            min="0.1"
                            max="100"
                            step="0.1"
                            value={config.position_size_pct}
                            onChange={(e) =>
                              setSymbolSettings((prev) => ({
                                ...prev,
                                [symbol]: {
                                  ...prev[symbol],
                                  position_size_pct:
                                    parseFloat(e.target.value) || 0,
                                },
                              }))
                            }
                          />
                        </div>
                        <div className="space-y-2">
                          <Label htmlFor={`max-positions-${symbol}`}>
                            Số vị thế tối đa
                          </Label>
                          <Input
                            id={`max-positions-${symbol}`}
                            type="number"
                            min="1"
                            max="10"
                            value={config.max_positions}
                            onChange={(e) =>
                              setSymbolSettings((prev) => ({
                                ...prev,
                                [symbol]: {
                                  ...prev[symbol],
                                  max_positions: parseInt(e.target.value) || 1,
                                },
                              }))
                            }
                          />
                        </div>
                      </div>
                      <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                        <div className="space-y-2">
                          <Label htmlFor={`stop-loss-${symbol}`}>
                            Stop Loss (%)
                          </Label>
                          <Input
                            id={`stop-loss-${symbol}`}
                            type="number"
                            min="0.1"
                            max="50"
                            step="0.1"
                            value={config.stop_loss_pct}
                            onChange={(e) =>
                              setSymbolSettings((prev) => ({
                                ...prev,
                                [symbol]: {
                                  ...prev[symbol],
                                  stop_loss_pct:
                                    parseFloat(e.target.value) || 0,
                                },
                              }))
                            }
                          />
                        </div>
                        <div className="space-y-2">
                          <Label htmlFor={`take-profit-${symbol}`}>
                            Take Profit (%)
                          </Label>
                          <Input
                            id={`take-profit-${symbol}`}
                            type="number"
                            min="0.1"
                            max="100"
                            step="0.1"
                            value={config.take_profit_pct}
                            onChange={(e) =>
                              setSymbolSettings((prev) => ({
                                ...prev,
                                [symbol]: {
                                  ...prev[symbol],
                                  take_profit_pct:
                                    parseFloat(e.target.value) || 0,
                                },
                              }))
                            }
                          />
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
                <div className="flex gap-4 pt-4">
                  <Button
                    onClick={updateSymbolSettings}
                    className="flex-1"
                    disabled={isLoadingSymbols}
                  >
                    {isLoadingSymbols ? (
                      <>
                        <RefreshCw className="h-4 w-4 mr-2 animate-spin" />
                        Đang lưu...
                      </>
                    ) : (
                      "Lưu cài đặt Symbols"
                    )}
                  </Button>
                  <Button
                    variant="outline"
                    onClick={loadSymbolSettings}
                    disabled={isLoadingSymbols}
                  >
                    <RefreshCw className="h-4 w-4 mr-2" />
                    Tải lại
                  </Button>
                </div>
              </>
            ) : (
              <div className="flex items-center justify-center p-8">
                <div className="text-center">
                  <Target className="h-8 w-8 mx-auto mb-2 opacity-50" />
                  <p className="text-muted-foreground">
                    Chưa có cài đặt symbols
                  </p>
                  <Button
                    variant="outline"
                    onClick={loadSymbolSettings}
                    className="mt-2"
                  >
                    Tải cài đặt
                  </Button>
                </div>
              </div>
            )}
          </div>
        </DialogContent>
      </Dialog>
    </div>
  );
};

export default TradingPaper;
