/**
 * NotificationSettings Component
 *
 * Email, push, and trading alert preferences with toggle switches.
 * Auto-saves changes with debounce.
 */

import { useState, useEffect, useCallback } from 'react';
import logger from "@/utils/logger";
import { GlassCard } from '@/components/ui/GlassCard';
import { Label } from '@/components/ui/label';
import { Input } from '@/components/ui/input';
import { Switch } from '@/components/ui/switch';
import { Save, Mail, Bell, TrendingUp, AlertCircle } from 'lucide-react';
import { debounce } from 'lodash';

interface NotificationSettingsData {
  // Email notifications
  emailEnabled: boolean;
  emailTradeExecuted: boolean;
  emailProfitTarget: boolean;
  emailStopLoss: boolean;
  emailDailyReport: boolean;
  emailWeeklyReport: boolean;

  // Push notifications
  pushEnabled: boolean;
  pushTradeExecuted: boolean;
  pushProfitTarget: boolean;
  pushStopLoss: boolean;
  pushPriceAlert: boolean;

  // Trading alerts
  alertsEnabled: boolean;
  alertProfitThreshold: number;
  alertLossThreshold: number;
  alertSoundEnabled: boolean;

  // In-app notifications
  inAppEnabled: boolean;
}

const DEFAULT_SETTINGS: NotificationSettingsData = {
  emailEnabled: true,
  emailTradeExecuted: false,
  emailProfitTarget: true,
  emailStopLoss: true,
  emailDailyReport: false,
  emailWeeklyReport: true,

  pushEnabled: true,
  pushTradeExecuted: true,
  pushProfitTarget: true,
  pushStopLoss: true,
  pushPriceAlert: false,

  alertsEnabled: true,
  alertProfitThreshold: 10,
  alertLossThreshold: 5,
  alertSoundEnabled: true,

  inAppEnabled: true,
};

export function NotificationSettings() {
  const [settings, setSettings] = useState<NotificationSettingsData>(DEFAULT_SETTINGS);
  const [isSaving, setIsSaving] = useState(false);
  const [lastSaved, setLastSaved] = useState<Date | null>(null);

  // Load settings
  useEffect(() => {
    const loadSettings = async () => {
      try {
        const saved = localStorage.getItem('notificationSettings');
        if (saved) {
          setSettings(JSON.parse(saved));
        }
      } catch (error) {
        logger.error('Failed to load notification settings:', error);
      }
    };
    loadSettings();
  }, []);

  // Auto-save with debounce
  const saveSettings = useCallback(
    debounce(async (newSettings: NotificationSettingsData) => {
      setIsSaving(true);
      try {
        localStorage.setItem('notificationSettings', JSON.stringify(newSettings));
        await new Promise((resolve) => setTimeout(resolve, 500));
        setLastSaved(new Date());
      } catch (error) {
        logger.error('Failed to save notification settings:', error);
      } finally {
        setIsSaving(false);
      }
    }, 1000),
    []
  );

  const updateSetting = <K extends keyof NotificationSettingsData>(
    key: K,
    value: NotificationSettingsData[K]
  ) => {
    const newSettings = { ...settings, [key]: value };
    setSettings(newSettings);
    saveSettings(newSettings);
  };

  return (
    <div className="space-y-6">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h2 className="text-2xl font-bold text-gray-100">Notifications</h2>
          <p className="text-sm text-gray-400 mt-1">
            Manage how you receive alerts and updates
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

      {/* Email Notifications */}
      <GlassCard>
        <div className="space-y-4">
          <div className="flex items-center gap-2 pb-2 border-b border-slate-700">
            <Mail className="w-5 h-5 text-sky-500" />
            <div className="flex-1">
              <Label className="text-gray-100 text-base font-semibold">
                Email Notifications
              </Label>
              <p className="text-xs text-gray-400 mt-1">
                Receive notifications via email
              </p>
            </div>
            <Switch
              checked={settings.emailEnabled}
              onCheckedChange={(checked) => updateSetting('emailEnabled', checked)}
            />
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="emailTradeExecuted" className="text-gray-300">
                  Trade Executed
                </Label>
                <p className="text-xs text-gray-400">
                  When a trade is opened or closed
                </p>
              </div>
              <Switch
                id="emailTradeExecuted"
                checked={settings.emailTradeExecuted}
                onCheckedChange={(checked) => updateSetting('emailTradeExecuted', checked)}
                disabled={!settings.emailEnabled}
              />
            </div>

            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="emailProfitTarget" className="text-gray-300">
                  Profit Target Hit
                </Label>
                <p className="text-xs text-gray-400">
                  When take profit level is reached
                </p>
              </div>
              <Switch
                id="emailProfitTarget"
                checked={settings.emailProfitTarget}
                onCheckedChange={(checked) => updateSetting('emailProfitTarget', checked)}
                disabled={!settings.emailEnabled}
              />
            </div>

            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="emailStopLoss" className="text-gray-300">
                  Stop Loss Triggered
                </Label>
                <p className="text-xs text-gray-400">
                  When stop loss is hit
                </p>
              </div>
              <Switch
                id="emailStopLoss"
                checked={settings.emailStopLoss}
                onCheckedChange={(checked) => updateSetting('emailStopLoss', checked)}
                disabled={!settings.emailEnabled}
              />
            </div>

            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="emailDailyReport" className="text-gray-300">
                  Daily Summary
                </Label>
                <p className="text-xs text-gray-400">
                  Daily performance report (9:00 AM)
                </p>
              </div>
              <Switch
                id="emailDailyReport"
                checked={settings.emailDailyReport}
                onCheckedChange={(checked) => updateSetting('emailDailyReport', checked)}
                disabled={!settings.emailEnabled}
              />
            </div>

            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="emailWeeklyReport" className="text-gray-300">
                  Weekly Summary
                </Label>
                <p className="text-xs text-gray-400">
                  Weekly performance report (Monday 9:00 AM)
                </p>
              </div>
              <Switch
                id="emailWeeklyReport"
                checked={settings.emailWeeklyReport}
                onCheckedChange={(checked) => updateSetting('emailWeeklyReport', checked)}
                disabled={!settings.emailEnabled}
              />
            </div>
          </div>
        </div>
      </GlassCard>

      {/* Push Notifications */}
      <GlassCard>
        <div className="space-y-4">
          <div className="flex items-center gap-2 pb-2 border-b border-slate-700">
            <Bell className="w-5 h-5 text-sky-500" />
            <div className="flex-1">
              <Label className="text-gray-100 text-base font-semibold">
                Push Notifications
              </Label>
              <p className="text-xs text-gray-400 mt-1">
                Instant alerts on your device
              </p>
            </div>
            <Switch
              checked={settings.pushEnabled}
              onCheckedChange={(checked) => updateSetting('pushEnabled', checked)}
            />
          </div>

          <div className="space-y-4">
            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="pushTradeExecuted" className="text-gray-300">
                  Trade Executed
                </Label>
                <p className="text-xs text-gray-400">
                  Instant notification when trade happens
                </p>
              </div>
              <Switch
                id="pushTradeExecuted"
                checked={settings.pushTradeExecuted}
                onCheckedChange={(checked) => updateSetting('pushTradeExecuted', checked)}
                disabled={!settings.pushEnabled}
              />
            </div>

            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="pushProfitTarget" className="text-gray-300">
                  Profit Target Hit
                </Label>
                <p className="text-xs text-gray-400">
                  Celebrate your wins
                </p>
              </div>
              <Switch
                id="pushProfitTarget"
                checked={settings.pushProfitTarget}
                onCheckedChange={(checked) => updateSetting('pushProfitTarget', checked)}
                disabled={!settings.pushEnabled}
              />
            </div>

            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="pushStopLoss" className="text-gray-300">
                  Stop Loss Triggered
                </Label>
                <p className="text-xs text-gray-400">
                  Know when losses are cut
                </p>
              </div>
              <Switch
                id="pushStopLoss"
                checked={settings.pushStopLoss}
                onCheckedChange={(checked) => updateSetting('pushStopLoss', checked)}
                disabled={!settings.pushEnabled}
              />
            </div>

            <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity">
              <div className="flex-1">
                <Label htmlFor="pushPriceAlert" className="text-gray-300">
                  Price Alerts
                </Label>
                <p className="text-xs text-gray-400">
                  Custom price level notifications
                </p>
              </div>
              <Switch
                id="pushPriceAlert"
                checked={settings.pushPriceAlert}
                onCheckedChange={(checked) => updateSetting('pushPriceAlert', checked)}
                disabled={!settings.pushEnabled}
              />
            </div>
          </div>
        </div>
      </GlassCard>

      {/* Trading Alerts */}
      <GlassCard>
        <div className="space-y-4">
          <div className="flex items-center gap-2 pb-2 border-b border-slate-700">
            <TrendingUp className="w-5 h-5 text-sky-500" />
            <div className="flex-1">
              <Label className="text-gray-100 text-base font-semibold">
                Trading Alerts
              </Label>
              <p className="text-xs text-gray-400 mt-1">
                Threshold-based trading notifications
              </p>
            </div>
            <Switch
              checked={settings.alertsEnabled}
              onCheckedChange={(checked) => updateSetting('alertsEnabled', checked)}
            />
          </div>

          <div className="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div className="space-y-2">
              <Label htmlFor="alertProfit" className="text-gray-300">
                Profit Alert Threshold (%)
              </Label>
              <p className="text-xs text-gray-400">
                Alert when profit exceeds this percentage
              </p>
              <div className="relative">
                <Input
                  id="alertProfit"
                  type="number"
                  min={1}
                  max={100}
                  step={1}
                  value={settings.alertProfitThreshold}
                  onChange={(e) => updateSetting('alertProfitThreshold', Number(e.target.value))}
                  className="pr-8 bg-slate-800/50 border-slate-700 text-gray-100"
                  disabled={!settings.alertsEnabled}
                />
                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400">%</span>
              </div>
            </div>

            <div className="space-y-2">
              <Label htmlFor="alertLoss" className="text-gray-300">
                Loss Alert Threshold (%)
              </Label>
              <p className="text-xs text-gray-400">
                Alert when loss exceeds this percentage
              </p>
              <div className="relative">
                <Input
                  id="alertLoss"
                  type="number"
                  min={1}
                  max={100}
                  step={1}
                  value={settings.alertLossThreshold}
                  onChange={(e) => updateSetting('alertLossThreshold', Number(e.target.value))}
                  className="pr-8 bg-slate-800/50 border-slate-700 text-gray-100"
                  disabled={!settings.alertsEnabled}
                />
                <span className="absolute right-3 top-1/2 -translate-y-1/2 text-gray-400">%</span>
              </div>
            </div>
          </div>

          <div className="flex items-center justify-between opacity-70 hover:opacity-100 transition-opacity pt-2">
            <div className="flex-1">
              <Label htmlFor="alertSound" className="text-gray-300">
                Sound Notifications
              </Label>
              <p className="text-xs text-gray-400">
                Play sound when alerts trigger
              </p>
            </div>
            <Switch
              id="alertSound"
              checked={settings.alertSoundEnabled}
              onCheckedChange={(checked) => updateSetting('alertSoundEnabled', checked)}
              disabled={!settings.alertsEnabled}
            />
          </div>
        </div>
      </GlassCard>

      {/* In-App Notifications */}
      <GlassCard>
        <div className="space-y-4">
          <div className="flex items-center gap-2">
            <AlertCircle className="w-5 h-5 text-sky-500" />
            <div className="flex-1">
              <Label className="text-gray-100 text-base font-semibold">
                In-App Notifications
              </Label>
              <p className="text-xs text-gray-400 mt-1">
                Show notifications within the application
              </p>
            </div>
            <Switch
              checked={settings.inAppEnabled}
              onCheckedChange={(checked) => updateSetting('inAppEnabled', checked)}
            />
          </div>
        </div>
      </GlassCard>
    </div>
  );
}
