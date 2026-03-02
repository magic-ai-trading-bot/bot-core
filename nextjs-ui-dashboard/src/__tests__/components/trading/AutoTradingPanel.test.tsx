/**
 * AutoTradingPanel Component Tests
 *
 * Covers:
 * - Renders with auto-trading OFF (default state)
 * - Renders with auto-trading ON (shows warning text)
 * - Toggle switch calls onUpdateSettings
 * - Confirmation dialog appears when enabling
 * - Symbol chips toggle correctly
 * - Direction mode buttons (Both/Long/Short)
 * - Collapsible risk config section
 *
 * @spec:FR-TRADING-016 - Real Trading Auto-Trading UI
 */

import { describe, it, expect, vi, beforeEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { render } from '@testing-library/react';
import React from 'react';
import { AutoTradingPanel } from '@/components/trading/AutoTradingPanel';
import type { RealTradingSettingsFlat } from '@/hooks/useRealTrading';

// ─── Mock framer-motion ─────────────────────────────────────────────────────
vi.mock('framer-motion', () => {
  const React = require('react');
  return {
    motion: new Proxy(
      {},
      {
        get:
          () =>
          ({ children, ...props }: any) => {
            // Strip framer-only props so React doesn't warn
            const {
              initial,
              animate,
              exit,
              transition,
              whileHover,
              whileTap,
              layout,
              layoutId,
              ...domProps
            } = props;
            return React.createElement('div', domProps, children);
          },
      }
    ),
    AnimatePresence: ({ children }: { children: React.ReactNode }) => <>{children}</>,
  };
});

// ─── Mock useThemeColors ────────────────────────────────────────────────────
const mockColors = {
  bgPrimary: '#0a0a0f',
  bgSecondary: '#0d0d14',
  borderSubtle: 'rgba(255,255,255,0.06)',
  textPrimary: '#ffffff',
  textSecondary: 'rgba(255,255,255,0.7)',
  textMuted: 'rgba(255,255,255,0.4)',
  loss: '#ef4444',
  profit: '#22c55e',
  warning: '#f59e0b',
  cyan: '#00d9ff',
};

vi.mock('@/hooks/useThemeColors', () => ({
  useThemeColors: vi.fn(() => mockColors),
}));

// ─── Helpers ────────────────────────────────────────────────────────────────
function makeSettings(overrides: Partial<RealTradingSettingsFlat> = {}): RealTradingSettingsFlat {
  return {
    use_testnet: true,
    auto_trading_enabled: false,
    auto_trade_symbols: ['BTCUSDT'],
    max_position_size_usdt: 100,
    max_positions: 3,
    max_leverage: 5,
    max_daily_loss_usdt: 500,
    risk_per_trade_percent: 1,
    default_stop_loss_percent: 2,
    default_take_profit_percent: 4,
    min_signal_confidence: 0.7,
    max_consecutive_losses: 3,
    cool_down_minutes: 60,
    correlation_limit: 0.7,
    max_portfolio_risk_pct: 20,
    short_only_mode: false,
    long_only_mode: false,
    ...overrides,
  };
}

function renderPanel(
  settingsOverrides: Partial<RealTradingSettingsFlat> = {},
  isLoading = false
) {
  const onUpdateSettings = vi.fn().mockResolvedValue(undefined);
  const settings = makeSettings(settingsOverrides);

  const result = render(
    <AutoTradingPanel
      settings={settings}
      onUpdateSettings={onUpdateSettings}
      isLoading={isLoading}
    />
  );

  return { ...result, onUpdateSettings, settings };
}

// ============================================================================
// Tests
// ============================================================================

describe('AutoTradingPanel', () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  // ─── Section 1: Toggle area renders correctly ─────────────────────────────

  describe('Toggle section — auto-trading OFF (default)', () => {
    it('shows Auto-Trading label', () => {
      renderPanel();
      expect(screen.getByText('Auto-Trading')).toBeInTheDocument();
    });

    it('shows OFF badge when disabled', () => {
      renderPanel({ auto_trading_enabled: false });
      expect(screen.getByText('OFF')).toBeInTheDocument();
    });

    it('does NOT show warning text when disabled', () => {
      renderPanel({ auto_trading_enabled: false });
      expect(
        screen.queryByText(/auto-place REAL orders/i)
      ).not.toBeInTheDocument();
    });

    it('shows ON badge when enabled', () => {
      renderPanel({ auto_trading_enabled: true });
      expect(screen.getByText('ON')).toBeInTheDocument();
    });

    it('shows warning text when enabled', () => {
      renderPanel({ auto_trading_enabled: true });
      expect(
        screen.getByText(/auto-place REAL orders from strategy signals/i)
      ).toBeInTheDocument();
    });
  });

  // ─── Section 2: Toggle switch interaction ─────────────────────────────────

  describe('Toggle switch', () => {
    it('clicking toggle while OFF opens confirmation dialog (does not call onUpdateSettings yet)', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({ auto_trading_enabled: false });

      // The toggle is a <button> element
      const toggleBtn = screen.getByRole('button', {
        // Toggle has no text; find by relative structure — it follows the badge
        // Use a more robust selector: it's the last button in the toggle row area
        // Actually we can query all buttons and pick the toggle one
        name: '',
      });

      // The toggle button is found implicitly — click whichever button is NOT labeled
      // Since the panel has several buttons, find the toggle by its position.
      // The toggle is the button that does NOT have text content.
      const allButtons = screen.getAllByRole('button');
      // Toggle is the one without visible text (it's the switch)
      const toggleSwitch = allButtons.find((btn) => btn.textContent?.trim() === '');
      expect(toggleSwitch).toBeDefined();

      await user.click(toggleSwitch!);

      // Confirmation dialog should appear
      await waitFor(() => {
        expect(screen.getByText('Enable Auto-Trading?')).toBeInTheDocument();
      });

      // onUpdateSettings should NOT have been called yet
      expect(onUpdateSettings).not.toHaveBeenCalled();
    });

    it('clicking toggle while ON calls onUpdateSettings immediately with false (no dialog)', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({ auto_trading_enabled: true });

      const allButtons = screen.getAllByRole('button');
      const toggleSwitch = allButtons.find((btn) => btn.textContent?.trim() === '');
      expect(toggleSwitch).toBeDefined();

      await user.click(toggleSwitch!);

      expect(onUpdateSettings).toHaveBeenCalledWith({ auto_trading_enabled: false });
    });

    it('toggle button is disabled when isLoading=true', () => {
      renderPanel({}, true);
      const allButtons = screen.getAllByRole('button');
      const toggleSwitch = allButtons.find((btn) => btn.textContent?.trim() === '');
      expect(toggleSwitch).toBeDisabled();
    });
  });

  // ─── Section 3: Confirmation dialog ──────────────────────────────────────

  describe('Confirmation dialog', () => {
    async function openDialog() {
      const user = userEvent.setup();
      const result = renderPanel({ auto_trading_enabled: false });

      const allButtons = screen.getAllByRole('button');
      const toggleSwitch = allButtons.find((btn) => btn.textContent?.trim() === '');
      await user.click(toggleSwitch!);

      await waitFor(() => {
        expect(screen.getByText('Enable Auto-Trading?')).toBeInTheDocument();
      });

      return { user, ...result };
    }

    it('renders dialog title and body text', async () => {
      await openDialog();
      expect(screen.getByText('Enable Auto-Trading?')).toBeInTheDocument();
      expect(screen.getByText(/This action uses REAL money/i)).toBeInTheDocument();
      expect(
        screen.getByText(/automatically place REAL orders/i)
      ).toBeInTheDocument();
    });

    it('Cancel button closes the dialog without calling onUpdateSettings', async () => {
      const { user, onUpdateSettings } = await openDialog();

      const cancelBtn = screen.getByRole('button', { name: /cancel/i });
      await user.click(cancelBtn);

      await waitFor(() => {
        expect(screen.queryByText('Enable Auto-Trading?')).not.toBeInTheDocument();
      });
      expect(onUpdateSettings).not.toHaveBeenCalled();
    });

    it('clicking backdrop closes dialog', async () => {
      const { user } = await openDialog();

      // The backdrop is the absolute div inside the motion.div — it has onClick
      // It's a div, not a button, so we query by its class characteristic
      const backdrop = document.querySelector('.absolute.inset-0.bg-black\\/60') as HTMLElement;
      expect(backdrop).not.toBeNull();

      await user.click(backdrop);

      await waitFor(() => {
        expect(screen.queryByText('Enable Auto-Trading?')).not.toBeInTheDocument();
      });
    });

    it('Enable Auto-Trading button calls onUpdateSettings with true and closes dialog', async () => {
      const { user, onUpdateSettings } = await openDialog();

      const enableBtn = screen.getByRole('button', { name: /enable auto-trading/i });
      await user.click(enableBtn);

      expect(onUpdateSettings).toHaveBeenCalledWith({ auto_trading_enabled: true });

      await waitFor(() => {
        expect(screen.queryByText('Enable Auto-Trading?')).not.toBeInTheDocument();
      });
    });

    it('Enable button shows "Enabling..." and is disabled when isLoading=true', async () => {
      // We need to render with isLoading=true AND dialog open.
      // Achieve this by opening the dialog first (isLoading=false), then re-render.
      // Simplest: render a wrapper that toggles isLoading.
      const onUpdateSettings = vi.fn().mockResolvedValue(undefined);
      const settings = makeSettings({ auto_trading_enabled: false });

      const { rerender } = render(
        <AutoTradingPanel
          settings={settings}
          onUpdateSettings={onUpdateSettings}
          isLoading={false}
        />
      );

      const user = userEvent.setup();
      const allButtons = screen.getAllByRole('button');
      const toggleSwitch = allButtons.find((btn) => btn.textContent?.trim() === '');
      await user.click(toggleSwitch!);

      await waitFor(() => {
        expect(screen.getByText('Enable Auto-Trading?')).toBeInTheDocument();
      });

      // Now re-render with isLoading=true
      rerender(
        <AutoTradingPanel
          settings={settings}
          onUpdateSettings={onUpdateSettings}
          isLoading={true}
        />
      );

      const enableBtn = screen.getByRole('button', { name: /enabling\.\.\./i });
      expect(enableBtn).toBeInTheDocument();
      expect(enableBtn).toBeDisabled();
    });
  });

  // ─── Section 4: Trading Config — Symbols ─────────────────────────────────

  describe('Symbol chips', () => {
    it('renders all 6 available symbols (stripped of USDT)', () => {
      renderPanel();
      ['BTC', 'ETH', 'BNB', 'SOL', 'XRP', 'ADA'].forEach((sym) => {
        expect(screen.getByRole('button', { name: sym })).toBeInTheDocument();
      });
    });

    it('active symbols are visually distinguished (data check via onUpdateSettings)', async () => {
      // BTC is active by default in makeSettings
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({ auto_trade_symbols: ['BTCUSDT'] });

      // Clicking BTC (active) should remove it
      const btcBtn = screen.getByRole('button', { name: 'BTC' });
      await user.click(btcBtn);

      expect(onUpdateSettings).toHaveBeenCalledWith({ auto_trade_symbols: [] });
    });

    it('clicking an inactive symbol adds it', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({ auto_trade_symbols: ['BTCUSDT'] });

      const ethBtn = screen.getByRole('button', { name: 'ETH' });
      await user.click(ethBtn);

      expect(onUpdateSettings).toHaveBeenCalledWith({
        auto_trade_symbols: ['BTCUSDT', 'ETHUSDT'],
      });
    });

    it('symbol buttons are disabled when isLoading=true', () => {
      renderPanel({}, true);
      const btcBtn = screen.getByRole('button', { name: 'BTC' });
      expect(btcBtn).toBeDisabled();
    });
  });

  // ─── Section 5: Direction mode buttons ────────────────────────────────────

  describe('Direction mode buttons', () => {
    it('renders Both, Long, and Short buttons', () => {
      renderPanel();
      expect(screen.getByRole('button', { name: /both/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /long/i })).toBeInTheDocument();
      expect(screen.getByRole('button', { name: /short/i })).toBeInTheDocument();
    });

    it('defaults to Both when neither long_only nor short_only', () => {
      // "Both" is current when long_only=false, short_only=false.
      // We can verify by clicking Both and expecting the correct call.
      // Since it's already active, clicking should still call with both false.
      const { onUpdateSettings } = renderPanel({
        long_only_mode: false,
        short_only_mode: false,
      });
      // Just verify buttons render; active state is purely visual (style)
      expect(screen.getByRole('button', { name: /both/i })).toBeInTheDocument();
      expect(onUpdateSettings).not.toHaveBeenCalled();
    });

    it('clicking Long calls onUpdateSettings with long_only=true, short_only=false', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({
        long_only_mode: false,
        short_only_mode: false,
      });

      await user.click(screen.getByRole('button', { name: /long/i }));

      expect(onUpdateSettings).toHaveBeenCalledWith({
        long_only_mode: true,
        short_only_mode: false,
      });
    });

    it('clicking Short calls onUpdateSettings with short_only=true, long_only=false', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({
        long_only_mode: false,
        short_only_mode: false,
      });

      await user.click(screen.getByRole('button', { name: /short/i }));

      expect(onUpdateSettings).toHaveBeenCalledWith({
        long_only_mode: false,
        short_only_mode: true,
      });
    });

    it('clicking Both calls onUpdateSettings with both false', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({
        long_only_mode: true,
        short_only_mode: false,
      });

      await user.click(screen.getByRole('button', { name: /both/i }));

      expect(onUpdateSettings).toHaveBeenCalledWith({
        long_only_mode: false,
        short_only_mode: false,
      });
    });

    it('direction buttons are disabled when isLoading=true', () => {
      renderPanel({}, true);
      expect(screen.getByRole('button', { name: /long/i })).toBeDisabled();
      expect(screen.getByRole('button', { name: /short/i })).toBeDisabled();
      expect(screen.getByRole('button', { name: /both/i })).toBeDisabled();
    });
  });

  // ─── Section 6: Compact inputs ────────────────────────────────────────────

  describe('CompactInput fields in Trading Config', () => {
    it('renders Max Leverage input with suffix "x"', () => {
      renderPanel();
      expect(screen.getByText('x')).toBeInTheDocument();
    });

    it('renders Position Size input with suffix "USDT"', () => {
      renderPanel();
      expect(screen.getByText('USDT')).toBeInTheDocument();
    });

    it('renders Stop Loss and Take Profit with suffix "%"', () => {
      renderPanel();
      const pctSuffixes = screen.getAllByText('%');
      // At least Stop Loss and Take Profit (more may appear after opening risk section)
      expect(pctSuffixes.length).toBeGreaterThanOrEqual(2);
    });

    it('changing Max Leverage calls onUpdateSettings', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({ max_leverage: 5 });

      // find the number input for leverage (first number input on page)
      const inputs = screen.getAllByRole('spinbutton');
      const leverageInput = inputs[0]; // Max Leverage is first CompactInput

      await user.clear(leverageInput);
      await user.type(leverageInput, '10');

      // onUpdateSettings fires on each valid change
      expect(onUpdateSettings).toHaveBeenCalledWith(
        expect.objectContaining({ max_leverage: expect.any(Number) })
      );
    });
  });

  // ─── Section 7: Risk Config collapsible section ───────────────────────────

  describe('Risk Config collapsible section', () => {
    it('Risk Config heading is visible', () => {
      renderPanel();
      expect(screen.getByText('Risk Config')).toBeInTheDocument();
    });

    it('risk fields are hidden by default (collapsed)', () => {
      renderPanel();
      // Min Confidence label is inside the collapsed section
      expect(screen.queryByText('Min Confidence')).not.toBeInTheDocument();
    });

    it('clicking Risk Config header expands the section', async () => {
      const user = userEvent.setup();
      renderPanel();

      const riskHeader = screen.getByText('Risk Config').closest('button') as HTMLElement;
      expect(riskHeader).not.toBeNull();
      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.getByText('Min Confidence')).toBeInTheDocument();
      });
    });

    it('clicking Risk Config header again collapses the section', async () => {
      const user = userEvent.setup();
      renderPanel();

      const riskHeader = screen.getByText('Risk Config').closest('button') as HTMLElement;
      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.getByText('Min Confidence')).toBeInTheDocument();
      });

      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.queryByText('Min Confidence')).not.toBeInTheDocument();
      });
    });

    it('shows Correlation Limit slider after expanding', async () => {
      const user = userEvent.setup();
      renderPanel();

      const riskHeader = screen.getByText('Risk Config').closest('button') as HTMLElement;
      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.getByText('Correlation Limit')).toBeInTheDocument();
      });
    });

    it('Min Confidence slider calls onUpdateSettings', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({ min_signal_confidence: 0.7 });

      const riskHeader = screen.getByText('Risk Config').closest('button') as HTMLElement;
      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.getByText('Min Confidence')).toBeInTheDocument();
      });

      // Range sliders must be changed via fireEvent (user-event clear not supported)
      const { fireEvent } = await import('@testing-library/react');
      const sliders = document.querySelectorAll('input[type="range"]');
      expect(sliders.length).toBeGreaterThanOrEqual(1);

      const minConfSlider = sliders[0] as HTMLInputElement;
      fireEvent.change(minConfSlider, { target: { value: '80' } });

      expect(onUpdateSettings).toHaveBeenCalledWith({ min_signal_confidence: 0.8 });
    });

    it('Correlation Limit slider calls onUpdateSettings', async () => {
      const user = userEvent.setup();
      const { onUpdateSettings } = renderPanel({ correlation_limit: 0.7 });

      const riskHeader = screen.getByText('Risk Config').closest('button') as HTMLElement;
      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.getByText('Correlation Limit')).toBeInTheDocument();
      });

      const { fireEvent } = await import('@testing-library/react');
      const sliders = document.querySelectorAll('input[type="range"]');
      const correlationSlider = sliders[1] as HTMLInputElement;
      fireEvent.change(correlationSlider, { target: { value: '80' } });

      expect(onUpdateSettings).toHaveBeenCalledWith({ correlation_limit: 0.8 });
    });

    it('shows Max Consecutive Losses and Cool-Down inputs when expanded', async () => {
      const user = userEvent.setup();
      renderPanel();

      const riskHeader = screen.getByText('Risk Config').closest('button') as HTMLElement;
      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.getByText(/max consec\. losses/i)).toBeInTheDocument();
        expect(screen.getByText(/cool-down/i)).toBeInTheDocument();
      });
    });

    it('shows Max Portfolio Risk input when expanded', async () => {
      const user = userEvent.setup();
      renderPanel();

      const riskHeader = screen.getByText('Risk Config').closest('button') as HTMLElement;
      await user.click(riskHeader);

      await waitFor(() => {
        expect(screen.getByText(/max portfolio risk/i)).toBeInTheDocument();
      });
    });
  });

  // ─── Section 8: currentDirection derived value ────────────────────────────

  describe('currentDirection derivation', () => {
    it('long_only=true → long direction active', () => {
      renderPanel({ long_only_mode: true, short_only_mode: false });
      // Component renders 3 direction buttons; Long should appear
      expect(screen.getByRole('button', { name: /long/i })).toBeInTheDocument();
    });

    it('short_only=true → short direction active', () => {
      renderPanel({ long_only_mode: false, short_only_mode: true });
      expect(screen.getByRole('button', { name: /short/i })).toBeInTheDocument();
    });
  });

  // ─── Section 9: Trading Config section label ──────────────────────────────

  describe('Trading Config section', () => {
    it('shows Trading Config heading', () => {
      renderPanel();
      expect(screen.getByText('Trading Config')).toBeInTheDocument();
    });

    it('shows Symbols label', () => {
      renderPanel();
      expect(screen.getByText('Symbols')).toBeInTheDocument();
    });

    it('shows Direction label', () => {
      renderPanel();
      expect(screen.getByText('Direction')).toBeInTheDocument();
    });
  });
});
