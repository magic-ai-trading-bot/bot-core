import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Button } from "@/components/ui/button";
import { Switch } from "@/components/ui/switch";
import { Slider } from "@/components/ui/slider";
import { Badge } from "@/components/ui/badge";
import { useState } from "react";

export function BotSettings() {
  const [botActive, setBotActive] = useState(true);
  const [leverage, setLeverage] = useState([10]);
  const [capitalAllocation, setCapitalAllocation] = useState([75]);
  const [riskThreshold, setRiskThreshold] = useState([5]);

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
            onCheckedChange={setBotActive}
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
            Amount: ${((12450 * capitalAllocation[0]) / 100).toLocaleString()}
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
            Max loss per trade: ${((12450 * riskThreshold[0]) / 100).toFixed(2)}
          </p>
        </div>

        {/* Trading Pairs */}
        <div className="space-y-3">
          <h3 className="font-semibold">Active Trading Pairs</h3>
          <div className="grid grid-cols-2 gap-2">
            {["BTC/USDT", "ETH/USDT", "BNB/USDT", "SOL/USDT"].map((pair) => (
              <div key={pair} className="flex items-center justify-between p-2 rounded bg-muted/50">
                <span className="text-sm font-medium">{pair}</span>
                <Switch defaultChecked={pair === "BTC/USDT" || pair === "ETH/USDT"} />
              </div>
            ))}
          </div>
        </div>

        {/* Action Buttons */}
        <div className="grid grid-cols-2 gap-3 pt-4">
          <Button variant="outline" className="w-full">
            Reset to Default
          </Button>
          <Button className="w-full bg-profit hover:bg-profit/90">
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
            <Button variant="destructive" size="sm">
              STOP ALL
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}