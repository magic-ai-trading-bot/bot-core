import { useState, useEffect, useCallback } from "react";
import { motion, AnimatePresence } from "framer-motion";
import logger from "@/utils/logger";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { useToast } from "@/hooks/use-toast";
import { useThemeColors } from "@/hooks/useThemeColors";
import {
  TrendingUp,
  AlertTriangle,
  Shield,
  RotateCcw,
  Save,
  Loader2,
  ChevronDown,
  Coins,
} from "lucide-react";
import {
  GlassCard,
  PremiumButton,
  Badge,
  GlowIcon,
  luxuryColors,
} from "@/styles/luxury-design-system";

// @spec:FR-TRADING-015 - Per-Symbol Configuration
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

// FALLBACK symbols - actual symbols are fetched dynamically from /api/market/symbols
const FALLBACK_SYMBOLS = ["BTCUSDT", "ETHUSDT", "BNBUSDT", "SOLUSDT"];

// Default config for symbols not in PRESETS
const DEFAULT_CONFIG: Omit<SymbolConfig, "symbol"> = {
  enabled: false,
  leverage: 5,
  position_size_pct: 3,
  stop_loss_pct: 2,
  take_profit_pct: 4,
  max_positions: 1,
};

interface PerSymbolSettingsProps {
  currentBalance?: number;
  onSettingsUpdate?: (configs: SymbolConfig[]) => void;
}

export function PerSymbolSettings({
  currentBalance = 10000,
  onSettingsUpdate,
}: PerSymbolSettingsProps) {
  const { toast } = useToast();
  const colors = useThemeColors();
  const [configs, setConfigs] = useState<SymbolConfig[]>([]);
  const [isLoading, setIsLoading] = useState(false);
  const [isSaving, setIsSaving] = useState(false);
  const [expandedSymbol, setExpandedSymbol] = useState<string | null>(null);
  // Dynamic symbols from API (includes user-added symbols from database)
  const [availableSymbols, setAvailableSymbols] = useState<string[]>([]);
  const [isLoadingSymbols, setIsLoadingSymbols] = useState(true);

  const API_BASE = import.meta.env.VITE_RUST_API_URL || "http://localhost:8080";

  /**
   * Fetch symbols dynamically from API
   */
  const fetchSymbols = useCallback(async (): Promise<string[]> => {
    setIsLoadingSymbols(true);
    try {
      const response = await fetch(`${API_BASE}/api/market/symbols`);
      const data = await response.json();
      // FIX: API returns {success: true, data: {symbols: [...]}} - access data.data.symbols
      if (data.success && data.data && data.data.symbols && data.data.symbols.length > 0) {
        const symbols = data.data.symbols;
        setAvailableSymbols(symbols);
        logger.info(`Loaded ${symbols.length} symbols for per-symbol settings`);
        return symbols;
      } else {
        setAvailableSymbols(FALLBACK_SYMBOLS);
        logger.warn("Using fallback symbols for per-symbol settings");
        return FALLBACK_SYMBOLS;
      }
    } catch (error) {
      logger.error("Failed to fetch symbols:", error);
      setAvailableSymbols(FALLBACK_SYMBOLS);
      return FALLBACK_SYMBOLS;
    } finally {
      setIsLoadingSymbols(false);
    }
  }, [API_BASE]);

  /**
   * Initialize with preset configurations using provided symbols
   */
  const initializePresetsWithSymbols = useCallback((symbols: string[]) => {
    const initialConfigs = symbols.map((symbol) => ({
      symbol,
      ...(PRESETS[symbol]?.config || DEFAULT_CONFIG),
    }));
    setConfigs(initialConfigs);
  }, []);

  /**
   * Load symbols first, then configs from backend
   */
  useEffect(() => {
    const loadData = async () => {
      setIsLoading(true);
      try {
        // First, fetch available symbols from API
        const symbols = await fetchSymbols();

        // Then, try to load configs from backend
        const response = await fetch(
          `${API_BASE}/api/paper-trading/symbol-settings`
        );

        if (response.ok) {
          const data = await response.json();
          if (data.success && data.data && data.data.length > 0) {
            setConfigs(data.data);
          } else {
            // Initialize with presets using dynamic symbols
            initializePresetsWithSymbols(symbols);
          }
        } else {
          // Initialize with presets on error
          initializePresetsWithSymbols(symbols);
        }
      } catch {
        // Initialize with presets on error using available symbols
        initializePresetsWithSymbols(availableSymbols.length > 0 ? availableSymbols : FALLBACK_SYMBOLS);
      } finally {
        setIsLoading(false);
      }
    };

    loadData();
  }, [API_BASE, fetchSymbols, initializePresetsWithSymbols]);

  /**
   * Reset to default presets using dynamic symbols
   */
  const initializePresets = useCallback(() => {
    const symbols = availableSymbols.length > 0 ? availableSymbols : FALLBACK_SYMBOLS;
    initializePresetsWithSymbols(symbols);
  }, [availableSymbols, initializePresetsWithSymbols]);

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
   * Get risk color
   */
  const getRiskColor = (level: RiskLevel) => {
    switch (level) {
      case "low":
        return colors.profit;
      case "moderate":
        return colors.warning;
      case "high":
        return colors.loss;
    }
  };

  /**
   * Get risk badge variant
   */
  const getRiskBadgeVariant = (level: RiskLevel): "success" | "warning" | "error" => {
    switch (level) {
      case "low":
        return "success";
      case "moderate":
        return "warning";
      case "high":
        return "error";
    }
  };

  /**
   * Get risk icon
   */
  const getRiskIcon = (level: RiskLevel) => {
    switch (level) {
      case "low":
        return Shield;
      case "moderate":
        return AlertTriangle;
      case "high":
        return TrendingUp;
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
   * Calculate position size in dollars
   */
  const calculatePositionSize = (positionSizePct: number, leverage: number) => {
    const baseSize = (currentBalance * positionSizePct) / 100;
    return baseSize * leverage;
  };

  if (isLoading) {
    return (
      <GlassCard>
        <div className="flex items-center justify-center py-12">
          <Loader2 className="h-8 w-8 animate-spin" style={{ color: colors.cyan }} />
          <span className="ml-3" style={{ color: colors.textMuted }}>
            Loading symbol settings...
          </span>
        </div>
      </GlassCard>
    );
  }

  return (
    <div className="space-y-4">
      {/* Header with Actions */}
      <GlassCard>
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-3">
            <GlowIcon icon={Coins} size="md" color={colors.cyan} />
            <div>
              <h3 className="font-semibold" style={{ color: colors.textPrimary }}>
                Per-Symbol Settings
              </h3>
              <p className="text-xs" style={{ color: colors.textMuted }}>
                Configure trading parameters for each symbol individually
              </p>
            </div>
          </div>
          <div className="flex gap-2">
            <PremiumButton
              variant="secondary"
              size="sm"
              onClick={resetToDefaults}
              disabled={isSaving}
            >
              <RotateCcw className="h-4 w-4" />
              Reset
            </PremiumButton>
            <PremiumButton
              variant="primary"
              size="sm"
              onClick={saveAllConfigs}
              disabled={isSaving}
              loading={isSaving}
            >
              <Save className="h-4 w-4" />
              Save All
            </PremiumButton>
          </div>
        </div>
      </GlassCard>

      {/* Symbol Cards */}
      <div className="space-y-3">
        {configs.map((config) => {
          const riskLevel = calculateRiskLevel(
            config.leverage,
            config.position_size_pct
          );
          const positionSize = calculatePositionSize(
            config.position_size_pct,
            config.leverage
          );
          const isExpanded = expandedSymbol === config.symbol;
          const RiskIcon = getRiskIcon(riskLevel);

          return (
            <GlassCard key={config.symbol} noPadding>
              {/* Symbol Header - Always Visible */}
              <div
                className="p-4 cursor-pointer transition-all duration-200"
                onClick={() => setExpandedSymbol(isExpanded ? null : config.symbol)}
                style={{
                  borderBottom: isExpanded ? `1px solid ${colors.borderSubtle}` : 'none',
                }}
              >
                <div className="flex items-center gap-4">
                  {/* Enable/Disable Switch */}
                  <div onClick={(e) => e.stopPropagation()}>
                    <Switch
                      checked={config.enabled}
                      onCheckedChange={(checked) =>
                        updateSymbolConfig(config.symbol, { enabled: checked })
                      }
                    />
                  </div>

                  {/* Symbol Info */}
                  <div className="flex-1">
                    <div className="flex items-center gap-3">
                      <span className="font-bold text-lg" style={{ color: colors.textPrimary }}>
                        {config.symbol}
                      </span>
                      <Badge variant={config.enabled ? "success" : "default"} glow={config.enabled}>
                        {config.enabled ? "Active" : "Disabled"}
                      </Badge>
                    </div>
                    <div className="flex items-center gap-4 mt-1 text-xs" style={{ color: colors.textMuted }}>
                      <span>{config.leverage}x Leverage</span>
                      <span>•</span>
                      <span>{config.position_size_pct}% Position</span>
                      <span>•</span>
                      <div className="flex items-center gap-1" style={{ color: getRiskColor(riskLevel) }}>
                        <RiskIcon className="h-3 w-3" />
                        <span className="capitalize">{riskLevel} Risk</span>
                      </div>
                    </div>
                  </div>

                  {/* Expand Arrow */}
                  <motion.div
                    animate={{ rotate: isExpanded ? 180 : 0 }}
                    transition={{ duration: 0.2 }}
                  >
                    <ChevronDown className="h-5 w-5" style={{ color: colors.textMuted }} />
                  </motion.div>
                </div>
              </div>

              {/* Expanded Settings */}
              <AnimatePresence>
                {isExpanded && (
                  <motion.div
                    initial={{ height: 0, opacity: 0 }}
                    animate={{ height: "auto", opacity: 1 }}
                    exit={{ height: 0, opacity: 0 }}
                    transition={{ duration: 0.2 }}
                    className="overflow-hidden"
                  >
                    <div className="p-4 space-y-6">
                      {/* Leverage */}
                      <SliderSetting
                        label="Leverage"
                        value={config.leverage}
                        unit="x"
                        min={1}
                        max={20}
                        step={1}
                        onChange={(v) => updateSymbolConfig(config.symbol, { leverage: v })}
                        color="warning"
                        description="Higher leverage increases both potential profit and risk"
                      />

                      {/* Position Size */}
                      <SliderSetting
                        label="Position Size"
                        value={config.position_size_pct}
                        unit="%"
                        min={1}
                        max={10}
                        step={0.5}
                        onChange={(v) => updateSymbolConfig(config.symbol, { position_size_pct: v })}
                        color="primary"
                        description={`$${positionSize.toFixed(2)} with leverage`}
                      />

                      {/* Stop Loss */}
                      <SliderSetting
                        label="Stop Loss"
                        value={config.stop_loss_pct}
                        unit="%"
                        min={0.5}
                        max={5}
                        step={0.1}
                        onChange={(v) => updateSymbolConfig(config.symbol, { stop_loss_pct: v })}
                        color="loss"
                        description="Maximum loss before automatically closing position"
                      />

                      {/* Take Profit */}
                      <SliderSetting
                        label="Take Profit"
                        value={config.take_profit_pct}
                        unit="%"
                        min={1}
                        max={10}
                        step={0.5}
                        onChange={(v) => updateSymbolConfig(config.symbol, { take_profit_pct: v })}
                        color="profit"
                        description="Target profit before automatically closing position"
                      />

                      {/* Max Positions */}
                      <SliderSetting
                        label="Max Concurrent Positions"
                        value={config.max_positions}
                        unit=""
                        min={1}
                        max={5}
                        step={1}
                        onChange={(v) => updateSymbolConfig(config.symbol, { max_positions: v })}
                        color="info"
                        description="Maximum simultaneous positions for this symbol"
                      />

                      {/* Risk Summary */}
                      <div
                        className="p-4 rounded-xl border"
                        style={{
                          backgroundColor: `${getRiskColor(riskLevel)}10`,
                          borderColor: `${getRiskColor(riskLevel)}30`,
                        }}
                      >
                        <div className="flex items-start gap-3">
                          <GlowIcon icon={RiskIcon} size="sm" color={getRiskColor(riskLevel)} />
                          <div className="flex-1">
                            <div className="flex items-center gap-2 mb-2">
                              <span className="font-medium text-sm" style={{ color: colors.textPrimary }}>
                                Risk Assessment
                              </span>
                              <Badge variant={getRiskBadgeVariant(riskLevel)} size="sm">
                                {riskLevel.toUpperCase()}
                              </Badge>
                            </div>
                            <div className="grid grid-cols-2 gap-3 text-xs">
                              <div>
                                <span style={{ color: colors.textMuted }}>Position Value:</span>
                                <span className="ml-2 font-medium" style={{ color: colors.textPrimary }}>
                                  ${positionSize.toFixed(2)}
                                </span>
                              </div>
                              <div>
                                <span style={{ color: colors.textMuted }}>Max Loss:</span>
                                <span className="ml-2 font-medium" style={{ color: colors.loss }}>
                                  -${((positionSize * config.stop_loss_pct) / 100).toFixed(2)}
                                </span>
                              </div>
                              <div>
                                <span style={{ color: colors.textMuted }}>Target Profit:</span>
                                <span className="ml-2 font-medium" style={{ color: colors.profit }}>
                                  +${((positionSize * config.take_profit_pct) / 100).toFixed(2)}
                                </span>
                              </div>
                              <div>
                                <span style={{ color: colors.textMuted }}>Risk/Reward:</span>
                                <span className="ml-2 font-medium" style={{ color: colors.textPrimary }}>
                                  1:{(config.take_profit_pct / config.stop_loss_pct).toFixed(2)}
                                </span>
                              </div>
                            </div>
                          </div>
                        </div>
                      </div>
                    </div>
                  </motion.div>
                )}
              </AnimatePresence>
            </GlassCard>
          );
        })}
      </div>

      {/* Quick Presets */}
      <GlassCard>
        <h4 className="text-sm font-medium mb-4" style={{ color: colors.textPrimary }}>
          Quick Presets
        </h4>
        {isLoadingSymbols ? (
          <div className="flex items-center justify-center p-4">
            <Loader2 className="h-4 w-4 animate-spin" style={{ color: colors.cyan }} />
            <span className="ml-2 text-sm" style={{ color: colors.textMuted }}>
              Loading presets...
            </span>
          </div>
        ) : (
          <div className="grid grid-cols-2 md:grid-cols-4 gap-2">
            {(availableSymbols.length > 0 ? availableSymbols : FALLBACK_SYMBOLS).map((symbol) => (
              <PremiumButton
                key={symbol}
                variant="secondary"
                size="sm"
                onClick={() => {
                  const preset = PRESETS[symbol];
                  if (preset) {
                    updateSymbolConfig(symbol, preset.config);
                    toast({
                      title: "Preset Applied",
                      description: `${preset.name} settings applied to ${symbol}`,
                    });
                  } else {
                    updateSymbolConfig(symbol, DEFAULT_CONFIG);
                    toast({
                      title: "Default Applied",
                      description: `Default settings applied to ${symbol}`,
                    });
                  }
                }}
              >
                {PRESETS[symbol]?.name || symbol}
              </PremiumButton>
            ))}
          </div>
        )}
      </GlassCard>
    </div>
  );
}

// ============================================================================
// SLIDER SETTING COMPONENT (Luxury Style)
// ============================================================================

interface SliderSettingProps {
  label: string;
  value: number;
  unit: string;
  min: number;
  max: number;
  step: number;
  onChange: (value: number) => void;
  color: "profit" | "loss" | "primary" | "warning" | "info";
  description?: string;
}

function SliderSetting({
  label,
  value,
  unit,
  min,
  max,
  step,
  onChange,
  color,
  description,
}: SliderSettingProps) {
  const colors = useThemeColors();

  const colorMap = {
    profit: luxuryColors.profit,
    loss: luxuryColors.loss,
    primary: luxuryColors.cyan,
    warning: luxuryColors.warning,
    info: luxuryColors.cyan,
  };

  return (
    <div className="space-y-2">
      <div className="flex justify-between items-center">
        <label className="text-sm" style={{ color: colors.textMuted }}>
          {label}
        </label>
        <span className="text-sm font-bold" style={{ color: colorMap[color] }}>
          {value}{unit}
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
        <p className="text-xs" style={{ color: colors.textMuted }}>
          {description}
        </p>
      )}
    </div>
  );
}
