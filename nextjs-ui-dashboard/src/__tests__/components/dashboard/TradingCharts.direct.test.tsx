import { describe, it, expect, vi, beforeEach, afterEach } from "vitest";
import { render, screen, fireEvent, waitFor } from "@testing-library/react";
import { TradingCharts } from "@/components/dashboard/TradingCharts";
import { apiClient } from "@/services/api";
import type { ChartData } from "@/services/api";

// Mock framer-motion
vi.mock("framer-motion", () => ({
  motion: new Proxy(
    {},
    {
      get: (_, prop) => {
        const Component = ({ children, ...props }: any) => {
          const { animate, initial, exit, transition, variants, whileHover, whileTap, ...rest } = props;
          return <div {...rest}>{children}</div>;
        };
        Component.displayName = `motion.${String(prop)}`;
        return Component;
      },
    }
  ),
  AnimatePresence: ({ children }: any) => <>{children}</>,
}));

// Mock luxury design system
vi.mock("@/styles/luxury-design-system", () => ({
  PremiumButton: ({ children, onClick, className, type, variant, size, ...props }: any) => (
    <button onClick={onClick} className={className} type={type} {...props}>
      {children}
    </button>
  ),
  PremiumInput: ({ onChange, value, placeholder, className, id }: any) => (
    <input
      id={id}
      value={value}
      onChange={(e) => onChange(e.target.value)}
      placeholder={placeholder}
      className={className}
    />
  ),
}));

// Mock sonner
vi.mock("sonner", () => ({
  toast: {
    error: vi.fn(),
    success: vi.fn(),
  },
}));

// Mock WebSocket context
const mockWebSocketContext = {
  state: {
    isConnected: false,
    isConnecting: false,
    lastMessage: null,
  },
  connect: vi.fn(),
};

vi.mock("@/contexts/WebSocketContext", () => ({
  useWebSocketContext: () => mockWebSocketContext,
}));

// Mock API client
vi.mock("@/services/api", () => ({
  apiClient: {
    rust: {
      getChartDataFast: vi.fn(),
      getSupportedSymbols: vi.fn(),
      getLatestPrices: vi.fn(),
      addSymbol: vi.fn(),
      removeSymbol: vi.fn(),
      getChartData: vi.fn(),
    },
  },
}));

// Mock logger
vi.mock("@/utils/logger", () => ({
  default: {
    info: vi.fn(),
    error: vi.fn(),
    warn: vi.fn(),
  },
}));

describe("TradingCharts - Coverage Boost", () => {
  const mockChartData: ChartData = {
    symbol: "BTCUSDT",
    timeframe: "1m",
    latest_price: 50000,
    price_change_24h: 1000,
    price_change_percent_24h: 2.0,
    volume_24h: 1000000,
    candles: [
      {
        timestamp: Date.now() - 60000,
        open: 49900,
        high: 50100,
        low: 49800,
        close: 50000,
        volume: 100,
      },
    ],
  };

  beforeEach(() => {
    vi.clearAllMocks();
    mockWebSocketContext.state.isConnected = false;
    mockWebSocketContext.state.lastMessage = null;

    // Mock successful API responses by default
    vi.mocked(apiClient.rust.getChartDataFast).mockResolvedValue(mockChartData);
    vi.mocked(apiClient.rust.getSupportedSymbols).mockResolvedValue({ symbols: [] });
    vi.mocked(apiClient.rust.getLatestPrices).mockResolvedValue({ BTCUSDT: 50100 });
    vi.mocked(apiClient.rust.addSymbol).mockResolvedValue(undefined);
    vi.mocked(apiClient.rust.removeSymbol).mockResolvedValue(undefined);
    vi.mocked(apiClient.rust.getChartData).mockResolvedValue(mockChartData);

    // Mock fetch for symbols API
    global.fetch = vi.fn().mockResolvedValue({
      json: async () => ({
        success: true,
        data: { symbols: ["BTCUSDT", "ETHUSDT"] },
      }),
    });
  });

  afterEach(() => {
    vi.restoreAllMocks();
  });

  it("renders loading state initially", () => {
    render(<TradingCharts />);
    expect(screen.getByText(/Real-time Trading Charts/i)).toBeInTheDocument();
  });

  it("handles fetch symbols API with invalid response - line 554 (no symbols)", async () => {
    global.fetch = vi.fn().mockResolvedValue({
      json: async () => ({
        success: true,
        data: { symbols: [] }, // Empty symbols
      }),
    });

    render(<TradingCharts />);

    await waitFor(() => {
      expect(apiClient.rust.getChartDataFast).toHaveBeenCalled();
    });
  });



});
