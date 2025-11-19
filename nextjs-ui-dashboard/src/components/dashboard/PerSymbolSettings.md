# PerSymbolSettings Component

## Overview

The `PerSymbolSettings` component provides a comprehensive UI for configuring different trading parameters for each cryptocurrency symbol (BTC, ETH, SOL, BNB). It enables granular control over leverage, position sizing, stop-loss, take-profit, and concurrent position limits on a per-symbol basis.

## Features

### Core Functionality
- **Multi-Symbol Configuration**: Support for BTCUSDT, ETHUSDT, BNBUSDT, SOLUSDT
- **Enable/Disable Toggle**: Activate or deactivate trading for specific symbols
- **Expandable Accordion**: Organized, collapsible interface for each symbol
- **Real-time Calculations**: Live updates of position values and risk metrics

### Configuration Options (Per Symbol)

1. **Leverage** (1x - 20x)
   - Slider control with real-time value display
   - Affects position size and risk calculations

2. **Position Size** (1% - 10% of portfolio)
   - Percentage-based allocation
   - Shows calculated dollar amount based on current balance

3. **Stop Loss** (0.5% - 5%)
   - Maximum acceptable loss before position closure
   - Displays calculated loss amount in dollars

4. **Take Profit** (1% - 10%)
   - Target profit for automatic position closure
   - Shows potential profit in dollars

5. **Max Concurrent Positions** (1-5)
   - Limit simultaneous positions per symbol
   - Helps with risk diversification

### Risk Assessment

The component provides real-time risk analysis:

- **Risk Level Calculation**: Automatically categorizes as Low/Moderate/High
- **Visual Indicators**: Color-coded badges and icons
- **Risk Summary Card**: Shows:
  - Position value
  - Maximum loss
  - Target profit
  - Risk/Reward ratio

### Preset Configurations

Pre-configured strategies for quick setup:

1. **Conservative (BTC)**
   - 10x leverage
   - 5% position size
   - 2% stop loss
   - 4% take profit
   - 2 max positions

2. **Moderate (ETH/BNB)**
   - 7x leverage
   - 4% position size
   - 2.5% stop loss
   - 5% take profit
   - 2 max positions

3. **Aggressive (SOL)**
   - 5x leverage
   - 3% position size
   - 3% stop loss
   - 6% take profit
   - 1 max position

## API Integration

### Backend Endpoints

The component integrates with the following REST API endpoints:

#### GET `/api/paper-trading/symbol-settings`
Load current symbol configurations.

**Response:**
```json
{
  "success": true,
  "data": [
    {
      "symbol": "BTCUSDT",
      "enabled": true,
      "leverage": 10,
      "position_size_pct": 5.0,
      "stop_loss_pct": 2.0,
      "take_profit_pct": 4.0,
      "max_positions": 2
    }
  ],
  "timestamp": "2025-11-19T10:30:00Z"
}
```

#### PUT `/api/paper-trading/symbol-settings`
Save all symbol configurations.

**Request:**
```json
{
  "symbols": [
    {
      "symbol": "BTCUSDT",
      "enabled": true,
      "leverage": 10,
      "position_size_pct": 5.0,
      "stop_loss_pct": 2.0,
      "take_profit_pct": 4.0,
      "max_positions": 2
    }
  ]
}
```

**Response:**
```json
{
  "success": true,
  "message": "Symbol settings updated successfully",
  "timestamp": "2025-11-19T10:30:00Z"
}
```

#### PUT `/api/paper-trading/symbol-settings/{symbol}`
Save individual symbol configuration.

**Request:**
```json
{
  "symbol": "BTCUSDT",
  "enabled": true,
  "leverage": 10,
  "position_size_pct": 5.0,
  "stop_loss_pct": 2.0,
  "take_profit_pct": 4.0,
  "max_positions": 2
}
```

## Usage

### Basic Usage

```tsx
import { PerSymbolSettings } from "@/components/dashboard/PerSymbolSettings";

export function Dashboard() {
  return (
    <PerSymbolSettings
      currentBalance={10000}
      onSettingsUpdate={(configs) => {
        console.log("Updated configs:", configs);
      }}
    />
  );
}
```

### Integration with usePaperTrading Hook

```tsx
import { PerSymbolSettings } from "@/components/dashboard/PerSymbolSettings";
import { usePaperTrading } from "@/hooks/usePaperTrading";

export function TradingDashboard() {
  const { portfolio } = usePaperTrading();

  return (
    <PerSymbolSettings
      currentBalance={portfolio.current_balance}
      onSettingsUpdate={(configs) => {
        // Handle configuration updates
        console.log("Symbol configs updated:", configs);
      }}
    />
  );
}
```

### Advanced Usage with State Management

```tsx
import { useState } from "react";
import { PerSymbolSettings, SymbolConfig } from "@/components/dashboard/PerSymbolSettings";
import { usePaperTrading } from "@/hooks/usePaperTrading";

export function AdvancedTradingDashboard() {
  const { portfolio, refreshData } = usePaperTrading();
  const [symbolConfigs, setSymbolConfigs] = useState<SymbolConfig[]>([]);

  const handleSettingsUpdate = async (configs: SymbolConfig[]) => {
    setSymbolConfigs(configs);

    // Refresh trading data after settings update
    await refreshData();

    // Additional custom logic
    console.log("Configurations updated:", configs);
  };

  return (
    <PerSymbolSettings
      currentBalance={portfolio.current_balance}
      onSettingsUpdate={handleSettingsUpdate}
    />
  );
}
```

## Component Props

### PerSymbolSettingsProps

| Prop | Type | Default | Description |
|------|------|---------|-------------|
| `currentBalance` | `number` | `10000` | Current portfolio balance for calculations |
| `onSettingsUpdate` | `(configs: SymbolConfig[]) => void` | `undefined` | Callback when settings are saved |

## Type Definitions

### SymbolConfig

```typescript
interface SymbolConfig {
  symbol: string;              // Trading pair (e.g., "BTCUSDT")
  enabled: boolean;            // Whether trading is enabled for this symbol
  leverage: number;            // Leverage multiplier (1-20)
  position_size_pct: number;   // Position size as % of portfolio (1-10)
  stop_loss_pct: number;       // Stop loss percentage (0.5-5)
  take_profit_pct: number;     // Take profit percentage (1-10)
  max_positions: number;       // Max concurrent positions (1-5)
}
```

### RiskLevel

```typescript
type RiskLevel = "low" | "moderate" | "high";
```

Risk level is calculated based on the formula:
```
riskScore = leverage × position_size_pct

- Low: riskScore ≤ 25
- Moderate: 25 < riskScore ≤ 50
- High: riskScore > 50
```

## Styling & Theming

The component uses Tailwind CSS and shadcn/ui components, supporting both light and dark themes:

- **Color Scheme**: Adapts to theme automatically
- **Risk Colors**:
  - Low: Green (`text-green-600 dark:text-green-400`)
  - Moderate: Yellow (`text-yellow-600 dark:text-yellow-400`)
  - High: Red (`text-red-600 dark:text-red-400`)

## Accessibility

- **Keyboard Navigation**: Full keyboard support via Accordion
- **Screen Readers**: Descriptive labels and ARIA attributes
- **Focus Management**: Clear focus indicators on interactive elements
- **Color Independence**: Risk levels indicated by both color and icon

## State Management

The component manages state internally:

1. **Initial Load**: Fetches configs from backend API
2. **Local Updates**: Modifies local state for immediate UI feedback
3. **Persistence**: Saves to backend when user clicks "Save All" or individual save buttons
4. **Reset**: Reverts to preset configurations

## Error Handling

- **Failed API Calls**: Falls back to preset configurations
- **Network Errors**: Shows toast notifications with error details
- **Validation**: Enforces min/max values via slider constraints

## Performance Considerations

- **Debouncing**: Slider updates are immediate (consider adding debouncing for production)
- **Lazy Loading**: Accordion items render content only when expanded
- **Memoization**: Consider using `useMemo` for expensive calculations in production

## Testing

### Unit Tests

```tsx
import { render, screen, fireEvent } from "@testing-library/react";
import { PerSymbolSettings } from "./PerSymbolSettings";

describe("PerSymbolSettings", () => {
  it("renders all symbols", () => {
    render(<PerSymbolSettings />);
    expect(screen.getByText("BTCUSDT")).toBeInTheDocument();
    expect(screen.getByText("ETHUSDT")).toBeInTheDocument();
    expect(screen.getByText("BNBUSDT")).toBeInTheDocument();
    expect(screen.getByText("SOLUSDT")).toBeInTheDocument();
  });

  it("calculates risk level correctly", () => {
    render(<PerSymbolSettings />);
    // Test risk level calculations
  });

  it("saves configurations", async () => {
    const onUpdate = jest.fn();
    render(<PerSymbolSettings onSettingsUpdate={onUpdate} />);

    const saveButton = screen.getByText("Save All");
    fireEvent.click(saveButton);

    expect(onUpdate).toHaveBeenCalled();
  });
});
```

### Integration Tests

```tsx
import { render, screen } from "@testing-library/react";
import { TradingDashboard } from "./TradingDashboard";
import { usePaperTrading } from "@/hooks/usePaperTrading";

jest.mock("@/hooks/usePaperTrading");

describe("TradingDashboard Integration", () => {
  it("integrates with usePaperTrading hook", () => {
    (usePaperTrading as jest.Mock).mockReturnValue({
      portfolio: { current_balance: 10000 },
      isLoading: false,
    });

    render(<TradingDashboard />);
    expect(screen.getByText("Per-Symbol Settings")).toBeInTheDocument();
  });
});
```

## Future Enhancements

- [ ] Add symbol search/filter functionality
- [ ] Support for custom symbols beyond the default 4
- [ ] Historical performance per symbol
- [ ] Copy settings from one symbol to another
- [ ] Import/export configuration profiles
- [ ] Batch edit multiple symbols
- [ ] Advanced risk analytics charts
- [ ] Symbol-specific strategy selection

## Troubleshooting

### Settings Not Saving
- Check browser console for API errors
- Verify backend endpoint is accessible
- Ensure CORS is properly configured

### Risk Level Not Updating
- Risk level updates automatically when leverage or position size changes
- Check that state updates are triggering re-renders

### Preset Not Applying
- Ensure preset exists in PRESETS configuration
- Check toast notifications for error messages

## References

- **Spec**: `FR-PAPER-002` - Per-Symbol Configuration
- **Design Doc**: `specs/02-design/2.5-components/COMP-RUST-TRADING.md#symbol-settings`
- **Test Cases**: `TC-PAPER-005`, `TC-PAPER-006`
- **Related Components**:
  - `usePaperTrading` hook
  - `BotSettings` component
  - `TradingSettings` component

## License

Part of Bot-Core trading system - Production-ready cryptocurrency trading bot.
