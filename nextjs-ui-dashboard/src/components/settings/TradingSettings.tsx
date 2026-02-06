/**
 * TradingSettings Component
 *
 * Trading preferences including default leverage, risk limits, and auto-close settings.
 * Auto-saves changes with debounce.
 *
 * Server-backed settings (via /api/paper-trading/basic-settings):
 * - defaultLeverage, maxPositionSize, defaultStopLoss, defaultTakeProfit
 *
 * Client-only settings (localStorage):
 * - autoCloseOnProfit, autoCloseOnLoss, paperModeByDefault, confirmBeforeTrade
 */

import { useState, useEffect, useCallback } from 'react';
import logger from "@/utils/logger";
import { GlassCard } from '@/components/ui/GlassCard';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Slider } from '@/components/ui/slider';
import { Switch } from '@/components/ui/switch';
import { Button } from '@/components/ui/button';
import { Save, RotateCcw, AlertCircle } from 'lucide-react';
import { debounce } from 'lodash';
import { toast } from 'sonner';

// API Base URL
const API_BASE = import.meta.env.VITE_RUST_API_URL || 'http://localhost:8080';

interface TradingSettingsData {
  defaultLeverage: number;
  maxPositionSize: number;
  defaultStopLoss: number;
  defaultTakeProfit: number;
  autoCloseOnProfit: boolean;
  autoCloseOnLoss: boolean;
  paperModeByDefault: boolean;
  confirmBeforeTrade: boolean;
}

const DEFAULT_SETTINGS: TradingSettingsData = {
  defaultLeverage: 1,
  maxPositionSize: 10000,
  defaultStopLoss: 2,
  defaultTakeProfit: 5,
  autoCloseOnProfit: false,
  autoCloseOnLoss: true,
  paperModeByDefault: true,
  confirmBeforeTrade: true,
};

export function TradingSettings() {
  const [settings, setSettings] = useState<TradingSettingsData>(DEFAULT_SETTINGS);
  const [isSaving, setIsSaving] = useState(false);
  const [isLoading, setIsLoading] = useState(true);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);
  const [loadError, setLoadError] = useState<string | null>(null);

  // Load settings from API (server-backed) and localStorage (client-only)
  useEffect(() => {
    const loadSettings = async () => {
      setIsLoading(true);
      setLoadError(null);

      try {
        // Load server-backed settings from API
        const response = await fetch(`${API_BASE}/api/paper-trading/basic-settings`);

        if (response.ok) {
          const data = await response.json();
          if (data.success && data.data) {
            const apiSettings = data.data;

            // Load client-only settings from localStorage
            const clientSettings = localStorage.getItem('tradingClientSettings');
            const clientData = clientSettings ? JSON.parse(clientSettings) : {};

            setSettings({
              // Server-backed settings
              defaultLeverage: apiSettings.default_leverage || DEFAULT_SETTINGS.defaultLeverage,
              maxPositionSize: apiSettings.max_position_size || DEFAULT_SETTINGS.maxPositionSize,
              defaultStopLoss: apiSettings.default_stop_loss_pct || DEFAULT_SETTINGS.defaultStopLoss,
              defaultTakeProfit: apiSettings.default_take_profit_pct || DEFAULT_SETTINGS.defaultTakeProfit,
              // Client-only settings (from localStorage)
              autoCloseOnProfit: clientData.autoCloseOnProfit ?? DEFAULT_SETTINGS.autoCloseOnProfit,
              autoCloseOnLoss: clientData.autoCloseOnLoss ?? DEFAULT_SETTINGS.autoCloseOnLoss,
              paperModeByDefault: clientData.paperModeByDefault ?? DEFAULT_SETTINGS.paperModeByDefault,
              confirmBeforeTrade: clientData.confirmBeforeTrade ?? DEFAULT_SETTINGS.confirmBeforeTrade,
            });
          }
        } else {
          // API not available, fall back to localStorage
          const saved = localStorage.getItem('tradingSettings');
          if (saved) {
            setSettings(JSON.parse(saved));
          }
          setLoadError('Using cached settings (API unavailable)');
        }
      } catch (error) {
        logger.error('Failed to load trading settings:', error);
        // Fall back to localStorage
        const saved = localStorage.getItem('tradingSettings');
        if (saved) {
          setSettings(JSON.parse(saved));
        }
        setLoadError('Using cached settings (connection error)');
      } finally {
        setIsLoading(false);
      }
    };
    loadSettings();
  }, []);

  // Auto-save with debounce - saves server-backed settings to API, client-only to localStorage
  const saveSettings = useCallback(
    debounce(async (newSettings: TradingSettingsData) => {
      setIsSaving(true);
      try {
        // Save server-backed settings to API
        const response = await fetch(`${API_BASE}/api/paper-trading/basic-settings`, {
          method: 'PUT',
          headers: {
            'Content-Type': 'application/json',
          },
          body: JSON.stringify({
            default_leverage: newSettings.defaultLeverage,
            default_stop_loss_pct: newSettings.defaultStopLoss,
            default_take_profit_pct: newSettings.defaultTakeProfit,
          }),
        });

        if (response.ok) {
          setLastSaved(new Date());
        } else {
          // Fallback to localStorage if API fails
          localStorage.setItem('tradingSettings', JSON.stringify(newSettings));
          setLastSaved(new Date());
          toast.error('Saved locally (API unavailable)');
        }

        // Save client-only settings to localStorage
        localStorage.setItem('tradingClientSettings', JSON.stringify({
          autoCloseOnProfit: newSettings.autoCloseOnProfit,
          autoCloseOnLoss: newSettings.autoCloseOnLoss,
          paperModeByDefault: newSettings.paperModeByDefault,
          confirmBeforeTrade: newSettings.confirmBeforeTrade,
        }));
      } catch (error) {
        logger.error('Failed to save trading settings:', error);
        // Fallback to localStorage
        localStorage.setItem('tradingSettings', JSON.stringify(newSettings));
        setLastSaved(new Date());
        toast.error('Saved locally (connection error)');
      } finally {
        setIsSaving(false);
      }
    }, 1000),
    []
  );

  const updateSetting = <K extends keyof TradingSettingsData>(
    key: K,
    value: TradingSettingsData[K]
  ) => {
    const newSettings = { ...settings, [key]: value };
    setSettings(newSettings);
    saveSettings(newSettings);
  };

  const resetToDefaults = () => {
    setSettings(DEFAULT_SETTINGS);
    saveSettings(DEFAULT_SETTINGS);
  };

  // Show loading skeleton while fetching settings
  if (isLoading) {
    return (
      <div className="space-y-6">
        <div className="flex items-center justify-between">
          <div>
            <h2 className="text-2xl font-bold text-gray-100">Trading Preferences</h2>
            <p className="text-sm text-gray-400 mt-1">Loading settings...</p>
          </div>
        </div>
        {[...Array(4)].map((_, i) => (
          <div
            key={i}
            className="h-32 rounded-xl bg-slate-800/50 animate-pulse"
          />
        ))}
      </div>
    );
  }

  return (
    <div className="space-y-6">
      {/* Error Banner */}
      {loadError && (
        <div className="flex items-center gap-2 p-3 rounded-lg bg-amber-500/10 border border-amber-500/20">
          <AlertCircle className="w-4 h-4 text-amber-500" />
          <span className="text-sm text-amber-500">{loadError}</span>
        </div>
      )}

      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-100">Trading Preferences</h2>
          <p className="text-sm text-gray-400 mt-1">
            Configure default trading parameters and risk limits
          </p>
        </div>
        <div className="flex items-center gap-2">
          {lastSaved && (
            <span className="text-xs text-gray-500">
              Saved {lastSaved.toLocaleTimeString()}
            </span>
          )}
          {isSaving && (
            <span className="text-xs text-sky-500 flex items-center gap-1">
              <Save className="w-3 h-3 animate-spin" />
              Saving...
            </span>
          )}
        </div>
      </div>

      {/* Leverage Settings */}
      <GlassCard>
        <div className="space-y-4">
          <div>
            <Label htmlFor="leverage" className="text-gray-100 text-base font-semibold">
              Default Leverage
            </Label>
            <p className="text-xs text-gray-400 mt-1">
              Applied to new trades. Can be adjusted per trade.
            </p>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <span className="text-3xl font-bold text-sky-500">
                {settings.defaultLeverage}x
              </span>
              <span className="text-sm text-gray-400">
                Max Risk: {(settings.defaultLeverage * 100).toFixed(0)}%
              </span>
            </div>

            <Slider
              id="leverage"
              min={1}
              max={125}
              step={1}
              value={[settings.defaultLeverage]}
              onValueChange={(value) => updateSetting('defaultLeverage', value[0])}
              className="w-full"
            />

            <div className="flex justify-between text-xs text-gray-500">
              <span>1x (Safe)</span>
              <span>25x</span>
              <span>50x</span>
              <span>75x</span>
              <span>125x (Max Risk)</span>
            </div>
          </div>
        </div>
      </GlassCard>

      {/* Position Limits */}
      <GlassCard>
        <div className="space-y-4">
          <div>
            <Label htmlFor="maxPosition" className="text-gray-100 text-base font-semibold">
              Max Position Size
            </Label>
            <p className="text-xs text-gray-400 mt-1">
              Maximum amount per trade in USD
            </p>
          </div>

          <div className="relative">
            <span className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400">$</span>
            <Input
              id="maxPosition"
              type="number"
              min={100}
              max={1000000}
              step={100}
              value={settings.maxPositionSize}
              onChange={(e) => updateSetting('maxPositionSize', Number(e.target.value))}
              className="pl-8 bg-slate-800/50 border-slate-700 text-gray-100"
            />
          </div>
        </div>
      </GlassCard>

      {/* Stop Loss & Take Profit */}
      <GlassCard>
        <div className="space-y-6">
          <div>
            <Label className="text-gray-100 text-base font-semibold">
              Default Stop Loss & Take Profit
            </Label>
            <p className="text-xs text-gray-400 mt-1">
              Automatically applied to new trades (percentage)
            </p>
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            {/* Stop Loss */}
            <div className="space-y-2">
              <Label htmlFor="stopLoss" className="text-gray-300">
                Stop Loss (%)
              </Label>
              <div className="relative">
                <Input
                  id="stopLoss"
                  type="number"
                  min={0.1}
                  max={50}
                  step={0.1}
                  value={settings.defaultStopLoss}
                  onChange={(e) => updateSetting('defaultStopLoss', Number(e.target.value))}
                  className="pr-8 bg-slate-800/50 border-slate-700 text-gray-100"
                />
                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400">%</span>
              </div>
            </div>

            {/* Take Profit */}
            <div className="space-y-2">
              <Label htmlFor="takeProfit" className="text-gray-300">
                Take Profit (%)
              </Label>
              <div className="relative">
                <Input
                  id="takeProfit"
                  type="number"
                  min={0.1}
                  max={100}
                  step={0.1}
                  value={settings.defaultTakeProfit}
                  onChange={(e) => updateSetting('defaultTakeProfit', Number(e.target.value))}
                  className="pr-8 bg-slate-800/50 border-slate-700 text-gray-100"
                />
                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400">%</span>
              </div>
            </div>
          </div>
        </div>
      </GlassCard>

      {/* Auto-Close Settings */}
      <GlassCard>
        <div className="space-y-4">
          <div>
            <Label className="text-gray-100 text-base font-semibold">
              Auto-Close Settings
            </Label>
            <p className="text-xs text-gray-400 mt-1">
              Automatically close positions when conditions are met
            </p>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="autoCloseProfit" className="text-gray-300">
                  Auto-close on Take Profit
                </Label>
                <p className="text-xs text-gray-400 mt-1">
                  Close position when profit target is reached
                </p>
              </div>
              <Switch
                id="autoCloseProfit"
                checked={settings.autoCloseOnProfit}
                onCheckedChange={(checked) => updateSetting('autoCloseOnProfit', checked)}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="autoCloseLoss" className="text-gray-300">
                  Auto-close on Stop Loss
                </Label>
                <p className="text-xs text-gray-400 mt-1">
                  Close position when loss limit is hit
                </p>
              </div>
              <Switch
                id="autoCloseLoss"
                checked={settings.autoCloseOnLoss}
                onCheckedChange={(checked) => updateSetting('autoCloseOnLoss', checked)}
              />
            </div>
          </div>
        </div>
      </GlassCard>

      {/* Safety Settings */}
      <GlassCard>
        <div className="space-y-4">
          <div>
            <Label className="text-gray-100 text-base font-semibold">
              Safety Settings
            </Label>
            <p className="text-xs text-gray-400 mt-1">
              Additional safety measures for trading
            </p>
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="paperMode" className="text-gray-300">
                  Paper Mode by Default
                </Label>
                <p className="text-xs text-gray-400 mt-1">
                  Start in paper trading mode for safety
                </p>
              </div>
              <Switch
                id="paperMode"
                checked={settings.paperModeByDefault}
                onCheckedChange={(checked) => updateSetting('paperModeByDefault', checked)}
              />
            </div>

            <div className="flex items-center justify-between">
              <div>
                <Label htmlFor="confirmTrade" className="text-gray-300">
                  Confirm Before Trade
                </Label>
                <p className="text-xs text-gray-400 mt-1">
                  Show confirmation dialog before executing trades
                </p>
              </div>
              <Switch
                id="confirmTrade"
                checked={settings.confirmBeforeTrade}
                onCheckedChange={(checked) => updateSetting('confirmBeforeTrade', checked)}
              />
            </div>
          </div>
        </div>
      </GlassCard>

      {/* Reset Button */}
      <div className="flex justify-end">
        <Button
          variant="outline"
          onClick={resetToDefaults}
          className="gap-2 border-slate-700 text-gray-300 hover:bg-slate-800"
        >
          <RotateCcw className="w-4 h-4" />
          Reset to Defaults
        </Button>
      </div>
    </div>
  );
}
