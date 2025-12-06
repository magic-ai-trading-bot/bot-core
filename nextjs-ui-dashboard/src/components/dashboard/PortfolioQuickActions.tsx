import { PremiumButton } from "@/styles/luxury-design-system";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import {
  TrendingUp,
  TrendingDown,
  DollarSign,
  RefreshCw,
  AlertCircle,
  Target,
} from "lucide-react";
import { toast } from "sonner";

export function PortfolioQuickActions() {
  const handleQuickAction = (action: string) => {
    toast.success(`${action} action triggered!`, {
      description: "This feature will be available in production mode.",
    });
  };

  const quickActions = [
    {
      id: "buy-btc",
      label: "Quick Buy BTC",
      description: "Market order at current price",
      icon: TrendingUp,
      color: "profit",
      action: () => handleQuickAction("Quick Buy BTC"),
    },
    {
      id: "sell-btc",
      label: "Quick Sell BTC",
      description: "Market order at current price",
      icon: TrendingDown,
      color: "loss",
      action: () => handleQuickAction("Quick Sell BTC"),
    },
    {
      id: "take-profit",
      label: "Take Profit All",
      description: "Close all profitable positions",
      icon: DollarSign,
      color: "warning",
      action: () => handleQuickAction("Take Profit All"),
    },
    {
      id: "stop-loss",
      label: "Emergency Stop",
      description: "Close all positions immediately",
      icon: AlertCircle,
      color: "destructive",
      action: () => handleQuickAction("Emergency Stop"),
    },
    {
      id: "rebalance",
      label: "Rebalance Portfolio",
      description: "Auto-adjust position sizes",
      icon: RefreshCw,
      color: "info",
      action: () => handleQuickAction("Rebalance Portfolio"),
    },
    {
      id: "set-targets",
      label: "Set TP/SL Targets",
      description: "Configure take profit & stop loss",
      icon: Target,
      color: "primary",
      action: () => handleQuickAction("Set TP/SL Targets"),
    },
  ];

  return (
    <Card>
      <CardHeader>
        <CardTitle className="text-lg">⚡ Quick Actions</CardTitle>
        <p className="text-sm text-muted-foreground">
          One-click trading actions for faster execution
        </p>
      </CardHeader>
      <CardContent>
        <div className="grid grid-cols-2 md:grid-cols-3 gap-3">
          {quickActions.map((action) => {
            const Icon = action.icon;
            return (
              <PremiumButton
                key={action.id}
                variant="secondary"
                className={`h-auto flex-col gap-2 p-4 focus-custom touch-target ${
                  action.color === "profit"
                    ? "hover:bg-profit/10 hover:border-profit/50"
                    : action.color === "loss"
                    ? "hover:bg-loss/10 hover:border-loss/50"
                    : action.color === "warning"
                    ? "hover:bg-warning/10 hover:border-warning/50"
                    : action.color === "destructive"
                    ? "hover:bg-destructive/10 hover:border-destructive/50"
                    : "hover:bg-info/10 hover:border-info/50"
                }`}
                onClick={action.action}
              >
                <Icon
                  className={`h-6 w-6 ${
                    action.color === "profit"
                      ? "text-profit"
                      : action.color === "loss"
                      ? "text-loss"
                      : action.color === "warning"
                      ? "text-warning"
                      : action.color === "destructive"
                      ? "text-destructive"
                      : "text-info"
                  }`}
                  aria-hidden="true"
                />
                <div className="text-center">
                  <p className="font-semibold text-xs">{action.label}</p>
                  <p className="text-[10px] text-muted-foreground leading-tight mt-1">
                    {action.description}
                  </p>
                </div>
              </PremiumButton>
            );
          })}
        </div>

        {/* Warning Notice */}
        <div className="mt-4 p-3 bg-warning/10 border border-warning/20 rounded-md">
          <div className="flex items-start gap-2">
            <AlertCircle className="h-4 w-4 text-warning flex-shrink-0 mt-0.5" aria-hidden="true" />
            <div className="text-xs text-muted-foreground">
              <strong className="text-warning">Trading Disabled:</strong> Enable
              production mode in Settings to activate live trading. Always use testnet
              first!
            </div>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
