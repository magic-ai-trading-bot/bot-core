import React, { useState, useEffect } from "react";
import logger from "@/utils/logger";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Badge } from "@/components/ui/badge";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {


// @spec:FR-DASHBOARD-004 - Settings Management
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-INTEGRATION-039

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
import { Separator } from "@/components/ui/separator";
import { toast } from "sonner";
import {
  Settings,
  Target,
  TrendingUp,
  TrendingDown,
  AlertTriangle,
  BarChart3,
  Zap,
  Shield,
  Gauge,
  Save,
  RefreshCw,
  ChevronRight,
  Info,
} from "lucide-react";

// Trading Strategy Settings Types
interface StrategySettings {
  rsi: {
    enabled: boolean;
    period: number;
    oversold_threshold: number;
    overbought_threshold: number;
    extreme_oversold: number;
    extreme_overbought: number;
  };
  macd: {
    enabled: boolean;
    fast_period: number;
    slow_period: number;
    signal_period: number;
    histogram_threshold: number;
  };
  volume: {
    enabled: boolean;
    sma_period: number;
    spike_threshold: number;
    correlation_period: number;
  };
  bollinger: {
    enabled: boolean;
    period: number;
    multiplier: number;
    squeeze_threshold: number;
  };
}

interface RiskSettings {
  max_risk_per_trade: number;
  max_portfolio_risk: number;
  stop_loss_percent: number;
  take_profit_percent: number;
  max_leverage: number;
  max_drawdown: number;
  daily_loss_limit: number;
  max_consecutive_losses: number;
}

interface EngineSettings {
  min_confidence_threshold: number;
  signal_combination_mode: string;
  enabled_strategies: string[];
  market_condition: string;
  risk_level: string;
}

interface TradingSettingsData {
  strategies: StrategySettings;
  risk: RiskSettings;
  engine: EngineSettings;
}

// Market Condition Presets
const MARKET_PRESETS = {
  low_volatility: {
    name: "Low Volatility",
    description: "Optimized for sideways/ranging markets with low volatility",
    icon: "📊",
    settings: {
      strategies: {
        rsi: {
          enabled: true,
          period: 14,
          oversold_threshold: 35, // Less sensitive
          overbought_threshold: 65, // Less sensitive
          extreme_oversold: 25,
          extreme_overbought: 75,
        },
        macd: {
          enabled: true,
          fast_period: 8, // Faster response
          slow_period: 21, // Faster response
          signal_period: 5, // Faster signal
          histogram_threshold: 0.0005, // Lower threshold
        },
        volume: {
          enabled: true,
          sma_period: 15, // Shorter period
          spike_threshold: 1.3, // Lower threshold
          correlation_period: 8,
        },
        bollinger: {
          enabled: true,
          period: 15, // Shorter period
          multiplier: 1.8, // Tighter bands
          squeeze_threshold: 0.015, // Lower threshold
        },
      },
      risk: {
        max_risk_per_trade: 1.5,
        max_portfolio_risk: 15,
        stop_loss_percent: 1.5,
        take_profit_percent: 2.5,
        max_leverage: 20,
        max_drawdown: 12,
        daily_loss_limit: 4,
        max_consecutive_losses: 4,
      },
      engine: {
        min_confidence_threshold: 0.45, // Lower threshold
        signal_combination_mode: "WeightedAverage",
        enabled_strategies: [
          "RSI Strategy",
          "MACD Strategy",
          "Volume Strategy",
          "Bollinger Bands Strategy",
        ],
        market_condition: "LowVolume",
        risk_level: "Moderate",
      },
    },
  },
  normal_volatility: {
    name: "Normal Volatility",
    description: "Balanced settings for typical market conditions",
    icon: "⚖️",
    settings: {
      strategies: {
        rsi: {
          enabled: true,
          period: 14,
          oversold_threshold: 30,
          overbought_threshold: 70,
          extreme_oversold: 20,
          extreme_overbought: 80,
        },
        macd: {
          enabled: true,
          fast_period: 12,
          slow_period: 26,
          signal_period: 9,
          histogram_threshold: 0.001,
        },
        volume: {
          enabled: true,
          sma_period: 20,
          spike_threshold: 2.0,
          correlation_period: 10,
        },
        bollinger: {
          enabled: true,
          period: 20,
          multiplier: 2.0,
          squeeze_threshold: 0.02,
        },
      },
      risk: {
        max_risk_per_trade: 2.0,
        max_portfolio_risk: 20,
        stop_loss_percent: 2.0,
        take_profit_percent: 4.0,
        max_leverage: 50,
        max_drawdown: 15,
        daily_loss_limit: 5,
        max_consecutive_losses: 5,
      },
      engine: {
        min_confidence_threshold: 0.65,
        signal_combination_mode: "WeightedAverage",
        enabled_strategies: [
          "RSI Strategy",
          "MACD Strategy",
          "Volume Strategy",
          "Bollinger Bands Strategy",
        ],
        market_condition: "Trending",
        risk_level: "Moderate",
      },
    },
  },
  high_volatility: {
    name: "High Volatility",
    description: "Conservative settings for highly volatile markets",
    icon: "🚀",
    settings: {
      strategies: {
        rsi: {
          enabled: true,
          period: 21, // Longer period
          oversold_threshold: 25, // More extreme
          overbought_threshold: 75, // More extreme
          extreme_oversold: 15,
          extreme_overbought: 85,
        },
        macd: {
          enabled: true,
          fast_period: 12,
          slow_period: 26,
          signal_period: 9,
          histogram_threshold: 0.002, // Higher threshold
        },
        volume: {
          enabled: true,
          sma_period: 25, // Longer period
          spike_threshold: 3.0, // Higher threshold
          correlation_period: 12,
        },
        bollinger: {
          enabled: true,
          period: 25, // Longer period
          multiplier: 2.2, // Wider bands
          squeeze_threshold: 0.025, // Higher threshold
        },
      },
      risk: {
        max_risk_per_trade: 1.0,
        max_portfolio_risk: 10,
        stop_loss_percent: 3.0,
        take_profit_percent: 6.0,
        max_leverage: 10,
        max_drawdown: 8,
        daily_loss_limit: 3,
        max_consecutive_losses: 3,
      },
      engine: {
        min_confidence_threshold: 0.75, // Higher threshold
        signal_combination_mode: "Conservative",
        enabled_strategies: [
          "RSI Strategy",
          "MACD Strategy",
          "Volume Strategy",
          "Bollinger Bands Strategy",
        ],
        market_condition: "Volatile",
        risk_level: "Conservative",
      },
    },
  },
};

export function TradingSettings() {
  const [isOpen, setIsOpen] = useState(false);
  const [settings, setSettings] = useState<TradingSettingsData>(
    () => MARKET_PRESETS.normal_volatility.settings
  );
  const [selectedPreset, setSelectedPreset] = useState("normal_volatility");
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);

  // Load settings from backend
  const loadSettings = async () => {
    try {
      setIsLoading(true);
      const response = await fetch(
        "http://localhost:8080/api/paper-trading/strategy-settings"
      );
      if (response.ok) {
        const data = await response.json();
        if (data.success && data.data) {
          setSettings(data.data);
        }
      }
    } catch (error) {
      logger.error("Failed to load settings:", error);
      toast.error("Failed to load trading settings");
    } finally {
      setIsLoading(false);
    }
  };

  // Save settings to backend
  const saveSettings = async () => {
    try {
      setIsSaving(true);
      const response = await fetch(
        "http://localhost:8080/api/paper-trading/strategy-settings",
        {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            settings: settings,
          }),
        }
      );

      if (response.ok) {
        toast.success("Trading settings saved successfully!");
      } else {
        throw new Error("Failed to save settings");
      }
    } catch (error) {
      logger.error("Failed to save settings:", error);
      toast.error("Failed to save trading settings");
    } finally {
      setIsSaving(false);
    }
  };

  // Apply preset
  const applyPreset = (presetKey: string) => {
    const preset = MARKET_PRESETS[presetKey as keyof typeof MARKET_PRESETS];
    if (preset) {
      setSettings(preset.settings);
      setSelectedPreset(presetKey);
      toast.success(`Applied ${preset.name} preset`);
    }
  };

  // Load settings on mount
  useEffect(() => {
    if (isOpen) {
      loadSettings();
    }
  }, [isOpen]);

  return (
    <Dialog open={isOpen} onOpenChange={setIsOpen}>
      <DialogTrigger asChild>
        <Button variant="outline" className="gap-2">
          <Settings className="h-4 w-4" />
          Trading Settings
        </Button>
      </DialogTrigger>
      <DialogContent className="max-w-4xl h-[80vh] flex flex-col">
        <DialogHeader>
          <DialogTitle className="flex items-center gap-2">
            <Settings className="h-5 w-5" />
            Trading Bot Settings
            <Badge variant="secondary" className="ml-2">
              Advanced Configuration
            </Badge>
          </DialogTitle>
        </DialogHeader>

        <div className="flex-1 overflow-y-auto">
          <Tabs defaultValue="presets" className="w-full">
            <TabsList className="grid w-full grid-cols-4">
              <TabsTrigger value="presets">Market Presets</TabsTrigger>
              <TabsTrigger value="strategies">Strategies</TabsTrigger>
              <TabsTrigger value="risk">Risk Management</TabsTrigger>
              <TabsTrigger value="engine">Engine Settings</TabsTrigger>
            </TabsList>

            {/* Market Presets Tab */}
            <TabsContent value="presets" className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {Object.entries(MARKET_PRESETS).map(([key, preset]) => (
                  <Card
                    key={key}
                    className={`cursor-pointer transition-all ${
                      selectedPreset === key ? "ring-2 ring-primary" : ""
                    }`}
                    onClick={() => applyPreset(key)}
                  >
                    <CardHeader className="pb-3">
                      <CardTitle className="text-lg flex items-center gap-2">
                        <span className="text-2xl">{preset.icon}</span>
                        {preset.name}
                      </CardTitle>
                      <p className="text-sm text-muted-foreground">
                        {preset.description}
                      </p>
                    </CardHeader>
                    <CardContent>
                      <div className="space-y-2">
                        <div className="flex justify-between text-sm">
                          <span>Confidence Threshold:</span>
                          <span className="font-mono">
                            {(
                              preset.settings.engine.min_confidence_threshold *
                              100
                            ).toFixed(0)}
                            %
                          </span>
                        </div>
                        <div className="flex justify-between text-sm">
                          <span>Max Risk per Trade:</span>
                          <span className="font-mono">
                            {preset.settings.risk.max_risk_per_trade}%
                          </span>
                        </div>
                        <div className="flex justify-between text-sm">
                          <span>Stop Loss:</span>
                          <span className="font-mono">
                            {preset.settings.risk.stop_loss_percent}%
                          </span>
                        </div>
                      </div>
                    </CardContent>
                  </Card>
                ))}
              </div>

              <div className="mt-6 p-4 bg-blue-50 dark:bg-blue-950 rounded-lg">
                <div className="flex items-start gap-3">
                  <Info className="h-5 w-5 text-blue-600 mt-0.5" />
                  <div>
                    <h4 className="font-semibold text-blue-900 dark:text-blue-100">
                      Low Volatility Market Recommendations
                    </h4>
                    <p className="text-sm text-blue-700 dark:text-blue-300 mt-1">
                      In low volatility markets, the bot uses more sensitive
                      parameters to catch smaller price movements. This includes
                      lower RSI thresholds, faster MACD periods, and reduced
                      confidence requirements.
                    </p>
                  </div>
                </div>
              </div>
            </TabsContent>

            {/* Strategies Tab */}
            <TabsContent value="strategies" className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                {/* RSI Strategy */}
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <TrendingUp className="h-4 w-4" />
                      RSI Strategy
                      <Switch
                        checked={settings.strategies.rsi.enabled}
                        onCheckedChange={(checked) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              rsi: { ...prev.strategies.rsi, enabled: checked },
                            },
                          }))
                        }
                      />
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="space-y-2">
                      <Label>
                        RSI Period: {settings.strategies.rsi.period}
                      </Label>
                      <Slider
                        value={[settings.strategies.rsi.period]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              rsi: { ...prev.strategies.rsi, period: value },
                            },
                          }))
                        }
                        min={5}
                        max={30}
                        step={1}
                        className="w-full"
                      />
                    </div>
                    <div className="grid grid-cols-2 gap-4">
                      <div>
                        <Label>
                          Oversold: {settings.strategies.rsi.oversold_threshold}
                        </Label>
                        <Slider
                          value={[settings.strategies.rsi.oversold_threshold]}
                          onValueChange={([value]) =>
                            setSettings((prev) => ({
                              ...prev,
                              strategies: {
                                ...prev.strategies,
                                rsi: {
                                  ...prev.strategies.rsi,
                                  oversold_threshold: value,
                                },
                              },
                            }))
                          }
                          min={20}
                          max={50}
                          step={1}
                          className="w-full"
                        />
                      </div>
                      <div>
                        <Label>
                          Overbought:{" "}
                          {settings.strategies.rsi.overbought_threshold}
                        </Label>
                        <Slider
                          value={[settings.strategies.rsi.overbought_threshold]}
                          onValueChange={([value]) =>
                            setSettings((prev) => ({
                              ...prev,
                              strategies: {
                                ...prev.strategies,
                                rsi: {
                                  ...prev.strategies.rsi,
                                  overbought_threshold: value,
                                },
                              },
                            }))
                          }
                          min={50}
                          max={80}
                          step={1}
                          className="w-full"
                        />
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* MACD Strategy */}
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <BarChart3 className="h-4 w-4" />
                      MACD Strategy
                      <Switch
                        checked={settings.strategies.macd.enabled}
                        onCheckedChange={(checked) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              macd: {
                                ...prev.strategies.macd,
                                enabled: checked,
                              },
                            },
                          }))
                        }
                      />
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div className="grid grid-cols-3 gap-4">
                      <div>
                        <Label>
                          Fast: {settings.strategies.macd.fast_period}
                        </Label>
                        <Slider
                          value={[settings.strategies.macd.fast_period]}
                          onValueChange={([value]) =>
                            setSettings((prev) => ({
                              ...prev,
                              strategies: {
                                ...prev.strategies,
                                macd: {
                                  ...prev.strategies.macd,
                                  fast_period: value,
                                },
                              },
                            }))
                          }
                          min={5}
                          max={20}
                          step={1}
                          className="w-full"
                        />
                      </div>
                      <div>
                        <Label>
                          Slow: {settings.strategies.macd.slow_period}
                        </Label>
                        <Slider
                          value={[settings.strategies.macd.slow_period]}
                          onValueChange={([value]) =>
                            setSettings((prev) => ({
                              ...prev,
                              strategies: {
                                ...prev.strategies,
                                macd: {
                                  ...prev.strategies.macd,
                                  slow_period: value,
                                },
                              },
                            }))
                          }
                          min={15}
                          max={35}
                          step={1}
                          className="w-full"
                        />
                      </div>
                      <div>
                        <Label>
                          Signal: {settings.strategies.macd.signal_period}
                        </Label>
                        <Slider
                          value={[settings.strategies.macd.signal_period]}
                          onValueChange={([value]) =>
                            setSettings((prev) => ({
                              ...prev,
                              strategies: {
                                ...prev.strategies,
                                macd: {
                                  ...prev.strategies.macd,
                                  signal_period: value,
                                },
                              },
                            }))
                          }
                          min={3}
                          max={15}
                          step={1}
                          className="w-full"
                        />
                      </div>
                    </div>
                  </CardContent>
                </Card>

                {/* Volume Strategy */}
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Gauge className="h-4 w-4" />
                      Volume Strategy
                      <Switch
                        checked={settings.strategies.volume.enabled}
                        onCheckedChange={(checked) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              volume: {
                                ...prev.strategies.volume,
                                enabled: checked,
                              },
                            },
                          }))
                        }
                      />
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label>
                        Volume Spike Threshold:{" "}
                        {settings.strategies.volume.spike_threshold.toFixed(1)}x
                      </Label>
                      <Slider
                        value={[settings.strategies.volume.spike_threshold]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              volume: {
                                ...prev.strategies.volume,
                                spike_threshold: value,
                              },
                            },
                          }))
                        }
                        min={1.0}
                        max={5.0}
                        step={0.1}
                        className="w-full"
                      />
                    </div>
                    <div>
                      <Label>
                        SMA Period: {settings.strategies.volume.sma_period}
                      </Label>
                      <Slider
                        value={[settings.strategies.volume.sma_period]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              volume: {
                                ...prev.strategies.volume,
                                sma_period: value,
                              },
                            },
                          }))
                        }
                        min={10}
                        max={30}
                        step={1}
                        className="w-full"
                      />
                    </div>
                  </CardContent>
                </Card>

                {/* Bollinger Bands Strategy */}
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Target className="h-4 w-4" />
                      Bollinger Bands
                      <Switch
                        checked={settings.strategies.bollinger.enabled}
                        onCheckedChange={(checked) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              bollinger: {
                                ...prev.strategies.bollinger,
                                enabled: checked,
                              },
                            },
                          }))
                        }
                      />
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label>
                        Period: {settings.strategies.bollinger.period}
                      </Label>
                      <Slider
                        value={[settings.strategies.bollinger.period]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              bollinger: {
                                ...prev.strategies.bollinger,
                                period: value,
                              },
                            },
                          }))
                        }
                        min={10}
                        max={30}
                        step={1}
                        className="w-full"
                      />
                    </div>
                    <div>
                      <Label>
                        Multiplier:{" "}
                        {settings.strategies.bollinger.multiplier.toFixed(1)}
                      </Label>
                      <Slider
                        value={[settings.strategies.bollinger.multiplier]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            strategies: {
                              ...prev.strategies,
                              bollinger: {
                                ...prev.strategies.bollinger,
                                multiplier: value,
                              },
                            },
                          }))
                        }
                        min={1.0}
                        max={3.0}
                        step={0.1}
                        className="w-full"
                      />
                    </div>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>

            {/* Risk Management Tab */}
            <TabsContent value="risk" className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Shield className="h-4 w-4" />
                      Position Risk
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label>
                        Max Risk per Trade: {settings.risk.max_risk_per_trade}%
                      </Label>
                      <Slider
                        value={[settings.risk.max_risk_per_trade]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            risk: { ...prev.risk, max_risk_per_trade: value },
                          }))
                        }
                        min={0.5}
                        max={5.0}
                        step={0.1}
                        className="w-full"
                      />
                    </div>
                    <div>
                      <Label>
                        Stop Loss: {settings.risk.stop_loss_percent}%
                      </Label>
                      <Slider
                        value={[settings.risk.stop_loss_percent]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            risk: { ...prev.risk, stop_loss_percent: value },
                          }))
                        }
                        min={0.5}
                        max={5.0}
                        step={0.1}
                        className="w-full"
                      />
                    </div>
                    <div>
                      <Label>
                        Take Profit: {settings.risk.take_profit_percent}%
                      </Label>
                      <Slider
                        value={[settings.risk.take_profit_percent]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            risk: { ...prev.risk, take_profit_percent: value },
                          }))
                        }
                        min={1.0}
                        max={10.0}
                        step={0.1}
                        className="w-full"
                      />
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <AlertTriangle className="h-4 w-4" />
                      Portfolio Risk
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label>
                        Max Portfolio Risk: {settings.risk.max_portfolio_risk}%
                      </Label>
                      <Slider
                        value={[settings.risk.max_portfolio_risk]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            risk: { ...prev.risk, max_portfolio_risk: value },
                          }))
                        }
                        min={5}
                        max={50}
                        step={1}
                        className="w-full"
                      />
                    </div>
                    <div>
                      <Label>Max Drawdown: {settings.risk.max_drawdown}%</Label>
                      <Slider
                        value={[settings.risk.max_drawdown]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            risk: { ...prev.risk, max_drawdown: value },
                          }))
                        }
                        min={5}
                        max={25}
                        step={1}
                        className="w-full"
                      />
                    </div>
                    <div>
                      <Label>
                        Max Consecutive Losses:{" "}
                        {settings.risk.max_consecutive_losses}
                      </Label>
                      <Slider
                        value={[settings.risk.max_consecutive_losses]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            risk: {
                              ...prev.risk,
                              max_consecutive_losses: value,
                            },
                          }))
                        }
                        min={2}
                        max={10}
                        step={1}
                        className="w-full"
                      />
                    </div>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>

            {/* Engine Settings Tab */}
            <TabsContent value="engine" className="space-y-4">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Zap className="h-4 w-4" />
                      Signal Processing
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label>
                        Min Confidence Threshold:{" "}
                        {(
                          settings.engine.min_confidence_threshold * 100
                        ).toFixed(0)}
                        %
                      </Label>
                      <Slider
                        value={[settings.engine.min_confidence_threshold]}
                        onValueChange={([value]) =>
                          setSettings((prev) => ({
                            ...prev,
                            engine: {
                              ...prev.engine,
                              min_confidence_threshold: value,
                            },
                          }))
                        }
                        min={0.3}
                        max={0.9}
                        step={0.05}
                        className="w-full"
                      />
                      <p className="text-xs text-muted-foreground mt-1">
                        Lower values = more signals (useful for low volatility)
                      </p>
                    </div>
                    <div>
                      <Label>Signal Combination Mode</Label>
                      <Select
                        value={settings.engine.signal_combination_mode}
                        onValueChange={(value) =>
                          setSettings((prev) => ({
                            ...prev,
                            engine: {
                              ...prev.engine,
                              signal_combination_mode: value,
                            },
                          }))
                        }
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="WeightedAverage">
                            Weighted Average
                          </SelectItem>
                          <SelectItem value="Consensus">Consensus</SelectItem>
                          <SelectItem value="BestConfidence">
                            Best Confidence
                          </SelectItem>
                          <SelectItem value="Conservative">
                            Conservative
                          </SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </CardContent>
                </Card>

                <Card>
                  <CardHeader>
                    <CardTitle className="flex items-center gap-2">
                      <Settings className="h-4 w-4" />
                      Market Conditions
                    </CardTitle>
                  </CardHeader>
                  <CardContent className="space-y-4">
                    <div>
                      <Label>Market Condition</Label>
                      <Select
                        value={settings.engine.market_condition}
                        onValueChange={(value) =>
                          setSettings((prev) => ({
                            ...prev,
                            engine: { ...prev.engine, market_condition: value },
                          }))
                        }
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="Trending">Trending</SelectItem>
                          <SelectItem value="Ranging">Ranging</SelectItem>
                          <SelectItem value="Volatile">Volatile</SelectItem>
                          <SelectItem value="LowVolume">Low Volume</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                    <div>
                      <Label>Risk Level</Label>
                      <Select
                        value={settings.engine.risk_level}
                        onValueChange={(value) =>
                          setSettings((prev) => ({
                            ...prev,
                            engine: { ...prev.engine, risk_level: value },
                          }))
                        }
                      >
                        <SelectTrigger>
                          <SelectValue />
                        </SelectTrigger>
                        <SelectContent>
                          <SelectItem value="Conservative">
                            Conservative
                          </SelectItem>
                          <SelectItem value="Moderate">Moderate</SelectItem>
                          <SelectItem value="Aggressive">Aggressive</SelectItem>
                        </SelectContent>
                      </Select>
                    </div>
                  </CardContent>
                </Card>
              </div>
            </TabsContent>
          </Tabs>
        </div>

        <div className="flex justify-between items-center pt-4 border-t">
          <div className="flex items-center gap-2">
            <Button
              variant="outline"
              onClick={() => loadSettings()}
              disabled={isLoading}
            >
              <RefreshCw
                className={`h-4 w-4 mr-2 ${isLoading ? "animate-spin" : ""}`}
              />
              {isLoading ? "Loading..." : "Reload"}
            </Button>
          </div>
          <div className="flex items-center gap-2">
            <Button variant="outline" onClick={() => setIsOpen(false)}>
              Cancel
            </Button>
            <Button onClick={saveSettings} disabled={isSaving}>
              <Save
                className={`h-4 w-4 mr-2 ${isSaving ? "animate-spin" : ""}`}
              />
              {isSaving ? "Saving..." : "Save Settings"}
            </Button>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
