import { useState, useEffect } from "react";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { Badge } from "@/components/ui/badge";
import {
  Accordion,
  AccordionContent,
  AccordionItem,
  AccordionTrigger,
} from "@/components/ui/accordion";
import { useToast } from "@/hooks/use-toast";
import {
  TrendingUp,
  AlertTriangle,
  Shield,
  RotateCcw,
  Save,
  Loader2,
} from "lucide-react";
import { cn } from "@/lib/utils";

// @spec:FR-PAPER-002 - Per-Symbol Configuration
// @ref:specs/02-design/2.5-components/COMP-RUST-TRADING.md#symbol-settings
// @test:TC-PAPER-005, TC-PAPER-006

/**
 * Symbol configuration interface
 */
export interface SymbolConfig {
  symbol: string;
  enabled: boolean;
  leverage: number; // 1x - 20x
  position_size_pct: number; // 1% - 10% of portfolio
  stop_loss_pct: number; // 0.5% - 5%
  take_profit_pct: number; // 1% - 10%
  max_positions: number; // 1-5 concurrent positions for this symbol
}

/**
 * Symbol preset configurations
 */
interface SymbolPreset {
  name: string;
  config: Omit<SymbolConfig, "symbol">;
}

/**
 * Risk level calculation based on leverage and position size
 */
type RiskLevel = "low" | "moderate" | "high";

const PRESETS: Record<string, SymbolPreset> = {
  BTCUSDT: {
    name: "Conservative (BTC)",
    config: {
      enabled: true,
      leverage: 10,
      position_size_pct: 5,
      stop_loss_pct: 2,
      take_profit_pct: 4,
      max_positions: 2,
    },
  },
  ETHUSDT: {
    name: "Moderate (ETH)",
    config: {
      enabled: true,
      leverage: 7,
      position_size_pct: 4,
      stop_loss_pct: 2.5,
      take_profit_pct: 5,
      max_positions: 2,
    },
  },
  SOLUSDT: {
    name: "Aggressive (SOL)",
    config: {
      enabled: true,
      leverage: 5,
      position_size_pct: 3,
      stop_loss_pct: 3,
      take_profit_pct: 6,
      max_positions: 1,
    },
  },
  BNBUSDT: {
    name: "Moderate (BNB)",
    config: {
      enabled: true,
      leverage: 7,
      position_size_pct: 4,
      stop_loss_pct: 2.5,
      take_profit_pct: 5,
      max_positions: 2,
    },
  },
};

const DEFAULT_SYMBOLS = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];

interface PerSymbolSettingsProps {
  currentBalance?: number;
  onSettingsUpdate?: (configs: SymbolConfig[]) => void;
}

export function PerSymbolSettings({
  currentBalance = 10000,
  onSettingsUpdate,
}: PerSymbolSettingsProps) {
  const { toast } = useToast();
  const [configs, setConfigs] = useState<SymbolConfig[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

  /**
   * Load symbol configs from backend
   */
  useEffect(() => {
    const loadConfigs = async () => {
      setIsLoading(true);
      try {
        const response = await fetch(
          `${API_BASE}/api/paper-trading/symbol-settings`
        );

        if (response.ok) {
          const data = await response.json();
          if (data.success && data.data) {
            setConfigs(data.data);
          } else {
            // Initialize with presets if no data
            initializePresets();
          }
        } else {
          // Initialize with presets on error
          initializePresets();
        }
      } catch (error) {
        // Initialize with presets on error
        initializePresets();
      } finally {
        setIsLoading(false);
      }
    };

    loadConfigs();
  }, [API_BASE]);

  /**
   * Initialize with preset configurations
   */
  const initializePresets = () => {
    const initialConfigs = DEFAULT_SYMBOLS.map((symbol) => ({
      symbol,
      ...(PRESETS[symbol]?.config || {
        enabled: false,
        leverage: 5,
        position_size_pct: 3,
        stop_loss_pct: 2,
        take_profit_pct: 4,
        max_positions: 1,
      }),
    }));
    setConfigs(initialConfigs);
  };

  /**
   * Calculate risk level based on leverage and position size
   */
  const calculateRiskLevel = (
    leverage: number,
    positionSize: number
  ): RiskLevel => {
    const riskScore = leverage * positionSize;

    if (riskScore > 50) return "high";
    if (riskScore > 25) return "moderate";
    return "low";
  };

  /**
   * Get risk color classes
   */
  const getRiskColor = (level: RiskLevel) => {
    switch (level) {
      case "low":
        return "text-green-600 dark:text-green-400";
      case "moderate":
        return "text-yellow-600 dark:text-yellow-400";
      case "high":
        return "text-red-600 dark:text-red-400";
    }
  };

  /**
   * Get risk icon
   */
  const getRiskIcon = (level: RiskLevel) => {
    switch (level) {
      case "low":
        return <Shield className="h-4 w-4" />;
      case "moderate":
        return <AlertTriangle className="h-4 w-4" />;
      case "high":
        return <TrendingUp className="h-4 w-4" />;
    }
  };

  /**
   * Update a specific symbol config
   */
  const updateSymbolConfig = (
    symbol: string,
    updates: Partial<SymbolConfig>
  ) => {
    setConfigs((prev) =>
      prev.map((config) =>
        config.symbol === symbol ? { ...config, ...updates } : config
      )
    );
  };

  /**
   * Reset all configs to presets
   */
  const resetToDefaults = () => {
    initializePresets();
    toast({
      title: "Settings Reset",
      description: "All symbol settings have been reset to defaults.",
    });
  };

  /**
   * Save all configs to backend
   */
  const saveAllConfigs = async () => {
    setIsSaving(true);
    try {
      const response = await fetch(
        `${API_BASE}/api/paper-trading/symbol-settings`,
        {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({ symbols: configs }),
        }
      );

      const data = await response.json();

      if (data.success) {
        toast({
          title: "Settings Saved",
          description: "All symbol settings have been saved successfully.",
        });
        onSettingsUpdate?.(configs);
      } else {
        throw new Error(data.error || "Failed to save settings");
      }
    } catch (error) {
      toast({
        title: "Save Failed",
        description:
          error instanceof Error ? error.message : "Failed to save settings",
        variant: "destructive",
      });
    } finally {
      setIsSaving(false);
    }
  };

  /**
   * Save individual symbol config
   */
  const saveSymbolConfig = async (symbol: string) => {
    const config = configs.find((c) => c.symbol === symbol);
    if (!config) return;

    try {
      const response = await fetch(
        `${API_BASE}/api/paper-trading/symbol-settings/${symbol}`,
        {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify(config),
        }
      );

      const data = await response.json();

      if (data.success) {
        toast({
          title: "Settings Saved",
          description: `${symbol} settings have been saved.`,
        });
      } else {
        throw new Error(data.error || "Failed to save settings");
      }
    } catch (error) {
      toast({
        title: "Save Failed",
        description:
          error instanceof Error ? error.message : "Failed to save settings",
        variant: "destructive",
      });
    }
  };

  /**
   * Calculate position size in dollars
   */
  const calculatePositionSize = (positionSizePct: number, leverage: number) => {
    const baseSize = (currentBalance * positionSizePct) / 100;
    return baseSize * leverage;
  };

  if (isLoading) {
    return (
      <Card>
        <CardHeader>
          <CardTitle>Per-Symbol Settings</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="flex items-center justify-center py-8">
            <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
          </div>
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle>Per-Symbol Settings</CardTitle>
            <p className="text-sm text-muted-foreground mt-1">
              Configure trading parameters for each symbol individually
            </p>
          </div>
          <div className="flex gap-2">
            <Button
              variant="outline"
              size="sm"
              onClick={resetToDefaults}
              disabled={isSaving}
            >
              <RotateCcw className="h-4 w-4 mr-2" />
              Reset
            </Button>
            <Button
              size="sm"
              onClick={saveAllConfigs}
              disabled={isSaving}
            >
              {isSaving ? (
                <>
                  <Loader2 className="h-4 w-4 mr-2 animate-spin" />
                  Saving...
                </>
              ) : (
                <>
                  <Save className="h-4 w-4 mr-2" />
                  Save All
                </>
              )}
            </Button>
          </div>
        </div>
      </CardHeader>
      <CardContent>
        <Accordion type="single" collapsible className="w-full">
          {configs.map((config) => {
            const riskLevel = calculateRiskLevel(
              config.leverage,
              config.position_size_pct
            );
            const positionSize = calculatePositionSize(
              config.position_size_pct,
              config.leverage
            );

            return (
              <AccordionItem key={config.symbol} value={config.symbol}>
                <AccordionTrigger className="hover:no-underline">
                  <div className="flex items-center gap-4 w-full pr-4">
                    <Switch
                      checked={config.enabled}
                      onCheckedChange={(checked) =>
                        updateSymbolConfig(config.symbol, { enabled: checked })
                      }
                      onClick={(e) => e.stopPropagation()}
                    />
                    <div className="flex-1 text-left">
                      <div className="flex items-center gap-2">
                        <span className="font-semibold">{config.symbol}</span>
                        <Badge
                          variant={config.enabled ? "default" : "secondary"}
                          className="text-xs"
                        >
                          {config.enabled ? "Active" : "Disabled"}
                        </Badge>
                      </div>
                      <div className="flex items-center gap-3 mt-1 text-xs text-muted-foreground">
                        <span>{config.leverage}x Leverage</span>
                        <span>•</span>
                        <span>{config.position_size_pct}% Position</span>
                        <span>•</span>
                        <div
                          className={cn(
                            "flex items-center gap-1",
                            getRiskColor(riskLevel)
                          )}
                        >
                          {getRiskIcon(riskLevel)}
                          <span className="capitalize">{riskLevel} Risk</span>
                        </div>
                      </div>
                    </div>
                  </div>
                </AccordionTrigger>
                <AccordionContent>
                  <div className="space-y-6 pt-4">
                    {/* Leverage */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="text-sm font-medium">Leverage</label>
                        <span className="text-sm text-muted-foreground">
                          {config.leverage}x
                        </span>
                      </div>
                      <Slider
                        value={[config.leverage]}
                        onValueChange={([value]) =>
                          updateSymbolConfig(config.symbol, { leverage: value })
                        }
                        min={1}
                        max={20}
                        step={1}
                        className="w-full"
                      />
                      <p className="text-xs text-muted-foreground">
                        Higher leverage increases both potential profit and risk
                      </p>
                    </div>

                    {/* Position Size */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="text-sm font-medium">
                          Position Size
                        </label>
                        <div className="text-sm text-muted-foreground">
                          {config.position_size_pct}% ($
                          {positionSize.toFixed(2)})
                        </div>
                      </div>
                      <Slider
                        value={[config.position_size_pct]}
                        onValueChange={([value]) =>
                          updateSymbolConfig(config.symbol, {
                            position_size_pct: value,
                          })
                        }
                        min={1}
                        max={10}
                        step={0.5}
                        className="w-full"
                      />
                      <p className="text-xs text-muted-foreground">
                        Percentage of portfolio to allocate per trade (with
                        leverage)
                      </p>
                    </div>

                    {/* Stop Loss */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="text-sm font-medium">Stop Loss</label>
                        <span className="text-sm text-muted-foreground">
                          {config.stop_loss_pct}%
                        </span>
                      </div>
                      <Slider
                        value={[config.stop_loss_pct]}
                        onValueChange={([value]) =>
                          updateSymbolConfig(config.symbol, {
                            stop_loss_pct: value,
                          })
                        }
                        min={0.5}
                        max={5}
                        step={0.1}
                        className="w-full"
                      />
                      <p className="text-xs text-muted-foreground">
                        Maximum loss before automatically closing position
                      </p>
                    </div>

                    {/* Take Profit */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="text-sm font-medium">
                          Take Profit
                        </label>
                        <span className="text-sm text-muted-foreground">
                          {config.take_profit_pct}%
                        </span>
                      </div>
                      <Slider
                        value={[config.take_profit_pct]}
                        onValueChange={([value]) =>
                          updateSymbolConfig(config.symbol, {
                            take_profit_pct: value,
                          })
                        }
                        min={1}
                        max={10}
                        step={0.5}
                        className="w-full"
                      />
                      <p className="text-xs text-muted-foreground">
                        Target profit before automatically closing position
                      </p>
                    </div>

                    {/* Max Positions */}
                    <div className="space-y-2">
                      <div className="flex items-center justify-between">
                        <label className="text-sm font-medium">
                          Max Concurrent Positions
                        </label>
                        <span className="text-sm text-muted-foreground">
                          {config.max_positions}
                        </span>
                      </div>
                      <Slider
                        value={[config.max_positions]}
                        onValueChange={([value]) =>
                          updateSymbolConfig(config.symbol, {
                            max_positions: value,
                          })
                        }
                        min={1}
                        max={5}
                        step={1}
                        className="w-full"
                      />
                      <p className="text-xs text-muted-foreground">
                        Maximum number of simultaneous positions for this symbol
                      </p>
                    </div>

                    {/* Risk Summary */}
                    <div className="p-4 rounded-lg bg-secondary/50 border">
                      <div className="flex items-start gap-2">
                        <div className={cn("mt-0.5", getRiskColor(riskLevel))}>
                          {getRiskIcon(riskLevel)}
                        </div>
                        <div className="flex-1">
                          <div className="flex items-center gap-2 mb-2">
                            <span className="font-medium text-sm">
                              Risk Assessment
                            </span>
                            <Badge
                              variant="outline"
                              className={cn(
                                "text-xs",
                                getRiskColor(riskLevel)
                              )}
                            >
                              {riskLevel.toUpperCase()}
                            </Badge>
                          </div>
                          <div className="grid grid-cols-2 gap-2 text-xs">
                            <div>
                              <span className="text-muted-foreground">
                                Position Value:
                              </span>
                              <span className="ml-2 font-medium">
                                ${positionSize.toFixed(2)}
                              </span>
                            </div>
                            <div>
                              <span className="text-muted-foreground">
                                Max Loss:
                              </span>
                              <span className="ml-2 font-medium text-red-600 dark:text-red-400">
                                -$
                                {(
                                  (positionSize * config.stop_loss_pct) /
                                  100
                                ).toFixed(2)}
                              </span>
                            </div>
                            <div>
                              <span className="text-muted-foreground">
                                Target Profit:
                              </span>
                              <span className="ml-2 font-medium text-green-600 dark:text-green-400">
                                +$
                                {(
                                  (positionSize * config.take_profit_pct) /
                                  100
                                ).toFixed(2)}
                              </span>
                            </div>
                            <div>
                              <span className="text-muted-foreground">
                                Risk/Reward:
                              </span>
                              <span className="ml-2 font-medium">
                                1:
                                {(
                                  config.take_profit_pct / config.stop_loss_pct
                                ).toFixed(2)}
                              </span>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>

                    {/* Individual Save Button */}
                    <div className="flex justify-end">
                      <Button
                        size="sm"
                        variant="outline"
                        onClick={() => saveSymbolConfig(config.symbol)}
                      >
                        <Save className="h-4 w-4 mr-2" />
                        Save {config.symbol} Settings
                      </Button>
                    </div>
                  </div>
                </AccordionContent>
              </AccordionItem>
            );
          })}
        </Accordion>

        {/* Quick Preset Actions */}
        <div className="mt-6 p-4 rounded-lg bg-muted/50 border">
          <h4 className="text-sm font-medium mb-3">Quick Presets</h4>
          <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-4 gap-2">
            {DEFAULT_SYMBOLS.map((symbol) => (
              <Button
                key={symbol}
                variant="outline"
                size="sm"
                onClick={() => {
                  const preset = PRESETS[symbol];
                  if (preset) {
                    updateSymbolConfig(symbol, preset.config);
                    toast({
                      title: "Preset Applied",
                      description: `${preset.name} settings applied to ${symbol}`,
                    });
                  }
                }}
              >
                {PRESETS[symbol]?.name || symbol}
              </Button>
            ))}
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
