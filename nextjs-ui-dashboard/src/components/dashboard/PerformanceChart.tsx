import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import {
  LineChart,
  Line,
  XAxis,
  YAxis,
  CartesianGrid,
  Tooltip,
  ResponsiveContainer,
} from "recharts";

export function PerformanceChart() {
  const mockData = {
    daily: [
      { date: "12/01", pnl: 0, balance: 10000 },
      { date: "12/02", pnl: 150, balance: 10150 },
      { date: "12/03", pnl: -75, balance: 10075 },
      { date: "12/04", pnl: 280, balance: 10355 },
      { date: "12/05", pnl: 420, balance: 10775 },
      { date: "12/06", pnl: 180, balance: 10955 },
      { date: "12/07", pnl: 650, balance: 11605 },
    ],
    weekly: [
      { date: "Week 1", pnl: 0, balance: 10000 },
      { date: "Week 2", pnl: 420, balance: 10420 },
      { date: "Week 3", pnl: 780, balance: 11200 },
      { date: "Week 4", pnl: 1200, balance: 12200 },
    ],
    monthly: [
      { date: "Nov", pnl: 0, balance: 10000 },
      { date: "Dec", pnl: 1200, balance: 11200 },
      { date: "Jan", pnl: 2450, balance: 12450 },
    ],
  };

  const stats = {
    totalTrades: 147,
    winRate: 67.3,
    avgProfit: 23.45,
    totalPnL: 2450.32,
  };

  const CustomTooltip = ({
    active,
    payload,
    label,
  }: {
    active?: boolean;
    payload?: Array<{ value: number; payload: { pnl: number } }>;
    label?: string;
  }) => {
    if (active && payload && payload.length) {
      return (
        <div className="bg-card border border-border rounded-lg p-3 shadow-lg">
          <p className="text-sm font-medium">{label}</p>
          <p className="text-sm text-profit">
            Balance: ${payload[0]?.value?.toLocaleString()}
          </p>
          <p className="text-sm text-info">
            P&L: ${(payload[0]?.payload?.pnl || 0).toLocaleString()}
          </p>
        </div>
      );
    }
    return null;
  };

  return (
    <div className="grid grid-cols-1 lg:grid-cols-3 gap-6">
      {/* Performance Chart */}
      <Card className="lg:col-span-2">
        <CardHeader>
          <CardTitle className="text-lg">Performance Overview</CardTitle>
        </CardHeader>
        <CardContent>
          <Tabs defaultValue="daily" className="space-y-4">
            <TabsList className="grid w-full grid-cols-3">
              <TabsTrigger value="daily">Daily</TabsTrigger>
              <TabsTrigger value="weekly">Weekly</TabsTrigger>
              <TabsTrigger value="monthly">Monthly</TabsTrigger>
            </TabsList>

            {Object.entries(mockData).map(([period, data]) => (
              <TabsContent key={period} value={period} className="space-y-4">
                <div className="h-64">
                  <ResponsiveContainer width="100%" height="100%">
                    <LineChart data={data}>
                      <CartesianGrid
                        strokeDasharray="3 3"
                        stroke="hsl(var(--border))"
                      />
                      <XAxis
                        dataKey="date"
                        stroke="hsl(var(--muted-foreground))"
                        fontSize={12}
                      />
                      <YAxis
                        stroke="hsl(var(--muted-foreground))"
                        fontSize={12}
                      />
                      <Tooltip content={<CustomTooltip />} />
                      <Line
                        type="monotone"
                        dataKey="balance"
                        stroke="hsl(var(--profit))"
                        strokeWidth={3}
                        dot={{
                          fill: "hsl(var(--profit))",
                          strokeWidth: 2,
                          r: 4,
                        }}
                        activeDot={{
                          r: 6,
                          stroke: "hsl(var(--profit))",
                          strokeWidth: 2,
                        }}
                      />
                    </LineChart>
                  </ResponsiveContainer>
                </div>
              </TabsContent>
            ))}
          </Tabs>
        </CardContent>
      </Card>

      {/* Statistics */}
      <Card>
        <CardHeader>
          <CardTitle className="text-lg">Performance Metrics</CardTitle>
        </CardHeader>
        <CardContent className="space-y-6">
          <div className="space-y-4">
            <div className="text-center p-4 rounded-lg bg-gradient-to-br from-profit/10 to-profit/5 border border-profit/20">
              <div className="text-2xl font-bold text-profit">
                ${stats.totalPnL.toLocaleString()}
              </div>
              <div className="text-sm text-muted-foreground">Total P&L</div>
            </div>

            <div className="grid grid-cols-1 gap-4">
              <div className="text-center p-3 rounded-lg bg-secondary/50">
                <div className="text-xl font-bold">{stats.totalTrades}</div>
                <div className="text-xs text-muted-foreground">
                  Total Trades
                </div>
              </div>

              <div className="text-center p-3 rounded-lg bg-secondary/50">
                <div className="text-xl font-bold text-profit">
                  {stats.winRate}%
                </div>
                <div className="text-xs text-muted-foreground">Win Rate</div>
              </div>

              <div className="text-center p-3 rounded-lg bg-secondary/50">
                <div className="text-xl font-bold text-info">
                  ${stats.avgProfit}
                </div>
                <div className="text-xs text-muted-foreground">Avg Profit</div>
              </div>
            </div>
          </div>

          {/* Win Rate Progress */}
          <div className="space-y-2">
            <div className="flex justify-between text-sm">
              <span>Win Rate Progress</span>
              <span className="text-profit font-semibold">
                {stats.winRate}%
              </span>
            </div>
            <div className="w-full bg-muted rounded-full h-2">
              <div
                className="bg-profit h-2 rounded-full transition-all duration-1000"
                style={{ width: `${stats.winRate}%` }}
              ></div>
            </div>
          </div>
        </CardContent>
      </Card>
    </div>
  );
}
