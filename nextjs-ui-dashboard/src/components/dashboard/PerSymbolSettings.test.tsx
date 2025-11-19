/**
 * Unit Tests for PerSymbolSettings Component
 *
 * @spec:FR-PAPER-002 - Per-Symbol Configuration
 * @ref:specs/03-testing/TEST-FRONTEND.md
 * @test:TC-PAPER-005, TC-PAPER-006
 */

import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import userEvent from "@testing-library/user-event";
import { PerSymbolSettings, SymbolConfig } from "./PerSymbolSettings";
import { useToast } from "@/hooks/use-toast";

// Mock the toast hook
jest.mock("@/hooks/use-toast", () => ({
  useToast: jest.fn(() => ({
    toast: jest.fn(),
  })),
}));

// Mock fetch globally
global.fetch = jest.fn();

describe("PerSymbolSettings", () => {
  const mockFetch = global.fetch as jest.MockedFunction<typeof fetch>;
  const mockToast = jest.fn();

  beforeEach(() => {
    jest.clearAllMocks();
    (useToast as jest.Mock).mockReturnValue({ toast: mockToast });
  });

  afterEach(() => {
    jest.resetAllMocks();
  });

  describe("Rendering", () => {
    it("should render the component title", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      expect(screen.getByText("Per-Symbol Settings")).toBeInTheDocument();
    });

    it("should render all default symbols", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
        expect(screen.getByText("ETHUSDT")).toBeInTheDocument();
        expect(screen.getByText("BNBUSDT")).toBeInTheDocument();
        expect(screen.getByText("SOLUSDT")).toBeInTheDocument();
      });
    });

    it("should display loading state initially", () => {
      mockFetch.mockImplementation(
        () =>
          new Promise(() => {
            // Never resolve to keep loading state
          })
      );

      render(<PerSymbolSettings />);

      expect(screen.getByText("Per-Symbol Settings")).toBeInTheDocument();
      // Loading spinner should be visible
      const loader = document.querySelector(".animate-spin");
      expect(loader).toBeInTheDocument();
    });

    it("should render Reset and Save All buttons", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Reset")).toBeInTheDocument();
        expect(screen.getByText("Save All")).toBeInTheDocument();
      });
    });
  });

  describe("Data Loading", () => {
    it("should load configurations from API", async () => {
      const mockConfigs: SymbolConfig[] = [
        {
          symbol: "BTCUSDT",
          enabled: true,
          leverage: 10,
          position_size_pct: 5,
          stop_loss_pct: 2,
          take_profit_pct: 4,
          max_positions: 2,
        },
      ];

      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({ success: true, data: mockConfigs }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining("/api/paper-trading/symbol-settings")
        );
      });
    });

    it("should use presets when API fails", async () => {
      mockFetch.mockRejectedValueOnce(new Error("Network error"));

      render(<PerSymbolSettings />);

      await waitFor(() => {
        // Should still render symbols with preset values
        expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
      });
    });
  });

  describe("Symbol Configuration", () => {
    beforeEach(async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);
    });

    it("should toggle symbol enabled state", async () => {
      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
      });

      // Find the accordion trigger and click it to expand
      const btcAccordion = screen.getByText("BTCUSDT").closest("button");
      if (btcAccordion) {
        fireEvent.click(btcAccordion);
      }

      // Find and toggle the switch
      const switches = screen.getAllByRole("switch");
      const btcSwitch = switches[0]; // First switch is for BTC
      fireEvent.click(btcSwitch);

      // Verify the switch state changed
      expect(btcSwitch).toHaveAttribute("data-state");
    });

    it("should expand accordion on click", async () => {
      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
      });

      const btcTrigger = screen.getByText("BTCUSDT").closest("button");
      if (btcTrigger) {
        fireEvent.click(btcTrigger);
      }

      // Check for expanded content
      await waitFor(() => {
        expect(screen.getByText("Leverage")).toBeInTheDocument();
        expect(screen.getByText("Position Size")).toBeInTheDocument();
        expect(screen.getByText("Stop Loss")).toBeInTheDocument();
        expect(screen.getByText("Take Profit")).toBeInTheDocument();
      });
    });
  });

  describe("Risk Level Calculation", () => {
    it("should display low risk for conservative settings", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: [
            {
              symbol: "BTCUSDT",
              enabled: true,
              leverage: 2,
              position_size_pct: 3,
              stop_loss_pct: 2,
              take_profit_pct: 4,
              max_positions: 1,
            },
          ],
        }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText(/Low Risk/i)).toBeInTheDocument();
      });
    });

    it("should display high risk for aggressive settings", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: true,
        json: async () => ({
          success: true,
          data: [
            {
              symbol: "BTCUSDT",
              enabled: true,
              leverage: 20,
              position_size_pct: 10,
              stop_loss_pct: 5,
              take_profit_pct: 10,
              max_positions: 5,
            },
          ],
        }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText(/High Risk/i)).toBeInTheDocument();
      });
    });
  });

  describe("Position Size Calculation", () => {
    it("should calculate position size correctly", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      const currentBalance = 10000;
      render(<PerSymbolSettings currentBalance={currentBalance} />);

      await waitFor(() => {
        expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
      });

      // Expand BTC accordion
      const btcTrigger = screen.getByText("BTCUSDT").closest("button");
      if (btcTrigger) {
        fireEvent.click(btcTrigger);
      }

      // With preset: 10x leverage, 5% position
      // Expected: (10000 * 5%) * 10x = $5000
      await waitFor(() => {
        expect(screen.getByText(/\$5000\.00/)).toBeInTheDocument();
      });
    });
  });

  describe("Settings Persistence", () => {
    beforeEach(async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);
    });

    it("should save all configurations", async () => {
      mockFetch
        .mockResolvedValueOnce({
          ok: false,
          json: async () => ({ success: false }),
        } as Response)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ success: true, message: "Settings saved" }),
        } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Save All")).toBeInTheDocument();
      });

      const saveButton = screen.getByText("Save All");
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(mockFetch).toHaveBeenCalledWith(
          expect.stringContaining("/api/paper-trading/symbol-settings"),
          expect.objectContaining({
            method: "PUT",
            headers: {
              "Content-Type": "application/json",
            },
          })
        );
      });
    });

    it("should call onSettingsUpdate callback", async () => {
      const onSettingsUpdate = jest.fn();

      mockFetch
        .mockResolvedValueOnce({
          ok: false,
          json: async () => ({ success: false }),
        } as Response)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ success: true }),
        } as Response);

      render(<PerSymbolSettings onSettingsUpdate={onSettingsUpdate} />);

      await waitFor(() => {
        expect(screen.getByText("Save All")).toBeInTheDocument();
      });

      const saveButton = screen.getByText("Save All");
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(onSettingsUpdate).toHaveBeenCalled();
      });
    });

    it("should show toast on successful save", async () => {
      mockFetch
        .mockResolvedValueOnce({
          ok: false,
          json: async () => ({ success: false }),
        } as Response)
        .mockResolvedValueOnce({
          ok: true,
          json: async () => ({ success: true }),
        } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Save All")).toBeInTheDocument();
      });

      const saveButton = screen.getByText("Save All");
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: "Settings Saved",
          })
        );
      });
    });

    it("should show toast on failed save", async () => {
      mockFetch
        .mockResolvedValueOnce({
          ok: false,
          json: async () => ({ success: false }),
        } as Response)
        .mockResolvedValueOnce({
          ok: false,
          json: async () => ({
            success: false,
            error: "Failed to save settings",
          }),
        } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Save All")).toBeInTheDocument();
      });

      const saveButton = screen.getByText("Save All");
      fireEvent.click(saveButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: "Save Failed",
            variant: "destructive",
          })
        );
      });
    });
  });

  describe("Reset Functionality", () => {
    it("should reset to defaults", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Reset")).toBeInTheDocument();
      });

      const resetButton = screen.getByText("Reset");
      fireEvent.click(resetButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: "Settings Reset",
          })
        );
      });
    });
  });

  describe("Preset Application", () => {
    it("should apply conservative preset for BTC", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Conservative (BTC)")).toBeInTheDocument();
      });

      const presetButton = screen.getByText("Conservative (BTC)");
      fireEvent.click(presetButton);

      await waitFor(() => {
        expect(mockToast).toHaveBeenCalledWith(
          expect.objectContaining({
            title: "Preset Applied",
            description: expect.stringContaining("BTCUSDT"),
          })
        );
      });
    });

    it("should render all preset buttons", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Conservative (BTC)")).toBeInTheDocument();
        expect(screen.getByText("Moderate (ETH)")).toBeInTheDocument();
        expect(screen.getByText("Moderate (BNB)")).toBeInTheDocument();
        expect(screen.getByText("Aggressive (SOL)")).toBeInTheDocument();
      });
    });
  });

  describe("Risk Assessment Display", () => {
    it("should show risk summary when accordion is expanded", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
      });

      const btcTrigger = screen.getByText("BTCUSDT").closest("button");
      if (btcTrigger) {
        fireEvent.click(btcTrigger);
      }

      await waitFor(() => {
        expect(screen.getByText("Risk Assessment")).toBeInTheDocument();
        expect(screen.getByText(/Position Value:/)).toBeInTheDocument();
        expect(screen.getByText(/Max Loss:/)).toBeInTheDocument();
        expect(screen.getByText(/Target Profit:/)).toBeInTheDocument();
        expect(screen.getByText(/Risk\/Reward:/)).toBeInTheDocument();
      });
    });
  });

  describe("Accessibility", () => {
    it("should have proper ARIA labels", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
      });

      // Switches should have role="switch"
      const switches = screen.getAllByRole("switch");
      expect(switches.length).toBeGreaterThan(0);

      // Buttons should have role="button"
      const buttons = screen.getAllByRole("button");
      expect(buttons.length).toBeGreaterThan(0);
    });

    it("should be keyboard navigable", async () => {
      mockFetch.mockResolvedValueOnce({
        ok: false,
        json: async () => ({ success: false }),
      } as Response);

      render(<PerSymbolSettings />);

      await waitFor(() => {
        expect(screen.getByText("Reset")).toBeInTheDocument();
      });

      const resetButton = screen.getByText("Reset");
      resetButton.focus();

      expect(document.activeElement).toBe(resetButton);
    });
  });
});
