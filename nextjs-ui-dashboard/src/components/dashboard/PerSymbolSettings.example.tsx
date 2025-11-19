/**
 * Example Integration of PerSymbolSettings Component
 *
 * This file demonstrates how to integrate the PerSymbolSettings component
 * into your dashboard and connect it with the usePaperTrading hook.
 */

import { PerSymbolSettings, SymbolConfig } from "./PerSymbolSettings";
import { usePaperTrading } from "@/hooks/usePaperTrading";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";

export function TradingDashboardWithSymbolSettings() {
  const {
    portfolio,
    settings,
    updateSettings,
    isLoading,
  } = usePaperTrading();

  /**
   * Handle symbol settings updates
   * This will be called when user saves symbol configurations
   */
  const handleSymbolSettingsUpdate = async (configs: SymbolConfig[]) => {
    // You can add additional logic here before/after updating
    // Symbol configs updated: configs

    // Optionally trigger a refresh of trading data
    // refreshData();
  };

  return (
    <div className="space-y-6">
      {/* Other dashboard components */}
      <Card>
        <CardHeader>
          <CardTitle>Portfolio Overview</CardTitle>
        </CardHeader>
        <CardContent>
          <div className="grid grid-cols-2 gap-4">
            <div>
              <p className="text-sm text-muted-foreground">Current Balance</p>
              <p className="text-2xl font-bold">
                ${portfolio.current_balance.toLocaleString()}
              </p>
            </div>
            <div>
              <p className="text-sm text-muted-foreground">Equity</p>
              <p className="text-2xl font-bold">
                ${portfolio.equity.toLocaleString()}
              </p>
            </div>
          </div>
        </CardContent>
      </Card>

      {/* Per-Symbol Settings Component */}
      <PerSymbolSettings
        currentBalance={portfolio.current_balance}
        onSettingsUpdate={handleSymbolSettingsUpdate}
      />
    </div>
  );
}

/**
 * Example: Using in Settings Page
 */
export function SettingsPageExample() {
  const { portfolio } = usePaperTrading();

  return (
    <div className="container mx-auto p-6 space-y-6">
      <div>
        <h1 className="text-3xl font-bold mb-2">Trading Settings</h1>
        <p className="text-muted-foreground">
          Configure your per-symbol trading parameters
        </p>
      </div>

      <PerSymbolSettings
        currentBalance={portfolio.current_balance}
        onSettingsUpdate={(configs) => {
          // Settings updated: configs
        }}
      />
    </div>
  );
}

/**
 * Example: Standalone Usage Without Hook
 */
export function StandaloneExample() {
  const currentBalance = 10000; // Or from your state management

  return (
    <PerSymbolSettings
      currentBalance={currentBalance}
      onSettingsUpdate={(configs) => {
        // Handle the update
        // Configs: configs
      }}
    />
  );
}
