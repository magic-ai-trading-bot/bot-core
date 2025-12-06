import React, { useState, useEffect } from "react";
import logger from "@/utils/logger";
import { Label } from "@/components/ui/label";
import { Slider } from "@/components/ui/slider";
import { Switch } from "@/components/ui/switch";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
// @spec:FR-DASHBOARD-004 - Settings Management
// @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
// @test:TC-INTEGRATION-039
import {
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
import { toast } from "sonner";
import { motion } from "framer-motion";
import {
  Settings,
  Target,
  TrendingUp,
  AlertTriangle,
  BarChart3,
  Zap,
  Shield,
  Gauge,
  Save,
  RefreshCw,
  Info,
  Check,
  Activity,
} from "lucide-react";
import {
  luxuryColors,
  GlassCard,
  GradientText,
  PremiumButton,
  Badge,
  GlowIcon,
} from "@/styles/luxury-design-system";

// API Base URL - using environment variable with fallback
const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

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
  stochastic: {
    enabled: boolean;
    k_period: number;
    d_period: number;
    oversold_threshold: number;
    overbought_threshold: number;
    extreme_oversold: number;
    extreme_overbought: number;
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
  correlation_limit: number;
}

interface EngineSettings {
  min_confidence_threshold: number;
  signal_combination_mode: string;
  enabled_strategies: string[];
  market_condition: string;
  risk_level: string;
  data_resolution?: string;
}

interface TradingSettingsData {
  strategies: StrategySettings;
  risk: RiskSettings;
  engine: EngineSettings;
  market_preset?: string;
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
          oversold_threshold: 35,
          overbought_threshold: 65,
          extreme_oversold: 25,
          extreme_overbought: 75,
        },
        macd: {
          enabled: true,
          fast_period: 8,
          slow_period: 21,
          signal_period: 5,
          histogram_threshold: 0.0005,
        },
        volume: {
          enabled: true,
          sma_period: 15,
          spike_threshold: 1.3,
          correlation_period: 8,
        },
        bollinger: {
          enabled: true,
          period: 15,
          multiplier: 1.8,
          squeeze_threshold: 0.015,
        },
        stochastic: {
          enabled: true,
          k_period: 14,
          d_period: 3,
          oversold_threshold: 25,
          overbought_threshold: 75,
          extreme_oversold: 15,
          extreme_overbought: 85,
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
        correlation_limit: 0.7,
      },
      engine: {
        min_confidence_threshold: 0.45,
        signal_combination_mode: "WeightedAverage",
        enabled_strategies: [
          "RSI Strategy",
          "MACD Strategy",
          "Volume Strategy",
          "Bollinger Bands Strategy",
          "Stochastic Strategy",
        ],
        market_condition: "LowVolume",
        risk_level: "Moderate",
        data_resolution: "15m",
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
        stochastic: {
          enabled: true,
          k_period: 14,
          d_period: 3,
          oversold_threshold: 20.0,
          overbought_threshold: 80.0,
          extreme_oversold: 10.0,
          extreme_overbought: 90.0,
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
        correlation_limit: 0.7,
      },
      engine: {
        min_confidence_threshold: 0.65,
        signal_combination_mode: "WeightedAverage",
        enabled_strategies: [
          "RSI Strategy",
          "MACD Strategy",
          "Volume Strategy",
          "Bollinger Bands Strategy",
          "Stochastic Strategy",
        ],
        market_condition: "Trending",
        risk_level: "Moderate",
        data_resolution: "15m",
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
          period: 21,
          oversold_threshold: 25,
          overbought_threshold: 75,
          extreme_oversold: 15,
          extreme_overbought: 85,
        },
        macd: {
          enabled: true,
          fast_period: 12,
          slow_period: 26,
          signal_period: 9,
          histogram_threshold: 0.002,
        },
        volume: {
          enabled: true,
          sma_period: 25,
          spike_threshold: 3.0,
          correlation_period: 12,
        },
        bollinger: {
          enabled: true,
          period: 25,
          multiplier: 2.2,
          squeeze_threshold: 0.025,
        },
        stochastic: {
          enabled: true,
          k_period: 14,
          d_period: 3,
          oversold_threshold: 15.0,
          overbought_threshold: 85.0,
          extreme_oversold: 5.0,
          extreme_overbought: 95.0,
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
        correlation_limit: 0.7,
      },
      engine: {
        min_confidence_threshold: 0.75,
        signal_combination_mode: "Conservative",
        enabled_strategies: [
          "RSI Strategy",
          "MACD Strategy",
          "Volume Strategy",
          "Bollinger Bands Strategy",
          "Stochastic Strategy",
        ],
        market_condition: "Volatile",
        risk_level: "Conservative",
        data_resolution: "15m",
      },
    },
  },
};

// Premium Slider Component
const PremiumSlider = ({
  label,
  value,
  unit,
  min,
  max,
  step,
  onChange,
  description,
}: {
  label: string;
  value: number;
  unit: string;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  description?: string;
}) => (
  <div className="space-y-3">
    <div className="flex justify-between items-center">
      <Label className="text-sm" style={{ color: luxuryColors.textSecondary }}>
        {label}
      </Label>
      <span
        className="text-sm font-bold font-mono"
        style={{ color: luxuryColors.cyan }}
      >
        {value}
        {unit}
      </span>
    </div>
    <Slider
      value={[value]}
      onValueChange={([v]) => onChange(v)}
      min={min}
      max={max}
      step={step}
      className="w-full"
    />
    {description && (
      <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
        {description}
      </p>
    )}
  </div>
);

// Premium Switch Component
const PremiumSwitch = ({
  checked,
  onCheckedChange,
}: {
  checked: boolean;
  onCheckedChange?: (checked: boolean) => void;
}) => (
  <Switch
    checked={checked}
    onCheckedChange={onCheckedChange}
    className="data-[state=checked]:bg-gradient-to-r data-[state=checked]:from-profit data-[state=checked]:to-emerald-400 data-[state=unchecked]:bg-muted"
  />
);

// Strategy Card Component
const StrategyCard = ({
  icon: Icon,
  title,
  enabled,
  onToggle,
  children,
}: {
  icon: React.ElementType;
  title: string;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  children: React.ReactNode;
}) => (
  <motion.div
    initial={{ opacity: 0, y: 10 }}
    animate={{ opacity: 1, y: 0 }}
    className="rounded-xl border p-4"
    style={{
      backgroundColor: enabled
        ? "rgba(0, 217, 255, 0.05)"
        : luxuryColors.bgSecondary,
      borderColor: enabled
        ? "rgba(0, 217, 255, 0.2)"
        : luxuryColors.borderSubtle,
    }}
  >
    <div className="flex items-center justify-between mb-4">
      <div className="flex items-center gap-2">
        <GlowIcon
          icon={Icon}
          size="sm"
          color={enabled ? luxuryColors.cyan : luxuryColors.textMuted}
        />
        <span
          className="font-semibold"
          style={{
            color: enabled ? luxuryColors.textPrimary : luxuryColors.textMuted,
          }}
        >
          {title}
        </span>
      </div>
      <PremiumSwitch checked={enabled} onCheckedChange={onToggle} />
    </div>
    <div className="space-y-4">{children}</div>
  </motion.div>
);

// Preset Card Component
const PresetCard = ({
  presetKey,
  preset,
  isSelected,
  onClick,
}: {
  presetKey: string;
  preset: (typeof MARKET_PRESETS)[keyof typeof MARKET_PRESETS];
  isSelected: boolean;
  onClick: () => void;
}) => (
  <motion.div
    whileHover={{ scale: 1.02 }}
    whileTap={{ scale: 0.98 }}
    onClick={onClick}
    className="cursor-pointer rounded-xl border p-4 transition-all"
    style={{
      backgroundColor: isSelected
        ? "rgba(0, 217, 255, 0.1)"
        : luxuryColors.bgSecondary,
      borderColor: isSelected ? luxuryColors.cyan : luxuryColors.borderSubtle,
      boxShadow: isSelected ? `0 0 20px ${luxuryColors.cyan}30` : "none",
    }}
  >
    <div className="flex items-center gap-2 mb-2">
      <span className="text-2xl">{preset.icon}</span>
      <h3
        className="font-semibold"
        style={{ color: luxuryColors.textPrimary }}
      >
        {preset.name}
      </h3>
      {isSelected && (
        <div
          className="ml-auto w-5 h-5 rounded-full flex items-center justify-center"
          style={{ backgroundColor: luxuryColors.cyan }}
        >
          <Check className="h-3 w-3 text-black" />
        </div>
      )}
    </div>
    <p className="text-sm mb-4" style={{ color: luxuryColors.textMuted }}>
      {preset.description}
    </p>
    <div className="space-y-2">
      <div className="flex justify-between text-sm">
        <span style={{ color: luxuryColors.textMuted }}>
          Confidence Threshold:
        </span>
        <span className="font-mono" style={{ color: luxuryColors.cyan }}>
          {(preset.settings.engine.min_confidence_threshold * 100).toFixed(0)}%
        </span>
      </div>
      <div className="flex justify-between text-sm">
        <span style={{ color: luxuryColors.textMuted }}>Max Risk per Trade:</span>
        <span className="font-mono" style={{ color: luxuryColors.cyan }}>
          {preset.settings.risk.max_risk_per_trade}%
        </span>
      </div>
      <div className="flex justify-between text-sm">
        <span style={{ color: luxuryColors.textMuted }}>Stop Loss:</span>
        <span className="font-mono" style={{ color: luxuryColors.cyan }}>
          {preset.settings.risk.stop_loss_percent}%
        </span>
      </div>
    </div>
  </motion.div>
);

// Premium Select Component
const PremiumSelect = ({
  label,
  value,
  onChange,
  options,
  description,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: { value: string; label: string }[];
  description?: string;
}) => (
  <div className="space-y-2">
    <Label className="text-sm" style={{ color: luxuryColors.textSecondary }}>
      {label}
    </Label>
    <Select value={value} onValueChange={onChange}>
      <SelectTrigger
        className="border"
        style={{
          backgroundColor: luxuryColors.bgTertiary,
          borderColor: luxuryColors.borderSubtle,
          color: luxuryColors.textPrimary,
        }}
      >
        <SelectValue />
      </SelectTrigger>
      <SelectContent
        style={{
          backgroundColor: luxuryColors.bgSecondary,
          borderColor: luxuryColors.borderSubtle,
        }}
      >
        {options.map((opt) => (
          <SelectItem
            key={opt.value}
            value={opt.value}
            style={{ color: luxuryColors.textPrimary }}
          >
            {opt.label}
          </SelectItem>
        ))}
      </SelectContent>
    </Select>
    {description && (
      <p className="text-xs" style={{ color: luxuryColors.textMuted }}>
        {description}
      </p>
    )}
  </div>
);

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
        `${API_BASE}/api/paper-trading/strategy-settings`
      );
      if (response.ok) {
        const data = await response.json();
        if (data.success && data.data) {
          setSettings(data.data);
          if (data.data.market_preset) {
            setSelectedPreset(data.data.market_preset);
          }
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
        `${API_BASE}/api/paper-trading/strategy-settings`,
        {
          method: "PUT",
          headers: {
            "Content-Type": "application/json",
          },
          body: JSON.stringify({
            settings: {
              ...settings,
              market_preset: selectedPreset,
            },
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
        <PremiumButton variant="secondary" size="sm">
          <Settings className="h-4 w-4" />
          Trading Settings
        </PremiumButton>
      </DialogTrigger>
      <DialogContent
        className="max-w-4xl h-[85vh] flex flex-col p-0 gap-0 border"
        style={{
          backgroundColor: luxuryColors.bgPrimary,
          borderColor: luxuryColors.borderSubtle,
        }}
      >
        {/* Header */}
        <DialogHeader
          className="px-6 py-4 border-b"
          style={{ borderColor: luxuryColors.borderSubtle }}
        >
          <DialogTitle className="flex items-center gap-3">
            <GlowIcon icon={Settings} size="md" color={luxuryColors.cyan} />
            <div>
              <GradientText className="text-xl font-bold">
                Trading Bot Settings
              </GradientText>
              <p className="text-xs font-normal" style={{ color: luxuryColors.textMuted }}>
                Advanced Configuration
              </p>
            </div>
            <Badge variant="info" size="sm" className="ml-auto">
              Paper Mode
            </Badge>
          </DialogTitle>
        </DialogHeader>

        {/* Content */}
        <div className="flex-1 overflow-y-auto px-6 py-4 custom-scrollbar">
          <Tabs defaultValue="presets" className="w-full">
            <TabsList
              className="grid w-full grid-cols-4 mb-6 p-1 rounded-xl"
              style={{ backgroundColor: luxuryColors.bgSecondary }}
            >
              {[
                { value: "presets", label: "Market Presets" },
                { value: "strategies", label: "Strategies" },
                { value: "risk", label: "Risk Management" },
                { value: "engine", label: "Engine Settings" },
              ].map((tab) => (
                <TabsTrigger
                  key={tab.value}
                  value={tab.value}
                  className="rounded-lg data-[state=active]:bg-gradient-to-r data-[state=active]:from-cyan-500/20 data-[state=active]:to-blue-500/20 data-[state=active]:text-white transition-all"
                  style={{ color: luxuryColors.textMuted }}
                >
                  {tab.label}
                </TabsTrigger>
              ))}
            </TabsList>

            {/* Market Presets Tab */}
            <TabsContent value="presets" className="space-y-6 mt-0">
              <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
                {Object.entries(MARKET_PRESETS).map(([key, preset]) => (
                  <PresetCard
                    key={key}
                    presetKey={key}
                    preset={preset}
                    isSelected={selectedPreset === key}
                    onClick={() => applyPreset(key)}
                  />
                ))}
              </div>

              {/* Info Box */}
              <div
                className="p-4 rounded-xl border flex items-start gap-3"
                style={{
                  backgroundColor: "rgba(0, 217, 255, 0.05)",
                  borderColor: "rgba(0, 217, 255, 0.2)",
                }}
              >
                <Info className="h-5 w-5 mt-0.5" style={{ color: luxuryColors.cyan }} />
                <div>
                  <h4
                    className="font-semibold text-sm"
                    style={{ color: luxuryColors.cyan }}
                  >
                    {MARKET_PRESETS[selectedPreset as keyof typeof MARKET_PRESETS]?.name} Recommendations
                  </h4>
                  <p
                    className="text-sm mt-1"
                    style={{ color: luxuryColors.textSecondary }}
                  >
                    {selectedPreset === "low_volatility" &&
                      "In low volatility markets, the bot uses more sensitive parameters to catch smaller price movements. This includes lower RSI thresholds, faster MACD periods, and reduced confidence requirements."}
                    {selectedPreset === "normal_volatility" &&
                      "Normal volatility settings provide balanced parameters suitable for most market conditions. Standard RSI, MACD, and confidence thresholds are used."}
                    {selectedPreset === "high_volatility" &&
                      "In high volatility markets, conservative settings are applied with stricter thresholds and lower risk exposure to protect against sharp price swings."}
                  </p>
                </div>
              </div>
            </TabsContent>

            {/* Strategies Tab */}
            <TabsContent value="strategies" className="space-y-4 mt-0">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* RSI Strategy */}
                <StrategyCard
                  icon={TrendingUp}
                  title="RSI Strategy"
                  enabled={settings.strategies.rsi.enabled}
                  onToggle={(checked) =>
                    setSettings((prev) => ({
                      ...prev,
                      strategies: {
                        ...prev.strategies,
                        rsi: { ...prev.strategies.rsi, enabled: checked },
                      },
                    }))
                  }
                >
                  <PremiumSlider
                    label="RSI Period"
                    value={settings.strategies.rsi.period}
                    unit=""
                    min={5}
                    max={30}
                    step={1}
                    onChange={(value) =>
                      setSettings((prev) => ({
                        ...prev,
                        strategies: {
                          ...prev.strategies,
                          rsi: { ...prev.strategies.rsi, period: value },
                        },
                      }))
                    }
                  />
                  <div className="grid grid-cols-2 gap-3">
                    <PremiumSlider
                      label="Oversold"
                      value={settings.strategies.rsi.oversold_threshold}
                      unit=""
                      min={20}
                      max={50}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          strategies: {
                            ...prev.strategies,
                            rsi: { ...prev.strategies.rsi, oversold_threshold: value },
                          },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Overbought"
                      value={settings.strategies.rsi.overbought_threshold}
                      unit=""
                      min={50}
                      max={80}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          strategies: {
                            ...prev.strategies,
                            rsi: { ...prev.strategies.rsi, overbought_threshold: value },
                          },
                        }))
                      }
                    />
                  </div>
                </StrategyCard>

                {/* MACD Strategy */}
                <StrategyCard
                  icon={BarChart3}
                  title="MACD Strategy"
                  enabled={settings.strategies.macd.enabled}
                  onToggle={(checked) =>
                    setSettings((prev) => ({
                      ...prev,
                      strategies: {
                        ...prev.strategies,
                        macd: { ...prev.strategies.macd, enabled: checked },
                      },
                    }))
                  }
                >
                  <div className="grid grid-cols-3 gap-3">
                    <PremiumSlider
                      label="Fast"
                      value={settings.strategies.macd.fast_period}
                      unit=""
                      min={5}
                      max={20}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          strategies: {
                            ...prev.strategies,
                            macd: { ...prev.strategies.macd, fast_period: value },
                          },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Slow"
                      value={settings.strategies.macd.slow_period}
                      unit=""
                      min={15}
                      max={35}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          strategies: {
                            ...prev.strategies,
                            macd: { ...prev.strategies.macd, slow_period: value },
                          },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Signal"
                      value={settings.strategies.macd.signal_period}
                      unit=""
                      min={3}
                      max={15}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          strategies: {
                            ...prev.strategies,
                            macd: { ...prev.strategies.macd, signal_period: value },
                          },
                        }))
                      }
                    />
                  </div>
                </StrategyCard>

                {/* Volume Strategy */}
                <StrategyCard
                  icon={Gauge}
                  title="Volume Strategy"
                  enabled={settings.strategies.volume.enabled}
                  onToggle={(checked) =>
                    setSettings((prev) => ({
                      ...prev,
                      strategies: {
                        ...prev.strategies,
                        volume: { ...prev.strategies.volume, enabled: checked },
                      },
                    }))
                  }
                >
                  <PremiumSlider
                    label="Spike Threshold"
                    value={settings.strategies.volume.spike_threshold}
                    unit="x"
                    min={1.0}
                    max={5.0}
                    step={0.1}
                    onChange={(value) =>
                      setSettings((prev) => ({
                        ...prev,
                        strategies: {
                          ...prev.strategies,
                          volume: { ...prev.strategies.volume, spike_threshold: value },
                        },
                      }))
                    }
                  />
                  <PremiumSlider
                    label="SMA Period"
                    value={settings.strategies.volume.sma_period}
                    unit=""
                    min={10}
                    max={30}
                    step={1}
                    onChange={(value) =>
                      setSettings((prev) => ({
                        ...prev,
                        strategies: {
                          ...prev.strategies,
                          volume: { ...prev.strategies.volume, sma_period: value },
                        },
                      }))
                    }
                  />
                </StrategyCard>

                {/* Bollinger Bands */}
                <StrategyCard
                  icon={Target}
                  title="Bollinger Bands"
                  enabled={settings.strategies.bollinger.enabled}
                  onToggle={(checked) =>
                    setSettings((prev) => ({
                      ...prev,
                      strategies: {
                        ...prev.strategies,
                        bollinger: { ...prev.strategies.bollinger, enabled: checked },
                      },
                    }))
                  }
                >
                  <PremiumSlider
                    label="Period"
                    value={settings.strategies.bollinger.period}
                    unit=""
                    min={10}
                    max={30}
                    step={1}
                    onChange={(value) =>
                      setSettings((prev) => ({
                        ...prev,
                        strategies: {
                          ...prev.strategies,
                          bollinger: { ...prev.strategies.bollinger, period: value },
                        },
                      }))
                    }
                  />
                  <PremiumSlider
                    label="Multiplier"
                    value={settings.strategies.bollinger.multiplier}
                    unit=""
                    min={1.0}
                    max={3.0}
                    step={0.1}
                    onChange={(value) =>
                      setSettings((prev) => ({
                        ...prev,
                        strategies: {
                          ...prev.strategies,
                          bollinger: { ...prev.strategies.bollinger, multiplier: value },
                        },
                      }))
                    }
                  />
                </StrategyCard>

                {/* Stochastic Strategy */}
                <StrategyCard
                  icon={Activity}
                  title="Stochastic Strategy"
                  enabled={settings.strategies.stochastic?.enabled ?? false}
                  onToggle={(checked) =>
                    setSettings((prev) => ({
                      ...prev,
                      strategies: {
                        ...prev.strategies,
                        stochastic: {
                          ...(prev.strategies.stochastic ?? {
                            k_period: 14,
                            d_period: 3,
                            oversold_threshold: 20.0,
                            overbought_threshold: 80.0,
                            extreme_oversold: 10.0,
                            extreme_overbought: 90.0,
                          }),
                          enabled: checked,
                        },
                      },
                    }))
                  }
                >
                  <div className="grid grid-cols-2 gap-3">
                    <PremiumSlider
                      label="K Period"
                      value={settings.strategies.stochastic?.k_period ?? 14}
                      unit=""
                      min={5}
                      max={30}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          strategies: {
                            ...prev.strategies,
                            stochastic: {
                              ...(prev.strategies.stochastic ?? {
                                enabled: true,
                                k_period: 14,
                                d_period: 3,
                                oversold_threshold: 20.0,
                                overbought_threshold: 80.0,
                                extreme_oversold: 10.0,
                                extreme_overbought: 90.0,
                              }),
                              k_period: value,
                            },
                          },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="D Period"
                      value={settings.strategies.stochastic?.d_period ?? 3}
                      unit=""
                      min={1}
                      max={10}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          strategies: {
                            ...prev.strategies,
                            stochastic: {
                              ...(prev.strategies.stochastic ?? {
                                enabled: true,
                                k_period: 14,
                                d_period: 3,
                                oversold_threshold: 20.0,
                                overbought_threshold: 80.0,
                                extreme_oversold: 10.0,
                                extreme_overbought: 90.0,
                              }),
                              d_period: value,
                            },
                          },
                        }))
                      }
                    />
                  </div>
                </StrategyCard>
              </div>
            </TabsContent>

            {/* Risk Management Tab */}
            <TabsContent value="risk" className="space-y-4 mt-0">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Position Risk */}
                <GlassCard>
                  <div className="flex items-center gap-2 mb-4">
                    <GlowIcon icon={Shield} size="sm" color={luxuryColors.cyan} />
                    <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                      Position Risk
                    </h3>
                  </div>
                  <div className="space-y-4">
                    <PremiumSlider
                      label="Max Risk per Trade"
                      value={settings.risk.max_risk_per_trade}
                      unit="%"
                      min={0.5}
                      max={5.0}
                      step={0.1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          risk: { ...prev.risk, max_risk_per_trade: value },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Stop Loss"
                      value={settings.risk.stop_loss_percent}
                      unit="%"
                      min={0.5}
                      max={5.0}
                      step={0.1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          risk: { ...prev.risk, stop_loss_percent: value },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Take Profit"
                      value={settings.risk.take_profit_percent}
                      unit="%"
                      min={1.0}
                      max={10.0}
                      step={0.1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          risk: { ...prev.risk, take_profit_percent: value },
                        }))
                      }
                    />
                  </div>
                </GlassCard>

                {/* Portfolio Risk */}
                <GlassCard>
                  <div className="flex items-center gap-2 mb-4">
                    <GlowIcon icon={AlertTriangle} size="sm" color={luxuryColors.warning} />
                    <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                      Portfolio Risk
                    </h3>
                  </div>
                  <div className="space-y-4">
                    <PremiumSlider
                      label="Max Portfolio Risk"
                      value={settings.risk.max_portfolio_risk}
                      unit="%"
                      min={5}
                      max={50}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          risk: { ...prev.risk, max_portfolio_risk: value },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Max Drawdown"
                      value={settings.risk.max_drawdown}
                      unit="%"
                      min={5}
                      max={25}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          risk: { ...prev.risk, max_drawdown: value },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Max Consecutive Losses"
                      value={settings.risk.max_consecutive_losses}
                      unit=""
                      min={2}
                      max={10}
                      step={1}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          risk: { ...prev.risk, max_consecutive_losses: value },
                        }))
                      }
                    />
                    <PremiumSlider
                      label="Position Correlation Limit"
                      value={settings.risk.correlation_limit * 100}
                      unit="%"
                      min={50}
                      max={100}
                      step={5}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          risk: { ...prev.risk, correlation_limit: value / 100 },
                        }))
                      }
                      description="Maximum % of positions in same direction"
                    />
                  </div>
                </GlassCard>
              </div>
            </TabsContent>

            {/* Engine Settings Tab */}
            <TabsContent value="engine" className="space-y-4 mt-0">
              <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
                {/* Signal Processing */}
                <GlassCard>
                  <div className="flex items-center gap-2 mb-4">
                    <GlowIcon icon={Zap} size="sm" color={luxuryColors.cyan} />
                    <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                      Signal Processing
                    </h3>
                  </div>
                  <div className="space-y-4">
                    <PremiumSlider
                      label="Min Confidence Threshold"
                      value={settings.engine.min_confidence_threshold * 100}
                      unit="%"
                      min={30}
                      max={90}
                      step={5}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          engine: {
                            ...prev.engine,
                            min_confidence_threshold: value / 100,
                          },
                        }))
                      }
                      description="Lower values = more signals (useful for low volatility)"
                    />
                    <PremiumSelect
                      label="Signal Combination Mode"
                      value={settings.engine.signal_combination_mode}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          engine: { ...prev.engine, signal_combination_mode: value },
                        }))
                      }
                      options={[
                        { value: "WeightedAverage", label: "Weighted Average" },
                        { value: "Consensus", label: "Consensus" },
                        { value: "BestConfidence", label: "Best Confidence" },
                        { value: "Conservative", label: "Conservative" },
                      ]}
                    />
                  </div>
                </GlassCard>

                {/* Market Conditions */}
                <GlassCard>
                  <div className="flex items-center gap-2 mb-4">
                    <GlowIcon icon={Settings} size="sm" color={luxuryColors.cyan} />
                    <h3 className="font-semibold" style={{ color: luxuryColors.textPrimary }}>
                      Market Conditions
                    </h3>
                  </div>
                  <div className="space-y-4">
                    <PremiumSelect
                      label="Market Condition"
                      value={settings.engine.market_condition}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          engine: { ...prev.engine, market_condition: value },
                        }))
                      }
                      options={[
                        { value: "Trending", label: "Trending" },
                        { value: "Ranging", label: "Ranging" },
                        { value: "Volatile", label: "Volatile" },
                        { value: "LowVolume", label: "Low Volume" },
                      ]}
                    />
                    <PremiumSelect
                      label="Risk Level"
                      value={settings.engine.risk_level}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          engine: { ...prev.engine, risk_level: value },
                        }))
                      }
                      options={[
                        { value: "Conservative", label: "Conservative" },
                        { value: "Moderate", label: "Moderate" },
                        { value: "Aggressive", label: "Aggressive" },
                      ]}
                    />
                    <PremiumSelect
                      label="Data Timeframe"
                      value={settings.engine.data_resolution || "15m"}
                      onChange={(value) =>
                        setSettings((prev) => ({
                          ...prev,
                          engine: { ...prev.engine, data_resolution: value },
                        }))
                      }
                      options={[
                        { value: "1m", label: "1 minute" },
                        { value: "3m", label: "3 minutes" },
                        { value: "5m", label: "5 minutes" },
                        { value: "15m", label: "15 minutes (Recommended)" },
                        { value: "30m", label: "30 minutes" },
                        { value: "1h", label: "1 hour" },
                        { value: "4h", label: "4 hours" },
                        { value: "1d", label: "1 day" },
                      ]}
                      description="Timeframe for trading signals and technical analysis"
                    />
                  </div>
                </GlassCard>
              </div>
            </TabsContent>
          </Tabs>
        </div>

        {/* Footer */}
        <div
          className="flex justify-between items-center px-6 py-4 border-t"
          style={{ borderColor: luxuryColors.borderSubtle }}
        >
          <PremiumButton
            variant="secondary"
            size="sm"
            onClick={() => loadSettings()}
            disabled={isLoading}
            loading={isLoading}
          >
            <RefreshCw className="h-4 w-4" />
            Reload
          </PremiumButton>
          <div className="flex items-center gap-3">
            <PremiumButton
              variant="secondary"
              size="sm"
              onClick={() => setIsOpen(false)}
            >
              Cancel
            </PremiumButton>
            <PremiumButton
              variant="primary"
              size="sm"
              onClick={saveSettings}
              disabled={isSaving}
              loading={isSaving}
            >
              <Save className="h-4 w-4" />
              Save Settings
            </PremiumButton>
          </div>
        </div>
      </DialogContent>
    </Dialog>
  );
}
