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
import type { RealTradingSettingsFlat, SyncPreview, SyncFieldChange } from '@/hooks/useRealTrading';
import {
  Zap,
  Shield,
  ChevronDown,
  ChevronRight,
  AlertTriangle,
  Settings2,
  TrendingUp,
  TrendingDown,
  RefreshCw,
  ArrowRight,
  Check,
  Info,
} from 'lucide-react';

interface AutoTradingPanelProps {
  settings: RealTradingSettingsFlat;
  onUpdateSettings: (partial: Partial<RealTradingSettingsFlat>) => Promise<void>;
  onSyncFromPaper?: () => Promise<SyncPreview | null>;
  onConfirmSync?: (changes: SyncFieldChange[]) => Promise<void>;
  isLoading: boolean;
}

const AVAILABLE_SYMBOLS = ['BTCUSDT', 'ETHUSDT', 'BNBUSDT', 'SOLUSDT', 'XRPUSDT', 'ADAUSDT'];

export function AutoTradingPanel({ settings, onUpdateSettings, onSyncFromPaper, onConfirmSync, isLoading }: AutoTradingPanelProps) {
  const colors = useThemeColors();
  const [showConfirmDialog, setShowConfirmDialog] = useState(false);
  const [showAdvanced, setShowAdvanced] = useState(false);
  const [showAdvancedRisk, setShowAdvancedRisk] = useState(false);
  const [showSyncDialog, setShowSyncDialog] = useState(false);
  const [syncPreview, setSyncPreview] = useState<SyncPreview | null>(null);
  const [isSyncing, setIsSyncing] = useState(false);
  const [showSkipped, setShowSkipped] = useState(false);

  const handleSyncFromPaper = useCallback(async () => {
    if (!onSyncFromPaper) return;
    setIsSyncing(true);
    const preview = await onSyncFromPaper();
    setIsSyncing(false);
    if (preview) {
      setSyncPreview(preview);
      setShowSyncDialog(true);
      setShowSkipped(false);
    }
  }, [onSyncFromPaper]);

  const handleConfirmSync = useCallback(async () => {
    if (!onConfirmSync || !syncPreview) return;
    setIsSyncing(true);
    await onConfirmSync(syncPreview.changes);
    setIsSyncing(false);
    setShowSyncDialog(false);
    setSyncPreview(null);
  }, [onConfirmSync, syncPreview]);

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

      {/* Sync from Paper Trading Dialog */}
      <AnimatePresence>
        {showSyncDialog && syncPreview && (
          <motion.div
            className="fixed inset-0 z-[100] flex items-center justify-center"
            initial={{ opacity: 0 }}
            animate={{ opacity: 1 }}
            exit={{ opacity: 0 }}
          >
            <div className="absolute inset-0 bg-black/60 backdrop-blur-sm" onClick={() => setShowSyncDialog(false)} />
            <motion.div
              className="relative z-10 max-w-md mx-4 rounded-2xl overflow-hidden max-h-[80vh] flex flex-col"
              style={{
                backgroundColor: colors.bgPrimary,
                border: '1px solid rgba(0, 217, 255, 0.3)',
                boxShadow: '0 0 40px rgba(0, 217, 255, 0.1)',
              }}
              initial={{ scale: 0.9, opacity: 0 }}
              animate={{ scale: 1, opacity: 1 }}
              exit={{ scale: 0.9, opacity: 0 }}
            >
              {/* Header */}
              <div className="p-4 border-b" style={{ borderColor: colors.borderSubtle }}>
                <div className="flex items-center gap-3">
                  <div
                    className="p-2 rounded-xl"
                    style={{ background: 'rgba(0, 217, 255, 0.15)', border: '1px solid rgba(0, 217, 255, 0.3)' }}
                  >
                    <RefreshCw className="w-4 h-4" style={{ color: colors.cyan }} />
                  </div>
                  <div>
                    <h3 className="text-sm font-bold" style={{ color: colors.textPrimary }}>
                      Sync from Paper Trading
                    </h3>
                    <p className="text-[10px] mt-0.5" style={{ color: colors.textMuted }}>
                      Copy proven settings to real trading
                    </p>
                  </div>
                </div>
              </div>

              {/* Body */}
              <div className="overflow-y-auto flex-1 p-4">
                {syncPreview.changes.length === 0 ? (
                  <div className="flex flex-col items-center gap-3 py-6">
                    <div
                      className="p-3 rounded-xl"
                      style={{ background: 'rgba(34, 197, 94, 0.15)', border: '1px solid rgba(34, 197, 94, 0.3)' }}
                    >
                      <Check className="w-5 h-5" style={{ color: '#22c55e' }} />
                    </div>
                    <p className="text-xs font-medium" style={{ color: colors.textSecondary }}>
                      All settings already match paper trading
                    </p>
                  </div>
                ) : (
                  <>
                    <p className="text-[10px] uppercase tracking-wider mb-2 font-bold" style={{ color: colors.textMuted }}>
                      {syncPreview.changes.length} field{syncPreview.changes.length > 1 ? 's' : ''} will change
                    </p>
                    <div className="space-y-1.5">
                      {syncPreview.changes.map((change) => (
                        <div
                          key={change.field}
                          className="flex items-center justify-between px-3 py-2 rounded-lg"
                          style={{ background: 'rgba(255, 255, 255, 0.03)', border: `1px solid ${colors.borderSubtle}` }}
                        >
                          <span className="text-[11px] font-medium" style={{ color: colors.textSecondary }}>
                            {change.label}
                          </span>
                          <div className="flex items-center gap-2">
                            <span className="text-[11px] font-mono" style={{ color: colors.textMuted }}>
                              {formatSyncValue(change.realValue)}
                            </span>
                            <ArrowRight className="w-3 h-3" style={{ color: colors.cyan }} />
                            <span className="text-[11px] font-mono font-bold" style={{ color: colors.cyan }}>
                              {formatSyncValue(change.paperValue)}
                            </span>
                          </div>
                        </div>
                      ))}
                    </div>
                  </>
                )}

                {/* Skipped fields */}
                {syncPreview.skipped.length > 0 && (
                  <div className="mt-4">
                    <button
                      onClick={() => setShowSkipped(!showSkipped)}
                      className="flex items-center gap-1.5 text-[10px] uppercase tracking-wider font-bold cursor-pointer"
                      style={{ color: colors.textMuted }}
                    >
                      {showSkipped ? (
                        <ChevronDown className="w-3 h-3" />
                      ) : (
                        <ChevronRight className="w-3 h-3" />
                      )}
                      <Info className="w-3 h-3" />
                      {syncPreview.skipped.length} fields not synced
                    </button>
                    <AnimatePresence>
                      {showSkipped && (
                        <motion.div
                          initial={{ height: 0, opacity: 0 }}
                          animate={{ height: 'auto', opacity: 1 }}
                          exit={{ height: 0, opacity: 0 }}
                          className="overflow-hidden"
                        >
                          <div className="mt-2 space-y-1">
                            {syncPreview.skipped.map((item) => (
                              <div
                                key={item.field}
                                className="px-3 py-1.5 rounded-lg"
                                style={{ background: 'rgba(255, 255, 255, 0.02)' }}
                              >
                                <span className="text-[10px] font-mono" style={{ color: colors.textMuted }}>
                                  {item.field}
                                </span>
                                <span className="text-[10px] ml-2" style={{ color: colors.textMuted }}>
                                  — {item.reason}
                                </span>
                              </div>
                            ))}
                          </div>
                        </motion.div>
                      )}
                    </AnimatePresence>
                  </div>
                )}
              </div>

              {/* Footer */}
              <div className="p-4 border-t flex gap-2" style={{ borderColor: colors.borderSubtle }}>
                <button
                  onClick={() => setShowSyncDialog(false)}
                  className="flex-1 px-4 py-2.5 text-xs font-bold rounded-xl transition-all"
                  style={{
                    background: 'rgba(255, 255, 255, 0.05)',
                    border: '1px solid rgba(255, 255, 255, 0.1)',
                    color: colors.textSecondary,
                  }}
                >
                  {syncPreview.changes.length === 0 ? 'Close' : 'Cancel'}
                </button>
                {syncPreview.changes.length > 0 && (
                  <button
                    onClick={handleConfirmSync}
                    disabled={isSyncing}
                    className="flex-1 px-4 py-2.5 text-xs font-bold rounded-xl transition-all"
                    style={{
                      background: 'linear-gradient(135deg, rgba(0, 217, 255, 0.7), rgba(0, 150, 255, 0.7))',
                      border: '1px solid rgba(0, 217, 255, 0.5)',
                      color: '#fff',
                      opacity: isSyncing ? 0.5 : 1,
                    }}
                  >
                    {isSyncing ? 'Applying...' : `Apply ${syncPreview.changes.length} Change${syncPreview.changes.length > 1 ? 's' : ''}`}
                  </button>
                )}
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

          {/* Sync from Paper Button */}
          {onSyncFromPaper && (
            <button
              onClick={handleSyncFromPaper}
              disabled={isSyncing || isLoading}
              className="mt-2 w-full flex items-center justify-center gap-1.5 px-3 py-1.5 text-[10px] font-bold rounded-lg transition-all cursor-pointer"
              style={{
                background: 'rgba(0, 217, 255, 0.08)',
                border: '1px solid rgba(0, 217, 255, 0.2)',
                color: colors.cyan,
                opacity: isSyncing || isLoading ? 0.5 : 1,
              }}
            >
              <RefreshCw className={`w-3 h-3 ${isSyncing ? 'animate-spin' : ''}`} />
              {isSyncing ? 'Loading...' : 'Sync from Paper Trading'}
            </button>
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

        {/* Section 4: Advanced Risk (collapsible) */}
        <div style={{ backgroundColor: colors.bgPrimary }}>
          <button
            onClick={() => setShowAdvancedRisk(!showAdvancedRisk)}
            className="w-full flex items-center justify-between p-3 cursor-pointer"
          >
            <div className="flex items-center gap-2">
              <Settings2 className="w-3.5 h-3.5" style={{ color: colors.textMuted }} />
              <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: colors.textMuted }}>
                Advanced Risk
              </span>
            </div>
            <ChevronDown
              className={`w-3.5 h-3.5 transition-transform duration-200 ${showAdvancedRisk ? 'rotate-180' : ''}`}
              style={{ color: colors.textMuted }}
            />
          </button>

          <AnimatePresence>
            {showAdvancedRisk && (
              <motion.div
                initial={{ height: 0, opacity: 0 }}
                animate={{ height: 'auto', opacity: 1 }}
                exit={{ height: 0, opacity: 0 }}
                className="overflow-hidden"
              >
                <div className="px-3 pb-3 space-y-3">

                  {/* ATR-Based SL/TP */}
                  <FeatureGroup
                    label="ATR-Based SL/TP"
                    enabled={settings.atr_stop_enabled}
                    onToggle={(v) => onUpdateSettings({ atr_stop_enabled: v })}
                    isLoading={isLoading}
                  >
                    <div className="grid grid-cols-2 gap-2">
                      <CompactInput label="ATR Period" value={settings.atr_period} min={2} max={100} onChange={(v) => onUpdateSettings({ atr_period: v })} disabled={isLoading} />
                      <CompactInput label="Base Risk %" value={settings.base_risk_pct} suffix="%" min={0.5} max={20} step={0.5} onChange={(v) => onUpdateSettings({ base_risk_pct: v })} disabled={isLoading} />
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                      <CompactInput label="SL Multiplier" value={settings.atr_stop_multiplier} min={0.5} max={5} step={0.1} onChange={(v) => onUpdateSettings({ atr_stop_multiplier: v })} disabled={isLoading} />
                      <CompactInput label="TP Multiplier" value={settings.atr_tp_multiplier} min={0.5} max={10} step={0.1} onChange={(v) => onUpdateSettings({ atr_tp_multiplier: v })} disabled={isLoading} />
                    </div>
                  </FeatureGroup>

                  {/* Kelly Criterion */}
                  <FeatureGroup
                    label="Kelly Criterion"
                    enabled={settings.kelly_enabled}
                    onToggle={(v) => onUpdateSettings({ kelly_enabled: v })}
                    isLoading={isLoading}
                  >
                    <div className="grid grid-cols-2 gap-2">
                      <CompactInput label="Min Trades" value={settings.kelly_min_trades} min={10} max={1000} onChange={(v) => onUpdateSettings({ kelly_min_trades: v })} disabled={isLoading} />
                      <CompactInput label="Fraction" value={settings.kelly_fraction} min={0.1} max={1.0} step={0.1} onChange={(v) => onUpdateSettings({ kelly_fraction: v })} disabled={isLoading} />
                    </div>
                    <CompactInput label="Lookback" value={settings.kelly_lookback} min={10} max={1000} onChange={(v) => onUpdateSettings({ kelly_lookback: v })} disabled={isLoading} />
                  </FeatureGroup>

                  {/* Regime Filters */}
                  <div className="space-y-2">
                    <span className="text-[10px] font-bold uppercase tracking-wider" style={{ color: colors.textMuted }}>Regime Filters</span>

                    <FeatureGroup
                      label="Funding Spike"
                      enabled={settings.funding_spike_filter_enabled}
                      onToggle={(v) => onUpdateSettings({ funding_spike_filter_enabled: v })}
                      isLoading={isLoading}
                    >
                      <div className="grid grid-cols-2 gap-2">
                        <CompactInput label="Threshold" value={settings.funding_spike_threshold} min={0.0001} max={0.01} step={0.0001} onChange={(v) => onUpdateSettings({ funding_spike_threshold: v })} disabled={isLoading} />
                        <CompactInput label="Reduction" value={settings.funding_spike_reduction} min={0.1} max={1.0} step={0.1} onChange={(v) => onUpdateSettings({ funding_spike_reduction: v })} disabled={isLoading} />
                      </div>
                    </FeatureGroup>

                    <FeatureGroup
                      label="ATR Spike"
                      enabled={settings.atr_spike_filter_enabled}
                      onToggle={(v) => onUpdateSettings({ atr_spike_filter_enabled: v })}
                      isLoading={isLoading}
                    >
                      <div className="grid grid-cols-2 gap-2">
                        <CompactInput label="Multiplier" value={settings.atr_spike_multiplier} min={1.1} max={5} step={0.1} onChange={(v) => onUpdateSettings({ atr_spike_multiplier: v })} disabled={isLoading} />
                        <CompactInput label="Reduction" value={settings.atr_spike_reduction} min={0.1} max={1.0} step={0.1} onChange={(v) => onUpdateSettings({ atr_spike_reduction: v })} disabled={isLoading} />
                      </div>
                    </FeatureGroup>

                    <FeatureGroup
                      label="Consec. Loss"
                      enabled={settings.consecutive_loss_reduction_enabled}
                      onToggle={(v) => onUpdateSettings({ consecutive_loss_reduction_enabled: v })}
                      isLoading={isLoading}
                    >
                      <div className="grid grid-cols-2 gap-2">
                        <CompactInput label="Threshold" value={settings.consecutive_loss_reduction_threshold} min={1} max={10} onChange={(v) => onUpdateSettings({ consecutive_loss_reduction_threshold: v })} disabled={isLoading} />
                        <CompactInput label="Reduction" value={settings.consecutive_loss_reduction_pct} min={0.1} max={1.0} step={0.1} onChange={(v) => onUpdateSettings({ consecutive_loss_reduction_pct: v })} disabled={isLoading} />
                      </div>
                    </FeatureGroup>
                  </div>

                  {/* Signal Reversal */}
                  <FeatureGroup
                    label="Signal Reversal"
                    enabled={settings.enable_signal_reversal}
                    onToggle={(v) => onUpdateSettings({ enable_signal_reversal: v })}
                    isLoading={isLoading}
                  >
                    <div className="grid grid-cols-2 gap-2">
                      <CompactInput label="Min Confidence" value={settings.reversal_min_confidence} min={0.5} max={1.0} step={0.05} onChange={(v) => onUpdateSettings({ reversal_min_confidence: v })} disabled={isLoading} />
                      <CompactInput label="Max PnL %" value={settings.reversal_max_pnl_pct} suffix="%" min={1} max={20} step={0.5} onChange={(v) => onUpdateSettings({ reversal_max_pnl_pct: v })} disabled={isLoading} />
                    </div>
                  </FeatureGroup>

                  {/* Trailing Stop */}
                  <FeatureGroup
                    label="Trailing Stop"
                    enabled={settings.enable_trailing_stop}
                    onToggle={(v) => onUpdateSettings({ enable_trailing_stop: v })}
                    isLoading={isLoading}
                  >
                    <div className="grid grid-cols-2 gap-2">
                      <CompactInput label="Activation %" value={settings.trailing_stop_activation_percent} suffix="%" min={0.5} max={10} step={0.5} onChange={(v) => onUpdateSettings({ trailing_stop_activation_percent: v })} disabled={isLoading} />
                      <CompactInput label="Trail %" value={settings.trailing_stop_percent} suffix="%" min={0.5} max={10} step={0.5} onChange={(v) => onUpdateSettings({ trailing_stop_percent: v })} disabled={isLoading} />
                    </div>
                  </FeatureGroup>

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
 * FeatureGroup - Toggle + expandable params for advanced features
 */
function FeatureGroup({
  label,
  enabled,
  onToggle,
  isLoading,
  children,
}: {
  label: string;
  enabled: boolean;
  onToggle: (enabled: boolean) => void;
  isLoading: boolean;
  children: React.ReactNode;
}) {
  const colors = useThemeColors();

  return (
    <div
      className="rounded-lg border p-2 space-y-2"
      style={{
        borderColor: enabled ? 'rgba(239, 68, 68, 0.3)' : colors.borderSubtle,
        backgroundColor: enabled ? 'rgba(239, 68, 68, 0.03)' : 'transparent',
      }}
    >
      <div className="flex items-center justify-between">
        <span className="text-[10px] font-medium" style={{ color: enabled ? colors.textPrimary : colors.textMuted }}>
          {label}
        </span>
        <button
          onClick={() => onToggle(!enabled)}
          disabled={isLoading}
          className={`relative w-7 h-4 rounded-full transition-colors ${enabled ? 'bg-red-500' : 'bg-white/10'}`}
        >
          <span
            className={`absolute top-0.5 left-0.5 w-3 h-3 rounded-full bg-white transition-transform ${enabled ? 'translate-x-3' : ''}`}
          />
        </button>
      </div>
      {enabled && <div className="space-y-2">{children}</div>}
    </div>
  );
}

function formatSyncValue(value: unknown): string {
  if (typeof value === 'boolean') return value ? 'Yes' : 'No';
  if (typeof value === 'number') {
    if (Number.isInteger(value)) return String(value);
    return value.toFixed(2);
  }
  return String(value);
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
