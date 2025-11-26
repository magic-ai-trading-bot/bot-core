import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Badge } from "@/components/ui/badge";
import { Skeleton } from "@/components/ui/skeleton";
import { TrendingUp, TrendingDown } from "lucide-react";
import { usePaperTradingContext } from "@/contexts/PaperTradingContext";
import { useMarketData } from "@/hooks/useMarketData";
import { useState, useEffect, useCallback } from "react";
import logger from "@/utils/logger";

// API Base URL - using environment variable with fallback
const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

// Default symbol used only when API fails
const DEFAULT_SYMBOL = "BTCUSDT";

export function BotStatus() {
  const { portfolio, openTrades: positions, isLoading } = usePaperTradingContext();
  const [primarySymbol, setPrimarySymbol] = useState(DEFAULT_SYMBOL);

  // Fetch primary symbol dynamically from API
  const fetchPrimarySymbol = useCallback(async () => {
    try {
      const response = await fetch(`${API_BASE}/api/market/symbols`);
      const data = await response.json();
      // FIX: API returns {success: true, data: {symbols: [...]}} - access data.data.symbols
      if (data.success && data.data && data.data.symbols && data.data.symbols.length > 0) {
        setPrimarySymbol(data.data.symbols[0]); // Use first symbol as primary
        logger.info(`Primary symbol set to: ${data.data.symbols[0]}`);
      }
    } catch (error) {
      logger.error("Failed to fetch primary symbol:", error);
      // Keep default symbol on error
    }
  }, []);

  // Load primary symbol on mount
  useEffect(() => {
    fetchPrimarySymbol();
  }, [fetchPrimarySymbol]);

  const { data: marketData } = useMarketData(primarySymbol, "1h", 5000);

  if (isLoading) {
    return (
      <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6">
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent className="space-y-4">
            <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
              <div className="space-y-2">
                <Skeleton className="h-4 w-24" />
                <Skeleton className="h-8 w-32" />
              </div>
              <div className="space-y-2">
                <Skeleton className="h-4 w-24" />
                <Skeleton className="h-8 w-32" />
              </div>
            </div>
            <div className="pt-2 border-t">
              <Skeleton className="h-6 w-full" />
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <Skeleton className="h-6 w-32" />
          </CardHeader>
          <CardContent className="space-y-3">
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-20 w-full" />
            <Skeleton className="h-20 w-full" />
          </CardContent>
        </Card>
      </div>
    );
  }

  return (
    <div className="grid grid-cols-1 lg:grid-cols-2 gap-4 lg:gap-6">
      {/* Balance Overview */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base lg:text-lg">Account Balance</CardTitle>
        </CardHeader>
        <CardContent className="space-y-4">
          <div className="grid grid-cols-1 sm:grid-cols-2 gap-4">
            <div>
              <p className="text-xs lg:text-sm text-muted-foreground">Total Balance</p>
              <p className="text-xl lg:text-2xl font-bold">${(portfolio?.current_balance || 0).toLocaleString()}</p>
            </div>
            <div>
              <p className="text-xs lg:text-sm text-muted-foreground">Available Funds</p>
              <p className="text-xl lg:text-2xl font-bold text-profit">${(portfolio?.free_margin ?? portfolio?.available_balance ?? 0).toLocaleString()}</p>
            </div>
          </div>
          <div className="pt-2 border-t">
            <div className="flex justify-between items-center">
              <span className="text-xs lg:text-sm text-muted-foreground">{primarySymbol.replace('USDT', '/USDT')}</span>
              <span className="font-mono text-base lg:text-lg">${(marketData?.price || 0).toLocaleString()}</span>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Open Positions */}
      <Card>
        <CardHeader>
          <CardTitle className="text-base lg:text-lg">Open Positions</CardTitle>
        </CardHeader>
        <CardContent className="space-y-3 lg:space-y-4">
          {!positions || positions.length === 0 ? (
            <div className="flex flex-col items-center justify-center py-8 text-center">
              <div className="rounded-full bg-muted p-3 mb-3">
                <TrendingUp className="h-6 w-6 text-muted-foreground" />
              </div>
              <p className="text-muted-foreground text-sm">
                Không có vị thế đang mở
              </p>
              <p className="text-xs text-muted-foreground mt-1">
                Bot sẽ tự động mở vị thế khi phát hiện tín hiệu
              </p>
            </div>
          ) : (
            positions.map((position) => (
              <div key={position.id} className="p-3 rounded-lg bg-secondary/50 border">
                <div className="flex flex-col sm:flex-row sm:justify-between sm:items-start gap-2 mb-2">
                  <div className="flex items-center gap-2 flex-wrap">
                    <Badge
                      variant={(position.trade_type === "Long" || position.side === "BUY") ? "default" : "secondary"}
                      className={(position.trade_type === "Long" || position.side === "BUY") ? "bg-profit text-profit-foreground" : "bg-loss text-loss-foreground"}
                    >
                      {(position.trade_type === "Long" || position.side === "BUY") ? "LONG" : "SHORT"}
                    </Badge>
                    <span className="font-semibold text-sm lg:text-base">{position.symbol}</span>
                    <span className="text-xs lg:text-sm text-muted-foreground">{position.leverage}x</span>
                  </div>
                  <div className="text-left sm:text-right">
                    <div className={`font-semibold text-base lg:text-lg flex items-center gap-1 ${(position.pnl ?? position.unrealized_pnl ?? 0) >= 0 ? 'text-profit' : 'text-loss'}`}>
                      {(position.pnl ?? position.unrealized_pnl ?? 0) >= 0 ? (
                        <TrendingUp className="h-4 w-4" aria-hidden="true" />
                      ) : (
                        <TrendingDown className="h-4 w-4" aria-hidden="true" />
                      )}
                      <span>{(position.pnl ?? position.unrealized_pnl ?? 0) >= 0 ? '+' : ''}${(position.pnl ?? position.unrealized_pnl ?? 0).toFixed(2)}</span>
                      <span className="sr-only">{(position.pnl ?? position.unrealized_pnl ?? 0) >= 0 ? 'Profit' : 'Loss'}</span>
                    </div>
                    <div className={`text-xs lg:text-sm flex items-center gap-1 ${(position.pnl_percentage ?? position.unrealized_pnl_percent ?? 0) >= 0 ? 'text-profit' : 'text-loss'}`}>
                      <span>{(position.pnl_percentage ?? position.unrealized_pnl_percent ?? 0) >= 0 ? '+' : ''}{(position.pnl_percentage ?? position.unrealized_pnl_percent ?? 0).toFixed(2)}%</span>
                    </div>
                  </div>
                </div>
                <div className="flex flex-col sm:flex-row sm:justify-between gap-1 text-xs lg:text-sm text-muted-foreground">
                  <span>Entry: ${(position?.entry_price || 0).toLocaleString()}</span>
                  <span>Size: {(position?.quantity || 0).toFixed(6)}</span>
                </div>
              </div>
            ))
          )}
        </CardContent>
      </Card>
    </div>
  );
}