/**
 * AppearanceSettings Component
 *
 * Theme, accent colors, and layout preferences.
 * Changes apply immediately.
 */

import { useState, useEffect } from 'react';
import { GlassCard } from '@/components/ui/GlassCard';
import { Label } from '@/components/ui/label';
import { colors } from '@/styles/tokens/colors';
import { Sun, Moon, Monitor } from 'lucide-react';
import { cn } from '@/lib/utils';

interface AppearanceSettingsData {
  theme: 'light' | 'dark' | 'system';
  accentColor: string;
  density: 'compact' | 'comfortable';
  chartStyle: 'candlestick' | 'line' | 'area';
}

const DEFAULT_SETTINGS: AppearanceSettingsData = {
  theme: 'dark',
  accentColor: colors.paper.accent,
  density: 'comfortable',
  chartStyle: 'candlestick',
};

const ACCENT_COLORS = [
  { name: 'Sky Blue', value: '#0EA5E9' },
  { name: 'Emerald', value: '#10B981' },
  { name: 'Purple', value: '#8B5CF6' },
  { name: 'Rose', value: '#F43F5E' },
  { name: 'Amber', value: '#F59E0B' },
  { name: 'Cyan', value: '#06B6D4' },
];

export function AppearanceSettings() {
  const [settings, setSettings] = useState<AppearanceSettingsData>(DEFAULT_SETTINGS);

  useEffect(() => {
    const saved = localStorage.getItem('appearanceSettings');
    if (saved) {
      setSettings(JSON.parse(saved));
    }
  }, []);

  const updateSetting = <K extends keyof AppearanceSettingsData>(
    key: K,
    value: AppearanceSettingsData[K]
  ) => {
    const newSettings = { ...settings, [key]: value };
    setSettings(newSettings);
    localStorage.setItem('appearanceSettings', JSON.stringify(newSettings));

    // Apply theme immediately
    if (key === 'theme') {
      document.documentElement.classList.toggle('dark', value === 'dark');
    }
  };

  return (
    <div className="space-y-6">
      <div>
        <h2 className="text-2xl font-bold text-gray-100">Appearance</h2>
        <p className="text-sm text-gray-400 mt-1">
          Customize the look and feel of your dashboard
        </p>
      </div>

      {/* Theme Selection */}
      <GlassCard>
        <div className="space-y-4">
          <Label className="text-gray-100 text-base font-semibold">Theme</Label>
          <div className="grid grid-cols-3 gap-3">
            {[
              { value: 'light', icon: Sun, label: 'Light' },
              { value: 'dark', icon: Moon, label: 'Dark' },
              { value: 'system', icon: Monitor, label: 'System' },
            ].map((theme) => {
              const Icon = theme.icon;
              return (
                <button
                  key={theme.value}
                  onClick={() => updateSetting('theme', theme.value as any)}
                  className={cn(
                    'flex flex-col items-center gap-2 p-4 rounded-lg border-2 transition-all',
                    settings.theme === theme.value
                      ? 'border-sky-500 bg-sky-500/10'
                      : 'border-slate-700 hover:border-slate-600 bg-slate-800/50'
                  )}
                >
                  <Icon className="w-6 h-6 text-gray-300" />
                  <span className="text-sm text-gray-300">{theme.label}</span>
                </button>
              );
            })}
          </div>
        </div>
      </GlassCard>

      {/* Accent Color */}
      <GlassCard>
        <div className="space-y-4">
          <Label className="text-gray-100 text-base font-semibold">Accent Color</Label>
          <div className="grid grid-cols-3 md:grid-cols-6 gap-3">
            {ACCENT_COLORS.map((color) => (
              <button
                key={color.value}
                onClick={() => updateSetting('accentColor', color.value)}
                className={cn(
                  'flex flex-col items-center gap-2 p-3 rounded-lg border-2 transition-all',
                  settings.accentColor === color.value
                    ? 'border-current'
                    : 'border-slate-700 hover:border-slate-600'
                )}
                style={{ color: color.value }}
              >
                <div
                  className="w-10 h-10 rounded-full"
                  style={{ backgroundColor: color.value }}
                />
                <span className="text-xs text-gray-300">{color.name}</span>
              </button>
            ))}
          </div>
        </div>
      </GlassCard>

      {/* Density */}
      <GlassCard>
        <div className="space-y-4">
          <Label className="text-gray-100 text-base font-semibold">Display Density</Label>
          <div className="grid grid-cols-2 gap-3">
            {[
              { value: 'compact', label: 'Compact', desc: 'More content per screen' },
              { value: 'comfortable', label: 'Comfortable', desc: 'More spacing' },
            ].map((density) => (
              <button
                key={density.value}
                onClick={() => updateSetting('density', density.value as any)}
                className={cn(
                  'flex flex-col items-start gap-1 p-4 rounded-lg border-2 transition-all text-left',
                  settings.density === density.value
                    ? 'border-sky-500 bg-sky-500/10'
                    : 'border-slate-700 hover:border-slate-600 bg-slate-800/50'
                )}
              >
                <span className="font-medium text-gray-100">{density.label}</span>
                <span className="text-xs text-gray-400">{density.desc}</span>
              </button>
            ))}
          </div>
        </div>
      </GlassCard>

      {/* Chart Style */}
      <GlassCard>
        <div className="space-y-4">
          <Label className="text-gray-100 text-base font-semibold">Default Chart Style</Label>
          <div className="grid grid-cols-3 gap-3">
            {[
              { value: 'candlestick', label: 'Candlestick' },
              { value: 'line', label: 'Line' },
              { value: 'area', label: 'Area' },
            ].map((style) => (
              <button
                key={style.value}
                onClick={() => updateSetting('chartStyle', style.value as any)}
                className={cn(
                  'p-4 rounded-lg border-2 transition-all',
                  settings.chartStyle === style.value
                    ? 'border-sky-500 bg-sky-500/10'
                    : 'border-slate-700 hover:border-slate-600 bg-slate-800/50'
                )}
              >
                <span className="text-sm text-gray-300">{style.label}</span>
              </button>
            ))}
          </div>
        </div>
      </GlassCard>
    </div>
  );
}
