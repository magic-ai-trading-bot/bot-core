/**
 * AutoTradingPanel - Auto-Trading Settings Panel for Real Trading
 *
 * 3 sections: Toggle, Trading Config, Risk Config (collapsible).
 * Uses GlassCard-style consistent with RealTrading page.
 *
 * @spec:FR-TRADING-016 - Real Trading Auto-Trading UI
 */

import { useState, useCallback } from 'react';
import { motion, AnimatePresence } from 'framer-motion';
import { useThemeColors } from '@/hooks/useThemeColors';
import type { RealTradingSettingsFlat } from '@/hooks/useRealTrading';
import {
  Zap,
  Shield,
  ChevronDown,
  AlertTriangle,
  Settings2,
  TrendingUp,
  TrendingDown,
} from 'lucide-react';

interface AutoTradingPanelProps {
  settings: RealTradingSettingsFlat;
  onUpdateSettings: (partial: Partial<RealTradingSettingsFlat>) => Promise<void>;
  isLoading: boolean;
}

const AVAILABLE_SYMBOLS = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT', 'XRPUSDT', 'ADAUSDT'];

export function AutoTradingPanel({ settings, onUpdateSettings, isLoading }: AutoTradingPanelProps) {
  const colors = useThemeColors();
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);

  const handleToggleAutoTrading = useCallback(() => {
    if (settings.auto_trading_enabled) {
      // Turning off — no confirmation needed
      onUpdateSettings({ auto_trading_enabled: false });
    } else {
      // Turning on — show confirmation
      setShowConfirmDialog(true);
    }
  }, [settings.auto_trading_enabled, onUpdateSettings]);

  const confirmEnableAutoTrading = useCallback(() => {
    onUpdateSettings({ auto_trading_enabled: true });
    setShowConfirmDialog(false);
  }, [onUpdateSettings]);

  const handleSymbolToggle = useCallback(
    (symbol: string) => {
      const current = settings.auto_trade_symbols || [];
      const updated = current.includes(symbol)
        ? current.filter((s) => s !== symbol)
        : [...current, symbol];
      onUpdateSettings({ auto_trade_symbols: updated });
    },
    [settings.auto_trade_symbols, onUpdateSettings]
  );

  const handleDirectionMode = useCallback(
    (mode: 'both' | 'long' | 'short') => {
      if (mode === 'long') {
        onUpdateSettings({ long_only_mode: true, short_only_mode: false });
      } else if (mode === 'short') {
        onUpdateSettings({ long_only_mode: false, short_only_mode: true });
      } else {
        onUpdateSettings({ long_only_mode: false, short_only_mode: false });
      }
    },
    [onUpdateSettings]
  );

  const currentDirection = settings.long_only_mode
    ? 'long'
    : settings.short_only_mode
      ? 'short'
      : 'both';

  return (
    <>
      {/* Confirmation Dialog */}
      <AnimatePresence>
        {showConfirmDialog && (
          <motion.div
            className="fixed inset-0 z-[100] flex items-center justify-center"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={() => setShowConfirmDialog(false)} />
            <motion.div
              className="relative z-10 max-w-sm mx-4 rounded-2xl overflow-hidden"
              style={{
                backgroundColor: colors.bgPrimary,
                border: '1px solid rgba(239, 68, 68, 0.3)',
                boxShadow: '0 0 40px rgba(239, 68, 68, 0.15)',
              }}
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
            >
              <div className="p-5">
                <div className="flex items-center gap-3 mb-4">
                  <div
                    className="p-2.5 rounded-xl"
                    style={{ background: 'rgba(239, 68, 68, 0.15)', border: '1px solid rgba(239, 68, 68, 0.3)' }}
                  >
                    <AlertTriangle className="w-5 h-5" style={{ color: colors.loss }} />
                  </div>
                  <div>
                    <h3 className="text-sm font-bold" style={{ color: colors.textPrimary }}>
                      Enable Auto-Trading?
                    </h3>
                    <p className="text-[10px] mt-0.5" style={{ color: colors.textMuted }}>
                      This action uses REAL money
                    </p>
                  </div>
                </div>
                <p className="text-xs leading-relaxed mb-4" style={{ color: colors.textSecondary }}>
                  The system will automatically place REAL orders based on strategy signals.
                  Make sure your risk parameters are properly configured.
                </p>
                <div className="flex gap-2">
                  <button
                    onClick={() => setShowConfirmDialog(false)}
                    className="flex-1 px-4 py-2.5 text-xs font-bold rounded-xl transition-all"
                    style={{
                      background: 'rgba(255, 255, 255, 0.05)',
                      border: '1px solid rgba(255, 255, 255, 0.1)',
                      color: colors.textSecondary,
                    }}
                  >
                    Cancel
                  </button>
                  <button
                    onClick={confirmEnableAutoTrading}
                    disabled={isLoading}
                    className="flex-1 px-4 py-2.5 text-xs font-bold rounded-xl transition-all"
                    style={{
                      background: 'linear-gradient(135deg, rgba(239, 68, 68, 0.8), rgba(220, 38, 38, 0.8))',
                      border: '1px solid rgba(239, 68, 68, 0.5)',
                      color: '#fff',
                      opacity: isLoading ? 0.5 : 1,
                    }}
                  >
                    {isLoading ? 'Enabling...' : 'Enable Auto-Trading'}
                  </button>
                </div>
              </div>
            </motion.div>
          </motion.div>
        )}
      </AnimatePresence>

      <div className="flex flex-col gap-[1px]" style={{ backgroundColor: colors.borderSubtle }}>
        {/* Section 1: Auto-Trading Toggle */}
        <div className="p-3" style={{ backgroundColor: colors.bgPrimary }}>
          <div className="flex items-center justify-between mb-2">
            <div className="flex items-center gap-2">
              <div
                className="p-1.5 rounded-lg"
                style={{
                  background: settings.auto_trading_enabled
                    ? 'rgba(34, 197, 94, 0.15)'
                    : 'rgba(239, 68, 68, 0.1)',
                  border: `1px solid ${settings.auto_trading_enabled ? 'rgba(34, 197, 94, 0.3)' : 'rgba(239, 68, 68, 0.2)'}`,
                }}
              >
                <Zap
                  className="w-3.5 h-3.5"
                  style={{ color: settings.auto_trading_enabled ? '#22c55e' : colors.loss }}
                />
              </div>
              <span className="text-xs font-bold" style={{ color: colors.textPrimary }}>
                Auto-Trading
              </span>
              <span
                className="px-2 py-0.5 rounded-md text-[9px] font-bold uppercase tracking-wider"
                style={{
                  background: settings.auto_trading_enabled
                    ? 'rgba(34, 197, 94, 0.15)'
                    : 'rgba(239, 68, 68, 0.15)',
                  color: settings.auto_trading_enabled ? '#22c55e' : colors.loss,
                  border: `1px solid ${settings.auto_trading_enabled ? 'rgba(34, 197, 94, 0.3)' : 'rgba(239, 68, 68, 0.3)'}`,
                }}
              >
                {settings.auto_trading_enabled ? 'ON' : 'OFF'}
              </span>
            </div>

            {/* Toggle Switch */}
            <button
              onClick={handleToggleAutoTrading}
              disabled={isLoading}
              className="relative w-10 h-5 rounded-full transition-all duration-300 cursor-pointer"
              style={{
                background: settings.auto_trading_enabled
                  ? 'linear-gradient(135deg, #22c55e, #16a34a)'
                  : 'rgba(255, 255, 255, 0.1)',
                border: `1px solid ${settings.auto_trading_enabled ? 'rgba(34, 197, 94, 0.5)' : 'rgba(255, 255, 255, 0.15)'}`,
                opacity: isLoading ? 0.5 : 1,
              }}
            >
              <motion.div
                className="absolute top-0.5 w-4 h-4 rounded-full bg-white shadow-md"
                animate={{ left: settings.auto_trading_enabled ? 21 : 1 }}
                transition={{ type: 'spring', stiffness: 300, damping: 20 }}
              />
            </button>
          </div>

          {settings.auto_trading_enabled && (
            <motion.p
              initial={{ opacity: 0, height: 0 }}
              animate={{ opacity: 1, height: 'auto' }}
              className="text-[10px] leading-relaxed px-1"
              style={{ color: colors.warning }}
            >
              System will auto-place REAL orders from strategy signals
            </motion.p>
          )}
        </div>

        {/* Section 2: Trading Config */}
        <div className="p-3" style={{ backgroundColor: colors.bgPrimary }}>
          <div className="flex items-center gap-2 mb-3">
            <Settings2 className="w-3.5 h-3.5" style={{ color: colors.textMuted }} />
            <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: colors.textMuted }}>
              Trading Config
            </span>
          </div>

          {/* Symbols */}
          <div className="mb-3">
            <label className="block text-[10px] uppercase tracking-wider mb-1.5" style={{ color: colors.textMuted }}>
              Symbols
            </label>
            <div className="flex flex-wrap gap-1">
              {AVAILABLE_SYMBOLS.map((symbol) => {
                const isActive = (settings.auto_trade_symbols || []).includes(symbol);
                return (
                  <button
                    key={symbol}
                    onClick={() => handleSymbolToggle(symbol)}
                    disabled={isLoading}
                    className="px-2 py-1 text-[10px] font-bold rounded-lg transition-all cursor-pointer"
                    style={{
                      background: isActive ? 'rgba(239, 68, 68, 0.15)' : 'rgba(255, 255, 255, 0.03)',
                      border: `1px solid ${isActive ? 'rgba(239, 68, 68, 0.3)' : 'rgba(255, 255, 255, 0.08)'}`,
                      color: isActive ? colors.loss : colors.textMuted,
                    }}
                  >
                    {symbol.replace('USDT', '')}
                  </button>
                );
              })}
            </div>
          </div>

          {/* Max Leverage + Position Size */}
          <div className="grid grid-cols-2 gap-2 mb-3">
            <CompactInput
              label="Max Leverage"
              value={settings.max_leverage}
              suffix="x"
              min={1}
              max={20}
              onChange={(v) => onUpdateSettings({ max_leverage: v })}
              disabled={isLoading}
            />
            <CompactInput
              label="Position Size"
              value={settings.max_position_size_usdt}
              suffix="USDT"
              min={10}
              max={10000}
              onChange={(v) => onUpdateSettings({ max_position_size_usdt: v })}
              disabled={isLoading}
            />
          </div>

          {/* SL/TP */}
          <div className="grid grid-cols-2 gap-2 mb-3">
            <CompactInput
              label="Stop Loss"
              value={settings.default_stop_loss_percent}
              suffix="%"
              min={0.5}
              max={10}
              step={0.1}
              onChange={(v) => onUpdateSettings({ default_stop_loss_percent: v })}
              disabled={isLoading}
            />
            <CompactInput
              label="Take Profit"
              value={settings.default_take_profit_percent}
              suffix="%"
              min={1}
              max={20}
              step={0.1}
              onChange={(v) => onUpdateSettings({ default_take_profit_percent: v })}
              disabled={isLoading}
            />
          </div>

          {/* Direction Mode */}
          <div>
            <label className="block text-[10px] uppercase tracking-wider mb-1.5" style={{ color: colors.textMuted }}>
              Direction
            </label>
            <div className="flex gap-1">
              {([
                { id: 'both', label: 'Both', icon: null },
                { id: 'long', label: 'Long', icon: TrendingUp },
                { id: 'short', label: 'Short', icon: TrendingDown },
              ] as const).map(({ id, label, icon: Icon }) => (
                <button
                  key={id}
                  onClick={() => handleDirectionMode(id)}
                  disabled={isLoading}
                  className="flex-1 flex items-center justify-center gap-1 px-2 py-1.5 text-[10px] font-bold rounded-lg transition-all cursor-pointer"
                  style={{
                    background: currentDirection === id
                      ? id === 'long'
                        ? 'rgba(34, 197, 94, 0.15)'
                        : id === 'short'
                          ? 'rgba(239, 68, 68, 0.15)'
                          : 'rgba(0, 217, 255, 0.15)'
                      : 'rgba(255, 255, 255, 0.03)',
                    border: `1px solid ${
                      currentDirection === id
                        ? id === 'long'
                          ? 'rgba(34, 197, 94, 0.3)'
                          : id === 'short'
                            ? 'rgba(239, 68, 68, 0.3)'
                            : 'rgba(0, 217, 255, 0.3)'
                        : 'rgba(255, 255, 255, 0.08)'
                    }`,
                    color: currentDirection === id
                      ? id === 'long'
                        ? '#22c55e'
                        : id === 'short'
                          ? colors.loss
                          : colors.cyan
                      : colors.textMuted,
                  }}
                >
                  {Icon && <Icon className="w-3 h-3" />}
                  {label}
                </button>
              ))}
            </div>
          </div>
        </div>

        {/* Section 3: Risk Config (collapsible) */}
        <div style={{ backgroundColor: colors.bgPrimary }}>
          <button
            onClick={() => setShowAdvanced(!showAdvanced)}
            className="w-full flex items-center justify-between p-3 cursor-pointer"
          >
            <div className="flex items-center gap-2">
              <Shield className="w-3.5 h-3.5" style={{ color: colors.textMuted }} />
              <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: colors.textMuted }}>
                Risk Config
              </span>
            </div>
            <ChevronDown
              className={`w-3.5 h-3.5 transition-transform duration-200 ${showAdvanced ? 'rotate-180' : ''}`}
              style={{ color: colors.textMuted }}
            />
          </button>

          <AnimatePresence>
            {showAdvanced && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 'auto', opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                className="overflow-hidden"
              >
                <div className="px-3 pb-3 space-y-2">
                  {/* Min Signal Confidence */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label className="text-[10px] uppercase tracking-wider" style={{ color: colors.textMuted }}>
                        Min Confidence
                      </label>
                      <span className="text-[10px] font-mono font-bold" style={{ color: colors.textSecondary }}>
                        {(settings.min_signal_confidence * 100).toFixed(0)}%
                      </span>
                    </div>
                    <input
                      type="range"
                      min="50"
                      max="95"
                      value={settings.min_signal_confidence * 100}
                      onChange={(e) =>
                        onUpdateSettings({ min_signal_confidence: parseInt(e.target.value) / 100 })
                      }
                      disabled={isLoading}
                      className="w-full h-1 rounded-full appearance-none cursor-pointer accent-red-500"
                      style={{ background: `linear-gradient(to right, ${colors.loss} ${((settings.min_signal_confidence * 100 - 50) / 45) * 100}%, rgba(255,255,255,0.1) 0%)` }}
                    />
                  </div>

                  {/* Max Consecutive Losses + Cool-Down */}
                  <div className="grid grid-cols-2 gap-2">
                    <CompactInput
                      label="Max Consec. Losses"
                      value={settings.max_consecutive_losses}
                      min={1}
                      max={10}
                      onChange={(v) => onUpdateSettings({ max_consecutive_losses: v })}
                      disabled={isLoading}
                    />
                    <CompactInput
                      label="Cool-Down"
                      value={settings.cool_down_minutes}
                      suffix="min"
                      min={15}
                      max={240}
                      onChange={(v) => onUpdateSettings({ cool_down_minutes: v })}
                      disabled={isLoading}
                    />
                  </div>

                  {/* Correlation Limit */}
                  <div>
                    <div className="flex items-center justify-between mb-1">
                      <label className="text-[10px] uppercase tracking-wider" style={{ color: colors.textMuted }}>
                        Correlation Limit
                      </label>
                      <span className="text-[10px] font-mono font-bold" style={{ color: colors.textSecondary }}>
                        {(settings.correlation_limit * 100).toFixed(0)}%
                      </span>
                    </div>
                    <input
                      type="range"
                      min="30"
                      max="100"
                      value={settings.correlation_limit * 100}
                      onChange={(e) =>
                        onUpdateSettings({ correlation_limit: parseInt(e.target.value) / 100 })
                      }
                      disabled={isLoading}
                      className="w-full h-1 rounded-full appearance-none cursor-pointer accent-red-500"
                      style={{ background: `linear-gradient(to right, ${colors.loss} ${((settings.correlation_limit * 100 - 30) / 70) * 100}%, rgba(255,255,255,0.1) 0%)` }}
                    />
                  </div>

                  {/* Max Portfolio Risk */}
                  <CompactInput
                    label="Max Portfolio Risk"
                    value={settings.max_portfolio_risk_pct}
                    suffix="%"
                    min={5}
                    max={50}
                    onChange={(v) => onUpdateSettings({ max_portfolio_risk_pct: v })}
                    disabled={isLoading}
                  />
                </div>
              </motion.div>
            )}
          </AnimatePresence>
        </div>
      </div>
    </>
  );
}

/**
 * CompactInput - Small number input for settings panel
 */
function CompactInput({
  label,
  value,
  suffix,
  min,
  max,
  step = 1,
  onChange,
  disabled,
}: {
  label: string;
  value: number;
  suffix?: string;
  min?: number;
  max?: number;
  step?: number;
  onChange: (value: number) => void;
  disabled?: boolean;
}) {
  const colors = useThemeColors();

  return (
    <div>
      <label className="block text-[10px] uppercase tracking-wider mb-1" style={{ color: colors.textMuted }}>
        {label}
      </label>
      <div
        className="flex items-center rounded-lg border transition-all focus-within:border-red-500/40"
        style={{
          backgroundColor: 'rgba(255, 255, 255, 0.03)',
          borderColor: colors.borderSubtle,
        }}
      >
        <input
          type="number"
          value={value}
          min={min}
          max={max}
          step={step}
          onChange={(e) => {
            const v = parseFloat(e.target.value);
            if (!isNaN(v)) onChange(v);
          }}
          disabled={disabled}
          className="w-full px-2 py-1.5 text-[11px] font-mono bg-transparent outline-none text-white placeholder:text-white/30"
        />
        {suffix && (
          <span className="pr-2 text-[10px] font-medium whitespace-nowrap" style={{ color: colors.textMuted }}>
            {suffix}
          </span>
        )}
      </div>
    </div>
  );
}
