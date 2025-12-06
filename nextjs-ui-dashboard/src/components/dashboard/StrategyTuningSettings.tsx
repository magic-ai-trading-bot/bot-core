import { Card, CardContent, CardHeader, CardTitle, CardDescription } from "@/components/ui/card";
import { PremiumButton } from "@/styles/luxury-design-system";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { Badge } from "@/components/ui/badge";
import { Label } from "@/components/ui/label";
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from "@/components/ui/select";
import { Alert, AlertDescription } from "@/components/ui/alert";
import { useState, useEffect } from "react";
import { useToast } from "@/hooks/use-toast";
import logger from "@/utils/logger";
import {
  Loader2,
  TrendingUp,
  Activity,
  BarChart3,
  Volume2,
  Settings2,
  RotateCcw,
  Download,
  Upload,
  Save,
  AlertTriangle,
  CheckCircle2,
  Info,
} from "lucide-react";

/**
 * Strategy Parameter Tuning UI Component
 *
 * Fine-tune individual strategy parameters for RSI, MACD, Bollinger Bands, and Volume strategies.
 * Provides tab-based interface with real-time validation and backend integration.
 *
 * @spec:FR-DASHBOARD-005 - Strategy Tuning UI
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
 */

// TypeScript interfaces matching Rust backend
interface RsiConfig {
  enabled: boolean;
  period: number;
  oversold_threshold: number;
  overbought_threshold: number;
  extreme_oversold: number;
  extreme_overbought: number;
}

interface MacdConfig {
  enabled: boolean;
  fast_period: number;
  slow_period: number;
  signal_period: number;
  histogram_threshold: number;
}

interface BollingerConfig {
  enabled: boolean;
  period: number;
  multiplier: number;
  squeeze_threshold: number;
}

interface VolumeConfig {
  enabled: boolean;
  sma_period: number;
  spike_threshold: number;
  correlation_period: number;
}

interface StochasticConfig {
  enabled: boolean;
  k_period: number;
  d_period: number;
  oversold_threshold: number;
  overbought_threshold: number;
  extreme_oversold: number;
  extreme_overbought: number;
}

interface StrategyConfigCollection {
  rsi: RsiConfig;
  macd: MacdConfig;
  bollinger: BollingerConfig;
  volume: VolumeConfig;
  stochastic: StochasticConfig;
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
  signal_combination_mode: "WeightedAverage" | "Consensus" | "MaxConfidence" | "Unanimous";
  enabled_strategies: string[];
  market_condition: string;
  risk_level: string;
}

interface TradingStrategySettings {
  strategies: StrategyConfigCollection;
  risk: RiskSettings;
  engine: EngineSettings;
}

// Default configurations
const DEFAULT_RSI: RsiConfig = {
  enabled: true,
  period: 14,
  oversold_threshold: 30,
  overbought_threshold: 70,
  extreme_oversold: 20,
  extreme_overbought: 80,
};

const DEFAULT_MACD: MacdConfig = {
  enabled: true,
  fast_period: 12,
  slow_period: 26,
  signal_period: 9,
  histogram_threshold: 0.001,
};

const DEFAULT_BOLLINGER: BollingerConfig = {
  enabled: true,
  period: 20,
  multiplier: 2.0,
  squeeze_threshold: 0.02,
};

const DEFAULT_VOLUME: VolumeConfig = {
  enabled: true,
  sma_period: 20,
  spike_threshold: 2.0,
  correlation_period: 10,
};

const DEFAULT_STOCHASTIC: StochasticConfig = {
  enabled: true,
  k_period: 14,
  d_period: 3,
  oversold_threshold: 20,
  overbought_threshold: 80,
  extreme_oversold: 10,
  extreme_overbought: 90,
};

const DEFAULT_ENGINE: EngineSettings = {
  min_confidence_threshold: 0.65,
  signal_combination_mode: "WeightedAverage",
  enabled_strategies: ["RSI Strategy", "MACD Strategy", "Bollinger Bands Strategy", "Volume Strategy", "Stochastic Strategy"],
  market_condition: "Trending",
  risk_level: "Moderate",
};

export function StrategyTuningSettings() {
  const { toast } = useToast();
  const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

  // State management
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [activeTab, setActiveTab] = useState("rsi");
  const [hasChanges, setHasChanges] = useState(false);

  // Strategy configurations
  const [rsiConfig, setRsiConfig] = useState<RsiConfig>(DEFAULT_RSI);
  const [macdConfig, setMacdConfig] = useState<MacdConfig>(DEFAULT_MACD);
  const [bollingerConfig, setBollingerConfig] = useState<BollingerConfig>(DEFAULT_BOLLINGER);
  const [volumeConfig, setVolumeConfig] = useState<VolumeConfig>(DEFAULT_VOLUME);
  const [stochasticConfig, setStochasticConfig] = useState<StochasticConfig>(DEFAULT_STOCHASTIC);
  const [engineConfig, setEngineConfig] = useState<EngineSettings>(DEFAULT_ENGINE);

  // Validation state
  const [validationErrors, setValidationErrors] = useState<string[]>([]);

  // Load current settings from backend
  useEffect(() => {
    const fetchSettings = async () => {
      try {
        setIsLoading(true);
        const response = await fetch(`${API_BASE}/api/paper-trading/strategy-settings`);
        const data = await response.json();

        if (data.success && data.data) {
          const settings: TradingStrategySettings = data.data;
          setRsiConfig(settings.strategies.rsi);
          setMacdConfig(settings.strategies.macd);
          setBollingerConfig(settings.strategies.bollinger);
          setVolumeConfig(settings.strategies.volume);
          setStochasticConfig(settings.strategies.stochastic || DEFAULT_STOCHASTIC);
          setEngineConfig(settings.engine);
        }
      } catch (error) {
        logger.error("Failed to load strategy settings:", error);
        toast({
          title: "Failed to Load Settings",
          description: "Could not fetch current strategy configuration",
          variant: "destructive",
        });
      } finally {
        setIsLoading(false);
      }
    };

    fetchSettings();
  }, [API_BASE, toast]);

  // Validation
  const validateSettings = (): boolean => {
    const errors: string[] = [];

    // RSI validation
    if (rsiConfig.enabled) {
      if (rsiConfig.period < 5 || rsiConfig.period > 50) {
        errors.push("RSI period must be between 5 and 50");
      }
      if (rsiConfig.oversold_threshold >= rsiConfig.overbought_threshold) {
        errors.push("RSI oversold threshold must be less than overbought threshold");
      }
      if (rsiConfig.extreme_oversold >= rsiConfig.oversold_threshold) {
        errors.push("RSI extreme oversold must be less than oversold threshold");
      }
      if (rsiConfig.extreme_overbought <= rsiConfig.overbought_threshold) {
        errors.push("RSI extreme overbought must be greater than overbought threshold");
      }
    }

    // MACD validation
    if (macdConfig.enabled) {
      if (macdConfig.fast_period >= macdConfig.slow_period) {
        errors.push("MACD fast period must be less than slow period");
      }
      if (macdConfig.fast_period < 5 || macdConfig.fast_period > 20) {
        errors.push("MACD fast period must be between 5 and 20");
      }
      if (macdConfig.slow_period < 15 || macdConfig.slow_period > 40) {
        errors.push("MACD slow period must be between 15 and 40");
      }
      if (macdConfig.signal_period < 5 || macdConfig.signal_period > 15) {
        errors.push("MACD signal period must be between 5 and 15");
      }
    }

    // Bollinger Bands validation
    if (bollingerConfig.enabled) {
      if (bollingerConfig.period < 10 || bollingerConfig.period > 30) {
        errors.push("Bollinger period must be between 10 and 30");
      }
      if (bollingerConfig.multiplier < 1.0 || bollingerConfig.multiplier > 3.0) {
        errors.push("Bollinger multiplier must be between 1.0 and 3.0");
      }
    }

    // Volume validation
    if (volumeConfig.enabled) {
      if (volumeConfig.sma_period < 10 || volumeConfig.sma_period > 30) {
        errors.push("Volume SMA period must be between 10 and 30");
      }
      if (volumeConfig.spike_threshold < 1.5 || volumeConfig.spike_threshold > 5.0) {
        errors.push("Volume spike threshold must be between 1.5 and 5.0");
      }
    }

    // Stochastic validation
    if (stochasticConfig.enabled) {
      if (stochasticConfig.k_period < 5 || stochasticConfig.k_period > 30) {
        errors.push("Stochastic K period must be between 5 and 30");
      }
      if (stochasticConfig.d_period < 1 || stochasticConfig.d_period > 10) {
        errors.push("Stochastic D period must be between 1 and 10");
      }
      if (stochasticConfig.oversold_threshold >= stochasticConfig.overbought_threshold) {
        errors.push("Stochastic oversold threshold must be less than overbought threshold");
      }
      if (stochasticConfig.extreme_oversold >= stochasticConfig.oversold_threshold) {
        errors.push("Stochastic extreme oversold must be less than oversold threshold");
      }
      if (stochasticConfig.extreme_overbought <= stochasticConfig.overbought_threshold) {
        errors.push("Stochastic extreme overbought must be greater than overbought threshold");
      }
    }

    // Engine validation
    if (engineConfig.min_confidence_threshold < 0.4 || engineConfig.min_confidence_threshold > 0.9) {
      errors.push("Confidence threshold must be between 0.4 and 0.9");
    }

    setValidationErrors(errors);
    return errors.length === 0;
  };

  // Save settings to backend
  const handleSaveSettings = async () => {
    if (!validateSettings()) {
      toast({
        title: "Validation Failed",
        description: "Please fix the errors before saving",
        variant: "destructive",
      });
      return;
    }

    setIsSaving(true);
    try {
      const settings: TradingStrategySettings = {
        strategies: {
          rsi: rsiConfig,
          macd: macdConfig,
          bollinger: bollingerConfig,
          volume: volumeConfig,
          stochastic: stochasticConfig,
        },
        risk: {
          max_risk_per_trade: 2.0,
          max_portfolio_risk: 20.0,
          stop_loss_percent: 2.0,
          take_profit_percent: 4.0,
          max_leverage: 10,
          max_drawdown: 15.0,
          daily_loss_limit: 5.0,
          max_consecutive_losses: 5,
        },
        engine: engineConfig,
      };

      const response = await fetch(`${API_BASE}/api/paper-trading/strategy-settings`, {
        method: "PUT",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ settings }),
      });

      const data = await response.json();

      if (data.success) {
        toast({
          title: "Settings Saved Successfully",
          description: "Strategy configuration has been updated",
        });
        setHasChanges(false);
      } else {
        throw new Error(data.error || "Failed to save settings");
      }
    } catch (error) {
      logger.error("Failed to save strategy settings:", error);
      toast({
        title: "Save Failed",
        description: error instanceof Error ? error.message : "Could not save strategy settings",
        variant: "destructive",
      });
    } finally {
      setIsSaving(false);
    }
  };

  // Reset to defaults
  const handleResetToDefaults = () => {
    setRsiConfig(DEFAULT_RSI);
    setMacdConfig(DEFAULT_MACD);
    setBollingerConfig(DEFAULT_BOLLINGER);
    setVolumeConfig(DEFAULT_VOLUME);
    setStochasticConfig(DEFAULT_STOCHASTIC);
    setEngineConfig(DEFAULT_ENGINE);
    setHasChanges(true);
    setValidationErrors([]);
    toast({
      title: "Reset to Defaults",
      description: "All strategy parameters have been reset",
    });
  };

  // Export configuration
  const handleExportConfig = () => {
    const config = {
      rsi: rsiConfig,
      macd: macdConfig,
      bollinger: bollingerConfig,
      volume: volumeConfig,
      stochastic: stochasticConfig,
      engine: engineConfig,
    };

    const blob = new Blob([JSON.stringify(config, null, 2)], { type: "application/json" });
    const url = URL.createObjectURL(blob);
    const a = document.createElement("a");
    a.href = url;
    a.download = `strategy-config-${new Date().toISOString().split("T")[0]}.json`;
    a.click();
    URL.revokeObjectURL(url);

    toast({
      title: "Configuration Exported",
      description: "Strategy configuration downloaded successfully",
    });
  };

  // Import configuration
  const handleImportConfig = (event: React.ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0];
    if (!file) return;

    const reader = new FileReader();
    reader.onload = (e) => {
      try {
        const config = JSON.parse(e.target?.result as string);
        setRsiConfig(config.rsi || DEFAULT_RSI);
        setMacdConfig(config.macd || DEFAULT_MACD);
        setBollingerConfig(config.bollinger || DEFAULT_BOLLINGER);
        setVolumeConfig(config.volume || DEFAULT_VOLUME);
        setStochasticConfig(config.stochastic || DEFAULT_STOCHASTIC);
        setEngineConfig(config.engine || DEFAULT_ENGINE);
        setHasChanges(true);
        toast({
          title: "Configuration Imported",
          description: "Strategy configuration loaded successfully",
        });
      } catch (error) {
        toast({
          title: "Import Failed",
          description: "Invalid configuration file",
          variant: "destructive",
        });
      }
    };
    reader.readAsText(file);
  };

  // Helper to count enabled strategies
  const getEnabledStrategiesCount = () => {
    return [rsiConfig.enabled, macdConfig.enabled, bollingerConfig.enabled, volumeConfig.enabled, stochasticConfig.enabled].filter(
      Boolean
    ).length;
  };

  if (isLoading) {
    return (
      <Card>
        <CardContent className="flex items-center justify-center h-96">
          <Loader2 className="h-8 w-8 animate-spin text-muted-foreground" />
        </CardContent>
      </Card>
    );
  }

  return (
    <Card>
      <CardHeader>
        <div className="flex items-center justify-between">
          <div>
            <CardTitle className="text-lg flex items-center gap-2">
              <Settings2 className="h-5 w-5" />
              Strategy Parameter Tuning
            </CardTitle>
            <CardDescription>
              Fine-tune individual strategy parameters for optimal performance
            </CardDescription>
          </div>
          <div className="flex items-center gap-2">
            <Badge variant={getEnabledStrategiesCount() > 0 ? "default" : "secondary"}>
              {getEnabledStrategiesCount()} / 5 Active
            </Badge>
            {hasChanges && (
              <Badge variant="outline" className="bg-warning/10 text-warning border-warning">
                Unsaved Changes
              </Badge>
            )}
          </div>
        </div>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Validation Errors */}
        {validationErrors.length > 0 && (
          <Alert variant="destructive">
            <AlertTriangle className="h-4 w-4" />
            <AlertDescription>
              <div className="font-semibold mb-1">Validation Errors:</div>
              <ul className="list-disc list-inside space-y-1">
                {validationErrors.map((error, index) => (
                  <li key={index} className="text-sm">{error}</li>
                ))}
              </ul>
            </AlertDescription>
          </Alert>
        )}

        {/* Tabs for each strategy */}
        <Tabs value={activeTab} onValueChange={setActiveTab}>
          <TabsList className="grid w-full grid-cols-6">
            <TabsTrigger value="rsi" className="flex items-center gap-1">
              <TrendingUp className="h-4 w-4" />
              <span className="hidden sm:inline">RSI</span>
            </TabsTrigger>
            <TabsTrigger value="macd" className="flex items-center gap-1">
              <Activity className="h-4 w-4" />
              <span className="hidden sm:inline">MACD</span>
            </TabsTrigger>
            <TabsTrigger value="bollinger" className="flex items-center gap-1">
              <BarChart3 className="h-4 w-4" />
              <span className="hidden sm:inline">Bollinger</span>
            </TabsTrigger>
            <TabsTrigger value="volume" className="flex items-center gap-1">
              <Volume2 className="h-4 w-4" />
              <span className="hidden sm:inline">Volume</span>
            </TabsTrigger>
            <TabsTrigger value="stochastic" className="flex items-center gap-1">
              <TrendingUp className="h-4 w-4" />
              <span className="hidden sm:inline">Stochastic</span>
            </TabsTrigger>
            <TabsTrigger value="engine" className="flex items-center gap-1">
              <Settings2 className="h-4 w-4" />
              <span className="hidden sm:inline">Engine</span>
            </TabsTrigger>
          </TabsList>

          {/* RSI Strategy Tab */}
          <TabsContent value="rsi" className="space-y-6">
            <div className="flex items-center justify-between p-4 rounded-lg bg-secondary/50 border">
              <div>
                <h3 className="font-semibold">RSI Strategy</h3>
                <p className="text-sm text-muted-foreground">
                  Relative Strength Index - Momentum oscillator
                </p>
              </div>
              <Switch
                checked={rsiConfig.enabled}
                onCheckedChange={(checked) => {
                  setRsiConfig({ ...rsiConfig, enabled: checked });
                  setHasChanges(true);
                }}
                className="data-[state=checked]:bg-profit"
              />
            </div>

            {rsiConfig.enabled && (
              <div className="space-y-6">
                {/* Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Period</Label>
                    <span className="text-sm font-semibold text-profit">{rsiConfig.period}</span>
                  </div>
                  <Slider
                    value={[rsiConfig.period]}
                    onValueChange={([value]) => {
                      setRsiConfig({ ...rsiConfig, period: value });
                      setHasChanges(true);
                    }}
                    min={5}
                    max={50}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>5 (Fast)</span>
                    <span className="text-profit font-medium">14 (Default)</span>
                    <span>50 (Slow)</span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Number of periods for RSI calculation. Lower = more sensitive, Higher = smoother
                  </p>
                </div>

                {/* Oversold Threshold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Oversold Threshold</Label>
                    <span className="text-sm font-semibold text-profit">{rsiConfig.oversold_threshold}</span>
                  </div>
                  <Slider
                    value={[rsiConfig.oversold_threshold]}
                    onValueChange={([value]) => {
                      setRsiConfig({ ...rsiConfig, oversold_threshold: value });
                      setHasChanges(true);
                    }}
                    min={10}
                    max={40}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>10 (Extreme)</span>
                    <span className="text-profit font-medium">30 (Default)</span>
                    <span>40 (Conservative)</span>
                  </div>
                </div>

                {/* Overbought Threshold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Overbought Threshold</Label>
                    <span className="text-sm font-semibold text-loss">{rsiConfig.overbought_threshold}</span>
                  </div>
                  <Slider
                    value={[rsiConfig.overbought_threshold]}
                    onValueChange={([value]) => {
                      setRsiConfig({ ...rsiConfig, overbought_threshold: value });
                      setHasChanges(true);
                    }}
                    min={60}
                    max={90}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>60 (Conservative)</span>
                    <span className="text-loss font-medium">70 (Default)</span>
                    <span>90 (Extreme)</span>
                  </div>
                </div>

                {/* Extreme Oversold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Extreme Oversold</Label>
                    <span className="text-sm font-semibold text-profit">{rsiConfig.extreme_oversold}</span>
                  </div>
                  <Slider
                    value={[rsiConfig.extreme_oversold]}
                    onValueChange={([value]) => {
                      setRsiConfig({ ...rsiConfig, extreme_oversold: value });
                      setHasChanges(true);
                    }}
                    min={5}
                    max={30}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>5 (Ultra-extreme)</span>
                    <span className="text-profit font-medium">20 (Default)</span>
                    <span>30</span>
                  </div>
                </div>

                {/* Extreme Overbought */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Extreme Overbought</Label>
                    <span className="text-sm font-semibold text-loss">{rsiConfig.extreme_overbought}</span>
                  </div>
                  <Slider
                    value={[rsiConfig.extreme_overbought]}
                    onValueChange={([value]) => {
                      setRsiConfig({ ...rsiConfig, extreme_overbought: value });
                      setHasChanges(true);
                    }}
                    min={70}
                    max={95}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>70</span>
                    <span className="text-loss font-medium">80 (Default)</span>
                    <span>95 (Ultra-extreme)</span>
                  </div>
                </div>

                {/* Helper Info */}
                <Alert>
                  <Info className="h-4 w-4" />
                  <AlertDescription className="text-xs">
                    <strong>Tip:</strong> RSI above 70 indicates overbought (potential sell), below 30 indicates
                    oversold (potential buy). Extreme levels provide stronger signals.
                  </AlertDescription>
                </Alert>
              </div>
            )}
          </TabsContent>

          {/* MACD Strategy Tab */}
          <TabsContent value="macd" className="space-y-6">
            <div className="flex items-center justify-between p-4 rounded-lg bg-secondary/50 border">
              <div>
                <h3 className="font-semibold">MACD Strategy</h3>
                <p className="text-sm text-muted-foreground">
                  Moving Average Convergence Divergence - Trend following
                </p>
              </div>
              <Switch
                checked={macdConfig.enabled}
                onCheckedChange={(checked) => {
                  setMacdConfig({ ...macdConfig, enabled: checked });
                  setHasChanges(true);
                }}
                className="data-[state=checked]:bg-profit"
              />
            </div>

            {macdConfig.enabled && (
              <div className="space-y-6">
                {/* Fast Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Fast Period (EMA)</Label>
                    <span className="text-sm font-semibold text-profit">{macdConfig.fast_period}</span>
                  </div>
                  <Slider
                    value={[macdConfig.fast_period]}
                    onValueChange={([value]) => {
                      setMacdConfig({ ...macdConfig, fast_period: value });
                      setHasChanges(true);
                    }}
                    min={5}
                    max={20}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>5 (Very Fast)</span>
                    <span className="text-profit font-medium">12 (Default)</span>
                    <span>20</span>
                  </div>
                </div>

                {/* Slow Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Slow Period (EMA)</Label>
                    <span className="text-sm font-semibold text-info">{macdConfig.slow_period}</span>
                  </div>
                  <Slider
                    value={[macdConfig.slow_period]}
                    onValueChange={([value]) => {
                      setMacdConfig({ ...macdConfig, slow_period: value });
                      setHasChanges(true);
                    }}
                    min={15}
                    max={40}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>15</span>
                    <span className="text-info font-medium">26 (Default)</span>
                    <span>40 (Very Slow)</span>
                  </div>
                  {macdConfig.fast_period >= macdConfig.slow_period && (
                    <p className="text-xs text-loss flex items-center gap-1">
                      <AlertTriangle className="h-3 w-3" />
                      Fast period must be less than slow period
                    </p>
                  )}
                </div>

                {/* Signal Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Signal Period</Label>
                    <span className="text-sm font-semibold text-warning">{macdConfig.signal_period}</span>
                  </div>
                  <Slider
                    value={[macdConfig.signal_period]}
                    onValueChange={([value]) => {
                      setMacdConfig({ ...macdConfig, signal_period: value });
                      setHasChanges(true);
                    }}
                    min={5}
                    max={15}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>5 (Responsive)</span>
                    <span className="text-warning font-medium">9 (Default)</span>
                    <span>15 (Smooth)</span>
                  </div>
                </div>

                {/* Histogram Threshold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Histogram Threshold</Label>
                    <span className="text-sm font-semibold">{macdConfig.histogram_threshold.toFixed(4)}</span>
                  </div>
                  <Slider
                    value={[macdConfig.histogram_threshold * 1000]}
                    onValueChange={([value]) => {
                      setMacdConfig({ ...macdConfig, histogram_threshold: value / 1000 });
                      setHasChanges(true);
                    }}
                    min={0}
                    max={10}
                    step={0.1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>0.0000 (Sensitive)</span>
                    <span className="font-medium">0.0010 (Default)</span>
                    <span>0.0100 (Conservative)</span>
                  </div>
                </div>

                {/* Helper Info */}
                <Alert>
                  <Info className="h-4 w-4" />
                  <AlertDescription className="text-xs">
                    <strong>Tip:</strong> MACD crossing above signal line = bullish, below = bearish. Histogram
                    shows momentum strength.
                  </AlertDescription>
                </Alert>
              </div>
            )}
          </TabsContent>

          {/* Bollinger Bands Tab */}
          <TabsContent value="bollinger" className="space-y-6">
            <div className="flex items-center justify-between p-4 rounded-lg bg-secondary/50 border">
              <div>
                <h3 className="font-semibold">Bollinger Bands Strategy</h3>
                <p className="text-sm text-muted-foreground">
                  Volatility-based bands for mean reversion
                </p>
              </div>
              <Switch
                checked={bollingerConfig.enabled}
                onCheckedChange={(checked) => {
                  setBollingerConfig({ ...bollingerConfig, enabled: checked });
                  setHasChanges(true);
                }}
                className="data-[state=checked]:bg-profit"
              />
            </div>

            {bollingerConfig.enabled && (
              <div className="space-y-6">
                {/* Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Period (SMA)</Label>
                    <span className="text-sm font-semibold text-profit">{bollingerConfig.period}</span>
                  </div>
                  <Slider
                    value={[bollingerConfig.period]}
                    onValueChange={([value]) => {
                      setBollingerConfig({ ...bollingerConfig, period: value });
                      setHasChanges(true);
                    }}
                    min={10}
                    max={30}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>10 (Responsive)</span>
                    <span className="text-profit font-medium">20 (Default)</span>
                    <span>30 (Smooth)</span>
                  </div>
                </div>

                {/* Standard Deviation Multiplier */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Standard Deviation Multiplier</Label>
                    <span className="text-sm font-semibold text-info">{bollingerConfig.multiplier.toFixed(1)}</span>
                  </div>
                  <Slider
                    value={[bollingerConfig.multiplier * 10]}
                    onValueChange={([value]) => {
                      setBollingerConfig({ ...bollingerConfig, multiplier: value / 10 });
                      setHasChanges(true);
                    }}
                    min={10}
                    max={30}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>1.0 (Tight)</span>
                    <span className="text-info font-medium">2.0 (Default)</span>
                    <span>3.0 (Wide)</span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Controls band width. Lower = tighter bands, Higher = wider bands
                  </p>
                </div>

                {/* Squeeze Threshold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Squeeze Threshold</Label>
                    <span className="text-sm font-semibold text-warning">{bollingerConfig.squeeze_threshold.toFixed(3)}</span>
                  </div>
                  <Slider
                    value={[bollingerConfig.squeeze_threshold * 1000]}
                    onValueChange={([value]) => {
                      setBollingerConfig({ ...bollingerConfig, squeeze_threshold: value / 1000 });
                      setHasChanges(true);
                    }}
                    min={10}
                    max={50}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>0.010 (Tight)</span>
                    <span className="text-warning font-medium">0.020 (Default)</span>
                    <span>0.050 (Wide)</span>
                  </div>
                </div>

                {/* Helper Info */}
                <Alert>
                  <Info className="h-4 w-4" />
                  <AlertDescription className="text-xs">
                    <strong>Tip:</strong> Price touching lower band = potential buy, upper band = potential sell.
                    Squeeze indicates low volatility before breakout.
                  </AlertDescription>
                </Alert>
              </div>
            )}
          </TabsContent>

          {/* Volume Strategy Tab */}
          <TabsContent value="volume" className="space-y-6">
            <div className="flex items-center justify-between p-4 rounded-lg bg-secondary/50 border">
              <div>
                <h3 className="font-semibold">Volume Strategy</h3>
                <p className="text-sm text-muted-foreground">
                  Volume analysis for trend confirmation
                </p>
              </div>
              <Switch
                checked={volumeConfig.enabled}
                onCheckedChange={(checked) => {
                  setVolumeConfig({ ...volumeConfig, enabled: checked });
                  setHasChanges(true);
                }}
                className="data-[state=checked]:bg-profit"
              />
            </div>

            {volumeConfig.enabled && (
              <div className="space-y-6">
                {/* SMA Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Volume SMA Period</Label>
                    <span className="text-sm font-semibold text-profit">{volumeConfig.sma_period}</span>
                  </div>
                  <Slider
                    value={[volumeConfig.sma_period]}
                    onValueChange={([value]) => {
                      setVolumeConfig({ ...volumeConfig, sma_period: value });
                      setHasChanges(true);
                    }}
                    min={10}
                    max={30}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>10 (Responsive)</span>
                    <span className="text-profit font-medium">20 (Default)</span>
                    <span>30 (Smooth)</span>
                  </div>
                </div>

                {/* Spike Threshold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Volume Spike Threshold</Label>
                    <span className="text-sm font-semibold text-warning">{volumeConfig.spike_threshold.toFixed(1)}x</span>
                  </div>
                  <Slider
                    value={[volumeConfig.spike_threshold * 10]}
                    onValueChange={([value]) => {
                      setVolumeConfig({ ...volumeConfig, spike_threshold: value / 10 });
                      setHasChanges(true);
                    }}
                    min={15}
                    max={50}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>1.5x (Sensitive)</span>
                    <span className="text-warning font-medium">2.0x (Default)</span>
                    <span>5.0x (Conservative)</span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Multiplier for detecting volume spikes above average
                  </p>
                </div>

                {/* Correlation Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Price-Volume Correlation Period</Label>
                    <span className="text-sm font-semibold text-info">{volumeConfig.correlation_period}</span>
                  </div>
                  <Slider
                    value={[volumeConfig.correlation_period]}
                    onValueChange={([value]) => {
                      setVolumeConfig({ ...volumeConfig, correlation_period: value });
                      setHasChanges(true);
                    }}
                    min={5}
                    max={20}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>5 (Short)</span>
                    <span className="text-info font-medium">10 (Default)</span>
                    <span>20 (Long)</span>
                  </div>
                </div>

                {/* Helper Info */}
                <Alert>
                  <Info className="h-4 w-4" />
                  <AlertDescription className="text-xs">
                    <strong>Tip:</strong> High volume confirms trends. Volume spikes with price movement indicate
                    strong conviction. Low volume suggests weak trends.
                  </AlertDescription>
                </Alert>
              </div>
            )}
          </TabsContent>

          {/* Stochastic Strategy Tab */}
          <TabsContent value="stochastic" className="space-y-6">
            <div className="flex items-center justify-between p-4 rounded-lg bg-secondary/50 border">
              <div>
                <h3 className="font-semibold">Stochastic Strategy</h3>
                <p className="text-sm text-muted-foreground">
                  Stochastic Oscillator - Momentum indicator
                </p>
              </div>
              <Switch
                checked={stochasticConfig.enabled}
                onCheckedChange={(checked) => {
                  setStochasticConfig({ ...stochasticConfig, enabled: checked });
                  setHasChanges(true);
                }}
                className="data-[state=checked]:bg-profit"
              />
            </div>

            {stochasticConfig.enabled && (
              <div className="space-y-6">
                {/* K Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>%K Period</Label>
                    <span className="text-sm font-semibold text-profit">{stochasticConfig.k_period}</span>
                  </div>
                  <Slider
                    value={[stochasticConfig.k_period]}
                    onValueChange={([value]) => {
                      setStochasticConfig({ ...stochasticConfig, k_period: value });
                      setHasChanges(true);
                    }}
                    min={5}
                    max={30}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>5 (Fast)</span>
                    <span className="text-profit font-medium">14 (Default)</span>
                    <span>30 (Slow)</span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Number of periods for %K calculation. Lower = more sensitive
                  </p>
                </div>

                {/* D Period */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>%D Period (Signal Line)</Label>
                    <span className="text-sm font-semibold text-info">{stochasticConfig.d_period}</span>
                  </div>
                  <Slider
                    value={[stochasticConfig.d_period]}
                    onValueChange={([value]) => {
                      setStochasticConfig({ ...stochasticConfig, d_period: value });
                      setHasChanges(true);
                    }}
                    min={1}
                    max={10}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>1 (Fast)</span>
                    <span className="text-info font-medium">3 (Default)</span>
                    <span>10 (Smooth)</span>
                  </div>
                  <p className="text-xs text-muted-foreground">
                    Smoothing period for %D line (moving average of %K)
                  </p>
                </div>

                {/* Oversold Threshold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Oversold Threshold</Label>
                    <span className="text-sm font-semibold text-profit">{stochasticConfig.oversold_threshold}</span>
                  </div>
                  <Slider
                    value={[stochasticConfig.oversold_threshold]}
                    onValueChange={([value]) => {
                      setStochasticConfig({ ...stochasticConfig, oversold_threshold: value });
                      setHasChanges(true);
                    }}
                    min={10}
                    max={30}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>10 (Extreme)</span>
                    <span className="text-profit font-medium">20 (Default)</span>
                    <span>30 (Conservative)</span>
                  </div>
                </div>

                {/* Overbought Threshold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Overbought Threshold</Label>
                    <span className="text-sm font-semibold text-loss">{stochasticConfig.overbought_threshold}</span>
                  </div>
                  <Slider
                    value={[stochasticConfig.overbought_threshold]}
                    onValueChange={([value]) => {
                      setStochasticConfig({ ...stochasticConfig, overbought_threshold: value });
                      setHasChanges(true);
                    }}
                    min={70}
                    max={90}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>70 (Conservative)</span>
                    <span className="text-loss font-medium">80 (Default)</span>
                    <span>90 (Extreme)</span>
                  </div>
                </div>

                {/* Extreme Oversold */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Extreme Oversold</Label>
                    <span className="text-sm font-semibold text-profit">{stochasticConfig.extreme_oversold}</span>
                  </div>
                  <Slider
                    value={[stochasticConfig.extreme_oversold]}
                    onValueChange={([value]) => {
                      setStochasticConfig({ ...stochasticConfig, extreme_oversold: value });
                      setHasChanges(true);
                    }}
                    min={5}
                    max={20}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>5 (Ultra-extreme)</span>
                    <span className="text-profit font-medium">10 (Default)</span>
                    <span>20</span>
                  </div>
                </div>

                {/* Extreme Overbought */}
                <div className="space-y-3">
                  <div className="flex justify-between items-center">
                    <Label>Extreme Overbought</Label>
                    <span className="text-sm font-semibold text-loss">{stochasticConfig.extreme_overbought}</span>
                  </div>
                  <Slider
                    value={[stochasticConfig.extreme_overbought]}
                    onValueChange={([value]) => {
                      setStochasticConfig({ ...stochasticConfig, extreme_overbought: value });
                      setHasChanges(true);
                    }}
                    min={80}
                    max={95}
                    step={1}
                    className="w-full"
                  />
                  <div className="flex justify-between text-xs text-muted-foreground">
                    <span>80</span>
                    <span className="text-loss font-medium">90 (Default)</span>
                    <span>95 (Ultra-extreme)</span>
                  </div>
                </div>

                {/* Helper Info */}
                <Alert>
                  <Info className="h-4 w-4" />
                  <AlertDescription className="text-xs">
                    <strong>Tip:</strong> Stochastic above 80 indicates overbought (potential sell), below 20 indicates
                    oversold (potential buy). %K crossing %D generates entry/exit signals.
                  </AlertDescription>
                </Alert>
              </div>
            )}
          </TabsContent>

          {/* Engine Settings Tab */}
          <TabsContent value="engine" className="space-y-6">
            <div className="p-4 rounded-lg bg-secondary/50 border">
              <h3 className="font-semibold mb-1">Trading Engine Configuration</h3>
              <p className="text-sm text-muted-foreground">
                Control how strategies are combined and executed
              </p>
            </div>

            <div className="space-y-6">
              {/* Min Confidence Threshold */}
              <div className="space-y-3">
                <div className="flex justify-between items-center">
                  <Label>Minimum Confidence Threshold</Label>
                  <span className="text-sm font-semibold text-profit">
                    {(engineConfig.min_confidence_threshold * 100).toFixed(0)}%
                  </span>
                </div>
                <Slider
                  value={[engineConfig.min_confidence_threshold * 100]}
                  onValueChange={([value]) => {
                    setEngineConfig({ ...engineConfig, min_confidence_threshold: value / 100 });
                    setHasChanges(true);
                  }}
                  min={40}
                  max={90}
                  step={1}
                  className="w-full"
                />
                <div className="flex justify-between text-xs text-muted-foreground">
                  <span>40% (Aggressive)</span>
                  <span className="text-profit font-medium">65% (Default)</span>
                  <span>90% (Conservative)</span>
                </div>
                <p className="text-xs text-muted-foreground">
                  Minimum confidence required for trade execution. Higher = fewer but stronger signals
                </p>
              </div>

              {/* Signal Combination Mode */}
              <div className="space-y-3">
                <Label>Signal Combination Mode</Label>
                <Select
                  value={engineConfig.signal_combination_mode}
                  onValueChange={(value) => {
                    setEngineConfig({
                      ...engineConfig,
                      signal_combination_mode: value as EngineSettings["signal_combination_mode"],
                    });
                    setHasChanges(true);
                  }}
                >
                  <SelectTrigger>
                    <SelectValue />
                  </SelectTrigger>
                  <SelectContent>
                    <SelectItem value="WeightedAverage">
                      <div>
                        <div className="font-medium">Weighted Average</div>
                        <div className="text-xs text-muted-foreground">
                          Average all strategy signals (balanced)
                        </div>
                      </div>
                    </SelectItem>
                    <SelectItem value="Consensus">
                      <div>
                        <div className="font-medium">Consensus</div>
                        <div className="text-xs text-muted-foreground">
                          Majority of strategies must agree (safe)
                        </div>
                      </div>
                    </SelectItem>
                    <SelectItem value="MaxConfidence">
                      <div>
                        <div className="font-medium">Max Confidence</div>
                        <div className="text-xs text-muted-foreground">
                          Use highest confidence strategy (aggressive)
                        </div>
                      </div>
                    </SelectItem>
                    <SelectItem value="Unanimous">
                      <div>
                        <div className="font-medium">Unanimous</div>
                        <div className="text-xs text-muted-foreground">
                          All strategies must agree (very safe)
                        </div>
                      </div>
                    </SelectItem>
                  </SelectContent>
                </Select>
              </div>

              {/* Enabled Strategies Summary */}
              <div className="space-y-3">
                <Label>Enabled Strategies</Label>
                <div className="grid grid-cols-2 gap-2">
                  <div className={`p-3 rounded-lg border ${rsiConfig.enabled ? "bg-profit/10 border-profit" : "bg-muted"}`}>
                    <div className="flex items-center gap-2">
                      <TrendingUp className="h-4 w-4" />
                      <span className="text-sm font-medium">RSI</span>
                    </div>
                    {rsiConfig.enabled && (
                      <CheckCircle2 className="h-4 w-4 text-profit mt-1" />
                    )}
                  </div>
                  <div className={`p-3 rounded-lg border ${macdConfig.enabled ? "bg-profit/10 border-profit" : "bg-muted"}`}>
                    <div className="flex items-center gap-2">
                      <Activity className="h-4 w-4" />
                      <span className="text-sm font-medium">MACD</span>
                    </div>
                    {macdConfig.enabled && (
                      <CheckCircle2 className="h-4 w-4 text-profit mt-1" />
                    )}
                  </div>
                  <div className={`p-3 rounded-lg border ${bollingerConfig.enabled ? "bg-profit/10 border-profit" : "bg-muted"}`}>
                    <div className="flex items-center gap-2">
                      <BarChart3 className="h-4 w-4" />
                      <span className="text-sm font-medium">Bollinger</span>
                    </div>
                    {bollingerConfig.enabled && (
                      <CheckCircle2 className="h-4 w-4 text-profit mt-1" />
                    )}
                  </div>
                  <div className={`p-3 rounded-lg border ${volumeConfig.enabled ? "bg-profit/10 border-profit" : "bg-muted"}`}>
                    <div className="flex items-center gap-2">
                      <Volume2 className="h-4 w-4" />
                      <span className="text-sm font-medium">Volume</span>
                    </div>
                    {volumeConfig.enabled && (
                      <CheckCircle2 className="h-4 w-4 text-profit mt-1" />
                    )}
                  </div>
                  <div className={`p-3 rounded-lg border ${stochasticConfig.enabled ? "bg-profit/10 border-profit" : "bg-muted"}`}>
                    <div className="flex items-center gap-2">
                      <TrendingUp className="h-4 w-4" />
                      <span className="text-sm font-medium">Stochastic</span>
                    </div>
                    {stochasticConfig.enabled && (
                      <CheckCircle2 className="h-4 w-4 text-profit mt-1" />
                    )}
                  </div>
                </div>
              </div>

              {/* Helper Info */}
              <Alert>
                <Info className="h-4 w-4" />
                <AlertDescription className="text-xs">
                  <strong>Recommendation:</strong> Use at least 2-3 strategies for better signal quality.
                  Weighted Average is best for beginners, Unanimous for safety.
                </AlertDescription>
              </Alert>
            </div>
          </TabsContent>
        </Tabs>

        {/* Action Buttons */}
        <div className="grid grid-cols-2 lg:grid-cols-4 gap-3 pt-4 border-t">
          <PremiumButton
            variant="secondary"
            onClick={handleResetToDefaults}
            disabled={isSaving}
            className="w-full"
          >
            <RotateCcw className="mr-2 h-4 w-4" />
            Reset
          </PremiumButton>
          <PremiumButton
            variant="secondary"
            onClick={handleExportConfig}
            disabled={isSaving}
            className="w-full"
          >
            <Download className="mr-2 h-4 w-4" />
            Export
          </PremiumButton>
          <PremiumButton
            variant="secondary"
            onClick={() => document.getElementById("import-config")?.click()}
            disabled={isSaving}
            className="w-full"
          >
            <Upload className="mr-2 h-4 w-4" />
            Import
          </PremiumButton>
          <input
            id="import-config"
            type="file"
            accept=".json"
            onChange={handleImportConfig}
            className="hidden"
          />
          <PremiumButton
            className="w-full bg-profit hover:bg-profit/90"
            onClick={handleSaveSettings}
            disabled={isSaving || validationErrors.length > 0}
          >
            {isSaving ? (
              <>
                <Loader2 className="mr-2 h-4 w-4 animate-spin" />
                Saving...
              </>
            ) : (
              <>
                <Save className="mr-2 h-4 w-4" />
                Save Settings
              </>
            )}
          </PremiumButton>
        </div>
      </CardContent>
    </Card>
  );
}
