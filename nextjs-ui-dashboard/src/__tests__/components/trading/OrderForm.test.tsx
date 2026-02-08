/**
 * OrderForm Component Tests
 *
 * Comprehensive tests for the OrderForm component covering:
 * - Rendering and initial state
 * - Form field validation
 * - Different order types (Market, Limit, Stop-Limit)
 * - Buy vs Sell order selection
 * - Form submission with valid data
 * - Error handling and display
 * - Trading mode awareness (Paper vs Real)
 * - Accessibility
 *
 * @spec:FR-TRADING-016 - Real Trading System
 */

import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { render } from '@/test/utils';
import { OrderForm, OrderFormData } from '@/components/trading/OrderForm';
import React from 'react';

// Mock dependencies
const mockToast = vi.fn();
vi.mock('@/hooks/use-toast', () => ({
  useToast: () => ({
    toast: mockToast,
  }),
}));

const mockMode = vi.fn(() => 'paper');
vi.mock('@/hooks/useTradingMode', () => ({
  useTradingMode: () => ({
    mode: mockMode(),
  }),
}));

vi.mock('@/utils/logger', () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
  },
}));

// Helper function to interact with Radix UI Select components
async function selectOption(user: ReturnType<typeof userEvent.setup>, selectLabel: RegExp, optionText: string) {
  const selectTrigger = screen.getByLabelText(selectLabel);
  await user.click(selectTrigger);
  // Use exact match to avoid partial matches (e.g., "Limit" matching "Stop-Limit")
  const option = screen.getByRole('option', { name: new RegExp(`^${optionText}$`, 'i') });
  await user.click(option);
}

describe('OrderForm', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    mockMode.mockReturnValue('paper');
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  // ========================================
  // Rendering Tests
  // ========================================

  describe('Rendering', () => {
    it('renders correctly with default props', () => {
      render(<OrderForm />);

      expect(screen.getByText('Place Order')).toBeInTheDocument();
      expect(screen.getByLabelText(/symbol/i)).toBeInTheDocument();
      expect(screen.getByText('Buy')).toBeInTheDocument();
      expect(screen.getByText('Sell')).toBeInTheDocument();
      expect(screen.getByLabelText(/order type/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/quantity/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/leverage/i)).toBeInTheDocument();
    });

    it('uses initial symbol from props', () => {
      render(<OrderForm symbol="ETHUSDT" />);

      // Check that the select trigger shows ETHUSDT (getAllByText because Radix renders both <span> and hidden <option>)
      const elements = screen.getAllByText('ETHUSDT');
      expect(elements.length).toBeGreaterThanOrEqual(1);
    });

    it('has all required symbols in dropdown', () => {
      render(<OrderForm />);

      // Check that the select trigger shows default BTCUSDT
      const elements = screen.getAllByText('BTCUSDT');
      expect(elements.length).toBeGreaterThanOrEqual(1);
    });

    it('displays order summary section', () => {
      render(<OrderForm />);

      expect(screen.getByText(/order value/i)).toBeInTheDocument();
      expect(screen.getByText(/with leverage/i)).toBeInTheDocument();
    });

    it('displays correct button text in paper mode', () => {
      mockMode.mockReturnValue('paper');
      render(<OrderForm />);

      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      expect(submitButton).toBeInTheDocument();
      expect(submitButton).not.toHaveTextContent('Real Money');
    });

    it('displays warning text in real mode', () => {
      mockMode.mockReturnValue('real');
      render(<OrderForm />);

      const submitButton = screen.getByRole('button', { name: /⚠️ buy btcusdt \(real money\)/i });
      expect(submitButton).toBeInTheDocument();
      expect(screen.getByText(/you will be asked to confirm/i)).toBeInTheDocument();
    });
  });

  // ========================================
  // Accessibility Tests
  // ========================================

  describe('Accessibility', () => {
    it('has proper labels for all form fields', () => {
      render(<OrderForm />);

      expect(screen.getByLabelText(/symbol/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/order type/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/quantity/i)).toBeInTheDocument();
      expect(screen.getByLabelText(/leverage/i)).toBeInTheDocument();
    });

    it('has proper aria attributes on submit button', () => {
      render(<OrderForm />);

      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      expect(submitButton).toHaveAttribute('type', 'submit');
    });

    it('form is keyboard navigable', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      // Tab through form fields
      await user.tab();
      expect(screen.getByLabelText(/symbol/i)).toHaveFocus();

      await user.tab();
      await user.tab(); // Skip tabs trigger
      await user.tab();
      expect(screen.getByLabelText(/order type/i)).toHaveFocus();
    });
  });

  // ========================================
  // Order Type Tests
  // ========================================

  describe('Order Types', () => {
    it('defaults to market order type', () => {
      render(<OrderForm />);

      // Check that Market is displayed (Radix renders both <span> and hidden <option>)
      const elements = screen.getAllByText('Market');
      expect(elements.length).toBeGreaterThanOrEqual(1);
    });

    it('does not show price field for market orders', () => {
      render(<OrderForm />);

      expect(screen.queryByLabelText(/limit price/i)).not.toBeInTheDocument();
      expect(screen.queryByLabelText(/stop price/i)).not.toBeInTheDocument();
    });

    it('shows limit price field for limit orders', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      await selectOption(user, /order type/i, 'Limit');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
      });
    });

    it('shows both limit and stop price fields for stop-limit orders', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      await selectOption(user, /order type/i, 'Stop-Limit');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/stop price/i)).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Buy/Sell Side Tests
  // ========================================

  describe('Buy/Sell Side Selection', () => {
    it('defaults to buy side', () => {
      render(<OrderForm />);

      const buyTab = screen.getByRole('tab', { name: /buy/i });
      expect(buyTab).toHaveAttribute('data-state', 'active');
    });

    it('switches to sell side', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      const sellTab = screen.getByRole('tab', { name: /sell/i });
      await user.click(sellTab);

      await waitFor(() => {
        expect(sellTab).toHaveAttribute('data-state', 'active');
      });
    });

    it('updates submit button text when switching sides', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      expect(screen.getByRole('button', { name: /buy btcusdt/i })).toBeInTheDocument();

      const sellTab = screen.getByRole('tab', { name: /sell/i });
      await user.click(sellTab);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /sell btcusdt/i })).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Form Validation Tests
  // ========================================

  describe('Form Validation', () => {
    it('shows error when submitting with empty quantity', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Invalid Quantity',
          description: 'Please enter a valid quantity',
          variant: 'destructive',
        });
      });
    });

    it('shows error when quantity is zero', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0');

      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Invalid Quantity',
          description: 'Please enter a valid quantity',
          variant: 'destructive',
        });
      });
    });

    it('shows error when quantity is negative', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '-1');

      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Invalid Quantity',
          description: 'Please enter a valid quantity',
          variant: 'destructive',
        });
      });
    });

    it('shows error when limit order has no price', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      // Switch to limit order
      await selectOption(user, /order type/i, 'Limit');

      // Enter valid quantity
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.01');

      // Submit without price
      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Invalid Price',
          description: 'Please enter a valid limit price',
          variant: 'destructive',
        });
      });
    });

    it('shows error when stop-limit order has no stop price', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      // Switch to stop-limit order
      const orderTypeSelect = screen.getByLabelText(/order type/i);
      await user.click(orderTypeSelect);
      const stopLimitOption = screen.getByRole('option', { name: /stop-limit/i });
      await user.click(stopLimitOption);

      // Enter valid quantity and limit price
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.01');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
      });

      const limitPriceInput = screen.getByLabelText(/limit price/i);
      await user.clear(limitPriceInput);
      await user.type(limitPriceInput, '50000');

      // Submit without stop price
      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Invalid Stop Price',
          description: 'Please enter a valid stop price',
          variant: 'destructive',
        });
      });
    });

    it('validates price is greater than zero for limit orders', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      // Switch to limit order
      await selectOption(user, /order type/i, 'Limit');

      // Enter valid quantity and invalid price
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.01');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
      });

      const limitPriceInput = screen.getByLabelText(/limit price/i);
      await user.clear(limitPriceInput);
      await user.type(limitPriceInput, '0');

      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Invalid Price',
          description: 'Please enter a valid limit price',
          variant: 'destructive',
        });
      });
    });
  });

  // ========================================
  // Form Submission Tests
  // ========================================

  describe('Form Submission', () => {
    it('calls onSubmit with correct data for market order in paper mode', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn();
      mockMode.mockReturnValue('paper');

      render(<OrderForm onSubmit={onSubmit} />);

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.5');

      // Submit
      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith({
          symbol: 'BTCUSDT',
          orderType: 'market',
          side: 'buy',
          quantity: 0.5,
          leverage: 10,
        });
      });
    });

    it('calls onSubmit with correct data for limit order', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn();
      mockMode.mockReturnValue('paper');

      render(<OrderForm onSubmit={onSubmit} />);

      // Switch to limit order
      await selectOption(user, /order type/i, 'Limit');

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.1');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
      });

      const limitPriceInput = screen.getByLabelText(/limit price/i);
      await user.clear(limitPriceInput);
      await user.type(limitPriceInput, '45000');

      // Submit
      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith({
          symbol: 'BTCUSDT',
          orderType: 'limit',
          side: 'buy',
          quantity: 0.1,
          price: 45000,
          leverage: 10,
        });
      });
    });

    it('calls onSubmit with correct data for stop-limit order', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn();
      mockMode.mockReturnValue('paper');

      render(<OrderForm onSubmit={onSubmit} />);

      // Switch to stop-limit order
      const orderTypeSelect = screen.getByLabelText(/order type/i);
      await user.click(orderTypeSelect);
      const stopLimitOption = screen.getByRole('option', { name: /stop-limit/i });
      await user.click(stopLimitOption);

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.2');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
        expect(screen.getByLabelText(/stop price/i)).toBeInTheDocument();
      });

      const limitPriceInput = screen.getByLabelText(/limit price/i);
      await user.clear(limitPriceInput);
      await user.type(limitPriceInput, '45000');

      const stopPriceInput = screen.getByLabelText(/stop price/i);
      await user.clear(stopPriceInput);
      await user.type(stopPriceInput, '44000');

      // Submit
      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith({
          symbol: 'BTCUSDT',
          orderType: 'stop-limit',
          side: 'buy',
          quantity: 0.2,
          price: 45000,
          stopPrice: 44000,
          leverage: 10,
        });
      });
    });

    it('calls onSubmit with correct data for sell order', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn();
      mockMode.mockReturnValue('paper');

      render(<OrderForm onSubmit={onSubmit} />);

      // Switch to sell
      const sellTab = screen.getByRole('tab', { name: /sell/i });
      await user.click(sellTab);

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.3');

      // Submit
      const submitButton = screen.getByRole('button', { name: /sell btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith({
          symbol: 'BTCUSDT',
          orderType: 'market',
          side: 'sell',
          quantity: 0.3,
          leverage: 10,
        });
      });
    });

    it('uses custom leverage value', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn();
      mockMode.mockReturnValue('paper');

      render(<OrderForm onSubmit={onSubmit} />);

      // Change leverage
      const leverageSelect = screen.getByLabelText(/leverage/i);
      await user.click(leverageSelect);
      const leverage20 = screen.getByRole('option', { name: /20x/i });
      await user.click(leverage20);

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.1');

      // Submit
      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith(
          expect.objectContaining({
            leverage: 20,
          })
        );
      });
    });

    it('calls onConfirmationRequired in real mode', async () => {
      const user = userEvent.setup();
      const onConfirmationRequired = vi.fn();
      mockMode.mockReturnValue('real');

      render(<OrderForm onConfirmationRequired={onConfirmationRequired} />);

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.5');

      // Submit
      const submitButton = screen.getByRole('button', { name: /⚠️ buy btcusdt \(real money\)/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(onConfirmationRequired).toHaveBeenCalledWith({
          symbol: 'BTCUSDT',
          orderType: 'market',
          side: 'buy',
          quantity: 0.5,
          leverage: 10,
        });
      });
    });

    it('shows default toast in paper mode when no onSubmit provided', async () => {
      const user = userEvent.setup();
      mockMode.mockReturnValue('paper');

      render(<OrderForm />);

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '0.5');

      // Submit
      const submitButton = screen.getByRole('button', { name: /buy btcusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith({
          title: 'Order Submitted (Paper)',
          description: 'BUY 0.5 BTCUSDT',
        });
      });
    });
  });

  // ========================================
  // Symbol Selection Tests
  // ========================================

  describe('Symbol Selection', () => {
    it('allows changing symbol', async () => {
      const user = userEvent.setup();
      const onSubmit = vi.fn();
      mockMode.mockReturnValue('paper');

      render(<OrderForm onSubmit={onSubmit} />);

      // Change symbol
      const symbolSelect = screen.getByLabelText(/symbol/i);
      await user.click(symbolSelect);
      const ethOption = screen.getByRole('option', { name: /ethusdt/i });
      await user.click(ethOption);

      // Fill form
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '1');

      // Submit
      const submitButton = screen.getByRole('button', { name: /buy ethusdt/i });
      await user.click(submitButton);

      await waitFor(() => {
        expect(onSubmit).toHaveBeenCalledWith(
          expect.objectContaining({
            symbol: 'ETHUSDT',
          })
        );
      });
    });

    it('updates button text when symbol changes', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      expect(screen.getByRole('button', { name: /buy btcusdt/i })).toBeInTheDocument();

      const symbolSelect = screen.getByLabelText(/symbol/i);
      await user.click(symbolSelect);
      const solOption = screen.getByRole('option', { name: /solusdt/i });
      await user.click(solOption);

      await waitFor(() => {
        expect(screen.getByRole('button', { name: /buy solusdt/i })).toBeInTheDocument();
      });
    });
  });

  // ========================================
  // Order Summary Calculation Tests
  // ========================================

  describe('Order Summary Calculations', () => {
    it('calculates order value for limit orders', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      // Switch to limit order
      await selectOption(user, /order type/i, 'Limit');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
      });

      // Enter quantity and price
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '2');

      const limitPriceInput = screen.getByLabelText(/limit price/i);
      await user.clear(limitPriceInput);
      await user.type(limitPriceInput, '1000');

      // Check calculated value
      await waitFor(() => {
        expect(screen.getByText('$2000.00')).toBeInTheDocument();
      });
    });

    it('calculates order value with leverage', async () => {
      const user = userEvent.setup();
      render(<OrderForm />);

      // Switch to limit order
      await selectOption(user, /order type/i, 'Limit');

      await waitFor(() => {
        expect(screen.getByLabelText(/limit price/i)).toBeInTheDocument();
      });

      // Enter quantity and price
      const quantityInput = screen.getByLabelText(/quantity/i);
      await user.clear(quantityInput);
      await user.type(quantityInput, '2');

      const limitPriceInput = screen.getByLabelText(/limit price/i);
      await user.clear(limitPriceInput);
      await user.type(limitPriceInput, '1000');

      // Check calculated value with default 10x leverage
      await waitFor(() => {
        expect(screen.getByText('$20000.00')).toBeInTheDocument();
      });
    });
  });
});
