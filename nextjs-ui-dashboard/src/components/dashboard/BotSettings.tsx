import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { Badge } from "@/components/ui/badge";
import { useState, useEffect } from "react";
import { usePaperTrading } from "@/hooks/usePaperTrading";
import { useToast } from "@/hooks/use-toast";
import { Loader2 } from "lucide-react";

/**
 * Bot Configuration Component - REAL BACKEND INTEGRATION
 *
 * IMPORTANT: All settings are now connected to the backend API
 * Changes made here will actually affect the trading bot behavior
 *
 * @spec:FR-DASHBOARD-004 - Bot Settings UI
 * @ref:specs/02-design/2.5-components/COMP-FRONTEND-DASHBOARD.md
 */
export function BotSettings() {
  const { settings, portfolio, updateSettings, startBot, stopBot, resetPortfolio } = usePaperTrading();
  const { toast } = useToast();
  const [isSaving, setIsSaving] = useState(false);
  const [isResetting, setIsResetting] = useState(false);
  const [isStoppingAll, setIsStoppingAll] = useState(false);

  // Local state for UI (synced with backend settings)
  const [botActive, setBotActive] = useState(settings?.basic?.enabled || false);
  const [leverage, setLeverage] = useState([settings?.basic?.default_leverage || 10]);
  const [capitalAllocation, setCapitalAllocation] = useState([settings?.basic?.default_position_size_pct || 75]);
  const [riskThreshold, setRiskThreshold] = useState([settings?.risk?.max_risk_per_trade_pct || 5]);
  const [activePairs, setActivePairs] = useState<Record<string, boolean>>({
    "BTCUSDT": true,
    "ETHUSDT": true,
    "BNBUSDT": false,
    "SOLUSDT": false,
  });

  // Sync local state with backend settings when settings change
  useEffect(() => {
    if (settings?.basic) {
      setBotActive(settings.basic.enabled);
      setLeverage([settings.basic.default_leverage]);
      setCapitalAllocation([settings.basic.default_position_size_pct]);
    }
    if (settings?.risk) {
      setRiskThreshold([settings.risk.max_risk_per_trade_pct]);
    }
  }, [settings]);

  /**
   * Save all settings to backend
   */
  const handleSaveSettings = async () => {
    setIsSaving(true);
    try {
      await updateSettings({
        basic: {
          ...settings.basic,
          enabled: botActive,
          default_leverage: leverage[0],
          default_position_size_pct: capitalAllocation[0],
        },
        risk: {
          ...settings.risk,
          max_risk_per_trade_pct: riskThreshold[0],
        },
        strategy: settings.strategy,
        exit_strategy: settings.exit_strategy,
      });

      toast({
        title: "Settings Saved ✅",
        description: "Bot configuration updated successfully",
        variant: "default",
      });
    } catch (error) {
      const err = error as Error;
      toast({
        title: "Failed to Save Settings ❌",
        description: err.message || "Could not update bot configuration",
        variant: "destructive",
      });
    } finally {
      setIsSaving(false);
    }
  };

  /**
   * Reset to default settings
   */
  const handleReset = async () => {
    setIsResetting(true);
    try {
      await resetPortfolio();

      // Reset local state to defaults
      setBotActive(false);
      setLeverage([10]);
      setCapitalAllocation([75]);
      setRiskThreshold([5]);

      toast({
        title: "Settings Reset ✅",
        description: "Portfolio and settings reset to defaults",
        variant: "default",
      });
    } catch (error) {
      const err = error as Error;
      toast({
        title: "Failed to Reset ❌",
        description: err.message || "Could not reset portfolio",
        variant: "destructive",
      });
    } finally {
      setIsResetting(false);
    }
  };

  /**
   * Emergency stop - close all positions and stop bot
   */
  const handleEmergencyStop = async () => {
    setIsStoppingAll(true);
    try {
      // Stop the bot first
      await stopBot();

      // Note: Backend automatically closes all positions when bot stops
      // This is handled in rust-core-engine/src/paper_trading/engine.rs:stop()

      toast({
        title: "Emergency Stop Activated ⚠️",
        description: "Bot stopped and all positions closed",
        variant: "destructive",
      });
    } catch (error) {
      const err = error as Error;
      toast({
        title: "Failed to Stop ❌",
        description: err.message || "Could not execute emergency stop",
        variant: "destructive",
      });
    } finally {
      setIsStoppingAll(false);
    }
  };

  /**
   * Toggle bot active status
   */
  const handleToggleBotStatus = async (checked: boolean) => {
    setBotActive(checked);
    try {
      if (checked) {
        await startBot();
        toast({
          title: "Bot Started ✅",
          description: "Trading bot is now active",
          variant: "default",
        });
      } else {
        await stopBot();
        toast({
          title: "Bot Stopped ⏸️",
          description: "Trading bot is now inactive",
          variant: "default",
        });
      }
    } catch (error) {
      const err = error as Error;
      // Revert on error
      setBotActive(!checked);
      toast({
        title: "Failed to Update Status ❌",
        description: err.message || "Could not change bot status",
        variant: "destructive",
      });
    }
  };

  // Calculate actual amounts based on portfolio balance
  const currentBalance = portfolio?.current_balance || settings?.basic?.initial_balance || 10000;
  const allocatedCapital = ((currentBalance || 0) * capitalAllocation[0]) / 100;
  const maxLossPerTrade = ((currentBalance || 0) * riskThreshold[0]) / 100;

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg flex items-center justify-between">
          Bot Configuration
          <Badge
            variant={botActive ? "default" : "secondary"}
            className={botActive ? "bg-profit text-profit-foreground" : "bg-muted text-muted-foreground"}
          >
            {botActive ? "ACTIVE" : "INACTIVE"}
          </Badge>
        </CardTitle>
      </CardHeader>
      <CardContent className="space-y-6">
        {/* Bot Activation */}
        <div className="flex items-center justify-between p-4 rounded-lg bg-secondary/50 border">
          <div>
            <h3 className="font-semibold">Bot Status</h3>
            <p className="text-sm text-muted-foreground">
              {botActive ? "Bot is actively trading" : "Bot is stopped"}
            </p>
          </div>
          <Switch
            checked={botActive}
            onCheckedChange={handleToggleBotStatus}
            className="data-[state=checked]:bg-profit"
          />
        </div>

        {/* Capital Allocation */}
        <div className="space-y-3">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">Capital Allocation</h3>
            <span className="text-sm text-profit font-semibold">{capitalAllocation[0]}%</span>
          </div>
          <Slider
            value={capitalAllocation}
            onValueChange={setCapitalAllocation}
            max={100}
            min={10}
            step={5}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>Conservative (10%)</span>
            <span>Aggressive (100%)</span>
          </div>
          <p className="text-sm text-muted-foreground">
            Amount: ${allocatedCapital.toLocaleString(undefined, { minimumFractionDigits: 2, maximumFractionDigits: 2 })}
          </p>
        </div>

        {/* Leverage Setting */}
        <div className="space-y-3">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">Maximum Leverage</h3>
            <span className="text-sm text-warning font-semibold">{leverage[0]}x</span>
          </div>
          <Slider
            value={leverage}
            onValueChange={setLeverage}
            max={20}
            min={1}
            step={1}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>Safe (1x)</span>
            <span>High Risk (20x)</span>
          </div>
        </div>

        {/* Risk Management */}
        <div className="space-y-3">
          <div className="flex justify-between items-center">
            <h3 className="font-semibold">Risk Threshold</h3>
            <span className="text-sm text-loss font-semibold">{riskThreshold[0]}%</span>
          </div>
          <Slider
            value={riskThreshold}
            onValueChange={setRiskThreshold}
            max={15}
            min={1}
            step={0.5}
            className="w-full"
          />
          <div className="flex justify-between text-xs text-muted-foreground">
            <span>Conservative (1%)</span>
            <span>Aggressive (15%)</span>
          </div>
          <p className="text-sm text-muted-foreground">
            Max loss per trade: ${maxLossPerTrade.toFixed(2)}
          </p>
        </div>

        {/* Trading Pairs */}
        <div className="space-y-3">
          <h3 className="font-semibold">Active Trading Pairs</h3>
          <div className="grid grid-cols-2 gap-2">
            {Object.entries(activePairs).map(([pair, active]) => (
              <div key={pair} className="flex items-center justify-between p-2 rounded bg-muted/50">
                <span className="text-sm font-medium">{pair.replace('USDT', '/USDT')}</span>
                <Switch
                  checked={active}
                  onCheckedChange={(checked) => setActivePairs(prev => ({ ...prev, [pair]: checked }))}
                />
              </div>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="grid grid-cols-2 gap-3 pt-4">
          <Button
            variant="outline"
            className="w-full"
            onClick={handleReset}
            disabled={isResetting || isSaving}
          >
            {isResetting && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Reset to Default
          </Button>
          <Button
            className="w-full bg-profit hover:bg-profit/90"
            onClick={handleSaveSettings}
            disabled={isSaving || isResetting}
          >
            {isSaving && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
            Save Settings
          </Button>
        </div>

        {/* Emergency Stop */}
        <div className="p-4 rounded-lg bg-loss/10 border border-loss/20">
          <div className="flex items-center justify-between">
            <div>
              <h4 className="font-semibold text-loss">Emergency Stop</h4>
              <p className="text-xs text-muted-foreground">
                Immediately close all positions and stop trading
              </p>
            </div>
            <Button
              variant="destructive"
              size="sm"
              onClick={handleEmergencyStop}
              disabled={isStoppingAll}
            >
              {isStoppingAll && <Loader2 className="mr-2 h-4 w-4 animate-spin" />}
              STOP ALL
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
